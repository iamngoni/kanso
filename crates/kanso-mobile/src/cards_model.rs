//! The plain data flashcards hand the shell (issue #198): cards to review,
//! counts, stats, settings, suggested cards and Quiz Me. No logic lives
//! here; `cards.rs` builds them from the store and the engine.

/// Which cards a session, a count or the stats cover.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardScopeKind {
    All,
    Notebook,
    Tag,
    Note,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardScopeArg {
    pub kind: CardScopeKind,
    /// The notebook, tag or note ID; empty for all.
    pub id: String,
}

/// A card to show. `intervals` reads "Again · 10 min", "Hard · …" in the
/// order of the four answers.
#[derive(Clone, Debug, PartialEq)]
pub struct ReviewCard {
    pub card_key: String,
    pub note_id: String,
    pub note_title: String,
    /// The line it is written on (0-based).
    pub line: u32,
    pub question: String,
    pub answer: String,
    pub is_new: bool,
    /// The next interval of each answer, as words ("4 days").
    pub intervals: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReviewCounts {
    pub total: u32,
    pub due: u32,
    pub new_cards: u32,
    pub suspended: u32,
    /// Due plus new within today's limits: the badge.
    pub waiting: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HardCardView {
    pub note_id: String,
    pub note_title: String,
    pub line: u32,
    pub question: String,
    pub agains: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ReviewStats {
    pub counts: ReviewCounts,
    /// Answers per day, the last 30 days, oldest first.
    pub per_day: Vec<u32>,
    /// Share of answers that weren't "Again" in those days, if any.
    pub retention: Option<f64>,
    /// Cards falling due on each of the next 7 days.
    pub forecast: Vec<u32>,
    pub hardest: Vec<HardCardView>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CardSettingsView {
    pub new_per_day: u32,
    pub reviews_per_day: u32,
    /// Target recall, 0.80 to 0.97.
    pub retention: f64,
    /// `HH:MM`, or none for off.
    pub reminder: Option<String>,
    pub show_icons: bool,
}

/// What `answer` leaves: the card's next due time and the cards now left.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Answered {
    pub due_ms: i64,
    pub waiting: u32,
}

/// A card of one note, for the editor's marks.
#[derive(Clone, Debug, PartialEq)]
pub struct NoteCardView {
    pub card_key: String,
    pub line: u32,
    pub question: String,
    pub answer: String,
    pub due_ms: Option<i64>,
    pub suspended: bool,
    /// Another card of the note has the same question.
    pub duplicate: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardPair {
    pub question: String,
    pub answer: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SuggestedCard {
    pub question: String,
    pub answer: String,
    pub source: Option<String>,
    pub line: Option<u32>,
    /// The quote isn't in the note: shown unticked.
    pub source_missing: bool,
}

/// What suggesting cards says as it works; ends with one of Done, Failed
/// or Stopped.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SuggestEvent {
    Status { line: String },
    Done { cards: Vec<SuggestedCard> },
    Failed { message: String },
    Stopped,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuizItem {
    pub question: String,
    pub note_id: String,
    pub note_title: String,
    pub source: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QuizEvent {
    Status { line: String },
    Questions { items: Vec<QuizItem> },
    Failed { message: String },
    Stopped,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuizVerdict {
    Right,
    Partly,
    Wrong,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GradeEvent {
    Graded {
        verdict: QuizVerdict,
        label: String,
        comment: String,
    },
    Failed {
        message: String,
    },
    Stopped,
}
