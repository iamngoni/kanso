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
//!
//! A note's text is what the library keeps and the editor shows, front
//! matter included, so offsets the editor and the live room speak in are
//! offsets of [`CollabDoc::note_text`], while the CRDT's body starts after
//! the front matter. [`CollabDoc::note_sticky`] and
//! [`CollabDoc::note_resolve`] translate, and [`CollabDoc::edit_note_in_parts`]
//! takes a local edit of the note's text and writes each property it
//! changed as its own entry, so a concurrent change to another property,
//! or to the body, is never undone by it.

use std::collections::HashSet;

use kanso_core::properties;
use kanso_core::properties_model::FrontEntry;
use yrs::{Any, Map as _, Number, Out, Transact};

use crate::doc::{CollabDoc, Field};

/// The `caps` key that says the note's properties are entries.
const PARTS_KEY: &str = "parts";

/// The `caps` keys that say an account's app keeps properties as entries.
const ACK_PREFIX: &str = "ack:";

/// What an account signs to say its app keeps properties as entries in a
/// shared note: bound to the share and the account.
pub fn parts_ack_message(share_id: &str, account: &str) -> Vec<u8> {
    let mut message = b"kanso-parts-ack\x01".to_vec();
    message.extend_from_slice(share_id.as_bytes());
    message.push(0);
    message.extend_from_slice(account.as_bytes());
    message
}

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

/// A note's text as properties and body, where it has none (an empty list)
/// or can keep them key by key; `None` for front matter it can't keep so.
fn parts(text: &str) -> Option<(Vec<FrontEntry>, &str)> {
    if properties::split(text).is_none() {
        return Some((Vec::new(), text));
    }
    split_note(text)
}

/// Whether `text` opens like a front-matter block (a `---` line) that
/// doesn't read as front matter at all: half typed (a line without its
/// colon yet, a closing line not there yet). Front matter that reads but
/// can't be kept key by key is not this: it stays text, as the module
/// documentation says.
fn half_typed(text: &str) -> bool {
    if properties::split(text).is_some() {
        return false;
    }
    let rest = text.strip_prefix('\u{feff}').unwrap_or(text);
    rest.split_inclusive('\n').next().is_some_and(|first| {
        first.ends_with('\n') && first.trim_end_matches(['\n', '\r', ' ', '\t']) == "---"
    })
}

/// Puts `key` into `order` after the nearest earlier key of `before` that is
/// already there, or first when none is (at the end for `fallback_end`).
fn insert_after(order: &mut Vec<String>, key: &str, before: &[&str], fallback_end: bool) {
    let at = before
        .iter()
        .rev()
        .find_map(|k| order.iter().position(|o| o == k))
        .map(|i| i + 1);
    match (at, fallback_end) {
        (Some(i), _) => order.insert(i, key.to_owned()),
        (None, true) => order.push(key.to_owned()),
        (None, false) => order.insert(0, key.to_owned()),
    }
}

