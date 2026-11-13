//! A shared notebook's live room (issue #390 › Shared notebooks › Live), the
//! notebook twin of [`super::live`]: one thread per open notebook keeps one
//! WebSocket to the notebook's room, which carries the manifest and every
//! note the app has open ([`Engine::book_open_note`]). Over it go the edits
//! of open notes (as the outbox's updates, sealed as the HTTP sync seals
//! them), comments, and each open note's caret as presence; what other
//! members wrote comes back as a stored entry, which this thread settles by
//! pulling that stream over HTTP, so the room only has to say *that* something
//! changed and the log stays the one source of what did. Offline it keeps
//! trying, and the HTTP sync ([`Engine::sync_book`]) carries on.
//!
//! The library stays the one place text lives: a note's text reaches the
//! room through the outbox ([`Engine::settle_note`] reads what differs from
//! `base`) and comes back to the library through the same settle.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{JoinHandle, ThreadId};
use std::time::{Duration, Instant};

use kanso_core::now_ms;
use kanso_store::SharedStatus;

use super::book::{COMMENT_SUFFIX, Ctx, comment_key, stream_key};
use super::book_sync::{read_base, split_outgoing};
use super::{CURSOR_TTL_MS, Engine, colour_of, lock};
use crate::api::{BookLiveEvent, BookLiveLink, CollabError, MANIFEST_STREAM, Result};
use crate::codec::new_id;
use crate::comments::{self, Logged};
use crate::doc::CollabDoc;
use crate::envelope::{self, parse_header};
use crate::model::{
    BookLiveNotice, CommentEvent, CommentOp, CommentView, Header, Kind, PresenceMessage,
    RemoteCursor, Role,
};

/// How long one wait for the room lasts: also how often edits and carets
/// are looked at.
const TICK: Duration = Duration::from_millis(100);
const PRESENCE_EVERY: Duration = Duration::from_millis(150);
const PRESENCE_REFRESH: Duration = Duration::from_secs(3);
const RETRY_FIRST: Duration = Duration::from_secs(1);
const RETRY_MOST: Duration = Duration::from_secs(30);
/// How often the HTTP sync runs while connected, and how soon after
/// something is queued for a stream that isn't open.
const RESYNC_EVERY: Duration = Duration::from_secs(60);
const HTTP_EVERY: Duration = Duration::from_secs(2);
/// Events taken in one go before the loop settles them.
const BATCH: usize = 64;

/// What the app is told.
pub type BookNotify = Arc<dyn Fn(BookLiveNotice) + Send + Sync>;

/// One step on a shared notebook at a time, by one thread at a time; a
/// thread that holds it may take it again (the sync calls itself through
/// resuming a share).
#[derive(Default)]
pub(crate) struct Gate {
    state: Mutex<(Option<ThreadId>, usize)>,
    free: Condvar,
}

pub(crate) struct GateGuard<'a>(&'a Gate);

impl Gate {
    pub(crate) fn enter(&self) -> GateGuard<'_> {
        let me = std::thread::current().id();
        let mut state = lock(&self.state);
        loop {
            match state.0 {
                None => {
                    *state = (Some(me), 1);
                    break;
                }
                Some(owner) if owner == me => {
                    state.1 += 1;
                    break;
                }
                Some(_) => {
                    state = self
                        .free
                        .wait(state)
                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                }
            }
        }
        GateGuard(self)
    }
}

impl Drop for GateGuard<'_> {
    fn drop(&mut self) {
        let mut state = lock(&self.0.state);
        state.1 -= 1;
        if state.1 == 0 {
            state.0 = None;
            self.0.free.notify_one();
        }
    }
}

/// What the app and the room's thread share about an open notebook.
#[derive(Default)]
pub(crate) struct BookRoom {
    /// The notes the app has open.
    open: BTreeSet<String>,
    /// This device's caret in each: (anchor, head) offsets of the note's text.
    carets: HashMap<String, (usize, usize)>,
    /// Other people's cursors: (stream, account, device).
    cursors: HashMap<(String, String, String), (PresenceMessage, i64)>,
}

/// Keeps one notebook's room open until dropped.
pub struct BookLiveHandle {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
    share_id: String,
}

impl BookLiveHandle {
    pub fn share_id(&self) -> &str {
        &self.share_id
    }
}

