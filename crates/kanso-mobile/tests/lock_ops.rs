//! What the desktop's lock menus do, as calls: locking with the notes open,
//! unlocking one note or notebook, removing a lock, locking and unlocking a
//! notebook, changing the lock password, and locking when the app leaves.

mod common;

use std::sync::mpsc;

use common::*;
use kanso_mobile::{KansoError, LockEvent, LockReason, LockState, NewNote};
use kanso_store::Store;
use kanso_sync::KdfParams;
use kanso_sync::lock::{Envelope, LockKey, LockPassword};
use proptest::prelude::*;

const SECRET: &str = "the secret plan";
const OTHER_PASSWORD: &str = "another password";

fn body(phone: &Phone, id: &str) -> Option<String> {
    phone.library.note(id).unwrap().body
}

fn note_in(phone: &Phone, book: &str, title: &str, text: &str) -> String {
    phone
        .library
        .create(NewNote {
            notebook_id: Some(book.to_owned()),
            title: title.into(),
            body: text.into(),
            ..Default::default()
        })
        .unwrap()
        .id
}

// no-prop-test: one call and the event it makes.
#[test]
fn leaving_the_app_locks_the_notes_with_its_own_reason() {
    let phone = lone_phone();
    let locks = phone.library.locks();
    let id = note(&phone.library, "Plan", SECRET);
    locks.lock_note(&id, LOCK_PASSWORD.into()).unwrap();
    let (tx, rx) = mpsc::channel();
    locks.lock_events(move |event| tx.send(event).is_ok());
    locks.lock_for_background().unwrap();
    // Nothing is open to close the second time.
    locks.lock_for_background().unwrap();
    assert!(!locks.is_unlocked().unwrap());
    assert_eq!(body(&phone, &id), None);
    assert_eq!(
        rx.try_iter().collect::<Vec<_>>(),
        vec![LockEvent::Locked {
            reason: LockReason::Background
        }]
    );
}

// no-prop-test: a fixed sequence of the calls and what each answers.
#[test]
fn a_note_is_locked_with_the_open_session_and_no_password() {
    let phone = lone_phone();
    let locks = phone.library.locks();
    let (first, second) = (
        note(&phone.library, "One", "ONE"),
        note(&phone.library, "Two", "TWO"),
    );
    // No key is held: there is nothing to lock with.
    assert_eq!(
        locks.lock_note_with_session(&first),
        Err(KansoError::Locked)
    );
    locks.lock_note(&first, LOCK_PASSWORD.into()).unwrap();
    let saved = locks.lock_note_with_session(&second).unwrap();
    assert_eq!(saved.id, second);
    assert_eq!(phone.library.note(&second).unwrap().lock, LockState::Open);
    assert!(matches!(
        locks.lock_note_with_session(&second),
        Err(KansoError::Invalid { .. })
    ));
    assert_eq!(
        locks.lock_note_with_session("note:nope"),
        Err(KansoError::NotFound)
    );
    locks.lock().unwrap();
    assert_eq!(body(&phone, &second), None);
    assert_eq!(locks.unlock(LOCK_PASSWORD.into()), Ok(true));
    assert_eq!(body(&phone, &second).as_deref(), Some("TWO"));
}

