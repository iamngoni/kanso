//! Find and replace across notes. A preview lists each note's matches with
//! its revision; applying writes only the notes still at that revision,
//! in one transaction, keeping each one's earlier version in its history
//! under one batch ID; undoing the batch brings back each note's text
//! unless it was edited again since. A locked note (sealed, or in a locked
//! notebook) is never read for a preview and never written by either call,
//! whatever the plan or the batch names.

use kanso_core::DocType;
use kanso_core::model::TRASH_BOOK_ID;
use kanso_core::replace::{MatchContext, Matcher, contexts};
use rusqlite::{OptionalExtension, Transaction, params};
use serde_json::{Map, Value};

use crate::write::{SNAPSHOTS_PER_DOC, Synced, load_row, next_rev, store_row};
use crate::{Origin, Result, Store};

/// The most matches a preview carries for one note; the count is exact.
pub const PREVIEW_MATCHES_PER_NOTE: usize = 50;

/// One note's matches, as a preview shows them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoteMatches {
    pub id: String,
    pub title: String,
    /// The revision the matches were found in.
    pub rev: String,
    pub trashed: bool,
    /// How many matches the note has.
    pub count: usize,
    /// The first [`PREVIEW_MATCHES_PER_NOTE`] of them, with their lines.
    pub matches: Vec<MatchContext>,
}

/// A note to change, at the revision its preview was made from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Planned {
    pub id: String,
    pub rev: String,
}

/// A note left alone, with its title.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Skipped {
    pub id: String,
    pub title: String,
}

/// What [`Store::replace_in_notes`] did.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReplaceReport {
    /// How many matches were replaced.
    pub replaced: usize,
    /// The notes changed.
    pub notes: Vec<String>,
    /// Notes changed (by sync, or anything else) since the preview, or
    /// gone: left as they are.
    pub skipped: Vec<Skipped>,
}

/// What [`Store::undo_replace`] did.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UndoReport {
    /// The notes whose text came back.
    pub restored: Vec<String>,
    /// Notes edited again since the replace, or gone: left as they are.
    pub skipped: Vec<Skipped>,
}

fn body(fields: &Map<String, Value>) -> &str {
    fields.get("body").and_then(Value::as_str).unwrap_or("")
}

