//! The lock session: unlocking, locking, locking a note, the events and the
//! auto-lock, and what the store is left holding.

mod common;

use std::sync::mpsc;
use std::time::{Duration, Instant};

use base64::Engine as _;
use common::*;
use kanso_core::DocType;
use kanso_core::model::{FILE_LOCK_FIELD, KANSO_FILES_FIELD, note_files_from_extra};
use kanso_core::tasks::linked_file_ids;
use kanso_engine::lock::AUTO_LOCK;
use kanso_mobile::{KansoError, ListPlace, ListQuery, LockEvent, LockReason, LockState, NewNote};
use kanso_store::{Origin, Put};
use kanso_store::{SearchParams, Store};
use kanso_sync::KdfParams;
use kanso_sync::lock::{Envelope, LockKey, LockPassword};
use proptest::prelude::*;
use serde_json::json;

const SECRET: &str = "the secret plan";

/// Two notes sealed in one notebook pass share one sealed attachment: the
/// file is wrapped under both notes' keys, and a password change re-wraps
/// both without rewriting the blob.
#[test]
fn two_notes_locked_together_share_one_sealed_attachment() {
    let phone = lone_phone();
    let store = phone.library.store();
    let plain = b"deterministic attachment bytes";
    let encoded = base64::engine::general_purpose::STANDARD.encode(plain);
    let file = store
        .put(
            DocType::File,
            serde_json::json!({"name":"shot.png","contentType":"image/png",
            "_attachments":{"index":{"content_type":"image/png","data":encoded}}}),
            Put::Upsert,
            Origin::Api,
        )
        .unwrap()
        .id;
    let body = format!("![shot](inkdrop://{file})");
    let book = phone
        .library
        .create_notebook("Vault".into(), None)
        .unwrap()
        .id;
    let first = phone
        .library
        .create(NewNote {
            notebook_id: Some(book.clone()),
            title: "First".into(),
            body: body.clone(),
            ..Default::default()
        })
        .unwrap()
        .id;
    let second = phone
        .library
        .create(NewNote {
            notebook_id: Some(book.clone()),
            title: "Second".into(),
            body: body.clone(),
            ..Default::default()
        })
        .unwrap()
        .id;
    let locks = phone.library.locks();
    assert_eq!(
        locks
            .lock_notebook(&book, Some(LOCK_PASSWORD.into()))
            .unwrap(),
        2
    );

    // One sealed original, owned and wrapped under both notes' keys.
    for id in [&first, &second] {
        let stored = store.note(id).unwrap().unwrap();
        assert!(stored.is_locked());
        let ownership = note_files_from_extra(&stored.extra).unwrap();
        assert_eq!(ownership.len(), 1);
        assert_eq!(ownership[0].file, file);
    }
    assert_eq!(
        store.attachment_state(&file).unwrap(),
        kanso_store::AttachmentState::Sealed
    );
    let container = store.stored_attachment(&file).unwrap().unwrap().data;
    assert_ne!(container, plain);
    assert_eq!(
        phone.library.attachment(&file).unwrap().bytes.as_slice(),
        plain
    );

    let before_digest = store.file_meta(&file).unwrap().unwrap().md5digest;
    assert_eq!(
        locks
            .change_password(LOCK_PASSWORD.into(), "new lock password".into())
            .unwrap(),
        0
    );
    assert_eq!(
        store.file_meta(&file).unwrap().unwrap().md5digest,
        before_digest
    );
    assert_eq!(
        store.stored_attachment(&file).unwrap().unwrap().data,
        container
    );
    locks.lock().unwrap();
    assert_eq!(phone.library.attachment(&file), Err(KansoError::Locked));
    assert_eq!(locks.unlock("new lock password".into()), Ok(true));
    for id in [&first, &second] {
        assert_eq!(
            phone.library.note(id).unwrap().body.as_deref(),
            Some(body.as_str())
        );
    }
    assert_eq!(
        phone.library.attachment(&file).unwrap().bytes.as_slice(),
        plain
    );
}

