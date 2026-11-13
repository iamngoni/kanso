//! Typed dates on the phone: the hint for an open task and the `@` list.

use kanso_mobile::{DateChoices, Library, LocalTime, WhenHint};
use proptest::prelude::*;

/// Tuesday 20 October 2026, 10:00.
fn now() -> LocalTime {
    LocalTime {
        year: 2026,
        month: 10,
        day: 20,
        hour: 10,
        minute: 0,
        second: 0,
    }
}

fn hint(line: &str) -> Option<WhenHint> {
    Library::when_parse(line, now(), false).unwrap()
}

fn choices(before: &str) -> Option<DateChoices> {
    Library::date_choices(before, now()).unwrap()
}

fn apply(line: &str, hint: &WhenHint) -> String {
    let units: Vec<u16> = line.encode_utf16().collect();
    let mut out = String::from_utf16(&units[..hint.start as usize]).unwrap();
    out.push_str(&hint.canonical);
    out.push_str(&String::from_utf16(&units[hint.end as usize..]).unwrap());
    out
}

#[test]
fn a_typed_phrase_gets_a_hint_with_its_words_and_token() {
    let line = "- [ ] film Pixel review fri 3pm every week";
    let found = hint(line).unwrap();
    assert_eq!(found.label, "Fri 23 Oct, 15:00 \u{b7} every week");
    assert_eq!(found.canonical, "@2026-10-23T15:00 every week");
    assert!(found.repeats);
    assert_eq!(
        apply(line, &found),
        "- [ ] film Pixel review @2026-10-23T15:00 every week"
    );
}

#[test]
fn a_date_with_no_repeat_does_not_repeat() {
    let found = hint("- [ ] call mum tomorrow").unwrap();
    assert_eq!(found.canonical, "@2026-10-21");
    assert!(!found.repeats);
}

#[test]
fn the_phrase_is_counted_in_utf16_units() {
    // The emoji is two UTF-16 units and four bytes.
    let line = "- [ ] \u{1F3AC} edit fri";
    let found = hint(line).unwrap();
    assert_eq!(found.end as usize, line.encode_utf16().count());
    assert_eq!(apply(line, &found), "- [ ] \u{1F3AC} edit @2026-10-23");
}

#[test]
fn only_open_tasks_with_words_before_the_date_get_a_hint() {
    assert!(hint("film Pixel review fri 3pm").is_none());
    assert!(hint("- [x] film Pixel review fri 3pm").is_none());
    assert!(hint("- [ ] fri").is_none());
    assert!(hint("- [ ] call Fri about the fri shoot").is_none());
    assert!(hint("- [ ] film review").is_none());
}

#[test]
fn a_line_that_has_its_token_already_gets_no_hint() {
    assert!(hint("- [ ] film @2026-10-23T15:00 every week").is_none());
    assert!(hint("- [ ] film @2026-10-23").is_none());
}

#[test]
fn the_slash_date_follows_the_order_the_phone_says() {
    let day_first = Library::when_parse("- [ ] pay 14/11", now(), false).unwrap();
    assert_eq!(day_first.unwrap().canonical, "@2026-11-14");
    let month_first = Library::when_parse("- [ ] pay 11/14", now(), true).unwrap();
    assert_eq!(month_first.unwrap().canonical, "@2026-11-14");
    assert!(
        Library::when_parse("- [ ] pay 11/14", now(), false)
            .unwrap()
            .is_none()
    );
}

#[test]
fn a_bad_clock_is_an_error_not_a_panic() {
    let mut bad = now();
    bad.month = 13;
    assert!(Library::when_parse("- [ ] a fri", bad, false).is_err());
    assert!(Library::date_choices("- [ ] a @", bad).is_err());
}

#[test]
fn an_at_in_a_task_lists_the_days() {
    let found = choices("- [ ] ring @").unwrap();
    assert_eq!(found.start, 11);
    assert_eq!(found.choices[0].label, "Today");
    assert_eq!(found.choices[0].insert, "@2026-10-20");
    assert_eq!(found.choices[1].insert, "@2026-10-21");
    assert!(found.choices[1].label.starts_with("Tomorrow"));
    assert_eq!(found.choices[0].detail.as_deref(), Some("2026-10-20"));
}

#[test]
fn what_is_typed_narrows_the_list() {
    let found = choices("- [ ] ring @fr").unwrap();
    assert_eq!(found.choices.len(), 1);
    assert_eq!(found.choices[0].insert, "@2026-10-23");
    assert!(choices("- [ ] ring @zz").is_none());
}

#[test]
fn outside_a_task_there_is_no_list() {
    assert!(choices("ring @").is_none());
    assert!(choices("- [x]").is_none());
    assert!(choices("- [ ] mail me@example").is_none());
}

