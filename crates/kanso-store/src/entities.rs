//! Reading people and projects: the pages, who they are called, where they
//! are mentioned, and what suggestions detection holds. Also a page's
//! cache of AI-written sections (`kansoPage`), which is written without
//! counting as an edit of the page.
//!
//! Every read leaves out what is locked (a note, or one in a locked
//! notebook) and what is in the trash. Reads for AI also leave out notes
//! tagged `#ai-exclude`.

use kanso_core::entities::{EntityKind, How};
use kanso_core::model::SHARE_FIELD;
use kanso_core::page_cache::{self, PAGE_FIELD, PageCache};
use rusqlite::params;
use serde_json::Value;

use crate::schema::{ai_excluded, note_locked};
use crate::write::{load_row, rewrite_note};
use crate::{Origin, Result, Store};

/// The tables emptied with the library.
pub(crate) const TABLES: [&str; 2] = ["entity_candidates", "entity_dismissed"];

/// A page for a person or a project.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityRow {
    pub id: String,
    pub title: String,
    pub kind: EntityKind,
    /// Every name it answers to, normalized: title first.
    pub names: Vec<String>,
    pub book_id: String,
    pub updated_at: i64,
}

/// A line of a note that mentions an entity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MentionRow {
    pub note_id: String,
    pub title: String,
    pub book_id: String,
    /// 0-based.
    pub line: u32,
    pub how: How,
    pub created_at: i64,
    pub updated_at: i64,
    /// Others can edit the note.
    pub shared: bool,
    /// The note may be sent to AI (not tagged `#ai-exclude`).
    pub ai_ok: bool,
}

/// An open task on a line that mentions an entity, or in a note linking to
/// its page.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntityTask {
    pub note_id: String,
    pub note_title: String,
    pub line: u32,
    pub text: String,
    pub due: Option<String>,
    pub note_updated_at: i64,
}

/// A name detection suggests.
#[derive(Clone, Debug, PartialEq)]
pub struct CandidateRow {
    pub name: String,
    pub kind: EntityKind,
    pub notes: u32,
    pub last_seen: i64,
    /// The ways it is written.
    pub forms: Vec<String>,
    pub score: f64,
    /// The page it may be another name of.
    pub alias_of: Option<String>,
}

/// A name the person turned away.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DismissedRow {
    pub name: String,
    /// `never` or `later`.
    pub kind: String,
    /// The number of notes it had when set aside ("not now").
    pub notes: u32,
    pub at: i64,
}

/// A note as detection reads it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DetectNote {
    pub id: String,
    pub title: String,
    pub body: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub tags: Vec<String>,
}

const VISIBLE: &str = "d.deleted = 0 AND IFNULL(d.book_id, '') != 'trash'";

fn entity_row(row: &rusqlite::Row) -> rusqlite::Result<(String, String, String, String, i64)> {
    Ok((
        row.get(0)?,
        row.get::<_, Option<String>>(1)?.unwrap_or_default(),
        row.get(2)?,
        row.get::<_, Option<String>>(3)?.unwrap_or_default(),
        row.get::<_, Option<i64>>(4)?.unwrap_or(0),
    ))
}

impl Store {
    /// Every page for a person or a project that can be read now, most
    /// recently changed first.
    pub fn entities(&self) -> Result<Vec<EntityRow>> {
        self.read(|conn| {
            let sql = format!(
                "SELECT e.note_id, d.title, e.kind, d.book_id, d.updated_at FROM entities e
                 JOIN docs d ON d.id = e.note_id
                 WHERE {VISIBLE} AND NOT {} ORDER BY d.updated_at DESC, d.id",
                note_locked!("d.")
            );
            let mut stmt = conn.prepare(&sql)?;
            let rows: Vec<_> = stmt
                .query_map([], entity_row)?
                .collect::<rusqlite::Result<_>>()?;
            let mut names =
                conn.prepare("SELECT name FROM entity_names WHERE note_id = ? ORDER BY name")?;
            let mut out = Vec::with_capacity(rows.len());
            for (id, title, kind, book_id, updated_at) in rows {
                let mut list: Vec<String> = names
                    .query_map([&id], |row| row.get(0))?
                    .collect::<rusqlite::Result<_>>()?;
                let own = kanso_core::entities::normalize_name(&title);
                list.sort_by_key(|name| *name != own);
                out.push(EntityRow {
                    id,
                    title,
                    kind: EntityKind::parse(&kind).unwrap_or(EntityKind::Person),
                    names: list,
                    book_id,
                    updated_at,
                });
            }
            Ok(out)
        })
    }

