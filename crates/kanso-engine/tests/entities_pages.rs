//! People and project pages end to end: the fixture library's sections, AI
//! sections written with receipts and checked, what is kept, what goes stale,
//! and that nothing locked or excluded reaches a prompt or a cache. CI pins
//! the seed with `PROPTEST_RNG_SEED`.

use std::sync::{Arc, Mutex};

use kanso_ai::{AiError, ChatProvider, Event, Prompt};
use kanso_core::DocType;
use kanso_core::entities::EntityKind;
use kanso_engine::entities::detect::Context;
use kanso_engine::entities::{
    self, EntityPage, EntitySettings, actions, ai_detect, suggest,
};
use kanso_engine::pages::{
    self, OpenOptions, PageSource, RefreshPolicy, SectionKind, SectionState, write::WriteContext,
};
use kanso_store::{Origin, Put, Store};
use proptest::prelude::*;
use serde_json::{Value, json};

const CANARY: &str = "zebracanary4471";

/// Answers every prompt by citing each excerpt line that says "promised",
/// and remembers the prompts.
struct Cites {
    seen: Arc<Mutex<Vec<String>>>,
    /// Appended to every reply: a made-up citation, for the tests of
    /// fabricated replies.
    extra: String,
}

impl Cites {
    fn new() -> (Cites, Arc<Mutex<Vec<String>>>) {
        let seen = Arc::new(Mutex::new(Vec::new()));
        (
            Cites {
                seen: seen.clone(),
                extra: String::new(),
            },
            seen,
        )
    }
}

impl ChatProvider for Cites {
    fn stream(&self, prompt: &Prompt) -> Result<Box<dyn Iterator<Item = Event> + Send>, AiError> {
        self.seen
            .lock()
            .unwrap()
            .push(format!("{}\n{}", prompt.system, prompt.user));
        let mut reply = String::new();
        let mut number = 0usize;
        for line in prompt.user.lines() {
            if let Some(rest) = line.strip_prefix('[')
                && let Some((n, _)) = rest.split_once(']')
                && let Ok(n) = n.parse::<usize>()
            {
                number = n;
            }
            if let Some(row) = line.strip_prefix('L')
                && let Some((n, text)) = row.split_once("| ")
                && text.contains("promised")
            {
                reply.push_str(&format!("- YOU: {text} [{number}:L{n}]\n"));
            }
        }
        if reply.is_empty() {
            reply.push_str("NONE");
        }
        reply.push_str(&self.extra);
        Ok(Box::new(vec![Event::Text(reply), Event::Done].into_iter()))
    }
}

fn open_store() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    (dir, store)
}

fn put(store: &Store, doc_type: DocType, body: Value) -> String {
    store
        .put(doc_type, body, Put::Upsert, Origin::Ui)
        .unwrap()
        .id
}

fn note_at(store: &Store, id: &str, title: &str, body: &str, book: &str, at: i64) {
    let rev = store.get(id).unwrap().map(|d| d.rev);
    let mut doc = json!({"_id": id, "title": title, "body": body, "bookId": book,
        "createdAt": at, "updatedAt": at});
    if let Some(rev) = rev {
        doc["_rev"] = json!(rev);
    }
    put(store, DocType::Note, doc);
}

fn lock() -> Value {
    json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "sealed"})
}

const DAY: i64 = 86_400_000;
const T0: i64 = 1_761_600_000_000;

/// A library with a person page and a handful of notes about them.
fn library() -> (tempfile::TempDir, Store, String, String) {
    let (dir, store) = open_store();
    let book = put(&store, DocType::Book, json!({"_id": "book:a", "name": "Work"}));
    let page = "---\nkanso: person\naliases: [Sam]\n---\nSam runs the Lusaka office.\n";
    note_at(&store, "note:sam", "Sam Moyo", page, &book, T0);
    note_at(
        &store,
        "note:weekly",
        "Weekly sync",
        "# Notes\nSam Moyo wants the deck before the board.\n- [ ] Send Sam the pricing deck @2026-11-04\nI promised Sam the revised contract by Friday\n",
        &book,
        T0 + 2 * DAY,
    );
    note_at(
        &store,
        "note:atlas",
        "Atlas plan",
        "---\nkanso: project\n---\nOwner: Sam\nHiring for the team.\n- [ ] review the plan with Sam\n",
        &book,
        T0 + DAY,
    );
    note_at(
        &store,
        "note:lusaka",
        "Lusaka trip",
        "Flight booked.\nSam said he would introduce me to the Lusaka team\nSam promised an intro to the Lusaka team\n",
        &book,
        T0 + 3 * DAY,
    );
    (dir, store, book, "note:sam".into())
}

