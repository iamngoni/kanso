//! What drawing on a canvas does, with no screen in it: the tool and pen the
//! person chose, the stroke in progress, the erasers' drag, the lasso and its
//! selection, and undo. `canvas_view.rs` turns pointer events into
//! [`PenSample`]s and paints what this holds; the document edits themselves
//! are `kanso_core::canvas`'s, so every change is stamped for the merge.
//!
//! Coordinates here are the canvas's, in points. The view maps the window to
//! them with [`Viewport`].

use std::collections::HashMap;
use std::sync::Arc;

use kanso_core::canvas::{
    self, Action, Canvas, Node, Point, Refused, Stroke, StrokeRef, unique_id,
};
use kanso_render::canvas::pen;

use crate::graph::Viewport;

/// The most undo steps kept.
const UNDO_DEPTH: usize = 100;
/// Past this many stroke outlines, those of dead strokes are dropped.
const OUTLINES_KEPT: usize = 20_000;
/// The eraser disc's radius, in points on the screen.
pub const ERASER_RADIUS: f64 = 10.;
/// The pen widths offered, in points.
pub const WIDTHS: [f64; 4] = [1.5, 3., 5., 9.];
/// The pen colours offered: JSON Canvas's six and ink.
pub const COLORS: [&str; 7] = [
    "#1e1e1e", "#e03131", "#e8590c", "#f08c00", "#2f9e44", "#1971c2", "#9c36b5",
];

/// What the pointer does on the surface.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Tool {
    Pen,
    Pencil,
    Marker,
    Highlighter,
    /// Takes whole strokes.
    Eraser,
    /// Rubs out the part it touches.
    Partial,
    /// Selects strokes inside a loop.
    Lasso,
    /// Puts a text card where it is clicked.
    Text,
}

impl Tool {
    pub fn label(self) -> &'static str {
        match self {
            Tool::Pen => "Pen",
            Tool::Pencil => "Pencil",
            Tool::Marker => "Marker",
            Tool::Highlighter => "Highlighter",
            Tool::Eraser => "Stroke eraser",
            Tool::Partial => "Partial eraser",
            Tool::Lasso => "Lasso",
            Tool::Text => "Text card",
        }
    }

    /// The stroke tool name a drawing tool writes.
    pub fn pen(self) -> Option<&'static str> {
        match self {
            Tool::Pen => Some("pen"),
            Tool::Pencil => Some("pencil"),
            Tool::Marker => Some("marker"),
            Tool::Highlighter => Some("highlighter"),
            _ => None,
        }
    }
}

/// Where a sample came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    Mouse,
    Pen,
    /// The pen turned over.
    PenEraser,
}

/// One pen sample, in canvas coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PenSample {
    pub x: f64,
    pub y: f64,
    /// 0 to 1; [`pen::SIMULATED`] when the device reports none.
    pub pressure: f64,
    /// Azimuth and altitude in radians, when the device reports a tilt.
    pub tilt: Option<(f64, f64)>,
    pub source: Source,
}

impl PenSample {
    pub fn mouse(x: f64, y: f64) -> PenSample {
        PenSample {
            x,
            y,
            pressure: pen::SIMULATED,
            tilt: None,
            source: Source::Mouse,
        }
    }

    fn point(self) -> Point {
        Point {
            x: self.x,
            y: self.y,
            pressure: self.pressure,
            tilt: self.tilt,
        }
    }
}

/// What a finished gesture changed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Nothing the document needs saving for.
    Nothing,
    /// The document changed: save it.
    Changed,
    /// A stroke wasn't drawn.
    Refused(Refused),
}

/// A canvas open for drawing.
pub struct CanvasState {
    pub canvas: Canvas,
    pub view: Viewport,
    pub tool: Tool,
    pub color: String,
    pub width: f64,
    /// The selection a lasso made.
    pub selection: Vec<StrokeRef>,
    /// The card the lasso last pressed, which Delete removes.
    pub card_selection: Option<String>,
    /// The lasso loop being drawn, in canvas coordinates.
    pub lasso: Vec<(f64, f64)>,
    live: Option<Live>,
    dragging: Option<Drag>,
    undo: Vec<Action>,
    redo: Vec<Action>,
    /// A card drag's moves so far: one undo step once the drag ends.
    card_drag: Action,
    /// Finished strokes as painted, by stroke ID: built once per version.
    outlines: HashMap<String, Painted>,
}

