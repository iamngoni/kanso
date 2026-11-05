//! The plain types of a recording becoming a note (issue #38): what the
//! Record sheet chooses, where the recording goes, and what was saved.

use std::path::PathBuf;

/// What a recording is made from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WhatToRecord {
    Microphone,
    MicrophoneAndSystem,
    SystemOnly,
}

impl WhatToRecord {
    /// Whether the microphone alone is recorded: the note is a memo, not a
    /// meeting.
    pub fn mic_only(self) -> bool {
        self == WhatToRecord::Microphone
    }

    /// Whether the recording listens to another app's audio.
    pub fn uses_system(self) -> bool {
        self != WhatToRecord::Microphone
    }

    /// Whether the microphone is recorded too.
    pub fn uses_microphone(self) -> bool {
        self != WhatToRecord::SystemOnly
    }

    /// What the recording bar names.
    pub fn label(self) -> &'static str {
        match self {
            WhatToRecord::Microphone => "Microphone",
            WhatToRecord::MicrophoneAndSystem => "Microphone + System Audio",
            WhatToRecord::SystemOnly => "System Audio",
        }
    }
}

/// Whose audio the system track holds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SystemSource {
    /// One app, by bundle ID, with the name the picker showed.
    App { bundle_id: String, name: String },
    /// Everything the Mac plays except Kanso.
    Everything,
}

impl SystemSource {
    /// What the recording bar names: `Zoom`, or `All system audio`.
    pub fn label(&self) -> String {
        match self {
            SystemSource::App { name, .. } => name.clone(),
            SystemSource::Everything => "All system audio".to_owned(),
        }
    }
}

/// When the recording is turned into text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TranscribeWhen {
    /// As each piece closes.
    WhileRecording,
    /// When the recording stops.
    AfterStopping,
}

impl TranscribeWhen {
    /// The sheet's default for a source: a microphone alone transcribes
    /// live; system audio waits for the stop so a call keeps the CPU.
    pub fn default_for(what: WhatToRecord) -> Self {
        if what.mic_only() {
            TranscribeWhen::WhileRecording
        } else {
            TranscribeWhen::AfterStopping
        }
    }
}

/// Where the recording goes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Destination {
    /// A new note, in `book_id` or else the Inbox.
    NewNote { book_id: Option<String> },
    /// An existing note, which gets the recording at its end.
    ThisNote { note_id: String },
}

impl Destination {
    /// The note the recording goes into, when it is an existing one.
    pub fn note_id(&self) -> Option<&str> {
        match self {
            Destination::ThisNote { note_id } => Some(note_id),
            Destination::NewNote { .. } => None,
        }
    }
}

/// A finished recording to put in the library.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Recorded {
    /// The recording's session ID: it makes the file and note IDs, so
    /// saving again after a failure finds what was already stored.
    pub session_id: String,
    /// The part files, in playing order.
    pub parts: Vec<PathBuf>,
    /// The whole recording's length.
    pub duration_secs: u64,
    pub mic_only: bool,
    /// The date text for a new note's title, such as `23 Oct 2026, 14:05`.
    pub when: String,
}

/// What saving a recording made.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SavedRecording {
    pub note_id: String,
    /// One file ID per part, in playing order. The first names the
    /// recording: every transcript link points at it.
    pub file_ids: Vec<String>,
    /// A new note was made.
    pub created: bool,
}

/// What saving the system audio track made.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SavedTrack {
    /// One file ID per part, in playing order.
    pub file_ids: Vec<String>,
}

impl SavedRecording {
    /// The file the transcript's links name.
    pub fn first_file(&self) -> &str {
        &self.file_ids[0]
    }
}

/// Why a recording could not be saved.
#[derive(Debug, thiserror::Error)]
pub enum FlowError {
    #[error("The note is locked. Unlock it first, then record into it.")]
    Locked,
    #[error("The note is gone.")]
    Missing,
    #[error("There was no audio to keep.")]
    Empty,
    #[error("A part of the recording is over 10 MB, so it can't be attached.")]
    TooLarge,
    #[error("Couldn't read the recording: {0}")]
    Read(String),
    #[error(transparent)]
    Store(#[from] kanso_store::StoreError),
}

/// What the Record sheet needs to start, once chosen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordPlan {
    pub what: WhatToRecord,
    /// Whose audio, when `what` includes system audio. Nothing is chosen
    /// until the person picks.
    pub system: Option<SystemSource>,
    pub destination: Destination,
    pub transcribe: TranscribeWhen,
    /// The sound output to record when system audio is on: a device `id`
    /// from the platform's backend, or `None` for the default output.
    pub system_output: Option<String>,
}

impl RecordPlan {
    /// Why this plan can't start yet, in the person's words.
    pub fn problem(&self) -> Option<&'static str> {
        (self.what.uses_system() && self.system.is_none()).then_some("Choose which app to record.")
    }

    /// What the recording bar says is being recorded: `Microphone · Zoom`.
    pub fn bar_label(&self) -> String {
        let system = self.system.as_ref().map(SystemSource::label);
        match (self.what, system) {
            (WhatToRecord::Microphone, _) | (_, None) => self.what.label().to_owned(),
            (WhatToRecord::MicrophoneAndSystem, Some(app)) => format!("Microphone · {app}"),
            (WhatToRecord::SystemOnly, Some(app)) => app,
        }
    }
}

impl RecordPlan {
    /// The defaults: the microphone, into a new note, transcribed as the
    /// source suggests (the owner's answer to open question 1: a new note
    /// is the default, "This note" is a choice).
    pub fn new() -> Self {
        let what = WhatToRecord::Microphone;
        RecordPlan {
            what,
            system: None,
            destination: Destination::NewNote { book_id: None },
            transcribe: TranscribeWhen::default_for(what),
            system_output: None,
        }
    }
}

impl Default for RecordPlan {
    fn default() -> Self {
        Self::new()
    }
}
