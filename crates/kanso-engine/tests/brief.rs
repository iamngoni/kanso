//! The morning brief and the weekly review over a real store: what they
//! list, and that locked and `#ai-exclude` notes never reach them. CI pins
//! the seed with `PROPTEST_RNG_SEED`.

use std::collections::BTreeSet;
use std::time::Instant;

use kanso_core::DocType;
use kanso_core::dates::Date;
use kanso_core::roles::daily_id;
use kanso_engine::brief::{self, Now, Why, morning_brief, week_start, weekly_review};
use kanso_store::{Origin, Put, Store};
use proptest::prelude::*;
use serde_json::{Value, json};

const DAY: i64 = 86_400_000;
const CANARY: &str = "zebracanary";

fn open() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    (dir, store)
}

/// Monday 2 November 2026, 10:30 in a zone two hours ahead of UTC.
fn now() -> Now {
    let date = Date::new(2026, 11, 2).unwrap();
    let offset_ms = 2 * 3_600_000;
    Now {
        utc_ms: date.to_days() * DAY + 10 * 3_600_000 + 30 * 60_000 - offset_ms,
        offset_ms,
    }
}

/// Local noon on `date`, in UTC ms.
fn noon(date: Date) -> i64 {
    now().start_of(date) + 12 * 3_600_000
}

fn put(store: &Store, doc_type: DocType, body: Value) -> String {
    store
        .put(doc_type, body, Put::Upsert, Origin::Ui)
        .unwrap()
        .id
}

fn note(store: &Store, id: &str, title: &str, body: &str, book: &str, at: i64) -> String {
    put(
        store,
        DocType::Note,
        json!({"_id": id, "title": title, "body": body, "bookId": book,
               "createdAt": at, "updatedAt": at}),
    )
}

fn lock_envelope() -> Value {
    json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "sealed"})
}

fn day(offset: i64) -> Date {
    Date::new(2026, 11, 2).unwrap().add_days(offset)
}

#[test]
fn the_brief_lists_what_is_due_open_and_relevant() {
    let (_dir, store) = open();
    let b = put(&store, DocType::Book, json!({"_id": "book:a", "name": "Work"}));
    let today = day(0);
    note(
        &store,
        &daily_id(today),
        "Today",
        "Plan: see [[Plans]].\nI'll call Rudo.",
        &b,
        noon(today),
    );
    note(
        &store,
        &daily_id(today.add_days(-365)),
        "A year ago",
        "Last year's entry.",
        &b,
        noon(today.add_days(-365)),
    );
    note(&store, "note:plans", "Plans", "The plans\nfor the quarter", &b, noon(day(-3)));
    note(
        &store,
        "note:tasks",
        "Tasks",
        "- [ ] pay rent @2026-11-01\n- [ ] standup @2026-11-02T14:00\n- [ ] early @2026-11-02T08:00\n\
         - [ ] later @2026-11-09\n- [x] done @2026-10-01\nWaiting on Tendai for the quote.\n",
        &b,
        noon(day(-1)),
    );
    let brief = morning_brief(&store, now(), None).unwrap();
    let due: Vec<_> = brief.due.iter().map(|t| t.text.as_str()).collect();
    assert_eq!(
        due,
        [
            "pay rent @2026-11-01",
            "early @2026-11-02T08:00",
            "standup @2026-11-02T14:00"
        ]
    );
    assert_eq!(brief.later_today.len(), 1);
    assert_eq!(brief.later_today[0].text, "standup @2026-11-02T14:00");
    assert_eq!(brief.loops_total, 2);
    assert_eq!(brief.loops.len(), 2);
    let relevant: Vec<_> = brief.relevant.iter().map(|r| (r.title.as_str(), r.why)).collect();
    assert_eq!(relevant, [("Plans", Why::Linked), ("A year ago", Why::YearAgo)]);
    assert_eq!(brief.relevant[0].excerpt, "The plans for the quarter");
    assert!(!brief.is_empty());
    assert_eq!(brief.notification_text(), "3 due, 2 open loops");
}

