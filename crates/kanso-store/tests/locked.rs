//! Locked notes: the store keeps only the sealed text (`kansoLock`), and
//! locking a note forgets what it said before, on disk too.

use kanso_core::{DocType, Query};
use kanso_store::{Origin, Put, SearchParams, Store, StoreError};
use serde_json::{Value, json};

const CANARY: &str = "zebracanary4471";

fn store() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    (dir, store)
}

/// A note with the canary in its text, edited once so it has history.
fn note_with_history(store: &Store) -> Value {
    let book = store
        .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
        .unwrap()
        .id;
    let body = format!("# Plans\n\n{CANARY} - [ ] call [[Sam]] @2026-10-25");
    let first = store
        .put(
            DocType::Note,
            json!({"bookId": book, "title": "Plans", "body": body}),
            Put::Upsert,
            Origin::Api,
        )
        .unwrap();
    let second = store
        .put(
            DocType::Note,
            json!({"_id": first.id, "_rev": first.rev, "bookId": book, "title": "Plans",
                   "body": format!("{body} and more")}),
            Put::Upsert,
            Origin::Api,
        )
        .unwrap();
    assert!(!store.revisions(&first.id).unwrap().is_empty());
    let mut note = store.get_api(&first.id, None, false).unwrap();
    note["_rev"] = second.rev.into();
    note
}

fn locked(mut note: Value) -> Value {
    note["body"] = "".into();
    note["kansoLock"] =
        json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "sealed"});
    note
}

fn search(store: &Store, text: &str) -> usize {
    let params = SearchParams {
        query: Query::parse(text),
        ..SearchParams::default()
    };
    store.search(&params).unwrap().len()
}

fn files_contain(dir: &std::path::Path, needle: &str) -> Vec<String> {
    std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|entry| {
            let path = entry.unwrap().path();
            let bytes = std::fs::read(&path).ok()?;
            bytes
                .windows(needle.len())
                .any(|w| w == needle.as_bytes())
                .then(|| path.display().to_string())
        })
        .collect()
}

#[test]
fn locking_forgets_the_text_in_search_links_tasks_history_and_on_disk() {
    let (dir, store) = store();
    let note = note_with_history(&store);
    let id = note["_id"].as_str().unwrap().to_owned();
    assert_eq!(search(&store, CANARY), 1);
    assert!(!files_contain(dir.path(), CANARY).is_empty());

    store.lock_note(locked(note)).unwrap();

    assert_eq!(search(&store, CANARY), 0);
    // The title stays: it shows in the list and wiki-links reach it.
    assert_eq!(search(&store, "title:Plans"), 1);
    assert!(store.revisions(&id).unwrap().is_empty());
    assert_eq!(store.derived_rows(&id).unwrap(), (0, 0, 0));
    let summaries = store.search_summaries(&SearchParams::default()).unwrap();
    assert!(summaries[0].locked);
    assert_eq!(summaries[0].snippet, "");
    // Nothing of the old text is left in the database or its logs.
    assert_eq!(files_contain(dir.path(), CANARY), Vec::<String>::new());
}

#[test]
fn only_a_locked_note_can_be_written_by_lock_note() {
    let (_dir, store) = store();
    let note = note_with_history(&store);
    assert!(matches!(store.lock_note(note), Err(StoreError::Invalid(_))));
}

#[test]
fn local_writes_cant_put_text_beside_the_lock_but_sync_can() {
    let (_dir, store) = store();
    let note = note_with_history(&store);
    let id = note["_id"].as_str().unwrap().to_owned();
    let rev = store.lock_note(locked(note)).unwrap().rev;

    for origin in [Origin::Ui, Origin::Api, Origin::Import] {
        let result = store.put(
            DocType::Note,
            json!({"_id": id, "_rev": rev, "body": "plain text"}),
            Put::Merge,
            origin,
        );
        assert!(matches!(result, Err(StoreError::Invalid(_))), "{origin:?}");
    }
    // Moving it, retitling it or tagging it is fine.
    store
        .put(
            DocType::Note,
            json!({"_id": id, "title": "Later"}),
            Put::Merge,
            Origin::Api,
        )
        .unwrap();
    // A plain-text version can't be slipped into its history either.
    let fields = json!({"title": "Later", "body": "plain text"});
    assert!(matches!(
        store.record_revision(&id, &rev, fields.as_object().unwrap(), Origin::Ui),
        Err(StoreError::Invalid(_))
    ));
    // Sync takes what another device wrote, whatever it is.
    store
        .put(
            DocType::Note,
            json!({"_id": id, "body": "from an older Kanso"}),
            Put::Merge,
            Origin::Sync,
        )
        .unwrap();
}

