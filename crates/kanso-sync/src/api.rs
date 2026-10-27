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
    WirePushResult, WireRemote, is_apns_token, is_fcm_token,
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

    /// Opens `session`'s account event socket at `path` (what health
    /// advertises) with this device's ID and proof ([`crate::events`]). A
    /// backend without one refuses with 404, which ends the event loop.
    fn events(
        &self,
        _session: &Session,
        _device: &crate::DeviceIdentity,
        _path: &str,
    ) -> Result<Box<dyn crate::events::EventSource>, crate::events::EventError> {
        Err(crate::events::EventError::Refused(ApiError::NotFound))
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
    /// `POST /v2/account/sign-in/password`: the first password.
    fn set_first_password(
        &self,
        _token: &SessionToken,
        _request: &SignInPasswordRequest,
    ) -> Result<PasswordResponse, ApiError> {
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

    // Push-token registration. A server without it answers 404, which is
    // also what these defaults give a backend that doesn't implement it.

    /// `PUT /v2/account/devices/<device_id>/push` (204): this device's
    /// notification token. The device must be the session's own.
    fn put_push(
        &self,
        _token: &SessionToken,
        _device_id: &str,
        _registration: &PushRegistration,
    ) -> Result<(), ApiError> {
        Err(ApiError::NotFound)
    }
    /// `DELETE /v2/account/devices/<device_id>/push` (2xx, idempotent):
    /// turns pushes off for the device.
    fn delete_push(&self, _token: &SessionToken, _device_id: &str) -> Result<(), ApiError> {
        Err(ApiError::NotFound)
    }

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

    // Device approval (docs/ACCOUNTS.md › Sign-in routes › Approvals). A
    // server without them answers 404, which is also what these defaults
    // give a backend that doesn't implement them.

    /// `POST /v2/account/sign-in/approvals`.
    fn create_approval(
        &self,
        _token: &SessionToken,
        _request: &ApprovalCreateRequest,
    ) -> Result<ApprovalCreated, ApiError> {
        Err(ApiError::NotFound)
    }
    /// `GET /v2/account/sign-in/approvals`: the account's pending requests
    /// other than the caller's own.
    fn approvals(&self, _token: &SessionToken) -> Result<ApprovalList, ApiError> {
        Err(ApiError::NotFound)
    }
    /// `GET /v2/account/sign-in/approvals/<id>`: the requester's poll.
    fn approval(&self, _token: &SessionToken, _id: &str) -> Result<ApprovalPoll, ApiError> {
        Err(ApiError::NotFound)
    }
    /// `POST /v2/account/sign-in/approvals/<id>/challenge`.
    fn challenge_approval(
        &self,
        _token: &SessionToken,
        _id: &str,
        _request: &ApprovalChallengeRequest,
    ) -> Result<(), ApiError> {
        Err(ApiError::NotFound)
    }
    /// `POST /v2/account/sign-in/approvals/<id>/name`.
    fn name_approval(
        &self,
        _token: &SessionToken,
        _id: &str,
        _request: &ApprovalNameRequest,
    ) -> Result<(), ApiError> {
        Err(ApiError::NotFound)
    }
    /// `POST /v2/account/sign-in/approvals/<id>/approve`.
    fn approve_approval(
        &self,
        _token: &SessionToken,
        _id: &str,
        _request: &ApprovalApproveRequest,
    ) -> Result<(), ApiError> {
        Err(ApiError::NotFound)
    }
    /// `DELETE /v2/account/sign-in/approvals/<id>` (204): cancel or decline.
    fn cancel_approval(&self, _token: &SessionToken, _id: &str) -> Result<(), ApiError> {
        Err(ApiError::NotFound)
    }
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
    SignInPassword,
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
    ApprovalCreate,
    ApprovalList,
    ApprovalPoll,
    ApprovalChallenge,
    ApprovalName,
    ApprovalApprove,
    ApprovalCancel,
    PutPush,
    DeletePush,
}

impl Route {
    /// Routes the shared conformance fixtures cover. The passkey routes
    /// ([`Route::PASSKEYS`]) are served by `MemoryServer` and the Worker and
    /// covered by `sign_in.json`.
    pub const ALL: [Route; 25] = [
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
        Route::ApprovalCreate,
        Route::ApprovalList,
        Route::ApprovalPoll,
        Route::ApprovalChallenge,
        Route::ApprovalName,
        Route::ApprovalApprove,
        Route::ApprovalCancel,
    ];

    /// The passkey, proof and first-password routes: the four public routes
    /// and the account's own, all verified by `MemoryServer` as the Worker
    /// verifies them.
    pub const PASSKEYS: [Route; 10] = [
        Route::PasskeyStart,
        Route::PasskeyFinish,
        Route::PasskeySignupStart,
        Route::PasskeySignupFinish,
        Route::AccountSignInMethods,
        Route::PasskeyProofStart,
        Route::AccountPasskeysStart,
        Route::AccountPasskeysFinish,
        Route::AccountPasskeyRemove,
        Route::SignInPassword,
    ];

    /// The push-token routes: served by `MemoryServer` and the Worker, and
    /// exercised by their own tests rather than the shared conformance cases.
    pub const PUSH_TOKENS: [Route; 2] = [Route::PutPush, Route::DeletePush];

    /// Provider routes; their own fixture and wire tests exercise them rather
    /// than the shared conformance cases.
    pub const PROVIDERS: [Route; 8] = [
        Route::ProviderStart,
        Route::ProviderPoll,
        Route::GoogleStart,
        Route::GoogleFinish,
        Route::SocialSignup,
        Route::SignInMethods,
        Route::IdentityStart,
        Route::IdentityUnlink,
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
            Route::SignInPassword => ("POST", "/v2/account/sign-in/password"),
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
            Route::ApprovalCreate => ("POST", "/v2/account/sign-in/approvals"),
            Route::ApprovalList => ("GET", "/v2/account/sign-in/approvals"),
            Route::ApprovalPoll => ("GET", "/v2/account/sign-in/approvals/<id>"),
            Route::ApprovalChallenge => ("POST", "/v2/account/sign-in/approvals/<id>/challenge"),
            Route::ApprovalName => ("POST", "/v2/account/sign-in/approvals/<id>/name"),
            Route::ApprovalApprove => ("POST", "/v2/account/sign-in/approvals/<id>/approve"),
            Route::ApprovalCancel => ("DELETE", "/v2/account/sign-in/approvals/<id>"),
            Route::PutPush => ("PUT", "/v2/account/devices/<device_id>/push"),
            Route::DeletePush => ("DELETE", "/v2/account/devices/<device_id>/push"),
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
/// `login-user`. The proof is zeroized when this value is dropped and never
/// printed.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoginDevice {
    pub id: String,
    pub proof: String,
}

/// `{"id", "sealed_name"}`: the device a sign-in evicted.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WireDevice {
    pub id: String,
    pub sealed_name: Option<String>,
}

impl fmt::Debug for WireDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WireDevice")
            .field("id", &self.id)
            .field(
                "sealed_name",
                &self.sealed_name.as_ref().map(|_| "[redacted]"),
            )
            .finish()
    }
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
    /// The live channels this server offers; absent on an older server,
    /// which means the app uses the GET fallbacks only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub events: Option<HealthEvents>,
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

