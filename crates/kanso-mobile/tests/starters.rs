//! The phone's notebook starters: the catalogue its cards are drawn from, a
//! notebook made from each, the first note to open, and the refusal inside a
//! locked notebook.

mod common;

use std::sync::Arc;

use common::*;
use kanso_core::DocType;
use kanso_mobile::{KansoError, NewNotebookKind};
use kanso_store::{Origin, Put, Store};
use kanso_sync::MemoryServer;
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use serde_json::json;

const LOCKED_HINT: &str =
    "Starters can't be made inside a locked notebook. Make it at the top level, then move it in.";

const KINDS: [NewNotebookKind; 8] = [
    NewNotebookKind::Blank,
    NewNotebookKind::Product,
    NewNotebookKind::Knowledge,
    NewNotebookKind::SideProjects,
    NewNotebookKind::Journal,
    NewNotebookKind::Reading,
    NewNotebookKind::Publishing,
    NewNotebookKind::VideoStudio,
];

fn config() -> ProptestConfig {
    // The seed comes from PROPTEST_RNG_SEED.
    ProptestConfig {
        cases: 8,
        failure_persistence: Some(Box::new(FileFailurePersistence::Off)),
        ..ProptestConfig::default()
    }
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

// no-prop-test: the catalogue is fixed text.
#[test]
fn the_catalogue_lists_the_eight_starters_in_order() {
    let phone = lone_phone();
    let starters = phone.library.notebook_starters();
    let kinds: Vec<NewNotebookKind> = starters.iter().map(|s| s.kind).collect();
    assert_eq!(kinds, KINDS);
    let labels: Vec<&str> = starters.iter().map(|s| s.label.as_str()).collect();
    assert_eq!(
        labels,
        [
            "Blank notebook",
            "Product development",
            "Knowledge base",
            "Side projects",
            "Daily journal",
            "Reading",
            "Writing & publishing",
            "Video studio",
        ]
    );

    let blank = &starters[0];
    assert_eq!(blank.icon, "notebook");
    assert_eq!(blank.description, "An empty notebook for notes.");
    assert!(blank.sub_notebooks.is_empty() && blank.tags.is_empty() && !blank.makes_notes);

    let product = &starters[1];
    assert_eq!(product.icon, "rocket");
    assert_eq!(product.sub_notebooks.len(), 9);
    assert_eq!(product.sub_notebooks[0], "Features");
    assert_eq!(product.sub_notebooks[8], "Decisions");
    let tags: Vec<&str> = product.tags.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(
        tags,
        ["web", "server", "mobile", "docs", "security", "important"]
    );
    assert!(product.makes_notes);

    // The year notebook is named for today.
    let year = chrono::Local::now().format("%Y").to_string();
    assert_eq!(starters[4].sub_notebooks, [year]);

    let video = &starters[7];
    assert!(video.sub_notebooks.is_empty() && !video.makes_notes);
}

// no-prop-test: the card's tags and the made tags are one list.
#[test]
fn a_product_notebook_has_what_its_card_shows() {
    let phone = lone_phone();
    let card = phone.library.notebook_starters().remove(1);
    let made = phone
        .library
        .create_notebook_as("Kanso".into(), None, NewNotebookKind::Product)
        .unwrap();
    assert_eq!(made.notebook.name, "Kanso");
    assert_eq!(made.notebook.icon.as_deref(), Some("rocket"));

    let inside: Vec<String> = phone
        .library
        .notebooks()
        .unwrap()
        .into_iter()
        .filter(|row| row.parent_id.as_deref() == Some(made.notebook.id.as_str()))
        .map(|row| row.name)
        .collect();
    let mut expected = card.sub_notebooks.clone();
    expected.sort();
    let mut inside = inside;
    inside.sort();
    assert_eq!(inside, expected);

    let tags = phone.library.tags().unwrap();
    for tag in &card.tags {
        let found = tags.iter().find(|t| t.name == tag.name).unwrap();
        assert_eq!(found.color, tag.color);
    }

    // The first note is the one in the first notebook, so the app can open it.
    let first = made.first_note_id.expect("a starter makes notes");
    let note = phone.library.note(&first).unwrap();
    let features = phone
        .library
        .notebooks()
        .unwrap()
        .into_iter()
        .find(|row| row.name == "Features")
        .unwrap();
    assert_eq!(note.notebook_id, features.id);
}

// no-prop-test: Blank and Video studio make no notes.
#[test]
fn blank_and_the_studio_have_no_first_note() {
    let phone = lone_phone();
    for kind in [NewNotebookKind::Blank, NewNotebookKind::VideoStudio] {
        let made = phone
            .library
            .create_notebook_as("Plain".into(), None, kind)
            .unwrap();
        assert_eq!(made.first_note_id, None);
    }
}

// no-prop-test: a fixed locked parent and the store's words.
#[test]
fn a_starter_that_makes_notes_is_refused_inside_a_locked_notebook() {
    let server = Arc::new(MemoryServer::new());
    let mut locked = String::new();
    let mut child = String::new();
    let phone = phone_with(&server, |store| {
        locked = book(store, "Private");
        lock_book(store, &locked);
        child = store
            .put(
                DocType::Book,
                json!({"name": "Inner", "parentBookId": locked}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
    });
    let before = phone.library.notebooks().unwrap().len();
    for parent in [&locked, &child] {
        for kind in KINDS {
            let made = phone
                .library
                .create_notebook_as("Plan".into(), Some(parent.clone()), kind);
            match phone
                .library
                .notebook_starters()
                .iter()
                .find(|s| s.kind == kind)
                .unwrap()
                .makes_notes
            {
                true => match made {
                    Err(KansoError::Invalid { message }) => assert_eq!(message, LOCKED_HINT),
                    other => panic!("{kind:?} inside a locked notebook: {other:?}"),
                },
                false => {
                    made.unwrap();
                }
            }
        }
    }
    // Only the plain ones were made (two kinds, two parents, and the
    // Templates notebook the studio's templates went into); a refused starter
    // left nothing behind.
    assert_eq!(phone.library.notebooks().unwrap().len(), before + 5);
    assert!(phone.library.tags().unwrap().is_empty());
}

proptest! {
    #![proptest_config(config())]

    /// A starter makes one note in each notebook inside it, the first of them
    /// opens, and a second run reuses the templates and the tags.
    #[test]
    fn prop_every_starter_makes_its_card(index in 1usize..7, name in "[A-Za-z][A-Za-z0-9 ]{0,12}") {
        let phone = lone_phone();
        let card = phone.library.notebook_starters().remove(index);
        let made = phone
            .library
            .create_notebook_as(name.clone(), None, card.kind)
            .unwrap();
        // One note in each notebook inside.
        prop_assert_eq!(made.notebook.count as usize, card.sub_notebooks.len());
        prop_assert!(made.first_note_id.is_some());

        let templates = phone.library.templates().unwrap().len();
        let tags = phone.library.tags().unwrap().len();
        prop_assert_eq!(tags, card.tags.len());
        phone
            .library
            .create_notebook_as(format!("{name} again"), None, card.kind)
            .unwrap();
        prop_assert_eq!(phone.library.templates().unwrap().len(), templates);
        prop_assert_eq!(phone.library.tags().unwrap().len(), tags);
    }
}
