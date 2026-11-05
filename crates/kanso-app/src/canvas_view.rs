//! The canvas, drawn: an endless surface with cards, arrows and ink, in the
//! place of the text editor for a note whose `doctype` is `canvas`.
//!
//! The note editor owns the note (its title, notebook, tags, history,
//! locking and saving); this view holds the canvas document being drawn on
//! (`canvas_state.rs`) and tells the editor when it changed
//! ([`CanvasEvent::Changed`]). Pan with two fingers, the wheel, middle-drag
//! or Space-drag; zoom with ⌘-scroll or a pinch; Fit shows everything. Ink is
//! filled outlines from `kanso_render::freehand` (the same ones the phone
//! and the SVG export draw); only what is on screen is painted.

use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{
    Disableable as _, Selectable as _, Sizable as _, StyledExt as _, h_flex, v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_collab::canvas_doc::StrokeHead;
use kanso_collab::{Engine, RemotePointer};
use kanso_core::canvas::{self, Budget, Node, Refused};
use kanso_render::canvas::{card_color, pen};
use kanso_store::Store;

use crate::canvas_state::{
    COLORS, CanvasState, ERASER_RADIUS, Outcome, PenSample, Source, Tool, WIDTHS,
};
use crate::graph::Viewport;
use crate::layout::TOP_BAND_HEIGHT;
use crate::theme::Colors;
use crate::zoom::px;

actions!(
    kanso,
    [
        CanvasUndo,
        CanvasRedo,
        CanvasDelete,
        CanvasFit,
        CanvasCancel
    ]
);

/// How far one press of a zoom button zooms.
const ZOOM_STEP: f32 = 1.4;
/// Below this zoom the minimap shows.
const MINIMAP_BELOW: f32 = 0.16;
/// Cards draw no text below this zoom: it would be a smudge.
const TEXT_ZOOM: f32 = 0.3;
/// A new card's text, until it is edited.
const NEW_CARD_TEXT: &str = "Text";
/// How often the stroke being drawn streams to the others on a shared canvas.
const STREAM_EVERY: Duration = Duration::from_millis(150);

/// A canvas drawn on with others (docs: issue #45, section 4): the engine
/// holds the document, this view hands it what was drawn and takes back
/// what everyone has drawn.
struct Shared {
    engine: Arc<Engine>,
    share_id: String,
    /// This account, the author of the strokes streamed from here.
    me: String,
    /// Where the others' pointers are.
    remote: Vec<RemotePointer>,
    /// When the stroke being drawn last streamed.
    streamed: Option<Instant>,
    /// Others' edits came in while a gesture was going: they are taken once
    /// it ends.
    behind: bool,
    /// The canvas is past what a note can hold.
    full: bool,
}

pub fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("secondary-z", CanvasUndo, Some("CanvasView")),
        KeyBinding::new("secondary-shift-z", CanvasRedo, Some("CanvasView")),
        KeyBinding::new("secondary-y", CanvasRedo, Some("CanvasView")),
        KeyBinding::new("backspace", CanvasDelete, Some("CanvasView")),
        KeyBinding::new("delete", CanvasDelete, Some("CanvasView")),
        KeyBinding::new("secondary-0", CanvasFit, Some("CanvasView")),
        KeyBinding::new("escape", CanvasCancel, Some("CanvasView")),
    ]);
}

/// What the view tells the note editor that holds it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CanvasEvent {
    /// The document changed; `body` is the note body to save. `stroke_ended`
    /// asks for a save now (a stroke or an erase just finished); otherwise
    /// the editor waits for two quiet seconds.
    Changed { body: String, stroke_ended: bool },
    /// The person asked for the raw JSON, to repair it.
    EditAsText,
    /// A note card was opened: show that note.
    OpenNote(String),
    /// Something to tell the person.
    Notice(String),
}

impl EventEmitter<CanvasEvent> for CanvasView {}

/// A gesture that moves the view or a card instead of drawing.
#[derive(Clone, Copy)]
enum Grab {
    Pan {
        last: [f32; 2],
    },
    /// A card follows the pointer from where it was pressed: `start` is the
    /// press, in view points, and `origin` the card's corner then, so a slow
    /// drag at a high zoom still moves it (each step rounds from the start).
    Card {
        start: [f32; 2],
        origin: (f64, f64),
        moved: bool,
    },
}

pub struct CanvasView {
    store: Arc<Store>,
    note_id: Option<String>,
    state: Option<CanvasState>,
    /// Why the body isn't a canvas, when it isn't.
    broken: Option<String>,
    read_only: bool,
    /// Live titles of note cards, by note ID.
    titles: HashMap<String, String>,
    grab: Option<Grab>,
    /// The source of the gesture drawing now.
    gesture: Option<Source>,
    space: bool,
    /// The eraser's ring under the pointer, in canvas coordinates.
    hover: Option<(f64, f64)>,
    bounds: Rc<Cell<Bounds<Pixels>>>,
    /// Fit the view at the next paint.
    fit: Rc<Cell<bool>>,
    /// The card a double click edits is the one pressed.
    pressed_card: Option<String>,
    /// The pointer over the surface, in view points: where a drop lands.
    pointer: [f32; 2],
    /// The document's size against its budget, as of the last change: the
    /// toolbar shows it every frame, and measuring means writing the JSON.
    budget: Budget,
    /// A drag (an eraser, a lasso move) changed the document and the editor
    /// hasn't been told: it is, with the body, when the gesture ends, not on
    /// every pointer move.
    unsaved: bool,
    /// Set while the canvas is shared and open: the engine is its source.
    shared: Option<Shared>,
    focus: FocusHandle,
}

impl Focusable for CanvasView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl CanvasView {
    pub fn new(store: Arc<Store>, cx: &mut Context<Self>) -> Self {
        CanvasView {
            store,
            note_id: None,
            state: None,
            broken: None,
            read_only: false,
            titles: HashMap::new(),
            grab: None,
            gesture: None,
            space: false,
            hover: None,
            bounds: Rc::new(Cell::new(Bounds::default())),
            fit: Rc::new(Cell::new(true)),
            pressed_card: None,
            pointer: [0., 0.],
            budget: Budget::Ok,
            unsaved: false,
            shared: None,
            focus: cx.focus_handle(),
        }
    }

