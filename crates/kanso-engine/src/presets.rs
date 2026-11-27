//! Making a notebook from a preset, without a window. Blank is an empty
//! notebook. A starter (Product development, Knowledge base, Side projects,
//! Daily journal, Reading, Writing & publishing) is a notebook with an icon,
//! notebooks inside it with their own icons, tags, a template for each kind
//! of note (named by its notebook, so "New from template" lists it first
//! there) and one note in each notebook made from that template; see
//! [`NotebookKind::starter`]. Video studio is a notebook with the properties a
//! video pipeline uses (`stage`, `embargo`, `publish`, `sponsor`, `owner`), a
//! board grouped by `stage` with its five lanes, and two templates (Video
//! brief and Shot list). The store makes all of it in one transaction.

use chrono::{Datelike as _, NaiveDateTime, Timelike as _};
use kanso_core::dates::{Date, DateTime};
use kanso_core::schema::{BoardSpec, PropertyDef, PropertyType};
use kanso_core::template::{self, Vars};
use kanso_core::{DocType, NoteStatus};
use kanso_store::{NotebookPreset, Origin, PresetNote, PresetTag, PresetTemplate, Put, Store};
use serde_json::json;

pub use crate::presets_model::{
    NotebookKind, SHOT_LIST, SHOT_LIST_BODY, STUDIO_STAGES, Starter, StarterBook, StarterTag,
    StarterTemplate, VIDEO_BRIEF, VIDEO_BRIEF_BODY,
};

/// A notebook made, with what a preset put in and beside it (nothing for
/// Blank).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreatedNotebook {
    pub book_id: String,
    /// The templates in the Templates notebook, reused ones included.
    pub template_ids: Vec<String>,
    /// The notebooks inside, in order.
    pub child_ids: Vec<String>,
    /// The tags a starter lists, in order; an existing tag's ID if one was
    /// reused.
    pub tag_ids: Vec<String>,
    /// The notes made, one in each notebook inside.
    pub note_ids: Vec<String>,
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
    let mut preset = NotebookPreset::named(name, parent);
    preset.properties = vec![
        def("stage", PropertyType::Select, &STUDIO_STAGES),
        def("embargo", PropertyType::DateTimeZone, &[]),
        def("publish", PropertyType::DateTime, &[]),
        def("sponsor", PropertyType::Text, &[]),
        def("owner", PropertyType::Person, &[]),
    ];
    preset.board = Some(BoardSpec {
        by: "stage".into(),
        lanes: STUDIO_STAGES.iter().map(|s| (*s).to_owned()).collect(),
        show_unset: true,
    });
    preset.templates = vec![
        PresetTemplate {
            title: VIDEO_BRIEF.into(),
            body: VIDEO_BRIEF_BODY.into(),
            key: None,
        },
        PresetTemplate {
            title: SHOT_LIST.into(),
            body: SHOT_LIST_BODY.into(),
            key: None,
        },
    ];
    preset
}

/// The starter `kind` makes for a notebook called `name` inside `parent`, as
/// of local time `now`: each note's text has its template's variables filled
/// in. None for Blank and Video studio.
pub fn starter_preset(
    kind: NotebookKind,
    name: &str,
    parent: Option<String>,
    now: NaiveDateTime,
) -> Option<NotebookPreset> {
    let starter = kind.starter(now.date())?;
    let when = DateTime {
        date: Date::new(now.year(), now.month(), now.day())?,
        hour: now.hour(),
        minute: now.minute(),
        second: now.second(),
    };
    let mut preset = NotebookPreset::named(name, parent);
    preset.icon = Some(kind.icon().to_owned());
    preset.tags = starter
        .tags
        .iter()
        .map(|tag| PresetTag {
            name: tag.name.to_owned(),
            color: tag.color,
        })
        .collect();
    preset.children = starter
        .children
        .iter()
        .map(|book| {
            let mut child = NotebookPreset::named(book.name.clone(), None);
            child.icon = Some(book.icon.to_owned());
            child.templates = book
                .templates
                .iter()
                .map(|template| PresetTemplate {
                    title: template.title.to_owned(),
                    body: template.body.to_owned(),
                    key: Some(template.key.to_owned()),
                })
                .collect();
            if let Some(first) = book.templates.first() {
                let vars = Vars {
                    now: when.clone(),
                    title: book.note_title.clone(),
                };
                child.notes = vec![PresetNote {
                    title: book.note_title.clone(),
                    body: template::expand(first.body, &vars).text,
                    status: NoteStatus::Active,
                }];
            }
            child
        })
        .collect();
    Some(preset)
}

/// Makes a notebook called `name` (trimmed; empty is refused) inside
/// `parent` as `kind`, as of the local time now. A starter or Video studio is
/// made whole or not at all; a starter that makes notes is refused inside a
/// locked notebook.
pub fn create_notebook(
    store: &Store,
    name: &str,
    parent: Option<String>,
    kind: NotebookKind,
    origin: Origin,
) -> kanso_store::Result<CreatedNotebook> {
    let now = chrono::Local::now().naive_local();
    create_notebook_at(store, name, parent, kind, origin, now)
}

/// [`create_notebook`] as of local time `now`.
pub fn create_notebook_at(
    store: &Store,
    name: &str,
    parent: Option<String>,
    kind: NotebookKind,
    origin: Origin,
    now: NaiveDateTime,
) -> kanso_store::Result<CreatedNotebook> {
    let preset = match kind {
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
            return Ok(CreatedNotebook {
                book_id: made.id,
                template_ids: Vec::new(),
                child_ids: Vec::new(),
                tag_ids: Vec::new(),
                note_ids: Vec::new(),
            });
        }
        NotebookKind::VideoStudio => video_studio(name, parent),
        starter => starter_preset(starter, name, parent, now)
            .ok_or_else(|| kanso_store::StoreError::Invalid("The starter has no layout.".into()))?,
    };
    let made = store.create_notebook_preset(&preset, origin)?;
    Ok(CreatedNotebook {
        book_id: made.book_id,
        template_ids: made.template_ids,
        child_ids: made.child_ids,
        tag_ids: made.tag_ids,
        note_ids: made.note_ids,
    })
}
