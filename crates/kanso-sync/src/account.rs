//! The account functions the app calls (docs/ACCOUNTS.md › Public API the app
//! calls, › Account flows). Frozen by step 1; the bodies are step 2a.
//!
//! These functions talk to the server only. Saving the Keychain items,
//! binding the library and writing the config are the app's flow, in the
//! order docs/ACCOUNTS.md › Library binding › Order and cancellation gives.

use std::fmt;
use std::sync::{Mutex, PoisonError};

use serde::{Deserialize, Serialize};

use crate::api::{
    Api, Backend, DeleteRequest, DeviceNameRequest, LoginDevice, LoginRequest, NewDevice,
    PasswordRequest, PreloginRequest, PushRegistration, RecoverFinishRequest, RecoverStartRequest,
    RecoveryKeyRequest, SignupRequest,
};
use crate::approvals::LockedSession;
use crate::crypto::{
    ContentKeys, CryptoError, DataKey, KdfParams, Opened, Password, PasswordError, RecoveryKey,
    RecoveryKeyError, WrapPurpose, b64, hex, key_id, password_keys, random, recovery_keys, unb64,
    unwrap, wrap,
};
use crate::keychain::{self, PendingLogout, PendingRecoveryKey, SecretError, SecretStore};
use crate::provider::{
    AccountMethods, GoogleFinishRequest, GoogleStartRequest, IdentityIntent, IdentityStartRequest,
    LinkOutcome, Loopback, MethodsReply, Provider, ProviderPollRequest, ProviderSignIn,
    ProviderStart, ProviderStartRequest, ProviderStep, ServerAuth, SocialSignupRequest,
    UnlinkRequest,
};
use crate::username::{UsernameError, check_new_username, normalize_username};
use crate::wire::{
    ApiError, PROTOCOL_VERSION, QuotaKind, SessionToken, SignOutReason, SignupClosedReason,
    is_account_id, is_token,
};
use crate::{DeviceIdentity, EndpointError};

/// The owner's Worker, compiled in as the default server. Accounts are bound
/// to their server's origin (security re-check 14); this name derives as
/// the server's first one, [`crate::OWNER_ORIGIN`], so accounts made there
/// keep working under it.
pub const DEFAULT_SERVER: &str = "https://sync.kansoapp.app";

/// What a running account task is doing, for the button's spinner text.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Step {
    /// "Stopping sync…": the app cancels and joins the running sync before
    /// any re-bind or sign-out. The account functions never report it.
    StoppingSync,
    /// "Securing your keys…": Argon2id and wrapping.
    DerivingKeys,
    /// "Contacting the server…".
    Contacting,
    /// "Saving…": Keychain, binding and config (the app's part of a flow).
    Saving,
}

/// A signed-in library. `Send + Sync`, shared as `Arc<Session>`. The token
/// and data key are private and zeroized on drop; `Debug` shows neither.
/// `server` and `origin` come from the Keychain item, never from the config.
pub struct Session {
    /// The URL the client connects to.
    pub server: String,
    /// `url::Url::parse(server)?.origin().ascii_serialization()`; with
    /// `account_id`, what "the same account" means (security re-check 3).
    pub origin: String,
    pub account_id: String,
    /// Normalized.
    pub username: String,
    /// `d_` + 32 hex, this Mac's ID in this account.
    pub device_id: String,
    token: SessionToken,
    data_key: DataKey,
    /// The library whose Keychain items hold this session (sign-out deletes
    /// them and queues its logout there).
    library_id: Option<String>,
    /// The device ID and proof the session was created with, when they
    /// aren't this library's own: a passkey or provider sign-in binds the
    /// session to its flow's device. Session routes that check the device
    /// (the event socket, a link's or proof's start) need that proof. Kept
    /// in the `sync-account` Keychain item, zeroized on drop, never printed.
    login_device: Option<LoginDevice>,
}

impl Session {
    /// A session from its parts (the account functions, and the Keychain
    /// item on load).
    pub(crate) fn new(
        server: String,
        origin: String,
        account_id: String,
        username: String,
        device_id: String,
        token: SessionToken,
        data_key: DataKey,
    ) -> Session {
        Session {
            server,
            origin,
            account_id,
            username,
            device_id,
            token,
            data_key,
            library_id: None,
            login_device: None,
        }
    }

    /// Records the device this session was created with. Ignored unless it
    /// is the session's own device ID with a 32-byte proof.
    pub(crate) fn set_login_device(&mut self, device: LoginDevice) {
        if device.id == self.device_id && unb64(&device.proof).is_some_and(|p| p.len() == 32) {
            self.login_device = Some(device);
        }
    }

    /// The recorded device, for the Keychain item.
    pub(crate) fn stored_login_device(&self) -> Option<&LoginDevice> {
        self.login_device.as_ref()
    }

    /// The device ID and proof this session's device-checked routes send:
    /// the ones it was created with (a passkey or provider sign-in), or this
    /// library's own (`identity`) for a session it made itself.
    pub fn login_device(&self, identity: &crate::binding::DeviceIdentity) -> LoginDevice {
        self.login_device
            .clone()
            .unwrap_or_else(|| LoginDevice::for_identity(identity, &self.origin, &self.account_id))
    }

    pub(crate) fn token(&self) -> &SessionToken {
        &self.token
    }

    /// A label for this login that can be kept in a library's plain
    /// settings: the device ID and a one-way digest of the token, never the
    /// token. A new sign-in on the same device gets a new label, so it tells
    /// "registered for this login" from "registered for an earlier one".
    pub fn login_label(&self) -> String {
        let digest = crate::crypto::sha256_hex(
            format!("kanso-login-label\0{}", self.token.expose()).as_bytes(),
        );
        format!("{}:{}", self.device_id, &digest[..32])
    }

    pub(crate) fn data_key(&self) -> &DataKey {
        &self.data_key
    }

    pub(crate) fn set_library_id(&mut self, library_id: &str) {
        self.library_id = Some(library_id.to_owned());
    }

    /// The data key's ID, as the server holds it (docs/ACCOUNTS.md › Keys).
    /// Not secret: used to check a wrap and by the approval flow's tests.
    pub fn key_id(&self) -> String {
        crate::crypto::key_id(&self.data_key)
    }

    /// The keys shared notes start from, derived from this session's data
    /// key (docs/ACCOUNTS.md › Collaboration).
    pub fn collab_roots(&self) -> crate::crypto::CollabRoots {
        crate::crypto::CollabRoots::derive(&self.data_key)
    }

    /// The `Authorization` header of this session's requests, for routes
    /// other crates send (shared notes). Zeroized on drop.
    pub fn authorization(&self) -> zeroize::Zeroizing<String> {
        zeroize::Zeroizing::new(format!("Bearer {}", self.token.expose()))
    }

    /// The library this session was created for or loaded from: its
    /// Keychain items are `sync-account:<library_id>` and friends.
    pub fn library_id(&self) -> Option<&str> {
        self.library_id.as_deref()
    }
}

impl fmt::Debug for Session {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Session")
            .field("server", &self.server)
            .field("origin", &self.origin)
            .field("account_id", &self.account_id)
            .field("username", &self.username)
            .field("device_id", &self.device_id)
            .field("library_id", &self.library_id)
            .finish_non_exhaustive()
    }
}

/// An active device of the account, with its name opened ("A Mac" in the UI
/// while `name` is `None`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeviceInfo {
    pub id: String,
    pub name: Option<String>,
    pub created_at: i64,
    pub last_seen_at: i64,
    /// This Mac.
    pub current: bool,
}

/// The account's effective quotas.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Quota {
    pub docs: u64,
    pub doc_bytes: u64,
    pub files: u64,
    pub file_bytes: u64,
    pub writes_per_day: u64,
}

/// Storage use, from `GET /v2/account`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Usage {
    pub docs: u64,
    pub doc_bytes: u64,
    pub files: u64,
    pub file_bytes: u64,
    pub writes_today: u64,
    pub quota: Quota,
}

/// Whether a server takes new accounts (`GET /v2/health`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignupState {
    Open,
    /// Switched off, or reserved for one name.
    Closed,
    /// `max_accounts` reached.
    Full,
}

/// A new account. Save `recovery_key` in the Keychain at once, then bind.
#[derive(Debug)]
pub struct SignedUp {
    pub session: Session,
    pub recovery_key: RecoveryKey,
}

/// A signed-in device; `evicted` is the device signed out to make room.
#[derive(Debug)]
pub struct SignedIn {
    pub session: Session,
    pub evicted: Option<DeviceInfo>,
}

/// A recovered account and its new recovery key (the used one stops working).
#[derive(Debug)]
pub struct Recovered {
    pub session: Session,
    pub recovery_key: RecoveryKey,
}

/// A changed password and the new recovery key set with it (the old key
/// stops working).
#[derive(Debug)]
pub struct PasswordChanged {
    /// How many other devices were signed out.
    pub signed_out_devices: usize,
    pub recovery_key: RecoveryKey,
}

