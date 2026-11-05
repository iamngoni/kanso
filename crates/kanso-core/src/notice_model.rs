//! What an update notice is made of. The list and the rules that read these
//! shapes are in [`crate::notices`].

use semver::Version;
use serde::{Deserialize, Serialize};

/// Which apps show a notice.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Platforms {
    Desktop,
    Phones,
    Both,
}

/// The app a notice is being asked about.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Platform {
    Desktop,
    Phones,
}

/// A page of Settings that a notice's button opens. Each app maps these to
/// its own settings screens.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NoticePage {
    General,
    Account,
    LocalApi,
    Updates,
}

/// The button a notice carries besides "Mark as fixed" and "Dismiss".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoticeAction {
    OpenSettings(NoticePage),
    OpenUrl(&'static str),
}

/// What a notice's `applies` function looks at. It is built before anything
/// is migrated, so it can still see how things were before the update.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoticeContext {
    /// The version running now.
    pub current: Version,
    /// The version that ran before the last update, when Kanso knows it.
    pub previous: Option<Version>,
    /// `config.json` held a local API username and password.
    pub had_login: bool,
    /// This library is signed in to sync.
    pub signed_in_to_sync: bool,
    /// The person has saved the library key.
    pub library_key_saved: bool,
    /// This library is still a plain file: encrypting it didn't happen (the
    /// system's key store refused the key, or the disk was too small).
    pub library_plain: bool,
}

/// A change that breaks something people rely on, and the steps to fix what
/// broke. Notices are compiled into the app and never fetched.
#[derive(Clone, Debug)]
pub struct Notice {
    /// Stable and never reused: `api-keys-0.10`.
    pub id: &'static str,
    /// The first version that shows it.
    pub since: Version,
    pub title: &'static str,
    pub what_changed: &'static str,
    pub steps: &'static [&'static str],
    pub action: Option<NoticeAction>,
    /// Whether this library is affected. Pure: it reads only the context.
    pub applies: fn(&NoticeContext) -> bool,
    pub platforms: Platforms,
}

/// What a person did with notices, kept in `config.json` on the desktop and
/// in the library on the phones.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct NoticeState {
    /// Banner hidden on this device. The notice stays in What's new and in
    /// the palette.
    pub dismissed: Vec<String>,
    /// Banner hidden and dropped from the palette list. Stays in What's new.
    pub fixed: Vec<String>,
}

/// The notice the banner shows, and how many are waiting (the banner says
/// "1 of 2").
#[derive(Clone, Copy, Debug)]
pub struct BannerPick<'a> {
    pub notice: &'a Notice,
    pub total: usize,
}
