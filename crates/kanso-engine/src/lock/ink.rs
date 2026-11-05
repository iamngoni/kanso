//! The ink attachment of an open locked note, read and written through the
//! session's file keys. `kanso_store::pdf_ink` owns the strokes and the
//! attachment link; this is its lock-aware byte store.

use std::sync::Arc;

use kanso_core::Note;
use kanso_store::pdf_ink::{
    Fetched, INK_FILE_CONTENT_TYPE, INK_FILE_NAME, INK_SEALED, InkFiles, Plain,
};
use kanso_store::{AttachmentState, Store};
use kanso_sync::lock::{Envelope, LockKey};

use super::files::{self, FilesError, KeyBook, Reading};

/// Reads and writes a note's ink attachment with the file keys this session
/// holds. A new attachment is sealed at once; an existing sealed one keeps
/// its seal and key, so its note's ownership wrap stays valid. Writing a
/// sealed attachment without its key refuses.
pub struct SealedInk {
    keys: KeyBook,
    writer: Option<Writer>,
    sealed_only: bool,
}

struct Writer {
    key: Arc<LockKey>,
    envelope: Envelope,
    note_id: String,
}

impl SealedInk {
    /// The adapter for reading only: a plain attachment opens, a sealed one
    /// opens only when its key is in `keys`, and a missing attachment is not
    /// created.
    pub fn reading(keys: KeyBook) -> Self {
        Self {
            keys,
            writer: None,
            sealed_only: true,
        }
    }

    /// The adapter for a plain note: reads use `keys`, and a new attachment
    /// is written in the plain form the store holds.
    pub fn plain(keys: KeyBook) -> Self {
        Self {
            keys,
            writer: None,
            sealed_only: false,
        }
    }

    /// The adapter for an open locked `note`: reads use `keys`, and a new
    /// attachment is sealed under the note's `key`.
    pub fn new(keys: KeyBook, note: &Note, key: Arc<LockKey>) -> Self {
        let writer = match Envelope::from_extra(&note.extra) {
            Some(Ok(envelope)) if key.fits(&envelope) => Some(Writer {
                key,
                envelope,
                note_id: note.id.clone(),
            }),
            _ => None,
        };
        Self {
            keys,
            writer,
            sealed_only: true,
        }
    }
}

impl InkFiles for SealedInk {
    fn fetch(&self, store: &Store, id: &str) -> Fetched {
        match files::read_file(store, id, &self.keys) {
            Reading::Opened(file) => Fetched::Bytes(file.bytes),
            Reading::Locked => Fetched::Sealed,
            Reading::Missing => Fetched::Missing,
        }
    }

    fn write(
        &self,
        store: &Store,
        existing: Option<&str>,
        fresh: &str,
        bytes: &[u8],
    ) -> Result<String, String> {
        let id = existing.unwrap_or(fresh);
        match store.attachment_state(id) {
            Ok(AttachmentState::Sealed) => files::rewrite_sealed_file(store, &self.keys, id, bytes)
                .map(|()| id.to_owned())
                .map_err(|error| words(error, "save", id)),
            Ok(AttachmentState::Plain) if !self.sealed_only => {
                Plain.write(store, existing, fresh, bytes)
            }
            Ok(AttachmentState::Plain) => {
                let Some(writer) = &self.writer else {
                    return Err(INK_SEALED.to_owned());
                };
                files::seal_new_file(
                    store,
                    &self.keys,
                    &writer.key,
                    &writer.envelope,
                    &writer.note_id,
                    id,
                    bytes.to_vec(),
                    INK_FILE_NAME,
                    INK_FILE_CONTENT_TYPE,
                )
                .map(|_| id.to_owned())
                .map_err(|error| words(error, "seal", id))
            }
            Ok(AttachmentState::Missing) => {
                let Some(writer) = &self.writer else {
                    if !self.sealed_only {
                        return Plain.write(store, existing, fresh, bytes);
                    }
                    return Err(INK_SEALED.to_owned());
                };
                files::seal_new_file(
                    store,
                    &self.keys,
                    &writer.key,
                    &writer.envelope,
                    &writer.note_id,
                    id,
                    bytes.to_vec(),
                    INK_FILE_NAME,
                    INK_FILE_CONTENT_TYPE,
                )
                .map(|entry| entry.file)
                .map_err(|error| words(error, "write", id))
            }
            Err(error) => Err(format!("Couldn't save the pencil marks: {error}")),
        }
    }
}

