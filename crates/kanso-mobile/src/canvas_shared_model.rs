//! What a shared canvas adds to the data the shell draws (#45 step 7): the
//! other people's pointers, and what one tick of the live room found.

use crate::canvas_model::CanvasView;

/// Someone else's pointer on the canvas: a dot and a name in the shell, in
/// the person's colour. The colour comes as both hexes (`#rrggbb`) the app
/// uses for shared notes' carets, so the dot reads on a light page and a
/// dark one.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CanvasPointerRow {
    pub account_id: String,
    pub name: String,
    /// In canvas coordinates.
    pub x: f64,
    pub y: f64,
    /// The tool they hold: `pen`, `eraser`, `lasso`, ...
    pub tool: String,
    /// Which of the app's eight cursor colours, from 0.
    pub colour: u32,
    pub colour_light: String,
    pub colour_dark: String,
}

/// What one tick of a shared canvas found: the canvas when it changed
/// (others drew, a role changed, sharing ended), and where the others'
/// pointers are now.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CanvasPulse {
    pub view: Option<CanvasView>,
    pub pointers: Vec<CanvasPointerRow>,
}
