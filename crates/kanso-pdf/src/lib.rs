//! PDF reading in Kanso (issue #43, step 1): the note's annotation block, the
//! links that open a page or a highlight, and the quotes written into a note.
//!
//! Everything here is pure data and text handling. The PDF engine lives in a
//! separate helper process added in a later step; nothing in this crate
//! touches a PDF's bytes.

pub mod annotation;
pub mod link;
pub mod model;
pub mod quote;
pub mod text;

pub use annotation::{BlockSpan, find_blocks, new_highlight_id, upsert_block};
pub use link::PdfLink;
pub use model::{
    BLOCK_FENCE, Color, Highlight, MAX_BLOCK_CHARS, MAX_COMMENT_CHARS, MAX_QUADS, MAX_QUOTE_CHARS,
    PdfBlock, Quad,
};
pub use quote::{Edit, append_quote, render_quote};
pub use text::{
    CharPos, FindIndex, Glyph, HighlightParts, PageLayout, PageText, Point, Selection, TextChar,
    TextDoc,
};

/// Why a block, a highlight or a link was refused.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PdfError {
    #[error("The PDF annotations could not be read: {0}")]
    Malformed(String),
    #[error("These PDF annotations were written by a newer Kanso (version {0}).")]
    NewerVersion(u32),
    #[error("This PDF has too many highlights to save in the note.")]
    BlockFull,
    #[error("{0}")]
    Invalid(String),
}
