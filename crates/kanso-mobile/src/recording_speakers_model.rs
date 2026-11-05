//! The plain data of renaming the speakers of a recording's transcript on
//! the phone (issue #38, step 13). No logic lives here;
//! `recording_speakers.rs` builds and reads them.

/// One name before paragraphs of a transcript.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpeakerNameView {
    /// As written in the note: `You`, `Speaker 2`, `Priya`.
    pub name: String,
    /// How many paragraphs it begins.
    pub paragraphs: u32,
}

/// A name to change.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpeakerChange {
    /// The name now in the note.
    pub from: String,
    /// What to write instead. Two voices given the same name merge.
    pub to: String,
}

/// What renaming changed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpeakersRenamed {
    /// Undo ticket for the one write, or `None` when nothing was written.
    pub undo: Option<u64>,
}
