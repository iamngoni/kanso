//! Attachments in the app: the only way it reads a file's bytes, so a
//! locked note's pictures, drawings and recordings open while the note is
//! open and never otherwise (`kanso_engine::lock::files` has the sealing).
//!
//! A sealed file's bytes are a container only the file key opens. The file
//! keys of the notes this session has open are in [`FileKeys`]; Lock Notes
//! Now, quitting and the auto-lock forget them with the lock keys
//! ([`forget_caches`] also drops what was decoded from them).

use std::collections::{HashMap, HashSet};

use gpui_kit::App;
use kanso_core::Note;
use kanso_engine::lock::files::{self, Counts, FilesError, Job, KeyBook, Progress};
use kanso_store::Store;
use kanso_sync::lock::LockKey;

use crate::locked::LockKeys;

pub use kanso_engine::lock::files::Reading as Opened;

/// The file keys this session holds: a handle readers on any thread share,
/// emptied when the lock keys are.
#[derive(Clone, Default)]
pub struct FileKeys(KeyBook);

impl FileKeys {
    /// The session's file keys. Taken once and kept by whoever reads off the
    /// main thread (a preview, a player): it empties when the notes lock.
    pub fn of(cx: &mut App) -> FileKeys {
        FileKeys(LockKeys::file_keys(cx))
    }

    /// The session's file keys, for a reader that doesn't keep them: with
    /// no session yet, nothing is open.
    pub fn peek(cx: &App) -> FileKeys {
        FileKeys(LockKeys::peek_file_keys(cx))
    }

    /// How many keys are held.
    #[allow(dead_code, reason = "the helper surface the tests and later steps use")]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[allow(dead_code, reason = "the helper surface the tests and later steps use")]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub(crate) fn book(&self) -> &KeyBook {
        &self.0
    }

    /// Forgets every key.
    pub fn forget(&self) {
        self.0.clear();
    }
}

/// Reads attachment `id`: a plain file as stored, a sealed one decrypted
/// with the file key this session holds, else [`Opened::Locked`].
#[allow(dead_code, reason = "the helper surface the tests and later steps use")]
pub fn open(store: &Store, id: &str, cx: &App) -> Opened {
    open_with(store, id, &FileKeys::peek(cx))
}

/// [`open`] for a reader off the main thread, which holds the keys.
pub fn open_with(store: &Store, id: &str, keys: &FileKeys) -> Opened {
    files::read_file(store, id, keys.book())
}

/// [`open`], when the bytes are all that is wanted.
#[allow(dead_code, reason = "the helper surface the tests and later steps use")]
pub fn open_bytes(store: &Store, id: &str, cx: &App) -> Option<Vec<u8>> {
    match open(store, id, cx) {
        Opened::Opened(file) => Some(file.bytes),
        Opened::Locked | Opened::Missing => None,
    }
}

/// What sealing the pictures and files an open locked note links did.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Sealed {
    pub counts: Counts,
    /// The text to seal: the note's, with its links moved to the sealed
    /// copy of any file another note shows. The editor keeps its own text.
    pub body: String,
}

/// Seals whatever open locked `note`'s text links that isn't sealed yet (a
/// picture dropped on it, a recording, a file pasted from elsewhere), and
/// lists it on the note, so a save never writes a plain file for a locked
/// note. `key` is the note's own lock key. A note that needs nothing is left
/// as it is, without a write.
pub fn seal(
    store: &Store,
    note: &mut Note,
    key: &LockKey,
    keys: &FileKeys,
) -> Result<Option<Sealed>, FilesError> {
    if !needs_sealing(store, note) {
        return Ok(None);
    }
    let locking = HashSet::from([note.id.clone()]);
    let mut copies = HashMap::new();
    let mut watch = |_: &Progress| Ok(());
    let mut job = Job::new(store, keys.book(), &locking, &mut copies, &mut watch);
    let text = note.body.clone();
    let (body, counts) = files::seal_note_files(&mut job, note, &text, key)?;
    Ok(Some(Sealed { counts, body }))
}

/// Locks `note` (plain, with its text in `body`) with `key`, as Lock Note
/// does: seals its pictures and files, then the note, in one transaction
/// that checks every listed file is sealed. Runs off the main thread;
/// `watch` hears the progress and stops it between two files by returning an
/// error. The note returned is as the editor keeps it: open, its text its
/// own, its revision the new one.
pub fn lock_note(
    store: &Store,
    keys: &FileKeys,
    key: &LockKey,
    mut note: Note,
    summary: files::Summary,
    watch: &mut dyn FnMut(&Progress) -> Result<(), String>,
) -> Result<(Note, Counts), String> {
    let locking = HashSet::from([note.id.clone()]);
    let mut copies = HashMap::new();
    let mut job = Job::new(store, keys.book(), &locking, &mut copies, watch);
    job.expect(summary);
    let text = note.body.clone();
    let (sealed_text, counts) =
        files::seal_note_files(&mut job, &mut note, &text, key).map_err(|err| err.to_string())?;
    let mut stored = note.clone();
    stored.body.clear();
    let value = serde_json::to_value(&stored).map_err(|err| err.to_string())?;
    let written = store.lock_note(value).map_err(|err| err.to_string())?;
    note.rev = Some(written.rev);
    // The editor shows its own text; the sealed one links any copies.
    let _ = sealed_text;
    note.body = text;
    Ok((note, counts))
}

