//! Biometric unlock: the keys kept for it, the tickets, what a wrong or an
//! invalidated secret does, and that the kept item holds nothing readable.

mod common;

use std::sync::mpsc;

use common::*;
use kanso_mobile::biometrics::{self, answer, status};
use kanso_mobile::{KansoError, LockEvent, LockReason, LockSession, LockState};
use kanso_sync::SecretStore;
use kanso_sync::lock::{Envelope, LockKey, LockPassword};
use proptest::prelude::*;

const SECRET: &str = "the secret plan";
const B: [u8; 32] = [42; 32];

fn enroll(locks: &LockSession, b: &[u8; 32]) -> i32 {
    let ticket = locks.begin_biometric_enrollment().unwrap();
    biometrics::deliver(ticket, status::OK, Some(b))
}

fn unlock(locks: &LockSession, b: &[u8; 32]) -> i32 {
    let ticket = locks.begin_biometric_unlock().unwrap().unwrap();
    biometrics::deliver(ticket, status::OK, Some(b))
}

/// The kept item, as text.
fn item(phone: &Phone) -> Option<String> {
    let id = phone.library.locks().library_id().unwrap();
    phone
        .secrets
        .get(&biometrics::item_name(&id))
        .unwrap()
        .map(|text| text.to_string())
}

fn body(phone: &Phone, id: &str) -> Option<String> {
    phone.library.note(id).unwrap().body
}

// no-prop-test: a fixed sequence of the calls and what each answers.
#[test]
fn biometrics_open_the_notes_after_a_check() {
    let phone = lone_phone();
    let locks = phone.library.locks();
    let id = note(&phone.library, "Plan", SECRET);

    // Nothing to enrol with until the notes are open.
    assert_eq!(locks.begin_biometric_enrollment(), Err(KansoError::Locked));
    assert!(!locks.biometrics_on().unwrap());
    assert_eq!(locks.begin_biometric_unlock(), Ok(None));
    let before = locks.lock_status().unwrap();
    assert!(!before.has_lock_password && !before.unlocked && !before.biometrics_on);

    locks.lock_note(&id, LOCK_PASSWORD.into()).unwrap();
    assert_eq!(enroll(&locks, &B), answer::APPLIED);
    assert!(locks.biometrics_on().unwrap());
    let on = locks.lock_status().unwrap();
    assert!(on.has_lock_password && on.unlocked && on.biometrics_on);

    let (tx, rx) = mpsc::channel();
    locks.lock_events(move |event| tx.send(event).is_ok());
    locks.lock().unwrap();
    assert_eq!(body(&phone, &id), None);
    assert_eq!(unlock(&locks, &B), answer::APPLIED);
    assert_eq!(body(&phone, &id).as_deref(), Some(SECRET));
    assert!(locks.is_unlocked().unwrap());
    // Opening again says nothing more.
    assert_eq!(unlock(&locks, &B), answer::APPLIED);
    let events: Vec<_> = rx.try_iter().collect();
    assert_eq!(
        events,
        vec![
            LockEvent::Locked {
                reason: LockReason::Manual
            },
            LockEvent::Unlocked
        ]
    );

    // The password still works, and turning biometrics off forgets the keys.
    locks.turn_off_biometrics().unwrap();
    assert!(!locks.biometrics_on().unwrap());
    assert_eq!(item(&phone), None);
    assert_eq!(locks.begin_biometric_unlock(), Ok(None));
    locks.lock().unwrap();
    assert_eq!(locks.unlock(LOCK_PASSWORD.into()), Ok(true));
}

