//! A shared notebook's manifest (issue #390 › Shared notebooks: data): one
//! Yrs document per notebook, sealed with the notebook's manifest key, that
//! says what the notebook is and which notes are in it.
//!
//! ```text
//! root
//! ├─ book:  Y.Map<field, string>          name, and the notebook's other fields as JSON
//! └─ notes: Y.Map<note id, Y.Map>         created_by, created_at, deleted_at?, deleted_by?
//! ```
//!
//! Every field is its own map key, so two people changing different ones
//! both win. A note is deleted by setting `deleted_at` and brought back by
//! removing it; a member who edits a note that was deleted removes it, so the
//! edit wins, as in private sync.

use std::collections::BTreeMap;

use yrs::updates::decoder::Decode;
use yrs::{
    Any, Doc, Map as _, MapPrelim, MapRef, OffsetKind, Options, Out, ReadTxn, StateVector,
    Transact, Update,
};

use crate::doc::{DocError, client_id};

/// One note the manifest lists.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ManifestNote {
    /// The note's stream: 16 bytes as 22 base64url characters.
    pub id: String,
    pub created_by: String,
    pub created_at: i64,
    pub deleted_at: Option<i64>,
    /// The account that deleted it, when the deleting device said (older
    /// ones didn't): so that person can be told an edit brought it back.
    pub deleted_by: Option<String>,
}

pub struct ManifestDoc {
    doc: Doc,
    book: MapRef,
    notes: MapRef,
}

impl std::fmt::Debug for ManifestDoc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ManifestDoc")
            .field("notes", &self.notes().len())
            .finish()
    }
}

impl Default for ManifestDoc {
    fn default() -> ManifestDoc {
        ManifestDoc::new()
    }
}

fn number(value: Option<Out>) -> Option<i64> {
    match value? {
        Out::Any(Any::Number(n)) => n.as_i64(),
        _ => None,
    }
}

impl ManifestDoc {
    pub fn new() -> ManifestDoc {
        let doc = Doc::with_options(Options {
            client_id: client_id(),
            offset_kind: OffsetKind::Utf16,
            ..Options::default()
        });
        let book = doc.get_or_insert_map("book");
        let notes = doc.get_or_insert_map("notes");
        ManifestDoc { doc, book, notes }
    }

    pub fn load(state: &[u8]) -> Result<ManifestDoc, DocError> {
        let doc = ManifestDoc::new();
        doc.apply(state)?;
        Ok(doc)
    }

    pub fn encode_state(&self) -> Vec<u8> {
        self.doc
            .transact()
            .encode_state_as_update_v1(&StateVector::default())
    }

    /// Merges an update; whether the manifest changed.
    pub fn apply(&self, update: &[u8]) -> Result<bool, DocError> {
        let update = Update::decode_v1(update).map_err(|_| DocError::Malformed)?;
        let before = self.doc.transact().snapshot();
        {
            let mut txn = self.doc.transact_mut();
            txn.apply_update(update).map_err(|_| DocError::Malformed)?;
        }
        Ok(self.doc.transact().snapshot() != before)
    }

    /// The notebook's fields (`name`, and the others it carries).
    pub fn fields(&self) -> BTreeMap<String, String> {
        let txn = self.doc.transact();
        self.book
            .iter(&txn)
            .filter_map(|(key, value)| match value {
                Out::Any(Any::String(text)) => Some((key.to_owned(), text.to_string())),
                _ => None,
            })
            .collect()
    }

    /// Makes `fields` the notebook's fields, writing only what differs. The
    /// update to send, if anything changed.
    pub fn set_fields(&self, fields: &BTreeMap<String, String>) -> Option<Vec<u8>> {
        let now = self.fields();
        let mut txn = self.doc.transact_mut();
        let mut changed = false;
        for (key, value) in fields {
            if now.get(key) != Some(value) {
                self.book.insert(&mut txn, key.as_str(), value.as_str());
                changed = true;
            }
        }
        for key in now.keys().filter(|k| !fields.contains_key(*k)) {
            self.book.remove(&mut txn, key);
            changed = true;
        }
        changed.then(|| txn.encode_update_v1())
    }

    /// The notes listed, by id.
    pub fn notes(&self) -> Vec<ManifestNote> {
        let txn = self.doc.transact();
        let mut out: Vec<ManifestNote> = self
            .notes
            .iter(&txn)
            .filter_map(|(id, value)| {
                let Out::YMap(map) = value else { return None };
                let created_by = match map.get(&txn, "created_by") {
                    Some(Out::Any(Any::String(by))) => by.to_string(),
                    _ => return None,
                };
                Some(ManifestNote {
                    id: id.to_owned(),
                    created_by,
                    created_at: number(map.get(&txn, "created_at")).unwrap_or(0),
                    deleted_at: number(map.get(&txn, "deleted_at")),
                    deleted_by: match map.get(&txn, "deleted_by") {
                        Some(Out::Any(Any::String(by))) => Some(by.to_string()),
                        _ => None,
                    },
                })
            })
            .collect();
        out.sort_by(|a, b| a.id.cmp(&b.id));
        out
    }

    pub fn note(&self, id: &str) -> Option<ManifestNote> {
        self.notes().into_iter().find(|n| n.id == id)
    }

