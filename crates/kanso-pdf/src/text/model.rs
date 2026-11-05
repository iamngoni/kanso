//! The text layer's data: what the worker sends, and what grouping produces.

use std::collections::BTreeMap;
use std::ops::Range;

use serde::{Deserialize, Serialize};

use crate::model::Quad;

fn is_zero(v: &u8) -> bool {
    *v == 0
}

fn is_false(v: &bool) -> bool {
    !*v
}

/// One character of a page, as the worker reads it from PDFium.
///
/// Wire shape: `{"ch":"H","quad":[x0,y0,x1,y1],"turns":0,"space_before":true}`;
/// `turns` may be left out when it is 0 and `space_before` when it is false.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextChar {
    pub ch: char,
    /// The character's tight box on the page as it is displayed (after the
    /// page's rotation), normalised like a highlight quad: `x / width`,
    /// `y / height`, origin top-left.
    pub quad: Quad,
    /// The direction the text runs in on the unrotated page, in clockwise
    /// quarter turns: 0 left to right, 1 top to bottom, 2 right to left,
    /// 3 bottom to top. From PDFium's character angle `a` (degrees, which
    /// PDFium measures clockwise: 90 is top to bottom) this is
    /// `(a / 90).rem_euclid(4)`.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub turns: u8,
    /// PDFium put a space (or a line break) in the text just before this
    /// character: a space character the PDF itself set, or one PDFium
    /// generated between two runs. The characters themselves are not sent,
    /// only this mark on the next visible one. The layout breaks words
    /// here, and only falls back to measuring gaps on a page where no
    /// character carries the mark.
    #[serde(default, skip_serializing_if = "is_false")]
    pub space_before: bool,
}

/// One page's characters. Any order is accepted; the spaces and line breaks
/// of the PDF are not characters here but the [`TextChar::space_before`]
/// mark on the character after them.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PageText {
    /// 1-based page number.
    pub page: u32,
    /// The displayed page's size in points (after rotation).
    pub width: f64,
    pub height: f64,
    /// The page's `/Rotate` in degrees clockwise: 0, 90, 180 or 270.
    #[serde(default)]
    pub rotation: u16,
    pub chars: Vec<TextChar>,
}

/// What lies between a glyph and the one before it in reading order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sep {
    None,
    Space,
    /// A new line inside a paragraph.
    Line,
    /// A new block (paragraph, column, cell).
    Paragraph,
}

impl Sep {
    /// The text this separator copies as.
    pub fn as_str(self) -> &'static str {
        match self {
            Sep::None => "",
            Sep::Space => " ",
            Sep::Line => "\n",
            Sep::Paragraph => "\n\n",
        }
    }
}

/// What copy puts between two pages. One newline, so a sentence that runs
/// across a page break stays one Markdown paragraph in the note.
pub const PAGE_BREAK: &str = "\n";

/// A character placed in reading order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glyph {
    pub ch: char,
    pub quad: Quad,
    pub sep: Sep,
    /// Index into [`PageLayout::lines`].
    pub line: u32,
    /// Index into [`PageLayout::blocks`].
    pub block: u32,
}

/// A run of glyphs on one baseline (a column's line, not a page-wide row).
#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    pub glyphs: Range<usize>,
    /// The line's whole box, displayed.
    pub band: Quad,
    /// Which way the line runs on the display, in clockwise quarter turns.
    pub turns: u8,
}

/// Lines that belong together: a paragraph, or one cell of a table.
#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub glyphs: Range<usize>,
}

/// One page, grouped and in reading order.
#[derive(Clone, Debug, PartialEq)]
pub struct PageLayout {
    pub page: u32,
    pub width: f64,
    pub height: f64,
    pub glyphs: Vec<Glyph>,
    pub lines: Vec<Line>,
    pub blocks: Vec<Block>,
}

/// A character's place in the document: page, then reading-order index.
/// Ordering is reading order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CharPos {
    pub page: u32,
    pub index: usize,
}

/// A point on a displayed page, normalised like a quad.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub page: u32,
    pub x: f64,
    pub y: f64,
}

/// A run of characters in reading order, both ends included. `start <= end`
/// always, so the same two points give the same selection in either order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Selection {
    pub start: CharPos,
    pub end: CharPos,
}

/// A selection as step 1's highlight wants it.
#[derive(Clone, Debug, PartialEq)]
pub struct HighlightParts {
    /// Quads by 1-based page, at most [`crate::MAX_QUADS`] in all.
    pub quads: BTreeMap<u32, Vec<Quad>>,
    /// The copied text, cut to [`crate::MAX_QUOTE_CHARS`].
    pub text: String,
}

/// The pages the reader has text for, laid out and sorted by page number.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextDoc {
    pub(crate) pages: Vec<PageLayout>,
}
