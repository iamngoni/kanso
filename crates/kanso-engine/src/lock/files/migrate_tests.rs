//! The migration of notes locked before attachments were sealed, against a
//! real store.

use std::sync::{Arc, LazyLock};

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use kanso_core::tasks::linked_file_ids;
use kanso_core::{DocType, Note};
use kanso_store::{AttachmentState, Origin, Put, Store};
use kanso_sync::KdfParams;
use kanso_sync::lock::{Envelope, LockKey, LockPassword};
use proptest::prelude::*;
use proptest::test_runner::RngSeed;
use serde_json::json;

use super::super::{Reading, read_file};
use super::*;

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

fn key() -> Arc<LockKey> {
    Arc::new(LockKey::from_secret(&KEY.to_secret()).unwrap())
}

fn other() -> Arc<LockKey> {
    Arc::new(LockKey::from_secret(&OTHER.to_secret()).unwrap())
}

const CANARY: &[u8] = b"CANARY-9f3a1c5e-plaintext-attachment-bytes";

struct World {
    dir: tempfile::TempDir,
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
    World { dir, store, book }
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

    fn note_in(&self, book: &str, files: &[String]) -> Note {
        let body: String = files
            .iter()
            .map(|id| format!("![](inkdrop://{id})\n"))
            .collect();
        let id = self
            .store
            .put(
                DocType::Note,
                json!({"bookId": book, "title": "T", "body": body}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        self.store.note(&id).unwrap().unwrap()
    }

    fn note(&self, files: &[String]) -> Note {
        self.note_in(&self.book, files)
    }

    /// Locks `note` as Kanso did before attachments were sealed: the text
    /// is sealed, the files stay plain, and there is no list.
    fn legacy_lock(&self, note: &mut Note, key: &LockKey) {
        let envelope = key.seal(&note.id, &note.body);
        Envelope::store(Some(&envelope), &mut note.extra);
        let mut stored = note.clone();
        stored.body.clear();
        let written = self
            .store
            .lock_note(serde_json::to_value(&stored).unwrap())
            .unwrap();
        note.rev = Some(written.rev);
    }

    fn state(&self, id: &str) -> AttachmentState {
        self.store.attachment_state(id).unwrap()
    }

    fn stored(&self, id: &str) -> Vec<u8> {
        self.store.stored_attachment(id).unwrap().unwrap().data
    }

    /// The files a locked note links, by opening its text with `key`.
    fn links(&self, id: &str, key: &LockKey) -> Vec<String> {
        let stored = self.store.note(id).unwrap().unwrap();
        let envelope = Envelope::from_extra(&stored.extra).unwrap().unwrap();
        linked_file_ids(&key.open(id, &envelope).unwrap())
    }

    /// The bytes of each file a locked note links, read with the keys its
    /// own list gives.
    fn contents(&self, id: &str, key: &LockKey) -> Vec<Option<Vec<u8>>> {
        let stored = self.store.note(id).unwrap().unwrap();
        let keys = KeyBook::default();
        keys.remember_note(&stored, key);
        self.links(id, key)
            .iter()
            .map(|file| match read_file(&self.store, file, &keys) {
                Reading::Opened(opened) => Some(opened.bytes),
                _ => None,
            })
            .collect()
    }

    /// Every byte the library's files hold, the write-ahead log too.
    fn raw(&self) -> Vec<u8> {
        let mut all = Vec::new();
        for name in ["k.db", "k.db-wal", "k.db-shm"] {
            if let Ok(bytes) = std::fs::read(self.dir.path().join(name)) {
                all.extend(bytes);
            }
        }
        all
    }
}

fn holds(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

fn run(world: &World, keys: &[Arc<LockKey>]) -> Outcome {
    migrate(&world.store, &KeyBook::default(), keys, &mut |_| Ok(()))
}

#[test]
fn a_note_locked_before_unlocks_seals_its_attachment_and_leaves_no_canary() {
    let world = world();
    let file = world.file("a.png", CANARY);
    let mut note = world.note(std::slice::from_ref(&file));
    world.legacy_lock(&mut note, &key());
    assert!(holds(&world.raw(), CANARY), "the control: it is on disk");
    assert_eq!(
        pending(&world.store),
        Pending {
            legacy: 1,
            unsealed: 0,
            stopped: 0
        }
    );

    let outcome = run(&world, &[key()]);
    assert_eq!(outcome.notes, 1);
    assert_eq!(outcome.sealed, 1);
    assert!(outcome.failed.is_empty());

    assert_eq!(world.state(&file), AttachmentState::Sealed);
    let stored = world.store.note(&note.id).unwrap().unwrap();
    assert!(stored.is_locked());
    let list = note_files_from_extra(&stored.extra).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].file, file);
    assert_eq!(
        world.contents(&note.id, &key()),
        vec![Some(CANARY.to_vec())]
    );
    assert!(
        !holds(&world.raw(), CANARY),
        "no canary in the database or the log"
    );
    assert!(pending(&world.store).is_done());
}

#[test]
fn a_file_an_unlocked_note_shows_is_copied_and_the_original_stays() {
    let world = world();
    let file = world.file("a.png", b"shared picture");
    let unlocked = world.note(std::slice::from_ref(&file));
    let mut locked = world.note(std::slice::from_ref(&file));
    world.legacy_lock(&mut locked, &key());

    let outcome = run(&world, &[key()]);
    assert_eq!((outcome.sealed, outcome.copied), (0, 1));
    // The unlocked note keeps its plain original.
    assert_eq!(world.state(&file), AttachmentState::Plain);
    assert_eq!(world.stored(&file), b"shared picture");
    assert_eq!(
        world.store.note(&unlocked.id).unwrap().unwrap().body,
        unlocked.body
    );
    // The locked note has a sealed copy of its own.
    let links = world.links(&locked.id, &key());
    assert_eq!(links.len(), 1);
    assert_ne!(links[0], file);
    assert_eq!(world.state(&links[0]), AttachmentState::Sealed);
    assert_eq!(
        world.contents(&locked.id, &key()),
        vec![Some(b"shared picture".to_vec())]
    );
}

#[test]
fn a_note_in_a_locked_notebook_is_covered_whether_sealed_or_not() {
    let world = world();
    let book = world
        .store
        .put(
            DocType::Book,
            json!({"name": "Vault"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    // A note put there plain, and one locked there by an older Kanso.
    let plain_file = world.file("p.png", b"plain in the vault");
    let loose = world.note_in(&book, std::slice::from_ref(&plain_file));
    let sealed_file = world.file("s.png", b"sealed note's file");
    let mut sealed = world.note_in(&book, std::slice::from_ref(&sealed_file));
    world.legacy_lock(&mut sealed, &key());
    // The notebook, locked on another device, leaving the loose note.
    let mut vault = world
        .store
        .books()
        .unwrap()
        .into_iter()
        .find(|b| b.id == book)
        .unwrap();
    Envelope::store(Some(&key().seal(&book, "")), &mut vault.extra);
    world
        .store
        .put(
            DocType::Book,
            serde_json::to_value(&vault).unwrap(),
            Put::Upsert,
            Origin::Sync,
        )
        .unwrap();
    assert_eq!(pending(&world.store).unsealed, 1);
    assert_eq!(pending(&world.store).legacy, 1);

    let outcome = run(&world, &[key()]);
    assert_eq!(outcome.notes, 2);
    assert_eq!(outcome.sealed, 2);
    assert!(pending(&world.store).is_done());
    assert_eq!(world.state(&plain_file), AttachmentState::Sealed);
    assert_eq!(world.state(&sealed_file), AttachmentState::Sealed);
    assert_eq!(
        world.contents(&loose.id, &key()),
        vec![Some(b"plain in the vault".to_vec())]
    );
    assert_eq!(
        world.contents(&sealed.id, &key()),
        vec![Some(b"sealed note's file".to_vec())]
    );
}

#[test]
fn a_note_the_password_does_not_open_is_left_untouched() {
    let world = world();
    let mine = world.file("m.png", b"mine");
    let theirs = world.file("t.png", b"theirs");
    let mut a = world.note(std::slice::from_ref(&mine));
    let mut b = world.note(std::slice::from_ref(&theirs));
    world.legacy_lock(&mut a, &key());
    world.legacy_lock(&mut b, &other());
    let before = world.store.note(&b.id).unwrap().unwrap();

    let outcome = run(&world, &[key()]);
    assert_eq!(outcome.notes, 1);
    assert_eq!(outcome.unopened, 1);
    assert_eq!(world.state(&theirs), AttachmentState::Plain);
    assert_eq!(world.store.note(&b.id).unwrap().unwrap(), before);
    assert_eq!(pending(&world.store).legacy, 1);

    // Its own password finishes it.
    let outcome = run(&world, &[key(), other()]);
    assert_eq!((outcome.notes, outcome.unopened), (1, 0));
    assert_eq!(world.state(&theirs), AttachmentState::Sealed);
}

#[test]
fn an_empty_note_is_marked_checked_and_not_written_twice() {
    let world = world();
    let mut note = world.note(&[]);
    world.legacy_lock(&mut note, &key());
    assert_eq!(run(&world, &[key()]).notes, 1);
    let after = world.store.note(&note.id).unwrap().unwrap();
    assert_eq!(note_files_from_extra(&after.extra), Some(Vec::new()));
    assert_eq!(run(&world, &[key()]).notes, 0);
    assert_eq!(world.store.note(&note.id).unwrap().unwrap(), after);
}

/// What a library of legacy notes looks like to the properties.
#[derive(Clone, Debug)]
struct Library {
    /// The bytes of each file.
    files: Vec<Vec<u8>>,
    /// For each legacy locked note, the files it links.
    locked: Vec<Vec<usize>>,
    /// For each unlocked note, the files it links.
    unlocked: Vec<Vec<usize>>,
}

fn libraries() -> impl Strategy<Value = Library> {
    (1usize..5).prop_flat_map(|count| {
        (
            prop::collection::vec(prop::collection::vec(any::<u8>(), 1..500), count),
            prop::collection::vec(prop::collection::vec(0..count, 0..4), 1..4),
            prop::collection::vec(prop::collection::vec(0..count, 0..3), 0..3),
        )
            .prop_map(|(files, locked, unlocked)| Library {
                files,
                locked,
                unlocked,
            })
    })
}

/// The bytes a note's links should open to: each file once, in order.
fn expected(library: &Library, picks: &[usize]) -> Vec<Option<Vec<u8>>> {
    let mut seen = Vec::new();
    for &pick in picks {
        if !seen.contains(&pick) {
            seen.push(pick);
        }
    }
    seen.iter()
        .map(|&i| Some(library.files[i].clone()))
        .collect()
}

fn build(world: &World, library: &Library) -> (Vec<String>, Vec<Note>, Vec<Note>) {
    let ids: Vec<String> = library
        .files
        .iter()
        .enumerate()
        .map(|(i, bytes)| world.file(&format!("f{i}.png"), bytes))
        .collect();
    let pick =
        |picks: &Vec<usize>| -> Vec<String> { picks.iter().map(|&i| ids[i].clone()).collect() };
    let unlocked: Vec<Note> = library
        .unlocked
        .iter()
        .map(|picks| world.note(&pick(picks)))
        .collect();
    let locked: Vec<Note> = library
        .locked
        .iter()
        .map(|picks| {
            let mut note = world.note(&pick(picks));
            world.legacy_lock(&mut note, &key());
            note
        })
        .collect();
    (ids, locked, unlocked)
}

proptest! {
    #![proptest_config(ProptestConfig {
        rng_seed: RngSeed::Fixed(0x0044_0007),
        cases: 48,
        ..ProptestConfig::default()
    })]

    /// After the migration no locked note links a plain file, and every
    /// file a locked note links opens to the bytes it had; an unlocked note
    /// keeps its readable files.
    #[test]
    fn prop_after_migration_no_locked_note_links_a_plain_file_and_every_file_opens(
        library in libraries(),
    ) {
        let world = world();
        let (ids, locked, unlocked) = build(&world, &library);
        let outcome = run(&world, &[key()]);
        prop_assert!(outcome.failed.is_empty());
        prop_assert_eq!(outcome.notes, locked.len());
        for (note, picks) in locked.iter().zip(&library.locked) {
            for file in world.links(&note.id, &key()) {
                prop_assert_eq!(world.state(&file), AttachmentState::Sealed);
            }
            prop_assert_eq!(
                world.contents(&note.id, &key()),
                expected(&library, picks)
            );
        }
        for (note, picks) in unlocked.iter().zip(&library.unlocked) {
            let after = world.store.note(&note.id).unwrap().unwrap();
            prop_assert_eq!(&after.body, &note.body);
            for &i in picks {
                prop_assert_eq!(world.state(&ids[i]), AttachmentState::Plain);
                prop_assert_eq!(world.stored(&ids[i]), library.files[i].clone());
            }
        }
        prop_assert!(pending(&world.store).is_done());
    }

    /// Running it twice seals once: the second run changes nothing, and no
    /// sealed file is written again.
    #[test]
    fn prop_migration_is_idempotent(library in libraries()) {
        let world = world();
        let (ids, locked, _) = build(&world, &library);
        run(&world, &[key()]);
        let mut all = ids.clone();
        for note in &locked {
            all.extend(world.links(&note.id, &key()));
        }
        all.sort();
        all.dedup();
        let blobs: Vec<Vec<u8>> = all.iter().map(|id| world.stored(id)).collect();
        let notes: Vec<Note> = locked
            .iter()
            .map(|note| world.store.note(&note.id).unwrap().unwrap())
            .collect();
        let again = run(&world, &[key()]);
        prop_assert_eq!(again.notes, 0);
        prop_assert_eq!((again.sealed, again.copied), (0, 0));
        let after: Vec<Vec<u8>> = all.iter().map(|id| world.stored(id)).collect();
        prop_assert_eq!(blobs, after);
        for note in &notes {
            prop_assert_eq!(&world.store.note(&note.id).unwrap().unwrap(), note);
        }
    }
}

/// The calls the watch hears for a migration of `build`'s library.
fn steps(library: &Library) -> usize {
    let world = world();
    build(&world, library);
    let mut calls = 0;
    migrate(&world.store, &KeyBook::default(), &[key()], &mut |_| {
        calls += 1;
        Ok(())
    });
    calls
}

fn crash_library() -> Library {
    Library {
        files: vec![
            b"first file bytes".to_vec(),
            b"second file bytes".to_vec(),
            b"shared file bytes".to_vec(),
        ],
        locked: vec![vec![0, 1, 2], vec![1, 2]],
        unlocked: vec![vec![2]],
    }
}

/// Whether every file still has its bytes: a plain one as it was, a sealed
/// one readable with the keys some locked note's list gives.
fn nothing_is_lost(world: &World, library: &Library, ids: &[String]) -> Result<(), String> {
    let keys = KeyBook::default();
    for note in world.store.locked_notes().unwrap() {
        keys.remember_note(&note, &key());
    }
    for (id, bytes) in ids.iter().zip(&library.files) {
        match world.state(id) {
            AttachmentState::Plain => {
                if &world.stored(id) != bytes {
                    return Err(format!("{id} changed while plain"));
                }
            }
            AttachmentState::Sealed => match read_file(&world.store, id, &keys) {
                Reading::Opened(opened) if &opened.bytes == bytes => {}
                other => return Err(format!("{id} is sealed and does not open: {other:?}")),
            },
            AttachmentState::Missing => return Err(format!("{id} is gone")),
        }
    }
    Ok(())
}

#[test]
fn a_crash_at_each_step_loses_no_file_and_the_next_run_finishes() {
    let library = crash_library();
    for stop in 0..steps(&library) {
        let world = world();
        let (ids, locked, _) = build(&world, &library);
        let mut calls = 0;
        let crashed = migrate(&world.store, &KeyBook::default(), &[key()], &mut |_| {
            calls += 1;
            if calls == stop + 1 {
                Err("crash".into())
            } else {
                Ok(())
            }
        });
        assert!(crashed.stopped, "step {stop} stops the run");
        nothing_is_lost(&world, &library, &ids).unwrap_or_else(|why| panic!("step {stop}: {why}"));

        // The app starts again: no keys, the password alone.
        let resumed = run(&world, &[key()]);
        assert!(
            resumed.failed.is_empty(),
            "step {stop}: {:?}",
            resumed.failed
        );
        assert!(!resumed.stopped);
        assert!(pending(&world.store).is_done(), "step {stop}");
        for (note, picks) in locked.iter().zip(&library.locked) {
            assert_eq!(
                world.contents(&note.id, &key()),
                expected(&library, picks),
                "step {stop}"
            );
            for file in world.links(&note.id, &key()) {
                assert_eq!(world.state(&file), AttachmentState::Sealed, "step {stop}");
            }
        }
    }
}

#[test]
fn a_crash_at_each_step_can_be_undone_with_the_password() {
    let library = crash_library();
    for stop in 0..steps(&library) {
        let world = world();
        let (ids, locked, _) = build(&world, &library);
        let before: Vec<Note> = locked
            .iter()
            .map(|note| world.store.note(&note.id).unwrap().unwrap())
            .collect();
        let mut calls = 0;
        migrate(&world.store, &KeyBook::default(), &[key()], &mut |_| {
            calls += 1;
            if calls == stop + 1 {
                Err("crash".into())
            } else {
                Ok(())
            }
        });
        let copies: Vec<String> = world
            .store
            .locked_notes()
            .unwrap()
            .iter()
            .filter(|note| is_stopped(note))
            .flat_map(|note| note_files_from_extra(&note.extra).unwrap_or_default())
            .filter(|entry| entry.extra.contains_key(JOURNAL_FROM))
            .map(|entry| entry.file)
            .collect();
        // Another password undoes nothing.
        let (_, unopened) = undo_all(&world.store, &[other()]).unwrap();
        let stopped = pending(&world.store).stopped;
        assert_eq!(unopened, stopped, "step {stop}");

        undo_all(&world.store, &[key()]).unwrap_or_else(|why| panic!("step {stop}: {why}"));
        assert_eq!(pending(&world.store).stopped, 0, "step {stop}");
        // Every file has its bytes, and no copy the undone run made is left.
        nothing_is_lost(&world, &library, &ids).unwrap_or_else(|why| panic!("step {stop}: {why}"));
        for copy in &copies {
            assert_eq!(world.state(copy), AttachmentState::Missing, "step {stop}");
        }
        // A note the run finished before the crash keeps its list; one it
        // hadn't reached is as it was.
        for (note, was) in locked.iter().zip(&before) {
            let now = world.store.note(&note.id).unwrap().unwrap();
            if !is_stopped(&now) && note_files_from_extra(&now.extra).is_none() {
                assert_eq!(&now.extra, &was.extra, "step {stop}");
            }
        }
        // And the migration still goes through afterwards.
        let done = run(&world, &[key()]);
        assert!(done.failed.is_empty(), "step {stop}: {:?}", done.failed);
        assert!(pending(&world.store).is_done(), "step {stop}");
        nothing_is_lost(&world, &library, &ids).unwrap_or_else(|why| panic!("step {stop}: {why}"));
    }
}

#[test]
fn the_scan_counts_what_the_keys_open_and_what_they_do_not() {
    let world = world();
    let a = world.file("a.png", &[1; 1000]);
    let b = world.file("b.png", &[2; 3000]);
    let shared = world.file("s.png", &[3; 500]);
    let _unlocked = world.note(std::slice::from_ref(&shared));
    let mut one = world.note(&[a, shared]);
    let mut two = world.note(std::slice::from_ref(&b));
    world.legacy_lock(&mut one, &key());
    world.legacy_lock(&mut two, &other());

    let scan = scan(&world.store, &[key()]);
    assert_eq!(scan.notes, 1);
    assert_eq!(scan.unopened, 1);
    assert_eq!(scan.summary.files, 2);
    assert_eq!(scan.summary.bytes, 1500);
    assert_eq!(scan.summary.shared, 1);
}

#[test]
fn a_password_gives_the_key_of_each_lock_it_opens_and_only_those() {
    let world = world();
    let a = world.file("a.png", b"a");
    let b = world.file("b.png", b"b");
    let mut one = world.note(std::slice::from_ref(&a));
    let mut two = world.note(std::slice::from_ref(&b));
    world.legacy_lock(&mut one, &key());
    world.legacy_lock(&mut two, &other());

    let keys: Vec<Arc<LockKey>> =
        keys_for(&world.store, &LockPassword::new("first password").unwrap())
            .unwrap()
            .into_iter()
            .map(Arc::new)
            .collect();
    assert_eq!(keys.len(), 1);
    assert_eq!(run(&world, &keys).notes, 1);
    let wrong = keys_for(
        &world.store,
        &LockPassword::new("not the password").unwrap(),
    );
    assert!(wrong.is_err());
    // The other password's note still waits.
    assert_eq!(pending(&world.store).legacy, 1);
}

// MARK: Plaintext left anywhere in the library's folder

/// Every file under `dir`, read whole, the folders inside it too.
fn every_byte_under(dir: &std::path::Path) -> Vec<u8> {
    let mut all = Vec::new();
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            all.extend(every_byte_under(&path));
        } else if let Ok(bytes) = std::fs::read(&path) {
            all.extend(bytes);
        }
    }
    all
}

