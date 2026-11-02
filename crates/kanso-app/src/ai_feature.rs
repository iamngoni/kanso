//! The model behind the AI features besides Ask: the inline assistant and
//! the note summary.
//!
//! A feature asks the backend chosen in Settings (a model with an API key,
//! or an agent on this computer) for one thing at a time. It sends its own
//! prompt, the selection or the one note it works on, and nothing more: no
//! tools, no notes server, no search. An agent is given a session with no
//! MCP server for it, so it cannot read a note the prompt did not carry.

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use gpui_kit::App;
use kanso_ai::{ClientSettings, Event, Prompt, SharedProvider};
use kanso_store::Store;

use crate::config::{AiBackend, AiSettings};
use crate::services::Services;
use crate::workspace::AppState;

/// What a feature says when the note it would work on is out of reach of AI.
pub const KEPT_OUT: &str = "This note is kept out of AI.";

/// Makes the provider a feature asks. It reads the Keychain and builds a
/// client, so it runs on the feature's worker thread, never on the UI thread.
pub type Maker = Arc<dyn Fn() -> Result<SharedProvider, String> + Send + Sync>;

/// Whether note `id` may be sent to AI: it is not locked, not in the trash
/// and not tagged `#ai-exclude`. The same decision Ask makes.
pub fn eligible(store: &Store, id: &str) -> bool {
    crate::ask::note_is_eligible(store, id)
}

/// How a feature started now gets its provider, from the settings as they
/// are now. Call it on the UI thread and run the result on a worker.
pub fn maker(cx: &App) -> Maker {
    #[cfg(test)]
    if let Some(injected) = cx.try_global::<testing::Injected>() {
        return injected.0.clone();
    }
    let ai = cx.global::<AppState>().config.ai.clone();
    let data_dir = cx.global::<AppState>().data_dir.clone();
    let secrets = cx
        .try_global::<Services>()
        .and_then(|services| services.secrets.clone());
    Arc::new(move || provider(&ai, &data_dir, secrets.as_deref()))
}

/// The provider for the backend `ai` names. A feature never gets tools from
/// it: features call `stream`, which carries none.
fn provider(
    ai: &AiSettings,
    data_dir: &Path,
    secrets: Option<&dyn kanso_sync::SecretStore>,
) -> Result<SharedProvider, String> {
    match ai.backend {
        AiBackend::Agent => Ok(kanso_ai::agent::agent_client(
            &crate::ai_agent::agent_settings(&ai.agent, data_dir),
        )),
        AiBackend::Api => {
            let saved =
                secrets.and_then(|secrets| crate::settings_ai::read_from(secrets, ai.provider));
            let settings = ClientSettings {
                provider: ai.provider,
                base_url: ai.base_url.clone(),
                model: ai.model.clone(),
                api_key: saved,
                region: ai.provider.default_region().to_owned(),
            };
            kanso_ai::cached_client(&settings).map_err(|err| err.to_string())
        }
    }
}

/// Asks `prompt` and hands the reply's events to `on_event` until it ends,
/// fails or `cancel` is set. The provider is held only to start the request.
pub fn stream(
    maker: &Maker,
    prompt: &Prompt,
    cancel: &Arc<AtomicBool>,
    mut on_event: impl FnMut(Event),
) {
    let shared = match maker() {
        Ok(shared) => shared,
        Err(message) => {
            on_event(Event::Error(message));
            return;
        }
    };
    let events = {
        let provider = shared
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        provider.stream(prompt)
    };
    kanso_ai::features::relay(events, cancel, &mut on_event);
}

/// Summarises one note (`text`, titled `title`), long ones in parts.
pub fn summarize(
    maker: &Maker,
    title: &str,
    text: &str,
    cancel: &Arc<AtomicBool>,
    mut on_event: impl FnMut(Event),
) {
    let shared = match maker() {
        Ok(shared) => shared,
        Err(message) => {
            on_event(Event::Error(message));
            return;
        }
    };
    let provider = shared
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    kanso_ai::features::summarize(provider.as_ref(), title, text, cancel, &mut on_event);
}

/// Refines an Inbox triage card with the model (issue #195): it ranks the
/// library's candidates and may suggest a title, and nothing else. The note
/// is checked for eligibility first; the error is a plain reason.
#[allow(clippy::too_many_arguments)]
pub fn triage(
    maker: &Maker,
    store: &Store,
    card: &kanso_engine::triage::Card,
    title: &str,
    text: &str,
    opts: &kanso_engine::triage::TriageOptions,
    cancel: &Arc<AtomicBool>,
) -> Result<Vec<kanso_engine::triage::Suggestion>, String> {
    let shared = maker()?;
    let provider = shared
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    kanso_engine::triage::ai::refine(store, provider.as_ref(), card, title, text, opts, cancel)
}

