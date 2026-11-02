//! Tables derived from note bodies and sealed-note metadata: links, headings,
//! tasks and attachment ownership.
//!
//! They are local to this device: rebuilt from the bodies here, kept current
//! by every write ([`write_note`], from `store_row`), never synced, and
//! emptied with the library. When the way they are derived changes,
//! [`DERIVED_VERSION`] moves on and [`rebuild_if_stale`] fills them again
//! from the bodies and note fields when the store opens.

use std::collections::HashSet;

use kanso_core::links::{
    LinkTarget, line_text, mention_lines, normalize_title, note_links, plain_line,
    retitle_wiki_links,
};
use kanso_core::loops::{Loops, locate, loop_items, loop_key, read_loops};
use kanso_core::model::{LOCK_FIELD, note_files_from_extra};
use kanso_core::now_ms;
use kanso_core::outline::{Heading, headings};
use kanso_core::reminders::task_key;
use kanso_core::tasks::{linked_file_ids, task_items};
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::{Map, Value};

use crate::write::rewrite_note;
use crate::{Origin, Result, Store, schema};

/// The `meta` key holding the [`DERIVED_VERSION`] the tables were built with.
pub const DERIVED_VERSION_KEY: &str = "index.derived_version";
/// Changes whenever a derived table's source or derivation changes.
pub const DERIVED_VERSION: &str = "4";

/// The tables, for emptying them.
pub(crate) const TABLES: [&str; 6] = [
    "note_links",
    "note_headings",
    "note_tasks",
    "note_files",
    "note_loops",
    "note_cards",
];

/// How long a finished item is remembered, in days.
const FINISHED_KEEP_DAYS: i64 = 400;

/// How long a backlink's or mention's context line gets, in characters.
const CONTEXT_CHARS: usize = 160;
/// The most unlinked mentions listed.
const MENTION_LIMIT: usize = 30;

/// A note that links to (or mentions) another, with the line it does it on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Backlink {
    pub id: String,
    pub title: String,
    /// The line (0-based) of the first link or mention.
    pub line: u32,
    /// That line as plain words, shortened.
    pub context: String,
}

/// A heading found by a search, with the note it is in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeadingHit {
    pub note_id: String,
    pub note_title: String,
    pub level: u8,
    pub text: String,
    pub line: u32,
}

