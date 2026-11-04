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
use std::time::Duration;

use crate::error::AudioError;
use crate::mic::{self, Failure};
use crate::model::{Channels, SAMPLE_RATE};
use crate::record_model::{Finished, Progress, RecordSource};
use crate::resample::To48k;
use crate::session::{Recorder, RecorderOptions, remove_session};

/// How often the thread collects what the microphone recorded.
const POLL: Duration = Duration::from_millis(20);
/// The stretch of sound the level is measured over: 50 ms.
const LEVEL_WINDOW: usize = SAMPLE_RATE as usize / 20;

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
}

/// A microphone recording in progress. Dropping it without calling
/// [`stop`](Self::stop) or [`discard`](Self::discard) stops it and keeps the
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
        options.encode.channels = Channels::Mono;
        options.encode.bitrate = Channels::Mono.default_bitrate();
        if !options.sources.contains(&RecordSource::Microphone) {
            options.sources.push(RecordSource::Microphone);
        }
        let shared = Arc::new(Shared::default());
        let (ready_tx, ready_rx) = sync_channel::<Result<PathBuf, AudioError>>(1);
        let root = root.to_path_buf();
        let thread_shared = shared.clone();
        let thread = thread::Builder::new()
            .name("kanso-recorder".into())
            .spawn(move || run(&root, options, &thread_shared, &ready_tx))
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

/// The recorder thread.
// no-prop-test: needs a microphone; `Pipeline`, the resampler and the recorder it drives have the properties.
fn run(
    root: &Path,
    options: RecorderOptions,
    shared: &Shared,
    ready: &std::sync::mpsc::SyncSender<Result<PathBuf, AudioError>>,
) -> Result<Option<Stopped>, AudioError> {
    let failure: Failure = Arc::new(Mutex::new(None));
    let started = (|| {
        let mic = mic::open(failure.clone())?;
        let recorder = Recorder::create(root, options)?;
        let dir = recorder.dir().to_path_buf();
        let pipeline = Pipeline::new(recorder, mic.rate)?;
        Ok::<_, AudioError>((mic, pipeline, dir))
    })();
    let (mut mic, mut pipeline, dir) = match started {
        Ok(started) => started,
        Err(e) => {
            let _ = ready.send(Err(match &e {
                AudioError::MicrophoneDenied => AudioError::MicrophoneDenied,
                AudioError::NoMicrophone => AudioError::NoMicrophone,
                other => AudioError::Device(other.to_string()),
            }));
            return Err(e);
        }
    };
    let _ = ready.send(Ok(dir));

    let mut reason = StopReason::Requested;
    let mut chunk: Vec<f32> = Vec::new();
    loop {
        thread::sleep(POLL);
        let stopping = shared.stop.load(Ordering::Relaxed);
        let was_paused = shared.paused.load(Ordering::Relaxed);
        if let Err(e) = pipeline.set_paused(was_paused) {
            reason = StopReason::Disk(e.to_string());
            break;
        }
        chunk.clear();
        let available = mic.samples.slots();
        if available > 0
            && let Ok(read) = mic.samples.read_chunk(available)
        {
            let (a, b) = read.as_slices();
            chunk.extend_from_slice(a);
            chunk.extend_from_slice(b);
            read.commit_all();
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
        if let Some(message) = failure.lock().ok().and_then(|f| f.clone()) {
            reason = StopReason::Device(message);
            break;
        }
        if stopping {
            break;
        }
    }
    if let Ok(mut ended) = shared.ended.lock()
        && reason != StopReason::Requested
    {
        *ended = Some(reason.clone());
    }
    drop(mic);
    if shared.discard.load(Ordering::Relaxed) {
        pipeline.discard()?;
        return Ok(None);
    }
    let finished = pipeline.finish()?;
    Ok(Some(Stopped { finished, reason }))
}
