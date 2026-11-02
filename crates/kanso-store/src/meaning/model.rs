//! What the meaning features answer with.

use crate::{RelatedNote, WriteResult};

/// What a search by meaning found.
#[derive(Clone, Debug, PartialEq)]
pub enum MeaningSearch {
    /// Related notes are off: there are no embeddings to search.
    Off,
    /// Related notes are on, but this process has no model to turn the
    /// search into a vector: the app is still loading it, or the store was
    /// opened without the app (`kanso serve`).
    NoModel,
    /// The notes nearest the search, most similar first.
    Found(Vec<RelatedNote>),
}

/// A tag a note's nearest notes have and it hasn't.
#[derive(Clone, Debug, PartialEq)]
pub struct TagSuggestion {
    pub tag_id: String,
    pub name: String,
    /// The similarities of the near notes that have it, added up.
    pub weight: f32,
}

/// Two notes made one: the note kept, with the other's words and tags,
/// and the other, now in the trash.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Merged {
    pub kept: WriteResult,
    pub trashed: WriteResult,
}

/// How close a note must be to be resurfaced, by the cosine of the
/// writing's vector and the note's: the "How close" setting's three steps.
pub const SIMILARITY_CLOSER: f32 = 0.60;
pub const SIMILARITY_DEFAULT: f32 = 0.50;
pub const SIMILARITY_LOOSER: f32 = 0.42;

/// A match at least this close ...
pub const BEFORE_SIMILARITY: f32 = 0.60;
/// ... and written at least this long ago (180 days, in milliseconds) is
/// "You wrote about this before".
pub const BEFORE_AGE_MS: i64 = 180 * 24 * 60 * 60 * 1000;

/// What resurfacing is asked for.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResurfaceOpts {
    /// Notes below this similarity are left out.
    pub threshold: f32,
    /// At most this many notes.
    pub k: usize,
}

impl Default for ResurfaceOpts {
    fn default() -> Self {
        ResurfaceOpts {
            threshold: SIMILARITY_DEFAULT,
            k: 3,
        }
    }
}

/// A past note about what is being written now.
#[derive(Clone, Debug, PartialEq)]
pub struct Resurfaced {
    pub id: String,
    pub title: String,
    /// The name of its notebook; empty when it has none.
    pub book: String,
    /// When it was written, in milliseconds.
    pub created_at: i64,
    /// The cosine similarity, from -1 to 1.
    pub score: f32,
    /// The byte range of the passage that matched, in the note's body.
    pub passage: (usize, usize),
    /// The passage as one plain line, shortened.
    pub snippet: String,
}

impl Resurfaced {
    /// Whether this is "You wrote about this before": a close match
    /// written at least 180 days before `now_ms`.
    pub fn wrote_before(&self, now_ms: i64) -> bool {
        self.score >= BEFORE_SIMILARITY && now_ms.saturating_sub(self.created_at) >= BEFORE_AGE_MS
    }
}

/// What resurfacing found.
#[derive(Clone, Debug, PartialEq)]
pub enum Resurfacing {
    /// Related notes are off: there are no embeddings.
    Off,
    /// Related notes are on but this process has no model to read the
    /// writing with.
    NoModel,
    /// The notes about the writing, closest first.
    Found(Vec<Resurfaced>),
}

/// A note written on a day, for "On this day".
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DayNote {
    pub id: String,
    pub title: String,
    /// The name of its notebook; empty when it has none.
    pub book: String,
    pub created_at: i64,
}
