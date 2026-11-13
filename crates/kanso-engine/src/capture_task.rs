//! Task capture: a task typed into Quick Capture's Task mode, written as a
//! task line into today's daily note or into the Inbox's task note.
//!
//! [`task_line`] turns the typed text into `- [ ] text @due every …`, with a
//! trailing date and repeat phrase read by [`kanso_core::due_phrase`].
//! [`append_task`] puts a line at the end of a note body. [`save_task`]
//! does both and saves, to the place [`TaskTarget`] names.
//!
//! Like [`crate::capture`], nothing is written into a locked notebook or a
//! locked note ([`CaptureError::Locked`]).

use kanso_core::DocType;
use kanso_core::dates::DateTime;
use kanso_core::due_phrase;
use kanso_core::due_phrase_model::{DateOrder, Parsed};
use kanso_core::model::MAX_BODY_LEN;
use kanso_core::roles::Role;
use kanso_store::{Origin, Put, Store, StoreError};
use serde_json::json;

use crate::capture::CaptureError;

/// The note in the Inbox that captured tasks collect in when the Inbox is
/// where they go (a fixed ID); the local API's Inbox tasks land there too.
pub use kanso_store::{INBOX_TASKS_ID, INBOX_TASKS_TITLE};

/// Where a captured task is written.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub enum TaskTarget {
    /// The end of today's daily note, made when there is none.
    #[default]
    DailyNote,
    /// The end of the Inbox's task note ([`INBOX_TASKS_ID`]).
    Inbox,
}

/// What a capture becomes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CaptureKind {
    /// A note of its own ([`crate::capture::save_capture`]).
    #[default]
    Note,
    /// A task line, written to `target`.
    Task { target: TaskTarget },
}

/// `text` without a leading task marker: `[]`, `[ ]`, `- [ ]` (or `*`/`+`
/// for the dash) and `todo`, with an optional colon after it.
fn without_task_marker(text: &str) -> &str {
    let text = text.trim_start();
    let text = text
        .strip_prefix(['-', '*', '+'])
        .map_or(text, str::trim_start);
    for marker in ["[ ]", "[]"] {
        if let Some(rest) = text.strip_prefix(marker) {
            return rest;
        }
    }
    if text.len() >= 4
        && text.is_char_boundary(4)
        && text[..4].eq_ignore_ascii_case("todo")
        && (text[4..].is_empty() || text[4..].starts_with(|c: char| c.is_whitespace() || c == ':'))
    {
        return text[4..].trim_start_matches(':');
    }
    text
}

/// The task line for typed `text`: `- [ ] words @2026-11-14T15:00 every
/// week`, or `None` when there is no task in it. The text is one task, so
/// its lines are joined with spaces. A date and repeat phrase at its end
/// become the tokens a task carries; a phrase with nothing before it stays
/// as the task's words. Pure.
pub fn task_line(text: &str, now: DateTime, order: DateOrder) -> Option<String> {
    let (words, parsed) = read_task(text, now, order)?;
    Some(match parsed {
        Some(parsed) => format!("- [ ] {words} {}", due_phrase::canonical(&parsed)),
        None => format!("- [ ] {words}"),
    })
}

/// What the line under the field says in Task mode: how the date and
/// repeat at the end of `text` read, such as `Fri 13 Nov, 15:00 · every
/// week`, or `None` when there is none to show. Pure.
pub fn task_hint(text: &str, now: DateTime, order: DateOrder) -> Option<String> {
    let (_, parsed) = read_task(text, now, order)?;
    parsed.map(|parsed| due_phrase::label(&parsed, now.date))
}

/// A typed task as it will be written and as an App Intent confirms it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskPreview {
    /// The line written: `- [ ] film Pixel review @2026-11-14T15:00 every
    /// week`.
    pub line: String,
    /// The task's words: `film Pixel review`.
    pub words: String,
    /// When it is due and how it repeats, as a person reads it: `Fri 14 Nov
    /// 15:00 · every week`. `None` when the text named no date.
    pub when: Option<String>,
}

impl TaskPreview {
    /// The sentence that says the task was added: `Added: film Pixel review
    /// · Fri 14 Nov 15:00 · every week`.
    pub fn confirmation(&self) -> String {
        match &self.when {
            Some(when) => format!("Added: {} \u{b7} {when}", self.words),
            None => format!("Added: {}", self.words),
        }
    }
}

/// The line, the words and the due date of the task in `text`, or `None`
/// when there is no task in it. The line is [`task_line`]'s. Pure.
pub fn task_preview(text: &str, now: DateTime, order: DateOrder) -> Option<TaskPreview> {
    let (words, parsed) = read_task(text, now, order)?;
    Some(match parsed {
        Some(parsed) => {
            let label = due_phrase::label(&parsed, now.date);
            // The confirmation reads `Fri 14 Nov 15:00`: no comma before
            // the time. The date part never holds one.
            let when = if parsed.due.time.is_some() {
                label.replacen(", ", " ", 1)
            } else {
                label
            };
            TaskPreview {
                line: format!("- [ ] {words} {}", due_phrase::canonical(&parsed)),
                words,
                when: Some(when),
            }
        }
        None => TaskPreview {
            line: format!("- [ ] {words}"),
            words,
            when: None,
        },
    })
}

