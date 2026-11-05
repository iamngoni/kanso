//! An open shared note's live room (docs/ACCOUNTS.md › Collaboration › The
//! server: storage and the live room): one thread per open note that keeps a
//! WebSocket to the note's room, catches up over HTTP whenever it (re)joins
//! or sees a gap, sends what this device writes and its caret, and tells the
//! app what changed. Offline, it keeps trying, and the HTTP sync carries on.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use kanso_core::now_ms;

use super::{Engine, lock};
use crate::api::{LiveEvent, LiveLink, Result};
use crate::doc::Field;
use crate::model::{CanvasPointer, Kind, LiveNotice, PresenceMessage};

/// What this device's presence last said: where the caret or the pointer
/// was.
#[derive(Clone, Debug, PartialEq)]
enum Placed {
    Caret(usize, usize),
    Pointer(CanvasPointer),
}

type LastPresence = Option<(Instant, Option<Placed>)>;

/// How long one wait for the room lasts: also how often the outbox and the
/// caret are looked at.
const TICK: Duration = Duration::from_millis(100);
/// Presence is sent at most this often, and again after this long anyway,
/// so someone who joins the room sees everyone's caret within it: the room
/// doesn't say when someone joins.
const PRESENCE_EVERY: Duration = Duration::from_millis(150);
const PRESENCE_REFRESH: Duration = Duration::from_secs(3);
/// How long to wait before joining again, doubling up to the most.
const RETRY_FIRST: Duration = Duration::from_secs(1);
const RETRY_MOST: Duration = Duration::from_secs(30);
/// How often the HTTP sync runs while connected (compaction, missed
/// entries).
const RESYNC_EVERY: Duration = Duration::from_secs(60);

/// What the app is told.
pub type Notify = Arc<dyn Fn(LiveNotice) + Send + Sync>;

/// Keeps one note's room open until dropped.
pub struct LiveHandle {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
    share_id: String,
}

impl LiveHandle {
    pub fn share_id(&self) -> &str {
        &self.share_id
    }
}

impl Drop for LiveHandle {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            // The thread notices within a tick; it isn't waited for.
            drop(thread);
        }
    }
}

impl Engine {
    /// Opens the note's room for as long as the handle lives. The editor
    /// attaches first ([`Engine::attach`]).
    pub fn go_live(engine: &Arc<Engine>, share_id: &str, notify: Notify) -> LiveHandle {
        let stop = Arc::new(AtomicBool::new(false));
        let thread = {
            let engine = engine.clone();
            let stop = stop.clone();
            let share_id = share_id.to_owned();
            std::thread::Builder::new()
                .name("kanso-collab-live".into())
                .spawn(move || engine.live_loop(&share_id, &stop, &notify))
                .ok()
        };
        LiveHandle {
            stop,
            thread,
            share_id: share_id.to_owned(),
        }
    }

    fn live_loop(&self, share_id: &str, stop: &AtomicBool, notify: &Notify) {
        let mut retry = RETRY_FIRST;
        while !stop.load(Ordering::Relaxed) {
            // Catch up first, so the room starts from a known place.
            let _ = self.sync_note(share_id);
            notify(LiveNotice::Text {
                share_id: share_id.to_owned(),
            });
            notify(LiveNotice::Comments {
                share_id: share_id.to_owned(),
            });
            if self.ended(share_id) {
                notify(LiveNotice::Ended {
                    share_id: share_id.to_owned(),
                });
                return;
            }
            match self.api.live(&self.account.caller, share_id) {
                Ok(link) => {
                    retry = RETRY_FIRST;
                    notify(LiveNotice::Connection {
                        share_id: share_id.to_owned(),
                        live: true,
                    });
                    let ended = self.in_room(share_id, link, stop, notify);
                    notify(LiveNotice::Connection {
                        share_id: share_id.to_owned(),
                        live: false,
                    });
                    if ended {
                        notify(LiveNotice::Ended {
                            share_id: share_id.to_owned(),
                        });
                        return;
                    }
                }
                Err(err) if err.status() == Some(404) => {
                    // No longer in the note, or not yet accepted.
                    let _ = self.refresh();
                    if self.ended(share_id) {
                        notify(LiveNotice::Ended {
                            share_id: share_id.to_owned(),
                        });
                        return;
                    }
                }
                Err(_) => {}
            }
            // Wait, a tick at a time so dropping the handle ends it soon.
            let until = Instant::now() + retry;
            while Instant::now() < until && !stop.load(Ordering::Relaxed) {
                std::thread::sleep(TICK);
            }
            retry = (retry * 2).min(RETRY_MOST);
        }
        self.leave_room(share_id);
    }

    /// Whether the note stopped being shared with this account.
    fn ended(&self, share_id: &str) -> bool {
        !matches!(
            self.store.shared_note(share_id),
            Ok(Some(note)) if note.status == kanso_store::SharedStatus::Active
        )
    }

