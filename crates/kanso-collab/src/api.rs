//! The server's shared-note routes (docs/ACCOUNTS.md › Collaboration › The
//! server: routes): [`CollabApi`], with [`crate::http::HttpCollab`] for the
//! Worker and [`crate::memory::MemoryCollab`] for tests, and their request
//! and response bodies. Field names are the wire's; unknown keys are
//! ignored.

use std::fmt;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use zeroize::Zeroizing;

use crate::identity::Published;
use crate::model::{PublicIdentity, Role};

/// Who is calling: the session's account, device and server, and its
/// `Authorization` header. Built from a `kanso_sync::Session`, or directly in
/// tests.
#[derive(Clone)]
pub struct Caller {
    pub server: String,
    pub account_id: String,
    pub username: String,
    pub device_id: String,
    pub authorization: Zeroizing<String>,
}

impl fmt::Debug for Caller {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Caller")
            .field("server", &self.server)
            .field("account_id", &self.account_id)
            .field("username", &self.username)
            .field("device_id", &self.device_id)
            .finish_non_exhaustive()
    }
}

impl Caller {
    pub fn of(session: &kanso_sync::Session) -> Caller {
        Caller {
            server: session.server.clone(),
            account_id: session.account_id.clone(),
            username: session.username.clone(),
            device_id: session.device_id.clone(),
            authorization: session.authorization(),
        }
    }
}

