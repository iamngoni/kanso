//! Reading every live note for a query view: its facts and its text.
//!
//! A locked note's text is never read here: `body` is empty for one sealed
//! itself or in a locked notebook. The trash is read too, so the pure
//! evaluation can leave it out with everything else it doesn't list.

use std::collections::HashMap;

use kanso_core::NoteStatus;
use kanso_core::query_view::ViewNote;

use crate::{Result, Store};

/// A note as a query view reads it.
#[derive(Clone, Debug, PartialEq)]
pub struct QueryNoteRow {
    pub id: String,
    pub rev: String,
    pub title: String,
    /// Empty for a locked note.
    pub body: String,
    pub book_id: String,
    pub book_name: Option<String>,
    pub status: String,
    /// The note's live tags.
    pub tag_ids: Vec<String>,
    pub tag_names: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub locked: bool,
    pub trashed: bool,
    /// The note's properties, read from the text that was selected.
    pub properties: Vec<(String, kanso_core::properties::Value)>,
}

impl From<QueryNoteRow> for ViewNote {
    /// A row as a query reads it. A locked note's properties are read from
    /// its empty text, so it has none.
    fn from(row: QueryNoteRow) -> Self {
        ViewNote {
            properties: row.properties,
            id: row.id,
            title: row.title,
            body: row.body,
            book_id: row.book_id,
            book_name: row.book_name,
            status: row.status,
            tag_ids: row.tag_ids,
            tag_names: row.tag_names,
            created_at: row.created_at,
            updated_at: row.updated_at,
            locked: row.locked,
            trashed: row.trashed,
        }
    }
}

impl Store {
    /// Every live note, oldest edit first, with the facts a query needs.
    pub fn query_view_notes(&self) -> Result<Vec<QueryNoteRow>> {
        self.query_view_notes_with_body(true)
    }

    /// Every live note with the facts a query needs. When `read_body` is
    /// false only the front matter is read: enough for properties, but not
    /// for a free-text search.
    pub fn query_view_notes_with_body(&self, read_body: bool) -> Result<Vec<QueryNoteRow>> {
        let tags: HashMap<String, String> = self
            .tags()?
            .into_iter()
            .map(|tag| (tag.id, tag.name))
            .collect();
        self.read(|conn| {
            let body = crate::schema::readable_body!("d.");
            let text = if read_body {
                body.to_owned()
            } else {
                format!("kanso_front_matter({body})")
            };
            let locked = crate::schema::note_locked!("d.");
            let mut stmt = conn.prepare(&format!(
                "SELECT d.id, d.rev, d.title, {text}, d.book_id, b.name, d.status, \
                 json_extract(d.json, '$.tags'), d.created_at, d.updated_at, {locked} \
                 FROM docs d LEFT JOIN docs b ON b.id = d.book_id AND b.type = 'book' AND b.deleted = 0 \
                 WHERE d.type = 'note' AND d.deleted = 0 \
                 ORDER BY d.updated_at, d.id"
            ))?;
            let rows = stmt.query_map([], |row| {
                let book_id: String = row.get::<_, Option<String>>(4)?.unwrap_or_default();
                let raw_tags: Option<String> = row.get(7)?;
                let tag_ids: Vec<String> = raw_tags
                    .and_then(|json| serde_json::from_str::<Vec<String>>(&json).ok())
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|id| tags.contains_key(id))
                    .collect();
                let tag_names = tag_ids
                    .iter()
                    .filter_map(|id| tags.get(id).cloned())
                    .collect();
                let status: String = row.get::<_, Option<String>>(6)?.unwrap_or_default();
                let text: String = row.get::<_, Option<String>>(3)?.unwrap_or_default();
                let (body, properties) = if read_body {
                    (
                        text.clone(),
                        kanso_core::properties::parse(&text),
                    )
                } else {
                    (
                        String::new(),
                        kanso_core::properties::parse_front(&text),
                    )
                };
                Ok(QueryNoteRow {
                    id: row.get(0)?,
                    rev: row.get(1)?,
                    title: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                    body,
                    trashed: book_id == kanso_core::TRASH_BOOK_ID,
                    book_id,
                    book_name: row.get::<_, Option<String>>(5)?,
                    status: NoteStatus::parse(&status)
                        .unwrap_or_default()
                        .as_str()
                        .to_owned(),
                    tag_ids,
                    tag_names,
                    created_at: row.get::<_, Option<i64>>(8)?.unwrap_or(0),
                    updated_at: row.get::<_, Option<i64>>(9)?.unwrap_or(0),
                    locked: row.get(10)?,
                    properties,
                })
            })?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kanso_core::DocType;
    use serde_json::json;

    fn store() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("kanso.db")).unwrap();
        (dir, store)
    }

