//! The reads behind a notebook's homepage (issue #201): the notebook's
//! subtree and the notes in it, its recent changes, how its notes link,
//! the tasks, people and notebooks in it, and its Home note.
//!
//! Every read leaves out what is locked: a locked notebook (and the ones
//! inside it) is never walked into, and a note that is sealed itself is
//! never a row. Not even a count includes them, so the numbers shown say
//! nothing about what is hidden. A notebook that is locked answers every
//! read here with nothing.

use std::collections::{BTreeMap, HashMap, HashSet};

use rusqlite::{Connection, params};

use crate::home_model::{
    HomeChild, HomeEntity, HomeLinked, HomeNoteMark, HomeNoteRow, HomeStats,
};
use crate::schema::note_locked;
use crate::{GraphLink, GraphNote, Result, Store};

/// The notebook and the unlocked notebooks inside it, as `sub(id)`.
fn cte(include_locked: bool) -> &'static str {
    if include_locked {
        "WITH RECURSIVE sub(id) AS (SELECT ?1 UNION SELECT b.id FROM docs b \
         JOIN sub ON b.parent_book_id = sub.id WHERE b.type = 'book' AND b.deleted = 0)"
    } else {
        "WITH RECURSIVE sub(id) AS (SELECT ?1 UNION SELECT b.id FROM docs b \
         JOIN sub ON b.parent_book_id = sub.id WHERE b.type = 'book' AND b.deleted = 0 \
         AND b.id NOT IN (SELECT id FROM locked_books))"
    }
}

/// A live note of the subtree that is not sealed, for alias `d`.
macro_rules! in_subtree {
    ($d:literal) => {
        concat!(
            $d,
            "type = 'note' AND ",
            $d,
            "deleted = 0 AND ",
            $d,
            "book_id IN (SELECT id FROM sub) AND NOT ",
            note_locked!($d)
        )
    };
}

/// Whether notebook `id` is a live notebook that is not locked.
fn readable(conn: &Connection, id: &str) -> Result<bool> {
    let live: bool = conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM docs WHERE id = ?1 AND type = 'book' AND deleted = 0)
         AND ?1 NOT IN (SELECT id FROM locked_books)",
        [id],
        |row| row.get(0),
    )?;
    Ok(live)
}

fn note_row(row: &rusqlite::Row) -> rusqlite::Result<HomeNoteRow> {
    Ok(HomeNoteRow {
        id: row.get(0)?,
        title: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
        book_id: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
        book_name: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
        created_at: row.get::<_, Option<i64>>(4)?.unwrap_or(0),
        updated_at: row.get::<_, Option<i64>>(5)?.unwrap_or(0),
        pinned: row.get::<_, Option<i64>>(6)?.unwrap_or(0) != 0,
        revisions: row.get::<_, Option<i64>>(7)?.unwrap_or(0) as u32,
    })
}

const NOTE_COLUMNS: &str = "d.id, d.title, d.book_id, b.name, d.created_at, d.updated_at, \
     d.pinned, 0";

impl Store {
    /// Notebook `id` and the notebooks inside it, `id` first. With
    /// `include_locked` false the walk stops at a locked notebook: neither
    /// it nor anything inside it is listed (and a locked `id` gives
    /// nothing).
    pub fn book_subtree(&self, id: &str, include_locked: bool) -> Result<Vec<String>> {
        self.read(|conn| {
            if !include_locked && !readable(conn, id)? {
                return Ok(Vec::new());
            }
            let mut stmt = conn.prepare(&format!("{} SELECT id FROM sub", cte(include_locked)))?;
            let rows = stmt.query_map([id], |row| row.get::<_, String>(0))?;
            let mut out: Vec<String> = rows.collect::<rusqlite::Result<_>>()?;
            if let Some(at) = out.iter().position(|b| b == id) {
                out.swap(0, at);
            }
            Ok(out)
        })
    }

