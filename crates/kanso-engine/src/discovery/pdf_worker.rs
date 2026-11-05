//! Text in PDFs, read through the PDF worker (issue #43, step 7): the
//! [`PdfTextSource`] the desktop app gives [`index_next`]
//! (`super::pdf_text::index_next`).
//!
//! The PDF is handed to `kanso-pdf-worker`, a separate contained process
//! that holds PDFium (see `kanso_pdf::client`), which reads each page's text
//! in windows, within caps on a page, on a document and on time. This crate
//! links only the worker's client: no PDFium, so the app and the phones
//! build without it. A worker is started for each PDF and ended with it.
//!
//! [`WorkerScans`] draws scanned pages for the text reader (step 8).
//!
//! A failure is only blamed on the PDF when the worker is known to work:
//! a refusal about the document (a password, damage, too many pages) is the
//! PDF's. A crash is tried once more in a fresh worker. A crash, a hang or a
//! strange answer is the PDF's only if the worker then reads a small PDF of
//! our own. Anything else is the reader's, and nothing is kept, so a worker
//! that can't start, or a bad moment, never marks a PDF as having no text.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use kanso_pdf::client::{DOCUMENT_TEXT_DEADLINE, Worker, WorkerError, WorkerPaths};
use kanso_pdf::protocol::ErrorKind;

use super::pdf_ocr::{ScanDoc, ScanSource};
use super::pdf_text::{ExtractError, PdfTextSource};

/// What the store records as having read a PDF's text.
pub const ENGINE: &str = "pdfium";

/// How long the worker has to read [`known_pdf`] when asked whether it works.
const KNOWN_PDF_BUDGET: Duration = Duration::from_secs(10);

/// Reads the text of PDFs with the PDF worker.
#[derive(Clone, Debug)]
pub struct WorkerText {
    paths: WorkerPaths,
    budget: Duration,
    cancel: Arc<AtomicBool>,
}

impl WorkerText {
    /// A source that starts the worker at `paths`.
    pub fn new(paths: WorkerPaths) -> Self {
        WorkerText {
            paths,
            budget: DOCUMENT_TEXT_DEADLINE,
            cancel: Arc::default(),
        }
    }

    /// A source over the worker beside the running app, if there is one.
    pub fn locate() -> Option<Self> {
        kanso_pdf::client::locate().map(Self::new)
    }

    /// Reads for at most `budget` a PDF (the default is
    /// [`DOCUMENT_TEXT_DEADLINE`]).
    pub fn with_budget(mut self, budget: Duration) -> Self {
        self.budget = budget;
        self
    }

    /// Stops a read in progress, and later ones, when `cancel` is set: the
    /// worker is ended between two windows of pages.
    pub fn cancelled_by(mut self, cancel: Arc<AtomicBool>) -> Self {
        self.cancel = cancel;
        self
    }
}

impl PdfTextSource for WorkerText {
    fn engine(&self) -> &'static str {
        ENGINE
    }

    fn extract(&self, pdf: &[u8]) -> Result<Vec<String>, ExtractError> {
        if self.cancelled() {
            return Err(ExtractError::Reader("stopped".to_owned()));
        }
        let first = match self.read(pdf, self.budget) {
            // A crash may be the computer's (memory, a killed process), not
            // the PDF's: once more in a fresh worker.
            Err(WorkerError::Crashed) if !self.cancelled() => self.read(pdf, self.budget),
            other => other,
        };
        first.map_err(|error| self.blame(error))
    }
}

impl WorkerText {
    fn cancelled(&self) -> bool {
        self.cancel.load(Ordering::Relaxed)
    }

    /// The text of `pdf`'s pages, in a worker of its own.
    fn read(&self, pdf: &[u8], budget: Duration) -> Result<Vec<String>, WorkerError> {
        let mut worker = Worker::start(&self.paths)?;
        // No password is ever tried: an encrypted PDF has no text to index.
        worker.open(pdf, None)?;
        let text = worker.document_text(budget, &|| self.cancelled())?;
        worker.close();
        Ok(text.pages)
    }

