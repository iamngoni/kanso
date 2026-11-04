//! The player without a sound card: a decoder that fills a ring buffer, an
//! output that drains it, and the atomics the interface reads and writes.
//!
//! ```text
//! controls ──► Shared ◄── position, ended ──── UI (10 Hz)
//!                │ epoch
//! parts ─► Decoder ─► samples ring ─► Output (the audio callback)
//!            │  resample, stretch        ▲
//!            └─► marks ring ─────────────┘
//! ```
//!
//! The ring carries stereo `f32` at the card's rate, plus a second ring of
//! small [`Mark`]s: each says "the next `frames` samples start at this moment
//! of the recording, at this speed, and belong to this epoch". That is the
//! whole protocol:
//!
//! - The callback's position is the mark's start plus what it has played of
//!   it, scaled by speed: the position is what is being heard, not what has
//!   been decoded.
//! - A seek, or a change of speed, bumps the epoch. Audio of an older epoch
//!   still in the ring is skipped by the callback, so a seek is heard at
//!   once and nothing waits for the ring to drain. The epoch shares one
//!   atomic with the position, so the callback can't write a stale position
//!   after a seek.
//! - The end of the recording is a mark of its own.
//!
//! The callback never locks and never allocates. The decoder runs on its own
//! thread (see [`super::device`]) or, in tests, a step at a time.

use std::io::Cursor;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use rtrb::{Consumer, Producer, RingBuffer};

use super::convert::{Resample, to_stereo};
use super::stretch::{SPEED_RANGE, Stretcher};
use super::timeline::{MICROS, Timeline, frames_to_us, us_to_frames};
use crate::error::AudioError;
use crate::source::{PcmSource, open};

/// One part of a recording, as it was attached.
#[derive(Clone, Debug)]
pub struct PartInput {
    pub bytes: Arc<[u8]>,
    /// `opus`, `m4a`, `mp3`…: a hint for Symphonia; Ogg Opus is told by its
    /// first page.
    pub extension: Option<String>,
}

impl PartInput {
    pub fn new(bytes: Vec<u8>, extension: Option<&str>) -> Self {
        PartInput {
            bytes: bytes.into(),
            extension: extension.map(str::to_owned),
        }
    }

    fn source(&self) -> Result<Box<dyn PcmSource>, AudioError> {
        open(
            Box::new(Cursor::new(self.bytes.clone())),
            self.extension.as_deref(),
        )
    }
}

/// How long each part plays, from the files themselves.
pub fn measure(parts: &[PartInput]) -> Result<Timeline, AudioError> {
    let mut lengths = Vec::with_capacity(parts.len());
    for part in parts {
        let mut source = part.source()?;
        let rate = source.sample_rate();
        let frames = match source.total_frames() {
            Some(frames) => frames,
            // A file that doesn't say is read through once.
            None => {
                let mut frames = 0u64;
                let mut buf = Vec::new();
                while let Some(channels) = source.read(&mut buf)? {
                    frames += (buf.len() / channels.max(1)) as u64;
                }
                frames
            }
        };
        lengths.push(frames_to_us(frames, rate));
    }
    Ok(Timeline::new(lengths))
}

const POSITION_BITS: u32 = 48;
const POSITION_MASK: u64 = (1 << POSITION_BITS) - 1;

fn pack(epoch: u64, position_us: u64) -> u64 {
    ((epoch & 0xffff) << POSITION_BITS) | (position_us & POSITION_MASK)
}

fn epoch_of(state: u64) -> u64 {
    state >> POSITION_BITS
}

fn position_of(state: u64) -> u64 {
    state & POSITION_MASK
}

/// What the interface, the decoder and the audio callback share.
pub struct Shared {
    /// The epoch in the top 16 bits and the position in microseconds below.
    state: AtomicU64,
    playing: AtomicBool,
    ended: AtomicBool,
    speed_milli: AtomicU32,
    volume_milli: AtomicU32,
    stop: AtomicBool,
    error: Mutex<Option<String>>,
    length_us: u64,
}

