//! The two-device approval flow end to end over `MemoryServer`
//! (docs/ACCOUNTS.md › Unlocking a new device, › Account flows), and the
//! pieces the app's screens depend on: the same code on both devices, the
//! approver's own commitment check, single use, cancellation and expiry.

use kanso_sync::{
    AccountError, ApprovalKeys, Backend, DeviceIdentity, LockedSession, MemorySecrets,
    MemoryServer, Password, RequestState, SessionToken, SignOutReason, SignedIn, cancel_request,
    challenge_request, complete_request, decline_request, devices, finish_approval, finish_unlock,
    name_device, open_request, pending_approvals, poll_approval, request_approval, sign_in,
    sign_up,
};

const SERVER: &str = "https://sync.kansoapp.app";
const PASSWORD: &str = "correct horse battery staple";
const USERNAME: &str = "alice";

fn no_step(_: kanso_sync::Step) {}

fn password() -> Password {
    Password::new(PASSWORD, USERNAME).expect("a valid password")
}

fn device(name: &str, secret: u8) -> DeviceIdentity {
    DeviceIdentity::from_parts(&format!("l_{:032x}", u64::from(secret)), name, [secret; 32])
}

fn token_of(session: &kanso_sync::Session) -> SessionToken {
    let header = session.authorization();
    SessionToken::new(header.trim_start_matches("Bearer ").to_owned())
}

/// A locked view of a session: the token and key ID, but not the data key.
fn locked_from(session: &kanso_sync::Session) -> LockedSession {
    LockedSession::new(
        session.server.clone(),
        session.origin.clone(),
        session.account_id.clone(),
        session.username.clone(),
        session.device_id.clone(),
        token_of(session),
        session.key_id(),
        "0".to_owned(),
        None,
        None,
    )
}

struct Fixture {
    server: MemoryServer,
    alice: SignedIn,
    new_device: DeviceIdentity,
    new_session: SignedIn,
    locked: LockedSession,
}

fn fixture() -> Fixture {
    let server = MemoryServer::new();
    let secrets = MemorySecrets::default();
    let alice_device = device("Alex's MacBook Air", 1);
    let signed_up = sign_up(
        &server,
        &secrets,
        SERVER,
        USERNAME,
        &password(),
        &alice_device,
        &no_step,
    )
    .expect("sign up");
    let new_device = device("Alex's Mac mini", 2);
    let signed_in = sign_in(
        &server,
        SERVER,
        USERNAME,
        &password(),
        &new_device,
        &no_step,
    )
    .expect("sign in the new device");
    let alice = SignedIn {
        session: signed_up.session,
        evicted: None,
    };
    let locked = locked_from(&signed_in.session);
    Fixture {
        server,
        alice,
        new_device,
        new_session: signed_in,
        locked,
    }
}

#[test]
fn two_devices_agree_on_the_code_and_the_name_and_the_new_device_unlocks() {
    let f = fixture();
    let keys_new = ApprovalKeys::generate();
    let request = request_approval(&f.server, &f.locked, &keys_new).expect("ask");
    assert_eq!(request.other_devices, 1);
    assert_eq!(request.epk, keys_new.epk());

    let pending = pending_approvals(&f.server, &f.alice.session).expect("list");
    assert_eq!(pending.len(), 1);
    let pending = &pending[0];
    assert_eq!(pending.id, request.id);
    assert_eq!(pending.device_id, f.locked.device_id);

    let keys_alice = ApprovalKeys::generate();
    challenge_request(&f.server, &f.alice.session, pending, &keys_alice).expect("challenge");

    let state = poll_approval(&f.server, &f.locked, &request).expect("poll");
    let RequestState::Challenged {
        approver_epk,
        approver_device_id,
    } = state
    else {
        panic!("the new device should be challenged, got {state:?}");
    };
    assert_eq!(approver_device_id, f.alice.session.device_id);
    let approval_new =
        complete_request(&f.locked, &request, &keys_new, &approver_epk).expect("code");
    name_device(&f.server, &f.locked, &request, &approval_new, &f.new_device).expect("name");

    let pending = pending_approvals(&f.server, &f.alice.session).expect("list");
    let opened = open_request(&f.alice.session, &pending[0], &keys_alice)
        .expect("open")
        .expect("named");
    assert_eq!(opened.approval.code(), approval_new.code());
    assert_eq!(opened.name, "Alex's Mac mini");
    assert_eq!(opened.approval.code().len(), 7);
    assert_eq!(opened.approval.code().as_bytes()[3], b' ');

    finish_approval(&f.server, &f.alice.session, &pending[0], &opened.approval).expect("approve");
    let state = poll_approval(&f.server, &f.locked, &request).expect("poll");
    let RequestState::Approved(approved) = state else {
        panic!("the new device should be approved, got {state:?}");
    };
    let unlocked = finish_unlock(
        &f.server,
        &f.locked,
        &approval_new,
        &approved,
        &f.new_device,
    )
    .expect("unlock");
    assert_eq!(unlocked.session.key_id(), f.alice.session.key_id());
    // It can now reach the account as an ordinary unlocked device.
    devices(&f.server, &unlocked.session).expect("the new device is signed in");
}

