//! The server behind the client: [`Backend`] (`HttpBackend` in the app,
//! `MemoryServer` in tests) and [`Api`], every v2 route of one server with
//! its request and response bodies (docs/ACCOUNTS.md › Routes).
//!
//! Frozen by step 1. Field names are the wire's; unknown keys are ignored on
//! both sides. `cloud/test/fixtures/auth.json` and `conformance.json` are
//! written in these shapes, and the Rust conformance runner (step 2a) drives
//! `MemoryServer` through [`Api`] with them.

use std::fmt;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

use crate::account::{Session, SignupState};
use crate::crypto::KdfParams;
use crate::provider::{
    GoogleFinishRequest, GoogleStartRequest, IdentityStartRequest, LinkOutcome, MethodsReply,
    Provider, ProviderPollRequest, ProviderSession, ProviderStart, ProviderStartRequest,
    ProviderStep, SocialSignupRequest, UnlinkRequest,
};
use crate::wire::{
    ApiError, PushRequest, PushResponse, PutFileResponse, SessionToken, WirePage, WirePushDoc,
    WirePushResult, WireRemote,
};

/// The server a library syncs with. `Send + Sync`: the app shares it as
/// `Arc<dyn Backend>` between the UI and its account and sync threads.
pub trait Backend: Send + Sync {
    /// Every route of the server at `server` (a URL that passed
    /// [`crate::check_endpoint`]).
    fn api(&self, server: &str) -> Box<dyn Api + '_>;

    /// The sync routes bound to `session` (its server and token).
    fn remote(&self, session: &Session) -> Box<dyn WireRemote + '_> {
        Box::new(SessionRemote {
            api: self.api(&session.server),
            token: session.token().clone(),
        })
    }

    /// The lowest KDF parameters this backend's servers may ask for:
    /// [`KdfParams::FLOOR`] for `HttpBackend`, [`KdfParams::TEST`] for
    /// `MemoryServer`. Nothing else can lower it.
    fn kdf_floor(&self) -> KdfParams;

    /// The parameters new accounts and password changes use.
    fn kdf_default(&self) -> KdfParams;

    /// How long to wait before sending a request again after its answer was
    /// lost (`retry` is 1 for the first resend): about 1 s, then 4 s, so a
    /// Wi-Fi link that dropped has time to come back. Tests that drop
    /// answers on purpose shorten it.
    fn retry_pause(&self, retry: usize) -> Duration {
        match retry {
            0 | 1 => Duration::from_secs(1),
            _ => Duration::from_secs(4),
        }
    }
}

/// [`Backend::remote`]'s default: [`Api`]'s sync routes with one token.
struct SessionRemote<'a> {
    api: Box<dyn Api + 'a>,
    token: SessionToken,
}

impl WireRemote for SessionRemote<'_> {
    fn push(&self, docs: &[WirePushDoc]) -> Result<Vec<WirePushResult>, ApiError> {
        let request = PushRequest {
            docs: docs.to_vec(),
        };
        Ok(self.api.push(&self.token, &request)?.results)
    }

    fn changes(&self, since: i64, limit: usize) -> Result<WirePage, ApiError> {
        self.api.changes(&self.token, Some(since), Some(limit))
    }

    fn put_file(&self, wire_id: &str, sealed: &[u8]) -> Result<(), ApiError> {
        self.api.put_file(&self.token, wire_id, sealed).map(|_| ())
    }

    fn get_file(&self, wire_id: &str) -> Result<Option<Vec<u8>>, ApiError> {
        self.api.get_file(&self.token, wire_id)
    }
}

