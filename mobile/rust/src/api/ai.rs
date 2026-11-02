//! The AI features: settings, Ask your notes, the inline assistant and the
//! note summary.

use crate::frb_generated::StreamSink;
use flutter_rust_bridge::frb;
use kanso_mobile::{
    AiAction as CoreAiAction, AiHandle, AiProviderInfo as CoreAiProviderInfo, AiRun,
    AiSettingsInput as CoreAiSettingsInput, AiSettingsView as CoreAiSettingsView, AiSummary,
    AskClosest as CoreAskClosest, AskConversation, AskEvent as CoreAskEvent,
    AskNote as CoreAskNote, AskReceipt as CoreAskReceipt, AskScopeInput as CoreAskScopeInput,
    AskScopeOption as CoreAskScopeOption, AskTime as CoreAskTime, AssistEvent as CoreAssistEvent,
    KansoError, LoopScan,
};

/// A provider the AI settings page can offer. This bridge view contains no
/// credentials; key presence is represented separately by `has_key`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AiProviderInfo {
    pub key: String,
    pub label: String,
    pub description: String,
    pub default_base_url: String,
    pub default_model: String,
    pub default_region: String,
    pub needs_key: bool,
    pub accepts_key: bool,
}

impl From<CoreAiProviderInfo> for AiProviderInfo {
    fn from(info: CoreAiProviderInfo) -> Self {
        AiProviderInfo {
            key: info.key,
            label: info.label,
            description: info.description,
            default_base_url: info.default_base_url,
            default_model: info.default_model,
            default_region: info.default_region,
            needs_key: info.needs_key,
            accepts_key: info.accepts_key,
        }
    }
}

/// One action the inline assistant can offer for selected text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AiAction {
    pub key: String,
    pub label: String,
    pub noun: String,
    pub replaces: bool,
}

impl From<CoreAiAction> for AiAction {
    fn from(action: CoreAiAction) -> Self {
        AiAction {
            key: action.key,
            label: action.label,
            noun: action.noun,
            replaces: action.replaces,
        }
    }
}

/// AI preferences exposed to Settings. The API key itself never crosses the
/// bridge; only whether one is stored is reported.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AiSettingsView {
    pub enabled: bool,
    pub provider: String,
    pub base_url: String,
    pub model: String,
    pub region: String,
    pub has_key: bool,
    pub host: String,
    pub is_remote: bool,
    pub ask_budget: String,
    pub budget_words_line: String,
    pub budget_cost_line: String,
    pub check_sources: bool,
    pub use_links: bool,
}

impl From<CoreAiSettingsView> for AiSettingsView {
    fn from(view: CoreAiSettingsView) -> Self {
        AiSettingsView {
            enabled: view.enabled,
            provider: view.provider,
            base_url: view.base_url,
            model: view.model,
            region: view.region,
            has_key: view.has_key,
            host: view.host,
            is_remote: view.is_remote,
            ask_budget: view.ask_budget,
            budget_words_line: view.budget_words_line,
            budget_cost_line: view.budget_cost_line,
            check_sources: view.check_sources,
            use_links: view.use_links,
        }
    }
}

/// AI settings accepted by the phone. Credentials use `KansoAi.set_key` and
/// remain in the platform secret store.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AiSettingsInput {
    pub enabled: bool,
    pub provider: String,
    pub base_url: String,
    pub model: String,
    pub region: String,
    pub ask_budget: String,
    pub check_sources: bool,
    pub use_links: bool,
}

impl From<AiSettingsInput> for CoreAiSettingsInput {
    fn from(input: AiSettingsInput) -> Self {
        CoreAiSettingsInput {
            enabled: input.enabled,
            provider: input.provider,
            base_url: input.base_url,
            model: input.model,
            region: input.region,
            ask_budget: input.ask_budget,
            check_sources: input.check_sources,
            use_links: input.use_links,
        }
    }
}

/// What a question is limited to: `all`, `book` (notebook ID) or `tag`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AskScopeInput {
    pub kind: String,
    pub value: String,
}

impl From<AskScopeInput> for CoreAskScopeInput {
    fn from(scope: AskScopeInput) -> Self {
        CoreAskScopeInput {
            kind: scope.kind,
            value: scope.value,
        }
    }
}

/// One choice in the scope sheet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AskScopeOption {
    pub kind: String,
    pub value: String,
    pub label: String,
}

impl From<CoreAskScopeOption> for AskScopeOption {
    fn from(option: CoreAskScopeOption) -> Self {
        AskScopeOption {
            kind: option.kind,
            value: option.value,
            label: option.label,
        }
    }
}

/// The lines an answer's sentence rests on, as they were sent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AskReceipt {
    pub n: u32,
    pub note_id: String,
    pub title: String,
    pub label: String,
    pub start: u32,
    pub end: u32,
    pub quote: String,
}

impl From<CoreAskReceipt> for AskReceipt {
    fn from(r: CoreAskReceipt) -> Self {
        AskReceipt {
            n: r.n,
            note_id: r.note_id,
            title: r.title,
            label: r.label,
            start: r.start,
            end: r.end,
            quote: r.quote,
        }
    }
}

