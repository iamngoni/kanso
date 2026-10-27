//! Meaning, from the embeddings related notes keep (`embed.rs`): searching
//! notes by what they are about, tags suggested by the nearest notes, a
//! note that says the same as another, and merging the two.
//!
//! The vectors are read from the store, so tag suggestions and the
//! duplicate check work without a model. A search needs its words turned
//! into a vector by the same model the notes were embedded with: the app
//! lends the store its model while it is loaded ([`Store::lend_embedder`]),
//! and a store opened without the app answers [`MeaningSearch::NoModel`].

pub mod model;
pub mod rank;

use std::collections::HashSet;
use std::sync::Arc;

use kanso_core::Note;
use kanso_core::excerpt::{HEAD_CHARS, excerpt_around};

use kanso_core::model::{LOCK_FIELD, TRASH_BOOK_ID};
use rusqlite::{Connection, params};
use serde_json::Value;

use crate::embed::{embedded_notes, nearest, own_vector};
use crate::write::{keep_snapshot, load_row, rewrite_note};
use crate::{Embedder, NoteSummary, Origin, RelatedNote, Result, Store, StoreError, WriteResult};
use model::{MeaningSearch, Merged, TagSuggestion};
use rank::{DUPLICATE_SIMILARITY, merged_body, merged_tags, suggest_tags};

/// How many of a note's nearest notes its tag suggestions come from.
const TAG_NEIGHBOURS: usize = 12;

impl Store {
    /// Lends the store the app's embedding model while it is loaded
    /// (`None` once it isn't), so searches by meaning can turn their words
    /// into a vector, in the app and through its local API.
    pub fn lend_embedder(&self, embedder: Option<Arc<dyn Embedder>>) {
        *self.embedder.write().expect("embedder poisoned") = embedder;
    }

    /// The lent model, when it is the one the notes are embedded with.
    fn query_embedder(&self, model: &str) -> Option<Arc<dyn Embedder>> {
        self.embedder
            .read()
            .expect("embedder poisoned")
            .clone()
            .filter(|embedder| embedder.model() == model)
    }

    /// The `k` live notes outside the trash nearest in meaning to `query`.
    pub fn search_by_meaning(&self, query: &str, k: usize) -> Result<MeaningSearch> {
        let Some(model) = self.embedding_model()? else {
            return Ok(MeaningSearch::Off);
        };
        let Some(embedder) = self.query_embedder(&model) else {
            return Ok(MeaningSearch::NoModel);
        };
        let words = query.trim();
        if words.is_empty() {
            return Ok(MeaningSearch::Found(Vec::new()));
        }
        let Some(vector) = embedder.embed(&[words.to_owned()]).pop() else {
            return Ok(MeaningSearch::Found(Vec::new()));
        };
        self.read(|conn| {
            let candidates = embedded_notes(conn, &model, &HashSet::new())?;
            Ok(MeaningSearch::Found(nearest(&vector, &candidates, k)))
        })
    }

    /// Tags for note `id` from its nearest notes by `model` (linked ones
    /// too), that it hasn't got: at most `limit`, heaviest first. `None`
    /// while the note has no embedding.
    pub fn tag_suggestions(
        &self,
        id: &str,
        model: &str,
        limit: usize,
    ) -> Result<Option<Vec<TagSuggestion>>> {
        self.read(|conn| {
            let Some(query) = own_vector(conn, id, model)? else {
                return Ok(None);
            };
            let skip = HashSet::from([id.to_owned()]);
            let candidates = embedded_notes(conn, model, &skip)?;
            let near = nearest(&query, &candidates, TAG_NEIGHBOURS);
            let neighbours: Vec<(f32, Vec<String>)> = near
                .iter()
                .map(|note| Ok((note.score, note_tags(conn, &note.id)?)))
                .collect::<Result<_>>()?;
            let own = note_tags(conn, id)?;
            suggest_tags(&neighbours, &own, limit)
                .into_iter()
                .filter_map(|(tag_id, weight)| match tag_name(conn, &tag_id) {
                    Ok(Some(name)) => Some(Ok(TagSuggestion {
                        tag_id,
                        name,
                        weight,
                    })),
                    Ok(None) => None,
                    Err(err) => Some(Err(err)),
                })
                .collect::<Result<Vec<_>>>()
                .map(Some)
        })
    }