/// Every v2 route of one server. Session routes take the bearer token
/// explicitly, so the conformance runner can replay any token it holds.
/// Errors are the route's status-table answers ([`ApiError`]).
pub trait Api {
    /// `GET /v2/health`.
    fn health(&self) -> Result<Health, ApiError>;
    /// `POST /v2/auth/prelogin`.
    fn prelogin(&self, request: &PreloginRequest) -> Result<PreloginResponse, ApiError>;
    /// `POST /v2/auth/signup` (201).
    fn signup(&self, request: &SignupRequest) -> Result<SessionResponse, ApiError>;
    /// `POST /v2/auth/login`.
    fn login(&self, request: &LoginRequest) -> Result<LoginResponse, ApiError>;
    /// `POST /v2/auth/recover/start`.
    fn recover_start(
        &self,
        request: &RecoverStartRequest,
    ) -> Result<RecoverStartResponse, ApiError>;
    /// `POST /v2/auth/recover/finish`.
    fn recover_finish(&self, request: &RecoverFinishRequest) -> Result<SessionResponse, ApiError>;
    /// `POST /v2/auth/logout` (204).
    fn logout(&self, token: &SessionToken) -> Result<(), ApiError>;
    /// `GET /v2/account`.
    fn account(&self, token: &SessionToken) -> Result<AccountResponse, ApiError>;
    /// `POST /v2/account/password`.
    fn change_password(
        &self,
        token: &SessionToken,
        request: &PasswordRequest,
    ) -> Result<PasswordResponse, ApiError>;
    /// `POST /v2/account/recovery-key`.
    fn replace_recovery_key(
        &self,
        token: &SessionToken,
        request: &RecoveryKeyRequest,
    ) -> Result<RecoveryKeyResponse, ApiError>;
    /// `POST /v2/account/delete`.
    fn delete_account(
        &self,
        token: &SessionToken,
        request: &DeleteRequest,
    ) -> Result<DeleteResponse, ApiError>;
    /// `PUT /v2/account/device` (204).
    fn set_device_name(
        &self,
        token: &SessionToken,
        request: &DeviceNameRequest,
    ) -> Result<(), ApiError>;
    /// `GET /v2/account/devices`.
    fn devices(&self, token: &SessionToken) -> Result<DevicesResponse, ApiError>;
    /// `DELETE /v2/account/devices/<device_id>` (204).
    fn revoke_device(&self, token: &SessionToken, device_id: &str) -> Result<(), ApiError>;
    /// `POST /v2/push`.
    fn push(&self, token: &SessionToken, request: &PushRequest) -> Result<PushResponse, ApiError>;
    /// `GET /v2/changes?since=&limit=`; `None` leaves the parameter out
    /// (`since` 0, `limit` 100).
    fn changes(
        &self,
        token: &SessionToken,
        since: Option<i64>,
        limit: Option<usize>,
    ) -> Result<WirePage, ApiError>;
    /// `PUT /v2/files/<wire_id>`; the implementation sends `x-kanso-sha256`.
    fn put_file(
        &self,
        token: &SessionToken,
        wire_id: &str,
        sealed: &[u8],
    ) -> Result<PutFileResponse, ApiError>;
    /// `GET /v2/files/<wire_id>`; `None` on 404.
    fn get_file(&self, token: &SessionToken, wire_id: &str) -> Result<Option<Vec<u8>>, ApiError>;

    // Provider sign-in (docs/ACCOUNTS.md › Sign-in routes). A backend
    // without these defaults to 404, which is also what an older server
    // answers.

    /// `POST /v2/auth/<apple|github>/start`.
    fn provider_start(
        &self,
        _provider: &Provider,
        _request: &ProviderStartRequest,
    ) -> Result<ProviderStart, ApiError> {
        Err(ApiError::NotFound)
    }
    /// `POST /v2/auth/google/start`.
    fn google_start(&self, _request: &GoogleStartRequest) -> Result<ProviderStart, ApiError> {
        Err(ApiError::NotFound)
    }
    /// `POST /v2/auth/<apple|github>/poll`.
    fn provider_poll(
        &self,
        _provider: &Provider,
        _request: &ProviderPollRequest,
    ) -> Result<ProviderStep, ApiError> {
        Err(ApiError::NotFound)
    }
    /// `POST /v2/auth/google/finish`.
    fn google_finish(&self, _request: &GoogleFinishRequest) -> Result<ProviderStep, ApiError> {
        Err(ApiError::NotFound)
    }
    /// `POST /v2/auth/social/signup` (201).
    fn social_signup(&self, _request: &SocialSignupRequest) -> Result<ProviderSession, ApiError> {
        Err(ApiError::NotFound)
    }
    /// `GET /v2/account/sign-in`.
    fn sign_in_methods(&self, _token: &SessionToken) -> Result<MethodsReply, ApiError> {
        Err(ApiError::NotFound)
    }
    /// `POST /v2/account/sign-in/identities/<provider>/start`.
    fn identity_start(
        &self,
        _token: &SessionToken,
        _provider: &Provider,
        _request: &IdentityStartRequest,
    ) -> Result<ProviderStart, ApiError> {
        Err(ApiError::NotFound)
    }
    /// The completion of a link or proof flow on Apple or GitHub.
    fn identity_poll(
        &self,
        _token: &SessionToken,
        _provider: &Provider,
        _request: &ProviderPollRequest,
    ) -> Result<LinkOutcome, ApiError> {
        Err(ApiError::NotFound)
    }
    /// The completion of a link or proof flow on Google.
    fn identity_finish_google(
        &self,
        _token: &SessionToken,
        _provider: &Provider,
        _request: &GoogleFinishRequest,
    ) -> Result<LinkOutcome, ApiError> {
        Err(ApiError::NotFound)
    }
    /// `POST /v2/account/sign-in/identities/<provider>/unlink` (204).
    fn unlink_identity(
        &self,
        _token: &SessionToken,
        _provider: &Provider,
        _request: &UnlinkRequest,
    ) -> Result<(), ApiError> {
        Err(ApiError::NotFound)
    }
}

