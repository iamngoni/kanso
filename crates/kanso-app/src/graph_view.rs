//! Drawing the note graph: the inspector's local graph (the open note and
//! the notes one or two links away) and the whole library's graph, opened
//! from the palette or the View menu over the window.
//!
//! The graph is read from the store and laid out off the main thread
//! (`graph.rs`), then drawn on a canvas: edges as one path, nodes as
//! circles tinted by notebook, the open note in the accent. Only what is
//! in view is drawn, and labels thin out as the view zooms out, so a
//! library of thousands of notes stays smooth. Pointing at a note lights
//! it and its neighbours; clicking one opens it; dragging moves the view;
//! scrolling zooms (with a mouse wheel, or ⌘ held on a trackpad) or moves
//! it (a trackpad).

use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::Theme;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenuItem};
use gpui_kit::component::{Sizable as _, StyledExt as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_store::Store;

use crate::graph::{self, Graph, Viewport, arrange, hit, screen_radius, segment_visible};
pub use crate::graph::{GraphFilter, Scope, book_color, in_notebook};
use crate::icon_themes::{Slot, library_icon};
use crate::layout::{BOTTOM_BAND_HEIGHT, TOP_BAND_HEIGHT, inset_after};
use crate::library::{Library, LibraryEvent};
use crate::note_list::quiet_empty;
use crate::theme::Colors;
use crate::workspace::Workspace;
use crate::zoom::px;

actions!(kanso, [ShowGraph, ShowLocalGraph, ShowRelated, CloseGraph]);
// The zoom keys zoom the graph while it has focus, not the whole app.
actions!(kanso, [GraphZoomIn, GraphZoomOut, GraphShowAll]);

/// How far one press of a zoom button or key zooms the graph.
const GRAPH_ZOOM_STEP: f32 = 1.4;

/// How long the library stays still before a shown graph is read again.
const RELOAD_DELAY: Duration = Duration::from_millis(600);
/// The most labels drawn at once in the library's graph.
const LABEL_LIMIT: usize = 90;
/// Labels are cut to this many characters.
const LABEL_CHARS: usize = 26;

pub fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("escape", CloseGraph, Some("GraphView")),
        KeyBinding::new("secondary-=", GraphZoomIn, Some("GraphView")),
        KeyBinding::new("secondary-+", GraphZoomIn, Some("GraphView")),
        KeyBinding::new("secondary--", GraphZoomOut, Some("GraphView")),
        KeyBinding::new("secondary-0", GraphShowAll, Some("GraphView")),
    ]);
}

/// What a laid-out graph was made for: when this changes, it is made
/// again.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Key {
    /// The local graph's middle; `None` for the library's.
    center: Option<String>,
    hops: u8,
    filter: GraphFilter,
    /// The lock password has opened the locked notes: they are in the graph.
    /// While it is closed they and their links are left out.
    unlocked: bool,
}

/// A graph laid out, ready to draw.
struct Shown {
    key: Key,
    graph: Graph,
    pos: Vec<[f32; 2]>,
    radii: Vec<f32>,
}

/// A press on the graph: where, and whether it has become a drag.
#[derive(Clone, Copy)]
struct Press {
    at: [f32; 2],
    last: [f32; 2],
    moved: bool,
}

