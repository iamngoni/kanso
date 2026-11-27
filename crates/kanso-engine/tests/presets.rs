//! The Video studio preset: a notebook with its stage board and its two
//! templates, made whole, and the template list that puts them first.

use kanso_core::DocType;
use kanso_core::TagColor;
use kanso_core::dates::{Date, DateTime};
use kanso_core::properties;
use kanso_core::schema::PropertyType;
use kanso_engine::board::{Board, Lane, grouped_board, property_board};
use kanso_engine::catalogue::book_icons::CHOICES;
use kanso_engine::presets::{
    CreatedNotebook, NotebookKind, SHOT_LIST, STUDIO_STAGES, VIDEO_BRIEF, create_notebook,
    create_notebook_at,
};
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

// MARK: Starters

fn on(year: i32, month: u32, day: u32) -> chrono::NaiveDateTime {
    chrono::NaiveDate::from_ymd_opt(year, month, day)
        .unwrap()
        .and_hms_opt(9, 30, 0)
        .unwrap()
}

fn doc(store: &Store, id: &str) -> serde_json::Value {
    store.get_api(id, None, false).unwrap()
}

fn make(store: &Store, name: &str, kind: NotebookKind) -> CreatedNotebook {
    create_notebook_at(store, name, None, kind, Origin::Ui, on(2026, 11, 27)).unwrap()
}

fn lock_book(store: &Store, id: &str) {
    let mut book = doc(store, id);
    book.as_object_mut().unwrap().remove("count");
    book["kansoLock"] =
        json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "notebook"});
    store.lock_book(book, Vec::new()).unwrap();
}

#[test]
fn the_kinds_are_the_eight_choices_in_order() {
    let keys: Vec<&str> = NotebookKind::ALL.iter().map(|k| k.key()).collect();
    assert_eq!(
        keys,
        [
            "blank",
            "product",
            "knowledge",
            "side-projects",
            "journal",
            "reading",
            "publishing",
            "video-studio"
        ]
    );
    let labels: Vec<&str> = NotebookKind::ALL.iter().map(|k| k.label()).collect();
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
            "Video studio"
        ]
    );
    assert_eq!(NotebookKind::parse("nope"), None);
    assert_eq!(NotebookKind::parse(""), None);
}

#[test]
fn every_icon_is_one_the_picker_offers() {
    let names: Vec<&str> = CHOICES.iter().map(|c| c.name).collect();
    for kind in NotebookKind::ALL {
        assert!(names.contains(&kind.icon()), "{kind:?}");
        let Some(starter) = kind.starter(on(2026, 11, 27).date()) else {
            continue;
        };
        for book in &starter.children {
            assert!(names.contains(&book.icon), "{} in {kind:?}", book.icon);
        }
    }
}

/// What each starter makes: sub-notebooks with icons and templates, then tags.
type Layout = (
    NotebookKind,
    &'static [(&'static str, &'static str, &'static str)],
    &'static [(&'static str, TagColor)],
);

