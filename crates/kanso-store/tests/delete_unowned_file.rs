//! Deleting one attachment only when nothing live owns it (issue #38, step
//! 11: "Keep transcript only" must not delete audio another note still
//! links: a copy, a duplicate, a merged note, a canvas card).

use base64::Engine as _;
use kanso_core::DocType;
use kanso_core::model::{KANSO_FILES_FIELD, LOCK_FIELD};
use kanso_store::{FileDrop, Origin, Put, Store};
use proptest::prelude::*;
use serde_json::json;

fn store() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    store
        .put(
            DocType::Book,
            json!({"_id": "book:b", "name": "B"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
    (dir, store)
}

fn audio(store: &Store) -> String {
    let data = base64::engine::general_purpose::STANDARD.encode(b"OggS");
    store
        .put(
            DocType::File,
            json!({"name": "a.opus", "contentType": "audio/ogg",
                   "_attachments": {"index": {"content_type": "audio/ogg", "data": data}}}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id
}

fn note(store: &Store, id: &str, book: &str, body: &str) {
    store
        .put(
            DocType::Note,
            json!({"_id": id, "title": id, "body": body, "bookId": book}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
}

#[test]
fn a_file_no_note_links_is_deleted() {
    let (_dir, store) = store();
    let file = audio(&store);
    note(&store, "note:a", "book:b", "No audio here.");
    assert_eq!(
        store.delete_file_if_unowned(&file, Origin::Ui).unwrap(),
        FileDrop::Deleted
    );
    assert!(store.file_meta(&file).unwrap().is_none());
    assert_eq!(
        store.delete_file_if_unowned(&file, Origin::Ui).unwrap(),
        FileDrop::Gone
    );
}

#[test]
fn a_copy_a_trashed_note_a_locked_note_or_a_canvas_keeps_the_file() {
    /// Makes a note that owns the file.
    type Owner = Box<dyn Fn(&Store, &str)>;
    let (_dir, store) = store();
    let owners: [(&str, Owner); 4] = [
        (
            "copy",
            Box::new(|store, file| {
                note(
                    store,
                    "note:copy",
                    "book:b",
                    &format!("[Audio](inkdrop://{file})"),
                )
            }),
        ),
        (
            "trashed",
            Box::new(|store, file| {
                note(
                    store,
                    "note:old",
                    "trash",
                    &format!("[Audio](inkdrop://{file})"),
                )
            }),
        ),
        (
            "locked",
            Box::new(|store, file| {
                let mut doc =
                    json!({"_id": "note:sealed", "title": "x", "body": "", "bookId": "book:b"});
                doc[LOCK_FIELD] = json!({"v": 1});
                doc[KANSO_FILES_FIELD] = json!([{"file": file, "v": 1,
                    "seal": "AAAAAAAAAAAAAAAAAAAAAA", "key": "wrapped"}]);
                store
                    .put(DocType::Note, doc, Put::Upsert, Origin::Ui)
                    .unwrap();
            }),
        ),
        (
            "canvas",
            Box::new(|store, file| {
                let body = format!(
                    "```canvas\n{{\"nodes\":[{{\"id\":\"a\",\"type\":\"file\",\"file\":\"{file}\",\"x\":0,\"y\":0,\"width\":9,\"height\":9}}]}}\n```\n"
                );
                note(store, "note:board", "book:b", &body);
            }),
        ),
    ];
    for (what, owner) in owners {
        let file = audio(&store);
        owner(&store, &file);
        assert_eq!(
            store.delete_file_if_unowned(&file, Origin::Ui).unwrap(),
            FileDrop::Owned,
            "{what}"
        );
        assert!(store.file_meta(&file).unwrap().is_some(), "{what}");
    }
}

#[test]
fn a_note_is_never_deleted_through_it() {
    let (_dir, store) = store();
    note(&store, "note:a", "book:b", "Words.");
    assert_eq!(
        store.delete_file_if_unowned("note:a", Origin::Ui).unwrap(),
        FileDrop::Gone
    );
    assert!(store.note("note:a").unwrap().is_some());
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 32, rng_seed: proptest::test_runner::RngSeed::Fixed(20261017), ..ProptestConfig::default() })]

    /// A file is deleted exactly when no live note links it, wherever the
    /// linking notes are (a notebook or the trash) and however many there
    /// are.
    #[test]
    fn prop_a_linked_file_is_never_deleted(
        linkers in proptest::collection::vec(proptest::bool::ANY, 0..4),
        deleted in proptest::collection::vec(proptest::bool::ANY, 0..4),
    ) {
        let (_dir, store) = store();
        let file = audio(&store);
        let mut live = 0;
        for (ix, trashed) in linkers.iter().enumerate() {
            let id = format!("note:n{ix}");
            let book = if *trashed { "trash" } else { "book:b" };
            note(&store, &id, book, &format!("[Audio](inkdrop://{file})"));
            if deleted.get(ix).copied().unwrap_or(false) {
                // Deleted for good (Empty Trash, or a delete through the API).
                store.delete(&id, Origin::Ui).unwrap();
            }
            if store.note(&id).unwrap().is_some() {
                live += 1;
            }
        }
        let result = store.delete_file_if_unowned(&file, Origin::Ui).unwrap();
        if live > 0 {
            prop_assert_eq!(result, FileDrop::Owned);
            prop_assert!(store.file_meta(&file).unwrap().is_some());
        } else {
            prop_assert_eq!(result, FileDrop::Deleted);
            prop_assert!(store.file_meta(&file).unwrap().is_none());
        }
    }
}