/// A notebook whose notes share a file with an unlocked note gets one
/// sealed copy for the whole notebook; the unlocked note keeps the original.
#[test]
fn locking_a_notebook_copies_a_shared_file_once() {
    let phone = lone_phone();
    let store = phone.library.store();
    let plain = b"a picture inside and outside the vault";
    let encoded = base64::engine::general_purpose::STANDARD.encode(plain);
    let file = store
        .put(
            DocType::File,
            serde_json::json!({"name":"shared.png","contentType":"image/png",
            "_attachments":{"index":{"content_type":"image/png","data":encoded}}}),
            Put::Upsert,
            Origin::Api,
        )
        .unwrap()
        .id;
    let body = format!("![](inkdrop://{file})");
    let outside = note(&phone.library, "Outside", &body);
    let book = phone
        .library
        .create_notebook("Vault".into(), None)
        .unwrap()
        .id;
    let mut inside = Vec::new();
    for title in ["First", "Second"] {
        inside.push(
            phone
                .library
                .create(NewNote {
                    notebook_id: Some(book.clone()),
                    title: title.into(),
                    body: body.clone(),
                    ..Default::default()
                })
                .unwrap()
                .id,
        );
    }
    let locks = phone.library.locks();
    assert_eq!(
        locks
            .lock_notebook(&book, Some(LOCK_PASSWORD.into()))
            .unwrap(),
        2
    );

    // The unlocked note keeps the readable original...
    assert_eq!(
        store.attachment_state(&file).unwrap(),
        kanso_store::AttachmentState::Plain
    );
    assert_eq!(store.file_owners(&file).unwrap(), vec![outside.clone()]);
    assert_eq!(
        phone.library.attachment(&file).unwrap().bytes.as_slice(),
        plain
    );
    // ...and both locked notes share one sealed copy.
    let mut copy = None;
    for id in &inside {
        let view = phone.library.note(id).unwrap();
        let links = linked_file_ids(view.body.as_deref().unwrap());
        assert_eq!(links.len(), 1);
        assert_ne!(links[0], file);
        assert_eq!(*copy.get_or_insert(links[0].clone()), links[0]);
        let stored = store.note(id).unwrap().unwrap();
        assert_eq!(
            note_files_from_extra(&stored.extra).unwrap()[0].file,
            links[0]
        );
    }
    let copy = copy.unwrap();
    assert_eq!(
        store.attachment_state(&copy).unwrap(),
        kanso_store::AttachmentState::Sealed
    );
    let mut owners = store.file_owners(&copy).unwrap();
    owners.sort();
    let mut expected = inside.clone();
    expected.sort();
    assert_eq!(owners, expected);

    locks.lock().unwrap();
    assert_eq!(
        phone.library.attachment(&file).unwrap().bytes.as_slice(),
        plain
    );
    assert_eq!(phone.library.attachment(&copy), Err(KansoError::Locked));
}

