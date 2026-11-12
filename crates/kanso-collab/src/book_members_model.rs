//! The shapes of a shared notebook's signed member list and the chain it
//! forms back to the owner (issue #390 › Shared notebooks: keys). Logic
//! lives in [`crate::book_members`].

use serde::{Deserialize, Serialize};

use crate::model::Member;

/// One link of a notebook's member-list chain, as the signer wrote it. Field
/// order is the wire's: `serde_json` writes a struct's fields in declaration
/// order, and the signature covers those exact bytes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BookMemberList {
    pub v: u32,
    /// The notebook's share id; signed, so a list cannot move to another
    /// notebook.
    pub share: String,
    /// Position in the chain, 1 for the owner's genesis list.
    pub rev: u32,
    pub epoch: u32,
    pub owner: String,
    /// The account that signed this list: the owner, or a manager the
    /// previous list names.
    pub signer: String,
    /// Hex hash of the previous list's bytes; absent only at `rev` 1.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prev: Option<String>,
    /// Commitment to `K_b[epoch]` ([`crate::BookKey::commit`]).
    pub key_commit: String,
    pub at: i64,
    /// From epoch 2 on: the log's head when the epoch began.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_seq: Option<u64>,
    /// Sorted by account; exactly one owner; managers carry the Manager role.
    pub members: Vec<Member>,
}

/// A list's bytes and its signature, as base64url: what the server stores
/// and relays.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedBookList {
    pub list: String,
    pub sig: String,
}

/// The newest list a client has checked all the way back to the owner's
/// genesis list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BookChainHead {
    pub list: BookMemberList,
    /// Hex hash of the list's bytes: what the next list names as `prev`.
    pub hash: String,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum BookListError {
    #[error("the notebook's member list isn't well-formed")]
    Malformed,
    #[error("the member list isn't signed by who it says")]
    BadSignature,
    #[error("the member list is for another notebook")]
    WrongShare,
    #[error("the member list's owner isn't the notebook's owner")]
    WrongOwner,
    #[error("the member list is signed by someone who may not change it")]
    NotAllowed,
    #[error("a manager changed something only the owner may change")]
    RankViolation,
    #[error("the member list doesn't follow the one before it")]
    Broken,
    #[error("the member list's epoch or key commitment is wrong")]
    BadEpoch,
    #[error("the chain is shorter than one already seen")]
    Stale,
    #[error("the chain is empty")]
    Empty,
}
