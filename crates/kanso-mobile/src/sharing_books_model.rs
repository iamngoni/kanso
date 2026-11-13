//! What the shared-notebook calls hand the shell: a notebook's people and
//! roles, the notebooks shared here, the invitations waiting, who last
//! edited each note, and whether the notebook is up to date.
//!
//! Plain data, like `sharing_model.rs`. Nothing here holds a note's text;
//! rows name notebooks and notes by ID and people by their member-list
//! names. Who may do what is decided in Rust (`can_manage`, `can_invite`
//! and the roles offered), never by the shell.

use kanso_collab::{MemberStanding, Role};

/// What a person may do in a shared notebook: the desktop's five roles.
/// `Manager` invites, removes and changes the roles below it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BookRole {
    Owner,
    Manager,
    Editor,
    Commenter,
    Viewer,
}

impl From<Role> for BookRole {
    fn from(role: Role) -> BookRole {
        match role {
            Role::Owner => BookRole::Owner,
            Role::Manager => BookRole::Manager,
            Role::Edit => BookRole::Editor,
            Role::Comment => BookRole::Commenter,
            Role::View => BookRole::Viewer,
        }
    }
}

impl From<BookRole> for Role {
    fn from(role: BookRole) -> Role {
        match role {
            BookRole::Owner => Role::Owner,
            BookRole::Manager => Role::Manager,
            BookRole::Editor => Role::Edit,
            BookRole::Commenter => Role::Comment,
            BookRole::Viewer => Role::View,
        }
    }
}

/// Where someone stands in a notebook.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BookStandingRow {
    Member,
    /// Invited, not joined yet.
    Invited,
    /// Left or declined.
    Left,
    /// The server tells only the owner and managers; offline nobody knows.
    Unknown,
}

impl From<MemberStanding> for BookStandingRow {
    fn from(standing: MemberStanding) -> BookStandingRow {
        match standing {
            MemberStanding::Member => BookStandingRow::Member,
            MemberStanding::Invited => BookStandingRow::Invited,
            MemberStanding::Left => BookStandingRow::Left,
            MemberStanding::Unknown => BookStandingRow::Unknown,
        }
    }
}

/// One person in a shared notebook.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BookMemberRow {
    pub account_id: String,
    /// The name the notebook's signed member list gives them.
    pub username: String,
    pub role: BookRole,
    /// This account.
    pub me: bool,
    pub standing: BookStandingRow,
    /// Their keys' fingerprint, to compare out loud.
    pub fingerprint: String,
    pub verified: bool,
    /// The roles this account may give them, highest first; empty when it
    /// may not change their role.
    pub roles: Vec<BookRole>,
    /// This account may remove them (or cancel their invitation).
    pub removable: bool,
}

/// A notebook's sharing, as its Share sheet shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BookShareInfo {
    pub share_id: String,
    /// The notebook in this library.
    pub notebook_id: String,
    pub name: String,
    pub my_role: BookRole,
    /// The owner's name.
    pub owner: String,
    /// The owner first, then the higher roles, then by name.
    pub members: Vec<BookMemberRow>,
    /// The roles this account may invite with, highest first; empty when
    /// it may not invite.
    pub invite_roles: Vec<BookRole>,
    /// This account owns the notebook: it stops sharing rather than leaves.
    pub owned: bool,
}

/// What sharing a notebook would do, before it is done.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BookShareCheck {
    pub name: String,
    /// Notes that would be shared.
    pub notes: u32,
    /// Locked notes, which can't be: they stay in a private
    /// "<name> (locked)" notebook.
    pub locked: u32,
    /// The name of that private notebook.
    pub locked_name: String,
    /// Titles of notes left out: already shared on their own, or canvases.
    pub left_out: Vec<String>,
}

/// How far sharing a notebook has got: "Sharing Studio… 120 of 340".
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BookShareProgress {
    pub notebook_id: String,
    pub done: u32,
    pub total: u32,
}

/// A notebook someone shared with this account that this device hasn't
/// joined: an invitation, or one another device of the account joined. Its
/// name is sealed until it is joined.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BookInvitationRow {
    pub share_id: String,
    /// Who shared it.
    pub from: String,
    pub role: BookRole,
    /// Waiting to be accepted (`false`: another device joined it, and it
    /// opens here too).
    pub invited: bool,
    /// When this device first saw it, Unix ms.
    pub at_ms: i64,
}

/// A notebook shared here, for the library's Shared section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SharedBookRow {
    pub share_id: String,
    pub notebook_id: String,
    pub name: String,
    pub my_role: BookRole,
    pub owner: String,
    /// Everyone's names, the owner first: the avatars.
    pub members: Vec<String>,
    pub status: BookSyncStatus,
}

/// Who last changed a note of a shared notebook: "Edited by Dan · 2 min ago".
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoteEditRow {
    pub note_id: String,
    /// The member-list name.
    pub name: String,
    /// This account made the change.
    pub me: bool,
    pub at_ms: i64,
}

/// Whether a shared notebook is up to date: "Offline · 4 changes waiting",
/// or "Up to date".
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BookSyncStatus {
    /// The last try to reach the notebook found no connection.
    pub offline: bool,
    /// Notes whose changes haven't gone out yet.
    pub waiting: u32,
    /// An open note of the notebook is connected live right now.
    pub live: bool,
}

/// What a notebook's end left: the private copy, and what to tell.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BookEndedRow {
    pub notebook_id: String,
    pub copy_name: String,
    pub message: String,
}

/// The words of a shared notebook's questions, the same on both apps.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BookWords {
    /// "Remove Dan from "Studio"?"
    pub remove_title: String,
    /// "Dan keeps the notes he already has…"
    pub remove_body: String,
    /// "Leave "Studio"? You'll keep a private copy."
    pub leave_title: String,
    /// "Stop sharing "Studio"? …"
    pub stop_title: String,
}
