//! Note embeddings for related notes.
//!
//! Each live note can have one vector per embedding model, made on this
//! device from its title and body and kept with a hash of that text, so a
//! note is embedded again only when its words change. The table is local:
//! never synced, emptied with the library, and emptied when related notes
//! are turned off. The model itself lives in the app; the store only keeps
//! vectors and compares them, so the local API can answer from them
//! without loading a model.
//!
//! Which model is in use is kept in `meta` under [`MODEL_KEY`]: present
//! only while related notes are on.
//!
//! A note that is locked (sealed, or in a locked notebook) or tagged
//! `#ai-exclude` is never embedded. Its vector is not made, a vector it
//! already had is deleted the next time the queue is read or a vector is
//! kept, and until then no read here (related notes, tag suggestions,
//! twins, searches by meaning) uses it.

use std::collections::HashSet;

use kanso_core::links::{normalize_title, plain_line};
use md5::{Digest as _, Md5};
use rusqlite::{Connection, OptionalExtension, params};

use crate::{Result, Store, schema};

/// The `meta` key holding the model whose embeddings are in use, while
/// related notes are on.
pub const MODEL_KEY: &str = "embed.model";

/// The table, for emptying it with the library.
pub(crate) const TABLES: [&str; 1] = ["note_embeddings"];

/// Changes whenever [`embed_text`] builds the text differently, so every
/// note is embedded again.
const TEXT_VERSION: &str = "1";
/// How much of a note is embedded, in characters. The models read a few
/// hundred words at most; the rest only costs time.
const TEXT_CHARS: usize = 6_000;

/// Something that turns texts into vectors of one fixed length: the app's
/// static model, or a fake in tests.
pub trait Embedder: Send + Sync {
    /// Names the model and its version; embeddings from different models
    /// are never compared.
    fn model(&self) -> &str;
    fn dim(&self) -> usize;
    /// One vector of [`Embedder::dim`] numbers per text.
    fn embed(&self, texts: &[String]) -> Vec<Vec<f32>>;
}

/// A note similar to another, by the cosine of their embeddings.
#[derive(Clone, Debug, PartialEq)]
pub struct RelatedNote {
    pub id: String,
    pub title: String,
    /// The cosine similarity, from -1 to 1.
    pub score: f32,
}

/// A note waiting to be embedded: its text and that text's hash.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingEmbedding {
    pub id: String,
    pub text: String,
    pub hash: String,
}

/// What [`Store::embedding_queue`] found.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EmbeddingQueue {
    /// The notes whose embedding is missing or out of date.
    pub pending: Vec<PendingEmbedding>,
    /// How many notes can be embedded, embedded or not: the live ones
    /// outside the trash that aren't locked or tagged `#ai-exclude`.
    pub total: usize,
}

/// The words of a note that are embedded: its title, then its body as
/// plain lines, without fenced code and link targets, cut short.
pub fn embed_text(title: &str, body: &str) -> String {
    let mut text = String::with_capacity(body.len().min(TEXT_CHARS) + title.len() + 2);
    text.push_str(title.trim());
    let mut in_fence = false;
    let mut chars = text.chars().count();
    for line in body.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        let plain = plain_line(line);
        if plain.is_empty() {
            continue;
        }
        text.push('\n');
        for c in plain.chars() {
            if chars >= TEXT_CHARS {
                return text;
            }
            text.push(c);
            chars += 1;
        }
    }
    text
}

/// A hash of the text a note is embedded from, and of how it is built.
pub fn content_hash(text: &str) -> String {
    let mut hasher = Md5::new();
    hasher.update(TEXT_VERSION.as_bytes());
    hasher.update([0]);
    hasher.update(text.as_bytes());
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// The cosine similarity of two vectors: 0 when either is all zeros or
/// they differ in length.
pub fn cosine(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.;
    }
    let (mut dot, mut aa, mut bb) = (0f64, 0f64, 0f64);
    for (x, y) in a.iter().zip(b) {
        let (x, y) = (f64::from(*x), f64::from(*y));
        dot += x * y;
        aa += x * x;
        bb += y * y;
    }
    if aa <= f64::EPSILON || bb <= f64::EPSILON || !dot.is_finite() {
        return 0.;
    }
    (dot / (aa.sqrt() * bb.sqrt())).clamp(-1., 1.) as f32
}