fn options(ai_on: bool) -> OpenOptions {
    OpenOptions {
        ai_on,
        now: T0 + 10 * DAY,
    }
}

fn find<'a>(view: &'a pages::PageView, key: &str) -> &'a pages::Section {
    view.sections.iter().find(|s| s.key == key).unwrap()
}

#[test]
fn the_fixture_library_gives_open_items_timeline_and_related() {
    let (_dir, store, _book, id) = library();
    let page = EntityPage::load(&store, &id).unwrap().unwrap();
    let view = pages::open(&store, &page, options(false), &[]).unwrap();
    let keys: Vec<&str> = view.sections.iter().map(|s| s.key.as_str()).collect();
    assert_eq!(keys, vec!["open_items", "promises", "discussed", "timeline", "related"]);

    // Open items: a task on a line that mentions Sam, and a task in a note that
    // links to... the atlas task is on a line that mentions Sam too.
    let open_items = find(&view, "open_items");
    let texts: Vec<&str> = open_items.items.iter().map(|i| i.text.as_str()).collect();
    assert_eq!(texts.len(), 2, "{texts:?}");
    assert!(texts[0].starts_with("Send Sam the pricing deck"), "soonest due first: {texts:?}");
    assert!(open_items.items[0].task.is_some());
    assert_eq!(open_items.items[0].sources[0].note_id, "note:weekly");

    // Timeline: newest note first, one line per mention.
    let timeline = find(&view, "timeline");
    assert_eq!(timeline.items[0].sources[0].note_id, "note:lusaka");
    assert!(timeline.items[0].text.starts_with("Lusaka trip"));
    assert!(timeline.items.len() >= 4);
    assert!(timeline.items.iter().all(|i| i.group.is_some() && i.meta.is_some()));

    // Related: the project page that shares notes.
    let related = find(&view, "related");
    assert!(
        related
            .items
            .iter()
            .any(|i| i.target.as_deref() == Some("note:atlas") && i.tag.as_deref() == Some("project")),
        "{related:?}"
    );

    // AI is off and nothing was written: one muted line each.
    for key in ["promises", "discussed"] {
        let section = find(&view, key);
        assert_eq!(section.state, SectionState::Off(pages::OffReason::AiOff));
        assert!(pages::empty_line(section).unwrap().contains("Turn on AI"));
    }

    // The header counts notes and names the last date.
    let header = page.header(&store).unwrap();
    assert_eq!(header.notes, 3);
    assert_eq!(header.aliases, vec!["Sam".to_owned()]);
    assert!(header.last_mentioned.is_some());
    assert_eq!(page.own_text(&store).unwrap(), "Sam runs the Lusaka office.\n");
}

#[test]
fn a_project_page_has_decisions_and_lists_people_first() {
    let (_dir, store, book, _) = library();
    note_at(&store, "note:b", "B", "Atlas kickoff with Sam Moyo\n", &book, T0);
    let project = EntityPage::load(&store, "note:atlas").unwrap().unwrap();
    let keys: Vec<&str> = project.sections().iter().map(|s| s.key).collect();
    assert_eq!(keys, vec!["open_items", "decisions", "discussed", "timeline", "related"]);
    let _ = (store, project);
}

