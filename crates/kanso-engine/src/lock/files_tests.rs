//! The lock flow for attachments, against a real store.

use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;
use std::time::Instant;

use base64::engine::general_purpose::STANDARD;
use kanso_core::model::note_files_from_extra;
use kanso_core::{DocType, Note};
use kanso_store::{AttachmentState, Origin, Put, Store};
use kanso_sync::KdfParams;
use kanso_sync::lock::{Envelope, LockKey, LockPassword};
use proptest::prelude::*;
use serde_json::json;

use super::*;
use crate::lock::rekey;

static KEY: LazyLock<LockKey> = LazyLock::new(|| {
    LockKey::create(
        &LockPassword::new("first password").unwrap(),
        KdfParams::TEST,
    )
    .unwrap()
});
static OTHER: LazyLock<LockKey> = LazyLock::new(|| {
    LockKey::create(
        &LockPassword::new("second password").unwrap(),
        KdfParams::TEST,
    )
    .unwrap()
});

fn key() -> LockKey {
    LockKey::from_secret(&KEY.to_secret()).unwrap()
}

fn other() -> LockKey {
    LockKey::from_secret(&OTHER.to_secret()).unwrap()
}

struct World {
    _dir: tempfile::TempDir,
    store: Store,
    book: String,
}

fn world() -> World {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("k.db")).unwrap();
    let book = store
        .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
        .unwrap()
        .id;
    World {
        _dir: dir,
        store,
        book,
    }
}

