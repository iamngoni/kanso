//! The microphone and the meeting on one timeline (issue #38, step 10).
//!
//! A recording with both sources is stored as two channels at 48 kHz: left
//! is you (the microphone), right is the meeting (system audio). Nothing is
//! summed, so the voices never fight for headroom and the speaker labels
//! can be read back from the channels; [`Mixer`] only makes the two sources
//! line up. They run on two different clocks (a microphone and a sound
//! card each count "48 000 a second" a little differently), so left alone
//! one drifts from the other by a few milliseconds an hour.
//!
//! [`Mixer`] queues each source and writes a frame only when both have it.
//! When one runs ahead of the other by more than [`TOLERANCE`] (the design's
//! 100 ms), it drops one 20 ms frame from the leader, at most once a second,
//! until the lead is back inside. A source that stalls is padded with silence
//! by whoever reads it (see [`crate::system_apps::GapFiller`]), so a quiet
//! source never holds the other back. [`MixPipeline`] is the recorder's
//! side: two resamplers, the mixer and the stereo [`Recorder`].

use std::collections::VecDeque;

use crate::error::AudioError;
use crate::model::{FRAME_SAMPLES, SAMPLE_RATE};
use crate::record_model::{Finished, Progress};
use crate::resample::To48k;
use crate::session::Recorder;

/// How far one source may run ahead of the other before a frame is dropped:
/// 100 ms at 48 kHz.
pub const TOLERANCE: usize = SAMPLE_RATE as usize / 10;
/// Output frames between two corrections: one second, so a stall of a
/// source costs at most 20 ms of the other's audio in every second.
pub const CORRECT_EVERY: u64 = SAMPLE_RATE as u64;

/// A sample as it is stored: finite, and never past full scale.
pub fn limit(sample: f32) -> f32 {
    if sample.is_finite() {
        sample.clamp(-1.0, 1.0)
    } else {
        0.0
    }
}

/// Two mono sources on one 48 kHz timeline, as interleaved left (you) and
/// right (the meeting) frames.
#[derive(Debug, Default)]
pub struct Mixer {
    you: VecDeque<f32>,
    meeting: VecDeque<f32>,
    /// Frames written since the last correction.
    since_correction: u64,
    corrections: u64,
}

impl Mixer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_you(&mut self, samples: &[f32]) {
        self.you.extend(samples.iter().copied());
    }

    pub fn push_meeting(&mut self, samples: &[f32]) {
        self.meeting.extend(samples.iter().copied());
    }

    /// Samples the microphone is ahead of the meeting by (negative: behind).
    pub fn skew(&self) -> i64 {
        self.you.len() as i64 - self.meeting.len() as i64
    }

    /// Frames dropped to keep the sources together so far.
    pub fn corrections(&self) -> u64 {
        self.corrections
    }

    /// Appends every frame both sources have to `out` and returns how many.
    /// Then, if one source leads by more than [`TOLERANCE`] and a second of
    /// audio has passed since the last correction, drops one frame from the
    /// leader.
    pub fn drain(&mut self, out: &mut Vec<f32>) -> usize {
        let frames = self.you.len().min(self.meeting.len());
        out.reserve(frames * 2);
        for (you, meeting) in self.you.drain(..frames).zip(self.meeting.drain(..frames)) {
            out.push(limit(you));
            out.push(limit(meeting));
        }
        self.since_correction = self.since_correction.saturating_add(frames as u64);
        let lead = self.you.len().max(self.meeting.len());
        if lead > TOLERANCE && self.since_correction >= CORRECT_EVERY {
            let leader = if self.you.len() > self.meeting.len() {
                &mut self.you
            } else {
                &mut self.meeting
            };
            leader.drain(..FRAME_SAMPLES.min(leader.len()));
            self.since_correction = 0;
            self.corrections += 1;
        }
        frames
    }

    /// Writes what is left, the shorter source padded with silence so the
    /// leader's last audio is kept. Returns the frames written.
    pub fn finish(&mut self, out: &mut Vec<f32>) -> usize {
        let frames = self.you.len().max(self.meeting.len());
        self.you.resize(frames, 0.0);
        self.meeting.resize(frames, 0.0);
        self.drain(out)
    }
}