/// Takes the lock off open locked `note`: its files are decrypted or copied,
/// and the plain note written, with its `#tags` counting again when
/// `inline_tags`. Runs off the main thread. The note returned is the plain
/// one, its text moved to any copies.
pub fn remove_lock(
    store: &Store,
    key: &LockKey,
    mut note: Note,
    inline_tags: bool,
    now: i64,
    watch: &mut dyn FnMut(&Progress) -> Result<(), String>,
) -> Result<(Note, files::Unsealed), String> {
    let (body, done) =
        files::unseal_or_copy(store, &mut note, key, watch).map_err(|err| err.to_string())?;
    note.body = body;
    kanso_sync::lock::Envelope::store(None, &mut note.extra);
    note.updated_at = now;
    if inline_tags {
        kanso_engine::links::tags::apply_inline_tags(store, &mut note, None);
    }
    let value = serde_json::to_value(&note).map_err(|err| err.to_string())?;
    let written = store.unlock_note(value).map_err(|err| match err {
        kanso_store::StoreError::Conflict => {
            "The note changed elsewhere meanwhile. Try again.".to_owned()
        }
        other => other.to_string(),
    })?;
    note.rev = Some(written.rev);
    Ok((note, done))
}

/// Whether `note`'s text shows a file that is plain, or one its list
/// doesn't hold (a link pasted from another locked note).
fn needs_sealing(store: &Store, note: &Note) -> bool {
    files::needs_sealing(store, note)
}

/// A copy of file `id` for a note that must not share it: another locked
/// note's sealed one, or the original an unlocked note keeps.
#[allow(dead_code, reason = "the helper surface the tests and later steps use")]
pub fn copy_for_share(store: &Store, id: &str) -> Result<Option<String>, FilesError> {
    files::copy_for_share(store, id)
}

/// Takes the seal off open locked `note`'s files for Remove Lock: a file no
/// other locked note shows is decrypted in place, one that is shared gets a
/// plain copy for this note. Returns the text to save, its links moved to
/// any copies.
#[allow(dead_code, reason = "the helper surface the tests and later steps use")]
pub fn unseal_or_copy(
    store: &Store,
    note: &mut Note,
    key: &LockKey,
) -> Result<(String, files::Unsealed), FilesError> {
    files::unseal_or_copy(store, note, key, &mut |_| Ok(()))
}

/// Drops what was decoded from sealed files: the rendered blocks, drawings
/// read from attachments, and the file keys themselves. Lock Notes Now, the
/// auto-lock and a new password call it, with the lock keys gone, so no
/// decoded picture stays reachable from a window.
pub fn forget_caches(cx: &mut App) {
    FileKeys::peek(cx).forget();
    crate::preview_rich::forget_scenes();
    kanso_render::forget_all();
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use kanso_core::DocType;
    use kanso_engine::lock::files::OpenedFile;
    use kanso_store::{Origin, Put};
    use kanso_sync::KdfParams;
    use kanso_sync::lock::LockPassword;

    use super::*;

    fn file(store: &Store, bytes: &[u8]) -> String {
        use base64::Engine as _;
        store
            .put(
                DocType::File,
                json!({"name": "a.png", "contentType": "image/png", "_attachments": {"index": {
                    "content_type": "image/png",
                    "data": base64::engine::general_purpose::STANDARD.encode(bytes),
                }}}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id
    }

    #[test]
    fn a_plain_file_opens_and_a_sealed_one_needs_its_key() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("k.db")).unwrap();
        let book = store
            .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
            .unwrap()
            .id;
        let id = file(&store, b"picture");
        let note = store
            .put(
                DocType::Note,
                json!({"bookId": book, "title": "T", "body": format!("![](inkdrop://{id})")}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        let keys = FileKeys::default();
        assert_eq!(
            open_with(&store, &id, &keys),
            Opened::Opened(OpenedFile {
                name: "a.png".into(),
                content_type: "image/png".into(),
                bytes: b"picture".to_vec()
            })
        );
        assert_eq!(open_with(&store, "file:none", &keys), Opened::Missing);

        let key =
            LockKey::create(&LockPassword::new("a password").unwrap(), KdfParams::TEST).unwrap();
        let mut note = store.note(&note).unwrap().unwrap();
        let sealed = seal(&store, &mut note, &key, &keys);
        // The note is plain, not yet locked: sealing lists the file and
        // seals it, and the keys are held.
        let sealed = sealed.unwrap().unwrap();
        assert_eq!(sealed.counts.sealed, 1);
        assert_eq!(sealed.body, note.body);
        assert_eq!(keys.len(), 1);
        assert!(matches!(open_with(&store, &id, &keys), Opened::Opened(_)));
        keys.forget();
        assert_eq!(open_with(&store, &id, &keys), Opened::Locked);
        // Nothing left to seal.
        assert!(!needs_sealing(&store, &note));
    }
}
