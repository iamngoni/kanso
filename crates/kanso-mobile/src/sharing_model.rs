//! What the shared-note calls hand the shell: a note's sharing, its people,
//! links and requests, the invitations waiting, and what a sync of shared
//! notes found.
//!
//! Plain data, like `model.rs`. A shared note's text is never here: the
//! rows name people, roles and notes by ID, and the text is read the way
//! any note's is (a locked note's never is).

use kanso_collab::Role;

/// What a person may do with a shared note. The same four roles as the
/// desktop's: `Editor` is its "Can edit", `Commenter` its "Can comment" and
/// `Viewer` its "Can view".
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ShareRole {
    Owner,
    Editor,
    Commenter,
    Viewer,
}

impl From<Role> for ShareRole {
    fn from(role: Role) -> ShareRole {
        match role {
            Role::Owner => ShareRole::Owner,
            Role::Edit => ShareRole::Editor,
            Role::Comment => ShareRole::Commenter,
            Role::View => ShareRole::Viewer,
        }
    }
}

impl From<ShareRole> for Role {
    fn from(role: ShareRole) -> Role {
        match role {
            ShareRole::Owner => Role::Owner,
            ShareRole::Editor => Role::Edit,
            ShareRole::Commenter => Role::Comment,
            ShareRole::Viewer => Role::View,
        }
    }
}

/// One person in a shared note.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemberRow {
    pub account_id: String,
    pub username: String,
    pub role: ShareRole,
    /// This account.
    pub me: bool,
    /// The fingerprint of the keys pinned for them, to compare out loud.
    pub fingerprint: String,
    /// Their keys aren't the ones pinned for them, or couldn't be checked
    /// (their account is gone): nothing new is sealed to them.
    pub key_changed: bool,
    /// Still to accept the invitation.
    pub invited: bool,
    /// Left the note, or declined it; the owner's next change drops them.
    pub left: bool,
    /// Fingerprints compared and marked as matching.
    pub verified: bool,
}

/// An invite link of a note, as its owner sees it. Its secret is never in
/// a row: the URL is known only to the call that made it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShareLinkRow {
    pub id: String,
    pub role: ShareRole,
    pub expires_ms: i64,
}

/// A note's sharing: who is in it, its links and who asked to join.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShareInfo {
    pub share_id: String,
    pub note_id: String,
    /// This account's role.
    pub my_role: ShareRole,
    /// The note's owner.
    pub owner: String,
    /// Everyone in the note, the owner first.
    pub members: Vec<MemberRow>,
    /// The owner's view only.
    pub links: Vec<ShareLinkRow>,
    /// Who asked through a link, with the link's role; the owner's view only.
    pub requests: Vec<MemberRow>,
}

/// A note someone shared with this account, waiting to be accepted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InvitationRow {
    pub share_id: String,
    /// Always `None`: a note's title is sealed to its members, and this
    /// account is not one until it accepts.
    pub title: Option<String>,
    /// The owner's username.
    pub from: String,
    pub role: ShareRole,
    /// When this phone first saw the invitation.
    pub at_ms: i64,
}

/// What a sync of shared notes found.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SharedEvent {
    /// The notes whose text, role or sharing changed.
    pub note_ids: Vec<String>,
    /// Invitations waiting now.
    pub invitations: u32,
    /// The notes whose comments changed: someone commented, edited,
    /// resolved or deleted one.
    pub comment_notes: Vec<String>,
    /// The open notes whose room connected or lost its connection, met a
    /// problem, or saw someone's cursor move: ask for their
    /// [`LiveStatus`] and cursors again.
    pub live_notes: Vec<String>,
}

/// What the page of a shared note says of its sharing, as the desktop's
/// footer and banner do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveStatus {
    /// This account's role in the note.
    pub role: ShareRole,
    /// The note's room is open and connected.
    pub connected: bool,
    /// Sharing ended while the note was open: it is a private copy now,
    /// edited as any note.
    pub ended: bool,
    /// The note uses past 80 % of its size limit.
    pub near_limit: bool,
    /// What couldn't be done for the note, in the engine's words, newest
    /// first, until dismissed.
    pub problems: Vec<String>,
}

/// Someone else's caret and selection in the open note, in UTF-16 units of
/// the editor's text, with the colours it is drawn in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CursorMark {
    pub name: String,
    /// Where their caret is.
    pub caret: u32,
    /// What they have selected; empty with none.
    pub start: u32,
    pub end: u32,
    /// Their colour on a light page and on a dark one, `0xRRGGBB`.
    pub light: u32,
    pub dark: u32,
}

/// Someone looked up by username, before sharing with them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PersonRow {
    pub account_id: String,
    pub username: String,
    /// To compare out loud before the first share.
    pub fingerprint: String,
    pub verified: bool,
}

/// What looking someone up found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PersonLookup {
    /// Their keys, pinned now or before.
    Found(PersonRow),
    /// The server gives other keys than the ones pinned for them: compare
    /// `offered`'s fingerprint, then `SharedNotes::accept_new_key`.
    Changed {
        pinned: Option<PersonRow>,
        offered: PersonRow,
    },
}

/// What asking to join a note through a link did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JoinRow {
    pub share_id: String,
    /// The owner, who lets this account in.
    pub owner: String,
    /// The role the link gives.
    pub role: ShareRole,
}

/// One change to a text, in UTF-8 byte offsets of the text before it:
/// `delete` bytes at `at` are replaced by `insert`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpliceRow {
    pub at: u32,
    pub delete: u32,
    pub insert: String,
}

/// What the open note's editor must do after a merge: the splices to apply,
/// in order, and the text they lead to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditorMergeRow {
    pub title: Vec<SpliceRow>,
    pub body: Vec<SpliceRow>,
    pub title_text: String,
    pub body_text: String,
}

/// Someone else's caret in the open note, in UTF-8 byte offsets of the body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CursorRow {
    pub account_id: String,
    pub name: String,
    pub anchor: u32,
    pub head: u32,
    /// Which of the app's cursor colours, from 0.
    pub colour: u32,
}
