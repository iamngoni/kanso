//! The Journal and templates: today's note, any day's, stepping between
//! days, the month calendar, the daily template and notes from templates.

mod common;

use std::collections::BTreeSet;

use common::*;
use kanso_core::dates::{Date, days_in_month};
use kanso_core::roles::daily_id;
use kanso_mobile::{DayDate, KansoError, Library, LocalTime};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use serde_json::json;

fn config(cases: u32) -> ProptestConfig {
    ProptestConfig {
        cases,
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
        ..ProptestConfig::default()
    }
}

/// 09:30 on 2026-05-`day`.
fn at(day: u32) -> LocalTime {
    LocalTime {
        year: 2026,
        month: 5,
        day,
        hour: 9,
        minute: 30,
        second: 0,
    }
}

fn day(day: u32) -> DayDate {
    DayDate {
        year: 2026,
        month: 5,
        day,
    }
}

/// Makes a note in the Templates notebook through the store and returns its
/// ID.
fn template(phone: &Phone, title: &str, body: &str) -> String {
    let made = phone.library.new_template().unwrap();
    assert!(made.title_first);
    let note = phone.library.note(&made.id).unwrap();
    let store = kanso_store::Store::open(phone.path()).unwrap();
    store
        .put(
            kanso_core::DocType::Note,
            json!({"_id": made.id, "title": title, "body": body}),
            kanso_store::Put::Merge,
            kanso_store::Origin::Ui,
        )
        .unwrap();
    assert_eq!(note.title, "");
    made.id
}

// no-prop-test: a fixed week of days and what each call answers.
#[test]
fn today_any_day_and_the_days_between() {
    let phone = lone_phone();
    let library = &phone.library;
    let today = library.journal_today(at(20)).unwrap();
    assert_eq!(today.id, daily_id(Date::new(2026, 5, 20).unwrap()));
    assert!(!today.title_first);
    assert_eq!(library.journal_today(at(20)).unwrap().id, today.id);
    // Today through `open_day` is today's note; another day is made for it.
    assert_eq!(
        library.journal_open_day(day(20), at(20)).unwrap().id,
        today.id
    );
    let third = library.journal_open_day(day(3), at(20)).unwrap();
    assert_eq!(third.id, daily_id(Date::new(2026, 5, 3).unwrap()));
    assert_eq!(library.note(&third.id).unwrap().title, "2026-05-03");
    // Stepping back from today finds the 3rd; forward from the 3rd is today.
    let back = library
        .journal_step(Some(today.id.clone()), at(20), false)
        .unwrap()
        .unwrap();
    assert_eq!(back.id, third.id);
    let forward = library
        .journal_step(Some(third.id.clone()), at(20), true)
        .unwrap()
        .unwrap();
    assert_eq!(forward.id, today.id);
    // Nothing later than today, nothing before the first.
    assert_eq!(library.journal_step(Some(today.id), at(20), true), Ok(None));
    assert_eq!(
        library.journal_step(Some(third.id), at(20), false),
        Ok(None)
    );
    assert_eq!(
        Library::daily_date_of("note:daily-2026-05-03"),
        Some(day(3))
    );
    assert_eq!(Library::daily_date_of("note:abc"), None);
}

// no-prop-test: a fixed table of what each wrong day or time answers.
#[test]
fn days_that_do_not_exist_are_refused() {
    let phone = lone_phone();
    let mut bad = at(20);
    bad.hour = 24;
    assert!(matches!(
        phone.library.journal_today(bad),
        Err(KansoError::Invalid { .. })
    ));
    for date in [
        DayDate {
            year: 2026,
            month: 2,
            day: 30,
        },
        DayDate {
            year: 2026,
            month: 13,
            day: 1,
        },
        DayDate {
            year: 0,
            month: 1,
            day: 1,
        },
    ] {
        assert!(matches!(
            phone.library.journal_open_day(date, at(20)),
            Err(KansoError::Invalid { .. })
        ));
    }
    assert!(matches!(
        phone.library.journal_month(2026, 13, 0, day(1)),
        Err(KansoError::Invalid { .. })
    ));
    assert!(matches!(
        phone.library.journal_month(2026, 5, 7, day(1)),
        Err(KansoError::Invalid { .. })
    ));
}

// no-prop-test: a fixed template and the offsets it must give.
#[test]
fn the_daily_template_fills_the_day_and_places_the_cursor_in_utf16() {
    let phone = lone_phone();
    let library = &phone.library;
    assert_eq!(library.daily_template(), Ok(None));
    let id = template(&phone, "Daily", "😀 {{date}}\n{{cursor}}\n");
    assert_eq!(
        library
            .templates()
            .unwrap()
            .iter()
            .map(|t| t.id.as_str())
            .collect::<Vec<_>>(),
        [id.as_str()]
    );
    library.set_daily_template(Some(id.clone())).unwrap();
    assert_eq!(library.daily_template(), Ok(Some(id.clone())));
    let made = library.journal_today(at(5)).unwrap();
    let body = library.note(&made.id).unwrap().body.unwrap();
    assert!(body.starts_with("😀 "), "{body}");
    // The cursor is after the first line, which holds an emoji of two
    // UTF-16 units and four bytes.
    assert!(body.starts_with("😀 2026-05-05\n"), "{body}");
    let expected = body
        .split_once('\n')
        .map(|(line, _)| line.encode_utf16().count() + 1);
    assert_eq!(made.cursor16.map(|c| c as usize), expected);
    // Found again: no cursor.
    assert_eq!(library.journal_today(at(5)).unwrap().cursor16, None);
    library.set_daily_template(None).unwrap();
    assert_eq!(library.daily_template(), Ok(None));
    assert_eq!(library.journal_today(at(6)).unwrap().cursor16, None);
}