/// The routes, by the names the fixtures use (`"request": "recover_start"`),
/// for [`crate::MemoryServer::drop_next_responses`] and its stats.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Route {
    Health,
    Prelogin,
    Signup,
    Login,
    RecoverStart,
    RecoverFinish,
    Logout,
    Account,
    Password,
    RecoveryKey,
    Delete,
    SetDevice,
    Devices,
    RevokeDevice,
    Push,
    Changes,
    PutFile,
    GetFile,
    ProviderStart,
    ProviderPoll,
    GoogleStart,
    GoogleFinish,
    SocialSignup,
    SignInMethods,
    IdentityStart,
    IdentityUnlink,
}

impl Route {
    pub const ALL: [Route; 18] = [
        Route::Health,
        Route::Prelogin,
        Route::Signup,
        Route::Login,
        Route::RecoverStart,
        Route::RecoverFinish,
        Route::Logout,
        Route::Account,
        Route::Password,
        Route::RecoveryKey,
        Route::Delete,
        Route::SetDevice,
        Route::Devices,
        Route::RevokeDevice,
        Route::Push,
        Route::Changes,
        Route::PutFile,
        Route::GetFile,
    ];

    /// The method and path (with `<…>` placeholders).
    pub fn http(self) -> (&'static str, &'static str) {
        match self {
            Route::Health => ("GET", "/v2/health"),
            Route::Prelogin => ("POST", "/v2/auth/prelogin"),
            Route::Signup => ("POST", "/v2/auth/signup"),
            Route::Login => ("POST", "/v2/auth/login"),
            Route::RecoverStart => ("POST", "/v2/auth/recover/start"),
            Route::RecoverFinish => ("POST", "/v2/auth/recover/finish"),
            Route::Logout => ("POST", "/v2/auth/logout"),
            Route::Account => ("GET", "/v2/account"),
            Route::Password => ("POST", "/v2/account/password"),
            Route::RecoveryKey => ("POST", "/v2/account/recovery-key"),
            Route::Delete => ("POST", "/v2/account/delete"),
            Route::SetDevice => ("PUT", "/v2/account/device"),
            Route::Devices => ("GET", "/v2/account/devices"),
            Route::RevokeDevice => ("DELETE", "/v2/account/devices/<device_id>"),
            Route::Push => ("POST", "/v2/push"),
            Route::Changes => ("GET", "/v2/changes"),
            Route::PutFile => ("PUT", "/v2/files/<wire_id>"),
            Route::GetFile => ("GET", "/v2/files/<wire_id>"),
            Route::ProviderStart => ("POST", "/v2/auth/<provider>/start"),
            Route::ProviderPoll => ("POST", "/v2/auth/<provider>/poll"),
            Route::GoogleStart => ("POST", "/v2/auth/google/start"),
            Route::GoogleFinish => ("POST", "/v2/auth/google/finish"),
            Route::SocialSignup => ("POST", "/v2/auth/social/signup"),
            Route::SignInMethods => ("GET", "/v2/account/sign-in"),
            Route::IdentityStart => ("POST", "/v2/account/sign-in/identities/<provider>/start"),
            Route::IdentityUnlink => ("POST", "/v2/account/sign-in/identities/<provider>/unlink"),
        }
    }