    /// Whose failure `error` is: the PDF's, or the reader's.
    fn blame(&self, error: WorkerError) -> ExtractError {
        let words = error.to_string();
        match error {
            WorkerError::Unavailable(_) | WorkerError::Cancelled => ExtractError::Reader(words),
            // The worker read the document and said what is wrong with it.
            WorkerError::Refused {
                kind:
                    ErrorKind::Unreadable
                    | ErrorKind::PasswordRequired
                    | ErrorKind::WrongPassword
                    | ErrorKind::TooManyPages
                    | ErrorKind::PageTooLarge
                    | ErrorKind::FileTooLarge
                    | ErrorKind::NoSuchPage,
                ..
            } => ExtractError::Document(words),
            // A crash, a hang, an answer out of protocol, or PDFium failing
            // inside: the PDF's only if the worker reads a PDF that is fine.
            WorkerError::Crashed
            | WorkerError::TimedOut
            | WorkerError::BadReply
            | WorkerError::Refused { .. } => {
                if !self.cancelled() && self.works() {
                    ExtractError::Document(words)
                } else {
                    ExtractError::Reader(words)
                }
            }
        }
    }

    /// Whether the worker starts and reads the text of [`known_pdf`].
    fn works(&self) -> bool {
        self.read(&known_pdf(), KNOWN_PDF_BUDGET)
            .is_ok_and(|pages| pages.len() == 1)
    }
}

/// Draws the pages of PDFs with the PDF worker, as PNGs for a text reader
/// (issue #43, step 8): the [`ScanSource`] `ocr_next`
/// (`super::pdf_ocr::ocr_next`) reads scans through. Blame for a failure is
/// placed as for text: a crash is the PDF's only if the worker then reads a
/// PDF of our own.
#[derive(Clone, Debug)]
pub struct WorkerScans(WorkerText);

impl WorkerScans {
    /// A source over the worker beside the running app, if there is one.
    pub fn locate() -> Option<Self> {
        WorkerText::locate().map(Self)
    }

    /// A source that starts the worker at `paths`.
    pub fn new(paths: WorkerPaths) -> Self {
        Self(WorkerText::new(paths))
    }

    /// Ends a read in progress, and later ones, when `cancel` is set.
    pub fn cancelled_by(self, cancel: Arc<AtomicBool>) -> Self {
        Self(self.0.cancelled_by(cancel))
    }
}

impl ScanSource for WorkerScans {
    fn open(&self, pdf: &[u8]) -> Result<Box<dyn ScanDoc>, ExtractError> {
        if self.0.cancelled() {
            return Err(ExtractError::Reader("stopped".to_owned()));
        }
        let started = Worker::start(&self.0.paths).and_then(|mut worker| {
            // No password is ever tried: an encrypted PDF has no scans to read.
            let opened = worker.open(pdf, None)?;
            Ok((worker, opened))
        });
        match started {
            Ok((worker, opened)) => Ok(Box::new(WorkerScan {
                worker: Some(worker),
                sizes: opened.sizes,
                source: self.0.clone(),
            })),
            Err(error) => Err(self.0.blame(error)),
        }
    }
}

struct WorkerScan {
    worker: Option<Worker>,
    sizes: Vec<[f32; 2]>,
    source: WorkerText,
}

impl ScanDoc for WorkerScan {
    fn size(&self, page: u32) -> Option<[f32; 2]> {
        self.sizes.get(page.checked_sub(1)? as usize).copied()
    }

    fn render_png(&mut self, page: u32, width_px: u32) -> Result<Vec<u8>, ExtractError> {
        if self.source.cancelled() {
            return Err(ExtractError::Reader("stopped".to_owned()));
        }
        let Some(worker) = self.worker.as_mut() else {
            return Err(ExtractError::Reader("the PDF reader is closed".to_owned()));
        };
        match worker.render(page.saturating_sub(1), width_px) {
            Ok(page) => png_of(page)
                .ok_or_else(|| ExtractError::Document("a page that can't be drawn".to_owned())),
            Err(error) => Err(self.source.blame(error)),
        }
    }
}

impl Drop for WorkerScan {
    fn drop(&mut self) {
        if let Some(worker) = self.worker.take() {
            worker.close();
        }
    }
}

