//! A notebook's board grouped by a property: its lanes come from the core,
//! a move writes the property, the settings group, rename, reorder, add and
//! remove lanes, and an undo puts it all back.

mod common;

use std::sync::Arc;

use common::*;
use kanso_core::schema::{BoardSpec, PropertyDef, PropertyType};
use kanso_mobile::{
    BoardKind, BoardLane, BoardView, CardKey, CardMoved, KansoError, LaneChange, ListPlace,
    ListQuery,
};
use kanso_store::{Origin, Store};
use kanso_sync::MemoryServer;
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

fn value(name: &str) -> BoardLane {
    BoardLane::Value { value: name.into() }
}

fn names(of: &[&str]) -> Vec<String> {
    of.iter().map(|s| (*s).to_owned()).collect()
}

fn in_book(id: &str) -> ListQuery {
    ListQuery {
        place: ListPlace::Notebook { id: id.into() },
        ..ListQuery::default()
    }
}

fn note_body(stage: Option<&str>) -> String {
    match stage {
        Some(stage) => format!("---\nstage: {stage}\n---\nbody"),
        None => "body".to_owned(),
    }
}

/// A notebook "Studio" with a `stage` select board of Idea, Scripting and
/// Shooting, and one note in each of Idea, Shooting, none and Gone (a value
/// no lane has).
fn studio() -> (Phone, String, Vec<String>) {
    let mut book_id = String::new();
    let mut ids = Vec::new();
    let phone = phone_with(&Arc::new(MemoryServer::new()), |store| {
        book_id = book(store, "Studio");
        store
            .set_notebook_properties(
                &book_id,
                &[PropertyDef {
                    key: "stage".into(),
                    kind: PropertyType::Select,
                    options: names(&["Idea", "Scripting", "Shooting"]),
                }],
                Origin::Ui,
            )
            .unwrap();
        store
            .set_notebook_board(
                &book_id,
                Some(&BoardSpec {
                    by: "stage".into(),
                    lanes: Vec::new(),
                    show_unset: true,
                }),
                Origin::Ui,
            )
            .unwrap();
        for (title, stage) in [
            ("Pixel", Some("Idea")),
            ("Tour", Some("Shooting")),
            ("Loose", None),
            ("Odd", Some("Gone")),
        ] {
            ids.push(raw_note(
                store,
                json!({"title": title, "body": note_body(stage), "bookId": book_id}),
            ));
        }
    });
    (phone, book_id, ids)
}

fn lane<'a>(board: &'a BoardView, lane: &BoardLane) -> Vec<&'a str> {
    board
        .columns
        .iter()
        .find(|c| c.lane == *lane)
        .unwrap_or_else(|| panic!("no column {lane:?} in {:?}", board.columns))
        .cards
        .iter()
        .map(|c| c.title.as_str())
        .collect()
}

fn note_key(id: &str) -> CardKey {
    CardKey::Note { id: id.into() }
}

// no-prop-test: a fixed notebook and the lanes the engine must make of it.
#[test]
fn a_notebooks_board_has_the_lanes_of_its_property() {
    let (phone, book, _) = studio();
    let board = phone
        .library
        .board(BoardKind::Notes, in_book(&book))
        .unwrap();
    let lanes: Vec<(&BoardLane, &str)> = board
        .columns
        .iter()
        .map(|c| (&c.lane, c.label.as_str()))
        .collect();
    assert_eq!(
        lanes,
        [
            (&value("Idea"), "Idea"),
            (&value("Scripting"), "Scripting"),
            (&value("Shooting"), "Shooting"),
            (&BoardLane::Unlisted, "Unlisted stage"),
            (&BoardLane::Unset, "No stage"),
        ]
    );
    assert_eq!(lane(&board, &value("Idea")), ["Pixel"]);
    assert_eq!(lane(&board, &value("Shooting")), ["Tour"]);
    assert_eq!(lane(&board, &BoardLane::Unlisted), ["Odd"]);
    assert_eq!(lane(&board, &BoardLane::Unset), ["Loose"]);
    let setup = board.setup.expect("the notebook's settings");
    assert_eq!(setup.book_id, book);
    assert_eq!(setup.group.as_deref(), Some("stage"));
    assert_eq!(setup.groupable, ["stage"]);
    assert_eq!(setup.lanes, ["Idea", "Scripting", "Shooting"]);
    assert!(setup.show_unset);
}

