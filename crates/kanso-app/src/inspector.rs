//! The inspector: a pane on the editor's right with the open note's outline
//! and the notes that link to it.
//!
//! Outline lists the note's headings as they are typed, the one the caret
//! is under marked; clicking one takes the caret there. Backlinks lists the
//! notes that link here, each with the line it links on, and below them the
//! notes that mention the title without linking, each with Link to make the
//! mention a link. Graph draws the notes around the open one
//! (`graph_view.rs`), Related lists the notes about the same things
//! (`related.rs`), Comments a shared note's comments
//! (`inspector_comments.rs`), and Ask answers a question from the notes
//! (`ask_panel.rs`).

#[path = "inspector_comments.rs"]
mod comments;
#[path = "inspector_properties.rs"]
mod properties_section;

use std::sync::Arc;

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonGroup, ButtonVariants as _};
use gpui_kit::component::{Icon, Selectable as _, Sizable as _, StyledExt as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_core::DocType;
use kanso_core::outline::{Heading, headings};
use kanso_store::{Backlink, Origin, Put, RelatedNote, Store};
use serde_json::json;

use crate::ask_panel::AskPanel;
use crate::config::InspectorTab;
use crate::editor::NoteEditor;
use crate::graph_view::{GraphView, Scope};
use crate::layout::{BOTTOM_BAND_HEIGHT, TOP_BAND_HEIGHT};
use crate::library::Library;
use crate::note_list::quiet_empty;
use crate::related::{Related, Status as RelatedStatus};
use crate::text_stats::plural;
use crate::theme::{Colors, section_label};
use crate::workspace::AppState;
use crate::zoom::px;

/// How far each heading level indents its row, in points.
const LEVEL_INDENT: f32 = 12.;
/// Room either side of the rows.
const PAD_X: f32 = 10.;

/// What the inspector asks of the workspace.
pub enum InspectorEvent {
    /// A tab was picked: kept for next time.
    Tab(InspectorTab),
}

/// What the pane shows of the editor: when none of it changes, a keystroke
/// leaves the pane alone.
#[derive(Clone, PartialEq)]
struct DrawnFrom {
    note: Option<String>,
    /// The outline's version, and the heading the caret is under in it.
    outline: Option<u64>,
    active: Option<usize>,
    backlinks: Vec<Backlink>,
    comments: Option<Vec<kanso_collab::CommentView>>,
}

/// How long typing pauses before the outline catches up.
const OUTLINE_DELAY: std::time::Duration = std::time::Duration::from_millis(150);

pub struct Inspector {
    store: Arc<Store>,
    library: Entity<Library>,
    editor: Entity<NoteEditor>,
    pub tab: InspectorTab,
    /// The outline of the open note: its ID, the text version it was made
    /// from, and its headings.
    outline: Option<(String, u64, Vec<Heading>)>,
    /// Parses the outline again once typing pauses.
    outline_task: Option<Task<()>>,
    /// What the pane last drew from.
    drawn: Option<DrawnFrom>,
    /// Unlinked mentions of the open note, and which note they are for.
    mentions: Vec<Backlink>,
    mentions_for: Option<String>,
    mentions_task: Option<Task<()>>,
    /// Whether the pane is on screen: mentions are only looked for then.
    pub visible: bool,
    /// The notes around the open one.
    pub(crate) graph: Entity<GraphView>,
    /// Ask Your Notes, while AI is on.
    ask: Entity<AskPanel>,
    /// The open note's related notes (`None` while it isn't embedded), and
    /// the note and embeddings they were found for.
    related: Option<Vec<RelatedNote>>,
    related_for: Option<(String, u64)>,
    related_task: Option<Task<()>>,
    /// The property value being edited, and the property being added.
    property_edit: Option<properties_section::PropertyEdit>,
    property_add: Option<properties_section::PropertyAdd>,
    scroll: ScrollHandle,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<InspectorEvent> for Inspector {}

impl Inspector {
    pub fn new(
        store: Arc<Store>,
        library: Entity<Library>,
        editor: Entity<NoteEditor>,
        tab: InspectorTab,
        hops: u8,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let graph =
            cx.new(|cx| GraphView::new(store.clone(), library.clone(), Scope::Local, hops, cx));
        let ask =
            cx.new(|cx| AskPanel::new(store.clone(), library.clone(), editor.clone(), window, cx));
        // The editor notifies on every keystroke; the pane draws again only
        // when what it shows changed: the note, the heading the caret is
        // under, or the backlinks. Typing schedules the outline instead.
        let subscriptions = vec![cx.observe(&editor, |this, _, cx| {
            if !this.visible {
                return;
            }
            let version = this.editor.read(cx).outline_version();
            if let (Some((id, at)), Some((kept, kept_at, _))) = (&version, &this.outline)
                && id == kept
                && at != kept_at
            {
                this.schedule_outline(cx);
            }
            if this.drawn.as_ref() != Some(&this.drawn_from(cx)) {
                cx.notify();
            }
        })];
        let mut subscriptions = subscriptions;
        // Tags suggested for the open note, shown on the Related tab.
        let meaning = editor.read(cx).meaning().clone();
        subscriptions.push(cx.observe(&meaning, |this, _, cx| {
            if this.visible && this.tab == InspectorTab::Related {
                cx.notify();
            }
        }));
        // A new state or new embeddings: the Related tab asks again.
        subscriptions.push(cx.observe_global::<Related>(|this, cx| {
            if this.visible && this.tab == InspectorTab::Related {
                cx.notify();
            }
        }));
        Inspector {
            store,
            library,
            editor,
            tab,
            outline: None,
            outline_task: None,
            drawn: None,
            mentions: Vec::new(),
            mentions_for: None,
            mentions_task: None,
            visible: false,
            graph,
            ask,
            related: None,
            related_for: None,
            related_task: None,
            property_edit: None,
            property_add: None,
            scroll: ScrollHandle::new(),
            _subscriptions: subscriptions,
        }
    }

    /// Puts the keyboard in Ask Your Notes' question box.
    #[cfg(test)]
    pub(crate) fn ask_panel_for_test(&self) -> Entity<AskPanel> {
        self.ask.clone()
    }

    pub fn focus_ask(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.ask
            .update(cx, |ask, cx| ask.focus_question(window, cx));
    }

    /// Summarises the open note in the Ask panel.
    pub fn summarize_note(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.ask
            .update(cx, |ask, cx| ask.summarize_note(window, cx));
    }

    /// Ends the Ask conversation, as turning AI off or closing the pane does.
    pub fn stop_ask(&mut self, cx: &mut Context<Self>) {
        self.ask.update(cx, |ask, cx| ask.end_conversation(cx));
    }

    /// Shows `tab`. Leaving Ask ends its conversation, which is never saved.
    pub fn set_tab(&mut self, tab: InspectorTab, cx: &mut Context<Self>) {
        if self.tab != tab {
            if self.tab == InspectorTab::Ask {
                self.ask.update(cx, |ask, cx| ask.end_conversation(cx));
            }
            self.tab = tab;
            self.scroll.set_offset(point(px(0.), px(0.)));
            cx.emit(InspectorEvent::Tab(tab));
            cx.notify();
        }
    }

    /// Looks for unlinked mentions again, as after a note changed.
    pub fn refresh(&mut self, cx: &mut Context<Self>) {
        self.mentions_for = None;
        self.related_for = None;
        if self.visible {
            cx.notify();
        }
    }

    /// The open note's headings. A note just opened is parsed at once;
    /// while it is typed in, the last outline stays until typing pauses and
    /// it is parsed again off the main thread, so a keystroke never waits
    /// on it.
    fn outline(&mut self, cx: &mut Context<Self>) -> &[Heading] {
        let Some((id, version)) = self.editor.read(cx).outline_version() else {
            self.outline = None;
            self.outline_task = None;
            return &[];
        };
        match &self.outline {
            Some((kept, at, _)) if *kept == id && *at == version => {}
            Some((kept, _, _)) if *kept == id => self.schedule_outline(cx),
            _ => {
                let text = self.editor.read(cx).body_text(cx);
                self.outline = Some((id, version, headings(&text)));
                self.outline_task = None;
            }
        }
        self.outline.as_ref().map_or(&[], |(_, _, h)| h.as_slice())
    }

    /// What the pane would draw from now, without parsing anything.
    fn drawn_from(&self, cx: &App) -> DrawnFrom {
        let editor = self.editor.read(cx);
        let note = editor.note_id().map(str::to_owned);
        let outline = self
            .outline
            .as_ref()
            .filter(|(kept, _, _)| Some(kept) == note.as_ref());
        DrawnFrom {
            outline: outline.map(|(_, at, _)| *at),
            active: outline.and_then(|(_, _, h)| current_heading(h, editor.caret_line(cx))),
            note,
            backlinks: editor.backlinks().to_vec(),
            comments: editor
                .shared_comments()
                .map(|(_, comments, _)| comments.to_vec()),
        }
    }

    /// Parses the outline again after [`OUTLINE_DELAY`], unless that is
    /// already on its way.
    fn schedule_outline(&mut self, cx: &mut Context<Self>) {
        if self.outline_task.is_some() {
            return;
        }
        self.outline_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(OUTLINE_DELAY).await;
            let Ok(Some((id, version, text))) =
                this.update(cx, |this, cx| this.editor.read(cx).outline_source(cx))
            else {
                return;
            };
            let found = cx
                .background_executor()
                .spawn(async move { headings(&text) })
                .await;
            this.update(cx, |this, cx| {
                this.outline_task = None;
                // A different note opened meanwhile: it was parsed then.
                if this
                    .outline
                    .as_ref()
                    .is_some_and(|(kept, _, _)| *kept == id)
                {
                    this.outline = Some((id, version, found));
                    cx.notify();
                }
            })
            .ok();
        }));
    }

    /// Starts looking for the open note's unlinked mentions, in the
    /// background, unless they are known.
    fn load_mentions(&mut self, cx: &mut Context<Self>) {
        let id = self.editor.read(cx).note_id().map(str::to_owned);
        if self.mentions_for == id {
            return;
        }
        self.mentions_for = id.clone();
        let Some(id) = id else {
            self.mentions.clear();
            self.mentions_task = None;
            return;
        };
        let store = self.store.clone();
        let search = cx
            .background_executor()
            .spawn(async move { store.unlinked_mentions(&id).unwrap_or_default() });
        self.mentions_task = Some(cx.spawn(async move |this, cx| {
            let found = search.await;
            this.update(cx, |this, cx| {
                this.mentions_task = None;
                if this.mentions != found {
                    this.mentions = found;
                    cx.notify();
                }
            })
            .ok();
        }));
    }

    /// Opens note `id` at `line`.
    fn open_at(&self, id: String, line: u32, cx: &mut App) {
        self.editor
            .update(cx, |editor, _| editor.reveal_after_open(id.clone(), line));
        self.library
            .update(cx, |library, cx| library.select(Some(id), cx));
    }

    /// Makes the mention in note `mention` a link to the open note.
    fn link_mention(&mut self, mention: &Backlink, window: &mut Window, cx: &mut Context<Self>) {
        let Some(target) = self.editor.read(cx).note_id().map(str::to_owned) else {
            return;
        };
        let linked = kanso_engine::links::mention::link_mention(
            &self.store,
            &target,
            &mention.id,
            mention.line,
        );
        match linked {
            Ok(None) => return,
            Ok(Some(_)) => {}
            Err(err) => {
                use gpui_kit::component::WindowExt as _;
                window.push_notification(
                    gpui_kit::component::notification::Notification::error(format!(
                        "Couldn't link the mention: {err}"
                    )),
                    cx,
                );
            }
        }
        self.mentions.retain(|m| m.id != mention.id);
        self.mentions_for = None;
        cx.notify();
    }

    fn render_tabs(&self, backlinks: usize, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let tab = self.tab;
        let open_comments = self.open_comments(cx);
        let options = visible_tabs(cx.global::<AppState>().config.ai.enabled);
        // Each tab shows its icon, the open one its name too, and Backlinks
        // and Comments their counts.
        let tabs = [
            (
                "tab-outline",
                Lucide::TableOfContents,
                "Outline",
                None,
                InspectorTab::Outline,
            ),
            (
                "tab-ask",
                Lucide::MessageCircle,
                "Ask",
                None,
                InspectorTab::Ask,
            ),
            (
                "tab-backlinks",
                Lucide::Link,
                "Backlinks",
                (backlinks > 0).then(|| backlinks.to_string()),
                InspectorTab::Backlinks,
            ),
            (
                "tab-graph",
                Lucide::Waypoints,
                "Graph",
                None,
                InspectorTab::Graph,
            ),
            (
                "tab-related",
                Lucide::Sparkles,
                "Related",
                None,
                InspectorTab::Related,
            ),
            (
                "tab-comments",
                Lucide::MessageSquare,
                "Comments",
                (open_comments > 0).then(|| open_comments.to_string()),
                InspectorTab::Comments,
            ),
        ]
        .into_iter()
        .filter(|(_, _, _, _, option)| options.contains(option))
        .collect::<Vec<_>>();
        h_flex()
            .h(px(TOP_BAND_HEIGHT))
            .flex_none()
            .px(px(PAD_X))
            .child(
                ButtonGroup::new("inspector-tabs")
                    .outline()
                    .small()
                    .children(tabs.into_iter().map(|(id, icon, label, count, option)| {
                        let selected = tab == option;
                        let color = if selected {
                            colors.text
                        } else {
                            colors.text_secondary
                        };
                        Button::new(id)
                            .selected(selected)
                            .when(!selected, |el| el.tooltip(label))
                            .child(
                                h_flex()
                                    .px_0p5()
                                    .gap(px(5.))
                                    .text_size(px(12.5))
                                    .text_color(color)
                                    .child(Icon::new(icon).size(px(14.)).text_color(color))
                                    .when(selected, |el| el.font_medium().child(label))
                                    .when_some(count, |el, count| {
                                        el.child(
                                            div()
                                                .text_size(px(11.5))
                                                .text_color(colors.text_muted)
                                                .child(count),
                                        )
                                    }),
                            )
                    }))
                    .on_click(cx.listener(move |this, clicked: &Vec<usize>, _, cx| {
                        if let Some(&ix) = clicked.first() {
                            this.set_tab(options[ix], cx);
                        }
                    })),
            )
    }

    fn render_outline(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let colors = Colors::get(cx);
        let caret = self.editor.read(cx).caret_line(cx);
        let has_note = self.editor.read(cx).note_id().is_some();
        let headings = self.outline(cx).to_vec();
        if !has_note {
            return quiet_empty(Lucide::TableOfContents, "No note open".into(), None, colors)
                .into_any_element();
        }
        if headings.is_empty() {
            return quiet_empty(
                Lucide::TableOfContents,
                "No headings yet".into(),
                Some("Start a line with # to add one.".into()),
                colors,
            )
            .into_any_element();
        }
        let top = headings.iter().map(|h| h.level).min().unwrap_or(1);
        let current = current_heading(&headings, caret);
        let editor = self.editor.clone();
        v_flex()
            .id("outline")
            .size_full()
            .overflow_y_scroll()
            .track_scroll(&self.scroll)
            .px(px(PAD_X))
            .pb(px(12.))
            .children(headings.into_iter().enumerate().map(|(ix, heading)| {
                let active = current == Some(ix);
                let indent = f32::from(heading.level.saturating_sub(top)) * LEVEL_INDENT;
                let editor = editor.clone();
                let line = heading.line;
                h_flex()
                    .id(ElementId::NamedInteger("heading".into(), ix as u64))
                    .w_full()
                    .h(px(28.))
                    .pl(px(8. + indent))
                    .pr(px(8.))
                    .rounded(px(6.))
                    .gap(px(8.))
                    .cursor_pointer()
                    .when(active, |el| el.bg(colors.row_active))
                    .when(!active, |el| el.hover(|el| el.bg(colors.row_hover)))
                    .child(
                        div()
                            .flex_none()
                            .w(px(2.))
                            .h(px(14.))
                            .rounded_full()
                            .bg(if active {
                                Hsla::from(colors.accent)
                            } else {
                                gpui_kit::transparent_black()
                            }),
                    )
                    .child(
                        div()
                            .min_w_0()
                            .flex_1()
                            .truncate()
                            .text_size(px(if heading.level <= top { 13. } else { 12.5 }))
                            .when(heading.level <= top, |el| el.font_medium())
                            .text_color(if active || heading.level <= top {
                                colors.text
                            } else {
                                colors.text_secondary
                            })
                            .child(heading.text),
                    )
                    .on_click(move |_, window, cx| {
                        editor.update(cx, |editor, cx| editor.reveal_line(line, window, cx));
                    })
            }))
            .into_any_element()
    }

    fn render_backlinks(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let colors = Colors::get(cx);
        let editor = self.editor.read(cx);
        if editor.note_id().is_none() {
            return quiet_empty(Lucide::Link, "No note open".into(), None, colors)
                .into_any_element();
        }
        let linked: Vec<Backlink> = editor.backlinks().to_vec();
        self.load_mentions(cx);
        let mentions = self.mentions.clone();
        if linked.is_empty() && mentions.is_empty() {
            return quiet_empty(
                Lucide::Link,
                "No links to this note yet".into(),
                Some("Type [[ in another note to link here.".into()),
                colors,
            )
            .into_any_element();
        }
        let view = cx.entity();
        let section = |label: String| {
            section_label(div(), colors)
                .px(px(8.))
                .pt(px(10.))
                .pb(px(6.))
                .child(label)
        };
        let row = |ix: usize, kind: &'static str, link: &Backlink, action: Option<AnyElement>| {
            let open = view.clone();
            let (id, line) = (link.id.clone(), link.line);
            let title = if link.title.trim().is_empty() {
                "Untitled".to_owned()
            } else {
                link.title.clone()
            };
            v_flex()
                .id(ElementId::NamedInteger(kind.into(), ix as u64))
                .group(SharedString::from(format!("{kind}-{ix}")))
                .w_full()
                .px(px(8.))
                .py(px(7.))
                .gap(px(2.))
                .rounded(px(6.))
                .cursor_pointer()
                .hover(|el| el.bg(colors.row_hover))
                .child(
                    h_flex()
                        .gap(px(6.))
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .truncate()
                                .text_size(px(13.))
                                .font_medium()
                                .text_color(colors.text)
                                .child(title),
                        )
                        .children(action),
                )
                .when(!link.context.is_empty(), |el| {
                    el.child(
                        div()
                            .text_size(px(12.))
                            .line_height(px(17.))
                            .text_color(colors.text_secondary)
                            .line_clamp(2)
                            .text_ellipsis()
                            .child(link.context.clone()),
                    )
                })
                .on_click(move |_, _, cx| {
                    let (id, line) = (id.clone(), line);
                    open.update(cx, |this, cx| this.open_at(id, line, cx));
                })
        };
        let mut list = v_flex()
            .id("backlinks")
            .size_full()
            .overflow_y_scroll()
            .track_scroll(&self.scroll)
            .px(px(PAD_X))
            .pb(px(12.));
        if !linked.is_empty() {
            list = list.child(section("Linked from".into()));
            for (ix, link) in linked.iter().enumerate() {
                list = list.child(row(ix, "backlink", link, None));
            }
        }
        if !mentions.is_empty() {
            list = list.child(section("Unlinked mentions".into()));
            for (ix, mention) in mentions.iter().enumerate() {
                let link_view = view.clone();
                let target = mention.clone();
                let action = Button::new(ElementId::NamedInteger("link-mention".into(), ix as u64))
                    .ghost()
                    .xsmall()
                    .child(
                        div()
                            .px_0p5()
                            .text_size(px(11.5))
                            .text_color(colors.link)
                            .child("Link"),
                    )
                    .tooltip("Make this mention a link to the note")
                    .on_click(move |_, window, cx| {
                        cx.stop_propagation();
                        link_view.update(cx, |this, cx| this.link_mention(&target, window, cx));
                    })
                    .into_any_element();
                list = list.child(row(ix, "mention", mention, Some(action)));
            }
        }
        list.into_any_element()
    }

    fn render_bottom_band(&mut self, backlinks: usize, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let summary = match self.tab {
            InspectorTab::Outline => {
                let n = self.outline(cx).len();
                match n {
                    0 => String::new(),
                    1 => "1 heading".to_owned(),
                    n => format!("{n} headings"),
                }
            }
            InspectorTab::Backlinks => {
                let mentions = self.mentions.len();
                let mut parts = Vec::new();
                match backlinks {
                    0 => {}
                    1 => parts.push("1 link".to_owned()),
                    n => parts.push(format!("{n} links")),
                }
                match mentions {
                    0 => {}
                    1 => parts.push("1 mention".to_owned()),
                    n => parts.push(format!("{n} mentions")),
                }
                parts.join(" · ")
            }
            InspectorTab::Graph => match self.graph.read(cx).counts() {
                Some((notes, links)) if notes > 0 => format!(
                    "{} · {}",
                    plural(notes, "note", "notes"),
                    plural(links, "link", "links")
                ),
                _ => String::new(),
            },
            InspectorTab::Related => match related_status(cx) {
                RelatedStatus::Indexing { done, total } => format!(
                    "Indexing {} of {}",
                    crate::text_stats::format_count(done),
                    plural(total, "note", "notes")
                ),
                RelatedStatus::Ready { notes } => {
                    format!("{} indexed", plural(notes, "note", "notes"))
                }
                _ => String::new(),
            },
            InspectorTab::Comments => self.comments_band(cx),
            InspectorTab::Ask => String::new(),
        };
        let graph_controls = (self.tab == InspectorTab::Graph).then(|| {
            let hops = self.graph.read(cx).hops;
            let graph = self.graph.clone();
            h_flex()
                .gap(px(4.))
                .child(
                    ButtonGroup::new("graph-hops")
                        .outline()
                        .xsmall()
                        .children([("1 link", 1u8), ("2 links", 2u8)].map(|(label, n)| {
                            let selected = hops == n;
                            Button::new(("hops", n as usize))
                                .selected(selected)
                                .tooltip(if n == 1 {
                                    "The notes this one links with"
                                } else {
                                    "And the notes those link with"
                                })
                                .child(
                                    div()
                                        .px_0p5()
                                        .text_size(px(11.5))
                                        .text_color(if selected {
                                            colors.text
                                        } else {
                                            colors.text_secondary
                                        })
                                        .child(label),
                                )
                        }))
                        .on_click(move |clicked: &Vec<usize>, _, cx| {
                            let Some(&ix) = clicked.first() else { return };
                            let hops = ix as u8 + 1;
                            graph.update(cx, |graph, cx| graph.set_hops(hops, cx));
                            AppState::update_config(cx, |config| {
                                config.discovery.graph_hops = hops
                            });
                        }),
                )
                .child(
                    Button::new("open-graph")
                        .ghost()
                        .xsmall()
                        .icon(Lucide::Maximize)
                        .tooltip("Graph of all notes")
                        .on_click(|_, window, cx| {
                            window.dispatch_action(Box::new(crate::graph_view::ShowGraph), cx)
                        }),
                )
        });
        h_flex()
            .h(px(BOTTOM_BAND_HEIGHT))
            .flex_none()
            .pl(px(PAD_X + 8.))
            .pr(px(PAD_X))
            .gap(px(8.))
            .border_t_1()
            .border_color(colors.divider)
            .text_size(px(12.))
            .text_color(colors.text_muted)
            .child(div().flex_1().min_w_0().truncate().child(summary))
            .children(graph_controls)
    }

    /// The local graph, centred on the open note.
    fn render_graph(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let note = self.editor.read(cx).note_id().map(str::to_owned);
        let visible = self.visible;
        self.graph.update(cx, |graph, cx| {
            graph.visible = visible;
            graph.set_center(note, cx);
        });
        self.graph.clone().into_any_element()
    }

    /// Starts looking for the open note's related notes, in the background,
    /// unless they are known for these embeddings.
    fn load_related(&mut self, generation: u64, cx: &mut Context<Self>) {
        let Some(id) = self.editor.read(cx).note_id().map(str::to_owned) else {
            self.related = None;
            self.related_for = None;
            return;
        };
        let wanted = Some((id.clone(), generation));
        if self.related_for == wanted {
            return;
        }
        // A different note: what was shown is not about it.
        if self
            .related_for
            .as_ref()
            .is_none_or(|(kept, _)| *kept != id)
        {
            self.related = None;
        }
        self.related_for = wanted;
        let store = self.store.clone();
        let search = cx.background_executor().spawn(async move {
            store
                .related_notes(&id, crate::related::MODEL_ID, RELATED_LIMIT)
                .unwrap_or(None)
        });
        self.related_task = Some(cx.spawn(async move |this, cx| {
            let found = search.await;
            this.update(cx, |this, cx| {
                this.related_task = None;
                if this.related != found {
                    this.related = found;
                    cx.notify();
                }
            })
            .ok();
        }));
    }

    fn render_related(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let colors = Colors::get(cx);
        if self.editor.read(cx).note_id().is_none() {
            return quiet_empty(Lucide::Sparkles, "No note open".into(), None, colors)
                .into_any_element();
        }
        let status = related_status(cx);
        let generation = cx.try_global::<Related>().map_or(0, |r| r.generation);
        let action = |id: &'static str, label: &'static str, run: fn(&mut Window, &mut App)| {
            Button::new(id)
                .outline()
                .small()
                .label(label)
                .on_click(move |_, window, cx| run(window, cx))
        };
        let state = |icon: Lucide, title: String, detail: String| {
            quiet_empty(icon, title, Some(detail), colors)
        };
        match status {
            RelatedStatus::Off => {
                return state(
                    Lucide::Sparkles,
                    "Related notes are off".into(),
                    format!(
                        "Find notes about the same things as this one. It works on {} and sends nothing anywhere.",
                        crate::settings_discovery::this_device()
                    ),
                )
                .child(div().mt(px(14.)).child(action(
                    "related-settings",
                    "Turn On in Settings…",
                    |window, cx| {
                        window.dispatch_action(Box::new(crate::settings::OpenGeneralSettings), cx)
                    },
                )))
                .into_any_element();
            }
            RelatedStatus::Downloading { done, total } => {
                return state(
                    Lucide::Download,
                    format!(
                        "Downloading the model… {}%",
                        crate::related::percent(done, total)
                    ),
                    "A one-time download of about 30\u{a0}MB.".into(),
                )
                .child(progress_bar(done as f32 / total.max(1) as f32, colors))
                .into_any_element();
            }
            RelatedStatus::Loading => {
                return state(
                    Lucide::Sparkles,
                    "Getting ready…".into(),
                    "Checking the language model.".into(),
                )
                .into_any_element();
            }
            RelatedStatus::Failed(message) => {
                return state(
                    Lucide::TriangleAlert,
                    "Related notes stopped".into(),
                    message,
                )
                .child(
                    div()
                        .mt(px(14.))
                        .child(action("related-retry", "Try Again", |_, cx| {
                            crate::related::start(cx)
                        })),
                )
                .into_any_element();
            }
            RelatedStatus::Indexing { .. } | RelatedStatus::Ready { .. } => {}
        }
        self.load_related(generation, cx);
        let Some(related) = self.related.clone() else {
            return state(
                Lucide::Sparkles,
                "Reading this note…".into(),
                "Related notes show once it's indexed.".into(),
            )
            .into_any_element();
        };
        let tags = self.render_suggested_tags(cx);
        if related.is_empty() && tags.is_none() {
            return state(
                Lucide::Sparkles,
                "Nothing related yet".into(),
                "Notes about the same things show here, apart from those already linked.".into(),
            )
            .into_any_element();
        }
        let view = cx.entity();
        v_flex()
            .id("related")
            .size_full()
            .overflow_y_scroll()
            .track_scroll(&self.scroll)
            .px(px(PAD_X))
            .pb(px(12.))
            .children(tags)
            .when(!related.is_empty(), |el| {
                el.child(
                    section_label(div(), colors)
                        .px(px(8.))
                        .pt(px(10.))
                        .pb(px(6.))
                        .child("Not linked yet"),
                )
            })
            .children(related.into_iter().enumerate().map(|(ix, note)| {
                let open = view.clone();
                let id = note.id.clone();
                let title = if note.title.trim().is_empty() {
                    "Untitled".to_owned()
                } else {
                    note.title.clone()
                };
                let share = note.score.clamp(0., 1.);
                h_flex()
                    .id(ElementId::NamedInteger("related".into(), ix as u64))
                    .w_full()
                    .px(px(8.))
                    .py(px(7.))
                    .gap(px(10.))
                    .rounded(px(6.))
                    .cursor_pointer()
                    .hover(|el| el.bg(colors.row_hover))
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .gap(px(3.))
                            .child(
                                div()
                                    .truncate()
                                    .text_size(px(13.))
                                    .font_medium()
                                    .text_color(colors.text)
                                    .child(title),
                            )
                            .child(
                                div()
                                    .text_size(px(11.5))
                                    .text_color(colors.text_muted)
                                    .child(crate::related::similarity_label(note.score)),
                            ),
                    )
                    .child(
                        // The similarity, as a short meter.
                        div()
                            .flex_none()
                            .w(px(36.))
                            .h(px(4.))
                            .rounded_full()
                            .bg(colors.fill)
                            .child(
                                div()
                                    .h_full()
                                    .w(px(36. * share))
                                    .rounded_full()
                                    .bg(Hsla::from(colors.accent).opacity(0.4 + 0.6 * share)),
                            ),
                    )
                    .on_click(move |_, _, cx| {
                        let id = id.clone();
                        open.update(cx, |this, cx| this.open_at(id, 0, cx));
                    })
            }))
            .into_any_element()
    }
}

