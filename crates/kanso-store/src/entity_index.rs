//! The derived tables of people and projects: which notes are pages for an
//! entity, the names each answers to, and the lines that mention one.
//!
//! Kept by [`crate::derived::write_note`] on every write, like `note_links`.
//! A locked note (or one in a locked notebook) and a note in the trash
//! neither is an entity nor mentions one: its body is empty here.

use std::collections::{BTreeSet, HashMap, HashSet};

use kanso_core::entities::{self, scan_mentions};
use rusqlite::{Connection, params};

use crate::{Result, schema};

/// What was true of a note before it was written: the names it answered
/// to as an entity, and the notes that mentioned it.
#[derive(Default)]
pub(crate) struct Before {
    names: BTreeSet<String>,
    was_entity: bool,
    mentioned_in: Vec<String>,
}

/// Reads what the tables say about note `id`, before its rows are
/// replaced.
pub(crate) fn before(conn: &Connection, id: &str) -> Result<Before> {
    let names: BTreeSet<String> = conn
        .prepare_cached("SELECT name FROM entity_names WHERE note_id = ?")?
        .query_map([id], |row| row.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    let was_entity = !names.is_empty();
    let mentioned_in = if was_entity {
        conn.prepare_cached("SELECT DISTINCT note_id FROM note_mentions WHERE entity_id = ?")?
            .query_map([id], |row| row.get(0))?
            .collect::<rusqlite::Result<_>>()?
    } else {
        Vec::new()
    };
    Ok(Before {
        names,
        was_entity,
        mentioned_in,
    })
}

/// A note as the index sees it.
pub(crate) struct Indexed<'a> {
    pub id: &'a str,
    pub title: &'a str,
    /// Empty for a locked note.
    pub body: &'a str,
    pub trashed: bool,
}

/// Brings the entity tables up to date for the note just written (its own
/// rows were emptied first): whether it is an entity now, what it mentions,
/// and, when its names changed, which other notes mention it.
pub(crate) fn write(conn: &Connection, note: &Indexed, before: Before) -> Result<()> {
    let live = !note.trashed && !note.body.is_empty();
    let kind = if live {
        entities::kind_of(note.body)
    } else {
        None
    };
    let names: BTreeSet<String> = match kind {
        Some(_) => entities::names_of(note.title, note.body)
            .into_iter()
            .collect(),
        None => BTreeSet::new(),
    };
    if let Some(kind) = kind {
        let name = entities::normalize_name(note.title);
        conn.prepare_cached(
            "INSERT OR REPLACE INTO entities (note_id, kind, name) VALUES (?1, ?2, ?3)",
        )?
        .execute(params![note.id, kind.as_str(), name])?;
        let mut stmt = conn
            .prepare_cached("INSERT OR IGNORE INTO entity_names (name, note_id) VALUES (?1, ?2)")?;
        for name in &names {
            stmt.execute(params![name, note.id])?;
        }
    }
    let changed = names != before.names || kind.is_some() != before.was_entity;
    if changed {
        conn.execute("DELETE FROM note_mentions WHERE entity_id = ?", [note.id])?;
        let mut affected: BTreeSet<String> = before.names.clone();
        affected.extend(names.iter().cloned());
        let mut rescan: BTreeSet<String> = before.mentioned_in.into_iter().collect();
        rescan.extend(notes_containing(conn, &affected)?);
        rescan.remove(note.id);
        for other in rescan {
            rescan_note(conn, &other)?;
        }
    }
    if live {
        write_mentions(conn, note.id, note.body)?;
    }
    Ok(())
}

/// Every name of every entity, as a map to the one entity that has it, or
/// `None` when more than one does.
fn name_map(conn: &Connection) -> Result<HashMap<String, Option<String>>> {
    let mut map: HashMap<String, Option<String>> = HashMap::new();
    let mut stmt = conn.prepare_cached("SELECT name, note_id FROM entity_names")?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    for row in rows {
        let (name, id) = row?;
        map.entry(name)
            .and_modify(|slot| *slot = None)
            .or_insert(Some(id));
    }
    Ok(map)
}

