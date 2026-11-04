//! The plain types of a recording in progress: the sidecar, the limits, what
//! a push or a stop returns, and what a crash leaves behind.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::model::{PartInfo, SAMPLE_RATE};

/// What a recording is made from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordSource {
    Microphone,
    SystemAudio,
}

/// Whether the recorder is still writing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionState {
    /// Written while recording; found like this after a crash.
    Recording,
    /// Finalised; the parts are whole and wait to be attached.
    Stopped,
}

/// A part as the sidecar lists it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionPart {
    pub index: u32,
    /// The file name in the session folder.
    pub file: String,
    pub bytes: u64,
    /// Samples per channel at 48 kHz.
    pub samples: u64,
    /// False for the part being written.
    pub closed: bool,
}

/// `session.json`: what a crash needs to know about a recording. Rewritten
/// atomically every 5 seconds of audio and whenever a part closes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionFile {
    pub version: u32,
    pub id: String,
    /// Wall-clock start, milliseconds since the Unix epoch.
    pub started_at_ms: u64,
    pub updated_at_ms: u64,
    pub sources: Vec<RecordSource>,
    pub channels: u8,
    /// The note the recording goes into, when it was started from one.
    pub note_id: Option<String>,
    pub state: SessionState,
    pub parts: Vec<SessionPart>,
    pub bytes: u64,
    /// Samples per channel recorded so far.
    pub samples: u64,
    /// The cap on the recording, in samples per channel.
    pub cap_samples: u64,
}

impl SessionFile {
    pub const VERSION: u32 = 1;

    pub fn duration_secs(&self) -> f64 {
        self.samples as f64 / f64::from(SAMPLE_RATE)
    }
}

/// How long a recording may run, and when to warn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    /// Samples per channel after which the recorder stops taking audio.
    pub cap_samples: u64,
    /// Samples per channel at which the warning is raised, once.
    pub warn_samples: u64,
}

impl Limits {
    /// 6 hours, warning at 5.
    pub const DEFAULT_CAP_SECS: u64 = 6 * 3_600;
    pub const DEFAULT_WARN_SECS: u64 = 5 * 3_600;

    pub fn from_secs(cap: u64, warn: u64) -> Self {
        let rate = u64::from(SAMPLE_RATE);
        Limits {
            cap_samples: cap * rate,
            warn_samples: warn.min(cap) * rate,
        }
    }
}

impl Default for Limits {
    fn default() -> Self {
        Limits::from_secs(Self::DEFAULT_CAP_SECS, Self::DEFAULT_WARN_SECS)
    }
}

/// What one push of audio led to.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Progress {
    /// Samples per channel recorded so far.
    pub samples: u64,
    /// The warning threshold was crossed by this push (raised once).
    pub warned: bool,
    /// The cap is reached: the recorder takes no more audio.
    pub capped: bool,
}

/// A recording that stopped: its folder and parts, in order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finished {
    pub dir: PathBuf,
    pub session: SessionFile,
    /// The part files with their sizes, in playing order.
    pub parts: Vec<(PathBuf, PartInfo)>,
}

/// A recording left behind by a crash or power cut.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Recoverable {
    pub id: String,
    pub dir: PathBuf,
    /// `None` when the sidecar is missing or unreadable; the parts are still
    /// recoverable.
    pub session: Option<SessionFile>,
    /// Whole audio found in the part files, as they are right now.
    pub parts: Vec<(PathBuf, PartInfo)>,
    /// Samples per channel across all parts.
    pub samples: u64,
}

impl Recoverable {
    pub fn duration_secs(&self) -> f64 {
        self.samples as f64 / f64::from(SAMPLE_RATE)
    }
}