/// `"events": {"account": "/v2/account/events"}`: the account event socket's
/// path.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthEvents {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
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
    /// The authenticator's AAGUID as 32 lowercase hex digits; absent from
    /// older servers and for an all-zero one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aaguid: Option<String>,
}

/// A fresh proof for a session-authenticated account change: the same
/// shape as [`crate::provider::ProofWire`], which it names.
pub type ProofRequest = crate::provider::ProofWire;

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
    #[serde(default)]
    pub rp_id: String,
    pub allow_credentials: Vec<PasskeyAllowedCredential>,
    #[serde(default)]
    pub user_verification: String,
    pub timeout: u64,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PasskeyProofRequest {
    #[serde(default, deserialize_with = "crate::provider::required_proof")]
    pub proof: ProofRequest,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovery_auth_key: Option<String>,
}
/// The registration options, as the sign-up start's. `user` may be left out
/// (the spec's "minus the username"): the client then uses the account's
/// user handle and its own username.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountPasskeysStartResponse {
    pub flow_id: String,
    pub challenge: String,
    pub rp_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<PasskeyUser>,
    pub pub_key_cred_params: Vec<i32>,
    pub authenticator_selection: PasskeySelection,
    pub exclude_credentials: Vec<PasskeyAllowedCredential>,
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountPasskeysStartRequest {
    #[serde(default, deserialize_with = "crate::provider::required_proof")]
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

/// `{"current_auth_key", "recovery_auth_key", "recovery_wrapped_key"}`, or
/// the same with a `proof` in place of `current_auth_key`, which is how a
/// passwordless account replaces its key. A `provider` proof also takes
/// `current_recovery_auth_key`.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryKeyRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_auth_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proof: Option<ProofRequest>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_recovery_auth_key: Option<String>,
    pub recovery_auth_key: String,
    pub recovery_wrapped_key: String,
}

