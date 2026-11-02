//! Resurfacing: past notes about what is being written now, notes written
//! on this day in earlier years, and the notes a person said were not
//! related. See `docs`-level description in the issue "Resurfacing related
//! notes while you write".
//!
//! Everything here reads vectors this device made; nothing leaves it.
//! Locked notes and notes tagged `#ai-exclude` are never searched or
//! shown, whatever the index still remembers of them.

use std::collections::HashSet;

use rusqlite::{Connection, OptionalExtension, params};

use crate::embed::{Embedded, TEXT_CHARS, embedded_notes, linked_notes};
use crate::passages::{Passage, decode_i8, dot_i8, passages, plain_text, quantize};
use crate::{DayNote, Result, ResurfaceOpts, Resurfaced, Resurfacing, Store};

use super::index::{CANDIDATES, Index};

/// How many of the nearest notes get their best passage looked for before
/// the first `k` are kept.
const PASSAGE_NOTES: usize = 8;
/// How long a snippet is, in characters.
const SNIPPET_CHARS: usize = 160;
/// Scoring in int8 can be off by this much; candidates a little under the
/// threshold are scored again in full.
const INT8_MARGIN: f32 = 0.03;

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as i64)
}

/// A passage's text as one plain line of at most [`SNIPPET_CHARS`]
/// characters, cut at a word, with an ellipsis where the passage goes on
/// or the note goes on before it.
pub fn snippet(body: &str, start: usize, end: usize) -> String {
    let mut start = start.min(body.len());
    let mut end = end.min(body.len()).max(start);
    while !body.is_char_boundary(start) {
        start -= 1;
    }
    while !body.is_char_boundary(end) {
        end += 1;
    }
    let plain = plain_text(&body[start..end]);
    let mut out = String::new();
    if start > 0 && !plain.is_empty() {
        out.push('\u{2026}');
    }
    if plain.chars().count() <= SNIPPET_CHARS {
        out.push_str(&plain);
        return out;
    }
    let mut cut = 0;
    for (n, (at, _)) in plain.char_indices().enumerate() {
        if n == SNIPPET_CHARS {
            cut = at;
            break;
        }
    }
    let mut text = &plain[..cut];
    if let Some(space) = text.rfind(' ') {
        text = &text[..space];
    }
    out.push_str(text.trim_end());
    out.push('\u{2026}');
    out
}

/// The words of `text` that count for a match: lowercase, three letters
/// or more.
fn match_words(text: &str) -> HashSet<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.chars().count() >= 3)
        .map(str::to_lowercase)
        .collect()
}

/// The passage of `body` sharing the most words with `query`: the stopgap
/// for a note whose passages aren't embedded yet.
fn overlapping_passage(body: &str, query: &str) -> Option<(usize, usize)> {
    let wanted = match_words(query);
    let found: Vec<Passage> = passages(body, TEXT_CHARS);
    found
        .iter()
        .max_by_key(|p| {
            (
                match_words(&p.text).intersection(&wanted).count(),
                std::cmp::Reverse(p.ord),
            )
        })
        .map(|p| (p.start, p.end))
}

/// The best of a note's embedded passages for `query`, if it has any.
fn best_passage(
    conn: &Connection,
    id: &str,
    model: &str,
    query: &(f32, Vec<i8>),
) -> Result<Option<(usize, usize)>> {
    let mut stmt = conn.prepare_cached(
        "SELECT start_byte, end_byte, dim, vector FROM note_passages
         WHERE note_id = ?1 AND model = ?2 ORDER BY ord",
    )?;
    let mut rows = stmt.query(params![id, model])?;
    let mut best: Option<(f32, (usize, usize))> = None;
    while let Some(row) = rows.next()? {
        let (start, end, dim): (i64, i64, i64) = (row.get(0)?, row.get(1)?, row.get(2)?);
        let bytes: Vec<u8> = row.get(3)?;
        let Some((scale, numbers)) = decode_i8(&bytes, dim as usize) else {
            continue;
        };
        let score = dot_i8(&query.1, query.0, &numbers, scale);
        if best.is_none_or(|(top, _)| score > top) {
            best = Some((score, (start as usize, end as usize)));
        }
    }
    Ok(best.map(|(_, range)| range))
}

impl Store {
    /// The nearest notes to `vector` from the index, loading it first when
    /// it isn't loaded or is of another model.
    fn index_nearest(
        &self,
        model: &str,
        vector: &[f32],
        skip: &str,
        floor: f32,
    ) -> Result<Vec<(String, f32)>> {
        let mut index = self.index.lock().expect("index poisoned");
        if index.as_ref().is_none_or(|i| i.model != model) {
            let notes: Vec<Embedded> = self.read(|conn| {
                let none = HashSet::new();
                embedded_notes(conn, model, &none)
            })?;
            let mut loaded = Index::new(model, vector.len());
            for (id, _, v) in &notes {
                loaded.put(id, v);
            }
            *index = Some(loaded);
        }
        Ok(index
            .as_ref()
            .map(|i| i.nearest(vector, skip, floor, CANDIDATES))
            .unwrap_or_default())
    }

