//! A plugin's writes: field-level merges at the revision it read, one batch
//! in one transaction, one undo, nothing in a locked note or notebook, and
//! each plugin's data on a note kept apart.

use kanso_core::DocType;
use kanso_store::{
    Origin, PluginCreate, PluginEdit, PluginWriter, Put, Store, StoreError, plugin_data,
};
use proptest::prelude::*;
use serde_json::{Value, json};

fn store() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    (dir, store)
}

fn book(store: &Store, name: &str) -> String {
    store
        .put(
            DocType::Book,
            json!({ "name": name }),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id
}

fn note(store: &Store, book: &str, title: &str, body: &str) -> (String, String) {
    let written = store
        .put(
            DocType::Note,
            json!({ "bookId": book, "title": title, "body": body, "kansoOther": 7 }),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
    (written.id, written.rev)
}

const WRITER: PluginWriter = PluginWriter {
    plugin_id: "com.example.wordcount",
    plugin_name: "Word count",
    batch: "plugin:batch-1",
};

fn edit(id: &str, rev: &str, body: &str) -> PluginEdit {
    PluginEdit {
        id: id.into(),
        base_rev: rev.into(),
        body: Some(body.into()),
        ..PluginEdit::default()
    }
}

#[test]
fn a_write_merges_the_fields_it_names_and_keeps_the_rest() {
    let (_dir, store) = store();
    let work = book(&store, "Work");
    let (id, rev) = note(&store, &work, "Draft", "one");
    let report = store
        .apply_plugin_batch(WRITER, &[edit(&id, &rev, "two")], &[], 1)
        .unwrap();
    assert_eq!(report.changed.len(), 1);
    let note = store.note(&id).unwrap().unwrap();
    assert_eq!(note.body, "two");
    assert_eq!(note.title, "Draft");
    assert_eq!(note.book_id, work);
    assert_eq!(note.extra["kansoOther"], 7);
    // The version it replaced is in history, by the plugin.
    let history = store.revisions(&id).unwrap();
    let by_plugin = history.iter().find(|r| r.origin == "plugin").unwrap();
    assert_eq!(by_plugin.author.as_deref(), Some("Word count"));
    assert_eq!(by_plugin.fields["body"], "one");
}

#[test]
fn a_stale_revision_fails_the_whole_batch() {
    let (_dir, store) = store();
    let work = book(&store, "Work");
    let (a, a_rev) = note(&store, &work, "A", "a");
    let (b, b_rev) = note(&store, &work, "B", "b");
    // B changes elsewhere after the plugin read it.
    store
        .put(
            DocType::Note,
            json!({ "_id": b, "_rev": b_rev, "body": "edited" }),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    let result = store.apply_plugin_batch(
        WRITER,
        &[edit(&a, &a_rev, "a2"), edit(&b, &b_rev, "b2")],
        &[],
        1,
    );
    assert!(matches!(result, Err(StoreError::Conflict)));
    assert_eq!(store.note(&a).unwrap().unwrap().body, "a");
    assert_eq!(store.note(&b).unwrap().unwrap().body, "edited");
}

#[test]
fn undo_puts_every_note_back_and_removes_made_ones() {
    let (_dir, store) = store();
    let work = book(&store, "Work");
    let (a, a_rev) = note(&store, &work, "A", "a");
    let (b, b_rev) = note(&store, &work, "B", "b");
    let report = store
        .apply_plugin_batch(
            WRITER,
            &[edit(&a, &a_rev, "a2"), edit(&b, &b_rev, "b2")],
            &[PluginCreate {
                book_id: work.clone(),
                title: "Made".into(),
                body: "new".into(),
            }],
            1,
        )
        .unwrap();
    let made = report.created[0].id.clone();
    assert!(store.note(&made).unwrap().is_some());
    let undone = store.undo_plugin_batch(WRITER.batch, 2).unwrap();
    assert_eq!(undone.restored.len(), 3);
    assert_eq!(store.note(&a).unwrap().unwrap().body, "a");
    assert_eq!(store.note(&b).unwrap().unwrap().body, "b");
    assert!(store.note(&made).unwrap().is_none());
    // A second undo does nothing.
    assert!(
        store
            .undo_plugin_batch(WRITER.batch, 3)
            .unwrap()
            .restored
            .is_empty()
    );
}

#[test]
fn undo_leaves_a_note_edited_again() {
    let (_dir, store) = store();
    let work = book(&store, "Work");
    let (a, a_rev) = note(&store, &work, "A", "a");
    let report = store
        .apply_plugin_batch(WRITER, &[edit(&a, &a_rev, "a2")], &[], 1)
        .unwrap();
    store
        .put(
            DocType::Note,
            json!({ "_id": a, "_rev": report.changed[0].rev, "body": "mine" }),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    let undone = store.undo_plugin_batch(WRITER.batch, 2).unwrap();
    assert!(undone.restored.is_empty());
    assert_eq!(undone.skipped.len(), 1);
    assert_eq!(store.note(&a).unwrap().unwrap().body, "mine");
}

fn sealed(store: &Store, id: &str) -> Value {
    let mut note = store.get_api(id, None, false).unwrap();
    note["body"] = "".into();
    note["kansoLock"] = json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "x"});
    note
}

#[test]
fn locked_notes_and_notebooks_refuse_every_plugin_write() {
    let (_dir, store) = store();
    let work = book(&store, "Work");
    let (sealed_id, _) = note(&store, &work, "Secret", "s");
    let locked = store.lock_note(sealed(&store, &sealed_id)).unwrap();
    let result = store.apply_plugin_batch(
        WRITER,
        &[PluginEdit {
            id: sealed_id.clone(),
            base_rev: locked.rev,
            title: Some("renamed".into()),
            ..PluginEdit::default()
        }],
        &[],
        1,
    );
    assert!(matches!(result, Err(StoreError::Invalid(_))));

    let vault = book(&store, "Vault");
    let (inside, _) = note(&store, &vault, "In", "i");
    let mut lock = store.get_api(&vault, None, false).unwrap();
    lock.as_object_mut().unwrap().remove("count");
    lock["kansoLock"] = json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "b"});
    store
        .lock_book(lock, vec![sealed(&store, &inside)])
        .unwrap();
    let rev = store.get(&inside).unwrap().unwrap().rev;
    let result = store.apply_plugin_batch(WRITER, &[edit(&inside, &rev, "leak")], &[], 1);
    assert!(matches!(result, Err(StoreError::Invalid(_))));
    let made = store.apply_plugin_batch(
        WRITER,
        &[],
        &[PluginCreate {
            book_id: vault.clone(),
            title: "t".into(),
            body: "b".into(),
        }],
        1,
    );
    assert!(made.is_err());
    let trash = store.apply_plugin_batch(
        WRITER,
        &[],
        &[PluginCreate {
            book_id: "trash".into(),
            title: "t".into(),
            body: "b".into(),
        }],
        1,
    );
    assert!(trash.is_err());
}

