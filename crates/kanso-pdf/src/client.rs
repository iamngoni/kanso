//! The app's side of the worker (issue #43, step 3): starts the helper,
//! sends it a document, asks for pages, and ends it when it hangs, crashes
//! or is no longer wanted.
//!
//! One [`Worker`] is one open document. Every call blocks, so the app makes
//! them from a background task. A call that outlives its deadline (about
//! 5 s per request, 60 s to open) kills the worker, and every later call
//! says so; the note and its annotations are untouched.

use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
#[cfg(not(windows))]
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use crate::PdfBlock;
use crate::protocol::{
    BYTES_PER_PIXEL, ErrorKind, FrameError, MAX_BLOB_BYTES, MAX_DOC_TEXT_BYTES,
    MAX_EXPORT_ANNOTATION_BYTES, MAX_PAGE_BYTES, MAX_TEXT_WINDOW, Request, Response,
    check_document, page_is_sound, plain_text_pages, read_frame, render_size, text_page_is_sound,
    text_window, write_frame,
};
use crate::text::PageText;

#[cfg(windows)]
mod launch_windows;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{MAX_PAGE_POINTS, check_page_size};
    use proptest::prelude::*;

    proptest! {
        /// A size a hostile worker could send that is not a page is refused.
        #[test]
        fn only_a_real_page_size_is_sound(w in any::<f32>(), h in any::<f32>()) {
            let sound = check_page_size(w, h).is_ok();
            prop_assert_eq!(
                sound,
                w.is_finite()
                    && h.is_finite()
                    && w > 0.0
                    && h > 0.0
                    && w <= MAX_PAGE_POINTS
                    && h <= MAX_PAGE_POINTS
            );
        }
    }

    proptest! {
        /// Pages that arrive one after another are kept within the
        /// document's room: the total never passes it, each page is the
        /// start of what arrived, and nothing is cut while there was room.
        #[test]
        fn a_documents_text_is_kept_within_its_room(
            pages in proptest::collection::vec(".{0,30}", 0..12),
            room in 0usize..120,
        ) {
            let mut used = 0usize;
            let mut kept = Vec::new();
            let mut any_cut = false;
            for page in &pages {
                let (fit, cut) = fit_text(page.clone(), room - used);
                prop_assert!(page.starts_with(&fit));
                prop_assert_eq!(cut, fit.len() < page.len());
                used += fit.len();
                any_cut |= cut;
                kept.push(fit);
            }
            prop_assert!(used <= room);
            let all: usize = pages.iter().map(String::len).sum();
            prop_assert_eq!(any_cut, all > room);
            prop_assert_eq!(kept.iter().map(String::len).sum::<usize>(), used);
        }
    }

    proptest! {
        /// The reader's words come from the kind alone, and are plain: not
        /// empty, one short line, no path.
        #[test]
        fn every_refusal_has_plain_words(kind in prop_oneof![
            Just(ErrorKind::Unreadable),
            Just(ErrorKind::PasswordRequired),
            Just(ErrorKind::WrongPassword),
            Just(ErrorKind::TooManyPages),
            Just(ErrorKind::PageTooLarge),
            Just(ErrorKind::FileTooLarge),
            Just(ErrorKind::NoSuchPage),
            Just(ErrorKind::Protocol),
            Just(ErrorKind::Internal),
        ]) {
            let words = refusal_words(kind);
            prop_assert!(!words.is_empty() && words.len() < 80);
            prop_assert!(!words.contains('\n') && !words.contains('/'));
        }
    }

    #[test]
    fn a_refusal_is_worded_by_the_app() {
        assert_eq!(
            refusal_words(ErrorKind::Internal),
            "This PDF couldn't be opened."
        );
        assert_eq!(
            refusal_words(ErrorKind::PasswordRequired),
            "This PDF needs a password."
        );
    }
}

/// How long a request may take before the worker is killed.
pub const REQUEST_DEADLINE: Duration = Duration::from_secs(5);
/// How long opening a document may take.
pub const OPEN_DEADLINE: Duration = Duration::from_secs(60);
/// How long reading a whole document's text may take, over all the windows
/// of it (each window is also held to [`REQUEST_DEADLINE`]).
pub const DOCUMENT_TEXT_DEADLINE: Duration = Duration::from_secs(120);

/// How long the worker gets before it is killed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Deadlines {
    pub open: Duration,
    pub request: Duration,
}

impl Default for Deadlines {
    fn default() -> Self {
        Deadlines {
            open: OPEN_DEADLINE,
            request: REQUEST_DEADLINE,
        }
    }
}

