//! A locked note's attachments: sealing them with the note, reading them
//! while it is open, and taking the seal off again (`kanso_sync::lock` has
//! the container and key wrapping; `kanso_store` the guards).
//!
//! Locking a note is three phases, so a note is never locked while one of
//! its files is still plain:
//!
//! 1. **Prepare.** Decide, for each linked file, whether to seal it in place,
//!    to seal a copy (another unlocked note shows it too), or to wrap the key
//!    of a file that is sealed already. The wraps go to the note's
//!    `kansoFiles` while the note is still plain, with the lock's salt beside
//!    them, so a crash leaves what [`recover`] needs.
//! 2. **Seal the files**, one transaction each. A file already sealed is
//!    skipped, so a lock that stopped resumes.
//! 3. **Lock the note**, by the caller (`Store::lock_note` or
//!    `Store::lock_book`), which checks that every listed file is sealed.
//!
//! Files sealed by lock steps for notes locked before sealing existed are in
//! [`migrate`].
//!
//! The functions take what they need (the store, a [`KeyBook`] of file keys)
//! and no window, so the desktop and the phones share them.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use kanso_core::model::{
    FileLock, KANSO_FILES_FIELD, NoteFile, note_files_from_extra, store_note_files,
};
use kanso_core::tasks::shown_file_ids;
use kanso_core::{DocType, Note};
use kanso_store::{Attachment, AttachmentState, Origin, Put, Store, StoreError};
use kanso_sync::lock::{
    Envelope, FileKey, LockKey, SEGMENT_CONTENT, SEGMENT_METADATA, SealId, SealedFileMetadata,
    SealedSegment, open_container, seal_container, unwrap_file_key, wrap_file_key,
};
use serde_json::{Value, json};

/// The key of a journal entry's extra fields that holds the lock's salt and
/// parameters, so a lock that stopped can be finished or undone with the
/// password alone.
const JOURNAL_LOCK: &str = "lock";
/// The journal entry field naming the file a copy was made from.
const JOURNAL_FROM: &str = "from";
/// The journal entry field set on a file this lock sealed (and so undoing it
/// unseals).
const JOURNAL_NEW: &str = "new";

/// File keys held for a session, by the seal generation they open. They are
/// zeroized when forgotten. A cloned book shares its keys, so a window's
/// readers see a key forgotten at once.
#[derive(Clone, Default)]
pub struct KeyBook(Arc<Mutex<HashMap<[u8; 16], FileKey>>>);

impl KeyBook {
    fn guard(&self) -> MutexGuard<'_, HashMap<[u8; 16], FileKey>> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// A copy of the key for `seal`, if held.
    pub fn get(&self, seal: &SealId) -> Option<FileKey> {
        self.guard()
            .get(seal.as_bytes())
            .map(|key| FileKey::from_bytes(*key.as_bytes()))
    }

    pub fn insert(&self, seal: &SealId, key: FileKey) {
        self.guard().insert(*seal.as_bytes(), key);
    }

    /// Forgets every key.
    pub fn clear(&self) {
        self.guard().clear();
    }

    pub fn len(&self) -> usize {
        self.guard().len()
    }

    pub fn is_empty(&self) -> bool {
        self.guard().is_empty()
    }

    /// Opens the keys of `note`'s list with `key` (the note's lock key) and
    /// holds them. A note without a list, or an entry that doesn't open, adds
    /// nothing. Returns how many were added.
    pub fn remember_note(&self, note: &Note, key: &LockKey) -> usize {
        let Some(Ok(envelope)) = Envelope::from_extra(&note.extra) else {
            return 0;
        };
        if !key.fits(&envelope) {
            return 0;
        }
        let mut added = 0;
        for entry in note_files_from_extra(&note.extra).unwrap_or_default() {
            if let Some((seal, file_key)) = unwrap_entry(key, &envelope, &note.id, &entry) {
                self.insert(&seal, file_key);
                added += 1;
            }
        }
        added
    }
}

impl fmt::Debug for KeyBook {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "KeyBook({} keys)", self.len())
    }
}

fn unwrap_entry(
    key: &LockKey,
    envelope: &Envelope,
    note_id: &str,
    entry: &NoteFile,
) -> Option<(SealId, FileKey)> {
    let seal = SealId::decode(&entry.seal)?;
    let wrapped = STANDARD.decode(&entry.key).ok()?;
    let file_key = unwrap_file_key(key, envelope, note_id, &entry.file, &seal, &wrapped).ok()?;
    Some((seal, file_key))
}

/// Why an attachment couldn't be sealed, opened or unsealed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FilesError {
    /// A file is sealed under another note's key, which isn't held: that
    /// note must be unlocked first.
    NeedsUnlock {
        file: String,
        /// The title of a note that has the key.
        owner: Option<String>,
    },
    /// The caller stopped the work between files.
    Stopped(String),
    /// Anything else, in words.
    Failed(String),
}

impl fmt::Display for FilesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FilesError::NeedsUnlock {
                owner: Some(owner), ..
            } => write!(
                f,
                "A picture or file in this note is sealed with \u{201c}{owner}\u{201d}. Unlock that note first."
            ),
            FilesError::NeedsUnlock { owner: None, .. } => f.write_str(
                "A picture or file in this note is sealed with another note's lock. Unlock that note first.",
            ),
            FilesError::Stopped(words) | FilesError::Failed(words) => f.write_str(words),
        }
    }
}

impl std::error::Error for FilesError {}

impl From<StoreError> for FilesError {
    fn from(err: StoreError) -> Self {
        FilesError::Failed(err.to_string())
    }
}

