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
        let (body, done, unsealing) = unseal_or_copy(&world.store, &mut revealed, &key, &mut |_| Ok(())).unwrap();
        prop_assert_eq!(done.decrypted, ids.len());
        prop_assert_eq!(done.copied, 0);
        revealed.body = body;
        Envelope::store(None, &mut revealed.extra);
        world
            .store
            .unlock_note_unsealing(serde_json::to_value(&revealed).unwrap(), unsealing)
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
    let (body, done, unsealing) =
        unseal_or_copy(&world.store, &mut open, &key(), &mut |_| Ok(())).unwrap();
    assert_eq!((done.copied, done.decrypted), (1, 0));
    assert_eq!(unsealing.copies.len(), 1);
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

// MARK: Review of #290

/// Starts locking `note` with `key` and stops it once `stop_at` files are
/// sealed, as a crash or a Cancel would.
fn stop_locking(world: &World, note: &mut Note, key: &LockKey, keys: &KeyBook, stop_at: usize) {
    let locking = HashSet::from([note.id.clone()]);
    let mut copies = HashMap::new();
    let mut watch = |progress: &Progress| {
        if progress.phase == Phase::Sealing && progress.done == stop_at {
            Err("stopped".to_owned())
        } else {
            Ok(())
        }
    };
    let mut job = Job::new(&world.store, keys, &locking, &mut copies, &mut watch);
    let text = note.body.clone();
    let err = seal_note_files(&mut job, note, &text, key).unwrap_err();
    assert_eq!(err, FilesError::Stopped("stopped".into()));
}

fn password(words: &str) -> LockPassword {
    LockPassword::new(words).unwrap()
}

/// A password change while a lock is stopped: the stopped lock's keys move
/// to the new password, so Finish locking works with it alone. Before, they
/// stayed under the old one, and with it forgotten the sealed files never
/// opened again.
#[test]
fn a_password_change_moves_a_stopped_locks_keys_to_the_new_password() {
    let world = world();
    let one = world.file("one.png", b"first picture bytes");
    let two = world.file("two.png", b"second picture bytes");
    let mut note = world.note(&[one.clone(), two.clone()]);
    // The first lock ever: no other locked note gives the old key away.
    stop_locking(&world, &mut note, &key(), &KeyBook::default(), 1);
    assert_eq!(
        world.store.attachment_state(&one).unwrap(),
        AttachmentState::Sealed
    );

    let (new_key, skipped) = rekey(
        &world.store,
        &password("first password"),
        &password("a new password"),
        KdfParams::TEST,
    )
    .unwrap();
    assert_eq!(skipped, 0);
    let stopped = world.store.note(&note.id).unwrap().unwrap();
    assert!(interrupted(&world.store, &stopped).is_some());
    assert!(
        !opens_interrupted(&stopped, &key()),
        "the old password is done"
    );
    // What the app does: the new password, with the stopped lock's header.
    let header = interrupted_header(&stopped).unwrap();
    let typed = LockKey::for_envelope(&password("a new password"), &header).unwrap();
    assert!(new_key.fits(&header));
    assert!(opens_interrupted(&stopped, &typed));

    let mut note = stopped;
    world.lock(&mut note, &typed, &KeyBook::default());
    let keys = KeyBook::default();
    let locked = world.store.note(&note.id).unwrap().unwrap();
    assert!(locked.is_locked());
    keys.remember_note(&locked, &typed);
    for (id, bytes) in [
        (&one, &b"first picture bytes"[..]),
        (&two, b"second picture bytes"),
    ] {
        assert_eq!(
            bytes_of(read_file(&world.store, id, &keys)).as_deref(),
            Some(bytes)
        );
    }
}