/// The file name of the worker beside the app.
pub const WORKER_NAME: &str = if cfg!(windows) {
    "kanso-pdf-worker.exe"
} else {
    "kanso-pdf-worker"
};

/// Where the worker and its library are.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkerPaths {
    pub exe: PathBuf,
    /// The PDFium library when it is not where the worker looks by itself
    /// (beside it, or in a macOS bundle's `Frameworks`).
    pub library: Option<PathBuf>,
}

/// Why the worker could not give what was asked.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum WorkerError {
    /// There is no worker to start, or it would not start (it refuses to run
    /// without containment).
    #[error("The PDF reader isn't available on this computer.")]
    Unavailable(String),
    /// The worker took too long and was stopped.
    #[error("This PDF took too long to read.")]
    TimedOut,
    /// The worker stopped by itself, or was killed.
    #[error("The PDF reader stopped unexpectedly.")]
    Crashed,
    /// The worker answered, but said no. The words are the app's own for the
    /// kind, never the worker's.
    #[error("{message}")]
    Refused { kind: ErrorKind, message: String },
    /// The worker's answer was not what was asked for.
    #[error("The PDF reader sent something unexpected.")]
    BadReply,
    /// The caller stopped a long read (text extraction in the background).
    #[error("Reading the PDF was stopped.")]
    Cancelled,
}

impl WorkerError {
    /// What the reader says: plain words, never a path or a code.
    pub fn message(&self) -> String {
        match self {
            WorkerError::Refused { message, .. } => message.clone(),
            WorkerError::TimedOut | WorkerError::Crashed => {
                "This PDF couldn't be opened.".to_owned()
            }
            other => other.to_string(),
        }
    }
}

/// An opened document.
#[derive(Clone, Debug, PartialEq)]
pub struct Opened {
    pub pages: u32,
    /// Each page's size in points, as shown.
    pub sizes: Vec<[f32; 2]>,
    pub encrypted: bool,
}

/// A page drawn: `bgra` is `width * height * 4` bytes, blue first.
#[derive(Clone, PartialEq, Eq)]
pub struct RenderedPage {
    pub page: u32,
    pub width: u32,
    pub height: u32,
    pub bgra: Vec<u8>,
}

impl std::fmt::Debug for RenderedPage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RenderedPage({}: {}x{})",
            self.page, self.width, self.height
        )
    }
}

/// The words of a document's pages, one string for each, page 1 first.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentText {
    pub pages: Vec<String>,
    /// A page was cut at [`crate::protocol::MAX_PAGE_CHARS`] or the document at
    /// [`MAX_DOC_TEXT_BYTES`]: what is here is the start of the text.
    pub truncated: bool,
}

/// `page` cut to at most `room` bytes, at a character boundary; whether it
/// was cut.
fn fit_text(mut page: String, room: usize) -> (String, bool) {
    if page.len() <= room {
        return (page, false);
    }
    let mut cut = room;
    while !page.is_char_boundary(cut) {
        cut -= 1;
    }
    page.truncate(cut);
    (page, true)
}

/// Finds the worker beside the running app (a macOS bundle's `MacOS`, the
/// folder of `kanso.exe`), or in `../lib/kanso` as an installed Linux package
/// has it, and in a development build the one `cargo build -p kanso-pdf
/// --features render` left in the target folder, with the library
/// `scripts/fetch-pdfium.sh` fetched.
pub fn locate() -> Option<WorkerPaths> {
    let exe_dir = std::env::current_exe().ok()?.parent()?.to_path_buf();
    let exe = [
        exe_dir.join(WORKER_NAME),
        exe_dir.join("../lib/kanso").join(WORKER_NAME),
    ]
    .into_iter()
    .find(|path| path.is_file())?;
    // Bundled: the worker finds its library by itself. In a development
    // build, the fetched one.
    let library = if cfg!(debug_assertions) {
        dev_library()
    } else {
        None
    };
    Some(WorkerPaths { exe, library })
}

/// `vendor/pdfium/<platform>/…` from `scripts/fetch-pdfium.sh`, for builds
/// that run from the source tree.
pub(crate) fn dev_library() -> Option<PathBuf> {
    let platform = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => "mac-arm64",
        ("macos", _) => "mac-x64",
        ("linux", "aarch64") => "linux-arm64",
        ("linux", _) => "linux-x64",
        ("windows", "aarch64") => "win-arm64",
        _ => "win-x64",
    };
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../vendor/pdfium");
    let dir = root.join(platform);
    let name = libloading_name();
    let sub = if cfg!(windows) { "bin" } else { "lib" };
    let library = dir.join(sub).join(name);
    library.is_file().then_some(library)
}

