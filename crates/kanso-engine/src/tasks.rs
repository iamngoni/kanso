//! Tasks with due dates, without a window (`- [ ] Pay rent @2026-10-25`,
//! `@2026-10-25T09:00`): how a date reads in words and how near it feels
//! ([`Tone`]), the Markdown the preview is handed for each date
//! ([`rewrite_for_preview`]), the tints the editor gives the dates
//! ([`marks`]), `@` autocomplete for a date ([`date_query`],
//! [`date_choices`]) and a task's words as a list shows them
//! ([`display_text`]).
//!
//! The preview is handed each task's due token written as a `kanso-due:`
//! link (as wiki-links become ordinary links), which the app draws as a
//! chip. The chip works out its words when it is drawn, so "Today" stays
//! true. [`reminders`] decides when a timed task notifies.
//!
//! Every function takes the time as a parameter; none reads a clock.

use std::borrow::Cow;
use std::ops::Range;

use kanso_core::dates::{Date, DateTime};
use kanso_core::links::{code_ranges, plain_line};
use kanso_core::tasks::{Due, TaskGroup, due_label, due_spans, due_token, task_group, task_items};

pub mod board;
mod board_model;
pub mod reminders;

/// The scheme of the links due dates become in the preview.
pub const DUE_SCHEME: &str = "kanso-due:";
/// Marks a done task's date in its link.
const DONE_SUFFIX: &str = ";done";

/// How a due date is shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    /// Upcoming, or without anything to say.
    Plain,
    /// Due today.
    Today,
    /// Past, and the task isn't done.
    Overdue,
    /// The task is done.
    Done,
}

/// How a task's date reads at local time `now`. Pure.
pub fn tone(due: Due, checked: bool, now: DateTime) -> Tone {
    match task_group(Some(due), checked, now) {
        TaskGroup::Done => Tone::Done,
        TaskGroup::Overdue => Tone::Overdue,
        TaskGroup::Today => Tone::Today,
        TaskGroup::Upcoming | TaskGroup::NoDate => Tone::Plain,
    }
}

/// A color of the theme a chip is drawn in, by what it is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChipColor {
    Danger,
    Link,
    TextSecondary,
    TextMuted,
    Fill,
}

/// A chip's colors: its text, and its fill at `fill_alpha` (0 to 1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChipTint {
    pub text: ChipColor,
    pub fill: ChipColor,
    pub fill_alpha: f32,
}

/// The colors a chip (and the tint in the editor) takes for `tone`: a warm
/// red once overdue, the link color for today, quiet otherwise. Pure.
pub fn chip_tint(tone: Tone) -> ChipTint {
    let tinted = |color| ChipTint {
        text: color,
        fill: color,
        fill_alpha: 0.16,
    };
    match tone {
        Tone::Overdue => tinted(ChipColor::Danger),
        Tone::Today => tinted(ChipColor::Link),
        Tone::Plain => ChipTint {
            text: ChipColor::TextSecondary,
            fill: ChipColor::Fill,
            fill_alpha: 1.0,
        },
        Tone::Done => ChipTint {
            text: ChipColor::TextMuted,
            fill: ChipColor::Fill,
            fill_alpha: 0.6,
        },
    }
}

/// The chip's words at local time `now`: "Tomorrow", "Fri 25 Oct 09:00",
/// and "Overdue · Yesterday" for a past date on a task not done. Pure.
pub fn chip_label(due: Due, checked: bool, now: DateTime) -> String {
    let label = due_label(due, now.date);
    if tone(due, checked, now) == Tone::Overdue {
        format!("Overdue · {label}")
    } else {
        label
    }
}

/// "Sunday, 25 October 2026", with "at 09:00" for a time: the chip's
/// tooltip.
pub fn long_label(due: Due) -> String {
    let day = due.date.long_label();
    match due.time {
        Some((hour, minute)) => format!("Due {day} at {hour:02}:{minute:02}"),
        None => format!("Due {day}"),
    }
}

