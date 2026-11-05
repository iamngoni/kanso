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
pub(crate) fn caller(
    keys: Option<&KeyStore>,
    tokens: &TokenStore,
    path: &str,
    header: Option<&[u8]>,
) -> Option<Caller> {
    let header = header?;
    if path == MCP_PATH
        && let Some(token) = bearer(header)
        && let Some(caller) = tokens.verify(token)
    {
        return Some(caller);
    }
    let key = key_from_header(header)?;
    keys?.verify(&key)
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
    match caller(
        gate.keys.as_ref(),
        &gate.tokens,
        request.uri().path(),
        header,
    ) {
        Some(caller) => {
            request.extensions_mut().insert(caller);
            next.run(request).await
        }
        None => ApiError::unauthorized().into_response(),
    }
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
}
