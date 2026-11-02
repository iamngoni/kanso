//! The data Ask passes between its steps: what a question is asked under
//! (scope, time, budget), what retrieval found, and what a checked answer
//! holds. No logic beyond small accessors; the steps are in the sibling
//! modules.

use std::ops::Range;

use kanso_ai::{Budget, Excerpt};
use kanso_core::dates::Date;
use kanso_core::when::TimeIntent;

pub use kanso_ai::SentExcerpt as ShownExcerpt;

/// What a question searches. `Book` includes the notebooks inside it.
/// `Notes` is a page's own notes (notebook homepages, people and projects).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum AskScope {
    #[default]
    All,
    /// A notebook by ID, and every notebook inside it.
    Book(String),
    /// A tag by name, as the person writes it.
    Tag(String),
    /// These notes, by ID.
    Notes(Vec<String>),
}

impl AskScope {
    /// Whether the scope is narrower than All notes.
    pub fn is_narrow(&self) -> bool {
        !matches!(self, AskScope::All)
    }
}

/// What a scope was refused for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScopeRefusal {
    /// The scope names a locked notebook.
    Locked,
}

impl ScopeRefusal {
    /// What the person is told.
    pub fn message(self) -> &'static str {
        match self {
            ScopeRefusal::Locked => {
                "This notebook is locked. Unlock it to see it, but Ask never reads locked notes."
            }
        }
    }
}

/// What came before a follow-up: its question, the titles of the notes the
/// answer cited and the passages it cited, so the follow-up can be
/// searched as a question of its own and can cite them again.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Previous {
    pub question: String,
    pub titles: Vec<String>,
    /// `(note id, first line, last line)`, 0-based, of what it cited.
    pub cited: Vec<(String, u32, u32)>,
}

/// One question, with everything it is asked under.
#[derive(Clone, Debug)]
pub struct AskRequest {
    pub question: String,
    pub scope: AskScope,
    /// The day it is asked, and the first day of the person's week (0 for
    /// Monday), for "last week" and "in October".
    pub today: Date,
    pub week_start: usize,
    /// The zone's offset from UTC in milliseconds, so a local day is cut
    /// where the person's day is.
    pub utc_offset_ms: i64,
    pub budget: Budget,
    /// Whether links and backlinks of the best notes are looked up too.
    pub use_links: bool,
    /// The person dropped the time range the question was understood to have.
    pub ignore_time: bool,
    pub previous: Option<Previous>,
    /// The follow-up as a model rewrote it into a stand-alone query, when the
    /// local rewrite found nothing.
    pub rewrite: Option<String>,
}

impl AskRequest {
    /// A request for `question` over all notes, today (UTC), with the
    /// standard budget.
    pub fn new(question: &str, today: Date) -> Self {
        AskRequest {
            question: question.to_owned(),
            scope: AskScope::All,
            today,
            week_start: 0,
            utc_offset_ms: 0,
            budget: Budget::default(),
            use_links: true,
            ignore_time: false,
            previous: None,
            rewrite: None,
        }
    }
}

/// The time a question was understood to be about, for the chip in the
/// answer's header.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimeNote {
    /// "October 2026".
    pub label: String,
    pub intent: TimeIntent,
    /// Too few notes were written in the range, so the closest notes are
    /// shown and the range only ranked them.
    pub relaxed: bool,
}

impl TimeNote {
    /// "No notes written in October 2026 matched; showing the closest."
    pub fn relaxed_line(&self) -> Option<String> {
        self.relaxed.then(|| {
            format!(
                "No notes written in {} matched; showing the closest.",
                self.label
            )
        })
    }
}

/// A passage found for a question, for "Closest notes".
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Closest {
    pub note_id: String,
    pub title: String,
    /// 0-based, inclusive.
    pub start: u32,
    pub end: u32,
}

