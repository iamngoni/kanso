//! Shared notes: sharing, invitations, links, joining, and the live room of an
//! open shared note (docs/ACCOUNTS.md › Collaboration).
//!
//! The collab engine is made for the signed-in session the first time it is
//! needed and dropped when the session changes, as the desktop's
//! `collab::engine` does, over the server client the library was opened with
//! (the real one in the app, [`kanso_collab::memory`] in tests). Every call
//! here takes a note by its library ID; the engine's share ID stays inside.
//!
//! A locked note is never shared, shown or synced through here: every call
//! that names a note refuses a locked one first with [`KansoError::Locked`],
//! and nothing here returns a note's title or text at all. The rows name
//! people, roles and notes by ID.
//!
//! An open shared note's room runs while the app is in front. Two threads
//! serve it: the engine's own, which keeps the WebSocket and the catch-up,
//! and a pump that every quarter second hands the library's text to
//! [`Engine::sync_editor`], so a local save goes out and a remote edit is
//! merged into the library at once, where [`crate::Library::changes`]
//! reports it. While an editor session is open on the note it drives the
//! merge itself (`SharedNotes::editor_merge`, when typing pauses and
//! whenever the room says edits are waiting) and the pump stands aside, as
//! the desktop editor does.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use kanso_collab::engine::live::Notify;
use kanso_collab::{Account, Caller, CollabApi, CollabError, Engine, LiveHandle, Role, ShareView};
use kanso_core::now_ms;
use kanso_store::SharedStatus;
use zeroize::Zeroizing;

use crate::boundary::{guard, held};
use crate::errors::KansoError;
use crate::library::Inner;
use crate::sharing_model::{
    CursorRow, InvitationRow, JoinRow, LiveStatus, MemberRow, PersonLookup, PersonRow, ShareInfo,
    ShareLinkRow, ShareRole, SharedEvent,
};
use crate::stream::Listeners;
use crate::syncing::SyncHandle;

mod comments;

/// How often a live note's text is handed to the engine.
const PUMP_EVERY: Duration = Duration::from_millis(250);

/// Shared notes: sharing, invitations, links, joining, and the live room of an
/// open shared note.
#[derive(Clone)]
pub struct SharedNotes {
    pub(crate) inner: Arc<Inner>,
}

/// What a library keeps for shared notes: the server client, the engine of
/// the signed-in session, the open rooms and who listens.
pub(crate) struct SharedState {
    api: Arc<dyn CollabApi>,
    engine: Mutex<Option<Made>>,
    /// Open rooms, by the library's note ID.
    rooms: Mutex<HashMap<String, Room>>,
    listeners: Listeners<SharedEvent>,
    /// When each waiting invitation was first seen, by share ID.
    first_seen: Mutex<HashMap<String, i64>>,
    /// The invitations the last event counted.
    told_invitations: Mutex<u32>,
}

impl SharedState {
    pub(crate) fn new(api: Arc<dyn CollabApi>) -> SharedState {
        SharedState {
            api,
            engine: Mutex::new(None),
            rooms: Mutex::new(HashMap::new()),
            listeners: Listeners::default(),
            first_seen: Mutex::new(HashMap::new()),
            told_invitations: Mutex::new(0),
        }
    }
}

/// An engine and the session it was made for.
struct Made {
    caller: Caller,
    identity_key: Zeroizing<[u8; 32]>,
    vault_key: Zeroizing<[u8; 32]>,
    engine: Arc<Engine>,
}

impl Made {
    /// Whether this engine is for the session `caller` and `roots` come from.
    fn is_for(&self, caller: &Caller, roots: &kanso_sync::CollabRoots) -> bool {
        self.caller.server == caller.server
            && self.caller.account_id == caller.account_id
            && self.caller.device_id == caller.device_id
            && *self.caller.authorization == *caller.authorization
            && *self.identity_key == *roots.identity_key
            && *self.vault_key == *roots.vault_key
    }
}

/// An open shared note: its live room and the pump that feeds it.
struct Room {
    engine: Arc<Engine>,
    share_id: String,
    stop: Arc<AtomicBool>,
    /// The room ended (the note stopped being shared, or this account was
    /// removed): opening the note again makes a new one.
    ended: Arc<AtomicBool>,
    /// An editor session merges for this note: the pump stands aside.
    editor: Arc<AtomicBool>,
    /// Held by the pump for a whole tick and by the editor's merge, so the
    /// pump never hands the engine text it read before the editor took over.
    turn: Arc<Mutex<()>>,
    /// Edits arrived (or the room just opened) that the editor session has
    /// not merged yet.
    waiting: Arc<AtomicBool>,
    /// The room is connected now, as it last said.
    connected: Arc<AtomicBool>,
    wake: Sender<()>,
    _live: LiveHandle,
}

impl Drop for Room {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        // The pump ends when it sees the flag or the sender go.
        let _ = self.wake.send(());
        self.engine.detach(&self.share_id);
    }
}

impl SharedNotes {
    /// Why a note can't be shared, if it can't: it is locked, in a locked
    /// notebook, in the trash, or already shared.
    pub fn can_share(&self, note_id: String) -> Result<(), KansoError> {
        guard(|| {
            self.refuse_locked(&note_id)?;
            self.engine()?
                .check_shareable(&note_id)
                .map_err(|err| collab_error(&err))
        })
    }

