//! What the library keeps of shared notebooks (issue #390 › Shared
//! notebooks). `kanso-collab` reads and writes these; the store only keeps
//! them.

use super::model::SharedStatus;

/// One shared notebook in this library (`collab_books`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SharedBook {
    pub share_id: String,
    /// The notebook in this library.
    pub book_id: String,
    pub owner_id: String,
    pub owner_name: String,
    /// This account's role: `owner`, `manager`, `edit`, `comment` or `view`.
    pub role: String,
    pub status: SharedStatus,
    /// The newest epoch this device has a key for.
    pub epoch: u32,
    /// The newest member list checked back to the owner's genesis list, as
    /// JSON (`kanso-collab`'s `BookChainHead`).
    pub chain: String,
    /// The newest log sequence seen for the notebook: every stream's
    /// sequence numbers share one count.
    pub head: u64,
    /// The last counter this device wrote an envelope under.
    pub counter: u64,
    /// Sharing began and not every note has gone up yet (a crash or an
    /// offline spell stopped it): it goes on where it was.
    pub uploading: bool,
    pub updated_at: i64,
}

/// One log of a shared notebook this device keeps (`collab_book_streams`):
/// the manifest, or a note.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BookStream {
    pub share_id: String,
    /// The log's id: 22 base64url characters, all `A` for the manifest.
    pub stream: String,
    /// The library note of a note's log; `None` for the manifest.
    pub note_id: Option<String>,
    /// The CRDT's whole state.
    pub state: Vec<u8>,
    /// The last log sequence merged into `state`.
    pub applied_seq: u64,
    /// The log has this note: its first snapshot went up (or it was read
    /// from the server).
    pub on_server: bool,
    /// What the library last held of it, as JSON: `[title, text]` for a
    /// note, the fields for the manifest. A difference from the library is
    /// a local edit.
    pub base: String,
}

/// Something written on this device to a shared notebook's log that the
/// server hasn't taken yet (`collab_book_outbox`): a Yrs update, as
/// plaintext, sealed when it is sent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BookOutgoing {
    pub id: i64,
    pub share_id: String,
    pub stream: String,
    pub payload: Vec<u8>,
    pub created_at: i64,
}
