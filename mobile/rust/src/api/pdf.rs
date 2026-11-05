//! Reading a PDF on the phone (issue #43): open an attachment of a note,
//! draw its pages, select and search its text, and highlight. The reader
//! runs the desktop's PDF code in the app (`kanso-mobile`'s `pdf_reader`).

use flutter_rust_bridge::frb;
use kanso_mobile::{
    KansoError, Library, PdfCharPos, PdfDeclined, PdfEdited, PdfFound, PdfInfo, PdfMarks,
    PdfOpening, PdfPageImage, PdfReader, PdfSelection,
};

/// Opening a PDF, and the highlights a note has for it.
#[frb(opaque)]
pub struct KansoPdf(Library);

impl KansoPdf {
    pub(super) fn new(library: Library) -> KansoPdf {
        KansoPdf(library)
    }

    /// Opens attachment `file` of note `note_id`. The password is used for
    /// this call only. Blocks while the document is read (off the UI thread).
    pub fn open(
        &self,
        note_id: String,
        file: String,
        password: Option<String>,
    ) -> Result<PdfOpenResult, KansoError> {
        Ok(
            match self.0.pdf_open(&note_id, &file, password.as_deref())? {
                PdfOpening::Opened(reader, info) => PdfOpenResult::Opened {
                    reader: KansoPdfReader(reader),
                    info,
                },
                PdfOpening::Declined(reason) => PdfOpenResult::Declined { reason },
            },
        )
    }

    /// The note's highlights for a PDF, as the reader draws them.
    pub fn marks(&self, note_id: String, file: String) -> Result<PdfMarks, KansoError> {
        self.0.pdf_marks(&note_id, &file)
    }
}

/// What opening a PDF came to.
pub enum PdfOpenResult {
    Opened {
        reader: KansoPdfReader,
        info: PdfInfo,
    },
    Declined {
        reason: PdfDeclined,
    },
}

/// A PDF open for reading. Dropping it ends the reader.
#[frb(opaque)]
pub struct KansoPdfReader(PdfReader);

impl KansoPdfReader {
    /// Draws a page (0-based), `width_px` wide, for a dark reader when
    /// `invert`. BGRA pixels.
    pub fn render(
        &self,
        page: u32,
        width_px: u32,
        invert: bool,
    ) -> Result<PdfPageImage, KansoError> {
        self.0.render(page, width_px, invert)
    }

    /// The word at a press on a page (1-based; `x` and `y` are fractions of
    /// the page from its top left), if one is near.
    pub fn word_at(&self, page: u32, x: f64, y: f64) -> Result<Option<PdfSelection>, KansoError> {
        self.0.word_at(page, x, y)
    }

    /// The line at the press.
    pub fn line_at(&self, page: u32, x: f64, y: f64) -> Result<Option<PdfSelection>, KansoError> {
        self.0.line_at(page, x, y)
    }

    /// The selection from the fixed end `anchor` to the character nearest the
    /// point: a handle was dragged there.
    pub fn extend(
        &self,
        anchor: PdfCharPos,
        page: u32,
        x: f64,
        y: f64,
    ) -> Result<Option<PdfSelection>, KansoError> {
        self.0.extend(anchor, page, x, y)
    }

    /// The selection between two places, drawn again.
    pub fn select(
        &self,
        start: PdfCharPos,
        end: PdfCharPos,
    ) -> Result<Option<PdfSelection>, KansoError> {
        self.0.select(start, end)
    }

    /// Searches the text. `limit` hits are returned (at most 500) and the
    /// total counted.
    pub fn find(&self, query: String, limit: u32) -> Result<PdfFound, KansoError> {
        self.0.find(&query, limit)
    }

    /// Highlights the selection, read at note revision `base_rev`. With
    /// `quote` the quote goes into the note in the same save. A note that
    /// moved meanwhile has the highlight made again on its newer text.
    pub fn add_highlight(
        &self,
        base_rev: String,
        start: PdfCharPos,
        end: PdfCharPos,
        color: String,
        quote: bool,
    ) -> Result<PdfEdited, KansoError> {
        self.0.add_highlight(&base_rev, start, end, &color, quote)
    }

    /// Ends the reader now.
    #[frb(sync)]
    pub fn close(&self) {
        self.0.close()
    }
}
