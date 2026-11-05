//! Scanned pages of PDFs, read in the background (issue #43, step 8).
//!
//! A scanned page has no text layer, so [`ocr_next`] draws it (through the
//! PDF worker, behind [`ScanSource`]) at about 250 dpi and hands the picture
//! to the platform's reader ([`WordReader`]: Apple's Vision on a Mac), which
//! answers with words and where they are. The words of each page are kept in
//! the store as they come (`kanso-store`'s `pdf_ocr.rs`), so reading can be
//! paused or stopped and go on where it was; when the last page is read the
//! whole set is written once into the file's `kansoPdfOcr`, which syncs, so
//! a device that can't read scans still finds, selects and highlights the
//! words a Mac read.
//!
//! The work is done in small chunks of pages so the app can show progress
//! ("Reading 40 of 300 pages"), and pause between two chunks. Only PDFs
//! whose text was extracted first are offered (that is how blank pages are
//! known), and only those a note that may be read links (the store decides).

use kanso_pdf::OcrPage;
use kanso_pdf::Quad;
use kanso_pdf::ocr::read_width_px;
use kanso_store::Store;

use super::pdf_text::ExtractError;

/// A reader of words in a picture. Apple's Vision implements it on a Mac.
pub trait WordReader: Send + Sync {
    /// Names the reader in the file's `kansoPdfOcr`.
    fn engine(&self) -> &'static str;
    /// The words in a PNG, each with its box on the picture, normalised to
    /// it with the origin at the top-left, in reading order. A picture with
    /// no words gives none.
    fn read_words(&self, png: &[u8]) -> Result<Vec<(String, Quad)>, String>;
}

/// Draws pages of a PDF as pictures for a [`WordReader`].
pub trait ScanSource: Send + Sync {
    /// Opens `pdf`. The error says whether the PDF or the reader is to
    /// blame, as for text (`ExtractError`).
    fn open(&self, pdf: &[u8]) -> Result<Box<dyn ScanDoc>, ExtractError>;
}

/// One open PDF.
pub trait ScanDoc {
    /// A page's size in points as shown; `page` is 1-based.
    fn size(&self, page: u32) -> Option<[f32; 2]>;
    /// The page as a PNG `width_px` wide.
    fn render_png(&mut self, page: u32, width_px: u32) -> Result<Vec<u8>, ExtractError>;
}

/// Where reading scanned pages stands.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Status {
    #[default]
    Off,
    /// Reading a PDF: `read` of its `scanned` pages are done, and `waiting`
    /// PDFs are waiting, this one included.
    Reading {
        read: u32,
        scanned: u32,
        waiting: usize,
    },
    /// Stopped by the person; `waiting` PDFs wait.
    Paused { waiting: usize },
    /// Every scanned page is read.
    Idle,
    /// The reader failed; `waiting` PDFs wait for it to be tried again.
    Waiting { waiting: usize },
}

/// What one [`ocr_next`] did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ocred {
    /// No PDF is waiting.
    Nothing,
    /// Pages were read. `waiting` PDFs were waiting, this one included;
    /// `read` of its `scanned` pages are done; `finished` says the PDF is
    /// done and its words are written.
    Read {
        waiting: usize,
        read: u32,
        scanned: u32,
        finished: bool,
    },
    /// The stop was asked for before a page was read.
    Stopped { waiting: usize },
    /// The reader failed; what was read stays kept and `waiting` PDFs still
    /// wait. Try again later.
    ReaderFailed { waiting: usize, error: String },
}

/// How many PDFs are asked of the store at once.
const LOOK_AHEAD: usize = 64;

/// Reads up to `chunk` scanned pages of the next PDF that has some, with
/// `scans` and `reader`, keeping each page's words as it goes. `stop` is
/// asked before each page. A PDF that can't be read (damaged, a password) is
/// kept as having no words, so it isn't tried again until its bytes change.
pub fn ocr_next(
    store: &Store,
    scans: &dyn ScanSource,
    reader: &dyn WordReader,
    chunk: usize,
    stop: &dyn Fn() -> bool,
) -> kanso_store::Result<Ocred> {
    let waiting_pdfs = store.pdfs_to_ocr(LOOK_AHEAD)?;
    let waiting = waiting_pdfs.len();
    let Some(pdf) = waiting_pdfs.into_iter().next() else {
        return Ok(Ocred::Nothing);
    };
    let pages = store.pdf_pages_to_ocr(&pdf.id, &pdf.md5)?;
    let state = |store: &Store, finished: bool| -> kanso_store::Result<Ocred> {
        let progress = store.pdf_ocr_progress(&pdf.id)?;
        Ok(Ocred::Read {
            waiting,
            // Once the set is written the kept pages are cleared: all read.
            read: if finished {
                progress.scanned
            } else {
                progress.read.min(progress.scanned)
            },
            scanned: progress.scanned,
            finished,
        })
    };
    let finish = |store: &Store| store.finish_pdf_ocr(&pdf.id, &pdf.md5, reader.engine());
    if pages.is_empty() {
        // Every page is read: write the set.
        finish(store)?;
        return state(store, true);
    }
    if stop() {
        return Ok(Ocred::Stopped { waiting });
    }
    let Some((_, attachment)) = store.attachment(&pdf.id)? else {
        // Its bytes are gone: nothing to read.
        for page in &pages {
            store.keep_pdf_ocr_page(&pdf.id, &pdf.md5, &OcrPage::new(*page, 1.0, 1.0, []))?;
        }
        finish(store)?;
        return state(store, true);
    };
    let mut doc = match scans.open(&attachment.data) {
        Ok(doc) => doc,
        Err(ExtractError::Reader(error)) => return Ok(Ocred::ReaderFailed { waiting, error }),
        Err(ExtractError::Document(err)) => {
            eprintln!("kanso: couldn't read the scans in {}: {err}", pdf.id);
            for page in &pages {
                store.keep_pdf_ocr_page(&pdf.id, &pdf.md5, &OcrPage::new(*page, 1.0, 1.0, []))?;
            }
            finish(store)?;
            return state(store, true);
        }
    };
    drop(attachment);
    let mut done = 0usize;
    for &page in pages.iter().take(chunk.max(1)) {
        if stop() {
            if done == 0 {
                return Ok(Ocred::Stopped { waiting });
            }
            break;
        }
        let [w, h] = doc.size(page).unwrap_or([1.0, 1.0]);
        let png = match doc.render_png(page, read_width_px(w)) {
            Ok(png) => Some(png),
            Err(ExtractError::Reader(error)) => {
                return Ok(Ocred::ReaderFailed { waiting, error });
            }
            Err(ExtractError::Document(err)) => {
                eprintln!("kanso: couldn't draw page {page} of {}: {err}", pdf.id);
                None
            }
        };
        let words = png.map_or_else(Vec::new, |png| {
            reader.read_words(&png).unwrap_or_else(|err| {
                eprintln!("kanso: couldn't read page {page} of {}: {err}", pdf.id);
                Vec::new()
            })
        });
        let kept = store.keep_pdf_ocr_page(
            &pdf.id,
            &pdf.md5,
            &OcrPage::new(page, f64::from(w), f64::from(h), words),
        )?;
        if !kept {
            // The file changed or was locked: its pages are no longer ours.
            return state(store, true);
        }
        done += 1;
    }
    let finished = pages.len() <= done;
    if finished {
        finish(store)?;
    }
    state(store, finished)
}

#[cfg(test)]
mod tests;
