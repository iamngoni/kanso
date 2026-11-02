//! Writes a plugin makes: field-level merges checked against the revision
//! the plugin read, and new notes, applied as one batch in one transaction
//! and undone as one.
//!
//! A plugin names only `title`, `body`, `tags`, `status` and its own data on
//! the note (`kansoPlugin["<plugin id>"]`); every other field is kept as it
//! is. A note changed since the plugin read it fails the whole batch with
//! [`StoreError::Conflict`], the same rule the REST API and the editor
//! follow. A locked note, or a note in a locked notebook, is refused
//! whatever the fields: this is the store's own wall behind the plugin
//! router's, which never shows the plugin such a note at all.
//!
//! Each note's earlier version is kept in its history under the batch ID
//! with origin `plugin` and the plugin's name as its author, never
//! throttled; a created note is kept as a marker so undo can remove it.

use kanso_core::DocType;
use kanso_core::model::{LOCK_FIELD, TRASH_BOOK_ID};
use rusqlite::params;
use serde_json::{Map, Value};

use crate::replace::Skipped;
use crate::write::{
    Row, SNAPSHOTS_PER_DOC, Synced, WriteResult, fresh_id, in_locked_book, load_row, next_rev,
    store_row,
};
use crate::{Origin, Result, Store, StoreError};

/// Where every plugin keeps its data on a note, by plugin ID.
pub const PLUGIN_DATA_FIELD: &str = "kansoPlugin";

/// The marker a created note's history entry holds instead of a version.
const CREATED: &str = "null";

/// A change to one note, as a plugin asked for it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PluginEdit {
    pub id: String,
    /// The revision the plugin read the note at.
    pub base_rev: String,
    pub title: Option<String>,
    pub body: Option<String>,
    /// Tag IDs.
    pub tags: Option<Vec<String>>,
    pub status: Option<String>,
    /// The plugin's own data on the note: `Some(None)` removes it.
    pub data: Option<Option<Value>>,
    /// Move the note to this notebook (an agent's move, or a merge's source
    /// going to the trash). Never set by a plugin.
    pub book_id: Option<String>,
}

/// A note a plugin makes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginCreate {
    pub book_id: String,
    pub title: String,
    pub body: String,
}

/// Who is writing: the plugin, and the batch its command opened.
#[derive(Clone, Copy, Debug)]
pub struct PluginWriter<'a> {
    pub plugin_id: &'a str,
    /// Shown in history as the version's author.
    pub plugin_name: &'a str,
    pub batch: &'a str,
}

/// Who writes a batch: a plugin, or an agent whose changes the person
/// approved. Both are applied and undone by the same code.
#[derive(Clone, Copy, Debug)]
pub enum BatchWriter<'a> {
    Plugin(PluginWriter<'a>),
    Agent {
        /// Shown in history as the version's author ("Claude Agent").
        agent: &'a str,
        /// The batch ID, see [`agent_batch_id`].
        batch: &'a str,
    },
}

impl<'a> BatchWriter<'a> {
    fn batch(&self) -> &'a str {
        match self {
            Self::Plugin(w) => w.batch,
            Self::Agent { batch, .. } => batch,
        }
    }
    fn author(&self) -> &'a str {
        match self {
            Self::Plugin(w) => w.plugin_name,
            Self::Agent { agent, .. } => agent,
        }
    }
    fn plugin_id(&self) -> &'a str {
        match self {
            Self::Plugin(w) => w.plugin_id,
            Self::Agent { .. } => "",
        }
    }
    fn origin(&self) -> Origin {
        match self {
            Self::Plugin(_) => Origin::Plugin,
            Self::Agent { .. } => Origin::Agent,
        }
    }
    fn origin_name(&self) -> &'static str {
        match self {
            Self::Plugin(_) => "plugin",
            Self::Agent { .. } => "agent",
        }
    }
    fn who(&self) -> &'static str {
        match self {
            Self::Plugin(_) => "a plugin",
            Self::Agent { .. } => "an agent",
        }
    }
}

/// A fresh batch ID for an agent task: `agent:` and 12 random bytes in hex.
pub fn agent_batch_id() -> String {
    let mut bytes = [0u8; 12];
    getrandom::fill(&mut bytes).expect("the system's random source");
    let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    format!("agent:{hex}")
}

/// What [`Store::apply_plugin_batch`] did.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PluginReport {
    pub changed: Vec<WriteResult>,
    pub created: Vec<WriteResult>,
}

