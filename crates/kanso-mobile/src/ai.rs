//! The AI features: Ask your notes, the inline assistant and the note
//! summary, on the engine's retrieval (`kanso_engine::ask`) and the shared AI
//! client (`kanso-ai`, without the desktop's agents).
//!
//! # What is kept here
//!
//! - The settings are Rust's: `ai.json` next to the library's database, off
//!   until the person turns it on. The API key is not in it. It goes to the
//!   phone's secret store ([`crate::Secrets`]) under `ai-key:<provider>`, as
//!   on the desktop, and is never returned, logged or in an error.
//! - A question is retrieved, asked and answered here. The shell hears
//!   [`AskEvent`]s; it never sees a note's text, only the answer, the titles
//!   and the IDs of the notes it drew on.
//! - A locked note, a note in the trash and a note tagged `#ai-exclude` are
//!   never searched, read or sent, checked by ID in Rust against the store
//!   (not against the lock session: a note stays kept out while its lock is
//!   open). The inline assistant and the summary refuse such a note with
//!   [`KEPT_OUT`] before anything is sent.
//!
//! # Answers
//!
//! An answer is Markdown, made safe to show: no HTML, no images, nothing that
//! makes the phone fetch anything. Each note it cites with `[[Title]]` is a
//! link to `kanso://note/<id>` ([`CITATION_PREFIX`]), where `<id>` is the ID
//! every other call takes, so tapping it can open the note. The shell
//! renders the Markdown itself (no web view); [`AskNote`] lists the cited
//! notes with their titles.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use kanso_ai::{AiError, ClientSettings, Event, Excerpt, InlineAction, Provider, SharedProvider};
use kanso_engine::ask::{self, Conversation};
use kanso_engine::assist;
use serde::{Deserialize, Serialize};

use crate::ai_model::{
    AiAction, AiProviderInfo, AiSettingsInput, AiSettingsView, AskEvent, AskNote, AssistEvent,
};
use crate::boundary::{guard, held};
use crate::errors::KansoError;
use crate::library::{Inner, Library};
use crate::stream::{Listeners, Sink};

/// What a note's citation link starts with; the note's ID follows.
pub const CITATION_PREFIX: &str = "kanso://note/";
/// What an inline assist or a summary says about a note kept out of AI.
pub use kanso_engine::assist::KEPT_OUT;
/// What a question says when no note matches it. No model is asked.
pub const NOTHING_MATCHES: &str = "Nothing in your notes matches this question.";
/// What a request says while AI is off.
pub const AI_OFF: &str = "AI is turned off. Turn it on in Settings.";
/// The file the settings are in, next to the library's database.
const SETTINGS_FILE: &str = "ai.json";

/// Makes the provider a request asks, from the settings it is asked under.
/// The real one builds (and reuses) the HTTP client; a test hands in a
/// provider that answers without a network.
pub type ProviderMaker =
    Arc<dyn Fn(&ClientSettings) -> Result<SharedProvider, AiError> + Send + Sync>;

/// The AI features of one library.
#[derive(Clone)]
pub struct AiHandle {
    pub(crate) inner: Arc<Inner>,
    maker: ProviderMaker,
}

/// What `ai.json` holds. Every field has a default, so a file from an older
/// or newer build still reads.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct SettingsFile {
    enabled: bool,
    provider: String,
    base_url: String,
    model: String,
    region: String,
}

impl Default for SettingsFile {
    fn default() -> Self {
        SettingsFile::for_provider(Provider::default())
    }
}

impl SettingsFile {
    /// The settings a provider starts with: its own address and model.
    fn for_provider(provider: Provider) -> SettingsFile {
        SettingsFile {
            enabled: false,
            provider: provider.key().to_owned(),
            base_url: provider.default_base_url().to_owned(),
            model: provider.default_model().to_owned(),
            region: provider.default_region().to_owned(),
        }
    }

    /// The provider named, or the first one for a name this build doesn't
    /// know.
    fn provider(&self) -> Provider {
        provider_named(&self.provider).unwrap_or_default()
    }
}

