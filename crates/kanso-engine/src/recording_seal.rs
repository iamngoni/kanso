//! A recording going into a locked note (issue #38, step 11; #44).
//!
//! The note's text is sealed, so adding the audio line means opening it with
//! the lock key the session holds, and the audio's own bytes are sealed
//! before they reach SQLite: each part gets a random file key, the part is
//! sealed under it (`kanso_sync::lock::seal_container`), and the key is
//! wrapped under the note's lock key and kept in the note's `kansoFiles`
//! list, exactly as #44 describes. Nothing here writes a plaintext part into
//! the library or a plaintext line into a locked note.
//!
//! **Minimum, pending #44 step 3.** The app's lock, unlock and rekey flow
//! (`feat/seal-lock-flow`) will bring a general helper that seals any
//! attachment of a locked note. This is the one thing recordings need of it:
//! seal a new part into a locked note that is open. It reads and writes the
//! same documents (`kansoFileLock`, `kansoFiles`), so the two reconcile by
//! one calling the other. When step 3 lands:
//!
//! - `SealedTarget::attach` calls its seal-an-attachment helper instead of
//!   sealing here, and `SealedTarget::write`/[`edit_text`] go through its
//!   locked-note save rather than `Store::lock_note` directly.
//! - The `NotSealable` refusal in [`open_target`] (no `kansoFiles`, links
//!   files) narrows to notes never unlocked since: step 3 seals those on
//!   unlock.
//! - The app's `locked::key_for_note`/`opened_parts` read file keys from
//!   step 3's session map (`FileKeys`) instead of unwrapping per call.
//! - Recording into a new note in a locked notebook can be allowed.
//!
//! Rekey needs nothing here: it re-wraps the `kansoFiles` keys this writes.
//!
//! A locked note that has no `kansoFiles` list was locked before sealing
//! existed and may link attachments that are still plain. It takes a
//! recording only when its text links no attachment at all, so the new list
//! is the whole truth; otherwise it is refused until step 3 seals what it
//! links.

use std::path::Path;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use kanso_core::DocType;
use kanso_core::model::{
    FileLock, FileMeta, KANSO_FILES_FIELD, Note, NoteFile, note_files_from_extra, store_note_files,
};
use kanso_core::tasks::linked_file_ids;
use kanso_store::{Attachment, Origin, Put, Store};
use kanso_sync::lock::{
    Envelope, FileKey, LockKey, SEGMENT_CONTENT, SEGMENT_METADATA, SealId, SealedFileMetadata,
    SealedSegment, seal_container, wrap_file_key,
};
use serde_json::json;

use crate::recording_flow::{AUDIO_TYPE, FlowError, part_file_id};

/// The note a recording goes into, read for editing: a plain note, or a
/// locked one opened with the session's key.
pub(crate) enum Target<'k> {
    Plain(Box<Note>),
    Sealed(Box<SealedTarget<'k>>),
}

/// A locked note, opened.
pub(crate) struct SealedTarget<'k> {
    note: Note,
    /// The note's text, opened.
    body: String,
    key: &'k LockKey,
    envelope: Envelope,
    /// The sealed files the note owns, including the ones just attached.
    files: Vec<NoteFile>,
}

/// Opens note `note_id` for a recording: a plain one as it is; a locked one
/// with `key`, which must open it. A note whose notebook is locked but which
/// holds no seal of its own is refused (the app seals it when it opens).
pub(crate) fn open_target<'k>(
    store: &Store,
    note_id: &str,
    key: Option<&'k LockKey>,
) -> Result<Target<'k>, FlowError> {
    let note = store.note(note_id)?.ok_or(FlowError::Missing)?;
    let in_locked_book = store.locked_books()?.contains(&note.book_id);
    let Some(envelope) = Envelope::from_extra(&note.extra) else {
        return if in_locked_book {
            Err(FlowError::Locked)
        } else {
            Ok(Target::Plain(Box::new(note)))
        };
    };
    let envelope = envelope.map_err(|_| FlowError::Locked)?;
    let key = key
        .filter(|key| key.fits(&envelope))
        .ok_or(FlowError::Locked)?;
    let body = key
        .open(&note.id, &envelope)
        .map_err(|_| FlowError::Locked)?
        .to_string();
    let files = match note_files_from_extra(&note.extra) {
        Some(files) => files,
        // No list: only a note that links nothing has nothing unsealed.
        None if note.extra.contains_key(KANSO_FILES_FIELD) => return Err(FlowError::NotSealable),
        None if linked_file_ids(&body).is_empty() => Vec::new(),
        None => return Err(FlowError::NotSealable),
    };
    Ok(Target::Sealed(Box::new(SealedTarget {
        note,
        body,
        key,
        envelope,
        files,
    })))
}

/// Rewrites the text of note `note_id` as `edit` returns it (`None` leaves
/// it as it is), as stored: a plain note as it is, a locked one opened with
/// `key` and sealed again, its list of sealed files untouched. A transcript
/// for a locked note that isn't open goes in this way, never through the
/// clipboard. Returns whether the note changed.
pub fn edit_text(
    store: &Store,
    note_id: &str,
    key: Option<&LockKey>,
    edit: impl FnOnce(&str) -> Option<String>,
    origin: Origin,
) -> Result<bool, FlowError> {
    let target = open_target(store, note_id, key)?;
    let Some(body) = edit(target.body()) else {
        return Ok(false);
    };
    target.rewrite(store, &body, &[], origin)?;
    Ok(true)
}

