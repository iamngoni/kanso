//! The microphone recorder: 48 kHz capture into the parts on disk.
//!
//! [`Capture::start`] opens the default microphone, creates the session
//! folder and runs a thread that moves sound from the device to the
//! [`Recorder`] every 20 ms. [`Pipeline`] is that thread's work without the
//! device, so it is tested with synthetic sound.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::mpsc::sync_channel;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::error::AudioError;
use crate::mic::{self, Failure, Microphone};
use crate::mix::MixPipeline;
use crate::model::{Channels, SAMPLE_RATE};
use crate::record_model::{Finished, Progress, RecordSource};
use crate::resample::To48k;
use crate::session::{Recorder, RecorderOptions, remove_session};
use crate::system_apps::GapFiller;
use crate::system_audio::{SystemAudioSource, SystemAudioTarget, platform_backend};

/// How often the thread collects what the microphone recorded.
const POLL: Duration = Duration::from_millis(20);
/// The stretch of sound the level is measured over: 50 ms.
const LEVEL_WINDOW: usize = SAMPLE_RATE as usize / 20;
/// A microphone that sends nothing for this long, while not paused, is gone:
/// some systems stop the stream on unplug without reporting an error.
const SILENT_DEVICE: Duration = Duration::from_secs(3);

/// The check for a microphone that went quiet without saying so. It looks at
/// microphones only: a system source (WASAPI loopback sends nothing while no
/// program plays) is silent for good reasons and is kept on the clock by
/// padding instead.
#[derive(Clone, Copy, Debug, Default)]
pub struct MicWatch {
    silent: Duration,
}

impl MicWatch {
    /// A poll `dt` after the last: whether anything `delivered` sound, and
    /// whether the recording is `paused`. True once the microphone has sent
    /// nothing for three seconds of recording.
    pub fn tick(&mut self, dt: Duration, delivered: bool, paused: bool) -> bool {
        if paused || delivered {
            self.silent = Duration::ZERO;
        } else {
            self.silent += dt;
        }
        self.silent >= SILENT_DEVICE
    }
}

/// Mono sound at the device's rate in, a recording on disk out: resample to
/// 48 kHz, drop the audio while paused, hand the rest to the recorder.
pub struct Pipeline {
    resampler: To48k,
    recorder: Recorder,
    paused: bool,
    scratch: Vec<f32>,
    rms: f32,
}

impl Pipeline {
    /// `recorder` must be mono.
    pub fn new(recorder: Recorder, device_rate: u32) -> Result<Self, AudioError> {
        if recorder.session().channels != 1 {
            return Err(AudioError::Config(
                "the microphone alone records one channel".into(),
            ));
        }
        Ok(Pipeline {
            resampler: To48k::new(device_rate)?,
            recorder,
            paused: false,
            scratch: Vec::new(),
            rms: 0.0,
        })
    }

    /// While paused, sound is dropped, so the recording's clock stops and
    /// timestamps have no gap for the pause.
    pub fn set_paused(&mut self, paused: bool) -> Result<(), AudioError> {
        if paused && !self.paused {
            self.recorder.checkpoint()?;
        }
        self.paused = paused;
        Ok(())
    }

    pub fn recorder(&self) -> &Recorder {
        &self.recorder
    }

    /// The root mean square of the last 50 ms taken; for the level meter.
    pub fn rms(&self) -> f32 {
        self.rms
    }

    /// Takes mono sound at the device's rate.
    pub fn feed(&mut self, mono: &[f32]) -> Result<Progress, AudioError> {
        if self.paused {
            return Ok(Progress {
                samples: self.recorder.samples(),
                ..Progress::default()
            });
        }
        self.scratch.clear();
        self.resampler.push(mono, &mut self.scratch);
        self.take()
    }

    fn take(&mut self) -> Result<Progress, AudioError> {
        let tail = self.scratch.len().saturating_sub(LEVEL_WINDOW);
        let window = &self.scratch[tail..];
        if !window.is_empty() {
            let sum: f64 = window.iter().map(|&s| f64::from(s) * f64::from(s)).sum();
            self.rms = (sum / window.len() as f64).sqrt() as f32;
        }
        self.recorder.push(&self.scratch)
    }