fn provider_named(key: &str) -> Option<Provider> {
    Provider::ALL
        .into_iter()
        .find(|provider| provider.key() == key.trim())
}

/// The Keychain item a provider's API key is kept in: the desktop's name.
fn key_item(provider: Provider) -> String {
    format!("ai-key:{}", provider.key())
}

impl Library {
    /// The AI features: settings, Ask your notes, the inline assistant and
    /// the note summary.
    pub fn ai(&self) -> AiHandle {
        AiHandle {
            inner: self.inner.clone(),
            maker: Arc::new(kanso_ai::cached_client),
        }
    }

    /// [`Library::ai`] with the provider made by `maker`: the tests', which
    /// answer without a network.
    #[cfg(any(test, feature = "test-support"))]
    pub fn ai_with_provider(&self, maker: ProviderMaker) -> AiHandle {
        AiHandle {
            inner: self.inner.clone(),
            maker,
        }
    }
}

impl AiHandle {
    /// The providers to choose from, in the order Settings lists them.
    pub fn providers(&self) -> Vec<AiProviderInfo> {
        Provider::ALL
            .into_iter()
            .map(|provider| AiProviderInfo {
                key: provider.key().to_owned(),
                label: provider.label().to_owned(),
                description: describe(provider).to_owned(),
                default_base_url: provider.default_base_url().to_owned(),
                default_model: provider.default_model().to_owned(),
                default_region: provider.default_region().to_owned(),
                needs_key: provider.needs_key(),
                accepts_key: provider.accepts_key(),
            })
            .collect()
    }

    /// What the inline assistant can do with a selection, in the menu's
    /// words.
    pub fn actions(&self) -> Vec<AiAction> {
        InlineAction::ALL
            .into_iter()
            .map(|action| AiAction {
                key: action_key(action).to_owned(),
                label: action.label().to_owned(),
                noun: action.noun().to_owned(),
                replaces: action.replaces(),
            })
            .collect()
    }

    /// The settings now.
    pub fn settings(&self) -> Result<AiSettingsView, KansoError> {
        guard(|| {
            let file = self.load();
            let provider = file.provider();
            Ok(AiSettingsView {
                enabled: file.enabled,
                provider: provider.key().to_owned(),
                has_key: self.read_key(provider).is_some(),
                host: provider.host(&file.base_url),
                is_remote: provider.is_remote(&file.base_url),
                base_url: file.base_url,
                model: file.model,
                region: file.region,
            })
        })
    }

    /// The plain line about where a question goes, with `device` naming the
    /// device it leaves ("this phone", "this tablet").
    pub fn privacy_line(&self, device: String) -> Result<String, KansoError> {
        guard(|| {
            let file = self.load();
            Ok(assist::requests_line(
                file.provider(),
                &file.base_url,
                device.trim(),
            ))
        })
    }

    /// Saves the settings and returns them as they are now. An empty address
    /// or model means the provider's own; the region is kept for Bedrock only.
    pub fn set_settings(&self, input: AiSettingsInput) -> Result<AiSettingsView, KansoError> {
        guard(|| {
            let provider = provider_named(&input.provider)
                .ok_or_else(|| KansoError::invalid("That isn't a provider Kanso knows."))?;
            let mut file = SettingsFile::for_provider(provider);
            file.enabled = input.enabled;
            if !input.base_url.trim().is_empty() && provider != Provider::Bedrock {
                file.base_url = input.base_url.trim().to_owned();
            }
            if !input.model.trim().is_empty() {
                file.model = input.model.trim().to_owned();
            }
            if provider == Provider::Bedrock && !input.region.trim().is_empty() {
                file.region = input.region.trim().to_owned();
            }
            self.save(&file)?;
            self.settings()
        })
    }