/// Replaces note `id`'s mention rows with those its `body` makes.
fn write_mentions(conn: &Connection, id: &str, body: &str) -> Result<()> {
    conn.execute("DELETE FROM note_mentions WHERE note_id = ?", [id])?;
    let names = name_map(conn)?;
    if names.is_empty() {
        return Ok(());
    }
    let mut stmt = conn.prepare_cached(
        "INSERT OR IGNORE INTO note_mentions (note_id, entity_id, line, how) VALUES (?1, ?2, ?3, ?4)",
    )?;
    let mut by_line: HashMap<(String, u32), entities::How> = HashMap::new();
    // ID links name an entity by its ID.
    let entity_ids: HashSet<String> = conn
        .prepare_cached("SELECT note_id FROM entities")?
        .query_map([], |row| row.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    for link in kanso_core::links::note_links(body) {
        if let kanso_core::links::LinkTarget::Note(target) = link.target
            && entity_ids.contains(&target)
        {
            by_line.insert((target, link.line), entities::How::Link);
        }
    }
    for (entity, mention) in scan_mentions(body, &names) {
        let slot = by_line.entry((entity, mention.line)).or_insert(mention.how);
        if mention.how < *slot {
            *slot = mention.how;
        }
    }
    for ((entity, line), how) in by_line {
        if entity != id {
            stmt.execute(params![id, entity, line, how.as_str()])?;
        }
    }
    Ok(())
}

/// Reads note `id` as the index sees it and writes its mentions again.
fn rescan_note(conn: &Connection, id: &str) -> Result<()> {
    let body: Option<String> = conn
        .query_row(
            concat!(
                "SELECT ",
                schema::readable_body!(""),
                " FROM docs WHERE id = ?1 AND type = 'note' AND deleted = 0
                   AND IFNULL(book_id, '') != 'trash'"
            ),
            [id],
            |row| row.get(0),
        )
        .ok()
        .flatten();
    match body {
        Some(body) if !body.is_empty() => write_mentions(conn, id, &body),
        _ => {
            conn.execute("DELETE FROM note_mentions WHERE note_id = ?", [id])?;
            Ok(())
        }
    }
}

/// The readable notes whose text holds any of `names`, found through the
/// trigram index where the name allows and by scanning where it doesn't.
fn notes_containing(conn: &Connection, names: &BTreeSet<String>) -> Result<BTreeSet<String>> {
    let mut found = BTreeSet::new();
    for name in names {
        if name.is_empty() {
            continue;
        }
        if name.is_ascii() && name.chars().count() >= 3 {
            let fts = format!("body : \"{}\"", name.replace('"', "\"\""));
            let mut stmt =
                conn.prepare_cached("SELECT note_id FROM notes_fts WHERE notes_fts MATCH ?1")?;
            let ids = stmt.query_map([fts], |row| row.get::<_, String>(0))?;
            for id in ids {
                found.insert(id?);
            }
        } else {
            let mut stmt = conn.prepare_cached(concat!(
                "SELECT id FROM docs WHERE type = 'note' AND deleted = 0
                   AND IFNULL(book_id, '') != 'trash' AND kanso_contains(",
                schema::readable_body!(""),
                ", ?1)"
            ))?;
            let ids = stmt.query_map([name], |row| row.get::<_, String>(0))?;
            for id in ids {
                found.insert(id?);
            }
        }
    }
    Ok(found)
}

/// Fills the tables for every note after the other derived tables were
/// rebuilt: first which notes are entities, then what mentions them.
/// `notes` is `(id, title, readable body or None, in the trash)`.
pub(crate) fn rebuild(
    conn: &Connection,
    notes: &[(String, String, Option<String>, bool)],
) -> Result<()> {
    for table in ["entities", "entity_names", "note_mentions"] {
        conn.execute(&format!("DELETE FROM {table}"), [])?;
    }
    for (id, title, body, trashed) in notes {
        let Some(body) = body.as_deref().filter(|b| !b.is_empty() && !trashed) else {
            continue;
        };
        let Some(kind) = entities::kind_of(body) else {
            continue;
        };
        conn.execute(
            "INSERT OR REPLACE INTO entities (note_id, kind, name) VALUES (?1, ?2, ?3)",
            params![id, kind.as_str(), entities::normalize_name(title)],
        )?;
        for name in entities::names_of(title, body) {
            conn.execute(
                "INSERT OR IGNORE INTO entity_names (name, note_id) VALUES (?1, ?2)",
                params![name, id],
            )?;
        }
    }
    let any: bool = conn.query_row("SELECT EXISTS (SELECT 1 FROM entities)", [], |row| {
        row.get(0)
    })?;
    if !any {
        return Ok(());
    }
    for (id, _, body, trashed) in notes {
        if let Some(body) = body.as_deref().filter(|b| !b.is_empty() && !trashed) {
            write_mentions(conn, id, body)?;
        }
    }
    Ok(())
}