/// The longest task line [`checked_task_line`] takes, in bytes.
pub const MAX_TASK_LINE: usize = 4096;

/// `line` when it is one open task line as [`task_line`] writes them: a
/// single line starting `- [ ] ` with words after it, at most
/// [`MAX_TASK_LINE`] bytes. Surrounding whitespace is dropped. What another
/// process hands over (an App Intent's drop) is checked with this before it
/// is written. Pure.
pub fn checked_task_line(line: &str) -> Option<&str> {
    let line = line.trim();
    if line.len() > MAX_TASK_LINE || line.contains(['\n', '\r']) {
        return None;
    }
    let words = line.strip_prefix("- [ ] ")?;
    (!words.trim().is_empty()).then_some(line)
}

/// The words of the task in `text` and the phrase that ends it, when it
/// has one and words come before it.
fn read_task(text: &str, now: DateTime, order: DateOrder) -> Option<(String, Option<Parsed>)> {
    let joined = without_task_marker(text)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if joined.is_empty() {
        return None;
    }
    if let Some(parsed) = due_phrase::parse(&joined, now, order) {
        let words = joined[..parsed.span.start].trim_end();
        if !words.is_empty() {
            return Some((words.to_owned(), Some(parsed)));
        }
    }
    Some((joined, None))
}

/// Whether `line` is a task-list item (open or done).
fn is_task_line(line: &str) -> bool {
    line.trim_start()
        .strip_prefix(['-', '*', '+'])
        .map(str::trim_start)
        .is_some_and(|rest| {
            ["[ ]", "[x]", "[X]"]
                .iter()
                .any(|marker| rest.starts_with(marker))
        })
}

/// `body` with the task `line` added at its end, keeping what is there. A
/// body whose last line is a task gets the line right under it, so tasks
/// stay one list; anything else gets a blank line first. Pure.
pub fn append_task(body: &str, line: &str) -> String {
    let kept = body.trim_end();
    let last = kept.lines().next_back().unwrap_or("");
    if kept.is_empty() {
        format!("{line}\n")
    } else if is_task_line(last) {
        format!("{kept}\n{line}\n")
    } else {
        format!("{kept}\n\n{line}\n")
    }
}

/// Saves `text` as a task at `target`. Returns the ID of the note it went
/// into, or `None` when there was no task in the text. `now` is the local
/// date and time, for today's note and for what a phrase means.
///
/// A task that would make the note longer than a note can be is refused as
/// invalid.
pub fn save_task(
    store: &Store,
    text: &str,
    target: TaskTarget,
    now: DateTime,
    order: DateOrder,
    origin: Origin,
) -> Result<Option<String>, CaptureError> {
    let Some(line) = task_line(text, now, order) else {
        return Ok(None);
    };
    save_task_line(store, &line, target, now, origin).map(Some)
}

/// Writes a task `line` that was read already ([`task_line`]'s output, as
/// an App Intent hands it over in a drop) at `target`, and returns the ID
/// of the note it went into. `now` is the local date and time the task was
/// captured at, for which daily note it goes in.
///
/// A line [`checked_task_line`] refuses, or one that would make the note
/// longer than a note can be, is refused as invalid; a locked Journal,
/// Inbox or note is [`CaptureError::Locked`]. Nothing is written then.
pub fn save_task_line(
    store: &Store,
    line: &str,
    target: TaskTarget,
    now: DateTime,
    origin: Origin,
) -> Result<String, CaptureError> {
    let Some(line) = checked_task_line(line) else {
        return Err(CaptureError::Store(StoreError::Invalid(
            "That isn't a task line.".into(),
        )));
    };
    let id = match target {
        TaskTarget::DailyNote => {
            // Made in the Journal; a locked Journal takes nothing.
            if let Some(journal) = store.find_role_book(Role::Journal)?
                && store.locked_books()?.contains(&journal)
            {
                return Err(CaptureError::Locked);
            }
            store.open_daily(now, origin)?.id
        }
        TaskTarget::Inbox => inbox_tasks_note(store, origin)?,
    };
    if store.note_is_locked(&id)? {
        return Err(CaptureError::Locked);
    }
    let Some(note) = store.note(&id)? else {
        return Err(CaptureError::Store(StoreError::Invalid(
            "The note for this task has gone.".into(),
        )));
    };
    let body = append_task(&note.body, line);
    if body.len() > MAX_BODY_LEN {
        return Err(CaptureError::Store(StoreError::Invalid(
            "That note is too long to add a task to.".into(),
        )));
    }
    store.put(
        DocType::Note,
        json!({"_id": id, "body": body}),
        Put::Merge,
        origin,
    )?;
    Ok(id)
}

/// The Inbox's task note ([`Store::inbox_tasks_note`], the same note the
/// local API adds to). Refuses a locked Inbox.
fn inbox_tasks_note(store: &Store, origin: Origin) -> Result<String, CaptureError> {
    let inbox = store.role_book(Role::Inbox, origin)?;
    if store.locked_books()?.contains(&inbox) {
        return Err(CaptureError::Locked);
    }
    Ok(store.inbox_tasks_note(origin)?)
}

#[cfg(test)]
#[path = "capture_task_tests.rs"]
mod tests;
