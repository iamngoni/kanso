//! The shapes shared notes are made of (docs/ACCOUNTS.md › Collaboration):
//! roles, members, comments, presence and what a note's text did when a
//! change was merged. Logic lives in the modules that use them.

use serde::{Deserialize, Serialize};

/// What a member may do with a shared note.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    View,
    Comment,
    Edit,
    /// Notebooks only: edits like an editor and, in a shared notebook, may
    /// invite, remove and change roles below Manager. Declared between Edit
    /// and Owner so the derived order is the rank.
    Manager,
    Owner,
}

impl Role {
    pub const ALL: [Role; 5] = [
        Role::View,
        Role::Comment,
        Role::Edit,
        Role::Manager,
        Role::Owner,
    ];

    /// As the wire and the member list write it.
    pub fn as_str(self) -> &'static str {
        match self {
            Role::View => "view",
            Role::Comment => "comment",
            Role::Edit => "edit",
            Role::Manager => "manager",
            Role::Owner => "owner",
        }
    }

    pub fn parse(text: &str) -> Option<Role> {
        Role::ALL.into_iter().find(|role| role.as_str() == text)
    }

    /// The byte a note-key wrap binds: `o`, `m`, `e`, `c` or `v`.
    pub fn byte(self) -> u8 {
        match self {
            Role::View => b'v',
            Role::Comment => b'c',
            Role::Edit => b'e',
            Role::Manager => b'm',
            Role::Owner => b'o',
        }
    }

    /// Whether this role may change the note's text.
    pub fn may_edit(self) -> bool {
        matches!(self, Role::Edit | Role::Manager | Role::Owner)
    }

    /// Whether this role may invite, remove and change the roles of people
    /// below Manager in a shared notebook.
    pub fn may_manage(self) -> bool {
        matches!(self, Role::Manager | Role::Owner)
    }

    /// Whether this role may write comments, and resolve or reopen them.
    pub fn may_comment(self) -> bool {
        !matches!(self, Role::View)
    }

    /// What the Share sheet calls it.
    pub fn label(self) -> &'static str {
        match self {
            Role::View => "Can view",
            Role::Comment => "Can comment",
            Role::Edit => "Can edit",
            Role::Manager => "Manager",
            Role::Owner => "Owner",
        }
    }
}

/// One person in a member list, with the keys the owner wrapped to.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Member {
    pub account: String,
    pub username: String,
    pub role: Role,
    /// Ed25519, base64url.
    pub sign_pub: String,
    /// X25519, base64url.
    pub box_pub: String,
}

/// The list of who is in a shared note for one epoch, as the owner signs it.
/// Field order is the wire's: `serde_json` writes a struct's fields in
/// declaration order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemberList {
    pub v: u32,
    pub share: String,
    pub epoch: u32,
    pub owner: String,
    pub key_commit: String,
    pub at: i64,
    /// From epoch 2 on: the log's head when the epoch began. Entries of an
    /// earlier epoch with a later sequence are dropped, so a removed or
    /// demoted editor's writes can't be slipped in after.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_seq: Option<u64>,
    /// `canvas` for a shared canvas; absent for a text note. The owner signs
    /// it with the list, so no member can turn a canvas into a text note (or
    /// back) for the others, whatever the note's body later says.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    pub members: Vec<Member>,
}

/// An account's public identity, as `GET /v2/users/<username>` answers it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicIdentity {
    pub account_id: String,
    pub username: String,
    pub sign_pub: String,
    pub box_pub: String,
    /// Ed25519 over the account, username and box key, base64url.
    pub binding: String,
}

/// What an envelope carries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    Update,
    Snapshot,
    Comment,
    Presence,
}

impl Kind {
    pub fn byte(self) -> u8 {
        match self {
            Kind::Update => b'u',
            Kind::Snapshot => b's',
            Kind::Comment => b'c',
            Kind::Presence => b'p',
        }
    }