/// Why a shared-note request or step failed.
#[derive(Debug, thiserror::Error)]
pub enum CollabError {
    /// The server's answer: a status, its error code and the whole body
    /// (some carry extras, like the current `epoch` or `head`).
    #[error("{status} {code}")]
    Api {
        status: u16,
        code: String,
        body: Value,
    },
    /// No HTTP answer (offline, DNS, TLS, timeout).
    #[error("transport: {0}")]
    Transport(String),
    /// An answer that doesn't follow the protocol.
    #[error("protocol: {0}")]
    Protocol(String),
    #[error("{0}")]
    Store(#[from] kanso_store::StoreError),
    #[error("{0}")]
    Identity(#[from] crate::identity::IdentityError),
    #[error("{0}")]
    List(#[from] crate::members::ListError),
    /// A notebook's chain of member lists didn't check out (a different
    /// history than one seen before, a forged link, a bad signature).
    #[error("{0}")]
    BookList(#[from] crate::book_members_model::BookListError),
    /// There is no Kanso account with that username (or it has no identity
    /// keys yet: it never opened sharing).
    #[error("there's no Kanso user called {0}")]
    NoSuchUser(String),
    /// The key the server gives for someone isn't the one pinned for them.
    #[error("{0}'s key changed")]
    KeyChanged(String),
    /// The note can't be shared (locked, in a locked notebook, in the trash).
    #[error("{0}")]
    NotShareable(String),
    /// What the caller asked isn't something its role allows, or the note
    /// isn't in a state that allows it.
    #[error("{0}")]
    Refused(String),
    /// A note key, wrap or envelope that doesn't open or verify.
    #[error("{0}")]
    Crypto(String),
}

impl CollabError {
    pub fn code(&self) -> Option<&str> {
        match self {
            CollabError::Api { code, .. } => Some(code),
            _ => None,
        }
    }

    pub fn status(&self) -> Option<u16> {
        match self {
            CollabError::Api { status, .. } => Some(*status),
            _ => None,
        }
    }

    /// A number in the error's body, such as `epoch` or `head`.
    pub fn extra(&self, key: &str) -> Option<u64> {
        match self {
            CollabError::Api { body, .. } => body.get(key)?.as_u64(),
            _ => None,
        }
    }

    /// A string in the error's body, such as the `account` of a
    /// `member_gone`.
    pub fn extra_str(&self, key: &str) -> Option<&str> {
        match self {
            CollabError::Api { body, .. } => body.get(key)?.as_str(),
            _ => None,
        }
    }

    pub fn api(status: u16, code: &str, body: Value) -> CollabError {
        CollabError::Api {
            status,
            code: code.to_owned(),
            body,
        }
    }

    /// Reads an error answer; a body that isn't one is a protocol error.
    pub fn from_response(status: u16, body: &[u8]) -> CollabError {
        match serde_json::from_slice::<Value>(body) {
            Ok(value) => {
                let code = value
                    .get("error")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown")
                    .to_owned();
                CollabError::Api {
                    status,
                    code,
                    body: value,
                }
            }
            Err(_) => CollabError::Protocol(format!("a {status} answer without a JSON error body")),
        }
    }

    /// Whether this means "offline": the request got no answer.
    pub fn is_offline(&self) -> bool {
        matches!(self, CollabError::Transport(_))
    }
}

pub type Result<T, E = CollabError> = std::result::Result<T, E>;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountWrap {
    pub account: String,
    pub wrapped: String,
    /// An earlier epoch's key, handed to a new member so they can read the
    /// comments written before they joined; absent for the new epoch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub epoch: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpochWrap {
    pub epoch: u32,
    pub wrapped: String,
}

/// `POST /v2/collab/shares`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewShare {
    pub share_id: String,
    pub list: String,
    pub list_sig: String,
    pub wraps: Vec<AccountWrap>,
    /// The epoch-1 snapshot envelope (base_seq 0), base64url.
    pub snapshot: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Created {
    pub share_id: String,
    pub epoch: u32,
    pub head: u64,
}

/// `POST /v2/collab/shares/<id>/epoch`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewEpoch {
    pub epoch: u32,
    pub list: String,
    pub list_sig: String,
    pub wraps: Vec<AccountWrap>,
}

/// One person's state in a share the caller owns.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemberState {
    pub account: String,
    /// `member` or `invited`.
    pub status: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinkInfo {
    pub link_id: String,
    pub role: Role,
    #[serde(default)]
    pub created_at: i64,
    pub expires_at: i64,
}

/// Someone who asked to join through a link.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JoinRequest {
    pub link_id: String,
    pub account: String,
    pub username: String,
    /// The role the link gives.
    pub role: Role,
    pub sign_pub: String,
    pub box_pub: String,
    pub binding: String,
    #[serde(rename = "requested_at")]
    pub at: i64,
}

impl JoinRequest {
    pub fn identity(&self) -> PublicIdentity {
        PublicIdentity {
            account_id: self.account.clone(),
            username: self.username.clone(),
            sign_pub: self.sign_pub.clone(),
            box_pub: self.box_pub.clone(),
            binding: self.binding.clone(),
        }
    }
}

/// One epoch's member list, as the owner signed it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpochList {
    pub epoch: u32,
    pub list: String,
    pub list_sig: String,
}

/// A share the caller is in or invited to (`GET /v2/collab/shares`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShareInfo {
    pub share_id: String,
    pub owner: String,
    pub owner_username: String,
    pub role: Role,
    /// `member` or `invited`.
    pub status: String,
    pub epoch: u32,
    pub list: String,
    pub list_sig: String,
    /// The caller's wraps, one per epoch it was in.
    pub wraps: Vec<EpochWrap>,
    /// The list of every epoch the caller has a wrap for, and the current
    /// one, oldest first.
    #[serde(default)]
    pub lists: Vec<EpochList>,
    pub head: u64,
    /// The owner's view only.
    #[serde(default)]
    pub members: Vec<MemberState>,
    #[serde(default)]
    pub links: Vec<LinkInfo>,
    #[serde(default)]
    pub requests: Vec<JoinRequest>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharesPage {
    pub shares: Vec<ShareInfo>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotEntry {
    pub seq: u64,
    pub base_seq: u64,
    /// Base64url.
    pub envelope: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogEntry {
    pub seq: u64,
    /// Base64url.
    pub envelope: String,
}

/// `GET /v2/collab/shares/<id>/log`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogPage {
    pub head: u64,
    #[serde(default)]
    pub snapshot: Option<SnapshotEntry>,
    pub entries: Vec<LogEntry>,
    #[serde(default)]
    pub more: bool,
}

/// The manifest's stream id in a notebook's logs: 16 zero bytes as base64url.
pub const MANIFEST_STREAM: &str = "AAAAAAAAAAAAAAAAAAAAAA";

/// `POST /v2/collab/books`: a notebook at epoch 1, with the owner's signed
/// genesis list and one wrap of the notebook key for each listed member.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewBook {
    pub share_id: String,
    pub list: String,
    pub list_sig: String,
    pub wraps: Vec<AccountWrap>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BookCreated {
    pub share_id: String,
    pub epoch: u32,
    pub rev: u32,
}

/// One list of a notebook's chain, as the server stores it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BookListEntry {
    pub rev: u32,
    pub epoch: u32,
    pub signer: String,
    pub list: String,
    pub list_sig: String,
}

/// An epoch's history link: the key of the epoch before, sealed under the
/// new one (base64url).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BookEpochLink {
    pub epoch: u32,
    pub history_link: String,
}

