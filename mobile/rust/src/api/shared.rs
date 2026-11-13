//! Shared notes: who a note is shared with, invitations, links, and an open
//! shared note kept live.

use crate::frb_generated::StreamSink;
use flutter_rust_bridge::frb;
use kanso_mobile::{
    BookEndedRow, BookInvitationRow, BookRole, BookShareCheck, BookShareInfo, BookShareProgress,
    BookSyncStatus, BookWords, CommentRow, InvitationRow, JoinRow, KansoError, LiveStatus,
    NoteEditRow, PersonLookup, PersonRow, ShareInfo, ShareRole, SharedBookRow, SharedEvent,
    SharedNotes,
};

#[frb(opaque)]
pub struct KansoShared(SharedNotes);

impl KansoShared {
    pub(super) fn new(shared: SharedNotes) -> KansoShared {
        KansoShared(shared)
    }

    pub fn can_share(&self, note_id: String) -> Result<(), KansoError> {
        self.0.can_share(note_id)
    }

    pub fn share(
        &self,
        note_id: String,
        invites: Vec<(String, ShareRole)>,
    ) -> Result<ShareInfo, KansoError> {
        self.0.share(note_id, invites)
    }

    pub fn info(&self, note_id: String) -> Result<Option<ShareInfo>, KansoError> {
        self.0.info(note_id)
    }

    pub fn invite(
        &self,
        note_id: String,
        username: String,
        role: ShareRole,
    ) -> Result<ShareInfo, KansoError> {
        self.0.invite(note_id, username, role)
    }

    pub fn set_role(
        &self,
        note_id: String,
        account_id: String,
        role: ShareRole,
    ) -> Result<ShareInfo, KansoError> {
        self.0.set_role(note_id, account_id, role)
    }

    pub fn remove(&self, note_id: String, account_id: String) -> Result<ShareInfo, KansoError> {
        self.0.remove(note_id, account_id)
    }

    pub fn approve_request(
        &self,
        note_id: String,
        account_id: String,
    ) -> Result<ShareInfo, KansoError> {
        self.0.approve_request(note_id, account_id)
    }

    pub fn decline_request(
        &self,
        note_id: String,
        account_id: String,
    ) -> Result<ShareInfo, KansoError> {
        self.0.decline_request(note_id, account_id)
    }

    pub fn create_link(
        &self,
        note_id: String,
        role: ShareRole,
        days: u32,
    ) -> Result<String, KansoError> {
        self.0.create_link(note_id, role, days)
    }

    pub fn cancel_link(&self, note_id: String, link_id: String) -> Result<ShareInfo, KansoError> {
        self.0.cancel_link(note_id, link_id)
    }

    pub fn join(&self, link: String) -> Result<JoinRow, KansoError> {
        self.0.join(link)
    }

    pub fn stop(&self, note_id: String) -> Result<(), KansoError> {
        self.0.stop(note_id)
    }

    pub fn leave(&self, note_id: String) -> Result<(), KansoError> {
        self.0.leave(note_id)
    }

    pub fn invitations(&self) -> Result<Vec<InvitationRow>, KansoError> {
        self.0.invitations()
    }

    pub fn accept(&self, share_id: String) -> Result<String, KansoError> {
        self.0.accept(share_id)
    }

    pub fn decline(&self, share_id: String) -> Result<(), KansoError> {
        self.0.decline(share_id)
    }

    pub fn lookup_person(&self, username: String) -> Result<PersonLookup, KansoError> {
        self.0.lookup_person(username)
    }

    pub fn verify_person(&self, account_id: String) -> Result<(), KansoError> {
        self.0.verify_person(account_id)
    }

    pub fn accept_new_key(
        &self,
        username: String,
        fingerprint: String,
    ) -> Result<PersonRow, KansoError> {
        self.0.accept_new_key(username, fingerprint)
    }

    pub fn sync_shared(&self) -> Result<SharedEvent, KansoError> {
        self.0.sync_shared()
    }

    /// What a sync of shared notes found, each time, until Dart stops
    /// listening.
    pub fn events(&self, sink: StreamSink<SharedEvent>) {
        self.0.events(move |event| sink.add(event).is_ok());
    }

    // ---------------------------------------------------- shared notebooks

    /// What sharing a notebook would do; refuses one that can't be shared.
    pub fn can_share_book(&self, notebook_id: String) -> Result<BookShareCheck, KansoError> {
        self.0.can_share_book(notebook_id)
    }

    pub fn share_book(
        &self,
        notebook_id: String,
        invites: Vec<(String, BookRole)>,
    ) -> Result<BookShareInfo, KansoError> {
        self.0.share_book(notebook_id, invites)
    }

