//! The streaming client: agent-runtime's `Llm`, driven from one small tokio
//! runtime on a named background thread, exposed to the app as a blocking
//! iterator of [`Event`]s.
//!
//! Dropping the iterator aborts its tokio task, which drops the provider
//! request and its response body. Stop therefore cancels immediately, and a
//! server that never answers fails with a typed timeout instead of hanging.

use agent_runtime::{
    ChatMessage, EventSink, Llm, ProviderError, RuntimeEvent, SharedHttpClient, Timeouts,
    ToolRegistry, ToolSessionRequest,
};
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock, mpsc};

use crate::Provider;
use crate::prompt::Prompt;

/// The most tokens an answer may take: room for the inline assistant to
/// give back its whole selection.
const MAX_TOKENS: u32 = 2048;
/// The most tool calls one question may make.
pub const MAX_TOOL_CALLS: usize = 6;
/// What the panel says when a stream stopped without its end marker.
pub const CUT_OFF: &str = "The answer was cut off before it finished.";

/// Something a stream says.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    /// A piece of the answer, to add to what came before.
    Text(String),
    /// A line about what is happening that isn't part of the answer: an
    /// agent starting, still working, or using one of its tools.
    Status(String),
    /// The answer is complete.
    Done,
    /// The answer stopped, with a plain reason.
    Error(String),
}

/// What went wrong before a stream opened, or while it ran.
#[derive(Debug)]
pub enum AiError {
    /// The provider needs a key and none is saved.
    MissingKey,
    /// No model name is set.
    MissingModel,
    /// The base URL is empty.
    MissingUrl,
    /// The server answered with an error.
    Status { status: u16, message: String },
    /// The provider answered with an error in its stream.
    Provider(String),
    /// The request couldn't reach the server.
    Transport(String),
    /// The server did not answer within its limits.
    Timeout(String),
    /// An agent on this computer couldn't answer; a whole sentence.
    Agent(String),
}

impl std::fmt::Display for AiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiError::MissingKey => write!(f, "Add an API key in Settings › AI."),
            AiError::MissingModel => write!(f, "Set a model in Settings › AI."),
            AiError::MissingUrl => write!(f, "Set the server's address in Settings › AI."),
            AiError::Status { status, message } => {
                write!(f, "The model's server answered {status}: {message}")
            }
            AiError::Provider(message) => write!(f, "The model reported an error: {message}"),
            AiError::Transport(message) => write!(f, "Can't reach the model's server: {message}"),
            AiError::Timeout(message) => {
                write!(f, "The model's server didn't answer in time ({message}).")
            }
            AiError::Agent(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for AiError {}

/// One earlier exchange kept by the panel for its lifetime: the question and
/// the answer the person saw. It is never written to disk.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Turn {
    pub question: String,
    pub answer: String,
}

/// One provider, ready to answer. Everything above this trait (retrieval, the
/// panel, the tools) sees only this, so a later ACP-agent backend can answer
/// instead without those features changing.
pub trait ChatProvider: Send {
    /// Asks `prompt`, returning the answer as it arrives. Dropping the
    /// iterator stops the request.
    fn stream(&self, prompt: &Prompt) -> Result<Box<dyn Iterator<Item = Event> + Send>, AiError>;

    /// Asks `prompt` as the next turn of `history`, with `tools` available.
    /// `on_tool_completed` runs after each tool call, on the runtime thread,
    /// so the panel can refresh the notes it has shown as used. The default
    /// keeps single-turn providers working by ignoring history and tools.
    fn ask(
        &self,
        prompt: &Prompt,
        _history: &[Turn],
        _tools: agent_runtime::ToolRegistry<()>,
        _on_tool_completed: Box<dyn FnMut() + Send>,
        _cancel: Arc<AtomicBool>,
    ) -> Result<Box<dyn Iterator<Item = Event> + Send>, AiError> {
        self.stream(prompt)
    }
}

/// What the app's settings decide a request uses.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientSettings {
    pub provider: Provider,
    pub base_url: String,
    pub model: String,
    pub api_key: Option<String>,
    /// The AWS region, for Bedrock only.
    pub region: String,
}

impl ClientSettings {
    fn validate(&self) -> Result<(), AiError> {
        if self.model.trim().is_empty() {
            return Err(AiError::MissingModel);
        }
        if self.provider != Provider::Bedrock && self.base_url.trim().is_empty() {
            return Err(AiError::MissingUrl);
        }
        if self.provider.needs_key()
            && self
                .api_key
                .as_deref()
                .is_none_or(|key| key.trim().is_empty())
        {
            return Err(AiError::MissingKey);
        }
        Ok(())
    }

    fn llm_builder(&self) -> agent_runtime::LlmBuilder {
        let mut builder = Llm::builder()
            .provider(self.provider.agent_kind())
            .api_key(self.api_key.clone().unwrap_or_default())
            .max_tokens(MAX_TOKENS)
            .max_completion_tokens(MAX_TOKENS as u64)
            .timeouts(Timeouts::default());
        if self.provider == Provider::Bedrock {
            if !self.region.trim().is_empty() {
                builder = builder.region(self.region.trim());
            }
        } else if !self.base_url.trim().is_empty() {
            builder = builder.base_url(self.base_url.trim());
        }
        builder
    }

