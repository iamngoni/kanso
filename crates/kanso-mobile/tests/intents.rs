//! What the phone's App Intents read and leave (#390 slice 18): "Add Kanso
//! task"'s preview and its `task` drops, saved by the inbox like shares, and
//! the notes "Open Kanso note" offers, never a locked one.

mod common;

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use common::*;
use kanso_core::DocType;
use kanso_core::roles::Role;
use kanso_mobile::intents::{
    MAX_INTENT_NOTES, intent_notes_json, matching_notes, task_preview_json,
};
use kanso_mobile::{DropsSaved, Library, LocalTime, NewNote, NoteRow};
use kanso_store::{Origin, Put, Store};
use kanso_sync::MemoryServer;
use proptest::prelude::*;
use serde_json::{Value, json};

/// Thursday 12 November 2026, 09:41.
fn now() -> LocalTime {
    LocalTime {
        year: 2026,
        month: 11,
        day: 12,
        hour: 9,
        minute: 41,
        second: 0,
    }
}

fn inbox_of(phone: &Phone) -> PathBuf {
    phone.dir.path().join("inbox")
}

fn id(n: u8) -> String {
    format!("{n:02x}").repeat(16)
}

fn write_drop(inbox: &Path, id: &str, manifest: &Value) -> PathBuf {
    let folder = inbox.join(id);
    fs::create_dir_all(&folder).unwrap();
    fs::write(folder.join("drop.json"), manifest.to_string()).unwrap();
    folder
}

fn task_drop(line: &str, target: Option<&str>, created: i64) -> Value {
    let mut drop = json!({
        "version": 1, "kind": "task", "createdAt": created,
        "line": line, "day": "2026-11-12", "time": "09:41",
    });
    if let Some(target) = target {
        drop["target"] = json!(target);
    }
    drop
}

fn lock_book(store: &Store, id: &str) {
    let lock = json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "notebook"});
    store
        .put(
            DocType::Book,
            json!({"_id": id, "kansoLock": lock}),
            Put::Merge,
            Origin::Sync,
        )
        .unwrap();
}

fn body(library: &Library, id: &str) -> String {
    library.note(id).unwrap().body.unwrap()
}

// no-prop-test: the spec's example, read once.
#[test]
fn the_preview_confirms_like_the_spec() {
    let json: Value = serde_json::from_str(&task_preview_json(
        "film Pixel review fri 3pm every week",
        now(),
        false,
    ))
    .unwrap();
    assert_eq!(
        json["line"],
        "- [ ] film Pixel review @2026-11-13T15:00 every week"
    );
    assert_eq!(json["words"], "film Pixel review");
    assert_eq!(
        json["confirmation"],
        "Added: film Pixel review \u{b7} Fri 13 Nov 15:00 \u{b7} every week"
    );
    assert_eq!(task_preview_json("   ", now(), false), "null");
    let bad = LocalTime { hour: 24, ..now() };
    assert_eq!(task_preview_json("x", bad, false), "null");
    // Month first reads 11/14 as November 14th.
    let us: Value =
        serde_json::from_str(&task_preview_json("call Sam 11/14", now(), true)).unwrap();
    assert_eq!(us["line"], "- [ ] call Sam @2026-11-14");
}

// no-prop-test: the contract's sequence for a task drop, once per target.
#[test]
fn a_task_drop_lands_in_the_daily_note_or_the_inbox_and_goes() {
    let phone = lone_phone();
    let inbox = inbox_of(&phone);
    let line = "- [ ] film Pixel review @2026-11-14T15:00 every week";
    let daily = write_drop(&inbox, &id(1), &task_drop(line, None, 1));
    let boxed = write_drop(
        &inbox,
        &id(2),
        &task_drop("- [ ] buy milk", Some("inbox"), 2),
    );

    let saved = phone.library.save_drops(&inbox).unwrap();
    assert_eq!(
        saved,
        DropsSaved {
            tasks: vec![line.to_owned(), "- [ ] buy milk".to_owned()],
            ..DropsSaved::default()
        }
    );
    assert!(!daily.exists() && !boxed.exists());
    let today = phone.library.today_note(now()).unwrap();
    assert_eq!(body(&phone.library, &today), format!("{line}\n"));
    assert_eq!(body(&phone.library, "note:inbox-tasks"), "- [ ] buy milk\n");

    let json: Value = serde_json::from_str(&saved.to_json()).unwrap();
    assert_eq!(json["tasks"], json!([line, "- [ ] buy milk"]));
    assert_eq!(json["notes"], json!([]));
}