#[test]
fn each_plugins_data_is_its_own_and_removal_sweeps_only_it() {
    let (_dir, store) = store();
    let work = book(&store, "Work");
    let (id, rev) = note(&store, &work, "A", "a");
    let other = PluginWriter {
        plugin_id: "com.example.other",
        plugin_name: "Other",
        batch: "plugin:batch-2",
    };
    let first = store
        .apply_plugin_batch(
            WRITER,
            &[PluginEdit {
                id: id.clone(),
                base_rev: rev,
                data: Some(Some(json!({ "count": 1 }))),
                ..PluginEdit::default()
            }],
            &[],
            1,
        )
        .unwrap();
    store
        .apply_plugin_batch(
            other,
            &[PluginEdit {
                id: id.clone(),
                base_rev: first.changed[0].rev.clone(),
                data: Some(Some(json!("theirs"))),
                ..PluginEdit::default()
            }],
            &[],
            2,
        )
        .unwrap();
    let doc = store.get(&id).unwrap().unwrap();
    let fields = doc.fields;
    assert_eq!(
        plugin_data(&fields, WRITER.plugin_id),
        Some(&json!({ "count": 1 }))
    );
    assert_eq!(
        plugin_data(&fields, other.plugin_id),
        Some(&json!("theirs"))
    );
    assert_eq!(store.remove_plugin_data(WRITER.plugin_id).unwrap(), 1);
    let fields = store.get(&id).unwrap().unwrap().fields;
    assert_eq!(plugin_data(&fields, WRITER.plugin_id), None);
    assert_eq!(
        plugin_data(&fields, other.plugin_id),
        Some(&json!("theirs"))
    );
    assert_eq!(store.remove_plugin_data(WRITER.plugin_id).unwrap(), 0);
}

