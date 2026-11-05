//! A canvas drawn on with others (#45 step 7), the phone's side of the
//! desktop's shared canvas view.
//!
//! The session still holds a [`Canvas`] and draws on it as it always does.
//! When the note is a shared canvas, the engine holds the truth (a CRDT):
//! the session hands it what was drawn ([`Engine::sync_canvas`]) and takes
//! back what everyone has drawn, and the library's body is the engine's
//! projection, which the session never writes itself. What the desktop's
//! review settled is kept here:
//!
//! - a viewer's session is read only, and so is one opened while nobody is
//!   signed in (the library refuses the write anyway);
//! - the stroke being drawn streams to the others under the ID it keeps;
//!   one the canvas refuses at pen-up, or the pen won't finish, is taken
//!   back, so the others never keep a ghost;
//! - others' edits wait for the shell: the shell calls [`pulse`] when
//!   nothing is in hand, never in the middle of a gesture;
//! - when sharing ends (stopped, left, removed) the canvas is a private one
//!   again, as it stands, with whatever had not been handed over merged in.
//!
//! [`pulse`]: CanvasSession::pulse

use std::collections::HashSet;
use std::time::{Duration, Instant};

use kanso_collab::canvas_doc::StrokeHead;
use kanso_collab::{Engine, Role};
use kanso_core::canvas::{self, Canvas};
use kanso_core::{Note, now_ms};
use kanso_engine::theme::CURSOR_COLOURS;
use std::sync::Arc;

use super::{CanvasSession, State, point};
use crate::boundary::guard;
use crate::canvas_model::{InkStyle, PenSample};
use crate::canvas_shared_model::{CanvasPointerRow, CanvasPulse};
use crate::errors::KansoError;
use crate::library::Library;
use crate::sharing::{SharedNotes, collab_error, share_mark};

/// The room is asked for news at least this often, whatever it said.
const SYNC_EVERY: Duration = Duration::from_secs(2);
/// A pointer is shown while its device has sent it this recently. The room
/// refreshes a held pointer every few seconds and sends nothing for one that
/// went, so a finger lifted from the glass goes from the others' screens in
/// a moment (the engine keeps it for thirty seconds).
const POINTER_FRESH_MS: i64 = 6_000;

/// Where a shared canvas session stands with the room.
pub(super) struct Link {
    shared: SharedNotes,
    engine: Arc<Engine>,
    share_id: String,
    note_id: String,
    /// This account, the author of the strokes streamed from here.
    me: String,
    role: Role,
    /// The canvas is past what a note can hold.
    full: bool,
    /// The ID of the stroke being drawn, once its points have started to
    /// stream.
    pub(super) stroke: Option<String>,
    /// The session's `edits` as of the last sync.
    synced_edits: u64,
    synced_at: Instant,
}

/// How a canvas note opens.
pub(super) enum Opening {
    /// Not shared: the library's body is the canvas.
    Private(Canvas),
    /// Shared, but the room can't be reached (nobody is signed in): the
    /// body as last written, to look at.
    Offline(Canvas),
    /// Shared and in the room: the engine's canvas.
    Live(Box<Link>, Canvas),
}

impl Library {
    /// Works out how canvas `note` opens: through the room when it is a
    /// shared canvas, else from the library's body.
    pub(super) fn open_canvas(&self, note: &Note, body: Canvas) -> Result<Opening, KansoError> {
        if share_mark(&note.extra).is_none() {
            return Ok(Opening::Private(body));
        }
        let shared = self.shared();
        match shared.canvas_room(&note.id) {
            Ok(None) => Ok(Opening::Private(body)),
            Ok(Some(room)) => {
                let canvas = room
                    .engine
                    .canvas(&room.share_id)
                    .map_err(|err| collab_error(&err))?;
                Ok(Opening::Live(
                    Box::new(Link {
                        me: room.engine.caller().account_id.clone(),
                        engine: room.engine,
                        share_id: room.share_id,
                        note_id: note.id.clone(),
                        role: room.role,
                        shared,
                        full: false,
                        stroke: None,
                        synced_edits: 0,
                        synced_at: Instant::now(),
                    }),
                    canvas,
                ))
            }
            Err(KansoError::SignedOut) => Ok(Opening::Offline(body)),
            Err(err) => Err(err),
        }
    }
}

impl Link {
    /// Whether this account may draw in the room.
    pub(super) fn may_edit(&self) -> bool {
        self.role.may_edit()
    }
}

