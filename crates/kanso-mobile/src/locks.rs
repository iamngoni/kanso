//! The lock session: the lock password's keys, held in memory only.
//!
//! Typing the lock password once opens every note locked with it until the
//! notes are locked again, or the auto-lock time passes
//! ([`kanso_engine::lock::AUTO_LOCK`]) since a key was added or a locked note
//! saved. The keys never leave this crate: the shell hears only that the
//! notes are open or closed ([`LockSession::lock_events`]).

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Instant;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use kanso_core::model::{FileLock, NoteFile, note_files_from_extra, store_note_files};
use kanso_core::tasks::linked_file_ids;
use kanso_core::{DocType, Note, now_ms};
use kanso_engine::links::tags::apply_inline_tags;
use kanso_engine::lock::{self, LockView, Target};
use kanso_store::{Attachment, Origin, Put};
use kanso_sync::lock::{
    Envelope, FileKey, LockError, LockKey, LockPassword, SealId, SealedSegment,
};

use crate::boundary::{guard, held};
use crate::editing::Editors;
use crate::errors::KansoError;
use crate::library::{Inner, Library};
use crate::model::{LockEvent, LockReason, LockStatus, Saved};

/// One lock that exists: a note's or notebook's envelope, and how many
/// notes and notebooks share its salt and parameters.
struct LockGroup {
    id: String,
    envelope: Envelope,
    count: usize,
}

/// The lock session of one library.
#[derive(Clone)]
pub struct LockSession {
    pub(crate) inner: Arc<Inner>,
}

impl LockSession {
    /// Seals a note being saved into a locked notebook, including its linked
    /// attachment keys, while keeping the body open for the editor.
    pub(crate) fn seal_for_notebook(&self, note: &mut Note) -> Result<Note, KansoError> {
        let target = Target::of_book(&note.book_id, &self.inner.store);
        let keys = held(&self.inner.keys);
        let key = target
            .as_ref()
            .and_then(|target| target.envelope.as_ref().ok())
            .and_then(|envelope| keys.fitting(envelope))
            .or_else(|| keys.current())
            .ok_or(KansoError::Locked)?;
        let body = note.body.clone();
        let locking = HashSet::from([note.id.clone()]);
        let copies = HashMap::new();
        let (body, envelope) = self.seal_note_files(note, &body, &key, &locking, &copies)?;
        // A file shared with an unlocked note is copied: the editor must
        // take the rewritten links, not the ones it started with.
        note.body = body;
        Envelope::store(Some(&envelope), &mut note.extra);
        let mut stored = note.clone();
        stored.body.clear();
        Ok(stored)
    }

    /// Seals notes imported or created in a locked notebook after its key is
    /// held, including any attachments in their bodies.
    pub(crate) fn seal_unsealed(&self) {
        let Ok(notes) = self.inner.store.unsealed_in_locked_books() else {
            return;
        };
        for mut note in notes {
            match self.seal_for_notebook(&mut note).and_then(|stored| {
                serde_json::to_value(stored)
                    .map_err(|error| KansoError::storage(error.to_string()))
                    .and_then(|value| self.inner.store.lock_note(value).map_err(Into::into))
            }) {
                Ok(_) => {}
                Err(error) => eprintln!("kanso: locked notebooks: couldn't seal a note: {error}"),
            }
        }
    }

    /// Seals one revealed legacy note in a locked notebook and updates its
    /// stored revision, without returning plaintext to storage.
    pub(crate) fn seal_unsealed_note(&self, note: &mut Note) -> Result<(), KansoError> {
        let stored = self.seal_for_notebook(note)?;
        let value =
            serde_json::to_value(stored).map_err(|error| KansoError::storage(error.to_string()))?;
        let written = self.inner.store.lock_note(value)?;
        note.rev = Some(written.rev);
        Ok(())
    }

    /// Opens the locked notes with `password`. `Ok(false)` when it isn't the
    /// lock password; an error when it can't be one (too short) or nothing
    /// is locked yet. Deriving the key takes a fraction of a second.
    pub fn unlock(&self, password: String) -> Result<bool, KansoError> {
        guard(|| {
            let password = lock::password(&password).map_err(KansoError::invalid)?;
            let target = lock::any_lock(&self.inner.store)
                .ok_or_else(|| KansoError::invalid("Nothing is locked yet."))?;
            self.try_password(&target, &password)
        })
    }

    /// Opens note `id` with `password`, tried on that note's own lock (or on
    /// the locked notebook it is in, when it isn't sealed yet), and keeps
    /// the key for the session: every note locked with it opens. `Ok(false)`
    /// when it isn't the password.
    pub fn unlock_note(&self, id: &str, password: String) -> Result<bool, KansoError> {
        guard(|| {
            let password = lock::password(&password).map_err(KansoError::invalid)?;
            let note = self.library().stored(id)?;
            let target = Target::of_note(&note, &self.inner.store)
                .ok_or_else(|| KansoError::invalid("That note isn't locked."))?;
            self.try_password(&target, &password)
        })
    }

