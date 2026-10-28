//! Voice input: the spoken punctuation and commands on what the phone's
//! recogniser hears.

use flutter_rust_bridge::frb;

/// The one text command that is inserted into the note for a phrase.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dictated {
    pub text: String,
    pub caret_utf16: u32,
}

/// Spoken-command preference saved beside the library database.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VoiceSettings {
    pub spoken_commands: bool,
}

/// The text to put in for `transcript` at the caret, between `before` and
/// `after`, and where the caret goes (in UTF-16 units). `None` when nothing
/// is left to put in.
#[frb(sync)]
pub fn dictated(
    before: String,
    after: Option<String>,
    transcript: String,
    commands: bool,
) -> Option<Dictated> {
    kanso_mobile::dictated(before, after, transcript, commands).map(|made| Dictated {
        text: made.text,
        caret_utf16: made.caret_utf16,
    })
}
