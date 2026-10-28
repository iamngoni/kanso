//! Settings › Account on the phone: the page's devices, signing other
//! devices out, signing this one out, the password and the recovery key,
//! and deleting the account. Against an in-memory server.

mod common;
mod second_device;

use std::sync::Arc;

use common::*;
use kanso_mobile::{AccountDetails, KansoError, SyncPhase};
use kanso_sync::{
    AccountError, ApprovalKeys, DEFAULT_SERVER, MemoryServer, Password, RecoveryKey, keychain,
};
use proptest::prelude::*;
use second_device::*;

const NEW_PASSWORD: &str = "a different password";

fn server_with_account() -> Arc<MemoryServer> {
    let server = Arc::new(MemoryServer::new());
    make_account(&server);
    server
}

fn signed_in(server: &Arc<MemoryServer>, name: &str) -> Phone {
    let phone = phone(server);
    sign_in(&phone, name);
    phone
}

fn titles(details: &AccountDetails) -> Vec<String> {
    details
        .devices
        .iter()
        .map(|device| device.title.clone())
        .collect()
}

/// The account's recovery key is held by the server.
fn server_holds(server: &MemoryServer, key: &str) -> bool {
    kanso_sync::recovery_key_held(
        server,
        DEFAULT_SERVER,
        ACCOUNT,
        &RecoveryKey::parse(key).unwrap(),
    )
    .unwrap()
}

/// The library's ID, which names its secret-store items.
fn library_id(phone: &Phone) -> String {
    let store = kanso_store::Store::open(phone.path()).unwrap();
    store
        .meta(kanso_sync::meta_keys::LIBRARY_ID)
        .unwrap()
        .unwrap()
}

// no-prop-test: the page of two phones on one account, read from each.
#[test]
fn the_page_lists_the_devices_with_this_phone_first() {
    let server = server_with_account();
    let a = signed_in(&server, "Phone A");
    let b = signed_in(&server, "Phone B");

    let from_a = a.library.account_settings().details().unwrap();
    assert_eq!(from_a.username, ACCOUNT);
    assert_eq!(from_a.server, DEFAULT_SERVER);
    assert!(from_a.has_password);
    assert!(from_a.providers.is_empty() && from_a.passkeys.is_empty());
    assert!(!from_a.recovery_key_unsaved);
    assert!(
        from_a
            .storage_line
            .as_deref()
            .is_some_and(|line| line.contains("items"))
    );
    // The setup device, Phone A and Phone B.
    assert_eq!(from_a.devices.len(), 3, "{:?}", from_a.devices);
    assert!(from_a.devices[0].this_device);
    assert_eq!(from_a.devices[0].title, "Phone A");
    assert_eq!(
        from_a
            .devices
            .iter()
            .filter(|device| device.this_device)
            .count(),
        1
    );
    let names = titles(&from_a);
    assert!(names.contains(&"Phone B".to_owned()), "{names:?}");

    let from_b = b.library.account_settings().details().unwrap();
    assert_eq!(from_b.devices[0].title, "Phone B");
    assert!(from_b.devices[0].this_device);
    assert_eq!(from_b.devices.len(), 3);
}

// no-prop-test: a revoked device's next sync, and the calls refused.
#[test]
fn revoking_another_device_ends_its_session() {
    let server = server_with_account();
    let a = signed_in(&server, "Phone A");
    let b = signed_in(&server, "Phone B");
    let settings = a.library.account_settings();
    let details = settings.details().unwrap();
    let on_b = details
        .devices
        .iter()
        .find(|device| device.title == "Phone B")
        .unwrap();
    let mine = details
        .devices
        .iter()
        .find(|device| device.this_device)
        .unwrap();

    // Not this phone: that is signing out.
    assert!(matches!(
        settings.revoke_device(mine.id.clone()),
        Err(KansoError::Invalid { .. })
    ));
    assert!(matches!(
        settings.revoke_device("nope".into()),
        Err(KansoError::Invalid { .. })
    ));

    let after = settings.revoke_device(on_b.id.clone()).unwrap();
    assert!(!titles(&after).contains(&"Phone B".to_owned()), "{after:?}");
    assert!(after.devices[0].this_device);
    // Revoking it again is fine: the server no longer knows it.
    assert!(settings.revoke_device(on_b.id.clone()).is_ok());

    assert_eq!(b.library.sync().sync_now(), Err(KansoError::SignedOut));
    assert_eq!(
        b.library.sync().status().unwrap().phase,
        SyncPhase::SignedOut
    );
    assert_eq!(
        b.library.account_settings().details(),
        Err(KansoError::SignedOut)
    );
    // This phone is as it was.
    a.library.sync().sync_now().unwrap();
}