    /// Opens locked notebook `book_id` (or the locked notebook it is in)
    /// with `password`, as [`LockSession::unlock_note`] does a note.
    pub fn unlock_notebook(&self, book_id: &str, password: String) -> Result<bool, KansoError> {
        guard(|| {
            let password = lock::password(&password).map_err(KansoError::invalid)?;
            self.require_book(book_id)?;
            let target = Target::of_book(book_id, &self.inner.store)
                .ok_or_else(|| KansoError::invalid("That notebook isn't locked."))?;
            self.try_password(&target, &password)
        })
    }

    /// Tries `password` on `target`; the key it gives is kept when it opens,
    /// unless the notes were locked while it was being derived
    /// ([`KansoError::Locked`]).
    fn try_password(&self, target: &Target, password: &LockPassword) -> Result<bool, KansoError> {
        self.try_password_since(target, password, self.generation())
    }

    /// [`LockSession::try_password`] for an unlock that began at lock
    /// generation `since`.
    pub(crate) fn try_password_since(
        &self,
        target: &Target,
        password: &LockPassword,
        since: u64,
    ) -> Result<bool, KansoError> {
        let envelope = target
            .envelope
            .as_ref()
            .map_err(|err| KansoError::invalid(lock::sentence(err)))?;
        let key = LockKey::for_envelope(password, envelope)
            .map_err(|err| KansoError::invalid(lock::sentence(&err)))?;
        match key.open(&target.id, envelope) {
            Ok(_) => {}
            Err(LockError::WrongPassword) => return Ok(false),
            Err(err) => return Err(KansoError::invalid(lock::sentence(&err))),
        }
        if self.add_key(key, since) {
            self.cache_note_file_keys(since);
            Ok(true)
        } else {
            Err(KansoError::Locked)
        }
    }

    /// Opens each wrapped attachment key whose note envelope the session holds.
    fn cache_note_file_keys(&self, since: u64) {
        let notes = self.inner.store.locked_notes().unwrap_or_default();
        let mut keys = held(&self.inner.keys);
        if self.inner.lock_gen.load(Ordering::SeqCst) != since {
            return;
        }
        keys.forget_file_keys();
        for note in notes {
            Self::unwrap_note_file_keys(&mut keys, &note);
        }
    }

    /// Adds only explicitly recorded `kansoFiles`; legacy missing fields stay unknown.
    pub(crate) fn cache_note_file_keys_for_note(&self, note: &Note) {
        if !note
            .extra
            .contains_key(kanso_core::model::KANSO_FILES_FIELD)
        {
            return;
        }
        let since = self.generation();
        let mut keys = held(&self.inner.keys);
        if self.inner.lock_gen.load(Ordering::SeqCst) != since {
            return;
        }
        Self::unwrap_note_file_keys(&mut keys, note);
    }

    fn unwrap_note_file_keys(keys: &mut kanso_engine::lock::Session, note: &Note) {
        let Some(files) = note_files_from_extra(&note.extra) else {
            return;
        };
        let Some(Ok(envelope)) = Envelope::from_extra(&note.extra) else {
            return;
        };
        let Some(key) = keys.fitting(&envelope) else {
            return;
        };
        for file in files {
            let Some(seal) = SealId::decode(&file.seal) else {
                continue;
            };
            let Ok(wrapped) = STANDARD.decode(file.key) else {
                continue;
            };
            if let Ok(file_key) = kanso_sync::lock::unwrap_file_key(
                &key, &envelope, &note.id, &file.file, &seal, &wrapped,
            ) {
                keys.remember_file_key(&seal, file_key);
            }
        }
    }

    /// The lock generation now: the number of times the notes were locked.
    pub(crate) fn generation(&self) -> u64 {
        self.inner.lock_gen.load(Ordering::SeqCst)
    }

    /// Where the lock stands: whether a lock password exists, whether the
    /// notes are open, and whether biometrics can open them.
    pub fn lock_status(&self) -> Result<LockStatus, KansoError> {
        guard(|| {
            Ok(LockStatus {
                has_lock_password: lock::any_lock(&self.inner.store).is_some(),
                unlocked: held(&self.inner.keys).is_unlocked(),
                biometrics_on: self.biometrics_on()?,
            })
        })
    }

    /// Closes every locked note. Their keys are forgotten.
    pub fn lock(&self) -> Result<(), KansoError> {
        guard(|| {
            self.close(LockReason::Manual);
            Ok(())
        })
    }

    /// Closes every locked note because the app is leaving the foreground.
    /// The keys are forgotten, as by [`LockSession::lock`].
    pub fn lock_for_background(&self) -> Result<(), KansoError> {
        guard(|| {
            self.close(LockReason::Background);
            Ok(())
        })
    }