impl Drop for BookLiveHandle {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            // The thread notices within a tick; it isn't waited for.
            drop(thread);
        }
    }
}

/// What one connection keeps between events.
struct Session {
    ctx: Option<Ctx>,
    /// Streams opened on the connection.
    opened: BTreeSet<String>,
    /// Entries sent on the room: counter to outbox id, until it echoes them.
    in_flight: HashMap<u64, i64>,
    /// Streams somebody else wrote to, to pull.
    dirty: BTreeSet<String>,
    /// Per stream: when presence last went out, and where the caret was.
    presence: HashMap<String, (Instant, Option<(usize, usize)>)>,
    last_sync: Instant,
    last_http: Instant,
}

impl Session {
    fn new() -> Session {
        Session {
            ctx: None,
            opened: BTreeSet::new(),
            in_flight: HashMap::new(),
            dirty: BTreeSet::new(),
            presence: HashMap::new(),
            last_sync: Instant::now(),
            last_http: Instant::now(),
        }
    }
}

impl Engine {
    /// Opens the notebook's room for as long as the handle lives. Notes the
    /// app has open are named with [`Engine::book_open_note`].
    pub fn go_live_book(
        engine: &Arc<Engine>,
        share_id: &str,
        notify: BookNotify,
    ) -> BookLiveHandle {
        let stop = Arc::new(AtomicBool::new(false));
        let thread = {
            let engine = engine.clone();
            let stop = stop.clone();
            let share_id = share_id.to_owned();
            std::thread::Builder::new()
                .name("kanso-collab-book-live".into())
                .spawn(move || engine.book_live_loop(&share_id, &stop, &notify))
                .ok()
        };
        BookLiveHandle {
            stop,
            thread,
            share_id: share_id.to_owned(),
        }
    }

    fn book_room(&self, share_id: &str) -> Arc<Mutex<BookRoom>> {
        lock(&self.book_rooms)
            .entry(share_id.to_owned())
            .or_default()
            .clone()
    }

    /// The app opened a note of the notebook: its edits, presence and
    /// comments go live.
    pub fn book_open_note(&self, share_id: &str, stream: &str) {
        lock(&self.book_room(share_id))
            .open
            .insert(stream.to_owned());
    }

    pub fn book_close_note(&self, share_id: &str, stream: &str) {
        let room = self.book_room(share_id);
        let mut room = lock(&room);
        room.open.remove(stream);
        room.carets.remove(stream);
        room.cursors.retain(|(s, _, _), _| s != stream);
    }

    /// This device's caret in an open note (offsets of the note's text), sent
    /// as presence by the room.
    pub fn set_book_caret(&self, share_id: &str, stream: &str, anchor: usize, head: usize) {
        lock(&self.book_room(share_id))
            .carets
            .insert(stream.to_owned(), (anchor, head));
    }

    /// Other people's cursors in a note, placed in its current text.
    pub fn book_cursors(&self, share_id: &str, stream: &str) -> Vec<RemoteCursor> {
        let Ok(Some(row)) = self.store.book_stream(share_id, stream) else {
            return Vec::new();
        };
        let Ok(doc) = CollabDoc::load(&row.state) else {
            return Vec::new();
        };
        // Names from the signed member list, never the one a presence
        // message carries.
        let chain = self.book_ctx(share_id).ok().map(|ctx| ctx.chain);
        let room = self.book_room(share_id);
        let mut room = lock(&room);
        let now = now_ms();
        room.cursors.retain(|_, (_, at)| now - *at < CURSOR_TTL_MS);
        let mut out: Vec<RemoteCursor> = room
            .cursors
            .iter()
            .filter(|((s, _, _), _)| s == stream)
            .filter_map(|((_, account, device), (presence, at))| {
                let name = chain
                    .as_ref()
                    .and_then(|chain| super::book_roster::name_in_chain(chain, account))?;
                Some(RemoteCursor {
                    account: account.clone(),
                    device: device.clone(),
                    name,
                    anchor: doc.note_resolve(&presence.cursor)?,
                    head: doc.note_resolve(&presence.head)?,
                    colour: colour_of(account),
                    seen_at: *at,
                })
            })
            .collect();
        out.sort_by(|a, b| (&a.name, &a.device).cmp(&(&b.name, &b.device)));
        out
    }

    // ------------------------------------------------------------ comments

