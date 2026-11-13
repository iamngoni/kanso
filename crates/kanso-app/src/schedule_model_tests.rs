use proptest::prelude::*;
use proptest::test_runner::{Config, RngAlgorithm, RngSeed};

use super::*;

fn d(year: i32, month: u32, day: u32) -> Date {
    Date::new(year, month, day).unwrap()
}

fn item(date: Date, kind: ScheduleKind, projected: bool) -> ScheduleItem {
    ScheduleItem {
        kind,
        note_id: "note:a".into(),
        note_title: "A".into(),
        book_id: "book:a".into(),
        line: (kind == ScheduleKind::Task).then_some(0),
        text: "film @2026-11-14T15:00 every week".into(),
        date,
        time: Some((15, 0)),
        zone: None,
        repeat: None,
        done: false,
        property: (kind == ScheduleKind::Note).then(|| "publish".to_owned()),
        projected,
    }
}

#[test]
fn a_month_is_whole_weeks_from_the_first_weekday() {
    // November 2026 starts on a Sunday and ends on a Monday.
    let monday = range_of(Mode::Month, d(2026, 11, 12), 0);
    assert_eq!(monday, d(2026, 10, 26)..d(2026, 12, 7));
    let sunday = range_of(Mode::Month, d(2026, 11, 12), 6);
    assert_eq!(sunday, d(2026, 11, 1)..d(2026, 12, 6));
}

#[test]
fn a_week_is_seven_days_from_the_first_weekday() {
    // Thursday 12 November 2026.
    assert_eq!(
        range_of(Mode::Week, d(2026, 11, 12), 0),
        d(2026, 11, 9)..d(2026, 11, 16)
    );
    assert_eq!(
        range_of(Mode::Week, d(2026, 11, 12), 6),
        d(2026, 11, 8)..d(2026, 11, 15)
    );
}

#[test]
fn titles_name_the_month_or_the_week() {
    assert_eq!(title(Mode::Month, d(2026, 11, 12), 0), "November 2026");
    assert_eq!(title(Mode::Week, d(2026, 11, 12), 0), "9 – 15 Nov 2026");
    assert_eq!(title(Mode::Week, d(2026, 11, 30), 0), "30 Nov – 6 Dec 2026");
    assert_eq!(
        title(Mode::Week, d(2026, 12, 31), 0),
        "28 Dec 2026 – 3 Jan 2027"
    );
}

#[test]
fn steps_go_by_a_month_or_a_week() {
    assert_eq!(step_period(Mode::Month, d(2026, 1, 31), 1), d(2026, 2, 28));
    assert_eq!(step_period(Mode::Week, d(2026, 11, 12), -1), d(2026, 11, 5));
}

#[test]
fn the_filters_ask_the_engine_for_what_they_name() {
    let range = d(2026, 11, 1)..d(2026, 12, 1);
    let none = Filters::default().query(range.clone());
    assert_eq!(none.books, None);
    assert_eq!(none.date_keys, None);
    assert!(none.include_done);

    let mut filters = Filters::default();
    filters.toggle_book("book:a");
    filters.toggle_key("publish");
    filters.hide_done = true;
    let query = filters.query(range.clone());
    assert_eq!(query.range, range);
    assert_eq!(query.books, Some(vec!["book:a".to_owned()]));
    assert_eq!(query.date_keys, Some(vec!["publish".to_owned()]));
    assert!(!query.include_done);
    assert!(filters.is_refined());
    filters.toggle_book("book:a");
    filters.toggle_key("publish");
    filters.hide_done = false;
    assert!(!filters.is_refined());
}

#[test]
fn items_sit_on_their_days_in_order() {
    let items = [
        item(d(2026, 11, 14), ScheduleKind::Task, false),
        item(d(2026, 11, 14), ScheduleKind::Note, false),
        item(d(2026, 11, 21), ScheduleKind::Task, true),
    ];
    let days = by_day(&items);
    assert_eq!(days[&d(2026, 11, 14)], vec![0, 1]);
    assert_eq!(days[&d(2026, 11, 21)], vec![2]);
}

#[test]
fn a_projected_occurrence_cannot_be_dragged_but_a_note_and_a_real_task_can() {
    assert!(can_drag(&item(d(2026, 11, 14), ScheduleKind::Task, false)));
    assert!(can_drag(&item(d(2026, 11, 14), ScheduleKind::Note, false)));
    assert!(!can_drag(&item(d(2026, 11, 21), ScheduleKind::Task, true)));
}

#[test]
fn marks_say_what_a_note_is() {
    assert_eq!(note_mark(Some("publish")), "▶");
    assert_eq!(note_mark(Some("embargo")), "⏸");
    assert_eq!(note_mark(Some("due")), "◆");
}

#[test]
fn the_selection_steps_and_stops_at_the_ends() {
    assert_eq!(step_selection(None, 3, 1), Some(0));
    assert_eq!(step_selection(None, 3, -1), Some(2));
    assert_eq!(step_selection(Some(2), 3, 1), Some(2));
    assert_eq!(step_selection(Some(0), 3, -1), Some(0));
    assert_eq!(step_selection(Some(1), 0, 1), None);
    assert_eq!(keep_selection(Some(5), 3), Some(2));
    assert_eq!(keep_selection(Some(1), 0), None);
}

// MARK: Properties

fn config() -> Config {
    Config {
        cases: 512,
        rng_seed: RngSeed::Fixed(0x0073_6368_6564),
        rng_algorithm: RngAlgorithm::ChaCha,
        failure_persistence: None,
        ..Config::default()
    }
}

