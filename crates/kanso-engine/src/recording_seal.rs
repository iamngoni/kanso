//! A recording going into a locked note (issue #38, step 11; #44).
//!
//! The note's text is sealed, so adding the audio line means opening it with
//! the lock key the session holds, and the audio's own bytes are sealed
//! before they reach SQLite. Both go through #44 step 3's helpers in
//! [`crate::lock::files`], so a recording is sealed exactly as any other
//! attachment of a locked note:
//!
//! - each part is sealed by [`files::seal_new_file`] (a random file key,
//!   the container checked to open before it is written, the key wrapped
//!   under the note's lock key into the note's `kansoFiles`);
//! - the note is written by [`files::save_locked`], the locked-note save,
//!   which also seals whatever else the text shows that isn't sealed yet.
//!   A note locked before sealing existed (no list) is sealed that way on
//!   its first recording, so it no longer refuses one.
//! - A new note in a locked notebook takes a recording when the key opens
//!   the notebook: it is made sealed, never with plain text.
//!
//! Nothing here writes a plaintext part into the library or a plaintext
//! line into a locked note. Rekey needs nothing here: it re-wraps the
//! `kansoFiles` keys this writes.

use std::path::Path;

use kanso_core::DocType;
use kanso_core::model::{
    KANSO_FILES_FIELD, Note, NoteFile, note_files_from_extra, store_note_files,
};
use kanso_store::{Origin, Put, Store};
use kanso_sync::lock::{Envelope, LockKey};
use serde_json::json;

use crate::lock::files::{self, KeyBook};
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
    /// The file keys of the parts sealed here.
    keys: KeyBook,
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
        // A damaged list can't say what the note holds.
        None if note.extra.contains_key(KANSO_FILES_FIELD) => return Err(FlowError::NotSealable),
        // No list: locked before sealing existed. The save seals what the
        // text shows ([`files::save_locked`]).
        None => Vec::new(),
    };
    Ok(Target::Sealed(Box::new(SealedTarget {
        note,
        body,
        key,
        envelope,
        files,
        keys: KeyBook::default(),
    })))
}

/// A new note `id` titled `title` in locked notebook `book`, made sealed
/// under `key` (which must open the notebook) and opened for a recording.
/// `None` when the notebook isn't locked. Made empty first: a note in a
/// locked notebook never holds plain text.
pub(crate) fn new_in_locked_book<'k>(
    store: &Store,
    id: &str,
    title: &str,
    book: &str,
    key: Option<&'k LockKey>,
) -> Result<Option<Target<'k>>, FlowError> {
    let Some(lock) = crate::lock::Target::of_book(book, store) else {
        return Ok(None);
    };
    let envelope = lock.envelope.map_err(|_| FlowError::Locked)?;
    let key = key
        .filter(|key| key.fits(&envelope))
        .ok_or(FlowError::Locked)?;
    let note = match store.note(id)? {
        // Saved before: open what is there.
        Some(note) if note.is_locked() => return open_target(store, id, Some(key)).map(Some),
        // Made, and a crash came before it was sealed: it is still empty.
        Some(note) => note,
        None => {
            let made = store.put(
                DocType::Note,
                json!({"_id": id, "title": title, "body": "", "bookId": book}),
                Put::Upsert,
                Origin::Ui,
            )?;
            store.note(&made.id)?.ok_or(FlowError::Missing)?
        }
    };
    Ok(Some(Target::Sealed(Box::new(SealedTarget {
        envelope: key.seal(&note.id, ""),
        note,
        body: String::new(),
        key,
        files: Vec::new(),
        keys: KeyBook::default(),
    }))))
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
        let entry = files::seal_new_file(
            store,
            &self.keys,
            self.key,
            &self.envelope,
            &self.note.id,
            &id,
            bytes,
            &format!("{label} part {}.opus", index + 1),
            AUDIO_TYPE,
        )
        .map_err(|err| FlowError::Read(err.to_string()))?;
        self.files.retain(|f| f.file != id);
        self.files.push(entry);
        Ok(id)
    }

    /// Seals `body` and the list of files into the note and stores it, the
    /// way the editor saves a locked note ([`files::save_locked`]).
    fn write(self, store: &Store, body: &str) -> Result<(), FlowError> {
        let SealedTarget {
            mut note,
            key,
            envelope,
            files: listed,
            keys,
            ..
        } = self;
        Envelope::store(Some(&envelope), &mut note.extra);
        store_note_files(Some(&listed), &mut note.extra);
        files::save_locked(store, &keys, &mut note, body, key).map_err(|err| match err {
            files::FilesError::NeedsUnlock { .. } => FlowError::NotSealable,
            other => FlowError::Read(other.to_string()),
        })?;
        Ok(())
    }
}
