//! The lock session: unlocking, locking, locking a note, the events and the
//! auto-lock, and what the store is left holding.

mod common;

use std::sync::mpsc;
use std::time::{Duration, Instant};

use common::*;
use kanso_engine::lock::AUTO_LOCK;
use kanso_mobile::{KansoError, ListPlace, ListQuery, LockEvent, LockReason, LockState, NewNote};
use kanso_store::{SearchParams, Store};
use kanso_sync::KdfParams;
use kanso_sync::lock::{LockKey, LockPassword};
use proptest::prelude::*;
use serde_json::json;

const SECRET: &str = "the secret plan";

// no-prop-test: a fixed sequence of the calls and what each answers.
#[test]
fn a_locked_note_closes_and_opens_with_the_password() {
    let phone = lone_phone();
    let locks = phone.library.locks();
    let id = note(&phone.library, "Plan", SECRET);

    // Nothing is locked yet, so there is no password to try.
    assert!(matches!(
        locks.unlock(LOCK_PASSWORD.into()),
        Err(KansoError::Invalid { .. })
    ));
    assert!(matches!(
        locks.lock_note(&id, "short".into()),
        Err(KansoError::Invalid { .. })
    ));
    assert!(!locks.is_unlocked().unwrap());

    // Locking the first note sets the password and leaves the note open.
    let saved = locks.lock_note(&id, LOCK_PASSWORD.into()).unwrap();
    assert_eq!(saved.id, id);
    assert!(locks.is_unlocked().unwrap());
    assert!(locks.is_note_unlocked(&id).unwrap());
    let open = phone.library.note(&id).unwrap();
    assert_eq!(open.lock, LockState::Open);
    assert_eq!(open.body.as_deref(), Some(SECRET));
    assert_eq!(open.rev, saved.rev);

    locks.lock().unwrap();
    assert!(!locks.is_unlocked().unwrap());
    assert!(!locks.is_note_unlocked(&id).unwrap());
    let closed = phone.library.note(&id).unwrap();
    assert_eq!(closed.lock, LockState::Closed { reason: None });
    assert_eq!(closed.body, None);
    assert_eq!(closed.title, "Plan");
    assert_eq!(
        phone.library.save(&id, &closed.rev, "overwritten".into()),
        Err(KansoError::Locked)
    );

    assert_eq!(locks.unlock("not the password".into()), Ok(false));
    assert!(!locks.is_unlocked().unwrap());
    assert_eq!(locks.unlock(LOCK_PASSWORD.into()), Ok(true));
    assert_eq!(
        phone.library.note(&id).unwrap().body.as_deref(),
        Some(SECRET)
    );

    // A second note needs the same password.
    let other = note(&phone.library, "Other", "also secret");
    locks.lock().unwrap();
    assert!(matches!(
        locks.lock_note(&other, "another password".into()),
        Err(KansoError::Auth { .. })
    ));
    assert!(!locks.is_unlocked().unwrap());
    locks.lock_note(&other, LOCK_PASSWORD.into()).unwrap();
    // Both are open now: the password was the same.
    assert!(locks.is_note_unlocked(&id).unwrap());
    // Locking a locked note again is refused.
    assert!(matches!(
        locks.lock_note(&other, LOCK_PASSWORD.into()),
        Err(KansoError::Invalid { .. })
    ));
    assert_eq!(
        locks.is_note_unlocked("note:nope"),
        Err(KansoError::NotFound)
    );
}

// no-prop-test: the events are a sequence of the calls above.
#[test]
fn the_notes_opening_and_closing_are_events() {
    let phone = lone_phone();
    let locks = phone.library.locks();
    let (tx, rx) = mpsc::channel();
    locks.lock_events(move |event| tx.send(event).is_ok());
    let id = note(&phone.library, "Plan", SECRET);
    locks.lock_note(&id, LOCK_PASSWORD.into()).unwrap();
    locks.lock().unwrap();
    // Locking again says nothing.
    locks.lock().unwrap();
    assert_eq!(locks.unlock(LOCK_PASSWORD.into()), Ok(true));
    // Typing it again while open says nothing either.
    assert_eq!(locks.unlock(LOCK_PASSWORD.into()), Ok(true));
    // The auto-lock time closes them, only once it has passed.
    assert!(!locks.lock_if_idle(Instant::now()));
    assert!(locks.lock_if_idle(Instant::now() + AUTO_LOCK + Duration::from_secs(1)));
    assert!(!locks.lock_if_idle(Instant::now() + AUTO_LOCK * 2));
    let events: Vec<_> = rx.try_iter().collect();
    assert_eq!(
        events,
        vec![
            LockEvent::Unlocked,
            LockEvent::Locked {
                reason: LockReason::Manual
            },
            LockEvent::Unlocked,
            LockEvent::Locked {
                reason: LockReason::Idle
            },
        ]
    );
}