    /// Shows the canvas in `body` for note `id`. The same note again keeps
    /// the view and the undo steps; a body that changed underneath (a sync)
    /// is taken in.
    pub fn open(&mut self, id: &str, body: &str, read_only: bool, cx: &mut Context<Self>) {
        self.read_only = read_only;
        let same = self.note_id.as_deref() == Some(id);
        if same && self.state.as_ref().is_some_and(|s| s.body() == body) {
            return;
        }
        // A shared canvas on screen is the engine's: the library's body is
        // its projection, a moment behind what is drawn.
        if same && self.shared.is_some() && self.state.is_some() {
            return;
        }
        self.note_id = Some(id.to_owned());
        self.grab = None;
        self.gesture = None;
        self.unsaved = false;
        self.budget = canvas::budget(body.len());
        self.refresh_titles();
        match CanvasState::from_body(body) {
            Ok(mut next) => {
                if let Some(old) = self.state.take().filter(|_| same) {
                    next.view = old.view;
                    next.tool = old.tool;
                    next.color = old.color;
                    next.width = old.width;
                } else {
                    self.fit.set(true);
                }
                self.state = Some(next);
                self.broken = None;
            }
            Err(why) => {
                self.state = None;
                self.broken = Some(why);
            }
        }
        cx.notify();
    }

    /// Shows nothing: no canvas note is open.
    pub fn clear(&mut self, cx: &mut Context<Self>) {
        self.unshare(cx);
        self.note_id = None;
        self.state = None;
        self.broken = None;
        cx.notify();
    }

    /// Draws on shared canvas `share_id` with the others in it: the canvas
    /// is the engine's from now on.
    pub fn share(&mut self, engine: Arc<Engine>, share_id: &str, cx: &mut Context<Self>) {
        let Ok(canvas) = engine.canvas(share_id) else {
            return;
        };
        if let Some(state) = &mut self.state {
            state.take_canvas(canvas);
            self.budget = canvas::budget(state.body().len());
        }
        self.shared = Some(Shared {
            me: engine.caller().account_id.clone(),
            remote: engine.pointers(share_id),
            share_id: share_id.to_owned(),
            engine,
            streamed: None,
            behind: false,
            full: false,
        });
        cx.notify();
    }

    /// The canvas stopped being shared (or the note closed): it is a private
    /// canvas again, as it stands.
    pub fn unshare(&mut self, cx: &mut Context<Self>) {
        if let Some(shared) = self.shared.take() {
            // A stroke streamed under a pen that is still down will never
            // be finished here: the others don't keep half of it.
            if shared.streamed.is_some()
                && let Some(id) = self.state.as_ref().and_then(|s| s.live_id())
                && let Err(err) = shared.engine.abandon_stroke(&shared.share_id, id)
            {
                eprintln!("kanso: taking back a stroke on the shared canvas: {err}");
            }
            shared.engine.set_pointer(&shared.share_id, None);
            cx.notify();
        }
    }

    /// Makes the view read only (a canvas shared to look at), or not.
    pub fn set_read_only(&mut self, read_only: bool, cx: &mut Context<Self>) {
        if self.read_only != read_only {
            self.read_only = read_only;
            cx.notify();
        }
    }

    /// Others' edits are waiting, or this view drew something: hands the
    /// canvas to the engine and takes back what it is now. Waits for a
    /// gesture in progress to end.
    pub fn sync_shared(&mut self, cx: &mut Context<Self>) {
        if self.gesture.is_some() || self.grab.is_some() {
            if let Some(shared) = &mut self.shared {
                shared.behind = true;
            }
            return;
        }
        self.sync_now(cx);
    }

    /// Takes in what others did while a gesture or a drag held it back,
    /// once nothing is in hand: whatever way the gesture ended (a pan, a
    /// card let go without moving, a stroke erased under the pen).
    fn catch_up(&mut self, cx: &mut Context<Self>) {
        if self.gesture.is_none()
            && self.grab.is_none()
            && self.shared.as_ref().is_some_and(|s| s.behind)
        {
            self.sync_now(cx);
        }
    }

    fn sync_now(&mut self, cx: &mut Context<Self>) {
        let Self { shared, state, .. } = self;
        let (Some(shared), Some(state)) = (shared.as_mut(), state.as_mut()) else {
            return;
        };
        shared.behind = false;
        let mut notice = None;
        match shared.engine.sync_canvas(&shared.share_id, &state.canvas) {
            Ok(merge) => {
                let full = merge.full;
                if merge.changed {
                    state.take_canvas(merge.canvas);
                }
                if full && !shared.full {
                    notice = Some(kanso_collab::engine::canvas::FULL.to_owned());
                }
                shared.full = full;
                self.budget = if full {
                    Budget::Full
                } else {
                    canvas::budget(state.body().len())
                };
            }
            Err(err) => eprintln!("kanso: syncing the shared canvas: {err}"),
        }
        shared.remote = shared.engine.pointers(&shared.share_id);
        if let Some(words) = notice {
            cx.emit(CanvasEvent::Notice(words));
        }
        cx.notify();
    }

    /// Others' pointers moved, came or went.
    pub fn refresh_pointers(&mut self, cx: &mut Context<Self>) {
        if let Some(shared) = &mut self.shared {
            let remote = shared.engine.pointers(&shared.share_id);
            if remote != shared.remote {
                shared.remote = remote;
                cx.notify();
            }
        }
    }

    /// Tells the room where the pointer is, in canvas coordinates.
    fn send_pointer(&self, position: Point<Pixels>) {
        let (Some(shared), Some(state)) = (&self.shared, &self.state) else {
            return;
        };
        let (x, y) = self.to_canvas(position);
        let tool = format!("{:?}", state.tool).to_lowercase();
        shared
            .engine
            .set_pointer(&shared.share_id, Some((x, y, tool.as_str())));
    }

    /// The pointer left the surface: the others' marks of it go.
    fn pointer_left(&self) {
        if let Some(shared) = &self.shared {
            shared.engine.set_pointer(&shared.share_id, None);
        }
    }

    /// Streams the stroke being drawn to the others, a chunk about every
    /// 150 ms, so they see the ink as it is made.
    fn stream_live(&mut self, cx: &mut Context<Self>) {
        // A full canvas takes no more ink (the pen's stroke is refused when
        // it ends), so none streams either.
        if self
            .shared
            .as_ref()
            .is_none_or(|s| s.full || s.streamed.is_some_and(|at| at.elapsed() < STREAM_EVERY))
        {
            return;
        }
        let now = Self::now(cx);
        let Some((stream, made_layer)) = self.state.as_mut().and_then(|s| s.streaming(now)) else {
            return;
        };
        // An ink layer made just now goes first: the points are relative to
        // it, and the others hide ink that has none.
        if made_layer {
            self.sync_now(cx);
        }
        let Some(shared) = &mut self.shared else {
            return;
        };
        shared.streamed = Some(Instant::now());
        let head = StrokeHead {
            id: stream.id,
            ink: stream.ink,
            owner: shared.me.clone(),
            tool: stream.tool,
            color: stream.color,
            width: stream.width,
            z: stream.z,
        };
        if let Ok(Err(_erased)) =
            shared
                .engine
                .stream_stroke(&shared.share_id, &head, &stream.points)
        {
            // Someone erased the stroke under the pen: it stops there, and
            // the erase comes in now.
            shared.streamed = None;
            shared.behind = true;
            if let Some(state) = &mut self.state {
                state.cancel();
            }
            self.gesture = None;
            self.catch_up(cx);
            cx.notify();
        }
    }

