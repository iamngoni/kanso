//! An inline sketch in a note (#45): the fixed box a `sketch` fence holds,
//! drawn on in place in Split or Preview.
//!
//! The preview parses the fence into a [`preview_rich::Block`] and asks
//! [`element`] for it. Each sketch in each open note has one [`SketchView`]
//! kept in a [`Registry`] (the markdown renderer draws from scratch every
//! time, so the stroke in progress cannot live in it). The view draws with
//! the canvas's own state and painting ([`CanvasState`], the same
//! perfect-freehand outlines), clipped to its box.
//!
//! Nothing is saved per point. When a stroke ends the view schedules one
//! write two seconds out ([`SAVE_AFTER`]); it sends the sketch's JSON to the
//! editor as a [`PreviewEdit::Sketch`], which replaces exactly the fence with
//! that ID and checks the size caps before the write. Leaving the note, or
//! the pointer leaving the box, sends it at once.
//!
//! In a shared note the sketch is read-only (a text CRDT would merge two
//! people's replaced fences character by character): it draws and exports
//! as usual, and "Copy as canvas" makes a private canvas of it.

use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{Disableable as _, Selectable as _, Sizable as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_core::canvas::{Canvas, Refused, sketch};
use kanso_render::canvas::pen;

use crate::canvas_state::{COLORS, CanvasState, ERASER_RADIUS, Outcome, PenSample, Source, Tool};
use crate::canvas_view::{CanvasCancel, CanvasDelete, CanvasRedo, CanvasUndo, fill_path, ink_hsla};
use crate::preview::{PreviewEdit, ResizeSender};
use crate::preview_rich::{Block, Rich};
use crate::theme::Colors;
use crate::zoom::px;

/// How long after a stroke ends the sketch is written into the note.
pub const SAVE_AFTER: Duration = Duration::from_secs(2);

/// How long a write the editor hasn't echoed back is still believed.
const ECHO_WITHIN: Duration = Duration::from_secs(3);

/// Shows every sketch's toolbar, as a hover would, for the snapshots.
#[cfg(test)]
pub(crate) static REVEAL_TOOLS: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// What a read-only sketch says.
pub const SHARED_WORDS: &str = "Shared note: copy it to a canvas to draw together.";

/// What the note's text now says about a sketch the view already holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sync {
    /// The note holds what the view holds, or the view's own edit is on its
    /// way: nothing to do.
    Keep,
    /// The note changed under the view (an undo, a sync, a text edit) and the
    /// view has nothing of its own in flight: show the note's.
    Adopt,
}

/// What to do when the preview hands a view the fence now in the note. The
/// view's own unsaved or just-sent ink is never replaced by the older text
/// that was in the note meanwhile; a fence that already holds everything
/// sent (the editor merged it with another writer's) is taken at once.
/// `covers` says whether `incoming` holds all of what was sent. Pure.
pub fn reconcile(
    held: &str,
    incoming: &str,
    drawing: bool,
    unsaved: bool,
    sent: Option<(&str, Duration)>,
    covers: bool,
) -> Sync {
    if held == incoming || drawing || unsaved {
        return Sync::Keep;
    }
    match sent {
        // The editor hasn't taken the write yet (or refused it): the text
        // is older than the view, for a moment.
        Some((json, since)) if json != incoming && !covers && since < ECHO_WITHIN => Sync::Keep,
        _ => Sync::Adopt,
    }
}

/// Whether the fence `incoming` holds everything in `sent`: merging the two
/// changes nothing. Pure.
fn holds(incoming: &str, sent: &str) -> bool {
    let (Ok(incoming), Ok(sent)) = (
        serde_json::from_str::<Canvas>(incoming),
        serde_json::from_str::<Canvas>(sent),
    ) else {
        return false;
    };
    kanso_core::canvas::merge(&sent, &incoming) == incoming
}

/// Where a view lives: the preview that draws it (its
/// [`Rich::sketch_owner`]), the note it is in and its ID.
type Key = (u64, Option<String>, String);

/// The views, by where they live.
#[derive(Default)]
struct Registry(HashMap<Key, Entity<SketchView>>);

impl Global for Registry {}

/// A name for a new preview's sketches, never given twice.
pub fn next_owner() -> u64 {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

/// A sketch's ink that has not reached the note: its ID, its JSON, and the
/// JSON of the fence it was drawn from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unsaved {
    pub id: String,
    pub json: String,
    pub base: String,
}

