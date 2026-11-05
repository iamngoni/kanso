//! The local link: how a second `kanso` process talks to the running app,
//! on every desktop platform. It carries `kanso://` callbacks (Windows and
//! Linux; docs/ACCOUNTS.md › Sign in with Apple, Google and GitHub), files
//! to open, sticky notes and notes to capture.
//!
//! On macOS, `ASWebAuthenticationSession` hands Apple's callback straight to
//! the process that started it. Elsewhere the system browser opens
//! `kanso://auth/apple?flow=…&s=…`, and the operating system starts a new
//! `kanso` with that URL as its argument (the `x-scheme-handler/kanso` desktop
//! entry on Linux, `HKCU\Software\Classes\kanso` from the Windows installer).
//! That second process can't open the data folder the running app holds, so
//! it forwards the URL here over a loopback connection and exits.
//!
//! Another app could register the same scheme; the owner accepts that. What
//! limits it:
//!
//! - [`Router`] accepts a callback only for a flow this process started and
//!   is still waiting on, once, and only until the flow's deadline.
//! - The URL's `s` is never logged, printed or shown: the failures here say
//!   what happened without the URL.
//! - The endpoint file is readable only by its owner, and each side proves
//!   it read it: the app greets with one token before the forwarder sends
//!   anything, so a port another process took over after Kanso quit never
//!   receives a secret; the forwarder answers with the other. The tokens
//!   come from the system's random source.
//! - The same link carries files to open (`kanso file.md` while Kanso runs,
//!   or the file manager's Open With): a second kind of message after the
//!   token, with at most [`MAX_FILES`] absolute paths of at most
//!   [`MAX_PATH`] bytes. The app opens only paths that are existing note
//!   files ([`crate::open_documents::note_files`]), whatever it is sent.
//! - And sticky notes to make (`kanso sticky`): a third kind, with the
//!   colour, the notebook and at most [`MAX_STICKY_TEXT`] bytes of text,
//!   which the app saves as a note and opens as a sticky (`sticky.rs`).
//! - And notes to add (`kanso capture`): a fourth kind, one line of JSON
//!   ([`crate::capture_request`]), which the app saves and answers with the
//!   new note's ID. The link has no message that reads, searches or lists:
//!   a program that finds the endpoint file can only add notes, and a
//!   message of any other kind is refused.
//! - Each connection is served on its own thread and gets [`IO_TIMEOUT`] in
//!   all, however it trickles bytes, so a silent or slow local client can't
//!   hold up a real forward; at most [`MAX_SERVING`] are served at once.

use std::io::{BufRead as _, BufReader, Read as _, Write as _};
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use crate::capture_request::{self, CaptureRequest};
use crate::provider_browser::BrowserError;

/// The scheme the browser returns to.
pub const SCHEME: &str = "kanso";

/// The longest a callback wait lasts, whatever the server says.
pub const MAX_WAIT: Duration = Duration::from_secs(10 * 60);

/// How long either side of a forward waits for the other, in all.
const IO_TIMEOUT: Duration = Duration::from_secs(3);

/// The most connections served at once; more are closed unanswered.
const MAX_SERVING: usize = 16;

/// The longest message a forwarder may send: two lines, a token and a URL.
const MAX_MESSAGE: u64 = 4096;

/// The most files one message may carry.
pub const MAX_FILES: usize = 32;

/// The longest path a message may carry, in bytes.
const MAX_PATH: usize = 4096;

/// The longest message that opens files: the token, the kind, the count
/// and [`MAX_FILES`] paths.
const MAX_OPEN_MESSAGE: u64 = (MAX_FILES * (MAX_PATH + 1) + 256) as u64;

/// The kind line of a message that opens files. No URL is this.
const OPEN_FILES: &str = "open-files";

/// Where the app sends the files a forwarder asks it to open.
pub type FileSink = Arc<dyn Fn(Vec<PathBuf>) + Send + Sync>;

/// The kind line of a message that makes a sticky note.
const OPEN_STICKY: &str = "open-sticky";

/// The most text a sticky note sent here may have, in bytes. With the
/// lines before it, well inside [`MAX_OPEN_MESSAGE`].
pub const MAX_STICKY_TEXT: usize = 64 * 1024;

/// The longest notebook name or ID a sticky message may carry, in bytes.
const MAX_BOOK: usize = 1024;

/// A sticky note `kanso sticky` asks the running app to make.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StickyRequest {
    pub text: String,
    /// The notebook's name or ID; the Stickies notebook when `None`.
    pub book: Option<String>,
    pub color: Option<crate::config::StickyColor>,
}

/// Where the app makes the stickies a forwarder asks for: `Err` says why
/// it didn't, and goes back to the forwarder.
pub type StickySink = Arc<dyn Fn(StickyRequest) -> Result<(), String> + Send + Sync>;

/// The app's [`StickySink`], set once it can make notes.
static STICKIES: std::sync::OnceLock<StickySink> = std::sync::OnceLock::new();

/// Makes `sink` take the stickies forwarders send from now on.
pub fn on_stickies(sink: StickySink) {
    STICKIES.set(sink).ok();
}

/// Where the app adds the notes a forwarder asks for: the new note's ID, or
/// why it didn't, which goes back to the forwarder.
pub type CaptureSink = Arc<dyn Fn(CaptureRequest) -> Result<String, String> + Send + Sync>;

/// The endpoint file in the data folder.
const ENDPOINT_FILE: &str = "sign-in-callback";

/// Whether a command-line argument is a `kanso://` URL the OS passed in,
/// rather than a command.
pub fn is_callback_argument(arg: &str) -> bool {
    arg.len() > SCHEME.len() + 3
        && arg[..SCHEME.len()].eq_ignore_ascii_case(SCHEME)
        && arg[SCHEME.len()..].starts_with("://")
}

/// What became of a callback URL.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Delivery {
    /// The flow that was waiting for it has it now.
    Delivered,
    /// No flow of this process is waiting for it (never started, already
    /// answered, cancelled or expired).
    NotWaiting,
    /// It isn't Apple's callback URL.
    NotACallback,
}

struct Waiter {
    flow: String,
    deadline: Instant,
    reply: async_channel::Sender<Result<String, BrowserError>>,
}

/// The flows waiting for their callback.
pub struct Router {
    waiters: Mutex<Vec<Waiter>>,
}

/// This process's router.
pub static ROUTER: Router = Router::new();

impl Router {
    pub const fn new() -> Router {
        Router {
            waiters: Mutex::new(Vec::new()),
        }
    }