    fn refresh_titles(&mut self) {
        self.titles = self
            .store
            .note_titles()
            .map(|all| {
                all.into_iter()
                    .map(|(id, title)| (id.strip_prefix("note:").unwrap_or(&id).to_owned(), title))
                    .collect()
            })
            .unwrap_or_default();
    }

    fn now(cx: &App) -> i64 {
        crate::clock::Clock::now_ms(cx)
    }

    fn size(&self) -> [f32; 2] {
        let b = self.bounds.get();
        [f32::from(b.size.width), f32::from(b.size.height)]
    }

    /// `position` (in the window) in view points.
    fn local(&self, position: Point<Pixels>) -> [f32; 2] {
        let b = self.bounds.get();
        [
            f32::from(position.x - b.origin.x),
            f32::from(position.y - b.origin.y),
        ]
    }

    /// `position` on the canvas.
    fn to_canvas(&self, position: Point<Pixels>) -> (f64, f64) {
        let at = self.local(position);
        let p = self
            .state
            .as_ref()
            .map_or(Viewport::default(), |s| s.view)
            .to_layout(at, self.size());
        (f64::from(p[0]), f64::from(p[1]))
    }

    /// The eraser's radius on the canvas, from its size on the screen.
    fn eraser_radius(&self) -> f64 {
        let zoom = self.state.as_ref().map_or(1.0, |s| f64::from(s.view.zoom));
        ERASER_RADIUS / zoom
    }

    /// Takes what an edit did. A change in the middle of a drag is only
    /// noted; one that ends a gesture (or a drag's earlier changes, once it
    /// ends) is sent to the editor with the body.
    fn report(&mut self, outcome: Outcome, ended: bool, cx: &mut Context<Self>) {
        match outcome {
            Outcome::Changed if !ended => self.unsaved = true,
            Outcome::Changed => self.emit_changed(cx),
            Outcome::Nothing => {
                if ended && self.unsaved {
                    self.emit_changed(cx);
                }
            }
            Outcome::Refused(why) => {
                if ended && self.unsaved {
                    self.emit_changed(cx);
                }
                let words = match why {
                    Refused::Full => {
                        "This canvas is full (1 MB). Erase some ink or move cards to a new canvas."
                    }
                    Refused::TooLong => "That stroke is too long.",
                    Refused::Empty => "",
                };
                if !words.is_empty() {
                    cx.emit(CanvasEvent::Notice(words.into()));
                }
            }
        }
        cx.notify();
    }

    /// Tells the editor the document changed, with the body to save now.
    fn emit_changed(&mut self, cx: &mut Context<Self>) {
        self.unsaved = false;
        // A shared canvas goes to the engine, which writes the note.
        if self.shared.is_some() {
            self.sync_shared(cx);
            return;
        }
        let Some(state) = &self.state else { return };
        let body = state.body();
        self.budget = canvas::budget(body.len());
        cx.emit(CanvasEvent::Changed {
            body,
            stroke_ended: true,
        });
    }

    // ---- pointer ----

    fn sample(&self, event_position: Point<Pixels>, source: Source) -> PenSample {
        let (x, y) = self.to_canvas(event_position);
        let mut sample = PenSample::mouse(x, y);
        sample.source = source;
        // A tablet's pressure and tilt, from the platform shim, when there is one.
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
        if self.state.is_none() {
            return;
        }
        let at = self.local(event.position);
        if event.button == MouseButton::Middle || self.space {
            self.grab = Some(Grab::Pan { last: at });
            cx.stop_propagation();
            return;
        }
        if event.button != MouseButton::Left {
            return;
        }
        let sample = self.sample(event.position, Source::Mouse);
        let (x, y) = (sample.x, sample.y);
        // A double click on a text card edits it, whatever the tool.
        if event.click_count >= 2
            && let Some(card) = self.state.as_ref().and_then(|s| s.card_at(x, y)).cloned()
        {
            if !self.read_only {
                self.edit_card(&card, window, cx);
            }
            return;
        }
        if self.read_only {
            self.grab = Some(Grab::Pan { last: at });
            return;
        }
        let tool = self.state.as_ref().map(|s| s.tool).unwrap_or(Tool::Pen);
        if tool == Tool::Text {
            match self.state.as_ref().and_then(|s| s.card_at(x, y)).cloned() {
                Some(card) => self.edit_card(&card, window, cx),
                None => self.new_card(x, y, window, cx),
            }
            return;
        }
        // A press on a card with the lasso moves the card.
        if tool == Tool::Lasso
            && let Some(card) = self.state.as_ref().and_then(|s| s.card_at(x, y))
            && !self.state.as_ref().is_some_and(|s| {
                s.selection_bounds()
                    .is_some_and(|b| x >= b.x && x <= b.right() && y >= b.y && y <= b.bottom())
            })
        {
            let (id, origin) = (card.id.clone(), (card.x, card.y));
            self.pressed_card = Some(id.clone());
            if let Some(state) = &mut self.state {
                state.select_card(Some(id));
            }
            self.grab = Some(Grab::Card {
                start: at,
                origin,
                moved: false,
            });
            return;
        }
        self.gesture = Some(sample.source);
        let radius = self.eraser_radius();
        let now = Self::now(cx);
        if let Some(state) = &mut self.state {
            let outcome = state.begin(sample, radius, now);
            self.report(outcome, false, cx);
        }
        cx.stop_propagation();
    }

