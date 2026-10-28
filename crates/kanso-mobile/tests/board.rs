//! Boards over the notes a list shows: notes by status, tasks by box, a
//! locked note kept to its title, and moves that change one thing.

mod common;

use std::sync::Arc;

use common::*;
use kanso_core::NoteStatus;
use kanso_mobile::{
    BoardCard, BoardKind, BoardLane, BoardView, CardKey, CardMoved, KansoError, ListPlace,
    ListQuery,
};
use kanso_store::Store;
use kanso_sync::lock::{LockKey, LockPassword};
use kanso_sync::{KdfParams, MemoryServer};
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

fn status(name: &str) -> BoardLane {
    BoardLane::Status {
        status: name.into(),
    }
}

fn column<'a>(board: &'a BoardView, lane: &BoardLane) -> &'a [BoardCard] {
    &board
        .columns
        .iter()
        .find(|c| c.lane == *lane)
        .expect("a column")
        .cards
}

fn titles(cards: &[BoardCard]) -> Vec<&str> {
    cards.iter().map(|c| c.title.as_str()).collect()
}

fn is_note(card: &BoardCard, id: &str) -> bool {
    matches!(&card.key, CardKey::Note { id: found } if found == id)
}

fn raw_status_note(store: &Store, title: &str, body: &str, status: &str) -> String {
    raw_note(
        store,
        json!({"title": title, "body": body, "bookId": "book:a", "status": status}),
    )
}

fn seeded() -> (Phone, [String; 3]) {
    let mut ids = [String::new(), String::new(), String::new()];
    let phone = phone_with(&Arc::new(MemoryServer::new()), |store| {
        store
            .put(
                kanso_core::DocType::Book,
                json!({"_id": "book:a", "name": "A"}),
                kanso_store::Put::Upsert,
                kanso_store::Origin::Ui,
            )
            .unwrap();
        ids[0] = raw_status_note(store, "Alpha", "alpha\n- [ ] one\n- [x] two\n", "active");
        ids[1] = raw_status_note(
            store,
            "Beta",
            "beta\n- [ ] three @2026-10-25\n",
            "completed",
        );
        ids[2] = raw_status_note(store, "Gamma", "gamma", "none");
    });
    (phone, ids)
}

// no-prop-test: a fixed library and the columns it must make.
#[test]
fn a_notes_board_has_a_column_per_status_without_the_trash() {
    let (phone, ids) = seeded();
    let library = &phone.library;
    let board = library
        .board(BoardKind::Notes, ListQuery::default())
        .unwrap();
    let lanes: Vec<&str> = board.columns.iter().map(|c| c.label.as_str()).collect();
    assert_eq!(
        lanes,
        ["No status", "Active", "On hold", "Completed", "Dropped"]
    );
    assert_eq!(titles(column(&board, &status("active"))), ["Alpha"]);
    assert_eq!(titles(column(&board, &status("completed"))), ["Beta"]);
    assert_eq!(titles(column(&board, &status("none"))), ["Gamma"]);
    let card = &column(&board, &status("active"))[0];
    assert_eq!(card.key, CardKey::Note { id: ids[0].clone() });
    assert!(card.snippet.contains("alpha"));
    assert!(!card.locked && card.due.is_none());
    // The trash never shows; a list narrows the board as it narrows the list.
    library.trash(&ids[0]).unwrap();
    let board = library
        .board(BoardKind::Notes, ListQuery::default())
        .unwrap();
    assert!(column(&board, &status("active")).is_empty());
    let trash = ListQuery {
        place: ListPlace::Trash,
        ..ListQuery::default()
    };
    let board = library.board(BoardKind::Notes, trash).unwrap();
    assert!(board.columns.iter().all(|c| c.cards.is_empty()));
    let only = ListQuery {
        status_only: Some("completed".into()),
        ..ListQuery::default()
    };
    let board = library.board(BoardKind::Notes, only).unwrap();
    assert_eq!(
        board.columns.iter().map(|c| c.cards.len()).sum::<usize>(),
        1
    );
    let bad = ListQuery {
        status_only: Some("nonsense".into()),
        ..ListQuery::default()
    };
    assert!(matches!(
        library.board(BoardKind::Notes, bad),
        Err(KansoError::Invalid { .. })
    ));
}

// no-prop-test: a fixed library and the cards it must make.
#[test]
fn a_tasks_board_has_to_do_and_done_in_the_notes_order() {
    let (phone, ids) = seeded();
    let library = &phone.library;
    let query = ListQuery {
        sort: kanso_mobile::SortKey::Title,
        descending: false,
        ..ListQuery::default()
    };
    let board = library.board(BoardKind::Tasks, query.clone()).unwrap();
    assert_eq!(
        board
            .columns
            .iter()
            .map(|c| c.label.as_str())
            .collect::<Vec<_>>(),
        ["To do", "Done"]
    );
    assert_eq!(titles(column(&board, &BoardLane::Todo)), ["one", "three"]);
    assert_eq!(titles(column(&board, &BoardLane::Done)), ["two"]);
    let three = &column(&board, &BoardLane::Todo)[1];
    assert_eq!(three.due.as_deref(), Some("2026-10-25"));
    assert_eq!(three.note_title, "Beta");
    assert_eq!(
        three.key,
        CardKey::Task {
            note_id: ids[1].clone(),
            line: 1,
            text: "three @2026-10-25".into()
        }
    );
    // Only the tasks of the notes the list shows.
    let one = ListQuery {
        status_only: Some("active".into()),
        ..query
    };
    let board = library.board(BoardKind::Tasks, one).unwrap();
    assert_eq!(titles(column(&board, &BoardLane::Todo)), ["one"]);
}

