//! Adding a task to the end of a note, today's daily note or the Inbox's
//! task list. CI pins the seed with `PROPTEST_RNG_SEED`.

use kanso_core::DocType;
use kanso_core::dates::{Date, DateTime};
use kanso_core::model::TRASH_BOOK_ID;
use kanso_core::roles::Role;
use kanso_core::roles::daily_id;
use kanso_core::tasks::task_items;
use kanso_store::{
    INBOX_TASKS_ID, INBOX_TASKS_TITLE, LEGACY_INBOX_TASKS_TITLE, Origin, Put, Store, StoreError,
    TaskTarget, merge_task_bodies,
};
use proptest::prelude::*;
use serde_json::json;

fn temp_store() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    (dir, store)
}

fn now() -> DateTime {
    DateTime {
        date: Date::new(2026, 11, 12).unwrap(),
        hour: 10,
        minute: 30,
        second: 0,
    }
}

fn note(store: &Store, body: &str) -> String {
    let book = store
        .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
        .unwrap()
        .id;
    store
        .put(
            DocType::Note,
            json!({"title": "N", "body": body, "bookId": book}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id
}

fn body(store: &Store, id: &str) -> String {
    store.note(id).unwrap().unwrap().body
}

#[test]
fn a_task_goes_after_the_last_line_whatever_ends_the_body() {
    let (_dir, store) = temp_store();
    for (before, after, line) in [
        ("", "- [ ] t\n", 0),
        ("a", "a\n- [ ] t\n", 1),
        ("a\n", "a\n- [ ] t\n", 1),
        ("a\n\nb\n", "a\n\nb\n- [ ] t\n", 3),
    ] {
        let id = note(&store, before);
        let added = store
            .add_task(
                &TaskTarget::Note(id.clone()),
                "- [ ] t",
                now(),
                5,
                Origin::Api,
            )
            .unwrap();
        assert_eq!(body(&store, &id), after);
        assert_eq!(added.line, line);
        assert_eq!(added.note_id, id);
    }
}

#[test]
fn a_missing_or_trashed_note_is_not_found() {
    let (_dir, store) = temp_store();
    let err = store
        .add_task(
            &TaskTarget::Note("note:nope".into()),
            "- [ ] t",
            now(),
            5,
            Origin::Api,
        )
        .unwrap_err();
    assert!(matches!(err, StoreError::NotFound));
}

#[test]
fn the_daily_note_is_made_once_and_grows() {
    let (_dir, store) = temp_store();
    let one = store
        .add_task(&TaskTarget::Daily, "- [ ] one", now(), 5, Origin::Api)
        .unwrap();
    let two = store
        .add_task(&TaskTarget::Daily, "- [ ] two", now(), 6, Origin::Api)
        .unwrap();
    assert_eq!(one.note_id, daily_id(now().date));
    assert_eq!(two.note_id, one.note_id);
    assert_eq!((one.line, two.line), (0, 1));
    assert_eq!(body(&store, &one.note_id), "- [ ] one\n- [ ] two\n");
}

#[test]
fn the_inbox_task_list_is_one_note_in_the_inbox() {
    let (_dir, store) = temp_store();
    let one = store
        .add_task(&TaskTarget::Inbox, "- [ ] one", now(), 5, Origin::Api)
        .unwrap();
    let two = store
        .add_task(&TaskTarget::Inbox, "- [ ] two", now(), 6, Origin::Api)
        .unwrap();
    assert_eq!(one.note_id, two.note_id);
    // The same note Quick Capture's Task mode writes to.
    assert_eq!(one.note_id, INBOX_TASKS_ID);
    let saved = store.note(&one.note_id).unwrap().unwrap();
    assert_eq!(saved.title, INBOX_TASKS_TITLE);
    assert_eq!(saved.body, "- [ ] one\n- [ ] two\n");
    assert_eq!(saved.updated_at, 6);
}

/// A note in the Inbox titled as the local API's old task note.
fn legacy_note(store: &Store, body: &str) -> String {
    let inbox = store.role_book(Role::Inbox, Origin::Ui).unwrap();
    store
        .put(
            DocType::Note,
            json!({"title": LEGACY_INBOX_TASKS_TITLE, "body": body, "bookId": inbox}),
            Put::Upsert,
            Origin::Api,
        )
        .unwrap()
        .id
}

#[test]
fn an_old_tasks_note_in_the_inbox_is_folded_into_the_inbox_tasks_note() {
    let (_dir, store) = temp_store();
    let old = legacy_note(&store, "- [ ] from raycast\n- [x] done one");
    let added = store
        .add_task(&TaskTarget::Inbox, "- [ ] new", now(), 9, Origin::Api)
        .unwrap();
    assert_eq!(added.note_id, INBOX_TASKS_ID);
    assert_eq!(added.line, 2);
    assert_eq!(
        body(&store, INBOX_TASKS_ID),
        "- [ ] from raycast\n- [x] done one\n- [ ] new\n"
    );
    // The old note goes to the trash, where it can still be found.
    assert_eq!(store.note(&old).unwrap().unwrap().book_id, TRASH_BOOK_ID);
    // Once folded, nothing is folded again.
    let again = store
        .add_task(&TaskTarget::Inbox, "- [ ] more", now(), 10, Origin::Api)
        .unwrap();
    assert_eq!(again.line, 3);
}

#[test]
fn a_tasks_note_outside_the_inbox_is_left_alone() {
    let (_dir, store) = temp_store();
    let book = store
        .put(
            DocType::Book,
            json!({"name": "Work"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    let elsewhere = store
        .put(
            DocType::Note,
            json!({"title": LEGACY_INBOX_TASKS_TITLE, "body": "- [ ] mine\n", "bookId": book}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    store
        .add_task(&TaskTarget::Inbox, "- [ ] new", now(), 9, Origin::Api)
        .unwrap();
    let kept = store.note(&elsewhere).unwrap().unwrap();
    assert_eq!(kept.book_id, book);
    assert_eq!(kept.body, "- [ ] mine\n");
    assert_eq!(body(&store, INBOX_TASKS_ID), "- [ ] new\n");
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// Folding keeps every line of both notes, the Inbox task note's first,
    /// and adds nothing else.
    #[test]
    fn prop_folding_keeps_every_line(
        into in prop::collection::vec("[a-z\\[\\] -]{0,10}", 0..5),
        from in prop::collection::vec("[a-z\\[\\] -]{0,10}", 0..5),
        into_end in any::<bool>(),
        from_end in any::<bool>(),
    ) {
        let join = |lines: &[String], end: bool| {
            let mut text = lines.join("\n");
            if end && !text.is_empty() {
                text.push('\n');
            }
            text
        };
        let (into, from) = (join(&into, into_end), join(&from, from_end));
        let merged = merge_task_bodies(&into, &from);
        if from.trim().is_empty() {
            prop_assert_eq!(&merged, &into);
        } else {
            let want: Vec<&str> = into.lines().chain(from.lines()).collect();
            prop_assert_eq!(merged.lines().collect::<Vec<_>>(), want);
            prop_assert!(merged.ends_with('\n'));
        }
    }

    /// Through the store: the old note's lines, then each new task, all in
    /// the one Inbox task note, folded once.
    #[test]
    fn prop_an_old_note_is_folded_once(
        old in prop::collection::vec("- \\[ \\] [a-z]{1,6}", 0..4),
    ) {
        let (_dir, store) = temp_store();
        legacy_note(&store, &old.join("\n"));
        store.add_task(&TaskTarget::Inbox, "- [ ] new", now(), 5, Origin::Api).unwrap();
        store.add_task(&TaskTarget::Inbox, "- [ ] newer", now(), 6, Origin::Api).unwrap();
        let mut want = old.clone();
        want.push("- [ ] new".into());
        want.push("- [ ] newer".into());
        let saved = body(&store, INBOX_TASKS_ID);
        prop_assert_eq!(saved.lines().map(str::to_owned).collect::<Vec<_>>(), want);
    }

    /// The line returned is where the task is, and the earlier lines are
    /// untouched.
    #[test]
    fn prop_the_returned_line_is_the_task(
        lines in prop::collection::vec("[a-z ]{0,8}", 0..6),
        trailing in any::<bool>(),
    ) {
        let (_dir, store) = temp_store();
        let mut before = lines.join("\n");
        if trailing && !before.is_empty() {
            before.push('\n');
        }
        let id = note(&store, &before);
        let added = store
            .add_task(&TaskTarget::Note(id.clone()), "- [ ] new thing", now(), 5, Origin::Api)
            .unwrap();
        let after = body(&store, &id);
        prop_assert!(after.starts_with(&before));
        let items = task_items(&after);
        let last = items.last().unwrap();
        prop_assert_eq!(last.line, added.line);
        prop_assert_eq!(&last.text, "new thing");
        prop_assert_eq!(after.lines().nth(added.line as usize), Some("- [ ] new thing"));
    }
}