    fn on_mouse_move(&mut self, event: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        let at = self.local(event.position);
        self.pointer = at;
        self.send_pointer(event.position);
        if let Some(grab) = self.grab {
            if event.pressed_button.is_none() {
                // The button went up where no mouse-up reached the view: a
                // card drag still ends here, as one saved undo step.
                if let Some(Grab::Card { moved, .. }) = self.grab.take() {
                    self.end_card_grab(moved, cx);
                }
                self.catch_up(cx);
                return;
            }
            match grab {
                Grab::Pan { last } => {
                    if let Some(state) = &mut self.state {
                        state.view = state.view.panned([at[0] - last[0], at[1] - last[1]]);
                    }
                    self.grab = Some(Grab::Pan { last: at });
                }
                Grab::Card {
                    start,
                    origin,
                    moved: was,
                } => {
                    let zoom = self.state.as_ref().map_or(1.0, |s| f64::from(s.view.zoom));
                    let (dx, dy) = (
                        f64::from(at[0] - start[0]) / zoom,
                        f64::from(at[1] - start[1]) / zoom,
                    );
                    let now = Self::now(cx);
                    let mut moved = false;
                    if let (Some(id), Some(state)) = (&self.pressed_card, &mut self.state)
                        && let Some(node) = state.canvas.node(id)
                    {
                        let to = ((origin.0 + dx).round(), (origin.1 + dy).round());
                        moved = (node.x, node.y) != to && state.move_card(id, to.0, to.1, now);
                    }
                    self.grab = Some(Grab::Card {
                        start,
                        origin,
                        moved: was || moved,
                    });
                }
            }
            cx.notify();
            return;
        }
        let (x, y) = self.to_canvas(event.position);
        if self.gesture.is_some() && event.pressed_button == Some(MouseButton::Left) {
            let sample = self.sample(event.position, self.gesture.unwrap_or(Source::Mouse));
            let radius = self.eraser_radius();
            let now = Self::now(cx);
            if let Some(state) = &mut self.state {
                let outcome = state.extend(sample, radius, now);
                self.hover = Some((x, y));
                self.report(outcome, false, cx);
            }
            self.stream_live(cx);
            return;
        }
        if self.gesture.is_some() {
            // The button went up outside the view.
            self.finish(cx);
        }
        let erasing = self
            .state
            .as_ref()
            .is_some_and(|s| matches!(s.tool, Tool::Eraser | Tool::Partial));
        let next = erasing.then_some((x, y));
        if next != self.hover {
            self.hover = next;
            cx.notify();
        }
    }

    fn on_mouse_up(&mut self, _: &MouseUpEvent, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(Grab::Card { moved, .. }) = self.grab.take() {
            self.end_card_grab(moved, cx);
        } else {
            self.finish(cx);
        }
        self.catch_up(cx);
    }

    /// Lets go of a pressed card: its drag becomes one undo step, saved if
    /// it moved.
    fn end_card_grab(&mut self, moved: bool, cx: &mut Context<Self>) {
        self.pressed_card = None;
        if let Some(state) = &mut self.state {
            state.end_card_drag();
        }
        if moved {
            self.emit_changed(cx);
        }
        cx.notify();
    }

    fn finish(&mut self, cx: &mut Context<Self>) {
        if self.gesture.take().is_none() {
            return;
        }
        // The stroke that streamed to the others while the pen was down.
        let streamed = self
            .shared
            .as_ref()
            .filter(|s| s.streamed.is_some())
            .and_then(|_| self.state.as_ref()?.live_id().map(str::to_owned));
        let now = Self::now(cx);
        let outcome = match &mut self.state {
            Some(state) => state.end(now),
            None => Outcome::Nothing,
        };
        let kept = matches!(outcome, Outcome::Changed);
        // Only a gesture that changed the document (now, or in an earlier
        // step of the drag) asks for a save: a lasso that only selected
        // writes nothing.
        self.report(outcome, true, cx);
        if let Some(shared) = &mut self.shared {
            shared.streamed = None;
            // A streamed stroke the canvas refused when it ended (too long,
            // the canvas full) is taken back from the others too.
            if let Some(id) = streamed.filter(|_| !kept) {
                if let Err(err) = shared.engine.abandon_stroke(&shared.share_id, &id) {
                    eprintln!("kanso: taking back a stroke on the shared canvas: {err}");
                }
                shared.behind = true;
            }
        }
        // What others drew while the pen was down comes in now.
        self.catch_up(cx);
    }

