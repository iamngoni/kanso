//! The player's data types, declared again for the generator. What the
//! fields mean is said where the facade declares them
//! (`crates/kanso-mobile/src/player_model.rs`).

use flutter_rust_bridge::frb;
// Public: the generated code names these types by this module's path.
pub use kanso_mobile::{NoteRecording, PartsState, PlayerStatus};

#[frb(mirror(PartsState))]
pub enum _PartsState {
    Ready,
    NotHere,
    Missing,
}

#[frb(mirror(NoteRecording))]
pub struct _NoteRecording {
    pub file_ids: Vec<String>,
    pub state: PartsState,
    pub length_ms: Option<u64>,
}

#[frb(mirror(PlayerStatus))]
pub struct _PlayerStatus {
    pub position_ms: u64,
    pub length_ms: u64,
    pub playing: bool,
    pub ended: bool,
    pub speed: f32,
    pub volume: f32,
    pub error: Option<String>,
}
