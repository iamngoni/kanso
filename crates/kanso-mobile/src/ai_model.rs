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
    /// Most each question may send, as a key: `small`, `standard` or `large`.
    /// What the model suits until it is chosen.
    pub ask_budget: String,
    /// "About 3,000 words of your notes per question."
    pub budget_words_line: String,
    /// "At most about $0.01 per question with gpt-4o-mini."; empty when the
    /// price isn't known.
    pub budget_cost_line: String,
    /// Check that each cited line was sent and supports its sentence.
    pub check_sources: bool,
    /// Look up related notes through links and backlinks.
    pub use_links: bool,
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
    /// `small`, `standard` or `large`; empty leaves the size the model
    /// suits.
    pub ask_budget: String,
    pub check_sources: bool,
    pub use_links: bool,
}

/// What a question is limited to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AskScopeInput {
    /// `all`, `book` (a notebook and the ones inside it) or `tag`.
    pub kind: String,
    /// The notebook's ID or the tag's name; empty for `all`.
    pub value: String,
}

/// One choice in the scope sheet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AskScopeOption {
    pub kind: String,
    pub value: String,
    /// "All notes", a notebook's path (`Work/Plans`) or `#tag`.
    pub label: String,
}

/// One receipt: the lines an answer's sentence rests on, as they were sent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AskReceipt {
    /// Its number in the answer, from 1.
    pub n: u32,
    pub note_id: String,
    pub title: String,
    /// "Pricing \u{b7} lines 12\u{2013}14 \u{b7} 14 Oct".
    pub label: String,
    /// 0-based, inclusive.
    pub start: u32,
    pub end: u32,
    /// The cited lines as they were sent.
    pub quote: String,
}

/// A passage found for a question the notes do not answer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AskClosest {
    pub note_id: String,
    pub title: String,
    pub start: u32,
    pub end: u32,
}

/// The time a question was understood to be about.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AskTime {
    /// "Written October 2026".
    pub label: String,
    /// The line that says the range was relaxed; empty when it wasn't.
    pub relaxed_line: String,
}

/// Where a receipt's lines are now, 0-based and inclusive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AskLines {
    pub start: u32,
    pub end: u32,
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
    /// The time the question was understood to be about.
    Time { time: AskTime },
    /// A note the earlier exchanges used is now kept out of AI, so they were
    /// forgotten: drop them from the screen and say `line`.
    Restarted { line: String },
    /// The answer is complete and its receipts were checked. `notes` are all
    /// the notes it drew on; `also_read` the ones it did not cite.
    Done {
        markdown: String,
        notes: Vec<AskNote>,
        receipts: Vec<AskReceipt>,
        also_read: Vec<AskNote>,
        /// The banner over the answer; empty when there is none.
        banner: String,
        /// How many sentences are marked as not found.
        unverified: u32,
        /// How many sentences cite nothing in an answer that cites elsewhere.
        no_source: u32,
        /// The notes do not hold the answer: "You never wrote that down."
        never_wrote: bool,
        closest: Vec<AskClosest>,
        time: Option<AskTime>,
    },
    /// Nothing in the notes matches the question, so no model was asked.
    /// `narrow` is whether the scope was narrower than all notes.
    NothingMatches { message: String, narrow: bool },
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