/// A finished stroke as painted: its outline and its rectangle, in its ink
/// layer's coordinates, for the version it was built from.
struct Painted {
    version: u64,
    /// A stroke someone else is still drawing grows without a new version.
    points: usize,
    bounds: Option<canvas::Bounds>,
    polygon: Arc<Vec<[f64; 2]>>,
}

/// A stroke in view, ready to fill.
pub struct VisibleInk {
    pub polygon: Arc<Vec<[f64; 2]>>,
    /// The ink layer's origin.
    pub origin: (f64, f64),
    pub color: String,
    pub tool: String,
    /// The stroke's rectangle in canvas coordinates, when the lasso holds it.
    pub selected: Option<canvas::Bounds>,
}

struct Live {
    points: Vec<Point>,
    tool: Tool,
    /// The stroke's ID from the first sample: on a shared canvas its points
    /// stream to the others under it, and the finished stroke keeps it.
    id: String,
}

/// The stroke being drawn, as a shared canvas streams it: its head and its
/// points so far, relative to the ink node.
pub struct Streaming {
    pub ink: String,
    pub id: String,
    pub tool: String,
    pub color: String,
    pub width: f64,
    pub z: f64,
    pub points: Vec<Point>,
}

/// An eraser or a lasso move in progress.
enum Drag {
    Erase { action: Action, last: (f64, f64) },
    Lasso,
    Move { last: (f64, f64), action: Action },
}

impl CanvasState {
    pub fn new(canvas: Canvas) -> Self {
        CanvasState {
            canvas,
            view: Viewport::default(),
            tool: Tool::Pen,
            color: COLORS[0].into(),
            width: WIDTHS[1],
            selection: Vec::new(),
            card_selection: None,
            lasso: Vec::new(),
            live: None,
            dragging: None,
            undo: Vec::new(),
            redo: Vec::new(),
            card_drag: Action::default(),
            outlines: HashMap::new(),
        }
    }

    /// A state for a note's body; an empty canvas when it holds none. An
    /// unparsable body is an error (the editor offers Edit as Text).
    pub fn from_body(body: &str) -> Result<Self, String> {
        Ok(Self::new(canvas::parse(body)?))
    }

    /// The document as a note body.
    pub fn body(&self) -> String {
        canvas::to_body(&self.canvas)
    }

