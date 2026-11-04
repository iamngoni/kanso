//! A canvas open for drawing: pen samples in, ready-to-paint outlines out.
//! The document lives in Rust; Dart paints the view each call returns and
//! passes the IDs of strokes it already holds (`known`) so only new strokes
//! carry an outline.

use flutter_rust_bridge::frb;
use kanso_mobile::{CanvasSession, KansoError, Saved};

use super::canvas_types::{
    CanvasPoint, CanvasRect, CanvasSaved, CanvasView, InkStyle, PenSample, StrokeKey,
};

/// A canvas open for drawing. Cheap to clone on the Rust side; Dart holds one.
#[frb(opaque)]
pub struct KansoCanvas(CanvasSession);

impl KansoCanvas {
    pub(super) fn new(session: CanvasSession) -> KansoCanvas {
        KansoCanvas(session)
    }

    pub fn view(&self, known: Vec<String>) -> Result<CanvasView, KansoError> {
        self.0.view(known)
    }

    pub fn draw(
        &self,
        style: InkStyle,
        samples: Vec<PenSample>,
        known: Vec<String>,
    ) -> Result<CanvasView, KansoError> {
        self.0.draw(style, samples, known)
    }

    pub fn erase_at(
        &self,
        x: f64,
        y: f64,
        radius: f64,
        known: Vec<String>,
    ) -> Result<CanvasView, KansoError> {
        self.0.erase_at(x, y, radius, known)
    }

    pub fn erase_along(
        &self,
        path: Vec<CanvasPoint>,
        radius: f64,
        known: Vec<String>,
    ) -> Result<CanvasView, KansoError> {
        self.0.erase_along(path, radius, known)
    }

    pub fn select(&self, polygon: Vec<CanvasPoint>) -> Result<Vec<StrokeKey>, KansoError> {
        self.0.select(polygon)
    }

    pub fn delete_strokes(
        &self,
        keys: Vec<StrokeKey>,
        known: Vec<String>,
    ) -> Result<CanvasView, KansoError> {
        self.0.delete_strokes(keys, known)
    }

    pub fn move_strokes(
        &self,
        keys: Vec<StrokeKey>,
        dx: f64,
        dy: f64,
        known: Vec<String>,
    ) -> Result<CanvasView, KansoError> {
        self.0.move_strokes(keys, dx, dy, known)
    }

    pub fn add_text_card(
        &self,
        x: f64,
        y: f64,
        text: String,
        known: Vec<String>,
    ) -> Result<CanvasView, KansoError> {
        self.0.add_text_card(x, y, text, known)
    }

    pub fn add_note_card(
        &self,
        note_id: String,
        x: f64,
        y: f64,
        known: Vec<String>,
    ) -> Result<CanvasView, KansoError> {
        self.0.add_note_card(note_id, x, y, known)
    }

    pub fn set_card_text(
        &self,
        id: String,
        text: String,
        known: Vec<String>,
    ) -> Result<CanvasView, KansoError> {
        self.0.set_card_text(id, text, known)
    }

    pub fn set_card_rect(
        &self,
        id: String,
        rect: CanvasRect,
        known: Vec<String>,
    ) -> Result<CanvasView, KansoError> {
        self.0.set_card_rect(id, rect, known)
    }

    pub fn delete_card(&self, id: String, known: Vec<String>) -> Result<CanvasView, KansoError> {
        self.0.delete_card(id, known)
    }

    pub fn add_arrow(
        &self,
        from: String,
        to: String,
        known: Vec<String>,
    ) -> Result<CanvasView, KansoError> {
        self.0.add_arrow(from, to, known)
    }

    pub fn undo(&self, known: Vec<String>) -> Result<CanvasView, KansoError> {
        self.0.undo(known)
    }

    pub fn redo(&self, known: Vec<String>) -> Result<CanvasView, KansoError> {
        self.0.redo(known)
    }

    /// Writes the canvas if it changed. Called when a stroke ends and after
    /// two seconds without input.
    pub fn save(&self) -> Result<CanvasSaved, KansoError> {
        self.0.save()
    }

    pub fn move_to_new_canvas(
        &self,
        keys: Vec<StrokeKey>,
        title: String,
        known: Vec<String>,
    ) -> Result<(Saved, CanvasView), KansoError> {
        self.0.move_to_new_canvas(keys, title, known)
    }

    pub fn export_svg(&self, dark: bool) -> Result<String, KansoError> {
        self.0.export_svg(dark)
    }

    pub fn export_excalidraw(&self) -> Result<String, KansoError> {
        self.0.export_excalidraw()
    }
}
