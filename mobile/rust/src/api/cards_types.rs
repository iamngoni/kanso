//! The flashcard data types, declared again for the generator, as
//! `types.rs` does for the rest. What the fields mean is said where the
//! facade declares them (`crates/kanso-mobile/src/cards_model.rs`).

use flutter_rust_bridge::frb;
// Public: the generated code names these types by this module's path.
pub use kanso_mobile::{
    Answered, CardPair, CardScopeArg, CardScopeKind, CardSettingsView, GradeEvent, HardCardView,
    NoteCardView, QuizEvent, QuizItem, QuizVerdict, ReviewCard, ReviewCounts, ReviewStats,
    SuggestEvent, SuggestedCard,
};

#[frb(mirror(CardScopeKind))]
pub enum _CardScopeKind {
    All,
    Notebook,
    Tag,
    Note,
}

#[frb(mirror(CardScopeArg))]
pub struct _CardScopeArg {
    pub kind: CardScopeKind,
    pub id: String,
}

#[frb(mirror(ReviewCard))]
pub struct _ReviewCard {
    pub card_key: String,
    pub note_id: String,
    pub note_title: String,
    pub line: u32,
    pub question: String,
    pub answer: String,
    pub is_new: bool,
    pub intervals: Vec<String>,
}

#[frb(mirror(ReviewCounts))]
pub struct _ReviewCounts {
    pub total: u32,
    pub due: u32,
    pub new_cards: u32,
    pub suspended: u32,
    pub waiting: u32,
}

#[frb(mirror(HardCardView))]
pub struct _HardCardView {
    pub note_id: String,
    pub note_title: String,
    pub line: u32,
    pub question: String,
    pub agains: u32,
}

#[frb(mirror(ReviewStats))]
pub struct _ReviewStats {
    pub counts: ReviewCounts,
    pub per_day: Vec<u32>,
    pub retention: Option<f64>,
    pub forecast: Vec<u32>,
    pub hardest: Vec<HardCardView>,
}

#[frb(mirror(CardSettingsView))]
pub struct _CardSettingsView {
    pub new_per_day: u32,
    pub reviews_per_day: u32,
    pub retention: f64,
    pub reminder: Option<String>,
    pub show_icons: bool,
}

#[frb(mirror(Answered))]
pub struct _Answered {
    pub due_ms: i64,
    pub waiting: u32,
}

#[frb(mirror(NoteCardView))]
pub struct _NoteCardView {
    pub card_key: String,
    pub line: u32,
    pub question: String,
    pub answer: String,
    pub due_ms: Option<i64>,
    pub suspended: bool,
    pub duplicate: bool,
}

#[frb(mirror(CardPair))]
pub struct _CardPair {
    pub question: String,
    pub answer: String,
}

#[frb(mirror(SuggestedCard))]
pub struct _SuggestedCard {
    pub question: String,
    pub answer: String,
    pub source: Option<String>,
    pub line: Option<u32>,
    pub source_missing: bool,
}

#[frb(mirror(SuggestEvent))]
pub enum _SuggestEvent {
    Status { line: String },
    Done { cards: Vec<SuggestedCard> },
    Failed { message: String },
    Stopped,
}

#[frb(mirror(QuizItem))]
pub struct _QuizItem {
    pub question: String,
    pub note_id: String,
    pub note_title: String,
    pub source: Option<String>,
}

#[frb(mirror(QuizEvent))]
pub enum _QuizEvent {
    Status { line: String },
    Questions { items: Vec<QuizItem> },
    Failed { message: String },
    Stopped,
}

#[frb(mirror(QuizVerdict))]
pub enum _QuizVerdict {
    Right,
    Partly,
    Wrong,
}

#[frb(mirror(GradeEvent))]
pub enum _GradeEvent {
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