/// What a note locked before sealing existed may have left in the library:
/// the file's bytes, its name, the text OCR read from it, the text of a
/// PDF's pages, and an earlier version of the file in its history. After
/// the migration none of it is anywhere in the folder: the database, its
/// log, or any other file there.
#[test]
fn after_migration_no_plaintext_of_a_migrated_file_is_left_in_the_folder() {
    const NAME: &[u8] = b"canaryname-4d1e.png";
    const OLD_NAME: &[u8] = b"canaryoldname-8b2c.png";
    const OCR: &[u8] = b"ocr-canary-0e7d5a";
    const PDF_BYTES: &[u8] = b"%PDF-1.4 pdf-bytes-canary-61c0";
    const PDF_TEXT: &[u8] = b"pdf-page-canary-93af";
    let world = world();
    let image = world.file(std::str::from_utf8(OLD_NAME).unwrap(), CANARY);
    // Renamed: the old name is in the file's history now.
    let meta = world.store.file_meta(&image).unwrap().unwrap();
    world
        .store
        .put(
            DocType::File,
            json!({"_id": image, "_rev": meta.rev, "name": std::str::from_utf8(NAME).unwrap()}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    let pdf = world
        .store
        .put(
            DocType::File,
            json!({"name": "doc.pdf", "contentType": "application/pdf",
                "_attachments": {"index": {"content_type": "application/pdf",
                "data": STANDARD.encode(PDF_BYTES)}}}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    let mut note = world.note(&[image.clone(), pdf.clone()]);
    // Read while the note was plain.
    let md5 = |id: &str| world.store.file_meta(id).unwrap().unwrap().md5digest.unwrap();
    assert!(
        world
            .store
            .set_image_text(&image, &md5(&image), std::str::from_utf8(OCR).unwrap(), "test")
            .unwrap()
    );
    assert!(
        world
            .store
            .set_pdf_text(
                &pdf,
                &md5(&pdf),
                &[std::str::from_utf8(PDF_TEXT).unwrap().to_owned()],
                "test"
            )
            .unwrap()
    );
    // Locked by an older Kanso on another device, and synced here: nothing
    // derived from its files was forgotten.
    let envelope = key().seal(&note.id, &note.body);
    Envelope::store(Some(&envelope), &mut note.extra);
    note.body.clear();
    world
        .store
        .put(
            DocType::Note,
            serde_json::to_value(&note).unwrap(),
            Put::Upsert,
            Origin::Sync,
        )
        .unwrap();
    assert_eq!(pending(&world.store).legacy, 1);
    let before = every_byte_under(world.dir.path());
    for canary in [CANARY, NAME, OLD_NAME, OCR, PDF_BYTES, PDF_TEXT] {
        assert!(
            holds(&before, canary),
            "the control: {} is on disk",
            String::from_utf8_lossy(canary)
        );
    }

    let outcome = run(&world, &[key()]);
    assert!(outcome.failed.is_empty(), "{:?}", outcome.failed);
    assert_eq!(outcome.sealed, 2);
    let after = every_byte_under(world.dir.path());
    for canary in [CANARY, NAME, OLD_NAME, OCR, PDF_BYTES, PDF_TEXT] {
        assert!(
            !holds(&after, canary),
            "{} is left in the folder",
            String::from_utf8_lossy(canary)
        );
    }
    assert_eq!(
        world.contents(&note.id, &key()),
        vec![Some(CANARY.to_vec()), Some(PDF_BYTES.to_vec())]
    );
}

// MARK: Races, other writers, ownership and other passwords

/// Stored locked note `id`, its text opened with `key`.
fn opened(world: &World, id: &str, key: &LockKey) -> Note {
    let mut note = world.store.note(id).unwrap().unwrap();
    let envelope = Envelope::from_extra(&note.extra).unwrap().unwrap();
    note.body = key.open(id, &envelope).unwrap().to_string();
    note
}

/// The migration pass and the lazy seal of a note's first open (the phone)
/// or first save (the desktop) can reach the same note at once. The second
/// must not write its journal under the first one's feet: the first seals
/// with its own journal's keys, and a file sealed under keys no stored list
/// holds is lost when the app quits.
#[test]
fn a_second_pass_cannot_rewrite_the_journal_of_a_note_being_sealed() {
    let world = world();
    let file = world.file("a.png", b"raced picture");
    let mut note = world.note(std::slice::from_ref(&file));
    world.legacy_lock(&mut note, &key());

    let mut raced = None;
    let outcome = migrate(&world.store, &KeyBook::default(), &[key()], &mut |progress| {
        if progress.phase == super::super::Phase::Sealing && raced.is_none() {
            // The pass wrote its journal and seals next. The lazy seal starts
            // on the same note, writes its own journal, and is stopped.
            let before = world.store.note(&note.id).unwrap().unwrap();
            let mut lazy = opened(&world, &note.id, &key());
            let text = lazy.body.clone();
            let locking = HashSet::from([lazy.id.clone()]);
            let mut copies = HashMap::new();
            let book = KeyBook::default();
            let mut stop = |progress: &Progress| {
                if progress.phase == super::super::Phase::Sealing {
                    Err("quit".to_owned())
                } else {
                    Ok(())
                }
            };
            let mut job = Job::new(&world.store, &book, &locking, &mut copies, &mut stop);
            let result = seal_note_files(&mut job, &mut lazy, &text, &key());
            let after = world.store.note(&note.id).unwrap().unwrap();
            raced = Some((result.is_err(), before == after));
        }
        Ok(())
    });
    let (refused, untouched) = raced.expect("the pass reached its files");
    assert!(refused, "the second pass is refused");
    assert!(untouched, "and writes nothing");
    assert!(outcome.failed.is_empty(), "{:?}", outcome.failed);
    // What the store alone opens, as after a quit.
    assert_eq!(
        world.contents(&note.id, &key()),
        vec![Some(b"raced picture".to_vec())]
    );
}

/// A writer the pass can't hold off (sync bringing the note from another
/// device) replaces the note's journal between its writing and the sealing.
/// A file is only left sealed when the keys the store holds open it; else
/// its plain bytes are put back and the next run seals it.
#[test]
fn a_file_is_only_left_sealed_when_the_stored_list_opens_it() {
    let world = world();
    let file = world.file("a.png", b"picture under a rewritten list");
    let mut note = world.note(std::slice::from_ref(&file));
    world.legacy_lock(&mut note, &key());

    let mut rewritten = false;
    let outcome = migrate(&world.store, &KeyBook::default(), &[key()], &mut |progress| {
        if progress.phase == super::super::Phase::Sealing && !rewritten {
            rewritten = true;
            // Another device's version of the note, from before its list.
            let mut theirs = world.store.note(&note.id).unwrap().unwrap();
            theirs.extra.remove(KANSO_FILES_FIELD);
            world
                .store
                .put(
                    DocType::Note,
                    serde_json::to_value(&theirs).unwrap(),
                    Put::Upsert,
                    Origin::Sync,
                )
                .unwrap();
        }
        Ok(())
    });
    assert!(rewritten);
    assert_eq!(outcome.failed.len(), 1, "the note is tried again");
    assert_eq!(world.state(&file), AttachmentState::Plain);
    assert_eq!(world.stored(&file), b"picture under a rewritten list");

    let again = run(&world, &[key()]);
    assert!(again.failed.is_empty(), "{:?}", again.failed);
    assert_eq!(world.state(&file), AttachmentState::Sealed);
    assert_eq!(
        world.contents(&note.id, &key()),
        vec![Some(b"picture under a rewritten list".to_vec())]
    );
}

/// The store keeps a file for any live note whose JSON names it (Empty
/// Trash's `still_owned`), a canvas card the canvas parser rejects too. Such
/// an unlocked note owns the file here as well: the locked note gets a
/// sealed copy, and the original is never sealed out from under it.
#[test]
fn a_file_an_unlocked_note_names_outside_a_link_stays_plain_for_it() {
    let world = world();
    let file = world.file("a.png", b"card picture");
    // Two cards with one ID: not a canvas the parser accepts, but its card
    // names the file.
    let canvas = format!(
        "```canvas\n{{\"nodes\":[{{\"id\":\"c\",\"type\":\"file\",\"file\":\"{file}\",\"x\":0,\"y\":0,\"width\":10,\"height\":10}},{{\"id\":\"c\",\"type\":\"text\",\"text\":\"t\",\"x\":0,\"y\":0,\"width\":10,\"height\":10}}],\"edges\":[]}}\n```\n"
    );
    assert!(kanso_core::tasks::shown_file_ids(&canvas).is_empty());
    world
        .store
        .put(
            DocType::Note,
            json!({"bookId": world.book, "title": "Board", "body": canvas}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
    let mut locked = world.note(std::slice::from_ref(&file));
    world.legacy_lock(&mut locked, &key());

    let outcome = run(&world, &[key()]);
    assert!(outcome.failed.is_empty(), "{:?}", outcome.failed);
    assert_eq!(world.state(&file), AttachmentState::Plain);
    assert_eq!(world.stored(&file), b"card picture");
    assert_eq!(
        world.contents(&locked.id, &key()),
        vec![Some(b"card picture".to_vec())]
    );
}

/// A file note A (one password) and note B (another) both link, before
/// either is migrated. A pass with A's password can't read B, so can't know
/// B links it: sealing it in place for A would leave B, opened later with
/// its own password, without its picture. B keeps it readable with its own
/// password, and both end up sealed.
#[test]
fn a_file_an_unopened_note_may_link_stays_readable_with_that_note_s_password() {
    let world = world();
    let file = world.file("s.png", b"picture of two passwords");
    let mut a = world.note(std::slice::from_ref(&file));
    let mut b = world.note(std::slice::from_ref(&file));
    world.legacy_lock(&mut a, &key());
    world.legacy_lock(&mut b, &other());

    let first = run(&world, &[key()]);
    assert!(first.failed.is_empty(), "{:?}", first.failed);
    assert_eq!(first.unopened, 1);
    assert_eq!(
        world.contents(&a.id, &key()),
        vec![Some(b"picture of two passwords".to_vec())]
    );

    // Later, B's password alone, on a fresh launch.
    let second = run(&world, &[other()]);
    assert!(second.failed.is_empty(), "{:?}", second.failed);
    assert!(pending(&world.store).is_done());
    for (id, k) in [(&a.id, key()), (&b.id, other())] {
        assert_eq!(
            world.contents(id, &k),
            vec![Some(b"picture of two passwords".to_vec())]
        );
        for linked in world.links(id, &k) {
            assert_eq!(world.state(&linked), AttachmentState::Sealed);
        }
    }
}
