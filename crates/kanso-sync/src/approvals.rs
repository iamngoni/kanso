//! Device approval, as the app runs it (docs/ACCOUNTS.md › Unlocking a new
//! device, › Account flows): the new device's request, poll, name and
//! unlock, and the approver's list, challenge, check and approval. The
//! cryptography is [`crate::approval`]; these functions only talk to the
//! server and check what it says.
//!
//! The server sees both public keys, the commitment, the sealed name and
//! the sealed data key, and can compute none of the code, the name or the
//! key. The approver checks the revealed key against the commitment itself,
//! and the new device checks the approver's key and both device IDs when it
//! unwraps. Saving the Keychain items and binding are the app's flow.

use std::fmt;

use crate::account::{AccountError, DeviceInfo, Session, SignedIn, checked_server, retrying};
use crate::api::{
    ApprovalApproveRequest, ApprovalChallengeRequest, ApprovalCreateRequest, ApprovalNameRequest,
    ApprovalPoll, ApprovalRow, Backend, DeviceNameRequest, LoginDevice, PairRequest, WireDevice,
};
use crate::approval::{Approval, ApprovalKeys, Side, epk_text, matches_commitment, parse_epk};
use crate::binding::DeviceIdentity;
use crate::crypto::{ContentKeys, DataKey, b64, key_id, random};
use crate::keychain::{self, SecretStore};
use crate::wire::{
    ApiError, PROTOCOL_VERSION, SessionToken, is_approval_id, is_device_id, is_sealed_name,
};
use zeroize::Zeroizing;

/// How long the new device waits for an answer before it puts the recovery
/// key first.
pub const NO_ANSWER_AFTER_MS: i64 = 120_000;

/// How often the new device polls its request while it waits (its five
/// minutes, at most three requests, fit `approval-poll`'s 900 an hour).
pub const POLL_EVERY_MS: u64 = 2_000;

/// Approval state needed to continue one request after relaunch. The
/// requester secret is retained only in zeroizing private storage; it is
/// never formatted or returned by an accessor.
#[derive(Clone, PartialEq, Eq)]
pub struct ApprovalResume {
    pub(crate) request_id: String,
    requester_secret: Zeroizing<[u8; 32]>,
    pub(crate) expires_at: i64,
    pub(crate) other_devices: usize,
    pub(crate) approver_epk: Option<[u8; 32]>,
    pub(crate) approver_device_id: Option<String>,
    pub(crate) sealed_name: Option<String>,
}

impl ApprovalResume {
    pub(crate) fn new(
        request_id: String,
        keys: &ApprovalKeys,
        expires_at: i64,
        other_devices: usize,
    ) -> ApprovalResume {
        ApprovalResume {
            request_id,
            requester_secret: Zeroizing::new(keys.secret_bytes()),
            expires_at,
            other_devices,
            approver_epk: None,
            approver_device_id: None,
            sealed_name: None,
        }
    }

    pub(crate) fn from_parts(
        request_id: String,
        requester_secret: [u8; 32],
        expires_at: i64,
        other_devices: usize,
        approver_epk: Option<[u8; 32]>,
        approver_device_id: Option<String>,
        sealed_name: Option<String>,
    ) -> ApprovalResume {
        ApprovalResume {
            request_id,
            requester_secret: Zeroizing::new(requester_secret),
            expires_at,
            other_devices,
            approver_epk,
            approver_device_id,
            sealed_name,
        }
    }

    /// The stable request ID, reused after a lost response or relaunch.
    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    /// The server expiry, or zero while the first create answer is pending.
    pub fn expires_at(&self) -> i64 {
        self.expires_at
    }

    /// The server's count of other active devices when this request was made.
    pub fn other_devices(&self) -> usize {
        self.other_devices
    }

    /// The requester's public key, recomputed from the private key.
    pub fn requester_epk(&self) -> [u8; 32] {
        self.keys().epk()
    }

    /// The approver's public key once a challenge has been observed.
    pub fn approver_epk(&self) -> Option<[u8; 32]> {
        self.approver_epk
    }

    /// The approver's device ID once a challenge has been observed.
    pub fn approver_device_id(&self) -> Option<&str> {
        self.approver_device_id.as_deref()
    }

    /// The exact ciphertext already sent to `/name`, if any.
    pub fn sealed_name(&self) -> Option<&str> {
        self.sealed_name.as_deref()
    }

    pub(crate) fn keys(&self) -> ApprovalKeys {
        ApprovalKeys::from_secret(*self.requester_secret)
    }

    pub(crate) fn requester_secret(&self) -> &[u8; 32] {
        &self.requester_secret
    }

    pub(crate) fn matches_keys(&self, keys: &ApprovalKeys) -> bool {
        self.requester_secret.as_ref() == keys.secret_bytes()
    }
}

impl fmt::Debug for ApprovalResume {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ApprovalResume")
            .field("request_id", &self.request_id)
            .field("expires_at", &self.expires_at)
            .field("other_devices", &self.other_devices)
            .field("approver_epk", &self.approver_epk.map(|_| "[present]"))
            .field("approver_device_id", &self.approver_device_id)
            .field(
                "sealed_name",
                &self.sealed_name.as_ref().map(|_| "[redacted]"),
            )
            .field("requester_secret", &"[redacted]")
            .finish()
    }
}