#[derive(Clone, Debug)]
struct Change {
    note: usize,
    title: Option<String>,
    body: Option<String>,
    status: Option<&'static str>,
    data: Option<Option<i64>>,
}

fn change() -> impl Strategy<Value = Change> {
    (
        0..4usize,
        prop::option::of("[a-z ]{0,12}"),
        prop::option::of("[a-z\n ]{0,40}"),
        prop::option::of(prop::sample::select(vec![
            "none",
            "active",
            "onHold",
            "completed",
            "dropped",
        ])),
        prop::option::of(prop::option::of(any::<i64>())),
    )
        .prop_map(|(note, title, body, status, data)| Change {
            note,
            title,
            body,
            status,
            data,
        })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// Invariant: whatever a batch changes, undoing it gives every note back
    /// its title, text, status and plugin data exactly, and the fields a
    /// plugin may not name never change.
    #[test]
    fn prop_undo_is_exact(changes in prop::collection::vec(change(), 1..6)) {
        let (_dir, store) = store();
        let work = book(&store, "Work");
        let notes: Vec<(String, String)> =
            (0..4).map(|n| note(&store, &work, &format!("N{n}"), "start")).collect();
        let before: Vec<Value> = notes.iter().map(|(id, _)| Value::Object(store.get(id).unwrap().unwrap().fields)).collect();
        // One edit per note: the last change named for it.
        let mut edits: Vec<PluginEdit> = Vec::new();
        for change in &changes {
            let (id, rev) = &notes[change.note];
            edits.retain(|e| &e.id != id);
            edits.push(PluginEdit {
                id: id.clone(),
                base_rev: rev.clone(),
                title: change.title.clone(),
                body: change.body.clone(),
                tags: None,
                status: change.status.map(str::to_owned),
                data: change.data.map(|d| d.map(Value::from)),
                book_id: None,
            });
        }
        store.apply_plugin_batch(WRITER, &edits, &[], 10).unwrap();
        for (id, _) in &notes {
            let now = store.get(id).unwrap().unwrap().fields;
            prop_assert_eq!(&now["bookId"], &Value::from(work.clone()));
            prop_assert_eq!(&now["kansoOther"], &Value::from(7));
        }
        store.undo_plugin_batch(WRITER.batch, 20).unwrap();
        for ((id, _), before) in notes.iter().zip(&before) {
            let now = store.get(id).unwrap().unwrap().fields;
            for key in ["title", "body", "status", "tags", "kansoPlugin", "bookId", "kansoOther"] {
                prop_assert_eq!(now.get(key), before.get(key), "{} of {}", key, id);
            }
        }
    }

    /// Invariant: a batch with any stale revision changes nothing at all.
    #[test]
    fn prop_a_conflict_writes_nothing(stale in 0..3usize, bodies in prop::collection::vec("[a-z]{1,8}", 3)) {
        let (_dir, store) = store();
        let work = book(&store, "Work");
        let notes: Vec<(String, String)> =
            (0..3).map(|n| note(&store, &work, &format!("N{n}"), "start")).collect();
        let (id, rev) = &notes[stale];
        store
            .put(DocType::Note, json!({ "_id": id, "_rev": rev, "title": "moved on" }), Put::Merge, Origin::Ui)
            .unwrap();
        let edits: Vec<PluginEdit> = notes
            .iter()
            .zip(&bodies)
            .map(|((id, rev), body)| edit(id, rev, body))
            .collect();
        prop_assert!(matches!(store.apply_plugin_batch(WRITER, &edits, &[], 1), Err(StoreError::Conflict)));
        for (id, _) in &notes {
            prop_assert_eq!(store.note(id).unwrap().unwrap().body, "start");
            prop_assert!(store.revisions(id).unwrap().iter().all(|r| r.origin != "plugin"));
        }
    }
}
