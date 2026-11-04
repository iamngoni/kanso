//! Sound from the microphone made ready for the model: one channel, 16 kHz,
//! and a loudness for the meter.

use rubato::audioadapter_buffers::direct::InterleavedSlice;
use rubato::{Fft, FixedSync, Resampler as _};

/// The rate the model hears at.
pub const MODEL_RATE: u32 = 16_000;

/// Calls `sink` with the mean of each whole frame of `interleaved`, each
/// sample converted by `to_f32`. Allocates nothing, so the microphone's
/// callback can use it; an incomplete last frame is dropped.
pub fn for_each_mono<T: Copy>(
    interleaved: &[T],
    channels: usize,
    to_f32: impl Fn(T) -> f32,
    mut sink: impl FnMut(f32),
) {
    let channels = channels.max(1);
    let scale = 1.0 / channels as f32;
    for frame in interleaved.chunks_exact(channels) {
        let sum: f32 = frame.iter().map(|&sample| to_f32(sample)).sum();
        sink(sum * scale);
    }
}

/// Mixes interleaved samples down to one channel by averaging each frame.
pub fn downmix(interleaved: &[f32], channels: usize) -> Vec<f32> {
    let mut mono = Vec::with_capacity(interleaved.len() / channels.max(1));
    for_each_mono(interleaved, channels, |s| s, |s| mono.push(s));
    mono
}

/// The root mean square of `samples`; 0 for none.
pub fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum: f64 = samples.iter().map(|&s| f64::from(s) * f64::from(s)).sum();
    (sum / samples.len() as f64).sqrt() as f32
}

/// Maps an RMS to 0..=1 for a meter (the curve lives in `kanso-audio`,
/// which the phones use without the speech model).
pub use kanso_audio::meter_level;

/// Frames the FFT resampler takes per call.
const CHUNK: usize = 1024;

/// Turns one channel at the microphone's rate into 16 kHz as it arrives.
///
/// Feed it with [`push`](Self::push) and call [`finish`](Self::finish) at
/// the end: altogether it gives `round(n × 16000 / rate)` samples for `n`
/// in, with the resampler's delay taken off the front.
pub struct Resampler16k {
    inner: Option<Fft<f32>>,
    rate: u32,
    pending: Vec<f32>,
    scratch: Vec<f32>,
    /// Output frames still to drop: the resampler's delay.
    to_trim: usize,
    frames_in: u64,
    frames_out: u64,
}

/// The rates audio is recorded at that Kanso resamples from.
pub const SUPPORTED_RATES: std::ops::RangeInclusive<u32> = 4_000..=384_000;

impl Resampler16k {
    /// A resampler from `rate` Hz (more than 0) to 16 kHz.
    pub fn new(rate: u32) -> Result<Self, String> {
        if rate == 0 {
            return Err("the microphone reports a rate of 0 Hz".into());
        }
        // A file can say any rate; one no sound card records at would only
        // make the resampler plan something enormous.
        if !SUPPORTED_RATES.contains(&rate) {
            return Err(format!("can't resample {rate} Hz"));
        }
        let inner = if rate == MODEL_RATE {
            None
        } else {
            let fft = Fft::<f32>::new(
                rate as usize,
                MODEL_RATE as usize,
                CHUNK,
                1,
                FixedSync::Input,
            )
            .map_err(|e| format!("can't resample {rate} Hz: {e}"))?;
            Some(fft)
        };
        let to_trim = inner.as_ref().map_or(0, |fft| fft.output_delay());
        let scratch = vec![0.0; inner.as_ref().map_or(0, |fft| fft.output_frames_max())];
        Ok(Self {
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
            let consumed = run(
                fft,
                &self.pending[start..start + needed],
                None,
                &mut self.scratch,
                &mut self.to_trim,
                &mut self.frames_out,
                out,
            );
            start += consumed;
        }
        self.pending.drain(..start);
    }

    /// Resamples what's left, padding with silence, and appends it to `out`
    /// so the total is exactly the expected length.
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
        // The last call may overshoot; drop what's beyond the expected length.
        let extra = self.frames_out.saturating_sub(expected) as usize;
        out.truncate(out.len() - extra.min(out.len()));
    }

    /// How many 16 kHz samples the input so far makes.
    fn expected_len(&self) -> u64 {
        let rate = u64::from(self.rate);
        (self.frames_in * u64::from(MODEL_RATE) + rate / 2) / rate
    }
}

/// One call of the FFT resampler: resamples `input` (of which only `partial`
/// frames are real, when given), drops the delay still to trim, and appends
/// the rest to `out`. Returns how many input frames it took.
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

