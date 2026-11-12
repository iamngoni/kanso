//! What a board grouped by a property is made of: the property, its lanes,
//! and what writing a move there leaves to undo. The board itself, and how
//! it lays cards out, is `board.rs`.

use kanso_core::properties::Value;
use kanso_core::schema::{BoardSpec, PropertyDef, PropertyType};
use kanso_store::PropertyEdit;

use crate::board::Lane;

/// A board's grouping by a property: the key its notes carry and the lanes
/// that name its values, left to right. Read from the notebook's
/// `kansoBoard` and `kansoProperties`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PropertyBoard {
    /// The front-matter key a card's lane is read from and a move writes.
    pub key: String,
    /// The lanes, in order. A select property's options when the board names
    /// none of its own.
    pub values: Vec<String>,
    /// Whether the lane of notes with no value shows when it is empty.
    pub show_unset: bool,
}

impl PropertyBoard {
    /// The grouping `spec` asks for, with `defs` (the notebook's property
    /// list) saying which values a select property has when the board
    /// lists none.
    pub fn from_spec(spec: &BoardSpec, defs: &[PropertyDef]) -> PropertyBoard {
        let values = if spec.lanes.is_empty() {
            defs.iter()
                .find(|def| def.kind == PropertyType::Select && def.key == spec.by)
                .map(|def| def.options.clone())
                .unwrap_or_default()
        } else {
            spec.lanes.clone()
        };
        PropertyBoard {
            key: spec.by.clone(),
            values,
            show_unset: spec.show_unset,
        }
    }

    /// The lane a note whose property has `value` is in: the lane of that
    /// value, "no value" for a missing or empty one, and the unlisted lane
    /// for a value that is none of the lanes (one a person typed by hand, or
    /// from a lane since removed). Never nothing: a note always has a lane.
    pub fn place(&self, value: Option<&Value>) -> Lane {
        let Some(value) = value.filter(|value| !value.is_empty()) else {
            return Lane::Unset;
        };
        let text = value.text();
        match self.values.iter().find(|lane| **lane == text) {
            Some(lane) if !value.is_raw() => Lane::Value(lane.clone()),
            _ => Lane::Unlisted,
        }
    }

    /// The words on a lane's head: "No stage" for the lane of notes with no
    /// value, "Unlisted stage" for those with a value no lane has.
    pub fn label(&self, lane: &Lane) -> String {
        match lane {
            Lane::Value(value) => value.clone(),
            Lane::Unset => format!("No {}", self.key),
            Lane::Unlisted => format!("Unlisted {}", self.key),
            other => other.label(),
        }
    }
}

/// A move written to a note's property, to put back in one step.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MoveUndo {
    pub edit: PropertyEdit,
}
