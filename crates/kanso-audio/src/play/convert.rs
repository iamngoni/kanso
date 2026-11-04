//! Samples of any channel count and rate made interleaved stereo at the
//! sound card's rate.

use rubato::audioadapter_buffers::direct::InterleavedSlice;
use rubato::{Fft, FixedSync, Resampler as _};

use crate::error::AudioError;
use crate::resample::SUPPORTED_RATES;

/// Frames the FFT resampler takes per call.
const CHUNK: usize = 1024;

/// Interleaved samples of `channels` as interleaved stereo: mono is copied
/// to both sides, stereo kept, anything wider keeps its first two channels.
/// Appends to `out`. Pure.
pub fn to_stereo(samples: &[f32], channels: usize, out: &mut Vec<f32>) {
    match channels {
        0 => {}
        1 => out.extend(samples.iter().flat_map(|s| [*s, *s])),
        2 => out.extend_from_slice(samples),
        n => {
            for frame in samples.chunks_exact(n) {
                out.extend_from_slice(&frame[..2]);
            }
        }
    }
}

/// Resamples interleaved stereo from one rate to another as it arrives.
///
/// `n` frames in give `round(n × to / from)` frames out in total, with the
/// resampler's delay taken off the front, so a position counted in output
/// frames is a position on the input's clock.
pub struct Resample {
    inner: Option<Fft<f32>>,
    from: u32,
    to: u32,
    pending: Vec<f32>,
    scratch: Vec<f32>,
    to_trim: usize,
    frames_in: u64,
    frames_out: u64,
}

impl Resample {
    pub fn new(from: u32, to: u32) -> Result<Self, AudioError> {
        if !SUPPORTED_RATES.contains(&from) || !SUPPORTED_RATES.contains(&to) {
            return Err(AudioError::Device(format!("can't resample {from} Hz to {to} Hz")));
        }
        let inner = if from == to {
            None
        } else {
            let fft = Fft::<f32>::new(from as usize, to as usize, CHUNK, 2, FixedSync::Input)
                .map_err(|e| AudioError::Device(format!("can't resample {from} Hz: {e}")))?;
            Some(fft)
        };
        let to_trim = inner.as_ref().map_or(0, |fft| fft.output_delay());
        let scratch = vec![0.0; inner.as_ref().map_or(0, |fft| fft.output_frames_max() * 2)];
        Ok(Resample {
            inner,
            from,
            to,
            pending: Vec::new(),
            scratch,
            to_trim,
            frames_in: 0,
            frames_out: 0,
        })
    }

    pub fn from_rate(&self) -> u32 {
        self.from
    }

    /// Resamples `input` (interleaved stereo), appending what is ready.
    pub fn push(&mut self, input: &[f32], out: &mut Vec<f32>) {
        self.frames_in += (input.len() / 2) as u64;
        let Some(fft) = self.inner.as_mut() else {
            out.extend_from_slice(input);
            self.frames_out += (input.len() / 2) as u64;
            return;
        };
        self.pending.extend_from_slice(input);
        let mut start = 0;
        while (self.pending.len() - start) / 2 >= fft.input_frames_next() {
            let needed = fft.input_frames_next();
            start += run(
                fft,
                &self.pending[start..start + needed * 2],
                None,
                &mut self.scratch,
                &mut self.to_trim,
                &mut self.frames_out,
                out,
            ) * 2;
        }
        self.pending.drain(..start);
    }

    /// Resamples what's left, padding with silence, so the total is exactly
    /// the expected length.
    pub fn finish(&mut self, out: &mut Vec<f32>) {
        let expected = self.expected_len();
        if let Some(fft) = self.inner.as_mut() {
            let mut left = std::mem::take(&mut self.pending);
            let needed = fft.input_frames_next();
            let mut partial = (left.len() / 2).min(needed);
            left.resize(needed * 2, 0.0);
            let mut guard = 0;
            while self.frames_out < expected && guard < 64 {
                run(
                    fft,
                    &left,
                    Some(partial),
                    &mut self.scratch,
                    &mut self.to_trim,
                    &mut self.frames_out,
                    out,
                );
                partial = 0;
                guard += 1;
            }
        }
        let extra = self.frames_out.saturating_sub(expected) as usize * 2;
        out.truncate(out.len() - extra.min(out.len()));
        self.frames_out = self.frames_out.min(expected);
    }

    /// How many output frames all the input so far makes.
    pub fn expected_len(&self) -> u64 {
        let from = u64::from(self.from);
        (self.frames_in * u64::from(self.to) + from / 2) / from
    }
}

fn run(
    fft: &mut Fft<f32>,
    input: &[f32],
    partial: Option<usize>,
    scratch: &mut [f32],
    to_trim: &mut usize,
    frames_out: &mut u64,
    out: &mut Vec<f32>,
) -> usize {
    let frames = input.len() / 2;
    let buffer_in = InterleavedSlice::new(input, 2, frames).expect("two channels fit");
    let out_frames = scratch.len() / 2;
    let mut buffer_out =
        InterleavedSlice::new_mut(scratch, 2, out_frames).expect("two channels fit");
    let indexing = partial.map(|len| rubato::Indexing::new().partial_len(len));
    let (consumed, produced) = fft
        .process_into_buffer(&buffer_in, &mut buffer_out, indexing.as_ref())
        .expect("buffers are sized from the resampler");
    let skip = (*to_trim).min(produced);
    *to_trim -= skip;
    out.extend_from_slice(&scratch[skip * 2..produced * 2]);
    *frames_out += (produced - skip) as u64;
    consumed
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::{FileFailurePersistence, RngSeed};

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 24,
            rng_seed: RngSeed::Fixed(0x006b_616e_736f),
            failure_persistence: Some(Box::new(FileFailurePersistence::Off)),
            ..ProptestConfig::default()
        })]

        /// However the input is cut up, n frames in make round(n × to / from)
        /// out.
        #[test]
        fn prop_length_is_exact_whatever_the_chunking(
            from in prop::sample::select(vec![8_000u32, 22_050, 44_100, 48_000, 96_000]),
            to in prop::sample::select(vec![44_100u32, 48_000, 96_000]),
            total in 0usize..20_000,
            chunk in 1usize..4_000,
        ) {
            let input = vec![0.1f32; total * 2];
            let mut r = Resample::new(from, to).unwrap();
            let mut out = Vec::new();
            for piece in input.chunks(chunk * 2) {
                r.push(piece, &mut out);
            }
            let expected = r.expected_len();
            r.finish(&mut out);
            prop_assert_eq!(out.len() as u64, expected * 2);
            prop_assert_eq!(
                expected,
                (total as u64 * u64::from(to) + u64::from(from) / 2) / u64::from(from)
            );
        }

        /// Whatever the channel count, the result is whole stereo frames,
        /// one per input frame.
        #[test]
        fn prop_stereo_has_a_frame_per_frame(channels in 1usize..7, frames in 0usize..50) {
            let input: Vec<f32> = (0..frames * channels).map(|i| i as f32).collect();
            let mut out = Vec::new();
            to_stereo(&input, channels, &mut out);
            prop_assert_eq!(out.len(), frames * 2);
        }
    }

    #[test]
    fn mono_is_copied_to_both_sides() {
        let mut out = Vec::new();
        to_stereo(&[0.5, -0.25], 1, &mut out);
        assert_eq!(out, vec![0.5, 0.5, -0.25, -0.25]);
    }

    #[test]
    fn rates_nobody_plays_at_are_refused() {
        assert!(Resample::new(0, 48_000).is_err());
        assert!(Resample::new(48_000, 1).is_err());
    }
}
