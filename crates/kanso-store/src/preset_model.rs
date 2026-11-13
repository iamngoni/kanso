//! What a notebook preset is made of. The store makes one in a single
//! transaction ([`crate::Store::create_notebook_preset`]); which presets
//! exist is the engine's business.

use kanso_core::schema::{BoardSpec, PropertyDef};

/// A template note a preset puts in the Templates notebook.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PresetTemplate {
    pub title: String,
    pub body: String,
}

/// A notebook made with its property list, board and templates already set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NotebookPreset {
    pub name: String,
    /// The notebook it goes inside, none for the top level.
    pub parent: Option<String>,
    pub properties: Vec<PropertyDef>,
    pub board: Option<BoardSpec>,
    /// Made in the Templates notebook and named by the notebook, in order.
    pub templates: Vec<PresetTemplate>,
}

/// What a preset made.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PresetMade {
    pub book_id: String,
    /// The template notes' IDs, in the order the preset lists them.
    pub template_ids: Vec<String>,
}
