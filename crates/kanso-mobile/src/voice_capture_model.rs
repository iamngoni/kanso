//! The plain data of voice capture on the phone (issue #193). No logic lives
//! here; `voice_capture.rs` builds and reads them.

/// A capture to save: the words, and when and how long they were spoken.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VoiceCaptureInput {
    /// 32 lowercase hex characters; empty to make a fresh one. Saving the
    /// same ID twice makes one note.
    pub id: String,
    pub started_ms: i64,
    pub duration_ms: u64,
    /// What was said, as the recogniser gave it.
    pub transcript: String,
    /// Whether the recogniser kept the audio on the phone. `false` is the
    /// server-backed recogniser, and is written on the note.
    pub on_device: bool,
}

/// What saving a capture made.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VoiceCaptureSaved {
    /// The note, or `None` when nothing was heard (nothing was saved).
    pub note_id: Option<String>,
    /// What was left out, a sentence each (audio over the attachment limit).
    pub problems: Vec<String>,
    /// The note was already there from an earlier save.
    pub existed: bool,
    /// The recording is a meeting (longer than three minutes, with meetings
    /// on), so its tidy-up has the meeting's shape.
    pub meeting: bool,
    /// Settings say to tidy it now and AI is on: the shell tidies it.
    pub tidy_now: bool,
}

/// A recording the lock screen, an App Intent or the Android tile left,
/// waiting to be transcribed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VoiceDrop {
    pub id: String,
    /// The audio file, to give to the recogniser.
    pub audio_path: String,
    pub started_ms: i64,
    pub duration_ms: u64,
}

/// What the shell needs to show a voice note's Tidy banner.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VoiceNoteInfo {
    /// "none", "pending" or "done".
    pub tidy: String,
    pub duration_ms: u64,
    /// Tidy is on, AI is set up, and the note may be sent to it.
    pub can_tidy: bool,
}

/// What a tidy did to a note.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TidyOutcome {
    /// The note had been edited meanwhile, so the tidy part went above the
    /// person's text instead of replacing it.
    pub inserted_above: bool,
    /// Decisions and tasks left out for want of a quote from the recording.
    pub dropped: u32,
}
