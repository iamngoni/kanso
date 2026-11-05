//! The canvas document's data: JSON Canvas cards and arrows plus Kanso's own
//! `ink` nodes and strokes. Only shapes live here; the rules are in the
//! sibling modules.
//!
//! Every node, edge and stroke carries `kansoVersion` (a counter),
//! `kansoUpdated` (ms) and `kansoZ` (stacking order). They exist for the
//! private-sync merge and for tombstones, not for display. A deleted item is
//! kept as a tombstone (`deleted: true`) so a merge on another device cannot
//! bring it back.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// The pens a stroke can be drawn with.
pub const TOOLS: &[&str] = &["pen", "pencil", "marker", "highlighter"];

/// The node type Kanso adds to JSON Canvas.
pub const INK: &str = "ink";

/// A `canvas` fence in an ordinary note: it names a canvas note instead of
/// holding one, and shows it (or one group of it) in place. Written and read
/// by [`super::embed`].
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Embed {
    /// The canvas note's ID, `note:…`.
    pub note: String,
    /// A `group` node of that canvas to show instead of all its content.
    pub group: Option<String>,
    /// The width to show it at, in points.
    pub width: Option<u32>,
}

/// A JSON Canvas 1.0 document with Kanso's ink extension.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Canvas {
    #[serde(default)]
    pub nodes: Vec<Node>,
    #[serde(default)]
    pub edges: Vec<Edge>,
    /// Set on an inline sketch: its fixed box.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kanso_sketch: Option<Sketch>,
    /// An inline sketch's stable identity inside its note.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kanso_sketch_id: Option<String>,
    /// Recognized handwriting, per ink node.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub kanso_ink_ocr: Vec<InkOcr>,
    /// Top-level fields JSON Canvas doesn't define.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// An inline sketch's drawing area, in points.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sketch {
    pub w: u32,
    pub h: u32,
    pub v: u32,
}

/// Handwriting read from one ink node.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InkOcr {
    pub node: String,
    /// A fingerprint of the strokes that were read.
    pub md5: String,
    pub engine: String,
    pub text: String,
}

/// A card, a group or a layer of ink on the canvas.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Node {
    pub id: String,
    /// `text`, `file`, `link`, `group` or `ink`; another app's type is kept.
    #[serde(rename = "type", default, skip_serializing_if = "String::is_empty")]
    pub kind: String,
    #[serde(default)]
    pub x: f64,
    #[serde(default)]
    pub y: f64,
    #[serde(default)]
    pub width: f64,
    #[serde(default)]
    pub height: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// A text card's Markdown.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// A file card's path, or a note card's Kanso ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subpath: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// A group's label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_style: Option<String>,
    /// A note card's title beside its ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// An ink node's strokes, points relative to the node's `x`/`y`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub strokes: Vec<Stroke>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub deleted: bool,
    #[serde(default, skip_serializing_if = "is_zero_u64")]
    pub kanso_version: u64,
    #[serde(default, skip_serializing_if = "is_zero_i64")]
    pub kanso_updated: i64,
    #[serde(default, skip_serializing_if = "is_zero_f64")]
    pub kanso_z: f64,
    /// Fields JSON Canvas doesn't define.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// An arrow between two nodes.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Edge {
    pub id: String,
    #[serde(default)]
    pub from_node: String,
    #[serde(default)]
    pub to_node: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_side: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_end: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_side: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_end: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub deleted: bool,
    #[serde(default, skip_serializing_if = "is_zero_u64")]
    pub kanso_version: u64,
    #[serde(default, skip_serializing_if = "is_zero_i64")]
    pub kanso_updated: i64,
    #[serde(default, skip_serializing_if = "is_zero_f64")]
    pub kanso_z: f64,
    /// Fields JSON Canvas doesn't define.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// One pen stroke. Its points are appended while it is drawn and never
/// changed after it ends: erasing tombstones it and writes new strokes.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stroke {
    pub id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub tool: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub color: String,
    /// In points.
    #[serde(default)]
    pub width: f64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub points: Vec<Point>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub deleted: bool,
    #[serde(default, skip_serializing_if = "is_zero_u64")]
    pub kanso_version: u64,
    #[serde(default, skip_serializing_if = "is_zero_i64")]
    pub kanso_updated: i64,
    #[serde(default, skip_serializing_if = "is_zero_f64")]
    pub kanso_z: f64,
    /// Fields Kanso doesn't define.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// A pen sample, stored as `[x, y, pressure]` or, for a pencil,
/// `[x, y, pressure, azimuth, altitude]`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "Vec<f64>", into = "Vec<f64>")]
pub struct Point {
    pub x: f64,
    pub y: f64,
    /// 0 to 1.
    pub pressure: f64,
    /// Azimuth and altitude, when the pen reports a tilt.
    pub tilt: Option<(f64, f64)>,
}

impl TryFrom<Vec<f64>> for Point {
    type Error = String;

    fn try_from(v: Vec<f64>) -> Result<Self, String> {
        match v.as_slice() {
            [x, y] => Ok(Point::new(*x, *y, 0.5)),
            [x, y, p] => Ok(Point::new(*x, *y, *p)),
            [x, y, p, az, alt] => Ok(Point {
                tilt: Some((*az, *alt)),
                ..Point::new(*x, *y, *p)
            }),
            _ => Err("A point has 2, 3 or 5 numbers.".into()),
        }
    }
}

impl From<Point> for Vec<f64> {
    fn from(p: Point) -> Vec<f64> {
        let mut out = vec![p.x, p.y, p.pressure];
        if let Some((az, alt)) = p.tilt {
            out.extend([az, alt]);
        }
        out
    }
}

impl Point {
    /// A point without tilt.
    pub fn new(x: f64, y: f64, pressure: f64) -> Point {
        Point {
            x,
            y,
            pressure,
            tilt: None,
        }
    }
}

fn is_false(v: &bool) -> bool {
    !*v
}
fn is_zero_u64(v: &u64) -> bool {
    *v == 0
}
fn is_zero_i64(v: &i64) -> bool {
    *v == 0
}
fn is_zero_f64(v: &f64) -> bool {
    *v == 0.0
}