    fn llm(&self) -> Result<Llm, AiError> {
        self.validate()?;
        let builder = self.llm_builder();
        // A phone trusts its own certificate store, not the web's roots
        // built into the program. A model on this network over plain http
        // has no certificate to check.
        #[cfg(any(target_os = "ios", target_os = "android"))]
        let builder = if self.provider == Provider::Bedrock
            || self
                .base_url
                .trim()
                .to_ascii_lowercase()
                .starts_with("https:")
        {
            builder.reqwest_client(phone_http_client(Timeouts::default())?)
        } else {
            builder
        };
        builder
            .build()
            .map_err(|error| map_error(error, self.api_key.as_deref()))
    }
}

/// The HTTP client on iOS and Android: what the runtime builds by default,
/// except that it checks certificates with the phone (`kanso-tls`). Fails,
/// instead of panicking later, when the Android app hasn't set that up.
#[cfg(any(target_os = "ios", target_os = "android"))]
// no-prop-test: needs the phone's certificate store; kanso-tls has the tests.
fn phone_http_client(timeouts: Timeouts) -> Result<reqwest::Client, AiError> {
    let tls = kanso_tls::client_config().map_err(|err| AiError::Transport(err.to_string()))?;
    let mut builder = reqwest::Client::builder().use_preconfigured_tls((*tls).clone());
    if let Some(connect) = timeouts.connect {
        builder = builder.connect_timeout(connect);
    }
    builder
        .build()
        .map_err(|err| AiError::Transport(err.to_string()))
}

/// A client for `settings`, or why one can't be made.
pub fn client(settings: &ClientSettings) -> Result<Box<dyn ChatProvider>, AiError> {
    settings.validate()?;
    Ok(Box::new(AgentProviderClient {
        llm: settings.llm()?,
        model: settings.model.clone(),
        api_key: settings.api_key.clone().unwrap_or_default(),
        tools_key: tools_key(settings),
    }))
}

/// A provider shared by every question using the same settings. The client
/// is built once (a reqwest client, its pool and TLS roots) and reused until
/// the provider, URL or model changes.
pub type SharedProvider = Arc<Mutex<Box<dyn ChatProvider>>>;

/// The client for `settings`, reused from the last call when the settings are
/// unchanged. Building one on the UI thread for every question is what this
/// avoids; call it from the question's background work.
pub fn cached_client(settings: &ClientSettings) -> Result<SharedProvider, AiError> {
    static CACHE: OnceLock<Mutex<Option<(ClientSettings, SharedProvider)>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(None));
    let mut cache = cache
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some((cached, provider)) = cache.as_ref()
        && cached == settings
    {
        return Ok(provider.clone());
    }
    let provider: SharedProvider = Arc::new(Mutex::new(client(settings)?));
    *cache = Some((settings.clone(), provider.clone()));
    Ok(provider)
}

/// A client over a specific `HttpClient`, for tests and advanced embedders.
pub fn client_with(
    settings: &ClientSettings,
    http: SharedHttpClient,
) -> Result<Box<dyn ChatProvider>, AiError> {
    client_with_timeouts(settings, http, Timeouts::default())
}

/// [`client_with`] with explicit timeouts.
pub fn client_with_timeouts(
    settings: &ClientSettings,
    http: SharedHttpClient,
    timeouts: Timeouts,
) -> Result<Box<dyn ChatProvider>, AiError> {
    settings.validate()?;
    let llm = settings
        .llm_builder()
        .http_client(http)
        .timeouts(timeouts)
        .build()
        .map_err(|error| map_error(error, settings.api_key.as_deref()))?;
    Ok(Box::new(AgentProviderClient {
        llm,
        model: settings.model.clone(),
        api_key: settings.api_key.clone().unwrap_or_default(),
        tools_key: tools_key(settings),
    }))
}

/// Asks a tiny question and returns the first word of the answer, so Settings
/// can say the settings work.
pub fn probe(settings: &ClientSettings) -> Result<String, AiError> {
    let provider = client(settings)?;
    let prompt = Prompt {
        system: "You answer with one short word.".to_owned(),
        user: "Reply with OK.".to_owned(),
    };
    let mut stream = provider.stream(&prompt)?;
    let mut said = String::new();
    for event in stream.by_ref().take(200) {
        match event {
            Event::Text(text) => {
                said.push_str(&text);
                if said.trim().chars().count() >= 2 {
                    return Ok(said.trim().to_owned());
                }
            }
            Event::Status(_) => {}
            Event::Done => break,
            Event::Error(message) => return Err(AiError::Provider(message)),
        }
    }
    let said = said.trim().to_owned();
    if said.is_empty() {
        Err(AiError::Transport(
            "The model answered without any text.".to_owned(),
        ))
    } else {
        Ok(said)
    }
}

/// Asks a question with tools: the initial excerpts are in `system`/`user`,
/// `history` holds earlier exchanges, and the model may call `tools` before
/// its final answer. Tool calls are capped at [`MAX_TOOL_CALLS`].
pub fn answer(
    settings: &ClientSettings,
    system: &str,
    user: &str,
    history: &[Turn],
    tools: ToolRegistry<()>,
    on_tool_completed: impl FnMut() + Send + 'static,
) -> Result<Box<dyn Iterator<Item = Event> + Send>, AiError> {
    let provider = client(settings)?;
    provider.ask(
        &Prompt {
            system: system.to_owned(),
            user: user.to_owned(),
        },
        history,
        tools,
        Box::new(on_tool_completed),
        Arc::new(AtomicBool::new(false)),
    )
}