// no-prop-test: other places and the tasks board keep their own lanes.
#[test]
fn other_boards_keep_the_status_and_task_lanes() {
    let (phone, book, _) = studio();
    let all = phone
        .library
        .board(BoardKind::Notes, ListQuery::default())
        .unwrap();
    assert!(all.setup.is_none());
    assert!(
        all.columns
            .iter()
            .all(|c| matches!(c.lane, BoardLane::Status { .. }))
    );
    let tasks = phone
        .library
        .board(BoardKind::Tasks, in_book(&book))
        .unwrap();
    assert!(tasks.setup.is_none());
    assert_eq!(tasks.columns.len(), 2);
}

// no-prop-test: a notebook with nothing to group by has no settings.
#[test]
fn a_notebook_without_a_select_property_has_nothing_to_set() {
    let mut plain = String::new();
    let phone = phone_with(&Arc::new(MemoryServer::new()), |store| {
        plain = book(store, "Plain");
    });
    let board = phone
        .library
        .board(BoardKind::Notes, in_book(&plain))
        .unwrap();
    assert!(board.setup.is_none());
    assert_eq!(board.columns.len(), 5);
}

// no-prop-test: the moves a property board makes, refuses or skips.
#[test]
fn a_move_writes_the_property_and_can_be_taken_back() {
    let (phone, book, ids) = studio();
    let library = &phone.library;
    let moved = library
        .move_card(note_key(&ids[0]), value("Scripting"), 5)
        .unwrap();
    let CardMoved::Changed { message, undo } = moved else {
        panic!("{moved:?}");
    };
    assert_eq!(message.as_deref(), Some("Moved to Scripting."));
    assert!(
        library
            .note(&ids[0])
            .unwrap()
            .body
            .unwrap()
            .contains("stage: Scripting")
    );
    let board = library.board(BoardKind::Notes, in_book(&book)).unwrap();
    assert_eq!(lane(&board, &value("Scripting")), ["Pixel"]);
    assert_eq!(library.undo(undo.expect("a ticket")).unwrap(), 1);
    let board = library.board(BoardKind::Notes, in_book(&book)).unwrap();
    assert_eq!(lane(&board, &value("Idea")), ["Pixel"]);
    // Nothing to write, nothing to take back.
    assert_eq!(
        library.move_card(note_key(&ids[0]), value("Idea"), 5),
        Ok(CardMoved::Unchanged)
    );
    // No value is the property left empty.
    assert!(matches!(
        library.move_card(note_key(&ids[0]), BoardLane::Unset, 5),
        Ok(CardMoved::Changed { .. })
    ));
    let board = library.board(BoardKind::Notes, in_book(&book)).unwrap();
    let mut unset = lane(&board, &BoardLane::Unset);
    unset.sort_unstable();
    assert_eq!(unset, ["Loose", "Pixel"]);
    // The unlisted lane and lanes the board lacks are refused.
    for to in [BoardLane::Unlisted, value("Nowhere"), BoardLane::Done] {
        assert!(
            matches!(
                library.move_card(note_key(&ids[1]), to.clone(), 5),
                Err(KansoError::Invalid { .. })
            ),
            "{to:?}"
        );
    }
}

// no-prop-test: grouping is saved on the notebook and keeps its lanes.
#[test]
fn grouping_and_the_empty_lane_are_saved_with_the_notebook() {
    let (phone, book, _) = studio();
    let library = &phone.library;
    library.set_board_show_unset(&book, false).unwrap();
    let board = library.board(BoardKind::Notes, in_book(&book)).unwrap();
    // "Loose" still has no stage, so its lane shows although it is off.
    assert!(board.columns.iter().any(|c| c.lane == BoardLane::Unset));
    assert!(!board.setup.as_ref().unwrap().show_unset);
    // Back to status lanes, and again by the same property: the lanes and
    // the setting come back with it.
    library.set_board_group(&book, None).unwrap();
    let board = library.board(BoardKind::Notes, in_book(&book)).unwrap();
    assert!(
        board
            .columns
            .iter()
            .all(|c| matches!(c.lane, BoardLane::Status { .. }))
    );
    let setup = board.setup.unwrap();
    assert_eq!(setup.group, None);
    assert_eq!(setup.groupable, ["stage"]);
    library
        .set_board_group(&book, Some("stage".into()))
        .unwrap();
    let board = library.board(BoardKind::Notes, in_book(&book)).unwrap();
    assert_eq!(
        board.setup.unwrap().lanes,
        ["Idea", "Scripting", "Shooting"]
    );
    // Only a select property of the notebook can group it.
    assert!(matches!(
        library.set_board_group(&book, Some("owner".into())),
        Err(KansoError::Invalid { .. })
    ));
    library.set_board_group(&book, None).unwrap();
    assert!(matches!(
        library.set_board_show_unset(&book, true),
        Err(KansoError::Invalid { .. })
    ));
}

