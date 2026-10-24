//! The v2 wire protocol: sealed documents, push results, change pages, the
//! session-bound [`WireRemote`], the patterns and limits both sides enforce,
//! and [`ApiError`] with its codes (docs/ACCOUNTS.md › HTTP protocol (v2)).
//!
//! Frozen by step 1 of the build order. The Worker (`cloud/`) and
//! [`crate::MemoryServer`] follow it; `cloud/test/fixtures/conformance.json`
//! and `auth.json` hold the cases both run.

use std::fmt;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Value, json};

// ---------------------------------------------------------------------------
// Limits (docs/ACCOUNTS.md › Conventions, › Sealed documents, › Routes)
// ---------------------------------------------------------------------------

/// Most documents in one push.
pub const MAX_PUSH_DOCS: usize = 200;
/// Largest push request body (bytes).
pub const MAX_PUSH_BODY: usize = 4 * 1024 * 1024;
/// Shortest sealed document: base64url of `0x01 || nonce(24) || tag(16)`.
pub const MIN_SEALED_LEN: usize = 55;
/// Longest sealed document (characters), so a D1 row stays under 2 MB.
pub const MAX_SEALED_LEN: usize = 1_900_000;
/// Largest `fields` JSON the client pushes (bytes); larger documents land in
/// the report's `too_large`.
pub const MAX_FIELDS_JSON: usize = 1_400_000;
/// Shortest sealed file: `0x01 || nonce(24) || tag(16)` around zero bytes.
pub const MIN_SEALED_FILE: usize = 41;
/// Longest sealed file: 10 MiB of plaintext plus 41 bytes.
pub const MAX_SEALED_FILE: usize = 10_485_801;
/// `changes` page size when none is given, and the largest allowed.
pub const DEFAULT_CHANGES_LIMIT: usize = 100;
pub const MAX_CHANGES_LIMIT: usize = 500;
/// Sealed text a `changes` page (and a push response's conflicts) carries at most.
pub const PAGE_SEALED_BUDGET: usize = 8 * 1024 * 1024;
/// Largest auth or account request body (bytes).
pub const MAX_AUTH_BODY: usize = 8 * 1024;
/// Response size caps the client applies (docs/ACCOUNTS.md › Conventions).
pub const MAX_AUTH_RESPONSE: u64 = 64 * 1024;
pub const MAX_SYNC_RESPONSE: u64 = 12 * 1024 * 1024;
pub const MAX_FILE_RESPONSE: u64 = MAX_SEALED_FILE as u64;
/// Active devices per account; signing in on one more signs out the least
/// recently seen.
pub const MAX_ACTIVE_DEVICES: usize = 10;
/// A session unseen this long (ms) is revoked as `expired`.
pub const SESSION_IDLE_MS: i64 = 180 * 24 * 60 * 60 * 1000;
/// The charged cost of one stored document beyond its sealed text, and of one
/// stored file beyond its size (docs/ACCOUNTS.md › Quotas and capacity).
pub const DOC_ROW_COST: u64 = 512;
pub const FILE_ROW_COST: u64 = 1024;
/// The protocol version `/v2/health` reports.
pub const PROTOCOL_VERSION: u32 = 2;

// ---------------------------------------------------------------------------
// Patterns (pure validators, implemented in step 1)
// ---------------------------------------------------------------------------

fn is_base64url_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'-' || b == b'_'
}

fn is_lower_hex(s: &str, len: usize) -> bool {
    s.len() == len && s.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// Only the base64url alphabet (`A–Z a–z 0–9 - _`), any length.
pub fn is_base64url(s: &str) -> bool {
    s.bytes().all(is_base64url_char)
}

/// A wire document ID: `^(note|book|tag|file):[A-Za-z0-9_-]{22}$`.
pub fn is_wire_id(id: &str) -> bool {
    let Some((kind, rest)) = id.split_once(':') else {
        return false;
    };
    matches!(kind, "note" | "book" | "tag" | "file") && rest.len() == 22 && is_base64url(rest)
}

/// A wire file ID: a [`is_wire_id`] of kind `file`.
pub fn is_wire_file_id(id: &str) -> bool {
    id.starts_with("file:") && is_wire_id(id)
}

/// A wire revision: `^[0-9a-f]{32}$`.
pub fn is_wire_rev(rev: &str) -> bool {
    is_lower_hex(rev, 32)
}

/// A sealed document or device name on the wire: base64url, `min..=max` characters.
pub fn is_sealed(sealed: &str, min: usize, max: usize) -> bool {
    (min..=max).contains(&sealed.len()) && is_base64url(sealed)
}

/// `a_` and 32 lowercase hex digits.
pub fn is_account_id(id: &str) -> bool {
    id.strip_prefix("a_")
        .is_some_and(|hex| is_lower_hex(hex, 32))
}

/// `d_` and 32 lowercase hex digits.
pub fn is_device_id(id: &str) -> bool {
    id.strip_prefix("d_")
        .is_some_and(|hex| is_lower_hex(hex, 32))
}

/// A key ID or an epoch: 16 lowercase hex digits.
pub fn is_hex16(s: &str) -> bool {
    is_lower_hex(s, 16)
}

/// A session token: `kanso_` and 43 base64url characters.
pub fn is_token(token: &str) -> bool {
    token
        .strip_prefix("kanso_")
        .is_some_and(|rest| rest.len() == 43 && is_base64url(rest))
}

/// A sealed device name: 55–400 base64url characters.
pub fn is_sealed_name(sealed: &str) -> bool {
    is_sealed(sealed, MIN_SEALED_LEN, 400)
}

// ---------------------------------------------------------------------------
// Documents, push results and pages
// ---------------------------------------------------------------------------

/// A stored document as the server holds it: the `current` of a conflict.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WireDoc {
    pub id: String,
    pub rev: String,
    pub deleted: bool,
    pub sealed: String,
}