    /// Shares a note with people, each by username with a role (never
    /// `Owner`), and returns its sharing. Each person's keys are looked up and
    /// pinned first; keys that changed since they were pinned refuse the
    /// share ([`SharedNotes::lookup_person`] says how).
    pub fn share(
        &self,
        note_id: String,
        invites: Vec<(String, ShareRole)>,
    ) -> Result<ShareInfo, KansoError> {
        guard(|| {
            self.refuse_locked(&note_id)?;
            let engine = self.engine()?;
            let invites: Vec<(String, Role)> = invites
                .into_iter()
                .map(|(username, role)| (username, role.into()))
                .collect();
            engine
                .check_shareable(&note_id)
                .map_err(|err| collab_error(&err))?;
            for (username, _) in &invites {
                engine.person(username).map_err(|err| collab_error(&err))?;
            }
            let share_id = engine
                .share(&note_id, &invites)
                .map_err(|err| collab_error(&err))?;
            self.settled(&engine, &share_id)
        })
    }

    /// A note's sharing, brought up to date with the server when it can be
    /// reached. `None` for a note that isn't shared.
    pub fn info(&self, note_id: String) -> Result<Option<ShareInfo>, KansoError> {
        guard(|| {
            self.refuse_locked(&note_id)?;
            let engine = self.engine()?;
            let Some(share_id) = engine.share_of(&note_id) else {
                return Ok(None);
            };
            match engine.refresh() {
                Ok(_) => {}
                Err(err) if err.is_offline() => {}
                Err(err) => return Err(collab_error(&err)),
            }
            // The note may have ended in that refresh.
            if engine.share_of(&note_id).is_none() {
                return Ok(None);
            }
            self.info_of(&engine, &share_id).map(Some)
        })
    }

    /// Invites someone by username with a role.
    pub fn invite(
        &self,
        note_id: String,
        username: String,
        role: ShareRole,
    ) -> Result<ShareInfo, KansoError> {
        self.change(&note_id, move |engine, share| {
            engine.person(&username)?;
            engine.invite(share, &username, role.into())
        })
    }

    /// Gives a member another role.
    pub fn set_role(
        &self,
        note_id: String,
        account_id: String,
        role: ShareRole,
    ) -> Result<ShareInfo, KansoError> {
        self.change(&note_id, move |engine, share| {
            engine.set_role(share, &account_id, role.into())
        })
    }

    /// Removes a member, or cancels an invitation. They keep what they had.
    pub fn remove(&self, note_id: String, account_id: String) -> Result<ShareInfo, KansoError> {
        self.change(&note_id, move |engine, share| {
            engine.remove(share, &account_id)
        })
    }

    /// Lets in someone who asked through a link, with the link's role.
    pub fn approve_request(
        &self,
        note_id: String,
        account_id: String,
    ) -> Result<ShareInfo, KansoError> {
        self.change(&note_id, move |engine, share| {
            engine.approve(share, &account_id)
        })
    }

    pub fn decline_request(
        &self,
        note_id: String,
        account_id: String,
    ) -> Result<ShareInfo, KansoError> {
        self.change(&note_id, move |engine, share| {
            engine.decline_request(share, &account_id)
        })
    }

    /// Makes an invite link with a role (never `Owner`), good for `days` (1 to
    /// 30), and returns its URL. The secret in it is known only here: the
    /// rows never carry it.
    pub fn create_link(
        &self,
        note_id: String,
        role: ShareRole,
        days: u32,
    ) -> Result<String, KansoError> {
        guard(|| {
            let (engine, share_id) = self.shared(&note_id)?;
            engine
                .create_link(&share_id, role.into(), i64::from(days))
                .map_err(|err| collab_error(&err))
        })
    }

    pub fn cancel_link(&self, note_id: String, link_id: String) -> Result<ShareInfo, KansoError> {
        self.change(&note_id, move |engine, share| {
            engine.cancel_link(share, &link_id)
        })
    }

    /// Asks to join the note an invite link is for. Its owner lets this
    /// account in; the note comes in with a sync after that. (The note has
    /// no ID here until then, so this answers with the request.)
    pub fn join(&self, link: String) -> Result<JoinRow, KansoError> {
        guard(|| {
            let engine = self.engine()?;
            let joined = engine.join(link.trim()).map_err(|err| collab_error(&err))?;
            Ok(JoinRow {
                share_id: joined.share_id,
                owner: joined.owner_username,
                role: joined.role.into(),
            })
        })
    }

    /// Stops sharing a note this account owns. Everyone keeps a private copy
    /// of its text as it is.
    pub fn stop(&self, note_id: String) -> Result<(), KansoError> {
        guard(|| {
            let (engine, share_id) = self.shared(&note_id)?;
            if engine.role(&share_id) != Some(Role::Owner) {
                return Err(KansoError::invalid(
                    "Only the note's owner can stop sharing it. You can leave it instead.",
                ));
            }
            self.close_live(note_id.clone());
            engine.stop(&share_id).map_err(|err| collab_error(&err))
        })
    }

    /// Leaves a note someone else shared, keeping a private copy.
    pub fn leave(&self, note_id: String) -> Result<(), KansoError> {
        guard(|| {
            let (engine, share_id) = self.shared(&note_id)?;
            if engine.role(&share_id) == Some(Role::Owner) {
                return Err(KansoError::invalid(
                    "You own this note. Stop sharing it instead of leaving.",
                ));
            }
            self.close_live(note_id.clone());
            engine.leave(&share_id).map_err(|err| collab_error(&err))
        })
    }

    /// Notes shared with this account that wait to be accepted, brought up
    /// to date with the server when it can be reached.
    pub fn invitations(&self) -> Result<Vec<InvitationRow>, KansoError> {
        guard(|| {
            let engine = self.engine()?;
            match engine.refresh() {
                Ok(_) => {}
                Err(err) if err.is_offline() => {}
                Err(err) => return Err(collab_error(&err)),
            }
            Ok(self.invitation_rows(&engine))
        })
    }