pub struct GraphView {
    store: Arc<Store>,
    library: Entity<Library>,
    pub scope: Scope,
    /// Drawn on screen: only then is the graph read and laid out.
    pub visible: bool,
    /// The open note: the local graph's middle, emphasized in both.
    center: Option<String>,
    pub hops: u8,
    pub filter: GraphFilter,
    /// The library changed since the graph was read.
    stale: bool,
    shown: Option<Arc<Shown>>,
    loading: Option<(Key, Task<()>)>,
    reload: Option<Task<()>>,
    view: Rc<Cell<Viewport>>,
    /// Fit the view to the graph at the next paint.
    fit: Rc<Cell<bool>>,
    bounds: Rc<Cell<Bounds<Pixels>>>,
    hover: Option<usize>,
    press: Option<Press>,
    focus: FocusHandle,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<()> for GraphView {}

impl Focusable for GraphView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl GraphView {
    pub fn new(
        store: Arc<Store>,
        library: Entity<Library>,
        scope: Scope,
        hops: u8,
        cx: &mut Context<Self>,
    ) -> Self {
        let subscriptions = vec![cx.subscribe(&library, |this, _, event, cx| {
            if let LibraryEvent::Changed(change) = event
                && change.id.starts_with("note:")
            {
                this.stale = true;
                if this.visible {
                    this.schedule_reload(cx);
                }
            }
        })];
        GraphView {
            store,
            library,
            scope,
            visible: false,
            center: None,
            hops: hops.clamp(1, 2),
            filter: GraphFilter::All,
            stale: true,
            shown: None,
            loading: None,
            reload: None,
            view: Rc::new(Cell::new(Viewport::default())),
            fit: Rc::new(Cell::new(true)),
            bounds: Rc::new(Cell::new(Bounds::default())),
            hover: None,
            press: None,
            focus: cx.focus_handle(),
            _subscriptions: subscriptions,
        }
    }

    /// The note the graph is about: the local graph's middle.
    pub fn set_center(&mut self, id: Option<String>, cx: &mut Context<Self>) {
        if self.center != id {
            self.center = id;
            self.hover = None;
            cx.notify();
        }
    }

    pub fn set_hops(&mut self, hops: u8, cx: &mut Context<Self>) {
        let hops = hops.clamp(1, 2);
        if self.hops != hops {
            self.hops = hops;
            cx.notify();
        }
    }

    pub fn set_filter(&mut self, filter: GraphFilter, cx: &mut Context<Self>) {
        if self.filter != filter {
            self.filter = filter;
            self.hover = None;
            cx.notify();
        }
    }

    /// The graph as last laid out: how many notes and links it shows.
    pub fn counts(&self) -> Option<(usize, usize)> {
        self.shown
            .as_ref()
            .map(|s| (s.graph.len(), s.graph.link_count()))
    }

    fn key(&self, cx: &App) -> Key {
        let unlocked = crate::locked::LockKeys::is_unlocked(cx);
        match self.scope {
            Scope::Local => Key {
                center: self.center.clone(),
                hops: self.hops,
                filter: GraphFilter::All,
                unlocked,
            },
            Scope::Library => Key {
                center: None,
                hops: 0,
                filter: self.filter.clone(),
                unlocked,
            },
        }
    }

