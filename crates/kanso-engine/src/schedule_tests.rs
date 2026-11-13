use kanso_core::DocType;
use kanso_core::repeat_model::{Every, MonthDay, Repeat, WeekdaySet};
use kanso_core::schema::{PropertyDef, PropertyType};
use kanso_core::tasks::task_items;
use kanso_core::tick::tick_task_at;
use kanso_store::{Origin, Put};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use serde_json::json;

use super::*;

fn day(year: i32, month: u32, day: u32) -> Date {
    Date::new(year, month, day).unwrap()
}

fn month(year: i32, month: u32) -> Range<Date> {
    let end = if month == 12 {
        day(year + 1, 1, 1)
    } else {
        day(year, month + 1, 1)
    };
    day(year, month, 1)..end
}

fn open() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    for (id, name) in [("book:a", "Studio"), ("book:b", "Other")] {
        store
            .put(
                DocType::Book,
                json!({"_id": id, "name": name}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
    }
    (dir, store)
}

fn note(store: &Store, id: &str, book: &str, title: &str, body: &str) {
    store
        .put(
            DocType::Note,
            json!({"_id": id, "title": title, "bookId": book, "body": body}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
}

fn def(key: &str, kind: PropertyType) -> PropertyDef {
    PropertyDef {
        key: key.into(),
        kind,
        options: Vec::new(),
    }
}

fn dates_of(items: &[ScheduleItem], note_id: &str) -> Vec<Date> {
    items
        .iter()
        .filter(|item| item.note_id == note_id)
        .map(|item| item.date)
        .collect()
}

fn run(store: &Store, range: Range<Date>) -> Vec<ScheduleItem> {
    schedule(store, &ScheduleQuery::new(range)).unwrap()
}

fn repeat_of(words: &str) -> Repeat {
    let body = format!("- [ ] x @2026-01-01 {words}\n");
    task_items(&body)[0].repeat.expect("a repeat")
}

#[test]
fn a_month_of_weekly_and_monthly_tasks() {
    let (_dir, store) = open();
    note(
        &store,
        "note:w",
        "book:a",
        "Weekly",
        "- [ ] Review @2026-10-30T15:00 every week\n",
    );
    note(
        &store,
        "note:m",
        "book:a",
        "Monthly",
        "- [ ] Rent @2026-08-05 every month\n",
    );
    note(
        &store,
        "note:once",
        "book:a",
        "Once",
        "- [ ] Dentist @2026-11-10\n- [x] Filed @2026-11-11\n- [ ] Later @2026-12-01\n- [ ] Undated\n",
    );
    let items = run(&store, month(2026, 11));

    let weekly: Vec<_> = items.iter().filter(|i| i.note_id == "note:w").collect();
    assert_eq!(
        weekly.iter().map(|i| i.date).collect::<Vec<_>>(),
        [
            day(2026, 11, 6),
            day(2026, 11, 13),
            day(2026, 11, 20),
            day(2026, 11, 27)
        ]
    );
    assert!(weekly.iter().all(|i| i.time == Some((15, 0))
        && i.repeat.as_deref() == Some("every week")
        && i.kind == ScheduleKind::Task
        && i.line == Some(0)
        && !i.done
        && i.property.is_none()));
    assert!(weekly.iter().all(|i| i.projected));

    assert_eq!(dates_of(&items, "note:m"), [day(2026, 11, 5)]);
    assert_eq!(
        dates_of(&items, "note:once"),
        [day(2026, 11, 10), day(2026, 11, 11)]
    );
    let done = items.iter().find(|i| i.date == day(2026, 11, 11)).unwrap();
    assert!(done.done && done.repeat.is_none() && !done.projected);
    assert_eq!(done.line, Some(1));
}

#[test]
fn month_ends_do_not_spill_over() {
    // "On the 31st" lands on the last day of a short month and does not drift.
    let on_31st = repeat_of("every month on the 31st");
    let due = Due::parse("2026-01-31").unwrap();
    assert_eq!(
        occurrences(&due, &on_31st, &month(2026, 2)),
        [day(2026, 2, 28)]
    );
    assert_eq!(
        occurrences(&due, &on_31st, &month(2026, 4)),
        [day(2026, 4, 30)]
    );
    assert_eq!(
        occurrences(&due, &on_31st, &month(2026, 5)),
        [day(2026, 5, 31)]
    );
    // A leap year keeps 29 February.
    let due = Due::parse("2027-12-31").unwrap();
    assert_eq!(
        occurrences(&due, &on_31st, &month(2028, 2)),
        [day(2028, 2, 29)]
    );
    // The last day of every month.
    let last = repeat_of("every month on the last day");
    let due = Due::parse("2026-01-31").unwrap();
    let range = day(2026, 1, 1)..day(2026, 5, 1);
    assert_eq!(
        occurrences(&due, &last, &range),
        [
            day(2026, 1, 31),
            day(2026, 2, 28),
            day(2026, 3, 31),
            day(2026, 4, 30)
        ]
    );
    // "Every month" keeps the day it is given, shortened in a short month.
    let same = repeat_of("every month");
    assert_eq!(
        occurrences(&due, &same, &range),
        [
            day(2026, 1, 31),
            day(2026, 2, 28),
            day(2026, 3, 28),
            day(2026, 4, 28)
        ]
    );
    // 29 February yearly.
    let yearly = repeat_of("every year");
    let due = Due::parse("2028-02-29").unwrap();
    assert_eq!(
        occurrences(&due, &yearly, &(day(2028, 1, 1)..day(2030, 1, 1))),
        [day(2028, 2, 29), day(2029, 2, 28)]
    );
}

#[test]
fn a_range_is_half_open_and_empty_ranges_are_empty() {
    let weekly = repeat_of("every week");
    let due = Due::parse("2026-11-06").unwrap();
    let range = day(2026, 11, 6)..day(2026, 11, 13);
    assert_eq!(occurrences(&due, &weekly, &range), [day(2026, 11, 6)]);
    assert!(occurrences(&due, &weekly, &(day(2026, 11, 13)..day(2026, 11, 13))).is_empty());
    let (_dir, store) = open();
    assert!(
        schedule(
            &store,
            &ScheduleQuery::new(day(2026, 11, 13)..day(2026, 11, 6))
        )
        .unwrap()
        .is_empty()
    );
}

#[test]
fn notes_sit_on_their_date_properties() {
    let (_dir, store) = open();
    store
        .set_notebook_properties(
            "book:a",
            &[
                def("publish", PropertyType::Date),
                def("shoot", PropertyType::DateTime),
                def("embargo", PropertyType::DateTimeZone),
                def("sponsor", PropertyType::Text),
            ],
            Origin::Ui,
        )
        .unwrap();
    note(
        &store,
        "note:v",
        "book:a",
        "Pixel review",
        "---\npublish: 2026-11-12\nshoot: 2026-11-03T10:30\nembargo: 2026-11-18T09:00 America/New_York\nsponsor: 2026-11-04\n---\nBody\n",
    );
    // A date property of a notebook that does not define it places nothing.
    note(
        &store,
        "note:o",
        "book:b",
        "Elsewhere",
        "---\npublish: 2026-11-12\n---\n",
    );
    // Outside the month, or not a date at all.
    note(
        &store,
        "note:x",
        "book:a",
        "Later",
        "---\npublish: 2026-12-02\nembargo: soon\n---\n",
    );
    let items = run(&store, month(2026, 11));
    let notes: Vec<_> = items
        .iter()
        .filter(|i| i.kind == ScheduleKind::Note)
        .collect();
    assert_eq!(notes.len(), 3, "{notes:#?}");
    assert_eq!(notes[0].property.as_deref(), Some("shoot"));
    assert_eq!(notes[0].time, Some((10, 30)));
    assert_eq!(notes[0].zone, None);
    assert_eq!(notes[1].property.as_deref(), Some("publish"));
    assert_eq!(notes[1].time, None);
    assert_eq!(notes[2].property.as_deref(), Some("embargo"));
    assert_eq!(notes[2].date, day(2026, 11, 18));
    assert_eq!(notes[2].time, Some((9, 0)));
    assert_eq!(notes[2].zone.as_deref(), Some("America/New_York"));
    assert!(notes.iter().all(|i| i.note_id == "note:v"
        && i.note_title == "Pixel review"
        && i.book_id == "book:a"
        && i.line.is_none()
        && !i.done
        && !i.projected));

    // Only the properties asked for.
    let mut query = ScheduleQuery::new(month(2026, 11));
    query.date_keys = Some(vec!["publish".into()]);
    let items = schedule(&store, &query).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].property.as_deref(), Some("publish"));
}

#[test]
fn notebooks_and_done_tasks_filter() {
    let (_dir, store) = open();
    note(
        &store,
        "note:a",
        "book:a",
        "A",
        "- [ ] One @2026-11-10\n- [x] Two @2026-11-10\n",
    );
    note(&store, "note:b", "book:b", "B", "- [ ] Three @2026-11-10\n");
    let mut query = ScheduleQuery::new(month(2026, 11));
    assert_eq!(schedule(&store, &query).unwrap().len(), 3);
    query.include_done = false;
    assert_eq!(schedule(&store, &query).unwrap().len(), 2);
    query.books = Some(vec!["book:b".into()]);
    let items = schedule(&store, &query).unwrap();
    assert_eq!(dates_of(&items, "note:b").len(), 1);
    assert_eq!(items.len(), 1);
    query.books = Some(Vec::new());
    assert!(schedule(&store, &query).unwrap().is_empty());
}

#[test]
fn items_come_in_day_and_time_order() {
    let (_dir, store) = open();
    note(
        &store,
        "note:a",
        "book:a",
        "A",
        "- [ ] Late @2026-11-10T18:00\n- [ ] Early @2026-11-10T08:00\n- [ ] All day @2026-11-10\n- [ ] Before @2026-11-09\n",
    );
    let items = run(&store, month(2026, 11));
    let order: Vec<_> = items.iter().map(|i| (i.date.day, i.time)).collect();
    assert_eq!(
        order,
        [
            (9, None),
            (10, None),
            (10, Some((8, 0))),
            (10, Some((18, 0)))
        ]
    );
}

fn every() -> impl Strategy<Value = Every> {
    let days = (0u8..7).prop_map(|bits| {
        let mut set = WeekdaySet::EMPTY;
        for d in 0..7usize {
            if bits as usize % 7 == d || (bits >> 1) as usize % 7 == d {
                set.insert(d);
            }
        }
        set
    });
    let rule = prop_oneof![
        Just(MonthDay::Same),
        (1u8..=31).prop_map(MonthDay::Date),
        Just(MonthDay::Last),
        (prop_oneof![Just(-1i8), 1i8..=4], 0u8..7).prop_map(|(n, d)| MonthDay::Nth(n, d)),
    ];
    prop_oneof![
        (1u16..40).prop_map(Every::Days),
        (1u16..5, days).prop_map(|(n, set)| Every::Weeks(n, set)),
        (1u16..5, rule).prop_map(|(n, r)| Every::Months(n, r)),
        (1u16..4).prop_map(Every::Years),
        Just(Every::Weekdays),
    ]
}

fn some_date() -> impl Strategy<Value = Date> {
    (0i64..20_000).prop_map(|d| Date::from_days(d + 10_000))
}

/// The repeat as the body writes and reads it back.
fn written(every: Every, due: &Due) -> (String, Repeat) {
    let words = Repeat {
        every,
        after_done: false,
    }
    .to_string();
    let body = format!("- [ ] task @{} {words}\n", due.format());
    let repeat = task_items(&body)[0].repeat.expect("the words read back");
    (body, repeat)
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 256,
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
        ..ProptestConfig::default()
    })]

    #[test]
    fn expansion_never_leaves_the_range(
        every in every(),
        after_done in any::<bool>(),
        start in some_date(),
        due_at in some_date(),
        len in 0i64..120,
        time in proptest::option::of((0u32..24, 0u32..60)),
    ) {
        let due = Due { date: due_at, time };
        let repeat = Repeat { every, after_done };
        let range = start..start.add_days(len);
        let found = occurrences(&due, &repeat, &range);
        for date in &found {
            prop_assert!(*date >= range.start && *date < range.end, "{date:?} outside {range:?}");
        }
        prop_assert!(found.windows(2).all(|w| w[0] < w[1]), "ascending, no repeats");
        // Nothing before the first due date.
        prop_assert!(found.iter().all(|d| *d >= due.date));
        // A wider range sees the same dates inside the narrow one.
        let wide = start.add_days(-40)..start.add_days(len + 40);
        let wider: Vec<_> = occurrences(&due, &repeat, &wide)
            .into_iter()
            .filter(|d| *d >= range.start && *d < range.end)
            .collect();
        prop_assert_eq!(found, wider);
    }

    #[test]
    fn ticking_an_occurrence_moves_to_the_next(
        every in every(),
        due_at in some_date(),
        pick in 0usize..6,
    ) {
        let due = Due { date: due_at, time: Some((15, 0)) };
        let (body, repeat) = written(every, &due);
        let range = due.date..due.date.add_days(3_000);
        let all = occurrences(&due, &repeat, &range);
        prop_assume!(all.len() > pick + 1);
        let ticked_on = all[pick];
        let after = tick_task_at(&body, 0, ticked_on).expect("ticks");
        let items = task_items(&after);
        // The next one is written above, the ticked one stays as a log.
        prop_assert_eq!(items.len(), 2);
        prop_assert!(!items[0].checked && items[1].checked);
        let next = Due::parse(items[0].due.as_deref().unwrap()).unwrap();
        prop_assert_eq!(next.date, all[pick + 1]);
        prop_assert_eq!(next.time, due.time);
        prop_assert!(items[0].repeat.is_some() && items[1].repeat.is_none());
        // And expanding the new task carries on exactly as before.
        let rest = occurrences(&next, &items[0].repeat.unwrap(), &range);
        prop_assert_eq!(&rest[..], &all[pick + 1..]);
    }
}