impl State {
    /// Whether the canvas takes drawing: it doesn't for a viewer.
    pub(super) fn writable(&self) -> Result<(), KansoError> {
        if self.read_only {
            return Err(KansoError::invalid(
                "This canvas is shared to look at: you can't draw on it.",
            ));
        }
        Ok(())
    }

    /// Whether the canvas is past what a shared note can hold.
    pub(super) fn full(&self) -> bool {
        self.link.as_ref().is_some_and(|link| link.full)
    }
}

impl CanvasSession {
    /// A shared canvas's tick, every few hundred milliseconds while it is
    /// open and nothing is in hand: hands the room what was drawn, takes
    /// what the others drew when the room says there is any, and says where
    /// their pointers are. The canvas comes back only when it changed, so a
    /// quiet room costs the shell nothing. A private canvas has no room:
    /// nothing comes back.
    pub fn pulse(&self, known: Vec<String>) -> Result<CanvasPulse, KansoError> {
        guard(|| {
            let mut state = self.lock();
            let Some(link) = state.link.as_ref() else {
                return Ok(CanvasPulse::default());
            };
            if link.stroke.is_some() {
                // The pen is down: the news waits for it (see `sync_locked`).
                return Ok(CanvasPulse {
                    view: None,
                    pointers: pointers_of(&state),
                });
            }
            let waiting = link.shared.take_waiting(&link.note_id);
            // Sharing ending is found at once, not at the next sync.
            let ended =
                link.engine.share_of(&link.note_id).as_deref() != Some(link.share_id.as_str());
            let due = waiting
                || ended
                || state.edits != link.synced_edits
                || state.dirty
                || link.synced_at.elapsed() >= SYNC_EVERY;
            let changed = due && self.sync_locked(&mut state)?;
            let view =
                changed.then(|| super::view(&state, &self.titles(), &known.into_iter().collect()));
            Ok(CanvasPulse {
                view,
                pointers: pointers_of(&state),
            })
        })
    }

    /// Hands the room what was drawn and takes back what it holds now.
    /// Whether the canvas changed. The caller has no gesture in hand.
    pub(super) fn sync_locked(&self, state: &mut State) -> Result<bool, KansoError> {
        let Some(link) = state.link.as_mut() else {
            return Ok(false);
        };
        if link.engine.share_of(&link.note_id).as_deref() != Some(link.share_id.as_str()) {
            self.unshare(state);
            return Ok(true);
        }
        if link.stroke.is_some() {
            // A pen is down on a stroke the others already see: the room's
            // canvas has it, half drawn, and taking that in now would leave
            // the finished stroke beside it. The room waits for the pen.
            return Ok(false);
        }
        let role = link.engine.role(&link.share_id).unwrap_or(Role::View);
        let merge = link
            .engine
            .sync_canvas(&link.share_id, &state.canvas)
            .map_err(|err| collab_error(&err))?;
        let moved = role != link.role || merge.full != link.full;
        link.role = role;
        link.full = merge.full;
        link.synced_at = Instant::now();
        state.read_only = !role.may_edit();
        state.dirty = false;
        if merge.changed {
            state.canvas = merge.canvas;
            state.edits += 1;
        }
        if let Some(link) = state.link.as_mut() {
            link.synced_edits = state.edits;
        }
        Ok(merge.changed || moved)
    }

    /// The canvas stopped being shared (stopped, left, or this account was
    /// removed): it is a private canvas again. The library holds the
    /// canvas as the room last had it; what this session drew since is
    /// merged in and saved with the next save.
    fn unshare(&self, state: &mut State) {
        state.link = None;
        state.shared = false;
        state.read_only = false;
        if let Ok((note, theirs)) = self.lib.read_canvas(&state.id) {
            state.canvas = canvas::merge(&state.canvas, &theirs);
            state.rev = note.rev.unwrap_or_default();
        }
        state.dirty = true;
        state.edits += 1;
    }