/// Undoing a stopped lock leaves sealed a file that a locked note took
/// since (its link pasted into the open locked note): before, it was
/// decrypted under that note.
#[test]
fn undoing_a_stopped_lock_keeps_a_file_another_locked_note_holds() {
    let world = world();
    let shared = world.file("shared.png", b"shared bytes");
    let other_file = world.file("other.png", b"other bytes");
    let own = world.file("own.png", b"own bytes");
    let keys = KeyBook::default();
    let mut locked = world.note(std::slice::from_ref(&own));
    world.lock(&mut locked, &key(), &keys);
    let mut stopped = world.note(&[shared.clone(), other_file.clone()]);
    stop_locking(&world, &mut stopped, &key(), &keys, 1);
    assert_eq!(
        world.store.attachment_state(&shared).unwrap(),
        AttachmentState::Sealed
    );
    // The open locked note takes the sealed file, with the key held.
    let body = format!("{}![](inkdrop://{shared})\n", locked.body);
    save_locked(&world.store, &keys, &mut locked, &body, &key()).unwrap();

    let stopped = world.store.note(&stopped.id).unwrap().unwrap();
    undo_locking(&world.store, &stopped, &key()).unwrap();
    assert_eq!(
        world.store.attachment_state(&shared).unwrap(),
        AttachmentState::Sealed,
        "the locked note's file stays sealed"
    );
    // The plain note shows a plain copy of it instead.
    let after = world.store.note(&stopped.id).unwrap().unwrap();
    assert!(!after.body.contains(&shared));
    let copy = kanso_core::tasks::linked_file_ids(&after.body)
        .into_iter()
        .find(|id| id != &other_file)
        .unwrap();
    assert_eq!(world.stored(&copy), b"shared bytes");
    assert_eq!(world.stored(&other_file), b"other bytes");
    let fresh = KeyBook::default();
    fresh.remember_note(&world.store.note(&locked.id).unwrap().unwrap(), &key());
    assert_eq!(
        bytes_of(read_file(&world.store, &shared, &fresh)),
        Some(b"shared bytes".to_vec())
    );
}

/// A notebook's lock stops on its second note, which found the file both
/// notes show sealed already. The first note is then deleted for good, and
/// the second one's lock undone: its list was the last holding the file's
/// key, so the file is decrypted. Before, it stayed sealed with no key
/// anywhere, under a plain note.
#[test]
fn undoing_the_last_list_that_holds_a_key_decrypts_the_file() {
    let world = world();
    let shared = world.file("shared.png", b"shared bytes");
    let first_own = world.file("a.png", b"a bytes");
    let second_own = world.file("b.png", b"b bytes");
    let first = world.note(&[shared.clone(), first_own.clone()]);
    let second = world.note(&[shared.clone(), second_own.clone()]);
    let mut seen = 0;
    let mut watch = |progress: &Progress| {
        if progress.phase == Phase::Sealing && progress.done == 2 {
            seen += 1;
            if seen == 2 {
                return Err("stopped".to_owned());
            }
        }
        Ok(())
    };
    let err = crate::lock::seal_notebook_with(
        &world.store,
        &world.book,
        &key(),
        &KeyBook::default(),
        &mut watch,
    )
    .unwrap_err();
    assert_eq!(err, "stopped");
    // Which note went first is the notebook's order.
    let (done, stopped) =
        if world.store.attachment_state(&second_own).unwrap() == AttachmentState::Plain {
            (first, second)
        } else {
            (second, first)
        };
    world.store.delete(&done.id, Origin::Ui).unwrap();

    let stopped = world.store.note(&stopped.id).unwrap().unwrap();
    assert!(interrupted(&world.store, &stopped).is_some());
    undo_locking(&world.store, &stopped, &key()).unwrap();
    assert_eq!(
        world.store.attachment_state(&shared).unwrap(),
        AttachmentState::Plain
    );
    assert_eq!(world.stored(&shared), b"shared bytes");
}

fn canvas_body(cards: &[&str]) -> String {
    use kanso_core::canvas::{Canvas, Node, to_body};
    to_body(&Canvas {
        nodes: cards
            .iter()
            .enumerate()
            .map(|(ix, file)| Node {
                id: format!("c{ix}"),
                kind: "file".into(),
                file: Some((*file).to_owned()),
                width: 100.0,
                height: 80.0,
                ..Node::default()
            })
            .collect(),
        ..Canvas::default()
    })
}

