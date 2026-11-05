//! Recording on the phone (issue #38, step 4): the desktop's recorder
//! (`kanso-audio`) behind the bridge. The shell shows the bar and the
//! buttons; this keeps the session, writes the parts to the in-progress
//! folder as they happen, and on stop attaches them and makes or updates
//! the note through `kanso_engine::recording_flow`, the same code the
//! desktop ends in.
//!
//! The phone has no on-device speech model, so a recording's note holds the
//! audio line and the transcript waits for the desktop's.

use std::path::Path;
use std::sync::Mutex;

use kanso_audio::session::{in_progress_dir, now_ms, remove_session};
use kanso_audio::{
    AudioError, Capture, Channels, Finished, RecordSource, RecorderOptions, StopReason,
    meter_level, recovery,
};
use kanso_engine::recording_flow::{
    self, Destination, Recorded, WhatToRecord, save_recording_or_new,
};
use kanso_store::Origin;

use crate::errors::KansoError;
use crate::library::Library;
use crate::recording_model::{RecordingEnd, RecordingFound, RecordingSaved, RecordingStatus};

fn words(err: AudioError) -> KansoError {
    match err {
        AudioError::MicrophoneDenied => KansoError::invalid(
            "Kanso isn't allowed to use the microphone. Allow it in Settings, then try again.",
        ),
        AudioError::NoMicrophone => KansoError::invalid("There's no microphone to record from."),
        other => KansoError::invalid(format!("Couldn't record: {other}")),
    }
}

fn flow_words(err: recording_flow::FlowError) -> KansoError {
    match err {
        recording_flow::FlowError::Locked => KansoError::Locked,
        recording_flow::FlowError::Missing => KansoError::NotFound,
        other => KansoError::invalid(other.to_string()),
    }
}

/// The sessions recording now in this process. Every window (an iPad or
/// Android note window runs its own engine, and its own recording bloc)
/// asks for what a crash left; a recording that is still running must not
/// be offered for Recover or Discard there.
static ACTIVE: Mutex<Vec<String>> = Mutex::new(Vec::new());

fn set_active(id: &str, on: bool) {
    let mut active = ACTIVE.lock().unwrap_or_else(|e| e.into_inner());
    active.retain(|a| a != id);
    if on {
        active.push(id.to_owned());
    }
}

fn is_active(id: &str) -> bool {
    ACTIVE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .any(|a| a == id)
}

/// A recording in progress. One at a time is the shell's rule.
pub struct PhoneRecording {
    capture: Mutex<Option<Capture>>,
    id: String,
    destination: Destination,
    started_ms: u64,
}

impl PhoneRecording {
    /// How the recording is going.
    pub fn status(&self) -> RecordingStatus {
        let guard = self.capture.lock().unwrap();
        let Some(capture) = guard.as_ref() else {
            return RecordingStatus {
                seconds: 0,
                level: 0.,
                paused: false,
                warned: false,
                ended: None,
            };
        };
        let s = capture.status();
        RecordingStatus {
            seconds: s.seconds() as u64,
            level: meter_level(s.rms),
            paused: s.paused,
            warned: s.warned,
            ended: s.ended.as_ref().and_then(|reason| match reason {
                StopReason::Requested => None,
                StopReason::Cap => Some(RecordingEnd::Cap),
                StopReason::Device(_) => Some(RecordingEnd::Device),
                StopReason::Disk(_) => Some(RecordingEnd::DiskFull),
            }),
        }
    }

    pub fn pause(&self) {
        if let Some(c) = self.capture.lock().unwrap().as_ref() {
            c.pause();
        }
    }

    pub fn resume(&self) {
        if let Some(c) = self.capture.lock().unwrap().as_ref() {
            c.resume();
        }
    }

    /// Stops, attaches the parts and puts the recording in its note.
    pub fn stop(&self, library: &Library) -> Result<RecordingSaved, KansoError> {
        let capture = self
            .capture
            .lock()
            .unwrap()
            .take()
            .ok_or_else(|| KansoError::invalid("The recording is finished."))?;
        let dir = capture.dir().to_path_buf();
        let stopped = capture.stop();
        // Stopped: from here a failure leaves the folder for Recover.
        set_active(&self.id, false);
        let finished = match stopped {
            Ok(stopped) => stopped.finished,
            // A full disk can fail the last write: finish what is there.
            Err(err) => {
                let found = recovery::find(&dir).ok_or_else(|| words(err))?;
                recovery::recover(&found).map_err(words)?
            }
        };
        save_finished(
            library,
            &finished,
            &self.id,
            self.destination.clone(),
            self.started_ms,
        )
    }

    /// Stops and deletes the recording.
    pub fn discard(&self) -> Result<(), KansoError> {
        let result = match self.capture.lock().unwrap().take() {
            Some(capture) => capture.discard().map_err(words),
            None => Ok(()),
        };
        set_active(&self.id, false);
        result
    }
}

impl Drop for PhoneRecording {
    /// Dropped without Stop (the shell went away): the capture stops and
    /// keeps its folder, which is then offered for Recover.
    fn drop(&mut self) {
        if let Ok(mut capture) = self.capture.lock() {
            drop(capture.take());
        }
        set_active(&self.id, false);
    }
}