    /// Whether the lock password has opened the locked notes.
    pub fn is_unlocked(&self) -> Result<bool, KansoError> {
        guard(|| Ok(held(&self.inner.keys).is_unlocked()))
    }

    /// Whether note `id` can be read now: it isn't locked, or a key this
    /// session holds opens it.
    pub fn is_note_unlocked(&self, id: &str) -> Result<bool, KansoError> {
        guard(|| {
            let library = Library {
                inner: self.inner.clone(),
            };
            let mut note = library.stored(id)?;
            Ok(!library.reveal(&mut note).is_closed())
        })
    }

    /// Locks note `id` with the lock password: its text is sealed, its earlier
    /// versions are deleted, and it stays open for another auto-lock time.
    ///
    /// The first locked note sets the lock password; later ones need the
    /// same one ([`KansoError::Auth`] when it isn't).
    pub fn lock_note(&self, id: &str, password: String) -> Result<Saved, KansoError> {
        guard(|| {
            let password = lock::password(&password).map_err(KansoError::invalid)?;
            let note = self.lockable(id)?;
            let since = self.generation();
            let key = self.key_from_password(&password)?;
            // Sealed with the key just typed, never one the session held.
            let saved = self.seal_note(note, &key)?;
            self.add_key(key, since);
            Ok(saved)
        })
    }

    /// Locks note `id` with the key this session holds for the library's
    /// lock, as the desktop does when the notes are open: no password is
    /// typed. [`KansoError::Locked`] when no held key opens an existing
    /// lock (none held, or only keys of locks that are gone).
    pub fn lock_note_with_session(&self, id: &str) -> Result<Saved, KansoError> {
        guard(|| {
            let note = self.lockable(id)?;
            let key = self.session_lock_key().ok_or(KansoError::Locked)?;
            self.seal_note(note, &key)
        })
    }

    /// Takes note `id`'s lock off: its text is saved as plain text. The note
    /// must be open ([`KansoError::Locked`] otherwise), and not in a locked
    /// notebook, where it stays locked.
    pub fn remove_note_lock(&self, id: &str) -> Result<Saved, KansoError> {
        guard(|| {
            let library = self.library();
            let stored = library.stored(id)?;
            // Reading a note in a locked notebook seals it, which moves its
            // revision.
            let mut note = stored.clone();
            let view = library.reveal(&mut note);
            match view {
                LockView::Plain => return Err(KansoError::invalid("That note isn't locked.")),
                LockView::Closed(_) => return Err(KansoError::Locked),
                LockView::Open => {}
            }
            if self.inner.store.note_in_locked_book(id)? {
                return Err(KansoError::invalid(
                    "The note is in a locked notebook, so it stays locked. Move it out first.",
                ));
            }
            let owned = if note
                .extra
                .contains_key(kanso_core::model::KANSO_FILES_FIELD)
            {
                note_files_from_extra(&note.extra).ok_or_else(|| {
                    KansoError::invalid("The note's attachment ownership list is damaged.")
                })?
            } else {
                Vec::new()
            };
            for ownership in &owned {
                let owners = self.inner.store.file_owners(&ownership.file)?;
                let shared = owners.iter().any(|owner| owner != id);
                let Some(meta) = self.inner.store.file_meta(&ownership.file)? else {
                    continue;
                };
                let Some(marker) = FileLock::from_extra(&meta.extra) else {
                    continue;
                };
                let Some(seal) = SealId::decode(&marker.seal) else {
                    return Err(KansoError::Locked);
                };
                let file_key = held(&self.inner.keys)
                    .file_key(&seal)
                    .map(|key| FileKey::from_bytes(*key.as_bytes()))
                    .ok_or(KansoError::Locked)?;
                let stored = self
                    .inner
                    .store
                    .stored_attachment(&ownership.file)?
                    .ok_or(KansoError::NotFound)?;
                let segments = kanso_sync::lock::open_container(
                    &ownership.file,
                    &seal,
                    &file_key,
                    &stored.data,
                )
                .map_err(|_| KansoError::Locked)?;
                let content = segments
                    .into_iter()
                    .find(|segment| segment.kind == kanso_sync::lock::SEGMENT_CONTENT)
                    .ok_or(KansoError::Locked)?;
                if shared {
                    // Another locked note still shows the sealed original;
                    // give this now-plain note its own readable copy so the
                    // original's seal and ownership stay untouched.
                    let copy = self.inner.store.put(
                        DocType::File,
                        serde_json::json!({
                            "name": meta.name.clone(),
                            "contentType": meta.content_type.clone(),
                            "_attachments": {"index": {
                                "content_type": meta.content_type.clone(),
                                "data": STANDARD.encode(&content.bytes),
                            }},
                        }),
                        Put::Upsert,
                        Origin::Ui,
                    )?;
                    note.body = note.body.replace(&ownership.file, &copy.id);
                } else {
                    self.inner
                        .store
                        .put_unsealed_file(meta, Attachment::new(content.bytes))?;
                }
            }
            store_note_files(None, &mut note.extra);
            // The text as it is stays, now plain; its `#tags` count again.
            Envelope::store(None, &mut note.extra);
            note.updated_at = now_ms();
            apply_inline_tags(&self.inner.store, &mut note, None);
            let value =
                serde_json::to_value(&note).map_err(|err| KansoError::storage(err.to_string()))?;
            match self.inner.store.unlock_note(value) {
                Ok(written) => Ok(Saved {
                    id: written.id,
                    rev: written.rev,
                }),
                Err(kanso_store::StoreError::Conflict) => Err(KansoError::Conflict {
                    current_rev: library
                        .stored(id)
                        .ok()
                        .and_then(|n| n.rev)
                        .unwrap_or_default(),
                }),
                Err(err) => Err(err.into()),
            }
        })
    }

