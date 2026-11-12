//! Ask Your Notes: a question box, the answer as it streams in from the
//! provider the person chose, receipts for what it says, and the notes it
//! used, each a link.
//!
//! Search, the excerpts, the prompt and the checking of the answer are done
//! on this device (`kanso_engine::ask`); only the excerpts chosen for the
//! question leave it. The answer is not saved unless the person presses Save
//! as note, and no other note is changed.

use gpui_component::scroll::ScrollableElement as _;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use gpui_kit::base::text::TextView;
use gpui_kit::component::Disableable as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{InputEvent, Textarea, TextareaState};
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenuItem};
use gpui_kit::component::notification::Notification;
use gpui_kit::component::{Sizable as _, StyledExt as _, WindowExt as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_ai::{ClientSettings, Excerpt, Provider};
use kanso_core::when::TimeIntent;
use kanso_engine::ask::{
    self, AskRequest, AskScope, Banner, Conversation, Finished, NEVER_WROTE, NOTHING_MATCHES,
    Receipt, TimeNote, Turn, TurnEvent,
};
use kanso_store::Store;

use crate::ai_feature;
use crate::chat_parts::note_row;
use crate::clock::Clock;
use crate::config::{AgentChoice, AiBackend};
use crate::editor::{HIGHLIGHT_HOLD, NoteEditor};
use crate::library::Library;
use crate::page_chrome::{ai_error_card, ai_trouble};
use crate::services::Services;
use crate::settings_ai::{read_from, requests_line};
use crate::text_stats::plural;
use crate::theme::{Colors, Rgb, section_label};
use crate::workspace::AppState;
use crate::zoom::px;

/// What a conversation was asked under. A different provider, URL or model,
/// or another agent, starts a new one.
#[derive(Clone, Debug, PartialEq, Eq)]
struct ConversationKey {
    backend: AiBackend,
    provider: Provider,
    base_url: String,
    model: String,
    agent: AgentChoice,
}

impl ConversationKey {
    fn of(ai: &crate::config::AiSettings) -> Self {
        Self {
            backend: ai.backend,
            provider: ai.provider,
            base_url: ai.base_url.clone(),
            model: ai.model.clone(),
            agent: ai.agent.clone(),
        }
    }

    /// The key as the engine's conversation compares it.
    fn identity(&self) -> String {
        format!("{self:?}")
    }
}

/// What the background thread says as the answer is made.
enum Update {
    /// Notes the answer has drawn on: this turn's first excerpts, then any
    /// the model searched or read through a tool.
    Used(Vec<Excerpt>),
    Text(String),
    /// A line about what is happening: searching, reading, writing,
    /// checking, an agent starting, or one of its tools.
    Status(String),
    /// The time the question was understood to be about.
    Time(TimeNote),
    /// A checked answer.
    Done(Box<Finished>),
    /// A summary of a note ended; its words are in the panel.
    SummaryDone,
    Error(String),
    /// The stream stopped without its end marker.
    CutOff,
    /// Nothing in the notes matched, so no model was asked. `narrow` is
    /// whether the scope was narrower than all notes.
    Nothing {
        narrow: bool,
    },
}

/// One exchange as it is shown.
struct Exchange {
    question: String,
    finished: Finished,
    /// What the question was limited to, for Save as note.
    scope: AskScope,
    /// The note the answer was saved as, once it was.
    saved: Option<String>,
}

pub struct AskPanel {
    store: Arc<Store>,
    library: Entity<Library>,
    editor: Entity<NoteEditor>,
    question: Entity<TextareaState>,
    /// The question asked last, so the panel can tell "before the first".
    asked: bool,
    /// The exchange in progress: its question, and the words that arrived.
    asking: Option<String>,
    answer: String,
    /// The exchanges so far, kept in memory for the panel's lifetime only,
    /// with the rules that drop one whose notes are no longer eligible.
    conversation: Conversation,
    exchanges: Vec<Exchange>,
    /// The notes this turn's answer has drawn on, saved with it.
    turn_notes: Vec<String>,
    /// What the conversation was asked under, if it has one.
    key: Option<ConversationKey>,
    /// The agent's side of the conversation: one ACP session for all its
    /// questions, and that session's notes token. Dropped with the
    /// conversation, which closes the session and revokes the token.
    agent: Option<Arc<kanso_ai::agent::Conversation>>,
    /// The notes the turn in progress has used, listed while it runs.
    used: Vec<Excerpt>,
    cut_off: bool,
    searching: bool,
    streaming: bool,
    /// Nothing matched the last question; `narrow` says whether searching
    /// all notes may help.
    nothing: Option<bool>,
    error: Option<String>,
    /// The last status line, until the answer's words arrive.
    notice: Option<String>,
    /// What the next question searches.
    scope: AskScope,
    /// The time the running question was understood to be about.
    time: Option<TimeNote>,
    /// The question asked last, to try again or without its time range.
    last_question: Option<String>,
    cancel: Option<Arc<AtomicBool>>,
    task: Option<Task<()>>,
    /// The edits the agent suggested, kept apart from the conversation:
    /// clearing it doesn't drop them.
    reviews: Option<kanso_server::ReviewQueue>,
    _review_watch: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

impl AskPanel {
    pub fn new(
        store: Arc<Store>,
        library: Entity<Library>,
        editor: Entity<NoteEditor>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let question = cx.new(|cx| {
            TextareaState::new(window, cx)
                .placeholder("Ask about your notes…")
                .submit_on_enter(true)
        });
        let subscriptions = vec![cx.subscribe_in(
            &question,
            window,
            |this: &mut AskPanel, _, event: &InputEvent, window, cx| match event {
                InputEvent::PressEnter { shift: false, .. } => this.ask(window, cx),
                InputEvent::Change => cx.notify(),
                _ => {}
            },
        )];
        let reviews = crate::review_queue::queue(cx);
        let review_watch = reviews.as_ref().map(|queue| {
            let events = queue.subscribe();
            cx.spawn(async move |this, cx| {
                while events.recv().await.is_ok() {
                    if this.update(cx, |_, cx| cx.notify()).is_err() {
                        break;
                    }
                }
            })
        });
        AskPanel {
            store,
            library,
            editor,
            question,
            asked: false,
            asking: None,
            answer: String::new(),
            conversation: Conversation::default(),
            exchanges: Vec::new(),
            turn_notes: Vec::new(),
            key: None,
            agent: None,
            used: Vec::new(),
            cut_off: false,
            searching: false,
            streaming: false,
            nothing: None,
            error: None,
            notice: None,
            scope: AskScope::All,
            time: None,
            last_question: None,
            cancel: None,
            task: None,
            reviews,
            _review_watch: review_watch,
            _subscriptions: subscriptions,
        }
    }

    /// Puts the keyboard in the question box.
    pub fn focus_question(&self, window: &mut Window, cx: &mut App) {
        let focus = self.question.read(cx).focus_handle(cx);
        window.focus(&focus, cx);
    }

    /// Limits the next question to `scope`.
    pub fn set_scope(&mut self, scope: AskScope, cx: &mut Context<Self>) {
        self.scope = scope;
        cx.notify();
    }

    /// Asks the question in the box as the next turn of this conversation.
    pub fn ask(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let question = self.question.read(cx).value().trim().to_owned();
        if question.is_empty() {
            return;
        }
        self.ask_with(question, false, window, cx);
    }

    /// Asks `question` as the next turn. `ignore_time` leaves out the time
    /// range the question was understood to have.
    fn ask_with(
        &mut self,
        question: String,
        ignore_time: bool,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let ai = cx.global::<AppState>().config.ai.clone();
        if !ai.enabled {
            return;
        }
        // Whatever is answering stops first, even if the new request cannot
        // be made: a failure to build the client must not leave an answer
        // running.
        self.stop(cx);
        // A locked note shown beside the panel ends the conversation.
        if self.editor.read(cx).lock_view() != crate::locked::LockView::Plain {
            self.clear_conversation();
        }
        let ended = self.begin_conversation(&ai);

        self.asked = true;
        self.answer.clear();
        self.nothing = None;
        self.error = None;
        self.notice = ended.then(|| ask::KEPT_OUT_ENDED.to_owned());
        self.cut_off = false;
        self.searching = true;
        self.time = None;
        self.used.clear();
        self.turn_notes.clear();
        self.asking = Some(question.clone());
        self.last_question = Some(question.clone());
        let cancel = Arc::new(AtomicBool::new(false));
        self.cancel = Some(cancel.clone());
        cx.notify();
        crate::metrics::record(cx, crate::metrics::Feature::Ai);

        let store = self.store.clone();
        let scope = self.scope.clone();
        let history = self.conversation.history();
        let previous = self.conversation.previous();
        let now = Clock::now_ms(cx);
        let offset_ms = i64::from(Clock::utc_offset_secs(cx, now)) * 1000;
        let mut request = AskRequest::new(
            &question,
            kanso_core::dates::Date::from_local_ms(now + offset_ms),
        );
        request.scope = scope;
        request.utc_offset_ms = offset_ms;
        let zone = Clock::zone(cx);
        request.zone = Some(ask::ZoneOffsets(std::sync::Arc::new(move |at_ms| {
            i64::from(zone(at_ms)) * 1000
        })));
        request.budget = ai.ask.budget_for(ai.provider, &ai.base_url);
        request.use_links = ai.ask.use_links;
        request.ignore_time = ignore_time;
        request.previous = previous;
        let check_sources = ai.ask.check_sources;
        let secrets = cx
            .try_global::<Services>()
            .and_then(|services| services.secrets.clone());
        let worker_ai = ai.clone();
        // An agent keeps one session for the conversation; it is opened on
        // this question if there isn't one yet.
        let agent = (ai.backend == AiBackend::Agent).then(|| {
            if self.agent.is_none() {
                let data_dir = cx.global::<AppState>().data_dir.clone();
                let settings = crate::ai_agent::agent_settings(&ai.agent, &data_dir);
                let notes = crate::ai_agent::notes_access(cx);
                self.agent = Some(kanso_ai::agent::conversation(&settings, notes));
            }
            self.agent.clone()
        });
        let agent = agent.flatten();
        let (tx, rx) = async_channel::unbounded::<Update>();
        let spawned = std::thread::Builder::new()
            .name("kanso-ask".into())
            .spawn(move || {
                let on_event = {
                    let tx = tx.clone();
                    move |event: TurnEvent| {
                        let update = match event {
                            TurnEvent::Status(line) => Update::Status(line),
                            TurnEvent::Used(used) => Update::Used(used),
                            TurnEvent::Time(time) => Update::Time(time),
                            TurnEvent::Text(text) => Update::Text(text),
                            TurnEvent::Done(done) => Update::Done(done),
                            TurnEvent::NothingMatches { narrow } => Update::Nothing { narrow },
                            TurnEvent::Failed(message) => Update::Error(message),
                            TurnEvent::CutOff => Update::CutOff,
                            // The panel already settled itself when it stopped.
                            TurnEvent::Stopped => return,
                        };
                        tx.try_send(update).ok();
                    }
                };
                if let Some(agent) = agent {
                    run_agent(
                        &store,
                        &agent,
                        request,
                        &history,
                        check_sources,
                        &cancel,
                        on_event,
                    );
                    return;
                }
                let provider = {
                    // The Keychain read and the client build happen here,
                    // never on the UI thread: the macOS prompt can block, and
                    // building a reqwest client is not work for a frame.
                    let saved = secrets
                        .as_ref()
                        .and_then(|secrets| read_from(secrets.as_ref(), worker_ai.provider));
                    let settings = ClientSettings {
                        provider: worker_ai.provider,
                        base_url: worker_ai.base_url.clone(),
                        model: worker_ai.model.clone(),
                        api_key: saved,
                        region: worker_ai.provider.default_region().to_owned(),
                    };
                    match kanso_ai::cached_client(&settings) {
                        Ok(provider) => provider,
                        Err(err) => {
                            tx.try_send(Update::Error(err.to_string())).ok();
                            return;
                        }
                    }
                };
                ask::run(
                    Turn {
                        store,
                        provider,
                        request,
                        history,
                        check_sources,
                        cancel,
                    },
                    on_event,
                );
            });
        if spawned.is_err() {
            self.cancel = None;
            self.asking = None;
            self.searching = false;
            self.error = Some("Couldn't start the question.".to_owned());
            cx.notify();
            return;
        }
        self.task = Some(Self::watch(rx, cx));
    }

    /// Starts the conversation, or carries it on: another provider, URL or
    /// model begins a new one, and an exchange whose notes can no longer be
    /// sent is dropped.
    fn begin_conversation(&mut self, ai: &crate::config::AiSettings) -> bool {
        let key = ConversationKey::of(ai);
        if self.key.as_ref().is_some_and(|last| last != &key) {
            self.clear_conversation();
        }
        let mut ended = false;
        let before = self.conversation.turns().len();
        let began = self.conversation.begin(&key.identity(), &self.store);
        if began == ask::Began::KeptOut {
            // A note it used was kept out since: the conversation ends, with
            // its agent session and what is shown.
            self.clear_conversation();
            self.conversation.begin(&key.identity(), &self.store);
            ended = true;
        }
        if self.conversation.turns().len() != before {
            // The agent's session holds what those turns said: it ends, and
            // the next question starts one with only what is still allowed.
            self.agent = None;
        }
        self.key = Some(key);
        // The shown exchanges follow the kept ones.
        let kept = self.conversation.turns().len();
        if self.exchanges.len() > kept {
            let questions: Vec<&str> = self
                .conversation
                .turns()
                .iter()
                .map(|turn| turn.question.as_str())
                .collect();
            self.exchanges
                .retain(|exchange| questions.contains(&exchange.question.as_str()));
        }
        ended
    }

    /// Carries what the worker says into the panel until the request is
    /// over, or settles the panel if the worker ended without saying so.
    fn watch(rx: async_channel::Receiver<Update>, cx: &mut Context<Self>) -> Task<()> {
        crate::chat_parts::watch(
            rx,
            cx,
            |this, update, cx| {
                match update {
                    Update::Used(used) => {
                        this.conversation
                            .note_used(&mut this.turn_notes, used.clone());
                        this.used = used;
                        this.searching = false;
                        this.streaming = true;
                    }
                    Update::Text(text) => {
                        this.notice = None;
                        this.answer.push_str(&text);
                    }
                    Update::Status(line) => this.notice = Some(line),
                    Update::Time(time) => this.time = Some(time),
                    Update::Done(done) => this.finish_turn(*done),
                    Update::SummaryDone => this.finish_summary(),
                    Update::CutOff => {
                        this.cut_off = true;
                        this.end_turn();
                    }
                    Update::Error(message) => {
                        this.error = Some(message);
                        this.end_turn();
                    }
                    Update::Nothing { narrow } => {
                        this.nothing = Some(narrow);
                        this.end_turn();
                    }
                }
                cx.notify();
                !this.searching && !this.streaming
            },
            // The worker or runtime ended without Done, Error or CutOff. Keep
            // Ask usable by settling the panel instead of leaving it running
            // forever.
            |this, cx| {
                if this.settle_after_closed_channel() {
                    cx.notify();
                }
            },
        )
    }

    /// Ends the turn without keeping an exchange.
    fn end_turn(&mut self) {
        self.streaming = false;
        self.searching = false;
        self.asking = None;
        self.turn_notes.clear();
        self.cancel = None;
        self.notice = None;
    }

    /// Keeps a checked answer as the newest exchange.
    fn finish_turn(&mut self, done: Finished) {
        if let Some(question) = self.asking.take() {
            let notes = std::mem::take(&mut self.turn_notes);
            if !done.raw.trim().is_empty() {
                self.conversation.finish_with(
                    question.clone(),
                    done.plain_answer(),
                    notes,
                    done.receipts.clone(),
                );
                self.exchanges.push(Exchange {
                    question,
                    finished: done,
                    scope: self.scope.clone(),
                    saved: None,
                });
            }
        }
        self.answer.clear();
        self.used.clear();
        self.time = None;
        self.turn_notes.clear();
        self.streaming = false;
        self.searching = false;
        self.cancel = None;
        self.notice = None;
    }

    /// Keeps a finished summary as the newest exchange.
    fn finish_summary(&mut self) {
        if let Some(question) = self.asking.take() {
            let answer = std::mem::take(&mut self.answer);
            if !answer.trim().is_empty() {
                let notes = std::mem::take(&mut self.turn_notes);
                self.conversation
                    .finish(question.clone(), answer.clone(), notes);
                let done = Finished {
                    markdown: ask::sanitize_answer(&answer, &[]),
                    raw: answer,
                    unchecked: true,
                    ..Finished::default()
                };
                self.exchanges.push(Exchange {
                    question,
                    finished: done,
                    scope: AskScope::All,
                    saved: None,
                });
            }
        }
        self.turn_notes.clear();
        self.streaming = false;
        self.searching = false;
        self.cancel = None;
    }

    /// Whether the note open beside the panel can be summarised: it is open
    /// in the clear and not in the trash. (Whether AI may see it is checked
    /// when the summary is asked for.)
    fn can_summarize(&self, cx: &App) -> bool {
        let editor = self.editor.read(cx);
        editor.lock_view() == crate::locked::LockView::Plain
            && editor
                .current_note(cx)
                .is_some_and(|note| note.book_id != kanso_core::model::TRASH_BOOK_ID)
    }

    /// Summarises the note open beside the panel as the next turn: the one
    /// note goes in the request, a long one in parts, and no note is
    /// changed. Nothing is sent for a note kept out of AI.
    pub fn summarize_note(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        let ai = cx.global::<AppState>().config.ai.clone();
        if !ai.enabled {
            return;
        }
        self.stop(cx);
        let Some(note) = self.editor.read(cx).current_note(cx) else {
            return;
        };
        let plain = self.editor.read(cx).lock_view() == crate::locked::LockView::Plain;
        let title = if note.title.trim().is_empty() {
            "Untitled".to_owned()
        } else {
            note.title.trim().to_owned()
        };
        // The same conversation rules as a question: another backend or model
        // starts afresh, and what can no longer be sent is dropped.
        let _ = self.begin_conversation(&ai);
        self.asked = true;
        self.answer.clear();
        self.nothing = None;
        self.error = None;
        self.notice = None;
        self.cut_off = false;
        self.searching = false;
        self.turn_notes.clear();
        self.used.clear();
        self.time = None;
        if !plain
            || note.book_id == kanso_core::model::TRASH_BOOK_ID
            || !ai_feature::eligible(&self.store, &note.id)
        {
            self.asking = None;
            self.streaming = false;
            self.error = Some(ai_feature::KEPT_OUT.to_owned());
            cx.notify();
            return;
        }
        let question = format!("Summary of \u{201c}{title}\u{201d}");
        self.asking = Some(question);
        self.turn_notes.push(note.id.clone());
        self.streaming = true;
        let cancel = Arc::new(AtomicBool::new(false));
        self.cancel = Some(cancel.clone());
        cx.notify();

        let maker = ai_feature::maker(cx);
        crate::metrics::record(cx, crate::metrics::Feature::Ai);
        let (tx, rx) = async_channel::unbounded::<Update>();
        let spawned = std::thread::Builder::new()
            .name("kanso-summary".into())
            .spawn(move || {
                ai_feature::summarize(&maker, &title, &note.body, &cancel.clone(), |event| {
                    let update = match event {
                        kanso_ai::Event::Text(text) => Update::Text(text),
                        kanso_ai::Event::Status(line) => Update::Status(line),
                        kanso_ai::Event::Tool {
                            title: Some(title), ..
                        } => Update::Status(title),
                        kanso_ai::Event::Tool { .. } | kanso_ai::Event::Thought(_) => return,
                        kanso_ai::Event::Done => Update::SummaryDone,
                        kanso_ai::Event::Error(message) if message == kanso_ai::CUT_OFF => {
                            Update::CutOff
                        }
                        kanso_ai::Event::Error(message) => Update::Error(message),
                    };
                    tx.try_send(update).ok();
                });
            });
        if spawned.is_err() {
            self.cancel = None;
            self.asking = None;
            self.streaming = false;
            self.error = Some("Couldn't start the summary.".to_owned());
            cx.notify();
            return;
        }
        self.task = Some(Self::watch(rx, cx));
    }

    /// Settles a request whose event channel closed without Done or Error.
    /// Returns whether it was still running (and so was settled).
    fn settle_after_closed_channel(&mut self) -> bool {
        if !self.searching && !self.streaming {
            return false;
        }
        self.cut_off = true;
        self.end_turn();
        true
    }

    /// Fills the panel with finished exchanges, for snapshots.
    #[cfg(test)]
    pub(crate) fn snapshot_exchanges(
        &mut self,
        exchanges: Vec<(&'static str, String, bool)>,
        scope: AskScope,
        cx: &mut Context<Self>,
    ) {
        self.asked = true;
        self.scope = scope.clone();
        for (question, raw, closest) in exchanges {
            self.exchanges.push(Exchange {
                question: question.to_owned(),
                finished: crate::snapshots::ask::finished_for(&raw, closest),
                scope: scope.clone(),
                saved: None,
            });
        }
        cx.notify();
    }

    /// How many suggested edits the panel says are waiting.
    #[cfg(test)]
    pub(crate) fn suggested_for_test(&self) -> usize {
        self.reviews.as_ref().map_or(0, |queue| queue.len())
    }

    /// The finished exchanges as (question, answer), for tests.
    #[cfg(test)]
    pub(crate) fn turns_for_test(&self) -> Vec<(String, String)> {
        self.conversation
            .turns()
            .iter()
            .map(|turn| (turn.question.clone(), turn.answer.clone()))
            .collect()
    }

    /// Whether a request is running, and the lines the panel shows for an
    /// error and for progress, for tests.
    #[cfg(test)]
    pub(crate) fn state_for_test(&self) -> (bool, Option<String>, Option<String>) {
        (
            self.searching || self.streaming,
            self.error.clone(),
            self.notice.clone(),
        )
    }

    #[cfg(test)]
    pub(crate) fn mark_running_for_test(&mut self) {
        self.searching = true;
        self.streaming = true;
        self.asking = Some("a question".to_owned());
        self.cut_off = false;
    }

    #[cfg(test)]
    pub(crate) fn settle_closed_for_test(&mut self) -> (bool, bool) {
        let settled = self.settle_after_closed_channel();
        (settled, self.cut_off)
    }

    /// Stops the answer being read. What arrived stays.
    pub fn stop(&mut self, cx: &mut Context<Self>) {
        if let Some(cancel) = self.cancel.take() {
            cancel.store(true, Ordering::Relaxed);
        }
        self.task = None;
        self.asking = None;
        self.searching = false;
        self.streaming = false;
        cx.notify();
    }

    /// Ends the conversation: cancels any answer and forgets every exchange.
    /// Called when the panel is cleared, closed, or AI is turned off.
    pub fn end_conversation(&mut self, cx: &mut Context<Self>) {
        self.stop(cx);
        self.clear_conversation();
        cx.notify();
    }

    fn clear_conversation(&mut self) {
        // The agent's session closes and its notes token is revoked.
        self.agent = None;
        self.conversation.clear();
        self.exchanges.clear();
        self.turn_notes.clear();
        self.key = None;
        self.used.clear();
        self.answer.clear();
        self.asking = None;
        self.asked = false;
        self.nothing = None;
        self.error = None;
        self.notice = None;
        self.cut_off = false;
        self.time = None;
    }

    /// Opens note `id` in the editor.
    fn open(&self, id: String, cx: &mut App) {
        self.editor
            .update(cx, |editor, _| editor.reveal_after_open(id.clone(), 0));
        self.library
            .update(cx, |library, cx| library.select(Some(id), cx));
    }

    /// Opens the lines a receipt points at, marked for three seconds. Lines
    /// that moved since the answer are found again; lines that are gone open
    /// the note and say so.
    fn open_receipt(&self, receipt: &Receipt, window: &mut Window, cx: &mut App) {
        let id = receipt.note_id.clone();
        let body = self
            .editor
            .read(cx)
            .note_id()
            .filter(|open| *open == id)
            .and_then(|_| self.editor.read(cx).current_note(cx))
            .map(|note| note.body)
            .or_else(|| self.store.note(&id).ok().flatten().map(|note| note.body));
        let found = body.and_then(|body| ask::relocate(&body, receipt));
        let (first, last) = match found {
            Some(lines) => lines,
            None => {
                window.push_notification(Notification::info(ask::LINES_CHANGED), cx);
                self.open(id, cx);
                return;
            }
        };
        self.open_lines(&id, first, last, window, cx);
    }

    /// Opens note `id` with lines `first` to `last` marked.
    fn open_lines(&self, id: &str, first: u32, last: u32, window: &mut Window, cx: &mut App) {
        if self.editor.read(cx).note_id() == Some(id) {
            self.editor.update(cx, |editor, cx| {
                editor.highlight_lines(first, last, HIGHLIGHT_HOLD, window, cx)
            });
        } else {
            self.editor.update(cx, |editor, _| {
                editor.highlight_after_open(id.to_owned(), first, last)
            });
            self.library
                .update(cx, |library, cx| library.select(Some(id.to_owned()), cx));
        }
    }

    /// Writes exchange `index` as a note: the one write Ask makes, and only
    /// because the person pressed the button.
    fn save_exchange(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(exchange) = self.exchanges.get(index) else {
            return;
        };
        if exchange.saved.is_some() {
            return;
        }
        match ask::save_answer(
            &self.store,
            &exchange.scope,
            &exchange.question,
            &exchange.finished,
        ) {
            Ok(id) => {
                self.exchanges[index].saved = Some(id);
                window.push_notification(Notification::info("Saved as a note."), cx);
            }
            Err(err) => {
                window.push_notification(
                    Notification::error(format!("Couldn't save the note: {err}")),
                    cx,
                );
            }
        }
        cx.notify();
    }

    /// Asks the last question again without its time range, as the range
    /// chip's ✕ does. The exchange it replaces is dropped.
    fn drop_time(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(question) = self.last_question.clone() else {
            return;
        };
        self.stop(cx);
        if self.exchanges.pop().is_some() {
            self.conversation.pop();
        }
        self.ask_with(question, true, window, cx);
    }

    /// Asks the last question again over all notes.
    fn search_all(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(question) = self.last_question.clone() else {
            return;
        };
        self.scope = AskScope::All;
        self.nothing = None;
        self.ask_with(question, false, window, cx);
    }

    /// Asks the last question again.
    fn retry(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(question) = self.last_question.clone() else {
            return;
        };
        self.error = None;
        self.ask_with(question, false, window, cx);
    }

    fn status(&self, cx: &App) -> Option<AnyElement> {
        let colors = Colors::get(cx);
        let line = |text: String, color: Rgb| {
            div()
                .text_size(px(12.5))
                .line_height(px(18.))
                .text_color(color)
                .child(text)
                .into_any_element()
        };
        if self.cut_off {
            return Some(line(kanso_ai::CUT_OFF.to_owned(), colors.text_secondary));
        }
        if self.nothing.is_some() {
            return Some(line(NOTHING_MATCHES.to_owned(), colors.text_secondary));
        }
        if self.searching {
            return Some(line(ask::SEARCHING.to_owned(), colors.text_secondary));
        }
        if let Some(notice) = &self.notice {
            return Some(line(notice.clone(), colors.text_secondary));
        }
        if self.answer.is_empty() && self.streaming {
            return Some(line("Thinking…".to_owned(), colors.text_secondary));
        }
        None
    }

    /// The scope chip: what the next question searches, and a menu to
    /// change it.
    fn scope_chip(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let store = self.store.clone();
        let paths = ask::book_paths(&self.store);
        let label = match &self.scope {
            AskScope::All => "All notes".to_owned(),
            AskScope::Book(id) => paths
                .get(id)
                .cloned()
                .unwrap_or_else(|| "A notebook".to_owned()),
            AskScope::Tag(name) => format!("#{name}"),
            AskScope::Notes(ids) => plural(ids.len(), "note", "notes"),
        };
        let current = self.scope.clone();
        let view = cx.entity();
        Button::new("ask-scope")
            .debug_selector(|| "ask-scope".into())
            .ghost()
            .small()
            .label(label)
            .dropdown_menu(move |mut menu, _, _| {
                let pick = |scope: AskScope| {
                    let view = view.clone();
                    move |_: &ClickEvent, _: &mut Window, cx: &mut App| {
                        view.update(cx, |this, cx| this.set_scope(scope.clone(), cx));
                    }
                };
                menu = menu.item(
                    PopupMenuItem::new("All notes")
                        .checked(current == AskScope::All)
                        .on_click(pick(AskScope::All)),
                );
                // Notebooks, by path; a locked one is never offered.
                let locked = store.locked_books().unwrap_or_default();
                let mut books: Vec<(String, String)> = ask::book_paths(&store)
                    .into_iter()
                    .filter(|(id, _)| !locked.contains(id))
                    .collect();
                books.sort_by_key(|book| book.1.to_lowercase());
                if !books.is_empty() {
                    menu = menu.separator();
                }
                for (id, path) in books {
                    let scope = AskScope::Book(id);
                    menu = menu.item(
                        PopupMenuItem::new(path)
                            .checked(current == scope)
                            .on_click(pick(scope)),
                    );
                }
                // Tags, except the one that keeps notes out of AI.
                let mut tags: Vec<String> = store
                    .tags()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|tag| tag.name)
                    .filter(|name| !ask::is_exclude_tag(name))
                    .collect();
                tags.sort_by_key(|name| name.to_lowercase());
                if !tags.is_empty() {
                    menu = menu.separator();
                }
                for name in tags {
                    let scope = AskScope::Tag(name.clone());
                    menu = menu.item(
                        PopupMenuItem::new(format!("#{name}"))
                            .checked(current == scope)
                            .on_click(pick(scope)),
                    );
                }
                menu
            })
    }

    /// The range chip with its ✕, and the line that says a range was
    /// relaxed.
    fn time_chip(&self, time: &TimeNote, removable: bool, cx: &mut Context<Self>) -> AnyElement {
        let colors = Colors::get(cx);
        let view = cx.entity();
        let text = match time.intent {
            TimeIntent::Written(_) => format!("Written {}", time.label),
            TimeIntent::About(_) => format!("About {}", time.label),
            TimeIntent::Either(_) => time.label.clone(),
        };
        v_flex()
            .gap(px(2.))
            .child(
                h_flex()
                    .debug_selector(|| "ask-time".into())
                    .gap(px(4.))
                    .items_center()
                    .child(
                        div()
                            .px(px(8.))
                            .py(px(2.))
                            .rounded(px(10.))
                            .bg(colors.fill)
                            .text_size(px(11.5))
                            .text_color(colors.text_secondary)
                            .child(text),
                    )
                    .when(removable, |el| {
                        el.child(
                            Button::new("ask-time-drop")
                                .debug_selector(|| "ask-time-drop".into())
                                .ghost()
                                .small()
                                .label("\u{2715}")
                                .tooltip("Search without this time range")
                                .on_click(move |_, window, cx| {
                                    view.update(cx, |this, cx| this.drop_time(window, cx));
                                }),
                        )
                    }),
            )
            .children(time.relaxed_line().map(|line| {
                div()
                    .text_size(px(11.5))
                    .text_color(colors.text_muted)
                    .child(line)
            }))
            .into_any_element()
    }

    fn render_used(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        if self.used.is_empty() {
            return None;
        }
        let colors = Colors::get(cx);
        let view = cx.entity();
        let ids: Vec<String> = self.used.iter().map(|e| e.id.clone()).collect();
        Some(
            v_flex()
                .gap(px(2.))
                .child(
                    section_label(div(), colors)
                        .text_size(px(11.5))
                        .text_color(colors.text_muted)
                        .child(format!("Used {}", plural(self.used.len(), "note", "notes"))),
                )
                .children(self.used.iter().enumerate().map(|(ix, excerpt)| {
                    let view = view.clone();
                    let id = excerpt.id.clone();
                    note_row(("ask-used", ix as u64), &excerpt.title, colors).on_click(
                        move |_, _, cx| {
                            view.update(cx, |this, cx| this.open(id.clone(), cx));
                        },
                    )
                }))
                .child(
                    div().mt(px(4.)).child(
                        Button::new("ask-draft-from")
                            .ghost()
                            .small()
                            .icon(gpui_kit::assets::IconName::PenTool)
                            .label("Draft from These Sources…")
                            .tooltip("Write a draft from the notes this answer used, with a source for each claim")
                            .on_click(move |_, _, cx| {
                                let ids = ids.clone();
                                crate::note_window::in_main_window(cx, move |workspace, window, cx| {
                                    workspace.show_learn(crate::learn_model::Mode::Draft, ids, window, cx)
                                });
                            }),
                    ),
                )
                .into_any_element(),
        )
    }

    /// One finished exchange: what was asked, the banner if one applies,
    /// the answer with its receipts, and the lists under it.
    fn render_exchange(
        &self,
        index: usize,
        last: bool,
        running: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let colors = Colors::get(cx);
        let view = cx.entity();
        let exchange = &self.exchanges[index];
        let finished = &exchange.finished;
        let mut column = v_flex().gap(px(8.)).child(
            div()
                .text_size(px(12.5))
                .font_semibold()
                .text_color(colors.text)
                .child(format!("You asked: {}", exchange.question.trim())),
        );
        if let Some(time) = &finished.time {
            column = column.child(self.time_chip(time, last && !running, cx));
        }
        if let Some(banner) = finished.banner {
            column = column.child(banner_row(banner, colors));
        }
        if finished.not_in_notes {
            column = column.child(
                div()
                    .text_size(px(13.))
                    .line_height(px(19.))
                    .text_color(colors.text)
                    .child(NEVER_WROTE),
            );
            if !finished.closest.is_empty() {
                let rows = finished.closest.iter().enumerate().map(|(ix, closest)| {
                    let view = view.clone();
                    let (id, first, last_line) =
                        (closest.note_id.clone(), closest.start, closest.end);
                    note_row(
                        ("ask-closest", (index * 10 + ix) as u64),
                        &closest.title,
                        colors,
                    )
                    .on_click(move |_, window, cx| {
                        view.update(cx, |this, cx| {
                            this.open_lines(&id, first, last_line, window, cx)
                        });
                    })
                });
                column = column.child(
                    v_flex()
                        .gap(px(2.))
                        .child(
                            section_label(div(), colors)
                                .text_size(px(11.5))
                                .child("Closest notes"),
                        )
                        .children(rows),
                );
            }
        } else if !finished.markdown.trim().is_empty() {
            let open = view.clone();
            let receipts = finished.receipts.clone();
            let markdown = with_marked_sentences(&with_superscripts(&finished.markdown));
            column = column.child(
                TextView::markdown(("ask-finished-text", index), markdown)
                    .selectable(true)
                    .on_link_click(move |url, _, window, cx| {
                        if let Some(n) = url.strip_prefix(ask::CITE_PREFIX) {
                            if let Some(receipt) =
                                receipts.iter().find(|r| n.parse() == Ok(r.n)).cloned()
                            {
                                open.update(cx, |this, cx| this.open_receipt(&receipt, window, cx));
                            }
                            return;
                        }
                        if url.starts_with(ask::UNVERIFIED_PREFIX) {
                            window.push_notification(Notification::info(ask::NOT_FOUND_HINT), cx);
                            return;
                        }
                        let Some(id) = url.strip_prefix(kanso_core::links::NOTE_LINK_PREFIX) else {
                            return;
                        };
                        let id = format!("note:{id}");
                        open.update(cx, |this, cx| this.open(id, cx));
                    }),
            );
        }
        if finished.unverified > 0 {
            column = column.child(
                div()
                    .text_size(px(11.5))
                    .text_color(colors.text_muted)
                    .child(ask::NOT_FOUND_HINT),
            );
        }
        if finished.no_source > 0 {
            column = column.child(
                div()
                    .text_size(px(11.5))
                    .text_color(colors.text_muted)
                    .child(no_source_line(finished.no_source)),
            );
        }
        if !finished.receipts.is_empty() {
            let rows = finished.receipts.iter().enumerate().map(|(ix, receipt)| {
                let view = view.clone();
                let target = receipt.clone();
                let quote = quote_card(&receipt.quote);
                h_flex()
                    .id(ElementId::NamedInteger(
                        "ask-source".into(),
                        (index * 100 + ix) as u64,
                    ))
                    .w_full()
                    .gap(px(6.))
                    .px(px(6.))
                    .py(px(4.))
                    .rounded(px(5.))
                    .cursor_pointer()
                    .hover(|el| el.bg(colors.row_hover))
                    .text_size(px(12.5))
                    .text_color(colors.link)
                    .tooltip(move |window, cx| {
                        gpui_kit::component::tooltip::Tooltip::new(quote.clone()).build(window, cx)
                    })
                    .child(div().w(px(14.)).child(superscript(receipt.n)))
                    .child(div().truncate().child(receipt.source_line()))
                    .on_click(move |_, window, cx| {
                        view.update(cx, |this, cx| this.open_receipt(&target, window, cx));
                    })
            });
            column = column.child(
                v_flex()
                    .gap(px(2.))
                    .child(
                        section_label(div(), colors)
                            .text_size(px(11.5))
                            .text_color(colors.text_muted)
                            .child("Sources"),
                    )
                    .children(rows),
            );
        }
        if !finished.also_read.is_empty() {
            let rows = finished.also_read.iter().enumerate().map(|(ix, excerpt)| {
                let view = view.clone();
                let id = excerpt.id.clone();
                note_row(
                    ("ask-also", (index * 100 + ix) as u64),
                    &excerpt.title,
                    colors,
                )
                .on_click(move |_, _, cx| {
                    view.update(cx, |this, cx| this.open(id.clone(), cx));
                })
            });
            column = column.child(
                v_flex()
                    .gap(px(2.))
                    .child(
                        section_label(div(), colors)
                            .text_size(px(11.5))
                            .text_color(colors.text_muted)
                            .child("Also read"),
                    )
                    .children(rows),
            );
        }
        if last && !running && !finished.not_in_notes {
            column = column.child(match &exchange.saved {
                Some(id) => {
                    let (view, id) = (view.clone(), id.clone());
                    h_flex()
                        .gap(px(8.))
                        .items_center()
                        .child(
                            div()
                                .text_size(px(12.))
                                .text_color(colors.text_muted)
                                .child("Saved as a note."),
                        )
                        .child(
                            Button::new("ask-open-saved")
                                .ghost()
                                .small()
                                .label("Open")
                                .on_click(move |_, _, cx| {
                                    view.update(cx, |this, cx| this.open(id.clone(), cx));
                                }),
                        )
                        .into_any_element()
                }
                None => Button::new("ask-save")
                    .debug_selector(|| "ask-save".into())
                    .outline()
                    .small()
                    .label("Save as note")
                    .on_click(move |_, window, cx| {
                        view.update(cx, |this, cx| this.save_exchange(index, window, cx));
                    })
                    .into_any_element(),
            });
        }
        column.into_any_element()
    }
}

/// Asks an agent the question: the same retrieval and prompt as any
/// provider, the lines the agent reads through the notes server numbered
/// after the prompt's, and the same checking of what it cites.
fn run_agent(
    store: &Arc<Store>,
    agent: &Arc<kanso_ai::agent::Conversation>,
    request: AskRequest,
    history: &[kanso_ai::Turn],
    check_sources: bool,
    cancel: &Arc<AtomicBool>,
    mut on_event: impl FnMut(TurnEvent) + Send,
) {
    on_event(TurnEvent::Status(ask::SEARCHING.to_owned()));
    let prepared = match ask::prepare(store, &request) {
        Ok(prepared) => prepared,
        Err(ask::AskError::Scope(refusal)) => {
            on_event(TurnEvent::Failed(refusal.message().to_owned()));
            return;
        }
        Err(ask::AskError::Store(err)) => {
            on_event(TurnEvent::Failed(format!(
                "Couldn't search your notes: {err}"
            )));
            return;
        }
    };
    if prepared.retrieval.is_empty() {
        on_event(TurnEvent::NothingMatches {
            narrow: request.scope.is_narrow(),
        });
        return;
    }
    if let Some(time) = &prepared.retrieval.time {
        on_event(TurnEvent::Time(time.clone()));
    }
    on_event(TurnEvent::Status(ask::reading_line(
        prepared.numbered.sent.len(),
    )));
    on_event(TurnEvent::Used(prepared.shown.used()));
    agent.number_from(prepared.numbered.sent.clone());
    // What the agent reaches through the notes server is checked again by
    // that server (eligibility, scope); the panel only lists it.
    let used = prepared.shown.clone();
    let on_read: kanso_ai::agent::OnRead = {
        let store = store.clone();
        let scope = prepared.scope.clone();
        let used = used.clone();
        Box::new(move |id: &str| {
            if ask::note_is_eligible(&store, id)
                && let Ok(Some(note)) = store.note(id)
                && scope.allows(&note.id, &note.book_id, &[])
            {
                used.note_used(id, &note.title);
            }
        })
    };
    let events = agent.ask(&prepared.numbered.prompt, history, on_read, cancel.clone());
    let mut raw = String::new();
    let mut ended = false;
    let mut failed = false;
    let mut wrote = false;
    ask::stream_events(events, cancel, |event| match event {
        kanso_ai::Event::Text(piece) => {
            if !wrote {
                wrote = true;
                on_event(TurnEvent::Status(ask::WRITING.to_owned()));
            }
            raw.push_str(&piece);
            on_event(TurnEvent::Text(piece));
        }
        kanso_ai::Event::Status(line) => on_event(TurnEvent::Status(line)),
        kanso_ai::Event::Tool {
            title: Some(title), ..
        } => on_event(TurnEvent::Status(title)),
        kanso_ai::Event::Tool { .. } | kanso_ai::Event::Thought(_) => {}
        kanso_ai::Event::Done => ended = true,
        kanso_ai::Event::Error(message) if message == kanso_ai::CUT_OFF => {}
        kanso_ai::Event::Error(message) => {
            failed = true;
            on_event(TurnEvent::Failed(message));
        }
    });
    if failed {
        return;
    }
    if cancel.load(Ordering::Relaxed) {
        return;
    }
    if !ended {
        on_event(TurnEvent::CutOff);
        return;
    }
    if raw.trim().is_empty() {
        on_event(TurnEvent::Failed("Nothing came back.".to_owned()));
        return;
    }
    // The excerpts the prompt carried and the lines the agent read, under
    // the numbers it was given.
    let shown = ask::Shown::new();
    for excerpt in agent.shown() {
        shown.register(excerpt);
    }
    for note in used.used() {
        shown.note_used(&note.id, &note.title);
    }
    if shown.sent().is_empty() {
        shown.start(&prepared.numbered);
    }
    on_event(TurnEvent::Used(shown.used()));
    if check_sources {
        on_event(TurnEvent::Status(ask::CHECKING.to_owned()));
    }
    on_event(TurnEvent::Done(Box::new(ask::finalize(
        &raw,
        &shown,
        check_sources,
        &prepared.retrieval,
    ))));
}

/// The banner over an answer that could not be checked.
fn banner_row(banner: Banner, colors: &Colors) -> AnyElement {
    div()
        .debug_selector(|| "ask-banner".into())
        .w_full()
        .px(px(10.))
        .py(px(6.))
        .rounded(px(6.))
        .border_1()
        .border_color(colors.warning)
        .text_size(px(12.))
        .line_height(px(17.))
        .text_color(colors.text)
        .child(banner.text())
        .into_any_element()
}

/// What hovering a source shows: the lines it points at.
fn quote_card(quote: &str) -> String {
    let mut text: String = quote.chars().take(400).collect();
    if quote.chars().count() > 400 {
        text.push('…');
    }
    text
}

/// `n` in superscript digits.
fn superscript(n: usize) -> String {
    n.to_string()
        .chars()
        .map(|digit| match digit {
            '0' => '\u{2070}',
            '1' => '\u{b9}',
            '2' => '\u{b2}',
            '3' => '\u{b3}',
            '4' => '\u{2074}',
            '5' => '\u{2075}',
            '6' => '\u{2076}',
            '7' => '\u{2077}',
            '8' => '\u{2078}',
            '9' => '\u{2079}',
            other => other,
        })
        .collect()
}

/// `markdown` with each receipt link `[3](kanso-cite:3)` written as a
/// superscript number, `[³](kanso-cite:3)`, so receipts read as footnotes.
fn with_superscripts(markdown: &str) -> String {
    let mut out = String::with_capacity(markdown.len());
    let mut rest = markdown;
    while let Some(at) = rest.find("](kanso-cite:") {
        let before = &rest[..at];
        let digits_at = before
            .rfind('[')
            .filter(|open| before[open + 1..].chars().all(|c| c.is_ascii_digit()));
        match digits_at {
            Some(open) if open + 1 < at => {
                let n: usize = before[open + 1..].parse().unwrap_or(0);
                out.push_str(&before[..open + 1]);
                out.push_str(&superscript(n));
            }
            _ => out.push_str(before),
        }
        out.push_str("](kanso-cite:");
        rest = &rest[at + "](kanso-cite:".len()..];
    }
    out.push_str(rest);
    out
}

/// "2 sentences cite no lines of your notes."
fn no_source_line(count: usize) -> String {
    if count == 1 {
        "1 sentence cites no lines of your notes (shown in italics).".to_owned()
    } else {
        format!("{count} sentences cite no lines of your notes (shown in italics).")
    }
}

/// `markdown` with the sentences that need a second look drawn so they read
/// so without a click: a sentence whose receipts failed gets a warning sign
/// before it (it stays a link that explains itself), and a sentence with no
/// source is set in italics, lighter than the cited ones.
fn with_marked_sentences(markdown: &str) -> String {
    const FAILED: &str = "](kanso-unverified:";
    const NO_SOURCE: &str = "](kanso-nosource:";
    let mut text = markdown.to_owned();
    let mut from = 0;
    while let Some(found) = text[from..].find(NO_SOURCE) {
        let at = from + found;
        let (Some(open), Some(close)) = (text[..at].rfind('['), text[at..].find(')')) else {
            break;
        };
        let sentence = text[open + 1..at].to_owned();
        text.replace_range(open..at + close + 1, &format!("*{sentence}*"));
        from = open + sentence.len() + 2;
    }
    let mut out = String::with_capacity(text.len() + 8);
    let mut rest = text.as_str();
    while let Some(at) = rest.find(FAILED) {
        match rest[..at].rfind('[') {
            Some(open) => {
                out.push_str(&rest[..open]);
                out.push_str("\u{26a0} ");
                out.push_str(&rest[open..at + FAILED.len()]);
            }
            None => out.push_str(&rest[..at + FAILED.len()]),
        }
        rest = &rest[at + FAILED.len()..];
    }
    out.push_str(rest);
    out
}

impl Render for AskPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let ai = cx.global::<AppState>().config.ai.clone();
        let running = self.searching || self.streaming;
        // Only the turn in progress is shown per chunk; finished exchanges
        // were rendered once, when they ended.
        let streaming = if self.answer.is_empty() {
            String::new()
        } else {
            ask::streaming_markdown(&self.answer, &self.used)
        };
        let used = self.render_used(cx);
        let view = cx.entity();
        let suggested = self.reviews.as_ref().map_or(0, |queue| queue.len());
        let count = self.exchanges.len();
        let exchanges: Vec<AnyElement> = (0..count)
            .map(|index| self.render_exchange(index, index + 1 == count, running, cx))
            .collect();
        let live_time = self
            .time
            .clone()
            .filter(|_| running)
            .map(|time| self.time_chip(&time, false, cx));
        let error_card = self.error.clone().map(|error| {
            if error == ask::ScopeRefusal::Locked.message() || error == ai_feature::KEPT_OUT {
                div()
                    .text_size(px(12.5))
                    .line_height(px(18.))
                    .text_color(colors.danger)
                    .child(error)
                    .into_any_element()
            } else {
                let (retry, dismiss) = (view.clone(), view.clone());
                ai_error_card(
                    &ai_trouble(&error),
                    move |window, cx| retry.update(cx, |this, cx| this.retry(window, cx)),
                    move |_, cx| {
                        dismiss.update(cx, |this, cx| {
                            this.error = None;
                            cx.notify();
                        })
                    },
                    window,
                    cx,
                )
                .into_any_element()
            }
        });
        v_flex()
            .id("ask")
            .size_full()
            .px(px(10.))
            .pt(px(10.))
            .pb(px(10.))
            .gap(px(8.))
            .when(suggested > 0, |el| {
                el.child(
                    h_flex()
                        .gap(px(8.))
                        .flex_wrap()
                        .items_center()
                        .child(
                            div()
                                .debug_selector(|| "ask-suggested".into())
                                .text_size(px(12.5))
                                .text_color(colors.text)
                                .child(format!(
                                    "{} —",
                                    crate::review_queue::pending_line(suggested)
                                )),
                        )
                        .child(
                            Button::new("ask-review")
                                .outline()
                                .small()
                                .label("Review")
                                .on_click(|_, window, cx| {
                                    crate::review_queue::open_review(window, cx);
                                }),
                        ),
                )
            })
            .when(
                !self.asked
                    && (ai.backend == AiBackend::Agent || ai.provider.is_remote(&ai.base_url)),
                |el| {
                    let line = if ai.backend == AiBackend::Agent {
                        crate::ai_agent::requests_line(&ai.agent)
                    } else {
                        requests_line(ai.provider, &ai.base_url)
                    };
                    el.child(
                        div()
                            .text_size(px(11.5))
                            .line_height(px(16.))
                            .text_color(colors.text_muted)
                            .child(line),
                    )
                },
            )
            .child(
                div()
                    .debug_selector(|| "ask-question".into())
                    .child(Textarea::new(&self.question).h(px(78.))),
            )
            .child(
                h_flex()
                    .gap(px(8.))
                    .flex_wrap()
                    .items_center()
                    .child(
                        Button::new("ask-send")
                            .outline()
                            .small()
                            .label("Ask")
                            .disabled(running)
                            .on_click({
                                let view = view.clone();
                                move |_, window, cx| {
                                    view.update(cx, |this, cx| this.ask(window, cx));
                                }
                            }),
                    )
                    .child(self.scope_chip(cx))
                    .when(!running && ai.enabled && self.can_summarize(cx), |el| {
                        let view = view.clone();
                        el.child(
                            Button::new("ask-summarize")
                                .ghost()
                                .small()
                                .label("Summarize this note")
                                .on_click(move |_, window, cx| {
                                    view.update(cx, |this, cx| this.summarize_note(window, cx));
                                }),
                        )
                    })
                    .when(running, |el| {
                        let view = view.clone();
                        el.child(
                            Button::new("ask-stop")
                                .outline()
                                .small()
                                .label("Stop")
                                .on_click(move |_, _, cx| {
                                    view.update(cx, |this, cx| this.stop(cx));
                                }),
                        )
                    })
                    .when(
                        !running && (self.asked || !self.exchanges.is_empty()),
                        |el| {
                            let view = view.clone();
                            el.child(
                                Button::new("ask-clear")
                                    .outline()
                                    .small()
                                    .label("Clear")
                                    .on_click(move |_, _, cx| {
                                        view.update(cx, |this, cx| this.end_conversation(cx));
                                    }),
                            )
                        },
                    ),
            )
            .when(!self.asked, |el| {
                el.child(
                    div()
                        .text_size(px(12.5))
                        .line_height(px(18.))
                        .text_color(colors.text_secondary)
                        .child("A question is answered from your own notes, with a receipt for each thing it says. The answer isn't saved unless you save it, and no note is changed."),
                )
            })
            .children(live_time)
            .children(self.status(cx))
            .when_some(error_card, |el, card| el.child(card))
            .when(self.nothing == Some(true), |el| {
                let view = view.clone();
                el.child(
                    Button::new("ask-search-all")
                        .debug_selector(|| "ask-search-all".into())
                        .outline()
                        .small()
                        .label("Search all notes")
                        .on_click(move |_, window, cx| {
                            view.update(cx, |this, cx| this.search_all(window, cx));
                        }),
                )
            })
            .when(!exchanges.is_empty() || !streaming.is_empty(), |el| {
                let mut answers = v_flex().gap(px(14.)).children(exchanges);
                if !streaming.is_empty() {
                    let open = view.clone();
                    answers = answers.child(
                        TextView::markdown("ask-answer-text", streaming)
                            .selectable(true)
                            .on_link_click(move |url, _, _, cx| {
                                let Some(id) = url.strip_prefix(kanso_core::links::NOTE_LINK_PREFIX)
                                else {
                                    return;
                                };
                                let id = format!("note:{id}");
                                open.update(cx, |this, cx| this.open(id, cx));
                            }),
                    );
                }
                el.child(
                    div()
                        .id("ask-answer")
                        .flex_1()
                        .min_h_0()
                        .overflow_y_scrollbar()
                        .child(answers),
                )
            })
            .when_some(used, |el, used| el.child(used))
    }
}