impl Inspector {
    /// The tags the open note's close notes have and it hasn't, each a
    /// chip that adds it; `None` when there are none.
    fn render_suggested_tags(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let colors = Colors::get(cx);
        let editor = self.editor.read(cx);
        let own = editor.current_note(cx).map(|n| n.tags).unwrap_or_default();
        let suggested = editor.meaning().read(cx).suggested_tags(&own);
        if suggested.is_empty() {
            return None;
        }
        let editor = self.editor.clone();
        Some(
            v_flex()
                .child(
                    section_label(div(), colors)
                        .px(px(8.))
                        .pt(px(10.))
                        .pb(px(6.))
                        .child("Suggested tags"),
                )
                .child(
                    h_flex()
                        .flex_wrap()
                        .gap(px(6.))
                        .px(px(8.))
                        .pb(px(4.))
                        .children(suggested.into_iter().enumerate().map(|(ix, tag)| {
                            let editor = editor.clone();
                            let id = tag.tag_id.clone();
                            h_flex()
                                .id(ElementId::NamedInteger("suggested-tag".into(), ix as u64))
                                .gap(px(3.))
                                .px(px(8.))
                                .py(px(3.))
                                .rounded(px(6.))
                                .border_1()
                                .border_color(colors.divider)
                                .cursor_pointer()
                                .text_size(px(12.))
                                .text_color(colors.text_secondary)
                                .hover(|el| el.bg(colors.row_hover))
                                .child(Icon::new(Lucide::Plus).size(px(11.)))
                                .child(format!("#{}", tag.name))
                                .tooltip(move |window, cx| {
                                    gpui_kit::component::tooltip::Tooltip::new(
                                        "Notes about the same things have this tag",
                                    )
                                    .build(window, cx)
                                })
                                .on_click(move |_, _, cx| {
                                    let id = id.clone();
                                    editor.update(cx, |editor, cx| editor.toggle_tag(id, cx));
                                })
                        })),
                )
                .into_any_element(),
        )
    }
}