    /// Accepts an invitation and takes the note in. Answers with the note's ID.
    pub fn accept(&self, share_id: String) -> Result<String, KansoError> {
        guard(|| {
            let engine = self.engine()?;
            engine.accept(&share_id).map_err(|err| collab_error(&err))?;
            held(&self.inner.shared.first_seen).remove(&share_id);
            self.inner
                .store
                .shared_note(&share_id)?
                .filter(|note| note.status == SharedStatus::Active)
                .map(|note| note.note_id)
                .ok_or_else(|| KansoError::invalid("That invitation is gone."))
        })
    }

    pub fn decline(&self, share_id: String) -> Result<(), KansoError> {
        guard(|| {
            let engine = self.engine()?;
            engine
                .decline(&share_id)
                .map_err(|err| collab_error(&err))?;
            held(&self.inner.shared.first_seen).remove(&share_id);
            Ok(())
        })
    }

    /// Looks someone up by username before sharing: their keys, pinned the
    /// first time, with the fingerprint to compare out loud.
    pub fn lookup_person(&self, username: String) -> Result<PersonLookup, KansoError> {
        guard(|| {
            let engine = self.engine()?;
            match engine.person(&username) {
                Ok(person) => Ok(PersonLookup::Found(person_row(&person))),
                Err(CollabError::KeyChanged(name)) => {
                    let offered = engine
                        .offered_key(&name)
                        .map_err(|err| collab_error(&err))?;
                    let pinned = engine.person_of(&offered.account_id).ok();
                    Ok(PersonLookup::Changed {
                        pinned: pinned.as_ref().map(person_row),
                        offered: person_row(&offered),
                    })
                }
                Err(err) => Err(collab_error(&err)),
            }
        })
    }

    /// Marks someone's fingerprint as compared and matching.
    pub fn verify_person(&self, account_id: String) -> Result<(), KansoError> {
        guard(|| {
            self.engine()?
                .verify_person(&account_id)
                .map_err(|err| collab_error(&err))
        })
    }

    /// Pins the keys the server now gives for someone whose keys changed,
    /// after the person compared the fingerprint `offered` showed. Keys with
    /// another fingerprint are refused.
    pub fn accept_new_key(
        &self,
        username: String,
        fingerprint: String,
    ) -> Result<PersonRow, KansoError> {
        guard(|| {
            self.engine()?
                .accept_new_key(&username, &fingerprint)
                .map(|person| person_row(&person))
                .map_err(|err| collab_error(&err))
        })
    }

    /// Syncs shared notes now: the account's shares and invitations, every
    /// shared note's log, and mentions. Answers with what it found, and tells
    /// the [`SharedNotes::events`] listeners when that is news.
    pub fn sync_shared(&self) -> Result<SharedEvent, KansoError> {
        guard(|| {
            let engine = self.engine()?;
            let store = &self.inner.store;
            // The notes that are, or are about to stop being, shared.
            let mut notes: HashSet<String> = store
                .shared_notes()?
                .into_iter()
                .map(|note| note.note_id)
                .collect();
            let changes = store.subscribe();
            let comments_before = comment_counts(store);
            let result = engine.sync_all();
            notes.extend(store.shared_notes()?.into_iter().map(|note| note.note_id));
            let mut note_ids = Vec::new();
            while let Ok(change) = changes.try_recv() {
                if notes.contains(&change.id) && !note_ids.contains(&change.id) {
                    note_ids.push(change.id);
                }
            }
            let invitations = u32::try_from(engine.invitations().len()).unwrap_or(u32::MAX);
            let comment_notes = changed_comment_notes(&comments_before, &comment_counts(store));
            let event = SharedEvent {
                note_ids,
                invitations,
                comment_notes,
                live_notes: Vec::new(),
            };
            let news = {
                let mut told = held(&self.inner.shared.told_invitations);
                let news = !event.note_ids.is_empty()
                    || !event.comment_notes.is_empty()
                    || *told != invitations;
                *told = invitations;
                news
            };
            if news {
                self.inner.shared.listeners.send(event.clone());
            }
            result.map_err(|err| collab_error(&err))?;
            Ok(event)
        })
    }

    /// Calls `sink` with what each sync of shared notes finds that is news
    /// (notes changed, a new count of invitations), and when a note's room
    /// ends, until the sink returns `false`.
    pub fn events(&self, sink: impl FnMut(SharedEvent) -> bool + Send + 'static) {
        self.inner.shared.listeners.add(sink);
    }

