//! The lines of the Tasks board: a task's identity, a line of the list,
//! and what a check-off did.

use kanso_core::tasks::TaskGroup;
use kanso_store::TaskRow;

/// A task as the list knows it: its note, its line and its words.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TaskId {
    pub note_id: String,
    pub line: u32,
    pub text: String,
}

impl TaskId {
    pub fn of(row: &TaskRow) -> TaskId {
        TaskId {
            note_id: row.note_id.clone(),
            line: row.line,
            text: row.text.clone(),
        }
    }
}

/// A line of the list: a group's header or a task (an index into the
/// board's rows).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Entry {
    Header {
        group: TaskGroup,
        count: usize,
        collapsed: bool,
    },
    Task(usize),
}

/// A check-off that ⌘Z can take back: the task and the state it was given.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckedOff {
    pub id: TaskId,
    pub checked: bool,
}

/// What a check-off did, for the view to say.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Toggled {
    Done(CheckedOff),
    /// Nothing to do: no task selected, or it already was that way.
    Nothing,
    /// The task's note changed since the list was read: it is read again.
    Moved,
    Failed(String),
}
