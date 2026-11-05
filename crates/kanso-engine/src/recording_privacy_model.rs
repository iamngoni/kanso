//! The plain types of recording privacy (issue #38, step 11): what happens
//! to a recording's audio once it is transcribed, and why deleting it can be
//! refused. Logic is in `recording_privacy.rs`.

use serde::{Deserialize, Serialize};

/// What becomes of a recording's audio after it is transcribed, chosen per
/// recording in the Record sheet.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeepAudio {
    /// The audio stays beside its transcript (the design's default: a
    /// recording is evidence, and deleting is irreversible).
    #[default]
    Keep,
    /// The audio is deleted once the transcript is in the note.
    TranscriptOnly,
}

impl KeepAudio {
    /// Whether the audio goes after the transcript.
    pub fn deletes_audio(self) -> bool {
        self == KeepAudio::TranscriptOnly
    }
}

/// Why a recording's audio was not deleted. Nothing is deleted in any of
/// these cases.
#[derive(Debug, thiserror::Error)]
pub enum DropError {
    /// The note doesn't hold a transcript of the recording yet (it failed,
    /// was cancelled, or isn't saved), so the audio is all there is.
    #[error("The transcript isn't in the note yet, so the audio is kept.")]
    NoTranscript,
    /// The note is gone.
    #[error("The note is gone, so the audio is kept.")]
    Missing,
    /// The note is locked and this session holds no key that opens it.
    #[error("The note is locked, so the audio is kept. Unlock it, then delete the audio.")]
    Locked,
    #[error("Couldn't delete the audio ({0}). It is kept.")]
    Store(String),
}

/// What deleting the audio did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dropped {
    /// The audio files deleted.
    pub files: usize,
}
