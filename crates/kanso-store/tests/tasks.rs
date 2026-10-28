//! Tasks across notes, ticking them off, reminder acks, find and replace
//! with its undo, and the version kept before a sync overwrite. CI pins
//! the seed with `PROPTEST_RNG_SEED`.

use kanso_core::DocType;
use kanso_core::dates::{Date, DateTime};
use kanso_core::reminders::{ACK_FIELD, read_acks, task_key};
use kanso_core::replace::{Matcher, Pattern};
use kanso_store::{CheckOff, Origin, Planned, Put, Store, SyncDoc, TaskQuery};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use serde_json::json;

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

fn body(store: &Store, id: &str) -> String {
    store.note(id).unwrap().unwrap().body
}

fn now() -> DateTime {
    DateTime {
        date: Date::new(2026, 10, 20).unwrap(),
        hour: 10,
        minute: 30,
        second: 0,
    }
}

#[test]
fn tasks_are_listed_and_counted() {
    let (_dir, store) = temp_store();
    let a = book(&store, "book:a");
    let b = book(&store, "book:b");
    note(
        &store,
        "note:one",
        "- [ ] Late @2026-10-19\n- [ ] Soon @2026-10-20T10:00\n- [ ] Later today @2026-10-20T11:00\n- [x] Done @2026-10-01\n",
        &a,
    );
    note(
        &store,
        "note:two",
        "- [ ] Someday\n- [ ] Next @2026-11-02\n",
        &b,
    );
    note(&store, "note:gone", "- [ ] Trashed @2026-01-01\n", "trash");

    let all = store.tasks(&TaskQuery::default()).unwrap();
    let texts: Vec<&str> = all.iter().map(|t| t.text.as_str()).collect();
    assert_eq!(
        texts,
        [
            "Late @2026-10-19",
            "Soon @2026-10-20T10:00",
            "Later today @2026-10-20T11:00",
            "Next @2026-11-02",
            "Someday"
        ]
    );
    assert_eq!(store.overdue_count(now()).unwrap(), 2);
    let in_b = store
        .tasks(&TaskQuery {
            book_id: Some(b.clone()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(in_b.len(), 2);
    let with_done = store
        .tasks(&TaskQuery {
            include_done: true,
            ..Default::default()
        })
        .unwrap();
    assert_eq!(with_done.len(), 6);
    let window = store
        .tasks(&TaskQuery {
            due_after: Some("2026-10-19".into()),
            due_before: Some("2026-11-02".into()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(window.len(), 2, "both on the 20th");
    let limited = store
        .tasks(&TaskQuery {
            limit: Some(1),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(limited.len(), 1);
}

#[test]
fn ticking_a_task_writes_only_its_box() {
    let (_dir, store) = temp_store();
    let a = book(&store, "book:a");
    let text = "# Plan\n- [ ] Pay rent @2026-11-01\n- [ ] Call\n";
    note(&store, "note:one", text, &a);
    let result = store
        .set_task_checked(
            "note:one",
            1,
            "Pay rent @2026-11-01",
            true,
            5000,
            Origin::Ui,
        )
        .unwrap();
    let CheckOff::Changed { rev } = result else {
        panic!("not changed: {result:?}");
    };
    assert_eq!(store.get("note:one").unwrap().unwrap().rev, rev);
    assert_eq!(
        body(&store, "note:one"),
        text.replace("- [ ] Pay", "- [x] Pay")
    );
    assert_eq!(store.note("note:one").unwrap().unwrap().updated_at, 5000);
    assert_eq!(
        store
            .set_task_checked(
                "note:one",
                1,
                "Pay rent @2026-11-01",
                true,
                6000,
                Origin::Ui
            )
            .unwrap(),
        CheckOff::Unchanged
    );
    assert_eq!(
        store
            .set_task_checked("note:one", 1, "Pay the rent", true, 6000, Origin::Ui)
            .unwrap(),
        CheckOff::Missing
    );
    store
        .set_task_checked(
            "note:one",
            1,
            "Pay rent @2026-11-01",
            false,
            7000,
            Origin::Ui,
        )
        .unwrap();
    assert_eq!(body(&store, "note:one"), text);
}

#[test]
fn reminder_acks_merge_and_keep_the_edit_time() {
    let (_dir, store) = temp_store();
    let a = book(&store, "book:a");
    note(
        &store,
        "note:r",
        "- [ ] Call Sam @2026-10-25T09:00\n- [ ] Pay @2026-10-26T08:00\n- [ ] Plain @2026-10-26\n",
        &a,
    );
    let tasks = store.reminder_tasks().unwrap();
    assert_eq!(tasks.len(), 2, "only timed tasks remind");
    let first = task_key("note:r", "Call Sam @2026-10-25T09:00", "2026-10-25T09:00");
    assert_eq!(tasks[0].key, first);
    store
        .ack_reminder("note:r", &first, 111, Origin::Ui)
        .unwrap();
    let second = tasks[1].key.clone();
    store
        .ack_reminder("note:r", &second, 222, Origin::Ui)
        .unwrap();
    // A second ack of the same key keeps the first time.
    store
        .ack_reminder("note:r", &first, 999, Origin::Ui)
        .unwrap();
    let doc = store.get("note:r").unwrap().unwrap();
    let acks = read_acks(&doc.fields);
    assert_eq!(acks.get(&first), Some(&111));
    assert_eq!(acks.get(&second), Some(&222));
    assert_eq!(doc.fields["updatedAt"], 1000, "an ack isn't an edit");
    assert!(
        store
            .reminder_tasks()
            .unwrap()
            .iter()
            .all(|t| t.acks.contains_key(&t.key))
    );
    // A task that changed drops its old ack on the next one.
    note(&store, "note:r", "- [ ] Call Sam @2026-10-25T10:00\n", &a);
    let moved = task_key("note:r", "Call Sam @2026-10-25T10:00", "2026-10-25T10:00");
    store
        .ack_reminder("note:r", &moved, 333, Origin::Ui)
        .unwrap();
    let doc = store.get("note:r").unwrap().unwrap();
    assert_eq!(
        doc.fields[ACK_FIELD],
        json!({ moved: 333 }),
        "acks of tasks no longer there are dropped"
    );
}

/// Decision 10: a sync that overwrites a note keeps the full version it
/// replaced in history.
#[test]
fn a_sync_overwrite_keeps_the_version_it_replaced() {
    let (_dir, store) = temp_store();
    let a = book(&store, "book:a");
    let rev = note(&store, "note:s", "mine\n", &a);
    store.mark_synced("note:s", &rev, None).unwrap();
    let remote = SyncDoc {
        id: "note:s".into(),
        rev: format!("{}-{}", 2, "0123456789abcdef0123456789abcdef"),
        deleted: false,
        fields: json!({"title": "note:s", "body": "theirs\n", "bookId": a, "updatedAt": 2000})
            .as_object()
            .unwrap()
            .clone(),
        base_rev: Some(rev.clone()),
    };
    store.apply_remote(&remote, None, None).unwrap();
    assert_eq!(body(&store, "note:s"), "theirs\n");
    let history = store.revisions("note:s").unwrap();
    assert!(
        history
            .iter()
            .any(|r| r.origin == "sync" && r.fields["body"] == "mine\n" && r.rev == rev),
        "{history:?}"
    );
}

fn matcher(find: &str, skip_code: bool) -> Matcher {
    Matcher::new(&Pattern {
        find: find.into(),
        match_case: true,
        skip_code,
        ..Default::default()
    })
    .unwrap()
}

fn plan(store: &Store, m: &Matcher, trash: bool) -> Vec<Planned> {
    store
        .replace_preview(m, trash)
        .unwrap()
        .into_iter()
        .map(|n| Planned {
            id: n.id,
            rev: n.rev,
        })
        .collect()
}

#[test]
fn replace_skips_notes_changed_since_the_preview_and_undoes() {
    let (_dir, store) = temp_store();
    let a = book(&store, "book:a");
    note(&store, "note:1", "cat and cat\n", &a);
    note(&store, "note:2", "a cat\n```\ncat\n```\n", &a);
    note(&store, "note:3", "cat in the trash\n", "trash");
    note(&store, "note:4", "no match\n", &a);
    let m = matcher("cat", true);
    let preview = store.replace_preview(&m, false).unwrap();
    assert_eq!(preview.len(), 2);
    assert_eq!(preview.iter().map(|n| n.count).sum::<usize>(), 3);
    assert_eq!(store.replace_preview(&m, true).unwrap().len(), 3);
    let planned = plan(&store, &m, false);
    // A sync changes note 2 between the preview and the apply.
    note(&store, "note:2", "a cat, changed elsewhere\n", &a);
    let report = store
        .replace_in_notes(&m, "dog", &planned, "batch-1", 9000)
        .unwrap();
    assert_eq!(report.notes, ["note:1"]);
    assert_eq!(report.replaced, 2);
    assert_eq!(report.skipped.len(), 1);
    assert_eq!(report.skipped[0].id, "note:2");
    assert_eq!(body(&store, "note:1"), "dog and dog\n");
    assert_eq!(
        body(&store, "note:3"),
        "cat in the trash\n",
        "trash untouched"
    );
    let undo = store.undo_replace("batch-1", 9100).unwrap();
    assert_eq!(undo.restored, ["note:1"]);
    assert_eq!(body(&store, "note:1"), "cat and cat\n");
    // Undone once only.
    assert_eq!(
        store.undo_replace("batch-1", 9200).unwrap(),
        Default::default()
    );
}

#[test]
fn undo_leaves_notes_edited_since() {
    let (_dir, store) = temp_store();
    let a = book(&store, "book:a");
    note(&store, "note:1", "cat\n", &a);
    note(&store, "note:2", "cat\n", &a);
    let m = matcher("cat", false);
    let planned = plan(&store, &m, false);
    store
        .replace_in_notes(&m, "dog", &planned, "b", 9000)
        .unwrap();
    note(&store, "note:2", "dog, edited again\n", &a);
    let undo = store.undo_replace("b", 9100).unwrap();
    assert_eq!(undo.restored, ["note:1"]);
    assert_eq!(undo.skipped.len(), 1);
    assert_eq!(undo.skipped[0].id, "note:2");
    assert_eq!(body(&store, "note:2"), "dog, edited again\n");
}

fn sealed_fields(store: &Store, id: &str) -> serde_json::Value {
    let mut note = store.get_api(id, None, false).unwrap();
    note["body"] = "".into();
    note["kansoLock"] =
        json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "sealed"});
    note
}

/// The notebook with its lock set, as sync brings it.
fn lock_book_by_sync(store: &Store, id: &str) {
    let mut book = store.get_api(id, None, false).unwrap();
    book.as_object_mut().unwrap().remove("count");
    book["kansoLock"] =
        json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "notebook"});
    store
        .put(DocType::Book, book, Put::Upsert, Origin::Sync)
        .unwrap();
}

/// A plan that names a sealed note, or one in a locked notebook, at its
/// right revision writes nothing into it: it is skipped and reported.
#[test]
fn replace_never_writes_into_a_locked_note_even_if_the_plan_names_it() {
    let (_dir, store) = temp_store();
    let a = book(&store, "book:a");
    note(&store, "note:open", "cat\n", &a);
    note(&store, "note:sealed", "cat\n", &a);
    store
        .lock_note(sealed_fields(&store, "note:sealed"))
        .unwrap();
    // Text beside the lock, as an older Kanso on another device can sync it.
    store
        .put(
            DocType::Note,
            json!({"_id": "note:sealed", "body": "cat from an older Kanso"}),
            Put::Merge,
            Origin::Sync,
        )
        .unwrap();
    let private = book(&store, "book:private");
    note(&store, "note:inside", "cat\n", &private);
    lock_book_by_sync(&store, &private);
    assert!(store.note_is_locked("note:sealed").unwrap());
    assert!(store.note_is_locked("note:inside").unwrap());

    let forged: Vec<Planned> = ["note:open", "note:sealed", "note:inside"]
        .iter()
        .map(|id| Planned {
            id: (*id).into(),
            rev: store.get(id).unwrap().unwrap().rev,
        })
        .collect();
    let sealed_before = store.get("note:sealed").unwrap().unwrap();
    let inside_before = store.get("note:inside").unwrap().unwrap();
    let report = store
        .replace_in_notes(&matcher("cat", false), "LEAK", &forged, "forged", 9000)
        .unwrap();
    let skipped: Vec<&str> = report.skipped.iter().map(|s| s.id.as_str()).collect();
    assert_eq!(skipped, ["note:sealed", "note:inside"]);
    assert_eq!(report.notes, ["note:open"]);
    assert_eq!(store.get("note:sealed").unwrap().unwrap(), sealed_before);
    assert_eq!(
        sealed_before.fields["body"].as_str(),
        Some("cat from an older Kanso")
    );
    assert_eq!(store.get("note:inside").unwrap().unwrap(), inside_before);
    let pending = store.unsealed_in_locked_books().unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].body, "cat\n");
    assert!(body(&store, "note:open").contains("LEAK"));
}

/// The preview reads only what the app reads: a locked note has no text
/// there, so it is never listed.
#[test]
fn the_preview_leaves_locked_notes_out() {
    let (_dir, store) = temp_store();
    let a = book(&store, "book:a");
    note(&store, "note:open", "cat\n", &a);
    note(&store, "note:sealed", "cat\n", &a);
    store
        .lock_note(sealed_fields(&store, "note:sealed"))
        .unwrap();
    let private = book(&store, "book:private");
    note(&store, "note:inside", "cat\n", &private);
    lock_book_by_sync(&store, &private);
    let found = store.replace_preview(&matcher("cat", false), true).unwrap();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].id, "note:open");
}