// no-prop-test: a fixed template, notebook and what each makes.
#[test]
fn a_note_from_a_template_goes_to_the_shown_notebook() {
    let phone = lone_phone();
    let library = &phone.library;
    let id = template(&phone, "Meeting", "# Meeting {{date}}\n\n{{cursor}}");
    let work = library.create_notebook("Work".into(), None).unwrap().id;
    let made = library
        .new_from_template(&id, Some(work.clone()), at(5))
        .unwrap();
    let note = library.note(&made.id).unwrap();
    assert_eq!(note.notebook_id, work);
    assert!(!made.title_first || note.title.trim().is_empty());
    assert_eq!(
        made.cursor16,
        note.body
            .as_deref()
            .and_then(|b| b.find("\n\n"))
            .map(|at| at as u32 + 2)
    );
    // Never into the Templates notebook: the Inbox instead.
    let templates_book = library.note(&id).unwrap().notebook_id;
    let elsewhere = library
        .new_from_template(&id, Some(templates_book.clone()), at(5))
        .unwrap();
    assert_ne!(
        library.note(&elsewhere.id).unwrap().notebook_id,
        templates_book
    );
    assert_eq!(
        library.new_from_template("note:missing", None, at(5)),
        Err(KansoError::NotFound)
    );
}

// no-prop-test: a fixed table of the platform calendars' first weekdays.
#[test]
fn the_platforms_first_weekday_is_the_grids() {
    let got: Vec<u32> = (0..=8).map(Library::journal_week_start).collect();
    assert_eq!(got, [0, 6, 0, 1, 2, 3, 4, 5, 0]);
}

proptest! {
    #![proptest_config(config(24))]

    /// Invariants: the month is whole weeks of seven days in a row, each day
    /// of the month shows once in it, the days that have a note (outside the
    /// trash) are exactly the marked ones, and today is marked at most once.
    #[test]
    fn prop_month_marks_the_days_with_notes(
        noted in prop::collection::btree_set(1u32..=31, 0..8),
        trashed in prop::collection::btree_set(1u32..=31, 0..3),
        year in 2020i32..2032,
        month in 1u32..=12,
        week_start in 0u32..7,
        today in 1u32..=28,
    ) {
        let phone = lone_phone();
        let library = &phone.library;
        let length = days_in_month(year, month);
        let noted: BTreeSet<u32> = noted.into_iter().map(|d| d.min(length)).collect();
        let now = LocalTime { year, month, day: today, hour: 8, minute: 0, second: 0 };
        let mut trashed_days = BTreeSet::new();
        for &d in &noted {
            let made = library
                .journal_open_day(DayDate { year, month, day: d }, now)
                .unwrap();
            if trashed.contains(&d) {
                library.trash(&made.id).unwrap();
                trashed_days.insert(d);
            }
        }
        let grid = library
            .journal_month(year, month, week_start, DayDate { year, month, day: today })
            .unwrap();
        prop_assert_eq!(grid.weekday_labels.len(), 7);
        prop_assert!((4..=6).contains(&grid.weeks.len()));
        let days: Vec<_> = grid.weeks.iter().flatten().copied().collect();
        prop_assert_eq!(days.len() % 7, 0);
        for pair in days.windows(2) {
            let a = Date::new(pair[0].date.year, pair[0].date.month, pair[0].date.day).unwrap();
            let b = Date::new(pair[1].date.year, pair[1].date.month, pair[1].date.day).unwrap();
            prop_assert_eq!(b, a.add_days(1));
        }
        for week in &grid.weeks {
            prop_assert_eq!(week.len(), 7);
            let first = Date::new(week[0].date.year, week[0].date.month, week[0].date.day).unwrap();
            prop_assert_eq!(first.weekday(), week_start as usize);
        }
        let inside: Vec<u32> = days.iter().filter(|d| d.in_month).map(|d| d.date.day).collect();
        prop_assert_eq!(inside, (1..=length).collect::<Vec<_>>());
        let marked: BTreeSet<u32> = days
            .iter()
            .filter(|d| d.in_month && d.has_note)
            .map(|d| d.date.day)
            .collect();
        let expected: BTreeSet<u32> = noted.difference(&trashed_days).copied().collect();
        prop_assert_eq!(marked, expected);
        let todays: Vec<_> = days.iter().filter(|d| d.is_today).collect();
        prop_assert_eq!(todays.len(), 1);
        prop_assert_eq!(todays[0].date.day, today);
    }

    /// Stepping back and forward visits the days that have notes in order,
    /// ending at today, and never lands on a day without one except today.
    #[test]
    fn prop_stepping_visits_the_noted_days_in_order(
        noted in prop::collection::btree_set(1u32..=19, 1..8),
    ) {
        let phone = lone_phone();
        let library = &phone.library;
        for &d in &noted {
            library.journal_open_day(day(d), at(20)).unwrap();
        }
        let mut seen = Vec::new();
        let mut open = Some(library.journal_today(at(20)).unwrap().id);
        while let Some(made) = library.journal_step(open.clone(), at(20), false).unwrap() {
            seen.push(Library::daily_date_of(&made.id).unwrap().day);
            open = Some(made.id);
        }
        let mut expected: Vec<u32> = noted.iter().copied().collect();
        expected.reverse();
        prop_assert_eq!(seen, expected);
        // And forward again to today, passing the same days.
        let mut forward = Vec::new();
        while let Some(made) = library.journal_step(open.clone(), at(20), true).unwrap() {
            forward.push(Library::daily_date_of(&made.id).unwrap().day);
            open = Some(made.id);
        }
        let mut expected: Vec<u32> = noted.iter().copied().skip(1).collect();
        expected.push(20);
        prop_assert_eq!(forward, expected);
    }
}