/// Replaces the derived rows of note `id`: from `fields` (its body), or
/// none for a deleted note. Also keeps the local history of what was
/// finished: a task newly ticked, a loop newly closed.
pub(crate) fn write_note(
    conn: &Connection,
    id: &str,
    fields: Option<&Map<String, Value>>,
    ownership_fields: Option<&Map<String, Value>>,
    in_locked_notebook: bool,
) -> Result<()> {
    let locked = in_locked_notebook || ownership_fields.is_some_and(|f| f.contains_key(LOCK_FIELD));
    let before: HashSet<String> = if fields.is_some() && !locked {
        let mut stmt =
            conn.prepare_cached("SELECT text FROM note_tasks WHERE note_id = ? AND checked = 0")?;
        stmt.query_map([id], |row| row.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<String>>>()?
            .into_iter()
            .map(|text| task_key(id, &text, ""))
            .collect()
    } else {
        HashSet::new()
    };
    for table in TABLES {
        conn.execute(&format!("DELETE FROM {table} WHERE note_id = ?"), [id])?;
    }
    let Some(fields) = fields else {
        conn.execute("DELETE FROM finished_items WHERE note_id = ?", [id])?;
        return Ok(());
    };
    let body = fields.get("body").and_then(Value::as_str).unwrap_or("");
    let loops = read_loops(fields);
    insert_rows(conn, id, body, &loops)?;
    if locked {
        conn.execute("DELETE FROM finished_items WHERE note_id = ?", [id])?;
    } else {
        record_finished(conn, id, body, &loops, &before)?;
    }
    if let Some(raw_fields) = ownership_fields {
        write_file_ownership(conn, id, raw_fields, in_locked_notebook)?;
    }
    Ok(())
}

/// Notes what a save finished. `before` holds the keys of the tasks that
/// were open until now: one that is ticked now is finished (at this
/// device's clock, so a tick that arrives by sync counts when it arrived);
/// one that is open again isn't. A loop closed in `kansoLoops` is
/// finished at the time it was closed.
fn record_finished(
    conn: &Connection,
    id: &str,
    body: &str,
    loops: &Loops,
    before: &HashSet<String>,
) -> Result<()> {
    let now = now_ms();
    for task in task_items(body) {
        let key = task_key(id, &task.text, "");
        if task.checked {
            if before.contains(&key) {
                conn.execute(
                    "INSERT OR REPLACE INTO finished_items (note_id, key, kind, text, at)
                     VALUES (?1, ?2, 'task', ?3, ?4)",
                    params![id, key, task.text, now],
                )?;
            }
        } else {
            conn.execute(
                "DELETE FROM finished_items WHERE note_id = ?1 AND key = ?2 AND kind = 'task'",
                params![id, key],
            )?;
        }
    }
    if loops.closed.is_empty() {
        return Ok(());
    }
    let mut texts: Vec<(String, String)> = loop_items(body)
        .into_iter()
        .map(|item| (loop_key(id, &item.text), item.text))
        .collect();
    texts.extend(loops.ai.values().map(|a| (a.key.clone(), a.text.clone())));
    for (key, at) in &loops.closed {
        let Some((_, text)) = texts.iter().find(|(k, _)| k == key) else {
            continue;
        };
        conn.execute(
            "INSERT OR IGNORE INTO finished_items (note_id, key, kind, text, at)
             VALUES (?1, ?2, 'loop', ?3, ?4)",
            params![id, key, text, at],
        )?;
    }
    Ok(())
}

fn write_file_ownership(
    conn: &Connection,
    note_id: &str,
    fields: &Map<String, Value>,
    in_locked_notebook: bool,
) -> Result<()> {
    let listed = note_files_from_extra(fields);
    let files = if (fields.contains_key(LOCK_FIELD) || in_locked_notebook) && listed.is_some() {
        listed.map(|items| items.into_iter().map(|item| item.file).collect())
    } else if in_locked_notebook {
        let body = fields.get("body").and_then(Value::as_str).unwrap_or("");
        Some(linked_file_ids(body))
    } else if fields.contains_key(LOCK_FIELD) {
        None
    } else {
        let body = fields.get("body").and_then(Value::as_str).unwrap_or("");
        Some(linked_file_ids(body))
    };
    if let Some(files) = files {
        let mut stmt = conn.prepare_cached(
            "INSERT OR IGNORE INTO note_files (note_id, file_id) VALUES (?1, ?2)",
        )?;
        for file_id in files {
            stmt.execute(params![note_id, file_id])?;
        }
    }
    Ok(())
}

fn insert_rows(conn: &Connection, id: &str, body: &str, loops: &Loops) -> Result<()> {
    if body.is_empty() {
        return Ok(());
    }
    {
        let mut stmt = conn.prepare_cached(
            "INSERT INTO note_links (note_id, ord, target_id, target_title, heading, line)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )?;
        for (ord, link) in note_links(body).into_iter().enumerate() {
            let (target_id, target_title) = match link.target {
                LinkTarget::Note(id) => (Some(id), None),
                LinkTarget::Title(title) => (None, Some(title)),
            };
            stmt.execute(params![
                id,
                ord as i64,
                target_id,
                target_title,
                link.heading,
                link.line
            ])?;
        }
    }
    {
        let mut stmt = conn.prepare_cached(
            "INSERT INTO note_headings (note_id, ord, level, text, line) VALUES (?1, ?2, ?3, ?4, ?5)",
        )?;
        for (ord, heading) in headings(body).into_iter().enumerate() {
            stmt.execute(params![
                id,
                ord as i64,
                heading.level,
                heading.text,
                heading.line
            ])?;
        }
    }
    {
        let mut stmt = conn.prepare_cached(
            "INSERT INTO note_tasks (note_id, ord, line, checked, text, due) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )?;
        for (ord, task) in task_items(body).into_iter().enumerate() {
            stmt.execute(params![
                id,
                ord as i64,
                task.line,
                task.checked,
                task.text,
                task.due
            ])?;
        }
    }
    {
        let mut stmt = conn.prepare_cached(
            "INSERT INTO note_cards (note_id, ord, card_key, line, question, answer)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )?;
        // A second card with the same question shares its key and isn't
        // scheduled twice.
        let mut seen = HashSet::new();
        let mut ord = 0i64;
        for card in kanso_core::cards::card_lines(body) {
            let key = kanso_core::cards::card_key(id, &card.question);
            if !seen.insert(key.clone()) {
                continue;
            }
            stmt.execute(params![id, ord, key, card.line, card.question, card.answer])?;
            ord += 1;
        }
    }
    {
        let mut stmt = conn.prepare_cached(
            "INSERT INTO note_loops (note_id, ord, line, kind, text, key, due, source)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        )?;
        let mut keys = HashSet::new();
        let mut ord = 0i64;
        for item in loop_items(body) {
            let key = loop_key(id, &item.text);
            keys.insert(key.clone());
            stmt.execute(params![
                id,
                ord,
                item.line,
                item.kind.as_str(),
                item.text,
                key,
                item.due,
                "rule"
            ])?;
            ord += 1;
        }
        // Loops an AI pass proposed and the person approved: kept while
        // their sentence is still in the note.
        for ai in loops.ai.values() {
            if keys.contains(&ai.key) {
                continue;
            }
            let Some(line) = locate(body, &ai.text, None) else {
                continue;
            };
            stmt.execute(params![
                id,
                ord,
                line,
                ai.kind.as_str(),
                ai.text,
                ai.key,
                kanso_core::tasks::due_date(&ai.text),
                "ai"
            ])?;
            ord += 1;
        }
    }
    Ok(())
}

/// Fills the tables again from every live note's body when they were built
/// another way, or never: in one transaction, then records the version.
pub(crate) fn rebuild_if_stale(conn: &mut Connection) -> Result<bool> {
    if schema::get_meta(conn, DERIVED_VERSION_KEY)?.as_deref() == Some(DERIVED_VERSION) {
        return Ok(false);
    }
    let tx = rusqlite::Transaction::new(conn, rusqlite::TransactionBehavior::Immediate)?;
    for table in TABLES {
        tx.execute(&format!("DELETE FROM {table}"), [])?;
    }
    let notes: Vec<(String, Option<String>, String, bool)> = tx
        .prepare(concat!(
            "SELECT id, ",
            crate::schema::readable_body!(""),
            ", json, IFNULL(book_id, '') IN (SELECT id FROM locked_books) ",
            "FROM docs WHERE type = 'note' AND deleted = 0"
        ))?
        .query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?
        .collect::<rusqlite::Result<_>>()?;
    for (id, body, json, in_locked_notebook) in notes {
        let fields = match serde_json::from_str::<Value>(&json)? {
            Value::Object(fields) => fields,
            _ => Map::new(),
        };
        let loops = if body.is_some() {
            read_loops(&fields)
        } else {
            Loops::default()
        };
        insert_rows(&tx, &id, body.as_deref().unwrap_or(""), &loops)?;
        write_file_ownership(&tx, &id, &fields, in_locked_notebook)?;
    }
    schema::set_meta(&tx, DERIVED_VERSION_KEY, DERIVED_VERSION)?;
    tx.commit()?;
    Ok(true)
}

/// Forgets what was finished more than [`FINISHED_KEEP_DAYS`] days ago.
pub(crate) fn prune_finished(conn: &Connection) -> Result<()> {
    let cutoff = now_ms() - FINISHED_KEEP_DAYS * 86_400_000;
    conn.execute("DELETE FROM finished_items WHERE at < ?", [cutoff])?;
    Ok(())
}

/// A note's context line, as plain words, shortened.
fn context(body: &str, line: u32) -> String {
    let text = plain_line(line_text(body, line));
    if text.chars().count() <= CONTEXT_CHARS {
        return text;
    }
    let mut short: String = text.chars().take(CONTEXT_CHARS - 1).collect();
    short.push('…');
    short
}

/// The live note outside the trash a title names: of those with that title
/// (normalized), the first created.
fn resolve(conn: &Connection, title: &str) -> Result<Option<(String, String)>> {
    let wanted = normalize_title(title);
    if wanted.is_empty() {
        return Ok(None);
    }
    Ok(conn
        .query_row(
            "SELECT id, title FROM docs WHERE type = 'note' AND deleted = 0
             AND IFNULL(book_id, '') != 'trash' AND kanso_norm_title(title) = ?
             ORDER BY created_at, id LIMIT 1",
            [wanted],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                ))
            },
        )
        .optional()?)
}

impl Store {
    /// The other live notes outside the trash that link to note `id`, by
    /// ID link or by a wiki-link to its title (when the title
    /// names it), most recently changed first, each with the line of its
    /// first link.
    pub fn backlinks(&self, id: &str) -> Result<Vec<Backlink>> {
        self.read(|conn| {
            let title: Option<String> = conn
                .query_row(
                    "SELECT title FROM docs WHERE id = ? AND deleted = 0",
                    [id],
                    |row| row.get(0),
                )
                .optional()?
                .flatten();
            let by_title = match title.as_deref() {
                Some(title) => resolve(conn, title)?
                    .filter(|(found, _)| found == id)
                    .map(|_| normalize_title(title)),
                None => None,
            };
            let mut stmt = conn.prepare(concat!(
                "SELECT d.id, d.title, MIN(l.line), ",
                crate::schema::readable_body!("d."),
                "
                 FROM note_links l JOIN docs d ON d.id = l.note_id
                 WHERE d.deleted = 0 AND IFNULL(d.book_id, '') != 'trash' AND d.id != ?1
                   AND (l.target_id = ?1 OR (?2 IS NOT NULL AND l.target_title = ?2))
                 GROUP BY d.id ORDER BY d.updated_at DESC, d.id"
            ))?;
            let rows = stmt.query_map(params![id, by_title], |row| {
                let body: Option<String> = row.get(3)?;
                let line: u32 = row.get(2)?;
                Ok(Backlink {
                    id: row.get(0)?,
                    title: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                    line,
                    context: context(body.as_deref().unwrap_or(""), line),
                })
            })?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }

    /// Live notes outside the trash that mention note `id`'s title as words
    /// of their own without linking to it, most recently changed first.
    pub fn unlinked_mentions(&self, id: &str) -> Result<Vec<Backlink>> {
        self.read(|conn| {
            let title: Option<String> = conn
                .query_row(
                    "SELECT title FROM docs WHERE id = ? AND deleted = 0",
                    [id],
                    |row| row.get(0),
                )
                .optional()?
                .flatten();
            let Some(title) = title.filter(|t| t.trim().chars().count() >= 2) else {
                return Ok(Vec::new());
            };
            let title = title.trim().to_owned();
            let normalized = normalize_title(&title);
            let use_fts = title.is_ascii() && title.chars().count() >= 3;
            let prefilter = if use_fts {
                "d.id IN (SELECT note_id FROM notes_fts WHERE notes_fts MATCH ?2) AND "
            } else {
                ""
            };
            let sql = format!(
                concat!(
                    "SELECT d.id, d.title, ",
                    crate::schema::readable_body!("d."),
                    " FROM docs d
                 WHERE d.type = 'note' AND d.deleted = 0 AND IFNULL(d.book_id, '') != 'trash'
                   AND d.id != ?1 AND {prefilter}kanso_contains(",
                    crate::schema::readable_body!("d."),
                    ", ?3)
                   AND NOT EXISTS (SELECT 1 FROM note_links l WHERE l.note_id = d.id
                       AND (l.target_id = ?1 OR l.target_title = ?4))
                 ORDER BY d.updated_at DESC, d.id"
                ),
                prefilter = prefilter
            );
            let fts = format!("body : \"{}\"", title.replace('"', "\"\""));
            let mut stmt = conn.prepare(&sql)?;
            let mut found = Vec::new();
            let mut rows = if use_fts {
                stmt.query(params![id, fts, title, normalized])?
            } else {
                // Unused parameters still need values.
                stmt.query(params![id, "", title, normalized])?
            };
            while let Some(row) = rows.next()? {
                let note: String = row.get(0)?;
                let note_title: String = row.get::<_, Option<String>>(1)?.unwrap_or_default();
                let body: String = row.get::<_, Option<String>>(2)?.unwrap_or_default();
                if let Some(&line) = mention_lines(&body, &title).first() {
                    found.push(Backlink {
                        id: note,
                        title: note_title,
                        line,
                        context: context(&body, line),
                    });
                    if found.len() >= MENTION_LIMIT {
                        break;
                    }
                }
            }
            Ok(found)
        })
    }

    /// Note `id`'s headings, in order.
    pub fn outline(&self, id: &str) -> Result<Vec<Heading>> {
        self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT level, text, line FROM note_headings WHERE note_id = ? ORDER BY ord",
            )?;
            let rows = stmt.query_map([id], |row| {
                Ok(Heading {
                    level: row.get(0)?,
                    text: row.get(1)?,
                    line: row.get(2)?,
                })
            })?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }

    /// The note a wiki-link's title names, as `(id, title)`: of the live
    /// notes outside the trash with that title (normalized), the first
    /// created.
    pub fn resolve_title(&self, title: &str) -> Result<Option<(String, String)>> {
        self.read(|conn| resolve(conn, title))
    }

    /// Every live note outside the trash as `(id, title)`, first created
    /// first: the order wiki-links resolve titles in.
    pub fn note_titles(&self) -> Result<Vec<(String, String)>> {
        self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, title FROM docs WHERE type = 'note' AND deleted = 0
                 AND IFNULL(book_id, '') != 'trash' ORDER BY created_at, id",
            )?;
            let rows = stmt.query_map([], |row| {
                Ok((
                    row.get(0)?,
                    row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                ))
            })?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }

    /// Headings of live notes outside the trash whose words contain
    /// `needle` (case-insensitive), in the most recently changed notes
    /// first, at most `limit`.
    pub fn search_headings(&self, needle: &str, limit: usize) -> Result<Vec<HeadingHit>> {
        if needle.trim().is_empty() {
            return Ok(Vec::new());
        }
        self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT h.note_id, d.title, h.level, h.text, h.line FROM note_headings h
                 JOIN docs d ON d.id = h.note_id
                 WHERE d.deleted = 0 AND IFNULL(d.book_id, '') != 'trash'
                   AND kanso_contains(h.text, ?1)
                 ORDER BY d.updated_at DESC, h.note_id, h.ord LIMIT ?2",
            )?;
            let rows = stmt.query_map(params![needle.trim(), limit as i64], |row| {
                Ok(HeadingHit {
                    note_id: row.get(0)?,
                    note_title: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                    level: row.get(2)?,
                    text: row.get(3)?,
                    line: row.get(4)?,
                })
            })?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }

    /// How many wiki-links to `title` (normalized) the live notes outside
    /// the trash have, leaving out note `except`'s own, and in how many
    /// notes.
    pub fn wiki_link_count(&self, title: &str, except: &str) -> Result<(usize, usize)> {
        let wanted = normalize_title(title);
        if wanted.is_empty() {
            return Ok((0, 0));
        }
        self.read(|conn| {
            Ok(conn.query_row(
                "SELECT COUNT(*), COUNT(DISTINCT l.note_id) FROM note_links l JOIN docs d ON d.id = l.note_id
                 WHERE d.deleted = 0 AND IFNULL(d.book_id, '') != 'trash' AND d.id != ?2
                   AND l.target_title = ?1",
                params![wanted, except],
                |row| Ok((row.get::<_, i64>(0)? as usize, row.get::<_, i64>(1)? as usize)),
            )?)
        })
    }

    /// Points the wiki-links to `old` (normalized) in the live notes outside
    /// the trash, other than `except`, at `new`, in one transaction, each
    /// note under a new revision. Returns how many links changed.
    pub fn retitle_links(
        &self,
        old: &str,
        new: &str,
        except: &str,
        origin: Origin,
    ) -> Result<usize> {
        let wanted = normalize_title(old);
        if wanted.is_empty() {
            return Ok(0);
        }
        self.write(|tx, changes| {
            let notes: Vec<String> = tx
                .prepare(
                    "SELECT DISTINCT l.note_id FROM note_links l JOIN docs d ON d.id = l.note_id
                     WHERE d.deleted = 0 AND IFNULL(d.book_id, '') != 'trash' AND d.id != ?2
                       AND l.target_title = ?1",
                )?
                .query_map(params![wanted, except], |row| row.get(0))?
                .collect::<rusqlite::Result<_>>()?;
            let mut changed = 0;
            let now = now_ms();
            for note in notes {
                rewrite_note(tx, &note, origin, changes, |fields| {
                    let body = fields.get("body").and_then(Value::as_str).unwrap_or("");
                    let (body, n) = retitle_wiki_links(body, old, new);
                    if n > 0 {
                        changed += n;
                        fields.insert("body".into(), body.into());
                        fields.insert("updatedAt".into(), now.into());
                    }
                })?;
            }
            Ok(changed)
        })
    }

    /// Whether the derived tables were filled again on open: test support.
    #[doc(hidden)]
    pub fn derived_rows(&self, id: &str) -> Result<(usize, usize, usize)> {
        self.read(|conn| {
            let count = |table: &str| -> Result<usize> {
                Ok(conn.query_row(
                    &format!("SELECT COUNT(*) FROM {table} WHERE note_id = ?"),
                    [id],
                    |row| row.get::<_, i64>(0),
                )? as usize)
            };
            Ok((
                count("note_links")?,
                count("note_headings")?,
                count("note_tasks")?,
            ))
        })
    }
}