/// A passage found for a question the notes do not answer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AskClosest {
    pub note_id: String,
    pub title: String,
    pub start: u32,
    pub end: u32,
}

impl From<CoreAskClosest> for AskClosest {
    fn from(c: CoreAskClosest) -> Self {
        AskClosest {
            note_id: c.note_id,
            title: c.title,
            start: c.start,
            end: c.end,
        }
    }
}

/// The time a question was understood to be about.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AskTime {
    pub label: String,
    pub relaxed_line: String,
}

impl From<CoreAskTime> for AskTime {
    fn from(t: CoreAskTime) -> Self {
        AskTime {
            label: t.label,
            relaxed_line: t.relaxed_line,
        }
    }
}

/// A title and ID for a note the answer used; note text never crosses the
/// bridge through citations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AskNote {
    pub id: String,
    pub title: String,
}

impl From<CoreAskNote> for AskNote {
    fn from(note: CoreAskNote) -> Self {
        AskNote {
            id: note.id,
            title: note.title,
        }
    }
}

/// Ask status, citations and streamed Markdown, with note content limited to
/// the answer the person asked for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AskEvent {
    Searching,
    Used {
        notes: Vec<AskNote>,
    },
    Status {
        line: String,
    },
    Answer {
        markdown: String,
    },
    Time {
        time: AskTime,
    },
    Done {
        markdown: String,
        notes: Vec<AskNote>,
        receipts: Vec<AskReceipt>,
        also_read: Vec<AskNote>,
        banner: String,
        unverified: u32,
        never_wrote: bool,
        closest: Vec<AskClosest>,
        time: Option<AskTime>,
    },
    NothingMatches {
        message: String,
        narrow: bool,
    },
    CutOff {
        message: String,
    },
    Failed {
        message: String,
    },
    Stopped,
}

impl From<CoreAskEvent> for AskEvent {
    fn from(event: CoreAskEvent) -> Self {
        match event {
            CoreAskEvent::Searching => AskEvent::Searching,
            CoreAskEvent::Used { notes } => AskEvent::Used {
                notes: notes.into_iter().map(AskNote::from).collect(),
            },
            CoreAskEvent::Status { line } => AskEvent::Status { line },
            CoreAskEvent::Answer { markdown } => AskEvent::Answer { markdown },
            CoreAskEvent::Time { time } => AskEvent::Time { time: time.into() },
            CoreAskEvent::Done {
                markdown,
                notes,
                receipts,
                also_read,
                banner,
                unverified,
                never_wrote,
                closest,
                time,
            } => AskEvent::Done {
                markdown,
                notes: notes.into_iter().map(AskNote::from).collect(),
                receipts: receipts.into_iter().map(AskReceipt::from).collect(),
                also_read: also_read.into_iter().map(AskNote::from).collect(),
                banner,
                unverified,
                never_wrote,
                closest: closest.into_iter().map(AskClosest::from).collect(),
                time: time.map(AskTime::from),
            },
            CoreAskEvent::NothingMatches { message, narrow } => {
                AskEvent::NothingMatches { message, narrow }
            }
            CoreAskEvent::CutOff { message } => AskEvent::CutOff { message },
            CoreAskEvent::Failed { message } => AskEvent::Failed { message },
            CoreAskEvent::Stopped => AskEvent::Stopped,
        }
    }
}

/// A streamed assistant status or suggestion; no note body is returned
/// except the selected editor text passed into the requested operation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssistEvent {
    Status { line: String },
    Text { text: String },
    Done { text: String },
    CutOff { message: String },
    Failed { message: String },
    Stopped,
}

impl From<CoreAssistEvent> for AssistEvent {
    fn from(event: CoreAssistEvent) -> Self {
        match event {
            CoreAssistEvent::Status { line } => AssistEvent::Status { line },
            CoreAssistEvent::Text { text } => AssistEvent::Text { text },
            CoreAssistEvent::Done { text } => AssistEvent::Done { text },
            CoreAssistEvent::CutOff { message } => AssistEvent::CutOff { message },
            CoreAssistEvent::Failed { message } => AssistEvent::Failed { message },
            CoreAssistEvent::Stopped => AssistEvent::Stopped,
        }
    }
}

#[frb(opaque)]
pub struct KansoAi(AiHandle);

impl KansoAi {
    pub(super) fn new(handle: AiHandle) -> KansoAi {
        KansoAi(handle)
    }

    #[frb(sync)]
    pub fn providers(&self) -> Vec<AiProviderInfo> {
        self.0
            .providers()
            .into_iter()
            .map(AiProviderInfo::from)
            .collect()
    }

    #[frb(sync)]
    pub fn actions(&self) -> Vec<AiAction> {
        self.0.actions().into_iter().map(AiAction::from).collect()
    }

    pub fn settings(&self) -> Result<AiSettingsView, KansoError> {
        self.0.settings().map(AiSettingsView::from)
    }

    /// The line about where a question goes; `device` is "this phone" or
    /// "this tablet".
    pub fn privacy_line(&self, device: String) -> Result<String, KansoError> {
        self.0.privacy_line(device)
    }