/// The note as another device sealed it: `salt` names the key, `data` the
/// sealed text.
fn sealed_by(note: &Value, rev: &str, salt: &str, data: &str) -> Value {
    let mut note = note.clone();
    note["_rev"] = rev.into();
    note["body"] = "".into();
    note["kansoLock"] = json!({"v": 1, "salt": salt, "m": 64, "t": 1, "p": 1, "data": data});
    note
}

#[test]
fn a_lock_arriving_by_sync_forgets_the_plain_text_here_too() {
    let (dir, store) = store();
    let note = note_with_history(&store);
    let id = note["_id"].as_str().unwrap().to_owned();
    let rev = note["_rev"].as_str().unwrap().to_owned();
    assert!(!store.revisions(&id).unwrap().is_empty());

    store
        .put(
            DocType::Note,
            sealed_by(&note, &rev, "c2FsdA", "sealed"),
            Put::Upsert,
            Origin::Sync,
        )
        .unwrap();

    // Locked on another device: no plain version stays in this one's history,
    // search or files.
    assert!(store.revisions(&id).unwrap().is_empty());
    assert_eq!(search(&store, CANARY), 0);
    assert_eq!(files_contain(dir.path(), CANARY), Vec::<String>::new());
}

#[test]
fn a_new_lock_password_forgets_versions_sealed_under_the_old_one() {
    let (_dir, store) = store();
    let note = note_with_history(&store);
    let id = note["_id"].as_str().unwrap().to_owned();
    let rev = store.lock_note(locked(note.clone())).unwrap().rev;
    // Edits under the same key keep their sealed versions.
    let rev = store
        .put(
            DocType::Note,
            sealed_by(&note, &rev, "c2FsdA", "edited"),
            Put::Upsert,
            Origin::Sync,
        )
        .unwrap()
        .rev;
    assert_eq!(store.revisions(&id).unwrap().len(), 1);
    // The password changed on another device: the old key's versions go.
    store
        .put(
            DocType::Note,
            sealed_by(&note, &rev, "bmV3c2FsdA", "rekeyed"),
            Put::Upsert,
            Origin::Sync,
        )
        .unwrap();
    assert!(store.revisions(&id).unwrap().is_empty());
}

#[test]
fn text_beside_a_lock_is_never_searched_linked_or_served() {
    let (_dir, store) = store();
    let note = note_with_history(&store);
    let id = note["_id"].as_str().unwrap().to_owned();
    store.lock_note(locked(note)).unwrap();
    // An older Kanso typed into the note it showed as empty.
    store
        .put(
            DocType::Note,
            json!({"_id": id, "body": format!("{CANARY} - [ ] call [[Sam]]")}),
            Put::Merge,
            Origin::Sync,
        )
        .unwrap();

    assert_eq!(search(&store, CANARY), 0);
    assert_eq!(store.derived_rows(&id).unwrap(), (0, 0, 0));
    let summaries = store.search_summaries(&SearchParams::default()).unwrap();
    assert_eq!(summaries[0].snippet, "");
    let served = store.get_api(&id, None, false).unwrap();
    assert_eq!(served["body"], "");
    // What it typed is kept as it came, for the app to deal with.
    assert!(store.note(&id).unwrap().unwrap().body.contains(CANARY));
}

#[test]
fn the_api_never_serves_the_sealed_text() {
    let (_dir, store) = store();
    let note = note_with_history(&store);
    let id = note["_id"].as_str().unwrap().to_owned();
    store.lock_note(locked(note)).unwrap();
    let served = store.get_api(&id, None, false).unwrap();
    assert!(served.get("kansoLock").is_none());
    assert_eq!(served["body"], "");
    assert_eq!(served["title"], "Plans");
    for listed in store.list_api(DocType::Note, None, 0).unwrap() {
        assert!(listed.get("kansoLock").is_none());
    }
}

// MARK: Locked notebooks

/// A notebook's lock as the app sets it: an envelope sealing nothing.
fn book_lock() -> Value {
    json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "notebook"})
}

