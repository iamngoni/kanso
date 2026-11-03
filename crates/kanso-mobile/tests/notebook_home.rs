//! Notebook homepages on a phone (issue #201): the card, the page with and
//! without AI, the map, the Home note, customising, locked notebooks, and
//! that a kept-out note is never sent. CI pins the seed with
//! `PROPTEST_RNG_SEED`.

mod common;

use common::model::*;
use common::*;
use kanso_core::DocType;
use kanso_mobile::{HomeSettings, NewNote, PageSectionState};
use kanso_store::{Origin, Put, Store};
use serde_json::json;

const CANARY: &str = "okapicanary9913";
const T0: i64 = 1_761_600_000_000;

fn store(phone: &Phone) -> Store {
    Store::open(phone.path()).unwrap()
}

fn nested_book(store: &Store, id: &str, name: &str, parent: Option<&str>) -> String {
    store
        .put(
            DocType::Book,
            json!({"_id": id, "name": name, "parentBookId": parent}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id
}

/// Research with Drafts and Papers inside it.
fn world(phone: &Phone) -> Store {
    let store = store(phone);
    nested_book(&store, "book:r", "Research", None);
    nested_book(&store, "book:d", "Drafts", Some("book:r"));
    nested_book(&store, "book:p", "Papers", Some("book:r"));
    let now = kanso_core::now_ms();
    raw_note(
        &store,
        json!({"_id": "note:plan", "title": "Chapter plan", "bookId": "book:r",
            "createdAt": T0, "updatedAt": now - 1000,
            "body": "# Plan\nWe decided to use the 2019 dataset, not 2021\n"}),
    );
    raw_note(
        &store,
        json!({"_id": "note:ch3", "title": "Chapter 3", "bookId": "book:d",
            "createdAt": T0, "updatedAt": now - 500,
            "body": "See [[Chapter plan]]\n- [ ] Email Dr Banda @2026-11-04\n"}),
    );
    store
}

// no-prop-test: settings defaults and one round trip.
#[test]
fn settings_default_on_and_round_trip() {
    let phone = lone_phone();
    let home = phone.library.notebook_home();
    let settings = home.settings();
    assert!(settings.show_homepages && settings.summaries_with_ai);
    assert_eq!(settings.refresh, "auto");
    let saved = home
        .set_settings(HomeSettings {
            show_homepages: false,
            refresh: "press".into(),
            ..settings
        })
        .unwrap();
    assert!(!saved.show_homepages);
    assert_eq!(saved.refresh, "press");
    // The refresh setting is the one people's pages use.
    assert_eq!(phone.library.people().settings().refresh, "press");
}

// no-prop-test: one scripted tour of the card and the computed sections.
#[test]
fn the_card_and_computed_sections_without_ai() {
    let phone = lone_phone();
    let _store = world(&phone);
    let home = phone.library.notebook_home();

    let card = home.card("book:r").unwrap();
    assert!(card.show && !card.opens_directly && !card.has_home_note);
    assert_eq!(card.name, "Research");
    assert!(card.about_line.is_empty());
    // Drafts holds a note; Papers holds none and nothing inside it.
    assert!(!home.card("book:d").unwrap().opens_directly);
    // A notebook with notebooks inside it and no notes of its own opens its homepage.
    nested_book(&_store, "book:wrap", "Wrapper", None);
    nested_book(&_store, "book:in", "Inner", Some("book:wrap"));
    assert!(home.card("book:wrap").unwrap().opens_directly);
    assert!(home.opens_directly("book:wrap").unwrap());

    let panel = home.open("book:r").unwrap().unwrap();
    assert!(!panel.ai_on && !panel.locked && !panel.locked_open);
    assert_eq!(panel.header.notes, 2);
    assert!(
        panel
            .header
            .line
            .starts_with("2 notes \u{b7} 2 notebooks inside")
    );
    let keys: Vec<&str> = panel.sections.iter().map(|s| s.key.as_str()).collect();
    assert_eq!(keys, ["key_notes", "recent", "open_tasks", "map", "inside"]);
    assert_eq!(panel.footer, home.words().ai_off);
    let tasks = panel
        .sections
        .iter()
        .find(|s| s.key == "open_tasks")
        .unwrap();
    assert_eq!(tasks.items.len(), 1);
    assert!(tasks.items[0].task.is_some());
    // Refresh is refused while AI is off.
    assert!(home.refresh_section("book:r", "about").is_err());

    let map = home.map("book:r").unwrap();
    assert_eq!(map.nodes.len(), 2);
    assert_eq!(map.edges.len(), 1);
    assert_eq!(home.map("book:r").unwrap(), map, "the same map every time");

    // Settings off: the card goes away.
    let mut settings = home.settings();
    settings.show_homepages = false;
    home.set_settings(settings).unwrap();
    assert!(!home.card("book:r").unwrap().show);
}

// no-prop-test: both platforms' words come from the engine.
#[test]
fn the_words_and_section_order_are_the_engines() {
    let phone = lone_phone();
    let words = phone.library.notebook_home().words();
    assert_eq!(
        words.section_keys,
        [
            "home_note",
            "about",
            "key_notes",
            "recent",
            "open_tasks",
            "people",
            "decisions",
            "map",
            "inside"
        ]
    );
    assert_eq!(words.ask_button, "Ask about this notebook");
    assert_eq!(words.first_visit_button, "Write a summary of this notebook");
    assert_eq!(
        words.ai_off,
        "Turn on AI in Settings to add a summary and decisions."
    );
    assert_eq!(words.locked, "Summaries are off for locked notebooks.");
}

// no-prop-test: one scripted model reply, checked against the lines it was sent.
#[test]
fn ai_sections_cite_lines_and_can_be_kept_hidden_copied_and_marked_wrong() {
    let model =
        Scripted::new("- We decided to use the 2019 dataset, not 2021 [1:L2]\n- made up [4:L9]\n");
    let (phone, ai) = ai_phone(&model);
    let store = world(&phone);
    let exclude = tag(&store, "ai-exclude");
    raw_note(
        &store,
        json!({"title": "Excluded", "bookId": "book:r", "tags": [exclude],
            "body": format!("{CANARY} we decided on excluded things\n")}),
    );
    let home = phone.library.notebook_home_with_ai(ai);
    let panel = home.open("book:r").unwrap().unwrap();
    assert!(panel.ai_on);
    let about = panel.sections.iter().find(|s| s.key == "about").unwrap();
    assert_eq!(about.state, PageSectionState::Unwritten);

    let panel = home
        .refresh_section("book:r", "decisions")
        .unwrap()
        .unwrap();
    let decisions = panel
        .sections
        .iter()
        .find(|s| s.key == "decisions")
        .unwrap();
    assert_eq!(decisions.items.len(), 1, "{decisions:?}");
    assert_eq!(decisions.state, PageSectionState::Fresh);
    assert!(
        !model.everything_sent().contains(CANARY),
        "kept-out notes are not sent"
    );

    // The card shows the About line once it is written.
    home.refresh_section("book:r", "about").unwrap();
    assert!(!home.card("book:r").unwrap().about_line.is_empty());

    let panel = home.pin("book:r", "decisions", true).unwrap().unwrap();
    let decisions = panel
        .sections
        .iter()
        .find(|s| s.key == "decisions")
        .unwrap();
    assert_eq!(decisions.state, PageSectionState::Pinned);
    let panel = home.hide("book:r", "recent", true).unwrap().unwrap();
    assert!(panel.sections.iter().all(|s| s.key != "recent"));
    assert_eq!(panel.hidden[0].title, "Recent activity");
    let panel = home.show_hidden("book:r").unwrap().unwrap();
    assert!(panel.hidden.is_empty());

    // Copy into the Home note: made, marked, with source links.
    let id = home.copy_into_home("book:r", "decisions").unwrap();
    let body = store.note(&id).unwrap().unwrap().body;
    assert!(
        body.contains("kanso: home") && body.contains("## Decisions"),
        "{body}"
    );
    assert!(body.contains("inkdrop://note/"));
    assert!(home.open("book:r").unwrap().unwrap().header.has_home_note);

    let panel = home.not_right("book:r", "decisions", 0).unwrap().unwrap();
    let decisions = panel
        .sections
        .iter()
        .find(|s| s.key == "decisions")
        .unwrap();
    assert!(decisions.items.is_empty());
}

// no-prop-test: one scripted tour of choosing, making and moving a Home note.
#[test]
fn the_home_note_is_chosen_made_cleared_and_moved() {
    let phone = lone_phone();
    let store = world(&phone);
    let home = phone.library.notebook_home();
    let candidates = home.candidates("book:r").unwrap();
    assert_eq!(candidates.len(), 1);
    home.choose_home("book:r", "note:plan").unwrap();
    assert!(home.candidates("book:r").unwrap()[0].current);
    let panel = home.open("book:r").unwrap().unwrap();
    assert_eq!(panel.header.home_note_id, "note:plan");
    assert_eq!(panel.sections[0].key, "home_note");
    let made = home.new_home("book:r").unwrap();
    assert_eq!(store.note(&made).unwrap().unwrap().title, "About Research");
    assert!(!kanso_core::home::is_home(
        &store.note("note:plan").unwrap().unwrap().body
    ));
    assert!(home.clear_home("book:r").unwrap());
    assert!(!home.open("book:r").unwrap().unwrap().header.has_home_note);
    // Moving a Home note into a notebook that has one removes its mark.
    home.choose_home("book:r", &made).unwrap();
    home.choose_home("book:d", "note:ch3").unwrap();
    store
        .put(
            DocType::Note,
            json!({"_id": made, "bookId": "book:d"}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    assert!(home.after_move(&made).unwrap());
}

// no-prop-test: order and hidden sections written and read back.
#[test]
fn customise_orders_and_hides_and_syncs_with_the_notebook() {
    let phone = lone_phone();
    let store = world(&phone);
    let home = phone.library.notebook_home();
    let panel = home
        .customise(
            "book:r",
            vec!["inside".into(), "recent".into()],
            vec!["key_notes".into()],
        )
        .unwrap()
        .unwrap();
    let keys: Vec<&str> = panel.sections.iter().map(|s| s.key.as_str()).collect();
    assert_eq!(keys[..2], ["inside", "recent"]);
    assert!(!keys.contains(&"key_notes"));
    let listed = home.sections("book:r").unwrap();
    assert_eq!(listed[0].key, "inside");
    assert!(listed.iter().find(|s| s.key == "key_notes").unwrap().hidden);
    assert!(store.page_cache("book:r").unwrap().layout.is_some());
    assert!(
        home.customise("book:r", vec!["nope".into()], vec![])
            .is_err()
    );
    let panel = home.reset_layout("book:r").unwrap().unwrap();
    assert_eq!(panel.sections[0].key, "key_notes");
}

// no-prop-test: a locked notebook shows nothing closed, computed sections open, and loses its cache when locked.
#[test]
fn a_locked_notebook_shows_nothing_until_opened_and_never_keeps_a_cache() {
    let model = Scripted::new("- We decided to use the 2019 dataset, not 2021 [1:L2]\n");
    let (phone, ai) = ai_phone(&model);
    let store = world(&phone);
    let home = phone.library.notebook_home_with_ai(ai);
    home.refresh_section("book:r", "decisions").unwrap();
    assert!(!store.page_cache("book:r").unwrap().is_empty());
    let locks = phone.library.locks();
    locks
        .lock_notebook("book:r", Some(LOCK_PASSWORD.into()))
        .unwrap();
    assert!(
        !store
            .get("book:r")
            .unwrap()
            .unwrap()
            .fields
            .contains_key("kansoPage"),
        "the cache is removed with the lock"
    );
    assert!(!home.card("book:r").unwrap().show);

    // Open this session: computed sections from the opened notes, no AI.
    let panel = home.open("book:r").unwrap().unwrap();
    assert!(panel.locked_open && !panel.ai_on && !panel.locked);
    assert_eq!(panel.footer, home.words().locked);
    assert!(panel.sections.iter().all(|s| !s.ai));
    assert!(panel.sections.iter().any(|s| s.key == "open_tasks"));
    assert!(
        store.page_cache("book:r").unwrap().is_empty(),
        "nothing written to disk"
    );
    assert!(home.refresh_section("book:r", "about").unwrap().is_none());

    // Locked again: nothing.
    locks.lock().unwrap();
    let panel = home.open("book:r").unwrap().unwrap();
    assert!(panel.locked && panel.sections.is_empty() && panel.header.line.is_empty());
    let _ = NewNote::default();
}