/// The chat messages sent for a turn: earlier questions and answers, then the
/// new user message carrying this turn's excerpts and question. Earlier
/// excerpts are deliberately not resent; retrieval runs fresh for each turn.
fn chat_messages(history: &[Turn], user: &str) -> Vec<ChatMessage> {
    let mut messages = Vec::with_capacity(history.len() * 2 + 1);
    for turn in history {
        if !turn.question.trim().is_empty() {
            messages.push(ChatMessage::user(turn.question.clone()));
        }
        if !turn.answer.trim().is_empty() {
            messages.push(ChatMessage::assistant(turn.answer.clone()));
        }
    }
    messages.push(ChatMessage::user(user.to_owned()));
    messages
}

/// A streamed answer bound to its tokio task. Dropping it aborts that task,
/// which cancels the request.
struct EventStream {
    rx: mpsc::Receiver<Event>,
    task: Option<tokio::task::JoinHandle<()>>,
    cancel: Arc<AtomicBool>,
    /// A Done or Error has already been yielded (or the disconnect was
    /// already reported), so the iterator is finished.
    final_event: bool,
}

impl Iterator for EventStream {
    type Item = Event;

    fn next(&mut self) -> Option<Event> {
        loop {
            // Waiting with a short timeout lets Stop abort the request even
            // while a silent server sends nothing at all.
            match self.rx.recv_timeout(std::time::Duration::from_millis(25)) {
                Ok(event) => {
                    if matches!(event, Event::Done | Event::Error(_)) {
                        self.final_event = true;
                    }
                    return Some(event);
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    // The task ended without saying how the answer ended (it
                    // panicked or was dropped). Report once, so a caller
                    // waiting on this stream cannot hang forever.
                    if !self.final_event {
                        self.final_event = true;
                        return Some(Event::Error(CUT_OFF.to_owned()));
                    }
                    return None;
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if self.cancel.load(Ordering::Relaxed) {
                        if let Some(task) = self.task.take() {
                            task.abort();
                        }
                        return None;
                    }
                }
            }
        }
    }
}

impl Drop for EventStream {
    fn drop(&mut self) {
        if let Some(task) = self.task.take() {
            task.abort();
        }
    }
}

/// The identity a tool-support decision belongs to. A different provider,
/// endpoint or model is a different session.
fn tools_key(settings: &ClientSettings) -> String {
    format!(
        "{}|{}|{}",
        settings.provider.key(),
        settings.base_url.trim(),
        settings.model.trim()
    )
}

fn unsupported_tools() -> &'static Mutex<HashSet<String>> {
    static UNSUPPORTED: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    UNSUPPORTED.get_or_init(|| Mutex::new(HashSet::new()))
}

fn tools_are_supported(key: &str) -> bool {
    !unsupported_tools()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .contains(key)
}

fn remember_tools_unsupported(key: &str) {
    unsupported_tools()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .insert(key.to_owned());
}

/// Whether a provider rejected the request only because it cannot use tools.
/// Ollama says "<model> does not support tools"; OpenAI-compatible servers
/// phrase it around tools or functions.
fn tool_calls_unsupported(error: &anyhow::Error) -> bool {
    let Some(ProviderError::Status { status, body, .. }) = error
        .chain()
        .find_map(|error| error.downcast_ref::<ProviderError>())
    else {
        return false;
    };
    if *status != 400 {
        return false;
    }
    let body = body.to_ascii_lowercase();
    let mentions = body.contains("tool") || body.contains("function");
    let rejects = body.contains("does not support")
        || body.contains("doesn't support")
        || body.contains("not supported")
        || body.contains("unsupported");
    mentions && rejects
}

/// One provider, ready to answer a prompt without tools.
struct AgentProviderClient {
    llm: Llm,
    model: String,
    /// Used only to remove credentials echoed by a provider from its errors.
    api_key: String,
    /// Which provider + endpoint + model this client's tool support belongs
    /// to, so a server without tools is remembered for this session only.
    tools_key: String,
}

impl ChatProvider for AgentProviderClient {
    fn stream(&self, prompt: &Prompt) -> Result<Box<dyn Iterator<Item = Event> + Send>, AiError> {
        let llm = self.llm.clone();
        let system = prompt.system.clone();
        let user = prompt.user.clone();
        run_on_runtime(
            self.model.clone(),
            self.api_key.clone(),
            Arc::new(AtomicBool::new(false)),
            Box::new(|| {}),
            move |model, mut sink| async move {
                let messages = [ChatMessage::user(user)];
                llm.stream_message(&model, &system, &messages, &mut sink)
                    .await
                    .map(|_| ())
            },
        )
    }

