//! Finding the recordings a crash or power cut left behind, and making them
//! whole.
//!
//! A killed process leaves its session folder in the in-progress folder: the
//! parts that were written (each ending at the last page that reached the
//! disk) and a sidecar at most five seconds old. [`scan`] lists them without
//! touching anything; [`recover`] cuts each part back to its last whole page
//! and returns the parts for attaching; [`discard`](crate::session::remove_session)
//! deletes one.

use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};

use crate::decode::OpusReader;
use crate::error::AudioError;
use crate::index::PageIndex;
use crate::model::PartInfo;
use crate::record_model::{Finished, Recoverable, SessionFile, SessionPart, SessionState};
use crate::session::{
    SESSION_FILE, now_ms, parse_part_name, part_file_name, read_session, sync_dir, valid_id,
    write_atomic,
};

/// The part files in `dir`, in order.
pub(crate) fn part_files(dir: &Path) -> std::io::Result<Vec<(u32, PathBuf)>> {
    let mut parts = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name();
        if let Some(index) = name.to_str().and_then(parse_part_name) {
            parts.push((index, entry.path()));
        }
    }
    parts.sort();
    Ok(parts)
}

/// Every unfinished recording under `root` (the in-progress folder), except
/// `active`, the one being recorded now. Folders with no audio in them are
/// left out. Nothing is changed.
pub fn scan(root: &Path, active: Option<&str>) -> Vec<Recoverable> {
    let Ok(entries) = fs::read_dir(root) else {
        return Vec::new();
    };
    let mut found = Vec::new();
    for entry in entries.flatten() {
        let Some(id) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        if !valid_id(&id) || Some(id.as_str()) == active || !entry.path().is_dir() {
            continue;
        }
        if let Some(recoverable) = find(&entry.path()) {
            found.push(recoverable);
        }
    }
    found.sort_by(|a, b| {
        let key = |r: &Recoverable| {
            (
                r.session.as_ref().map_or(0, |s| s.started_at_ms),
                r.id.clone(),
            )
        };
        key(a).cmp(&key(b))
    });
    found
}

/// The audio left in one session folder, or `None` when there is none (or
/// the folder isn't a session's). Nothing is changed.
pub fn find(dir: &Path) -> Option<Recoverable> {
    let id = dir.file_name()?.to_str()?.to_owned();
    if !valid_id(&id) || !dir.is_dir() {
        return None;
    }
    let session = read_session(dir).ok();
    let mut parts = Vec::new();
    for (index, path) in part_files(dir).unwrap_or_default() {
        let Ok(file) = File::open(&path) else {
            continue;
        };
        let bytes = file.metadata().map_or(0, |m| m.len());
        // A part whose headers never reached the disk has no audio.
        let Ok(reader) = OpusReader::open(file) else {
            continue;
        };
        let samples = reader.info().total_samples;
        if samples > 0 {
            parts.push((
                path,
                PartInfo {
                    index,
                    bytes,
                    samples,
                },
            ));
        }
    }
    if parts.is_empty() {
        return None;
    }
    let samples = parts.iter().map(|(_, p)| p.samples).sum();
    Some(Recoverable {
        id,
        dir: dir.to_path_buf(),
        session,
        parts,
        samples,
    })
}

/// Makes a left-behind recording whole: each part is cut back to its last
/// complete page (dropping a torn tail), parts with no audio are removed, and
/// the sidecar is rewritten as stopped. The result is what
/// [`Recorder::finish`](crate::session::Recorder::finish) would have
/// returned, less the last few seconds.
pub fn recover(found: &Recoverable) -> Result<Finished, AudioError> {
    let mut parts = Vec::new();
    for (index, path) in part_files(&found.dir)? {
        let mut file = File::open(&path)?;
        let len = file.metadata()?.len();
        let Ok(reader) = OpusReader::open(&mut file) else {
            fs::remove_file(&path)?;
            continue;
        };
        let samples = reader.info().total_samples;
        let end = PageIndex::scan(&mut file)?.end_offset;
        drop(file);
        if samples == 0 {
            fs::remove_file(&path)?;
            continue;
        }
        let mut bytes = len;
        if end < len {
            let file = OpenOptions::new().write(true).open(&path)?;
            file.set_len(end)?;
            file.sync_all()?;
            bytes = end;
        }
        parts.push((
            path,
            PartInfo {
                index,
                bytes,
                samples,
            },
        ));
    }
    sync_dir(&found.dir)?;
    if parts.is_empty() {
        return Err(AudioError::NoSound);
    }
    let samples = parts.iter().map(|(_, p)| p.samples).sum();
    let bytes = parts.iter().map(|(_, p)| p.bytes).sum();
    let now = now_ms();
    let mut session = found.session.clone().unwrap_or_else(|| SessionFile {
        version: SessionFile::VERSION,
        id: found.id.clone(),
        started_at_ms: now,
        updated_at_ms: now,
        sources: Vec::new(),
        channels: 1,
        note_id: None,
        state: SessionState::Recording,
        parts: Vec::new(),
        bytes: 0,
        samples: 0,
        cap_samples: 0,
    });
    session.state = SessionState::Stopped;
    session.updated_at_ms = now;
    session.bytes = bytes;
    session.samples = samples;
    session.parts = parts
        .iter()
        .map(|(_, p)| SessionPart {
            index: p.index,
            file: part_file_name(p.index),
            bytes: p.bytes,
            samples: p.samples,
            closed: true,
        })
        .collect();
    let json = serde_json::to_vec_pretty(&session).map_err(std::io::Error::other)?;
    // On a full disk the parts are whole after the truncation above; a
    // sidecar left saying "recording" only means the folder is offered for
    // recovery again if it isn't attached.
    match write_atomic(&found.dir.join(SESSION_FILE), &json) {
        Err(e) if e.kind() != std::io::ErrorKind::StorageFull => return Err(e.into()),
        _ => {}
    }
    Ok(Finished {
        dir: found.dir.clone(),
        session,
        parts,
    })
}