/// A device that has a session but not the data key (a passkey without PRF
/// or a provider sign-in). It can request approval and sign out; it can't
/// sync. The token is private and zeroized on drop; `Debug` doesn't show it.
#[derive(Clone)]
pub struct LockedSession {
    pub server: String,
    pub origin: String,
    pub account_id: String,
    pub username: String,
    pub device_id: String,
    /// The data key's ID the server holds, checked after unwrapping.
    pub key_id: String,
    pub epoch: String,
    /// Whether the account has a password wrap available for password unlock.
    pub has_password: bool,
    /// The password wrap returned by a sign-in, when the account has a password.
    /// It is ciphertext; the pending item may use it to offer password unlock.
    pub wrapped_key: Option<String>,
    /// A device the sign-in signed out to make room, for the notice after unlock.
    pub evicted: Option<DeviceInfo>,
    // Kept separately so the ciphertext never appears in Debug and can be
    // opened once the data key is available.
    evicted_sealed_name: Option<String>,
    // The sign-in proof must survive until password unlock, including when a
    // provider used an ephemeral device ID unrelated to this library.
    login_device: Option<LoginDevice>,
    evicted_wire_device: Option<WireDevice>,
    approval_resume: Option<ApprovalResume>,
    token: SessionToken,
    library_id: Option<String>,
}

impl LockedSession {
    /// A locked session from its parts: a sign-in that returned no usable
    /// wrap, or the pending Keychain item on launch.
    #[allow(clippy::too_many_arguments, reason = "the session's parts")]
    pub fn new(
        server: String,
        origin: String,
        account_id: String,
        username: String,
        device_id: String,
        token: SessionToken,
        key_id: String,
        epoch: String,
        wrapped_key: Option<String>,
        evicted: Option<DeviceInfo>,
    ) -> LockedSession {
        LockedSession {
            server,
            origin,
            account_id,
            username,
            device_id,
            key_id,
            epoch,
            has_password: false,
            wrapped_key,
            evicted,
            evicted_sealed_name: None,
            login_device: None,
            evicted_wire_device: None,
            approval_resume: None,
            token,
            library_id: None,
        }
    }

    pub(crate) fn token(&self) -> &SessionToken {
        &self.token
    }

    pub fn set_library_id(&mut self, library_id: &str) {
        self.library_id = Some(library_id.to_owned());
    }

    /// The library whose Keychain items hold this session.
    pub fn library_id(&self) -> Option<&str> {
        self.library_id.as_deref()
    }

    /// Replaces this locked session's token with an unlocked session token
    /// after confirming that the unlocked session belongs to the same
    /// library, server origin, account, username, device, and data key.
    pub fn adopt_unlocked_session(&mut self, unlocked: &Session) -> Result<(), AccountError> {
        if self.library_id() != unlocked.library_id()
            || self.server != unlocked.server
            || self.origin != unlocked.origin
            || self.account_id != unlocked.account_id
            || self.username != unlocked.username
            || self.device_id != unlocked.device_id
            || self.key_id != unlocked.key_id()
        {
            return Err(AccountError::KeyMismatch);
        }
        self.token = unlocked.token().clone();
        Ok(())
    }

    /// Records whether this account can be unlocked with its password.
    pub fn with_has_password(mut self, has_password: bool) -> LockedSession {
        self.has_password = has_password;
        self
    }

    /// Carries the original server ciphertext through a locked-session
    /// round trip without exposing it in Debug.
    pub fn with_evicted_sealed_name(mut self, sealed_name: Option<String>) -> LockedSession {
        self.evicted_sealed_name = sealed_name;
        self
    }

    /// Retains the proof used by the provider or passkey sign-in. The proof
    /// stays private, is redacted in `Debug`, and is zeroized when dropped.
    pub fn with_login_device(mut self, login_device: LoginDevice) -> LockedSession {
        self.login_device = Some(login_device);
        self
    }

    /// Retains the server's raw evicted-device response until a data key is
    /// available to open its sealed name.
    pub fn with_evicted_wire_device(mut self, device: Option<WireDevice>) -> LockedSession {
        self.evicted = device.as_ref().map(|device| DeviceInfo {
            id: device.id.clone(),
            name: None,
            created_at: 0,
            last_seen_at: 0,
            current: false,
        });
        self.evicted_sealed_name = device
            .as_ref()
            .and_then(|device| device.sealed_name.clone());
        self.evicted_wire_device = device;
        self
    }

    /// Retains an approval request for an idempotent create/poll after relaunch.
    pub fn with_approval_resume(mut self, resume: ApprovalResume) -> LockedSession {
        self.approval_resume = Some(resume);
        self
    }

    /// The pending approval metadata, if this locked session has a resumable request.
    pub fn approval_resume(&self) -> Option<&ApprovalResume> {
        self.approval_resume.as_ref()
    }

    /// The original raw eviction answer, including its sealed name.
    pub fn evicted_wire_device(&self) -> Option<&WireDevice> {
        self.evicted_wire_device.as_ref()
    }

    pub(crate) fn login_device(&self) -> Option<&LoginDevice> {
        self.login_device.as_ref()
    }

    pub(crate) fn evicted_sealed_name(&self) -> Option<&str> {
        self.evicted_sealed_name.as_deref()
    }

    pub(crate) fn approval_resume_mut(&mut self) -> Option<&mut ApprovalResume> {
        self.approval_resume.as_mut()
    }
}

impl fmt::Debug for LockedSession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LockedSession")
            .field("server", &self.server)
            .field("origin", &self.origin)
            .field("account_id", &self.account_id)
            .field("username", &self.username)
            .field("device_id", &self.device_id)
            .field("key_id", &self.key_id)
            .field("epoch", &self.epoch)
            .field("has_password", &self.has_password)
            .field(
                "wrapped_key",
                &self.wrapped_key.as_ref().map(|_| "[redacted]"),
            )
            .field("evicted", &self.evicted.as_ref().map(|device| &device.id))
            .field(
                "device_proof",
                &self.login_device.as_ref().map(|_| "[redacted]"),
            )
            .field(
                "evicted_sealed_name",
                &self.evicted_sealed_name.as_ref().map(|_| "[redacted]"),
            )
            .field("approval_resume", &self.approval_resume)
            .field("library_id", &self.library_id)
            .finish_non_exhaustive()
    }
}