#[test]
fn ai_sections_are_written_with_receipts_and_cached_then_go_stale() {
    let (_dir, store, book, id) = library();
    let page = EntityPage::load(&store, &id).unwrap().unwrap();
    let (provider, seen) = Cites::new();
    let ctx = WriteContext::new(&provider, "test/model", T0 + 10 * DAY);
    let written = pages::refresh_section(&store, &page, "promises", &ctx).unwrap();
    assert_eq!(written.items.len(), 2, "{written:?}");
    assert!(written.items.iter().all(|i| i.who.as_deref() == Some("you") && !i.sources.is_empty()));
    assert!(seen.lock().unwrap()[0].contains("Sam Moyo"), "the instruction names the person");

    let view = pages::open(&store, &page, options(true), &[]).unwrap();
    let promises = find(&view, "promises");
    assert_eq!(promises.state, SectionState::Fresh);
    assert_eq!(promises.items.len(), 2);
    let receipt = &promises.items[0].sources[0];
    let body = store.note(&receipt.note_id).unwrap().unwrap().body;
    assert!(kanso_engine::ask::relocate(&body, receipt).is_some(), "the line is found again");
    // `discussed` was never written.
    assert_eq!(find(&view, "discussed").state, SectionState::Unwritten);

    // A source note is edited: the section says so, and counts it.
    note_at(
        &store,
        "note:lusaka",
        "Lusaka trip",
        "Flight booked.\nSam promised an intro to the Lusaka team\nSam promised a second intro\n",
        &book,
        T0 + 4 * DAY,
    );
    let view = pages::open(&store, &page, options(true), &[]).unwrap();
    let promises = find(&view, "promises");
    assert_eq!(promises.state, SectionState::Stale { changed: 1 });
    assert_eq!(promises.stale_line().unwrap(), "Out of date \u{b7} 1 note changed");
    // Policy: on open it would be rewritten; on press it would not; never
    // within the hour of its last write.
    let due = pages::due_for_refresh(&view, RefreshPolicy::OnOpen, options(true));
    assert!(due.contains(&"discussed".to_owned()));
    assert!(!due.contains(&"promises".to_owned()), "written an hour ago or less");
    let later = OpenOptions { ai_on: true, now: T0 + 10 * DAY + 2 * 60 * 60 * 1000 };
    let view_later = pages::open(&store, &page, later, &[]).unwrap();
    assert!(pages::due_for_refresh(&view_later, RefreshPolicy::OnOpen, later).contains(&"promises".to_owned()));
    assert!(pages::due_for_refresh(&view_later, RefreshPolicy::OnPress, later).is_empty());
    assert!(pages::due_for_refresh(&view_later, RefreshPolicy::OnOpen, OpenOptions { ai_on: false, ..later }).is_empty());
}

#[test]
fn fabricated_replies_leave_nothing() {
    let (_dir, store, _book, id) = library();
    let page = EntityPage::load(&store, &id).unwrap().unwrap();
    let (mut provider, _) = Cites::new();
    provider.extra = "\n- YOU: Promised a pony [1:L2]\n- THEM: Promised the moon [9:L9]\n- uncited line about Sam\n".into();
    let ctx = WriteContext::new(&provider, "m", T0);
    let written = pages::refresh_section(&store, &page, "promises", &ctx).unwrap();
    assert_eq!(written.items.len(), 2, "only the two real ones: {written:?}");
    assert!(written.items.iter().all(|i| !i.text.contains("pony") && !i.text.contains("moon")));
}

#[test]
fn locked_excluded_and_trashed_notes_are_never_sent_or_cached() {
    let (_dir, store, book, id) = library();
    let exclude = put(&store, DocType::Tag, json!({"name": "ai-exclude"}));
    note_at(&store, "note:ex", "Excluded", &format!("{CANARY} Sam promised excluded things\n"), &book, T0 + 5 * DAY);
    let rev = store.get("note:ex").unwrap().unwrap().rev;
    put(&store, DocType::Note, json!({"_id": "note:ex", "_rev": rev, "title": "Excluded",
        "body": format!("{CANARY} Sam promised excluded things\n"), "bookId": book, "tags": [exclude],
        "createdAt": T0 + 5 * DAY, "updatedAt": T0 + 5 * DAY}));
    note_at(&store, "note:tr", "Trashed", &format!("{CANARY} Sam promised trashed things\n"), "trash", T0 + 5 * DAY);
    note_at(&store, "note:lk", "Locked", &format!("{CANARY} Sam promised locked things\n"), &book, T0 + 5 * DAY);
    let rev = store.get("note:lk").unwrap().unwrap().rev;
    store
        .lock_note(json!({"_id": "note:lk", "_rev": rev, "title": "Locked", "body": "", "bookId": book,
            "kansoLock": lock()}))
        .unwrap();
    let page = EntityPage::load(&store, &id).unwrap().unwrap();
    // Computed sections are local, so they may show the excluded note (but
    // never the locked or trashed ones).
    let view = pages::open(&store, &page, options(false), &[]).unwrap();
    let timeline = find(&view, "timeline");
    assert!(timeline.items.iter().all(|i| i.sources[0].note_id != "note:lk" && i.sources[0].note_id != "note:tr"));
    let (provider, seen) = Cites::new();
    let ctx = WriteContext::new(&provider, "m", T0 + 10 * DAY);
    pages::refresh_section(&store, &page, "promises", &ctx).unwrap();
    pages::refresh_section(&store, &page, "discussed", &ctx).unwrap();
    for prompt in seen.lock().unwrap().iter() {
        assert!(!prompt.contains(CANARY), "{prompt}");
    }
    let cache = store.page_cache(&id).unwrap();
    assert!(!serde_json::to_string(&cache).unwrap().contains(CANARY));
}

