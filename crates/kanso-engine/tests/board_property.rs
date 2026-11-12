//! A board grouped by a property: a notebook's `kansoBoard` names the
//! property, its select options (or the board's own lanes) are the columns,
//! and moving a card writes the property in its note. Notes whose value is
//! missing or not a lane are never dropped: they have columns of their own.

use kanso_core::DocType;
use kanso_core::properties;
use kanso_core::schema::{BoardSpec, PropertyDef, PropertyType};
use kanso_engine::board::{
    Board, Card, CardId, Lane, Written, grouped_board, property_board, undo_move,
    write_move_undoable,
};
use kanso_store::{NoteSummary, Origin, Put, Store};
use proptest::prelude::*;
use proptest::test_runner::{FileFailurePersistence, RngSeed};
use serde_json::json;

const STAGES: [&str; 5] = ["Idea", "Scripting", "Shooting", "Editing", "Published"];

fn config() -> ProptestConfig {
    ProptestConfig {
        cases: 64,
        rng_seed: RngSeed::Fixed(0x006b_616e_736f),
        failure_persistence: Some(Box::new(FileFailurePersistence::Off)),
        ..ProptestConfig::default()
    }
}

fn summary(id: &str) -> NoteSummary {
    NoteSummary {
        id: id.into(),
        rev: String::new(),
        title: id.into(),
        snippet: String::new(),
        book_id: "book:studio".into(),
        status: kanso_core::NoteStatus::None,
        tags: Vec::new(),
        pinned: false,
        updated_at: 0,
        created_at: 0,
        tasks_done: 0,
        tasks_total: 0,
        locked: false,
        canvas: false,
    }
}

