//! People & Projects on a phone (issue #200): the list, the suggestions, a
//! page's live sections with and without AI, what is kept and hidden, and
//! that a note kept out of AI is never sent. CI pins the seed with
//! `PROPTEST_RNG_SEED`.

mod common;

use common::model::*;
use common::*;
use kanso_core::DocType;
use kanso_mobile::{PageSectionState, PeopleKind, PeopleSettings};
use kanso_store::{Origin, Put, Store};
use serde_json::json;

const CANARY: &str = "zebracanary4471";

fn store(phone: &Phone) -> Store {
    Store::open(phone.path()).unwrap()
}

fn lock() -> serde_json::Value {
    json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "sealed"})
}

/// A person page for Sam, and notes that mention him.
fn world(phone: &Phone) -> (String, String, Store) {
    let store = store(phone);
    let people = phone.library.people();
    let sam = people
        .create(PeopleKind::Person, "Sam Moyo".into(), vec!["Sam".into()])
        .unwrap();
    let book = book(&store, "Work");
    let weekly = raw_note(
        &store,
        json!({"title": "Weekly sync", "bookId": book, "createdAt": 2_000_000_000_000_i64,
            "body": "I promised Sam the revised contract by Friday\n- [ ] Send Sam the pricing deck @2026-11-04\n"}),
    );
    raw_note(
        &store,
        json!({"title": "Lusaka", "bookId": book, "createdAt": 1_000_000_000_000_i64, "body": "Sam said the office opens in March\n"}),
    );
    (sam, weekly, store)
}

// no-prop-test: settings defaults and one round trip.
#[test]
fn settings_default_on_and_round_trip() {
    let phone = lone_phone();
    let people = phone.library.people();
    let settings = people.settings();
    assert!(settings.suggest && settings.write_with_ai && !settings.detect_with_ai);
    assert_eq!(
        (settings.refresh.as_str(), settings.keep_in.as_str()),
        ("auto", "")
    );
    let saved = people
        .set_settings(PeopleSettings {
            refresh: "press".into(),
            suggest: false,
            ..settings
        })
        .unwrap();
    assert_eq!(saved.refresh, "press");
    assert!(!people.settings().suggest);
    assert_eq!(people.detect(true).unwrap(), 0, "suggestions are off");
}

// no-prop-test: one scripted tour of the pages and their sections.
#[test]
fn a_page_shows_computed_sections_and_off_lines_without_ai() {
    let phone = lone_phone();
    let (sam, weekly, _store) = world(&phone);
    let people = phone.library.people();
    assert_eq!(people.status().unwrap().pages, 1);
    let listed = people.pages().unwrap();
    assert_eq!(
        (listed[0].title.as_str(), listed[0].kind),
        ("Sam Moyo", PeopleKind::Person)
    );
    assert_eq!(listed[0].notes, 2);
    assert!(listed[0].detail.starts_with("2 notes"));

    assert!(
        people.page(&weekly).unwrap().is_none(),
        "an ordinary note has no panel"
    );
    let panel = people.page(&sam).unwrap().unwrap();
    assert_eq!(panel.header.title, "Sam Moyo");
    assert!(
        panel.header.line.contains("Also known as Sam"),
        "{}",
        panel.header.line
    );
    assert!(!panel.ai_on);
    let keys: Vec<&str> = panel.sections.iter().map(|s| s.key.as_str()).collect();
    assert_eq!(
        keys,
        vec!["open_items", "promises", "discussed", "timeline", "related"]
    );
    let open = &panel.sections[0];
    assert_eq!(open.items.len(), 1);
    assert!(open.items[0].task.is_some());
    assert_eq!(open.items[0].sources[0].note_id, weekly);
    let promises = &panel.sections[1];
    assert_eq!(promises.state, PageSectionState::Off);
    assert!(promises.empty_line.contains("Turn on AI"));
    assert_eq!(panel.sections[3].items.len(), 3);
    assert!(panel.due.is_empty());
    // Refresh is refused while AI is off.
    assert!(people.refresh_section(&sam, "promises").is_err());
}

