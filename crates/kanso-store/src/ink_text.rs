//! Where handwriting recognition finds its canvases, and how it writes back.
//!
//! The recognized text lives in the canvas's own body (`kansoInkOcr`), or in
//! an inline sketch's, so it is sealed with the note, kept in its history and
//! synced with it. This is only the store's part: which notes may be read
//! (never a locked one, never one in a locked notebook, never one tagged
//! `#ai-exclude`, never one in the trash, never a shared one) and a body
//! write that fails rather than overwrite a newer edit.
//!
//! Finding them is incremental: [`Store::ink_scan`] looks only at the notes
//! written since the change sequence a reader saw last, so a library of big
//! canvases is not read whole every time a note is saved.

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
    /// When the note was last edited (ms), for reading the newest first.
    pub updated_at: i64,
}

/// What changed since a change sequence, for handwriting recognition.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InkScan {
    /// The store's change sequence now: give it back next time.
    pub seq: i64,
    /// Every note written since that may be read for its handwriting, with
    /// its body.
    pub notes: Vec<InkNote>,
    /// Every other note written since (gone, locked, trashed, shared,
    /// excluded, or no longer holding ink): what was waiting for it is
    /// dropped.
    pub dropped: Vec<String>,
    /// A notebook or a tag changed since, which can change which notes may
    /// be read (a notebook locked or unlocked, a tag renamed to
    /// `ai-exclude`): look at every note again.
    pub rules_changed: bool,
}

/// The rule for a note recognition may read: discoverable, not shared, and a
/// canvas or a note with a `sketch` fence.
macro_rules! ink_readable {
    () => {
        concat!(
            "(",
            schema::discoverable!("d."),
            " AND json_extract(d.json, '$.kansoShare') IS NULL",
            " AND (json_extract(d.json, '$.doctype') = 'canvas'
                   OR instr(json_extract(d.json, '$.body'), '```sketch') > 0))"
        )
    };
}

impl Store {
    /// The notes written after change sequence `after` (0: every note), split
    /// into those recognition may read (with their bodies) and the rest. A
    /// locked note (sealed, or in a locked notebook), one in the trash, one
    /// tagged `#ai-exclude` and a shared one (its text is merged live) are
    /// never offered, so their bodies are never read.
    pub fn ink_scan(&self, after: i64) -> Result<InkScan> {
        self.read(|conn| {
            let seq: i64 =
                conn.query_row("SELECT IFNULL(MAX(seq), 0) FROM docs", [], |row| row.get(0))?;
            let rules_changed = after > 0
                && conn.query_row(
                    "SELECT EXISTS (SELECT 1 FROM docs WHERE seq > ?1 AND type IN ('book', 'tag'))",
                    params![after],
                    |row| row.get::<_, bool>(0),
                )?;
            let mut stmt = conn.prepare(concat!(
                "SELECT d.id, d.rev, json_extract(d.json, '$.doctype') = 'canvas', ",
                "CASE WHEN ",
                ink_readable!(),
                " THEN ",
                schema::readable_body!("d."),
                " END, IFNULL(d.updated_at, 0)",
                " FROM docs d WHERE d.seq > ?1 AND d.type = 'note'",
                " ORDER BY d.updated_at DESC, d.id"
            ))?;
            let rows = stmt.query_map(params![after], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<bool>>(2)?.unwrap_or(false),
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, i64>(4)?,
                ))
            })?;
            let mut scan = InkScan {
                seq,
                rules_changed,
                ..InkScan::default()
            };
            for row in rows {
                let (id, rev, canvas, body, updated_at) = row?;
                match body.filter(|b| !b.is_empty()) {
                    Some(body) => scan.notes.push(InkNote {
                        id,
                        rev,
                        body,
                        canvas,
                        updated_at,
                    }),
                    None => scan.dropped.push(id),
                }
            }
            Ok(scan)
        })
    }

    /// Note `id` with its body, if recognition may read it right now: the
    /// same rule as [`ink_scan`](Self::ink_scan).
    pub fn ink_note(&self, id: &str) -> Result<Option<InkNote>> {
        self.read(|conn| {
            let mut stmt = conn.prepare(concat!(
                "SELECT d.id, d.rev, json_extract(d.json, '$.doctype') = 'canvas', ",
                schema::readable_body!("d."),
                ", IFNULL(d.updated_at, 0) FROM docs d WHERE d.id = ?1 AND ",
                ink_readable!()
            ))?;
            let mut rows = stmt.query_map(params![id], |row| {
                Ok(InkNote {
                    id: row.get(0)?,
                    rev: row.get(1)?,
                    canvas: row.get::<_, Option<bool>>(2)?.unwrap_or(false),
                    body: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                    updated_at: row.get(4)?,
                })
            })?;
            Ok(rows.next().transpose()?.filter(|n| !n.body.is_empty()))
        })
    }

    /// Replaces note `id`'s body with `body`, which carries the recognized
    /// text, if the note is still at `rev` and may still be read. Nothing
    /// else on the note changes, its date included. The new revision when it
    /// was written; `None` means the note changed, was locked or is gone, and
    /// is looked at again.
    pub fn set_ink_note_body(&self, id: &str, rev: &str, body: &str) -> Result<Option<String>> {
        if self.ink_note(id)?.is_none() {
            return Ok(None);
        }
        let fields = json!({"_id": id, "_rev": rev, "body": body});
        match self.put(DocType::Note, fields, Put::Merge, Origin::Ui) {
            Ok(put) => Ok(Some(put.rev)),
            Err(StoreError::Conflict | StoreError::NotFound) => Ok(None),
            Err(err) => Err(err),
        }
    }
}
