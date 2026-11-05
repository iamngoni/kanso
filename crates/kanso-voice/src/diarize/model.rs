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

/// WeSpeaker's ResNet34 speaker embedding, trained on VoxCeleb, as ONNX: a
/// stretch of one person's speech in, 256 numbers that are close for the
/// same voice. WeSpeaker's code is Apache-2.0, but its pretrained models
/// take their dataset's licence, and VoxCeleb's is CC BY 4.0 (WeSpeaker's
/// `docs/pretrained.md`, "Model License"), so the model is CC BY 4.0 and is
/// credited in [`ATTRIBUTION`] like the speech model.
pub const EMBEDDING: Pinned = Pinned {
    repository: "csukuangfj/speaker-embedding-models",
    revision: "0743f301363dec56491a490f6d6cbc9d67f9a3bf",
    folder: "models/diarize-embedding-wespeaker-resnet34",
    name: "wespeaker_en_voxceleb_resnet34_LM.onnx",
    size: 26_530_550,
    sha256: "e9848563da86f263117134dfd7ad63c92355b37de492b55e325400c9d9c39012",
};

/// Who made the models, for the settings row beside the download.
pub const ATTRIBUTION: &str = "Speaker segmentation: pyannote segmentation 3.0 by CNRS (MIT), ONNX export by the sherpa-onnx project. Speaker embedding: WeSpeaker ResNet34 trained on VoxCeleb (CC BY 4.0).";

/// The speaker embedding's licence.
pub const EMBEDDING_LICENSE_URL: &str = "https://creativecommons.org/licenses/by/4.0/";

/// Whether `bytes` are exactly `pinned`'s file: its size and SHA-256. A
/// cut-short download, a damaged file or a different model is not. Pure.
pub fn is_pinned(bytes: &[u8], pinned: &Pinned) -> bool {
    use sha2::{Digest as _, Sha256};
    if bytes.len() as u64 != pinned.size {
        return false;
    }
    let hash: String = Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    hash == pinned.sha256
}

/// The file of `pinned` in `dir`, read whole and checked before anything
/// loads it: the bytes that are checked are the bytes that run, so a file
/// swapped after the check is never the one loaded.
// no-prop-test: reads a file; `is_pinned` has the property.
pub fn read_pinned(dir: &std::path::Path, pinned: &Pinned) -> Result<Vec<u8>, crate::VoiceError> {
    let path = dir.join(pinned.name);
    let missing = || {
        crate::VoiceError::Model(format!(
            "the speaker model {} isn't the downloaded one. Download it again in Settings.",
            pinned.name
        ))
    };
    // Never read more than the pinned size (and one byte to tell it is longer).
    let file = std::fs::File::open(&path).map_err(|_| missing())?;
    let mut bytes = Vec::with_capacity(pinned.size as usize);
    std::io::Read::read_to_end(&mut std::io::Read::take(file, pinned.size + 1), &mut bytes)
        .map_err(|_| missing())?;
    if is_pinned(&bytes, pinned) {
        Ok(bytes)
    } else {
        Err(missing())
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::{FileFailurePersistence, RngSeed};
    use sha2::{Digest as _, Sha256};

    fn pinned_of(bytes: &[u8]) -> Pinned {
        let hash: String = Sha256::digest(bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        Pinned {
            repository: "r",
            revision: "v",
            folder: "f",
            name: "model.onnx",
            size: bytes.len() as u64,
            sha256: Box::leak(hash.into_boxed_str()),
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 64,
            rng_seed: RngSeed::Fixed(0x006b_616e_736f),
            failure_persistence: Some(Box::new(FileFailurePersistence::Off)),
            ..ProptestConfig::default()
        })]

        /// Only the exact file passes: cut short, longer, or one byte
        /// changed, it is refused.
        #[test]
        fn prop_only_the_pinned_bytes_pass(
            bytes in proptest::collection::vec(any::<u8>(), 1..256),
            at in any::<prop::sample::Index>(),
            flip in 1u8..=255,
        ) {
            let pinned = pinned_of(&bytes);
            prop_assert!(is_pinned(&bytes, &pinned));
            prop_assert!(!is_pinned(&bytes[..bytes.len() - 1], &pinned));
            let mut longer = bytes.clone();
            longer.push(0);
            prop_assert!(!is_pinned(&longer, &pinned));
            let mut changed = bytes.clone();
            let i = at.index(changed.len());
            changed[i] ^= flip;
            prop_assert!(!is_pinned(&changed, &pinned));
        }
    }

    #[test]
    fn a_partial_or_swapped_file_on_disk_is_never_read_as_the_model() {
        let dir = tempfile::tempdir().unwrap();
        let good = b"the real model bytes".to_vec();
        let pinned = pinned_of(&good);
        let path = dir.path().join(pinned.name);
        assert!(read_pinned(dir.path(), &pinned).is_err(), "missing");
        std::fs::write(&path, &good[..5]).unwrap();
        assert!(read_pinned(dir.path(), &pinned).is_err(), "partial");
        std::fs::write(&path, b"the fake model bytes").unwrap();
        assert!(
            read_pinned(dir.path(), &pinned).is_err(),
            "same size, swapped"
        );
        std::fs::write(&path, &good).unwrap();
        assert_eq!(read_pinned(dir.path(), &pinned).unwrap(), good);
    }

    #[test]
    fn the_pinned_models_are_the_published_ones() {
        assert_eq!(SEGMENTATION.revision.len(), 40);
        assert_eq!(EMBEDDING.revision.len(), 40);
        assert_eq!(total_size(), 28_071_056);
        assert!(ATTRIBUTION.contains("MIT") && ATTRIBUTION.contains("CC BY 4.0"));
    }
}
