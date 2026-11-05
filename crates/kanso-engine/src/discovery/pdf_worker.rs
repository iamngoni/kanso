//! Text in PDFs, read through the PDF worker (issue #43, step 7): the
//! [`PdfTextSource`] the desktop app gives [`index_next`]
//! (`super::pdf_text::index_next`).
//!
//! The PDF is handed to `kanso-pdf-worker`, a separate contained process
//! that holds PDFium (see `kanso_pdf::client`), which reads each page's text
//! in windows, within caps on a page, on a document and on time. This crate
//! links only the worker's client: no PDFium, so the app and the phones
//! build without it. A worker is started for each PDF and ended with it.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use kanso_pdf::client::{DOCUMENT_TEXT_DEADLINE, Worker, WorkerPaths};

use super::pdf_text::PdfTextSource;

/// What the store records as having read a PDF's text.
pub const ENGINE: &str = "pdfium";

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

    fn extract(&self, pdf: &[u8]) -> Result<Vec<String>, String> {
        let cancelled = || self.cancel.load(Ordering::Relaxed);
        if cancelled() {
            return Err("stopped".to_owned());
        }
        let mut worker = Worker::start(&self.paths).map_err(|e| e.to_string())?;
        // No password is ever tried: an encrypted PDF has no text to index.
        worker.open(pdf, None).map_err(|e| e.message())?;
        let text = worker
            .document_text(self.budget, &cancelled)
            .map_err(|e| e.message())?;
        worker.close();
        Ok(text.pages)
    }
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

    /// Without a worker to start, a PDF is unreadable, not a panic.
    #[test]
    fn a_missing_worker_is_an_error_not_a_panic() {
        assert_eq!(nowhere().engine(), "pdfium");
        assert!(nowhere().extract(b"%PDF-1.7").is_err());
    }

    /// A source told to stop reads nothing, and never starts a worker.
    #[test]
    fn a_cancelled_source_does_not_start_the_worker() {
        let cancel = Arc::new(AtomicBool::new(true));
        let source = nowhere().cancelled_by(cancel);
        assert_eq!(source.extract(b"%PDF-1.7"), Err("stopped".to_owned()));
    }
}
