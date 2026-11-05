//! Helpers shared by the integration tests.
#![allow(dead_code)]

use std::net::SocketAddr;
use std::sync::{Arc, OnceLock};

use axum::Router;
use axum::body::Body;
use axum::http::{HeaderMap, Method, Request, StatusCode, header};
use http_body_util::BodyExt;
use kanso_server::{KeyAccess, KeyStore, ServerConfig, router};
use kanso_store::Store;
use serde_json::{Value, json};
use tower::ServiceExt;

pub const USER: &str = "kanso-test";
/// The key every test request is signed with: it can read and write.
pub const KEY: &str = "kcl_AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
/// The same key, for tools that only take a login: the password.
pub const PASS: &str = KEY;

pub fn config() -> ServerConfig {
    ServerConfig {
        bind: SocketAddr::from(([127, 0, 0, 1], 0)),
    }
}

/// Saves `key` in `store` with `access`, limited to `books` (`None` is
/// every notebook).
pub fn save_key(store: &Store, key: &str, access: KeyAccess, books: Option<Vec<String>>) {
    use sha2::{Digest as _, Sha256};
    let _ = store.api_key_insert(&kanso_store::NewApiKey {
        id: format!("key-{}", &key[key.len() - 8..]),
        name: format!("Test key {}", &key[key.len() - 8..]),
        client: "other".into(),
        token_hash: Sha256::digest(key.as_bytes()).into(),
        access,
        books,
        created_at: 1,
    });
}

/// The library's keys: [`KEY`], which can read and write, is among them.
pub fn keys_for(store: &Arc<Store>) -> KeyStore {
    save_key(store, KEY, KeyAccess::Write, None);
    KeyStore::open(store.clone()).unwrap()
}

/// The `Authorization` header of [`KEY`].
pub fn auth() -> String {
    bearer(KEY)
}

pub fn bearer(key: &str) -> String {
    format!("Bearer {key}")
}

pub struct App {
    pub dir: tempfile::TempDir,
    pub store: Arc<Store>,
    pub router: Router,
    pub keys: KeyStore,
}

pub fn app() -> App {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    let keys = keys_for(&store);
    let router = router(store.clone(), &config(), keys.clone());
    App {
        dir,
        store,
        router,
        keys,
    }
}

/// A runtime for property tests, which are synchronous.
pub fn runtime() -> &'static tokio::runtime::Runtime {
    static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap()
    })
}

pub struct Response {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub bytes: Vec<u8>,
}

impl Response {
    pub fn json(&self) -> Value {
        if self.bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&self.bytes)
                .unwrap_or_else(|err| panic!("{err}: {}", String::from_utf8_lossy(&self.bytes)))
        }
    }
}

pub async fn send(router: &Router, request: Request<Body>) -> Response {
    let response = router.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes()
        .to_vec();
    Response {
        status,
        headers,
        bytes,
    }
}

pub fn request(method: Method, uri: &str, body: Option<&Value>) -> Request<Body> {
    let builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::AUTHORIZATION, auth());
    match body {
        Some(body) => builder
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap(),
        None => builder.body(Body::empty()).unwrap(),
    }
}

/// An authenticated request; returns the status and the JSON body.
pub async fn call(
    router: &Router,
    method: Method,
    uri: &str,
    body: Option<&Value>,
) -> (StatusCode, Value) {
    let response = send(router, request(method, uri, body)).await;
    (response.status, response.json())
}

pub async fn get(router: &Router, uri: &str) -> (StatusCode, Value) {
    call(router, Method::GET, uri, None).await
}

pub async fn post(router: &Router, uri: &str, body: Value) -> (StatusCode, Value) {
    call(router, Method::POST, uri, Some(&body)).await
}

/// POSTs and expects `{ok: true, id, rev}`; returns `(id, rev)`.
pub async fn write(router: &Router, uri: &str, body: Value) -> (String, String) {
    let (status, res) = post(router, uri, body.clone()).await;
    assert_eq!(status, StatusCode::OK, "POST {uri} {body}: {res}");
    assert_eq!(res["ok"], true, "{res}");
    (
        res["id"].as_str().unwrap().to_owned(),
        res["rev"].as_str().unwrap().to_owned(),
    )
}

/// A JSON-RPC request to `/mcp`; returns the whole answer.
pub async fn rpc(router: &Router, method: &str, params: Value) -> Value {
    let message = json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params });
    let (status, answer) = post(router, "/mcp", message).await;
    assert_eq!(status, StatusCode::OK, "{answer}");
    answer
}

/// Calls a tool through `/mcp`; returns its result.
pub async fn tool(router: &Router, name: &str, arguments: Value) -> Value {
    let answer = rpc(
        router,
        "tools/call",
        json!({ "name": name, "arguments": arguments }),
    )
    .await;
    answer["result"].clone()
}

/// The text of a tool result's `index`th content block.
pub fn text(result: &Value, index: usize) -> &str {
    result["content"][index]["text"]
        .as_str()
        .unwrap_or_else(|| panic!("no text block {index} in {result}"))
}

/// The JSON document in a tool result's first text block.
pub fn tool_json(result: &Value) -> Value {
    assert_ne!(result["isError"], true, "{result}");
    serde_json::from_str(text(result, 0)).unwrap()
}

/// A valid 1x1 PNG.
pub const PNG: [u8; 70] = [
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0x15, 0xc4,
    0x89, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x44, 0x41, 0x54, 0x78, 0xda, 0x63, 0xd0, 0xaa, 0xbf, 0xf2,
    0x1f, 0x00, 0x04, 0xd1, 0x02, 0x7d, 0xe2, 0x3a, 0xb3, 0x7a, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45,
    0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
];

/// Percent-encodes a query-string value (everything but RFC 3986 unreserved).
pub fn encode(value: &str) -> String {
    value
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}