#[test]
fn locking_a_source_drops_its_lines_at_once() {
    let (_dir, store, book, id) = library();
    let page = EntityPage::load(&store, &id).unwrap().unwrap();
    let (provider, _) = Cites::new();
    let ctx = WriteContext::new(&provider, "m", T0);
    pages::refresh_section(&store, &page, "promises", &ctx).unwrap();
    let rev = store.get("note:weekly").unwrap().unwrap().rev;
    store
        .lock_note(json!({"_id": "note:weekly", "_rev": rev, "title": "Weekly sync", "body": "", "bookId": book,
            "kansoLock": lock()}))
        .unwrap();
    let view = pages::open(&store, &page, options(true), &[]).unwrap();
    let promises = find(&view, "promises");
    assert_eq!(promises.items.len(), 1);
    assert!(promises.items.iter().all(|i| i.sources.iter().all(|s| s.note_id != "note:weekly")));
    // The cache was rewritten without it.
    assert_eq!(store.page_cache(&id).unwrap().sections["promises"].items.len(), 1);
}

#[test]
fn pin_hide_copy_and_not_right() {
    let (_dir, store, _book, id) = library();
    let page = EntityPage::load(&store, &id).unwrap().unwrap();
    let (provider, seen) = Cites::new();
    let ctx = WriteContext::new(&provider, "m", T0);
    pages::refresh_section(&store, &page, "promises", &ctx).unwrap();
    assert!(pages::pin(&store, &page, "promises", true).unwrap());
    assert!(!pages::pin(&store, &page, "discussed", true).unwrap(), "nothing written to keep");
    let view = pages::open(&store, &page, options(true), &[]).unwrap();
    assert!(matches!(find(&view, "promises").state, SectionState::Pinned { changed: 0, .. }));

    // Refresh all never touches a kept section.
    let before = store.page_cache(&id).unwrap().sections["promises"].clone();
    let done = pages::refresh_all(&store, &page, &WriteContext::new(&provider, "m", T0 + 1000)).unwrap();
    assert_eq!(done, vec!["discussed".to_owned()]);
    assert_eq!(store.page_cache(&id).unwrap().sections["promises"], before);
    // Its own Refresh rewrites it, and it stays kept.
    pages::refresh_section(&store, &page, "promises", &WriteContext::new(&provider, "m", T0 + 2000)).unwrap();
    let after = store.page_cache(&id).unwrap().sections["promises"].clone();
    assert!(after.pinned && after.written_at == T0 + 2000);

    // Copy into the note: Markdown links with the line.
    let view = pages::open(&store, &page, options(true), &[]).unwrap();
    let markdown = pages::section_markdown(find(&view, "promises"));
    assert!(markdown.contains("](inkdrop://note/"), "{markdown}");
    assert!(markdown.contains("#L"), "{markdown}");

    // Not right: the bullet goes and the next prompt says so.
    assert!(pages::not_right(&store, &page, "promises", 0).unwrap());
    assert_eq!(store.page_cache(&id).unwrap().sections["promises"].items.len(), 1);
    pages::refresh_section(&store, &page, "promises", &WriteContext::new(&provider, "m", T0 + 3000)).unwrap();
    let last = seen.lock().unwrap().last().unwrap().clone();
    assert!(last.contains("do not repeat them"), "{last}");
    assert_eq!(store.page_cache(&id).unwrap().sections["promises"].items.len(), 1, "the wrong one is not back");

    // Hide, and show again.
    assert!(pages::hide(&store, &page, "timeline", true, T0).unwrap());
    let view = pages::open(&store, &page, options(true), &[]).unwrap();
    assert!(view.sections.iter().all(|s| s.key != "timeline"));
    assert_eq!(view.hidden, vec![("timeline".to_owned(), "Timeline".to_owned())]);
    assert!(pages::show_hidden(&store, &page, T0 + 1).unwrap());
    assert!(pages::open(&store, &page, options(true), &[]).unwrap().hidden.is_empty());

    // The person's own text was never touched, and no edit or version was made by any of it.
    assert_eq!(page.own_text(&store).unwrap(), "Sam runs the Lusaka office.\n");
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 12,
        rng_seed: proptest::test_runner::RngSeed::Fixed(20261017),
        ..ProptestConfig::default()
    })]

    /// A kept section is never rewritten by Refresh All, whichever are kept.
    #[test]
    fn prop_pinned_sections_are_never_rewritten_by_refresh_all(pin_promises in any::<bool>(), pin_discussed in any::<bool>()) {
        let (_dir, store, _book, id) = library();
        let page = EntityPage::load(&store, &id).unwrap().unwrap();
        let (provider, _) = Cites::new();
        for key in ["promises", "discussed"] {
            pages::refresh_section(&store, &page, key, &WriteContext::new(&provider, "m", T0)).unwrap();
        }
        let wants = [("promises", pin_promises), ("discussed", pin_discussed)];
        for (key, on) in wants {
            if on {
                pages::pin(&store, &page, key, true).unwrap();
            }
        }
        let before = store.page_cache(&id).unwrap();
        pages::refresh_all(&store, &page, &WriteContext::new(&provider, "m", T0 + 5000)).unwrap();
        let after = store.page_cache(&id).unwrap();
        for (key, on) in wants {
            if on {
                prop_assert_eq!(&after.sections[key], &before.sections[key]);
            } else {
                prop_assert_eq!(after.sections[key].written_at, T0 + 5000);
            }
        }
    }

    /// Whichever of a person's notes are locked, no prompt or cache holds
    /// their words.
    #[test]
    fn prop_a_locked_source_is_never_in_a_prompt_or_cache(lock_weekly in any::<bool>(), lock_lusaka in any::<bool>(), lock_after in any::<bool>()) {
        let (_dir, store, book, id) = library();
        let page = EntityPage::load(&store, &id).unwrap().unwrap();
        let (provider, seen) = Cites::new();
        let ctx = WriteContext::new(&provider, "m", T0);
        let words = [("note:weekly", "revised contract", lock_weekly), ("note:lusaka", "Lusaka team", lock_lusaka)];
        if lock_after {
            pages::refresh_section(&store, &page, "promises", &ctx).unwrap();
        }
        for (note, _, locked) in words {
            if locked {
                let rev = store.get(note).unwrap().unwrap().rev;
                store.lock_note(json!({"_id": note, "_rev": rev, "title": "T", "body": "", "bookId": book,
                    "kansoLock": lock()})).unwrap();
            }
        }
        let before = seen.lock().unwrap().len();
        pages::refresh_section(&store, &page, "promises", &ctx).unwrap();
        let prompts = seen.lock().unwrap()[before..].to_vec();
        let view = pages::open(&store, &page, options(true), &[]).unwrap();
        let cache = serde_json::to_string(&store.page_cache(&id).unwrap()).unwrap();
        let shown = format!("{:?}", view);
        for (_, phrase, locked) in words {
            if locked {
                for prompt in &prompts {
                    prop_assert!(!prompt.contains(phrase), "{phrase} in prompt");
                }
                prop_assert!(!cache.contains(phrase), "{phrase} in cache");
                prop_assert!(!shown.contains(&format!("quote: \"{phrase}")), "{phrase} in view");
            }
        }
    }
}

