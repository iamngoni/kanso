//! Parakeet TDT on ONNX Runtime: audio at 16 kHz in, text out.

use std::path::Path;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

use ort::environment::Environment;
use ort::session::Session;
use ort::session::builder::GraphOptimizationLevel;
use ort::value::{Tensor, TensorRef};

use crate::VoiceError;
use crate::decode::{DURATIONS, Joint, tdt_greedy};
use crate::text::{Vocabulary, join_pieces};
use crate::{chunk, model};

/// The ONNX Runtime environment, made once per process with telemetry off.
/// ONNX Runtime allows one environment per process.
fn environment() -> Result<&'static Environment, VoiceError> {
    static ENVIRONMENT: OnceLock<Result<Environment, String>> = OnceLock::new();
    ENVIRONMENT
        .get_or_init(|| {
            ort::init()
                .with_name("kanso-voice")
                .with_telemetry(false)
                .build()
                .map_err(|e| e.to_string())
        })
        .as_ref()
        .map_err(|e| VoiceError::Model(format!("ONNX Runtime didn't start: {e}")))
}

/// Threads one model run uses: half the cores, between 1 and 4, leaving the
/// rest to the app.
pub fn intra_threads(cores: usize) -> usize {
    (cores / 2).clamp(1, 4)
}

// no-prop-test: ONNX Runtime setup; the model test runs it.
fn open(path: &Path) -> Result<Session, VoiceError> {
    let fail = |e: String| VoiceError::Model(format!("{}: {e}", path.display()));
    let cores = std::thread::available_parallelism().map_or(1, |n| n.get());
    Session::builder(environment()?)
        .map_err(|e| fail(e.to_string()))?
        .with_optimization_level(GraphOptimizationLevel::Level3)
        .map_err(|e| fail(e.to_string()))?
        .with_intra_threads(intra_threads(cores))
        .map_err(|e| fail(e.to_string()))?
        .commit_from_file(path)
        .map_err(|e| fail(e.to_string()))
}

/// [`open`] for a model already in memory (read and checked by the caller).
// no-prop-test: ONNX Runtime setup; the model test runs it.
pub(crate) fn open_bytes(name: &str, bytes: &[u8]) -> Result<Session, VoiceError> {
    let fail = |e: String| VoiceError::Model(format!("{name}: {e}"));
    let cores = std::thread::available_parallelism().map_or(1, |n| n.get());
    Session::builder(environment()?)
        .map_err(|e| fail(e.to_string()))?
        .with_optimization_level(GraphOptimizationLevel::Level3)
        .map_err(|e| fail(e.to_string()))?
        .with_intra_threads(intra_threads(cores))
        .map_err(|e| fail(e.to_string()))?
        .commit_from_memory(bytes)
        .map_err(|e| fail(e.to_string()))
}

/// Pieces shorter than this (100 ms) hold no words; they're skipped.
const MIN_SAMPLES: usize = 1_600;

/// The loaded model: its three networks and its tokens.
pub struct Parakeet {
    preprocessor: Session,
    encoder: Session,
    decoder: Session,
    vocabulary: Vocabulary,
    /// The prediction network's state shapes: `[layers, 1, hidden]`.
    state_shapes: [[i64; 3]; 2],
}

impl Parakeet {
    /// Loads the model from `dir`, which holds [`model::FILES`].
    // no-prop-test: loads real files; the model test runs it.
    pub fn load(dir: &Path) -> Result<Parakeet, VoiceError> {
        let vocab_text = std::fs::read_to_string(dir.join(model::VOCABULARY))
            .map_err(|e| VoiceError::Model(format!("{}: {e}", model::VOCABULARY)))?;
        let vocabulary = Vocabulary::parse(&vocab_text).map_err(VoiceError::Model)?;
        let preprocessor = open(&dir.join(model::PREPROCESSOR))?;
        let encoder = open(&dir.join(model::ENCODER))?;
        let decoder = open(&dir.join(model::DECODER_JOINT))?;

        let state_shape = |name: &str| -> Result<[i64; 3], VoiceError> {
            let input = decoder
                .inputs()
                .iter()
                .find(|input| input.name() == name)
                .ok_or_else(|| VoiceError::Model(format!("the decoder has no input {name}")))?;
            match input
                .dtype()
                .tensor_shape()
                .map(|shape| shape.to_vec())
                .as_deref()
            {
                Some(&[layers, _, hidden]) if layers > 0 && hidden > 0 => Ok([layers, 1, hidden]),
                other => Err(VoiceError::Model(format!(
                    "the decoder's {name} has shape {other:?}"
                ))),
            }
        };
        let state_shapes = [
            state_shape("input_states_1")?,
            state_shape("input_states_2")?,
        ];
        Ok(Parakeet {
            preprocessor,
            encoder,
            decoder,
            vocabulary,
            state_shapes,
        })
    }

    /// Transcribes a recording of any length: cuts it at pauses
    /// ([`chunk::split`]) and joins the pieces' texts with spaces.
    // no-prop-test: runs the real model; `split` and `join_pieces` have the properties.
    pub fn transcribe_all(
        &mut self,
        samples: &[f32],
        cancel: &AtomicBool,
    ) -> Result<String, VoiceError> {
        let mut texts = Vec::new();
        for range in chunk::split(samples, crate::audio::MODEL_RATE) {
            texts.push(self.transcribe(&samples[range], cancel)?);
        }
        Ok(join_pieces(texts.iter().map(String::as_str)))
    }