#[cfg(test)]
mod tests {
    use super::{ConversationKey, superscript, with_marked_sentences, with_superscripts};
    use crate::config::AiSettings;
    use kanso_ai::Provider;
    use proptest::prelude::*;

    fn ai(provider: Provider, base_url: &str, model: &str) -> AiSettings {
        AiSettings {
            provider,
            base_url: base_url.to_owned(),
            model: model.to_owned(),
            ..AiSettings::default()
        }
    }

    #[test]
    fn a_different_provider_url_or_model_is_a_different_conversation() {
        let start = ConversationKey::of(&ai(
            Provider::LocalModel,
            "http://localhost:11434/v1",
            "llama3.2",
        ));
        let same = ConversationKey::of(&ai(
            Provider::LocalModel,
            "http://localhost:11434/v1",
            "llama3.2",
        ));
        let provider = ConversationKey::of(&ai(
            Provider::OpenAi,
            "http://localhost:11434/v1",
            "llama3.2",
        ));
        let url = ConversationKey::of(&ai(
            Provider::LocalModel,
            "https://example.com/v1",
            "llama3.2",
        ));
        let model = ConversationKey::of(&ai(
            Provider::LocalModel,
            "http://localhost:11434/v1",
            "gemma",
        ));
        assert_eq!(start, same);
        assert_eq!(start.identity(), same.identity());
        assert_ne!(start, provider);
        assert_ne!(start, url);
        assert_ne!(start, model);
        assert_ne!(start.identity(), model.identity());
    }

