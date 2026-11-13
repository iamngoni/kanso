//! Making a notebook from a preset, without a window. Blank is an empty
//! notebook. Video studio is a notebook with the properties a video
//! pipeline uses (`stage`, `embargo`, `publish`, `sponsor`, `owner`), a board
//! grouped by `stage` with its five lanes, and two templates (Video brief and
//! Shot list) the notebook names, so "New from template" lists them first
//! there. The store makes all of it in one transaction.

use kanso_core::DocType;
use kanso_core::schema::{BoardSpec, PropertyDef, PropertyType};
use kanso_store::{NotebookPreset, Origin, PresetTemplate, Put, Store};
use serde_json::json;

pub use crate::presets_model::{
    NotebookKind, SHOT_LIST, SHOT_LIST_BODY, STUDIO_STAGES, VIDEO_BRIEF, VIDEO_BRIEF_BODY,
};

/// A notebook made, with the templates a preset put in the Templates
/// notebook (none for Blank).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreatedNotebook {
    pub book_id: String,
    pub template_ids: Vec<String>,
}

fn def(key: &str, kind: PropertyType, options: &[&str]) -> PropertyDef {
    PropertyDef {
        key: key.into(),
        kind,
        options: options.iter().map(|o| (*o).to_owned()).collect(),
    }
}

/// The Video studio preset for a notebook called `name` inside `parent`.
pub fn video_studio(name: &str, parent: Option<String>) -> NotebookPreset {
    NotebookPreset {
        name: name.to_owned(),
        parent,
        properties: vec![
            def("stage", PropertyType::Select, &STUDIO_STAGES),
            def("embargo", PropertyType::DateTimeZone, &[]),
            def("publish", PropertyType::DateTime, &[]),
            def("sponsor", PropertyType::Text, &[]),
            def("owner", PropertyType::Person, &[]),
        ],
        board: Some(BoardSpec {
            by: "stage".into(),
            lanes: STUDIO_STAGES.iter().map(|s| (*s).to_owned()).collect(),
            show_unset: true,
        }),
        templates: vec![
            PresetTemplate {
                title: VIDEO_BRIEF.into(),
                body: VIDEO_BRIEF_BODY.into(),
            },
            PresetTemplate {
                title: SHOT_LIST.into(),
                body: SHOT_LIST_BODY.into(),
            },
        ],
    }
}

/// Makes a notebook called `name` (trimmed; empty is refused) inside
/// `parent` as `kind`. Video studio is made whole or not at all.
pub fn create_notebook(
    store: &Store,
    name: &str,
    parent: Option<String>,
    kind: NotebookKind,
    origin: Origin,
) -> kanso_store::Result<CreatedNotebook> {
    match kind {
        NotebookKind::Blank => {
            let name = name.trim();
            if name.is_empty() {
                return Err(kanso_store::StoreError::Invalid(
                    "A notebook needs a name.".into(),
                ));
            }
            let made = store.put(
                DocType::Book,
                json!({"name": name, "parentBookId": parent}),
                Put::Upsert,
                origin,
            )?;
            Ok(CreatedNotebook {
                book_id: made.id,
                template_ids: Vec::new(),
            })
        }
        NotebookKind::VideoStudio => {
            let made = store.create_notebook_preset(&video_studio(name, parent), origin)?;
            Ok(CreatedNotebook {
                book_id: made.book_id,
                template_ids: made.template_ids,
            })
        }
    }
}
