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
    AiAction, AiProviderInfo, AiSettingsInput, AiSettingsView, AskClosest, AskEvent, AskNote,
    AskReceipt, AskScopeInput, AskScopeOption, AskTime, AssistEvent,
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
    ask: kanso_engine::ask::settings::AskSettings,
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
            ask: Default::default(),
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
            let budget = file.ask.budget_for(provider, &file.base_url);
            Ok(AiSettingsView {
                ask_budget: budget.size.key().to_owned(),
                budget_words_line: budget.words_line(),
                budget_cost_line: kanso_ai::prices::cost_line(provider, &file.model, &budget)
                    .unwrap_or_default(),
                check_sources: file.ask.check_sources,
                use_links: file.ask.use_links,
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
            file.ask.budget = kanso_ai::BudgetSize::from_key(input.ask_budget.trim())
                .map(|size| size.key().to_owned());
            file.ask.check_sources = input.check_sources;
            file.ask.use_links = input.use_links;
            self.save(&file)?;
            self.settings()
        })
    }

    /// What a question can be limited to: all notes, each notebook that
    /// isn't locked by its path, and each tag but `#ai-exclude`.
    pub fn ask_scopes(&self) -> Result<Vec<AskScopeOption>, KansoError> {
        guard(|| {
            let store = &self.inner.store;
            let locked = store.locked_books().unwrap_or_default();
            let mut options = vec![AskScopeOption {
                kind: "all".to_owned(),
                value: String::new(),
                label: "All notes".to_owned(),
            }];
            let mut books: Vec<(String, String)> = ask::book_paths(store)
                .into_iter()
                .filter(|(id, _)| !locked.contains(id))
                .collect();
            books.sort_by_key(|(_, path)| path.to_lowercase());
            options.extend(books.into_iter().map(|(id, path)| AskScopeOption {
                kind: "book".to_owned(),
                value: id,
                label: path,
            }));
            let mut tags: Vec<String> = store
                .tags()
                .unwrap_or_default()
                .into_iter()
                .map(|tag| tag.name)
                .filter(|name| !ask::is_exclude_tag(name))
                .collect();
            tags.sort_by_key(|name| name.to_lowercase());
            options.extend(tags.into_iter().map(|name| AskScopeOption {
                kind: "tag".to_owned(),
                label: format!("#{name}"),
                value: name,
            }));
            Ok(options)
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
            kanso_ai::probe(&settings).map_err(|err| ai_error(err, settings.api_key.as_deref()))
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
            last: Arc::default(),
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
            self.inner.metrics().count(crate::metrics::Feature::Ai);
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
            self.inner.metrics().count(crate::metrics::Feature::Ai);
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
    pub(crate) fn enabled(&self) -> Result<ClientSettings, KansoError> {
        let file = self.load();
        if !file.enabled {
            return Err(KansoError::invalid(AI_OFF));
        }
        Ok(self.client_settings(&file))
    }

    /// The provider to ask under `settings`, or why one can't be made (no
    /// key, no model).
    pub(crate) fn provider_for(
        &self,
        settings: &ClientSettings,
    ) -> Result<SharedProvider, KansoError> {
        (self.maker)(settings).map_err(|err| ai_error(err, settings.api_key.as_deref()))
    }

    /// The provider Inbox triage asks, while AI is on.
    pub(crate) fn triage_provider(&self) -> Result<SharedProvider, KansoError> {
        let settings = self.enabled()?;
        self.provider_for(&settings)
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
    /// The newest checked answer, for Save as note.
    last: Arc<Mutex<Option<LastAnswer>>>,
}

/// An answer kept so the person can save it as a note.
struct LastAnswer {
    question: String,
    scope: ask::AskScope,
    finished: ask::Finished,
}

impl AskConversation {
    /// Asks `question` of the notes. A question already being answered stops
    /// first.
    ///
    /// Returns once the question has started, or at once with why it can't:
    /// AI is off, the provider has no key or model, the question is empty.
    /// The answer arrives as [`AskEvent`]s on `sink`, the last of which
    /// ends it.
    ///
    /// `scope` limits what is searched; `ignore_time` leaves out the time
    /// range the question was understood to have; `utc_offset_minutes` is
    /// the phone's offset from UTC, for "last week" and "in October".
    pub fn ask(
        &self,
        question: String,
        scope: AskScopeInput,
        ignore_time: bool,
        utc_offset_minutes: i32,
        sink: Sink<AskEvent>,
    ) -> Result<(), KansoError> {
        guard(|| {
            let question = question.trim().to_owned();
            if question.is_empty() {
                return Err(KansoError::invalid("Type a question."));
            }
            let scope = match scope.kind.as_str() {
                "book" => ask::AskScope::Book(scope.value),
                "tag" => ask::AskScope::Tag(scope.value),
                _ => ask::AskScope::All,
            };
            let settings = self.ai.enabled()?;
            let ask_settings = self.ai.load().ask;
            let provider = self.ai.provider_for(&settings)?;
            self.ai.inner.metrics().count(crate::metrics::Feature::Ai);
            self.stop();
            // A different provider, address or model is a different
            // conversation; what can no longer be sent is dropped.
            let (history, previous) = {
                let mut turns = held(&self.turns);
                turns.begin(&identity(&settings), &self.ai.inner.store);
                (turns.history(), turns.previous())
            };
            let offset_ms = i64::from(utc_offset_minutes) * 60_000;
            let mut request = ask::AskRequest::new(
                &question,
                kanso_core::dates::Date::from_local_ms(kanso_core::now_ms() + offset_ms),
            );
            request.scope = scope.clone();
            request.utc_offset_ms = offset_ms;
            request.budget = ask_settings.budget_for(settings.provider, &settings.base_url);
            request.use_links = ask_settings.use_links;
            request.ignore_time = ignore_time;
            request.previous = previous;
            let cancel = Arc::new(AtomicBool::new(false));
            *held(&self.running) = Some(cancel.clone());
            let emitter = Arc::new(Emitter::new(sink, cancel.clone()));
            let worker = Question {
                emitter: emitter.clone(),
                cancel,
                store: self.ai.inner.store.clone(),
                turns: self.turns.clone(),
                last: self.last.clone(),
                provider,
                history,
                text: question,
                scope,
                request,
                check_sources: ask_settings.check_sources,
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
        *held(&self.last) = None;
    }

    /// Saves the newest answer as a note, titled "Q: <question>", with its
    /// Sources as note links to the lines cited, in the notebook the
    /// question was limited to or else the Inbox. The one write Ask makes,
    /// and only when the person asks for it. Returns the new note's ID.
    pub fn save_last(&self) -> Result<String, KansoError> {
        guard(|| {
            let last = held(&self.last);
            let Some(last) = last.as_ref() else {
                return Err(KansoError::invalid("There is no answer to save."));
            };
            ask::save_answer(
                &self.ai.inner.store,
                &last.scope,
                &last.question,
                &last.finished,
            )
            .map_err(|err| KansoError::storage(err.to_string()))
        })
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

    /// The flag a request polls to see whether it was asked to stop.
    pub(crate) fn cancel_flag(&self) -> Arc<AtomicBool> {
        self.cancel.clone()
    }

    /// Whether the request was asked to stop.
    pub(crate) fn stopped(&self) -> bool {
        self.cancel.load(Ordering::Relaxed)
    }
}

/// One question being answered on its own thread.
struct Question {
    emitter: Arc<Emitter<AskEvent>>,
    cancel: Arc<AtomicBool>,
    store: Arc<kanso_store::Store>,
    turns: Arc<Mutex<Conversation>>,
    last: Arc<Mutex<Option<LastAnswer>>>,
    provider: SharedProvider,
    history: Vec<kanso_ai::Turn>,
    text: String,
    scope: ask::AskScope,
    request: ask::AskRequest,
    check_sources: bool,
}

impl Question {
    fn run(self) {
        let Question {
            emitter,
            cancel,
            store,
            turns,
            last,
            provider,
            history,
            text,
            scope,
            request,
            check_sources,
        } = self;
        let mut answer = String::new();
        let mut used: Vec<Excerpt> = Vec::new();
        let mut turn_notes: Vec<String> = Vec::new();
        ask::run(
            ask::Turn {
                store,
                provider,
                request,
                history,
                check_sources,
                cancel: cancel.clone(),
            },
            |event| match event {
                ask::TurnEvent::Status(line) if line == ask::SEARCHING => {
                    emitter.send(AskEvent::Searching)
                }
                ask::TurnEvent::Status(line) => emitter.send(AskEvent::Status { line }),
                ask::TurnEvent::Used(list) => {
                    held(&turns).note_used(&mut turn_notes, list.clone());
                    emitter.send(AskEvent::Used {
                        notes: notes_of(&list),
                    });
                    used = list;
                }
                ask::TurnEvent::Time(time) => emitter.send(AskEvent::Time {
                    time: time_of(&time),
                }),
                ask::TurnEvent::Text(piece) => {
                    answer.push_str(&piece);
                    emitter.send(AskEvent::Answer {
                        markdown: link_notes(&ask::streaming_markdown(&answer, &used)),
                    });
                }
                ask::TurnEvent::Done(done) => {
                    let done = *done;
                    held(&turns).finish_with(
                        text.clone(),
                        done.plain_answer(),
                        turn_notes.clone(),
                        done.receipts.clone(),
                    );
                    let event = AskEvent::Done {
                        markdown: link_notes(&done.markdown),
                        notes: notes_of(&used),
                        receipts: done.receipts.iter().map(receipt_of).collect(),
                        also_read: notes_of(&done.also_read),
                        banner: done.banner.map(|b| b.text().to_owned()).unwrap_or_default(),
                        unverified: done.unverified as u32,
                        never_wrote: done.not_in_notes,
                        closest: done
                            .closest
                            .iter()
                            .map(|c| AskClosest {
                                note_id: c.note_id.clone(),
                                title: c.title.clone(),
                                start: c.start,
                                end: c.end,
                            })
                            .collect(),
                        time: done.time.as_ref().map(time_of),
                    };
                    *held(&last) = Some(LastAnswer {
                        question: text.clone(),
                        scope: scope.clone(),
                        finished: done,
                    });
                    emitter.finish(event);
                }
                ask::TurnEvent::NothingMatches { narrow } => {
                    emitter.finish(AskEvent::NothingMatches {
                        message: NOTHING_MATCHES.to_owned(),
                        narrow,
                    })
                }
                ask::TurnEvent::Failed(message) => emitter.finish(AskEvent::Failed { message }),
                ask::TurnEvent::CutOff => emitter.finish(AskEvent::CutOff {
                    message: kanso_ai::CUT_OFF.to_owned(),
                }),
                ask::TurnEvent::Stopped => emitter.finish(AskEvent::Stopped),
            },
        );
        // A run that ended without saying how (a stopped stream).
        if cancel.load(Ordering::Relaxed) {
            emitter.finish(AskEvent::Stopped);
        }
    }
}

/// An answer for display: note links point at `kanso://note/<id>`.
fn link_notes(markdown: &str) -> String {
    markdown.replace(
        kanso_core::links::NOTE_LINK_PREFIX,
        &format!("{CITATION_PREFIX}note:"),
    )
}

fn time_of(time: &ask::TimeNote) -> AskTime {
    use kanso_core::when::TimeIntent;
    AskTime {
        label: match time.intent {
            TimeIntent::Written(_) => format!("Written {}", time.label),
            TimeIntent::About(_) => format!("About {}", time.label),
            TimeIntent::Either(_) => time.label.clone(),
        },
        relaxed_line: time.relaxed_line().unwrap_or_default(),
    }
}

fn receipt_of(receipt: &ask::Receipt) -> AskReceipt {
    AskReceipt {
        n: receipt.n as u32,
        note_id: receipt.note_id.clone(),
        title: receipt.title.clone(),
        label: receipt.source_line(),
        start: receipt.start,
        end: receipt.end,
        quote: receipt.quote.clone(),
    }
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
pub(crate) struct Emitter<T: Clone + 'static> {
    listeners: Listeners<T>,
    ended: AtomicBool,
    cancel: Arc<AtomicBool>,
}

impl<T: Clone + 'static> Emitter<T> {
    pub(crate) fn new(sink: Sink<T>, cancel: Arc<AtomicBool>) -> Emitter<T> {
        let listeners = Listeners::default();
        listeners.add(sink);
        Emitter {
            listeners,
            ended: AtomicBool::new(false),
            cancel,
        }
    }

    /// Sends a value that isn't the end, unless the stream has ended.
    pub(crate) fn send(&self, value: T) {
        if self.ended.load(Ordering::SeqCst) || self.cancel.load(Ordering::Relaxed) {
            return;
        }
        self.listeners.send(value);
        if self.listeners.is_empty() {
            self.cancel.store(true, Ordering::Relaxed);
        }
    }

    /// Sends the end, once.
    pub(crate) fn finish(&self, value: T) {
        if !self.ended.swap(true, Ordering::SeqCst) {
            self.listeners.send(value);
        }
    }
}

/// Runs `work` on a named thread, or says why it can't.
pub(crate) fn spawn(name: &str, work: impl FnOnce() + Send + 'static) -> Result<(), KansoError> {
    std::thread::Builder::new()
        .name(name.into())
        .spawn(work)
        .map(|_| ())
        .map_err(|err| KansoError::storage(format!("Couldn't start the request: {err}")))
}

pub(crate) fn ai_error(err: AiError, api_key: Option<&str>) -> KansoError {
    let message = kanso_ai::redact_provider_error(&err.to_string(), api_key);
    match err {
        AiError::Transport(_) | AiError::Timeout(_) => KansoError::network(message),
        _ => KansoError::invalid(message),
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
