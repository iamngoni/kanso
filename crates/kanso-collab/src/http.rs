//! [`CollabApi`] over HTTPS, as `kanso-sync` talks to the Worker: no
//! redirects, no `Origin`, capped answers; and the live room over a
//! WebSocket (`tungstenite`) on the same rustls and web PKI roots.

use std::io;
use std::time::Duration;

use kanso_sync::ws::{Stream, WsError};
use serde::Serialize;
use serde::de::DeserializeOwned;
use tungstenite::Message;
use tungstenite::protocol::WebSocket;
use ureq::Agent;

use crate::api::{
    Caller, CollabApi, CollabError, Created, Joined, LiveEvent, LiveLink, LogPage, NewEpoch,
    NewLink, NewShare, NoticeIn, NoticeOut, Result, ShareInfo, SharesPage, parse_control,
    parse_frame,
};
use crate::codec::b64;
use crate::identity::Published;
use crate::model::PublicIdentity;

/// The largest answer read: a log page (about 8 MiB of envelopes as
/// base64url) and a little more.
const MAX_RESPONSE: u64 = 12 * 1024 * 1024;
const USER_AGENT: &str = concat!("kanso-sync/", env!("CARGO_PKG_VERSION"));

/// The Worker's shared-note routes.
#[derive(Clone, Copy, Debug, Default)]
pub struct HttpCollab;

fn agent() -> Agent {
    Agent::config_builder()
        .http_status_as_error(false)
        .max_redirects(0)
        .timeout_connect(Some(Duration::from_secs(20)))
        .timeout_global(Some(Duration::from_secs(120)))
        .user_agent(USER_AGENT)
        .build()
        .into()
}

/// A path segment, percent-encoded.
fn segment(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes())
        .collect::<String>()
        .replace('+', "%20")
}

fn url(caller: &Caller, path: &str) -> String {
    format!("{}{}", caller.server.trim_end_matches('/'), path)
}

/// The answer's body for a 2xx (empty for 204), else the error it carries.
fn answer(
    response: std::result::Result<ureq::http::Response<ureq::Body>, ureq::Error>,
) -> Result<Option<Vec<u8>>> {
    let mut response = response.map_err(|err| CollabError::Transport(err.to_string()))?;
    let status = response.status().as_u16();
    if (300..400).contains(&status) {
        return Err(CollabError::Protocol(format!(
            "the server answered {status} (a redirect), which the client never follows"
        )));
    }
    if status == 204 {
        return Ok(None);
    }
    let body = response
        .body_mut()
        .with_config()
        .limit(MAX_RESPONSE)
        .read_to_vec()
        .map_err(|err| CollabError::Transport(err.to_string()))?;
    if (200..300).contains(&status) {
        Ok(Some(body))
    } else {
        Err(CollabError::from_response(status, &body))
    }
}

fn json_of<T: DeserializeOwned>(body: Option<Vec<u8>>) -> Result<T> {
    let body = body.ok_or_else(|| CollabError::Protocol("an empty answer".into()))?;
    serde_json::from_slice(&body).map_err(|err| CollabError::Protocol(err.to_string()))
}

fn get(caller: &Caller, path: &str) -> Result<Option<Vec<u8>>> {
    answer(
        agent()
            .get(url(caller, path))
            .header("authorization", caller.authorization.as_str())
            .call(),
    )
}

fn delete(caller: &Caller, path: &str) -> Result<()> {
    answer(
        agent()
            .delete(url(caller, path))
            .header("authorization", caller.authorization.as_str())
            .call(),
    )
    .map(|_| ())
}

fn send_json<T: Serialize>(
    caller: &Caller,
    method: &str,
    path: &str,
    body: &T,
) -> Result<Option<Vec<u8>>> {
    let bytes = serde_json::to_vec(body).map_err(|err| CollabError::Protocol(err.to_string()))?;
    let target = url(caller, path);
    let request = match method {
        "PUT" => agent().put(target),
        _ => agent().post(target),
    };
    answer(
        request
            .header("authorization", caller.authorization.as_str())
            .header("content-type", "application/json")
            .send(&bytes[..]),
    )
}