    /// Whether the route takes `Authorization: Bearer`.
    pub fn needs_session(self) -> bool {
        !matches!(
            self,
            Route::Health
                | Route::Prelogin
                | Route::Signup
                | Route::Login
                | Route::RecoverStart
                | Route::RecoverFinish
                | Route::ProviderStart
                | Route::ProviderPoll
                | Route::GoogleStart
                | Route::GoogleFinish
                | Route::SocialSignup
        )
    }
}

// ---------------------------------------------------------------------------
// Shared shapes
// ---------------------------------------------------------------------------

/// `"kdf": {"alg": "argon2id", "version": 19, "m_kib", "t", "p"}`. `alg`
/// and `version` are carried as given so a server can refuse others with
/// `unsupported_kdf`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WireKdf {
    pub alg: String,
    pub version: u32,
    pub m_kib: u32,
    pub t: u32,
    pub p: u32,
}

impl From<KdfParams> for WireKdf {
    fn from(params: KdfParams) -> WireKdf {
        WireKdf {
            alg: "argon2id".into(),
            version: 19,
            m_kib: params.m_kib,
            t: params.t,
            p: params.p,
        }
    }
}

impl WireKdf {
    /// The parameters, when `alg` is `argon2id` and `version` is 19.
    pub fn params(&self) -> Option<KdfParams> {
        (self.alg == "argon2id" && self.version == 19).then_some(KdfParams {
            m_kib: self.m_kib,
            t: self.t,
            p: self.p,
        })
    }
}

/// The device of a sign-up or a recovery: its per-account ID, its sealed
/// name (or null), and its proof (43 base64url characters; security
/// re-check 5).
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewDevice {
    pub id: String,
    pub sealed_name: Option<String>,
    pub proof: String,
}

/// The device of a sign-in: no name yet (the client doesn't hold the data
/// key), but its proof, which alone exempts a returning device from
/// `login-user`.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoginDevice {
    pub id: String,
    pub proof: String,
}

/// `{"id", "sealed_name"}`: the device a sign-in evicted.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WireDevice {
    pub id: String,
    pub sealed_name: Option<String>,
}

// ---------------------------------------------------------------------------
// Bodies, route by route
// ---------------------------------------------------------------------------

/// `GET /v2/health` 200. A setting of `only:<name>` reports `closed`;
/// `full` when `max_accounts` is reached; the daily cap isn't reported.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Health {
    pub ok: bool,
    pub protocol: u32,
    pub signup: SignupState,
    /// The sign-in methods this server offers. An older server omits it,
    /// which means username and password only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth: Option<HealthAuth>,
}

/// `"auth"` in `GET /v2/health`: the deployment's advertisement.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthAuth {
    pub password: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub passkeys: Option<HealthPasskeys>,
    #[serde(default)]
    pub providers: Vec<Provider>,
}

/// `"passkeys"` in the advertisement: the relying party and its origins.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthPasskeys {
    pub rp_id: String,
    #[serde(default)]
    pub origins: Vec<String>,
    pub security_key: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreloginRequest {
    pub username: String,
}

/// Unknown names get a deterministic fake salt and account ID and the
/// default parameters.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreloginResponse {
    pub kdf: WireKdf,
    /// 22 base64url characters (16 bytes).
    pub salt: String,
    pub account_id: String,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignupRequest {
    pub username: String,
    /// `a_` + 32 hex, chosen by the client.
    pub account_id: String,
    pub kdf: WireKdf,
    pub salt: String,
    pub auth_key: String,
    pub wrapped_key: String,
    pub recovery_auth_key: String,
    pub recovery_wrapped_key: String,
    pub key_id: String,
    pub device: NewDevice,
}