// no-prop-test: the server ends this phone's session while the page is open.
#[test]
fn a_page_the_server_ended_signs_this_phone_out() {
    let server = server_with_account();
    let a = signed_in(&server, "Phone A");
    let b = signed_in(&server, "Phone B");
    let details = b.library.account_settings().details().unwrap();
    let on_a = details
        .devices
        .iter()
        .find(|device| device.title == "Phone A")
        .unwrap();
    b.library
        .account_settings()
        .revoke_device(on_a.id.clone())
        .unwrap();

    let statuses = Arc::new(std::sync::Mutex::new(Vec::new()));
    let seen = statuses.clone();
    a.library.sync().status_stream(move |status| {
        seen.lock().unwrap().push(status.phase);
        true
    });
    assert_eq!(
        a.library.account_settings().details(),
        Err(KansoError::SignedOut)
    );
    assert_eq!(
        a.library.sync().status().unwrap().phase,
        SyncPhase::SignedOut
    );
    assert_eq!(statuses.lock().unwrap().last(), Some(&SyncPhase::SignedOut));
}

// no-prop-test: signing out, twice, with a note on the phone.
#[test]
fn signing_out_leaves_the_notes_and_can_be_done_twice() {
    let server = server_with_account();
    let a = signed_in(&server, "Phone A");
    let id = note(&a.library, "Mine", "stays");
    a.library.sync().sync_now().unwrap();
    let settings = a.library.account_settings();

    settings.sign_out().unwrap();
    assert_eq!(
        a.library.sync().status().unwrap().phase,
        SyncPhase::SignedOut
    );
    assert_eq!(settings.details(), Err(KansoError::SignedOut));
    assert!(
        !a.secrets
            .items()
            .iter()
            .any(|item| item.starts_with("sync-account:"))
    );
    assert_eq!(a.library.note(&id).unwrap().title, "Mine");
    settings.sign_out().unwrap();

    // The server was told: the session is gone there too.
    let b = signed_in(&server, "Phone B");
    let seen = b.library.account_settings().details().unwrap();
    assert!(!titles(&seen).contains(&"Phone A".to_owned()), "{seen:?}");
}

// no-prop-test: a sign-in that never opened, with its device request.
#[test]
fn signing_out_ends_a_locked_sign_in_and_its_request() {
    let server = server_with_account();
    let a = signed_in(&server, "Phone A");
    let mut c = new_device(&server, "Phone C");
    // The locked sign-in is what the phone holds, saved under its library.
    kanso_sync::keychain::save_locked_pending(
        c.phone.secrets.as_ref(),
        &c.device.library_id,
        &c.locked,
    )
    .unwrap();
    c.locked =
        kanso_sync::keychain::load_locked_pending(c.phone.secrets.as_ref(), &c.device.library_id)
            .unwrap()
            .unwrap();
    kanso_sync::request_approval_resumable(
        server.as_ref(),
        c.phone.secrets.as_ref(),
        &c.locked,
        &kanso_sync::new_approval_request_id(),
        &ApprovalKeys::generate(),
    )
    .unwrap();
    assert_eq!(a.library.approvals().refresh().unwrap().len(), 1);

    // Not signed in, but locked: the page needs the data key.
    let settings = c.phone.library.account_settings();
    assert!(matches!(settings.details(), Err(KansoError::Auth { .. })));
    settings.sign_out().unwrap();
    assert!(
        !c.phone
            .secrets
            .items()
            .iter()
            .any(|item| item.starts_with("sync-pending:")),
        "{:?}",
        c.phone.secrets.items()
    );
    assert_eq!(settings.details(), Err(KansoError::SignedOut));
    // The request went with it.
    assert!(a.library.approvals().refresh().unwrap().is_empty());
}