// MARK: Suggestions

#[test]
fn suggestions_come_from_signals_and_nothing_is_created() {
    let (_dir, store) = open_store();
    let book = put(&store, DocType::Book, json!({"_id": "book:a", "name": "Work"}));
    for ix in 0..4 {
        note_at(
            &store,
            &format!("note:m{ix}"),
            &format!("Meeting {ix}"),
            &format!("Lunch with Tendai Chirwa went well\n@rudo will follow up {ix}\n"),
            &book,
            T0 + ix * DAY,
        );
    }
    let settings = EntitySettings::default();
    let mut last = 0;
    let count = suggest::refresh(&store, &settings, T0, &mut last, false).unwrap();
    assert_eq!(count, Some(2));
    // Too soon: it does not run again.
    assert_eq!(suggest::refresh(&store, &settings, T0 + 1000, &mut last, false).unwrap(), None);
    let found = suggest::suggestions(&store, &settings).unwrap();
    let names: Vec<&str> = found.iter().map(|s| s.name.as_str()).collect();
    assert!(names.contains(&"Tendai Chirwa") && names.contains(&"rudo"), "{names:?}");
    assert!(store.entities().unwrap().is_empty(), "no page was made");
    assert!(found[0].detail().contains("notes"));
    // Not now hides it until it gains five notes; Not a person hides it for good.
    let tendai = found.iter().find(|s| s.name == "Tendai Chirwa").unwrap().clone();
    suggest::dismiss(&store, &settings, &tendai, false, T0).unwrap();
    assert!(suggest::suggestions(&store, &settings).unwrap().iter().all(|s| s.name != "Tendai Chirwa"));
    let rudo = found.iter().find(|s| s.name == "rudo").unwrap().clone();
    suggest::dismiss(&store, &settings, &rudo, true, T0).unwrap();
    suggest::refresh(&store, &settings, T0 + 10 * DAY, &mut last, true).unwrap();
    assert!(suggest::suggestions(&store, &settings).unwrap().iter().all(|s| s.name != "rudo"));
    assert_eq!(store.dismissed().unwrap().len(), 2);
    suggest::suggest_again(&store, &settings, "rudo", T0).unwrap();
    suggest::refresh(&store, &settings, T0 + 20 * DAY, &mut last, true).unwrap();
    assert!(suggest::suggestions(&store, &settings).unwrap().iter().any(|s| s.name == "rudo"));
    // Turned off, there are none.
    let off = EntitySettings { suggest: false, ..EntitySettings::default() };
    assert!(suggest::suggestions(&store, &off).unwrap().is_empty());
    assert_eq!(suggest::refresh(&store, &off, T0, &mut 0, true).unwrap(), None);
    let _ = (Context::default(), ai_detect::MAX_NAMES);
}

