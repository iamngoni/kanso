//! Open loops across notes, read from the derived `note_loops` table, and
//! closing them; the loops an AI pass proposed and the person approved;
//! and the local history of what was finished (for the weekly review).
//!
//! A loop is open when it has a row and its key is in neither `closed` nor
//! `dismissed` of its note's `kansoLoops` field. Locked notes and notes in
//! locked notebooks write no rows (their body is blanked when indexed), and
//! every read here leaves them out again.

use kanso_core::loops::{
    AiLoop, LOOPS_FIELD, LoopKind, hide, locate, loop_items, loop_key, loops_value, make_task,
    mark_done, merge_loops, read_loops,
};
use rusqlite::params;
use rusqlite::types::Value as SqlValue;
use serde_json::Value;

use crate::schema::note_locked;
use crate::write::{load_row, rewrite_note};
use crate::tasks::CheckOff;
use crate::{Origin, Result, Store, StoreError};

/// Where a loop came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoopSource {
    /// The phrase rules found it.
    Rule,
    /// An AI pass proposed it and the person approved it.
    Ai,
}

impl LoopSource {
    pub fn as_str(self) -> &'static str {
        match self {
            LoopSource::Rule => "rule",
            LoopSource::Ai => "ai",
        }
    }
}

/// An open loop, with the note it is in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoopRow {
    pub note_id: String,
    pub note_title: String,
    pub book_id: String,
    /// The line (0-based) it is on in the note's body.
    pub line: u32,
    pub kind: LoopKind,
    pub text: String,
    pub key: String,
    pub due: Option<String>,
    pub source: LoopSource,
    pub note_updated_at: i64,
}

/// Which loops [`Store::open_loops`] lists.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LoopQuery {
    /// Only loops in this notebook's notes.
    pub book_id: Option<String>,
    pub kind: Option<LoopKind>,
    /// Only loops in notes changed at or after this time (ms).
    pub since: Option<i64>,
    /// Only loops in notes changed before this time (ms).
    pub before: Option<i64>,
    pub limit: Option<usize>,
}

/// How a loop is closed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoopClose {
    /// Done. With `mark_in_note` the sentence gets a trailing tick in the
    /// note; otherwise only the note's `kansoLoops` field records it.
    Done { mark_in_note: bool },
    /// "Not a loop": hidden for good.
    Dismissed,
    /// The sentence becomes a `- [ ]` task in one edit of the body.
    MakeTask,
}

/// A task ticked or a loop closed, from the local history.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FinishedRow {
    pub note_id: String,
    pub note_title: String,
    pub key: String,
    /// `task` or `loop`.
    pub kind: String,
    pub text: String,
    /// When, in ms since the epoch.
    pub at: i64,
}

/// A note created or changed in a span, for the weekly review.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChangedNote {
    pub id: String,
    pub title: String,
    pub book_id: String,
    pub created_at: i64,
    pub updated_at: i64,
}

/// SQL for "`l` is a loop of a live note outside the trash that isn't
/// locked, and isn't closed or dismissed".
const OPEN_LOOPS: &str = concat!(
    "FROM note_loops l JOIN docs d ON d.id = l.note_id
     WHERE d.deleted = 0 AND IFNULL(d.book_id, '') != 'trash' AND NOT ",
    note_locked!("d."),
    " AND json_type(d.json, '$.kansoLoops.closed.\"' || l.key || '\"') IS NULL
       AND json_type(d.json, '$.kansoLoops.dismissed.\"' || l.key || '\"') IS NULL"
);

fn filter(query: &LoopQuery) -> (String, Vec<SqlValue>) {
    let mut sql = String::new();
    let mut values = Vec::new();
    if let Some(book) = &query.book_id {
        sql.push_str(" AND d.book_id = ?");
        values.push(SqlValue::Text(book.clone()));
    }
    if let Some(kind) = query.kind {
        sql.push_str(" AND l.kind = ?");
        values.push(SqlValue::Text(kind.as_str().to_owned()));
    }
    if let Some(since) = query.since {
        sql.push_str(" AND d.updated_at >= ?");
        values.push(SqlValue::Integer(since));
    }
    if let Some(before) = query.before {
        sql.push_str(" AND d.updated_at < ?");
        values.push(SqlValue::Integer(before));
    }
    (sql, values)
}