#[test]
fn yesterdays_counts_and_the_empty_brief() {
    let (_dir, store) = open();
    let b = put(&store, DocType::Book, json!({"_id": "book:a", "name": "Work"}));
    let empty = morning_brief(&store, now(), None).unwrap();
    assert!(empty.is_empty());
    assert_eq!(empty.notification_text(), "Nothing due today");
    note(&store, "note:one", "One", "- [ ] a\n- [ ] b\n", &b, noon(day(-1)));
    note(&store, "note:two", "Two", "text", &b, noon(day(-1)));
    note(&store, "note:old", "Old", "text", &b, noon(day(-5)));
    // Ticking a task finishes it (at the store's clock, which is now: the
    // brief looks at yesterday by the date it was given, so shift it).
    store
        .put(
            DocType::Note,
            json!({"_id": "note:one", "body": "- [x] a\n- [ ] b\n"}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    let real_now = Now {
        utc_ms: kanso_core::now_ms() + DAY,
        offset_ms: 0,
    };
    let brief = morning_brief(&store, real_now, None).unwrap();
    assert_eq!(brief.yesterday.finished, 1);
    assert_eq!(brief.yesterday.wrote, 0, "the notes were made in 2026");
    let brief = morning_brief(&store, now(), None).unwrap();
    assert_eq!(brief.yesterday.wrote, 2);
    assert_eq!(brief.yesterday.finished, 0);
    assert_eq!(brief::notice_text(1, 1), "1 due, 1 open loop");
}

#[test]
fn old_loops_are_still_open_from_last_week_and_the_brief_is_capped() {
    let (_dir, store) = open();
    let b = put(&store, DocType::Book, json!({"_id": "book:a", "name": "Work"}));
    for i in 0..9 {
        note(
            &store,
            &format!("note:r{i}"),
            "Recent",
            &format!("I'll do recent thing number {i}."),
            &b,
            noon(day(-(i % 3))),
        );
    }
    for i in 0..5 {
        note(
            &store,
            &format!("note:s{i}"),
            "Stale",
            &format!("I'll do stale thing number {i}."),
            &b,
            noon(day(-20 - i)),
        );
    }
    let brief = morning_brief(&store, now(), None).unwrap();
    assert_eq!(brief.loops_total, 14);
    assert_eq!(brief.loops.len(), brief::BRIEF_LOOPS);
    let stale = brief
        .loops
        .iter()
        .filter(|l| l.note_id.starts_with("note:s"))
        .count();
    assert_eq!(stale, 3, "the oldest are shown beside the newest");
}

#[test]
fn the_weekly_review_covers_the_week() {
    let (_dir, store) = open();
    let b = put(&store, DocType::Book, json!({"_id": "book:a", "name": "Work"}));
    let monday = week_start(day(3));
    assert_eq!(monday, day(0));
    note(&store, "note:new", "New", "I'll call Rudo.", &b, noon(day(1)));
    // Created last week, changed this week.
    put(
        &store,
        DocType::Note,
        json!({"_id": "note:changed", "title": "Changed", "body": "text", "bookId": b,
               "createdAt": noon(day(-10)), "updatedAt": noon(day(2))}),
    );
    note(&store, "note:before", "Before", "text", &b, noon(day(-10)));
    note(&store, "note:after", "After", "text", &b, noon(day(8)));
    note(
        &store,
        "note:tasks",
        "Tasks",
        "- [ ] overdue @2026-10-30\n- [ ] next week @2026-11-10\n- [ ] far @2026-12-25\n",
        &b,
        noon(day(-8)),
    );
    let review = weekly_review(&store, monday, now()).unwrap();
    assert_eq!((review.from, review.to), (day(0), day(6)));
    let ids: BTreeSet<_> = review.written.iter().map(|n| n.id.as_str()).collect();
    assert_eq!(ids, BTreeSet::from(["note:new", "note:changed"]));
    assert_eq!(review.counts_text(), "1 new, 1 changed");
    assert_eq!(review.written[0].book_name, "Work");
    assert_eq!(review.still_open.len(), 1);
    assert_eq!(review.overdue.len(), 1);
    assert_eq!(review.next.len(), 1);
    assert_eq!(review.next[0].text, "next week @2026-11-10");
    let md = brief::review_markdown(&review);
    assert!(md.starts_with("# Weekly review \u{2014} 2026-W45\n"));
    assert!(md.contains("[[New]]") && md.contains("[[Changed]]"));
}

#[test]
fn saving_a_review_writes_one_journal_note_per_week_and_asks_before_replacing() {
    let (_dir, store) = open();
    let b = put(&store, DocType::Book, json!({"_id": "book:a", "name": "Work"}));
    note(&store, "note:new", "New", "text", &b, noon(day(1)));
    let review = weekly_review(&store, day(0), now()).unwrap();
    assert_eq!(brief::saved_review(&store, day(0)).unwrap(), None);
    let id = match brief::save_review(&store, &review, false, Origin::Ui).unwrap() {
        brief::Saved::Written { id } => id,
        other => panic!("{other:?}"),
    };
    assert_eq!(id, "note:weekly-review-2026-w45");
    let saved = store.note(&id).unwrap().unwrap();
    assert_eq!(saved.title, "Weekly review \u{2014} 2026-W45");
    assert_eq!(saved.book_id, "book:journal");
    assert!(saved.body.contains("[[New]]"));
    // The same week again asks first.
    assert_eq!(
        brief::save_review(&store, &review, false, Origin::Ui).unwrap(),
        brief::Saved::Exists { id: id.clone() }
    );
    note(&store, "note:more", "More", "text", &b, noon(day(2)));
    let again = weekly_review(&store, day(0), now()).unwrap();
    brief::save_review(&store, &again, true, Origin::Ui).unwrap();
    assert!(store.note(&id).unwrap().unwrap().body.contains("[[More]]"));
    assert_eq!(brief::saved_review(&store, day(0)).unwrap(), Some(id));
}

// MARK: Properties

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Plain,
    Locked,
    Excluded,
    InLockedBook,
    Trashed,
}

fn kind() -> impl Strategy<Value = Kind> {
    prop_oneof![
        Just(Kind::Plain),
        Just(Kind::Locked),
        Just(Kind::Excluded),
        Just(Kind::InLockedBook),
        Just(Kind::Trashed),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 32, ..ProptestConfig::default() })]

    /// However plain, locked, locked-notebook, `#ai-exclude` and trashed
    /// notes mix, nothing locked or trashed is in the brief or the review,
    /// and nothing excluded is suggested or counted for themes.
    #[test]
    fn prop_brief_never_contains_a_locked_or_excluded_note(
        kinds in prop::collection::vec(kind(), 1..8),
    ) {
        let (_dir, store) = open();
        let open_book = put(&store, DocType::Book, json!({"_id": "book:open", "name": "Open"}));
        let private = put(&store, DocType::Book, json!({"_id": "book:private", "name": "Private"}));
        let exclude = put(&store, DocType::Tag, json!({"name": "ai-exclude"}));
        let today = day(0);
        let mut links = String::new();
        let mut notes = Vec::new();
        for (n, kind) in kinds.iter().enumerate() {
            let id = format!("note:n{n}");
            let book = if *kind == Kind::InLockedBook { &private } else { &open_book };
            // Every note says the canary, has a task and a loop.
            let body = format!(
                "{CANARY}{n} orchard orchard pruning\n- [ ] task {CANARY}{n} @2026-11-01\nI'll call about {CANARY}{n}."
            );
            note(&store, &id, &format!("Note {n}"), &body, book, noon(day(1)));
            links.push_str(&format!("[[Note {n}]] "));
            notes.push((id, *kind));
        }
        note(&store, &daily_id(today), "Today", &links, &open_book, noon(today));
        for (id, kind) in &notes {
            match kind {
                Kind::Plain | Kind::InLockedBook => {}
                Kind::Locked => {
                    let mut sealed = store.get_api(id, None, false).unwrap();
                    sealed["body"] = "".into();
                    sealed["kansoLock"] = lock_envelope();
                    store.lock_note(sealed).unwrap();
                }
                Kind::Excluded => {
                    store.put(DocType::Note, json!({"_id": id, "tags": [exclude]}),
                        Put::Merge, Origin::Ui).unwrap();
                }
                Kind::Trashed => {
                    store.put(DocType::Note, json!({"_id": id, "bookId": "trash"}),
                        Put::Merge, Origin::Ui).unwrap();
                }
            }
        }
        if notes.iter().any(|(_, k)| *k == Kind::InLockedBook) {
            let sealed: Vec<Value> = notes.iter()
                .filter(|(_, k)| *k == Kind::InLockedBook)
                .map(|(id, _)| {
                    let mut s = store.get_api(id, None, false).unwrap();
                    s["body"] = "".into();
                    s["kansoLock"] = lock_envelope();
                    s
                })
                .collect();
            let mut book = store.get_api(&private, None, false).unwrap();
            book.as_object_mut().unwrap().remove("count");
            book["kansoLock"] = lock_envelope();
            store.lock_book(book, sealed).unwrap();
        }
        let hidden: BTreeSet<&str> = notes.iter()
            .filter(|(_, k)| matches!(k, Kind::Locked | Kind::InLockedBook | Kind::Trashed))
            .map(|(id, _)| id.as_str()).collect();
        let excluded: BTreeSet<&str> = notes.iter()
            .filter(|(_, k)| *k == Kind::Excluded).map(|(id, _)| id.as_str()).collect();
        let title_of = |id: &str| format!("Note {}", &id[6..]);

        let brief = morning_brief(&store, now(), None).unwrap();
        let review = weekly_review(&store, week_start(today), now()).unwrap();
        let shown: Vec<&str> = brief.due.iter().map(|t| t.note_id.as_str())
            .chain(brief.loops.iter().map(|l| l.note_id.as_str()))
            .chain(brief.later_today.iter().map(|t| t.note_id.as_str()))
            .chain(brief.relevant.iter().map(|r| r.id.as_str()))
            .chain(review.written.iter().map(|n| n.id.as_str()))
            .chain(review.finished.iter().map(|f| f.note_id.as_str()))
            .chain(review.still_open.iter().map(|l| l.note_id.as_str()))
            .chain(review.overdue.iter().map(|t| t.note_id.as_str()))
            .chain(review.next.iter().map(|t| t.note_id.as_str()))
            .collect();
        for id in &shown {
            prop_assert!(!hidden.contains(id), "{id} is locked or trashed");
        }
        for r in &brief.relevant {
            prop_assert!(!excluded.contains(r.id.as_str()), "{} is excluded", r.id);
        }
        // Words only ever come from notes that may be read.
        let mut text = String::new();
        for theme in &review.themes { text.push_str(&theme.term); text.push(' '); }
        text.push_str(&format!("{brief:?}"));
        text.push_str(&brief::review_markdown(&review));
        for id in hidden.iter() {
            prop_assert!(!text.contains(&format!("{CANARY}{}", &id[6..])), "canary of {id}");
            prop_assert!(!text.contains(&format!("[[{}]]", title_of(id))), "link to {id}");
        }
        for id in excluded.iter() {
            prop_assert!(!brief.relevant.iter().any(|r| r.title == title_of(id)));
            prop_assert!(!review.themes.iter().any(|t| t.term.contains(CANARY)));
        }
    }

    /// The review's counts are the notes the store has in that week.
    #[test]
    fn prop_review_counts_match_store(
        specs in prop::collection::vec((-10i64..10, 0i64..4), 0..12),
    ) {
        let (_dir, store) = open();
        let b = put(&store, DocType::Book, json!({"_id": "book:a", "name": "Work"}));
        let monday = day(0);
        let from = now().start_of(monday);
        let to = now().start_of(monday.add_days(7));
        let (mut new, mut changed) = (0usize, 0usize);
        for (n, (created_day, age)) in specs.iter().enumerate() {
            let created = noon(day(*created_day));
            let updated = created + age * DAY;
            put(&store, DocType::Note, json!({
                "_id": format!("note:n{n}"), "title": format!("N{n}"), "body": "text",
                "bookId": b, "createdAt": created, "updatedAt": updated,
            }));
            let created_in = (from..to).contains(&created);
            let updated_in = (from..to).contains(&updated);
            if created_in { new += 1; } else if updated_in { changed += 1; }
        }
        let review = weekly_review(&store, monday, now()).unwrap();
        prop_assert_eq!(review.new_count(), new);
        prop_assert_eq!(review.changed_count(), changed);
        prop_assert_eq!(review.written.len(), new + changed);
        prop_assert_eq!(
            store.notes_changed_between(from, to).unwrap().len(),
            new + changed
        );
    }
}

