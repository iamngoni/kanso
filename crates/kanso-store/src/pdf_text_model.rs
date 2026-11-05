//! The shapes of text in PDFs (`pdf_text.rs`).

/// The most characters kept of one page; a page over it is indexed
/// truncated.
pub const MAX_PAGE_CHARS: usize = 200_000;

/// The most bytes of text kept for one PDF, over all its pages; pages past
/// it are kept empty, so the page count stays right.
pub const MAX_PDF_CHARS: usize = 4 * 1024 * 1024;

/// A PDF whose pages haven't been read, or were read from other bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PdfToIndex {
    pub id: String,
    pub md5: String,
}