    fn ask(
        &self,
        prompt: &Prompt,
        history: &[Turn],
        tools: ToolRegistry<()>,
        on_tool_completed: Box<dyn FnMut() + Send>,
        cancel: Arc<AtomicBool>,
    ) -> Result<Box<dyn Iterator<Item = Event> + Send>, AiError> {
        let llm = self.llm.clone();
        let system = prompt.system.clone();
        let messages = chat_messages(history, &prompt.user);
        let tools_key = self.tools_key.clone();
        run_on_runtime(
            self.model.clone(),
            self.api_key.clone(),
            cancel,
            on_tool_completed,
            move |model, mut sink| async move {
                if !tools_are_supported(&tools_key) {
                    return llm
                        .stream_message(&model, &system, &messages, &mut sink)
                        .await
                        .map(|_| ());
                }
                let outcome = llm
                    .execute_tool_session(
                        ToolSessionRequest {
                            model: &model,
                            decision_system_prompt: &system,
                            followup_system_prompt: &system,
                            history: &messages,
                            tool_registry: &tools,
                            tool_context: (),
                            max_tool_calls: MAX_TOOL_CALLS,
                        },
                        &mut sink,
                    )
                    .await;
                match outcome {
                    Ok(_) => Ok(()),
                    Err(err) if tool_calls_unsupported(&err) => {
                        // Ollama models without tool support (gemma,
                        // deepseek-r1) and some OpenAI-compatible servers
                        // reject the tool definitions outright. Answer from
                        // the same excerpts without tools, and remember it
                        // for this provider + endpoint + model.
                        remember_tools_unsupported(&tools_key);
                        llm.stream_message(&model, &system, &messages, &mut sink)
                            .await
                            .map(|_| ())
                    }
                    Err(err) => Err(err),
                }
            },
        )
    }
}

/// Runs `call` on the AI runtime and returns its events as a blocking
/// iterator.
fn run_on_runtime<F, Fut>(
    model: String,
    api_key: String,
    cancel: Arc<AtomicBool>,
    on_tool_completed: impl FnMut() + Send + 'static,
    call: F,
) -> Result<Box<dyn Iterator<Item = Event> + Send>, AiError>
where
    F: FnOnce(String, ChannelSink) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = anyhow::Result<()>> + Send,
{
    let runtime = runtime()?;
    let (tx, rx) = mpsc::channel();
    let finished = Arc::new(AtomicBool::new(false));
    let sink = ChannelSink {
        tx,
        cancel: cancel.clone(),
        on_tool: Box::new(on_tool_completed),
        finished: finished.clone(),
    };
    let task = runtime.spawn(async move {
        let tx = sink.tx.clone();
        let result = call(model, sink).await;
        if !finished.load(Ordering::Relaxed) {
            finished.store(true, Ordering::Relaxed);
            let event = match result {
                Ok(()) => Event::Done,
                Err(err) => Event::Error(error_text(&err, &api_key)),
            };
            tx.send(event).ok();
        }
    });
    Ok(Box::new(EventStream {
        rx,
        task: Some(task),
        cancel,
        final_event: false,
    }))
}

/// Maps agent-runtime's events onto Kanso's, aborting when `cancel` is set.
struct ChannelSink {
    tx: mpsc::Sender<Event>,
    cancel: Arc<AtomicBool>,
    on_tool: Box<dyn FnMut() + Send>,
    finished: Arc<AtomicBool>,
}

#[async_trait::async_trait]
impl EventSink for ChannelSink {
    async fn emit(&mut self, event: RuntimeEvent) -> anyhow::Result<()> {
        if self.cancel.load(Ordering::Relaxed) {
            anyhow::bail!("cancelled");
        }
        match event {
            RuntimeEvent::AssistantDelta { delta } => {
                self.tx.send(Event::Text(delta)).ok();
            }
            RuntimeEvent::AssistantEnded { complete } => {
                self.finished.store(true, Ordering::Relaxed);
                let event = if complete {
                    Event::Done
                } else {
                    Event::Error(CUT_OFF.to_owned())
                };
                self.tx.send(event).ok();
            }
            RuntimeEvent::ToolCompleted { .. } => (self.on_tool)(),
            RuntimeEvent::AssistantStarted { .. } | RuntimeEvent::ToolStarted { .. } => {}
        }
        Ok(())
    }
}

/// The one runtime every provider call runs on: a single worker thread owned
/// by this crate, so GPUI's executor is never blocked.
fn runtime() -> Result<&'static tokio::runtime::Runtime, AiError> {
    static RUNTIME: OnceLock<Result<tokio::runtime::Runtime, String>> = OnceLock::new();
    RUNTIME
        .get_or_init(|| {
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(1)
                .thread_name("kanso-ai")
                .enable_all()
                .build()
                .map_err(|err| err.to_string())
        })
        .as_ref()
        .map_err(|err| AiError::Transport(format!("Couldn't start the AI runtime: {err}")))
}

/// A provider error's plain words, without credentials or request bodies.
fn error_text(error: &anyhow::Error, api_key: &str) -> String {
    if let Some(provider) = error
        .chain()
        .find_map(|error| error.downcast_ref::<ProviderError>())
    {
        return redact_provider_error(&provider.to_string(), Some(api_key));
    }
    redact_provider_error(&error.to_string(), Some(api_key))
}

/// Removes a configured API key and any `Authorization` header value from
/// provider error text before it crosses into an application error or event.
pub fn redact_provider_error(message: &str, configured_key: Option<&str>) -> String {
    let message = match configured_key.filter(|key| !key.is_empty()) {
        Some(key) => message.replace(key, "[REDACTED]"),
        None => message.to_owned(),
    };
    redact_authorization_values(&message)
}