impl Shared {
    pub fn new(length_us: u64) -> Arc<Self> {
        Arc::new(Shared {
            state: AtomicU64::new(pack(0, 0)),
            playing: AtomicBool::new(false),
            ended: AtomicBool::new(false),
            speed_milli: AtomicU32::new(1_000),
            volume_milli: AtomicU32::new(1_000),
            stop: AtomicBool::new(false),
            error: Mutex::new(None),
            length_us,
        })
    }

    pub(crate) fn stopped(&self) -> bool {
        self.stop.load(Ordering::Relaxed)
    }

    pub(crate) fn fail(&self, message: String) {
        if let Ok(mut slot) = self.error.lock() {
            slot.get_or_insert(message);
        }
    }

    /// Sets the position and starts a new epoch.
    fn restart(&self, f: impl Fn(u64) -> u64) {
        let mut current = self.state.load(Ordering::Acquire);
        loop {
            let next = pack(epoch_of(current) + 1, f(position_of(current)));
            match self.state.compare_exchange_weak(
                current,
                next,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return,
                Err(seen) => current = seen,
            }
        }
    }
}

/// Where playback stands, for the interface.
#[derive(Clone, Debug, PartialEq)]
pub struct Status {
    pub position_us: u64,
    pub length_us: u64,
    pub playing: bool,
    pub ended: bool,
    pub speed: f32,
    pub volume: f32,
    pub error: Option<String>,
}

/// The controls. Cheap to call from any thread.
#[derive(Clone)]
pub struct Control {
    shared: Arc<Shared>,
}

impl Control {
    pub fn new(shared: Arc<Shared>) -> Self {
        Control { shared }
    }

    pub fn length_us(&self) -> u64 {
        self.shared.length_us
    }

    /// The moment being heard, never past the end.
    pub fn position_us(&self) -> u64 {
        position_of(self.shared.state.load(Ordering::Acquire)).min(self.shared.length_us)
    }

    pub fn is_playing(&self) -> bool {
        self.shared.playing.load(Ordering::Relaxed)
    }

    pub fn speed(&self) -> f32 {
        self.shared.speed_milli.load(Ordering::Relaxed) as f32 / 1_000.0
    }

    pub fn volume(&self) -> f32 {
        self.shared.volume_milli.load(Ordering::Relaxed) as f32 / 1_000.0
    }

    pub fn status(&self) -> Status {
        Status {
            position_us: self.position_us(),
            length_us: self.length_us(),
            playing: self.is_playing(),
            ended: self.shared.ended.load(Ordering::Relaxed),
            speed: self.speed(),
            volume: self.volume(),
            error: self.shared.error.lock().ok().and_then(|e| e.clone()),
        }
    }

    /// Plays from where it is; from the start when it had reached the end.
    pub fn play(&self) {
        if self.shared.ended.load(Ordering::Relaxed) || self.position_us() >= self.shared.length_us
        {
            self.seek_us(0);
        }
        self.shared.playing.store(true, Ordering::Relaxed);
    }

    pub fn pause(&self) {
        self.shared.playing.store(false, Ordering::Relaxed);
    }

    pub fn toggle(&self) {
        if self.is_playing() {
            self.pause();
        } else {
            self.play();
        }
    }

    /// Moves to `at`, held within the recording. Heard at once if playing.
    pub fn seek_us(&self, at: u64) {
        let at = at.min(self.shared.length_us);
        self.shared.ended.store(false, Ordering::Relaxed);
        self.shared.restart(|_| at);
    }

    /// Moves by `delta_us` from the position now.
    pub fn skip_us(&self, delta_us: i64) {
        let length = self.shared.length_us;
        let now = self.position_us();
        let moved = if delta_us >= 0 {
            now.saturating_add(delta_us as u64)
        } else {
            now.saturating_sub(delta_us.unsigned_abs())
        };
        self.seek_us(moved.min(length));
    }