/// A drawn page (BGRA) as an opaque PNG. The pixels are turned to RGB in
/// place, so a page at the size cap (64 MB of BGRA) never has a second
/// copy beside it.
fn png_of(page: kanso_pdf::client::RenderedPage) -> Option<Vec<u8>> {
    use image::ImageEncoder as _;
    use image::codecs::png::{CompressionType, FilterType, PngEncoder};
    let kanso_pdf::client::RenderedPage {
        width,
        height,
        mut bgra,
        ..
    } = page;
    // The client already refuses a page whose pixels aren't its size; the
    // encoder would panic on one.
    let pixels = (width as usize).checked_mul(height as usize)?;
    if pixels.checked_mul(4) != Some(bgra.len()) {
        return None;
    }
    for i in 0..pixels {
        let (b, g, r) = (bgra[4 * i], bgra[4 * i + 1], bgra[4 * i + 2]);
        bgra[3 * i..3 * i + 3].copy_from_slice(&[r, g, b]);
    }
    bgra.truncate(pixels * 3);
    let rgb = bgra;
    let mut out = Vec::new();
    PngEncoder::new_with_quality(&mut out, CompressionType::Fast, FilterType::NoFilter)
        .write_image(&rgb, width, height, image::ExtendedColorType::Rgb8)
        .ok()?;
    Some(out)
}

/// A one-page PDF with a line of text, for asking whether the worker works.
fn known_pdf() -> Vec<u8> {
    let content = "BT /F1 12 Tf 10 30 Td (Kanso) Tj ET";
    let objects = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_owned(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_owned(),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 100 60] /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>".to_owned(),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_owned(),
        format!("<< /Length {} >>\nstream\n{content}\nendstream", content.len()),
    ];
    let mut out = b"%PDF-1.4\n".to_vec();
    let mut offsets = Vec::new();
    for (i, body) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n{body}\nendobj\n", i + 1).as_bytes());
    }
    let xref = out.len();
    out.extend_from_slice(
        format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes(),
    );
    for offset in offsets {
        out.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            objects.len() + 1
        )
        .as_bytes(),
    );
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nowhere() -> WorkerText {
        WorkerText::new(WorkerPaths {
            exe: "/no/such/kanso-pdf-worker".into(),
            library: None,
        })
    }

    /// A drawn page becomes a PNG of the same pixels, in their own colours,
    /// turned to RGB in place.
    #[test]
    fn a_drawn_page_is_its_own_pixels_as_a_png() {
        let (width, height) = (3u32, 2u32);
        let bgra: Vec<u8> = (0..width * height)
            .flat_map(|i| [i as u8, 100 + i as u8, 200 + i as u8, 255])
            .collect();
        let png = png_of(kanso_pdf::client::RenderedPage {
            page: 0,
            width,
            height,
            bgra,
        })
        .unwrap();
        let back = image::load_from_memory(&png).unwrap().to_rgb8();
        assert_eq!(back.dimensions(), (width, height));
        for (i, px) in back.pixels().enumerate() {
            assert_eq!(px.0, [200 + i as u8, 100 + i as u8, i as u8], "pixel {i}");
        }
        // Pixels that don't match the size are refused, not drawn.
        assert!(
            png_of(kanso_pdf::client::RenderedPage {
                page: 0,
                width: 4,
                height: 4,
                bgra: vec![0; 8],
            })
            .is_none()
        );
    }

    /// Without a worker to start, the reader failed, not the PDF: nothing
    /// is kept, so the PDF is read once there is a worker.
    #[test]
    fn a_missing_worker_is_the_readers_failure() {
        assert_eq!(nowhere().engine(), "pdfium");
        assert!(matches!(
            nowhere().extract(b"%PDF-1.7"),
            Err(ExtractError::Reader(_))
        ));
    }

    /// A source told to stop reads nothing, and never starts a worker.
    #[test]
    fn a_cancelled_source_does_not_start_the_worker() {
        let cancel = Arc::new(AtomicBool::new(true));
        let source = nowhere().cancelled_by(cancel);
        assert_eq!(
            source.extract(b"%PDF-1.7"),
            Err(ExtractError::Reader("stopped".to_owned()))
        );
    }

    /// The PDF used to ask whether the worker works is well formed: its
    /// cross-reference offsets point at its objects.
    #[test]
    fn the_known_pdf_is_well_formed() {
        let pdf = known_pdf();
        let text = String::from_utf8(pdf.clone()).unwrap();
        let xref = text.find("xref\n").unwrap();
        for (i, line) in text[xref..].lines().skip(3).take(5).enumerate() {
            let offset: usize = line[..10].parse().unwrap();
            assert!(
                text[offset..].starts_with(&format!("{} 0 obj", i + 1)),
                "{line}"
            );
        }
        assert!(text.ends_with("%%EOF\n"));
    }
}
