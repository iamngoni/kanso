//! Errors in CouchDB's JSON shape, `{"error": …, "reason": …}`, as the local-server API's
//! local server returns them.

use axum::Json;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use kanso_store::StoreError;
use serde_json::json;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ApiError {
    pub status: StatusCode,
    pub error: &'static str,
    pub reason: String,
}

pub(crate) type ApiResult<T> = Result<T, ApiError>;

impl ApiError {
    fn new(status: StatusCode, error: &'static str, reason: impl Into<String>) -> Self {
        ApiError {
            status,
            error,
            reason: reason.into(),
        }
    }

    pub fn not_found() -> Self {
        Self::new(StatusCode::NOT_FOUND, "not_found", "missing")
    }

    pub fn conflict() -> Self {
        Self::new(
            StatusCode::CONFLICT,
            "conflict",
            "Document update conflict.",
        )
    }

    pub fn bad_request(reason: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, "bad_request", reason)
    }

    pub fn unauthorized() -> Self {
        Self::new(
            StatusCode::UNAUTHORIZED,
            "unauthorized",
            "Kanso needs an API key. Create one in Settings \u{203a} Local API \u{203a} API keys.",
        )
    }

    /// A key that may not do this. The reason says which key would.
    pub fn forbidden(reason: impl Into<String>) -> Self {
        Self::new(StatusCode::FORBIDDEN, "forbidden", reason)
    }

    pub fn too_large() -> Self {
        Self::new(
            StatusCode::PAYLOAD_TOO_LARGE,
            "too_large",
            "The request body is larger than 16 MiB.",
        )
    }

    pub fn method_not_allowed() -> Self {
        Self::new(
            StatusCode::METHOD_NOT_ALLOWED,
            "method_not_allowed",
            "This method is not allowed on this resource.",
        )
    }

    pub fn internal(reason: impl Into<String>) -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_server_error",
            reason,
        )
    }

    /// The text the Node MCP server reports when a request fails, e.g.
    /// `Fetch error [404] Not Found`.
    pub fn fetch_error_text(&self) -> String {
        format!(
            "Fetch error [{}] {}",
            self.status.as_u16(),
            self.status.canonical_reason().unwrap_or_default()
        )
    }
}

impl From<StoreError> for ApiError {
    fn from(err: StoreError) -> Self {
        match err {
            StoreError::NotFound => ApiError::not_found(),
            StoreError::Conflict => ApiError::conflict(),
            StoreError::Invalid(reason) => ApiError::bad_request(reason),
            other => ApiError::internal(other.to_string()),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = Json(json!({ "error": self.error, "reason": self.reason }));
        let mut response = (self.status, body).into_response();
        if self.status == StatusCode::UNAUTHORIZED {
            response.headers_mut().insert(
                header::WWW_AUTHENTICATE,
                HeaderValue::from_static("Basic realm=\"Kanso\""),
            );
        }
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    /// Every kind of store error, with an arbitrary message where it has one.
    fn store_error(kind: usize, message: String) -> StoreError {
        match kind {
            0 => StoreError::NotFound,
            1 => StoreError::Conflict,
            2 => StoreError::Invalid(message),
            3 => StoreError::Io(std::io::Error::other(message)),
            4 => StoreError::Sqlite(rusqlite::Error::InvalidQuery),
            _ => StoreError::Json(serde_json::from_str::<serde_json::Value>("{").unwrap_err()),
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 64,
            failure_persistence: Some(Box::new(
                proptest::test_runner::FileFailurePersistence::WithSource("regressions"),
            )),
            ..ProptestConfig::default()
        })]

        /// Status-code invariant: every store error maps to exactly one status
        /// and error name, and invalid-input messages pass through unchanged.
        #[test]
        fn status_code_invariant_for_store_errors(kind in 0..6usize, message in ".{0,40}") {
            let err = store_error(kind, message);
            let expected = match &err {
                StoreError::NotFound => (StatusCode::NOT_FOUND, "not_found", Some("missing".to_owned())),
                StoreError::Conflict => (StatusCode::CONFLICT, "conflict", Some("Document update conflict.".to_owned())),
                StoreError::Invalid(m) => (StatusCode::BAD_REQUEST, "bad_request", Some(m.clone())),
                _ => (StatusCode::INTERNAL_SERVER_ERROR, "internal_server_error", None),
            };
            let api = ApiError::from(err);
            prop_assert_eq!(api.status, expected.0);
            prop_assert_eq!(api.error, expected.1);
            if let Some(reason) = expected.2 {
                prop_assert_eq!(api.reason, reason);
            }
        }
    }
}
