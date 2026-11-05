//! Text in PDFs: the words on each page of a PDF attachment, read on this
//! device as soon as the PDF arrives so search finds a note by what its PDFs
//! say (issue #43, step 7).
//!
//! Like text in images (`ocr.rs`) it is a background loop, one PDF at a time:
//! PDFs attached, imported or synced since, or whose bytes changed. Which PDF
//! is next, and keeping what was read, are the engine's
//! (`kanso_engine::discovery::pdf_text`); the reading is the PDF worker's
//! (`kanso-pdf-worker`, a separate contained process, reached through the
//! engine's `WorkerText`), so this process never opens a PDF. The words stay
//! on this device: PDFium reads them again wherever the PDF is.
//!
//! Only PDFs a note that may be read links are offered: not those only a
//! locked note, a note in the trash or a note tagged `#ai-exclude` links
//! (the store decides). Turning the setting off forgets every PDF's words;
//! turning it on reads them again.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use gpui_kit::{App, Global, Task};
use kanso_engine::discovery::pdf_text::{Indexed, PdfTextSource, index_next};
use kanso_engine::discovery::pdf_worker::WorkerText;
use kanso_store::Store;

use crate::workspace::AppState;

pub use kanso_engine::discovery::pdf_text::Status;

/// The pause between PDFs, so reading many never crowds the app.
const PDF_PAUSE: Duration = Duration::from_millis(250);
/// How long attachments stay still before new ones are read.
const SETTLE: Duration = Duration::from_millis(2000);
/// After the reader fails (the worker won't start, or keeps crashing), the
/// first wait before trying again; it doubles with each failure in a row,
/// up to [`RETRY_MAX`].
const RETRY_FIRST: Duration = Duration::from_secs(15);
const RETRY_MAX: Duration = Duration::from_secs(30 * 60);

/// How long to wait after `failures` reader failures in a row (1 or more).
fn retry_after(failures: u32) -> Duration {
    let doublings = failures.saturating_sub(1).min(16);
    RETRY_FIRST.saturating_mul(1 << doublings).min(RETRY_MAX)
}

/// Whether this build can read PDFs at all: the worker is beside the app.
pub fn supported() -> bool {
    kanso_pdf::client::locate().is_some()
}

#[derive(Default)]
pub struct PdfText {
    pub status: Status,
    task: Option<Task<()>>,
    cancel: Arc<AtomicBool>,
    /// Held while a PDF is read, so forgetting waits for a read in progress
    /// (see [`forget_all`]).
    gate: Arc<Mutex<()>>,
}

impl Global for PdfText {}

fn set_status(status: Status, cx: &mut App) {
    let state = cx.default_global::<PdfText>();
    if state.status != status {
        state.status = status;
        cx.refresh_windows();
    }
}

/// Starts reading at launch when it is on and this build can.
pub fn init(cx: &mut App) {
    cx.default_global::<PdfText>();
    if cx.global::<AppState>().config.discovery.pdf_text {
        start(cx);
    }
}

/// Turns reading on or off, from Settings. Off also forgets the words.
pub fn set_enabled(on: bool, cx: &mut App) {
    AppState::update_config(cx, |config| config.discovery.pdf_text = on);
    if on {
        start(cx);
        return;
    }
    let store = cx.global::<AppState>().store.clone();
    let state = cx.default_global::<PdfText>();
    state.cancel.store(true, Ordering::Relaxed);
    state.task = None;
    let gate = state.gate.clone();
    set_status(Status::Off, cx);
    cx.background_executor()
        .spawn(async move {
            if let Err(err) = forget_all(&store, &gate) {
                eprintln!("kanso: forgetting the text in PDFs failed: {err}");
            }
        })
        .detach();
}

/// Reads the next PDF with `source`, holding `gate` while it does.
fn read_one(
    store: &Store,
    source: &dyn PdfTextSource,
    gate: &Mutex<()>,
) -> kanso_store::Result<Indexed> {
    let _reading = gate.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    index_next(store, source)
}

/// Deletes the words of every PDF. It waits for a read in progress to end
/// first (that read was cancelled, and ends by keeping nothing for its PDF),
/// so no PDF is left marked as read once the words are gone.
fn forget_all(store: &Store, gate: &Mutex<()>) -> kanso_store::Result<()> {
    let _idle = gate.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    store.forget_all_pdf_text()
}