    /// Forgets what was in flight on a room that closed: the HTTP sync
    /// sends it again.
    fn leave_room(&self, share_id: &str) {
        if let Ok(loaded) = self.loaded(share_id) {
            let mut guard = lock(&loaded);
            guard.in_flight.clear();
            guard.cursors.clear();
        }
    }

    /// Runs one connection until it drops or the handle does. Whether the
    /// note ended (removed, or sharing stopped).
    fn in_room(
        &self,
        share_id: &str,
        mut link: Box<dyn LiveLink>,
        stop: &AtomicBool,
        notify: &Notify,
    ) -> bool {
        let mut last_presence: LastPresence = None;
        let mut last_sync = Instant::now();
        let ended = loop {
            if stop.load(Ordering::Relaxed) {
                break false;
            }
            match link.recv(TICK) {
                Ok(Some(event)) => match self.on_event(share_id, event, notify) {
                    Ok(Some(ended)) => break ended,
                    Ok(None) => {}
                    Err(_) => break false,
                },
                Ok(None) => {}
                Err(_) => break false,
            }
            if self.send_outbox(share_id, link.as_mut()).is_err() {
                break false;
            }
            if self
                .send_presence(share_id, link.as_mut(), &mut last_presence)
                .is_err()
            {
                break false;
            }
            if last_sync.elapsed() >= RESYNC_EVERY {
                last_sync = Instant::now();
                if self.sync_note(share_id).is_ok() {
                    notify(LiveNotice::Text {
                        share_id: share_id.to_owned(),
                    });
                }
            }
            self.flush_mentions(share_id);
            self.flush_canvas(share_id);
        };
        self.leave_room(share_id);
        ended
    }

    /// One event from the room. `Some(ended)` when the connection is over.
    fn on_event(&self, share_id: &str, event: LiveEvent, notify: &Notify) -> Result<Option<bool>> {
        match event {
            LiveEvent::Hello { epoch, .. } | LiveEvent::Epoch(epoch) => {
                let behind = {
                    let loaded = self.loaded(share_id)?;
                    lock(&loaded).note.epoch < epoch
                };
                if behind {
                    self.refresh()?;
                    if self.ended(share_id) {
                        return Ok(Some(true));
                    }
                    self.sync_note(share_id)?;
                    notify(LiveNotice::Text {
                        share_id: share_id.to_owned(),
                    });
                }
            }
            LiveEvent::Stored { seq, envelope } => {
                let gap = {
                    let loaded = self.loaded(share_id)?;
                    let mut guard = lock(&loaded);
                    let gap = seq > Engine::known_seq_of(&guard) + 1;
                    guard.inbox.insert(seq, envelope);
                    gap
                };
                if gap {
                    // Something went by while away: the log has it.
                    let _ = self.sync_note(share_id);
                }
                let (text, comments) = self.merge_now(share_id)?;
                if text {
                    notify(LiveNotice::Text {
                        share_id: share_id.to_owned(),
                    });
                }
                if comments {
                    notify(LiveNotice::Comments {
                        share_id: share_id.to_owned(),
                    });
                }
                let refresh = {
                    let loaded = self.loaded(share_id)?;
                    std::mem::take(&mut lock(&loaded).needs_refresh)
                };
                if refresh {
                    self.refresh()?;
                    let _ = self.sync_note(share_id);
                }
            }
            LiveEvent::Presence { envelope } => {
                if self.take_presence(share_id, &envelope)? {
                    notify(LiveNotice::Presence {
                        share_id: share_id.to_owned(),
                    });
                }
            }
            LiveEvent::Error { code } => {
                // What was in flight is sent again over HTTP.
                if let Ok(loaded) = self.loaded(share_id) {
                    lock(&loaded).in_flight.clear();
                }
                match code.as_str() {
                    "epoch_changed" | "forbidden_role" => {
                        self.refresh()?;
                        if self.ended(share_id) {
                            return Ok(Some(true));
                        }
                        let _ = self.sync_note(share_id);
                    }
                    "compact_first" => {
                        let _ = self.sync_note(share_id);
                    }
                    "rate_limited" => std::thread::sleep(Duration::from_secs(1)),
                    _ => {
                        let _ = self.sync_note(share_id);
                    }
                }
            }
            LiveEvent::Closed { .. } => {
                let _ = self.refresh();
                return Ok(Some(self.ended(share_id)));
            }
        }
        Ok(None)
    }

    pub(crate) fn known_seq_of(loaded: &super::Loaded) -> u64 {
        let mut seq = loaded.note.applied_seq;
        while loaded.inbox.contains_key(&(seq + 1)) {
            seq += 1;
        }
        seq
    }