/// Two microphone-rate (or system-rate) streams in, a stereo recording on
/// disk out. `recorder` must be stereo.
pub struct MixPipeline {
    you: To48k,
    meeting: To48k,
    mixer: Mixer,
    recorder: Recorder,
    paused: bool,
    resampled: Vec<f32>,
    mixed: Vec<f32>,
    /// Samples each source has given, at 48 kHz.
    given_you: u64,
    given_meeting: u64,
    rms: f32,
}

impl MixPipeline {
    pub fn new(recorder: Recorder, you_rate: u32, meeting_rate: u32) -> Result<Self, AudioError> {
        if recorder.session().channels != 2 {
            return Err(AudioError::Config(
                "the microphone and system audio record two channels".into(),
            ));
        }
        Ok(MixPipeline {
            you: To48k::new(you_rate)?,
            meeting: To48k::new(meeting_rate)?,
            mixer: Mixer::new(),
            recorder,
            paused: false,
            resampled: Vec::new(),
            mixed: Vec::new(),
            given_you: 0,
            given_meeting: 0,
            rms: 0.0,
        })
    }

    pub fn recorder(&self) -> &Recorder {
        &self.recorder
    }

    pub fn mixer(&self) -> &Mixer {
        &self.mixer
    }

    /// Samples at 48 kHz the microphone has given the recording.
    pub fn given_you(&self) -> u64 {
        self.given_you
    }

    /// Samples at 48 kHz the meeting has given the recording, silence that
    /// stood in for it included.
    pub fn given_meeting(&self) -> u64 {
        self.given_meeting
    }

    /// The louder channel's root mean square over what was written last;
    /// for the level meter.
    pub fn rms(&self) -> f32 {
        self.rms
    }

    /// While paused, sound is dropped, so the clock stops. Pausing writes
    /// what is queued first, so the two sources start together again.
    pub fn set_paused(&mut self, paused: bool) -> Result<(), AudioError> {
        if paused && !self.paused {
            self.mixed.clear();
            self.mixer.finish(&mut self.mixed);
            self.recorder.push(&self.mixed)?;
            self.recorder.checkpoint()?;
        }
        self.paused = paused;
        Ok(())
    }

    /// Takes the microphone's sound at its own rate.
    pub fn feed_you(&mut self, mono: &[f32]) {
        if self.paused {
            return;
        }
        self.resampled.clear();
        self.you.push(mono, &mut self.resampled);
        self.given_you += self.resampled.len() as u64;
        self.mixer.push_you(&self.resampled);
    }

    /// Takes the meeting's sound at its own rate.
    pub fn feed_meeting(&mut self, mono: &[f32]) {
        if self.paused {
            return;
        }
        self.resampled.clear();
        self.meeting.push(mono, &mut self.resampled);
        self.given_meeting += self.resampled.len() as u64;
        self.mixer.push_meeting(&self.resampled);
    }

    /// Writes what both sources have given so far.
    pub fn flush(&mut self) -> Result<Progress, AudioError> {
        if self.paused {
            return Ok(Progress {
                samples: self.recorder.samples(),
                ..Progress::default()
            });
        }
        self.mixed.clear();
        self.mixer.drain(&mut self.mixed);
        self.measure();
        self.recorder.push(&self.mixed)
    }

    fn measure(&mut self) {
        let tail = self
            .mixed
            .len()
            .saturating_sub(SAMPLE_RATE as usize / 20 * 2);
        let window = &self.mixed[tail..];
        if window.is_empty() {
            return;
        }
        let (mut left, mut right) = (0.0f64, 0.0f64);
        for &[l, r] in window.as_chunks::<2>().0 {
            left += f64::from(l) * f64::from(l);
            right += f64::from(r) * f64::from(r);
        }
        let frames = (window.len() / 2).max(1) as f64;
        self.rms = (left.max(right) / frames).sqrt() as f32;
    }

