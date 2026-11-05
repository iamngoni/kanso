//! The data of telling speakers apart (issue #38, step 13): which two small
//! models it runs, and what it gives back.

use std::ops::Range;

/// One model file, pinned to a revision of a Hugging Face repository and
/// checked by size and SHA-256 like the speech model.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pinned {
    pub repository: &'static str,
    pub revision: &'static str,
    /// Where it goes, under the app's data folder.
    pub folder: &'static str,
    pub name: &'static str,
    pub size: u64,
    pub sha256: &'static str,
}

/// pyannote's segmentation 3.0 (MIT), exported to ONNX and quantized to
/// int8 by the sherpa-onnx project: ten seconds in, who is speaking in each
/// frame out, for up to three people at once.
pub const SEGMENTATION: Pinned = Pinned {
    repository: "csukuangfj/sherpa-onnx-pyannote-segmentation-3-0",
    revision: "9403a6902bb58e3d5ae8c7e77c3422de279db2e0",
    folder: "models/diarize-segmentation-pyannote-3-0",
    name: "model.int8.onnx",
    size: 1_540_506,
    sha256: "d582f4b4c6b48205de7e0643c57df0df5615a3c176189be3fc461e9d18827b5d",
};

/// WeSpeaker's ResNet34 speaker embedding (Apache-2.0, trained on
/// VoxCeleb), as ONNX: a stretch of one person's speech in, 256 numbers
/// that are close for the same voice.
pub const EMBEDDING: Pinned = Pinned {
    repository: "csukuangfj/speaker-embedding-models",
    revision: "0743f301363dec56491a490f6d6cbc9d67f9a3bf",
    folder: "models/diarize-embedding-wespeaker-resnet34",
    name: "wespeaker_en_voxceleb_resnet34_LM.onnx",
    size: 26_530_550,
    sha256: "e9848563da86f263117134dfd7ad63c92355b37de492b55e325400c9d9c39012",
};

/// Who made the models, for the settings row beside the download.
pub const ATTRIBUTION: &str = "Speaker segmentation: pyannote 3.0 (MIT), ONNX export by the sherpa-onnx project. Speaker embedding: WeSpeaker ResNet34 (Apache-2.0).";

/// How many bytes both models download.
pub const fn total_size() -> u64 {
    SEGMENTATION.size + EMBEDDING.size
}

/// A stretch of one person talking.
#[derive(Clone, Debug, PartialEq)]
pub struct Turn {
    /// Seconds from the start of the recording.
    pub start: f64,
    pub end: f64,
    /// Which voice, numbered from 0 in no particular order.
    pub speaker: usize,
}

impl Turn {
    /// How many seconds of `range` (start, end) this turn covers. Pure.
    pub fn overlap(&self, range: &Range<f64>) -> f64 {
        (self.end.min(range.end) - self.start.max(range.start)).max(0.)
    }
}

/// Who spoke when, over one recording.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Diarization {
    /// Turns in start order. Two voices overlap when they talk over each
    /// other.
    pub turns: Vec<Turn>,
    /// How long the recording is, in seconds.
    pub seconds: f64,
}

impl Diarization {
    /// How many different voices there are.
    pub fn speakers(&self) -> usize {
        self.turns
            .iter()
            .map(|t| t.speaker)
            .collect::<std::collections::BTreeSet<_>>()
            .len()
    }
}