fn libloading_name() -> &'static str {
    if cfg!(target_os = "macos") {
        "libpdfium.dylib"
    } else if cfg!(windows) {
        "pdfium.dll"
    } else {
        "libpdfium.so"
    }
}

/// A started process the client can end.
enum Process {
    #[cfg(not(windows))]
    Std(Child),
    #[cfg(windows)]
    Windows(launch_windows::Launched),
}

impl Process {
    fn kill(&mut self) {
        match self {
            #[cfg(not(windows))]
            Process::Std(child) => {
                child.kill().ok();
            }
            #[cfg(windows)]
            Process::Windows(launched) => launched.kill(),
        }
    }

    fn id(&self) -> Option<u32> {
        match self {
            #[cfg(not(windows))]
            Process::Std(child) => Some(child.id()),
            #[cfg(windows)]
            Process::Windows(_) => None,
        }
    }

    fn wait(&mut self) {
        match self {
            #[cfg(not(windows))]
            Process::Std(child) => {
                child.wait().ok();
            }
            #[cfg(windows)]
            Process::Windows(launched) => launched.wait(),
        }
    }
}

/// What the watchdog thread watches.
#[derive(Default)]
struct Watch {
    deadline: Option<Instant>,
    fired: bool,
    stop: bool,
}

type Shared = Arc<(Mutex<Watch>, Condvar)>;

/// One document's worker.
pub struct Worker {
    process: Arc<Mutex<Process>>,
    input: BufWriter<Box<dyn std::io::Write + Send>>,
    output: BufReader<Box<dyn std::io::Read + Send>>,
    watch: Shared,
    opened: bool,
    /// The page sizes the open reported (and [`check_document`] passed): a
    /// page that comes back is held to the size asked for, not just the caps.
    sizes: Vec<[f32; 2]>,
    dead: Option<WorkerError>,
    deadlines: Deadlines,
}

impl Worker {
    /// Starts the worker with an empty environment and only the pipes.
    pub fn start(paths: &WorkerPaths) -> Result<Worker, WorkerError> {
        Worker::start_with(paths, Deadlines::default())
    }

    /// As [`Worker::start`], with other deadlines (tests).
    pub fn start_with(paths: &WorkerPaths, deadlines: Deadlines) -> Result<Worker, WorkerError> {
        let (process, input, output) = spawn(paths)?;
        let process = Arc::new(Mutex::new(process));
        let watch: Shared = Arc::default();
        let watcher = (process.clone(), watch.clone());
        std::thread::Builder::new()
            .name("pdf-worker-watchdog".into())
            .spawn(move || watchdog(&watcher.0, &watcher.1))
            .map_err(|e| WorkerError::Unavailable(e.to_string()))?;
        Ok(Worker {
            process,
            input: BufWriter::new(input),
            output: BufReader::new(output),
            watch,
            opened: false,
            sizes: Vec::new(),
            dead: None,
            deadlines,
        })
    }

    /// Opens the document (once). The password is used for this call only.
    pub fn open(&mut self, bytes: &[u8], password: Option<&str>) -> Result<Opened, WorkerError> {
        if self.opened {
            return Err(WorkerError::BadReply);
        }
        // Over the limit the worker would hang up on the frame; say why here.
        if bytes.len() > MAX_BLOB_BYTES {
            let kind = ErrorKind::FileTooLarge;
            return Err(self.end(WorkerError::Refused {
                kind,
                message: refusal_words(kind).to_owned(),
            }));
        }
        let request = Request::Open {
            password: password.map(str::to_owned),
        };
        let (response, _) = self.call(&request, bytes, self.deadlines.open)?;
        match response {
            // The worker has checked this; a worker that sends a document it
            // should have refused is not trusted with the rest.
            Response::Opened {
                pages,
                sizes,
                encrypted,
            } if check_document(pages, &sizes).is_ok() => {
                self.opened = true;
                self.sizes.clone_from(&sizes);
                Ok(Opened {
                    pages,
                    sizes,
                    encrypted,
                })
            }
            _ => Err(self.misbehaved()),
        }
    }

    /// Draws a page `width_px` wide (the worker caps it).
    pub fn render(&mut self, page: u32, width_px: u32) -> Result<RenderedPage, WorkerError> {
        self.render_as(page, width_px, false)
    }