// no-prop-test: fixed refusals and the two answers.
#[test]
fn a_password_is_tried_on_the_note_or_notebook_asked_about() {
    let phone = lone_phone();
    let locks = phone.library.locks();
    let id = note(&phone.library, "Plan", SECRET);
    let plain = note(&phone.library, "Plain", "plain");
    let book = phone
        .library
        .create_notebook("Vault".into(), None)
        .unwrap()
        .id;
    locks.lock_note(&id, LOCK_PASSWORD.into()).unwrap();
    locks.lock().unwrap();
    assert_eq!(locks.unlock_note(&id, "not the password".into()), Ok(false));
    assert!(!locks.is_unlocked().unwrap());
    assert!(matches!(
        locks.unlock_note(&id, "short".into()),
        Err(KansoError::Invalid { .. })
    ));
    assert!(matches!(
        locks.unlock_note(&plain, LOCK_PASSWORD.into()),
        Err(KansoError::Invalid { .. })
    ));
    assert_eq!(
        locks.unlock_note("note:nope", LOCK_PASSWORD.into()),
        Err(KansoError::NotFound)
    );
    assert!(matches!(
        locks.unlock_notebook(&book, LOCK_PASSWORD.into()),
        Err(KansoError::Invalid { .. })
    ));
    assert_eq!(
        locks.unlock_notebook("book:nope", LOCK_PASSWORD.into()),
        Err(KansoError::NotFound)
    );
    assert_eq!(locks.unlock_note(&id, LOCK_PASSWORD.into()), Ok(true));
    assert_eq!(body(&phone, &id).as_deref(), Some(SECRET));
}

// no-prop-test: fixed refusals; the property below is the text kept.
#[test]
fn removing_a_note_lock_needs_it_open_and_outside_a_locked_notebook() {
    let phone = lone_phone();
    let locks = phone.library.locks();
    let id = note(&phone.library, "Plan", SECRET);
    assert!(matches!(
        locks.remove_note_lock(&id),
        Err(KansoError::Invalid { .. })
    ));
    locks.lock_note(&id, LOCK_PASSWORD.into()).unwrap();
    locks.lock().unwrap();
    assert_eq!(locks.remove_note_lock(&id), Err(KansoError::Locked));
    assert_eq!(locks.unlock(LOCK_PASSWORD.into()), Ok(true));
    let saved = locks.remove_note_lock(&id).unwrap();
    assert_eq!(saved.id, id);
    let view = phone.library.note(&id).unwrap();
    assert_eq!(view.lock, LockState::Plain);
    assert_eq!(view.rev, saved.rev);
    assert_eq!(view.body.as_deref(), Some(SECRET));
    // Plain again: it is in the store as it was, and shows after a lock.
    let stored = Store::open(phone.path())
        .unwrap()
        .note(&id)
        .unwrap()
        .unwrap();
    assert!(!stored.is_locked());
    assert_eq!(stored.body, SECRET);
    locks.lock().unwrap();
    assert_eq!(body(&phone, &id).as_deref(), Some(SECRET));

    // In a locked notebook the note stays locked.
    let book = phone
        .library
        .create_notebook("Vault".into(), None)
        .unwrap()
        .id;
    let inside = note_in(&phone, &book, "Inside", "INSIDE");
    assert_eq!(
        locks.lock_notebook(&book, Some(LOCK_PASSWORD.into())),
        Ok(1)
    );
    assert_eq!(
        locks.remove_note_lock(&inside),
        Err(KansoError::invalid(
            "The note is in a locked notebook, so it stays locked. Move it out first."
        ))
    );
    assert_eq!(phone.library.note(&inside).unwrap().lock, LockState::Open);
}