/// Locking two notes one at a time gives each its own sealed file, so the
/// note left plain keeps a readable original. Removing one lock decrypts
/// only that note's copy; the other note's seal and ownership stay put.
#[test]
fn notes_locked_one_at_a_time_get_their_own_sealed_copy() {
    let phone = lone_phone();
    let store = phone.library.store();
    let plain = b"deterministic attachment bytes";
    let encoded = base64::engine::general_purpose::STANDARD.encode(plain);
    let file = store
        .put(
            DocType::File,
            serde_json::json!({"name":"shot.png","contentType":"image/png",
            "_attachments":{"index":{"content_type":"image/png","data":encoded}}}),
            Put::Upsert,
            Origin::Api,
        )
        .unwrap()
        .id;
    let body = format!("![shot](inkdrop://{file})");
    let first = note(&phone.library, "First", &body);
    let second = note(&phone.library, "Second", &body);
    let locks = phone.library.locks();

    locks.lock_note(&first, LOCK_PASSWORD.into()).unwrap();
    let first_view = phone.library.note(&first).unwrap();
    let first_file = linked_file_ids(first_view.body.as_deref().unwrap())[0].clone();
    assert_ne!(first_file, file);
    assert_eq!(
        store.attachment_state(&first_file).unwrap(),
        kanso_store::AttachmentState::Sealed
    );
    assert_eq!(
        store.attachment_state(&file).unwrap(),
        kanso_store::AttachmentState::Plain
    );
    assert_eq!(store.file_owners(&file).unwrap(), vec![second.clone()]);

    locks.lock_note_with_session(&second).unwrap();
    let second_view = phone.library.note(&second).unwrap();
    assert_eq!(
        linked_file_ids(second_view.body.as_deref().unwrap()),
        vec![file.clone()]
    );
    assert_eq!(
        store.attachment_state(&file).unwrap(),
        kanso_store::AttachmentState::Sealed
    );

    let before = [&first_file, &file].map(|id| {
        (
            store.file_meta(id).unwrap().unwrap().md5digest,
            store.stored_attachment(id).unwrap().unwrap().data,
        )
    });
    assert_eq!(
        locks
            .change_password(LOCK_PASSWORD.into(), "new lock password".into())
            .unwrap(),
        0
    );
    for (id, (digest, container)) in [&first_file, &file].into_iter().zip(before) {
        assert_eq!(store.file_meta(id).unwrap().unwrap().md5digest, digest);
        assert_eq!(
            store.stored_attachment(id).unwrap().unwrap().data,
            container
        );
    }
    locks.lock().unwrap();
    assert_eq!(
        phone.library.attachment(&first_file),
        Err(KansoError::Locked)
    );
    assert_eq!(phone.library.attachment(&file), Err(KansoError::Locked));
    assert_eq!(locks.unlock("new lock password".into()), Ok(true));
    assert_eq!(
        phone
            .library
            .attachment(&first_file)
            .unwrap()
            .bytes
            .as_slice(),
        plain
    );
    assert_eq!(
        phone.library.attachment(&file).unwrap().bytes.as_slice(),
        plain
    );

    locks.remove_note_lock(&first).unwrap();
    let first_view = phone.library.note(&first).unwrap();
    assert_eq!(
        linked_file_ids(first_view.body.as_deref().unwrap()),
        vec![first_file.clone()]
    );
    assert_eq!(
        store.attachment_state(&first_file).unwrap(),
        kanso_store::AttachmentState::Plain
    );
    assert_eq!(store.file_owners(&first_file).unwrap(), vec![first.clone()]);
    assert_eq!(
        phone
            .library
            .attachment(&first_file)
            .unwrap()
            .bytes
            .as_slice(),
        plain
    );
    // The second note's sealed original is untouched.
    assert_eq!(
        store.attachment_state(&file).unwrap(),
        kanso_store::AttachmentState::Sealed
    );
    assert_eq!(store.file_owners(&file).unwrap(), vec![second.clone()]);
    let second_view = phone.library.note(&second).unwrap();
    assert_eq!(
        linked_file_ids(second_view.body.as_deref().unwrap()),
        vec![file.clone()]
    );
    locks.lock().unwrap();
    assert_eq!(
        phone
            .library
            .attachment(&first_file)
            .unwrap()
            .bytes
            .as_slice(),
        plain
    );
    assert_eq!(phone.library.attachment(&file), Err(KansoError::Locked));
}

#[test]
fn removing_the_last_note_lock_restores_plain_attachment_without_session_keys() {
    let phone = lone_phone();
    let store = phone.library.store();
    let plain = b"restored attachment bytes";
    let encoded = base64::engine::general_purpose::STANDARD.encode(plain);
    let file = store
        .put(
            DocType::File,
            serde_json::json!({"name":"restored.png","contentType":"image/png",
            "_attachments":{"index":{"content_type":"image/png","data":encoded}}}),
            Put::Upsert,
            Origin::Api,
        )
        .unwrap()
        .id;
    let note_id = note(
        &phone.library,
        "Attachment",
        &format!("![](inkdrop://{file})"),
    );
    let locks = phone.library.locks();
    locks.lock_note(&note_id, LOCK_PASSWORD.into()).unwrap();
    locks.lock().unwrap();
    assert_eq!(phone.library.attachment(&file), Err(KansoError::Locked));
    assert_eq!(locks.unlock(LOCK_PASSWORD.into()), Ok(true));
    locks.remove_note_lock(&note_id).unwrap();
    locks.lock().unwrap();

    assert_eq!(
        store.attachment_state(&file).unwrap(),
        kanso_store::AttachmentState::Plain
    );
    assert_eq!(
        phone.library.attachment(&file).unwrap().bytes.as_slice(),
        plain
    );
}