    /// Waits for `flow`'s callback until `deadline` (at most [`MAX_WAIT`]
    /// from now). The receiver gets the URL once, or closes when the wait
    /// is dropped. A second wait for the same flow replaces the first.
    pub fn wait_for(
        &self,
        flow: &str,
        deadline: Instant,
    ) -> async_channel::Receiver<Result<String, BrowserError>> {
        let (reply, receiver) = async_channel::bounded(1);
        let deadline = deadline.min(Instant::now() + MAX_WAIT);
        let mut waiters = self.lock();
        waiters.retain(|waiter| waiter.flow != flow);
        waiters.push(Waiter {
            flow: flow.to_owned(),
            deadline,
            reply,
        });
        receiver
    }

    /// Hands `url` to the flow waiting for it, if there is one.
    pub fn deliver(&self, url: &str) -> Delivery {
        self.deliver_at(url, Instant::now())
    }

    fn deliver_at(&self, url: &str, now: Instant) -> Delivery {
        let Some(callback) = crate::provider_browser::apple_flow(url) else {
            return Delivery::NotACallback;
        };
        let mut waiters = self.lock();
        // Expired waits and those whose flow stopped listening go first.
        waiters.retain(|waiter| waiter.deadline > now && !waiter.reply.is_closed());
        let Some(index) = waiters
            .iter()
            .position(|waiter| waiter.flow == callback.flow)
        else {
            return Delivery::NotWaiting;
        };
        let waiter = waiters.swap_remove(index);
        match waiter.reply.try_send(Ok(url.to_owned())) {
            Ok(()) => Delivery::Delivered,
            Err(_) => Delivery::NotWaiting,
        }
    }

    /// How many flows are waiting (for tests).
    #[cfg(test)]
    fn waiting(&self) -> usize {
        self.lock().len()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Vec<Waiter>> {
        self.waiters
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

// ---------------------------------------------------------------- the link

/// Where the running app listens, and the two tokens.
#[derive(Clone, PartialEq, Eq)]
struct Endpoint {
    port: u16,
    /// What the app greets a forwarder with.
    server_token: String,
    /// What a forwarder answers with.
    client_token: String,
}

impl Endpoint {
    fn new(port: u16) -> Endpoint {
        Endpoint {
            port,
            server_token: token(),
            client_token: token(),
        }
    }

    fn to_text(&self) -> String {
        format!(
            "{}\n{}\n{}\n",
            self.port, self.server_token, self.client_token
        )
    }

    fn parse(text: &str) -> Option<Endpoint> {
        let mut lines = text.lines();
        let port = lines.next()?.parse().ok()?;
        let server_token = lines.next()?.to_owned();
        let client_token = lines.next()?.to_owned();
        (is_token(&server_token) && is_token(&client_token)).then_some(Endpoint {
            port,
            server_token,
            client_token,
        })
    }
}

/// 32 bytes from the system's random source, in hex.
fn token() -> String {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).expect("the system's random source failed");
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn is_token(text: &str) -> bool {
    text.len() == 64 && text.bytes().all(|b| b.is_ascii_hexdigit())
}

/// Compares two tokens in time that doesn't depend on where they differ.
fn same(a: &str, b: &str) -> bool {
    a.len() == b.len()
        && a.bytes()
            .zip(b.bytes())
            .fold(0u8, |diff, (x, y)| diff | (x ^ y))
            == 0
}

fn endpoint_path(data_dir: &Path) -> PathBuf {
    data_dir.join(ENDPOINT_FILE)
}

/// Writes the endpoint file readable only by this user.
fn write_endpoint(path: &Path, endpoint: &Endpoint) -> std::io::Result<()> {
    let temporary = path.with_extension("tmp");
    {
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create(true).truncate(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt as _;
            options.mode(0o600);
        }
        let mut file = options.open(&temporary)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            file.set_permissions(std::fs::Permissions::from_mode(0o600))?;
        }
        file.write_all(endpoint.to_text().as_bytes())?;
        file.sync_all()?;
    }
    std::fs::rename(&temporary, path)
}

/// Starts listening for forwarded callbacks for `router`, and writes the
/// endpoint into `data_dir`. Runs for the rest of the process.
#[cfg(test)]
pub fn listen(data_dir: &Path, router: &'static Router, files: FileSink) -> std::io::Result<()> {
    listen_with(data_dir, router, files, no_captures())
}

/// A sink for a link that adds no notes: every capture is refused.
#[cfg(test)]
fn no_captures() -> CaptureSink {
    Arc::new(|_| {
        Err("This Kanso isn't taking notes from outside yet. Try again in a moment.".into())
    })
}

/// [`listen`] with the sink that adds the notes forwarders send.
pub fn listen_with(
    data_dir: &Path,
    router: &'static Router,
    files: FileSink,
    captures: CaptureSink,
) -> std::io::Result<()> {
    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?;
    let endpoint = Arc::new(Endpoint::new(listener.local_addr()?.port()));
    write_endpoint(&endpoint_path(data_dir), &endpoint)?;
    let serving = Arc::new(AtomicUsize::new(0));
    std::thread::Builder::new()
        .name("kanso-callbacks".into())
        .spawn(move || {
            for stream in listener.incoming() {
                let Ok(stream) = stream else { continue };
                if serving.fetch_add(1, Ordering::AcqRel) >= MAX_SERVING {
                    serving.fetch_sub(1, Ordering::AcqRel);
                    continue;
                }
                let endpoint = endpoint.clone();
                let slot = serving.clone();
                let files = files.clone();
                let captures = captures.clone();
                let spawned = std::thread::Builder::new()
                    .name("kanso-callback".into())
                    .spawn(move || {
                        let deadline = Instant::now() + IO_TIMEOUT;
                        if let Err(error) =
                            serve(stream, &endpoint, router, &files, &captures, deadline)
                        {
                            // The error never carries the URL.
                            eprintln!("kanso: a sign-in callback couldn't be read: {error}");
                        }
                        slot.fetch_sub(1, Ordering::AcqRel);
                    });
                if spawned.is_err() {
                    serving.fetch_sub(1, Ordering::AcqRel);
                }
            }
        })?;
    Ok(())
}

/// A connection that gives up at `deadline`: each read or write may wait
/// only for the time left, so trickled bytes can't stretch it.
struct Until {
    stream: TcpStream,
    deadline: Instant,
}

impl Until {
    fn left(&self) -> std::io::Result<Duration> {
        let left = self.deadline.saturating_duration_since(Instant::now());
        if left.is_zero() {
            Err(std::io::ErrorKind::TimedOut.into())
        } else {
            Ok(left)
        }
    }
}

impl std::io::Read for Until {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.stream.set_read_timeout(Some(self.left()?))?;
        self.stream.read(buf)
    }
}

impl std::io::Write for Until {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.stream.set_write_timeout(Some(self.left()?))?;
        self.stream.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.stream.flush()
    }
}