fn views(cx: &mut App, keep: impl Fn(&Key) -> bool) -> Vec<Entity<SketchView>> {
    cx.default_global::<Registry>()
        .0
        .iter()
        .filter(|(key, _)| keep(key))
        .map(|(_, view)| view.clone())
        .collect()
}

/// Takes the ink of `note`'s sketches in preview `owner` that has not been
/// written yet, a stroke still being drawn included, as if it had been sent:
/// the editor writes it before it leaves the note, closes or quits.
pub fn take_unsaved(owner: u64, note: Option<&str>, cx: &mut App) -> Vec<Unsaved> {
    views(cx, |(o, n, _)| *o == owner && n.as_deref() == note)
        .into_iter()
        .filter_map(|view| view.update(cx, |view, cx| view.take_unsaved(cx)))
        .collect()
}

/// Forgets preview `owner`'s views of notes other than `note`.
pub fn keep_only(owner: u64, note: Option<&str>, cx: &mut App) {
    cx.default_global::<Registry>()
        .0
        .retain(|key, _| kept(key, owner, note));
}

/// Whether the view at `key` stays when preview `owner` moves to `note`:
/// another preview's always do (a second window on another note keeps its
/// unsaved ink), and this one's only on that note. Pure.
fn kept((o, n, _): &Key, owner: u64, note: Option<&str>) -> bool {
    *o != owner || n.as_deref() == note
}

/// Forgets every view of preview `owner`: its note was locked, so neither
/// its ink nor its undo steps stay in memory in the clear.
pub fn forget(owner: u64, cx: &mut App) {
    cx.default_global::<Registry>()
        .0
        .retain(|(o, _, _), _| *o != owner);
}

/// Draws a short line at height `y` in sketch `id` of `note` in preview
/// `owner`, as a mouse would, and ends it: its write is then waiting two
/// seconds. Whether the sketch was there to draw in.
#[cfg(test)]
pub fn draw_for_test(owner: u64, note: &str, id: &str, y: f64, cx: &mut TestAppContext) -> bool {
    cx.update(|cx| {
        let key = (owner, Some(note.to_owned()), id.to_owned());
        let Some(view) = cx.default_global::<Registry>().0.get(&key).cloned() else {
            return false;
        };
        view.update(cx, |view, cx| {
            let now = SketchView::now(cx);
            view.gesture = Some(Source::Mouse);
            let _ = view.state.begin(PenSample::mouse(10., y), 1., now);
            let _ = view.state.extend(PenSample::mouse(60., y + 4.), 1., now);
            let _ = view.state.extend(PenSample::mouse(120., y + 8.), 1., now);
            // Ended by the hand-over, not here: a stroke still being drawn
            // when the window closes is kept too.
        });
        true
    })
}

/// [`take_unsaved`], for a test.
#[cfg(test)]
pub fn take_unsaved_for_test(owner: u64, note: &str, cx: &mut TestAppContext) -> Vec<Unsaved> {
    cx.update(|cx| take_unsaved(owner, Some(note), cx))
}

/// Whether a sketch in note `id` has ink the note doesn't hold yet, in any
/// window: being drawn, waiting for its write, or written a moment ago and
/// not yet in the note. The handwriting reader leaves such a note alone.
pub fn busy(id: &str, cx: &App) -> bool {
    cx.try_global::<Registry>().is_some_and(|registry| {
        registry
            .0
            .iter()
            .filter(|((_, n, _), _)| n.as_deref() == Some(id))
            .any(|(_, view)| {
                let view = view.read(cx);
                view.gesture.is_some()
                    || view.unsaved
                    || view
                        .sent
                        .as_ref()
                        .is_some_and(|(_, at)| at.elapsed() < ECHO_WITHIN)
            })
    })
}

