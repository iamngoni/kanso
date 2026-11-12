//! What's new, once after an update: the release's video and notes.

use kanso_mobile::{KansoError, PhoneWhatsNew};

use super::library::KansoLibrary;

impl KansoLibrary {
    /// The What's new sheet to show now, or none. Reads the release's
    /// signed feed and fetches its video into `cache_dir`.
    pub fn whats_new(&self, cache_dir: String) -> Result<Option<PhoneWhatsNew>, KansoError> {
        self.0.whats_new(&cache_dir)
    }

    /// Notes that this version's What's new was shown.
    pub fn mark_whats_new_seen(&self) -> Result<(), KansoError> {
        self.0.mark_whats_new_seen()
    }
}
