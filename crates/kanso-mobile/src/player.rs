//! Playing a recording on the phone (issue #38, step 5): the desktop's
//! player (`kanso-audio`) behind the bridge. A decoder thread fills a ring
//! buffer for the speaker; the shell reads an atomic position about ten
//! times a second and draws the card (play, scrubber, speed, volume).
//!
//! Which recording a note holds, and whether its bytes are here, come from
//! the same code the desktop reads (`kanso_engine::recording_player`).

use kanso_audio::play::{PartInput, Player, measure};
use kanso_engine::recording_player::{self, RecordingRef};

use crate::errors::KansoError;
use crate::library::Library;
use crate::player_model::{NoteRecording, PartsState, PlayerStatus};

/// A recording on the speaker. One at a time is the shell's rule: dropping
/// it stops the sound.
pub struct PhonePlayer {
    inner: Player,
}

impl PhonePlayer {
    pub fn status(&self) -> PlayerStatus {
        let s = self.inner.status();
        PlayerStatus {
            position_ms: s.position_us / 1_000,
            length_ms: s.length_us / 1_000,
            playing: s.playing,
            ended: s.ended,
            speed: s.speed,
            volume: s.volume,
            error: s.error,
        }
    }

    /// Plays from where it is; from the start after the end.
    pub fn play(&self) {
        self.inner.play();
    }

    pub fn pause(&self) {
        self.inner.pause();
    }

    pub fn toggle(&self) {
        self.inner.toggle();
    }

    /// Moves to `ms` into the recording, held within it.
    pub fn seek_ms(&self, ms: u64) {
        self.inner.seek_us(ms.saturating_mul(1_000));
    }

    /// Moves by `delta_ms` (negative goes back).
    pub fn skip_ms(&self, delta_ms: i64) {
        self.inner.skip_us(delta_ms.saturating_mul(1_000));
    }

    /// 0.5× to 4×.
    pub fn set_speed(&self, speed: f32) {
        self.inner.set_speed(speed);
    }

    /// 0 to 1.
    pub fn set_volume(&self, volume: f32) {
        self.inner.set_volume(volume);
    }
}

fn is_audio(content_type: &str) -> bool {
    content_type
        .trim()
        .to_ascii_lowercase()
        .starts_with("audio/")
}

/// What a content type is called to the decoder (Ogg Opus is told by its
/// first page, so its hint is only a hint).
fn extension_for(content_type: &str) -> Option<&'static str> {
    match content_type.split(';').next().unwrap_or("").trim() {
        "audio/ogg" | "audio/opus" | "application/ogg" => Some("opus"),
        "audio/mp4" | "audio/x-m4a" | "audio/m4a" | "audio/aac" => Some("m4a"),
        "audio/mpeg" | "audio/mp3" => Some("mp3"),
        "audio/wav" | "audio/x-wav" | "audio/wave" => Some("wav"),
        "audio/flac" | "audio/x-flac" => Some("flac"),
        "audio/aiff" | "audio/x-aiff" => Some("aiff"),
        "audio/x-caf" => Some("caf"),
        _ => None,
    }
}

impl Library {
    /// The recording the note holds, if it holds one: the first audio line
    /// Kanso wrote, else the first line of links to audio the person
    /// attached. `None` for a locked note (closed), a note without audio, or
    /// one that isn't there. Its length is read from the files when they are
    /// all here.
    pub fn note_recording(&self, note_id: &str) -> Result<Option<NoteRecording>, KansoError> {
        let Some((_, text)) = self.readable_text(note_id)? else {
            return Ok(None);
        };
        let lines = recording_player::link_lines(&text);
        let chosen: Option<RecordingRef> = lines
            .iter()
            .find(|l| l.made_here)
            .or_else(|| {
                lines.iter().find(|l| {
                    matches!(
                        self.inner.store.file_meta(&l.file_ids[0]),
                        Ok(Some(meta)) if is_audio(&meta.content_type)
                    )
                })
            })
            .cloned();
        let Some(recording) = chosen else {
            return Ok(None);
        };
        let state = self.parts_state(&recording.file_ids);
        let length_ms = match state {
            PartsState::Ready => self
                .part_inputs(&recording.file_ids)
                .ok()
                .and_then(|parts| measure(&parts).ok())
                .map(|timeline| timeline.total_us() / 1_000),
            _ => None,
        };
        Ok(Some(NoteRecording {
            file_ids: recording.file_ids,
            state,
            length_ms,
        }))
    }

    /// Where the bytes of `file_ids` stand on this device: a part whose file
    /// document is known but whose bytes haven't been fetched is
    /// [`PartsState::NotHere`]; one that is gone or sealed is
    /// [`PartsState::Missing`].
    pub fn parts_state(&self, file_ids: &[String]) -> PartsState {
        let store = &self.inner.store;
        let mut state = PartsState::Ready;
        for id in file_ids {
            match store.attachment_state(id) {
                Ok(kanso_store::AttachmentState::Plain) => {
                    if !matches!(store.blob_md5(id), Ok(Some(_))) {
                        state = PartsState::NotHere;
                    }
                }
                _ => return PartsState::Missing,
            }
        }
        state
    }

    fn part_inputs(&self, file_ids: &[String]) -> Result<Vec<PartInput>, KansoError> {
        let mut parts = Vec::with_capacity(file_ids.len());
        for id in file_ids {
            let (content_type, attachment) = self
                .inner
                .store
                .attachment(id)?
                .ok_or(KansoError::NotFound)?;
            parts.push(PartInput::new(
                attachment.data,
                extension_for(&content_type),
            ));
        }
        Ok(parts)
    }

    /// Opens the parts of a recording on the speaker, paused at the start.
    pub fn player_open(&self, file_ids: Vec<String>) -> Result<PhonePlayer, KansoError> {
        match self.parts_state(&file_ids) {
            PartsState::Ready => {}
            PartsState::NotHere => {
                return Err(KansoError::invalid("The audio isn't on this device yet."));
            }
            PartsState::Missing => return Err(KansoError::NotFound),
        }
        let parts = self.part_inputs(&file_ids)?;
        let inner = Player::open(parts)
            .map_err(|e| KansoError::invalid(format!("Couldn't play the recording: {e}")))?;
        Ok(PhonePlayer { inner })
    }
}
