//! The Tasks place: groups like the desktop's, locked notes left out, and
//! ticking with an undo.

mod common;

use std::sync::Arc;

use common::*;
use kanso_core::DocType;
use kanso_core::dates::{Date, DateTime};
use kanso_mobile::{DueTone, Library, TaskBoard, TaskGroupKind, TaskItem, TaskListQuery, Ticked};
use kanso_store::{Origin, Put, Store};
use kanso_sync::MemoryServer;
use proptest::prelude::*;
use serde_json::json;

fn now() -> DateTime {
    DateTime {
        date: Date::new(2026, 10, 20).unwrap(),
        hour: 10,
        minute: 0,
        second: 0,
    }
}

fn server() -> Arc<MemoryServer> {
    Arc::new(MemoryServer::new())
}

fn lock_book(store: &Store, id: &str) {
    store
        .put(
            DocType::Book,
            json!({"_id": id, "kansoLock": {"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "sealed"}}),
            Put::Merge,
            // As another device's lock arrives: the notebook's notes are
            // read as locked from then on.
            Origin::Sync,
        )
        .unwrap();
}

fn note_in(store: &Store, book: &str, title: &str, body: &str) -> String {
    raw_note(store, json!({"title": title, "body": body, "bookId": book}))
}

fn items(board: &TaskBoard) -> Vec<&TaskItem> {
    board.groups.iter().flat_map(|g| g.items.iter()).collect()
}

fn texts(board: &TaskBoard, group: TaskGroupKind) -> Vec<String> {
    board
        .groups
        .iter()
        .filter(|g| g.group == group)
        .flat_map(|g| g.items.iter().map(|i| i.text.clone()))
        .collect()
}

#[test]
fn tasks_are_grouped_like_the_desktop() {
    let mut work = String::new();
    let phone = phone_with(&server(), |store| {
        work = book(store, "Work");
        note_in(
            store,
            &work,
            "Plan",
            "- [ ] Late @2026-10-19\n- [ ] Now @2026-10-20T11:00\n- [ ] Next @2026-10-22\n- [ ] Plain\n- [x] Old @2026-10-01\n",
        );
    });
    let board = phone
        .library
        .tasks_at(TaskListQuery::default(), now())
        .unwrap();
    let kinds: Vec<(TaskGroupKind, &str)> = board
        .groups
        .iter()
        .map(|g| (g.group, g.label.as_str()))
        .collect();
    assert_eq!(
        kinds,
        [
            (TaskGroupKind::Overdue, "Overdue"),
            (TaskGroupKind::Today, "Today"),
            (TaskGroupKind::Upcoming, "Upcoming"),
            (TaskGroupKind::NoDate, "No date"),
        ]
    );
    let late = &board.groups[0].items[0];
    assert_eq!(late.text, "Late");
    assert_eq!(late.source_text, "Late @2026-10-19");
    assert_eq!(late.due_label.as_deref(), Some("Overdue · Yesterday"));
    assert_eq!(late.due_tone, Some(DueTone::Overdue));
    assert_eq!(late.note_title, "Plan");
    assert_eq!(late.notebook_name, "Work");
    assert_eq!(late.notebook_id, work);
    assert_eq!(late.line, 0);
    let now_item = &board.groups[1].items[0];
    assert_eq!(now_item.due_label.as_deref(), Some("Today 11:00"));
    assert_eq!(now_item.due_tone, Some(DueTone::Today));
    assert_eq!(board.groups[2].items[0].due_tone, Some(DueTone::Plain));
    let plain = &board.groups[3].items[0];
    assert_eq!((plain.due_label.clone(), plain.due_tone), (None, None));
    assert_eq!(phone.library.overdue_tasks_at(now()).unwrap(), 1);

    let all = phone
        .library
        .tasks_at(
            TaskListQuery {
                show_done: true,
                ..TaskListQuery::default()
            },
            now(),
        )
        .unwrap();
    assert_eq!(texts(&all, TaskGroupKind::Done), ["Old"]);
    assert_eq!(all.groups.last().unwrap().label, "Completed");
    assert_eq!(
        all.groups.last().unwrap().items[0].due_tone,
        Some(DueTone::Done)
    );
}

#[test]
fn a_notebook_narrows_the_tasks() {
    let (mut a, mut b) = (String::new(), String::new());
    let phone = phone_with(&server(), |store| {
        a = book(store, "A");
        b = book(store, "B");
        note_in(store, &a, "In A", "- [ ] one\n");
        note_in(store, &b, "In B", "- [ ] two\n");
    });
    let only = |book: &str| {
        let board = phone
            .library
            .tasks_at(
                TaskListQuery {
                    notebook_id: Some(book.to_owned()),
                    ..TaskListQuery::default()
                },
                now(),
            )
            .unwrap();
        items(&board)
            .iter()
            .map(|i| i.text.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(only(&a), ["one"]);
    assert_eq!(only(&b), ["two"]);
}

#[test]
fn a_locked_notebooks_tasks_are_left_out() {
    let (mut open_book, mut locked) = (String::new(), String::new());
    let phone = phone_with(&server(), |store| {
        open_book = book(store, "Open");
        locked = book(store, "Vault");
        note_in(store, &open_book, "Visible", "- [ ] seen @2026-10-01\n");
        note_in(
            store,
            &locked,
            "VAULTTITLE",
            "- [ ] VAULTTASK @2026-10-01\n- [ ] VAULTTWO\n",
        );
        lock_book(store, &locked);
    });
    let query = TaskListQuery {
        show_done: true,
        ..TaskListQuery::default()
    };
    let board = phone.library.tasks_at(query, now()).unwrap();
    let text = format!("{board:?}");
    assert!(!text.contains("VAULT"), "{text}");
    assert_eq!(texts(&board, TaskGroupKind::Overdue), ["seen"]);
    assert_eq!(phone.library.overdue_tasks_at(now()).unwrap(), 1);
    // The locked notebook's own filter is empty.
    let filtered = phone
        .library
        .tasks_at(
            TaskListQuery {
                notebook_id: Some(locked),
                ..TaskListQuery::default()
            },
            now(),
        )
        .unwrap();
    assert!(filtered.groups.is_empty());
}

#[test]
fn a_sealed_notes_tasks_are_left_out_open_or_closed() {
    let phone = lone_phone();
    note(&phone.library, "Visible", "- [ ] seen @2026-10-01\n");
    let sealed = note(
        &phone.library,
        "SEALEDTITLE",
        "- [ ] SEALEDTASK @2026-10-01\n",
    );
    let locks = phone.library.locks();
    locks.lock_note(&sealed, LOCK_PASSWORD.into()).unwrap();
    for session_open in [true, false, true] {
        assert_eq!(locks.is_unlocked().unwrap(), session_open);
        let board = phone
            .library
            .tasks_at(
                TaskListQuery {
                    show_done: true,
                    ..TaskListQuery::default()
                },
                now(),
            )
            .unwrap();
        let text = format!("{board:?}");
        assert!(!text.contains("SEALED"), "{text}");
        assert_eq!(texts(&board, TaskGroupKind::Overdue), ["seen"]);
        assert_eq!(phone.library.overdue_tasks_at(now()).unwrap(), 1);
        if session_open {
            locks.lock().unwrap();
        } else {
            assert!(locks.unlock(LOCK_PASSWORD.into()).unwrap());
        }
    }
}

fn single(library: &Library) -> TaskItem {
    let board = library.tasks_at(TaskListQuery::default(), now()).unwrap();
    items(&board)[0].clone()
}

#[test]
fn ticking_from_the_board_keeps_the_task_and_undo_puts_it_back() {
    let phone = phone_with(&server(), |store| {
        let b = book(store, "A");
        note_in(store, &b, "N", "- [ ] Pay @2026-10-19\n- [ ] Other\n");
    });
    let library = &phone.library;
    let item = single(library);
    assert_eq!(item.text, "Pay");
    let Ticked::Changed { undo, message, .. } = library
        .set_task(&item.note_id, item.line, &item.source_text, true)
        .unwrap()
    else {
        panic!("not ticked");
    };
    // The notice says it in the desktop's words, without the date.
    assert_eq!(message, "Ticked off “Pay”");
    // Read again without the keep: it is gone, and the badge drops.
    let board = library.tasks_at(TaskListQuery::default(), now()).unwrap();
    assert_eq!(texts(&board, TaskGroupKind::Overdue), Vec::<String>::new());
    assert_eq!(library.overdue_tasks_at(now()).unwrap(), 0);
    // Kept: it stays in Overdue, done and struck.
    let kept = library
        .tasks_at(
            TaskListQuery {
                kept: vec![item.key.clone()],
                ..TaskListQuery::default()
            },
            now(),
        )
        .unwrap();
    let ticked = &kept.groups[0];
    assert_eq!(ticked.group, TaskGroupKind::Overdue);
    assert!(ticked.items[0].checked);
    assert_eq!(ticked.items[0].key, item.key);
    // With done shown it is in Completed.
    let shown = library
        .tasks_at(
            TaskListQuery {
                show_done: true,
                ..TaskListQuery::default()
            },
            now(),
        )
        .unwrap();
    assert_eq!(texts(&shown, TaskGroupKind::Done), ["Pay"]);
    // Undo puts it back, open and overdue.
    assert_eq!(library.undo(undo).unwrap(), 1);
    let back = single(library);
    assert_eq!((back.text.as_str(), back.checked), ("Pay", false));
    assert_eq!(library.overdue_tasks_at(now()).unwrap(), 1);
    assert_eq!(library.undo(undo).unwrap(), 0);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    /// Whatever the notes hold, no task of a locked notebook or sealed
    /// note comes back, the sections hold exactly the unlocked open tasks,
    /// and the badge equals the Overdue section.
    #[test]
    fn prop_locked_tasks_never_come_back(
        notes in prop::collection::vec(
            (0usize..3, prop::collection::vec((prop::option::of(15u32..26), any::<bool>()), 0..4)),
            0..7,
        ),
        show_done in any::<bool>(),
    ) {
        let mut ids = Vec::new();
        let phone = phone_with(&server(), |store| {
            let books: Vec<String> = ["A", "B", "L"].iter().map(|n| book(store, n)).collect();
            for (n, (b, tasks)) in notes.iter().enumerate() {
                let body: String = tasks.iter().enumerate().map(|(t, (day, done))| {
                    let mark = if *done { "x" } else { " " };
                    match day {
                        Some(d) => format!("- [{mark}] n{n}t{t} @2026-10-{d:02}\n"),
                        None => format!("- [{mark}] n{n}t{t}\n"),
                    }
                }).collect();
                ids.push(note_in(store, &books[*b], &format!("title{n}"), &body));
            }
            lock_book(store, &books[2]);
        });
        let board = phone.library.tasks_at(
            TaskListQuery { show_done, ..TaskListQuery::default() },
            now(),
        ).unwrap();
        let mut expected = 0;
        let mut expected_overdue = 0;
        for (b, tasks) in &notes {
            if *b == 2 { continue; }
            for (day, done) in tasks {
                if *done && !show_done { continue; }
                expected += 1;
                if !*done && day.is_some_and(|d| d < 20) { expected_overdue += 1; }
            }
        }
        prop_assert_eq!(items(&board).len(), expected);
        for item in items(&board) {
            let n: usize = item.text[1..].split('t').next().unwrap().parse().unwrap();
            prop_assert!(notes[n].0 != 2, "{:?}", item);
        }
        prop_assert_eq!(phone.library.overdue_tasks_at(now()).unwrap(), expected_overdue);
        // Groups in the desktop's order, none empty.
        let order: Vec<usize> = board.groups.iter().map(|g| match g.group {
            TaskGroupKind::Overdue => 0, TaskGroupKind::Today => 1, TaskGroupKind::Upcoming => 2,
            TaskGroupKind::NoDate => 3, TaskGroupKind::Done => 4,
        }).collect();
        prop_assert!(order.windows(2).all(|w| w[0] < w[1]));
        prop_assert!(board.groups.iter().all(|g| !g.items.is_empty()));
    }
}