fn book(store: &Store, name: &str, parent: Option<&str>) -> String {
    store
        .put(
            DocType::Book,
            json!({"name": name, "parentBookId": parent}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id
}

/// A note in `book` with the canary in its text, edited once so it has
/// history.
fn plain_note(store: &Store, book: &str, title: &str) -> String {
    let body = format!("# {title}\n\n{CANARY} - [ ] call [[Sam]]");
    let first = store
        .put(
            DocType::Note,
            json!({"bookId": book, "title": title, "body": body}),
            Put::Upsert,
            Origin::Api,
        )
        .unwrap();
    store
        .put(
            DocType::Note,
            json!({"_id": first.id, "body": format!("{body} and more")}),
            Put::Merge,
            Origin::Api,
        )
        .unwrap();
    assert!(!store.revisions(&first.id).unwrap().is_empty());
    first.id
}

/// The note as the app seals it.
fn sealed(store: &Store, id: &str) -> Value {
    locked(store.get_api(id, None, false).unwrap())
}

/// The notebook with its lock set, as the app writes it.
fn with_lock(store: &Store, id: &str) -> Value {
    let mut book = store.get_api(id, None, false).unwrap();
    book.as_object_mut().unwrap().remove("count");
    book["kansoLock"] = book_lock();
    book
}

/// Every way the store gives a note's text out, but the app's own read.
fn served_anywhere(store: &Store, id: &str) -> bool {
    let served = store.get_api(id, None, false).unwrap().to_string();
    let listed = store.list_api(DocType::Note, None, 0).unwrap();
    let searched = store.search_api(&SearchParams::default()).unwrap();
    let changes = store.changes_since(0, None, false).unwrap();
    let summaries = store.search_summaries(&SearchParams::default()).unwrap();
    let this = |doc: &&Value| doc["_id"] == id;
    served.contains(CANARY)
        || listed
            .iter()
            .filter(this)
            .any(|doc| doc.to_string().contains(CANARY))
        || searched
            .iter()
            .filter(this)
            .any(|doc| doc.to_string().contains(CANARY))
        || changes.iter().filter(|doc| doc.id == id).any(|doc| {
            doc.to_served(Default::default())
                .to_string()
                .contains(CANARY)
        })
        || summaries
            .iter()
            .any(|note| note.id == id && note.snippet.contains(CANARY))
}

/// Whether note `id` has a vector that reads can use.
fn embedded(store: &Store, id: &str) -> bool {
    store.related_notes(id, "m", 1).unwrap().is_some()
}

fn embed_everything(store: &Store) {
    store.set_embedding_model(Some("m")).unwrap();
    let queue = store.embedding_queue("m").unwrap();
    let rows: Vec<_> = queue
        .pending
        .into_iter()
        .map(|pending| (pending.id, pending.hash, vec![1.0, 0.0]))
        .collect();
    store.put_embeddings("m", &rows).unwrap();
}

#[test]
fn locking_a_notebook_seals_every_note_in_it_and_in_the_notebooks_inside_it() {
    let (dir, store) = store();
    let outer = book(&store, "Private", None);
    let inner = book(&store, "Deeper", Some(&outer));
    let a = plain_note(&store, &outer, "Plans");
    let b = plain_note(&store, &inner, "More plans");
    embed_everything(&store);
    assert!(embedded(&store, &a));
    assert_eq!(search(&store, CANARY), 2);

    store
        .lock_book(
            with_lock(&store, &outer),
            vec![sealed(&store, &a), sealed(&store, &b)],
        )
        .unwrap();

    assert_eq!(
        store.locked_books().unwrap(),
        [outer.clone(), inner.clone()].into_iter().collect()
    );
    for id in [&a, &b] {
        assert!(store.note_is_locked(id).unwrap());
        assert!(store.revisions(id).unwrap().is_empty());
        assert_eq!(store.derived_rows(id).unwrap(), (0, 0, 0));
        assert!(!served_anywhere(&store, id));
        assert!(!embedded(&store, id), "its embedding is forgotten");
    }
    assert_eq!(search(&store, CANARY), 0);
    assert_eq!(search(&store, "title:plans"), 2);
    let served = store.get_api(&outer, None, false).unwrap();
    assert!(
        served.get("kansoLock").is_none(),
        "the API never sees its lock"
    );
    assert!(
        store
            .list_api(DocType::Book, None, 0)
            .unwrap()
            .iter()
            .all(|book| book.get("kansoLock").is_none())
    );
    assert!(store.unsealed_in_locked_books().unwrap().is_empty());
    assert_eq!(files_contain(dir.path(), CANARY), Vec::<String>::new());
}

#[test]
fn a_notebook_wont_lock_around_a_note_left_with_its_text() {
    let (_dir, store) = store();
    let outer = book(&store, "Private", None);
    let a = plain_note(&store, &outer, "Plans");
    let b = plain_note(&store, &outer, "Other");
    let result = store.lock_book(with_lock(&store, &outer), vec![sealed(&store, &a)]);
    assert!(matches!(result, Err(StoreError::Invalid(_))));
    // Nothing of it was written.
    assert!(store.locked_books().unwrap().is_empty());
    assert!(!store.note_is_locked(&a).unwrap());
    assert!(!store.note_is_locked(&b).unwrap());
    // An empty note is fine: it has nothing to seal.
    store
        .put(
            DocType::Note,
            json!({"_id": b, "body": ""}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    store
        .lock_book(with_lock(&store, &outer), vec![sealed(&store, &a)])
        .unwrap();
    assert!(store.note_is_locked(&b).unwrap());
    assert_eq!(store.unsealed_in_locked_books().unwrap().len(), 1);
}

/// A note in a locked notebook that isn't sealed yet, as one written by
/// another device reads before the app has the key.
fn unsealed_in_locked_notebook(store: &Store) -> (String, String, String) {
    let outer = book(store, "Private", None);
    let inner = book(store, "Deeper", Some(&outer));
    let a = plain_note(store, &inner, "Plans");
    store
        .put(
            DocType::Book,
            with_lock(store, &outer),
            Put::Upsert,
            Origin::Sync,
        )
        .unwrap();
    assert!(store.note_is_locked(&a).unwrap());
    (outer, inner, a)
}

#[test]
fn a_locked_notebooks_text_never_leaves_by_a_template_or_daily_note() {
    let (_dir, store) = store();
    let (_, _, a) = unsealed_in_locked_notebook(&store);
    let now = kanso_core::dates::DateTime {
        date: kanso_core::dates::Date::new(2026, 10, 21).unwrap(),
        hour: 9,
        minute: 0,
        second: 0,
    };
    let copy = store
        .create_from_template(&a, None, None, now, Origin::Api)
        .unwrap();
    assert!(!store.note(&copy.id).unwrap().unwrap().body.contains(CANARY));

    store.set_daily_template(Some(&a), Origin::Api).unwrap();
    let daily = store.open_daily(now, Origin::Api).unwrap();
    assert!(
        !store
            .note(&daily.id)
            .unwrap()
            .unwrap()
            .body
            .contains(CANARY)
    );
}

#[test]
fn the_api_cant_take_a_notebook_out_of_a_locked_one() {
    let (_dir, store) = store();
    let (outer, inner, a) = unsealed_in_locked_notebook(&store);
    let mut moved = store.get_api(&inner, None, false).unwrap();
    moved.as_object_mut().unwrap().remove("count");
    moved["parentBookId"] = Value::Null;
    assert!(matches!(
        store.put(DocType::Book, moved.clone(), Put::Upsert, Origin::Api),
        Err(StoreError::Invalid(_))
    ));
    assert!(store.note_is_locked(&a).unwrap());
    assert!(store.locked_books().unwrap().contains(&inner));
    // Another device moving it is a change the lock already allowed there.
    store
        .put(DocType::Book, moved, Put::Upsert, Origin::Sync)
        .unwrap();
    assert!(!store.locked_books().unwrap().contains(&inner));
    assert!(store.locked_books().unwrap().contains(&outer));
}

#[test]
fn meaning_rows_of_a_locked_notebook_show_no_text() {
    let (_dir, store) = store();
    let (_, _, a) = unsealed_in_locked_notebook(&store);
    let rows = store.summaries_of(&[a.as_str()]).unwrap();
    assert_eq!(rows.len(), 1);
    assert!(rows[0].locked);
    assert!(!rows[0].snippet.contains(CANARY));
}

#[test]
fn an_embedding_made_before_a_lock_is_not_kept_after_it() {
    let (_dir, store) = store();
    let outer = book(&store, "Private", None);
    let a = plain_note(&store, &outer, "Plans");
    store.set_embedding_model(Some("m")).unwrap();
    // The worker read the note's words, then the notebook was locked.
    let stale: Vec<_> = store
        .embedding_queue("m")
        .unwrap()
        .pending
        .into_iter()
        .map(|pending| (pending.id, pending.hash, vec![1.0, 0.0]))
        .collect();
    assert!(stale.iter().any(|(id, _, _)| *id == a));
    store
        .put(
            DocType::Book,
            with_lock(&store, &outer),
            Put::Upsert,
            Origin::Sync,
        )
        .unwrap();
    store.put_embeddings("m", &stale).unwrap();
    assert_eq!(store.embedded_count("m").unwrap(), 0);
    // A locked note is not embedded again, not even by its title.
    embed_everything(&store);
    assert_eq!(store.embedded_count("m").unwrap(), 0);
    assert_eq!(store.embedding_queue("m").unwrap().total, 0);
}

#[test]
fn a_notebook_locked_by_sync_hides_the_text_of_notes_not_sealed_here_yet() {
    let (_dir, store) = store();
    let outer = book(&store, "Private", None);
    let inner = book(&store, "Deeper", Some(&outer));
    let a = plain_note(&store, &inner, "Plans");
    embed_everything(&store);

    // Locked on another device; this note was written here, offline.
    store
        .put(
            DocType::Book,
            with_lock(&store, &outer),
            Put::Upsert,
            Origin::Sync,
        )
        .unwrap();

    assert!(store.note_is_locked(&a).unwrap());
    assert!(!served_anywhere(&store, &a));
    assert_eq!(search(&store, CANARY), 0);
    assert!(store.revisions(&a).unwrap().is_empty());
    assert_eq!(store.derived_rows(&a).unwrap(), (0, 0, 0));
    assert!(
        !store
            .embedding_queue("m")
            .unwrap()
            .pending
            .iter()
            .any(|pending| pending.id == a)
    );
    assert_eq!(store.embedded_count("m").unwrap(), 0);
    assert!(
        !store
            .embedding_queue("m")
            .unwrap()
            .pending
            .iter()
            .any(|pending| pending.text.contains(CANARY))
    );
    // Its text is kept for the app to seal once it holds the key.
    let pending = store.unsealed_in_locked_books().unwrap();
    assert_eq!(pending.len(), 1);
    assert!(pending[0].body.contains(CANARY));

    // The lock taken off elsewhere: the note reads as it is again.
    let mut unlocked = store.get_api(&outer, None, false).unwrap();
    unlocked.as_object_mut().unwrap().remove("count");
    store
        .put(DocType::Book, unlocked, Put::Upsert, Origin::Sync)
        .unwrap();
    assert!(!store.note_is_locked(&a).unwrap());
    assert_eq!(search(&store, CANARY), 1);
}

#[test]
fn plain_text_comes_into_a_locked_notebook_only_by_sync_and_never_leaves_but_sealed() {
    let (_dir, store) = store();
    let outer = book(&store, "Private", None);
    let open = book(&store, "Open", None);
    let loose = book(&store, "Loose", None);
    let kept = plain_note(&store, &open, "Plans");
    let lost = plain_note(&store, &loose, "Notes");
    store
        .lock_book(with_lock(&store, &outer), Vec::new())
        .unwrap();

    for origin in [Origin::Ui, Origin::Api, Origin::Import] {
        // No new note with text in it, and no note moved in with its text.
        let created = store.put(
            DocType::Note,
            json!({"bookId": outer, "title": "New", "body": CANARY}),
            Put::Upsert,
            origin,
        );
        assert!(matches!(created, Err(StoreError::Invalid(_))), "{origin:?}");
        let moved = store.put(
            DocType::Note,
            json!({"_id": kept, "bookId": outer}),
            Put::Merge,
            origin,
        );
        assert!(matches!(moved, Err(StoreError::Invalid(_))), "{origin:?}");
        // A notebook with notes that have text doesn't move into it either.
        let nested = store.put(
            DocType::Book,
            json!({"_id": open, "parentBookId": outer}),
            Put::Merge,
            origin,
        );
        assert!(matches!(nested, Err(StoreError::Invalid(_))), "{origin:?}");
    }
    // An empty note may be made there: the app seals it.
    let empty = store
        .put(
            DocType::Note,
            json!({"bookId": outer, "title": "New", "body": ""}),
            Put::Upsert,
            Origin::Api,
        )
        .unwrap()
        .id;
    assert!(store.note_is_locked(&empty).unwrap());

    // Another device moves a plain note in; here, only Kanso's seal moves it
    // on or changes its text, and nothing keeps a plain version of it.
    store
        .put(
            DocType::Note,
            json!({"_id": lost, "bookId": outer}),
            Put::Merge,
            Origin::Sync,
        )
        .unwrap();
    for (change, origin) in [
        (json!({"_id": lost, "bookId": open}), Origin::Ui),
        (json!({"_id": lost, "bookId": "trash"}), Origin::Api),
        (json!({"_id": lost, "body": "replaced"}), Origin::Api),
        (json!({"_id": lost, "body": ""}), Origin::Api),
    ] {
        let result = store.put(DocType::Note, change, Put::Merge, origin);
        assert!(matches!(result, Err(StoreError::Invalid(_))));
    }
    assert!(matches!(
        store.delete(&outer, Origin::Ui),
        Err(StoreError::Invalid(_))
    ));
    let fields = json!({"title": "Notes", "body": CANARY});
    assert!(matches!(
        store.record_revision(&lost, "1-x", fields.as_object().unwrap(), Origin::Ui),
        Err(StoreError::Invalid(_))
    ));
    // Retitling or tagging it leaves the text as it was, so that's fine.
    store
        .put(
            DocType::Note,
            json!({"_id": lost, "title": "Renamed"}),
            Put::Merge,
            Origin::Api,
        )
        .unwrap();
    assert!(store.revisions(&lost).unwrap().is_empty());
    assert!(!served_anywhere(&store, &lost));

    // Sealed, it moves anywhere and stays sealed.
    store.lock_note(sealed(&store, &lost)).unwrap();
    store
        .put(
            DocType::Note,
            json!({"_id": lost, "bookId": open}),
            Put::Merge,
            Origin::Api,
        )
        .unwrap();
    assert!(store.note_is_locked(&lost).unwrap());
    assert!(store.note(&lost).unwrap().unwrap().is_locked());
}

#[test]
fn a_note_locked_on_its_own_keeps_its_own_seal_in_a_locked_notebook() {
    let (_dir, store) = store();
    let outer = book(&store, "Private", None);
    let a = plain_note(&store, &outer, "Plans");
    let mut own = sealed(&store, &a);
    own["kansoLock"]["salt"] = "b3duc2FsdA".into();
    let rev = store.lock_note(own).unwrap().rev;
    store
        .lock_book(with_lock(&store, &outer), Vec::new())
        .unwrap();
    let note = store.note(&a).unwrap().unwrap();
    assert_eq!(note.rev.as_deref(), Some(rev.as_str()));
    assert_eq!(note.extra["kansoLock"]["salt"], "b3duc2FsdA");
    // The notebook's lock taken off, the note stays locked.
    let mut book = store.get_api(&outer, None, false).unwrap();
    book.as_object_mut().unwrap().remove("count");
    store
        .put(DocType::Book, book, Put::Upsert, Origin::Ui)
        .unwrap();
    assert!(store.locked_books().unwrap().is_empty());
    assert!(store.note_is_locked(&a).unwrap());
}

// MARK: Model-based

mod model {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    /// `b1` is inside `b0`, `b2` inside `b1`; `b3` stands alone and sync
    /// moves it around.
    const BOOKS: [&str; 4] = ["book:b0", "book:b1", "book:b2", "book:b3"];

    #[derive(Clone, Copy, Debug)]
    enum By {
        Api,
        Ui,
        Sync,
    }

    impl By {
        fn origin(self) -> Origin {
            match self {
                By::Api => Origin::Api,
                By::Ui => Origin::Ui,
                By::Sync => Origin::Sync,
            }
        }
    }

    #[derive(Clone, Debug)]
    enum Op {
        Create {
            book: usize,
            text: bool,
            by: By,
        },
        Edit {
            note: usize,
            text: bool,
            by: By,
        },
        Move {
            note: usize,
            book: Option<usize>,
            by: By,
        },
        /// The app locks a notebook, sealing what's in it.
        LockBook {
            book: usize,
        },
        /// Another device's lock arrives.
        SyncLockBook {
            book: usize,
        },
        UnlockBook {
            book: usize,
            by: By,
        },
        /// Another device moves `b3` under another notebook, or to the top.
        SyncMoveB3 {
            parent: Option<usize>,
        },
        /// The app seals what it finds unsealed in locked notebooks.
        SealPending,
        LockNote {
            note: usize,
        },
    }

    fn by() -> impl Strategy<Value = By> {
        prop_oneof![Just(By::Api), Just(By::Ui), Just(By::Sync)]
    }

    fn op() -> impl Strategy<Value = Op> {
        prop_oneof![
            3 => (0..4usize, any::<bool>(), by()).prop_map(|(book, text, by)| Op::Create { book, text, by }),
            2 => (0..8usize, any::<bool>(), by()).prop_map(|(note, text, by)| Op::Edit { note, text, by }),
            2 => (0..8usize, prop::option::weighted(0.85, 0..4usize), by())
                .prop_map(|(note, book, by)| Op::Move { note, book, by }),
            2 => (0..4usize).prop_map(|book| Op::LockBook { book }),
            1 => (0..4usize).prop_map(|book| Op::SyncLockBook { book }),
            1 => (0..4usize, by()).prop_map(|(book, by)| Op::UnlockBook { book, by }),
            1 => prop::option::of(prop_oneof![Just(0usize), Just(2usize)])
                .prop_map(|parent| Op::SyncMoveB3 { parent }),
            1 => Just(Op::SealPending),
            1 => (0..8usize).prop_map(|note| Op::LockNote { note }),
        ]
    }

    fn text(n: usize) -> String {
        format!("secret{n:03}words - [ ] call [[Sam]] @2026-10-25")
    }

    fn canary(n: usize) -> String {
        format!("secret{n:03}words")
    }

    fn book_doc(store: &Store, id: &str) -> Value {
        let mut book = store.get_api(id, None, false).unwrap();
        book.as_object_mut().unwrap().remove("count");
        // The served copy leaves the lock out; the app's own read has it.
        let own = store.get(id).unwrap().unwrap();
        if let Some(lock) = own.fields.get("kansoLock") {
            book["kansoLock"] = lock.clone();
        }
        book
    }

    /// What the app seals: the note with its text gone into the lock.
    fn seal(store: &Store, id: &str) -> Value {
        let note = store.note(id).unwrap().unwrap();
        let mut value = serde_json::to_value(&note).unwrap();
        value["body"] = "".into();
        value["kansoLock"] =
            json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "sealed"});
        value
    }

    /// The notebooks inside `book`, and itself.
    fn subtree(store: &Store, book: &str) -> Vec<String> {
        let books = store.books().unwrap();
        let mut found = vec![book.to_owned()];
        let mut i = 0;
        while i < found.len() {
            for b in &books {
                if b.parent_book_id.as_deref() == Some(found[i].as_str()) && !found.contains(&b.id)
                {
                    found.push(b.id.clone());
                }
            }
            i += 1;
        }
        found
    }

    fn apply(store: &Store, notes: &mut Vec<(String, usize)>, op: &Op) {
        let note_id = |n: usize, notes: &Vec<(String, usize)>| {
            (!notes.is_empty()).then(|| notes[n % notes.len()].clone())
        };
        match op {
            Op::Create { book, text: t, by } => {
                let n = notes.len();
                let body = if *t { text(n) } else { String::new() };
                if let Ok(made) = store.put(
                    DocType::Note,
                    json!({"bookId": BOOKS[*book], "title": format!("N{n}"), "body": body}),
                    Put::Upsert,
                    by.origin(),
                ) {
                    notes.push((made.id, n));
                }
            }
            Op::Edit { note, text: t, by } => {
                if let Some((id, n)) = note_id(*note, notes) {
                    let body = if *t {
                        format!("{} again", text(n))
                    } else {
                        String::new()
                    };
                    let _ = store.put(
                        DocType::Note,
                        json!({"_id": id, "body": body}),
                        Put::Merge,
                        by.origin(),
                    );
                }
            }
            Op::Move { note, book, by } => {
                if let Some((id, _)) = note_id(*note, notes) {
                    let book = book.map_or("trash", |b| BOOKS[b]);
                    let _ = store.put(
                        DocType::Note,
                        json!({"_id": id, "bookId": book}),
                        Put::Merge,
                        by.origin(),
                    );
                }
            }
            Op::LockBook { book } => {
                let id = BOOKS[*book];
                let inside = subtree(store, id);
                let sealed: Vec<Value> = store
                    .search(&SearchParams::default())
                    .unwrap()
                    .into_iter()
                    .filter(|doc| {
                        doc.str_field("bookId")
                            .is_some_and(|b| inside.iter().any(|i| i == b))
                            && !doc.fields.contains_key("kansoLock")
                    })
                    .map(|doc| seal(store, &doc.id))
                    .collect();
                let mut book = book_doc(store, id);
                book["kansoLock"] = book_lock();
                store
                    .lock_book(book, sealed)
                    .expect("the app's lock always goes through");
            }
            Op::SyncLockBook { book } => {
                let mut book = book_doc(store, BOOKS[*book]);
                book["kansoLock"] = book_lock();
                store
                    .put(DocType::Book, book, Put::Upsert, Origin::Sync)
                    .unwrap();
            }
            Op::UnlockBook { book, by } => {
                let mut book = book_doc(store, BOOKS[*book]);
                book.as_object_mut().unwrap().remove("kansoLock");
                let _ = store.put(DocType::Book, book, Put::Upsert, by.origin());
            }
            Op::SyncMoveB3 { parent } => {
                let mut book = book_doc(store, BOOKS[3]);
                book["parentBookId"] = parent.map_or(Value::Null, |p| BOOKS[p].into());
                store
                    .put(DocType::Book, book, Put::Upsert, Origin::Sync)
                    .unwrap();
            }
            Op::SealPending => {
                for note in store.unsealed_in_locked_books().unwrap() {
                    store.lock_note(seal(store, &note.id)).unwrap();
                }
                assert!(store.unsealed_in_locked_books().unwrap().is_empty());
            }
            Op::LockNote { note } => {
                if let Some((id, _)) = note_id(*note, notes)
                    && !store.note(&id).unwrap().unwrap().is_locked()
                {
                    store.lock_note(seal(store, &id)).unwrap();
                }
            }
        }
    }

    /// Nothing but the app's own read of the note gives out the text of a
    /// locked note or of a note in a locked notebook.
    fn check(store: &Store, notes: &[(String, usize)]) -> Result<(), TestCaseError> {
        let queue = store.embedding_queue("m").unwrap();
        for (id, n) in notes {
            if !store.note_is_locked(id).unwrap() {
                continue;
            }
            let canary = canary(*n);
            let served = store.get_api(id, None, false).unwrap();
            prop_assert_eq!(&served["body"], "");
            prop_assert!(served.get("kansoLock").is_none());
            let everywhere = store
                .list_api(DocType::Note, None, 0)
                .unwrap()
                .into_iter()
                .chain(store.search_api(&SearchParams::default()).unwrap())
                .chain(
                    store
                        .changes_since(0, None, false)
                        .unwrap()
                        .into_iter()
                        .map(|doc| doc.to_served(Default::default())),
                )
                .any(|doc| doc["_id"] == id.as_str() && doc.to_string().contains(&canary));
            prop_assert!(!everywhere, "served: {id}");
            prop_assert!(
                !store
                    .search(&SearchParams {
                        query: Query::parse(&canary),
                        ..SearchParams::default()
                    })
                    .unwrap()
                    .iter()
                    .any(|doc| &doc.id == id),
                "searched: {id}"
            );
            let summary = store
                .search_summaries(&SearchParams::default())
                .unwrap()
                .into_iter()
                .find(|note| &note.id == id);
            if let Some(summary) = summary {
                prop_assert!(summary.locked && summary.snippet.is_empty());
            }
            prop_assert_eq!(store.derived_rows(id).unwrap(), (0, 0, 0));
            for revision in store.revisions(id).unwrap() {
                let plain = revision
                    .fields
                    .get("body")
                    .and_then(Value::as_str)
                    .is_some_and(|b| !b.is_empty())
                    && !revision.fields.contains_key("kansoLock");
                prop_assert!(!plain, "a plain version kept: {id}");
            }
            prop_assert!(
                !queue
                    .pending
                    .iter()
                    .any(|pending| &pending.id == id && pending.text.contains(&canary)),
                "embedded: {id}"
            );
        }
        Ok(())
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 64,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        })]

        /// Model-based: no sequence of creating, editing and moving notes
        /// (from the app, the API or sync), locking and unlocking notebooks
        /// here or elsewhere, moving notebooks and sealing ever leaves the
        /// text of a note in a locked notebook (or a locked note) in what
        /// the API, MCP or changes feed serve, in search, in links and
        /// tasks, in history, or in what related notes embed.
        #[test]
        fn prop_a_locked_notebooks_text_never_leaves_the_app(ops in prop::collection::vec(op(), 1..24)) {
            let (_dir, store) = store();
            store.set_embedding_model(Some("m")).unwrap();
            store
                .put(DocType::Book, json!({"_id": BOOKS[0], "name": "B0"}), Put::Upsert, Origin::Ui)
                .unwrap();
            store
                .put(DocType::Book, json!({"_id": BOOKS[1], "name": "B1", "parentBookId": BOOKS[0]}), Put::Upsert, Origin::Ui)
                .unwrap();
            store
                .put(DocType::Book, json!({"_id": BOOKS[2], "name": "B2", "parentBookId": BOOKS[1]}), Put::Upsert, Origin::Ui)
                .unwrap();
            store
                .put(DocType::Book, json!({"_id": BOOKS[3], "name": "B3"}), Put::Upsert, Origin::Ui)
                .unwrap();
            let mut notes = Vec::new();
            for op in &ops {
                apply(&store, &mut notes, op);
                check(&store, &notes)?;
                // Related notes embed whatever they are given to.
                let queue = store.embedding_queue("m").unwrap();
                let rows: Vec<_> = queue
                    .pending
                    .into_iter()
                    .map(|pending| (pending.id, pending.hash, vec![1.0, 0.0]))
                    .collect();
                store.put_embeddings("m", &rows).unwrap();
            }
        }
    }
}
