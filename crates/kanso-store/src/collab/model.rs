//! What the library keeps of shared notes (docs/ACCOUNTS.md › Collaboration
//! › Rust crates). `kanso-collab` reads and writes these; the store only
//! keeps them.

/// Where a shared note stands for this account.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SharedStatus {
    /// Shared, and this account is in it.
    Active,
    /// This account was removed (or left): the note is a private copy now.
    Removed,
    /// Its owner stopped sharing it: a private copy too.
    Stopped,
}

impl SharedStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            SharedStatus::Active => "active",
            SharedStatus::Removed => "removed",
            SharedStatus::Stopped => "stopped",
        }
    }

    pub fn parse(text: &str) -> Option<SharedStatus> {
        [
            SharedStatus::Active,
            SharedStatus::Removed,
            SharedStatus::Stopped,
        ]
        .into_iter()
        .find(|s| s.as_str() == text)
    }
}

/// What a shared note is: the kind of CRDT its text lives in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SharedKind {
    /// Markdown: a title and a body, each a text.
    #[default]
    Text,
    /// A canvas (`doctype: "canvas"`): cards, arrows and ink, each an item.
    Canvas,
}

impl SharedKind {
    pub fn as_str(self) -> &'static str {
        match self {
            SharedKind::Text => "text",
            SharedKind::Canvas => "canvas",
        }
    }

    pub fn parse(text: &str) -> SharedKind {
        if text == "canvas" {
            SharedKind::Canvas
        } else {
            SharedKind::Text
        }
    }
}

/// One shared note in this library (`collab_notes`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SharedNote {
    pub share_id: String,
    pub note_id: String,
    pub owner_id: String,
    pub owner_name: String,
    /// This account's role: `owner`, `edit`, `comment` or `view`.
    pub role: String,
    pub status: SharedStatus,
    /// Text or canvas, decided when the note is shared and never changed.
    pub kind: SharedKind,
    /// The newest epoch this device has a key for.
    pub epoch: u32,
    /// That epoch's member list and its signature, as received (base64url).
    pub list: String,
    pub list_sig: String,
    /// The CRDT's whole state.
    pub state: Vec<u8>,
    /// The last log sequence merged into `state`.
    pub applied_seq: u64,
    /// Updates, and their bytes, since the last snapshot the log holds.
    pub since_snapshot: u64,
    pub bytes_since_snapshot: u64,
    /// The last counter this device wrote an envelope under.
    pub counter: u64,
    /// The owner's device started an epoch whose snapshot isn't written yet.
    pub needs_snapshot: bool,
    pub updated_at: i64,
}

/// A note key as the library keeps it (`collab_keys`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SealedKey {
    pub epoch: u32,
    /// Sealed under `vault_key`.
    pub sealed: Vec<u8>,
    /// The epoch's member list and signature (base64url), when seen.
    pub list: Option<String>,
    pub list_sig: Option<String>,
}

/// Something written on this device that the server hasn't taken yet
/// (`collab_outbox`): a Yrs update (`u`) or a comment event (`c`), as
/// plaintext, sealed when it is sent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outgoing {
    pub id: i64,
    pub share_id: String,
    pub kind: String,
    pub payload: Vec<u8>,
    pub created_at: i64,
}

/// A comment event as the log delivered it, opened (`collab_comment_events`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommentRecord {
    pub seq: u64,
    pub author: String,
    pub author_name: String,
    /// The author's role in the epoch the event was written in.
    pub role: String,
    /// The event's JSON.
    pub json: String,
}

/// Someone's public keys, pinned the first time this library saw them
/// (`collab_contacts`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Contact {
    pub account_id: String,
    pub username: String,
    pub sign_pub: String,
    pub box_pub: String,
    /// When this account compared fingerprints and marked them verified.
    pub verified_at: Option<i64>,
    pub seen_at: i64,
}

/// A mention notification, opened (`collab_notices`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoredNotice {
    pub id: String,
    pub share_id: String,
    pub comment_id: String,
    pub from_id: String,
    pub from_name: String,
    pub title: String,
    pub excerpt: String,
    pub at: i64,
    pub read: bool,
}