const LAYOUTS: &[Layout] = &[
    (
        NotebookKind::Product,
        &[
            ("Features", "lightbulb", "Feature plan"),
            ("Bugs", "bug", "Bug report"),
            ("Performance", "zap", "Performance investigation"),
            ("Experiments", "flask-conical", "Experiment"),
            ("Refactoring", "wrench", "Refactoring plan"),
            ("Testing", "list-checks", "Test plan"),
            ("Chores", "hammer", "Chore"),
            ("Release notes", "megaphone", "Release plan"),
            ("Decisions", "git-branch", "Decision record"),
        ],
        &[
            ("web", TagColor::Blue),
            ("server", TagColor::Violet),
            ("mobile", TagColor::Green),
            ("docs", TagColor::Grey),
            ("security", TagColor::Red),
            ("important", TagColor::Orange),
        ],
    ),
    (
        NotebookKind::Knowledge,
        &[
            ("Learning", "graduation-cap", "Lesson"),
            ("Research", "microscope", "Research note"),
            ("Troubleshooting", "wrench", "Fix"),
            ("Tools", "hammer", "Tool note"),
            ("Architecture", "building-2", "Architecture note"),
        ],
        &[
            ("important", TagColor::Orange),
            ("read-later", TagColor::Teal),
        ],
    ),
    (
        NotebookKind::SideProjects,
        &[
            ("My first project", "rocket", "Project brief"),
            ("Ideas", "lightbulb", "Idea"),
        ],
        &[],
    ),
    (
        NotebookKind::Journal,
        &[("2026", "calendar", "Daily entry")],
        &[],
    ),
    (
        NotebookKind::Reading,
        &[
            ("Books", "book-open", "Book notes"),
            ("Articles", "newspaper", "Article notes"),
        ],
        &[("to-read", TagColor::Blue), ("favorite", TagColor::Pink)],
    ),
    (
        NotebookKind::Publishing,
        &[
            ("Ideas", "lightbulb", "Idea"),
            ("Posts", "file-text", "Post outline"),
            ("Videos", "video", "Video outline"),
            ("Analytics", "chart-line", "Monthly numbers"),
            ("Resources", "bookmark", "Resource"),
        ],
        &[
            ("draft", TagColor::Grey),
            ("in-review", TagColor::Yellow),
            ("published", TagColor::Green),
        ],
    ),
];

#[test]
fn each_starter_makes_exactly_its_catalogue() {
    for (kind, books, tags) in LAYOUTS {
        let (_dir, store) = store();
        let made = make(&store, "  Mine ", *kind);
        let root = doc(&store, &made.book_id);
        assert_eq!(root["name"], "Mine", "{kind:?}");
        assert_eq!(root["kansoIcon"], kind.icon(), "{kind:?}");

        assert_eq!(made.child_ids.len(), books.len(), "{kind:?}");
        assert_eq!(made.note_ids.len(), books.len(), "{kind:?}");
        assert_eq!(made.template_ids.len(), books.len(), "{kind:?}");
        for (i, (name, icon, template)) in books.iter().enumerate() {
            let book = doc(&store, &made.child_ids[i]);
            assert_eq!(book["name"], *name, "{kind:?}");
            assert_eq!(book["kansoIcon"], *icon, "{name}");
            assert_eq!(book["parentBookId"], made.book_id.as_str());
            // Named template, and its note.
            assert_eq!(
                book["kansoTemplates"],
                json!([made.template_ids[i]]),
                "{name}"
            );
            assert_eq!(doc(&store, &made.template_ids[i])["title"], *template);
            let note = doc(&store, &made.note_ids[i]);
            assert_eq!(note["bookId"], made.child_ids[i].as_str());
            assert_eq!(note["status"], "active");
            if *kind != NotebookKind::Journal {
                assert_eq!(note["title"], *template, "{name}");
            }
            // Variables are filled in: none of ours is left in the text.
            let body = note["body"].as_str().unwrap();
            assert!(!body.contains("{{"), "{name}: {body}");
            assert!(body.contains("##"), "{name}");
        }

        assert_eq!(made.tag_ids.len(), tags.len(), "{kind:?}");
        for (i, (name, color)) in tags.iter().enumerate() {
            let tag = doc(&store, &made.tag_ids[i]);
            assert_eq!(tag["name"], *name);
            assert_eq!(tag["color"], serde_json::to_value(color).unwrap());
        }
        assert_eq!(
            store.list_api(DocType::Tag, None, 0).unwrap().len(),
            tags.len()
        );
        // The root, its children and the Templates notebook, nothing else.
        assert_eq!(store.books().unwrap().len(), 2 + books.len(), "{kind:?}");
        // No notes but the starters', and the templates.
        let notes = store.list_api(DocType::Note, None, 0).unwrap().len();
        assert_eq!(notes, 2 * books.len(), "{kind:?}");
    }
}