    /// Flushes the resampler and closes the recording.
    pub fn finish(mut self) -> Result<Finished, AudioError> {
        if !self.paused {
            self.scratch.clear();
            self.resampler.finish(&mut self.scratch);
            self.recorder.push(&self.scratch)?;
        }
        self.recorder.finish()
    }

    pub fn discard(self) -> std::io::Result<()> {
        self.recorder.discard()
    }
}

/// Why a recording ended.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StopReason {
    /// The user stopped it.
    Requested,
    /// It reached the cap.
    Cap,
    /// The microphone failed (unplugged, or the system took it).
    Device(String),
    /// The disk refused a write (full); what was written is kept.
    Disk(String),
}

/// What the recording bar shows.
#[derive(Clone, Debug, PartialEq)]
pub struct CaptureStatus {
    /// Samples per channel at 48 kHz written: the timer, pause excluded.
    pub samples: u64,
    /// The root mean square of the last 50 ms; map it with the level curve.
    pub rms: f32,
    pub paused: bool,
    /// The warning threshold has been crossed.
    pub warned: bool,
    /// The recorder has stopped by itself; call [`Capture::stop`] for the
    /// result.
    pub ended: Option<StopReason>,
    /// Why the system audio of a recording with both sources stopped; the
    /// microphone records on and the meeting's channel is silent.
    pub system_failure: Option<String>,
}

impl CaptureStatus {
    pub fn seconds(&self) -> f64 {
        self.samples as f64 / f64::from(SAMPLE_RATE)
    }
}

/// What a stopped recording gives back.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stopped {
    pub finished: Finished,
    pub reason: StopReason,
}

#[derive(Default)]
struct Shared {
    stop: AtomicBool,
    discard: AtomicBool,
    paused: AtomicBool,
    warned: AtomicBool,
    samples: AtomicU64,
    rms: AtomicU32,
    ended: Mutex<Option<StopReason>>,
    system_failure: Mutex<Option<String>>,
}

/// What a capture listens to.
enum Kind {
    Microphone,
    System(SystemAudioTarget),
    /// Both, on one stereo timeline: you on the left, the meeting on the
    /// right.
    Both(SystemAudioTarget),
}

/// Where a capture's sound comes from while it runs.
enum Feed {
    Mic(Microphone, Failure),
    System(Box<dyn SystemAudioSource>),
}

impl Feed {
    fn rate(&self) -> u32 {
        match self {
            Feed::Mic(mic, _) => mic.rate,
            Feed::System(source) => source.sample_rate(),
        }
    }

    /// Whether silence from this feed means the device is gone: true of a
    /// microphone only. Loopback sends nothing while nothing plays.
    fn watched(&self) -> bool {
        matches!(self, Feed::Mic(..))
    }

    /// Appends the sound that arrived since the last call to `out`.
    fn read(&mut self, out: &mut Vec<f32>) {
        match self {
            Feed::Mic(mic, _) => {
                let available = mic.samples.slots();
                if available > 0
                    && let Ok(read) = mic.samples.read_chunk(available)
                {
                    let (a, b) = read.as_slices();
                    out.extend_from_slice(a);
                    out.extend_from_slice(b);
                    read.commit_all();
                }
            }
            Feed::System(source) => source.read(out),
        }
    }

    fn take_dropped(&mut self) -> u64 {
        match self {
            Feed::Mic(mic, _) => mic.dropped.swap(0, Ordering::Relaxed),
            Feed::System(source) => source.take_dropped(),
        }
    }

    fn failure(&self) -> Option<String> {
        match self {
            Feed::Mic(_, failure) => failure.lock().ok().and_then(|f| f.clone()),
            Feed::System(source) => source.failure(),
        }
    }
}

/// A recording in progress, from the microphone or from system audio.
/// Dropping it without calling [`stop`](Self::stop) or [`discard`](Self::discard) stops it and keeps the
/// session folder, which is then found as recoverable.
pub struct Capture {
    shared: Arc<Shared>,
    dir: PathBuf,
    thread: Option<JoinHandle<Result<Option<Stopped>, AudioError>>>,
}

