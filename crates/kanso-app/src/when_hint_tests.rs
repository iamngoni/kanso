//! The typed-date hint without a window: when a line gets one, what Tab
//! writes, and the properties of writing it.

use kanso_core::dates::{Date, DateTime};
use kanso_core::due_phrase::parse;
use kanso_core::due_phrase_model::DateOrder;
use proptest::prelude::*;
use proptest::test_runner::{Config, RngAlgorithm, RngSeed};

use super::{WhenHint, edit, hint_at, order_for_locale};
use kanso_engine::edit::Edit;

/// Tuesday 10 November 2026, 09:00.
fn now() -> DateTime {
    DateTime {
        date: Date::new(2026, 11, 10).unwrap(),
        hour: 9,
        minute: 0,
        second: 0,
    }
}

fn hint(text: &str) -> Option<WhenHint> {
    hint_at(text, text.len(), now(), DateOrder::DayMonth)
}

#[test]
fn a_phrase_at_the_end_of_a_task_gets_a_hint() {
    let text = "- [ ] film Pixel review fri 3pm every week";
    let hint = hint(text).expect("a hint");
    assert_eq!(&text[hint.phrase.clone()], "fri 3pm every week");
    assert_eq!(hint.canonical, "@2026-11-13T15:00 every week");
    assert_eq!(hint.label, "Fri 13 Nov, 15:00 \u{b7} every week");
    assert_eq!(hint.line, 0..text.len());
    let edit = edit(&hint);
    assert_eq!(
        edit.apply(text),
        "- [ ] film Pixel review @2026-11-13T15:00 every week"
    );
    // The caret goes after what was written.
    assert_eq!(edit.selection, text.len() - 18 + 28..text.len() - 18 + 28);
}

#[test]
fn the_hint_names_the_year_when_it_is_not_this_one() {
    let hint = hint("- [ ] renew 2027-02-03").unwrap();
    assert_eq!(hint.label, "Wed 3 Feb 2027");
    assert_eq!(hint.canonical, "@2027-02-03");
}

#[test]
fn prose_and_other_lines_get_none() {
    // Not a task, not at the end, no date word, no repeat on its own.
    assert_eq!(hint("film Pixel review fri 3pm"), None);
    assert_eq!(hint("- film Pixel review fri"), None);
    assert_eq!(hint("- [ ] call fri about the fri shoot"), None);
    assert_eq!(hint("- [ ] film every week"), None);
    assert_eq!(hint("- [ ] film"), None);
    assert_eq!(hint(""), None);
    // Already written the stored way.
    assert_eq!(hint("- [ ] film @2026-11-13T15:00 every week"), None);
    assert_eq!(hint("- [ ] film @2026-11-13"), None);
    // A line with a date has no second one.
    assert_eq!(hint("- [ ] film @2026-11-13 fri"), None);
    // Code is not a task.
    assert_eq!(
        hint_at("```\n- [ ] x fri\n```", 12, now(), DateOrder::DayMonth),
        None
    );
}

#[test]
fn the_hint_is_for_the_caret_line() {
    let text = "- [ ] a fri\n- [ ] b sat\nplain";
    let first = hint_at(text, 3, now(), DateOrder::DayMonth).unwrap();
    assert_eq!(&text[first.phrase], "fri");
    let second = hint_at(text, 20, now(), DateOrder::DayMonth).unwrap();
    assert_eq!(&text[second.phrase], "sat");
    assert_eq!(hint_at(text, text.len(), now(), DateOrder::DayMonth), None);
}

#[test]
fn slash_dates_read_in_the_locale_order() {
    let day_first = hint_at("- [ ] x 3/4", 11, now(), DateOrder::DayMonth).unwrap();
    assert_eq!(day_first.canonical, "@2027-04-03");
    let month_first = hint_at("- [ ] x 3/4", 11, now(), DateOrder::MonthDay).unwrap();
    assert_eq!(month_first.canonical, "@2027-03-04");
}