/// One document of a `POST /v2/push`. All five fields are required on the
/// wire; `base_rev` is serialized as `null` when absent.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WirePushDoc {
    pub id: String,
    pub rev: String,
    pub deleted: bool,
    pub sealed: String,
    pub base_rev: Option<String>,
}

/// `POST /v2/push` request body.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PushRequest {
    pub docs: Vec<WirePushDoc>,
}

/// `POST /v2/push` response body: one result per document, in order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PushResponse {
    pub results: Vec<WirePushResult>,
}

/// The outcome of pushing one document (docs/ACCOUNTS.md › POST /v2/push).
///
/// Wire shapes: `{"id","status":"ok","seq"}`,
/// `{"id","status":"conflict","current":{…}}`,
/// `{"id","status":"conflict","current":null}` ([`WirePushResult::Lost`]) and
/// `{"id","status":"behind"}`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WirePushResult {
    /// Stored under `seq`.
    Ok { id: String, seq: i64 },
    /// `base_rev` isn't the current revision; `current` is the server's version.
    Conflict { id: String, current: WireDoc },
    /// The client sent a `base_rev`, but the server has no such document.
    Lost { id: String },
    /// A conflict whose current version didn't fit the response's 8 MiB
    /// budget; the client pulls before retrying.
    Behind { id: String },
}

impl WirePushResult {
    pub fn id(&self) -> &str {
        match self {
            WirePushResult::Ok { id, .. }
            | WirePushResult::Conflict { id, .. }
            | WirePushResult::Lost { id }
            | WirePushResult::Behind { id } => id,
        }
    }
}

impl Serialize for WirePushResult {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let value = match self {
            WirePushResult::Ok { id, seq } => json!({"id": id, "status": "ok", "seq": seq}),
            WirePushResult::Conflict { id, current } => {
                json!({"id": id, "status": "conflict", "current": current})
            }
            WirePushResult::Lost { id } => json!({"id": id, "status": "conflict", "current": null}),
            WirePushResult::Behind { id } => json!({"id": id, "status": "behind"}),
        };
        value.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for WirePushResult {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let mut map = Map::<String, Value>::deserialize(deserializer)?;
        let id = match map.remove("id") {
            Some(Value::String(id)) => id,
            _ => return Err(D::Error::missing_field("id")),
        };
        let status = map.remove("status");
        match status.as_ref().and_then(Value::as_str) {
            Some("ok") => {
                let seq = map
                    .remove("seq")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| D::Error::missing_field("seq"))?;
                Ok(WirePushResult::Ok { id, seq })
            }
            Some("conflict") => match map.remove("current") {
                Some(Value::Null) => Ok(WirePushResult::Lost { id }),
                Some(current) => Ok(WirePushResult::Conflict {
                    id,
                    current: serde_json::from_value(current).map_err(D::Error::custom)?,
                }),
                None => Err(D::Error::missing_field("current")),
            },
            Some("behind") => Ok(WirePushResult::Behind { id }),
            _ => Err(D::Error::custom("unknown push status")),
        }
    }
}

/// A document in the change feed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WireChange {
    pub id: String,
    pub rev: String,
    pub deleted: bool,
    pub sealed: String,
    pub seq: i64,
}

/// One page of `GET /v2/changes?since=&limit=` (docs/ACCOUNTS.md › GET
/// /v2/changes). `epoch` and `key_id` are required, so a server can't skip
/// the client's checks by leaving them out.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WirePage {
    pub docs: Vec<WireChange>,
    /// The last document's seq, or `since` for an empty page.
    pub last_seq: i64,
    /// More documents follow (the page was cut, or the first read returned
    /// `limit` rows).
    pub more: bool,
    /// The account's `last_seq`.
    pub head_seq: i64,
    /// 16 hex digits; changed only by the owner after a restore (M1).
    pub epoch: String,
    /// The account's key ID, 16 hex digits.
    pub key_id: String,
}

/// `PUT /v2/files/<wire_id>` response body.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PutFileResponse {
    pub ok: bool,
    /// The stored (sealed) size in bytes.
    pub size: u64,
}