/// The properties a local edit leaves, over what the note has now: each key
/// the person changed or added takes their value, each key they removed
/// goes, and every key they left alone keeps whatever the note has for it
/// now (a change made meanwhile, or its removal, stands). Keys added
/// meanwhile stay. The order is the note's, unless the person reordered the
/// keys, then theirs.
fn merge_entries(base: &[FrontEntry], local: &[FrontEntry], now: &[FrontEntry]) -> Vec<FrontEntry> {
    let in_base = |key: &str| base.iter().find(|e| e.key == key);
    let in_now = |key: &str| now.iter().find(|e| e.key == key);
    let in_local = |key: &str| local.iter().any(|e| e.key == key);
    let mut values: Vec<FrontEntry> = Vec::new();
    for entry in local {
        if in_base(&entry.key).is_some_and(|b| b.rest == entry.rest) {
            if let Some(current) = in_now(&entry.key) {
                values.push(current.clone());
            }
        } else {
            values.push(entry.clone());
        }
    }
    for entry in now {
        if in_base(&entry.key).is_none() && !in_local(&entry.key) {
            values.push(entry.clone());
        }
    }
    let kept: HashSet<&str> = values.iter().map(|e| e.key.as_str()).collect();
    let common = |list: &[FrontEntry], other: &[FrontEntry]| -> Vec<String> {
        list.iter()
            .filter(|e| other.iter().any(|o| o.key == e.key))
            .map(|e| e.key.clone())
            .collect()
    };
    let reordered = common(base, local) != common(local, base);
    let now_keys: Vec<&str> = now.iter().map(|e| e.key.as_str()).collect();
    let local_keys: Vec<&str> = local.iter().map(|e| e.key.as_str()).collect();
    let mut order: Vec<String> = if reordered {
        local_keys
            .iter()
            .filter(|k| kept.contains(**k))
            .map(|k| (*k).to_owned())
            .collect()
    } else {
        now_keys
            .iter()
            .filter(|k| kept.contains(**k))
            .map(|k| (*k).to_owned())
            .collect()
    };
    let missing: Vec<&str> = values
        .iter()
        .map(|e| e.key.as_str())
        .filter(|k| !order.iter().any(|o| o == k))
        .collect();
    for key in missing {
        // A key the person has goes where their text has it; one only the
        // note has goes where the note has it.
        if in_local(key) {
            let at = local_keys.iter().position(|k| *k == key).unwrap_or(0);
            insert_after(&mut order, key, &local_keys[..at], reordered);
        } else {
            let at = now_keys.iter().position(|k| *k == key).unwrap_or(0);
            insert_after(&mut order, key, &now_keys[..at], false);
        }
    }
    order
        .iter()
        .filter_map(|key| values.iter().find(|e| e.key == *key).cloned())
        .collect()
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
        // Text that doesn't read as front matter yet doesn't clear the
        // properties it was meant to become.
        if half_typed(new) && !self.props().is_empty() {
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

    /// A document holding `title` and a note's text, its front matter as
    /// properties. Only for a document nobody with an older app edits (a
    /// shared notebook's note); a shared note starts with
    /// [`CollabDoc::from_text`] and moves over by
    /// [`CollabDoc::edit_note_in_parts`] once everyone who edits can read it.
    pub fn from_note_text(title: &str, text: &str) -> CollabDoc {
        let doc = CollabDoc::new();
        doc.set_text(Field::Title, title);
        doc.set_note_text_in_parts(text, usize::MAX);
        doc.mark_parts();
        doc
    }

    /// Whether the note keeps its front matter as properties (entries) and
    /// not as text in the body: it was moved over, or made so.
    ///
    /// Only the note's record of the move says so ([`CollabDoc::mark_parts`]),
    /// never the presence of a property: the move is decided once every app
    /// that edits has said it reads them. The record itself is a plain entry
    /// any editor could write, so it is best-effort against an editor
    /// forcing the move early (who could damage the note in any case); it
    /// can't make an editor appear to have said anything: see
    /// [`CollabDoc::ack_parts`].
    pub fn in_parts(&self) -> bool {
        let txn = self.doc().transact();
        self.caps.get(&txn, PARTS_KEY).is_some()
    }

    /// Records that the note keeps its properties as entries, from now on.
    /// Returns the update, if it wasn't recorded.
    pub fn mark_parts(&self) -> Option<Vec<u8>> {
        let mut txn = self.doc().transact_mut();
        if self.caps.get(&txn, PARTS_KEY).is_some() {
            return None;
        }
        self.caps.insert(&mut txn, PARTS_KEY, 1);
        Some(txn.encode_update_v1())
    }

    /// Says that `account`'s app keeps properties as entries: it can edit
    /// a note that does. An app that can't says nothing, which is how the
    /// others know. The word is `signature`, the account's own signature
    /// over [`parts_ack_message`] (its share and its account), so nobody can
    /// say it for another: [`CollabDoc::acked_parts`] counts only words that
    /// verify against the member's key. Returns the update, if `account`
    /// hadn't said it.
    pub fn ack_parts(&self, account: &str, signature: &[u8; 64]) -> Option<Vec<u8>> {
        let key = format!("{ACK_PREFIX}{account}");
        let mut txn = self.doc().transact_mut();
        if self.caps.get(&txn, &key).is_some() {
            return None;
        }
        self.caps
            .insert(&mut txn, key.as_str(), crate::codec::b64(signature));
        Some(txn.encode_update_v1())
    }

    /// Whether every one of `members` (account, sign key) has said their app
    /// keeps properties as entries in this note's share, by a signature that
    /// verifies ([`CollabDoc::ack_parts`]).
    pub fn acked_parts(&self, share_id: &str, members: &[(&str, [u8; 32])]) -> bool {
        let txn = self.doc().transact();
        members.iter().all(|(account, sign_pub)| {
            let Some(Out::Any(Any::String(word))) =
                self.caps.get(&txn, &format!("{ACK_PREFIX}{account}"))
            else {
                return false;
            };
            crate::codec::unb64_array::<64>(&word).is_some_and(|signature| {
                crate::identity::verify(sign_pub, &parts_ack_message(share_id, account), &signature)
            })
        })
    }

    /// How many bytes of [`CollabDoc::note_text`] come before the body.
    pub fn front_len(&self) -> usize {
        let entries = self.props();
        if entries.is_empty() {
            0
        } else {
            FENCE.len() * 2 + properties::render_front(&entries).len()
        }
    }

    /// A position in the note's text (a byte offset of
    /// [`CollabDoc::note_text`]) that follows the text around it through
    /// every later edit, as base64url. One in the front matter sticks to
    /// the start of the body.
    pub fn note_sticky(&self, offset: usize) -> String {
        self.sticky(Field::Body, offset.saturating_sub(self.front_len()))
    }

    /// Where a position made by [`CollabDoc::note_sticky`] is now, as a byte
    /// offset of the note's text; `None` for one that isn't of the body.
    pub fn note_resolve(&self, sticky: &str) -> Option<usize> {
        match self.resolve_field(sticky)? {
            (Field::Body, offset) => Some(offset + self.front_len()),
            (Field::Title, _) => None,
        }
    }

    /// Takes a local edit of the note's text, `base` to `local`, onto what
    /// the note says now, as the updates to send. Each property the edit
    /// changed is written on its own, so another person's change to a
    /// different property, or the same one made meanwhile when the edit left
    /// it alone, stands; the body is rebased onto what changed in it
    /// meanwhile. A note whose text isn't kept key by key (see the module
    /// documentation) is rebased as one text.
    ///
    /// A note that doesn't keep its properties as entries yet (one made by
    /// an app that didn't, see [`CollabDoc::in_parts`]) is moved over only
    /// when `migrate` says every app that edits it can read them that way
    /// ([`CollabDoc::acked_parts`]); until then its whole text, front
    /// matter included, is the body, as that app keeps it.
    pub fn edit_note_in_parts(
        &self,
        base: &str,
        local: &str,
        part: usize,
        migrate: bool,
    ) -> Vec<Vec<u8>> {
        self.try_edit_note_in_parts(base, local, part, migrate)
            .unwrap_or_default()
    }

    /// [`CollabDoc::edit_note_in_parts`], `None` when the edit is held back:
    /// the person is in the middle of typing in the front matter of a note
    /// that has properties (a line without its colon yet, a closing line not
    /// there yet), so it doesn't read as properties, and taking it would wipe
    /// them for everyone. Nothing is taken then, the caller keeps `base`
    /// where it is and leaves the library's text alone, and the whole edit,
    /// the body's included, is taken once the front matter reads again.
    pub fn try_edit_note_in_parts(
        &self,
        base: &str,
        local: &str,
        part: usize,
        migrate: bool,
    ) -> Option<Vec<Vec<u8>>> {
        if base == local {
            return Some(Vec::new());
        }
        let in_parts = self.in_parts();
        // (Properties without the record of the move are only ever from a
        // writer that wrote them directly: the note's text is still read
        // with them, so such a note is edited as one that has them.)
        if !in_parts && !migrate && self.props().is_empty() {
            let body = crate::engine::rebase(base, local, &self.body_text());
            return Some(self.set_text_in_parts(Field::Body, &body, part));
        }
        let now_props = self.props();
        let now_body = self.body_text();
        // A note that still has its front matter in the body (made before
        // properties had entries) moves it into them on this edit.
        let legacy = now_props.is_empty() && properties::split(&now_body).is_some();
        if !legacy
            && half_typed(local)
            && (!now_props.is_empty() || parts(base).is_some_and(|(props, _)| !props.is_empty()))
        {
            return None;
        }
        let mut updates: Vec<Vec<u8>> = Vec::new();
        if !in_parts {
            updates.extend(self.mark_parts());
        }
        updates.extend(self.edit_taken(base, local, part, legacy, now_props, now_body));
        Some(updates)
    }

    fn edit_taken(
        &self,
        base: &str,
        local: &str,
        part: usize,
        legacy: bool,
        now_props: Vec<FrontEntry>,
        now_body: String,
    ) -> Vec<Vec<u8>> {
        if !legacy
            && let (Some((base_props, base_body)), Some((props, body))) =
                (parts(base), parts(local))
        {
            let merged = merge_entries(&base_props, &props, &now_props);
            let mut updates: Vec<Vec<u8>> = self.write_props(&merged).into_iter().collect();
            let body = crate::engine::rebase(base_body, body, &now_body);
            updates.extend(self.set_text_in_parts(Field::Body, &body, part));
            return updates;
        }
        let text = crate::engine::rebase(base, local, &self.note_text());
        self.set_note_text_in_parts(&text, part)
    }
}
