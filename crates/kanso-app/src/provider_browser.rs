//! The system browser sessions for Sign in with Apple, Google and GitHub
//! (docs/ACCOUNTS.md › Sign in with Apple, Google and GitHub, › App). The
//! app never embeds a web view for sign-in and never sees a provider token:
//! the Worker exchanges the code and verifies the ID token.
//!
//! - **Apple** uses `ASWebAuthenticationSession` on macOS with the `kanso`
//!   callback scheme; the Worker's page redirects to
//!   `kanso://auth/apple?flow=<flow id>&s=<secret>` and the app polls with
//!   its device and that secret.
//! - **Google** is a desktop client with a loopback redirect: the app binds
//!   `127.0.0.1:0`, gives the Worker that exact URI and its PKCE challenge,
//!   opens the URL in the system browser, and answers the one request to
//!   the listener with "Return to Kanso". The verifier stays here.
//! - **GitHub** is the device flow: a screen shows the code and the app opens
//!   `https://github.com/login/device` in the system browser.
//!
//! Nothing here is on the account's critical path in tests: the listener and
//! the parsers are pure enough to exercise without a browser, and
//! [`open_external`] is the only platform call for Google and GitHub.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use kanso_sync::{AppleStart, GithubStart, GoogleStart};

/// What can go wrong opening a browser or waiting for its answer.
#[derive(Debug)]
pub enum BrowserError {
    /// The loopback address couldn't be bound.
    Bind(std::io::Error),
    /// The listener or the browser failed.
    Io(String),
    /// The person dismissed the browser.
    Cancelled,
    /// The provider's absolute flow deadline elapsed.
    Expired,
    /// The provider returned an explicit denial or cancellation.
    ProviderDenied,
    /// The callback wasn't for this flow (`state` didn't match, or a foreign
    /// path).
    WrongCallback,
    /// This platform has no native Apple session.
    Unsupported(&'static str),
}

impl std::fmt::Display for BrowserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BrowserError::Bind(err) => write!(f, "couldn't listen on this Mac: {err}"),
            BrowserError::Io(message) => f.write_str(message),
            BrowserError::Cancelled => f.write_str("the sign-in was closed"),
            BrowserError::Expired => f.write_str("the sign-in expired"),
            BrowserError::ProviderDenied => f.write_str("sign-in was cancelled"),
            BrowserError::WrongCallback => f.write_str("the browser returned another sign-in"),
            BrowserError::Unsupported(what) => write!(f, "{what}"),
        }
    }
}

impl std::error::Error for BrowserError {}

/// The URL the system opener may be given: an https page, or plain http to
/// this device, with no user info, whitespace or control characters, in its
/// parsed form. `kanso-sync` already pins a provider start's page to the
/// provider's host or the server's; this is the second check, so no caller
/// can hand `open`, the Windows URL handler (which also opens files, UNC
/// paths and `search-ms:`/`ms-*:` links) or `xdg-open` (which reads a
/// leading `-` as an option) something else.
pub fn external_url(text: &str) -> Result<String, BrowserError> {
    let refuse = || BrowserError::Io("Kanso won't open that page".into());
    if text.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return Err(refuse());
    }
    let url = url::Url::parse(text).map_err(|_| refuse())?;
    if !url.username().is_empty() || url.password().is_some() {
        return Err(refuse());
    }
    let allowed = match url.scheme() {
        "https" => url.host_str().is_some(),
        "http" => match url.host() {
            Some(url::Host::Domain(host)) => host == "localhost",
            Some(url::Host::Ipv4(ip)) => ip.is_loopback(),
            Some(url::Host::Ipv6(ip)) => ip.is_loopback(),
            None => false,
        },
        _ => false,
    };
    if !allowed {
        return Err(refuse());
    }
    Ok(url.into())
}

/// Opens `url` in the system browser, once [`external_url`] allows it.
/// Never a web view.
pub fn open_external(url: &str) -> Result<(), BrowserError> {
    let url = external_url(url)?;
    open_system(url.as_str())
}