/// What retrieval found for one question.
#[derive(Clone, Debug, Default)]
pub struct Retrieval {
    /// The notes to send, best first, their lines numbered.
    pub excerpts: Vec<Excerpt>,
    /// The time understood, when there was one.
    pub time: Option<TimeNote>,
    /// The words that were searched (the question without its time words,
    /// and for a follow-up the earlier subject).
    pub query: String,
    /// The best three passages, shown when the model says the notes do not
    /// answer.
    pub closest: Vec<Closest>,
}

impl Retrieval {
    /// Whether nothing matched, so no model is asked.
    pub fn is_empty(&self) -> bool {
        self.excerpts.is_empty()
    }
}

/// One receipt that survived checking: the lines an answer's sentence rests
/// on, as they were sent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Receipt {
    /// Its number in the answer, from 1, in order of first use.
    pub n: usize,
    pub note_id: String,
    pub title: String,
    /// The day the note last changed, as it was sent.
    pub updated: Option<String>,
    /// The first and last line cited, 0-based and inclusive.
    pub start: u32,
    pub end: u32,
    /// The cited lines as they were sent, joined with `\n`.
    pub quote: String,
}

/// An answer after its receipts were checked.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Checked {
    /// The answer with valid markers written as `[1](kanso-cite:1)` links
    /// and invalid ones removed.
    pub markdown: String,
    pub receipts: Vec<Receipt>,
    /// The sentences, as byte ranges of `markdown`, whose receipts failed.
    pub unsupported: Vec<Range<usize>>,
    /// The answer was exactly the sentinel: the notes do not answer.
    pub not_in_notes: bool,
    /// Sentences that stated something from the notes, to judge the share
    /// that failed: those with at least one marker, valid or not.
    pub claims: usize,
}

impl Checked {
    /// Whether more than half of the claims failed their check, so the whole
    /// answer gets a banner.
    pub fn mostly_unchecked(&self) -> bool {
        self.claims > 0 && self.unsupported.len() * 2 > self.claims
    }

    /// Whether a non-empty answer carries no receipt at all (not even a
    /// failed one): the model did not follow the format.
    pub fn has_no_sources(&self) -> bool {
        !self.not_in_notes && self.claims == 0 && !self.markdown.trim().is_empty()
    }
}

/// The link scheme a receipt's number is written with.
pub const CITE_PREFIX: &str = "kanso-cite:";
/// The link scheme that marks a sentence whose receipts failed.
pub const UNVERIFIED_PREFIX: &str = "kanso-unverified:";
/// What an unverified sentence is labelled with.
pub const NOT_FOUND_LABEL: &str = "Not found in your notes";
/// What hovering an unverified sentence explains.
pub const NOT_FOUND_HINT: &str = "The answer said this, but the lines it pointed to aren't in the notes Kanso sent. Treat it as unverified.";
/// The banner over an answer most of whose claims failed.
pub const MOSTLY_UNCHECKED: &str = "Most of this answer couldn't be checked against your notes.";
/// The banner over an answer with no sources at all.
pub const NO_SOURCES: &str = "This answer has no sources. Your model may not follow citation instructions; try a larger model.";
/// What Ask says when the notes do not hold the answer.
pub const NEVER_WROTE: &str = "You never wrote that down.";
/// What it says under that when nothing matched at all.
pub const NOTHING_MATCHES: &str = "Nothing in your notes matches this question.";
/// While receipts are being checked.
pub const CHECKING: &str = "Checking the sources…";
/// While retrieval runs.
pub const SEARCHING: &str = "Searching your notes…";
/// While the model writes.
pub const WRITING: &str = "Writing the answer…";
/// The notes quietly changed since the answer.
pub const LINES_CHANGED: &str = "These lines have changed since the answer was written.";

/// "Reading 6 notes…".
pub fn reading_line(notes: usize) -> String {
    format!(
        "Reading {notes} {}…",
        if notes == 1 { "note" } else { "notes" }
    )
}