/// `GET /v2/collab/books/<id>`: the chain from `since_rev` (a page of it,
/// `more` when further lists follow), the caller's wraps and every epoch's
/// history link. Fields only the owner and managers get are ignored.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BookInfo {
    pub share_id: String,
    pub owner: String,
    pub role: Role,
    /// `member` or `invited`.
    pub status: String,
    pub epoch: u32,
    pub rev: u32,
    pub hash: String,
    pub lists: Vec<BookListEntry>,
    #[serde(default)]
    pub more: bool,
    pub wraps: Vec<EpochWrap>,
    #[serde(default)]
    pub epochs: Vec<BookEpochLink>,
    /// Every member's status, for the owner and managers only (empty for
    /// the others).
    #[serde(default)]
    pub members: Vec<BookMemberStatus>,
}

/// One person's standing in a notebook, as the owner and managers see it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BookMemberStatus {
    pub account: String,
    pub role: Role,
    /// `member`, `invited` or `left` (they left, or declined; the signed
    /// list names them until a manager cuts them).
    pub status: String,
}

/// One row of `GET /v2/collab/books`: a notebook the caller is invited to or
/// in, found without knowing its share id.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BookSummary {
    pub share_id: String,
    /// The owner's account id and username.
    pub owner: String,
    pub owner_username: String,
    pub role: Role,
    /// `member` or `invited`.
    pub status: String,
    pub epoch: u32,
    pub rev: u32,
}

/// `POST /v2/collab/books/<id>/epoch`: the next signed list. Within an epoch
/// the wraps are for those added or given another role; a new epoch carries
/// one for every listed member and a history link.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewBookList {
    pub rev: u32,
    pub list: String,
    pub list_sig: String,
    pub wraps: Vec<AccountWrap>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub history_link: Option<String>,
}

/// What a notebook's stream returns: `head` is the notebook's (every
/// stream's sequences share one count), `stream_head` this log's last.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BookLogPage {
    pub head: u64,
    pub stream_head: u64,
    #[serde(default)]
    pub snapshot: Option<SnapshotEntry>,
    pub entries: Vec<LogEntry>,
    #[serde(default)]
    pub more: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewLink {
    pub link_id: String,
    pub proof_hash: String,
    pub role: Role,
    pub expires_at: i64,
}

/// `POST /v2/collab/join/<link_id>`'s answer.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Joined {
    pub share_id: String,
    pub owner_username: String,
    pub role: Role,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoticeIn {
    pub share_id: String,
    pub recipient: String,
    pub sealed: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoticeOut {
    pub id: String,
    pub share_id: String,
    pub from: String,
    pub sealed: String,
    pub at: i64,
}

/// What a live connection delivers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LiveEvent {
    /// `{"type":"hello"}`: the log's head and the epoch, on connect.
    Hello { head: u64, epoch: u32 },
    /// An update or comment the room stored, the caller's own included.
    Stored { seq: u64, envelope: Vec<u8> },
    /// Someone else's presence, never stored.
    Presence { envelope: Vec<u8> },
    /// The note moved to a new epoch.
    Epoch(u32),
    /// A frame the room refused.
    Error { code: String },
    /// The caller was removed, or sharing stopped; the room closes.
    Closed { reason: String },
}

/// One live connection to a note's room.
pub trait LiveLink: Send {
    /// Sends one envelope.
    fn send(&mut self, envelope: &[u8]) -> Result<()>;
    /// The next event, waiting at most `wait`; `None` when nothing came.
    fn recv(&mut self, wait: Duration) -> Result<Option<LiveEvent>>;
}

/// What a notebook's live connection delivers (`cloud/src/book_room.ts`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BookLiveEvent {
    /// `{"type":"hello"}`: the notebook's head and epoch, on connect.
    Hello { head: u64, epoch: u32 },
    /// A note's stream is open on this socket; `head` is that stream's last
    /// sequence, to catch up from over HTTP.
    Opened { stream: String, head: u64 },
    /// An entry the room stored in a stream, the caller's own included.
    Stored {
        stream: String,
        seq: u64,
        envelope: Vec<u8>,
    },
    /// Someone else's presence in a stream, never stored.
    Presence { stream: String, envelope: Vec<u8> },
    /// The notebook moved to a new epoch.
    Epoch(u32),
    /// A frame the room refused.
    Error { code: String },
    /// The caller was removed, or sharing stopped; the room closes.
    Closed { reason: String },
}

/// One live connection to a notebook's room. The manifest is always open;
/// a note's stream is opened (at most 32 at a time) before it is written.
pub trait BookLiveLink: Send {
    fn open(&mut self, stream: &str) -> Result<()>;
    fn close(&mut self, stream: &str) -> Result<()>;
    /// Sends one envelope to a stream.
    fn send(&mut self, stream: &str, envelope: &[u8]) -> Result<()>;
    /// The next event, waiting at most `wait`; `None` when nothing came.
    fn recv(&mut self, wait: Duration) -> Result<Option<BookLiveEvent>>;
}

