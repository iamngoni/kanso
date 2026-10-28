use std::sync::OnceLock;

use base64::Engine as _;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use kanso_engine::account::approval::ApproverKey;
use kanso_sync::approval::Side;
use kanso_sync::{
    ApprovalKeys, DeviceIdentity, MemorySecrets, MemoryServer, Password, PinnedRequest,
    new_approval_request_id,
};
use proptest::prelude::*;

use super::*;

static SESSION: OnceLock<Session> = OnceLock::new();

/// A real session of the account "ada", on an in-memory server.
fn session() -> &'static Session {
    SESSION.get_or_init(|| {
        let server = MemoryServer::new();
        let device =
            DeviceIdentity::from_parts("l_00000000000000000000000000000001", "Phone", [1; 32]);
        let password = Password::new("correct horse battery", "ada").unwrap();
        kanso_sync::sign_up(
            &server,
            &MemorySecrets::new(),
            kanso_sync::DEFAULT_SERVER,
            "ada",
            &password,
            &device,
            &|_| {},
        )
        .unwrap()
        .session
    })
}

/// A request that checks out: the asking device's key and sealed name,
/// answered by this phone's device with `key`.
struct Request {
    pending: PendingApproval,
    key: ApproverKey,
    /// The code the asking device shows.
    code: String,
}

fn request(secret_new: [u8; 32], secret_approver: [u8; 32], name: &str) -> Request {
    let session = session();
    let asking = ApprovalKeys::from_secret(secret_new);
    let approver = ApprovalKeys::from_secret(secret_approver);
    let id = new_approval_request_id();
    let approval = asking
        .complete(Side::New, &approver.epk(), &session.account_id, &id)
        .unwrap();
    let mut pending = PendingApproval {
        id,
        device_id: format!("d_{}", "1".repeat(32)),
        commit: asking.commitment(),
        epk: None,
        sealed_name: None,
        approver_device_id: None,
        approver_epk: None,
        created_at: 1,
        expires_at: 2,
    };
    // Pinned before this phone's key is posted, as the first sight does.
    let key = ApproverKey {
        pin: PinnedRequest::of(&pending),
        keys: approver,
    };
    pending.approver_device_id = Some(session.device_id.clone());
    pending.approver_epk = Some(key.keys.epk());
    pending.epk = Some(asking.epk());
    pending.sealed_name = Some(approval.seal_name(name));
    Request {
        pending,
        key,
        code: approval.code(),
    }
}