/// Opens a locked server session with this library's already available key.
/// No network request is made; the origin, account and data-key ID must all
/// match before the locked server session is resumed.
pub fn unlock_with_local_key(
    locked: &LockedSession,
    local: &Session,
) -> Result<SignedIn, AccountError> {
    if locked.origin != local.origin
        || locked.account_id != local.account_id
        || !is_device_id(&locked.device_id)
        || key_id(local.data_key()) != locked.key_id
        || locked
            .library_id()
            .is_some_and(|id| local.library_id() != Some(id))
    {
        return Err(AccountError::KeyMismatch);
    }
    let mut session = Session::new(
        locked.server.clone(),
        locked.origin.clone(),
        locked.account_id.clone(),
        locked.username.clone(),
        locked.device_id.clone(),
        locked.token().clone(),
        DataKey::from_bytes(*local.data_key().bytes()),
    );
    if let Some(library_id) = locked.library_id() {
        session.set_library_id(library_id);
    } else if let Some(library_id) = local.library_id() {
        session.set_library_id(library_id);
    }
    if let Some(device) = locked.login_device() {
        session.set_login_device(device.clone());
    }
    let content = ContentKeys::derive(local.data_key());
    let evicted = opened_evicted(locked, &content);
    Ok(SignedIn { session, evicted })
}

/// The path of `server`'s account event socket, from its health check, or
/// `None` when it doesn't advertise one (an older server): the app then uses
/// the GET fallbacks only.
pub fn account_events_path(b: &dyn Backend, server: &str) -> Result<Option<String>, AccountError> {
    let (server, _) = checked_server(server)?;
    let health = b.api(&server).health()?;
    if !health.ok || health.protocol != PROTOCOL_VERSION {
        return Err(AccountError::ServerOutdated);
    }
    Ok(health
        .events
        .and_then(|events| events.account)
        .filter(|path| is_event_path(path)))
}

/// A path on the same server: `/` and then letters, digits, `-`, `_` and `/`
/// only, so it can't name another host, a query or a fragment.
fn is_event_path(path: &str) -> bool {
    path.starts_with('/')
        && !path.starts_with("//")
        && path.len() <= 128
        && path
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'/' | b'-' | b'_'))
}

// ---------------------------------------------------------------------------
// The new device
// ---------------------------------------------------------------------------

/// A request this device made: its ID, its own public key, when it expires
/// (ms) and how many other devices could answer it when it was made.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApprovalRequest {
    pub id: String,
    pub epk: [u8; 32],
    pub expires_at: i64,
    pub other_devices: usize,
}

/// Generates the canonical 32-byte request ID that the caller passes to
/// [`request_approval_resumable`] and reuses until that request ends.
pub fn new_approval_request_id() -> String {
    b64(&random::<32>())
}

/// Consumes a screenless-device code and creates the matching locked session
/// and approval request. The one-time endpoint is deliberately not retried:
/// after a lost answer the code may already have been consumed, so the user
/// must request a fresh code rather than risk creating a second device.
pub fn sign_in_with_pairing(
    b: &dyn Backend,
    server: &str,
    code: &str,
) -> Result<(LockedSession, ApprovalRequest, ApprovalKeys), AccountError> {
    let (server, origin) = checked_server(server)?;
    let device = LoginDevice::ephemeral();
    let keys = ApprovalKeys::generate();
    let request_id = new_approval_request_id();
    let request = PairRequest {
        code: code.to_owned(),
        device: device.clone(),
        request_id: request_id.clone(),
        commit: keys.commitment(),
    };
    let paired = b.api(&server).pair(&request)?;
    if paired.approval_id != request_id || !is_approval_id(&paired.approval_id) {
        return Err(AccountError::Protocol(
            "the pairing answer changed its approval request ID".into(),
        ));
    }
    let provider = paired.session;
    let locked = LockedSession::new(
        server,
        origin,
        provider.account_id,
        provider.username,
        device.id.clone(),
        provider.token,
        provider.key_id,
        provider.epoch,
        provider.passkey_wrapped_key,
        None,
    )
    .with_has_password(provider.has_password)
    .with_login_device(device)
    .with_evicted_wire_device(provider.evicted_device)
    .with_approval_resume(ApprovalResume::new(
        request_id.clone(),
        &keys,
        paired.expires_at,
        usize::try_from(paired.other_devices).unwrap_or(usize::MAX),
    ));
    let approval = ApprovalRequest {
        id: request_id,
        epk: keys.epk(),
        expires_at: paired.expires_at,
        other_devices: usize::try_from(paired.other_devices).unwrap_or(usize::MAX),
    };
    Ok((locked, approval, keys))
}

/// The approver's answer, as the new device's poll returns it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Approved {
    pub approver_epk: [u8; 32],
    pub approver_device_id: String,
    pub wrapped_key: String,
}

/// Where a request is, from the new device's poll.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RequestState {
    /// No device has answered yet.
    Waiting,
    /// A device answered with its key: show the code, then post the name.
    Challenged {
        approver_epk: [u8; 32],
        approver_device_id: String,
    },
    /// The data key is on its way: open it with [`finish_unlock`].
    Approved(Approved),
    Cancelled,
    Expired,
}

/// Asks the account's other devices to approve this one: posts the
/// commitment to `keys`' public key, which is revealed only after a device
/// answers. `other_devices` is 0 when no other device is signed in; the app
/// then offers the recovery key at once.
pub fn request_approval(
    b: &dyn Backend,
    locked: &LockedSession,
    keys: &ApprovalKeys,
) -> Result<ApprovalRequest, AccountError> {
    let api = b.api(&locked.server);
    let request = ApprovalCreateRequest {
        request_id: new_approval_request_id(),
        commit: keys.commitment(),
    };
    // The stable caller ID makes an in-process resend safe even when the
    // Worker's answer was lost. Relaunch safety requires the persisted API below.
    let created = retrying(b, || api.create_approval(locked.token(), &request))?;
    if !is_approval_id(&created.id) || created.id != request.request_id {
        return Err(AccountError::Protocol(
            "the approval create answer changed its request ID".into(),
        ));
    }
    Ok(ApprovalRequest {
        id: created.id,
        epk: keys.epk(),
        expires_at: created.expires_at,
        other_devices: usize::try_from(created.other_devices).unwrap_or(usize::MAX),
    })
}

