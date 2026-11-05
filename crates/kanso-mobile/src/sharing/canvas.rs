//! A shared canvas on a phone (#45 step 7): the room, the engine and the
//! share a canvas session draws through.

use std::sync::Arc;
use std::sync::atomic::Ordering;

use kanso_collab::{Engine, Role};
use kanso_store::SharedKind;

use super::SharedNotes;
use crate::boundary::held;
use crate::errors::KansoError;

/// A canvas note's place in the shared-note engine.
pub(crate) struct CanvasRoom {
    pub(crate) engine: Arc<Engine>,
    pub(crate) share_id: String,
    pub(crate) role: Role,
}

impl SharedNotes {
    /// The engine and share of canvas note `note_id`, with its live room
    /// opened (once; opening it again does nothing). `None` when the note is
    /// not shared (a mark left behind by a share that ended).
    /// [`KansoError::SignedOut`] while nobody is signed in: the canvas is
    /// shown as it was last written, read only.
    pub(crate) fn canvas_room(&self, note_id: &str) -> Result<Option<CanvasRoom>, KansoError> {
        self.refuse_locked(note_id)?;
        let engine = self.engine()?;
        let Some(share_id) = engine.share_of(note_id) else {
            return Ok(None);
        };
        if engine.kind(&share_id) != Some(SharedKind::Canvas) {
            return Err(KansoError::invalid("That note isn't a shared canvas."));
        }
        self.open_live(note_id.to_owned())?;
        let role = engine.role(&share_id).unwrap_or(Role::View);
        Ok(Some(CanvasRoom {
            engine,
            share_id,
            role,
        }))
    }

    /// Whether the room said edits are waiting since this was last asked.
    pub(crate) fn take_waiting(&self, note_id: &str) -> bool {
        held(&self.inner.shared.rooms)
            .get(note_id)
            .is_some_and(|room| room.waiting.swap(false, Ordering::SeqCst))
    }

    /// Opens the canvas's room again after the app came back to the front
    /// (the library closes every room in the background). Nothing when it
    /// is open.
    pub(crate) fn rejoin_canvas(&self, note_id: &str) -> Result<(), KansoError> {
        if self.merges(note_id) {
            return Ok(());
        }
        self.open_live(note_id.to_owned())
    }
}