/// Resamples a whole clip at `rate` to 16 kHz in one go.
pub fn resample_to_16k(samples: &[f32], rate: u32) -> Result<Vec<f32>, String> {
    let mut resampler = Resampler16k::new(rate)?;
    let mut out = Vec::new();
    resampler.push(samples, &mut out);
    resampler.finish(&mut out);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn sine(freq: f32, rate: u32, len: usize, amp: f32) -> Vec<f32> {
        (0..len)
            .map(|i| amp * (2.0 * std::f32::consts::PI * freq * i as f32 / rate as f32).sin())
            .collect()
    }

    const RATES: [u32; 8] = [
        8_000, 16_000, 22_050, 24_000, 32_000, 44_100, 48_000, 96_000,
    ];

    #[test]
    fn stereo_frames_average() {
        assert_eq!(
            downmix(&[1.0, 0.0, 0.5, 0.5, -1.0, 1.0], 2),
            vec![0.5, 0.5, 0.0]
        );
    }

    #[test]
    fn an_incomplete_frame_is_dropped() {
        assert_eq!(downmix(&[1.0, 1.0, 1.0], 2), vec![1.0]);
    }

    #[test]
    fn a_full_scale_square_wave_fills_the_meter() {
        assert_eq!(meter_level(rms(&[1.0, -1.0, 1.0, -1.0])), 1.0);
        assert_eq!(meter_level(rms(&[])), 0.0);
        assert!((meter_level(0.001) - 0.0).abs() < 1e-6);
        assert!((meter_level(0.031_622_8) - 0.5).abs() < 1e-3);
    }

    #[test]
    fn sixteen_kilohertz_passes_through() {
        let input = sine(440.0, 16_000, 3_000, 0.5);
        assert_eq!(resample_to_16k(&input, 16_000).unwrap(), input);
    }

    #[test]
    fn a_440_hz_tone_keeps_its_loudness() {
        for rate in RATES {
            let input = sine(440.0, rate, rate as usize, 0.5);
            let out = resample_to_16k(&input, rate).unwrap();
            // Away from the edges, where the resampler fades in and out.
            let middle = &out[out.len() / 4..out.len() * 3 / 4];
            let expected = 0.5 / std::f32::consts::SQRT_2;
            let got = rms(middle);
            assert!(
                (got - expected).abs() / expected < 0.05,
                "{rate} Hz: rms {got}"
            );
        }
    }

    #[test]
    fn a_rate_of_zero_is_refused() {
        assert!(Resampler16k::new(0).is_err());
    }

    #[test]
    fn a_rate_no_recorder_uses_is_refused() {
        assert!(Resampler16k::new(4_294_967_291).is_err());
        assert!(Resampler16k::new(1_000).is_err());
        assert!(Resampler16k::new(8_000).is_ok());
    }

    proptest! {
        #[test]
        fn downmix_gives_one_sample_per_frame_within_the_frame(
            channels in 1usize..8,
            samples in prop::collection::vec(-1.0f32..1.0, 0..400),
        ) {
            let mono = downmix(&samples, channels);
            prop_assert_eq!(mono.len(), samples.len() / channels);
            for (frame, &m) in samples.chunks_exact(channels).zip(&mono) {
                let lo = frame.iter().cloned().fold(f32::INFINITY, f32::min);
                let hi = frame.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
                prop_assert!(m >= lo - 1e-6 && m <= hi + 1e-6);
            }
        }

        #[test]
        fn downmix_of_identical_channels_is_that_channel(
            channels in 1usize..8,
            mono in prop::collection::vec(-1.0f32..1.0, 0..200),
        ) {
            let interleaved: Vec<f32> = mono.iter().flat_map(|&s| std::iter::repeat_n(s, channels)).collect();
            let back = downmix(&interleaved, channels);
            for (a, b) in back.iter().zip(&mono) {
                prop_assert!((a - b).abs() < 1e-6);
            }
        }

        #[test]
        fn rms_is_between_zero_and_the_peak(samples in prop::collection::vec(-1.0f32..1.0, 0..500)) {
            let r = rms(&samples);
            let peak = samples.iter().fold(0.0f32, |m, s| m.max(s.abs()));
            prop_assert!(r >= 0.0 && r <= peak + 1e-6);
            let level = meter_level(r);
            prop_assert!((0.0..=1.0).contains(&level));
        }

        #[test]
        fn rms_of_a_constant_is_its_magnitude(value in -1.0f32..1.0, len in 1usize..500) {
            prop_assert!((rms(&vec![value; len]) - value.abs()).abs() < 1e-5);
        }

        #[test]
        fn the_meter_rises_with_loudness(a in 0.0f32..1.0, b in 0.0f32..1.0) {
            let (quiet, loud) = if a <= b { (a, b) } else { (b, a) };
            prop_assert!(meter_level(quiet) <= meter_level(loud));
        }

        #[test]
        fn resampled_length_matches_the_ratio(
            rate_index in 0usize..RATES.len(),
            len in 0usize..20_000,
            pieces in prop::collection::vec(1usize..3_000, 1..8),
        ) {
            let rate = RATES[rate_index];
            let input = sine(300.0, rate, len, 0.3);
            // Fed in uneven pieces, as the microphone does.
            let mut resampler = Resampler16k::new(rate).unwrap();
            let mut out = Vec::new();
            let mut at = 0;
            for piece in pieces.iter().cycle() {
                if at >= len { break; }
                let end = (at + piece).min(len);
                resampler.push(&input[at..end], &mut out);
                at = end;
            }
            resampler.finish(&mut out);
            let expected = len as f64 * 16_000.0 / rate as f64;
            prop_assert!((out.len() as f64 - expected).abs() <= 1.0, "{} vs {}", out.len(), expected);
            prop_assert!(out.iter().all(|s| s.is_finite()));
        }

        #[test]
        fn resampling_keeps_a_constant(rate_index in 0usize..RATES.len(), level in -0.9f32..0.9) {
            let rate = RATES[rate_index];
            let out = resample_to_16k(&vec![level; rate as usize], rate).unwrap();
            let middle = &out[out.len() / 4..out.len() * 3 / 4];
            let mean = middle.iter().sum::<f32>() / middle.len() as f32;
            prop_assert!((mean - level).abs() < 0.01 + level.abs() * 0.02, "mean {mean} for {level}");
        }
    }
}
