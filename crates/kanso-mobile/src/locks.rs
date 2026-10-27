//! The lock session: the lock password's keys, held in memory only.
//!
//! Typing the lock password once opens every note locked with it until the
//! notes are locked again, or the auto-lock time passes
//! ([`kanso_engine::lock::AUTO_LOCK`]) since a key was added or a locked note
//! saved. The keys never leave this crate: the shell hears only that the
//! notes are open or closed ([`LockSession::lock_events`]).

use std::sync::Arc;
use std::time::Instant;

use kanso_engine::lock::{self, Session as KeySession};
use kanso_sync::lock::{Envelope, LockError, LockKey};

use crate::boundary::{guard, held};
use crate::editing::Editors;
use crate::errors::KansoError;
use crate::library::{Inner, Library};
use crate::model::{LockEvent, LockReason, Saved};

/// The lock session of one library.
#[derive(Clone)]
pub struct LockSession {
    pub(crate) inner: Arc<Inner>,
}

impl LockSession {
    /// Opens the locked notes with `password`. `Ok(false)` when it isn't the
    /// lock password; an error when it can't be one (too short) or nothing
    /// is locked yet. Deriving the key takes a fraction of a second.
    pub fn unlock(&self, password: String) -> Result<bool, KansoError> {
        guard(|| {
            let password = lock::password(&password).map_err(KansoError::invalid)?;
            let target = lock::any_lock(&self.inner.store)
                .ok_or_else(|| KansoError::invalid("Nothing is locked yet."))?;
            let envelope = target
                .envelope
                .as_ref()
                .map_err(|err| KansoError::invalid(lock::sentence(err)))?;
            let key = match LockKey::for_envelope(&password, envelope) {
                Ok(key) => key,
                Err(err) => return Err(KansoError::invalid(lock::sentence(&err))),
            };
            match key.open(&target.id, envelope) {
                Ok(_) => {}
                Err(LockError::WrongPassword) => return Ok(false),
                Err(err) => return Err(KansoError::invalid(lock::sentence(&err))),
            }
            self.add_key(key);
            Ok(true)
        })
    }

    /// Closes every locked note. Their keys are forgotten.
    pub fn lock(&self) -> Result<(), KansoError> {
        guard(|| {
            self.close(LockReason::Manual);
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
            let library = Library {
                inner: self.inner.clone(),
            };
            let mut note = library.stored(id)?;
            if note.is_locked() || self.inner.store.note_in_locked_book(id)? {
                return Err(KansoError::invalid("That note is locked already."));
            }
            let key = match lock::any_lock(&self.inner.store) {
                Some(target) => lock::key_for(&target, &password)
                    .map_err(|_| KansoError::auth("That isn't the lock password."))?,
                None => {
                    let params = held(&self.inner.keys).params();
                    LockKey::create(&password, params)
                        .map_err(|err| KansoError::invalid(lock::sentence(&err)))?
                }
            };
            self.add_key(key);
            let key = held(&self.inner.keys)
                .current()
                .expect("a key was just added");
            let body = std::mem::take(&mut note.body);
            Envelope::store(Some(&key.seal(&note.id, &body)), &mut note.extra);
            let value =
                serde_json::to_value(&note).map_err(|err| KansoError::storage(err.to_string()))?;
            let written = self.inner.store.lock_note(value)?;
            Ok(Saved {
                id: written.id,
                rev: written.rev,
            })
        })
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

    /// Holds `key`, and tells listeners if the notes weren't open before.
    fn add_key(&self, key: LockKey) {
        let was_open = {
            let mut keys = held(&self.inner.keys);
            let was_open = keys.is_unlocked();
            keys.add(key, Instant::now());
            was_open
        };
        // Notes put in a locked notebook before a key was held are sealed
        // now.
        seal_unsealed(&self.inner.keys, &self.inner.store);
        if !was_open {
            self.inner.lock_events.send(LockEvent::Unlocked);
        }
    }

    /// Forgets the keys, and tells listeners if any were held. The notes
    /// open for editing are saved, sealed, before the keys go, and their
    /// text is wiped from memory after.
    fn close(&self, reason: LockReason) {
        let editing = self.inner.editors.begin_lock();
        let was_open = {
            let mut keys = held(&self.inner.keys);
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

fn seal_unsealed(keys: &std::sync::Mutex<KeySession>, store: &kanso_store::Store) {
    held(keys).seal_unsealed(store);
}
