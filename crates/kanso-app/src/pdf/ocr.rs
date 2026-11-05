//! Scanned PDF pages, read in the background (issue #43, step 8).
//!
//! A scanned page has no text layer. Where this device has a reader of text
//! in pictures (Apple's Vision on a Mac), this loop draws each scanned page
//! through the PDF worker at about 250 dpi and reads its words with their
//! places, a few pages at a time, so Settings can say "Reading 40 of 300
//! pages" and the person can pause. Which PDF is next, and keeping what was
//! read, are the engine's (`kanso_engine::discovery::pdf_ocr`); the words
//! are written into the file once its last scanned page is read, and sync
//! with it, so Windows and Linux, which have no reader, search, select and
//! highlight words a Mac read.
//!
//! It runs when both "Search text in PDFs" and "Search text in images" are
//! on, since it is both: the words in a picture, in a PDF. Only PDFs whose
//! text was extracted (so blank pages are known) and that a note that may
//! be read links are offered (the store decides). Reading is at low priority
//! (a pause between chunks), and a pause is kept until it is lifted or the
//! app quits.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use gpui_kit::{App, Global, Task};
use kanso_engine::discovery::pdf_ocr::{Ocred, ScanSource, WordReader, ocr_next};
use kanso_engine::discovery::pdf_worker::WorkerScans;
use kanso_store::Store;

use super::index::retry_after;
use crate::workspace::AppState;

pub use kanso_engine::discovery::pdf_ocr::Status;

/// Pages read before the status is told and a pause is looked at.
const CHUNK: usize = 4;
/// The pause between chunks, so reading never crowds the app.
const CHUNK_PAUSE: Duration = Duration::from_millis(200);
/// How often the loop looks again when nothing was waiting: the text of a
/// new PDF is extracted first, in another loop, which does not tell this one.
const LOOK_AGAIN: Duration = Duration::from_secs(20);
/// How often a paused loop looks at the switch.
const PAUSED_LOOK: Duration = Duration::from_millis(500);
/// How often a paused loop counts the PDFs still waiting, for Settings.
const PAUSED_RECOUNT: Duration = Duration::from_secs(30);

/// Whether this device can read scanned pages: a text reader and the PDF
/// worker are both here.
pub fn supported() -> bool {
    #[cfg(test)]
    if SHOWN_AS_SUPPORTED.load(Ordering::Relaxed) {
        return true;
    }
    crate::ocr::supported() && super::index::supported()
}

/// Snapshots draw the row of a device that can read scans, with no worker
/// beside the test binary.
#[cfg(test)]
pub(crate) static SHOWN_AS_SUPPORTED: AtomicBool = AtomicBool::new(false);

#[derive(Default)]
pub struct ScannedPages {
    pub status: Status,
    task: Option<Task<()>>,
    cancel: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
}

impl Global for ScannedPages {}

impl ScannedPages {
    /// Whether the person paused reading.
    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::Relaxed)
    }
}

fn set_status(status: Status, cx: &mut App) {
    let state = cx.default_global::<ScannedPages>();
    if state.status != status {
        state.status = status;
        cx.refresh_windows();
    }
}

/// Starts reading at launch when it is on and this device can.
pub fn init(cx: &mut App) {
    cx.default_global::<ScannedPages>();
    refresh(cx);
}

/// Brings the loop in line with the settings: running when PDF search and
/// image text are both on and this device can read, stopped otherwise.
/// Called at launch and when either switch changes.
pub fn refresh(cx: &mut App) {
    let discovery = cx.global::<AppState>().config.discovery;
    if discovery.pdf_text && discovery.image_text && supported() {
        start(cx);
    } else {
        let state = cx.default_global::<ScannedPages>();
        state.cancel.store(true, Ordering::Relaxed);
        state.task = None;
        set_status(Status::Off, cx);
    }
}

/// Pauses or resumes reading, from Settings.
pub fn set_paused(paused: bool, cx: &mut App) {
    let state = cx.default_global::<ScannedPages>();
    state.paused.store(paused, Ordering::Relaxed);
    cx.refresh_windows();
}

/// What Settings says about where reading stands: nothing when there is
/// nothing to say.
pub fn words(status: &Status) -> Option<String> {
    let count = crate::text_stats::format_count;
    match status {
        Status::Reading {
            read,
            scanned,
            waiting,
        } if *scanned == 0 => Some(format!("Reading scanned pages… {} to go.", pdfs(*waiting))),
        Status::Reading {
            read,
            scanned,
            waiting,
        } => Some(format!(
            "Reading {} of {} pages{}",
            count((*read as usize + 1).min(*scanned as usize)),
            count(*scanned as usize),
            if *waiting > 1 {
                format!("; {} to go.", pdfs(waiting - 1))
            } else {
                ".".to_owned()
            }
        )),
        Status::Paused { waiting } => Some(format!("Paused. {} to go.", pdfs(*waiting))),
        Status::Waiting { waiting } => Some(format!(
            "Kanso's PDF reader isn't working right now. It will try again soon; {} to go.",
            pdfs(*waiting)
        )),
        Status::Off | Status::Idle => None,
    }
}