fn gen_day() -> impl Strategy<Value = Date> {
    (-20_000i64..60_000).prop_map(Date::from_days)
}

proptest! {
    #![proptest_config(config())]

    /// A week is seven consecutive days, starts on the week's first day and
    /// holds the day it was asked for.
    #[test]
    fn prop_a_week_holds_its_day_and_starts_on_the_first_weekday(
        day in gen_day(),
        week_start in 0usize..7,
    ) {
        let days = week_of(day, week_start);
        prop_assert!(days.contains(&day));
        prop_assert_eq!(days[0].weekday(), week_start);
        for pair in days.windows(2) {
            prop_assert_eq!(pair[1].to_days() - pair[0].to_days(), 1);
        }
        let range = range_of(Mode::Week, day, week_start);
        prop_assert_eq!(range.start, days[0]);
        prop_assert_eq!(range.end.to_days() - range.start.to_days(), 7);
    }

    /// A month's range is whole weeks that hold every day of the month and
    /// the focused day, from the first weekday.
    #[test]
    fn prop_a_month_range_is_whole_weeks_holding_the_month(
        day in gen_day(),
        week_start in 0usize..7,
    ) {
        let range = range_of(Mode::Month, day, week_start);
        let length = range.end.to_days() - range.start.to_days();
        prop_assert_eq!(length % 7, 0);
        prop_assert!((28..=42).contains(&length));
        prop_assert_eq!(range.start.weekday(), week_start);
        prop_assert!(range.start <= day && day < range.end);
        let first = Date::new(day.year, day.month, 1).unwrap();
        prop_assert!(range.start <= first);
        let last = first.add_days(i64::from(kanso_core::dates::days_in_month(day.year, day.month)) - 1);
        prop_assert!(last < range.end);
    }

    /// Stepping a week and back is the identity; stepping any period puts
    /// the new focus in the period it went to.
    #[test]
    fn prop_stepping_lands_in_the_next_period(
        day in gen_day(),
        week_start in 0usize..7,
        delta in -24i32..24,
    ) {
        let there = step_period(Mode::Week, day, delta);
        prop_assert_eq!(step_period(Mode::Week, there, -delta), day);
        prop_assert_eq!(there.weekday(), day.weekday());

        let moved = step_period(Mode::Month, day, delta);
        let (year, month) = kanso_engine::calendar::shift_month(day.year, day.month, delta);
        prop_assert_eq!((moved.year, moved.month), (year, month));
        let range = range_of(Mode::Month, moved, week_start);
        prop_assert!(range.start <= moved && moved < range.end);
    }

    /// Toggling a notebook or a property twice changes nothing, and the
    /// query names exactly the toggled ones (nothing when none).
    #[test]
    fn prop_filters_toggle_and_ask_for_exactly_what_is_on(
        ids in proptest::collection::vec("[a-c]", 0..8),
        keys in proptest::collection::vec("[x-z]", 0..8),
        hide in proptest::bool::ANY,
    ) {
        let mut filters = Filters { hide_done: hide, ..Filters::default() };
        for id in &ids { filters.toggle_book(id); }
        for key in &keys { filters.toggle_key(key); }
        let on = |list: &[String], name: &str| list.iter().filter(|x| *x == name).count() % 2 == 1;
        for name in ["a", "b", "c"] {
            prop_assert_eq!(filters.books.iter().any(|x| x == name), on(&ids, name));
        }
        prop_assert!(filters.books.iter().collect::<std::collections::HashSet<_>>().len() == filters.books.len());
        let range = d(2026, 11, 1)..d(2026, 12, 1);
        let query = filters.query(range);
        prop_assert_eq!(query.books.is_some(), !filters.books.is_empty());
        prop_assert_eq!(query.date_keys.is_some(), !filters.keys.is_empty());
        prop_assert_eq!(query.include_done, !hide);
        prop_assert_eq!(filters.is_refined(), query.books.is_some() || query.date_keys.is_some() || hide);
        let before = filters.clone();
        filters.toggle_book("q");
        filters.toggle_book("q");
        filters.toggle_key("q");
        filters.toggle_key("q");
        prop_assert_eq!(filters, before);
    }

    /// Every item is on exactly one day, once, and the days list them in
    /// the order given.
    #[test]
    fn prop_every_item_is_on_one_day(days in proptest::collection::vec(0i64..30, 0..40)) {
        let items: Vec<ScheduleItem> = days
            .iter()
            .map(|n| item(Date::from_days(20_000 + n), ScheduleKind::Task, false))
            .collect();
        let grouped = by_day(&items);
        let mut seen: Vec<usize> = grouped.values().flatten().copied().collect();
        prop_assert_eq!(seen.len(), items.len());
        for (day, ixs) in &grouped {
            prop_assert!(ixs.windows(2).all(|pair| pair[0] < pair[1]));
            prop_assert!(ixs.iter().all(|&ix| items[ix].date == *day));
        }
        seen.sort_unstable();
        prop_assert_eq!(seen, (0..items.len()).collect::<Vec<_>>());
    }

    /// The selection is always on an item or on none, and a step never
    /// leaves the list.
    #[test]
    fn prop_the_selection_stays_on_an_item(
        selected in proptest::option::of(0usize..10),
        count in 0usize..8,
        delta in -3isize..4,
    ) {
        for next in [step_selection(selected, count, delta), keep_selection(selected, count)] {
            match next {
                Some(at) => prop_assert!(at < count),
                None => prop_assert!(count == 0 || selected.is_none() && next.is_none()),
            }
        }
    }
}