    /// Sends what this device wrote that isn't on its way yet: sealed
    /// under the note's lock, sent after it is let go.
    fn send_outbox(&self, share_id: &str, link: &mut dyn LiveLink) -> Result<()> {
        let sealed = {
            let loaded = self.loaded(share_id)?;
            let mut guard = lock(&loaded);
            let sent: std::collections::HashSet<i64> = guard.in_flight.values().copied().collect();
            let mut out = Vec::new();
            for outgoing in self.store.outbox(share_id)? {
                if sent.contains(&outgoing.id) {
                    continue;
                }
                let kind = if outgoing.kind == "c" {
                    Kind::Comment
                } else {
                    Kind::Update
                };
                let allowed = match kind {
                    Kind::Comment => guard.role().may_comment(),
                    _ => guard.role().may_edit(),
                };
                if !allowed {
                    continue;
                }
                let (counter, envelope) = self.seal(&mut guard, kind, None, &outgoing.payload)?;
                guard.in_flight.insert(counter, outgoing.id);
                out.push(envelope);
            }
            out
        };
        for envelope in sealed {
            link.send(&envelope)?;
        }
        Ok(())
    }

    /// Sends this device's caret when it moved (at most every 150 ms), and
    /// again every 10 s so others keep showing it.
    fn send_presence(
        &self,
        share_id: &str,
        link: &mut dyn LiveLink,
        last: &mut LastPresence,
    ) -> Result<()> {
        let loaded = self.loaded(share_id)?;
        let mut guard = lock(&loaded);
        let placed = match guard.doc.canvas() {
            Some(_) => guard.pointer.clone().map(Placed::Pointer),
            None => guard
                .caret
                .map(|(anchor, head)| Placed::Caret(anchor, head)),
        };
        let due = match last {
            None => placed.is_some(),
            Some((at, sent)) => {
                (*sent != placed && at.elapsed() >= PRESENCE_EVERY)
                    || (placed.is_some() && at.elapsed() >= PRESENCE_REFRESH)
            }
        };
        let Some(placed) = placed.filter(|_| due) else {
            return Ok(());
        };
        let name = self.account.caller.username.clone();
        let message = match &placed {
            Placed::Caret(anchor, head) => PresenceMessage {
                cursor: guard
                    .doc
                    .text()
                    .map_or_else(String::new, |d| d.sticky(Field::Body, *anchor)),
                head: guard
                    .doc
                    .text()
                    .map_or_else(String::new, |d| d.sticky(Field::Body, *head)),
                name,
                canvas: None,
            },
            Placed::Pointer(pointer) => PresenceMessage {
                cursor: String::new(),
                head: String::new(),
                name,
                canvas: Some(pointer.clone()),
            },
        };
        let plain = serde_json::to_vec(&message).expect("presence serializes");
        let sealed = self.seal_presence(&mut guard, &plain)?;
        drop(guard);
        link.send(&sealed)?;
        *last = Some((Instant::now(), Some(placed)));
        Ok(())
    }

    /// Presence is sealed like any envelope, under counter 0: it is never
    /// stored, so it takes no counter of its own (and writes nothing).
    pub(crate) fn seal_presence(
        &self,
        loaded: &mut super::Loaded,
        plain: &[u8],
    ) -> Result<Vec<u8>> {
        let identity = self.identity()?;
        let key = loaded.keys.get(&loaded.note.epoch).ok_or_else(|| {
            crate::api::CollabError::Crypto("this device has no key for the note".into())
        })?;
        let header = crate::model::Header {
            kind: Kind::Presence,
            epoch: loaded.note.epoch,
            author_account: self.account.caller.account_id.clone(),
            author_device: self.account.caller.device_id.clone(),
            counter: 0,
        };
        Ok(crate::envelope::seal(
            &identity,
            key,
            &loaded.note.share_id,
            &header,
            None,
            plain,
        ))
    }

    /// Someone else's presence. Whether it was taken.
    pub(crate) fn take_presence(&self, share_id: &str, envelope: &[u8]) -> Result<bool> {
        let loaded = self.loaded(share_id)?;
        let mut guard = lock(&loaded);
        let Ok(opened) = self.open_entry(&guard, envelope, None) else {
            return Ok(false);
        };
        let caller = &self.account.caller;
        // Presence only under the note's current epoch.
        if opened.header.kind != Kind::Presence
            || opened.header.epoch != guard.note.epoch
            || (opened.header.author_account == caller.account_id
                && opened.header.author_device == caller.device_id)
        {
            return Ok(false);
        }
        let Ok(mut message) = serde_json::from_slice::<PresenceMessage>(&opened.plain) else {
            return Ok(false);
        };
        // A pointer is somewhere real, or not shown.
        if message
            .canvas
            .as_ref()
            .is_some_and(|p| !(p.x.is_finite() && p.y.is_finite()))
        {
            return Ok(false);
        }
        let key = (
            opened.header.author_account.clone(),
            opened.header.author_device.clone(),
        );
        // A move that arrives after a newer one is ignored.
        if let (Some(new), Some((old, _))) = (&message.canvas, guard.cursors.get(&key))
            && let Some(old) = &old.canvas
            && new.seq < old.seq
        {
            return Ok(false);
        }
        // The name shown is the one the owner's list gives, not the sender's.
        message.name = opened.author_name;
        guard.cursors.insert(key, (message, now_ms()));
        Ok(true)
    }
}