fn send_bytes(
    caller: &Caller,
    path: &str,
    bytes: &[u8],
    base_seq: Option<u64>,
) -> Result<Option<Vec<u8>>> {
    let mut request = agent()
        .post(url(caller, path))
        .header("authorization", caller.authorization.as_str())
        .header("content-type", "application/octet-stream");
    if let Some(seq) = base_seq {
        request = request.header("x-kanso-base-seq", seq.to_string());
    }
    answer(request.send(bytes))
}

/// `None` for a 404, the value otherwise.
fn found<T>(result: Result<T>) -> Result<Option<T>> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(CollabError::Api { status: 404, .. }) => Ok(None),
        Err(err) => Err(err),
    }
}

#[derive(serde::Deserialize)]
struct Seq {
    seq: u64,
}

impl CollabApi for HttpCollab {
    fn put_identity(&self, caller: &Caller, published: &Published) -> Result<()> {
        send_json(caller, "PUT", "/v2/account/identity", published).map(|_| ())
    }

    fn get_identity(&self, caller: &Caller) -> Result<Option<Published>> {
        found(get(caller, "/v2/account/identity").and_then(json_of))
    }

    fn lookup(&self, caller: &Caller, username: &str) -> Result<Option<PublicIdentity>> {
        found(get(caller, &format!("/v2/users/{}", segment(username.trim()))).and_then(json_of))
    }

    fn create_share(&self, caller: &Caller, share: &NewShare) -> Result<Created> {
        json_of(send_json(caller, "POST", "/v2/collab/shares", share)?)
    }

    fn shares(&self, caller: &Caller) -> Result<Vec<ShareInfo>> {
        Ok(json_of::<SharesPage>(get(caller, "/v2/collab/shares")?)?.shares)
    }

    fn accept(&self, caller: &Caller, share_id: &str) -> Result<()> {
        send_json(
            caller,
            "POST",
            &format!("/v2/collab/shares/{}/accept", segment(share_id)),
            &serde_json::json!({}),
        )
        .map(|_| ())
    }

    fn leave(&self, caller: &Caller, share_id: &str) -> Result<()> {
        delete(
            caller,
            &format!("/v2/collab/shares/{}/membership", segment(share_id)),
        )
    }

    fn new_epoch(&self, caller: &Caller, share_id: &str, epoch: &NewEpoch) -> Result<u32> {
        #[derive(serde::Deserialize)]
        struct Epoch {
            epoch: u32,
        }
        let path = format!("/v2/collab/shares/{}/epoch", segment(share_id));
        Ok(json_of::<Epoch>(send_json(caller, "POST", &path, epoch)?)?.epoch)
    }

    fn stop(&self, caller: &Caller, share_id: &str) -> Result<()> {
        delete(caller, &format!("/v2/collab/shares/{}", segment(share_id)))
    }

    fn log(&self, caller: &Caller, share_id: &str, since: u64, limit: usize) -> Result<LogPage> {
        json_of(get(
            caller,
            &format!(
                "/v2/collab/shares/{}/log?since={since}&limit={limit}",
                segment(share_id)
            ),
        )?)
    }

    fn append(&self, caller: &Caller, share_id: &str, envelope: &[u8]) -> Result<u64> {
        let path = format!("/v2/collab/shares/{}/log", segment(share_id));
        Ok(json_of::<Seq>(send_bytes(caller, &path, envelope, None)?)?.seq)
    }

    fn snapshot(
        &self,
        caller: &Caller,
        share_id: &str,
        base_seq: u64,
        envelope: &[u8],
    ) -> Result<u64> {
        let path = format!("/v2/collab/shares/{}/snapshot", segment(share_id));
        Ok(json_of::<Seq>(send_bytes(caller, &path, envelope, Some(base_seq))?)?.seq)
    }

    fn create_link(&self, caller: &Caller, share_id: &str, link: &NewLink) -> Result<()> {
        let path = format!("/v2/collab/shares/{}/links", segment(share_id));
        send_json(caller, "POST", &path, link).map(|_| ())
    }

    fn delete_link(&self, caller: &Caller, share_id: &str, link_id: &str) -> Result<()> {
        delete(
            caller,
            &format!(
                "/v2/collab/shares/{}/links/{}",
                segment(share_id),
                segment(link_id)
            ),
        )
    }

    fn join(&self, caller: &Caller, link_id: &str, proof: &str) -> Result<Joined> {
        json_of(send_json(
            caller,
            "POST",
            &format!("/v2/collab/join/{}", segment(link_id)),
            &serde_json::json!({ "proof": proof }),
        )?)
    }