// no-prop-test: one ordering of a check and a lock, the race the generation
// counter closes.
#[test]
fn a_check_answered_after_another_lock_opens_nothing() {
    let phone = lone_phone();
    let locks = phone.library.locks();
    let id = note(&phone.library, "Plan", SECRET);
    locks.lock_note(&id, LOCK_PASSWORD.into()).unwrap();
    assert_eq!(enroll(&locks, &B), answer::APPLIED);
    locks.lock().unwrap();

    // The check starts, the app goes to the background and locks again,
    // then the check's answer arrives: it is for a lock that is over.
    let ticket = locks.begin_biometric_unlock().unwrap().unwrap();
    locks.lock_for_background().unwrap();
    biometrics::deliver(ticket, status::OK, Some(&B));
    assert!(!locks.is_unlocked().unwrap());
    assert_eq!(body(&phone, &id), None);

    // A check started after the lock still opens the notes.
    assert_eq!(unlock(&locks, &B), answer::APPLIED);
    assert_eq!(body(&phone, &id).as_deref(), Some(SECRET));
}

// no-prop-test: the ID is one value read before and after a sign-in.
#[test]
fn the_library_id_is_stable_and_not_the_device_name() {
    let server = std::sync::Arc::new(kanso_sync::MemoryServer::new());
    make_account(&server);
    let phone = phone(&server);
    let locks = phone.library.locks();
    let id = locks.library_id().unwrap();
    assert!(id.starts_with("l_"), "{id}");
    assert_eq!(locks.library_id().unwrap(), id);
    // Signing in names the phone, and the ID stays.
    sign_in(&phone, "Ada's iPhone");
    assert_eq!(locks.library_id().unwrap(), id);
    assert!(!id.contains("iPhone"));
}

// no-prop-test: three secrets and what each does to the kept item.
#[test]
fn a_wrong_secret_forgets_the_keys_and_so_does_an_invalidated_one() {
    let phone = lone_phone();
    let locks = phone.library.locks();
    let id = note(&phone.library, "Plan", SECRET);
    locks.lock_note(&id, LOCK_PASSWORD.into()).unwrap();
    assert_eq!(enroll(&locks, &B), answer::APPLIED);
    locks.lock().unwrap();

    // Cancelled, unavailable, failed and a secret of the wrong size change
    // nothing.
    for code in [status::CANCELLED, status::UNAVAILABLE, status::FAILED] {
        let ticket = locks.begin_biometric_unlock().unwrap().unwrap();
        assert_eq!(biometrics::deliver(ticket, code, None), answer::APPLIED);
    }
    let ticket = locks.begin_biometric_unlock().unwrap().unwrap();
    assert_eq!(
        biometrics::deliver(ticket, status::OK, Some(&B[..31])),
        answer::APPLIED
    );
    assert!(locks.biometrics_on().unwrap());
    assert!(!locks.is_unlocked().unwrap());

    // Another secret opens nothing, and the item is gone.
    assert_eq!(unlock(&locks, &[1; 32]), answer::WRONG_SECRET);
    assert!(!locks.is_unlocked().unwrap());
    assert_eq!(item(&phone), None);
    assert_eq!(locks.begin_biometric_unlock(), Ok(None));

    // Enrolled again, an invalidated answer forgets them too.
    assert_eq!(locks.unlock(LOCK_PASSWORD.into()), Ok(true));
    assert_eq!(enroll(&locks, &B), answer::APPLIED);
    locks.lock().unwrap();
    let ticket = locks.begin_biometric_unlock().unwrap().unwrap();
    assert_eq!(
        biometrics::deliver(ticket, status::INVALIDATED, None),
        answer::APPLIED
    );
    assert_eq!(item(&phone), None);
    assert_eq!(locks.begin_biometric_unlock(), Ok(None));
    assert!(!locks.is_unlocked().unwrap());
}

// no-prop-test: the one-shot rule on fixed tickets.
#[test]
fn a_ticket_answers_once_and_an_unknown_one_not_at_all() {
    let phone = lone_phone();
    let locks = phone.library.locks();
    let id = note(&phone.library, "Plan", SECRET);
    locks.lock_note(&id, LOCK_PASSWORD.into()).unwrap();
    let ticket = locks.begin_biometric_enrollment().unwrap();
    assert_eq!(biometrics::deliver(ticket, status::OK, Some(&B)), 1);
    assert_eq!(
        biometrics::deliver(ticket, status::OK, Some(&B)),
        answer::NO_TICKET
    );
    assert_eq!(
        biometrics::deliver(0, status::OK, Some(&B)),
        answer::NO_TICKET
    );
    // An enrolment ticket can't open the notes, nor an unlock ticket enrol.
    locks.lock().unwrap();
    let ticket = locks.begin_biometric_unlock().unwrap().unwrap();
    assert_eq!(locks.begin_biometric_enrollment(), Err(KansoError::Locked));
    assert_eq!(biometrics::deliver(ticket, status::OK, Some(&B)), 1);
    assert!(locks.is_unlocked().unwrap());
}