/// The element for `block`, a sketch, in the preview of `rich`'s note.
pub fn element(
    block: &Block,
    at: u64,
    rich: &Arc<Rich>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    let Some(canvas) = serde_json::from_str::<Canvas>(&block.source)
        .ok()
        .filter(|c| c.validate_sketch().is_ok())
    else {
        return div().into_any_element();
    };
    let id = canvas.kanso_sketch_id.clone().unwrap_or_default();
    let note = rich.note.lock().ok().and_then(|note| note.clone());
    let read_only = rich.sketch_read_only();
    let scale = rich.text_scale();
    let key = (rich.sketch_owner, note, id.clone());
    let existing = cx.default_global::<Registry>().0.get(&key).cloned();
    let view = match existing {
        Some(view) => view,
        None => {
            let edits = rich.edits.clone();
            let json = block.source.to_string();
            let view = cx.new(|cx| SketchView::new(id, canvas, json, edits, cx));
            cx.default_global::<Registry>().0.insert(key, view.clone());
            view
        }
    };
    view.update(cx, |view, cx| {
        view.sync(&block.source, read_only, scale, cx)
    });
    let _ = (window, at);
    view.into_any_element()
}

pub struct SketchView {
    id: String,
    state: CanvasState,
    /// The JSON the view shows, as the note holds it or as the view last
    /// wrote it.
    json: String,
    /// The fence the view's ink is drawn on top of: what the note held when
    /// the view last took it in, or what the view last sent. Sent with each
    /// write, so the editor merges rather than writes over a fence that
    /// changed meanwhile.
    base: String,
    /// The ink not yet sent to the editor.
    unsaved: bool,
    /// What was sent and when, until the note shows it.
    sent: Option<(String, Instant)>,
    save: Option<Task<()>>,
    edits: ResizeSender,
    read_only: bool,
    scale: f32,
    gesture: Option<Source>,
    /// The eraser ring under the pointer, in the box's points.
    hover: Option<(f64, f64)>,
    /// The pointer is over the sketch: the toolbar shows.
    over: bool,
    bounds: Rc<Cell<Bounds<Pixels>>>,
    notice: Option<SharedString>,
    focus: FocusHandle,
}

impl SketchView {
    fn new(
        id: String,
        canvas: Canvas,
        json: String,
        edits: ResizeSender,
        cx: &mut Context<Self>,
    ) -> Self {
        let nearly_full = sketch::budget(json.len()) != kanso_core::canvas::Budget::Ok;
        SketchView {
            notice: nearly_full.then(|| sketch::SKETCH_NEARLY_FULL.into()),
            id,
            state: CanvasState::for_sketch(canvas),
            base: json.clone(),
            json,
            unsaved: false,
            sent: None,
            save: None,
            edits,
            read_only: false,
            scale: 1.,
            gesture: None,
            hover: None,
            over: false,
            bounds: Rc::new(Cell::new(Bounds::default())),
            focus: cx.focus_handle(),
        }
    }

    /// The fence now in the note (`json`) and how it may be used.
    fn sync(&mut self, json: &str, read_only: bool, scale: f32, cx: &mut Context<Self>) {
        let mut changed = false;
        if self.read_only != read_only {
            self.read_only = read_only;
            if read_only {
                self.state.cancel();
                self.gesture = None;
            }
            changed = true;
        }
        if (self.scale - scale).abs() > f32::EPSILON {
            self.scale = scale;
            changed = true;
        }
        let covers = self
            .sent
            .as_ref()
            .is_some_and(|(sent, _)| sent != json && holds(json, sent));
        let decision = reconcile(
            &self.json,
            json,
            self.gesture.is_some(),
            self.unsaved,
            self.sent
                .as_ref()
                .map(|(json, at)| (json.as_str(), at.elapsed())),
            covers,
        );
        // The note shows what was sent: it has been taken.
        if self.sent.as_ref().is_some_and(|(sent, _)| sent == json) {
            self.sent = None;
        }
        if decision == Sync::Adopt
            && let Ok(canvas) = serde_json::from_str::<Canvas>(json)
            && canvas.validate_sketch().is_ok()
        {
            let (tool, color, width) =
                (self.state.tool, self.state.color.clone(), self.state.width);
            self.state = CanvasState::for_sketch(canvas);
            self.state.tool = tool;
            self.state.color = color;
            self.state.width = width;
            self.json = json.to_owned();
            self.base = json.to_owned();
            self.sent = None;
            self.warn_when_nearly_full(json.len());
            changed = true;
        }
        if changed {
            cx.notify();
        }
    }

    /// The ink not sent yet, marked as sent: the JSON to write, and the fence
    /// it was drawn on.
    fn take(&mut self) -> Unsaved {
        self.unsaved = false;
        self.save = None;
        let json = sketch::to_json(&self.state.canvas);
        let base = std::mem::replace(&mut self.base, json.clone());
        self.json = json.clone();
        self.sent = Some((json.clone(), Instant::now()));
        self.warn_when_nearly_full(json.len());
        Unsaved {
            id: self.id.clone(),
            json,
            base,
        }
    }

