//! Rename Speakers (issue #38, step 13): the names before the paragraphs of
//! a recording's transcript, listed and changed in one write with an Undo
//! ticket.

use flutter_rust_bridge::frb;
use kanso_mobile::{KansoError, Library};

use super::recording_speakers_types::{SpeakerChange, SpeakerNameView, SpeakersRenamed};

#[frb(opaque)]
pub struct KansoRecordingSpeakers(Library);

impl KansoRecordingSpeakers {
    pub(super) fn new(library: Library) -> KansoRecordingSpeakers {
        KansoRecordingSpeakers(library)
    }

    /// The names before the paragraphs of note `id`'s first recording, in
    /// the order they first appear. Empty when there is none to rename.
    pub fn names(&self, id: String) -> Result<Vec<SpeakerNameView>, KansoError> {
        self.0.speaker_names(&id)
    }

    /// Writes `changes` to those names, in one change with an Undo ticket.
    pub fn rename(
        &self,
        id: String,
        changes: Vec<SpeakerChange>,
    ) -> Result<SpeakersRenamed, KansoError> {
        self.0.rename_speakers(&id, changes)
    }
}