/// Tests run provider flows against `MemoryServer`: nothing opens.
#[cfg(test)]
fn open_system(_url: &str) -> Result<(), BrowserError> {
    Ok(())
}

#[cfg(not(test))]
fn open_system(url: &str) -> Result<(), BrowserError> {
    #[cfg(target_os = "macos")]
    {
        let status = std::process::Command::new("open")
            .arg(url)
            .status()
            .map_err(|err| BrowserError::Io(format!("couldn't open the browser: {err}")))?;
        if status.success() {
            return Ok(());
        }
        Err(BrowserError::Io("couldn't open the browser".into()))
    }
    #[cfg(target_os = "windows")]
    {
        // Not `cmd /C start`: cmd splits the URL at every `&`, and provider
        // URLs have several. The URL handler takes it as one argument.
        let status = std::process::Command::new("rundll32.exe")
            .args(["url.dll,FileProtocolHandler", url])
            .status()
            .map_err(|err| BrowserError::Io(format!("couldn't open the browser: {err}")))?;
        if status.success() {
            return Ok(());
        }
        Err(BrowserError::Io("couldn't open the browser".into()))
    }
    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    {
        let status = std::process::Command::new("xdg-open")
            .arg(url)
            .status()
            .map_err(|err| BrowserError::Io(format!("couldn't open the browser: {err}")))?;
        if status.success() {
            return Ok(());
        }
        Err(BrowserError::Io("couldn't open the browser".into()))
    }
}

// ------------------------------------------------------------------- Apple

/// Opens `start.auth_url` and waits for the `kanso://` callback URL. On
/// macOS that is `ASWebAuthenticationSession`, which delivers the callback
/// to this process only. On Windows and Linux it is the system browser, and
/// the callback comes back through the `kanso` scheme handler and
/// [`crate::url_callback`], which hands it only to this flow while it waits.
/// The flow ID alone grants nothing either way: the caller polls the Worker
/// with its device and proof and the secret the callback URL carries.
pub fn apple_sign_in(
    start: &AppleStart,
) -> Result<async_channel::Receiver<Result<String, BrowserError>>, BrowserError> {
    #[cfg(target_os = "macos")]
    {
        apple::start(&start.auth_url, crate::url_callback::SCHEME)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let deadline = apple_deadline(start.expires_at, kanso_core::now_ms());
        let callback = crate::url_callback::ROUTER.wait_for(&start.flow_id, deadline);
        open_external(&start.auth_url)?;
        Ok(callback)
    }
}

/// When the wait for Apple's callback ends: the flow's own expiry, never
/// later than [`crate::url_callback::MAX_WAIT`].
#[cfg_attr(target_os = "macos", allow(dead_code))]
fn apple_deadline(expires_at_ms: i64, now_ms: i64) -> Instant {
    let left = u64::try_from(expires_at_ms.saturating_sub(now_ms)).unwrap_or(0);
    Instant::now() + Duration::from_millis(left).min(crate::url_callback::MAX_WAIT)
}

/// The flow ID and secret in Apple's callback URL, when it is the callback
/// the Worker's page makes.
pub fn apple_flow(callback: &str) -> Option<kanso_sync::AppleCallback> {
    kanso_sync::apple_callback_flow(callback)
}

#[cfg(target_os = "macos")]
mod apple {
    use std::cell::RefCell;
    use std::sync::atomic::{AtomicU64, Ordering};

    use block2::RcBlock;
    use objc2::rc::Retained;
    use objc2::runtime::{AnyObject, Bool, NSObject};
    use objc2::{MainThreadMarker, MainThreadOnly, class, define_class, msg_send};
    use objc2_app_kit::{NSApplication, NSWindow};
    use objc2_foundation::{NSError, NSString, NSURL};

    use super::BrowserError;

    #[link(name = "AuthenticationServices", kind = "framework")]
    unsafe extern "C" {}

