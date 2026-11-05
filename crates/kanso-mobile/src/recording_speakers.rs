//! Rename Speakers on the phone (issue #38, step 13): the names before the
//! paragraphs of a recording's transcript, listed and changed behind the
//! bridge. The phone does not tell voices apart itself (it has no
//! on-device speech model); the names come with the transcript a computer
//! made, through sync, and the person fixes them here.
//!
//! The rules are the engine's (`kanso_engine::recording_speakers`), which
//! the desktop's sheet uses as well. The write is one change with an Undo
//! ticket.

use kanso_engine::recording_speakers::{SpeakersError, apply_renames, offer};
use kanso_store::Origin;

use crate::boundary::{guard, held};
use crate::errors::KansoError;
use crate::library::Library;
use crate::recording_speakers_model::{SpeakerChange, SpeakerNameView, SpeakersRenamed};

fn map(err: SpeakersError) -> KansoError {
    match err {
        SpeakersError::Missing => KansoError::NotFound,
        SpeakersError::Locked => KansoError::Locked,
        other => KansoError::invalid(other.to_string()),
    }
}

impl Library {
    /// The names before the paragraphs of note `id`'s first recording, in
    /// the order they first appear. Empty (not an error) when the note has
    /// none to rename: no transcript, one that does not say who spoke, a
    /// locked note or one in the trash. The shell shows the action only for
    /// a note that has some.
    pub fn speaker_names(&self, id: &str) -> Result<Vec<SpeakerNameView>, KansoError> {
        guard(|| match offer(&self.inner.store, id) {
            Ok(offered) => Ok(offered
                .names
                .into_iter()
                .map(|n| SpeakerNameView {
                    name: n.name,
                    paragraphs: n.paragraphs,
                })
                .collect()),
            Err(
                SpeakersError::Missing
                | SpeakersError::Locked
                | SpeakersError::NoTranscript
                | SpeakersError::NoSpeakers,
            ) => Ok(Vec::new()),
            Err(other) => Err(map(other)),
        })
    }

    /// Writes `changes` to the names in note `id`'s first recording's
    /// transcript, in one change with an Undo ticket. Each paragraph is
    /// matched by the name it has now, so swapping two names or naming one
    /// after another does not cascade. A note edited meanwhile is refused,
    /// not overwritten.
    pub fn rename_speakers(
        &self,
        id: &str,
        changes: Vec<SpeakerChange>,
    ) -> Result<SpeakersRenamed, KansoError> {
        guard(|| {
            let pairs: Vec<(String, String)> =
                changes.into_iter().map(|c| (c.from, c.to)).collect();
            let edit = apply_renames(&self.inner.store, id, &pairs, Origin::Ui).map_err(map)?;
            let undo = held(&self.inner.undo).keep(crate::undo::Undoable::Fields(vec![edit]));
            Ok(SpeakersRenamed { undo: Some(undo) })
        })
    }
}
