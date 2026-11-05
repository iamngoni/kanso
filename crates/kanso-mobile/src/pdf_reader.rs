//! The PDF reader on the phone (issue #43): pages, text, search and
//! highlights, on the code the desktop reader runs.
//!
//! * Pages and the text layer come from `kanso-pdf`'s worker loop, run on a
//!   thread of the app (a phone can't start a process: see
//!   [`kanso_pdf::local`]). The client around it is the desktop's, so the
//!   frame, page, pixel and time caps are the same, and a page and its
//!   characters are the same bytes.
//! * Selection, copy, word expansion, search and a highlight's quads are
//!   `kanso_pdf::text` (`TextDoc`, `FindIndex`).
//! * A highlight is written with `kanso_pdf::highlights::Annotations` and
//!   `merge`, in one save of the note's text, so the phone and the desktop
//!   write the same block and unite their changes the same way.
//!
//! Nothing here draws. The app asks for a page as pixels and for boxes
//! (normalised to the page) and draws both.

use std::sync::{Arc, Mutex, MutexGuard};

use kanso_core::links::FILE_LINK_PREFIX;
use kanso_pdf::client::{Deadlines, Worker, WorkerError};
use kanso_pdf::highlights::{Annotations, Status};
use kanso_pdf::model::Quad;
use kanso_pdf::ocr::PdfOcr;
use kanso_pdf::protocol::{ErrorKind, MAX_PAGE_CHARS};
use kanso_pdf::text::{CharPos, FindIndex, PageText, Point, Selection, TextDoc};

use crate::boundary::guard;
use crate::errors::KansoError;
use crate::library::Library;
use crate::pdf_notes::{self, write_with_retry};
use crate::pdf_reader_model::{
    PdfCharPos, PdfDeclined, PdfEdited, PdfFindHit, PdfFound, PdfInfo, PdfInkStroke, PdfMark,
    PdfMarks, PdfPageImage, PdfPageMarks, PdfPageSize, PdfRect, PdfSelection,
};

/// What a PDF in a locked note answers while the note is closed: unlocking
/// opens it (its bytes are sealed with the lock, and are opened with the file
/// keys of this session; nothing derived from it is ever written unsealed).
pub const PDF_IN_LOCKED_NOTE_OPEN: &str = "Unlock the note to open its PDF.";

/// The most hits of a search handed to the app at once.
pub const FIND_HITS_SHOWN: usize = 500;
/// How far from a word, in points, a press still counts as on it.
const NEAR_POINTS: f64 = 28.0;

/// The outcome of opening a PDF.
pub enum PdfOpening {
    Opened(PdfReader, PdfInfo),
    Declined(PdfDeclined),
}

/// A PDF open on the phone. Dropping it frees the page cache and ends the
/// reader's thread. Ink uses the desktop's canvas stroke format and PDF
/// annotation merge code; the app paints the returned page-unit strokes over
/// the rendered page and sends new strokes through the same revision retry.
pub struct PdfReader {
    library: Library,
    note_id: String,
    file: String,
    name: String,
    md5: String,
    inner: Arc<Mutex<Inner>>,
}

struct Inner {
    worker: Worker,
    /// OCR for the current PDF bytes, available only for unlocked plain files.
    ocr: Option<PdfOcr>,
    /// A closed or newly locked reader is terminal; its bytes and text are
    /// dropped before any later request can read from the caches.
    closed: bool,
    /// Each page's size in points, as shown.
    sizes: Vec<[f32; 2]>,
    text: TextDoc,
    /// Which pages have been read (0-based).
    read: Vec<bool>,
    find: Option<FindIndex>,
}

fn words(error: &WorkerError) -> KansoError {
    KansoError::invalid(error.message())
}