    /// Hands over the ink not yet sent, for the editor to write before it
    /// leaves the note, closes or quits. A stroke still being drawn is ended
    /// and kept, not dropped.
    fn take_unsaved(&mut self, cx: &mut Context<Self>) -> Option<Unsaved> {
        if self.gesture.is_some() {
            self.finish(cx);
        }
        if !self.unsaved {
            return None;
        }
        let unsaved = self.take();
        cx.notify();
        Some(unsaved)
    }

    /// Sends the sketch to the editor now.
    fn flush(&mut self, cx: &mut Context<Self>) {
        self.save = None;
        if !self.unsaved || self.gesture.is_some() {
            return;
        }
        let Unsaved { id, json, base } = self.take();
        self.edits
            .try_send(PreviewEdit::Sketch { id, json, base })
            .ok();
        cx.notify();
    }

    /// Says when the sketch passes 75% of its cap, as a canvas does.
    fn warn_when_nearly_full(&mut self, bytes: usize) {
        let nearly = sketch::budget(bytes) != kanso_core::canvas::Budget::Ok;
        let warning: SharedString = sketch::SKETCH_NEARLY_FULL.into();
        if nearly && self.notice.is_none() {
            self.notice = Some(warning);
        } else if !nearly && self.notice.as_ref() == Some(&warning) {
            self.notice = None;
        }
    }

