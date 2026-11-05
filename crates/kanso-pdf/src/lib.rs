//! PDF reading in Kanso (issue #43, step 1): the note's annotation block, the
//! links that open a page or a highlight, and the quotes written into a note.
//!
//! Annotations, links and quotes are pure data and text handling. The PDF
//! engine lives in a separate helper process (step 3): [`protocol`] is what
//! the app and the helper say to each other, [`client`] starts and supervises
//! the helper, [`contain`] is how the helper confines itself, and, with the
//! `render` feature, [`worker`] is the helper's own code around PDFium. The
//! app never links PDFium.

pub mod annotation;
pub mod client;
pub mod contain;
#[cfg(feature = "test-support")]
pub mod fixtures;
#[cfg(feature = "test-support")]
pub mod hostile;
pub mod invert;
pub mod link;
pub mod model;
pub mod protocol;
pub mod quote;
pub mod text;
#[cfg(feature = "render")]
pub mod worker;

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