    /// The note that says nearly the same as note `id` by `model`, at least
    /// [`DUPLICATE_SIMILARITY`] similar, if there is one. Locked notes and
    /// notes tagged `#ai-exclude` are never embedded, so they are left out
    /// on both sides.
    pub fn near_duplicate(&self, id: &str, model: &str) -> Result<Option<RelatedNote>> {
        self.read(|conn| {
            if is_locked(conn, id)? {
                return Ok(None);
            }
            let Some(query) = own_vector(conn, id, model)? else {
                return Ok(None);
            };
            let skip = HashSet::from([id.to_owned()]);
            let candidates = embedded_notes(conn, model, &skip)?;
            Ok(nearest(&query, &candidates, 1)
                .into_iter()
                .find(|note| note.score >= DUPLICATE_SIMILARITY))
        })
    }

    /// Merges note `other` into note `keep`: `keep` gets `other`'s words
    /// after its own ([`merged_body`]) and its tags, and `other` goes to the
    /// trash, both in one transaction. Both versions as they were go into
    /// their notes' history first. Refused with [`StoreError::Conflict`]
    /// when either revision isn't the current one, and for locked notes.
    pub fn merge_notes(
        &self,
        keep: &str,
        keep_rev: &str,
        other: &str,
        other_rev: &str,
        origin: Origin,
    ) -> Result<Merged> {
        if keep == other {
            return Err(StoreError::Invalid(
                "A note can't be merged into itself.".into(),
            ));
        }
        self.write(|tx, changes| {
            let load = |id: &str, rev: &str| -> Result<crate::write::Row> {
                let row = load_row(tx, id)?
                    .filter(|row| row.doc_type == kanso_core::DocType::Note && !row.deleted)
                    .ok_or(StoreError::NotFound)?;
                if row.rev != rev {
                    return Err(StoreError::Conflict);
                }
                if row.fields.contains_key(LOCK_FIELD) {
                    return Err(StoreError::Invalid("A locked note can't be merged.".into()));
                }
                Ok(row)
            };
            let kept_row = load(keep, keep_rev)?;
            let other_row = load(other, other_rev)?;
            // However recent the last saved version, these two are kept:
            // the write path saves them itself for every other origin, and
            // for the app at most every few minutes.
            if origin == Origin::Ui {
                keep_snapshot(tx, keep, &kept_row, origin)?;
                keep_snapshot(tx, other, &other_row, origin)?;
            }
            let text = |row: &crate::write::Row, field: &str| {
                row.fields
                    .get(field)
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_owned()
            };
            let tags = |row: &crate::write::Row| -> Vec<String> {
                row.fields
                    .get("tags")
                    .and_then(Value::as_array)
                    .map(|tags| {
                        tags.iter()
                            .filter_map(|t| t.as_str().map(str::to_owned))
                            .collect()
                    })
                    .unwrap_or_default()
            };
            let body = merged_body(
                &text(&kept_row, "body"),
                &text(&other_row, "title"),
                &text(&other_row, "body"),
            );
            let all_tags = merged_tags(&tags(&kept_row), &tags(&other_row));
            rewrite_note(tx, keep, origin, changes, |fields| {
                fields.insert("body".into(), body.into());
                fields.insert("tags".into(), all_tags.into());
            })?;
            rewrite_note(tx, other, origin, changes, |fields| {
                fields.insert("bookId".into(), TRASH_BOOK_ID.into());
            })?;
            let rev = |id: &str| -> Result<WriteResult> {
                let row = load_row(tx, id)?.ok_or(StoreError::NotFound)?;
                Ok(WriteResult {
                    id: id.to_owned(),
                    rev: row.rev,
                })
            };
            Ok(Merged {
                kept: rev(keep)?,
                trashed: rev(other)?,
            })
        })
    }

    /// The live notes with `ids` as the note list shows them, in that
    /// order; any gone meanwhile are left out.
    pub fn summaries_of(&self, ids: &[&str]) -> Result<Vec<NoteSummary>> {
        let mut out = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some(note) = self.note(id)? {
                let locked = note.is_locked() || self.note_is_locked(id)?;
                out.push(summary(&note, locked));
            }
        }
        Ok(out)
    }
}

