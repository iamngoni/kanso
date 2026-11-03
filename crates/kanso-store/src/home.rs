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

use crate::TaskRow;
use crate::home_model::{
    HomeBook, HomeChild, HomeEntity, HomeLinked, HomeNoteMark, HomeNoteRow, HomeScan, HomeStats,
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

/// `ids` as a JSON array, for `json_each`: the tables kept per note are
/// read by note ID, never by walking the notebook's documents again.
fn ids_json<'a>(ids: impl Iterator<Item = &'a str>) -> String {
    serde_json::to_string(&ids.collect::<Vec<_>>()).unwrap_or_else(|_| "[]".to_owned())
}

impl HomeScan {
    /// What the header line counts.
    pub fn stats(&self) -> HomeStats {
        HomeStats {
            notes: self.notes.len() as u32,
            notebooks_inside: self.books.len().saturating_sub(1) as u32,
            last_edited: self.notes.iter().map(|n| n.updated_at).max(),
        }
    }

    /// Pinned notes, newest first.
    pub fn pinned(&self, limit: usize) -> Vec<HomeNoteRow> {
        let mut out: Vec<HomeNoteRow> = self.notes.iter().filter(|n| n.pinned).cloned().collect();
        out.sort_by(|a, b| {
            b.updated_at
                .cmp(&a.updated_at)
                .then_with(|| a.id.cmp(&b.id))
        });
        out.truncate(limit);
        out
    }