// no-prop-test: a fixed sequence over one notebook.
#[test]
fn a_notebook_is_locked_opened_and_unlocked_as_on_the_desktop() {
    let phone = lone_phone();
    let locks = phone.library.locks();
    let work = phone
        .library
        .create_notebook("Work".into(), None)
        .unwrap()
        .id;
    let inner = phone
        .library
        .create_notebook("Acme".into(), Some(work.clone()))
        .unwrap()
        .id;
    let (a, b) = (
        note_in(&phone, &work, "A", "A-TEXT"),
        note_in(&phone, &inner, "B", "B-TEXT"),
    );
    let elsewhere = phone
        .library
        .create_notebook("Home".into(), None)
        .unwrap()
        .id;
    let outside = note_in(&phone, &elsewhere, "Outside", "OUTSIDE");
    assert!(locks.notebook_is_open(&work).unwrap());
    assert_eq!(
        locks.notebook_is_open("book:nope"),
        Err(KansoError::NotFound)
    );

    // No key and no password: nothing to lock with. Too short a password too.
    assert_eq!(locks.lock_notebook(&work, None), Err(KansoError::Locked));
    assert!(matches!(
        locks.lock_notebook(&work, Some("short".into())),
        Err(KansoError::Invalid { .. })
    ));
    assert_eq!(
        locks.lock_notebook("book:nope", Some(LOCK_PASSWORD.into())),
        Err(KansoError::NotFound)
    );
    // The first lock sets the password; both notes and the inner notebook's
    // note are sealed, and stay open for the session.
    assert_eq!(
        locks.lock_notebook(&work, Some(LOCK_PASSWORD.into())),
        Ok(2)
    );
    assert!(locks.is_unlocked().unwrap());
    assert!(locks.notebook_is_open(&work).unwrap());
    assert!(locks.notebook_is_open(&inner).unwrap());
    assert_eq!(phone.library.note(&a).unwrap().lock, LockState::Open);
    assert!(matches!(
        locks.lock_notebook(&work, None),
        Err(KansoError::Invalid { .. })
    ));
    assert!(matches!(
        locks.lock_notebook(&inner, None),
        Err(KansoError::Invalid { .. })
    ));

    locks.lock().unwrap();
    assert!(!locks.notebook_is_open(&work).unwrap());
    assert!(!locks.notebook_is_open(&inner).unwrap());
    assert!(
        locks
            .notebook_is_open(&phone_book(&phone, &outside))
            .unwrap()
    );
    assert_eq!(body(&phone, &a), None);
    assert_eq!(body(&phone, &b), None);
    assert_eq!(locks.remove_notebook_lock(&work), Err(KansoError::Locked));
    assert_eq!(
        locks.unlock_notebook(&work, "wrong password".into()),
        Ok(false)
    );
    assert!(!locks.notebook_is_open(&work).unwrap());
    assert_eq!(locks.unlock_notebook(&work, LOCK_PASSWORD.into()), Ok(true));
    assert!(locks.notebook_is_open(&work).unwrap());
    assert_eq!(body(&phone, &b).as_deref(), Some("B-TEXT"));

    // A note made there afterwards is sealed.
    let later = note_in(&phone, &work, "Later", "LATER");
    assert!(
        Store::open(phone.path())
            .unwrap()
            .note(&later)
            .unwrap()
            .unwrap()
            .is_locked()
    );

    // Removing the lock needs the notebook open, leaves its notes locked and
    // doesn't lock what is put there later. An inner notebook has no lock of
    // its own to remove.
    assert!(matches!(
        locks.remove_notebook_lock(&inner),
        Err(KansoError::Invalid { .. })
    ));
    locks.remove_notebook_lock(&work).unwrap();
    assert!(
        phone
            .library
            .notebooks()
            .unwrap()
            .iter()
            .all(|row| !row.locked)
    );
    let after = note_in(&phone, &work, "After", "AFTER");
    assert_eq!(phone.library.note(&after).unwrap().lock, LockState::Plain);
    assert_eq!(phone.library.note(&a).unwrap().lock, LockState::Open);
    locks.lock().unwrap();
    assert_eq!(body(&phone, &a), None, "its notes stay locked");
    assert_eq!(body(&phone, &after).as_deref(), Some("AFTER"));
    assert!(matches!(
        locks.remove_notebook_lock(&work),
        Err(KansoError::Invalid { .. })
    ));
}

/// The notebook note `id` is in.
fn phone_book(phone: &Phone, id: &str) -> String {
    phone.library.note(id).unwrap().notebook_id
}