fn serve(
    stream: TcpStream,
    endpoint: &Endpoint,
    router: &Router,
    files: &FileSink,
    captures: &CaptureSink,
    deadline: Instant,
) -> std::io::Result<()> {
    if !stream.peer_addr()?.ip().is_loopback() {
        return Ok(());
    }
    let mut writer = Until {
        stream: stream.try_clone()?,
        deadline,
    };
    writer.write_all(format!("{}\n", endpoint.server_token).as_bytes())?;
    let mut reader = BufReader::new(Until { stream, deadline }.take(MAX_OPEN_MESSAGE));
    let mut client_token = String::new();
    reader.read_line(&mut client_token)?;
    if !same(client_token.trim_end(), &endpoint.client_token) {
        return Ok(());
    }
    // A capture is one long line of JSON; the other kinds stay short.
    if reader.fill_buf()?.first() == Some(&b'{') {
        reader
            .get_mut()
            .set_limit(capture_request::MAX_LINE as u64 + 2);
        let answer = match read_capture(&mut reader)? {
            Err(reason) => format!("no {reason}"),
            Ok(request) => match captures(request) {
                Ok(id) => format!("ok {id}"),
                Err(reason) => format!("no {}", reason.replace(['\n', '\r'], " ")),
            },
        };
        return writer.write_all(format!("{answer}\n").as_bytes());
    }
    let mut kind = String::new();
    reader.read_line(&mut kind)?;
    if kind.trim_end() == OPEN_STICKY {
        let answer = match read_sticky(&mut reader)? {
            None => "no".to_owned(),
            Some(request) => match STICKIES.get() {
                // Still starting: the forwarder tries again.
                None => "no".to_owned(),
                Some(sink) => match sink(request) {
                    Ok(()) => "ok".to_owned(),
                    Err(reason) => format!("no {}", reason.replace(['\n', '\r'], " ")),
                },
            },
        };
        return writer.write_all(format!("{answer}\n").as_bytes());
    }
    let accepted = if kind.trim_end() == OPEN_FILES {
        match read_paths(&mut reader)? {
            Some(paths) => {
                let paths = crate::open_documents::note_files(paths);
                let any = !paths.is_empty();
                if any {
                    files(paths);
                }
                any
            }
            None => false,
        }
    } else {
        kind.len() as u64 <= MAX_MESSAGE && router.deliver(kind.trim_end()) == Delivery::Delivered
    };
    writer.write_all(if accepted { b"ok\n" } else { b"no\n" })
}

/// A capture message line. `Err` is the reason it was refused: too long,
/// not text, or not a capture.
fn read_capture(
    reader: &mut impl std::io::BufRead,
) -> std::io::Result<Result<CaptureRequest, String>> {
    let mut line = Vec::new();
    reader.read_until(b'\n', &mut line)?;
    if line.last() != Some(&b'\n') && line.len() > capture_request::MAX_LINE {
        return Ok(Err(
            "That message is too large for Kanso. Send a smaller note.".into(),
        ));
    }
    let Ok(line) = String::from_utf8(line) else {
        return Ok(Err(
            "Kanso couldn't read that message: it isn't text. `kanso capture` sends text.".into(),
        ));
    };
    Ok(CaptureRequest::from_line(
        line.trim_end_matches(['\r', '\n']),
    ))
}

/// The count line and that many absolute paths, one per line. `None` for a
/// count over [`MAX_FILES`], a path that is too long or isn't absolute.
fn read_paths(reader: &mut impl std::io::BufRead) -> std::io::Result<Option<Vec<PathBuf>>> {
    let mut count = String::new();
    reader.read_line(&mut count)?;
    let Some(count) = count
        .trim_end()
        .parse::<usize>()
        .ok()
        .filter(|n| *n <= MAX_FILES)
    else {
        return Ok(None);
    };
    let mut paths = Vec::with_capacity(count);
    for _ in 0..count {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        let line = line.strip_suffix('\n').unwrap_or(&line);
        let line = line.strip_suffix('\r').unwrap_or(line);
        let path = PathBuf::from(line);
        if line.is_empty() || line.len() > MAX_PATH || !path.is_absolute() {
            return Ok(None);
        }
        paths.push(path);
    }
    Ok(Some(paths))
}

/// One line without its line break.
fn read_line_trimmed(reader: &mut impl std::io::BufRead) -> std::io::Result<String> {
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let line = line.strip_suffix('\n').unwrap_or(&line);
    Ok(line.strip_suffix('\r').unwrap_or(line).to_owned())
}

/// A sticky message after its kind line: the colour (or an empty line),
/// the notebook (or an empty line), the text's length in bytes, and the
/// text. `None` for a colour that isn't one, a notebook or text too long,
/// or text that isn't UTF-8.
fn read_sticky(reader: &mut impl std::io::BufRead) -> std::io::Result<Option<StickyRequest>> {
    let color = read_line_trimmed(reader)?;
    let color = match color.as_str() {
        "" => None,
        name => match crate::config::StickyColor::parse(name) {
            Some(color) => Some(color),
            None => return Ok(None),
        },
    };
    let book = read_line_trimmed(reader)?;
    if book.len() > MAX_BOOK {
        return Ok(None);
    }
    let book = (!book.trim().is_empty()).then_some(book);
    let Some(length) = read_line_trimmed(reader)?
        .parse::<usize>()
        .ok()
        .filter(|n| *n <= MAX_STICKY_TEXT)
    else {
        return Ok(None);
    };
    let mut text = vec![0; length];
    reader.read_exact(&mut text)?;
    let Ok(text) = String::from_utf8(text) else {
        return Ok(None);
    };
    Ok(Some(StickyRequest { text, book, color }))
}

/// Why a URL couldn't be forwarded to the running app.
#[derive(Debug, PartialEq, Eq)]
pub enum ForwardError {
    /// No running Kanso has written an endpoint, or it isn't listening.
    NotRunning,
    /// Something answered on the port that isn't the Kanso that wrote the
    /// file; nothing was sent to it.
    WrongPeer,
    /// The running Kanso has no flow waiting for this callback.
    NotWaiting,
    /// The running Kanso opened none of the files.
    Refused,
    /// A path that can't be sent: not absolute, too long, or with a line
    /// break.
    BadPath,
    /// A sticky note too long to send, or a notebook name with a line
    /// break.
    BadSticky,
    /// A note over the limits of the link, or without a title; says why and
    /// what to do.
    BadCapture(String),
    /// The running Kanso said why it refused.
    Said(String),
    Io(String),
}