    pub fn from_byte(byte: u8) -> Option<Kind> {
        match byte {
            b'u' => Some(Kind::Update),
            b's' => Some(Kind::Snapshot),
            b'c' => Some(Kind::Comment),
            b'p' => Some(Kind::Presence),
            _ => None,
        }
    }
}

/// The readable part of an envelope: who wrote it, when (by epoch) and
/// what.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Header {
    pub kind: Kind,
    pub epoch: u32,
    pub author_account: String,
    pub author_device: String,
    pub counter: u64,
}

/// One change to a note's text, in UTF-8 byte offsets of the text before
/// it: `delete` bytes at `at` are replaced by `insert`. A merge reports its
/// changes in order, each against the text the ones before it left.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Splice {
    pub at: usize,
    pub delete: usize,
    pub insert: String,
}

/// A comment event, as members write them (`kind` `c`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommentEvent {
    pub id: String,
    pub op: CommentOp,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anchor: Option<Anchor>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mentions: Vec<String>,
    pub at: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CommentOp {
    Add,
    Edit,
    Resolve,
    Reopen,
    Delete,
}

/// Where a comment points: two sticky positions in the note's text
/// (base64url) and the words they held when it was written.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Anchor {
    pub start: String,
    pub end: String,
    pub quote: String,
}

/// A comment as its events fold up.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Comment {
    pub id: String,
    pub author: String,
    pub author_name: String,
    pub anchor: Anchor,
    pub body: String,
    pub mentions: Vec<String>,
    pub resolved: bool,
    pub created_at: i64,
    pub edited_at: Option<i64>,
    /// The log sequence of its last event.
    pub seq: u64,
}

/// Where a comment sits in the text now.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Placement {
    /// Its words, as a byte range of the current text.
    Anchored(std::ops::Range<usize>),
    /// Its words were deleted: the nearest place left.
    Orphaned(usize),
}

/// Someone else's cursor in an open shared note, as presence carries it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PresenceMessage {
    /// Sticky positions (base64url): where the selection started and where
    /// the caret is. Equal for a plain caret. Empty on a canvas.
    pub cursor: String,
    pub head: String,
    pub name: String,
    /// On a canvas: where the person's pointer is.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canvas: Option<CanvasPointer>,
}

/// Where someone's pen or pointer is on a shared canvas, in canvas
/// coordinates: the only thing sent while the pen hovers. A stroke being
/// drawn is not here: it streams as points, so viewers see the real ink.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CanvasPointer {
    pub x: f64,
    pub y: f64,
    /// The tool in hand (`pen`, `eraser`, `lasso`, ...).
    pub tool: String,
    /// Counts up with every move this device sends, so a late arrival can
    /// be told from a newer one.
    #[serde(default)]
    pub seq: u64,
}

/// Someone else's pointer on the open shared canvas.
#[derive(Clone, Debug, PartialEq)]
pub struct RemotePointer {
    pub account: String,
    pub device: String,
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub tool: String,
    /// Which of [`CURSOR_COLOURS`] this author gets.
    pub colour: usize,
    pub seen_at: i64,
}

/// What [`Engine::sync_canvas`](crate::Engine::sync_canvas) gives back.
#[derive(Clone, Debug, PartialEq)]
pub struct CanvasMerge {
    /// The canvas as it is now, tombstones included: the view takes it.
    pub canvas: kanso_core::canvas::Canvas,
    /// It differs from the canvas the view sent: others' edits came in, or
    /// an edit was refused.
    pub changed: bool,
    /// The canvas is past what a note can hold: the library keeps the last
    /// canvas that fit, and the view should say so and refuse new ink.
    pub full: bool,
}

/// A remote caret placed in the current text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemoteCursor {
    pub account: String,
    pub device: String,
    pub name: String,
    /// The selection as byte offsets: `anchor..head` (either order).
    pub anchor: usize,
    pub head: usize,
    /// Which of [`CURSOR_COLOURS`] this author gets.
    pub colour: usize,
    pub seen_at: i64,
}