    /// Locks notebook `book_id`: every note in it and in the notebooks inside
    /// it is sealed, as are those put there later. Returns how many notes
    /// were sealed.
    ///
    /// With a `password`, its key seals them: the lock password when
    /// something is locked already ([`KansoError::Auth`] when it isn't), else
    /// the first one, which sets it. Without one, the notes must be open
    /// with a key that opens an existing lock, which then seals them
    /// ([`KansoError::Locked`] otherwise).
    pub fn lock_notebook(
        &self,
        book_id: &str,
        password: Option<String>,
    ) -> Result<u32, KansoError> {
        guard(|| {
            self.require_book(book_id)?;
            if Target::of_book(book_id, &self.inner.store).is_some() {
                return Err(KansoError::invalid("That notebook is locked already."));
            }
            let since = self.generation();
            let sealed = match password {
                // A typed password decides the key, open notes or not.
                Some(text) => {
                    let password = lock::password(&text).map_err(KansoError::invalid)?;
                    let key = self.key_from_password(&password)?;
                    let sealed = self.seal_book(book_id, &key)?;
                    self.add_key(key, since);
                    sealed
                }
                None => {
                    let key = self.session_lock_key().ok_or(KansoError::Locked)?;
                    let sealed = self.seal_book(book_id, &key)?;
                    held(&self.inner.keys).touch(Instant::now());
                    sealed
                }
            };
            Ok(sealed)
        })
    }

    fn seal_book(&self, book_id: &str, key: &LockKey) -> Result<u32, KansoError> {
        let books = self.inner.store.books()?;
        let inside = lock::book_subtree(book_id, &books);
        let mut book = books
            .into_iter()
            .find(|book| book.id == book_id)
            .ok_or(KansoError::NotFound)?;
        let mut sealed = Vec::new();
        let notes = self.inner.store.notes_in_books(&inside)?;
        // Every note in the subtree is sealed in this pass, so a file they
        // share stays one sealed original rather than becoming a copy each.
        let locking: HashSet<String> = notes.iter().map(|note| note.id.clone()).collect();
        // One copy per shared original for this whole pass, so every note in
        // the notebook points at the same sealed copy.
        let mut copies = HashMap::new();
        for note in &notes {
            if note.is_locked() {
                continue;
            }
            for id in note_file_ids(note) {
                if copies.contains_key(&id) {
                    continue;
                }
                if self.inner.store.attachment_state(&id)? == kanso_store::AttachmentState::Plain
                    && self.shared_with_unlocked(&id, &note.id, &locking)?
                    && let Some((copy, _)) = self.copy_plain_file(&id)?
                {
                    copies.insert(id, copy);
                }
            }
        }
        for mut note in notes {
            if note.is_locked() {
                continue;
            }
            let body = note.body.clone();
            let (_, envelope) = self.seal_note_files(&mut note, &body, key, &locking, &copies)?;
            Envelope::store(Some(&envelope), &mut note.extra);
            note.body.clear();
            sealed.push(
                serde_json::to_value(note).map_err(|err| KansoError::storage(err.to_string()))?,
            );
        }
        Envelope::store(Some(&key.seal(book_id, "")), &mut book.extra);
        let count = sealed.len();
        let book =
            serde_json::to_value(book).map_err(|err| KansoError::storage(err.to_string()))?;
        self.inner.store.lock_book(book, sealed)?;
        Ok(u32::try_from(count).unwrap_or(u32::MAX))
    }

    /// Takes notebook `book_id`'s lock off: notes put in it from now on
    /// aren't locked, and the notes in it keep their own locks. The notebook
    /// must be open ([`KansoError::Locked`] otherwise).
    pub fn remove_notebook_lock(&self, book_id: &str) -> Result<(), KansoError> {
        guard(|| {
            self.require_book(book_id)?;
            let target = Target::of_book(book_id, &self.inner.store)
                .filter(|target| target.id == book_id)
                .ok_or_else(|| KansoError::invalid("That notebook isn't locked."))?;
            if !target.is_open(&held(&self.inner.keys)) {
                return Err(KansoError::Locked);
            }
            lock::take_lock_off(&self.inner.store, book_id).map_err(KansoError::storage)
        })
    }