    define_class!(
        /// The window `ASWebAuthenticationSession` may lay its sheet over.
        #[unsafe(super(NSObject))]
        #[thread_kind = MainThreadOnly]
        #[name = "KansoAuthPresenter"]
        #[ivars = ()]
        struct Presenter;

        impl Presenter {
            #[unsafe(method(presentationAnchorForWebAuthenticationSession:))]
            fn anchor(&self, _session: &AnyObject) -> *mut NSWindow {
                // SAFETY: this class is MainThreadOnly, so this is the main
                // thread.
                let mtm = unsafe { MainThreadMarker::new_unchecked() };
                let app = NSApplication::sharedApplication(mtm);
                match app.keyWindow().or_else(|| app.mainWindow()) {
                    Some(window) => &*window as *const NSWindow as *mut NSWindow,
                    None => std::ptr::null_mut(),
                }
            }
        }
    );

    impl Presenter {
        fn new(mtm: MainThreadMarker) -> Retained<Presenter> {
            let this = Presenter::alloc(mtm).set_ivars(());
            // SAFETY: NSObject's init, on a freshly allocated object.
            unsafe { msg_send![super(this), init] }
        }
    }

    #[allow(
        dead_code,
        reason = "held only to keep the session and its presenter alive"
    )]
    struct Live {
        id: u64,
        session: Retained<AnyObject>,
        presenter: Retained<Presenter>,
    }

    thread_local! {
        /// The sessions still waiting for their callback. `presenter` is
        /// weak on the session, so we keep it here too.
        static LIVE: RefCell<Vec<Live>> = const { RefCell::new(Vec::new()) };
    }

    static NEXT: AtomicU64 = AtomicU64::new(1);

    pub fn start(
        auth_url: &str,
        scheme: &str,
    ) -> Result<async_channel::Receiver<Result<String, BrowserError>>, BrowserError> {
        let mtm = MainThreadMarker::new().ok_or(BrowserError::Unsupported(
            "the sign-in sheet needs the main thread",
        ))?;
        // The start's URL as kanso-sync parsed and checked it; the same
        // web-page rule as the system opener, again.
        let auth_url = super::external_url(auth_url)?;
        let url = NSURL::URLWithString(&NSString::from_str(&auth_url))
            .ok_or_else(|| BrowserError::Io("the provider's URL is malformed".into()))?;
        let scheme = NSString::from_str(scheme);
        let presenter = Presenter::new(mtm);
        let id = NEXT.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = async_channel::bounded(1);
        let answer = RcBlock::new(move |callback: *mut NSURL, error: *mut NSError| {
            let answer = if !callback.is_null() {
                // SAFETY: ASWebAuthenticationSession calls this once with an
                // absolute NSURL on success.
                let text: Retained<NSString> = unsafe { msg_send![callback, absoluteString] };
                Ok(text.to_string())
            } else {
                let _ = error;
                Err(BrowserError::Cancelled)
            };
            LIVE.with(|live| live.borrow_mut().retain(|entry| entry.id != id));
            let _ = tx.try_send(answer);
        });
        // SAFETY: the class is present in AuthenticationServices, which is
        // linked above; the initializer retains the completion block.
        let allocated: *mut AnyObject =
            unsafe { msg_send![class!(ASWebAuthenticationSession), alloc] };
        let initialized: *mut AnyObject = unsafe {
            msg_send![
                allocated,
                initWithURL: &*url,
                callbackURLScheme: &*scheme,
                completionHandler: &*answer
            ]
        };
        // SAFETY: the initializer returned a +1 object, or nil on failure.
        let session = unsafe { Retained::from_raw(initialized) }
            .ok_or_else(|| BrowserError::Io("the sign-in sheet didn't open".into()))?;
        // SAFETY: the presenter outlives the session (LIVE keeps it).
        let _: () = unsafe { msg_send![&*session, setPresentationContextProvider: &*presenter] };
        LIVE.with(|live| {
            live.borrow_mut().push(Live {
                id,
                session: session.clone(),
                presenter,
            });
        });
        // SAFETY: starting on the main thread; the answer comes through the
        // completion block.
        let started: Bool = unsafe { msg_send![&*session, start] };
        if !started.as_bool() {
            LIVE.with(|live| live.borrow_mut().retain(|entry| entry.id != id));
            return Err(BrowserError::Io("the sign-in sheet didn't open".into()));
        }
        Ok(rx)
    }
}