    /// As [`Worker::render`]; with `smart_invert` the page is drawn for a dark
    /// reader (text light, pictures in their own colours).
    pub fn render_as(
        &mut self,
        page: u32,
        width_px: u32,
        smart_invert: bool,
    ) -> Result<RenderedPage, WorkerError> {
        let (response, bgra) = self.call(
            &Request::Render {
                page,
                width_px,
                smart_invert,
            },
            &[],
            self.deadlines.request,
        )?;
        match response {
            Response::Page {
                page: answered,
                width,
                height,
            } if answered == page
                && page_is_sound(width, height, bgra.len())
                && self.fits_request(page, width_px, width, height) =>
            {
                Ok(RenderedPage {
                    page,
                    width,
                    height,
                    bgra,
                })
            }
            _ => Err(self.misbehaved()),
        }
    }

    /// Writes the note's highlights, comments and ink into an exported copy.
    /// The worker receives the original bytes held by its open document and
    /// returns a new byte vector; it never writes to a path.
    pub fn export(&mut self, annotations: PdfBlock) -> Result<Vec<u8>, WorkerError> {
        let annotations = serde_json::to_vec(&annotations).map_err(|_| self.misbehaved())?;
        if annotations.len() > MAX_EXPORT_ANNOTATION_BYTES {
            return Err(self.misbehaved());
        }
        let (response, bytes) =
            self.call(&Request::Export, &annotations, self.deadlines.request)?;
        match response {
            Response::Exported if bytes.starts_with(b"%PDF-") && bytes.len() <= MAX_BLOB_BYTES => {
                Ok(bytes)
            }
            _ => Err(self.misbehaved()),
        }
    }

    /// Whether a page that came back is no bigger than the one asked for:
    /// the size [`render_size`] gives for the page's declared size and the
    /// requested width. Within the global caps alone, a worker that has been
    /// taken over could answer every small request with a 64 MB page.
    fn fits_request(&self, page: u32, width_px: u32, width: u32, height: u32) -> bool {
        self.sizes
            .get(page as usize)
            .and_then(|[w, h]| render_size(*w, *h, width_px))
            .is_some_and(|(max_w, max_h)| width <= max_w && height <= max_h)
    }

    /// A page's characters with their boxes (`page` is 0-based; the text's
    /// own page number is 1-based). Cut at [`crate::protocol::MAX_PAGE_CHARS`] characters.
    pub fn text(&mut self, page: u32) -> Result<PageText, WorkerError> {
        let (response, blob) = self.call(&Request::Text { page }, &[], self.deadlines.request)?;
        match response {
            Response::TextPage {
                page: answered,
                chars,
                ..
            } if answered == page => {
                let size = self.sizes.get(page as usize).copied();
                match (size, serde_json::from_slice::<PageText>(&blob)) {
                    (Some(size), Ok(text)) if text_page_is_sound(page, size, chars, &text) => {
                        Ok(text)
                    }
                    _ => Err(self.misbehaved()),
                }
            }
            _ => Err(self.misbehaved()),
        }
    }

    /// The plain text of up to `count` pages from `from` (0-based): no more
    /// than [`MAX_TEXT_WINDOW`] pages and none past the document's end. The
    /// answer has one string for each page of that window, each cut at
    /// [`crate::protocol::MAX_PAGE_CHARS`]; a page with no text layer is empty.
    pub fn plain_text(&mut self, from: u32, count: u32) -> Result<Vec<String>, WorkerError> {
        self.plain_text_within(from, count, self.deadlines.request)
            .map(|text| text.pages)
    }

    fn plain_text_within(
        &mut self,
        from: u32,
        count: u32,
        deadline: Duration,
    ) -> Result<DocumentText, WorkerError> {
        let pages = self.sizes.len() as u32;
        // Nothing to ask for is the worker's NoSuchPage, said here.
        if text_window(pages, from, count).is_none() {
            let kind = ErrorKind::NoSuchPage;
            return Err(WorkerError::Refused {
                kind,
                message: refusal_words(kind).to_owned(),
            });
        }
        let (response, blob) = self.call(&Request::PlainText { from, count }, &[], deadline)?;
        match response {
            Response::PlainText {
                from: answered,
                lens,
                truncated,
            } => match plain_text_pages(pages, (from, count), answered, &lens, &blob) {
                Some(pages) => Ok(DocumentText { pages, truncated }),
                None => Err(self.misbehaved()),
            },
            _ => Err(self.misbehaved()),
        }
    }