    /// Saves the API key for `provider` in the phone's secret store; an empty
    /// key removes it. The key goes nowhere else and is never given back.
    pub fn set_key(&self, provider: String, key: String) -> Result<(), KansoError> {
        guard(|| {
            let provider = provider_named(&provider)
                .ok_or_else(|| KansoError::invalid("That isn't a provider Kanso knows."))?;
            let key = key.trim();
            if key.is_empty() {
                self.inner.secrets.delete(&key_item(provider))?;
            } else {
                self.inner.secrets.set(&key_item(provider), key)?;
            }
            Ok(())
        })
    }

    /// Asks the model the settings name a tiny question and returns its
    /// first word, so Settings can say the settings work.
    pub fn test(&self) -> Result<String, KansoError> {
        guard(|| {
            let file = self.load();
            let settings = self.client_settings(&file);
            kanso_ai::probe(&settings).map_err(ai_error)
        })
    }

    /// Whether note `id` may be sent to AI: it isn't locked (even if the lock
    /// is open), isn't in the trash and isn't tagged `#ai-exclude`. The shell
    /// uses it to show or hide the AI actions.
    pub fn note_allowed(&self, id: &str) -> Result<bool, KansoError> {
        guard(|| Ok(assist::eligible(&self.inner.store, id)))
    }

    /// A new conversation: Ask your notes, with the exchanges so far kept in
    /// memory only.
    pub fn conversation(&self) -> AskConversation {
        AskConversation {
            ai: self.clone(),
            turns: Arc::default(),
            running: Arc::default(),
        }
    }

    /// Asks for `action` (one of [`AiHandle::actions`]' keys) on the text
    /// selected in note `id`. `before` is the note's text just before the
    /// selection, for context. With nothing selected, Continue writes on
    /// from the paragraph before.
    ///
    /// Refuses with [`KEPT_OUT`] when the note is kept out of AI, and with
    /// [`AI_OFF`] while AI is off; nothing is sent either way. Otherwise the
    /// events arrive on `sink`, and `run` (a new [`AiRun`] for each request)
    /// stops them.
    pub fn assist(
        &self,
        id: &str,
        action: &str,
        before: String,
        selection: String,
        run: &AiRun,
        sink: Sink<AssistEvent>,
    ) -> Result<(), KansoError> {
        guard(|| {
            let action = action_named(action)
                .ok_or_else(|| KansoError::invalid("That isn't something the assistant does."))?;
            let settings = self.enabled()?;
            let note = self.allowed_note(id)?;
            let provider = self.provider_for(&settings)?;
            let (before, selection) = split_selection(action, before, selection)?;
            let prompt =
                kanso_ai::features::inline_prompt(action, &note.title, &before, &selection);
            let emitter = Arc::new(Emitter::new(sink, run.cancel.clone()));
            let worker = emitter.clone();
            let cancel = run.cancel.clone();
            spawn("kanso-inline", move || {
                let events = {
                    let provider = held(&provider);
                    provider.stream(&prompt)
                };
                let mut said = String::new();
                kanso_ai::features::relay(events, &cancel, &mut |event| {
                    feature_event(
                        &worker,
                        &mut said,
                        event,
                        kanso_ai::features::clean_suggestion,
                    );
                });
                settle(&worker, &cancel);
            })
        })
    }

    /// Summarises note `id` (long notes in parts), reading its text here.
    /// Refuses like [`AiHandle::assist`]. The summary is Markdown made safe
    /// to show, like an answer.
    pub fn summarize(
        &self,
        id: &str,
        run: &AiRun,
        sink: Sink<AssistEvent>,
    ) -> Result<(), KansoError> {
        guard(|| {
            let settings = self.enabled()?;
            let note = self.allowed_note(id)?;
            let provider = self.provider_for(&settings)?;
            if note.body.trim().is_empty() {
                return Err(KansoError::invalid("This note is empty."));
            }
            let title = if note.title.trim().is_empty() {
                "Untitled".to_owned()
            } else {
                note.title.trim().to_owned()
            };
            let emitter = Arc::new(Emitter::new(sink, run.cancel.clone()));
            let worker = emitter.clone();
            let cancel = run.cancel.clone();
            spawn("kanso-summary", move || {
                let mut said = String::new();
                {
                    let provider = held(&provider);
                    kanso_ai::features::summarize(
                        provider.as_ref(),
                        &title,
                        &note.body,
                        &cancel,
                        |event| feature_event(&worker, &mut said, event, summary_text),
                    );
                }
                settle(&worker, &cancel);
            })
        })
    }

