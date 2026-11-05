//! Dictation on this device: the microphone in, text out, with NVIDIA's
//! Parakeet TDT 0.6B v3 (int8, 25 European languages) on ONNX Runtime.
//!
//! The app downloads the model ([`model::FILES`]) and checks it; this crate
//! only says which files, and runs them. [`Dictation`] records from the
//! default input device at its own rate, resamples to 16 kHz
//! ([`audio::Resampler16k`]), cuts the recording at pauses every 20 seconds
//! or so ([`chunk::plan_cut`]) and transcribes each piece on its own thread
//! while recording goes on ([`engine::Parakeet`]), so stopping waits for the
//! last piece only.
//!
//! [`file::transcribe`] does the same for an audio file, decoding it a
//! piece at a time so a long recording never sits in memory whole.
//!
//! Nothing leaves the device: ONNX Runtime is linked in, runs on the CPU,
//! and its telemetry is off.

pub mod audio;
pub mod chunk;
pub mod decode;
pub mod diarize;
mod dictation;
pub mod engine;
pub mod file;
pub mod live;
pub mod model;
pub mod permission;
pub mod text;

pub use audio::{Resampler16k, downmix, meter_level, resample_to_16k, rms};
pub use chunk::{plan_cut, split};
pub use decode::{Emitted, Joint, tdt_greedy};
pub use dictation::{Dictation, MAX_RECORDING};
pub use engine::Parakeet;
pub use permission::Permission;
pub use text::{Vocabulary, detokenize};

/// Why a dictation didn't give text.
#[derive(Debug, thiserror::Error)]
pub enum VoiceError {
    /// There is no input device.
    #[error("there's no microphone")]
    NoMicrophone,
    /// The person, or a policy, denied the microphone.
    #[error("Kanso isn't allowed to use the microphone")]
    MicrophoneDenied,
    /// The input device failed or refused its settings.
    #[error("the microphone failed: {0}")]
    Device(String),
    /// The model is missing, damaged, or failed to run.
    #[error("the speech model failed: {0}")]
    Model(String),
    /// The dictation was cancelled.
    #[error("dictation was cancelled")]
    Cancelled,
    /// The recording held no words.
    #[error("no speech was heard")]
    NothingHeard,
    /// An audio file that can't be read: not audio, damaged, or in a
    /// format Kanso doesn't decode.
    #[error("the audio can't be read: {0}")]
    Audio(String),
}
