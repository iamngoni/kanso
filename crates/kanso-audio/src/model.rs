//! The plain types and limits of a recording on disk.

use crate::error::AudioError;

/// Opus's native rate, and the rate recordings are written at.
pub const SAMPLE_RATE: u32 = 48_000;
/// Samples per channel in one 20 ms frame.
pub const FRAME_SAMPLES: usize = 960;
/// Frames in one second of audio.
pub const FRAMES_PER_SECOND: u64 = 50;
/// A part is closed before it passes this many bytes, leaving room under the
/// 10 MiB attachment cap.
pub const MAX_PART_BYTES: u64 = 9 * 1024 * 1024;
/// A part is closed after this many frames: 60 minutes.
pub const MAX_PART_FRAMES: u64 = 60 * 60 * FRAMES_PER_SECOND;
/// A page is ended every this many frames: one second.
pub const PAGE_FRAMES: u64 = FRAMES_PER_SECOND;
/// The file is synced to disk every this many frames: two seconds.
pub const SYNC_FRAMES: u64 = 2 * FRAMES_PER_SECOND;
/// How much audio a seek decodes and throws away before the target so the
/// decoder has settled: 80 ms, as RFC 7845 recommends.
pub const SEEK_PREROLL: u64 = 3_840;
/// The smallest part limit accepted; below it a page could not fit.
pub const MIN_PART_BYTES: u64 = 16 * 1024;

/// Mono, or two channels (left = you, right = the meeting).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Channels {
    Mono,
    Stereo,
}

impl Channels {
    pub fn count(self) -> usize {
        match self {
            Channels::Mono => 1,
            Channels::Stereo => 2,
        }
    }

    pub fn from_count(count: usize) -> Option<Self> {
        match count {
            1 => Some(Channels::Mono),
            2 => Some(Channels::Stereo),
            _ => None,
        }
    }

    /// The bit rate that sounds right for speech: 16 kbps mono, 24 kbps
    /// stereo.
    pub fn default_bitrate(self) -> i32 {
        match self {
            Channels::Mono => 16_000,
            Channels::Stereo => 24_000,
        }
    }
}

/// How a recording is encoded and cut into parts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EncodeConfig {
    pub channels: Channels,
    pub bitrate: i32,
    /// The Ogg stream serial of part 0; part `n` uses this plus `n`.
    pub serial: u32,
    pub max_part_bytes: u64,
    pub max_part_frames: u64,
}

impl EncodeConfig {
    pub fn new(channels: Channels) -> Self {
        EncodeConfig {
            channels,
            bitrate: channels.default_bitrate(),
            serial: 0x6b61_6e73,
            max_part_bytes: MAX_PART_BYTES,
            max_part_frames: MAX_PART_FRAMES,
        }
    }

    pub(crate) fn validate(&self) -> Result<(), AudioError> {
        if self.max_part_bytes < MIN_PART_BYTES {
            return Err(AudioError::Config(format!(
                "a part must be allowed at least {MIN_PART_BYTES} bytes"
            )));
        }
        if self.max_part_frames == 0 {
            return Err(AudioError::Config("a part must hold a frame".into()));
        }
        if !(500..=512_000).contains(&self.bitrate) {
            return Err(AudioError::Config("the bit rate is out of range".into()));
        }
        Ok(())
    }
}

/// A finished part.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PartInfo {
    pub index: u32,
    /// Size of the file.
    pub bytes: u64,
    /// Audio in it, in samples per channel at 48 kHz.
    pub samples: u64,
}

/// The part being written.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpenPart {
    pub index: u32,
    /// Bytes written to the file so far (whole pages).
    pub bytes: u64,
    /// Samples per channel taken into it so far.
    pub samples: u64,
}

/// The `OpusHead` identification header (RFC 7845 section 5.1), mapping
/// family 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpusHead {
    pub channels: u8,
    pub pre_skip: u16,
    pub input_rate: u32,
    pub gain: i16,
}

/// Where a page starts in a file, and where the audio stands there.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PageEntry {
    /// Byte offset of the page.
    pub offset: u64,
    /// Granule position (48 kHz samples, pre-skip included) at the start of
    /// the page's first packet.
    pub granule: u64,
}

/// What a file says about itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StreamInfo {
    pub channels: Channels,
    pub pre_skip: u16,
    /// Samples per channel after pre-skip and end trimming.
    pub total_samples: u64,
}

impl StreamInfo {
    pub fn duration_secs(&self) -> f64 {
        self.total_samples as f64 / f64::from(SAMPLE_RATE)
    }
}

/// Below this, the meter shows nothing: -60 dBFS.
const METER_FLOOR_DB: f32 = -60.0;

/// Maps an RMS to 0..=1 for a meter, on a decibel scale from -60 dBFS
/// (empty) to 0 dBFS (full), so quiet speech still moves it.
pub fn meter_level(rms: f32) -> f32 {
    if !rms.is_finite() || rms <= 0.0 {
        return 0.0;
    }
    let db = 20.0 * rms.log10();
    ((db - METER_FLOOR_DB) / -METER_FLOOR_DB).clamp(0.0, 1.0)
}
