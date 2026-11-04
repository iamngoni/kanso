//! The plain types of a recording on show in a note (issue #38, steps 5 and
//! 6): which attachments make up the recording, and what a key does to the
//! player.

/// The attachments one line of a note links to, in playing order: the
/// recording the player plays as one timeline.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordingRef {
    /// File IDs, with their `file:` prefix.
    pub file_ids: Vec<String>,
    /// The line, from 0, in the note's body.
    pub line: usize,
    /// Whether Kanso wrote the line (`[Audio · …]`), so it is a recording
    /// for certain; any other line of file links is one only if its first
    /// file turns out to be audio.
    pub made_here: bool,
}

/// What a key does to a player that has the keyboard.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayerKey {
    PlayPause,
    /// Back or forward by this many seconds (negative is back).
    Skip(i32),
}
