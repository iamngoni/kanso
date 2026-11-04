//! Draft from Notes, Make Flashcards and Quiz Me (issue #198). Draft is a
//! page over the window, like Inbox triage. Make Flashcards and Quiz Me
//! are modals over the note (`modal.rs`), so closing them leaves the note
//! as it was. The three share the reading column and the card that says a
//! model failed (`page_chrome`).
//!
//! What a model is sent is only the notes the page names, and each is
//! checked again on the worker through the one rule Ask uses, so a note
//! locked or tagged `#ai-exclude` since the page opened is not sent.
//! Nothing is written until the person says so.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::component::input::{InputEvent, Textarea, TextareaState};
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenuItem};
use gpui_kit::component::notification::Notification;
use gpui_kit::component::{
    Disableable as _, Icon, Selectable as _, Sizable as _, StyledExt as _, WindowExt as _, h_flex,
    v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_ai::draft::SourceRef;
use kanso_ai::draft_model::{
    CheckedDraft, Graded, Length, QuizQuestion, Shape, Source, Suggestion,
};
use kanso_ai::prompt::Excerpt;
use kanso_engine::draft::{self as engine, SourceList};
use kanso_store::Store;

use crate::layout::BOTTOM_BAND_HEIGHT;
use crate::learn_model::{Mode, Pick, Quiz, Run, add_label, chosen, picks_from, runs, unsourced};
use crate::page_chrome::{self, ActionStyle, AiTrouble, HeaderAction};
use crate::theme::Colors;
use crate::workspace::{AppState, Workspace};
use crate::zoom::px;

mod modal;

#[cfg(test)]
pub use modal::{CARDS_WIDTH, QUIZ_WIDTH};

actions!(
    kanso,
    [
        CloseLearn,
        DraftFromNote,
        MakeFlashcardsFromNote,
        QuizMeOnNote
    ]
);

/// What the page asks of the workspace.
pub enum LearnEvent {
    /// Open the note, closing the page.
    Open(String),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    /// Choosing what to write.
    Setup,
    /// The model is working.
    Working,
    /// The result is on the page.
    Ready,
    /// It could not be done (see `trouble` or `notice`).
    Failed,
}

/// What a worker tells the page.
enum Update {
    Text(String),
    Status(String),
    Done,
    Error(String),
    Cards(Result<Vec<Suggestion>, String>),
    Questions(Result<Vec<QuizQuestion>, String>),
    Grade(Result<Graded, String>),
}

type Job = Box<dyn FnOnce(async_channel::Sender<Update>, Arc<AtomicBool>) + Send>;

pub struct LearnView {
    store: Arc<Store>,
    mode: Mode,
    pub visible: bool,
    phase: Phase,
    /// The notes the page is about.
    ids: Vec<String>,
    list: SourceList,
    /// Why the page cannot do its job, in plain words (no model involved).
    notice: Option<String>,
    trouble: Option<AiTrouble>,
    status: Option<String>,
    cancel: Arc<AtomicBool>,
    task: Option<Task<()>>,
    pane_width: Pixels,
    leading_inset: Pixels,
    focus: FocusHandle,
    // Draft
    shape: Shape,
    length: Length,
    request: Entity<TextareaState>,
    sources: Vec<Source>,
    raw: String,
    checked: CheckedDraft,
    /// Whether sentences with no source are saved too. Off, they are left out.
    keep_unsourced: bool,
    refs: Vec<SourceRef>,
    book: String,
    // Make Flashcards
    picks: Vec<Pick>,
    added: Option<usize>,
    /// The card being edited, with its two fields.
    editing: Option<usize>,
    edit_question: Entity<TextareaState>,
    edit_answer: Entity<TextareaState>,
    /// Drawn in a dialog over the note, not as the page.
    modal: bool,
    // Quiz Me
    excerpts: Vec<Excerpt>,
    quiz: Quiz,
    answer: Entity<TextareaState>,
    asked: String,
    grading: bool,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<LearnEvent> for LearnView {}

impl Drop for LearnView {
    /// A modal that goes away takes its model call with it.
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}

impl Focusable for LearnView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

/// Errors that are already plain words, not a model's.
fn is_plain(message: &str) -> bool {
    message == crate::ai_feature::KEPT_OUT
        || message.starts_with("This note")
        || message.starts_with("The model")
        || message.starts_with("None of these")
}

impl LearnView {
    pub fn new(store: Arc<Store>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let request = cx.new(|cx| {
            TextareaState::new(window, cx)
                .placeholder("What should it say? (optional for a report, an essay or an outline)")
        });
        let answer = cx.new(|cx| {
            TextareaState::new(window, cx)
                .placeholder("Your answer…")
                .submit_on_enter(true)
        });
        let edit_question = cx.new(|cx| TextareaState::new(window, cx).placeholder("Question"));
        let edit_answer = cx.new(|cx| TextareaState::new(window, cx).placeholder("Answer"));
        let subscriptions = vec![cx.subscribe_in(
            &answer,
            window,
            |this: &mut LearnView, _, event: &InputEvent, window, cx| match event {
                InputEvent::PressEnter { shift: false, .. } => this.check_answer(window, cx),
                InputEvent::Change => cx.notify(),
                _ => {}
            },
        )];
        LearnView {
            store,
            mode: Mode::Draft,
            visible: false,
            phase: Phase::Setup,
            ids: Vec::new(),
            list: SourceList::default(),
            notice: None,
            trouble: None,
            status: None,
            cancel: Arc::new(AtomicBool::new(false)),
            task: None,
            pane_width: px(900.),
            leading_inset: px(0.),
            focus: cx.focus_handle(),
            shape: Shape::Report,
            length: Length::Medium,
            request,
            sources: Vec::new(),
            raw: String::new(),
            checked: CheckedDraft::default(),
            keep_unsourced: false,
            refs: Vec::new(),
            book: "book:inbox".into(),
            picks: Vec::new(),
            added: None,
            editing: None,
            edit_question,
            edit_answer,
            modal: false,
            excerpts: Vec::new(),
            quiz: Quiz::default(),
            answer,
            asked: String::new(),
            grading: false,
            _subscriptions: subscriptions,
        }
    }

    /// The page's width and leading inset, for fitting its header.
    pub fn set_frame(&mut self, width: Pixels, inset: Pixels, cx: &mut Context<Self>) {
        if self.pane_width != width || self.leading_inset != inset {
            self.pane_width = width;
            self.leading_inset = inset;
            cx.notify();
        }
    }

    /// Stops whatever a model is doing for the page.
    pub fn stop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
        self.task = None;
    }

    /// Opens the page in `mode` for notes `ids`.
    pub fn show(
        &mut self,
        mode: Mode,
        ids: Vec<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.stop();
        self.mode = mode;
        self.visible = true;
        self.phase = Phase::Setup;
        self.notice = None;
        self.trouble = None;
        self.status = None;
        self.raw.clear();
        self.checked = CheckedDraft::default();
        self.picks.clear();
        self.added = None;
        self.editing = None;
        self.quiz = Quiz::default();
        self.grading = false;
        self.list = engine::resolve(&self.store, &ids);
        self.ids = ids;
        self.answer.update(cx, |a, cx| a.set_value("", window, cx));
        if !cx.global::<AppState>().config.ai.enabled {
            self.fail_plain("AI is off. Turn it on in Settings › AI to use this.");
        } else if self.list.notes.is_empty() {
            self.fail_plain(
                "None of these notes can be sent to AI. Locked notes and notes tagged #ai-exclude are never sent.",
            );
        } else if self.list.too_many {
            self.fail_plain("That's more than a draft can read. Pick up to 200 notes.");
        } else if mode != Mode::Draft {
            self.start(cx);
        }
        // A modal takes the keyboard when its dialog opens, and the note
        // gets it back when the dialog closes.
        if !self.modal {
            window.focus(&self.focus, cx);
        }
        cx.notify();
    }

    fn fail_plain(&mut self, message: &str) {
        self.notice = Some(message.to_owned());
        self.trouble = None;
        self.phase = Phase::Failed;
    }

    fn fail(&mut self, message: &str) {
        if is_plain(message) {
            self.fail_plain(message);
        } else {
            self.notice = None;
            self.trouble = Some(page_chrome::ai_trouble(message));
            self.phase = Phase::Failed;
        }
    }

    // MARK: Running

    /// Runs `job` on a worker thread and carries what it says to the page.
    fn run_job(&mut self, job: Job, cx: &mut Context<Self>) {
        self.stop();
        let cancel = Arc::new(AtomicBool::new(false));
        self.cancel = cancel.clone();
        let (tx, rx) = async_channel::unbounded::<Update>();
        let spawned = std::thread::Builder::new()
            .name("kanso-learn".into())
            .spawn({
                let tx = tx.clone();
                move || job(tx, cancel)
            });
        if spawned.is_err() {
            self.fail("Couldn't start. Try again.");
            return;
        }
        drop(tx);
        self.task = Some(cx.spawn(async move |this, cx| {
            while let Ok(update) = rx.recv().await {
                if this
                    .update(cx, |this, cx| {
                        this.apply(update);
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
            }
            // A worker that ended without saying so does not leave the
            // page working.
            this.update(cx, |this, cx| {
                if this.phase == Phase::Working && !this.cancel.load(Ordering::Relaxed) {
                    this.fail("Nothing came back.");
                    cx.notify();
                }
            })
            .ok();
        }));
    }

    fn apply(&mut self, update: Update) {
        match update {
            Update::Text(piece) => {
                self.status = None;
                self.raw.push_str(&piece);
                self.checked = engine::prepare(&self.raw, &self.sources);
            }
            Update::Status(line) => self.status = Some(line),
            Update::Done => {
                self.checked = engine::prepare(&self.raw, &self.sources);
                self.phase = Phase::Ready;
            }
            Update::Error(message) => self.fail(&message),
            Update::Cards(Ok(found)) => {
                if found.is_empty() {
                    self.fail_plain(
                        "The model found nothing in this note worth a card. Add a line like  Question :: Answer  yourself, or try again.",
                    );
                } else {
                    self.picks = picks_from(found);
                    self.phase = Phase::Ready;
                }
            }
            Update::Cards(Err(message)) | Update::Questions(Err(message)) => self.fail(&message),
            Update::Questions(Ok(asked)) => {
                self.quiz = Quiz::new(asked);
                self.phase = Phase::Ready;
            }
            Update::Grade(Ok(graded)) => {
                self.quiz.grade(graded);
                self.grading = false;
            }
            Update::Grade(Err(message)) => {
                self.grading = false;
                self.fail(&message);
            }
        }
    }

    /// Starts the page's model job: writing, suggesting or asking.
    fn start(&mut self, cx: &mut Context<Self>) {
        self.trouble = None;
        self.notice = None;
        self.status = None;
        let maker = crate::ai_feature::maker(cx);
        let store = self.store.clone();
        let ids = self.ids.clone();
        match self.mode {
            Mode::Draft => {
                let request = self.request.read(cx).value().trim().to_owned();
                if request.is_empty() && self.shape == Shape::Custom {
                    self.notice = Some("Say what the draft should be.".to_owned());
                    self.phase = Phase::Setup;
                    return;
                }
                let sources = engine::sources(&store, &ids, &request);
                if sources.is_empty() {
                    self.fail_plain(
                        "None of these notes can be sent to AI. Locked notes and notes tagged #ai-exclude are never sent.",
                    );
                    return;
                }
                self.refs = engine::refs(&sources);
                self.book = engine::default_book(&store, &self.refs);
                self.raw.clear();
                self.checked = CheckedDraft::default();
                self.sources = sources.clone();
                self.phase = Phase::Working;
                let (shape, length) = (self.shape, self.length);
                self.run_job(
                    Box::new(move |tx, cancel| {
                        let shared = match maker() {
                            Ok(shared) => shared,
                            Err(message) => {
                                tx.try_send(Update::Error(message)).ok();
                                return;
                            }
                        };
                        let provider = shared
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner());
                        let mut said = false;
                        kanso_ai::draft::write_draft(
                            provider.as_ref(),
                            shape,
                            &request,
                            length,
                            &sources,
                            &cancel,
                            |event| {
                                let update = match event {
                                    kanso_ai::Event::Text(piece) => {
                                        said = true;
                                        Update::Text(piece)
                                    }
                                    kanso_ai::Event::Status(line) => Update::Status(line),
                                    kanso_ai::Event::Done if said => Update::Done,
                                    kanso_ai::Event::Done => {
                                        Update::Error("Nothing came back.".into())
                                    }
                                    kanso_ai::Event::Error(message) => Update::Error(message),
                                };
                                tx.try_send(update).ok();
                            },
                        );
                    }),
                    cx,
                );
            }
            Mode::Make => {
                let Some(id) = ids.first().cloned() else {
                    return;
                };
                self.phase = Phase::Working;
                self.run_job(
                    Box::new(move |tx, cancel| {
                        let result = maker().and_then(|shared| {
                            let provider = shared
                                .lock()
                                .unwrap_or_else(|poisoned| poisoned.into_inner());
                            kanso_engine::flashcards::suggest(
                                &store,
                                provider.as_ref(),
                                &id,
                                &cancel,
                            )
                        });
                        tx.try_send(Update::Cards(result)).ok();
                    }),
                    cx,
                );
            }
            Mode::Quiz => {
                let excerpts = kanso_engine::flashcards::quiz_excerpts(&store, &ids);
                self.excerpts = excerpts.clone();
                self.phase = Phase::Working;
                self.run_job(
                    Box::new(move |tx, cancel| {
                        let result = maker().and_then(|shared| {
                            let provider = shared
                                .lock()
                                .unwrap_or_else(|poisoned| poisoned.into_inner());
                            kanso_engine::flashcards::quiz(provider.as_ref(), &excerpts, &cancel)
                        });
                        tx.try_send(Update::Questions(result)).ok();
                    }),
                    cx,
                );
            }
        }
        cx.notify();
    }

    fn check_answer(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        if self.mode != Mode::Quiz || self.grading || self.quiz.graded().is_some() {
            return;
        }
        let Some(question) = self.quiz.current().cloned() else {
            return;
        };
        let text = self.answer.read(cx).value().trim().to_owned();
        if text.is_empty() {
            return;
        }
        let Some(excerpt) = self
            .excerpts
            .get(question.excerpt.saturating_sub(1))
            .cloned()
        else {
            return;
        };
        self.asked = text.clone();
        self.grading = true;
        self.trouble = None;
        let maker = crate::ai_feature::maker(cx);
        self.run_job(
            Box::new(move |tx, cancel| {
                let result = maker().and_then(|shared| {
                    let provider = shared
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                    kanso_engine::flashcards::grade(
                        provider.as_ref(),
                        &question.question,
                        &excerpt,
                        &text,
                        &cancel,
                    )
                });
                tx.try_send(Update::Grade(result)).ok();
            }),
            cx,
        );
        cx.notify();
    }

    fn next_question(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.quiz.next();
        self.answer.update(cx, |a, cx| a.set_value("", window, cx));
        cx.notify();
    }

    /// Opens card `ix` for editing, keeping what was typed in another.
    fn begin_edit(&mut self, ix: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.end_edit(cx);
        let Some(pick) = self.picks.get(ix) else {
            return;
        };
        let (question, answer) = (pick.question.clone(), pick.answer.clone());
        self.edit_question
            .update(cx, |q, cx| q.set_value(question, window, cx));
        self.edit_answer
            .update(cx, |a, cx| a.set_value(answer, window, cx));
        self.editing = Some(ix);
        cx.notify();
    }

    /// Keeps the edit in progress, unless it left the question or the
    /// answer empty.
    fn end_edit(&mut self, cx: &mut Context<Self>) {
        let Some(ix) = self.editing.take() else {
            return;
        };
        let question = self.edit_question.read(cx).value().trim().to_owned();
        let answer = self.edit_answer.read(cx).value().trim().to_owned();
        if let Some(pick) = self.picks.get_mut(ix)
            && !question.is_empty()
            && !answer.is_empty()
        {
            pick.question = question;
            pick.answer = answer;
        }
    }

    /// Writes the ticked cards into the note, then says so.
    pub(crate) fn add_cards(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.end_edit(cx);
        let Some(id) = self.ids.first().cloned() else {
            return;
        };
        let cards = chosen(&self.picks);
        match kanso_engine::flashcards::add_cards(&self.store, &id, &cards) {
            Ok(n) => {
                self.added = Some(n);
                window.push_notification(
                    Notification::success(match n {
                        0 => "Those cards are already in the note.".to_owned(),
                        1 => "Added 1 card under Flashcards. Undo from the note.".to_owned(),
                        n => format!("Added {n} cards under Flashcards. Undo from the note."),
                    }),
                    cx,
                );
            }
            Err(message) => {
                window.push_notification(Notification::error(message), cx);
            }
        }
        cx.notify();
    }

    fn markdown(&self) -> String {
        let leave_out = if self.keep_unsourced {
            Vec::new()
        } else {
            kanso_ai::draft::unsourced_ranges(&self.checked)
        };
        engine::final_markdown(&self.checked.text, &self.refs, &leave_out)
    }

    fn save_draft(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let title = engine::title_of(&self.checked.text);
        match engine::save(
            &self.store,
            &title,
            &self.markdown(),
            &self.book,
            self.refs.len(),
        ) {
            Ok(id) => {
                window.push_notification(
                    Notification::success(format!("Saved “{title}”. Undo deletes it.")),
                    cx,
                );
                cx.emit(LearnEvent::Open(id));
            }
            Err(err) => {
                window.push_notification(Notification::error(err.to_string()), cx);
            }
        }
    }

    fn discard(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let view = cx.entity().downgrade();
        crate::dialogs::confirm(
            "Discard this draft?",
            "It hasn't been saved, so it will be gone.",
            "Discard",
            window,
            cx,
            move |_, cx| {
                view.update(cx, |this, cx| {
                    this.raw.clear();
                    this.checked = CheckedDraft::default();
                    this.phase = Phase::Setup;
                    cx.notify();
                })
                .ok();
            },
        );
    }

    // MARK: Drawing

    fn header(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let view = cx.entity().downgrade();
        let act =
            |id: &'static str,
             icon: Lucide,
             label: String,
             run: fn(&mut LearnView, &mut Window, &mut Context<LearnView>)| {
                let view = view.clone();
                HeaderAction::new(id, icon, label, move |window, cx| {
                    view.update(cx, |this, cx| run(this, window, cx)).ok();
                })
            };
        let mut actions: Vec<HeaderAction> = Vec::new();
        match (self.mode, self.phase) {
            (Mode::Draft, Phase::Setup) => actions.push(
                act(
                    "learn-write",
                    Lucide::Sparkles,
                    "Write Draft".into(),
                    |t, _, cx| t.start(cx),
                )
                .style(ActionStyle::Primary)
                .tooltip("Write a draft from the notes (⌘↩)"),
            ),
            (Mode::Draft, Phase::Working) => actions.push(
                act("learn-stop", Lucide::Close, "Stop".into(), |t, _, cx| {
                    t.stop();
                    t.phase = if t.raw.is_empty() {
                        Phase::Setup
                    } else {
                        Phase::Ready
                    };
                    cx.notify();
                })
                .style(ActionStyle::Outline),
            ),
            (Mode::Draft, Phase::Ready) => {
                actions.push(
                    act(
                        "learn-discard",
                        Lucide::Trash,
                        "Discard".into(),
                        |t, w, cx| t.discard(w, cx),
                    )
                    .tooltip("Throw the draft away"),
                );
                actions.push(
                    act("learn-copy", Lucide::Copy, "Copy".into(), |t, w, cx| {
                        cx.write_to_clipboard(ClipboardItem::new_string(t.markdown()));
                        w.push_notification(Notification::info("Copied the draft."), cx);
                    })
                    .tooltip("Copy the draft as Markdown"),
                );
                actions.push(
                    act(
                        "learn-again",
                        Lucide::RefreshCw,
                        "Draft Again".into(),
                        |t, _, cx| t.start(cx),
                    )
                    .tooltip("Write it again with the same settings"),
                );
                actions.push(
                    act(
                        "learn-save",
                        Lucide::FilePlus,
                        "Save as Note".into(),
                        |t, w, cx| t.save_draft(w, cx),
                    )
                    .style(ActionStyle::Primary)
                    .tooltip("Save the draft as a new note"),
                );
            }
            _ => {}
        }
        let close = HeaderAction::new("learn-close", Lucide::Close, "Close", |window, cx| {
            window.dispatch_action(Box::new(CloseLearn), cx)
        })
        .tooltip("Close (Esc)");
        page_chrome::page_header(
            self.mode.title(),
            self.leading_inset,
            self.pane_width,
            actions,
            Some(close),
            window,
            cx,
        )
        .border_b_1()
        .border_color(colors.divider)
        .bg(colors.list)
    }

    /// The column the content sits in: the page's reading column, or the
    /// whole width of a modal.
    fn column(&self) -> Div {
        if self.modal {
            v_flex().w_full()
        } else {
            page_chrome::column()
        }
    }

    fn section_label(text: &'static str, colors: &Colors) -> Div {
        div()
            .text_size(px(12.))
            .font_semibold()
            .text_color(colors.text_muted)
            .child(text)
    }

    fn sources_card(&self, colors: &Colors) -> Div {
        let names = self.list.notes.iter().take(8).map(|n| {
            h_flex()
                .gap(px(6.))
                .text_size(px(13.))
                .text_color(colors.text_secondary)
                .child(
                    Icon::new(Lucide::FileText)
                        .size(px(13.))
                        .text_color(colors.text_muted),
                )
                .child(div().truncate().child(if n.title.trim().is_empty() {
                    "Untitled".to_owned()
                } else {
                    n.title.clone()
                }))
        });
        let more = self.list.notes.len().saturating_sub(8);
        v_flex()
            .debug_selector(|| "learn-sources".into())
            .w_full()
            .p(px(14.))
            .gap(px(6.))
            .rounded(px(10.))
            .border_1()
            .border_color(colors.divider)
            .bg(colors.fill)
            .child(
                div()
                    .text_size(px(13.5))
                    .font_semibold()
                    .text_color(colors.text)
                    .child(self.list.summary()),
            )
            .children(names)
            .when(more > 0, |el| {
                el.child(
                    div()
                        .text_size(px(12.5))
                        .text_color(colors.text_muted)
                        .child(format!("and {more} more")),
                )
            })
            .child(
                div()
                    .mt(px(4.))
                    .text_size(px(12.))
                    .text_color(colors.text_muted)
                    .child("Only these notes are sent to your model. Locked notes and notes tagged #ai-exclude never are."),
            )
    }

    fn choice<T: Copy + PartialEq + 'static>(
        &self,
        id: &'static str,
        options: &[(&'static str, T)],
        current: T,
        set: fn(&mut LearnView, T),
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        h_flex()
            .gap(px(6.))
            .flex_wrap()
            .children(options.iter().enumerate().map(|(ix, (label, value))| {
                let (value, view) = (*value, cx.entity().downgrade());
                Button::new((id, ix))
                    .outline()
                    .small()
                    .selected(value == current)
                    .label(*label)
                    .debug_selector(move || format!("{id}-{ix}"))
                    .on_click(move |_, _, cx| {
                        view.update(cx, |this, cx| {
                            set(this, value);
                            cx.notify();
                        })
                        .ok();
                    })
            }))
    }

    fn render_setup(&self, cx: &mut Context<Self>) -> AnyElement {
        let colors = Colors::get(cx);
        let shapes: Vec<(&'static str, Shape)> =
            Shape::ALL.iter().map(|s| (s.label(), *s)).collect();
        let lengths: Vec<(&'static str, Length)> =
            Length::ALL.iter().map(|l| (l.label(), *l)).collect();
        page_chrome::column()
            .gap(px(16.))
            .pt(px(18.))
            .child(self.sources_card(colors))
            .child(
                v_flex()
                    .gap(px(6.))
                    .child(Self::section_label("Kind", colors))
                    .child(self.choice("learn-shape", &shapes, self.shape, |t, v| t.shape = v, cx)),
            )
            .child(
                v_flex()
                    .gap(px(6.))
                    .child(Self::section_label("Length", colors))
                    .child(self.choice(
                        "learn-length",
                        &lengths,
                        self.length,
                        |t, v| t.length = v,
                        cx,
                    )),
            )
            .child(
                v_flex()
                    .gap(px(6.))
                    .child(Self::section_label("Request", colors))
                    .child(Textarea::new(&self.request).h(px(96.))),
            )
            .when_some(self.notice.clone(), |el, notice| {
                el.child(
                    div()
                        .text_size(px(13.))
                        .text_color(colors.warning)
                        .child(notice),
                )
            })
            .into_any_element()
    }

    fn render_working(&self, what: &'static str, colors: &Colors) -> Div {
        h_flex()
            .gap(px(6.))
            .text_size(px(12.5))
            .text_color(colors.text_muted)
            .child(
                Icon::new(Lucide::Sparkles)
                    .size(px(13.))
                    .text_color(colors.accent),
            )
            .child(self.status.clone().unwrap_or_else(|| what.to_owned()))
    }

    fn render_draft(&self, cx: &mut Context<Self>) -> AnyElement {
        let colors = Colors::get(cx);
        let mut highlights: Vec<(std::ops::Range<usize>, HighlightStyle)> = Vec::new();
        let mut at = 0usize;
        for (piece, run) in runs(&self.checked.text, &self.checked.spans) {
            let end = at + piece.len();
            if run == Run::Unsourced {
                highlights.push((
                    at..end,
                    HighlightStyle {
                        underline: Some(UnderlineStyle {
                            thickness: px(1.),
                            color: Some(Hsla::from(colors.text_muted)),
                            wavy: false,
                        }),
                        ..Default::default()
                    },
                ));
            }
            at = end;
        }
        let missing = unsourced(&self.checked.spans);
        let books = self.store.books().unwrap_or_default();
        let current = books
            .iter()
            .find(|b| b.id == self.book)
            .map(|b| b.name.clone())
            .unwrap_or_else(|| "Inbox".to_owned());
        let view = cx.entity().downgrade();
        let ready = self.phase == Phase::Ready;
        page_chrome::column()
            .gap(px(12.))
            .pt(px(16.))
            .pb(px(24.))
            .when(!ready, |el| el.child(self.render_working("Writing…", colors)))
            .when(ready && missing > 0, |el| {
                let view = view.clone();
                el.child(
                    v_flex()
                        .gap(px(6.))
                        .child(
                            div()
                                .text_size(px(12.5))
                                .text_color(colors.text_muted)
                                .child(format!(
                                    "{missing} {} underlined: no source in your notes. {} left out when you save.",
                                    if missing == 1 { "sentence is" } else { "sentences are" },
                                    if missing == 1 { "It is" } else { "They are" },
                                )),
                        )
                        .child(
                            Checkbox::new("learn-keep-unsourced")
                                .checked(self.keep_unsourced)
                                .label(if missing == 1 {
                                    "Keep it anyway"
                                } else {
                                    "Keep them anyway"
                                })
                                .on_click(move |on: &bool, _, cx| {
                                    let on = *on;
                                    view.update(cx, |this, cx| {
                                        this.keep_unsourced = on;
                                        cx.notify();
                                    })
                                    .ok();
                                }),
                        ),
                )
            })
            .when(ready && self.checked.removed > 0, |el| {
                el.child(
                    div()
                        .text_size(px(12.5))
                        .text_color(colors.text_muted)
                        .child(format!(
                            "{} citation{} to a note that wasn't given was removed.",
                            self.checked.removed,
                            if self.checked.removed == 1 { "" } else { "s" }
                        )),
                )
            })
            .child(
                div()
                    .debug_selector(|| "learn-draft".into())
                    .w_full()
                    .p(px(16.))
                    .rounded(px(10.))
                    .border_1()
                    .border_color(colors.divider)
                    .bg(colors.fill)
                    .text_size(px(14.5))
                    .line_height(px(23.))
                    .text_color(colors.text)
                    .child(
                        StyledText::new(self.checked.text.clone()).with_highlights(highlights),
                    ),
            )
            .when(ready, |el| {
                el.child(
                    h_flex()
                        .gap(px(8.))
                        .items_center()
                        .child(
                            div()
                                .text_size(px(12.5))
                                .text_color(colors.text_muted)
                                .child("Save in"),
                        )
                        .child(
                            Button::new("learn-book")
                                .outline()
                                .small()
                                .icon(Lucide::Notebook)
                                .label(current)
                                .dropdown_caret(true)
                                .dropdown_menu(move |mut menu, _, _| {
                                    for book in books.iter().filter(|b| !b.id.starts_with("book:trash")) {
                                        let (view, id) = (view.clone(), book.id.clone());
                                        menu = menu.item(
                                            PopupMenuItem::new(book.name.clone()).on_click(
                                                move |_, _, cx| {
                                                    view.update(cx, |this, cx| {
                                                        this.book = id.clone();
                                                        cx.notify();
                                                    })
                                                    .ok();
                                                },
                                            ),
                                        );
                                    }
                                    menu
                                }),
                        ),
                )
            })
            .into_any_element()
    }

    fn render_cards(&self, cx: &mut Context<Self>) -> AnyElement {
        let colors = Colors::get(cx);
        let rows = self.picks.iter().enumerate().map(|(ix, pick)| {
            let view = cx.entity().downgrade();
            if self.editing == Some(ix) {
                let done = view.clone();
                return v_flex()
                    .debug_selector(move || format!("learn-card-{ix}"))
                    .w_full()
                    .p(px(12.))
                    .gap(px(8.))
                    .rounded(px(10.))
                    .border_1()
                    .border_color(colors.accent)
                    .bg(colors.fill)
                    .child(Textarea::new(&self.edit_question).h(px(64.)))
                    .child(Textarea::new(&self.edit_answer).h(px(80.)))
                    .child(
                        h_flex().justify_end().child(
                            Button::new(("learn-edit-done", ix))
                                .primary()
                                .small()
                                .label("Done")
                                .on_click(move |_, _, cx| {
                                    done.update(cx, |this, cx| {
                                        this.end_edit(cx);
                                        cx.notify();
                                    })
                                    .ok();
                                }),
                        ),
                    );
            }
            let edit = view.clone();
            v_flex()
                .debug_selector(move || format!("learn-card-{ix}"))
                .w_full()
                .p(px(12.))
                .gap(px(4.))
                .rounded(px(10.))
                .border_1()
                .border_color(colors.divider)
                .bg(colors.fill)
                .child(
                    h_flex()
                        .items_start()
                        .gap(px(8.))
                        .child(
                            div().flex_1().min_w_0().child(
                                Checkbox::new(ElementId::NamedInteger(
                                    "learn-pick".into(),
                                    ix as u64,
                                ))
                                .checked(pick.on)
                                .label(pick.question.clone())
                                .on_click(
                                    move |on: &bool, _, cx| {
                                        let on = *on;
                                        view.update(cx, |this, cx| {
                                            if let Some(p) = this.picks.get_mut(ix) {
                                                p.on = on;
                                            }
                                            cx.notify();
                                        })
                                        .ok();
                                    },
                                ),
                            ),
                        )
                        .when(self.added.is_none(), |el| {
                            el.child(
                                Button::new(("learn-edit", ix))
                                    .ghost()
                                    .small()
                                    .icon(Lucide::Pencil)
                                    .tooltip("Edit this card")
                                    .on_click(move |_, window, cx| {
                                        edit.update(cx, |this, cx| this.begin_edit(ix, window, cx))
                                            .ok();
                                    }),
                            )
                        }),
                )
                .child(
                    div()
                        .pl(px(26.))
                        .text_size(px(13.5))
                        .text_color(colors.text_secondary)
                        .child(pick.answer.clone()),
                )
                .when_some(pick.source.clone(), |el, quote| {
                    el.child(
                        div()
                            .pl(px(26.))
                            .text_size(px(12.))
                            .text_color(if pick.missing {
                                colors.warning
                            } else {
                                colors.text_muted
                            })
                            .child(if pick.missing {
                                format!("Not found in the note: “{quote}”")
                            } else {
                                match pick.line {
                                    Some(line) => format!("Line {line}: “{quote}”"),
                                    None => format!("“{quote}”"),
                                }
                            }),
                    )
                })
        });
        self.column()
            .gap(px(10.))
            .pt(px(16.))
            .pb(px(24.))
            .child(
                div()
                    .text_size(px(12.5))
                    .text_color(colors.text_muted)
                    .child(match self.added {
                        Some(_) => "Added. Open the note to see them under Flashcards.".to_owned(),
                        None => "Tick the cards to add. They go under a Flashcards heading in the note, as one edit you can undo."
                            .to_owned(),
                    }),
            )
            .children(rows)
            .into_any_element()
    }

    fn render_quiz(&self, cx: &mut Context<Self>) -> AnyElement {
        let colors = Colors::get(cx);
        if self.quiz.is_done() {
            return self
                .column()
                .gap(px(10.))
                .pt(px(24.))
                .child(
                    div()
                        .text_size(px(20.))
                        .font_semibold()
                        .text_color(colors.text)
                        .child("Done"),
                )
                .child(
                    div()
                        .debug_selector(|| "learn-quiz-summary".into())
                        .text_size(px(14.))
                        .text_color(colors.text_secondary)
                        .child(self.quiz.summary()),
                )
                .into_any_element();
        }
        let Some(question) = self.quiz.current() else {
            return div().into_any_element();
        };
        let graded = self.quiz.graded().cloned();
        let source = self
            .excerpts
            .get(question.excerpt.saturating_sub(1))
            .map(|e| e.title.clone());
        let id = self
            .excerpts
            .get(question.excerpt.saturating_sub(1))
            .map(|e| e.id.clone());
        let view = cx.entity().downgrade();
        let next_view = view.clone();
        self.column()
            .gap(px(12.))
            .pt(px(18.))
            .pb(px(24.))
            .child(
                div()
                    .text_size(px(12.))
                    .text_color(colors.text_muted)
                    .child(self.quiz.progress()),
            )
            .child(
                div()
                    .debug_selector(|| "learn-question".into())
                    .text_size(px(19.))
                    .line_height(px(27.))
                    .font_medium()
                    .text_color(colors.text)
                    .child(question.question.clone()),
            )
            .child(
                Textarea::new(&self.answer)
                    .h(px(110.))
                    .disabled(graded.is_some() || self.grading),
            )
            .when(graded.is_none(), |el| {
                el.child(
                    h_flex()
                        .gap(px(8.))
                        .child(
                            Button::new("learn-check")
                                .primary()
                                .small()
                                .label(if self.grading { "Checking…" } else { "Check Answer" })
                                .loading(self.grading)
                                .disabled(
                                    self.grading || self.answer.read(cx).value().trim().is_empty(),
                                )
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.check_answer(window, cx)
                                })),
                        )
                        .child(
                            div()
                                .text_size(px(12.))
                                .text_color(colors.text_muted)
                                .child("Return checks it. Your answer is compared with this one note only."),
                        ),
                )
            })
            .when_some(graded, |el, graded| {
                el.child(
                    v_flex()
                        .debug_selector(|| "learn-verdict".into())
                        .w_full()
                        .p(px(14.))
                        .gap(px(6.))
                        .rounded(px(10.))
                        .border_1()
                        .border_color(colors.divider)
                        .bg(colors.fill)
                        .child(
                            div()
                                .text_size(px(14.))
                                .font_semibold()
                                .text_color(colors.text)
                                .child(graded.verdict.label()),
                        )
                        .child(
                            div()
                                .text_size(px(13.5))
                                .text_color(colors.text_secondary)
                                .child(graded.comment),
                        )
                        .when_some(source.zip(id), |el, (title, id)| {
                            el.child(
                                Button::new("learn-source")
                                    .ghost()
                                    .small()
                                    .icon(Lucide::FileText)
                                    .label(format!("From {title}"))
                                    .on_click(move |_, _, cx| {
                                        view.update(cx, |_, cx| cx.emit(LearnEvent::Open(id.clone())))
                                            .ok();
                                    }),
                            )
                        })
                        .child(
                            div().mt(px(6.)).child(
                                Button::new("learn-next")
                                    .primary()
                                    .small()
                                    .label("Next Question")
                                    .on_click(move |_, window, cx| {
                                        next_view
                                            .update(cx, |this, cx| this.next_question(window, cx))
                                            .ok();
                                    }),
                            ),
                        ),
                )
            })
            .into_any_element()
    }

    /// What the view shows for its mode and phase, without a frame.
    fn body(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let colors = Colors::get(cx);
        match (self.phase, self.mode) {
            (Phase::Failed, _) => self.render_failed(cx, window),
            (Phase::Setup, Mode::Draft) => self.render_setup(cx),
            (Phase::Working, Mode::Draft) | (Phase::Ready, Mode::Draft) => self.render_draft(cx),
            (Phase::Working, _) => self
                .column()
                .pt(px(24.))
                .child(self.render_working(
                    if self.mode == Mode::Quiz {
                        if self.grading {
                            "Checking your answer…"
                        } else {
                            "Writing questions…"
                        }
                    } else {
                        "Reading the note…"
                    },
                    colors,
                ))
                .into_any_element(),
            (_, Mode::Make) => self.render_cards(cx),
            (_, Mode::Quiz) => self.render_quiz(cx),
        }
    }

    fn render_failed(&self, cx: &mut Context<Self>, window: &mut Window) -> AnyElement {
        let colors = Colors::get(cx);
        let view = cx.entity().downgrade();
        let column = self.column().pt(px(16.));
        if let Some(trouble) = &self.trouble {
            let retry = view.clone();
            let dismiss = view;
            return column
                .child(page_chrome::ai_error_card(
                    trouble,
                    move |window, cx| {
                        retry
                            .update(cx, |this, cx| {
                                if this.mode == Mode::Quiz && this.quiz.current().is_some() {
                                    this.phase = Phase::Ready;
                                    this.check_answer(window, cx);
                                } else {
                                    this.start(cx);
                                }
                            })
                            .ok();
                    },
                    move |_, cx| {
                        dismiss
                            .update(cx, |this, cx| {
                                this.trouble = None;
                                this.phase = if this.mode == Mode::Draft {
                                    Phase::Setup
                                } else if this.quiz.current().is_some() {
                                    Phase::Ready
                                } else {
                                    Phase::Failed
                                };
                                cx.notify();
                            })
                            .ok();
                    },
                    window,
                    cx,
                ))
                .into_any_element();
        }
        let (title, detail) = match self.mode {
            Mode::Draft => (
                "Nothing to draft from",
                self.notice.clone().unwrap_or_default(),
            ),
            Mode::Make => (
                "No cards to suggest",
                self.notice.clone().unwrap_or_default(),
            ),
            Mode::Quiz => ("No quiz", self.notice.clone().unwrap_or_default()),
        };
        column
            .child(page_chrome::page_empty(
                Lucide::Sparkles,
                title,
                detail,
                vec![
                    Button::new("learn-back")
                        .primary()
                        .label("Back to Notes")
                        .on_click(|_, window, cx| window.dispatch_action(Box::new(CloseLearn), cx))
                        .into_any_element(),
                ],
                cx,
            ))
            .text_color(colors.text)
            .into_any_element()
    }
}

impl LearnView {
    /// The keys the view answers to, in a line.
    fn hint(&self) -> &'static str {
        match self.mode {
            Mode::Draft => "⌘↩ write · Esc close",
            Mode::Make => "Space ticks a card · Esc close",
            Mode::Quiz => "↩ check · Esc close",
        }
    }

    fn render_page(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let colors = Colors::get(cx);
        let body = self.body(window, cx);
        let footer = self.hint();
        v_flex()
            .id("learn-view")
            .key_context("LearnView")
            .track_focus(&self.focus)
            .size_full()
            .bg(colors.editor)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                let k = &event.keystroke;
                if k.key == "escape" {
                    cx.stop_propagation();
                    window.dispatch_action(Box::new(CloseLearn), cx);
                } else if k.key == "enter"
                    && (k.modifiers.platform || k.modifiers.control)
                    && this.mode == Mode::Draft
                    && this.phase == Phase::Setup
                {
                    cx.stop_propagation();
                    this.start(cx);
                }
            }))
            .on_any_mouse_down(|_, _, cx| cx.stop_propagation())
            .child(self.header(window, cx))
            .child(
                div().flex_1().min_h_0().child(
                    div()
                        .id("learn-body")
                        .size_full()
                        .overflow_y_scroll()
                        .child(page_chrome::centred(body)),
                ),
            )
            .child(
                h_flex()
                    .h(px(BOTTOM_BAND_HEIGHT))
                    .flex_none()
                    .px(px(18.))
                    .border_t_1()
                    .border_color(colors.divider)
                    .bg(colors.list)
                    .text_size(px(12.))
                    .text_color(colors.text_muted)
                    .child(div().truncate().child(footer)),
            )
            .into_any_element()
    }
}