/// The note's Markdown with each task's due date, outside code, written as
/// a `kanso-due:` link for the app to draw as a chip. Text without dates
/// comes back as it is (borrowed). Pure.
pub fn rewrite_for_preview(text: &str) -> Cow<'_, str> {
    if !text.contains('@') {
        return Cow::Borrowed(text);
    }
    let spans = due_spans(text);
    if spans.is_empty() {
        return Cow::Borrowed(text);
    }
    let code = code_ranges(text);
    let mut out = String::with_capacity(text.len() + spans.len() * 24);
    let mut last = 0;
    for span in spans {
        if code
            .iter()
            .any(|c| c.start < span.range.end && span.range.start < c.end)
        {
            continue;
        }
        out.push_str(&text[last..span.range.start]);
        let token = &text[span.range.clone()];
        out.push('[');
        out.push_str(token);
        out.push_str("](");
        out.push_str(DUE_SCHEME);
        out.push_str(&token[1..]);
        if span.checked {
            out.push_str(DONE_SUFFIX);
        }
        out.push(')');
        last = span.range.end;
    }
    out.push_str(&text[last..]);
    Cow::Owned(out)
}

/// The due date (and whether its task is done) a `kanso-due:` link holds.
pub fn parse_due_url(url: &str) -> Option<(Due, bool)> {
    let rest = url.strip_prefix(DUE_SCHEME)?;
    let (date, checked) = match rest.strip_suffix(DONE_SUFFIX) {
        Some(date) => (date, true),
        None => (rest, false),
    };
    Some((Due::parse(date)?, checked))
}

/// The tint of each due date in `body` at local time `now`. Pure.
pub fn marks(body: &str, now: DateTime) -> Vec<(Range<usize>, Tone)> {
    due_spans(body)
        .into_iter()
        .map(|span| (span.range, tone(span.due, span.checked, now)))
        .collect()
}

// MARK: Autocomplete

/// What `@` autocomplete is completing in a task, from the text of the
/// caret's line up to the caret: where the `@` is and what was typed after
/// it. `None` outside a task, or once the typing can't be a date. Pure.
pub fn date_query(before_caret: &str) -> Option<(usize, &str)> {
    let at = before_caret.rfind('@')?;
    if at > 0 && !before_caret[..at].ends_with([' ', '\t', '(']) {
        return None;
    }
    let query = &before_caret[at + 1..];
    if query.len() > 16
        || !query
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == ':')
    {
        return None;
    }
    // A task: its box, then some words, before the `@`.
    let line = format!("{}x", &before_caret[..at]);
    (!task_items(&line).is_empty()).then_some((at, query))
}

/// The dates offered for `query` on `today`, as `(label, date)`: today,
/// tomorrow, next week (the coming Monday) and the next weekdays, those
/// whose words or date start with what was typed, then the date typed
/// itself when it is a whole one. Pure.
pub fn date_choices(query: &str, today: Date) -> Vec<(String, String)> {
    let query = query.trim().to_lowercase();
    let mut choices: Vec<(String, Date)> = vec![
        ("Today".into(), today),
        ("Tomorrow".into(), today.add_days(1)),
        (
            "Next week".into(),
            today.add_days(7 - today.weekday() as i64),
        ),
    ];
    for ahead in 2..7 {
        let day = today.add_days(ahead);
        choices.push((kanso_core::dates::WEEKDAYS[day.weekday()].to_owned(), day));
    }
    let mut out: Vec<(String, String)> = choices
        .into_iter()
        .filter(|(label, date)| {
            query.is_empty()
                || label.to_lowercase().starts_with(&query)
                || label
                    .to_lowercase()
                    .split(' ')
                    .any(|word| word.starts_with(&query))
                || date.iso().starts_with(&query)
        })
        .map(|(label, date)| {
            let words = due_label(Due { date, time: None }, today);
            let label = if words == label {
                label
            } else {
                format!("{label} · {words}")
            };
            (label, date.iso())
        })
        .collect();
    if let Some(due) = Due::parse(&query.to_uppercase().replace('t', "T"))
        && !out.iter().any(|(_, iso)| *iso == due.format())
    {
        out.push((due_label(due, today), due.format()));
    }
    out
}

// MARK: Lists