#[test]
fn the_approver_refuses_a_revealed_key_that_does_not_match_the_commitment() {
    let f = fixture();
    let keys_new = ApprovalKeys::generate();
    let request = request_approval(&f.server, &f.locked, &keys_new).expect("ask");
    let keys_alice = ApprovalKeys::generate();
    let pending = pending_approvals(&f.server, &f.alice.session).expect("list");
    challenge_request(&f.server, &f.alice.session, &pending[0], &keys_alice).expect("challenge");
    let state = poll_approval(&f.server, &f.locked, &request).expect("poll");
    let RequestState::Challenged { approver_epk, .. } = state else {
        panic!("challenged");
    };
    let approval = complete_request(&f.locked, &request, &keys_new, &approver_epk).expect("code");
    name_device(&f.server, &f.locked, &request, &approval, &f.new_device).expect("name");

    let mut pending = pending_approvals(&f.server, &f.alice.session).expect("list");
    // A malicious server swaps in a key it chose: the approver checks the
    // commitment itself and refuses before opening the name.
    pending[0].epk = Some([9; 32]);
    let err = open_request(&f.alice.session, &pending[0], &keys_alice).expect_err("refused");
    assert!(matches!(err, AccountError::ApprovalChanged), "{err:?}");
}

#[test]
fn the_request_is_single_use_and_either_side_can_cancel() {
    let f = fixture();
    let keys_new = ApprovalKeys::generate();
    let request = request_approval(&f.server, &f.locked, &keys_new).expect("ask");

    // The approver declines: the requester's poll says cancelled.
    let pending = pending_approvals(&f.server, &f.alice.session).expect("list");
    decline_request(&f.server, &f.alice.session, &pending[0].id).expect("decline");
    assert_eq!(
        poll_approval(&f.server, &f.locked, &request).expect("poll"),
        RequestState::Cancelled
    );

    // The requester cancels too; a second cancel is harmless.
    let request = request_approval(&f.server, &f.locked, &keys_new).expect("ask again");
    cancel_request(&f.server, &f.locked, &request.id).expect("cancel");
    cancel_request(&f.server, &f.locked, &request.id).expect("cancel again");
    assert_eq!(
        poll_approval(&f.server, &f.locked, &request).expect("poll"),
        RequestState::Cancelled
    );
}

#[test]
fn a_request_expires_after_five_minutes_and_a_second_approval_is_refused() {
    let f = fixture();
    let now = 1_800_000_000_000;
    f.server.set_now(now);
    let keys_new = ApprovalKeys::generate();
    let request = request_approval(&f.server, &f.locked, &keys_new).expect("ask");
    f.server.set_now(now + 300_001);
    assert_eq!(
        poll_approval(&f.server, &f.locked, &request).expect("poll"),
        RequestState::Expired
    );

    let f = fixture();
    let keys_new = ApprovalKeys::generate();
    let request = request_approval(&f.server, &f.locked, &keys_new).expect("ask");
    let keys_alice = ApprovalKeys::generate();
    let pending = pending_approvals(&f.server, &f.alice.session).expect("list");
    challenge_request(&f.server, &f.alice.session, &pending[0], &keys_alice).expect("challenge");
    let state = poll_approval(&f.server, &f.locked, &request).expect("poll");
    let RequestState::Challenged { approver_epk, .. } = state else {
        panic!("challenged");
    };
    let approval = complete_request(&f.locked, &request, &keys_new, &approver_epk).expect("code");
    name_device(&f.server, &f.locked, &request, &approval, &f.new_device).expect("name");
    let pending = pending_approvals(&f.server, &f.alice.session).expect("list");
    let opened = open_request(&f.alice.session, &pending[0], &keys_alice)
        .expect("open")
        .expect("named");
    finish_approval(&f.server, &f.alice.session, &pending[0], &opened.approval).expect("approve");
    // The approver can't answer again with a different key.
    let again = pending_approvals(&f.server, &f.alice.session).expect("list");
    assert!(again.is_empty(), "an answered request is no longer pending");
}

#[test]
fn a_device_cannot_approve_its_own_request_and_only_the_account_reads_it() {
    let f = fixture();
    let keys_new = ApprovalKeys::generate();
    let request = request_approval(&f.server, &f.locked, &keys_new).expect("ask");
    let own = pending_approvals(&f.server, &f.new_session.session).expect("list");
    assert!(
        own.is_empty(),
        "the list never shows the caller's own request"
    );
    assert_eq!(
        f.server
            .api(SERVER)
            .approval(&token_of(&f.new_session.session), "nope")
            .expect_err("bad id"),
        kanso_sync::ApiError::BadRequest
    );
    // A session of another account sees nothing and can't read the request.
    let bob = device("Bob's Mac", 3);
    let bob_session = sign_up(
        &f.server,
        &MemorySecrets::default(),
        SERVER,
        "bob",
        &Password::new(PASSWORD, "bob").expect("password"),
        &bob,
        &no_step,
    )
    .expect("bob");
    assert!(
        pending_approvals(&f.server, &bob_session.session)
            .expect("list")
            .is_empty()
    );
    let err = f
        .server
        .api(SERVER)
        .approval(&token_of(&bob_session.session), &request.id)
        .expect_err("another account");
    assert_eq!(err, kanso_sync::ApiError::NotFound);
}

#[test]
fn a_lost_session_is_refused_with_its_reason() {
    let f = fixture();
    let mut locked = locked_from(&f.new_session.session);
    let token = token_of(&f.new_session.session);
    f.server.api(SERVER).logout(&token).expect("logout");
    // The locked session still holds the token; the server says signed out.
    locked.set_library_id("l_00000000000000000000000000000000");
    let err =
        request_approval(&f.server, &locked, &ApprovalKeys::generate()).expect_err("signed out");
    let AccountError::Unauthorized { reason } = err else {
        panic!("expected unauthorized, got {err:?}");
    };
    assert_eq!(reason, SignOutReason::SignedOut);
}