// no-prop-test: one change of password and what it does to each device.
#[test]
fn changing_the_password_gives_a_new_key_and_signs_the_others_out() {
    let server = server_with_account();
    let a = signed_in(&server, "Phone A");
    let b = signed_in(&server, "Phone B");
    let settings = a.library.account_settings();

    // Nothing changes on a wrong password, or one that can't be used.
    assert!(matches!(
        settings.change_password("not the password".into(), NEW_PASSWORD.into()),
        Err(KansoError::Auth { .. })
    ));
    assert!(matches!(
        settings.change_password(ACCOUNT_PASSWORD.into(), "short".into()),
        Err(KansoError::Invalid { .. })
    ));
    assert_eq!(settings.pending_recovery_key().unwrap(), None);
    assert!(!settings.details().unwrap().recovery_key_unsaved);

    let key = settings
        .change_password(ACCOUNT_PASSWORD.into(), NEW_PASSWORD.into())
        .unwrap();
    assert_eq!(key.key.split('-').count(), 7, "{}", key.key);
    assert!(server_holds(&server, &key.key));
    // Kept until it is confirmed saved.
    assert!(settings.details().unwrap().recovery_key_unsaved);
    assert_eq!(settings.pending_recovery_key().unwrap().unwrap(), key);

    // Phone B was signed out; this phone still syncs.
    assert_eq!(b.library.sync().sync_now(), Err(KansoError::SignedOut));
    a.library.sync().sync_now().unwrap();
    // The old password no longer opens the account; the new one does.
    let c = phone(&server);
    assert!(matches!(
        c.library.sync().sign_in_password(
            DEFAULT_SERVER.into(),
            ACCOUNT.into(),
            ACCOUNT_PASSWORD.into(),
            "Phone C".into()
        ),
        Err(KansoError::Auth { .. })
    ));
    c.library
        .sync()
        .sign_in_password(
            DEFAULT_SERVER.into(),
            ACCOUNT.into(),
            NEW_PASSWORD.into(),
            "Phone C".into(),
        )
        .unwrap();

    // "I've saved it": the item goes (as the account handle does it).
    keychain::delete_recovery_key(a.secrets.as_ref(), &library_id(&a)).unwrap();
    assert!(!settings.details().unwrap().recovery_key_unsaved);
    assert_eq!(settings.pending_recovery_key().unwrap(), None);
}

// no-prop-test: two new keys in a row; only the last stays good.
#[test]
fn a_new_recovery_key_replaces_the_one_before() {
    let server = server_with_account();
    let a = signed_in(&server, "Phone A");
    let settings = a.library.account_settings();

    assert!(matches!(
        settings.new_recovery_key("not the password".into()),
        Err(KansoError::Auth { .. })
    ));
    assert_eq!(settings.pending_recovery_key().unwrap(), None);

    let first = settings.new_recovery_key(ACCOUNT_PASSWORD.into()).unwrap();
    assert!(server_holds(&server, &first.key));
    // Until it is confirmed saved, asking again gives the same key.
    assert_eq!(
        settings.new_recovery_key(ACCOUNT_PASSWORD.into()).unwrap(),
        first
    );
    // "I've saved it": the item goes (as the account handle does it).
    keychain::delete_recovery_key(a.secrets.as_ref(), &library_id(&a)).unwrap();
    assert_eq!(settings.pending_recovery_key().unwrap(), None);
    let second = settings.new_recovery_key(ACCOUNT_PASSWORD.into()).unwrap();
    assert_ne!(first, second);
    assert!(server_holds(&server, &second.key));
    assert!(!server_holds(&server, &first.key));
    assert_eq!(settings.pending_recovery_key().unwrap().unwrap(), second);
    // Other devices keep their sessions: only the key changed.
    let b = signed_in(&server, "Phone B");
    keychain::delete_recovery_key(a.secrets.as_ref(), &library_id(&a)).unwrap();
    settings.new_recovery_key(ACCOUNT_PASSWORD.into()).unwrap();
    b.library.sync().sync_now().unwrap();
}

// no-prop-test: the refused deletions, then the real one.
#[test]
fn deleting_the_account_needs_its_name_and_password_and_keeps_the_notes() {
    let server = server_with_account();
    let a = signed_in(&server, "Phone A");
    let b = signed_in(&server, "Phone B");
    let id = note(&a.library, "Mine", "stays");
    a.library.sync().sync_now().unwrap();
    let settings = a.library.account_settings();

    // The wrong name is refused before the password is looked at.
    for typed in ["", "grace", "ad"] {
        assert!(
            matches!(
                settings.delete_account(ACCOUNT_PASSWORD.into(), typed.into()),
                Err(KansoError::Invalid { .. })
            ),
            "{typed:?}"
        );
    }
    assert!(matches!(
        settings.delete_account("not the password".into(), ACCOUNT.into()),
        Err(KansoError::Auth { .. })
    ));
    // Still an account.
    settings.details().unwrap();
    a.library.sync().sync_now().unwrap();

    // The name as typed on a phone: any case, around spaces.
    settings
        .delete_account(
            ACCOUNT_PASSWORD.into(),
            format!("  {}  ", ACCOUNT.to_uppercase()),
        )
        .unwrap();
    assert_eq!(
        a.library.sync().status().unwrap().phase,
        SyncPhase::SignedOut
    );
    assert_eq!(settings.details(), Err(KansoError::SignedOut));
    assert_eq!(a.library.note(&id).unwrap().title, "Mine");
    assert_eq!(b.library.sync().sync_now(), Err(KansoError::SignedOut));
    let again = phone(&server);
    assert!(matches!(
        again.library.sync().sign_in_password(
            DEFAULT_SERVER.into(),
            ACCOUNT.into(),
            ACCOUNT_PASSWORD.into(),
            "Phone C".into()
        ),
        Err(KansoError::Auth { .. })
    ));
    assert!(
        !a.secrets
            .items()
            .iter()
            .any(|item| item.starts_with("sync-account:") || item.starts_with("sync-recovery-key:"))
    );
}