#[test]
fn the_journal_is_a_year_with_todays_entry() {
    let (_dir, store) = store();
    let made = make(&store, "Journal", NotebookKind::Journal);
    assert_eq!(doc(&store, &made.child_ids[0])["name"], "2026");
    let note = doc(&store, &made.note_ids[0]);
    assert_eq!(note["title"], "2026-11-27");
    assert!(note["body"].as_str().unwrap().contains("## What I did"));
}

#[test]
fn the_decision_record_is_dated_today() {
    let (_dir, store) = store();
    let made = make(&store, "Product", NotebookKind::Product);
    let decisions = &made.note_ids[8];
    let body = doc(&store, decisions)["body"].as_str().unwrap().to_owned();
    assert!(body.starts_with("Status: proposed \u{b7} 2026-11-27\n"));
    assert!(body.contains("## Alternatives"));
}

#[test]
fn a_starter_goes_inside_its_parent() {
    let (_dir, store) = store();
    let parent = create_notebook(&store, "Work", None, NotebookKind::Blank, Origin::Ui).unwrap();
    let made = create_notebook(
        &store,
        "Reading",
        Some(parent.book_id.clone()),
        NotebookKind::Reading,
        Origin::Ui,
    )
    .unwrap();
    assert_eq!(
        doc(&store, &made.book_id)["parentBookId"],
        parent.book_id.as_str()
    );
}

#[test]
fn a_second_run_reuses_the_templates_and_the_tags() {
    let (_dir, store) = store();
    let first = make(&store, "One", NotebookKind::Product);
    let second = make(&store, "Two", NotebookKind::Product);
    assert_eq!(first.template_ids, second.template_ids);
    assert_eq!(first.tag_ids, second.tag_ids);
    assert_ne!(first.book_id, second.book_id);
    assert_ne!(first.note_ids, second.note_ids);
    assert_eq!(store.templates().unwrap().len(), 9);
    assert_eq!(store.list_api(DocType::Tag, None, 0).unwrap().len(), 6);
    // The second notebook's subnotebooks name the same templates.
    assert_eq!(
        store.notebook_templates(&second.child_ids[1]).unwrap(),
        [first.template_ids[1].clone()]
    );
}

#[test]
fn the_idea_template_is_shared_between_starters() {
    let (_dir, store) = store();
    let side = make(&store, "Side", NotebookKind::SideProjects);
    let publishing = make(&store, "Writing", NotebookKind::Publishing);
    assert_eq!(side.template_ids[1], publishing.template_ids[0]);
    assert_eq!(store.templates().unwrap().len(), 2 + 4);
}