impl Library {
    /// Opens attachment `file` (`file:<id>`) of note `note_id` as a PDF.
    /// The password is for this call only and kept nowhere. A note that is
    /// locked declines (see [`PDF_IN_LOCKED_NOTE_OPEN`]); so does a file
    /// that isn't a PDF. Blocks while the document is read, so the app calls
    /// it off the UI thread.
    pub fn pdf_open(
        &self,
        note_id: &str,
        file: &str,
        password: Option<&str>,
    ) -> Result<PdfOpening, KansoError> {
        guard(|| {
            // A locked note's text, and the PDF sealed with it, open once the
            // session holds the lock's keys (as the recording player's parts
            // do): `readable_text` is that check, and `attachment` opens the
            // sealed bytes with the file keys the session cached.
            let Some((_, body)) = self.readable_text(note_id)? else {
                return Ok(PdfOpening::Declined(PdfDeclined::LockedNote {
                    message: PDF_IN_LOCKED_NOTE_OPEN.to_owned(),
                }));
            };
            if !kanso_core::tasks::linked_file_ids(&body).contains(&file.to_owned()) {
                return Ok(PdfOpening::Declined(PdfDeclined::NotAPdf));
            }
            let attachment = match self.attachment(file) {
                Ok(attachment) => attachment,
                Err(KansoError::NotFound) => return Ok(PdfOpening::Declined(PdfDeclined::NotAPdf)),
                // Sealed, and this session has not got the key.
                Err(KansoError::Locked) => {
                    return Ok(PdfOpening::Declined(PdfDeclined::LockedNote {
                        message: PDF_IN_LOCKED_NOTE_OPEN.to_owned(),
                    }));
                }
                Err(error) => return Err(error),
            };
            if attachment.content_type != "application/pdf"
                || !attachment.bytes.starts_with(b"%PDF-")
            {
                return Ok(PdfOpening::Declined(PdfDeclined::NotAPdf));
            }
            let md5 = self
                .inner
                .store
                .file_meta(file)?
                .and_then(|meta| meta.md5digest)
                .unwrap_or_default();
            // The store validates OCR against the file's current digest and
            // excludes sealed files. Keep only a result for the exact bytes
            // being opened; closing a reader drops this derived text too.
            let ocr = self
                .inner
                .store
                .pdf_ocr(file)?
                .filter(|ocr| !md5.is_empty() && ocr.md5 == md5);
            let mut worker = match Worker::start_local(Deadlines::default()) {
                Ok(worker) => worker,
                Err(error) => {
                    return Ok(PdfOpening::Declined(PdfDeclined::Unavailable {
                        message: match error {
                            WorkerError::Unavailable(_) => {
                                "PDFs can't be read on this phone.".to_owned()
                            }
                            other => other.message(),
                        },
                    }));
                }
            };
            let opened = match worker.open(&attachment.bytes, password) {
                Ok(opened) => opened,
                Err(WorkerError::Refused { kind, message }) => {
                    return Ok(PdfOpening::Declined(match kind {
                        ErrorKind::PasswordRequired => PdfDeclined::NeedsPassword,
                        ErrorKind::WrongPassword => PdfDeclined::WrongPassword,
                        _ => PdfDeclined::Refused { message },
                    }));
                }
                Err(other) => {
                    return Ok(PdfOpening::Declined(PdfDeclined::Refused {
                        message: other.message(),
                    }));
                }
            };
            let editable = Annotations::from_body(&body, file, &md5).is_editable();
            let info = PdfInfo {
                file: file.to_owned(),
                name: attachment.name.clone(),
                pages: opened.pages,
                sizes: opened
                    .sizes
                    .iter()
                    .map(|[width, height]| PdfPageSize {
                        width: *width,
                        height: *height,
                    })
                    .collect(),
                encrypted: opened.encrypted,
                note_id: note_id.to_owned(),
                editable,
            };
            let reader = PdfReader {
                library: self.clone(),
                note_id: note_id.to_owned(),
                file: file.to_owned(),
                name: attachment.name,
                md5,
                inner: Arc::new(Mutex::new(Inner {
                    read: vec![false; opened.pages as usize],
                    closed: false,
                    sizes: opened.sizes,
                    worker,
                    ocr,
                    text: TextDoc::default(),
                    find: None,
                })),
            };
            Ok(PdfOpening::Opened(reader, info))
        })
    }

    /// A PDF's highlights as the reader draws them, with the revision they
    /// were read at.
    pub fn pdf_marks(&self, note_id: &str, file: &str) -> Result<PdfMarks, KansoError> {
        guard(|| {
            let (note, body) = self.readable_text(note_id)?.ok_or(KansoError::Locked)?;
            let md5 = self
                .inner
                .store
                .file_meta(file)?
                .and_then(|meta| meta.md5digest)
                .unwrap_or_default();
            let annots = Annotations::from_body(&body, file, &md5);
            let (editable, why) = match &annots.status {
                Status::Editable => (true, None),
                Status::Restorable => (false, None),
                Status::Unreadable(why) => (false, Some(why.clone())),
            };
            let marks = annots
                .block
                .highlights
                .iter()
                .map(|h| PdfMark {
                    id: h.id.clone(),
                    color: pdf_notes::color_name(h.color).to_owned(),
                    page: h.page,
                    marks: marks_of(h.quads.iter().map(|(page, q)| (*page, q.as_slice()))),
                })
                .collect();
            let ink = self.pdf_ink_pages(&note, &body, file)?;
            Ok(PdfMarks {
                rev: note.rev.unwrap_or_default(),
                editable,
                why,
                marks,
                ink,
            })
        })
    }
}