fn title(fields: &Map<String, Value>) -> String {
    fields
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

/// Whether note `id` reads as locked now: sealed itself, or in a locked
/// notebook. A note that isn't there reads as not locked.
fn locked_now(tx: &Transaction, id: &str) -> Result<bool> {
    Ok(tx
        .query_row(
            concat!(
                "SELECT ",
                crate::schema::note_locked!(""),
                " FROM docs WHERE id = ?"
            ),
            [id],
            |row| row.get(0),
        )
        .optional()?
        .unwrap_or(false))
}

impl Store {
    /// The live notes whose text `matcher` matches, most recently changed
    /// first: outside the trash, and inside it with `include_trash`.
    pub fn replace_preview(
        &self,
        matcher: &Matcher,
        include_trash: bool,
    ) -> Result<Vec<NoteMatches>> {
        self.read(|conn| {
            let mut stmt = conn.prepare(concat!(
                "SELECT id, rev, title, book_id, ",
                crate::schema::readable_body!(""),
                " FROM docs
                 WHERE type = 'note' AND deleted = 0 AND (?1 OR IFNULL(book_id, '') != 'trash')
                 ORDER BY updated_at DESC, id"
            ))?;
            let mut rows = stmt.query(params![include_trash])?;
            let mut found = Vec::new();
            while let Some(row) = rows.next()? {
                let body: String = row.get::<_, Option<String>>(4)?.unwrap_or_default();
                let matches = matcher.matches(&body);
                if matches.is_empty() {
                    continue;
                }
                let shown = &matches[..matches.len().min(PREVIEW_MATCHES_PER_NOTE)];
                found.push(NoteMatches {
                    id: row.get(0)?,
                    rev: row.get(1)?,
                    title: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                    trashed: row.get::<_, Option<String>>(3)?.as_deref() == Some(TRASH_BOOK_ID),
                    count: matches.len(),
                    matches: contexts(&body, shown),
                });
            }
            Ok(found)
        })
    }

    /// Replaces `matcher`'s matches with `replacement` in the `planned`
    /// notes, in one transaction. A note no longer at its planned revision
    /// (changed by sync, say, since the preview) or gone is skipped and
    /// reported, and so is a note that reads as locked (sealed, or in a
    /// locked notebook), whatever the plan says: its text is never written
    /// here. Each changed note's earlier version is kept in its history
    /// under `batch`, for [`Store::undo_replace`], and its edit time
    /// becomes `now_ms`.
    pub fn replace_in_notes(
        &self,
        matcher: &Matcher,
        replacement: &str,
        planned: &[Planned],
        batch: &str,
        now_ms: i64,
    ) -> Result<ReplaceReport> {
        self.write(|tx, changes| {
            let mut report = ReplaceReport::default();
            for plan in planned {
                let row = load_row(tx, &plan.id)?.filter(|r| !r.deleted);
                let row = match row {
                    Some(r) if r.rev == plan.rev && !locked_now(tx, &plan.id)? => Some(r),
                    _ => None,
                };
                let Some(row) = row else {
                    let title = load_row(tx, &plan.id)?
                        .map(|r| title(&r.fields))
                        .unwrap_or_default();
                    report.skipped.push(Skipped {
                        id: plan.id.clone(),
                        title,
                    });
                    continue;
                };
                let (text, count) = matcher.replace(body(&row.fields), replacement);
                if count == 0 {
                    continue;
                }
                let mut fields = row.fields.clone();
                fields.insert("body".into(), text.into());
                fields.insert("updatedAt".into(), now_ms.into());
                let fields = crate::validate::normalize(
                    DocType::Note,
                    &plan.id,
                    fields,
                    Some(&row.fields),
                    now_ms,
                    crate::validate::Refs::Skip,
                )?;
                let rev = next_rev(Some(&row), &Value::Object(fields.clone()))?;
                // The version replaced, under the batch, before the write
                // (whose own snapshot it then stands for).
                tx.execute(
                    "INSERT INTO revisions (doc_id, rev, json, saved_at, origin, batch, batch_rev)
                     VALUES (?1, ?2, ?3, ?4, 'ui', ?5, ?6)",
                    params![
                        plan.id,
                        row.rev,
                        serde_json::to_string(&row.fields)?,
                        crate::history::version_time(tx, &plan.id)?,
                        batch,
                        rev
                    ],
                )?;
                tx.execute(
                    "DELETE FROM revisions WHERE doc_id = ?1 AND id NOT IN
                     (SELECT id FROM revisions WHERE doc_id = ?1 ORDER BY saved_at DESC, id DESC LIMIT ?2)",
                    params![plan.id, SNAPSHOTS_PER_DOC],
                )?;
                store_row(
                    tx,
                    &plan.id,
                    DocType::Note,
                    &rev,
                    false,
                    &fields,
                    Some(&row),
                    Synced::Keep,
                    Origin::Ui,
                    changes,
                )?;
                report.replaced += count;
                report.notes.push(plan.id.clone());
            }
            Ok(report)
        })
    }

    /// Brings back the text each note had before replace `batch`, in one
    /// transaction, unless the note was edited again since, is locked now
    /// or is gone: those are left as they are and reported. A batch is undone once.
    pub fn undo_replace(&self, batch: &str, now_ms: i64) -> Result<UndoReport> {
        self.write(|tx, changes| {
            let kept: Vec<(String, String, Option<String>)> = tx
                .prepare(
                    "SELECT doc_id, json, batch_rev FROM revisions WHERE batch = ? ORDER BY id",
                )?
                .query_map([batch], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
                .collect::<rusqlite::Result<_>>()?;
            let mut report = UndoReport::default();
            for (id, json, batch_rev) in kept {
                let before: Map<String, Value> = match serde_json::from_str(&json) {
                    Ok(Value::Object(map)) => map,
                    _ => Map::new(),
                };
                let row = load_row(tx, &id)?.filter(|r| !r.deleted);
                // A note locked since (sealed, or its notebook locked) keeps
                // what it has: plain text never goes back into it.
                let row = match row {
                    Some(r) if Some(&r.rev) == batch_rev.as_ref() && !locked_now(tx, &id)? => {
                        Some(r)
                    }
                    _ => None,
                };
                let Some(row) = row else {
                    let title = load_row(tx, &id)?
                        .filter(|r| !r.deleted)
                        .map_or_else(|| title(&before), |r| title(&r.fields));
                    report.skipped.push(Skipped { id, title });
                    continue;
                };
                let mut fields = row.fields.clone();
                fields.insert("body".into(), body(&before).into());
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
                    Origin::Api,
                    changes,
                )?;
                report.restored.push(id);
            }
            tx.execute(
                "UPDATE revisions SET batch = NULL, batch_rev = NULL WHERE batch = ?",
                [batch],
            )?;
            Ok(report)
        })
    }
}