/// The `k` candidates most similar to `query`, most similar first, ties by
/// ID; candidates with no direction (all zeros) are left out.
pub fn top_k<'a>(
    query: &[f32],
    candidates: impl IntoIterator<Item = (&'a str, &'a [f32])>,
    k: usize,
) -> Vec<(String, f32)> {
    if k == 0 {
        return Vec::new();
    }
    let mut scored: Vec<(&str, f32)> = candidates
        .into_iter()
        .filter(|(_, v)| v.iter().any(|x| *x != 0.))
        .map(|(id, v)| (id, cosine(query, v)))
        .collect();
    scored.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(b.0)));
    scored.truncate(k);
    scored
        .into_iter()
        .map(|(id, score)| (id.to_owned(), score))
        .collect()
}

/// A vector as stored: little-endian f32s.
pub fn encode_vector(vector: &[f32]) -> Vec<u8> {
    vector.iter().flat_map(|x| x.to_le_bytes()).collect()
}

/// A stored vector, or `None` when the bytes don't hold `dim` numbers.
pub fn decode_vector(bytes: &[u8], dim: usize) -> Option<Vec<f32>> {
    if bytes.len() != dim * 4 {
        return None;
    }
    Some(
        bytes
            .as_chunks::<4>()
            .0
            .iter()
            .map(|b| f32::from_le_bytes(*b))
            .collect(),
    )
}

/// Removes note `id`'s embeddings, when it is deleted.
pub(crate) fn forget_note(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM note_embeddings WHERE note_id = ?", [id])?;
    Ok(())
}