/// `{}`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryKeyResponse {}

/// `{"auth_key", "username"}` for a password account, or `{"proof",
/// "username"}`; a `provider` proof also takes `current_recovery_auth_key`.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeleteRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proof: Option<ProofRequest>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_recovery_auth_key: Option<String>,
    pub username: String,
}

/// `POST /v2/account/sign-in/password`: the first password of an account
/// made with a passkey or a provider, the data key wrapped under its KEK, and
/// a new recovery key. A `provider` proof also takes
/// `current_recovery_auth_key`. 200 [`PasswordResponse`].
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignInPasswordRequest {
    #[serde(default, deserialize_with = "crate::provider::required_proof")]
    pub proof: ProofRequest,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_recovery_auth_key: Option<String>,
    pub kdf: WireKdf,
    pub salt: String,
    pub auth_key: String,
    pub wrapped_key: String,
    pub recovery_auth_key: String,
    pub recovery_wrapped_key: String,
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

/// The push service a token belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PushPlatform {
    Apns,
    Fcm,
}

/// Which APNs environment issued an APNs token; FCM has only `production`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PushEnvironment {
    Production,
    Development,
}

/// `PUT /v2/account/devices/<device_id>/push`: where the server wakes this
/// device. The token is an OS-issued opaque value, not a session secret, but
/// it is still kept out of `Debug` output.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PushRegistration {
    pub platform: PushPlatform,
    pub token: String,
    pub environment: PushEnvironment,
}

impl fmt::Debug for PushRegistration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PushRegistration")
            .field("platform", &self.platform)
            .field("token", &"<redacted>")
            .field("environment", &self.environment)
            .finish()
    }
}

impl PushRegistration {
    /// Whether the server would accept it: an APNs token of lowercase hex
    /// (even length, 32 to 200) in either environment, or an FCM token in
    /// `production`.
    pub fn is_valid(&self) -> bool {
        match self.platform {
            PushPlatform::Apns => is_apns_token(&self.token),
            PushPlatform::Fcm => {
                self.environment == PushEnvironment::Production && is_fcm_token(&self.token)
            }
        }
    }
}

/// `POST /v2/account/sign-in/approvals`: the requester's commitment to its
/// ephemeral key, which it reveals only after the challenge.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalCreateRequest {
    /// Caller-generated, stable ID used to make a lost response safe to retry.
    pub request_id: String,
    pub commit: String,
}

/// The new request: its ID (43 base64url characters), when it expires (ms),
/// and how many other active sessions the account has.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalCreated {
    pub id: String,
    pub expires_at: i64,
    pub other_devices: u64,
}

/// `GET /v2/account/sign-in/approvals` 200, oldest first.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalList {
    pub approvals: Vec<ApprovalRow>,
}

/// A pending request as the approver sees it. `epk` and `sealed_name`
/// appear once the requester has posted them, after a challenge.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalRow {
    pub id: String,
    pub device_id: String,
    pub commit: String,
    #[serde(default)]
    pub epk: Option<String>,
    pub created_at: i64,
    pub expires_at: i64,
    #[serde(default)]
    pub approver_epk: Option<String>,
    #[serde(default)]
    pub approver_device_id: Option<String>,
    #[serde(default)]
    pub sealed_name: Option<String>,
    pub status: String,
}