fn failed(words: impl Into<String>) -> FilesError {
    FilesError::Failed(words.into())
}

// MARK: Reading

/// An attachment's content, opened.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenedFile {
    pub name: String,
    pub content_type: String,
    pub bytes: Vec<u8>,
}

/// What reading an attachment gives.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reading {
    Opened(OpenedFile),
    /// Sealed, and no key for it is held: the note must be unlocked.
    Locked,
    /// No such file, or it has no bytes yet.
    Missing,
}

/// Reads file `id`: a plain one as stored, a sealed one decrypted with the
/// key `keys` holds for its seal generation.
pub fn read_file(store: &Store, id: &str, keys: &KeyBook) -> Reading {
    match store.attachment_state(id) {
        Ok(AttachmentState::Plain) => {
            let Ok(Some(meta)) = store.file_meta(id) else {
                return Reading::Missing;
            };
            match store.attachment(id) {
                Ok(Some((content_type, blob))) => Reading::Opened(OpenedFile {
                    name: meta.name,
                    content_type,
                    bytes: blob.data,
                }),
                _ => Reading::Missing,
            }
        }
        Ok(AttachmentState::Sealed) => {
            let Ok(Some(meta)) = store.file_meta(id) else {
                return Reading::Missing;
            };
            let Some(marker) = FileLock::from_extra(&meta.extra) else {
                return Reading::Locked;
            };
            let Some(seal) = SealId::decode(&marker.seal) else {
                return Reading::Locked;
            };
            let Some(file_key) = keys.get(&seal) else {
                return Reading::Locked;
            };
            let Ok(Some(container)) = store.stored_attachment(id) else {
                return Reading::Missing;
            };
            match open_content(id, &seal, &file_key, &marker, &meta, &container.data) {
                Some(opened) => Reading::Opened(opened),
                None => Reading::Locked,
            }
        }
        _ => Reading::Missing,
    }
}

fn open_content(
    id: &str,
    seal: &SealId,
    file_key: &FileKey,
    marker: &FileLock,
    meta: &kanso_core::model::FileMeta,
    container: &[u8],
) -> Option<OpenedFile> {
    let segments = open_container(id, seal, file_key, container).ok()?;
    let sealed_meta = if marker.meta {
        segments
            .iter()
            .find(|segment| segment.kind == SEGMENT_METADATA)
            .and_then(|segment| SealedFileMetadata::from_canonical_json(&segment.bytes).ok())?
    } else {
        SealedFileMetadata {
            name: meta.name.clone(),
            content_type: meta.content_type.clone(),
            public_in: meta.public_in.clone(),
        }
    };
    let content = segments
        .into_iter()
        .find(|segment| segment.kind == SEGMENT_CONTENT)?;
    Some(OpenedFile {
        name: sealed_meta.name,
        content_type: sealed_meta.content_type,
        bytes: content.bytes,
    })
}

// MARK: Sealing

/// Where a lock stands, for a progress sheet.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Phase {
    #[default]
    Preparing,
    Sealing,
    Locking,
}

/// How far a lock has come.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Progress {
    pub phase: Phase,
    /// Files sealed, of `total`.
    pub done: usize,
    pub total: usize,
    pub bytes_done: u64,
    pub bytes_total: u64,
}

/// What a lock will do: how many files and bytes it seals, and how many of
/// them are also shown by an unlocked note (so the locked note gets a sealed
/// copy).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Summary {
    pub files: usize,
    pub bytes: u64,
    pub shared: usize,
}

/// What a note's lock did to its files.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    /// Files sealed in place.
    pub sealed: usize,
    /// Sealed copies made of files an unlocked note also shows.
    pub copied: usize,
    /// Files already sealed, whose key this note now holds too.
    pub shared: usize,
    /// Files whose bytes haven't arrived (a sync still fetching them), left
    /// for the next save to seal. Only a save of a note already locked
    /// leaves one; a lock refuses instead.
    pub waiting: usize,
}

/// The state a lock reads and updates.
pub struct Job<'a> {
    pub store: &'a Store,
    /// File keys held, and those made here.
    pub keys: &'a KeyBook,
    /// The notes this pass locks: a file only they show is sealed in place.
    pub locking: &'a HashSet<String>,
    /// The sealed copy made for a shared original in this pass, so every
    /// note in it points at the same one.
    pub copies: &'a mut HashMap<String, String>,
    pub progress: Progress,
    /// Called as the lock moves; an error stops it before the next file.
    pub watch: &'a mut dyn FnMut(&Progress) -> Result<(), String>,
    /// A save, not a lock: a file whose bytes haven't arrived is left for
    /// the next save ([`Counts::waiting`]) rather than refusing, so the text
    /// is never held back. A note already locked is always saved this way.
    pub leave_waiting: bool,
}

impl<'a> Job<'a> {
    pub fn new(
        store: &'a Store,
        keys: &'a KeyBook,
        locking: &'a HashSet<String>,
        copies: &'a mut HashMap<String, String>,
        watch: &'a mut dyn FnMut(&Progress) -> Result<(), String>,
    ) -> Job<'a> {
        Job {
            store,
            keys,
            locking,
            copies,
            progress: Progress::default(),
            watch,
            leave_waiting: false,
        }
    }

    /// Sets what the sheet counts toward, from [`summarize`].
    pub fn expect(&mut self, summary: Summary) {
        self.progress.total = summary.files;
        self.progress.bytes_total = summary.bytes;
    }

    fn tell(&mut self, phase: Phase) -> Result<(), FilesError> {
        self.progress.phase = phase;
        let progress = self.progress;
        (self.watch)(&progress).map_err(FilesError::Stopped)
    }
}

