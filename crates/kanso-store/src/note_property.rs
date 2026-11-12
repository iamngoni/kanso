//! Setting one property in a note's front matter, in one write, and
//! putting the note back the way it was in one write: what moving a card
//! on a board grouped by a property does.

use kanso_core::model::LOCK_FIELD;
use kanso_core::properties::{self, Value};
use serde_json::Value as Json;

use crate::note_property_model::PropertyEdit;
use crate::write::{load_row, rewrite_note};
use crate::{Origin, Result, Store, StoreError};

impl Store {
    /// Sets property `key` of note `id` to `value`: the body changes only in
    /// that entry, and the note's edit time becomes `now_ms`. Nothing is
    /// written when the note already has that value. A sealed note is
    /// refused ([`StoreError::Invalid`]), a missing or trashed one is
    /// [`StoreError::NotFound`].
    pub fn set_note_property(
        &self,
        id: &str,
        key: &str,
        value: &Value,
        now_ms: i64,
        origin: Origin,
    ) -> Result<PropertyEdit> {
        self.write(|tx, changes| {
            let row = load_row(tx, id)?
                .filter(|r| !r.deleted)
                .ok_or(StoreError::NotFound)?;
            if row.fields.contains_key(LOCK_FIELD) {
                return Err(StoreError::Invalid("This note is locked.".into()));
            }
            let before = row
                .fields
                .get("body")
                .and_then(Json::as_str)
                .unwrap_or("")
                .to_owned();
            let before_updated_at = row
                .fields
                .get("updatedAt")
                .and_then(Json::as_i64)
                .unwrap_or(0);
            let after = properties::set_property(&before, key, value);
            let changed = after != before;
            if changed {
                let body = after.clone();
                rewrite_note(tx, id, origin, changes, |fields| {
                    fields.insert("body".into(), body.into());
                    fields.insert("updatedAt".into(), now_ms.into());
                })?;
            }
            Ok(PropertyEdit {
                note_id: id.to_owned(),
                before_body: before,
                before_updated_at,
                after_body: after,
                changed,
            })
        })
    }

    /// Puts note `edit.note_id` back as `edit` found it, in one write. Only
    /// while its body is still the one the edit made: a note edited since
    /// keeps its newer words and this says `false`.
    pub fn undo_note_property(&self, edit: &PropertyEdit, origin: Origin) -> Result<bool> {
        if !edit.changed {
            return Ok(true);
        }
        self.write(|tx, changes| {
            let Some(row) = load_row(tx, &edit.note_id)?.filter(|r| !r.deleted) else {
                return Ok(false);
            };
            let body = row.fields.get("body").and_then(Json::as_str).unwrap_or("");
            if body != edit.after_body || row.fields.contains_key(LOCK_FIELD) {
                return Ok(false);
            }
            rewrite_note(tx, &edit.note_id, origin, changes, |fields| {
                fields.insert("body".into(), edit.before_body.clone().into());
                fields.insert("updatedAt".into(), edit.before_updated_at.into());
            })?;
            Ok(true)
        })
    }
}
