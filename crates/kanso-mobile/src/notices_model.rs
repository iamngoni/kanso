//! What the phone's notice card is handed. The list, the wording and the
//! rules are `kanso_core::notices`, which the desktop runs too; these types
//! are the card with its words already chosen, so the screen draws and does
//! not decide.

/// What a notice's button does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhoneNoticeActionKind {
    /// The notice has no button besides Dismiss and Mark as fixed.
    None,
    /// Opens a page of Settings, named by `action_target`.
    OpenSettings,
    /// Opens `action_target` in the browser.
    OpenUrl,
}

/// One notice, as the card and its How to fix sheet show it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhoneNotice {
    /// Stable; what Dismiss and Mark as fixed are given.
    pub id: String,
    pub title: String,
    pub what_changed: String,
    pub steps: Vec<String>,
    pub action_kind: PhoneNoticeActionKind,
    /// "Open Settings" or "Learn more"; empty without a button.
    pub action_label: String,
    /// The page (`general`, `account`, `local-api`, `updates`) or the
    /// address; empty without a button.
    pub action_target: String,
}

/// What the top of the notes list shows: the newest notice that applies to
/// this phone and hasn't been put away.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PhoneNotices {
    pub card: Option<PhoneNotice>,
    /// How many are waiting, the card's included.
    pub total: u32,
    /// "1 of 2" when more than one is waiting, otherwise empty.
    pub count_label: String,
}