    /// Reads the graph again once the library stays still.
    fn schedule_reload(&mut self, cx: &mut Context<Self>) {
        self.reload = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(RELOAD_DELAY).await;
            this.update(cx, |this, cx| {
                this.reload = None;
                cx.notify();
            })
            .ok();
        }));
    }

    /// Starts reading and laying out the graph when what is shown isn't
    /// what should be.
    fn ensure_loaded(&mut self, cx: &mut Context<Self>) {
        let key = self.key(cx);
        let current = self.shown.as_ref().is_some_and(|s| s.key == key) && !self.stale;
        let underway = self.loading.as_ref().is_some_and(|(k, _)| *k == key);
        if current || underway || self.reload.is_some() {
            return;
        }
        if self.scope == Scope::Local && key.center.is_none() {
            self.shown = None;
            self.loading = None;
            return;
        }
        let store = self.store.clone();
        let books: Vec<(String, Option<String>)> = self
            .library
            .read(cx)
            .books
            .iter()
            .map(|b| (b.id.clone(), b.parent_book_id.clone()))
            .collect();
        let scope = self.scope;
        let for_key = key.clone();
        let work = cx.background_executor().spawn(async move {
            let full = graph::read(&store, for_key.unlocked).unwrap_or_default();
            let arranged = arrange(
                full,
                scope,
                for_key.center.as_deref(),
                for_key.hops,
                &for_key.filter,
                &books,
            );
            Shown {
                key: for_key,
                graph: arranged.graph,
                pos: arranged.pos,
                radii: arranged.radii,
            }
        });
        let task = cx.spawn(async move |this, cx| {
            let shown = work.await;
            this.update(cx, |this, cx| {
                if this.loading.as_ref().is_some_and(|(k, _)| *k == shown.key) {
                    this.loading = None;
                    this.stale = this.stale && shown.key != this.key(cx);
                    let new_place = this.shown.as_ref().is_none_or(|s| s.key != shown.key);
                    if new_place {
                        this.fit.set(true);
                        this.hover = None;
                    }
                    this.shown = Some(Arc::new(shown));
                    cx.notify();
                }
            })
            .ok();
        });
        self.stale = false;
        self.loading = Some((key, task));
    }

    /// Fits the whole graph in view.
    pub fn fit(&mut self, cx: &mut Context<Self>) {
        self.fit.set(true);
        cx.notify();
    }

    /// Zooms by `factor` about the middle of the view.
    pub fn zoom_by(&mut self, factor: f32, cx: &mut Context<Self>) {
        let size = self.size();
        let view = self.view.get();
        self.view
            .set(view.zoom_at(factor, [size[0] / 2., size[1] / 2.], size));
        cx.notify();
    }

    fn size(&self) -> [f32; 2] {
        let b = self.bounds.get();
        [f32::from(b.size.width), f32::from(b.size.height)]
    }

    /// `position` (in the window) in the canvas.
    fn local(&self, position: Point<Pixels>) -> [f32; 2] {
        let b = self.bounds.get();
        [
            f32::from(position.x - b.origin.x),
            f32::from(position.y - b.origin.y),
        ]
    }

    fn node_at(&self, at: [f32; 2]) -> Option<usize> {
        let shown = self.shown.as_ref()?;
        hit(
            &shown.pos,
            &shown.radii,
            &self.view.get(),
            self.size(),
            at,
            4.,
        )
    }

    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if event.button != MouseButton::Left {
            return;
        }
        if self.scope == Scope::Library {
            window.focus(&self.focus, cx);
        }
        let at = self.local(event.position);
        self.press = Some(Press {
            at,
            last: at,
            moved: false,
        });
        cx.stop_propagation();
    }

    fn on_mouse_move(&mut self, event: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        let at = self.local(event.position);
        if let Some(press) = &mut self.press {
            if event.pressed_button != Some(MouseButton::Left) {
                self.press = None;
            } else {
                let far = (at[0] - press.at[0]).abs() + (at[1] - press.at[1]).abs() > 3.;
                press.moved |= far;
                if press.moved {
                    let delta = [at[0] - press.last[0], at[1] - press.last[1]];
                    press.last = at;
                    self.view.set(self.view.get().panned(delta));
                    cx.notify();
                    return;
                }
            }
        }
        let hover = self.node_at(at);
        if hover != self.hover {
            self.hover = hover;
            cx.notify();
        }
    }

    fn on_mouse_up(&mut self, event: &MouseUpEvent, window: &mut Window, cx: &mut Context<Self>) {
        let Some(press) = self.press.take() else {
            return;
        };
        if press.moved {
            return;
        }
        let at = self.local(event.position);
        let Some(ix) = self.node_at(at) else {
            return;
        };
        let Some(id) = self.shown.as_ref().map(|s| s.graph.nodes[ix].id.clone()) else {
            return;
        };
        self.library
            .update(cx, |library, cx| library.select(Some(id), cx));
        if self.scope == Scope::Library {
            window.dispatch_action(Box::new(CloseGraph), cx);
        }
    }

    fn on_scroll(&mut self, event: &ScrollWheelEvent, _: &mut Window, cx: &mut Context<Self>) {
        let size = self.size();
        let at = self.local(event.position);
        let view = self.view.get();
        let zoom = matches!(event.delta, ScrollDelta::Lines(_))
            || event.modifiers.platform
            || event.modifiers.control;
        let delta = event.delta.pixel_delta(px(20.));
        let (dx, dy) = (f32::from(delta.x), f32::from(delta.y));
        let next = if zoom {
            view.zoom_at((dy * 0.006).exp(), at, size)
        } else {
            view.panned([dx, dy])
        };
        if next != view {
            self.view.set(next);
            cx.notify();
        }
        cx.stop_propagation();
    }

    /// The canvas the graph is drawn on.
    fn canvas(&self, cx: &App) -> impl IntoElement {
        let colors = Colors::get(cx);
        let dark = Theme::global(cx).is_dark();
        let shown = self.shown.clone();
        let shown_paint = shown.clone();
        let view = self.view.clone();
        let fit = self.fit.clone();
        let bounds_cell = self.bounds.clone();
        let hover = self.hover;
        let scope = self.scope;
        let center = self.center.clone();
        canvas(
            move |bounds, _, _| {
                bounds_cell.set(bounds);
                if let Some(shown) = &shown
                    && fit.replace(false)
                {
                    let size = [f32::from(bounds.size.width), f32::from(bounds.size.height)];
                    let (margin, max_zoom) = match scope {
                        Scope::Local => (52., 2.6),
                        Scope::Library => (60., 2.0),
                    };
                    view.set(Viewport::fit(&shown.pos, size, margin, max_zoom));
                }
                view.get()
            },
            move |bounds, view, window, cx| {
                let Some(shown) = shown_paint else {
                    return;
                };
                let center = center.as_deref().and_then(|id| shown.graph.index_of(id));
                window.with_content_mask(Some(ContentMask { bounds }), |window| {
                    paint_graph(
                        &shown, &view, bounds, hover, center, scope, colors, dark, window, cx,
                    );
                });
            },
        )
        .size_full()
    }

    fn render_toolbar(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let library = self.library.read(cx);
        let filter_label = match &self.filter {
            GraphFilter::All => "All notes".to_owned(),
            GraphFilter::Book(id) => library
                .book(id)
                .map_or_else(|| "Notebook".to_owned(), |b| b.name.clone()),
            GraphFilter::Tag(id) => library
                .tag(id)
                .map_or_else(|| "Tag".to_owned(), |t| format!("#{}", t.name)),
        };
        let books: Vec<(usize, String, String)> = library
            .book_tree()
            .into_iter()
            .map(|(depth, b)| (depth, b.id.clone(), b.name.clone()))
            .collect();
        let tags: Vec<(String, String)> = library
            .tags
            .iter()
            .map(|t| (t.id.clone(), t.name.clone()))
            .collect();
        let current = self.filter.clone();
        let this = cx.entity();
        let menu = Button::new("graph-filter")
            .outline()
            .small()
            .icon(Lucide::ListFilter)
            .label(filter_label)
            .dropdown_caret(true)
            .dropdown_menu(move |mut menu, _, _| {
                let pick = |filter: GraphFilter| {
                    let this = this.clone();
                    move |_: &ClickEvent, _: &mut Window, cx: &mut App| {
                        this.update(cx, |view, cx| view.set_filter(filter.clone(), cx));
                    }
                };
                menu = menu.item(
                    PopupMenuItem::new("All notes")
                        .checked(current == GraphFilter::All)
                        .on_click(pick(GraphFilter::All)),
                );
                if !books.is_empty() {
                    menu = menu.separator().label("Notebooks");
                    for (depth, id, name) in &books {
                        let filter = GraphFilter::Book(id.clone());
                        menu = menu.item(
                            PopupMenuItem::new(format!("{}{name}", "    ".repeat(*depth)))
                                .checked(current == filter)
                                .on_click(pick(filter)),
                        );
                    }
                }
                if !tags.is_empty() {
                    menu = menu.separator().label("Tags");
                    for (id, name) in &tags {
                        let filter = GraphFilter::Tag(id.clone());
                        menu = menu.item(
                            PopupMenuItem::new(format!("#{name}"))
                                .checked(current == filter)
                                .on_click(pick(filter)),
                        );
                    }
                }
                menu
            });
        let counts = match self.counts() {
            Some((notes, links)) => format!(
                "{} · {}",
                crate::text_stats::plural(notes, "note", "notes"),
                crate::text_stats::plural(links, "link", "links")
            ),
            None => String::new(),
        };
        let icon_button = |id: &'static str, icon: Lucide, tip: &'static str| {
            Button::new(id).ghost().small().icon(icon).tooltip(tip)
        };
        h_flex()
            .h(px(TOP_BAND_HEIGHT))
            .flex_none()
            .pl(px(inset_after(0., window.is_fullscreen())))
            .pr(px(12.))
            .gap(px(10.))
            .border_b_1()
            .border_color(colors.divider)
            .bg(colors.list)
            .child(
                div()
                    .text_size(px(14.))
                    .font_medium()
                    .text_color(colors.text)
                    .child("Graph"),
            )
            .child(
                div()
                    .text_size(px(12.5))
                    .text_color(colors.text_muted)
                    .child(counts),
            )
            .child(div().flex_1())
            .child(menu)
            .child(
                h_flex()
                    .gap(px(2.))
                    .child(
                        icon_button("graph-zoom-out", Lucide::ZoomOut, "Zoom out").on_click(
                            cx.listener(|this, _, _, cx| this.zoom_by(1. / GRAPH_ZOOM_STEP, cx)),
                        ),
                    )
                    .child(
                        icon_button("graph-zoom-in", Lucide::ZoomIn, "Zoom in").on_click(
                            cx.listener(|this, _, _, cx| this.zoom_by(GRAPH_ZOOM_STEP, cx)),
                        ),
                    )
                    .child(
                        icon_button("graph-fit", Lucide::Scan, "Show all")
                            .on_click(cx.listener(|this, _, _, cx| this.fit(cx))),
                    ),
            )
            .child(
                icon_button("graph-close", Lucide::Close, "Close (Esc)").on_click(
                    cx.listener(|_, _, window, cx| {
                        window.dispatch_action(Box::new(CloseGraph), cx)
                    }),
                ),
            )
    }

    fn render_body(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let colors = Colors::get(cx);
        if self.scope == Scope::Local && self.center.is_none() {
            return quiet_empty(
                library_icon(Slot::Graph),
                "No note open".into(),
                None,
                colors,
            )
            .into_any_element();
        }
        let Some(shown) = self.shown.clone() else {
            return quiet_empty(
                library_icon(Slot::Graph),
                "Drawing the graph…".into(),
                None,
                colors,
            )
            .into_any_element();
        };
        if shown.graph.is_empty() {
            let (title, detail) = match self.scope {
                Scope::Local => ("Drawing the graph…".to_owned(), None),
                Scope::Library if self.filter == GraphFilter::All => (
                    "No notes yet".to_owned(),
                    Some("Notes and the links between them show here.".to_owned()),
                ),
                Scope::Library => ("No notes here".to_owned(), None),
            };
            return quiet_empty(library_icon(Slot::Graph), title, detail, colors)
                .into_any_element();
        }
        let lonely = self.scope == Scope::Local && shown.graph.len() == 1;
        div()
            .id("graph-canvas")
            .relative()
            .size_full()
            .when(self.hover.is_some(), |el| el.cursor_pointer())
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_scroll_wheel(cx.listener(Self::on_scroll))
            .child(self.canvas(cx))
            .when(lonely, |el| {
                el.child(
                    div()
                        .absolute()
                        .bottom(px(14.))
                        .left_0()
                        .right_0()
                        .px(px(20.))
                        .text_center()
                        .text_size(px(12.5))
                        .text_color(colors.text_muted)
                        .child("No links yet. Type [[ to link another note."),
                )
            })
            .into_any_element()
    }
}