    /// Transcribes one piece of 16 kHz mono audio (at most about 25 s;
    /// longer pieces work but slow down with the square of their length).
    /// Checks `cancel` between steps.
    // no-prop-test: runs the real model; the model test runs it.
    pub fn transcribe(
        &mut self,
        samples: &[f32],
        cancel: &AtomicBool,
    ) -> Result<String, VoiceError> {
        if samples.len() < MIN_SAMPLES {
            return Ok(String::new());
        }
        let err =
            |what: &'static str| move |e: ort::Error| VoiceError::Model(format!("{what}: {e}"));

        let waveforms =
            TensorRef::from_array_view(([1usize, samples.len()], samples)).map_err(err("audio"))?;
        let lengths = Tensor::from_array(([1usize], vec![samples.len() as i64]))
            .map_err(err("audio length"))?;
        let features = self
            .preprocessor
            .run(ort::inputs!["waveforms" => waveforms, "waveforms_lens" => lengths])
            .map_err(err("preprocessor"))?;
        if cancel.load(Ordering::Relaxed) {
            return Err(VoiceError::Cancelled);
        }

        let encoded = self
            .encoder
            .run(ort::inputs![
                "audio_signal" => &features["features"],
                "length" => &features["features_lens"],
            ])
            .map_err(err("encoder"))?;
        let (shape, data) = encoded["outputs"]
            .try_extract_tensor::<f32>()
            .map_err(err("encoder output"))?;
        let (_, lengths) = encoded["encoded_lengths"]
            .try_extract_tensor::<i64>()
            .map_err(err("encoder lengths"))?;
        // `outputs` is [1, dims, frames]: one column per frame.
        let &[1, dims, total] = &shape[..] else {
            return Err(VoiceError::Model(format!(
                "the encoder gave shape {shape:?}"
            )));
        };
        let (dims, total) = (dims as usize, total as usize);
        let frames = lengths.first().map_or(0, |&n| n.max(0) as usize).min(total);

        let vocab_size = self.vocabulary.len();
        let blank = self.vocabulary.blank();
        let mut joint = OrtJoint {
            decoder: &mut self.decoder,
            encoded: data,
            dims,
            total,
            column: vec![0.0; dims],
            state_shapes: self.state_shapes,
        };
        let emitted = tdt_greedy(&mut joint, frames, vocab_size, blank, &DURATIONS, cancel)?;
        Ok(self.vocabulary.decode(emitted.iter().map(|e| e.token)))
    }
}

/// The decoder-joint network over one piece's encoder output.
struct OrtJoint<'a> {
    decoder: &'a mut Session,
    /// The encoder output, `[dims, total]` in row-major order.
    encoded: &'a [f32],
    dims: usize,
    total: usize,
    column: Vec<f32>,
    state_shapes: [[i64; 3]; 2],
}

impl Joint for OrtJoint<'_> {
    type State = (Vec<f32>, Vec<f32>);

    fn initial_state(&self) -> Self::State {
        let size = |s: [i64; 3]| s.iter().product::<i64>() as usize;
        (
            vec![0.0; size(self.state_shapes[0])],
            vec![0.0; size(self.state_shapes[1])],
        )
    }

    // no-prop-test: runs the real decoder; `tdt_greedy` has the properties.
    fn step(
        &mut self,
        frame: usize,
        previous: u32,
        state: &Self::State,
        logits: &mut Vec<f32>,
    ) -> Result<Self::State, VoiceError> {
        let err = |what: &'static str| {
            move |e: ort::Error| VoiceError::Model(format!("decoder {what}: {e}"))
        };
        for (d, value) in self.column.iter_mut().enumerate() {
            *value = self.encoded[d * self.total + frame];
        }
        let encoder_outputs =
            TensorRef::from_array_view(([1usize, self.dims, 1], &self.column[..]))
                .map_err(err("frame"))?;
        let targets =
            Tensor::from_array(([1usize, 1], vec![previous as i32])).map_err(err("target"))?;
        let target_length =
            Tensor::from_array(([1usize], vec![1i32])).map_err(err("target length"))?;
        let states_1 = TensorRef::from_array_view((self.state_shapes[0], &state.0[..]))
            .map_err(err("state"))?;
        let states_2 = TensorRef::from_array_view((self.state_shapes[1], &state.1[..]))
            .map_err(err("state"))?;
        let outputs = self
            .decoder
            .run(ort::inputs![
                "encoder_outputs" => encoder_outputs,
                "targets" => targets,
                "target_length" => target_length,
                "input_states_1" => states_1,
                "input_states_2" => states_2,
            ])
            .map_err(err("run"))?;
        let (_, scores) = outputs["outputs"]
            .try_extract_tensor::<f32>()
            .map_err(err("output"))?;
        logits.extend_from_slice(scores);
        let (_, next_1) = outputs["output_states_1"]
            .try_extract_tensor::<f32>()
            .map_err(err("state"))?;
        let (_, next_2) = outputs["output_states_2"]
            .try_extract_tensor::<f32>()
            .map_err(err("state"))?;
        Ok((next_1.to_vec(), next_2.to_vec()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn inference_uses_half_the_cores_up_to_four(cores in 0usize..512) {
            let n = intra_threads(cores);
            prop_assert!((1..=4).contains(&n));
            prop_assert!(n <= cores.max(1));
            if (2..=8).contains(&cores) {
                prop_assert_eq!(n, cores / 2);
            }
        }
    }
}
