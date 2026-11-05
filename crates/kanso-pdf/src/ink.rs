//! Pencil ink on a PDF's pages (issue #43, step 10).
//!
//! Ink is the canvas's ink, not a second format: a page's strokes are the
//! canvas's [`Stroke`]s (points with pressure and optional tilt, a pen, a
//! colour, a width, and the version stamps the private-sync merge reads), so
//! the partial eraser, the stroke eraser, the outliner and the merge are the
//! canvas's own. A page's ink is read as a one-layer canvas
//! ([`PageInk::to_canvas`]) and the canvas's edits are written back with
//! [`PageInk::from_canvas`].
//!
//! Coordinates are in **page units**: the page is [`PAGE_UNITS`] across and
//! as tall as its shape makes it, so a stroke is the same at every zoom and
//! DPI and needs no page size to be read. A pen width is in the same units.
//!
//! Small amounts of ink live in the annotation block. Past
//! [`INLINE_INK_BYTES`] the strokes move to a companion attachment, an
//! [`InkFile`], that the block names with `inkFile`, so the note's body and
//! its revisions stay small. The PDF itself is untouched either way.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use kanso_core::canvas::{self, Canvas, INK, MAX_STROKE_POINTS, Node, Stroke, TOOLS};

use crate::PdfError;

/// How many units a page is wide.
pub const PAGE_UNITS: f64 = 1_000.0;
/// The tallest a page may be, in units (a page twenty times as tall as wide).
pub const MAX_PAGE_HEIGHT: f64 = PAGE_UNITS * 20.0;
/// Ink inside the annotation block stops here; more goes to the companion
/// attachment. About what a page of handwriting costs.
pub const INLINE_INK_BYTES: usize = 100_000;
/// The most ink a PDF may hold, as the companion attachment is written.
/// Well under the attachment limit (10 MB), as the canvas is under its own.
pub const MAX_INK_BYTES: usize = 8_000_000;
/// The companion attachment's content type.
pub const INK_FILE_TYPE: &str = kanso_core::model::INK_DOCUMENT_TYPE;
/// The companion attachment's format version.
pub const INK_FILE_VERSION: u32 = 1;
/// The id of the one ink layer a page's canvas has.
pub const LAYER: &str = "ink";
/// The pen widths offered, in page units.
pub const WIDTHS: [f64; 4] = [2.0, 4.0, 7.0, 12.0];
/// The eraser disc's radius on the screen, in logical pixels.
pub const ERASER_PX: f64 = 10.0;
/// The most pages a block holds ink for.
pub const MAX_INK_PAGES: usize = 5_000;
/// The longest a stroke id may be.
const MAX_STROKE_ID: usize = 40;
/// The widest a pen may be, in units.
const MAX_WIDTH: f64 = 200.0;

