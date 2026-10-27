//! Push-token registration through the facade, against an in-memory server:
//! what is remembered, when the server is told, and that the session never
//! shows. CI pins the seed with `PROPTEST_RNG_SEED`.

mod common;

use std::sync::Arc;

use common::*;
use kanso_mobile::{KansoError, PushEnvironmentKind, PushPlatformKind, SyncHandle, SyncPhase};
use kanso_store::Store;
use kanso_sync::api::{PushEnvironment, PushPlatform, PushRegistration};
use kanso_sync::{MemoryServer, Route};
use proptest::prelude::*;

fn server_with_account() -> Arc<MemoryServer> {
    let server = Arc::new(MemoryServer::new());
    make_account(&server);
    server
}

/// The tokens the server holds for the account's active devices.
fn held(server: &MemoryServer) -> Vec<PushRegistration> {
    server
        .devices(ACCOUNT)
        .into_iter()
        .filter(|device| device.revoked.is_none())
        .filter_map(|device| server.push_registration(ACCOUNT, &device.id))
        .collect()
}

fn puts(server: &MemoryServer) -> usize {
    server.stats().count(Route::PutPush)
}

fn deletes(server: &MemoryServer) -> usize {
    server.stats().count(Route::DeletePush)
}

fn token(n: u8) -> String {
    format!("{n:02x}").repeat(32)
}

fn apns(n: u8) -> PushRegistration {
    PushRegistration {
        platform: PushPlatform::Apns,
        token: token(n),
        environment: PushEnvironment::Development,
    }
}

fn set(sync: &SyncHandle, n: u8) -> Result<(), KansoError> {
    sync.set_push_token(
        PushPlatformKind::Apns,
        token(n),
        PushEnvironmentKind::Development,
    )
}

/// What the library keeps for push, read from its database.
fn stored(phone: &Phone) -> Vec<(String, Option<String>)> {
    let store = Store::open(phone.path()).unwrap();
    [
        "push.token",
        "push.platform",
        "push.environment",
        "push.enabled",
        "push.registered_for",
    ]
    .into_iter()
    .map(|key| (key.to_owned(), store.meta(key).unwrap()))
    .collect()
}

// no-prop-test: a walk through the settings' defaults.
#[test]
fn pushes_are_on_until_turned_off() {
    let server = server_with_account();
    let phone = phone(&server);
    let sync = phone.library.sync();
    assert_eq!(sync.push_enabled(), Ok(true));
    sync.set_push_enabled(false).unwrap();
    assert_eq!(sync.push_enabled(), Ok(false));
    sync.set_push_enabled(true).unwrap();
    assert_eq!(sync.push_enabled(), Ok(true));
    // Signed out, none of it asks the server anything.
    assert_eq!(puts(&server) + deletes(&server), 0);
    assert_eq!(sync.refresh_push(), Ok(()));
}

// no-prop-test: a token handed over before and after signing in.
#[test]
fn a_token_given_before_signing_in_is_registered_by_signing_in() {
    let server = server_with_account();
    let phone = phone(&server);
    let sync = phone.library.sync();
    set(&sync, 1).unwrap();
    assert_eq!(puts(&server), 0, "nobody to tell yet");
    assert!(held(&server).is_empty());

    sign_in(&phone, "Phone");
    assert_eq!(held(&server), vec![apns(1)]);
    assert_eq!(puts(&server), 1);

    // The same token again, and a refresh, tell the server nothing.
    set(&sync, 1).unwrap();
    sync.refresh_push().unwrap();
    assert_eq!(puts(&server), 1);

    // A new token moves the registration.
    set(&sync, 2).unwrap();
    assert_eq!(held(&server), vec![apns(2)]);
    assert_eq!(puts(&server), 2);
}

// no-prop-test: a token handed over once signed in.
#[test]
fn a_token_given_while_signed_in_is_registered_at_once() {
    let server = server_with_account();
    let phone = phone(&server);
    sign_in(&phone, "Phone");
    assert_eq!(puts(&server), 0, "no token yet");
    set(&phone.library.sync(), 3).unwrap();
    assert_eq!(held(&server), vec![apns(3)]);
}

// no-prop-test: turning pushes off and on.
#[test]
fn off_removes_the_token_and_on_registers_it_again() {
    let server = server_with_account();
    let phone = phone(&server);
    let sync = phone.library.sync();
    sign_in(&phone, "Phone");
    set(&sync, 1).unwrap();

    sync.set_push_enabled(false).unwrap();
    assert!(held(&server).is_empty());
    assert_eq!(deletes(&server), 1);
    // Off twice, a refresh and a new token while off: nothing more is sent.
    sync.set_push_enabled(false).unwrap();
    sync.refresh_push().unwrap();
    set(&sync, 2).unwrap();
    assert_eq!((puts(&server), deletes(&server)), (1, 1));
    assert!(held(&server).is_empty());

    // On registers the newest token.
    sync.set_push_enabled(true).unwrap();
    assert_eq!(held(&server), vec![apns(2)]);
    assert_eq!(puts(&server), 2);
}

