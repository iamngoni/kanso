//! A note under embargo, as reminders read it (see
//! [`crate::Store::embargo_notes`]).

use kanso_core::embargo::Embargo;
use kanso_core::reminders::Acks;

/// A live note outside the trash whose `embargo` property Kanso can read,
/// with the reminder acknowledgements it keeps.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EmbargoNote {
    pub note_id: String,
    pub note_title: String,
    pub book_id: String,
    pub embargo: Embargo,
    pub acks: Acks,
}
