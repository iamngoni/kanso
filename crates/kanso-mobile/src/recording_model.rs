//! The plain data of recording on the phone (issue #38, step 4).

/// How a recording ended on its own.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordingEnd {
    /// It reached the 6-hour cap.
    Cap,
    /// The microphone stopped (taken by a call, unplugged).
    Device,
    /// The disk is full; what was written is kept.
    DiskFull,
}

/// What the recording bar shows.
#[derive(Clone, Debug, PartialEq)]
pub struct RecordingStatus {
    /// Audio time recorded, pause excluded.
    pub seconds: u64,
    /// The level meter, 0 to 1.
    pub level: f32,
    pub paused: bool,
    /// The 5-hour warning has been raised.
    pub warned: bool,
    /// The recorder stopped by itself; call stop to keep what it has.
    pub ended: Option<RecordingEnd>,
}

/// What stopping or recovering made.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordingSaved {
    pub note_id: String,
    pub file_ids: Vec<String>,
    pub duration_secs: u64,
    pub created: bool,
}

/// A recording a crash left behind.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordingFound {
    pub id: String,
    pub started_ms: i64,
    pub seconds: u64,
    /// The banner's text: "Kanso closed while recording at 14:05 (52 min)."
    pub text: String,
}