    /// Opens the note's live room, for as long as it stays open: the person
    /// is in the note, and edits cross both ways within a moment. Remote
    /// merges land in the library, so [`crate::Library::changes`] reports
    /// them. The app closes every room when it goes to the background.
    pub fn open_live(&self, note_id: String) -> Result<(), KansoError> {
        guard(|| {
            let (engine, share_id) = self.shared(&note_id)?;
            let mut rooms = held(&self.inner.shared.rooms);
            if let Some(room) = rooms.get(&note_id) {
                if !room.ended.load(Ordering::SeqCst) && Arc::ptr_eq(&room.engine, &engine) {
                    return Ok(());
                }
                rooms.remove(&note_id);
            }
            engine.attach(&share_id).map_err(|err| collab_error(&err))?;
            let stop = Arc::new(AtomicBool::new(false));
            let ended = Arc::new(AtomicBool::new(false));
            let editor = Arc::new(AtomicBool::new(false));
            let connected = Arc::new(AtomicBool::new(false));
            let turn = Arc::new(Mutex::new(()));
            // A session already open on the note merges as soon as the room
            // is there.
            let waiting = Arc::new(AtomicBool::new(true));
            let (wake, woken) = mpsc::channel::<()>();
            let notify: Notify = {
                let wake = Mutex::new(wake.clone());
                let ended = ended.clone();
                let waiting = waiting.clone();
                let connected = connected.clone();
                let inner = Arc::downgrade(&self.inner);
                let note_id = note_id.clone();
                // The page asks again for what the room says now.
                let tell_live = {
                    let inner = inner.clone();
                    let note_id = note_id.clone();
                    move || {
                        if let Some(inner) = inner.upgrade() {
                            inner.shared.listeners.send(SharedEvent {
                                live_notes: vec![note_id.clone()],
                                invitations: *held(&inner.shared.told_invitations),
                                ..SharedEvent::default()
                            });
                        }
                    }
                };
                Arc::new(move |notice| match notice {
                    kanso_collab::LiveNotice::Text { .. } => {
                        waiting.store(true, Ordering::SeqCst);
                        let _ = held(&wake).send(());
                        if let Some(inner) = inner.upgrade() {
                            inner.editors.wake();
                        }
                    }
                    kanso_collab::LiveNotice::Ended { .. } => {
                        ended.store(true, Ordering::SeqCst);
                        if let Some(inner) = inner.upgrade() {
                            inner.shared.listeners.send(SharedEvent {
                                note_ids: vec![note_id.clone()],
                                invitations: *held(&inner.shared.told_invitations),
                                ..SharedEvent::default()
                            });
                        }
                    }
                    kanso_collab::LiveNotice::Comments { .. } => {
                        if let Some(inner) = inner.upgrade() {
                            inner.shared.listeners.send(SharedEvent {
                                comment_notes: vec![note_id.clone()],
                                invitations: *held(&inner.shared.told_invitations),
                                ..SharedEvent::default()
                            });
                        }
                    }
                    kanso_collab::LiveNotice::Connection { live, .. } => {
                        connected.store(live, Ordering::SeqCst);
                        tell_live();
                    }
                    kanso_collab::LiveNotice::Presence { .. }
                    | kanso_collab::LiveNotice::Problem { .. } => tell_live(),
                })
            };
            let live = Engine::go_live(&engine, &share_id, notify);
            let pump = Pump {
                engine: engine.clone(),
                share_id: share_id.clone(),
                note_id: note_id.clone(),
                stop: stop.clone(),
                editor: editor.clone(),
                turn: turn.clone(),
            };
            let spawned = std::thread::Builder::new()
                .name("kanso-collab-pump".into())
                .spawn(move || pump.run(&woken));
            if let Err(err) = spawned {
                engine.detach(&share_id);
                return Err(KansoError::Internal {
                    message: format!("couldn't start the live note's thread: {err}"),
                });
            }
            rooms.insert(
                note_id,
                Room {
                    engine,
                    share_id,
                    stop,
                    ended,
                    editor,
                    turn,
                    waiting,
                    connected,
                    wake,
                    _live: live,
                },
            );
            drop(rooms);
            self.inner.editors.wake();
            Ok(())
        })
    }

    /// Leaves the note's room.
    pub fn close_live(&self, note_id: String) {
        let room = held(&self.inner.shared.rooms).remove(&note_id);
        drop(room);
    }

    /// Leaves every room: the app went to the background, or the session
    /// changed.
    pub(crate) fn close_all_live(&self) {
        let rooms = std::mem::take(&mut *held(&self.inner.shared.rooms));
        drop(rooms);
    }

    /// Whether a note's room is open (and still running).
    #[cfg(any(test, feature = "test-support"))]
    pub fn is_live(&self, note_id: &str) -> bool {
        held(&self.inner.shared.rooms)
            .get(note_id)
            .is_some_and(|room| !room.ended.load(Ordering::SeqCst))
    }

    /// The open note's editor text, merged with what other members wrote:
    /// takes the editor's `title` and `body` in, writes the merge to the
    /// library and says how the editor's text must change. `None` when the
    /// note has no open room. From the first call on, the pump stands aside
    /// for this note until its room closes.
    pub(crate) fn editor_merge(
        &self,
        note_id: &str,
        title: &str,
        body: &str,
    ) -> Result<Option<kanso_collab::EditorMerge>, KansoError> {
        self.refuse_locked(note_id)?;
        let (engine, share_id, turn) = {
            let rooms = held(&self.inner.shared.rooms);
            let Some(room) = rooms.get(note_id) else {
                return Ok(None);
            };
            if room.ended.load(Ordering::SeqCst) {
                return Ok(None);
            }
            room.waiting.store(false, Ordering::SeqCst);
            (
                room.engine.clone(),
                room.share_id.clone(),
                room.turn.clone(),
            )
        };
        // After a tick the pump may be in, and before any other.
        let _turn = held(&turn);
        if let Some(room) = held(&self.inner.shared.rooms).get(note_id) {
            room.editor.store(true, Ordering::SeqCst);
        }
        let merge = engine
            .sync_editor(&share_id, title, body)
            .map_err(|err| collab_error(&err))?;
        Ok(Some(merge))
    }

    /// Whether the note's room is open, for its editor session to merge
    /// through.
    pub(crate) fn merges(&self, note_id: &str) -> bool {
        held(&self.inner.shared.rooms)
            .get(note_id)
            .is_some_and(|room| !room.ended.load(Ordering::SeqCst))
    }

