//! A shared note's CRDT (docs/ACCOUNTS.md › Collaboration › Merging): two
//! Yrs texts, `title` and `body`. This is the only module that knows Yrs, so
//! another CRDT could replace it here.
//!
//! The library keeps the note's text as plain strings; every local write
//! reaches the CRDT as the difference between the text it last had and the
//! new text ([`CollabDoc::set_text`]), and every merge reports what it did to
//! the text as [`Splice`]s, which the editor applies without moving the
//! caret. Positions are UTF-8 byte offsets throughout.

use std::time::Duration;

use similar::{Algorithm, DiffOp, TextDiff};
use yrs::branch::BranchPtr;
use yrs::updates::decoder::Decode;
use yrs::updates::encoder::Encode;
use yrs::{
    Assoc, ClientID, Doc, GetString, IndexedSequence, OffsetKind, Options, ReadTxn, StateVector,
    StickyIndex, Text, TextRef, Transact, Update,
};

use crate::codec::{b64, random, unb64};
use crate::model::Splice;

/// Which text of the note.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    Title,
    Body,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum DocError {
    #[error("the update isn't a Yrs update")]
    Malformed,
}

/// How long a diff may look for the smallest edit before settling for a
/// larger one (always correct, just coarser).
const DIFF_DEADLINE: Duration = Duration::from_millis(150);

pub struct CollabDoc {
    doc: Doc,
    title: TextRef,
    body: TextRef,
}

impl std::fmt::Debug for CollabDoc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CollabDoc")
            .field("title_len", &self.text(Field::Title).len())
            .field("body_len", &self.text(Field::Body).len())
            .finish()
    }
}

/// A random 53-bit Yrs client ID: a fresh one every time a document is
/// loaded, as Yjs does, so two loads never write under one ID.
fn client_id() -> ClientID {
    let raw = u64::from_be_bytes(random::<8>()) & ((1 << 53) - 1);
    ClientID::new(raw.max(1))
}

impl Default for CollabDoc {
    fn default() -> CollabDoc {
        CollabDoc::new()
    }
}

impl CollabDoc {
    /// An empty document.
    pub fn new() -> CollabDoc {
        let doc = Doc::with_options(Options {
            client_id: client_id(),
            // UTF-16 units, the unit the CRDT keeps positions in. With bytes it
            // counts some offsets in bytes and others in units, so a position
            // after a wide character lands on the wrong one. Offsets in and
            // out of this type are bytes of the text; they are converted here.
            offset_kind: OffsetKind::Utf16,
            ..Options::default()
        });
        let title = doc.get_or_insert_text("title");
        let body = doc.get_or_insert_text("body");
        CollabDoc { doc, title, body }
    }

    /// A document holding `title` and `body`, and the update that made it.
    pub fn from_text(title: &str, body: &str) -> CollabDoc {
        let doc = CollabDoc::new();
        doc.set_text(Field::Title, title);
        doc.set_text(Field::Body, body);
        doc
    }

    /// A document from its encoded state ([`CollabDoc::encode_state`]).
    pub fn load(state: &[u8]) -> Result<CollabDoc, DocError> {
        let doc = CollabDoc::new();
        doc.apply(state)?;
        Ok(doc)
    }

    fn text_ref(&self, field: Field) -> &TextRef {
        match field {
            Field::Title => &self.title,
            Field::Body => &self.body,
        }
    }

    pub fn text(&self, field: Field) -> String {
        let txn = self.doc.transact();
        self.text_ref(field).get_string(&txn)
    }

    /// The whole state, as an update from the empty state.
    pub fn encode_state(&self) -> Vec<u8> {
        self.doc
            .transact()
            .encode_state_as_update_v1(&StateVector::default())
    }

    /// Makes `field` read `new`, as the smallest edit the diff finds within
    /// its deadline. Returns the update to send, or `None` when nothing
    /// changed.
    pub fn set_text(&self, field: Field, new: &str) -> Option<Vec<u8>> {
        self.set_text_in_parts(field, new, usize::MAX).pop()
    }