// no-prop-test: each way a task drop is refused, once.
#[test]
fn unreadable_or_locked_task_drops_are_refused_and_say_so() {
    let phone = phone_with(&Arc::new(MemoryServer::new()), |store| {
        let id = store.role_book(Role::Journal, Origin::Ui).unwrap();
        lock_book(store, &id);
    });
    let inbox = inbox_of(&phone);
    let locked = write_drop(&inbox, &id(1), &task_drop("- [ ] secret plan", None, 1));
    let not_a_task = write_drop(&inbox, &id(2), &task_drop("# heading", None, 2));
    let mut no_day = task_drop("- [ ] a", None, 3);
    no_day["day"] = json!("13/11/2026");
    let no_day = write_drop(&inbox, &id(3), &no_day);
    let odd_target = write_drop(&inbox, &id(4), &task_drop("- [ ] a", Some("desk"), 4));

    let saved = phone.library.save_drops(&inbox).unwrap();
    assert!(saved.tasks.is_empty() && saved.notes.is_empty());
    assert_eq!(saved.kept, 0);
    assert_eq!(saved.problems.len(), 4, "{:?}", saved.problems);
    assert_eq!(
        saved.problems[0],
        "The Journal is locked, so a task from Shortcuts wasn't added: “secret plan”."
    );
    for problem in &saved.problems[1..] {
        assert_eq!(
            problem,
            "A task from Shortcuts couldn't be read and was discarded."
        );
    }
    for folder in [locked, not_a_task, no_day, odd_target] {
        assert!(!folder.exists());
    }
}

// no-prop-test: shares and tasks in one inbox, in time order.
#[test]
fn task_and_share_drops_are_saved_together_oldest_first() {
    let phone = lone_phone();
    let inbox = inbox_of(&phone);
    write_drop(
        &inbox,
        &id(1),
        &task_drop("- [ ] second", Some("inbox"), 20),
    );
    write_drop(
        &inbox,
        &id(2),
        &json!({"version": 1, "createdAt": 10, "text": "shared"}),
    );
    write_drop(&inbox, &id(3), &task_drop("- [ ] first", Some("inbox"), 5));
    let saved = phone.library.save_drops(&inbox).unwrap();
    assert_eq!(saved.notes, vec![format!("note:cap-{}", id(2))]);
    assert_eq!(saved.tasks, vec!["- [ ] first", "- [ ] second"]);
    assert_eq!(
        body(&phone.library, "note:inbox-tasks"),
        "- [ ] first\n- [ ] second\n"
    );
}