impl Render for LearnView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.modal {
            self.render_modal(window, cx)
        } else {
            self.render_page(window, cx)
        }
    }
}

// MARK: The workspace

impl Workspace {
    /// Opens the Draft, Make Flashcards or Quiz Me page for `ids`.
    pub(crate) fn show_learn(
        &mut self,
        mode: Mode,
        ids: Vec<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !cx.global::<AppState>().config.ai.enabled {
            window.push_notification(
                Notification::info("Turn on AI in Settings › AI to use this."),
                cx,
            );
            return;
        }
        if ids.is_empty() {
            return;
        }
        self.close_settings(window, cx);
        self.close_graph(window, cx);
        self.close_board(window, cx);
        self.close_triage(window, cx);
        self.close_flashcards(window, cx);
        self.set_focus_mode(false, window, cx);
        self.flush(cx);
        let view = match &self.learn {
            Some(view) => view.clone(),
            None => {
                let store = cx.global::<AppState>().store.clone();
                let view = cx.new(|cx| LearnView::new(store, window, cx));
                self._learn_events = Some(cx.subscribe_in(
                    &view,
                    window,
                    |this, _, event, window, cx| match event {
                        LearnEvent::Open(id) => {
                            this.close_learn(window, cx);
                            let id = id.clone();
                            this.library
                                .update(cx, |library, cx| library.select(Some(id), cx));
                            this.editor
                                .update(cx, |editor, cx| editor.focus_body(window, cx));
                        }
                    },
                ));
                self.learn = Some(view.clone());
                view
            }
        };
        view.update(cx, |view, cx| view.show(mode, ids, window, cx));
        self.learn_open = true;
        window.focus(&view.focus_handle(cx), cx);
        cx.notify();
    }