    fn book(store: &Store, name: &str) -> String {
        store
            .put(
                DocType::Book,
                json!({"name": name}),
                crate::Put::Upsert,
                crate::Origin::Ui,
            )
            .unwrap()
            .id
    }

    fn note(store: &Store, title: &str, body: &str, book: &str) -> String {
        store
            .put(
                DocType::Note,
                json!({"title": title, "body": body, "bookId": book}),
                crate::Put::Upsert,
                crate::Origin::Ui,
            )
            .unwrap()
            .id
    }

    #[test]
    fn reads_live_notes_and_leaves_the_trash_on_its_own() {
        let (_dir, store) = store();
        let inbox = book(&store, "Inbox");
        let live = note(&store, "Live", "a: 1\n", &inbox);
        let gone = book(&store, "Gone");
        let trashed = note(&store, "Trashed", "b: 2\n", &gone);
        store.delete(&gone, crate::Origin::Ui).unwrap();
        let rows = store.query_view_notes().unwrap();
        let by_id: HashMap<&str, &QueryNoteRow> =
            rows.iter().map(|row| (row.id.as_str(), row)).collect();
        assert_eq!(by_id[&live.as_str()].title, "Live");
        assert!(!by_id[&live.as_str()].trashed);
        assert!(by_id[&trashed.as_str()].trashed);
    }

    #[test]
    fn without_the_body_only_the_front_matter_is_read() {
        let (_dir, store) = store();
        let inbox = book(&store, "Clients");
        let id = note(
            &store,
            "Ana",
            "---\nclient: Acme\nrating: 4\n---\n# Ana\n\nSecret body text.\n",
            &inbox,
        );
        let with = store.query_view_notes_with_body(true).unwrap();
        let without = store.query_view_notes_with_body(false).unwrap();
        let full = with.iter().find(|row| row.id == id).unwrap();
        let front = without.iter().find(|row| row.id == id).unwrap();
        assert_eq!(
            full.body,
            "---\nclient: Acme\nrating: 4\n---\n# Ana\n\nSecret body text.\n"
        );
        assert_eq!(front.body, "");
        assert_eq!(front.properties, full.properties);
        assert!(!front.body.contains("Secret"));
    }

    #[test]
    fn a_locked_note_never_has_its_text_read() {
        let (_dir, store) = store();
        let inbox = book(&store, "Inbox");
        let sealed = store
            .put(
                DocType::Note,
                json!({"title": "Sealed", "body": "", "bookId": inbox,
                       "kansoLock": {"salt": "s", "nonce": "n", "data": "d"}}),
                crate::Put::Upsert,
                crate::Origin::Ui,
            )
            .unwrap()
            .id;
        let rows = store.query_view_notes().unwrap();
        let by_id: HashMap<&str, &QueryNoteRow> =
            rows.iter().map(|row| (row.id.as_str(), row)).collect();
        assert!(by_id[&sealed.as_str()].locked);
        assert_eq!(by_id[&sealed.as_str()].body, "");
    }

