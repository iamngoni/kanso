//! What editing a property board's lanes is made of: the edits, why one is
//! refused, and what is left to undo one. The rules are `board_settings.rs`.

use kanso_core::schema::{BoardSpec, PropertyDef};
use kanso_store::PropertyEdit;

/// One change to the lanes of a board grouped by a property.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LaneEdit {
    /// A lane gets another name; the notes in it get it as their value.
    Rename { from: String, to: String },
    /// The lane at index `from` goes to index `to`.
    Move { from: usize, to: usize },
    /// A lane at the end.
    Add(String),
    /// A lane goes; its notes move to lane `move_to`, or to no value.
    Remove {
        lane: String,
        move_to: Option<String>,
    },
}

/// Why a lane edit is refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LaneEditError {
    /// The name is empty.
    Empty,
    /// A lane has that name already.
    Duplicate(String),
    /// There is no such lane.
    Missing(String),
    /// The position is not on the board.
    Position,
    /// The only lane cannot go: a board needs one.
    LastLane,
    /// The notes cannot move to that lane.
    Target(String),
}

impl LaneEditError {
    pub fn message(&self) -> String {
        match self {
            LaneEditError::Empty => "A lane needs a name.".to_owned(),
            LaneEditError::Duplicate(name) => format!("There is a lane called {name} already."),
            LaneEditError::Missing(name) => format!("There is no lane called {name}."),
            LaneEditError::Position => "That lane isn't on the board.".to_owned(),
            LaneEditError::LastLane => "A board needs at least one lane.".to_owned(),
            LaneEditError::Target(name) => format!("{name} isn't a lane to move the notes to."),
        }
    }
}

/// What puts a board back as it was before a lane edit: its saved layout,
/// the notebook's property list, and each note the edit rewrote.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SettingsUndo {
    pub book_id: String,
    pub spec: BoardSpec,
    pub defs: Vec<PropertyDef>,
    pub edits: Vec<PropertyEdit>,
}

/// A lane edit that was written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaneEdited {
    /// The lanes now, in order.
    pub lanes: Vec<String>,
    /// How many notes were rewritten.
    pub rewritten: usize,
    pub undo: SettingsUndo,
}
