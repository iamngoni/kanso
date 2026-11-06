//! A recording on disk while it is made: the private in-progress folder, the
//! part files, the `session.json` sidecar, and the cap.
//!
//! [`Recorder`] takes 48 kHz samples (from the microphone through
//! [`crate::capture`], or from anything else) and writes them through
//! [`PartWriter`]. Every file it creates is flushed and its folder is synced,
//! so a power cut leaves the parts that were written, and the sidecar says
//! what they are.

use std::fs::{self, DirBuilder, File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::encode::{PartSink, PartWriter};
use crate::error::AudioError;
use crate::model::{Channels, EncodeConfig, PartInfo, SAMPLE_RATE};
use crate::record_model::{
    Finished, Limits, Progress, RecordSource, SessionFile, SessionPart, SessionState,
};

/// The sidecar's file name.
pub const SESSION_FILE: &str = "session.json";
/// The sidecar is rewritten every this many samples per channel: 5 seconds.
pub const CHECKPOINT_SAMPLES: u64 = 5 * SAMPLE_RATE as u64;

/// `<data dir>/recordings/in-progress`: where unfinished recordings live.
pub fn in_progress_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("recordings").join("in-progress")
}

/// The file name of part `index`.
pub fn part_file_name(index: u32) -> String {
    format!("part-{index:04}.opus")
}

/// The part index in a file name made by [`part_file_name`].
pub fn parse_part_name(name: &str) -> Option<u32> {
    let digits = name.strip_prefix("part-")?.strip_suffix(".opus")?;
    if digits.len() < 4 || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

/// A session id is a folder name and nothing else: letters, digits and `-`.
pub fn valid_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 64 && id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
}

/// A fresh session id: the start time and 32 random bits.
pub fn new_id() -> String {
    format!("{:x}-{:08x}", now_ms() / 1000, fastrand::u32(..))
}

pub fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as u64)
}

/// Creates `path` and any missing parents readable by this user only
/// (`0700`; on Windows the folder inherits the profile's access list).
pub fn create_private_dir(path: &Path) -> io::Result<()> {
    let mut builder = DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(path)
}

/// Syncs a folder, so the files created in it survive a power cut. A no-op
/// where folders can't be opened (Windows).
pub fn sync_dir(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        File::open(path)?.sync_all()
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        Ok(())
    }
}

/// Writes `bytes` to `path` so that either the old or the new content
/// survives a crash: a temp file, synced, renamed over, and the folder synced.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    use io::Write as _;
    let tmp = path.with_extension("tmp");
    let mut file = private_file(&tmp, true)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    drop(file);
    fs::rename(&tmp, path)?;
    if let Some(dir) = path.parent() {
        sync_dir(dir)?;
    }
    Ok(())
}

/// Creates (or, when `overwrite`, truncates) a file readable by this user
/// only.
fn private_file(path: &Path, overwrite: bool) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.write(true);
    if overwrite {
        options.create(true).truncate(true);
    } else {
        options.create_new(true);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path)
}

/// Opens the part files of one session folder. Each new file is created
/// exclusively, and the folder is synced so the file is there after a power
/// cut.
pub struct DirSink {
    dir: PathBuf,
    syncs: Arc<AtomicUsize>,
}

