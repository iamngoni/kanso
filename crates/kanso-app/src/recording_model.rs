//! The plain types of recording in the app (issue #38, step 4): what the
//! recording bar draws, what the notice strip under it can say, and the
//! transcription a quit interrupted.

use serde::{Deserialize, Serialize};

/// What the recording bar shows.
#[derive(Clone, Debug, PartialEq)]
pub struct BarState {
    /// Audio time recorded, pause excluded.
    pub seconds: u64,
    /// The level meter, 0 to 1.
    pub level: f32,
    pub paused: bool,
    /// "Microphone"
    pub label: &'static str,
    /// The red dot is lit this beat.
    pub dot_lit: bool,
}

/// A recording the last run left unfinished: a crash or power cut.
#[derive(Clone, Debug, PartialEq)]
pub struct CrashBanner {
    pub id: String,
    pub text: String,
}

/// A transcription a quit cut off. Written when one starts and removed when
/// it ends or is cancelled, so the next launch offers to finish it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingTranscript {
    pub note_id: String,
    pub file_ids: Vec<String>,
    pub duration_secs: u64,
}

/// What the strip under the title bar shows, most urgent first.
#[derive(Clone, Debug, PartialEq)]
pub enum Notice {
    Recording(BarState),
    Crash(CrashBanner),
    Pending(PendingTranscript),
}
