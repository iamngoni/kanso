//! Sealing the attachments of notes locked before attachments were sealed
//! (#44, step 7).
//!
//! Nothing can seal those files without the lock password: the note's text,
//! so the list of what it links, is sealed. With it, [`migrate`] opens each
//! such note, seals the files it links the way a new lock does (a file an
//! unlocked note also shows is copied first, so that note keeps the plain
//! original), and writes the note's list.
//!
//! It uses the lock's own journal ([`super::seal_note_files`]): a note's
//! wraps are written before any of its files is sealed, and a plain file is
//! only replaced by a container that was opened again and found to hold the
//! same bytes. A run that stopped (a crash, Cancel, a full disk) leaves the
//! journal on the note, and the next run finishes it; [`undo_stopped`] takes
//! it back instead. Running it twice seals once: a note that has its list is
//! done, and a sealed file is never sealed again.
//!
//! Three kinds of note are covered:
//!
//! * a locked note with no list (locked by an older Kanso),
//! * a note in a locked notebook whose text isn't sealed yet (put there by
//!   an older Kanso or another device), and
//! * a locked note a stopped run left a journal on.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use kanso_core::DocType;
use kanso_core::Note;
use kanso_core::model::{KANSO_FILES_FIELD, NoteFile, note_files_from_extra, store_note_files};
use kanso_store::{AttachmentState, Origin, Put, Store};
use kanso_sync::lock::{Envelope, LockKey};

use super::{
    FilesError, JOURNAL_FROM, JOURNAL_LOCK, JOURNAL_NEW, Job, KeyBook, Progress, Summary, failed,
    header, seal_note_files, summarize, unseal_in_place, unwrap_entry,
};
use crate::lock::{Target, display_title};

/// What is left to migrate, from the store alone (no password).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Pending {
    /// Locked notes with no attachment list.
    pub legacy: usize,
    /// Notes in locked notebooks whose text isn't sealed yet.
    pub unsealed: usize,
    /// Locked notes a stopped run left a journal on.
    pub stopped: usize,
}

impl Pending {
    /// Notes that need the password.
    pub fn notes(&self) -> usize {
        self.legacy + self.unsealed + self.stopped
    }

    /// Nothing is left.
    pub fn is_done(&self) -> bool {
        self.notes() == 0
    }
}

/// Whether the list of locked `note` still carries a stopped run's journal.
pub fn is_stopped(note: &Note) -> bool {
    note.is_locked()
        && note_files_from_extra(&note.extra).is_some_and(|entries| {
            entries
                .iter()
                .any(|entry| entry.extra.contains_key(JOURNAL_LOCK))
        })
}

/// Whether locked `note` has no attachment list: an older Kanso locked it.
pub fn is_legacy(note: &Note) -> bool {
    note.is_locked() && !note.extra.contains_key(KANSO_FILES_FIELD)
}

/// How many notes wait for the migration. Cheap: it opens nothing.
pub fn pending(store: &Store) -> Pending {
    let mut pending = Pending::default();
    for note in store.locked_notes().unwrap_or_default() {
        if is_stopped(&note) {
            pending.stopped += 1;
        } else if is_legacy(&note) {
            pending.legacy += 1;
        }
    }
    pending.unsealed = store
        .unsealed_in_locked_books()
        .map_or(0, |notes| notes.len());
    pending
}

/// A note to migrate, opened.
struct Item {
    /// With its text in `body`.
    note: Note,
    key: Arc<LockKey>,
}

struct Gathered {
    items: Vec<Item>,
    /// Notes none of the keys opens.
    unopened: usize,
}

