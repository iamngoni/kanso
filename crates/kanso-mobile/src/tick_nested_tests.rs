use std::sync::Arc;

use kanso_sync::{MemorySecrets, MemoryServer};
use proptest::prelude::*;
use proptest::test_runner::{Config, RngAlgorithm, RngSeed};

use super::*;
use crate::model::{NewNote, Ticked};

const NESTED: &str = "- [ ] ship\n  - [ ] build\n  - [ ] test\n";

fn library(path: &std::path::Path) -> Library {
    Library::open_for_test(
        path,
        Arc::new(MemorySecrets::new()),
        Arc::new(MemoryServer::new()),
    )
    .unwrap()
}

fn with_note(body: &str) -> (tempfile::TempDir, Library, String) {
    let dir = tempfile::tempdir().unwrap();
    let library = library(&dir.path().join("library.db"));
    let id = library
        .create(NewNote {
            title: "Ship".into(),
            body: body.into(),
            ..NewNote::default()
        })
        .unwrap()
        .id;
    (dir, library, id)
}

fn body_of(library: &Library, id: &str) -> String {
    library.inner.store.note(id).unwrap().unwrap().body
}

#[test]
fn the_setting_asks_until_changed_and_is_kept() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("library.db");
    let opened = library(&path);
    assert_eq!(opened.tick_nested().unwrap(), TickNested::Ask);
    opened.set_tick_nested(TickNested::Always).unwrap();
    drop(opened);
    assert_eq!(library(&path).tick_nested().unwrap(), TickNested::Always);
}

#[test]
fn set_task_follows_the_setting() {
    let (_dir, library, id) = with_note(NESTED);
    // Ask ticks one box: the app asks before it links.
    let Ticked::Changed { .. } = library.set_task(&id, 0, "ship", true).unwrap() else {
        panic!("not changed");
    };
    assert_eq!(
        body_of(&library, &id),
        "- [x] ship\n  - [ ] build\n  - [ ] test\n"
    );

    let (_dir, library, id) = with_note(NESTED);
    library.set_tick_nested(TickNested::Always).unwrap();
    let Ticked::Changed { undo, .. } = library.set_task(&id, 0, "ship", true).unwrap() else {
        panic!("not changed");
    };
    assert_eq!(
        body_of(&library, &id),
        "- [x] ship\n  - [x] build\n  - [x] test\n"
    );
    // One undo takes the whole cascade back.
    assert_eq!(library.undo(undo).unwrap(), 1);
    assert_eq!(body_of(&library, &id), NESTED);
}

#[test]
fn an_explicit_link_beats_the_setting() {
    let (_dir, library, id) = with_note(NESTED);
    library.set_tick_nested(TickNested::Never).unwrap();
    library.set_task_linked(&id, 0, "ship", true, true).unwrap();
    assert_eq!(
        body_of(&library, &id),
        "- [x] ship\n  - [x] build\n  - [x] test\n"
    );
}

#[test]
fn the_page_tick_links_and_says_when_it_would() {
    let (_dir, library, id) = with_note(NESTED);
    assert!(library.toggle_task_would_cascade(&id, 0).unwrap());
    assert!(library.task_would_cascade(&id, 0, "ship", true).unwrap());
    // Clearing a task with nothing ticked above it changes nothing more.
    assert!(!library.task_would_cascade(&id, 0, "ship", false).unwrap());
    assert!(!library.task_would_cascade(&id, 9, "ship", true).unwrap());
    let rev = library
        .inner
        .store
        .note(&id)
        .unwrap()
        .unwrap()
        .rev
        .unwrap_or_default();
    library.toggle_task_at_linked(&id, 0, &rev, true).unwrap();
    assert_eq!(
        body_of(&library, &id),
        "- [x] ship\n  - [x] build\n  - [x] test\n"
    );
}

fn config() -> Config {
    Config {
        cases: 24,
        rng_seed: RngSeed::Fixed(0x0074_6e65_7374),
        rng_algorithm: RngAlgorithm::ChaCha,
        failure_persistence: None,
        ..Config::default()
    }
}

proptest! {
    #![proptest_config(config())]

    /// `task_would_cascade` is exactly whether the linked write differs
    /// from the plain one, for any task of a nested list.
    #[test]
    fn prop_would_cascade_is_whether_linking_differs(
        lines in prop::collection::vec((0usize..3, "[a-z]{1,6}", any::<bool>()), 1..6),
        pick in any::<prop::sample::Index>(),
    ) {
        let body: String = lines
            .iter()
            .map(|(depth, word, done)| format!("{}- [{}] {word}\n", "  ".repeat(*depth), if *done { 'x' } else { ' ' }))
            .collect();
        let items = kanso_core::tasks::task_items(&body);
        let item = &items[pick.index(items.len())];
        let (_dir_a, plain, id_a) = with_note(&body);
        let (_dir_b, linked, id_b) = with_note(&body);
        let asked = plain.task_would_cascade(&id_a, item.line, &item.text, !item.checked).unwrap();
        plain.set_task_linked(&id_a, item.line, &item.text, !item.checked, false).unwrap();
        linked.set_task_linked(&id_b, item.line, &item.text, !item.checked, true).unwrap();
        // Equal lines make the task ambiguous only when it moved; the line
        // is right here, so the first match is the one asked about.
        prop_assert_eq!(asked, body_of(&plain, &id_a) != body_of(&linked, &id_b));
    }
}