/// A task's words as the list shows them: plain, without its due date
/// (the chip beside it says that). Pure.
pub fn display_text(text: &str) -> String {
    let without = match due_token(text) {
        Some(range) => format!("{}{}", &text[..range.start], &text[range.end..]),
        None => text.to_owned(),
    };
    let plain = plain_line(&without);
    let plain = plain.split_whitespace().collect::<Vec<_>>().join(" ");
    if plain.is_empty() {
        text.trim().to_owned()
    } else {
        plain
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    fn now() -> DateTime {
        DateTime {
            date: Date::new(2026, 10, 20).unwrap(),
            hour: 10,
            minute: 0,
            second: 0,
        }
    }

    #[test]
    fn chips_read_in_words() {
        let due = |s: &str| Due::parse(s).unwrap();
        assert_eq!(chip_label(due("2026-10-21"), false, now()), "Tomorrow");
        assert_eq!(
            chip_label(due("2026-10-23T09:00"), false, now()),
            "Fri 23 Oct 09:00"
        );
        assert_eq!(
            chip_label(due("2026-10-19"), false, now()),
            "Overdue · Yesterday"
        );
        assert_eq!(chip_label(due("2026-10-19"), true, now()), "Yesterday");
        assert_eq!(tone(due("2026-10-20T09:00"), false, now()), Tone::Overdue);
        assert_eq!(tone(due("2026-10-20T11:00"), false, now()), Tone::Today);
        assert_eq!(tone(due("2026-10-19"), true, now()), Tone::Done);
        assert_eq!(
            long_label(due("2026-10-25T09:00")),
            "Due Sunday, 25 October 2026 at 09:00"
        );
    }

    #[test]
    fn chips_are_tinted_by_how_near_they_are() {
        let overdue = chip_tint(Tone::Overdue);
        assert_eq!(
            (overdue.text, overdue.fill),
            (ChipColor::Danger, ChipColor::Danger)
        );
        assert_eq!(chip_tint(Tone::Today).text, ChipColor::Link);
        let plain = chip_tint(Tone::Plain);
        assert_eq!(
            (plain.text, plain.fill_alpha),
            (ChipColor::TextSecondary, 1.0)
        );
        assert_eq!(chip_tint(Tone::Done).text, ChipColor::TextMuted);
    }

    #[test]
    fn the_preview_gets_links_for_due_dates() {
        let text = "- [ ] Pay @2026-10-25 now\n- [x] Done @2026-10-01T09:00\n`- [ ] @2026-10-02`\n```\n- [ ] @2026-10-03\n```\nmail me@2026-10-04\n";
        let out = rewrite_for_preview(text);
        assert_eq!(
            out.as_ref(),
            "- [ ] Pay [@2026-10-25](kanso-due:2026-10-25) now\n- [x] Done [@2026-10-01T09:00](kanso-due:2026-10-01T09:00;done)\n`- [ ] @2026-10-02`\n```\n- [ ] @2026-10-03\n```\nmail me@2026-10-04\n"
        );
        assert!(matches!(
            rewrite_for_preview("no dates here"),
            Cow::Borrowed("no dates here")
        ));
        assert_eq!(
            parse_due_url("kanso-due:2026-10-01T09:00;done"),
            Some((Due::parse("2026-10-01T09:00").unwrap(), true))
        );
        assert_eq!(parse_due_url("kanso-due:soon"), None);
    }

    #[test]
    fn autocomplete_offers_dates_in_tasks() {
        assert_eq!(date_query("- [ ] Pay rent @"), Some((15, "")));
        assert_eq!(date_query("- [ ] Pay rent @tom"), Some((15, "tom")));
        assert_eq!(date_query("Pay rent @tom"), None, "not a task");
        assert_eq!(date_query("- [ ] mail me@x"), None);
        assert_eq!(date_query("- [ ] a @to do"), None);
        let today = now().date; // A Tuesday.
        let all = date_choices("", today);
        assert_eq!(all[0], ("Today".into(), "2026-10-20".into()));
        assert_eq!(all[1], ("Tomorrow".into(), "2026-10-21".into()));
        assert_eq!(
            all[2],
            ("Next week · Mon 26 Oct".into(), "2026-10-26".into())
        );
        assert_eq!(all.len(), 8);
        let fri = date_choices("fr", today);
        assert_eq!(
            fri,
            vec![("Friday · Fri 23 Oct".into(), "2026-10-23".into())]
        );
        let typed = date_choices("2026-12-01t08:30", today);
        assert_eq!(
            typed.last().unwrap(),
            &("Tue 1 Dec 08:30".into(), "2026-12-01T08:30".into())
        );
        assert!(date_choices("zz", today).is_empty());
    }

    #[test]
    fn display_text_drops_the_date() {
        assert_eq!(
            display_text("Pay **rent** @2026-10-25 soon"),
            "Pay rent soon"
        );
        assert_eq!(display_text("@2026-10-25"), "@2026-10-25");
        assert_eq!(display_text("See [doc](https://x.y)"), "See doc");
    }

    fn config() -> ProptestConfig {
        ProptestConfig {
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    proptest! {
        #![proptest_config(config())]

        /// Round trip: every due date the preview gets as a link reads back
        /// from it as the same date, done or not, and the rest of the text
        /// is kept.
        #[test]
        fn prop_due_links_round_trip(
            days in 0i64..40_000,
            time in prop::option::of((0u32..24, 0u32..60)),
            checked: bool,
            words in "[a-z ]{1,10}",
        ) {
            let due = Due { date: Date::from_days(days), time };
            let mark = if checked { 'x' } else { ' ' };
            let text = format!("- [{mark}] {words} @{}\n", due.format());
            let out = rewrite_for_preview(&text);
            let url_start = out.find(DUE_SCHEME).unwrap();
            let url_end = url_start + out[url_start..].find(')').unwrap();
            prop_assert_eq!(parse_due_url(&out[url_start..url_end]), Some((due, checked)));
            let start = format!("- [{mark}] {words} [@{}](", due.format());
            prop_assert!(out.starts_with(&start));
            // The tints cover exactly the token.
            let marks = marks(&text, now());
            prop_assert_eq!(marks.len(), 1);
            prop_assert_eq!(&text[marks[0].0.clone()], format!("@{}", due.format()));
        }

        /// Every choice autocomplete offers is a real date written the way
        /// tasks carry them, and the words it shows start with what was
        /// typed (or its date does).
        #[test]
        fn prop_choices_are_dates(query in "[a-z0-9-]{0,4}", days in 0i64..40_000) {
            let today = Date::from_days(days);
            for (label, iso) in date_choices(&query, today) {
                let due = Due::parse(&iso);
                prop_assert!(due.is_some(), "{}", iso);
                let label = label.to_lowercase();
                prop_assert!(
                    query.is_empty()
                        || label.split([' ', '·']).any(|w| w.starts_with(&query))
                        || iso.starts_with(&query)
                        || Due::parse(&query).is_some(),
                    "{} for {}", label, query
                );
            }
        }

        /// A chip never fades its text or its fill out entirely, and a
        /// task's tone follows its group: done tasks are `Done`, never
        /// overdue.
        #[test]
        fn prop_tints_are_visible_and_done_is_done(
            days in 0i64..40_000,
            time in prop::option::of((0u32..24, 0u32..60)),
            checked: bool,
            hour in 0u32..24,
        ) {
            let due = Due { date: Date::from_days(days), time };
            let at = DateTime { date: Date::from_days(days), hour, minute: 0, second: 0 };
            let tone = tone(due, checked, at);
            prop_assert_eq!(tone == Tone::Done, checked);
            let tint = chip_tint(tone);
            prop_assert!(tint.fill_alpha > 0.0 && tint.fill_alpha <= 1.0);
            prop_assert_eq!(
                chip_label(due, checked, at).starts_with("Overdue · "),
                tone == Tone::Overdue
            );
        }

        /// The list's words never keep the date token and are never empty
        /// for a task with some text.
        #[test]
        fn prop_display_text_has_no_date(
            words in "[a-z]{1,8}( [a-z]{1,8}){0,3}",
            days in 0i64..40_000,
        ) {
            let due = Due { date: Date::from_days(days), time: None };
            let shown = display_text(&format!("{words} @{}", due.format()));
            prop_assert_eq!(shown, words);
        }
    }
}