/// The sync routes of one session: what `Sealed` pushes sealed documents
/// through. Obtained from [`crate::Backend::remote`].
pub trait WireRemote {
    /// `POST /v2/push`: one result per document, in order.
    fn push(&self, docs: &[WirePushDoc]) -> Result<Vec<WirePushResult>, ApiError>;
    /// `GET /v2/changes?since=&limit=`.
    fn changes(&self, since: i64, limit: usize) -> Result<WirePage, ApiError>;
    /// `PUT /v2/files/<wire_id>` with the sealed bytes; the implementation
    /// sends their SHA-256 in `x-kanso-sha256`.
    fn put_file(&self, wire_id: &str, sealed: &[u8]) -> Result<(), ApiError>;
    /// `GET /v2/files/<wire_id>`; `None` on 404.
    fn get_file(&self, wire_id: &str) -> Result<Option<Vec<u8>>, ApiError>;
}

// ---------------------------------------------------------------------------
// Codes that carry a reason
// ---------------------------------------------------------------------------

/// Why a session ended: the `reason` of a 401 `unauthorized`, and the
/// `revoked_reason` of a session row (docs/ACCOUNTS.md › Status table).
/// `Unknown` covers a missing, malformed or unknown token, and any reason
/// this client doesn't know.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignOutReason {
    SignedOut,
    Revoked,
    PasswordChanged,
    Recovered,
    Evicted,
    Expired,
    Rekeyed,
    #[serde(other)]
    Unknown,
}

impl SignOutReason {
    pub const ALL: [SignOutReason; 8] = [
        SignOutReason::Unknown,
        SignOutReason::SignedOut,
        SignOutReason::Revoked,
        SignOutReason::PasswordChanged,
        SignOutReason::Recovered,
        SignOutReason::Evicted,
        SignOutReason::Expired,
        SignOutReason::Rekeyed,
    ];

    /// The wire spelling.
    pub fn as_str(self) -> &'static str {
        match self {
            SignOutReason::Unknown => "unknown",
            SignOutReason::SignedOut => "signed_out",
            SignOutReason::Revoked => "revoked",
            SignOutReason::PasswordChanged => "password_changed",
            SignOutReason::Recovered => "recovered",
            SignOutReason::Evicted => "evicted",
            SignOutReason::Expired => "expired",
            SignOutReason::Rekeyed => "rekeyed",
        }
    }

    /// The wire spelling read back; anything unrecognised is `Unknown`.
    pub fn parse(text: &str) -> SignOutReason {
        SignOutReason::ALL
            .into_iter()
            .find(|r| r.as_str() == text)
            .unwrap_or(SignOutReason::Unknown)
    }
}

/// The `reason` of a 403 `signup_closed`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignupClosedReason {
    /// Switched off, or reserved for another username (`only:<name>`).
    Closed,
    /// `max_accounts` reached.
    Full,
    /// `signups_per_day` reached.
    Busy,
}

impl SignupClosedReason {
    pub fn as_str(self) -> &'static str {
        match self {
            SignupClosedReason::Closed => "closed",
            SignupClosedReason::Full => "full",
            SignupClosedReason::Busy => "busy",
        }
    }
}

/// The `quota` of a 413 `quota_exceeded`. When several apply, the first in
/// this order is reported.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuotaKind {
    /// Live documents of every kind.
    Docs,
    /// `length(sealed) + 512` over the account's document rows.
    DocBytes,
    /// Stored file objects.
    Files,
    /// `size + 1024` over the account's files.
    FileBytes,
    /// A server-wide cap (`global_doc_bytes` or `global_file_bytes`).
    Server,
}

impl QuotaKind {
    pub const ALL: [QuotaKind; 5] = [
        QuotaKind::Docs,
        QuotaKind::DocBytes,
        QuotaKind::Files,
        QuotaKind::FileBytes,
        QuotaKind::Server,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            QuotaKind::Docs => "docs",
            QuotaKind::DocBytes => "doc_bytes",
            QuotaKind::Files => "files",
            QuotaKind::FileBytes => "file_bytes",
            QuotaKind::Server => "server",
        }
    }
}

// ---------------------------------------------------------------------------
// ApiError
// ---------------------------------------------------------------------------