    /// The plain text of the whole document, for search: window after window
    /// of [`MAX_TEXT_WINDOW`] pages, each under the request deadline, all
    /// within `budget`. Its own job, not a render: `cancelled` is asked
    /// between windows, and a read that is cancelled or runs out of budget
    /// ends the worker. Pages past [`MAX_DOC_TEXT_BYTES`] of text in all are
    /// left empty, and the last page that fits is cut; both set `truncated`.
    pub fn document_text(
        &mut self,
        budget: Duration,
        cancelled: &dyn Fn() -> bool,
    ) -> Result<DocumentText, WorkerError> {
        let total = self.sizes.len() as u32;
        let started = Instant::now();
        let mut out = DocumentText {
            pages: Vec::with_capacity(total as usize),
            truncated: false,
        };
        let mut bytes = 0usize;
        let mut next = 0;
        while let Some(window) = text_window(total, next, MAX_TEXT_WINDOW) {
            if cancelled() {
                return Err(self.end(WorkerError::Cancelled));
            }
            let left = budget.saturating_sub(started.elapsed());
            if left.is_zero() {
                return Err(self.end(WorkerError::TimedOut));
            }
            let reply = self.plain_text_within(
                window.start,
                MAX_TEXT_WINDOW,
                left.min(self.deadlines.request),
            )?;
            out.truncated |= reply.truncated;
            for page in reply.pages {
                let (page, cut) = fit_text(page, MAX_DOC_TEXT_BYTES - bytes);
                bytes += page.len();
                out.truncated |= cut;
                out.pages.push(page);
            }
            next = window.end;
            if bytes >= MAX_DOC_TEXT_BYTES && next < total {
                // The rest is not read: the document is indexed truncated.
                out.pages.resize(total as usize, String::new());
                out.truncated = true;
                break;
            }
        }
        Ok(out)
    }

    /// The worker's process id, while it has one.
    pub fn pid(&self) -> Option<u32> {
        self.process.lock().ok()?.id()
    }

    /// Whether the worker is gone; the error is what ended it.
    pub fn failure(&self) -> Option<&WorkerError> {
        self.dead.as_ref()
    }

    /// Ends the worker now, for a reader that was closed or a render that is
    /// no longer wanted. Later calls report [`WorkerError::Crashed`].
    pub fn kill(&mut self) {
        self.end(WorkerError::Crashed);
    }

    /// Asks the worker to close, then ends it either way.
    pub fn close(mut self) {
        if self.dead.is_none() {
            let _ = self.call(&Request::Close, &[], self.deadlines.request);
        }
    }

    /// One request and its answer, under the deadline.
    fn call(
        &mut self,
        request: &Request,
        blob: &[u8],
        deadline: Duration,
    ) -> Result<(Response, Vec<u8>), WorkerError> {
        if let Some(error) = &self.dead {
            return Err(error.clone());
        }
        self.arm(deadline);
        let result = self.exchange(request, blob);
        let timed_out = self.disarm();
        match result {
            // Only a page and a page's text carry a blob; anything else with
            // one attached is not what a worker sends.
            Ok((ref response, ref blob))
                if !blob.is_empty()
                    && !matches!(
                        response,
                        Response::Page { .. }
                            | Response::TextPage { .. }
                            | Response::PlainText { .. }
                            | Response::Exported
                    ) =>
            {
                Err(self.misbehaved())
            }
            Ok((Response::Error { kind, .. }, _)) => {
                // The words are the app's, chosen by the kind: a worker that
                // has been taken over could otherwise put any text, of any
                // length, in the reader.
                let refused = WorkerError::Refused {
                    kind,
                    message: refusal_words(kind).to_owned(),
                };
                // A failed open ends the worker's use; other failures leave
                // it ready for the next request.
                if !self.opened {
                    self.end(refused.clone());
                }
                Err(refused)
            }
            Ok(reply) => Ok(reply),
            Err(_) if timed_out => Err(self.end(WorkerError::TimedOut)),
            Err(FrameError::Io(_)) => Err(self.end(WorkerError::Crashed)),
            Err(_) => Err(self.misbehaved()),
        }
    }

    fn exchange(
        &mut self,
        request: &Request,
        blob: &[u8],
    ) -> Result<(Response, Vec<u8>), FrameError> {
        write_frame(&mut self.input, request, blob)?;
        // A page may be up to 64 MB of pixels (4096 x 4096 x 4), more than a
        // PDF may be; no answer carries more.
        match read_frame::<Response>(&mut self.output, MAX_PAGE_BYTES)? {
            Some(reply) => Ok(reply),
            None => Err(FrameError::Io(std::io::ErrorKind::UnexpectedEof.into())),
        }
    }

