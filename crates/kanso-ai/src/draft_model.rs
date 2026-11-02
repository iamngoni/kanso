//! The shapes drafting, suggested cards and Quiz Me pass around.

use crate::prompt::Excerpt;

/// What kind of writing a draft is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    Report,
    Essay,
    Update,
    Outline,
    Custom,
}

impl Shape {
    pub const ALL: [Shape; 5] = [
        Shape::Report,
        Shape::Essay,
        Shape::Update,
        Shape::Outline,
        Shape::Custom,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Shape::Report => "Report",
            Shape::Essay => "Essay",
            Shape::Update => "Update",
            Shape::Outline => "Outline",
            Shape::Custom => "Custom",
        }
    }

    pub(crate) fn instruction(self) -> &'static str {
        match self {
            Shape::Report => {
                "Write a report: a title line, a short summary, then sections with headings."
            }
            Shape::Essay => "Write an essay: a title line, then flowing paragraphs with no lists.",
            Shape::Update => {
                "Write a short update: what happened, what is next, and anything that is blocked."
            }
            Shape::Outline => "Write an outline: nested Markdown bullets under short headings.",
            Shape::Custom => "Write what the request asks for, in the shape it implies.",
        }
    }

    pub fn from_name(name: &str) -> Option<Shape> {
        Self::ALL
            .into_iter()
            .find(|s| s.label().eq_ignore_ascii_case(name.trim()))
    }
}

/// How long a draft should be.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Length {
    Short,
    Medium,
    Long,
}

impl Length {
    pub const ALL: [Length; 3] = [Length::Short, Length::Medium, Length::Long];

    pub fn words(self) -> usize {
        match self {
            Length::Short => 300,
            Length::Medium => 800,
            Length::Long => 2_000,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Length::Short => "Short",
            Length::Medium => "Medium",
            Length::Long => "Long",
        }
    }

    pub fn from_name(name: &str) -> Option<Length> {
        Self::ALL
            .into_iter()
            .find(|l| l.label().eq_ignore_ascii_case(name.trim()))
    }
}

/// A source note with the number the draft cites it by.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Source {
    pub number: usize,
    pub excerpt: Excerpt,
}

/// How a draft will be written from its sources.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Plan {
    /// Every source fits in one prompt.
    Single,
    /// Each of these sources (by index) is read on its own first.
    Map {
        reads: Vec<usize>,
        /// Sources left unread because there were more than the reads allow.
        left_out: usize,
    },
}

/// A citation as it stands in the checked draft.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cite {
    /// The source's number (1-based).
    pub number: usize,
    /// A heading of that note, when the draft named one that exists.
    pub heading: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpanKind {
    Cited(Vec<Cite>),
    /// No valid citation: "No source in your notes".
    Unsourced,
}

/// A stretch of the checked draft's text (byte offsets, on char boundaries).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub kind: SpanKind,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CheckedDraft {
    /// The draft with invalid citations removed and `[n]` kept as text.
    pub text: String,
    pub spans: Vec<Span>,
    /// Citations removed because no such source was given.
    pub removed: usize,
}

/// A card the model suggested for a note.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Suggestion {
    pub question: String,
    pub answer: String,
    /// The short quote the model says the card comes from.
    pub source: Option<String>,
    /// The line of the note the quote was found on.
    pub line: Option<u32>,
    /// The quote was not found in the note (the card starts unticked).
    pub source_missing: bool,
}

/// A question Quiz Me asks, from one excerpt.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuizQuestion {
    pub question: String,
    /// The excerpt's number the question comes from (1-based).
    pub excerpt: usize,
    pub source: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Right,
    Partly,
    Wrong,
}

impl Verdict {
    pub fn label(self) -> &'static str {
        match self {
            Verdict::Right => "Right",
            Verdict::Partly => "Partly",
            Verdict::Wrong => "Not quite",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Graded {
    pub verdict: Verdict,
    /// One sentence citing the excerpt.
    pub comment: String,
}