/// Opens every note that needs the migration and a key in `keys` fits.
fn gather(store: &Store, keys: &[Arc<LockKey>]) -> Gathered {
    let mut gathered = Gathered {
        items: Vec::new(),
        unopened: 0,
    };
    for mut note in store.locked_notes().unwrap_or_default() {
        if !(is_legacy(&note) || is_stopped(&note)) {
            continue;
        }
        let opened = Envelope::from_extra(&note.extra)
            .and_then(Result::ok)
            .and_then(|envelope| {
                let key = keys.iter().find(|key| key.fits(&envelope))?;
                let text = key.open(&note.id, &envelope).ok()?;
                Some((Arc::clone(key), text.to_string()))
            });
        match opened {
            Some((key, text)) => {
                note.body = text;
                gathered.items.push(Item { note, key });
            }
            None => gathered.unopened += 1,
        }
    }
    for note in store.unsealed_in_locked_books().unwrap_or_default() {
        let key = Target::of_book(&note.book_id, store)
            .and_then(|target| target.envelope.ok())
            .and_then(|envelope| keys.iter().find(|key| key.fits(&envelope)).cloned());
        match key {
            Some(key) => gathered.items.push(Item { note, key }),
            None => gathered.unopened += 1,
        }
    }
    gathered
}

/// What a migration would do with the keys held.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Scan {
    /// Notes the keys open.
    pub notes: usize,
    /// Notes they don't open.
    pub unopened: usize,
    /// The plain files those notes link, their size, and how many an
    /// unlocked note also shows.
    pub summary: Summary,
}

/// What [`migrate`] would seal with `keys`.
pub fn scan(store: &Store, keys: &[Arc<LockKey>]) -> Scan {
    let gathered = gather(store, keys);
    let notes: Vec<Note> = gathered
        .items
        .iter()
        .map(|item| item.note.clone())
        .collect();
    Scan {
        notes: notes.len(),
        unopened: gathered.unopened,
        summary: summarize(store, &notes),
    }
}

/// A note that couldn't be migrated, in words.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Failure {
    pub title: String,
    pub words: String,
}

/// What a migration did.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Outcome {
    /// Notes whose attachments are sealed and listed now.
    pub notes: usize,
    /// Files sealed in place.
    pub sealed: usize,
    /// Sealed copies made of files an unlocked note also shows.
    pub copied: usize,
    /// Files already sealed by another note, which this one now holds a key
    /// to as well.
    pub shared: usize,
    /// Notes the keys didn't open: another password locks them.
    pub unopened: usize,
    /// Notes that failed, with why; they keep what they had and are tried
    /// again next time.
    pub failed: Vec<Failure>,
    /// The caller stopped it between two files.
    pub stopped: bool,
}

impl Outcome {
    /// Files made sealed: in place or by copy.
    pub fn files(&self) -> usize {
        self.sealed + self.copied
    }
}

/// Seals the attachments of every note that needs it and a key in `keys`
/// opens, one note at a time. `book` holds the file keys made (the session's,
/// so the notes open at once). `watch` hears the progress and stops the run
/// before the next file by returning an error; what was done stays done and
/// the rest is finished by the next run.
pub fn migrate(
    store: &Store,
    book: &KeyBook,
    keys: &[Arc<LockKey>],
    watch: &mut dyn FnMut(&Progress) -> Result<(), String>,
) -> Outcome {
    // The notes done before hold files others may link too: their keys are
    // needed to give those notes a wrap of their own.
    for note in store.locked_notes().unwrap_or_default() {
        if let Some(Ok(envelope)) = Envelope::from_extra(&note.extra)
            && let Some(key) = keys.iter().find(|key| key.fits(&envelope))
        {
            book.remember_note(&note, key);
        }
    }
    let gathered = gather(store, keys);
    let mut outcome = Outcome {
        unopened: gathered.unopened,
        ..Outcome::default()
    };
    let opened: Vec<Note> = gathered
        .items
        .iter()
        .map(|item| item.note.clone())
        .collect();
    let summary = summarize(store, &opened);
    let locking: HashSet<String> = opened.iter().map(|note| note.id.clone()).collect();
    let mut copies = HashMap::new();
    let mut job = Job::new(store, book, &locking, &mut copies, watch);
    job.expect(summary);
    for item in gathered.items {
        let Item { mut note, key } = item;
        let title = display_title(&note);
        let text = note.body.clone();
        match migrate_note(&mut job, &mut note, &text, &key) {
            Ok(counts) => {
                outcome.notes += 1;
                outcome.sealed += counts.sealed;
                outcome.copied += counts.copied;
                outcome.shared += counts.shared;
            }
            Err(FilesError::Stopped(_)) => {
                outcome.stopped = true;
                break;
            }
            Err(err) => outcome.failed.push(Failure {
                title,
                words: err.to_string(),
            }),
        }
    }
    outcome
}