impl World {
    fn canvas(&self, cards: &[&str]) -> Note {
        let id = self
            .store
            .put(
                DocType::Note,
                json!({"bookId": self.book, "title": "Board", "doctype": "canvas", "body": canvas_body(cards)}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        self.store.note(&id).unwrap().unwrap()
    }
}

/// A canvas's attachment card shows a file without a link: locking a note
/// that shows it too copies it, so the unlocked canvas keeps reading it.
#[test]
fn a_canvas_card_keeps_its_file_when_a_note_that_shows_it_locks() {
    let world = world();
    let file = world.file("card.png", b"card bytes");
    let _canvas = world.canvas(&[&file]);
    assert_eq!(world.store.file_owners(&file).unwrap().len(), 1);
    let mut note = world.note(std::slice::from_ref(&file));
    let counts = world.lock(&mut note, &key(), &KeyBook::default());
    assert_eq!(counts.copied, 1);
    assert_eq!(
        world.store.attachment_state(&file).unwrap(),
        AttachmentState::Plain
    );
    assert_eq!(world.stored(&file), b"card bytes");
}

/// A locked canvas seals the files its cards show.
#[test]
fn a_locked_canvas_seals_the_files_its_cards_show() {
    let world = world();
    let file = world.file("card.png", b"card bytes");
    let mut canvas = world.canvas(&[&file]);
    let keys = KeyBook::default();
    let counts = world.lock(&mut canvas, &key(), &keys);
    assert_eq!(counts.sealed, 1);
    assert_eq!(
        world.store.attachment_state(&file).unwrap(),
        AttachmentState::Sealed
    );
    assert_eq!(
        bytes_of(read_file(&world.store, &file, &keys)),
        Some(b"card bytes".to_vec())
    );
}

/// A file document a sync brought without its bytes.
fn without_bytes(world: &World) -> String {
    let id = "file:waiting1".to_owned();
    world
        .store
        .apply_remote(
            &kanso_store::SyncDoc {
                id: id.clone(),
                rev: "1-aaaa".into(),
                deleted: false,
                fields: json!({
                    "name": "later.png",
                    "contentType": "image/png",
                    "contentLength": 4,
                    "md5digest": "0123456789abcdef0123456789abcdef",
                    "createdAt": 1,
                    "updatedAt": 1,
                })
                .as_object()
                .unwrap()
                .clone(),
                base_rev: None,
            },
            None,
            None,
        )
        .unwrap();
    assert_eq!(
        world.store.attachment_state(&id).unwrap(),
        AttachmentState::Plain
    );
    id
}

/// A file whose bytes haven't arrived: a lock refuses before it writes
/// anything, and a save of a locked note leaves it for later instead of
/// refusing the save (which held the typed text back).
#[test]
fn a_file_still_downloading_refuses_a_lock_and_never_holds_back_a_save() {
    let world = world();
    let ready = world.file("ready.png", b"ready bytes");
    let waiting = without_bytes(&world);
    let mut note = world.note(&[ready.clone(), waiting.clone()]);
    let before = world.store.note(&note.id).unwrap().unwrap();
    let locking = HashSet::from([note.id.clone()]);
    let mut copies = HashMap::new();
    let mut watch = |_: &Progress| Ok(());
    let keys = KeyBook::default();
    let mut job = Job::new(&world.store, &keys, &locking, &mut copies, &mut watch);
    let text = note.body.clone();
    let err = seal_note_files(&mut job, &mut note, &text, &key()).unwrap_err();
    assert!(err.to_string().contains("finished downloading"), "{err}");
    assert_eq!(
        world.store.note(&note.id).unwrap().unwrap(),
        before,
        "nothing written"
    );
    assert_eq!(
        world.store.attachment_state(&ready).unwrap(),
        AttachmentState::Plain
    );

    // A locked note's save, with the file linked since.
    let mut locked = world.note(std::slice::from_ref(&ready));
    world.lock(&mut locked, &key(), &keys);
    let body = format!("{}![](inkdrop://{waiting})\nmore text\n", locked.body);
    let counts = save_locked(&world.store, &keys, &mut locked, &body, &key()).unwrap();
    assert_eq!(counts.waiting, 1);
    let stored = world.store.note(&locked.id).unwrap().unwrap();
    let envelope = Envelope::from_extra(&stored.extra).unwrap().unwrap();
    assert!(
        key()
            .open(&stored.id, &envelope)
            .unwrap()
            .contains("more text")
    );
}

/// A PDF over 10 MB already in a note seals with its lock and opens again:
/// the store takes a sealed PDF up to the 50 MB PDF limit.
#[test]
fn a_pdf_over_ten_megabytes_seals_and_opens() {
    let world = world();
    let mut pdf = b"%PDF-1.4\n".to_vec();
    pdf.resize(12 * 1024 * 1024, b'x');
    let id = world
        .store
        .put(
            DocType::File,
            json!({"name": "big.pdf", "contentType": "application/pdf", "_attachments": {"index": {
                "content_type": "application/pdf",
                "data": STANDARD.encode(&pdf),
            }}}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    let mut note = world.note(std::slice::from_ref(&id));
    let keys = KeyBook::default();
    world.lock(&mut note, &key(), &keys);
    assert_eq!(
        world.store.attachment_state(&id).unwrap(),
        AttachmentState::Sealed
    );
    assert_eq!(bytes_of(read_file(&world.store, &id, &keys)), Some(pdf));
}

/// `set_note_files` writes a list beside a note whose text links sealed
/// files, but only those the list holds: it can't point a plain note at
/// another note's sealed file.
#[test]
fn a_list_write_never_lets_a_plain_note_link_a_sealed_file_it_doesnt_hold() {
    let world = world();
    let file = world.file("theirs.png", b"theirs");
    let note = world.note(std::slice::from_ref(&file));
    // The file is sealed (for another note) under the note's feet.
    let mut meta = world.store.file_meta(&file).unwrap().unwrap();
    let seal = SealId::random();
    FileLock::store(Some(&FileLock::sealed(seal.encode())), &mut meta.extra);
    world
        .store
        .put_sealed_file(meta, Attachment::new(vec![1; 64]))
        .unwrap();
    assert!(world.store.set_note_files(&note.id, &[]).is_err());
    let entry = NoteFile {
        file: file.clone(),
        v: 1,
        seal: seal.encode(),
        key: STANDARD.encode([0u8; 73]),
        extra: Default::default(),
    };
    assert!(world.store.set_note_files(&note.id, &[entry]).is_ok());
}

/// The store canary scan, after a lock: no byte of a locked note's picture,
/// its words read by OCR, or its PDF's page text is left anywhere in the
/// library's files (the database, its write-ahead log).
#[test]
fn a_locked_notes_files_leave_no_plaintext_in_the_library() {
    const CANARY: &str = "zqxcanary-lockflow-41c7-harbour";
    let world = world();
    let picture = world.file("canary.png", format!("PNG {CANARY} bytes").as_bytes());
    let md5 = world
        .store
        .file_meta(&picture)
        .unwrap()
        .unwrap()
        .md5digest
        .unwrap();
    assert!(
        world
            .store
            .set_image_text(&picture, &md5, &format!("ocr {CANARY}"), "test")
            .unwrap()
    );
    let pdf = world
        .store
        .put(
            DocType::File,
            json!({"name": "c.pdf", "contentType": "application/pdf", "_attachments": {"index": {
                "content_type": "application/pdf",
                "data": STANDARD.encode(b"%PDF-1.4 canary pdf body"),
            }}}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    let pdf_md5 = world
        .store
        .file_meta(&pdf)
        .unwrap()
        .unwrap()
        .md5digest
        .unwrap();
    world
        .store
        .set_pdf_text(&pdf, &pdf_md5, &[format!("page {CANARY}")], "test")
        .unwrap();
    let mut note = world.note(&[picture.clone(), pdf.clone()]);
    let scan = |world: &World| -> bool {
        let db = world.store.path().to_owned();
        ["", "-wal"].iter().any(|suffix| {
            let mut name = db.as_os_str().to_owned();
            name.push(suffix);
            std::fs::read(std::path::PathBuf::from(name)).is_ok_and(|bytes| {
                bytes
                    .windows(CANARY.len())
                    .any(|window| window == CANARY.as_bytes())
            })
        })
    };
    assert!(scan(&world), "positive control: the plain library shows it");
    world.lock(&mut note, &key(), &KeyBook::default());
    assert!(
        !scan(&world),
        "a plain copy of a locked note's file is left"
    );
}

// MARK: Remove Lock is all or nothing

const REMOVE_CANARY: &str = "zqxcanary-removelock-77d2-lantern";

impl World {
    /// A locked, open note with `count` pictures that each hold the canary,
    /// and (when `shared`) its first picture also listed by a second locked
    /// note, so Remove Lock has a copy to make. Returns the note as the
    /// editor holds it, the ids, and the keys.
    fn locked_with_canaries(&self, count: usize, shared: bool) -> (Note, Vec<String>, KeyBook) {
        let ids: Vec<String> = (0..count)
            .map(|i| {
                self.file(
                    &format!("p{i}.png"),
                    format!("PNG {REMOVE_CANARY} {i}").as_bytes(),
                )
            })
            .collect();
        let keys = KeyBook::default();
        let mut note = self.note(&ids);
        self.lock(&mut note, &key(), &keys);
        if shared && let Some(first) = ids.first() {
            let mut second = self.note(&[]);
            let envelope = other().seal(&second.id, "");
            Envelope::store(Some(&envelope), &mut second.extra);
            second.body.clear();
            self.store
                .lock_note(serde_json::to_value(&second).unwrap())
                .unwrap();
            let mut second = self.store.note(&second.id).unwrap().unwrap();
            let text = format!("![](inkdrop://{first})");
            let locking = HashSet::from([second.id.clone()]);
            let mut copies = HashMap::new();
            let mut watch = |_: &Progress| Ok(());
            let mut job = Job::new(&self.store, &keys, &locking, &mut copies, &mut watch);
            seal_note_files(&mut job, &mut second, &text, &other()).unwrap();
            second.body.clear();
            self.store
                .lock_note(serde_json::to_value(&second).unwrap())
                .unwrap();
        }
        let stored = self.store.note(&note.id).unwrap().unwrap();
        let mut open = stored.clone();
        let envelope = Envelope::from_extra(&stored.extra).unwrap().unwrap();
        open.body = key().open(&stored.id, &envelope).unwrap().to_string();
        (open, ids, keys)
    }

    fn file_documents(&self) -> usize {
        self.store
            .counts()
            .unwrap()
            .get(&DocType::File)
            .copied()
            .unwrap_or_default()
    }

    /// The canary check: while the note is locked, nothing the library's API
    /// gives out holds a listed file's plain bytes, and no file but the ones
    /// there were is left.
    fn assert_still_sealed(&self, note: &Note, ids: &[String], documents: usize) {
        let stored = self.store.note(&note.id).unwrap().unwrap();
        assert!(stored.is_locked(), "the note is still locked");
        assert_eq!(stored.body, "", "no text beside the lock");
        for id in ids {
            assert_eq!(
                self.store.attachment_state(id).unwrap(),
                AttachmentState::Sealed,
                "{id} is still sealed"
            );
            assert!(
                self.store.attachment(id).unwrap().is_none(),
                "the lock-blind API reads nothing of {id}"
            );
            let raw = self.stored(id);
            assert!(
                !raw.windows(REMOVE_CANARY.len())
                    .any(|window| window == REMOVE_CANARY.as_bytes()),
                "{id} holds no plain bytes"
            );
        }
        assert_eq!(self.file_documents(), documents, "no stray copy is left");
    }

    /// Remove Lock for `open`: prepared, then written in one transaction.
    fn remove_lock(&self, open: &mut Note, stop_at: Option<usize>) -> Result<(), String> {
        let mut seen = 0;
        let mut watch = |_: &Progress| {
            seen += 1;
            if Some(seen - 1) == stop_at {
                Err("stopped".to_owned())
            } else {
                Ok(())
            }
        };
        let (body, _, unsealing) =
            unseal_or_copy(&self.store, open, &key(), &mut watch).map_err(|err| err.to_string())?;
        open.body = body;
        Envelope::store(None, &mut open.extra);
        self.store
            .unlock_note_unsealing(serde_json::to_value(&*open).unwrap(), unsealing)
            .map(|_| ())
            .map_err(|err| err.to_string())
    }
}

/// A Remove Lock stopped before it writes (Cancel, or the app dying while it
/// opens the files) changes nothing at any file: the note is locked and every
/// file sealed, with no plain byte readable.
#[test]
fn a_remove_lock_stopped_at_any_file_changes_nothing() {
    for shared in [false, true] {
        for stop_at in 0..4 {
            let world = world();
            let (mut open, ids, _keys) = world.locked_with_canaries(4, shared);
            let documents = world.file_documents();
            let err = world.remove_lock(&mut open, Some(stop_at)).unwrap_err();
            assert!(err.contains("stopped"), "{err}");
            world.assert_still_sealed(&open, &ids, documents);
        }
    }
}

/// The write itself failing partway (a file turned stale after the first was
/// ready, as a sync or a crash would leave it) rolls everything back: files
/// before the bad one are sealed again, and the copies are gone.
#[test]
fn a_remove_lock_failing_inside_its_transaction_rolls_every_file_back() {
    for shared in [false, true] {
        for bad in 0..4 {
            let world = world();
            let (mut open, ids, _keys) = world.locked_with_canaries(4, shared);
            let documents = world.file_documents();
            let mut watch = |_: &Progress| Ok(());
            let (body, _, mut unsealing) =
                unseal_or_copy(&world.store, &mut open, &key(), &mut watch).unwrap();
            assert_eq!(unsealing.copies.len() + unsealing.files.len(), 4);
            if bad < unsealing.files.len() {
                // This file moved on since it was read.
                unsealing.files[bad].0.rev = Some("9-stale".to_owned());
            } else {
                // No file to spoil: the note moved on instead.
                open.rev = Some("9-stale".to_owned());
            }
            open.body = body;
            Envelope::store(None, &mut open.extra);
            let err = world
                .store
                .unlock_note_unsealing(serde_json::to_value(&open).unwrap(), unsealing)
                .unwrap_err();
            assert!(matches!(err, kanso_store::StoreError::Conflict), "{err:?}");
            world.assert_still_sealed(&open, &ids, documents);
        }
    }
}

/// The finished Remove Lock: every file plain, the note plain, and a shared
/// file's sealed original still there for the other locked note.
#[test]
fn a_remove_lock_that_finishes_leaves_plain_files_and_keeps_a_shared_original() {
    let world = world();
    let (mut open, ids, _keys) = world.locked_with_canaries(3, true);
    let documents = world.file_documents();
    world.remove_lock(&mut open, None).unwrap();
    let stored = world.store.note(&open.id).unwrap().unwrap();
    assert!(!stored.is_locked());
    assert!(!stored.body.contains(&ids[0]), "its link moved to a copy");
    assert_eq!(world.file_documents(), documents + 1, "one copy");
    assert_eq!(
        world.store.attachment_state(&ids[0]).unwrap(),
        AttachmentState::Sealed,
        "the other locked note keeps its sealed original"
    );
    for id in &ids[1..] {
        assert_eq!(
            world.store.attachment_state(id).unwrap(),
            AttachmentState::Plain
        );
        assert!(world.store.attachment(id).unwrap().is_some());
    }
}

proptest! {
    /// However many files a note holds and wherever Remove Lock stops, the
    /// outcome is all or nothing: finished, or locked with every file sealed.
    #[test]
    fn prop_remove_lock_is_all_or_nothing(
        count in 1usize..6,
        shared in any::<bool>(),
        stop in proptest::option::of(0usize..8),
    ) {
        let world = world();
        let (mut open, ids, _keys) = world.locked_with_canaries(count, shared);
        let documents = world.file_documents();
        match world.remove_lock(&mut open, stop) {
            Err(_) => world.assert_still_sealed(&open, &ids, documents),
            Ok(()) => {
                let stored = world.store.note(&open.id).unwrap().unwrap();
                prop_assert!(!stored.is_locked());
                let sealed = ids
                    .iter()
                    .filter(|id| world.store.attachment_state(id).unwrap() == AttachmentState::Sealed)
                    .count();
                prop_assert_eq!(sealed, usize::from(shared));
            }
        }
    }
}

/// A PDF over the 10 MB image limit (the PDF limit is 50 MB) seals with its
/// note and comes back byte for byte when the lock is removed: the lock
/// itself has no 10 MB limit of its own.
// no-prop-test: one large file; the size is the input and a property run
// would repeat 12 MB seals for nothing.
#[test]
fn a_pdf_over_ten_megabytes_seals_and_comes_back_when_the_lock_is_removed() {
    let world = world();
    let mut pdf = b"%PDF-1.7\n".to_vec();
    pdf.resize(
        kanso_core::model::MAX_ATTACHMENT_SIZE + 2 * 1024 * 1024,
        b'p',
    );
    let id = world
        .store
        .put(
            DocType::File,
            json!({
                "name": "contract.pdf",
                "contentType": "application/pdf",
                "_attachments": {"index": {
                    "content_type": "application/pdf",
                    "data": STANDARD.encode(&pdf),
                }},
            }),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    let keys = KeyBook::default();
    let mut note = world.note(std::slice::from_ref(&id));
    world.lock(&mut note, &key(), &keys);
    assert_eq!(
        world.store.attachment_state(&id).unwrap(),
        AttachmentState::Sealed
    );
    assert!(world.store.attachment(&id).unwrap().is_none());

    let stored = world.store.note(&note.id).unwrap().unwrap();
    let mut open = stored.clone();
    let envelope = Envelope::from_extra(&stored.extra).unwrap().unwrap();
    open.body = key().open(&stored.id, &envelope).unwrap().to_string();
    world.remove_lock(&mut open, None).unwrap();
    assert_eq!(
        world.store.attachment_state(&id).unwrap(),
        AttachmentState::Plain
    );
    let (_, back) = world.store.attachment(&id).unwrap().unwrap();
    assert_eq!(back.data, pdf);
}