    /// Lists a note. The update to send; `None` when it is listed already.
    pub fn add_note(&self, id: &str, created_by: &str, created_at: i64) -> Option<Vec<u8>> {
        if self.note(id).is_some() {
            return None;
        }
        let mut txn = self.doc.transact_mut();
        let map = self.notes.insert(&mut txn, id, MapPrelim::default());
        map.insert(&mut txn, "created_by", created_by);
        map.insert(&mut txn, "created_at", created_at);
        Some(txn.encode_update_v1())
    }

    /// Lists several notes in one update (`(id, created_at)`); those listed
    /// already are left alone. `None` when there is nothing to add.
    pub fn add_notes(&self, notes: &[(String, i64)], created_by: &str) -> Option<Vec<u8>> {
        let known: std::collections::HashSet<String> =
            self.notes().into_iter().map(|n| n.id).collect();
        let mut txn = self.doc.transact_mut();
        let mut any = false;
        for (id, created_at) in notes.iter().filter(|(id, _)| !known.contains(id)) {
            let map = self
                .notes
                .insert(&mut txn, id.as_str(), MapPrelim::default());
            map.insert(&mut txn, "created_by", created_by);
            map.insert(&mut txn, "created_at", *created_at);
            any = true;
        }
        any.then(|| txn.encode_update_v1())
    }

    /// Marks a note deleted, or brings it back (`None`). The update to send;
    /// `None` when the note is unknown or already so.
    pub fn set_deleted(&self, id: &str, deleted_at: Option<i64>) -> Option<Vec<u8>> {
        self.set_deleted_by(id, deleted_at.map(|at| (at, None)))
    }

    /// [`Self::set_deleted`], saying who deleted it.
    pub fn mark_deleted(&self, id: &str, at: i64, by: &str) -> Option<Vec<u8>> {
        self.set_deleted_by(id, Some((at, Some(by))))
    }

    fn set_deleted_by(&self, id: &str, deleted: Option<(i64, Option<&str>)>) -> Option<Vec<u8>> {
        let deleted_at = deleted.map(|(at, _)| at);
        let now = self.note(id)?;
        if now.deleted_at.is_some() == deleted_at.is_some() {
            return None;
        }
        let mut txn = self.doc.transact_mut();
        let Some(Out::YMap(map)) = self.notes.get(&txn, id) else {
            return None;
        };
        match deleted {
            Some((at, by)) => {
                map.insert(&mut txn, "deleted_at", at);
                if let Some(by) = by {
                    map.insert(&mut txn, "deleted_by", by);
                }
            }
            None => {
                map.remove(&mut txn, "deleted_at");
                map.remove(&mut txn, "deleted_by");
            }
        }
        Some(txn.encode_update_v1())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn notes_deletion_and_fields_round_trip() {
        let a = ManifestDoc::new();
        let mut updates = Vec::new();
        updates.extend(a.set_fields(&BTreeMap::from([("name".into(), "Studio".into())])));
        updates.extend(a.add_note("n1", "a_1", 10));
        updates.extend(a.add_note("n2", "a_1", 11));
        assert!(a.add_note("n1", "a_1", 12).is_none());
        updates.extend(a.set_deleted("n2", Some(20)));
        assert!(a.set_deleted("n2", Some(21)).is_none());
        let b = ManifestDoc::new();
        for update in &updates {
            assert!(b.apply(update).unwrap());
        }
        assert_eq!(b.fields().get("name").map(String::as_str), Some("Studio"));
        assert_eq!(b.notes(), a.notes());
        assert_eq!(b.note("n2").unwrap().deleted_at, Some(20));
        assert_eq!(b.note("n2").unwrap().deleted_by, None);
        // An edit brings a deleted note back.
        let back = b.set_deleted("n2", None).unwrap();
        a.apply(&back).unwrap();
        assert_eq!(a.note("n2").unwrap().deleted_at, None);
        // Who deleted it goes with the deletion, and away with it.
        let marked = a.mark_deleted("n2", 30, "a_1").unwrap();
        b.apply(&marked).unwrap();
        assert_eq!(b.note("n2").unwrap().deleted_by.as_deref(), Some("a_1"));
        b.apply(&a.set_deleted("n2", None).unwrap()).unwrap();
        assert_eq!(b.note("n2").unwrap().deleted_by, None);
        let loaded = ManifestDoc::load(&a.encode_state()).unwrap();
        assert_eq!(loaded.notes(), a.notes());
        assert_eq!(ManifestDoc::new().apply(b"junk"), Err(DocError::Malformed));
    }

    proptest! {
        #![proptest_config(crate::tests::config(48))]

        /// Two members listing and deleting notes at once end with the same
        /// manifest (each person's updates arrive in the order they were made).
        #[test]
        fn prop_manifests_converge(ids in prop::collection::vec("[a-c]", 1..6), flips in prop::collection::vec(any::<bool>(), 6)) {
            let (a, b) = (ManifestDoc::new(), ManifestDoc::new());
            let (mut from_a, mut from_b) = (Vec::new(), Vec::new());
            for (i, id) in ids.iter().enumerate() {
                let (doc, out, who) = if flips[i % 6] { (&a, &mut from_a, "a_a") } else { (&b, &mut from_b, "a_b") };
                out.extend(doc.add_note(id, who, i as i64));
                if i % 2 == 1 {
                    out.extend(doc.set_deleted(id, Some(i as i64)));
                }
            }
            for update in &from_b { a.apply(update).unwrap(); }
            for update in &from_a { b.apply(update).unwrap(); }
            prop_assert_eq!(a.notes(), b.notes());
        }
    }
}
