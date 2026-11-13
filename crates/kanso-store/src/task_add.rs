//! Adding a task to a note: the end of a note, today's daily note, or the
//! Inbox's task list. Used by the local API's `kanso-add-task`, and the
//! Inbox's task list is the same note Quick Capture's Task mode writes to.

use kanso_core::dates::DateTime;
use kanso_core::roles::Role;
use serde_json::{Value, json};

use crate::write::{load_row, rewrite_note};
use crate::{Origin, Put, Result, Store, StoreError};
use kanso_core::DocType;
use kanso_core::model::TRASH_BOOK_ID;

/// The note in the Inbox that tasks added there collect in, from the local
/// API and from Quick Capture alike. A fixed ID, so every device adds to
/// the same note once they sync.
pub const INBOX_TASKS_ID: &str = "note:inbox-tasks";

/// The title that note is made with.
pub const INBOX_TASKS_TITLE: &str = "Inbox tasks";

/// The title the local API used to give its own Inbox task note. A note
/// with it in the Inbox is folded into [`INBOX_TASKS_ID`] when that note is
/// next used, so tasks added before the two were one aren't left behind.
pub const LEGACY_INBOX_TASKS_TITLE: &str = "Tasks";

/// Where a new task goes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TaskTarget {
    /// The end of this note (its ID).
    Note(String),
    /// The daily note for the day of `now`, made when missing.
    Daily,
    /// The Inbox's task note ([`INBOX_TASKS_ID`]), made when missing.
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

    /// The Inbox's task note ([`INBOX_TASKS_ID`]): made (with the Inbox)
    /// when missing, brought back from the trash when it was deleted, and
    /// with any older [`LEGACY_INBOX_TASKS_TITLE`] note in the Inbox folded
    /// into it ([`merge_task_bodies`]) and that note moved to the trash.
    /// Nothing is folded while the Inbox or either note is locked.
    pub fn inbox_tasks_note(&self, origin: Origin) -> Result<String> {
        let inbox = self.role_book(Role::Inbox, origin)?;
        let mut body = match self.note(INBOX_TASKS_ID)? {
            None => {
                self.put(
                    DocType::Note,
                    json!({"_id": INBOX_TASKS_ID, "title": INBOX_TASKS_TITLE, "body": "", "bookId": inbox}),
                    Put::Upsert,
                    origin,
                )?;
                String::new()
            }
            Some(note) if note.book_id == TRASH_BOOK_ID => {
                self.put(
                    DocType::Note,
                    json!({"_id": INBOX_TASKS_ID, "bookId": inbox}),
                    Put::Merge,
                    origin,
                )?;
                note.body
            }
            Some(note) => note.body,
        };
        let legacy: Vec<String> = self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id FROM docs WHERE type = 'note' AND deleted = 0
                 AND book_id = ?1 AND title = ?2 AND id <> ?3 ORDER BY id",
            )?;
            let ids = stmt
                .query_map([&inbox, LEGACY_INBOX_TASKS_TITLE, INBOX_TASKS_ID], |row| {
                    row.get(0)
                })?
                .collect::<std::result::Result<Vec<String>, _>>()?;
            Ok(ids)
        })?;
        // Locked words aren't plain text to move.
        if self.note_is_locked(INBOX_TASKS_ID)? || self.locked_books()?.contains(&inbox) {
            return Ok(INBOX_TASKS_ID.to_owned());
        }
        for id in legacy {
            if self.note_is_locked(&id)? {
                continue;
            }
            let Some(old) = self.note(&id)? else { continue };
            body = merge_task_bodies(&body, &old.body);
            self.put(
                DocType::Note,
                json!({"_id": INBOX_TASKS_ID, "body": body}),
                Put::Merge,
                origin,
            )?;
            self.put(
                DocType::Note,
                json!({"_id": id, "bookId": TRASH_BOOK_ID}),
                Put::Merge,
                origin,
            )?;
        }
        Ok(INBOX_TASKS_ID.to_owned())
    }
}

/// `into` with `from`'s lines after it: a line break between the two when
/// `into` doesn't end in one, and nothing added when `from` is blank.
pub fn merge_task_bodies(into: &str, from: &str) -> String {
    if from.trim().is_empty() {
        return into.to_owned();
    }
    let mut out = into.to_owned();
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(from);
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}
