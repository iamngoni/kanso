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
    ApprovalPoll, ApprovalRow, Backend, DeviceNameRequest,
};
use crate::approval::{Approval, ApprovalKeys, Side, epk_text, matches_commitment, parse_epk};
use crate::binding::DeviceIdentity;
use crate::crypto::{ContentKeys, key_id};
use crate::wire::{
    ApiError, PROTOCOL_VERSION, SessionToken, is_approval_id, is_device_id, is_sealed_name,
};

/// How long the new device waits for an answer before it puts the recovery
/// key first.
pub const NO_ANSWER_AFTER_MS: i64 = 120_000;

/// How often the new device polls its request while it waits (its five
/// minutes, at most three requests, fit `approval-poll`'s 900 an hour).
pub const POLL_EVERY_MS: u64 = 2_000;

/// A device that has a session but not the data key (a passkey without PRF
/// or a provider sign-in). It can request approval and sign out; it can't
/// sync. The token is private and zeroized on drop; `Debug` doesn't show it.
pub struct LockedSession {
    pub server: String,
    pub origin: String,
    pub account_id: String,
    pub username: String,
    pub device_id: String,
    /// The data key's ID the server holds, checked after unwrapping.
    pub key_id: String,
    pub epoch: String,
    /// The password wrap returned by a sign-in, when the account has a password.
    /// It is ciphertext; the pending item may use it to offer password unlock.
    pub wrapped_key: Option<String>,
    /// A device the sign-in signed out to make room, for the notice after unlock.
    pub evicted: Option<DeviceInfo>,
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
            wrapped_key,
            evicted,
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
            .field(
                "wrapped_key",
                &self.wrapped_key.as_ref().map(|_| "[redacted]"),
            )
            .field("evicted", &self.evicted.as_ref().map(|device| &device.id))
            .field("library_id", &self.library_id)
            .finish_non_exhaustive()
    }
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
        commit: keys.commitment(),
    };
    // Not retried: a lost answer would make a second request, and the
    // first is cancelled when a fourth is made, or expires.
    let created = api.create_approval(locked.token(), &request)?;
    if !is_approval_id(&created.id) {
        return Err(AccountError::Protocol(
            "an approval ID of the wrong shape".into(),
        ));
    }
    Ok(ApprovalRequest {
        id: created.id,
        epk: keys.epk(),
        expires_at: created.expires_at,
        other_devices: usize::try_from(created.other_devices).unwrap_or(usize::MAX),
    })
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
    Ok(SignedIn {
        session,
        evicted: None,
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
/// ("Don't approve"). A request another device answered is
/// [`AccountError::ApprovalUsed`].
pub fn open_request(
    session: &Session,
    pending: &PendingApproval,
    keys: &ApprovalKeys,
) -> Result<Option<OpenedRequest>, AccountError> {
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
}
