//! The data of renaming speakers in a transcript.

/// One name before paragraphs of a transcript, and how many it begins.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpeakerName {
    /// As written in the note: `You`, `Speaker 2`, `Priya`.
    pub name: String,
    /// How many paragraphs it begins.
    pub paragraphs: u32,
}

/// The longest name Kanso writes before a paragraph.
pub const MAX_NAME_CHARS: usize = 40;

/// What the sheet shows for a note.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpeakerOffer {
    /// The recording whose transcript it is: its first attachment.
    pub file_id: String,
    /// The names before its paragraphs, in the order they first appear.
    pub names: Vec<SpeakerName>,
}

/// Why names could not be shown or changed.
#[derive(Debug, thiserror::Error)]
pub enum SpeakersError {
    #[error("That note isn't there any more.")]
    Missing,
    #[error(
        "A locked note's transcript can't be renamed here. Open it on a computer where it is unlocked."
    )]
    Locked,
    /// In the trash, or tagged `#ai-exclude`: renaming leaves it alone.
    #[error(
        "This note is in the trash or tagged #ai-exclude, so Kanso leaves its transcript alone."
    )]
    KeptOut,
    #[error("This note has no recording with a transcript.")]
    NoTranscript,
    #[error("This transcript doesn't say who spoke.")]
    NoSpeakers,
    #[error("Those names are already what the note has.")]
    NothingToChange,
    #[error("The note changed while the names were being edited. Try again.")]
    Changed,
    #[error("{0}")]
    Store(String),
}

impl From<kanso_store::StoreError> for SpeakersError {
    fn from(err: kanso_store::StoreError) -> Self {
        SpeakersError::Store(err.to_string())
    }
}