impl Capture {
    /// Opens the microphone and starts recording into a new session folder
    /// under `root`. The microphone's errors (denied, none) come back here.
    pub fn start(root: &Path, mut options: RecorderOptions) -> Result<Capture, AudioError> {
        if !options.sources.contains(&RecordSource::Microphone) {
            options.sources.push(RecordSource::Microphone);
        }
        Self::launch(root, options, Kind::Microphone)
    }

    /// Records system audio (one app's on a Mac, or what the computer plays)
    /// as a mono track of its own, into a new session folder under `root`,
    /// through this platform's [`platform_backend`]. Whether Kanso may
    /// record it, and whether the app is running, come back here.
    pub fn start_system(
        root: &Path,
        mut options: RecorderOptions,
        target: SystemAudioTarget,
    ) -> Result<Capture, AudioError> {
        options.sources = vec![RecordSource::SystemAudio];
        Self::launch(root, options, Kind::System(target))
    }

    /// Records the microphone and system audio together as one stereo
    /// recording (left you, right the meeting), their clocks kept together
    /// by the [`Mixer`](crate::mix::Mixer). The microphone's errors and the
    /// system's both come back here; if the system audio stops later, the
    /// microphone goes on ([`CaptureStatus::system_failure`]).
    pub fn start_both(
        root: &Path,
        mut options: RecorderOptions,
        target: SystemAudioTarget,
    ) -> Result<Capture, AudioError> {
        options.sources = vec![RecordSource::Microphone, RecordSource::SystemAudio];
        Self::launch(root, options, Kind::Both(target))
    }

    fn launch(
        root: &Path,
        mut options: RecorderOptions,
        kind: Kind,
    ) -> Result<Capture, AudioError> {
        let channels = match kind {
            Kind::Both(_) => Channels::Stereo,
            _ => Channels::Mono,
        };
        options.encode.channels = channels;
        options.encode.bitrate = channels.default_bitrate();
        let shared = Arc::new(Shared::default());
        let (ready_tx, ready_rx) = sync_channel::<Result<PathBuf, AudioError>>(1);
        let root = root.to_path_buf();
        let thread_shared = shared.clone();
        let thread = thread::Builder::new()
            .name("kanso-recorder".into())
            .spawn(move || run(&root, options, kind, &thread_shared, &ready_tx))
            .map_err(AudioError::Io)?;
        let dir = ready_rx
            .recv()
            .map_err(|_| AudioError::Device("the recorder stopped before it started".into()))??;
        Ok(Capture {
            shared,
            dir,
            thread: Some(thread),
        })
    }

    /// The session folder.
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn status(&self) -> CaptureStatus {
        CaptureStatus {
            samples: self.shared.samples.load(Ordering::Relaxed),
            rms: f32::from_bits(self.shared.rms.load(Ordering::Relaxed)),
            paused: self.shared.paused.load(Ordering::Relaxed),
            warned: self.shared.warned.load(Ordering::Relaxed),
            ended: self
                .shared
                .ended
                .lock()
                .map_or(None, |reason| reason.clone()),
            system_failure: self
                .shared
                .system_failure
                .lock()
                .map_or(None, |why| why.clone()),
        }
    }

    pub fn pause(&self) {
        self.shared.paused.store(true, Ordering::Relaxed);
    }

    pub fn resume(&self) {
        self.shared.paused.store(false, Ordering::Relaxed);
    }

    /// Stops, closes the recording and returns its parts. The folder stays
    /// until the caller has attached them (see
    /// [`remove_session`](crate::session::remove_session)).
    pub fn stop(mut self) -> Result<Stopped, AudioError> {
        self.shared.stop.store(true, Ordering::Relaxed);
        self.join()
    }

    /// Stops and deletes the recording.
    pub fn discard(mut self) -> Result<(), AudioError> {
        self.shared.discard.store(true, Ordering::Relaxed);
        self.shared.stop.store(true, Ordering::Relaxed);
        let dir = self.dir.clone();
        let result = self.join();
        remove_session(&dir)?;
        match result {
            Ok(_) | Err(AudioError::Config(_)) => Ok(()),
            Err(e) => Err(e),
        }
    }