/// Sign-up (201) and recovery finish (200): the new session. A sign-up
/// retried with the same username and auth key answers 201 with the
/// existing account (security re-check 7).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionResponse {
    pub account_id: String,
    pub username: String,
    pub token: SessionToken,
    pub epoch: String,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub auth_key: String,
    pub device: LoginDevice,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoginResponse {
    pub account_id: String,
    pub username: String,
    pub token: SessionToken,
    pub wrapped_key: String,
    pub key_id: String,
    pub epoch: String,
    pub evicted_device: Option<WireDevice>,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoverStartRequest {
    pub username: String,
    pub recovery_auth_key: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoverStartResponse {
    pub account_id: String,
    pub recovery_wrapped_key: String,
    pub key_id: String,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoverFinishRequest {
    pub username: String,
    pub recovery_auth_key: String,
    pub kdf: WireKdf,
    pub salt: String,
    pub auth_key: String,
    pub wrapped_key: String,
    pub recovery_auth_key_new: String,
    pub recovery_wrapped_key_new: String,
    pub device: NewDevice,
}

/// `GET /v2/account` 200.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountResponse {
    pub account_id: String,
    pub username: String,
    pub created_at: i64,
    pub usage: WireUsage,
    pub quota: WireQuota,
}

/// Usage: live documents, charged document bytes (`length(sealed) + 512`
/// per row, tombstones included), stored files, charged file bytes (`size +
/// 1024` per file), and today's `writes-account` count.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WireUsage {
    pub docs: u64,
    pub doc_bytes: u64,
    pub files: u64,
    pub file_bytes: u64,
    pub writes_today: u64,
}

/// The account's effective quotas.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WireQuota {
    pub docs: u64,
    pub doc_bytes: u64,
    pub files: u64,
    pub file_bytes: u64,
    pub writes_per_day: u64,
}

