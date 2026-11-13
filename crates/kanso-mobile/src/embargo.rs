//! Embargoes on the phone: whether a note is held back now, for the banner
//! under its title and the question before publishing it, making a share
//! link to it or exporting it. The lift notice is a reminder
//! (`reminders.rs`).

use kanso_core::embargo::{self, Embargo, Lift};

use crate::boundary::guard;
use crate::embargo_model::EmbargoView;
use crate::errors::KansoError;
use crate::library::Library;

impl Library {
    /// The embargo of note `note_id` while it is in force at `now_utc_ms`,
    /// in a phone whose local time is that plus `offset_ms`. `None` when the
    /// note has none, it has lifted, or the note is locked (its text is
    /// sealed).
    pub fn embargo(
        &self,
        note_id: &str,
        now_utc_ms: i64,
        offset_ms: i64,
    ) -> Result<Option<EmbargoView>, KansoError> {
        guard(|| {
            let Some(note) = self.inner.store.note(note_id)? else {
                return Ok(None);
            };
            let now_local = now_utc_ms.saturating_add(offset_ms);
            Ok(embargo::check(&note.body, now_utc_ms, now_local)
                .map(|found| view_of(&found, now_utc_ms, now_local)))
        })
    }
}

/// What the phone is told of `found` at `now_utc_ms` (local `now_local`).
fn view_of(found: &Embargo, now_utc_ms: i64, now_local: i64) -> EmbargoView {
    let remaining = found.remaining_ms(now_utc_ms, now_local);
    EmbargoView {
        lifts: found.label(),
        lifts_in: embargo::in_words(remaining),
        lifts_at_ms: now_utc_ms.saturating_add(remaining),
        zone: match &found.lift {
            Lift::Zoned(at) => Some(at.zone.clone()),
            Lift::Local(_) => None,
        },
    }
}

#[cfg(test)]
#[path = "embargo_tests.rs"]
mod tests;