/// What locking `notes` would seal: the plain files they link, and how many
/// of those another unlocked note shows too.
pub fn summarize(store: &Store, notes: &[Note]) -> Summary {
    let locking: HashSet<String> = notes.iter().map(|note| note.id.clone()).collect();
    let mut seen = HashSet::new();
    let mut summary = Summary::default();
    for note in notes {
        for id in shown_file_ids(&note.body) {
            if !seen.insert(id.clone())
                || store.attachment_state(&id).ok() != Some(AttachmentState::Plain)
            {
                continue;
            }
            summary.files += 1;
            summary.bytes += store
                .file_meta(&id)
                .ok()
                .flatten()
                .map_or(0, |meta| meta.content_length);
            if shared_with_unlocked(store, &id, &locking) {
                summary.shared += 1;
            }
        }
    }
    summary
}

/// Whether a file has a live owner outside `locking` that is not locked, and
/// so must keep the readable original.
fn shared_with_unlocked(store: &Store, file: &str, locking: &HashSet<String>) -> bool {
    store
        .file_owners(file)
        .unwrap_or_default()
        .into_iter()
        .filter(|owner| !locking.contains(owner))
        .any(|owner| {
            store
                .note(&owner)
                .ok()
                .flatten()
                .is_some_and(|other| !other.is_locked())
        })
}

/// Replaces the links to file `from` in `body` with links to `to`, only
/// where `from` is a whole ID.
pub fn replace_link(body: &str, from: &str, to: &str) -> String {
    let mut out = String::with_capacity(body.len());
    let mut rest = body;
    while let Some(at) = rest.find(from) {
        let after = &rest[at + from.len()..];
        let whole = !after
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
        out.push_str(&rest[..at]);
        out.push_str(if whole { to } else { from });
        rest = after;
    }
    out.push_str(rest);
    out
}

/// A plain copy of file `id` with an ID of its own: the same name, type and
/// bytes. Used to give a locked note its own file when another note keeps
/// the original (copy-on-share), and an unlocked note its own when a sealed
/// one is shared (copy-on-unlock).
pub fn copy_for_share(store: &Store, id: &str) -> Result<Option<String>, FilesError> {
    let Some(meta) = store.file_meta(id)? else {
        return Ok(None);
    };
    let Some((content_type, blob)) = store.attachment(id)? else {
        return Ok(None);
    };
    put_plain(
        store,
        &meta.name,
        &content_type,
        &meta.public_in,
        &blob.data,
    )
    .map(Some)
}

fn put_plain(
    store: &Store,
    name: &str,
    content_type: &str,
    public_in: &[String],
    bytes: &[u8],
) -> Result<String, FilesError> {
    let written = store.put(
        DocType::File,
        json!({
            "name": name,
            "contentType": content_type,
            "publicIn": public_in,
            "_attachments": {"index": {
                "content_type": content_type,
                "data": STANDARD.encode(bytes),
            }},
        }),
        Put::Upsert,
        Origin::Ui,
    )?;
    Ok(written.id)
}

/// What one linked file needs.
struct Planned {
    /// The file the note will list: the original, or its copy.
    target: String,
    /// The file it was a copy of.
    from: Option<String>,
    seal: SealId,
    key: FileKey,
    /// Plain now: this lock seals it.
    to_seal: bool,
    /// Sealed by this lock (or an earlier try of it), so undoing unseals it.
    new: bool,
}

/// The journal a plain note carries while its files are sealed: an entry per
/// file, with the lock's header beside it.
fn journal_entry(plan: &Planned, key: &LockKey, envelope: &Envelope, note_id: &str) -> NoteFile {
    let mut extra = serde_json::Map::new();
    extra.insert(
        JOURNAL_LOCK.into(),
        json!({"salt": envelope.salt, "m": envelope.m, "t": envelope.t, "p": envelope.p}),
    );
    if let Some(from) = &plan.from {
        extra.insert(JOURNAL_FROM.into(), Value::String(from.clone()));
    }
    if plan.new {
        extra.insert(JOURNAL_NEW.into(), Value::Bool(true));
    }
    NoteFile {
        file: plan.target.clone(),
        v: 1,
        seal: plan.seal.encode(),
        key: STANDARD.encode(wrap_file_key(
            key,
            envelope,
            note_id,
            &plan.target,
            &plan.seal,
            &plan.key,
        )),
        extra,
    }
}

/// The lock header a journal entry carries, as an envelope without text.
fn header(entry: &NoteFile) -> Option<Envelope> {
    let lock = entry.extra.get(JOURNAL_LOCK)?;
    Some(Envelope {
        v: 1,
        salt: lock.get("salt")?.as_str()?.to_owned(),
        m: lock.get("m")?.as_u64()?.try_into().ok()?,
        t: lock.get("t")?.as_u64()?.try_into().ok()?,
        p: lock.get("p")?.as_u64()?.try_into().ok()?,
        data: String::new(),
    })
}

