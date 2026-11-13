//! Moving something dated to another day, keeping the rest of what it
//! says: what the Schedule does when a note or a task is dragged onto a day.
//!
//! A date property keeps its time and its zone, and a task keeps its time
//! and its repeat words. Only the day changes, and every other character is
//! left as written, so moving a value to its own day gives it back exactly.

use crate::dates::Date;
use crate::properties::Value;
use crate::tasks::{TaskEdit, due_token, task_items};
use crate::zoned;

/// `value` on day `day`: a date becomes the day; a date-time and a date-time
/// with a zone keep their time of day, their seconds if they have any, and
/// their zone. `None` for a value that is not a date of one of the three
/// kinds, or whose text is not well formed.
pub fn move_value(value: &Value, day: Date) -> Option<Value> {
    match value {
        Value::Date(text) => {
            Date::parse_iso(text)?;
            Some(Value::Date(day.iso()))
        }
        Value::DateTime(text) => {
            zoned::parse_local(text)?;
            Some(Value::DateTime(with_day(text, day)))
        }
        Value::DateTimeZone(text) => {
            zoned::parse_zoned(text)?;
            Some(Value::DateTimeZone(with_day(text, day)))
        }
        _ => None,
    }
}

/// `text` with its first ten characters (`YYYY-MM-DD`) replaced by `day`.
fn with_day(text: &str, day: Date) -> String {
    format!("{}{}", day.iso(), &text[10..])
}

/// A task's words on day `day`: the date of its first `@YYYY-MM-DD` (or
/// `@YYYY-MM-DDTHH:MM`) is the new one, and the time, the repeat and every
/// other word stay. `None` when the words have no due date.
pub fn move_due(text: &str, day: Date) -> Option<String> {
    let token = due_token(text)?;
    let at = token.start + 1;
    Some(format!("{}{}{}", &text[..at], day.iso(), &text[at + 10..]))
}

/// The body with the due date of the task `text` (looked for on `line`,
/// 0-based, then on any one other line that has it, as
/// [`crate::tasks::set_task_checked`] does) moved to `day`.
/// [`TaskEdit::Unchanged`] when it is on that day already, and
/// [`TaskEdit::Missing`] when the task is not there or has no due date.
pub fn set_task_due(body: &str, line: u32, text: &str, day: Date) -> TaskEdit {
    let items = task_items(body);
    let target = items
        .iter()
        .find(|t| t.line == line && t.text == text)
        .or_else(|| {
            let mut same = items.iter().filter(|t| t.text == text);
            match (same.next(), same.next()) {
                (Some(item), None) => Some(item),
                _ => None,
            }
        });
    let Some(item) = target else {
        return TaskEdit::Missing;
    };
    let Some(moved) = move_due(&item.text, day) else {
        return TaskEdit::Missing;
    };
    if moved == item.text {
        return TaskEdit::Unchanged;
    }
    let mut out = String::with_capacity(body.len());
    for (ix, raw) in body.split_inclusive('\n').enumerate() {
        if ix as u32 != item.line {
            out.push_str(raw);
            continue;
        }
        let bare = raw.trim_end_matches(['\n', '\r']);
        let eol = &raw[bare.len()..];
        // `task_items` reads the text from three bytes after the bracket.
        let Some(text_at) = bare.find('[').map(|at| at + 3) else {
            return TaskEdit::Missing;
        };
        let Some(tail) = bare.get(text_at..) else {
            return TaskEdit::Missing;
        };
        let Some(moved_tail) = move_due(tail, day) else {
            return TaskEdit::Missing;
        };
        out.push_str(&bare[..text_at]);
        out.push_str(&moved_tail);
        out.push_str(eol);
    }
    TaskEdit::Changed(out)
}

#[cfg(test)]
#[path = "reschedule_tests.rs"]
mod tests;
