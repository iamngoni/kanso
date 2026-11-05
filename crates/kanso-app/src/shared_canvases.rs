//! How the local API and MCP change a canvas that is shared (issue #45,
//! step 12): through the signed-in account's engine, as one transaction on
//! the canvas's CRDT, so people drawing on it see the change arrive like a
//! collaborator's. The server holds no engine of its own; it holds this.

use std::sync::{Arc, RwLock};

use kanso_collab::Engine;
use kanso_server::{CanvasChange, SharedCanvases};

/// The engine of the signed-in session, or none while signed out. The
/// server thread reads it; the app sets it when the session starts and ends.
#[derive(Default)]
pub struct EngineHost {
    engine: RwLock<Option<Arc<Engine>>>,
}

impl EngineHost {
    pub fn set(&self, engine: Option<Arc<Engine>>) {
        if let Ok(mut slot) = self.engine.write() {
            *slot = engine;
        }
    }
}

impl SharedCanvases for EngineHost {
    fn edit(&self, note_id: &str, change: &mut CanvasChange<'_>) -> Result<bool, String> {
        let Some(engine) = self.engine.read().ok().and_then(|slot| slot.clone()) else {
            return Ok(false);
        };
        let Some(share_id) = engine.share_of(note_id) else {
            return Ok(false);
        };
        engine
            .edit_canvas(&share_id, |live| {
                change(live).map(|(next, title)| (next, title, ()))
            })
            .map(|()| true)
            .map_err(|err| err.to_string())
    }
}