/// A note locked by an older Kanso has sealed text and a plain attachment,
/// with no `kansoFiles` list. Opening it with the password seals the file and
/// writes the list once; opening it again does not write the note again.
#[test]
fn a_legacy_locked_note_seals_its_attachment_when_opened() {
    let phone = lone_phone();
    let store = phone.library.store();
    let plain = b"legacy attachment bytes";
    let encoded = base64::engine::general_purpose::STANDARD.encode(plain);
    let file = store
        .put(
            DocType::File,
            serde_json::json!({"name":"legacy.png","contentType":"image/png",
            "_attachments":{"index":{"content_type":"image/png","data":encoded}}}),
            Put::Upsert,
            Origin::Api,
        )
        .unwrap()
        .id;
    let body = format!("![](inkdrop://{file})");
    let id = note(&phone.library, "Legacy", &body);

    // The older Kanso sealed only the text.
    let mut stored = store.note(&id).unwrap().unwrap();
    let key = LockKey::create(&LockPassword::new(LOCK_PASSWORD).unwrap(), KdfParams::TEST).unwrap();
    let envelope = key.seal(&id, &stored.body);
    Envelope::store(Some(&envelope), &mut stored.extra);
    stored.body.clear();
    assert!(!stored.extra.contains_key(KANSO_FILES_FIELD));
    store
        .lock_note(serde_json::to_value(stored).unwrap())
        .unwrap();
    assert_eq!(
        store.attachment_state(&file).unwrap(),
        kanso_store::AttachmentState::Plain
    );

    let locks = phone.library.locks();
    assert_eq!(locks.unlock(LOCK_PASSWORD.into()), Ok(true));
    let view = phone.library.note(&id).unwrap();
    assert_eq!(view.body.as_deref(), Some(body.as_str()));
    let stored = store.note(&id).unwrap().unwrap();
    let ownership = note_files_from_extra(&stored.extra).unwrap();
    assert_eq!(ownership.len(), 1);
    assert_eq!(ownership[0].file, file);
    assert_eq!(
        store.attachment_state(&file).unwrap(),
        kanso_store::AttachmentState::Sealed
    );
    assert_eq!(
        phone.library.attachment(&file).unwrap().bytes.as_slice(),
        plain
    );

    // A second open sees the list and leaves the revision alone.
    let rev = stored.rev;
    assert_eq!(
        phone.library.note(&id).unwrap().body.as_deref(),
        Some(body.as_str())
    );
    let stored = store.note(&id).unwrap().unwrap();
    assert_eq!(stored.rev, rev);
    assert_eq!(note_files_from_extra(&stored.extra).unwrap().len(), 1);

    locks.lock().unwrap();
    assert_eq!(phone.library.attachment(&file), Err(KansoError::Locked));
    assert_eq!(locks.unlock(LOCK_PASSWORD.into()), Ok(true));
    assert_eq!(
        phone.library.attachment(&file).unwrap().bytes.as_slice(),
        plain
    );
}