impl DirSink {
    pub fn new(dir: PathBuf) -> Self {
        DirSink {
            dir,
            syncs: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// How many times the folder has been synced after creating a part.
    pub fn dir_syncs(&self) -> Arc<AtomicUsize> {
        self.syncs.clone()
    }
}

impl PartSink for DirSink {
    type Out = File;

    fn open(&mut self, index: u32) -> io::Result<File> {
        let file = private_file(&self.dir.join(part_file_name(index)), false)?;
        sync_dir(&self.dir)?;
        self.syncs.fetch_add(1, Ordering::Relaxed);
        Ok(file)
    }
}

/// How a recording is made.
#[derive(Clone, Debug)]
pub struct RecorderOptions {
    /// The session id (the folder name); a fresh one when `None`.
    pub id: Option<String>,
    pub sources: Vec<RecordSource>,
    pub note_id: Option<String>,
    pub limits: Limits,
    pub encode: EncodeConfig,
}

impl RecorderOptions {
    pub fn new(channels: Channels, sources: Vec<RecordSource>) -> Self {
        RecorderOptions {
            id: None,
            sources,
            note_id: None,
            limits: Limits::default(),
            encode: EncodeConfig::new(channels),
        }
    }
}

/// Writes one recording into its own folder under the in-progress folder.
pub struct Recorder {
    dir: PathBuf,
    writer: Option<PartWriter<DirSink>>,
    session: SessionFile,
    limits: Limits,
    channels: usize,
    samples: u64,
    last_checkpoint: u64,
    parts_at_checkpoint: usize,
    warned: bool,
    dir_syncs: Arc<AtomicUsize>,
}

impl Recorder {
    /// Creates `<root>/<id>` (`0700`, with `root` and its parents too) and
    /// writes the first sidecar. Fails when the session already exists.
    pub fn create(root: &Path, options: RecorderOptions) -> Result<Self, AudioError> {
        let id = options.id.unwrap_or_else(new_id);
        if !valid_id(&id) {
            return Err(AudioError::Config(format!("{id:?} isn't a session id")));
        }
        create_private_dir(root)?;
        let dir = root.join(&id);
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            DirBuilder::new().mode(0o700).create(&dir)?;
        }
        #[cfg(not(unix))]
        DirBuilder::new().create(&dir)?;
        sync_dir(root)?;

        let now = now_ms();
        let session = SessionFile {
            version: SessionFile::VERSION,
            id,
            started_at_ms: now,
            updated_at_ms: now,
            sources: options.sources,
            channels: options.encode.channels.count() as u8,
            note_id: options.note_id,
            state: SessionState::Recording,
            parts: Vec::new(),
            bytes: 0,
            samples: 0,
            cap_samples: options.limits.cap_samples,
        };
        let sink = DirSink::new(dir.clone());
        let dir_syncs = sink.dir_syncs();
        let channels = options.encode.channels.count();
        let writer = PartWriter::new(sink, options.encode)?;
        let recorder = Recorder {
            dir,
            writer: Some(writer),
            session,
            limits: options.limits,
            channels,
            samples: 0,
            last_checkpoint: 0,
            parts_at_checkpoint: 0,
            warned: false,
            dir_syncs,
        };
        recorder.write_sidecar()?;
        Ok(recorder)
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn session(&self) -> &SessionFile {
        &self.session
    }

    /// Samples per channel taken so far.
    pub fn samples(&self) -> u64 {
        self.samples
    }

    /// How many times a new part's folder has been synced.
    pub fn dir_syncs(&self) -> usize {
        self.dir_syncs.load(Ordering::Relaxed)
    }

    /// Takes interleaved samples (whole frames of `channels`), up to the
    /// cap. The sidecar is rewritten when 5 seconds of audio or a part
    /// boundary have passed.
    pub fn push(&mut self, pcm: &[f32]) -> Result<Progress, AudioError> {
        let writer = self
            .writer
            .as_mut()
            .ok_or_else(|| AudioError::Config("the recording is finished".into()))?;
        let room = self.limits.cap_samples.saturating_sub(self.samples);
        let frames = ((pcm.len() / self.channels) as u64).min(room);
        let before = self.samples;
        if frames > 0 {
            writer.push(&pcm[..frames as usize * self.channels])?;
            self.samples += frames;
        }
        let warned = !self.warned
            && before < self.limits.warn_samples
            && self.samples >= self.limits.warn_samples
            && frames > 0;
        self.warned |= warned;
        if self.samples - self.last_checkpoint >= CHECKPOINT_SAMPLES
            || self.writer.as_ref().map_or(0, |w| w.finished().len()) != self.parts_at_checkpoint
        {
            self.checkpoint()?;
        }
        Ok(Progress {
            samples: self.samples,
            warned,
            capped: self.samples >= self.limits.cap_samples,
        })
    }

    /// Forces the file to disk and rewrites the sidecar now (for a pause).
    pub fn checkpoint(&mut self) -> Result<(), AudioError> {
        if let Some(writer) = &mut self.writer {
            writer.sync()?;
        }
        self.refresh_session();
        self.write_sidecar()?;
        self.last_checkpoint = self.samples;
        self.parts_at_checkpoint = self.writer.as_ref().map_or(0, |w| w.finished().len());
        Ok(())
    }

    /// Ends the stream, closes the last part and marks the session stopped.
    /// The folder stays, with its parts, until the caller has attached them
    /// and calls [`remove_session`].
    pub fn finish(mut self) -> Result<Finished, AudioError> {
        let writer = self
            .writer
            .take()
            .ok_or_else(|| AudioError::Config("the recording is finished".into()))?;
        let parts = writer.finish()?;
        self.session.parts = parts.iter().map(|p| session_part(p, true)).collect();
        self.session.bytes = parts.iter().map(|p| p.bytes).sum();
        self.session.samples = parts.iter().map(|p| p.samples).sum();
        self.session.state = SessionState::Stopped;
        self.session.updated_at_ms = now_ms();
        self.write_sidecar()?;
        let paths = parts
            .iter()
            .map(|p| (self.dir.join(part_file_name(p.index)), *p))
            .collect();
        Ok(Finished {
            dir: self.dir,
            session: self.session,
            parts: paths,
        })
    }

    /// Throws the recording away: the parts, the sidecar and the folder.
    pub fn discard(mut self) -> io::Result<()> {
        self.writer = None;
        remove_session(&self.dir)
    }

    fn refresh_session(&mut self) {
        let Some(writer) = &self.writer else { return };
        let mut parts: Vec<SessionPart> = writer
            .finished()
            .iter()
            .map(|p| session_part(p, true))
            .collect();
        if let Some(open) = writer.open_part() {
            parts.push(SessionPart {
                index: open.index,
                file: part_file_name(open.index),
                bytes: open.bytes,
                samples: open.samples,
                closed: false,
            });
        }
        self.session.bytes = parts.iter().map(|p| p.bytes).sum();
        self.session.samples = self.samples;
        self.session.parts = parts;
        self.session.updated_at_ms = now_ms();
    }

    fn write_sidecar(&self) -> io::Result<()> {
        let json = serde_json::to_vec_pretty(&self.session).map_err(io::Error::other)?;
        write_atomic(&self.dir.join(SESSION_FILE), &json)
    }
}

fn session_part(part: &PartInfo, closed: bool) -> SessionPart {
    SessionPart {
        index: part.index,
        file: part_file_name(part.index),
        bytes: part.bytes,
        samples: part.samples,
        closed,
    }
}

/// Reads a sidecar.
pub fn read_session(dir: &Path) -> Result<SessionFile, AudioError> {
    let bytes = fs::read(dir.join(SESSION_FILE))?;
    serde_json::from_slice(&bytes).map_err(|e| AudioError::Session(e.to_string()))
}

/// Deletes a session folder: after its parts are attached, or on discard.
pub fn remove_session(dir: &Path) -> io::Result<()> {
    match fs::remove_dir_all(dir) {
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        other => other,
    }
}
