//! Voice capture on the phone (issue #193): saving a voice note into the
//! Inbox, the recordings the lock screen, the Action Button and the Android
//! tile leave to be transcribed, and tidying a voice note with the person's
//! own AI.
//!
//! The rules are the engine's (`kanso_engine::voice_capture` and
//! `voice_tidy`), which the desktop's Quick Capture uses as well. This is the
//! shell's side: the folder of drops, the settings, and the model call.
//!
//! # Voice drops
//!
//! A recording made outside the app is a *voice drop*: a folder in the
//! library's `inbox/` with its audio and a `drop.json` of
//! `{"version": 1, "kind": "voice", "createdAt", "started_ms", "duration_ms",
//! "audio": "<file>"}` (see `mobile/docs/capture.md`). `capture::save_drops`
//! leaves such a folder alone. When the app is open it lists them
//! ([`Library::voice_drops`]), the shell transcribes each file with the
//! phone's recogniser, and [`Library::save_voice_drop`] makes the note and
//! removes the folder. Nothing in the intent's process ever reads the
//! library or a key.

use std::fs;
use std::path::Path;
use std::sync::atomic::Ordering;

use kanso_ai::tidy::tidy as run_tidy;
use kanso_core::capture_mark::{TidyState, read_tidy};
use kanso_engine::capture::{CaptureError, is_drop_id};
use kanso_engine::voice_capture::{
    Engine, VoiceCapture, is_meeting, new_capture_id, save_audio_only, save_voice_capture,
};
use kanso_engine::voice_tidy::{TidyError, apply_tidy, tidy_source};
use kanso_store::Origin;
use serde_json::Value;

use crate::ai::{AiHandle, AiRun, KEPT_OUT};
use crate::boundary::{guard, held};
use crate::errors::KansoError;
use crate::library::Library;
use crate::voice::read_settings;
use crate::voice_capture_model::{
    TidyOutcome, VoiceCaptureInput, VoiceCaptureSaved, VoiceDrop, VoiceNoteInfo,
};
use crate::voice_model::VoiceTidy;

/// What a recording that held no speech says.
pub const NOTHING_HEARD: &str = "Nothing was heard. Nothing was saved.";

fn engine_of(on_device: bool) -> Engine {
    if on_device {
        Engine::SystemOnDevice
    } else {
        Engine::SystemServer
    }
}

fn map_capture(err: CaptureError) -> KansoError {
    match err {
        CaptureError::Locked => KansoError::Locked,
        CaptureError::BadId => KansoError::Internal {
            message: "the capture's ID was not valid".into(),
        },
        CaptureError::Empty(_) => KansoError::invalid(NOTHING_HEARD),
        CaptureError::Store(err) => err.into(),
    }
}

fn map_tidy(err: TidyError) -> KansoError {
    match err {
        TidyError::KeptOut => KansoError::invalid(KEPT_OUT),
        TidyError::Missing => KansoError::NotFound,
        TidyError::Store(err) => err.into(),
        other => KansoError::invalid(other.to_string()),
    }
}

impl Library {
    /// Saves a voice note made in the app (or transcribed from a drop) as a
    /// note in the Inbox. `Ok` with no `note_id` when nothing was heard.
    /// A locked Inbox refuses ([`KansoError::Locked`]).
    pub fn save_voice(&self, input: VoiceCaptureInput) -> Result<VoiceCaptureSaved, KansoError> {
        guard(|| self.save_voice_with(input, None))
    }

    fn save_voice_with(
        &self,
        input: VoiceCaptureInput,
        audio: Option<&Path>,
    ) -> Result<VoiceCaptureSaved, KansoError> {
        let settings = read_settings(self.path());
        let id = if input.id.is_empty() {
            new_capture_id()
        } else {
            input.id.clone()
        };
        let meeting = is_meeting(input.duration_ms, settings.meetings);
        let ai_on = AiHandle::is_on(self);
        let tidy_now = settings.tidy == VoiceTidy::Always && ai_on;
        let capture = VoiceCapture {
            id,
            started_ms: input.started_ms,
            duration_ms: input.duration_ms,
            transcript: input.transcript,
            segments: Vec::new(),
            engine: engine_of(input.on_device),
            audio_file: audio.map(Path::to_path_buf),
        };
        let keep = settings.keep_audio || meeting;
        let saved = save_voice_capture(&self.inner.store, &capture, keep, tidy_now, Origin::Ui)
            .map_err(map_capture)?;
        let Some(saved) = saved else {
            return Ok(VoiceCaptureSaved {
                note_id: None,
                problems: Vec::new(),
                existed: false,
                meeting,
                tidy_now: false,
            });
        };
        if !saved.existed {
            self.inner
                .metrics()
                .count(crate::metrics::Feature::VoiceInput);
        }
        Ok(VoiceCaptureSaved {
            note_id: Some(saved.id),
            problems: saved.problems,
            existed: saved.existed,
            meeting,
            tidy_now: tidy_now && !saved.existed,
        })
    }