// no-prop-test: each lane edit's effect on the notes and the board, with its words.
#[test]
fn lane_edits_rewrite_the_notes_and_say_what_they_did() {
    let (phone, book, ids) = studio();
    let library = &phone.library;
    let edited = library
        .edit_board_lanes(
            &book,
            LaneChange::Rename {
                from: "Shooting".into(),
                to: " Filming ".into(),
            },
            5,
        )
        .unwrap();
    assert_eq!(edited.lanes, ["Idea", "Scripting", "Filming"]);
    assert_eq!(edited.rewritten, 1);
    assert_eq!(
        edited.message,
        "Renamed Shooting to Filming. 1 note updated."
    );
    let board = library.board(BoardKind::Notes, in_book(&book)).unwrap();
    assert_eq!(lane(&board, &value("Filming")), ["Tour"]);
    assert!(
        library
            .note(&ids[1])
            .unwrap()
            .body
            .unwrap()
            .contains("stage: Filming")
    );

    let edited = library
        .edit_board_lanes(&book, LaneChange::Move { from: 2, to: 0 }, 6)
        .unwrap();
    assert_eq!(edited.lanes, ["Filming", "Idea", "Scripting"]);
    assert_eq!(edited.message, "Moved a lane.");

    let edited = library
        .edit_board_lanes(
            &book,
            LaneChange::Add {
                name: "Editing".into(),
            },
            7,
        )
        .unwrap();
    assert_eq!(edited.lanes, ["Filming", "Idea", "Scripting", "Editing"]);
    let board = library.board(BoardKind::Notes, in_book(&book)).unwrap();
    assert!(lane(&board, &value("Editing")).is_empty());

    // Removing a lane moves its notes where asked.
    let removal = library.board_lane_removal(&book, "Filming").unwrap();
    assert_eq!(removal.notes, 1);
    assert_eq!(removal.prompt, "1 note is in Filming. Move it to:");
    assert_eq!(removal.target.as_deref(), Some("Idea"));
    assert_eq!(removal.targets, ["Idea", "Scripting", "Editing"]);
    let edited = library
        .edit_board_lanes(
            &book,
            LaneChange::Remove {
                lane: "Filming".into(),
                move_to: Some("Editing".into()),
            },
            8,
        )
        .unwrap();
    assert_eq!(edited.message, "Removed Filming. 1 note moved to Editing.");
    let board = library.board(BoardKind::Notes, in_book(&book)).unwrap();
    assert_eq!(lane(&board, &value("Editing")), ["Tour"]);
    // An empty lane goes with no question; with no target the notes have no value.
    let removal = library.board_lane_removal(&book, "Scripting").unwrap();
    assert_eq!((removal.notes, removal.prompt.as_str()), (0, ""));
    let edited = library
        .edit_board_lanes(
            &book,
            LaneChange::Remove {
                lane: "Editing".into(),
                move_to: None,
            },
            9,
        )
        .unwrap();
    assert_eq!(edited.message, "Removed Editing. 1 note moved to No stage.");
    let board = library.board(BoardKind::Notes, in_book(&book)).unwrap();
    let mut unset = lane(&board, &BoardLane::Unset);
    unset.sort_unstable();
    assert_eq!(unset, ["Loose", "Tour"]);
}