/// One page's ink: its strokes, tombstones included (a deleted stroke stays
/// so a merge with a device that still has it cannot bring it back).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PageInk {
    /// The page, 1-based.
    pub page: u32,
    #[serde(default)]
    pub strokes: Vec<Stroke>,
    /// Fields a newer Kanso added; kept so a save loses nothing.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl PageInk {
    pub fn new(page: u32) -> PageInk {
        PageInk {
            page,
            ..PageInk::default()
        }
    }

    /// The strokes that are drawn.
    pub fn live(&self) -> impl Iterator<Item = &Stroke> {
        self.strokes.iter().filter(|s| !s.deleted)
    }

    /// How many strokes are drawn.
    pub fn live_count(&self) -> usize {
        self.live().count()
    }

    /// Whether nothing is drawn (tombstones may remain).
    pub fn is_blank(&self) -> bool {
        self.live_count() == 0
    }

    /// The page as a one-layer canvas, for the canvas's edits to work on.
    pub fn to_canvas(&self) -> Canvas {
        Canvas {
            nodes: vec![Node {
                id: LAYER.into(),
                kind: INK.into(),
                width: PAGE_UNITS,
                height: PAGE_UNITS,
                strokes: self.strokes.clone(),
                ..Node::default()
            }],
            ..Canvas::default()
        }
    }

    /// The strokes of a canvas that worked on a page (every ink layer's, in
    /// stacking order, with the layer's origin applied), keeping `self`'s
    /// extras.
    pub fn from_canvas(&self, canvas: &Canvas) -> PageInk {
        let mut strokes: Vec<Stroke> = Vec::new();
        for node in canvas.nodes.iter().filter(|n| n.kind == INK) {
            for stroke in &node.strokes {
                let mut stroke = stroke.clone();
                if node.x != 0.0 || node.y != 0.0 {
                    for p in &mut stroke.points {
                        p.x += node.x;
                        p.y += node.y;
                    }
                }
                strokes.push(stroke);
            }
        }
        strokes.sort_by(|p, q| {
            p.kanso_z
                .total_cmp(&q.kanso_z)
                .then_with(|| p.id.cmp(&q.id))
        });
        PageInk {
            page: self.page,
            strokes,
            extra: self.extra.clone(),
        }
    }

    /// Checks the page's ink against the block's rules.
    pub fn validate(&self) -> Result<(), PdfError> {
        let bad = |m: &str| Err(PdfError::Invalid(m.to_string()));
        if self.page == 0 || self.page > crate::annotation::MAX_PAGE {
            return bad("Ink names a page that does not exist.");
        }
        let mut seen = std::collections::HashSet::new();
        for s in &self.strokes {
            if s.id.is_empty() || s.id.len() > MAX_STROKE_ID || !seen.insert(s.id.as_str()) {
                return bad("A pencil stroke has no usable id.");
            }
            if !s.tool.is_empty() && !TOOLS.contains(&s.tool.as_str()) {
                return bad("A pencil stroke uses a pen Kanso does not have.");
            }
            if !s.width.is_finite() || s.width < 0.0 || s.width > MAX_WIDTH {
                return bad("A pencil stroke has a width that makes no sense.");
            }
            if s.points.len() > MAX_STROKE_POINTS {
                return bad("A pencil stroke is too long.");
            }
            let ok = |v: f64, lo: f64, hi: f64| v.is_finite() && (lo..=hi).contains(&v);
            if s.points.iter().any(|p| {
                !ok(p.x, -PAGE_UNITS, 2.0 * PAGE_UNITS)
                    || !ok(p.y, -PAGE_UNITS, MAX_PAGE_HEIGHT + PAGE_UNITS)
                    || !p.pressure.is_finite()
            }) {
                return bad("A pencil stroke leaves its page.");
            }
        }
        Ok(())
    }
}

/// Where a point on a page of `width` x `height` (any unit) is, in page
/// units, clamped into the page.
pub fn to_units(x: f64, y: f64, width: f64, height: f64) -> (f64, f64) {
    if !(x.is_finite() && y.is_finite() && width.is_finite() && height.is_finite())
        || width <= 0.0
        || height <= 0.0
    {
        return (0.0, 0.0);
    }
    let h = (height / width * PAGE_UNITS).clamp(0.0, MAX_PAGE_HEIGHT);
    (
        (x / width * PAGE_UNITS).clamp(0.0, PAGE_UNITS),
        (y / width * PAGE_UNITS).clamp(0.0, h),
    )
}

/// The size of a page's drawing area in units: [`PAGE_UNITS`] across and as
/// tall as the page's shape.
pub fn page_extent(width: f64, height: f64) -> (f64, f64) {
    if !(width.is_finite() && height.is_finite()) || width <= 0.0 || height <= 0.0 {
        return (PAGE_UNITS, PAGE_UNITS);
    }
    (
        PAGE_UNITS,
        (height / width * PAGE_UNITS).clamp(1.0, MAX_PAGE_HEIGHT),
    )
}

/// The union of two versions of a page's ink: the canvas's own merge, so a
/// stroke drawn on either device is kept and a stroke erased on either
/// stays erased. Commutative, associative and idempotent.
pub fn merge_page(a: &PageInk, b: &PageInk) -> PageInk {
    let merged = canvas::merge(&a.to_canvas(), &b.to_canvas());
    let mut out = a.from_canvas(&merged);
    out.extra = crate::merge::merge_maps(&a.extra, &b.extra);
    out
}