/// Deletes the vectors of notes that are locked or tagged `#ai-exclude`:
/// they may have been embedded before they were locked or tagged.
fn forget_undiscoverable(conn: &Connection) -> Result<()> {
    conn.execute(
        concat!(
            "DELETE FROM note_embeddings WHERE note_id IN (SELECT d.id FROM docs d
                WHERE d.type = 'note' AND (",
            crate::schema::note_locked!("d."),
            " OR ",
            crate::schema::ai_excluded!("d."),
            "))"
        ),
        [],
    )?;
    Ok(())
}

/// The live notes outside the trash that note `id` links to, or that link
/// to it, by ID or by wiki-link title.
fn linked_notes(conn: &Connection, id: &str) -> Result<HashSet<String>> {
    let title: Option<String> = conn
        .query_row("SELECT title FROM docs WHERE id = ?", [id], |row| {
            row.get(0)
        })
        .optional()?
        .flatten();
    let own_title = title.map(|t| normalize_title(&t)).unwrap_or_default();
    let mut stmt = conn.prepare(
        "SELECT target_id FROM note_links WHERE note_id = ?1 AND target_id IS NOT NULL
         UNION SELECT d.id FROM note_links l JOIN docs d
             ON d.type = 'note' AND d.deleted = 0 AND kanso_norm_title(d.title) = l.target_title
             WHERE l.note_id = ?1 AND l.target_title IS NOT NULL
         UNION SELECT note_id FROM note_links
             WHERE target_id = ?1 OR (?2 != '' AND target_title = ?2)",
    )?;
    let rows = stmt.query_map(params![id, own_title], |row| row.get::<_, String>(0))?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

impl Store {
    /// Deletes the vectors of notes that are locked or tagged `#ai-exclude`.
    fn forget_undiscoverable(&self) -> Result<()> {
        self.write(|tx, _| forget_undiscoverable(tx))
    }

    /// The model whose embeddings are in use, while related notes are on.
    pub fn embedding_model(&self) -> Result<Option<String>> {
        self.meta(MODEL_KEY)
    }

    /// Turns related notes on with `model`, or off (`None`). Embeddings
    /// from any other model are removed; turning off removes them all.
    pub fn set_embedding_model(&self, model: Option<&str>) -> Result<()> {
        self.write(|tx, _| {
            match model {
                Some(model) => {
                    tx.execute("DELETE FROM note_embeddings WHERE model != ?", [model])?;
                    schema::set_meta(tx, MODEL_KEY, model)?;
                }
                None => {
                    tx.execute("DELETE FROM note_embeddings", [])?;
                    tx.execute("DELETE FROM meta WHERE key = ?", [MODEL_KEY])?;
                }
            }
            Ok(())
        })
    }

    /// The live notes outside the trash whose embedding by `model` is
    /// missing or was made from other words, oldest change last so the
    /// notes worked on most recently are embedded first. Locked notes and
    /// notes tagged `#ai-exclude` are not in it, and any vector they have
    /// is deleted first.
    pub fn embedding_queue(&self, model: &str) -> Result<EmbeddingQueue> {
        self.forget_undiscoverable()?;
        self.read(|conn| {
            let mut stmt = conn.prepare(concat!(
                "SELECT d.id, d.title, ",
                crate::schema::readable_body!("d."),
                ", e.content_hash
                 FROM docs d LEFT JOIN note_embeddings e ON e.note_id = d.id AND e.model = ?1
                 WHERE ",
                crate::schema::discoverable!("d."),
                " ORDER BY d.updated_at DESC, d.id"
            ))?;
            let mut rows = stmt.query([model])?;
            let mut queue = EmbeddingQueue::default();
            while let Some(row) = rows.next()? {
                queue.total += 1;
                let title: Option<String> = row.get(1)?;
                let body: Option<String> = row.get(2)?;
                let text = embed_text(
                    title.as_deref().unwrap_or(""),
                    body.as_deref().unwrap_or(""),
                );
                let hash = content_hash(&text);
                let kept: Option<String> = row.get(3)?;
                if kept.as_deref() != Some(hash.as_str()) {
                    queue.pending.push(PendingEmbedding {
                        id: row.get(0)?,
                        text,
                        hash,
                    });
                }
            }
            Ok(queue)
        })
    }

    /// Keeps embeddings made by `model`, each with the hash of the text it
    /// was made from. Notes deleted meanwhile are skipped, and nothing is
    /// kept once related notes are off or on another model.
    pub fn put_embeddings(&self, model: &str, rows: &[(String, String, Vec<f32>)]) -> Result<()> {
        self.write(|tx, _| {
            if schema::get_meta(tx, MODEL_KEY)?.as_deref() != Some(model) {
                return Ok(());
            }
            // Whatever can't be embedded any more loses its vectors, even
            // those of other notes than these.
            forget_undiscoverable(tx)?;
            let mut stmt = tx.prepare_cached(concat!(
                "INSERT INTO note_embeddings (note_id, model, dim, vector, content_hash)
                 SELECT ?1, ?2, ?3, ?4, ?5 WHERE EXISTS
                     (SELECT 1 FROM docs d WHERE d.id = ?1 AND ",
                crate::schema::discoverable!("d."),
                ")
                 ON CONFLICT (note_id, model) DO UPDATE SET dim = excluded.dim,
                     vector = excluded.vector, content_hash = excluded.content_hash"
            ))?;
            // The words as the note reads now: one locked (or edited) since
            // the queue was read keeps no vector made from its old words.
            let mut now = tx.prepare_cached(concat!(
                "SELECT title, ",
                crate::schema::readable_body!(""),
                " FROM docs d WHERE d.id = ? AND ",
                crate::schema::discoverable!("d.")
            ))?;
            for (id, hash, vector) in rows {
                let current = now
                    .query_row([id], |row| {
                        Ok(content_hash(&embed_text(
                            row.get::<_, Option<String>>(0)?.as_deref().unwrap_or(""),
                            row.get::<_, Option<String>>(1)?.as_deref().unwrap_or(""),
                        )))
                    })
                    .optional()?;
                if current.as_deref() != Some(hash.as_str()) {
                    continue;
                }
                stmt.execute(params![
                    id,
                    model,
                    vector.len() as i64,
                    encode_vector(vector),
                    hash
                ])?;
            }
            Ok(())
        })
    }

    /// How many notes that can be embedded (live, outside the trash, not
    /// locked or tagged `#ai-exclude`) have an embedding by `model`.
    pub fn embedded_count(&self, model: &str) -> Result<usize> {
        self.read(|conn| {
            Ok(conn.query_row(
                concat!(
                    "SELECT COUNT(*) FROM note_embeddings e JOIN docs d ON d.id = e.note_id
                     WHERE e.model = ? AND ",
                    crate::schema::discoverable!("d.")
                ),
                [model],
                |row| row.get::<_, i64>(0),
            )? as usize)
        })
    }

    /// The `k` live notes outside the trash most similar to note `id` by
    /// `model`'s embeddings, leaving out the note itself and the notes it
    /// links to or is linked from. `None` when note `id` has no embedding
    /// yet.
    pub fn related_notes(
        &self,
        id: &str,
        model: &str,
        k: usize,
    ) -> Result<Option<Vec<RelatedNote>>> {
        self.read(|conn| {
            let Some(query) = own_vector(conn, id, model)? else {
                return Ok(None);
            };
            let mut skip = linked_notes(conn, id)?;
            skip.insert(id.to_owned());
            let candidates = embedded_notes(conn, model, &skip)?;
            Ok(Some(nearest(&query, &candidates, k)))
        })
    }
}

/// Note `id`'s embedding by `model`, if it has one and the note can be
/// embedded (it isn't locked or tagged `#ai-exclude`).
pub(crate) fn own_vector(conn: &Connection, id: &str, model: &str) -> Result<Option<Vec<f32>>> {
    let own: Option<(i64, Vec<u8>)> = conn
        .query_row(
            concat!(
                "SELECT e.dim, e.vector FROM note_embeddings e JOIN docs d ON d.id = e.note_id
                 WHERE e.note_id = ? AND e.model = ? AND ",
                crate::schema::discoverable!("d.")
            ),
            params![id, model],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    Ok(own.and_then(|(dim, v)| decode_vector(&v, dim as usize)))
}

/// A note with an embedding: its ID, title and vector.
pub(crate) type Embedded = (String, String, Vec<f32>);

/// The live notes outside the trash with an embedding by `model`, apart
/// from those in `skip` and from locked notes and notes tagged
/// `#ai-exclude`.
pub(crate) fn embedded_notes(
    conn: &Connection,
    model: &str,
    skip: &HashSet<String>,
) -> Result<Vec<Embedded>> {
    let mut stmt = conn.prepare(concat!(
        "SELECT d.id, d.title, e.dim, e.vector FROM note_embeddings e
         JOIN docs d ON d.id = e.note_id
         WHERE e.model = ?1 AND ",
        crate::schema::discoverable!("d.")
    ))?;
    let mut rows = stmt.query(params![model])?;
    let mut candidates = Vec::new();
    while let Some(row) = rows.next()? {
        let note: String = row.get(0)?;
        if skip.contains(&note) {
            continue;
        }
        let dim: i64 = row.get(2)?;
        let bytes: Vec<u8> = row.get(3)?;
        if let Some(vector) = decode_vector(&bytes, dim as usize) {
            let title: Option<String> = row.get(1)?;
            candidates.push((note, title.unwrap_or_default(), vector));
        }
    }
    Ok(candidates)
}

/// The `k` of `candidates` nearest `query`, most similar first.
pub(crate) fn nearest(query: &[f32], candidates: &[Embedded], k: usize) -> Vec<RelatedNote> {
    let best = top_k(
        query,
        candidates
            .iter()
            .map(|(id, _, v)| (id.as_str(), v.as_slice())),
        k,
    );
    best.into_iter()
        .map(|(id, score)| {
            let title = candidates
                .iter()
                .find(|(c, _, _)| *c == id)
                .map(|(_, t, _)| t.clone())
                .unwrap_or_default();
            RelatedNote { id, title, score }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Origin, Put};
    use kanso_core::DocType;
    use proptest::prelude::*;
    use serde_json::json;

    /// Counts letters: notes with the same letters point the same way.
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

    fn note(store: &Store, title: &str, body: &str) -> String {
        store
            .put(
                DocType::Note,
                json!({"title": title, "body": body, "bookId": book(store)}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id
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

    /// Embeds whatever is waiting, as the app's indexer does.
    fn index(store: &Store, embedder: &dyn Embedder) -> usize {
        let queue = store.embedding_queue(embedder.model()).unwrap();
        let texts: Vec<String> = queue.pending.iter().map(|p| p.text.clone()).collect();
        let vectors = embedder.embed(&texts);
        let rows: Vec<_> = queue
            .pending
            .into_iter()
            .zip(vectors)
            .map(|(p, v)| (p.id, p.hash, v))
            .collect();
        let n = rows.len();
        store.put_embeddings(embedder.model(), &rows).unwrap();
        n
    }

    fn open() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("kanso.db")).unwrap();
        (dir, store)
    }

    /// How many vectors the table holds for `model`, whoever they are of.
    fn stored_vectors(store: &Store, model: &str) -> i64 {
        store
            .read(|conn| {
                Ok(conn.query_row(
                    "SELECT COUNT(*) FROM note_embeddings WHERE model = ?",
                    [model],
                    |row| row.get(0),
                )?)
            })
            .unwrap()
    }

    #[test]
    fn vectors_of_notes_locked_or_excluded_since_are_deleted_for_good() {
        let (_dir, store) = open();
        store.set_embedding_model(Some("letters")).unwrap();
        let plain = note(&store, "Plain", "orchard fruit");
        let locked = note(&store, "Locked", "orchard fruit");
        let excluded = note(&store, "Excluded", "orchard fruit");
        index(&store, &Letters);
        assert_eq!(stored_vectors(&store, "letters"), 3);

        let mut sealed = store.get_api(&locked, None, false).unwrap();
        sealed["body"] = "".into();
        sealed["kansoLock"] =
            json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "sealed"});
        store.lock_note(sealed).unwrap();
        let tag = store
            .put(
                DocType::Tag,
                json!({"name": "#AI-exclude"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        store
            .put(
                DocType::Note,
                json!({"_id": excluded, "tags": [tag]}),
                Put::Merge,
                Origin::Ui,
            )
            .unwrap();
        // The write that hid them may already have dropped their vectors, so
        // however many rows are left, no read uses one of them, and reading
        // the queue deletes the rest (below).
        assert!(stored_vectors(&store, "letters") <= 3);
        assert_eq!(store.embedded_count("letters").unwrap(), 1);
        assert!(
            store
                .related_notes(&plain, "letters", 5)
                .unwrap()
                .unwrap()
                .is_empty()
        );
        assert!(
            store
                .related_notes(&locked, "letters", 5)
                .unwrap()
                .is_none()
        );

        let queue = store.embedding_queue("letters").unwrap();
        assert_eq!((queue.total, queue.pending.len()), (1, 0));
        assert_eq!(stored_vectors(&store, "letters"), 1);

        // The tag taken off: the note is embedded again.
        store
            .put(
                DocType::Note,
                json!({"_id": excluded, "tags": []}),
                Put::Merge,
                Origin::Ui,
            )
            .unwrap();
        assert_eq!(index(&store, &Letters), 1);
        assert_eq!(stored_vectors(&store, "letters"), 2);
    }

    /// A library that embedded a note before it was locked keeps its vector:
    /// no write made the lock, so nothing dropped it. No read uses it, and
    /// the queue deletes it, so it can never be found again.
    #[test]
    fn a_vector_from_before_the_lock_is_deleted_for_good_when_the_queue_is_read() {
        let (_dir, store) = open();
        store.set_embedding_model(Some("letters")).unwrap();
        let id = note(&store, "Secret", "orchard fruit");
        let mut sealed = store.get_api(&id, None, false).unwrap();
        sealed["body"] = "".into();
        sealed["kansoLock"] =
            json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "sealed"});
        store.lock_note(sealed).unwrap();
        store
            .write(|tx, _| {
                tx.execute(
                    "INSERT OR REPLACE INTO note_embeddings
                         (note_id, model, dim, vector, content_hash)
                     VALUES (?1, 'letters', 26, ?2, 'stale')",
                    rusqlite::params![id, vec![0u8; 26 * 4]],
                )?;
                Ok(())
            })
            .unwrap();
        assert_eq!(stored_vectors(&store, "letters"), 1);

        assert_eq!(store.embedded_count("letters").unwrap(), 0);
        assert!(store.related_notes(&id, "letters", 5).unwrap().is_none());

        let queue = store.embedding_queue("letters").unwrap();
        assert_eq!((queue.total, queue.pending.len()), (0, 0));
        assert_eq!(stored_vectors(&store, "letters"), 0);
    }

    #[test]
    fn a_vector_for_a_note_that_may_not_be_embedded_is_refused() {
        let (_dir, store) = open();
        store.set_embedding_model(Some("letters")).unwrap();
        let tag = store
            .put(
                DocType::Tag,
                json!({"name": "ai-exclude"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        let book = book(&store);
        let excluded = store
            .put(
                DocType::Note,
                json!({"title": "X", "body": "words", "bookId": book, "tags": [tag]}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        let hash = content_hash(&embed_text("X", "words"));
        store
            .put_embeddings("letters", &[(excluded, hash, vec![1.0; 26])])
            .unwrap();
        assert_eq!(stored_vectors(&store, "letters"), 0);
    }

    #[test]
    fn embeddings_follow_the_words_and_the_switch() {
        let (_dir, store) = open();
        let a = note(&store, "aaaa", "aaa ab");
        let b = note(&store, "aaab", "aaa");
        let c = note(&store, "zzzz", "zzz");
        // Off: nothing is kept.
        store
            .put_embeddings("letters", &[(a.clone(), "h".into(), vec![1.; 26])])
            .unwrap();
        assert_eq!(store.embedded_count("letters").unwrap(), 0);
        store.set_embedding_model(Some("letters")).unwrap();
        assert_eq!(index(&store, &Letters), 3);
        // Nothing changed: nothing to do.
        assert_eq!(index(&store, &Letters), 0);
        let related = store.related_notes(&a, "letters", 5).unwrap().unwrap();
        assert_eq!(related[0].id, b);
        assert_eq!(related.last().unwrap().id, c);
        assert!(related[0].score > related[1].score);
        // Changing the words makes the note wait again; deleting forgets it.
        let rev = store.note(&c).unwrap().unwrap().rev.unwrap();
        store
            .put(
                DocType::Note,
                json!({"_id": c, "_rev": rev, "body": "aaaa"}),
                Put::Merge,
                Origin::Ui,
            )
            .unwrap();
        assert_eq!(store.embedding_queue("letters").unwrap().pending.len(), 1);
        store.delete(&c, Origin::Ui).unwrap();
        assert_eq!(store.embedding_queue("letters").unwrap().pending.len(), 0);
        assert_eq!(store.embedded_count("letters").unwrap(), 2);
        // Off again: all gone, and the library's own emptying clears them.
        store.set_embedding_model(None).unwrap();
        assert_eq!(store.embedded_count("letters").unwrap(), 0);
        store.set_embedding_model(Some("letters")).unwrap();
        index(&store, &Letters);
        store.replace_library(&[]).unwrap();
        assert_eq!(store.embedded_count("letters").unwrap(), 0);
    }

    #[test]
    fn related_notes_leave_out_linked_ones() {
        let (_dir, store) = open();
        let a = note(&store, "Alpha", "apples and pears");
        let b = note(&store, "Beta", "apples and pears too");
        let c = note(&store, "Gamma", "apples and pears again, see [[Alpha]]");
        let d = note(
            &store,
            "Delta",
            &format!(
                "apples, pears [x](inkdrop://note/{})",
                b.strip_prefix("note:").unwrap()
            ),
        );
        store.set_embedding_model(Some("letters")).unwrap();
        index(&store, &Letters);
        let ids = |id: &str| -> Vec<String> {
            store
                .related_notes(id, "letters", 10)
                .unwrap()
                .unwrap()
                .into_iter()
                .map(|r| r.id)
                .collect()
        };
        // C links to A by title; D links to B by ID.
        let mut found = ids(&a);
        found.sort();
        let mut expected = vec![b.clone(), d.clone()];
        expected.sort();
        assert_eq!(found, expected);
        assert!(!ids(&b).contains(&d));
        assert!(!ids(&c).contains(&a));
        assert!(!ids(&a).contains(&a));
        // A note not embedded yet has no answer.
        let e = note(&store, "Epsilon", "new");
        assert_eq!(store.related_notes(&e, "letters", 3).unwrap(), None);
    }

    #[test]
    fn embed_text_takes_the_words_without_code() {
        let text = embed_text(
            " Plan ",
            "# Goals\n\n- [x] Ship **it**\n```rust\nfn x() {}\n```\nSee [[Other note|this]].",
        );
        assert_eq!(text, "Plan\nGoals\nShip it\nSee this.");
    }

    fn config(cases: u32) -> ProptestConfig {
        ProptestConfig {
            cases,
            failure_persistence: Some(Box::new(
                proptest::test_runner::FileFailurePersistence::WithSource("regressions"),
            )),
            ..ProptestConfig::default()
        }
    }

    proptest! {
        #![proptest_config(config(128))]

        /// Cosine stays in [-1, 1], is symmetric, and a vector is most
        /// like itself.
        #[test]
        fn prop_cosine_is_bounded_and_symmetric(
            a in prop::collection::vec(-100f32..100., 1..16),
            b in prop::collection::vec(-100f32..100., 1..16),
        ) {
            let b: Vec<f32> = b.into_iter().cycle().take(a.len()).collect();
            let ab = cosine(&a, &b);
            prop_assert!((-1. ..=1.).contains(&ab));
            prop_assert!((ab - cosine(&b, &a)).abs() < 1e-6);
            if a.iter().any(|x| x.abs() > 1e-3) {
                prop_assert!((cosine(&a, &a) - 1.).abs() < 1e-4);
                prop_assert!(cosine(&a, &a) >= ab - 1e-6);
            }
        }

        /// Top-k is sorted best first, at most k long, ties by ID, and
        /// nothing left out beats anything kept.
        #[test]
        fn prop_top_k_orders_and_keeps_the_best(
            query in prop::collection::vec(-10f32..10., 4),
            rows in prop::collection::vec(prop::collection::vec(-10f32..10., 4), 0..30),
            k in 0usize..12,
        ) {
            let ids: Vec<String> = (0..rows.len()).map(|i| format!("note:{i:03}")).collect();
            let best = top_k(&query, ids.iter().map(String::as_str).zip(rows.iter().map(Vec::as_slice)), k);
            let live = rows.iter().filter(|v| v.iter().any(|x| *x != 0.)).count();
            prop_assert_eq!(best.len(), k.min(live));
            for pair in best.windows(2) {
                prop_assert!(pair[0].1 > pair[1].1 || (pair[0].1 == pair[1].1 && pair[0].0 < pair[1].0));
            }
            if let Some(worst) = best.last() {
                for (id, v) in ids.iter().zip(&rows) {
                    if !best.iter().any(|(b, _)| b == id) && v.iter().any(|x| *x != 0.) {
                        prop_assert!(cosine(&query, v) <= worst.1);
                    }
                }
            }
        }

        /// Vectors round-trip through their stored bytes.
        #[test]
        fn prop_vectors_round_trip(v in prop::collection::vec(any::<f32>().prop_filter("finite", |x| x.is_finite()), 0..64)) {
            prop_assert_eq!(decode_vector(&encode_vector(&v), v.len()), Some(v.clone()));
            prop_assert_eq!(decode_vector(&encode_vector(&v), v.len() + 1), None);
        }

    }

    proptest! {
        #![proptest_config(config(24))]

        /// The cache follows the words: the same text keeps its hash, and
        /// a note whose embedded words change waits to be embedded again
        /// while one whose other fields change does not.
        #[test]
        fn prop_the_cache_invalidates_on_content_change(
            title in "[a-z ]{0,12}",
            body in "[a-z \n#*-]{0,60}",
            extra in "[a-z]{1,8}",
        ) {
            let (_dir, store) = open();
            let id = note(&store, &title, &body);
            store.set_embedding_model(Some("letters")).unwrap();
            index(&store, &Letters);
            prop_assert!(store.embedding_queue("letters").unwrap().pending.is_empty());
            let rev = store.note(&id).unwrap().unwrap().rev.unwrap();
            let rev = store.put(DocType::Note, json!({"_id": id, "_rev": rev, "pinned": true}), Put::Merge, Origin::Ui).unwrap().rev;
            prop_assert!(store.embedding_queue("letters").unwrap().pending.is_empty());
            let new_body = format!("{body}\n{extra}");
            let changed = embed_text(&title, &new_body) != embed_text(&title, &body);
            store.put(DocType::Note, json!({"_id": id, "_rev": rev, "body": new_body}), Put::Merge, Origin::Ui).unwrap();
            prop_assert_eq!(store.embedding_queue("letters").unwrap().pending.len(), usize::from(changed));
        }
    }
}