#[cfg(test)]
impl GraphView {
    /// How far the graph is zoomed in.
    pub fn zoom_for_test(&self) -> f32 {
        self.view.get().zoom
    }

    /// Points at the note titled `title`, as the pointer would.
    pub fn hover_for_test(&mut self, title: &str, cx: &mut Context<Self>) {
        self.hover = self
            .shown
            .as_ref()
            .and_then(|s| s.graph.nodes.iter().position(|n| n.title == title));
        cx.notify();
    }
}

impl Render for GraphView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.visible {
            self.ensure_loaded(cx);
        }
        let colors = Colors::get(cx);
        match self.scope {
            Scope::Local => div()
                .size_full()
                .child(self.render_body(cx))
                .into_any_element(),
            Scope::Library => v_flex()
                .id("graph-view")
                .key_context("GraphView")
                .track_focus(&self.focus)
                .size_full()
                .bg(colors.editor)
                .on_action(cx.listener(|this, _: &GraphZoomIn, _, cx| {
                    this.zoom_by(GRAPH_ZOOM_STEP, cx)
                }))
                .on_action(cx.listener(|this, _: &GraphZoomOut, _, cx| {
                    this.zoom_by(1. / GRAPH_ZOOM_STEP, cx)
                }))
                .on_action(cx.listener(|this, _: &GraphShowAll, _, cx| this.fit(cx)))
                .on_action(cx.listener(|this, _: &CloseGraph, window, cx| {
                    this.visible = false;
                    window.dispatch_action(Box::new(CloseGraphView), cx);
                }))
                // Presses here are the graph's, not the window's below.
                .on_any_mouse_down(|_, _, cx| cx.stop_propagation())
                .child(self.render_toolbar(window, cx))
                .child(div().flex_1().min_h_0().child(self.render_body(cx)))
                .child(
                    h_flex()
                        .h(px(BOTTOM_BAND_HEIGHT))
                        .flex_none()
                        .px(px(18.))
                        .border_t_1()
                        .border_color(colors.divider)
                        .bg(colors.list)
                        .text_size(px(12.))
                        .text_color(colors.text_muted)
                        .child(
                            "Click a note to open it · Drag to move · Scroll with ⌘ or a mouse wheel to zoom",
                        ),
                )
                .into_any_element(),
        }
    }
}