    /// The voice recordings waiting in `inbox` (a folder of drops) to be
    /// transcribed, oldest first.
    pub fn voice_drops(&self, inbox: &Path) -> Result<Vec<VoiceDrop>, KansoError> {
        guard(|| Ok(voice_drops_in(inbox)))
    }

    /// Saves the voice drop `id` in `inbox` with the `transcript` the shell's
    /// recogniser made, and removes its folder. A transcript with no words
    /// removes the folder and saves nothing. A locked Inbox refuses and
    /// removes nothing: the recording waits.
    pub fn save_voice_drop(
        &self,
        inbox: &Path,
        id: &str,
        transcript: String,
        on_device: bool,
    ) -> Result<VoiceCaptureSaved, KansoError> {
        guard(|| {
            let drop = find_drop(inbox, id)?;
            let audio = inbox.join(id).join(&drop.file);
            let saved = self.save_voice_with(
                VoiceCaptureInput {
                    id: id.to_owned(),
                    started_ms: drop.info.started_ms,
                    duration_ms: drop.info.duration_ms,
                    transcript,
                    on_device,
                },
                Some(&audio),
            )?;
            remove_drop(inbox, id);
            Ok(saved)
        })
    }

    /// Saves voice drop `id` as an audio note, for a recording the phone
    /// could not turn into text: the audio is attached to a note in the
    /// Inbox that says so, and the folder removed. Nothing is lost.
    pub fn save_voice_drop_audio_only(
        &self,
        inbox: &Path,
        id: &str,
    ) -> Result<VoiceCaptureSaved, KansoError> {
        guard(|| {
            let drop = find_drop(inbox, id)?;
            let audio = inbox.join(id).join(&drop.file);
            let saved = save_audio_only(
                &self.inner.store,
                id,
                "Voice note",
                drop.info.duration_ms,
                Engine::SystemOnDevice,
                &audio,
                Origin::Ui,
            )
            .map_err(|err| match err {
                CaptureError::Empty(why) => KansoError::invalid(why.join(" ")),
                other => map_capture(other),
            })?;
            remove_drop(inbox, id);
            Ok(VoiceCaptureSaved {
                note_id: Some(saved.id),
                problems: saved.problems,
                existed: saved.existed,
                meeting: false,
                tidy_now: false,
            })
        })
    }

    /// Removes the voice drop `id` without saving it.
    pub fn discard_voice_drop(&self, inbox: &Path, id: &str) -> Result<(), KansoError> {
        guard(|| {
            if is_drop_id(id) {
                remove_drop(inbox, id);
            }
            Ok(())
        })
    }

    /// What the shell shows of voice note `id`: whether it was tidied, and
    /// whether Tidy may be offered. `None` for a note that isn't a voice
    /// capture.
    pub fn voice_note_info(&self, id: &str) -> Result<Option<VoiceNoteInfo>, KansoError> {
        guard(|| {
            let Some(doc) = self.inner.store.get(id)? else {
                return Ok(None);
            };
            let Some(state) = read_tidy(&doc.fields) else {
                return Ok(None);
            };
            let settings = read_settings(self.path());
            let can_tidy = state != TidyState::Done
                && settings.tidy != VoiceTidy::Off
                && AiHandle::is_on(self)
                && kanso_engine::assist::note_for_ai(&self.inner.store, id).is_some();
            Ok(Some(VoiceNoteInfo {
                tidy: match state {
                    TidyState::None => "none",
                    TidyState::Pending => "pending",
                    TidyState::Done => "done",
                }
                .to_owned(),
                duration_ms: kanso_core::capture_mark::read_duration_ms(&doc.fields).unwrap_or(0),
                can_tidy,
            }))
        })
    }
}

/// What a voice drop's `drop.json` says.
struct DropInfo {
    started_ms: i64,
    duration_ms: u64,
}