// no-prop-test: reading the one stored item and the store of secrets.
#[test]
fn the_kept_item_holds_neither_the_password_nor_a_key() {
    let phone = lone_phone();
    let locks = phone.library.locks();
    let id = note(&phone.library, "Plan", SECRET);
    locks.lock_note(&id, LOCK_PASSWORD.into()).unwrap();
    assert_eq!(enroll(&locks, &B), answer::APPLIED);
    let text = item(&phone).unwrap();

    // The key the password gives, in the form the desktop keeps.
    let stored = kanso_store::Store::open(phone.path())
        .unwrap()
        .note(&id)
        .unwrap()
        .unwrap();
    let envelope = Envelope::from_extra(&stored.extra).unwrap().unwrap();
    let key = LockKey::for_envelope(&LockPassword::new(LOCK_PASSWORD).unwrap(), &envelope).unwrap();
    let secret = key.to_secret();
    let fields: Vec<&str> = secret.split(':').collect();
    assert!(!text.contains(LOCK_PASSWORD));
    assert!(!text.contains(secret.as_str()));
    // Neither the key's own text nor any long piece of it.
    assert!(!text.contains(fields[1]), "the key in the clear");
    // The desktop's Touch ID item isn't made here.
    let library_id = locks.library_id().unwrap();
    assert!(
        phone
            .secrets
            .get(&kanso_sync::keychain::lock_keys_item(&library_id))
            .unwrap()
            .is_none()
    );
}