    /// 0.5× to 4×; anything else is held to that.
    pub fn set_speed(&self, speed: f32) {
        let speed = if speed.is_finite() { speed } else { 1.0 };
        let speed = speed.clamp(*SPEED_RANGE.start(), *SPEED_RANGE.end());
        let milli = (speed * 1_000.0).round() as u32;
        if self.shared.speed_milli.swap(milli, Ordering::Relaxed) != milli {
            self.shared.restart(|at| at);
        }
    }

    /// 0 to 1.
    pub fn set_volume(&self, volume: f32) {
        let volume = if volume.is_finite() { volume } else { 1.0 };
        let milli = (volume.clamp(0.0, 1.0) * 1_000.0).round() as u32;
        self.shared.volume_milli.store(milli, Ordering::Relaxed);
    }

    /// Asks the decoder thread to end.
    pub fn stop(&self) {
        self.shared.playing.store(false, Ordering::Relaxed);
        self.shared.stop.store(true, Ordering::Relaxed);
    }
}

/// What the next samples in the ring are.
#[derive(Clone, Copy, Debug)]
struct Mark {
    epoch: u64,
    /// Stereo frames in the chunk (none for the end).
    frames: u32,
    /// The recording's moment at the chunk's first frame.
    start_us: u64,
    speed_milli: u32,
    end: bool,
}

/// Frames in one chunk: 20 ms.
fn chunk_frames(rate: u32) -> usize {
    (rate as usize / 50).max(64)
}

/// A ring big enough for three-quarters of a second.
fn ring_samples(rate: u32) -> usize {
    rate as usize * 2 * 3 / 4
}

const MARK_SLOTS: usize = 128;

/// What a decoder step did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// Made progress; step again.
    Worked,
    /// The ring is full: wait a little.
    Blocked,
    /// Nothing to do until a seek.
    Idle,
}

/// A part being read.
struct Reading {
    part: usize,
    source: Box<dyn PcmSource>,
    resample: Resample,
    stretch: Option<Stretcher>,
    /// The recording's moment of output frame 0.
    origin_us: u64,
    /// Output frames handed to the ring.
    emitted: u64,
    /// The last part is finished and flushed.
    input_done: bool,
}

/// Fills the ring from the parts.
pub struct Decoder {
    shared: Arc<Shared>,
    parts: Vec<PartInput>,
    timeline: Timeline,
    rate: u32,
    samples: Producer<f32>,
    marks: Producer<Mark>,
    epoch: Option<u64>,
    reading: Option<Reading>,
    /// Processed samples not yet in the ring.
    out: Vec<f32>,
    scratch: Vec<f32>,
    stereo: Vec<f32>,
    resampled: Vec<f32>,
    finished: bool,
}

impl Decoder {
    /// Reads from the epoch's position until the ring is full or the
    /// recording ends.
    pub fn step(&mut self) -> Step {
        let state = self.shared.state.load(Ordering::Acquire);
        let epoch = epoch_of(state);
        if self.epoch != Some(epoch) {
            self.restart(epoch, position_of(state));
            return Step::Worked;
        }
        if !self.out.is_empty() {
            return self.push_chunk();
        }
        if self.finished {
            return Step::Idle;
        }
        let Some(reading) = self.reading.as_mut() else {
            return self.push_end();
        };
        if reading.input_done {
            return self.push_end();
        }
        self.fill();
        Step::Worked
    }

    fn restart(&mut self, epoch: u64, position_us: u64) {
        self.epoch = Some(epoch);
        self.out.clear();
        self.reading = None;
        self.finished = false;
        let Some(place) = self.timeline.locate(position_us) else {
            return;
        };
        if position_us >= self.timeline.total_us() {
            return;
        }
        let speed = self.shared.speed_milli.load(Ordering::Relaxed) as f32 / 1_000.0;
        match self.open_part(place.part, Some(place.offset_us), speed) {
            Ok(mut reading) => {
                reading.origin_us = position_us;
                self.reading = Some(reading);
            }
            Err(e) => self.shared.fail(e.to_string()),
        }
    }