    /// Whether edits wait in the note's room for its editor session.
    pub(crate) fn edits_waiting(&self, note_id: &str) -> bool {
        held(&self.inner.shared.rooms)
            .get(note_id)
            .is_some_and(|room| {
                !room.ended.load(Ordering::SeqCst) && room.waiting.load(Ordering::SeqCst)
            })
    }

    /// What the page of shared note `note_id` says of its sharing: the
    /// desktop's footer and problems banner. `None` for a note that isn't
    /// shared and never was while open. Signed out, the role is the one the
    /// note was last given, and the room is not connected.
    pub fn live_status(&self, note_id: String) -> Result<Option<LiveStatus>, KansoError> {
        guard(|| {
            self.refuse_locked(&note_id)?;
            let room = held(&self.inner.shared.rooms).get(&note_id).map(|room| {
                (
                    room.engine.clone(),
                    room.share_id.clone(),
                    room.ended.load(Ordering::SeqCst),
                    room.connected.load(Ordering::SeqCst),
                )
            });
            let marked = self
                .inner
                .store
                .note(&note_id)?
                .and_then(|note| share_mark(&note.extra));
            let (engine, share_id, ended, connected) = match (room, marked) {
                (Some(room), _) => room,
                (None, Some((share_id, _))) => match self.engine() {
                    Ok(engine) => (engine, share_id, false, false),
                    Err(KansoError::SignedOut) => {
                        return Ok(marked_status(&self.inner.store, &note_id));
                    }
                    Err(err) => return Err(err),
                },
                (None, None) => return Ok(None),
            };
            let role = engine
                .role(&share_id)
                .or_else(|| marked_role(&self.inner.store, &note_id))
                .unwrap_or(Role::View);
            let mut problems: Vec<_> = engine
                .problems()
                .into_iter()
                .filter(|problem| problem.share_id == share_id)
                .collect();
            problems.sort_by_key(|problem| std::cmp::Reverse(problem.at));
            let near_limit = !ended
                && engine
                    .size_use(&share_id)
                    .is_some_and(|size| size.is_near());
            Ok(Some(LiveStatus {
                role: role.into(),
                connected: connected && !ended,
                ended,
                near_limit,
                problems: problems.into_iter().map(|p| p.message).collect(),
            }))
        })
    }

    /// Forgets what couldn't be done for shared note `note_id`, as the
    /// banner's Dismiss does.
    pub fn dismiss_problems(&self, note_id: String) -> Result<(), KansoError> {
        guard(|| {
            let (engine, share_id) = match held(&self.inner.shared.rooms).get(&note_id) {
                Some(room) => (room.engine.clone(), room.share_id.clone()),
                None => self.shared(&note_id)?,
            };
            engine.dismiss_problems(Some(&share_id));
            Ok(())
        })
    }

    /// This device's caret in the open note (byte offsets of the body), sent
    /// to the others as presence.
    pub(crate) fn set_caret(&self, note_id: &str, anchor: usize, head: usize) {
        let rooms = held(&self.inner.shared.rooms);
        if let Some(room) = rooms.get(note_id) {
            room.engine.set_caret(&room.share_id, anchor, head);
        }
    }

    /// Other members' carets in the open note.
    pub(crate) fn cursors(&self, note_id: &str) -> Vec<CursorRow> {
        let rooms = held(&self.inner.shared.rooms);
        let Some(room) = rooms.get(note_id) else {
            return Vec::new();
        };
        room.engine
            .cursors(&room.share_id)
            .into_iter()
            .map(|cursor| CursorRow {
                account_id: cursor.account,
                name: cursor.name,
                anchor: clip(cursor.anchor),
                head: clip(cursor.head),
                colour: clip(cursor.colour),
            })
            .collect()
    }

    // ------------------------------------------------------------- helpers

    /// The engine of the signed-in session, made now if the session is new.
    /// A session that changed takes the old engine's rooms with it.
    /// [`KansoError::SignedOut`] while nobody is signed in.
    pub(crate) fn engine(&self) -> Result<Arc<Engine>, KansoError> {
        let session = SyncHandle {
            inner: self.inner.clone(),
        }
        .load_session()?;
        let mut made = held(&self.inner.shared.engine);
        let Some(session) = session else {
            if made.take().is_some() {
                drop(made);
                self.close_all_live();
            }
            return Err(KansoError::SignedOut);
        };
        let caller = Caller::of(&session);
        let roots = session.collab_roots();
        if let Some(current) = made.as_ref()
            && current.is_for(&caller, &roots)
        {
            return Ok(current.engine.clone());
        }
        let engine = Arc::new(Engine::new(
            self.inner.store.clone(),
            self.inner.shared.api.clone(),
            Account::of(&session),
        ));
        let replaced = made
            .replace(Made {
                caller,
                identity_key: roots.identity_key.clone(),
                vault_key: roots.vault_key.clone(),
                engine: engine.clone(),
            })
            .is_some();
        drop(made);
        if replaced {
            self.close_all_live();
        }
        Ok(engine)
    }

    /// Refuses a note that is locked or in a locked notebook, and one that
    /// doesn't exist, before anything is asked of the engine.
    fn refuse_locked(&self, note_id: &str) -> Result<(), KansoError> {
        let store = &self.inner.store;
        if store.note(note_id)?.is_none() {
            return Err(KansoError::NotFound);
        }
        if store.note_is_locked(note_id)? || store.note_in_locked_book(note_id)? {
            return Err(KansoError::Locked);
        }
        Ok(())
    }