/// A library with a sealed note and two notes in a locked notebook: one that
/// was there before the lock and one that arrived from another device after.
fn with_locked() -> (Phone, String, String) {
    let phone = phone_with(&Arc::new(MemoryServer::new()), |store| {
        raw_note(
            store,
            json!({"title": "Open", "body": "OPENTEXT\n- [ ] open task @2026-10-22T09:00\n", "status": "active"}),
        );
    });
    let library = &phone.library;
    let sealed = note(
        library,
        "Sealed",
        "SEALEDTEXT\n- [ ] sealed task @2026-10-22T09:00\n",
    );
    library
        .locks()
        .lock_note(&sealed, LOCK_PASSWORD.into())
        .unwrap();
    let store = Store::open(phone.path()).unwrap();
    let vault = book(&store, "Vault");
    let key = LockKey::create(&LockPassword::new(LOCK_PASSWORD).unwrap(), KdfParams::TEST).unwrap();
    // Locking the notebook later, around a note that is not sealed yet.
    let loose = raw_note(
        &store,
        json!({"title": "Loose", "body": "LOOSETEXT\n- [ ] loose task @2026-10-22T09:00\n", "bookId": vault}),
    );
    kanso_engine::lock::seal_notebook(&store, &vault, &key).unwrap();
    let again = store
        .put(
            kanso_core::DocType::Note,
            json!({"title": "Slipped in", "body": "SLIPPEDTEXT\n- [ ] slipped task\n", "bookId": vault}),
            kanso_store::Put::Upsert,
            kanso_store::Origin::Sync,
        )
        .unwrap()
        .id;
    let _ = loose;
    (phone, sealed, again)
}

// no-prop-test: a fixed library; the locked-note property is in tests/locked.rs.
#[test]
fn a_locked_note_is_a_card_with_its_title_and_no_text_or_tasks() {
    let (phone, sealed, slipped) = with_locked();
    let library = &phone.library;
    for open_session in [true, false] {
        if !open_session {
            library.locks().lock().unwrap();
        }
        let notes = library
            .board(BoardKind::Notes, ListQuery::default())
            .unwrap();
        let tasks = library
            .board(BoardKind::Tasks, ListQuery::default())
            .unwrap();
        let seen = format!("{notes:?}{tasks:?}");
        for secret in [
            "SEALEDTEXT",
            "LOOSETEXT",
            "SLIPPEDTEXT",
            "sealed task",
            "loose task",
            "slipped task",
        ] {
            assert!(!seen.contains(secret), "{secret} in {seen}");
        }
        let card = column(&notes, &status("none"))
            .iter()
            .find(|c| c.key == CardKey::Note { id: sealed.clone() })
            .expect("the locked note is a card");
        assert!(card.locked && card.snippet.is_empty());
        assert_eq!(card.title, "Sealed");
        // The plain note's task is there, the locked ones are not.
        assert_eq!(titles(column(&tasks, &BoardLane::Todo)), ["open task"]);
    }
    // The status of a locked note isn't changed from the board.
    for id in [&sealed, &slipped] {
        assert_eq!(
            phone
                .library
                .move_card(CardKey::Note { id: id.clone() }, status("active"), 5),
            Err(KansoError::Locked)
        );
    }
}

// no-prop-test: a fixed table of the moves a board refuses or skips.
#[test]
fn moves_that_do_nothing_or_cannot_be_made() {
    let (phone, ids) = seeded();
    let library = &phone.library;
    let note = |id: &str| CardKey::Note { id: id.into() };
    assert_eq!(
        library.move_card(note(&ids[0]), status("active"), 5),
        Ok(CardMoved::Unchanged)
    );
    assert_eq!(
        library.move_card(note(&ids[0]), status("dropped"), 5),
        Ok(CardMoved::Changed)
    );
    assert_eq!(library.note(&ids[0]).unwrap().status, "dropped");
    assert!(matches!(
        library.move_card(note(&ids[0]), BoardLane::Done, 5),
        Err(KansoError::Invalid { .. })
    ));
    assert!(matches!(
        library.move_card(note(&ids[0]), status("nonsense"), 5),
        Err(KansoError::Invalid { .. })
    ));
    assert_eq!(
        library.move_card(note("note:nowhere"), status("active"), 5),
        Err(KansoError::NotFound)
    );
    let task = |text: &str, line: u32| CardKey::Task {
        note_id: ids[0].clone(),
        line,
        text: text.into(),
    };
    assert_eq!(
        library.move_card(task("one", 1), BoardLane::Todo, 5),
        Ok(CardMoved::Unchanged)
    );
    assert_eq!(
        library.move_card(task("one", 1), BoardLane::Done, 5),
        Ok(CardMoved::Changed)
    );
    assert_eq!(
        library.note(&ids[0]).unwrap().body.as_deref(),
        Some("alpha\n- [x] one\n- [x] two\n")
    );
    // Gone, or changed since the board was read: nothing is written.
    assert_eq!(
        library.move_card(task("not there", 1), BoardLane::Done, 5),
        Ok(CardMoved::Missing)
    );
    assert_eq!(
        library.move_card(task("one", 7), BoardLane::Todo, 5),
        Ok(CardMoved::Missing)
    );
    assert!(matches!(
        library.move_card(task("one", 1), status("active"), 5),
        Err(KansoError::Invalid { .. })
    ));
}

