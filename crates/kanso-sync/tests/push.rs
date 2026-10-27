//! Push-token registration against `MemoryServer`
//! (`PUT`/`DELETE /v2/account/devices/<device_id>/push`): the request's
//! shape, the server's validation, whose token it is, and that it goes with
//! its session. CI pins the seed with `PROPTEST_RNG_SEED`.

use kanso_sync::api::{PushEnvironment, PushPlatform, PushRegistration};
use kanso_sync::keychain;
use kanso_sync::{
    AccountError, ApiError, Backend, DeviceIdentity, MemorySecrets, MemoryServer, Password, Route,
    SecretStore, Session, SessionToken, SignOutReason, Step,
};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use serde_json::json;

const SERVER: &str = "http://127.0.0.1:8787";

fn config(cases: u32) -> ProptestConfig {
    ProptestConfig {
        cases,
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
        ..ProptestConfig::default()
    }
}

fn mac(n: u8) -> DeviceIdentity {
    DeviceIdentity::from_parts(&format!("l_{:032x}", n), &format!("Mac {n}"), [n; 32])
}

fn quiet(_: Step) {}

fn password() -> Password {
    Password::new("correct horse battery", "alice").unwrap()
}

/// The account "alice" with one session per device in `devices`.
fn account(server: &MemoryServer, devices: &[u8]) -> Vec<Session> {
    let mut sessions = Vec::new();
    for (i, n) in devices.iter().enumerate() {
        // Each device a second apart, so "least recently seen" is the first.
        server.set_now(1_800_000_000_000 + i as i64 * 1000);
        let session = if i == 0 {
            kanso_sync::sign_up(
                server,
                &MemorySecrets::new(),
                SERVER,
                "alice",
                &password(),
                &mac(*n),
                &quiet,
            )
            .unwrap()
            .session
        } else {
            kanso_sync::sign_in(server, SERVER, "alice", &password(), &mac(*n), &quiet)
                .unwrap()
                .session
        };
        sessions.push(session);
    }
    sessions
}

/// The session's token, read back through the Keychain item format.
fn token_of(session: &Session) -> SessionToken {
    let secrets = MemorySecrets::new();
    let library = "l_00000000000000000000000000000000";
    keychain::save_session(&secrets, library, session).unwrap();
    let json = secrets
        .get(&keychain::account_item(library))
        .unwrap()
        .unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    SessionToken::new(value["token"].as_str().unwrap())
}

fn apns(token: &str, environment: PushEnvironment) -> PushRegistration {
    PushRegistration {
        platform: PushPlatform::Apns,
        token: token.to_owned(),
        environment,
    }
}

fn fcm(token: &str) -> PushRegistration {
    PushRegistration {
        platform: PushPlatform::Fcm,
        token: token.to_owned(),
        environment: PushEnvironment::Production,
    }
}

fn apns_token() -> String {
    "ab".repeat(32)
}

fn fcm_token() -> String {
    format!("fcm:{}", "A-z_0".repeat(30))
}

fn registered(server: &MemoryServer, session: &Session) -> Option<PushRegistration> {
    server.push_registration("alice", &session.device_id)
}

// no-prop-test: the wire's exact JSON, a table.
#[test]
fn the_request_is_the_wires_json() {
    let reg = apns(&apns_token(), PushEnvironment::Development);
    assert_eq!(
        serde_json::to_value(&reg).unwrap(),
        json!({"platform": "apns", "token": apns_token(), "environment": "development"})
    );
    let reg = fcm(&fcm_token());
    assert_eq!(
        serde_json::to_value(&reg).unwrap(),
        json!({"platform": "fcm", "token": fcm_token(), "environment": "production"})
    );
    assert_eq!(
        Route::PutPush.http(),
        ("PUT", "/v2/account/devices/<device_id>/push")
    );
    assert_eq!(
        Route::DeletePush.http(),
        ("DELETE", "/v2/account/devices/<device_id>/push")
    );
    assert!(Route::PutPush.needs_session() && Route::DeletePush.needs_session());
}

