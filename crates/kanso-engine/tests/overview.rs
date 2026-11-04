//! Overview end to end (issue #240): Today and the week over a real store,
//! the instant snapshot, privacy for locked notes, and the speed budget of
//! the cheap pass over fifty thousand notes.

use std::time::Instant;

use kanso_core::DocType;
use kanso_core::dates::Date;
use kanso_engine::brief::Now;
use kanso_engine::overview::{self, AgendaKind, Event, ListKind, Options, State};
use kanso_store::{Origin, Put, Store};
use serde_json::{Value, json};

const DAY: i64 = 86_400_000;
/// Wednesday 2026-11-04, 08:30 UTC.
const NOW_MS: i64 = 20_761 * DAY + 8 * 3_600_000 + 30 * 60_000;

fn now() -> Now {
    Now {
        utc_ms: NOW_MS,
        offset_ms: 0,
    }
}

fn store() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    store
        .put(
            DocType::Book,
            json!({"_id": "book:a", "name": "Kanso"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
    (dir, store)
}

fn note(store: &Store, id: &str, title: &str, body: &str, updated: i64) {
    let current = store.get(id).unwrap().map(|d| d.rev);
    let mut doc = json!({"_id": id, "title": title, "body": body, "bookId": "book:a",
        "createdAt": updated, "updatedAt": updated});
    if let Some(rev) = current {
        doc["_rev"] = json!(rev);
    }
    store
        .put(DocType::Note, doc, Put::Upsert, Origin::Ui)
        .unwrap();
}

fn lock() -> Value {
    json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "sealed"})
}

fn today_iso() -> String {
    now().today().iso()
}

#[test]
fn an_empty_library_is_a_new_one() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    let page = overview::today(&store, now(), &Options::new()).unwrap();
    assert_eq!(page.today.state, State::New);
    assert_eq!(page.today.lede, "Nothing on today.");
}

#[test]
fn the_agenda_merges_tasks_and_events_in_time_order() {
    let (_dir, store) = store();
    let today = today_iso();
    let yesterday = now().today().add_days(-3).iso();
    note(
        &store,
        "note:tasks",
        "Work",
        &format!(
            "- [ ] Late thing @{yesterday}\n- [ ] Send the deck @{today}\n- [ ] Call Rudo @{today}T14:00\n- [ ] Done thing @{today}\n"
        ),
        1000,
    );
    let day_start = now().start_of(now().today());
    let mut opts = Options::new();
    opts.events = vec![
        Event {
            uid: "u1".into(),
            start: day_start + 9 * 3_600_000,
            end: day_start + 9 * 3_600_000 + 1_800_000,
            title: "Standup".into(),
            ..Event::default()
        },
        Event {
            uid: "u2".into(),
            start: day_start + 11 * 3_600_000 + 30 * 60_000,
            end: day_start + 12 * 3_600_000,
            title: "Call with Tendai".into(),
            attendees: vec![overview::Attendee {
                name: "Tendai Moyo".into(),
                ..overview::Attendee::default()
            }],
            conference_url: "https://meet.google.com/abc".into(),
            ..Event::default()
        },
        Event {
            uid: "u3".into(),
            start: day_start,
            end: day_start + DAY,
            title: "Sprint 45 ends".into(),
            all_day: true,
            ..Event::default()
        },
        Event {
            uid: "u4".into(),
            start: day_start + 15 * 3_600_000,
            end: day_start + 16 * 3_600_000,
            title: "Declined".into(),
            declined: true,
            ..Event::default()
        },
    ];
    let page = overview::today(&store, now(), &opts).unwrap();
    let t = &page.today;
    let titles: Vec<&str> = t.agenda.rows.iter().map(|r| r.title.as_str()).collect();
    assert_eq!(
        titles,
        ["Late thing", "Standup", "Call with Tendai", "Call Rudo", "Done thing", "Send the deck"]
    );
    assert_eq!(t.agenda.rows[0].kind, AgendaKind::Overdue);
    assert_eq!(t.agenda.rows[0].days_late, 3);
    assert_eq!(t.all_day, ["Sprint 45 ends"]);
    assert!(t.overline.starts_with("WEDNESDAY 4 NOVEMBER"), "{}", t.overline);
    assert!(t.overline.contains("Sprint 45 ends"));
    assert_eq!(
        t.agenda.rows[2].context,
        "11:30\u{2013}12:00 \u{b7} Tendai Moyo \u{b7} Google Meet"
    );
    assert_eq!(
        t.lede,
        "Two meetings and four tasks today, one of them three days late. Next: Standup in 30 minutes."
    );
    assert_eq!(t.greeting, "Good morning");
}