#[test]
fn an_existing_tag_keeps_its_color_whatever_its_case() {
    let (_dir, store) = store();
    let mine = store
        .put(
            DocType::Tag,
            json!({"name": "Important", "color": "black"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    let made = make(&store, "Know", NotebookKind::Knowledge);
    assert_eq!(made.tag_ids[0], mine);
    assert_eq!(doc(&store, &mine)["color"], "black");
    assert_eq!(store.list_api(DocType::Tag, None, 0).unwrap().len(), 2);
}

#[test]
fn a_locked_parent_refuses_the_starters_that_make_notes() {
    let (_dir, store) = store();
    let secret = create_notebook(&store, "Secret", None, NotebookKind::Blank, Origin::Ui).unwrap();
    lock_book(&store, &secret.book_id);
    let books = store.books().unwrap().len();
    for kind in NotebookKind::ALL.into_iter().filter(|k| k.makes_notes()) {
        let refused = create_notebook(
            &store,
            "Inside",
            Some(secret.book_id.clone()),
            kind,
            Origin::Ui,
        );
        assert!(
            matches!(
                &refused,
                Err(kanso_store::StoreError::Invalid(m)) if m
                    == "Starters can't be made inside a locked notebook. Make it at the top level, then move it in."
            ),
            "{kind:?}: {refused:?}"
        );
    }
    assert_eq!(store.books().unwrap().len(), books);
    assert!(store.templates().unwrap().is_empty());
    assert!(store.list_api(DocType::Tag, None, 0).unwrap().is_empty());
    // Blank and Video studio still go in.
    for kind in [NotebookKind::Blank, NotebookKind::VideoStudio] {
        create_notebook(
            &store,
            "Inside",
            Some(secret.book_id.clone()),
            kind,
            Origin::Ui,
        )
        .unwrap();
    }
}

#[test]
fn a_refused_starter_leaves_nothing_behind() {
    let (_dir, store) = store();
    for (name, parent) in [("", None), ("  ", None), ("Notes", Some("book:nowhere"))] {
        let made = create_notebook(
            &store,
            name,
            parent.map(str::to_owned),
            NotebookKind::Product,
            Origin::Ui,
        );
        assert!(made.is_err(), "{name:?} in {parent:?}");
    }
    assert!(store.books().unwrap().is_empty());
    assert!(store.templates().unwrap().is_empty());
    assert!(store.list_api(DocType::Tag, None, 0).unwrap().is_empty());
    assert!(store.list_api(DocType::Note, None, 0).unwrap().is_empty());
}

#[test]
fn every_template_text_is_a_guide() {
    for kind in NotebookKind::ALL {
        let Some(starter) = kind.starter(on(2026, 11, 27).date()) else {
            continue;
        };
        for template in starter.children.iter().flat_map(|b| &b.templates) {
            assert!(template.body.contains("## "), "{}", template.title);
            assert!(template.body.ends_with('\n'), "{}", template.title);
            // Only variables Kanso knows.
            for part in template.body.split("{{").skip(1) {
                let name = part.split("}}").next().unwrap();
                let name = name.split(':').next().unwrap().trim();
                assert!(kanso_core::template::KNOWN.contains(&name), "{name}");
            }
            assert!(
                template.key.len() <= 64
                    && template
                        .key
                        .bytes()
                        .all(|b| b.is_ascii_lowercase() || b == b'/' || b == b'-'),
                "{}",
                template.key
            );
        }
    }
}

proptest! {
    #![proptest_config(config())]

    /// A key names its kind and back, and nothing else parses.
    #[test]
    fn prop_key_and_parse_round_trip(junk in "[a-z-]{0,14}") {
        for kind in NotebookKind::ALL {
            prop_assert_eq!(NotebookKind::parse(kind.key()), Some(kind));
        }
        let parsed = NotebookKind::parse(&junk);
        prop_assert_eq!(
            parsed.is_some(),
            NotebookKind::ALL.iter().any(|k| k.key() == junk)
        );
    }

    /// Whatever day it is, the journal's year and note are that day's, and
    /// the starters' keys never repeat inside one starter.
    #[test]
    fn prop_the_journal_follows_the_date(days in 0i64..80_000) {
        let date = chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap() + chrono::Duration::days(days);
        let starter = NotebookKind::Journal.starter(date).unwrap();
        prop_assert_eq!(starter.children.len(), 1);
        prop_assert_eq!(starter.children[0].name.clone(), date.format("%Y").to_string());
        prop_assert_eq!(starter.children[0].note_title.clone(), date.format("%Y-%m-%d").to_string());
    }

    /// However a name is spaced, a starter trims it and makes the same layout.
    #[test]
    fn prop_a_starter_trims_its_name(
        name in "[A-Za-z][A-Za-z0-9]{0,10}",
        left in " {0,3}",
        right in " {0,3}",
    ) {
        let (_dir, store) = store();
        let made = make(&store, &format!("{left}{name}{right}"), NotebookKind::Reading);
        let book = doc(&store, &made.book_id);
        prop_assert_eq!(book["name"].as_str(), Some(name.as_str()));
        prop_assert_eq!(made.child_ids.len(), 2);
    }
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