    /// Whether notebook `book_id`'s notes can be read now: it (and the
    /// notebooks it is in) isn't locked, or a key this session holds opens
    /// it.
    pub fn notebook_is_open(&self, book_id: &str) -> Result<bool, KansoError> {
        guard(|| {
            self.require_book(book_id)?;
            Ok(Target::of_book(book_id, &self.inner.store)
                .is_none_or(|target| target.is_open(&held(&self.inner.keys))))
        })
    }

    /// Seals every locked note and notebook `old` opens again under a new
    /// password, and holds only the new key from now on. Returns how many
    /// `old` didn't open: they keep their own password. A wrong `old` is
    /// [`KansoError::Auth`]; biometrics, if on, keep the new key.
    pub fn change_password(&self, old: String, new: String) -> Result<u32, KansoError> {
        guard(|| self.change_password_since(old, new, self.generation()))
    }

    /// [`LockSession::change_password`] for a change that began at lock
    /// generation `since`. When the notes were locked meanwhile the notes
    /// are still re-sealed, but the new key isn't held (they stay closed)
    /// and biometrics, whose kept keys are now the old ones, are turned off.
    pub(crate) fn change_password_since(
        &self,
        old: String,
        new: String,
        since: u64,
    ) -> Result<u32, KansoError> {
        let wrong = || KansoError::auth(lock::sentence(&LockError::WrongPassword));
        // A shorter text can't be the password.
        let old = LockPassword::new(&old).map_err(|_| wrong())?;
        let new = lock::password(&new).map_err(KansoError::invalid)?;
        if lock::any_lock(&self.inner.store).is_none() {
            return Err(KansoError::invalid("Nothing is locked yet."));
        }
        let params = held(&self.inner.keys).params();
        let (key, skipped) =
            lock::rekey(&self.inner.store, &old, &new, params).map_err(|words| {
                if words == lock::sentence(&LockError::WrongPassword) {
                    wrong()
                } else {
                    KansoError::invalid(words)
                }
            })?;
        let applied = {
            let mut keys = held(&self.inner.keys);
            if self.inner.lock_gen.load(Ordering::SeqCst) == since {
                let was_open = keys.is_unlocked();
                keys.replace(Some(key), Instant::now());
                Some(was_open)
            } else {
                None
            }
        };
        match applied {
            Some(was_open) => {
                self.cache_note_file_keys(since);
                crate::biometrics::keep(&self.inner);
                if !was_open {
                    self.inner.lock_events.send(LockEvent::Unlocked);
                }
            }
            None => crate::biometrics::forget_quietly(&self.inner),
        }
        Ok(u32::try_from(skipped).unwrap_or(u32::MAX))
    }

    /// Calls `sink` each time the notes open or close, until it returns
    /// `false`.
    pub fn lock_events(&self, sink: impl FnMut(LockEvent) -> bool + Send + 'static) {
        self.inner.lock_events.add(sink);
    }

    /// Closes the notes if the auto-lock time has passed at `now`; whether it
    /// did. The library's timer calls it every few seconds; a phone that goes
    /// to the background may call it, or [`LockSession::lock`], at once.
    pub fn lock_if_idle(&self, now: Instant) -> bool {
        let idle = held(&self.inner.keys).idle(now);
        if idle {
            self.close(LockReason::Idle);
        }
        idle
    }

    /// Holds `key` if the notes weren't locked since generation `since`,
    /// and tells listeners if they weren't open before. Biometrics, if on,
    /// keep the session's keys again. Whether the key was held.
    fn add_key(&self, key: LockKey, since: u64) -> bool {
        self.add_keys(vec![key], true, since)
    }

    fn library(&self) -> Library {
        Library {
            inner: self.inner.clone(),
        }
    }

    /// Notebook `book_id` as a row of the library, or [`KansoError::NotFound`].
    fn require_book(&self, book_id: &str) -> Result<(), KansoError> {
        let books = self.inner.store.books()?;
        if books.iter().any(|book| book.id == book_id) {
            Ok(())
        } else {
            Err(KansoError::NotFound)
        }
    }

    /// Note `id`, which may be locked now: not sealed, and not in a locked
    /// notebook.
    fn lockable(&self, id: &str) -> Result<Note, KansoError> {
        let note = self.library().stored(id)?;
        if note.is_locked() || self.inner.store.note_in_locked_book(id)? {
            return Err(KansoError::invalid("That note is locked already."));
        }
        Ok(note)
    }