    fn on_scroll(&mut self, event: &ScrollWheelEvent, _: &mut Window, cx: &mut Context<Self>) {
        let size = self.size();
        let at = self.local(event.position);
        let Some(state) = &mut self.state else { return };
        let view = state.view;
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
            state.view = next;
            cx.notify();
        }
        cx.stop_propagation();
    }

    fn on_pinch(&mut self, event: &PinchEvent, _: &mut Window, cx: &mut Context<Self>) {
        let size = self.size();
        let at = self.local(event.position);
        if let Some(state) = &mut self.state {
            state.view = state.view.zoom_at(1.0 + event.delta, at, size);
            cx.notify();
        }
        cx.stop_propagation();
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        if event.keystroke.key == "space" && !event.is_held {
            self.space = true;
            cx.notify();
        }
    }

    fn on_key_up(&mut self, event: &KeyUpEvent, _: &mut Window, cx: &mut Context<Self>) {
        if event.keystroke.key == "space" {
            self.space = false;
            cx.notify();
        }
    }

    // ---- cards ----

    /// Notes dragged from the list become note cards where they are dropped.
    /// A locked note, or one in a locked notebook, is refused: its title
    /// would otherwise be written into the canvas in the clear.
    pub(crate) fn drop_notes(&mut self, ids: &[String], cx: &mut Context<Self>) {
        if self.read_only {
            return;
        }
        let at = self
            .state
            .as_ref()
            .map_or(Viewport::default(), |s| s.view)
            .to_layout(self.pointer, self.size());
        let (mut x, y) = (f64::from(at[0]), f64::from(at[1]));
        let locked_books = self.store.locked_books().unwrap_or_default();
        let now = Self::now(cx);
        let mut added = false;
        let mut skipped = 0;
        for id in ids {
            if self.note_id.as_deref() == Some(id.as_str()) {
                skipped += 1;
                continue;
            }
            let Ok(Some(note)) = self.store.note(id) else {
                continue;
            };
            if note.is_locked() || locked_books.contains(&note.book_id) || note.is_trashed() {
                skipped += 1;
                continue;
            }
            let title = if note.title.trim().is_empty() {
                "Untitled".to_owned()
            } else {
                note.title.clone()
            };
            if let Some(state) = &mut self.state
                && state.add_note_card(x, y, id, &title, now).is_some()
            {
                added = true;
                x += 280.;
            }
        }
        if added {
            self.emit_changed(cx);
        }
        if skipped > 0 {
            cx.emit(CanvasEvent::Notice(
                "A locked, trashed or this very note can't be a card.".into(),
            ));
        }
        self.refresh_titles();
        cx.notify();
    }

    fn new_card(&mut self, x: f64, y: f64, window: &mut Window, cx: &mut Context<Self>) {
        let now = Self::now(cx);
        let Some(state) = &mut self.state else { return };
        let id = state.add_text_card(x, y, NEW_CARD_TEXT, now);
        if let Some(card) = state.canvas.node(&id).cloned() {
            self.emit_changed(cx);
            cx.notify();
            self.edit_card(&card, window, cx);
        }
    }

    fn edit_card(&mut self, card: &Node, window: &mut Window, cx: &mut Context<Self>) {
        if card.kind != "text" {
            // A note card opens its note.
            if let Some(note) = canvas::Canvas::note_id(card) {
                let id = format!("note:{}", note.trim_start_matches("note:"));
                cx.emit(CanvasEvent::OpenNote(id));
            }
            return;
        }
        let this = cx.entity();
        let id = card.id.clone();
        crate::dialogs::prompt_text(
            "Edit Card",
            card.text.as_deref().unwrap_or(""),
            "Save",
            window,
            cx,
            move |text, _, cx| {
                this.update(cx, |view, cx| {
                    let now = Self::now(cx);
                    if let Some(state) = &mut view.state
                        && state.set_card_text(&id, &text, now)
                    {
                        view.emit_changed(cx);
                        cx.notify();
                    }
                });
            },
        );
    }

    // ---- commands ----

    pub(crate) fn undo(&mut self, cx: &mut Context<Self>) {
        let now = Self::now(cx);
        if let Some(state) = &mut self.state {
            let outcome = state.undo(now);
            self.report(outcome, true, cx);
        }
    }

    fn redo(&mut self, cx: &mut Context<Self>) {
        let now = Self::now(cx);
        if let Some(state) = &mut self.state {
            let outcome = state.redo(now);
            self.report(outcome, true, cx);
        }
    }

    fn delete(&mut self, cx: &mut Context<Self>) {
        let now = Self::now(cx);
        if self.read_only {
            return;
        }
        if let Some(state) = &mut self.state {
            let outcome = state.delete_selection(now);
            self.report(outcome, true, cx);
        }
    }

    pub(crate) fn set_tool(&mut self, tool: Tool, cx: &mut Context<Self>) {
        if let Some(state) = &mut self.state {
            state.cancel();
            state.tool = tool;
            state.selection.clear();
            state.card_selection = None;
            self.hover = None;
            self.gesture = None;
            // A drag cut short keeps what it erased: save it.
            self.report(Outcome::Nothing, true, cx);
        }
    }

    fn zoom_by(&mut self, factor: f32, cx: &mut Context<Self>) {
        let size = self.size();
        if let Some(state) = &mut self.state {
            state.view = state
                .view
                .zoom_at(factor, [size[0] / 2., size[1] / 2.], size);
            cx.notify();
        }
    }

    fn fit(&mut self, cx: &mut Context<Self>) {
        self.fit.set(true);
        cx.notify();
    }

    // ---- painting ----

    fn render_toolbar(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let state = self.state.as_ref();
        let tool = state.map(|s| s.tool);
        let icon_button = |id: &'static str, icon: Lucide, tip: &'static str| {
            Button::new(id).ghost().small().icon(icon).tooltip(tip)
        };
        let tool_button = |t: Tool, id: &'static str, icon: Lucide, cx: &mut Context<Self>| {
            icon_button(id, icon, t.label())
                .selected(tool == Some(t))
                .on_click(cx.listener(move |this, _, _, cx| this.set_tool(t, cx)))
        };
        let swatch = |color: &'static str, cx: &mut Context<Self>| {
            let chosen = state.is_some_and(|s| s.color == color);
            let shade = pen::parse_hex(color).map_or(0, |(r, g, b)| {
                (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b)
            });
            div()
                .id(SharedString::from(format!("canvas-colour-{color}")))
                .size(px(18.))
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
                    if let Some(s) = &mut this.state {
                        s.color = color.into();
                        cx.notify();
                    }
                }))
        };
        let width_dot = |w: f64, cx: &mut Context<Self>| {
            let chosen = state.is_some_and(|s| (s.width - w).abs() < 1e-9);
            div()
                .id(SharedString::from(format!("canvas-width-{w}")))
                .size(px(22.))
                .flex()
                .items_center()
                .justify_center()
                .rounded(px(5.))
                .when(chosen, |el| el.bg(colors.selected))
                .cursor_pointer()
                .child(
                    div()
                        .size(px((w as f32 + 2.).min(14.)))
                        .rounded_full()
                        .bg(colors.text),
                )
                .on_click(cx.listener(move |this, _, _, cx| {
                    if let Some(s) = &mut this.state {
                        s.width = w;
                        cx.notify();
                    }
                }))
        };
        let divider = || div().w(px(1.)).h(px(18.)).mx(px(4.)).bg(colors.divider);
        let (can_undo, can_redo) = state.map_or((false, false), |s| (s.can_undo(), s.can_redo()));
        let budget = state.map(|_| self.budget);
        h_flex()
            .h(px(TOP_BAND_HEIGHT - 8.))
            .flex_none()
            .px(px(12.))
            .gap(px(2.))
            .border_b_1()
            .border_color(colors.divider)
            .bg(colors.editor)
            .when(!self.read_only, |el| {
                el.child(tool_button(Tool::Pen, "canvas-pen", Lucide::PenTool, cx))
                    .child(tool_button(
                        Tool::Pencil,
                        "canvas-pencil",
                        Lucide::Pencil,
                        cx,
                    ))
                    .child(tool_button(
                        Tool::Marker,
                        "canvas-marker",
                        Lucide::Brush,
                        cx,
                    ))
                    .child(tool_button(
                        Tool::Highlighter,
                        "canvas-highlighter",
                        Lucide::Droplet,
                        cx,
                    ))
                    .child(divider())
                    .child(tool_button(
                        Tool::Eraser,
                        "canvas-eraser",
                        Lucide::Trash,
                        cx,
                    ))
                    .child(tool_button(
                        Tool::Partial,
                        "canvas-partial",
                        Lucide::Split,
                        cx,
                    ))
                    .child(tool_button(Tool::Lasso, "canvas-lasso", Lucide::Focus, cx))
                    .child(tool_button(Tool::Text, "canvas-text", Lucide::Type, cx))
                    .child(divider())
                    .children(COLORS.iter().map(|&c| swatch(c, cx).into_any_element()))
                    .child(divider())
                    .children(WIDTHS.iter().map(|w| width_dot(*w, cx).into_any_element()))
                    .child(divider())
                    .child(
                        icon_button("canvas-undo", Lucide::Undo2, "Undo")
                            .disabled(!can_undo)
                            .on_click(cx.listener(|this, _, _, cx| this.undo(cx))),
                    )
                    .child(
                        icon_button("canvas-redo", Lucide::Redo2, "Redo")
                            .disabled(!can_redo)
                            .on_click(cx.listener(|this, _, _, cx| this.redo(cx))),
                    )
            })
            .child(div().flex_1())
            .when(budget == Some(Budget::Warn), |el| {
                el.child(
                    div()
                        .text_size(px(12.))
                        .text_color(colors.text_muted)
                        .child("Getting large: 700 KB of 1 MB"),
                )
            })
            .when(budget == Some(Budget::Full), |el| {
                el.child(
                    div()
                        .text_size(px(12.))
                        .text_color(colors.text_muted)
                        .child("Full: new ink is refused"),
                )
            })
            .child(
                icon_button("canvas-zoom-out", Lucide::ZoomOut, "Zoom out")
                    .on_click(cx.listener(|this, _, _, cx| this.zoom_by(1. / ZOOM_STEP, cx))),
            )
            .child(
                icon_button("canvas-zoom-in", Lucide::ZoomIn, "Zoom in")
                    .on_click(cx.listener(|this, _, _, cx| this.zoom_by(ZOOM_STEP, cx))),
            )
            .child(
                icon_button("canvas-fit", Lucide::Scan, "Fit")
                    .on_click(cx.listener(|this, _, _, cx| this.fit(cx))),
            )
            .child(
                icon_button("canvas-text-edit", Lucide::Code, "Edit as text")
                    .on_click(cx.listener(|_, _, _, cx| cx.emit(CanvasEvent::EditAsText))),
            )
            .pl(px(
                crate::layout::inset_after(0., window.is_fullscreen()).min(12.)
            ))
    }

    /// Cards and groups, as elements placed over the surface.
    fn cards(&self, view: Viewport, size: [f32; 2], colors: &Colors) -> Vec<AnyElement> {
        let Some(state) = &self.state else {
            return Vec::new();
        };
        let seen = view.visible(size, 40.);
        let mut out = Vec::new();
        for node in state.canvas.live_nodes() {
            if node.kind == canvas::INK {
                continue;
            }
            let b = node.bounds();
            if (b.right() as f32) < seen.0[0]
                || (b.x as f32) > seen.1[0]
                || (b.bottom() as f32) < seen.0[1]
                || (b.y as f32) > seen.1[1]
            {
                continue;
            }
            let tl = view.to_screen([b.x as f32, b.y as f32], size);
            let (w, h) = (b.width as f32 * view.zoom, b.height as f32 * view.zoom);
            let tint = card_color(node.color.as_deref())
                .and_then(|c| pen::parse_hex(&c))
                .map(|(r, g, b)| rgb((u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b)));
            let held = state.card_selection.as_deref() == Some(node.id.as_str());
            let border: Hsla = if held {
                colors.accent.into()
            } else {
                tint.map_or(Hsla::from(colors.divider), Hsla::from)
            };
            let group = node.kind == "group";
            let text = match node.kind.as_str() {
                "text" => node.text.clone().unwrap_or_default(),
                "group" => node.label.clone().unwrap_or_default(),
                _ => node
                    .file
                    .as_deref()
                    .and_then(|f| f.strip_prefix(canvas::NOTE_PREFIX))
                    .and_then(|id| self.titles.get(id).cloned())
                    .or_else(|| node.title.clone())
                    .or_else(|| node.url.clone())
                    .or_else(|| node.file.clone())
                    .unwrap_or_default(),
            };
            let font = (14. * view.zoom).clamp(1., 60.);
            out.push(
                div()
                    .absolute()
                    .left(gpui_kit::px(tl[0]))
                    .top(gpui_kit::px(tl[1]))
                    .w(gpui_kit::px(w))
                    .h(gpui_kit::px(h))
                    .overflow_hidden()
                    .rounded(gpui_kit::px(8. * view.zoom))
                    .border(gpui_kit::px((1.5 * view.zoom).max(0.5)))
                    .border_color(border)
                    .when(!group, |el| el.bg(colors.fill))
                    .when(group, |el| el.bg(border.opacity(0.06)))
                    .when(view.zoom >= TEXT_ZOOM, |el| {
                        el.p(gpui_kit::px(10. * view.zoom))
                            .text_size(gpui_kit::px(font))
                            .text_color(if group {
                                colors.text_muted
                            } else {
                                colors.text
                            })
                            .when(node.kind == "file" || node.kind == "link", |el| {
                                el.font_semibold()
                            })
                            .child(text)
                    })
                    .into_any_element(),
            );
        }
        out
    }

    fn paint_surface(&mut self, dark: bool, size: [f32; 2]) -> Option<Surface> {
        let view = self.state.as_ref()?.view;
        let seen = view.visible(size, 24.);
        let state = self.state.as_mut()?;
        // Only the strokes in view, each from its cached outline: nothing of
        // a stroke is copied or measured again from frame to frame.
        let mut selected: Vec<[f32; 4]> = Vec::new();
        let strokes: Vec<InkPaint> = state
            .visible_ink(seen)
            .into_iter()
            .map(|ink| {
                if let Some(b) = ink.selected {
                    selected.push([b.x as f32, b.y as f32, b.right() as f32, b.bottom() as f32]);
                }
                InkPaint {
                    polygon: ink.polygon,
                    origin: ink.origin,
                    color: ink_hsla(&ink.color, dark),
                    opacity: pen::opacity(&ink.tool) as f32,
                }
            })
            .collect();
        let live = state.live_outline(&[]).map(|(stroke, polygon)| InkPaint {
            polygon: Arc::new(polygon),
            origin: (0., 0.),
            color: ink_hsla(&stroke.color, dark),
            opacity: pen::opacity(&stroke.tool) as f32,
        });
        let edges = edge_lines(&state.canvas);
        let minimap = (view.zoom < MINIMAP_BELOW)
            .then(|| state.canvas.content_bounds())
            .flatten()
            .map(|b| [b.x as f32, b.y as f32, b.right() as f32, b.bottom() as f32]);
        Some(Surface {
            strokes,
            live,
            edges,
            selected,
            lasso: state.lasso.clone(),
            minimap,
        })
    }
}