// no-prop-test: one registration, one account.
#[test]
fn a_registration_stays_off_the_debug_output() {
    let reg = apns(&apns_token(), PushEnvironment::Production);
    let shown = format!("{reg:?}");
    assert!(!shown.contains(&apns_token()), "{shown}");
}

// no-prop-test: a walk through one device's registration.
#[test]
fn register_replace_and_turn_off() {
    let server = MemoryServer::new();
    let s = account(&server, &[1]).remove(0);
    assert_eq!(registered(&server, &s), None);

    let first = apns(&apns_token(), PushEnvironment::Production);
    kanso_sync::register_push(&server, &s, &first).unwrap();
    assert_eq!(registered(&server, &s), Some(first.clone()));
    // The same again changes nothing.
    kanso_sync::register_push(&server, &s, &first).unwrap();
    assert_eq!(registered(&server, &s), Some(first));

    // A new token moves the device's registration, it doesn't add one.
    let second = apns(&"cd".repeat(40), PushEnvironment::Development);
    kanso_sync::register_push(&server, &s, &second).unwrap();
    assert_eq!(registered(&server, &s), Some(second));

    kanso_sync::unregister_push(&server, &s).unwrap();
    assert_eq!(registered(&server, &s), None);
    // Turning off twice, or when never on, is fine.
    kanso_sync::unregister_push(&server, &s).unwrap();
    assert_eq!(server.stats().count(Route::DeletePush), 2);
}

// no-prop-test: one device changing platform.
#[test]
fn a_device_can_move_between_services() {
    let server = MemoryServer::new();
    let s = account(&server, &[1]).remove(0);
    kanso_sync::register_push(
        &server,
        &s,
        &apns(&apns_token(), PushEnvironment::Production),
    )
    .unwrap();
    let to = fcm(&fcm_token());
    kanso_sync::register_push(&server, &s, &to).unwrap();
    assert_eq!(registered(&server, &s), Some(to));
}

// no-prop-test: a table of the ways a session ends.
#[test]
fn the_registration_goes_with_its_session() {
    let server = MemoryServer::new();
    let mut sessions = account(&server, &[1, 2, 3]);
    let reg = apns(&apns_token(), PushEnvironment::Production);
    for s in &sessions {
        kanso_sync::register_push(&server, s, &reg).unwrap();
        assert_eq!(registered(&server, s), Some(reg.clone()));
    }

    // Signed out by another device.
    kanso_sync::revoke_device(&server, &sessions[0], &sessions[1].device_id).unwrap();
    assert_eq!(registered(&server, &sessions[1]), None);
    assert_eq!(registered(&server, &sessions[0]), Some(reg.clone()));
    assert_eq!(registered(&server, &sessions[2]), Some(reg.clone()));

    // Signing out.
    kanso_sync::sign_out(&server, &MemorySecrets::new(), &sessions[2]).unwrap();
    assert_eq!(registered(&server, &sessions[2]), None);

    // A revoked session can't register again, and signing back in starts
    // with nothing registered.
    let err = kanso_sync::register_push(&server, &sessions[1], &reg).unwrap_err();
    assert!(
        matches!(
            err,
            AccountError::Unauthorized {
                reason: SignOutReason::Revoked
            }
        ),
        "{err:?}"
    );
    let back = kanso_sync::sign_in(&server, SERVER, "alice", &password(), &mac(2), &quiet)
        .unwrap()
        .session;
    assert_eq!(registered(&server, &back), None);
    sessions[1] = back;

    // Ending every session.
    server.revoke_all("alice", SignOutReason::Revoked);
    assert_eq!(registered(&server, &sessions[0]), None);
    assert_eq!(registered(&server, &sessions[1]), None);
}