#[test]
fn locking_a_notebook_seals_and_reveals_its_attachment() {
    let phone = lone_phone();
    let store = phone.library.store();
    let plain = b"notebook attachment";
    let encoded = base64::engine::general_purpose::STANDARD.encode(plain);
    let file = store
        .put(
            DocType::File,
            serde_json::json!({"name":"book.png","contentType":"image/png",
            "_attachments":{"index":{"content_type":"image/png","data":encoded}}}),
            Put::Upsert,
            Origin::Api,
        )
        .unwrap()
        .id;
    let book = phone
        .library
        .create_notebook("Vault".into(), None)
        .unwrap()
        .id;
    let note_id = phone
        .library
        .create(NewNote {
            notebook_id: Some(book.clone()),
            title: "Image".into(),
            body: format!("![](inkdrop://{file})"),
            ..Default::default()
        })
        .unwrap()
        .id;
    let locks = phone.library.locks();
    assert_eq!(
        locks
            .lock_notebook(&book, Some(LOCK_PASSWORD.into()))
            .unwrap(),
        1
    );
    let stored = store.note(&note_id).unwrap().unwrap();
    assert!(stored.is_locked());
    assert!(stored.extra.contains_key(KANSO_FILES_FIELD));
    assert!(
        store
            .file_meta(&file)
            .unwrap()
            .unwrap()
            .extra
            .contains_key(FILE_LOCK_FIELD)
    );
    let container = store.stored_attachment(&file).unwrap().unwrap().data;
    locks.lock().unwrap();
    assert!(matches!(
        phone.library.attachment(&file),
        Err(KansoError::Locked)
    ));
    assert_eq!(
        store.stored_attachment(&file).unwrap().unwrap().data,
        container
    );
    assert_eq!(locks.unlock(LOCK_PASSWORD.into()), Ok(true));
    assert_eq!(
        phone.library.attachment(&file).unwrap().bytes.as_slice(),
        plain
    );
}