/// The fields a plugin edit sets, merged over `fields`. Pure.
fn merged(fields: &Map<String, Value>, edit: &PluginEdit, plugin_id: &str) -> Map<String, Value> {
    let mut fields = fields.clone();
    if let Some(title) = &edit.title {
        fields.insert("title".into(), title.clone().into());
    }
    if let Some(body) = &edit.body {
        fields.insert("body".into(), body.clone().into());
    }
    if let Some(tags) = &edit.tags {
        fields.insert("tags".into(), tags.clone().into());
    }
    if let Some(status) = &edit.status {
        fields.insert("status".into(), status.clone().into());
    }
    if let Some(data) = &edit.data {
        set_plugin_data(&mut fields, plugin_id, data.clone());
    }
    if let Some(book) = &edit.book_id {
        fields.insert("bookId".into(), book.clone().into());
    }
    fields
}

/// Sets (or with `None` removes) `plugin_id`'s data on a note's fields,
/// leaving other plugins' alone and dropping the field once it is empty.
/// Pure.
pub fn set_plugin_data(fields: &mut Map<String, Value>, plugin_id: &str, data: Option<Value>) {
    let mut all = match fields.remove(PLUGIN_DATA_FIELD) {
        Some(Value::Object(all)) => all,
        _ => Map::new(),
    };
    match data {
        Some(value) => {
            all.insert(plugin_id.to_owned(), value);
        }
        None => {
            all.remove(plugin_id);
        }
    }
    if !all.is_empty() {
        fields.insert(PLUGIN_DATA_FIELD.into(), Value::Object(all));
    }
}

/// `plugin_id`'s data on a note's fields. Pure.
pub fn plugin_data<'a>(fields: &'a Map<String, Value>, plugin_id: &str) -> Option<&'a Value> {
    fields.get(PLUGIN_DATA_FIELD)?.get(plugin_id)
}

/// Refuses any plugin write to a note that is locked or in a locked
/// notebook, or that isn't live.
fn guard(tx: &rusqlite::Transaction, row: &Row, who: &str) -> Result<()> {
    if row.deleted || row.doc_type != DocType::Note {
        return Err(StoreError::NotFound);
    }
    let book = row.fields.get("bookId").and_then(Value::as_str);
    if row.fields.contains_key(LOCK_FIELD) || in_locked_book(tx, book)? {
        return Err(StoreError::Invalid(format!(
            "The note is locked: {who} can't change it."
        )));
    }
    Ok(())
}

/// Keeps `before` in note `id`'s history under `writer`'s batch, as the
/// version `rev` replaced (or, with `before` `None`, marks `id` created by
/// the batch).
fn keep(
    tx: &rusqlite::Transaction,
    id: &str,
    before: Option<(&str, &Map<String, Value>)>,
    rev: &str,
    writer: BatchWriter,
) -> Result<()> {
    let (old_rev, json) = match before {
        Some((old_rev, fields)) => (old_rev.to_owned(), serde_json::to_string(fields)?),
        None => (rev.to_owned(), CREATED.to_owned()),
    };
    tx.execute(
        "INSERT INTO revisions (doc_id, rev, json, saved_at, origin, author, batch, batch_rev)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            id,
            old_rev,
            json,
            crate::history::version_time(tx, id)?,
            writer.origin_name(),
            writer.author(),
            writer.batch(),
            rev
        ],
    )?;
    tx.execute(
        "DELETE FROM revisions WHERE doc_id = ?1 AND id NOT IN
         (SELECT id FROM revisions WHERE doc_id = ?1 ORDER BY saved_at DESC, id DESC LIMIT ?2)",
        params![id, SNAPSHOTS_PER_DOC],
    )?;
    Ok(())
}

fn capitalised(s: &str) -> String {
    let mut c = s.chars();
    c.next()
        .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
        .unwrap_or_default()
}

impl Store {
    /// Applies a plugin's `edits` and `creates` in one transaction: every
    /// note changes, or none does. A note not at its `base_rev` is
    /// [`StoreError::Conflict`]; a locked note, a note in a locked notebook
    /// or a missing one fails too. New notes go only into live, unlocked
    /// notebooks other than the trash.
    pub fn apply_plugin_batch(
        &self,
        writer: PluginWriter,
        edits: &[PluginEdit],
        creates: &[PluginCreate],
        now_ms: i64,
    ) -> Result<PluginReport> {
        self.apply_batch(BatchWriter::Plugin(writer), edits, creates, now_ms)
    }