fn studio(lanes: &[&str], show_unset: bool) -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    store
        .put(
            DocType::Book,
            json!({"_id": "book:studio", "name": "Studio"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
    store
        .set_notebook_properties(
            "book:studio",
            &[PropertyDef {
                key: "stage".into(),
                kind: PropertyType::Select,
                options: STAGES.iter().map(|s| (*s).to_owned()).collect(),
            }],
            Origin::Ui,
        )
        .unwrap();
    store
        .set_notebook_board(
            "book:studio",
            Some(&BoardSpec {
                by: "stage".into(),
                lanes: lanes.iter().map(|s| (*s).to_owned()).collect(),
                show_unset,
            }),
            Origin::Ui,
        )
        .unwrap();
    (dir, store)
}

fn add(store: &Store, ix: usize, front: Option<&str>) -> NoteSummary {
    let id = format!("note:n{ix}");
    let body = match front {
        Some(front) => format!("---\n{front}\nsponsor: Acme\n---\n# Video {ix}\n- [ ] shot\n"),
        None => format!("# Video {ix}\n"),
    };
    store
        .put(
            DocType::Note,
            json!({"_id": id, "title": format!("Video {ix}"), "body": body,
                "bookId": "book:studio", "tags": []}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
    NoteSummary {
        title: format!("Video {ix}"),
        ..summary(&id)
    }
}

fn board_of(store: &Store, notes: Vec<NoteSummary>) -> Board {
    let group = property_board(store, "book:studio").unwrap().unwrap();
    grouped_board(store, group, notes).unwrap()
}

fn titles(board: &Board, lane: &Lane) -> Vec<String> {
    board
        .columns()
        .into_iter()
        .find(|(l, _)| l == lane)
        .map(|(_, cards)| {
            cards
                .into_iter()
                .map(|ix| match &board.cards[ix] {
                    Card::Note(note) => note.title.clone(),
                    Card::Task(_) => unreachable!(),
                })
                .collect()
        })
        .unwrap_or_default()
}

/// A note's `stage:` line as a person or another app might have written it.
fn front() -> impl Strategy<Value = Option<String>> {
    prop_oneof![
        Just(None),
        (0usize..5).prop_map(|i| Some(format!("stage: {}", STAGES[i]))),
        Just(Some("stage:".to_owned())),
        Just(Some("stage: \"\"".to_owned())),
        Just(Some("stage: Pitched".to_owned())),
        Just(Some("stage: 42".to_owned())),
        Just(Some("stage: true".to_owned())),
        Just(Some("stage: [Idea, Editing]".to_owned())),
        Just(Some("stage: {a: 1}".to_owned())),
        Just(Some("other: Idea".to_owned())),
    ]
}

fn lanes() -> impl Strategy<Value = Vec<&'static str>> {
    prop_oneof![
        Just(Vec::new()),
        Just(vec!["Editing", "Idea"]),
        Just(vec![
            "Published",
            "Shooting",
            "Scripting",
            "Idea",
            "Editing"
        ]),
    ]
}

proptest! {
    #![proptest_config(config())]

    /// Whatever the notes hold in the property, every note is in exactly one
    /// column, and a note with no value or a value that is no lane is in a
    /// column named for that, never dropped.
    #[test]
    fn prop_every_note_is_in_exactly_one_column(
        fronts in prop::collection::vec(front(), 1..10),
        lanes in lanes(),
        show_unset in any::<bool>(),
    ) {
        let (_dir, store) = studio(&lanes, show_unset);
        let notes: Vec<NoteSummary> = fronts.iter().enumerate()
            .map(|(ix, f)| add(&store, ix, f.as_deref())).collect();
        let board = board_of(&store, notes);
        let mut seen: Vec<usize> = board.columns().iter().flat_map(|(_, c)| c.clone()).collect();
        seen.sort_unstable();
        prop_assert_eq!(seen, (0..fronts.len()).collect::<Vec<_>>());
        for (lane, cards) in board.columns() {
            for ix in cards {
                let stored = store.note(board.cards[ix].note_id()).unwrap().unwrap();
                let all = properties::parse(&stored.body);
                let value = properties::get(&all, "stage").filter(|v| !v.is_empty());
                match (&lane, value) {
                    (Lane::Unset, v) => prop_assert!(v.is_none()),
                    (Lane::Value(name), Some(v)) => {
                        prop_assert!(!v.is_raw());
                        prop_assert_eq!(&v.text(), name);
                    }
                    (Lane::Unlisted, Some(v)) => {
                        prop_assert!(v.is_raw() || !board.lanes().contains(&Lane::Value(v.text())));
                    }
                    other => prop_assert!(false, "{:?}", other),
                }
            }
        }
    }

    /// The columns follow the select's options in their order (or the
    /// board's own lanes), then the unlisted column when a note needs it,
    /// then the one for notes without a value.
    #[test]
    fn prop_column_order_follows_the_options(
        fronts in prop::collection::vec(front(), 0..10),
        lanes in lanes(),
    ) {
        let (_dir, store) = studio(&lanes, true);
        let notes: Vec<NoteSummary> = fronts.iter().enumerate()
            .map(|(ix, f)| add(&store, ix, f.as_deref())).collect();
        let board = board_of(&store, notes);
        let expected: Vec<&str> = if lanes.is_empty() { STAGES.to_vec() } else { lanes.clone() };
        let columns: Vec<Lane> = board.columns().into_iter().map(|(l, _)| l).collect();
        let values: Vec<String> = columns.iter().filter_map(|l| match l {
            Lane::Value(v) => Some(v.clone()),
            _ => None,
        }).collect();
        prop_assert_eq!(values, expected.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>());
        // Values first, then Unlisted, then Unset last.
        let rank = |l: &Lane| match l { Lane::Value(_) => 0, Lane::Unlisted => 1, _ => 2 };
        prop_assert!(columns.windows(2).all(|w| rank(&w[0]) <= rank(&w[1])));
        prop_assert_eq!(columns.last(), Some(&Lane::Unset));
    }

    /// Moving a card to any column and then undoing puts the note back
    /// exactly (body and edit time), and a move changes only the
    /// property's line in the note.
    #[test]
    fn prop_move_then_undo_is_identity(
        fronts in prop::collection::vec(front(), 1..8),
        picks in prop::collection::vec((0usize..8, 0usize..7), 1..6),
    ) {
        let (_dir, store) = studio(&[], true);
        let notes: Vec<NoteSummary> = fronts.iter().enumerate()
            .map(|(ix, f)| add(&store, ix, f.as_deref())).collect();
        let mut board = board_of(&store, notes);
        for (card, to) in picks {
            let id = format!("note:n{}", card % fronts.len());
            let lanes: Vec<Lane> = board.lanes().into_iter().filter(|l| *l != Lane::Unlisted).collect();
            let to = lanes[to % lanes.len()].clone();
            let before = store.note(&id).unwrap().unwrap();
            let lane_before = board.lane_of(board.cards.iter().find(|c| c.note_id() == id).unwrap());
            let Some(mv) = board.move_card(&CardId::Note(id.clone()), to.clone()) else {
                // Already there: nothing to write.
                prop_assert_eq!(lane_before, to);
                continue;
            };
            prop_assert_eq!(mv.property.as_deref(), Some("stage"));
            let (written, undo) = write_move_undoable(&store, &mv, 9_999).unwrap();
            prop_assert_eq!(written, Written::Done);
            let after = store.note(&id).unwrap().unwrap();
            // The note now says what the lane says.
            let all = properties::parse(&after.body);
            match &to {
                Lane::Value(name) => prop_assert_eq!(
                    properties::get(&all, "stage").map(|v| v.text()), Some(name.clone())),
                _ => prop_assert!(properties::get(&all, "stage").is_none_or(|v| v.is_empty())),
            }
            prop_assert_eq!(after.title.clone(), before.title.clone());
            prop_assert_eq!(properties::get(&all, "sponsor").map(|v| v.text()),
                properties::get(&properties::parse(&before.body), "sponsor").map(|v| v.text()));
            match undo {
                Some(undo) => {
                    prop_assert!(undo_move(&store, &undo).unwrap());
                    let back = store.note(&id).unwrap().unwrap();
                    prop_assert_eq!(back.body, before.body);
                    prop_assert_eq!(back.updated_at, before.updated_at);
                    // The board reads the same lane again.
                    let group = board.group.clone().unwrap();
                    let notes = board.cards.iter().filter_map(|c| match c {
                        Card::Note(n) => Some(n.clone()),
                        Card::Task(_) => None,
                    }).collect();
                    board = grouped_board(&store, group, notes).unwrap();
                    prop_assert_eq!(
                        board.lane_of(board.cards.iter().find(|c| c.note_id() == id).unwrap()),
                        lane_before);
                }
                None => prop_assert_eq!(after.body, before.body),
            }
        }
    }
}

#[test]
fn moving_a_card_writes_the_stage() {
    let (_dir, store) = studio(&[], true);
    let notes = vec![
        add(&store, 0, Some("stage: Idea")),
        add(&store, 1, Some("stage: Shooting")),
        add(&store, 2, None),
        add(&store, 3, Some("stage: Pitched")),
    ];
    let mut board = board_of(&store, notes);
    assert_eq!(titles(&board, &Lane::Value("Idea".into())), ["Video 0"]);
    assert_eq!(titles(&board, &Lane::Unset), ["Video 2"]);
    assert_eq!(titles(&board, &Lane::Unlisted), ["Video 3"]);
    assert_eq!(board.label(&Lane::Unset), "No stage");
    assert_eq!(board.label(&Lane::Unlisted), "Unlisted stage");

    let id = CardId::Note("note:n0".into());
    let mv = board
        .move_card(&id, Lane::Value("Editing".into()))
        .expect("a move");
    let (written, undo) = write_move_undoable(&store, &mv, 5).unwrap();
    assert_eq!(written, Written::Done);
    let body = store.note("note:n0").unwrap().unwrap().body;
    assert_eq!(
        body,
        "---\nstage: Editing\nsponsor: Acme\n---\n# Video 0\n- [ ] shot\n"
    );
    assert!(undo_move(&store, &undo.unwrap()).unwrap());
    assert!(
        store
            .note("note:n0")
            .unwrap()
            .unwrap()
            .body
            .contains("stage: Idea")
    );

    // Into "No stage" clears the value, and a note with a bad value moves
    // out of its Unlisted column like any other.
    let mv = board.move_card(&id, Lane::Unset).unwrap();
    write_move_undoable(&store, &mv, 6).unwrap();
    let bad = CardId::Note("note:n3".into());
    let mv = board.move_card(&bad, Lane::Value("Idea".into())).unwrap();
    assert_eq!(mv.from, Lane::Unlisted);
    write_move_undoable(&store, &mv, 7).unwrap();
    let board = board_of(
        &store,
        board
            .cards
            .iter()
            .filter_map(|c| match c {
                Card::Note(n) => Some(n.clone()),
                Card::Task(_) => None,
            })
            .collect(),
    );
    assert_eq!(titles(&board, &Lane::Value("Idea".into())), ["Video 3"]);
    assert!(titles(&board, &Lane::Unlisted).is_empty());
    assert!(!board.lanes().contains(&Lane::Unlisted));
}

#[test]
fn an_undo_never_overwrites_a_later_edit() {
    let (_dir, store) = studio(&[], true);
    let notes = vec![add(&store, 0, Some("stage: Idea"))];
    let mut board = board_of(&store, notes);
    let mv = board
        .move_card(
            &CardId::Note("note:n0".into()),
            Lane::Value("Editing".into()),
        )
        .unwrap();
    let (_, undo) = write_move_undoable(&store, &mv, 5).unwrap();
    store
        .put(
            DocType::Note,
            json!({"_id": "note:n0", "body": "---\nstage: Editing\n---\nrewritten\n"}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    assert!(!undo_move(&store, &undo.unwrap()).unwrap());
    assert!(
        store
            .note("note:n0")
            .unwrap()
            .unwrap()
            .body
            .contains("rewritten")
    );
}

#[test]
fn a_notebook_without_a_board_keeps_its_status_board() {
    let (_dir, store) = studio(&[], true);
    store
        .set_notebook_board("book:studio", None, Origin::Ui)
        .unwrap();
    assert_eq!(property_board(&store, "book:studio").unwrap(), None);
    assert_eq!(store.notebook_board("book:studio").unwrap(), None);
    // Nothing is left of a board in a notebook that never had one, and the
    // status board is what `Board::new` gives, with a status move writing
    // the status alone.
    let note = add(&store, 0, Some("stage: Idea"));
    let mut board = Board::new(kanso_engine::board::BoardKind::Notes);
    board.set_cards(vec![Card::Note(note)]);
    assert_eq!(board.lanes().len(), 5);
    let mv = board
        .move_card(
            &CardId::Note("note:n0".into()),
            Lane::Status(kanso_core::NoteStatus::Active),
        )
        .unwrap();
    assert_eq!(mv.property, None);
    assert!(write_move_undoable(&store, &mv, 5).unwrap().1.is_none());
    let stored = store.note("note:n0").unwrap().unwrap();
    assert_eq!(stored.status, kanso_core::NoteStatus::Active);
    assert!(stored.body.contains("stage: Idea"));
}
