//! The shapes the flashcard store functions take and return.

use kanso_core::srs::CardState;
use serde::{Deserialize, Serialize};

/// Which cards a session or a count covers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CardScope {
    All,
    /// A notebook and the notebooks inside it.
    Book(String),
    Tag(String),
    Note(String),
}

/// Settings > Flashcards. Local to the device, except that the limits
/// count answers from every device (the log syncs).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CardSettings {
    pub new_per_day: u32,
    pub reviews_per_day: u32,
    /// Target recall, 0.80 to 0.97.
    pub retention: f64,
    /// `HH:MM` of the daily reminder; `None` is off.
    pub reminder: Option<String>,
    pub show_icons: bool,
}

impl Default for CardSettings {
    fn default() -> Self {
        CardSettings {
            new_per_day: 20,
            reviews_per_day: 200,
            retention: kanso_core::srs::DEFAULT_RETENTION,
            reminder: None,
            show_icons: true,
        }
    }
}

/// A card to show in a session.
#[derive(Clone, Debug, PartialEq)]
pub struct DueCard {
    pub card_key: String,
    pub note_id: String,
    pub note_title: String,
    /// The line it is written on (0-based).
    pub line: u32,
    pub question: String,
    pub answer: String,
    pub is_new: bool,
    pub state: CardState,
}

/// A card of one note with its schedule, for the editor's gutter.
#[derive(Clone, Debug, PartialEq)]
pub struct NoteCard {
    pub card_key: String,
    pub line: u32,
    pub question: String,
    pub answer: String,
    /// `None` for a card never answered.
    pub due_ms: Option<i64>,
    pub suspended: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CardCounts {
    pub total: usize,
    /// Answered before and due now.
    pub due: usize,
    pub new: usize,
    pub suspended: usize,
}

/// One of the hardest cards: the most "Again" answers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HardCard {
    pub card_key: String,
    pub note_id: String,
    pub note_title: String,
    pub line: u32,
    pub question: String,
    pub agains: u32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CardStats {
    pub counts: CardCounts,
    /// Answers per day for the last 30 days, oldest first.
    pub per_day: Vec<u32>,
    /// Share of the last 30 days' answers that weren't "Again", if any.
    pub retention: Option<f64>,
    /// Cards falling due on each of the next 7 days (today first, with
    /// what is overdue).
    pub forecast: Vec<u32>,
    pub hardest: Vec<HardCard>,
}