// no-prop-test: a library with a locked notebook and a trashed note, once.
#[test]
fn open_note_offers_no_locked_or_trashed_note() {
    let phone = phone_with(&Arc::new(MemoryServer::new()), |store| {
        store
            .put(
                DocType::Book,
                json!({"_id": "book:vault", "name": "Vault"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
        store
            .put(
                DocType::Note,
                json!({"_id": "note:secret", "title": "Pixel secret", "body": "x", "bookId": "book:vault"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
        lock_book(store, "book:vault");
        assert_eq!(
            store.role_book(Role::Inbox, Origin::Ui).unwrap(),
            "book:inbox"
        );
    });
    let library = &phone.library;
    let made = library
        .create(NewNote {
            notebook_id: Some("book:inbox".into()),
            title: "Pixel review".into(),
            body: "notes".into(),
            place: None,
            ..Default::default()
        })
        .unwrap();
    let gone = library
        .create(NewNote {
            notebook_id: Some("book:inbox".into()),
            title: "Pixel old".into(),
            body: "old".into(),
            place: None,
            ..Default::default()
        })
        .unwrap();
    library.trash(&gone.id).unwrap();

    let found = library.intent_notes("pixel", 10).unwrap();
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].id, made.id);
    assert_eq!(found[0].title, "Pixel review");
    assert!(!found[0].notebook.is_empty());
    assert!(library.intent_notes("secret", 10).unwrap().is_empty());
    assert_eq!(library.intent_notes("", 0).unwrap(), vec![]);

    let json: Value = serde_json::from_str(&intent_notes_json(&found)).unwrap();
    assert_eq!(json["notes"][0]["id"], made.id.as_str());
    assert_eq!(json["notes"][0]["title"], "Pixel review");
}

fn row(n: usize, title: String, locked: bool) -> NoteRow {
    NoteRow {
        id: format!("note:{n}"),
        rev: String::new(),
        title,
        snippet: String::new(),
        notebook_id: "book:a".into(),
        status: "none".into(),
        tags: vec![],
        pinned: false,
        created_ms: 0,
        updated_ms: 0,
        tasks_done: 0,
        tasks_total: 0,
        locked,
        canvas: false,
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Invariant: what is offered is a subsequence of the list, in order, at
    /// most the limit, never locked, and every query word is in each title.
    #[test]
    fn prop_offered_notes_are_unlocked_matches_in_order(
        rows in prop::collection::vec(("[a-cA-C ]{0,8}", any::<bool>()), 0..20),
        query in "[a-c ]{0,4}",
        limit in 0usize..80,
    ) {
        let rows: Vec<NoteRow> = rows
            .into_iter()
            .enumerate()
            .map(|(n, (title, locked))| row(n, title, locked))
            .collect();
        let books = HashMap::from([("book:a".to_owned(), "A".to_owned())]);
        let found = matching_notes(&rows, &books, &query, limit);
        prop_assert!(found.len() <= limit.min(MAX_INTENT_NOTES));
        let mut at = 0;
        for note in &found {
            let index = rows[at..].iter().position(|r| r.id == note.id).map(|i| i + at);
            prop_assert!(index.is_some(), "out of order or not listed");
            let index = index.unwrap();
            prop_assert!(!rows[index].locked);
            let title = rows[index].title.to_lowercase();
            for word in query.split_whitespace() {
                prop_assert!(title.contains(word));
            }
            prop_assert_eq!(&note.notebook, "A");
            at = index + 1;
        }
        // Nothing that could be offered is skipped below the limit.
        let eligible = rows
            .iter()
            .filter(|r| !r.locked)
            .filter(|r| query.split_whitespace().all(|w| r.title.to_lowercase().contains(w)))
            .count();
        prop_assert_eq!(found.len(), eligible.min(limit).min(MAX_INTENT_NOTES));
    }

    /// Invariant: whatever a task drop's line says, it is either written as
    /// exactly one open task line or refused; it never blocks the inbox.
    #[test]
    fn prop_any_task_drop_is_written_once_or_refused(line in "[- \\[\\]xa-z\n]{0,24}") {
        let phone = lone_phone();
        let inbox = inbox_of(&phone);
        let folder = write_drop(&inbox, &id(1), &task_drop(&line, Some("inbox"), 1));
        let saved = phone.library.save_drops(&inbox).unwrap();
        prop_assert!(!folder.exists());
        prop_assert_eq!(saved.kept, 0);
        match saved.tasks.as_slice() {
            [] => {
                prop_assert_eq!(saved.problems.len(), 1);
                prop_assert!(phone.library.note("note:inbox-tasks").is_err());
            }
            [written] => {
                prop_assert!(saved.problems.is_empty());
                prop_assert_eq!(written.as_str(), line.trim());
                let body = body(&phone.library, "note:inbox-tasks");
                prop_assert_eq!(body, format!("{written}\n"));
            }
            more => prop_assert!(false, "{more:?}"),
        }
    }
}