// no-prop-test: two keys of different salts through two unlocks.
#[test]
fn keys_added_while_biometrics_are_on_are_kept_without_a_check() {
    let phone = lone_phone();
    let locks = phone.library.locks();
    let first = note(&phone.library, "One", "ONE-TEXT");
    let second = note(&phone.library, "Two", "TWO-TEXT");
    locks.lock_note(&first, LOCK_PASSWORD.into()).unwrap();
    // A second note locked as another device would: the same password and
    // another salt.
    {
        let store = kanso_store::Store::open(phone.path()).unwrap();
        let other = LockKey::create(
            &LockPassword::new(LOCK_PASSWORD).unwrap(),
            kanso_sync::KdfParams::TEST,
        )
        .unwrap();
        let mut stored = store.note(&second).unwrap().unwrap();
        let sealed = other.seal(&stored.id, &stored.body);
        stored.body.clear();
        Envelope::store(Some(&sealed), &mut stored.extra);
        store
            .lock_note(serde_json::to_value(&stored).unwrap())
            .unwrap();
    }
    assert_eq!(enroll(&locks, &B), answer::APPLIED);
    locks.lock().unwrap();
    assert_eq!(locks.unlock_note(&first, LOCK_PASSWORD.into()), Ok(true));
    assert_eq!(body(&phone, &second), None, "another salt, not open yet");
    assert_eq!(locks.unlock_note(&second, LOCK_PASSWORD.into()), Ok(true));
    locks.lock().unwrap();
    assert_eq!(unlock(&locks, &B), answer::APPLIED);
    assert_eq!(body(&phone, &first).as_deref(), Some("ONE-TEXT"));
    assert_eq!(body(&phone, &second).as_deref(), Some("TWO-TEXT"));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(12))]

    /// Changing the password keeps biometrics working: a note locked before
    /// or after opens with them, and never the old password's text.
    #[test]
    fn prop_a_changed_password_is_kept_for_biometrics(
        new in "[a-z]{8,20}",
        b in prop::array::uniform32(any::<u8>()),
        text in "\\PC{1,40}",
    ) {
        prop_assume!(new != LOCK_PASSWORD);
        let phone = lone_phone();
        let locks = phone.library.locks();
        let before = note(&phone.library, "Before", &text);
        let after = note(&phone.library, "After", "after text");
        locks.lock_note(&before, LOCK_PASSWORD.into()).unwrap();
        prop_assert_eq!(enroll(&locks, &b), answer::APPLIED);
        prop_assert_eq!(locks.change_password(LOCK_PASSWORD.into(), new.clone()), Ok(0));
        locks.lock_note_with_session(&after).unwrap();
        locks.lock().unwrap();
        // The old password opens nothing now.
        prop_assert_eq!(locks.unlock(LOCK_PASSWORD.into()), Ok(false));
        prop_assert_eq!(unlock(&locks, &b), answer::APPLIED);
        prop_assert_eq!(body(&phone, &before), Some(text.clone()));
        prop_assert_eq!(body(&phone, &after), Some("after text".to_owned()));
        locks.lock().unwrap();
        prop_assert_eq!(locks.unlock(new.clone()), Ok(true));
        prop_assert_eq!(phone.library.note(&before).unwrap().lock, LockState::Open);
    }

    /// A secret that isn't `B` never opens anything, and the item is gone
    /// after it.
    #[test]
    fn prop_a_wrong_secret_never_opens(
        b in prop::array::uniform32(any::<u8>()),
        wrong in prop::array::uniform32(any::<u8>()),
    ) {
        prop_assume!(b != wrong);
        let phone = lone_phone();
        let locks = phone.library.locks();
        let id = note(&phone.library, "Plan", SECRET);
        locks.lock_note(&id, LOCK_PASSWORD.into()).unwrap();
        prop_assert_eq!(enroll(&locks, &b), answer::APPLIED);
        locks.lock().unwrap();
        prop_assert_eq!(unlock(&locks, &wrong), answer::WRONG_SECRET);
        prop_assert!(!locks.is_unlocked().unwrap());
        prop_assert_eq!(body(&phone, &id), None);
        prop_assert_eq!(item(&phone), None);
    }

    /// Whatever the passwords, the kept item never contains either.
    #[test]
    fn prop_the_item_never_contains_a_password(
        password in "[a-zA-Z0-9]{8,24}",
        b in prop::array::uniform32(any::<u8>()),
    ) {
        let phone = lone_phone();
        let locks = phone.library.locks();
        let id = note(&phone.library, "Plan", SECRET);
        locks.lock_note(&id, password.clone()).unwrap();
        prop_assert_eq!(enroll(&locks, &b), answer::APPLIED);
        let text = item(&phone).unwrap();
        prop_assert!(!text.contains(&password));
        prop_assert!(!text.contains(SECRET));
    }
}

// no-prop-test: a secret store that refuses once, and the state after it.
#[test]
fn keys_that_cant_be_kept_again_turn_biometrics_off() {
    let phone = lone_phone();
    let locks = phone.library.locks();
    let id = note(&phone.library, "Plan", SECRET);
    locks.lock_note(&id, LOCK_PASSWORD.into()).unwrap();
    assert_eq!(enroll(&locks, &B), answer::APPLIED);
    assert!(locks.biometrics_on().unwrap());

    // The next write (keeping the new key for biometrics) is refused. The
    // password still changes, and the old keys must not stay for biometrics.
    phone.secrets.fail_write(0);
    assert_eq!(
        locks.change_password(LOCK_PASSWORD.into(), "another password".into()),
        Ok(0)
    );
    assert!(!locks.biometrics_on().unwrap());
    assert_eq!(item(&phone), None);
    assert_eq!(locks.begin_biometric_unlock(), Ok(None));
    // The new password opens the notes.
    locks.lock().unwrap();
    assert_eq!(locks.unlock("another password".into()), Ok(true));
    assert_eq!(body(&phone, &id).as_deref(), Some(SECRET));
}