    /// A stroke (or an erase) ended: write once, [`SAVE_AFTER`] from now,
    /// unless a write is already waiting.
    fn schedule(&mut self, cx: &mut Context<Self>) {
        self.unsaved = true;
        if self.save.is_some() {
            return;
        }
        self.save = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(SAVE_AFTER).await;
            this.update(cx, |this, cx| {
                this.save = None;
                if this.gesture.is_some() {
                    // Never mid-stroke: try again once it ends.
                    this.schedule(cx);
                } else {
                    this.flush(cx);
                }
            })
            .ok();
        }));
    }

    fn now(cx: &App) -> i64 {
        crate::clock::Clock::now_ms(cx)
    }

    /// The box's points per screen point: the box is drawn at its size times
    /// the text scale, smaller to fit a narrow note.
    fn zoom(&self) -> f64 {
        let (w, _) = self.size();
        let shown = f64::from(f32::from(self.bounds.get().size.width));
        if shown > 1. && w > 0. {
            shown / w
        } else {
            f64::from(self.scale)
        }
    }

    fn size(&self) -> (f64, f64) {
        self.state
            .sketch
            .map_or((0., 0.), |s| (f64::from(s.w), f64::from(s.h)))
    }

    /// `position` (in the window) in the box's points.
    fn to_box(&self, position: Point<Pixels>) -> (f64, f64) {
        let b = self.bounds.get();
        let k = self.zoom();
        (
            f64::from(f32::from(position.x - b.origin.x)) / k,
            f64::from(f32::from(position.y - b.origin.y)) / k,
        )
    }

    fn sample(&self, position: Point<Pixels>, source: Source) -> PenSample {
        let (x, y) = self.to_box(position);
        let mut sample = PenSample::mouse(x, y);
        sample.source = source;
        if let Some(pen) = crate::pen_input::current() {
            sample.pressure = pen.pressure;
            sample.tilt = pen.tilt;
            if pen.eraser {
                sample.source = Source::PenEraser;
            } else if pen.in_proximity {
                sample.source = Source::Pen;
            }
        }
        sample
    }

    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        window.focus(&self.focus, cx);
        if self.read_only || event.button != MouseButton::Left {
            return;
        }
        let sample = self.sample(event.position, Source::Mouse);
        self.gesture = Some(sample.source);
        self.notice = None;
        let radius = ERASER_RADIUS / self.zoom();
        let now = Self::now(cx);
        let outcome = self.state.begin(sample, radius, now);
        self.report(outcome, false, cx);
        cx.stop_propagation();
    }

    fn on_mouse_move(&mut self, event: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        if self.read_only {
            return;
        }
        let (x, y) = self.to_box(event.position);
        if let Some(source) = self.gesture
            && event.pressed_button == Some(MouseButton::Left)
        {
            let sample = self.sample(event.position, source);
            let radius = ERASER_RADIUS / self.zoom();
            let now = Self::now(cx);
            let outcome = self.state.extend(sample, radius, now);
            self.hover = Some((x, y));
            self.report(outcome, false, cx);
            return;
        }
        if self.gesture.is_some() {
            // The button went up outside the box.
            self.finish(cx);
        }
        let erasing = matches!(self.state.tool, Tool::Eraser | Tool::Partial);
        let next = erasing.then_some((x, y));
        if next != self.hover {
            self.hover = next;
            cx.notify();
        }
    }

    fn on_mouse_up(&mut self, _: &MouseUpEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.finish(cx);
    }

    fn finish(&mut self, cx: &mut Context<Self>) {
        if self.gesture.take().is_none() {
            return;
        }
        let now = Self::now(cx);
        let outcome = self.state.end(now);
        self.report(outcome, true, cx);
    }

    fn report(&mut self, outcome: Outcome, ended: bool, cx: &mut Context<Self>) {
        match outcome {
            Outcome::Changed if ended => {
                self.schedule(cx);
                let bytes = sketch::to_json(&self.state.canvas).len();
                self.warn_when_nearly_full(bytes);
            }
            Outcome::Changed => self.unsaved = true,
            Outcome::Nothing => {}
            Outcome::Refused(why) => {
                self.notice = match why {
                    Refused::Full => Some(sketch::SKETCH_FULL.into()),
                    Refused::TooLong => Some("That stroke is too long.".into()),
                    Refused::Empty => None,
                };
            }
        }
        cx.notify();
    }

    fn set_tool(&mut self, tool: Tool, cx: &mut Context<Self>) {
        self.state.cancel();
        self.state.tool = tool;
        self.state.selection.clear();
        self.hover = None;
        self.gesture = None;
        cx.notify();
    }

    fn undo(&mut self, cx: &mut Context<Self>) {
        let now = Self::now(cx);
        let outcome = self.state.undo(now);
        self.report(outcome, true, cx);
    }

    fn redo(&mut self, cx: &mut Context<Self>) {
        let now = Self::now(cx);
        let outcome = self.state.redo(now);
        self.report(outcome, true, cx);
    }

    fn delete(&mut self, cx: &mut Context<Self>) {
        if self.read_only {
            return;
        }
        let now = Self::now(cx);
        let outcome = self.state.delete_selection(now);
        self.report(outcome, true, cx);
    }

    /// "Open as canvas" (or "Copy as canvas" when read only): what is drawn
    /// goes to the note first, then the editor makes the canvas.
    fn open_as_canvas(&mut self, cx: &mut Context<Self>) {
        self.flush(cx);
        self.edits
            .try_send(PreviewEdit::SketchToCanvas {
                id: self.id.clone(),
            })
            .ok();
    }

    fn toolbar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let tool = self.state.tool;
        let icon_button = |id: &'static str, icon: Lucide, tip: &'static str| {
            Button::new(id).ghost().small().icon(icon).tooltip(tip)
        };
        let tool_button = |t: Tool, id: &'static str, icon: Lucide, cx: &mut Context<Self>| {
            icon_button(id, icon, t.label())
                .selected(tool == t)
                .on_click(cx.listener(move |this, _, _, cx| this.set_tool(t, cx)))
        };
        let swatch = |color: &'static str, cx: &mut Context<Self>| {
            let chosen = self.state.color == color;
            let shade = pen::parse_hex(color).map_or(0, |(r, g, b)| {
                (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b)
            });
            div()
                .id(SharedString::from(format!("sketch-colour-{color}")))
                .size(px(14.))
                .rounded_full()
                .bg(Hsla::from(rgb(shade)))
                .border_2()
                .border_color(if chosen {
                    colors.accent
                } else {
                    colors.divider
                })
                .cursor_pointer()
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.state.color = color.into();
                    cx.notify();
                }))
        };
        let divider = || div().w(px(1.)).h(px(16.)).mx(px(4.)).bg(colors.divider);
        // In a narrow preview the tools give way (the colours last in the
        // row first) and "Open as canvas" always shows whole.
        let tools = h_flex()
            .flex_1()
            .min_w_0()
            .overflow_hidden()
            .items_center()
            .gap(px(2.))
            .when(!self.read_only, |el| {
                el.child(tool_button(Tool::Pen, "sketch-pen", Lucide::PenTool, cx))
                    .child(tool_button(
                        Tool::Pencil,
                        "sketch-pencil",
                        Lucide::Pencil,
                        cx,
                    ))
                    .child(tool_button(
                        Tool::Marker,
                        "sketch-marker",
                        Lucide::Brush,
                        cx,
                    ))
                    .child(tool_button(
                        Tool::Highlighter,
                        "sketch-highlighter",
                        Lucide::Droplet,
                        cx,
                    ))
                    .child(divider())
                    .child(tool_button(
                        Tool::Eraser,
                        "sketch-eraser",
                        Lucide::Trash,
                        cx,
                    ))
                    .child(tool_button(
                        Tool::Partial,
                        "sketch-partial",
                        Lucide::Split,
                        cx,
                    ))
                    .child(tool_button(Tool::Lasso, "sketch-lasso", Lucide::Focus, cx))
                    .child(divider())
                    .child(
                        icon_button("sketch-undo", Lucide::Undo2, "Undo")
                            .disabled(!self.state.can_undo())
                            .on_click(cx.listener(|this, _, _, cx| this.undo(cx))),
                    )
                    .child(
                        icon_button("sketch-redo", Lucide::Redo2, "Redo")
                            .disabled(!self.state.can_redo())
                            .on_click(cx.listener(|this, _, _, cx| this.redo(cx))),
                    )
                    .child(divider())
                    .children(COLORS.iter().map(|&c| swatch(c, cx).into_any_element()))
            });
        h_flex()
            .h(px(TOOLBAR_HEIGHT))
            .w_full()
            .items_center()
            .gap(px(2.))
            .child(tools)
            .child(
                Button::new("sketch-canvas")
                    .flex_none()
                    .ghost()
                    .small()
                    .icon(Lucide::PenTool)
                    .label(if self.read_only {
                        "Copy as canvas"
                    } else {
                        "Open as canvas"
                    })
                    .tooltip(if self.read_only {
                        "Make a private canvas from this sketch, to keep drawing"
                    } else {
                        "Turn this sketch into a canvas note, with no limit"
                    })
                    .on_click(cx.listener(|this, _, _, cx| this.open_as_canvas(cx))),
            )
    }
}