impl PdfReader {
    fn lock(&self) -> Result<MutexGuard<'_, Inner>, KansoError> {
        self.inner
            .lock()
            .map_err(|_| KansoError::invalid("This PDF couldn't be read."))
    }

    /// Refuses a stale reader after its note is locked, and clears anything
    /// derived from the PDF before returning the lock error.
    fn ensure_readable(&self, inner: &mut Inner) -> Result<(), KansoError> {
        if inner.closed {
            return Err(KansoError::Locked);
        }
        if self.library.readable_text(&self.note_id)?.is_some() {
            return Ok(());
        }
        Self::close_inner(inner);
        Err(KansoError::Locked)
    }

    fn close_inner(inner: &mut Inner) {
        inner.closed = true;
        inner.worker.kill();
        inner.text = TextDoc::default();
        inner.find = None;
        inner.read.clear();
        inner.sizes.clear();
        inner.ocr = None;
    }

    /// Draws page `page` (0-based), `width_px` wide (the worker caps it), for
    /// a dark reader when `invert`: text light, pictures in their colours.
    pub fn render(
        &self,
        page: u32,
        width_px: u32,
        invert: bool,
    ) -> Result<PdfPageImage, KansoError> {
        guard(|| {
            let mut inner = self.lock()?;
            self.ensure_readable(&mut inner)?;
            let drawn = inner
                .worker
                .render_as(page, width_px, invert)
                .map_err(|e| words(&e))?;
            Ok(PdfPageImage {
                page,
                width: drawn.width,
                height: drawn.height,
                bgra: drawn.bgra,
            })
        })
    }

    /// Reads the text of page `page` (1-based) if it hasn't been.
    fn load(inner: &mut Inner, page: u32) -> Result<(), KansoError> {
        let at = page.saturating_sub(1) as usize;
        if page == 0 || at >= inner.read.len() {
            return Ok(());
        }
        if inner.read[at] {
            return Ok(());
        }
        let mut text = match inner.worker.text(page - 1) {
            Ok(text) => text,
            // A page the worker won't give text for has none.
            Err(WorkerError::Refused { .. }) => empty_page(page, inner.sizes[at]),
            Err(other) => return Err(words(&other)),
        };
        if text.chars.is_empty()
            && let Some(scanned) = ocr_words(inner.ocr.as_ref(), page)
        {
            text = scanned;
        }
        inner.text.set_page(&text);
        inner.read[at] = true;
        inner.find = None;
        Ok(())
    }

    /// The word nearest (page `page` 1-based, `x` and `y` normalised) when
    /// it is close enough to be the one pressed.
    pub fn word_at(&self, page: u32, x: f64, y: f64) -> Result<Option<PdfSelection>, KansoError> {
        guard(|| {
            let mut inner = self.lock()?;
            self.ensure_readable(&mut inner)?;
            Self::load(&mut inner, page)?;
            let Some(pos) = Self::near(&inner, Point { page, x, y }) else {
                return Ok(None);
            };
            Ok(inner.text.word_at(pos).map(|sel| view(&inner.text, sel)))
        })
    }

    /// The line of text at the point, for a double press.
    pub fn line_at(&self, page: u32, x: f64, y: f64) -> Result<Option<PdfSelection>, KansoError> {
        guard(|| {
            let mut inner = self.lock()?;
            self.ensure_readable(&mut inner)?;
            Self::load(&mut inner, page)?;
            let Some(pos) = Self::near(&inner, Point { page, x, y }) else {
                return Ok(None);
            };
            Ok(inner.text.line_at(pos).map(|sel| view(&inner.text, sel)))
        })
    }

    /// The selection that runs from the fixed end `anchor` to the character
    /// nearest the point: a handle was dragged there.
    pub fn extend(
        &self,
        anchor: PdfCharPos,
        page: u32,
        x: f64,
        y: f64,
    ) -> Result<Option<PdfSelection>, KansoError> {
        guard(|| {
            let mut inner = self.lock()?;
            self.ensure_readable(&mut inner)?;
            Self::load(&mut inner, page)?;
            Self::load(&mut inner, anchor.page)?;
            let Some(pos) = inner.text.hit(Point { page, x, y }) else {
                return Ok(None);
            };
            let sel = Selection::between(from_pos(anchor), pos);
            Ok(Some(view(&inner.text, sel)))
        })
    }

    /// The selection from one place to another, as the app asks for it again
    /// (after a zoom, say). Pages are read as needed.
    pub fn select(
        &self,
        start: PdfCharPos,
        end: PdfCharPos,
    ) -> Result<Option<PdfSelection>, KansoError> {
        guard(|| {
            let mut inner = self.lock()?;
            self.ensure_readable(&mut inner)?;
            Self::load(&mut inner, start.page)?;
            Self::load(&mut inner, end.page)?;
            let (a, b) = (from_pos(start), from_pos(end));
            let valid = |p: CharPos| {
                inner
                    .text
                    .page(p.page)
                    .is_some_and(|page| p.index < page.glyphs.len())
            };
            if !valid(a) || !valid(b) {
                return Ok(None);
            }
            Ok(Some(view(&inner.text, Selection::between(a, b))))
        })
    }

    /// The nearest character to `p` when it lies within [`NEAR_POINTS`] of
    /// it.
    fn near(inner: &Inner, p: Point) -> Option<CharPos> {
        let pos = inner.text.hit(p)?;
        let page = inner.text.page(p.page)?;
        let [x0, y0, x1, y1] = page.glyphs.get(pos.index)?.quad.0;
        let dx = (x0 - p.x).max(p.x - x1).max(0.0) * page.width;
        let dy = (y0 - p.y).max(p.y - y1).max(0.0) * page.height;
        (dx.hypot(dy) <= NEAR_POINTS).then_some(pos)
    }

    /// Searches the document's text. Reads every page not yet read (a scan's
    /// pages hold no text, and are counted in `pages_without_text`).
    pub fn find(&self, query: &str, limit: u32) -> Result<PdfFound, KansoError> {
        guard(|| {
            let mut inner = self.lock()?;
            self.ensure_readable(&mut inner)?;
            for page in 1..=inner.read.len() as u32 {
                Self::load(&mut inner, page)?;
            }
            let without = inner
                .text
                .pages()
                .iter()
                .filter(|p| p.glyphs.is_empty())
                .count() as u32;
            if inner.find.is_none() {
                inner.find = Some(inner.text.find_index());
            }
            let Some(index) = inner.find.as_ref() else {
                return Ok(PdfFound {
                    total: 0,
                    hits: Vec::new(),
                    pages_without_text: without,
                });
            };
            let all = index.hits(query);
            let cap = (limit as usize).min(FIND_HITS_SHOWN);
            let hits = all
                .iter()
                .take(cap)
                .map(|sel| PdfFindHit {
                    page: sel.start.page,
                    marks: marks_of(
                        inner
                            .text
                            .quads(sel)
                            .iter()
                            .map(|(page, q)| (*page, q.as_slice())),
                    ),
                })
                .collect();
            Ok(PdfFound {
                total: all.len() as u32,
                hits,
                pages_without_text: without,
            })
        })
    }

    /// Highlights the selection in `color`, in one save of the note read at
    /// `base_rev`; with `quote` its quote goes into the note in the same
    /// save. If the note moved meanwhile the highlight is made again on its
    /// newer text (the other changes are kept: `kanso_pdf::merge`), never
    /// dropped.
    pub fn add_highlight(
        &self,
        base_rev: &str,
        start: PdfCharPos,
        end: PdfCharPos,
        color: &str,
        quote: bool,
    ) -> Result<PdfEdited, KansoError> {
        guard(|| {
            let color = pdf_notes::parse_color(color)?;
            let parts = {
                let mut inner = self.lock()?;
                self.ensure_readable(&mut inner)?;
                Self::load(&mut inner, start.page)?;
                Self::load(&mut inner, end.page)?;
                inner
                    .text
                    .highlight_parts(&Selection::between(from_pos(start), from_pos(end)))
            };
            let made: std::cell::RefCell<Option<String>> = std::cell::RefCell::new(None);
            let saved = write_with_retry(&self.library, &self.note_id, base_rev, |body| {
                let invalid = |e: kanso_pdf::PdfError| KansoError::invalid(e.to_string());
                let mut annots = Annotations::from_body(body, &self.file, &self.md5);
                let id = annots
                    .add(parts.clone(), color, kanso_core::now_ms())
                    .map_err(invalid)?;
                let Some(edit) = annots.save_edit(body).map_err(invalid)? else {
                    return Ok(None);
                };
                let mut next = edit.apply(body);
                if quote && let Some(edit) = annots.quote_edit(&next, &id, &self.name) {
                    next = edit.apply(&next);
                }
                *made.borrow_mut() = Some(id);
                Ok(Some(next))
            })?;
            Ok(PdfEdited {
                rev: saved.rev,
                id: made.into_inner(),
                merged: saved.merged,
                gone: false,
            })
        })
    }

    /// Adds a pencil stroke to page `page` (1-based), using the same
    /// annotation and merge format as the desktop reader.
    pub fn add_ink_stroke(
        &self,
        base_rev: &str,
        page: u32,
        stroke: PdfInkStroke,
    ) -> Result<PdfEdited, KansoError> {
        self.library
            .pdf_ink_add(&self.note_id, base_rev, &self.file, page, stroke)
    }

    /// The reader's attachment, `file:<id>`.
    pub fn file(&self) -> &str {
        &self.file
    }

    /// The link a quote of page `page` makes (for sharing a page).
    pub fn page_url(&self, page: u32) -> String {
        format!(
            "{FILE_LINK_PREFIX}{}#page={page}",
            self.file.strip_prefix("file:").unwrap_or(&self.file)
        )
    }

    /// Ends the reader now: the page cache and the thread are freed.
    pub fn close(&self) {
        if let Ok(mut inner) = self.inner.lock() {
            Self::close_inner(&mut inner);
        }
    }
}

