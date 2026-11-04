//! Kanso's local HTTP API.
//!
//! The REST endpoints speak a widely used local-server REST API, wire for
//! wire, so the MCP server `@inkdropapp/mcp-server` and scripts written for
//! that API work against Kanso unchanged. `/mcp` serves the same 17 MCP tools natively
//! on the same port. Every route requires HTTP Basic authentication.
//!
//! Agents on this computer reach `/mcp` with a short-lived bearer token
//! instead ([`TokenStore`]): read only, and only the notes in the token's
//! scope ([`AgentScope`]). A scope the person allowed may also suggest edits,
//! which wait in a [`ReviewQueue`] and are written only by the person's Apply.
//! When the local API is off, [`spawn_agents`] serves
//! just that, on a loopback port of its own.
//!
//! The server stays thin: validation, conflicts and search live in
//! [`kanso_store::Store`], which is synchronous and is called on tokio's
//! blocking pool.

pub mod agent_tools;
mod api;
mod auth;
mod changeset;
mod error;
mod mcp;
mod proposal;
mod rest;
mod review;
mod scope;
mod tokens;

use std::fmt;
use std::future::Future;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::middleware;
use axum::routing::post;
use kanso_store::Store;

pub use auth::basic_header;
pub use changeset::{
    AppliedSet, ApplyFailure, ChangeSet, ChangeSets, Counts, MergeTarget, OpKind, OpState,
    ProposedOp, Selection, TaskId, undo as undo_changes,
};
pub use proposal::{Applied, ApplyError, ChangeId, NewChange, ProposedChange};
pub use review::{EXPIRY as REVIEW_EXPIRY, ReviewEvent, ReviewQueue};
pub use scope::{
    ASK_TOOLS, AgentScope, Caller, READ_TOOLS, SUGGEST_TOOLS, TASK_TOOLS, TaskLimits, TaskOps,
    has_readable_notes, readable_notes,
};
pub use tokens::{Minted, ShownNote, TokenId, TokenStore};

/// Largest request body accepted. A 10 MB image is about 13.4 MB in base64.
pub const MAX_BODY_BYTES: usize = 16 * 1024 * 1024;

/// Largest body of a REST attachment upload (`POST /files`, `/file:<id>`,
/// `/file/<id>`): a 50 MiB PDF is 69,905,068 bytes in base64, and this
/// leaves about 5 MiB for the rest of the JSON. Every other route, `/mcp`
/// included, keeps [`MAX_BODY_BYTES`].
pub const MAX_ATTACHMENT_BODY_BYTES: usize = 72 * 1024 * 1024;

/// Whether a request is a REST attachment upload, which may carry
/// [`MAX_ATTACHMENT_BODY_BYTES`]. The path is the raw one, so a
/// percent-encoded `:` counts too; the store still checks the size for the
/// file's kind (10 MiB, or 50 MiB for a PDF).
fn is_attachment_upload(method: &axum::http::Method, path: &str) -> bool {
    let encoded_colon = path
        .get(..8)
        .is_some_and(|head| head.eq_ignore_ascii_case("/file%3a"));
    method == axum::http::Method::POST
        && (path == "/files"
            || path.starts_with("/file:")
            || path.starts_with("/file/")
            || encoded_colon)
}

/// Caps every body at [`MAX_BODY_BYTES`] but an attachment upload's, which
/// the router's [`DefaultBodyLimit`] caps at [`MAX_ATTACHMENT_BODY_BYTES`].
/// Over the cap the extractors refuse with 413 (`too_large`) as before.
async fn body_limit(
    request: axum::extract::Request,
    next: middleware::Next,
) -> axum::response::Response {
    if is_attachment_upload(request.method(), request.uri().path()) {
        return next.run(request).await;
    }
    let (parts, body) = request.into_parts();
    let body = axum::body::Body::new(http_body_util::Limited::new(body, MAX_BODY_BYTES));
    next.run(axum::extract::Request::from_parts(parts, body))
        .await
}

/// Where the server listens and the credentials it requires.
#[derive(Clone, PartialEq, Eq)]
pub struct ServerConfig {
    pub bind: SocketAddr,
    pub username: String,
    pub password: String,
}

impl fmt::Debug for ServerConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ServerConfig")
            .field("bind", &self.bind)
            .field("username", &self.username)
            .field("password", &"<redacted>")
            .finish()
    }
}

#[derive(Clone)]
pub(crate) struct AppState {
    pub store: Arc<Store>,
    /// Whether MCP tools may read and write local files (`create-file` with
    /// `filePath`, `read-file` with `outputPath`). Only when the server is
    /// bound to a loopback address, so every client is on this machine.
    pub local_files: bool,
    /// The agents' tokens, to count and record what each one reads.
    pub tokens: TokenStore,
}