/// A big generated library under the budgets of the design: the brief in
/// under 100 ms, the open-loops list in under 30 ms, the review in a few
/// hundred. Run with `KANSO_PERF_NOTES=50000 cargo test -p kanso-engine
/// --test brief --release -- --ignored`; the default is a tenth of that.
#[test]
#[ignore = "slow: builds a large library"]
fn the_brief_and_the_review_are_fast_on_a_large_library() {
    let n: usize = std::env::var("KANSO_PERF_NOTES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5000);
    let (_dir, store) = open();
    let b = put(&store, DocType::Book, json!({"_id": "book:a", "name": "Work"}));
    let docs: Vec<Value> = (0..n)
        .map(|i| {
            let day_offset = -((i % 400) as i64);
            json!({
                "_id": format!("note:p{i}"), "title": format!("Perf {i}"),
                "body": format!("# Perf {i}\n\nIntro text about topic{} and gardens.\nI'll follow the thread {i}.\n- [ ] task {i} @2026-11-0{}\n", i % 50, 1 + i % 9),
                "bookId": b, "createdAt": noon(day(day_offset)), "updatedAt": noon(day(day_offset)),
            })
        })
        .collect();
    for doc in docs {
        store.put(DocType::Note, doc, Put::Upsert, Origin::Import).unwrap();
    }
    let t = Instant::now();
    let brief = morning_brief(&store, now(), None).unwrap();
    let brief_ms = t.elapsed().as_millis();
    let t = Instant::now();
    let loops = store.open_loops(&kanso_store::LoopQuery::default()).unwrap();
    let loops_ms = t.elapsed().as_millis();
    let t = Instant::now();
    let review = weekly_review(&store, week_start(day(0)), now()).unwrap();
    let review_ms = t.elapsed().as_millis();
    eprintln!(
        "{n} notes: brief {brief_ms} ms ({} due, {} loops), open_loops {loops_ms} ms ({}), review {review_ms} ms ({} written)",
        brief.due.len(), brief.loops_total, loops.len(), review.written.len()
    );
    let scale = (n as u128).max(5000) as f64 / 50_000.0;
    assert!((brief_ms as f64) < 100.0 * scale.max(1.0) * 10.0);
    assert!((loops_ms as f64) < 30.0 * scale.max(1.0) * 10.0);
}