/// Seals the files `note`'s text (`body`) links, for locking it with `key`.
///
/// On success `note`'s lock and `kansoFiles` are set and the text to seal is
/// returned (its links moved to any copies); the caller empties the note's
/// body and writes it with `Store::lock_note` (or `lock_book`), which checks
/// that every listed file is sealed. A note already locked (an attachment
/// added while it is open, or a legacy note) keeps its lock and gets its
/// list; the caller writes it as it writes any save.
pub fn seal_note_files(
    job: &mut Job,
    note: &mut Note,
    body: &str,
    key: &LockKey,
) -> Result<(String, Counts), FilesError> {
    job.tell(Phase::Preparing)?;
    let store = job.store;
    let locked = note.is_locked();
    // What the note's own list says: a lock that stopped (its journal
    // carries the lock's header), or the list of a note already locked. Its
    // keys open again here, and a copy made for a file earlier is reused,
    // so a link the editor still shows is not copied again at every save.
    let mut resumed: HashMap<String, NoteFile> = HashMap::new();
    let own_envelope = Envelope::from_extra(&note.extra).and_then(Result::ok);
    for entry in note_files_from_extra(&note.extra).unwrap_or_default() {
        let envelope = match (&own_envelope, locked) {
            (Some(envelope), true) => Some(envelope.clone()),
            _ => header(&entry),
        };
        let Some(envelope) = envelope.filter(|envelope| key.fits(envelope)) else {
            continue;
        };
        if let Some((seal, file_key)) = unwrap_entry(key, &envelope, &note.id, &entry) {
            job.keys.insert(&seal, file_key);
            let origin = entry
                .extra
                .get(JOURNAL_FROM)
                .and_then(Value::as_str)
                .map_or_else(|| entry.file.clone(), str::to_owned);
            resumed.insert(origin, entry);
        }
    }
    let mut body = body.to_owned();
    for (from, to) in job.copies.iter() {
        body = replace_link(&body, from, to);
    }
    let mut counts = Counts::default();
    let mut locking: HashSet<String> = job.locking.clone();
    locking.insert(note.id.clone());
    let mut plans: Vec<Planned> = Vec::new();
    // A file whose bytes haven't arrived yet (a sync still fetching them)
    // can't be sealed: a lock refuses before it writes anything, and a save
    // leaves it plain and unlisted for the next save to seal.
    let mut waiting = HashSet::new();
    for id in shown_file_ids(&body) {
        if store.attachment_state(&id)? == AttachmentState::Plain && !has_current_bytes(store, &id)?
        {
            if !(locked || job.leave_waiting) {
                let name = store.file_meta(&id)?.map_or_else(
                    || "A picture or file".to_owned(),
                    |meta| format!("\u{201c}{}\u{201d}", meta.name),
                );
                return Err(failed(format!(
                    "{name} hasn't finished downloading from your other devices, so it can't be sealed yet. Try again once sync has fetched it."
                )));
            }
            waiting.insert(id);
        }
    }
    let mut seen = HashSet::new();
    for id in shown_file_ids(&body)
        .into_iter()
        .filter(|id| seen.insert(id.clone()))
    {
        let state = store.attachment_state(&id)?;
        match state {
            AttachmentState::Missing => continue,
            AttachmentState::Sealed => {
                let meta = store
                    .file_meta(&id)?
                    .ok_or_else(|| failed("The attachment is gone."))?;
                let marker = FileLock::from_extra(&meta.extra)
                    .and_then(|marker| SealId::decode(&marker.seal))
                    .ok_or_else(|| failed("The attachment's seal is damaged."))?;
                let Some(file_key) = job.keys.get(&marker) else {
                    return Err(FilesError::NeedsUnlock {
                        owner: owner_title(store, &id, &note.id),
                        file: id,
                    });
                };
                // Shared with another locked note: one more wrap.
                let own = note_files_from_extra(&note.extra)
                    .unwrap_or_default()
                    .iter()
                    .any(|entry| entry.file == id);
                if !own {
                    counts.shared += 1;
                }
                let entry = resumed.get(&id);
                let new = entry.is_some_and(|entry| entry.extra.contains_key(JOURNAL_NEW));
                // A copy made for a file earlier keeps naming it.
                let from = entry
                    .and_then(|entry| entry.extra.get(JOURNAL_FROM))
                    .and_then(Value::as_str)
                    .map(str::to_owned);
                plans.push(Planned {
                    target: id,
                    from,
                    seal: marker,
                    key: file_key,
                    to_seal: false,
                    new,
                });
            }
            AttachmentState::Plain if waiting.contains(&id) => {
                counts.waiting += 1;
            }
            AttachmentState::Plain => {
                // A copy this pass made, or an earlier try left sealed.
                if let Some(entry) = resumed.get(&id).filter(|entry| entry.file != id)
                    && let Some(seal) = SealId::decode(&entry.seal)
                    && let Some(file_key) = job.keys.get(&seal)
                    && store.attachment_state(&entry.file)? == AttachmentState::Sealed
                {
                    body = replace_link(&body, &id, &entry.file);
                    job.copies.insert(id.clone(), entry.file.clone());
                    counts.copied += 1;
                    plans.push(Planned {
                        target: entry.file.clone(),
                        from: Some(id),
                        seal,
                        key: file_key,
                        to_seal: false,
                        new: true,
                    });
                    continue;
                }
                let others = shared_with_unlocked(store, &id, &locking);
                let (target, from) = if others {
                    let copy = match job.copies.get(&id) {
                        Some(copy) => copy.clone(),
                        None => {
                            let Some(copy) = copy_for_share(store, &id)? else {
                                continue;
                            };
                            job.copies.insert(id.clone(), copy.clone());
                            counts.copied += 1;
                            copy
                        }
                    };
                    body = replace_link(&body, &id, &copy);
                    (copy, Some(id))
                } else {
                    counts.sealed += 1;
                    (id, None)
                };
                // The pass may have sealed this copy for a note before.
                let sealed_copy = store.attachment_state(&target)? == AttachmentState::Sealed;
                if sealed_copy {
                    let marker = store
                        .file_meta(&target)?
                        .and_then(|meta| FileLock::from_extra(&meta.extra))
                        .and_then(|marker| SealId::decode(&marker.seal))
                        .ok_or_else(|| failed("The attachment's seal is damaged."))?;
                    let file_key =
                        job.keys
                            .get(&marker)
                            .ok_or_else(|| FilesError::NeedsUnlock {
                                owner: owner_title(store, &target, &note.id),
                                file: target.clone(),
                            })?;
                    plans.push(Planned {
                        target,
                        from,
                        seal: marker,
                        key: file_key,
                        to_seal: false,
                        new: true,
                    });
                } else {
                    plans.push(Planned {
                        target,
                        from,
                        seal: SealId::random(),
                        key: FileKey::random(),
                        to_seal: true,
                        new: true,
                    });
                }
            }
        }
    }
    // A copy reached by its original and by its own link is one file.
    let mut listed = HashSet::new();
    plans.retain(|plan| listed.insert(plan.target.clone()));
    let envelope = key.seal(&note.id, &body);
    let entries: Vec<NoteFile> = plans
        .iter()
        .map(|plan| journal_entry(plan, key, &envelope, &note.id))
        .collect();
    let pending: Vec<&Planned> = plans.iter().filter(|plan| plan.to_seal).collect();
    if job.progress.total == 0 {
        job.progress.total = pending.len();
        job.progress.bytes_total = pending
            .iter()
            .map(|plan| {
                store
                    .file_meta(&plan.target)
                    .ok()
                    .flatten()
                    .map_or(0, |meta| meta.content_length)
            })
            .sum();
    }
    // Phase 1: the wraps go to the note first, so no sealed file outlives the
    // keys that open it.
    if !pending.is_empty() {
        match store.set_note_files(&note.id, &entries) {
            Ok(written) => note.rev = Some(written.rev),
            // A note not stored yet has nothing to lose a key from.
            Err(StoreError::NotFound) => {}
            Err(err) => return Err(err.into()),
        }
    }
    // Phase 2: the files.
    job.tell(Phase::Sealing)?;
    for plan in &plans {
        if !plan.to_seal {
            job.keys
                .insert(&plan.seal, FileKey::from_bytes(*plan.key.as_bytes()));
            continue;
        }
        seal_file(store, plan)?;
        job.keys
            .insert(&plan.seal, FileKey::from_bytes(*plan.key.as_bytes()));
        job.progress.done += 1;
        job.progress.bytes_done += store
            .stored_attachment(&plan.target)?
            .map_or(0, |blob| blob.data.len() as u64);
        job.tell(Phase::Sealing)?;
    }
    // Phase 3 is the caller's; the list it writes has no journal in it.
    let list: Vec<NoteFile> = plans
        .iter()
        .map(|plan| {
            let mut entry = journal_entry(plan, key, &envelope, &note.id);
            // What the list keeps of a copy: the file it was made from, so
            // the link the editor still shows is not copied again.
            entry.extra.retain(|name, _| name == JOURNAL_FROM);
            entry
        })
        .collect();
    store_note_files(Some(&list), &mut note.extra);
    Envelope::store(Some(&envelope), &mut note.extra);
    job.tell(Phase::Locking)?;
    Ok((body, counts))
}