    /// The key `password` gives: the one that opens what is locked already
    /// ([`KansoError::Auth`] when it opens none of it), or a new one when
    /// nothing is. Locks made with different passwords (on another device)
    /// each get a try, the most used lock first, one derivation per
    /// distinct salt.
    fn key_from_password(&self, password: &LockPassword) -> Result<LockKey, KansoError> {
        let groups = self.lock_groups();
        if groups.is_empty() {
            let params = held(&self.inner.keys).params();
            return LockKey::create(password, params)
                .map_err(|err| KansoError::invalid(lock::sentence(&err)));
        }
        for group in &groups {
            let Ok(key) = LockKey::for_envelope(password, &group.envelope) else {
                continue;
            };
            if key.open(&group.id, &group.envelope).is_ok() {
                return Ok(key);
            }
        }
        Err(KansoError::auth("That isn't the lock password."))
    }

    /// The locks that exist, one per distinct salt and parameters, the one
    /// most notes and notebooks use first (then in ID order).
    fn lock_groups(&self) -> Vec<LockGroup> {
        let mut groups: Vec<LockGroup> = Vec::new();
        let mut add = |id: String, envelope: Envelope| {
            let same = |other: &Envelope| {
                (&other.salt, other.m, other.t, other.p)
                    == (&envelope.salt, envelope.m, envelope.t, envelope.p)
            };
            match groups.iter_mut().find(|group| same(&group.envelope)) {
                Some(group) => group.count += 1,
                None => groups.push(LockGroup {
                    id,
                    envelope,
                    count: 1,
                }),
            }
        };
        for note in self.inner.store.locked_notes().unwrap_or_default() {
            if let Some(Ok(envelope)) = Envelope::from_extra(&note.extra) {
                add(note.id, envelope);
            }
        }
        for book in self.inner.store.books().unwrap_or_default() {
            if let Some(Ok(envelope)) = Envelope::from_extra(&book.extra) {
                add(book.id, envelope);
            }
        }
        groups.sort_by_key(|group| std::cmp::Reverse(group.count));
        groups
    }

    /// The key this session holds for the library's lock: of the held keys
    /// that open some lock that exists, the one that opens the most (the
    /// first held wins a tie). A key of locks that are all gone is stale and
    /// never chosen, nor is one of a stray lock another device made.
    fn session_lock_key(&self) -> Option<Arc<LockKey>> {
        let groups = self.lock_groups();
        let keys = held(&self.inner.keys);
        let mut best: Option<(usize, Arc<LockKey>)> = None;
        for key in keys.keys() {
            let opens: usize = groups
                .iter()
                .filter(|group| key.fits(&group.envelope))
                .map(|group| group.count)
                .sum();
            if opens > best.as_ref().map_or(0, |(most, _)| *most) {
                best = Some((opens, key.clone()));
            }
        }
        best.map(|(_, key)| key)
    }

    /// Seals `note` with `key` and writes it.
    fn seal_note(&self, mut note: Note, key: &LockKey) -> Result<Saved, KansoError> {
        let body = std::mem::take(&mut note.body);
        let locking = HashSet::from([note.id.clone()]);
        let copies = HashMap::new();
        let (_, envelope) = self.seal_note_files(&mut note, &body, key, &locking, &copies)?;
        Envelope::store(Some(&envelope), &mut note.extra);
        let value =
            serde_json::to_value(&note).map_err(|err| KansoError::storage(err.to_string()))?;
        let written = self.inner.store.lock_note(value)?;
        held(&self.inner.keys).touch(Instant::now());
        Ok(Saved {
            id: written.id,
            rev: written.rev,
        })
    }

    /// Whether a plain file has an owner outside `locking` that is still
    /// unlocked, and so must keep the readable original.
    fn shared_with_unlocked(
        &self,
        file: &str,
        note: &str,
        locking: &HashSet<String>,
    ) -> Result<bool, KansoError> {
        Ok(self
            .inner
            .store
            .file_owners(file)?
            .into_iter()
            .filter(|owner| owner != note && !locking.contains(owner))
            .any(|owner| {
                self.inner
                    .store
                    .note(&owner)
                    .ok()
                    .flatten()
                    .is_some_and(|other| !other.is_locked())
            }))
    }

    /// A readable copy of a plain file, with its document, for the note
    /// being locked to point at instead of the original.
    fn copy_plain_file(
        &self,
        file: &str,
    ) -> Result<Option<(String, kanso_core::model::FileMeta)>, KansoError> {
        let Some(meta) = self.inner.store.file_meta(file)? else {
            return Ok(None);
        };
        let Some((content_type, stored)) = self.inner.store.attachment(file)? else {
            return Ok(None);
        };
        let copy = self.inner.store.put(
            DocType::File,
            serde_json::json!({
                "name": meta.name.clone(),
                "contentType": content_type.clone(),
                "_attachments": {"index": {
                    "content_type": content_type,
                    "data": STANDARD.encode(&stored.data),
                }},
            }),
            Put::Upsert,
            Origin::Ui,
        )?;
        let copied = self.inner.store.file_meta(&copy.id)?;
        Ok(copied.map(|meta| (copy.id, meta)))
    }

