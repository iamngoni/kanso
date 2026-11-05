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
//! source on Linux, and ScreenCaptureKit on macOS (one app's audio, with
//! the Screen Recording permission asked for only by [`system_apps`]'s
//! callers, in the Record sheet).
//!
//! [`mix`] puts the microphone and system audio on one stereo timeline
//! (left you, right the meeting) and keeps their clocks together;
//! [`speaker`] reads the labels back from the two channels.
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
pub mod mix;
pub mod model;
pub mod play;
pub mod record_model;
pub mod recovery;
pub mod resample;
pub mod session;
pub mod source;
pub mod speaker;
pub mod system_apps;
pub mod system_audio;
#[cfg(target_os = "linux")]
mod system_audio_linux;
#[cfg(target_os = "macos")]
mod system_audio_mac;
#[cfg(windows)]
mod system_audio_windows;

pub use capture::{Capture, CaptureStatus, Pipeline, StopReason, Stopped};
pub use decode::OpusReader;
pub use encode::{DurableWrite, PartSink, PartWriter};
pub use error::AudioError;
pub use index::PageIndex;
pub use mix::{MixPipeline, Mixer};
pub use model::*;
pub use play::{Player, Status as PlayStatus};
pub use record_model::*;
pub use session::{Recorder, RecorderOptions};
pub use source::{PcmSource, open, open_bytes};
pub use speaker::{Speaker, energy, speaker_of};
pub use system_apps::{
    CaptureApp, GapFiller, MEETING_APPS, PcmBuffer, ScreenAccess, access_words, append_mono,
    denied_words, picker_order,
};
#[cfg(target_os = "macos")]
pub use system_audio_mac::{
    access as screen_access, request_access as request_screen_access, running_apps,
};
/// Whether Kanso may record other apps' audio. Only macOS asks, so
/// elsewhere nothing is ever allowed.
#[cfg(not(target_os = "macos"))]
pub fn screen_access() -> ScreenAccess {
    ScreenAccess::NotAllowed
}

/// Asks for Screen Recording; only macOS has such a permission.
#[cfg(not(target_os = "macos"))]
pub fn request_screen_access() -> ScreenAccess {
    ScreenAccess::NotAllowed
}

/// The apps that can be picked; only macOS records one app.
#[cfg(not(target_os = "macos"))]
pub fn running_apps() -> Vec<CaptureApp> {
    Vec::new()
}

pub use system_audio::{
    Availability, SystemAudioBackend, SystemAudioDevice, SystemAudioSource, SystemAudioTarget,
    Unavailable, platform_backend,
};
