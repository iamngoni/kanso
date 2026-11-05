//! One open PDF's worker, driven from a background thread (issue #43, step 4).
//!
//! The reader never blocks on the helper process. A [`Session`] owns a thread
//! that reads the attachment, starts `kanso-pdf-worker`, opens the document
//! (asking the reader for a password when it needs one), and then draws the
//! pages it is told to, newest request first. The reader sends the whole list
//! of pages it wants with [`Session::set_jobs`], which replaces whatever was
//! still waiting, so a page that scrolled away is never drawn; only the page
//! being drawn at that moment finishes.
//!
//! A password is handed over once, used for the open, and dropped. Nothing
//! here keeps it, and it is not part of any event or of `Debug`.

use crate::attachments::FileKeys;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Condvar, Mutex};

use gpui_kit::RenderImage;
use kanso_pdf::client::{Worker, WorkerError, WorkerPaths};
use kanso_pdf::protocol::ErrorKind;
use kanso_store::Store;

use super::model::{NOT_READABLE, UNAVAILABLE};
use super::texture;

/// A page to draw: a page of the column, or its thumbnail.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Job {
    pub thumb: bool,
    /// 0-based.
    pub page: usize,
    pub width_px: u32,
    /// Drawn for a dark reader (smart invert).
    pub invert: bool,
}

/// What the thread tells the reader.
pub enum Event {
    /// The document is open. Sizes are in points, in page order.
    Opened { sizes: Vec<[f32; 2]> },
    /// The document needs a password (`wrong`: the one given was not it).
    NeedsPassword { wrong: bool },
    /// A page drawn, as a texture.
    Rendered {
        job: Job,
        image: Arc<RenderImage>,
        /// The texture's size in bytes, for the budget.
        bytes: usize,
    },
    /// One page could not be drawn; the worker is still up.
    RenderFailed { job: Job },
    /// The characters of a page, for selecting and highlighting (1-based
    /// page number inside).
    Text(Box<kanso_pdf::PageText>),
    /// The worker is gone or the document cannot be shown; the words are for
    /// the person reading.
    Failed(String),
}

#[derive(Default)]
struct Inbox {
    jobs: Vec<Job>,
    /// Pages (0-based) whose characters are wanted, most wanted first.
    text: Vec<u32>,
    running: Option<Job>,
    password: Option<String>,
    closed: bool,
}

struct Shared {
    inbox: Mutex<Inbox>,
    wake: Condvar,
    pid: AtomicU32,
}

/// The reader's end of the worker thread.
pub struct Session {
    shared: Arc<Shared>,
}

impl std::fmt::Debug for Session {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Session")
    }
}

impl Session {
    /// Starts the thread for `file_id`. Events come on the channel.
    pub fn start(
        store: Arc<Store>,
        keys: FileKeys,
        file_id: String,
        paths: Option<WorkerPaths>,
    ) -> (Session, async_channel::Receiver<Event>) {
        let (tx, rx) = async_channel::unbounded();
        // Nothing to start: say so at once, without a thread.
        if paths.is_none() {
            tx.try_send(Event::Failed(UNAVAILABLE.to_owned())).ok();
            let shared = Arc::new(Shared {
                inbox: Mutex::new(Inbox::default()),
                wake: Condvar::new(),
                pid: AtomicU32::new(0),
            });
            return (Session { shared }, rx);
        }
        let shared = Arc::new(Shared {
            inbox: Mutex::new(Inbox::default()),
            wake: Condvar::new(),
            pid: AtomicU32::new(0),
        });
        let thread_shared = shared.clone();
        let spawned = std::thread::Builder::new()
            .name("pdf-session".into())
            .spawn(move || run(&thread_shared, &store, &keys, &file_id, paths.as_ref(), &tx));
        if spawned.is_err() {
            // Nothing to draw with; the reader will say so on the next call.
            shared.inbox.lock().map(|mut i| i.closed = true).ok();
        }
        (Session { shared }, rx)
    }

    /// The pages to draw, most wanted first, instead of any still waiting.
    pub fn set_jobs(&self, jobs: Vec<Job>) {
        if let Ok(mut inbox) = self.shared.inbox.lock() {
            let running = inbox.running;
            inbox.jobs = jobs.into_iter().filter(|j| Some(*j) != running).collect();
            self.shared.wake.notify_all();
        }
    }

    /// More pages (0-based) whose text the reader wants, most wanted first.
    pub fn want_text(&self, pages: Vec<u32>) {
        if let Ok(mut inbox) = self.shared.inbox.lock() {
            for page in pages {
                if !inbox.text.contains(&page) {
                    inbox.text.push(page);
                }
            }
            self.shared.wake.notify_all();
        }
    }

    /// The password the person typed. Used for the next open attempt only.
    pub fn submit_password(&self, password: String) {
        if let Ok(mut inbox) = self.shared.inbox.lock() {
            inbox.password = Some(password);
            self.shared.wake.notify_all();
        }
    }