    /// A note's comments, folded, with where each sits in the text now; this
    /// device's unsent ones included as pending.
    pub fn book_comments(&self, share_id: &str, stream: &str) -> Result<Vec<CommentView>> {
        let ctx = self.book_ctx(share_id)?;
        let row = self
            .store
            .book_stream(share_id, stream)?
            .ok_or_else(|| CollabError::Refused("The note isn't in the notebook.".into()))?;
        let doc = CollabDoc::load(&row.state).map_err(|e| CollabError::Crypto(e.to_string()))?;
        let me = self.account.caller.account_id.clone();
        let role = ctx.role(&me).unwrap_or(Role::View);
        let mut events: Vec<Logged> = self
            .store
            .comment_events(&comment_key(share_id, stream))?
            .into_iter()
            .filter_map(|record| {
                Some(Logged {
                    seq: record.seq,
                    author: record.author,
                    author_name: record.author_name,
                    role: Role::parse(&record.role)?,
                    event: serde_json::from_str(&record.json).ok()?,
                })
            })
            .collect();
        let logged = events.len();
        let queued = format!("{stream}{COMMENT_SUFFIX}");
        for (i, outgoing) in self
            .store
            .book_outbox(share_id)?
            .into_iter()
            .filter(|o| o.stream == queued)
            .enumerate()
        {
            if let Ok(event) = serde_json::from_slice::<CommentEvent>(&outgoing.payload) {
                events.push(Logged {
                    seq: u64::MAX / 2 + i as u64,
                    author: me.clone(),
                    author_name: self.account.caller.username.clone(),
                    role,
                    event,
                });
            }
        }
        let pending: HashSet<u64> = events[logged..].iter().map(|e| e.seq).collect();
        let mut out: Vec<CommentView> = comments::fold(&events)
            .into_values()
            .map(|comment| CommentView {
                placement: comments::place(&doc, &comment),
                pending: pending.contains(&comment.seq),
                mine: comment.author == me,
                comment,
            })
            .collect();
        out.sort_by_key(|view| match &view.placement {
            crate::model::Placement::Anchored(range) => (range.start, view.comment.created_at),
            crate::model::Placement::Orphaned(at) => (*at, view.comment.created_at),
        });
        Ok(out)
    }

    fn book_comment_event(
        &self,
        share_id: &str,
        stream: &str,
        event: CommentEvent,
    ) -> Result<String> {
        let ctx = self.book_ctx(share_id)?;
        let role = ctx.role(&self.account.caller.account_id);
        if !role.is_some_and(|r| r.may_comment()) {
            return Err(CollabError::Refused(
                "You can view this note, but not comment on it.".into(),
            ));
        }
        if self.store.book_stream(share_id, stream)?.is_none() {
            return Err(CollabError::Refused(
                "The note isn't in the notebook.".into(),
            ));
        }
        let json = serde_json::to_vec(&event).expect("a comment event serializes");
        if json.len() > envelope::MAX_COMMENT - 512 {
            return Err(CollabError::Refused("The comment is too long.".into()));
        }
        self.store
            .push_book_outbox(share_id, &format!("{stream}{COMMENT_SUFFIX}"), &json)?;
        Ok(event.id)
    }

    /// Comments on the byte range `range` of the note's text.
    pub fn add_book_comment(
        &self,
        share_id: &str,
        stream: &str,
        range: std::ops::Range<usize>,
        body: &str,
    ) -> Result<String> {
        let body = body.trim();
        if body.is_empty() || body.chars().count() > comments::MAX_BODY_CHARS {
            return Err(CollabError::Refused(
                "A comment needs some words, and not too many.".into(),
            ));
        }
        let row = self
            .store
            .book_stream(share_id, stream)?
            .ok_or_else(|| CollabError::Refused("The note isn't in the notebook.".into()))?;
        let doc = CollabDoc::load(&row.state).map_err(|e| CollabError::Crypto(e.to_string()))?;
        self.book_comment_event(
            share_id,
            stream,
            CommentEvent {
                id: new_id(),
                op: CommentOp::Add,
                anchor: Some(comments::anchor(&doc, range)),
                body: Some(body.to_owned()),
                mentions: Vec::new(),
                at: now_ms(),
            },
        )
    }

