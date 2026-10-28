//! The plain data of the AI features: settings, the providers to choose from,
//! the inline assistant's actions and the events a question or a rewrite
//! streams.
//!
//! No logic lives here; `ai.rs` builds each one. Like the rest of the
//! facade's types, every one is `Clone + Debug + PartialEq` with public
//! fields, so the bridge can mirror them field by field.

/// What the AI settings are now. The key is never in it: [`Self::has_key`]
/// says whether one is saved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AiSettingsView {
    /// AI is off until the person turns it on.
    pub enabled: bool,
    /// The provider's key in [`AiProviderInfo::key`].
    pub provider: String,
    pub base_url: String,
    pub model: String,
    /// The AWS region, for Bedrock only; empty for the others.
    pub region: String,
    /// Whether a key is saved for [`Self::provider`].
    pub has_key: bool,
    /// The server questions go to, for the line that says where they go.
    pub host: String,
    /// Whether questions leave this device for [`Self::host`]: true unless
    /// the host is this device itself.
    pub is_remote: bool,
}

/// The settings to save. Unlike [`AiSettingsView`] it has no key (see
/// `AiHandle::set_key`) and nothing derived.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AiSettingsInput {
    pub enabled: bool,
    pub provider: String,
    pub base_url: String,
    pub model: String,
    pub region: String,
}

/// A provider to choose from, as the desktop's Settings › AI describes it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AiProviderInfo {
    pub key: String,
    pub label: String,
    /// One line about the provider and where its key comes from.
    pub description: String,
    pub default_base_url: String,
    pub default_model: String,
    /// The AWS region Bedrock starts in; empty for the others.
    pub default_region: String,
    /// Whether a key is needed before asking.
    pub needs_key: bool,
    /// Whether the provider takes a key at all.
    pub accepts_key: bool,
}

/// One thing the inline assistant can do with the selected text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AiAction {
    /// The name `AiHandle::assist` takes.
    pub key: String,
    /// The menu label, in the desktop's words.
    pub label: String,
    /// What the suggestion is called once it is there ("Improved").
    pub noun: String,
    /// Whether accepting replaces the selection (or else goes after it).
    pub replaces: bool,
}

/// A note an answer drew on, or cited.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AskNote {
    /// The note's ID, as every other call takes it.
    pub id: String,
    pub title: String,
}

/// What a question says as it is answered. A question ends with exactly one
/// of [`AskEvent::Done`], [`AskEvent::NothingMatches`], [`AskEvent::CutOff`],
/// [`AskEvent::Failed`] or [`AskEvent::Stopped`], and sends nothing after it.
///
/// Every answer is Markdown with no HTML and no images, in which each note it
/// cites is a link whose address is `kanso://note/<note id>`
/// ([`crate::ai::CITATION_PREFIX`]): the shell opens that note when it is
/// tapped.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AskEvent {
    /// Looking through the notes.
    Searching,
    /// The notes the answer has drawn on so far: the first excerpts, then
    /// any the model searched or read. Each time, the whole list.
    Used { notes: Vec<AskNote> },
    /// A line about what is happening that isn't part of the answer.
    Status { line: String },
    /// The whole answer so far.
    Answer { markdown: String },
    /// The answer is complete. `notes` are all the notes it drew on.
    Done {
        markdown: String,
        notes: Vec<AskNote>,
    },
    /// Nothing in the notes matches the question, so no model was asked.
    NothingMatches { message: String },
    /// The answer stopped before its end. What arrived stays shown.
    CutOff { message: String },
    /// The question couldn't be answered, in plain words.
    Failed { message: String },
    /// The person stopped it.
    Stopped,
}

/// What the inline assistant and the note summary say as they work.
///
/// Like a question, one ends with exactly one of [`AssistEvent::Done`],
/// [`AssistEvent::CutOff`], [`AssistEvent::Failed`] or
/// [`AssistEvent::Stopped`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssistEvent {
    /// A line about what is happening: reading part 2 of 5 of a long note.
    Status {
        line: String,
    },
    /// The whole suggestion so far, as Markdown to put in the note.
    Text {
        text: String,
    },
    /// The suggestion is complete.
    Done {
        text: String,
    },
    /// The suggestion stopped before its end.
    CutOff {
        message: String,
    },
    Failed {
        message: String,
    },
    Stopped,
}