    #[test]
    fn notes_in_locked_notebooks_never_appear_in_query_views() {
        use kanso_core::query_view::{self, ViewNote};

        let (_dir, store) = store();
        let inbox = book(&store, "Inbox");
        let outside = note(&store, "Open", "---\nclient: Acme\n---\n# Open\n", &inbox);
        store
            .put(
                DocType::Note,
                json!({"_id": outside, "status": "active"}),
                crate::Put::Merge,
                crate::Origin::Ui,
            )
            .unwrap();

        let outer = book(&store, "Private");
        let inner = store
            .put(
                DocType::Book,
                json!({"name": "Deeper", "parentBookId": outer}),
                crate::Put::Upsert,
                crate::Origin::Ui,
            )
            .unwrap()
            .id;
        let mut hidden = Vec::new();
        for (book_id, title) in [(&outer, "Outer"), (&inner, "Nested")] {
            let id = note(
                &store,
                title,
                &format!("---\nclient: {title}\n---\n# {title}\n"),
                book_id,
            );
            store
                .put(
                    DocType::Note,
                    json!({"_id": id, "status": "active"}),
                    crate::Put::Merge,
                    crate::Origin::Ui,
                )
                .unwrap();
            hidden.push(id);
        }

        let seal = |id: &str| {
            let mut note = store.get_api(id, None, false).unwrap();
            note["body"] = "".into();
            note["kansoLock"] =
                json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "sealed"});
            note
        };
        let mut locked_book = store.get_api(&outer, None, false).unwrap();
        locked_book.as_object_mut().unwrap().remove("count");
        locked_book["kansoLock"] =
            json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "sealed"});
        store
            .lock_book(
                locked_book,
                hidden.iter().map(|id| seal(id)).collect::<Vec<_>>(),
            )
            .unwrap();

        let rows = store.query_view_notes().unwrap();
        for id in &hidden {
            let row = rows.iter().find(|row| &row.id == id).expect(id);
            assert!(row.locked, "{id} reads as locked");
            assert!(row.body.is_empty(), "{id} gives no text");
            assert!(row.properties.is_empty(), "{id} gives no properties");
        }
        let notes: Vec<ViewNote> = rows.into_iter().map(ViewNote::from).collect();
        let spec = query_view::parse("from: status:active").unwrap();
        let found = query_view::evaluate(&spec, &notes, None);
        let ids: Vec<&str> = found.iter().map(|row| row.id.as_str()).collect();
        assert_eq!(ids, [outside.as_str()]);
    }

    #[test]
    fn five_thousand_notes_answer_quickly() {
        use kanso_core::query_view::{self, ViewNote};

        let (_dir, store) = store();
        let inbox = book(&store, "Clients");
        for ix in 0..5_000 {
            store
                .put(
                    DocType::Note,
                    json!({
                        "title": format!("Note {ix}"),
                        "body": format!("---\nrating: {}\n---\n# Note\n", ix % 50),
                        "bookId": inbox,
                        "status": "active",
                    }),
                    crate::Put::Upsert,
                    crate::Origin::Ui,
                )
                .unwrap();
        }
        let answer = || {
            let started = std::time::Instant::now();
            let notes: Vec<ViewNote> = store
                .query_view_notes()
                .unwrap()
                .into_iter()
                .map(ViewNote::from)
                .collect();
            let spec = query_view::parse(
                "from: status:active\ncolumns: rating, book\nsort: rating desc\nlimit: 50",
            )
            .unwrap();
            let rows = query_view::evaluate(&spec, &notes, None);
            assert_eq!(rows.len(), 50);
            started.elapsed()
        };
        // The fastest of a few runs: the work is the same each time, so the
        // fastest is what it costs, and a run the scheduler put aside on a
        // busy machine (load averages of 400 stalled single runs past the
        // limit) doesn't count. A query that really got slower is slow in
        // every run and still fails. A debug build takes about 50 ms; a
        // release build is well under that.
        let elapsed = (0..5).map(|_| answer()).min().unwrap();
        assert!(
            elapsed < std::time::Duration::from_millis(500),
            "{elapsed:?}"
        );
        eprintln!("5000 notes answered in {elapsed:?} at best");
    }

    #[test]
    fn evaluation_leaves_out_locked_trashed_and_self() {
        use kanso_core::query_view::{self, ViewNote};

        let (_dir, store) = store();
        let inbox = book(&store, "Clients");
        let open = note(
            &store,
            "Open",
            "---\nclient: Acme\nrating: 4\n---\n# Open\n",
            &inbox,
        );
        let sealed = store
            .put(
                DocType::Note,
                json!({"title": "Sealed", "body": "", "bookId": inbox,
                       "kansoLock": {"salt": "s", "nonce": "n", "data": "d"}}),
                crate::Put::Upsert,
                crate::Origin::Ui,
            )
            .unwrap()
            .id;
        let trash = book(&store, "Gone");
        let trashed = note(
            &store,
            "Trashed",
            "---\nclient: Acme\n---\n# Trashed\n",
            &trash,
        );
        store.delete(&trash, crate::Origin::Ui).unwrap();
        let self_note = note(&store, "Self", "---\nclient: Acme\n---\n# Self\n", &inbox);

        let notes: Vec<ViewNote> = store
            .query_view_notes()
            .unwrap()
            .into_iter()
            .map(ViewNote::from)
            .collect();
        let spec = query_view::parse("from: has:client\ncolumns: rating").unwrap();
        let rows = query_view::evaluate(&spec, &notes, Some(&self_note));
        let ids: Vec<&str> = rows.iter().map(|row| row.id.as_str()).collect();
        assert_eq!(ids, [open.as_str()]);
        assert_eq!(
            rows[0].cells,
            vec![Some(kanso_core::properties::Value::Number(4.))]
        );
        assert!(!ids.contains(&sealed.as_str()));
        assert!(!ids.contains(&trashed.as_str()));
    }
}