    /// The settings file's folder: the library's.
    fn settings_path(&self) -> PathBuf {
        self.inner
            .path
            .parent()
            .filter(|dir| !dir.as_os_str().is_empty())
            .unwrap_or_else(|| std::path::Path::new("."))
            .join(SETTINGS_FILE)
    }

    /// The saved settings, or the defaults when there are none (or the file
    /// can't be read: a damaged file must not lock AI settings away).
    fn load(&self) -> SettingsFile {
        std::fs::read(self.settings_path())
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    /// Writes the settings whole, by writing beside them and renaming, so a
    /// crash leaves the old file or the new one.
    fn save(&self, file: &SettingsFile) -> Result<(), KansoError> {
        let path = self.settings_path();
        let bytes =
            serde_json::to_vec_pretty(file).map_err(|err| KansoError::storage(err.to_string()))?;
        let part = path.with_extension("json.part");
        std::fs::write(&part, bytes)
            .and_then(|()| std::fs::rename(&part, &path))
            .map_err(|err| KansoError::storage(err.to_string()))
    }

    /// The key saved for `provider`, if any.
    fn read_key(&self, provider: Provider) -> Option<String> {
        self.inner
            .secrets
            .get(&key_item(provider))
            .ok()
            .flatten()
            .map(|key| key.to_string())
            .filter(|key| !key.trim().is_empty())
    }

    /// What a request is asked under: the settings and the saved key.
    fn client_settings(&self, file: &SettingsFile) -> ClientSettings {
        let provider = file.provider();
        ClientSettings {
            provider,
            base_url: file.base_url.clone(),
            model: file.model.clone(),
            api_key: self.read_key(provider),
            region: file.region.clone(),
        }
    }

    /// The settings a request is asked under, or [`AI_OFF`].
    fn enabled(&self) -> Result<ClientSettings, KansoError> {
        let file = self.load();
        if !file.enabled {
            return Err(KansoError::invalid(AI_OFF));
        }
        Ok(self.client_settings(&file))
    }

    /// The provider to ask under `settings`, or why one can't be made (no
    /// key, no model).
    fn provider_for(&self, settings: &ClientSettings) -> Result<SharedProvider, KansoError> {
        (self.maker)(settings).map_err(ai_error)
    }

    /// Note `id`, when it may be sent to AI; otherwise [`KEPT_OUT`].
    fn allowed_note(&self, id: &str) -> Result<ask::Candidate, KansoError> {
        assist::note_for_ai(&self.inner.store, id).ok_or_else(|| KansoError::invalid(KEPT_OUT))
    }
}

/// A conversation with the notes: questions and answers kept in memory for
/// as long as this handle lives, never written down.
///
/// It starts over when what it is asked under changes (the provider, the
/// address or the model), forgets an exchange whose notes have since been
/// locked, trashed or tagged `#ai-exclude`, and resends at most
/// [`ask::HISTORY_BUDGET`] characters of earlier exchanges.
#[derive(Clone)]
pub struct AskConversation {
    ai: AiHandle,
    turns: Arc<Mutex<Conversation>>,
    /// The question being answered, as the flag that stops it.
    running: Arc<Mutex<Option<Arc<AtomicBool>>>>,
}

impl AskConversation {
    /// Asks `question` of the notes. A question already being answered stops
    /// first.
    ///
    /// Returns once the question has started, or at once with why it can't:
    /// AI is off, the provider has no key or model, the question is empty.
    /// The answer arrives as [`AskEvent`]s on `sink`, the last of which
    /// ends it.
    pub fn ask(&self, question: String, sink: Sink<AskEvent>) -> Result<(), KansoError> {
        guard(|| {
            let question = question.trim().to_owned();
            if question.is_empty() {
                return Err(KansoError::invalid("Type a question."));
            }
            let settings = self.ai.enabled()?;
            let provider = self.ai.provider_for(&settings)?;
            self.stop();
            // A different provider, address or model is a different
            // conversation; what can no longer be sent is dropped.
            let history = {
                let mut turns = held(&self.turns);
                turns.begin(&identity(&settings), &self.ai.inner.store);
                turns.history()
            };
            let cancel = Arc::new(AtomicBool::new(false));
            *held(&self.running) = Some(cancel.clone());
            let emitter = Arc::new(Emitter::new(sink, cancel.clone()));
            let worker = Question {
                emitter: emitter.clone(),
                cancel,
                store: self.ai.inner.store.clone(),
                turns: self.turns.clone(),
                provider,
                history,
                text: question,
            };
            spawn("kanso-ask", move || worker.run())
        })
    }