impl Store {
    /// The open loops of live notes outside the trash that `query` asks
    /// for, the most recently changed notes first, each note's in order.
    pub fn open_loops(&self, query: &LoopQuery) -> Result<Vec<LoopRow>> {
        let (extra, mut values) = filter(query);
        let sql = format!(
            "SELECT l.note_id, d.title, d.book_id, l.line, l.kind, l.text, l.key, l.due, l.source,
                    d.updated_at {OPEN_LOOPS}{extra}
             ORDER BY d.updated_at DESC, l.note_id, l.ord LIMIT ?"
        );
        values.push(SqlValue::Integer(query.limit.map_or(-1, |n| n as i64)));
        self.read(|conn| {
            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt.query_map(rusqlite::params_from_iter(values), |row| {
                let kind: String = row.get(4)?;
                let source: String = row.get(8)?;
                Ok(LoopRow {
                    note_id: row.get(0)?,
                    note_title: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                    book_id: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                    line: row.get(3)?,
                    kind: LoopKind::parse(&kind).unwrap_or(LoopKind::Mine),
                    text: row.get(5)?,
                    key: row.get(6)?,
                    due: row.get(7)?,
                    source: if source == "ai" {
                        LoopSource::Ai
                    } else {
                        LoopSource::Rule
                    },
                    note_updated_at: row.get::<_, Option<i64>>(9)?.unwrap_or(0),
                })
            })?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }

    /// How many loops are open, as [`Store::open_loops`] with `query`
    /// would list (its limit ignored).
    pub fn open_loop_count(&self, query: &LoopQuery) -> Result<u32> {
        let (extra, values) = filter(query);
        let sql = format!("SELECT COUNT(*) {OPEN_LOOPS}{extra}");
        self.read(|conn| {
            Ok(
                conn.query_row(&sql, rusqlite::params_from_iter(values), |row| {
                    row.get::<_, i64>(0)
                })? as u32,
            )
        })
    }

    /// Closes the loop `key` of note `id` as `how`, in one write. Done and
    /// Dismissed record it in the note's `kansoLoops` field (an ack isn't
    /// an edit, so the note's edit time stays; with `mark_in_note` the
    /// body gets a tick and the time moves). Make Task rewrites the
    /// sentence's line as a task, and the note's edit time becomes
    /// `now_ms`. `line` is where the caller saw the loop; a moved loop is
    /// found by its sentence. `Missing` when the loop isn't in the note
    /// any more.
    pub fn close_loop(
        &self,
        id: &str,
        key: &str,
        line: u32,
        how: LoopClose,
        now_ms: i64,
        origin: Origin,
    ) -> Result<CheckOff> {
        self.write(|tx, changes| {
            let row = load_row(tx, id)?
                .filter(|r| !r.deleted)
                .ok_or(StoreError::NotFound)?;
            let body = row.fields.get("body").and_then(Value::as_str).unwrap_or("");
            let loops = read_loops(&row.fields);
            let found = loop_items(body)
                .into_iter()
                .map(|item| (loop_key(id, &item.text), item.line, item.text))
                .chain(loops.ai.values().filter_map(|ai| {
                    Some((ai.key.clone(), locate(body, &ai.text, Some(line))?, ai.text.clone()))
                }))
                .collect::<Vec<_>>();
            let Some((_, at, text)) = found.iter().find(|(k, ..)| k == key).cloned() else {
                return Ok(CheckOff::Missing);
            };
            let at = if found.iter().any(|(k, l, _)| k == key && *l == line) {
                line
            } else {
                at
            };
            let live: Vec<String> = found.iter().map(|(k, ..)| k.clone()).collect();
            match how {
                LoopClose::Done { .. } | LoopClose::Dismissed => {
                    let dismissed = how == LoopClose::Dismissed;
                    let marked = match how {
                        LoopClose::Done { mark_in_note: true } => mark_done(body, at, &text),
                        _ => None,
                    };
                    let next = hide(&loops, key, dismissed, now_ms, &live);
                    if next == loops && marked.is_none() {
                        return Ok(CheckOff::Unchanged);
                    }
                    rewrite_note(tx, id, origin, changes, |fields| {
                        fields.insert(LOOPS_FIELD.into(), loops_value(&next));
                        if let Some(body) = marked {
                            fields.insert("body".into(), body.into());
                            fields.insert("updatedAt".into(), now_ms.into());
                        }
                    })?;
                }
                LoopClose::MakeTask => {
                    let Some(body) = make_task(body, at, &text) else {
                        return Ok(CheckOff::Missing);
                    };
                    rewrite_note(tx, id, origin, changes, |fields| {
                        fields.insert("body".into(), body.into());
                        fields.insert("updatedAt".into(), now_ms.into());
                    })?;
                }
            }
            let rev = load_row(tx, id)?.map(|r| r.rev).unwrap_or_default();
            Ok(CheckOff::Changed { rev })
        })
    }

    /// Keeps `approved`, loops an AI pass proposed and the person approved,
    /// in note `id`'s `kansoLoops` field, merged with what it holds. Only
    /// sentences that are in the note's body, and aren't loops the rules
    /// find already, are kept. Returns how many were added. The note's
    /// edit time stays.
    pub fn approve_ai_loops(&self, id: &str, approved: &[AiLoop], origin: Origin) -> Result<usize> {
        self.write(|tx, changes| {
            let Some(row) = load_row(tx, id)?.filter(|r| !r.deleted) else {
                return Ok(0);
            };
            if row.fields.contains_key(kanso_core::model::LOCK_FIELD) {
                return Ok(0);
            }
            let body = row.fields.get("body").and_then(Value::as_str).unwrap_or("");
            let rule_keys: Vec<String> = loop_items(body)
                .into_iter()
                .map(|item| loop_key(id, &item.text))
                .collect();
            let loops = read_loops(&row.fields);
            let mut add = kanso_core::loops::Loops::default();
            for ai in approved {
                let key = loop_key(id, &ai.text);
                if rule_keys.contains(&key) || locate(body, &ai.text, None).is_none() {
                    continue;
                }
                add.ai.insert(
                    key.clone(),
                    AiLoop {
                        key,
                        text: ai.text.clone(),
                        kind: ai.kind,
                        approved_at: ai.approved_at,
                    },
                );
            }
            let merged = merge_loops(&loops, &add);
            let added = merged.ai.len() - loops.ai.len();
            if merged == loops {
                return Ok(0);
            }
            rewrite_note(tx, id, origin, changes, |fields| {
                fields.insert(LOOPS_FIELD.into(), loops_value(&merged));
            })?;
            Ok(added)
        })
    }

    /// What was finished between `from_ms` (inclusive) and `to_ms`
    /// (exclusive), newest first, leaving out locked and trashed notes.
    pub fn finished_between(&self, from_ms: i64, to_ms: i64) -> Result<Vec<FinishedRow>> {
        self.read(|conn| {
            let mut stmt = conn.prepare(concat!(
                "SELECT f.note_id, d.title, f.key, f.kind, f.text, f.at
                 FROM finished_items f JOIN docs d ON d.id = f.note_id
                 WHERE f.at >= ?1 AND f.at < ?2 AND d.deleted = 0
                   AND IFNULL(d.book_id, '') != 'trash' AND NOT ",
                note_locked!("d."),
                " ORDER BY f.at DESC, f.note_id, f.key"
            ))?;
            let rows = stmt.query_map(params![from_ms, to_ms], |row| {
                Ok(FinishedRow {
                    note_id: row.get(0)?,
                    note_title: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                    key: row.get(2)?,
                    kind: row.get(3)?,
                    text: row.get(4)?,
                    at: row.get(5)?,
                })
            })?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }

    /// Live notes outside the trash, not locked, created or changed
    /// between `from_ms` (inclusive) and `to_ms` (exclusive), the most
    /// recently changed first.
    pub fn notes_changed_between(&self, from_ms: i64, to_ms: i64) -> Result<Vec<ChangedNote>> {
        self.read(|conn| {
            let mut stmt = conn.prepare(concat!(
                "SELECT d.id, d.title, d.book_id, IFNULL(d.created_at, 0), IFNULL(d.updated_at, 0)
                 FROM docs d
                 WHERE d.type = 'note' AND d.deleted = 0 AND IFNULL(d.book_id, '') != 'trash'
                   AND NOT ",
                note_locked!("d."),
                " AND ((d.updated_at >= ?1 AND d.updated_at < ?2)
                     OR (d.created_at >= ?1 AND d.created_at < ?2))
                 ORDER BY d.updated_at DESC, d.id"
            ))?;
            let rows = stmt.query_map(params![from_ms, to_ms], |row| {
                Ok(ChangedNote {
                    id: row.get(0)?,
                    title: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                    book_id: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            })?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }
}