impl Target<'_> {
    pub(crate) fn id(&self) -> &str {
        match self {
            Target::Plain(note) => &note.id,
            Target::Sealed(sealed) => &sealed.note.id,
        }
    }

    /// The note's text, as the person sees it.
    pub(crate) fn body(&self) -> &str {
        match self {
            Target::Plain(note) => &note.body,
            Target::Sealed(sealed) => &sealed.body,
        }
    }

    /// Attaches part `index` of session `session_id` from `path`: as it is
    /// for a plain note, sealed for a locked one. Returns the file's ID.
    pub(crate) fn attach(
        &mut self,
        store: &Store,
        session_id: &str,
        index: usize,
        path: &Path,
        label: &str,
        origin: Origin,
    ) -> Result<String, FlowError> {
        match self {
            Target::Plain(_) => {
                crate::recording_flow::attach_part(store, session_id, index, path, label, origin)
            }
            Target::Sealed(sealed) => sealed.attach(store, session_id, index, path, label),
        }
    }

    /// Writes the note with its text replaced by `body`, and (for a locked
    /// note) no longer owning the files `released`.
    pub(crate) fn rewrite(
        self,
        store: &Store,
        body: &str,
        released: &[String],
        origin: Origin,
    ) -> Result<(), FlowError> {
        match self {
            Target::Plain(note) => {
                store.put(
                    DocType::Note,
                    json!({"_id": note.id, "_rev": note.rev, "body": body}),
                    Put::Merge,
                    origin,
                )?;
            }
            Target::Sealed(mut sealed) => {
                sealed.files.retain(|f| !released.contains(&f.file));
                sealed.write(store, body)?;
            }
        }
        Ok(())
    }

    /// Writes the note with `line` at its end.
    pub(crate) fn append(self, store: &Store, line: &str, origin: Origin) -> Result<(), FlowError> {
        let body = crate::recording_flow::append_to_body(self.body(), line);
        match self {
            Target::Plain(note) => {
                store.put(
                    DocType::Note,
                    json!({"_id": note.id, "_rev": note.rev, "body": body}),
                    Put::Merge,
                    origin,
                )?;
            }
            Target::Sealed(sealed) => sealed.write(store, &body)?,
        }
        Ok(())
    }
}

impl SealedTarget<'_> {
    fn attach(
        &mut self,
        store: &Store,
        session_id: &str,
        index: usize,
        path: &Path,
        label: &str,
    ) -> Result<String, FlowError> {
        let id = part_file_id(session_id, index);
        // Saved before, and the note still holds its key: nothing to do.
        if self.files.iter().any(|f| f.file == id) && store.get(&id)?.is_some() {
            return Ok(id);
        }
        let size = std::fs::metadata(path)
            .map_err(|e| FlowError::Read(e.to_string()))?
            .len();
        if size > kanso_core::model::MAX_ATTACHMENT_SIZE as u64 {
            return Err(FlowError::TooLarge);
        }
        let bytes = std::fs::read(path).map_err(|e| FlowError::Read(e.to_string()))?;
        let seal = SealId::random();
        let file_key = FileKey::random();
        let metadata = SealedFileMetadata {
            name: format!("{label} part {}.opus", index + 1),
            content_type: AUDIO_TYPE.to_owned(),
            public_in: Vec::new(),
        };
        let container = seal_container(
            &id,
            &seal,
            &file_key,
            &[
                SealedSegment {
                    kind: SEGMENT_CONTENT,
                    bytes,
                },
                SealedSegment {
                    kind: SEGMENT_METADATA,
                    bytes: metadata
                        .to_canonical_json()
                        .map_err(|e| FlowError::Read(e.to_string()))?,
                },
            ],
        );
        let wrapped = wrap_file_key(
            self.key,
            &self.envelope,
            &self.note.id,
            &id,
            &seal,
            &file_key,
        );
        // The stored name and type are the placeholders; the real ones are
        // in the container.
        let mut file = FileMeta {
            id: id.clone(),
            rev: store.get(&id)?.map(|doc| doc.rev),
            name: kanso_core::model::SEALED_FILE_NAME.to_owned(),
            content_type: kanso_core::model::SEALED_FILE_CONTENT_TYPE.to_owned(),
            content_length: 0,
            public_in: Vec::new(),
            md5digest: None,
            created_at: kanso_core::now_ms(),
            extra: Default::default(),
        };
        let mut lock = FileLock::sealed(seal.encode());
        lock.meta = true;
        FileLock::store(Some(&lock), &mut file.extra);
        store.put_sealed_file(file, Attachment::new(container))?;
        self.files.retain(|f| f.file != id);
        self.files.push(NoteFile {
            file: id.clone(),
            v: 1,
            seal: seal.encode(),
            key: STANDARD.encode(wrapped),
            extra: Default::default(),
        });
        Ok(id)
    }

    /// Seals `body` and the list of files into the note and stores it.
    fn write(self, store: &Store, body: &str) -> Result<(), FlowError> {
        let SealedTarget {
            mut note,
            key,
            files,
            ..
        } = self;
        Envelope::store(Some(&key.seal(&note.id, body)), &mut note.extra);
        store_note_files(Some(&files), &mut note.extra);
        note.body.clear();
        let value = serde_json::to_value(&note).map_err(|e| FlowError::Read(e.to_string()))?;
        store.lock_note(value)?;
        Ok(())
    }
}