    fn join(&mut self) -> Result<Stopped, AudioError> {
        let thread = self
            .thread
            .take()
            .ok_or_else(|| AudioError::Config("the recording is finished".into()))?;
        thread
            .join()
            .map_err(|_| AudioError::Device("the recorder thread panicked".into()))??
            .ok_or_else(|| AudioError::Config("the recording was discarded".into()))
    }
}

impl Drop for Capture {
    fn drop(&mut self) {
        if self.thread.is_some() {
            self.shared.stop.store(true, Ordering::Relaxed);
            let _ = self.join();
        }
    }
}

/// Finishes a session folder from what is on disk: each part cut back to
/// its last whole page.
fn recover_written(dir: &Path) -> Result<Finished, AudioError> {
    let found = crate::recovery::find(dir).ok_or(AudioError::NoSound)?;
    crate::recovery::recover(&found)
}

/// What `run` and `run_both` close at the end: a mono or a stereo pipeline.
trait Writes {
    fn close(self) -> Result<Finished, AudioError>;
    fn delete(self) -> std::io::Result<()>;
}

impl Writes for Pipeline {
    fn close(self) -> Result<Finished, AudioError> {
        self.finish()
    }
    fn delete(self) -> std::io::Result<()> {
        self.discard()
    }
}

impl Writes for MixPipeline {
    fn close(self) -> Result<Finished, AudioError> {
        self.finish()
    }
    fn delete(self) -> std::io::Result<()> {
        self.discard()
    }
}

/// Maps a failure to start onto what the caller is told.
fn start_error(e: &AudioError) -> AudioError {
    match e {
        AudioError::MicrophoneDenied => AudioError::MicrophoneDenied,
        AudioError::NoMicrophone => AudioError::NoMicrophone,
        AudioError::ScreenRecordingDenied => AudioError::ScreenRecordingDenied,
        AudioError::SystemAudio(why) => AudioError::SystemAudio(why.clone()),
        other => AudioError::Device(other.to_string()),
    }
}

/// Ends the recorder thread: says why, drops the devices' work, and closes
/// or deletes the recording.
fn conclude<P: Writes>(
    pipeline: P,
    dir: &Path,
    shared: &Shared,
    mut reason: StopReason,
) -> Result<Option<Stopped>, AudioError> {
    if let Ok(mut ended) = shared.ended.lock()
        && reason != StopReason::Requested
    {
        *ended = Some(reason.clone());
    }
    if shared.discard.load(Ordering::Relaxed) {
        pipeline.delete()?;
        return Ok(None);
    }
    // After a refused write the open part may end in a torn page, and
    // closing it would put good pages after the tear. A full (or failing)
    // disk may refuse the closing pages and the sidecar too. Either way,
    // what reached the disk is finished the way a crash is recovered.
    let finished = if matches!(reason, StopReason::Disk(_)) {
        drop(pipeline);
        recover_written(dir)?
    } else {
        match pipeline.close() {
            Ok(finished) => finished,
            Err(e) => {
                reason = StopReason::Disk(e.to_string());
                recover_written(dir).map_err(|_| e)?
            }
        }
    };
    Ok(Some(Stopped { finished, reason }))
}