// no-prop-test: a fixed sequence of the calls and what each answers.
#[test]
fn locking_one_note_copies_a_file_an_unlocked_note_still_shows() {
    let phone = lone_phone();
    let store = phone.library.store();
    let plain = b"a picture two notes show";
    let encoded = base64::engine::general_purpose::STANDARD.encode(plain);
    let file = store
        .put(
            DocType::File,
            serde_json::json!({"name":"shared.png","contentType":"image/png",
            "_attachments":{"index":{"content_type":"image/png","data":encoded}}}),
            Put::Upsert,
            Origin::Api,
        )
        .unwrap()
        .id;
    let link = format!("![](inkdrop://{file})");
    let locked_note = note(&phone.library, "Locked", &link);
    let plain_note = note(&phone.library, "Plain", &link);
    let locks = phone.library.locks();

    locks.lock_note(&locked_note, LOCK_PASSWORD.into()).unwrap();

    // The unlocked note keeps the readable original...
    assert_eq!(
        store.attachment_state(&file).unwrap(),
        kanso_store::AttachmentState::Plain
    );
    assert_eq!(store.file_owners(&file).unwrap(), vec![plain_note.clone()]);
    assert_eq!(
        phone.library.attachment(&file).unwrap().bytes.as_slice(),
        plain
    );
    // ...and the locked note links its own sealed copy.
    let view = phone.library.note(&locked_note).unwrap();
    let links = linked_file_ids(view.body.as_deref().unwrap());
    assert_eq!(links.len(), 1);
    assert_ne!(links[0], file);
    assert_eq!(
        store.attachment_state(&links[0]).unwrap(),
        kanso_store::AttachmentState::Sealed
    );
    assert_eq!(
        store.file_owners(&links[0]).unwrap(),
        vec![locked_note.clone()]
    );

    locks.lock().unwrap();
    // The plain note still reads its file without any lock keys.
    assert_eq!(
        phone.library.attachment(&file).unwrap().bytes.as_slice(),
        plain
    );
    assert_eq!(phone.library.attachment(&links[0]), Err(KansoError::Locked));
}

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

    /// Sealing a legacy note's attachment list is idempotent: one open
    /// seals it, a second leaves its revision and list alone.
    #[test]
    fn prop_legacy_attachment_migration_is_idempotent(
        prefix in "\\PC{0,40}",
        suffix in "\\PC{0,40}",
    ) {
        let phone = lone_phone();
        let store = phone.library.store();
        let encoded = base64::engine::general_purpose::STANDARD.encode(b"legacy");
        let file = store
            .put(
                DocType::File,
                serde_json::json!({"name":"legacy.png","contentType":"image/png",
                "_attachments":{"index":{"content_type":"image/png","data":encoded}}}),
                Put::Upsert,
                Origin::Api,
            )
            .unwrap()
            .id;
        let body = format!("{prefix}![](inkdrop://{file}){suffix}");
        let id = note(&phone.library, "Legacy", &body);
        let mut stored = store.note(&id).unwrap().unwrap();
        let key = LockKey::create(
            &LockPassword::new(LOCK_PASSWORD).unwrap(),
            KdfParams::TEST,
        )
        .unwrap();
        let envelope = key.seal(&id, &stored.body);
        Envelope::store(Some(&envelope), &mut stored.extra);
        stored.body.clear();
        store
            .lock_note(serde_json::to_value(stored).unwrap())
            .unwrap();

        let locks = phone.library.locks();
        locks.unlock(LOCK_PASSWORD.into()).unwrap();
        let first = phone.library.note(&id).unwrap();
        prop_assert_eq!(first.body.as_deref(), Some(body.as_str()));
        let sealed = store.note(&id).unwrap().unwrap();
        let rev = sealed.rev.clone();
        let ownership = note_files_from_extra(&sealed.extra).unwrap();
        prop_assert_eq!(ownership.len(), 1);
        prop_assert_eq!(&ownership[0].file, &file);
        prop_assert_eq!(
            store.attachment_state(&file).unwrap(),
            kanso_store::AttachmentState::Sealed
        );

        let second = phone.library.note(&id).unwrap();
        prop_assert_eq!(second.body.as_deref(), Some(body.as_str()));
        let again = store.note(&id).unwrap().unwrap();
        prop_assert_eq!(again.rev, rev);
        prop_assert_eq!(note_files_from_extra(&again.extra).unwrap().len(), 1);
    }

    /// A file that unlocked notes still show is copied for the note being
    /// locked: the original stays readable and keeps those owners.
    #[test]
    fn prop_locking_copies_a_file_unlocked_notes_still_show(
        bytes in proptest::collection::vec(any::<u8>(), 0..=2048)
    ) {
        let phone = lone_phone();
        let store = phone.library.store();
        let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
        let file = store
            .put(
                DocType::File,
                serde_json::json!({"name":"shared.png","contentType":"image/png",
                "_attachments":{"index":{"content_type":"image/png","data":encoded}}}),
                Put::Upsert,
                Origin::Api,
            )
            .unwrap()
            .id;
        let body = format!("![](inkdrop://{file})");
        let locked = note(&phone.library, "Locked", &body);
        let first_plain = note(&phone.library, "One", &body);
        let second_plain = note(&phone.library, "Two", &body);
        let locks = phone.library.locks();

        locks.lock_note(&locked, LOCK_PASSWORD.into()).unwrap();

        prop_assert_eq!(
            store.attachment_state(&file).unwrap(),
            kanso_store::AttachmentState::Plain
        );
        let mut owners = store.file_owners(&file).unwrap();
        let mut expected = vec![first_plain.clone(), second_plain.clone()];
        owners.sort();
        expected.sort();
        prop_assert_eq!(owners, expected);
        let original = phone.library.attachment(&file).unwrap();
        prop_assert_eq!(original.bytes.as_slice(), bytes.as_slice());
        let view = phone.library.note(&locked).unwrap();
        let links = linked_file_ids(view.body.as_deref().unwrap());
        prop_assert_eq!(links.len(), 1);
        prop_assert_ne!(&links[0], &file);
        prop_assert_eq!(
            store.attachment_state(&links[0]).unwrap(),
            kanso_store::AttachmentState::Sealed
        );
        let sealed = phone.library.attachment(&links[0]).unwrap();
        prop_assert_eq!(sealed.bytes.as_slice(), bytes.as_slice());
    }

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
