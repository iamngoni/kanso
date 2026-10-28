//! Voice input's rules and setting. The phone's recogniser makes the words;
//! the engine's spoken punctuation and commands (`kanso_engine::voice_commands`)
//! turn them into the text that goes in at the caret, the same rules as the
//! desktop's.
//!
//! The setting is Rust's, like the AI settings: `voice.json` next to the
//! library's database.

use serde::{Deserialize, Serialize};

use crate::boundary::guard;
use crate::errors::KansoError;
use crate::library::Library;
use crate::voice_model::{Dictated, VoiceSettings};

/// The file the setting is in, next to the library's database.
const SETTINGS_FILE: &str = "voice.json";

/// What `voice.json` holds; a field missing from an older file reads as its
/// default.
#[derive(Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct SettingsFile {
    spoken_commands: bool,
}

impl Default for SettingsFile {
    fn default() -> Self {
        SettingsFile {
            spoken_commands: VoiceSettings::default().spoken_commands,
        }
    }
}

/// The text to put in for `transcript`, between `before` (the note's text up
/// to the caret) and `after` (the text after it; only its first character
/// matters), and where the caret goes. With `commands` off, the transcript
/// goes in as said, trimmed. `None` when nothing is left to put in.
pub fn dictated(
    before: String,
    after: Option<String>,
    transcript: String,
    commands: bool,
) -> Option<Dictated> {
    let next = after.as_deref().and_then(|text| text.chars().next());
    kanso_engine::voice_commands::dictated(&before, next, &transcript, commands).map(
        |(text, caret)| Dictated {
            caret_utf16: kanso_engine::text::byte_to_utf16(&text, caret) as u32,
            text,
        },
    )
}

impl Library {
    /// Voice input's settings.
    pub fn voice_settings(&self) -> Result<VoiceSettings, KansoError> {
        guard(|| {
            let file: SettingsFile = std::fs::read(self.voice_path())
                .ok()
                .and_then(|bytes| serde_json::from_slice(&bytes).ok())
                .unwrap_or_default();
            Ok(VoiceSettings {
                spoken_commands: file.spoken_commands,
            })
        })
    }

    /// Saves voice input's settings.
    pub fn set_voice_settings(&self, settings: VoiceSettings) -> Result<(), KansoError> {
        guard(|| {
            let path = self.voice_path();
            let bytes = serde_json::to_vec_pretty(&SettingsFile {
                spoken_commands: settings.spoken_commands,
            })
            .map_err(|err| KansoError::storage(err.to_string()))?;
            let part = path.with_extension("json.part");
            std::fs::write(&part, bytes)
                .and_then(|()| std::fs::rename(&part, &path))
                .map_err(|err| KansoError::storage(err.to_string()))
        })
    }

    fn voice_path(&self) -> std::path::PathBuf {
        self.path()
            .parent()
            .filter(|dir| !dir.as_os_str().is_empty())
            .unwrap_or_else(|| std::path::Path::new("."))
            .join(SETTINGS_FILE)
    }
}
