//! A blocking WebSocket client to the Worker (`tungstenite` over the rustls
//! and web PKI roots ureq already uses), shared by the account event socket
//! ([`crate::events`]) and shared notes' live room (`kanso-collab`).
//!
//! The upgrade carries the caller's headers and never an `Origin`. It
//! follows no redirect: tungstenite's client handshake refuses anything but
//! 101, and a refusal hands back the status and the JSON error body, so the
//! caller maps it as it maps an HTTP answer.

use std::io::{self, Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::Arc;
use std::time::Duration;

use tungstenite::client::IntoClientRequest;
use tungstenite::http::HeaderValue;
use tungstenite::protocol::WebSocket;

const USER_AGENT: &str = concat!("kanso-sync/", env!("CARGO_PKG_VERSION"));

/// How long connecting, a stalled send and (until the caller sets its own) a
/// read may take.
const SOCKET_TIMEOUT: Duration = Duration::from_secs(20);

/// A plain or TLS stream under the WebSocket.
pub enum Stream {
    Plain(TcpStream),
    Tls(Box<rustls::StreamOwned<rustls::ClientConnection, TcpStream>>),
}

impl Stream {
    /// The TCP socket underneath, for read timeouts.
    pub fn tcp(&self) -> &TcpStream {
        match self {
            Stream::Plain(tcp) => tcp,
            Stream::Tls(tls) => tls.get_ref(),
        }
    }
}

impl Read for Stream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Stream::Plain(tcp) => tcp.read(buf),
            Stream::Tls(tls) => tls.read(buf),
        }
    }
}

impl Write for Stream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Stream::Plain(tcp) => tcp.write(buf),
            Stream::Tls(tls) => tls.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Stream::Plain(tcp) => tcp.flush(),
            Stream::Tls(tls) => tls.flush(),
        }
    }
}

/// Why an upgrade failed.
#[derive(Debug)]
pub enum WsError {
    /// The server URL or a header can't be used; nothing was sent.
    Request(String),
    /// No answer: DNS, connect, TLS, or the connection dropped.
    Transport(String),
    /// The server answered the upgrade with `status` instead of 101.
    Refused { status: u16, body: Vec<u8> },
}

fn tls_config() -> Arc<rustls::ClientConfig> {
    let roots = rustls::RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    Arc::new(
        rustls::ClientConfig::builder_with_provider(Arc::new(
            rustls::crypto::ring::default_provider(),
        ))
        .with_safe_default_protocol_versions()
        .expect("ring supports the default TLS versions")
        .with_root_certificates(roots)
        .with_no_client_auth(),
    )
}

/// Opens a WebSocket to `path` on `server` (an http or https URL; the
/// server's own path, if it has one, is kept) with `headers`, which may hold
/// a session token: they go only to this server, in the upgrade.
pub fn connect(
    server: &str,
    path: &str,
    headers: &[(&str, &str)],
) -> Result<WebSocket<Stream>, WsError> {
    let base = url::Url::parse(server).map_err(|err| WsError::Request(err.to_string()))?;
    let tls = match base.scheme() {
        "https" => true,
        "http" => false,
        other => return Err(WsError::Request(format!("can't reach {other}://"))),
    };
    let host = base
        .host_str()
        .ok_or_else(|| WsError::Request("no host".into()))?
        .trim_start_matches('[')
        .trim_end_matches(']')
        .to_owned();
    let port = base
        .port_or_known_default()
        .unwrap_or(if tls { 443 } else { 80 });
    // Each address in turn, as the HTTP client does.
    let mut last = WsError::Transport("no address".into());
    let mut connected = None;
    for address in (host.as_str(), port)
        .to_socket_addrs()
        .map_err(|err| WsError::Transport(err.to_string()))?
    {
        match TcpStream::connect_timeout(&address, SOCKET_TIMEOUT) {
            Ok(tcp) => {
                connected = Some(tcp);
                break;
            }
            Err(err) => last = WsError::Transport(err.to_string()),
        }
    }
    let tcp = connected.ok_or(last)?;
    tcp.set_nodelay(true).ok();
    tcp.set_read_timeout(Some(SOCKET_TIMEOUT)).ok();
    // A stalled send fails rather than holding the caller's thread.
    tcp.set_write_timeout(Some(SOCKET_TIMEOUT)).ok();
    let stream = if tls {
        let name = rustls::pki_types::ServerName::try_from(host.clone())
            .map_err(|err| WsError::Request(err.to_string()))?;
        let connection = rustls::ClientConnection::new(tls_config(), name)
            .map_err(|err| WsError::Transport(err.to_string()))?;
        Stream::Tls(Box::new(rustls::StreamOwned::new(connection, tcp)))
    } else {
        Stream::Plain(tcp)
    };
    let ws_url = format!(
        "{}://{}{}{}",
        if tls { "wss" } else { "ws" },
        base.host_str().unwrap_or_default().to_owned()
            + &base.port().map(|p| format!(":{p}")).unwrap_or_default(),
        base.path().trim_end_matches('/'),
        path
    );
    let mut request = ws_url
        .into_client_request()
        .map_err(|err| WsError::Request(err.to_string()))?;
    let request_headers = request.headers_mut();
    for (name, value) in headers {
        let value =
            HeaderValue::from_str(value).map_err(|err| WsError::Request(err.to_string()))?;
        request_headers.insert(
            tungstenite::http::HeaderName::from_bytes(name.as_bytes())
                .map_err(|err| WsError::Request(err.to_string()))?,
            value,
        );
    }
    request_headers.insert("user-agent", HeaderValue::from_static(USER_AGENT));
    let (socket, _) = tungstenite::client::client(request, stream).map_err(|err| match err {
        tungstenite::handshake::HandshakeError::Failure(tungstenite::Error::Http(response)) => {
            WsError::Refused {
                status: response.status().as_u16(),
                body: response.body().clone().unwrap_or_default(),
            }
        }
        other => WsError::Transport(other.to_string()),
    })?;
    Ok(socket)
}