    /// The page note `id` is, when it is one.
    pub fn entity(&self, id: &str) -> Result<Option<EntityRow>> {
        Ok(self.entities()?.into_iter().find(|e| e.id == id))
    }

    /// The pages that answer to `name` (normalized as names are): one,
    /// or more than one when the name is ambiguous.
    pub fn entities_named(&self, name: &str) -> Result<Vec<EntityRow>> {
        let wanted = kanso_core::entities::normalize_name(name);
        Ok(self
            .entities()?
            .into_iter()
            .filter(|e| e.names.contains(&wanted))
            .collect())
    }

    /// The lines of live, readable notes that mention entity `id`, newest
    /// note first (by when it was made), at most `limit` lines.
    pub fn entity_mentions(&self, id: &str, limit: usize) -> Result<Vec<MentionRow>> {
        self.read(|conn| {
            let sql = format!(
                "SELECT m.note_id, d.title, d.book_id, m.line, m.how,
                        json_extract(d.json, '$.createdAt'), d.updated_at,
                        json_extract(d.json, '$.{SHARE_FIELD}') IS NOT NULL,
                        NOT {}
                 FROM note_mentions m JOIN docs d ON d.id = m.note_id
                 WHERE m.entity_id = ?1 AND {VISIBLE} AND NOT {} AND m.note_id != ?1
                 ORDER BY json_extract(d.json, '$.createdAt') DESC, m.note_id, m.line LIMIT ?2",
                ai_excluded!("d."),
                note_locked!("d.")
            );
            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt.query_map(params![id, limit as i64], |row| {
                Ok(MentionRow {
                    note_id: row.get(0)?,
                    title: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                    book_id: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                    line: row.get(3)?,
                    how: How::parse(&row.get::<_, String>(4)?),
                    created_at: row.get::<_, Option<i64>>(5)?.unwrap_or(0),
                    updated_at: row.get::<_, Option<i64>>(6)?.unwrap_or(0),
                    shared: row.get(7)?,
                    ai_ok: row.get(8)?,
                })
            })?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }

    /// How many notes mention entity `id`, and when the latest of them was
    /// made.
    pub fn entity_mention_stats(&self, id: &str) -> Result<(u32, i64)> {
        self.read(|conn| {
            let sql = format!(
                "SELECT COUNT(DISTINCT m.note_id), IFNULL(MAX(json_extract(d.json, '$.createdAt')), 0)
                 FROM note_mentions m JOIN docs d ON d.id = m.note_id
                 WHERE m.entity_id = ?1 AND {VISIBLE} AND NOT {} AND m.note_id != ?1",
                note_locked!("d.")
            );
            Ok(conn.query_row(&sql, [id], |row| {
                Ok((row.get::<_, i64>(0)? as u32, row.get::<_, i64>(1)?))
            })?)
        })
    }

    /// Open tasks on lines that mention entity `id`, or in notes that link
    /// to its page, soonest due first.
    pub fn entity_tasks(&self, id: &str, limit: usize) -> Result<Vec<EntityTask>> {
        self.read(|conn| {
            let sql = format!(
                "SELECT t.note_id, d.title, t.line, t.text, t.due, d.updated_at
                 FROM note_tasks t JOIN docs d ON d.id = t.note_id
                 WHERE t.checked = 0 AND {VISIBLE} AND NOT {} AND t.note_id != ?1
                   AND (EXISTS (SELECT 1 FROM note_mentions m WHERE m.note_id = t.note_id
                                 AND m.entity_id = ?1 AND m.line = t.line)
                        OR EXISTS (SELECT 1 FROM note_links l WHERE l.note_id = t.note_id
                                 AND (l.target_id = ?1 OR l.target_title IN
                                      (SELECT name FROM entity_names WHERE note_id = ?1))))
                 ORDER BY (t.due IS NULL), t.due, d.updated_at DESC, t.note_id, t.ord LIMIT ?2",
                note_locked!("d.")
            );
            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt.query_map(params![id, limit as i64], |row| {
                Ok(EntityTask {
                    note_id: row.get(0)?,
                    note_title: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                    line: row.get(2)?,
                    text: row.get(3)?,
                    due: row.get(4)?,
                    note_updated_at: row.get::<_, Option<i64>>(5)?.unwrap_or(0),
                })
            })?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }

    /// The pages that are mentioned in the same notes as entity `id`, most
    /// shared notes first, with how many.
    pub fn co_entities(&self, id: &str, limit: usize) -> Result<Vec<(EntityRow, u32)>> {
        let counts: Vec<(String, u32)> = self.read(|conn| {
            let sql = format!(
                "SELECT b.entity_id, COUNT(DISTINCT a.note_id) AS n
                 FROM note_mentions a JOIN note_mentions b ON a.note_id = b.note_id
                 JOIN docs d ON d.id = a.note_id
                 WHERE a.entity_id = ?1 AND b.entity_id != ?1 AND {VISIBLE} AND NOT {}
                 GROUP BY b.entity_id ORDER BY n DESC, b.entity_id LIMIT ?2",
                note_locked!("d.")
            );
            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt.query_map(params![id, (limit * 3) as i64], |row| {
                Ok((row.get(0)?, row.get::<_, i64>(1)? as u32))
            })?;
            Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
        })?;
        let all = self.entities()?;
        Ok(counts
            .into_iter()
            .filter_map(|(entity, n)| all.iter().find(|e| e.id == entity).map(|e| (e.clone(), n)))
            .take(limit)
            .collect())
    }

    /// Notes the page links to from its own text (by ID or by title), then
    /// the notes that mention it most, at most `limit`: `(id, title)`.
    pub fn entity_related_notes(&self, id: &str, limit: usize) -> Result<Vec<(String, String)>> {
        self.read(|conn| {
            let sql = format!(
                "SELECT d.id, d.title, 0 AS rank, 0 AS n FROM note_links l JOIN docs d ON
                   (d.id = l.target_id OR (l.target_id IS NULL AND kanso_norm_title(d.title) = l.target_title))
                 WHERE l.note_id = ?1 AND d.id != ?1 AND d.type = 'note' AND {VISIBLE} AND NOT {lock}
                   AND d.id NOT IN (SELECT note_id FROM entities)
                 UNION
                 SELECT d.id, d.title, 1 AS rank, -COUNT(*) AS n FROM note_mentions m JOIN docs d ON d.id = m.note_id
                 WHERE m.entity_id = ?1 AND {VISIBLE} AND NOT {lock}
                 GROUP BY d.id
                 ORDER BY rank, n, 1 LIMIT ?2",
                lock = note_locked!("d.")
            );
            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt.query_map(params![id, (limit * 2) as i64], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                ))
            })?;
            let mut seen = std::collections::HashSet::new();
            let mut out = Vec::new();
            for row in rows {
                let (note, title) = row?;
                if seen.insert(note.clone()) {
                    out.push((note, title));
                }
            }
            out.truncate(limit);
            Ok(out)
        })
    }

    // ---- the cache of AI-written sections ----

    /// The cache `kansoPage` of document `id` (a note, or a notebook).
    pub fn page_cache(&self, id: &str) -> Result<PageCache> {
        Ok(match self.get(id)? {
            Some(doc) if !doc.fields.contains_key(kanso_core::model::LOCK_FIELD) => {
                page_cache::read(&doc.fields)
            }
            _ => PageCache::default(),
        })
    }

    /// Stores `cache` as the `kansoPage` field of note `id`. It is not an
    /// edit by the person: the note's text and edit time stay, and no
    /// version is added to its history. Does nothing for a locked note.
    pub fn set_page_cache(&self, id: &str, cache: &PageCache) -> Result<bool> {
        let value = (!cache.is_empty()).then(|| page_cache::to_value(cache));
        self.write(|tx, changes| {
            let Some(prev) = load_row(tx, id)?.filter(|p| !p.deleted) else {
                return Ok(false);
            };
            if prev.fields.contains_key(kanso_core::model::LOCK_FIELD)
                || prev.fields.get(PAGE_FIELD) == value.as_ref()
            {
                return Ok(false);
            }
            let known: i64 = tx.query_row(
                "SELECT COUNT(*) FROM revisions WHERE doc_id = ?1 AND rev = ?2",
                params![id, prev.rev],
                |row| row.get(0),
            )?;
            rewrite_note(tx, id, Origin::Api, changes, |fields| match &value {
                Some(value) => {
                    fields.insert(PAGE_FIELD.into(), value.clone());
                }
                None => {
                    fields.remove(PAGE_FIELD);
                }
            })?;
            if known == 0 {
                // The rewrite filed the version it replaced; a cache is not a
                // version of what the person wrote.
                tx.execute(
                    "DELETE FROM revisions WHERE doc_id = ?1 AND rev = ?2",
                    params![id, prev.rev],
                )?;
            }
            Ok(true)
        })
    }

    // ---- suggestions ----

    /// Replaces the suggestions with `rows`.
    pub fn set_candidates(&self, rows: &[CandidateRow]) -> Result<()> {
        self.write(|tx, _| {
            tx.execute("DELETE FROM entity_candidates", [])?;
            let mut stmt = tx.prepare(
                "INSERT OR REPLACE INTO entity_candidates (name, kind, notes, last_seen, forms, score, alias_of)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            )?;
            for row in rows {
                stmt.execute(params![
                    row.name,
                    row.kind.as_str(),
                    row.notes,
                    row.last_seen,
                    serde_json::to_string(&row.forms)?,
                    row.score,
                    row.alias_of
                ])?;
            }
            Ok(())
        })
    }

    /// The suggestions, best first.
    pub fn candidates(&self) -> Result<Vec<CandidateRow>> {
        self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT name, kind, notes, last_seen, forms, score, alias_of FROM entity_candidates
                 ORDER BY score DESC, notes DESC, name",
            )?;
            let rows = stmt.query_map([], |row| {
                let forms: String = row.get(4)?;
                Ok(CandidateRow {
                    name: row.get(0)?,
                    kind: EntityKind::parse(&row.get::<_, String>(1)?)
                        .unwrap_or(EntityKind::Person),
                    notes: row.get::<_, i64>(2)? as u32,
                    last_seen: row.get(3)?,
                    forms: serde_json::from_str(&forms).unwrap_or_default(),
                    score: row.get(5)?,
                    alias_of: row.get(6)?,
                })
            })?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }

    /// Remembers that the person turned `name` away: for good (`never`) or
    /// until it has `notes` + 5 mentions (`later`).
    pub fn dismiss_name(&self, name: &str, never: bool, notes: u32, at: i64) -> Result<()> {
        let name = kanso_core::entities::normalize_name(name);
        if name.is_empty() {
            return Ok(());
        }
        self.write(|tx, _| {
            tx.execute(
                "INSERT INTO entity_dismissed (name, kind, notes, at) VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT (name) DO UPDATE SET kind = CASE WHEN entity_dismissed.kind = 'never'
                     THEN 'never' ELSE excluded.kind END, notes = excluded.notes, at = excluded.at",
                params![name, if never { "never" } else { "later" }, notes, at],
            )?;
            tx.execute("DELETE FROM entity_candidates WHERE name = ?", [&name])?;
            Ok(())
        })
    }

    /// Forgets a dismissal ("Suggest again").
    pub fn undismiss_name(&self, name: &str) -> Result<()> {
        let name = kanso_core::entities::normalize_name(name);
        self.write(|tx, _| {
            tx.execute("DELETE FROM entity_dismissed WHERE name = ?", [name])?;
            Ok(())
        })
    }

    /// Every name the person turned away.
    pub fn dismissed(&self) -> Result<Vec<DismissedRow>> {
        self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT name, kind, notes, at FROM entity_dismissed ORDER BY at DESC, name",
            )?;
            let rows = stmt.query_map([], |row| {
                Ok(DismissedRow {
                    name: row.get(0)?,
                    kind: row.get(1)?,
                    notes: row.get::<_, i64>(2)? as u32,
                    at: row.get(3)?,
                })
            })?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }

    /// Hands every note detection may read to `each`: live, outside the
    /// trash, not locked, not tagged `#ai-exclude`. Notes are visited one
    /// at a time so a big library is never held in memory at once.
    pub fn for_each_detection_note(&self, mut each: impl FnMut(DetectNote)) -> Result<()> {
        self.read(|conn| {
            let sql = format!(
                "SELECT d.id, d.title, json_extract(d.json, '$.body'),
                        json_extract(d.json, '$.createdAt'), d.updated_at,
                        (SELECT group_concat(json_extract(t.json, '$.name'), char(31))
                           FROM note_tags nt JOIN docs t ON t.id = nt.tag_id
                           WHERE nt.note_id = d.id AND t.deleted = 0)
                 FROM docs d WHERE {} ORDER BY d.id",
                crate::schema::discoverable!("d.")
            );
            let mut stmt = conn.prepare(&sql)?;
            let mut rows = stmt.query([])?;
            while let Some(row) = rows.next()? {
                let tags: Option<String> = row.get(5)?;
                each(DetectNote {
                    id: row.get(0)?,
                    title: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                    body: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                    created_at: row.get::<_, Option<i64>>(3)?.unwrap_or(0),
                    updated_at: row.get::<_, Option<i64>>(4)?.unwrap_or(0),
                    tags: tags
                        .map(|t| t.split('\u{1f}').map(str::to_owned).collect())
                        .unwrap_or_default(),
                });
            }
            Ok(())
        })
    }