fn pdfs(n: usize) -> String {
    let count = crate::text_stats::format_count(n);
    if n == 1 {
        format!("{count} PDF")
    } else {
        format!("{count} PDFs")
    }
}

/// Reads one chunk. `stop` says whether to stop between pages.
fn read_chunk(
    store: &Store,
    scans: &dyn ScanSource,
    reader: &dyn WordReader,
    stop: &dyn Fn() -> bool,
) -> kanso_store::Result<Ocred> {
    ocr_next(store, scans, reader, CHUNK, stop)
}

fn start(cx: &mut App) {
    let (Some(scans), Some(reader)) = (WorkerScans::locate(), crate::ocr::word_reader()) else {
        return;
    };
    let store = cx.global::<AppState>().store.clone();
    let state = cx.default_global::<ScannedPages>();
    state.cancel.store(true, Ordering::Relaxed);
    let cancel = Arc::new(AtomicBool::new(false));
    state.cancel = cancel.clone();
    let paused = state.paused.clone();
    let scans = Arc::new(scans.cancelled_by(cancel.clone()));
    let task = cx.spawn(async move |cx| {
        let mut failures = 0u32;
        // When the PDFs waiting were last counted while paused.
        let mut counted: Option<Instant> = None;
        loop {
            if cancel.load(Ordering::Relaxed) {
                return;
            }
            if paused.load(Ordering::Relaxed) {
                // Counted when the pause starts and now and then after, off
                // the main thread; the switch itself is looked at often.
                if counted.is_none_or(|at| at.elapsed() >= PAUSED_RECOUNT) {
                    let store2 = store.clone();
                    let waiting = cx
                        .background_executor()
                        .spawn(async move {
                            store2.pdfs_to_ocr(usize::MAX).map(|w| w.len()).unwrap_or(0)
                        })
                        .await;
                    counted = Some(Instant::now());
                    cx.update(|cx| {
                        set_status(
                            if waiting == 0 {
                                Status::Idle
                            } else {
                                Status::Paused { waiting }
                            },
                            cx,
                        )
                    });
                }
                cx.background_executor().timer(PAUSED_LOOK).await;
                continue;
            }
            counted = None;
            let (store2, scans2, reader2) = (store.clone(), scans.clone(), reader.clone());
            let (cancel2, paused2) = (cancel.clone(), paused.clone());
            let step = cx
                .background_executor()
                .spawn(async move {
                    let stop =
                        || cancel2.load(Ordering::Relaxed) || paused2.load(Ordering::Relaxed);
                    read_chunk(&store2, &*scans2, &*reader2, &stop)
                })
                .await;
            if cancel.load(Ordering::Relaxed) {
                return;
            }
            match step {
                Ok(Ocred::Nothing) => {
                    failures = 0;
                    cx.update(|cx| set_status(Status::Idle, cx));
                    cx.background_executor().timer(LOOK_AGAIN).await;
                }
                Ok(Ocred::Read {
                    waiting,
                    read,
                    scanned,
                    finished,
                }) => {
                    failures = 0;
                    cx.update(|cx| {
                        set_status(
                            if finished && waiting <= 1 {
                                Status::Idle
                            } else {
                                Status::Reading {
                                    read: if finished { 0 } else { read },
                                    scanned: if finished { 0 } else { scanned },
                                    waiting: if finished { waiting - 1 } else { waiting },
                                }
                            },
                            cx,
                        )
                    });
                    cx.background_executor().timer(CHUNK_PAUSE).await;
                }
                Ok(Ocred::Stopped { waiting }) => {
                    cx.update(|cx| set_status(Status::Paused { waiting }, cx));
                }
                Ok(Ocred::ReaderFailed { waiting, error }) => {
                    failures = failures.saturating_add(1);
                    let wait = retry_after(failures);
                    eprintln!(
                        "kanso: the PDF reader failed on a scan ({error}); trying again in {} s",
                        wait.as_secs()
                    );
                    cx.update(|cx| set_status(Status::Waiting { waiting }, cx));
                    cx.background_executor().timer(wait).await;
                }
                Err(err) => {
                    eprintln!("kanso: reading scanned PDF pages stopped: {err}");
                    cx.update(|cx| set_status(Status::Idle, cx));
                    return;
                }
            }
        }
    });
    let state = cx.default_global::<ScannedPages>();
    state.task = Some(task);
    if state.status == Status::Off {
        state.status = Status::Idle;
    }
}

#[cfg(test)]
mod tests;
