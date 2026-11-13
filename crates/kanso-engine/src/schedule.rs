//! The Schedule without a window: the tasks and notes that fall in a range
//! of days. A task is on its due date; an open repeating task is also on
//! every later day its repeat comes round within the range (worked out
//! here, never written). A note is on the date of each date-typed property
//! its notebook defines. The desktop's month grid and the phone's agenda
//! both read this.

use std::collections::{HashMap, HashSet};
use std::ops::Range;

use kanso_core::dates::Date;
use kanso_core::model::Note;
use kanso_core::properties::{self, Value};
use kanso_core::repeat::repeat_token;
use kanso_core::repeat_model::Repeat;
use kanso_core::schema::PropertyType;
use kanso_core::tasks::Due;
use kanso_core::zoned;
use kanso_store::{Result, Store, TaskQuery};

pub use crate::schedule_model::{ScheduleItem, ScheduleKind, ScheduleQuery};

/// The most occurrences stepped over to reach a range, so a task due long
/// ago that repeats daily cannot stall a view.
const MAX_STEPS: usize = 100_000;

/// The dates a repeating task, first due `due`, has in `range`: `due`
/// itself when it is there, then each later one. Pure. Counted as
/// [`Repeat::next`] counts for a task done on its day, so ticking one
/// occurrence on its day writes the next one in this list.
pub fn occurrences(due: &Due, repeat: &Repeat, range: &Range<Date>) -> Vec<Date> {
    let mut found = Vec::new();
    let mut current = *due;
    for _ in 0..MAX_STEPS {
        if current.date >= range.end {
            break;
        }
        if current.date >= range.start {
            found.push(current.date);
        }
        current = repeat.next(&current, current.date);
    }
    found
}

/// The items of `query.range`, by day, then time (an item without one
/// first), then tasks before notes, then title.
pub fn schedule(store: &Store, query: &ScheduleQuery) -> Result<Vec<ScheduleItem>> {
    let mut items = Vec::new();
    if query.range.start >= query.range.end {
        return Ok(items);
    }
    if query.books.as_ref().is_none_or(|books| !books.is_empty()) {
        task_items(store, query, &mut items)?;
        note_items(store, query, &mut items)?;
    }
    items.sort_by(|a, b| {
        (a.date, a.time, a.kind, &a.note_title, a.line, &a.property).cmp(&(
            b.date,
            b.time,
            b.kind,
            &b.note_title,
            b.line,
            &b.property,
        ))
    });
    Ok(items)
}

fn task_items(store: &Store, query: &ScheduleQuery, items: &mut Vec<ScheduleItem>) -> Result<()> {
    let rows = store.tasks(&TaskQuery {
        book_ids: query.books.clone(),
        include_done: query.include_done,
        ..Default::default()
    })?;
    let mut locked: HashMap<String, bool> = HashMap::new();
    for row in rows {
        let Some(due) = row.due.as_deref().and_then(Due::parse) else {
            continue;
        };
        let repeat = if row.checked {
            None
        } else {
            repeat_token(&row.text).map(|(_, repeat)| repeat)
        };
        let dates = match &repeat {
            Some(repeat) => occurrences(&due, repeat, &query.range),
            None if due.date >= query.range.start && due.date < query.range.end => {
                vec![due.date]
            }
            None => Vec::new(),
        };
        if dates.is_empty() {
            continue;
        }
        // The text of a locked note is not to be read.
        let sealed = match locked.get(&row.note_id) {
            Some(sealed) => *sealed,
            None => {
                let sealed = store.note_is_locked(&row.note_id)?;
                locked.insert(row.note_id.clone(), sealed);
                sealed
            }
        };
        if sealed {
            continue;
        }
        let label = repeat.map(|repeat| repeat.to_string());
        for date in dates {
            items.push(ScheduleItem {
                kind: ScheduleKind::Task,
                note_id: row.note_id.clone(),
                note_title: row.note_title.clone(),
                book_id: row.book_id.clone(),
                line: Some(row.line),
                text: row.text.clone(),
                date,
                time: due.time,
                zone: None,
                repeat: label.clone(),
                done: row.checked,
                property: None,
                projected: date != due.date,
            });
        }
    }
    Ok(())
}

fn note_items(store: &Store, query: &ScheduleQuery, items: &mut Vec<ScheduleItem>) -> Result<()> {
    let locked = store.locked_books()?;
    for book in store.books()? {
        if locked.contains(&book.id)
            || query
                .books
                .as_ref()
                .is_some_and(|books| !books.contains(&book.id))
        {
            continue;
        }
        let keys: HashSet<String> = store
            .notebook_properties(&book.id)?
            .into_iter()
            .filter(|def| {
                matches!(
                    def.kind,
                    PropertyType::Date | PropertyType::DateTime | PropertyType::DateTimeZone
                )
            })
            .map(|def| def.key)
            .filter(|key| {
                query
                    .date_keys
                    .as_ref()
                    .is_none_or(|only| only.contains(key))
            })
            .collect();
        if keys.is_empty() {
            continue;
        }
        for note in store.notes_in_books(std::slice::from_ref(&book.id))? {
            if note.is_locked() {
                continue;
            }
            place_note(&note, &keys, &query.range, items);
        }
    }
    Ok(())
}

fn place_note(
    note: &Note,
    keys: &HashSet<String>,
    range: &Range<Date>,
    items: &mut Vec<ScheduleItem>,
) {
    for (key, value) in properties::parse(&note.body) {
        if !keys.contains(&key) {
            continue;
        }
        let (date, time, zone) = match &value {
            Value::Date(text) => match Date::parse_iso(text) {
                Some(date) => (date, None, None),
                None => continue,
            },
            Value::DateTime(text) => match zoned::parse_local(text) {
                Some(at) => (at.date, Some((at.hour, at.minute)), None),
                None => continue,
            },
            Value::DateTimeZone(text) => match zoned::parse_zoned(text) {
                Some(at) => (
                    at.local.date,
                    Some((at.local.hour, at.local.minute)),
                    Some(at.zone),
                ),
                None => continue,
            },
            _ => continue,
        };
        if date < range.start || date >= range.end {
            continue;
        }
        items.push(ScheduleItem {
            kind: ScheduleKind::Note,
            note_id: note.id.clone(),
            note_title: note.title.clone(),
            book_id: note.book_id.clone(),
            line: None,
            text: note.title.clone(),
            date,
            time,
            zone,
            repeat: None,
            done: false,
            property: Some(key),
            projected: false,
        });
    }
}

#[cfg(test)]
#[path = "schedule_tests.rs"]
mod tests;