// ------------------------------------------------------------------ Google

/// A one-request loopback listener for Google's redirect. Bind it first so
/// its `uri()` can be the Worker's `redirect_uri`, then open the browser and
/// [`Loopback::wait`] for the answer.
pub struct Loopback {
    listener: TcpListener,
    port: u16,
}

impl Loopback {
    /// Binds `127.0.0.1:0`: the system picks a free port.
    pub fn bind() -> Result<Loopback, BrowserError> {
        let listener = TcpListener::bind(("127.0.0.1", 0)).map_err(BrowserError::Bind)?;
        let port = listener.local_addr().map_err(BrowserError::Bind)?.port();
        Ok(Loopback { listener, port })
    }

    /// The exact redirect URI: `http://127.0.0.1:<port>`, nothing after it.
    pub fn uri(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    #[cfg(test)]
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Accepts one request, checks `state`, answers "Return to Kanso" and
    /// returns the `code`. The listener is done either way.
    #[cfg(test)]
    pub fn wait(&self, expected_state: &str, timeout: Duration) -> Result<String, BrowserError> {
        let cancel = AtomicBool::new(false);
        self.wait_until(expected_state, Instant::now() + timeout, &cancel)
    }

    /// Waits for one valid callback without blocking cancellation or the
    /// server-provided absolute expiry deadline.
    ///
    /// A browser may open more than one connection before (or instead of)
    /// sending the callback on one: a speculative connection that never
    /// speaks, an HTTPS attempt before it falls back to `http`, a request
    /// for `/favicon.ico`. Every connection is served side by side, and only
    /// a request for `/` ends the wait; the others are refused or dropped.
    /// Waiting on the first connection alone left the real callback unread
    /// in the backlog until the flow expired.
    pub fn wait_until(
        &self,
        expected_state: &str,
        deadline: Instant,
        cancel: &AtomicBool,
    ) -> Result<String, BrowserError> {
        self.listener
            .set_nonblocking(true)
            .map_err(BrowserError::Bind)?;
        let mut open: Vec<Connection> = Vec::new();
        loop {
            if cancel.load(Ordering::Acquire) {
                return Err(BrowserError::Cancelled);
            }
            if Instant::now() >= deadline {
                return Err(BrowserError::Expired);
            }
            let mut idle = true;
            loop {
                match self.listener.accept() {
                    Ok((stream, _)) => {
                        idle = false;
                        // One that can't be read without blocking is dropped;
                        // the browser tries again.
                        if open.len() < MAX_CONNECTIONS && stream.set_nonblocking(true).is_ok() {
                            open.push(Connection {
                                stream,
                                bytes: Vec::new(),
                            });
                        }
                    }
                    Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => break,
                    Err(err) if err.kind() == std::io::ErrorKind::Interrupted => {}
                    Err(err) => return Err(BrowserError::Bind(err)),
                }
            }
            let mut index = 0;
            while index < open.len() {
                match open[index].read() {
                    Heard::Nothing { progressed } => {
                        idle &= !progressed;
                        index += 1;
                    }
                    Heard::Gone => {
                        open.swap_remove(index);
                    }
                    Heard::Request(request) => {
                        idle = false;
                        let mut connection = open.swap_remove(index);
                        if let Some(answer) =
                            answer_request(&mut connection.stream, &request, expected_state)
                        {
                            return answer;
                        }
                    }
                }
            }
            if idle {
                std::thread::sleep(
                    Duration::from_millis(25)
                        .min(deadline.saturating_duration_since(Instant::now())),
                );
            }
        }
    }
}

/// The most connections the loopback keeps open at once.
const MAX_CONNECTIONS: usize = 16;

/// A connection to the loopback and what it has sent so far.
struct Connection {
    stream: TcpStream,
    bytes: Vec<u8>,
}

/// What reading a [`Connection`] found.
enum Heard {
    /// No whole request yet; `progressed` when bytes came in.
    Nothing { progressed: bool },
    /// Closed, broken, too long or not HTTP: dropped.
    Gone,
    /// A whole request head.
    Request(String),
}

impl Connection {
    fn read(&mut self) -> Heard {
        let mut buffer = [0u8; 512];
        let mut progressed = false;
        loop {
            match self.stream.read(&mut buffer) {
                Ok(0) => return Heard::Gone,
                Ok(read) => {
                    progressed = true;
                    self.bytes.extend_from_slice(&buffer[..read]);
                    if self.bytes.windows(4).any(|window| window == b"\r\n\r\n") {
                        return match String::from_utf8(std::mem::take(&mut self.bytes)) {
                            Ok(request) => Heard::Request(request),
                            Err(_) => Heard::Gone,
                        };
                    }
                    // A TLS hello, or anything else that doesn't start like
                    // a request line, never becomes one.
                    if self.bytes.len() > 16 * 1024 || !self.bytes[0].is_ascii_uppercase() {
                        return Heard::Gone;
                    }
                }
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    return Heard::Nothing { progressed };
                }
                Err(err) if err.kind() == std::io::ErrorKind::Interrupted => {}
                Err(_) => return Heard::Gone,
            }
        }
    }
}