/// One stroke ready to fill.
struct InkPaint {
    polygon: Arc<Vec<[f64; 2]>>,
    /// The ink layer's origin.
    origin: (f64, f64),
    color: Hsla,
    opacity: f32,
}

/// What the surface paints this frame.
struct Surface {
    strokes: Vec<InkPaint>,
    live: Option<InkPaint>,
    edges: Vec<[f32; 4]>,
    selected: Vec<[f32; 4]>,
    lasso: Vec<(f64, f64)>,
    minimap: Option<[f32; 4]>,
}

pub(crate) fn ink_hsla(color: &str, dark: bool) -> Hsla {
    // A near-black pen on a dark page draws in the text colour, as the
    // export does (`pen::ink_color`).
    let palette = if dark {
        kanso_render::Palette::DARK
    } else {
        kanso_render::Palette::LIGHT
    };
    let hex = pen::ink_color(color, &palette);
    let (r, g, b) = pen::parse_hex(&hex).unwrap_or((0x1e, 0x1e, 0x1e));
    Hsla::from(rgb((u32::from(r) << 16)
        | (u32::from(g) << 8)
        | u32::from(b)))
}

/// The arrows as straight segments between the cards' nearest sides.
fn edge_lines(canvas: &canvas::Canvas) -> Vec<[f32; 4]> {
    canvas
        .edges
        .iter()
        .filter(|e| !e.deleted)
        .filter_map(|e| {
            let a = canvas.node(&e.from_node)?.bounds();
            let b = canvas.node(&e.to_node)?.bounds();
            let (ac, bc) = (
                (a.x + a.width / 2., a.y + a.height / 2.),
                (b.x + b.width / 2., b.y + b.height / 2.),
            );
            let from = edge_point(&a, bc);
            let to = edge_point(&b, ac);
            Some([from.0 as f32, from.1 as f32, to.0 as f32, to.1 as f32])
        })
        .collect()
}

