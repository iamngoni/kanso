//! The data the phone's PDF reader trades with the app (issue #43): what a
//! document is, a page as pixels, a selection as boxes to draw, a search, and
//! what an edit to the highlights came to. What the calls do is in
//! [`crate::pdf_reader`].
//!
//! Every box is normalised to the page as shown, origin at the top left, like
//! a highlight's quad in the note: the app multiplies by the size it draws the
//! page at, so a zoom or a rotation never changes what is stored.

/// A page's size in points, as shown (rotation applied).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PdfPageSize {
    pub width: f32,
    pub height: f32,
}

/// An opened document.
#[derive(Clone, Debug, PartialEq)]
pub struct PdfInfo {
    /// The attachment: `file:<id>`.
    pub file: String,
    pub name: String,
    pub pages: u32,
    /// One size for each page, page 1 first.
    pub sizes: Vec<PdfPageSize>,
    pub encrypted: bool,
    /// The note this was opened from, which its highlights belong to.
    pub note_id: String,
    /// Whether the note's highlights for this PDF can be changed now.
    pub editable: bool,
}

/// Why a PDF was not opened, in the words the app shows. None of these is an
/// error of the library: they are answers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PdfDeclined {
    /// The PDF is encrypted and a password is needed.
    NeedsPassword,
    /// The password given is not right.
    WrongPassword,
    /// The note is closed, and its session has no key for the sealed PDF
    /// (the desktop's `PDF_IN_LOCKED_NOTE_OPEN`, word for word).
    LockedNote { message: String },
    /// The reader isn't in this build, or PDFium did not load.
    Unavailable { message: String },
    /// The PDF itself is refused: unreadable, too many pages, a page too
    /// large, too big a file.
    Refused { message: String },
    /// The attachment isn't a PDF, or this note doesn't link it.
    NotAPdf,
}

/// A page drawn: `bgra` is `width * height * 4` bytes, blue first.
#[derive(Clone, PartialEq, Eq)]
pub struct PdfPageImage {
    /// 0-based.
    pub page: u32,
    pub width: u32,
    pub height: u32,
    pub bgra: Vec<u8>,
}

impl std::fmt::Debug for PdfPageImage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PdfPageImage({}: {}x{})",
            self.page, self.width, self.height
        )
    }
}

/// A box on a page.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PdfRect {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

/// The boxes on one page (1-based), for a selection, a hit of a search or a
/// highlight.
#[derive(Clone, Debug, PartialEq)]
pub struct PdfPageMarks {
    pub page: u32,
    pub rects: Vec<PdfRect>,
}

/// A character's place: 1-based page, then reading-order index.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PdfCharPos {
    pub page: u32,
    pub index: u32,
}

/// Selected words: where they run from and to, the text to copy, and the
/// boxes to draw.
#[derive(Clone, Debug, PartialEq)]
pub struct PdfSelection {
    pub start: PdfCharPos,
    pub end: PdfCharPos,
    pub text: String,
    pub marks: Vec<PdfPageMarks>,
}

/// One hit of a search.
#[derive(Clone, Debug, PartialEq)]
pub struct PdfFindHit {
    /// The page it starts on (1-based).
    pub page: u32,
    pub marks: Vec<PdfPageMarks>,
}

/// A search of the document. `total` is every hit; `hits` the first of them
/// (the app steps through those and asks again to go on).
#[derive(Clone, Debug, PartialEq)]
pub struct PdfFound {
    pub total: u32,
    pub hits: Vec<PdfFindHit>,
    /// Pages with no text layer, which a search can't see into. Zero when
    /// every page has been read.
    pub pages_without_text: u32,
}

/// A highlight as the reader draws it.
#[derive(Clone, Debug, PartialEq)]
pub struct PdfMark {
    pub id: String,
    pub color: String,
    /// The page it starts on (1-based).
    pub page: u32,
    pub marks: Vec<PdfPageMarks>,
}

/// A PDF's highlights as the note has them, with the revision they were read
/// at.
#[derive(Clone, Debug, PartialEq)]
pub struct PdfMarks {
    pub rev: String,
    pub editable: bool,
    pub why: Option<String>,
    pub marks: Vec<PdfMark>,
    /// Pencil strokes, in PDF page units (1,000 units across each page).
    pub ink: Vec<PdfInkPage>,
}

/// Pencil strokes on one PDF page, in page units.
#[derive(Clone, Debug, PartialEq)]
pub struct PdfInkPage {
    pub page: u32,
    pub strokes: Vec<PdfInkStroke>,
}

/// A saved or newly drawn PDF pencil stroke.
#[derive(Clone, Debug, PartialEq)]
pub struct PdfInkStroke {
    pub tool: String,
    pub color: String,
    pub width: f64,
    pub points: Vec<PdfInkPoint>,
}

/// A pen sample in PDF page units.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PdfInkPoint {
    pub x: f64,
    pub y: f64,
    pub pressure: f64,
}

/// What an edit to the highlights came to. An edit is never dropped: a note
/// that moved meanwhile has the change made again on its newer text, and
/// `merged` says so; the one thing that stops an edit is a highlight that is
/// no longer there, and `gone` says that.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PdfEdited {
    /// The note's revision now.
    pub rev: String,
    /// The highlight made, for a new one.
    pub id: Option<String>,
    /// The note had changed since it was read, and the edit was made again on
    /// what it holds now.
    pub merged: bool,
    /// The highlight the edit was about was removed meanwhile (on another
    /// device); nothing was written.
    pub gone: bool,
}