    pub fn edit_book_comment(
        &self,
        share_id: &str,
        stream: &str,
        comment_id: &str,
        body: &str,
    ) -> Result<()> {
        self.book_comment_event(
            share_id,
            stream,
            CommentEvent {
                id: comment_id.to_owned(),
                op: CommentOp::Edit,
                anchor: None,
                body: Some(body.trim().to_owned()),
                mentions: Vec::new(),
                at: now_ms(),
            },
        )
        .map(|_| ())
    }

    pub fn delete_book_comment(
        &self,
        share_id: &str,
        stream: &str,
        comment_id: &str,
    ) -> Result<()> {
        self.simple_book_comment(share_id, stream, comment_id, CommentOp::Delete)
    }

    pub fn resolve_book_comment(
        &self,
        share_id: &str,
        stream: &str,
        comment_id: &str,
        resolved: bool,
    ) -> Result<()> {
        let op = if resolved {
            CommentOp::Resolve
        } else {
            CommentOp::Reopen
        };
        self.simple_book_comment(share_id, stream, comment_id, op)
    }

    fn simple_book_comment(
        &self,
        share_id: &str,
        stream: &str,
        comment_id: &str,
        op: CommentOp,
    ) -> Result<()> {
        self.book_comment_event(
            share_id,
            stream,
            CommentEvent {
                id: comment_id.to_owned(),
                op,
                anchor: None,
                body: None,
                mentions: Vec::new(),
                at: now_ms(),
            },
        )
        .map(|_| ())
    }

    // ---------------------------------------------------------------- loop

    fn book_live_loop(&self, share_id: &str, stop: &AtomicBool, notify: &BookNotify) {
        let mut retry = RETRY_FIRST;
        while !stop.load(Ordering::Relaxed) {
            // Catch up first, so the room starts from a known place.
            if let Ok(sync) = self.sync_book(share_id) {
                if sync.removed {
                    notify(BookLiveNotice::Ended {
                        share_id: share_id.to_owned(),
                    });
                    return;
                }
                self.told(share_id, &sync, notify);
            }
            if self.book_ended(share_id) {
                notify(BookLiveNotice::Ended {
                    share_id: share_id.to_owned(),
                });
                return;
            }
            match self.api.book_live(&self.account.caller, share_id) {
                Ok(link) => {
                    retry = RETRY_FIRST;
                    notify(BookLiveNotice::Connection {
                        share_id: share_id.to_owned(),
                        live: true,
                    });
                    let ended = self.in_book_room(share_id, link, stop, notify);
                    notify(BookLiveNotice::Connection {
                        share_id: share_id.to_owned(),
                        live: false,
                    });
                    if ended {
                        notify(BookLiveNotice::Ended {
                            share_id: share_id.to_owned(),
                        });
                        return;
                    }
                }
                Err(err) if err.status() == Some(404) => {
                    // No longer in the notebook.
                    let _ = self.sync_book(share_id);
                    if self.book_ended(share_id) {
                        notify(BookLiveNotice::Ended {
                            share_id: share_id.to_owned(),
                        });
                        return;
                    }
                }
                Err(_) => {}
            }
            let until = Instant::now() + retry;
            while Instant::now() < until && !stop.load(Ordering::Relaxed) {
                std::thread::sleep(TICK);
            }
            retry = (retry * 2).min(RETRY_MOST);
        }
    }

    fn book_ended(&self, share_id: &str) -> bool {
        !matches!(
            self.store.shared_book(share_id),
            Ok(Some(row)) if row.status == SharedStatus::Active
        )
    }

    /// Tells the app what a sync changed.
    fn told(&self, share_id: &str, sync: &crate::book_model::BookSync, notify: &BookNotify) {
        if sync.notes_written > 0 || sync.notes_trashed > 0 || sync.pulled > 0 {
            notify(BookLiveNotice::Notes {
                share_id: share_id.to_owned(),
            });
            for stream in lock(&self.book_room(share_id)).open.clone() {
                notify(BookLiveNotice::Text {
                    share_id: share_id.to_owned(),
                    stream: stream.clone(),
                });
                notify(BookLiveNotice::Comments {
                    share_id: share_id.to_owned(),
                    stream,
                });
            }
        }
    }