    /// The notes `id` was said to be unrelated to ("Not related" in its
    /// margin) and the twins it was said not to be.
    pub fn dismissed_related(&self, id: &str) -> Result<HashSet<String>> {
        self.read(|conn| dismissed_for(conn, id))
    }

    /// Note `other` is not shown beside note `id` again.
    pub fn dismiss_related(&self, id: &str, other: &str) -> Result<()> {
        self.write(|tx, _| {
            tx.execute(
                "INSERT OR REPLACE INTO resurface_dismissed (note_id, other_id, at)
                 VALUES (?1, ?2, ?3)",
                params![id, other, now_ms()],
            )?;
            Ok(())
        })
    }

    /// The two notes are different notes after all: dismissed from both
    /// sides, for the twin banner and the margin alike.
    pub fn dismiss_twin(&self, a: &str, b: &str) -> Result<()> {
        self.dismiss_related(a, b)?;
        self.dismiss_related(b, a)
    }

    /// Whether `a` and `b` were said not to be twins (or not related),
    /// from either side.
    pub fn is_dismissed(&self, a: &str, b: &str) -> Result<bool> {
        self.read(|conn| {
            Ok(conn
                .query_row(
                    "SELECT 1 FROM resurface_dismissed
                     WHERE (note_id = ?1 AND other_id = ?2) OR (note_id = ?2 AND other_id = ?1)",
                    params![a, b],
                    |_| Ok(()),
                )
                .optional()?
                .is_some())
        })
    }

    /// Up to `opts.k` past notes about `text` (the paragraph being written),
    /// closest first, apart from note `id` itself, the notes it links to or
    /// from, those in the trash, locked or tagged `#ai-exclude`, those
    /// dismissed for it, and those below `opts.threshold`; each with the
    /// passage that matched.
    pub fn resurface(&self, text: &str, id: &str, opts: ResurfaceOpts) -> Result<Resurfacing> {
        let Some(model) = self.embedding_model()? else {
            return Ok(Resurfacing::Off);
        };
        let Some(embedder) = self.query_embedder(&model) else {
            return Ok(Resurfacing::NoModel);
        };
        let words = text.trim();
        if words.is_empty() || opts.k == 0 {
            return Ok(Resurfacing::Found(Vec::new()));
        }
        let Some(vector) = embedder.embed(&[words.to_owned()]).pop() else {
            return Ok(Resurfacing::Found(Vec::new()));
        };
        let near = self.index_nearest(&model, &vector, id, opts.threshold - INT8_MARGIN)?;
        let (found, gone) = self.read(|conn| {
            let mut skip = linked_notes(conn, id)?;
            skip.extend(dismissed_for(conn, id)?);
            let query = quantize(&vector);
            let mut gone: Vec<String> = Vec::new();
            // The candidates again in full precision, with what is shown.
            let mut scored: Vec<(f32, Resurfaced)> = Vec::new();
            for (other, _) in near {
                if skip.contains(&other) {
                    continue;
                }
                match candidate(conn, &other, &model)? {
                    None => gone.push(other),
                    Some((mut shown, exact)) => {
                        let score = crate::embed::cosine(&vector, &exact);
                        if score >= opts.threshold {
                            shown.score = score;
                            scored.push((score, shown));
                        }
                    }
                }
            }
            scored.sort_by(|a, b| b.0.total_cmp(&a.0).then_with(|| a.1.id.cmp(&b.1.id)));
            scored.truncate(PASSAGE_NOTES.max(opts.k));
            let mut found = Vec::new();
            for (_, mut shown) in scored.into_iter().take(opts.k) {
                let body = note_body(conn, &shown.id)?;
                let range = match best_passage(conn, &shown.id, &model, &query)? {
                    Some(range) => Some(range),
                    None => overlapping_passage(&body, words),
                };
                if let Some((start, end)) = range {
                    shown.passage = (start, end);
                    shown.snippet = snippet(&body, start, end);
                }
                found.push(shown);
            }
            Ok((found, gone))
        })?;
        if !gone.is_empty()
            && let Some(index) = self.index.lock().expect("index poisoned").as_mut()
        {
            for id in gone {
                index.remove(&id);
            }
        }
        Ok(Resurfacing::Found(found))
    }

