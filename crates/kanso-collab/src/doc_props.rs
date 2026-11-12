//! A shared note's properties, one key at a time.
//!
//! The note's front matter lives in two Yrs maps beside the body text:
//! `props` (key to the entry's text after its colon) and `order` (key to a
//! position, so the front matter reads in the order it was written). Each
//! key is its own CRDT entry, so two people changing different properties
//! offline both keep their change, and two changing the same one end with
//! one of the two values (Yrs's order decides, identically on every
//! replica), never a mix of both. The body keeps only the text after the
//! front matter.
//!
//! [`CollabDoc::note_text`] renders front matter and body as the Markdown
//! note the library stores and search reads; [`CollabDoc::set_note_text_in_parts`]
//! takes the note's text back, writing only the keys that differ.
//!
//! A note whose front matter can't be kept this way without changing it (a
//! comment between entries, CRLF line endings, a key written twice, a byte
//! order mark, an empty block) keeps it in the body as text, as every note
//! did before this module: the same functions handle both, and an old
//! document, which has no `props`, opens as one.

use kanso_core::properties;
use kanso_core::properties_model::FrontEntry;
use yrs::{Any, Map as _, Number, Out, Transact};

use crate::doc::CollabDoc;

/// The line that opens and closes the front matter.
const FENCE: &str = "---\n";

/// Front matter and body as one note text.
fn render_note(entries: &[FrontEntry], body: &str) -> String {
    if entries.is_empty() {
        return body.to_owned();
    }
    format!("{FENCE}{}{FENCE}{body}", properties::render_front(entries))
}

/// A note's text as properties and the body after them, when its front
/// matter can be kept key by key without changing a byte of the note.
fn split_note(text: &str) -> Option<(Vec<FrontEntry>, &str)> {
    if text.starts_with('\u{feff}') {
        return None;
    }
    let front = properties::split(text)?;
    let entries = properties::entry_sources(front.front)?;
    let body = &text[front.body_start..];
    (!entries.is_empty() && render_note(&entries, body) == text).then_some((entries, body))
}

/// Positions for `keys` (in the order the note now has them) that keep the
/// ones already placed and put each new key between its neighbours.
fn positions(keys: &[&str], old: &dyn Fn(&str) -> Option<f64>) -> Vec<f64> {
    let placed: Vec<Option<f64>> = keys.iter().map(|key| old(key)).collect();
    let known: Vec<f64> = placed.iter().flatten().copied().collect();
    let by_index = || (0..keys.len()).map(|i| i as f64).collect::<Vec<_>>();
    if known.windows(2).any(|pair| pair[0] >= pair[1]) {
        return by_index();
    }
    let mut out = vec![0.0; keys.len()];
    let mut at = 0;
    while at < keys.len() {
        if let Some(position) = placed[at] {
            out[at] = position;
            at += 1;
            continue;
        }
        let end = (at..keys.len())
            .find(|&i| placed[i].is_some())
            .unwrap_or(keys.len());
        let low = at.checked_sub(1).map(|i| out[i]);
        let high = placed.get(end).copied().flatten();
        let count = (end - at) as f64;
        for (n, slot) in out[at..end].iter_mut().enumerate() {
            let n = n as f64 + 1.0;
            *slot = match (low, high) {
                (Some(low), Some(high)) => low + (high - low) * n / (count + 1.0),
                (Some(low), None) => low + n,
                (None, Some(high)) => high - count + n - 1.0,
                (None, None) => n - 1.0,
            };
        }
        at = end;
    }
    // Halving ran out of precision between two neighbours: start over.
    if out.windows(2).any(|pair| pair[0] >= pair[1]) {
        return by_index();
    }
    out
}

impl CollabDoc {
    /// The note's properties as entries, in the order they are written.
    pub fn props(&self) -> Vec<FrontEntry> {
        let txn = self.doc().transact();
        let mut rows: Vec<(f64, FrontEntry)> = Vec::new();
        for (key, value) in self.props.iter(&txn) {
            let Out::Any(Any::String(rest)) = value else {
                continue;
            };
            let position = match self.order.get(&txn, key) {
                Some(Out::Any(Any::Number(n))) => n.as_f64().unwrap_or(f64::MAX),
                _ => f64::MAX,
            };
            rows.push((
                position,
                FrontEntry {
                    key: key.to_owned(),
                    rest: rest.to_string(),
                },
            ));
        }
        rows.sort_by(|a, b| a.0.total_cmp(&b.0).then_with(|| a.1.key.cmp(&b.1.key)));
        rows.into_iter().map(|(_, entry)| entry).collect()
    }

    /// The whole note as Markdown: the properties as front matter, then the
    /// body. This is the text the library keeps and search reads.
    pub fn note_text(&self) -> String {
        render_note(&self.props(), &self.body_text())
    }

    /// Makes the note read `new`, writing only what differs: each changed,
    /// added or removed property on its own, and the body as the smallest
    /// edit. Returns the updates to send, properties first, in parts of at
    /// most about `part` bytes (see [`CollabDoc::set_text_in_parts`]).
    pub fn set_note_text_in_parts(&self, new: &str, part: usize) -> Vec<Vec<u8>> {
        if self.note_text() == new {
            return Vec::new();
        }
        let (entries, body) = split_note(new).unwrap_or_else(|| (Vec::new(), new));
        let mut updates = Vec::new();
        if let Some(update) = self.write_props(&entries) {
            updates.push(update);
        }
        updates.extend(self.set_text_in_parts(crate::doc::Field::Body, body, part));
        updates
    }

    /// Makes the properties `entries`; the update, if anything changed.
    fn write_props(&self, entries: &[FrontEntry]) -> Option<Vec<u8>> {
        let now = self.props();
        let keys: Vec<&str> = entries.iter().map(|entry| entry.key.as_str()).collect();
        let (old_positions, new_positions) = {
            let txn = self.doc().transact();
            let old = |key: &str| match self.order.get(&txn, key) {
                Some(Out::Any(Any::Number(n))) => n.as_f64(),
                _ => None,
            };
            let new = positions(&keys, &old);
            let old: Vec<Option<f64>> = keys.iter().map(|key| old(key)).collect();
            (old, new)
        };
        let mut changed = false;
        let mut txn = self.doc().transact_mut();
        for entry in &now {
            if !keys.contains(&entry.key.as_str()) {
                self.props.remove(&mut txn, &entry.key);
                self.order.remove(&mut txn, &entry.key);
                changed = true;
            }
        }
        for (i, entry) in entries.iter().enumerate() {
            if !now.iter().any(|seen| seen == entry) {
                self.props
                    .insert(&mut txn, entry.key.as_str(), entry.rest.as_str());
                changed = true;
            }
            if old_positions[i] != Some(new_positions[i]) {
                self.order.insert(
                    &mut txn,
                    entry.key.as_str(),
                    Any::Number(Number::from(new_positions[i])),
                );
                changed = true;
            }
        }
        changed.then(|| txn.encode_update_v1())
    }
}
