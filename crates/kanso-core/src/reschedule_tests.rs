use proptest::prelude::*;
use proptest::test_runner::{Config, RngAlgorithm, RngSeed};

use super::*;
use crate::tasks::{Due, due_date};

fn d(year: i32, month: u32, day: u32) -> Date {
    Date::new(year, month, day).unwrap()
}

#[test]
fn a_date_becomes_the_day() {
    let moved = move_value(&Value::Date("2026-11-18".into()), d(2026, 11, 21));
    assert_eq!(moved, Some(Value::Date("2026-11-21".into())));
}

#[test]
fn a_date_time_keeps_its_time() {
    let moved = move_value(&Value::DateTime("2026-11-18T09:30".into()), d(2026, 12, 1));
    assert_eq!(moved, Some(Value::DateTime("2026-12-01T09:30".into())));
}

#[test]
fn a_date_time_with_a_zone_keeps_its_time_and_its_zone() {
    let moved = move_value(
        &Value::DateTimeZone("2026-11-18T09:00 America/New_York".into()),
        d(2026, 11, 20),
    );
    assert_eq!(
        moved,
        Some(Value::DateTimeZone(
            "2026-11-20T09:00 America/New_York".into()
        ))
    );
}

#[test]
fn seconds_are_kept_as_written() {
    let moved = move_value(
        &Value::DateTimeZone("2026-11-18T09:00:30 Europe/London".into()),
        d(2026, 11, 19),
    );
    assert_eq!(
        moved,
        Some(Value::DateTimeZone(
            "2026-11-19T09:00:30 Europe/London".into()
        ))
    );
}

#[test]
fn values_that_are_not_dates_are_not_moved() {
    for value in [
        Value::Text("2026-11-18".into()),
        Value::Number(3.),
        Value::Bool(true),
        Value::Raw("{a: 1}".into()),
        Value::Date("not a date".into()),
        Value::DateTimeZone("2026-11-18T09:00 Nowhere/Land".into()),
    ] {
        assert_eq!(move_value(&value, d(2026, 11, 21)), None, "{value:?}");
    }
}

#[test]
fn a_tasks_day_changes_and_its_time_and_repeat_stay() {
    assert_eq!(
        move_due("film @2026-11-14T15:00 every week #shoot", d(2026, 11, 18)).as_deref(),
        Some("film @2026-11-18T15:00 every week #shoot")
    );
    assert_eq!(move_due("no date here", d(2026, 11, 18)), None);
}

#[test]
fn a_task_in_a_body_moves_and_the_rest_is_untouched() {
    let body = "# Shoots\r\n- [ ] film @2026-11-14T15:00 every week\r\n- [ ] other @2026-11-14\n";
    let TaskEdit::Changed(after) = set_task_due(
        body,
        1,
        "film @2026-11-14T15:00 every week",
        d(2026, 11, 17),
    ) else {
        panic!("expected a change");
    };
    assert_eq!(
        after,
        "# Shoots\r\n- [ ] film @2026-11-17T15:00 every week\r\n- [ ] other @2026-11-14\n"
    );
}

#[test]
fn a_task_that_moved_lines_is_found_by_its_words() {
    let body = "intro\n\n- [ ] pay @2026-11-14\n";
    assert_eq!(
        set_task_due(body, 0, "pay @2026-11-14", d(2026, 11, 20)),
        TaskEdit::Changed("intro\n\n- [ ] pay @2026-11-20\n".into())
    );
}

#[test]
fn a_task_on_that_day_already_is_unchanged_and_a_missing_one_is_missing() {
    let body = "- [ ] pay @2026-11-14\n- [ ] undated\n";
    assert_eq!(
        set_task_due(body, 0, "pay @2026-11-14", d(2026, 11, 14)),
        TaskEdit::Unchanged
    );
    assert_eq!(
        set_task_due(body, 0, "gone @2026-11-14", d(2026, 11, 15)),
        TaskEdit::Missing
    );
    assert_eq!(
        set_task_due(body, 1, "undated", d(2026, 11, 15)),
        TaskEdit::Missing
    );
}

// MARK: Properties

fn config() -> Config {
    Config {
        cases: 512,
        rng_seed: RngSeed::Fixed(0x0072_6573_6368),
        rng_algorithm: RngAlgorithm::ChaCha,
        failure_persistence: None,
        ..Config::default()
    }
}

fn gen_day() -> impl Strategy<Value = Date> {
    (-20_000i64..60_000).prop_map(Date::from_days)
}

fn gen_zone() -> impl Strategy<Value = &'static str> {
    prop_oneof![
        Just("UTC"),
        Just("America/New_York"),
        Just("Europe/London"),
        Just("Asia/Kolkata"),
        Just("Pacific/Auckland"),
        Just("Africa/Harare"),
    ]
}

fn gen_time() -> impl Strategy<Value = (u32, u32)> {
    (0u32..24, 0u32..60)
}

fn gen_seconds() -> impl Strategy<Value = Option<u32>> {
    prop_oneof![Just(None), (1u32..60).prop_map(Some)]
}