/// Answers one request to the loopback. `Some` ends the wait: the callback
/// at `/`, right or wrong. `None` is a request for something else (the
/// browser's `/favicon.ico`), which is refused while the wait goes on.
fn answer_request(
    stream: &mut TcpStream,
    request: &str,
    expected_state: &str,
) -> Option<Result<String, BrowserError>> {
    // The answer is small: let it go out whole.
    let _ = stream.set_nonblocking(false);
    let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));
    let at_root = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .is_some_and(|target| target == "/" || target.starts_with("/?"));
    if !at_root {
        let _ = respond(stream, 404, "Not found.");
        return None;
    }
    Some(match parse_loopback_callback(request, expected_state) {
        Ok(code) => {
            let _ = respond(stream, 200, "Signed in. Return to Kanso.");
            Ok(code)
        }
        Err(BrowserError::ProviderDenied) => {
            let _ = respond(stream, 200, "Sign-in was cancelled. Return to Kanso.");
            Err(BrowserError::ProviderDenied)
        }
        Err(_) => {
            let _ = respond(
                stream,
                400,
                "That sign-in didn't come from Kanso. Return to Kanso and try again.",
            );
            Err(BrowserError::WrongCallback)
        }
    })
}

fn respond(stream: &mut TcpStream, status: u16, sentence: &str) -> std::io::Result<()> {
    let reason = match status {
        200 => "OK",
        404 => "Not Found",
        _ => "Bad Request",
    };
    let body = format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><title>Kanso</title></head><body><p>{sentence}</p></body></html>"
    );
    write!(
        stream,
        "HTTP/1.1 {status} {reason}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\nCache-Control: no-store\r\n\r\n{body}",
        body.len()
    )?;
    stream.flush()
}

/// The `code` in a loopback request's `GET /?code=…&state=…`, when the
/// request line is well formed and `state` is the one the start returned.
/// Google's codes are percent-encoded; `+` is a space in a query.
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "the loopback parses inline; tests use this")
)]
pub fn parse_loopback_request(request: &str, expected_state: &str) -> Option<String> {
    parse_loopback_callback(request, expected_state).ok()
}