    /// Stops the question being answered, if one is. Its stream ends with
    /// [`AskEvent::Stopped`].
    pub fn stop(&self) {
        if let Some(cancel) = held(&self.running).take() {
            cancel.store(true, Ordering::Relaxed);
        }
    }

    /// Stops any answer and forgets every exchange.
    pub fn clear(&self) {
        self.stop();
        held(&self.turns).clear();
    }

    /// How many exchanges are kept to resend.
    pub fn turn_count(&self) -> u32 {
        held(&self.turns).turns().len() as u32
    }
}

/// A request to stop: made before the request starts, so the shell holds it
/// while the events arrive. One is for one request.
#[derive(Clone, Default)]
pub struct AiRun {
    cancel: Arc<AtomicBool>,
}

impl AiRun {
    pub fn new() -> AiRun {
        AiRun::default()
    }

    /// Stops the request. Its stream ends with `Stopped`, unless it had
    /// already ended.
    pub fn stop(&self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}

/// One question being answered on its own thread.
struct Question {
    emitter: Arc<Emitter<AskEvent>>,
    cancel: Arc<AtomicBool>,
    store: Arc<kanso_store::Store>,
    turns: Arc<Mutex<Conversation>>,
    provider: SharedProvider,
    history: Vec<kanso_ai::Turn>,
    text: String,
}

impl Question {
    fn run(self) {
        let Question {
            emitter,
            cancel,
            store,
            turns,
            provider,
            history,
            text,
        } = self;
        emitter.send(AskEvent::Searching);
        let excerpts = match ask::retrieve(&store, &text) {
            Ok(excerpts) => excerpts,
            Err(err) => {
                emitter.finish(AskEvent::Failed {
                    message: format!("Couldn't search your notes: {err}"),
                });
                return;
            }
        };
        if cancel.load(Ordering::Relaxed) {
            emitter.finish(AskEvent::Stopped);
            return;
        }
        if excerpts.is_empty() {
            emitter.finish(AskEvent::NothingMatches {
                message: NOTHING_MATCHES.to_owned(),
            });
            return;
        }
        let prompt = kanso_ai::build_prompt(&text, &excerpts, ask::BUDGET);
        // Every note this question's tools search or read is kept, so the
        // answer can list and cite all of them.
        let used = Arc::new(Mutex::new(excerpts));
        let turn_notes = Arc::new(Mutex::new(Vec::<String>::new()));
        let report = {
            let (emitter, used, turns, turn_notes) = (
                emitter.clone(),
                used.clone(),
                turns.clone(),
                turn_notes.clone(),
            );
            move || {
                let list = held(&used).clone();
                held(&turns).note_used(&mut held(&turn_notes), list.clone());
                emitter.send(AskEvent::Used {
                    notes: notes_of(&list),
                });
            }
        };
        report();
        let tools = ask::tool_registry(store.clone(), used.clone());
        // The provider is held only to start the request.
        let events = {
            let provider = held(&provider);
            provider.ask(
                &prompt,
                &history,
                tools,
                Box::new({
                    let report = report.clone();
                    move || report()
                }),
                cancel.clone(),
            )
        };
        let mut answer = String::new();
        ask::stream_events(events, &cancel, |event| match event {
            Event::Text(piece) => {
                answer.push_str(&piece);
                let list = held(&used).clone();
                emitter.send(AskEvent::Answer {
                    markdown: render(&answer, &list),
                });
            }
            Event::Status(line) => emitter.send(AskEvent::Status { line }),
            Event::Done => {
                if answer.trim().is_empty() {
                    emitter.finish(AskEvent::Failed {
                        message: "Nothing came back.".to_owned(),
                    });
                    return;
                }
                let list = held(&used).clone();
                held(&turns).finish(text.clone(), answer.clone(), held(&turn_notes).clone());
                emitter.finish(AskEvent::Done {
                    markdown: render(&answer, &list),
                    notes: notes_of(&list),
                });
            }
            Event::Error(message) if message == kanso_ai::CUT_OFF => {
                emitter.finish(AskEvent::CutOff { message });
            }
            Event::Error(message) => emitter.finish(AskEvent::Failed { message }),
        });
        // The stream ended without saying how: stopped, or cut short.
        if cancel.load(Ordering::Relaxed) {
            emitter.finish(AskEvent::Stopped);
        } else {
            emitter.finish(AskEvent::CutOff {
                message: kanso_ai::CUT_OFF.to_owned(),
            });
        }
    }
}

/// An answer for display: citations linked to their notes, images and HTML
/// made harmless.
fn render(answer: &str, used: &[Excerpt]) -> String {
    ask::sanitize_answer(answer, used).replace(
        kanso_core::links::NOTE_LINK_PREFIX,
        &format!("{CITATION_PREFIX}note:"),
    )
}

fn notes_of(list: &[Excerpt]) -> Vec<AskNote> {
    list.iter()
        .map(|excerpt| AskNote {
            id: excerpt.id.clone(),
            title: excerpt.title.clone(),
        })
        .collect()
}

/// What a conversation was asked under.
fn identity(settings: &ClientSettings) -> String {
    format!(
        "{}|{}|{}|{}",
        settings.provider.key(),
        settings.base_url.trim(),
        settings.model.trim(),
        settings.region.trim()
    )
}

/// What a feature's events do: the suggestion so far, then the end.
fn feature_event(
    emitter: &Emitter<AssistEvent>,
    said: &mut String,
    event: Event,
    shape: fn(&str) -> String,
) {
    match event {
        Event::Text(piece) => {
            said.push_str(&piece);
            emitter.send(AssistEvent::Text { text: shape(said) });
        }
        Event::Status(line) => emitter.send(AssistEvent::Status { line }),
        Event::Done => {
            let text = shape(said);
            if text.trim().is_empty() {
                emitter.finish(AssistEvent::Failed {
                    message: "Nothing came back.".to_owned(),
                });
            } else {
                emitter.finish(AssistEvent::Done { text });
            }
        }
        Event::Error(message) if message == kanso_ai::CUT_OFF => {
            emitter.finish(AssistEvent::CutOff { message });
        }
        Event::Error(message) => emitter.finish(AssistEvent::Failed { message }),
    }
}

/// The ending of a feature whose stream stopped without one.
fn settle(emitter: &Emitter<AssistEvent>, cancel: &AtomicBool) {
    if cancel.load(Ordering::Relaxed) {
        emitter.finish(AssistEvent::Stopped);
    } else {
        emitter.finish(AssistEvent::CutOff {
            message: kanso_ai::CUT_OFF.to_owned(),
        });
    }
}

/// A summary for display.
fn summary_text(text: &str) -> String {
    ask::sanitize_answer(&kanso_ai::features::clean_suggestion(text), &[])
}

/// The selection a feature works on, and the text before it. Continue with
/// nothing selected writes on from the paragraph before.
fn split_selection(
    action: InlineAction,
    before: String,
    selection: String,
) -> Result<(String, String), KansoError> {
    if !selection.trim().is_empty() {
        return Ok((before, selection));
    }
    if action != InlineAction::Continue {
        return Err(KansoError::invalid("Select some text first."));
    }
    let trimmed = before.trim_end();
    let start = trimmed.rfind("\n\n").map_or(0, |at| at + 2);
    if trimmed[start..].trim().is_empty() {
        return Err(KansoError::invalid(
            "Put the cursor after some text to continue it.",
        ));
    }
    Ok((trimmed[..start].to_owned(), trimmed[start..].to_owned()))
}

/// A request's stream, kept to one ending. Events go to the shell's sink; a
/// sink that says it is gone stops the request.
struct Emitter<T: Clone + 'static> {
    listeners: Listeners<T>,
    ended: AtomicBool,
    cancel: Arc<AtomicBool>,
}

impl<T: Clone + 'static> Emitter<T> {
    fn new(sink: Sink<T>, cancel: Arc<AtomicBool>) -> Emitter<T> {
        let listeners = Listeners::default();
        listeners.add(sink);
        Emitter {
            listeners,
            ended: AtomicBool::new(false),
            cancel,
        }
    }

