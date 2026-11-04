//! Recordings on disk.
//!
//! [`encode::PartWriter`] writes Ogg Opus as audio arrives: 48 kHz, 20 ms
//! frames, parts of at most 9 MiB or 60 minutes, a page every second and an
//! `fsync` every two. [`decode::OpusReader`] reads it back and seeks with the
//! page table in [`index`]. [`source::open`] reads any file Kanso accepts as
//! audio: Ogg Opus through libopus (Symphonia has no Opus decoder), M4A, MP3,
//! FLAC and WAV through Symphonia, telling them apart by the `OpusHead`
//! magic rather than the extension.
//!
//! Nothing here knows about speech models, so playback does not pull in
//! ONNX Runtime; `kanso-voice` depends on this crate, not the other way
//! round.

pub mod decode;
pub mod encode;
pub mod error;
pub mod header;
pub mod index;
pub mod model;
pub mod source;

pub use decode::OpusReader;
pub use encode::{DurableWrite, PartSink, PartWriter};
pub use error::AudioError;
pub use index::PageIndex;
pub use model::*;
pub use source::{PcmSource, open, open_bytes};
