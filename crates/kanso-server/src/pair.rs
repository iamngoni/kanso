//! Pairing (issue #390, step 12): how a tool on this computer, Raycast
//! first, asks for an API key without anyone copying one.
//!
//! ```text
//! POST /pair            {client, name, access: "capture", books, secret}
//!   -> 202 {request, code, expires}
//! GET  /pair/<request>  header X-Kanso-Pair-Secret: <secret>
//!   -> 202 {state: "waiting"} | 200 {state: "allowed", key, id}
//!    | 403 {state: "denied"}  | 410 {state: "expired" | "gone"}
//! ```
//!
//! These are the only routes that take no key, so they are narrow:
//!
//! - **Loopback only.** A peer that is not `127.0.0.1` or `::1` is refused,
//!   even when the server is bound to more. The `Host` must name this
//!   computer (`localhost`, `127.0.0.1`, `[::1]`), which stops a DNS
//!   rebinding page, and a request with an `Origin` is refused: a browser
//!   sends one, a tool does not.
//! - **One request waits at a time**, for [`EXPIRY`], and at most
//!   [`MAX_PER_MINUTE`] are taken a minute.
//! - **A person decides.** [`Pairing::pending`] gives the sheet the name
//!   the tool gave itself and the [`code`](PendingPair::code) to compare;
//!   [`Pairing::approve`] or [`Pairing::deny`] answers it.
//! - **Only a Capture key is ever made**, whatever was asked.
//! - **The key is minted at approval and handed over once**, to the
//!   poller that holds the secret (kept only as its SHA-256). A key not
//!   fetched within [`DELIVERY`] is revoked.
//!
//! Nothing here logs: the key, the secret and the request id are never
//! written anywhere, and `Debug` prints none of them.

use std::fmt;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex, MutexGuard};

use async_channel::{Receiver, Sender};
use axum::Router;
use axum::extract::{ConnectInfo, Path, Request, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use kanso_store::KeyAccess;
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest as _, Sha256};
use subtle::ConstantTimeEq as _;

use crate::error::ApiError;
use crate::key_model::{KeyId, NewKey};
use crate::keys::KeyStore;

/// The header the poller sends its secret in.
pub(crate) const SECRET_HEADER: &str = "x-kanso-pair-secret";
/// How long a request waits for a person, in seconds.
pub const EXPIRY: i64 = 120;
/// How long after Allow the key waits to be fetched, in seconds. After
/// that it is revoked.
pub const DELIVERY: i64 = 30;
/// How many requests are taken in a minute.
pub const MAX_PER_MINUTE: usize = 5;
/// How long a finished request is still answered, in seconds.
const KEEP: i64 = 60;
/// Longest body of a pairing request.
const MAX_BODY: usize = 4 * 1024;
const MAX_BOOKS: usize = 100;

/// A request for the sheet to show.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingPair {
    /// The request's id, which [`Pairing::approve`] and [`Pairing::deny`]
    /// take, so an answer never lands on a newer request.
    pub request: String,
    /// What the tool calls itself, for "calling itself <name>".
    pub name: String,
    /// Which tool it says it is: `raycast`.
    pub client: String,
    /// Six digits, the same on the sheet and in the tool.
    pub code: String,
    /// The notebooks it asked to read; `None` is all of them.
    pub books: Option<Vec<String>>,
    /// Whole seconds left to answer.
    pub expires_in: i64,
    /// Another app asked while this one waited. The sheet warns: it may be
    /// a program holding the slot to be the one a hurried person allows.
    pub contested: bool,
}

/// What the app learns without polling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PairEvent {
    /// A request was accepted: raise the window and show the sheet.
    Requested,
}

/// Why an answer did not take.
#[derive(Debug)]
pub enum PairError {
    /// That request is not waiting: answered, expired, or replaced.
    NotPending,
    /// The key could not be made.
    Store(kanso_store::StoreError),
}

impl fmt::Display for PairError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PairError::NotPending => f.write_str("That request is no longer waiting."),
            PairError::Store(e) => write!(f, "Kanso could not make the key: {e}"),
        }
    }
}

impl std::error::Error for PairError {}

enum Phase {
    Waiting,
    /// Approved. `key` is there until it is fetched.
    Allowed {
        key: Option<String>,
        id: KeyId,
        at: i64,
    },
    Denied {
        at: i64,
    },
    Expired {
        at: i64,
    },
    Delivered {
        at: i64,
    },
}

struct PairRequest {
    id: String,
    secret_hash: [u8; 32],
    code: String,
    client: String,
    name: String,
    books: Option<Vec<String>>,
    created: i64,
    /// Another request was refused while this one waited.
    contested: bool,
    phase: Phase,
}

impl fmt::Debug for PairRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PairRequest")
            .field("client", &self.client)
            .finish_non_exhaustive()
    }
}

/// The shared state, kept by the [`KeyStore`].
pub(crate) struct PairState {
    current: Option<PairRequest>,
    /// When the requests of the last minute were taken, in ms.
    attempts: Vec<i64>,
    tx: Sender<PairEvent>,
    rx: Receiver<PairEvent>,
}

impl Default for PairState {
    fn default() -> Self {
        let (tx, rx) = async_channel::bounded(16);
        PairState {
            current: None,
            attempts: Vec::new(),
            tx,
            rx,
        }
    }
}

impl fmt::Debug for PairState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PairState").finish_non_exhaustive()
    }
}

/// What a poll finds.
#[derive(PartialEq, Eq)]
pub(crate) enum Poll {
    Waiting,
    Key {
        key: String,
        id: KeyId,
    },
    Denied,
    Expired,
    Gone,
    /// No such request, or not this secret: the two look the same.
    Unknown,
}