/// The recorder thread.
// no-prop-test: needs a microphone; `Pipeline`, the resampler and the recorder it drives have the properties.
fn run(
    root: &Path,
    options: RecorderOptions,
    kind: Kind,
    shared: &Shared,
    ready: &std::sync::mpsc::SyncSender<Result<PathBuf, AudioError>>,
) -> Result<Option<Stopped>, AudioError> {
    let failure: Failure = Arc::new(Mutex::new(None));
    let target = match &kind {
        Kind::Both(target) => {
            return run_both(root, options, target, shared, ready);
        }
        Kind::Microphone => None,
        Kind::System(target) => Some(target.clone()),
    };
    let started = (|| {
        let feed = match &target {
            None => Feed::Mic(mic::open(failure.clone())?, failure.clone()),
            Some(target) => {
                let backend = platform_backend().ok_or_else(|| {
                    AudioError::SystemAudio("this system can't record other sound yet".into())
                })?;
                Feed::System(backend.open(target)?)
            }
        };
        let recorder = Recorder::create(root, options)?;
        let dir = recorder.dir().to_path_buf();
        let pipeline = Pipeline::new(recorder, feed.rate())?;
        Ok::<_, AudioError>((feed, pipeline, dir))
    })();
    let (mut feed, mut pipeline, dir) = match started {
        Ok(started) => started,
        Err(e) => {
            let _ = ready.send(Err(start_error(&e)));
            return Err(e);
        }
    };
    let _ = ready.send(Ok(dir.clone()));

    let mut reason = StopReason::Requested;
    let mut chunk: Vec<f32> = Vec::new();
    let mut watch = MicWatch::default();
    let mut filler = GapFiller::new();
    let mut last_tick = Instant::now();
    loop {
        thread::sleep(POLL);
        let now = Instant::now();
        let dt = now.duration_since(last_tick);
        last_tick = now;
        let stopping = shared.stop.load(Ordering::Relaxed);
        let was_paused = shared.paused.load(Ordering::Relaxed);
        if let Err(e) = pipeline.set_paused(was_paused) {
            reason = StopReason::Disk(e.to_string());
            break;
        }
        chunk.clear();
        feed.read(&mut chunk);
        // What the full queue dropped while the disk stalled comes back as
        // silence, so later timestamps still match the clock.
        let dropped = feed.take_dropped();
        if !was_paused {
            chunk.resize(chunk.len() + dropped as usize, 0.0);
        }
        if feed.watched() {
            if watch.tick(dt, !chunk.is_empty(), was_paused) {
                reason = StopReason::Device("the microphone stopped sending sound".into());
                break;
            }
        } else {
            // An app that isn't playing sends nothing; that is not a dead
            // device, and the track must stay as long as the microphone's.
            filler.advance(dt, was_paused);
            if !was_paused {
                pad_to_clock(
                    &filler,
                    &mut chunk,
                    feed.rate(),
                    pipeline.recorder().samples(),
                );
            }
        }
        match pipeline.feed(&chunk) {
            Ok(progress) => {
                shared.samples.store(progress.samples, Ordering::Relaxed);
                shared
                    .rms
                    .store(pipeline.rms().to_bits(), Ordering::Relaxed);
                if progress.warned {
                    shared.warned.store(true, Ordering::Relaxed);
                }
                if progress.capped {
                    reason = StopReason::Cap;
                    break;
                }
            }
            Err(e) => {
                reason = StopReason::Disk(e.to_string());
                break;
            }
        }
        if let Some(message) = feed.failure() {
            reason = StopReason::Device(message);
            break;
        }
        if stopping {
            break;
        }
    }
    drop(feed);
    conclude(pipeline, &dir, shared, reason)
}

/// Adds the silence a source owes the wall clock to `chunk` (at the
/// source's `rate`), given the `written` samples at 48 kHz so far.
fn pad_to_clock(filler: &GapFiller, chunk: &mut Vec<f32>, rate: u32, written: u64) {
    let owed = owed_silence(filler, chunk.len(), rate, written);
    chunk.resize(chunk.len() + owed, 0.0);
}

/// The silence, in samples at the source's `rate`, that a source owes the
/// wall clock when `chunk_len` new samples arrive after `written` samples at
/// 48 kHz. Pure.
fn owed_silence(filler: &GapFiller, chunk_len: usize, rate: u32, written: u64) -> usize {
    // Everything is counted at 48 kHz; the chunk is at the device's rate
    // (48 kHz too, unless the system says otherwise).
    let rate = u64::from(rate);
    let coming = chunk_len as u64 * u64::from(SAMPLE_RATE) / rate;
    let owed = filler.owed(written + coming) as u64;
    (owed * rate / u64::from(SAMPLE_RATE)) as usize
}

/// Puts the silence the meeting owes the clock *before* the sound that just
/// arrived: the gap happened before it. On the first poll this is the time
/// the system audio took to start while the microphone was already
/// recording, so the two start together instead of the microphone running
/// ahead and being cut back a frame a second.
fn lead_with_silence(filler: &GapFiller, chunk: &mut Vec<f32>, rate: u32, written: u64) {
    let owed = owed_silence(filler, chunk.len(), rate, written);
    if owed > 0 {
        chunk.splice(0..0, std::iter::repeat_n(0.0, owed));
    }
}

