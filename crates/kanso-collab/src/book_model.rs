//! The shapes shared notebooks' keys are made of (docs/ACCOUNTS.md ›
//! Collaboration; issue #390 › Shared notebooks: keys). Logic lives in
//! [`crate::book`].

use std::collections::BTreeMap;
use std::fmt;

use zeroize::Zeroizing;

/// A notebook's key for one epoch, `K_b[e]`. Every note's key and the
/// manifest's key derive from it. Zeroized on drop; `Debug` shows nothing.
#[derive(Clone)]
pub struct BookKey(pub(crate) Zeroizing<[u8; 32]>);

impl fmt::Debug for BookKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("BookKey(<redacted>)")
    }
}

impl PartialEq for BookKey {
    fn eq(&self, other: &BookKey) -> bool {
        self.0[..] == other.0[..]
    }
}

impl BookKey {
    pub fn from_bytes(bytes: [u8; 32]) -> BookKey {
        BookKey(Zeroizing::new(bytes))
    }

    pub fn bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// The keys of a notebook's earlier epochs, as far as a member's history
/// links reach, by epoch number.
#[derive(Debug, Default, PartialEq)]
pub struct BookHistory {
    pub keys: BTreeMap<u32, BookKey>,
}

/// What a device keeps of a notebook's member-list chain: the newest list
/// checked back to the owner's genesis list, and the last list of every
/// epoch (who held which role then, and where the next epoch began), which
/// is what decides whether an entry written under that epoch counts.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct StoredChain {
    pub head: crate::book_members_model::BookChainHead,
    pub epochs: BTreeMap<u32, crate::book_members_model::BookMemberList>,
}

/// Progress of sharing a notebook: "Sharing Studio… 120 of 340".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BookProgress {
    pub done: usize,
    pub total: usize,
}

/// What sharing a notebook did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BookShared {
    pub share_id: String,
    /// Notes now in the shared notebook.
    pub shared: usize,
    /// Locked notes moved to the private "<name> (locked)" notebook.
    pub moved_locked: usize,
    /// Titles of notes left out: ones already shared on their own, and
    /// canvases (a shared notebook holds text notes for now).
    pub left_out: Vec<String>,
}

/// What one sync of a notebook did.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BookSync {
    /// Log entries pulled and entries pushed.
    pub pulled: usize,
    pub pushed: usize,
    /// Notes whose text the library took from the notebook.
    pub notes_written: usize,
    /// This account is no longer in the notebook.
    pub removed: bool,
    /// The notebook ended here during this sync (removed, or it was
    /// stopped): the private copy it left.
    pub ended: Option<BookEnded>,
}

/// Why a notebook stopped being shared on this device.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BookEndReason {
    /// An owner or manager removed this account.
    Removed,
    /// This account left.
    Left,
    /// The owner stopped sharing.
    Stopped,
    /// The server no longer shows the notebook to this account: it was
    /// stopped, or this account was removed (the server answers both with
    /// the same 404).
    Gone,
}

/// A notebook that ended here and the private copy it left.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BookEnded {
    pub share_id: String,
    pub reason: BookEndReason,
    /// The notebook's name while it was shared.
    pub name: String,
    /// The private notebook the notes are in now.
    pub book_id: String,
    /// What the private notebook is called.
    pub copy_name: String,
    /// Notes in the private copy.
    pub notes: usize,
    /// What to tell the person ([`crate::book_copy`]).
    pub message: String,
}

/// A notebook joined on this device.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JoinedBook {
    pub share_id: String,
    /// The notebook in this library.
    pub book_id: String,
    pub notes: usize,
}
