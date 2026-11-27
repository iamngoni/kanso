//! Making a notebook from a preset: the notebook, its property list and
//! board, its templates, the notebooks inside it with their notes, and its
//! tags, all in one write, so a failure leaves nothing behind and a sync
//! never sees half a layout.

use kanso_core::DocType;
use kanso_core::model::{BOOK_ICON_FIELD, TRASH_BOOK_ID};
use kanso_core::roles::Role;
use kanso_core::schema::{self, BOARD_FIELD, FIELD, STARTER_TEMPLATE_FIELD, TEMPLATES_FIELD};
use rusqlite::{OptionalExtension, Transaction};
use serde_json::{Map, Value, json};

use crate::preset_model::{NotebookPreset, PresetMade, PresetNote, PresetTag, PresetTemplate};
use crate::roles::{create, role_book_in};
use crate::validate::starter_key_ok;
use crate::write::{fresh_id, in_locked_book};
use crate::{Change, Origin, Result, Store, StoreError};

/// Most notebooks a preset makes inside its top one.
const MAX_CHILDREN: usize = 12;
/// Most tags a preset makes.
const MAX_TAGS: usize = 20;
/// Most notes a preset makes in one notebook.
const MAX_NOTES: usize = 12;
/// Longest icon name.
const MAX_ICON_LEN: usize = 40;

/// The refusal for a starter whose notes would land in a locked notebook.
pub const LOCKED_STARTER: &str =
    "Starters can't be made inside a locked notebook. Make it at the top level, then move it in.";

fn invalid<T>(message: impl Into<String>) -> Result<T> {
    Err(StoreError::Invalid(message.into()))
}

/// Checks one notebook of a preset (and, for the top one, its tags).
fn check(preset: &NotebookPreset, top: bool) -> Result<()> {
    if preset.name.trim().is_empty() {
        return invalid("A notebook needs a name.");
    }
    if preset
        .icon
        .as_deref()
        .is_some_and(|icon| icon.is_empty() || icon.len() > MAX_ICON_LEN)
    {
        return invalid("The notebook has an invalid icon.");
    }
    schema::validate(&preset.properties).map_err(|err| StoreError::Invalid(err.to_string()))?;
    if let Some(board) = &preset.board {
        schema::validate_board(board).map_err(|err| StoreError::Invalid(err.to_string()))?;
    }
    if preset.templates.len() > schema::MAX_TEMPLATES {
        return invalid("Too many templates.");
    }
    if preset
        .templates
        .iter()
        .any(|t| t.key.as_deref().is_some_and(|key| !starter_key_ok(key)))
    {
        return invalid("A template has an invalid key.");
    }
    if preset.notes.len() > MAX_NOTES {
        return invalid("Too many notes.");
    }
    if preset.children.len() > MAX_CHILDREN {
        return invalid("Too many notebooks.");
    }
    if preset.tags.len() > MAX_TAGS {
        return invalid("Too many tags.");
    }
    if !top && (!preset.children.is_empty() || !preset.tags.is_empty()) {
        return invalid("A notebook inside a preset can't hold notebooks or tags of its own.");
    }
    preset
        .children
        .iter()
        .try_for_each(|child| check(child, false))
}

/// The ID `sql` finds, if any.
fn find(tx: &Transaction, sql: &str, args: impl rusqlite::Params) -> Result<Option<String>> {
    Ok(tx.query_row(sql, args, |row| row.get(0)).optional()?)
}

/// The tag called `tag.name` (any case), or a new one.
fn tag_id(
    tx: &Transaction,
    tag: &PresetTag,
    origin: Origin,
    changes: &mut Vec<Change>,
) -> Result<String> {
    let name = tag.name.trim();
    let existing = find(
        tx,
        "SELECT id FROM docs WHERE type = 'tag' AND deleted = 0
         AND lower(name) = lower(?) ORDER BY id LIMIT 1",
        [name],
    )?;
    if let Some(id) = existing {
        return Ok(id);
    }
    let id = fresh_id(tx, DocType::Tag)?;
    let mut fields = Map::new();
    fields.insert("name".into(), name.into());
    fields.insert("color".into(), serde_json::to_value(tag.color)?);
    create(tx, DocType::Tag, &id, fields, origin, changes)?;
    Ok(id)
}

