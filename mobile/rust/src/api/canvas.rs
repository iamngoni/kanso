//! A canvas open for drawing: pen samples in, ready-to-paint outlines out.
//! The document lives in Rust; Dart paints the view each call returns and
//! passes the IDs of strokes it already holds (`known`) so only new strokes
//! carry an outline.

use flutter_rust_bridge::frb;
use kanso_mobile::{CanvasSession, KansoError, Saved, ink_preview};

use super::canvas_types::{
    CanvasPoint, CanvasPointerRow, CanvasPulse, CanvasRect, CanvasSaved, CanvasView, InkStyle,
    PenSample, StrokeKey,
};

/// The outline of a stroke still being drawn, to paint under the pen each
/// frame. Runs on the UI isolate in microseconds; stores nothing.
#[frb(sync)]
pub fn canvas_ink_preview(style: InkStyle, samples: Vec<PenSample>) -> Vec<f64> {
    ink_preview(style, samples)
}

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

    pub fn erase_strokes_along(
        &self,
        path: Vec<CanvasPoint>,
        radius: f64,
        known: Vec<String>,
    ) -> Result<CanvasView, KansoError> {
        self.0.erase_strokes_along(path, radius, known)
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

    /// A shared canvas's tick: hands the room what was drawn, takes what the
    /// others drew, and says where their pointers are. Call it when no
    /// gesture is in hand.
    pub fn pulse(&self, known: Vec<String>) -> Result<CanvasPulse, KansoError> {
        self.0.pulse(known)
    }

    /// Streams the stroke being drawn to the others. `true` when someone
    /// erased it meanwhile: the pen's stroke stops there.
    pub fn stream_stroke(
        &self,
        style: InkStyle,
        samples: Vec<PenSample>,
    ) -> Result<bool, KansoError> {
        self.0.stream_stroke(style, samples)
    }

    /// The pen won't finish the stroke it streamed: the others drop it.
    pub fn cancel_stroke(&self) -> Result<(), KansoError> {
        self.0.cancel_stroke()
    }

    /// This pointer's place on the canvas, for the dot others see.
    pub fn set_pointer(&self, x: f64, y: f64, tool: String) {
        self.0.set_pointer(x, y, tool);
    }

    /// The pointer left the canvas.
    pub fn clear_pointer(&self) {
        self.0.clear_pointer();
    }

    /// Other people's pointers on the canvas now.
    pub fn pointers(&self) -> Vec<CanvasPointerRow> {
        self.0.pointers()
    }

    /// The app is back in front: joins the room again.
    pub fn reconnect(&self) -> Result<(), KansoError> {
        self.0.reconnect()
    }

    /// The screen closed: writes what was drawn and leaves the room.
    pub fn close(&self) -> Result<(), KansoError> {
        self.0.close()
    }
}
