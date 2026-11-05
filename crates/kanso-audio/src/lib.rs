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
//! [`session::Recorder`] writes a recording into a private in-progress
//! folder with a `session.json` sidecar, a 6-hour cap and an `fsync` of the
//! folder when a part is created; [`capture::Capture`] feeds it from the
//! microphone at 48 kHz; [`recovery`] finds and finishes what a crash left.
//!
//! [`system_audio`] is the platform-neutral way to record what the computer
//! is playing: WASAPI loopback on Windows, a PulseAudio or PipeWire monitor
//! source on Linux (macOS plugs in the same traits).
//!
//! Nothing here knows about speech models, so playback does not pull in
//! ONNX Runtime; `kanso-voice` depends on this crate, not the other way
//! round.

pub mod capture;
pub mod decode;
pub mod encode;
pub mod error;
pub mod header;
pub mod index;
#[cfg(any(windows, target_os = "linux"))]
mod loopback;
pub mod mic;
pub mod model;
pub mod play;
pub mod record_model;
pub mod recovery;
pub mod resample;
pub mod session;
pub mod source;
pub mod system_audio;
#[cfg(target_os = "linux")]
mod system_audio_linux;
#[cfg(windows)]
mod system_audio_windows;

pub use capture::{Capture, CaptureStatus, Pipeline, StopReason, Stopped};
pub use decode::OpusReader;
pub use encode::{DurableWrite, PartSink, PartWriter};
pub use error::AudioError;
pub use index::PageIndex;
pub use model::*;
pub use play::{Player, Status as PlayStatus};
pub use record_model::*;
pub use session::{Recorder, RecorderOptions};
pub use source::{PcmSource, open, open_bytes};
pub use system_audio::{
    Availability, SystemAudioBackend, SystemAudioDevice, SystemAudioSource, SystemAudioTarget,
    Unavailable, platform_backend,
};