fn words(error: FilesError, action: &str, id: &str) -> String {
    match error {
        FilesError::NeedsUnlock { .. } => INK_SEALED.to_owned(),
        other => format!("Couldn't {action} the pencil marks ({id}): {other}"),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};
    use std::sync::Arc;

    use kanso_core::DocType;
    use kanso_core::Note;
    use kanso_core::model::note_files_from_extra;
    use kanso_store::pdf_ink::{Fetched, INK_SEALED, InkFiles, Plain};
    use kanso_store::{AttachmentState, Origin, Put, Store};
    use kanso_sync::KdfParams;
    use kanso_sync::lock::{LockKey, LockPassword};
    use serde_json::json;

    use super::*;
    use crate::lock::files::{Job, Progress, save_locked, seal_note_files, unseal_or_copy};

    fn world() -> (tempfile::TempDir, Store, Note, LockKey, KeyBook) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("k.db")).unwrap();
        let book = store
            .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
            .unwrap()
            .id;
        let mut note = store
            .put(
                DocType::Note,
                json!({"bookId": book, "title": "T", "body": ""}),
                Put::Upsert,
                Origin::Ui,
            )
            .map(|written| store.note(&written.id).unwrap().unwrap())
            .unwrap();
        let key =
            LockKey::create(&LockPassword::new("password").unwrap(), KdfParams::TEST).unwrap();
        let keys = KeyBook::default();
        let locking = HashSet::from([note.id.clone()]);
        let mut copies = HashMap::new();
        let mut watch = |_: &Progress| Ok(());
        let mut job = Job::new(&store, &keys, &locking, &mut copies, &mut watch);
        let (_, _) = seal_note_files(&mut job, &mut note, "", &key).unwrap();
        let mut stored = note.clone();
        stored.body.clear();
        let written = store
            .lock_note(serde_json::to_value(stored).unwrap())
            .unwrap();
        note.rev = Some(written.rev);
        (dir, store, note, key, keys)
    }

    #[test]
    fn ink_writes_sealed_and_refuses_without_the_key() {
        let (_dir, store, note, key, keys) = world();
        let adapter = SealedInk::new(keys.clone(), &note, Arc::new(key));
        let id = adapter
            .write(&store, None, "file:ink-test", b"first")
            .expect("the new ink is sealed");
        assert_eq!(
            store.attachment_state(&id).unwrap(),
            AttachmentState::Sealed
        );
        assert_eq!(
            adapter.fetch(&store, &id),
            Fetched::Bytes(b"first".to_vec())
        );

        let rewritten = adapter
            .write(&store, Some(&id), "file:ink-test", b"second")
            .expect("the sealed ink is rewritten under the same seal");
        assert_eq!(rewritten, id);
        assert_eq!(
            adapter.fetch(&store, &id),
            Fetched::Bytes(b"second".to_vec())
        );
        assert_eq!(
            Plain.fetch(&store, &id),
            Fetched::Sealed,
            "the plain reader never gets the sealed bytes"
        );
        assert_eq!(
            SealedInk::reading(KeyBook::default()).fetch(&store, &id),
            Fetched::Sealed,
            "a reader without the file key cannot open it"
        );
    }

    #[test]
    fn an_existing_plain_ink_attachment_is_sealed_before_rewrite() {
        let (_dir, store, note, key, keys) = world();
        let id = "file:ink-legacy-plain";
        Plain
            .write(&store, None, id, b"earlier strokes")
            .expect("the prior attachment is plain");
        let adapter = SealedInk::new(keys, &note, Arc::new(key));

        adapter
            .write(&store, Some(id), id, b"updated strokes")
            .expect("the open locked note seals its existing companion");

        assert_eq!(store.attachment_state(id).unwrap(), AttachmentState::Sealed);
        assert_eq!(store.attachment(id).unwrap(), None);
        assert_eq!(
            adapter.fetch(&store, id),
            Fetched::Bytes(b"updated strokes".to_vec())
        );
        assert_eq!(Plain.fetch(&store, id), Fetched::Sealed);
    }

    #[test]
    fn the_plain_adapter_creates_and_rewrites_a_plain_attachment() {
        let (_dir, store, _note, _key, keys) = world();
        let adapter = SealedInk::plain(keys);
        let id = adapter
            .write(&store, None, "file:ink-plain", b"first")
            .expect("a plain note writes its ink plain");
        assert_eq!(store.attachment_state(&id).unwrap(), AttachmentState::Plain);
        assert_eq!(
            adapter.fetch(&store, &id),
            Fetched::Bytes(b"first".to_vec())
        );
        assert_eq!(
            adapter
                .write(&store, Some(&id), "file:ink-plain", b"second")
                .unwrap(),
            id
        );
        assert_eq!(
            adapter.fetch(&store, &id),
            Fetched::Bytes(b"second".to_vec())
        );
    }

    #[test]
    fn a_sealed_adapter_without_the_key_creates_nothing() {
        let (_dir, store, _note, _key, keys) = world();
        let adapter = SealedInk::reading(keys);
        assert_eq!(
            adapter.write(&store, None, "file:ink-nokey", b"secret"),
            Err(INK_SEALED.to_owned())
        );
        assert_eq!(
            store.attachment_state("file:ink-nokey").unwrap(),
            AttachmentState::Missing
        );
    }

    #[test]
    fn a_new_sealed_ink_file_is_listed_on_the_next_locked_save() {
        let (_dir, store, mut note, key, keys) = world();
        let key = Arc::new(key);
        let adapter = SealedInk::new(keys.clone(), &note, key.clone());
        let id = adapter
            .write(&store, None, "file:ink-wrap", b"sealed ink")
            .expect("the new ink is sealed");
        let body = format!("inkdrop://{id}");
        save_locked(&store, &keys, &mut note, &body, &key).unwrap();
        let listed = note_files_from_extra(&note.extra).unwrap();
        assert!(listed.iter().any(|entry| entry.file == id));
        assert_eq!(store.files_of_note(&note.id).unwrap(), vec![id.clone()]);
        assert_eq!(
            store.attachment_state(&id).unwrap(),
            AttachmentState::Sealed
        );
    }

    #[test]
    fn a_note_with_companion_ink_locks_and_unlocks_with_its_attachment() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("k.db")).unwrap();
        let book = store
            .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
            .unwrap()
            .id;
        let mut note = store
            .put(
                DocType::Note,
                json!({"bookId": book, "title": "T", "body": ""}),
                Put::Upsert,
                Origin::Ui,
            )
            .map(|written| store.note(&written.id).unwrap().unwrap())
            .unwrap();
        let id = "file:ink-lock-transition";
        let bytes = b"recognizable stroke data";
        Plain
            .write(&store, None, id, bytes)
            .expect("the unlocked note writes its companion");
        note.body = format!("Ink: inkdrop://{id}");
        let key =
            LockKey::create(&LockPassword::new("password").unwrap(), KdfParams::TEST).unwrap();
        let keys = KeyBook::default();
        let locking = HashSet::from([note.id.clone()]);
        let mut copies = HashMap::new();
        let mut watch = |_: &Progress| Ok(());
        let mut job = Job::new(&store, &keys, &locking, &mut copies, &mut watch);
        let original_body = note.body.clone();
        let (sealed_body, _) = seal_note_files(&mut job, &mut note, &original_body, &key)
            .expect("a note with ink in a companion attachment can be locked");
        let mut stored_note = note.clone();
        stored_note.body.clear();
        let written = store
            .lock_note(serde_json::to_value(stored_note).unwrap())
            .unwrap();
        note.rev = Some(written.rev);
        note.body = sealed_body;
        assert_eq!(store.attachment_state(id).unwrap(), AttachmentState::Sealed);
        assert_eq!(store.attachment(id).unwrap(), None);
        let stored = serde_json::to_string(&store.get(&note.id).unwrap().unwrap().fields).unwrap();
        assert!(!stored.contains("recognizable stroke data"));
        assert_eq!(
            SealedInk::reading(KeyBook::default()).fetch(&store, id),
            Fetched::Sealed
        );

        let (body, _, unsealing) = unseal_or_copy(&store, &mut note, &key, &mut watch)
            .expect("unlocking opens the companion attachment");
        note.body = body;
        Envelope::store(None, &mut note.extra);
        store
            .unlock_note_unsealing(serde_json::to_value(&note).unwrap(), unsealing)
            .expect("unlocking writes the note and its companion together");
        assert_eq!(store.attachment_state(id).unwrap(), AttachmentState::Plain);
        assert_eq!(store.attachment(id).unwrap().unwrap().1.data, bytes);
    }
}