/// The toolbar's height above the box, in points: kept even when the toolbar
/// is hidden, so the note does not move when the pointer comes over.
const TOOLBAR_HEIGHT: f32 = 30.;

/// One stroke ready to fill.
struct Ink {
    polygon: Arc<Vec<[f64; 2]>>,
    color: Hsla,
    opacity: f32,
}

impl Render for SketchView {
    // no-prop-test: layout and painting; `reconcile` and `canvas_state.rs`
    // have the properties and the snapshots show the widget in a note.
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let preview = crate::themes::preview_colors(cx);
        let dark = gpui_kit::component::Theme::global(cx).is_dark();
        let (w, h) = self.size();
        let bounds_cell = self.bounds.clone();
        let surface: Hsla = rgb(preview.code_background.0).into();
        let border: Hsla = rgb(preview.border.0).into();
        let accent: Hsla = colors.accent.into();
        let muted: Hsla = colors.text_muted.into();
        let k = self.zoom();
        let ring = (ERASER_RADIUS * k) as f32;
        let hover = self
            .hover
            .filter(|_| matches!(self.state.tool, Tool::Eraser | Tool::Partial));
        // The strokes: finished ones from their cached outlines, then the one
        // in progress.
        let mut selected: Vec<[f32; 4]> = Vec::new();
        let mut strokes: Vec<Ink> = self
            .state
            .visible_ink(([0., 0.], [w as f32, h as f32]))
            .into_iter()
            .map(|ink| {
                if let Some(b) = ink.selected {
                    selected.push([b.x as f32, b.y as f32, b.right() as f32, b.bottom() as f32]);
                }
                Ink {
                    polygon: Arc::new(
                        ink.polygon
                            .iter()
                            .map(|p| [p[0] + ink.origin.0, p[1] + ink.origin.1])
                            .collect(),
                    ),
                    color: ink_hsla(&ink.color, dark),
                    opacity: pen::opacity(&ink.tool) as f32,
                }
            })
            .collect();
        if let Some((stroke, polygon)) = self.state.live_outline(&[]) {
            strokes.push(Ink {
                polygon: Arc::new(polygon),
                color: ink_hsla(&stroke.color, dark),
                opacity: pen::opacity(&stroke.tool) as f32,
            });
        }
        let lasso = self.state.lasso.clone();
        let editable = !self.read_only;
        // A refusal shows the tools with it; the nearly-full warning alone
        // waits for the pointer, as the toolbar does.
        let warning_only = self.notice.as_deref() == Some(sketch::SKETCH_NEARLY_FULL);
        let show_bar =
            self.over || self.gesture.is_some() || (self.notice.is_some() && !warning_only);
        #[cfg(test)]
        let show_bar = show_bar || REVEAL_TOOLS.load(std::sync::atomic::Ordering::Relaxed);
        let box_el = canvas(
            move |bounds, _, _| bounds_cell.set(bounds),
            move |bounds, _, window, _cx| {
                let origin = bounds.origin;
                let to_px = |x: f64, y: f64| {
                    point(
                        origin.x + gpui_kit::px((x * k) as f32),
                        origin.y + gpui_kit::px((y * k) as f32),
                    )
                };
                window.paint_quad(
                    fill(bounds, surface)
                        .corner_radii(gpui_kit::px(8.))
                        .border_widths(gpui_kit::px(1.))
                        .border_color(border),
                );
                window.with_content_mask(Some(ContentMask { bounds }), |window| {
                    for ink in &strokes {
                        if let Some(path) = fill_path(&ink.polygon, (0., 0.), &to_px) {
                            window.paint_path(path, ink.color.opacity(ink.opacity));
                        }
                    }
                    for r in &selected {
                        let (a, b) = (
                            to_px(f64::from(r[0]), f64::from(r[1])),
                            to_px(f64::from(r[2]), f64::from(r[3])),
                        );
                        window.paint_quad(
                            fill(Bounds::from_corners(a, b), accent.opacity(0.12))
                                .corner_radii(gpui_kit::px(3.)),
                        );
                    }
                    if lasso.len() > 1 {
                        let mut l = PathBuilder::stroke(gpui_kit::px(1.2));
                        let mut it = lasso.iter();
                        if let Some(first) = it.next() {
                            l.move_to(to_px(first.0, first.1));
                            for p in it {
                                l.line_to(to_px(p.0, p.1));
                            }
                        }
                        if let Ok(path) = l.build() {
                            window.paint_path(path, accent);
                        }
                    }
                });
            },
        )
        .absolute()
        .size_full();
        let drawing_box = div()
            .id("sketch-surface")
            .debug_selector(|| "sketch-surface".into())
            .key_context("CanvasView")
            .track_focus(&self.focus)
            .relative()
            .w(px(w as f32 * self.scale))
            .max_w(relative(1.))
            .aspect_ratio((w / h.max(1.)) as f32)
            .cursor(if editable {
                CursorStyle::Crosshair
            } else {
                CursorStyle::Arrow
            })
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_action(cx.listener(|this, _: &CanvasUndo, _, cx| this.undo(cx)))
            .on_action(cx.listener(|this, _: &CanvasRedo, _, cx| this.redo(cx)))
            .on_action(cx.listener(|this, _: &CanvasDelete, _, cx| this.delete(cx)))
            .on_action(cx.listener(|this, _: &CanvasCancel, _, cx| {
                this.state.cancel();
                this.state.selection.clear();
                this.gesture = None;
                this.report(Outcome::Nothing, true, cx);
            }))
            .child(box_el)
            .when_some(hover, |el, (x, y)| {
                el.child(
                    div()
                        .absolute()
                        .left(px((x * k) as f32 - ring))
                        .top(px((y * k) as f32 - ring))
                        .size(px(ring * 2.))
                        .rounded_full()
                        .border_1()
                        .border_color(muted),
                )
            });
        let words: Option<SharedString> = if self.read_only {
            Some(SHARED_WORDS.into())
        } else {
            self.notice.clone()
        };
        v_flex()
            .id("sketch")
            .py(px(4.))
            .w_full()
            .gap(px(0.))
            .on_hover(cx.listener(|this, over: &bool, _, cx| {
                this.over = *over;
                if !*over {
                    // Leaving the box sends what was drawn.
                    this.hover = None;
                    this.flush(cx);
                }
                cx.notify();
            }))
            .child(
                div()
                    .h(px(TOOLBAR_HEIGHT))
                    .w_full()
                    .opacity(if show_bar { 1. } else { 0. })
                    .child(self.toolbar(cx)),
            )
            .child(drawing_box)
            .when_some(words, |el, words| {
                el.child(
                    div()
                        .pt(px(4.))
                        .text_size(px(12.))
                        .text_color(colors.text_muted)
                        .child(words),
                )
            })
    }
}