/// Whether open locked `note`'s text shows a file that is plain, or a
/// sealed one its list doesn't hold (a link pasted from another locked
/// note): its save must seal before it writes.
pub fn needs_sealing(store: &Store, note: &Note) -> bool {
    let listed: HashSet<String> = note_files_from_extra(&note.extra)
        .unwrap_or_default()
        .into_iter()
        .map(|entry| entry.file)
        .collect();
    shown_file_ids(&note.body)
        .iter()
        .any(|id| match store.attachment_state(id) {
            Ok(AttachmentState::Plain) => true,
            Ok(AttachmentState::Sealed) => !listed.contains(id),
            _ => false,
        })
}

/// Whether file `id` has the bytes its document describes: a sync may have
/// brought the document before them, or an older version's.
fn has_current_bytes(store: &Store, id: &str) -> Result<bool, FilesError> {
    let Some(meta) = store.file_meta(id)? else {
        return Ok(false);
    };
    let Some(blob) = store.stored_attachment(id)? else {
        return Ok(false);
    };
    Ok(meta
        .md5digest
        .as_deref()
        .is_none_or(|md5| md5.eq_ignore_ascii_case(&blob.md5_hex())))
}

/// The title of a note other than `except` that lists `file`, for words.
fn owner_title(store: &Store, file: &str, except: &str) -> Option<String> {
    store
        .file_owners(file)
        .ok()?
        .into_iter()
        .filter(|owner| owner != except)
        .find_map(|owner| store.note(&owner).ok().flatten())
        .map(|owner| crate::lock::display_title(&owner))
}