/// The template note for `template` in `templates_book`: the one a starter
/// made earlier under the same key, or a new one.
fn template_id(
    tx: &Transaction,
    templates_book: &str,
    template: &PresetTemplate,
    origin: Origin,
    changes: &mut Vec<Change>,
) -> Result<String> {
    if let Some(key) = &template.key {
        let existing = find(
            tx,
            &format!(
                "SELECT id FROM docs WHERE type = 'note' AND deleted = 0
                 AND book_id = ?1 AND book_id != '{TRASH_BOOK_ID}'
                 AND json_extract(json, '$.{STARTER_TEMPLATE_FIELD}') = ?2
                 ORDER BY id LIMIT 1"
            ),
            [templates_book, key.as_str()],
        )?;
        if let Some(id) = existing {
            return Ok(id);
        }
    }
    let id = fresh_id(tx, DocType::Note)?;
    let mut fields = Map::new();
    fields.insert("title".into(), template.title.clone().into());
    fields.insert("body".into(), template.body.clone().into());
    fields.insert("bookId".into(), templates_book.into());
    if let Some(key) = &template.key {
        fields.insert(STARTER_TEMPLATE_FIELD.into(), key.clone().into());
    }
    create(tx, DocType::Note, &id, fields, origin, changes)?;
    Ok(id)
}

fn note_fields(book_id: &str, note: &PresetNote) -> Map<String, Value> {
    let mut fields = Map::new();
    fields.insert("title".into(), note.title.clone().into());
    fields.insert("body".into(), note.body.clone().into());
    fields.insert("bookId".into(), book_id.into());
    fields.insert("status".into(), note.status.as_str().into());
    fields
}

/// Makes the notebook `preset` describes, naming `template_ids`.
fn make_book(
    tx: &Transaction,
    preset: &NotebookPreset,
    parent: Option<&str>,
    template_ids: &[String],
    origin: Origin,
    changes: &mut Vec<Change>,
) -> Result<String> {
    let book_id = fresh_id(tx, DocType::Book)?;
    let Value::Object(mut fields) = json!({
        "name": preset.name.trim(),
        "parentBookId": parent,
    }) else {
        unreachable!("a JSON object");
    };
    let optional = [
        (FIELD, schema::to_json(&preset.properties)),
        (BOARD_FIELD, schema::board_to_json(preset.board.as_ref())),
        (TEMPLATES_FIELD, schema::templates_to_json(template_ids)),
        (BOOK_ICON_FIELD, preset.icon.clone().map(Value::String)),
    ];
    for (field, value) in optional {
        if let Some(value) = value {
            fields.insert(field.into(), value);
        }
    }
    create(tx, DocType::Book, &book_id, fields, origin, changes)?;
    Ok(book_id)
}

fn makes_notes(preset: &NotebookPreset) -> bool {
    !preset.notes.is_empty() || preset.children.iter().any(makes_notes)
}

impl Store {
    /// Makes the notebook `preset` describes, with its templates in the
    /// Templates notebook (made if needed), the notebooks inside it, their
    /// notes and its tags. A refused name, parent, icon, property list,
    /// board or tree makes nothing, and so does a preset that makes notes
    /// inside a locked notebook.
    pub fn create_notebook_preset(
        &self,
        preset: &NotebookPreset,
        origin: Origin,
    ) -> Result<PresetMade> {
        check(preset, true)?;
        self.write(|tx, changes| {
            if makes_notes(preset) && in_locked_book(tx, preset.parent.as_deref())? {
                return invalid(LOCKED_STARTER);
            }
            let mut tag_ids = Vec::with_capacity(preset.tags.len());
            for tag in &preset.tags {
                tag_ids.push(tag_id(tx, tag, origin, changes)?);
            }
            let templates_book = role_book_in(tx, Role::Templates, origin, changes)?;
            let mut template_ids = Vec::new();
            let mut books = vec![preset];
            books.extend(preset.children.iter());
            let mut named = Vec::with_capacity(books.len());
            for book in &books {
                let mut ids = Vec::with_capacity(book.templates.len());
                for template in &book.templates {
                    ids.push(template_id(tx, &templates_book, template, origin, changes)?);
                }
                template_ids.extend(ids.iter().cloned());
                named.push(ids);
            }
            let book_id = make_book(
                tx,
                preset,
                preset.parent.as_deref(),
                &named[0],
                origin,
                changes,
            )?;
            let mut made = vec![book_id.clone()];
            for (child, ids) in preset.children.iter().zip(&named[1..]) {
                made.push(make_book(tx, child, Some(&book_id), ids, origin, changes)?);
            }
            let mut note_ids = Vec::new();
            for (book, id) in books.iter().zip(&made) {
                for note in &book.notes {
                    let note_id = fresh_id(tx, DocType::Note)?;
                    let fields = note_fields(id, note);
                    create(tx, DocType::Note, &note_id, fields, origin, changes)?;
                    note_ids.push(note_id);
                }
            }
            let child_ids = made[1..].to_vec();
            Ok(PresetMade {
                book_id,
                template_ids,
                child_ids,
                tag_ids,
                note_ids,
            })
        })
    }
}
