//! The tasks in note bodies, read from the derived `note_tasks` table: the
//! Tasks view and its overdue count, ticking a task off from there, and
//! the timed tasks reminders come from.

use chrono::{Datelike as _, Local, TimeZone as _};
use kanso_core::dates::{Date, DateTime};
use kanso_core::embargo;
use kanso_core::model::LOCK_FIELD;
use kanso_core::reminders::{ACK_FIELD, Acks, acknowledge, acks_value, read_acks, task_key};
use kanso_core::reschedule::set_task_due;
use kanso_core::tasks::{TaskEdit, set_task_checked, task_items};
use rusqlite::params;
use rusqlite::types::Value as SqlValue;
use serde_json::Value;

use crate::note_property_model::PropertyEdit;
use crate::write::{load_row, rewrite_note};
use crate::{Origin, Result, Store, StoreError};

/// A task-list item, with the note it is in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskRow {
    pub note_id: String,
    pub note_title: String,
    pub book_id: String,
    /// The line (0-based) it is on in the note's body.
    pub line: u32,
    pub checked: bool,
    /// The words after the box, as written (due date included).
    pub text: String,
    /// `YYYY-MM-DD` or `YYYY-MM-DDTHH:MM`.
    pub due: Option<String>,
    pub note_updated_at: i64,
}

/// Which tasks [`Store::tasks`] lists.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TaskQuery {
    /// Only tasks in this notebook's notes.
    pub book_id: Option<String>,
    /// Only tasks in the notes of these notebooks (a notebook and the ones
    /// inside it, say). Together with `book_id`, both must hold.
    pub book_ids: Option<Vec<String>>,
    /// Done tasks too.
    pub include_done: bool,
    /// Only tasks due on a day before this one (`YYYY-MM-DD`).
    pub due_before: Option<String>,
    /// Only tasks due on a day after this one (`YYYY-MM-DD`).
    pub due_after: Option<String>,
    pub limit: Option<usize>,
}

/// What [`Store::set_task_checked`] did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckOff {
    /// The note was written with its new revision.
    Changed { rev: String },
    /// The task already was that way.
    Unchanged,
    /// The task isn't in the note any more (or is ambiguous now).
    Missing,
}

/// What puts back the body a tick that wrote a repeating task's next one
/// changed: the body before, and the body the tick made. The undo writes
/// `before` only while the note still is `after`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TickUndo {
    pub before: String,
    pub after: String,
}

/// What [`Store::tick_task`] did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TickResult {
    pub off: CheckOff,
    /// Set when the tick wrote the next one of a repeating task: the way
    /// to take both changes back in one write ([`Store::undo_tick`]).
    pub undo: Option<TickUndo>,
}

/// The local calendar day of `now_ms`, in this device's zone.
pub fn local_today(now_ms: i64) -> Date {
    let day = Local
        .timestamp_millis_opt(now_ms)
        .earliest()
        .map(|t| t.date_naive())
        .unwrap_or_default();
    Date::new(day.year(), day.month(), day.day()).unwrap_or(Date::from_days(0))
}

/// An open timed task that may remind, with the acks of its note.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReminderTask {
    pub note_id: String,
    pub note_title: String,
    pub line: u32,
    pub text: String,
    pub due: String,
    pub key: String,
    pub acks: Acks,
}

/// SQL for "`t` is a live task outside the trash", joined to its note `d`.
const LIVE_TASKS: &str = "FROM note_tasks t JOIN docs d ON d.id = t.note_id
     WHERE d.deleted = 0 AND IFNULL(d.book_id, '') != 'trash'";

