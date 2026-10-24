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
    /// `POST /v2/auth/passkey/start` (username-first to bind device proof).
    fn passkey_start(
        &self,
        _request: &PasskeyStartRequest,
    ) -> Result<PasskeyStartResponse, ApiError> {
        Err(ApiError::NotFound)
    }
    /// `POST /v2/auth/passkey/finish`.
    fn passkey_finish(
        &self,
        _request: &PasskeyFinishRequest,
    ) -> Result<PasskeySessionResponse, ApiError> {
        Err(ApiError::NotFound)
    }
    /// `POST /v2/auth/passkey/signup/start`.
    fn passkey_signup_start(
        &self,
        _request: &PasskeySignupStartRequest,
    ) -> Result<PasskeySignupStartResponse, ApiError> {
        Err(ApiError::NotFound)
    }
    /// `POST /v2/auth/passkey/signup/finish`.
    fn passkey_signup_finish(
        &self,
        _request: &PasskeySignupFinishRequest,
    ) -> Result<PasskeySessionResponse, ApiError> {
        Err(ApiError::NotFound)
    }
    /// `GET /v2/account/sign-in` (specified in revision 3; handler absent
    /// from this checkout's Worker).
    fn account_sign_in_methods(
        &self,
        _token: &SessionToken,
    ) -> Result<AccountSignInMethodsResponse, ApiError> {
        Err(ApiError::NotFound)
    }
    /// `POST /v2/account/sign-in/proof/start`.
    fn passkey_proof_start(
        &self,
        _token: &SessionToken,
        _request: &PasskeyProofStartRequest,
    ) -> Result<PasskeyProofStartResponse, ApiError> {
        Err(ApiError::NotFound)
    }
    /// `POST /v2/account/sign-in/passkeys/start`.
    fn account_passkeys_start(
        &self,
        _token: &SessionToken,
        _request: &AccountPasskeysStartRequest,
    ) -> Result<AccountPasskeysStartResponse, ApiError> {
        Err(ApiError::NotFound)
    }
    /// `POST /v2/account/sign-in/passkeys/finish`.
    fn account_passkeys_finish(
        &self,
        _token: &SessionToken,
        _request: &AccountPasskeysFinishRequest,
    ) -> Result<AccountPasskeysFinishResponse, ApiError> {
        Err(ApiError::NotFound)
    }
    /// `POST /v2/account/sign-in/passkeys/<credential_id>/remove` (204).
    fn account_passkey_remove(
        &self,
        _token: &SessionToken,
        _credential_id: &str,
        _request: &PasskeyProofRequest,
    ) -> Result<(), ApiError> {
        Err(ApiError::NotFound)
    }
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
}

/// The routes implemented by [`crate::MemoryServer`], by the names its
/// fixtures use (`"request": "recover_start"`) and its stats.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Route {
    Health,
    Prelogin,
    Signup,
    Login,
    PasskeyStart,
    PasskeyFinish,
    PasskeySignupStart,
    PasskeySignupFinish,
    AccountSignInMethods,
    PasskeyProofStart,
    AccountPasskeysStart,
    AccountPasskeysFinish,
    AccountPasskeyRemove,
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
}

impl Route {
    /// Routes the shared conformance fixtures cover. The four public passkey
    /// routes are implemented by `MemoryServer` and exercised by its own
    /// tests, but have no fixture case yet; the account-management passkeys
    /// are contracts only (neither the Worker nor `MemoryServer` serves them).
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

    /// Passkey HTTP contracts: the four public routes `MemoryServer` fakes,
    /// plus account-management routes described by revision 3 but not yet
    /// implemented by this checkout's Worker or `MemoryServer`.
    pub const PASSKEYS: [Route; 9] = [
        Route::PasskeyStart,
        Route::PasskeyFinish,
        Route::PasskeySignupStart,
        Route::PasskeySignupFinish,
        Route::AccountSignInMethods,
        Route::PasskeyProofStart,
        Route::AccountPasskeysStart,
        Route::AccountPasskeysFinish,
        Route::AccountPasskeyRemove,
    ];