/// Persists the caller's stable request ID and X25519 secret before sending
/// create. If the answer is lost, call this again with the same ID and keys,
/// or [`resume_approval`] with the loaded locked session.
pub fn request_approval_resumable(
    b: &dyn Backend,
    secrets: &dyn SecretStore,
    locked: &LockedSession,
    request_id: &str,
    keys: &ApprovalKeys,
) -> Result<ApprovalRequest, AccountError> {
    if !is_approval_id(request_id) {
        return Err(AccountError::BadRequest);
    }
    let library_id = locked.library_id().ok_or_else(|| {
        AccountError::Protocol("the locked session has no library ID for Keychain storage".into())
    })?;
    let persisted = keychain::load_locked_pending(secrets, library_id)?;
    let mut pending = match persisted {
        Some(saved) if same_locked_session(&saved, locked) => saved,
        Some(_) => return Err(AccountError::KeyMismatch),
        None => locked.clone(),
    };
    if pending.approval_resume.is_none()
        && let Some(resume) = locked.approval_resume()
    {
        pending.approval_resume = Some(resume.clone());
    }
    match pending.approval_resume.as_ref() {
        Some(saved) if saved.request_id == request_id && saved.matches_keys(keys) => {}
        Some(_) => return Err(AccountError::ApprovalChanged),
        None => {
            pending.approval_resume = Some(ApprovalResume::new(request_id.to_owned(), keys, 0, 0));
        }
    }
    persist_locked_pending(secrets, &pending)?;
    create_saved_request(b, secrets, &pending)
}

/// Recreates the same key pair and request ID from persisted state, then
/// replays create idempotently. It never generates replacement state.
pub fn resume_approval(
    b: &dyn Backend,
    secrets: &dyn SecretStore,
    locked: &LockedSession,
) -> Result<ApprovalRequest, AccountError> {
    if locked.approval_resume.is_none() {
        return Err(AccountError::Protocol(
            "there is no saved approval request to resume".into(),
        ));
    }
    create_saved_request(b, secrets, locked)
}

fn create_saved_request(
    b: &dyn Backend,
    secrets: &dyn SecretStore,
    locked: &LockedSession,
) -> Result<ApprovalRequest, AccountError> {
    let locked = saved_locked_pending(secrets, locked)?;
    let resume = locked
        .approval_resume()
        .ok_or_else(|| AccountError::Protocol("there is no saved approval request".into()))?;
    let keys = resume.keys();
    let request = ApprovalCreateRequest {
        request_id: resume.request_id.clone(),
        commit: keys.commitment(),
    };
    let api = b.api(&locked.server);
    let created = retrying(b, || api.create_approval(locked.token(), &request))?;
    if !is_approval_id(&created.id) || created.id != request.request_id || created.expires_at <= 0 {
        return Err(AccountError::Protocol(
            "the approval create answer doesn't match its saved request".into(),
        ));
    }
    let mut updated = locked;
    let saved = updated
        .approval_resume_mut()
        .ok_or_else(|| AccountError::Protocol("there is no saved approval request".into()))?;
    saved.expires_at = created.expires_at;
    saved.other_devices = usize::try_from(created.other_devices).unwrap_or(usize::MAX);
    let request = approval_request(saved);
    persist_locked_pending(secrets, &updated)?;
    Ok(request)
}

fn approval_request(resume: &ApprovalResume) -> ApprovalRequest {
    ApprovalRequest {
        id: resume.request_id.clone(),
        epk: resume.requester_epk(),
        expires_at: resume.expires_at,
        other_devices: resume.other_devices,
    }
}

fn persist_locked_pending(
    secrets: &dyn SecretStore,
    locked: &LockedSession,
) -> Result<(), AccountError> {
    let library_id = locked.library_id().ok_or_else(|| {
        AccountError::Protocol("the locked session has no library ID for Keychain storage".into())
    })?;
    if keychain::load_locked_pending(secrets, library_id)?.is_some() {
        keychain::update_locked_pending(secrets, library_id, locked)?;
    } else {
        keychain::save_locked_pending(secrets, library_id, locked)?;
    }
    Ok(())
}

fn same_locked_session(left: &LockedSession, right: &LockedSession) -> bool {
    left.server == right.server
        && left.origin == right.origin
        && left.account_id == right.account_id
        && left.username == right.username
        && left.device_id == right.device_id
        && left.key_id == right.key_id
        && left.epoch == right.epoch
        && left.token() == right.token()
        && left.library_id() == right.library_id()
        && left.login_device() == right.login_device()
}

/// The locked session as the secret store holds it, which is where a request
/// made by [`request_approval_resumable`] keeps its keys and the approver's
/// answer. A `LockedSession` the caller already holds does not change when
/// the store does, so read the saved one before using the resume state. It
/// must be the same sign-in as `locked`.
pub fn saved_locked_pending(
    secrets: &dyn SecretStore,
    locked: &LockedSession,
) -> Result<LockedSession, AccountError> {
    let library_id = locked.library_id().ok_or_else(|| {
        AccountError::Protocol("the locked session has no library ID for Keychain storage".into())
    })?;
    let saved = keychain::load_locked_pending(secrets, library_id)?
        .ok_or_else(|| AccountError::Protocol("there is no saved locked session".into()))?;
    if !same_locked_session(&saved, locked) {
        return Err(AccountError::KeyMismatch);
    }
    Ok(saved)
}

/// The request's state, from the requester's poll.
pub fn poll_approval(
    b: &dyn Backend,
    locked: &LockedSession,
    request: &ApprovalRequest,
) -> Result<RequestState, AccountError> {
    let poll = retrying(b, || {
        b.api(&locked.server).approval(locked.token(), &request.id)
    })?;
    request_state(&poll)
}