    fn decline_request(&self, caller: &Caller, share_id: &str, account: &str) -> Result<()> {
        delete(
            caller,
            &format!(
                "/v2/collab/shares/{}/requests/{}",
                segment(share_id),
                segment(account)
            ),
        )
    }

    fn notify(&self, caller: &Caller, notice: &NoticeIn) -> Result<()> {
        send_json(caller, "POST", "/v2/collab/notifications", notice).map(|_| ())
    }

    fn notifications(&self, caller: &Caller) -> Result<Vec<NoticeOut>> {
        #[derive(serde::Deserialize)]
        struct Page {
            notifications: Vec<NoticeOut>,
        }
        Ok(json_of::<Page>(get(caller, "/v2/collab/notifications")?)?.notifications)
    }

    fn delete_notification(&self, caller: &Caller, id: &str) -> Result<()> {
        delete(caller, &format!("/v2/collab/notifications/{}", segment(id)))
    }

    fn live(&self, caller: &Caller, share_id: &str) -> Result<Box<dyn LiveLink>> {
        Ok(Box::new(WsLink::connect(caller, share_id)?))
    }
}

/// The live room over a WebSocket. The session's token goes in the upgrade
/// request's `Authorization` header; there is no `Origin`.
struct WsLink {
    socket: WebSocket<Stream>,
}

impl WsLink {
    fn connect(caller: &Caller, share_id: &str) -> Result<WsLink> {
        let path = format!("/v2/collab/shares/{}/live", segment(share_id));
        let socket = kanso_sync::ws::connect(
            &caller.server,
            &path,
            &[("authorization", caller.authorization.as_str())],
        )
        .map_err(|err| match err {
            WsError::Request(message) => CollabError::Protocol(message),
            WsError::Transport(message) => CollabError::Transport(message),
            WsError::Refused { status, body } => CollabError::from_response(status, &body),
        })?;
        Ok(WsLink { socket })
    }
}

impl LiveLink for WsLink {
    fn send(&mut self, envelope: &[u8]) -> Result<()> {
        self.socket
            .send(Message::binary(envelope.to_vec()))
            .map_err(|err| CollabError::Transport(err.to_string()))
    }

    fn recv(&mut self, wait: Duration) -> Result<Option<LiveEvent>> {
        self.socket
            .get_ref()
            .tcp()
            .set_read_timeout(Some(wait.max(Duration::from_millis(1))))
            .ok();
        loop {
            match self.socket.read() {
                Ok(Message::Binary(frame)) => {
                    return parse_frame(&frame)
                        .map(Some)
                        .ok_or_else(|| CollabError::Protocol("a short frame".into()));
                }
                Ok(Message::Text(text)) => {
                    if let Some(event) = parse_control(text.as_str()) {
                        return Ok(Some(event));
                    }
                }
                Ok(Message::Close(_)) => {
                    return Ok(Some(LiveEvent::Closed {
                        reason: "closed".into(),
                    }));
                }
                Ok(_) => {}
                Err(tungstenite::Error::Io(err))
                    if matches!(
                        err.kind(),
                        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                    ) =>
                {
                    return Ok(None);
                }
                Err(err) => return Err(CollabError::Transport(err.to_string())),
            }
        }
    }
}

/// Base64url, for envelopes in JSON bodies.
pub fn envelope_text(envelope: &[u8]) -> String {
    b64(envelope)
}

#[cfg(test)]
mod tests {
    use std::io::{BufRead, BufReader, Read, Write};
    use std::net::TcpListener;
    use std::sync::mpsc;

    use zeroize::Zeroizing;

    use super::*;

    fn caller(server: String) -> Caller {
        Caller {
            server,
            account_id: "a_0123456789abcdef0123456789abcdef".into(),
            username: "ana".into(),
            device_id: "d_fedcba9876543210fedcba9876543210".into(),
            authorization: Zeroizing::new("Bearer kanso_test".into()),
        }
    }