    /// Up to `k` notes for "On this day": daily notes with the IDs in
    /// `daily_ids` (in that order, newest year first) and then notes
    /// created within `ranges` (millisecond ranges, `start` up to but not
    /// including `end`, newest first), without notes in the trash, locked
    /// or tagged `#ai-exclude`. Needs no embeddings.
    pub fn on_this_day(
        &self,
        ranges: &[(i64, i64)],
        daily_ids: &[String],
        k: usize,
    ) -> Result<Vec<DayNote>> {
        self.read(|conn| {
            let mut out: Vec<DayNote> = Vec::new();
            let mut seen: HashSet<String> = HashSet::new();
            let mut push = |note: DayNote, out: &mut Vec<DayNote>| {
                if out.len() < k && seen.insert(note.id.clone()) {
                    out.push(note);
                }
            };
            for id in daily_ids {
                if out.len() >= k {
                    break;
                }
                if let Some(note) = day_note(conn, id)? {
                    push(note, &mut out);
                }
            }
            let mut stmt = conn.prepare_cached(concat!(
                "SELECT d.id FROM docs d WHERE d.created_at >= ?1 AND d.created_at < ?2 AND ",
                crate::schema::discoverable!("d."),
                " ORDER BY d.created_at DESC, d.id"
            ))?;
            for (start, end) in ranges {
                if out.len() >= k {
                    break;
                }
                let ids: Vec<String> = stmt
                    .query_map(params![start, end], |row| row.get(0))?
                    .collect::<rusqlite::Result<_>>()?;
                for id in ids {
                    if let Some(note) = day_note(conn, &id)? {
                        push(note, &mut out);
                    }
                }
            }
            Ok(out)
        })
    }
}

fn dismissed_for(conn: &Connection, id: &str) -> Result<HashSet<String>> {
    let mut stmt =
        conn.prepare_cached("SELECT other_id FROM resurface_dismissed WHERE note_id = ?")?;
    Ok(stmt
        .query_map([id], |row| row.get(0))?
        .collect::<rusqlite::Result<_>>()?)
}

/// A note's words as the discovery features read them.
fn note_body(conn: &Connection, id: &str) -> Result<String> {
    Ok(conn
        .query_row(
            concat!(
                "SELECT ",
                crate::schema::readable_body!(""),
                " FROM docs WHERE id = ?"
            ),
            [id],
            |row| row.get::<_, Option<String>>(0),
        )
        .optional()?
        .flatten()
        .unwrap_or_default())
}

/// A note that may be shown, with its full vector, or `None` when it no
/// longer qualifies (deleted, in the trash, locked, excluded, or not
/// embedded).
fn candidate(conn: &Connection, id: &str, model: &str) -> Result<Option<(Resurfaced, Vec<f32>)>> {
    let row = conn
        .query_row(
            concat!(
                "SELECT d.title, d.created_at,
                    IFNULL((SELECT json_extract(b.json, '$.name') FROM docs b WHERE b.id = d.book_id), ''),
                    e.dim, e.vector
                 FROM docs d JOIN note_embeddings e ON e.note_id = d.id AND e.model = ?2
                 WHERE d.id = ?1 AND ",
                crate::schema::discoverable!("d.")
            ),
            params![id, model],
            |row| {
                Ok((
                    row.get::<_, Option<String>>(0)?,
                    row.get::<_, Option<i64>>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, Vec<u8>>(4)?,
                ))
            },
        )
        .optional()?;
    let Some((title, created_at, book, dim, bytes)) = row else {
        return Ok(None);
    };
    let Some(vector) = crate::embed::decode_vector(&bytes, dim as usize) else {
        return Ok(None);
    };
    Ok(Some((
        Resurfaced {
            id: id.to_owned(),
            title: title.unwrap_or_default(),
            book,
            created_at: created_at.unwrap_or(0),
            score: 0.,
            passage: (0, 0),
            snippet: String::new(),
        },
        vector,
    )))
}

fn day_note(conn: &Connection, id: &str) -> Result<Option<DayNote>> {
    Ok(conn
        .query_row(
            concat!(
                "SELECT d.title, d.created_at,
                    IFNULL((SELECT json_extract(b.json, '$.name') FROM docs b WHERE b.id = d.book_id), '')
                 FROM docs d WHERE d.id = ? AND ",
                crate::schema::discoverable!("d.")
            ),
            [id],
            |row| {
                Ok(DayNote {
                    id: id.to_owned(),
                    title: row.get::<_, Option<String>>(0)?.unwrap_or_default(),
                    created_at: row.get::<_, Option<i64>>(1)?.unwrap_or(0),
                    book: row.get(2)?,
                })
            },
        )
        .optional()?)
}

/// For tests: how many passages note `id` has by `model`.
#[cfg(test)]
pub(crate) fn passage_count(store: &Store, id: &str, model: &str) -> i64 {
    store
        .read(|conn| {
            Ok(conn.query_row(
                "SELECT COUNT(*) FROM note_passages WHERE note_id = ?1 AND model = ?2",
                params![id, model],
                |row| row.get(0),
            )?)
        })
        .unwrap()
}

#[cfg(test)]
mod tests;
