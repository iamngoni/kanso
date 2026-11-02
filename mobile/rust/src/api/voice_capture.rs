//! Voice capture: a voice note into the Inbox, the recordings the lock
//! screen, the Action Button and the Android tile leave to be transcribed,
//! and tidying a voice note with the person's own AI.

use flutter_rust_bridge::frb;
use kanso_mobile::{KansoError, Library};

use super::ai::KansoAiRun;
use super::voice_capture_types::{
    TidyOutcome, VoiceCaptureInput, VoiceCaptureSaved, VoiceDrop, VoiceNoteInfo,
};

#[frb(opaque)]
pub struct KansoVoiceCapture(Library);

impl KansoVoiceCapture {
    pub(super) fn new(library: Library) -> KansoVoiceCapture {
        KansoVoiceCapture(library)
    }

    /// Saves a voice note as a note in the Inbox. No `note_id` in the answer
    /// means nothing was heard and nothing was saved.
    pub fn save(&self, input: VoiceCaptureInput) -> Result<VoiceCaptureSaved, KansoError> {
        self.0.save_voice(input)
    }

    /// The recordings waiting in `inbox` to be transcribed, oldest first.
    pub fn drops(&self, inbox: String) -> Result<Vec<VoiceDrop>, KansoError> {
        self.0.voice_drops(std::path::Path::new(&inbox))
    }

    /// Saves recording `id` with the transcript the phone's recogniser made.
    pub fn save_drop(
        &self,
        inbox: String,
        id: String,
        transcript: String,
        on_device: bool,
    ) -> Result<VoiceCaptureSaved, KansoError> {
        self.0
            .save_voice_drop(std::path::Path::new(&inbox), &id, transcript, on_device)
    }

    /// Removes recording `id` without saving it.
    pub fn discard_drop(&self, inbox: String, id: String) -> Result<(), KansoError> {
        self.0.discard_voice_drop(std::path::Path::new(&inbox), &id)
    }

    /// Whether note `id` is a voice note, whether it was tidied, and whether
    /// Tidy may be offered.
    pub fn note_info(&self, id: String) -> Result<Option<VoiceNoteInfo>, KansoError> {
        self.0.voice_note_info(&id)
    }

    /// Tidies voice note `id`. Blocks until the model has answered; `run`
    /// (from `KansoAi::new_run`) stops it.
    pub fn tidy_note(&self, id: String, run: &KansoAiRun) -> Result<TidyOutcome, KansoError> {
        self.0.ai().tidy_note(&id, &run.0)
    }
}