// no-prop-test: the edits the core refuses, with its words, and that nothing is written.
#[test]
fn refused_lane_edits_write_nothing() {
    let (phone, book, _) = studio();
    let library = &phone.library;
    let before = library.board(BoardKind::Notes, in_book(&book)).unwrap();
    let refused = |change| match library.edit_board_lanes(&book, change, 5) {
        Err(KansoError::Invalid { message }) => message,
        other => panic!("{other:?}"),
    };
    assert_eq!(
        refused(LaneChange::Add { name: "  ".into() }),
        "A lane needs a name."
    );
    assert_eq!(
        refused(LaneChange::Add {
            name: "Idea".into()
        }),
        "There is a lane called Idea already."
    );
    assert_eq!(
        refused(LaneChange::Rename {
            from: "Idea".into(),
            to: "Shooting".into()
        }),
        "There is a lane called Shooting already."
    );
    assert_eq!(
        refused(LaneChange::Move { from: 0, to: 9 }),
        "That lane isn't on the board."
    );
    assert_eq!(
        refused(LaneChange::Remove {
            lane: "Idea".into(),
            move_to: Some("Nowhere".into())
        }),
        "Nowhere isn't a lane to move the notes to."
    );
    assert_eq!(
        library.board(BoardKind::Notes, in_book(&book)).unwrap(),
        before
    );
}

// no-prop-test: undo of one rename of a lane with notes.
#[test]
fn a_lane_edit_can_be_taken_back() {
    let (phone, book, ids) = studio();
    let library = &phone.library;
    let before = library.board(BoardKind::Notes, in_book(&book)).unwrap();
    let body = library.note(&ids[0]).unwrap().body;
    let edited = library
        .edit_board_lanes(
            &book,
            LaneChange::Remove {
                lane: "Idea".into(),
                move_to: Some("Scripting".into()),
            },
            5,
        )
        .unwrap();
    assert_eq!(library.undo(edited.undo).unwrap(), 1);
    assert_eq!(library.note(&ids[0]).unwrap().body, body);
    assert_eq!(
        library.board(BoardKind::Notes, in_book(&book)).unwrap(),
        before
    );
    // A ticket is used once.
    assert_eq!(library.undo(edited.undo).unwrap(), 0);
}

// no-prop-test: the only lane of a board stays.
#[test]
fn the_last_lane_cannot_be_removed() {
    let mut book_id = String::new();
    let phone = phone_with(&Arc::new(MemoryServer::new()), |store: &Store| {
        book_id = book(store, "One");
        store
            .set_notebook_properties(
                &book_id,
                &[PropertyDef {
                    key: "stage".into(),
                    kind: PropertyType::Select,
                    options: names(&["Only"]),
                }],
                Origin::Ui,
            )
            .unwrap();
        store
            .set_notebook_board(
                &book_id,
                Some(&BoardSpec {
                    by: "stage".into(),
                    lanes: Vec::new(),
                    show_unset: false,
                }),
                Origin::Ui,
            )
            .unwrap();
    });
    assert_eq!(
        phone.library.edit_board_lanes(
            &book_id,
            LaneChange::Remove {
                lane: "Only".into(),
                move_to: None
            },
            5
        ),
        Err(KansoError::Invalid {
            message: "A board needs at least one lane.".into()
        })
    );
}

fn is_invalid<T>(result: &Result<T, KansoError>) -> bool {
    matches!(result, Err(KansoError::Invalid { .. }))
}

#[derive(Clone, Debug)]
enum Step {
    Rename(usize, u8),
    Move(usize, usize),
    Add(u8),
    Remove(usize, Option<usize>),
    Drag(usize, usize),
}

fn step() -> impl Strategy<Value = Step> {
    prop_oneof![
        (0usize..6, 0u8..8).prop_map(|(lane, name)| Step::Rename(lane, name)),
        (0usize..6, 0usize..6).prop_map(|(from, to)| Step::Move(from, to)),
        (0u8..8).prop_map(Step::Add),
        (0usize..6, prop::option::of(0usize..6)).prop_map(|(lane, to)| Step::Remove(lane, to)),
        (0usize..4, 0usize..7).prop_map(|(note, lane)| Step::Drag(note, lane)),
    ]
}