/// How many colours remote cursors cycle through (the app owns the
/// colours themselves).
pub const CURSOR_COLOURS: usize = 8;

/// A notification about a mention, opened.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Notice {
    pub comment: String,
    pub title: String,
    pub excerpt: String,
    pub at: i64,
}

/// Someone this account shares with, with the fingerprint of the keys
/// pinned for them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Person {
    pub account_id: String,
    pub username: String,
    pub fingerprint: String,
    /// This account compared fingerprints and marked them verified.
    pub verified: bool,
}

/// A note someone shared with this account, waiting to be accepted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Invitation {
    pub share_id: String,
    pub owner: Person,
    pub role: Role,
}

/// One person in a shared note, as the Share sheet lists them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemberView {
    pub person: Person,
    pub role: Role,
    /// Still to accept the invitation.
    pub invited: bool,
    /// Left the note or declined it: still in the signed list until the
    /// owner's device starts the next epoch without them.
    pub left: bool,
    /// Their keys couldn't be checked (the account is gone, or the lookup
    /// failed): no new key is wrapped to them. What they wrote under a list
    /// that names them still merges, checked against that list's keys.
    pub unverified: bool,
}

/// An invite link, as its owner sees it (never its secret).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkView {
    pub link_id: String,
    pub role: Role,
    pub expires_at: i64,
}

/// Someone who asked to join through a link.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequestView {
    pub person: Person,
    /// The link they asked through.
    pub link_id: String,
    /// The role the link gives.
    pub role: Role,
    pub at: i64,
}

/// A shared note as the Share sheet shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShareView {
    pub share_id: String,
    pub note_id: String,
    /// This account's role.
    pub role: Role,
    pub owner: Person,
    pub members: Vec<MemberView>,
    /// The owner's view only.
    pub links: Vec<LinkView>,
    pub requests: Vec<RequestView>,
    pub epoch: u32,
}

/// What [`crate::engine::Engine::sync_editor`] did to the open note: the
/// splices to apply to the editor's title and body (in order), and the text
/// they lead to.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EditorMerge {
    pub title: Vec<Splice>,
    pub body: Vec<Splice>,
    pub title_text: String,
    pub body_text: String,
}

/// A comment as the Comments panel shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommentView {
    pub comment: Comment,
    pub placement: Placement,
    /// Written on this device and not taken by the server yet.
    pub pending: bool,
    /// Written by this account.
    pub mine: bool,
}

/// Something the engine had to do that the person should hear about: an
/// edit it couldn't share (kept as a private note), a note at its size
/// limit, a note it can't write because it is locked.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Problem {
    pub share_id: String,
    pub message: String,
    pub at: i64,
}

/// How much of a shared note's size limit is used, in bytes of its state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SizeUse {
    pub used: usize,
    pub limit: usize,
}

impl SizeUse {
    /// Past 80 %: time to say so.
    pub fn is_near(&self) -> bool {
        self.used * 5 >= self.limit * 4
    }
}

/// What one sync of a shared note did.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NoteSync {
    /// Entries merged from the log.
    pub pulled: usize,
    /// Entries this device sent.
    pub pushed: usize,
    /// A snapshot was written.
    pub compacted: bool,
}

/// What the live connection of an open shared note tells the app.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LiveNotice {
    /// Remote edits are waiting: call `sync_editor`.
    Text { share_id: String },
    /// Someone's cursor moved, came or went.
    Presence { share_id: String },
    /// Comments changed.
    Comments { share_id: String },
    /// The room is connected (`true`) or not.
    Connection { share_id: String, live: bool },
    /// This account is no longer in the note, or sharing stopped.
    Ended { share_id: String },
    /// A [`Problem`] was added: read them with `Engine::problems`.
    Problem { share_id: String },
}
