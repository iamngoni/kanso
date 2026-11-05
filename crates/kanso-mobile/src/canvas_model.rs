//! The canvas data the shell draws and sends back (#45). Plain data: what a
//! canvas looks like to Flutter is ready to paint (ink as filled outlines,
//! arrows as curves, cards as rectangles), so the shell decides nothing about
//! the document.

/// A point on the canvas, in points.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CanvasPoint {
    pub x: f64,
    pub y: f64,
}

/// A rectangle on the canvas.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CanvasRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// One pen sample. `pressure` is 0 to 1 (0.5 for a finger or mouse, which
/// draws its width from speed); `azimuth` and `altitude` are the pen's tilt
/// in radians when it reports one.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PenSample {
    pub x: f64,
    pub y: f64,
    pub pressure: f64,
    pub azimuth: Option<f64>,
    pub altitude: Option<f64>,
}

/// How the pen draws: `pen`, `pencil`, `marker` or `highlighter`, a `#rrggbb`
/// colour and a width in points. Each stroke keeps its own.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct InkStyle {
    pub tool: String,
    pub color: String,
    pub width: f64,
}

/// A stroke named by its ink node and its own ID.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct StrokeKey {
    pub node_id: String,
    pub stroke_id: String,
}

/// Where the canvas stands against its 1 MB budget.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CanvasBudget {
    #[default]
    Ok,
    /// 700 KB or more: the shell warns and offers to move ink to a new
    /// canvas.
    Warn,
    /// 1 MB or more: new ink is refused.
    Full,
}

/// A stroke, ready to fill.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CanvasStroke {
    pub id: String,
    pub node_id: String,
    pub tool: String,
    /// `#rrggbb` as drawn; the shell turns a near-black or near-white pen to
    /// the page's text colour, as the SVG does.
    pub color: String,
    pub opacity: f64,
    pub width: f64,
    pub bounds: CanvasRect,
    /// The outline polygon in canvas coordinates, as `x0, y0, x1, y1, ...`.
    /// Empty when `known` (the shell already holds this stroke's outline).
    pub outline: Vec<f64>,
    pub known: bool,
}

/// A card: text, a note, a link or a file.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CanvasCard {
    pub id: String,
    /// `text`, `note`, `link` or `file`.
    pub kind: String,
    pub rect: CanvasRect,
    /// What the card says: a text card's text, a note card's live title.
    pub text: String,
    pub color: Option<String>,
    /// The note a note card points at.
    pub note_id: Option<String>,
    /// A note card whose note is gone.
    pub missing: bool,
}

/// A group's frame.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CanvasGroup {
    pub id: String,
    pub rect: CanvasRect,
    pub label: String,
    pub color: Option<String>,
}

/// An arrow as a cubic curve.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CanvasArrow {
    pub id: String,
    pub from: CanvasPoint,
    pub c1: CanvasPoint,
    pub c2: CanvasPoint,
    pub to: CanvasPoint,
    pub from_arrow: bool,
    pub to_arrow: bool,
    pub color: Option<String>,
    pub label: String,
}

/// An open canvas, as the shell paints it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CanvasView {
    pub note_id: String,
    pub rev: String,
    pub title: String,
    /// In stacking order, bottom first.
    pub strokes: Vec<CanvasStroke>,
    pub cards: Vec<CanvasCard>,
    pub groups: Vec<CanvasGroup>,
    pub arrows: Vec<CanvasArrow>,
    /// What the content covers; `None` for an empty canvas.
    pub bounds: Option<CanvasRect>,
    /// The size of the saved document, in bytes.
    pub bytes: u32,
    pub budget: CanvasBudget,
    pub can_undo: bool,
    pub can_redo: bool,
    /// Edits not saved yet.
    pub dirty: bool,
    /// The canvas is shared: the document is the room's, and others draw on
    /// it with this one.
    pub shared: bool,
    /// Nothing can be drawn: a canvas shared to look at, or one shared
    /// while nobody is signed in.
    pub read_only: bool,
}

/// What a save did.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CanvasSaved {
    pub rev: String,
    pub bytes: u32,
    /// Whether the note had changed elsewhere meanwhile and the two were
    /// merged (strokes from both are kept).
    pub merged: bool,
}

/// An inline sketch a session is open on: its ID in the note, its box in
/// points, and whether it can't be drawn on (the note is shared or in the
/// trash: "Copy as canvas" is what is offered).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SketchInfo {
    pub id: String,
    pub width: u32,
    pub height: u32,
    pub read_only: bool,
}

/// A note's text with an empty sketch put in it: `body`, the new sketch's
/// `id`, and where the cursor goes (after it, in UTF-16 units).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SketchInserted {
    pub body: String,
    pub id: String,
    pub cursor: u32,
}