fn save_finished(
    library: &Library,
    finished: &Finished,
    id: &str,
    destination: Destination,
    started_ms: u64,
) -> Result<RecordingSaved, KansoError> {
    let rate = u64::from(kanso_audio::SAMPLE_RATE);
    let duration_secs = finished
        .parts
        .iter()
        .map(|(_, p)| p.samples)
        .sum::<u64>()
        .max(finished.session.samples)
        / rate;
    let recorded = Recorded {
        session_id: id.to_owned(),
        parts: finished.parts.iter().map(|(p, _)| p.clone()).collect(),
        duration_secs,
        mic_only: WhatToRecord::Microphone.mic_only(),
        when: recording_flow::when_text(started_ms as i64),
    };
    // A note deleted or locked while recording sends the audio to a new
    // note rather than leaving it out of the library.
    match save_recording_or_new(&library.inner.store, &recorded, &destination, Origin::Ui) {
        Ok(saved) => {
            let _ = remove_session(&finished.dir);
            Ok(RecordingSaved {
                note_id: saved.note_id,
                file_ids: saved.file_ids,
                duration_secs,
                created: saved.created,
            })
        }
        Err(recording_flow::FlowError::Empty) => {
            let _ = remove_session(&finished.dir);
            Err(KansoError::invalid("There was no audio to keep."))
        }
        // Kept in its folder: the next launch offers to recover it.
        Err(err) => Err(flow_words(err)),
    }
}

impl Library {
    /// Starts recording the microphone into a new note, or into `note_id`.
    /// `data_dir` is the app's data folder; the parts go in its
    /// `recordings/in-progress` folder. A locked note is refused.
    pub fn recording_start(
        &self,
        data_dir: &Path,
        note_id: Option<String>,
    ) -> Result<PhoneRecording, KansoError> {
        let destination = match note_id {
            Some(note_id) => Destination::ThisNote { note_id },
            None => Destination::NewNote { book_id: None },
        };
        // A gone or locked note (or one in a locked notebook) is refused
        // now, not after the recording.
        recording_flow::check_destination(&self.inner.store, &destination).map_err(flow_words)?;
        let root = in_progress_dir(data_dir);
        let id = kanso_audio::session::new_id();
        let mut options = RecorderOptions::new(Channels::Mono, vec![RecordSource::Microphone]);
        options.id = Some(id.clone());
        options.note_id = destination.note_id().map(str::to_owned);
        let capture = Capture::start(&root, options).map_err(words)?;
        set_active(&id, true);
        Ok(PhoneRecording {
            capture: Mutex::new(Some(capture)),
            id,
            destination,
            started_ms: now_ms(),
        })
    }

    /// The recordings a crash left in `data_dir`. First it deletes the ones
    /// nobody recovered or discarded for a week (issue #38, step 11); a
    /// recording running now, or touched since, is never among them.
    pub fn recording_found(&self, data_dir: &Path) -> Vec<RecordingFound> {
        let root = in_progress_dir(data_dir);
        let live: Vec<String> = ACTIVE.lock().unwrap_or_else(|e| e.into_inner()).clone();
        let live: Vec<&str> = live.iter().map(String::as_str).collect();
        kanso_audio::sweep::sweep_in_progress(&root, &live, now_ms(), kanso_audio::sweep::KEEP_MS);
        recovery::scan(&root, None)
            .into_iter()
            .filter(|r| !is_active(&r.id))
            .map(|r| {
                let started = r.session.as_ref().map_or(0, |s| s.started_at_ms);
                let seconds = r.duration_secs() as u64;
                let at = recording_flow::time_of_day(started as i64);
                RecordingFound {
                    id: r.id,
                    started_ms: started as i64,
                    seconds,
                    text: recording_flow::recovery_text(at.as_deref(), seconds),
                }
            })
            .collect()
    }

    /// Recover: finishes the partial recording and attaches it, into its
    /// note when that is still there.
    pub fn recording_recover(
        &self,
        data_dir: &Path,
        id: &str,
    ) -> Result<RecordingSaved, KansoError> {
        if !kanso_audio::session::valid_id(id) {
            return Err(KansoError::NotFound);
        }
        if is_active(id) {
            return Err(KansoError::invalid("That recording is still running."));
        }
        let found = recovery::find(&in_progress_dir(data_dir).join(id))
            .filter(|r| r.id == id)
            .ok_or(KansoError::NotFound)?;
        let finished = recovery::recover(&found).map_err(words)?;
        let session = finished.session.clone();
        let destination = match session.note_id.as_deref() {
            Some(note) if self.inner.store.note(note).ok().flatten().is_some() => {
                Destination::ThisNote {
                    note_id: note.to_owned(),
                }
            }
            _ => Destination::NewNote { book_id: None },
        };
        save_finished(self, &finished, id, destination, session.started_at_ms)
    }

    /// Discard: deletes the left-behind recording.
    pub fn recording_discard_found(&self, data_dir: &Path, id: &str) -> Result<(), KansoError> {
        if !kanso_audio::session::valid_id(id) {
            return Err(KansoError::invalid("That isn't a recording."));
        }
        if is_active(id) {
            return Err(KansoError::invalid("That recording is still running."));
        }
        remove_session(&in_progress_dir(data_dir).join(id))
            .map_err(|e| KansoError::storage(e.to_string()))
    }
}
