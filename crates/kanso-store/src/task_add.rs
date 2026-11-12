//! Adding a task to a note: the end of a note, today's daily note, or the
//! Inbox's task list. Used by the local API's `kanso-add-task`.

use kanso_core::dates::DateTime;
use kanso_core::roles::Role;
use rusqlite::OptionalExtension;
use serde_json::{Value, json};

use crate::write::{load_row, rewrite_note};
use crate::{Origin, Put, Result, Store, StoreError};
use kanso_core::DocType;

/// The title of the note in the Inbox that holds tasks added there.
pub const INBOX_TASKS_TITLE: &str = "Tasks";

/// Where a new task goes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TaskTarget {
    /// The end of this note (its ID).
    Note(String),
    /// The daily note for the day of `now`, made when missing.
    Daily,
    /// The note titled [`INBOX_TASKS_TITLE`] in the Inbox, made when missing.
    Inbox,
}

/// A task added.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskAdded {
    pub note_id: String,
    /// The line (0-based) of the new task in the note's body.
    pub line: u32,
    pub rev: String,
}

impl Store {
    /// Appends `task` (a whole line, `- [ ] …`) to the end of the note
    /// `target` names, in one write: after the body's last line, and with
    /// the note's edit time at `now_ms`. Nothing else in the body changes.
    pub fn add_task(
        &self,
        target: &TaskTarget,
        task: &str,
        now: DateTime,
        now_ms: i64,
        origin: Origin,
    ) -> Result<TaskAdded> {
        let id = match target {
            TaskTarget::Note(id) => id.clone(),
            TaskTarget::Daily => self.open_daily(now, origin)?.id,
            TaskTarget::Inbox => self.inbox_tasks_note(origin)?,
        };
        // A locked note keeps its words, and gets no plain line appended.
        if self.note_is_locked(&id)? || self.note_in_locked_book(&id)? {
            return Err(StoreError::Invalid(
                "That note is locked. Unlock it in Kanso to add a task to it.".into(),
            ));
        }
        self.write(|tx, changes| {
            let row = load_row(tx, &id)?
                .filter(|r| !r.deleted && r.doc_type == DocType::Note)
                .ok_or(StoreError::NotFound)?;
            let body = row.fields.get("body").and_then(Value::as_str).unwrap_or("");
            let mut next = body.to_owned();
            if !next.is_empty() && !next.ends_with('\n') {
                next.push('\n');
            }
            let line = next.matches('\n').count() as u32;
            next.push_str(task);
            next.push('\n');
            rewrite_note(tx, &id, origin, changes, |fields| {
                fields.insert("body".into(), next.into());
                fields.insert("updatedAt".into(), now_ms.into());
            })?;
            let rev = load_row(tx, &id)?.map(|r| r.rev).unwrap_or_default();
            Ok(TaskAdded {
                note_id: id.clone(),
                line,
                rev,
            })
        })
    }

    /// The Inbox's task list note, made (with the Inbox) when missing.
    fn inbox_tasks_note(&self, origin: Origin) -> Result<String> {
        let inbox = self.role_book(Role::Inbox, origin)?;
        let found: Option<String> = self.read(|conn| {
            Ok(conn
                .query_row(
                    "SELECT id FROM docs WHERE type = 'note' AND deleted = 0
                     AND book_id = ?1 AND title = ?2 ORDER BY id LIMIT 1",
                    [&inbox, INBOX_TASKS_TITLE],
                    |row| row.get(0),
                )
                .optional()?)
        })?;
        if let Some(id) = found {
            return Ok(id);
        }
        let made = self.put(
            DocType::Note,
            json!({ "title": INBOX_TASKS_TITLE, "body": "", "bookId": inbox }),
            Put::Upsert,
            origin,
        )?;
        Ok(made.id)
    }
}
