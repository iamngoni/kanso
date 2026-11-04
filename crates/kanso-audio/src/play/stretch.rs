//! Pitch-preserving time-stretch for speech: WSOLA, waveform-similarity
//! overlap-add.
//!
//! Windows of 40 ms are taken from the input at a hop of `speed` times the
//! output hop, each nudged by up to 10 ms to line its waveform up with the
//! one before, and overlap-added with a Hann window. Played faster the
//! words come quicker and the voice keeps its pitch. Stereo is stretched as
//! one (the channels share the nudge), so the image doesn't move.
//!
//! The output is linear in time: after `n` output frames the input is
//! `n × speed` frames in, whatever the nudges did, which is what lets the
//! player report a position on the recording's own clock.
//!
//! In-house rather than a crate: `signalsmith-stretch` (MIT) is a C++ library
//! behind a wrapper, and would put a C++ toolchain in the way of the Android,
//! iOS, Windows and Linux builds for one slider.

/// The speeds the player offers.
pub const SPEEDS: [f32; 5] = [0.75, 1.0, 1.25, 1.5, 2.0];

/// The slowest and fastest speed the stretcher accepts.
pub const SPEED_RANGE: std::ops::RangeInclusive<f32> = 0.5..=4.0;

pub struct Stretcher {
    speed: f64,
    n: usize,
    hs: usize,
    tol: usize,
    window: Vec<f32>,
    /// Interleaved stereo from absolute frame `base` on.
    input: Vec<f32>,
    base: u64,
    /// Where the next window nominally starts, in absolute input frames.
    next_in: f64,
    /// Where the previous window was actually taken from.
    prev: Option<u64>,
    /// The second half of the previous window, windowed, still to be added.
    tail: Vec<f32>,
}

impl Stretcher {
    /// A stretcher for stereo at `rate` Hz that plays `speed` times faster.
    pub fn new(rate: u32, speed: f32) -> Self {
        let speed = f64::from(speed.clamp(*SPEED_RANGE.start(), *SPEED_RANGE.end()));
        let n = ((rate as usize * 40 / 1000) & !1).max(64);
        let hs = n / 2;
        let tol = (rate as usize * 10 / 1000).max(8);
        let window = (0..n)
            .map(|i| 0.5 - 0.5 * (std::f32::consts::TAU * i as f32 / n as f32).cos())
            .collect();
        Stretcher {
            speed,
            n,
            hs,
            tol,
            window,
            input: Vec::new(),
            base: 0,
            next_in: 0.0,
            prev: None,
            tail: Vec::new(),
        }
    }

    pub fn speed(&self) -> f32 {
        self.speed as f32
    }

    /// Takes interleaved stereo and appends what is ready to `out`.
    pub fn push(&mut self, data: &[f32], out: &mut Vec<f32>) {
        self.input.extend_from_slice(data);
        self.run(out, false);
    }

    /// Ends the stream: what is left comes out, as if silence followed.
    pub fn finish(&mut self, out: &mut Vec<f32>) {
        self.run(out, true);
        out.extend_from_slice(&self.tail);
        self.tail.clear();
    }

    fn available(&self) -> u64 {
        self.base + (self.input.len() / 2) as u64
    }

    fn sample(&self, abs: u64, ch: usize) -> f32 {
        if abs < self.base {
            return 0.0;
        }
        self.input
            .get((abs - self.base) as usize * 2 + ch)
            .copied()
            .unwrap_or(0.0)
    }

    fn mono(&self, abs: u64) -> f32 {
        0.5 * (self.sample(abs, 0) + self.sample(abs, 1))
    }

    fn run(&mut self, out: &mut Vec<f32>, last: bool) {
        loop {
            let p = self.next_in.round() as u64;
            let needs = p + (self.tol + self.n) as u64;
            if last {
                if p >= self.available() {
                    break;
                }
            } else if needs > self.available() {
                break;
            }
            let q = match self.prev {
                Some(prev) => self.best_start(p, prev + self.hs as u64),
                None => p,
            };
            self.emit(q, out);
            self.prev = Some(q);
            self.next_in += self.hs as f64 * self.speed;
            // Keep what the next search and window can still reach.
            let keep_from =
                (q + self.hs as u64).min((self.next_in as u64).saturating_sub(self.tol as u64));
            if keep_from > self.base {
                let drop = ((keep_from - self.base) as usize).min(self.input.len() / 2);
                self.input.drain(..drop * 2);
                self.base += drop as u64;
            }
        }
    }

