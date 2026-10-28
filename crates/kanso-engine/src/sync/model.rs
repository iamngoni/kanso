//! The plain data the sync words are written over (docs/ACCOUNTS.md ›
//! Services): how a status reads at a glance, the account as the app
//! shows it, and a status line.

use kanso_sync::{SignOutReason, Step};

/// How a status line reads at a glance, as the color of its dot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    Good,
    Busy,
    Quiet,
    Warning,
}

/// A status line for the top of a page: a colored dot with a few words, and
/// a sentence that says what it means or what to do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusLine {
    pub tone: Tone,
    pub label: String,
    pub detail: String,
}

/// The account as the app shows it (docs/ACCOUNTS.md › Services).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum AccountState {
    #[default]
    SignedOut,
    SignedIn {
        username: String,
    },
    /// Provider identity is verified, but this device has not obtained the
    /// library key. `Services.session` deliberately remains empty here.
    SignedInLocked {
        username: String,
    },
    /// The server ended this device's session (a 401): the Sign in form
    /// shows under the reason.
    SessionEnded {
        username: String,
        reason: SignOutReason,
    },
    /// An account task is running, at this step.
    Busy(Step),
}
