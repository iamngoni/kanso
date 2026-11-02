//! The shapes of a voice capture (issue #193).

use std::path::PathBuf;

pub use kanso_core::capture_mark::{Engine, TidyState};

/// A recording longer than this is a meeting or lecture, unless the person
/// chose otherwise.
pub const MEETING_AFTER_MS: u64 = 3 * 60_000;

/// A stretch of recognised speech with its time, for engines that give times.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Segment {
    pub text: String,
    pub start_ms: u64,
    pub end_ms: u64,
}

/// One finished capture, ready to become a note.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VoiceCapture {
    /// The capture's ID: 32 lowercase hex characters, like a share's drop ID
    /// (`is_drop_id`). The note is `note:cap-<id>`, so saving the same
    /// capture twice makes one note.
    pub id: String,
    pub started_ms: i64,
    pub duration_ms: u64,
    /// What was said, as the recogniser gave it.
    pub transcript: String,
    /// The same words with their times, when the engine has them; pauses
    /// longer than two seconds then become paragraph breaks.
    pub segments: Vec<Segment>,
    pub engine: Engine,
    /// The recording, kept only when `keep_audio` is asked for.
    pub audio_file: Option<PathBuf>,
}

/// What [`super::save_voice_capture`] made.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SavedCapture {
    /// The note's ID.
    pub id: String,
    /// Said to the person when something was left out (the audio was too
    /// large, say). The note is saved either way.
    pub problems: Vec<String>,
    /// The note was already there from an earlier save.
    pub existed: bool,
}