    /// Answers one request on a loopback port with `response`, and hands
    /// back what was asked (head and body).
    fn one_answer(response: &'static str) -> (String, mpsc::Receiver<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let server = format!("http://{}", listener.local_addr().unwrap());
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut request = String::new();
            let mut length = 0;
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                    length = value.trim().parse().unwrap();
                }
                request.push_str(&line);
                if line == "\r\n" {
                    break;
                }
            }
            let mut body = vec![0; length];
            reader.read_exact(&mut body).unwrap();
            request.push_str(&String::from_utf8_lossy(&body));
            (&stream).write_all(response.as_bytes()).unwrap();
            tx.send(request).unwrap();
        });
        (server, rx)
    }

    #[test]
    fn a_snapshot_goes_as_bytes_with_its_base_the_session_and_no_origin() {
        let (server, asked) = one_answer(
            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: 9\r\nconnection: close\r\n\r\n{\"seq\":5}",
        );
        let seq = HttpCollab
            .snapshot(&caller(server), "ab/c", 5, b"envelope")
            .unwrap();
        assert_eq!(seq, 5);
        let request = asked.recv().unwrap();
        let lower = request.to_ascii_lowercase();
        assert!(
            request.starts_with("POST /v2/collab/shares/ab%2Fc/snapshot "),
            "{request}"
        );
        assert!(lower.contains("authorization: bearer kanso_test"));
        assert!(lower.contains("x-kanso-base-seq: 5"));
        assert!(lower.contains("content-type: application/octet-stream"));
        assert!(!lower.contains("\norigin:"));
        assert!(request.ends_with("envelope"));
    }

    #[test]
    fn errors_keep_their_code_and_extras_and_redirects_are_refused() {
        let (server, _) = one_answer(
            "HTTP/1.1 409 Conflict\r\ncontent-type: application/json\r\ncontent-length: 32\r\nconnection: close\r\n\r\n{\"error\":\"head_moved\",\"head\":9}\n",
        );
        let err = HttpCollab
            .snapshot(&caller(server), "s", 4, b"e")
            .unwrap_err();
        assert_eq!(err.code(), Some("head_moved"));
        assert_eq!(err.extra("head"), Some(9));
        let (server, _) = one_answer(
            "HTTP/1.1 302 Found\r\nlocation: https://elsewhere.test/\r\ncontent-length: 0\r\nconnection: close\r\n\r\n",
        );
        let err = HttpCollab.shares(&caller(server)).unwrap_err();
        assert!(matches!(err, CollabError::Protocol(_)), "{err:?}");
    }

    // tungstenite's handshake callback returns its own (large) error type.
    #[allow(clippy::result_large_err)]
    #[test]
    fn the_live_room_authenticates_and_carries_frames_both_ways() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let server = format!("http://{}", listener.local_addr().unwrap());
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut seen = String::new();
            let mut socket = tungstenite::accept_hdr(
                stream,
                |request: &tungstenite::handshake::server::Request,
                 response: tungstenite::handshake::server::Response| {
                    seen = format!(
                        "{} auth={:?} origin={:?}",
                        request.uri().path(),
                        request.headers().get("authorization"),
                        request.headers().get("origin")
                    );
                    Ok(response)
                },
            )
            .unwrap();
            socket
                .send(Message::text(r#"{"type":"hello","head":3,"epoch":2}"#))
                .unwrap();
            let mut frame = 3u64.to_be_bytes().to_vec();
            frame.extend_from_slice(b"stored");
            socket.send(Message::binary(frame)).unwrap();
            let sent = socket.read().unwrap();
            socket
                .send(Message::text(r#"{"type":"closed","reason":"removed"}"#))
                .unwrap();
            tx.send((seen, sent.into_data().to_vec())).unwrap();
            let _ = socket.close(None);
            let _ = socket.flush();
        });
        let mut link = HttpCollab.live(&caller(server), "share").unwrap();
        let wait = Duration::from_secs(5);
        assert_eq!(
            link.recv(wait).unwrap(),
            Some(LiveEvent::Hello { head: 3, epoch: 2 })
        );
        assert_eq!(
            link.recv(wait).unwrap(),
            Some(LiveEvent::Stored {
                seq: 3,
                envelope: b"stored".to_vec()
            })
        );
        link.send(b"mine").unwrap();
        assert_eq!(
            link.recv(wait).unwrap(),
            Some(LiveEvent::Closed {
                reason: "removed".into()
            })
        );
        let (seen, sent) = rx.recv().unwrap();
        assert_eq!(
            seen,
            r#"/v2/collab/shares/share/live auth=Some("Bearer kanso_test") origin=None"#
        );
        assert_eq!(sent, b"mine");
    }
}