// no-prop-test: a locked note on a signed-in phone, and the page.
#[test]
fn nothing_of_a_locked_note_reaches_the_page() {
    let server = server_with_account();
    let a = signed_in(&server, "Phone A");
    let id = note(&a.library, "SECRET-TITLE-23", "SECRET-BODY-23");
    a.library
        .locks()
        .lock_note(&id, LOCK_PASSWORD.into())
        .unwrap();
    a.library.locks().lock().unwrap();
    a.library.sync().sync_now().unwrap();
    let settings = a.library.account_settings();
    let shown = format!(
        "{:?}{:?}",
        settings.details().unwrap(),
        settings.new_recovery_key(ACCOUNT_PASSWORD.into())
    );
    assert!(!shown.contains("SECRET"), "{shown}");
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(6))]

    /// The page holds nothing of the session: not its token, not the
    /// password, in any written form. (The recovery key is shown on
    /// purpose, and only by the calls that make one; the data key is out
    /// of the test's reach, and the page is built from fields that never
    /// touch it.)
    #[test]
    fn prop_the_page_carries_no_secret_of_the_session(
        password in "[b-z]{12,20}",
        name in "[A-Za-z0-9]{1,20}",
    ) {
        use base64::Engine as _;
        use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};

        let server = Arc::new(MemoryServer::new());
        let setup = kanso_sync::DeviceIdentity::from_parts(
            "l_ffffffffffffffffffffffffffffffff", "Setup", [9; 32]);
        kanso_sync::sign_up(
            server.as_ref(),
            &kanso_sync::MemorySecrets::new(),
            DEFAULT_SERVER,
            ACCOUNT,
            &Password::new(&password, ACCOUNT).unwrap(),
            &setup,
            &|_| {},
        )
        .unwrap();
        let phone = phone(&server);
        phone
            .library
            .sync()
            .sign_in_password(DEFAULT_SERVER.into(), ACCOUNT.into(), password.clone(), name.clone())
            .unwrap();
        let settings = phone.library.account_settings();
        let shown = format!("{:?}", settings.details().unwrap());

        let library_id = library_id(&phone);
        let meta = |key| {
            kanso_store::Store::open(phone.path()).unwrap().meta(key).unwrap().unwrap()
        };
        let session = keychain::load_session(
            phone.secrets.as_ref(),
            &library_id,
            &meta(kanso_sync::meta_keys::ORIGIN),
            &meta(kanso_sync::meta_keys::ACCOUNT_ID),
        )
        .unwrap()
        .unwrap();
        let token = session.authorization();
        let token = token.trim_start_matches("Bearer ");
        // The token as the wire writes it, and as it would be written again.
        for secret in [
            token.to_owned(),
            URL_SAFE_NO_PAD.encode(token),
            STANDARD.encode(token),
            password.clone(),
            URL_SAFE_NO_PAD.encode(&password),
        ] {
            prop_assert!(!shown.contains(&secret), "{shown} holds {secret}");
        }
        prop_assert_eq!(&settings.details().unwrap().devices[0].title, &name);
    }
}

// no-prop-test: the error a wrong typed name is, with the desktop's words.
#[test]
fn the_wrong_name_says_why() {
    let words = kanso_engine::account::words::error_words(
        &AccountError::UsernameMismatch,
        kanso_engine::account::words::ErrorContext::CurrentPassword,
    );
    let server = server_with_account();
    let a = signed_in(&server, "Phone A");
    assert_eq!(
        a.library
            .account_settings()
            .delete_account(ACCOUNT_PASSWORD.into(), "grace".into()),
        Err(KansoError::Invalid { message: words })
    );
}
