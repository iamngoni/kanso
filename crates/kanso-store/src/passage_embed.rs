//! Passage vectors for long notes (Ask with receipts).
//!
//! A note's own vector is made from its first `TEXT_CHARS` characters, so
//! what a long note says further down can't be found by meaning. Each
//! passage (`kanso_core::passages`) of a note longer than that gets a vector
//! of its own, so a question can reach the last section of a long note.
//! Shorter notes use their note vector: the table stays at a tenth of what
//! embedding every passage would cost.
//!
//! The table is local like `note_embeddings`: never synced, emptied with the
//! library and when related notes are turned off. A locked note and a note
//! tagged `#ai-exclude` never have a row: the queue deletes any it finds,
//! writes skip them, and every read joins the `discoverable!` predicate.

use std::collections::{HashMap, HashSet};

use kanso_core::passages::passages;
use rusqlite::{OptionalExtension, params};

use crate::embed::{
    MODEL_KEY, TEXT_CHARS, content_hash, cosine, decode_vector, embed_text, encode_vector,
};
use crate::{Result, Store, schema};

/// The most passages one note gets a vector for.
pub const MAX_PASSAGE_VECTORS: usize = 400;

/// A passage waiting for its vector.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingPassage {
    pub note_id: String,
    /// The passage's first and last line, 0-based.
    pub start: u32,
    pub end: u32,
    /// The words that are embedded.
    pub text: String,
    /// The hash of `text`, kept with the vector.
    pub hash: String,
}

/// What [`Store::passage_queue`] found.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PassageQueue {
    /// Up to the batch limit of passages that have no vector, or one made
    /// from other words.
    pub pending: Vec<PendingPassage>,
    /// How many notes still have a passage waiting (more than `pending`
    /// holds, when the batch limit cut it).
    pub notes_waiting: usize,
}

/// A passage's vector, to be kept.
#[derive(Clone, Debug, PartialEq)]
pub struct EmbeddedLines {
    pub note_id: String,
    pub start: u32,
    pub end: u32,
    pub hash: String,
    pub vector: Vec<f32>,
}

/// A passage found by meaning.
#[derive(Clone, Debug, PartialEq)]
pub struct PassageHit {
    pub note_id: String,
    pub start: u32,
    pub end: u32,
    pub score: f32,
}