    /// Flushes the resamplers and the queues and closes the recording.
    pub fn finish(mut self) -> Result<Finished, AudioError> {
        if !self.paused {
            self.resampled.clear();
            self.you.finish(&mut self.resampled);
            self.mixer.push_you(&self.resampled);
            self.resampled.clear();
            self.meeting.finish(&mut self.resampled);
            self.mixer.push_meeting(&self.resampled);
            self.mixed.clear();
            self.mixer.finish(&mut self.mixed);
            self.recorder.push(&self.mixed)?;
        }
        self.recorder.finish()
    }

    pub fn discard(self) -> std::io::Result<()> {
        self.recorder.discard()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::{FileFailurePersistence, RngSeed};

    /// What a drifting run leaves.
    struct Run {
        mixer: Mixer,
        written: u64,
        pushed_you: u64,
        pushed_meeting: u64,
    }

    /// Runs two sources with clocks that differ by `ppm` parts per million
    /// for `seconds`, each delivering every 20 ms tick of its own length.
    /// Without `correct` the mixer is kept from correcting, to show what
    /// the drift would do.
    fn run_drifting(ppm: i64, seconds: u64, correct: bool) -> Run {
        let mut run = Run {
            mixer: Mixer::new(),
            written: 0,
            pushed_you: 0,
            pushed_meeting: 0,
        };
        let mut out = Vec::new();
        let (mut owed_you, mut owed_meeting) = (0f64, 0f64);
        let you_rate = 1.0 + ppm as f64 / 1e6;
        for _ in 0..seconds * 50 {
            owed_you += FRAME_SAMPLES as f64 * you_rate;
            owed_meeting += FRAME_SAMPLES as f64;
            let (a, b) = (owed_you.floor() as usize, owed_meeting.floor() as usize);
            owed_you -= a as f64;
            owed_meeting -= b as f64;
            run.mixer.push_you(&vec![0.1; a]);
            run.mixer.push_meeting(&vec![0.1; b]);
            run.pushed_you += a as u64;
            run.pushed_meeting += b as u64;
            out.clear();
            run.written += run.mixer.drain(&mut out) as u64;
            if !correct {
                run.mixer.since_correction = 0;
            }
        }
        run
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 64,
            rng_seed: RngSeed::Fixed(0x006b_616e_736f),
            failure_persistence: Some(Box::new(FileFailurePersistence::Off)),
            ..ProptestConfig::default()
        })]

        /// However loud and however broken the input, every sample written
        /// is within full scale.
        #[test]
        fn prop_mixing_never_clips_past_the_limit(
            you in proptest::collection::vec(prop_oneof![
                -50.0f32..50.0,
                Just(f32::NAN),
                Just(f32::INFINITY),
                Just(f32::NEG_INFINITY),
            ], 0..400),
            meeting in proptest::collection::vec(-50.0f32..50.0, 0..400),
        ) {
            let mut mixer = Mixer::new();
            mixer.push_you(&you);
            mixer.push_meeting(&meeting);
            let mut out = Vec::new();
            mixer.drain(&mut out);
            mixer.finish(&mut out);
            prop_assert_eq!(out.len(), you.len().max(meeting.len()) * 2);
            for sample in out {
                prop_assert!((-1.0..=1.0).contains(&sample), "{}", sample);
            }
        }

        /// Samples in range pass through untouched, you on the left and the
        /// meeting on the right, none lost or invented: the mix is a
        /// lossless interleave.
        #[test]
        fn prop_the_mix_is_the_two_sources_side_by_side(
            you in proptest::collection::vec(-1.0f32..1.0, 0..300),
            meeting in proptest::collection::vec(-1.0f32..1.0, 0..300),
            split in 0usize..300,
        ) {
            let mut mixer = Mixer::new();
            let mut out = Vec::new();
            // The sources arrive in different cuts.
            let cut = split.min(you.len());
            mixer.push_you(&you[..cut]);
            mixer.push_meeting(&meeting);
            mixer.drain(&mut out);
            mixer.push_you(&you[cut..]);
            mixer.drain(&mut out);
            mixer.finish(&mut out);
            let frames = you.len().max(meeting.len());
            prop_assert_eq!(out.len(), frames * 2);
            for i in 0..frames {
                prop_assert_eq!(out[2 * i], you.get(i).copied().unwrap_or(0.0));
                prop_assert_eq!(out[2 * i + 1], meeting.get(i).copied().unwrap_or(0.0));
            }
        }

        /// Clocks that differ by up to 0.5 % are brought together: after
        /// any run the lead is back within the tolerance plus a frame.
        #[test]
        fn prop_drift_correction_converges(ppm in -5_000i64..5_000, seconds in 30u64..120) {
            let run = run_drifting(ppm, seconds, true);
            let lead = run.mixer.skew().unsigned_abs() as usize;
            prop_assert!(lead <= TOLERANCE + FRAME_SAMPLES * 2, "lead {lead}");
            prop_assert!(run.mixer.corrections() <= seconds);
        }

        /// Nothing is lost except the corrections: what the sources gave is
        /// what was written, what is still queued, and the frames dropped.
        #[test]
        fn prop_correction_drops_only_what_it_says(ppm in -5_000i64..5_000, seconds in 30u64..90) {
            let run = run_drifting(ppm, seconds, true);
            let queued = (run.mixer.you.len() + run.mixer.meeting.len()) as u64;
            let dropped = run.mixer.corrections() * FRAME_SAMPLES as u64;
            prop_assert_eq!(
                run.pushed_you + run.pushed_meeting,
                2 * run.written + queued + dropped
            );
        }
    }

    #[test]
    fn without_correction_a_fast_clock_would_run_away() {
        let run = run_drifting(5_000, 120, false);
        // 0.5 % of two minutes is 600 ms.
        assert!(run.mixer.skew() > 5 * TOLERANCE as i64);
        let run = run_drifting(5_000, 120, true);
        assert!(run.mixer.skew() <= (TOLERANCE + FRAME_SAMPLES) as i64);
        assert!(run.mixer.corrections() > 0);
    }

    #[test]
    fn sources_on_the_same_clock_are_never_corrected() {
        let run = run_drifting(0, 60, true);
        assert_eq!(run.mixer.corrections(), 0);
        assert_eq!(run.written, 60 * 50 * FRAME_SAMPLES as u64);
    }

    #[test]
    fn a_hundred_milliseconds_of_lead_is_left_alone() {
        let mut mixer = Mixer::new();
        mixer.push_you(&vec![0.0; TOLERANCE]);
        let mut out = Vec::new();
        mixer.since_correction = CORRECT_EVERY;
        mixer.drain(&mut out);
        assert_eq!(mixer.corrections(), 0);
        mixer.push_you(&[0.0]);
        mixer.drain(&mut out);
        assert_eq!(mixer.corrections(), 1);
        assert_eq!(mixer.skew(), (TOLERANCE + 1 - FRAME_SAMPLES) as i64);
    }

    #[test]
    fn a_quiet_microphone_keeps_the_meeting_when_it_is_padded() {
        // The reader pads a stalled source with silence; the meeting's
        // audio then comes out whole beside it.
        let mut mixer = Mixer::new();
        mixer.push_you(&vec![0.0; 4800]);
        mixer.push_meeting(&vec![0.5; 4800]);
        let mut out = Vec::new();
        mixer.drain(&mut out);
        assert!(
            out.as_chunks::<2>()
                .0
                .iter()
                .all(|f| f[0] == 0.0 && f[1] == 0.5)
        );
    }
}