/// A note that reads as locked by the time the batch is undone keeps what
/// it has, even if its revision is still the one the replace made.
#[test]
fn undo_never_writes_plain_text_into_a_note_locked_since() {
    let (_dir, store) = temp_store();
    let a = book(&store, "book:a");
    note(&store, "note:1", "cat\n", &a);
    let m = matcher("cat", false);
    let planned = plan(&store, &m, false);
    store
        .replace_in_notes(&m, "dog", &planned, "b", 9000)
        .unwrap();
    // The notebook is locked on another device: the text here is kept for
    // the app to seal, and the note's versions are forgotten.
    lock_book_by_sync(&store, &a);
    assert!(store.revisions("note:1").unwrap().is_empty());
    // A batch entry that outlived that (restored from a copy, say).
    let rev = store.get("note:1").unwrap().unwrap().rev;
    let conn = rusqlite::Connection::open(_dir.path().join("kanso.db")).unwrap();
    conn.execute(
        "INSERT INTO revisions (doc_id, rev, json, saved_at, origin, batch, batch_rev)
         VALUES ('note:1', 'old', ?1, 1, 'ui', 'b', ?2)",
        rusqlite::params![r#"{"body":"cat\n","title":"note:1"}"#, rev],
    )
    .unwrap();
    drop(conn);
    let undo = store.undo_replace("b", 9100).unwrap();
    assert!(undo.restored.is_empty());
    assert_eq!(undo.skipped.len(), 1);
    assert_eq!(undo.skipped[0].id, "note:1");
    assert_eq!(store.unsealed_in_locked_books().unwrap()[0].body, "dog\n");
}

fn bodies() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(
        prop::collection::vec(
            prop_oneof![
                "[ab ]{0,10}".prop_map(|s| s + "\n"),
                Just("```\n".to_owned()),
                "[ab]{1,3}".prop_map(|s| format!("- [ ] {s}\n")),
            ],
            0..6,
        )
        .prop_map(|lines| lines.concat()),
        1..5,
    )
}

