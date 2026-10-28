//! The listener for Google's redirect (docs/ACCOUNTS.md › Sign in with
//! Apple, Google and GitHub).
//!
//! Google's sign-in on the phone is the desktop client's, with a loopback
//! redirect: Kanso binds `127.0.0.1:0` before the Worker is asked to start
//! the flow, gives the Worker that exact address and its PKCE challenge, and
//! the shell opens Google's page in the system's sign-in session. Google
//! sends the browser to `http://127.0.0.1:<port>/?code=…&state=…`, the
//! listener answers "Return to Kanso" and hands the code to the flow. The
//! code verifier never leaves this process.
//!
//! Plain `std::net`; the page the person sees is one sentence.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use kanso_engine::account::provider as flow;

/// What can go wrong listening for the redirect.
#[derive(Debug)]
pub(crate) enum LoopbackError {
    /// The address couldn't be bound or read.
    Io,
    /// The flow was cancelled while it waited.
    Cancelled,
    /// The flow's deadline passed.
    Expired,
    /// The provider sent back an error for this flow's `state`.
    ProviderDenied,
    /// The request wasn't this flow's: another `state`, path or method.
    WrongCallback,
}

impl From<LoopbackError> for flow::BrowserError {
    fn from(err: LoopbackError) -> Self {
        match err {
            LoopbackError::Cancelled => flow::BrowserError::Cancelled,
            LoopbackError::Expired => flow::BrowserError::Expired,
            LoopbackError::ProviderDenied => flow::BrowserError::ProviderDenied,
            LoopbackError::Io | LoopbackError::WrongCallback => flow::BrowserError::Failed,
        }
    }
}

/// A one-request loopback listener. Bind it first so its address can be the
/// Worker's `redirect_uri`, then [`Loopback::wait_until`] for the answer.
pub(crate) struct Loopback {
    listener: TcpListener,
    port: u16,
}

/// The longest request line and headers read.
const MAX_REQUEST: usize = 16 * 1024;

/// How often the listener looks at the clock and the cancel flag.
const POLL: Duration = Duration::from_millis(25);

impl Loopback {
    /// Binds `127.0.0.1:0`: the system picks a free port.
    pub fn bind() -> Result<Loopback, LoopbackError> {
        let listener = TcpListener::bind(("127.0.0.1", 0)).map_err(|_| LoopbackError::Io)?;
        let port = listener.local_addr().map_err(|_| LoopbackError::Io)?.port();
        Ok(Loopback { listener, port })
    }