pub fn parse_loopback_callback(
    request: &str,
    expected_state: &str,
) -> Result<String, BrowserError> {
    let line = request.lines().next().ok_or(BrowserError::WrongCallback)?;
    let mut parts = line.split_whitespace();
    let method = parts.next().ok_or(BrowserError::WrongCallback)?;
    let target = parts.next().ok_or(BrowserError::WrongCallback)?;
    let version = parts.next().ok_or(BrowserError::WrongCallback)?;
    if method != "GET" || !version.starts_with("HTTP/") {
        return Err(BrowserError::WrongCallback);
    }
    let (path, query) = target.split_once('?').ok_or(BrowserError::WrongCallback)?;
    if path != "/" {
        return Err(BrowserError::WrongCallback);
    }
    let mut code = None;
    let mut state = None;
    let mut error = None;
    for pair in query.split('&') {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        let value = percent_decode(value);
        match key {
            "code" => code = Some(value),
            "state" => state = Some(value),
            "error" => error = Some(value),
            _ => {}
        }
    }
    if state.as_deref() != Some(expected_state) {
        return Err(BrowserError::WrongCallback);
    }
    if error.is_some() {
        return Err(BrowserError::ProviderDenied);
    }
    code.filter(|code| !code.is_empty())
        .ok_or(BrowserError::WrongCallback)
}

