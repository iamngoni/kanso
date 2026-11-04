//! Open loops across notes: what the rules find, closing them, the
//! approved AI loops, the history of what was finished, and that locked
//! notes never produce or keep any of it. CI pins the seed with
//! `PROPTEST_RNG_SEED`.

use std::collections::BTreeSet;

use kanso_core::DocType;
use kanso_core::loops::{AiLoop, LOOPS_FIELD, LoopKind, loop_items, loop_key};
use kanso_store::{CheckOff, LoopClose, LoopQuery, Origin, Put, Store};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use serde_json::{Value, json};

const CANARY: &str = "zebracanary4471";

fn config(cases: u32) -> ProptestConfig {
    ProptestConfig {
        cases,
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
        ..ProptestConfig::default()
    }
}

fn temp_store() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    (dir, store)
}

fn book(store: &Store, id: &str) -> String {
    store
        .put(
            DocType::Book,
            json!({"_id": id, "name": id}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id
}

fn note(store: &Store, id: &str, body: &str, book: &str) -> String {
    let current = store.get(id).unwrap().map(|d| d.rev);
    let mut doc = json!({"_id": id, "title": id, "body": body, "bookId": book, "updatedAt": 1000});
    if let Some(rev) = current {
        doc["_rev"] = json!(rev);
    }
    store
        .put(DocType::Note, doc, Put::Upsert, Origin::Ui)
        .unwrap()
        .rev
}

/// Changes a note's text, keeping its other fields (as the app's saves do).
fn edit(store: &Store, id: &str, body: &str) {
    store
        .put(
            DocType::Note,
            json!({"_id": id, "body": body, "updatedAt": 2000}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
}

fn body(store: &Store, id: &str) -> String {
    store.note(id).unwrap().unwrap().body
}

fn keys(store: &Store) -> BTreeSet<String> {
    store
        .open_loops(&LoopQuery::default())
        .unwrap()
        .into_iter()
        .map(|l| l.key)
        .collect()
}

#[test]
fn loops_are_listed_with_their_line_kind_and_note() {
    let (_dir, store) = temp_store();
    let b = book(&store, "book:a");
    note(
        &store,
        "note:one",
        "Intro\n\nI'll send Tendai the deck.\n- [ ] a task\nWaiting on Rudo for the quote\n",
        &b,
    );
    let rows = store.open_loops(&LoopQuery::default()).unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!((rows[0].line, rows[0].kind), (2, LoopKind::Mine));
    assert_eq!(rows[0].text, "I'll send Tendai the deck.");
    assert_eq!((rows[1].line, rows[1].kind), (4, LoopKind::Waiting));
    assert_eq!(rows[0].note_title, "note:one");
    assert_eq!(rows[0].key, loop_key("note:one", &rows[0].text));
    assert_eq!(
        store
            .open_loops(&LoopQuery {
                kind: Some(LoopKind::Waiting),
                ..LoopQuery::default()
            })
            .unwrap()
            .len(),
        1
    );
    assert_eq!(store.open_loop_count(&LoopQuery::default()).unwrap(), 2);
    // Edited away: gone.
    note(&store, "note:one", "Nothing here.", &b);
    assert!(keys(&store).is_empty());
}

#[test]
fn done_and_dismissed_hide_a_loop_without_touching_the_text() {
    let (_dir, store) = temp_store();
    let b = book(&store, "book:a");
    note(
        &store,
        "note:one",
        "I'll call Rudo.\nNeed to pay the rent.",
        &b,
    );
    let before = store.note("note:one").unwrap().unwrap();
    let rows = store.open_loops(&LoopQuery::default()).unwrap();
    let (a, c) = (&rows[0], &rows[1]);
    let done = store
        .close_loop(
            "note:one",
            &a.key,
            a.line,
            LoopClose::Done {
                mark_in_note: false,
            },
            5000,
            Origin::Ui,
        )
        .unwrap();
    assert!(matches!(done, CheckOff::Changed { .. }));
    assert_eq!(keys(&store).len(), 1);
    assert_eq!(body(&store, "note:one"), before.body);
    // Closing again changes nothing.
    assert_eq!(
        store
            .close_loop(
                "note:one",
                &a.key,
                a.line,
                LoopClose::Done {
                    mark_in_note: false
                },
                6000,
                Origin::Ui,
            )
            .unwrap(),
        CheckOff::Unchanged
    );
    store
        .close_loop(
            "note:one",
            &c.key,
            c.line,
            LoopClose::Dismissed,
            5000,
            Origin::Ui,
        )
        .unwrap();
    assert!(keys(&store).is_empty());
    // The edit time stays: an ack isn't an edit.
    let after = store.note("note:one").unwrap().unwrap();
    assert_eq!(after.updated_at, before.updated_at);
    // The closure is in the field, and lists it as finished.
    let doc = store.get_api("note:one", None, false).unwrap();
    assert!(doc[LOOPS_FIELD]["closed"][&a.key].is_number());
    assert!(doc[LOOPS_FIELD]["dismissed"][&c.key].is_number());
    let finished = store.finished_between(0, 10_000).unwrap();
    assert_eq!(finished.len(), 1);
    assert_eq!((finished[0].kind.as_str(), finished[0].at), ("loop", 5000));
    // A missing loop is reported.
    assert_eq!(
        store
            .close_loop(
                "note:one",
                "note:one~nope",
                0,
                LoopClose::Dismissed,
                1,
                Origin::Ui
            )
            .unwrap(),
        CheckOff::Missing
    );
}

#[test]
fn marking_done_in_the_note_adds_a_tick_when_asked() {
    let (_dir, store) = temp_store();
    let b = book(&store, "book:a");
    note(&store, "note:one", "I'll call Rudo.\n", &b);
    let row = store.open_loops(&LoopQuery::default()).unwrap().remove(0);
    store
        .close_loop(
            "note:one",
            &row.key,
            row.line,
            LoopClose::Done { mark_in_note: true },
            5000,
            Origin::Ui,
        )
        .unwrap();
    assert_eq!(body(&store, "note:one"), "I'll call Rudo. \u{2713}\n");
    assert!(keys(&store).is_empty());
}

#[test]
fn make_task_rewrites_one_line_and_undoes() {
    let (_dir, store) = temp_store();
    let b = book(&store, "book:a");
    let original = "intro\n- I'll call Rudo.\nend\n";
    note(&store, "note:one", original, &b);
    let row = store.open_loops(&LoopQuery::default()).unwrap().remove(0);
    store
        .close_loop(
            "note:one",
            &row.key,
            row.line,
            LoopClose::MakeTask,
            5000,
            Origin::Api,
        )
        .unwrap();
    assert_eq!(
        body(&store, "note:one"),
        "intro\n- [ ] I'll call Rudo.\nend\n"
    );
    // It is a task now and no longer a loop.
    assert!(keys(&store).is_empty());
    assert_eq!(store.tasks(&Default::default()).unwrap().len(), 1);
    // The version before is kept, so the edit undoes.
    let revisions = store.revisions("note:one").unwrap();
    let last = revisions.last().expect("a saved version");
    assert_eq!(last.fields["body"], json!(original));
}

#[test]
fn approved_ai_loops_are_stored_checked_and_follow_their_sentence() {
    let (_dir, store) = temp_store();
    let b = book(&store, "book:a");
    note(
        &store,
        "note:one",
        "Lease renewal is due.\nI'll call Rudo.\n",
        &b,
    );
    let ai = |text: &str| AiLoop {
        key: String::new(),
        text: text.to_owned(),
        kind: LoopKind::Mine,
        approved_at: 7,
    };
    let added = store
        .approve_ai_loops(
            "note:one",
            &[
                ai("Lease renewal is due."),
                ai("A sentence that is not there"),
                ai("I'll call Rudo."),
            ],
            Origin::Ui,
        )
        .unwrap();
    assert_eq!(added, 1, "unverified and rule duplicates are dropped");
    let rows = store.open_loops(&LoopQuery::default()).unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[1].source, kanso_store::LoopSource::Ai);
    assert_eq!(rows[1].line, 0);
    // Moved: found again at its new line. Edited away: dropped.
    edit(
        &store,
        "note:one",
        "x\nLease renewal is due.\nI'll call Rudo.\n",
    );
    assert_eq!(store.open_loops(&LoopQuery::default()).unwrap()[1].line, 1);
    edit(&store, "note:one", "I'll call Rudo.\n");
    assert_eq!(keys(&store).len(), 1);
    // Nothing is stored twice.
    edit(
        &store,
        "note:one",
        "Lease renewal is due.\nI'll call Rudo.\n",
    );
    assert_eq!(keys(&store).len(), 2);
}

#[test]
fn ticking_then_unticking_leaves_no_history_and_a_tick_is_recorded() {
    let (_dir, store) = temp_store();
    let b = book(&store, "book:a");
    note(&store, "note:one", "- [ ] pay rent\n- [ ] call Rudo\n", &b);
    assert!(store.finished_between(0, i64::MAX).unwrap().is_empty());
    note(&store, "note:one", "- [x] pay rent\n- [ ] call Rudo\n", &b);
    let done = store.finished_between(0, i64::MAX).unwrap();
    assert_eq!(done.len(), 1);
    assert_eq!(
        (done[0].kind.as_str(), done[0].text.as_str()),
        ("task", "pay rent")
    );
    note(&store, "note:one", "- [ ] pay rent\n- [ ] call Rudo\n", &b);
    assert!(store.finished_between(0, i64::MAX).unwrap().is_empty());
    // A task that was done when first written isn't a tick.
    note(&store, "note:two", "- [x] already done\n", &b);
    assert!(store.finished_between(0, i64::MAX).unwrap().is_empty());
}

#[test]
fn a_synced_tick_is_recorded_when_it_arrives() {
    let (_dir, store) = temp_store();
    let b = book(&store, "book:a");
    note(&store, "note:one", "- [ ] pay rent\n", &b);
    let rev = store.note("note:one").unwrap().unwrap().rev;
    let doc = kanso_store::SyncDoc {
        id: "note:one".into(),
        rev: "2-aa".into(),
        deleted: false,
        fields: json!({"bookId": b, "title": "note:one", "body": "- [x] pay rent\n", "updatedAt": 9999999})
            .as_object()
            .unwrap()
            .clone(),
        base_rev: None,
    };
    let _ = rev;
    store.apply_remote(&doc, None, None).unwrap();
    assert_eq!(body(&store, "note:one"), "- [x] pay rent\n");
    let done = store.finished_between(0, i64::MAX).unwrap();
    assert_eq!(done.len(), 1);
    assert!(done[0].at > 1_700_000_000_000, "stamped with this clock");
}

#[test]
fn history_goes_with_a_trashed_deleted_or_locked_note() {
    let (_dir, store) = temp_store();
    let b = book(&store, "book:a");
    note(&store, "note:one", "- [ ] pay rent\n", &b);
    note(&store, "note:one", "- [x] pay rent\n", &b);
    assert_eq!(store.finished_between(0, i64::MAX).unwrap().len(), 1);
    let sealed = json!({"_id": "note:one", "_rev": store.note("note:one").unwrap().unwrap().rev,
        "bookId": b, "title": "note:one", "body": "",
        "kansoLock": {"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "sealed"}});
    store.lock_note(sealed).unwrap();
    assert!(store.finished_between(0, i64::MAX).unwrap().is_empty());
}

// MARK: Locks

fn lock() -> Value {
    json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "sealed"})
}

#[test]
fn a_locked_note_and_a_locked_notebook_produce_no_loops() {
    let (_dir, store) = temp_store();
    let plain = book(&store, "book:plain");
    let secret = book(&store, "book:secret");
    note(
        &store,
        "note:plain",
        &format!("I'll call {CANARY}."),
        &plain,
    );
    note(&store, "note:own", &format!("I'll call {CANARY}."), &plain);
    note(&store, "note:in", &format!("I'll call {CANARY}."), &secret);
    assert_eq!(keys(&store).len(), 3);

    let own = json!({"_id": "note:own", "_rev": store.note("note:own").unwrap().unwrap().rev,
        "bookId": plain, "title": "own", "body": "", "kansoLock": lock()});
    store.lock_note(own).unwrap();
    let sealed_in = json!({"_id": "note:in", "_rev": store.note("note:in").unwrap().unwrap().rev,
        "bookId": secret, "title": "in", "body": "", "kansoLock": lock()});
    let mut book_doc = store.get_api(&secret, None, false).unwrap();
    book_doc.as_object_mut().unwrap().remove("count");
    book_doc["kansoLock"] = lock();
    store.lock_book(book_doc, vec![sealed_in]).unwrap();

    let rows = store.open_loops(&LoopQuery::default()).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].note_id, "note:plain");
    assert_eq!(store.open_loop_count(&LoopQuery::default()).unwrap(), 1);
    for id in ["note:own", "note:in"] {
        assert!(
            store
                .notes_changed_between(0, i64::MAX)
                .unwrap()
                .iter()
                .all(|n| n.id != id)
        );
    }
}

#[test]
fn locking_clears_kanso_loops() {
    let (_dir, store) = temp_store();
    let b = book(&store, "book:a");
    note(&store, "note:one", "I'll call Rudo about the zebra.", &b);
    let row = store.open_loops(&LoopQuery::default()).unwrap().remove(0);
    store
        .close_loop("note:one", &row.key, 0, LoopClose::Dismissed, 5, Origin::Ui)
        .unwrap();
    store
        .approve_ai_loops(
            "note:one",
            &[AiLoop {
                key: String::new(),
                text: "I'll call Rudo about the zebra.".into(),
                kind: LoopKind::Mine,
                approved_at: 1,
            }],
            Origin::Ui,
        )
        .unwrap();
    let mut doc = store.get_api("note:one", None, false).unwrap();
    assert!(doc.get(LOOPS_FIELD).is_some());
    // The app seals the note: the body goes, the field was in the doc it read.
    doc["body"] = "".into();
    doc["kansoLock"] = lock();
    store.lock_note(doc).unwrap();
    let raw = store.get("note:one").unwrap().unwrap();
    assert!(!raw.fields.contains_key(LOOPS_FIELD), "the field is gone",);
}

// MARK: Properties

#[derive(Clone, Debug)]
enum Op {
    Edit(Vec<usize>),
    Done(usize),
    Dismiss(usize),
    Task(usize),
}

const LINES: [&str; 8] = [
    "I'll call Rudo.",
    "Waiting on Tendai for the quote",
    "Need to pay the rent",
    "Check whether the invoice was paid?",
    "Just a plain line",
    "- [ ] a real task",
    "TODO: renew passport",
    "Another plain line",
];

fn op() -> impl Strategy<Value = Op> {
    prop_oneof![
        prop::collection::vec(0..LINES.len(), 0..7).prop_map(Op::Edit),
        (0usize..8).prop_map(Op::Done),
        (0usize..8).prop_map(Op::Dismiss),
        (0usize..8).prop_map(Op::Task),
    ]
}

proptest! {
    #![proptest_config(config(48))]

    /// What is open always equals the rules' loops in the current text,
    /// minus those closed or dismissed (whose sentence is still there).
    #[test]
    fn prop_open_loops_equals_rules_minus_closed_and_dismissed(
        ops in prop::collection::vec(op(), 1..14)
    ) {
        let (_dir, store) = temp_store();
        let b = book(&store, "book:a");
        note(&store, "note:one", "", &b);
        let mut hidden: BTreeSet<String> = BTreeSet::new();
        for op in ops {
            match op {
                Op::Edit(picks) => {
                    let text = picks.iter().map(|&i| LINES[i]).collect::<Vec<_>>().join("\n");
                    edit(&store, "note:one", &text);
                }
                Op::Done(i) | Op::Dismiss(i) | Op::Task(i) => {
                    let rows = store.open_loops(&LoopQuery::default()).unwrap();
                    if rows.is_empty() { continue; }
                    let row = &rows[i % rows.len()];
                    let how = match op {
                        Op::Done(_) => LoopClose::Done { mark_in_note: false },
                        Op::Dismiss(_) => LoopClose::Dismissed,
                        _ => LoopClose::MakeTask,
                    };
                    // Closing prunes what is no longer in the note (so the
                    // field doesn't grow); a sentence that comes back
                    // after that is open again.
                    let live: Vec<String> = loop_items(&body(&store, "note:one"))
                        .iter()
                        .map(|l| loop_key("note:one", &l.text))
                        .collect();
                    // (Make Task rewrites the line and leaves the field be.)
                    if !matches!(op, Op::Task(_)) {
                        hidden.retain(|k| live.contains(k));
                    }
                    store.close_loop("note:one", &row.key, row.line, how, 10, Origin::Ui).unwrap();
                    if !matches!(op, Op::Task(_)) {
                        hidden.insert(row.key.clone());
                    }
                }
            }
            let text = body(&store, "note:one");
            let expected: BTreeSet<String> = loop_items(&text)
                .iter()
                .map(|l| loop_key("note:one", &l.text))
                .filter(|k| !hidden.contains(k))
                .collect();
            prop_assert_eq!(keys(&store), expected);
        }
    }

    /// However the box is edited on the way to a tick (space deleted,
    /// bracket removed, then the x typed), the tick is finished exactly
    /// once, and the half-typed saves finish nothing.
    #[test]
    fn prop_a_tick_typed_over_several_saves_is_finished_once(
        n in 1usize..5,
        pick in 0usize..5,
        steps in proptest::collection::vec(0usize..4, 0..4),
    ) {
        const HALF: [&str; 4] = ["[]", "[", "", "[ "];
        let (_dir, store) = temp_store();
        let b = book(&store, "book:a");
        let pick = pick % n;
        let with = |box_: &str| -> String {
            (0..n)
                .map(|i| {
                    if i == pick {
                        format!("- {box_} task {i}\n")
                    } else {
                        format!("- [ ] task {i}\n")
                    }
                })
                .collect()
        };
        note(&store, "note:one", &with("[ ]"), &b);
        for step in steps {
            note(&store, "note:one", &with(HALF[step]), &b);
            prop_assert!(store.finished_between(0, i64::MAX).unwrap().is_empty());
        }
        note(&store, "note:one", &with("[x]"), &b);
        let done = store.finished_between(0, i64::MAX).unwrap();
        prop_assert_eq!(done.len(), 1);
        prop_assert_eq!(done[0].text.clone(), format!("task {pick}"));
    }

    /// Ticking a task and unticking it leaves no history; ticking it
    /// leaves exactly one row.
    #[test]
    fn prop_ticking_then_unticking_leaves_no_row(n in 1usize..5, pick in 0usize..5) {
        let (_dir, store) = temp_store();
        let b = book(&store, "book:a");
        let open: String = (0..n).map(|i| format!("- [ ] task {i}\n")).collect();
        note(&store, "note:one", &open, &b);
        let pick = pick % n;
        let ticked: String = (0..n)
            .map(|i| format!("- [{}] task {i}\n", if i == pick { "x" } else { " " }))
            .collect();
        note(&store, "note:one", &ticked, &b);
        prop_assert_eq!(store.finished_between(0, i64::MAX).unwrap().len(), 1);
        note(&store, "note:one", &open, &b);
        prop_assert!(store.finished_between(0, i64::MAX).unwrap().is_empty());
    }
}

#[test]
fn a_checked_line_that_was_never_open_is_not_a_tick() {
    let (_dir, store) = temp_store();
    let b = book(&store, "book:a");
    note(&store, "note:one", "- [ ] pay rent\n", &b);
    note(&store, "note:one", "- [] pay rent\n", &b);
    note(&store, "note:one", "- [x] pay rent\n- [x] never open\n", &b);
    let done = store.finished_between(0, i64::MAX).unwrap();
    assert_eq!(done.len(), 1);
    assert_eq!(done[0].text, "pay rent");
}
