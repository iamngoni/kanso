//! Canonical sketch document format.
//!
//! This is the truth for a sketch. It is CBOR-encoded (compact, self-describing,
//! schema-evolvable) and carries `format_version` from day one so the format can
//! evolve without breaking old sketches. Native input is normalized into this
//! model on save; export (SVG/PNG) is derived from it.

use serde::{Deserialize, Serialize};

/// Current on-disk format version. Bump on any breaking schema change and add a
/// migration path in the engine.
pub const FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SketchDoc {
    pub format_version: u32,
    pub background: Background,
    pub elements: Vec<Element>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Background {
    Blank,
    Dotted,
    Grid,
    Lined,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Element {
    Stroke(Stroke),
    Shape(Shape),
    Arrow(Arrow),
    Text(TextLabel),
}

/// A freehand stroke: a pressure/tilt/time-stamped point list plus styling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stroke {
    pub points: Vec<Point>,
    pub color: Rgba,
    pub base_width: f32,
    pub tool: Tool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    #[serde(default = "default_pressure")]
    pub pressure: f32,
    #[serde(default)]
    pub tilt: f32,
    /// Milliseconds since stroke start — kept for replay and velocity-based width.
    #[serde(default)]
    pub t: f32,
}

fn default_pressure() -> f32 {
    1.0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tool {
    Pen,
    Pencil,
    Marker,
    Highlighter,
    Eraser,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shape {
    pub shape: ShapeKind,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub color: Rgba,
    pub stroke_width: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShapeKind {
    Rect,
    Ellipse,
    Line,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Arrow {
    pub from: (f32, f32),
    pub to: (f32, f32),
    pub color: Rgba,
    pub stroke_width: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextLabel {
    pub x: f32,
    pub y: f32,
    pub content: String,
    pub size: f32,
    pub color: Rgba,
}

impl SketchDoc {
    pub fn new() -> Self {
        Self {
            format_version: FORMAT_VERSION,
            background: Background::Blank,
            elements: Vec::new(),
        }
    }

    /// Encode to the canonical CBOR blob stored in `sketches.data_blob`.
    pub fn to_cbor(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        ciborium::into_writer(self, &mut buf)
            .expect("sketch CBOR encoding is infallible into a Vec");
        buf
    }

    /// Decode from a stored CBOR blob.
    pub fn from_cbor(bytes: &[u8]) -> Result<Self, ciborium::de::Error<std::io::Error>> {
        ciborium::from_reader(bytes)
    }
}

impl Default for SketchDoc {
    fn default() -> Self {
        Self::new()
    }
}