    /// The engine and the share a note is, for a call that needs it shared.
    fn shared(&self, note_id: &str) -> Result<(Arc<Engine>, String), KansoError> {
        self.refuse_locked(note_id)?;
        let engine = self.engine()?;
        let share_id = engine
            .share_of(note_id)
            .ok_or_else(|| KansoError::invalid("The note isn't shared."))?;
        Ok((engine, share_id))
    }

    /// Runs `work` on a shared note, then answers with its sharing.
    fn change(
        &self,
        note_id: &str,
        work: impl FnOnce(&Engine, &str) -> kanso_collab::api::Result<()>,
    ) -> Result<ShareInfo, KansoError> {
        guard(|| {
            let (engine, share_id) = self.shared(note_id)?;
            work(&engine, &share_id).map_err(|err| collab_error(&err))?;
            self.settled(&engine, &share_id)
        })
    }

    /// A share's rows after a change: the server's view taken again first,
    /// as the desktop's Share sheet does (offline, the last view shows).
    fn settled(&self, engine: &Engine, share_id: &str) -> Result<ShareInfo, KansoError> {
        let _ = engine.refresh();
        self.info_of(engine, share_id)
    }

    fn info_of(&self, engine: &Engine, share_id: &str) -> Result<ShareInfo, KansoError> {
        let view = engine
            .share_view(share_id)
            .ok_or_else(|| KansoError::invalid("Open the note's sharing again."))?;
        Ok(share_info(&view, &engine.caller().account_id))
    }

    fn invitation_rows(&self, engine: &Engine) -> Vec<InvitationRow> {
        let waiting = engine.invitations();
        let now = now_ms();
        let mut seen = held(&self.inner.shared.first_seen);
        seen.retain(|share_id, _| waiting.iter().any(|i| &i.share_id == share_id));
        waiting
            .iter()
            .map(|invitation| InvitationRow {
                share_id: invitation.share_id.clone(),
                title: None,
                from: invitation.owner.username.clone(),
                role: invitation.role.into(),
                at_ms: *seen.entry(invitation.share_id.clone()).or_insert(now),
            })
            .collect()
    }
}

/// Hands the library's text to the engine while a note is live.
struct Pump {
    engine: Arc<Engine>,
    share_id: String,
    note_id: String,
    stop: Arc<AtomicBool>,
    editor: Arc<AtomicBool>,
    turn: Arc<Mutex<()>>,
}

impl Pump {
    fn run(self, woken: &mpsc::Receiver<()>) {
        loop {
            match woken.recv_timeout(PUMP_EVERY) {
                Ok(()) | Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => return,
            }
            // Wakes pile up while a tick runs; one tick covers them.
            while woken.try_recv().is_ok() {}
            if self.stop.load(Ordering::SeqCst) {
                return;
            }
            let _turn = held(&self.turn);
            if self.editor.load(Ordering::SeqCst) {
                continue;
            }
            if self.tick().is_err() {
                return;
            }
        }
    }

    /// One merge. An error means the note is no longer shared here.
    fn tick(&self) -> Result<(), ()> {
        let store = self.engine.store();
        // A locked note's sealed text is never handed on.
        if store.note_is_locked(&self.note_id).unwrap_or(true)
            || store.note_in_locked_book(&self.note_id).unwrap_or(true)
        {
            return Ok(());
        }
        let Ok(Some(note)) = store.note(&self.note_id) else {
            return Ok(());
        };
        match self
            .engine
            .sync_editor(&self.share_id, &note.title, &note.body)
        {
            Err(CollabError::Refused(_)) => Err(()),
            _ => Ok(()),
        }
    }
}

/// How many comment events the library holds for each active shared note,
/// by library note ID: a sync that brings more has news in comments.
fn comment_counts(store: &kanso_store::Store) -> HashMap<String, usize> {
    store
        .shared_notes()
        .unwrap_or_default()
        .into_iter()
        .filter(|note| note.status == SharedStatus::Active)
        .map(|note| {
            let count = store.comment_events(&note.share_id).map_or(0, |e| e.len());
            (note.note_id, count)
        })
        .collect()
}

/// The notes whose comment events differ between two counts, in a fixed
/// order.
fn changed_comment_notes(
    before: &HashMap<String, usize>,
    after: &HashMap<String, usize>,
) -> Vec<String> {
    let mut changed: Vec<String> = after
        .iter()
        .filter(|(note, count)| before.get(*note) != Some(count))
        .map(|(note, _)| note.clone())
        .collect();
    changed.sort();
    changed
}

/// The share a note's mark says it is, and this account's role in it, from
/// the library: what is known of a shared note signed out.
fn share_mark(extra: &serde_json::Map<String, serde_json::Value>) -> Option<(String, Role)> {
    let mark = extra.get(kanso_core::model::SHARE_FIELD)?.as_object()?;
    let share_id = mark.get("shareId")?.as_str()?.to_owned();
    let role = mark
        .get("role")
        .and_then(|r| r.as_str())
        .and_then(Role::parse)
        .unwrap_or(Role::View);
    Some((share_id, role))
}

fn marked_role(store: &kanso_store::Store, note_id: &str) -> Option<Role> {
    let note = store.note(note_id).ok()??;
    share_mark(&note.extra).map(|(_, role)| role)
}

/// A marked note's status with no engine: its role, and nothing live.
fn marked_status(store: &kanso_store::Store, note_id: &str) -> Option<LiveStatus> {
    marked_role(store, note_id).map(|role| LiveStatus {
        role: role.into(),
        connected: false,
        ended: false,
        near_limit: false,
        problems: Vec::new(),
    })
}