/// The tag IDs note `id` has.
fn note_tags(conn: &Connection, id: &str) -> Result<Vec<String>> {
    let mut stmt = conn.prepare_cached("SELECT tag_id FROM note_tags WHERE note_id = ?")?;
    let rows = stmt.query_map([id], |row| row.get::<_, String>(0))?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

/// A live tag's name.
fn tag_name(conn: &Connection, id: &str) -> Result<Option<String>> {
    use rusqlite::OptionalExtension as _;
    Ok(conn
        .query_row(
            "SELECT json_extract(json, '$.name') FROM docs
             WHERE id = ? AND type = 'tag' AND deleted = 0",
            params![id],
            |row| row.get::<_, Option<String>>(0),
        )
        .optional()?
        .flatten())
}

fn is_locked(conn: &Connection, id: &str) -> Result<bool> {
    use rusqlite::OptionalExtension as _;
    Ok(conn
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

/// A note as a list row, its snippet from the start of its body unless
/// it is `locked` (sealed, or in a locked notebook).
fn summary(note: &Note, locked: bool) -> NoteSummary {
    let head: String = if locked {
        String::new()
    } else {
        note.body.chars().take(HEAD_CHARS).collect()
    };
    NoteSummary {
        id: note.id.clone(),
        rev: note.rev.clone().unwrap_or_default(),
        title: note.title.clone(),
        snippet: excerpt_around(&note.title, &head, None, &[] as &[&str]),
        book_id: note.book_id.clone(),
        status: note.status,
        tags: note.tags.clone(),
        pinned: note.pinned,
        updated_at: note.updated_at,
        created_at: note.created_at,
        tasks_done: note.num_of_checked_tasks,
        tasks_total: note.num_of_tasks,
        locked,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Put, embed_text};
    use kanso_core::DocType;
    use proptest::prelude::*;
    use serde_json::json;

    /// Counts letters: texts with the same letters point the same way.
    struct Letters;

    impl Embedder for Letters {
        fn model(&self) -> &str {
            "letters"
        }
        fn dim(&self) -> usize {
            26
        }
        fn embed(&self, texts: &[String]) -> Vec<Vec<f32>> {
            texts
                .iter()
                .map(|t| {
                    let mut v = vec![0f32; 26];
                    for c in t.to_ascii_lowercase().bytes() {
                        if c.is_ascii_lowercase() {
                            v[(c - b'a') as usize] += 1.;
                        }
                    }
                    v
                })
                .collect()
        }
    }

    fn open() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("kanso.db")).unwrap();
        store
            .put(
                DocType::Book,
                json!({"_id": "book:b", "name": "Notes"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
        (dir, store)
    }

    fn note(store: &Store, title: &str, body: &str, tags: &[&str]) -> String {
        let tags: Vec<String> = tags
            .iter()
            .map(|t| store.tag_named(t, Origin::Ui).unwrap())
            .collect();
        store
            .put(
                DocType::Note,
                json!({"title": title, "body": body, "bookId": "book:b", "tags": tags}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id
    }

    /// Embeds whatever is waiting, as the app's indexer does.
    fn index(store: &Store) {
        let queue = store.embedding_queue("letters").unwrap();
        let texts: Vec<String> = queue.pending.iter().map(|p| p.text.clone()).collect();
        let rows: Vec<_> = queue
            .pending
            .into_iter()
            .zip(Letters.embed(&texts))
            .map(|(p, v)| (p.id, p.hash, v))
            .collect();
        store.put_embeddings("letters", &rows).unwrap();
    }

    #[test]
    fn a_search_by_meaning_needs_the_feature_and_the_model() {
        let (_dir, store) = open();
        let a = note(&store, "zzz", "zzzz yyy", &[]);
        let b = note(&store, "abc", "aabbcc", &[]);
        assert_eq!(
            store.search_by_meaning("zz", 5).unwrap(),
            MeaningSearch::Off
        );
        store.set_embedding_model(Some("letters")).unwrap();
        index(&store);
        // On, but this process has no model.
        assert_eq!(
            store.search_by_meaning("zz", 5).unwrap(),
            MeaningSearch::NoModel
        );
        store.lend_embedder(Some(Arc::new(Letters)));
        let MeaningSearch::Found(found) = store.search_by_meaning("zzy", 5).unwrap() else {
            panic!("no search");
        };
        assert_eq!(found[0].id, a);
        assert_eq!(found[1].id, b);
        assert!(found[0].score > found[1].score);
        let rows = store.summaries_of(&[&b, "note:gone", &a]).unwrap();
        let titles: Vec<&str> = rows.iter().map(|r| r.title.as_str()).collect();
        assert_eq!(titles, ["abc", "zzz"]);
        assert_eq!(rows[0].snippet, "aabbcc");
        // A model lent for another embedding is not used.
        store.set_embedding_model(Some("other")).unwrap();
        assert_eq!(
            store.search_by_meaning("zz", 5).unwrap(),
            MeaningSearch::NoModel
        );
        store.lend_embedder(None);
    }

    #[test]
    fn tags_are_suggested_from_close_notes() {
        let (_dir, store) = open();
        let a = note(&store, "aaa", "aaaa bbb", &["mine"]);
        note(&store, "aab", "aaaa bb", &["mine", "letters", "ab"]);
        note(&store, "aaa", "aaa bbbb", &["letters"]);
        note(&store, "zzz", "zzzz", &["far"]);
        store.set_embedding_model(Some("letters")).unwrap();
        assert_eq!(store.tag_suggestions(&a, "letters", 5).unwrap(), None);
        index(&store);
        let names: Vec<String> = store
            .tag_suggestions(&a, "letters", 5)
            .unwrap()
            .unwrap()
            .into_iter()
            .map(|s| s.name)
            .collect();
        assert_eq!(names, ["letters", "ab"]);
    }

    fn rev(store: &Store, id: &str) -> String {
        store.note(id).unwrap().unwrap().rev.unwrap()
    }

    #[test]
    fn a_copy_is_a_near_duplicate_and_merging_keeps_everything() {
        let (_dir, store) = open();
        let a = note(&store, "Groceries", "milk eggs bread", &["home"]);
        let b = note(&store, "Groceries", "milk eggs bread jam", &["shop"]);
        let c = note(&store, "Taxes", "forms receipts", &[]);
        store.set_embedding_model(Some("letters")).unwrap();
        index(&store);
        let twin = store.near_duplicate(&a, "letters").unwrap().unwrap();
        assert_eq!(twin.id, b);
        assert!(twin.score >= DUPLICATE_SIMILARITY);
        assert_eq!(store.near_duplicate(&c, "letters").unwrap(), None);

        // A stale revision is refused, and nothing changes.
        let stale = store.merge_notes(&a, "1-x", &b, &rev(&store, &b), Origin::Ui);
        assert!(matches!(stale, Err(StoreError::Conflict)));
        let (a_rev, b_rev) = (rev(&store, &a), rev(&store, &b));
        let merged = store
            .merge_notes(&a, &a_rev, &b, &b_rev, Origin::Ui)
            .unwrap();
        let kept = store.note(&a).unwrap().unwrap();
        assert_eq!(kept.rev.as_deref(), Some(merged.kept.rev.as_str()));
        assert_eq!(
            kept.body,
            "milk eggs bread\n\n## Groceries\n\nmilk eggs bread jam\n"
        );
        assert_eq!(kept.tags.len(), 2);
        let trashed = store.note(&b).unwrap().unwrap();
        assert_eq!(trashed.book_id, TRASH_BOOK_ID);
        assert_eq!(trashed.body, "milk eggs bread jam");
        // Both versions from before are in history.
        assert!(store.revisions(&a).unwrap().iter().any(|r| r.rev == a_rev));
        assert!(store.revisions(&b).unwrap().iter().any(|r| r.rev == b_rev));
        // In the trash, it is no longer anyone's duplicate.
        index(&store);
        assert_eq!(store.near_duplicate(&a, "letters").unwrap(), None);
        assert!(matches!(
            store.merge_notes(&a, &merged.kept.rev, &a, &merged.kept.rev, Origin::Ui),
            Err(StoreError::Invalid(_))
        ));
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 24,
            failure_persistence: Some(Box::new(
                proptest::test_runner::FileFailurePersistence::WithSource("regressions"),
            )),
            ..ProptestConfig::default()
        })]

        /// Through the store, merging loses no text: the kept note holds
        /// both bodies' words and the other title, has every tag of both,
        /// and the other note is in the trash with its words as they were.
        #[test]
        fn prop_merging_through_the_store_loses_no_text(
            body in "[a-z \n]{0,40}",
            title in "[A-Za-z]{0,10}",
            other in "[a-z \n]{0,40}",
            tags in prop::collection::vec("[a-c]", 0..3),
            other_tags in prop::collection::vec("[b-e]", 0..3),
        ) {
            let (_dir, store) = open();
            let tags: Vec<&str> = tags.iter().map(String::as_str).collect();
            let other_tags: Vec<&str> = other_tags.iter().map(String::as_str).collect();
            let a = note(&store, "Kept", &body, &tags);
            let b = note(&store, &title, &other, &other_tags);
            let before = (store.note(&a).unwrap().unwrap(), store.note(&b).unwrap().unwrap());
            store.merge_notes(&a, &rev(&store, &a), &b, &rev(&store, &b), Origin::Ui).unwrap();
            let kept = store.note(&a).unwrap().unwrap();
            for word in before.0.body.split_whitespace().chain(before.1.body.split_whitespace()) {
                prop_assert!(kept.body.contains(word));
            }
            prop_assert!(embed_text("", &kept.body).contains(title.trim()) || title.trim().is_empty());
            for tag in before.0.tags.iter().chain(&before.1.tags) {
                prop_assert!(kept.tags.contains(tag));
            }
            let trashed = store.note(&b).unwrap().unwrap();
            prop_assert_eq!(trashed.book_id.as_str(), TRASH_BOOK_ID);
            prop_assert_eq!(trashed.body, before.1.body);
        }
    }
}