    /// How far a notebook being shared has got, until Dart stops listening.
    pub fn book_progress(&self, sink: StreamSink<BookShareProgress>) {
        self.0
            .book_progress(move |progress| sink.add(progress).is_ok());
    }

    /// A notebook's sharing; `None` when it isn't shared.
    pub fn book_info(&self, notebook_id: String) -> Result<Option<BookShareInfo>, KansoError> {
        self.0.book_info(notebook_id)
    }

    pub fn invite_to_book(
        &self,
        notebook_id: String,
        username: String,
        role: BookRole,
    ) -> Result<BookShareInfo, KansoError> {
        self.0.invite_to_book(notebook_id, username, role)
    }

    pub fn set_book_role(
        &self,
        notebook_id: String,
        account_id: String,
        role: BookRole,
    ) -> Result<BookShareInfo, KansoError> {
        self.0.set_book_role(notebook_id, account_id, role)
    }

    pub fn remove_from_book(
        &self,
        notebook_id: String,
        account_id: String,
    ) -> Result<BookShareInfo, KansoError> {
        self.0.remove_from_book(notebook_id, account_id)
    }

    pub fn leave_book(&self, notebook_id: String) -> Result<BookEndedRow, KansoError> {
        self.0.leave_book(notebook_id)
    }

    pub fn stop_book(&self, notebook_id: String) -> Result<BookEndedRow, KansoError> {
        self.0.stop_book(notebook_id)
    }

    /// The questions' words, the same as the desktop's.
    #[frb(sync)]
    pub fn book_words(&self, name: String, who: String) -> BookWords {
        self.0.book_words(name, who)
    }

    pub fn book_invitations(&self) -> Result<Vec<BookInvitationRow>, KansoError> {
        self.0.book_invitations()
    }

    /// Joins the notebook; answers with its library ID.
    pub fn accept_book(&self, share_id: String) -> Result<String, KansoError> {
        self.0.accept_book(share_id)
    }

    pub fn decline_book(&self, share_id: String) -> Result<(), KansoError> {
        self.0.decline_book(share_id)
    }

    /// The notebooks shared here, for the library's Shared section.
    pub fn shared_books(&self) -> Result<Vec<SharedBookRow>, KansoError> {
        self.0.shared_books()
    }

    /// "Edited by" for the notes of a shared notebook.
    pub fn book_edits(&self, notebook_id: String) -> Result<Vec<NoteEditRow>, KansoError> {
        self.0.book_edits(notebook_id)
    }

    /// "Offline · 4 changes waiting" or "Up to date"; `None` when the
    /// notebook isn't shared.
    pub fn book_status(&self, notebook_id: String) -> Result<Option<BookSyncStatus>, KansoError> {
        self.0.book_status(notebook_id)
    }

    pub fn open_live(&self, note_id: String) -> Result<(), KansoError> {
        self.0.open_live(note_id)
    }

    pub fn close_live(&self, note_id: String) {
        self.0.close_live(note_id);
    }

    /// What the note's page says of its sharing; `None` when it isn't
    /// shared.
    pub fn live_status(&self, note_id: String) -> Result<Option<LiveStatus>, KansoError> {
        self.0.live_status(note_id)
    }

    pub fn dismiss_problems(&self, note_id: String) -> Result<(), KansoError> {
        self.0.dismiss_problems(note_id)
    }

    /// The note's comments: those on words still in the text in order, then
    /// the orphaned. Offsets are UTF-16 units of the note's body.
    pub fn comments(&self, note_id: String) -> Result<Vec<CommentRow>, KansoError> {
        self.0.comments(note_id)
    }

    pub fn open_comments(&self, note_id: String) -> Result<u32, KansoError> {
        self.0.open_comments(note_id)
    }

    pub fn comments_band(&self, note_id: String) -> Result<String, KansoError> {
        self.0.comments_band(note_id)
    }

    pub fn add_comment(
        &self,
        note_id: String,
        start: u32,
        end: u32,
        body: String,
    ) -> Result<String, KansoError> {
        self.0.add_comment(note_id, start, end, body)
    }

    pub fn edit_comment(
        &self,
        note_id: String,
        comment_id: String,
        body: String,
    ) -> Result<(), KansoError> {
        self.0.edit_comment(note_id, comment_id, body)
    }

    pub fn delete_comment(&self, note_id: String, comment_id: String) -> Result<(), KansoError> {
        self.0.delete_comment(note_id, comment_id)
    }

    pub fn resolve_comment(
        &self,
        note_id: String,
        comment_id: String,
        resolved: bool,
    ) -> Result<(), KansoError> {
        self.0.resolve_comment(note_id, comment_id, resolved)
    }
}
