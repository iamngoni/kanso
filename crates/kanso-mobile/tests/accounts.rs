//! The account flows of the phone against an in-memory server: signing up
//! and in, the binding question, providers, passkeys (a software passkey plays
//! the platform's sheet through the same JSON the native plugin uses),
//! locked sessions, device approval as the new device, and recovery.

mod common;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use common::*;
use kanso_engine::account::approval::{ApproverKey, FrontStep, front_step};
use kanso_mobile::passkey_bridge;
use kanso_mobile::{
    AccountHandle, AccountState, ApprovalPhase, ApprovalWait, BindingAnswer, KansoError, Library,
    PasskeyRequest, PasskeyRequestKind, ProviderEvent, ProviderKind, SyncPhase,
};
use kanso_sync::passkey::{
    PasskeyAssertionChallenge, PasskeyCeremony, PasskeyRegistrationChallenge,
};
use kanso_sync::provider::Provider;
use kanso_sync::{
    MEMORY_PASSKEY_ORIGIN, MEMORY_PASSKEY_SERVER, MemorySecrets, MemoryServer, RecoveryKey,
    Session, TestAuthenticator,
};
use proptest::prelude::*;
use serde_json::{Value, json};

/// The server every account here lives on: its host is the passkey
/// server's, so passkeys work against it.
const SERVER: &str = MEMORY_PASSKEY_SERVER;
const PASSWORD: &str = ACCOUNT_PASSWORD;

// ------------------------------------------------------------------ helpers

/// A log a sink writes and a test reads.
fn log<T: Send + 'static>() -> (Arc<Mutex<Vec<T>>>, impl FnMut(T) -> bool + Send + 'static) {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let writer = seen.clone();
    (seen, move |value| {
        writer.lock().unwrap().push(value);
        true
    })
}

