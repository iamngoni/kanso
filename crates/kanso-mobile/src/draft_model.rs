//! The plain data drafting hands the shell (issue #198). No logic lives
//! here; `draft.rs` builds them from the engine's (`kanso_engine::draft`).

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DraftShape {
    Report,
    Essay,
    Update,
    Outline,
    Custom,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DraftLength {
    Short,
    Medium,
    Long,
}

/// What the Sources part of the sheet lists.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DraftSourceNote {
    pub id: String,
    pub title: String,
    pub words: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DraftSources {
    pub notes: Vec<DraftSourceNote>,
    pub left_out_locked: u32,
    pub left_out_excluded: u32,
    pub too_many: bool,
    /// "12 notes, about 9,000 words. 2 locked notes left out."
    pub summary: String,
}

/// A source as the draft numbers it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DraftSourceRef {
    pub number: u32,
    pub id: String,
    pub title: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DraftCite {
    pub number: u32,
    pub heading: Option<String>,
}

/// A stretch of the draft's text, in UTF-16 units (what Dart indexes by).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DraftSpan {
    pub start: u32,
    pub end: u32,
    /// Empty for an unsourced stretch.
    pub cites: Vec<DraftCite>,
    pub unsourced: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DraftView {
    /// The draft with citations kept as `[n]` text.
    pub text: String,
    pub spans: Vec<DraftSpan>,
    /// Citations dropped because no such source was given.
    pub removed: u32,
    /// The model said the notes don't cover the request.
    pub not_covered: bool,
}

/// What writing a draft says as it goes; ends with one of Done, CutOff,
/// Failed or Stopped.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DraftEvent {
    Status {
        line: String,
    },
    Text {
        draft: DraftView,
    },
    Done {
        draft: DraftView,
        sources: Vec<DraftSourceRef>,
    },
    CutOff {
        message: String,
    },
    Failed {
        message: String,
    },
    Stopped,
}

/// The note a draft would become, for the review step.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DraftPreview {
    pub title: String,
    pub markdown: String,
    pub default_book_id: String,
}