/// An error from a v2 route, one variant per error code of the status table
/// (docs/ACCOUNTS.md › Conventions), plus transport and protocol failures.
/// The body is `{"error": "<code>", "message": "…", ...extras}`; the client
/// never shows `message`.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ApiError {
    /// 400: malformed JSON, wrong types, bad ID, revision, length, encoding or
    /// device ID, sealed text under 55 characters, a file body under 41 bytes.
    #[error("400 bad_request")]
    BadRequest,
    #[error("400 invalid_username")]
    InvalidUsername,
    #[error("400 reserved_username")]
    ReservedUsername,
    /// `kdf.alg` isn't `argon2id` or `kdf.version` isn't 19.
    #[error("400 unsupported_kdf")]
    UnsupportedKdf,
    /// Parameters outside the floor or ceilings.
    #[error("400 weak_kdf")]
    WeakKdf,
    #[error("400 username_mismatch")]
    UsernameMismatch,
    #[error("400 checksum_mismatch")]
    ChecksumMismatch,
    /// 401: session missing, malformed, unknown, revoked or expired.
    #[error("401 unauthorized ({})", reason.as_str())]
    Unauthorized { reason: SignOutReason },
    /// 401: wrong proof, or an unknown username.
    #[error("401 invalid_credentials")]
    InvalidCredentials,
    /// 401 `reauth_required`: a sensitive change needs a fresh password or
    /// passkey proof.
    #[error("401 reauth_required")]
    ReauthRequired,
    /// 400 `flow_expired`, or 410 when the provider itself expired the flow
    /// (GitHub's `expired_token`).
    #[error("flow_expired")]
    FlowExpired { gone: bool },
    /// 400 `nonce_reused`: a token or a flow was already used.
    #[error("400 nonce_reused")]
    NonceReused,
    #[error("403 forbidden_origin")]
    ForbiddenOrigin,
    #[error("403 account_disabled")]
    AccountDisabled,
    #[error("403 signup_closed ({})", reason.as_str())]
    SignupClosed { reason: SignupClosedReason },
    /// 403 `provider_disabled`: this server has no such provider configured.
    #[error("403 provider_disabled")]
    ProviderDisabled,
    /// 403 `provider_denied`: the person declined at the provider.
    #[error("403 provider_denied")]
    ProviderDenied,
    /// 404: unknown route, file, or well-formed but unknown device.
    #[error("404 not_found")]
    NotFound,
    /// 405, with an `Allow` header.
    #[error("405 method_not_allowed")]
    MethodNotAllowed,
    #[error("409 username_taken")]
    UsernameTaken,
    #[error("409 account_id_taken")]
    AccountIdTaken,
    /// 409 `identity_taken`: another account owns that provider identity.
    #[error("409 identity_taken")]
    IdentityTaken,
    /// 409 `last_method`: removing this would leave no way to sign in.
    #[error("409 last_method")]
    LastMethod,
    /// 409 `already_has_password`.
    #[error("409 already_has_password")]
    AlreadyHasPassword,
    /// 409 `credential_exists`.
    #[error("409 credential_exists")]
    CredentialExists,
    #[error("413 too_large")]
    TooLarge,
    /// `used` is the usage before this request; for `server` it is the
    /// server-wide sum and `limit` the global cap.
    #[error("413 quota_exceeded ({}: {used} of {limit})", quota.as_str())]
    QuotaExceeded {
        quota: QuotaKind,
        used: u64,
        limit: u64,
    },
    #[error("415 unsupported_media_type")]
    UnsupportedMediaType,
    /// 429: `retry_after` in seconds; `limit` names the rule (`signup-ip`,
    /// `login-user`, `login-ip`, `recover-ip`, `password-account`,
    /// `writes-account`, `uploads-account`, `binding` for the Workers
    /// rate-limit binding, and the server-wide daily caps `writes-global`
    /// and `uploads-global`).
    /// The engine treats any 429 on a push or an upload as a pause.
    #[error("429 rate_limited ({limit}, retry after {retry_after} s)")]
    RateLimited { retry_after: u64, limit: String },
    #[error("500 internal")]
    Internal,
    /// 503: `AUTH_PEPPER` missing or malformed.
    #[error("503 misconfigured")]
    Misconfigured,
    /// 503 `provider_unavailable`: the provider's own endpoint failed or
    /// timed out.
    #[error("503 provider_unavailable")]
    ProviderUnavailable,
    /// A status or code this client doesn't know.
    #[error("{status} {code}")]
    Unrecognized { status: u16, code: String },
    /// The request didn't get an HTTP answer (DNS, TLS, timeout, reset).
    #[error("transport: {0}")]
    Transport(String),
    /// An answer that doesn't follow the protocol (bad JSON, missing
    /// fields, over the response cap, a redirect).
    #[error("protocol: {0}")]
    Protocol(String),
}

/// How an [`ApiError::Transport`] message starts when the request never left
/// this Mac.
const UNSENT: &str = "not sent: ";

impl ApiError {
    /// A transport error from before the request left this Mac: the name
    /// didn't resolve, the connection was refused or never opened. The server
    /// can't have acted on it. It is still [`ApiError::Transport`], so
    /// everything that treats a transport error as "offline" keeps doing so.
    pub fn unsent(message: impl fmt::Display) -> ApiError {
        ApiError::Transport(format!("{UNSENT}{message}"))
    }

    /// Whether this is an [`ApiError::unsent`] error.
    pub fn was_unsent(&self) -> bool {
        matches!(self, ApiError::Transport(message) if message.starts_with(UNSENT))
    }

