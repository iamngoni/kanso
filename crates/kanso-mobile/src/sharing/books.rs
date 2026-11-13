//! Shared notebooks on a phone (issue #390 slice 39), the desktop's Share
//! notebook sheet, Shared section, "Edited by" and offline line: sharing a
//! notebook, its people and their roles, invitations, leaving and stopping,
//! and the live room of a notebook whose note is open.
//!
//! Calls take a notebook by its library ID; the engine's share ID stays
//! inside, except for invitations, which have no notebook here yet. Who may
//! do what is the engine's rule ([`may_change_role`]), handed to the shell
//! as the roles each member's menu offers; the shell decides nothing.
//!
//! A note of a shared notebook goes live the way a shared note does
//! ([`SharedNotes::open_live`]): one room per notebook, kept while any of
//! its notes is open. Its text goes through the library, which the
//! notebook's sync merges, so the editor saves it as any note; the room
//! carries presence, and the cursors' names come from the signed list.

use std::collections::{BTreeSet, HashMap};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Weak};

use kanso_collab::engine::book_live::{BookLiveHandle, BookNotify};
use kanso_collab::engine::book_roster::may_change_role;
use kanso_collab::{BookLiveNotice, BookMemberView, BookView, Engine, Role, book_copy};
use kanso_core::now_ms;
use kanso_store::SharedStatus;

use super::{SharedNotes, collab_error};
use crate::boundary::{guard, held};
use crate::errors::KansoError;
use crate::library::Inner;
use crate::sharing_books_model::{
    BookEndedRow, BookInvitationRow, BookMemberRow, BookRole, BookShareCheck, BookShareInfo,
    BookShareProgress, BookSyncStatus, BookWords, NoteEditRow, SharedBookRow,
};
use crate::sharing_model::{CursorRow, LiveStatus, SharedEvent};

/// The roles of a notebook, highest first.
const BOOK_ROLES: [Role; 5] = [
    Role::Owner,
    Role::Manager,
    Role::Edit,
    Role::Comment,
    Role::View,
];

/// A notebook's live room while any of its notes is open here.
pub(crate) struct BookRoom {
    engine: Arc<Engine>,
    /// Open notes: library note ID to the note's log.
    notes: HashMap<String, String>,
    connected: Arc<AtomicBool>,
    ended: Arc<AtomicBool>,
    _live: BookLiveHandle,
}

impl Drop for BookRoom {
    fn drop(&mut self) {
        // The room's own lists of open notes, carets and cursors go too.
        if let Some(share_id) = self.share_id() {
            for stream in self.notes.values() {
                self.engine.book_close_note(&share_id, stream);
            }
        }
    }
}

impl BookRoom {
    fn share_id(&self) -> Option<String> {
        Some(self._live.share_id().to_owned())
    }
}

/// The role menu and Remove of `member`, seen by an account of `mine`: the
/// engine's rank rule. Empty roles when the role can't change. Pure.
pub(crate) fn member_actions(mine: Option<Role>, member: &BookMemberView) -> (Vec<Role>, bool) {
    let Some(mine) = mine.filter(|r| r.may_manage()) else {
        return (Vec::new(), false);
    };
    if member.me || member.role == Role::Owner {
        return (Vec::new(), false);
    }
    let roles: Vec<Role> = BOOK_ROLES
        .iter()
        .copied()
        .filter(|to| *to == member.role || may_change_role(mine, member.role, *to))
        .collect();
    let remove = member.role < Role::Manager || mine == Role::Owner;
    (if roles.len() > 1 { roles } else { Vec::new() }, remove)
}

/// The roles an account of `mine` invites with, highest first. Pure.
pub(crate) fn invite_roles(mine: Option<Role>) -> Vec<Role> {
    let Some(mine) = mine.filter(|r| r.may_manage()) else {
        return Vec::new();
    };
    BOOK_ROLES
        .iter()
        .copied()
        .filter(|to| *to != Role::Owner && may_change_role(mine, Role::View, *to))
        .collect()
}