    /// Seals linked plaintext files and records this note's wrapped ownership.
    /// A file an unlocked note still shows is copied first, so that note keeps
    /// its readable original; `locking` names every note this pass seals.
    /// Existing sealed files are reused only when this session has their key.
    /// Returns the body, whose links move to any copies, and its envelope.
    fn seal_note_files(
        &self,
        note: &mut Note,
        body: &str,
        key: &LockKey,
        locking: &HashSet<String>,
        copies: &HashMap<String, String>,
    ) -> Result<(String, Envelope), KansoError> {
        let mut body = body.to_owned();
        for (from, to) in copies {
            body = body.replace(from, to);
        }
        let ids = if note
            .extra
            .contains_key(kanso_core::model::KANSO_FILES_FIELD)
        {
            note_files_from_extra(&note.extra)
                .ok_or_else(|| {
                    KansoError::invalid("The note's attachment ownership list is damaged.")
                })?
                .into_iter()
                .map(|file| file.file)
                .collect::<Vec<_>>()
        } else {
            linked_file_ids(&body)
        };
        let mut seen = HashSet::new();
        let mut sealed_files: Vec<(String, SealId, FileKey)> = Vec::new();
        for id in ids.into_iter().filter(|id| seen.insert(id.clone())) {
            let id = copies.get(&id).cloned().unwrap_or(id);
            let Some(mut meta) = self.inner.store.file_meta(&id)? else {
                continue;
            };
            let (target, seal, file_key) = match self.inner.store.attachment_state(&id)? {
                kanso_store::AttachmentState::Missing => continue,
                kanso_store::AttachmentState::Sealed => {
                    let marker = FileLock::from_extra(&meta.extra)
                        .ok_or_else(|| KansoError::invalid("The attachment seal is damaged."))?;
                    let seal = SealId::decode(&marker.seal)
                        .ok_or_else(|| KansoError::invalid("The attachment seal is damaged."))?;
                    let file_key = held(&self.inner.keys)
                        .file_key(&seal)
                        .map(|key| FileKey::from_bytes(*key.as_bytes()))
                        .ok_or(KansoError::Locked)?;
                    (id.clone(), seal, file_key)
                }
                kanso_store::AttachmentState::Plain => {
                    let target = if self.shared_with_unlocked(&id, &note.id, locking)? {
                        let Some((copy, copied)) = self.copy_plain_file(&id)? else {
                            continue;
                        };
                        body = body.replace(&id, &copy);
                        meta = copied;
                        copy
                    } else {
                        id.clone()
                    };
                    let Some((_, stored)) = self.inner.store.attachment(&target)? else {
                        continue;
                    };
                    let seal = SealId::random();
                    let file_key = FileKey::random();
                    let container = kanso_sync::lock::seal_container(
                        &target,
                        &seal,
                        &file_key,
                        &[SealedSegment {
                            kind: kanso_sync::lock::SEGMENT_CONTENT,
                            bytes: stored.data,
                        }],
                    );
                    FileLock::store(Some(&FileLock::sealed(seal.encode())), &mut meta.extra);
                    self.inner
                        .store
                        .put_sealed_file(meta, Attachment::new(container))?;
                    held(&self.inner.keys)
                        .remember_file_key(&seal, FileKey::from_bytes(*file_key.as_bytes()));
                    (target, seal, file_key)
                }
            };
            sealed_files.push((target, seal, file_key));
        }
        let envelope = key.seal(&note.id, &body);
        let ownership: Vec<NoteFile> = sealed_files
            .into_iter()
            .map(|(file, seal, file_key)| NoteFile {
                file: file.clone(),
                v: 1,
                seal: seal.encode(),
                key: STANDARD.encode(kanso_sync::lock::wrap_file_key(
                    key, &envelope, &note.id, &file, &seal, &file_key,
                )),
                extra: Default::default(),
            })
            .collect();
        store_note_files(Some(&ownership), &mut note.extra);
        Ok((body, envelope))
    }

    /// Forgets the keys, and tells listeners if any were held. The notes
    /// open for editing are saved, sealed, before the keys go, and their
    /// text is wiped from memory after.
    fn close(&self, reason: LockReason) {
        let editing = self.inner.editors.begin_lock();
        let was_open = {
            let mut keys = held(&self.inner.keys);
            // Under the mutex, so an unlock that began earlier and applies
            // later sees the change and doesn't reopen the notes.
            self.inner.lock_gen.fetch_add(1, Ordering::SeqCst);
            let was_open = keys.is_unlocked();
            keys.replace(None, Instant::now());
            was_open
        };
        Editors::finish_lock(editing);
        if was_open {
            self.inner.lock_events.send(LockEvent::Locked { reason });
        }
    }
}