// no-prop-test: one scripted model reply, checked against the lines it was sent.
#[test]
fn ai_sections_cite_lines_and_can_be_kept_hidden_copied_and_marked_wrong() {
    let model = Scripted::new(
        "- YOU: I promised Sam the revised contract by Friday [1:L1]\n- THEM: made-up [4:L9]\n",
    );
    let (phone, ai) = ai_phone(&model);
    let (sam, weekly, store) = world(&phone);
    // An excluded and a locked note never reach the model.
    let exclude = tag(&store, "ai-exclude");
    raw_note(
        &store,
        json!({"title": "Excluded", "tags": [exclude], "body": format!("{CANARY} Sam Moyo promised excluded things\n")}),
    );
    let locked = raw_note(
        &store,
        json!({"title": "Locked", "body": format!("{CANARY} Sam Moyo promised locked things\n")}),
    );
    let rev = store.get(&locked).unwrap().unwrap().rev;
    store
        .lock_note(
            json!({"_id": locked, "_rev": rev, "title": "Locked", "body": "",
            "bookId": store.get(&locked).unwrap().unwrap().fields["bookId"], "kansoLock": lock()}),
        )
        .unwrap();
    let people = phone.library.people_with_ai(ai);
    assert!(people.page(&sam).unwrap().unwrap().ai_on);
    let panel = people.refresh_section(&sam, "promises").unwrap().unwrap();
    let promises = panel.sections.iter().find(|s| s.key == "promises").unwrap();
    assert_eq!(promises.items.len(), 1, "{promises:?}");
    assert_eq!(promises.items[0].who, "you");
    assert_eq!(promises.items[0].sources[0].note_id, weekly);
    assert_eq!(promises.state, PageSectionState::Fresh);
    assert!(
        !model.everything_sent().contains(CANARY),
        "kept-out notes are not sent"
    );
    let discussed = panel
        .sections
        .iter()
        .find(|s| s.key == "discussed")
        .unwrap();
    assert_eq!(discussed.state, PageSectionState::Unwritten);

    // Keep it, hide another, copy it into the note, mark a bullet wrong.
    let panel = people.pin(&sam, "promises", true).unwrap().unwrap();
    assert_eq!(panel.sections[1].state, PageSectionState::Pinned);
    let panel = people.hide(&sam, "timeline", true).unwrap().unwrap();
    assert!(panel.sections.iter().all(|s| s.key != "timeline"));
    assert_eq!(panel.hidden[0].title, "Timeline");
    let panel = people.show_hidden(&sam).unwrap().unwrap();
    assert!(panel.hidden.is_empty());
    assert!(
        people
            .section_markdown(&sam, "promises")
            .unwrap()
            .contains("inkdrop://note/")
    );
    assert!(people.copy_into_note(&sam, "promises").unwrap());
    let body = store.note(&sam).unwrap().unwrap().body;
    assert!(
        body.contains("## Promises") && body.contains("inkdrop://note/"),
        "{body}"
    );
    let panel = people.not_right(&sam, "promises", 0).unwrap().unwrap();
    assert!(panel.sections[1].items.is_empty());
    // Refresh All leaves a kept section alone.
    let kept = store.page_cache(&sam).unwrap().sections["promises"].clone();
    people.refresh_all(&sam).unwrap();
    assert_eq!(store.page_cache(&sam).unwrap().sections["promises"], kept);
}

// no-prop-test: one scripted tour of suggestions, dismissals and merging.
#[test]
fn suggestions_come_from_signals_and_pages_merge() {
    let phone = lone_phone();
    let store = store(&phone);
    let people = phone.library.people();
    let book = book(&store, "Notes");
    for ix in 0..3 {
        raw_note(
            &store,
            json!({"title": format!("Lunch {ix}"), "bookId": book, "body": "Lunch with Tendai Chirwa went well\n@rudo follows up\n"}),
        );
    }
    assert_eq!(people.detect(true).unwrap(), 2);
    let found = people.suggestions().unwrap();
    assert_eq!(found.len(), 2);
    assert_eq!(people.status().unwrap().new_label, "2 new");
    assert!(store.entities().unwrap().is_empty(), "nothing was made");
    let rudo = found.iter().find(|s| s.name == "rudo").unwrap();
    people.dismiss(rudo.key.clone(), true).unwrap();
    assert_eq!(people.suggestions().unwrap().len(), 1);
    assert_eq!(people.dismissed().unwrap(), vec!["rudo".to_owned()]);
    people.suggest_again("rudo".into()).unwrap();
    people.detect(true).unwrap();
    assert_eq!(people.suggestions().unwrap().len(), 2);

    let a = people
        .create(PeopleKind::Person, "Tendai Chirwa".into(), vec![])
        .unwrap();
    let b = people
        .create(PeopleKind::Person, "T. Chirwa".into(), vec![])
        .unwrap();
    assert!(people.add_alias(a.clone(), "Tendai".into()).unwrap());
    let done = people.merge(a.clone(), b).unwrap();
    assert_eq!(done.into, a);
    assert_eq!(people.pages().unwrap().len(), 1);
    assert!(people.merge(a.clone(), a).is_err());
    assert!(people.detect_with_ai().is_err(), "off until chosen");
}

// no-prop-test: a locked page keeps only its own text.
#[test]
fn a_locked_page_has_no_live_sections() {
    let phone = lone_phone();
    let (sam, _, store) = world(&phone);
    let people = phone.library.people();
    let rev = store.get(&sam).unwrap().unwrap().rev;
    let book = store.get(&sam).unwrap().unwrap().fields["bookId"].clone();
    store
        .lock_note(
            json!({"_id": sam, "_rev": rev, "title": "Sam Moyo", "body": "", "bookId": book,
            "kansoLock": lock()}),
        )
        .unwrap();
    assert!(people.page(&sam).unwrap().is_none());
    assert!(people.pages().unwrap().is_empty());
    let _ = DocType::Note;
    let _ = (Origin::Ui, Put::Upsert);
}

// no-prop-test: one note previewed with one `@name` of a page.
#[test]
fn the_preview_links_an_at_mention_to_its_page() {
    let phone = lone_phone();
    let (sam, _weekly, store) = world(&phone);
    let book = book(&store, "Home");
    let id = raw_note(
        &store,
        json!({"title": "Call", "bookId": book, "body": "Rang @Sam about it, mail a@b.com\n"}),
    );
    let theme = r##"{"name":"t","appearance":"light","ui":{"editor":"#ffffff","text":"#222222","accent":"#2185d0"},"syntax":{},"preview":{}}"##;
    let palette = kanso_mobile::Renderer::theme_tokens(theme.into())
        .unwrap()
        .palette;
    let page = phone.library.renderer().preview_html(&id, palette).unwrap();
    let url = format!("inkdrop://note/{}", sam.trim_start_matches("note:"));
    assert!(page.contains(&url), "{page}");
    // A tap on it opens the page.
    let target = phone.library.renderer().link_target(&id, &url).unwrap();
    assert!(format!("{target:?}").contains(&sam), "{target:?}");
    assert!(!page.contains("inkdrop://note/a@b"), "{page}");
}
