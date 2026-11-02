//! What Triage hands the apps: a suggestion for one field of an Inbox note,
//! the options that shape them, and what filing did.

use serde::{Deserialize, Serialize};

/// Which part of a note a suggestion would change.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Field {
    Notebook,
    Tag,
    Title,
    Link,
}

impl Field {
    pub const ALL: [Field; 4] = [Field::Notebook, Field::Tag, Field::Title, Field::Link];

    /// The name rejections are kept under.
    pub fn as_str(self) -> &'static str {
        match self {
            Field::Notebook => "notebook",
            Field::Tag => "tag",
            Field::Title => "title",
            Field::Link => "link",
        }
    }

    pub fn parse(name: &str) -> Option<Field> {
        Field::ALL.into_iter().find(|f| f.as_str() == name)
    }
}

/// Where a suggestion came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Source {
    /// The library: embeddings, titles and links, with no model.
    Library,
    /// The person's own AI, ranking the library's candidates.
    Ai,
}

/// One chip on a card.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Suggestion {
    pub field: Field,
    /// A notebook ID, a tag ID (or a new tag's name when `is_new`), the
    /// title's text, or the target note's ID.
    pub value: String,
    /// What the chip shows.
    pub label: String,
    /// From 0 to 1.
    pub score: f32,
    /// Up to three note IDs that drove it.
    pub because: Vec<String>,
    pub source: Source,
    /// A tag that doesn't exist yet: filing makes it.
    pub is_new: bool,
}

/// How many notes a suggestion cites.
pub const BECAUSE_MAX: usize = 3;

/// What shapes the suggestions for a card (Settings › General › Inbox).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TriageOptions {
    /// The Inbox notebook's ID: never suggested as a target.
    pub inbox_id: String,
    pub suggest_titles: bool,
    pub suggest_links: bool,
    /// The most tags suggested for a note.
    pub max_tags: usize,
}

impl TriageOptions {
    pub fn new(inbox_id: impl Into<String>) -> TriageOptions {
        TriageOptions {
            inbox_id: inbox_id.into(),
            suggest_titles: true,
            suggest_links: true,
            max_tags: 3,
        }
    }
}

/// Why a card has no notebook or tag chips.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    /// Related notes are off: no embeddings at all.
    RelatedOff,
    /// The note isn't embedded yet.
    NotEmbedded,
}

/// A card: one Inbox note with what is suggested for it.
#[derive(Clone, Debug, PartialEq)]
pub struct Card {
    pub note_id: String,
    /// The revision the suggestions were made from; filing is refused when
    /// the note has moved on.
    pub rev: String,
    pub suggestions: Vec<Suggestion>,
    /// Everything the library found, including what was too weak to
    /// suggest: the notebooks, tags and links an AI may choose among.
    pub candidates: Vec<Suggestion>,
    /// Set when notebook and tag suggestions are not available.
    pub missing: Option<Reason>,
}

/// What a person accepted for one note.
#[derive(Clone, Debug, PartialEq)]
pub struct Filing {
    pub note_id: String,
    /// The revision the card was read at.
    pub rev: String,
    pub accepted: Vec<Suggestion>,
}

/// What filing a batch does (or did).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Filed {
    /// The notes written, each with the write that takes it back.
    pub edits: Vec<crate::batch::Edit>,
    /// The notes dropped because they changed since the card was read, were
    /// locked, or are gone.
    pub stale: Vec<String>,
    /// The tags this filing made (none existed by that name), so an undo
    /// can remove them again when nothing else uses them.
    pub new_tags: Vec<String>,
    pub moved: usize,
    pub tags: usize,
    pub titles: usize,
    pub links: usize,
}

impl Filed {
    /// The confirmation's words for a batch: "Moves 4 to notebooks, adds 9
    /// tags, 3 titles, 6 links." Pure.
    pub fn sentence(&self) -> String {
        let mut parts = Vec::new();
        if self.moved > 0 {
            parts.push(format!("moves {} to notebooks", self.moved));
        }
        if self.tags > 0 {
            parts.push(format!("adds {} tags", self.tags));
        }
        if self.titles > 0 {
            parts.push(format!("{} titles", self.titles));
        }
        if self.links > 0 {
            parts.push(format!("{} links", self.links));
        }
        if parts.is_empty() {
            return "Marks them filed.".to_owned();
        }
        let mut text = parts.join(", ");
        text.replace_range(..1, &text[..1].to_uppercase());
        text.push('.');
        text
    }
}
