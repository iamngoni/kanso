//! Playing a recording on the phone (issue #38, step 5): the desktop's
//! player behind the bridge. Find the recording a note holds, check its
//! bytes are here, open it on the speaker, and drive it: play, pause, seek,
//! speed and volume, with a status the card reads about ten times a second.

use flutter_rust_bridge::frb;
use kanso_mobile::{KansoError, Library, PhonePlayer};

use super::player_types::{NoteRecording, PartsState, PlayerStatus};

/// Finding a note's recording and opening it.
#[frb(opaque)]
pub struct KansoPlayers(Library);

impl KansoPlayers {
    pub(super) fn new(library: Library) -> KansoPlayers {
        KansoPlayers(library)
    }

    /// The recording note `note_id` holds, if any: its parts, whether their
    /// bytes are here, and its length.
    pub fn note_recording(&self, note_id: String) -> Result<Option<NoteRecording>, KansoError> {
        self.0.note_recording(&note_id)
    }

    /// Where the bytes of the parts stand on this device.
    #[frb(sync)]
    pub fn parts_state(&self, file_ids: Vec<String>) -> PartsState {
        self.0.parts_state(&file_ids)
    }

    /// Opens the parts on the speaker, paused at the start.
    pub fn open(&self, file_ids: Vec<String>) -> Result<KansoPlayer, KansoError> {
        Ok(KansoPlayer {
            inner: self.0.player_open(file_ids)?,
        })
    }
}

/// A recording on the speaker. Dropping it stops the sound.
#[frb(opaque)]
pub struct KansoPlayer {
    inner: PhonePlayer,
}

impl KansoPlayer {
    #[frb(sync)]
    pub fn status(&self) -> PlayerStatus {
        self.inner.status()
    }

    #[frb(sync)]
    pub fn play(&self) {
        self.inner.play()
    }

    #[frb(sync)]
    pub fn pause(&self) {
        self.inner.pause()
    }

    #[frb(sync)]
    pub fn toggle(&self) {
        self.inner.toggle()
    }

    /// Moves to `ms` into the recording.
    #[frb(sync)]
    pub fn seek_ms(&self, ms: u64) {
        self.inner.seek_ms(ms)
    }

    /// Moves by `delta_ms` (negative goes back).
    #[frb(sync)]
    pub fn skip_ms(&self, delta_ms: i64) {
        self.inner.skip_ms(delta_ms)
    }

    #[frb(sync)]
    pub fn set_speed(&self, speed: f32) {
        self.inner.set_speed(speed)
    }

    #[frb(sync)]
    pub fn set_volume(&self, volume: f32) {
        self.inner.set_volume(volume)
    }
}
