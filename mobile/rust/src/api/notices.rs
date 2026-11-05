//! Update notices: the card at the top of the notes list when a change in
//! an update affects this phone (issue #243).

use kanso_mobile::{KansoError, PhoneNotices};

use super::library::KansoLibrary;

impl KansoLibrary {
    /// The card to show now. `signed_in_to_sync` is whether this phone has
    /// an account.
    pub fn update_notices(&self, signed_in_to_sync: bool) -> Result<PhoneNotices, KansoError> {
        self.0.notices(signed_in_to_sync)
    }

    /// Hides the card on this phone for good.
    pub fn dismiss_update_notice(&self, id: String) -> Result<(), KansoError> {
        self.0.dismiss_notice(&id)
    }

    /// Hides the card because the person fixed what it said.
    pub fn mark_update_notice_fixed(&self, id: String) -> Result<(), KansoError> {
        self.0.mark_notice_fixed(&id)
    }
}