    /// The exact redirect address: `http://127.0.0.1:<port>`, nothing after.
    pub fn uri(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    #[cfg(test)]
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Accepts one request, checks `state`, answers it and returns the
    /// `code`, without blocking cancellation or the flow's deadline.
    pub fn wait_until(
        &self,
        expected_state: &str,
        deadline: Instant,
        cancel: &AtomicBool,
    ) -> Result<String, LoopbackError> {
        self.listener
            .set_nonblocking(true)
            .map_err(|_| LoopbackError::Io)?;
        let mut stream = loop {
            check(deadline, cancel)?;
            match self.listener.accept() {
                Ok((stream, _)) => break stream,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => pause(deadline),
                Err(_) => return Err(LoopbackError::Io),
            }
        };
        stream
            .set_nonblocking(true)
            .map_err(|_| LoopbackError::Io)?;
        let request = read_request(&mut stream, deadline, cancel)?;
        match parse_callback(&request, expected_state) {
            Ok(code) => {
                let _ = respond(&mut stream, 200, "Signed in. Return to Kanso.");
                Ok(code)
            }
            Err(LoopbackError::ProviderDenied) => {
                let _ = respond(&mut stream, 200, "Sign-in was cancelled. Return to Kanso.");
                Err(LoopbackError::ProviderDenied)
            }
            Err(err) => {
                let _ = respond(
                    &mut stream,
                    400,
                    "That sign-in didn't come from Kanso. Return to Kanso and try again.",
                );
                Err(err)
            }
        }
    }
}

fn check(deadline: Instant, cancel: &AtomicBool) -> Result<(), LoopbackError> {
    if cancel.load(Ordering::Acquire) {
        return Err(LoopbackError::Cancelled);
    }
    if Instant::now() >= deadline {
        return Err(LoopbackError::Expired);
    }
    Ok(())
}

fn pause(deadline: Instant) {
    std::thread::sleep(POLL.min(deadline.saturating_duration_since(Instant::now())));
}

/// The request's head, up to the blank line after its headers.
fn read_request(
    stream: &mut TcpStream,
    deadline: Instant,
    cancel: &AtomicBool,
) -> Result<String, LoopbackError> {
    let mut bytes = Vec::with_capacity(1024);
    let mut buffer = [0u8; 512];
    loop {
        check(deadline, cancel)?;
        match stream.read(&mut buffer) {
            Ok(0) => return Err(LoopbackError::WrongCallback),
            Ok(read) => {
                bytes.extend_from_slice(&buffer[..read]);
                if bytes.windows(4).any(|window| window == b"\r\n\r\n") {
                    return String::from_utf8(bytes).map_err(|_| LoopbackError::WrongCallback);
                }
                if bytes.len() > MAX_REQUEST {
                    return Err(LoopbackError::WrongCallback);
                }
            }
            Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => pause(deadline),
            Err(_) => return Err(LoopbackError::Io),
        }
    }
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
/// A provider's `error` for this `state` is a denial.
pub(crate) fn parse_callback(request: &str, expected_state: &str) -> Result<String, LoopbackError> {
    let line = request.lines().next().ok_or(LoopbackError::WrongCallback)?;
    let mut parts = line.split_whitespace();
    let method = parts.next().ok_or(LoopbackError::WrongCallback)?;
    let target = parts.next().ok_or(LoopbackError::WrongCallback)?;
    let version = parts.next().ok_or(LoopbackError::WrongCallback)?;
    if method != "GET" || !version.starts_with("HTTP/") {
        return Err(LoopbackError::WrongCallback);
    }
    let (path, query) = target.split_once('?').ok_or(LoopbackError::WrongCallback)?;
    if path != "/" {
        return Err(LoopbackError::WrongCallback);
    }
    let (mut code, mut state, mut error) = (None, None, None);
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
        return Err(LoopbackError::WrongCallback);
    }
    if error.is_some() {
        return Err(LoopbackError::ProviderDenied);
    }
    code.filter(|code| !code.is_empty())
        .ok_or(LoopbackError::WrongCallback)
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
            b'%' if i + 2 < bytes.len() => match (hex_digit(bytes[i + 1]), hex_digit(bytes[i + 2]))
            {
                (Some(high), Some(low)) => {
                    out.push(high << 4 | low);
                    i += 3;
                }
                _ => {
                    out.push(b'%');
                    i += 1;
                }
            },
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

/// The page a provider flow may ask the shell to open: `https` with a host,
/// or plain `http` to this device, with no user info, whitespace or control
/// characters, in its parsed form. `kanso-sync` pins a provider start's page
/// to the provider's host or the server's; this is the second check, so
/// the shell is never handed another scheme to open.
pub(crate) fn page_url(text: &str) -> Result<String, ()> {
    if text.chars().any(|c| c.is_whitespace() || c.is_control()) {
        return Err(());
    }
    let url = url::Url::parse(text).map_err(|_| ())?;
    if !url.username().is_empty() || url.password().is_some() {
        return Err(());
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
    if allowed { Ok(url.into()) } else { Err(()) }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    proptest! {
        /// The loopback accepts only `GET /` with this flow's state, and hands
        /// back the code exactly as the browser sent it, percent-decoded.
        #[test]
        fn prop_the_parser_returns_the_code_of_its_own_state_only(
            code in "[A-Za-z0-9/_.~-]{1,40}",
            state in "[A-Za-z0-9_-]{1,40}",
            other in "[A-Za-z0-9_-]{1,40}",
        ) {
            prop_assume!(state != other);
            let escaped = code.replace('/', "%2F");
            let own = format!("GET /?code={escaped}&state={state} HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n");
            prop_assert_eq!(parse_callback(&own, &state).ok(), Some(code.clone()));
            prop_assert!(matches!(parse_callback(&own, &other), Err(LoopbackError::WrongCallback)));
            let denied = format!("GET /?error=access_denied&state={state} HTTP/1.1\r\n\r\n");
            prop_assert!(matches!(parse_callback(&denied, &state), Err(LoopbackError::ProviderDenied)));
            prop_assert!(matches!(parse_callback(&denied, &other), Err(LoopbackError::WrongCallback)));
        }

        /// Whatever a browser or a stranger sends, the parser neither panics
        /// nor returns a code for a state it wasn't given.
        #[test]
        fn prop_no_request_yields_a_code_without_the_state(request in "\\PC{0,300}") {
            if parse_callback(&request, "the-state").is_ok() {
                prop_assert!(request.contains("state="));
            }
        }

        /// A page the shell is asked to open has no user info and no
        /// whitespace, and is https or this device.
        #[test]
        fn prop_pages_are_https_or_this_device(text in "\\PC{0,80}") {
            if let Ok(url) = page_url(&text) {
                prop_assert!(url.starts_with("https://") || url.starts_with("http://"));
                prop_assert!(!url.chars().any(|c| c.is_whitespace() || c.is_control()));
                prop_assert!(!url.contains('@'));
            }
        }
    }

    // no-prop-test: a table of what may and may not be opened.
    #[test]
    fn only_web_pages_are_handed_to_the_shell() {
        for good in [
            "https://github.com/login/device",
            "https://accounts.google.com/o/oauth2/v2/auth?client_id=1&state=2",
            "http://127.0.0.1:8787/device",
            "http://localhost:8787/device",
        ] {
            assert_eq!(page_url(good).as_deref(), Ok(good), "{good}");
        }
        for bad in [
            "file:///etc/passwd",
            "javascript:alert(1)",
            "kanso://auth/apple?flow=x&s=y",
            "http://evil.example/page",
            "https://user:pass@evil.example/",
            " https://example.com/",
            "https://example.com/\npage",
            "https://",
            "-aflag",
            "",
        ] {
            assert!(page_url(bad).is_err(), "{bad:?}");
        }
    }

    // no-prop-test: a table of the callbacks a real browser or a stranger
    // makes.
    #[test]
    fn the_parser_needs_the_right_path_method_and_state() {
        assert_eq!(
            parse_callback(
                "GET /?code=4%2F0abc&state=s1 HTTP/1.1\r\nHost: x\r\n\r\n",
                "s1"
            )
            .ok(),
            Some("4/0abc".to_owned())
        );
        assert_eq!(
            parse_callback("GET /?code=a+b&state=s1 HTTP/1.1\r\n\r\n", "s1").ok(),
            Some("a b".to_owned())
        );
        for bad in [
            "GET /cb?code=x&state=s1 HTTP/1.1\r\n\r\n",
            "GET /?code=x&state=other HTTP/1.1\r\n\r\n",
            "POST /?code=x&state=s1 HTTP/1.1\r\n\r\n",
            "GET /?state=s1 HTTP/1.1\r\n\r\n",
            "GET /?code=&state=s1 HTTP/1.1\r\n\r\n",
            "GET /?error=access_denied&state=wrong HTTP/1.1\r\n\r\n",
            "GET /?error=access_denied HTTP/1.1\r\n\r\n",
            "nonsense",
            "",
        ] {
            assert!(
                matches!(parse_callback(bad, "s1"), Err(LoopbackError::WrongCallback)),
                "{bad}"
            );
        }
    }

    // no-prop-test: one real connection to a real listener, and the two
    // ways it ends without one.
    #[test]
    fn a_listener_answers_one_request_and_returns_its_code() {
        let loopback = Loopback::bind().unwrap();
        assert_eq!(
            loopback.uri(),
            format!("http://127.0.0.1:{}", loopback.port())
        );
        let port = loopback.port();
        let waiter = std::thread::spawn(move || {
            let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
            stream
                .write_all(
                    b"GET /?code=4%2Fsecret&state=the-state HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n",
                )
                .unwrap();
            let mut answer = String::new();
            stream.read_to_string(&mut answer).unwrap();
            answer
        });
        let never = AtomicBool::new(false);
        let code = loopback
            .wait_until("the-state", Instant::now() + Duration::from_secs(5), &never)
            .unwrap();
        assert_eq!(code, "4/secret");
        let answer = waiter.join().unwrap();
        assert!(answer.starts_with("HTTP/1.1 200"));
        assert!(answer.contains("Return to Kanso"));
        assert!(!answer.contains("secret"), "the code is not echoed");

        // Another flow's state is refused with a 400, and ends the wait.
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
            loopback.wait_until("right", Instant::now() + Duration::from_secs(5), &never),
            Err(LoopbackError::WrongCallback)
        ));
        assert!(waiter.join().unwrap().starts_with("HTTP/1.1 400"));

        // Cancelled and expired before anyone calls.
        let loopback = Loopback::bind().unwrap();
        let cancelled = AtomicBool::new(true);
        assert!(matches!(
            loopback.wait_until("s", Instant::now() + Duration::from_secs(1), &cancelled),
            Err(LoopbackError::Cancelled)
        ));
        assert!(matches!(
            loopback.wait_until("s", Instant::now(), &never),
            Err(LoopbackError::Expired)
        ));
    }
}
