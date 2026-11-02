//! Inbox triage, the store's side (issue #195): which Inbox notes still
//! wait to be filed, the suggestions a person rejected, and the reads the
//! suggestion engine (`kanso_engine::triage`) needs that the other
//! discovery reads leave out: a note's nearest notes with links kept, the
//! titles of the notes that can be linked to, and the notes it already
//! links to.
//!
//! Rejections are local to this device, never synced, and emptied with the
//! library. Nothing here reads a locked note, a note in a locked notebook,
//! a trashed note or one tagged `#ai-exclude`.

use std::collections::HashSet;

use rusqlite::params;

use crate::embed::{embedded_notes, nearest, own_vector};
use crate::{RelatedNote, Result, Store};

/// The tables, for emptying them with the library.
pub(crate) const TABLES: [&str; 1] = ["triage_dismissed"];

/// The note field that says a note was filed (or looked at) in Triage:
/// `{"v": 1, "at": <ms>}`. It syncs with the note.
pub const INBOX_TRIAGED_FIELD: &str = "kansoTriaged";

/// An Inbox note waiting in Triage.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InboxNote {
    pub id: String,
    pub rev: String,
    pub updated_at: i64,
}

impl Store {
    /// The notes in notebook `inbox` that Triage still shows, newest
    /// first: live, not locked, and not marked triaged since they were
    /// last edited.
    pub fn inbox_waiting(&self, inbox: &str) -> Result<Vec<InboxNote>> {
        self.read(|conn| {
            let mut stmt = conn.prepare(concat!(
                "SELECT d.id, d.rev, d.updated_at FROM docs d
                 WHERE d.type = 'note' AND d.deleted = 0 AND d.book_id = ?1
                 AND NOT ",
                crate::schema::note_locked!("d."),
                " AND IFNULL(json_extract(d.json, '$.kansoTriaged.at'), -1) < d.updated_at
                 ORDER BY d.updated_at DESC, d.id"
            ))?;
            let rows = stmt.query_map(params![inbox], |row| {
                Ok(InboxNote {
                    id: row.get(0)?,
                    rev: row.get(1)?,
                    updated_at: row.get(2)?,
                })
            })?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }

    /// Remembers that the person rejected `value` for `field` of note
    /// `note_id`, so it is not suggested again.
    pub fn dismiss_triage(
        &self,
        note_id: &str,
        field: &str,
        value: &str,
        now_ms: i64,
    ) -> Result<()> {
        self.write(|tx, _| {
            tx.execute(
                "INSERT OR REPLACE INTO triage_dismissed (note_id, field, value, at)
                 VALUES (?, ?, ?, ?)",
                params![note_id, field, value, now_ms],
            )?;
            Ok(())
        })
    }

    /// Takes back a rejection, when the person accepts the chip after all.
    pub fn restore_triage(&self, note_id: &str, field: &str, value: &str) -> Result<()> {
        self.write(|tx, _| {
            tx.execute(
                "DELETE FROM triage_dismissed WHERE note_id = ? AND field = ? AND value = ?",
                params![note_id, field, value],
            )?;
            Ok(())
        })
    }

    /// What was rejected for note `note_id`, as `(field, value)`.
    pub fn triage_dismissed(&self, note_id: &str) -> Result<Vec<(String, String)>> {
        self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT field, value FROM triage_dismissed WHERE note_id = ?
                 ORDER BY at, field, value",
            )?;
            let rows = stmt.query_map([note_id], |row| Ok((row.get(0)?, row.get(1)?)))?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }

    /// The `k` notes nearest note `id` by `model`'s embeddings, leaving
    /// out only the note itself (linked notes stay: Triage wants to know
    /// them). `None` while the note has no embedding.
    pub fn nearest_notes(
        &self,
        id: &str,
        model: &str,
        k: usize,
    ) -> Result<Option<Vec<RelatedNote>>> {
        self.read(|conn| {
            let Some(query) = own_vector(conn, id, model)? else {
                return Ok(None);
            };
            let skip = HashSet::from([id.to_owned()]);
            let candidates = embedded_notes(conn, model, &skip)?;
            Ok(Some(nearest(&query, &candidates, k)))
        })
    }

    /// The notes that can be linked to or cited: live, outside the trash,
    /// not locked and not tagged `#ai-exclude`, as `(id, title)`, titled
    /// ones only.
    pub fn linkable_titles(&self) -> Result<Vec<(String, String)>> {
        self.read(|conn| {
            let mut stmt = conn.prepare(concat!(
                "SELECT d.id, d.title FROM docs d WHERE ",
                crate::schema::discoverable!("d."),
                " AND IFNULL(d.title, '') != '' ORDER BY d.id"
            ))?;
            let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }

    /// The notes note `id` links to already, by ID or by title.
    pub fn links_from(&self, id: &str) -> Result<HashSet<String>> {
        self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT target_id FROM note_links WHERE note_id = ?1 AND target_id IS NOT NULL
                 UNION SELECT d.id FROM note_links l JOIN docs d
                     ON d.type = 'note' AND d.deleted = 0
                     AND kanso_norm_title(d.title) = l.target_title
                     WHERE l.note_id = ?1 AND l.target_title IS NOT NULL",
            )?;
            let rows = stmt.query_map([id], |row| row.get::<_, String>(0))?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Origin, Put};
    use kanso_core::DocType;
    use serde_json::json;

    fn open() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("kanso.db")).unwrap();
        (dir, store)
    }

    fn book(store: &Store, id: &str) {
        store
            .put(
                DocType::Book,
                json!({"_id": id, "name": id}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
    }

    fn note(store: &Store, book: &str, title: &str, extra: serde_json::Value) -> String {
        let mut body = json!({"title": title, "body": "text", "bookId": book});
        body.as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        store
            .put(DocType::Note, body, Put::Upsert, Origin::Ui)
            .unwrap()
            .id
    }

    #[test]
    fn dismissals_are_kept_and_restored() {
        let (_d, store) = open();
        store.dismiss_triage("note:a", "tag", "t1", 5).unwrap();
        store.dismiss_triage("note:a", "tag", "t1", 6).unwrap();
        store.dismiss_triage("note:a", "notebook", "b", 7).unwrap();
        assert_eq!(store.triage_dismissed("note:a").unwrap().len(), 2);
        assert!(store.triage_dismissed("note:b").unwrap().is_empty());
        store.restore_triage("note:a", "tag", "t1").unwrap();
        assert_eq!(
            store.triage_dismissed("note:a").unwrap(),
            vec![("notebook".to_owned(), "b".to_owned())]
        );
    }

    #[test]
    fn the_inbox_waiting_leaves_out_what_was_triaged_since_the_last_edit() {
        let (_d, store) = open();
        book(&store, "book:inbox");
        book(&store, "book:other");
        let waiting = note(&store, "book:inbox", "One", json!({}));
        let done = note(
            &store,
            "book:inbox",
            "Two",
            json!({"updatedAt": 100, INBOX_TRIAGED_FIELD: {"v": 1, "at": 100}}),
        );
        let edited_since = note(
            &store,
            "book:inbox",
            "Three",
            json!({"updatedAt": 200, INBOX_TRIAGED_FIELD: {"v": 1, "at": 100}}),
        );
        note(&store, "book:other", "Elsewhere", json!({}));
        let ids: Vec<String> = store
            .inbox_waiting("book:inbox")
            .unwrap()
            .into_iter()
            .map(|n| n.id)
            .collect();
        assert!(ids.contains(&waiting) && ids.contains(&edited_since));
        assert!(!ids.contains(&done));
        assert_eq!(ids.len(), 2);
    }
}