impl Store {
    /// The tasks of live notes outside the trash that `query` asks for:
    /// dated ones first, soonest first, then the rest, each note's in
    /// order.
    pub fn tasks(&self, query: &TaskQuery) -> Result<Vec<TaskRow>> {
        // Asked for by notebooks, the notes of those notebooks lead through
        // the notebook index; otherwise the planner reads every task.
        let from = if query.book_ids.is_some() {
            "FROM docs d INDEXED BY docs_book CROSS JOIN note_tasks t ON t.note_id = d.id
             WHERE d.type = 'note' AND d.deleted = 0 AND IFNULL(d.book_id, '') != 'trash'"
        } else {
            LIVE_TASKS
        };
        let mut sql = format!(
            "SELECT t.note_id, d.title, d.book_id, t.line, t.checked, t.text, t.due, d.updated_at {from}"
        );
        let mut values: Vec<SqlValue> = Vec::new();
        if let Some(book) = &query.book_id {
            sql.push_str(" AND d.book_id = ?");
            values.push(SqlValue::Text(book.clone()));
        }
        if let Some(books) = &query.book_ids {
            if books.is_empty() {
                sql.push_str(" AND 0");
            } else {
                sql.push_str(" AND d.book_id IN (");
                sql.push_str(&vec!["?"; books.len()].join(","));
                sql.push(')');
                values.extend(books.iter().cloned().map(SqlValue::Text));
            }
        }
        if !query.include_done {
            sql.push_str(" AND t.checked = 0");
        }
        if let Some(before) = &query.due_before {
            sql.push_str(" AND t.due IS NOT NULL AND substr(t.due, 1, 10) < ?");
            values.push(SqlValue::Text(before.clone()));
        }
        if let Some(after) = &query.due_after {
            sql.push_str(" AND t.due IS NOT NULL AND substr(t.due, 1, 10) > ?");
            values.push(SqlValue::Text(after.clone()));
        }
        sql.push_str(
            " ORDER BY (t.due IS NULL), t.due, d.updated_at DESC, t.note_id, t.ord LIMIT ?",
        );
        values.push(SqlValue::Integer(query.limit.map_or(-1, |n| n as i64)));
        self.read(|conn| {
            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt.query_map(rusqlite::params_from_iter(values), |row| {
                Ok(TaskRow {
                    note_id: row.get(0)?,
                    note_title: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                    book_id: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                    line: row.get(3)?,
                    checked: row.get(4)?,
                    text: row.get(5)?,
                    due: row.get(6)?,
                    note_updated_at: row.get::<_, Option<i64>>(7)?.unwrap_or(0),
                })
            })?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }

    /// How many open tasks of live notes outside the trash are overdue at
    /// local time `now`: due on an earlier day, or earlier today.
    pub fn overdue_count(&self, now: DateTime) -> Result<u32> {
        let today = now.date.iso();
        let time = format!("{:02}:{:02}", now.hour, now.minute);
        self.read(|conn| {
            Ok(conn.query_row(
                &format!(
                    "SELECT COUNT(*) {LIVE_TASKS} AND t.checked = 0 AND t.due IS NOT NULL
                     AND (substr(t.due, 1, 10) < ?1
                          OR (substr(t.due, 1, 10) = ?1 AND length(t.due) = 16 AND substr(t.due, 12, 5) < ?2))"
                ),
                params![today, time],
                |row| row.get::<_, i64>(0),
            )? as u32)
        })
    }

    /// Ticks (or clears) the task with `text` on `line` of note `id`, in one
    /// write: the body changes only in its box (and, for an open repeating
    /// task, gets its next one above it, see [`kanso_core::tick`]), and the
    /// note's edit time becomes `now_ms`. When the line moved, the one task
    /// with that text is changed; when the task is gone, nothing is written.
    /// Every place that ticks a task comes through here.
    pub fn set_task_checked(
        &self,
        id: &str,
        line: u32,
        text: &str,
        checked: bool,
        now_ms: i64,
        origin: Origin,
    ) -> Result<CheckOff> {
        Ok(self
            .tick_task(id, line, text, checked, local_today(now_ms), now_ms, origin)?
            .off)
    }

    /// [`Store::set_task_checked`] on the local day `today`, with what takes
    /// a repeating task's tick back.
    #[allow(clippy::too_many_arguments)]
    pub fn tick_task(
        &self,
        id: &str,
        line: u32,
        text: &str,
        checked: bool,
        today: Date,
        now_ms: i64,
        origin: Origin,
    ) -> Result<TickResult> {
        self.write(|tx, changes| {
            let row = load_row(tx, id)?
                .filter(|r| !r.deleted)
                .ok_or(StoreError::NotFound)?;
            let body = row.fields.get("body").and_then(Value::as_str).unwrap_or("");
            let plain = |off| TickResult { off, undo: None };
            match set_task_checked(body, line, text, checked, today) {
                TaskEdit::Missing => Ok(plain(CheckOff::Missing)),
                TaskEdit::Unchanged => Ok(plain(CheckOff::Unchanged)),
                TaskEdit::Changed(after) => {
                    let wrote_next = after.lines().count() > body.lines().count();
                    let undo = wrote_next.then(|| TickUndo {
                        before: body.to_owned(),
                        after: after.clone(),
                    });
                    rewrite_note(tx, id, origin, changes, |fields| {
                        fields.insert("body".into(), after.into());
                        fields.insert("updatedAt".into(), now_ms.into());
                    })?;
                    let rev = load_row(tx, id)?.map(|r| r.rev).unwrap_or_default();
                    Ok(TickResult {
                        off: CheckOff::Changed { rev },
                        undo,
                    })
                }
            }
        })
    }

    /// Moves the due date of the task with text `text` (looked for on `line`
    /// first, as [`Store::set_task_checked`] does) to `day`, in one write:
    /// its time and repeat stay. `None` when the task is gone or has no due
    /// date; nothing is written when it is on that day already. A sealed note
    /// is refused ([`StoreError::Invalid`]). The edit puts the note back
    /// through [`Store::undo_note_property`].
    pub fn set_task_due(
        &self,
        id: &str,
        line: u32,
        text: &str,
        day: Date,
        now_ms: i64,
        origin: Origin,
    ) -> Result<Option<PropertyEdit>> {
        self.write(|tx, changes| {
            let row = load_row(tx, id)?
                .filter(|r| !r.deleted)
                .ok_or(StoreError::NotFound)?;
            if row.fields.contains_key(LOCK_FIELD) {
                return Err(StoreError::Invalid("This note is locked.".into()));
            }
            let before = row
                .fields
                .get("body")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_owned();
            let before_updated_at = row
                .fields
                .get("updatedAt")
                .and_then(Value::as_i64)
                .unwrap_or(0);
            let after = match set_task_due(&before, line, text, day) {
                TaskEdit::Missing => return Ok(None),
                TaskEdit::Unchanged => before.clone(),
                TaskEdit::Changed(after) => after,
            };
            let changed = after != before;
            if changed {
                let body = after.clone();
                rewrite_note(tx, id, origin, changes, |fields| {
                    fields.insert("body".into(), body.into());
                    fields.insert("updatedAt".into(), now_ms.into());
                })?;
            }
            Ok(Some(PropertyEdit {
                note_id: id.to_owned(),
                before_body: before,
                before_updated_at,
                after_body: after,
                changed,
            }))
        })
    }

    /// Takes a tick back in one write: the note's body becomes the one it
    /// was before, provided it still is the one the tick made.
    /// [`CheckOff::Unchanged`] when the note was edited since, and
    /// [`CheckOff::Missing`] when it is gone.
    pub fn undo_tick(
        &self,
        id: &str,
        undo: &TickUndo,
        now_ms: i64,
        origin: Origin,
    ) -> Result<CheckOff> {
        self.write(|tx, changes| {
            let Some(row) = load_row(tx, id)?.filter(|r| !r.deleted) else {
                return Ok(CheckOff::Missing);
            };
            if row.fields.get("body").and_then(Value::as_str) != Some(undo.after.as_str()) {
                return Ok(CheckOff::Unchanged);
            }
            rewrite_note(tx, id, origin, changes, |fields| {
                fields.insert("body".into(), undo.before.clone().into());
                fields.insert("updatedAt".into(), now_ms.into());
            })?;
            let rev = load_row(tx, id)?.map(|r| r.rev).unwrap_or_default();
            Ok(CheckOff::Changed { rev })
        })
    }

    /// Open tasks with a time, of live notes outside the trash, each with
    /// its reminder's key and its note's acks.
    pub fn reminder_tasks(&self) -> Result<Vec<ReminderTask>> {
        self.read(|conn| {
            let mut stmt = conn.prepare(&format!(
                "SELECT t.note_id, d.title, t.line, t.text, t.due, json_extract(d.json, '$.{ACK_FIELD}')
                 {LIVE_TASKS} AND t.checked = 0 AND length(t.due) = 16
                 ORDER BY t.due, t.note_id, t.ord"
            ))?;
            let rows = stmt.query_map([], |row| {
                let note_id: String = row.get(0)?;
                let text: String = row.get(3)?;
                let due: String = row.get(4)?;
                let acks: Option<String> = row.get(5)?;
                let mut fields = serde_json::Map::new();
                if let Some(acks) = acks.and_then(|a| serde_json::from_str(&a).ok()) {
                    fields.insert(ACK_FIELD.into(), acks);
                }
                Ok(ReminderTask {
                    key: task_key(&note_id, &text, &due),
                    note_id,
                    note_title: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                    line: row.get(2)?,
                    text,
                    due,
                    acks: read_acks(&fields),
                })
            })?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }

    /// Records in note `id` that the reminder `key` (fired at `fired_at`)
    /// was dismissed or done, merged with the acks it has (and leaving out
    /// those of tasks no longer in it). The note's edit time stays: an ack
    /// isn't an edit. Nothing is written when it was acked already.
    pub fn ack_reminder(&self, id: &str, key: &str, fired_at: i64, origin: Origin) -> Result<()> {
        self.write(|tx, changes| {
            let Some(row) = load_row(tx, id)?.filter(|r| !r.deleted) else {
                return Ok(());
            };
            let acks = read_acks(&row.fields);
            let body = row.fields.get("body").and_then(Value::as_str).unwrap_or("");
            // The note's embargo reminds too: its key stays while it does.
            let live: Vec<String> = task_items(body)
                .into_iter()
                .filter_map(|t| Some(task_key(id, &t.text, &t.due?)))
                .chain(embargo::of(body).map(|e| embargo::reminder_key(id, &e)))
                .collect();
            let next = acknowledge(&acks, key, fired_at, &live);
            if next == acks {
                return Ok(());
            }
            rewrite_note(tx, id, origin, changes, |fields| {
                fields.insert(ACK_FIELD.into(), acks_value(&next));
            })
        })
    }
}
