//! The lock session: the lock password's keys stay in Rust.

use crate::frb_generated::StreamSink;
use flutter_rust_bridge::frb;
use kanso_mobile::{KansoError, LockEvent, LockSession, LockStatus, Saved};

#[frb(opaque)]
pub struct KansoLocks(LockSession);

impl KansoLocks {
    pub(super) fn new(session: LockSession) -> KansoLocks {
        KansoLocks(session)
    }

    pub fn unlock(&self, password: String) -> Result<bool, KansoError> {
        self.0.unlock(password)
    }

    pub fn lock(&self) -> Result<(), KansoError> {
        self.0.lock()
    }

    pub fn is_unlocked(&self) -> Result<bool, KansoError> {
        self.0.is_unlocked()
    }

    pub fn is_note_unlocked(&self, id: String) -> Result<bool, KansoError> {
        self.0.is_note_unlocked(&id)
    }

    pub fn lock_note(&self, id: String, password: String) -> Result<Saved, KansoError> {
        self.0.lock_note(&id, password)
    }

    pub fn lock_for_background(&self) -> Result<(), KansoError> {
        self.0.lock_for_background()
    }

    pub fn lock_status(&self) -> Result<LockStatus, KansoError> {
        self.0.lock_status()
    }

    pub fn unlock_note(&self, id: String, password: String) -> Result<bool, KansoError> {
        self.0.unlock_note(&id, password)
    }

    pub fn unlock_notebook(&self, book_id: String, password: String) -> Result<bool, KansoError> {
        self.0.unlock_notebook(&book_id, password)
    }

    pub fn lock_note_with_session(&self, id: String) -> Result<Saved, KansoError> {
        self.0.lock_note_with_session(&id)
    }

    pub fn remove_note_lock(&self, id: String) -> Result<Saved, KansoError> {
        self.0.remove_note_lock(&id)
    }

    pub fn lock_notebook(
        &self,
        book_id: String,
        password: Option<String>,
    ) -> Result<u32, KansoError> {
        self.0.lock_notebook(&book_id, password)
    }

    pub fn remove_notebook_lock(&self, book_id: String) -> Result<(), KansoError> {
        self.0.remove_notebook_lock(&book_id)
    }

    pub fn notebook_is_open(&self, book_id: String) -> Result<bool, KansoError> {
        self.0.notebook_is_open(&book_id)
    }

    pub fn change_password(&self, old: String, new: String) -> Result<u32, KansoError> {
        self.0.change_password(old, new)
    }

    /// The library's ID, which names the platform's biometric key or item.
    pub fn library_id(&self) -> Result<String, KansoError> {
        self.0.library_id()
    }

    pub fn biometrics_on(&self) -> Result<bool, KansoError> {
        self.0.biometrics_on()
    }

    pub fn begin_biometric_enrollment(&self) -> Result<u64, KansoError> {
        self.0.begin_biometric_enrollment()
    }

    pub fn begin_biometric_unlock(&self) -> Result<Option<u64>, KansoError> {
        self.0.begin_biometric_unlock()
    }

    pub fn turn_off_biometrics(&self) -> Result<(), KansoError> {
        self.0.turn_off_biometrics()
    }

    /// Each time the notes open or close, until Dart stops listening.
    pub fn lock_events(&self, sink: StreamSink<LockEvent>) {
        self.0.lock_events(move |event| sink.add(event).is_ok());
    }
}