/// Tidies a voice transcript (issue #193): a title, paragraphs, and decisions
/// and tasks the transcript backs with quotes. It sends the transcript and a
/// title hint, nothing else. The error is a plain reason.
pub fn tidy(
    maker: &Maker,
    kind: kanso_ai::tidy::TidyKind,
    title_hint: &str,
    transcript: &str,
    cancel: &Arc<AtomicBool>,
) -> Result<kanso_ai::tidy::Tidy, String> {
    let shared = maker()?;
    let provider = shared
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    kanso_ai::tidy::tidy(provider.as_ref(), kind, title_hint, transcript, cancel)
}

/// Fakes for the tests of the features: a provider that records what it was
/// sent, an agent in process, and the hook that hands either to a feature.
#[cfg(test)]
pub(crate) mod testing {
    use std::sync::Mutex;

    use gpui_kit::Global;
    use kanso_ai::{AiError, ChatProvider};

    use super::*;

    /// The provider every feature gets in a test, in place of the settings'.
    pub(crate) struct Injected(pub(crate) Maker);

    impl Global for Injected {}

    /// A model with an API key that says `reply` and keeps what it was sent.
    pub(crate) struct Recorder {
        reply: String,
        pub(crate) prompts: Mutex<Vec<Prompt>>,
        /// Times it was asked with tools or history (a feature never does).
        pub(crate) tool_asks: Mutex<usize>,
    }

    impl Recorder {
        pub(crate) fn new(reply: &str) -> Arc<Recorder> {
            Arc::new(Recorder {
                reply: reply.to_owned(),
                prompts: Mutex::new(Vec::new()),
                tool_asks: Mutex::new(0),
            })
        }

        /// A maker handing out this recorder.
        pub(crate) fn maker(self: &Arc<Self>) -> Maker {
            let recorder = self.clone();
            Arc::new(move || {
                let shared: SharedProvider =
                    Arc::new(Mutex::new(Box::new(Handle(recorder.clone()))));
                Ok(shared)
            })
        }

        pub(crate) fn sent(&self) -> Vec<Prompt> {
            self.prompts.lock().unwrap().clone()
        }
    }

    struct Handle(Arc<Recorder>);

    impl ChatProvider for Handle {
        fn stream(
            &self,
            prompt: &Prompt,
        ) -> Result<Box<dyn Iterator<Item = Event> + Send>, AiError> {
            self.0.prompts.lock().unwrap().push(prompt.clone());
            let reply = self.0.reply.clone();
            let mid = reply
                .char_indices()
                .nth(reply.chars().count() / 2)
                .map_or(0, |(at, _)| at);
            Ok(Box::new(
                vec![
                    Event::Text(reply[..mid].to_owned()),
                    Event::Text(reply[mid..].to_owned()),
                    Event::Done,
                ]
                .into_iter(),
            ))
        }

        fn ask(
            &self,
            prompt: &Prompt,
            _history: &[kanso_ai::Turn],
            _tools: kanso_ai::ToolRegistry<()>,
            _on_tool_completed: Box<dyn FnMut() + Send>,
            _cancel: Arc<AtomicBool>,
        ) -> Result<Box<dyn Iterator<Item = Event> + Send>, AiError> {
            *self.0.tool_asks.lock().unwrap() += 1;
            self.stream(prompt)
        }
    }

    /// An agent on this computer, in process, running `script`. `name` keeps
    /// the warm agent of one test from being another's.
    pub(crate) fn fake_agent(script: kanso_acp::fake::Script, name: &str) -> SharedProvider {
        let settings = kanso_ai::agent::AgentSettings {
            kind: kanso_ai::AgentKind::Custom,
            command: Some(
                std::env::current_exe()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
            ),
            args: Vec::new(),
            env: Vec::new(),
            workspace: std::env::temp_dir().join(format!("kanso-feature-agent-{name}")),
        };
        kanso_ai::agent::agent_client_with(
            &settings,
            kanso_acp::Limits::default(),
            Arc::new(move |spec, limits| {
                let (from_agent, to_agent) = kanso_acp::fake::in_process(script.clone())
                    .map_err(|err| kanso_acp::AcpError::Spawn(err.to_string()))?;
                kanso_acp::AgentProcess::connect(
                    &spec.label,
                    from_agent,
                    to_agent,
                    limits,
                    kanso_acp::reject_all(),
                )
            }),
        )
    }

