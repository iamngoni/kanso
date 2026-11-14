//! A note's embargo: the banner under its title, and the question before
//! publishing it, making a share link to it or exporting it.

use kanso_mobile::{EmbargoView, KansoError};

use super::library::KansoLibrary;

impl KansoLibrary {
    /// The embargo of note `note_id` while it is in force at `now_utc_ms`,
    /// in a phone whose local time is that plus `offset_ms`; `None` when
    /// it has none or it has lifted.
    pub fn embargo(
        &self,
        note_id: String,
        now_utc_ms: i64,
        offset_ms: i64,
    ) -> Result<Option<EmbargoView>, KansoError> {
        self.0.embargo(&note_id, now_utc_ms, offset_ms)
    }
}