impl World {
    fn file(&self, name: &str, bytes: &[u8]) -> String {
        self.store
            .put(
                DocType::File,
                json!({
                    "name": name,
                    "contentType": "image/png",
                    "_attachments": {"index": {
                        "content_type": "image/png",
                        "data": STANDARD.encode(bytes),
                    }},
                }),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id
    }

    fn note(&self, files: &[String]) -> Note {
        let body: String = files
            .iter()
            .map(|id| format!("![](inkdrop://{id})\n"))
            .collect();
        let id = self
            .store
            .put(
                DocType::Note,
                json!({"bookId": self.book, "title": "T", "body": body}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        self.store.note(&id).unwrap().unwrap()
    }

    /// Locks `note` with `key` the way the app does and returns the file
    /// keys it made.
    fn lock(&self, note: &mut Note, key: &LockKey, keys: &KeyBook) -> Counts {
        let locking = HashSet::from([note.id.clone()]);
        let mut copies = HashMap::new();
        let mut watch = |_: &Progress| Ok(());
        let mut job = Job::new(&self.store, keys, &locking, &mut copies, &mut watch);
        let text = note.body.clone();
        let (body, counts) = seal_note_files(&mut job, note, &text, key).unwrap();
        note.body = body.clone();
        let mut stored = note.clone();
        stored.body.clear();
        let written = self
            .store
            .lock_note(serde_json::to_value(&stored).unwrap())
            .unwrap();
        note.rev = Some(written.rev);
        counts
    }

    fn stored(&self, id: &str) -> Vec<u8> {
        self.store.stored_attachment(id).unwrap().unwrap().data
    }
}

fn bytes_of(reading: Reading) -> Option<Vec<u8>> {
    match reading {
        Reading::Opened(file) => Some(file.bytes),
        _ => None,
    }
}

fn contents() -> impl Strategy<Value = Vec<Vec<u8>>> {
    prop::collection::vec(prop::collection::vec(any::<u8>(), 1..2_000), 0..4)
}

proptest! {
    /// Round trip: whatever is attached, locking seals it, the note's keys
    /// read it back, taking the lock off gives the same bytes in plain
    /// files, and nothing readable was left in the database meanwhile.
    #[test]
    fn prop_lock_then_unlock_then_remove_lock_returns_the_same_bytes(files in contents()) {
        let world = world();
        let ids: Vec<String> = files
            .iter()
            .enumerate()
            .map(|(i, bytes)| world.file(&format!("f{i}.png"), bytes))
            .collect();
        let mut note = world.note(&ids);
        let keys = KeyBook::default();
        let key = key();
        world.lock(&mut note, &key, &keys);

        for (id, bytes) in ids.iter().zip(&files) {
            prop_assert_eq!(world.store.attachment_state(id).unwrap(), AttachmentState::Sealed);
            prop_assert_ne!(&world.stored(id), bytes);
            prop_assert_eq!(bytes_of(read_file(&world.store, id, &keys)), Some(bytes.clone()));
        }
        // Locked again: nothing opens.
        keys.clear();
        for id in &ids {
            prop_assert_eq!(read_file(&world.store, id, &keys), Reading::Locked);
        }
        // Unlocked again: the note's list gives the keys back.
        let stored = world.store.note(&note.id).unwrap().unwrap();
        let mut revealed = stored.clone();
        let envelope = Envelope::from_extra(&stored.extra).unwrap().unwrap();
        revealed.body = key.open(&stored.id, &envelope).unwrap().to_string();
        keys.remember_note(&stored, &key);
        for (id, bytes) in ids.iter().zip(&files) {
            prop_assert_eq!(bytes_of(read_file(&world.store, id, &keys)), Some(bytes.clone()));
        }
        // Remove Lock.
        let (body, done) = unseal_or_copy(&world.store, &mut revealed, &key, &mut |_| Ok(())).unwrap();
        prop_assert_eq!(done.decrypted, ids.len());
        prop_assert_eq!(done.copied, 0);
        revealed.body = body;
        Envelope::store(None, &mut revealed.extra);
        world
            .store
            .unlock_note(serde_json::to_value(&revealed).unwrap())
            .unwrap();
        for (id, bytes) in ids.iter().zip(&files) {
            prop_assert_eq!(world.store.attachment_state(id).unwrap(), AttachmentState::Plain);
            prop_assert_eq!(&world.stored(id), bytes);
        }
    }

    /// Changing the password moves wraps and nothing else: no blob changes,
    /// and every file opens with the new password and not the old.
    #[test]
    fn prop_rekey_changes_no_blob_and_every_file_opens_with_the_new_password(files in contents()) {
        let world = world();
        let ids: Vec<String> = files
            .iter()
            .enumerate()
            .map(|(i, bytes)| world.file(&format!("f{i}.png"), bytes))
            .collect();
        let mut note = world.note(&ids);
        // Sealed with the key the password gives, as a real lock is.
        let old_password = LockPassword::new("first password").unwrap();
        let old = LockKey::create(&old_password, KdfParams::TEST).unwrap();
        world.lock(&mut note, &old, &KeyBook::default());
        let before: Vec<Vec<u8>> = ids.iter().map(|id| world.stored(id)).collect();

        let new_password = LockPassword::new("a new password").unwrap();
        let (new, skipped) = rekey(&world.store, &old_password, &new_password, KdfParams::TEST).unwrap();
        prop_assert_eq!(skipped, 0);

        let after: Vec<Vec<u8>> = ids.iter().map(|id| world.stored(id)).collect();
        prop_assert_eq!(before, after);
        let stored = world.store.note(&note.id).unwrap().unwrap();
        let fresh = KeyBook::default();
        prop_assert_eq!(fresh.remember_note(&stored, &new), ids.len());
        for (id, bytes) in ids.iter().zip(&files) {
            prop_assert_eq!(bytes_of(read_file(&world.store, id, &fresh)), Some(bytes.clone()));
        }
        let stale = KeyBook::default();
        prop_assert_eq!(stale.remember_note(&stored, &old), 0);
    }

    /// A picture an unlocked note also shows: locking the other note gives
    /// it a sealed copy, and both notes read their picture.
    #[test]
    fn prop_a_file_shared_by_a_locked_and_an_unlocked_note_stays_readable_by_both(
        bytes in prop::collection::vec(any::<u8>(), 1..2_000),
    ) {
        let world = world();
        let file = world.file("shared.png", &bytes);
        let mut locked = world.note(std::slice::from_ref(&file));
        let unlocked = world.note(std::slice::from_ref(&file));
        let keys = KeyBook::default();
        let counts = world.lock(&mut locked, &key(), &keys);
        prop_assert_eq!(counts.copied, 1);

        let stored = world.store.note(&locked.id).unwrap().unwrap();
        let listed = note_files_from_extra(&stored.extra).unwrap();
        prop_assert_eq!(listed.len(), 1);
        let copy = &listed[0].file;
        prop_assert_ne!(copy, &file);
        // The locked note's text links the copy.
        prop_assert!(locked.body.contains(copy.as_str()));
        let original_link = ["inkdrop://", file.as_str()].concat();
        prop_assert!(!locked.body.contains(&original_link));
        // The unlocked note keeps the readable original.
        prop_assert_eq!(world.store.attachment_state(&file).unwrap(), AttachmentState::Plain);
        prop_assert_eq!(
            bytes_of(read_file(&world.store, &file, &KeyBook::default())),
            Some(bytes.clone())
        );
        prop_assert_eq!(world.store.note(&unlocked.id).unwrap().unwrap().body, unlocked.body);
        prop_assert_eq!(bytes_of(read_file(&world.store, copy, &keys)), Some(bytes.clone()));
        prop_assert_eq!(read_file(&world.store, copy, &KeyBook::default()), Reading::Locked);
    }
}

#[test]
fn a_lock_resumes_after_a_crash_between_files() {
    let world = world();
    let one = world.file("one.png", b"first picture bytes");
    let two = world.file("two.png", b"second picture bytes");
    let mut note = world.note(&[one.clone(), two.clone()]);
    let key = key();

    // The lock stops after the first file is sealed.
    let keys = KeyBook::default();
    {
        let locking = HashSet::from([note.id.clone()]);
        let mut copies = HashMap::new();
        let mut watch = |progress: &Progress| {
            if progress.done == 1 {
                Err("stopped".to_owned())
            } else {
                Ok(())
            }
        };
        let mut job = Job::new(&world.store, &keys, &locking, &mut copies, &mut watch);
        let text = note.body.clone();
        let err = seal_note_files(&mut job, &mut note, &text, &key).unwrap_err();
        assert_eq!(err, FilesError::Stopped("stopped".into()));
    }
    let stored = world.store.note(&note.id).unwrap().unwrap();
    assert!(!stored.is_locked(), "the note is still plain");
    assert!(stored.body.contains(&one), "and still readable");
    assert_eq!(
        world.store.attachment_state(&one).unwrap(),
        AttachmentState::Sealed
    );
    assert_eq!(
        world.store.attachment_state(&two).unwrap(),
        AttachmentState::Plain
    );
    assert_eq!(
        interrupted(&world.store, &stored),
        Some(Interrupted {
            files: 2,
            sealed: 1
        })
    );

    // The app quits: the keys are gone. Finishing needs only the password.
    drop(keys);
    let keys = KeyBook::default();
    let mut note = stored;
    let counts = world.lock(&mut note, &key, &keys);
    assert_eq!(counts.sealed, 1, "only the second file is sealed now");
    assert_eq!(
        world.store.attachment_state(&two).unwrap(),
        AttachmentState::Sealed
    );
    // The first file was not sealed again.
    let locked = world.store.note(&note.id).unwrap().unwrap();
    assert!(locked.is_locked());
    assert_eq!(interrupted(&world.store, &locked), None);
    assert_eq!(
        bytes_of(read_file(&world.store, &one, &keys)),
        Some(b"first picture bytes".to_vec())
    );
    assert_eq!(
        bytes_of(read_file(&world.store, &two, &keys)),
        Some(b"second picture bytes".to_vec())
    );
}

#[test]
fn a_stopped_lock_can_be_undone() {
    let world = world();
    let one = world.file("one.png", b"first picture bytes");
    let two = world.file("two.png", b"second picture bytes");
    let shared = world.file("shared.png", b"shared bytes");
    let _unlocked = world.note(std::slice::from_ref(&shared));
    let mut note = world.note(&[one.clone(), two.clone(), shared.clone()]);
    let key = key();
    let keys = KeyBook::default();
    {
        let locking = HashSet::from([note.id.clone()]);
        let mut copies = HashMap::new();
        let mut watch = |progress: &Progress| {
            if progress.done == 2 {
                Err("stopped".to_owned())
            } else {
                Ok(())
            }
        };
        let mut job = Job::new(&world.store, &keys, &locking, &mut copies, &mut watch);
        let text = note.body.clone();
        seal_note_files(&mut job, &mut note, &text, &key).unwrap_err();
    }
    let stored = world.store.note(&note.id).unwrap().unwrap();
    assert!(interrupted(&world.store, &stored).is_some());
    // Another password can't undo it.
    assert!(undo_locking(&world.store, &stored, &other()).is_err());
    let undone = undo_locking(&world.store, &stored, &key).unwrap();
    assert_eq!(undone, 3, "two unsealed, one copy removed");
    for (id, bytes) in [
        (&one, &b"first picture bytes"[..]),
        (&two, b"second picture bytes"),
    ] {
        assert_eq!(
            world.store.attachment_state(id).unwrap(),
            AttachmentState::Plain
        );
        assert_eq!(world.stored(id), bytes);
    }
    assert_eq!(
        world.store.attachment_state(&shared).unwrap(),
        AttachmentState::Plain
    );
    let after = world.store.note(&note.id).unwrap().unwrap();
    assert_eq!(interrupted(&world.store, &after), None);
    assert!(
        !after
            .extra
            .contains_key(kanso_core::model::KANSO_FILES_FIELD)
    );
}

#[test]
fn a_note_cannot_lock_with_an_unsealed_listed_file() {
    let world = world();
    let file = world.file("plain.png", b"still plain");
    let mut note = world.note(std::slice::from_ref(&file));
    // A list naming a file that was never sealed.
    let envelope = key().seal(&note.id, &note.body);
    Envelope::store(Some(&envelope), &mut note.extra);
    let entry = kanso_core::model::NoteFile {
        file: file.clone(),
        v: 1,
        seal: SealId::random().encode(),
        key: STANDARD.encode([0u8; 73]),
        extra: Default::default(),
    };
    store_note_files(Some(&[entry]), &mut note.extra);
    note.body.clear();
    let value = serde_json::to_value(&note).unwrap();
    assert!(world.store.lock_note(value.clone()).is_err());
    // A notebook's lock refuses it too.
    let book = json!({"_id": world.book, "name": "B", "kansoLock": serde_json::to_value(key().seal(&world.book, "")).unwrap()});
    assert!(world.store.lock_book(book, vec![value]).is_err());
    assert!(!world.store.note(&note.id).unwrap().unwrap().is_locked());
}

#[test]
fn the_second_note_with_a_different_password_is_refused_or_copies() {
    let world = world();
    let bytes = b"one picture, two locks".to_vec();
    let file = world.file("both.png", &bytes);
    let mut first = world.note(std::slice::from_ref(&file));
    let keys = KeyBook::default();
    world.lock(&mut first, &key(), &keys);
    assert_eq!(
        world.store.attachment_state(&file).unwrap(),
        AttachmentState::Sealed
    );

    // A second note, locked under another password, now shows the sealed file.
    let mut second = world.note(&[]);
    let envelope = other().seal(&second.id, "");
    Envelope::store(Some(&envelope), &mut second.extra);
    second.body.clear();
    world
        .store
        .lock_note(serde_json::to_value(&second).unwrap())
        .unwrap();
    let mut second = world.store.note(&second.id).unwrap().unwrap();
    let text = format!("![](inkdrop://{file})");
    let locking = HashSet::from([second.id.clone()]);

    // Without the first note's file key it is refused, naming that note.
    let none = KeyBook::default();
    let mut copies = HashMap::new();
    let mut watch = |_: &Progress| Ok(());
    let mut job = Job::new(&world.store, &none, &locking, &mut copies, &mut watch);
    let err = seal_note_files(&mut job, &mut second, &text, &other()).unwrap_err();
    assert!(
        matches!(err, FilesError::NeedsUnlock { owner: Some(_), .. }),
        "{err:?}"
    );
    assert!(err.to_string().contains("Unlock that note first"));

    // With it (the first note is open), the second only adds a wrap.
    let mut copies = HashMap::new();
    let mut watch = |_: &Progress| Ok(());
    let mut job = Job::new(&world.store, &keys, &locking, &mut copies, &mut watch);
    let (_, counts) = seal_note_files(&mut job, &mut second, &text, &other()).unwrap();
    assert_eq!(counts.shared, 1);
    second.body.clear();
    world
        .store
        .lock_note(serde_json::to_value(&second).unwrap())
        .unwrap();
    let stored = world.store.note(&second.id).unwrap().unwrap();
    let only_second = KeyBook::default();
    assert_eq!(only_second.remember_note(&stored, &other()), 1);
    assert_eq!(
        bytes_of(read_file(&world.store, &file, &only_second)),
        Some(bytes.clone())
    );

    // Taking the first note's lock off leaves the second its sealed file: the
    // first gets a plain copy.
    let first_stored = world.store.note(&first.id).unwrap().unwrap();
    let mut open = first_stored.clone();
    open.body = first.body.clone();
    let (body, done) = unseal_or_copy(&world.store, &mut open, &key(), &mut |_| Ok(())).unwrap();
    assert_eq!((done.copied, done.decrypted), (1, 0));
    assert!(!body.contains(&file));
    assert_eq!(
        world.store.attachment_state(&file).unwrap(),
        AttachmentState::Sealed
    );
    assert_eq!(
        bytes_of(read_file(&world.store, &file, &only_second)),
        Some(bytes)
    );
}

#[test]
fn a_note_locked_in_a_pass_with_the_notes_that_share_its_file_keeps_one_original() {
    let world = world();
    let file = world.file("both.png", b"shared inside the pass");
    let a = world.note(std::slice::from_ref(&file));
    let b = world.note(std::slice::from_ref(&file));
    let keys = KeyBook::default();
    let sealed =
        crate::lock::seal_notebook_with(&world.store, &world.book, &key(), &keys, &mut |_| Ok(()))
            .unwrap();
    assert_eq!(sealed, 2);
    assert_eq!(
        world.store.attachment_state(&file).unwrap(),
        AttachmentState::Sealed
    );
    for id in [&a.id, &b.id] {
        let stored = world.store.note(id).unwrap().unwrap();
        assert!(stored.is_locked());
        let list = note_files_from_extra(&stored.extra).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].file, file);
    }
}

#[test]
fn locking_one_hundred_images_reports_progress_and_finishes() {
    let world = world();
    let ids: Vec<String> = (0..100u8)
        .map(|i| world.file(&format!("{i}.png"), &vec![i; 20_000]))
        .collect();
    let mut note = world.note(&ids);
    let keys = KeyBook::default();
    let locking = HashSet::from([note.id.clone()]);
    let mut copies = HashMap::new();
    let mut seen: Vec<Progress> = Vec::new();
    let mut watch = |progress: &Progress| {
        seen.push(*progress);
        Ok(())
    };
    let started = Instant::now();
    let mut job = Job::new(&world.store, &keys, &locking, &mut copies, &mut watch);
    let text = note.body.clone();
    seal_note_files(&mut job, &mut note, &text, &key()).unwrap();
    eprintln!("sealed 100 images of 20 kB in {:?}", started.elapsed());

    let last = seen.last().unwrap();
    assert_eq!((last.done, last.total), (100, 100));
    assert!(
        last.bytes_done >= 100 * 20_000 && last.bytes_done == last.bytes_total.max(last.bytes_done)
    );
    assert_eq!(last.phase, Phase::Locking);
    // Progress only moves forward, one file at a time.
    assert!(seen.windows(2).all(|pair| pair[0].done <= pair[1].done));
    assert!(seen.iter().any(|progress| progress.done == 50));
    assert!(
        ids.iter()
            .all(|id| world.store.attachment_state(id).unwrap() == AttachmentState::Sealed)
    );
}

#[test]
fn a_file_added_to_a_locked_note_is_sealed_with_a_list_of_its_own() {
    let world = world();
    let first = world.file("one.png", b"one");
    let mut note = world.note(std::slice::from_ref(&first));
    let keys = KeyBook::default();
    world.lock(&mut note, &key(), &keys);

    // An attachment dropped on the open note.
    let added = world.file("two.png", b"two");
    note.body.push_str(&format!("![](inkdrop://{added})\n"));
    let locking = HashSet::from([note.id.clone()]);
    let mut copies = HashMap::new();
    let mut watch = |_: &Progress| Ok(());
    let mut job = Job::new(&world.store, &keys, &locking, &mut copies, &mut watch);
    let text = note.body.clone();
    let (_, counts) = seal_note_files(&mut job, &mut note, &text, &key()).unwrap();
    assert_eq!(counts.sealed, 1);
    let list = note_files_from_extra(&note.extra).unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(
        world.store.attachment_state(&added).unwrap(),
        AttachmentState::Sealed
    );
    assert_eq!(
        bytes_of(read_file(&world.store, &added, &keys)),
        Some(b"two".to_vec())
    );
}

#[test]
fn links_are_replaced_only_where_the_whole_id_matches() {
    assert_eq!(
        replace_link(
            "a inkdrop://file:abc b inkdrop://file:abcd",
            "file:abc",
            "file:new"
        ),
        "a inkdrop://file:new b inkdrop://file:abcd"
    );
}
