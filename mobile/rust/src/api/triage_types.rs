//! The Inbox triage data types, declared again for the generator, as
//! `types.rs` does for the rest. What the fields mean is said where the
//! facade declares them (`crates/kanso-mobile/src/triage_model.rs`).

use flutter_rust_bridge::frb;
// Public: the generated code names these types by this module's path.
pub use kanso_mobile::{
    TriageCard, TriageChip, TriageField, TriageFiled, TriageFiling, TriageMissing, TriageNote,
    TriageSettings, TriageStatus,
};

#[frb(mirror(TriageField))]
pub enum _TriageField {
    Notebook,
    Tag,
    Title,
    Link,
}

#[frb(mirror(TriageNote))]
pub struct _TriageNote {
    pub id: String,
    pub title: String,
}

#[frb(mirror(TriageChip))]
pub struct _TriageChip {
    pub field: TriageField,
    pub value: String,
    pub label: String,
    pub score: f32,
    pub because: Vec<TriageNote>,
    pub ai: bool,
    pub is_new: bool,
}

#[frb(mirror(TriageMissing))]
pub enum _TriageMissing {
    Nothing,
    RelatedOff,
    NotRead,
}

#[frb(mirror(TriageCard))]
pub struct _TriageCard {
    pub note_id: String,
    pub rev: String,
    pub title: String,
    pub snippet: String,
    pub updated_ms: i64,
    pub chips: Vec<TriageChip>,
    pub missing: TriageMissing,
}

#[frb(mirror(TriageStatus))]
pub struct _TriageStatus {
    pub waiting: u32,
    pub related_on: bool,
    pub ai_on: bool,
}

#[frb(mirror(TriageSettings))]
pub struct _TriageSettings {
    pub show_bar: bool,
    pub suggest_titles: bool,
    pub suggest_links: bool,
    pub max_tags: u32,
    pub use_ai: bool,
}

#[frb(mirror(TriageFiling))]
pub struct _TriageFiling {
    pub note_id: String,
    pub rev: String,
    pub accepted: Vec<TriageChip>,
}

#[frb(mirror(TriageFiled))]
pub struct _TriageFiled {
    pub undo: u64,
    pub filed: u32,
    pub stale: Vec<String>,
    pub moved: u32,
    pub tags: u32,
    pub titles: u32,
    pub links: u32,
    pub sentence: String,
}