// no-prop-test: a second notebook with a lock already made.
#[test]
fn the_password_of_a_second_notebook_must_be_the_first() {
    let phone = lone_phone();
    let locks = phone.library.locks();
    let (one, two) = (
        phone
            .library
            .create_notebook("One".into(), None)
            .unwrap()
            .id,
        phone
            .library
            .create_notebook("Two".into(), None)
            .unwrap()
            .id,
    );
    note_in(&phone, &one, "A", "A");
    note_in(&phone, &two, "B", "B");
    locks
        .lock_notebook(&one, Some(LOCK_PASSWORD.into()))
        .unwrap();
    locks.lock().unwrap();
    assert!(matches!(
        locks.lock_notebook(&two, Some(OTHER_PASSWORD.into())),
        Err(KansoError::Auth { .. })
    ));
    assert!(!locks.is_unlocked().unwrap());
    assert_eq!(locks.lock_notebook(&two, Some(LOCK_PASSWORD.into())), Ok(1));
    assert!(locks.is_unlocked().unwrap());
}

// no-prop-test: a fixed sequence and its counted refusals.
#[test]
fn the_lock_password_is_changed_and_what_it_didnt_open_is_counted() {
    let phone = lone_phone();
    let locks = phone.library.locks();
    let id = note(&phone.library, "Plan", SECRET);
    let stray = note(&phone.library, "Stray", "STRAY");
    assert!(matches!(
        locks.change_password(LOCK_PASSWORD.into(), OTHER_PASSWORD.into()),
        Err(KansoError::Invalid { .. })
    ));
    locks.lock_note(&id, LOCK_PASSWORD.into()).unwrap();
    // A note locked with another password, as another device might have.
    {
        let store = Store::open(phone.path()).unwrap();
        let key = LockKey::create(
            &LockPassword::new("unrelated words").unwrap(),
            KdfParams::TEST,
        )
        .unwrap();
        let mut stored = store.note(&stray).unwrap().unwrap();
        let sealed = key.seal(&stored.id, &stored.body);
        stored.body.clear();
        Envelope::store(Some(&sealed), &mut stored.extra);
        store
            .lock_note(serde_json::to_value(&stored).unwrap())
            .unwrap();
    }
    locks.lock().unwrap();

    let wrong = locks.change_password("not the password".into(), OTHER_PASSWORD.into());
    assert_eq!(
        wrong,
        Err(KansoError::auth("That isn't the lock password."))
    );
    assert_eq!(
        locks.change_password("short".into(), OTHER_PASSWORD.into()),
        Err(KansoError::auth("That isn't the lock password."))
    );
    assert!(matches!(
        locks.change_password(LOCK_PASSWORD.into(), "short".into()),
        Err(KansoError::Invalid { .. })
    ));
    assert!(!locks.is_unlocked().unwrap());

    assert_eq!(
        locks.change_password(LOCK_PASSWORD.into(), OTHER_PASSWORD.into()),
        Ok(1)
    );
    // The new key is held: the notes are open.
    assert!(locks.is_unlocked().unwrap());
    assert_eq!(body(&phone, &id).as_deref(), Some(SECRET));
    locks.lock().unwrap();
    assert_eq!(locks.unlock_note(&id, LOCK_PASSWORD.into()), Ok(false));
    assert_eq!(locks.unlock_note(&id, OTHER_PASSWORD.into()), Ok(true));
    assert_eq!(body(&phone, &stray), None, "it keeps its own password");
}

/// Seals note `id` with a password of its own, as another device might.
fn lock_with_foreign_password(phone: &Phone, id: &str, password: &str) {
    let store = Store::open(phone.path()).unwrap();
    let key = LockKey::create(&LockPassword::new(password).unwrap(), KdfParams::TEST).unwrap();
    let mut stored = store.note(id).unwrap().unwrap();
    let sealed = key.seal(&stored.id, &stored.body);
    stored.body.clear();
    Envelope::store(Some(&sealed), &mut stored.extra);
    store
        .lock_note(serde_json::to_value(&stored).unwrap())
        .unwrap();
}

