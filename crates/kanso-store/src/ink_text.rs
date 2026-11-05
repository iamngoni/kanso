//! Where handwriting recognition finds its canvases, and how it writes back.
//!
//! The recognized text lives in the canvas's own body (`kansoInkOcr`), or in
//! an inline sketch's, so it is sealed with the note, kept in its history and
//! synced with it. This is only the store's part: which notes may be read
//! (never a locked one, never one in a locked notebook, never one tagged
//! `#ai-exclude`, never one in the trash) and a body write that fails rather
//! than overwrite a newer edit.

use kanso_core::DocType;
use rusqlite::params;
use serde_json::json;

use crate::{Origin, Put, Result, Store, StoreError, schema};

/// A note that may hold handwriting, with the body recognition may read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InkNote {
    pub id: String,
    pub rev: String,
    pub body: String,
    /// A canvas note: the body is one canvas. Otherwise it is Markdown with
    /// `sketch` fences.
    pub canvas: bool,
}

impl Store {
    /// The canvases and the notes with an inline sketch that recognition may
    /// read, most recently edited first. A locked note (sealed, or in a
    /// locked notebook), one in the trash, one tagged `#ai-exclude` and a
    /// shared one (its text is merged live) are never offered, so their
    /// bodies are never read.
    pub fn ink_notes(&self) -> Result<Vec<InkNote>> {
        self.read(|conn| {
            let mut stmt = conn.prepare(concat!(
                "SELECT d.id, d.rev, json_extract(d.json, '$.doctype') = 'canvas', ",
                schema::readable_body!("d."),
                " FROM docs d WHERE ",
                schema::discoverable!("d."),
                " AND json_extract(d.json, '$.kansoShare') IS NULL",
                " AND (json_extract(d.json, '$.doctype') = 'canvas'
                       OR instr(json_extract(d.json, '$.body'), '```sketch') > 0)
                  ORDER BY d.updated_at DESC, d.id"
            ))?;
            let rows = stmt.query_map([], |row| {
                Ok(InkNote {
                    id: row.get(0)?,
                    rev: row.get(1)?,
                    canvas: row.get::<_, Option<bool>>(2)?.unwrap_or(false),
                    body: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                })
            })?;
            Ok(rows
                .collect::<rusqlite::Result<Vec<_>>>()?
                .into_iter()
                .filter(|n| !n.body.is_empty())
                .collect())
        })
    }

    /// Whether note `id` may be read for its handwriting right now: the same
    /// rule as [`ink_notes`](Self::ink_notes).
    pub fn ink_readable(&self, id: &str) -> Result<bool> {
        self.read(|conn| {
            Ok(conn.query_row(
                concat!(
                    "SELECT COUNT(*) FROM docs d WHERE d.id = ?1 AND ",
                    schema::discoverable!("d."),
                    " AND json_extract(d.json, '$.kansoShare') IS NULL"
                ),
                params![id],
                |row| row.get::<_, i64>(0),
            )? > 0)
        })
    }

    /// Replaces note `id`'s body with `body`, which carries the recognized
    /// text, if the note is still at `rev` and may still be read. Nothing
    /// else on the note changes, its date included. Whether it was written;
    /// `false` means the note changed, was locked or is gone, and is looked
    /// at again.
    pub fn set_ink_note_body(&self, id: &str, rev: &str, body: &str) -> Result<bool> {
        if !self.ink_readable(id)? {
            return Ok(false);
        }
        let fields = json!({"_id": id, "_rev": rev, "body": body});
        match self.put(DocType::Note, fields, Put::Merge, Origin::Ui) {
            Ok(_) => Ok(true),
            Err(StoreError::Conflict | StoreError::NotFound) => Ok(false),
            Err(err) => Err(err),
        }
    }
}