// no-prop-test: a walk through signing out and in.
#[test]
fn signing_out_keeps_the_os_token_and_forgets_the_registration() {
    let server = server_with_account();
    let phone = phone(&server);
    let sync = phone.library.sync();
    sign_in(&phone, "Phone");
    set(&sync, 1).unwrap();
    assert!(
        stored(&phone)
            .iter()
            .any(|(key, value)| key == "push.registered_for" && value.is_some())
    );

    sync.sign_out().unwrap();
    assert!(held(&server).is_empty(), "the server dropped it");
    let kept = stored(&phone);
    let value = |key: &str| {
        kept.iter()
            .find(|(k, _)| k == key)
            .and_then(|(_, v)| v.clone())
            .filter(|v| !v.is_empty())
    };
    assert_eq!(value("push.token"), Some(token(1)));
    assert_eq!(value("push.registered_for"), None);

    // Signing in again registers the same token, though it is the same one.
    sign_in(&phone, "Phone");
    assert_eq!(held(&server), vec![apns(1)]);
    assert_eq!(puts(&server), 2);
}

// no-prop-test: the shapes the server refuses are refused here, unsent.
#[test]
fn a_token_the_server_would_refuse_is_not_kept() {
    let server = server_with_account();
    let phone = phone(&server);
    sign_in(&phone, "Phone");
    let sync = phone.library.sync();
    let bad = [
        (PushPlatformKind::Apns, "xyz".to_owned()),
        (PushPlatformKind::Apns, "AB".repeat(32)),
        (PushPlatformKind::Apns, "ab".repeat(101)),
        (PushPlatformKind::Fcm, "short".to_owned()),
        (PushPlatformKind::Fcm, "a b".repeat(40)),
    ];
    for (platform, token) in bad {
        let err = sync
            .set_push_token(platform, token, PushEnvironmentKind::Production)
            .unwrap_err();
        assert!(matches!(err, KansoError::Invalid { .. }), "{err:?}");
    }
    // FCM has no development environment.
    let err = sync
        .set_push_token(
            PushPlatformKind::Fcm,
            "a".repeat(150),
            PushEnvironmentKind::Development,
        )
        .unwrap_err();
    assert!(matches!(err, KansoError::Invalid { .. }), "{err:?}");
    assert_eq!(puts(&server), 0);
    assert!(
        stored(&phone)
            .iter()
            .all(|(key, value)| key != "push.token" || value.is_none())
    );
    // A good FCM token is taken.
    sync.set_push_token(
        PushPlatformKind::Fcm,
        "f".repeat(152),
        PushEnvironmentKind::Production,
    )
    .unwrap();
    assert_eq!(puts(&server), 1);
}

// no-prop-test: two lost requests, one at sign-in and one at a call.
#[test]
fn a_registration_that_fails_is_retried_by_a_refresh_and_never_fails_a_sign_in() {
    let server = server_with_account();
    let phone = phone(&server);
    let sync = phone.library.sync();
    set(&sync, 1).unwrap();

    server.lose_next_requests(Route::PutPush, 1);
    sign_in(&phone, "Phone");
    assert_eq!(sync.status().unwrap().phase, SyncPhase::Idle, "signed in");
    assert!(held(&server).is_empty());

    sync.refresh_push().unwrap();
    assert_eq!(held(&server), vec![apns(1)]);

    // From a call the failure shows, the token is kept, and a refresh retries.
    server.lose_next_requests(Route::PutPush, 1);
    let err = set(&sync, 2).unwrap_err();
    assert!(matches!(err, KansoError::Network { .. }), "{err:?}");
    assert_eq!(held(&server), vec![apns(1)]);
    sync.refresh_push().unwrap();
    assert_eq!(held(&server), vec![apns(2)]);

    // And off that fails stays on the server until a refresh turns it off.
    server.lose_next_requests(Route::DeletePush, 1);
    let err = sync.set_push_enabled(false).unwrap_err();
    assert!(matches!(err, KansoError::Network { .. }), "{err:?}");
    assert_eq!(sync.push_enabled(), Ok(false));
    assert_eq!(held(&server), vec![apns(2)]);
    sync.refresh_push().unwrap();
    assert!(held(&server).is_empty());
}

