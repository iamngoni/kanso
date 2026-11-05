//! The speaker embedding's input: 80 log mel filterbank energies per 10 ms,
//! computed the way Kaldi (and so WeSpeaker, which trained the model) does,
//! with the average of each band over the clip taken out.

use std::sync::OnceLock;

use realfft::RealFftPlanner;

/// Mel bands per frame.
pub const BANDS: usize = 80;
/// A frame is 25 ms at 16 kHz.
const FRAME: usize = 400;
/// A new frame every 10 ms.
const HOP: usize = 160;
/// The FFT size: the frame padded to a power of two.
const FFT: usize = 512;
/// Kaldi's pre-emphasis.
const PREEMPHASIS: f32 = 0.97;
/// The lowest and highest frequency the bands span.
const LOW_HZ: f32 = 20.;
const HIGH_HZ: f32 = 8000.;
/// WeSpeaker feeds samples scaled to 16-bit integers.
const SCALE: f32 = 32_768.;

/// How many frames `samples` samples make: none for less than one frame.
/// Pure.
pub fn frames_in(samples: usize) -> usize {
    if samples < FRAME {
        0
    } else {
        1 + (samples - FRAME) / HOP
    }
}

fn mel(hz: f32) -> f32 {
    1127. * (1. + hz / 700.).ln()
}

/// For each of the 256 FFT bins below Nyquist, its weight in each band.
fn filters() -> &'static Vec<[f32; BANDS]> {
    static FILTERS: OnceLock<Vec<[f32; BANDS]>> = OnceLock::new();
    FILTERS.get_or_init(|| {
        let bins = FFT / 2;
        let width = 16_000. / FFT as f32;
        let (low, high) = (mel(LOW_HZ), mel(HIGH_HZ));
        let delta = (high - low) / (BANDS + 1) as f32;
        (0..bins)
            .map(|bin| {
                let m = mel(width * bin as f32);
                let mut weights = [0.0f32; BANDS];
                for (band, weight) in weights.iter_mut().enumerate() {
                    let left = low + band as f32 * delta;
                    let center = left + delta;
                    let right = center + delta;
                    if m > left && m < right {
                        *weight = if m <= center {
                            (m - left) / (center - left)
                        } else {
                            (right - m) / (right - center)
                        };
                    }
                }
                weights
            })
            .collect()
    })
}

fn window() -> &'static [f32; FRAME] {
    static WINDOW: OnceLock<[f32; FRAME]> = OnceLock::new();
    WINDOW.get_or_init(|| {
        let mut w = [0.0f32; FRAME];
        for (i, v) in w.iter_mut().enumerate() {
            let phase = 2. * std::f32::consts::PI * i as f32 / (FRAME - 1) as f32;
            *v = 0.54 - 0.46 * phase.cos();
        }
        w
    })
}

/// The log mel energies of `samples` (16 kHz, -1 to 1): [`frames_in`] rows
/// of [`BANDS`] numbers, flattened, each band's mean over the clip
/// subtracted. Empty when there is less than one frame. Pure.
pub fn features(samples: &[f32]) -> Vec<f32> {
    let frames = frames_in(samples.len());
    if frames == 0 {
        return Vec::new();
    }
    let fft = RealFftPlanner::<f32>::new().plan_fft_forward(FFT);
    let (filters, window) = (filters(), window());
    let mut out = Vec::with_capacity(frames * BANDS);
    let mut input = fft.make_input_vec();
    let mut spectrum = fft.make_output_vec();
    let mut scratch = fft.make_scratch_vec();
    for frame in 0..frames {
        let chunk = &samples[frame * HOP..frame * HOP + FRAME];
        let mean = chunk.iter().map(|&s| s * SCALE).sum::<f32>() / FRAME as f32;
        input.fill(0.);
        for i in 0..FRAME {
            let current = chunk[i] * SCALE - mean;
            let before = if i == 0 {
                current
            } else {
                chunk[i - 1] * SCALE - mean
            };
            input[i] = (current - PREEMPHASIS * before) * window[i];
        }
        // The transform cannot fail: the buffers are the planned sizes.
        let _ = fft.process_with_scratch(&mut input, &mut spectrum, &mut scratch);
        let mut bands = [0.0f32; BANDS];
        for (bin, weights) in filters.iter().enumerate() {
            let power = spectrum[bin].norm_sqr();
            for (band, weight) in weights.iter().enumerate() {
                bands[band] += power * weight;
            }
        }
        out.extend(bands.iter().map(|&e| e.max(f32::EPSILON).ln()));
    }
    for band in 0..BANDS {
        let mean = (0..frames).map(|f| out[f * BANDS + band]).sum::<f32>() / frames as f32;
        for f in 0..frames {
            out[f * BANDS + band] -= mean;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::{FileFailurePersistence, RngSeed};

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 48,
            rng_seed: RngSeed::Fixed(0x006b_616e_736f),
            failure_persistence: Some(Box::new(FileFailurePersistence::Off)),
            ..ProptestConfig::default()
        })]

        /// The right number of frames, every number finite, and every band
        /// averages to nothing over the clip.
        #[test]
        fn prop_features_are_finite_and_centred(
            samples in proptest::collection::vec(-1.0f32..1.0, 0..4_000),
        ) {
            let out = features(&samples);
            prop_assert_eq!(out.len(), frames_in(samples.len()) * BANDS);
            prop_assert!(out.iter().all(|v| v.is_finite()));
            let frames = frames_in(samples.len());
            for band in 0..BANDS {
                if frames > 0 {
                    let mean = (0..frames).map(|f| out[f * BANDS + band]).sum::<f32>() / frames as f32;
                    prop_assert!(mean.abs() < 1e-3, "band {band} mean {mean}");
                }
            }
        }

        /// Making a clip louder changes nothing once the bands are centred
        /// (a gain only adds a constant to every log energy), as long as it
        /// stays above the floor.
        #[test]
        fn prop_loudness_does_not_change_the_features(
            samples in proptest::collection::vec(-0.5f32..0.5, 800..2_000),
            gain in 0.5f32..1.9,
        ) {
            let quiet = features(&samples);
            let loud: Vec<f32> = samples.iter().map(|s| s * gain).collect();
            let loud = features(&loud);
            for (a, b) in quiet.iter().zip(&loud) {
                prop_assert!((a - b).abs() < 1e-2, "{a} vs {b}");
            }
        }
    }

    #[test]
    fn frames_follow_kaldis_count() {
        assert_eq!(frames_in(399), 0);
        assert_eq!(frames_in(400), 1);
        assert_eq!(frames_in(559), 1);
        assert_eq!(frames_in(560), 2);
        assert_eq!(frames_in(16_000), 98);
    }

    #[test]
    fn a_pure_tone_lights_the_band_at_its_pitch() {
        let tone: Vec<f32> = (0..16_000)
            .map(|i| 0.5 * (2. * std::f32::consts::PI * 1_000. * i as f32 / 16_000.).sin())
            .collect();
        // Centred bands say nothing about a steady tone, so follow it with
        // silence to see where the energy sat.
        let mut half = tone[..8_000].to_vec();
        half.extend(std::iter::repeat_n(0.0, 8_000));
        let out = features(&half);
        let first = &out[..BANDS];
        let peak = first
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .unwrap()
            .0;
        // 1 kHz is about band 24 of 80 on the mel scale.
        assert!((20..30).contains(&peak), "peak at band {peak}");
    }
}