    /// The start near `p` whose first half is most like the input from
    /// `target`: where the previous window would have gone on.
    fn best_start(&self, p: u64, target: u64) -> u64 {
        let lo = p.saturating_sub(self.tol as u64).max(self.base);
        let hi = (p + self.tol as u64).max(lo);
        let score = |cand: u64, stride: usize| -> f32 {
            let (mut dot, mut energy) = (0.0f32, 1e-9f32);
            let mut i = 0;
            while i < self.hs {
                let a = self.mono(cand + i as u64);
                dot += a * self.mono(target + i as u64);
                energy += a * a;
                i += stride;
            }
            dot / energy.sqrt()
        };
        let mut best = (f32::MIN, p.clamp(lo, hi));
        let mut cand = lo;
        while cand <= hi {
            let s = score(cand, 2);
            if s > best.0 {
                best = (s, cand);
            }
            cand += 4;
        }
        let (mut best_score, mut best_at) = (f32::MIN, best.1);
        for cand in best.1.saturating_sub(3).max(lo)..=(best.1 + 3).min(hi) {
            let s = score(cand, 1);
            if s > best_score {
                best_score = s;
                best_at = cand;
            }
        }
        best_at
    }

    /// Adds the window at `q` to the tail, emits one hop and keeps the rest.
    fn emit(&mut self, q: u64, out: &mut Vec<f32>) {
        let first = self.prev.is_none();
        let (n, hs) = (self.n, self.hs);
        let mut next_tail = Vec::with_capacity(hs * 2);
        for i in 0..n {
            // The very first window has no earlier one to fade out of.
            let w = if first && i < hs { 1.0 } else { self.window[i] };
            let abs = q + i as u64;
            let (l, r) = (self.sample(abs, 0) * w, self.sample(abs, 1) * w);
            if i < hs {
                let (tl, tr) = if first {
                    (0.0, 0.0)
                } else {
                    (self.tail[i * 2], self.tail[i * 2 + 1])
                };
                out.push(tl + l);
                out.push(tr + r);
            } else {
                next_tail.push(l);
                next_tail.push(r);
            }
        }
        self.tail = next_tail;
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::{FileFailurePersistence, RngSeed};

    pub(crate) fn tone(freq: f32, rate: u32, secs: f32) -> Vec<f32> {
        (0..(rate as f32 * secs) as usize)
            .flat_map(|i| {
                let v = 0.5 * (std::f32::consts::TAU * freq * i as f32 / rate as f32).sin();
                [v, v]
            })
            .collect()
    }

    /// Hz of a left channel, by counting rising zero crossings.
    pub(crate) fn pitch(stereo: &[f32], rate: u32) -> f32 {
        let left: Vec<f32> = stereo.as_chunks::<2>().0.iter().map(|f| f[0]).collect();
        let crossings = left
            .windows(2)
            .filter(|w| w[0] < 0.0 && w[1] >= 0.0)
            .count();
        crossings as f32 * rate as f32 / left.len() as f32
    }

    fn stretch(input: &[f32], rate: u32, speed: f32, chunk: usize) -> Vec<f32> {
        let mut s = Stretcher::new(rate, speed);
        let mut out = Vec::new();
        for piece in input.chunks(chunk * 2) {
            s.push(piece, &mut out);
        }
        s.finish(&mut out);
        out
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 24,
            rng_seed: RngSeed::Fixed(0x006b_616e_736f),
            failure_persistence: Some(Box::new(FileFailurePersistence::Off)),
            ..ProptestConfig::default()
        })]

        /// Faster is shorter by the speed, and the pitch stays where it was,
        /// however the input arrives.
        #[test]
        fn prop_length_scales_and_pitch_holds(
            speed in prop::sample::select(SPEEDS.to_vec()),
            freq in 120f32..420.0,
            chunk in 100usize..6_000,
        ) {
            let rate = 16_000;
            let input = tone(freq, rate, 2.0);
            let out = stretch(&input, rate, speed, chunk);
            let in_frames = (input.len() / 2) as f32;
            let out_frames = (out.len() / 2) as f32;
            let expected = in_frames / speed;
            prop_assert!(
                (out_frames - expected).abs() < rate as f32 * 0.12,
                "{out_frames} frames out, wanted about {expected}"
            );
            let heard = pitch(&out, rate);
            prop_assert!(
                (heard - freq).abs() / freq < 0.04,
                "{freq} Hz came out as {heard} Hz at {speed}x"
            );
        }

        /// However it is cut up, the same input makes the same output.
        #[test]
        fn prop_chunking_does_not_matter(chunk in 50usize..3_000) {
            let input = tone(200.0, 16_000, 1.0);
            prop_assert_eq!(
                stretch(&input, 16_000, 1.5, chunk),
                stretch(&input, 16_000, 1.5, 977)
            );
        }
    }

    #[test]
    fn silence_stays_silent_and_nothing_goes_wild() {
        let out = stretch(&vec![0.0; 40_000], 16_000, 2.0, 1_000);
        assert!(out.iter().all(|v| *v == 0.0));
        let loud = tone(300.0, 16_000, 1.0);
        assert!(
            stretch(&loud, 16_000, 0.75, 700)
                .iter()
                .all(|v| v.abs() <= 1.0)
        );
    }
}
