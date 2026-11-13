//! The phone's Video studio: a notebook made as one has the five-lane stage
//! board and both templates, and the template list for it puts them first.

mod common;

use common::*;
use kanso_mobile::{BoardKind, BoardLane, KansoError, ListPlace, ListQuery, NewNotebookKind};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;

const STAGES: [&str; 5] = ["Idea", "Scripting", "Shooting", "Editing", "Published"];

fn config() -> ProptestConfig {
    // The seed comes from PROPTEST_RNG_SEED.
    ProptestConfig {
        cases: 12,
        failure_persistence: Some(Box::new(FileFailurePersistence::Off)),
        ..ProptestConfig::default()
    }
}

fn in_book(id: &str) -> ListQuery {
    ListQuery {
        place: ListPlace::Notebook { id: id.into() },
        ..ListQuery::default()
    }
}

// no-prop-test: a fixed studio and what the phone must show of it.
#[test]
fn a_studio_has_five_lanes_and_both_templates_first() {
    let phone = lone_phone();
    let made = phone
        .library
        .create_notebook_as("Channel".into(), None, NewNotebookKind::VideoStudio)
        .unwrap();
    assert_eq!(made.name, "Channel");

    let board = phone
        .library
        .board(BoardKind::Notes, in_book(&made.id))
        .unwrap();
    let lanes: Vec<(&BoardLane, &str)> = board
        .columns
        .iter()
        .map(|c| (&c.lane, c.label.as_str()))
        .collect();
    let mut expected: Vec<(BoardLane, String)> = STAGES
        .iter()
        .map(|s| (BoardLane::Value { value: (*s).into() }, (*s).to_owned()))
        .collect();
    expected.push((BoardLane::Unset, "No stage".to_owned()));
    let expected: Vec<(&BoardLane, &str)> = expected.iter().map(|(l, s)| (l, s.as_str())).collect();
    assert_eq!(lanes, expected);

    let listed = phone.library.templates_for(Some(made.id.clone())).unwrap();
    let shape: Vec<(&str, bool)> = listed
        .iter()
        .map(|t| (t.title.as_str(), t.in_notebook))
        .collect();
    assert_eq!(shape, [("Video brief", true), ("Shot list", true)]);
}

// no-prop-test: a fixed pair of notebooks.
#[test]
fn a_blank_notebook_has_no_templates_of_its_own() {
    let phone = lone_phone();
    let studio = phone
        .library
        .create_notebook_as("Channel".into(), None, NewNotebookKind::VideoStudio)
        .unwrap();
    let blank = phone
        .library
        .create_notebook_as("Notes".into(), None, NewNotebookKind::Blank)
        .unwrap();
    let for_blank = phone.library.templates_for(Some(blank.id)).unwrap();
    assert_eq!(for_blank.len(), 2);
    assert!(for_blank.iter().all(|t| !t.in_notebook));
    // The plain list is the same two, none marked.
    let plain = phone.library.templates().unwrap();
    assert_eq!(plain.len(), 2);
    assert!(plain.iter().all(|t| !t.in_notebook));
    // And a template from the studio's list makes a note in the studio.
    let first = phone
        .library
        .templates_for(Some(studio.id.clone()))
        .unwrap()
        .remove(0);
    let now = kanso_mobile::LocalTime {
        year: 2026,
        month: 11,
        day: 13,
        hour: 9,
        minute: 0,
        second: 0,
    };
    let made = phone
        .library
        .new_from_template(&first.id, Some(studio.id.clone()), now)
        .unwrap();
    let note = phone.library.note(&made.id).unwrap();
    assert_eq!(note.notebook_id, studio.id);
}

// no-prop-test: refusals are fixed cases.
#[test]
fn a_refused_studio_makes_nothing() {
    let phone = lone_phone();
    let empty = phone
        .library
        .create_notebook_as("  ".into(), None, NewNotebookKind::VideoStudio);
    assert!(matches!(empty, Err(KansoError::Invalid { .. })));
    let orphan = phone.library.create_notebook_as(
        "Channel".into(),
        Some("book:nowhere".into()),
        NewNotebookKind::VideoStudio,
    );
    assert!(matches!(orphan, Err(KansoError::NotFound)));
    assert!(phone.library.templates().unwrap().is_empty());
}

proptest! {
    #![proptest_config(config())]

    /// Each studio lists its own two templates first, whatever the others.
    #[test]
    fn prop_each_studio_lists_its_own_templates_first(
        names in prop::collection::vec("[A-Za-z][A-Za-z0-9]{0,8}", 1..4),
    ) {
        let phone = lone_phone();
        for (made_so_far, name) in names.iter().enumerate() {
            let made = phone
                .library
                .create_notebook_as(name.clone(), None, NewNotebookKind::VideoStudio)
                .unwrap();
            let listed = phone.library.templates_for(Some(made.id)).unwrap();
            prop_assert_eq!(listed.len(), 2 * (made_so_far + 1));
            prop_assert!(listed[0].in_notebook && listed[1].in_notebook);
            prop_assert!(listed.iter().skip(2).all(|t| !t.in_notebook));
        }
    }
}