    /// The `kansoDismissed` names of notebook `id` (the notebook pages are
    /// kept in), which sync with it.
    pub fn book_dismissed(&self, id: &str) -> Result<Vec<String>> {
        Ok(match self.get(id)? {
            Some(doc) => doc
                .fields
                .get(DISMISSED_FIELD)
                .and_then(Value::as_array)
                .map(|list| {
                    list.iter()
                        .filter_map(Value::as_str)
                        .map(str::to_owned)
                        .collect()
                })
                .unwrap_or_default(),
            None => Vec::new(),
        })
    }

    /// Sets the `kansoDismissed` names of notebook `id`, sorted and capped.
    /// Nothing is written when they are what the notebook has.
    pub fn set_book_dismissed(&self, id: &str, names: &[String]) -> Result<()> {
        let mut names: Vec<String> = names.to_vec();
        names.sort();
        names.dedup();
        names.truncate(MAX_DISMISSED);
        let mut have = self.book_dismissed(id)?;
        have.sort();
        if have == names || self.get(id)?.is_none() {
            return Ok(());
        }
        self.put(
            kanso_core::DocType::Book,
            serde_json::json!({ "_id": id, DISMISSED_FIELD: names }),
            crate::Put::Merge,
            Origin::Ui,
        )?;
        Ok(())
    }
}

/// The notebook field the dismissed names sync in.
pub const DISMISSED_FIELD: &str = "kansoDismissed";
/// The most dismissed names kept.
pub const MAX_DISMISSED: usize = 2000;