    #[cfg(test)]
    pub fn budget(&self) -> canvas::Budget {
        canvas::budget(self.body().len())
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    fn push(&mut self, action: Action) {
        if action.is_empty() {
            return;
        }
        self.redo.clear();
        self.undo.push(action);
        if self.undo.len() > UNDO_DEPTH {
            self.undo.remove(0);
        }
    }

    /// The tool a sample is used with: the pen's eraser end rubs out whatever
    /// tool is chosen.
    fn tool_for(&self, sample: &PenSample) -> Tool {
        if sample.source == Source::PenEraser {
            Tool::Partial
        } else {
            self.tool
        }
    }

    /// Starts a gesture at `sample`. `radius` is the eraser's, in canvas
    /// points (the view divides its screen radius by the zoom).
    pub fn begin(&mut self, sample: PenSample, radius: f64, now: i64) -> Outcome {
        self.finish_gesture();
        self.card_selection = None;
        let tool = self.tool_for(&sample);
        match tool {
            Tool::Pen | Tool::Pencil | Tool::Marker | Tool::Highlighter => {
                self.selection.clear();
                let taken: std::collections::HashSet<String> = self
                    .canvas
                    .stroke_ids()
                    .into_iter()
                    .map(str::to_owned)
                    .collect();
                self.live = Some(Live {
                    points: vec![sample.point()],
                    tool,
                    id: unique_id("s", &taken),
                });
                Outcome::Nothing
            }
            Tool::Eraser | Tool::Partial => {
                self.selection.clear();
                self.dragging = Some(Drag::Erase {
                    action: Action::default(),
                    last: (sample.x, sample.y),
                });
                self.erase_step(
                    tool,
                    (sample.x, sample.y),
                    (sample.x, sample.y),
                    radius,
                    now,
                )
            }
            Tool::Lasso => {
                // Pressing inside the selection drags it; elsewhere a new loop.
                if self.inside_selection(sample.x, sample.y) {
                    self.dragging = Some(Drag::Move {
                        last: (sample.x, sample.y),
                        action: Action::default(),
                    });
                } else {
                    self.selection.clear();
                    self.lasso = vec![(sample.x, sample.y)];
                    self.dragging = Some(Drag::Lasso);
                }
                Outcome::Nothing
            }
            Tool::Text => Outcome::Nothing,
        }
    }

    /// Continues the gesture to `sample`. For a live stroke, `predicted`
    /// samples are drawn for the frame only and never stored: pass them to
    /// [`CanvasState::live_outline`] instead.
    pub fn extend(&mut self, sample: PenSample, radius: f64, now: i64) -> Outcome {
        if let Some(live) = &mut self.live {
            live.points.push(sample.point());
            return Outcome::Nothing;
        }
        match self.dragging.take() {
            Some(Drag::Erase { action, last }) => {
                let tool = self.tool_for(&sample);
                let at = (sample.x, sample.y);
                self.dragging = Some(Drag::Erase { action, last: at });
                self.erase_step(tool, last, at, radius, now)
            }
            Some(Drag::Lasso) => {
                if self.lasso.last() != Some(&(sample.x, sample.y)) {
                    self.lasso.push((sample.x, sample.y));
                }
                self.dragging = Some(Drag::Lasso);
                Outcome::Nothing
            }
            Some(Drag::Move { last, action }) => {
                let (dx, dy) = (sample.x - last.0, sample.y - last.1);
                let refs = self.selection.clone();
                let mut action = action;
                let mut moved = Outcome::Nothing;
                if let Some(step) = self.canvas.move_strokes(&refs, dx, dy, now) {
                    // The strokes now have new IDs: follow them.
                    self.selection = step
                        .added
                        .iter()
                        .map(|(node, s)| StrokeRef {
                            node: node.clone(),
                            stroke: s.id.clone(),
                        })
                        .collect();
                    action = action.then(step);
                    moved = Outcome::Changed;
                }
                for p in &mut self.lasso {
                    p.0 += dx;
                    p.1 += dy;
                }
                self.dragging = Some(Drag::Move {
                    last: (sample.x, sample.y),
                    action,
                });
                moved
            }
            None => Outcome::Nothing,
        }
    }

    /// Ends the gesture.
    pub fn end(&mut self, now: i64) -> Outcome {
        if let Some(live) = self.live.take() {
            return self.commit_stroke(live, now);
        }
        match self.dragging.take() {
            Some(Drag::Erase { action, .. }) | Some(Drag::Move { action, .. }) => {
                let changed = !action.is_empty();
                self.push(action);
                if changed {
                    Outcome::Changed
                } else {
                    Outcome::Nothing
                }
            }
            Some(Drag::Lasso) => {
                let polygon = std::mem::take(&mut self.lasso);
                self.selection = self.canvas.strokes_in(&polygon);
                Outcome::Nothing
            }
            None => Outcome::Nothing,
        }
    }

    /// Abandons a gesture (the pointer left the window, or Escape): what a
    /// drag already erased stays erased and can be undone.
    pub fn cancel(&mut self) {
        self.live = None;
        self.finish_gesture();
    }

    fn finish_gesture(&mut self) {
        self.end_card_drag();
        if let Some(Drag::Erase { action, .. }) | Some(Drag::Move { action, .. }) =
            self.dragging.take()
        {
            self.push(action);
        }
        self.lasso.clear();
    }

    fn commit_stroke(&mut self, live: Live, now: i64) -> Outcome {
        let Some(name) = live.tool.pen() else {
            return Outcome::Nothing;
        };
        let stroke = Stroke {
            id: live.id,
            tool: name.into(),
            color: self.color.clone(),
            width: self.width,
            points: live.points,
            ..Stroke::default()
        };
        match self.canvas.draw_as(stroke, now) {
            Ok((_, action)) => {
                self.push(action);
                Outcome::Changed
            }
            Err(refused) => Outcome::Refused(refused),
        }
    }

    fn erase_step(
        &mut self,
        tool: Tool,
        from: (f64, f64),
        to: (f64, f64),
        radius: f64,
        now: i64,
    ) -> Outcome {
        let step = if tool == Tool::Eraser {
            self.canvas.erase_strokes_at(to.0, to.1, radius, now)
        } else {
            self.canvas.erase_along(&[from, to], radius, now)
        };
        let Some(step) = step else {
            return Outcome::Nothing;
        };
        if let Some(Drag::Erase { action, .. }) = &mut self.dragging {
            *action = std::mem::take(action).then(step);
        }
        Outcome::Changed
    }

    /// Undoes the last step.
    pub fn undo(&mut self, now: i64) -> Outcome {
        self.cancel();
        self.card_selection = None;
        let Some(action) = self.undo.pop() else {
            return Outcome::Nothing;
        };
        self.selection.clear();
        let changed = self.canvas.apply(&action, false, now);
        self.redo.push(action);
        if changed {
            Outcome::Changed
        } else {
            Outcome::Nothing
        }
    }

    /// Does the step undo took back.
    pub fn redo(&mut self, now: i64) -> Outcome {
        self.cancel();
        self.card_selection = None;
        let Some(action) = self.redo.pop() else {
            return Outcome::Nothing;
        };
        self.selection.clear();
        let changed = self.canvas.apply(&action, true, now);
        self.undo.push(action);
        if changed {
            Outcome::Changed
        } else {
            Outcome::Nothing
        }
    }

    /// Deletes the selected card, or the lasso's selected strokes.
    pub fn delete_selection(&mut self, now: i64) -> Outcome {
        if let Some(id) = self.card_selection.clone() {
            return if self.delete_card(&id, now) {
                Outcome::Changed
            } else {
                Outcome::Nothing
            };
        }
        let refs = std::mem::take(&mut self.selection);
        match self.canvas.delete_strokes(&refs, now) {
            Some(action) => {
                self.push(action);
                Outcome::Changed
            }
            None => Outcome::Nothing,
        }
    }

    fn inside_selection(&self, x: f64, y: f64) -> bool {
        self.selection_bounds()
            .is_some_and(|b| x >= b.x && x <= b.right() && y >= b.y && y <= b.bottom())
    }

    /// The rectangle round the selection, in canvas coordinates.
    pub fn selection_bounds(&self) -> Option<canvas::Bounds> {
        self.selection
            .iter()
            .filter_map(|r| {
                let node = self.canvas.node(&r.node)?;
                let s = node
                    .strokes
                    .iter()
                    .find(|s| s.id == r.stroke && !s.deleted)?;
                let b = s.bounds()?;
                Some(canvas::Bounds {
                    x: b.x + node.x,
                    y: b.y + node.y,
                    ..b
                })
            })
            .reduce(canvas::Bounds::union)
    }

    /// Adds a text card with its top left at `(x, y)`: one undo step.
    pub fn add_text_card(&mut self, x: f64, y: f64, text: &str, now: i64) -> String {
        self.end_card_drag();
        let taken = self.canvas.nodes.iter().map(|n| n.id.clone()).collect();
        let id = unique_id("t", &taken);
        let action = self.canvas.put_card(
            Node {
                id: id.clone(),
                kind: "text".into(),
                x: x.round(),
                y: y.round(),
                width: 260.0,
                height: 120.0,
                text: Some(text.to_owned()),
                ..Node::default()
            },
            now,
        );
        self.push(action);
        id
    }

    /// Adds a card for note `note_id` (its title kept beside the ID, as the
    /// document defines) with its top left at `(x, y)`: one undo step. `None`
    /// when the ID isn't a note's.
    pub fn add_note_card(
        &mut self,
        x: f64,
        y: f64,
        note_id: &str,
        title: &str,
        now: i64,
    ) -> Option<String> {
        if !note_id.starts_with(canvas::NOTE_PREFIX) {
            return None;
        }
        self.end_card_drag();
        let taken = self.canvas.nodes.iter().map(|n| n.id.clone()).collect();
        let id = unique_id("n", &taken);
        let action = self.canvas.put_card(
            Node {
                id: id.clone(),
                kind: "file".into(),
                file: Some(note_id.to_owned()),
                title: Some(title.to_owned()),
                x: x.round(),
                y: y.round(),
                width: 260.0,
                height: 64.0,
                ..Node::default()
            },
            now,
        );
        self.push(action);
        Some(id)
    }

    /// Changes a text card's text: one undo step.
    pub fn set_card_text(&mut self, id: &str, text: &str, now: i64) -> bool {
        let Some(mut node) = self.canvas.node(id).cloned() else {
            return false;
        };
        if node.kind != "text" {
            return false;
        }
        self.end_card_drag();
        node.text = Some(text.to_owned());
        let action = self.canvas.put_card(node, now);
        self.push(action);
        true
    }

    /// Moves a card or group. The moves of one drag are one undo step, which
    /// [`CanvasState::end_card_drag`] closes (anything else that edits or
    /// undoes closes it too).
    pub fn move_card(&mut self, id: &str, x: f64, y: f64, now: i64) -> bool {
        match self.canvas.move_card(id, x.round(), y.round(), now) {
            Some(step) => {
                self.card_drag = std::mem::take(&mut self.card_drag).then(step);
                true
            }
            None => false,
        }
    }

    /// Ends a card drag: what it moved is one undo step.
    pub fn end_card_drag(&mut self) {
        let drag = std::mem::take(&mut self.card_drag);
        self.push(drag);
    }

    /// Marks `id` as the card Delete removes.
    pub fn select_card(&mut self, id: Option<String>) {
        self.selection.clear();
        self.card_selection = id;
    }

    /// Deletes a card or group; its arrows stay, hidden. One undo step.
    pub fn delete_card(&mut self, id: &str, now: i64) -> bool {
        self.end_card_drag();
        match self.canvas.delete_card(id, now) {
            Some(action) => {
                if self.card_selection.as_deref() == Some(id) {
                    self.card_selection = None;
                }
                self.push(action);
                true
            }
            None => false,
        }
    }

    /// The topmost card (not ink) under `(x, y)`.
    pub fn card_at(&self, x: f64, y: f64) -> Option<&Node> {
        let mut found = None;
        for node in self.canvas.live_nodes() {
            if node.kind == canvas::INK || node.kind == "group" {
                continue;
            }
            let b = node.bounds();
            if x >= b.x && x <= b.right() && y >= b.y && y <= b.bottom() {
                found = Some(node);
            }
        }
        found
    }

    /// The viewport showing everything, `margin` points in from `size`'s edges.
    pub fn fit(&mut self, size: [f32; 2]) {
        let Some(b) = self.canvas.content_bounds() else {
            self.view = Viewport::default();
            return;
        };
        let corners = [
            [b.x as f32, b.y as f32],
            [b.right() as f32, b.bottom() as f32],
        ];
        self.view = Viewport::fit(&corners, size, 48., 2.0);
    }

    #[cfg(test)]
    /// The outline polygon of a finished stroke, kept until the stroke
    /// changes (a stroke never changes: its version is its identity).
    pub fn outline_of(&mut self, stroke: &Stroke) -> Arc<Vec<[f64; 2]>> {
        painted(&mut self.outlines, stroke).polygon.clone()
    }

    /// The finished strokes whose rectangles meet `seen` (canvas
    /// coordinates, low and high corners), bottom first. Outlines and
    /// rectangles are built once per stroke version, and nothing is copied
    /// but the outline's handle: this runs every frame, over every stroke.
    pub fn visible_ink(&mut self, seen: ([f32; 2], [f32; 2])) -> Vec<VisibleInk> {
        let CanvasState {
            canvas,
            outlines,
            selection,
            ..
        } = self;
        let (lo, hi) = (
            [f64::from(seen.0[0]), f64::from(seen.0[1])],
            [f64::from(seen.1[0]), f64::from(seen.1[1])],
        );
        let mut out = Vec::new();
        let mut live = 0;
        for node in canvas.live_nodes().filter(|n| n.kind == canvas::INK) {
            let (ox, oy) = (node.x, node.y);
            for stroke in node.strokes.iter().filter(|s| !s.deleted) {
                live += 1;
                let painted = painted(outlines, stroke);
                let Some(b) = painted.bounds else { continue };
                if b.right() + ox < lo[0]
                    || b.x + ox > hi[0]
                    || b.bottom() + oy < lo[1]
                    || b.y + oy > hi[1]
                {
                    continue;
                }
                let held = selection
                    .iter()
                    .any(|r| r.stroke == stroke.id && r.node == node.id);
                out.push(VisibleInk {
                    polygon: painted.polygon.clone(),
                    origin: (ox, oy),
                    color: stroke.color.clone(),
                    tool: stroke.tool.clone(),
                    selected: held.then_some(canvas::Bounds {
                        x: b.x + ox,
                        y: b.y + oy,
                        ..b
                    }),
                });
            }
        }
        if outlines.len() > OUTLINES_KEPT.max(2 * live) {
            // Mostly strokes since erased or moved: keep the live ones only.
            let live: std::collections::HashSet<&str> =
                canvas.live_strokes().map(|(_, s)| s.id.as_str()).collect();
            outlines.retain(|id, _| live.contains(id.as_str()));
        }
        out
    }

    /// The stroke being drawn with a pen, for a shared canvas to stream:
    /// `None` when there is none or it is still a single dot. The ink layer
    /// the points go in is made now if the canvas has none, which is a change
    /// to send before the points (`layer_made`).
    pub fn streaming(&mut self, now: i64) -> Option<(Streaming, bool)> {
        let live = self.live.as_ref()?;
        let name = live.tool.pen()?;
        if live.points.len() < 2 {
            return None;
        }
        let (id, tool, raw) = (live.id.clone(), name.to_owned(), live.points.clone());
        let target = self.canvas.stream_target(now);
        Some((
            Streaming {
                ink: target.ink,
                id,
                tool,
                color: self.color.clone(),
                width: self.width,
                z: target.z,
                points: canvas::streamed_points(&raw, target.origin),
            },
            target.layer_made,
        ))
    }

    /// The ID of the stroke being drawn, if a pen is down.
    pub fn live_id(&self) -> Option<&str> {
        self.live.as_ref().map(|live| live.id.as_str())
    }

    /// Takes the canvas a shared canvas's engine gives back, keeping the
    /// view, the tool and what is still there of the selection.
    pub fn take_canvas(&mut self, canvas: Canvas) {
        self.canvas = canvas;
        let canvas = &self.canvas;
        self.selection.retain(|r| {
            canvas
                .live_strokes()
                .any(|(node, s)| node.id == r.node && s.id == r.stroke)
        });
        if self
            .card_selection
            .as_deref()
            .is_some_and(|id| canvas.node(id).is_none())
        {
            self.card_selection = None;
        }
    }

    /// The outline of the stroke in progress, with `predicted` samples
    /// appended for this frame only. Rebuilt every frame: it is one stroke.
    pub fn live_outline(&self, predicted: &[PenSample]) -> Option<(Stroke, Vec<[f64; 2]>)> {
        let live = self.live.as_ref()?;
        let mut points = live.points.clone();
        points.extend(predicted.iter().map(|s| s.point()));
        let stroke = Stroke {
            tool: live.tool.pen()?.into(),
            color: self.color.clone(),
            width: self.width,
            points,
            ..Stroke::default()
        };
        let polygon = pen::outline(&stroke, false);
        Some((stroke, polygon))
    }
}

/// `stroke`'s outline and rectangle from `cache`, built when it has none for
/// the stroke's version.
fn painted<'a>(cache: &'a mut HashMap<String, Painted>, stroke: &Stroke) -> &'a Painted {
    let fresh = cache
        .get(stroke.id.as_str())
        .is_none_or(|p| p.version != stroke.kanso_version || p.points != stroke.points.len());
    if fresh {
        cache.insert(
            stroke.id.clone(),
            Painted {
                version: stroke.kanso_version,
                points: stroke.points.len(),
                bounds: stroke.bounds(),
                polygon: Arc::new(pen::outline(stroke, true)),
            },
        );
    }
    &cache[stroke.id.as_str()]
}
