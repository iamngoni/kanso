//! The plain data Inbox triage hands the shell (issue #195): cards, chips,
//! the settings and what filing did. No logic lives here; `triage.rs` builds
//! them from the engine's (`kanso_engine::triage`).

/// Which part of a note a chip would change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TriageField {
    Notebook,
    Tag,
    Title,
    Link,
}

/// A note a chip cites, or links to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TriageNote {
    pub id: String,
    pub title: String,
}

/// One suggestion on a card. The shell keeps whether it is accepted or
/// rejected; filing takes the accepted chips back whole.
#[derive(Clone, Debug, PartialEq)]
pub struct TriageChip {
    pub field: TriageField,
    /// A notebook ID, a tag ID (or a new tag's name when `is_new`), the
    /// title's text, or the target note's ID.
    pub value: String,
    /// What the chip shows.
    pub label: String,
    /// From 0 to 1.
    pub score: f32,
    /// Up to three notes that drove it.
    pub because: Vec<TriageNote>,
    /// Made by the person's AI (a sparkle).
    pub ai: bool,
    /// A tag that doesn't exist yet.
    pub is_new: bool,
}

/// Why a card has no notebook or tag chips.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TriageMissing {
    /// Nothing is missing.
    Nothing,
    /// Related notes are off.
    RelatedOff,
    /// The note isn't read yet.
    NotRead,
}

/// One Inbox note with its suggestions.
#[derive(Clone, Debug, PartialEq)]
pub struct TriageCard {
    pub note_id: String,
    /// The revision the suggestions were made from; filing is refused when
    /// the note has moved on.
    pub rev: String,
    pub title: String,
    pub snippet: String,
    pub updated_ms: i64,
    pub chips: Vec<TriageChip>,
    pub missing: TriageMissing,
}

/// How many notes wait, and what is switched on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TriageStatus {
    pub waiting: u32,
    /// Related notes are on: notebook and tag suggestions work.
    pub related_on: bool,
    /// AI is on and so is its Inbox switch: cards are refined by the model.
    pub ai_on: bool,
}

/// Settings › General › Inbox, and AI's Inbox switch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TriageSettings {
    pub show_bar: bool,
    pub suggest_titles: bool,
    pub suggest_links: bool,
    pub max_tags: u32,
    /// Let the model rank the suggestions (it sends the note and the names
    /// of likely notebooks and tags to the provider).
    pub use_ai: bool,
}

/// What was accepted for one note.
#[derive(Clone, Debug, PartialEq)]
pub struct TriageFiling {
    pub note_id: String,
    pub rev: String,
    pub accepted: Vec<TriageChip>,
}

/// What filing does (or did).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TriageFiled {
    /// The ticket `Library::undo` takes this batch back with; 0 for a plan
    /// that wasn't written, or when nothing was.
    pub undo: u64,
    pub filed: u32,
    /// Notes dropped because they changed since their card was read.
    pub stale: Vec<String>,
    pub moved: u32,
    pub tags: u32,
    pub titles: u32,
    pub links: u32,
    /// "Moves 4 to notebooks, adds 9 tags, 3 titles, 6 links."
    pub sentence: String,
}
