//! Shared notes: who a note is shared with, invitations, links, and an open
//! shared note kept live.

use crate::frb_generated::StreamSink;
use flutter_rust_bridge::frb;
use kanso_mobile::{
    CommentRow, InvitationRow, JoinRow, KansoError, LiveStatus, PersonLookup, PersonRow, ShareInfo,
    ShareRole, SharedEvent, SharedNotes,
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