impl std::fmt::Display for ForwardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ForwardError::NotRunning => f.write_str("Kanso isn't running"),
            ForwardError::WrongPeer => f.write_str("another program answered instead of Kanso"),
            ForwardError::NotWaiting => f.write_str("Kanso isn't waiting for this sign-in"),
            ForwardError::Refused => f.write_str("Kanso didn't open the files"),
            ForwardError::BadPath => f.write_str("a file's path can't be passed to Kanso"),
            ForwardError::BadSticky => write!(
                f,
                "a sticky note can have at most {} KB of text, and a notebook's name no line break",
                MAX_STICKY_TEXT / 1024
            ),
            ForwardError::BadCapture(reason) | ForwardError::Said(reason) => f.write_str(reason),
            ForwardError::Io(error) => f.write_str(error),
        }
    }
}

/// Sends `url` to the Kanso running with `data_dir`.
pub fn forward(data_dir: &Path, url: &str) -> Result<(), ForwardError> {
    if url.contains(['\n', '\r']) {
        return Err(ForwardError::NotWaiting);
    }
    send(data_dir, &format!("{url}\n"))
        .map(|_| ())
        .map_err(|error| match error {
            ForwardError::Refused => ForwardError::NotWaiting,
            other => other,
        })
}

/// Asks the Kanso running with `data_dir` to open `paths` (absolute), in
/// messages of at most [`MAX_FILES`]. The app opens only those that are
/// existing note files.
pub fn forward_files(data_dir: &Path, paths: &[PathBuf]) -> Result<(), ForwardError> {
    let lines: Vec<&str> = paths
        .iter()
        .map(|path| {
            path.to_str()
                .filter(|text| {
                    path.is_absolute() && text.len() <= MAX_PATH && !text.contains(['\n', '\r'])
                })
                .ok_or(ForwardError::BadPath)
        })
        .collect::<Result<_, _>>()?;
    for batch in lines.chunks(MAX_FILES) {
        let mut body = format!("{OPEN_FILES}\n{}\n", batch.len());
        for line in batch {
            body.push_str(line);
            body.push('\n');
        }
        send(data_dir, &body)?;
    }
    Ok(())
}

/// Asks the Kanso running with `data_dir` to make the sticky note
/// `request` and show it.
pub fn forward_sticky(data_dir: &Path, request: &StickyRequest) -> Result<(), ForwardError> {
    let book = request.book.as_deref().unwrap_or("");
    if request.text.len() > MAX_STICKY_TEXT || book.len() > MAX_BOOK || book.contains(['\n', '\r'])
    {
        return Err(ForwardError::BadSticky);
    }
    let color = request.color.map_or("", |color| color.as_str());
    let body = format!(
        "{OPEN_STICKY}\n{color}\n{book}\n{}\n{}",
        request.text.len(),
        request.text
    );
    send(data_dir, &body).map(|_| ())
}

/// Asks the Kanso running with `data_dir` to add the note `request`.
/// Returns the new note's ID. This is all the link does with notes: it
/// has no way to read one.
pub fn forward_capture(data_dir: &Path, request: &CaptureRequest) -> Result<String, ForwardError> {
    let line = request.to_line().map_err(ForwardError::BadCapture)?;
    let id = send(data_dir, &format!("{line}\n"))?;
    if id.is_empty() {
        return Err(ForwardError::Said(
            "Kanso added the note but didn't say which. Look for it in the notebook.".into(),
        ));
    }
    Ok(id)
}