/// The words of a scanned page, when they have been read for these PDF bytes.
fn ocr_words(ocr: Option<&PdfOcr>, page: u32) -> Option<PageText> {
    ocr.and_then(|ocr| ocr.page(page).map(kanso_pdf::ocr::OcrPage::to_page_text))
}

fn empty_page(page: u32, size: [f32; 2]) -> PageText {
    PageText {
        page,
        width: f64::from(size[0]),
        height: f64::from(size[1]),
        rotation: 0,
        chars: Vec::new(),
    }
}

fn from_pos(pos: PdfCharPos) -> CharPos {
    CharPos {
        page: pos.page,
        index: pos.index as usize,
    }
}

fn to_pos(pos: CharPos) -> PdfCharPos {
    PdfCharPos {
        page: pos.page,
        index: u32::try_from(pos.index).unwrap_or(u32::MAX),
    }
}

fn rect(q: &Quad) -> PdfRect {
    PdfRect {
        x0: q.0[0],
        y0: q.0[1],
        x1: q.0[2],
        y1: q.0[3],
    }
}

/// Boxes by page, in page order.
fn marks_of<'a>(by_page: impl Iterator<Item = (u32, &'a [Quad])>) -> Vec<PdfPageMarks> {
    by_page
        .map(|(page, quads)| PdfPageMarks {
            page,
            rects: quads.iter().map(rect).collect(),
        })
        .collect()
}