/// Seals plain file `plan.target` under the plan's key.
fn seal_file(store: &Store, plan: &Planned) -> Result<(), FilesError> {
    let mut meta = store
        .file_meta(&plan.target)?
        .ok_or_else(|| failed("The attachment is gone."))?;
    let Some((_, blob)) = store.attachment(&plan.target)? else {
        return Err(failed("The attachment has no content to seal."));
    };
    let metadata = SealedFileMetadata {
        name: meta.name.clone(),
        content_type: meta.content_type.clone(),
        public_in: meta.public_in.clone(),
    }
    .to_canonical_json()
    .map_err(|_| failed("The attachment's name and type can't be sealed."))?;
    let plain = blob.data.clone();
    // The plain bytes are only replaced by a container that opens to them.
    let container = sealed_container(&plan.target, &plan.seal, &plan.key, blob.data, metadata)?;
    let restore = meta.clone();
    let mut marker = FileLock::sealed(plan.seal.encode());
    marker.meta = true;
    FileLock::store(Some(&marker), &mut meta.extra);
    store.put_sealed_file(meta, Attachment::new(container))?;
    // And what was stored is read back and opened before the plain bytes
    // are let go: a write that didn't take puts them back.
    let stored = store.stored_attachment(&plan.target)?;
    if !stored.is_some_and(|stored| opens_to(plan, &stored.data, &plain)) {
        let mut back = store
            .file_meta(&plan.target)?
            .ok_or_else(|| failed("The attachment is gone."))?;
        back.name = restore.name;
        back.content_type = restore.content_type;
        back.public_in = restore.public_in;
        store.put_unsealed_file(back, Attachment::new(plain))?;
        return Err(failed("The attachment couldn't be sealed safely."));
    }
    Ok(())
}

/// Seals `content` and `metadata` for file `id`, and opens the container
/// again to check it gives them back: the plain bytes it replaces are gone
/// once it is written, so a container that doesn't open is never written.
fn sealed_container(
    id: &str,
    seal: &SealId,
    key: &FileKey,
    content: Vec<u8>,
    metadata: Vec<u8>,
) -> Result<Vec<u8>, FilesError> {
    let (want_content, want_metadata) = (content.clone(), metadata.clone());
    let container = seal_container(
        id,
        seal,
        key,
        &[
            SealedSegment {
                kind: SEGMENT_CONTENT,
                bytes: content,
            },
            SealedSegment {
                kind: SEGMENT_METADATA,
                bytes: metadata,
            },
        ],
    );
    let opened = open_container(id, seal, key, &container).map_err(|_| {
        failed("The attachment couldn't be sealed safely, so it was left as it is.")
    })?;
    let gives_back = |kind: u8, want: &[u8]| {
        opened
            .iter()
            .find(|segment| segment.kind == kind)
            .is_some_and(|segment| segment.bytes == want)
    };
    if !gives_back(SEGMENT_CONTENT, &want_content) || !gives_back(SEGMENT_METADATA, &want_metadata)
    {
        return Err(failed(
            "The attachment couldn't be sealed safely, so it was left as it is.",
        ));
    }
    Ok(container)
}

/// Seals new attachment `id` (`bytes`, never stored plain) for locked note
/// `note_id`, whose lock is `envelope` under `key`: the container goes to
/// the store, its file key to `keys`, and the entry for the note's list is
/// returned. A recording into a locked note attaches its parts this way.
#[allow(clippy::too_many_arguments)]
pub fn seal_new_file(
    store: &Store,
    keys: &KeyBook,
    key: &LockKey,
    envelope: &Envelope,
    note_id: &str,
    id: &str,
    bytes: Vec<u8>,
    name: &str,
    content_type: &str,
) -> Result<NoteFile, FilesError> {
    let seal = SealId::random();
    let file_key = FileKey::random();
    let metadata = SealedFileMetadata {
        name: name.to_owned(),
        content_type: content_type.to_owned(),
        public_in: Vec::new(),
    }
    .to_canonical_json()
    .map_err(|_| failed("The attachment's name and type can't be sealed."))?;
    let container = sealed_container(id, &seal, &file_key, bytes, metadata)?;
    // The stored name and type are the placeholders; the real ones are in
    // the container.
    let mut file = kanso_core::model::FileMeta {
        id: id.to_owned(),
        rev: store.get(id)?.map(|doc| doc.rev),
        name: kanso_core::model::SEALED_FILE_NAME.to_owned(),
        content_type: kanso_core::model::SEALED_FILE_CONTENT_TYPE.to_owned(),
        content_length: 0,
        public_in: Vec::new(),
        md5digest: None,
        created_at: kanso_core::now_ms(),
        extra: Default::default(),
    };
    let mut marker = FileLock::sealed(seal.encode());
    marker.meta = true;
    FileLock::store(Some(&marker), &mut file.extra);
    store.put_sealed_file(file, Attachment::new(container))?;
    let entry = NoteFile {
        file: id.to_owned(),
        v: 1,
        seal: seal.encode(),
        key: STANDARD.encode(wrap_file_key(key, envelope, note_id, id, &seal, &file_key)),
        extra: Default::default(),
    };
    keys.insert(&seal, file_key);
    Ok(entry)
}

/// Saves open locked `note` with `body` as its text, the way the editor
/// saves one: whatever its text shows that isn't sealed yet is sealed and
/// listed (a legacy note's files on its first save, a file linked from
/// elsewhere), the list is rewrapped, and the note is written sealed in
/// one `Store::lock_note` transaction. `key` must open the note. Returns
/// what was sealed.
pub fn save_locked(
    store: &Store,
    keys: &KeyBook,
    note: &mut Note,
    body: &str,
    key: &LockKey,
) -> Result<Counts, FilesError> {
    let locking = HashSet::from([note.id.clone()]);
    let mut copies = HashMap::new();
    let mut watch = |_: &Progress| Ok(());
    let mut job = Job::new(store, keys, &locking, &mut copies, &mut watch);
    job.leave_waiting = true;
    let (_, counts) = seal_note_files(&mut job, note, body, key)?;
    let mut stored = note.clone();
    stored.body.clear();
    let value = serde_json::to_value(&stored).map_err(|err| failed(err.to_string()))?;
    let written = store.lock_note(value)?;
    note.rev = Some(written.rev);
    Ok(counts)
}