proptest! {
    #![proptest_config(config(24))]

    /// Model-based: any run of lane edits and drags leaves every note in
    /// exactly one lane, the board's value lanes are the model's lanes in
    /// order, and no note is left in a lane the board does not have (a
    /// value typed by hand stays "unlisted", as written).
    #[test]
    fn prop_edits_and_moves_keep_each_note_in_one_lane(
        steps in prop::collection::vec(step(), 0..14),
    ) {
        let (phone, book, ids) = studio();
        let library = &phone.library;
        let mut lanes = names(&["Idea", "Scripting", "Shooting"]);
        // Each note's value; "Gone" is the one no lane has.
        let mut stage: Vec<Option<String>> = vec![
            Some("Idea".into()), Some("Shooting".into()), None, Some("Gone".into()),
        ];
        for step in steps {
            match step {
                Step::Rename(at, name) => {
                    let to = format!("L{name}");
                    let Some(from) = lanes.get(at).cloned() else { continue };
                    let result = library.edit_board_lanes(
                        &book, LaneChange::Rename { from: from.clone(), to: to.clone() }, 5);
                    if to != from && lanes.contains(&to) {
                        prop_assert!(is_invalid(&result));
                    } else {
                        prop_assert!(result.is_ok());
                        lanes[at] = to.clone();
                        for s in stage.iter_mut().flatten() {
                            if *s == from { *s = to.clone(); }
                        }
                    }
                }
                Step::Move(from, to) => {
                    let result = library.edit_board_lanes(
                        &book, LaneChange::Move { from: from as u32, to: to as u32 }, 5);
                    if from < lanes.len() && to < lanes.len() {
                        prop_assert!(result.is_ok());
                        let lane = lanes.remove(from);
                        lanes.insert(to, lane);
                    } else {
                        prop_assert!(is_invalid(&result));
                    }
                }
                Step::Add(name) => {
                    let name = format!("L{name}");
                    let result = library.edit_board_lanes(
                        &book, LaneChange::Add { name: name.clone() }, 5);
                    if lanes.contains(&name) {
                        prop_assert!(is_invalid(&result));
                    } else {
                        prop_assert!(result.is_ok());
                        lanes.push(name);
                    }
                }
                Step::Remove(at, to) => {
                    let Some(lane) = lanes.get(at).cloned() else { continue };
                    let target = to.and_then(|to| lanes.get(to).cloned());
                    let result = library.edit_board_lanes(
                        &book,
                        LaneChange::Remove { lane: lane.clone(), move_to: target.clone() },
                        5,
                    );
                    let valid = lanes.len() > 1 && target.as_ref().is_none_or(|t| *t != lane);
                    if valid {
                        prop_assert!(result.is_ok());
                        lanes.remove(at);
                        for s in stage.iter_mut() {
                            if s.as_deref() == Some(lane.as_str()) { *s = target.clone(); }
                        }
                    } else {
                        prop_assert!(is_invalid(&result));
                    }
                }
                Step::Drag(note, to) => {
                    let to_lane = match lanes.get(to) {
                        Some(lane) => value(lane),
                        None => BoardLane::Unset,
                    };
                    let result = library.move_card(note_key(&ids[note]), to_lane, 5).unwrap();
                    let new = lanes.get(to).cloned();
                    let same = match (&stage[note], &new) {
                        (Some(a), Some(b)) => a == b,
                        (None, None) => true,
                        _ => false,
                    };
                    prop_assert_eq!(matches!(result, CardMoved::Unchanged), same);
                    stage[note] = new;
                }
            }
            let board = library.board(BoardKind::Notes, in_book(&book)).unwrap();
            let shown: Vec<String> = board
                .columns
                .iter()
                .filter_map(|c| match &c.lane {
                    BoardLane::Value { value } => Some(value.clone()),
                    _ => None,
                })
                .collect();
            prop_assert_eq!(&shown, &lanes);
            prop_assert_eq!(&board.setup.as_ref().unwrap().lanes, &lanes);
            let cards: usize = board.columns.iter().map(|c| c.cards.len()).sum();
            prop_assert_eq!(cards, ids.len());
            let titles = ["Pixel", "Tour", "Loose", "Odd"];
            for (ix, title) in titles.iter().enumerate() {
                let expected = match &stage[ix] {
                    Some(s) if lanes.contains(s) => value(s),
                    Some(_) => BoardLane::Unlisted,
                    None => BoardLane::Unset,
                };
                prop_assert!(lane(&board, &expected).contains(title), "{title} in {expected:?}");
            }
        }
    }
}