/// A date-valued property, of any of the three kinds, as text.
fn gen_value() -> impl Strategy<Value = Value> {
    (gen_day(), gen_time(), gen_seconds(), gen_zone(), 0usize..3).prop_map(
        |(day, (hour, minute), seconds, zone, kind)| {
            let time = match seconds {
                Some(s) => format!("{hour:02}:{minute:02}:{s:02}"),
                None => format!("{hour:02}:{minute:02}"),
            };
            match kind {
                0 => Value::Date(day.iso()),
                1 => Value::DateTime(format!("{}T{time}", day.iso())),
                _ => Value::DateTimeZone(format!("{}T{time} {zone}", day.iso())),
            }
        },
    )
}

/// What is left of a date value once its day is taken away.
fn rest_of(value: &Value) -> (&'static str, String) {
    match value {
        Value::Date(_) => ("date", String::new()),
        Value::DateTime(text) => ("date-time", text[10..].to_owned()),
        Value::DateTimeZone(text) => ("zoned", text[10..].to_owned()),
        _ => unreachable!("only dates are generated"),
    }
}

fn day_of(value: &Value) -> Date {
    match value {
        Value::Date(text) | Value::DateTime(text) | Value::DateTimeZone(text) => {
            Date::parse_iso(&text[..10]).unwrap()
        }
        _ => unreachable!("only dates are generated"),
    }
}

fn gen_task_text() -> impl Strategy<Value = (String, Option<(u32, u32)>)> {
    (
        "[a-z ]{0,12}",
        proptest::option::of(gen_time()),
        gen_day(),
        prop_oneof![
            Just(""),
            Just(" every week"),
            Just(" every 2 weeks on mon and thu"),
            Just(" every month"),
        ],
        prop_oneof![Just(""), Just(" #shoot"), Just(" later")],
    )
        .prop_map(|(words, time, day, repeat, tail)| {
            let due = Due { date: day, time };
            (format!("t {words} @{}{repeat}{tail}", due.format()), time)
        })
}

proptest! {
    #![proptest_config(config())]

    /// The value is on the new day and nothing else about it changed: the
    /// same kind, the same time (seconds too) and the same zone.
    #[test]
    fn prop_a_moved_value_has_the_new_day_and_all_else_as_written(
        value in gen_value(),
        day in gen_day(),
    ) {
        let moved = move_value(&value, day).unwrap();
        prop_assert_eq!(day_of(&moved), day);
        prop_assert_eq!(rest_of(&moved), rest_of(&value));
    }

    /// A moved value is still a well formed date of its kind, so the
    /// Schedule reads it back where it was put.
    #[test]
    fn prop_a_moved_value_still_parses(value in gen_value(), day in gen_day()) {
        let moved = move_value(&value, day).unwrap();
        match &moved {
            Value::Date(text) => prop_assert_eq!(Date::parse_iso(text), Some(day)),
            Value::DateTime(text) => {
                prop_assert_eq!(zoned::parse_local(text).map(|at| at.date), Some(day));
            }
            Value::DateTimeZone(text) => {
                prop_assert_eq!(zoned::parse_zoned(text).map(|at| at.local.date), Some(day));
            }
            _ => prop_assert!(false, "the kind changed"),
        }
    }

    /// Moving to the day it is on is a no-op, and moving away and back
    /// gives the original text.
    #[test]
    fn prop_moving_away_and_back_is_the_identity(
        value in gen_value(),
        away in gen_day(),
    ) {
        prop_assert_eq!(move_value(&value, day_of(&value)), Some(value.clone()));
        let there = move_value(&value, away).unwrap();
        prop_assert_eq!(move_value(&there, day_of(&value)), Some(value));
    }

    /// A task moves to the new day and keeps its time, its repeat and
    /// every other word.
    #[test]
    fn prop_a_moved_task_keeps_its_time_and_words(
        (text, time) in gen_task_text(),
        day in gen_day(),
    ) {
        let moved = move_due(&text, day).unwrap();
        let due = Due::parse(&due_date(&moved).unwrap()).unwrap();
        prop_assert_eq!(due.date, day);
        prop_assert_eq!(due.time, time);
        // Only the ten characters of the day differ.
        let old_at = text.find('@').unwrap() + 1;
        prop_assert_eq!(&moved[..old_at], &text[..old_at]);
        prop_assert_eq!(&moved[old_at + 10..], &text[old_at + 10..]);
        prop_assert_eq!(crate::repeat::repeat_token(&moved).is_some(), crate::repeat::repeat_token(&text).is_some());
    }

    /// In a body, only the task's own line changes, and it is the one
    /// `move_due` makes.
    #[test]
    fn prop_a_task_moves_in_its_body_and_no_other_line_changes(
        (text, _) in gen_task_text(),
        day in gen_day(),
        before in "(# [a-z]{1,6}\n)?",
        after in "(- \\[ \\] [a-z]{1,6}\n)?",
    ) {
        let body = format!("{before}- [ ] {text}\n{after}");
        let line = before.matches('\n').count() as u32;
        let old = Due::parse(&due_date(&text).unwrap()).unwrap().date;
        match set_task_due(&body, line, &text, day) {
            TaskEdit::Changed(out) => {
                prop_assert!(old != day);
                let expected = format!("{before}- [ ] {}\n{after}", move_due(&text, day).unwrap());
                prop_assert_eq!(out, expected);
            }
            TaskEdit::Unchanged => prop_assert_eq!(old, day),
            TaskEdit::Missing => prop_assert!(false, "the task is there"),
        }
    }
}