    /// [`Store::apply_plugin_batch`] for either kind of writer. An agent's
    /// batch writes with `Origin::Agent` and names the agent in each
    /// replaced version's history entry.
    pub fn apply_batch(
        &self,
        writer: BatchWriter,
        edits: &[PluginEdit],
        creates: &[PluginCreate],
        now_ms: i64,
    ) -> Result<PluginReport> {
        self.write(|tx, changes| {
            let mut report = PluginReport::default();
            for edit in edits {
                let row = load_row(tx, &edit.id)?.ok_or(StoreError::NotFound)?;
                guard(tx, &row, writer.who())?;
                if row.rev != edit.base_rev {
                    return Err(StoreError::Conflict);
                }
                let mut fields = merged(&row.fields, edit, writer.plugin_id());
                if fields == row.fields {
                    continue;
                }
                fields.insert("updatedAt".into(), now_ms.into());
                let fields = crate::validate::normalize(
                    DocType::Note,
                    &edit.id,
                    fields,
                    Some(&row.fields),
                    now_ms,
                    crate::validate::Refs::Check(tx),
                )?;
                let rev = next_rev(Some(&row), &Value::Object(fields.clone()))?;
                keep(tx, &edit.id, Some((&row.rev, &row.fields)), &rev, writer)?;
                store_row(
                    tx,
                    &edit.id,
                    DocType::Note,
                    &rev,
                    false,
                    &fields,
                    Some(&row),
                    Synced::Keep,
                    writer.origin(),
                    changes,
                )?;
                report.changed.push(WriteResult {
                    id: edit.id.clone(),
                    rev,
                });
            }
            for create in creates {
                let book = load_row(tx, &create.book_id)?
                    .filter(|b| !b.deleted && b.doc_type == DocType::Book)
                    .ok_or(StoreError::NotFound)?;
                if create.book_id == TRASH_BOOK_ID
                    || book.fields.contains_key(LOCK_FIELD)
                    || in_locked_book(tx, Some(&create.book_id))?
                {
                    return Err(StoreError::Invalid(format!(
                        "{} can't make notes in that notebook.",
                        capitalised(writer.who())
                    )));
                }
                let id = fresh_id(tx, DocType::Note)?;
                let mut fields = Map::new();
                fields.insert("bookId".into(), create.book_id.clone().into());
                fields.insert("title".into(), create.title.clone().into());
                fields.insert("body".into(), create.body.clone().into());
                fields.insert("createdAt".into(), now_ms.into());
                fields.insert("updatedAt".into(), now_ms.into());
                let fields = crate::validate::normalize(
                    DocType::Note,
                    &id,
                    fields,
                    None,
                    now_ms,
                    crate::validate::Refs::Check(tx),
                )?;
                let rev = next_rev(None, &Value::Object(fields.clone()))?;
                store_row(
                    tx,
                    &id,
                    DocType::Note,
                    &rev,
                    false,
                    &fields,
                    None,
                    Synced::Keep,
                    writer.origin(),
                    changes,
                )?;
                keep(tx, &id, None, &rev, writer)?;
                report.created.push(WriteResult { id, rev });
            }
            if matches!(writer, BatchWriter::Agent { .. }) {
                for done in &report.changed {
                    tx.execute(
                        "UPDATE revisions SET agent = ?1 WHERE doc_id = ?2 AND origin = 'agent' AND agent IS NULL",
                        params![writer.author(), done.id],
                    )?;
                }
            }
            Ok(report)
        })
    }

    /// Takes back plugin batch `batch` in one transaction: each note gets
    /// back the title, text, tags, status and plugin data it had, and each
    /// note the batch made is removed, unless it changed again since (or is
    /// gone): those are left as they are and reported. A batch is undone
    /// once.
    pub fn undo_plugin_batch(&self, batch: &str, now_ms: i64) -> Result<crate::UndoReport> {
        self.undo_batch(batch, now_ms)
    }

    /// [`Store::undo_plugin_batch`] for a plugin's or an agent's batch. A
    /// restored note is stamped `updatedAt = now_ms`, never its old time.
    pub fn undo_batch(&self, batch: &str, now_ms: i64) -> Result<crate::UndoReport> {
        self.undo_batch_inner(batch, now_ms, false)
    }

    /// [`Store::undo_batch`] for an agent task's batch: a note the batch made
    /// goes to the Trash instead of being deleted, so Undo never destroys
    /// text for good.
    pub fn undo_agent_batch(&self, batch: &str, now_ms: i64) -> Result<crate::UndoReport> {
        self.undo_batch_inner(batch, now_ms, true)
    }