    pub(crate) fn close_learn(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.learn_open {
            return;
        }
        self.learn_open = false;
        if let Some(view) = &self.learn {
            view.update(cx, |view, _| {
                view.visible = false;
                view.stop();
            });
        }
        window.focus(&self.focus, cx);
        cx.notify();
    }

    /// The open note's ID, if there is one.
    fn open_note_id(&self, cx: &App) -> Option<String> {
        self.library.read(cx).selected.clone()
    }
}

/// The workspace's actions for the page.
pub fn on_actions(el: Div, cx: &mut Context<Workspace>) -> Div {
    el.on_action(cx.listener(|this, _: &CloseLearn, window, cx| this.close_learn(window, cx)))
        .on_action(cx.listener(|this, _: &DraftFromNote, window, cx| {
            if let Some(id) = this.open_note_id(cx) {
                let ids = this
                    .library
                    .read(cx)
                    .several_selected()
                    .unwrap_or_else(|| vec![id]);
                this.show_learn(Mode::Draft, ids, window, cx);
            }
        }))
        .on_action(cx.listener(|this, _: &MakeFlashcardsFromNote, window, cx| {
            if let Some(id) = this.open_note_id(cx) {
                this.show_learn_modal(Mode::Make, id, window, cx);
            }
        }))
        .on_action(cx.listener(|this, _: &QuizMeOnNote, window, cx| {
            if let Some(id) = this.open_note_id(cx) {
                this.show_learn_modal(Mode::Quiz, id, window, cx);
            }
        }))
}

/// The page over the window, beside the sidebar, while open.
pub fn overlay(workspace: &Workspace, leading: f32) -> Option<impl IntoElement> {
    let view = workspace.learn.clone().filter(|_| workspace.learn_open)?;
    Some(
        div()
            .absolute()
            .top_0()
            .left(px(leading))
            .right_0()
            .bottom_0()
            .debug_selector(|| "learn-overlay".into())
            .child(view),
    )
}