/// Polls the saved request after relaunch, stores a newly observed challenge,
/// and clears only approval state when the server says cancelled or expired.
pub fn poll_saved_approval(
    b: &dyn Backend,
    secrets: &dyn SecretStore,
    locked: &LockedSession,
) -> Result<RequestState, AccountError> {
    let saved = saved_locked_pending(secrets, locked)?;
    let resume = saved
        .approval_resume()
        .ok_or_else(|| AccountError::Protocol("there is no saved approval request".into()))?;
    let request = approval_request(resume);
    let state = poll_approval(b, &saved, &request)?;
    match &state {
        RequestState::Challenged {
            approver_epk,
            approver_device_id,
        } => {
            let mut updated = saved.clone();
            let saved = updated.approval_resume_mut().ok_or_else(|| {
                AccountError::Protocol("there is no saved approval request".into())
            })?;
            if saved
                .approver_epk
                .is_some_and(|existing| existing != *approver_epk)
                || saved
                    .approver_device_id
                    .as_deref()
                    .is_some_and(|existing| existing != approver_device_id)
            {
                return Err(AccountError::ApprovalChanged);
            }
            saved.approver_epk = Some(*approver_epk);
            saved.approver_device_id = Some(approver_device_id.clone());
            persist_locked_pending(secrets, &updated)?;
        }
        RequestState::Cancelled | RequestState::Expired => {
            let library_id = saved.library_id().ok_or_else(|| {
                AccountError::Protocol(
                    "the locked session has no library ID for Keychain storage".into(),
                )
            })?;
            keychain::clear_pending_approval(secrets, library_id)?;
        }
        _ => {}
    }
    Ok(state)
}

fn request_state(poll: &ApprovalPoll) -> Result<RequestState, AccountError> {
    let approver = || -> Result<([u8; 32], String), AccountError> {
        let epk = poll
            .approver_epk
            .as_deref()
            .and_then(parse_epk)
            .ok_or_else(|| AccountError::Protocol("no approver key".into()))?;
        let device = poll
            .approver_device_id
            .clone()
            .filter(|id| is_device_id(id))
            .ok_or_else(|| AccountError::Protocol("no approver device".into()))?;
        Ok((epk, device))
    };
    Ok(match poll.status.as_str() {
        "waiting" => RequestState::Waiting,
        "challenged" => {
            let (approver_epk, approver_device_id) = approver()?;
            RequestState::Challenged {
                approver_epk,
                approver_device_id,
            }
        }
        "approved" => {
            let (approver_epk, approver_device_id) = approver()?;
            let wrapped_key = poll
                .wrapped_key
                .clone()
                .ok_or_else(|| AccountError::Protocol("approved without a key".into()))?;
            RequestState::Approved(Approved {
                approver_epk,
                approver_device_id,
                wrapped_key,
            })
        }
        "cancelled" => RequestState::Cancelled,
        "expired" => RequestState::Expired,
        other => return Err(AccountError::Protocol(format!("approval status {other}"))),
    })
}

/// The new device's side of the exchange, once a device has answered: the
/// approval key and the code to show.
pub fn complete_request(
    locked: &LockedSession,
    request: &ApprovalRequest,
    keys: &ApprovalKeys,
    approver_epk: &[u8; 32],
) -> Result<Approval, AccountError> {
    if keys.epk() != request.epk {
        return Err(AccountError::KeyMismatch);
    }
    Ok(keys.complete(Side::New, approver_epk, &locked.account_id, &request.id)?)
}

/// Completes the request from its saved key pair without requiring the
/// caller to retain or reconstruct requester secret material.
pub fn complete_saved_request(
    locked: &LockedSession,
    approver_epk: &[u8; 32],
) -> Result<Approval, AccountError> {
    let resume = locked
        .approval_resume()
        .ok_or_else(|| AccountError::Protocol("there is no saved approval request".into()))?;
    let keys = resume.keys();
    complete_request(locked, &approval_request(resume), &keys, approver_epk)
}

/// Reveals this device's key and posts its name sealed for the approver
/// (`device.name`). A repeat of the identical body is answered 200, so a lost
/// answer is sent again.
pub fn name_device(
    b: &dyn Backend,
    locked: &LockedSession,
    request: &ApprovalRequest,
    approval: &Approval,
    device: &DeviceIdentity,
) -> Result<(), AccountError> {
    let body = ApprovalNameRequest {
        epk: epk_text(&request.epk),
        sealed_name: approval.seal_name(&device.name),
    };
    retrying(b, || {
        b.api(&locked.server)
            .name_approval(locked.token(), &request.id, &body)
    })?;
    Ok(())
}

/// Seals the new device name once, persists that exact ciphertext before
/// `/name`, then reuses it on every retry or relaunch.
pub fn name_device_resumable(
    b: &dyn Backend,
    secrets: &dyn SecretStore,
    locked: &LockedSession,
    device: &DeviceIdentity,
) -> Result<(), AccountError> {
    let mut updated = saved_locked_pending(secrets, locked)?;
    let (request_id, body) = {
        let resume = updated
            .approval_resume_mut()
            .ok_or_else(|| AccountError::Protocol("there is no saved approval request".into()))?;
        let approver_epk = resume.approver_epk.ok_or_else(|| {
            AccountError::Protocol("the approval request has no challenge".into())
        })?;
        if resume
            .approver_device_id
            .as_deref()
            .is_none_or(|id| !is_device_id(id))
        {
            return Err(AccountError::Protocol(
                "the approval request has no approver device".into(),
            ));
        }
        let keys = resume.keys();
        let approval = keys.complete(
            Side::New,
            &approver_epk,
            &locked.account_id,
            &resume.request_id,
        )?;
        let sealed_name = resume
            .sealed_name
            .clone()
            .unwrap_or_else(|| approval.seal_name(&device.name));
        if resume.sealed_name.is_none() {
            resume.sealed_name = Some(sealed_name.clone());
        }
        (
            resume.request_id.clone(),
            ApprovalNameRequest {
                epk: epk_text(&keys.epk()),
                sealed_name,
            },
        )
    };
    persist_locked_pending(secrets, &updated)?;
    let result = retrying(b, || {
        b.api(&locked.server)
            .name_approval(locked.token(), &request_id, &body)
    })
    .map_err(AccountError::from);
    match result {
        Ok(()) => Ok(()),
        Err(error @ (AccountError::ApprovalExpired | AccountError::NotFound)) => {
            let library_id = locked.library_id().ok_or_else(|| {
                AccountError::Protocol(
                    "the locked session has no library ID for Keychain storage".into(),
                )
            })?;
            keychain::clear_pending_approval(secrets, library_id)?;
            Err(error)
        }
        Err(error) => Err(error),
    }
}

