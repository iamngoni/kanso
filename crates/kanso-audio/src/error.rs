//! What can go wrong writing or reading a recording.

/// Why a recording could not be written or read.
#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("the Opus codec failed: {0}")]
    Codec(String),
    #[error("it isn't a readable Ogg Opus file: {0}")]
    Format(String),
    #[error("{0}")]
    Decode(String),
    #[error("there's no sound in it")]
    NoSound,
    #[error("it doesn't say its sample rate")]
    NoRate,
    #[error("{0}")]
    Config(String),
    #[error("Kanso isn't allowed to use the microphone")]
    MicrophoneDenied,
    #[error("there's no microphone")]
    NoMicrophone,
    #[error("the microphone failed: {0}")]
    Device(String),
    #[error("system audio isn't available: {0}")]
    SystemAudio(String),
    #[error("Kanso isn't allowed to record other apps' audio")]
    ScreenRecordingDenied,
    #[error("the recording's notes file is unreadable: {0}")]
    Session(String),
}

impl From<opus::Error> for AudioError {
    fn from(e: opus::Error) -> Self {
        AudioError::Codec(e.to_string())
    }
}

impl From<ogg::OggReadError> for AudioError {
    fn from(e: ogg::OggReadError) -> Self {
        AudioError::Format(e.to_string())
    }
}

impl From<symphonia::core::errors::Error> for AudioError {
    fn from(e: symphonia::core::errors::Error) -> Self {
        AudioError::Decode(e.to_string())
    }
}
