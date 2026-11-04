//! HTTP Basic authentication. Every route requires it, as the local-server API does.
//! `/mcp` also takes an agent's `Authorization: Bearer` token, and nothing
//! else does.

use std::sync::Arc;

use axum::extract::{Request, State};
use axum::http::header::AUTHORIZATION;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use base64::Engine as _;
use base64::engine::general_purpose::{GeneralPurpose, GeneralPurposeConfig};
use base64::engine::{DecodePaddingMode, general_purpose};
use subtle::ConstantTimeEq;

use crate::error::ApiError;
use crate::scope::Caller;
use crate::tokens::TokenStore;

/// The one route an agent's token opens.
pub(crate) const MCP_PATH: &str = "/mcp";

/// Basic tokens with or without `=` padding.
const BASIC: GeneralPurpose = GeneralPurpose::new(
    &base64::alphabet::STANDARD,
    GeneralPurposeConfig::new().with_decode_padding_mode(DecodePaddingMode::Indifferent),
);

pub(crate) struct Credentials {
    /// `username:password`, the decoded form of a valid Basic token.
    expected: Vec<u8>,
}

impl Credentials {
    pub fn new(username: &str, password: &str) -> Self {
        Credentials {
            expected: format!("{username}:{password}").into_bytes(),
        }
    }

    /// Whether an `Authorization` header value carries exactly these
    /// credentials. The comparison takes the same time for every token of a
    /// given length.
    pub fn accepts(&self, header: Option<&[u8]>) -> bool {
        let Some(given) = header.and_then(decode_basic) else {
            return false;
        };
        bool::from(given.ct_eq(&self.expected))
    }
}

pub(crate) fn decode_basic(header: &[u8]) -> Option<Vec<u8>> {
    let header = std::str::from_utf8(header).ok()?.trim();
    let (scheme, token) = header.split_once(' ')?;
    if !scheme.eq_ignore_ascii_case("basic") {
        return None;
    }
    BASIC.decode(token.trim()).ok()
}

/// The `Authorization` header value for `username` and `password`.
pub fn basic_header(username: &str, password: &str) -> String {
    format!(
        "Basic {}",
        general_purpose::STANDARD.encode(format!("{username}:{password}"))
    )
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

/// Who an `Authorization` header on `path` signs in as: the API's
/// credentials anywhere, an agent's live token on `/mcp` only. `None` is
/// refused.
pub(crate) fn caller(
    credentials: Option<&Credentials>,
    tokens: &TokenStore,
    path: &str,
    header: Option<&[u8]>,
) -> Option<Caller> {
    if let Some(token) = header.and_then(bearer) {
        return (path == MCP_PATH).then(|| tokens.verify(token)).flatten();
    }
    credentials
        .is_some_and(|credentials| credentials.accepts(header))
        .then_some(Caller::Full)
}

/// What the sign-in middleware checks against: the API's credentials, when
/// it has them (the private agents' listener has none), and the tokens.
#[derive(Clone)]
pub(crate) struct Gate {
    pub credentials: Option<Arc<Credentials>>,
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
        gate.credentials.as_deref(),
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

    #[test]
    fn accepts_only_the_exact_credentials() {
        let creds = Credentials::new("user", "p:ss");
        let ok = basic_header("user", "p:ss");
        assert!(creds.accepts(Some(ok.as_bytes())));
        let unpadded = ok.trim_end_matches('=').to_owned();
        assert!(creds.accepts(Some(unpadded.as_bytes())));
        assert!(creds.accepts(Some(ok.replacen("Basic", "basic", 1).as_bytes())));
        for bad in [
            basic_header("user", "p:s"),
            basic_header("User", "p:ss"),
            basic_header("user", "p:ss "),
            "Bearer abc".to_owned(),
            "Basic !!!".to_owned(),
            String::new(),
        ] {
            assert!(!creds.accepts(Some(bad.as_bytes())), "{bad}");
        }
        assert!(!creds.accepts(None));
    }

    #[test]
    fn a_token_opens_mcp_and_nothing_else() {
        let creds = Credentials::new("user", "pass");
        let tokens = TokenStore::new();
        let minted = tokens.mint(crate::scope::AgentScope::ask(None));
        let bearer = format!("Bearer {}", minted.token);
        let header = Some(bearer.as_bytes());
        assert!(matches!(
            caller(Some(&creds), &tokens, "/mcp", header),
            Some(Caller::Agent { .. })
        ));
        for path in ["/notes", "/", "/mcp/", "/_changes", "/note:abc"] {
            assert!(
                caller(Some(&creds), &tokens, path, header).is_none(),
                "{path}"
            );
        }
        // A wrong token never falls back to the API's sign-in.
        let wrong = Some("Bearer nope".as_bytes());
        assert!(caller(Some(&creds), &tokens, "/mcp", wrong).is_none());
        let basic = basic_header("user", "pass");
        assert!(matches!(
            caller(Some(&creds), &tokens, "/notes", Some(basic.as_bytes())),
            Some(Caller::Full)
        ));
        // Without credentials (the private listener), Basic opens nothing.
        assert!(caller(None, &tokens, "/mcp", Some(basic.as_bytes())).is_none());
        tokens.revoke(minted.id);
        assert!(caller(Some(&creds), &tokens, "/mcp", header).is_none());
    }
}
