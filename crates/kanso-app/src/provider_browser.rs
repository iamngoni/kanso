//! The system browser sessions for Sign in with Apple, Google and GitHub
//! (docs/ACCOUNTS.md › Sign in with Apple, Google and GitHub, › App). The
//! app never embeds a web view for sign-in and never sees a provider token:
//! the Worker exchanges the code and verifies the ID token.
//!
//! - **Apple** uses `ASWebAuthenticationSession` on macOS with the `kanso`
//!   callback scheme; the Worker's page redirects to
//!   `kanso://auth/apple?flow=<flow id>` and the app polls with its device.
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
            BrowserError::WrongCallback => f.write_str("the browser returned another sign-in"),
            BrowserError::Unsupported(what) => write!(f, "{what}"),
        }
    }
}

impl std::error::Error for BrowserError {}

/// Opens `url` in the system browser. Never a web view.
pub fn open_external(url: &str) -> Result<(), BrowserError> {
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
        let status = std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
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

/// Opens `start.auth_url` in `ASWebAuthenticationSession` (macOS) with the
/// `kanso` scheme and waits for the callback URL. The flow ID alone grants
/// nothing: the caller polls the Worker with its device and proof.
pub fn apple_sign_in(
    start: &AppleStart,
) -> Result<async_channel::Receiver<Result<String, BrowserError>>, BrowserError> {
    #[cfg(target_os = "macos")]
    {
        apple::start(&start.auth_url, "kanso")
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = start;
        Err(BrowserError::Unsupported(
            "Sign in with Apple needs macOS's system sign-in sheet",
        ))
    }
}

/// The flow ID in Apple's callback URL, when it is the callback the Worker's
/// page makes.
pub fn apple_flow(callback: &str) -> Option<String> {
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
        let url = NSURL::URLWithString(&NSString::from_str(auth_url))
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

    pub fn port(&self) -> u16 {
        self.port
    }

    /// Accepts one request, checks `state`, answers "Return to Kanso" and
    /// returns the `code`. The listener is done either way.
    pub fn wait(&self, expected_state: &str, timeout: Duration) -> Result<String, BrowserError> {
        self.listener
            .set_nonblocking(true)
            .map_err(BrowserError::Bind)?;
        let started = Instant::now();
        let mut stream = loop {
            match self.listener.accept() {
                Ok((stream, _)) => break stream,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    if started.elapsed() > timeout {
                        return Err(BrowserError::Io("the browser didn't return in time".into()));
                    }
                    std::thread::sleep(Duration::from_millis(25));
                }
                Err(err) => return Err(BrowserError::Bind(err)),
            }
        };
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .map_err(BrowserError::Bind)?;
        let request = read_request(&mut stream)?;
        let answer = parse_loopback_request(&request, expected_state);
        match answer {
            Some(code) => {
                let _ = respond(&mut stream, 200, "Signed in. Return to Kanso.");
                Ok(code)
            }
            None => {
                let _ = respond(
                    &mut stream,
                    400,
                    "That sign-in didn't come from Kanso. Return to Kanso and try again.",
                );
                Err(BrowserError::WrongCallback)
            }
        }
    }
}

fn read_request(stream: &mut TcpStream) -> Result<String, BrowserError> {
    let mut bytes = Vec::with_capacity(1024);
    let mut buffer = [0u8; 512];
    loop {
        let read = stream
            .read(&mut buffer)
            .map_err(|err| BrowserError::Io(format!("the browser didn't answer: {err}")))?;
        if read == 0 {
            break;
        }
        bytes.extend_from_slice(&buffer[..read]);
        if bytes.windows(4).any(|w| w == b"\r\n\r\n") || bytes.len() > 16 * 1024 {
            break;
        }
    }
    String::from_utf8(bytes)
        .map_err(|_| BrowserError::Io("the browser sent bytes that aren't a URL".into()))
}

fn respond(stream: &mut TcpStream, status: u16, sentence: &str) -> std::io::Result<()> {
    let reason = if status == 200 { "OK" } else { "Bad Request" };
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
pub fn parse_loopback_request(request: &str, expected_state: &str) -> Option<String> {
    let line = request.lines().next()?;
    let mut parts = line.split_whitespace();
    let method = parts.next()?;
    let target = parts.next()?;
    let version = parts.next()?;
    if method != "GET" || !version.starts_with("HTTP/") {
        return None;
    }
    let (path, query) = target.split_once('?')?;
    if path != "/" {
        return None;
    }
    let mut code = None;
    let mut state = None;
    for pair in query.split('&') {
        let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
        let value = percent_decode(value);
        match key {
            "code" => code = Some(value),
            "state" => state = Some(value),
            _ => {}
        }
    }
    let code = code.filter(|code| !code.is_empty())?;
    (state.as_deref() == Some(expected_state)).then_some(code)
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

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(
            apple_flow(&format!("kanso://auth/apple?flow={flow}")).as_deref(),
            Some(flow.as_str())
        );
        assert_eq!(apple_flow("kanso://auth/apple?flow=short"), None);
    }
}