    #[test]
    fn receipts_read_as_superscripts() {
        assert_eq!(superscript(12), "\u{b9}\u{b2}");
        assert_eq!(
            with_superscripts("Yearly [1](kanso-cite:1) and monthly [12](kanso-cite:12)."),
            "Yearly [\u{b9}](kanso-cite:1) and monthly [\u{b9}\u{b2}](kanso-cite:12)."
        );
        // A link that is not a receipt is left alone.
        assert_eq!(
            with_superscripts("[Pricing](inkdrop://note/a) [x](kanso-cite:x)"),
            "[Pricing](inkdrop://note/a) [x](kanso-cite:x)"
        );
    }

    #[test]
    fn failed_and_unsourced_sentences_are_drawn_apart_before_any_click() {
        let shown = with_marked_sentences(
            "Fine [\u{b9}](kanso-cite:1) [Made up.](kanso-unverified:1) *Not found in your notes* [Maybe.](kanso-nosource:2) End.",
        );
        // A failed sentence has a warning sign and stays a link that explains.
        assert!(
            shown.contains("\u{26a0} [Made up.](kanso-unverified:1)"),
            "{shown}"
        );
        // An unsourced one is italic and no longer a link.
        assert!(shown.contains("*Maybe.*"), "{shown}");
        assert!(!shown.contains("kanso-nosource"), "{shown}");
        // Nothing else moves.
        assert!(shown.starts_with("Fine [\u{b9}](kanso-cite:1) "), "{shown}");
        assert!(shown.ends_with(" End."), "{shown}");
        // An answer with neither is untouched.
        assert_eq!(
            with_marked_sentences("Plain [x](kanso-cite:1)."),
            "Plain [x](kanso-cite:1)."
        );
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: 64, ..ProptestConfig::default() })]

        /// Every receipt keeps its link target, and no digit is left in the
        /// link text.
        #[test]
        fn prop_superscripts_keep_every_target(numbers in proptest::collection::vec(1usize..400, 0..6)) {
            let markdown: String = numbers
                .iter()
                .map(|n| format!("word [{n}](kanso-cite:{n}) "))
                .collect();
            let out = with_superscripts(&markdown);
            for n in &numbers {
                let want = format!("[{}](kanso-cite:{n})", superscript(*n));
                prop_assert!(out.contains(&want), "{out}");
            }
            prop_assert_eq!(out.matches("kanso-cite:").count(), numbers.len());
        }
    }
}
