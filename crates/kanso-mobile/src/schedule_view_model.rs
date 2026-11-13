//! What the Schedule hands the phone: the engine's items with their days and
//! times as the bridge carries them. The rules (which tasks, how a repeat
//! is expanded, which properties place a note) are `kanso_engine::schedule`.

use crate::model::DayDate;

/// What puts a row on the Schedule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScheduleRowKind {
    /// A task, on its due date or a day its repeat comes round.
    Task,
    /// A note, on a date property it holds.
    Note,
}

/// What the Schedule is asked for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScheduleRequest {
    /// The first day.
    pub from: DayDate,
    /// The day after the last one.
    pub until: DayDate,
    /// Only the notes of these notebooks; all when `None`.
    pub books: Option<Vec<String>>,
    /// Only these date properties place a note; all the notebook's date
    /// properties when `None`.
    pub date_keys: Option<Vec<String>>,
    /// Done tasks too.
    pub include_done: bool,
}

/// One thing on one day.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScheduleRow {
    pub kind: ScheduleRowKind,
    pub note_id: String,
    pub note_title: String,
    pub book_id: String,
    /// A task's line (0-based) in its note; `None` for a note.
    pub line: Option<u32>,
    /// A task's words as written, or the note's title.
    pub text: String,
    /// The day (in `zone`, for a zoned date-time).
    pub date: DayDate,
    /// `09:00`, on the wall clock of `zone` or the local one; `None` for a
    /// date without a time.
    pub time: Option<String>,
    /// The IANA zone of a zoned date-time property.
    pub zone: Option<String>,
    /// `every week`, on a repeating task's every row.
    pub repeat: Option<String>,
    pub done: bool,
    /// The property a note is placed by; `None` for a task.
    pub property: Option<String>,
    /// A later occurrence of a repeating task, worked out and never written.
    pub projected: bool,
}