#[test]
fn locked_notes_never_reach_the_page() {
    let (_dir, store) = store();
    let today = today_iso();
    note(&store, "note:secret", "Secret", &format!("- [ ] Hide me @{today}\n"), 1000);
    let doc = store.get("note:secret").unwrap().unwrap();
    store
        .put(
            DocType::Note,
            json!({"_id": "note:secret", "_rev": doc.rev, "title": "Secret", "body": "",
                "bookId": "book:a", "kansoLock": lock(), "createdAt": 1000, "updatedAt": 1000}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
    let page = overview::today(&store, now(), &Options::new()).unwrap();
    let json = serde_json::to_string(&page).unwrap();
    assert!(!json.contains("Hide me") && !json.contains("Secret"));
}

#[test]
fn pick_up_shows_the_words_where_the_caret_was() {
    let (_dir, store) = store();
    let text = "and the arrow tool should snap to the shape's edge, not its centre, so that";
    note(&store, "note:a", "Excalidraw in Kanso", &format!("# T\n\nFirst.\n\n{text}"), 1000);
    store
        .note_writing(
            "note:a",
            4,
            text.chars().count() as u32,
            120,
            &kanso_core::writing::anchor_hash(text),
            NOW_MS - 2 * 3_600_000,
        )
        .unwrap();
    let page = overview::today(&store, now(), &Options::new()).unwrap();
    let list = page
        .today
        .lists
        .iter()
        .find(|l| l.kind == ListKind::PickUp)
        .expect("a pick up list");
    assert_eq!(list.rows[0].title, "Excalidraw in Kanso");
    assert!(list.rows[0].detail.ends_with("so that"));
    assert!(list.rows[0].caret);
    assert_eq!(list.rows[0].line, 4);
    assert_eq!(list.rows[0].place, "Kanso");
    // The open note is left out.
    let mut opts = Options::new();
    opts.open_note = Some("note:a".into());
    let page = overview::today(&store, now(), &opts).unwrap();
    assert!(page.today.lists.iter().all(|l| l.kind != ListKind::PickUp));
}

#[test]
fn the_third_list_folds_into_also() {
    let (_dir, store) = store();
    let person = "---\nkanso: person\n---\nRudo page.\n";
    note(&store, "note:rudo", "Rudo", person, 1000);
    note(&store, "note:w", "Work", "Waiting on Rudo for the sign-off.", 1000);
    note(&store, "note:p", "Draft", "A long enough paragraph of my own words that I was writing.", 5000);
    store.note_writing("note:p", 0, 20, 100, "", NOW_MS - 3_600_000).unwrap();
    let mut opts = Options::new();
    opts.agent = vec![overview::ActivityItem {
        kind: overview::RowKind::AgentSuggestion,
        note_id: "note:p".into(),
        title: "Release checklist".into(),
        who: "Claude".into(),
        count: 4,
        at: NOW_MS - 600_000,
    }];
    let page = overview::today(&store, now(), &opts).unwrap();
    let kinds: Vec<ListKind> = page.today.lists.iter().map(|l| l.kind).collect();
    assert_eq!(kinds, [ListKind::PickUp, ListKind::Waiting]);
    assert!(page
        .today
        .also
        .iter()
        .any(|l| l.kind == overview::AlsoKind::Folded(ListKind::Activity)));
    assert_eq!(page.today.lists[1].rows[0].title, "Rudo");
    assert!(page.today.lists[1].rows[0].quiet_days >= 5);
}

#[test]
fn the_snapshot_round_trips_and_belongs_to_one_day() {
    let (_dir, store) = store();
    note(&store, "note:a", "A", "- [ ] Thing @2026-11-04\n", 1000);
    let page = overview::today(&store, now(), &Options::new()).unwrap();
    overview::save_snapshot(&store, &page).unwrap();
    assert_eq!(overview::load_snapshot(&store, &page.day), Some(page.clone()));
    assert_eq!(overview::load_snapshot(&store, "2026-11-05"), None);
}

#[test]
fn the_week_counts_what_was_written_and_what_is_ahead() {
    let (_dir, store) = store();
    let monday = Date::parse_iso("2026-11-02").unwrap();
    let start = now().start_of(monday);
    note(&store, "note:a", "Excalidraw", "- [x] Shipped it\n\nWords.", start + 3_600_000);
    note(&store, "note:b", "Another", "- [ ] Next thing @2026-11-06\n", start + DAY + 3_600_000);
    let week = overview::week(&store, monday, now(), &Options::new()).unwrap();
    assert_eq!(week.from, "2026-11-02");
    assert_eq!(week.range_label, "2 \u{2013} 8 NOVEMBER");
    assert_eq!(week.week_label, "Week 45");
    assert_eq!(week.wrote, 2);
    assert_eq!(week.projects[0].name, "Kanso");
    assert_eq!(week.projects[0].next_task, "Next thing");
    assert!(week.lede.starts_with("You wrote 2 notes"));
    let due: Vec<&str> = week
        .ahead
        .iter()
        .flat_map(|d| d.rows.iter().map(|r| r.title.as_str()))
        .collect();
    assert_eq!(due, ["Next thing"]);
}

/// The cheap pass stays inside its budget on a big library.
#[test]
#[ignore = "slow: builds fifty thousand notes; run with --ignored --release"]
fn bench_overview_of_fifty_thousand_notes() {
    let (_dir, store) = store();
    let today = today_iso();
    for i in 0..50_000 {
        let body = if i % 50 == 0 {
            format!("- [ ] Task {i} @{today}\n\nSome words of note {i}.")
        } else {
            format!("Some words of note {i}. I'll look at it again.")
        };
        note(&store, &format!("note:n{i}"), &format!("Note {i}"), &body, 1000 + i);
    }
    let start = Instant::now();
    let page = overview::today(&store, now(), &Options::new()).unwrap();
    let took = start.elapsed();
    assert!(!page.today.agenda.rows.is_empty());
    assert!(took.as_millis() < 200, "phase A took {took:?}");
    let monday = Date::parse_iso("2026-11-02").unwrap();
    let start = Instant::now();
    overview::week(&store, monday, now(), &Options::new()).unwrap();
    assert!(start.elapsed().as_millis() < 800);
}
