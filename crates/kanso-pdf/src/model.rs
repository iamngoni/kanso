//! The annotation block's data: one block per PDF in a note's body.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// The info string of the fence that holds the block.
pub const BLOCK_FENCE: &str = "kanso-pdf";
/// The block's text is capped well under the body limit (1,048,576).
pub const MAX_BLOCK_CHARS: usize = 800_000;
/// Quads one highlight may carry.
pub const MAX_QUADS: usize = 32;
/// Characters in one comment.
pub const MAX_COMMENT_CHARS: usize = 10_000;
/// Characters of quoted text in one highlight.
pub const MAX_QUOTE_CHARS: usize = 1_000;
/// The block format this build writes and reads.
pub const BLOCK_VERSION: u32 = 1;
/// Deleted highlights' ids a block remembers, so a sync merge never brings
/// one back. Far more than the block's ~2,000 highlights ever leave.
pub const MAX_REMOVED: usize = 4_096;

/// A box on a page, `[x0, y0, x1, y1]`, normalised to the displayed page box
/// with the origin at the top-left. Values are kept to four decimals.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Quad(pub [f64; 4]);

fn unit(v: f64) -> f64 {
    let v = if v.is_finite() { v } else { 0.0 };
    (v.clamp(0.0, 1.0) * 10_000.0).round() / 10_000.0
}

impl Quad {
    /// A quad from any two corners: clipped to the page, ordered, rounded.
    pub fn new(x0: f64, y0: f64, x1: f64, y1: f64) -> Self {
        let (a, b, c, d) = (unit(x0), unit(y0), unit(x1), unit(y1));
        Quad([a.min(c), b.min(d), a.max(c), b.max(d)])
    }

    /// From PDFium's bottom-left points on a page `width` x `height` points.
    pub fn from_pdf_points(
        left: f64,
        bottom: f64,
        right: f64,
        top: f64,
        width: f64,
        height: f64,
    ) -> Self {
        if width <= 0.0 || height <= 0.0 {
            return Quad([0.0; 4]);
        }
        Quad::new(
            left / width,
            1.0 - top / height,
            right / width,
            1.0 - bottom / height,
        )
    }

    /// Back to `(left, bottom, right, top)` in PDF points.
    pub fn to_pdf_points(&self, width: f64, height: f64) -> (f64, f64, f64, f64) {
        let [x0, y0, x1, y1] = self.0;
        (
            x0 * width,
            (1.0 - y1) * height,
            x1 * width,
            (1.0 - y0) * height,
        )
    }

    /// Whether the quad is ordered and inside the page.
    pub fn is_valid(&self) -> bool {
        let [x0, y0, x1, y1] = self.0;
        self.0
            .iter()
            .all(|v| v.is_finite() && (0.0..=1.0).contains(v))
            && x0 <= x1
            && y0 <= y1
    }
}

/// A highlight's colour.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Color {
    #[default]
    Yellow,
    Green,
    Blue,
    Pink,
}

impl Color {
    pub const ALL: [Color; 4] = [Color::Yellow, Color::Green, Color::Blue, Color::Pink];
}

/// One highlight, possibly across pages.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Highlight {
    pub id: String,
    /// The first page, 1-based.
    pub page: u32,
    /// Quads by 1-based page.
    pub quads: BTreeMap<u32, Vec<Quad>>,
    pub text: String,
    #[serde(default)]
    pub color: Color,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(default)]
    pub created_at: i64,
    /// How many times its colour or comment was changed. Of two versions of
    /// the same highlight (two devices, two editors), the one changed more
    /// often wins a merge. Not written while 0.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub rev: u32,
    /// Fields a newer Kanso added; kept so a save loses nothing.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// The block for one PDF.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfBlock {
    /// The attachment's document id, `file:abc123`.
    pub file: String,
    /// The attachment's `md5digest` when the annotations were made.
    #[serde(default)]
    pub md5: String,
    pub version: u32,
    #[serde(default)]
    pub highlights: Vec<Highlight>,
    /// Ink strokes by page, in the shared ink format (step 10 defines them).
    #[serde(default)]
    pub ink: Vec<Value>,
    /// The companion ink attachment, when the strokes outgrew the block.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ink_file: Option<String>,
    /// The ids of deleted highlights, sorted: a merge with a version that
    /// still has one leaves it deleted. Not written while empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub removed: Vec<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

fn is_zero(n: &u32) -> bool {
    *n == 0
}

impl PdfBlock {
    pub fn new(file: impl Into<String>, md5: impl Into<String>) -> Self {
        PdfBlock {
            file: file.into(),
            md5: md5.into(),
            version: BLOCK_VERSION,
            highlights: Vec::new(),
            ink: Vec::new(),
            ink_file: None,
            removed: Vec::new(),
            extra: Map::new(),
        }
    }

    /// "4 highlights · 1 comment", the preview's quiet line; `None` when empty.
    pub fn summary(&self) -> Option<String> {
        let h = self.highlights.len();
        let c = self
            .highlights
            .iter()
            .filter(|h| h.comment.is_some())
            .count();
        if h == 0 {
            return None;
        }
        let mut s = format!("{h} highlight{}", if h == 1 { "" } else { "s" });
        if c > 0 {
            s.push_str(&format!(" · {c} comment{}", if c == 1 { "" } else { "s" }));
        }
        Some(s)
    }

    /// Whether the attachment changed since the annotations were made.
    pub fn pdf_changed(&self, current_md5: &str) -> bool {
        !self.md5.is_empty() && !current_md5.is_empty() && self.md5 != current_md5
    }
}
