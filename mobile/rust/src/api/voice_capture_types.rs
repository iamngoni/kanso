//! The voice capture data types, declared again for the generator, as
//! `types.rs` does for the rest. What the fields mean is said where the
//! facade declares them (`crates/kanso-mobile/src/voice_capture_model.rs`).

use flutter_rust_bridge::frb;
// Public: the generated code names these types by this module's path.
pub use kanso_mobile::{
    TidyOutcome, VoiceCaptureInput, VoiceCaptureSaved, VoiceDrop, VoiceNoteInfo,
};

#[frb(mirror(VoiceCaptureInput))]
pub struct _VoiceCaptureInput {
    pub id: String,
    pub started_ms: i64,
    pub duration_ms: u64,
    pub transcript: String,
    pub on_device: bool,
}

#[frb(mirror(VoiceCaptureSaved))]
pub struct _VoiceCaptureSaved {
    pub note_id: Option<String>,
    pub problems: Vec<String>,
    pub existed: bool,
    pub meeting: bool,
    pub tidy_now: bool,
}

#[frb(mirror(VoiceDrop))]
pub struct _VoiceDrop {
    pub id: String,
    pub audio_path: String,
    pub started_ms: i64,
    pub duration_ms: u64,
}

#[frb(mirror(VoiceNoteInfo))]
pub struct _VoiceNoteInfo {
    pub tidy: String,
    pub duration_ms: u64,
    pub can_tidy: bool,
}

#[frb(mirror(TidyOutcome))]
pub struct _TidyOutcome {
    pub inserted_above: bool,
    pub dropped: u32,
}
