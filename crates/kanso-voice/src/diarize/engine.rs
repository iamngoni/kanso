//! The two models on ONNX Runtime, fed a recording as it is decoded.
//!
//! Every five seconds the segmentation model looks at the last ten and says
//! who is speaking where, for up to three people. For each of those people
//! the embedding model hears the stretch they spoke alone and gives that
//! stretch a voiceprint. When the recording ends, the voiceprints are
//! clustered into people ([`super::cluster`]) and every window votes
//! ([`super::timeline`]). Only the voiceprints and the answers are kept, so
//! a three-hour meeting costs a few megabytes.

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use ort::session::Session;
use ort::value::Tensor;

use super::cluster::{THRESHOLD, cluster_weighted};
use super::fbank::{BANDS, features};
use super::model::{Diarization, EMBEDDING, SEGMENTATION};
use super::powerset::{CLASSES, SPEAKERS, decode};
use super::timeline::{FRAME_SAMPLES, FRAME_SECONDS, Window, turns};
use crate::VoiceError;
use crate::engine::open;

/// The segmentation model's window: ten seconds.
const WINDOW: usize = 160_000;
/// How far the window moves: five seconds.
const SHIFT: usize = 80_000;
/// Frames the model answers for in one window.
const WINDOW_FRAMES: usize = 589;
/// A person must have spoken alone for this many frames (0.5 s) in a window
/// for their voice to be taken.
const MIN_ALONE_FRAMES: usize = 30;

/// How many seconds of speech a voice needs altogether to count as a person
/// on its own. Less than this is crosstalk or a misheard stretch, and joins
/// the nearest voice.
pub const MIN_VOICE_SECONDS: f32 = 3.;

/// Frames in which person `label` speaks and nobody else does. Pure.
pub fn alone(active: &[[bool; SPEAKERS]], label: usize) -> Vec<usize> {
    active
        .iter()
        .enumerate()
        .filter(|(_, on)| on[label] && on.iter().filter(|&&b| b).count() == 1)
        .map(|(frame, _)| frame)
        .collect()
}

/// The segmentation and embedding models, loaded, and a recording being
/// fed through them.
pub struct Diarizer {
    segmentation: Session,
    embedding: Session,
    /// Samples not yet dropped, from sample `buffer_start` on.
    buffer: Vec<f32>,
    buffer_start: usize,
    /// How many samples there have been.
    total: usize,
    /// Where the last window ended.
    covered: usize,
    windows: Vec<Window>,
    /// A voiceprint, the window and label it belongs to, and how many
    /// frames of speech it was taken from.
    voiceprints: Vec<(Vec<f32>, usize, usize, usize)>,
}

impl Diarizer {
    /// Loads the models from the folders `segmentation` and `embedding`
    /// under the app's data folder, which hold [`SEGMENTATION`] and
    /// [`EMBEDDING`].
    // no-prop-test: loads real files; the ignored model test runs it.
    pub fn load(segmentation: &Path, embedding: &Path) -> Result<Diarizer, VoiceError> {
        Ok(Diarizer {
            segmentation: open(&segmentation.join(SEGMENTATION.name))?,
            embedding: open(&embedding.join(EMBEDDING.name))?,
            buffer: Vec::new(),
            buffer_start: 0,
            total: 0,
            covered: 0,
            windows: Vec::new(),
            voiceprints: Vec::new(),
        })
    }

    /// Takes the next stretch of the recording (16 kHz mono), looking at
    /// each ten seconds as they complete. Checks `cancel` between windows.
    // no-prop-test: runs the real models; `alone`, `decode` and `turns` have the properties.
    pub fn push(&mut self, samples: &[f32], cancel: &AtomicBool) -> Result<(), VoiceError> {
        self.buffer.extend_from_slice(samples);
        self.total += samples.len();
        while self.buffer.len() >= WINDOW {
            if cancel.load(Ordering::Relaxed) {
                return Err(VoiceError::Cancelled);
            }
            let window: Vec<f32> = self.buffer[..WINDOW].to_vec();
            self.look(&window, WINDOW)?;
            self.covered = self.buffer_start + WINDOW;
            self.buffer.drain(..SHIFT);
            self.buffer_start += SHIFT;
        }
        Ok(())
    }