// The library graph closes in the workspace, which holds it.
actions!(kanso, [CloseGraphView]);

/// Draws `shown` through `view` in `bounds`, in the window's own pixels
/// (`gpui_kit::px`, not the app zoom's): the graph zooms itself, and the
/// pointer finds nodes in those pixels.
#[allow(clippy::too_many_arguments)]
fn paint_graph(
    shown: &Shown,
    view: &Viewport,
    bounds: Bounds<Pixels>,
    hover: Option<usize>,
    center: Option<usize>,
    scope: Scope,
    colors: &Colors,
    dark: bool,
    window: &mut Window,
    cx: &mut App,
) {
    let extent = [f32::from(bounds.size.width), f32::from(bounds.size.height)];
    let origin = bounds.origin;
    let at = |p: [f32; 2]| {
        let s = view.to_screen(p, extent);
        point(origin.x + gpui_kit::px(s[0]), origin.y + gpui_kit::px(s[1]))
    };
    let seen = view.visible(extent, 24.);
    let graph = &shown.graph;
    let focus = hover.or(center);
    let lit = |ix: usize| -> bool {
        match focus {
            Some(f) => ix == f || graph.neighbours[f].binary_search(&(ix as u32)).is_ok(),
            None => true,
        }
    };
    let dimmed = hover.is_some();

    // Edges: the quiet ones in one path, the lit ones over them in another.
    let line = if view.zoom < 0.35 { 0.75 } else { 1. };
    let mut quiet = PathBuilder::stroke(gpui_kit::px(line));
    let mut bright = PathBuilder::stroke(gpui_kit::px(1.4));
    let (mut quiet_any, mut bright_any) = (false, false);
    for &(a, b) in &graph.edges {
        if a == b {
            continue;
        }
        let (pa, pb) = (shown.pos[a as usize], shown.pos[b as usize]);
        if !segment_visible(pa, pb, seen) {
            continue;
        }
        let touches = focus.is_some_and(|f| f == a as usize || f == b as usize);
        let path = if touches { &mut bright } else { &mut quiet };
        path.move_to(at(pa));
        path.line_to(at(pb));
        if touches {
            bright_any = true;
        } else {
            quiet_any = true;
        }
    }
    let edge = Hsla::from(colors.text_muted);
    if quiet_any && let Ok(path) = quiet.build() {
        window.paint_path(path, edge.opacity(if dimmed { 0.18 } else { 0.42 }));
    }
    if bright_any && let Ok(path) = bright.build() {
        window.paint_path(path, Hsla::from(colors.accent).opacity(0.85));
    }

    // Nodes.
    let ring = Hsla::from(colors.editor);
    let mut labels: Vec<(usize, f32)> = Vec::new();
    for ix in 0..graph.len() {
        let p = shown.pos[ix];
        if p[0] < seen.0[0] || p[0] > seen.1[0] || p[1] < seen.0[1] || p[1] > seen.1[1] {
            continue;
        }
        let mut r = screen_radius(shown.radii[ix], view.zoom);
        let is_center = center == Some(ix);
        if is_center {
            r = (r * 1.35).max(6.);
        }
        let color: Hsla = if is_center {
            colors.accent.into()
        } else {
            gpui_kit::rgb(book_color(&graph.nodes[ix].book_id, dark)).into()
        };
        let color = if !dimmed || lit(ix) {
            color
        } else {
            color.opacity(0.28)
        };
        let c = at(p);
        if is_center || hover == Some(ix) {
            let halo = r + 3.;
            window.paint_quad(
                fill(
                    Bounds::new(
                        point(c.x - gpui_kit::px(halo), c.y - gpui_kit::px(halo)),
                        size2(halo * 2.),
                    ),
                    Hsla::from(colors.accent).opacity(0.25),
                )
                .corner_radii(gpui_kit::px(halo)),
            );
        }
        // A thin ring of the background keeps touching circles apart.
        if r >= 3. {
            let outer = r + 1.;
            window.paint_quad(
                fill(
                    Bounds::new(
                        point(c.x - gpui_kit::px(outer), c.y - gpui_kit::px(outer)),
                        size2(outer * 2.),
                    ),
                    ring,
                )
                .corner_radii(gpui_kit::px(outer)),
            );
        }
        window.paint_quad(
            fill(
                Bounds::new(
                    point(c.x - gpui_kit::px(r), c.y - gpui_kit::px(r)),
                    size2(r * 2.),
                ),
                color,
            )
            .corner_radii(gpui_kit::px(r)),
        );
        labels.push((ix, r));
    }

    // Labels: always for the lit notes; for the rest as room allows.
    let wants = |ix: usize| -> u8 {
        if hover == Some(ix) || center == Some(ix) {
            0
        } else if focus.is_some() && lit(ix) {
            1
        } else {
            2
        }
    };
    let all_room = match scope {
        Scope::Local => true,
        Scope::Library => view.zoom >= 0.85,
    };
    let some_room = scope == Scope::Library && view.zoom >= 0.45;
    labels.sort_by_key(|&(ix, _)| (wants(ix), std::cmp::Reverse(graph.degree(ix)), ix));
    let limit = match scope {
        Scope::Local => 60,
        Scope::Library if all_room => LABEL_LIMIT,
        Scope::Library => 24,
    };
    let font = window.text_style().font();
    let font_size = gpui_kit::px(11.);
    let mut drawn = 0;
    // Where labels went: a label that would cover another is left out,
    // unless it is the open or pointed-at note's.
    let mut placed: Vec<Bounds<Pixels>> = Vec::new();
    for (ix, r) in labels {
        let priority = wants(ix);
        if priority == 2 && (!(all_room || some_room) || dimmed) {
            continue;
        }
        if drawn >= limit && priority == 2 {
            continue;
        }
        drawn += 1;
        let title = graph.nodes[ix].title.trim();
        let mut text: String = if title.is_empty() {
            "Untitled".to_owned()
        } else {
            title.chars().take(LABEL_CHARS).collect()
        };
        if title.chars().count() > LABEL_CHARS {
            text.push('…');
        }
        let color: Hsla = if priority == 0 {
            colors.text.into()
        } else {
            colors.text_secondary.into()
        };
        let run = TextRun {
            len: text.len(),
            font: if priority == 0 {
                font.clone().bold()
            } else {
                font.clone()
            },
            color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let line = window
            .text_system()
            .shape_line(text.into(), font_size, &[run], None);
        let c = at(shown.pos[ix]);
        let width = line.width;
        // Below the dot first, then above, right and left of it; each kept
        // inside the view, so a note near the edge reads whole. A label
        // with nowhere free is left out, unless it is the open or
        // pointed-at note's.
        let inside = |left: Pixels, top: Pixels| {
            point(
                left.max(bounds.left() + gpui_kit::px(4.))
                    .min(bounds.right() - width - gpui_kit::px(4.)),
                top.max(bounds.top() + gpui_kit::px(2.))
                    .min(bounds.bottom() - gpui_kit::px(16.)),
            )
        };
        let spots = [
            inside(c.x - width / 2., c.y + gpui_kit::px(r + 3.)),
            inside(c.x - width / 2., c.y - gpui_kit::px(r + 17.)),
            inside(c.x + gpui_kit::px(r + 6.), c.y - gpui_kit::px(7.)),
            inside(c.x - gpui_kit::px(r + 6.) - width, c.y - gpui_kit::px(7.)),
        ];
        let plate_at = |at: Point<Pixels>| {
            Bounds::new(
                point(at.x - gpui_kit::px(3.), at.y - gpui_kit::px(1.)),
                size(width + gpui_kit::px(6.), gpui_kit::px(15.)),
            )
        };
        let free = spots
            .iter()
            .copied()
            .find(|&at| !placed.iter().any(|p| p.intersects(&plate_at(at))));
        let (left, top) = match free {
            Some(at) => (at.x, at.y),
            None if priority == 0 => (spots[0].x, spots[0].y),
            None => continue,
        };
        let plate = plate_at(point(left, top));
        placed.push(plate);
        // A soft plate of the background under the words.
        window.paint_quad(fill(plate, ring.opacity(0.78)).corner_radii(gpui_kit::px(3.)));
        line.paint(
            point(left, top),
            gpui_kit::px(13.),
            TextAlign::Left,
            None,
            window,
            cx,
        )
        .ok();
    }
}

fn size2(side: f32) -> Size<Pixels> {
    size(gpui_kit::px(side), gpui_kit::px(side))
}

/// The workspace's part: opening and closing the library's graph.
impl Workspace {
    pub(crate) fn show_graph(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.close_settings(window, cx);
        self.close_board(window, cx);
        self.set_focus_mode(false, window, cx);
        let graph = match &self.graph {
            Some(graph) => graph.clone(),
            None => {
                let store = cx.global::<crate::workspace::AppState>().store.clone();
                let library = self.library.clone();
                let graph = cx.new(|cx| GraphView::new(store, library, Scope::Library, 1, cx));
                self.graph = Some(graph.clone());
                graph
            }
        };
        let center = self.editor.read(cx).note_id().map(str::to_owned);
        graph.update(cx, |graph, cx| {
            graph.set_center(center, cx);
            graph.visible = true;
            graph.fit(cx);
        });
        self.graph_open = true;
        window.focus(&graph.focus_handle(cx), cx);
        cx.notify();
    }

    pub(crate) fn close_graph(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.graph_open {
            return;
        }
        self.graph_open = false;
        if let Some(graph) = &self.graph {
            graph.update(cx, |graph, _| graph.visible = false);
        }
        window.focus(&self.focus, cx);
        cx.notify();
    }
}

/// The workspace's actions for the graph.
pub fn on_actions(el: Div, cx: &mut Context<Workspace>) -> Div {
    el.on_action(cx.listener(|this, _: &ShowGraph, window, cx| this.show_graph(window, cx)))
        .on_action(cx.listener(|this, _: &CloseGraphView, window, cx| this.close_graph(window, cx)))
        .on_action(cx.listener(|this, _: &ShowLocalGraph, window, cx| {
            this.close_graph(window, cx);
            this.close_settings(window, cx);
            this.set_focus_mode(false, window, cx);
            this.show_inspector(Some(crate::config::InspectorTab::Graph), cx);
        }))
        .on_action(cx.listener(|this, _: &ShowRelated, window, cx| {
            this.close_graph(window, cx);
            this.close_settings(window, cx);
            this.set_focus_mode(false, window, cx);
            this.show_inspector(Some(crate::config::InspectorTab::Related), cx);
        }))
}

/// The library's graph over the window, while open.
pub fn overlay(workspace: &Workspace) -> Option<impl IntoElement> {
    let graph = workspace.graph.clone().filter(|_| workspace.graph_open)?;
    Some(
        div()
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .debug_selector(|| "graph-overlay".into())
            .child(graph),
    )
}

#[cfg(test)]
mod tests {
    use crate::graph::ZOOM_RANGE;

    #[test]
    fn zoom_range_holds_the_fitted_views() {
        assert!(ZOOM_RANGE.0 < 2.6 && 2.6 < ZOOM_RANGE.1);
    }
}
