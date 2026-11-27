//! What a notebook preset is made of. The store makes one in a single
//! transaction ([`crate::Store::create_notebook_preset`]); which presets
//! exist is the engine's business.

use kanso_core::schema::{BoardSpec, PropertyDef};
use kanso_core::{NoteStatus, TagColor};

/// A template note a preset puts in the Templates notebook.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PresetTemplate {
    pub title: String,
    pub body: String,
    /// A short key (`"product/bug-report"`) that marks the note, so the next
    /// preset that lists the same key reuses it instead of making a copy.
    /// None is always made new.
    pub key: Option<String>,
}

/// A note a preset makes in one of its notebooks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PresetNote {
    pub title: String,
    /// The text as it is stored: any template variables are already filled in.
    pub body: String,
    pub status: NoteStatus,
}

/// A tag a preset makes, unless one of that name (any case) already exists.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PresetTag {
    pub name: String,
    pub color: TagColor,
}

/// A notebook made with its property list, board, templates, notes and
/// notebooks inside it already set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NotebookPreset {
    pub name: String,
    /// The notebook it goes inside, none for the top level.
    pub parent: Option<String>,
    /// Its icon, by Lucide name.
    pub icon: Option<String>,
    pub properties: Vec<PropertyDef>,
    pub board: Option<BoardSpec>,
    /// Made in the Templates notebook and named by the notebook, in order.
    pub templates: Vec<PresetTemplate>,
    /// Made in this notebook, in order.
    pub notes: Vec<PresetNote>,
    /// Notebooks made inside this one, in order. One level only: a child
    /// can't have children of its own.
    pub children: Vec<NotebookPreset>,
    /// Tags to make. Read on the top notebook only.
    pub tags: Vec<PresetTag>,
}

impl NotebookPreset {
    /// A preset with a name and a parent and nothing else set.
    pub fn named(name: impl Into<String>, parent: Option<String>) -> Self {
        Self {
            name: name.into(),
            parent,
            icon: None,
            properties: Vec::new(),
            board: None,
            templates: Vec::new(),
            notes: Vec::new(),
            children: Vec::new(),
            tags: Vec::new(),
        }
    }
}

/// What a preset made.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PresetMade {
    pub book_id: String,
    /// The template notes' IDs (reused ones included): the top notebook's,
    /// then each child's, in the order the preset lists them.
    pub template_ids: Vec<String>,
    /// The notebooks inside, in order.
    pub child_ids: Vec<String>,
    /// The tags' IDs in the order listed; an existing tag's ID if one was reused.
    pub tag_ids: Vec<String>,
    /// The notes made, the top notebook's first, then each child's.
    pub note_ids: Vec<String>,
}