    fn open_part(
        &self,
        part: usize,
        offset_us: Option<u64>,
        speed: f32,
    ) -> Result<Reading, AudioError> {
        let mut source = self.parts[part].source()?;
        let src_rate = source.sample_rate();
        if let Some(offset) = offset_us.filter(|o| *o > 0) {
            source.seek(us_to_frames(offset, src_rate))?;
        }
        Ok(Reading {
            part,
            resample: Resample::new(src_rate, self.rate)?,
            source,
            stretch: ((speed - 1.0).abs() > 0.001).then(|| Stretcher::new(self.rate, speed)),
            origin_us: 0,
            emitted: 0,
            input_done: false,
        })
    }

    /// Decodes one packet (or the end of a part) into `out`.
    fn fill(&mut self) {
        let Some(mut reading) = self.reading.take() else {
            return;
        };
        match reading.source.read(&mut self.scratch) {
            Ok(Some(channels)) => {
                self.stereo.clear();
                to_stereo(&self.scratch, channels, &mut self.stereo);
                self.resampled.clear();
                reading.resample.push(&self.stereo, &mut self.resampled);
                Self::through(&mut reading, &self.resampled, &mut self.out);
            }
            Ok(None) => self.next_part(&mut reading),
            Err(e) => {
                self.shared.fail(e.to_string());
                self.end_input(&mut reading);
            }
        }
        self.reading = Some(reading);
    }

    fn through(reading: &mut Reading, samples: &[f32], out: &mut Vec<f32>) {
        match reading.stretch.as_mut() {
            Some(stretch) => stretch.push(samples, out),
            None => out.extend_from_slice(samples),
        }
    }

    /// A part ran out: the next one follows without a gap, or the recording
    /// ends.
    fn next_part(&mut self, reading: &mut Reading) {
        let next = reading.part + 1;
        if next >= self.parts.len() {
            self.end_input(reading);
            return;
        }
        let source = match self.parts[next].source() {
            Ok(source) => source,
            Err(e) => {
                self.shared.fail(e.to_string());
                self.end_input(reading);
                return;
            }
        };
        let rate = source.sample_rate();
        if rate != reading.resample.from_rate() {
            // Another rate: the old resampler gives up what it holds first.
            self.resampled.clear();
            reading.resample.finish(&mut self.resampled);
            Self::through(reading, &self.resampled, &mut self.out);
            match Resample::new(rate, self.rate) {
                Ok(resample) => reading.resample = resample,
                Err(e) => {
                    self.shared.fail(e.to_string());
                    self.end_input(reading);
                    return;
                }
            }
        }
        // The stretcher keeps going across the seam; only the file changes.
        reading.source = source;
        reading.part = next;
    }

    /// No more input: flush the resampler and the stretcher.
    fn end_input(&mut self, reading: &mut Reading) {
        self.resampled.clear();
        reading.resample.finish(&mut self.resampled);
        Self::through(reading, &self.resampled, &mut self.out);
        if let Some(stretch) = reading.stretch.as_mut() {
            stretch.finish(&mut self.out);
        }
        reading.input_done = true;
    }