/// How many related notes the tab lists.
const RELATED_LIMIT: usize = 8;

fn related_status(cx: &App) -> RelatedStatus {
    cx.try_global::<Related>()
        .map(|r| r.status.clone())
        .unwrap_or_default()
}

/// A thin bar `share` (0 to 1) of the way full.
// no-prop-test: drawing; the Related snapshots show it.
fn progress_bar(share: f32, colors: &Colors) -> impl IntoElement {
    div()
        .mt(px(14.))
        .w(px(160.))
        .h(px(4.))
        .rounded_full()
        .bg(colors.fill)
        .child(
            div()
                .h_full()
                .w(px(160. * share.clamp(0., 1.)))
                .rounded_full()
                .bg(colors.accent),
        )
}

#[cfg(test)]
impl Inspector {
    /// The outline as last drawn, by heading text.
    pub fn outline_for_test(&self) -> Vec<String> {
        self.outline
            .as_ref()
            .map(|(_, _, h)| h.iter().map(|h| h.text.clone()).collect())
            .unwrap_or_default()
    }

    /// The tabs the pane would offer now.
    pub fn visible_tabs_for_test(&self, cx: &App) -> Vec<InspectorTab> {
        let enabled = cx
            .try_global::<AppState>()
            .is_some_and(|state| state.config.ai.enabled);
        visible_tabs(enabled)
    }

