//! The Tasks place (#48 step 7): every open task across the notes, grouped
//! Overdue, Today, Upcoming and No date (and Completed, when shown), the
//! way the desktop's Tasks view groups them (`kanso_engine::tasks::board`).
//!
//! Locked notes are left out entirely: a note that is sealed, or in a locked
//! notebook, gives no task, no title and no count, whether the lock is open
//! or closed. Ticking a task is [`Library::set_task`](crate::Library::set_task),
//! one write with an undo ticket.

use std::collections::{HashMap, HashSet};

use chrono::{Local, Offset as _, TimeZone as _};
use kanso_core::dates::DateTime;
use kanso_core::now_ms;
use kanso_core::tasks::{Due, TaskGroup};
use kanso_engine::tasks::board::{Board, Entry, TaskId};
use kanso_engine::tasks::{Tone, chip_label, display_text, tone};
use kanso_store::{TaskQuery, TaskRow};

use crate::boundary::guard;
use crate::errors::KansoError;
use crate::library::Library;
use crate::model::{DueTone, TaskBoard, TaskGroupKind, TaskItem, TaskListQuery, TaskSection};

impl Library {
    /// The tasks of the notes outside the trash, in groups, at the phone's
    /// local time now. Locked notes are left out.
    pub fn tasks(&self, query: TaskListQuery) -> Result<TaskBoard, KansoError> {
        self.tasks_at(query, local_now())
    }

    /// How many open tasks are overdue at the phone's local time now: the
    /// badge of the Tasks place. Locked notes are left out.
    pub fn overdue_tasks(&self) -> Result<u32, KansoError> {
        self.overdue_tasks_at(local_now())
    }

    /// [`Library::tasks`] at local time `now`.
    pub fn tasks_at(&self, query: TaskListQuery, now: DateTime) -> Result<TaskBoard, KansoError> {
        guard(|| {
            let rows = self.open_rows(
                query.notebook_id.as_deref(),
                query.show_done || !query.kept.is_empty(),
            )?;
            let names: HashMap<String, String> = self
                .inner
                .store
                .books()?
                .into_iter()
                .map(|book| (book.id, book.name))
                .collect();
            let mut board = Board {
                book: query.notebook_id,
                show_done: query.show_done,
                kept: HashSet::new(),
                ..Board::default()
            };
            let kept: HashSet<String> = query.kept.into_iter().collect();
            for row in &rows {
                if kept.contains(&key(row)) {
                    board.kept.insert(TaskId::of(row));
                }
            }
            board.set_rows(rows);
            let mut groups: Vec<TaskSection> = Vec::new();
            for entry in board.entries(now) {
                match entry {
                    Entry::Header { group, .. } => groups.push(TaskSection {
                        group: kind(group),
                        label: group.label().to_owned(),
                        items: Vec::new(),
                    }),
                    Entry::Task(ix) => {
                        let row = &board.rows[ix];
                        let item = item(row, names.get(&row.book_id), now);
                        if let Some(section) = groups.last_mut() {
                            section.items.push(item);
                        }
                    }
                }
            }
            Ok(TaskBoard { groups })
        })
    }

    /// [`Library::overdue_tasks`] at local time `now`.
    pub fn overdue_tasks_at(&self, now: DateTime) -> Result<u32, KansoError> {
        guard(|| {
            let rows = self.open_rows(None, false)?;
            let board = Board::default();
            Ok(rows
                .iter()
                .filter(|row| board.group_of(row, now) == TaskGroup::Overdue)
                .count() as u32)
        })
    }

    /// The task rows of the live, unlocked notes (in `notebook` when given),
    /// done ones too when asked.
    fn open_rows(
        &self,
        notebook: Option<&str>,
        include_done: bool,
    ) -> Result<Vec<TaskRow>, KansoError> {
        let store = &self.inner.store;
        let locked_books = store.locked_books()?;
        if notebook.is_some_and(|book| locked_books.contains(book)) {
            return Ok(Vec::new());
        }
        let rows = store.tasks(&TaskQuery {
            book_id: notebook.map(str::to_owned),
            include_done,
            ..TaskQuery::default()
        })?;
        let mut sealed: HashMap<String, bool> = HashMap::new();
        let mut kept = Vec::with_capacity(rows.len());
        for row in rows {
            if locked_books.contains(&row.book_id) {
                continue;
            }
            let locked = match sealed.get(&row.note_id) {
                Some(locked) => *locked,
                None => {
                    let locked = store.note_is_locked(&row.note_id)?;
                    sealed.insert(row.note_id.clone(), locked);
                    locked
                }
            };
            if !locked {
                kept.push(row);
            }
        }
        Ok(kept)
    }
}

/// The key a task is known by: its note, its line and its words as written.
fn key(row: &TaskRow) -> String {
    format!("{}\u{1f}{}\u{1f}{}", row.note_id, row.line, row.text)
}

/// A row as the place shows it at local time `now`.
fn item(row: &TaskRow, notebook_name: Option<&String>, now: DateTime) -> TaskItem {
    let due = row.due.as_deref().and_then(Due::parse);
    TaskItem {
        key: key(row),
        note_id: row.note_id.clone(),
        note_title: row.note_title.clone(),
        notebook_id: row.book_id.clone(),
        notebook_name: notebook_name.cloned().unwrap_or_default(),
        line: row.line,
        text: display_text(&row.text),
        source_text: row.text.clone(),
        checked: row.checked,
        due_label: due.map(|due| chip_label(due, row.checked, now)),
        due_tone: due.map(|due| match tone(due, row.checked, now) {
            Tone::Plain => DueTone::Plain,
            Tone::Today => DueTone::Today,
            Tone::Overdue => DueTone::Overdue,
            Tone::Done => DueTone::Done,
        }),
    }
}

/// The group a heading names.
fn kind(group: TaskGroup) -> TaskGroupKind {
    match group {
        TaskGroup::Overdue => TaskGroupKind::Overdue,
        TaskGroup::Today => TaskGroupKind::Today,
        TaskGroup::Upcoming => TaskGroupKind::Upcoming,
        TaskGroup::NoDate => TaskGroupKind::NoDate,
        TaskGroup::Done => TaskGroupKind::Done,
    }
}

/// The phone's local time now.
pub(crate) fn local_now() -> DateTime {
    let now = now_ms();
    let offset = Local
        .timestamp_millis_opt(now)
        .single()
        .map_or(0, |at| i64::from(at.offset().fix().local_minus_utc()));
    DateTime::from_local_ms(now + offset * 1000)
}