/// The recorder thread of a recording with both sources: the microphone
/// (checked for going quiet) and system audio (padded to the clock when it
/// sends nothing, and given up, with the microphone going on, if it fails),
/// mixed onto one stereo timeline.
// no-prop-test: needs a microphone; `Mixer`, `MicWatch` and the pipeline have the properties.
fn run_both(
    root: &Path,
    options: RecorderOptions,
    target: &SystemAudioTarget,
    shared: &Shared,
    ready: &std::sync::mpsc::SyncSender<Result<PathBuf, AudioError>>,
) -> Result<Option<Stopped>, AudioError> {
    let failure: Failure = Arc::new(Mutex::new(None));
    let started = (|| {
        let mic = Feed::Mic(mic::open(failure.clone())?, failure.clone());
        // The microphone records from here; the clock starts with it.
        let opened = Instant::now();
        let backend = platform_backend().ok_or_else(|| {
            AudioError::SystemAudio("this system can't record other sound yet".into())
        })?;
        let system = Feed::System(backend.open(target)?);
        let recorder = Recorder::create(root, options)?;
        let dir = recorder.dir().to_path_buf();
        let pipeline = MixPipeline::new(recorder, mic.rate(), system.rate())?;
        Ok::<_, AudioError>((mic, system, pipeline, dir, opened))
    })();
    let (mut mic, system, mut pipeline, dir, opened) = match started {
        Ok(started) => started,
        Err(e) => {
            let _ = ready.send(Err(start_error(&e)));
            return Err(e);
        }
    };
    let _ = ready.send(Ok(dir.clone()));

    let system_rate = system.rate();
    let mut system = Some(system);
    let mut reason = StopReason::Requested;
    let mut heard: Vec<f32> = Vec::new();
    let mut meeting: Vec<f32> = Vec::new();
    let mut watch = MicWatch::default();
    let mut filler = GapFiller::new();
    // The first poll counts from when the microphone opened, so the time the
    // system audio took to start is owed to the meeting as silence.
    let mut last_tick = opened;
    loop {
        thread::sleep(POLL);
        let now = Instant::now();
        let dt = now.duration_since(last_tick);
        last_tick = now;
        let stopping = shared.stop.load(Ordering::Relaxed);
        let was_paused = shared.paused.load(Ordering::Relaxed);
        if let Err(e) = pipeline.set_paused(was_paused) {
            reason = StopReason::Disk(e.to_string());
            break;
        }
        heard.clear();
        mic.read(&mut heard);
        let dropped = mic.take_dropped();
        if !was_paused {
            heard.resize(heard.len() + dropped as usize, 0.0);
        }
        // Only the microphone is checked for going quiet.
        if watch.tick(dt, !heard.is_empty(), was_paused) {
            reason = StopReason::Device("the microphone stopped sending sound".into());
            break;
        }
        meeting.clear();
        if let Some(source) = &mut system {
            source.read(&mut meeting);
            let dropped = source.take_dropped();
            if !was_paused {
                meeting.resize(meeting.len() + dropped as usize, 0.0);
            }
        }
        // Whatever the system sends or doesn't, its channel stays on the
        // clock, silent when it has nothing (or has failed).
        filler.advance(dt, was_paused);
        if !was_paused {
            lead_with_silence(&filler, &mut meeting, system_rate, pipeline.given_meeting());
        }
        pipeline.feed_you(&heard);
        pipeline.feed_meeting(&meeting);
        match pipeline.flush() {
            Ok(progress) => {
                shared.samples.store(progress.samples, Ordering::Relaxed);
                shared
                    .rms
                    .store(pipeline.rms().to_bits(), Ordering::Relaxed);
                if progress.warned {
                    shared.warned.store(true, Ordering::Relaxed);
                }
                if progress.capped {
                    reason = StopReason::Cap;
                    break;
                }
            }
            Err(e) => {
                reason = StopReason::Disk(e.to_string());
                break;
            }
        }
        if let Some(message) = mic.failure() {
            reason = StopReason::Device(message);
            break;
        }
        if let Some(message) = system.as_ref().and_then(Feed::failure) {
            if let Ok(mut why) = shared.system_failure.lock() {
                *why = Some(message);
            }
            system = None;
        }
        if stopping {
            break;
        }
    }
    drop(mic);
    drop(system);
    conclude(pipeline, &dir, shared, reason)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::{FileFailurePersistence, RngSeed};

    /// A system source that never sends a sample, as WASAPI loopback does
    /// while no program plays.
    struct Quiet;

    impl SystemAudioSource for Quiet {
        fn sample_rate(&self) -> u32 {
            48_000
        }
        fn read(&mut self, _out: &mut Vec<f32>) {}
        fn take_dropped(&mut self) -> u64 {
            0
        }
        fn failure(&self) -> Option<String> {
            None
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 64,
            rng_seed: RngSeed::Fixed(0x006b_616e_736f),
            failure_persistence: Some(Box::new(FileFailurePersistence::Off)),
            ..ProptestConfig::default()
        })]

        /// A system source is never checked for going quiet, however long
        /// it sends nothing; the microphone check is the only one.
        #[test]
        fn prop_system_sources_are_never_declared_dead(
            ticks in proptest::collection::vec((1u64..2_000, any::<bool>()), 0..200),
        ) {
            let feed = Feed::System(Box::new(Quiet));
            prop_assert!(!feed.watched());
            let mut watch = MicWatch::default();
            for (ms, paused) in ticks {
                // The loop asks the watch only of a watched feed.
                let dead = feed.watched() && watch.tick(Duration::from_millis(ms), false, paused);
                prop_assert!(!dead);
            }
        }

        /// A microphone is declared dead once it has sent nothing for three
        /// seconds of recording, and never while it sends or is paused.
        #[test]
        fn prop_mic_watch_trips_after_three_quiet_seconds(
            ticks in proptest::collection::vec((1u64..500, any::<bool>(), any::<bool>()), 0..200),
        ) {
            let mut watch = MicWatch::default();
            let mut quiet = Duration::ZERO;
            for (ms, delivered, paused) in ticks {
                let dt = Duration::from_millis(ms);
                if paused || delivered {
                    quiet = Duration::ZERO;
                } else {
                    quiet += dt;
                }
                prop_assert_eq!(watch.tick(dt, delivered, paused), quiet >= SILENT_DEVICE);
            }
        }

        /// System audio that takes `startup` to begin while the microphone
        /// already records is placed where it happened: its first sound lands
        /// within the clock's tolerance of `startup` into the recording, and
        /// the two sources then agree, so nothing of yours is cut back.
        #[test]
        fn prop_a_slow_start_is_owed_to_the_meeting_as_silence(
            startup_ms in 0u64..3_000,
            seconds in 2u64..6,
        ) {
            use crate::mix::Mixer;
            let tick = Duration::from_millis(20);
            let per_tick = 960usize;
            let startup = Duration::from_millis(startup_ms);
            let mut filler = GapFiller::new();
            let mut mixer = Mixer::new();
            let mut out = Vec::new();
            let mut given_meeting = 0u64;
            // The first poll: the microphone's backlog since it opened, and
            // the meeting's first 20 ms.
            let first = startup + tick;
            let mut you = vec![0.0f32; (first.as_secs_f64() * 48_000.0).round() as usize];
            let mut meeting = vec![0.5f32; per_tick];
            for n in 0..=seconds * 50 {
                let dt = if n == 0 { first } else { tick };
                if n > 0 {
                    you = vec![0.0; per_tick];
                    meeting = vec![0.5; per_tick];
                }
                filler.advance(dt, false);
                lead_with_silence(&filler, &mut meeting, 48_000, given_meeting);
                given_meeting += meeting.len() as u64;
                mixer.push_you(&you);
                mixer.push_meeting(&meeting);
                mixer.drain(&mut out);
            }
            let tolerance = crate::system_apps::GAP_TOLERANCE.as_millis() as usize * 48;
            prop_assert_eq!(mixer.corrections(), 0);
            prop_assert!(mixer.skew().unsigned_abs() as usize <= tolerance);
            let heard = out.iter().skip(1).step_by(2).position(|&s| s != 0.0).unwrap();
            let want = (startup.as_secs_f64() * 48_000.0) as usize;
            prop_assert!(heard.abs_diff(want) <= tolerance, "heard at {heard}, want {want}");
        }
    }
}