// no-prop-test: a fixed sequence; two keys in the session and which one seals.
#[test]
fn a_note_is_sealed_with_the_password_typed_not_the_sessions_first_key() {
    const FOREIGN: &str = "unrelated words";
    let phone = lone_phone();
    let locks = phone.library.locks();
    let (plan, stray) = (
        note(&phone.library, "Plan", SECRET),
        note(&phone.library, "Stray", "STRAY"),
    );
    locks.lock_note(&plan, LOCK_PASSWORD.into()).unwrap();
    lock_with_foreign_password(&phone, &stray, FOREIGN);
    locks.lock().unwrap();
    // The change skips the stray note, which keeps its password.
    assert_eq!(
        locks.change_password(LOCK_PASSWORD.into(), OTHER_PASSWORD.into()),
        Ok(1)
    );
    // The stray note is opened with its own password: the session now holds
    // that key first, and the new one is added after it.
    locks.lock().unwrap();
    assert_eq!(locks.unlock_note(&stray, FOREIGN.into()), Ok(true));
    let fresh = note(&phone.library, "Fresh", "FRESH");
    locks.lock_note(&fresh, OTHER_PASSWORD.into()).unwrap();

    // The fresh note opens with the new password only.
    locks.lock().unwrap();
    assert_eq!(locks.unlock_note(&fresh, FOREIGN.into()), Ok(false));
    assert_eq!(locks.unlock_note(&fresh, OTHER_PASSWORD.into()), Ok(true));
    assert_eq!(body(&phone, &fresh).as_deref(), Some("FRESH"));

    // With both keys held, the session's own key for a note is the one most
    // locks use (the new password's), not the stray's.
    locks.lock().unwrap();
    assert_eq!(locks.unlock_note(&stray, FOREIGN.into()), Ok(true));
    assert_eq!(locks.unlock_note(&plan, OTHER_PASSWORD.into()), Ok(true));
    let another = note(&phone.library, "Another", "ANOTHER");
    locks.lock_note_with_session(&another).unwrap();
    locks.lock().unwrap();
    assert_eq!(locks.unlock_note(&another, FOREIGN.into()), Ok(false));
    assert_eq!(locks.unlock_note(&another, OTHER_PASSWORD.into()), Ok(true));
}

// no-prop-test: a fixed sequence; a stale key in the session and a new lock
// password.
#[test]
fn a_stale_session_key_never_seals_when_every_lock_is_gone() {
    let phone = lone_phone();
    let locks = phone.library.locks();
    let book = phone
        .library
        .create_notebook("Shelf".into(), None)
        .unwrap()
        .id;
    let (first, second, third) = (
        note(&phone.library, "One", "ONE"),
        note(&phone.library, "Two", "TWO"),
        note_in(&phone, &book, "Three", "THREE"),
    );
    locks.lock_note(&first, LOCK_PASSWORD.into()).unwrap();
    locks.remove_note_lock(&first).unwrap();
    // The session still holds the first password's key, but no lock uses it.
    assert!(locks.is_unlocked().unwrap());
    assert!(!locks.lock_status().unwrap().has_lock_password);
    assert_eq!(
        locks.lock_note_with_session(&second),
        Err(KansoError::Locked)
    );
    assert_eq!(locks.lock_notebook(&book, None), Err(KansoError::Locked));

    // A new lock password P2 seals the note, not the stale key.
    locks.lock_note(&second, OTHER_PASSWORD.into()).unwrap();
    locks.lock().unwrap();
    assert_eq!(locks.unlock_note(&second, LOCK_PASSWORD.into()), Ok(false));
    assert_eq!(locks.unlock_note(&second, OTHER_PASSWORD.into()), Ok(true));

    // Same for a notebook: its password, typed while a stale key is held. The
    // first notebook is the default, so all three notes are in it.
    locks.remove_note_lock(&second).unwrap();
    assert!(!locks.lock_status().unwrap().has_lock_password);
    assert_eq!(
        locks.lock_notebook(&book, Some(LOCK_PASSWORD.into())),
        Ok(3)
    );
    locks.lock().unwrap();
    assert_eq!(locks.unlock_note(&third, OTHER_PASSWORD.into()), Ok(false));
    assert_eq!(locks.unlock_note(&third, LOCK_PASSWORD.into()), Ok(true));
}

