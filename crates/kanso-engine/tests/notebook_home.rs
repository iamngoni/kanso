//! Notebook homepages end to end (issue #201): the computed sections over a
//! nested notebook, the Home note, AI sections written with receipts and
//! checked, what goes stale, the map, the layout, and that nothing locked
//! or excluded reaches a prompt or a cache. CI pins the seed with
//! `PROPTEST_RNG_SEED`.

use std::sync::{Arc, Mutex};

use kanso_ai::{AiError, ChatProvider, Event, Prompt};
use kanso_core::DocType;
use kanso_engine::pages::notebook::{HomeOptions, NotebookPage, open_home};
use kanso_engine::pages::notebook_actions as actions;
use kanso_engine::pages::{self, PageSource, SectionState, write::WriteContext};
use kanso_store::{Origin, Put, Store};
use proptest::prelude::*;
use serde_json::{Value, json};

const CANARY: &str = "okapicanary9913";
const DAY: i64 = 86_400_000;
const T0: i64 = 1_761_600_000_000;
const NOW: i64 = T0 + 9 * DAY + 3_600_000;

/// The sections of a homepage in order. The map is drawn only by builds with
/// the `discovery` graph (the app and the phone have it, a bare engine
/// does not), so it is listed only there.
fn sections(keys: &[&'static str]) -> Vec<&'static str> {
    let mut out = Vec::new();
    for &k in keys {
        if k != "map" || cfg!(feature = "discovery") {
            out.push(k);
        }
    }
    out
}

/// Answers every prompt by citing each excerpt line that contains the
/// needle, and remembers the prompts.
struct Cites {
    needle: &'static str,
    seen: Arc<Mutex<Vec<String>>>,
    extra: String,
}

impl Cites {
    fn new(needle: &'static str) -> (Cites, Arc<Mutex<Vec<String>>>) {
        let seen = Arc::new(Mutex::new(Vec::new()));
        (
            Cites {
                needle,
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
                && text.contains(self.needle)
            {
                reply.push_str(&format!("- {text} [{number}:L{n}]\n"));
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

fn book(store: &Store, id: &str, name: &str, parent: Option<&str>) -> String {
    put(
        store,
        DocType::Book,
        json!({"_id": id, "name": name, "parentBookId": parent}),
    )
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

fn seal(store: &Store, id: &str) {
    let mut note = store.get_api(id, None, false).unwrap();
    note["body"] = "".into();
    note["kansoLock"] = lock();
    store.lock_note(note).unwrap();
}

fn lock_book_by_sync(store: &Store, id: &str) {
    let mut book = store.get_api(id, None, false).unwrap();
    book.as_object_mut().unwrap().remove("count");
    book["kansoLock"] = lock();
    store
        .put(DocType::Book, book, Put::Upsert, Origin::Sync)
        .unwrap();
}

fn on(ai_on: bool) -> HomeOptions {
    HomeOptions { ai_on, now: NOW }
}

fn page(store: &Store, id: &str) -> NotebookPage {
    NotebookPage::load(store, id, NOW).unwrap().unwrap()
}

/// Research with Drafts and Papers inside it.
fn library() -> (tempfile::TempDir, Store) {
    let (dir, store) = open_store();
    book(&store, "book:r", "Research", None);
    book(&store, "book:d", "Drafts", Some("book:r"));
    book(&store, "book:p", "Papers", Some("book:r"));
    note_at(
        &store,
        "note:plan",
        "Chapter plan",
        "# Plan\nChapter structure for the thesis\nWe decided to use the 2019 dataset, not 2021\n",
        "book:r",
        T0,
    );
    note_at(
        &store,
        "note:ch3",
        "Chapter 3 draft",
        "Draft text, see [[Chapter plan]]\n- [ ] Email Dr Banda @2026-11-04\n- [ ] Re-run the regression\n",
        "book:d",
        T0 + 8 * DAY,
    );
    note_at(
        &store,
        "note:lal",
        "Lal 2004",
        "Reading notes on soil carbon\nLinks to [[Chapter plan]]\n- [x] Skim the appendix\n",
        "book:p",
        T0 + 9 * DAY,
    );
    note_at(
        &store,
        "note:top",
        "Reading list",
        "- [ ] Fix figure 3\nAgreed with the supervisor to drop the Zambia comparison\n",
        "book:r",
        T0 + 9 * DAY + 1000,
    );
    (dir, store)
}

fn find<'a>(view: &'a pages::PageView, key: &str) -> &'a pages::Section {
    view.sections
        .iter()
        .find(|s| s.key == key)
        .unwrap_or_else(|| {
            panic!(
                "no {key} in {:?}",
                view.sections.iter().map(|s| &s.key).collect::<Vec<_>>()
            )
        })
}

#[test]
fn a_nested_notebook_gets_its_computed_sections() {
    let (_dir, store) = library();
    let p = page(&store, "book:r");
    let view = open_home(&store, &p, on(false), &[]).unwrap();
    assert_eq!(view.header.notes, 4);
    assert_eq!(view.header.notebooks_inside, 2);
    assert_eq!(
        view.header.line,
        "4 notes \u{b7} 2 notebooks inside \u{b7} last edited today"
    );
    let keys: Vec<&str> = view.page.sections.iter().map(|s| s.key.as_str()).collect();
    assert_eq!(
        keys,
        sections(&["key_notes", "recent", "open_tasks", "map", "inside"])
    );
    // AI is off: the AI sections are left out and one line says why.
    assert_eq!(
        view.footer.as_deref(),
        Some(pages::notebook_model::AI_OFF_LINE)
    );

    // Key notes: nothing pinned, so the one both notes link to, first.
    let key = find(&view.page, "key_notes");
    assert_eq!(key.items[0].text, "Chapter plan");
    assert!(
        key.items[0]
            .meta
            .as_deref()
            .unwrap()
            .starts_with("Linked from 2 notes")
    );

    // Recent activity: newest first, sub-notebook names in front, grouped by day.
    let recent = find(&view.page, "recent");
    assert_eq!(recent.items[0].text, "Reading list");
    assert!(recent.items.iter().any(|i| i.text == "Papers/Lal 2004"));
    assert!(recent.items.iter().all(|i| i.group.is_some()));
    assert_eq!(recent.items[0].group.as_deref(), Some("Today"));

    // Open tasks: the ones in the notebooks inside it too, dated first.
    let tasks = find(&view.page, "open_tasks");
    assert_eq!(tasks.total, 3);
    assert!(tasks.items[0].text.starts_with("Email Dr Banda"));
    assert!(tasks.items.iter().all(|i| i.task.is_some()));

    // Inside: counts, last edit and open tasks.
    let inside = find(&view.page, "inside");
    assert_eq!(inside.items.len(), 2);
    let drafts = inside.items.iter().find(|i| i.text == "Drafts").unwrap();
    assert!(
        drafts
            .meta
            .as_deref()
            .unwrap()
            .starts_with("1 note \u{b7} edited ")
    );
    assert!(drafts.meta.as_deref().unwrap().contains("2 open"));
}

#[test]
fn a_notebook_with_all_tasks_done_says_so_and_empty_sections_are_left_out() {
    let (_dir, store) = open_store();
    book(&store, "book:a", "Archive", None);
    note_at(&store, "note:a", "A", "- [x] done\n", "book:a", T0);
    let view = open_home(&store, &page(&store, "book:a"), on(false), &[]).unwrap();
    let tasks = find(&view.page, "open_tasks");
    assert!(tasks.items.is_empty());
    assert_eq!(tasks.note.as_deref(), Some("No open tasks"));
    assert!(
        view.page
            .sections
            .iter()
            .all(|s| s.key != "inside" && s.key != "people")
    );
    // A notebook with no tasks at all has no such section.
    let (_d2, empty) = open_store();
    book(&empty, "book:e", "Empty", None);
    let view = open_home(&empty, &page(&empty, "book:e"), on(false), &[]).unwrap();
    assert!(view.page.sections.is_empty());
    assert_eq!(view.header.line, "0 notes");
}

#[test]
fn people_and_projects_are_listed_by_number_of_notes() {
    let (_dir, store) = library();
    note_at(
        &store,
        "note:banda",
        "Dr Banda",
        "---\nkanso: person\n---\nSupervisor\n",
        "book:p",
        T0,
    );
    let view = open_home(&store, &page(&store, "book:r"), on(false), &[]).unwrap();
    let people = find(&view.page, "people");
    assert_eq!(people.items[0].text, "Dr Banda");
    assert_eq!(people.items[0].tag.as_deref(), Some("person"));
    assert_eq!(people.items[0].meta.as_deref(), Some("1 note"));
}

#[test]
fn locked_notes_and_notebooks_contribute_nothing_not_even_counts() {
    let (_dir, store) = library();
    book(&store, "book:vault", "Vault", Some("book:r"));
    note_at(
        &store,
        "note:s1",
        "Secret one",
        &format!("{CANARY}\n- [ ] hidden task\n"),
        "book:vault",
        T0 + 9 * DAY,
    );
    note_at(
        &store,
        "note:s2",
        "Secret two",
        &format!("{CANARY} [[Chapter plan]]\n- [ ] sealed task\n"),
        "book:r",
        T0 + 9 * DAY,
    );
    seal(&store, "note:s2");
    lock_book_by_sync(&store, "book:vault");
    let view = open_home(&store, &page(&store, "book:r"), on(false), &[]).unwrap();
    assert_eq!(view.header.notes, 4, "neither locked note is counted");
    assert_eq!(
        view.header.notebooks_inside, 2,
        "the locked notebook is not counted"
    );
    let whole = format!("{view:?}");
    assert!(!whole.contains("Secret"));
    assert!(!whole.contains("hidden task"));
    assert!(!whole.contains("sealed task"));
    // The locked notebook is listed by name alone, with a lock.
    let inside = find(&view.page, "inside");
    let vault = inside.items.iter().find(|i| i.text == "Vault").unwrap();
    assert_eq!(vault.tag.as_deref(), Some("locked"));
    assert!(vault.meta.is_none());
    // A locked notebook has no homepage on disk.
    assert!(
        NotebookPage::load(&store, "book:vault", NOW)
            .unwrap()
            .is_none()
    );
    let map = page(&store, "book:r").map(&store).unwrap();
    assert!(map.nodes.iter().all(|n| !n.title.contains("Secret")));
}

#[test]
fn about_and_decisions_are_written_with_receipts_cached_and_go_stale_only_for_notes_they_read() {
    let (_dir, store) = library();
    let p = page(&store, "book:r");
    let view = open_home(&store, &p, on(true), &[]).unwrap();
    assert_eq!(find(&view.page, "about").state, SectionState::Unwritten);
    assert_eq!(find(&view.page, "decisions").state, SectionState::Unwritten);

    let (provider, seen) = Cites::new("soil carbon");
    let ctx = WriteContext::new(&provider, "test/model", NOW);
    let about = pages::refresh_section(&store, &p, "about", &ctx).unwrap();
    assert_eq!(about.items.len(), 1, "{about:?}");
    assert!(!about.items[0].sources.is_empty());
    assert!(
        seen.lock().unwrap()[0].contains("Research"),
        "the instruction names the notebook"
    );

    let (provider, _) = Cites::new("decided");
    let ctx = WriteContext::new(&provider, "test/model", NOW);
    let decisions = pages::refresh_section(&store, &p, "decisions", &ctx).unwrap();
    assert_eq!(decisions.items.len(), 1);
    assert!(decisions.items[0].text.contains("2019 dataset"));

    let view = open_home(&store, &p, on(true), &[]).unwrap();
    assert_eq!(find(&view.page, "about").state, SectionState::Fresh);
    assert_eq!(find(&view.page, "decisions").state, SectionState::Fresh);
    let receipt = &find(&view.page, "decisions").items[0].sources[0];
    let body = store.note(&receipt.note_id).unwrap().unwrap().body;
    assert!(kanso_engine::ask::relocate(&body, receipt).is_some());

    // A note that fed it changes: out of date, counted.
    note_at(
        &store,
        "note:plan",
        "Chapter plan",
        "# Plan\nChapter structure for the thesis\nWe decided to use the 2019 dataset, not 2021\nAnd on soil carbon too\n",
        "book:r",
        T0 + 11 * DAY,
    );
    let view = open_home(&store, &p, on(true), &[]).unwrap();
    assert!(matches!(
        find(&view.page, "decisions").state,
        SectionState::Stale { .. }
    ));

    // Kept: pinned stays through a change.
    pages::pin(&store, &p, "decisions", true, NOW).unwrap();
    let view = open_home(&store, &p, on(true), &[]).unwrap();
    assert!(matches!(
        find(&view.page, "decisions").state,
        SectionState::Pinned { .. }
    ));
}

#[test]
fn a_change_to_a_note_the_section_never_read_leaves_it_fresh() {
    let (_dir, store) = open_store();
    book(&store, "book:a", "Big", None);
    for i in 0..40 {
        let id = format!("note:n{i:02}");
        put(
            &store,
            DocType::Note,
            json!({"_id": id, "title": format!("Note {i}"), "body": format!("line about topic {i}\n"),
                   "bookId": "book:a", "pinned": (30..36).contains(&i),
                   "createdAt": T0 + i as i64 * 1000, "updatedAt": T0 + i as i64 * 1000}),
        );
    }
    let p = page(&store, "book:a");
    let (provider, _) = Cites::new("topic");
    let ctx = WriteContext::new(&provider, "m", NOW);
    pages::refresh_section(&store, &p, "about", &ctx).unwrap();
    // The oldest note is beyond the ten most recent: not read by About.
    note_at(
        &store,
        "note:n00",
        "Note 0",
        "rewritten entirely\n",
        "book:a",
        T0 + 99 * DAY,
    );
    let view = open_home(&store, &p, on(true), &[]).unwrap();
    // It is now the newest, so it is read.
    assert!(matches!(
        find(&view.page, "about").state,
        SectionState::Stale { .. }
    ));
    // But an edit to one in the middle that stays out of the ten recent is not.
    pages::refresh_section(&store, &p, "about", &ctx).unwrap();
    note_at(
        &store,
        "note:n05",
        "Note 5",
        "edited but old\n",
        "book:a",
        T0 + 5000,
    );
    let view = open_home(&store, &p, on(true), &[]).unwrap();
    assert_eq!(find(&view.page, "about").state, SectionState::Fresh);
}

#[test]
fn fabricated_replies_leave_nothing() {
    let (_dir, store) = library();
    let p = page(&store, "book:r");
    let (mut provider, _) = Cites::new("decided");
    provider.extra =
        "\n- Decided on a pony [1:L2]\n- Decided on the moon [9:L9]\n- uncited decision\n".into();
    let ctx = WriteContext::new(&provider, "m", NOW);
    let written = pages::refresh_section(&store, &p, "decisions", &ctx).unwrap();
    assert_eq!(
        written.items.len(),
        1,
        "only the real line survives: {written:?}"
    );
}

#[test]
fn decisions_reads_the_cue_lines_of_an_old_note_among_many() {
    let (_dir, store) = open_store();
    book(&store, "book:a", "Log", None);
    note_at(
        &store,
        "note:old",
        "Old",
        "intro\nWe settled on Postgres for the store\n",
        "book:a",
        T0,
    );
    for i in 0..30 {
        note_at(
            &store,
            &format!("note:n{i:02}"),
            &format!("Note {i}"),
            &format!("{}\n", "filler text ".repeat(80)),
            "book:a",
            T0 + (i as i64 + 1) * 1000,
        );
    }
    let p = page(&store, "book:a");
    let passages = p.passages_for(&store, "decisions", 1500).unwrap();
    assert!(
        passages.excerpts.iter().any(|e| e.id == "note:old"),
        "the cue line is read first even though the note is the oldest"
    );
}

#[test]
fn locked_ai_excluded_and_trashed_notes_never_reach_a_prompt_or_the_cache() {
    let (_dir, store) = library();
    let exclude = put(&store, DocType::Tag, json!({"name": "ai-exclude"}));
    note_at(
        &store,
        "note:private",
        "Private",
        &format!("{CANARY} decided against it\n"),
        "book:r",
        T0 + 9 * DAY,
    );
    let rev = store.get("note:private").unwrap().unwrap().rev;
    put(
        &store,
        DocType::Note,
        json!({"_id": "note:private", "_rev": rev, "title": "Private", "body": format!("{CANARY} decided against it\n"),
               "bookId": "book:r", "tags": [exclude], "createdAt": T0, "updatedAt": T0 + 9 * DAY}),
    );
    note_at(
        &store,
        "note:sealed",
        "Sealed",
        &format!("{CANARY} decided\n"),
        "book:r",
        T0 + 9 * DAY,
    );
    seal(&store, "note:sealed");
    let p = page(&store, "book:r");
    let (provider, seen) = Cites::new("decided");
    let ctx = WriteContext::new(&provider, "m", NOW);
    pages::refresh_section(&store, &p, "decisions", &ctx).unwrap();
    pages::refresh_section(&store, &p, "about", &ctx).unwrap();
    assert!(seen.lock().unwrap().iter().all(|p| !p.contains(CANARY)));
    let cache = format!("{:?}", store.page_cache("book:r").unwrap());
    assert!(!cache.contains(CANARY));
    // The excluded note is local: it still shows in computed sections.
    let view = open_home(&store, &p, on(true), &[]).unwrap();
    assert!(
        find(&view.page, "recent")
            .items
            .iter()
            .any(|i| i.text == "Private")
    );
}

#[test]
fn locking_a_notebook_removes_its_page_cache_and_its_homepage() {
    let (_dir, store) = library();
    let p = page(&store, "book:r");
    let (provider, _) = Cites::new("decided");
    let ctx = WriteContext::new(&provider, "m", NOW);
    pages::refresh_section(&store, &p, "decisions", &ctx).unwrap();
    assert!(!store.page_cache("book:r").unwrap().is_empty());
    let mut sealed_book = store.get_api("book:r", None, false).unwrap();
    sealed_book.as_object_mut().unwrap().remove("count");
    sealed_book["kansoLock"] = lock();
    let mut sealed = Vec::new();
    for id in store
        .home_all_notes("book:r")
        .unwrap()
        .into_iter()
        .map(|n| n.id)
    {
        let mut n = store.get_api(&id, None, false).unwrap();
        n["body"] = "".into();
        n["kansoLock"] = lock();
        sealed.push(n);
    }
    store.lock_book(sealed_book, sealed).unwrap();
    assert!(
        !store
            .get("book:r")
            .unwrap()
            .unwrap()
            .fields
            .contains_key("kansoPage")
    );
    assert!(store.page_cache("book:r").unwrap().is_empty());
    assert!(NotebookPage::load(&store, "book:r", NOW).unwrap().is_none());
    let view = open_home(
        &store,
        &NotebookPage::new("book:r", "Research", NOW),
        on(true),
        &[],
    )
    .unwrap();
    assert!(view.locked);
    assert!(view.page.sections.is_empty());
}

#[test]
fn the_home_note_is_chosen_made_and_moved_by_the_rule() {
    let (_dir, store) = library();
    let p = page(&store, "book:r");
    assert!(
        !open_home(&store, &p, on(false), &[])
            .unwrap()
            .header
            .has_home_note
    );
    actions::choose_home(&store, "book:r", "note:plan", NOW).unwrap();
    let view = open_home(&store, &p, on(false), &[]).unwrap();
    assert!(view.header.has_home_note);
    let home = find(&view.page, "home_note");
    assert_eq!(home.items[0].target.as_deref(), Some("note:plan"));
    assert!(
        !home.items[0].text.contains("kanso: home"),
        "front matter is not shown"
    );
    // Choosing another takes the mark off the first.
    actions::choose_home(&store, "book:r", "note:top", NOW + 5).unwrap();
    let marked: Vec<String> = store
        .home_notes_marked("book:r")
        .unwrap()
        .into_iter()
        .map(|m| m.id)
        .collect();
    assert_eq!(marked, ["note:top"]);
    // A note of another notebook can't be chosen.
    assert!(actions::choose_home(&store, "book:r", "note:lal", NOW).is_err());
    // Making one: "About Research", and the old one is demoted.
    let made = actions::new_home(&store, "book:r", "Research", NOW + 10).unwrap();
    let doc = store.get(&made).unwrap().unwrap();
    assert_eq!(doc.fields["title"], "About Research");
    let marked: Vec<String> = store
        .home_notes_marked("book:r")
        .unwrap()
        .into_iter()
        .map(|m| m.id)
        .collect();
    assert_eq!(marked, std::slice::from_ref(&made));
    // Moving it into a notebook that has a Home note removes its mark.
    actions::choose_home(&store, "book:d", "note:ch3", NOW + 20).unwrap();
    store
        .put(
            DocType::Note,
            json!({"_id": made, "bookId": "book:d"}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    assert!(actions::after_move(&store, &made, NOW + 30).unwrap());
    assert!(!kanso_core::home::is_home(
        store.get(&made).unwrap().unwrap().fields["body"]
            .as_str()
            .unwrap()
    ));
    // A person page can't be a Home note.
    note_at(
        &store,
        "note:sam",
        "Sam",
        "---\nkanso: person\n---\nSam\n",
        "book:r",
        T0,
    );
    assert!(actions::choose_home(&store, "book:r", "note:sam", NOW).is_err());
}

#[test]
fn copy_into_home_note_creates_it_when_missing_and_appends_with_links() {
    let (_dir, store) = library();
    let p = page(&store, "book:r");
    let (provider, _) = Cites::new("decided");
    let ctx = WriteContext::new(&provider, "m", NOW);
    pages::refresh_section(&store, &p, "decisions", &ctx).unwrap();
    let view = open_home(&store, &p, on(true), &[]).unwrap();
    let md = pages::section_markdown(find(&view.page, "decisions"));
    let id = actions::copy_into_home(&store, "book:r", "Research", "Decisions", &md, NOW)
        .unwrap()
        .unwrap();
    let body = store.get(&id).unwrap().unwrap().fields["body"]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(kanso_core::home::is_home(&body));
    assert!(body.contains("## Decisions"));
    assert!(body.contains("inkdrop://note/plan#"), "{body}");
    // Copying again appends to the same note.
    let again = actions::copy_into_home(&store, "book:r", "Research", "Decisions", &md, NOW + 5)
        .unwrap()
        .unwrap();
    assert_eq!(id, again);
}

#[test]
#[cfg(feature = "discovery")]
fn the_same_library_draws_the_same_map_twice_and_caps_it() {
    let (_dir, store) = open_store();
    book(&store, "book:a", "Wiki", None);
    for i in 0..60 {
        note_at(
            &store,
            &format!("note:n{i:02}"),
            &format!("Note {i:02}"),
            &format!(
                "links [[Note {:02}]] and [[Note {:02}]]\n",
                (i + 1) % 60,
                (i + 7) % 60
            ),
            "book:a",
            T0 + i as i64,
        );
    }
    let p = page(&store, "book:a");
    let one = p.map(&store).unwrap();
    let two = p.map(&store).unwrap();
    assert_eq!(one, two);
    assert_eq!(one.nodes.len(), 40);
    assert!(
        one.nodes
            .iter()
            .all(|n| n.x >= 0. && n.x <= one.width && n.y >= 0. && n.y <= one.height)
    );
    assert!(
        one.edges
            .iter()
            .all(|&(a, b)| a <= b && (b as usize) < one.nodes.len())
    );
}

#[test]
#[cfg(feature = "discovery")]
fn links_to_notes_outside_the_notebook_are_stubs() {
    let (_dir, store) = library();
    book(&store, "book:o", "Elsewhere", None);
    note_at(
        &store,
        "note:out",
        "Outside note",
        "elsewhere\n",
        "book:o",
        T0,
    );
    note_at(
        &store,
        "note:in",
        "Inside note",
        "points at [[Outside note]]\n",
        "book:r",
        T0 + DAY,
    );
    let map = page(&store, "book:r").map(&store).unwrap();
    let node = map.nodes.iter().find(|n| n.id == "note:in").unwrap();
    assert_eq!(node.stubs, 1);
    assert!(map.nodes.iter().all(|n| n.id != "note:out"));
}

#[test]
fn customising_orders_and_hides_sections_and_the_later_write_wins() {
    let (_dir, store) = library();
    let p = page(&store, "book:r");
    actions::customise(
        &store,
        &p,
        &["inside".into(), "recent".into()],
        &["key_notes".into()],
        NOW,
    )
    .unwrap();
    let view = open_home(&store, &p, on(false), &[]).unwrap();
    let keys: Vec<&str> = view.page.sections.iter().map(|s| s.key.as_str()).collect();
    assert_eq!(keys, sections(&["inside", "recent", "open_tasks", "map"]));
    assert_eq!(
        view.page.hidden,
        vec![("key_notes".into(), "Key notes".into())]
    );
    assert!(actions::customise(&store, &p, &["nope".into()], &[], NOW).is_err());
    actions::reset_layout(&store, &p, NOW + 1).unwrap();
    let view = open_home(&store, &p, on(false), &[]).unwrap();
    let keys: Vec<&str> = view.page.sections.iter().map(|s| s.key.as_str()).collect();
    assert_eq!(
        keys,
        sections(&["key_notes", "recent", "open_tasks", "map", "inside"])
    );
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 32, rng_seed: proptest::test_runner::RngSeed::Fixed(20261017), ..ProptestConfig::default() })]

    /// However the Home note is chosen, made, cleared and notes are moved
    /// around, a notebook marks at most one note as its Home note after
    /// the rule is applied, and its page shows at most one.
    #[test]
    fn prop_at_most_one_home_note_shown_per_notebook(
        steps in proptest::collection::vec((0u8..4, 0usize..2, 0usize..4), 1..12),
    ) {
        let (_dir, store) = open_store();
        book(&store, "book:a", "A", None);
        book(&store, "book:b", "B", None);
        let books = ["book:a", "book:b"];
        for i in 0..4 {
            note_at(&store, &format!("note:n{i}"), &format!("N{i}"), "text\n", books[i % 2], T0 + i as i64);
        }
        let mut now = NOW;
        for (op, b, n) in steps {
            now += 10;
            let id = format!("note:n{n}");
            match op {
                0 => { let _ = actions::choose_home(&store, books[b], &id, now); }
                1 => { let _ = actions::new_home(&store, books[b], "X", now); }
                2 => { let _ = actions::clear_home(&store, books[b], now); }
                _ => {
                    // Move the note to the other notebook, then apply the rule.
                    let to = books[(b + 1) % 2];
                    store.put(DocType::Note, json!({"_id": id, "bookId": to, "updatedAt": now}), Put::Merge, Origin::Ui).unwrap();
                    let _ = actions::after_move(&store, &id, now);
                }
            }
            for book in books {
                prop_assert!(store.home_notes_marked(book).unwrap().len() <= 1);
                let view = open_home(&store, &page(&store, book), on(false), &[]).unwrap();
                let shown = view.page.sections.iter().filter(|s| s.key == "home_note").map(|s| s.items.len()).sum::<usize>();
                prop_assert!(shown <= 1);
            }
        }
    }
}

/// The homepage of a notebook in a library of 50,000 notes: computed
/// sections, the fingerprints and the map layout against their budgets.
/// Run with `cargo test -p kanso-engine --test notebook_home bench -- --ignored --nocapture`.
#[test]
#[ignore = "benchmark: 50,000 notes"]
fn bench_homepage_of_fifty_thousand_notes() {
    use std::time::Instant;
    let (_dir, store) = open_store();
    book(&store, "book:big", "Big", None);
    // Parts 0-3 sit inside "Ten": a subtree of 10,000 notes, the size the
    // budgets are set for.
    book(&store, "book:ten", "Ten", Some("book:big"));
    for n in 0..20 {
        book(
            &store,
            &format!("book:big{n}"),
            &format!("Part {n}"),
            Some(if n < 4 { "book:ten" } else { "book:big" }),
        );
    }
    for n in 0..50_000 {
        let body = format!(
            "# Note {n}\n\nSome words about thing {}.\n- [ ] task {n}\n",
            n % 97
        );
        note_at(
            &store,
            &format!("note:b{n}"),
            &format!("Note {n}"),
            &body,
            &format!("book:big{}", n % 20),
            NOW - i64::from(n) * 1000,
        );
    }
    let measure = |id: &str| {
        let start = Instant::now();
        let big = page(&store, id);
        let view = open_home(&store, &big, on(false), &[]).unwrap();
        let computed = start.elapsed();
        let start = Instant::now();
        let _ = NotebookPage::load(&store, id, NOW).unwrap().unwrap();
        let fingerprints = start.elapsed();
        for key in kanso_core::home::SECTION_KEYS {
            let start = Instant::now();
            let _ = kanso_engine::pages::PageSource::compute(&big, &store, key, false);
            println!("    {key} {:?}", start.elapsed());
        }
        let start = Instant::now();
        let _ = big.map(&store).unwrap();
        let map = start.elapsed();
        println!(
            "{id}: sections {}, computed {computed:?}, load {fingerprints:?}, map {map:?}",
            view.page.sections.len()
        );
        (computed, fingerprints, map)
    };
    macro_rules! t {
        ($name:expr, $e:expr) => {{
            let s = Instant::now();
            let r = $e;
            println!("  {} {:?}", $name, s.elapsed());
            r
        }};
    }
    let b = "book:ten";
    t!("subtree", store.book_subtree(b, false).unwrap());
    t!("stats", store.home_stats(b).unwrap());
    t!("recent", store.home_recent(b, 8).unwrap());
    t!("pinned", store.home_pinned(b, 8).unwrap());
    t!("linked", store.home_linked(b, 64).unwrap());
    t!("children", store.home_children(b).unwrap());
    t!("opentasks", store.home_open_task_count(b).unwrap());
    t!("entities", store.home_entities(b, 8).unwrap());
    t!("home_note", store.home_note(b).unwrap());
    t!("edited", store.home_edited_since(b, 0, 16).unwrap());
    t!("all_notes", store.home_all_notes(b).unwrap());
    t!("graph", store.home_graph_rows(b).unwrap());
    t!(
        "tasks",
        store
            .tasks(&kanso_store::TaskQuery {
                book_ids: Some(store.book_subtree(b, false).unwrap()),
                limit: Some(8),
                ..Default::default()
            })
            .unwrap()
    );
    // The whole library as one notebook: reported, the worst case.
    measure("book:big");
    measure("book:big0");
    // A notebook of 10,000 notes in a library of 50,000: the budgets. Run
    // with `--features discovery` (as the apps are built) for the map.
    // Measured on an M1 Pro: 30 ms with kanso-core, kanso-store and
    // kanso-engine optimised (`--config 'profile.dev.package.kanso-store.opt-level=2'`
    // and the same for the other two), 75 ms unoptimised. The guard is for
    // the unoptimised build, so a regression, not the build, fails it.
    let (computed, fingerprints, map) = measure("book:ten");
    assert!(computed.as_millis() < 120, "computed sections {computed:?}");
    assert!(
        fingerprints.as_millis() < 20,
        "fingerprints {fingerprints:?}"
    );
    // The map's budget is 100 ms: 53 ms optimised, 95 ms unoptimised.
    assert!(map.as_millis() < 150, "map layout {map:?}");
}