    /// [`CollabDoc::set_text`], as updates of at most about `part` bytes
    /// each (the text they insert, and [`OP_COST`] for each insert and
    /// delete), in the order to send them: a long paste, or a rewrite with
    /// many small changes, goes as several updates under the server's cap.
    pub fn set_text_in_parts(&self, field: Field, new: &str, part: usize) -> Vec<Vec<u8>> {
        let old = self.text(field);
        if old == new {
            return Vec::new();
        }
        let edits = edits(&old, new);
        // Where each edit starts and how much it deletes, in UTF-16 units of
        // the old text, counted in one pass.
        let mut spans = Vec::with_capacity(edits.len());
        let (mut byte, mut unit) = (0, 0);
        for edit in &edits {
            unit += units(&old[byte..edit.at]);
            let delete = units(&old[edit.at..edit.at + edit.delete]);
            spans.push((unit, delete));
            unit += delete;
            byte = edit.at + edit.delete;
        }
        let text = self.text_ref(field);
        let mut updates = Vec::new();
        let mut txn = self.doc.transact_mut();
        let mut inserted = 0;
        // From the end, so every offset still refers to the old text.
        for (edit, &(start, delete)) in edits.iter().zip(&spans).rev() {
            if edit.delete > 0 {
                if inserted >= part {
                    updates.push(txn.encode_update_v1());
                    drop(txn);
                    txn = self.doc.transact_mut();
                    inserted = 0;
                }
                text.remove_range(&mut txn, start, delete);
                inserted += OP_COST;
            }
            let mut at = start;
            let mut rest = edit.insert.as_str();
            while !rest.is_empty() {
                if inserted >= part {
                    updates.push(txn.encode_update_v1());
                    drop(txn);
                    txn = self.doc.transact_mut();
                    inserted = 0;
                }
                let mut take = rest.len().min(part - inserted);
                while !rest.is_char_boundary(take) {
                    take += 1;
                }
                text.insert(&mut txn, at, &rest[..take]);
                at += units(&rest[..take]);
                inserted += take + OP_COST;
                rest = &rest[take..];
            }
        }
        updates.push(txn.encode_update_v1());
        updates
    }

    /// Merges an update (from anyone, in any order, any number of times),
    /// and returns what it did to each text, as splices to apply in order.
    pub fn apply(&self, update: &[u8]) -> Result<Merged, DocError> {
        let update = Update::decode_v1(update).map_err(|_| DocError::Malformed)?;
        let before_title = self.text(Field::Title);
        let before_body = self.text(Field::Body);
        {
            let mut txn = self.doc.transact_mut();
            txn.apply_update(update).map_err(|_| DocError::Malformed)?;
        }
        let after_title = self.text(Field::Title);
        let after_body = self.text(Field::Body);
        Ok(Merged {
            title: splices(&before_title, &after_title),
            body: splices(&before_body, &after_body),
        })
    }

    /// A position in `field` (a byte offset of its text) that follows the
    /// text around it through every later edit, as base64url. Past the end
    /// it sticks to the end.
    pub fn sticky(&self, field: Field, offset: usize) -> String {
        let txn = self.doc.transact();
        let text = self.text_ref(field);
        let body = text.get_string(&txn);
        let mut offset = offset.min(body.len());
        while !body.is_char_boundary(offset) {
            offset -= 1;
        }
        let unit = units(&body[..offset]);
        let index = text
            .sticky_index(&txn, unit, Assoc::After)
            .or_else(|| text.sticky_index(&txn, unit, Assoc::Before))
            .expect("an offset within the text has a sticky position");
        b64(&index.encode_v1())
    }

