//! The Rename Speakers data types, declared again for the generator, as
//! `types.rs` does for the rest. What the fields mean is said where the
//! facade declares them
//! (`crates/kanso-mobile/src/recording_speakers_model.rs`).

use flutter_rust_bridge::frb;
// Public: the generated code names these types by this module's path.
pub use kanso_mobile::{SpeakerChange, SpeakerNameView, SpeakersRenamed};

#[frb(mirror(SpeakerNameView))]
pub struct _SpeakerNameView {
    pub name: String,
    pub paragraphs: u32,
}

#[frb(mirror(SpeakerChange))]
pub struct _SpeakerChange {
    pub from: String,
    pub to: String,
}

#[frb(mirror(SpeakersRenamed))]
pub struct _SpeakersRenamed {
    pub undo: Option<u64>,
}
