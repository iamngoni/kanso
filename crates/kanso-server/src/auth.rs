//! Sign-in by API key (issue #243). Every route requires a key, sent as
//! `Authorization: Bearer kcl_…`, or as the password of HTTP Basic for tools
//! that only take a login. `/mcp` also takes an agent's session token, and
//! nothing else does.

use axum::extract::{Request, State};
use axum::http::header::AUTHORIZATION;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use base64::Engine as _;
use base64::engine::DecodePaddingMode;
use base64::engine::general_purpose::{GeneralPurpose, GeneralPurposeConfig};

use crate::error::ApiError;
use crate::keys::{KeyStore, key_from_header};
use crate::lease::{CURRENT, Lease};
use crate::scope::Caller;
use crate::tokens::TokenStore;

/// The one route an agent's token opens.
pub(crate) const MCP_PATH: &str = "/mcp";

/// Basic tokens with or without `=` padding.
const BASIC: GeneralPurpose = GeneralPurpose::new(
    &base64::alphabet::STANDARD,
    GeneralPurposeConfig::new().with_decode_padding_mode(DecodePaddingMode::Indifferent),
);

pub(crate) fn decode_basic(header: &[u8]) -> Option<Vec<u8>> {
    let header = std::str::from_utf8(header).ok()?.trim();
    let (scheme, token) = header.split_once(' ')?;
    if !scheme.eq_ignore_ascii_case("basic") {
        return None;
    }
    BASIC.decode(token.trim()).ok()
}

/// The token of a `Bearer` header value.
pub(crate) fn bearer(header: &[u8]) -> Option<&str> {
    let header = std::str::from_utf8(header).ok()?.trim();
    let (scheme, token) = header.split_once(' ')?;
    scheme
        .eq_ignore_ascii_case("bearer")
        .then(|| token.trim())
        .filter(|token| !token.is_empty())
}

/// Who an `Authorization` header on `path` signs in as: an API key
/// anywhere (`Bearer kcl_…`, or `Basic` with the key as the password), an
/// agent's live session token on `/mcp` only. `None` is refused. A key is
/// accepted only when `keys` is given (the agents' private listener has
/// none).
#[cfg(test)]
pub(crate) fn caller(
    keys: Option<&KeyStore>,
    tokens: &TokenStore,
    path: &str,
    header: Option<&[u8]>,
) -> Option<Caller> {
    caller_leased(keys, tokens, path, header).map(|(caller, _)| caller)
}

/// [`caller`], and the [`Lease`] a key's request holds until it is answered.
pub(crate) fn caller_leased(
    keys: Option<&KeyStore>,
    tokens: &TokenStore,
    path: &str,
    header: Option<&[u8]>,
) -> Option<(Caller, Option<Lease>)> {
    let header = header?;
    if path == MCP_PATH
        && let Some(token) = bearer(header)
        && let Some(caller) = tokens.verify(token)
    {
        return Some((caller, None));
    }
    let key = key_from_header(header)?;
    let (caller, lease) = keys?.verify_leased(&key)?;
    Some((caller, Some(lease)))
}

/// What the sign-in middleware checks against: the library's API keys, when
/// they are accepted (the private agents' listener takes none), and the
/// session tokens.
#[derive(Clone)]
pub(crate) struct Gate {
    pub keys: Option<KeyStore>,
    pub tokens: TokenStore,
}

pub(crate) async fn require_auth(
    State(gate): State<Gate>,
    mut request: Request,
    next: Next,
) -> Response {
    let header = request
        .headers()
        .get(AUTHORIZATION)
        .map(|value| value.as_bytes());
    match caller_leased(
        gate.keys.as_ref(),
        &gate.tokens,
        request.uri().path(),
        header,
    ) {
        Some((caller, None)) => {
            request.extensions_mut().insert(caller);
            next.run(request).await
        }
        Some((caller, Some(lease))) => {
            request.extensions_mut().insert(caller);
            // The request runs holding its key's lease, and is dropped the
            // moment the key is revoked. A store call it already started
            // keeps its own lease (`rest::blocking`), which the revoke
            // waits for; none begins after the revoke.
            let watch = lease.also();
            let work = CURRENT.scope(lease, next.run(request));
            tokio::select! {
                response = work => {
                    if watch.is_revoked() {
                        revoked_response(&gate)
                    } else {
                        response
                    }
                }
                () = watch.revoked() => revoked_response(&gate),
            }
        }
        None => {
            if let Some(keys) = &gate.keys {
                keys.note_refused();
            }
            ApiError::unauthorized().into_response()
        }
    }
}