    /// How many notes and notebooks the header line counts, and the latest
    /// edit.
    pub fn home_stats(&self, id: &str) -> Result<HomeStats> {
        self.read(|conn| {
            if !readable(conn, id)? {
                return Ok(HomeStats::default());
            }
            let (notes, last): (i64, Option<i64>) = conn.query_row(
                &format!(
                    "{} SELECT COUNT(*), MAX(d.updated_at) FROM docs d WHERE {}",
                    cte(false),
                    in_subtree!("d.")
                ),
                [id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )?;
            let books: i64 = conn.query_row(
                &format!("{} SELECT COUNT(*) FROM sub", cte(false)),
                [id],
                |row| row.get(0),
            )?;
            Ok(HomeStats {
                notes: notes as u32,
                notebooks_inside: (books - 1).max(0) as u32,
                last_edited: last,
            })
        })
    }

    /// The notes changed most recently in the subtree, newest first.
    pub fn home_recent(&self, id: &str, limit: usize) -> Result<Vec<HomeNoteRow>> {
        self.home_notes(
            id,
            "ORDER BY d.updated_at DESC, d.id LIMIT ?2",
            limit as i64,
        )
    }

    /// Pinned notes of the subtree, newest first.
    pub fn home_pinned(&self, id: &str, limit: usize) -> Result<Vec<HomeNoteRow>> {
        self.home_notes(
            id,
            "AND d.pinned = 1 ORDER BY d.updated_at DESC, d.id LIMIT ?2",
            limit as i64,
        )
    }

    /// Notes of the subtree changed since `since_ms`, most saved versions
    /// first (an edit count), then newest.
    pub fn home_edited_since(
        &self,
        id: &str,
        since_ms: i64,
        limit: usize,
    ) -> Result<Vec<HomeNoteRow>> {
        self.read(|conn| {
            if !readable(conn, id)? {
                return Ok(Vec::new());
            }
            let mut stmt = conn.prepare(&format!(
                "{} SELECT d.id, d.title, d.book_id, b.name, d.created_at, d.updated_at, d.pinned, \
                 (SELECT COUNT(*) FROM revisions r WHERE r.doc_id = d.id) \
                 FROM docs d LEFT JOIN docs b ON b.id = d.book_id \
                 WHERE {} AND d.updated_at >= ?3 \
                 ORDER BY 8 DESC, d.updated_at DESC, d.id LIMIT ?2",
                cte(false),
                in_subtree!("d.")
            ))?;
            let rows = stmt.query_map(params![id, limit as i64, since_ms], note_row)?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }

    /// The notes of the subtree by ID, for titles and dates.
    pub fn home_notes_by_id(&self, id: &str, ids: &[String]) -> Result<Vec<HomeNoteRow>> {
        let wanted: HashSet<&str> = ids.iter().map(String::as_str).collect();
        Ok(self
            .home_notes(id, "ORDER BY d.created_at, d.id LIMIT ?2", -1)?
            .into_iter()
            .filter(|n| wanted.contains(n.id.as_str()))
            .collect())
    }

    /// Every note of the subtree, first created first.
    pub fn home_all_notes(&self, id: &str) -> Result<Vec<HomeNoteRow>> {
        self.home_notes(id, "ORDER BY d.created_at, d.id LIMIT ?2", -1)
    }

    fn home_notes(&self, id: &str, tail: &str, limit: i64) -> Result<Vec<HomeNoteRow>> {
        self.read(|conn| {
            if !readable(conn, id)? {
                return Ok(Vec::new());
            }
            let mut stmt = conn.prepare(&format!(
                "{} SELECT {NOTE_COLUMNS} FROM docs d LEFT JOIN docs b ON b.id = d.book_id \
                 WHERE {} {tail}",
                cte(false),
                in_subtree!("d.")
            ))?;
            let rows = stmt.query_map(params![id, limit], note_row)?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }

    /// For each note of the subtree linked to from other notes of it: how
    /// many notes link to it, most first, then by ID. Wiki-links count
    /// through the note their title names; links from or to anything
    /// outside the subtree, and a note's links to itself, are not counted.
    pub fn home_linked(&self, id: &str, limit: usize) -> Result<Vec<HomeLinked>> {
        let (by_id, by_title, inside) = self.read(|conn| {
            if !readable(conn, id)? {
                return Ok((Vec::new(), Vec::new(), HashSet::new()));
            }
            let mut ids = conn.prepare(&format!(
                "{} SELECT d.id FROM docs d WHERE {}",
                cte(false),
                in_subtree!("d.")
            ))?;
            let inside: HashSet<String> = ids
                .query_map([id], |row| row.get(0))?
                .collect::<rusqlite::Result<_>>()?;
            let mut stmt = conn.prepare(&format!(
                "{} SELECT l.note_id, l.target_id, l.target_title FROM note_links l \
                 JOIN docs d ON d.id = l.note_id WHERE {}",
                cte(false),
                in_subtree!("d.")
            ))?;
            let mut by_id = Vec::new();
            let mut by_title = Vec::new();
            let rows = stmt.query_map([id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            })?;
            for row in rows {
                let (from, target, title) = row?;
                match (target, title) {
                    (Some(target), _) => by_id.push((from, target)),
                    (None, Some(title)) => by_title.push((from, title)),
                    (None, None) => {}
                }
            }
            Ok((by_id, by_title, inside))
        })?;
        let mut sources: HashMap<String, HashSet<String>> = HashMap::new();
        for (from, target) in by_id {
            if from != target && inside.contains(&target) {
                sources.entry(target).or_default().insert(from);
            }
        }
        let mut resolved: HashMap<String, Option<String>> = HashMap::new();
        for (from, title) in by_title {
            if !resolved.contains_key(&title) {
                let found = self.resolve_title(&title)?.map(|(id, _)| id);
                resolved.insert(title.clone(), found);
            }
            if let Some(Some(target)) = resolved.get(&title)
                && *target != from
                && inside.contains(target)
            {
                sources.entry(target.clone()).or_default().insert(from);
            }
        }
        let mut out: Vec<HomeLinked> = sources
            .into_iter()
            .map(|(note_id, from)| HomeLinked {
                note_id,
                linked_from: from.len() as u32,
            })
            .collect();
        out.sort_by(|a, b| {
            b.linked_from
                .cmp(&a.linked_from)
                .then_with(|| a.note_id.cmp(&b.note_id))
        });
        out.truncate(limit);
        Ok(out)
    }

    /// The notebooks directly inside `id`, with counts for the unlocked
    /// ones. A locked one is listed by name alone.
    pub fn home_children(&self, id: &str) -> Result<Vec<HomeChild>> {
        let kids: Vec<(String, String, bool)> = self.read(|conn| {
            if !readable(conn, id)? {
                return Ok(Vec::new());
            }
            let mut stmt = conn.prepare(
                "SELECT b.id, IFNULL(b.name, ''), b.id IN (SELECT id FROM locked_books) \
                 FROM docs b WHERE b.type = 'book' AND b.deleted = 0 AND b.parent_book_id = ?1 \
                 ORDER BY IFNULL(json_extract(b.json, '$.order'), 9223372036854775807), b.name, b.id",
            )?;
            let rows = stmt.query_map([id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })?;
        let mut out = Vec::new();
        for (kid, name, locked) in kids {
            if locked {
                out.push(HomeChild {
                    id: kid,
                    name,
                    locked: true,
                    notes: 0,
                    last_edited: None,
                    open_tasks: 0,
                });
                continue;
            }
            let stats = self.home_stats(&kid)?;
            let open_tasks = self.home_open_task_count(&kid)?;
            out.push(HomeChild {
                id: kid,
                name,
                locked: false,
                notes: stats.notes,
                last_edited: stats.last_edited,
                open_tasks,
            });
        }
        Ok(out)
    }

    /// How many unchecked tasks the subtree's notes hold.
    pub fn home_open_task_count(&self, id: &str) -> Result<u32> {
        self.read(|conn| {
            if !readable(conn, id)? {
                return Ok(0);
            }
            Ok(conn.query_row(
                &format!(
                    "{} SELECT COUNT(*) FROM note_tasks t JOIN docs d ON d.id = t.note_id \
                     WHERE t.checked = 0 AND {}",
                    cte(false),
                    in_subtree!("d.")
                ),
                [id],
                |row| row.get::<_, i64>(0),
            )? as u32)
        })
    }

    /// How many tasks (done or not) the subtree's notes hold.
    pub fn home_task_count(&self, id: &str) -> Result<u32> {
        self.read(|conn| {
            if !readable(conn, id)? {
                return Ok(0);
            }
            Ok(conn.query_row(
                &format!(
                    "{} SELECT COUNT(*) FROM note_tasks t JOIN docs d ON d.id = t.note_id \
                     WHERE {}",
                    cte(false),
                    in_subtree!("d.")
                ),
                [id],
                |row| row.get::<_, i64>(0),
            )? as u32)
        })
    }

    /// The people and projects the subtree's notes mention, by number of
    /// notes, then name.
    pub fn home_entities(&self, id: &str, limit: usize) -> Result<Vec<HomeEntity>> {
        self.read(|conn| {
            if !readable(conn, id)? {
                return Ok(Vec::new());
            }
            let mut stmt = conn.prepare(&format!(
                "{} SELECT e.note_id, IFNULL(p.title, ''), e.kind, COUNT(DISTINCT m.note_id) AS c \
                 FROM note_mentions m \
                 JOIN docs d ON d.id = m.note_id \
                 JOIN entities e ON e.note_id = m.entity_id \
                 JOIN docs p ON p.id = e.note_id AND p.deleted = 0 \
                 WHERE {} AND m.note_id != m.entity_id AND NOT {} \
                 GROUP BY e.note_id ORDER BY c DESC, p.title, e.note_id LIMIT ?2",
                cte(false),
                in_subtree!("d."),
                note_locked!("p.")
            ))?;
            let rows = stmt.query_map(params![id, limit as i64], |row| {
                Ok(HomeEntity {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    kind: row.get(2)?,
                    notes: row.get::<_, i64>(3)? as u32,
                })
            })?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }

    /// The notes and links the notebook's map is drawn from: every note of
    /// the subtree (first created first, as wiki-link titles resolve) and
    /// the links they make. Locked notes and notebooks are not in it.
    pub fn home_graph_rows(&self, id: &str) -> Result<(Vec<GraphNote>, Vec<GraphLink>)> {
        self.read(|conn| {
            if !readable(conn, id)? {
                return Ok((Vec::new(), Vec::new()));
            }
            let mut stmt = conn.prepare(&format!(
                "{} SELECT d.id, d.title, d.book_id, json_extract(d.json, '$.tags') FROM docs d \
                 WHERE {} ORDER BY d.created_at, d.id",
                cte(false),
                in_subtree!("d.")
            ))?;
            let notes = stmt
                .query_map([id], |row| {
                    let tags: Option<String> = row.get(3)?;
                    Ok(GraphNote {
                        id: row.get(0)?,
                        title: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                        book_id: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                        tags: tags
                            .and_then(|t| serde_json::from_str(&t).ok())
                            .unwrap_or_default(),
                    })
                })?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            let mut stmt = conn.prepare(&format!(
                "{} SELECT l.note_id, l.target_id, l.target_title FROM note_links l \
                 JOIN docs d ON d.id = l.note_id WHERE {} ORDER BY l.note_id, l.ord",
                cte(false),
                in_subtree!("d.")
            ))?;
            let links = stmt
                .query_map([id], |row| {
                    Ok(GraphLink {
                        from: row.get(0)?,
                        target_id: row.get(1)?,
                        target_title: row.get(2)?,
                    })
                })?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            Ok((notes, links))
        })
    }

    /// The notes of notebook `book` (directly in it) marked as its Home
    /// note, newest first. More than one happens when two devices chose
    /// different notes offline: the first is the one shown.
    pub fn home_notes_marked(&self, book: &str) -> Result<Vec<HomeNoteMark>> {
        self.read(|conn| {
            if !readable(conn, book)? {
                return Ok(Vec::new());
            }
            let mut stmt = conn.prepare(&format!(
                "SELECT d.id, IFNULL(d.title, ''), IFNULL(d.updated_at, 0), json_extract(d.json, '$.body') \
                 FROM docs d WHERE d.type = 'note' AND d.deleted = 0 AND d.book_id = ?1 \
                 AND NOT {} AND instr(json_extract(d.json, '$.body'), 'kanso') > 0 \
                 ORDER BY d.updated_at DESC, d.id",
                note_locked!("d.")
            ))?;
            let rows = stmt.query_map([book], |row| {
                Ok((
                    HomeNoteMark {
                        id: row.get(0)?,
                        title: row.get(1)?,
                        updated_at: row.get(2)?,
                    },
                    row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                ))
            })?;
            let mut out = Vec::new();
            for row in rows {
                let (mark, body) = row?;
                if kanso_core::home::is_home(&body) {
                    out.push(mark);
                }
            }
            Ok(out)
        })
    }

    /// Notebook `book`'s Home note, if it has one: the newest marked.
    pub fn home_note(&self, book: &str) -> Result<Option<HomeNoteMark>> {
        Ok(self.home_notes_marked(book)?.into_iter().next())
    }

    /// The notebooks that are not locked, with their parents: a map from
    /// each notebook to its name, for labelling activity.
    pub fn home_book_names(&self) -> Result<BTreeMap<String, String>> {
        self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, IFNULL(name, '') FROM docs WHERE type = 'book' AND deleted = 0 \
                 AND id NOT IN (SELECT id FROM locked_books)",
            )?;
            let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }
}