struct FoundDrop {
    info: DropInfo,
    file: String,
}

/// The voice drop `id`, if it is one and its audio is there.
fn find_drop(inbox: &Path, id: &str) -> Result<FoundDrop, KansoError> {
    if !is_drop_id(id) {
        return Err(KansoError::NotFound);
    }
    read_voice_drop(&inbox.join(id)).ok_or(KansoError::NotFound)
}

fn read_voice_drop(folder: &Path) -> Option<FoundDrop> {
    let meta = fs::symlink_metadata(folder).ok()?;
    if !meta.is_dir() {
        return None;
    }
    let bytes = fs::read(folder.join("drop.json")).ok()?;
    let json: Value = serde_json::from_slice(&bytes).ok()?;
    if json.get("kind").and_then(Value::as_str) != Some("voice") {
        return None;
    }
    let file = json.get("audio")?.as_str()?.to_owned();
    if !crate::capture::is_plain_name(&file) {
        return None;
    }
    let audio = fs::symlink_metadata(folder.join(&file)).ok()?;
    if !audio.is_file() {
        return None;
    }
    let created = json.get("createdAt").and_then(Value::as_i64).unwrap_or(0);
    Some(FoundDrop {
        info: DropInfo {
            started_ms: json
                .get("started_ms")
                .and_then(Value::as_i64)
                .unwrap_or(created),
            duration_ms: json.get("duration_ms").and_then(Value::as_u64).unwrap_or(0),
        },
        file,
    })
}

fn voice_drops_in(inbox: &Path) -> Vec<VoiceDrop> {
    let Ok(entries) = fs::read_dir(inbox) else {
        return Vec::new();
    };
    let mut found: Vec<(i64, VoiceDrop)> = Vec::new();
    for entry in entries.flatten() {
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        if !is_drop_id(&name) {
            continue;
        }
        let folder = entry.path();
        let Some(drop) = read_voice_drop(&folder) else {
            continue;
        };
        found.push((
            drop.info.started_ms,
            VoiceDrop {
                id: name,
                audio_path: folder.join(&drop.file).to_string_lossy().into_owned(),
                started_ms: drop.info.started_ms,
                duration_ms: drop.info.duration_ms,
            },
        ));
    }
    found.sort_by(|a, b| (a.0, &a.1.id).cmp(&(b.0, &b.1.id)));
    found.into_iter().map(|(_, drop)| drop).collect()
}

fn remove_drop(inbox: &Path, id: &str) {
    if let Err(err) = fs::remove_dir_all(inbox.join(id)) {
        eprintln!("kanso: couldn't remove the voice drop {id}: {err}");
    }
}

impl AiHandle {
    /// Whether AI is turned on for `library`.
    pub(crate) fn is_on(library: &Library) -> bool {
        library.ai().enabled().is_ok()
    }

    /// Tidies voice note `id` with the person's own AI: a title, short
    /// paragraphs, and the decisions and tasks the recording backs with a
    /// quote. Blocks until the model has answered; `run` stops it. Refuses a
    /// note that isn't a voice capture, and one kept out of AI (locked, in
    /// the trash, `#ai-exclude`) before anything is sent. The note is
    /// changed in one write at the end, so Undo and its history bring the
    /// original back; a failed tidy changes nothing.
    pub fn tidy_note(&self, id: &str, run: &AiRun) -> Result<TidyOutcome, KansoError> {
        guard(|| {
            let settings = self.enabled()?;
            let meetings = read_settings(&self.inner.path).meetings;
            let source = tidy_source(&self.inner.store, id, meetings).map_err(map_tidy)?;
            let provider = self.provider_for(&settings)?;
            self.inner.metrics().count(crate::metrics::Feature::Ai);
            let result = {
                let provider = held(&provider);
                run_tidy(
                    provider.as_ref(),
                    source.kind,
                    &source.title,
                    &source.transcript,
                    &run.cancel_flag(),
                )
            };
            if run.cancel_flag().load(Ordering::Relaxed) {
                return Err(KansoError::invalid("Stopped."));
            }
            let tidy = result.map_err(KansoError::invalid)?;
            let applied =
                apply_tidy(&self.inner.store, &source, &tidy, Origin::Ui).map_err(map_tidy)?;
            Ok(TidyOutcome {
                inserted_above: applied.inserted_above,
                dropped: applied.dropped as u32,
            })
        })
    }
}
