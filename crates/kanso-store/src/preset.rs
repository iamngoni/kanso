//! Making a notebook from a preset: the notebook, its property list and
//! board, and its templates, all in one write, so a failure leaves nothing
//! behind and a sync never sees half a studio.

use kanso_core::DocType;
use kanso_core::roles::Role;
use kanso_core::schema::{self, BOARD_FIELD, FIELD, TEMPLATES_FIELD};
use serde_json::{Map, Value, json};

use crate::preset_model::{NotebookPreset, PresetMade};
use crate::roles::{create, role_book_in};
use crate::write::fresh_id;
use crate::{Origin, Result, Store, StoreError};

impl Store {
    /// Makes the notebook `preset` describes, with its templates in the
    /// Templates notebook (made if needed). A refused name, parent, property
    /// list or board makes nothing.
    pub fn create_notebook_preset(
        &self,
        preset: &NotebookPreset,
        origin: Origin,
    ) -> Result<PresetMade> {
        let name = preset.name.trim();
        if name.is_empty() {
            return Err(StoreError::Invalid("A notebook needs a name.".into()));
        }
        schema::validate(&preset.properties).map_err(|err| StoreError::Invalid(err.to_string()))?;
        if let Some(board) = &preset.board {
            schema::validate_board(board).map_err(|err| StoreError::Invalid(err.to_string()))?;
        }
        if preset.templates.len() > schema::MAX_TEMPLATES {
            return Err(StoreError::Invalid("Too many templates.".into()));
        }
        self.write(|tx, changes| {
            let templates_book = role_book_in(tx, Role::Templates, origin, changes)?;
            let mut template_ids = Vec::with_capacity(preset.templates.len());
            for template in &preset.templates {
                let id = fresh_id(tx, DocType::Note)?;
                let mut fields = Map::new();
                fields.insert("title".into(), template.title.clone().into());
                fields.insert("body".into(), template.body.clone().into());
                fields.insert("bookId".into(), templates_book.clone().into());
                create(tx, DocType::Note, &id, fields, origin, changes)?;
                template_ids.push(id);
            }
            let book_id = fresh_id(tx, DocType::Book)?;
            let Value::Object(mut fields) = json!({
                "name": name,
                "parentBookId": preset.parent,
            }) else {
                unreachable!("a JSON object");
            };
            let optional = [
                (FIELD, schema::to_json(&preset.properties)),
                (BOARD_FIELD, schema::board_to_json(preset.board.as_ref())),
                (TEMPLATES_FIELD, schema::templates_to_json(&template_ids)),
            ];
            for (field, value) in optional {
                if let Some(value) = value {
                    fields.insert(field.into(), value);
                }
            }
            create(tx, DocType::Book, &book_id, fields, origin, changes)?;
            Ok(PresetMade {
                book_id,
                template_ids,
            })
        })
    }
}