// no-prop-test: a session the server ended.
#[test]
fn a_session_the_server_ended_says_signed_out_and_the_next_sign_in_registers_anew() {
    let server = server_with_account();
    let phone = phone(&server);
    let sync = phone.library.sync();
    sign_in(&phone, "Phone");
    set(&sync, 1).unwrap();
    server.revoke_all(ACCOUNT, kanso_sync::SignOutReason::Revoked);
    assert!(held(&server).is_empty());
    assert_eq!(set(&sync, 2), Err(KansoError::SignedOut));
    // Sign-in on this phone again: a new login, so a new registration, even
    // though the device and the token are the ones it had.
    sync.sign_out().unwrap();
    sign_in(&phone, "Phone");
    assert_eq!(held(&server), vec![apns(2)]);
}

// no-prop-test: the session token is a secret the settings and errors never hold.
#[test]
fn nothing_the_facade_returns_or_keeps_holds_the_session() {
    let server = server_with_account();
    let phone = phone(&server);
    let sync = phone.library.sync();
    server.lose_next_requests(Route::PutPush, 1);
    let mut shown = vec![format!("{:?}", set(&sync, 1))];
    shown.push(format!("{:?}", sync.status()));
    sign_in(&phone, "Phone");
    shown.push(format!("{:?}", sync.refresh_push()));
    shown.push(format!("{:?}", set(&sync, 2)));
    shown.push(format!("{:?}", sync.set_push_enabled(false)));
    shown.push(format!("{:?}", sync.push_enabled()));
    shown.push(format!("{:?}", sync.status()));
    shown.extend(
        stored(&phone)
            .into_iter()
            .map(|(key, value)| format!("{key}={value:?}")),
    );
    server.revoke_all(ACCOUNT, kanso_sync::SignOutReason::Revoked);
    shown.push(format!("{:?}", set(&sync, 3)));
    for text in &shown {
        // A session token is `kanso_` and 43 characters; the login label is
        // the device and a digest of it, which is not a session.
        assert!(!text.contains("kanso_"), "{text}");
    }
    let label = stored(&phone)
        .into_iter()
        .find(|(key, _)| key == "push.registered_for")
        .unwrap()
        .1;
    // Cleared by the revocation's sign-out, or the device and a digest.
    assert!(label.is_none_or(|l| l.is_empty() || l.starts_with("d_")));
}

#[derive(Clone, Debug)]
enum Op {
    Token(u8),
    Enable(bool),
    Refresh,
    SignOut,
    SignIn,
}

fn ops() -> impl Strategy<Value = Vec<Op>> {
    prop::collection::vec(
        prop_oneof![
            (1u8..=3).prop_map(Op::Token),
            any::<bool>().prop_map(Op::Enable),
            Just(Op::Refresh),
            Just(Op::SignOut),
            Just(Op::SignIn),
        ],
        1..10,
    )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(12))]

    /// Invariant: whatever order the calls come in, the server holds the
    /// remembered token exactly when the phone is signed in with pushes on,
    /// and it is sent only when it is not already registered for this login.
    #[test]
    fn prop_the_server_holds_the_token_when_signed_in_and_on(ops in ops()) {
        let server = server_with_account();
        let phone = phone(&server);
        let sync = phone.library.sync();
        let (mut signed_in, mut enabled) = (false, true);
        let mut remembered: Option<u8> = None;
        let mut registered: Option<u8> = None;
        for op in ops {
            let puts_before = puts(&server);
            match op {
                Op::Token(n) => {
                    prop_assert!(set(&sync, n).is_ok());
                    remembered = Some(n);
                }
                Op::Enable(on) => {
                    prop_assert!(sync.set_push_enabled(on).is_ok());
                    enabled = on;
                }
                Op::Refresh => prop_assert!(sync.refresh_push().is_ok()),
                Op::SignOut => {
                    prop_assert!(sync.sign_out().is_ok());
                    signed_in = false;
                    registered = None;
                }
                Op::SignIn => {
                    if !signed_in {
                        sign_in(&phone, "Phone");
                        signed_in = true;
                        registered = None;
                    }
                }
            }
            if signed_in && !enabled {
                registered = None;
            }
            let wanted = if signed_in && enabled { remembered } else { None };
            let sends = usize::from(wanted.is_some() && wanted != registered);
            if wanted.is_some() {
                registered = wanted;
            }
            prop_assert_eq!(held(&server), wanted.map(apns).into_iter().collect::<Vec<_>>());
            prop_assert_eq!(puts(&server) - puts_before, sends);
        }
    }
}
