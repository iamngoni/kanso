//! What the sync status line and the sync settings hand the shell: the words
//! of the desktop's Settings › Account status, the account's state they are
//! told over, and the choices for how often sync runs.
//!
//! Plain data. The words themselves are `kanso_engine::sync::words`, the
//! same code the desktop calls.

/// How a status line reads at a glance, as the color of its dot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyncTone {
    Good,
    Busy,
    Quiet,
    Warning,
}

/// The status line: a dot's tone, a few words, and a sentence that says
/// what they mean or what to do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyncStatusLine {
    pub tone: SyncTone,
    pub label: String,
    pub detail: String,
}

/// Why a device was signed out (docs/ACCOUNTS.md › Problems).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SignOutReasonKind {
    SignedOut,
    Revoked,
    PasswordChanged,
    Recovered,
    Evicted,
    Expired,
    Rekeyed,
    Unknown,
}

/// What a full account has no room for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuotaLimit {
    Docs,
    DocBytes,
    Files,
    FileBytes,
    Server,
}

/// What an account task that is running is doing, for the status line's
/// words while it runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccountStep {
    StoppingSync,
    DerivingKeys,
    Contacting,
    Saving,
}

/// Where the account stands, as the status line tells it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SyncAccount {
    /// No account is signed in.
    #[default]
    SignedOut,
    SignedIn,
    /// Signed in, but this phone has no data key yet: it waits for a
    /// device's approval, the password or the recovery key.
    Locked,
    /// The server ended this phone's session; the reason is in the
    /// status's `signed_out_reason`.
    SessionEnded,
    /// An account task is running, at this step. The shell sets this while
    /// its own sign-in or sign-up call runs; the status never reports it.
    Busy(AccountStep),
}

/// How the sync settings stand: whether Kanso syncs on its own, and how
/// often.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SyncSettingsRow {
    /// Sync when the app comes to the front, soon after an edit and on the
    /// interval. Sync now always works.
    pub enabled: bool,
    /// One of the options ([`SyncIntervalOption`]) in seconds.
    pub interval_secs: u32,
}

impl Default for SyncSettingsRow {
    fn default() -> SyncSettingsRow {
        SyncSettingsRow {
            enabled: true,
            interval_secs: 300,
        }
    }
}

/// A choice for how often sync runs, with the desktop's words for it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyncIntervalOption {
    pub minutes: u32,
    /// "1 minute", "15 minutes", "1 hour", "4 hours".
    pub label: String,
}