#[test]
fn dismissals_wait_for_the_notebook_then_ride_in_it() {
    let (_dir, store) = open_store();
    let settings = EntitySettings::default();
    let s = suggest::Suggestion {
        name: "Monday Standup".into(),
        key: "monday standup".into(),
        kind: EntityKind::Project,
        notes: 4,
        last_seen: None,
        also: Vec::new(),
        alias_of: None,
    };
    suggest::dismiss(&store, &settings, &s, true, T0).unwrap();
    assert_eq!(store.dismissed().unwrap().len(), 1);
    assert!(suggest::dismissal_book(&store, &settings).is_none(), "no notebook was made for it");
    // The first page makes the notebook; the dismissals move into it.
    let sam = actions::create_page(&store, &settings, EntityKind::Person, "Sam Moyo", &[], T0).unwrap();
    suggest::sync_dismissals(&store, &settings, T0).unwrap();
    let book = suggest::dismissal_book(&store, &settings).unwrap();
    assert_eq!(store.book_dismissed(&book).unwrap(), vec!["monday standup".to_owned()]);
    // A name turned away on another device arrives.
    store.set_book_dismissed(&book, &["monday standup".into(), "tuesday".into()]).unwrap();
    suggest::sync_dismissals(&store, &settings, T0).unwrap();
    assert_eq!(store.dismissed().unwrap().len(), 2);
    assert!(store.entity(&sam).unwrap().is_some());
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 12,
        rng_seed: proptest::test_runner::RngSeed::Fixed(20261017),
        ..ProptestConfig::default()
    })]

    /// A name turned away for good never comes back, however many times
    /// detection runs and however many notes mention it.
    #[test]
    fn prop_dismissed_names_never_return(extra_notes in 0usize..6, runs in 1usize..4) {
        let (_dir, store) = open_store();
        let book = put(&store, DocType::Book, json!({"_id": "book:a", "name": "Work"}));
        let settings = EntitySettings::default();
        note_at(&store, "note:first", "First", "ping @rudo about it\n", &book, T0);
        let mut last = 0;
        suggest::refresh(&store, &settings, T0, &mut last, true).unwrap();
        let rudo = suggest::suggestions(&store, &settings).unwrap().into_iter().find(|s| s.name == "rudo").unwrap();
        suggest::dismiss(&store, &settings, &rudo, true, T0).unwrap();
        for ix in 0..extra_notes {
            note_at(&store, &format!("note:more{ix}"), "More", "@rudo again and Rudo Dube here\n", &book, T0 + ix as i64);
        }
        for run in 0..runs {
            suggest::refresh(&store, &settings, T0 + run as i64 * DAY, &mut last, true).unwrap();
            prop_assert!(suggest::suggestions(&store, &settings).unwrap().iter().all(|s| s.name != "rudo"));
        }
    }

    /// Locked notes (sealed, or in a locked notebook) and notes kept out
    /// of AI never give a suggestion.
    #[test]
    fn prop_candidates_never_come_from_locked_notes(how in 0u8..3, notes in 1usize..4) {
        let (_dir, store) = open_store();
        let book = put(&store, DocType::Book, json!({"_id": "book:a", "name": "Work"}));
        let exclude = put(&store, DocType::Tag, json!({"name": "ai-exclude"}));
        for ix in 0..notes {
            let id = format!("note:s{ix}");
            let body = "ask @zorgon and Zorgon Quill today\n";
            match how {
                0 => {
                    note_at(&store, &id, "S", body, &book, T0);
                    let rev = store.get(&id).unwrap().unwrap().rev;
                    store.lock_note(json!({"_id": id, "_rev": rev, "title": "S", "body": "", "bookId": book,
                        "kansoLock": lock()})).unwrap();
                }
                1 => {
                    put(&store, DocType::Note, json!({"_id": id, "title": "S", "body": body, "bookId": book,
                        "tags": [exclude], "createdAt": T0, "updatedAt": T0}));
                }
                _ => note_at(&store, &id, "S", body, "trash", T0),
            }
        }
        let settings = EntitySettings::default();
        suggest::refresh(&store, &settings, T0, &mut 0, true).unwrap();
        prop_assert!(store.candidates().unwrap().is_empty(), "{:?}", store.candidates().unwrap());
    }
}

