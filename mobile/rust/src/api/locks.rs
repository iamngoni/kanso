//! The lock session: the lock password's keys stay in Rust.

use crate::frb_generated::StreamSink;
use flutter_rust_bridge::frb;
use kanso_mobile::{
    FilesMigration, InterruptedLock, KansoError, LockEvent, LockSession, LockStatus, LockSummary,
    MigrationOutcome, Saved, SealProgress,
};

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

    /// What locking note `id` would seal.
    pub fn note_lock_summary(&self, id: String) -> Result<LockSummary, KansoError> {
        self.0.note_lock_summary(&id)
    }

    /// What locking notebook `book_id` would seal.
    pub fn notebook_lock_summary(&self, book_id: String) -> Result<LockSummary, KansoError> {
        self.0.notebook_lock_summary(&book_id)
    }

    /// Stops the lock in progress between two files.
    pub fn cancel_lock(&self) {
        self.0.cancel_lock();
    }

    /// Whether the phone should say, once, that notes locked before this
    /// update sealed their attachments.
    pub fn take_sealed_files_notice(&self) -> Result<bool, KansoError> {
        self.0.take_sealed_files_notice()
    }

    /// How far sealing a locking note's pictures and files has come, until
    /// Dart stops listening.
    pub fn lock_progress(&self, sink: StreamSink<SealProgress>) {
        self.0
            .lock_progress(move |progress| sink.add(progress).is_ok());
    }

    /// The lock that stopped between note `id`'s files, if any: it waits to
    /// be finished or undone.
    pub fn interrupted_lock(&self, id: String) -> Result<Option<InterruptedLock>, KansoError> {
        self.0.interrupted_lock(&id)
    }

    /// Finishes the lock that stopped on note `id`; `password` is the lock
    /// password when the session doesn't hold its key.
    pub fn finish_locking(
        &self,
        id: String,
        password: Option<String>,
    ) -> Result<Saved, KansoError> {
        self.0.finish_locking(&id, password)
    }

    /// Undoes the lock that stopped on note `id`.
    pub fn undo_locking(&self, id: String, password: Option<String>) -> Result<(), KansoError> {
        self.0.undo_locking(&id, password)
    }

    /// The notes locked before attachments were sealed, and what sealing
    /// theirs would do.
    pub fn files_migration(&self) -> Result<FilesMigration, KansoError> {
        self.0.files_migration()
    }

    /// "Later" at the first-launch prompt.
    pub fn postpone_files_migration(&self) -> Result<(), KansoError> {
        self.0.postpone_files_migration()
    }

    /// Seals the attachments of every locked note the held keys, or
    /// `password`, open. Progress comes on `lock_progress`.
    pub fn seal_locked_attachments(
        &self,
        password: Option<String>,
    ) -> Result<MigrationOutcome, KansoError> {
        self.0.seal_locked_attachments(password)
    }

    /// Undoes a stopped sealing pass.
    pub fn undo_files_migration(&self, password: Option<String>) -> Result<u32, KansoError> {
        self.0.undo_files_migration(password)
    }

    /// Each time the notes open or close, until Dart stops listening.
    pub fn lock_events(&self, sink: StreamSink<LockEvent>) {
        self.0.lock_events(move |event| sink.add(event).is_ok());
    }
}
