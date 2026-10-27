//! What the graph view draws: every live note outside the trash, and the
//! links in their bodies, from the `note_links` index. The app resolves
//! wiki-link titles and lays the graph out.

use std::collections::HashSet;

use kanso_core::links::normalize_title;

use crate::{Result, Store};

/// A note as the graph shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphNote {
    pub id: String,
    pub title: String,
    pub book_id: String,
    pub tags: Vec<String>,
}

/// One link in a note's body, as indexed: to a note's ID, or to a title
/// (normalized) for a wiki-link.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphLink {
    pub from: String,
    pub target_id: Option<String>,
    pub target_title: Option<String>,
}

impl Store {
    /// Every live note outside the trash, first created first (the order
    /// wiki-link titles resolve in), and every link those notes make.
    ///
    /// Locked notes (sealed, or in a locked notebook) are left out unless
    /// `show_locked` (the lock password has opened them this session), and
    /// so is every link to or from one: a title that a hidden note owns
    /// stays its own, so a link to it doesn't turn into one to another note
    /// with the same title. Notes tagged `#ai-exclude` stay: the graph
    /// is drawn on this device and sends nothing anywhere.
    pub fn graph_rows(&self, show_locked: bool) -> Result<(Vec<GraphNote>, Vec<GraphLink>)> {
        self.read(|conn| {
            let mut stmt = conn.prepare(concat!(
                "SELECT id, title, book_id, json_extract(json, '$.tags'), ",
                crate::schema::note_locked!(""),
                " FROM docs
                 WHERE type = 'note' AND deleted = 0 AND IFNULL(book_id, '') != 'trash'
                 ORDER BY created_at, id"
            ))?;
            let mut notes = Vec::new();
            // Hidden notes' IDs, and the titles that belong to one.
            let mut hidden: HashSet<String> = HashSet::new();
            let mut hidden_titles: HashSet<String> = HashSet::new();
            let mut taken: HashSet<String> = HashSet::new();
            let mut rows = stmt.query([])?;
            while let Some(row) = rows.next()? {
                let tags: Option<String> = row.get(3)?;
                let note = GraphNote {
                    id: row.get(0)?,
                    title: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                    book_id: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                    tags: tags
                        .and_then(|t| serde_json::from_str(&t).ok())
                        .unwrap_or_default(),
                };
                let locked: bool = row.get(4)?;
                let title = normalize_title(&note.title);
                // The first created keeps the title, hidden or not.
                let first = !title.is_empty() && taken.insert(title.clone());
                if locked && !show_locked {
                    if first {
                        hidden_titles.insert(title);
                    }
                    hidden.insert(note.id);
                } else {
                    notes.push(note);
                }
            }
            let mut stmt = conn.prepare(
                "SELECT l.note_id, l.target_id, l.target_title FROM note_links l
                 JOIN docs d ON d.id = l.note_id
                 WHERE d.deleted = 0 AND IFNULL(d.book_id, '') != 'trash'
                 ORDER BY l.note_id, l.ord",
            )?;
            let links = stmt
                .query_map([], |row| {
                    Ok(GraphLink {
                        from: row.get(0)?,
                        target_id: row.get(1)?,
                        target_title: row.get(2)?,
                    })
                })?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            let links = links
                .into_iter()
                .filter(|link| {
                    !hidden.contains(&link.from)
                        && !link
                            .target_id
                            .as_ref()
                            .is_some_and(|id| hidden.contains(id))
                        && !(link.target_id.is_none()
                            && link
                                .target_title
                                .as_ref()
                                .is_some_and(|title| hidden_titles.contains(title)))
                })
                .collect();
            Ok((notes, links))
        })
    }
}