    fn undo_batch_inner(
        &self,
        batch: &str,
        now_ms: i64,
        trash_created: bool,
    ) -> Result<crate::UndoReport> {
        self.write(|tx, changes| {
            let kept: Vec<(String, String, Option<String>)> = tx
                .prepare(
                    "SELECT doc_id, json, batch_rev FROM revisions
                     WHERE batch = ? AND origin IN ('plugin', 'agent') ORDER BY id",
                )?
                .query_map([batch], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
                .collect::<rusqlite::Result<_>>()?;
            let mut report = crate::UndoReport::default();
            for (id, json, batch_rev) in kept {
                let row = load_row(tx, &id)?.filter(|r| !r.deleted);
                let title_of = |fields: &Map<String, Value>| {
                    fields
                        .get("title")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_owned()
                };
                let Some(row) = row.filter(|r| Some(&r.rev) == batch_rev.as_ref()) else {
                    let title = load_row(tx, &id)?
                        .filter(|r| !r.deleted)
                        .map(|r| title_of(&r.fields))
                        .unwrap_or_default();
                    report.skipped.push(Skipped { id, title });
                    continue;
                };
                // A notebook locked since the batch was applied: the note is
                // left alone, as for any write.
                if guard(tx, &row, "Kanso").is_err() {
                    let title = title_of(&row.fields);
                    report.skipped.push(Skipped { id, title });
                    continue;
                }
                if json == CREATED {
                    if trash_created {
                        let mut fields = row.fields.clone();
                        fields.insert("bookId".into(), TRASH_BOOK_ID.into());
                        fields.insert("updatedAt".into(), now_ms.into());
                        let fields = crate::validate::normalize(
                            DocType::Note,
                            &id,
                            fields,
                            Some(&row.fields),
                            now_ms,
                            crate::validate::Refs::Skip,
                        )?;
                        let rev = next_rev(Some(&row), &Value::Object(fields.clone()))?;
                        store_row(
                            tx,
                            &id,
                            DocType::Note,
                            &rev,
                            false,
                            &fields,
                            Some(&row),
                            Synced::Keep,
                            Origin::Ui,
                            changes,
                        )?;
                    } else {
                        crate::write::delete_row(tx, &id, &row, Origin::Ui, changes)?;
                    }
                    report.restored.push(id);
                    continue;
                }
                let before: Map<String, Value> = match serde_json::from_str(&json) {
                    Ok(Value::Object(map)) => map,
                    _ => Map::new(),
                };
                let mut fields = row.fields.clone();
                for key in ["title", "body", "tags", "status", "bookId", PLUGIN_DATA_FIELD] {
                    match before.get(key) {
                        Some(value) => fields.insert(key.into(), value.clone()),
                        None => fields.remove(key),
                    };
                }
                fields.insert("updatedAt".into(), now_ms.into());
                let fields = crate::validate::normalize(
                    DocType::Note,
                    &id,
                    fields,
                    Some(&row.fields),
                    now_ms,
                    crate::validate::Refs::Skip,
                )?;
                let rev = next_rev(Some(&row), &Value::Object(fields.clone()))?;
                store_row(
                    tx,
                    &id,
                    DocType::Note,
                    &rev,
                    false,
                    &fields,
                    Some(&row),
                    Synced::Keep,
                    Origin::Ui,
                    changes,
                )?;
                report.restored.push(id);
            }
            tx.execute(
                "UPDATE revisions SET batch = NULL, batch_rev = NULL WHERE batch = ?",
                [batch],
            )?;
            // The created markers stood for no version of their own.
            tx.execute(
                "DELETE FROM revisions WHERE origin IN ('plugin', 'agent') AND json = ? AND batch IS NULL",
                [CREATED],
            )?;
            Ok(report)
        })
    }

    /// Removes `plugin_id`'s data from every note that has it, in one
    /// transaction, when the plugin is removed. Returns how many notes
    /// changed.
    pub fn remove_plugin_data(&self, plugin_id: &str) -> Result<usize> {
        self.write(|tx, changes| {
            let ids: Vec<String> = tx
                .prepare(
                    "SELECT id FROM docs WHERE type = 'note' AND deleted = 0
                     AND json_type(json, '$.kansoPlugin') = 'object'",
                )?
                .query_map([], |row| row.get(0))?
                .collect::<rusqlite::Result<_>>()?;
            let mut changed = 0;
            for id in ids {
                let Some(row) = load_row(tx, &id)?.filter(|r| !r.deleted) else {
                    continue;
                };
                if plugin_data(&row.fields, plugin_id).is_none() {
                    continue;
                }
                let mut fields = row.fields.clone();
                set_plugin_data(&mut fields, plugin_id, None);
                let rev = next_rev(Some(&row), &Value::Object(fields.clone()))?;
                store_row(
                    tx,
                    &id,
                    DocType::Note,
                    &rev,
                    false,
                    &fields,
                    Some(&row),
                    Synced::Keep,
                    Origin::Ui,
                    changes,
                )?;
                changed += 1;
            }
            Ok(changed)
        })
    }

    /// The notes plugin batch `batch` changed or made, with their titles
    /// now, for the review sheet's undo notice.
    pub fn plugin_batch_notes(&self, batch: &str) -> Result<Vec<String>> {
        self.read(|conn| {
            let ids = conn
                .prepare(
                    "SELECT DISTINCT doc_id FROM revisions
                     WHERE batch = ? AND origin IN ('plugin', 'agent') ORDER BY id",
                )?
                .query_map([batch], |row| row.get(0))?
                .collect::<rusqlite::Result<_>>()?;
            Ok(ids)
        })
    }
}