    /// Runs one connection until it drops or the handle does. Whether the
    /// notebook ended (removed, or sharing stopped).
    fn in_book_room(
        &self,
        share_id: &str,
        mut link: Box<dyn BookLiveLink>,
        stop: &AtomicBool,
        notify: &BookNotify,
    ) -> bool {
        let room = self.book_room(share_id);
        let mut session = Session::new();
        loop {
            if stop.load(Ordering::Relaxed) {
                return false;
            }
            let step = (|| -> Result<Option<bool>> {
                self.reconcile_open(&room, link.as_mut(), &mut session)?;
                let mut wait = TICK;
                for _ in 0..BATCH {
                    match link.recv(wait)? {
                        Some(event) => {
                            if let Some(ended) =
                                self.on_book_event(share_id, event, &room, &mut session, notify)?
                            {
                                return Ok(Some(ended));
                            }
                            wait = Duration::ZERO;
                        }
                        None => break,
                    }
                }
                let _gate = self.book_gate.enter();
                if let Some(ended) = self.settle_dirty(share_id, &mut session, notify)? {
                    return Ok(Some(ended));
                }
                self.flush_book(share_id, link.as_mut(), &mut session, notify)?;
                self.send_book_presence(share_id, link.as_mut(), &room, &mut session)?;
                if session.last_sync.elapsed() >= RESYNC_EVERY {
                    session.last_sync = Instant::now();
                    let sync = self.sync_book(share_id)?;
                    session.ctx = None;
                    if sync.removed {
                        return Ok(Some(true));
                    }
                    self.told(share_id, &sync, notify);
                }
                Ok(None)
            })();
            match step {
                Ok(None) => {}
                Ok(Some(ended)) => return ended,
                Err(_) => return self.book_ended(share_id),
            }
        }
    }

    /// The notes the app has open are open on the connection, and no others.
    fn reconcile_open(
        &self,
        room: &Mutex<BookRoom>,
        link: &mut dyn BookLiveLink,
        session: &mut Session,
    ) -> Result<()> {
        let wanted = lock(room).open.clone();
        for stream in wanted.difference(&session.opened.clone()) {
            link.open(stream)?;
            session.opened.insert(stream.clone());
        }
        for stream in session.opened.clone().difference(&wanted) {
            link.close(stream)?;
            session.opened.remove(stream);
            session.presence.remove(stream);
        }
        Ok(())
    }