/// `%XX` and `+` as a query decoder; a malformed escape is left as it is.
fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                match (hex_digit(bytes[i + 1]), hex_digit(bytes[i + 2])) {
                    (Some(high), Some(low)) => {
                        out.push(high << 4 | low);
                        i += 3;
                    }
                    _ => {
                        out.push(b'%');
                        i += 1;
                    }
                }
            }
            byte => {
                out.push(byte);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn hex_digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

// ------------------------------------------------------------------ GitHub

/// Opens GitHub's device page; the caller shows `user_code` and polls.
pub fn github_device_page(start: &GithubStart) -> Result<(), BrowserError> {
    open_external(&start.verification_uri)
}

/// Google's start: the loopback first, then open the URL.
pub fn google_device_page(start: &GoogleStart) -> Result<(), BrowserError> {
    open_external(&start.auth_url)
}

// ------------------------------------------------------------- The engine

// no-prop-test: thin adapters from the system browser to the engine's
// `Browser` seam; the flows that call them are tested in `kanso_engine`.
use kanso_engine::account::provider as flow;

impl From<BrowserError> for flow::BrowserError {
    fn from(err: BrowserError) -> Self {
        match err {
            BrowserError::Cancelled => flow::BrowserError::Cancelled,
            BrowserError::Expired => flow::BrowserError::Expired,
            BrowserError::ProviderDenied => flow::BrowserError::ProviderDenied,
            _ => flow::BrowserError::Failed,
        }
    }
}

impl flow::Listener for Loopback {
    fn uri(&self) -> String {
        Loopback::uri(self)
    }

    fn wait_until(
        &self,
        state: &str,
        deadline: Instant,
        cancel: &AtomicBool,
    ) -> Result<String, flow::BrowserError> {
        Loopback::wait_until(self, state, deadline, cancel).map_err(Into::into)
    }
}

/// Apple's callback, as the engine's flow waits for it.
pub struct AppleReceiver(pub async_channel::Receiver<Result<String, BrowserError>>);

impl flow::AppleWait for AppleReceiver {
    fn poll(&self) -> flow::ApplePoll {
        match self.0.try_recv() {
            Ok(result) => flow::ApplePoll::Answer(result.map_err(Into::into)),
            Err(async_channel::TryRecvError::Closed) => flow::ApplePoll::Closed,
            Err(async_channel::TryRecvError::Empty) => flow::ApplePoll::Waiting,
        }
    }
}

/// The system browser and a loopback listener, for the engine's provider
/// sign-in flows.
pub struct Desktop;

impl flow::Browser for Desktop {
    fn listen(&self) -> Result<Box<dyn flow::Listener>, flow::BrowserError> {
        Ok(Box::new(Loopback::bind()?))
    }

    fn open_google(&self, start: &GoogleStart) -> Result<(), flow::BrowserError> {
        google_device_page(start).map_err(Into::into)
    }

    fn open_github(&self, start: &GithubStart) -> Result<(), flow::BrowserError> {
        github_device_page(start).map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_web_pages_reach_the_system_opener() {
        assert_eq!(
            external_url("https://appleid.apple.com/auth/authorize?a=1&b=2").unwrap(),
            "https://appleid.apple.com/auth/authorize?a=1&b=2"
        );
        assert!(external_url("http://127.0.0.1:8787/device").is_ok());
        assert!(external_url("http://localhost:8787/device").is_ok());
        for bad in [
            "file:///C:/Windows/System32/calc.exe",
            "\\\\evil\\share\\run.exe",
            "search-ms:query=x&crumb=location:\\\\evil\\s",
            "ms-settings:",
            "ms-msdt:/id",
            "-aflag",
            "--help",
            "javascript:alert(1)",
            "kanso://auth/apple?flow=x&s=y",
            "http://evil.example/page",
            "https://user:pass@evil.example/",
            " https://example.com/",
            "https://example.com/\npage",
            "",
        ] {
            assert!(external_url(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn the_loopback_parser_needs_the_right_path_and_state() {
        assert_eq!(
            parse_loopback_request(
                "GET /?code=4%2F0abc&state=s1 HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n",
                "s1"
            )
            .as_deref(),
            Some("4/0abc")
        );
        assert_eq!(
            parse_loopback_request("GET /?code=a+b&state=s1 HTTP/1.1\r\n\r\n", "s1").as_deref(),
            Some("a b")
        );
        for bad in [
            "GET /cb?code=x&state=s1 HTTP/1.1\r\n\r\n",
            "GET /?code=x&state=other HTTP/1.1\r\n\r\n",
            "POST /?code=x&state=s1 HTTP/1.1\r\n\r\n",
            "GET /?state=s1 HTTP/1.1\r\n\r\n",
            "GET /?code=&state=s1 HTTP/1.1\r\n\r\n",
            "nonsense",
        ] {
            assert_eq!(parse_loopback_request(bad, "s1"), None, "{bad}");
        }
    }

    #[test]
    fn loopback_oauth_errors_require_the_matching_state_and_are_denials() {
        assert!(matches!(
            parse_loopback_callback("GET /?error=access_denied&state=s1 HTTP/1.1\r\n\r\n", "s1"),
            Err(BrowserError::ProviderDenied)
        ));
        assert!(matches!(
            parse_loopback_callback("GET /?error=server_error&state=s1 HTTP/1.1\r\n\r\n", "s1"),
            Err(BrowserError::ProviderDenied)
        ));
        for wrong in [
            "GET /?error=access_denied&state=wrong HTTP/1.1\r\n\r\n",
            "GET /?error=access_denied HTTP/1.1\r\n\r\n",
        ] {
            assert!(matches!(
                parse_loopback_callback(wrong, "s1"),
                Err(BrowserError::WrongCallback)
            ));
        }
    }

    #[test]
    fn loopback_wait_honors_cancellation_and_expiry_before_accepting() {
        use std::sync::atomic::AtomicBool;

        let loopback = Loopback::bind().unwrap();
        let cancelled = AtomicBool::new(true);
        assert!(matches!(
            loopback.wait_until("state", Instant::now() + Duration::from_secs(1), &cancelled),
            Err(BrowserError::Cancelled)
        ));

        let active = AtomicBool::new(false);
        assert!(matches!(
            loopback.wait_until("state", Instant::now(), &active),
            Err(BrowserError::Expired)
        ));
    }

    #[test]
    fn a_loopback_listener_answers_one_request_and_returns_the_code() {
        let loopback = Loopback::bind().unwrap();
        let uri = loopback.uri();
        assert_eq!(
            kanso_sync::Loopback::parse(&uri),
            Some(kanso_sync::Loopback {
                port: loopback.port()
            })
        );
        let port = loopback.port();
        let waiter = std::thread::spawn(move || {
            let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
            stream
                .write_all(
                    b"GET /?code=4%2Fsecret-code&state=the-state HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n",
                )
                .unwrap();
            let mut answer = String::new();
            stream.read_to_string(&mut answer).unwrap();
            answer
        });
        let code = loopback.wait("the-state", Duration::from_secs(5)).unwrap();
        assert_eq!(code, "4/secret-code");
        let answer = waiter.join().unwrap();
        assert!(answer.starts_with("HTTP/1.1 200"));
        assert!(answer.contains("Return to Kanso"));
    }

    /// The browser's other connections (one that never speaks, an HTTPS
    /// attempt, a favicon) don't keep the callback from being read.
    #[test]
    fn a_loopback_listener_reads_the_callback_past_the_browsers_other_connections() {
        let loopback = Loopback::bind().unwrap();
        let port = loopback.port();
        let waiter = std::thread::spawn(move || {
            let silent = TcpStream::connect(("127.0.0.1", port)).unwrap();
            let mut tls = TcpStream::connect(("127.0.0.1", port)).unwrap();
            tls.write_all(&[0x16, 0x03, 0x01, 0x02, 0x00, 0x01])
                .unwrap();
            let mut icon = TcpStream::connect(("127.0.0.1", port)).unwrap();
            icon.write_all(b"GET /favicon.ico HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n")
                .unwrap();
            let mut refused = String::new();
            icon.read_to_string(&mut refused).unwrap();
            assert!(refused.starts_with("HTTP/1.1 404"), "{refused}");
            let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
            stream
                .write_all(
                    b"GET /?code=the-code&state=the-state HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n",
                )
                .unwrap();
            let mut answer = String::new();
            stream.read_to_string(&mut answer).unwrap();
            drop(silent);
            answer
        });
        let code = loopback.wait("the-state", Duration::from_secs(5)).unwrap();
        assert_eq!(code, "the-code");
        assert!(waiter.join().unwrap().contains("Return to Kanso"));
    }

    #[test]
    fn a_loopback_listener_refuses_another_flows_state() {
        let loopback = Loopback::bind().unwrap();
        let port = loopback.port();
        let waiter = std::thread::spawn(move || {
            let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
            stream
                .write_all(b"GET /?code=x&state=wrong HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n")
                .unwrap();
            let mut answer = String::new();
            stream.read_to_string(&mut answer).unwrap();
            answer
        });
        assert!(matches!(
            loopback.wait("right", Duration::from_secs(5)),
            Err(BrowserError::WrongCallback)
        ));
        assert!(waiter.join().unwrap().starts_with("HTTP/1.1 400"));
    }

    #[test]
    fn apple_callbacks_name_the_flow() {
        let flow = "A".repeat(43);
        let secret = "b".repeat(43);
        let callback = apple_flow(&format!("kanso://auth/apple?flow={flow}&s={secret}")).unwrap();
        assert_eq!(callback.flow, flow);
        assert_eq!(callback.secret, secret);
        assert_eq!(apple_flow(&format!("kanso://auth/apple?flow={flow}")), None);
        assert_eq!(apple_flow("kanso://auth/apple?flow=short"), None);
    }

    /// The browser wait ends with the flow, and never after the cap.
    #[test]
    fn apple_waits_end_with_the_flow() {
        let now = Instant::now();
        let soon = apple_deadline(1_000 + 90_000, 1_000);
        assert!(
            soon >= now + Duration::from_secs(89)
                && soon <= Instant::now() + Duration::from_secs(90)
        );
        assert!(apple_deadline(0, 1_000) <= Instant::now());
        assert!(apple_deadline(i64::MAX, 0) <= Instant::now() + crate::url_callback::MAX_WAIT);
    }
}