    /// Whether the server may have carried out the request although the
    /// client got no clean answer: an answer lost on the way (a transport
    /// error after sending), a 5xx, or a 2xx the client couldn't read.
    /// Every 4xx refused it, and an unsent request never arrived.
    pub fn may_have_applied(&self) -> bool {
        match self {
            ApiError::Transport(_) => !self.was_unsent(),
            ApiError::Internal | ApiError::Protocol(_) => true,
            ApiError::Unrecognized { status, .. } => *status >= 500,
            _ => false,
        }
    }

    /// The HTTP status, when the error came from an answer.
    pub fn status(&self) -> Option<u16> {
        Some(match self {
            ApiError::BadRequest
            | ApiError::InvalidUsername
            | ApiError::ReservedUsername
            | ApiError::UnsupportedKdf
            | ApiError::WeakKdf
            | ApiError::UsernameMismatch
            | ApiError::ChecksumMismatch => 400,
            ApiError::Unauthorized { .. }
            | ApiError::InvalidCredentials
            | ApiError::ReauthRequired => 401,
            ApiError::FlowExpired { gone: true } => 410,
            ApiError::FlowExpired { gone: false } | ApiError::NonceReused => 400,
            ApiError::ForbiddenOrigin
            | ApiError::AccountDisabled
            | ApiError::SignupClosed { .. }
            | ApiError::ProviderDisabled
            | ApiError::ProviderDenied => 403,
            ApiError::NotFound => 404,
            ApiError::MethodNotAllowed => 405,
            ApiError::UsernameTaken
            | ApiError::AccountIdTaken
            | ApiError::IdentityTaken
            | ApiError::LastMethod
            | ApiError::AlreadyHasPassword
            | ApiError::CredentialExists => 409,
            ApiError::TooLarge | ApiError::QuotaExceeded { .. } => 413,
            ApiError::UnsupportedMediaType => 415,
            ApiError::RateLimited { .. } => 429,
            ApiError::Internal => 500,
            ApiError::Misconfigured | ApiError::ProviderUnavailable => 503,
            ApiError::Unrecognized { status, .. } => *status,
            ApiError::Transport(_) | ApiError::Protocol(_) => return None,
        })
    }

    /// The error code, when the error came from an answer.
    pub fn code(&self) -> Option<&str> {
        Some(match self {
            ApiError::BadRequest => "bad_request",
            ApiError::InvalidUsername => "invalid_username",
            ApiError::ReservedUsername => "reserved_username",
            ApiError::UnsupportedKdf => "unsupported_kdf",
            ApiError::WeakKdf => "weak_kdf",
            ApiError::UsernameMismatch => "username_mismatch",
            ApiError::ChecksumMismatch => "checksum_mismatch",
            ApiError::Unauthorized { .. } => "unauthorized",
            ApiError::InvalidCredentials => "invalid_credentials",
            ApiError::ReauthRequired => "reauth_required",
            ApiError::FlowExpired { .. } => "flow_expired",
            ApiError::NonceReused => "nonce_reused",
            ApiError::ProviderDisabled => "provider_disabled",
            ApiError::ProviderDenied => "provider_denied",
            ApiError::IdentityTaken => "identity_taken",
            ApiError::LastMethod => "last_method",
            ApiError::AlreadyHasPassword => "already_has_password",
            ApiError::CredentialExists => "credential_exists",
            ApiError::ForbiddenOrigin => "forbidden_origin",
            ApiError::AccountDisabled => "account_disabled",
            ApiError::SignupClosed { .. } => "signup_closed",
            ApiError::NotFound => "not_found",
            ApiError::MethodNotAllowed => "method_not_allowed",
            ApiError::UsernameTaken => "username_taken",
            ApiError::AccountIdTaken => "account_id_taken",
            ApiError::TooLarge => "too_large",
            ApiError::QuotaExceeded { .. } => "quota_exceeded",
            ApiError::UnsupportedMediaType => "unsupported_media_type",
            ApiError::RateLimited { .. } => "rate_limited",
            ApiError::Internal => "internal",
            ApiError::Misconfigured => "misconfigured",
            ApiError::ProviderUnavailable => "provider_unavailable",
            ApiError::Unrecognized { code, .. } => code,
            ApiError::Transport(_) | ApiError::Protocol(_) => return None,
        })
    }

    /// The error body the Worker sends for this error, without `message`
    /// (`MemoryServer` answers with it; conformance compares it).
    pub fn body(&self) -> Option<Value> {
        let code = self.code()?;
        let mut body = json!({ "error": code });
        let extras = match self {
            ApiError::Unauthorized { reason } => json!({"reason": reason.as_str()}),
            ApiError::ReauthRequired => json!({"reason": "reauth"}),
            ApiError::SignupClosed { reason } => json!({"reason": reason.as_str()}),
            ApiError::QuotaExceeded { quota, used, limit } => {
                json!({"quota": quota.as_str(), "used": used, "limit": limit})
            }
            ApiError::RateLimited { retry_after, limit } => {
                json!({"retry_after": retry_after, "limit": limit})
            }
            _ => json!({}),
        };
        if let (Value::Object(body), Value::Object(extras)) = (&mut body, extras) {
            body.extend(extras);
        }
        Some(body)
    }

