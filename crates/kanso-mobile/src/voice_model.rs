//! The plain data of voice input: the setting and what a dictation turns
//! into. No logic lives here; `voice.rs` builds them.

/// Voice input's settings. The phones use the platform's on-device
/// recogniser, so unlike the desktop's there is no engine to choose.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VoiceSettings {
    /// Whether "comma", "new line", "delete that" and the like, said on their
    /// own, act instead of going in as words. On unless turned off.
    pub spoken_commands: bool,
}

impl Default for VoiceSettings {
    fn default() -> Self {
        VoiceSettings {
            spoken_commands: true,
        }
    }
}

/// What a transcript turns into at the caret.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dictated {
    /// The text to put in at the caret.
    pub text: String,
    /// Where the caret goes in `text`, in UTF-16 code units, on a character
    /// boundary and never past the end.
    pub caret_utf16: u32,
}