impl Focusable for SketchView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use proptest::prelude::*;

    use super::{ECHO_WITHIN, Sync, holds, kept, reconcile};

    type Case = (String, String, bool, bool, Option<(String, Duration)>, bool);

    fn arb_case() -> impl Strategy<Value = Case> {
        (
            "[ab]{0,3}",
            "[ab]{0,3}",
            any::<bool>(),
            any::<bool>(),
            prop::option::of(
                ("[ab]{0,3}", 0u64..6000).prop_map(|(j, ms)| (j, Duration::from_millis(ms))),
            ),
            any::<bool>(),
        )
    }

    fn check((held, incoming, drawing, unsaved, sent, covers): Case) -> Result<(), TestCaseError> {
        let sent_ref = sent.as_ref().map(|(j, d)| (j.as_str(), *d));
        let decision = reconcile(&held, &incoming, drawing, unsaved, sent_ref, covers);
        let echo_pending = sent
            .as_ref()
            .is_some_and(|(json, since)| *json != incoming && !covers && *since < ECHO_WITHIN);
        let keeps = held == incoming || drawing || unsaved || echo_pending;
        prop_assert_eq!(decision, if keeps { Sync::Keep } else { Sync::Adopt });
        // The same text is never adopted again.
        prop_assert_eq!(
            reconcile(&incoming, &incoming, drawing, unsaved, sent_ref, covers),
            Sync::Keep
        );
        Ok(())
    }

    fn stroke(y: f64) -> kanso_core::canvas::Stroke {
        use kanso_core::canvas::{Point, Stroke};
        Stroke {
            tool: "pen".into(),
            color: "#1e1e1e".into(),
            width: 3.,
            points: vec![Point::new(4., y, 0.5), Point::new(60., y + 5., 0.5)],
            ..Stroke::default()
        }
    }

    proptest! {
        /// A view never gives up ink it is drawing, has not sent, or sent a
        /// moment ago, for the older text of the note; and with nothing of
        /// its own in flight it always takes a changed note.
        #[test]
        fn prop_own_ink_is_never_replaced_by_older_text(case in arb_case()) {
            check(case)?;
        }

        /// A preview moving to another note lets go only of its own views of
        /// other notes; another window's views, on any note, all stay.
        #[test]
        fn prop_a_preview_lets_go_only_of_its_own_views(
            owner in 1u64..4,
            other in 1u64..4,
            note in prop::option::of("[ab]"),
            theirs in prop::option::of("[ab]"),
        ) {
            let key = (other, theirs.clone(), "sk".to_owned());
            let stays = kept(&key, owner, note.as_deref());
            prop_assert_eq!(stays, other != owner || theirs == note);
            prop_assert!(kept(&key, other.wrapping_add(10), note.as_deref()));
        }

        /// A fence holds what was sent exactly when it is the sent sketch
        /// merged with another writer's: the editor's merge is taken at once,
        /// and the older fence it replaced never is.
        #[test]
        fn prop_a_merge_holds_what_was_sent(mine in 1usize..4, theirs in 0usize..4) {
            use kanso_core::canvas::{merge, sketch};
            let base = sketch::new_sketch("sk-h", 320, 240);
            let mut sent = base.clone();
            for i in 0..mine {
                let _ = sent.draw(stroke(10. * i as f64), 10 + i as i64);
            }
            let mut other = base.clone();
            for i in 0..theirs {
                let _ = other.draw(stroke(150. + 10. * i as f64), 20 + i as i64);
            }
            let merged = merge(&sent, &other);
            let json = |c| sketch::to_json(c);
            prop_assert!(holds(&json(&merged), &json(&sent)));
            prop_assert!(holds(&json(&sent), &json(&sent)));
            prop_assert!(!holds(&json(&base), &json(&sent)));
            prop_assert_eq!(holds(&json(&other), &json(&sent)), false);
        }
    }
}