/// Why an account function failed: one variant per server error code, plus
/// the client's own failures (docs/ACCOUNTS.md › Public API the app calls).
#[derive(Debug, thiserror::Error)]
pub enum AccountError {
    #[error("the server refused the request as malformed")]
    BadRequest,
    #[error("invalid username")]
    InvalidUsername,
    #[error("reserved username")]
    ReservedUsername,
    #[error("the server doesn't support these key parameters")]
    UnsupportedKdf,
    #[error("the server refused these key parameters")]
    WeakKdf,
    #[error("the typed username isn't this account's")]
    UsernameMismatch,
    #[error("a file didn't match its checksum")]
    ChecksumMismatch,
    #[error("this server doesn't offer that sign-in method")]
    UnsupportedMethod,
    #[error("the sign-in flow expired; try again")]
    FlowExpired,
    /// 401 `unauthorized`: this Mac's session ended, for `reason`.
    #[error("signed out ({})", reason.as_str())]
    Unauthorized { reason: SignOutReason },
    /// 401: wrong password or recovery key, or an unknown username.
    #[error("invalid credentials")]
    InvalidCredentials,
    /// 401 `reauth_required`: the change needs a fresh password or passkey
    /// proof, not just this session.
    #[error("this change needs your password or a passkey")]
    ReauthRequired,
    /// 400 `nonce_reused`: the token or flow was already used.
    #[error("this sign-in was already used")]
    NonceReused,
    #[error("the server refused a browser request")]
    ForbiddenOrigin,
    #[error("the account is turned off")]
    AccountDisabled,
    #[error("sign-ups are closed ({})", reason.as_str())]
    SignupClosed { reason: SignupClosedReason },
    /// 403 `provider_disabled`: the server doesn't offer this provider.
    #[error("that provider isn't available on this server")]
    ProviderDisabled,
    /// 403 `provider_denied`: the sign-in was declined at the provider.
    #[error("the sign-in was declined")]
    ProviderDenied,
    /// 503 `provider_unavailable`: the provider's endpoint failed.
    #[error("the provider is unavailable; try again")]
    ProviderUnavailable,
    #[error("not found")]
    NotFound,
    #[error("method not allowed")]
    MethodNotAllowed,
    #[error("username taken")]
    UsernameTaken,
    /// The client generates a new account ID and retries; it never surfaces.
    #[error("account ID taken")]
    AccountIdTaken,
    /// 409 `identity_taken`: another account already owns that provider
    /// identity. It never says which account.
    #[error("that provider account is already linked to another Kanso account")]
    IdentityTaken,
    /// 409 `last_method`: removing this would leave no way to sign in.
    #[error("this is the last way to sign in; use your recovery key")]
    LastMethod,
    /// 409 `already_has_password`.
    #[error("this account already has a password")]
    AlreadyHasPassword,
    /// 409 `no_password`: `account/password` on an account without one; it
    /// sets a first password with [`crate::set_password`] instead.
    #[error("this account has no password")]
    NoPassword,
    #[error("that passkey is already registered")]
    CredentialExists,
    /// 403 `self_approval`: "A device can't approve itself."
    #[error("a device can't approve itself")]
    SelfApproval,
    /// 409 `approval_used`: "That request was already answered. Ask again."
    #[error("that approval request was already answered")]
    ApprovalUsed,
    /// 409 `approval_request_id_conflict`: an ID was reused with different
    /// account, device or commitment data.
    #[error("that approval request ID is already in use")]
    ApprovalRequestIdConflict,
    /// 409 `approval_expired`: "That approval request expired. Ask again."
    #[error("that approval request expired")]
    ApprovalExpired,
    /// The requester's key doesn't match its commitment, or its sealed name
    /// doesn't open: "This request changed after it was made. Don't
    /// approve." Nothing was sent.
    #[error("the approval request changed after it was made")]
    ApprovalChanged,
    #[error("too large")]
    TooLarge,
    #[error("quota exceeded ({})", quota.as_str())]
    QuotaExceeded {
        quota: QuotaKind,
        used: u64,
        limit: u64,
    },
    #[error("unsupported media type")]
    UnsupportedMediaType,
    /// "Too many attempts. Try again in *N* minutes."
    #[error("rate limited for {retry_after} s")]
    RateLimited { retry_after: u64 },
    #[error("the server failed")]
    Internal,
    #[error("the server is misconfigured")]
    Misconfigured,
    /// No HTTP answer: "Can't reach the server."
    #[error("can't reach the server: {0}")]
    Transport(String),
    /// An answer outside the protocol, or an unknown error code.
    #[error("unexpected answer from the server: {0}")]
    Protocol(String),
    /// 404 from `/v2/health`: not a v2 Kanso server.
    #[error("that server doesn't run this version of Kanso sync")]
    ServerOutdated,
    /// Prelogin parameters outside the backend's floor or the ceilings; no
    /// key was derived or sent.
    #[error("the server asked for unsafe key parameters")]
    UnsafeKdf,
    /// A wrap didn't open, or the key ID differs. Nothing was saved, and a
    /// new session was logged out.
    #[error("the server's keys don't match")]
    KeyMismatch,
    /// The server URL failed [`crate::check_endpoint`].
    #[error("server URL: {0}")]
    Endpoint(#[from] EndpointError),
    #[error("password: {0}")]
    Password(#[from] PasswordError),
    #[error("username: {0}")]
    Username(#[from] UsernameError),
    #[error("recovery key: {0}")]
    RecoveryKey(#[from] RecoveryKeyError),
    /// The Keychain refused. Before a request that sets a new recovery key
    /// this means nothing was sent.
    #[error("{0}")]
    Keychain(#[from] SecretError),
    /// The library's Keychain still holds another account's recovery key
    /// that nobody confirmed saving; a new one would replace it. Nothing was
    /// sent.
    #[error("the recovery key for {username} isn't saved yet")]
    UnsavedRecoveryKey { username: String },
}

/// Why a call that sets a new recovery key (sign-up, recovery, a password
/// change, "New recovery key") failed, with that key whenever the server
/// may hold it: show it, and never drop it.
#[derive(Debug, thiserror::Error)]
#[error("{error}")]
pub struct RotationError {
    pub error: AccountError,
    /// The unsaved key left in `sync-recovery-key:<library>`: this call's,
    /// when its request may have reached the server (the answer was lost,
    /// garbled or a 5xx; `unconfirmed` is set), or one an earlier call left
    /// there. `None` when the Keychain holds none.
    pub recovery_key: Option<Box<PendingRecoveryKey>>,
}

impl From<AccountError> for RotationError {
    fn from(error: AccountError) -> RotationError {
        RotationError {
            error,
            recovery_key: None,
        }
    }
}

impl From<ApiError> for AccountError {
    fn from(err: ApiError) -> AccountError {
        match err {
            ApiError::BadRequest => AccountError::BadRequest,
            ApiError::InvalidUsername => AccountError::InvalidUsername,
            ApiError::ReservedUsername => AccountError::ReservedUsername,
            ApiError::UnsupportedKdf => AccountError::UnsupportedKdf,
            ApiError::WeakKdf => AccountError::WeakKdf,
            ApiError::UsernameMismatch => AccountError::UsernameMismatch,
            ApiError::ChecksumMismatch => AccountError::ChecksumMismatch,
            ApiError::UnsupportedMethod => AccountError::UnsupportedMethod,
            ApiError::FlowExpired { .. } => AccountError::FlowExpired,
            ApiError::Unauthorized { reason } => AccountError::Unauthorized { reason },
            ApiError::InvalidCredentials => AccountError::InvalidCredentials,
            ApiError::ReauthRequired => AccountError::ReauthRequired,
            ApiError::NonceReused => AccountError::NonceReused,
            ApiError::ForbiddenOrigin => AccountError::ForbiddenOrigin,
            ApiError::AccountDisabled => AccountError::AccountDisabled,
            ApiError::SignupClosed { reason } => AccountError::SignupClosed { reason },
            ApiError::ProviderDisabled => AccountError::ProviderDisabled,
            ApiError::ProviderDenied => AccountError::ProviderDenied,
            ApiError::ProviderUnavailable => AccountError::ProviderUnavailable,
            ApiError::NotFound => AccountError::NotFound,
            ApiError::MethodNotAllowed => AccountError::MethodNotAllowed,
            ApiError::UsernameTaken => AccountError::UsernameTaken,
            ApiError::AccountIdTaken => AccountError::AccountIdTaken,
            ApiError::IdentityTaken => AccountError::IdentityTaken,
            ApiError::LastMethod => AccountError::LastMethod,
            ApiError::AlreadyHasPassword => AccountError::AlreadyHasPassword,
            // The app's words match `AccountError` exhaustively; until it
            // has a variant for this race, it reads as an unexpected answer.
            ApiError::NoPassword => AccountError::NoPassword,
            ApiError::CredentialExists => AccountError::CredentialExists,
            ApiError::SelfApproval => AccountError::SelfApproval,
            ApiError::ApprovalUsed => AccountError::ApprovalUsed,
            ApiError::ApprovalRequestIdConflict => AccountError::ApprovalRequestIdConflict,
            ApiError::ApprovalExpired => AccountError::ApprovalExpired,
            ApiError::TooLarge => AccountError::TooLarge,
            ApiError::QuotaExceeded { quota, used, limit } => {
                AccountError::QuotaExceeded { quota, used, limit }
            }
            ApiError::UnsupportedMediaType => AccountError::UnsupportedMediaType,
            ApiError::RateLimited { retry_after, .. } => AccountError::RateLimited { retry_after },
            ApiError::Internal => AccountError::Internal,
            ApiError::Misconfigured => AccountError::Misconfigured,
            ApiError::Transport(message) => AccountError::Transport(message),
            ApiError::Protocol(message) => AccountError::Protocol(message),
            ApiError::Unrecognized { status, code } => {
                AccountError::Protocol(format!("{status} {code}"))
            }
        }
    }
}

impl From<CryptoError> for AccountError {
    fn from(err: CryptoError) -> AccountError {
        match err {
            CryptoError::KeyMismatch => AccountError::KeyMismatch,
            CryptoError::Kdf(message) => {
                AccountError::Protocol(format!("key derivation: {message}"))
            }
        }
    }
}

/// How often a request is sent in all when its answer is lost (a transport
/// error). Only routes whose repeat the server answers idempotently use it
/// (docs/ACCOUNTS.md › Routes, "Retrying").
const ATTEMPTS: usize = 3;

/// Sends the identical request again after a transport error, pausing as
/// the backend says first (about 1 s, then 4 s).
pub(crate) fn retrying<T>(
    b: &dyn Backend,
    send: impl FnMut() -> Result<T, ApiError>,
) -> Result<T, ApiError> {
    retrying_noted(b, send).result
}

/// What a request sent up to [`ATTEMPTS`] times came to.
pub(crate) struct Attempts<T> {
    /// The last attempt's result.
    pub(crate) result: Result<T, ApiError>,
    /// Whether any attempt may have been carried out by the server: its
    /// answer was lost, garbled or a 5xx ([`ApiError::may_have_applied`]).
    /// A later attempt that never left this Mac, or that the server refused
    /// (a 429), doesn't undo an earlier one that arrived (review-verify 1).
    pub(crate) reached: bool,
}

/// [`retrying`], noting whether any attempt may have reached the server.
pub(crate) fn retrying_noted<T>(
    b: &dyn Backend,
    mut send: impl FnMut() -> Result<T, ApiError>,
) -> Attempts<T> {
    let mut attempt = 1;
    let mut reached = false;
    loop {
        let result = send();
        if let Err(err) = &result {
            reached |= err.may_have_applied();
        }
        match result {
            Err(ApiError::Transport(_)) if attempt < ATTEMPTS => {
                std::thread::sleep(b.retry_pause(attempt));
                attempt += 1;
            }
            result => return Attempts { result, reached },
        }
    }
}

/// A new recovery key on its way to the server (sec-client 1). It is chosen
/// before the request is built and saved in `sync-recovery-key:<library>`,
/// marked unconfirmed, before the request is sent; nothing is sent when the
/// Keychain refuses. A key the server may hold is never dropped:
///
/// - The server says yes: the item is marked confirmed.
/// - Every attempt was refused (a 4xx) or never left this Mac: the item
///   goes back to what it was.
/// - Any attempt's answer was lost, garbled or a 5xx: the item stays, and
///   the error carries the key so the app shows it. A later attempt that
///   was refused or never sent doesn't change that: the earlier one may
///   have been carried out.
///
/// While the Keychain holds an unsaved key for the same account, a new
/// rotation sets that same key again (unless it is the key a recovery
/// proves), so a retried or repeated change can never leave the server
/// with a key the Mac no longer has.
pub(crate) struct Rotation<'a> {
    secrets: &'a dyn SecretStore,
    library_id: String,
    /// The item before this rotation.
    before: Option<PendingRecoveryKey>,
    /// The item this rotation writes.
    item: PendingRecoveryKey,
    /// Whether the item was written yet.
    written: bool,
}

impl<'a> Rotation<'a> {
    /// Chooses the key, reading the Keychain; writes nothing yet. `used` is
    /// the key a recovery proves, which has to change.
    #[allow(clippy::too_many_arguments, reason = "the item's fields")]
    pub(crate) fn choose(
        secrets: &'a dyn SecretStore,
        library_id: &str,
        server: &str,
        origin: &str,
        account_id: &str,
        username: &str,
        used: Option<&RecoveryKey>,
    ) -> Result<Rotation<'a>, RotationError> {
        let before =
            keychain::load_any_recovery_key(secrets, library_id).map_err(AccountError::Keychain)?;
        // Another account's key nobody saved yet stays, and comes back with
        // the error to be shown again. Usernames are unique on a server; a
        // sign-up's account ID changes between tries.
        if let Some(other) = &before
            && (other.origin != origin || other.username != username)
        {
            let username = other.username.clone();
            return Err(RotationError {
                error: AccountError::UnsavedRecoveryKey { username },
                recovery_key: before.map(Box::new),
            });
        }
        let kept = before
            .as_ref()
            .map(|kept| kept.recovery_key.clone())
            .filter(|key| Some(key) != used);
        Ok(Rotation {
            secrets,
            library_id: library_id.to_owned(),
            before,
            item: PendingRecoveryKey {
                origin: origin.to_owned(),
                account_id: account_id.to_owned(),
                username: username.to_owned(),
                server: server.to_owned(),
                recovery_key: kept.unwrap_or_else(RecoveryKey::generate),
                unconfirmed: true,
            },
            written: false,
        })
    }

    pub(crate) fn key(&self) -> &RecoveryKey {
        &self.item.recovery_key
    }

    /// Saves the key, unconfirmed, for `account_id`: right before sending.
    pub(crate) fn keep(&mut self, account_id: &str) -> Result<(), SecretError> {
        self.item.account_id = account_id.to_owned();
        keychain::save_recovery_key(self.secrets, &self.library_id, &self.item)?;
        self.written = true;
        Ok(())
    }

    /// The server took it. Marking it confirmed may fail: the key stays in
    /// the Keychain, unconfirmed, and the app shows it either way.
    pub(crate) fn accepted(mut self) -> RecoveryKey {
        self.item.unconfirmed = false;
        keychain::save_recovery_key(self.secrets, &self.library_id, &self.item).ok();
        self.item.recovery_key
    }

    /// The request failed with `error`; `maybe_applied` says whether the
    /// server may have carried it out.
    pub(crate) fn failed(self, error: AccountError, maybe_applied: bool) -> RotationError {
        if maybe_applied {
            return RotationError {
                error,
                recovery_key: Some(Box::new(self.item)),
            };
        }
        if self.written {
            let _ = match &self.before {
                Some(before) => keychain::save_recovery_key(self.secrets, &self.library_id, before),
                None => keychain::delete_recovery_key(self.secrets, &self.library_id),
            };
        }
        RotationError {
            error,
            recovery_key: self.before.map(Box::new),
        }
    }

    /// The request got `error` back instead of an answer; `reached` says
    /// whether an earlier attempt may have been carried out.
    pub(crate) fn failed_api(self, error: ApiError, reached: bool) -> RotationError {
        let maybe_applied = reached || error.may_have_applied();
        self.failed(error.into(), maybe_applied)
    }
}

/// The checked server URL and the origin the derivations bind.
pub(crate) fn checked_server(server: &str) -> Result<(String, String), AccountError> {
    let server = crate::check_endpoint(server)?;
    let origin = crate::origin(&server)?;
    Ok((server, origin))
}

/// Refuses a password equal to the (normalized) username, ignoring case.
pub(crate) fn check_not_username(password: &Password, username: &str) -> Result<(), AccountError> {
    if password.is_username(username) {
        return Err(PasswordError::SameAsUsername.into());
    }
    Ok(())
}

/// The parameters new keys use: the backend's default, which must itself
/// lie within the backend's floor and the ceilings.
pub(crate) fn default_params(b: &dyn Backend) -> Result<KdfParams, AccountError> {
    let params = b.kdf_default();
    if params.is_within(&b.kdf_floor()) {
        Ok(params)
    } else {
        Err(AccountError::UnsafeKdf)
    }
}

/// A prelogin answer, checked.
struct Prelogin {
    params: Option<KdfParams>,
    salt: [u8; 16],
    account_id: String,
}

impl Prelogin {
    /// The account's parameters, refused unless within the backend's floor
    /// and the ceilings, before any key is derived or sent.
    fn params(&self, b: &dyn Backend) -> Result<KdfParams, AccountError> {
        self.params
            .filter(|params| params.is_within(&b.kdf_floor()))
            .ok_or(AccountError::UnsafeKdf)
    }
}

fn prelogin(b: &dyn Backend, api: &dyn Api, username: &str) -> Result<Prelogin, AccountError> {
    let answer = retrying(b, || {
        api.prelogin(&PreloginRequest {
            username: username.to_owned(),
        })
    })?;
    let salt = unb64(&answer.salt)
        .and_then(|salt| <[u8; 16]>::try_from(salt.as_slice()).ok())
        .ok_or_else(|| AccountError::Protocol("prelogin: the salt isn't 16 bytes".into()))?;
    if !is_account_id(&answer.account_id) {
        return Err(AccountError::Protocol(
            "prelogin: a malformed account ID".into(),
        ));
    }
    Ok(Prelogin {
        params: answer.kdf.params(),
        salt,
        account_id: answer.account_id,
    })
}

/// `a_` and 32 random lowercase hex digits.
pub(crate) fn new_account_id() -> String {
    format!("a_{}", hex(&random::<16>()))
}

/// The value of a sealed name, or `None` when it doesn't open.
pub(crate) fn opened_name(
    keys: &ContentKeys,
    account_id: &str,
    device_id: &str,
    sealed: Option<&str>,
) -> Option<String> {
    match keys.open_name(account_id, device_id, sealed?) {
        Opened::Opened(name) => Some(name),
        Opened::Undecryptable | Opened::TooNew => None,
    }
}

/// This device as a sign-up or recovery sends it: its ID, sealed name and
/// proof for the account.
pub(crate) fn new_device(
    device: &DeviceIdentity,
    keys: &ContentKeys,
    origin: &str,
    account_id: &str,
) -> NewDevice {
    let id = device.device_id(account_id);
    NewDevice {
        sealed_name: Some(keys.seal_name(account_id, &id, &device.name)),
        proof: device.device_proof(origin, account_id).to_base64url(),
        id,
    }
}

/// A provider sign-up keeps the identity bound to the provider flow. Its ID
/// and proof must exactly match the values sent when that flow was started;
/// only the device name is sealed with this account attempt's content keys.
fn provider_signup_device(
    identity: &DeviceIdentity,
    login_device: &LoginDevice,
    keys: &ContentKeys,
    account_id: &str,
) -> NewDevice {
    NewDevice {
        id: login_device.id.clone(),
        sealed_name: Some(keys.seal_name(account_id, &login_device.id, &identity.name)),
        proof: login_device.proof.clone(),
    }
}

/// Checks a session answer (sign-up, recovery): its account and username,
/// and the token's shape.
fn check_answer(
    account_id: &str,
    username: &str,
    answer: (&str, &str, &SessionToken),
) -> Result<(), AccountError> {
    let (answer_account, answer_username, token) = answer;
    if answer_account != account_id || answer_username != username || !is_token(token.expose()) {
        return Err(AccountError::Protocol(
            "the server answered for another account".into(),
        ));
    }
    Ok(())
}

/// `GET /v2/health` on `server`: whether it takes new accounts. A 404, or
/// an answer that isn't a protocol-2 health check, is
/// [`AccountError::ServerOutdated`].
pub fn server_info(b: &dyn Backend, server: &str) -> Result<SignupState, AccountError> {
    let (server, _) = checked_server(server)?;
    match b.api(&server).health() {
        Ok(health) if health.ok && health.protocol == PROTOCOL_VERSION => Ok(health.signup),
        Ok(_) | Err(ApiError::NotFound | ApiError::Protocol(_) | ApiError::Unrecognized { .. }) => {
            Err(AccountError::ServerOutdated)
        }
        Err(err) => Err(err.into()),
    }
}

/// `GET /v2/health`'s advertised sign-in methods. An older server omits
/// `auth`, which means username and password only and no live approvals.
pub fn server_auth(b: &dyn Backend, server: &str) -> Result<ServerAuth, AccountError> {
    let (server, _) = checked_server(server)?;
    let health = match b.api(&server).health() {
        Ok(health) if health.ok && health.protocol == PROTOCOL_VERSION => health,
        Ok(_) | Err(ApiError::NotFound | ApiError::Protocol(_) | ApiError::Unrecognized { .. }) => {
            return Err(AccountError::ServerOutdated);
        }
        Err(err) => return Err(err.into()),
    };
    let Some(auth) = health.auth else {
        return Ok(ServerAuth::default());
    };
    Ok(ServerAuth {
        password: auth.password,
        passkeys: auth
            .passkeys
            .map(|passkeys| crate::provider::PasskeyMethods {
                rp_id: passkeys.rp_id,
                security_key: passkeys.security_key,
            }),
        providers: auth.providers,
        account_events: None,
    })
}

/// `GET /v2/account/sign-in`, with the server's advertisement: the account's
/// ways in, and which of the advertised providers it hasn't linked.
pub fn methods(b: &dyn Backend, session: &Session) -> Result<AccountMethods, AccountError> {
    let api = b.api(&session.server);
    let auth = server_auth(b, &session.server)?;
    let reply: MethodsReply =
        retrying(b, || api.sign_in_methods(session.token())).map_err(AccountError::from)?;
    Ok(reply.methods(auth.providers, auth.passkeys.map(|passkeys| passkeys.rp_id)))
}

/// Refreshes the password-availability hint using only a locked session's
/// bearer token. This is display guidance; password unlock still authenticates
/// and checks the account and data-key ID independently.
pub fn methods_locked(b: &dyn Backend, locked: &LockedSession) -> Result<bool, AccountError> {
    let (server, origin) = checked_server(&locked.server)?;
    if origin != locked.origin || normalize_username(&locked.username)? != locked.username {
        return Err(AccountError::KeyMismatch);
    }
    let api = b.api(&server);
    let reply: MethodsReply =
        retrying(b, || api.sign_in_methods(locked.token())).map_err(AccountError::from)?;
    if !normalize_username(&reply.username).is_ok_and(|username| username == locked.username) {
        return Err(AccountError::KeyMismatch);
    }
    Ok(reply.has_password)
}

/// Refreshes and stores the locked-session password hint.
pub fn refresh_locked_methods(
    b: &dyn Backend,
    locked: &mut LockedSession,
) -> Result<(), AccountError> {
    locked.has_password = methods_locked(b, locked)?;
    Ok(())
}

/// A provider flow's device: its ID and proof. For a link it is the
/// session's; for a sign-in this Mac builds one and keeps it for the whole
/// flow (the Worker binds the flow to it).
fn provider_device(device: &LoginDevice) -> LoginDevice {
    device.clone()
}

/// Starts a provider sign-in or sign-up flow (docs/ACCOUNTS.md › Sign-in
/// routes). Google needs the loopback and the S256 challenge the app made;
/// the verifier stays in the app until [`finish_google`].
pub fn start_provider(
    b: &dyn Backend,
    server: &str,
    provider: &Provider,
    device: &LoginDevice,
    loopback: Option<&Loopback>,
    code_challenge: Option<&str>,
) -> Result<ProviderStart, AccountError> {
    let (server, _) = checked_server(server)?;
    let api = b.api(&server);
    let device = provider_device(device);
    let start = match provider {
        Provider::Google => {
            let loopback = loopback.ok_or(AccountError::BadRequest)?;
            let challenge = code_challenge.ok_or(AccountError::BadRequest)?;
            retrying(b, || {
                api.google_start(&GoogleStartRequest {
                    device: device.clone(),
                    redirect_uri: loopback.uri(),
                    code_challenge: challenge.to_owned(),
                })
            })
            .map_err(AccountError::from)
        }
        _ => retrying(b, || {
            api.provider_start(
                provider,
                &ProviderStartRequest {
                    device: device.clone(),
                },
            )
        })
        .map_err(AccountError::from),
    }?;
    // Nothing opens a page the server named until it is the provider's.
    let mut start = start;
    crate::provider::check_provider_start(provider, &mut start, &server)?;
    Ok(start)
}

/// One Apple or GitHub poll of a started flow. `secret` is Apple's, from the
/// callback URL ([`crate::provider::apple_callback_flow`]); GitHub has none.
pub fn poll_provider(
    b: &dyn Backend,
    server: &str,
    provider: &Provider,
    flow: &str,
    secret: Option<&str>,
    device: &LoginDevice,
) -> Result<ProviderStep, AccountError> {
    let (server, _) = checked_server(server)?;
    let api = b.api(&server);
    retrying(b, || {
        api.provider_poll(
            provider,
            &ProviderPollRequest {
                flow_id: flow.to_owned(),
                s: secret.map(str::to_owned),
                device: provider_device(device),
            },
        )
    })
    .map_err(AccountError::from)
}

/// Google's finish: the loopback's code and the verifier whose challenge the
/// start carried. The Worker exchanges and verifies; this crate never sees a
/// token.
pub fn finish_google(
    b: &dyn Backend,
    server: &str,
    flow: &str,
    code: &str,
    code_verifier: &str,
    device: &LoginDevice,
) -> Result<ProviderStep, AccountError> {
    let (server, _) = checked_server(server)?;
    let api = b.api(&server);
    retrying(b, || {
        api.google_finish(&GoogleFinishRequest {
            flow_id: flow.to_owned(),
            code: code.to_owned(),
            code_verifier: code_verifier.to_owned(),
            device: provider_device(device),
        })
    })
    .map_err(AccountError::from)
}

/// The provider sign-in's answer for Apple or GitHub, once the browser flow
/// has been completed. Google's answer comes from [`finish_google`]. A
/// pending flow is [`AccountError::Protocol`]: the app keeps polling with
/// [`poll_provider`].
pub fn sign_in_with_provider(
    b: &dyn Backend,
    server: &str,
    provider: &Provider,
    flow: &str,
    secret: Option<&str>,
    device: &LoginDevice,
    progress: &dyn Fn(Step),
) -> Result<ProviderSignIn, AccountError> {
    if matches!(provider, Provider::Google) {
        return Err(AccountError::Protocol(
            "Google's flow is answered by finish_google".into(),
        ));
    }
    progress(Step::Contacting);
    match poll_provider(b, server, provider, flow, secret, device)? {
        ProviderStep::Session(session) => Ok(ProviderSignIn::Session(session)),
        ProviderStep::NewAccount(account) => Ok(ProviderSignIn::NewAccount(account)),
        ProviderStep::Pending { .. } => Err(AccountError::Protocol(
            "the provider sign-in is still pending".into(),
        )),
    }
}

/// Creates the passwordless account a provider ticket names
/// (docs/ACCOUNTS.md › Sign in with Apple, Google and GitHub › Common
/// rules): a random KDF salt, auth hash and password wrap that no client
/// holds, the identity, the session and the recovery material in one insert.
/// The new recovery key is saved in `sync-recovery-key:<library>` before the
/// request is sent.
#[allow(clippy::too_many_arguments, reason = "the flow's inputs")]
pub fn sign_up_with_provider(
    b: &dyn Backend,
    secrets: &dyn SecretStore,
    server: &str,
    _provider: &Provider,
    flow: &str,
    username: &str,
    device: &DeviceIdentity,
    login_device: &LoginDevice,
    progress: &dyn Fn(Step),
) -> Result<SignedUp, RotationError> {
    let username = check_new_username(username).map_err(AccountError::from)?;
    let (server, origin) = checked_server(server)?;
    progress(Step::Contacting);
    match server_info(b, &server)? {
        SignupState::Open => {}
        SignupState::Closed => {
            return Err(AccountError::SignupClosed {
                reason: SignupClosedReason::Closed,
            }
            .into());
        }
        SignupState::Full => {
            return Err(AccountError::SignupClosed {
                reason: SignupClosedReason::Full,
            }
            .into());
        }
    }
    let mut data_key = DataKey::generate();
    let mut account_id = new_account_id();
    let mut rotation = Rotation::choose(
        secrets,
        &device.library_id,
        &server,
        &origin,
        &account_id,
        &username,
        None,
    )?;
    progress(Step::DerivingKeys);
    let recovery = recovery_keys(rotation.key(), &origin, &username);
    let mut content = ContentKeys::derive(&data_key);
    let api = b.api(&server);
    let mut fresh_ids = 0;
    let mut reached = false;
    loop {
        let request = SocialSignupRequest {
            ticket: flow.to_owned(),
            username: username.clone(),
            account_id: account_id.clone(),
            recovery_auth_key: recovery.auth_key.to_base64url(),
            recovery_wrapped_key: wrap(
                &recovery.kek,
                WrapPurpose::Recovery,
                &account_id,
                &data_key,
            ),
            key_id: key_id(&data_key),
            device: provider_signup_device(device, login_device, &content, &account_id),
        };
        progress(Step::Contacting);
        if let Err(err) = rotation.keep(&account_id) {
            return Err(rotation.failed(err.into(), false));
        }
        let sent = retrying_noted(b, || api.social_signup(&request));
        reached |= sent.reached;
        match sent.result {
            Ok(answer) => {
                if answer.account_id != account_id
                    || answer.username != username
                    || !is_token(answer.token.expose())
                    || answer.key_id != key_id(&data_key)
                {
                    return Err(rotation.failed(
                        AccountError::Protocol("the server answered for another account".into()),
                        true,
                    ));
                }
                let recovery_key = rotation.accepted();
                let mut session = Session::new(
                    server,
                    origin,
                    account_id,
                    username,
                    request.device.id.clone(),
                    answer.token,
                    data_key,
                );
                session.set_library_id(&device.library_id);
                session.set_login_device(login_device.clone());
                return Ok(SignedUp {
                    session,
                    recovery_key,
                });
            }
            Err(ApiError::AccountIdTaken) if fresh_ids < 3 => {
                fresh_ids += 1;
                account_id = new_account_id();
                data_key = DataKey::generate();
                content = ContentKeys::derive(&data_key);
            }
            Err(err) => return Err(rotation.failed_api(err, reached)),
        }
    }
}

/// `POST /v2/account/sign-in/identities/<provider>/start`: a link (with a
/// password or passkey proof) or a provider proof (the session is enough).
/// The page it names is checked as [`start_provider`]'s is.
#[allow(clippy::too_many_arguments, reason = "the flow's inputs")]
pub fn start_identity(
    b: &dyn Backend,
    session: &Session,
    provider: &Provider,
    intent: IdentityIntent,
    proof: Option<crate::provider::ProofWire>,
    loopback: Option<&Loopback>,
    code_challenge: Option<&str>,
    device: &LoginDevice,
) -> Result<ProviderStart, AccountError> {
    let api = b.api(&session.server);
    let start = retrying(b, || {
        api.identity_start(
            session.token(),
            provider,
            &IdentityStartRequest {
                intent,
                proof: proof.clone(),
                redirect_uri: loopback.map(Loopback::uri),
                code_challenge: code_challenge.map(str::to_owned),
                device: provider_device(device),
            },
        )
    })
    .map_err(AccountError::from)?;
    // As for a sign-in start: nothing opens a page the server named until
    // it is the provider's.
    let mut start = start;
    crate::provider::check_provider_start(provider, &mut start, &session.server)?;
    Ok(start)
}

/// The completion of a link or proof flow on Apple or GitHub; `secret` as
/// for [`poll_provider`].
pub fn finish_identity(
    b: &dyn Backend,
    session: &Session,
    provider: &Provider,
    flow: &str,
    secret: Option<&str>,
    device: &LoginDevice,
) -> Result<LinkOutcome, AccountError> {
    let api = b.api(&session.server);
    retrying(b, || {
        api.identity_poll(
            session.token(),
            provider,
            &ProviderPollRequest {
                flow_id: flow.to_owned(),
                s: secret.map(str::to_owned),
                device: provider_device(device),
            },
        )
    })
    .map_err(AccountError::from)
}

/// The completion of a link or proof flow on Google.
pub fn finish_identity_google(
    b: &dyn Backend,
    session: &Session,
    provider: &Provider,
    flow: &str,
    code: &str,
    code_verifier: &str,
    device: &LoginDevice,
) -> Result<LinkOutcome, AccountError> {
    let api = b.api(&session.server);
    retrying(b, || {
        api.identity_finish_google(
            session.token(),
            provider,
            &GoogleFinishRequest {
                flow_id: flow.to_owned(),
                code: code.to_owned(),
                code_verifier: code_verifier.to_owned(),
                device: provider_device(device),
            },
        )
    })
    .map_err(AccountError::from)
}

/// Unlinks `provider`. A provider proof can only unlink its own provider;
/// the Worker refuses `last_method` unless `recovery_auth_key` proves the
/// recovery key.
pub fn unlink_identity(
    b: &dyn Backend,
    session: &Session,
    provider: &Provider,
    proof: crate::provider::ProofWire,
    recovery: Option<&RecoveryKey>,
) -> Result<(), AccountError> {
    let request = UnlinkRequest {
        proof,
        recovery_auth_key: recovery.map(|key| {
            // The Worker checks SHA-256 of the recovery auth key.
            recovery_keys(key, &session.origin, &session.username)
                .auth_key
                .to_base64url()
        }),
    };
    let api = b.api(&session.server);
    retrying(b, || {
        api.unlink_identity(session.token(), provider, &request)
    })
    .map_err(AccountError::from)
}

/// Creates an account and this device's session (docs/ACCOUNTS.md › Account
/// flows › Sign up). Refuses invalid and reserved names and bad passwords
/// offline. The new recovery key is saved in `sync-recovery-key:<library>`
/// before the request is sent ([`Rotation`]). A lost 201 is retried with the
/// identical body, which the server answers idempotently (security re-check
/// 7); an account ID someone else took is replaced and the keys rewrapped.
/// The caller then saves the session and binds.
pub fn sign_up(
    b: &dyn Backend,
    secrets: &dyn SecretStore,
    server: &str,
    username: &str,
    password: &Password,
    device: &DeviceIdentity,
    progress: &dyn Fn(Step),
) -> Result<SignedUp, RotationError> {
    let username = check_new_username(username).map_err(AccountError::from)?;
    check_not_username(password, &username)?;
    let (server, origin) = checked_server(server)?;
    progress(Step::Contacting);
    match server_info(b, &server)? {
        SignupState::Open => {}
        SignupState::Closed => {
            return Err(AccountError::SignupClosed {
                reason: SignupClosedReason::Closed,
            }
            .into());
        }
        SignupState::Full => {
            return Err(AccountError::SignupClosed {
                reason: SignupClosedReason::Full,
            }
            .into());
        }
    }
    let params = default_params(b)?;
    let salt = random::<16>();
    let data_key = DataKey::generate();
    let mut account_id = new_account_id();
    let mut rotation = Rotation::choose(
        secrets,
        &device.library_id,
        &server,
        &origin,
        &account_id,
        &username,
        None,
    )?;
    progress(Step::DerivingKeys);
    let keys =
        password_keys(password, &origin, &username, &salt, params).map_err(AccountError::from)?;
    let recovery = recovery_keys(rotation.key(), &origin, &username);
    let content = ContentKeys::derive(&data_key);
    let api = b.api(&server);
    let mut fresh_ids = 0;
    // Whether any attempt, for any account ID tried, may have created the
    // account: they all carry this key.
    let mut reached = false;
    loop {
        let request = SignupRequest {
            username: username.clone(),
            account_id: account_id.clone(),
            kdf: params.into(),
            salt: b64(&salt),
            auth_key: keys.auth_key.to_base64url(),
            wrapped_key: wrap(&keys.kek, WrapPurpose::Password, &account_id, &data_key),
            recovery_auth_key: recovery.auth_key.to_base64url(),
            recovery_wrapped_key: wrap(
                &recovery.kek,
                WrapPurpose::Recovery,
                &account_id,
                &data_key,
            ),
            key_id: key_id(&data_key),
            device: new_device(device, &content, &origin, &account_id),
        };
        progress(Step::Contacting);
        if let Err(err) = rotation.keep(&account_id) {
            return Err(rotation.failed(err.into(), false));
        }
        let sent = retrying_noted(b, || api.signup(&request));
        reached |= sent.reached;
        match sent.result {
            Ok(answer) => {
                if let Err(err) = check_answer(
                    &account_id,
                    &username,
                    (&answer.account_id, &answer.username, &answer.token),
                ) {
                    return Err(rotation.failed(err, true));
                }
                let recovery_key = rotation.accepted();
                let mut session = Session::new(
                    server,
                    origin,
                    account_id,
                    username,
                    request.device.id.clone(),
                    answer.token,
                    data_key,
                );
                session.set_library_id(&device.library_id);
                return Ok(SignedUp {
                    session,
                    recovery_key,
                });
            }
            Err(ApiError::AccountIdTaken) if fresh_ids < 3 => {
                fresh_ids += 1;
                account_id = new_account_id();
            }
            Err(err) => return Err(rotation.failed_api(err, reached)),
        }
    }
}

/// Signs this device in: prelogin (refusing unsafe parameters), derive, log
/// in with the device proof, unwrap and compare the key ID, then `PUT
/// /v2/account/device` with the sealed name. A key mismatch logs the new
/// session out and saves nothing.
pub fn sign_in(
    b: &dyn Backend,
    server: &str,
    username: &str,
    password: &Password,
    device: &DeviceIdentity,
    progress: &dyn Fn(Step),
) -> Result<SignedIn, AccountError> {
    let username = normalize_username(username)?;
    let (server, origin) = checked_server(server)?;
    let api = b.api(&server);
    progress(Step::Contacting);
    let pre = prelogin(b, &*api, &username)?;
    let params = pre.params(b)?;
    progress(Step::DerivingKeys);
    let keys = password_keys(password, &origin, &username, &pre.salt, params)?;
    let account_id = pre.account_id;
    let device_id = device.device_id(&account_id);
    let request = LoginRequest {
        username: username.clone(),
        auth_key: keys.auth_key.to_base64url(),
        device: LoginDevice {
            id: device_id.clone(),
            proof: device.device_proof(&origin, &account_id).to_base64url(),
        },
    };
    progress(Step::Contacting);
    let answer = retrying(b, || api.login(&request))?;
    let fail = |err: AccountError| {
        let _ = api.logout(&answer.token);
        Err(err)
    };
    if let Err(err) = check_answer(
        &account_id,
        &username,
        (&answer.account_id, &answer.username, &answer.token),
    ) {
        return fail(err);
    }
    let data_key = match unwrap(
        &keys.kek,
        WrapPurpose::Password,
        &account_id,
        &answer.wrapped_key,
    ) {
        Ok(data_key) if key_id(&data_key) == answer.key_id => data_key,
        _ => return fail(AccountError::KeyMismatch),
    };
    let content = ContentKeys::derive(&data_key);
    // The name is cosmetic ("A Mac" until set); a failure here doesn't undo
    // the sign-in.
    let _ = api.set_device_name(
        &answer.token,
        &DeviceNameRequest {
            sealed_name: content.seal_name(&account_id, &device_id, &device.name),
        },
    );
    let evicted = answer.evicted_device.as_ref().map(|evicted| DeviceInfo {
        id: evicted.id.clone(),
        name: opened_name(
            &content,
            &account_id,
            &evicted.id,
            evicted.sealed_name.as_deref(),
        ),
        created_at: 0,
        last_seen_at: 0,
        current: false,
    });
    let mut session = Session::new(
        server,
        origin,
        account_id,
        username,
        device_id,
        answer.token.clone(),
        data_key,
    );
    session.set_library_id(&device.library_id);
    Ok(SignedIn { session, evicted })
}

/// Unlocks a provider/passkey session by signing in with the account's
/// password on the same server and device. The returned account, device,
/// origin and key ID must match the pending session; a mismatched login is
/// logged out (or queued) before the error is returned.
pub fn unlock_with_password(
    b: &dyn Backend,
    locked: &LockedSession,
    password: &Password,
    device: &DeviceIdentity,
    progress: &dyn Fn(Step),
) -> Result<SignedIn, AccountError> {
    if locked
        .library_id()
        .is_some_and(|library_id| library_id != device.library_id)
    {
        return Err(AccountError::KeyMismatch);
    }
    let username = normalize_username(&locked.username)?;
    let (server, origin) = checked_server(&locked.server)?;
    if origin != locked.origin || username != locked.username {
        return Err(AccountError::KeyMismatch);
    }
    let api = b.api(&server);
    progress(Step::Contacting);
    let pre = prelogin(b, &*api, &username)?;
    if pre.account_id != locked.account_id {
        return Err(AccountError::KeyMismatch);
    }
    let params = pre.params(b)?;
    progress(Step::DerivingKeys);
    let keys = password_keys(password, &origin, &username, &pre.salt, params)?;
    let login_device = locked
        .login_device()
        .cloned()
        .unwrap_or_else(|| LoginDevice {
            id: locked.device_id.clone(),
            proof: device
                .device_proof(&locked.origin, &locked.account_id)
                .to_base64url(),
        });
    if login_device.id != locked.device_id
        || unb64(&login_device.proof).is_none_or(|proof| proof.len() != 32)
    {
        return Err(AccountError::KeyMismatch);
    }
    let kept_device = login_device.clone();
    let request = LoginRequest {
        username: username.clone(),
        auth_key: keys.auth_key.to_base64url(),
        device: login_device,
    };
    debug_assert_eq!(request.device.id, locked.device_id);
    progress(Step::Contacting);
    let answer = retrying(b, || api.login(&request))?;
    let fail = |error: AccountError| {
        let _ = api.logout(&answer.token);
        Err(error)
    };
    if let Err(error) = check_answer(
        &locked.account_id,
        &locked.username,
        (&answer.account_id, &answer.username, &answer.token),
    ) {
        return fail(error);
    }
    let data_key = match unwrap(
        &keys.kek,
        WrapPurpose::Password,
        &locked.account_id,
        &answer.wrapped_key,
    ) {
        Ok(data_key) if key_id(&data_key) == answer.key_id && answer.key_id == locked.key_id => {
            data_key
        }
        _ => return fail(AccountError::KeyMismatch),
    };
    if !is_token(answer.token.expose()) {
        return fail(AccountError::Protocol(
            "login returned a malformed token".into(),
        ));
    }
    let content = ContentKeys::derive(&data_key);
    let _ = api.set_device_name(
        &answer.token,
        &DeviceNameRequest {
            sealed_name: content.seal_name(&locked.account_id, &locked.device_id, &device.name),
        },
    );
    let evicted = answer
        .evicted_device
        .as_ref()
        .map(|evicted| DeviceInfo {
            id: evicted.id.clone(),
            name: opened_name(
                &content,
                &locked.account_id,
                &evicted.id,
                evicted.sealed_name.as_deref(),
            ),
            created_at: 0,
            last_seen_at: 0,
            current: false,
        })
        .or_else(|| crate::approvals::opened_evicted(locked, &content));
    let mut session = Session::new(
        server,
        origin,
        locked.account_id.clone(),
        username,
        locked.device_id.clone(),
        answer.token,
        data_key,
    );
    session.set_library_id(&device.library_id);
    session.set_login_device(kept_device);
    Ok(SignedIn { session, evicted })
}

/// Recovers the account with its recovery key and sets `new_password`. The
/// used key stops working; the new one is saved in
/// `sync-recovery-key:<library>` before `recover/finish` is sent
/// ([`Rotation`]), and the result carries it.
#[allow(clippy::too_many_arguments, reason = "the flow's inputs")]
pub fn recover(
    b: &dyn Backend,
    secrets: &dyn SecretStore,
    server: &str,
    username: &str,
    key: &RecoveryKey,
    new_password: &Password,
    device: &DeviceIdentity,
    progress: &dyn Fn(Step),
) -> Result<Recovered, RotationError> {
    let username = normalize_username(username).map_err(AccountError::from)?;
    check_not_username(new_password, &username)?;
    let (server, origin) = checked_server(server)?;
    let api = b.api(&server);
    progress(Step::Contacting);
    let pre = prelogin(b, &*api, &username)?;
    let used = recovery_keys(key, &origin, &username);
    let start = retrying(b, || {
        api.recover_start(&RecoverStartRequest {
            username: username.clone(),
            recovery_auth_key: used.auth_key.to_base64url(),
        })
    })
    .map_err(AccountError::from)?;
    if start.account_id != pre.account_id {
        return Err(AccountError::Protocol("recovery answered for another account".into()).into());
    }
    let account_id = start.account_id;
    let data_key = match unwrap(
        &used.kek,
        WrapPurpose::Recovery,
        &account_id,
        &start.recovery_wrapped_key,
    ) {
        Ok(data_key) if key_id(&data_key) == start.key_id => data_key,
        _ => return Err(AccountError::KeyMismatch.into()),
    };
    let params = default_params(b)?;
    let salt = random::<16>();
    let mut rotation = Rotation::choose(
        secrets,
        &device.library_id,
        &server,
        &origin,
        &account_id,
        &username,
        Some(key),
    )?;
    progress(Step::DerivingKeys);
    let keys = password_keys(new_password, &origin, &username, &salt, params)
        .map_err(AccountError::from)?;
    let next = recovery_keys(rotation.key(), &origin, &username);
    let content = ContentKeys::derive(&data_key);
    let request = RecoverFinishRequest {
        username: username.clone(),
        recovery_auth_key: used.auth_key.to_base64url(),
        kdf: params.into(),
        salt: b64(&salt),
        auth_key: keys.auth_key.to_base64url(),
        wrapped_key: wrap(&keys.kek, WrapPurpose::Password, &account_id, &data_key),
        recovery_auth_key_new: next.auth_key.to_base64url(),
        recovery_wrapped_key_new: wrap(&next.kek, WrapPurpose::Recovery, &account_id, &data_key),
        device: new_device(device, &content, &origin, &account_id),
    };
    progress(Step::Contacting);
    if let Err(err) = rotation.keep(&account_id) {
        return Err(rotation.failed(err.into(), false));
    }
    let sent = retrying_noted(b, || api.recover_finish(&request));
    let answer = match sent.result {
        Ok(answer) => answer,
        Err(err) => return Err(rotation.failed_api(err, sent.reached)),
    };
    if let Err(err) = check_answer(
        &account_id,
        &username,
        (&answer.account_id, &answer.username, &answer.token),
    ) {
        return Err(rotation.failed(err, true));
    }
    let recovery_key = rotation.accepted();
    let mut session = Session::new(
        server,
        origin,
        account_id,
        username,
        request.device.id.clone(),
        answer.token,
        data_key,
    );
    session.set_library_id(&device.library_id);
    Ok(Recovered {
        session,
        recovery_key,
    })
}

/// Whether `key` is `username`'s recovery key on `server` now: one
/// `recover/start`, sent once, which changes nothing on the server
/// (review-verify 6). It settles a key whose rotation's answer was lost.
/// `Ok(false)` when the server refuses the key (401); any other failure is
/// an error and says nothing about the key. Each call counts against the
/// server's `recover-ip` limit (10 an hour per IP), so the app asks once
/// per key.
pub fn recovery_key_held(
    b: &dyn Backend,
    server: &str,
    username: &str,
    key: &RecoveryKey,
) -> Result<bool, AccountError> {
    let username = normalize_username(username)?;
    let (server, origin) = checked_server(server)?;
    let auth = recovery_keys(key, &origin, &username).auth_key;
    let asked = b.api(&server).recover_start(&RecoverStartRequest {
        username,
        recovery_auth_key: auth.to_base64url(),
    });
    match asked {
        Ok(_) => Ok(true),
        Err(ApiError::InvalidCredentials) => Ok(false),
        Err(err) => Err(err.into()),
    }
}

/// Deletes `sync-account:<library>` and any `sync-recovery-key:<library>`,
/// then tries `logout` (the HTTP backend gives it 5 s). On failure it queues
/// `{server, token}` in `sync-logout:<library>`; a 401 counts as done. The
/// logout is tried even when the Keychain refuses a deletion (A14); that
/// refusal is the error then. The caller has already cancelled and joined
/// the running sync.
pub fn sign_out(
    b: &dyn Backend,
    secrets: &dyn SecretStore,
    session: &Session,
) -> Result<(), AccountError> {
    let mut deleted = Ok(());
    if let Some(library_id) = session.library_id() {
        let account = keychain::delete_session(secrets, library_id);
        let key = keychain::delete_recovery_key(secrets, library_id);
        let pending = keychain::delete_pending(secrets, library_id);
        deleted = account.and(key).and(pending);
    }
    let logged_out = logout_or_queue(b, secrets, session);
    deleted?;
    logged_out
}

/// Cancels a locked sign-in: removes its pending Keychain item, then logs out
/// or queues its token without needing or constructing a data-key-bearing
/// `Session`.
pub fn discard_locked_session(
    b: &dyn Backend,
    secrets: &dyn SecretStore,
    locked: &LockedSession,
) -> Result<(), AccountError> {
    sign_out_locked(b, secrets, locked)
}

/// Signs out a locked session and removes its pending Keychain item. The
/// library's account and recovery items are left intact.
pub fn sign_out_locked(
    b: &dyn Backend,
    secrets: &dyn SecretStore,
    locked: &LockedSession,
) -> Result<(), AccountError> {
    let deleted = match locked.library_id() {
        Some(library_id) => keychain::delete_locked_pending(secrets, library_id),
        None => Ok(()),
    };
    let logged_out = logout_token_or_queue(
        b,
        secrets,
        &locked.server,
        locked.token(),
        locked.library_id(),
    );
    deleted?;
    logged_out
}

/// Logs out a session this Mac never saved or used: one whose binding
/// question was cancelled, or whose saving failed. Like [`sign_out`], a
/// failed logout is queued in `sync-logout:<library>`, but the library's
/// `sync-account` and `sync-recovery-key` items stay: they may hold the
/// account this Mac is still signed in to, or a recovery key still on
/// screen.
pub fn discard_session(
    b: &dyn Backend,
    secrets: &dyn SecretStore,
    session: &Session,
) -> Result<(), AccountError> {
    logout_or_queue(b, secrets, session)
}

/// Held around every read-modify-write of a `sync-logout:<library>` item in
/// this process (sync 9): a sign-out queuing a token while a flush runs on
/// another thread must not lose it. Never held across a network call.
static LOGOUT_QUEUE: Mutex<()> = Mutex::new(());

fn logout_queue() -> std::sync::MutexGuard<'static, ()> {
    LOGOUT_QUEUE.lock().unwrap_or_else(PoisonError::into_inner)
}

/// `logout`; on failure other than a 401, the session's token is queued.
fn logout_or_queue(
    b: &dyn Backend,
    secrets: &dyn SecretStore,
    session: &Session,
) -> Result<(), AccountError> {
    logout_token_or_queue(
        b,
        secrets,
        &session.server,
        session.token(),
        session.library_id(),
    )
}

fn logout_token_or_queue(
    b: &dyn Backend,
    secrets: &dyn SecretStore,
    server: &str,
    token: &SessionToken,
    library_id: Option<&str>,
) -> Result<(), AccountError> {
    match b.api(server).logout(token) {
        Ok(()) => Ok(()),
        Err(err) if err.status() == Some(401) => Ok(()),
        Err(_) => {
            if let Some(library_id) = library_id {
                let _queue = logout_queue();
                let mut pending = keychain::pending_logouts(secrets, library_id)?;
                let logout = PendingLogout {
                    server: server.to_owned(),
                    token: token.clone(),
                };
                if !pending.contains(&logout) {
                    pending.push(logout);
                }
                keychain::set_pending_logouts(secrets, library_id, &pending)?;
            }
            Ok(())
        }
    }
}

/// Retries every pending logout of the library until the server answers 204
/// or 401. Runs at launch and before each sync; failures stay queued, and
/// Keychain errors are left for the next run. The queue is read, the
/// logouts sent, then the ones that went are taken out of the queue as it
/// is by then, so a token queued meanwhile stays.
pub fn flush_pending_logouts(b: &dyn Backend, secrets: &dyn SecretStore, library_id: &str) {
    let pending = {
        let _queue = logout_queue();
        match keychain::pending_logouts(secrets, library_id) {
            Ok(pending) => pending,
            Err(_) => return,
        }
    };
    let gone: Vec<PendingLogout> = pending
        .into_iter()
        .filter(|logout| {
            let Ok(server) = crate::check_endpoint(&logout.server) else {
                return true;
            };
            match b.api(&server).logout(&logout.token) {
                Ok(()) => true,
                Err(err) => err.status() == Some(401),
            }
        })
        .collect();
    if gone.is_empty() {
        return;
    }
    let _queue = logout_queue();
    let Ok(mut pending) = keychain::pending_logouts(secrets, library_id) else {
        return;
    };
    pending.retain(|logout| !gone.contains(logout));
    let _ = keychain::set_pending_logouts(secrets, library_id, &pending);
}

/// The current password's auth key: prelogin for the salt and parameters
/// (refused outside the floor or ceilings), then Argon2id.
pub(crate) fn current_auth(
    b: &dyn Backend,
    api: &dyn Api,
    session: &Session,
    current: &Password,
    progress: &dyn Fn(Step),
) -> Result<crate::crypto::DerivedKeys, AccountError> {
    progress(Step::Contacting);
    let pre = prelogin(b, api, &session.username)?;
    let params = pre.params(b)?;
    progress(Step::DerivingKeys);
    Ok(password_keys(
        current,
        &session.origin,
        &session.username,
        &pre.salt,
        params,
    )?)
}

/// Changes the password and, with it, the recovery key: a recovery key
/// that outlived the password would let whoever holds it take the account
/// back. The data key is rewrapped, not replaced (M4). The new recovery key
/// is saved in `sync-recovery-key:<library>` before the request is sent
/// ([`Rotation`]); the result carries it and how many other devices were
/// signed out.
pub fn change_password(
    b: &dyn Backend,
    secrets: &dyn SecretStore,
    session: &Session,
    current: &Password,
    new: &Password,
    progress: &dyn Fn(Step),
) -> Result<PasswordChanged, RotationError> {
    check_not_username(new, &session.username)?;
    let library_id = session_library(session)?;
    let api = b.api(&session.server);
    let current_keys = current_auth(b, &*api, session, current, progress)?;
    let params = default_params(b)?;
    let salt = random::<16>();
    let keys = password_keys(new, &session.origin, &session.username, &salt, params)
        .map_err(AccountError::from)?;
    let mut rotation = Rotation::choose(
        secrets,
        library_id,
        &session.server,
        &session.origin,
        &session.account_id,
        &session.username,
        None,
    )?;
    let next = recovery_keys(rotation.key(), &session.origin, &session.username);
    let request = PasswordRequest {
        current_auth_key: current_keys.auth_key.to_base64url(),
        kdf: params.into(),
        salt: b64(&salt),
        auth_key: keys.auth_key.to_base64url(),
        wrapped_key: wrap(
            &keys.kek,
            WrapPurpose::Password,
            &session.account_id,
            session.data_key(),
        ),
        recovery_auth_key: next.auth_key.to_base64url(),
        recovery_wrapped_key: wrap(
            &next.kek,
            WrapPurpose::Recovery,
            &session.account_id,
            session.data_key(),
        ),
    };
    progress(Step::Contacting);
    if let Err(err) = rotation.keep(&session.account_id) {
        return Err(rotation.failed(err.into(), false));
    }
    let sent = retrying_noted(b, || api.change_password(session.token(), &request));
    match sent.result {
        Ok(answer) => Ok(PasswordChanged {
            signed_out_devices: usize::try_from(answer.signed_out_devices).unwrap_or(usize::MAX),
            recovery_key: rotation.accepted(),
        }),
        Err(err) => Err(rotation.failed_api(err, sent.reached)),
    }
}

/// The library whose Keychain items hold `session`.
pub(crate) fn session_library(session: &Session) -> Result<&str, AccountError> {
    session
        .library_id()
        .ok_or_else(|| AccountError::Keychain(SecretError("the session names no library".into())))
}

/// Replaces the recovery key while signed in and returns the new one. It is
/// saved in `sync-recovery-key:<library>` before the request is sent
/// ([`Rotation`]).
pub fn new_recovery_key(
    b: &dyn Backend,
    secrets: &dyn SecretStore,
    session: &Session,
    current: &Password,
    progress: &dyn Fn(Step),
) -> Result<RecoveryKey, RotationError> {
    let library_id = session_library(session)?;
    let api = b.api(&session.server);
    let current_keys = current_auth(b, &*api, session, current, progress)?;
    let mut rotation = Rotation::choose(
        secrets,
        library_id,
        &session.server,
        &session.origin,
        &session.account_id,
        &session.username,
        None,
    )?;
    let next = recovery_keys(rotation.key(), &session.origin, &session.username);
    let request = RecoveryKeyRequest {
        current_auth_key: Some(current_keys.auth_key.to_base64url()),
        proof: None,
        current_recovery_auth_key: None,
        recovery_auth_key: next.auth_key.to_base64url(),
        recovery_wrapped_key: wrap(
            &next.kek,
            WrapPurpose::Recovery,
            &session.account_id,
            session.data_key(),
        ),
    };
    progress(Step::Contacting);
    if let Err(err) = rotation.keep(&session.account_id) {
        return Err(rotation.failed(err.into(), false));
    }
    let sent = retrying_noted(b, || api.replace_recovery_key(session.token(), &request));
    match sent.result {
        Ok(_) => Ok(rotation.accepted()),
        Err(err) => Err(rotation.failed_api(err, sent.reached)),
    }
}

/// Deletes the account and its server data. A retry whose response was lost
/// gets 401; only a 401 with reason `unknown` counts as success (security
/// re-check 12). A typed username that isn't this account's is refused
/// before anything is sent. The app then unbinds.
pub fn delete_account(
    b: &dyn Backend,
    session: &Session,
    password: &Password,
    typed_username: &str,
    progress: &dyn Fn(Step),
) -> Result<(), AccountError> {
    let typed = normalize_username(typed_username).map_err(|_| AccountError::UsernameMismatch)?;
    if typed != session.username {
        return Err(AccountError::UsernameMismatch);
    }
    let api = b.api(&session.server);
    let keys = current_auth(b, &*api, session, password, progress)?;
    let request = DeleteRequest {
        auth_key: Some(keys.auth_key.to_base64url()),
        proof: None,
        current_recovery_auth_key: None,
        username: typed,
    };
    progress(Step::Contacting);
    match retrying(b, || api.delete_account(session.token(), &request)) {
        Ok(_)
        | Err(ApiError::Unauthorized {
            reason: SignOutReason::Unknown,
        }) => Ok(()),
        Err(err) => Err(err.into()),
    }
}

/// The account's active devices, names opened, most recently seen first.
pub fn devices(b: &dyn Backend, session: &Session) -> Result<Vec<DeviceInfo>, AccountError> {
    let answer = b.api(&session.server).devices(session.token())?;
    let content = ContentKeys::derive(session.data_key());
    let mut devices: Vec<DeviceInfo> = answer
        .devices
        .into_iter()
        .map(|row| DeviceInfo {
            name: opened_name(
                &content,
                &session.account_id,
                &row.id,
                row.sealed_name.as_deref(),
            ),
            current: row.id == session.device_id,
            id: row.id,
            created_at: row.created_at,
            last_seen_at: row.last_seen_at,
        })
        .collect();
    devices.sort_by_key(|d| std::cmp::Reverse(d.last_seen_at));
    Ok(devices)
}

/// Signs out another device of this account (revoking this one is a logout).
pub fn revoke_device(
    b: &dyn Backend,
    session: &Session,
    device_id: &str,
) -> Result<(), AccountError> {
    Ok(b.api(&session.server)
        .revoke_device(session.token(), device_id)?)
}

/// Tells the server where to wake this device: its OS push token. The
/// server drops it when the session ends, so signing in again registers anew.
pub fn register_push(
    b: &dyn Backend,
    session: &Session,
    registration: &PushRegistration,
) -> Result<(), AccountError> {
    Ok(b.api(&session.server)
        .put_push(session.token(), &session.device_id, registration)?)
}

/// Turns pushes off for this device. Safe to repeat.
pub fn unregister_push(b: &dyn Backend, session: &Session) -> Result<(), AccountError> {
    Ok(b.api(&session.server)
        .delete_push(session.token(), &session.device_id)?)
}

/// Storage use and quotas.
pub fn usage(b: &dyn Backend, session: &Session) -> Result<Usage, AccountError> {
    let answer = b.api(&session.server).account(session.token())?;
    Ok(Usage {
        docs: answer.usage.docs,
        doc_bytes: answer.usage.doc_bytes,
        files: answer.usage.files,
        file_bytes: answer.usage.file_bytes,
        writes_today: answer.usage.writes_today,
        quota: Quota {
            docs: answer.quota.docs,
            doc_bytes: answer.quota.doc_bytes,
            files: answer.quota.files,
            file_bytes: answer.quota.file_bytes,
            writes_per_day: answer.quota.writes_per_day,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::Route;
    use crate::memory::MemoryServer;

    fn account_device() -> DeviceIdentity {
        DeviceIdentity::from_parts("l_0123456789abcdef0123456789abcdef", "Test Mac", [9; 32])
    }

    /// Every coded `ApiError` keeps its meaning as an `AccountError`.
    #[test]
    fn api_errors_map_to_account_errors() {
        let reason = SignOutReason::PasswordChanged;
        assert!(matches!(
            AccountError::from(ApiError::Unauthorized { reason }),
            AccountError::Unauthorized {
                reason: SignOutReason::PasswordChanged
            }
        ));
        assert!(matches!(
            AccountError::from(ApiError::RateLimited {
                retry_after: 120,
                limit: "login-user".into()
            }),
            AccountError::RateLimited { retry_after: 120 }
        ));
        assert!(matches!(
            AccountError::from(ApiError::Unrecognized {
                status: 418,
                code: "teapot".into()
            }),
            AccountError::Protocol(_)
        ));
        assert!(matches!(
            AccountError::from(ApiError::Transport("reset".into())),
            AccountError::Transport(_)
        ));
        assert!(matches!(
            AccountError::from(ApiError::UnsupportedMethod),
            AccountError::UnsupportedMethod
        ));
        assert!(matches!(
            AccountError::from(ApiError::FlowExpired { gone: false }),
            AccountError::FlowExpired
        ));
        assert!(matches!(
            AccountError::from(ApiError::ReauthRequired),
            AccountError::ReauthRequired
        ));
        assert!(matches!(
            AccountError::from(ApiError::LastMethod),
            AccountError::LastMethod
        ));
        assert!(matches!(
            AccountError::from(ApiError::CredentialExists),
            AccountError::CredentialExists
        ));
    }

    #[test]
    fn password_unlock_checks_locked_identity_and_uses_login_session() {
        let server = MemoryServer::new();
        let secrets = crate::keychain::MemorySecrets::new();
        let device = account_device();
        let password = Password::new("a sufficiently long password", "alice").unwrap();
        let signed_up = sign_up(
            &server,
            &secrets,
            "https://a.example",
            "alice",
            &password,
            &device,
            &|_| {},
        )
        .unwrap();
        let original_token = signed_up.session.token().clone();
        let locked = LockedSession::new(
            signed_up.session.server.clone(),
            signed_up.session.origin.clone(),
            signed_up.session.account_id.clone(),
            signed_up.session.username.clone(),
            signed_up.session.device_id.clone(),
            original_token.clone(),
            signed_up.session.key_id(),
            "0123456789abcdef".into(),
            None,
            None,
        );

        let wrong_password = Password::new("a wrong password entirely", "alice").unwrap();
        assert!(unlock_with_password(&server, &locked, &wrong_password, &device, &|_| {}).is_err());

        let unlocked = unlock_with_password(&server, &locked, &password, &device, &|_| {}).unwrap();
        assert_eq!(unlocked.session.account_id, locked.account_id);
        assert_eq!(unlocked.session.device_id, locked.device_id);
        assert_eq!(unlocked.session.origin, locked.origin);
        assert_eq!(unlocked.session.key_id(), locked.key_id);
        assert_ne!(unlocked.session.token(), &original_token);
        assert!(matches!(
            server.api(&locked.server).account(&original_token),
            Err(ApiError::Unauthorized { .. })
        ));

        let mut wrong_key = locked.clone();
        wrong_key.key_id = "0000000000000000".into();
        assert!(matches!(
            unlock_with_password(&server, &wrong_key, &password, &device, &|_| {}),
            Err(AccountError::KeyMismatch)
        ));
    }

    #[test]
    fn password_unlock_uses_the_persisted_provider_device_proof() {
        use crate::crypto::b64;

        let server = MemoryServer::new();
        let secrets = crate::keychain::MemorySecrets::new();
        let library_device = account_device();
        let password = Password::new("a sufficiently long password", "alice").unwrap();
        let signed_up = sign_up(
            &server,
            &secrets,
            "https://a.example",
            "alice",
            &password,
            &library_device,
            &|_| {},
        )
        .unwrap();
        let api = server.api("https://a.example");
        let pre = prelogin(&server, &*api, "alice").unwrap();
        let params = pre.params(&server).unwrap();
        let keys = password_keys(
            &password,
            &signed_up.session.origin,
            "alice",
            &pre.salt,
            params,
        )
        .unwrap();
        let provider_device = LoginDevice {
            id: "d_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
            proof: b64(&[91; 32]),
        };
        let initial = api
            .login(&LoginRequest {
                username: "alice".into(),
                auth_key: keys.auth_key.to_base64url(),
                device: provider_device.clone(),
            })
            .unwrap();
        let mut locked = LockedSession::new(
            signed_up.session.server.clone(),
            signed_up.session.origin.clone(),
            initial.account_id.clone(),
            initial.username.clone(),
            provider_device.id.clone(),
            initial.token,
            initial.key_id.clone(),
            initial.epoch.clone(),
            None,
            None,
        )
        .with_login_device(provider_device.clone());
        locked.set_library_id(&library_device.library_id);
        assert_ne!(
            locked.device_id,
            library_device.device_id(&locked.account_id)
        );

        // Fill login-user without changing this provider device. A proof
        // regenerated from the library identity would no longer be treated as
        // the same device and would be rate limited; the persisted proof works.
        for index in 0..9u8 {
            let failed = api.login(&LoginRequest {
                username: "alice".into(),
                auth_key: b64(&[0; 32]),
                device: LoginDevice {
                    id: format!("d_{:032x}", u128::from(index) + 1),
                    proof: b64(&[index.wrapping_add(1); 32]),
                },
            });
            assert!(matches!(failed, Err(ApiError::InvalidCredentials)));
        }

        let unlocked = unlock_with_password(&server, &locked, &password, &library_device, &|_| {})
            .expect("the stored provider device proof bypasses the new-device throttle");
        assert_eq!(unlocked.session.device_id, provider_device.id);
        assert_eq!(unlocked.session.account_id, signed_up.session.account_id);
        assert_eq!(unlocked.session.origin, signed_up.session.origin);
        assert_eq!(unlocked.session.key_id(), signed_up.session.key_id());

        let wrong_proof = LoginDevice {
            id: provider_device.id.clone(),
            proof: b64(&[92; 32]),
        };
        let mut wrong_device = LockedSession::new(
            locked.server.clone(),
            locked.origin.clone(),
            locked.account_id.clone(),
            locked.username.clone(),
            locked.device_id.clone(),
            locked.token().clone(),
            locked.key_id.clone(),
            locked.epoch.clone(),
            None,
            None,
        )
        .with_login_device(wrong_proof);
        wrong_device.set_library_id(&library_device.library_id);
        assert!(matches!(
            unlock_with_password(&server, &wrong_device, &password, &library_device, &|_| {}),
            Err(AccountError::RateLimited { .. })
        ));

        let mut wrong_account = locked.clone();
        wrong_account.account_id = "a_ffffffffffffffffffffffffffffffff".into();
        assert!(matches!(
            unlock_with_password(&server, &wrong_account, &password, &library_device, &|_| {}),
            Err(AccountError::KeyMismatch)
        ));
        let mut wrong_origin = locked.clone();
        wrong_origin.origin = "https://foreign.example".into();
        assert!(matches!(
            unlock_with_password(&server, &wrong_origin, &password, &library_device, &|_| {}),
            Err(AccountError::KeyMismatch)
        ));
        let mut wrong_key = locked;
        wrong_key.key_id = "0000000000000000".into();
        assert!(matches!(
            unlock_with_password(&server, &wrong_key, &password, &library_device, &|_| {}),
            Err(AccountError::KeyMismatch)
        ));
    }

    #[test]
    fn locked_methods_refresh_password_hint_with_only_the_token() {
        let server = MemoryServer::new();
        let secrets = crate::keychain::MemorySecrets::new();
        let device = account_device();
        let password = Password::new("a sufficiently long password", "alice").unwrap();
        let signed_up = sign_up(
            &server,
            &secrets,
            "https://a.example",
            "alice",
            &password,
            &device,
            &|_| {},
        )
        .unwrap();
        let mut locked = LockedSession::new(
            signed_up.session.server.clone(),
            signed_up.session.origin.clone(),
            signed_up.session.account_id.clone(),
            signed_up.session.username.clone(),
            signed_up.session.device_id.clone(),
            signed_up.session.token().clone(),
            signed_up.session.key_id(),
            "0123456789abcdef".into(),
            None,
            None,
        )
        .with_login_device(LoginDevice::for_identity(
            &device,
            &signed_up.session.origin,
            &signed_up.session.account_id,
        ));
        locked.set_library_id(&device.library_id);
        crate::keychain::save_locked_pending(&secrets, &device.library_id, &locked).unwrap();
        let mut restored = crate::keychain::load_locked_pending(&secrets, &device.library_id)
            .unwrap()
            .unwrap();
        assert!(!restored.has_password);
        assert!(methods_locked(&server, &restored).unwrap());
        refresh_locked_methods(&server, &mut restored).unwrap();
        assert!(restored.has_password);
    }

    #[test]
    fn sign_out_locked_never_deletes_another_accounts_session_or_recovery_item() {
        let server = MemoryServer::new();
        let secrets = crate::keychain::MemorySecrets::new();
        let alice_device = account_device();
        let alice_password = Password::new("a sufficiently long password", "alice").unwrap();
        let alice = sign_up(
            &server,
            &secrets,
            "https://a.example",
            "alice",
            &alice_password,
            &alice_device,
            &|_| {},
        )
        .unwrap();
        crate::keychain::save_session(&secrets, &alice_device.library_id, &alice.session).unwrap();

        let bob_device =
            DeviceIdentity::from_parts("l_22222222222222222222222222222222", "Bob's Mac", [22; 32]);
        let bob_password = Password::new("another sufficiently long password", "bob").unwrap();
        let bob = sign_up(
            &server,
            &crate::keychain::MemorySecrets::new(),
            "https://a.example",
            "bob",
            &bob_password,
            &bob_device,
            &|_| {},
        )
        .unwrap();
        let mut locked_bob = LockedSession::new(
            bob.session.server.clone(),
            bob.session.origin.clone(),
            bob.session.account_id.clone(),
            bob.session.username.clone(),
            bob.session.device_id.clone(),
            bob.session.token().clone(),
            bob.session.key_id(),
            "0123456789abcdef".into(),
            None,
            None,
        )
        .with_login_device(LoginDevice::for_identity(
            &bob_device,
            &bob.session.origin,
            &bob.session.account_id,
        ));
        locked_bob.set_library_id(&alice_device.library_id);
        crate::keychain::save_locked_pending(&secrets, &alice_device.library_id, &locked_bob)
            .unwrap();

        sign_out_locked(&server, &secrets, &locked_bob).unwrap();

        assert!(
            crate::keychain::load_locked_pending(&secrets, &alice_device.library_id)
                .unwrap()
                .is_none()
        );
        assert!(
            crate::keychain::load_session(
                &secrets,
                &alice_device.library_id,
                &alice.session.origin,
                &alice.session.account_id,
            )
            .unwrap()
            .is_some()
        );
        assert!(
            crate::keychain::load_any_recovery_key(&secrets, &alice_device.library_id)
                .unwrap()
                .is_some()
        );
    }

    #[test]
    fn cancelling_a_locked_session_deletes_it_and_queues_failed_logout() {
        let server = MemoryServer::new();
        let secrets = crate::keychain::MemorySecrets::new();
        let device = account_device();
        let password = Password::new("a sufficiently long password", "alice").unwrap();
        let signed_up = sign_up(
            &server,
            &secrets,
            "https://a.example",
            "alice",
            &password,
            &device,
            &|_| {},
        )
        .unwrap();
        let mut locked = LockedSession::new(
            signed_up.session.server.clone(),
            signed_up.session.origin.clone(),
            signed_up.session.account_id.clone(),
            signed_up.session.username.clone(),
            signed_up.session.device_id.clone(),
            signed_up.session.token().clone(),
            signed_up.session.key_id(),
            "0123456789abcdef".into(),
            None,
            None,
        );
        let login_device = LoginDevice::for_identity(&device, &locked.origin, &locked.account_id);
        locked = locked.with_login_device(login_device);
        locked.set_library_id(&device.library_id);
        crate::keychain::save_pending_session(&secrets, &device.library_id, &locked).unwrap();
        server.lose_next_requests(Route::Logout, 1);

        discard_locked_session(&server, &secrets, &locked).unwrap();

        assert!(
            crate::keychain::load_pending_session(
                &secrets,
                &device.library_id,
                &locked.origin,
                &locked.account_id,
            )
            .unwrap()
            .is_none()
        );
        let pending = crate::keychain::pending_logouts(&secrets, &device.library_id).unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].server, locked.server);
        assert_eq!(pending[0].token, *locked.token());
    }

    #[test]
    fn sign_out_also_deletes_a_pending_session() {
        let server = MemoryServer::new();
        let secrets = crate::keychain::MemorySecrets::new();
        let device = account_device();
        let password = Password::new("a sufficiently long password", "alice").unwrap();
        let signed_up = sign_up(
            &server,
            &secrets,
            "https://a.example",
            "alice",
            &password,
            &device,
            &|_| {},
        )
        .unwrap();
        let mut locked = LockedSession::new(
            signed_up.session.server.clone(),
            signed_up.session.origin.clone(),
            signed_up.session.account_id.clone(),
            signed_up.session.username.clone(),
            signed_up.session.device_id.clone(),
            signed_up.session.token().clone(),
            signed_up.session.key_id(),
            "0123456789abcdef".into(),
            None,
            None,
        );
        let login_device = LoginDevice::for_identity(&device, &locked.origin, &locked.account_id);
        locked = locked.with_login_device(login_device);
        locked.set_library_id(&device.library_id);
        crate::keychain::save_pending_session(&secrets, &device.library_id, &locked).unwrap();

        sign_out(&server, &secrets, &signed_up.session).unwrap();

        assert!(
            crate::keychain::load_pending_session(
                &secrets,
                &device.library_id,
                &locked.origin,
                &locked.account_id,
            )
            .unwrap()
            .is_none()
        );
    }
}