    /// Streams the stroke being drawn to the others (the shell calls it
    /// about every 150 ms with all the stroke's samples so far), so they
    /// see the ink as it is made. `true` when someone erased the stroke
    /// meanwhile: the pen's stroke stops there, and the shell takes the
    /// erase in once the gesture is over. A private canvas, a canvas that
    /// is full, and a stroke that is still a dot stream nothing.
    pub fn stream_stroke(
        &self,
        style: InkStyle,
        samples: Vec<PenSample>,
    ) -> Result<bool, KansoError> {
        guard(|| {
            let mut state = self.lock();
            if state.link.is_none() || state.full() {
                return Ok(false);
            }
            state.writable()?;
            if samples.len() < 2 {
                return Ok(false);
            }
            let raw: Vec<_> = samples.iter().map(point).collect();
            let target = state.canvas.stream_target(now_ms());
            if target.layer_made {
                // An ink layer made just now goes first: the points are
                // relative to it, and the others hide ink that has none.
                state.edits += 1;
                self.sync_locked(&mut state)?;
            }
            let taken: HashSet<String> = state
                .canvas
                .stroke_ids()
                .into_iter()
                .map(str::to_owned)
                .collect();
            let Some(link) = state.link.as_mut() else {
                return Ok(false);
            };
            let id = link
                .stroke
                .get_or_insert_with(|| canvas::unique_id("s", &taken))
                .clone();
            let head = StrokeHead {
                id,
                ink: target.ink,
                owner: link.me.clone(),
                tool: style.tool,
                color: style.color,
                width: style.width,
                z: target.z,
            };
            let points = canvas::streamed_points(&raw, target.origin);
            match link.engine.stream_stroke(&link.share_id, &head, &points) {
                Ok(Ok(_)) => Ok(false),
                Ok(Err(_erased)) => {
                    link.stroke = None;
                    Ok(true)
                }
                Err(err) => Err(collab_error(&err)),
            }
        })
    }

    /// The pen won't finish the stroke (the system took the pointer, the
    /// screen closed under it): what the others already got of it goes.
    pub fn cancel_stroke(&self) -> Result<(), KansoError> {
        guard(|| {
            let mut state = self.lock();
            self.withdraw(&mut state);
            Ok(())
        })
    }

    /// Takes back the streamed stroke, if there is one.
    pub(super) fn withdraw(&self, state: &mut State) {
        let Some(link) = state.link.as_mut() else {
            return;
        };
        let Some(id) = link.stroke.take() else {
            return;
        };
        if let Err(err) = link.engine.abandon_stroke(&link.share_id, &id) {
            eprintln!("kanso: taking back a stroke on the shared canvas: {err}");
        }
    }

    /// Tells the room where this pointer is on the canvas, in canvas
    /// coordinates, for the dot others see. Nothing is sent for a pointer
    /// that hasn't moved; a private canvas has no one to tell.
    pub fn set_pointer(&self, x: f64, y: f64, tool: String) {
        let state = self.lock();
        if let Some(link) = &state.link
            && x.is_finite()
            && y.is_finite()
        {
            link.engine
                .set_pointer(&link.share_id, Some((x, y, tool.as_str())));
        }
    }

    /// The pointer left the canvas: the others' dot of it goes.
    pub fn clear_pointer(&self) {
        let state = self.lock();
        if let Some(link) = &state.link {
            link.engine.set_pointer(&link.share_id, None);
        }
    }

    /// Other people's pointers on the canvas now.
    pub fn pointers(&self) -> Vec<CanvasPointerRow> {
        pointers_of(&self.lock())
    }

    /// The app came back to the front: joins the room again (the library
    /// leaves every room in the background).
    pub fn reconnect(&self) -> Result<(), KansoError> {
        guard(|| {
            let state = self.lock();
            match &state.link {
                Some(link) => link.shared.rejoin_canvas(&link.note_id),
                None => Ok(()),
            }
        })
    }

    /// The screen closed: takes back a stroke still under the pen, writes
    /// what was drawn, and leaves the room. The engine writes the library's
    /// body from the shared document as the room is left.
    pub fn close(&self) -> Result<(), KansoError> {
        guard(|| {
            let mut state = self.lock();
            if state.link.is_none() {
                return Ok(());
            }
            self.withdraw(&mut state);
            if let Some(link) = &state.link {
                link.engine.set_pointer(&link.share_id, None);
            }
            let synced = if state.read_only {
                Ok(false)
            } else {
                self.sync_locked(&mut state)
            };
            if let Some(link) = state.link.take() {
                link.shared.close_live(link.note_id);
            }
            synced.map(|_| ())
        })
    }
}

/// Other people's pointers, with the colours they get.
fn pointers_of(state: &State) -> Vec<CanvasPointerRow> {
    let Some(link) = &state.link else {
        return Vec::new();
    };
    let now = now_ms();
    link.engine
        .pointers(&link.share_id)
        .into_iter()
        .filter(|p| now - p.seen_at < POINTER_FRESH_MS)
        .map(|p| {
            let (light, dark) = CURSOR_COLOURS[p.colour % CURSOR_COLOURS.len()];
            CanvasPointerRow {
                account_id: p.account,
                name: p.name,
                x: p.x,
                y: p.y,
                tool: p.tool,
                colour: u32::try_from(p.colour).unwrap_or(0),
                colour_light: light.hex(),
                colour_dark: dark.hex(),
            }
        })
        .collect()
}