/// Opens the data key the approver sealed to this device and checks it:
/// the approver's key must be the one the code was derived from, the wrap
/// must open for this device and that approver, and its key ID must be the
/// server's. Sets the device name, and returns the session. Nothing is
/// saved; the caller saves `sync-account`, deletes the pending item, then
/// binds.
pub fn finish_unlock(
    b: &dyn Backend,
    locked: &LockedSession,
    approval: &Approval,
    approved: &Approved,
    device: &DeviceIdentity,
) -> Result<SignedIn, AccountError> {
    if approved.approver_epk != approval.epk_approver() {
        return Err(AccountError::ApprovalChanged);
    }
    let data_key = approval.unwrap(
        &locked.device_id,
        &approved.approver_device_id,
        &approved.wrapped_key,
    )?;
    if key_id(&data_key) != locked.key_id {
        return Err(AccountError::KeyMismatch);
    }
    let content = ContentKeys::derive(&data_key);
    // The name is cosmetic ("A Mac" until set); a failure doesn't undo the unlock.
    let _ = b.api(&locked.server).set_device_name(
        locked.token(),
        &DeviceNameRequest {
            sealed_name: content.seal_name(&locked.account_id, &locked.device_id, &device.name),
        },
    );
    let mut session = Session::new(
        locked.server.clone(),
        locked.origin.clone(),
        locked.account_id.clone(),
        locked.username.clone(),
        locked.device_id.clone(),
        locked.token().clone(),
        data_key,
    );
    if let Some(library_id) = locked.library_id() {
        session.set_library_id(library_id);
    }
    if let Some(device) = locked.login_device() {
        session.set_login_device(device.clone());
    }
    let evicted = opened_evicted(locked, &content);
    Ok(SignedIn { session, evicted })
}

pub(crate) fn opened_evicted(locked: &LockedSession, content: &ContentKeys) -> Option<DeviceInfo> {
    let raw = locked.evicted_wire_device.as_ref();
    let existing = locked.evicted.as_ref();
    let (device_id, base) = match (raw, existing) {
        (Some(raw), _) => (raw.id.as_str(), existing),
        (None, Some(existing)) => (existing.id.as_str(), Some(existing)),
        (None, None) => return None,
    };
    let sealed_name = raw
        .and_then(|device| device.sealed_name.as_deref())
        .or(locked.evicted_sealed_name.as_deref());
    let name = sealed_name.and_then(|sealed| {
        match content.open_name(&locked.account_id, device_id, sealed) {
            crate::crypto::Opened::Opened(name) => Some(name),
            crate::crypto::Opened::Undecryptable | crate::crypto::Opened::TooNew => None,
        }
    });
    Some(DeviceInfo {
        id: device_id.to_owned(),
        name,
        created_at: base.map_or(0, |device| device.created_at),
        last_seen_at: base.map_or(0, |device| device.last_seen_at),
        current: false,
    })
}

/// Cancels this device's request (also when its codes didn't match). A
/// request that is already gone counts as cancelled.
pub fn cancel_request(
    b: &dyn Backend,
    locked: &LockedSession,
    request_id: &str,
) -> Result<(), AccountError> {
    match retrying(b, || {
        b.api(&locked.server)
            .cancel_approval(locked.token(), request_id)
    }) {
        Ok(()) | Err(ApiError::NotFound) => Ok(()),
        Err(err) => Err(err.into()),
    }
}

/// Cancels the saved request and removes only its approval resume data. The
/// locked session and token remain available for password or recovery unlock.
pub fn cancel_saved_approval(
    b: &dyn Backend,
    secrets: &dyn SecretStore,
    locked: &LockedSession,
) -> Result<(), AccountError> {
    let saved = saved_locked_pending(secrets, locked)?;
    let resume = saved
        .approval_resume()
        .ok_or_else(|| AccountError::Protocol("there is no saved approval request".into()))?;
    cancel_request(b, &saved, &resume.request_id)?;
    let library_id = saved.library_id().ok_or_else(|| {
        AccountError::Protocol("the locked session has no library ID for Keychain storage".into())
    })?;
    keychain::clear_pending_approval(secrets, library_id)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// The approver
// ---------------------------------------------------------------------------

/// A request another device of the account made, as the approver lists it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingApproval {
    pub id: String,
    /// The device asking.
    pub device_id: String,
    pub commit: String,
    /// The requester's key, once revealed (after a challenge).
    pub epk: Option<[u8; 32]>,
    pub sealed_name: Option<String>,
    pub approver_device_id: Option<String>,
    pub approver_epk: Option<[u8; 32]>,
    pub created_at: i64,
    pub expires_at: i64,
}

