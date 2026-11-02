//! The plain data of voice input: the setting and what a dictation turns
//! into. No logic lives here; `voice.rs` builds them.

/// Whether a voice note is tidied by the person's own AI (issue #193).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VoiceTidy {
    Off,
    /// A Tidy button is offered each time.
    #[default]
    Ask,
    /// Every voice capture is tidied once the app can.
    Always,
}

/// Voice input's settings. The phones use the platform's on-device
/// recogniser, so unlike the desktop's there is no engine to choose.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VoiceSettings {
    /// Whether "comma", "new line", "delete that" and the like, said on their
    /// own, act instead of going in as words. On unless turned off.
    pub spoken_commands: bool,
    /// Tidy voice notes: Off, Ask each time (the default), Always.
    pub tidy: VoiceTidy,
    /// Treat recordings longer than three minutes as meetings.
    pub meetings: bool,
    /// Keep the recording's audio with the note. A meeting's audio is kept
    /// either way (within the attachment limit).
    pub keep_audio: bool,
    /// Let the system's server-backed recogniser transcribe when the phone
    /// can't do it on the device. Off: such a recording is kept, untouched,
    /// until this is turned on.
    pub allow_server_recognition: bool,
}

impl Default for VoiceSettings {
    fn default() -> Self {
        VoiceSettings {
            spoken_commands: true,
            tidy: VoiceTidy::default(),
            meetings: true,
            keep_audio: false,
            allow_server_recognition: false,
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