    /// This notebook's context, loaded again after anything else changed it.
    fn session_ctx<'a>(&self, share_id: &str, session: &'a mut Session) -> Result<&'a mut Ctx> {
        if session.ctx.is_none() {
            session.ctx = Some(self.book_ctx(share_id)?);
        }
        Ok(session.ctx.as_mut().expect("loaded"))
    }

    /// One event from the room. `Some(ended)` when the connection is over.
    fn on_book_event(
        &self,
        share_id: &str,
        event: BookLiveEvent,
        room: &Mutex<BookRoom>,
        session: &mut Session,
        notify: &BookNotify,
    ) -> Result<Option<bool>> {
        let me = self.account.caller.clone();
        match event {
            BookLiveEvent::Hello { epoch, .. } | BookLiveEvent::Epoch(epoch) => {
                if self.session_ctx(share_id, session)?.row.epoch < epoch {
                    let sync = self.sync_book(share_id)?;
                    session.ctx = None;
                    if sync.removed || self.book_ended(share_id) {
                        return Ok(Some(true));
                    }
                    self.told(share_id, &sync, notify);
                }
            }
            BookLiveEvent::Opened { stream, head } => {
                let behind = self
                    .store
                    .book_stream(share_id, &stream)?
                    .is_none_or(|row| head > row.applied_seq);
                if behind {
                    session.dirty.insert(stream);
                }
            }
            BookLiveEvent::Stored {
                stream,
                seq,
                envelope,
            } => {
                let Some((header, _)) = parse_header(&envelope) else {
                    return Ok(None);
                };
                let mine =
                    header.author_account == me.account_id && header.author_device == me.device_id;
                // Only an envelope that opens (a member's, signed, for this
                // stream) counts as the room taking what this device sent.
                let opened = if mine && session.in_flight.contains_key(&header.counter) {
                    let ctx = self.session_ctx(share_id, session)?;
                    self.open_book_entry(ctx, &stream, &envelope, None).ok()
                } else {
                    None
                };
                if let Some(opened) = opened
                    && let Some(id) = session.in_flight.remove(&header.counter)
                {
                    self.store.drop_book_outbox(id)?;
                    // Its own comment is kept now (a comment shouldn't wait).
                    if header.kind == Kind::Comment && stream != MANIFEST_STREAM {
                        let ctx = self.session_ctx(share_id, session)?;
                        self.keep_book_comment(ctx, &stream, seq, &opened)?;
                        notify(BookLiveNotice::Comments {
                            share_id: share_id.to_owned(),
                            stream,
                        });
                    }
                    return Ok(None);
                }
                session.dirty.insert(stream);
            }
            BookLiveEvent::Presence { stream, envelope } => {
                if self.take_book_presence(share_id, &stream, &envelope, room, session)? {
                    notify(BookLiveNotice::Presence {
                        share_id: share_id.to_owned(),
                        stream,
                    });
                }
            }
            BookLiveEvent::Error { code } => {
                // What was in flight is sent again.
                session.in_flight.clear();
                match code.as_str() {
                    "epoch_changed" | "forbidden_role" => {
                        let sync = self.sync_book(share_id)?;
                        session.ctx = None;
                        if sync.removed || self.book_ended(share_id) {
                            return Ok(Some(true));
                        }
                        self.told(share_id, &sync, notify);
                    }
                    "not_open" => session.opened.clear(),
                    "rate_limited" => std::thread::sleep(Duration::from_secs(1)),
                    _ => {}
                }
            }
            BookLiveEvent::Closed { .. } => {
                let _ = self.sync_book(share_id);
                session.ctx = None;
                return Ok(Some(self.book_ended(share_id)));
            }
        }
        Ok(None)
    }

    /// Pulls what others wrote: the whole notebook for a change to its
    /// manifest (notes added, deleted or restored; its fields), otherwise
    /// each note that changed. `Some(ended)` when the notebook ended.
    fn settle_dirty(
        &self,
        share_id: &str,
        session: &mut Session,
        notify: &BookNotify,
    ) -> Result<Option<bool>> {
        let dirty = std::mem::take(&mut session.dirty);
        if dirty.is_empty() {
            return Ok(None);
        }
        if dirty.contains(MANIFEST_STREAM) {
            let sync = self.sync_book(share_id)?;
            session.ctx = None;
            if sync.removed {
                return Ok(Some(true));
            }
            self.told(share_id, &sync, notify);
            // The notes' own news came with it.
            session.last_sync = Instant::now();
            return Ok(None);
        }
        for stream in dirty {
            let text_before = self.text_of(share_id, &stream);
            let comments_before = self
                .store
                .comment_events(&comment_key(share_id, &stream))?
                .len();
            match self.pull_one_note(share_id, &stream) {
                Ok(()) => {}
                Err(err) if matches!(err.code(), Some("epoch_changed" | "forbidden_role")) => {
                    let sync = self.sync_book(share_id)?;
                    if sync.removed {
                        return Ok(Some(true));
                    }
                    self.told(share_id, &sync, notify);
                }
                Err(err) => return Err(err),
            }
            session.ctx = None;
            if self.text_of(share_id, &stream) != text_before {
                notify(BookLiveNotice::Text {
                    share_id: share_id.to_owned(),
                    stream: stream.clone(),
                });
            }
            if self
                .store
                .comment_events(&comment_key(share_id, &stream))?
                .len()
                != comments_before
            {
                notify(BookLiveNotice::Comments {
                    share_id: share_id.to_owned(),
                    stream,
                });
            }
        }
        Ok(None)
    }

    /// The library's text of a notebook note: title and body.
    fn text_of(&self, share_id: &str, stream: &str) -> Option<(String, String)> {
        let row = self.store.book_stream(share_id, stream).ok().flatten()?;
        let note = self.store.note(&row.note_id?).ok().flatten()?;
        Some((note.title, note.body))
    }

    /// Pulls one note's log and settles the library with it.
    fn pull_one_note(&self, share_id: &str, stream: &str) -> Result<()> {
        let mut ctx = self.book_ctx(share_id)?;
        let Some(role) = ctx.role(&self.account.caller.account_id) else {
            return Ok(());
        };
        let mut result = crate::book_model::BookSync::default();
        self.sync_note_stream(&mut ctx, stream, role, &mut result)?;
        self.save_ctx(&mut ctx)
    }

    /// What the library holds of an open note that isn't in the log yet
    /// becomes updates in the outbox, and what is queued for an open stream
    /// (or the manifest) goes out on the room. Anything queued for a note
    /// that isn't open goes with the HTTP sync, soon.
    fn flush_book(
        &self,
        share_id: &str,
        link: &mut dyn BookLiveLink,
        session: &mut Session,
        notify: &BookNotify,
    ) -> Result<()> {
        let me = self.account.caller.account_id.clone();
        for stream in session.opened.clone() {
            if self.take_local_note(share_id, &stream)? {
                session.ctx = None;
            }
        }
        let outbox = self.store.book_outbox(share_id)?;
        if outbox.is_empty() {
            return Ok(());
        }
        let sent: HashSet<i64> = session.in_flight.values().copied().collect();
        let mut left_for_http = false;
        for outgoing in outbox {
            if sent.contains(&outgoing.id) {
                continue;
            }
            let (stream, kind) = split_outgoing(&outgoing.stream);
            let open_here = stream == MANIFEST_STREAM || session.opened.contains(stream);
            let ctx = self.session_ctx(share_id, session)?;
            let role = ctx.role(&me);
            let allowed = match kind {
                Kind::Comment => role.is_some_and(|r| r.may_comment()),
                _ => role.is_some_and(|r| r.may_edit()),
            };
            if !allowed {
                // The role changed: the HTTP sync drops what can't go.
                left_for_http = true;
                continue;
            }
            if !open_here {
                left_for_http = true;
                continue;
            }
            let sealed = self.seal_book(ctx, stream, kind, None, &outgoing.payload)?;
            let Some((header, _)) = parse_header(&sealed) else {
                continue;
            };
            session.in_flight.insert(header.counter, outgoing.id);
            link.send(stream, &sealed)?;
        }
        if left_for_http && session.last_http.elapsed() >= HTTP_EVERY {
            session.last_http = Instant::now();
            let sync = self.sync_book(share_id)?;
            session.ctx = None;
            self.told(share_id, &sync, notify);
        }
        Ok(())
    }

    /// Takes the library's edit of a note (its text against what the engine
    /// last wrote) into the outbox. Whether it queued anything.
    fn take_local_note(&self, share_id: &str, stream: &str) -> Result<bool> {
        let Some(mut row) = self.store.book_stream(share_id, stream)? else {
            return Ok(false);
        };
        let (Some(note_id), Some(base)) = (row.note_id.clone(), read_base(&row)) else {
            return Ok(false);
        };
        let Some(note) = self.store.note(&note_id)? else {
            return Ok(false);
        };
        if (note.title, note.body) == base {
            return Ok(false);
        }
        let mut ctx = self.book_ctx(share_id)?;
        let Some(role) = ctx.role(&self.account.caller.account_id) else {
            return Ok(false);
        };
        if !role.may_edit() {
            return Ok(false);
        }
        let doc = CollabDoc::load(&row.state).map_err(|e| CollabError::Crypto(e.to_string()))?;
        let mut result = crate::book_model::BookSync::default();
        let edited = self.settle_note(&mut ctx, &mut row, &doc, false, role, &mut result)?;
        self.save_ctx(&mut ctx)?;
        Ok(edited)
    }

    // ------------------------------------------------------------ presence

    /// Sends each open note's caret when it moved (at most every 150 ms), and
    /// again every few seconds so someone who joins sees it.
    fn send_book_presence(
        &self,
        share_id: &str,
        link: &mut dyn BookLiveLink,
        room: &Mutex<BookRoom>,
        session: &mut Session,
    ) -> Result<()> {
        for stream in session.opened.clone() {
            let placed = lock(room).carets.get(&stream).copied();
            let due = match session.presence.get(&stream) {
                None => placed.is_some(),
                Some((at, sent)) => {
                    (*sent != placed && at.elapsed() >= PRESENCE_EVERY)
                        || (placed.is_some() && at.elapsed() >= PRESENCE_REFRESH)
                }
            };
            let Some((anchor, head)) = placed.filter(|_| due) else {
                continue;
            };
            let Some(row) = self.store.book_stream(share_id, &stream)? else {
                continue;
            };
            let doc =
                CollabDoc::load(&row.state).map_err(|e| CollabError::Crypto(e.to_string()))?;
            let message = PresenceMessage {
                cursor: doc.note_sticky(anchor),
                head: doc.note_sticky(head),
                name: self.account.caller.username.clone(),
                canvas: None,
            };
            let plain = serde_json::to_vec(&message).expect("presence serializes");
            let ctx = self.session_ctx(share_id, session)?;
            let sealed = self.seal_book_presence(ctx, &stream, &plain)?;
            link.send(&stream, &sealed)?;
            session
                .presence
                .insert(stream, (Instant::now(), Some((anchor, head))));
        }
        Ok(())
    }

    /// Presence is sealed like any envelope, under counter 0: never stored,
    /// so it takes no counter of its own (and writes nothing).
    fn seal_book_presence(&self, ctx: &Ctx, stream: &str, plain: &[u8]) -> Result<Vec<u8>> {
        let identity = self.identity()?;
        let key = ctx
            .keys
            .get(&ctx.row.epoch)
            .ok_or_else(|| CollabError::Crypto("this device has no key for the notebook".into()))?;
        let header = Header {
            kind: Kind::Presence,
            epoch: ctx.row.epoch,
            author_account: self.account.caller.account_id.clone(),
            author_device: self.account.caller.device_id.clone(),
            counter: 0,
        };
        Ok(envelope::seal(
            &identity,
            &stream_key(key, &ctx.row.share_id, stream),
            &ctx.row.share_id,
            &header,
            None,
            plain,
        ))
    }

    /// Someone else's presence in a note. Whether it was taken.
    fn take_book_presence(
        &self,
        share_id: &str,
        stream: &str,
        envelope: &[u8],
        room: &Mutex<BookRoom>,
        session: &mut Session,
    ) -> Result<bool> {
        let ctx = self.session_ctx(share_id, session)?;
        let Ok(opened) = self.open_book_entry(ctx, stream, envelope, None) else {
            return Ok(false);
        };
        let caller = &self.account.caller;
        // Presence only under the notebook's current epoch, from someone else.
        if opened.header.kind != Kind::Presence
            || opened.header.epoch != ctx.row.epoch
            || (opened.header.author_account == caller.account_id
                && opened.header.author_device == caller.device_id)
        {
            return Ok(false);
        }
        let Ok(mut message) = serde_json::from_slice::<PresenceMessage>(&opened.plain) else {
            return Ok(false);
        };
        // The name shown is the one the owner's list gives, not the sender's.
        message.name = opened.author_name;
        let key = (
            stream.to_owned(),
            opened.header.author_account,
            opened.header.author_device,
        );
        lock(room).cursors.insert(key, (message, now_ms()));
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::super::book_tests::{note_of, notebook, shared};
    use super::super::tests::device;
    use super::*;
    use crate::memory::MemoryCollab;

    /// An echo that doesn't open (forged, or damaged) is not the room taking
    /// what this device sent: the outbox row stays.
    #[test]
    fn a_forged_echo_does_not_drop_the_outbox() {
        let server = MemoryCollab::new();
        let (a, b) = (device(&server, 1, 1), device(&server, 2, 1));
        let (book, _) = notebook(&a.store, "Studio", &[("Plan".to_owned(), "one".to_owned())]);
        let share = shared(&a, &[(&b, Role::Edit)], &book);
        let id = note_of(&a, &book, "Plan");
        let stream = a.store.book_stream_of_note(&id).unwrap().unwrap().stream;
        let row = a
            .store
            .push_book_outbox(&share, &stream, b"ignored")
            .unwrap();
        let mut ctx = a.engine.book_ctx(&share).unwrap();
        let sealed = a
            .engine
            .seal_book(&mut ctx, &stream, Kind::Update, None, b"ignored")
            .unwrap();
        let counter = parse_header(&sealed).unwrap().0.counter;
        let mut forged = sealed.clone();
        let last = forged.len() - 1;
        forged[last] ^= 0xff;
        let room = Arc::new(Mutex::new(BookRoom::default()));
        let notify: BookNotify = Arc::new(|_| {});
        let mut session = Session::new();
        session.in_flight.insert(counter, row);
        let event = |envelope: Vec<u8>| BookLiveEvent::Stored {
            stream: stream.clone(),
            seq: 9,
            envelope,
        };
        a.engine
            .on_book_event(&share, event(forged), &room, &mut session, &notify)
            .unwrap();
        assert_eq!(a.store.book_outbox(&share).unwrap().len(), 1);
        assert!(session.in_flight.contains_key(&counter));
        a.engine
            .on_book_event(&share, event(sealed), &room, &mut session, &notify)
            .unwrap();
        assert!(a.store.book_outbox(&share).unwrap().is_empty());
    }
}