    /// Reads a non-2xx answer: its status and JSON body. Unknown keys are
    /// ignored; a missing or unknown code gives [`ApiError::Unrecognized`];
    /// missing extras fall back to `unknown` reasons and zeros. A body that
    /// isn't a JSON object gives [`ApiError::Protocol`].
    pub fn from_response(status: u16, body: &[u8]) -> ApiError {
        let Ok(Value::Object(body)) = serde_json::from_slice::<Value>(body) else {
            return ApiError::Protocol(format!("a {status} answer without a JSON error body"));
        };
        let text = |key: &str| body.get(key).and_then(Value::as_str).unwrap_or("");
        let number = |key: &str| body.get(key).and_then(Value::as_u64).unwrap_or(0);
        let code = text("error");
        let error = match code {
            "bad_request" => ApiError::BadRequest,
            "invalid_username" => ApiError::InvalidUsername,
            "reserved_username" => ApiError::ReservedUsername,
            "unsupported_kdf" => ApiError::UnsupportedKdf,
            "weak_kdf" => ApiError::WeakKdf,
            "username_mismatch" => ApiError::UsernameMismatch,
            "checksum_mismatch" => ApiError::ChecksumMismatch,
            "unauthorized" => ApiError::Unauthorized {
                reason: SignOutReason::parse(text("reason")),
            },
            "invalid_credentials" => ApiError::InvalidCredentials,
            "reauth_required" => ApiError::ReauthRequired,
            "flow_expired" => ApiError::FlowExpired {
                gone: status == 410,
            },
            "nonce_reused" => ApiError::NonceReused,
            "provider_disabled" => ApiError::ProviderDisabled,
            "provider_denied" => ApiError::ProviderDenied,
            "identity_taken" => ApiError::IdentityTaken,
            "last_method" => ApiError::LastMethod,
            "already_has_password" => ApiError::AlreadyHasPassword,
            "credential_exists" => ApiError::CredentialExists,
            "forbidden_origin" => ApiError::ForbiddenOrigin,
            "account_disabled" => ApiError::AccountDisabled,
            "signup_closed" => ApiError::SignupClosed {
                reason: match text("reason") {
                    "full" => SignupClosedReason::Full,
                    "busy" => SignupClosedReason::Busy,
                    _ => SignupClosedReason::Closed,
                },
            },
            "not_found" => ApiError::NotFound,
            "method_not_allowed" => ApiError::MethodNotAllowed,
            "username_taken" => ApiError::UsernameTaken,
            "account_id_taken" => ApiError::AccountIdTaken,
            "too_large" => ApiError::TooLarge,
            "quota_exceeded" => ApiError::QuotaExceeded {
                quota: QuotaKind::ALL
                    .into_iter()
                    .find(|q| q.as_str() == text("quota"))
                    .unwrap_or(QuotaKind::Server),
                used: number("used"),
                limit: number("limit"),
            },
            "unsupported_media_type" => ApiError::UnsupportedMediaType,
            "rate_limited" => ApiError::RateLimited {
                retry_after: number("retry_after"),
                limit: text("limit").to_owned(),
            },
            "internal" => ApiError::Internal,
            "misconfigured" => ApiError::Misconfigured,
            "provider_unavailable" => ApiError::ProviderUnavailable,
            _ => {
                return ApiError::Unrecognized {
                    status,
                    code: code.to_owned(),
                };
            }
        };
        if error.status() == Some(status) {
            error
        } else {
            ApiError::Unrecognized {
                status,
                code: code.to_owned(),
            }
        }
    }
}

/// A session token: `kanso_` and 43 base64url characters. Zeroized on drop;
/// its `Debug` is redacted.
#[derive(Clone, PartialEq, Eq)]
pub struct SessionToken(zeroize::Zeroizing<String>);

impl SessionToken {
    pub fn new(token: impl Into<String>) -> SessionToken {
        SessionToken(zeroize::Zeroizing::new(token.into()))
    }

    /// The token text, for the `Authorization` header and the Keychain item.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for SessionToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SessionToken(<redacted>)")
    }
}