fn view(text: &TextDoc, sel: Selection) -> PdfSelection {
    let quads = text.quads(&sel);
    PdfSelection {
        start: to_pos(sel.start),
        end: to_pos(sel.end),
        text: text.text(&sel).chars().take(MAX_PAGE_CHARS).collect(),
        marks: marks_of(quads.iter().map(|(page, q)| (*page, q.as_slice()))),
    }
}

#[cfg(test)]
mod tests {
    use kanso_pdf::model::Quad;
    use kanso_pdf::ocr::{OcrPage, PdfOcr};

    use super::ocr_words;

    #[test]
    fn scanned_words_use_the_shared_selectable_text_layout() {
        let ocr = PdfOcr {
            md5: "0123456789abcdef0123456789abcdef".to_owned(),
            engine: "vision".to_owned(),
            at: 0,
            pages: vec![OcrPage::new(
                2,
                612.0,
                792.0,
                [("scanned".to_owned(), Quad::new(0.1, 0.2, 0.5, 0.3))],
            )],
        };

        assert!(ocr_words(None, 2).is_none());
        assert!(ocr_words(Some(&ocr), 1).is_none());
        let page = ocr_words(Some(&ocr), 2).expect("the scanned page has OCR words");
        assert_eq!(page.page, 2);
        assert_eq!(page.width, 612.0);
        assert_eq!(page.height, 792.0);
        assert_eq!(
            page.chars.iter().map(|c| c.ch).collect::<String>(),
            "scanned"
        );
        assert!(page.chars[0].space_before);
    }
}