// no-prop-test: a device evicted by an eleventh sign-in.
#[test]
fn an_evicted_device_loses_its_registration() {
    let server = MemoryServer::new();
    let devices: Vec<u8> = (1..=10).collect();
    let sessions = account(&server, &devices);
    let reg = apns(&apns_token(), PushEnvironment::Production);
    kanso_sync::register_push(&server, &sessions[0], &reg).unwrap();
    server.set_now(1_800_000_100_000);
    kanso_sync::sign_in(&server, SERVER, "alice", &password(), &mac(11), &quiet).unwrap();
    assert_eq!(registered(&server, &sessions[0]), None);
}

// no-prop-test: the routes' answers for someone else's device.
#[test]
fn another_devices_registration_is_not_yours_to_change() {
    let server = MemoryServer::new();
    let sessions = account(&server, &[1, 2]);
    let reg = apns(&apns_token(), PushEnvironment::Production);
    kanso_sync::register_push(&server, &sessions[1], &reg).unwrap();

    let api = server.api(SERVER);
    let mine = token_of(&sessions[0]);
    let theirs = &sessions[1].device_id;
    assert_eq!(
        api.put_push(
            &mine,
            theirs,
            &apns(&"cd".repeat(20), PushEnvironment::Production)
        ),
        Err(ApiError::NotFound)
    );
    assert_eq!(api.delete_push(&mine, theirs), Err(ApiError::NotFound));
    assert_eq!(registered(&server, &sessions[1]), Some(reg));
    assert_eq!(registered(&server, &sessions[0]), None);

    // A device ID that isn't one is a bad request, a stranger's is not found.
    assert_eq!(
        api.put_push(
            &mine,
            "d_nope",
            &apns(&apns_token(), PushEnvironment::Production)
        ),
        Err(ApiError::BadRequest)
    );
    assert_eq!(
        api.delete_push(&mine, &format!("d_{}", "0".repeat(32))),
        Err(ApiError::NotFound)
    );
    // No session, no answer.
    let nobody = SessionToken::new(format!("kanso_{}", "A".repeat(43)));
    assert!(matches!(
        api.put_push(
            &nobody,
            &sessions[0].device_id,
            &apns(&apns_token(), PushEnvironment::Production)
        ),
        Err(ApiError::Unauthorized { .. })
    ));
    assert!(matches!(
        api.delete_push(&nobody, &sessions[0].device_id),
        Err(ApiError::Unauthorized { .. })
    ));
}

// no-prop-test: a refused registration leaves the old one.
#[test]
fn a_refused_registration_leaves_the_old_one() {
    let server = MemoryServer::new();
    let s = account(&server, &[1]).remove(0);
    let good = apns(&apns_token(), PushEnvironment::Production);
    kanso_sync::register_push(&server, &s, &good).unwrap();
    let err = kanso_sync::register_push(&server, &s, &fcm("short")).unwrap_err();
    assert!(matches!(err, AccountError::BadRequest), "{err:?}");
    assert_eq!(registered(&server, &s), Some(good));
}

fn hex_token() -> impl Strategy<Value = String> {
    (16usize..=100).prop_flat_map(|pairs| {
        prop::string::string_regex(&format!("[0-9a-f]{{{}}}", pairs * 2)).unwrap()
    })
}

fn fcm_chars() -> impl Strategy<Value = String> {
    (100usize..=600)
        .prop_flat_map(|n| prop::string::string_regex(&format!("[A-Za-z0-9_:-]{{{n}}}")).unwrap())
}

fn valid() -> impl Strategy<Value = PushRegistration> {
    prop_oneof![
        (
            hex_token(),
            prop_oneof![
                Just(PushEnvironment::Production),
                Just(PushEnvironment::Development)
            ]
        )
            .prop_map(|(token, environment)| apns(&token, environment)),
        fcm_chars().prop_map(|token| fcm(&token)),
    ]
}

