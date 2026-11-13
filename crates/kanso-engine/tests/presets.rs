//! The Video studio preset: a notebook with its stage board and its two
//! templates, made whole, and the template list that puts them first.

use kanso_core::DocType;
use kanso_core::dates::{Date, DateTime};
use kanso_core::properties;
use kanso_core::schema::PropertyType;
use kanso_engine::board::{Board, Lane, grouped_board, property_board};
use kanso_engine::presets::{NotebookKind, SHOT_LIST, STUDIO_STAGES, VIDEO_BRIEF, create_notebook};
use kanso_store::{Origin, Put, Store};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use serde_json::json;

fn store() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    (dir, store)
}

fn config() -> ProptestConfig {
    // The seed comes from PROPTEST_RNG_SEED.
    ProptestConfig {
        cases: 24,
        failure_persistence: Some(Box::new(FileFailurePersistence::Off)),
        ..ProptestConfig::default()
    }
}

fn now() -> DateTime {
    DateTime {
        date: Date::new(2026, 11, 13).unwrap(),
        hour: 9,
        minute: 0,
        second: 0,
    }
}

fn lanes(board: &Board) -> Vec<Lane> {
    board.columns().into_iter().map(|(lane, _)| lane).collect()
}

#[test]
fn a_studio_has_a_board_with_five_lanes_and_both_templates_first() {
    let (_dir, store) = store();
    // A template of the person's own, which sorts before both by title.
    let templates = store
        .role_book(kanso_core::roles::Role::Templates, Origin::Ui)
        .unwrap();
    store
        .put(
            DocType::Note,
            json!({"title": "Aardvark", "body": "", "bookId": templates}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();

    let made = create_notebook(
        &store,
        "  Studio ",
        None,
        NotebookKind::VideoStudio,
        Origin::Ui,
    )
    .unwrap();
    assert_eq!(made.template_ids.len(), 2);

    // The board: grouped by stage, the five lanes, then the notes with none.
    let group = property_board(&store, &made.book_id).unwrap().unwrap();
    let board = grouped_board(&store, group, Vec::new()).unwrap();
    let mut expected: Vec<Lane> = STUDIO_STAGES
        .iter()
        .map(|s| Lane::Value((*s).to_owned()))
        .collect();
    expected.push(Lane::Unset);
    assert_eq!(lanes(&board), expected);

    // The templates: this notebook's two first, then the rest.
    let listed = store.templates_for(Some(&made.book_id)).unwrap();
    let titles: Vec<(&str, bool)> = listed
        .iter()
        .map(|t| (t.title.as_str(), t.in_notebook))
        .collect();
    assert_eq!(
        titles,
        [(VIDEO_BRIEF, true), (SHOT_LIST, true), ("Aardvark", false),]
    );
    assert_eq!(
        listed.iter().take(2).map(|t| &t.id).collect::<Vec<_>>(),
        made.template_ids.iter().collect::<Vec<_>>()
    );

    // Anywhere else they come last, in title order.
    let elsewhere = store.templates_for(None).unwrap();
    assert!(elsewhere.iter().all(|t| !t.in_notebook));
    assert_eq!(elsewhere[0].title, "Aardvark");
    assert_eq!(elsewhere.len(), 3);
}

#[test]
fn a_studio_defines_its_properties() {
    let (_dir, store) = store();
    let made = create_notebook(
        &store,
        "Studio",
        None,
        NotebookKind::VideoStudio,
        Origin::Ui,
    )
    .unwrap();
    let defs = store.notebook_properties(&made.book_id).unwrap();
    let shape: Vec<(&str, PropertyType)> = defs.iter().map(|d| (d.key.as_str(), d.kind)).collect();
    assert_eq!(
        shape,
        [
            ("stage", PropertyType::Select),
            ("embargo", PropertyType::DateTimeZone),
            ("publish", PropertyType::DateTime),
            ("sponsor", PropertyType::Text),
            ("owner", PropertyType::Person),
        ]
    );
    assert_eq!(defs[0].options, STUDIO_STAGES);
    assert_eq!(
        store.notebook_templates(&made.book_id).unwrap(),
        made.template_ids
    );
}

#[test]
fn a_note_from_the_video_brief_lands_in_the_idea_lane() {
    let (_dir, store) = store();
    let made = create_notebook(
        &store,
        "Studio",
        None,
        NotebookKind::VideoStudio,
        Origin::Ui,
    )
    .unwrap();
    let note = kanso_engine::daily::from_template(
        &store,
        &made.template_ids[0],
        Some(made.book_id.clone()),
        now(),
    )
    .unwrap();
    let body = store.note(&note.id).unwrap().unwrap();
    assert_eq!(body.book_id, made.book_id);
    assert!(body.body.contains("- [ ] Script review @2026-11-13"));
    assert_eq!(
        properties::get(&properties::parse(&body.body), "stage"),
        Some(&properties::Value::Text("Idea".into()))
    );
    // Nothing the schema defines is warned about.
    assert!(store.note_property_warnings(&note.id).unwrap().is_empty());
}

#[test]
fn a_blank_notebook_has_none_of_it() {
    let (_dir, store) = store();
    let made = create_notebook(&store, "Notes", None, NotebookKind::Blank, Origin::Ui).unwrap();
    assert!(made.template_ids.is_empty());
    assert!(store.notebook_board(&made.book_id).unwrap().is_none());
    assert!(store.notebook_properties(&made.book_id).unwrap().is_empty());
    assert!(store.templates().unwrap().is_empty());
}

#[test]
fn a_refused_studio_leaves_nothing_behind() {
    let (_dir, store) = store();
    for (name, parent) in [("", None), ("   ", None), ("Studio", Some("book:nowhere"))] {
        let made = create_notebook(
            &store,
            name,
            parent.map(str::to_owned),
            NotebookKind::VideoStudio,
            Origin::Ui,
        );
        assert!(made.is_err(), "{name:?} in {parent:?}");
    }
    assert!(store.templates().unwrap().is_empty());
    assert!(store.books().unwrap().iter().all(|b| b.name != "Studio"));
}

#[test]
fn a_studio_goes_inside_its_parent() {
    let (_dir, store) = store();
    let parent = create_notebook(&store, "Work", None, NotebookKind::Blank, Origin::Ui).unwrap();
    let made = create_notebook(
        &store,
        "Studio",
        Some(parent.book_id.clone()),
        NotebookKind::VideoStudio,
        Origin::Ui,
    )
    .unwrap();
    let book = store
        .books()
        .unwrap()
        .into_iter()
        .find(|b| b.id == made.book_id)
        .unwrap();
    assert_eq!(
        book.parent_book_id.as_deref(),
        Some(parent.book_id.as_str())
    );
}

proptest! {
    #![proptest_config(config())]

    /// However many studios there are, each has the five lanes, names its
    /// own two templates and lists them first, and none shares a template.
    #[test]
    fn prop_every_studio_lists_its_own_templates_first(
        names in prop::collection::vec("[A-Za-z][A-Za-z0-9 ]{0,10}", 1..4),
        extra in prop::collection::vec("[A-Za-z]{1,8}", 0..3),
    ) {
        let (_dir, store) = store();
        let templates = store
            .role_book(kanso_core::roles::Role::Templates, Origin::Ui)
            .unwrap();
        for title in &extra {
            store
                .put(
                    DocType::Note,
                    json!({"title": title, "body": "", "bookId": templates}),
                    Put::Upsert,
                    Origin::Ui,
                )
                .unwrap();
        }
        let mut seen = std::collections::HashSet::new();
        for name in &names {
            let made = create_notebook(&store, name, None, NotebookKind::VideoStudio, Origin::Ui)
                .unwrap();
            prop_assert!(made.template_ids.iter().all(|id| seen.insert(id.clone())));
            let group = property_board(&store, &made.book_id).unwrap().unwrap();
            let board = grouped_board(&store, group, Vec::new()).unwrap();
            prop_assert_eq!(lanes(&board).len(), STUDIO_STAGES.len() + 1);
            let listed = store.templates_for(Some(&made.book_id)).unwrap();
            let firsts: Vec<&String> = listed.iter().take(2).map(|t| &t.id).collect();
            prop_assert_eq!(firsts, made.template_ids.iter().collect::<Vec<_>>());
            prop_assert!(listed.iter().take(2).all(|t| t.in_notebook));
            prop_assert!(listed.iter().skip(2).all(|t| !t.in_notebook));
            // Every template is listed once.
            prop_assert_eq!(listed.len(), store.templates().unwrap().len());
        }
    }
}
