//! An agent's approved change set, written through the same batch code as a
//! plugin's: all or nothing, moves, one undo, the agent named in history.

use kanso_core::DocType;
use kanso_store::{
    BatchWriter, Origin, PluginCreate, PluginEdit, Put, Store, StoreError, agent_batch_id,
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
    let w = store
        .put(
            DocType::Note,
            json!({ "bookId": book, "title": title, "body": body }),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
    (w.id, w.rev)
}

fn sealed(store: &Store, id: &str) -> Value {
    let mut note = store.get_api(id, None, false).unwrap();
    note["body"] = "".into();
    note["kansoLock"] = json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "x"});
    note
}

fn edit(id: &str, rev: &str) -> PluginEdit {
    PluginEdit {
        id: id.into(),
        base_rev: rev.into(),
        ..PluginEdit::default()
    }
}

#[test]
fn the_batch_id_says_agent() {
    let a = agent_batch_id();
    assert!(a.starts_with("agent:") && a.len() == 6 + 24);
    assert_ne!(a, agent_batch_id());
}

#[test]
fn an_agent_batch_names_the_agent_in_history() {
    let (_dir, store) = store();
    let work = book(&store, "Work");
    let (id, rev) = note(&store, &work, "Draft", "one");
    let batch = agent_batch_id();
    let writer = BatchWriter::Agent {
        agent: "Claude Agent",
        batch: &batch,
    };
    let mut e = edit(&id, &rev);
    e.body = Some("two".into());
    store.apply_batch(writer, &[e], &[], 5).unwrap();
    let history = store.revisions(&id).unwrap();
    let kept = history.iter().find(|r| r.origin == "agent").unwrap();
    assert_eq!(kept.agent.as_deref(), Some("Claude Agent"));
    assert_eq!(kept.fields["body"], "one");
}

#[test]
fn a_batch_touching_a_locked_note_writes_nothing() {
    let (_dir, store) = store();
    let work = book(&store, "Work");
    let (a, a_rev) = note(&store, &work, "A", "a");
    let (s, _) = note(&store, &work, "S", "s");
    let locked = store.lock_note(sealed(&store, &s)).unwrap();
    let batch = agent_batch_id();
    let writer = BatchWriter::Agent {
        agent: "X",
        batch: &batch,
    };
    let mut ea = edit(&a, &a_rev);
    ea.body = Some("changed".into());
    let mut es = edit(&s, &locked.rev);
    es.book_id = Some("trash".into());
    let result = store.apply_batch(writer, &[ea, es], &[], 5);
    assert!(matches!(result, Err(StoreError::Invalid(_))));
    assert_eq!(store.note(&a).unwrap().unwrap().body, "a");
}

#[test]
fn a_move_and_a_trashed_source_undo_together() {
    let (_dir, store) = store();
    let from = book(&store, "From");
    let to = book(&store, "To");
    let (a, a_rev) = note(&store, &from, "A", "a");
    let (b, b_rev) = note(&store, &from, "B", "b");
    let batch = agent_batch_id();
    let writer = BatchWriter::Agent {
        agent: "X",
        batch: &batch,
    };
    let mut ma = edit(&a, &a_rev);
    ma.book_id = Some(to.clone());
    let mut tb = edit(&b, &b_rev);
    tb.book_id = Some("trash".into());
    let created = PluginCreate {
        book_id: to.clone(),
        title: "Merged".into(),
        body: "ab".into(),
    };
    let report = store.apply_batch(writer, &[ma, tb], &[created], 5).unwrap();
    assert_eq!(store.note(&a).unwrap().unwrap().book_id, to);
    assert_eq!(store.note(&b).unwrap().unwrap().book_id, "trash");
    let new_id = report.created[0].id.clone();
    let undone = store.undo_batch(&batch, 9).unwrap();
    assert!(undone.skipped.is_empty());
    assert_eq!(store.note(&a).unwrap().unwrap().book_id, from);
    assert_eq!(store.note(&b).unwrap().unwrap().book_id, from);
    assert!(store.get(&new_id).unwrap().is_none_or(|d| d.deleted));
    // Undo stamps now, never the old time.
    assert_eq!(store.note(&a).unwrap().unwrap().updated_at, 9);
}

#[test]
fn undo_skips_notes_changed_since() {
    let (_dir, store) = store();
    let work = book(&store, "Work");
    let (a, a_rev) = note(&store, &work, "A", "a");
    let batch = agent_batch_id();
    let writer = BatchWriter::Agent {
        agent: "X",
        batch: &batch,
    };
    let mut e = edit(&a, &a_rev);
    e.body = Some("agent".into());
    store.apply_batch(writer, &[e], &[], 5).unwrap();
    store
        .put(
            DocType::Note,
            json!({ "_id": a, "bookId": work, "title": "A", "body": "person" }),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    let undone = store.undo_batch(&batch, 9).unwrap();
    assert_eq!(undone.skipped.len(), 1);
    assert_eq!(store.note(&a).unwrap().unwrap().body, "person");
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    /// Invariant: undoing an applied agent batch of random edits, moves and
    /// creates gives every note back its title, text and notebook, and
    /// leaves no live new note.
    #[test]
    fn prop_apply_then_undo_restores_every_note(
        ops in prop::collection::vec((0usize..4, 0u8..3, "[a-z ]{0,12}"), 1..8),
        creates in 0usize..3,
    ) {
        let (_dir, store) = store();
        let one = book(&store, "One");
        let two = book(&store, "Two");
        let notes: Vec<(String, String)> =
            (0..4).map(|n| note(&store, &one, &format!("N{n}"), "start")).collect();
        let before: Vec<_> = notes.iter().map(|(id, _)| store.get(id).unwrap().unwrap().fields).collect();
        let mut edits: Vec<PluginEdit> = Vec::new();
        for (n, kind, text) in &ops {
            let (id, rev) = &notes[*n];
            edits.retain(|e| &e.id != id);
            let mut e = edit(id, rev);
            match kind {
                0 => e.body = Some(text.clone()),
                1 => e.book_id = Some(two.clone()),
                _ => { e.title = Some(format!("T {text}")); e.book_id = Some("trash".into()); }
            }
            edits.push(e);
        }
        let new: Vec<PluginCreate> = (0..creates)
            .map(|i| PluginCreate { book_id: two.clone(), title: format!("C{i}"), body: "c".into() })
            .collect();
        let batch = agent_batch_id();
        let report = store
            .apply_batch(BatchWriter::Agent { agent: "X", batch: &batch }, &edits, &new, 10)
            .unwrap();
        let undone = store.undo_batch(&batch, 20).unwrap();
        prop_assert!(undone.skipped.is_empty());
        for ((id, _), was) in notes.iter().zip(&before) {
            let now = store.get(id).unwrap().unwrap().fields;
            for key in ["title", "body", "bookId"] {
                prop_assert_eq!(now.get(key), was.get(key));
            }
        }
        for made in &report.created {
            prop_assert!(store.get(&made.id).unwrap().is_none_or(|d| d.deleted));
        }
    }
}