/// A stream id as the room's frames carry it: 16 bytes.
pub fn stream_bytes(stream: &str) -> Option<[u8; 16]> {
    crate::codec::unb64_array::<16>(stream)
}

/// Parses a notebook room's binary frame: `stream || u64(seq) || envelope`;
/// seq 0 is presence.
pub fn parse_book_frame(frame: &[u8]) -> Option<BookLiveEvent> {
    let stream = crate::codec::b64(frame.get(..16)?);
    let seq = u64::from_be_bytes(frame.get(16..24)?.try_into().ok()?);
    let envelope = frame[24..].to_vec();
    Some(if seq == 0 {
        BookLiveEvent::Presence { stream, envelope }
    } else {
        BookLiveEvent::Stored {
            stream,
            seq,
            envelope,
        }
    })
}

/// Parses a notebook room's text frame.
pub fn parse_book_control(text: &str) -> Option<BookLiveEvent> {
    let value: Value = serde_json::from_str(text).ok()?;
    let number = |key: &str| value.get(key).and_then(Value::as_u64);
    match value.get("type")?.as_str()? {
        "opened" => Some(BookLiveEvent::Opened {
            stream: value.get("stream")?.as_str()?.to_owned(),
            head: number("head")?,
        }),
        "hello" | "epoch" | "error" | "closed" => {
            let event = parse_control(text)?;
            Some(match event {
                LiveEvent::Hello { head, epoch } => BookLiveEvent::Hello { head, epoch },
                LiveEvent::Epoch(epoch) => BookLiveEvent::Epoch(epoch),
                LiveEvent::Error { code } => BookLiveEvent::Error { code },
                LiveEvent::Closed { reason } => BookLiveEvent::Closed { reason },
                _ => return None,
            })
        }
        _ => None,
    }
}

/// Parses a room's binary frame: `u64(seq) || envelope`; seq 0 is presence.
pub fn parse_frame(frame: &[u8]) -> Option<LiveEvent> {
    let seq = u64::from_be_bytes(frame.get(..8)?.try_into().ok()?);
    let envelope = frame[8..].to_vec();
    Some(if seq == 0 {
        LiveEvent::Presence { envelope }
    } else {
        LiveEvent::Stored { seq, envelope }
    })
}

/// Parses a room's text frame.
pub fn parse_control(text: &str) -> Option<LiveEvent> {
    let value: Value = serde_json::from_str(text).ok()?;
    let number = |key: &str| value.get(key).and_then(Value::as_u64);
    match value.get("type")?.as_str()? {
        "hello" => Some(LiveEvent::Hello {
            head: number("head")?,
            epoch: u32::try_from(number("epoch")?).ok()?,
        }),
        "epoch" => Some(LiveEvent::Epoch(u32::try_from(number("epoch")?).ok()?)),
        "error" => Some(LiveEvent::Error {
            code: value.get("error")?.as_str()?.to_owned(),
        }),
        "closed" => Some(LiveEvent::Closed {
            reason: value
                .get("reason")
                .and_then(Value::as_str)
                .unwrap_or("closed")
                .to_owned(),
        }),
        _ => None,
    }
}

/// The server's shared-note routes for one caller.
pub trait CollabApi: Send + Sync {
    /// `PUT /v2/account/identity`; 409 `identity_exists` for another one.
    fn put_identity(&self, caller: &Caller, published: &Published) -> Result<()>;
    /// `GET /v2/account/identity`; `None` on 404.
    fn get_identity(&self, caller: &Caller) -> Result<Option<Published>>;
    /// `GET /v2/users/<username>`; `None` on 404.
    fn lookup(&self, caller: &Caller, username: &str) -> Result<Option<PublicIdentity>>;
    fn create_share(&self, caller: &Caller, share: &NewShare) -> Result<Created>;
    fn shares(&self, caller: &Caller) -> Result<Vec<ShareInfo>>;
    fn accept(&self, caller: &Caller, share_id: &str) -> Result<()>;
    fn leave(&self, caller: &Caller, share_id: &str) -> Result<()>;
    /// The new epoch; 409 `epoch_changed`.
    fn new_epoch(&self, caller: &Caller, share_id: &str, epoch: &NewEpoch) -> Result<u32>;
    fn stop(&self, caller: &Caller, share_id: &str) -> Result<()>;
    fn log(&self, caller: &Caller, share_id: &str, since: u64, limit: usize) -> Result<LogPage>;
    /// The stored entry's sequence.
    fn append(&self, caller: &Caller, share_id: &str, envelope: &[u8]) -> Result<u64>;
    /// 409 `head_moved` when `base_seq` isn't the head.
    fn snapshot(
        &self,
        caller: &Caller,
        share_id: &str,
        base_seq: u64,
        envelope: &[u8],
    ) -> Result<u64>;
    fn create_link(&self, caller: &Caller, share_id: &str, link: &NewLink) -> Result<()>;
    fn delete_link(&self, caller: &Caller, share_id: &str, link_id: &str) -> Result<()>;
    fn join(&self, caller: &Caller, link_id: &str, proof: &str) -> Result<Joined>;
    fn decline_request(&self, caller: &Caller, share_id: &str, account: &str) -> Result<()>;
    fn notify(&self, caller: &Caller, notice: &NoticeIn) -> Result<()>;
    fn notifications(&self, caller: &Caller) -> Result<Vec<NoticeOut>>;
    fn delete_notification(&self, caller: &Caller, id: &str) -> Result<()>;
    /// Opens the note's live room.
    fn live(&self, caller: &Caller, share_id: &str) -> Result<Box<dyn LiveLink>>;