/// A new password and, with it, a new recovery key: the old key stops
/// working, like every proof of the old password (docs/ACCOUNTS.md › Routes).
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PasswordRequest {
    pub current_auth_key: String,
    pub kdf: WireKdf,
    pub salt: String,
    pub auth_key: String,
    pub wrapped_key: String,
    pub recovery_auth_key: String,
    pub recovery_wrapped_key: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PasswordResponse {
    pub signed_out_devices: u64,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryKeyRequest {
    pub current_auth_key: String,
    pub recovery_auth_key: String,
    pub recovery_wrapped_key: String,
}

/// `{}`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryKeyResponse {}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeleteRequest {
    pub auth_key: String,
    pub username: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeleteResponse {
    pub deleted: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceNameRequest {
    pub sealed_name: String,
}

/// Active sessions only, most recently seen first.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DevicesResponse {
    pub devices: Vec<DeviceRow>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceRow {
    pub id: String,
    pub sealed_name: Option<String>,
    pub created_at: i64,
    /// Updated when more than an hour old, so known to the hour.
    pub last_seen_at: i64,
    pub current: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kdf_shapes() {
        let wire = WireKdf::from(KdfParams::DEFAULT);
        assert_eq!(
            serde_json::to_value(&wire).unwrap(),
            serde_json::json!({"alg": "argon2id", "version": 19, "m_kib": 1048576, "t": 3, "p": 4})
        );
        assert_eq!(wire.params(), Some(KdfParams::DEFAULT));
        assert_eq!(
            WireKdf {
                alg: "argon2d".into(),
                ..wire.clone()
            }
            .params(),
            None
        );
        assert_eq!(
            WireKdf {
                version: 16,
                ..wire
            }
            .params(),
            None
        );
    }

    /// No request's `Debug` shows an auth key or a device proof, and every
    /// other field still shows.
    #[test]
    fn requests_never_print_their_secrets() {
        let kdf = WireKdf::from(KdfParams::DEFAULT);
        let device = NewDevice {
            id: "d_shown".into(),
            sealed_name: Some("name-shown".into()),
            proof: "SECRET-proof".into(),
        };
        let login_device = LoginDevice {
            id: "d_shown".into(),
            proof: "SECRET-proof".into(),
        };
        let printed = [
            format!("{device:?}"),
            format!("{login_device:?}"),
            format!(
                "{:?}",
                SignupRequest {
                    username: "alice".into(),
                    account_id: "a_shown".into(),
                    kdf: kdf.clone(),
                    salt: "salt-shown".into(),
                    auth_key: "SECRET-auth".into(),
                    wrapped_key: "wrap-shown".into(),
                    recovery_auth_key: "SECRET-recovery".into(),
                    recovery_wrapped_key: "rwrap-shown".into(),
                    key_id: "kid-shown".into(),
                    device: device.clone(),
                }
            ),
            format!(
                "{:?}",
                LoginRequest {
                    username: "alice".into(),
                    auth_key: "SECRET-auth".into(),
                    device: login_device.clone(),
                }
            ),
            format!(
                "{:?}",
                RecoverStartRequest {
                    username: "alice".into(),
                    recovery_auth_key: "SECRET-recovery".into(),
                }
            ),
            format!(
                "{:?}",
                RecoverFinishRequest {
                    username: "alice".into(),
                    recovery_auth_key: "SECRET-recovery".into(),
                    kdf: kdf.clone(),
                    salt: "salt-shown".into(),
                    auth_key: "SECRET-auth".into(),
                    wrapped_key: "wrap-shown".into(),
                    recovery_auth_key_new: "SECRET-new".into(),
                    recovery_wrapped_key_new: "rwrap-shown".into(),
                    device: device.clone(),
                }
            ),
            format!(
                "{:?}",
                PasswordRequest {
                    current_auth_key: "SECRET-current".into(),
                    kdf: kdf.clone(),
                    salt: "salt-shown".into(),
                    auth_key: "SECRET-auth".into(),
                    wrapped_key: "wrap-shown".into(),
                    recovery_auth_key: "SECRET-recovery".into(),
                    recovery_wrapped_key: "rwrap-shown".into(),
                }
            ),
            format!(
                "{:?}",
                RecoveryKeyRequest {
                    current_auth_key: "SECRET-current".into(),
                    recovery_auth_key: "SECRET-recovery".into(),
                    recovery_wrapped_key: "rwrap-shown".into(),
                }
            ),
            format!(
                "{:?}",
                DeleteRequest {
                    auth_key: "SECRET-auth".into(),
                    username: "alice".into(),
                }
            ),
        ];
        for text in &printed {
            assert!(!text.contains("SECRET"), "{text}");
            assert!(text.contains("<redacted>"), "{text}");
        }
        assert!(printed[2].contains("a_shown") && printed[2].contains("d_shown"));
        assert!(printed[6].contains("rwrap-shown"));
    }

    #[test]
    fn route_names_are_the_fixture_names() {
        for route in Route::ALL {
            let name = serde_json::to_value(route).unwrap();
            let back: Route = serde_json::from_value(name.clone()).unwrap();
            assert_eq!(back, route, "{name}");
        }
        assert_eq!(
            serde_json::to_value(Route::RecoverStart).unwrap(),
            "recover_start"
        );
        assert!(Route::Push.needs_session() && !Route::Login.needs_session());
    }
}

// ---------------------------------------------------------------------------
// Secrets in requests (sec-client 6)
// ---------------------------------------------------------------------------

/// `Debug` that shows `$shown` and redacts `$secret`, and a `Drop` that
/// zeroizes `$secret`: auth keys and device proofs never reach a log and
/// don't outlive their request in memory.
macro_rules! secret_fields {
    ($name:ident { shown: [$($shown:ident),*], secret: [$($secret:ident),+] }) => {
        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct(stringify!($name))
                    $(.field(stringify!($shown), &self.$shown))*
                    $(.field(stringify!($secret), &"<redacted>"))+
                    .finish()
            }
        }

        impl Drop for $name {
            fn drop(&mut self) {
                $(self.$secret.zeroize();)+
            }
        }
    };
}

secret_fields!(NewDevice {
    shown: [id, sealed_name],
    secret: [proof]
});
secret_fields!(LoginDevice {
    shown: [id],
    secret: [proof]
});
secret_fields!(SignupRequest {
    shown: [
        username,
        account_id,
        kdf,
        salt,
        wrapped_key,
        recovery_wrapped_key,
        key_id,
        device
    ],
    secret: [auth_key, recovery_auth_key]
});
secret_fields!(LoginRequest {
    shown: [username, device],
    secret: [auth_key]
});
secret_fields!(RecoverStartRequest {
    shown: [username],
    secret: [recovery_auth_key]
});
secret_fields!(RecoverFinishRequest {
    shown: [
        username,
        kdf,
        salt,
        wrapped_key,
        recovery_wrapped_key_new,
        device
    ],
    secret: [recovery_auth_key, auth_key, recovery_auth_key_new]
});
secret_fields!(PasswordRequest {
    shown: [kdf, salt, wrapped_key, recovery_wrapped_key],
    secret: [current_auth_key, auth_key, recovery_auth_key]
});
secret_fields!(RecoveryKeyRequest {
    shown: [recovery_wrapped_key],
    secret: [current_auth_key, recovery_auth_key]
});
secret_fields!(DeleteRequest {
    shown: [username],
    secret: [auth_key]
});
