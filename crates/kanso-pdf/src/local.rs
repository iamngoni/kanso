//! PDFium inside the app, for the phones (issue #43, the phone reader).
//!
//! The desktop reads a PDF in a separate process under OS containment
//! ([`crate::contain`]). A phone can't: iOS lets an app start no process of
//! its own, and Android has no sandbox an app can put around code it loaded.
//! So on a phone the same worker loop runs on a thread of the app, over
//! in-memory pipes, and everything that can be kept is kept:
//!
//! * the loop is [`crate::worker`]'s own, so a page, its text and its smart
//!   inversion are the bytes the desktop gets (one text layer, one set of
//!   highlight quads);
//! * the client is [`crate::client::Worker`], so the frame limits, the
//!   page-count and page-size caps, the pixel caps, the answer checks and the
//!   per-request deadlines are the desktop's, unchanged;
//! * PDFium is the non-V8 build `scripts/fetch-pdfium.sh` pins, and takes the
//!   bytes of the document from memory, never a path.
//!
//! What is lost is the process boundary: a PDFium bug that corrupts memory
//! corrupts the app, and a request that hangs inside PDFium can be given up
//! on (the client's deadline fires and the pipes close) but not killed. The
//! thread is left behind, and PDFium (which serialises its calls) answers no
//! one else until the app restarts. The caps on pages, sizes and pixels are
//! what bound the work a hostile file can ask of it; the apps' memory limit
//! is the OS's own.

use std::collections::VecDeque;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use pdfium_render::prelude::*;

/// The stack the reader's thread gets: PDFium recurses on nested content.
const STACK_BYTES: usize = 16 * 1024 * 1024;
/// How long a closed reader waits for its thread before leaving it behind.
const LET_GO_AFTER: Duration = Duration::from_secs(2);

/// Where the PDFium library may be, most likely first: named by
/// `KANSO_PDFIUM_LIBRARY`; beside the code that holds this one (the
/// bridge's library: on a phone Flutter puts the bundled PDFium next to it,
/// in its own framework on iOS); by name, for the loader to find in the
/// app's own libraries (Android); and, in a development build, the one
/// `scripts/fetch-pdfium.sh` fetched.
pub fn library_candidates() -> Vec<PathBuf> {
    let mut found = Vec::new();
    if let Some(path) = std::env::var_os("KANSO_PDFIUM_LIBRARY") {
        found.push(PathBuf::from(path));
    }
    if let Some(dir) = module_dir() {
        for name in beside_names() {
            found.push(dir.join(name));
        }
    }
    if cfg!(any(target_os = "android", target_os = "linux")) {
        found.push(PathBuf::from("libpdfium.so"));
    }
    if cfg!(target_os = "ios") {
        found.push(PathBuf::from("pdfium.framework/pdfium"));
    }
    if cfg!(debug_assertions)
        && let Some(path) = crate::client::dev_library()
    {
        found.push(path);
    }
    found
}

/// Names the library may have next to the bridge's own.
fn beside_names() -> Vec<&'static str> {
    if cfg!(target_vendor = "apple") {
        vec![
            "libpdfium.dylib",
            "pdfium.framework/pdfium",
            "../pdfium.framework/pdfium",
        ]
    } else if cfg!(windows) {
        vec!["pdfium.dll"]
    } else {
        vec!["libpdfium.so"]
    }
}

/// The folder of the library this code is in, when the system can say.
#[cfg(unix)]
fn module_dir() -> Option<PathBuf> {
    // SAFETY: `dladdr` reads the address of a function of this module and
    // fills in `info`; the name it points to is valid until the module is
    // unloaded, and is copied at once.
    unsafe {
        let mut info: libc::Dl_info = std::mem::zeroed();
        let here = module_dir as *const () as *const libc::c_void;
        if libc::dladdr(here, &mut info) == 0 || info.dli_fname.is_null() {
            return None;
        }
        let name = std::ffi::CStr::from_ptr(info.dli_fname).to_str().ok()?;
        std::path::Path::new(name)
            .parent()
            .map(std::path::Path::to_path_buf)
    }
}

#[cfg(not(unix))]
fn module_dir() -> Option<PathBuf> {
    None
}

/// The process-wide PDFium. Its bindings are global in `pdfium-render` (a
/// second bind in one process is refused), so every reader shares this one.
struct Shared(Pdfium);

// SAFETY: a `Pdfium` holds no state but two pointers for a custom font
// provider, which is never set here; every call goes through the library's
// global bindings, which the `thread_safe` feature serialises behind a mutex.
unsafe impl Send for Shared {}
// SAFETY: as above; a shared reference only reaches the global bindings.
unsafe impl Sync for Shared {}

static PDFIUM: OnceLock<Result<Shared, String>> = OnceLock::new();

/// Held by whichever reader is inside PDFium (see
/// [`crate::worker::serve_with`]).
static GATE: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Loads PDFium once; the same answer for every later call.
fn pdfium() -> Result<&'static Pdfium, String> {
    PDFIUM
        .get_or_init(|| {
            let mut tried = Vec::new();
            for path in library_candidates() {
                match Pdfium::bind_to_library(&path) {
                    Ok(bindings) => return Ok(Shared(Pdfium::new(bindings))),
                    Err(e) => tried.push(format!("{}: {e}", path.display())),
                }
            }
            Err(if tried.is_empty() {
                "the PDFium library isn't in this build".to_owned()
            } else {
                format!("PDFium didn't load ({})", tried.join("; "))
            })
        })
        .as_ref()
        .map(|shared| &shared.0)
        .map_err(Clone::clone)
}