#[test]
fn a_date_and_a_space_offer_the_repeats() {
    let before = "- [ ] film @2026-10-23T15:00 ";
    let found = choices(before).unwrap();
    let labels: Vec<_> = found.choices.iter().map(|c| c.label.as_str()).collect();
    assert_eq!(
        labels,
        ["Every day", "Every week", "Every weekday", "Every month"]
    );
    assert_eq!(found.choices[1].insert, "every week");
    assert_eq!(found.start as usize, before.len());
    assert!(found.choices.iter().all(|c| c.detail.is_none()));
}

#[test]
fn right_after_a_date_the_list_is_the_date_alone() {
    // A Return here is a new line, never a repeat.
    let found = choices("- [ ] film @2026-10-23").unwrap();
    assert_eq!(found.choices.len(), 1);
    assert_eq!(found.choices[0].insert, "@2026-10-23");
}

#[test]
fn the_repeats_narrow_as_they_are_typed_and_stop_when_whole() {
    let before = "- [ ] film @2026-10-23 ";
    let all = choices(before).unwrap();
    assert_eq!(all.choices.len(), 4);
    assert_eq!(all.choices[0].insert, "every day");
    assert_eq!(all.start as usize, before.len());

    let weeks = choices("- [ ] film @2026-10-23 every wee").unwrap();
    assert_eq!(weeks.choices.len(), 2);
    assert_eq!(weeks.start as usize, before.len());

    // `every week` is whole; `every weekday` is still one key away.
    let whole = choices("- [ ] film @2026-10-23 every week").unwrap();
    assert_eq!(whole.choices.len(), 1);
    assert_eq!(whole.choices[0].label, "Every weekday");
    assert!(choices("- [ ] film @2026-10-23 every day").is_none());
    assert!(choices("- [ ] film @2026-10-23 and then").is_none());
}

#[test]
fn every_row_is_words_the_parser_reads_back() {
    for (_, insert) in kanso_mobile::REPEAT_CHOICES {
        let line = format!("- [ ] film @2026-10-23 {insert}");
        let item = kanso_core::tasks::task_items(&line).remove(0);
        assert_eq!(item.repeat.unwrap().to_string(), insert);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Whatever a line holds, the hint's range is inside it, on character
    /// boundaries, and tapping it writes a line that is no longer hinted.
    #[test]
    fn a_hint_rewrites_once_and_stays_in_the_line(
        words in "[a-z\u{e9}\u{1F3AC} ]{1,20}",
        phrase in prop::sample::select(vec![
            "fri", "fri 3pm", "tomorrow", "today 9:30", "in 3 days",
            "next mon 08:00", "14 nov", "fri 3pm every week",
            "tomorrow every 2 weeks", "noon",
        ]),
        trailing in "[ ]{0,2}",
    ) {
        let line = format!("- [ ] {words} {phrase}{trailing}");
        if let Some(found) = hint(&line) {
            let units: Vec<u16> = line.encode_utf16().collect();
            prop_assert!(found.start <= found.end);
            prop_assert!(found.end as usize <= units.len());
            // Both ends sit on characters: the slices are valid UTF-16.
            prop_assert!(String::from_utf16(&units[..found.start as usize]).is_ok());
            prop_assert!(String::from_utf16(&units[found.end as usize..]).is_ok());
            prop_assert!(found.canonical.starts_with("@20"));
            prop_assert!(!found.label.is_empty());
            let rewritten = apply(&line, &found);
            prop_assert!(hint(&rewritten).is_none());
            // The words before the date are untouched.
            let head = String::from_utf16(&units[..found.start as usize]).unwrap();
            prop_assert!(rewritten.starts_with(&head));
        }
    }

    /// Any text at all is answered without a panic, and a list's range ends
    /// at the caret and starts on a character.
    #[test]
    fn any_text_is_answered_and_a_list_ends_at_the_caret(
        before in "(- \\[ \\] )?[ -~\u{e9}\u{1F3AC}@]{0,30}",
    ) {
        let _ = Library::when_parse(&before, now(), false).unwrap();
        if let Some(found) = choices(&before) {
            let units: Vec<u16> = before.encode_utf16().collect();
            prop_assert!(found.start as usize <= units.len());
            prop_assert!(String::from_utf16(&units[..found.start as usize]).is_ok());
            prop_assert!(!found.choices.is_empty());
            prop_assert!(found.choices.iter().all(|c| !c.insert.is_empty()));
        }
    }

    /// Every day row inserts a date that is a real one.
    #[test]
    fn the_day_rows_write_real_dates(typed in "[a-z0-9:-]{0,8}") {
        let before = format!("- [ ] x @{typed}");
        if let Some(found) = choices(&before) {
            for row in found.choices {
                prop_assert!(row.insert.starts_with('@') || row.insert.starts_with(' ') || row.insert.starts_with("every"));
                if let Some(day) = row.detail {
                    prop_assert!(kanso_core::tasks::Due::parse(&day).is_some());
                }
            }
        }
    }
}