#[test]
fn locales_name_their_order() {
    assert_eq!(order_for_locale("en_US"), DateOrder::MonthDay);
    assert_eq!(order_for_locale("en-US.UTF-8"), DateOrder::MonthDay);
    assert_eq!(order_for_locale("fil_PH@x"), DateOrder::MonthDay);
    assert_eq!(order_for_locale("en_GB.UTF-8"), DateOrder::DayMonth);
    assert_eq!(order_for_locale("de-DE"), DateOrder::DayMonth);
    assert_eq!(order_for_locale("C"), DateOrder::DayMonth);
    assert_eq!(order_for_locale(""), DateOrder::DayMonth);
}

fn day() -> impl Strategy<Value = &'static str> {
    prop_oneof![
        Just("today"),
        Just("tomorrow"),
        Just("fri"),
        Just("next mon"),
        Just("in 3 days"),
        Just("2026-12-01"),
        Just("14 nov"),
        Just("tonight"),
    ]
}

fn clock() -> impl Strategy<Value = &'static str> {
    prop_oneof![
        Just(""),
        Just(" 3pm"),
        Just(" at 15:30"),
        Just(" noon"),
        Just(" 9:05am"),
    ]
}

fn repeat() -> impl Strategy<Value = &'static str> {
    prop_oneof![
        Just(""),
        Just(" every week"),
        Just(" every 2 weeks on mon and thu"),
        Just(" weekly"),
        Just(" every weekday"),
        Just(" every month on the last day"),
        Just(" every 3 days after done"),
    ]
}

proptest! {
    #![proptest_config(Config {
        cases: 512,
        rng_seed: RngSeed::Fixed(0x0077_6865_6e68),
        rng_algorithm: RngAlgorithm::ChaCha,
        failure_persistence: None,
        ..Config::default()
    })]

    /// Applying the hint and undoing it is the identity, and the text it
    /// writes parses to the same date and repeat the phrase did, with
    /// nothing left to hint at.
    #[test]
    fn applying_then_undoing_is_identity_and_reparses_the_same(
        (indent, lead, day, clock, repeat, tail, monthly) in (
            "[ ]{0,3}",
            "[a-z]{1,8}( [a-z]{1,8}){0,2}",
            day(),
            clock(),
            repeat(),
            "(\n[a-z ]{0,12}){0,2}",
            any::<bool>(),
        ),
    ) {
        let order = if monthly { DateOrder::MonthDay } else { DateOrder::DayMonth };
        let line = format!("{indent}- [ ] {lead} {day}{clock}{repeat}");
        let text = format!("{line}{tail}");
        let Some(hint) = hint_at(&text, line.len(), now(), order) else {
            // Some lead words are date words themselves ("in", "at") and
            // make no phrase of the whole; there is then nothing to apply.
            prop_assume!(false);
            unreachable!();
        };
        let before = parse(&line, now(), order).expect("the hint came from a parse");
        let edit = edit(&hint);
        let applied = edit.apply(&text);

        // One replacement of the phrase; the rest is untouched.
        prop_assert_eq!(&applied[..hint.phrase.start], &text[..hint.phrase.start]);
        prop_assert_eq!(&applied[hint.phrase.start + hint.canonical.len()..], &text[hint.phrase.end..]);
        prop_assert_eq!(&edit.selection, &(hint.phrase.start + hint.canonical.len()..hint.phrase.start + hint.canonical.len()));

        // Undo: the phrase put back where the stored form is.
        let undo = Edit {
            range: hint.phrase.start..hint.phrase.start + hint.canonical.len(),
            text: text[hint.phrase.clone()].to_owned(),
            selection: hint.phrase.end..hint.phrase.end,
        };
        prop_assert_eq!(undo.apply(&applied), text.clone());

        // The stored form means the same.
        let applied_line = applied.lines().next().unwrap();
        let after = parse(applied_line, now(), order).expect("the stored form parses");
        prop_assert_eq!(after.due, before.due);
        prop_assert_eq!(after.repeat, before.repeat);
        prop_assert_eq!(&applied_line[after.span], hint.canonical.as_str());
        prop_assert_eq!(hint_at(&applied, applied_line.len(), now(), order), None);
    }
}