    fn push_chunk(&mut self) -> Step {
        let frames = (self.out.len() / 2).min(chunk_frames(self.rate));
        // Whole chunks only, until the last: a chunk is a mark's worth.
        if self.samples.slots() < frames * 2 || self.marks.slots() < 1 {
            return Step::Blocked;
        }
        let Some(reading) = self.reading.as_mut() else {
            self.out.clear();
            return Step::Worked;
        };
        let speed_milli = (reading.stretch.as_ref().map_or(1.0, Stretcher::speed) * 1_000.0)
            .round() as u32;
        let start_us = reading.origin_us
            + (reading.emitted as f64 * f64::from(speed_milli) / 1_000.0 * MICROS as f64
                / f64::from(self.rate)) as u64;
        let Ok(mut chunk) = self.samples.write_chunk_uninit(frames * 2) else {
            return Step::Blocked;
        };
        let (a, b) = chunk.as_mut_slices();
        let split = a.len();
        for (slot, value) in a.iter_mut().zip(&self.out[..split]) {
            slot.write(*value);
        }
        for (slot, value) in b.iter_mut().zip(&self.out[split..frames * 2]) {
            slot.write(*value);
        }
        // SAFETY: every slot of the chunk was written just above.
        unsafe { chunk.commit_all() };
        self.out.drain(..frames * 2);
        reading.emitted += frames as u64;
        let mark = Mark {
            epoch: self.epoch.unwrap_or(0),
            frames: frames as u32,
            start_us,
            speed_milli,
            end: false,
        };
        let _ = self.marks.push(mark);
        Step::Worked
    }

    fn push_end(&mut self) -> Step {
        if self.marks.slots() < 1 {
            return Step::Blocked;
        }
        let _ = self.marks.push(Mark {
            epoch: self.epoch.unwrap_or(0),
            frames: 0,
            start_us: self.timeline.total_us(),
            speed_milli: 1_000,
            end: true,
        });
        self.finished = true;
        Step::Worked
    }
}

/// Drains the ring into the sound card's buffer.
pub struct Output {
    shared: Arc<Shared>,
    samples: Consumer<f32>,
    marks: Consumer<Mark>,
    rate: u32,
    current: Option<Mark>,
    left: u32,
    played: u64,
    gain: f32,
}

impl Output {
    /// Fills `out` (interleaved, `channels` per frame) with what is next in
    /// the ring, or silence: when paused, when the ring has run dry, and
    /// after the end. Moves the position to what has been played.
    pub fn render(&mut self, out: &mut [f32], channels: usize) {
        let channels = channels.max(1);
        let frames = out.len() / channels;
        let target = f32::from(self.shared.volume_milli.load(Ordering::Relaxed) as u16) / 1_000.0;
        let target = target * target;
        if !self.shared.playing.load(Ordering::Relaxed) {
            out.fill(0.0);
            self.gain = target;
            return;
        }
        let mut written = 0;
        let mut moved = false;
        while written < frames {
            let state = self.shared.state.load(Ordering::Acquire);
            if let Some(mark) = self.current
                && mark.epoch != epoch_of(state)
            {
                self.drop_rest();
            }
            if self.left == 0 {
                let Ok(mark) = self.marks.pop() else { break };
                if mark.epoch != epoch_of(state) {
                    self.skip(mark.frames as usize * 2);
                    continue;
                }
                if mark.end {
                    self.finish(mark.epoch);
                    break;
                }
                self.current = Some(mark);
                self.left = mark.frames;
                self.played = 0;
            }
            let n = (self.left as usize).min(frames - written);
            self.copy(&mut out[written * channels..(written + n) * channels], channels, n);
            self.left -= n as u32;
            self.played += n as u64;
            written += n;
            moved = true;
        }
        out[written * channels..].fill(0.0);
        // A short ramp to the volume asked for, so turning it never clicks.
        let start = self.gain;
        for (i, frame) in out.chunks_exact_mut(channels).take(written).enumerate() {
            let g = start + (target - start) * ((i + 1) as f32 / frames as f32);
            for sample in frame {
                *sample *= g;
            }
        }
        self.gain = target;
        if moved {
            self.advance();
        }
    }

