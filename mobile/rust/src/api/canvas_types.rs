//! The canvas data types, declared again for the generator, as `types.rs`
//! does for the rest. What the fields mean is said where the facade declares
//! them (`crates/kanso-mobile/src/canvas_model.rs`).

use flutter_rust_bridge::frb;
// Public: the generated code names these types by this module's path.
pub use kanso_mobile::{
    CanvasArrow, CanvasBudget, CanvasCard, CanvasGroup, CanvasPoint, CanvasRect, CanvasSaved,
    CanvasStroke, CanvasView, InkStyle, PenSample, SketchInfo, SketchInserted, StrokeKey,
};

#[frb(mirror(CanvasPoint))]
pub struct _CanvasPoint {
    pub x: f64,
    pub y: f64,
}

#[frb(mirror(CanvasRect))]
pub struct _CanvasRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[frb(mirror(PenSample))]
pub struct _PenSample {
    pub x: f64,
    pub y: f64,
    pub pressure: f64,
    pub azimuth: Option<f64>,
    pub altitude: Option<f64>,
}

#[frb(mirror(InkStyle))]
pub struct _InkStyle {
    pub tool: String,
    pub color: String,
    pub width: f64,
}

#[frb(mirror(StrokeKey))]
pub struct _StrokeKey {
    pub node_id: String,
    pub stroke_id: String,
}

#[frb(mirror(CanvasBudget))]
pub enum _CanvasBudget {
    Ok,
    Warn,
    Full,
}

#[frb(mirror(CanvasStroke))]
pub struct _CanvasStroke {
    pub id: String,
    pub node_id: String,
    pub tool: String,
    pub color: String,
    pub opacity: f64,
    pub width: f64,
    pub bounds: CanvasRect,
    pub outline: Vec<f64>,
    pub known: bool,
}

#[frb(mirror(CanvasCard))]
pub struct _CanvasCard {
    pub id: String,
    pub kind: String,
    pub rect: CanvasRect,
    pub text: String,
    pub color: Option<String>,
    pub note_id: Option<String>,
    pub missing: bool,
}

#[frb(mirror(CanvasGroup))]
pub struct _CanvasGroup {
    pub id: String,
    pub rect: CanvasRect,
    pub label: String,
    pub color: Option<String>,
}

#[frb(mirror(CanvasArrow))]
pub struct _CanvasArrow {
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

#[frb(mirror(CanvasView))]
pub struct _CanvasView {
    pub note_id: String,
    pub rev: String,
    pub title: String,
    pub strokes: Vec<CanvasStroke>,
    pub cards: Vec<CanvasCard>,
    pub groups: Vec<CanvasGroup>,
    pub arrows: Vec<CanvasArrow>,
    pub bounds: Option<CanvasRect>,
    pub bytes: u32,
    pub budget: CanvasBudget,
    pub can_undo: bool,
    pub can_redo: bool,
    pub dirty: bool,
}

#[frb(mirror(CanvasSaved))]
pub struct _CanvasSaved {
    pub rev: String,
    pub bytes: u32,
    pub merged: bool,
}

#[frb(mirror(SketchInfo))]
pub struct _SketchInfo {
    pub id: String,
    pub width: u32,
    pub height: u32,
    pub read_only: bool,
}

#[frb(mirror(SketchInserted))]
pub struct _SketchInserted {
    pub body: String,
    pub id: String,
    pub cursor: u32,
}