/// Whether `container` opens, with the plan's key, to content `plain`.
fn opens_to(plan: &Planned, container: &[u8], plain: &[u8]) -> bool {
    open_container(&plan.target, &plan.seal, &plan.key, container)
        .ok()
        .and_then(|segments| {
            segments
                .into_iter()
                .find(|segment| segment.kind == SEGMENT_CONTENT)
        })
        .is_some_and(|content| content.bytes == plain)
}

// MARK: Crash recovery

/// A lock that stopped between its files, as a plain note shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Interrupted {
    /// Files the stopped lock listed.
    pub files: usize,
    /// How many of them are sealed already.
    pub sealed: usize,
}

/// Whether `note` is a plain note carrying a stopped lock's list. The
/// editor offers to finish or undo it.
pub fn interrupted(store: &Store, note: &Note) -> Option<Interrupted> {
    if note.is_locked() || !note.extra.contains_key(KANSO_FILES_FIELD) {
        return None;
    }
    let entries = note_files_from_extra(&note.extra)?;
    if entries.is_empty() {
        return None;
    }
    let sealed = entries
        .iter()
        .filter(|entry| store.attachment_state(&entry.file).ok() == Some(AttachmentState::Sealed))
        .count();
    Some(Interrupted {
        files: entries.len(),
        sealed,
    })
}

/// Whether `key` opens the list of the lock that stopped on `note`: the
/// password was the one that lock used.
pub fn opens_interrupted(note: &Note, key: &LockKey) -> bool {
    let entries = note_files_from_extra(&note.extra).unwrap_or_default();
    entries.iter().any(|entry| {
        header(entry)
            .filter(|envelope| key.fits(envelope))
            .and_then(|envelope| unwrap_entry(key, &envelope, &note.id, entry))
            .is_some()
    })
}

/// The lock a stopped lock's list was made under, for asking the password
/// that opens it: an envelope without text.
pub fn interrupted_header(note: &Note) -> Option<Envelope> {
    note_files_from_extra(&note.extra)?.iter().find_map(header)
}

/// Undoes a stopped lock: unseals the files it sealed and deletes the copies
/// it made, then takes the list off the note. `key` must be the lock key the
/// stopped lock used. Returns how many files it unsealed or removed.
///
/// A file stays sealed while another note holds a key for it (a locked note
/// that shares it, or another note of the same stopped notebook lock), so
/// undoing one never leaves a locked note listing a plain file. And it is
/// unsealed when this note's list is the last that holds its key, however
/// it came to be sealed: taking the list off must never leave a sealed file
/// nobody can open.
pub fn undo_locking(store: &Store, note: &Note, key: &LockKey) -> Result<usize, FilesError> {
    let entries = note_files_from_extra(&note.extra).unwrap_or_default();
    // Every key opens before anything changes: a wrong password changes
    // nothing.
    let mut opened = Vec::with_capacity(entries.len());
    for entry in &entries {
        let Some(envelope) = header(entry) else {
            continue;
        };
        if !key.fits(&envelope) {
            return Err(failed("That isn't the lock password."));
        }
        let Some((seal, file_key)) = unwrap_entry(key, &envelope, &note.id, entry) else {
            return Err(failed("That isn't the lock password."));
        };
        opened.push((entry, seal, file_key));
    }
    let mut undone = 0;
    let mut relinks: Vec<(String, String)> = Vec::new();
    for (entry, seal, file_key) in opened {
        let others = store
            .file_key_holders(&entry.file)?
            .iter()
            .any(|holder| holder != &note.id);
        let state = store.attachment_state(&entry.file)?;
        if entry.extra.contains_key(JOURNAL_FROM) {
            // A copy made for this lock: removed once nobody else shows it.
            let shown = store
                .file_owners(&entry.file)?
                .iter()
                .any(|owner| owner != &note.id);
            if state != AttachmentState::Missing && !others && !shown {
                store.delete(&entry.file, Origin::Ui)?;
                undone += 1;
            }
            continue;
        }
        if state != AttachmentState::Sealed {
            continue;
        }
        if others {
            // A locked note took the file meanwhile: it keeps the sealed
            // one, and this note gets a plain copy, as Remove Lock gives.
            let opened = open_sealed(store, &entry.file, &seal, &file_key)?;
            let copy = put_plain(
                store,
                &opened.name,
                &opened.content_type,
                &opened.public_in,
                &opened.bytes,
            )?;
            relinks.push((entry.file.clone(), copy));
        } else {
            unseal_in_place(store, &entry.file, &seal, &file_key)?;
        }
        undone += 1;
    }
    let mut fresh = store
        .note(&note.id)?
        .ok_or_else(|| failed("The note was deleted."))?;
    for (from, to) in &relinks {
        fresh.body = replace_link(&fresh.body, from, to);
    }
    store_note_files(None, &mut fresh.extra);
    let value = serde_json::to_value(&fresh).map_err(|err| failed(err.to_string()))?;
    store.put(DocType::Note, value, Put::Upsert, Origin::Ui)?;
    Ok(undone)
}

