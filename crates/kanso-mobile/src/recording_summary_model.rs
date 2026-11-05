//! The plain data of a recording's summary on the phone (issue #38, step
//! 12). No logic lives here; `recording_summary.rs` builds them.

/// Whether Summarise Recording may be offered for a note, and what to say
/// before anything is sent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SummaryOffer {
    /// The note holds a recording's transcript, AI is on, and the note may
    /// be sent. When false the rest is empty.
    pub available: bool,
    /// The words that would be sent.
    pub words: u32,
    /// The sentence that names where the transcript goes and says the audio
    /// is not sent.
    pub send_line: String,
    /// The note already has a summary, which a new one replaces.
    pub replaces: bool,
}

/// A decision or action item the transcript backs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SummaryItemView {
    pub text: String,
    pub who: Option<String>,
    pub when: Option<String>,
    /// Seconds into the recording where it was said.
    pub moment: u32,
}

/// A summary to look over before anything is written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SummaryDraftView {
    /// The recording the moments are in.
    pub file_id: String,
    pub overview: Vec<String>,
    pub decisions: Vec<SummaryItemView>,
    pub actions: Vec<SummaryItemView>,
    /// Items left out for want of a quote from the recording.
    pub dropped: u32,
    /// Only the first part of a very long transcript was read.
    pub cut: bool,
}

/// What writing a summary did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SummaryApplied {
    /// An earlier summary was replaced.
    pub replaced: bool,
    /// The ticket for [`crate::Library::undo`].
    pub undo: Option<u64>,
}
