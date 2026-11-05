//! Telling speakers apart (issue #38, step 13): who spoke each paragraph of
//! a recording, found on this device.
//!
//! Two small models do it, together about 28 MB: pyannote's segmentation
//! says who is speaking where in each ten seconds, and WeSpeaker's
//! embedding gives each person's stretch a voiceprint. Voiceprints that
//! are close are one person. Nothing is sent anywhere.
//!
//! [`assign::names`] turns that into the labels in the note: "Speaker 1",
//! "Speaker 2"... in the order they first speak, with the microphone of a
//! two-track recording staying "You".

pub mod assign;
pub mod cluster;
pub mod engine;
pub mod fbank;
pub mod model;
pub mod powerset;
pub mod timeline;

pub use assign::{names, speaker_name, voice_in};
pub use engine::Diarizer;
pub use model::{Diarization, EMBEDDING, Pinned, SEGMENTATION, Turn, total_size};