/// The forms a 32-byte secret could be written in.
fn forms(secret: &[u8; 32]) -> Vec<String> {
    let hex: String = secret.iter().map(|byte| format!("{byte:02x}")).collect();
    vec![
        hex,
        URL_SAFE_NO_PAD.encode(secret),
        STANDARD.encode(secret),
        format!("{secret:?}"),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// A request is Ready only when nothing about it changed since this
    /// phone first saw it: the commitment, the revealed key, the sealed
    /// name, this phone's key, the asking device and the request itself
    /// each turn it into Changed, with no name and no code shown.
    #[test]
    fn prop_only_a_request_that_checks_out_is_ready(
        secret_new in any::<[u8; 32]>(),
        secret_approver in any::<[u8; 32]>(),
        junk in any::<[u8; 32]>(),
        how in 0u8..7,
        name in "[a-zA-Z0-9 ]{1,20}",
    ) {
        let mut request = request(secret_new, secret_approver, &name);
        let session = session();
        match how {
            0 => {}
            1 => {
                let other = ApprovalKeys::from_secret(junk).commitment();
                prop_assume!(other != request.pending.commit);
                request.pending.commit = other;
            }
            2 => {
                prop_assume!(Some(junk) != request.pending.epk);
                request.pending.epk = Some(junk);
            }
            3 => {
                // A name sealed under another exchange.
                let approver_epk = request.key.keys.epk();
                let other = ApprovalKeys::from_secret(junk)
                    .complete(Side::New, &approver_epk, &session.account_id, &request.pending.id)
                    .unwrap();
                prop_assume!(other.code() != request.code);
                request.pending.sealed_name = Some(other.seal_name(&name));
            }
            4 => {
                prop_assume!(Some(junk) != request.pending.approver_epk);
                request.pending.approver_epk = Some(junk);
            }
            5 => request.pending.device_id = format!("d_{}", "2".repeat(32)),
            _ => request.pending.id = new_approval_request_id(),
        }
        let step = front_step(session, &request.pending, &request.key, true);
        let seen = Seen::of(request.pending, step);
        if how == 0 {
            prop_assert_eq!(seen.prompt.state, PromptState::Ready);
            prop_assert_eq!(seen.prompt.code, Some(request.code));
            prop_assert_eq!(seen.prompt.device_name, Some(device_name(&name)));
            prop_assert!(seen.opened.is_some());
        } else {
            prop_assert_eq!(seen.prompt.state, PromptState::Changed);
            prop_assert_eq!(seen.prompt.code, None);
            prop_assert_eq!(seen.prompt.device_name, None);
            prop_assert!(seen.opened.is_none());
        }
    }

    /// Until the asking device has revealed its key and its name, a
    /// request is never Ready and shows neither name nor code; it is New
    /// until this phone has answered it, Challenged after.
    #[test]
    fn prop_a_request_waits_for_the_key_and_the_name(
        secret_new in any::<[u8; 32]>(),
        secret_approver in any::<[u8; 32]>(),
        has_key in any::<bool>(),
        has_name in any::<bool>(),
        answered in any::<bool>(),
        challenged in any::<bool>(),
    ) {
        prop_assume!(!has_key || !has_name);
        let mut request = request(secret_new, secret_approver, "Phone");
        if !has_key {
            request.pending.epk = None;
        }
        if !has_name {
            request.pending.sealed_name = None;
        }
        if !answered {
            request.pending.approver_epk = None;
            request.pending.approver_device_id = None;
        }
        let step = front_step(session(), &request.pending, &request.key, challenged);
        let seen = Seen::of(request.pending, step);
        let expected = if !answered && !challenged {
            PromptState::New
        } else {
            PromptState::Challenged
        };
        prop_assert_eq!(seen.prompt.state, expected);
        prop_assert_eq!(seen.prompt.code, None);
        prop_assert_eq!(seen.prompt.device_name, None);
    }

    /// What crosses the facade never holds a key: not the asking device's
    /// secret, not this phone's, in any written form.
    #[test]
    fn prop_a_prompt_carries_no_key_material(
        secret_new in any::<[u8; 32]>(),
        secret_approver in any::<[u8; 32]>(),
        name in "[a-zA-Z0-9 ]{1,20}",
    ) {
        let request = request(secret_new, secret_approver, &name);
        let step = front_step(session(), &request.pending, &request.key, true);
        let seen = Seen::of(request.pending, step);
        prop_assert_eq!(seen.prompt.state, PromptState::Ready);
        let shown = format!("{:?}", vec![seen.prompt]);
        for secret in [&secret_new, &secret_approver] {
            for form in forms(secret) {
                prop_assert!(!shown.contains(&form), "{shown} holds {form}");
            }
        }
    }
}

// no-prop-test: the four steps and what each is to the shell.
#[test]
fn each_step_of_a_request_has_its_state() {
    let request = request([1; 32], [2; 32], "Phone");
    let state = |step| Seen::of(request.pending.clone(), step).prompt.state;
    assert_eq!(state(FrontStep::Challenge), PromptState::New);
    assert_eq!(state(FrontStep::Wait), PromptState::Challenged);
    assert_eq!(
        state(FrontStep::Refuse(AccountError::ApprovalChanged)),
        PromptState::Changed
    );
    let opened = kanso_sync::open_request(
        session(),
        &request.pending,
        &request.key.pin,
        &request.key.keys,
    )
    .unwrap()
    .unwrap();
    assert_eq!(state(FrontStep::Open(opened)), PromptState::Ready);
}

// no-prop-test: a request another device answered, and one this phone did.
#[test]
fn a_request_another_device_answered_is_not_this_phones() {
    let mut request = request([1; 32], [2; 32], "Phone");
    assert!(!answered_elsewhere(session(), &request.pending));
    request.pending.approver_device_id = Some(format!("d_{}", "9".repeat(32)));
    assert!(answered_elsewhere(session(), &request.pending));
    request.pending.approver_device_id = None;
    assert!(!answered_elsewhere(session(), &request.pending));
}