    /// The method and path (with `<…>` placeholders).
    pub fn http(self) -> (&'static str, &'static str) {
        match self {
            Route::Health => ("GET", "/v2/health"),
            Route::Prelogin => ("POST", "/v2/auth/prelogin"),
            Route::Signup => ("POST", "/v2/auth/signup"),
            Route::Login => ("POST", "/v2/auth/login"),
            Route::PasskeyStart => ("POST", "/v2/auth/passkey/start"),
            Route::PasskeyFinish => ("POST", "/v2/auth/passkey/finish"),
            Route::PasskeySignupStart => ("POST", "/v2/auth/passkey/signup/start"),
            Route::PasskeySignupFinish => ("POST", "/v2/auth/passkey/signup/finish"),
            Route::AccountSignInMethods => ("GET", "/v2/account/sign-in"),
            Route::PasskeyProofStart => ("POST", "/v2/account/sign-in/proof/start"),
            Route::AccountPasskeysStart => ("POST", "/v2/account/sign-in/passkeys/start"),
            Route::AccountPasskeysFinish => ("POST", "/v2/account/sign-in/passkeys/finish"),
            Route::AccountPasskeyRemove => (
                "POST",
                "/v2/account/sign-in/passkeys/<credential_id>/remove",
            ),
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
                | Route::PasskeyStart
                | Route::PasskeyFinish
                | Route::PasskeySignupStart
                | Route::PasskeySignupFinish
                | Route::RecoverStart
                | Route::RecoverFinish
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
    /// Absent on older protocol-2 servers. An absent `passkeys` value means
    /// this server does not advertise passkey ceremonies.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth: Option<AuthAdvertisement>,
}

/// The sign-in methods advertised by the Worker.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthAdvertisement {
    pub password: bool,
    pub passkeys: Option<PasskeyAdvertisement>,
    pub providers: Vec<String>,
}

/// The passkey relying-party settings a local ceremony must use.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PasskeyAdvertisement {
    pub rp_id: String,
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

/// Device-bound username-first passkey sign-in start.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PasskeyStartRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    pub device: LoginDevice,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PasskeyStartResponse {
    pub flow_id: String,
    pub challenge: String,
    pub rp_id: String,
    pub allow_credentials: Vec<PasskeyAllowedCredential>,
    pub user_verification: String,
    pub timeout: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PasskeyAllowedCredential {
    pub id: String,
    pub transports: Vec<String>,
}

/// Passkey assertion result from the platform ceremony.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PasskeyFinishRequest {
    pub flow_id: String,
    pub credential_id: String,
    pub authenticator_data: String,
    pub client_data_json: String,
    pub signature: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_handle: Option<String>,
    pub device: LoginDevice,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PasskeySessionKind {
    Session,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PasskeySessionResponse {
    pub kind: PasskeySessionKind,
    pub account_id: String,
    pub username: String,
    pub token: SessionToken,
    pub key_id: String,
    pub epoch: String,
    pub has_password: bool,
    pub passkey_wrapped_key: Option<String>,
    pub evicted_device: Option<WireDevice>,
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PasskeySignupStartRequest {
    pub username: String,
    pub account_id: String,
    pub device: NewDevice,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PasskeySignupStartResponse {
    pub flow_id: String,
    pub challenge: String,
    pub rp_id: String,
    pub user: PasskeyUser,
    pub pub_key_cred_params: Vec<i32>,
    pub authenticator_selection: PasskeySelection,
    pub exclude_credentials: Vec<PasskeyAllowedCredential>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PasskeyUser {
    pub id: String,
    pub name: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PasskeySelection {
    #[serde(rename = "residentKey")]
    pub resident_key: String,
    #[serde(rename = "userVerification")]
    pub user_verification: String,
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PasskeySignupFinishRequest {
    pub flow_id: String,
    pub username: String,
    pub account_id: String,
    pub credential_id: String,
    pub attestation_object: String,
    pub client_data_json: String,
    pub transports: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub passkey_wrapped_key: Option<String>,
    pub recovery_auth_key: String,
    pub recovery_wrapped_key: String,
    pub key_id: String,
    pub device: NewDevice,
}

// Account method/proof contracts mirror revision 3. This worktree's Worker
// currently has no handlers for these session routes; the default Api methods
// return NotFound until that server-side pass lands.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountSignInMethodsResponse {
    pub username: String,
    pub has_password: bool,
    pub identities: Vec<LinkedIdentityResponse>,
    pub passkeys: Vec<AccountPasskeyResponse>,
    pub recovery: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinkedIdentityResponse {
    pub provider: String,
    pub created_at: i64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountPasskeyResponse {
    pub credential_id: String,
    pub created_at: i64,
    pub last_used_at: Option<i64>,
    pub prf: bool,
    pub transports: Vec<String>,
}

/// A fresh proof for a session-authenticated account change. `Debug` is
/// omitted because password proofs and assertion bytes are credentials.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProofRequest {
    Password {
        auth_key: String,
    },
    Passkey {
        flow: String,
        assertion: PasskeyProofAssertion,
    },
    Provider {
        flow: String,
    },
}

impl fmt::Debug for ProofRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProofRequest::Password { .. } => f
                .debug_struct("Password")
                .field("auth_key", &"<redacted>")
                .finish(),
            ProofRequest::Passkey { flow, assertion } => f
                .debug_struct("Passkey")
                .field("flow", flow)
                .field("assertion", assertion)
                .finish(),
            ProofRequest::Provider { flow } => {
                f.debug_struct("Provider").field("flow", flow).finish()
            }
        }
    }
}

impl Drop for ProofRequest {
    fn drop(&mut self) {
        if let ProofRequest::Password { auth_key } = self {
            auth_key.zeroize();
        }
    }
}

/// The assertion fields used by a sensitive-change proof. No user handle is
/// sent because proof/start has already selected a credential.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PasskeyProofAssertion {
    pub credential_id: String,
    pub authenticator_data: String,
    pub client_data_json: String,
    pub signature: String,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PasskeyProofStartRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_id: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PasskeyProofStartResponse {
    pub flow_id: String,
    pub challenge: String,
    pub allow_credentials: Vec<PasskeyAllowedCredential>,
    pub timeout: u64,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PasskeyProofRequest {
    pub proof: ProofRequest,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovery_auth_key: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountPasskeysStartResponse {
    pub flow_id: String,
    pub challenge: String,
    pub rp_id: String,
    pub pub_key_cred_params: Vec<i32>,
    pub authenticator_selection: PasskeySelection,
    pub exclude_credentials: Vec<PasskeyAllowedCredential>,
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountPasskeysStartRequest {
    pub proof: ProofRequest,
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountPasskeysFinishRequest {
    pub flow_id: String,
    pub credential_id: String,
    pub attestation_object: String,
    pub client_data_json: String,
    pub transports: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub passkey_wrapped_key: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountPasskeysFinishResponse {
    pub credential_id: String,
    pub prf: bool,
    pub created_at: i64,
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

    #[test]
    fn passkey_contracts_match_worker_json_names() {
        // This is the exact registration-options shape `cloud/src/passkeys.ts`
        // returns: `pub_key_cred_params` is a list of COSE integer IDs, not
        // WebAuthn `{type, alg}` objects.
        let start: PasskeySignupStartResponse = serde_json::from_value(serde_json::json!({
            "flow_id": "f",
            "challenge": "c",
            "rp_id": "sync.example",
            "user": {"id": "u", "name": "alice", "displayName": "alice"},
            "pub_key_cred_params": [-7, -8, -257],
            "authenticator_selection": {"residentKey": "required", "userVerification": "required"},
            "exclude_credentials": []
        }))
        .unwrap();
        assert_eq!(start.user.display_name, "alice");
        assert_eq!(start.authenticator_selection.resident_key, "required");
        assert_eq!(start.authenticator_selection.user_verification, "required");
        assert_eq!(start.pub_key_cred_params, vec![-7, -8, -257]);
        assert_eq!(
            Route::PasskeySignupFinish.http(),
            ("POST", "/v2/auth/passkey/signup/finish")
        );
    }

    #[test]
    fn passkey_request_secrets_are_redacted() {
        let proof = ProofRequest::Password {
            auth_key: "SECRET-password-proof".into(),
        };
        assert!(!format!("{proof:?}").contains("SECRET-password-proof"));
        let request = PasskeySignupFinishRequest {
            flow_id: "f".into(),
            username: "alice".into(),
            account_id: "a_0123456789abcdef0123456789abcdef".into(),
            credential_id: "credential".into(),
            attestation_object: "attestation".into(),
            client_data_json: "client-data".into(),
            transports: vec![],
            passkey_wrapped_key: None,
            recovery_auth_key: "SECRET-recovery-auth".into(),
            recovery_wrapped_key: "recovery-wrap".into(),
            key_id: "0123456789abcdef".into(),
            device: NewDevice {
                id: "d_0123456789abcdef0123456789abcdef".into(),
                sealed_name: None,
                proof: "SECRET-device-proof".into(),
            },
        };
        let printed = format!("{request:?}");
        assert!(!printed.contains("SECRET-recovery-auth"));
        assert!(!printed.contains("SECRET-device-proof"));
        assert!(printed.contains("recovery-wrap"));
    }

    #[test]
    fn passkey_signup_finish_body_and_session_response_match_the_worker() {
        let request = PasskeySignupFinishRequest {
            flow_id: "flow".into(),
            username: "alice".into(),
            account_id: "a_0123456789abcdef0123456789abcdef".into(),
            credential_id: "credential".into(),
            attestation_object: "attestation".into(),
            client_data_json: "client-data".into(),
            transports: vec!["usb".into()],
            passkey_wrapped_key: Some("wrapped-passkey-key".into()),
            recovery_auth_key: "recovery-proof".into(),
            recovery_wrapped_key: "recovery-wrap".into(),
            key_id: "0123456789abcdef".into(),
            device: NewDevice {
                id: "d_0123456789abcdef0123456789abcdef".into(),
                sealed_name: Some("sealed-name".into()),
                proof: "device-proof".into(),
            },
        };
        let body = serde_json::to_value(request).unwrap();
        assert_eq!(body["flow_id"], "flow");
        assert_eq!(body["credential_id"], "credential");
        assert_eq!(body["passkey_wrapped_key"], "wrapped-passkey-key");
        assert_eq!(body["recovery_auth_key"], "recovery-proof");
        assert_eq!(body["device"]["proof"], "device-proof");

        let response: PasskeySessionResponse = serde_json::from_value(serde_json::json!({
            "kind": "session",
            "account_id": "a_0123456789abcdef0123456789abcdef",
            "username": "alice",
            "token": "kanso_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "key_id": "0123456789abcdef",
            "epoch": "0123456789abcdef",
            "has_password": false,
            "passkey_wrapped_key": null,
            "evicted_device": null
        }))
        .unwrap();
        assert_eq!(response.kind, PasskeySessionKind::Session);
        assert_eq!(response.account_id, "a_0123456789abcdef0123456789abcdef");
        assert_eq!(
            response.token.expose(),
            "kanso_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        );
        assert_eq!(response.passkey_wrapped_key, None);
        assert_eq!(response.evicted_device, None);
        assert!(!format!("{response:?}").contains("kanso_"));
    }

    #[test]
    fn optional_assertion_user_handle_is_omitted_when_absent() {
        let request = PasskeyFinishRequest {
            flow_id: "flow".into(),
            credential_id: "credential".into(),
            authenticator_data: "auth-data".into(),
            client_data_json: "client-data".into(),
            signature: "signature".into(),
            user_handle: None,
            device: LoginDevice {
                id: "d_0123456789abcdef0123456789abcdef".into(),
                proof: "device-proof".into(),
            },
        };
        let body = serde_json::to_value(request).unwrap();
        assert!(body.get("user_handle").is_none());
    }

    #[test]
    fn health_passkey_advertisement_is_optional_and_typed() {
        let old: Health = serde_json::from_value(serde_json::json!({
            "ok": true, "protocol": 2, "signup": "open"
        }))
        .unwrap();
        assert!(old.auth.is_none());

        let current: Health = serde_json::from_value(serde_json::json!({
            "ok": true,
            "protocol": 2,
            "signup": "open",
            "auth": {
                "password": true,
                "passkeys": {
                    "rp_id": "sync.example",
                    "origins": ["https://sync.example"],
                    "security_key": true
                },
                "providers": []
            }
        }))
        .unwrap();
        assert_eq!(
            current.auth.unwrap().passkeys.unwrap().rp_id,
            "sync.example"
        );
    }

    #[test]
    fn account_sign_in_methods_use_the_specified_top_level_shape() {
        let methods: AccountSignInMethodsResponse = serde_json::from_value(serde_json::json!({
            "username": "alice",
            "has_password": true,
            "identities": [{"provider": "apple", "created_at": 1}],
            "passkeys": [{
                "credential_id": "credential",
                "created_at": 2,
                "last_used_at": null,
                "prf": true,
                "transports": ["usb"]
            }],
            "recovery": true
        }))
        .unwrap();
        assert_eq!(methods.username, "alice");
        assert!(methods.has_password && methods.recovery);
        assert_eq!(methods.passkeys.len(), 1);
        assert!(methods.passkeys[0].prf);
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
secret_fields!(PasskeySignupFinishRequest {
    shown: [
        flow_id,
        username,
        account_id,
        credential_id,
        attestation_object,
        client_data_json,
        transports,
        passkey_wrapped_key,
        recovery_wrapped_key,
        key_id,
        device
    ],
    secret: [recovery_auth_key]
});
secret_fields!(PasskeyProofRequest {
    shown: [proof],
    secret: [recovery_auth_key]
});
secret_fields!(PasskeyProofAssertion {
    shown: [credential_id],
    secret: [authenticator_data, client_data_json, signature]
});