    /// Sends a value that isn't the end, unless the stream has ended.
    fn send(&self, value: T) {
        if self.ended.load(Ordering::SeqCst) || self.cancel.load(Ordering::Relaxed) {
            return;
        }
        self.listeners.send(value);
        if self.listeners.is_empty() {
            self.cancel.store(true, Ordering::Relaxed);
        }
    }

    /// Sends the end, once.
    fn finish(&self, value: T) {
        if !self.ended.swap(true, Ordering::SeqCst) {
            self.listeners.send(value);
        }
    }
}

/// Runs `work` on a named thread, or says why it can't.
fn spawn(name: &str, work: impl FnOnce() + Send + 'static) -> Result<(), KansoError> {
    std::thread::Builder::new()
        .name(name.into())
        .spawn(work)
        .map(|_| ())
        .map_err(|err| KansoError::storage(format!("Couldn't start the request: {err}")))
}

fn ai_error(err: AiError) -> KansoError {
    match err {
        AiError::Transport(_) | AiError::Timeout(_) => KansoError::network(err.to_string()),
        other => KansoError::invalid(other.to_string()),
    }
}

fn action_key(action: InlineAction) -> &'static str {
    match action {
        InlineAction::Improve => "improve",
        InlineAction::Shorten => "shorten",
        InlineAction::Fix => "fix",
        InlineAction::Continue => "continue",
        InlineAction::Summarize => "summarize",
    }
}

