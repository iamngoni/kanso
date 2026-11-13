//! What a property row's editor is made of, and what a pick writes. The
//! rules that choose an editor and apply a pick are in [`crate::property_pick`].

use crate::model::Property;

/// The editor a property row has, besides a text field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickKind {
    /// One of the notebook's options.
    Select,
    /// One of a shared note's people.
    Person,
    Date,
    DateTime,
    /// A date and time, and the zone they are in.
    DateTimeZone,
}

/// A day and a minute of it, as a calendar and a clock show them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Moment {
    pub year: i32,
    /// 1-12.
    pub month: u32,
    /// 1-31.
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
}

/// How a property row is edited.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PropertyPick {
    pub kind: PickKind,
    /// The rows of a select or a person: the options in order, with the
    /// current value first when the notebook no longer lists it. Empty for
    /// every other kind.
    pub rows: Vec<String>,
    /// The current value is one the rows only hold so that it still shows.
    pub unlisted: bool,
    /// The day and time the value holds, when it holds one.
    pub moment: Option<Moment>,
    /// The zone a date and time with a zone is in, or the zone this device
    /// is in while the value has none yet.
    pub zone: Option<String>,
    /// The value has a zone of its own (`zone` is not the device's).
    pub zone_set: bool,
    /// A zoned time as a person says it: `Tue 18 Nov, 09:00 New York`.
    pub label: Option<String>,
}

/// What a picker picked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PropertyChoice {
    /// Nothing: the value is emptied.
    Clear,
    /// One of a select's options or a person's usernames.
    Option { text: String },
    /// A day (a date property; the time of the moment is ignored).
    Day { moment: Moment },
    /// A day and time with no zone to put it in yet.
    Moment { moment: Moment },
    /// A day and time in a zone.
    Zoned { moment: Moment, zone: String },
    /// A zone, for the day and time the value already holds.
    Zone { zone: String },
}

/// What a pick did to the note.
#[derive(Clone, Debug, PartialEq)]
pub struct PickOutcome {
    /// False when the front matter changed under the pick (the row is not
    /// the one the picker showed, or no longer holds what it showed) and the
    /// pick was dropped.
    pub applied: bool,
    /// The note's properties as they are now.
    pub properties: Vec<Property>,
}

/// One zone of the picker's list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZoneRow {
    /// The tz database name: `America/New_York`.
    pub name: String,
    /// How a person reads it: `New York`.
    pub label: String,
}
