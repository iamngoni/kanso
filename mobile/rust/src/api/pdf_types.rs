//! The PDF reader's data types, declared again for the generator. What the
//! fields mean is said where the facade declares them
//! (`crates/kanso-mobile/src/pdf_reader_model.rs`).

use flutter_rust_bridge::frb;
// Public: the generated code names these types by this module's path.
pub use kanso_mobile::{
    PdfCharPos, PdfDeclined, PdfEdited, PdfFindHit, PdfFound, PdfInfo, PdfMark, PdfMarks,
    PdfPageImage, PdfPageMarks, PdfPageSize, PdfRect, PdfSelection,
};

#[frb(mirror(PdfPageSize))]
pub struct _PdfPageSize {
    pub width: f32,
    pub height: f32,
}

#[frb(mirror(PdfInfo))]
pub struct _PdfInfo {
    pub file: String,
    pub name: String,
    pub pages: u32,
    pub sizes: Vec<PdfPageSize>,
    pub encrypted: bool,
    pub note_id: String,
    pub editable: bool,
}

#[frb(mirror(PdfDeclined))]
pub enum _PdfDeclined {
    NeedsPassword,
    WrongPassword,
    LockedNote { message: String },
    Unavailable { message: String },
    Refused { message: String },
    NotAPdf,
}

#[frb(mirror(PdfPageImage))]
pub struct _PdfPageImage {
    pub page: u32,
    pub width: u32,
    pub height: u32,
    pub bgra: Vec<u8>,
}

#[frb(mirror(PdfRect))]
pub struct _PdfRect {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

#[frb(mirror(PdfPageMarks))]
pub struct _PdfPageMarks {
    pub page: u32,
    pub rects: Vec<PdfRect>,
}

#[frb(mirror(PdfCharPos))]
pub struct _PdfCharPos {
    pub page: u32,
    pub index: u32,
}

#[frb(mirror(PdfSelection))]
pub struct _PdfSelection {
    pub start: PdfCharPos,
    pub end: PdfCharPos,
    pub text: String,
    pub marks: Vec<PdfPageMarks>,
}

#[frb(mirror(PdfFindHit))]
pub struct _PdfFindHit {
    pub page: u32,
    pub marks: Vec<PdfPageMarks>,
}

#[frb(mirror(PdfFound))]
pub struct _PdfFound {
    pub total: u32,
    pub hits: Vec<PdfFindHit>,
    pub pages_without_text: u32,
}

#[frb(mirror(PdfMark))]
pub struct _PdfMark {
    pub id: String,
    pub color: String,
    pub page: u32,
    pub marks: Vec<PdfPageMarks>,
}

#[frb(mirror(PdfMarks))]
pub struct _PdfMarks {
    pub rev: String,
    pub editable: bool,
    pub why: Option<String>,
    pub marks: Vec<PdfMark>,
}

#[frb(mirror(PdfEdited))]
pub struct _PdfEdited {
    pub rev: String,
    pub id: Option<String>,
    pub merged: bool,
    pub gone: bool,
}