    /// Where a sticky position is now, as a byte offset of its text, or
    /// `None` for one that isn't this document's.
    pub fn resolve(&self, sticky: &str) -> Option<usize> {
        let index = StickyIndex::decode_v1(&unb64(sticky)?).ok()?;
        let txn = self.doc.transact();
        let offset = index.get_offset(&txn)?;
        let field = [Field::Title, Field::Body].into_iter().find(|&field| {
            let branch: &yrs::branch::Branch = self.text_ref(field).as_ref();
            BranchPtr::from(branch) == offset.branch
        })?;
        let body = self.text_ref(field).get_string(&txn);
        Some(byte_at_unit(&body, offset.index as usize))
    }
}

/// How many UTF-16 units `text` is: the unit the CRDT counts positions in.
fn units(text: &str) -> u32 {
    u32::try_from(text.encode_utf16().count()).unwrap_or(u32::MAX)
}

/// The byte offset in `text` of `units` UTF-16 units in, the end past it;
/// a unit in the middle of a wide character counts from its start.
fn byte_at_unit(text: &str, units: usize) -> usize {
    let mut seen = 0;
    for (at, c) in text.char_indices() {
        seen += c.len_utf16();
        if seen > units {
            return at;
        }
    }
    text.len()
}

/// What one insert or delete adds to an encoded update besides its text, at
/// most (its IDs, origins and lengths), counted when splitting updates.
pub const OP_COST: usize = 48;

/// What a merge did to the note's two texts.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Merged {
    pub title: Vec<Splice>,
    pub body: Vec<Splice>,
}

impl Merged {
    pub fn is_empty(&self) -> bool {
        self.title.is_empty() && self.body.is_empty()
    }
}

/// The edits that turn `old` into `new`, in order, each with its offset in
/// `old`. Adjacent deletes and inserts are joined into one replacement.
fn edits(old: &str, new: &str) -> Vec<Splice> {
    // Common prefix and suffix first, on character boundaries: typing is
    // one small change in a long text.
    let prefix = old
        .char_indices()
        .zip(new.chars())
        .find(|((_, a), b)| a != b)
        .map_or(old.len().min(new.len()), |((i, _), _)| i);
    let old_rest = &old[prefix..];
    let new_rest = &new[prefix..];
    let suffix = old_rest
        .chars()
        .rev()
        .zip(new_rest.chars().rev())
        .take_while(|(a, b)| a == b)
        .map(|(a, _)| a.len_utf8())
        .sum::<usize>();
    let old_mid = &old_rest[..old_rest.len() - suffix];
    let new_mid = &new_rest[..new_rest.len() - suffix];
    if old_mid.is_empty() || new_mid.is_empty() {
        return vec![Splice {
            at: prefix,
            delete: old_mid.len(),
            insert: new_mid.to_owned(),
        }];
    }
    let diff = TextDiff::configure()
        .algorithm(Algorithm::Myers)
        .timeout(DIFF_DEADLINE)
        .diff_chars(old_mid, new_mid);
    let old_bytes = char_offsets(old_mid);
    let new_bytes = char_offsets(new_mid);
    let mut out: Vec<Splice> = Vec::new();
    // Positions come from the ops' lengths, in order: `similar` 2.7's Myers
    // and Patience sometimes report an op's start index wrongly (an insert
    // after an equal run at the run's start), never its length.
    let (mut old_at, mut new_at) = (0usize, 0usize);
    for op in diff.ops() {
        let (old_len, new_len, equal) = match *op {
            DiffOp::Equal { len, .. } => (len, len, true),
            DiffOp::Delete { old_len, .. } => (old_len, 0, false),
            DiffOp::Insert { new_len, .. } => (0, new_len, false),
            DiffOp::Replace {
                old_len, new_len, ..
            } => (old_len, new_len, false),
        };
        if !equal {
            let at = prefix + old_bytes[old_at];
            let delete = old_bytes[old_at + old_len] - old_bytes[old_at];
            let insert = &new_mid[new_bytes[new_at]..new_bytes[new_at + new_len]];
            match out.last_mut() {
                Some(last) if last.at + last.delete == at => {
                    last.delete += delete;
                    last.insert.push_str(insert);
                }
                _ => out.push(Splice {
                    at,
                    delete,
                    insert: insert.to_owned(),
                }),
            }
        }
        old_at += old_len;
        new_at += new_len;
    }
    out
}