proptest! {
    #![proptest_config(config(48))]

    /// Round trip: replacing across notes and undoing the batch gives every
    /// body back byte for byte, and with code skipped no fence changed.
    #[test]
    fn prop_replace_then_undo_restores_bodies(bodies in bodies(), find in "[ab]{1,2}", with in "[xy\n]{0,3}", skip: bool) {
        let (_dir, store) = temp_store();
        let a = book(&store, "book:a");
        let ids: Vec<String> = bodies.iter().enumerate().map(|(i, b)| {
            let id = format!("note:n{i}");
            note(&store, &id, b, &a);
            id
        }).collect();
        let m = matcher(&find, skip);
        let planned = plan(&store, &m, false);
        let report = store.replace_in_notes(&m, &with, &planned, "batch", 5000).unwrap();
        let expected: usize = bodies.iter().map(|b| m.matches(b).len()).sum();
        prop_assert_eq!(report.replaced, expected);
        prop_assert!(report.skipped.is_empty());
        for (id, original) in ids.iter().zip(&bodies) {
            let now = body(&store, id);
            prop_assert_eq!(&now, &m.replace(original, &with).0);
            if skip {
                let fences = |b: &str| kanso_core::replace::fence_ranges(b).into_iter().map(|r| b[r].to_owned()).collect::<Vec<_>>();
                if !with.contains('\n') {
                    prop_assert_eq!(fences(&now), fences(original));
                }
            }
        }
        let undo = store.undo_replace("batch", 6000).unwrap();
        prop_assert_eq!(undo.restored.len(), report.notes.len());
        for (id, original) in ids.iter().zip(&bodies) {
            prop_assert_eq!(&body(&store, id), original);
        }
    }

    /// Model-based: ticking and clearing tasks in any order through the
    /// store leaves each note's tasks as a simple list of flags says, with
    /// the rest of the body untouched.
    #[test]
    fn prop_check_offs_follow_the_model(
        count in 1usize..6,
        ops in prop::collection::vec((any::<prop::sample::Index>(), any::<bool>()), 0..20),
    ) {
        let (_dir, store) = temp_store();
        let a = book(&store, "book:a");
        let text: String = (0..count).map(|i| format!("intro {i}\n- [ ] task {i} @2026-10-2{i}\n")).collect();
        note(&store, "note:m", &text, &a);
        let mut model = vec![false; count];
        for (ix, checked) in ops {
            let i = ix.index(count);
            let rows = store.tasks(&TaskQuery { include_done: true, ..Default::default() }).unwrap();
            let row = rows.iter().find(|r| r.text.starts_with(&format!("task {i} "))).unwrap();
            let result = store.set_task_checked("note:m", row.line, &row.text, checked, 7000, Origin::Ui).unwrap();
            let expected_changed = model[i] != checked;
            prop_assert_eq!(matches!(result, CheckOff::Changed { .. }), expected_changed);
            prop_assert_eq!(result == CheckOff::Unchanged, !expected_changed);
            model[i] = checked;
        }
        let rows = store.tasks(&TaskQuery { include_done: true, ..Default::default() }).unwrap();
        prop_assert_eq!(rows.len(), count);
        for (i, done) in model.iter().enumerate() {
            let row = rows.iter().find(|r| r.text.starts_with(&format!("task {i} "))).unwrap();
            prop_assert_eq!(row.checked, *done);
        }
        let expected: String = (0..count).map(|i| {
            let mark = if model[i] { 'x' } else { ' ' };
            format!("intro {i}\n- [{mark}] task {i} @2026-10-2{i}\n")
        }).collect();
        prop_assert_eq!(body(&store, "note:m"), expected);
    }
}