/// Seals one note's files and writes the note with its list.
fn migrate_note(
    job: &mut Job,
    note: &mut Note,
    text: &str,
    key: &LockKey,
) -> Result<super::Counts, FilesError> {
    let store = job.store;
    let (_, counts) = seal_note_files(job, note, text, key)?;
    let mut stored = note.clone();
    stored.body.clear();
    let value = serde_json::to_value(&stored).map_err(|err| failed(err.to_string()))?;
    store.lock_note(value)?;
    Ok(counts)
}

/// Takes back what a stopped run did to locked `note`: the files it sealed
/// are unsealed, the copies it made removed, and the note goes back to
/// having no list (or the list it had before, if it had files then). `key`
/// must be the lock key the run used. Returns how many files it undid.
pub fn undo_stopped(store: &Store, note: &Note, key: &LockKey) -> Result<usize, FilesError> {
    let entries = note_files_from_extra(&note.extra).unwrap_or_default();
    let mut undone = 0;
    let mut kept: Vec<NoteFile> = Vec::new();
    for entry in &entries {
        let Some(envelope) = header(entry) else {
            kept.push(entry.clone());
            continue;
        };
        let Some((seal, file_key)) = key
            .fits(&envelope)
            .then(|| unwrap_entry(key, &envelope, &note.id, entry))
            .flatten()
        else {
            return Err(failed("That isn't the lock password."));
        };
        if !entry.extra.contains_key(JOURNAL_NEW) {
            // A file this run only gained a key to: the list keeps it.
            let mut plain = entry.clone();
            plain.extra.retain(|name, _| name == JOURNAL_FROM);
            kept.push(plain);
            continue;
        }
        if entry.extra.contains_key(JOURNAL_FROM) {
            // A copy made for this run: nobody else shows it.
            if store.attachment_state(&entry.file)? != AttachmentState::Missing
                && store
                    .file_owners(&entry.file)?
                    .iter()
                    .all(|owner| owner == &note.id)
            {
                store.delete(&entry.file, Origin::Ui)?;
                undone += 1;
            }
            continue;
        }
        if store.attachment_state(&entry.file)? == AttachmentState::Sealed {
            unseal_in_place(store, &entry.file, &seal, &file_key)?;
            undone += 1;
        }
    }
    let mut fresh = store
        .note(&note.id)?
        .ok_or_else(|| failed("The note was deleted."))?;
    store_note_files(
        (!kept.is_empty()).then_some(kept.as_slice()),
        &mut fresh.extra,
    );
    let value = serde_json::to_value(&fresh).map_err(|err| failed(err.to_string()))?;
    store.put(DocType::Note, value, Put::Upsert, Origin::Ui)?;
    Ok(undone)
}

/// Undoes every stopped run a key in `keys` opens. Returns the files undone
/// and the notes no key opens.
pub fn undo_all(store: &Store, keys: &[Arc<LockKey>]) -> Result<(usize, usize), FilesError> {
    let mut undone = 0;
    let mut unopened = 0;
    for note in store.locked_notes()? {
        if !is_stopped(&note) {
            continue;
        }
        let key = note_files_from_extra(&note.extra)
            .unwrap_or_default()
            .iter()
            .find_map(header)
            .and_then(|envelope| keys.iter().find(|key| key.fits(&envelope)).cloned());
        match key {
            Some(key) => undone += undo_stopped(store, &note, &key)?,
            None => unopened += 1,
        }
    }
    Ok((undone, unopened))
}

#[cfg(test)]
#[path = "migrate_tests.rs"]
mod tests;