/// Connects to the running app, proves it is Kanso by its greeting, and
/// sends the client token and then `body`.
/// Returns what the app said after `ok`, trimmed: a note's ID for a
/// capture, nothing for the rest.
fn send(data_dir: &Path, body: &str) -> Result<String, ForwardError> {
    let text =
        std::fs::read_to_string(endpoint_path(data_dir)).map_err(|_| ForwardError::NotRunning)?;
    let endpoint = Endpoint::parse(&text).ok_or(ForwardError::NotRunning)?;
    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, endpoint.port));
    let stream =
        TcpStream::connect_timeout(&address, IO_TIMEOUT).map_err(|_| ForwardError::NotRunning)?;
    let io = |error: std::io::Error| ForwardError::Io(error.to_string());
    stream.set_read_timeout(Some(IO_TIMEOUT)).map_err(io)?;
    stream.set_write_timeout(Some(IO_TIMEOUT)).map_err(io)?;
    let mut writer = stream.try_clone().map_err(io)?;
    let mut reader = BufReader::new(stream.take(MAX_MESSAGE));
    let mut greeting = String::new();
    reader
        .read_line(&mut greeting)
        .map_err(|_| ForwardError::WrongPeer)?;
    if !same(greeting.trim_end(), &endpoint.server_token) {
        return Err(ForwardError::WrongPeer);
    }
    writer
        .write_all(format!("{}\n{body}", endpoint.client_token).as_bytes())
        .map_err(io)?;
    let mut answer = String::new();
    reader.read_line(&mut answer).map_err(io)?;
    match answer.trim_end() {
        "ok" => Ok(String::new()),
        answer if answer.starts_with("ok ") => Ok(answer[3..].trim().to_owned()),
        answer => match answer.strip_prefix("no ") {
            Some(reason) if !reason.trim().is_empty() => {
                Err(ForwardError::Said(reason.trim().to_owned()))
            }
            _ => Err(ForwardError::Refused),
        },
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    fn b64_32(byte: u8) -> String {
        use base64::Engine as _;
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode([byte; 32])
    }

    fn callback(flow: &str) -> String {
        format!("kanso://auth/apple?flow={flow}&s={}", b64_32(9))
    }

    fn no_files() -> FileSink {
        Arc::new(|_| {})
    }

    fn leaked() -> &'static Router {
        Box::leak(Box::new(Router::new()))
    }

    #[test]
    fn callback_arguments_are_told_from_commands() {
        assert!(is_callback_argument(&callback(&b64_32(1))));
        assert!(is_callback_argument("KANSO://auth/apple"));
        for not in [
            "serve",
            "kanso",
            "kanso:",
            "kanso://",
            "https://kanso",
            "-psn_0_1",
        ] {
            assert!(!is_callback_argument(not), "{not}");
        }
    }

    /// The OS starts `kanso <url>`: the URL arrives as the command, which
    /// `main` forwards to the default data folder instead of running. A link
    /// with anything beside it, before or after, is refused, so no flag can
    /// pick the folder it's forwarded to.
    #[test]
    fn a_link_counts_only_as_the_whole_command_line() {
        let url = callback(&b64_32(1));
        let parsed = crate::parse_args_from([url.clone()]).unwrap();
        assert_eq!(parsed.command.as_deref(), Some(url.as_str()));
        assert_eq!(parsed.data_dir, crate::paths::default_data_dir());
        assert!(parsed.rest.is_empty() && !parsed.after_update);
        for args in [
            vec!["--data-dir".to_owned(), "/tmp/k".to_owned(), url.clone()],
            vec![url.clone(), "--data-dir".to_owned(), "/tmp/k".to_owned()],
            vec![url.clone(), "--after-update".to_owned()],
            vec!["--after-update".to_owned(), url.clone()],
            vec![url.clone(), url.clone()],
        ] {
            assert!(crate::parse_args_from(args.clone()).is_err(), "{args:?}");
        }
        // A link that is an argument of another command is that command's.
        let parsed = crate::parse_args_from(["import".to_owned(), url.clone()]).unwrap();
        assert_eq!(parsed.command.as_deref(), Some("import"));
    }

    /// The desktop entry and the installer register the scheme the router
    /// expects, passing the URL as one argument.
    #[test]
    fn the_scheme_is_registered_where_the_os_looks() {
        let desktop = include_str!("../../../assets/linux/kanso.desktop");
        assert!(desktop.contains("MimeType=x-scheme-handler/kanso;"));
        assert!(desktop.lines().any(|line| line == "Exec=kanso %U"));
        let installer = include_str!("../../../packaging/windows/kanso.iss");
        assert!(installer.contains(r#"Subkey: "Software\Classes\kanso""#));
        assert!(installer.contains(r#"ValueName: "URL Protocol""#));
        assert!(installer.contains(r#"""{app}\kanso.exe"" ""%1"""#));
    }

    #[test]
    fn a_callback_reaches_only_the_flow_waiting_for_it_once() {
        let router = Router::new();
        let flow = b64_32(1);
        let waiting = router.wait_for(&flow, Instant::now() + Duration::from_secs(60));
        // Another flow's callback, and something that isn't a callback.
        assert_eq!(router.deliver(&callback(&b64_32(2))), Delivery::NotWaiting);
        assert_eq!(
            router.deliver("kanso://auth/apple?flow=short&s=x"),
            Delivery::NotACallback
        );
        assert_eq!(
            router.deliver(&format!(
                "https://evil.example/auth/apple?flow={flow}&s={}",
                b64_32(9)
            )),
            Delivery::NotACallback
        );
        assert_eq!(router.deliver(&callback(&flow)), Delivery::Delivered);
        assert_eq!(waiting.try_recv().unwrap().unwrap(), callback(&flow));
        // Answered: a replay finds nobody waiting.
        assert_eq!(router.deliver(&callback(&flow)), Delivery::NotWaiting);
        assert_eq!(router.waiting(), 0);
    }

    #[test]
    fn an_expired_or_abandoned_wait_takes_nothing() {
        let router = Router::new();
        let flow = b64_32(3);
        let start = Instant::now();
        let _waiting = router.wait_for(&flow, start + Duration::from_secs(5));
        assert_eq!(
            router.deliver_at(&callback(&flow), start + Duration::from_secs(6)),
            Delivery::NotWaiting
        );
        assert_eq!(router.waiting(), 0);

        let dropped = router.wait_for(&flow, start + Duration::from_secs(60));
        drop(dropped);
        assert_eq!(router.deliver(&callback(&flow)), Delivery::NotWaiting);
    }

    #[test]
    fn no_wait_outlasts_the_cap() {
        let router = Router::new();
        let flow = b64_32(4);
        let _waiting = router.wait_for(&flow, Instant::now() + Duration::from_secs(86_400));
        assert_eq!(
            router.deliver_at(
                &callback(&flow),
                Instant::now() + MAX_WAIT + Duration::from_secs(1)
            ),
            Delivery::NotWaiting
        );
    }

    #[test]
    fn a_forwarded_callback_reaches_the_running_app() {
        let dir = tempfile::tempdir().unwrap();
        let router = leaked();
        let flow = b64_32(5);
        let waiting = router.wait_for(&flow, Instant::now() + Duration::from_secs(60));
        listen(dir.path(), router, no_files()).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            let mode = std::fs::metadata(endpoint_path(dir.path()))
                .unwrap()
                .permissions()
                .mode();
            assert_eq!(mode & 0o777, 0o600);
        }
        assert_eq!(
            forward(dir.path(), &callback(&b64_32(6))),
            Err(ForwardError::NotWaiting)
        );
        assert_eq!(forward(dir.path(), &callback(&flow)), Ok(()));
        assert_eq!(waiting.try_recv().unwrap().unwrap(), callback(&flow));
    }

    #[test]
    fn nothing_is_sent_without_a_running_kanso_or_to_another_listener() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            forward(dir.path(), &callback(&b64_32(7))),
            Err(ForwardError::NotRunning)
        );
        // A stale file whose port another program now holds: it doesn't
        // know the greeting, so the URL is never written to it.
        let impostor = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let endpoint = Endpoint::new(impostor.local_addr().unwrap().port());
        write_endpoint(&endpoint_path(dir.path()), &endpoint).unwrap();
        let received = std::thread::spawn(move || {
            let (mut stream, _) = impostor.accept().unwrap();
            stream.write_all(b"not-the-token\n").unwrap();
            stream
                .set_read_timeout(Some(Duration::from_millis(500)))
                .unwrap();
            let mut seen = Vec::new();
            stream.read_to_end(&mut seen).ok();
            seen
        });
        assert_eq!(
            forward(dir.path(), &callback(&b64_32(7))),
            Err(ForwardError::WrongPeer)
        );
        assert!(received.join().unwrap().is_empty());
    }

    #[test]
    fn a_forwarder_without_the_token_is_ignored() {
        let dir = tempfile::tempdir().unwrap();
        let router = leaked();
        let flow = b64_32(8);
        let waiting = router.wait_for(&flow, Instant::now() + Duration::from_secs(60));
        listen(dir.path(), router, no_files()).unwrap();
        let endpoint =
            Endpoint::parse(&std::fs::read_to_string(endpoint_path(dir.path())).unwrap()).unwrap();
        let mut stream = TcpStream::connect((Ipv4Addr::LOCALHOST, endpoint.port)).unwrap();
        stream
            .write_all(format!("{}\n{}\n", "0".repeat(64), callback(&flow)).as_bytes())
            .unwrap();
        let mut answer = String::new();
        stream.read_to_string(&mut answer).ok();
        assert!(!answer.contains("ok"));
        assert!(waiting.try_recv().is_err());
    }

    /// A local client that connects and says nothing doesn't hold up the
    /// real forward behind it.
    #[test]
    fn a_silent_connection_does_not_hold_up_a_forward() {
        let dir = tempfile::tempdir().unwrap();
        let router = leaked();
        let flow = b64_32(10);
        let waiting = router.wait_for(&flow, Instant::now() + Duration::from_secs(60));
        listen(dir.path(), router, no_files()).unwrap();
        let endpoint =
            Endpoint::parse(&std::fs::read_to_string(endpoint_path(dir.path())).unwrap()).unwrap();
        let _silent: Vec<TcpStream> = (0..3)
            .map(|_| TcpStream::connect((Ipv4Addr::LOCALHOST, endpoint.port)).unwrap())
            .collect();
        std::thread::sleep(Duration::from_millis(50));
        let started = Instant::now();
        assert_eq!(forward(dir.path(), &callback(&flow)), Ok(()));
        assert!(started.elapsed() < Duration::from_secs(1));
        assert_eq!(waiting.try_recv().unwrap().unwrap(), callback(&flow));
    }

    /// A client trickling a byte at a time is cut off when its time is up,
    /// not kept alive by each byte.
    #[test]
    fn a_trickling_connection_gets_the_timeout_in_all() {
        let dir = tempfile::tempdir().unwrap();
        listen(dir.path(), leaked(), no_files()).unwrap();
        let endpoint =
            Endpoint::parse(&std::fs::read_to_string(endpoint_path(dir.path())).unwrap()).unwrap();
        let stream = TcpStream::connect((Ipv4Addr::LOCALHOST, endpoint.port)).unwrap();
        let started = Instant::now();
        let mut reader = stream.try_clone().unwrap();
        let closed = std::thread::spawn(move || {
            let mut seen = Vec::new();
            reader.read_to_end(&mut seen).ok();
            Instant::now()
        });
        let mut writer = stream;
        let trickle = endpoint.client_token.clone();
        for byte in trickle.bytes().take(20) {
            if writer.write_all(&[byte]).is_err() || closed.is_finished() {
                break;
            }
            std::thread::sleep(Duration::from_millis(300));
        }
        let closed_at = closed.join().unwrap();
        let lasted = closed_at - started;
        assert!(
            lasted >= IO_TIMEOUT - Duration::from_millis(100),
            "{lasted:?}"
        );
        assert!(
            lasted < IO_TIMEOUT + Duration::from_millis(1500),
            "{lasted:?}"
        );
    }

    /// A sticky message after its kind line, as `forward_sticky` writes it.
    fn sticky_message(color: &str, book: &str, text: &str) -> Vec<u8> {
        format!("{color}\n{book}\n{}\n{text}", text.len()).into_bytes()
    }

    #[test]
    fn a_sticky_message_reads_back() {
        let read = |bytes: Vec<u8>| read_sticky(&mut std::io::Cursor::new(bytes)).unwrap();
        assert_eq!(
            read(sticky_message("pink", "Work", "docker ps -a\nline two")),
            Some(StickyRequest {
                text: "docker ps -a\nline two".into(),
                book: Some("Work".into()),
                color: Some(crate::config::StickyColor::Pink),
            })
        );
        // No colour and no notebook: the defaults.
        assert_eq!(
            read(sticky_message("", "", "hi")),
            Some(StickyRequest {
                text: "hi".into(),
                book: None,
                color: None,
            })
        );
        // A colour that isn't one, text past the limit, a notebook name
        // too long, and bytes that aren't text: refused.
        assert_eq!(read(sticky_message("teal", "", "hi")), None);
        assert_eq!(
            read(format!("\n\n{}\n", MAX_STICKY_TEXT + 1).into_bytes()),
            None
        );
        assert_eq!(
            read(sticky_message("", &"b".repeat(MAX_BOOK + 1), "hi")),
            None
        );
        let mut bad = b"\n\n2\n".to_vec();
        bad.extend([0xff, 0xfe]);
        assert_eq!(read(bad), None);
        // Shorter than it says: an error, not a note.
        assert!(read_sticky(&mut std::io::Cursor::new(b"\n\n9\nshort".to_vec())).is_err());
    }

    #[test]
    fn a_sticky_too_long_is_not_sent() {
        let dir = tempfile::tempdir().unwrap();
        let long = StickyRequest {
            text: "x".repeat(MAX_STICKY_TEXT + 1),
            book: None,
            color: None,
        };
        assert_eq!(
            forward_sticky(dir.path(), &long),
            Err(ForwardError::BadSticky)
        );
        let broken_book = StickyRequest {
            text: "x".into(),
            book: Some("a\nb".into()),
            color: None,
        };
        assert_eq!(
            forward_sticky(dir.path(), &broken_book),
            Err(ForwardError::BadSticky)
        );
        // Nothing listening: not running.
        let fine = StickyRequest {
            text: "x".into(),
            book: None,
            color: None,
        };
        assert_eq!(
            forward_sticky(dir.path(), &fine),
            Err(ForwardError::NotRunning)
        );
    }

    /// A sink that keeps what the app was asked to open.
    fn collecting() -> (FileSink, Arc<Mutex<Vec<PathBuf>>>) {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let sink = seen.clone();
        (
            Arc::new(move |paths| sink.lock().unwrap().extend(paths)),
            seen,
        )
    }

    fn running_with_files() -> (tempfile::TempDir, Endpoint, Arc<Mutex<Vec<PathBuf>>>) {
        let dir = tempfile::tempdir().unwrap();
        let (sink, seen) = collecting();
        listen(dir.path(), leaked(), sink).unwrap();
        let endpoint =
            Endpoint::parse(&std::fs::read_to_string(endpoint_path(dir.path())).unwrap()).unwrap();
        (dir, endpoint, seen)
    }

    fn note_in(dir: &Path, name: &str) -> PathBuf {
        let path = dir.join(name);
        std::fs::write(&path, "# note").unwrap();
        path
    }

    /// Never the fixed port the local API uses.
    #[test]
    fn the_link_does_not_use_the_api_port() {
        let (_dir, endpoint, _) = running_with_files();
        assert_ne!(endpoint.port, 19840);
    }

    #[test]
    fn forwarded_files_reach_the_running_app() {
        let (dir, _, seen) = running_with_files();
        let a = note_in(dir.path(), "a.md");
        let b = note_in(dir.path(), "b b.txt");
        assert_eq!(forward_files(dir.path(), &[a.clone(), b.clone()]), Ok(()));
        assert_eq!(*seen.lock().unwrap(), vec![a, b]);
    }

    /// Whatever is sent, only existing note files are opened; a message with
    /// none of them is refused.
    #[test]
    fn forwarded_paths_open_only_existing_note_files() {
        let (dir, _, seen) = running_with_files();
        let good = note_in(dir.path(), "good.md");
        let image = note_in(dir.path(), "picture.png");
        let missing = dir.path().join("missing.md");
        let folder = dir.path().join("folder.md");
        std::fs::create_dir(&folder).unwrap();
        assert_eq!(
            forward_files(dir.path(), &[image, missing.clone(), folder.clone()]),
            Err(ForwardError::Refused)
        );
        assert!(seen.lock().unwrap().is_empty());
        assert_eq!(
            forward_files(dir.path(), &[missing, good.clone(), folder]),
            Ok(())
        );
        assert_eq!(*seen.lock().unwrap(), vec![good]);
    }

    #[test]
    fn paths_that_cannot_be_sent_are_refused_before_connecting() {
        let dir = tempfile::tempdir().unwrap();
        for bad in [
            PathBuf::from("relative/a.md"),
            PathBuf::from("/tmp/a\nb.md"),
            PathBuf::from(format!("/tmp/{}.md", "x".repeat(MAX_PATH))),
        ] {
            assert_eq!(
                forward_files(dir.path(), &[bad]),
                Err(ForwardError::BadPath)
            );
        }
    }

    /// More than the cap goes in several messages, all opened.
    #[test]
    fn many_files_go_in_batches() {
        let (dir, _, seen) = running_with_files();
        let paths: Vec<PathBuf> = (0..MAX_FILES * 2 + 3)
            .map(|n| note_in(dir.path(), &format!("{n}.md")))
            .collect();
        assert_eq!(forward_files(dir.path(), &paths), Ok(()));
        assert_eq!(*seen.lock().unwrap(), paths);
    }

    /// Speaks the protocol by hand, as an attacker with the wrong token or
    /// an oversized message would.
    fn raw_exchange(endpoint: &Endpoint, token: &str, body: &str) -> String {
        let mut stream = TcpStream::connect((Ipv4Addr::LOCALHOST, endpoint.port)).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut greeting = String::new();
        reader.read_line(&mut greeting).unwrap();
        stream.write_all(format!("{token}\n{body}").as_bytes()).ok();
        let mut answer = String::new();
        reader.read_line(&mut answer).ok();
        answer
    }

    #[test]
    fn files_are_accepted_only_with_the_token() {
        let (dir, endpoint, seen) = running_with_files();
        let good = note_in(dir.path(), "good.md");
        let body = format!("open-files\n1\n{}\n", good.display());
        assert!(!raw_exchange(&endpoint, &"0".repeat(64), &body).contains("ok"));
        assert!(!raw_exchange(&endpoint, "", &body).contains("ok"));
        assert!(!raw_exchange(&endpoint, &endpoint.server_token, &body).contains("ok"));
        assert!(seen.lock().unwrap().is_empty());
        assert!(raw_exchange(&endpoint, &endpoint.client_token, &body).starts_with("ok"));
        assert_eq!(*seen.lock().unwrap(), vec![good]);
    }

    #[test]
    fn a_message_over_the_cap_opens_nothing() {
        let (dir, endpoint, seen) = running_with_files();
        let good = note_in(dir.path(), "good.md");
        let line = good.display().to_string();
        // One more file than a message may carry.
        let body = format!(
            "open-files\n{}\n{}",
            MAX_FILES + 1,
            format!("{line}\n").repeat(MAX_FILES + 1)
        );
        assert!(!raw_exchange(&endpoint, &endpoint.client_token, &body).starts_with("ok"));
        // A relative path, and a count that isn't a number.
        let body = "open-files\n1\nrelative.md\n";
        assert!(!raw_exchange(&endpoint, &endpoint.client_token, body).starts_with("ok"));
        let body = "open-files\nmany\n";
        assert!(!raw_exchange(&endpoint, &endpoint.client_token, body).starts_with("ok"));
        assert!(seen.lock().unwrap().is_empty());
    }

    /// A URL line is never taken for the open-files kind, and a sign-in
    /// callback still needs its flow.
    #[test]
    fn urls_still_go_to_the_router_beside_files() {
        let dir = tempfile::tempdir().unwrap();
        let router = leaked();
        let flow = b64_32(11);
        let waiting = router.wait_for(&flow, Instant::now() + Duration::from_secs(60));
        let (sink, seen) = collecting();
        listen(dir.path(), router, sink).unwrap();
        assert_eq!(forward(dir.path(), &callback(&flow)), Ok(()));
        assert!(waiting.try_recv().is_ok());
        assert!(seen.lock().unwrap().is_empty());
    }

    /// A running link whose capture sink keeps what it is asked to add and
    /// answers with an ID of its own.
    fn running_with_captures() -> (tempfile::TempDir, Endpoint, Arc<Mutex<Vec<CaptureRequest>>>) {
        let dir = tempfile::tempdir().unwrap();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let sink: CaptureSink = {
            let seen = seen.clone();
            Arc::new(move |request| {
                let mut seen = seen.lock().unwrap();
                seen.push(request);
                Ok(format!("note-{}", seen.len()))
            })
        };
        listen_with(dir.path(), leaked(), no_files(), sink).unwrap();
        let endpoint =
            Endpoint::parse(&std::fs::read_to_string(endpoint_path(dir.path())).unwrap()).unwrap();
        (dir, endpoint, seen)
    }

    #[test]
    fn a_captured_note_reaches_the_app_and_its_id_comes_back() {
        let (dir, _, seen) = running_with_captures();
        let request = CaptureRequest {
            title: "Buy milk".into(),
            body: "and\neggs ✓".into(),
            book: Some("Ideas".into()),
        };
        assert_eq!(forward_capture(dir.path(), &request), Ok("note-1".into()));
        let inbox = CaptureRequest {
            book: None,
            ..request.clone()
        };
        assert_eq!(forward_capture(dir.path(), &inbox), Ok("note-2".into()));
        assert_eq!(*seen.lock().unwrap(), vec![request, inbox]);
    }

    #[test]
    fn a_capture_needs_the_token_and_a_sink_that_says_no_is_heard() {
        let (dir, endpoint, seen) = running_with_captures();
        let line = CaptureRequest {
            title: "x".into(),
            body: String::new(),
            book: None,
        }
        .to_line()
        .unwrap();
        let body = format!("{line}\n");
        assert!(!raw_exchange(&endpoint, &"0".repeat(64), &body).starts_with("ok"));
        assert!(!raw_exchange(&endpoint, &endpoint.server_token, &body).starts_with("ok"));
        assert!(seen.lock().unwrap().is_empty());
        assert!(raw_exchange(&endpoint, &endpoint.client_token, &body).starts_with("ok note-1"));
        // The app's reason comes back to the forwarder.
        let refusing = tempfile::tempdir().unwrap();
        let sink: CaptureSink = Arc::new(|_| Err("That notebook is locked.\nUnlock it.".into()));
        listen_with(refusing.path(), leaked(), no_files(), sink).unwrap();
        let request = CaptureRequest {
            title: "x".into(),
            body: String::new(),
            book: None,
        };
        assert_eq!(
            forward_capture(refusing.path(), &request),
            Err(ForwardError::Said(
                "That notebook is locked. Unlock it.".into()
            ))
        );
        drop(dir);
    }

    #[test]
    fn oversized_and_malformed_captures_add_nothing() {
        let (dir, endpoint, seen) = running_with_captures();
        let token = &endpoint.client_token;
        let too_big = serde_json::json!({
            "kind": "capture",
            "title": "t",
            "body": "x".repeat(capture_request::MAX_BODY + 1),
        });
        let over_the_line = format!(
            "{{\"kind\":\"capture\",\"title\":\"{}",
            "a".repeat(capture_request::MAX_LINE)
        );
        for body in [
            format!("{too_big}\n"),
            format!("{over_the_line}\n"),
            "{\n".to_owned(),
            "{\"kind\":\"capture\"}\n".to_owned(),
            "{\"kind\":\"capture\",\"title\":\"t\",\"body\":\"\",\"extra\":1}\n".to_owned(),
            "{\"kind\":\"capture\",\"title\":\"  \",\"body\":\"\"}\n".to_owned(),
        ] {
            let answer = raw_exchange(&endpoint, token, &body);
            assert!(!answer.starts_with("ok"), "{answer}");
        }
        // Not text.
        let mut stream = TcpStream::connect((Ipv4Addr::LOCALHOST, endpoint.port)).unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut greeting = String::new();
        reader.read_line(&mut greeting).unwrap();
        stream
            .write_all(format!("{token}\n{{\"kind\":\"").as_bytes())
            .unwrap();
        stream.write_all(&[0xff, 0xfe, b'\n']).unwrap();
        let mut answer = String::new();
        reader.read_line(&mut answer).unwrap();
        assert!(answer.starts_with("no "), "{answer}");
        assert!(seen.lock().unwrap().is_empty());
        // The sender checks first, too.
        let big = CaptureRequest {
            title: "t".into(),
            body: "x".repeat(capture_request::MAX_BODY + 1),
            book: None,
        };
        assert!(matches!(
            forward_capture(dir.path(), &big),
            Err(ForwardError::BadCapture(_))
        ));
        assert!(seen.lock().unwrap().is_empty());
    }

    /// The link has no message that reads. Whatever kind arrives, JSON or
    /// a line, and whatever it asks for, nothing is added and nothing but
    /// a refusal comes back.
    #[test]
    fn the_link_has_no_read_message() {
        let (_dir, endpoint, seen) = running_with_captures();
        for kind in [
            "read",
            "get-note",
            "list-notes",
            "search",
            "export",
            "notes",
            "books",
            "capture ",
            "Capture",
            "",
        ] {
            let line = serde_json::json!({"kind": kind, "id": "x", "query": "secret"});
            let answer = raw_exchange(&endpoint, &endpoint.client_token, &format!("{line}\n"));
            assert!(answer.starts_with("no"), "{kind}: {answer}");
            // And as the bare kind line the older messages use.
            let answer = raw_exchange(&endpoint, &endpoint.client_token, &format!("{kind}\n"));
            assert!(!answer.starts_with("ok"), "{kind}: {answer}");
        }
        assert!(seen.lock().unwrap().is_empty());
    }

    #[test]
    fn tokens_differ_each_time() {
        let tokens: std::collections::HashSet<String> = (0..64).map(|_| token()).collect();
        assert_eq!(tokens.len(), 64);
        assert!(tokens.iter().all(|token| is_token(token)));
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(24))]

        /// Through a real listener, with the token: a message of any kind
        /// but a well-formed `capture` is refused and adds nothing.
        #[test]
        fn prop_no_unknown_message_is_taken(
            kind in "[a-zA-Z_ -]{0,20}",
            extra in "[a-z]{0,10}",
            raw in "\\PC{0,120}",
        ) {
            prop_assume!(kind != "capture");
            let (_dir, endpoint, seen) = running_with_captures();
            let json = serde_json::json!({"kind": kind, "q": extra}).to_string();
            let answer = raw_exchange(&endpoint, &endpoint.client_token, &format!("{json}\n"));
            prop_assert!(!answer.starts_with("ok"), "{}", answer);
            let answer = raw_exchange(&endpoint, &endpoint.client_token, &format!("{{{raw}\n"));
            prop_assert!(!answer.starts_with("ok"), "{}", answer);
            prop_assert!(seen.lock().unwrap().is_empty());
        }

        /// A capture of any text comes out the other side as it went in.
        #[test]
        fn prop_a_capture_round_trips_over_the_link(
            title in "\\PC{1,60}",
            body in "\\PC{0,200}",
            book in prop::option::of("\\PC{1,30}"),
        ) {
            prop_assume!(!title.trim().is_empty());
            prop_assume!(book.as_ref().is_none_or(|b| !b.trim().is_empty()));
            let (dir, _endpoint, seen) = running_with_captures();
            let request = CaptureRequest { title, body, book };
            prop_assert!(forward_capture(dir.path(), &request).is_ok());
            prop_assert_eq!(seen.lock().unwrap().clone(), vec![request]);
        }
    }

    proptest! {
        #[test]
        fn prop_the_endpoint_file_round_trips(port in 1u16..) {
            let endpoint = Endpoint::new(port);
            prop_assert!(Endpoint::parse(&endpoint.to_text()) == Some(endpoint));
        }

        #[test]
        fn prop_tokens_compare_equal_only_when_equal(a in "[0-9a-f]{64}", b in "[0-9a-f]{64}") {
            prop_assert_eq!(same(&a, &b), a == b);
            prop_assert!(same(&a, &a));
        }
    }
}