/// The full API: REST endpoints, `/mcp`, Basic auth and the body limit.
pub fn router(store: Arc<Store>, config: &ServerConfig) -> Router {
    router_with_tokens(store, config, TokenStore::new())
}

/// [`router`], where `/mcp` also takes the agents' tokens in `tokens`.
pub fn router_with_tokens(store: Arc<Store>, config: &ServerConfig, tokens: TokenStore) -> Router {
    let state = AppState {
        store,
        local_files: config.bind.ip().is_loopback(),
        tokens: tokens.clone(),
    };
    let gate = auth::Gate {
        credentials: Some(Arc::new(auth::Credentials::new(
            &config.username,
            &config.password,
        ))),
        tokens,
    };
    rest::routes()
        .route(
            auth::MCP_PATH,
            post(mcp::handle).get(mcp::no_stream).delete(mcp::no_stream),
        )
        .fallback(rest::not_found)
        .method_not_allowed_fallback(rest::method_not_allowed)
        .with_state(state)
        .layer(DefaultBodyLimit::max(MAX_ATTACHMENT_BODY_BYTES))
        .layer(middleware::from_fn(body_limit))
        .layer(middleware::from_fn_with_state(gate, auth::require_auth))
}

/// The agents' private listener: `/mcp` alone, opened by a token in
/// `tokens` alone. No REST routes, no Basic sign-in, no local files.
pub fn agents_router(store: Arc<Store>, tokens: TokenStore) -> Router {
    let state = AppState {
        store,
        local_files: false,
        tokens: tokens.clone(),
    };
    let gate = auth::Gate {
        credentials: None,
        tokens,
    };
    Router::new()
        .route(
            auth::MCP_PATH,
            post(mcp::handle).get(mcp::no_stream).delete(mcp::no_stream),
        )
        .fallback(rest::not_found)
        .method_not_allowed_fallback(rest::method_not_allowed)
        .with_state(state)
        .layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
        .layer(middleware::from_fn_with_state(gate, auth::require_auth))
}

/// How long in-flight requests may run on after shutdown is requested.
const SHUTDOWN_GRACE: Duration = Duration::from_secs(5);

/// Binds `config.bind` and serves until `shutdown` resolves, then lets
/// in-flight requests finish for up to five seconds.
///
/// When the port is taken the error is an [`io::Error`] of kind
/// [`io::ErrorKind::AddrInUse`]; see [`is_addr_in_use`].
pub async fn serve(
    store: Arc<Store>,
    config: ServerConfig,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind(config.bind).await?;
    serve_on(listener, router(store, &config), shutdown).await
}