    /// Looks at what is left, groups the voices and says who spoke when.
    // no-prop-test: runs the real models; `cluster` and `turns` have the properties.
    pub fn finish(mut self, cancel: &AtomicBool) -> Result<Diarization, VoiceError> {
        if self.total > self.covered {
            if cancel.load(Ordering::Relaxed) {
                return Err(VoiceError::Cancelled);
            }
            let valid = self.buffer.len().min(WINDOW);
            let mut window = self.buffer[..valid].to_vec();
            window.resize(WINDOW, 0.);
            self.look(&window, valid)?;
        }
        let seconds = self.total as f64 / 16_000.;
        let prints: Vec<Vec<f32>> = self
            .voiceprints
            .iter()
            .map(|(v, _, _, _)| v.clone())
            .collect();
        let spoken: Vec<f32> = self
            .voiceprints
            .iter()
            .map(|(_, _, _, frames)| *frames as f32 * FRAME_SECONDS as f32)
            .collect();
        let people = cluster_weighted(&prints, &spoken, THRESHOLD, MIN_VOICE_SECONDS);
        let count = people.iter().max().map_or(0, |m| m + 1);
        for ((_, window, label, _), person) in self.voiceprints.iter().zip(&people) {
            self.windows[*window].person[*label] = Some(*person);
        }
        let frames = self.total.div_ceil(FRAME_SAMPLES);
        Ok(Diarization {
            turns: turns(&self.windows, count, frames),
            seconds,
        })
    }

    /// One window: who speaks where, and the voice of each person who
    /// spoke alone for long enough. `valid` samples are real; the rest is
    /// padding.
    fn look(&mut self, window: &[f32], valid: usize) -> Result<(), VoiceError> {
        let err = |what: &'static str| {
            move |e: ort::Error| VoiceError::Model(format!("speaker {what}: {e}"))
        };
        let input =
            Tensor::from_array(([1usize, 1, WINDOW], window.to_vec())).map_err(err("window"))?;
        let outputs = self
            .segmentation
            .run(ort::inputs![input])
            .map_err(err("segmentation"))?;
        let (shape, scores) = outputs[0]
            .try_extract_tensor::<f32>()
            .map_err(err("segmentation output"))?;
        let &[1, frames, classes] = &shape[..] else {
            return Err(VoiceError::Model(format!(
                "the segmentation model gave shape {shape:?}"
            )));
        };
        if classes as usize != CLASSES || frames as usize != WINDOW_FRAMES {
            return Err(VoiceError::Model(format!(
                "the segmentation model gave shape {shape:?}"
            )));
        }
        let mut active = decode(scores);
        // Padding is not speech.
        for (frame, on) in active.iter_mut().enumerate() {
            if frame * FRAME_SAMPLES >= valid {
                *on = [false; SPEAKERS];
            }
        }
        drop(outputs);
        let start = self.buffer_start;
        let index = self.windows.len();
        for label in 0..SPEAKERS {
            let frames = alone(&active, label);
            if frames.len() < MIN_ALONE_FRAMES {
                continue;
            }
            let spoken = frames.len();
            let mut heard = Vec::with_capacity(frames.len() * FRAME_SAMPLES);
            for frame in frames {
                let from = frame * FRAME_SAMPLES;
                let to = (from + FRAME_SAMPLES).min(valid);
                if from < to {
                    heard.extend_from_slice(&window[from..to]);
                }
            }
            let bands = features(&heard);
            if bands.is_empty() {
                continue;
            }
            let steps = bands.len() / BANDS;
            let feats =
                Tensor::from_array(([1usize, steps, BANDS], bands)).map_err(err("features"))?;
            let outputs = self
                .embedding
                .run(ort::inputs![feats])
                .map_err(err("embedding"))?;
            let (_, print) = outputs[0]
                .try_extract_tensor::<f32>()
                .map_err(err("embedding output"))?;
            self.voiceprints
                .push((print.to_vec(), index, label, spoken));
        }
        self.windows.push(Window {
            first_frame: (start as f64 / FRAME_SAMPLES as f64).round() as usize,
            active,
            person: [None; SPEAKERS],
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::{FileFailurePersistence, RngSeed};

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 64,
            rng_seed: RngSeed::Fixed(0x006b_616e_736f),
            failure_persistence: Some(Box::new(FileFailurePersistence::Off)),
            ..ProptestConfig::default()
        })]

        /// A frame is "alone" for a person only when they are the only one
        /// on, and every such frame is found exactly once, in order.
        #[test]
        fn prop_alone_means_only_them(
            active in proptest::collection::vec(any::<[bool; 3]>(), 0..80),
            label in 0usize..3,
        ) {
            let found = alone(&active, label);
            for pair in found.windows(2) {
                prop_assert!(pair[0] < pair[1]);
            }
            for (frame, on) in active.iter().enumerate() {
                let only = on[label] && on.iter().filter(|&&b| b).count() == 1;
                prop_assert_eq!(found.contains(&frame), only);
            }
        }
    }
}