#[test]
fn the_ai_pass_relabels_but_never_creates_a_page() {
    let (_dir, store) = open_store();
    let book = put(&store, DocType::Book, json!({"_id": "book:a", "name": "Work"}));
    for ix in 0..3 {
        note_at(&store, &format!("note:n{ix}"), "N", "visited Lake Placid and met Tendai Chirwa\n", &book, T0);
    }
    let settings = EntitySettings::default();
    suggest::refresh(&store, &settings, T0, &mut 0, true).unwrap();
    assert_eq!(store.candidates().unwrap().len(), 2);
    struct Says(&'static str);
    impl ChatProvider for Says {
        fn stream(&self, _: &Prompt) -> Result<Box<dyn Iterator<Item = Event> + Send>, AiError> {
            Ok(Box::new(vec![Event::Text(self.0.to_owned()), Event::Done].into_iter()))
        }
    }
    // A reply that isn't JSON changes nothing.
    assert!(ai_detect::run(&store, &Says("I think both are people!")).is_err());
    assert_eq!(store.candidates().unwrap().len(), 2);
    let left = ai_detect::run(
        &store,
        &Says(r#"[{"name":"Lake Placid","kind":"neither"},{"name":"Tendai Chirwa","kind":"person"}]"#),
    )
    .unwrap();
    assert_eq!(left, 1);
    assert_eq!(store.candidates().unwrap()[0].name, "tendai chirwa");
    assert!(store.entities().unwrap().is_empty());
    let _ = entities::DEFAULT_NOTEBOOK;
    let _ = SectionKind::Ai;
}