/// Where the line from a card's middle toward `toward` leaves it.
fn edge_point(b: &canvas::Bounds, toward: (f64, f64)) -> (f64, f64) {
    let (cx, cy) = (b.x + b.width / 2., b.y + b.height / 2.);
    let (dx, dy) = (toward.0 - cx, toward.1 - cy);
    if dx == 0. && dy == 0. {
        return (cx, cy);
    }
    let sx = if dx == 0. {
        f64::MAX
    } else {
        (b.width / 2.) / dx.abs()
    };
    let sy = if dy == 0. {
        f64::MAX
    } else {
        (b.height / 2.) / dy.abs()
    };
    let s = sx.min(sy);
    (cx + dx * s, cy + dy * s)
}

/// A closed outline as one filled path, its corners rounded through the
/// midpoints of its edges.
pub(crate) fn fill_path(
    polygon: &[[f64; 2]],
    origin: (f64, f64),
    to_px: &impl Fn(f64, f64) -> Point<Pixels>,
) -> Option<Path<Pixels>> {
    let n = polygon.len();
    if n < 3 {
        return None;
    }
    let at = |p: [f64; 2]| to_px(p[0] + origin.0, p[1] + origin.1);
    let mid = |a: [f64; 2], b: [f64; 2]| [(a[0] + b[0]) / 2., (a[1] + b[1]) / 2.];
    let mut path = PathBuilder::fill();
    path.move_to(at(mid(polygon[0], polygon[1])));
    for i in 1..=n {
        let (a, b) = (polygon[i % n], polygon[(i + 1) % n]);
        path.curve_to(at(mid(a, b)), at(a));
    }
    path.close();
    path.build().ok()
}