    /// The notes found mentioning the open note, by ID.
    pub fn mentions_for_test(&self) -> Vec<String> {
        self.mentions.iter().map(|m| m.id.clone()).collect()
    }

    /// Presses Link on the first mention.
    pub fn link_first_mention_for_test(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(first) = self.mentions.first().cloned() {
            self.link_mention(&first, window, cx);
        }
    }
}

/// The inspector's tabs, with Ask only while AI is on.
fn visible_tabs(ai_enabled: bool) -> Vec<InspectorTab> {
    [
        InspectorTab::Outline,
        InspectorTab::Ask,
        InspectorTab::Backlinks,
        InspectorTab::Graph,
        InspectorTab::Related,
        InspectorTab::Comments,
    ]
    .into_iter()
    .filter(|tab| ai_enabled || *tab != InspectorTab::Ask)
    .collect()
}

/// The heading the caret on `line` is under: the last one at or above it.
pub use kanso_engine::outline::current_heading;

impl Render for Inspector {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let backlinks = self.editor.read(cx).backlinks().len();
        // A plugin's panel, while one is open, takes the pane's body.
        let panel = crate::plugin_panel::element(window, cx);
        let body = match self.tab {
            _ if panel.is_some() => panel.unwrap_or_else(|| div().into_any_element()),
            InspectorTab::Outline => {
                let properties = self.render_properties(cx);
                let outline = self.render_outline(cx);
                match properties {
                    Some(properties) => v_flex()
                        .size_full()
                        .child(properties)
                        .child(div().flex_1().min_h_0().child(outline))
                        .into_any_element(),
                    None => outline,
                }
            }
            InspectorTab::Backlinks => self.render_backlinks(cx),
            InspectorTab::Graph => self.render_graph(cx),
            InspectorTab::Related => self.render_related(cx),
            InspectorTab::Comments => self.render_comments(cx),
            InspectorTab::Ask => self.ask.clone().into_any_element(),
        };
        if self.tab != InspectorTab::Graph {
            self.graph.update(cx, |graph, _| graph.visible = false);
        }
        self.drawn = Some(self.drawn_from(cx));
        v_flex()
            .size_full()
            .bg(colors.list)
            .border_l_1()
            .border_color(colors.divider)
            .child(self.render_tabs(backlinks, cx))
            .child(div().flex_1().min_h_0().child(body))
            .child(self.render_bottom_band(backlinks, cx))
    }
}

