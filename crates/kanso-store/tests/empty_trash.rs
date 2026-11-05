//! Empty Trash (issue #44, answer 4): every trashed note goes, locked ones
//! too, with the attachments only they owned; an attachment a note outside
//! the trash still owns stays.

use base64::Engine as _;
use kanso_core::DocType;
use kanso_core::model::{FileLock, KANSO_FILES_FIELD, LOCK_FIELD};
use kanso_store::{Attachment, Emptied, Origin, Put, Store, TrashCounts};
use proptest::prelude::*;
use serde_json::{Value, json};

fn store() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    (dir, store)
}

fn file(store: &Store) -> String {
    let data = base64::engine::general_purpose::STANDARD.encode(b"bytes");
    store
        .put(
            DocType::File,
            json!({"name": "a.png", "contentType": "image/png",
                   "_attachments": {"index": {"content_type": "image/png", "data": data}}}),
            Put::Upsert,
            Origin::Api,
        )
        .unwrap()
        .id
}

/// Makes `id` a sealed file the way the app does.
fn seal(store: &Store, id: &str) {
    let mut meta = store.file_meta(id).unwrap().unwrap();
    FileLock::store(
        Some(&FileLock::sealed(
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode([7u8; 16]),
        )),
        &mut meta.extra,
    );
    store
        .put_sealed_file(meta, Attachment::new(vec![0x01, 0x01, 0x00, 0, 0, 0, 0]))
        .unwrap();
}

/// A note in `book` owning `files`: locked ones list them, plain ones link
/// them in the body.
fn note(store: &Store, id: &str, book: &str, locked: bool, files: &[String]) {
    let mut doc = json!({"_id": id, "title": id, "body": "", "bookId": book});
    if locked {
        doc[LOCK_FIELD] = json!({"v": 1});
        doc[KANSO_FILES_FIELD] = files
            .iter()
            .map(|f| json!({"file": f, "v": 1, "seal": "AAAAAAAAAAAAAAAAAAAAAA", "key": "wrapped"}))
            .collect::<Vec<Value>>()
            .into();
    } else {
        doc["body"] = files
            .iter()
            .map(|f| format!("![](inkdrop://{f})"))
            .collect::<Vec<_>>()
            .join(" ")
            .into();
    }
    store
        .put(DocType::Note, doc, Put::Upsert, Origin::Ui)
        .unwrap();
}

fn book(store: &Store) {
    store
        .put(
            DocType::Book,
            json!({"_id": "book:b", "name": "B"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
}

#[test]
fn locked_notes_and_their_sealed_files_are_deleted_but_a_shared_file_stays() {
    let (_dir, store) = store();
    book(&store);
    let only = file(&store);
    let shared = file(&store);
    seal(&store, &only);
    note(
        &store,
        "note:secret",
        "trash",
        true,
        &[only.clone(), shared.clone()],
    );
    note(&store, "note:plain", "trash", false, &[]);
    note(
        &store,
        "note:live",
        "book:b",
        true,
        std::slice::from_ref(&shared),
    );

    assert_eq!(
        store.trash_counts().unwrap(),
        TrashCounts {
            notes: 2,
            locked: 1
        }
    );
    assert_eq!(
        store.empty_trash(Origin::Ui).unwrap(),
        Emptied {
            notes: 2,
            locked: 1,
            files: 1
        }
    );
    assert!(store.note("note:secret").unwrap().is_none());
    assert!(store.note("note:plain").unwrap().is_none());
    assert!(store.note("note:live").unwrap().is_some());
    assert!(store.file_meta(&only).unwrap().is_none());
    assert!(store.file_meta(&shared).unwrap().is_some());
    assert_eq!(store.trash_counts().unwrap(), TrashCounts::default());
    assert_eq!(
        store.empty_trash(Origin::Ui).unwrap(),
        Emptied::default(),
        "a second run has nothing left"
    );
}

#[test]
fn a_file_no_note_owns_is_left_alone() {
    let (_dir, store) = store();
    let stray = file(&store);
    note(&store, "note:gone", "trash", false, &[]);
    store.empty_trash(Origin::Ui).unwrap();
    assert!(store.file_meta(&stray).unwrap().is_some());
}

#[derive(Clone, Debug)]
struct NoteSpec {
    trashed: bool,
    locked: bool,
    files: Vec<usize>,
}

fn specs() -> impl Strategy<Value = Vec<NoteSpec>> {
    proptest::collection::vec(
        (
            any::<bool>(),
            any::<bool>(),
            proptest::collection::vec(0usize..5, 0..4),
        )
            .prop_map(|(trashed, locked, files)| NoteSpec {
                trashed,
                locked,
                files,
            }),
        0..8,
    )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// Invariant: after Empty Trash no trashed note remains, no file owned
    /// only by trashed notes remains, and a file a note outside the trash
    /// owns, or no note owns, survives.
    #[test]
    fn prop_empty_trash_leaves_no_trashed_note_or_trash_only_file(specs in specs()) {
        let (_dir, store) = store();
        book(&store);
        let files: Vec<String> = (0..5).map(|_| file(&store)).collect();
        for (i, spec) in specs.iter().enumerate() {
            let owned: Vec<String> = spec.files.iter().map(|&f| files[f].clone()).collect();
            let book = if spec.trashed { "trash" } else { "book:b" };
            note(&store, &format!("note:n{i}"), book, spec.locked, &owned);
        }
        let owned_by = |wanted_trashed: bool, f: usize| {
            specs.iter().any(|s| s.trashed == wanted_trashed && s.files.contains(&f))
        };
        let trashed = specs.iter().filter(|s| s.trashed).count() as u32;
        let locked = specs.iter().filter(|s| s.trashed && s.locked).count() as u32;
        prop_assert_eq!(store.trash_counts().unwrap(), TrashCounts { notes: trashed, locked });

        let done = store.empty_trash(Origin::Ui).unwrap();

        let doomed = (0..5).filter(|&f| owned_by(true, f) && !owned_by(false, f)).count() as u32;
        prop_assert_eq!(done, Emptied { notes: trashed, locked, files: doomed });
        prop_assert_eq!(store.trash_counts().unwrap(), TrashCounts::default());
        for (i, spec) in specs.iter().enumerate() {
            let present = store.note(&format!("note:n{i}")).unwrap().is_some();
            prop_assert_eq!(present, !spec.trashed);
        }
        for (f, id) in files.iter().enumerate() {
            let present = store.file_meta(id).unwrap().is_some();
            prop_assert_eq!(present, !(owned_by(true, f) && !owned_by(false, f)));
        }
    }
}
