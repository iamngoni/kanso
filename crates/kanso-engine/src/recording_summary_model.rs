//! The data a recording's summary is made of (issue #38, step 12).

use std::fmt;

use kanso_store::StoreError;

/// One piece of the transcript as the summary reads it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SummarySegment {
    /// Seconds from the start of the recording.
    pub start_secs: u32,
    /// The words of the paragraph, without its timestamp link.
    pub text: String,
    /// Where the paragraph starts in [`SummarySource::said`], in characters.
    pub from: usize,
}

/// What a summary starts from, read once.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SummarySource {
    pub id: String,
    pub title: String,
    /// The recording whose transcript is summarised: the first one in the
    /// note.
    pub file_id: String,
    /// The day it was recorded, as a meeting heading says it.
    pub date: Option<String>,
    pub segments: Vec<SummarySegment>,
    /// The words that were said, paragraph after paragraph: what a quote is
    /// looked for in.
    pub said: String,
    /// What the model reads: each paragraph behind its time in brackets.
    pub prompt_text: String,
    /// The words that will be sent: those before the cap when the
    /// transcript is longer than it.
    pub words: usize,
    /// The transcript is longer than the cap, so only its first part is
    /// sent.
    pub cut: bool,
    /// How many recordings with a transcript the note holds. Only the
    /// first one's is summarised.
    pub recordings: usize,
    /// The note already holds a summary, which a new one replaces.
    pub replaces: bool,
}

/// A decision or an action item the transcript backs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SummaryItem {
    pub text: String,
    /// Who, only when a name was said.
    pub who: Option<String>,
    /// When, only when it was said, in the words said.
    pub when: Option<String>,
    /// Seconds into the recording where the quote that backs it was said.
    pub moment: u32,
}

/// A summary with every item checked against the transcript.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SummaryDraft {
    /// The recording the moments are in.
    pub file_id: String,
    pub overview: Vec<String>,
    pub decisions: Vec<SummaryItem>,
    pub actions: Vec<SummaryItem>,
    /// Items left out for want of a quote in the transcript.
    pub dropped: usize,
    /// Only the first part of a transcript longer than the cap was read.
    pub cut: bool,
}

/// Why a note wasn't summarised.
#[derive(Debug)]
pub enum SummaryError {
    /// The note isn't there.
    Missing,
    /// The note holds no transcript of a recording.
    NoTranscript,
    /// Locked, trashed or `#ai-exclude`.
    KeptOut,
    /// The note changed while the model worked and the write was refused.
    Changed,
    /// The model gave nothing the transcript backs.
    Empty,
    Store(StoreError),
}

impl fmt::Display for SummaryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SummaryError::Missing => f.write_str("That note isn't there any more."),
            SummaryError::NoTranscript => {
                f.write_str("This note has no transcript of a recording to summarise.")
            }
            SummaryError::KeptOut => f.write_str(crate::assist::KEPT_OUT),
            SummaryError::Changed => f.write_str("The note changed while it was being summarised."),
            SummaryError::Empty => f.write_str("There was nothing in the transcript to summarise."),
            SummaryError::Store(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for SummaryError {}

impl From<StoreError> for SummaryError {
    fn from(err: StoreError) -> SummaryError {
        SummaryError::Store(err)
    }
}

/// What writing a summary into the note did.
#[derive(Clone, Debug, PartialEq)]
pub struct Applied {
    pub id: String,
    /// An earlier summary was replaced.
    pub replaced: bool,
    /// The write that puts the note back as it was, for Undo.
    pub edit: crate::batch::Edit,
}