async fn serve_on(
    listener: tokio::net::TcpListener,
    app: Router,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> anyhow::Result<()> {
    let (stopping_tx, stopping) = tokio::sync::watch::channel(false);
    tokio::spawn(async move {
        shutdown.await;
        let _ = stopping_tx.send(true);
    });
    let stopped = |mut rx: tokio::sync::watch::Receiver<bool>| async move {
        let _ = rx.wait_for(|stopping| *stopping).await;
    };
    let graceful = axum::serve(listener, app).with_graceful_shutdown(stopped(stopping.clone()));
    let deadline = async {
        stopped(stopping).await;
        tokio::time::sleep(SHUTDOWN_GRACE).await;
    };
    tokio::select! {
        result = graceful => result?,
        // A client that never finishes its request can't hold the server open.
        () = deadline => {}
    }
    Ok(())
}

/// Whether a [`serve`] error means the port is already in use.
pub fn is_addr_in_use(err: &anyhow::Error) -> bool {
    err.chain().any(|cause| {
        cause
            .downcast_ref::<io::Error>()
            .is_some_and(|e| e.kind() == io::ErrorKind::AddrInUse)
    })
}

/// A server running on its own thread and tokio runtime, for callers that
/// are not async (the GPUI app). Dropping the handle asks the server to stop.
pub struct ServerHandle {
    bound: Result<SocketAddr, io::Error>,
    tokens: TokenStore,
    stop: Option<tokio::sync::oneshot::Sender<()>>,
    thread: Option<JoinHandle<anyhow::Result<()>>>,
}

/// Binds `config.bind` right away, then serves on a dedicated thread with
/// its own tokio runtime. The bind result is available immediately from the
/// handle, so the caller can show "API off: port in use".
pub fn spawn(store: Arc<Store>, config: ServerConfig) -> ServerHandle {
    spawn_with_tokens(store, config, TokenStore::new())
}

/// [`spawn`], where `/mcp` also takes the agents' tokens in `tokens`.
pub fn spawn_with_tokens(
    store: Arc<Store>,
    config: ServerConfig,
    tokens: TokenStore,
) -> ServerHandle {
    let app = router_with_tokens(store, &config, tokens.clone());
    spawn_router(config.bind, app, tokens)
}

/// The agents' private listener ([`agents_router`]) on a free loopback port,
/// for when the local API is off.
pub fn spawn_agents(store: Arc<Store>, tokens: TokenStore) -> ServerHandle {
    let app = agents_router(store, tokens.clone());
    spawn_router(
        SocketAddr::from((std::net::Ipv4Addr::LOCALHOST, 0)),
        app,
        tokens,
    )
}

fn spawn_router(bind: SocketAddr, app: Router, tokens: TokenStore) -> ServerHandle {
    let failed = |err| ServerHandle {
        bound: Err(err),
        tokens: tokens.clone(),
        stop: None,
        thread: None,
    };
    let listener = match std::net::TcpListener::bind(bind) {
        Ok(listener) => listener,
        Err(err) => return failed(err),
    };
    let addr = match listener
        .set_nonblocking(true)
        .and_then(|()| listener.local_addr())
    {
        Ok(addr) => addr,
        Err(err) => return failed(err),
    };
    let (stop_tx, stop_rx) = tokio::sync::oneshot::channel::<()>();
    let thread = std::thread::Builder::new()
        .name("kanso-server".into())
        .spawn(move || -> anyhow::Result<()> {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .thread_name("kanso-server-worker")
                .enable_all()
                .build()?;
            let result = runtime.block_on(async move {
                let listener = tokio::net::TcpListener::from_std(listener)?;
                serve_on(listener, app, async move {
                    let _ = stop_rx.await;
                })
                .await
            });
            runtime.shutdown_timeout(Duration::from_secs(5));
            result
        });
    match thread {
        Ok(thread) => ServerHandle {
            bound: Ok(addr),
            tokens,
            stop: Some(stop_tx),
            thread: Some(thread),
        },
        Err(err) => failed(err),
    }
}

impl ServerHandle {
    /// The bound address, or why binding failed.
    pub fn bind_result(&self) -> Result<SocketAddr, &io::Error> {
        self.bound.as_ref().copied()
    }

    /// The bound address, when binding succeeded.
    pub fn local_addr(&self) -> Option<SocketAddr> {
        self.bound.as_ref().ok().copied()
    }

    /// The agents' tokens this server accepts on `/mcp`.
    pub fn tokens(&self) -> &TokenStore {
        &self.tokens
    }

    /// Whether binding failed because the port is taken.
    pub fn is_addr_in_use(&self) -> bool {
        self.bound
            .as_ref()
            .is_err_and(|err| err.kind() == io::ErrorKind::AddrInUse)
    }

    /// Whether the server thread is still serving.
    pub fn is_running(&self) -> bool {
        self.thread
            .as_ref()
            .is_some_and(|thread| !thread.is_finished())
    }

    /// Stops the server, waits for in-flight requests and returns how the
    /// server thread ended.
    pub fn stop(mut self) -> anyhow::Result<()> {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        match self.thread.take() {
            Some(thread) => thread
                .join()
                .map_err(|_| anyhow::anyhow!("the server thread panicked"))?,
            None => Ok(()),
        }
    }
}

impl Drop for ServerHandle {
    fn drop(&mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
    }
}

impl fmt::Debug for ServerHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ServerHandle")
            .field("bound", &self.bound)
            .field("running", &self.is_running())
            .finish()
    }
}

#[cfg(test)]
mod body_limit_tests {
    use super::is_attachment_upload;
    use axum::http::Method;

    // no-prop-test: a fixed table of the routes that create files.
    #[test]
    fn only_file_uploads_take_the_attachment_limit() {
        for path in [
            "/files",
            "/file:abc",
            "/file/abc",
            "/file%3Aabc",
            "/FILE%3aabc",
        ] {
            assert!(is_attachment_upload(&Method::POST, path), "{path}");
        }
        for path in ["/notes", "/note:abc", "/mcp", "/filesx", "/fil", "/books"] {
            assert!(!is_attachment_upload(&Method::POST, path), "{path}");
        }
        assert!(!is_attachment_upload(&Method::GET, "/files"));
        assert!(!is_attachment_upload(&Method::DELETE, "/file:abc"));
    }
}