    fn copy(&mut self, out: &mut [f32], channels: usize, frames: usize) {
        let Ok(chunk) = self.samples.read_chunk(frames * 2) else {
            out.fill(0.0);
            return;
        };
        let (a, b) = chunk.as_slices();
        let mut source = a.iter().chain(b.iter());
        for frame in out.chunks_exact_mut(channels) {
            let l = source.next().copied().unwrap_or(0.0);
            let r = source.next().copied().unwrap_or(0.0);
            match channels {
                1 => frame[0] = 0.5 * (l + r),
                _ => {
                    frame[0] = l;
                    frame[1] = r;
                    frame[2..].fill(0.0);
                }
            }
        }
        chunk.commit_all();
    }

    fn skip(&mut self, samples: usize) {
        if samples > 0
            && let Ok(chunk) = self.samples.read_chunk(samples)
        {
            chunk.commit_all();
        }
    }

    fn drop_rest(&mut self) {
        let left = self.left as usize * 2;
        self.left = 0;
        self.current = None;
        self.skip(left);
    }

    fn finish(&mut self, epoch: u64) {
        self.current = None;
        let length = self.shared.length_us;
        let mut current = self.shared.state.load(Ordering::Acquire);
        while epoch_of(current) == epoch {
            match self.shared.state.compare_exchange_weak(
                current,
                pack(epoch, length),
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    self.shared.ended.store(true, Ordering::Relaxed);
                    self.shared.playing.store(false, Ordering::Relaxed);
                    return;
                }
                Err(seen) => current = seen,
            }
        }
    }

    /// Sets the position to the end of what was played, unless a seek got
    /// there first.
    fn advance(&mut self) {
        let Some(mark) = self.current else { return };
        let into = self.played as f64 * f64::from(mark.speed_milli) / 1_000.0 * MICROS as f64
            / f64::from(self.rate);
        let at = (mark.start_us + into as u64).min(self.shared.length_us);
        let mut current = self.shared.state.load(Ordering::Acquire);
        while epoch_of(current) == mark.epoch {
            match self.shared.state.compare_exchange_weak(
                current,
                pack(mark.epoch, at),
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return,
                Err(seen) => current = seen,
            }
        }
    }
}

/// A player's three halves, built for a card of `rate` Hz.
pub struct Engine {
    pub control: Control,
    pub decoder: Decoder,
    pub output: Output,
}

impl Engine {
    pub fn new(shared: Arc<Shared>, parts: Vec<PartInput>, timeline: Timeline, rate: u32) -> Self {
        let (samples_in, samples_out) = RingBuffer::<f32>::new(ring_samples(rate));
        let (marks_in, marks_out) = RingBuffer::<Mark>::new(MARK_SLOTS);
        let gain = f32::from(shared.volume_milli.load(Ordering::Relaxed) as u16) / 1_000.0;
        Engine {
            control: Control::new(shared.clone()),
            decoder: Decoder {
                shared: shared.clone(),
                parts,
                timeline,
                rate,
                samples: samples_in,
                marks: marks_in,
                epoch: None,
                reading: None,
                out: Vec::new(),
                scratch: Vec::new(),
                stereo: Vec::new(),
                resampled: Vec::new(),
                finished: false,
            },
            output: Output {
                shared,
                samples: samples_out,
                marks: marks_out,
                rate,
                current: None,
                left: 0,
                played: 0,
                gain: gain * gain,
            },
        }
    }

    /// A player with no sound card, for tests: `parts` measured and ready to
    /// be stepped and rendered by hand.
    pub fn headless(parts: Vec<PartInput>, rate: u32) -> Result<Self, AudioError> {
        let timeline = measure(&parts)?;
        let shared = Shared::new(timeline.total_us());
        Ok(Engine::new(shared, parts, timeline, rate))
    }

    /// Steps the decoder until it has nothing more to do now.
    pub fn pump(&mut self) {
        while self.decoder.step() == Step::Worked {}
    }

    /// Pumps, then renders `frames` of stereo.
    pub fn render(&mut self, frames: usize) -> Vec<f32> {
        self.pump();
        let mut out = vec![0.0; frames * 2];
        self.output.render(&mut out, 2);
        out
    }
}