/// Waits until `done` holds for what `seen` has, and returns it.
fn wait_for<T: Clone>(
    seen: &Arc<Mutex<Vec<T>>>,
    what: &str,
    done: impl Fn(&[T]) -> bool,
) -> Vec<T> {
    let end = Instant::now() + Duration::from_secs(60);
    loop {
        let now = seen.lock().unwrap().clone();
        if done(&now) {
            return now;
        }
        assert!(Instant::now() < end, "timed out waiting for {what}");
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn account(phone: &Phone) -> AccountHandle {
    phone.library.account()
}

/// Plays the platform's passkey sheet with `key`, from the requests the
/// shell would receive, answering through the bridge as the native plugin
/// does. `prf` replaces the key's PRF output when it is asked for; `seen`
/// keeps every request's options.
fn native(
    account: &AccountHandle,
    key: &Arc<TestAuthenticator>,
    prf: Option<[u8; 32]>,
    seen: &Arc<Mutex<Vec<PasskeyRequest>>>,
) {
    let (key, seen) = (key.clone(), seen.clone());
    account.passkey_requests(move |request| {
        seen.lock().unwrap().push(request.clone());
        let json = sheet(&key, &request, prf);
        // A request another listener answered first is refused; that is fine.
        let _ = passkey_bridge::answer(request.ticket, &json);
        true
    });
}

fn bytes(options: &Value, field: &str) -> Vec<u8> {
    URL_SAFE_NO_PAD
        .decode(options[field].as_str().unwrap())
        .unwrap()
}

fn encode(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

/// What the platform answers `request` with `key`.
fn sheet(key: &TestAuthenticator, request: &PasskeyRequest, prf: Option<[u8; 32]>) -> String {
    let options: Value = serde_json::from_str(&request.options_json).unwrap();
    let wants_prf = !options["prf_input"].is_null();
    let ids = |field: &str| -> Vec<Vec<u8>> {
        options[field]
            .as_array()
            .unwrap()
            .iter()
            .map(|id| URL_SAFE_NO_PAD.decode(id.as_str().unwrap()).unwrap())
            .collect()
    };
    match request.kind {
        PasskeyRequestKind::Register => {
            let made = key
                .register(&PasskeyRegistrationChallenge {
                    rp_id: options["rp_id"].as_str().unwrap().to_owned(),
                    origin: MEMORY_PASSKEY_ORIGIN.to_owned(),
                    challenge: bytes(&options, "challenge"),
                    user_handle: bytes(&options, "user_id"),
                    username: options["username"].as_str().unwrap().to_owned(),
                    display_name: options["display_name"].as_str().unwrap().to_owned(),
                    exclude_credentials: ids("exclude_credentials"),
                    request_prf: wants_prf,
                })
                .unwrap();
            let prf = made.prf().map(|own| prf.unwrap_or(*own));
            json!({
                "credential_id": encode(&made.credential_id),
                "attestation_object": encode(&made.attestation_object),
                "client_data_json": encode(&made.client_data_json),
                "transports": made.transports,
                "prf": prf.map(|prf| encode(&prf)),
            })
            .to_string()
        }
        PasskeyRequestKind::Assert => {
            let made = key
                .assert(&PasskeyAssertionChallenge {
                    rp_id: options["rp_id"].as_str().unwrap().to_owned(),
                    origin: MEMORY_PASSKEY_ORIGIN.to_owned(),
                    challenge: bytes(&options, "challenge"),
                    allow_credentials: ids("allow_credentials"),
                    request_prf: wants_prf,
                })
                .unwrap();
            let prf = made.prf().map(|own| prf.unwrap_or(*own));
            json!({
                "credential_id": encode(&made.credential_id),
                "authenticator_data": encode(&made.authenticator_data),
                "client_data_json": encode(&made.client_data_json),
                "signature": encode(&made.signature),
                "user_handle": made.user_handle.as_deref().map(encode),
                "prf": prf.map(|prf| encode(&prf)),
            })
            .to_string()
        }
    }
}

/// The session a phone holds, read back from its secret store.
fn session_of(secrets: &MemorySecrets) -> Session {
    let item = secrets
        .items()
        .into_iter()
        .find(|item| item.starts_with("sync-account:"))
        .expect("the phone is signed in");
    let raw: Value = serde_json::from_str(
        &kanso_sync::SecretStore::get(secrets, &item)
            .unwrap()
            .unwrap(),
    )
    .unwrap();
    kanso_sync::keychain::load_session(
        secrets,
        item.trim_start_matches("sync-account:"),
        raw["origin"].as_str().unwrap(),
        raw["account_id"].as_str().unwrap(),
    )
    .unwrap()
    .expect("the session loads")
}

/// A device of the account that holds the key answers the request that
/// shows up, as the sheet of the desktop does: it challenges, opens the
/// named request and approves. Returns the code it showed.
fn approve(server: &Arc<MemoryServer>, approver: &Session) -> String {
    let end = Instant::now() + Duration::from_secs(60);
    let mut key: Option<ApproverKey> = None;
    let mut challenged = false;
    loop {
        assert!(Instant::now() < end, "no request to approve");
        std::thread::sleep(Duration::from_millis(50));
        let pending = kanso_sync::pending_approvals(server.as_ref(), approver).unwrap();
        let Some(front) = pending.first() else {
            continue;
        };
        let held = key.get_or_insert_with(|| ApproverKey::new(front));
        match front_step(approver, front, held, challenged) {
            FrontStep::Challenge => {
                kanso_sync::challenge_request(server.as_ref(), approver, front, &held.keys)
                    .unwrap();
                challenged = true;
            }
            FrontStep::Wait => {}
            FrontStep::Open(opened) => {
                kanso_sync::finish_approval(server.as_ref(), approver, front, &opened.approval)
                    .unwrap();
                return opened.approval.code();
            }
            FrontStep::Refuse(err) => panic!("an honest request was refused: {err}"),
        }
    }
}

/// A phone that made the account `ada` with a password, wrote `notes`, and
/// synced them up. Returns the phone and the recovery key.
fn first_phone(server: &Arc<MemoryServer>, notes: &[(&str, &str)]) -> (Phone, String) {
    let a = phone(server);
    let made = account(&a)
        .sign_up_password(
            SERVER.into(),
            ACCOUNT.into(),
            PASSWORD.into(),
            "Phone A".into(),
        )
        .unwrap();
    assert!(matches!(made.state, AccountState::SignedIn { .. }));
    for (title, body) in notes {
        note(&a.library, title, body);
    }
    a.library.sync().sync_now().unwrap();
    (a, made.recovery_key.key)
}

/// The passkey of the first phone, without PRF: signs in but can't unlock.
fn passkey_phone(
    server: &Arc<MemoryServer>,
    key: &Arc<TestAuthenticator>,
    notes: &[(&str, &str)],
) -> (Phone, String) {
    let a = phone(server);
    native(&account(&a), key, None, &Arc::default());
    let made = account(&a)
        .passkey_sign_up(SERVER.into(), "ada".into(), "Phone A".into())
        .unwrap();
    assert!(matches!(made.state, AccountState::SignedIn { .. }));
    for (title, body) in notes {
        note(&a.library, title, body);
    }
    a.library.sync().sync_now().unwrap();
    (a, made.recovery_key.key)
}

/// Runs GitHub's device flow on `phone` to its last event.
fn github(phone: &Phone, device: &str) -> Vec<ProviderEvent> {
    let (events, sink) = log();
    account(phone).start_provider(SERVER.into(), ProviderKind::Github, device.into(), sink);
    wait_for(&events, "the provider flow", |events| {
        events.last().is_some_and(is_last)
    })
}

fn is_last(event: &ProviderEvent) -> bool {
    matches!(
        event,
        ProviderEvent::NeedsUsername { .. }
            | ProviderEvent::Finished(_)
            | ProviderEvent::Failed { .. }
            | ProviderEvent::Cancelled
    )
}

/// An account `ada` with a password, GitHub linked, and `notes` synced:
/// the phone that made it and its recovery key. The fake GitHub signs
/// `octocat` in.
fn account_with_github(server: &Arc<MemoryServer>, notes: &[(&str, &str)]) -> (Phone, String) {
    server.set_providers(vec![Provider::Github]);
    server.set_github_device_flow("octocat", "ABCD-1234", 1);
    let (a, key) = first_phone(server, notes);
    server.push_github_answer("authorized");
    let (events, sink) = log();
    a.library
        .account_settings()
        .link_provider(ProviderKind::Github, PASSWORD.into(), sink);
    let events = wait_for(&events, "the link", |events| {
        events.last().is_some_and(is_last)
    });
    assert!(
        matches!(
            events.last(),
            Some(ProviderEvent::Finished(AccountState::SignedIn { .. }))
        ),
        "{events:?}"
    );
    (a, key)
}

/// A phone locked by GitHub on the account of [`account_with_github`].
fn locked_phone(server: &Arc<MemoryServer>) -> Phone {
    server.push_github_answer("authorized");
    let b = phone(server);
    let events = github(&b, "Phone B");
    assert!(
        matches!(
            events.last(),
            Some(ProviderEvent::Finished(AccountState::Locked(_)))
        ),
        "{events:?}"
    );
    b
}

fn locked_info(state: &AccountState) -> &kanso_mobile::LockedInfo {
    match state {
        AccountState::Locked(info) => info,
        other => panic!("expected a locked phone, got {other:?}"),
    }
}

fn signed_in(state: &AccountState) -> bool {
    matches!(state, AccountState::SignedIn { .. })
}

/// A library seeded with one note of its own.
fn phone_with_note(server: &Arc<MemoryServer>, title: &str, body: &str) -> Phone {
    phone_with(server, |store| {
        let notebook = book(store, "Mine");
        raw_note(
            store,
            json!({"title": title, "body": body, "bookId": notebook}),
        );
    })
}

/// The text of every note on `library`.
fn bodies(library: &Library) -> Vec<(String, String)> {
    let mut found: Vec<_> = library
        .list(kanso_mobile::ListQuery::default())
        .unwrap()
        .rows
        .iter()
        .map(|row| {
            let view = library.note(&row.id).unwrap();
            (view.title, view.body.unwrap_or_default())
        })
        .collect();
    found.sort();
    found
}

// ------------------------------------------------------- signing up and in

// no-prop-test: a fixed walk through sign-up, the recovery key, sync, and
// a second phone.
#[test]
fn signing_up_with_a_password_gives_a_recovery_key_and_the_phone_syncs() {
    let server = Arc::new(MemoryServer::new());
    let a = phone(&server);
    let account = account(&a);
    let (states, sink) = log();
    account.state_stream(sink);
    assert_eq!(account.state().unwrap(), AccountState::SignedOut);

    // Offline checks come first and keep nothing.
    for (user, password) in [("x", PASSWORD), (ACCOUNT, "short")] {
        let err = account
            .sign_up_password(
                SERVER.into(),
                user.into(),
                password.into(),
                "Phone A".into(),
            )
            .unwrap_err();
        assert!(matches!(err, KansoError::Invalid { .. }), "{err:?}");
    }
    assert!(a.secrets.items().is_empty());

    let made = account
        .sign_up_password(
            SERVER.into(),
            ACCOUNT.into(),
            PASSWORD.into(),
            "Phone A".into(),
        )
        .unwrap();
    assert_eq!(
        made.state,
        AccountState::SignedIn {
            username: ACCOUNT.into(),
            server: SERVER.into(),
        }
    );
    // The key is the account's: it parses, and shows as the desktop shows it.
    assert!(RecoveryKey::parse(&made.recovery_key.key).is_ok());
    assert_eq!(made.recovery_key.key.matches('-').count(), 6);
    assert_eq!(account.state().unwrap(), made.state);

    let id = note(&a.library, "Written here", "its text");
    let sent = a.library.sync().sync_now().unwrap();
    assert!(sent.pushed >= 1 && sent.pending == 0);
    assert_eq!(a.library.sync().status().unwrap().phase, SyncPhase::Synced);

    // The same name is taken; another phone signs in and reads the note.
    let err = self::account(&phone(&server))
        .sign_up_password(
            SERVER.into(),
            ACCOUNT.into(),
            PASSWORD.into(),
            "Phone Z".into(),
        )
        .unwrap_err();
    assert!(matches!(err, KansoError::Invalid { .. }), "{err:?}");
    let b = phone(&server);
    let state = self::account(&b)
        .sign_in_password(
            SERVER.into(),
            ACCOUNT.into(),
            PASSWORD.into(),
            "Phone B".into(),
        )
        .unwrap();
    assert!(signed_in(&state));
    b.library.sync().sync_now().unwrap();
    assert_eq!(
        b.library.note(&id).unwrap().body.as_deref(),
        Some("its text")
    );

    // A wrong password is an auth error and changes nothing.
    let c = phone(&server);
    let err = self::account(&c)
        .sign_in_password(
            SERVER.into(),
            ACCOUNT.into(),
            "not the right password".into(),
            "C".into(),
        )
        .unwrap_err();
    assert!(matches!(err, KansoError::Auth { .. }), "{err:?}");
    assert_eq!(self::account(&c).state().unwrap(), AccountState::SignedOut);

    // Every change of the state was heard, once.
    let heard = wait_for(&states, "the state stream", |states| states.len() >= 2);
    assert_eq!(heard[0], AccountState::SignedOut);
    assert_eq!(heard[1], made.state);
    a.library.sync().sign_out().unwrap();
    let heard = wait_for(&states, "the sign-out", |states| states.len() >= 3);
    assert_eq!(heard[2], AccountState::SignedOut);
    assert_eq!(heard.len(), 3);
}

// no-prop-test: a fixed walk: the recovery key is kept until it is saved.
#[test]
fn a_recovery_key_is_kept_until_the_person_says_they_saved_it() {
    let server = Arc::new(MemoryServer::new());
    let a = phone(&server);
    let account = account(&a);
    assert_eq!(account.pending_recovery_key().unwrap(), None);
    let made = account
        .sign_up_password(
            SERVER.into(),
            ACCOUNT.into(),
            PASSWORD.into(),
            "Phone A".into(),
        )
        .unwrap();
    // After a restart (or an error) the key can be shown again.
    drop(account);
    let shown = a.library.account().pending_recovery_key().unwrap();
    assert_eq!(shown.as_ref(), Some(&made.recovery_key));
    a.library.account().recovery_key_saved().unwrap();
    assert_eq!(a.library.account().pending_recovery_key().unwrap(), None);
    assert!(
        !a.secrets
            .items()
            .iter()
            .any(|item| item.starts_with("sync-recovery-key:")),
        "{:?}",
        a.secrets.items()
    );
    // Saying it twice, or with no key shown, changes nothing.
    a.library.account().recovery_key_saved().unwrap();
}

// ---------------------------------------------------------- the question

// no-prop-test: a fixed walk through Cancel, Add and Replace.
#[test]
fn a_library_with_notes_of_its_own_asks_and_cancel_add_and_replace_do_what_the_desktop_does() {
    let server = Arc::new(MemoryServer::new());
    let (a, _) = first_phone(&server, &[("From the account", "account text")]);

    // Cancel: the new session is signed out, nothing is bound or kept.
    let b = phone_with_note(&server, "Mine", "my text");
    let acct = account(&b);
    let asked = acct
        .sign_in_password(
            SERVER.into(),
            ACCOUNT.into(),
            PASSWORD.into(),
            "Phone B".into(),
        )
        .unwrap();
    let AccountState::Question(question) = asked.clone() else {
        panic!("a question, got {asked:?}");
    };
    assert_eq!(
        question.answers,
        vec![BindingAnswer::Add, BindingAnswer::Replace]
    );
    assert!(question.body.contains("1 note"), "{question:?}");
    assert_eq!(
        acct.state().unwrap(),
        asked,
        "the question stays until answered"
    );
    // Nothing else starts meanwhile.
    let err = acct
        .sign_in_password(
            SERVER.into(),
            ACCOUNT.into(),
            PASSWORD.into(),
            "Phone B".into(),
        )
        .unwrap_err();
    assert!(matches!(err, KansoError::Invalid { .. }), "{err:?}");
    assert_eq!(acct.answer_binding(None).unwrap(), AccountState::SignedOut);
    assert!(
        !b.secrets
            .items()
            .iter()
            .any(|item| item.starts_with("sync-account:")),
        "{:?}",
        b.secrets.items()
    );
    assert_eq!(
        b.library.sync().status().unwrap().phase,
        SyncPhase::SignedOut
    );
    assert_eq!(bodies(&b.library), vec![("Mine".into(), "my text".into())]);
    // No question: nothing to answer.
    let err = acct.answer_binding(Some(BindingAnswer::Add)).unwrap_err();
    assert!(matches!(err, KansoError::Invalid { .. }), "{err:?}");

    // Add: both sets of notes are in the library, and the account has mine.
    let AccountState::Question(_) = acct
        .sign_in_password(
            SERVER.into(),
            ACCOUNT.into(),
            PASSWORD.into(),
            "Phone B".into(),
        )
        .unwrap()
    else {
        panic!("asked again");
    };
    assert!(signed_in(
        &acct.answer_binding(Some(BindingAnswer::Add)).unwrap()
    ));
    b.library.sync().sync_now().unwrap();
    a.library.sync().sync_now().unwrap();
    let both = vec![
        ("From the account".to_owned(), "account text".to_owned()),
        ("Mine".to_owned(), "my text".to_owned()),
    ];
    assert_eq!(bodies(&b.library), both);
    assert_eq!(bodies(&a.library), both);

    // Replace: the library is backed up, then holds the account's notes.
    let c = phone_with_note(&server, "Only here", "gone soon");
    let acct = account(&c);
    acct.sign_in_password(
        SERVER.into(),
        ACCOUNT.into(),
        PASSWORD.into(),
        "Phone C".into(),
    )
    .unwrap();
    assert!(signed_in(
        &acct.answer_binding(Some(BindingAnswer::Replace)).unwrap()
    ));
    c.library.sync().sync_now().unwrap();
    assert_eq!(bodies(&c.library), both);
    let backups = c.dir.path().join("backups");
    assert!(
        std::fs::read_dir(&backups).is_ok_and(|dir| dir.count() >= 1),
        "a backup was saved in {backups:?}"
    );
}

// no-prop-test: signing up from a library whose notes came from another
// account asks first, and nothing is sent until the answer.
#[test]
fn a_sign_up_over_notes_from_another_account_asks_before_it_makes_anything() {
    let server = Arc::new(MemoryServer::new());
    let (_a, _) = first_phone(&server, &[("Theirs", "text")]);
    let b = phone(&server);
    let acct = account(&b);
    acct.sign_in_password(
        SERVER.into(),
        ACCOUNT.into(),
        PASSWORD.into(),
        "Phone B".into(),
    )
    .unwrap();
    b.library.sync().sync_now().unwrap();
    b.library.sync().sign_out().unwrap();

    let asked = acct
        .sign_up_password(
            SERVER.into(),
            "grace".into(),
            PASSWORD.into(),
            "Phone B".into(),
        )
        .unwrap();
    let AccountState::Question(question) = asked.state.clone() else {
        panic!("a question, got {:?}", asked.state);
    };
    assert_eq!(
        question.answers,
        vec![BindingAnswer::Add],
        "only upload or cancel"
    );
    assert!(asked.recovery_key.key.is_empty(), "no account yet");
    // The account does not exist until the answer.
    let probe = phone(&server);
    let err = account(&probe)
        .sign_in_password(SERVER.into(), "grace".into(), PASSWORD.into(), "P".into())
        .unwrap_err();
    assert!(matches!(err, KansoError::Auth { .. }), "{err:?}");
    // Replace has no meaning for a new account.
    let err = acct
        .answer_binding(Some(BindingAnswer::Replace))
        .unwrap_err();
    assert!(matches!(err, KansoError::Invalid { .. }), "{err:?}");
    assert!(matches!(acct.state().unwrap(), AccountState::Question(_)));

    // Cancel leaves it all as it was.
    assert_eq!(acct.answer_binding(None).unwrap(), AccountState::SignedOut);
    let err = account(&probe)
        .sign_in_password(SERVER.into(), "grace".into(), PASSWORD.into(), "P".into())
        .unwrap_err();
    assert!(matches!(err, KansoError::Auth { .. }), "{err:?}");

    // Upload makes the account; its key is the pending one.
    acct.sign_up_password(
        SERVER.into(),
        "grace".into(),
        PASSWORD.into(),
        "Phone B".into(),
    )
    .unwrap();
    let state = acct.answer_binding(Some(BindingAnswer::Add)).unwrap();
    assert_eq!(
        state,
        AccountState::SignedIn {
            username: "grace".into(),
            server: SERVER.into(),
        }
    );
    let key = acct.pending_recovery_key().unwrap().expect("the new key");
    assert!(RecoveryKey::parse(&key.key).is_ok());
    b.library.sync().sync_now().unwrap();
    assert_eq!(bodies(&b.library), vec![("Theirs".into(), "text".into())]);
}

// ---------------------------------------------------------------- passkeys

// no-prop-test: a fixed run with two phones and one passkey that answers PRF.
#[test]
fn a_passkey_with_prf_signs_up_and_unlocks_a_second_phone_without_approval() {
    let server = Arc::new(MemoryServer::new());
    let key = Arc::new(TestAuthenticator::new(1));
    let seen = Arc::new(Mutex::new(Vec::new()));
    let a = phone(&server);
    native(&account(&a), &key, None, &seen);
    let made = account(&a)
        .passkey_sign_up(SERVER.into(), "ada".into(), "Phone A".into())
        .unwrap();
    assert!(signed_in(&made.state));
    assert!(RecoveryKey::parse(&made.recovery_key.key).is_ok());
    note(&a.library, "Kept", "by the passkey");
    a.library.sync().sync_now().unwrap();
    {
        let requests = seen.lock().unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].kind, PasskeyRequestKind::Register);
    }

    let b = phone(&server);
    native(&account(&b), &key, None, &seen);
    let state = account(&b)
        .passkey_sign_in(SERVER.into(), "Phone B".into())
        .unwrap();
    assert!(signed_in(&state), "the PRF wrap opened the key: {state:?}");
    {
        let requests = seen.lock().unwrap();
        assert_eq!(requests[1].kind, PasskeyRequestKind::Assert);
        let options: Value = serde_json::from_str(&requests[1].options_json).unwrap();
        assert_eq!(options["allow_credentials"], json!([]));
        assert_eq!(options["prf_input"], encode(&kanso_sync::PRF_INPUT));
    }
    b.library.sync().sync_now().unwrap();
    assert_eq!(
        bodies(&b.library),
        vec![("Kept".into(), "by the passkey".into())]
    );
}

// no-prop-test: a fixed run: no sheet, a dismissed sheet and a failed one.
#[test]
fn a_passkey_that_cannot_run_or_is_dismissed_leaves_the_phone_as_it_was() {
    let server = Arc::new(MemoryServer::new());
    let a = phone(&server);
    let acct = account(&a);

    // Nothing listens for sheets.
    let err = acct
        .passkey_sign_in(SERVER.into(), "Phone".into())
        .unwrap_err();
    assert!(matches!(err, KansoError::Invalid { .. }), "{err:?}");
    assert_eq!(acct.state().unwrap(), AccountState::SignedOut);

    // The shell says it couldn't run the sheet, then the person dismissed it.
    let answers = Arc::new(Mutex::new(vec![
        kanso_mobile::PasskeyFailure::Unavailable,
        kanso_mobile::PasskeyFailure::Cancelled,
        kanso_mobile::PasskeyFailure::Failed,
    ]));
    let handle = acct.clone();
    acct.passkey_requests(move |request| {
        let failure = answers.lock().unwrap().remove(0);
        handle.passkey_failed(request.ticket, failure);
        true
    });
    let err = acct
        .passkey_sign_in(SERVER.into(), "Phone".into())
        .unwrap_err();
    assert!(matches!(err, KansoError::Invalid { .. }), "{err:?}");
    assert_eq!(
        acct.passkey_sign_in(SERVER.into(), "Phone".into()).unwrap(),
        AccountState::SignedOut,
        "a dismissed sheet is not an error"
    );
    let made = acct
        .passkey_sign_up(SERVER.into(), "ada".into(), "Phone".into())
        .unwrap_err();
    assert!(matches!(made, KansoError::Invalid { .. }), "{made:?}");
    assert!(
        a.secrets
            .items()
            .iter()
            .all(|item| !item.starts_with("sync-account:"))
    );

    // A ticket nobody waits on takes no answer.
    assert!(!passkey_bridge::answer(u64::MAX, "{}"));
}

// no-prop-test: a fixed run: the sheet is shown for the owner's relying
// party only on the phones' rule; a server's own is fine on a computer.
#[test]
fn the_server_methods_follow_what_the_server_offers() {
    let server = Arc::new(MemoryServer::new());
    let a = phone(&server);
    let acct = account(&a);
    let methods = acct.server_methods(SERVER.into()).unwrap();
    assert!(methods.password && methods.signups_open);
    assert!(methods.passkeys);
    assert!(methods.providers.is_empty());

    server.set_providers(vec![Provider::Github, Provider::Google, Provider::Apple]);
    server.set_passkey_env(None, None);
    server.set_signup(kanso_sync::Signup::Closed);
    let methods = acct.server_methods(SERVER.into()).unwrap();
    assert!(!methods.passkeys && !methods.signups_open);
    assert_eq!(
        methods.providers,
        vec![
            ProviderKind::Github,
            ProviderKind::Google,
            ProviderKind::Apple
        ]
    );
    let err = acct.server_methods("http://127.0.0.1:1".into());
    assert!(err.is_ok() || matches!(err, Err(KansoError::Network { .. })));
}

// ------------------------------------------------------ approval, locked

// no-prop-test: the step's acceptance walk: a new phone joins by device
// approval and decrypts.
#[test]
fn a_new_phone_joins_by_device_approval_and_decrypts() {
    let server = Arc::new(MemoryServer::new());
    let (a, _) = account_with_github(
        &server,
        &[("Secret plans", "only the account can read this")],
    );
    let b = locked_phone(&server);
    let acct = account(&b);

    let state = acct.state().unwrap();
    let info = locked_info(&state);
    assert_eq!(info.username, ACCOUNT);
    assert!(!info.approval_pending);
    // Locked, the phone can't sync.
    assert_eq!(b.library.sync().sync_now(), Err(KansoError::SignedOut));
    assert!(
        b.library
            .list(kanso_mobile::ListQuery::default())
            .unwrap()
            .rows
            .is_empty()
    );

    let (waits, sink) = log::<ApprovalWait>();
    acct.request_approval("Phone B".into(), sink);
    wait_for(&waits, "the request", |waits| {
        waits
            .iter()
            .any(|wait| wait.phase == ApprovalPhase::Waiting)
    });
    assert!(locked_info(&acct.state().unwrap()).approval_pending);

    let code = approve(&server, &session_of(&a.secrets));
    let waits = wait_for(&waits, "the approval", |waits| {
        waits.last().is_some_and(|wait| {
            matches!(
                wait.phase,
                ApprovalPhase::Approved | ApprovalPhase::Failed { .. }
            )
        })
    });
    let last = waits.last().unwrap();
    assert_eq!(last.phase, ApprovalPhase::Approved, "{waits:?}");
    // Both screens compared one code.
    let compare = waits
        .iter()
        .find(|wait| wait.phase == ApprovalPhase::Compare)
        .expect("a code was shown");
    assert_eq!(compare.code.as_deref(), Some(code.as_str()));
    assert!(waits.iter().all(|wait| !wait.offer_recovery), "{waits:?}");
    assert!(waits.iter().any(|wait| wait.other_devices >= 1));

    assert_eq!(
        acct.state().unwrap(),
        AccountState::SignedIn {
            username: ACCOUNT.into(),
            server: SERVER.into(),
        }
    );
    let report = b.library.sync().sync_now().unwrap();
    assert!(report.pulled >= 1);
    assert_eq!(
        bodies(&b.library),
        vec![(
            "Secret plans".into(),
            "only the account can read this".into()
        )]
    );
}

// no-prop-test: cancelling an approval request and giving up on a locked
// sign-in.
#[test]
fn cancelling_the_request_and_then_the_locked_sign_in_leaves_a_signed_out_phone() {
    let server = Arc::new(MemoryServer::new());
    let (_a, _) = account_with_github(&server, &[("N", "t")]);
    let b = locked_phone(&server);
    let acct = account(&b);
    let (waits, sink) = log::<ApprovalWait>();
    acct.request_approval("Phone B".into(), sink);
    wait_for(&waits, "the request", |waits| {
        waits
            .iter()
            .any(|wait| wait.phase == ApprovalPhase::Waiting)
    });
    acct.cancel_approval();
    let waits = wait_for(&waits, "the cancel", |waits| {
        waits
            .last()
            .is_some_and(|wait| wait.phase == ApprovalPhase::Cancelled)
    });
    assert!(
        waits.last().unwrap().offer_recovery,
        "recovery is offered after a cancel"
    );
    assert!(!locked_info(&acct.state().unwrap()).approval_pending);

    assert_eq!(acct.cancel_locked().unwrap(), AccountState::SignedOut);
    assert!(
        b.secrets
            .items()
            .iter()
            .all(|item| !item.starts_with("sync-pending:")),
        "{:?}",
        b.secrets.items()
    );
    // Nothing is locked any more; asking for approval says so.
    let (waits, sink) = log::<ApprovalWait>();
    acct.request_approval("Phone B".into(), sink);
    let waits = wait_for(&waits, "the answer", |waits| !waits.is_empty());
    assert!(
        matches!(waits[0].phase, ApprovalPhase::Failed { .. }),
        "{waits:?}"
    );
}

// no-prop-test: a fixed run: wrong and right password on a locked phone.
#[test]
fn a_locked_phone_opens_with_the_accounts_password() {
    let server = Arc::new(MemoryServer::new());
    let (_a, _) = account_with_github(&server, &[("Open me", "with the password")]);
    let b = locked_phone(&server);
    let acct = account(&b);
    let err = acct
        .unlock_with_password("not the right password".into())
        .unwrap_err();
    assert!(matches!(err, KansoError::Auth { .. }), "{err:?}");
    assert!(matches!(acct.state().unwrap(), AccountState::Locked(_)));
    let err = acct.unlock_with_password("short".into()).unwrap_err();
    assert!(matches!(err, KansoError::Invalid { .. }), "{err:?}");

    assert!(signed_in(
        &acct.unlock_with_password(PASSWORD.into()).unwrap()
    ));
    b.library.sync().sync_now().unwrap();
    assert_eq!(
        bodies(&b.library),
        vec![("Open me".into(), "with the password".into())]
    );
    assert!(
        b.secrets
            .items()
            .iter()
            .all(|item| !item.starts_with("sync-pending:")),
        "the pending item went with the unlock"
    );
}

// no-prop-test: the acceptance walk: a locked phone recovers with the key,
// the old key stops working and a new one is returned.
#[test]
fn a_locked_phone_recovers_with_the_key_and_the_old_key_stops_working() {
    let server = Arc::new(MemoryServer::new());
    let key = Arc::new(TestAuthenticator::new(2).without_prf());
    let (_a, recovery) = passkey_phone(&server, &key, &[("Recoverable", "body text")]);

    let b = phone(&server);
    native(&account(&b), &key, None, &Arc::default());
    let acct = account(&b);
    let state = acct
        .passkey_sign_in(SERVER.into(), "Phone B".into())
        .unwrap();
    let info = locked_info(&state);
    assert!(!info.has_password && !info.approval_pending, "{info:?}");

    // A wrong key is refused; the phone stays locked.
    let other = RecoveryKey::generate().display();
    let err = acct
        .recover(
            SERVER.into(),
            ACCOUNT.into(),
            other,
            PASSWORD.into(),
            "Phone B".into(),
        )
        .unwrap_err();
    assert!(matches!(err, KansoError::Auth { .. }), "{err:?}");
    let err = acct
        .recover(
            SERVER.into(),
            ACCOUNT.into(),
            "nonsense".into(),
            PASSWORD.into(),
            "B".into(),
        )
        .unwrap_err();
    assert!(matches!(err, KansoError::Invalid { .. }), "{err:?}");
    assert!(matches!(acct.state().unwrap(), AccountState::Locked(_)));

    // Server and username given are ignored in favour of the locked session's.
    let made = acct
        .recover(
            "http://elsewhere.invalid".into(),
            "someone".into(),
            recovery.clone(),
            PASSWORD.into(),
            "Phone B".into(),
        )
        .unwrap();
    assert!(signed_in(&made.state), "{:?}", made.state);
    assert_ne!(made.recovery_key.key, recovery, "a new key");
    assert!(RecoveryKey::parse(&made.recovery_key.key).is_ok());
    b.library.sync().sync_now().unwrap();
    assert_eq!(
        bodies(&b.library),
        vec![("Recoverable".into(), "body text".into())]
    );

    // The old key no longer works, from a phone that is signed out; the new
    // one does, and the password it set signs in.
    let c = phone(&server);
    let err = account(&c)
        .recover(
            SERVER.into(),
            ACCOUNT.into(),
            recovery,
            PASSWORD.into(),
            "Phone C".into(),
        )
        .unwrap_err();
    assert!(matches!(err, KansoError::Auth { .. }), "{err:?}");
    let again = account(&c)
        .recover(
            SERVER.into(),
            ACCOUNT.into(),
            made.recovery_key.key.clone(),
            PASSWORD.into(),
            "Phone C".into(),
        )
        .unwrap();
    assert!(signed_in(&again.state));
    assert_ne!(again.recovery_key.key, made.recovery_key.key);
    c.library.sync().sync_now().unwrap();
    assert_eq!(bodies(&c.library), bodies(&b.library));
    let d = phone(&server);
    assert!(signed_in(
        &account(&d)
            .sign_in_password(
                SERVER.into(),
                ACCOUNT.into(),
                PASSWORD.into(),
                "Phone D".into()
            )
            .unwrap()
    ));
}

// no-prop-test: a fixed run: a recovery over notes of its own asks, and the
// key is returned even when the question is cancelled.
#[test]
fn a_recovery_over_notes_of_its_own_asks_and_still_returns_the_new_key() {
    let server = Arc::new(MemoryServer::new());
    let (_a, recovery) = first_phone(&server, &[("Theirs", "t")]);
    let b = phone_with_note(&server, "Mine", "m");
    let acct = account(&b);
    let made = acct
        .recover(
            SERVER.into(),
            ACCOUNT.into(),
            recovery.clone(),
            "a brand new password".into(),
            "B".into(),
        )
        .unwrap();
    assert!(
        matches!(made.state, AccountState::Question(_)),
        "{:?}",
        made.state
    );
    assert!(RecoveryKey::parse(&made.recovery_key.key).is_ok());
    assert_eq!(acct.answer_binding(None).unwrap(), AccountState::SignedOut);
    // The key shown still works, and the used one doesn't.
    assert_eq!(
        acct.pending_recovery_key().unwrap(),
        Some(made.recovery_key.clone())
    );
    let c = phone(&server);
    assert!(signed_in(
        &account(&c)
            .recover(
                SERVER.into(),
                ACCOUNT.into(),
                made.recovery_key.key,
                "yet another password".into(),
                "C".into()
            )
            .unwrap()
            .state
    ));
    let d = phone(&server);
    assert!(
        account(&d)
            .recover(
                SERVER.into(),
                ACCOUNT.into(),
                recovery,
                "and one more password".into(),
                "D".into()
            )
            .is_err()
    );
}

// no-prop-test: a fixed run: closing the library and opening it again.
#[test]
fn a_locked_session_survives_closing_and_reopening_the_library() {
    let server = Arc::new(MemoryServer::new());
    let (a, _) = account_with_github(&server, &[("Persisted", "across a restart")]);
    let b = locked_phone(&server);
    let path = b.path();
    let secrets = b.secrets.clone();
    let before = account(&b).state().unwrap();
    // A request made before the restart, as the app would have left it.
    {
        let id = kanso_sync::new_approval_request_id();
        let library_id = secrets
            .items()
            .into_iter()
            .find_map(|item| item.strip_prefix("sync-pending:").map(str::to_owned))
            .expect("the pending item");
        let locked = kanso_sync::keychain::load_locked_pending(secrets.as_ref(), &library_id)
            .unwrap()
            .unwrap();
        kanso_sync::request_approval_resumable(
            server.as_ref(),
            secrets.as_ref(),
            &locked,
            &id,
            &kanso_sync::ApprovalKeys::generate(),
        )
        .unwrap();
    }
    drop(b.library);
    // The launch check's thread lets go of the library as it ends.
    let end = Instant::now() + Duration::from_secs(10);
    while Library::existing(&path).is_some() {
        assert!(Instant::now() < end, "the library did not close");
        std::thread::sleep(Duration::from_millis(20));
    }

    let reopened = Library::open_for_test(&path, secrets.clone(), server.clone()).unwrap();
    let after = reopened.account().state().unwrap();
    let (before, after) = (locked_info(&before), locked_info(&after));
    assert_eq!(
        (&before.username, &before.server),
        (&after.username, &after.server)
    );
    assert!(
        !before.approval_pending && after.approval_pending,
        "{after:?}"
    );

    // The saved request is resumed, not made again, and is approved.
    let (waits, sink) = log::<ApprovalWait>();
    reopened.account().request_approval("Phone B".into(), sink);
    let approver = session_of(&a.secrets);
    let code = approve(&server, &approver);
    let waits = wait_for(&waits, "the approval", |waits| {
        waits.last().is_some_and(|wait| {
            matches!(
                wait.phase,
                ApprovalPhase::Approved | ApprovalPhase::Failed { .. }
            )
        })
    });
    assert_eq!(
        waits.last().unwrap().phase,
        ApprovalPhase::Approved,
        "{waits:?}"
    );
    assert_eq!(
        waits
            .iter()
            .find(|wait| wait.phase == ApprovalPhase::Compare)
            .and_then(|wait| wait.code.clone()),
        Some(code)
    );
    assert!(signed_in(&reopened.account().state().unwrap()));
    reopened.sync().sync_now().unwrap();
    assert_eq!(
        bodies(&reopened),
        vec![("Persisted".into(), "across a restart".into())]
    );
}

// ------------------------------------------------------------- providers

// no-prop-test: a fixed run of GitHub's device flow for an identity no
// account has, through the username to the key.
#[test]
fn a_github_identity_no_account_has_is_asked_for_a_username_and_makes_an_account() {
    let server = Arc::new(MemoryServer::new());
    server.set_providers(vec![Provider::Github]);
    server.set_github_device_flow("octocat", "ABCD-1234", 1);
    server.push_github_answer("authorized");
    let a = phone(&server);
    let events = github(&a, "Phone A");
    assert_eq!(events.len(), 4, "{events:?}");
    assert_eq!(
        events[0],
        ProviderEvent::ShowCode {
            user_code: "ABCD-1234".into(),
            verification_uri: "https://github.com/login/device".into(),
        }
    );
    assert_eq!(
        events[1],
        ProviderEvent::OpenPage {
            url: "https://github.com/login/device".into(),
            callback_scheme: None,
        }
    );
    assert_eq!(events[2], ProviderEvent::ClosePage);
    let ProviderEvent::NeedsUsername { suggested } = &events[3] else {
        panic!("{events:?}");
    };
    assert!(!suggested.is_empty());

    let acct = account(&a);
    // A username that can't be (too short) fails and can be tried again.
    let err = acct.finish_provider_sign_up("x".into()).unwrap_err();
    assert!(matches!(err, KansoError::Invalid { .. }), "{err:?}");
    let made = acct.finish_provider_sign_up("octocat".into()).unwrap();
    assert_eq!(
        made.state,
        AccountState::SignedIn {
            username: "octocat".into(),
            server: SERVER.into(),
        }
    );
    assert!(RecoveryKey::parse(&made.recovery_key.key).is_ok());
    note(&a.library, "From GitHub", "text");
    a.library.sync().sync_now().unwrap();
    // The ticket was used up.
    let err = acct.finish_provider_sign_up("octocat2".into()).unwrap_err();
    assert!(matches!(err, KansoError::Invalid { .. }), "{err:?}");
}

// no-prop-test: a fixed run: cancelling while the code is shown.
#[test]
fn cancelling_a_provider_flow_ends_it_quietly_and_keeps_nothing() {
    let server = Arc::new(MemoryServer::new());
    server.set_providers(vec![Provider::Github]);
    server.set_github_device_flow("octocat", "ABCD-1234", 1);
    let a = phone(&server);
    let acct = account(&a);
    let (events, sink) = log();
    acct.start_provider(SERVER.into(), ProviderKind::Github, "Phone".into(), sink);
    wait_for(&events, "the code", |events| {
        events
            .iter()
            .any(|event| matches!(event, ProviderEvent::ShowCode { .. }))
    });
    // One flow at a time.
    let (second, sink) = log();
    acct.start_provider(SERVER.into(), ProviderKind::Github, "Phone".into(), sink);
    let second = wait_for(&second, "the refusal", |events| {
        events.last().is_some_and(is_last)
    });
    assert!(
        matches!(second.last(), Some(ProviderEvent::Failed { .. })),
        "{second:?}"
    );

    acct.cancel_provider();
    let events = wait_for(&events, "the end", |events| {
        events.last().is_some_and(is_last)
    });
    assert_eq!(events.last(), Some(&ProviderEvent::Cancelled), "{events:?}");
    assert_eq!(
        events
            .iter()
            .filter(|event| **event == ProviderEvent::ClosePage)
            .count(),
        1
    );
    assert!(a.secrets.items().is_empty());
    assert_eq!(acct.state().unwrap(), AccountState::SignedOut);
    // The slot is free again.
    let err = acct.finish_provider_sign_up("octocat".into()).unwrap_err();
    assert!(matches!(err, KansoError::Invalid { .. }), "{err:?}");
}

// no-prop-test: Apple's sheet: dismissed, and answered for another flow.
#[test]
fn apple_ends_with_its_sheet_and_only_its_own_callback_counts() {
    let server = Arc::new(MemoryServer::new());
    server.set_providers(vec![Provider::Apple]);
    let a = phone(&server);
    let acct = account(&a);

    let (events, sink) = log();
    acct.start_provider(SERVER.into(), ProviderKind::Apple, "Phone".into(), sink);
    let seen = wait_for(&events, "the page", |events| {
        events
            .iter()
            .any(|event| matches!(event, ProviderEvent::OpenPage { .. }))
    });
    assert_eq!(
        seen[0],
        ProviderEvent::OpenPage {
            url: "https://appleid.apple.com/auth/authorize?client_id=memory".into(),
            callback_scheme: Some("kanso".into()),
        }
    );
    acct.provider_page_ended(None);
    let events = wait_for(&events, "the dismissal", |events| {
        events.last().is_some_and(is_last)
    });
    assert_eq!(events.last(), Some(&ProviderEvent::Cancelled), "{events:?}");

    let (events, sink) = log();
    acct.start_provider(SERVER.into(), ProviderKind::Apple, "Phone".into(), sink);
    wait_for(&events, "the page", |events| {
        events
            .iter()
            .any(|event| matches!(event, ProviderEvent::OpenPage { .. }))
    });
    let flow = "A".repeat(43);
    let secret = "b".repeat(43);
    acct.provider_page_ended(Some(format!("kanso://auth/apple?flow={flow}&s={secret}")));
    let events = wait_for(&events, "the refusal", |events| {
        events.last().is_some_and(is_last)
    });
    assert!(
        matches!(events.last(), Some(ProviderEvent::Failed { .. })),
        "{events:?}"
    );
    // With no flow running, a callback is ignored.
    acct.provider_page_ended(Some("kanso://auth/apple?flow=x&s=y".into()));
    acct.provider_page_ended(None);
}

// -------------------------------------------------------- account methods

// no-prop-test: a fixed run: adding and removing a passkey, linking and
// unlinking a provider, with the password as the proof.
#[test]
fn the_ways_in_are_added_and_removed_with_the_password_as_proof() {
    let server = Arc::new(MemoryServer::new());
    server.set_providers(vec![Provider::Github]);
    server.set_github_device_flow("octocat", "ABCD-1234", 1);
    let (a, _) = first_phone(&server, &[]);
    let settings = a.library.account_settings();
    let key = Arc::new(TestAuthenticator::new(3));
    // A sheet the person dismisses adds nothing and isn't an error.
    let dismissing = Arc::new(AtomicBool::new(true));
    let (flag, handle) = (dismissing.clone(), account(&a));
    account(&a).passkey_requests(move |request| {
        if flag.load(Ordering::SeqCst) {
            handle.passkey_failed(request.ticket, kanso_mobile::PasskeyFailure::Cancelled);
        }
        true
    });
    native(&account(&a), &key, None, &Arc::default());
    let unchanged = settings.add_passkey(PASSWORD.into()).unwrap();
    assert!(unchanged.passkeys.is_empty());
    dismissing.store(false, Ordering::SeqCst);

    let added = settings.add_passkey(PASSWORD.into()).unwrap();
    assert_eq!(added.passkeys.len(), 1);
    assert!(added.passkeys[0].can_unlock);
    assert!(added.has_password);
    assert_eq!(added.username, ACCOUNT);
    let id = added.passkeys[0].id.clone();
    let err = settings
        .remove_passkey("not base64!".into(), PASSWORD.into())
        .unwrap_err();
    assert!(matches!(err, KansoError::Invalid { .. }), "{err:?}");
    let removed = settings.remove_passkey(id, PASSWORD.into()).unwrap();
    assert!(removed.passkeys.is_empty());

    // Linking: a wrong password ends in a failure event; the right one links.
    let (events, sink) = log();
    settings.link_provider(ProviderKind::Github, "short".into(), sink);
    let events = wait_for(&events, "the refusal", |events| {
        events.last().is_some_and(is_last)
    });
    assert!(
        matches!(events.last(), Some(ProviderEvent::Failed { .. })),
        "{events:?}"
    );
    server.push_github_answer("authorized");
    let (events, sink) = log();
    settings.link_provider(ProviderKind::Github, PASSWORD.into(), sink);
    let events = wait_for(&events, "the link", |events| {
        events.last().is_some_and(is_last)
    });
    assert!(
        matches!(
            events.last(),
            Some(ProviderEvent::Finished(AccountState::SignedIn { .. }))
        ),
        "{events:?}"
    );
    let after = settings
        .unlink_provider(ProviderKind::Github, PASSWORD.into())
        .unwrap();
    assert!(after.providers.iter().all(|row| !row.linked), "{after:?}");
    assert_eq!(after.providers.len(), 1);
}

// ---------------------------------------------------------------- secrets

/// Every way a PRF output might be written down.
fn spellings(prf: &[u8; 32]) -> Vec<String> {
    let hex: String = prf.iter().map(|b| format!("{b:02x}")).collect();
    vec![
        encode(prf),
        base64::engine::general_purpose::STANDARD.encode(prf),
        base64::engine::general_purpose::STANDARD_NO_PAD.encode(prf),
        hex.to_uppercase(),
        hex,
        format!("{prf:?}"),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(6))]

    /// The PRF output of any passkey stays in Rust: nothing a flow hands the
    /// shell (states, keys, events, errors, requests) spells it, however it
    /// is written, and the passkey it unlocks still works.
    #[test]
    fn prop_the_prf_output_never_crosses_the_facade(prf in any::<[u8; 32]>()) {
        let server = Arc::new(MemoryServer::new());
        let key = Arc::new(TestAuthenticator::new(9));
        let seen = Arc::new(Mutex::new(Vec::new()));
        let mut crossed = String::new();

        let a = phone(&server);
        native(&account(&a), &key, Some(prf), &seen);
        let made = account(&a)
            .passkey_sign_up(SERVER.into(), "ada".into(), "Phone A".into())
            .unwrap();
        crossed += &format!("{made:?}");
        crossed += &format!("{:?}", account(&a).state());
        crossed += &format!("{:?}", account(&a).pending_recovery_key());
        note(&a.library, "n", "t");
        crossed += &format!("{:?}", a.library.sync().sync_now());

        let b = phone(&server);
        native(&account(&b), &key, Some(prf), &seen);
        let signed = account(&b).passkey_sign_in(SERVER.into(), "Phone B".into());
        crossed += &format!("{signed:?}");
        // The same passkey with another PRF output can't open the key: the
        // phone is locked, and says so without spelling either output.
        let c = phone(&server);
        let mut other = prf;
        other[0] ^= 0xff;
        native(&account(&c), &key, Some(other), &seen);
        let locked = account(&c).passkey_sign_in(SERVER.into(), "Phone C".into());
        crossed += &format!("{locked:?}");
        let added = a.library.account_settings().add_passkey(PASSWORD.into());
        crossed += &format!("{added:?}");
        for request in seen.lock().unwrap().iter() {
            crossed += &format!("{request:?}");
            crossed += &request.options_json;
        }

        prop_assert!(signed.as_ref().is_ok_and(signed_in), "{signed:?}");
        for spelling in spellings(&prf).into_iter().chain(spellings(&other)) {
            prop_assert!(!crossed.contains(&spelling), "{spelling} crossed the facade");
        }
        // Nor does it reach the stores' secret items as text of its own: the
        // wrap is ciphertext, and the PRF itself is never written.
        for secrets in [&a.secrets, &b.secrets, &c.secrets] {
            for item in secrets.items() {
                let text = kanso_sync::SecretStore::get(secrets.as_ref(), &item).unwrap().unwrap_or_default();
                for spelling in spellings(&prf) {
                    prop_assert!(!text.contains(&spelling), "{spelling} was stored in {item}");
                }
            }
        }
    }
}

