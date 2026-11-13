//! Shared notes as the app holds them (docs/ACCOUNTS.md › Collaboration):
//! the signed-in account's engine and what it last said, the shared note
//! open in the editor, and the Share sheet's steps. The logic is in
//! `collab.rs`, `collab_text.rs`, `editor_collab.rs` and `share_sheet.rs`.

use std::collections::{HashMap, HashSet};
use std::ops::Range;
use std::sync::Arc;

use gpui_kit::base::input::TextDecorationCollection;
use gpui_kit::{AnyWindowHandle, Global, Task};
use kanso_collab::engine::BookLiveHandle;
use kanso_collab::{
    BookSummary, CommentView, EditorMerge, Engine, Invitation, LiveHandle, NoteEdit, Person,
    Problem, Role, SizeUse,
};
use kanso_store::StoredNotice;
use kanso_sync::Session;

use crate::book_share_model::BookRow;

/// The signed-in account's shared notes: its engine, made for the session
/// in [`crate::services::Services`] and dropped with it, and what the last
/// background sync brought.
#[derive(Default)]
pub struct Collab {
    pub engine: Option<Arc<Engine>>,
    /// The session the engine was made for.
    pub session: Option<Arc<Session>>,
    /// This account's fingerprint, once its identity keys are made.
    pub fingerprint: Option<String>,
    /// Notes shared with this account, waiting to be accepted.
    pub invitations: Vec<Invitation>,
    /// Mentions of this account in comments, newest first.
    pub notices: Vec<StoredNotice>,
    /// Notebooks shared with this account that this device hasn't joined:
    /// invitations, and notebooks another device of the account joined.
    pub book_invitations: Vec<BookSummary>,
    /// The server said this Kanso is too old for shared notebooks, so the
    /// Shared section says to update where invitations would be.
    pub book_update_needed: bool,
    /// Shared notebooks joined here, as the last sync left them.
    pub books: Vec<BookRow>,
    /// Who last edited each note of a shared notebook, by note id.
    pub book_edits: HashMap<String, NoteEdit>,
    /// The live rooms of shared notebooks with a note open in an editor.
    pub book_rooms: HashMap<String, BookLiveHandle>,
    /// The notebooks whose room is connected now.
    pub book_live: HashSet<String>,
    /// The last sync got no answer from the server.
    pub offline: bool,
    /// A sync of shared notes is running, and another was asked for.
    pub syncing: bool,
    pub again: bool,
    /// Tests put an engine in over the in-memory server: kept whatever the
    /// session.
    #[cfg(test)]
    pub pinned: bool,
}

impl Global for Collab {}

impl Collab {
    /// Mentions not read yet.
    pub fn unread(&self) -> usize {
        self.notices.iter().filter(|n| !n.read).count()
    }

    /// The shared notebook joined here that `book_id` is, if it is one.
    pub fn book(&self, book_id: &str) -> Option<&BookRow> {
        self.books.iter().find(|b| b.view.book_id == book_id)
    }
}

/// A note of a shared notebook open in the editor: its room shows who else
/// is in the note, and where.
pub struct OpenBookNote {
    pub share_id: String,
    pub stream: String,
    /// The caret last told the room: (anchor, head) body byte offsets.
    pub caret_sent: Option<(usize, usize)>,
    pub cursors: Vec<RemoteMark>,
    /// The tints on other people's selections.
    pub selections: Option<TextDecorationCollection>,
}

/// The shared note open in the editor: the engine it is attached to, its
/// live room, and what the room last said.
pub struct OpenShare {
    pub share_id: String,
    pub note_id: String,
    pub role: Role,
    /// The shared note is a canvas, drawn on rather than typed in: its
    /// document goes through the canvas view.
    pub canvas: bool,
    pub engine: Arc<Engine>,
    /// Keeps the room open; dropped when the note closes.
    pub live: Option<LiveHandle>,
    /// The room is connected.
    pub connected: bool,
    /// The note stopped being shared while open: it is a private copy now.
    pub ended: bool,
    /// The title and body the editor last got from `sync_editor`.
    pub synced: (String, String),
    /// A merge `sync_editor` returned that waits for the window to apply.
    pub pending: Option<(String, String, EditorMerge)>,
    /// The caret last told the room: (anchor, head) body byte offsets.
    pub caret_sent: Option<(usize, usize)>,
    pub cursors: Vec<RemoteMark>,
    /// The window the editor is in, for merges that arrive outside a
    /// window update.
    pub window: AnyWindowHandle,
    /// The tints on commented text, and on other people's selections.
    pub highlights: Option<TextDecorationCollection>,
    pub selections: Option<TextDecorationCollection>,
    /// The note's comments as last folded, for the inspector's Comments
    /// tab, and the usernames in the note, to set @mentions apart.
    pub comments: Vec<CommentView>,
    pub members: Vec<String>,
    /// What the engine couldn't do for this note, to show above the text.
    pub problems: Vec<Problem>,
    /// How much of the note's size limit it uses, as last read.
    pub size: Option<SizeUse>,
    /// Runs `sync_editor` once typing pauses.
    pub sync_task: Option<Task<()>>,
    /// Takes what the live room says, while the note is open.
    pub _notices: Task<()>,
}

/// Someone else's caret or selection in the open note, placed in the
/// editor's text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemoteMark {
    pub name: String,
    /// Where their caret is (a byte offset of the body).
    pub caret: usize,
    /// What they selected; empty for a plain caret.
    pub selection: Range<usize>,
    /// Which of [`crate::theme::CURSOR_COLOURS`] they get.
    pub colour: usize,
}

/// What the Share sheet shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SheetStep {
    /// Sharing needs an account.
    SignedOut,
    /// The note can't be shared: why.
    NotShareable(String),
    /// Members, requests, links and the invite field.
    Main,
    /// The first share with someone: their fingerprint to compare before
    /// inviting them.
    Verify { person: Person, role: Role },
    /// Someone's keys changed since they were pinned: the old and the new
    /// fingerprint, before inviting them.
    KeyChanged {
        pinned: Option<Person>,
        offered: Person,
        role: Role,
    },
}

/// A task the Share sheet is running.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SheetBusy {
    Loading,
    LookingUp,
    Inviting,
    Changing,
    MakingLink,
    Stopping,
}

/// An invite link made in this sheet: its secret is only known here, so
/// its URL shows until the sheet closes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MadeLink {
    pub url: String,
    pub role: Role,
    pub days: i64,
}

/// What looking someone up for the Share sheet found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Lookup {
    /// Their keys, pinned (now, or before).
    Found(Person),
    /// The server gives other keys than the ones pinned for them.
    Changed {
        pinned: Option<Person>,
        offered: Person,
    },
}
