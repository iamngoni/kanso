//! Typed dates on the phone: the hint for `film Pixel review fri 3pm every
//! week` and the `@` list, both from the desktop's parser
//! (`kanso_core::due_phrase`), run on the device with the phone's clock.
//!
//! The words of a hint and of a row are the desktop's. Nothing here edits
//! text: the shell rewrites it when the hint or a row is tapped.

use kanso_core::dates::DateTime;
use kanso_core::due_phrase;
use kanso_core::due_phrase_model::DateOrder;
use kanso_core::tasks::{due_token, task_items};
use kanso_engine::tasks::{date_choices, date_query};

use crate::boundary::guard;
use crate::errors::KansoError;
use crate::journal::time_of;
use crate::library::Library;
use crate::model::LocalTime;
use crate::when_model::{DateChoice, DateChoices, WhenHint};

pub use kanso_engine::tasks::REPEAT_CHOICES;

fn units(text: &str, bytes: usize) -> u32 {
    text[..bytes].encode_utf16().count().min(u32::MAX as usize) as u32
}

fn order(month_first: bool) -> DateOrder {
    if month_first {
        DateOrder::MonthDay
    } else {
        DateOrder::DayMonth
    }
}

/// The hint for one line of the note, or `None`: the line is not an open
/// task, has no words besides the date, ends in no date, or already has the
/// canonical token. Pure.
pub fn parse_hint(line: &str, now: DateTime, month_first: bool) -> Option<WhenHint> {
    let item = task_items(line).into_iter().next()?;
    if item.checked {
        return None;
    }
    let at = line.find('[').map_or(line.len(), |at| at + 3);
    let body = line.get(at..)?;
    let parsed = due_phrase::parse(body, now, order(month_first))?;
    // "fri" alone is not a task about a date; the words come first.
    if body[..parsed.span.start].trim().is_empty() {
        return None;
    }
    let canonical = due_phrase::canonical(&parsed);
    if body[parsed.span.clone()] == canonical {
        return None;
    }
    let start = at + parsed.span.start;
    let end = at + parsed.span.end;
    Some(WhenHint {
        start: units(line, start),
        end: units(line, end),
        label: due_phrase::label(&parsed, now.date),
        canonical,
        repeats: parsed.repeat.is_some(),
    })
}

/// The `@` list for the text of the caret's line up to the caret, or `None`
/// when the caret is not after an `@` in a task, nor after a date and a
/// space.
///
/// After an `@`: today, tomorrow, next week and the next weekdays that
/// match what was typed. After a whole date and a space: "Every day", "Every week",
/// "Every weekday" and "Every month", narrowed by what is typed after it.
/// Pure.
pub fn parse_choices(before_caret: &str, now: DateTime) -> Option<DateChoices> {
    if let Some(choices) = repeat_choices(before_caret) {
        return Some(choices);
    }
    let (at, query) = date_query(before_caret)?;
    let choices: Vec<DateChoice> = date_choices(query, now.date)
        .into_iter()
        .map(|(label, iso)| DateChoice {
            label,
            detail: Some(iso.clone()),
            insert: format!("@{iso}"),
        })
        .collect();
    (!choices.is_empty()).then(|| DateChoices {
        start: units(before_caret, at),
        choices,
    })
}

fn repeat_choices(before: &str) -> Option<DateChoices> {
    if task_items(&format!("{before}x")).is_empty() {
        return None;
    }
    let token = due_token(before)?;
    // After the date and a space, and part of the words if any. Not right
    // after the date: a Return typed there is a new line, not a repeat.
    let typed = before[token.end..].strip_prefix(' ')?;
    let from = token.end + 1;
    let typed = typed.to_lowercase();
    let choices: Vec<DateChoice> = REPEAT_CHOICES
        .iter()
        .filter(|(_, insert)| insert.starts_with(&typed) && *insert != typed)
        .map(|(label, insert)| DateChoice {
            label: (*label).to_owned(),
            detail: None,
            insert: (*insert).to_owned(),
        })
        .collect();
    // Words that start no repeat, or a whole one: nothing to offer.
    (!choices.is_empty()).then(|| DateChoices {
        start: units(before, from),
        choices,
    })
}

impl Library {
    /// The hint for `line` (one line of a note) at the phone's local time
    /// `now`; `month_first` reads `11/14` as November 14th. `None` when the
    /// line has nothing to offer.
    pub fn when_parse(
        line: &str,
        now: LocalTime,
        month_first: bool,
    ) -> Result<Option<WhenHint>, KansoError> {
        guard(|| Ok(parse_hint(line, time_of(now)?, month_first)))
    }

    /// The `@` list for `before_caret`, the text of the caret's line up to
    /// the caret, at the phone's local time `now`.
    pub fn date_choices(
        before_caret: &str,
        now: LocalTime,
    ) -> Result<Option<DateChoices>, KansoError> {
        guard(|| Ok(parse_choices(before_caret, time_of(now)?)))
    }
}
