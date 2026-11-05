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
use kanso_core::model::{note_files_from_extra, store_note_files};
use kanso_core::{Note, now_ms};
use kanso_engine::links::tags::apply_inline_tags;
use kanso_engine::lock::files::migrate;
use kanso_engine::lock::files::{self, Job, Progress};
use kanso_engine::lock::{self, LockView, Target};
use kanso_sync::lock::{Envelope, LockError, LockKey, LockPassword, SealId};

/// Whether the notice about notes locked before attachments were sealed is
/// waiting (`pending`) or was said (`said`) on this phone.
const FILES_NOTICE_META: &str = "locks.files_notice";

/// `later` once the first-launch prompt to seal the attachments of notes
/// locked before was answered with "Later".
const MIGRATION_META: &str = "locks.files_migration";

use crate::boundary::{guard, held};
use crate::editing::Editors;
use crate::errors::KansoError;
use crate::library::{Inner, Library};
use crate::model::{
    FilesMigration, InterruptedLock, LockEvent, LockReason, LockStatus, LockSummary,
    MigrationOutcome, Saved, SealProgress,
};

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
        // The session's lock is released before the files are sealed, which
        // reads its file keys again.
        let key = {
            let keys = held(&self.inner.keys);
            target
                .as_ref()
                .and_then(|target| target.envelope.as_ref().ok())
                .and_then(|envelope| keys.fitting(envelope))
                .or_else(|| keys.current())
                .ok_or(KansoError::Locked)?
        };
        let body = note.body.clone();
        let locking = HashSet::from([note.id.clone()]);
        let mut copies = HashMap::new();
        let (body, envelope) = self.seal_note_files(note, &body, &key, &locking, &mut copies)?;
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
            self.inner.metrics().count(crate::metrics::Feature::Lock);
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
            let saved = self.seal_note(note, &key)?;
            self.inner.metrics().count(crate::metrics::Feature::Lock);
            Ok(saved)
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
            let envelope = Envelope::from_extra(&note.extra)
                .and_then(Result::ok)
                .ok_or(KansoError::Locked)?;
            let key = held(&self.inner.keys)
                .fitting(&envelope)
                .ok_or(KansoError::Locked)?;
            // Each file is decrypted back, or copied when another locked note
            // still shows it, so that note keeps its sealed copy.
            let events = &self.inner.seal_progress;
            let mut watch = |progress: &Progress| {
                events.send(SealProgress::from(*progress));
                Ok(())
            };
            let (body, _) = files::unseal_or_copy(&self.inner.store, &mut note, &key, &mut watch)
                .map_err(files_error)?;
            note.body = body;
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
            self.inner.lock_cancelled.store(false, Ordering::SeqCst);
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
            self.inner.metrics().count(crate::metrics::Feature::Lock);
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
        for mut note in notes {
            if note.is_locked() {
                continue;
            }
            let body = note.body.clone();
            let (_, envelope) =
                self.seal_note_files(&mut note, &body, key, &locking, &mut copies)?;
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

    /// Calls `sink` as locking a note or notebook seals its pictures and
    /// files, until it returns `false`.
    pub fn lock_progress(&self, sink: impl FnMut(SealProgress) -> bool + Send + 'static) {
        self.inner.seal_progress.add(sink);
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
        // A new lock starts: an earlier Cancel does not carry over.
        self.inner.lock_cancelled.store(false, Ordering::SeqCst);
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
        let mut copies = HashMap::new();
        let (_, envelope) = self.seal_note_files(&mut note, &body, key, &locking, &mut copies)?;
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

    /// Seals the attachments of a note locked by an older Kanso, whose
    /// `kansoFiles` list is missing, now that its key is held. The note's
    /// text is already open in `note.body`; the returned count is what was
    /// sealed. No-op once the list exists.
    pub(crate) fn migrate_note_attachments(&self, note: &mut Note) -> Result<u32, KansoError> {
        if !note.is_locked()
            || note
                .extra
                .contains_key(kanso_core::model::KANSO_FILES_FIELD)
        {
            return Ok(0);
        }
        let Some(Ok(envelope)) = Envelope::from_extra(&note.extra) else {
            return Ok(0);
        };
        let key = held(&self.inner.keys)
            .fitting(&envelope)
            .ok_or(KansoError::Locked)?;
        let body = note.body.clone();
        let locking = HashSet::from([note.id.clone()]);
        let mut copies = HashMap::new();
        let (body, envelope) = self.seal_note_files(note, &body, &key, &locking, &mut copies)?;
        note.body = body;
        Envelope::store(Some(&envelope), &mut note.extra);
        // The first time, the phone says it once (`take_sealed_files_notice`).
        if self.inner.store.meta(FILES_NOTICE_META)?.is_none() {
            self.inner.store.set_meta(FILES_NOTICE_META, "pending")?;
        }
        let count = note_files_from_extra(&note.extra)
            .map(|files| u32::try_from(files.len()).unwrap_or(u32::MAX))
            .unwrap_or_default();
        let mut stored = note.clone();
        stored.body.clear();
        let value =
            serde_json::to_value(stored).map_err(|err| KansoError::storage(err.to_string()))?;
        let written = self.inner.store.lock_note(value)?;
        note.rev = Some(written.rev);
        Ok(count)
    }

    /// Seals linked plaintext files and records this note's wrapped ownership,
    /// the way the desktop does (`kanso_engine::lock::files`): the wraps go to
    /// the note while it is plain, then the files are sealed one by one, and
    /// the caller locks the note last. A file an unlocked note still shows is
    /// copied first, so that note keeps its readable original; `locking`
    /// names every note this pass seals and `copies` the copies it made.
    /// Returns the text, whose links move to any copies, and the note's lock.
    fn seal_note_files(
        &self,
        note: &mut Note,
        body: &str,
        key: &LockKey,
        locking: &HashSet<String>,
        copies: &mut HashMap<String, String>,
    ) -> Result<(String, Envelope), KansoError> {
        let keys = held(&self.inner.keys).file_keys();
        let events = &self.inner.seal_progress;
        let cancelled = &self.inner.lock_cancelled;
        let mut watch = |progress: &Progress| {
            events.send(SealProgress::from(*progress));
            if cancelled.load(Ordering::SeqCst) {
                return Err("Locking was cancelled.".to_owned());
            }
            Ok(())
        };
        let mut job = Job::new(&self.inner.store, &keys, locking, copies, &mut watch);
        let (body, _) = files::seal_note_files(&mut job, note, body, key).map_err(files_error)?;
        let envelope = Envelope::from_extra(&note.extra)
            .and_then(Result::ok)
            .ok_or_else(|| KansoError::invalid("The note's lock is damaged."))?;
        Ok((body, envelope))
    }

    /// What locking note `id` would seal.
    pub fn note_lock_summary(&self, id: &str) -> Result<LockSummary, KansoError> {
        guard(|| {
            let note = self.library().stored(id)?;
            let notes = [note];
            Ok(files::summarize(&self.inner.store, &notes).into())
        })
    }

    /// What locking notebook `book_id`, and the notebooks in it, would seal.
    pub fn notebook_lock_summary(&self, book_id: &str) -> Result<LockSummary, KansoError> {
        guard(|| {
            self.require_book(book_id)?;
            Ok(lock::notebook_summary(&self.inner.store, book_id).into())
        })
    }

    /// Stops the lock in progress between two files. The note stays plain,
    /// and [`LockSession::interrupted_lock`] then offers to finish or undo
    /// it. A lock that is not running is not affected.
    pub fn cancel_lock(&self) {
        self.inner.lock_cancelled.store(true, Ordering::SeqCst);
    }

    /// Whether the first open of a note locked by an older Kanso sealed its
    /// attachments since this was last asked: the phone says so once.
    pub fn take_sealed_files_notice(&self) -> Result<bool, KansoError> {
        guard(|| {
            let pending = self.inner.store.meta(FILES_NOTICE_META)?.as_deref() == Some("pending");
            if pending {
                self.inner.store.set_meta(FILES_NOTICE_META, "said")?;
            }
            Ok(pending)
        })
    }

    /// The notes locked before attachments were sealed, for the first-launch
    /// prompt, the Settings row and the quiet reminder.
    pub fn files_migration(&self) -> Result<FilesMigration, KansoError> {
        guard(|| {
            let store = &self.inner.store;
            let pending = migrate::pending(store);
            let keys = held(&self.inner.keys).keys().to_vec();
            let scan =
                (!pending.is_done() && !keys.is_empty()).then(|| migrate::scan(store, &keys));
            let count = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
            let mut state = FilesMigration {
                notes: count(pending.notes()),
                stopped: count(pending.stopped),
                postponed: store.meta(MIGRATION_META)?.as_deref() == Some("later"),
                ..FilesMigration::default()
            };
            if let Some(scan) = scan.filter(|scan| scan.notes > 0 && scan.unopened == 0) {
                state.counted = true;
                state.files = count(scan.summary.files);
                state.bytes = scan.summary.bytes;
                state.shared = count(scan.summary.shared);
            }
            Ok(state)
        })
    }

    /// "Later" at the first-launch prompt: it isn't asked again, and the
    /// reminder stays until the attachments are sealed.
    pub fn postpone_files_migration(&self) -> Result<(), KansoError> {
        guard(|| {
            self.inner.store.set_meta(MIGRATION_META, "later")?;
            Ok(())
        })
    }

    /// Seals the attachments of every locked note a held key, or `password`,
    /// opens: a file an unlocked note also shows is copied first. Progress
    /// goes to [`LockSession::lock_progress`]; [`LockSession::cancel_lock`]
    /// stops it between two files and running it again finishes. Without a
    /// `password` a key must be held ([`KansoError::Locked`] otherwise); a
    /// password that opens no waiting note is [`KansoError::Auth`].
    pub fn seal_locked_attachments(
        &self,
        password: Option<String>,
    ) -> Result<MigrationOutcome, KansoError> {
        guard(|| {
            self.inner.lock_cancelled.store(false, Ordering::SeqCst);
            let since = self.generation();
            self.add_password_keys(password, since)?;
            let keys = held(&self.inner.keys).keys().to_vec();
            if keys.is_empty() {
                return Err(KansoError::Locked);
            }
            let book = held(&self.inner.keys).file_keys();
            let events = &self.inner.seal_progress;
            let cancelled = &self.inner.lock_cancelled;
            let mut watch = |progress: &Progress| {
                events.send(SealProgress::from(*progress));
                if cancelled.load(Ordering::SeqCst) {
                    return Err("Sealing was cancelled.".to_owned());
                }
                Ok(())
            };
            let outcome = migrate::migrate(&self.inner.store, &book, &keys, &mut watch);
            held(&self.inner.keys).touch(Instant::now());
            // The notes now list their files: their keys are held for reading.
            self.cache_note_file_keys(since);
            Ok(MigrationOutcome::from(&outcome))
        })
    }

    /// Undoes a stopped sealing pass: the files it sealed are readable files
    /// again. The held keys, or `password`, must open the notes. Returns how
    /// many files it undid.
    pub fn undo_files_migration(&self, password: Option<String>) -> Result<u32, KansoError> {
        guard(|| {
            let since = self.generation();
            self.add_password_keys(password, since)?;
            let keys = held(&self.inner.keys).keys().to_vec();
            if keys.is_empty() {
                return Err(KansoError::Locked);
            }
            let (undone, unopened) =
                migrate::undo_all(&self.inner.store, &keys).map_err(files_error)?;
            if undone == 0 && unopened > 0 {
                return Err(KansoError::auth(lock::sentence(&LockError::WrongPassword)));
            }
            Ok(u32::try_from(undone).unwrap_or(u32::MAX))
        })
    }

    /// Keeps the keys `password` gives for the waiting notes' locks.
    fn add_password_keys(&self, password: Option<String>, since: u64) -> Result<(), KansoError> {
        let Some(text) = password else {
            return Ok(());
        };
        let password = lock::password(&text).map_err(KansoError::invalid)?;
        let keys = migrate::keys_for(&self.inner.store, &password).map_err(KansoError::auth)?;
        for key in keys {
            self.add_key(key, since);
        }
        Ok(())
    }

    /// Whether note `id` carries a lock that stopped between its files (a
    /// crash, or Cancel): it is plain, some of its pictures and files are
    /// sealed, and it waits to be finished or undone.
    pub fn interrupted_lock(&self, id: &str) -> Result<Option<InterruptedLock>, KansoError> {
        guard(|| {
            let note = self.library().stored(id)?;
            Ok(
                files::interrupted(&self.inner.store, &note).map(|stopped| InterruptedLock {
                    files: u32::try_from(stopped.files).unwrap_or(u32::MAX),
                    sealed: u32::try_from(stopped.sealed).unwrap_or(u32::MAX),
                }),
            )
        })
    }

    /// Finishes the lock that stopped on note `id`. The session's key for it
    /// is used when held; else `password` (the lock password the stopped lock
    /// used) opens it ([`KansoError::Auth`] when it isn't).
    pub fn finish_locking(&self, id: &str, password: Option<String>) -> Result<Saved, KansoError> {
        guard(|| {
            let note = self.lockable(id)?;
            let key = self.key_for_interrupted(&note, password)?;
            self.seal_note(note, &key)
        })
    }

    /// Undoes the lock that stopped on note `id`: the files it sealed are
    /// decrypted again and the note stays as it was.
    pub fn undo_locking(&self, id: &str, password: Option<String>) -> Result<(), KansoError> {
        guard(|| {
            let note = self.lockable(id)?;
            let key = self.key_for_interrupted(&note, password)?;
            files::undo_locking(&self.inner.store, &note, &key)
                .map(drop)
                .map_err(files_error)
        })
    }

    /// The key of the lock that stopped on `note`: the session's, else the
    /// one `password` gives for the lock's own header.
    fn key_for_interrupted(
        &self,
        note: &Note,
        password: Option<String>,
    ) -> Result<Arc<LockKey>, KansoError> {
        let header = files::interrupted_header(note)
            .ok_or_else(|| KansoError::invalid("That note has no lock to finish."))?;
        if let Some(key) = held(&self.inner.keys).fitting(&header) {
            return Ok(key);
        }
        let password = password.ok_or(KansoError::Locked)?;
        let password = lock::password(&password).map_err(KansoError::invalid)?;
        let wrong = || KansoError::auth(lock::sentence(&LockError::WrongPassword));
        let key = LockKey::for_envelope(&password, &header).map_err(|_| wrong())?;
        if !files::opens_interrupted(note, &key) {
            return Err(wrong());
        }
        let since = self.generation();
        self.add_key(key, since);
        held(&self.inner.keys)
            .fitting(&header)
            .ok_or(KansoError::Locked)
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

/// A files error as the facade's.
fn files_error(err: files::FilesError) -> KansoError {
    match err {
        files::FilesError::NeedsUnlock { .. } => KansoError::Locked,
        other => KansoError::invalid(other.to_string()),
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
