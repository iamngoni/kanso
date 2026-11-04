//! The plain data of playing a recording on the phone (issue #38, step 5).

/// Where the bytes of a recording's parts stand on this device.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PartsState {
    /// Every part is here; it can play.
    Ready,
    /// A part's file is known, but its bytes haven't arrived by sync yet.
    NotHere,
    /// A part is gone, or sealed under a lock.
    Missing,
}

/// The recording a note holds, for the player card.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoteRecording {
    /// The part files, in playing order. The first is the file every
    /// timestamp link in the note names.
    pub file_ids: Vec<String>,
    pub state: PartsState,
    /// The whole recording's length, once its files can be read.
    pub length_ms: Option<u64>,
}

/// How playback stands, for the card: read about ten times a second while
/// something plays.
#[derive(Clone, Debug, PartialEq)]
pub struct PlayerStatus {
    pub position_ms: u64,
    pub length_ms: u64,
    pub playing: bool,
    /// It reached the end by itself; Play starts again from the top.
    pub ended: bool,
    pub speed: f32,
    pub volume: f32,
    /// Why it stopped, if the speaker failed.
    pub error: Option<String>,
}