    /// A maker handing out `provider`.
    pub(crate) fn maker_of(provider: SharedProvider) -> Maker {
        Arc::new(move || Ok(provider.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::testing::{Recorder, fake_agent, maker_of};
    use super::*;

    fn run(maker: &Maker, prompt: &Prompt) -> (String, Vec<Event>) {
        let mut events = Vec::new();
        stream(maker, prompt, &Arc::new(AtomicBool::new(false)), |event| {
            events.push(event)
        });
        let text = events
            .iter()
            .filter_map(|event| match event {
                Event::Text(text) => Some(text.as_str()),
                _ => None,
            })
            .collect();
        (text, events)
    }

    fn prompt() -> Prompt {
        kanso_ai::features::inline_prompt(
            kanso_ai::InlineAction::Improve,
            "Plan",
            "Earlier words.",
            "We ship on friday.",
        )
    }

    /// A feature on a model with an API key sends its own prompt with
    /// `stream`, never `ask` with tools or history.
    // no-prop-test: glue over a fake provider; the prompts have the
    // properties in `kanso_ai::features`.
    #[test]
    fn the_api_path_streams_the_prompt_and_asks_for_no_tools() {
        let recorder = Recorder::new("We ship on Friday.");
        let (text, events) = run(&recorder.maker(), &prompt());
        assert_eq!(text, "We ship on Friday.");
        assert_eq!(events.last(), Some(&Event::Done));
        assert_eq!(recorder.sent(), vec![prompt()]);
        assert_eq!(*recorder.tool_asks.lock().unwrap(), 0);
    }

    /// An agent gets the same prompt, in a session that names no MCP
    /// server: the fake agent that would search Kanso's notes through one
    /// says it has none.
    // no-prop-test: glue over the in-process fake agent.
    #[test]
    fn an_agent_feature_gets_a_session_without_a_notes_server() {
        let agent = fake_agent(kanso_acp::fake::Script::Notes, "no-servers");
        let (text, events) = run(&maker_of(agent), &prompt());
        assert!(text.contains("no notes server"), "{text}");
        assert_eq!(events.last(), Some(&Event::Done));
    }

    /// The agent is sent the feature's own instructions and the text it
    /// works on, and nothing else from the library.
    // no-prop-test: glue over the in-process fake agent.
    #[test]
    fn an_agent_is_sent_the_prompt_and_nothing_else() {
        let agent = fake_agent(kanso_acp::fake::Script::Echo, "echo");
        let (text, _) = run(&maker_of(agent), &prompt());
        assert!(text.starts_with(&prompt().system), "{text}");
        assert!(text.contains("We ship on friday."), "{text}");
        assert!(text.ends_with(&prompt().user), "{text}");
    }

    fn ai_without_its_agent(backend: AiBackend) -> AiSettings {
        let mut ai = AiSettings::for_provider(kanso_ai::Provider::LocalModel);
        ai.enabled = true;
        ai.backend = backend;
        ai.agent.kind = kanso_ai::AgentKind::Custom;
        ai.agent.command = Some("/no/such/kanso-removed-agent".to_owned());
        ai
    }

    /// With the backend on an API key, an agent that was uninstalled is
    /// never looked at: the provider is made, and no error is raised.
    // no-prop-test: glue over the settings; no input to vary.
    #[test]
    fn an_uninstalled_agent_does_not_touch_the_api_path() {
        let ai = ai_without_its_agent(AiBackend::Api);
        let made = provider(&ai, &std::env::temp_dir(), None);
        assert!(made.is_ok(), "{:?}", made.err());
    }

    /// With the backend on an agent that is gone, a feature gets one plain
    /// line that points to Settings, not a crash or a hang.
    // no-prop-test: glue over the settings; no input to vary.
    #[test]
    fn a_missing_agent_is_a_line_that_points_to_settings() {
        let ai = ai_without_its_agent(AiBackend::Agent);
        let data_dir = std::env::temp_dir().join("kanso-removed-agent-test");
        let maker: Maker = Arc::new(move || provider(&ai, &data_dir, None));
        let (text, events) = run(&maker, &prompt());
        assert_eq!(text, "");
        let Some(Event::Error(message)) = events.last() else {
            panic!("no error: {events:?}");
        };
        assert!(message.contains("Kanso can't find"), "{message}");
        assert!(message.contains("Settings › AI"), "{message}");
    }

    #[test]
    fn a_provider_that_cannot_be_made_is_an_error_event() {
        let maker: Maker = Arc::new(|| Err("No key".to_owned()));
        let (_, events) = run(&maker, &prompt());
        assert_eq!(events, vec![Event::Error("No key".to_owned())]);
    }

    /// A note summary goes out as one request on the API path.
    // no-prop-test: glue over a fake provider; `features` has the
    // properties for summaries.
    #[test]
    fn a_summary_is_asked_without_tools() {
        let recorder = Recorder::new("- one point");
        let mut said = String::new();
        summarize(
            &recorder.maker(),
            "Plan",
            "We ship on Friday.",
            &Arc::new(AtomicBool::new(false)),
            |event| {
                if let Event::Text(text) = event {
                    said.push_str(&text);
                }
            },
        );
        assert_eq!(said, "- one point");
        assert_eq!(recorder.sent().len(), 1);
        assert_eq!(*recorder.tool_asks.lock().unwrap(), 0);
    }
}