/// Redacts plain or quoted Authorization header values while retaining the
/// header name and surrounding message.
fn redact_authorization_values(message: &str) -> String {
    const HEADER: &str = "authorization";
    let bytes = message.as_bytes();
    let mut result = String::with_capacity(message.len());
    let mut copied_until = 0;
    let mut cursor = 0;

    while cursor + HEADER.len() <= bytes.len() {
        if !message
            .get(cursor..cursor + HEADER.len())
            .is_some_and(|candidate| candidate.eq_ignore_ascii_case(HEADER))
        {
            cursor += 1;
            continue;
        }

        let mut value_start = cursor + HEADER.len();
        if bytes.get(value_start) == Some(&b'"') {
            value_start += 1;
        }
        while bytes.get(value_start).is_some_and(u8::is_ascii_whitespace) {
            value_start += 1;
        }
        if bytes.get(value_start) != Some(&b':') {
            cursor += HEADER.len();
            continue;
        }
        value_start += 1;
        while bytes.get(value_start).is_some_and(u8::is_ascii_whitespace) {
            value_start += 1;
        }
        let quoted = bytes.get(value_start) == Some(&b'"');
        if quoted {
            value_start += 1;
        }
        let mut value_end = value_start;
        while let Some(byte) = bytes.get(value_end) {
            if (quoted && *byte == b'"') || (!quoted && matches!(*byte, b'\r' | b'\n')) {
                break;
            }
            value_end += 1;
        }

        result.push_str(&message[copied_until..value_start]);
        result.push_str("[REDACTED]");
        copied_until = value_end;
        cursor = value_end;
    }
    result.push_str(&message[copied_until..]);
    result
}

