//! One channel at the microphone's rate made 48 kHz as it arrives.

use rubato::audioadapter_buffers::direct::InterleavedSlice;
use rubato::{Fft, FixedSync, Resampler as _};

use crate::error::AudioError;
use crate::model::SAMPLE_RATE;

/// Frames the FFT resampler takes per call.
const CHUNK: usize = 1024;

/// The rates a sound card records at that Kanso resamples from.
pub const SUPPORTED_RATES: std::ops::RangeInclusive<u32> = 4_000..=384_000;

/// Resamples a mono stream from `rate` Hz to 48 kHz.
///
/// Feed it with [`push`](Self::push) and end with [`finish`](Self::finish):
/// altogether it gives `round(n × 48000 / rate)` samples for `n` in, with the
/// resampler's delay taken off the front, so a timestamp in the recording is
/// a timestamp on the clock.
pub struct To48k {
    inner: Option<Fft<f32>>,
    rate: u32,
    pending: Vec<f32>,
    scratch: Vec<f32>,
    to_trim: usize,
    frames_in: u64,
    frames_out: u64,
}

impl To48k {
    pub fn new(rate: u32) -> Result<Self, AudioError> {
        if !SUPPORTED_RATES.contains(&rate) {
            return Err(AudioError::Device(format!("can't resample {rate} Hz")));
        }
        let inner = if rate == SAMPLE_RATE {
            None
        } else {
            let fft = Fft::<f32>::new(
                rate as usize,
                SAMPLE_RATE as usize,
                CHUNK,
                1,
                FixedSync::Input,
            )
            .map_err(|e| AudioError::Device(format!("can't resample {rate} Hz: {e}")))?;
            Some(fft)
        };
        let to_trim = inner.as_ref().map_or(0, |fft| fft.output_delay());
        let scratch = vec![0.0; inner.as_ref().map_or(0, |fft| fft.output_frames_max())];
        Ok(To48k {
            inner,
            rate,
            pending: Vec::new(),
            scratch,
            to_trim,
            frames_in: 0,
            frames_out: 0,
        })
    }

    /// Resamples `input`, appending what's ready to `out`.
    pub fn push(&mut self, input: &[f32], out: &mut Vec<f32>) {
        self.frames_in += input.len() as u64;
        let Some(fft) = self.inner.as_mut() else {
            out.extend_from_slice(input);
            self.frames_out += input.len() as u64;
            return;
        };
        self.pending.extend_from_slice(input);
        let mut start = 0;
        while self.pending.len() - start >= fft.input_frames_next() {
            let needed = fft.input_frames_next();
            start += run(
                fft,
                &self.pending[start..start + needed],
                None,
                &mut self.scratch,
                &mut self.to_trim,
                &mut self.frames_out,
                out,
            );
        }
        self.pending.drain(..start);
    }

    /// Resamples what's left, padding with silence, so the total is exactly
    /// the expected length.
    pub fn finish(mut self, out: &mut Vec<f32>) {
        let expected = self.expected_len();
        if let Some(fft) = self.inner.as_mut() {
            let mut left = std::mem::take(&mut self.pending);
            let needed = fft.input_frames_next();
            let mut partial = left.len().min(needed);
            left.resize(needed, 0.0);
            while self.frames_out < expected {
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
            }
        }
        let extra = self.frames_out.saturating_sub(expected) as usize;
        out.truncate(out.len() - extra.min(out.len()));
    }

    /// How many 48 kHz samples all the input so far makes.
    pub fn expected_len(&self) -> u64 {
        let rate = u64::from(self.rate);
        (self.frames_in * u64::from(SAMPLE_RATE) + rate / 2) / rate
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
    let frames = input.len();
    let buffer_in = InterleavedSlice::new(input, 1, frames).expect("one channel fits");
    let out_len = scratch.len();
    let mut buffer_out = InterleavedSlice::new_mut(scratch, 1, out_len).expect("one channel fits");
    let indexing = partial.map(|len| rubato::Indexing::new().partial_len(len));
    let (consumed, produced) = fft
        .process_into_buffer(&buffer_in, &mut buffer_out, indexing.as_ref())
        .expect("buffers are sized from the resampler");
    let skip = (*to_trim).min(produced);
    *to_trim -= skip;
    out.extend_from_slice(&scratch[skip..produced]);
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

        /// However the input is cut up, n samples in make round(n × 48000 /
        /// rate) out.
        #[test]
        fn prop_length_is_exact_whatever_the_chunking(
            rate in prop::sample::select(vec![8_000u32, 16_000, 24_000, 44_100, 48_000, 96_000]),
            total in 0usize..30_000,
            chunk in 1usize..5_000,
        ) {
            let input = vec![0.1f32; total];
            let mut r = To48k::new(rate).unwrap();
            let mut out = Vec::new();
            for piece in input.chunks(chunk) {
                r.push(piece, &mut out);
            }
            let expected = r.expected_len();
            r.finish(&mut out);
            prop_assert_eq!(out.len() as u64, expected);
            prop_assert_eq!(
                expected,
                (total as u64 * 48_000 + u64::from(rate) / 2) / u64::from(rate)
            );
        }
    }

    #[test]
    fn rates_nobody_records_at_are_refused() {
        assert!(To48k::new(0).is_err());
        assert!(To48k::new(3_999).is_err());
        assert!(To48k::new(384_001).is_err());
    }
}