/// The answer to a request whose key was revoked while it ran.
fn revoked_response(gate: &Gate) -> Response {
    if let Some(keys) = &gate.keys {
        keys.note_refused();
    }
    ApiError::unauthorized().into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::key_model::{KeyId, NewKey};
    use kanso_store::{KeyAccess, Store};
    use proptest::prelude::*;

    fn fixture() -> (tempfile::TempDir, KeyStore, TokenStore) {
        let dir = tempfile::tempdir().unwrap();
        let store = std::sync::Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
        (dir, KeyStore::open(store).unwrap(), TokenStore::new())
    }

    fn mint(keys: &KeyStore) -> String {
        keys.mint(NewKey {
            name: "Test".into(),
            client: "other".into(),
            access: KeyAccess::Read,
            books: None,
        })
        .unwrap()
        .key
    }

    // no-prop-test: a fixed table of paths and header kinds.
    #[test]
    fn a_key_opens_every_path_and_a_session_token_opens_mcp_alone() {
        let (_dir, keys, tokens) = fixture();
        let key = mint(&keys);
        let bearer_key = format!("Bearer {key}");
        for path in ["/mcp", "/notes", "/", "/_changes", "/note:abc"] {
            assert!(
                matches!(
                    caller(Some(&keys), &tokens, path, Some(bearer_key.as_bytes())),
                    Some(Caller::Key { .. })
                ),
                "{path}"
            );
        }
        let minted = tokens.mint(crate::scope::AgentScope::ask(None));
        let session = format!("Bearer {}", minted.token);
        assert!(matches!(
            caller(Some(&keys), &tokens, "/mcp", Some(session.as_bytes())),
            Some(Caller::Agent { .. })
        ));
        for path in ["/notes", "/", "/mcp/", "/_changes"] {
            assert!(
                caller(Some(&keys), &tokens, path, Some(session.as_bytes())).is_none(),
                "{path}"
            );
        }
        // The private listener takes no key.
        assert!(caller(None, &tokens, "/mcp", Some(bearer_key.as_bytes())).is_none());
        assert!(caller(Some(&keys), &tokens, "/mcp", None).is_none());
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(128))]

        /// Whatever the header says, a caller is a key only when it carries
        /// a key that was minted and not revoked.
        #[test]
        fn prop_only_a_live_key_signs_in(header in "[ -~]{0,60}", ours in any::<bool>(), revoke in any::<bool>()) {
            let (_dir, keys, tokens) = fixture();
            let key = mint(&keys);
            let real = format!("Bearer {key}");
            let sent = if ours { real.clone() } else { header };
            if revoke {
                let id = keys.list().unwrap()[0].id.clone();
                keys.revoke(&KeyId(id)).unwrap();
            }
            let got = caller(Some(&keys), &tokens, "/notes", Some(sent.as_bytes()));
            prop_assert_eq!(got.is_some(), sent.trim() == real && !revoke);
        }
    }

    // A key revoked while its request runs: that request never finishes
    // its work after the revoke returns, and answers 401.
    mod in_flight {
        use super::*;
        use crate::AppState;
        use axum::body::Body;
        use axum::extract::State;
        use axum::http::{Request, StatusCode};
        use axum::routing::get;
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::time::Duration;
        use tower::ServiceExt as _;

        struct Rig {
            _dir: tempfile::TempDir,
            keys: KeyStore,
            app: axum::Router,
            /// Store calls that ran to the end.
            finished: Arc<AtomicUsize>,
            /// Store calls that began.
            begun: Arc<AtomicUsize>,
        }

        fn rig(store_ms: u64, wait_ms: u64) -> Rig {
            let dir = tempfile::tempdir().unwrap();
            let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
            let keys = KeyStore::open(store.clone()).unwrap();
            let tokens = TokenStore::new();
            let finished = Arc::new(AtomicUsize::new(0));
            let begun = Arc::new(AtomicUsize::new(0));
            let (f, b) = (finished.clone(), begun.clone());
            let handler = move |state: State<AppState>| {
                let (f, b) = (f.clone(), b.clone());
                async move {
                    // Waiting before the store call, as a slow body does.
                    tokio::time::sleep(Duration::from_millis(wait_ms)).await;
                    crate::rest::blocking(&state, move |_| {
                        b.fetch_add(1, Ordering::SeqCst);
                        std::thread::sleep(Duration::from_millis(store_ms));
                        f.fetch_add(1, Ordering::SeqCst);
                        Ok(())
                    })
                    .await
                    .map(|()| "done")
                }
            };
            let gate = Gate {
                keys: Some(keys.clone()),
                tokens: tokens.clone(),
            };
            let app = axum::Router::new()
                .route("/slow", get(handler))
                .with_state(AppState {
                    store,
                    tokens,
                    canvases: None,
                })
                .layer(axum::middleware::from_fn_with_state(gate, require_auth));
            Rig {
                _dir: dir,
                keys,
                app,
                finished,
                begun,
            }
        }

        fn get_slow(rig: &Rig, key: &str) -> tokio::task::JoinHandle<StatusCode> {
            let app = rig.app.clone();
            let request = Request::builder()
                .uri("/slow")
                .header("authorization", format!("Bearer {key}"))
                .body(Body::empty())
                .unwrap();
            tokio::spawn(async move { app.oneshot(request).await.unwrap().status() })
        }

        async fn revoke(keys: &KeyStore, id: &str) {
            let (keys, id) = (keys.clone(), KeyId(id.to_owned()));
            tokio::task::spawn_blocking(move || keys.revoke(&id).unwrap())
                .await
                .unwrap();
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
        async fn a_store_call_under_way_ends_before_the_revoke_returns() {
            let rig = rig(300, 0);
            let key = mint(&rig.keys);
            let id = rig.keys.list().unwrap()[0].id.clone();
            let request = get_slow(&rig, &key);
            while rig.begun.load(Ordering::SeqCst) == 0 {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
            revoke(&rig.keys, &id).await;
            // Nothing is left to finish once the revoke has returned.
            assert_eq!(rig.finished.load(Ordering::SeqCst), 1);
            assert_eq!(request.await.unwrap(), StatusCode::UNAUTHORIZED);
            assert_eq!(rig.finished.load(Ordering::SeqCst), 1);
        }

        #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
        async fn a_request_still_waiting_never_reaches_the_store() {
            let rig = rig(0, 400);
            let key = mint(&rig.keys);
            let other = mint(&rig.keys);
            let id = rig
                .keys
                .list()
                .unwrap()
                .into_iter()
                .find(|row| {
                    rig.keys.verify(&key).is_some_and(
                        |c| matches!(c, Caller::Key { id: ref k, .. } if k.as_str() == row.id),
                    )
                })
                .unwrap()
                .id;
            let waiting = get_slow(&rig, &key);
            let kept = get_slow(&rig, &other);
            tokio::time::sleep(Duration::from_millis(100)).await;
            revoke(&rig.keys, &id).await;
            assert_eq!(waiting.await.unwrap(), StatusCode::UNAUTHORIZED);
            // The other key's request is untouched.
            assert_eq!(kept.await.unwrap(), StatusCode::OK);
            assert_eq!(rig.begun.load(Ordering::SeqCst), 1);
            assert_eq!(rig.finished.load(Ordering::SeqCst), 1);
            // And the revoked key is refused from now on.
            assert_eq!(
                get_slow(&rig, &key).await.unwrap(),
                StatusCode::UNAUTHORIZED
            );
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256,
            rng_seed: proptest::test_runner::RngSeed::Fixed(0x006b_6579_7376),
            ..ProptestConfig::default()
        })]

        /// Whatever order requests come, finish and the key is revoked in,
        /// no lease is granted once the gate is closed, and a drain ends
        /// exactly when every lease is let go.
        #[test]
        fn prop_a_closed_gate_grants_nothing(steps in prop::collection::vec(0u8..4, 0..40)) {
            use crate::lease::KeyGate;
            use std::time::Duration;
            let gate = KeyGate::new();
            let mut held = Vec::new();
            let mut closed = false;
            for step in steps {
                match step {
                    0 => match gate.lease() {
                        Some(lease) => { prop_assert!(!closed); held.push(lease); }
                        None => prop_assert!(closed),
                    },
                    1 => { held.pop(); }
                    2 => {
                        if let Some(first) = held.first() {
                            let more = first.also();
                            held.push(more);
                        }
                    }
                    _ => { gate.close(); closed = true; }
                }
                prop_assert_eq!(gate.held(), held.len());
                prop_assert_eq!(gate.drain(Duration::ZERO), held.is_empty());
                if closed {
                    prop_assert!(held.iter().all(|lease| lease.is_revoked()));
                }
            }
        }
    }
}