impl fmt::Debug for Poll {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Poll::Waiting => f.write_str("Waiting"),
            Poll::Key { .. } => f.write_str("Key(<redacted>)"),
            Poll::Denied => f.write_str("Denied"),
            Poll::Expired => f.write_str("Expired"),
            Poll::Gone => f.write_str("Gone"),
            Poll::Unknown => f.write_str("Unknown"),
        }
    }
}

/// The handle the routes and the app's sheet share. Cloning shares the
/// same request.
#[derive(Clone)]
pub struct Pairing {
    keys: KeyStore,
    state: Arc<Mutex<PairState>>,
}

impl fmt::Debug for Pairing {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Pairing").finish_non_exhaustive()
    }
}

fn hash(text: &str) -> [u8; 32] {
    Sha256::digest(text.as_bytes()).into()
}

fn random_bytes<const N: usize>() -> [u8; N] {
    let mut bytes = [0u8; N];
    getrandom::fill(&mut bytes).expect("the system's random source failed");
    bytes
}

/// Six digits, uniform.
fn random_code() -> String {
    // Reject the top sliver of the range so no digit is likelier.
    const LIMIT: u32 = u32::MAX - (u32::MAX % 1_000_000);
    loop {
        let n = u32::from_le_bytes(random_bytes::<4>());
        if n < LIMIT {
            return format!("{:06}", n % 1_000_000);
        }
    }
}

/// What a tool asks for.
pub(crate) struct NewPair {
    pub client: String,
    pub name: String,
    pub books: Option<Vec<String>>,
    pub secret: String,
}

/// What a tool is given when its request is taken.
pub(crate) struct Started {
    pub request: String,
    pub code: String,
}

impl Pairing {
    pub(crate) fn new(keys: KeyStore, state: Arc<Mutex<PairState>>) -> Pairing {
        Pairing { keys, state }
    }

    // The state is replaced whole by each writer, so a poisoned lock still
    // holds a consistent one.
    fn lock(&self) -> MutexGuard<'_, PairState> {
        self.state.lock().unwrap_or_else(|p| p.into_inner())
    }

    /// Moves time forward: a request nobody answered expires, a key nobody
    /// fetched is revoked, and a finished request is forgotten.
    fn tick(&self, state: &mut PairState, now: i64) {
        let Some(request) = state.current.as_mut() else {
            return;
        };
        let secs = |ms: i64| ms / 1000;
        match &request.phase {
            Phase::Waiting if secs(now - request.created) >= EXPIRY => {
                request.phase = Phase::Expired { at: now };
            }
            Phase::Allowed {
                key: Some(_),
                id,
                at,
            } if secs(now - at) >= DELIVERY => {
                // Never fetched: nobody holds this key, so nobody may.
                let _ = self.keys.revoke(id);
                request.phase = Phase::Expired { at: now };
            }
            Phase::Denied { at } | Phase::Expired { at } | Phase::Delivered { at }
                if secs(now - at) >= KEEP =>
            {
                state.current = None;
            }
            Phase::Allowed { key: None, at, .. } if secs(now - at) >= KEEP => {
                state.current = None;
            }
            _ => {}
        }
    }

    /// Takes a request, or says why not.
    pub(crate) fn start(&self, new: NewPair) -> Result<Started, ApiError> {
        let now = self.keys.now();
        let mut state = self.lock();
        self.tick(&mut state, now);
        state.attempts.retain(|at| now - *at < 60_000);
        if state.attempts.len() >= MAX_PER_MINUTE {
            return Err(ApiError::too_many_requests(
                "Too many pairing requests. Wait a minute and try again.",
            ));
        }
        let blocking = state.current.as_ref().is_some_and(|r| {
            matches!(
                r.phase,
                Phase::Waiting | Phase::Allowed { key: Some(_), .. }
            )
        });
        if blocking {
            // A refused attempt costs the person nothing, so it does not
            // count against the limit; it does mark the waiting request.
            if let Some(current) = state.current.as_mut() {
                current.contested = true;
            }
            return Err(ApiError::too_many_requests(
                "A request is already waiting in Kanso.",
            ));
        }
        let request = PairRequest {
            id: URL_SAFE_NO_PAD.encode(random_bytes::<16>()),
            secret_hash: hash(&new.secret),
            code: random_code(),
            client: new.client,
            name: new.name,
            books: new.books,
            created: now,
            contested: false,
            phase: Phase::Waiting,
        };
        state.attempts.push(now);
        let started = Started {
            request: request.id.clone(),
            code: request.code.clone(),
        };
        state.current = Some(request);
        let _ = state.tx.try_send(PairEvent::Requested);
        Ok(started)
    }

    /// What `request` holds for the poller that gives `secret`. The key
    /// comes out once.
    pub(crate) fn poll(&self, request: &str, secret: &str) -> Poll {
        let now = self.keys.now();
        let mut state = self.lock();
        self.tick(&mut state, now);
        let Some(current) = state.current.as_mut() else {
            return Poll::Unknown;
        };
        // Both are compared, in constant time, whatever the first says.
        let same_id = current.id.as_bytes().ct_eq(request.as_bytes());
        let same_secret = current.secret_hash.ct_eq(&hash(secret));
        if !bool::from(same_id & same_secret) {
            return Poll::Unknown;
        }
        match &mut current.phase {
            Phase::Waiting => Poll::Waiting,
            Phase::Allowed { key, id, .. } => match key.take() {
                Some(key) => {
                    let id = id.clone();
                    current.phase = Phase::Delivered { at: now };
                    Poll::Key { key, id }
                }
                None => Poll::Gone,
            },
            Phase::Denied { .. } => Poll::Denied,
            Phase::Expired { .. } => Poll::Expired,
            Phase::Delivered { .. } => Poll::Gone,
        }
    }

    /// The request a person is being asked about, if one is waiting.
    pub fn pending(&self) -> Option<PendingPair> {
        let now = self.keys.now();
        let mut state = self.lock();
        self.tick(&mut state, now);
        let request = state.current.as_ref()?;
        matches!(request.phase, Phase::Waiting).then(|| PendingPair {
            request: request.id.clone(),
            name: request.name.clone(),
            client: request.client.clone(),
            code: request.code.clone(),
            books: request.books.clone(),
            expires_in: (EXPIRY - (now - request.created) / 1000).max(0),
            contested: request.contested,
        })
    }

    /// Takes the app's events: a request arrived.
    pub fn events(&self) -> Receiver<PairEvent> {
        self.lock().rx.clone()
    }

    /// Allow: makes the key, a Capture key whatever was asked, and holds it
    /// for the poller. Returns the key's id, never the key.
    pub fn approve(&self, request: &str) -> Result<KeyId, PairError> {
        let now = self.keys.now();
        let mut state = self.lock();
        self.tick(&mut state, now);
        let current = state
            .current
            .as_mut()
            .filter(|r| bool::from(r.id.as_bytes().ct_eq(request.as_bytes())))
            .filter(|r| matches!(r.phase, Phase::Waiting))
            .ok_or(PairError::NotPending)?;
        let minted = self
            .keys
            .mint(NewKey {
                name: current.name.clone(),
                client: current.client.clone(),
                access: KeyAccess::Capture,
                books: current.books.clone(),
            })
            .map_err(PairError::Store)?;
        let id = minted.id.clone();
        current.phase = Phase::Allowed {
            key: Some(minted.key),
            id: minted.id,
            at: now,
        };
        Ok(id)
    }

    /// Deny: the poller is told no. `false` when that request is not
    /// waiting.
    pub fn deny(&self, request: &str) -> bool {
        let now = self.keys.now();
        let mut state = self.lock();
        self.tick(&mut state, now);
        match state
            .current
            .as_mut()
            .filter(|r| bool::from(r.id.as_bytes().ct_eq(request.as_bytes())))
            .filter(|r| matches!(r.phase, Phase::Waiting))
        {
            Some(current) => {
                current.phase = Phase::Denied { at: now };
                true
            }
            None => false,
        }
    }
}