// no-prop-test: three reads of the files the phone keeps.
#[test]
fn the_store_only_ever_holds_a_locked_note_sealed() {
    let phone = lone_phone();
    let locks = phone.library.locks();
    let id = note(&phone.library, "Plan", "FIRST-DRAFT-TEXT");
    // An earlier saved version is in history in plain text.
    let rev = phone.library.note(&id).unwrap().rev;
    phone
        .library
        .save(&id, &rev, "SECOND-DRAFT-TEXT".into())
        .unwrap();
    let rev = locks.lock_note(&id, LOCK_PASSWORD.into()).unwrap().rev;
    let rev = phone
        .library
        .save(&id, &rev, "THIRD-DRAFT-TEXT".into())
        .unwrap()
        .rev;
    assert!(!rev.is_empty());

    // Another connection to the same file sees what is stored.
    let store = Store::open(phone.path()).unwrap();
    let stored = store.note(&id).unwrap().unwrap();
    assert!(stored.is_locked());
    assert_eq!(stored.body, "");
    let raw = serde_json::to_string(&stored).unwrap();
    for text in ["FIRST-DRAFT", "SECOND-DRAFT", "THIRD-DRAFT"] {
        assert!(!raw.contains(text), "{text} in the stored note");
        let found = store
            .search_summaries(&SearchParams {
                query: kanso_core::Query::parse(text),
                ..SearchParams::default()
            })
            .unwrap();
        assert!(found.is_empty(), "{text} is searchable");
    }
    // The earlier versions were deleted when it was locked.
    assert!(store.revisions(&id).unwrap().iter().all(|revision| {
        !serde_json::to_string(&revision.fields)
            .unwrap()
            .contains("DRAFT")
    }));
    // A saved text is read back from the seal.
    assert_eq!(
        phone.library.note(&id).unwrap().body.as_deref(),
        Some("THIRD-DRAFT-TEXT")
    );
}

/// A library whose notebook "Vault" was locked before it was opened, with
/// two notes in it.
fn vault() -> (Phone, String, [String; 2]) {
    let mut ids = [String::new(), String::new()];
    let mut vault = String::new();
    let phone = phone_with(
        &std::sync::Arc::new(kanso_sync::MemoryServer::new()),
        |store| {
            vault = book(store, "Vault");
            for (i, id) in ids.iter_mut().enumerate() {
                *id = raw_note(
                    store,
                    json!({"title": format!("Vaulted {i}"), "body": format!("VAULT-TEXT-{i}"), "bookId": vault}),
                );
            }
            let key = LockKey::create(&LockPassword::new(LOCK_PASSWORD).unwrap(), KdfParams::TEST)
                .unwrap();
            assert_eq!(
                kanso_engine::lock::seal_notebook(store, &vault, &key),
                Ok(2)
            );
        },
    );
    (phone, vault, ids)
}

// no-prop-test: a fixed notebook locked before the library opens.
#[test]
fn the_notes_of_a_locked_notebook_are_locked_with_it() {
    let (phone, vault, ids) = vault();
    let locks = phone.library.locks();
    let books = phone.library.notebooks().unwrap();
    assert!(books.iter().any(|b| b.id == vault && b.locked));
    let rows = phone.library.list(ListQuery::default()).unwrap().rows;
    assert_eq!(rows.len(), 2);
    for row in &rows {
        assert!(row.locked);
        assert!(row.snippet.is_empty());
        assert!(row.title.starts_with("Vaulted"));
    }
    assert!(!locks.is_note_unlocked(&ids[0]).unwrap());
    let into_vault = NewNote {
        notebook_id: Some(vault.clone()),
        title: "New".into(),
        body: "NEW-VAULT-TEXT".into(),
        ..NewNote::default()
    };
    assert_eq!(
        phone.library.create(into_vault.clone()),
        Err(KansoError::Locked)
    );
    // Made from the locked notebook's own list is the same refusal, with
    // nothing written; a preference for it is passed over for another.
    let from_its_list = NewNote {
        place: Some(ListPlace::Notebook { id: vault.clone() }),
        ..into_vault.clone()
    };
    assert_eq!(
        phone.library.create(NewNote {
            notebook_id: None,
            ..from_its_list
        }),
        Err(KansoError::Locked)
    );
    assert_eq!(locks.unlock("wrong password".into()), Ok(false));
    assert_eq!(locks.unlock(LOCK_PASSWORD.into()), Ok(true));
    assert_eq!(
        phone.library.note(&ids[1]).unwrap().body.as_deref(),
        Some("VAULT-TEXT-1")
    );
    let made = phone.library.create(into_vault).unwrap();
    let view = phone.library.note(&made.id).unwrap();
    assert_eq!(view.lock, LockState::Open);
    assert_eq!(view.body.as_deref(), Some("NEW-VAULT-TEXT"));
    let store = Store::open(phone.path()).unwrap();
    assert_eq!(store.note(&made.id).unwrap().unwrap().body, "");
    locks.lock().unwrap();
    assert!(phone.library.note(&made.id).unwrap().body.is_none());
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    /// A text locked and opened again is the text, whatever it is: the
    /// seal round-trips through the store, and a note made plain by nothing
    /// else stays sealed.
    #[test]
    fn prop_a_locked_text_comes_back_as_it_was(body in "\\PC{0,80}", edit in "\\PC{0,80}") {
        let phone = lone_phone();
        let locks = phone.library.locks();
        let id = note(&phone.library, "Plan", &body);
        locks.lock_note(&id, LOCK_PASSWORD.into()).unwrap();
        prop_assert_eq!(phone.library.note(&id).unwrap().body, Some(body));
        let rev = phone.library.note(&id).unwrap().rev;
        phone.library.save(&id, &rev, edit.clone()).unwrap();
        locks.lock().unwrap();
        prop_assert_eq!(locks.unlock(LOCK_PASSWORD.into()), Ok(true));
        prop_assert_eq!(phone.library.note(&id).unwrap().body, Some(edit));
        // The one stored copy is sealed.
        let store = Store::open(phone.path()).unwrap();
        prop_assert!(store.note(&id).unwrap().unwrap().is_locked());
    }
}
