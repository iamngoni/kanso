//! A notebook's property list (`kansoProperties`): the properties its notes
//! are meant to have, each with a type (see `kanso_core::schema`).
//!
//! The list is a field of the notebook, so it syncs with it; two versions
//! made apart are united one property at a time (`sync.rs`). Saving a list
//! never rewrites a note: a value that does not fit is only warned about.

use kanso_core::DocType;
use kanso_core::properties;
use kanso_core::schema::{self, BOARD_FIELD, BoardSpec, FIELD, Mismatch, PropertyDef, SchemaError};
use serde_json::json;

use crate::{Origin, Put, Result, Store, StoreError, WriteResult};

impl Store {
    /// The property list notebook `book_id` defines, empty when it has none
    /// or the notebook is not there. Entries this version cannot read are
    /// skipped.
    pub fn notebook_properties(&self, book_id: &str) -> Result<Vec<PropertyDef>> {
        Ok(self
            .get(book_id)?
            .filter(|doc| doc.doc_type == DocType::Book)
            .map(|doc| schema::from_json(doc.fields.get(FIELD)))
            .unwrap_or_default())
    }

    /// Makes `defs` notebook `book_id`'s property list (an empty list clears
    /// it). Refused when the list would confuse a note: a property without a
    /// name, one defined twice, options on a property that is not a select.
    pub fn set_notebook_properties(
        &self,
        book_id: &str,
        defs: &[PropertyDef],
        origin: Origin,
    ) -> Result<WriteResult> {
        schema::validate(defs).map_err(|err: SchemaError| StoreError::Invalid(err.to_string()))?;
        if self
            .get(book_id)?
            .is_none_or(|doc| doc.doc_type != DocType::Book)
        {
            return Err(StoreError::NotFound);
        }
        self.put(
            DocType::Book,
            json!({"_id": book_id, FIELD: schema::to_json(defs)}),
            Put::Merge,
            origin,
        )
    }

    /// The board notebook `book_id` is laid out as, `None` when it has none
    /// (its notes are then laid out by status) or the notebook is not there.
    pub fn notebook_board(&self, book_id: &str) -> Result<Option<BoardSpec>> {
        Ok(self
            .get(book_id)?
            .filter(|doc| doc.doc_type == DocType::Book)
            .and_then(|doc| schema::board_from_json(doc.fields.get(BOARD_FIELD))))
    }

    /// Makes `spec` notebook `book_id`'s board (`None` clears it). Saving it
    /// never rewrites a note.
    pub fn set_notebook_board(
        &self,
        book_id: &str,
        spec: Option<&BoardSpec>,
        origin: Origin,
    ) -> Result<WriteResult> {
        if let Some(spec) = spec {
            schema::validate_board(spec)
                .map_err(|err: SchemaError| StoreError::Invalid(err.to_string()))?;
        }
        if self
            .get(book_id)?
            .is_none_or(|doc| doc.doc_type != DocType::Book)
        {
            return Err(StoreError::NotFound);
        }
        self.put(
            DocType::Book,
            json!({"_id": book_id, BOARD_FIELD: schema::board_to_json(spec)}),
            Put::Merge,
            origin,
        )
    }

    /// The warnings for note `note_id`: each property its notebook's list
    /// defines whose value does not fit the type. Empty when the notebook
    /// defines none. A note in the trash or one that is not there has none.
    pub fn note_property_warnings(&self, note_id: &str) -> Result<Vec<(String, Mismatch)>> {
        let Some(note) = self.note(note_id)? else {
            return Ok(Vec::new());
        };
        let defs = self.notebook_properties(&note.book_id)?;
        if defs.is_empty() {
            return Ok(Vec::new());
        }
        Ok(schema::check_all(&defs, &properties::parse(&note.body)))
    }
}