// no-prop-test: the status as the calls move it.
#[test]
fn the_status_follows_the_session() {
    let phone = lone_phone();
    let locks = phone.library.locks();
    let status = |locks: &kanso_mobile::LockSession| {
        let s = locks.lock_status().unwrap();
        (s.has_lock_password, s.unlocked, s.biometrics_on)
    };
    assert_eq!(status(&locks), (false, false, false));
    let id = note(&phone.library, "Plan", SECRET);
    locks.lock_note(&id, LOCK_PASSWORD.into()).unwrap();
    assert_eq!(status(&locks), (true, true, false));
    locks.lock().unwrap();
    assert_eq!(status(&locks), (true, false, false));
    locks.unlock(LOCK_PASSWORD.into()).unwrap();
    locks.remove_note_lock(&id).unwrap();
    // Nothing is locked now, though the session still holds the key.
    assert_eq!(status(&locks), (false, true, false));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    /// A notebook locked seals every note in it and in the notebooks inside
    /// it and no other; they close with the session, open again with the
    /// password to the text they had, and keep their locks when the
    /// notebook's is removed.
    #[test]
    fn prop_a_locked_notebook_seals_what_is_in_it(
        texts in prop::collection::vec("\\PC{1,30}", 1..5),
        inside_inner in prop::collection::vec(any::<bool>(), 5),
        outside_text in "\\PC{1,30}",
    ) {
        let phone = lone_phone();
        let locks = phone.library.locks();
        let work = phone.library.create_notebook("Work".into(), None).unwrap().id;
        let inner = phone
            .library
            .create_notebook("Inner".into(), Some(work.clone()))
            .unwrap()
            .id;
        let ids: Vec<String> = texts
            .iter()
            .enumerate()
            .map(|(i, text)| {
                let book = if inside_inner[i] { &inner } else { &work };
                note_in(&phone, book, &format!("N{i}"), text)
            })
            .collect();
        let elsewhere = phone.library.create_notebook("Home".into(), None).unwrap().id;
        let outside = note_in(&phone, &elsewhere, "Outside", &outside_text);
        prop_assert_eq!(
            locks.lock_notebook(&work, Some(LOCK_PASSWORD.into())),
            Ok(texts.len() as u32)
        );
        locks.lock().unwrap();
        for id in &ids {
            prop_assert_eq!(body(&phone, id), None);
        }
        prop_assert_eq!(body(&phone, &outside), Some(outside_text.clone()));
        prop_assert_eq!(locks.unlock_notebook(&inner, LOCK_PASSWORD.into()), Ok(true));
        for (id, text) in ids.iter().zip(&texts) {
            prop_assert_eq!(body(&phone, id), Some(text.clone()));
        }
        locks.remove_notebook_lock(&work).unwrap();
        locks.lock().unwrap();
        for id in &ids {
            prop_assert_eq!(body(&phone, id), None);
        }
    }

    /// Removing a note's lock keeps its text exactly, as plain text.
    #[test]
    fn prop_removing_a_lock_keeps_the_text(text in "\\PC{0,80}") {
        let phone = lone_phone();
        let locks = phone.library.locks();
        let id = note(&phone.library, "Plan", &text);
        locks.lock_note(&id, LOCK_PASSWORD.into()).unwrap();
        locks.remove_note_lock(&id).unwrap();
        let stored = Store::open(phone.path()).unwrap().note(&id).unwrap().unwrap();
        prop_assert!(!stored.is_locked());
        prop_assert_eq!(stored.body, text.clone());
        locks.lock().unwrap();
        prop_assert_eq!(body(&phone, &id), Some(text));
    }
}