impl Store {
    /// The passages of long notes that need a vector by `model`, at most
    /// `limit` of them. Passages that no longer exist, and those of notes
    /// that are no longer long, locked or tagged `#ai-exclude`, lose their
    /// vectors here.
    pub fn passage_queue(&self, model: &str, limit: usize) -> Result<PassageQueue> {
        self.write(|tx, _| {
            // Whatever can't be embedded any more loses its vectors first.
            tx.execute(
                &format!(
                    "DELETE FROM passage_embeddings WHERE note_id IN (SELECT d.id FROM docs d
                        WHERE d.type = 'note' AND ({} OR {}))",
                    schema::note_locked!("d."),
                    schema::ai_excluded!("d."),
                ),
                [],
            )?;
            Ok(())
        })?;
        let (queue, stale) = self.read(|conn| {
            let mut stmt = conn.prepare(&format!(
                "SELECT d.id, d.title, {body} FROM docs d
                 WHERE {discoverable} AND length({body}) > ?1
                 ORDER BY d.updated_at DESC, d.id",
                body = schema::readable_body!("d."),
                discoverable = schema::discoverable!("d."),
            ))?;
            let mut kept = conn.prepare(
                "SELECT start_line, content_hash FROM passage_embeddings
                 WHERE note_id = ?1 AND model = ?2",
            )?;
            let mut rows = stmt.query([TEXT_CHARS as i64])?;
            let mut queue = PassageQueue::default();
            let mut stale: Vec<(String, u32)> = Vec::new();
            let mut long: HashSet<String> = HashSet::new();
            while let Some(row) = rows.next()? {
                let id: String = row.get(0)?;
                let title: Option<String> = row.get(1)?;
                let body: Option<String> = row.get(2)?;
                let body = body.unwrap_or_default();
                long.insert(id.clone());
                let have: HashMap<u32, String> = kept
                    .query_map(params![id, model], |r| {
                        Ok((r.get::<_, i64>(0)? as u32, r.get::<_, String>(1)?))
                    })?
                    .collect::<rusqlite::Result<_>>()?;
                let mut present = HashSet::new();
                let mut waiting = false;
                for passage in passages(&body).into_iter().take(MAX_PASSAGE_VECTORS) {
                    present.insert(passage.start);
                    let heading = passage.heading.clone().unwrap_or_default();
                    let text = embed_text(
                        &format!("{} {}", title.as_deref().unwrap_or(""), heading),
                        &passage.text,
                    );
                    let hash = content_hash(&text);
                    if have.get(&passage.start) == Some(&hash) {
                        continue;
                    }
                    waiting = true;
                    if queue.pending.len() < limit {
                        queue.pending.push(PendingPassage {
                            note_id: id.clone(),
                            start: passage.start,
                            end: passage.end,
                            text,
                            hash,
                        });
                    }
                }
                if waiting {
                    queue.notes_waiting += 1;
                }
                for start in have.keys() {
                    if !present.contains(start) {
                        stale.push((id.clone(), *start));
                    }
                }
            }
            // Notes that used to be long and are not any more.
            let mut owners = conn.prepare("SELECT DISTINCT note_id FROM passage_embeddings")?;
            let owners: Vec<String> = owners
                .query_map([], |r| r.get(0))?
                .collect::<rusqlite::Result<_>>()?;
            for owner in owners {
                if !long.contains(&owner) {
                    stale.push((owner, u32::MAX));
                }
            }
            Ok((queue, stale))
        })?;
        if !stale.is_empty() {
            self.write(|tx, _| {
                for (id, start) in &stale {
                    if *start == u32::MAX {
                        tx.execute("DELETE FROM passage_embeddings WHERE note_id = ?", [id])?;
                    } else {
                        tx.execute(
                            "DELETE FROM passage_embeddings WHERE note_id = ?1 AND start_line = ?2",
                            params![id, *start as i64],
                        )?;
                    }
                }
                Ok(())
            })?;
        }
        Ok(queue)
    }

    /// Keeps passage vectors made by `model`. Notes locked, tagged
    /// `#ai-exclude` or deleted meanwhile are skipped, and nothing is kept
    /// once related notes are off or on another model.
    pub fn put_passage_embeddings(&self, model: &str, rows: &[EmbeddedLines]) -> Result<()> {
        self.write(|tx, _| {
            if schema::get_meta(tx, MODEL_KEY)?.as_deref() != Some(model) {
                return Ok(());
            }
            let mut add = tx.prepare_cached(concat!(
                "INSERT OR REPLACE INTO passage_embeddings
                     (note_id, start_line, end_line, model, dim, vector, content_hash)
                 SELECT ?1, ?2, ?3, ?4, ?5, ?6, ?7 WHERE EXISTS
                     (SELECT 1 FROM docs d WHERE d.id = ?1 AND ",
                crate::schema::discoverable!("d."),
                ")"
            ))?;
            for row in rows {
                add.execute(params![
                    row.note_id,
                    row.start as i64,
                    row.end as i64,
                    model,
                    row.vector.len() as i64,
                    encode_vector(&row.vector),
                    row.hash,
                ])?;
            }
            Ok(())
        })
    }

    /// How many passage vectors `model` has, for tests and the status line.
    pub fn passage_vector_count(&self, model: &str) -> Result<usize> {
        self.read(|conn| {
            Ok(conn.query_row(
                concat!(
                    "SELECT COUNT(*) FROM passage_embeddings p JOIN docs d ON d.id = p.note_id
                     WHERE p.model = ? AND ",
                    crate::schema::discoverable!("d.")
                ),
                [model],
                |row| row.get::<_, i64>(0),
            )? as usize)
        })
    }

    /// The `k` passages of long notes nearest in meaning to `query`, best
    /// first. Empty while related notes are off or the model isn't lent.
    pub fn search_passages_by_meaning(&self, query: &str, k: usize) -> Result<Vec<PassageHit>> {
        let Some(model) = self.embedding_model()? else {
            return Ok(Vec::new());
        };
        let Some(embedder) = self
            .embedder
            .read()
            .expect("embedder poisoned")
            .clone()
            .filter(|embedder| embedder.model() == model)
        else {
            return Ok(Vec::new());
        };
        let words = query.trim();
        if words.is_empty() || k == 0 {
            return Ok(Vec::new());
        }
        let Some(vector) = embedder.embed(&[words.to_owned()]).pop() else {
            return Ok(Vec::new());
        };
        self.read(|conn| {
            let mut stmt = conn.prepare(concat!(
                "SELECT p.note_id, p.start_line, p.end_line, p.dim, p.vector
                 FROM passage_embeddings p JOIN docs d ON d.id = p.note_id
                 WHERE p.model = ?1 AND ",
                crate::schema::discoverable!("d.")
            ))?;
            let mut rows = stmt.query([&model])?;
            let mut best: Vec<PassageHit> = Vec::new();
            while let Some(row) = rows.next()? {
                let dim: i64 = row.get(3)?;
                let bytes: Vec<u8> = row.get(4)?;
                let Some(stored) = decode_vector(&bytes, dim as usize) else {
                    continue;
                };
                let score = cosine(&vector, &stored);
                if best.len() == k && best.last().is_some_and(|last| last.score >= score) {
                    continue;
                }
                best.push(PassageHit {
                    note_id: row.get(0)?,
                    start: row.get::<_, i64>(1)? as u32,
                    end: row.get::<_, i64>(2)? as u32,
                    score,
                });
                best.sort_by(|a, b| {
                    b.score
                        .total_cmp(&a.score)
                        .then_with(|| a.note_id.cmp(&b.note_id))
                        .then_with(|| a.start.cmp(&b.start))
                });
                best.truncate(k);
            }
            Ok(best)
        })
    }

    /// Whether any passage vector is kept for note `id`, for tests.
    pub fn has_passage_vectors(&self, id: &str) -> Result<bool> {
        self.read(|conn| {
            Ok(conn
                .query_row(
                    "SELECT 1 FROM passage_embeddings WHERE note_id = ? LIMIT 1",
                    [id],
                    |_| Ok(()),
                )
                .optional()?
                .is_some())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Embedder, Origin, Put};
    use kanso_core::DocType;
    use proptest::prelude::*;
    use serde_json::json;
    use std::sync::Arc;

    /// Counts letters: passages with the same letters point the same way.
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
        store.set_embedding_model(Some("letters")).unwrap();
        store.lend_embedder(Some(Arc::new(Letters)));
        (dir, store)
    }

    fn book(store: &Store) -> String {
        if let Some(b) = store.books().unwrap().first() {
            return b.id.clone();
        }
        store
            .put(
                DocType::Book,
                json!({"name": "Notes"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id
    }

    fn note(store: &Store, title: &str, body: &str, tags: Vec<String>) -> String {
        store
            .put(
                DocType::Note,
                json!({"title": title, "body": body, "bookId": book(store), "tags": tags}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id
    }

    /// A body of `sections` long sections, the last one about `last`.
    fn long_body(sections: usize, last: &str) -> String {
        let mut body = String::new();
        for ix in 0..sections {
            body.push_str(&format!(
                "# Part {ix}\n{}\n\n",
                "filler words here ".repeat(80)
            ));
        }
        body.push_str(&format!("# Last\n{last}\n"));
        body
    }

    /// Embeds every waiting passage, as the indexer does.
    fn index(store: &Store) -> usize {
        let queue = store.passage_queue("letters", 1000).unwrap();
        let texts: Vec<String> = queue.pending.iter().map(|p| p.text.clone()).collect();
        let vectors = Letters.embed(&texts);
        let rows: Vec<EmbeddedLines> = queue
            .pending
            .into_iter()
            .zip(vectors)
            .map(|(p, vector)| EmbeddedLines {
                note_id: p.note_id,
                start: p.start,
                end: p.end,
                hash: p.hash,
                vector,
            })
            .collect();
        let n = rows.len();
        store.put_passage_embeddings("letters", &rows).unwrap();
        n
    }

    #[test]
    fn only_a_long_note_gets_passage_vectors_and_its_tail_is_found() {
        let (_dir, store) = open();
        note(&store, "Short", "A short note about zebras.", vec![]);
        let long = note(
            &store,
            "Long",
            &long_body(8, "zzzz quzzy jazz fizz buzz"),
            vec![],
        );
        assert!(index(&store) > 0);
        assert!(store.has_passage_vectors(&long).unwrap());
        let hits = store
            .search_passages_by_meaning("zzzz quzzy jazz fizz buzz", 3)
            .unwrap();
        assert_eq!(hits[0].note_id, long);
        assert!(hits[0].start > 10, "{hits:?}");
        // Nothing is waiting once everything is embedded.
        assert!(
            store
                .passage_queue("letters", 100)
                .unwrap()
                .pending
                .is_empty()
        );
    }

    #[test]
    fn a_changed_passage_is_embedded_again_and_the_rest_are_kept() {
        let (_dir, store) = open();
        let id = note(&store, "Long", &long_body(8, "first ending"), vec![]);
        let first = index(&store);
        store
            .put(
                DocType::Note,
                json!({"_id": id, "body": long_body(8, "second ending")}),
                Put::Merge,
                Origin::Ui,
            )
            .unwrap();
        let queue = store.passage_queue("letters", 100).unwrap();
        assert_eq!(queue.pending.len(), 1, "of {first}");
        assert_eq!(queue.notes_waiting, 1);
    }

    #[test]
    fn a_note_that_stops_being_long_loses_its_vectors() {
        let (_dir, store) = open();
        let id = note(&store, "Long", &long_body(8, "tail"), vec![]);
        index(&store);
        assert!(store.has_passage_vectors(&id).unwrap());
        store
            .put(
                DocType::Note,
                json!({"_id": id, "body": "short now"}),
                Put::Merge,
                Origin::Ui,
            )
            .unwrap();
        store.passage_queue("letters", 100).unwrap();
        assert!(!store.has_passage_vectors(&id).unwrap());
    }

    #[test]
    fn turning_related_notes_off_empties_the_table() {
        let (_dir, store) = open();
        let id = note(&store, "Long", &long_body(8, "tail"), vec![]);
        index(&store);
        store.set_embedding_model(None).unwrap();
        assert!(!store.has_passage_vectors(&id).unwrap());
        assert!(
            store
                .search_passages_by_meaning("tail", 3)
                .unwrap()
                .is_empty()
        );
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(24))]

        /// A locked or `#ai-exclude` note never holds a passage vector, nor
        /// shows up in a search by meaning, whenever it was locked or tagged.
        #[test]
        fn prop_passage_embeddings_never_hold_a_locked_or_excluded_note(
            lock_before in any::<bool>(),
            tag_before in any::<bool>(),
            lock in any::<bool>(),
            tag in any::<bool>(),
        ) {
            let (_dir, store) = open();
            let exclude = store
                .put(DocType::Tag, json!({"name": "ai-exclude"}), Put::Upsert, Origin::Ui)
                .unwrap()
                .id;
            let secret = long_body(8, "the secret passphrase is quokka");
            let tags = |on: bool| if on { vec![exclude.clone()] } else { vec![] };
            let id = note(&store, "Secret", &secret, tags(tag_before));
            if lock_before {
                store.put(DocType::Note, json!({"_id": id, "kansoLock": {"salt": "abc"}}), Put::Merge, Origin::Sync).unwrap();
            }
            index(&store);
            if lock {
                store.put(DocType::Note, json!({"_id": id, "kansoLock": {"salt": "abc"}}), Put::Merge, Origin::Sync).unwrap();
            }
            if tag {
                store.put(DocType::Note, json!({"_id": id, "tags": tags(true)}), Put::Merge, Origin::Ui).ok();
            }
            // The next queue read, and any write, drop what can't be kept.
            let queue = store.passage_queue("letters", 1000).unwrap();
            let hidden = lock_before || tag_before || lock || tag;
            if hidden {
                prop_assert!(queue.pending.iter().all(|p| p.note_id != id));
                prop_assert!(!store.has_passage_vectors(&id).unwrap());
                let hits = store.search_passages_by_meaning("the secret passphrase is quokka", 5).unwrap();
                prop_assert!(hits.iter().all(|h| h.note_id != id));
                // A late write for it is refused.
                store.put_passage_embeddings("letters", &[EmbeddedLines {
                    note_id: id.clone(), start: 0, end: 1, hash: "h".into(), vector: vec![1.; 26],
                }]).unwrap();
                prop_assert!(!store.has_passage_vectors(&id).unwrap());
            }
        }
    }
}
