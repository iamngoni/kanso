//! The recording data types, declared again for the generator. What the
//! fields mean is said where the facade declares them
//! (`crates/kanso-mobile/src/recording_model.rs`).

use flutter_rust_bridge::frb;
// Public: the generated code names these types by this module's path.
pub use kanso_mobile::{RecordingEnd, RecordingFound, RecordingSaved, RecordingStatus};

#[frb(mirror(RecordingEnd))]
pub enum _RecordingEnd {
    Cap,
    Device,
    DiskFull,
}

#[frb(mirror(RecordingStatus))]
pub struct _RecordingStatus {
    pub seconds: u64,
    pub level: f32,
    pub paused: bool,
    pub warned: bool,
    pub ended: Option<RecordingEnd>,
}

#[frb(mirror(RecordingSaved))]
pub struct _RecordingSaved {
    pub note_id: String,
    pub file_ids: Vec<String>,
    pub duration_secs: u64,
    pub created: bool,
}

#[frb(mirror(RecordingFound))]
pub struct _RecordingFound {
    pub id: String,
    pub started_ms: i64,
    pub seconds: u64,
    pub text: String,
}