fn invalid() -> impl Strategy<Value = PushRegistration> {
    prop_oneof![
        // APNs: odd length, too short, too long, upper case, not hex.
        (16usize..=100)
            .prop_flat_map(|pairs| {
                prop::string::string_regex(&format!("[0-9a-f]{{{}}}", pairs * 2 + 1)).unwrap()
            })
            .prop_map(|t| apns(&t, PushEnvironment::Production)),
        prop::string::string_regex("[0-9a-f]{0,30}")
            .unwrap()
            .prop_map(|t| apns(&t, PushEnvironment::Production)),
        Just(apns(&"ab".repeat(101), PushEnvironment::Development)),
        Just(apns(&"AB".repeat(32), PushEnvironment::Production)),
        Just(apns(
            &format!("{}zz", "ab".repeat(31)),
            PushEnvironment::Production
        )),
        // FCM: too short, too long, a character outside the set, development.
        prop::string::string_regex("[A-Za-z0-9_:-]{0,99}")
            .unwrap()
            .prop_map(|t| fcm(&t)),
        Just(fcm(&"a".repeat(4097))),
        Just(fcm(&format!("{} {}", "a".repeat(60), "b".repeat(60)))),
        Just(fcm(&format!("{}.{}", "a".repeat(60), "b".repeat(60)))),
        fcm_chars().prop_map(|t| PushRegistration {
            environment: PushEnvironment::Development,
            ..fcm(&t)
        }),
    ]
}

proptest! {
    #![proptest_config(config(64))]

    /// Invariant: the request's JSON is the wire's, and reads back whole.
    #[test]
    fn prop_a_registration_round_trips_through_its_json(reg in valid()) {
        let value = serde_json::to_value(&reg).unwrap();
        let object = value.as_object().unwrap();
        prop_assert_eq!(object.len(), 3);
        prop_assert_eq!(
            object["platform"].as_str().unwrap(),
            match reg.platform { PushPlatform::Apns => "apns", PushPlatform::Fcm => "fcm" }
        );
        prop_assert_eq!(object["token"].as_str().unwrap(), reg.token.as_str());
        prop_assert_eq!(
            object["environment"].as_str().unwrap(),
            match reg.environment {
                PushEnvironment::Production => "production",
                PushEnvironment::Development => "development",
            }
        );
        let back: PushRegistration = serde_json::from_value(value).unwrap();
        prop_assert_eq!(back, reg);
    }

    /// Invariant: the server keeps a token of the shapes it takes and
    /// refuses the others, and a refusal leaves what was registered.
    #[test]
    fn prop_valid_tokens_are_kept_and_invalid_ones_refused(
        good in valid(),
        bad in invalid(),
    ) {
        let server = MemoryServer::new();
        let s = account(&server, &[1]).remove(0);
        prop_assert!(good.is_valid());
        prop_assert!(!bad.is_valid());
        prop_assert!(kanso_sync::register_push(&server, &s, &good).is_ok());
        prop_assert_eq!(registered(&server, &s), Some(good.clone()));
        let refused = kanso_sync::register_push(&server, &s, &bad);
        prop_assert!(matches!(refused, Err(AccountError::BadRequest)));
        prop_assert_eq!(registered(&server, &s), Some(good));
    }

    /// Invariant: a login's label never holds its token, and a new login of
    /// the same device has another.
    #[test]
    fn prop_the_login_label_is_not_the_token(n in 1u8..=200) {
        let server = MemoryServer::new();
        let s = account(&server, &[n]).remove(0);
        let token = token_of(&s);
        let label = s.login_label();
        prop_assert!(!label.contains(token.expose()));
        prop_assert!(!label.contains("kanso_"));
        prop_assert!(label.starts_with(&s.device_id));
        prop_assert_eq!(label.clone(), s.login_label());
        let again = kanso_sync::sign_in(&server, SERVER, "alice", &password(), &mac(n), &quiet)
            .unwrap()
            .session;
        prop_assert_eq!(&again.device_id, &s.device_id);
        prop_assert_ne!(again.login_label(), label);
    }
}