fn map_error(error: anyhow::Error, api_key: Option<&str>) -> AiError {
    let Some(provider) = error
        .chain()
        .find_map(|error| error.downcast_ref::<ProviderError>())
    else {
        return AiError::Transport(redact_provider_error(&error_text(&error, ""), api_key));
    };
    match provider {
        ProviderError::Status { status, body, .. } => AiError::Status {
            status: *status,
            message: crate::prompt::truncate(&redact_provider_error(body.trim(), api_key), 300),
        },
        ProviderError::Timeout { stage, after, .. } => {
            AiError::Timeout(format!("{} after {}s", stage.as_str(), after.as_secs_f64()))
        }
        ProviderError::Transport { source, .. } => {
            AiError::Transport(redact_provider_error(&source.to_string(), api_key))
        }
        other => AiError::Provider(redact_provider_error(&other.to_string(), api_key)),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::{Duration, Instant};

    use agent_runtime::{
        HttpClient, HttpRequest, HttpResponse, HttpStreamResponse, Timeouts, Tool, ToolCall,
        ToolDefinition, ToolOutput, ToolRegistry,
    };
    use async_trait::async_trait;
    use futures_util::stream;

    use super::{
        AiError, CUT_OFF, ClientSettings, Event, ProviderError, client_with_timeouts, map_error,
        redact_provider_error,
    };
    use crate::Provider;
    use crate::prompt::Prompt;

    const OPENAI: &str = include_str!("../tests/fixtures/openai.sse");
    const ANTHROPIC: &str = include_str!("../tests/fixtures/anthropic.sse");
    const ANTHROPIC_ERROR: &str = include_str!("../tests/fixtures/anthropic_error.sse");

    #[test]
    fn provider_errors_redact_the_configured_key_and_authorization_header() {
        assert_eq!(
            redact_provider_error("rejected key test-secret", Some("test-secret")),
            "rejected key [REDACTED]"
        );
        assert_eq!(
            redact_provider_error("Authorization: Bearer header-secret", None),
            "Authorization: [REDACTED]"
        );
        let error = anyhow::Error::new(ProviderError::from_status(
            "test",
            400,
            "Provider rejected key test-secret",
        ));
        assert!(matches!(
            map_error(error, Some("test-secret")),
            AiError::Status { message, .. }
                if message == "Provider rejected key [REDACTED]"
        ));
    }

    #[test]
    fn provider_error_redaction_leaves_unrelated_messages_untouched() {
        let message = "The model is temporarily unavailable.";
        assert_eq!(redact_provider_error(message, Some("test-secret")), message);
        let error = anyhow::Error::new(ProviderError::from_status("test", 400, message));
        assert!(matches!(
            map_error(error, Some("test-secret")),
            AiError::Status { message: mapped, .. } if mapped == message
        ));
    }

    /// Replays a recorded body in chunks, recording the request.
    #[derive(Clone, Default)]
    struct Replay {
        body: Arc<std::sync::Mutex<Option<String>>>,
        requests: Arc<std::sync::Mutex<Vec<HttpRequest>>>,
    }

    impl Replay {
        fn new(body: &str) -> Self {
            Self {
                body: Arc::new(std::sync::Mutex::new(Some(body.to_owned()))),
                requests: Arc::default(),
            }
        }
    }

    #[async_trait]
    impl HttpClient for Replay {
        async fn send(&self, request: HttpRequest) -> anyhow::Result<HttpResponse> {
            self.requests.lock().unwrap().push(request);
            Ok(HttpResponse {
                status: 200,
                body: b"{\"choices\":[{\"message\":{\"content\":\"ok\"}}]}".to_vec(),
            })
        }

        async fn send_streaming(&self, request: HttpRequest) -> anyhow::Result<HttpStreamResponse> {
            self.requests.lock().unwrap().push(request);
            let body = self.body.lock().unwrap().take().expect("one recorded body");
            // Split at every character so the runtime's decoder has to cope
            // with reads ending anywhere.
            let chunks: Vec<anyhow::Result<Vec<u8>>> = body
                .char_indices()
                .map(|(at, ch)| {
                    let end = at + ch.len_utf8();
                    Ok(body.as_bytes()[at..end].to_vec())
                })
                .collect();
            Ok(HttpStreamResponse {
                status: 200,
                body: Box::pin(stream::iter(chunks)),
            })
        }
    }

    /// Never answers.
    struct Silent;

    #[async_trait]
    impl HttpClient for Silent {
        async fn send(&self, _request: HttpRequest) -> anyhow::Result<HttpResponse> {
            std::future::pending().await
        }

        async fn send_streaming(
            &self,
            _request: HttpRequest,
        ) -> anyhow::Result<HttpStreamResponse> {
            std::future::pending().await
        }
    }

    fn settings(provider: Provider) -> ClientSettings {
        ClientSettings {
            provider,
            base_url: provider.default_base_url().to_owned(),
            model: provider.default_model().to_owned(),
            api_key: Some("test-key".to_owned()),
            region: provider.default_region().to_owned(),
        }
    }

    /// On a phone the client trusts the phone's store: it builds where that
    /// store is ready (iOS), and is refused, not panicked, where it isn't yet
    /// (an Android app before it hands over its Context).
    // no-prop-test: one client built from fixed settings; kanso-tls has the properties.
    #[cfg(any(target_os = "ios", target_os = "android"))]
    #[test]
    fn a_phones_client_uses_the_phones_certificates_once_they_are_ready() {
        let made = super::client(&settings(Provider::OpenAi));
        if cfg!(target_os = "android") {
            assert!(matches!(made, Err(AiError::Transport(_))));
        } else {
            assert!(made.is_ok());
        }
    }

    fn events(provider: Provider, body: &str) -> Vec<Event> {
        let client = client_with_timeouts(
            &settings(provider),
            Arc::new(Replay::new(body)),
            Timeouts::default(),
        )
        .unwrap();
        client
            .stream(&Prompt {
                system: "be brief".into(),
                user: "hello".into(),
            })
            .unwrap()
            .collect()
    }

    fn answer(events: &[Event]) -> String {
        events
            .iter()
            .filter_map(|event| match event {
                Event::Text(text) => Some(text.as_str()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn a_recorded_openai_answer_streams_and_stops_at_done() {
        let events = events(Provider::OpenAi, OPENAI);
        assert_eq!(answer(&events), "Your notes say hello.");
        assert_eq!(events.last(), Some(&Event::Done));
    }

    #[test]
    fn a_recorded_anthropic_answer_streams_to_message_stop() {
        let events = events(Provider::Anthropic, ANTHROPIC);
        assert_eq!(answer(&events), "Your notes say hello.");
        assert_eq!(events.last(), Some(&Event::Done));
    }

    #[test]
    fn an_anthropic_error_is_plain_words() {
        let events = events(Provider::Anthropic, ANTHROPIC_ERROR);
        assert!(matches!(
            events.first(),
            Some(Event::Error(message)) if message.contains("overloaded")
        ));
    }

    #[test]
    fn a_stream_without_done_is_shown_as_cut_off() {
        let events = events(
            Provider::OpenAi,
            "data: {\"choices\":[{\"delta\":{\"content\":\"hi\"}}]}\n\n",
        );
        assert_eq!(answer(&events), "hi");
        assert_eq!(events.last(), Some(&Event::Error(CUT_OFF.to_owned())));
    }

    #[test]
    fn a_cloud_provider_needs_a_key_and_a_model() {
        let mut openai = settings(Provider::OpenAi);
        openai.api_key = None;
        assert!(matches!(
            client_with_timeouts(&openai, Arc::new(Silent), Timeouts::default()),
            Err(AiError::MissingKey)
        ));
        openai.api_key = Some("key".into());
        openai.model = String::new();
        assert!(matches!(
            client_with_timeouts(&openai, Arc::new(Silent), Timeouts::default()),
            Err(AiError::MissingModel)
        ));
        openai.model = "gpt-4o-mini".into();
        assert!(client_with_timeouts(&openai, Arc::new(Silent), Timeouts::default()).is_ok());
        let mut local = settings(Provider::LocalModel);
        local.api_key = None;
        assert!(client_with_timeouts(&local, Arc::new(Silent), Timeouts::default()).is_ok());
    }

    #[test]
    fn a_silent_server_times_out_instead_of_hanging() {
        let client = client_with_timeouts(
            &settings(Provider::OpenAi),
            Arc::new(Silent),
            Timeouts::default().response_headers(Some(Duration::from_millis(30))),
        )
        .expect("client builds");
        let events: Vec<Event> = client
            .stream(&Prompt {
                system: "be brief".into(),
                user: "hello".into(),
            })
            .expect("the request starts")
            .take(50)
            .collect();
        assert!(
            matches!(
                events.last(),
                Some(Event::Error(message)) if message.contains("timed out")
            ),
            "{events:?}"
        );
    }

    #[test]
    fn the_request_is_the_one_the_provider_speaks() {
        let replay = Arc::new(Replay::new(OPENAI));
        let client = client_with_timeouts(
            &settings(Provider::OpenAi),
            replay.clone(),
            Timeouts::default(),
        )
        .unwrap();
        let _ = client
            .stream(&Prompt {
                system: "sys".into(),
                user: "hello".into(),
            })
            .unwrap()
            .collect::<Vec<_>>();
        let requests = replay.requests.lock().unwrap();
        let request = requests.first().expect("a request");
        assert!(
            request.url.ends_with("/chat/completions"),
            "{}",
            request.url
        );
        let body = String::from_utf8_lossy(&request.body);
        assert!(body.contains("\"max_completion_tokens\":2048"), "{body}");
        assert!(body.contains("\"stream\":true"), "{body}");
    }
    #[test]
    fn a_follow_up_sends_the_earlier_exchange_and_the_new_question() {
        let replay = Arc::new(Replay::default());
        let client = client_with_timeouts(
            &settings(Provider::OpenAi),
            replay.clone(),
            Timeouts::default(),
        )
        .expect("client builds");
        let history = [super::Turn {
            question: "what did we decide?".to_owned(),
            answer: "the yearly plan".to_owned(),
        }];
        let events: Vec<Event> = client
            .ask(
                &Prompt {
                    system: "sys".into(),
                    user: "and when?".into(),
                },
                &history,
                ToolRegistry::new(),
                Box::new(|| {}),
                Arc::new(AtomicBool::new(false)),
            )
            .expect("the request starts")
            .take(20)
            .collect();
        assert!(matches!(events.last(), Some(Event::Done)), "{events:?}");
        let requests = replay.requests.lock().unwrap();
        let body = String::from_utf8_lossy(&requests[0].body);
        assert!(body.contains("what did we decide?"), "{body}");
        assert!(body.contains("the yearly plan"), "{body}");
        assert!(body.contains("and when?"), "{body}");
    }

    struct NoopTool {
        definition: ToolDefinition,
    }

    #[async_trait]
    impl Tool<()> for NoopTool {
        fn definition(&self) -> &ToolDefinition {
            &self.definition
        }

        async fn execute(&self, _ctx: (), _call: &ToolCall) -> anyhow::Result<ToolOutput> {
            Ok(ToolOutput {
                content: serde_json::json!({ "ok": true }),
            })
        }
    }

    #[test]
    fn the_tools_reach_the_request() {
        let replay = Arc::new(Replay::default());
        let client = client_with_timeouts(
            &settings(Provider::OpenAi),
            replay.clone(),
            Timeouts::default(),
        )
        .expect("client builds");
        let mut tools = ToolRegistry::new();
        tools.register(NoopTool {
            definition: ToolDefinition {
                name: "search_notes".to_owned(),
                description: "Search the notes.".to_owned(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": { "query": { "type": "string" } },
                    "required": ["query"]
                }),
            },
        });
        let _ = client
            .ask(
                &Prompt {
                    system: "sys".into(),
                    user: "hello".into(),
                },
                &[],
                tools,
                Box::new(|| {}),
                Arc::new(AtomicBool::new(false)),
            )
            .expect("the request starts")
            .take(20)
            .collect::<Vec<_>>();
        let requests = replay.requests.lock().unwrap();
        let body = String::from_utf8_lossy(&requests[0].body);
        assert!(body.contains("search_notes"), "{body}");
        assert!(body.contains("\"tools\""), "{body}");
    }

    #[test]
    fn stopping_cancels_a_silent_request() {
        let cancel = Arc::new(AtomicBool::new(false));
        let client = client_with_timeouts(
            &settings(Provider::OpenAi),
            Arc::new(Silent),
            Timeouts::none(),
        )
        .expect("client builds");
        let events = client
            .ask(
                &Prompt {
                    system: "be brief".into(),
                    user: "hello".into(),
                },
                &[],
                ToolRegistry::new(),
                Box::new(|| {}),
                cancel.clone(),
            )
            .expect("the request starts");
        let stopper = {
            let cancel = cancel.clone();
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_millis(50));
                cancel.store(true, Ordering::SeqCst);
            })
        };
        let started = Instant::now();
        let got: Vec<Event> = events.collect();
        stopper.join().unwrap();
        assert!(got.is_empty(), "{got:?}");
        assert!(
            started.elapsed() < Duration::from_secs(2),
            "Stop must abort a silent request promptly"
        );
    }
    /// A server that rejects tool definitions and then answers without them.
    #[derive(Clone, Default)]
    struct ToolsRejectedThenAnswers {
        body: Arc<std::sync::Mutex<Option<String>>>,
        requests: Arc<std::sync::Mutex<Vec<HttpRequest>>>,
    }

    #[async_trait]
    impl HttpClient for ToolsRejectedThenAnswers {
        async fn send(&self, request: HttpRequest) -> anyhow::Result<HttpResponse> {
            self.requests.lock().unwrap().push(request);
            Ok(HttpResponse {
                status: 400,
                body: br#"{"error":"registry.ollama.ai/library/gemma:2b does not support tools"}"#
                    .to_vec(),
            })
        }

        async fn send_streaming(&self, request: HttpRequest) -> anyhow::Result<HttpStreamResponse> {
            self.requests.lock().unwrap().push(request);
            let body = self
                .body
                .lock()
                .unwrap()
                .take()
                .expect("one recorded streamed body");
            let chunks: Vec<anyhow::Result<Vec<u8>>> = body
                .as_bytes()
                .chunks(7)
                .map(|chunk| Ok(chunk.to_vec()))
                .collect();
            Ok(HttpStreamResponse {
                status: 200,
                body: Box::pin(stream::iter(chunks)),
            })
        }
    }

    #[test]
    fn a_model_without_tools_falls_back_to_a_plain_streamed_answer() {
        let http = Arc::new(ToolsRejectedThenAnswers {
            body: Arc::new(std::sync::Mutex::new(Some(OPENAI.to_owned()))),
            requests: Arc::default(),
        });
        let mut settings = settings(Provider::LocalModel);
        settings.model = "gemma-tool-fallback".to_owned();
        let client =
            client_with_timeouts(&settings, http.clone(), Timeouts::default()).expect("client");
        let mut tools = ToolRegistry::new();
        tools.register(NoopTool {
            definition: ToolDefinition {
                name: "search_notes".to_owned(),
                description: "Search the notes.".to_owned(),
                input_schema: serde_json::json!({ "type": "object" }),
            },
        });

        let events: Vec<Event> = client
            .ask(
                &Prompt {
                    system: "sys".into(),
                    user: "hello".into(),
                },
                &[],
                tools,
                Box::new(|| {}),
                Arc::new(AtomicBool::new(false)),
            )
            .expect("the request starts")
            .take(50)
            .collect();
        assert_eq!(answer(&events), "Your notes say hello.");
        assert_eq!(events.last(), Some(&Event::Done));

        {
            let requests = http.requests.lock().unwrap();
            assert_eq!(requests.len(), 2, "{requests:?}");
            let first = String::from_utf8_lossy(&requests[0].body);
            assert!(first.contains("\"tools\""), "{first}");
            let second = String::from_utf8_lossy(&requests[1].body);
            assert!(!second.contains("\"tools\""), "{second}");
        }

        // The unsupported decision is remembered for this provider + model:
        // the next question streams straight away with no tool definitions.
        let second_client = client_with_timeouts(
            &settings,
            Arc::new(ToolsRejectedThenAnswers {
                body: Arc::new(std::sync::Mutex::new(Some(OPENAI.to_owned()))),
                requests: http.requests.clone(),
            }),
            Timeouts::default(),
        )
        .expect("client");
        let _ = second_client
            .ask(
                &Prompt {
                    system: "sys".into(),
                    user: "again".into(),
                },
                &[],
                ToolRegistry::new(),
                Box::new(|| {}),
                Arc::new(AtomicBool::new(false)),
            )
            .expect("the request starts")
            .take(50)
            .collect::<Vec<_>>();
        let requests = http.requests.lock().unwrap();
        assert_eq!(requests.len(), 3, "{requests:?}");
        let last = String::from_utf8_lossy(&requests[2].body);
        assert!(!last.contains("\"tools\""), "{last}");
    }

    #[test]
    fn a_direct_answer_is_delivered_without_a_stream() {
        let replay = Arc::new(Replay::new(OPENAI));
        let client = client_with_timeouts(
            &settings(Provider::OpenAi),
            replay.clone(),
            Timeouts::default(),
        )
        .expect("client");
        let events: Vec<Event> = client
            .ask(
                &Prompt {
                    system: "sys".into(),
                    user: "hello".into(),
                },
                &[],
                ToolRegistry::new(),
                Box::new(|| {}),
                Arc::new(AtomicBool::new(false)),
            )
            .expect("the request starts")
            .take(20)
            .collect();
        assert_eq!(answer(&events), "ok");
        assert_eq!(events.last(), Some(&Event::Done));
        // The buffered first turn answered; the recorded streamed body was
        // never taken.
        assert!(replay.body.lock().unwrap().is_some());
        assert_eq!(replay.requests.lock().unwrap().len(), 1);
    }

    #[test]
    fn a_closed_event_channel_reports_a_cut_off_once() {
        let (tx, rx) = std::sync::mpsc::channel();
        drop(tx);
        let mut stream = super::EventStream {
            rx,
            task: None,
            cancel: Arc::new(AtomicBool::new(false)),
            final_event: false,
        };
        assert_eq!(stream.next(), Some(Event::Error(CUT_OFF.to_owned())));
        assert_eq!(stream.next(), None);

        let (tx, rx) = std::sync::mpsc::channel();
        tx.send(Event::Done).unwrap();
        drop(tx);
        let mut stream = super::EventStream {
            rx,
            task: None,
            cancel: Arc::new(AtomicBool::new(false)),
            final_event: false,
        };
        assert_eq!(stream.next(), Some(Event::Done));
        assert_eq!(stream.next(), None);
    }

    #[test]
    fn a_client_is_reused_until_its_settings_change() {
        // A model on this network (plain http), which checks no certificate,
        // so this runs on an Android device before its app sets up TLS.
        let first = settings(Provider::LocalModel);
        let one = super::cached_client(&first).expect("client");
        let two = super::cached_client(&first).expect("client");
        assert!(Arc::ptr_eq(&one, &two));

        let mut changed = first.clone();
        changed.model = "another-model".to_owned();
        let three = super::cached_client(&changed).expect("client");
        assert!(!Arc::ptr_eq(&one, &three));
    }
}