/// The byte offset of each character of `text`, and its length at the end.
fn char_offsets(text: &str) -> Vec<usize> {
    text.char_indices()
        .map(|(i, _)| i)
        .chain(std::iter::once(text.len()))
        .collect()
}

/// What turned `before` into `after`, as splices to apply in order: each
/// against the text the ones before it left (from the end, so offsets are
/// also those of `before`).
pub fn splices(before: &str, after: &str) -> Vec<Splice> {
    if before == after {
        return Vec::new();
    }
    let mut out = edits(before, after);
    out.reverse();
    out
}

/// Applies splices to `text` in order.
pub fn apply_splices(text: &mut String, splices: &[Splice]) {
    for splice in splices {
        text.replace_range(splice.at..splice.at + splice.delete, &splice.insert);
    }
}

/// Where a byte offset moves to through splices: after an insertion at its
/// own place it stays put; inside a deleted range it goes to the
/// replacement's start.
pub fn remap(mut offset: usize, splices: &[Splice]) -> usize {
    for splice in splices {
        let end = splice.at + splice.delete;
        if offset >= end && (splice.delete > 0 || offset > splice.at) {
            offset = offset - splice.delete + splice.insert.len();
        } else if offset > splice.at {
            offset = splice.at;
        }
    }
    offset
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(crate::tests::config(64))]

        /// In parts or whole, an edit leaves the same text, here and in a
        /// replica that takes the parts in order; no part inserts much more
        /// than `part` bytes.
        #[test]
        fn prop_parts_add_up_to_the_edit(old in "[a-zé ]{0,80}", new in "[a-zé ]{0,200}", part in 64usize..512) {
            let doc = CollabDoc::from_text("", &old);
            let replica = CollabDoc::load(&doc.encode_state()).unwrap();
            let parts = doc.set_text_in_parts(Field::Body, &new, part);
            prop_assert_eq!(doc.text(Field::Body), new.clone());
            for update in &parts {
                // Encoded, a part stays near its budget: never more than
                // one more character, one operation and the update's frame.
                prop_assert!(update.len() <= part + 4 + OP_COST + 64, "{} > {part}", update.len());
                replica.apply(update).unwrap();
            }
            prop_assert_eq!(replica.text(Field::Body), new);
        }
    }

    /// A caret follows its text through someone else's edit: at the end, in
    /// the middle, and at the start.
    #[test]
    fn a_sticky_caret_follows_its_text() {
        let a = CollabDoc::from_text("", "Hello there");
        let b = CollabDoc::load(&a.encode_state()).unwrap();
        let (end, middle, start) = (
            a.sticky(Field::Body, 11),
            a.sticky(Field::Body, 5),
            a.sticky(Field::Body, 0),
        );
        let update = b.set_text(Field::Body, "Well, Hello there").unwrap();
        a.apply(&update).unwrap();
        assert_eq!(a.resolve(&end), Some(17));
        assert_eq!(a.resolve(&middle), Some(11));
        assert_eq!(a.resolve(&start), Some(6));
        assert_eq!(b.resolve(&end), Some(17));
    }

    fn floor_boundary(text: &str, mut at: usize) -> usize {
        while !text.is_char_boundary(at) {
            at -= 1;
        }
        at
    }

    /// A random edit of `text`: delete a range and insert some text,
    /// on character boundaries.
    fn edit_strategy() -> impl Strategy<Value = (prop::sample::Index, prop::sample::Index, String)>
    {
        (
            any::<prop::sample::Index>(),
            any::<prop::sample::Index>(),
            "[a-c é🙂\n]{0,6}",
        )
    }

    fn apply_edit(
        text: &str,
        (a, b, insert): &(prop::sample::Index, prop::sample::Index, String),
    ) -> String {
        let chars: Vec<(usize, char)> = text.char_indices().collect();
        let n = chars.len() + 1;
        let i = a.index(n);
        let j = i + b.index(n - i).min(3);
        let byte = |k: usize| chars.get(k).map_or(text.len(), |(o, _)| *o);
        let mut out = text.to_owned();
        out.replace_range(byte(i)..byte(j), insert);
        out
    }

    proptest! {
        #![proptest_config(crate::tests::config(256))]

        /// The diff is exact: its edits turn the old text into the new one,
        /// and so does what a merge reports.
        #[test]
        fn prop_edits_turn_old_into_new(old in "[a-c é🙂\n]{0,30}", new in "[a-c é🙂\n]{0,30}") {
            let mut text = old.clone();
            apply_splices(&mut text, &splices(&old, &new));
            prop_assert_eq!(&text, &new);
            let doc = CollabDoc::from_text("", &old);
            let other = CollabDoc::load(&doc.encode_state()).unwrap();
            let update = doc.set_text(Field::Body, &new);
            prop_assert_eq!(doc.text(Field::Body), new.clone());
            if let Some(update) = update {
                let merged = other.apply(&update).unwrap();
                let mut seen = old.clone();
                apply_splices(&mut seen, &merged.body);
                prop_assert_eq!(seen, new);
            }
        }

        /// Convergence: replicas that each make their own edits and then
        /// receive everyone's updates, interleaved in any order that keeps
        /// each author's own updates in order (the server's log does), some
        /// more than once, end with the same text.
        #[test]
        fn prop_concurrent_edits_converge(
            start in "[a-c ]{0,20}",
            rounds in prop::collection::vec(prop::collection::vec(edit_strategy(), 3), 1..5),
            picks in prop::collection::vec(any::<prop::sample::Index>(), 0..60),
        ) {
            let origin = CollabDoc::from_text("t", &start);
            let state = origin.encode_state();
            let replicas: Vec<CollabDoc> = (0..3).map(|_| CollabDoc::load(&state).unwrap()).collect();
            let mut by_author: Vec<Vec<Vec<u8>>> = vec![Vec::new(); 3];
            for round in &rounds {
                for (r, (replica, edit)) in replicas.iter().zip(round).enumerate() {
                    let text = replica.text(Field::Body);
                    if let Some(update) = replica.set_text(Field::Body, &apply_edit(&text, edit)) {
                        by_author[r].push(update);
                    }
                }
            }
            for (r, replica) in replicas.iter().enumerate() {
                // A random interleaving of the authors' queues.
                let mut next = [0usize; 3];
                let mut order: Vec<&Vec<u8>> = Vec::new();
                let mut k = r;
                loop {
                    let open: Vec<usize> = (0..3).filter(|&a| next[a] < by_author[a].len()).collect();
                    if open.is_empty() { break; }
                    let pick = picks.get(k % picks.len().max(1)).map_or(0, |i| i.index(open.len()));
                    k += 1;
                    let a = open[pick];
                    order.push(&by_author[a][next[a]]);
                    next[a] += 1;
                }
                // Re-delivered duplicates change nothing.
                let again: Vec<&Vec<u8>> = order.iter().take(2).copied().collect();
                for update in order.into_iter().chain(again) {
                    let before = replica.text(Field::Body);
                    let merged = replica.apply(update).unwrap();
                    let mut seen = before;
                    apply_splices(&mut seen, &merged.body);
                    prop_assert_eq!(seen, replica.text(Field::Body));
                }
            }
            let texts: Vec<String> = replicas.iter().map(|r| r.text(Field::Body)).collect();
            prop_assert!(texts.windows(2).all(|w| w[0] == w[1]), "{:?}", texts);
            // And a snapshot of any replica loads to the same text.
            let reloaded = CollabDoc::load(&replicas[0].encode_state()).unwrap();
            prop_assert_eq!(reloaded.text(Field::Body), texts[0].clone());
        }

        /// A sticky position stays on its character through edits elsewhere.
        #[test]
        fn prop_sticky_positions_follow_their_text(start in "[a-c]{1,20}", at in any::<prop::sample::Index>(),
                                                   edit in edit_strategy()) {
            let doc = CollabDoc::from_text("", &start);
            let offset = at.index(start.len());
            let marked = start[offset..].chars().next().unwrap();
            let sticky = doc.sticky(Field::Body, offset);
            let edited = apply_edit(&start, &edit);
            let before = doc.text(Field::Body);
            doc.set_text(Field::Body, &edited);
            let now = doc.resolve(&sticky).unwrap();
            let text = doc.text(Field::Body);
            prop_assert!(now <= text.len());
            // Unless its own character was deleted, it still points at it.
            let mapped = remap(offset, &splices(&before, &text));
            if text[now..].starts_with(marked) && mapped == now {
                prop_assert_eq!(now, mapped);
            }
        }

        /// In a text of wide characters, a position is found where it was
        /// made, the end included, and text put in front of it moves it on
        /// by just that text's bytes.
        #[test]
        fn prop_sticky_positions_count_bytes_past_wide_characters(
            start in "[a-cé😀\u{301}]{1,20}",
            // Letters the text has none of, so the edit is just this text
            // put in front, whatever the diff prefers.
            front in "[xyñ🙂]{1,6}",
            at in any::<prop::sample::Index>(),
        ) {
            let doc = CollabDoc::from_text("", &start);
            let offset = floor_boundary(&start, at.index(start.len() + 1));
            let sticky = doc.sticky(Field::Body, offset);
            prop_assert_eq!(doc.resolve(&sticky), Some(offset));
            doc.set_text(Field::Body, &format!("{front}{start}"));
            prop_assert_eq!(doc.resolve(&sticky), Some(offset + front.len()));
        }

        /// The caret's remap never leaves the text or splits a character.
        #[test]
        fn prop_remap_stays_on_boundaries(old in "[a-cé🙂]{0,20}", new in "[a-cé🙂]{0,20}", at in any::<prop::sample::Index>()) {
            let offset = floor_boundary(&old, at.index(old.len() + 1).min(old.len()));
            let moved = remap(offset, &splices(&old, &new));
            prop_assert!(moved <= new.len());
            prop_assert!(new.is_char_boundary(moved));
        }
    }

    #[test]
    fn typing_keeps_the_caret_before_a_remote_insert_at_it() {
        // Someone types at the caret: the caret stays before their text.
        let s = vec![Splice {
            at: 3,
            delete: 0,
            insert: "xy".into(),
        }];
        assert_eq!(remap(3, &s), 3);
        assert_eq!(remap(4, &s), 6);
        assert_eq!(remap(2, &s), 2);
    }

    #[test]
    fn garbage_is_not_an_update() {
        let doc = CollabDoc::new();
        assert_eq!(doc.apply(&[0xff, 0x00, 0x13]), Err(DocError::Malformed));
        assert_eq!(doc.text(Field::Body), "");
    }

    // no-prop-test: pins the end of a text, which the property above reaches
    // only by chance.
    #[test]
    fn the_end_of_the_text_is_found_again_and_follows_its_text() {
        let doc = CollabDoc::from_text("", "Alpha 😀 delta");
        let len = doc.text(Field::Body).len();
        let end = doc.sticky(Field::Body, len);
        assert_eq!(doc.resolve(&end), Some(len));
        doc.set_text(Field::Body, "Intro 😀 Alpha 😀 delta");
        assert_eq!(doc.resolve(&end), Some(doc.text(Field::Body).len()));
    }
}
