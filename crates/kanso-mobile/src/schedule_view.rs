//! The Schedule: the tasks and dated notes in a range of days.
//!
//! Which items, the repeat expansion and the properties that place a note
//! are `kanso_engine::schedule`, which the desktop's Schedule reads too.
//! This only turns days and times into what the bridge carries.

use kanso_core::properties;
use kanso_core::reschedule::move_value;
use kanso_core::schema::PropertyType;
use kanso_engine::schedule::{self, ScheduleItem, ScheduleKind, ScheduleQuery};

use crate::boundary::guard;
use crate::errors::KansoError;
use crate::journal::{date_of, day_of};
use crate::library::Library;
use crate::model::{CardMoved, DayDate};
use crate::schedule_view_model::{ScheduleRequest, ScheduleRow, ScheduleRowKind};
use crate::undo::Undoable;

impl Library {
    /// What falls on the days of `request`, by day and time: tasks by their
    /// due date (a repeating task on every day it comes round in the range),
    /// and notes by their date properties. Locked notes are left out.
    pub fn schedule(&self, request: ScheduleRequest) -> Result<Vec<ScheduleRow>, KansoError> {
        guard(|| {
            let mut query = ScheduleQuery::new(date_of(request.from)?..date_of(request.until)?);
            query.books = request.books;
            query.date_keys = request.date_keys;
            query.include_done = request.include_done;
            let items = schedule::schedule(&self.inner.store, &query)?;
            Ok(items.into_iter().map(row_of).collect())
        })
    }

    /// The date properties the notebooks `books` (every notebook when
    /// `None`) define, sorted and without repeats: what the Schedule's
    /// property filter offers.
    pub fn schedule_date_keys(
        &self,
        books: Option<Vec<String>>,
    ) -> Result<Vec<String>, KansoError> {
        guard(|| {
            let store = &self.inner.store;
            let ids: Vec<String> = match books {
                Some(ids) => ids,
                None => store.books()?.into_iter().map(|book| book.id).collect(),
            };
            let mut keys = Vec::new();
            for id in ids {
                for def in store.notebook_properties(&id)? {
                    if matches!(
                        def.kind,
                        PropertyType::Date | PropertyType::DateTime | PropertyType::DateTimeZone
                    ) {
                        keys.push(def.key);
                    }
                }
            }
            keys.sort();
            keys.dedup();
            Ok(keys)
        })
    }

    /// Moves a scheduled thing to `to`, keeping the rest of what it says: a
    /// note's date property `property` keeps its time and zone, and a task
    /// (`line`, `text`) keeps its time and repeat. A projected occurrence is
    /// not a date of its own, so the caller passes the real task's line and
    /// words. The change can be taken back with [`Library::undo`] under the
    /// ticket. [`CardMoved::Missing`] when the note, task or property is
    /// gone or no longer a date; [`KansoError::Locked`] for a locked note.
    pub fn move_scheduled(
        &self,
        note_id: &str,
        line: Option<u32>,
        text: &str,
        property: Option<String>,
        to: DayDate,
    ) -> Result<CardMoved, KansoError> {
        guard(|| {
            let day = date_of(to)?;
            let store = &self.inner.store;
            let Some(note) = store.note(note_id)? else {
                return Ok(CardMoved::Missing);
            };
            if store.note_is_locked(note_id)? || store.locked_books()?.contains(&note.book_id) {
                return Err(KansoError::Locked);
            }
            let now = kanso_core::now_ms();
            let edit = match (property, line) {
                (Some(key), _) => {
                    let Some((_, value)) = properties::parse(&note.body)
                        .into_iter()
                        .find(|(name, _)| *name == key)
                    else {
                        return Ok(CardMoved::Missing);
                    };
                    let Some(moved) = move_value(&value, day) else {
                        return Ok(CardMoved::Missing);
                    };
                    store.set_note_property(note_id, &key, &moved, now, kanso_store::Origin::Ui)?
                }
                (None, Some(line)) => {
                    match store.set_task_due(
                        note_id,
                        line,
                        text,
                        day,
                        now,
                        kanso_store::Origin::Ui,
                    )? {
                        Some(edit) => edit,
                        None => return Ok(CardMoved::Missing),
                    }
                }
                (None, None) => return Ok(CardMoved::Missing),
            };
            if !edit.changed {
                return Ok(CardMoved::Unchanged);
            }
            let what = match line {
                Some(_) => kanso_engine::tasks::display_text(text),
                None => note.title.clone(),
            };
            Ok(CardMoved::Changed {
                message: Some(format!("Moved “{what}” to {}.", day.long_label())),
                undo: Some(self.keep_undo(Undoable::Moved(edit))),
            })
        })
    }
}

fn row_of(item: ScheduleItem) -> ScheduleRow {
    ScheduleRow {
        kind: match item.kind {
            ScheduleKind::Task => ScheduleRowKind::Task,
            ScheduleKind::Note => ScheduleRowKind::Note,
        },
        note_id: item.note_id,
        note_title: item.note_title,
        book_id: item.book_id,
        line: item.line,
        text: item.text,
        date: day_of(item.date),
        time: item
            .time
            .map(|(hour, minute)| format!("{hour:02}:{minute:02}")),
        zone: item.zone,
        repeat: item.repeat,
        done: item.done,
        property: item.property,
        projected: item.projected,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use kanso_core::DocType;
    use kanso_store::{Origin, Put};
    use kanso_sync::{MemorySecrets, MemoryServer};
    use serde_json::json;

    use super::*;
    use crate::model::DayDate;

    fn day(year: i32, month: u32, day: u32) -> DayDate {
        DayDate { year, month, day }
    }

    #[test]
    fn rows_carry_what_the_views_need() {
        let dir = tempfile::tempdir().unwrap();
        let library = Library::open_for_test(
            &dir.path().join("library.db"),
            Arc::new(MemorySecrets::new()),
            Arc::new(MemoryServer::new()),
        )
        .unwrap();
        let store = &library.inner.store;
        store
            .put(
                DocType::Book,
                json!({"_id": "book:a", "name": "A"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
        store
            .put(
                DocType::Note,
                json!({"_id": "note:a", "title": "Plans", "bookId": "book:a",
                    "body": "- [ ] Review @2026-11-06T15:00 every week\n"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
        let request = |from, until| ScheduleRequest {
            from,
            until,
            books: None,
            date_keys: None,
            include_done: true,
        };
        let rows = library
            .schedule(request(day(2026, 11, 1), day(2026, 12, 1)))
            .unwrap();
        assert_eq!(rows.len(), 4);
        assert_eq!(rows[0].kind, ScheduleRowKind::Task);
        assert_eq!(rows[0].date, day(2026, 11, 6));
        assert_eq!(rows[0].time.as_deref(), Some("15:00"));
        assert_eq!(rows[0].repeat.as_deref(), Some("every week"));
        assert_eq!(rows[0].line, Some(0));
        assert!(!rows[0].projected && rows[1].projected);
        assert_eq!(rows[3].date, day(2026, 11, 27));
        // A day that is not on the calendar is refused.
        assert!(
            library
                .schedule(request(day(2026, 2, 30), day(2026, 3, 1)))
                .is_err()
        );
    }
}

#[cfg(test)]
#[path = "schedule_move_tests.rs"]
mod move_tests;