fn action_named(key: &str) -> Option<InlineAction> {
    InlineAction::ALL
        .into_iter()
        .find(|action| action_key(*action) == key.trim())
}

/// What a provider is, in a line: the desktop's Settings › AI words, except
/// that a local model is one on a computer on the network.
fn describe(provider: Provider) -> &'static str {
    match provider {
        Provider::LocalModel => {
            "Ollama or LM Studio on a computer on your network. No key unless the server asks for one."
        }
        Provider::OpenAi => "OpenAI's own models, with a key from your OpenAI account.",
        Provider::Anthropic => "Claude models, with a key from your Anthropic account.",
        Provider::Gemini => "Google Gemini models, with a key from Google AI Studio.",
        Provider::OpenRouter => "Many providers through OpenRouter, with a key from your account.",
        Provider::Groq => "Groq's fast models, with a key from your Groq account.",
        Provider::DeepSeek => "DeepSeek's models, with a key from your DeepSeek account.",
        Provider::Xai => "xAI's Grok models, with a key from your xAI account.",
        Provider::Mistral => "Mistral's models, with a key from your Mistral account.",
        Provider::Cohere => "Cohere's models, with a key from your Cohere account.",
        Provider::Bedrock => "AWS Bedrock with a bearer token, in the region you choose.",
        Provider::Kimi => "Moonshot's Kimi models, with a key from your Moonshot account.",
        Provider::Custom => {
            "Another OpenAI-compatible server, such as LM Studio, at an address you type. A key is optional."
        }
    }
}