/// A byte offset or count as the shell's `u32`.
fn clip(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

fn person_row(person: &kanso_collab::Person) -> PersonRow {
    PersonRow {
        account_id: person.account_id.clone(),
        username: person.username.clone(),
        fingerprint: person.fingerprint.clone(),
        verified: person.verified,
    }
}

/// A share's view as the rows the shell shows. The owner comes first, then
/// the others as the signed list has them.
pub(crate) fn share_info(view: &ShareView, me: &str) -> ShareInfo {
    let mut members: Vec<MemberRow> = view
        .members
        .iter()
        .map(|member| MemberRow {
            account_id: member.person.account_id.clone(),
            username: member.person.username.clone(),
            role: member.role.into(),
            me: member.person.account_id == me,
            fingerprint: member.person.fingerprint.clone(),
            key_changed: member.unverified,
            invited: member.invited,
            left: member.left,
            verified: member.person.verified,
        })
        .collect();
    members.sort_by_key(|member| member.role != ShareRole::Owner);
    ShareInfo {
        share_id: view.share_id.clone(),
        note_id: view.note_id.clone(),
        my_role: view.role.into(),
        owner: view.owner.username.clone(),
        members,
        links: view
            .links
            .iter()
            .map(|link| ShareLinkRow {
                id: link.link_id.clone(),
                role: link.role.into(),
                expires_ms: link.expires_at,
            })
            .collect(),
        requests: view
            .requests
            .iter()
            .map(|request| MemberRow {
                account_id: request.person.account_id.clone(),
                username: request.person.username.clone(),
                role: request.role.into(),
                me: false,
                fingerprint: request.person.fingerprint.clone(),
                key_changed: false,
                invited: false,
                left: false,
                verified: request.person.verified,
            })
            .collect(),
    }
}

/// What a failed sharing step is to the shell, in the desktop's words.
pub(crate) fn collab_error(err: &CollabError) -> KansoError {
    let message = words(err);
    match err {
        CollabError::Transport(_) | CollabError::Protocol(_) => KansoError::Network { message },
        CollabError::Api { status: 401, .. } => KansoError::SignedOut,
        CollabError::Api { status, .. } if *status == 429 || *status >= 500 => {
            KansoError::Network { message }
        }
        CollabError::Store(err) => KansoError::from(clone_store_error(err)),
        _ => KansoError::Invalid { message },
    }
}

/// A store error's kind, rebuilt: the engine keeps the error behind a shared
/// reference.
fn clone_store_error(err: &kanso_store::StoreError) -> kanso_store::StoreError {
    use kanso_store::StoreError;
    match err {
        StoreError::NotFound => StoreError::NotFound,
        StoreError::Conflict => StoreError::Conflict,
        StoreError::Invalid(message) => StoreError::Invalid(message.clone()),
        StoreError::GuardFailed => StoreError::GuardFailed,
        other => StoreError::Io(std::io::Error::other(other.to_string())),
    }
}

/// What a failed sharing step means, in plain words. Never the server's body.
pub(crate) fn words(err: &CollabError) -> String {
    match err {
        CollabError::NoSuchUser(name) => format!(
            "There's no Kanso account called {name} that can share yet. They need to sign in to Kanso once first."
        ),
        CollabError::KeyChanged(name) => {
            format!("{name}'s keys changed since you last shared with them.")
        }
        CollabError::NotShareable(words) | CollabError::Refused(words) => words.clone(),
        CollabError::Transport(_) => {
            "Couldn't reach your sync server. Check your connection and try again.".into()
        }
        CollabError::Api { status: 401, .. } => {
            "Your account signed this device out. Sign in again to share.".into()
        }
        CollabError::Api { status: 403, .. } => "Your role in this note doesn't allow that.".into(),
        CollabError::Api { status: 404, .. } => {
            "Your sync server doesn't have that any more. It may have been stopped or cancelled."
                .into()
        }
        CollabError::Api { status: 409, .. } => {
            "Someone changed the note's sharing at the same time. Try again.".into()
        }
        CollabError::Api { status: 410, .. } => "That invite link has expired.".into(),
        CollabError::Api { status: 413, .. } => {
            "That's more than your sync server takes for a shared note.".into()
        }
        CollabError::Api { status: 429, .. } => {
            "Your sync server is busy. Try again in a minute.".into()
        }
        CollabError::Api { .. } | CollabError::Protocol(_) => {
            "Your sync server gave an answer Kanso doesn't understand.".into()
        }
        CollabError::Store(_) => "The library couldn't be read or written.".into(),
        CollabError::Identity(_) | CollabError::List(_) | CollabError::Crypto(_) => {
            "The note's keys couldn't be checked, so Kanso didn't go on.".into()
        }
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use serde_json::json;

    use super::*;

    fn roles() -> impl Strategy<Value = Role> {
        prop::sample::select(Role::ALL.to_vec())
    }

    fn person(n: u8) -> kanso_collab::Person {
        kanso_collab::Person {
            account_id: format!("a_{n:02}"),
            username: format!("user{n}"),
            fingerprint: format!("fp {n}"),
            verified: n.is_multiple_of(2),
        }
    }

    /// A share's view: the owner somewhere in the list, other members, some
    /// links and requests.
    fn views() -> impl Strategy<Value = (ShareView, u8)> {
        (
            prop::collection::vec((roles(), any::<(bool, bool, bool)>()), 1..6),
            prop::collection::vec((0u8..6, roles(), 0i64..1_000_000), 0..4),
            prop::collection::vec((6u8..9, roles()), 0..3),
            roles(),
            0u8..6,
        )
            .prop_map(|(members, links, requests, mine, me)| {
                let view = ShareView {
                    share_id: "s_1".into(),
                    note_id: "note:1".into(),
                    role: mine,
                    owner: person(0),
                    members: members
                        .into_iter()
                        .enumerate()
                        .map(
                            |(n, (role, (invited, left, unverified)))| kanso_collab::MemberView {
                                person: person(u8::try_from(n).unwrap()),
                                role,
                                invited,
                                left,
                                unverified,
                            },
                        )
                        .collect(),
                    links: links
                        .into_iter()
                        .map(|(n, role, at)| kanso_collab::LinkView {
                            link_id: format!("l_{n}"),
                            role,
                            expires_at: at,
                        })
                        .collect(),
                    requests: requests
                        .into_iter()
                        .map(|(n, role)| kanso_collab::RequestView {
                            person: person(n),
                            link_id: "l_0".into(),
                            role,
                            at: 5,
                        })
                        .collect(),
                    epoch: 1,
                };
                (view, me)
            })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        /// Every role survives the trip to the shell's role and back, and
        /// the shell's four roles are the engine's four.
        #[test]
        fn prop_roles_round_trip(role in roles()) {
            let shared = ShareRole::from(role);
            prop_assert_eq!(Role::from(shared), role);
            prop_assert_eq!(shared == ShareRole::Owner, role == Role::Owner);
            prop_assert_eq!(
                matches!(shared, ShareRole::Owner | ShareRole::Editor),
                role.may_edit()
            );
            prop_assert_eq!(
                !matches!(shared, ShareRole::Viewer),
                role.may_comment()
            );
        }

        /// The rows say what the view says: every member and request once,
        /// in order but for the owner first, each role mapped, `me` on the
        /// account's own row only, and no row beyond what the view holds.
        #[test]
        fn prop_rows_say_what_the_view_says((view, me) in views()) {
            let me = format!("a_{me:02}");
            let info = share_info(&view, &me);
            prop_assert_eq!(info.share_id, view.share_id.clone());
            prop_assert_eq!(info.note_id, view.note_id.clone());
            prop_assert_eq!(info.my_role, ShareRole::from(view.role));
            prop_assert_eq!(info.members.len(), view.members.len());
            prop_assert_eq!(info.links.len(), view.links.len());
            prop_assert_eq!(info.requests.len(), view.requests.len());
            // The owners come first, and the rest keep the list's order.
            let owners = view.members.iter().filter(|m| m.role == Role::Owner).count();
            prop_assert!(
                info.members.iter().take(owners).all(|m| m.role == ShareRole::Owner)
            );
            let rest: Vec<_> = info.members.iter().skip(owners).map(|m| &m.account_id).collect();
            let listed: Vec<_> = view
                .members
                .iter()
                .filter(|m| m.role != Role::Owner)
                .map(|m| &m.person.account_id)
                .collect();
            prop_assert_eq!(rest, listed);
            for row in &info.members {
                prop_assert_eq!(row.me, row.account_id == me);
                let source = view
                    .members
                    .iter()
                    .find(|m| m.person.account_id == row.account_id && ShareRole::from(m.role) == row.role)
                    .expect("a row has a member");
                prop_assert_eq!(row.key_changed, source.unverified);
                prop_assert_eq!(row.invited, source.invited);
                prop_assert_eq!(row.left, source.left);
                prop_assert_eq!(&row.fingerprint, &source.person.fingerprint);
            }
            prop_assert!(info.requests.iter().all(|r| !r.me && !r.invited && !r.left));
            let links_match = info.links.iter().zip(&view.links).all(|(row, link)| {
                row.id == link.link_id
                    && row.expires_ms == link.expires_at
                    && Role::from(row.role) == link.role
            });
            prop_assert!(links_match);
        }

        /// Every error says something as a sentence, and never the server's
        /// own words; the kind follows the status.
        #[test]
        fn prop_every_error_is_a_sentence(status in 400u16..600, code in "[a-z_]{1,12}") {
            let err = CollabError::api(status, &code, json!({"secret": "body"}));
            let words = words(&err);
            prop_assert!(words.ends_with('.'));
            prop_assert!(!words.contains("secret"));
            let kind = collab_error(&err);
            match status {
                401 => prop_assert_eq!(kind, KansoError::SignedOut),
                429 | 500..=599 => prop_assert!(is_network(&kind)),
                _ => prop_assert!(is_invalid(&kind)),
            }
        }
    }

    fn is_network(err: &KansoError) -> bool {
        matches!(err, KansoError::Network { .. })
    }

    fn is_invalid(err: &KansoError) -> bool {
        matches!(err, KansoError::Invalid { .. })
    }

    // no-prop-test: a table of the engine's own error kinds.
    #[test]
    fn engine_errors_map_to_the_kinds_the_shell_tells_apart() {
        assert!(matches!(
            collab_error(&CollabError::Transport("dns".into())),
            KansoError::Network { .. }
        ));
        assert_eq!(
            collab_error(&CollabError::NotShareable(
                "The note is in the trash.".into()
            )),
            KansoError::invalid("The note is in the trash.")
        );
        assert!(matches!(
            collab_error(&CollabError::NoSuchUser("bob".into())),
            KansoError::Invalid { message } if message.contains("bob")
        ));
        assert_eq!(
            collab_error(&CollabError::Store(kanso_store::StoreError::NotFound)),
            KansoError::NotFound
        );
    }
}
