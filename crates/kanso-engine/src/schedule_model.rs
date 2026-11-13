//! What the Schedule hands the desktop and the phone: one dated thing on a
//! day, and what asks for a range of days.

use std::ops::Range;

use kanso_core::dates::Date;

/// What puts an item on the Schedule.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ScheduleKind {
    /// A task, on its due date or on a day its repeat comes round.
    Task,
    /// A note, on the date a date-typed property of it holds.
    Note,
}

/// One thing on one day.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScheduleItem {
    pub kind: ScheduleKind,
    pub note_id: String,
    pub note_title: String,
    pub book_id: String,
    /// A task's line (0-based) in its note; `None` for a note.
    pub line: Option<u32>,
    /// A task's words as written (its date and repeat included), or the
    /// note's title.
    pub text: String,
    /// The day. For a date-time with a zone it is the day in that zone.
    pub date: Date,
    /// `(hour, minute)` on the wall clock of `zone` (or the local one).
    pub time: Option<(u32, u32)>,
    /// The IANA zone of a zoned date-time property.
    pub zone: Option<String>,
    /// A repeating task's words (`every week`), on the written occurrence
    /// and the projected ones alike.
    pub repeat: Option<String>,
    /// A task's box is ticked. Always `false` for a note.
    pub done: bool,
    /// The property a note is placed by (`publish`); `None` for a task.
    pub property: Option<String>,
    /// A repeating task's later occurrence, worked out for this range and
    /// never written. `false` for the date the task itself carries.
    pub projected: bool,
}

/// What [`crate::schedule::schedule`] is asked for.
#[derive(Clone, Debug)]
pub struct ScheduleQuery {
    /// The days, from the first to the one before the last.
    pub range: Range<Date>,
    /// Only the notes of these notebooks; all of them when `None`.
    pub books: Option<Vec<String>>,
    /// Only these date properties place a note; every date-typed property
    /// of its notebook when `None`.
    pub date_keys: Option<Vec<String>>,
    /// Done tasks too.
    pub include_done: bool,
}

impl ScheduleQuery {
    pub fn new(range: Range<Date>) -> ScheduleQuery {
        ScheduleQuery {
            range,
            books: None,
            date_keys: None,
            include_done: true,
        }
    }
}