/// Every file a note points at, whether its text is available or only its
/// sealed ownership list is.
fn note_file_ids(note: &Note) -> Vec<String> {
    if note
        .extra
        .contains_key(kanso_core::model::KANSO_FILES_FIELD)
    {
        note_files_from_extra(&note.extra)
            .map(|files| files.into_iter().map(|file| file.file).collect())
            .unwrap_or_default()
    } else {
        linked_file_ids(&note.body)
    }
}

#[cfg(test)]
mod tests {
    use kanso_sync::{MemorySecrets, MemoryServer};
    use proptest::prelude::*;

    use super::*;
    use crate::biometrics::{self, answer, status};
    use crate::{Library, NewNote};

    const PASSWORD: &str = "open sesame";
    const NEW_PASSWORD: &str = "another password";
    const B: [u8; 32] = [9; 32];

    /// A library with one note locked with `PASSWORD`, the notes now closed.
    fn closed(dir: &std::path::Path) -> (Library, String) {
        let library = Library::open_for_test(
            &dir.join("library.db"),
            Arc::new(MemorySecrets::new()),
            Arc::new(MemoryServer::new()),
        )
        .unwrap();
        let id = library
            .create(NewNote {
                notebook_id: None,
                title: "Plan".into(),
                body: "the plan".into(),
                ..Default::default()
            })
            .unwrap()
            .id;
        let locks = library.locks();
        locks.lock_note(&id, PASSWORD.into()).unwrap();
        locks.lock().unwrap();
        (library, id)
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(6))]

        /// A password unlock, a password change and a biometric unlock that
        /// begin, then have the notes locked any number of times before
        /// they apply, leave the notes closed; with no lock between they
        /// open them.
        #[test]
        fn prop_a_lock_between_begin_and_apply_keeps_the_notes_closed(
            locks_between in 0usize..3,
        ) {
            let dir = tempfile::tempdir().unwrap();
            let (library, _) = closed(dir.path());
            let locks = library.locks();
            let interleave = || {
                for _ in 0..locks_between {
                    locks.lock().unwrap();
                }
            };
            let expect_open = locks_between == 0;

            // Password unlock.
            let target = lock::any_lock(&library.inner.store).unwrap();
            let password = LockPassword::new(PASSWORD).unwrap();
            let since = locks.generation();
            interleave();
            let result = locks.try_password_since(&target, &password, since);
            if expect_open {
                prop_assert_eq!(result, Ok(true));
            } else {
                prop_assert_eq!(result, Err(KansoError::Locked));
            }
            prop_assert_eq!(locks.is_unlocked().unwrap(), expect_open);
            locks.lock().unwrap();

            // Biometric unlock.
            let ticket = {
                locks.unlock(PASSWORD.into()).unwrap();
                let enrolling = locks.begin_biometric_enrollment().unwrap();
                prop_assert_eq!(
                    biometrics::deliver(enrolling, status::OK, Some(&B)),
                    answer::APPLIED
                );
                locks.lock().unwrap();
                locks.begin_biometric_unlock().unwrap().unwrap()
            };
            interleave();
            let reply = biometrics::deliver(ticket, status::OK, Some(&B));
            prop_assert_eq!(
                reply,
                if expect_open { answer::APPLIED } else { answer::FAULT }
            );
            prop_assert_eq!(locks.is_unlocked().unwrap(), expect_open);
            locks.lock().unwrap();

            // Password change: the notes are re-sealed under the new
            // password either way; the new key is held only if no lock
            // came between.
            let since = locks.generation();
            interleave();
            let skipped = locks.change_password_since(PASSWORD.into(), NEW_PASSWORD.into(), since);
            prop_assert_eq!(skipped, Ok(0));
            prop_assert_eq!(locks.is_unlocked().unwrap(), expect_open);
            locks.lock().unwrap();
            prop_assert_eq!(locks.unlock(PASSWORD.into()), Ok(false));
            prop_assert_eq!(locks.unlock(NEW_PASSWORD.into()), Ok(true));
        }
    }

    // no-prop-test: a lock after a change that began earlier leaves biometrics
    // off, as their kept keys are the old ones.
    #[test]
    fn a_password_change_overtaken_by_a_lock_turns_biometrics_off() {
        let dir = tempfile::tempdir().unwrap();
        let (library, _) = closed(dir.path());
        let locks = library.locks();
        locks.unlock(PASSWORD.into()).unwrap();
        let ticket = locks.begin_biometric_enrollment().unwrap();
        assert_eq!(
            biometrics::deliver(ticket, status::OK, Some(&B)),
            answer::APPLIED
        );
        assert!(locks.biometrics_on().unwrap());
        let since = locks.generation();
        locks.lock().unwrap();
        locks
            .change_password_since(PASSWORD.into(), NEW_PASSWORD.into(), since)
            .unwrap();
        assert!(!locks.is_unlocked().unwrap());
        assert!(!locks.biometrics_on().unwrap());
    }
}
