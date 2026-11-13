//! A note's embargo, as its `embargo` property holds it (see
//! [`crate::embargo`]).

use crate::dates::DateTime;
use crate::zoned_model::ZonedTime;

/// When an embargo lifts, as written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Lift {
    /// A wall-clock time in a named zone: `2026-11-18T09:00
    /// America/New_York`. The same moment on every device.
    Zoned(ZonedTime),
    /// A wall-clock time with no zone (`2026-11-18T09:00`) or a day
    /// (`2026-11-18`, its midnight): it lifts at that time wherever the
    /// device is.
    Local(DateTime),
}

/// A note's embargo: when it lifts, and the value as written, which names
/// its reminder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Embargo {
    pub lift: Lift,
    /// The property's value as written, without surrounding spaces.
    pub text: String,
}

/// What the embargo holds back.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Gate {
    /// Publishing the note at a link anyone can open.
    Publish,
    /// Making an invite link to the shared note.
    ShareLink,
    /// Exporting the note as a web page or a PDF.
    Export,
}

/// What asking before a held-back step says.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ask {
    /// "This note is under embargo".
    pub title: String,
    /// "Embargo lifts Tue 18 Nov, 09:00 New York (in 5 days). Publish
    /// anyway?"
    pub body: String,
    /// The button that goes ahead: "Publish anyway".
    pub confirm: String,
}