    // Shared notebooks (`/v2/collab/books`).

    /// `GET /v2/collab/books`: every notebook the caller is invited to or in
    /// (not those left or declined), oldest first; no share id needed.
    fn books(&self, caller: &Caller) -> Result<Vec<BookSummary>>;
    /// Opens the notebook's live room: one socket for the manifest and every
    /// note stream opened on it.
    fn book_live(&self, caller: &Caller, share_id: &str) -> Result<Box<dyn BookLiveLink>>;
    /// Shares a notebook; 409 `share_id_taken`, `share_retired`.
    fn create_book(&self, caller: &Caller, book: &NewBook) -> Result<BookCreated>;
    /// The chain from `since_rev` (1 for all of it), the caller's wraps and
    /// the epochs' history links; 404 for a notebook the caller isn't in.
    fn book(&self, caller: &Caller, share_id: &str, since_rev: u32) -> Result<BookInfo>;
    /// An invitation becomes membership.
    fn accept_book(&self, caller: &Caller, share_id: &str) -> Result<()>;
    /// The next signed list; 409 `rev_conflict`, `head_moved`,
    /// `member_gone` (`account`: someone the list keeps whose account is
    /// deleted).
    fn new_book_list(&self, caller: &Caller, share_id: &str, list: &NewBookList) -> Result<()>;
    /// Leaves (or declines) a notebook, losing the wraps; 403 for the
    /// owner, who stops sharing instead.
    fn leave_book(&self, caller: &Caller, share_id: &str) -> Result<()>;
    /// Stops sharing a notebook (the owner); the id is retired for good.
    fn stop_book(&self, caller: &Caller, share_id: &str) -> Result<()>;
    /// A page of one stream (the manifest is [`MANIFEST_STREAM`]).
    fn book_log(
        &self,
        caller: &Caller,
        share_id: &str,
        stream: &str,
        since: u64,
        limit: usize,
    ) -> Result<BookLogPage>;
    /// Appends an update to a stream; its sequence. 409 `epoch_changed`,
    /// `compact_first`.
    fn book_append(
        &self,
        caller: &Caller,
        share_id: &str,
        stream: &str,
        envelope: &[u8],
    ) -> Result<u64>;
    /// A stream's snapshot, taken only while `base_seq` is that stream's
    /// last sequence (0 for a stream with nothing); 409 `head_moved`.
    fn book_snapshot(
        &self,
        caller: &Caller,
        share_id: &str,
        stream: &str,
        base_seq: u64,
        envelope: &[u8],
    ) -> Result<u64>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_and_control_messages_parse() {
        let mut frame = 7u64.to_be_bytes().to_vec();
        frame.extend_from_slice(b"env");
        assert_eq!(
            parse_frame(&frame),
            Some(LiveEvent::Stored {
                seq: 7,
                envelope: b"env".to_vec()
            })
        );
        let presence = [0u8; 8].iter().copied().chain(*b"p").collect::<Vec<_>>();
        assert!(matches!(
            parse_frame(&presence),
            Some(LiveEvent::Presence { .. })
        ));
        assert_eq!(parse_frame(b"short"), None);
        assert_eq!(
            parse_control(r#"{"type":"hello","head":4,"epoch":2}"#),
            Some(LiveEvent::Hello { head: 4, epoch: 2 })
        );
        assert_eq!(
            parse_control(r#"{"type":"closed","reason":"removed"}"#),
            Some(LiveEvent::Closed {
                reason: "removed".into()
            })
        );
        assert_eq!(parse_control("not json"), None);
    }
}