    /// The worker's process id, for the test that kills it.
    #[cfg(test)]
    pub fn pid(&self) -> Option<u32> {
        match self.shared.pid.load(Ordering::SeqCst) {
            0 => None,
            pid => Some(pid),
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        if let Ok(mut inbox) = self.shared.inbox.lock() {
            inbox.closed = true;
            inbox.password = None;
            self.shared.wake.notify_all();
        }
    }
}

fn run(
    shared: &Shared,
    store: &Store,
    keys: &FileKeys,
    file_id: &str,
    paths: Option<&WorkerPaths>,
    tx: &async_channel::Sender<Event>,
) {
    let send = |event: Event| tx.send_blocking(event).is_ok();
    let Some(paths) = paths else {
        send(Event::Failed(UNAVAILABLE.to_owned()));
        return;
    };
    // A sealed PDF opens only with the file key this session holds for it.
    let bytes = match crate::attachments::open_with(store, file_id, keys) {
        crate::attachments::Opened::Opened(file) if file.content_type == "application/pdf" => {
            file.bytes
        }
        _ => {
            send(Event::Failed(NOT_READABLE.to_owned()));
            return;
        }
    };

    let mut password: Option<String> = None;
    let (mut worker, sizes) = loop {
        let mut worker = match Worker::start(paths) {
            Ok(worker) => worker,
            Err(error) => {
                send(Event::Failed(error.message()));
                return;
            }
        };
        shared
            .pid
            .store(worker.pid().unwrap_or(0), Ordering::SeqCst);
        // The password is used for this one open and dropped with it.
        let attempt = worker.open(&bytes, password.take().as_deref());
        match attempt {
            Ok(opened) => break (worker, opened.sizes),
            Err(WorkerError::Refused {
                kind: kind @ (ErrorKind::PasswordRequired | ErrorKind::WrongPassword),
                ..
            }) => {
                worker.close();
                if !send(Event::NeedsPassword {
                    wrong: kind == ErrorKind::WrongPassword,
                }) {
                    return;
                }
                match wait_for_password(shared) {
                    Some(given) => password = Some(given),
                    None => return,
                }
            }
            Err(error) => {
                send(Event::Failed(error.message()));
                return;
            }
        }
    };
    drop(bytes);
    if !send(Event::Opened { sizes }) {
        worker.close();
        return;
    }

    // The words read from the PDF's scanned pages, once a blank page asks.
    let mut scans: Option<Option<kanso_pdf::PdfOcr>> = None;
    loop {
        let job = {
            let Ok(mut inbox) = shared.inbox.lock() else {
                break;
            };
            loop {
                if inbox.closed {
                    drop(inbox);
                    worker.close();
                    return;
                }
                // Text is small and quick, and a selection waits on it.
                if !inbox.text.is_empty() {
                    let page = inbox.text.remove(0);
                    drop(inbox);
                    match worker.text(page) {
                        Ok(text) => {
                            // A scan has no characters: its words come from
                            // the pictures that were read (here or by
                            // another device), at word granularity.
                            let text = if text.chars.is_empty() {
                                // The file's words are read once a session,
                                // not once a page.
                                let ocr = scans
                                    .get_or_insert_with(|| store.pdf_ocr(file_id).ok().flatten());
                                ocr.as_ref()
                                    .and_then(|ocr| scanned_page(ocr, &text))
                                    .unwrap_or(text)
                            } else {
                                text
                            };
                            if !send(Event::Text(Box::new(text))) {
                                worker.close();
                                return;
                            }
                        }
                        Err(error) => {
                            if worker.failure().is_some() {
                                send(Event::Failed(error.message()));
                                return;
                            }
                        }
                    }
                    let Ok(again) = shared.inbox.lock() else {
                        return;
                    };
                    inbox = again;
                    continue;
                }
                if !inbox.jobs.is_empty() {
                    let job = inbox.jobs.remove(0);
                    inbox.running = Some(job);
                    break job;
                }
                inbox = match shared.wake.wait(inbox) {
                    Ok(inbox) => inbox,
                    Err(_) => return,
                };
            }
        };
        let drawn = worker.render_as(job.page as u32, job.width_px, job.invert);
        if let Ok(mut inbox) = shared.inbox.lock() {
            inbox.running = None;
        }
        let event = match drawn {
            Ok(page) => {
                let bytes = page.bgra.len();
                match texture::texture(page) {
                    Some(image) => Event::Rendered { job, image, bytes },
                    None => Event::RenderFailed { job },
                }
            }
            Err(error) => {
                if worker.failure().is_some() {
                    send(Event::Failed(error.message()));
                    return;
                }
                Event::RenderFailed { job }
            }
        };
        if !send(event) {
            break;
        }
    }
    worker.close();
}

/// The text layer of a page PDFium found no characters on, from the words
/// read from its picture (step 8), on the page's own size. `None` when
/// the page was not read.
#[cfg(test)]
pub(super) fn scanned_text(
    store: &Store,
    file_id: &str,
    empty: &kanso_pdf::PageText,
) -> Option<kanso_pdf::PageText> {
    scanned_page(&store.pdf_ocr(file_id).ok()??, empty)
}

/// As [`scanned_text`], from the words already read out of the file.
fn scanned_page(
    ocr: &kanso_pdf::PdfOcr,
    empty: &kanso_pdf::PageText,
) -> Option<kanso_pdf::PageText> {
    let mut text = ocr.page(empty.page)?.to_page_text();
    text.width = empty.width;
    text.height = empty.height;
    // The words were read on the page as shown, upright: no rotation.
    (!text.chars.is_empty()).then_some(text)
}

/// Waits for the person to give a password; `None` when the reader closed.
fn wait_for_password(shared: &Shared) -> Option<String> {
    let mut inbox = shared.inbox.lock().ok()?;
    loop {
        if inbox.closed {
            return None;
        }
        if let Some(password) = inbox.password.take() {
            return Some(password);
        }
        inbox = shared.wake.wait(inbox).ok()?;
    }
}