proptest! {
    #![proptest_config(config(24))]

    /// Model-based: any moves of the notes between status columns change only
    /// the moved note's status; titles, texts and notebooks stay, and the
    /// board read again has each note in the column its status says.
    #[test]
    fn prop_note_moves_change_only_the_status(
        moves in prop::collection::vec((0usize..3, 0usize..5), 0..16),
    ) {
        let (phone, ids) = seeded();
        let library = &phone.library;
        let names = ["none", "active", "onHold", "completed", "dropped"];
        let mut model: Vec<String> = ids
            .iter()
            .map(|id| library.note(id).unwrap().status)
            .collect();
        let texts: Vec<_> = ids.iter().map(|id| library.note(id).unwrap().body).collect();
        for (note, lane) in moves {
            let moved = library
                .move_card(CardKey::Note { id: ids[note].clone() }, status(names[lane]), 9)
                .unwrap();
            let expected = if model[note] == names[lane] {
                CardMoved::Unchanged
            } else {
                CardMoved::Changed
            };
            prop_assert_eq!(moved, expected);
            model[note] = names[lane].to_owned();
            let board = library
                .board(BoardKind::Notes, ListQuery::default())
                .unwrap();
            for (i, id) in ids.iter().enumerate() {
                let view = library.note(id).unwrap();
                prop_assert_eq!(&view.status, &model[i]);
                prop_assert_eq!(&view.body, &texts[i]);
                let lane = column(&board, &status(&model[i]));
                prop_assert!(lane.iter().any(|card| is_note(card, id)));
            }
            let cards: usize = board.columns.iter().map(|c| c.cards.len()).sum();
            prop_assert_eq!(cards, ids.len());
        }
    }

    /// Model-based, for tasks: dragging between To do and Done ticks the
    /// task's box in its note and changes nothing else in the note.
    #[test]
    fn prop_task_moves_change_only_the_box(
        checked in prop::collection::vec(any::<bool>(), 1..7),
        drags in prop::collection::vec((0usize..7, any::<bool>()), 0..16),
    ) {
        let body = |boxes: &[bool]| -> String {
            std::iter::once("intro\n".to_owned())
                .chain(boxes.iter().enumerate().map(|(i, c)| {
                    format!("- [{}] task {i}\n", if *c { "x" } else { " " })
                }))
                .collect()
        };
        let phone = lone_phone();
        let library = &phone.library;
        let id = note(library, "Tasks", &body(&checked));
        let mut model = checked.clone();
        for (task, done) in drags {
            let task = task % checked.len();
            let key = CardKey::Task {
                note_id: id.clone(),
                line: task as u32 + 1,
                text: format!("task {task}"),
            };
            let lane = if done { BoardLane::Done } else { BoardLane::Todo };
            let moved = library.move_card(key, lane, 9).unwrap();
            prop_assert_eq!(
                moved,
                if model[task] == done { CardMoved::Unchanged } else { CardMoved::Changed }
            );
            model[task] = done;
            prop_assert_eq!(library.note(&id).unwrap().body.unwrap(), body(&model));
            let board = library
                .board(BoardKind::Tasks, ListQuery::default())
                .unwrap();
            let in_done = column(&board, &BoardLane::Done).len();
            prop_assert_eq!(in_done, model.iter().filter(|c| **c).count());
            prop_assert_eq!(
                column(&board, &BoardLane::Todo).len() + in_done,
                model.len()
            );
        }
    }
}

// no-prop-test: lanes are what NoteStatus lists; checked against the library's own names.
#[test]
fn the_status_columns_are_the_notes_statuses() {
    let phone = lone_phone();
    let board = phone
        .library
        .board(BoardKind::Notes, ListQuery::default())
        .unwrap();
    let lanes: Vec<BoardLane> = board.columns.iter().map(|c| c.lane.clone()).collect();
    let expected: Vec<BoardLane> = NoteStatus::ALL.iter().map(|s| status(s.as_str())).collect();
    assert_eq!(lanes, expected);
}
