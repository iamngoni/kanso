//! Recording a note on the phone (issue #38, step 4): the desktop's
//! recorder behind the bridge. Start, watch the status, pause, stop into a
//! note or discard; and after a crash, find, recover or discard what it
//! left.

use flutter_rust_bridge::frb;
use kanso_mobile::{KansoError, Library, PhoneRecording};

use super::recording_types::{RecordingFound, RecordingSaved, RecordingStatus};

/// Starting a recording and what a crash left.
#[frb(opaque)]
pub struct KansoRecorder(Library);

impl KansoRecorder {
    pub(super) fn new(library: Library) -> KansoRecorder {
        KansoRecorder(library)
    }

    /// Starts recording the microphone into a new note, or into `note_id`.
    /// `data_dir` is the app's data folder.
    pub fn start(
        &self,
        data_dir: String,
        note_id: Option<String>,
    ) -> Result<KansoRecording, KansoError> {
        let inner = self
            .0
            .recording_start(std::path::Path::new(&data_dir), note_id)?;
        Ok(KansoRecording {
            inner,
            library: self.0.clone(),
        })
    }

    /// The recordings a crash left.
    pub fn found(&self, data_dir: String) -> Vec<RecordingFound> {
        self.0.recording_found(std::path::Path::new(&data_dir))
    }

    /// Finishes a left-behind recording and attaches it to its note.
    pub fn recover(&self, data_dir: String, id: String) -> Result<RecordingSaved, KansoError> {
        self.0
            .recording_recover(std::path::Path::new(&data_dir), &id)
    }

    /// Deletes a left-behind recording.
    pub fn discard_found(&self, data_dir: String, id: String) -> Result<(), KansoError> {
        self.0
            .recording_discard_found(std::path::Path::new(&data_dir), &id)
    }
}

/// A recording in progress.
#[frb(opaque)]
pub struct KansoRecording {
    inner: PhoneRecording,
    library: Library,
}

impl KansoRecording {
    /// The clock, level and whether it ended by itself.
    #[frb(sync)]
    pub fn status(&self) -> RecordingStatus {
        self.inner.status()
    }

    #[frb(sync)]
    pub fn pause(&self) {
        self.inner.pause()
    }

    #[frb(sync)]
    pub fn resume(&self) {
        self.inner.resume()
    }

    /// Stops, attaches the audio and puts it in its note.
    pub fn stop(&self) -> Result<RecordingSaved, KansoError> {
        self.inner.stop(&self.library)
    }

    /// Stops and deletes the recording.
    pub fn discard(&self) -> Result<(), KansoError> {
        self.inner.discard()
    }
}