fn start(cx: &mut App) {
    let Some(source) = WorkerText::locate() else {
        return;
    };
    let store = cx.global::<AppState>().store.clone();
    let changes = store.subscribe();
    let state = cx.default_global::<PdfText>();
    state.cancel.store(true, Ordering::Relaxed);
    let cancel = Arc::new(AtomicBool::new(false));
    state.cancel = cancel.clone();
    let gate = state.gate.clone();
    let source = Arc::new(source.cancelled_by(cancel.clone()));
    let task = cx.spawn(async move |cx| {
        let mut failures = 0u32;
        loop {
            if cancel.load(Ordering::Relaxed) {
                return;
            }
            let (store2, source2, gate2) = (store.clone(), source.clone(), gate.clone());
            let waiting = cx
                .background_executor()
                .spawn(async move { read_one(&store2, &*source2, &gate2) })
                .await;
            if cancel.load(Ordering::Relaxed) {
                return;
            }
            match waiting {
                Ok(Indexed::Nothing) => {
                    failures = 0;
                    cx.update(|cx| set_status(Status::Idle, cx));
                    // Wait for a file to change, then for things to settle.
                    loop {
                        let Ok(change) = changes.recv().await else {
                            return;
                        };
                        // A new PDF, or a note that may now link one (or no
                        // longer be locked or excluded).
                        if (change.id.starts_with("file:") && !change.deleted)
                            || change.id.starts_with("note:")
                            || change.id.starts_with("tag:")
                        {
                            break;
                        }
                    }
                    cx.background_executor().timer(SETTLE).await;
                    while changes.try_recv().is_ok() {}
                }
                Ok(Indexed::Read { waiting }) => {
                    failures = 0;
                    cx.update(|cx| set_status(Status::Indexing { left: waiting - 1 }, cx));
                    cx.background_executor().timer(PDF_PAUSE).await;
                }
                // The worker failed, not the PDF: nothing was kept. Wait,
                // longer each time, and try the same PDF again.
                Ok(Indexed::ReaderFailed { waiting, error }) => {
                    failures = failures.saturating_add(1);
                    let wait = retry_after(failures);
                    eprintln!(
                        "kanso: the PDF reader failed ({error}); trying again in {} s",
                        wait.as_secs()
                    );
                    cx.update(|cx| set_status(Status::Waiting { left: waiting }, cx));
                    cx.background_executor().timer(wait).await;
                }
                Err(err) => {
                    eprintln!("kanso: reading text in PDFs stopped: {err}");
                    cx.update(|cx| set_status(Status::Idle, cx));
                    return;
                }
            }
        }
    });
    let state = cx.default_global::<PdfText>();
    state.task = Some(task);
    if state.status == Status::Off {
        state.status = Status::Idle;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kanso_core::{DocType, Query};
    use kanso_engine::attach::{attach_pdf, file_markdown};
    use kanso_engine::discovery::pdf_text::ExtractError;
    use kanso_store::{Origin, Put, SearchParams};
    use proptest::prelude::*;
    use serde_json::json;
    use std::sync::atomic::AtomicUsize;

    /// Says each PDF's pages as the text after its `%PDF-` header split on
    /// `|`, and counts.
    struct Fake(AtomicUsize);

    impl PdfTextSource for Fake {
        fn engine(&self) -> &'static str {
            "fake"
        }
        fn extract(&self, pdf: &[u8]) -> Result<Vec<String>, ExtractError> {
            self.0.fetch_add(1, Ordering::Relaxed);
            let text = String::from_utf8_lossy(&pdf[5..]).into_owned();
            Ok(text.split('|').map(str::to_owned).collect())
        }
    }

    fn store() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("k.db")).unwrap();
        (dir, store)
    }

    /// A note that links a new PDF saying `words`.
    fn note_with_pdf(store: &Store, book: &str, words: &str) -> String {
        let file = attach_pdf(store, format!("%PDF-{words}").as_bytes(), "a.pdf", None).unwrap();
        store
            .put(
                DocType::Note,
                json!({"title": "Shows", "body": file_markdown(&file, "a.pdf"), "bookId": book}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id
    }

    fn book(store: &Store) -> String {
        store
            .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
            .unwrap()
            .id
    }

    fn found(store: &Store, word: &str) -> usize {
        store
            .search_summaries(&SearchParams {
                query: Query::parse(word),
                ocr: true,
                ..Default::default()
            })
            .unwrap()
            .len()
    }

    #[test]
    fn the_switch_is_off_until_a_worker_is_beside_the_app() {
        // Under test there is no worker beside the test binary, and the
        // default is on: the Settings row says why nothing is read.
        assert!(!supported());
        assert!(crate::config::DiscoverySettings::default().pdf_text);
    }

    /// A config written before the switch existed reads as on.
    #[test]
    fn an_older_config_has_pdf_search_on() {
        let old: crate::config::DiscoverySettings =
            serde_json::from_value(json!({"relatedNotes": true, "imageText": false})).unwrap();
        assert!(old.pdf_text && old.related_notes && !old.image_text);
        let back = serde_json::to_value(old).unwrap();
        assert_eq!(back["pdfText"], json!(true));
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: 12, ..ProptestConfig::default() })]

        /// However many PDFs, reading them one by one counts down what is
        /// left, finds them in search, and forgetting takes every word out
        /// of search and leaves them all waiting to be read again.
        #[test]
        fn prop_reading_then_forgetting_round_trips(count in 0usize..5) {
            let (_dir, store) = store();
            let book = book(&store);
            for n in 0..count {
                note_with_pdf(&store, &book, &format!("wordnumber{n}"));
            }
            let (source, gate) = (Fake(AtomicUsize::new(0)), Mutex::new(()));
            for left in (1..=count).rev() {
                prop_assert_eq!(read_one(&store, &source, &gate).unwrap(), Indexed::Read { waiting: left });
            }
            prop_assert_eq!(read_one(&store, &source, &gate).unwrap(), Indexed::Nothing);
            prop_assert_eq!(source.0.load(Ordering::Relaxed), count);
            for n in 0..count {
                prop_assert_eq!(found(&store, &format!("wordnumber{n}")), 1);
            }
            forget_all(&store, &gate).unwrap();
            for n in 0..count {
                prop_assert_eq!(found(&store, &format!("wordnumber{n}")), 0);
            }
            for left in (1..=count).rev() {
                prop_assert_eq!(read_one(&store, &source, &gate).unwrap(), Indexed::Read { waiting: left });
            }
        }

        /// The wait after a reader failure grows with each failure in a row,
        /// starts short, and never passes the cap.
        #[test]
        fn prop_the_retry_wait_grows_to_a_cap(failures in 1u32..200) {
            let wait = retry_after(failures);
            prop_assert!(wait >= RETRY_FIRST && wait <= RETRY_MAX);
            prop_assert!(retry_after(failures + 1) >= wait);
        }
    }

    #[test]
    fn the_first_retry_is_soon_and_later_ones_wait_long() {
        assert_eq!(retry_after(1), RETRY_FIRST);
        assert_eq!(retry_after(2), RETRY_FIRST * 2);
        assert_eq!(retry_after(u32::MAX), RETRY_MAX);
    }

    /// A reader that fails leaves every PDF waiting and nothing in search;
    /// once it works, they are read.
    #[test]
    fn a_failed_reader_leaves_pdfs_waiting() {
        struct Broken;
        impl PdfTextSource for Broken {
            fn engine(&self) -> &'static str {
                "fake"
            }
            fn extract(&self, _: &[u8]) -> Result<Vec<String>, ExtractError> {
                Err(ExtractError::Reader("won't start".into()))
            }
        }
        let (_dir, store) = store();
        let book = book(&store);
        note_with_pdf(&store, &book, "laterword");
        let gate = Mutex::new(());
        for _ in 0..3 {
            assert!(matches!(
                read_one(&store, &Broken, &gate).unwrap(),
                Indexed::ReaderFailed { waiting: 1, .. }
            ));
        }
        assert_eq!(found(&store, "laterword"), 0);
        let source = Fake(AtomicUsize::new(0));
        assert_eq!(
            read_one(&store, &source, &gate).unwrap(),
            Indexed::Read { waiting: 1 }
        );
        assert_eq!(found(&store, "laterword"), 1);
    }

    /// Forgetting waits for a read in progress, so the read cannot leave a
    /// PDF marked as read after the words are gone.
    #[test]
    fn forgetting_waits_for_a_read_in_progress() {
        let (_dir, store) = store();
        let book = book(&store);
        note_with_pdf(&store, &book, "slowwords");
        let store = Arc::new(store);
        let gate = Arc::new(Mutex::new(()));
        let reading = gate.lock().unwrap();
        let forgot = Arc::new(AtomicBool::new(false));
        let thread = {
            let (store, gate, forgot) = (store.clone(), gate.clone(), forgot.clone());
            std::thread::spawn(move || {
                forget_all(&store, &gate).unwrap();
                forgot.store(true, Ordering::Relaxed);
            })
        };
        std::thread::sleep(Duration::from_millis(150));
        assert!(
            !forgot.load(Ordering::Relaxed),
            "forgot while a PDF was being read"
        );
        drop(reading);
        thread.join().unwrap();
        assert!(forgot.load(Ordering::Relaxed));
    }
}