/// A notebook's sharing as its sheet shows it. Pure.
pub(crate) fn book_info(view: &BookView) -> BookShareInfo {
    let members = view
        .members
        .iter()
        .map(|m| {
            let (roles, removable) = member_actions(view.my_role, m);
            BookMemberRow {
                account_id: m.account.clone(),
                username: m.username.clone(),
                role: m.role.into(),
                me: m.me,
                standing: m.standing.into(),
                fingerprint: String::new(),
                verified: false,
                roles: roles.into_iter().map(BookRole::from).collect(),
                removable,
            }
        })
        .collect();
    BookShareInfo {
        share_id: view.share_id.clone(),
        notebook_id: view.book_id.clone(),
        name: view.name.clone(),
        my_role: view.my_role.unwrap_or(Role::View).into(),
        owner: view.owner_name.clone(),
        members,
        invite_roles: invite_roles(view.my_role)
            .into_iter()
            .map(BookRole::from)
            .collect(),
        owned: view.my_role == Some(Role::Owner),
    }
}

fn clip(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

impl SharedNotes {
    // ------------------------------------------------------------- sharing

    /// What sharing a notebook would do: how many notes go in, how many
    /// locked ones stay out in a private notebook, and which notes are left
    /// out. Refuses one that can't be shared (locked, already shared).
    pub fn can_share_book(&self, notebook_id: String) -> Result<BookShareCheck, KansoError> {
        guard(|| {
            let engine = self.engine()?;
            let check = engine
                .check_book_shareable(&notebook_id)
                .map_err(|err| collab_error(&err))?;
            Ok(BookShareCheck {
                locked_name: format!("{} (locked)", check.name),
                name: check.name,
                notes: clip(check.eligible.len()),
                locked: clip(check.locked.len()),
                left_out: check.left_out,
            })
        })
    }

    /// Shares a notebook with people, each by username with a role (never
    /// Owner), moving its locked notes to a private "<name> (locked)"
    /// notebook first. Each person's keys are looked up and pinned first.
    /// [`SharedNotes::book_progress`] hears how far it got.
    pub fn share_book(
        &self,
        notebook_id: String,
        invites: Vec<(String, BookRole)>,
    ) -> Result<BookShareInfo, KansoError> {
        guard(|| {
            let engine = self.engine()?;
            let invites: Vec<(String, Role)> = invites
                .into_iter()
                .map(|(username, role)| (username.trim().to_owned(), role.into()))
                .collect();
            for (username, _) in &invites {
                engine.person(username).map_err(|err| collab_error(&err))?;
            }
            let listeners = &self.inner.shared.book_progress;
            let done = engine
                .share_book(&notebook_id, &invites, true, &mut |p| {
                    listeners.send(BookShareProgress {
                        notebook_id: notebook_id.clone(),
                        done: clip(p.done),
                        total: clip(p.total),
                    });
                })
                .map_err(|err| collab_error(&err))?;
            self.tell_notebook(&notebook_id);
            self.view(&engine, &done.share_id, true)
        })
    }

    /// Calls `sink` with how far a notebook being shared has got, until
    /// the sink returns `false`.
    pub fn book_progress(&self, sink: impl FnMut(BookShareProgress) -> bool + Send + 'static) {
        self.inner.shared.book_progress.add(sink);
    }

    /// A notebook's sharing, brought up to date with the server when it can
    /// be reached. `None` for a notebook that isn't shared.
    pub fn book_info(&self, notebook_id: String) -> Result<Option<BookShareInfo>, KansoError> {
        guard(|| {
            let engine = self.engine()?;
            let Some(share_id) = self.book_share(&notebook_id)? else {
                return Ok(None);
            };
            self.view(&engine, &share_id, true).map(Some)
        })
    }

    /// Invites someone to a shared notebook, by username, with a role.
    pub fn invite_to_book(
        &self,
        notebook_id: String,
        username: String,
        role: BookRole,
    ) -> Result<BookShareInfo, KansoError> {
        self.book_change(&notebook_id, move |engine, share| {
            let username = username.trim().to_owned();
            engine.person(&username)?;
            engine.invite_to_book(share, &[(username, role.into())])
        })
    }

    /// Gives a member another role. Lowering one changes the notebook's key.
    pub fn set_book_role(
        &self,
        notebook_id: String,
        account_id: String,
        role: BookRole,
    ) -> Result<BookShareInfo, KansoError> {
        self.book_change(&notebook_id, move |engine, share| {
            engine.set_book_role(share, &account_id, role.into())
        })
    }

    /// Removes a member, or cancels an invitation. They keep what they
    /// already have; the notebook's key changes.
    pub fn remove_from_book(
        &self,
        notebook_id: String,
        account_id: String,
    ) -> Result<BookShareInfo, KansoError> {
        self.book_change(&notebook_id, move |engine, share| {
            engine.remove_book_member(share, &account_id)
        })
    }

    /// Leaves a notebook someone else shared, keeping a private copy.
    pub fn leave_book(&self, notebook_id: String) -> Result<BookEndedRow, KansoError> {
        self.end_book(&notebook_id, false)
    }

    /// Stops sharing a notebook this account owns. Everyone keeps a private
    /// copy of its notes as they are now.
    pub fn stop_book(&self, notebook_id: String) -> Result<BookEndedRow, KansoError> {
        self.end_book(&notebook_id, true)
    }

    /// The words of a notebook's questions: removing `who`, leaving,
    /// stopping. The same as the desktop's.
    pub fn book_words(&self, name: String, who: String) -> BookWords {
        BookWords {
            remove_title: book_copy::remove_title(&who, &name),
            remove_body: book_copy::remove_body(&who),
            leave_title: book_copy::leave_title(&name),
            stop_title: book_copy::stop_title(&name),
        }
    }

    // --------------------------------------------------------- invitations

    /// What the Shared section says where invitations would be once the
    /// sync server refused this Kanso as too old for shared notebooks
    /// (`426 client_too_old`): "Update Kanso to open this shared notebook.",
    /// the desktop's words. None while the server is happy.
    pub fn book_update_line(&self) -> Option<String> {
        update_line(kanso_collab::client_refused())
    }

    /// Notebooks shared with this account that this device hasn't joined:
    /// invitations, and ones another of its devices joined.
    pub fn book_invitations(&self) -> Result<Vec<BookInvitationRow>, KansoError> {
        guard(|| {
            let engine = self.engine()?;
            let books = match engine.joinable_books() {
                Ok(books) => books,
                Err(err) if err.is_offline() => Vec::new(),
                Err(err) => return Err(collab_error(&err)),
            };
            let now = now_ms();
            let mut seen = held(&self.inner.shared.book_seen);
            seen.retain(|share, _| books.iter().any(|b| &b.share_id == share));
            Ok(books
                .into_iter()
                .map(|b| BookInvitationRow {
                    at_ms: *seen.entry(b.share_id.clone()).or_insert(now),
                    invited: b.status == "invited",
                    share_id: b.share_id,
                    from: b.owner_username,
                    role: b.role.into(),
                })
                .collect())
        })
    }

    /// Joins a notebook from an invitation (or one another device joined)
    /// and takes its notes in. Answers with the notebook's library ID.
    pub fn accept_book(&self, share_id: String) -> Result<String, KansoError> {
        guard(|| {
            let engine = self.engine()?;
            let joined = engine
                .join_book(&share_id)
                .map_err(|err| collab_error(&err))?;
            held(&self.inner.shared.book_seen).remove(&share_id);
            self.tell_notebook(&joined.book_id);
            Ok(joined.book_id)
        })
    }

    /// Declines an invitation to a notebook.
    pub fn decline_book(&self, share_id: String) -> Result<(), KansoError> {
        guard(|| {
            self.engine()?
                .decline_book(&share_id)
                .map_err(|err| collab_error(&err))?;
            held(&self.inner.shared.book_seen).remove(&share_id);
            Ok(())
        })
    }

    // ------------------------------------------------------------- library

    /// The notebooks shared here, for the library's Shared section, by
    /// name. Empty while nobody is signed in.
    pub fn shared_books(&self) -> Result<Vec<SharedBookRow>, KansoError> {
        guard(|| {
            let engine = match self.engine() {
                Ok(engine) => engine,
                Err(KansoError::SignedOut) => return Ok(Vec::new()),
                Err(err) => return Err(err),
            };
            let mut rows = Vec::new();
            for row in self.inner.store.shared_books()? {
                if row.status != SharedStatus::Active {
                    continue;
                }
                let Ok(view) = engine.book_view(&row.share_id) else {
                    continue;
                };
                rows.push(SharedBookRow {
                    status: self.status_of(&engine, &row.share_id),
                    share_id: row.share_id,
                    notebook_id: view.book_id.clone(),
                    name: view.name.clone(),
                    my_role: view.my_role.unwrap_or(Role::View).into(),
                    owner: view.owner_name.clone(),
                    members: view.members.iter().map(|m| m.username.clone()).collect(),
                });
            }
            rows.sort_by(|a, b| {
                a.name
                    .to_lowercase()
                    .cmp(&b.name.to_lowercase())
                    .then_with(|| a.share_id.cmp(&b.share_id))
            });
            Ok(rows)
        })
    }

    /// Who last changed each note of a shared notebook that this device saw
    /// changed: "Edited by Dan · 2 min ago". Empty for a notebook that
    /// isn't shared, and while nobody is signed in.
    pub fn book_edits(&self, notebook_id: String) -> Result<Vec<NoteEditRow>, KansoError> {
        guard(|| {
            let engine = match self.engine() {
                Ok(engine) => engine,
                Err(KansoError::SignedOut) => return Ok(Vec::new()),
                Err(err) => return Err(err),
            };
            let Some(share_id) = self.book_share(&notebook_id)? else {
                return Ok(Vec::new());
            };
            let mut rows: Vec<NoteEditRow> = engine
                .book_edits()
                .into_iter()
                .filter(|e| e.share_id == share_id)
                .map(|e| NoteEditRow {
                    note_id: e.note_id,
                    name: e.name,
                    me: e.mine,
                    at_ms: e.at,
                })
                .collect();
            rows.sort_by(|a, b| a.note_id.cmp(&b.note_id));
            Ok(rows)
        })
    }

    /// Whether a shared notebook is up to date. `None` for one that isn't
    /// shared, and while nobody is signed in.
    pub fn book_status(&self, notebook_id: String) -> Result<Option<BookSyncStatus>, KansoError> {
        guard(|| {
            let engine = match self.engine() {
                Ok(engine) => engine,
                Err(KansoError::SignedOut) => return Ok(None),
                Err(err) => return Err(err),
            };
            Ok(self
                .book_share(&notebook_id)?
                .map(|share_id| self.status_of(&engine, &share_id)))
        })
    }

    // ---------------------------------------------------------------- sync

    /// Syncs every notebook shared here; what changed goes into `event`.
    pub(super) fn sync_books(&self, engine: &Engine, event: &mut SharedEvent) {
        for (share_id, result) in engine.sync_books() {
            let book_id = self
                .inner
                .store
                .shared_book(&share_id)
                .ok()
                .flatten()
                .map(|row| row.book_id);
            let offline = matches!(&result, Err(err) if err.is_offline());
            let was = held(&self.inner.shared.book_offline).insert(share_id.clone(), offline);
            let changed = match &result {
                Ok(sync) => {
                    if let Some(ended) = &sync.ended {
                        event.book_notices.push(ended.message.clone());
                        self.close_book_room(&share_id);
                    }
                    sync.pulled > 0 || sync.pushed > 0 || sync.ended.is_some()
                }
                Err(_) => false,
            };
            if (changed || was != Some(offline))
                && let Some(book_id) = book_id
                && !event.notebooks.contains(&book_id)
            {
                event.notebooks.push(book_id);
            }
        }
        event.book_invitations = match engine.book_invitations() {
            Ok(books) => clip(books.len()),
            Err(_) => *held(&self.inner.shared.told_book_invitations),
        };
    }

    // ---------------------------------------------------------------- live

    /// The notebook and log of a note of a notebook shared here.
    pub(super) fn book_note(&self, note_id: &str) -> Result<Option<(String, String)>, KansoError> {
        let store = &self.inner.store;
        let Some(stream) = store.book_stream_of_note(note_id)? else {
            return Ok(None);
        };
        let active = store
            .shared_book(&stream.share_id)?
            .is_some_and(|row| row.status == SharedStatus::Active);
        Ok(active.then_some((stream.share_id, stream.stream)))
    }

    /// Opens the room of the notebook `note_id` is in (once per notebook)
    /// and puts the note in it.
    pub(super) fn open_book_live(
        &self,
        note_id: String,
        share_id: String,
        stream: String,
    ) -> Result<(), KansoError> {
        let engine = self.engine()?;
        let mut rooms = held(&self.inner.shared.book_rooms);
        let stale = rooms.get(&share_id).is_some_and(|room| {
            room.ended.load(Ordering::SeqCst) || !Arc::ptr_eq(&room.engine, &engine)
        });
        if stale {
            rooms.remove(&share_id);
        }
        let room = match rooms.entry(share_id.clone()) {
            std::collections::hash_map::Entry::Occupied(room) => room.into_mut(),
            std::collections::hash_map::Entry::Vacant(slot) => {
                let connected = Arc::new(AtomicBool::new(false));
                let ended = Arc::new(AtomicBool::new(false));
                let notify = book_notify(
                    Arc::downgrade(&self.inner),
                    share_id.clone(),
                    connected.clone(),
                    ended.clone(),
                );
                let live = Engine::go_live_book(&engine, &share_id, notify);
                slot.insert(BookRoom {
                    engine: engine.clone(),
                    notes: HashMap::new(),
                    connected,
                    ended,
                    _live: live,
                })
            }
        };
        room.engine.book_open_note(&share_id, &stream);
        room.notes.insert(note_id, stream);
        Ok(())
    }

    /// Takes the note out of its notebook's room; the room closes with the
    /// notebook's last open note.
    pub(super) fn close_book_live(&self, note_id: &str) {
        let mut rooms = held(&self.inner.shared.book_rooms);
        let Some(share_id) = rooms
            .iter()
            .find(|(_, room)| room.notes.contains_key(note_id))
            .map(|(share, _)| share.clone())
        else {
            return;
        };
        let empty = rooms.get_mut(&share_id).is_some_and(|room| {
            if let Some(stream) = room.notes.remove(note_id) {
                room.engine.book_close_note(&share_id, &stream);
            }
            room.notes.is_empty()
        });
        if empty {
            let room = rooms.remove(&share_id);
            drop(rooms);
            drop(room);
        }
    }

    fn close_book_room(&self, share_id: &str) {
        let room = held(&self.inner.shared.book_rooms).remove(share_id);
        drop(room);
    }

    /// The open notebook note's room: its engine, notebook and log.
    fn book_room_of(&self, note_id: &str) -> Option<(Arc<Engine>, String, String, bool)> {
        let rooms = held(&self.inner.shared.book_rooms);
        rooms.iter().find_map(|(share, room)| {
            room.notes.get(note_id).map(|stream| {
                (
                    room.engine.clone(),
                    share.clone(),
                    stream.clone(),
                    room.connected.load(Ordering::SeqCst) && !room.ended.load(Ordering::SeqCst),
                )
            })
        })
    }

    /// Whether `note_id` is a notebook note whose room is open.
    pub(crate) fn in_book_room(&self, note_id: &str) -> bool {
        self.book_room_of(note_id).is_some()
    }

    pub(super) fn set_book_caret(&self, note_id: &str, anchor: usize, head: usize) -> bool {
        match self.book_room_of(note_id) {
            Some((engine, share, stream, _)) => {
                engine.set_book_caret(&share, &stream, anchor, head);
                true
            }
            None => false,
        }
    }

    /// Other members' carets in an open notebook note, named by the signed
    /// member list. `None` when the note isn't one.
    pub(super) fn book_cursors(&self, note_id: &str) -> Option<Vec<CursorRow>> {
        let (engine, share, stream, _) = self.book_room_of(note_id)?;
        Some(
            engine
                .book_cursors(&share, &stream)
                .into_iter()
                .map(|cursor| CursorRow {
                    account_id: cursor.account,
                    name: cursor.name,
                    anchor: clip(cursor.anchor),
                    head: clip(cursor.head),
                    colour: clip(cursor.colour),
                })
                .collect(),
        )
    }

    /// What the page of a notebook note says of its sharing: the role the
    /// signed list gives this account now, and the room.
    pub(super) fn book_live_status(
        &self,
        note_id: &str,
        share_id: &str,
    ) -> Result<LiveStatus, KansoError> {
        let room = self.book_room_of(note_id);
        let connected = room.as_ref().is_some_and(|r| r.3);
        let engine = match room {
            Some((engine, ..)) => engine,
            None => self.engine()?,
        };
        let role = engine
            .book_view(share_id)
            .ok()
            .and_then(|v| v.my_role)
            .unwrap_or(Role::View);
        let mut problems: Vec<_> = engine
            .problems()
            .into_iter()
            .filter(|p| p.share_id == share_id)
            .collect();
        problems.sort_by_key(|p| std::cmp::Reverse(p.at));
        Ok(LiveStatus {
            role: role.into(),
            connected,
            ended: false,
            near_limit: false,
            problems: problems.into_iter().map(|p| p.message).collect(),
        })
    }

    /// Leaves every notebook room.
    pub(super) fn close_all_book_rooms(&self) {
        let rooms = std::mem::take(&mut *held(&self.inner.shared.book_rooms));
        drop(rooms);
    }

    #[cfg(any(test, feature = "test-support"))]
    pub(super) fn book_room_live(&self, note_id: &str) -> bool {
        self.book_room_of(note_id).is_some()
    }

    // ------------------------------------------------------------- helpers

    /// The share of a notebook shared here.
    fn book_share(&self, notebook_id: &str) -> Result<Option<String>, KansoError> {
        Ok(self
            .inner
            .store
            .shared_book_of(notebook_id)?
            .filter(|row| row.status == SharedStatus::Active)
            .map(|row| row.share_id))
    }

    fn shared_book(&self, notebook_id: &str) -> Result<(Arc<Engine>, String), KansoError> {
        let engine = self.engine()?;
        let share_id = self
            .book_share(notebook_id)?
            .ok_or_else(|| KansoError::invalid("The notebook isn't shared."))?;
        Ok((engine, share_id))
    }

    /// Runs `work` on a shared notebook, then answers with its sharing.
    fn book_change(
        &self,
        notebook_id: &str,
        work: impl FnOnce(&Engine, &str) -> kanso_collab::api::Result<()>,
    ) -> Result<BookShareInfo, KansoError> {
        guard(|| {
            let (engine, share_id) = self.shared_book(notebook_id)?;
            work(&engine, &share_id).map_err(|err| collab_error(&err))?;
            self.tell_notebook(notebook_id);
            self.view(&engine, &share_id, true)
        })
    }

    fn end_book(&self, notebook_id: &str, stop: bool) -> Result<BookEndedRow, KansoError> {
        guard(|| {
            let (engine, share_id) = self.shared_book(notebook_id)?;
            let owned =
                engine.book_view(&share_id).ok().and_then(|v| v.my_role) == Some(Role::Owner);
            if stop && !owned {
                return Err(KansoError::invalid(
                    "Only the notebook's owner can stop sharing it. You can leave it instead.",
                ));
            }
            if !stop && owned {
                return Err(KansoError::invalid(
                    "You own this notebook. Stop sharing it instead of leaving.",
                ));
            }
            self.close_book_room(&share_id);
            let ended = if stop {
                engine.stop_book(&share_id)
            } else {
                engine.leave_book(&share_id)
            }
            .map_err(|err| collab_error(&err))?;
            self.tell_notebook(&ended.book_id);
            Ok(BookEndedRow {
                notebook_id: ended.book_id,
                copy_name: ended.copy_name,
                message: ended.message,
            })
        })
    }

    /// A notebook's sharing; `fresh` brings the chain up to the server's
    /// first (offline, what this device trusts).
    fn view(
        &self,
        engine: &Engine,
        share_id: &str,
        fresh: bool,
    ) -> Result<BookShareInfo, KansoError> {
        let view = if fresh {
            engine.book_view_fresh(share_id)
        } else {
            engine.book_view(share_id)
        }
        .map_err(|err| collab_error(&err))?;
        let mut info = book_info(&view);
        for member in &mut info.members {
            if let Ok(person) = engine.person_of(&member.account_id) {
                member.fingerprint = person.fingerprint;
                member.verified = person.verified;
            }
        }
        Ok(info)
    }

    fn status_of(&self, engine: &Engine, share_id: &str) -> BookSyncStatus {
        let live = held(&self.inner.shared.book_rooms)
            .get(share_id)
            .is_some_and(|room| room.connected.load(Ordering::SeqCst));
        let offline = !live
            && held(&self.inner.shared.book_offline)
                .get(share_id)
                .copied()
                .unwrap_or(false);
        BookSyncStatus {
            offline,
            waiting: clip(engine.book_waiting(share_id).unwrap_or(0)),
            live,
        }
    }

    /// Tells the listeners a notebook's sharing changed.
    fn tell_notebook(&self, notebook_id: &str) {
        self.inner.shared.listeners.send(SharedEvent {
            notebooks: vec![notebook_id.to_owned()],
            invitations: *held(&self.inner.shared.told_invitations),
            book_invitations: *held(&self.inner.shared.told_book_invitations),
            ..SharedEvent::default()
        });
    }
}

/// What a notebook's room tells the shell: the notes and notebook it is
/// about, as a [`SharedEvent`].
fn book_notify(
    inner: Weak<Inner>,
    share_id: String,
    connected: Arc<AtomicBool>,
    ended: Arc<AtomicBool>,
) -> BookNotify {
    Arc::new(move |notice| {
        let Some(inner) = inner.upgrade() else {
            return;
        };
        let open = |stream: Option<&str>| -> Vec<String> {
            held(&inner.shared.book_rooms)
                .get(&share_id)
                .map(|room| {
                    room.notes
                        .iter()
                        .filter(|(_, s)| stream.is_none_or(|want| want == s.as_str()))
                        .map(|(note, _)| note.clone())
                        .collect::<BTreeSet<_>>()
                        .into_iter()
                        .collect()
                })
                .unwrap_or_default()
        };
        let notebook = || {
            inner
                .store
                .shared_book(&share_id)
                .ok()
                .flatten()
                .map(|row| vec![row.book_id])
                .unwrap_or_default()
        };
        let base = SharedEvent {
            invitations: *held(&inner.shared.told_invitations),
            book_invitations: *held(&inner.shared.told_book_invitations),
            ..SharedEvent::default()
        };
        let event = match notice {
            BookLiveNotice::Notes { .. } => SharedEvent {
                notebooks: notebook(),
                ..base
            },
            BookLiveNotice::Text { stream, .. } => {
                inner.editors.wake();
                SharedEvent {
                    note_ids: open(Some(&stream)),
                    notebooks: notebook(),
                    ..base
                }
            }
            BookLiveNotice::Presence { stream, .. } => SharedEvent {
                live_notes: open(Some(&stream)),
                ..base
            },
            BookLiveNotice::Comments { stream, .. } => SharedEvent {
                comment_notes: open(Some(&stream)),
                ..base
            },
            BookLiveNotice::Connection { live, .. } => {
                connected.store(live, Ordering::SeqCst);
                if live {
                    held(&inner.shared.book_offline).insert(share_id.clone(), false);
                }
                SharedEvent {
                    live_notes: open(None),
                    notebooks: notebook(),
                    ..base
                }
            }
            BookLiveNotice::Ended { .. } => {
                ended.store(true, Ordering::SeqCst);
                SharedEvent {
                    note_ids: open(None),
                    notebooks: notebook(),
                    ..base
                }
            }
        };
        inner.shared.listeners.send(event);
    })
}

/// The Shared section's update line, given whether the server refused this
/// Kanso.
fn update_line(refused: bool) -> Option<String> {
    refused.then(|| kanso_collab::UPDATE_FOR_INVITATION.to_owned())
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_update_line_shows_only_once_the_server_refused_and_says_to_update() {
        assert_eq!(super::update_line(false), None);
        assert_eq!(
            super::update_line(true).as_deref(),
            Some("Update Kanso to open this shared notebook.")
        );
    }

    use kanso_collab::MemberStanding;
    use proptest::prelude::*;

    use super::*;

    fn roles() -> impl Strategy<Value = Role> {
        prop::sample::select(BOOK_ROLES.to_vec())
    }

    fn standing() -> impl Strategy<Value = MemberStanding> {
        prop::sample::select(vec![
            MemberStanding::Member,
            MemberStanding::Invited,
            MemberStanding::Left,
            MemberStanding::Unknown,
        ])
    }

    fn views() -> impl Strategy<Value = BookView> {
        (
            prop::collection::vec((roles(), standing()), 0..8),
            prop::option::of(roles()),
            0usize..8,
        )
            .prop_map(|(others, mine, me)| BookView {
                share_id: "s".into(),
                book_id: "book:b".into(),
                name: "Studio".into(),
                owner_id: "a_00".into(),
                owner_name: "user0".into(),
                my_role: mine,
                epoch: 1,
                members: std::iter::once((Role::Owner, MemberStanding::Member))
                    .chain(others.into_iter().filter(|(r, _)| *r != Role::Owner))
                    .enumerate()
                    .map(|(i, (role, standing))| BookMemberView {
                        account: format!("a_{i:02}"),
                        username: format!("user{i}"),
                        role,
                        standing,
                        me: i == me,
                    })
                    .collect(),
                uploading: false,
            })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(128))]

        /// Every role a menu offers is one the engine's rank rule takes
        /// (or the member's own), the menus never touch the owner or this
        /// account, and only an owner or manager sees any menu.
        #[test]
        fn prop_menus_offer_only_what_the_engine_takes(view in views()) {
            let info = book_info(&view);
            prop_assert_eq!(info.members.len(), view.members.len());
            for (row, member) in info.members.iter().zip(&view.members) {
                prop_assert_eq!(&row.account_id, &member.account);
                prop_assert_eq!(row.role, BookRole::from(member.role));
                if row.me || member.role == Role::Owner {
                    prop_assert!(row.roles.is_empty() && !row.removable);
                }
                if !view.my_role.is_some_and(Role::may_manage) {
                    prop_assert!(row.roles.is_empty() && !row.removable);
                }
                for to in &row.roles {
                    let to = Role::from(*to);
                    let mine = view.my_role.unwrap();
                    prop_assert!(to == member.role || may_change_role(mine, member.role, to));
                }
                if !row.roles.is_empty() {
                    prop_assert!(row.roles.contains(&row.role));
                }
            }
            prop_assert_eq!(info.owned, view.my_role == Some(Role::Owner));
        }

        /// Invitations never offer Owner; a manager's stay below Manager.
        #[test]
        fn prop_invite_roles_stay_below_the_inviter(mine in prop::option::of(roles())) {
            let offered = invite_roles(mine);
            prop_assert!(!offered.contains(&Role::Owner));
            match mine {
                Some(Role::Owner) => prop_assert_eq!(offered.len(), 4),
                Some(Role::Manager) => prop_assert!(offered.iter().all(|r| *r < Role::Manager)),
                _ => prop_assert!(offered.is_empty()),
            }
            prop_assert!(offered.windows(2).all(|w| w[0] > w[1]));
        }

        /// Every role makes the trip to the shell's role and back.
        #[test]
        fn prop_book_roles_round_trip(role in roles()) {
            prop_assert_eq!(Role::from(BookRole::from(role)), role);
        }
    }
}