impl PendingApproval {
    fn from_row(row: ApprovalRow) -> Option<PendingApproval> {
        let key = |text: Option<String>| -> Option<Option<[u8; 32]>> {
            match text {
                None => Some(None),
                Some(text) => parse_epk(&text).map(Some),
            }
        };
        if !is_approval_id(&row.id) || !is_device_id(&row.device_id) || !is_approval_id(&row.commit)
        {
            return None;
        }
        if row
            .sealed_name
            .as_deref()
            .is_some_and(|s| !is_sealed_name(s))
        {
            return None;
        }
        Some(PendingApproval {
            epk: key(row.epk)?,
            approver_epk: key(row.approver_epk)?,
            id: row.id,
            device_id: row.device_id,
            commit: row.commit,
            sealed_name: row.sealed_name,
            approver_device_id: row.approver_device_id,
            created_at: row.created_at,
            expires_at: row.expires_at,
        })
    }
}

/// The account's pending requests from other devices, oldest first: the GET
/// that runs at launch, when Settings › Account opens, on foreground, after
/// every socket connect and after every event. Rows of the wrong shape are
/// left out; this device's own requests never appear.
pub fn pending_approvals(
    b: &dyn Backend,
    session: &Session,
) -> Result<Vec<PendingApproval>, AccountError> {
    let list = retrying(b, || b.api(&session.server).approvals(session.token()))?;
    Ok(list
        .approvals
        .into_iter()
        .filter_map(PendingApproval::from_row)
        .filter(|p| p.device_id != session.device_id)
        .collect())
}

/// Answers a request with this device's ephemeral key. A repeat with the
/// same key is answered 200, so a lost answer is sent again.
pub fn challenge_request(
    b: &dyn Backend,
    session: &Session,
    pending: &PendingApproval,
    keys: &ApprovalKeys,
) -> Result<(), AccountError> {
    if pending.device_id == session.device_id {
        return Err(AccountError::SelfApproval);
    }
    let body = ApprovalChallengeRequest {
        epk: epk_text(&keys.epk()),
    };
    retrying(b, || {
        b.api(&session.server)
            .challenge_approval(session.token(), &pending.id, &body)
    })?;
    Ok(())
}

/// What the approver saw of a request when it made its ephemeral key for it:
/// the request, the device asking and the requester's commitment. It is
/// taken before this device's key is posted and kept next to that key, so a
/// later read can't swap in a commitment chosen after the key was seen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PinnedRequest {
    pub id: String,
    pub device_id: String,
    pub commit: String,
}

impl PinnedRequest {
    /// Pins `pending` as it reads now. Call it when the key for the request
    /// is generated, never after the key has been posted.
    pub fn of(pending: &PendingApproval) -> PinnedRequest {
        PinnedRequest {
            id: pending.id.clone(),
            device_id: pending.device_id.clone(),
            commit: pending.commit.clone(),
        }
    }

    /// Whether `pending` is still the request that was pinned.
    pub fn matches(&self, pending: &PendingApproval) -> bool {
        self.id == pending.id
            && self.device_id == pending.device_id
            && self.commit == pending.commit
    }
}

/// What the approver shows: the device's name and, through the approval,
/// the code.
#[derive(Debug)]
pub struct OpenedRequest {
    pub approval: Approval,
    pub name: String,
}

/// Checks a named request and opens it, or `Ok(None)` while the requester
/// hasn't revealed its key yet. The revealed key must match the commitment
/// made before this device's key existed, and the name must open under the
/// key both sides derive; otherwise [`AccountError::ApprovalChanged`]
/// ("Don't approve"). The commitment and the requesting device must be the
/// ones `pinned` saw before this device's key was posted: a server that
/// learned that key could otherwise grind a key of its own whose code
/// matches, and swap in its commitment. A request another device answered is
/// [`AccountError::ApprovalUsed`].
pub fn open_request(
    session: &Session,
    pending: &PendingApproval,
    pinned: &PinnedRequest,
    keys: &ApprovalKeys,
) -> Result<Option<OpenedRequest>, AccountError> {
    if !pinned.matches(pending) {
        return Err(AccountError::ApprovalChanged);
    }
    if pending
        .approver_device_id
        .as_deref()
        .is_some_and(|id| id != session.device_id)
        || pending.approver_epk.is_some_and(|epk| epk != keys.epk())
    {
        return Err(AccountError::ApprovalUsed);
    }
    let (Some(epk), Some(sealed)) = (pending.epk, pending.sealed_name.as_deref()) else {
        return Ok(None);
    };
    if !matches_commitment(&epk, &pending.commit) {
        return Err(AccountError::ApprovalChanged);
    }
    let approval = keys.complete(Side::Approver, &epk, &session.account_id, &pending.id)?;
    let name = approval
        .open_name(sealed)
        .ok_or(AccountError::ApprovalChanged)?;
    Ok(Some(OpenedRequest { approval, name }))
}

/// Seals this device's data key to the requester and sends it. The same
/// sealed key is resent after a lost answer (the server answers a repeat
/// 200 and refuses a different one).
pub fn finish_approval(
    b: &dyn Backend,
    session: &Session,
    pending: &PendingApproval,
    approval: &Approval,
) -> Result<(), AccountError> {
    if pending.device_id == session.device_id {
        return Err(AccountError::SelfApproval);
    }
    let body = ApprovalApproveRequest {
        wrapped_key: approval.wrap(&pending.device_id, &session.device_id, session.data_key()),
    };
    retrying(b, || {
        b.api(&session.server)
            .approve_approval(session.token(), &pending.id, &body)
    })?;
    Ok(())
}