/// The union of two lists of pages' ink, by page, in page order.
pub fn merge_pages(a: &[PageInk], b: &[PageInk]) -> Vec<PageInk> {
    let mut by_page: BTreeMap<u32, PageInk> = BTreeMap::new();
    for p in a.iter().chain(b) {
        let next = match by_page.get(&p.page) {
            Some(old) => merge_page(old, p),
            None => p.clone(),
        };
        by_page.insert(p.page, next);
    }
    by_page.into_values().collect()
}

/// `pages` without the ones that hold nothing at all.
pub fn without_empty(pages: Vec<PageInk>) -> Vec<PageInk> {
    pages
        .into_iter()
        .filter(|p| !p.strokes.is_empty())
        .collect()
}

/// The pages' ink as written, in bytes.
pub fn written_len(pages: &[PageInk]) -> usize {
    pages
        .iter()
        .map(|p| serde_json::to_string(p).map_or(0, |s| s.len()) + 6)
        .sum()
}

/// Checks all of a block's ink: every page, once each, and the total size.
pub fn validate_pages(pages: &[PageInk]) -> Result<(), PdfError> {
    if pages.len() > MAX_INK_PAGES {
        return Err(PdfError::Invalid("There is ink on too many pages.".into()));
    }
    let mut seen = std::collections::HashSet::new();
    for p in pages {
        p.validate()?;
        if !seen.insert(p.page) {
            return Err(PdfError::Malformed("two pages share their ink".into()));
        }
    }
    Ok(())
}

/// The strokes' tombstones for every drawn stroke of a page: the page's ink
/// cleared in one edit that a merge honours. `now` stamps them.
pub fn cleared(page: &PageInk, now: i64) -> PageInk {
    let mut canvas = page.to_canvas();
    let refs: Vec<canvas::StrokeRef> = canvas
        .live_strokes()
        .map(|(n, s)| canvas::StrokeRef {
            node: n.id.clone(),
            stroke: s.id.clone(),
        })
        .collect();
    canvas.delete_strokes(&refs, now);
    page.from_canvas(&canvas)
}

/// The companion attachment: all of a PDF's ink.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InkFile {
    pub version: u32,
    /// The PDF the ink is on, `file:abc123`.
    pub file: String,
    #[serde(default)]
    pub pages: Vec<PageInk>,
}

impl InkFile {
    pub fn new(file: &str, pages: Vec<PageInk>) -> InkFile {
        InkFile {
            version: INK_FILE_VERSION,
            file: file.to_owned(),
            pages,
        }
    }

    /// The attachment's bytes.
    pub fn to_bytes(&self) -> Result<Vec<u8>, PdfError> {
        let bytes = serde_json::to_vec(self).map_err(|e| PdfError::Malformed(e.to_string()))?;
        if bytes.len() > MAX_INK_BYTES {
            return Err(PdfError::Invalid(FULL.into()));
        }
        Ok(bytes)
    }

    /// Reads and checks the attachment's bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<InkFile, PdfError> {
        if bytes.len() > MAX_INK_BYTES {
            return Err(PdfError::Malformed("the ink file is too large".into()));
        }
        let file: InkFile =
            serde_json::from_slice(bytes).map_err(|e| PdfError::Malformed(e.to_string()))?;
        if file.version == 0 || file.version > INK_FILE_VERSION {
            return Err(PdfError::NewerVersion(file.version));
        }
        validate_pages(&file.pages)?;
        Ok(file)
    }
}

/// The attachment id (`file:abc123`) an `inkFile` names, written either as
/// that or as the link (`inkdrop://file:abc123`) that makes the note own it.
pub fn file_id(name: &str) -> Option<String> {
    let id = name
        .strip_prefix(kanso_core::links::FILE_LINK_PREFIX)
        .unwrap_or(name.strip_prefix("file:").unwrap_or(name));
    let ok = !id.is_empty()
        && id.len() <= 64
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-');
    ok.then(|| format!("file:{id}"))
}

/// The `inkFile` value for attachment `id` (`file:abc123`): the link.
pub fn file_link(id: &str) -> String {
    format!(
        "{}{}",
        kanso_core::links::FILE_LINK_PREFIX,
        id.strip_prefix("file:").unwrap_or(id)
    )
}

/// What is said when a PDF can hold no more ink.
pub const FULL: &str = "This PDF has all the pencil marks it can hold. Erase some to draw more.";

#[cfg(test)]
mod tests;