/// `GET /v2/account/sign-in/approvals/<id>` 200: `status` is `waiting`,
/// `challenged`, `approved`, `cancelled` or `expired`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalPoll {
    pub status: String,
    #[serde(default)]
    pub expires_at: Option<i64>,
    #[serde(default)]
    pub approver_epk: Option<String>,
    #[serde(default)]
    pub approver_device_id: Option<String>,
    #[serde(default)]
    pub sealed_name: Option<String>,
    #[serde(default)]
    pub wrapped_key: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalChallengeRequest {
    pub epk: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalNameRequest {
    pub epk: String,
    pub sealed_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalApproveRequest {
    pub wrapped_key: String,
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
                    current_auth_key: Some("SECRET-current".into()),
                    proof: None,
                    current_recovery_auth_key: Some("SECRET-old-recovery".into()),
                    recovery_auth_key: "SECRET-recovery".into(),
                    recovery_wrapped_key: "rwrap-shown".into(),
                }
            ),
            format!(
                "{:?}",
                DeleteRequest {
                    auth_key: Some("SECRET-auth".into()),
                    proof: None,
                    current_recovery_auth_key: None,
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
    fn proofs_have_the_three_specified_shapes() {
        let password = ProofRequest::password("A".repeat(43));
        assert_eq!(
            serde_json::to_value(&password).unwrap(),
            serde_json::json!({"kind": "password", "auth_key": "A".repeat(43)})
        );
        let passkey = ProofRequest::passkey(
            "F".repeat(43),
            PasskeyProofAssertion {
                credential_id: "cred".into(),
                authenticator_data: "ad".into(),
                client_data_json: "cd".into(),
                signature: "sig".into(),
            },
        );
        assert_eq!(
            serde_json::to_value(&passkey).unwrap(),
            serde_json::json!({"kind": "passkey", "flow": "F".repeat(43), "assertion": {
                "credential_id": "cred", "authenticator_data": "ad",
                "client_data_json": "cd", "signature": "sig"}})
        );
        let provider = ProofRequest::provider("G".repeat(43));
        assert_eq!(
            serde_json::to_value(&provider).unwrap(),
            serde_json::json!({"kind": "provider", "flow": "G".repeat(43)})
        );
        for proof in [password, passkey, provider] {
            let back: ProofRequest =
                serde_json::from_value(serde_json::to_value(&proof).unwrap()).unwrap();
            assert_eq!(back, proof);
            let printed = format!("{proof:?}");
            assert!(
                !printed.contains(&"A".repeat(43)) && !printed.contains("\"sig\""),
                "{printed}"
            );
        }
    }

    #[test]
    fn proof_bodies_leave_out_what_they_dont_carry() {
        let request = RecoveryKeyRequest {
            current_auth_key: None,
            proof: Some(ProofRequest::provider("G".repeat(43))),
            current_recovery_auth_key: Some("R".repeat(43)),
            recovery_auth_key: "N".repeat(43),
            recovery_wrapped_key: "W".repeat(98),
        };
        let body = serde_json::to_value(&request).unwrap();
        assert!(body.get("current_auth_key").is_none());
        assert_eq!(body["proof"]["kind"], "provider");
        assert_eq!(body["current_recovery_auth_key"], "R".repeat(43));
        let old: RecoveryKeyRequest = serde_json::from_value(serde_json::json!({
            "current_auth_key": "C", "recovery_auth_key": "N", "recovery_wrapped_key": "W"
        }))
        .unwrap();
        assert_eq!(old.current_auth_key.as_deref(), Some("C"));
        assert!(old.proof.is_none());

        let delete = DeleteRequest {
            auth_key: None,
            proof: Some(ProofRequest::password("A".repeat(43))),
            current_recovery_auth_key: None,
            username: "alice".into(),
        };
        let body = serde_json::to_value(&delete).unwrap();
        assert_eq!(
            body,
            serde_json::json!({"proof": {"kind": "password", "auth_key": "A".repeat(43)}, "username": "alice"})
        );
        assert!(!format!("{delete:?}").contains(&"A".repeat(43)));

        let first = SignInPasswordRequest {
            proof: ProofRequest::provider("G".repeat(43)),
            current_recovery_auth_key: Some("SECRET-recovery".into()),
            kdf: WireKdf::from(KdfParams::DEFAULT),
            salt: "s".repeat(22),
            auth_key: "SECRET-auth".into(),
            wrapped_key: "w".repeat(98),
            recovery_auth_key: "SECRET-new".into(),
            recovery_wrapped_key: "r".repeat(98),
        };
        let printed = format!("{first:?}");
        assert!(!printed.contains("SECRET"), "{printed}");
        let body = serde_json::to_value(&first).unwrap();
        for key in [
            "proof",
            "current_recovery_auth_key",
            "kdf",
            "salt",
            "auth_key",
            "wrapped_key",
            "recovery_auth_key",
            "recovery_wrapped_key",
        ] {
            assert!(body.get(key).is_some(), "{key}");
        }
        assert_eq!(
            Route::SignInPassword.http(),
            ("POST", "/v2/account/sign-in/password")
        );
    }

    #[test]
    fn route_names_are_the_fixture_names() {
        for route in Route::ALL
            .into_iter()
            .chain(Route::PASSKEYS)
            .chain(Route::PUSH_TOKENS)
            .chain(Route::PROVIDERS)
        {
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
    shown: [proof, recovery_wrapped_key],
    secret: [
        current_auth_key,
        current_recovery_auth_key,
        recovery_auth_key
    ]
});
secret_fields!(DeleteRequest {
    shown: [username, proof],
    secret: [auth_key, current_recovery_auth_key]
});
secret_fields!(SignInPasswordRequest {
    shown: [proof, kdf, salt, wrapped_key, recovery_wrapped_key],
    secret: [current_recovery_auth_key, auth_key, recovery_auth_key]
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