/// Declines a request (the sheet's Cancel). One that is already gone counts
/// as declined.
pub fn decline_request(
    b: &dyn Backend,
    session: &Session,
    request_id: &str,
) -> Result<(), AccountError> {
    match retrying(b, || {
        b.api(&session.server)
            .cancel_approval(session.token(), request_id)
    }) {
        Ok(()) | Err(ApiError::NotFound) => Ok(()),
        Err(err) => Err(err.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_paths_stay_on_the_server() {
        assert!(is_event_path("/v2/account/events"));
        for path in [
            "//evil.example/x",
            "https://evil.example",
            "/v2/a?b",
            "/v2/a#b",
            "v2/a",
            "",
        ] {
            assert!(!is_event_path(path), "{path}");
        }
    }

    #[test]
    fn poll_states() {
        let epk = epk_text(&[7; 32]);
        let device = "d_00000000000000000000000000000002".to_owned();
        let poll = |status: &str| ApprovalPoll {
            status: status.into(),
            expires_at: Some(1),
            approver_epk: Some(epk.clone()),
            approver_device_id: Some(device.clone()),
            sealed_name: None,
            wrapped_key: Some("w".into()),
        };
        assert_eq!(
            request_state(&poll("waiting")).unwrap(),
            RequestState::Waiting
        );
        assert!(matches!(
            request_state(&poll("challenged")).unwrap(),
            RequestState::Challenged { .. }
        ));
        assert!(matches!(
            request_state(&poll("approved")).unwrap(),
            RequestState::Approved(_)
        ));
        assert_eq!(
            request_state(&poll("expired")).unwrap(),
            RequestState::Expired
        );
        assert!(request_state(&poll("odd")).is_err());
        let mut bad = poll("challenged");
        bad.approver_epk = Some("nope".into());
        assert!(request_state(&bad).is_err());
    }

    #[test]
    fn local_key_unlock_requires_the_locked_key_id() {
        let data_key = DataKey::from_bytes([7; 32]);
        let mut local = Session::new(
            "https://a.example".into(),
            "https://a.example".into(),
            "a_0123456789abcdef0123456789abcdef".into(),
            "alice".into(),
            "d_0123456789abcdef0123456789abcdef".into(),
            SessionToken::new(format!("kanso_{}", "b".repeat(43))),
            data_key,
        );
        local.set_library_id("l_1");
        let locked = LockedSession::new(
            "https://a.example".into(),
            "https://a.example".into(),
            local.account_id.clone(),
            "alice".into(),
            "d_ffffffffffffffffffffffffffffffff".into(),
            SessionToken::new(format!("kanso_{}", "a".repeat(43))),
            local.key_id(),
            "0123456789abcdef".into(),
            None,
            None,
        )
        .with_has_password(true);

        let unlocked = unlock_with_local_key(&locked, &local).unwrap();
        assert_eq!(unlocked.session.account_id, locked.account_id);
        assert_eq!(unlocked.session.device_id, locked.device_id);
        assert_eq!(unlocked.session.key_id(), locked.key_id);
        assert_eq!(unlocked.session.token(), locked.token());
        assert_eq!(unlocked.session.library_id(), Some("l_1"));

        let wrong_key = LockedSession::new(
            locked.server.clone(),
            locked.origin.clone(),
            locked.account_id.clone(),
            locked.username.clone(),
            locked.device_id.clone(),
            locked.token().clone(),
            "0000000000000000".into(),
            locked.epoch.clone(),
            None,
            None,
        );
        assert!(matches!(
            unlock_with_local_key(&wrong_key, &local),
            Err(AccountError::KeyMismatch)
        ));

        let wrong_account = LockedSession::new(
            locked.server.clone(),
            locked.origin.clone(),
            "a_ffffffffffffffffffffffffffffffff".into(),
            locked.username.clone(),
            locked.device_id.clone(),
            locked.token().clone(),
            locked.key_id.clone(),
            locked.epoch.clone(),
            None,
            None,
        );
        assert!(matches!(
            unlock_with_local_key(&wrong_account, &local),
            Err(AccountError::KeyMismatch)
        ));
    }

    #[test]
    fn locked_evicted_name_is_opened_after_the_key_is_available() {
        let data_key = DataKey::from_bytes([9; 32]);
        let content = ContentKeys::derive(&data_key);
        let account_id = "a_0123456789abcdef0123456789abcdef";
        let evicted_id = "d_11111111111111111111111111111111";
        let sealed = content.seal_name(account_id, evicted_id, "Alex’s MacBook Air");
        let locked = LockedSession::new(
            "https://a.example".into(),
            "https://a.example".into(),
            account_id.into(),
            "alice".into(),
            "d_0123456789abcdef0123456789abcdef".into(),
            SessionToken::new(format!("kanso_{}", "a".repeat(43))),
            key_id(&data_key),
            "0123456789abcdef".into(),
            None,
            Some(DeviceInfo {
                id: evicted_id.into(),
                name: None,
                created_at: 0,
                last_seen_at: 0,
                current: false,
            }),
        )
        .with_evicted_sealed_name(Some(sealed));

        let opened = opened_evicted(&locked, &content).unwrap();
        assert_eq!(opened.name.as_deref(), Some("Alex’s MacBook Air"));
        assert!(!format!("{locked:?}").contains("Alex’s MacBook Air"));
    }

    proptest::proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig::with_cases(64))]

        /// A pin matches the row it was taken from whatever else changes, and
        /// any change to the request, the device or the commitment breaks it.
        #[test]
        fn prop_a_pin_only_matches_its_own_request(
            epk in proptest::option::of(proptest::array::uniform32(0u8..)),
            which in 0usize..3,
            other in "[a-z0-9]{1,8}",
        ) {
            let row = PendingApproval {
                id: "r_1".into(),
                device_id: "d_1".into(),
                commit: "c_1".into(),
                epk: None,
                sealed_name: None,
                approver_device_id: None,
                approver_epk: None,
                created_at: 1,
                expires_at: 2,
            };
            let pin = PinnedRequest::of(&row);
            let mut later = row.clone();
            later.epk = epk;
            later.sealed_name = Some("x".into());
            proptest::prop_assert!(pin.matches(&later));
            match which {
                0 => later.id = format!("r_{other}x"),
                1 => later.device_id = format!("d_{other}x"),
                _ => later.commit = format!("c_{other}x"),
            }
            proptest::prop_assert!(!pin.matches(&later));
        }
    }
}