#[cfg(test)]
mod tests {
    use super::current_heading;
    use kanso_core::outline::Heading;
    use proptest::prelude::*;

    fn at(lines: &[u32]) -> Vec<Heading> {
        lines
            .iter()
            .map(|&line| Heading {
                level: 2,
                text: "h".into(),
                line,
            })
            .collect()
    }

    #[test]
    fn the_caret_is_under_the_last_heading_above_it() {
        let headings = at(&[2, 5, 9]);
        assert_eq!(current_heading(&headings, 0), None);
        assert_eq!(current_heading(&headings, 5), Some(1));
        assert_eq!(current_heading(&headings, 7), Some(1));
        assert_eq!(current_heading(&headings, 40), Some(2));
    }

    proptest! {
        /// The heading found is at or above the caret, and the next one (if
        /// any) is below it.
        #[test]
        fn prop_current_heading_brackets_the_caret(mut lines in prop::collection::btree_set(0u32..200, 0..12), caret in 0u32..220) {
            let lines: Vec<u32> = std::mem::take(&mut lines).into_iter().collect();
            let headings = at(&lines);
            match current_heading(&headings, caret) {
                Some(ix) => {
                    prop_assert!(headings[ix].line <= caret);
                    if let Some(next) = headings.get(ix + 1) {
                        prop_assert!(next.line > caret);
                    }
                }
                None => prop_assert!(lines.iter().all(|&l| l > caret)),
            }
        }
    }
}