    pub fn set_settings(&self, input: AiSettingsInput) -> Result<AiSettingsView, KansoError> {
        self.0.set_settings(input.into()).map(AiSettingsView::from)
    }

    /// Saves the key in the phone's secret store; empty removes it.
    pub fn set_key(&self, provider: String, key: String) -> Result<(), KansoError> {
        self.0.set_key(provider, key)
    }

    pub fn test(&self) -> Result<String, KansoError> {
        self.0.test()
    }

    pub fn note_allowed(&self, id: String) -> Result<bool, KansoError> {
        self.0.note_allowed(&id)
    }

    /// What a question can be limited to: all notes, notebooks, tags.
    pub fn ask_scopes(&self) -> Result<Vec<AskScopeOption>, KansoError> {
        self.0
            .ask_scopes()
            .map(|options| options.into_iter().map(AskScopeOption::from).collect())
    }

    /// Reads the recent notes with the model for open loops the rules
    /// missed. Blocks until it is done, on a worker thread; `run` (from
    /// `new_run`) stops it. Saves nothing.
    pub fn find_open_loops(&self, run: &KansoAiRun) -> Result<LoopScan, KansoError> {
        self.0.find_open_loops(&run.0)
    }

    /// A short paragraph over today's brief.
    pub fn summarise_brief(&self, run: &KansoAiRun) -> Result<AiSummary, KansoError> {
        self.0.summarise_brief(&run.0)
    }

    /// A short paragraph over the week from `from` (`YYYY-MM-DD`), or this
    /// week.
    pub fn summarise_review(
        &self,
        from: Option<String>,
        run: &KansoAiRun,
    ) -> Result<AiSummary, KansoError> {
        self.0.summarise_review(from, &run.0)
    }

    #[frb(sync)]
    pub fn agent_tasks(&self) -> super::agent_tasks::KansoAgentTasks {
        super::agent_tasks::KansoAgentTasks(self.0.agent_tasks())
    }

    #[frb(sync)]
    pub fn conversation(&self) -> KansoAskConversation {
        KansoAskConversation(self.0.conversation())
    }

    /// A request to stop, for one `assist` or `summarize`.
    #[frb(sync)]
    pub fn new_run(&self) -> KansoAiRun {
        KansoAiRun(AiRun::new())
    }

    /// The suggestion's events, until it ends or Dart stops listening. A
    /// request that can't start (AI off, a note kept out) is one `Failed`
    /// event with the reason. `run` (from `new_run`) stops it.
    pub fn assist(
        &self,
        id: String,
        action: String,
        before: String,
        selection: String,
        run: &KansoAiRun,
        sink: StreamSink<AssistEvent>,
    ) {
        let refused = sink.clone();
        if let Err(err) = self.0.assist(
            &id,
            &action,
            before,
            selection,
            &run.0,
            Box::new(move |event| sink.add(event.into()).is_ok()),
        ) {
            refused
                .add(AssistEvent::Failed {
                    message: err.to_string(),
                })
                .ok();
        }
    }

    pub fn summarize(&self, id: String, run: &KansoAiRun, sink: StreamSink<AssistEvent>) {
        let refused = sink.clone();
        if let Err(err) = self.0.summarize(
            &id,
            &run.0,
            Box::new(move |event| sink.add(event.into()).is_ok()),
        ) {
            refused
                .add(AssistEvent::Failed {
                    message: err.to_string(),
                })
                .ok();
        }
    }
}

#[frb(opaque)]
pub struct KansoAskConversation(AskConversation);

impl KansoAskConversation {
    /// The answer's events, until it ends or Dart stops listening. A question
    /// that can't start (AI off, no key) is one `Failed` event with the reason.
    pub fn ask(
        &self,
        question: String,
        scope: AskScopeInput,
        ignore_time: bool,
        utc_offset_minutes: i32,
        sink: StreamSink<AskEvent>,
    ) {
        let refused = sink.clone();
        if let Err(err) = self.0.ask(
            question,
            scope.into(),
            ignore_time,
            utc_offset_minutes,
            Box::new(move |event| sink.add(event.into()).is_ok()),
        ) {
            refused
                .add(AskEvent::Failed {
                    message: err.to_string(),
                })
                .ok();
        }
    }

    #[frb(sync)]
    pub fn stop(&self) {
        self.0.stop();
    }

    #[frb(sync)]
    pub fn clear(&self) {
        self.0.clear();
    }

    #[frb(sync)]
    pub fn turn_count(&self) -> u32 {
        self.0.turn_count()
    }

    /// Forgets the newest exchange.
    #[frb(sync)]
    pub fn forget_last(&self) {
        self.0.forget_last();
    }

    /// Saves the newest answer as a note and returns its ID.
    pub fn save_last(&self) -> Result<String, KansoError> {
        self.0.save_last()
    }
}

#[frb(opaque)]
pub struct KansoAiRun(pub(super) AiRun);

impl KansoAiRun {
    #[frb(sync)]
    pub fn stop(&self) {
        self.0.stop();
    }
}