    fn arm(&self, after: Duration) {
        let (lock, wake) = &*self.watch;
        let mut watch = lock.lock().expect("watchdog state poisoned");
        watch.deadline = Some(Instant::now() + after);
        wake.notify_all();
    }

    /// Stops the clock; whether the watchdog fired first.
    fn disarm(&self) -> bool {
        let (lock, _) = &*self.watch;
        let mut watch = lock.lock().expect("watchdog state poisoned");
        watch.deadline = None;
        watch.fired
    }

    /// The worker said something it should not have: end it.
    fn misbehaved(&mut self) -> WorkerError {
        self.end(WorkerError::BadReply)
    }

    fn end(&mut self, error: WorkerError) -> WorkerError {
        if self.dead.is_none() {
            self.dead = Some(error);
        }
        if let Ok(mut process) = self.process.lock() {
            process.kill();
        }
        self.dead.clone().unwrap_or(WorkerError::Crashed)
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        {
            let (lock, wake) = &*self.watch;
            if let Ok(mut watch) = lock.lock() {
                watch.stop = true;
                wake.notify_all();
            }
        }
        if let Ok(mut process) = self.process.lock() {
            process.kill();
            process.wait();
        }
    }
}

/// Kills the worker when the armed deadline passes.
fn watchdog(process: &Mutex<Process>, shared: &(Mutex<Watch>, Condvar)) {
    let (lock, wake) = shared;
    let mut watch = lock.lock().expect("watchdog state poisoned");
    loop {
        if watch.stop {
            return;
        }
        match watch.deadline {
            None => watch = wake.wait(watch).expect("watchdog state poisoned"),
            Some(deadline) => {
                let now = Instant::now();
                if now >= deadline {
                    watch.fired = true;
                    watch.deadline = None;
                    if let Ok(mut process) = process.lock() {
                        process.kill();
                    }
                } else {
                    watch = wake
                        .wait_timeout(watch, deadline - now)
                        .expect("watchdog state poisoned")
                        .0;
                }
            }
        }
    }
}

type Pipes = (
    Process,
    Box<dyn std::io::Write + Send>,
    Box<dyn std::io::Read + Send>,
);

#[cfg(not(windows))]
fn spawn(paths: &WorkerPaths) -> Result<Pipes, WorkerError> {
    let mut command = Command::new(&paths.exe);
    command
        .env_clear()
        .current_dir("/")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    if let Some(library) = &paths.library {
        command.arg("--library").arg(library);
    }
    let mut child = command
        .spawn()
        .map_err(|e| WorkerError::Unavailable(e.to_string()))?;
    let stdin: ChildStdin = child.stdin.take().ok_or(WorkerError::Crashed)?;
    let stdout: ChildStdout = child.stdout.take().ok_or(WorkerError::Crashed)?;
    Ok((Process::Std(child), Box::new(stdin), Box::new(stdout)))
}

#[cfg(windows)]
fn spawn(paths: &WorkerPaths) -> Result<Pipes, WorkerError> {
    let (launched, stdin, stdout) = launch_windows::launch(&paths.exe, paths.library.as_deref())
        .map_err(WorkerError::Unavailable)?;
    Ok((
        Process::Windows(launched),
        Box::new(stdin),
        Box::new(stdout),
    ))
}

/// What the reader says when the worker refuses a request.
pub fn refusal_words(kind: ErrorKind) -> &'static str {
    match kind {
        ErrorKind::PasswordRequired => "This PDF needs a password.",
        ErrorKind::WrongPassword => "That password isn't right.",
        ErrorKind::TooManyPages => "This PDF has more than 2,000 pages.",
        ErrorKind::PageTooLarge => "This PDF has a page too large to show.",
        ErrorKind::FileTooLarge => "This PDF is larger than Kanso can show.",
        ErrorKind::NoSuchPage => "There is no such page.",
        ErrorKind::Unreadable | ErrorKind::Protocol | ErrorKind::Internal => {
            "This PDF couldn't be opened."
        }
    }
}

/// Bytes a page of this many pixels takes; the app's texture budget.
pub fn page_bytes(width: u32, height: u32) -> usize {
    width as usize * height as usize * BYTES_PER_PIXEL
}