// no-prop-test: a locked note's marker is looked for in what the question
// and the states say; the account calls read no text.
#[test]
fn a_locked_note_is_never_in_what_the_account_calls_say() {
    let server = Arc::new(MemoryServer::new());
    let (_a, _) = first_phone(&server, &[("Theirs", "t")]);
    let b = phone(&server);
    let id = note(&b.library, "Closed note", "SECRET-MARKER-0001 text");
    b.library
        .locks()
        .lock_note(&id, LOCK_PASSWORD.into())
        .unwrap();
    b.library.locks().lock().unwrap();
    let acct = account(&b);
    let asked = acct
        .sign_in_password(SERVER.into(), ACCOUNT.into(), PASSWORD.into(), "B".into())
        .unwrap();
    let AccountState::Question(question) = &asked else {
        panic!("{asked:?}");
    };
    assert!(question.body.contains("1 note"), "{question:?}");
    let mut said = format!("{asked:?}{:?}", acct.state());
    said += &format!("{:?}", acct.answer_binding(Some(BindingAnswer::Add)));
    said += &format!("{:?}{:?}", acct.pending_recovery_key(), acct.state());
    assert!(!said.contains("SECRET-MARKER"), "{said}");
    assert!(
        !said.contains("Closed note"),
        "titles stay out of it too: {said}"
    );
}

// no-prop-test: a provider flow holds the one account task slot.
#[test]
fn one_account_task_runs_at_a_time() {
    let server = Arc::new(MemoryServer::new());
    server.set_providers(vec![Provider::Github]);
    server.set_github_device_flow("octocat", "ABCD-1234", 1);
    let a = phone(&server);
    let acct = account(&a);
    let (events, sink) = log();
    acct.start_provider(SERVER.into(), ProviderKind::Github, "Phone".into(), sink);
    wait_for(&events, "the code", |events| {
        events
            .iter()
            .any(|event| matches!(event, ProviderEvent::ShowCode { .. }))
    });
    let err = acct
        .sign_up_password(
            SERVER.into(),
            ACCOUNT.into(),
            PASSWORD.into(),
            "Phone".into(),
        )
        .unwrap_err();
    assert!(matches!(err, KansoError::Invalid { .. }), "{err:?}");
    acct.cancel_provider();
    wait_for(&events, "the end", |events| {
        events.last().is_some_and(is_last)
    });
}