impl Serialize for SessionToken {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for SessionToken {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(SessionToken::new(String::deserialize(deserializer)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn every_error() -> Vec<ApiError> {
        let mut all = vec![
            ApiError::BadRequest,
            ApiError::InvalidUsername,
            ApiError::ReservedUsername,
            ApiError::UnsupportedKdf,
            ApiError::WeakKdf,
            ApiError::UsernameMismatch,
            ApiError::ChecksumMismatch,
            ApiError::InvalidCredentials,
            ApiError::ReauthRequired,
            ApiError::FlowExpired { gone: false },
            ApiError::FlowExpired { gone: true },
            ApiError::NonceReused,
            ApiError::ProviderDisabled,
            ApiError::ProviderDenied,
            ApiError::IdentityTaken,
            ApiError::LastMethod,
            ApiError::AlreadyHasPassword,
            ApiError::CredentialExists,
            ApiError::ProviderUnavailable,
            ApiError::ForbiddenOrigin,
            ApiError::AccountDisabled,
            ApiError::NotFound,
            ApiError::MethodNotAllowed,
            ApiError::UsernameTaken,
            ApiError::AccountIdTaken,
            ApiError::TooLarge,
            ApiError::UnsupportedMediaType,
            ApiError::RateLimited {
                retry_after: 60,
                limit: "login-user".into(),
            },
            ApiError::Internal,
            ApiError::Misconfigured,
        ];
        all.extend(
            SignOutReason::ALL
                .into_iter()
                .map(|reason| ApiError::Unauthorized { reason }),
        );
        all.extend(
            [
                SignupClosedReason::Closed,
                SignupClosedReason::Full,
                SignupClosedReason::Busy,
            ]
            .map(|reason| ApiError::SignupClosed { reason }),
        );
        all.extend(
            QuotaKind::ALL
                .into_iter()
                .map(|quota| ApiError::QuotaExceeded {
                    quota,
                    used: 7,
                    limit: 9,
                }),
        );
        all
    }

    /// Round trip: every coded error survives being sent as the Worker's
    /// body and read back by the client, with a `message` and unknown keys
    /// added.
    #[test]
    fn every_error_round_trips_through_its_body() {
        for error in every_error() {
            let mut body = error.body().unwrap();
            body["message"] = "for logs".into();
            body["future_key"] = 1.into();
            let read = ApiError::from_response(
                error.status().unwrap(),
                &serde_json::to_vec(&body).unwrap(),
            );
            assert_eq!(read, error);
        }
    }

    /// Only an answer lost after sending, a 5xx or an unreadable 2xx may
    /// have changed the server; every coded 4xx refused the request, and an
    /// unsent one never arrived.
    #[test]
    fn which_errors_may_have_applied() {
        // 503 misconfigured is answered before the Worker does anything.
        for error in every_error() {
            let applied = matches!(error, ApiError::Internal);
            assert_eq!(error.may_have_applied(), applied, "{error}");
            assert!(!error.was_unsent());
        }
        let lost = ApiError::Transport("connection reset".into());
        assert!(lost.may_have_applied() && !lost.was_unsent());
        let unsent = ApiError::unsent("dns failed");
        assert!(matches!(unsent, ApiError::Transport(_)));
        assert!(unsent.was_unsent() && !unsent.may_have_applied());
        assert!(ApiError::Protocol("bad json".into()).may_have_applied());
        let newer = |status| ApiError::Unrecognized {
            status,
            code: "new".into(),
        };
        assert!(newer(502).may_have_applied() && !newer(422).may_have_applied());
    }

    #[test]
    fn odd_answers_are_unrecognized_or_protocol_errors() {
        assert_eq!(
            ApiError::from_response(418, br#"{"error":"teapot"}"#),
            ApiError::Unrecognized {
                status: 418,
                code: "teapot".into()
            }
        );
        // A known code under the wrong status is not trusted.
        assert_eq!(
            ApiError::from_response(500, br#"{"error":"not_found"}"#),
            ApiError::Unrecognized {
                status: 500,
                code: "not_found".into()
            }
        );
        assert!(matches!(
            ApiError::from_response(502, b"<html>bad gateway</html>"),
            ApiError::Protocol(_)
        ));
        assert_eq!(
            ApiError::from_response(401, br#"{"error":"unauthorized","reason":"bored"}"#),
            ApiError::Unauthorized {
                reason: SignOutReason::Unknown
            }
        );
    }

    #[test]
    fn push_results_use_the_wire_shapes() {
        let cases = [
            (
                r#"{"id":"note:a","status":"ok","seq":3}"#,
                WirePushResult::Ok {
                    id: "note:a".into(),
                    seq: 3,
                },
            ),
            (
                r#"{"id":"note:a","status":"conflict","current":null}"#,
                WirePushResult::Lost {
                    id: "note:a".into(),
                },
            ),
            (
                r#"{"id":"note:a","status":"behind"}"#,
                WirePushResult::Behind {
                    id: "note:a".into(),
                },
            ),
            (
                r#"{"id":"note:a","status":"conflict","current":{"id":"note:a","rev":"r","deleted":true,"sealed":"s"}}"#,
                WirePushResult::Conflict {
                    id: "note:a".into(),
                    current: WireDoc {
                        id: "note:a".into(),
                        rev: "r".into(),
                        deleted: true,
                        sealed: "s".into(),
                    },
                },
            ),
        ];
        for (text, expected) in cases {
            let parsed: WirePushResult = serde_json::from_str(text).unwrap();
            assert_eq!(parsed, expected);
            let written = serde_json::to_value(&parsed).unwrap();
            assert_eq!(written, serde_json::from_str::<Value>(text).unwrap());
        }
        assert!(
            serde_json::from_str::<WirePushResult>(r#"{"id":"note:a","status":"conflict"}"#)
                .is_err()
        );
        assert!(
            serde_json::from_str::<WirePushResult>(r#"{"id":"note:a","status":"maybe"}"#).is_err()
        );
        // Unknown keys are ignored.
        let parsed: WirePushResult =
            serde_json::from_str(r#"{"id":"note:a","status":"ok","seq":1,"x":2}"#).unwrap();
        assert_eq!(
            parsed,
            WirePushResult::Ok {
                id: "note:a".into(),
                seq: 1
            }
        );
    }

    #[test]
    fn pattern_examples() {
        assert!(is_wire_id("note:Q2hhbmdlTWVQbGVhc2UxMg"));
        assert!(is_wire_id("tag:Q2hhbmdlTWVQbGVhc2UxMg"));
        assert!(!is_wire_id("note:Q2hhbmdlTWVQbGVhc2UxM"));
        assert!(!is_wire_id("note:Q2hhbmdlTWVQbGVhc2Ux+g"));
        assert!(!is_wire_id("page:Q2hhbmdlTWVQbGVhc2UxMg"));
        assert!(!is_wire_id("note:07HmFZwo"));
        assert!(is_wire_file_id("file:Q2hhbmdlTWVQbGVhc2UxMg"));
        assert!(!is_wire_file_id("note:Q2hhbmdlTWVQbGVhc2UxMg"));
        assert!(is_wire_rev("0123456789abcdef0123456789abcdef"));
        assert!(!is_wire_rev("0123456789ABCDEF0123456789abcdef"));
        assert!(!is_wire_rev("1-0123456789abcdef0123456789abcd"));
        assert!(is_account_id("a_0123456789abcdef0123456789abcdef"));
        assert!(!is_account_id("d_0123456789abcdef0123456789abcdef"));
        assert!(is_device_id("d_0123456789abcdef0123456789abcdef"));
        assert!(!is_device_id("d_0123456789abcdef0123456789abcde"));
        assert!(is_hex16("0123456789abcdef"));
        assert!(!is_hex16("0123456789abcdeg"));
        assert!(is_token(&format!("kanso_{}", "A".repeat(43))));
        assert!(!is_token(&format!("kanso_{}", "A".repeat(42))));
        assert!(!is_token(&format!("Bearer {}", "A".repeat(43))));
        assert!(is_sealed(&"A".repeat(55), MIN_SEALED_LEN, MAX_SEALED_LEN));
        assert!(!is_sealed(&"A".repeat(54), MIN_SEALED_LEN, MAX_SEALED_LEN));
        assert!(!is_sealed(
            &format!("{}=", "A".repeat(55)),
            MIN_SEALED_LEN,
            MAX_SEALED_LEN
        ));
    }

    #[test]
    fn session_tokens_never_show_in_debug() {
        let token = SessionToken::new("kanso_secret");
        assert!(!format!("{token:?}").contains("secret"));
        assert_eq!(token.expose(), "kanso_secret");
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Oracle: the hand-written wire-ID check agrees with the spec's
        /// pattern `^(note|book|tag|file):[A-Za-z0-9_-]{22}$`, spelled out
        /// as a proptest regex.
        #[test]
        fn prop_wire_ids_match_the_pattern(id in "(note|book|tag|file|page|):[A-Za-z0-9_+/=-]{20,24}") {
            let kind_ok = ["note:", "book:", "tag:", "file:"].iter().any(|k| id.starts_with(k));
            let rest = id.split_once(':').map_or("", |(_, r)| r);
            let expected = kind_ok && rest.len() == 22
                && rest.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
            prop_assert_eq!(is_wire_id(&id), expected);
        }

        /// Refinement: any accepted session token is exactly 49 ASCII bytes.
        #[test]
        fn prop_accepted_tokens_have_the_token_shape(token in "(kanso_|Kanso_|)[A-Za-z0-9_+/-]{41,45}") {
            if is_token(&token) {
                prop_assert_eq!(token.len(), 49);
                prop_assert!(token.starts_with("kanso_"));
            }
        }

        /// Round trip: a push result is read back as written.
        #[test]
        fn prop_push_results_round_trip(seq in 1i64.., which in 0..4u8, deleted: bool) {
            let id = "note:Q2hhbmdlTWVQbGVhc2UxMg".to_owned();
            let result = match which {
                0 => WirePushResult::Ok { id, seq },
                1 => WirePushResult::Lost { id },
                2 => WirePushResult::Behind { id },
                _ => WirePushResult::Conflict {
                    id: id.clone(),
                    current: WireDoc { id, rev: "0".repeat(32), deleted, sealed: "A".repeat(55) },
                },
            };
            let text = serde_json::to_string(&result).unwrap();
            prop_assert_eq!(serde_json::from_str::<WirePushResult>(&text).unwrap(), result);
        }
    }
}