    fn ids_json(&self) -> String {
        ids_json(self.notes.iter().map(|n| n.id.as_str()))
    }
}

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

    /// Reads notebook `id`'s subtree once: its unlocked notebooks and their
    /// readable notes. Empty when the notebook is gone or locked.
    pub fn home_scan(&self, id: &str) -> Result<HomeScan> {
        self.read(|conn| {
            let mut scan = HomeScan {
                id: id.to_owned(),
                ..HomeScan::default()
            };
            if !readable(conn, id)? {
                return Ok(scan);
            }
            let mut stmt = conn.prepare(&format!(
                "{} SELECT b.id, IFNULL(b.name, ''), b.parent_book_id FROM sub s JOIN docs b ON b.id = s.id",
                cte(false)
            ))?;
            let rows = stmt.query_map([id], |row| {
                Ok(HomeBook {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    parent: row.get(2)?,
                })
            })?;
            scan.books = rows.collect::<rusqlite::Result<_>>()?;
            if let Some(at) = scan.books.iter().position(|b| b.id == id) {
                scan.books.swap(0, at);
            }
            let names: HashMap<&str, &str> = scan
                .books
                .iter()
                .map(|b| (b.id.as_str(), b.name.as_str()))
                .collect();
            let mut stmt = conn.prepare(&format!(
                "{} SELECT d.id, d.title, d.book_id, d.created_at, d.updated_at, d.pinned \
                 FROM sub s JOIN docs d INDEXED BY docs_book ON d.type = 'note' AND d.book_id = s.id \
                 WHERE {} ORDER BY d.created_at, d.id",
                cte(false),
                in_subtree!("d.")
            ))?;
            let rows = stmt.query_map([id], |row| {
                let book_id = row.get::<_, Option<String>>(2)?.unwrap_or_default();
                Ok(HomeNoteRow {
                    id: row.get(0)?,
                    title: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                    book_name: names.get(book_id.as_str()).copied().unwrap_or("").to_owned(),
                    book_id,
                    created_at: row.get::<_, Option<i64>>(3)?.unwrap_or(0),
                    updated_at: row.get::<_, Option<i64>>(4)?.unwrap_or(0),
                    pinned: row.get::<_, Option<i64>>(5)?.unwrap_or(0) != 0,
                    revisions: 0,
                })
            })?;
            scan.notes = rows.collect::<rusqlite::Result<_>>()?;
            Ok(scan)
        })
    }

    /// How many notes and notebooks the header line counts, and the latest
    /// edit.
    pub fn home_stats(&self, id: &str) -> Result<HomeStats> {
        Ok(self.home_scan(id)?.stats())
    }

    /// The notes changed most recently in the subtree, newest first.
    pub fn home_recent(&self, id: &str, limit: usize) -> Result<Vec<HomeNoteRow>> {
        self.read(|conn| {
            if !readable(conn, id)? {
                return Ok(Vec::new());
            }
            let mut stmt = conn.prepare(&format!(
                "{} SELECT {NOTE_COLUMNS} FROM docs d LEFT JOIN docs b ON b.id = d.book_id \
                 WHERE {} ORDER BY d.updated_at DESC, d.id LIMIT ?2",
                cte(false),
                in_subtree!("d.")
            ))?;
            let rows = stmt.query_map(params![id, limit as i64], note_row)?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }

    /// Pinned notes of the subtree, newest first.
    pub fn home_pinned(&self, id: &str, limit: usize) -> Result<Vec<HomeNoteRow>> {
        Ok(self.home_scan(id)?.pinned(limit))
    }

    /// Notes of the subtree changed since `since_ms`, most saved versions
    /// first (an edit count), then newest.
    pub fn home_edited_since(
        &self,
        id: &str,
        since_ms: i64,
        limit: usize,
    ) -> Result<Vec<HomeNoteRow>> {
        self.home_edited_since_in(&self.home_scan(id)?, since_ms, limit)
    }

    /// [`Store::home_edited_since`] over a scan already read.
    pub fn home_edited_since_in(
        &self,
        scan: &HomeScan,
        since_ms: i64,
        limit: usize,
    ) -> Result<Vec<HomeNoteRow>> {
        let mut out: Vec<HomeNoteRow> = scan
            .notes
            .iter()
            .filter(|n| n.updated_at >= since_ms)
            .cloned()
            .collect();
        if out.is_empty() {
            return Ok(out);
        }
        let counts: HashMap<String, u32> = self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT r.doc_id, COUNT(*) FROM json_each(?1) j \
                 CROSS JOIN revisions r ON r.doc_id = j.value GROUP BY r.doc_id",
            )?;
            let rows = stmt.query_map([ids_json(out.iter().map(|n| n.id.as_str()))], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as u32))
            })?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })?;
        for note in &mut out {
            note.revisions = counts.get(&note.id).copied().unwrap_or(0);
        }
        out.sort_by(|a, b| {
            b.revisions
                .cmp(&a.revisions)
                .then_with(|| b.updated_at.cmp(&a.updated_at))
                .then_with(|| a.id.cmp(&b.id))
        });
        out.truncate(limit);
        Ok(out)
    }

    /// The notes of the subtree by ID, for titles and dates.
    pub fn home_notes_by_id(&self, id: &str, ids: &[String]) -> Result<Vec<HomeNoteRow>> {
        let wanted: HashSet<&str> = ids.iter().map(String::as_str).collect();
        Ok(self
            .home_scan(id)?
            .notes
            .into_iter()
            .filter(|n| wanted.contains(n.id.as_str()))
            .collect())
    }

    /// Every note of the subtree, first created first.
    pub fn home_all_notes(&self, id: &str) -> Result<Vec<HomeNoteRow>> {
        Ok(self.home_scan(id)?.notes)
    }

    /// The links the subtree's notes make: `(from, target ID, wiki-link
    /// title)`, each note's in order.
    fn home_links_in(&self, scan: &HomeScan) -> Result<Vec<HomeLink>> {
        if scan.notes.is_empty() {
            return Ok(Vec::new());
        }
        self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT l.note_id, l.target_id, l.target_title FROM json_each(?1) j \
                 CROSS JOIN note_links l ON l.note_id = j.value ORDER BY l.note_id, l.ord",
            )?;
            let rows = stmt.query_map([scan.ids_json()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            })?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }

    /// For each note of the subtree linked to from other notes of it: how
    /// many notes link to it, most first, then by ID. Wiki-links count
    /// through the note their title names; links from or to anything
    /// outside the subtree, and a note's links to itself, are not counted.
    pub fn home_linked(&self, id: &str, limit: usize) -> Result<Vec<HomeLinked>> {
        self.home_linked_in(&self.home_scan(id)?, limit)
    }

    /// [`Store::home_linked`] over a scan already read.
    pub fn home_linked_in(&self, scan: &HomeScan, limit: usize) -> Result<Vec<HomeLinked>> {
        let inside: HashSet<&str> = scan.notes.iter().map(|n| n.id.as_str()).collect();
        let mut sources: HashMap<String, HashSet<String>> = HashMap::new();
        let mut resolved: HashMap<String, Option<String>> = HashMap::new();
        for (from, target, title) in self.home_links_in(scan)? {
            let target = match (target, title) {
                (Some(target), _) => target,
                (None, Some(title)) => {
                    if !resolved.contains_key(&title) {
                        let found = self.resolve_title(&title)?.map(|(id, _)| id);
                        resolved.insert(title.clone(), found);
                    }
                    match resolved.get(&title) {
                        Some(Some(target)) => target.clone(),
                        _ => continue,
                    }
                }
                (None, None) => continue,
            };
            if target != from && inside.contains(target.as_str()) {
                sources.entry(target).or_default().insert(from);
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

    /// For each note of the subtree with tasks: `(unchecked, all)`.
    pub fn home_task_counts_in(&self, scan: &HomeScan) -> Result<HashMap<String, (u32, u32)>> {
        if scan.notes.is_empty() {
            return Ok(HashMap::new());
        }
        self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT t.note_id, SUM(t.checked = 0), COUNT(*) FROM json_each(?1) j \
                 CROSS JOIN note_tasks t ON t.note_id = j.value GROUP BY t.note_id",
            )?;
            let rows = stmt.query_map([scan.ids_json()], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    (row.get::<_, i64>(1)? as u32, row.get::<_, i64>(2)? as u32),
                ))
            })?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }

    /// The subtree's unchecked tasks as [`Store::tasks`] orders them: dated
    /// first, soonest first, then the rest, newest note first, each note's
    /// in order. At most `limit`.
    pub fn home_open_tasks_in(&self, scan: &HomeScan, limit: usize) -> Result<Vec<TaskRow>> {
        if scan.notes.is_empty() {
            return Ok(Vec::new());
        }
        let notes: HashMap<&str, &HomeNoteRow> =
            scan.notes.iter().map(|n| (n.id.as_str(), n)).collect();
        // Ordered on the narrow columns first; the words are read only for
        // the tasks kept, so a notebook of thousands of tasks reads five.
        let keys: Vec<(String, i64, Option<String>)> = self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT t.note_id, t.ord, t.due FROM json_each(?1) j \
                 CROSS JOIN note_tasks t ON t.note_id = j.value WHERE t.checked = 0",
            )?;
            let rows = stmt.query_map([scan.ids_json()], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })?;
        let mut keys: Vec<(&HomeNoteRow, i64, Option<String>)> = keys
            .into_iter()
            .filter_map(|(id, ord, due)| notes.get(id.as_str()).map(|n| (*n, ord, due)))
            .collect();
        let order = |a: &(&HomeNoteRow, i64, Option<String>),
                     b: &(&HomeNoteRow, i64, Option<String>)| {
            a.2.is_none()
                .cmp(&b.2.is_none())
                .then_with(|| a.2.cmp(&b.2))
                .then_with(|| b.0.updated_at.cmp(&a.0.updated_at))
                .then_with(|| a.0.id.cmp(&b.0.id))
                .then_with(|| a.1.cmp(&b.1))
        };
        if keys.len() > limit && limit > 0 {
            keys.select_nth_unstable_by(limit - 1, order);
        }
        keys.truncate(limit);
        keys.sort_by(order);
        self.read(|conn| {
            let mut stmt = conn.prepare_cached(
                "SELECT line, text FROM note_tasks WHERE note_id = ?1 AND ord = ?2",
            )?;
            let mut out = Vec::with_capacity(keys.len());
            for (note, ord, due) in keys {
                let (line, text): (i64, String) =
                    stmt.query_row(params![note.id, ord], |row| Ok((row.get(0)?, row.get(1)?)))?;
                out.push(TaskRow {
                    note_id: note.id.clone(),
                    note_title: note.title.clone(),
                    book_id: note.book_id.clone(),
                    line: line as u32,
                    checked: false,
                    text,
                    due,
                    note_updated_at: note.updated_at,
                });
            }
            Ok(out)
        })
    }

    /// The notebooks directly inside `id`, with counts for the unlocked
    /// ones. A locked one is listed by name alone.
    pub fn home_children(&self, id: &str) -> Result<Vec<HomeChild>> {
        let scan = self.home_scan(id)?;
        let counts = self.home_task_counts_in(&scan)?;
        self.home_children_in(&scan, &counts)
    }

    /// [`Store::home_children`] over a scan and its task counts.
    pub fn home_children_in(
        &self,
        scan: &HomeScan,
        task_counts: &HashMap<String, (u32, u32)>,
    ) -> Result<Vec<HomeChild>> {
        if scan.books.is_empty() {
            return Ok(Vec::new());
        }
        let kids: Vec<(String, String, bool)> = self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT b.id, IFNULL(b.name, ''), b.id IN (SELECT id FROM locked_books) \
                 FROM docs b WHERE b.type = 'book' AND b.deleted = 0 AND b.parent_book_id = ?1 \
                 ORDER BY IFNULL(json_extract(b.json, '$.order'), 9223372036854775807), b.name, b.id",
            )?;
            let rows =
                stmt.query_map([&scan.id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })?;
        // Each notebook of the subtree, to the child of `scan.id` it is in.
        let parents: HashMap<&str, Option<&str>> = scan
            .books
            .iter()
            .map(|b| (b.id.as_str(), b.parent.as_deref()))
            .collect();
        fn child_of<'a>(
            parents: &HashMap<&'a str, Option<&'a str>>,
            top: &str,
            book: &'a str,
        ) -> Option<&'a str> {
            let mut at = book;
            for _ in 0..parents.len() {
                let parent = parents.get(at).copied().flatten()?;
                if parent == top {
                    return Some(at);
                }
                at = parent;
            }
            None
        }
        let mut tally: HashMap<&str, (u32, Option<i64>, u32)> = HashMap::new();
        for note in &scan.notes {
            let Some(kid) = child_of(&parents, &scan.id, &note.book_id) else {
                continue;
            };
            let entry = tally.entry(kid).or_default();
            entry.0 += 1;
            entry.1 = Some(
                entry
                    .1
                    .map_or(note.updated_at, |at| at.max(note.updated_at)),
            );
            entry.2 += task_counts.get(&note.id).map_or(0, |c| c.0);
        }
        Ok(kids
            .into_iter()
            .map(|(kid, name, locked)| {
                let (notes, last_edited, open_tasks) = if locked {
                    (0, None, 0)
                } else {
                    tally.get(kid.as_str()).copied().unwrap_or_default()
                };
                HomeChild {
                    id: kid,
                    name,
                    locked,
                    notes,
                    last_edited,
                    open_tasks,
                }
            })
            .collect())
    }

    /// How many unchecked tasks the subtree's notes hold.
    pub fn home_open_task_count(&self, id: &str) -> Result<u32> {
        let counts = self.home_task_counts_in(&self.home_scan(id)?)?;
        Ok(counts.values().map(|c| c.0).sum())
    }

    /// How many tasks (done or not) the subtree's notes hold.
    pub fn home_task_count(&self, id: &str) -> Result<u32> {
        let counts = self.home_task_counts_in(&self.home_scan(id)?)?;
        Ok(counts.values().map(|c| c.1).sum())
    }

    /// The people and projects the subtree's notes mention, by number of
    /// notes, then name.
    pub fn home_entities(&self, id: &str, limit: usize) -> Result<Vec<HomeEntity>> {
        self.home_entities_in(&self.home_scan(id)?, limit)
    }

    /// [`Store::home_entities`] over a scan already read.
    pub fn home_entities_in(&self, scan: &HomeScan, limit: usize) -> Result<Vec<HomeEntity>> {
        if scan.notes.is_empty() {
            return Ok(Vec::new());
        }
        self.read(|conn| {
            let mut stmt = conn.prepare(&format!(
                "SELECT e.note_id, IFNULL(p.title, ''), e.kind, COUNT(DISTINCT m.note_id) AS c \
                 FROM json_each(?1) j \
                 CROSS JOIN note_mentions m ON m.note_id = j.value \
                 JOIN entities e ON e.note_id = m.entity_id \
                 JOIN docs p ON p.id = e.note_id AND p.deleted = 0 \
                 WHERE m.note_id != m.entity_id AND NOT {} \
                 GROUP BY e.note_id ORDER BY c DESC, p.title, e.note_id LIMIT ?2",
                note_locked!("p.")
            ))?;
            let rows = stmt.query_map(params![scan.ids_json(), limit as i64], |row| {
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
                "{} SELECT l.note_id, l.target_id, l.target_title FROM sub s JOIN docs d INDEXED BY docs_book ON d.type = 'note' AND d.book_id = s.id \
                 JOIN note_links l ON l.note_id = d.id WHERE {} ORDER BY l.note_id, l.ord",
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
                 FROM docs d INDEXED BY docs_book WHERE d.type = 'note' AND d.deleted = 0 AND d.book_id = ?1 \
                 AND NOT {} AND d.json LIKE '%kanso%home%' \
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

/// One link a subtree note makes: `(from, target ID, wiki-link title)`.
type HomeLink = (String, Option<String>, Option<String>);