impl Render for CanvasView {
    // no-prop-test: layout and painting; `canvas_state.rs` has the properties
    // and the snapshots show the view.
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        if let Some(why) = self.broken.clone() {
            return v_flex()
                .size_full()
                .items_center()
                .justify_center()
                .gap(px(10.))
                .bg(colors.editor)
                .text_color(colors.text_muted)
                .child("This canvas can't be opened.")
                .child(div().text_size(px(12.)).child(why))
                .child(
                    Button::new("canvas-edit-text")
                        .outline()
                        .small()
                        .label("Edit as text")
                        .on_click(cx.listener(|_, _, _, cx| cx.emit(CanvasEvent::EditAsText))),
                )
                .into_any_element();
        }
        let size = {
            let s = self.size();
            if s[0] > 1. { s } else { [900., 600.] }
        };
        if self.fit.get() && self.bounds.get().size.width > px(1.) {
            self.fit.set(false);
            if let Some(state) = &mut self.state {
                state.fit(size);
            }
        }
        let view = self
            .state
            .as_ref()
            .map_or(Viewport::default(), |state| state.view);
        let dark = gpui_kit::component::Theme::global(cx).is_dark();
        let surface = self.paint_surface(dark, size);
        let cards = self.cards(view, size, colors);
        let bounds_cell = self.bounds.clone();
        let hover = self
            .hover
            .zip(self.state.as_ref())
            .and_then(|(h, s)| matches!(s.tool, Tool::Eraser | Tool::Partial).then_some(h));
        let eraser = ERASER_RADIUS as f32;
        let accent: Hsla = colors.accent.into();
        let muted: Hsla = colors.text_muted.into();
        let grid: Hsla = Hsla::from(colors.divider).opacity(0.55);
        let cursor = match self.state.as_ref().map(|s| s.tool) {
            Some(Tool::Text) => CursorStyle::IBeam,
            Some(Tool::Lasso) => CursorStyle::Arrow,
            _ => CursorStyle::Crosshair,
        };
        // The others' pointers: a dot and a name, in their colour.
        let pointers: Vec<_> = self
            .shared
            .as_ref()
            .map(|shared| {
                shared
                    .remote
                    .iter()
                    .map(|p| {
                        let s = view.to_screen([p.x as f32, p.y as f32], size);
                        let colour = crate::theme::cursor_colour(p.colour, dark);
                        div()
                            .absolute()
                            .left(gpui_kit::px(s[0]))
                            .top(gpui_kit::px(s[1]))
                            .flex()
                            .items_center()
                            .gap(px(4.))
                            .child(div().size(px(9.)).rounded_full().bg(colour))
                            .child(
                                div()
                                    .px(px(5.))
                                    .rounded(px(4.))
                                    .bg(colour)
                                    .text_size(px(10.))
                                    .text_color(gpui::white())
                                    .child(p.name.clone()),
                            )
                    })
                    .collect()
            })
            .unwrap_or_default();
        let surface_el = canvas(
            move |bounds, _, _| {
                bounds_cell.set(bounds);
            },
            move |bounds, _, window, _cx| {
                let origin = bounds.origin;
                let extent = [f32::from(bounds.size.width), f32::from(bounds.size.height)];
                let to_px = |x: f64, y: f64| {
                    let s = view.to_screen([x as f32, y as f32], extent);
                    point(origin.x + gpui_kit::px(s[0]), origin.y + gpui_kit::px(s[1]))
                };
                window.with_content_mask(Some(ContentMask { bounds }), |window| {
                    // A dotted grid, so panning shows movement on an empty page.
                    paint_grid(window, bounds, &view, extent, grid);
                    let Some(surface) = &surface else { return };
                    for ink in surface.strokes.iter().chain(surface.live.iter()) {
                        if let Some(path) = fill_path(&ink.polygon, ink.origin, &to_px) {
                            window.paint_path(path, ink.color.opacity(ink.opacity));
                        }
                    }
                    if !surface.edges.is_empty() {
                        let mut lines = PathBuilder::stroke(gpui_kit::px(1.5 * view.zoom.max(0.5)));
                        for e in &surface.edges {
                            lines.move_to(to_px(f64::from(e[0]), f64::from(e[1])));
                            lines.line_to(to_px(f64::from(e[2]), f64::from(e[3])));
                        }
                        if let Ok(path) = lines.build() {
                            window.paint_path(path, muted);
                        }
                    }
                    for r in &surface.selected {
                        let (a, b) = (
                            to_px(f64::from(r[0]), f64::from(r[1])),
                            to_px(f64::from(r[2]), f64::from(r[3])),
                        );
                        window.paint_quad(
                            fill(Bounds::from_corners(a, b), accent.opacity(0.12))
                                .corner_radii(gpui_kit::px(3.)),
                        );
                    }
                    if surface.lasso.len() > 1 {
                        let mut l = PathBuilder::stroke(gpui_kit::px(1.2));
                        let mut it = surface.lasso.iter();
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
                    if let Some(r) = surface.minimap {
                        paint_minimap(window, bounds, r, &view, extent, accent, muted);
                    }
                });
            },
        )
        .absolute()
        .size_full();

        let body = div()
            .id("canvas-surface")
            .debug_selector(|| "canvas-surface".into())
            .key_context("CanvasView")
            .track_focus(&self.focus)
            .relative()
            .flex_1()
            .min_h_0()
            .overflow_hidden()
            .bg(colors.editor)
            .cursor(
                if self.space || matches!(self.grab, Some(Grab::Pan { .. })) {
                    CursorStyle::OpenHand
                } else {
                    cursor
                },
            )
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_down(MouseButton::Middle, cx.listener(Self::on_mouse_down))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_up(MouseButton::Middle, cx.listener(Self::on_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_scroll_wheel(cx.listener(Self::on_scroll))
            .on_pinch(cx.listener(Self::on_pinch))
            .on_drop(
                cx.listener(|this, notes: &crate::note_list::DraggedNotes, _, cx| {
                    this.drop_notes(&notes.ids, cx)
                }),
            )
            .on_key_down(cx.listener(Self::on_key_down))
            .on_key_up(cx.listener(Self::on_key_up))
            .on_action(cx.listener(|this, _: &CanvasUndo, _, cx| this.undo(cx)))
            .on_action(cx.listener(|this, _: &CanvasRedo, _, cx| this.redo(cx)))
            .on_action(cx.listener(|this, _: &CanvasDelete, _, cx| this.delete(cx)))
            .on_action(cx.listener(|this, _: &CanvasFit, _, cx| this.fit(cx)))
            .on_action(cx.listener(|this, _: &CanvasCancel, _, cx| {
                if this.shared.as_ref().is_some_and(|s| s.streamed.is_some()) {
                    // Its points already went to the others: the stroke
                    // ends where it is, rather than leaving a ghost of it.
                    this.finish(cx);
                    return;
                }
                if let Some(state) = &mut this.state {
                    state.cancel();
                    state.selection.clear();
                    state.card_selection = None;
                }
                this.gesture = None;
                this.report(Outcome::Nothing, true, cx);
            }))
            .on_hover(cx.listener(|this, over: &bool, _, _| {
                if !*over {
                    this.pointer_left();
                }
            }))
            .child(surface_el)
            .children(cards)
            .children(pointers)
            .when_some(hover, |el, (x, y)| {
                let s = view.to_screen([x as f32, y as f32], size);
                el.child(
                    div()
                        .absolute()
                        .left(gpui_kit::px(s[0] - eraser))
                        .top(gpui_kit::px(s[1] - eraser))
                        .size(gpui_kit::px(eraser * 2.))
                        .rounded_full()
                        .border_1()
                        .border_color(muted),
                )
            });

        v_flex()
            .size_full()
            .bg(colors.editor)
            .child(self.render_toolbar(window, cx))
            .child(body)
            .into_any_element()
    }
}

fn paint_grid(
    window: &mut Window,
    bounds: Bounds<Pixels>,
    view: &Viewport,
    extent: [f32; 2],
    color: Hsla,
) {
    // One dot every 32 points, thinned out as the view zooms out.
    let mut step = 32.0_f32;
    while step * view.zoom < 14. {
        step *= 2.;
    }
    if step * view.zoom > 400. {
        return;
    }
    let (lo, hi) = view.visible(extent, 0.);
    let (x0, y0) = ((lo[0] / step).floor() * step, (lo[1] / step).floor() * step);
    let mut x = x0;
    let mut count = 0;
    while x <= hi[0] && count < 6000 {
        let mut y = y0;
        while y <= hi[1] && count < 6000 {
            let s = view.to_screen([x, y], extent);
            window.paint_quad(
                fill(
                    Bounds::new(
                        point(
                            bounds.origin.x + gpui_kit::px(s[0] - 0.8),
                            bounds.origin.y + gpui_kit::px(s[1] - 0.8),
                        ),
                        size(gpui_kit::px(1.6), gpui_kit::px(1.6)),
                    ),
                    color,
                )
                .corner_radii(gpui_kit::px(1.)),
            );
            count += 1;
            y += step;
        }
        x += step;
    }
}

/// A small map of the content with the view on it, when zoomed far out.
fn paint_minimap(
    window: &mut Window,
    bounds: Bounds<Pixels>,
    content: [f32; 4],
    view: &Viewport,
    extent: [f32; 2],
    accent: Hsla,
    muted: Hsla,
) {
    let (w, h) = (160.0_f32, 110.0_f32);
    let origin = point(
        bounds.origin.x + bounds.size.width - gpui_kit::px(w + 14.),
        bounds.origin.y + bounds.size.height - gpui_kit::px(h + 14.),
    );
    window.paint_quad(
        fill(
            Bounds::new(origin, size(gpui_kit::px(w), gpui_kit::px(h))),
            muted.opacity(0.1),
        )
        .corner_radii(gpui_kit::px(6.)),
    );
    let (cw, ch) = (
        (content[2] - content[0]).max(1.),
        (content[3] - content[1]).max(1.),
    );
    let scale = ((w - 12.) / cw).min((h - 12.) / ch);
    let map = |x: f32, y: f32| {
        point(
            origin.x + gpui_kit::px(6. + (x - content[0]) * scale),
            origin.y + gpui_kit::px(6. + (y - content[1]) * scale),
        )
    };
    window.paint_quad(
        fill(
            Bounds::from_corners(map(content[0], content[1]), map(content[2], content[3])),
            muted.opacity(0.3),
        )
        .corner_radii(gpui_kit::px(2.)),
    );
    let (lo, hi) = view.visible(extent, 0.);
    window.paint_quad(outline(
        Bounds::from_corners(map(lo[0], lo[1]), map(hi[0], hi[1])),
        accent,
        BorderStyle::Solid,
    ));
}

#[cfg(test)]
impl CanvasView {
    /// The canvas being drawn on, to set a test up and look at it.
    pub fn state_for_test(&mut self) -> Option<&mut CanvasState> {
        self.state.as_mut()
    }
}