/// The stopped lock's list on plain `note`, its keys moved from `old` to
/// `new` (`new_envelope` is the lock they now go with): what a change of
/// password writes, so finishing or undoing the lock takes the new
/// password. `None` when the note carries no such list or `old` doesn't
/// open every entry of it (it is then left as it is).
pub fn rewrap_journal(
    note: &Note,
    old: &LockKey,
    new: &LockKey,
    new_envelope: &Envelope,
) -> Option<Vec<NoteFile>> {
    if note.is_locked() {
        return None;
    }
    let entries = note_files_from_extra(&note.extra)?;
    let mut moved = Vec::with_capacity(entries.len());
    for mut entry in entries {
        let envelope = header(&entry)?;
        if !old.fits(&envelope) {
            return None;
        }
        let (seal, file_key) = unwrap_entry(old, &envelope, &note.id, &entry)?;
        entry.key = STANDARD.encode(wrap_file_key(
            new,
            new_envelope,
            &note.id,
            &entry.file,
            &seal,
            &file_key,
        ));
        entry.extra.insert(
            JOURNAL_LOCK.into(),
            json!({
                "salt": new_envelope.salt,
                "m": new_envelope.m,
                "t": new_envelope.t,
                "p": new_envelope.p,
            }),
        );
        moved.push(entry);
    }
    (!moved.is_empty()).then_some(moved)
}

// MARK: Unsealing

/// What taking a note's lock off did to its files.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Unsealed {
    /// Files decrypted back to ordinary ones.
    pub decrypted: usize,
    /// Files copied, because another locked note still shows the sealed one.
    pub copied: usize,
}

/// Prepares the files of open locked `note` for its lock coming off: each
/// is decrypted back to a plain file, unless another locked note also shows
/// it, when this note gets a plain copy and the sealed one stays. Returns the
/// text with its links moved to any copies. `key` is the note's lock key.
/// The caller then writes the plain note with `Store::unlock_note`.
pub fn unseal_or_copy(
    store: &Store,
    note: &mut Note,
    key: &LockKey,
    watch: &mut dyn FnMut(&Progress) -> Result<(), String>,
) -> Result<(String, Unsealed), FilesError> {
    let Some(Ok(envelope)) = Envelope::from_extra(&note.extra) else {
        return Ok((note.body.clone(), Unsealed::default()));
    };
    let mut body = note.body.clone();
    let mut done = Unsealed::default();
    let entries = if note.extra.contains_key(KANSO_FILES_FIELD) {
        note_files_from_extra(&note.extra)
            .ok_or_else(|| failed("The note's attachment list is damaged."))?
    } else {
        Vec::new()
    };
    let mut progress = Progress {
        phase: Phase::Sealing,
        total: entries.len(),
        ..Progress::default()
    };
    for entry in entries {
        watch(&progress).map_err(FilesError::Stopped)?;
        progress.done += 1;
        if store.attachment_state(&entry.file)? != AttachmentState::Sealed {
            continue;
        }
        let Some((seal, file_key)) = unwrap_entry(key, &envelope, &note.id, &entry) else {
            return Err(failed("A picture or file in this note won't open."));
        };
        let shared = store
            .file_owners(&entry.file)?
            .iter()
            .any(|owner| owner != &note.id);
        if shared {
            let opened = open_sealed(store, &entry.file, &seal, &file_key)?;
            let copy = put_plain(
                store,
                &opened.name,
                &opened.content_type,
                &opened.public_in,
                &opened.bytes,
            )?;
            body = replace_link(&body, &entry.file, &copy);
            done.copied += 1;
        } else {
            unseal_in_place(store, &entry.file, &seal, &file_key)?;
            done.decrypted += 1;
        }
    }
    store_note_files(None, &mut note.extra);
    Ok((body, done))
}

struct Sealed {
    name: String,
    content_type: String,
    public_in: Vec<String>,
    bytes: Vec<u8>,
}

fn open_sealed(
    store: &Store,
    id: &str,
    seal: &SealId,
    file_key: &FileKey,
) -> Result<Sealed, FilesError> {
    let meta = store
        .file_meta(id)?
        .ok_or_else(|| failed("The attachment is gone."))?;
    let marker = FileLock::from_extra(&meta.extra)
        .ok_or_else(|| failed("The attachment's seal is damaged."))?;
    let container = store
        .stored_attachment(id)?
        .ok_or_else(|| failed("The attachment has no content."))?;
    let segments = open_container(id, seal, file_key, &container.data)
        .map_err(|_| failed("A picture or file in this note won't open."))?;
    let content = segments
        .iter()
        .find(|segment| segment.kind == SEGMENT_CONTENT)
        .ok_or_else(|| failed("The attachment has no content."))?;
    let (name, content_type, public_in) = if marker.meta {
        let sealed = segments
            .iter()
            .find(|segment| segment.kind == SEGMENT_METADATA)
            .and_then(|segment| SealedFileMetadata::from_canonical_json(&segment.bytes).ok())
            .ok_or_else(|| failed("The attachment's name is damaged."))?;
        (sealed.name, sealed.content_type, sealed.public_in)
    } else {
        (meta.name, meta.content_type, meta.public_in)
    };
    Ok(Sealed {
        name,
        content_type,
        public_in,
        bytes: content.bytes.clone(),
    })
}

fn unseal_in_place(
    store: &Store,
    id: &str,
    seal: &SealId,
    file_key: &FileKey,
) -> Result<(), FilesError> {
    let opened = open_sealed(store, id, seal, file_key)?;
    let mut meta = store
        .file_meta(id)?
        .ok_or_else(|| failed("The attachment is gone."))?;
    meta.name = opened.name;
    meta.content_type = opened.content_type;
    meta.public_in = opened.public_in;
    store.put_unsealed_file(meta, Attachment::new(opened.bytes))?;
    Ok(())
}

pub mod migrate;

#[cfg(test)]
#[path = "files_tests.rs"]
mod tests;