// ---- the routes ----

pub(crate) fn routes(pairing: Pairing) -> Router {
    Router::new()
        .route("/pair", post(start))
        .route("/pair/{request}", get(poll))
        .with_state(pairing)
}

fn refused(reason: &str) -> ApiError {
    ApiError::forbidden(reason.to_owned())
}

/// Whether `host` (a `Host` header) names this computer.
fn is_local_host(host: &str) -> bool {
    let name = match host.strip_prefix('[') {
        Some(rest) => match rest.split_once(']') {
            Some((ip, port)) if port.is_empty() || port.starts_with(':') => {
                return ip == "::1" && port_ok(port);
            }
            _ => return false,
        },
        None => match host.split_once(':') {
            Some((name, port)) if port_ok(&format!(":{port}")) => name,
            Some(_) => return false,
            None => host,
        },
    };
    name.eq_ignore_ascii_case("localhost") || name == "127.0.0.1"
}

fn port_ok(port: &str) -> bool {
    port.is_empty()
        || port
            .strip_prefix(':')
            .is_some_and(|p| !p.is_empty() && p.len() <= 5 && p.bytes().all(|b| b.is_ascii_digit()))
}

/// The checks every pairing request passes first: a loopback peer, a
/// loopback `Host`, and no `Origin`.
fn guard(request: &Request) -> Result<(), ApiError> {
    let peer = request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ConnectInfo(addr)| addr.ip());
    // No peer address is no proof: refused.
    let loopback = peer.is_some_and(|ip: IpAddr| ip.to_canonical().is_loopback());
    if !loopback {
        return Err(refused(
            "Pairing works only from this computer. Create a key in Settings \u{203a} Local API instead.",
        ));
    }
    let host = request
        .headers()
        .get(header::HOST)
        .and_then(|value| value.to_str().ok());
    if !host.is_some_and(is_local_host) {
        return Err(refused(
            "Pairing answers only to localhost, 127.0.0.1 or [::1].",
        ));
    }
    if request.headers().contains_key(header::ORIGIN) {
        return Err(refused("Pairing is for apps, not web pages."));
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Asked {
    client: String,
    name: String,
    #[serde(default)]
    access: Option<String>,
    #[serde(default)]
    books: Option<Vec<String>>,
    secret: String,
}

/// Characters that draw nothing or change how text runs (Unicode format
/// characters and the bidi controls), which could make a name on the sheet
/// read as another.
fn is_invisible(c: char) -> bool {
    matches!(c as u32,
        0x00AD | 0x0600..=0x0605 | 0x061C | 0x06DD | 0x070F | 0x180E
        | 0x200B..=0x200F | 0x2028..=0x202E | 0x2060..=0x206F
        | 0xFEFF | 0xFFF9..=0xFFFB | 0xE0000..=0xE007F)
}

fn is_secret(text: &str) -> bool {
    text.len() == 32
        && text
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

fn is_client(text: &str) -> bool {
    (1..=32).contains(&text.len())
        && text
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

fn check(asked: Asked) -> Result<NewPair, ApiError> {
    // Pairing makes Capture keys and nothing else. Asking for more is
    // refused, not trimmed, so the tool learns it will never get it.
    if asked.access.as_deref().is_some_and(|a| a != "capture") {
        return Err(refused(
            "Pairing can only make a key that adds notes and ticks tasks. Create a key with more access in Settings \u{203a} Local API.",
        ));
    }
    if !is_secret(&asked.secret) {
        return Err(ApiError::bad_request(
            "`secret` must be 32 characters of base64url.",
        ));
    }
    if !is_client(&asked.client) {
        return Err(ApiError::bad_request(
            "`client` must be 1 to 32 lowercase letters, digits or dashes.",
        ));
    }
    let name = asked.name.trim().to_owned();
    if name.is_empty()
        || name.chars().count() > 60
        || name.chars().any(|c| c.is_control() || is_invisible(c))
    {
        return Err(ApiError::bad_request(
            "`name` must be 1 to 60 characters, with no control or invisible characters.",
        ));
    }
    if let Some(books) = &asked.books
        && (books.len() > MAX_BOOKS || books.iter().any(|b| b.is_empty() || b.len() > 128))
    {
        return Err(ApiError::bad_request("`books` is not a list of notebooks."));
    }
    Ok(NewPair {
        client: asked.client,
        name,
        books: asked.books,
        secret: asked.secret,
    })
}

async fn start(State(pairing): State<Pairing>, request: Request) -> Response {
    if let Err(e) = guard(&request) {
        return e.into_response();
    }
    let body = match axum::body::to_bytes(request.into_body(), MAX_BODY).await {
        Ok(body) => body,
        Err(_) => return ApiError::too_large().into_response(),
    };
    let asked: Asked = match serde_json::from_slice(&body) {
        Ok(asked) => asked,
        Err(_) => {
            return ApiError::bad_request(
                "Send JSON: {client, name, access: \"capture\", books, secret}.",
            )
            .into_response();
        }
    };
    let new = match check(asked) {
        Ok(new) => new,
        Err(e) => return e.into_response(),
    };
    // The key store does its work on this thread: a tick may revoke.
    match pairing.start(new) {
        Ok(started) => (
            StatusCode::ACCEPTED,
            axum::Json(json!({
                "request": started.request,
                "code": started.code,
                "expires": EXPIRY,
            })),
        )
            .into_response(),
        Err(e) => e.into_response(),
    }
}

async fn poll(
    State(pairing): State<Pairing>,
    Path(id): Path<String>,
    request: Request,
) -> Response {
    if let Err(e) = guard(&request) {
        return e.into_response();
    }
    let secret = header_text(request.headers());
    let state = |status: StatusCode, state: &str| {
        (status, axum::Json(json!({ "state": state }))).into_response()
    };
    match pairing.poll(&id, secret) {
        Poll::Waiting => state(StatusCode::ACCEPTED, "waiting"),
        Poll::Key { key, id } => {
            let mut response = (
                StatusCode::OK,
                axum::Json(json!({ "state": "allowed", "key": key, "id": id.as_str() })),
            )
                .into_response();
            response
                .headers_mut()
                .insert(header::CACHE_CONTROL, "no-store".parse().unwrap());
            response
        }
        Poll::Denied => state(StatusCode::FORBIDDEN, "denied"),
        Poll::Expired => state(StatusCode::GONE, "expired"),
        Poll::Gone => state(StatusCode::GONE, "gone"),
        Poll::Unknown => ApiError::not_found().into_response(),
    }
}

fn header_text(headers: &HeaderMap) -> &str {
    headers
        .get(SECRET_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Method;
    use kanso_store::Store;
    use proptest::prelude::*;
    use proptest::test_runner::{Config, RngAlgorithm, RngSeed, TestRng, TestRunner};
    use tower::ServiceExt as _;

    const SECRET: &str = "abcdefghijklmnopqrstuvwxyz012345";
    const OTHER_SECRET: &str = "ZYXWVUTSRQPONMLKJIHGFEDCBA543210";

    struct Fixture {
        _dir: tempfile::TempDir,
        clock: Arc<Mutex<i64>>,
        keys: KeyStore,
        pairing: Pairing,
        app: Router,
    }

    fn fixture() -> Fixture {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
        let clock = Arc::new(Mutex::new(1_000_000i64));
        let read = clock.clone();
        let keys = KeyStore::with_clock(store, Arc::new(move || *read.lock().unwrap())).unwrap();
        let pairing = keys.pairing();
        let app = routes(pairing.clone());
        Fixture {
            _dir: dir,
            clock,
            keys,
            pairing,
            app,
        }
    }

    impl Fixture {
        fn advance(&self, secs: i64) {
            *self.clock.lock().unwrap() += secs * 1000;
        }
    }

    fn peer(ip: &str) -> ConnectInfo<SocketAddr> {
        ConnectInfo(SocketAddr::new(ip.parse().unwrap(), 50_000))
    }

    fn body(secret: &str) -> serde_json::Value {
        json!({"client": "raycast", "name": "Raycast", "access": "capture", "books": null, "secret": secret})
    }

    async fn send(
        app: &Router,
        request: axum::http::request::Builder,
        peer: Option<ConnectInfo<SocketAddr>>,
        body: Option<serde_json::Value>,
    ) -> (StatusCode, serde_json::Value) {
        let mut request = request
            .body(match body {
                Some(body) => Body::from(body.to_string()),
                None => Body::empty(),
            })
            .unwrap();
        if let Some(peer) = peer {
            request.extensions_mut().insert(peer);
        }
        let response = app.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = axum::body::to_bytes(response.into_body(), 1 << 20)
            .await
            .unwrap();
        (
            status,
            serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null),
        )
    }

    fn post_req(host: &str) -> axum::http::request::Builder {
        Request::builder()
            .method(Method::POST)
            .uri("/pair")
            .header(header::HOST, host)
    }

    fn get_req(host: &str, id: &str, secret: &str) -> axum::http::request::Builder {
        Request::builder()
            .method(Method::GET)
            .uri(format!("/pair/{id}"))
            .header(header::HOST, host)
            .header(SECRET_HEADER, secret)
    }

    async fn pair(f: &Fixture, secret: &str) -> (String, String) {
        let (status, json) = send(
            &f.app,
            post_req("127.0.0.1:19840"),
            Some(peer("127.0.0.1")),
            Some(body(secret)),
        )
        .await;
        assert_eq!(status, StatusCode::ACCEPTED, "{json}");
        assert_eq!(json["expires"], EXPIRY);
        (
            json["request"].as_str().unwrap().to_owned(),
            json["code"].as_str().unwrap().to_owned(),
        )
    }

    async fn poll_with(f: &Fixture, id: &str, secret: &str) -> (StatusCode, serde_json::Value) {
        send(
            &f.app,
            get_req("localhost:19840", id, secret),
            Some(peer("::1")),
            None,
        )
        .await
    }

    #[tokio::test]
    async fn a_request_gets_an_id_a_code_and_a_wait() {
        let f = fixture();
        let (id, code) = pair(&f, SECRET).await;
        assert_eq!(id.len(), 22);
        assert_eq!(code.len(), 6);
        assert!(code.bytes().all(|b| b.is_ascii_digit()));
        let pending = f.pairing.pending().unwrap();
        assert_eq!(pending.request, id);
        assert_eq!(pending.code, code);
        assert_eq!(pending.name, "Raycast");
        assert_eq!(pending.expires_in, EXPIRY);
        assert_eq!(poll_with(&f, &id, SECRET).await.0, StatusCode::ACCEPTED);
        assert_eq!(f.pairing.events().try_recv().unwrap(), PairEvent::Requested);
    }

    #[tokio::test]
    async fn only_loopback_peers_are_served() {
        let f = fixture();
        for ip in [
            "192.168.1.20",
            "10.0.0.5",
            "8.8.8.8",
            "fe80::1",
            "2001:db8::1",
        ] {
            let (status, _) = send(
                &f.app,
                post_req("127.0.0.1"),
                Some(peer(ip)),
                Some(body(SECRET)),
            )
            .await;
            assert_eq!(status, StatusCode::FORBIDDEN, "{ip}");
            let (status, _) = send(
                &f.app,
                get_req("127.0.0.1", "x", SECRET),
                Some(peer(ip)),
                None,
            )
            .await;
            assert_eq!(status, StatusCode::FORBIDDEN, "{ip}");
        }
        // No peer address at all is refused too.
        let (status, _) = send(&f.app, post_req("127.0.0.1"), None, Some(body(SECRET))).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert!(f.pairing.pending().is_none());
        for ip in ["127.0.0.1", "127.0.0.2", "::1", "::ffff:127.0.0.1"] {
            let (status, _) = send(
                &f.app,
                post_req("127.0.0.1"),
                Some(peer(ip)),
                Some(body(SECRET)),
            )
            .await;
            assert_eq!(status, StatusCode::ACCEPTED, "{ip}");
            f.advance(61);
            f.pairing.deny(&f.pairing.pending().unwrap().request);
        }
    }

    #[tokio::test]
    async fn a_host_that_is_not_this_computer_is_refused() {
        let f = fixture();
        for host in [
            "evil.example",
            "evil.example:19840",
            "127.0.0.1.evil.example",
            "localhost.evil.example",
            "192.168.1.5:19840",
            "0.0.0.0:19840",
            "[::2]:19840",
            "localhost:abc",
            "localhost:",
            "",
        ] {
            let (status, _) = send(
                &f.app,
                post_req(host),
                Some(peer("127.0.0.1")),
                Some(body(SECRET)),
            )
            .await;
            assert_eq!(status, StatusCode::FORBIDDEN, "{host:?}");
        }
        // No Host header at all.
        let request = Request::builder().method(Method::POST).uri("/pair");
        let (status, _) = send(&f.app, request, Some(peer("127.0.0.1")), Some(body(SECRET))).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        for host in [
            "localhost",
            "LOCALHOST:19840",
            "127.0.0.1:1",
            "[::1]",
            "[::1]:19840",
        ] {
            assert!(is_local_host(host), "{host}");
        }
        assert!(f.pairing.pending().is_none());
    }

    #[tokio::test]
    async fn a_request_from_a_web_page_is_refused() {
        let f = fixture();
        for origin in ["https://evil.example", "http://localhost:3000", "null"] {
            let (status, _) = send(
                &f.app,
                post_req("localhost").header(header::ORIGIN, origin),
                Some(peer("127.0.0.1")),
                Some(body(SECRET)),
            )
            .await;
            assert_eq!(status, StatusCode::FORBIDDEN, "{origin}");
        }
        assert!(f.pairing.pending().is_none());
        let (id, _) = pair(&f, SECRET).await;
        let (status, _) = send(
            &f.app,
            get_req("localhost", &id, SECRET).header(header::ORIGIN, "https://evil.example"),
            Some(peer("127.0.0.1")),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn one_request_waits_at_a_time() {
        let f = fixture();
        pair(&f, SECRET).await;
        let (status, json) = send(
            &f.app,
            post_req("localhost"),
            Some(peer("127.0.0.1")),
            Some(body(OTHER_SECRET)),
        )
        .await;
        assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(json["reason"], "A request is already waiting in Kanso.");
        // Once it is answered, the next is taken.
        let first = f.pairing.pending().unwrap();
        assert!(f.pairing.deny(&first.request));
        let (id, _) = pair(&f, OTHER_SECRET).await;
        assert_ne!(id, first.request);
    }

    #[tokio::test]
    async fn a_second_app_marks_the_waiting_request_contested() {
        let f = fixture();
        pair(&f, SECRET).await;
        assert!(!f.pairing.pending().unwrap().contested);
        let (status, _) = send(
            &f.app,
            post_req("localhost"),
            Some(peer("127.0.0.1")),
            Some(body(OTHER_SECRET)),
        )
        .await;
        assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
        assert!(f.pairing.pending().unwrap().contested);
        // The next request starts clean.
        let id = f.pairing.pending().unwrap().request;
        f.pairing.deny(&id);
        pair(&f, SECRET).await;
        assert!(!f.pairing.pending().unwrap().contested);
    }

    #[tokio::test]
    async fn refused_attempts_do_not_burn_the_rate_limit() {
        let f = fixture();
        let (id, _) = pair(&f, SECRET).await;
        for _ in 0..20 {
            let (status, _) = send(
                &f.app,
                post_req("localhost"),
                Some(peer("127.0.0.1")),
                Some(body(OTHER_SECRET)),
            )
            .await;
            assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
        }
        f.pairing.deny(&id);
        // Four more accepted requests fit in the minute: 5 in all.
        for _ in 0..MAX_PER_MINUTE - 1 {
            let (id, _) = pair(&f, SECRET).await;
            f.pairing.deny(&id);
        }
        let (status, json) = send(
            &f.app,
            post_req("localhost"),
            Some(peer("127.0.0.1")),
            Some(body(SECRET)),
        )
        .await;
        assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
        assert!(json["reason"].as_str().unwrap().contains("Too many"));
    }

    #[tokio::test]
    async fn a_name_with_invisible_or_bidi_characters_is_refused() {
        let f = fixture();
        // "Raycast" reversed by a right-to-left override.
        for name in [
            "\u{202E}tsacyaR",
            "Ray\u{200B}cast",
            "Ray\u{200F}cast",
            "\u{2066}Raycast\u{2069}",
            "\u{FEFF}Raycast",
            "Ray\u{00AD}cast",
            "Ray\u{2028}cast",
        ] {
            let mut asked = body(SECRET);
            asked["name"] = json!(name);
            let (status, _) = send(
                &f.app,
                post_req("localhost"),
                Some(peer("127.0.0.1")),
                Some(asked),
            )
            .await;
            assert_eq!(status, StatusCode::BAD_REQUEST, "{name:?}");
        }
        assert!(f.pairing.pending().is_none());
        // Ordinary non-ASCII names still pass.
        let mut asked = body(SECRET);
        asked["name"] = json!("Raycast \u{00E9}\u{65E5}\u{672C}");
        let (status, _) = send(
            &f.app,
            post_req("localhost"),
            Some(peer("127.0.0.1")),
            Some(asked),
        )
        .await;
        assert_eq!(status, StatusCode::ACCEPTED);
    }

    #[tokio::test]
    async fn at_most_five_requests_a_minute() {
        let f = fixture();
        for _ in 0..MAX_PER_MINUTE {
            let (id, _) = pair(&f, SECRET).await;
            assert!(f.pairing.deny(&id));
        }
        let (status, json) = send(
            &f.app,
            post_req("localhost"),
            Some(peer("127.0.0.1")),
            Some(body(SECRET)),
        )
        .await;
        assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
        assert!(json["reason"].as_str().unwrap().contains("Too many"));
        assert!(f.pairing.pending().is_none());
        f.advance(61);
        pair(&f, SECRET).await;
    }

    #[tokio::test]
    async fn a_request_expires() {
        let f = fixture();
        let (id, _) = pair(&f, SECRET).await;
        f.advance(EXPIRY - 1);
        assert_eq!(poll_with(&f, &id, SECRET).await.0, StatusCode::ACCEPTED);
        assert_eq!(f.pairing.pending().unwrap().expires_in, 1);
        f.advance(1);
        let (status, json) = poll_with(&f, &id, SECRET).await;
        assert_eq!(status, StatusCode::GONE);
        assert_eq!(json["state"], "expired");
        assert!(f.pairing.pending().is_none());
        // Too late to allow.
        assert!(matches!(f.pairing.approve(&id), Err(PairError::NotPending)));
        assert!(f.keys.verify("kcl_x").is_none());
    }

    #[tokio::test]
    async fn allow_hands_over_a_capture_key_once() {
        let f = fixture();
        let (id, _) = pair(&f, SECRET).await;
        let key_id = f.pairing.approve(&id).unwrap();
        let (status, json) = poll_with(&f, &id, SECRET).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["state"], "allowed");
        assert_eq!(json["id"], key_id.as_str());
        let key = json["key"].as_str().unwrap().to_owned();
        assert!(key.starts_with("kcl_") && key.len() == crate::key_model::KEY_LEN);
        // It is a live Capture key for the client that asked.
        match f.keys.verify(&key).unwrap() {
            crate::scope::Caller::Key { access, .. } => {
                assert_eq!(access, KeyAccess::Capture)
            }
            other => panic!("{other:?}"),
        }
        // A second fetch gets nothing, with the right secret or not.
        for secret in [SECRET, OTHER_SECRET] {
            let (status, json) = poll_with(&f, &id, secret).await;
            assert_ne!(status, StatusCode::OK);
            assert!(json.get("key").is_none(), "{json}");
        }
        assert_eq!(poll_with(&f, &id, SECRET).await.0, StatusCode::GONE);
    }

    #[tokio::test]
    async fn only_the_holder_of_the_secret_gets_the_key() {
        let f = fixture();
        let (id, _) = pair(&f, SECRET).await;
        f.pairing.approve(&id).unwrap();
        for secret in [OTHER_SECRET, "", "short"] {
            let (status, json) = poll_with(&f, &id, secret).await;
            assert_eq!(status, StatusCode::NOT_FOUND, "{secret}");
            assert!(json.get("key").is_none());
        }
        let (status, _) = poll_with(&f, "not-the-id", SECRET).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        // The real poller still gets it: guesses did not spend it.
        assert_eq!(poll_with(&f, &id, SECRET).await.0, StatusCode::OK);
    }

    #[tokio::test]
    async fn deny_tells_the_poller_no_and_makes_no_key() {
        let f = fixture();
        let (id, _) = pair(&f, SECRET).await;
        assert!(f.pairing.deny(&id));
        let (status, json) = poll_with(&f, &id, SECRET).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(json["state"], "denied");
        assert!(matches!(f.pairing.approve(&id), Err(PairError::NotPending)));
        assert!(!f.pairing.deny(&id));
    }

    #[tokio::test]
    async fn only_a_capture_key_can_be_asked_for() {
        let f = fixture();
        for access in ["write", "suggest", "read", "admin", ""] {
            let mut asked = body(SECRET);
            asked["access"] = json!(access);
            let (status, _) = send(
                &f.app,
                post_req("localhost"),
                Some(peer("127.0.0.1")),
                Some(asked),
            )
            .await;
            assert_eq!(status, StatusCode::FORBIDDEN, "{access:?}");
        }
        assert!(f.pairing.pending().is_none());
        // Leaving access out asks for Capture, which is all there is.
        let mut asked = body(SECRET);
        asked.as_object_mut().unwrap().remove("access");
        let (status, _) = send(
            &f.app,
            post_req("localhost"),
            Some(peer("127.0.0.1")),
            Some(asked),
        )
        .await;
        assert_eq!(status, StatusCode::ACCEPTED);
        let id = f.pairing.pending().unwrap().request;
        f.pairing.approve(&id).unwrap();
        let (_, json) = poll_with(&f, &id, SECRET).await;
        let key = json["key"].as_str().unwrap();
        assert!(matches!(
            f.keys.verify(key).unwrap(),
            crate::scope::Caller::Key {
                access: KeyAccess::Capture,
                ..
            }
        ));
    }

    #[tokio::test]
    async fn a_bad_request_is_refused_before_it_waits() {
        let f = fixture();
        let mut cases = Vec::new();
        for (field, value) in [
            ("secret", json!("tooshort")),
            ("secret", json!("abcdefghijklmnopqrstuvwxyz01234!")),
            ("client", json!("Ray Cast")),
            ("client", json!("")),
            ("client", json!("x".repeat(33))),
            ("name", json!("   ")),
            ("name", json!("a\nb")),
            ("name", json!("n".repeat(61))),
            ("books", json!([""])),
            ("books", json!(vec!["b"; 101])),
            ("extra", json!(true)),
        ] {
            let mut asked = body(SECRET);
            asked[field] = value;
            cases.push(asked);
        }
        cases.push(json!({"client": "raycast"}));
        for asked in cases {
            let (status, _) = send(
                &f.app,
                post_req("localhost"),
                Some(peer("127.0.0.1")),
                Some(asked.clone()),
            )
            .await;
            assert_eq!(status, StatusCode::BAD_REQUEST, "{asked}");
        }
        let request = post_req("localhost").body(Body::from("not json")).unwrap();
        let mut request = request;
        request.extensions_mut().insert(peer("127.0.0.1"));
        let response = f.app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        // And a body over the cap is refused, unread.
        let mut big = body(SECRET);
        big["name"] = json!("n".repeat(MAX_BODY));
        let (status, _) = send(
            &f.app,
            post_req("localhost"),
            Some(peer("127.0.0.1")),
            Some(big),
        )
        .await;
        assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
        assert!(f.pairing.pending().is_none());
    }

    #[tokio::test]
    async fn a_key_that_is_not_fetched_is_revoked() {
        let f = fixture();
        let (id, _) = pair(&f, SECRET).await;
        f.pairing.approve(&id).unwrap();
        let live_before = format!("{:?}", f.keys);
        assert!(live_before.contains("live: 1"));
        f.advance(DELIVERY);
        let (status, json) = poll_with(&f, &id, SECRET).await;
        assert_eq!(status, StatusCode::GONE);
        assert!(json.get("key").is_none());
        assert!(format!("{:?}", f.keys).contains("live: 0"));
    }

    #[tokio::test]
    async fn the_key_text_is_not_in_any_debug_output() {
        let f = fixture();
        let (id, _) = pair(&f, SECRET).await;
        f.pairing.approve(&id).unwrap();
        let shown = {
            let state = f.pairing.lock();
            format!("{:?} {:?} {:?}", state, state.current, f.pairing)
        };
        let key = match f.pairing.poll(&id, SECRET) {
            Poll::Key { key, .. } => key,
            other => panic!("{other:?}"),
        };
        assert!(!shown.contains(&key[4..]) && !shown.contains(SECRET) && !shown.contains(&id));
        assert_eq!(
            format!(
                "{:?}",
                Poll::Key {
                    key: key.clone(),
                    id: KeyId("i".into())
                }
            ),
            "Key(<redacted>)"
        );
        assert!(!format!("{:?}", f.keys).contains(&key));
    }

    #[test]
    fn the_secret_is_kept_only_as_a_hash() {
        let f = fixture();
        let new = NewPair {
            client: "raycast".into(),
            name: "Raycast".into(),
            books: None,
            secret: SECRET.into(),
        };
        f.pairing.start(new).unwrap();
        let state = f.pairing.lock();
        let request = state.current.as_ref().unwrap();
        assert_eq!(request.secret_hash, hash(SECRET));
    }

    /// A real listener, to see that the peer address reaches the routes.
    #[test]
    fn a_real_socket_pairs_from_loopback() {
        let f = fixture();
        let app = routes(f.pairing.clone());
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let listener = runtime
            .block_on(tokio::net::TcpListener::bind("127.0.0.1:0"))
            .unwrap();
        let addr = listener.local_addr().unwrap();
        runtime.spawn(async move {
            let _ = axum::serve(
                listener,
                app.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .await;
        });
        let client = reqwest::blocking::Client::new();
        let response = client
            .post(format!("http://{addr}/pair"))
            .json(&body(SECRET))
            .send()
            .unwrap();
        assert_eq!(response.status(), 202);
        let json: serde_json::Value = response.json().unwrap();
        let id = json["request"].as_str().unwrap().to_owned();
        f.pairing.approve(&id).unwrap();
        let got: serde_json::Value = client
            .get(format!("http://{addr}/pair/{id}"))
            .header(SECRET_HEADER, SECRET)
            .send()
            .unwrap()
            .json()
            .unwrap();
        assert_eq!(got["state"], "allowed");
        // A browser's Origin on the same socket is refused.
        let refused = client
            .post(format!("http://{addr}/pair"))
            .header("Origin", "https://evil.example")
            .json(&body(SECRET))
            .send()
            .unwrap();
        assert_eq!(refused.status(), 403);
        // And a Host that is not local, however it got here.
        let rebound = client
            .post(format!("http://{addr}/pair"))
            .header("Host", "evil.example")
            .json(&body(SECRET))
            .send()
            .unwrap();
        assert_eq!(rebound.status(), 403);
        runtime.shutdown_background();
    }

    #[test]
    fn the_whole_router_serves_pairing_without_a_key() {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
        let keys = KeyStore::open(store.clone()).unwrap();
        let config = crate::ServerConfig {
            bind: "127.0.0.1:0".parse().unwrap(),
        };
        let app = crate::router(store, &config, keys);
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let listener = runtime
            .block_on(tokio::net::TcpListener::bind("127.0.0.1:0"))
            .unwrap();
        let addr = listener.local_addr().unwrap();
        runtime.spawn(async move {
            let _ = axum::serve(
                listener,
                app.into_make_service_with_connect_info::<SocketAddr>(),
            )
            .await;
        });
        let client = reqwest::blocking::Client::new();
        let paired = client
            .post(format!("http://{addr}/pair"))
            .json(&body(SECRET))
            .send()
            .unwrap();
        assert_eq!(paired.status(), 202);
        // Everything else still wants a key.
        for path in ["/", "/notes", "/books"] {
            let status = client
                .get(format!("http://{addr}{path}"))
                .send()
                .unwrap()
                .status();
            assert_eq!(status, 401, "{path}");
        }
        runtime.shutdown_background();
    }

    fn model_run(ops: &[(u8, u8)]) {
        // Ops: 0 start, 1 approve, 2 deny, 3 poll holder, 4 poll stranger,
        // 5 advance (arg seconds).
        let f = fixture();
        let mut current: Option<(String, bool, bool)> = None; // id, approved, delivered
        let mut delivered_ids = std::collections::HashSet::new();
        let mut counter = 0u8;
        for &(op, arg) in ops {
            let now = f.keys.now();
            let _ = now;
            match op % 6 {
                0 => {
                    counter = counter.wrapping_add(1);
                    let new = NewPair {
                        client: "raycast".into(),
                        name: format!("R{counter}"),
                        books: None,
                        secret: SECRET.into(),
                    };
                    if let Ok(started) = f.pairing.start(new) {
                        current = Some((started.request, false, false));
                    }
                }
                1 => {
                    if let Some((id, approved, _)) = current.as_mut()
                        && f.pairing.approve(id).is_ok()
                    {
                        *approved = true;
                    }
                }
                2 => {
                    if let Some((id, ..)) = &current {
                        f.pairing.deny(id);
                    }
                }
                3 => {
                    if let Some((id, approved, delivered)) = current.as_mut()
                        && let Poll::Key { .. } = f.pairing.poll(id, SECRET)
                    {
                        assert!(*approved, "a key without an approval");
                        assert!(!*delivered, "a key delivered twice");
                        assert!(delivered_ids.insert(id.clone()));
                        *delivered = true;
                    }
                }
                4 => {
                    if let Some((id, ..)) = &current {
                        assert!(!matches!(
                            f.pairing.poll(id, OTHER_SECRET),
                            Poll::Key { .. }
                        ));
                    }
                }
                _ => f.advance(i64::from(arg % 70)),
            }
        }
    }

    #[test]
    fn prop_a_key_is_never_returned_twice_and_never_without_approval() {
        let config = Config {
            cases: 96,
            rng_seed: RngSeed::Fixed(0x390_0012),
            failure_persistence: None,
            ..Config::default()
        };
        let mut runner =
            TestRunner::new_with_rng(config, TestRng::from_seed(RngAlgorithm::ChaCha, &[7; 32]));
        runner
            .run(
                &proptest::collection::vec((0u8..6, any::<u8>()), 1..80),
                |ops| {
                    model_run(&ops);
                    Ok(())
                },
            )
            .unwrap();
    }
}