/// Whether PDFium can be loaded here; the reason when it can't.
pub fn availability() -> Result<(), String> {
    pdfium().map(|_| ())
}

/// One direction of the in-memory pipe the client and the loop talk over.
#[derive(Default)]
struct PipeState {
    data: VecDeque<u8>,
    closed: bool,
}

#[derive(Clone, Default)]
pub(crate) struct Pipe(Arc<(Mutex<PipeState>, Condvar)>);

impl Pipe {
    /// Ends the pipe: a reader gets what is left and then the end, a writer
    /// gets a broken pipe.
    fn close(&self) {
        let (lock, wake) = &*self.0;
        if let Ok(mut state) = lock.lock() {
            state.closed = true;
        }
        wake.notify_all();
    }

    /// Ends the pipe and drops what was in it.
    fn discard(&self) {
        let (lock, wake) = &*self.0;
        if let Ok(mut state) = lock.lock() {
            state.closed = true;
            state.data.clear();
        }
        wake.notify_all();
    }
}

struct PipeWriter(Pipe);
struct PipeReader(Pipe);

impl Write for PipeWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let (lock, wake) = &*(self.0).0;
        let mut state = lock.lock().map_err(|_| io::ErrorKind::BrokenPipe)?;
        if state.closed {
            return Err(io::ErrorKind::BrokenPipe.into());
        }
        state.data.extend(buf);
        wake.notify_all();
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Read for PipeReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        let (lock, wake) = &*(self.0).0;
        let mut state = lock.lock().map_err(|_| io::ErrorKind::BrokenPipe)?;
        loop {
            if !state.data.is_empty() {
                let n = buf.len().min(state.data.len());
                for (slot, byte) in buf.iter_mut().zip(state.data.drain(..n)) {
                    *slot = byte;
                }
                return Ok(n);
            }
            if state.closed {
                return Ok(0);
            }
            state = wake.wait(state).map_err(|_| io::ErrorKind::BrokenPipe)?;
        }
    }
}

/// Closing a side closes the pipe, so a client that drops its end frees the
/// loop and a loop that ends frees the client.
impl Drop for PipeWriter {
    fn drop(&mut self) {
        self.0.close();
    }
}

impl Drop for PipeReader {
    fn drop(&mut self) {
        self.0.close();
    }
}

/// A reader's loop, running on a thread of the app.
pub(crate) struct Local {
    to_loop: Pipe,
    from_loop: Pipe,
    thread: Option<JoinHandle<()>>,
}

impl Local {
    /// Ends the reader: both pipes close, so the loop's next read or write
    /// fails and the client's pending read returns.
    pub(crate) fn kill(&mut self) {
        self.to_loop.discard();
        self.from_loop.discard();
    }

    /// Waits a moment for the loop to finish, then leaves it behind if it is
    /// stuck inside PDFium.
    pub(crate) fn wait(&mut self) {
        let Some(thread) = self.thread.take() else {
            return;
        };
        let until = Instant::now() + LET_GO_AFTER;
        while !thread.is_finished() && Instant::now() < until {
            std::thread::sleep(Duration::from_millis(5));
        }
        if thread.is_finished() {
            thread.join().ok();
        }
    }
}

/// The pipes of a reader and the thread that serves them.
pub(crate) type Started = (Local, Box<dyn Write + Send>, Box<dyn Read + Send>);

/// Starts a reader's loop. `Err` says why PDFium isn't available.
pub(crate) fn start() -> Result<Started, String> {
    let pdfium = pdfium()?;
    let (to_loop, from_loop) = (Pipe::default(), Pipe::default());
    let (loop_in, loop_out) = (to_loop.clone(), from_loop.clone());
    let thread = std::thread::Builder::new()
        .name("pdf-reader".into())
        .stack_size(STACK_BYTES)
        .spawn(move || {
            let mut input = std::io::BufReader::new(PipeReader(loop_in));
            let mut output = std::io::BufWriter::new(PipeWriter(loop_out));
            // The client treats the end of the pipe as a crash, whatever
            // the loop returned.
            let _ = crate::worker::serve_with(pdfium, &mut input, &mut output, false, &GATE);
        })
        .map_err(|e| e.to_string())?;
    Ok((
        Local {
            to_loop: to_loop.clone(),
            from_loop: from_loop.clone(),
            thread: Some(thread),
        },
        Box::new(PipeWriter(to_loop)),
        Box::new(PipeReader(from_loop)),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_closed_pipe_ends_a_blocked_read() {
        let pipe = Pipe::default();
        let mut reader = PipeReader(pipe.clone());
        let closer = pipe.clone();
        let t = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(50));
            closer.close();
        });
        let mut buf = [0u8; 4];
        assert_eq!(reader.read(&mut buf).unwrap(), 0);
        t.join().unwrap();
    }

    #[test]
    fn bytes_arrive_in_order_and_a_closed_pipe_refuses_writes() {
        let pipe = Pipe::default();
        let mut writer = PipeWriter(pipe.clone());
        let mut reader = PipeReader(pipe.clone());
        writer.write_all(b"hello world").unwrap();
        let mut buf = [0u8; 5];
        reader.read_exact(&mut buf).unwrap();
        assert_eq!(&buf, b"hello");
        pipe.close();
        assert_eq!(
            writer.write(b"x").unwrap_err().kind(),
            io::ErrorKind::BrokenPipe
        );
    }
}
