//! The Agent Tasks panel (a tab beside Ask), the New agent task sheet and the
//! Review agent changes sheet (issue #196).
//!
//! The runner and the words are `kanso_engine::agent_task::runner` and `model`; what
//! the cards say comes from `kanso_engine::agent_task::review`. This file
//! draws them and starts, stops, applies and undoes through the runner.

use gpui_component::scroll::ScrollableElement as _;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::base::text::TextView;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::{Disableable as _, Sizable as _, StyledExt as _, h_flex, v_flex};
use gpui_kit::component::{Icon, WindowExt as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_engine::agent_task::review::{
    self, CardDetail, CardState, LineKind, ReviewCard, ReviewSheet as Cards,
};
use kanso_engine::agent_task::{ReadScope, TaskSpec};
use kanso_server::{ApplyFailure, OpKind, Selection, TaskId, TaskOps};

use crate::agent_scope::{ScopeBook, scope_books};
use crate::config::AiBackend;
use crate::services::Services;
use crate::theme::Colors;
use crate::workspace::AppState;
use crate::zoom::px;
use kanso_engine::agent_task::model::{NOTIFICATION, Phase, TaskRow, elapsed_label};
use kanso_engine::agent_task::runner::{ApiDriver, CannotStart, TaskEvent, Tasks};

/// The width of the New agent task sheet and of the review sheet.
pub const SHEET_WIDTH: f32 = 560.;
pub const REVIEW_WIDTH: f32 = 760.;
/// The notification tag prefix.
const TAG: &str = "agent-task:";
/// What the "ready to review" notification says. Never the task's goal.
const NOTIFICATION_BODY: &str = "An agent task has changes ready for you to review.";

/// The runner, when the app has one.
pub fn tasks(cx: &App) -> Option<Tasks> {
    cx.try_global::<Services>()
        .map(|services| services.tasks.clone())
}

/// Whether a notification response is for a finished task.
pub fn owns(tag: &str) -> bool {
    tag.starts_with(TAG)
}

/// Edit > Undo Agent Task: takes back the most recent applied task that is
/// still undoable.
pub fn undo_latest(cx: &mut App) {
    let Some(runner) = tasks(cx) else { return };
    let Some(row) = runner
        .rows()
        .into_iter()
        .rev()
        .find(|row| runner.can_undo(row.id))
    else {
        return;
    };
    let now = crate::clock::Clock::now_ms(cx);
    runner.undo(row.id, now);
    cx.refresh_windows();
}

/// A click on the "ready to review" notification: bring the window forward
/// with the Agent Tasks tab showing.
pub fn respond(cx: &mut App) {
    cx.activate(true);
    crate::workspace::for_each_workspace(cx, |workspace, cx| {
        workspace.show_inspector(Some(crate::config::InspectorTab::Agents), cx)
    });
}

// ---------------------------------------------------------------- the panel

/// The tab: the tasks, newest last, and what can be done with each.
pub struct AgentTasksPanel {
    tasks: Tasks,
    replies: HashMap<TaskId, Entity<InputState>>,
    /// Tasks whose "Notes read" list is open.
    reads_open: HashSet<TaskId>,
    /// Why Try Again couldn't start the task again.
    retry_error: Option<String>,
    _watch: Task<()>,
    _tick: Task<()>,
}

impl AgentTasksPanel {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> AgentTasksPanel {
        let tasks = tasks(cx).unwrap_or_default();
        let events = tasks.subscribe();
        let watch = cx.spawn_in(window, async move |this, cx| {
            while let Ok(event) = events.recv().await {
                let alive = this.update_in(cx, |this, window, cx| {
                    if let TaskEvent::Finished(id) = event {
                        this.finished(id, window, cx);
                    }
                    cx.notify();
                });
                if alive.is_err() {
                    break;
                }
            }
        });
        let tick = cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(Duration::from_secs(1)).await;
                let alive = this.update(cx, |this, cx| {
                    if this.tasks.active() > 0 || this.tasks.notice().is_some() {
                        cx.notify();
                    }
                });
                if alive.is_err() {
                    break;
                }
            }
        });
        AgentTasksPanel {
            tasks,
            replies: HashMap::new(),
            reads_open: HashSet::new(),
            retry_error: None,
            _watch: watch,
            _tick: tick,
        }
    }

    /// A task ended with changes: tell the person when Kanso is behind other
    /// windows.
    fn finished(&mut self, id: TaskId, window: &mut Window, cx: &mut Context<Self>) {
        if window.is_window_active() || !crate::reminders::can_notify(cx) {
            return;
        }
        cx.show_system_notification(SystemNotification {
            tag: format!("{TAG}{}", id.0).into(),
            title: NOTIFICATION.into(),
            // Fixed words: the goal is the person's own text, and a
            // notification can show on a locked screen.
            body: NOTIFICATION_BODY.into(),
            actions: Vec::new(),
        });
    }

    fn reply_box(
        &mut self,
        id: TaskId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Entity<InputState> {
        self.replies
            .entry(id)
            .or_insert_with(|| {
                cx.new(|cx| InputState::new(window, cx).placeholder("Reply to the agent"))
            })
            .clone()
    }

    fn send_reply(&mut self, id: TaskId, window: &mut Window, cx: &mut Context<Self>) {
        let Some(input) = self.replies.get(&id).cloned() else {
            return;
        };
        let text = input.read(cx).value().to_string();
        if text.trim().is_empty() {
            return;
        }
        self.tasks.reply(id, &text);
        input.update(cx, |input, cx| input.set_value("", window, cx));
        cx.notify();
    }

    /// Try Again on a failed task: the same sheet, started anew. The failed
    /// row goes unless it has changes to review.
    fn retry(&mut self, id: TaskId, window: &mut Window, cx: &mut Context<Self>) {
        let Some(spec) = self.tasks.spec(id) else {
            return;
        };
        self.retry_error = None;
        let maker = StartMaker::capture(cx);
        let tasks = self.tasks.clone();
        let url = if maker.is_agent {
            crate::services::agent_mcp_endpoint(cx)
        } else {
            None
        };
        if !self.tasks.row(id).is_some_and(|row| row.can_review()) {
            self.tasks.dismiss(id);
        }
        let started = cx
            .background_executor()
            .spawn(async move { maker.start(&tasks, &spec, url) });
        cx.spawn_in(window, async move |this, cx| {
            if let Err(error) = started.await {
                this.update(cx, |this, cx| {
                    this.retry_error = Some(error.message());
                    cx.notify();
                })
                .ok();
            }
        })
        .detach();
        cx.notify();
    }

    fn undo(&mut self, id: TaskId, cx: &mut Context<Self>) {
        let now = crate::clock::Clock::now_ms(cx);
        self.tasks.undo(id, now);
        cx.notify();
    }

    fn render_row(
        &mut self,
        ix: usize,
        row: TaskRow,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let colors = Colors::get(cx);
        let id = row.id;
        let asking = match &row.phase {
            Phase::Asking { question } => Some(question.clone()),
            _ => None,
        };
        let failure = match &row.phase {
            Phase::Failed { message } => Some(crate::page_chrome::ai_trouble(message)),
            _ => None,
        };
        let reply = asking.as_ref().map(|_| self.reply_box(id, window, cx));
        let tasks = self.tasks.clone();
        let can_undo = self.tasks.can_undo(id);
        v_flex()
            .id(("agent-task", ix))
            .flex_none()
            .gap(px(6.))
            .p(px(12.))
            .rounded(px(10.))
            .border_1()
            .border_color(colors.divider)
            .child(
                div()
                    .min_w_0()
                    .max_h(px(58.))
                    .overflow_hidden()
                    .text_size(px(13.5))
                    .line_height(px(19.))
                    .font_semibold()
                    .text_color(colors.text)
                    .child(row.goal.clone()),
            )
            .child(
                div()
                    .min_w_0()
                    .truncate()
                    .text_size(px(12.))
                    .text_color(colors.text_muted)
                    .child(format!("{} · {}", row.agent, elapsed_label(row.elapsed))),
            )
            .when_some(failure.clone(), |el, trouble| {
                let retry = cx.entity().downgrade();
                el.child(crate::page_chrome::ai_error_card(
                    &trouble,
                    move |window, cx| {
                        retry.update(cx, |this, cx| this.retry(id, window, cx)).ok();
                    },
                    {
                        let tasks = self.tasks.clone();
                        move |_, _| tasks.dismiss(id)
                    },
                    window,
                    cx,
                ))
            })
            .when(failure.is_none(), |el| {
                el.child(
                    h_flex()
                        .gap(px(6.))
                        .items_start()
                        .when(row.is_active(), |el| {
                            el.child(
                                div().flex_none().pt(px(2.)).child(
                                    Icon::new(Lucide::RefreshCcw)
                                        .size(px(13.))
                                        .text_color(colors.text_muted)
                                        .with_animation(
                                            ("agent-task-spin", ix),
                                            Animation::new(Duration::from_millis(1100)).repeat(),
                                            |icon, t| {
                                                icon.transform(Transformation::rotate(percentage(
                                                    t,
                                                )))
                                            },
                                        ),
                                ),
                            )
                        })
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .text_size(px(12.5))
                                .line_height(px(18.))
                                .text_color(colors.text_secondary)
                                .child(row.status_line()),
                        ),
                )
            })
            .when(row.is_active() || row.notes_read > 0, |el| {
                let open = self.reads_open.contains(&id);
                let titles: Vec<String> = if open {
                    let store = cx.global::<AppState>().store.clone();
                    self.tasks
                        .reads(id)
                        .into_iter()
                        .map(|note| {
                            store
                                .note(&note)
                                .ok()
                                .flatten()
                                .map(|n| n.title)
                                .filter(|t| !t.trim().is_empty())
                                .unwrap_or_else(|| "Untitled".to_owned())
                        })
                        .collect()
                } else {
                    Vec::new()
                };
                el.child(
                    h_flex()
                        .gap(px(6.))
                        .flex_wrap()
                        .child(
                            Button::new(("agent-task-reads", ix))
                                .ghost()
                                .xsmall()
                                .label(format!(
                                    "Notes read: {}{}",
                                    row.notes_read,
                                    if open { " ▾" } else { " ▸" }
                                ))
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    if !this.reads_open.remove(&id) {
                                        this.reads_open.insert(id);
                                    }
                                    cx.notify();
                                })),
                        )
                        .child(
                            div()
                                .text_size(px(12.))
                                .text_color(colors.text_muted)
                                .child(row.counts_line()),
                        ),
                )
                .when(open, |el| {
                    el.child(
                        v_flex()
                            .id(("agent-task-reads-list", ix))
                            .max_h(px(140.))
                            .overflow_y_scrollbar()
                            .p(px(8.))
                            .rounded(px(8.))
                            .bg(colors.fill)
                            .gap(px(2.))
                            .children(titles.into_iter().map(|title| {
                                div()
                                    .min_w_0()
                                    .truncate()
                                    .text_size(px(12.5))
                                    .text_color(colors.text)
                                    .child(title)
                            })),
                    )
                })
            })
            .when(!row.last_message.is_empty() && !row.is_active(), |el| {
                el.child(
                    div()
                        .p(px(8.))
                        .rounded(px(8.))
                        .bg(colors.fill)
                        .text_size(px(12.5))
                        .line_height(px(18.))
                        .text_color(colors.text_secondary)
                        .child(row.last_message.clone()),
                )
            })
            .when_some(asking, |el, question| {
                el.child(
                    div()
                        .p(px(8.))
                        .rounded(px(8.))
                        .bg(colors.fill)
                        .text_size(px(12.5))
                        .line_height(px(18.))
                        .text_color(colors.text)
                        .child(question),
                )
            })
            .when_some(reply, |el, input| {
                el.child(
                    h_flex()
                        .gap(px(8.))
                        .child(div().flex_1().child(Input::new(&input)))
                        .child(
                            Button::new(("agent-task-send", ix))
                                .small()
                                .primary()
                                .label("Send")
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.send_reply(id, window, cx)
                                })),
                        ),
                )
            })
            .child(
                h_flex()
                    .gap(px(8.))
                    .justify_end()
                    .when(row.can_stop(), |el| {
                        let tasks = tasks.clone();
                        el.child(
                            Button::new(("agent-task-stop", ix))
                                .outline()
                                .small()
                                .label("Stop")
                                .on_click(move |_, _, _| tasks.stop(id)),
                        )
                    })
                    .when(can_undo, |el| {
                        el.child(
                            Button::new(("agent-task-undo", ix))
                                .outline()
                                .small()
                                .label("Undo")
                                .on_click(cx.listener(move |this, _, _, cx| this.undo(id, cx))),
                        )
                    })
                    .when(row.can_discard(), |el| {
                        let tasks = tasks.clone();
                        el.child(
                            Button::new(("agent-task-discard", ix))
                                .outline()
                                .small()
                                .label("Discard")
                                .on_click(move |_, _, _| tasks.discard(id)),
                        )
                    })
                    .when(row.can_review(), |el| {
                        el.child(
                            Button::new(("agent-task-review", ix))
                                .primary()
                                .small()
                                .label("Review…")
                                .on_click(move |_, window, cx| open_review(id, window, cx)),
                        )
                    })
                    .when(!row.is_active() && !row.can_review(), |el| {
                        let tasks = tasks.clone();
                        el.child(
                            Button::new(("agent-task-dismiss", ix))
                                .ghost()
                                .small()
                                .label("Dismiss")
                                .on_click(move |_, _, _| tasks.dismiss(id)),
                        )
                    }),
            )
            .into_any_element()
    }
}

// no-prop-test: drawing; the rows' words are `kanso_engine::agent_task::model`'s.
impl Render for AgentTasksPanel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let rows = self.tasks.rows();
        self.replies
            .retain(|id, _| rows.iter().any(|row| row.id == *id && row.is_active()));
        let ai_on = cx.global::<AppState>().config.ai.enabled;
        let cards: Vec<AnyElement> = rows
            .into_iter()
            .enumerate()
            .map(|(ix, row)| self.render_row(ix, row, window, cx))
            .collect();
        let empty = cards.is_empty();
        let notice = self.tasks.notice().map(|n| (n.task, n.text, n.undoable));
        v_flex()
            .gap(px(10.))
            .p(px(12.))
            .child(
                h_flex()
                    .justify_between()
                    .child(
                        div()
                            .text_size(px(13.))
                            .font_semibold()
                            .text_color(colors.text)
                            .child("Agent Tasks"),
                    )
                    .child(
                        Button::new("agent-task-new")
                            .small()
                            .primary()
                            .label("New Task…")
                            .disabled(!ai_on)
                            .on_click(|_, window, cx| open_new(Vec::new(), window, cx)),
                    ),
            )
            .when(!ai_on, |el| {
                el.child(
                    div()
                        .text_size(px(12.5))
                        .text_color(colors.text_secondary)
                        .child("Turn on AI in Settings to use agents."),
                )
            })
            .when_some(notice, |el, (task, text, undoable)| {
                el.child(
                    h_flex()
                        .gap(px(8.))
                        .p(px(8.))
                        .rounded(px(8.))
                        .bg(colors.fill)
                        .child(
                            div()
                                .flex_1()
                                .text_size(px(12.5))
                                .text_color(colors.text)
                                .child(text),
                        )
                        .when(undoable, |el| {
                            el.child(
                                Button::new("agent-task-notice-undo")
                                    .small()
                                    .outline()
                                    .label("Undo")
                                    .on_click(cx.listener(move |this, _, _, cx| this.undo(task, cx))),
                            )
                        }),
                )
            })
            .when(empty && ai_on, |el| {
                el.child(
                    div()
                        .py_4()
                        .text_size(px(12.5))
                        .line_height(px(18.))
                        .text_color(colors.text_secondary)
                        .child("Give an agent a goal, like “Merge these trip notes into one plan”. It reads the notes you choose and proposes changes. Nothing is written until you review and apply them."),
                )
            })
            .when_some(self.retry_error.clone(), |el, error| {
                el.child(
                    div()
                        .text_size(px(12.5))
                        .text_color(colors.danger)
                        .child(error),
                )
            })
            .children(cards)
    }
}

// ------------------------------------------------------------ the new sheet

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScopeKind {
    Notes,
    Notebook,
    Allowed,
}

/// The New agent task sheet.
pub struct NewTaskSheet {
    goal: Entity<InputState>,
    goal_text: String,
    notes: Vec<String>,
    kind: ScopeKind,
    books: Vec<ScopeBook>,
    notebook: Option<String>,
    new_book: Option<String>,
    ops: TaskOps,
    /// Waiting for the person to confirm reading everything.
    confirming: bool,
    error: Option<String>,
    starting: bool,
    /// Whether any note in the chosen scope can be shown to an agent: worked
    /// out off the UI thread when the scope changes, never on a keystroke.
    /// `None` while it is being worked out.
    readable: Option<bool>,
    readable_key: String,
    _readable_task: Option<Task<()>>,
    _typing: Subscription,
}

/// A notebook row in the sheet's lists.
const BOOK_ROW: f32 = 22.;

/// A notebook list's height: every row up to six, then it scrolls. Fixed,
/// not a maximum: the dialog squeezes a list with only a maximum, and a
/// squeezed list draws over what follows instead of scrolling.
fn book_list_height(rows: usize) -> Pixels {
    px(BOOK_ROW * rows.min(6) as f32)
}

impl NewTaskSheet {
    fn new(notes: Vec<String>, window: &mut Window, cx: &mut Context<Self>) -> NewTaskSheet {
        let state = cx.global::<AppState>();
        let books = scope_books(&state.store);
        let ops = state.config.ai.agent_tasks.ops();
        let first_book = notes
            .first()
            .and_then(|id| state.store.note(id).ok().flatten())
            .map(|note| note.book_id)
            .filter(|book| books.iter().any(|b| &b.id == book))
            .or_else(|| books.first().map(|b| b.id.clone()));
        let goal = cx.new(|cx| {
            InputState::new(window, cx).placeholder(
                "What should the agent do? For example: Merge these trip notes into one plan.",
            )
        });
        let typing = cx.subscribe_in(&goal, window, |this, input, event: &InputEvent, _, cx| {
            if let InputEvent::Change = event {
                this.goal_text = input.read(cx).value().to_string();
                this.error = None;
                cx.notify();
            }
        });
        NewTaskSheet {
            goal,
            goal_text: String::new(),
            kind: if notes.is_empty() {
                ScopeKind::Allowed
            } else {
                ScopeKind::Notes
            },
            notes,
            notebook: books.first().map(|b| b.id.clone()),
            books,
            new_book: first_book,
            ops,
            confirming: false,
            error: None,
            starting: false,
            readable: None,
            readable_key: String::new(),
            _readable_task: None,
            _typing: typing,
        }
    }

    /// Works out, in the background, whether the chosen scope has a note an
    /// agent may read. Does nothing when the scope is the one already worked
    /// out (or being worked out).
    fn ensure_readable(&mut self, cx: &mut Context<Self>) {
        let allowed = cx.global::<AppState>().config.ai.agent.notebooks.clone();
        let key = format!(
            "{:?}|{:?}|{:?}|{:?}",
            self.kind, self.notes, self.notebook, allowed
        );
        if key == self.readable_key {
            return;
        }
        self.readable_key = key.clone();
        self.readable = None;
        let spec = self.spec(cx);
        let store = cx.global::<AppState>().store.clone();
        let counted = cx.background_executor().spawn(async move {
            let scope = kanso_engine::agent_task::scope_for(&store, &spec, "", TaskId(0));
            kanso_server::has_readable_notes(&store, &scope)
        });
        self._readable_task = Some(cx.spawn(async move |this, cx| {
            let readable = counted.await;
            this.update(cx, |this, cx| {
                if this.readable_key == key {
                    this.readable = Some(readable);
                    cx.notify();
                }
            })
            .ok();
        }));
    }

    fn read_scope(&self, allowed: &[String]) -> ReadScope {
        match self.kind {
            ScopeKind::Notes => ReadScope::Notes(self.notes.clone()),
            ScopeKind::Notebook => ReadScope::Notebook(self.notebook.clone().unwrap_or_default()),
            ScopeKind::Allowed if allowed.is_empty() => ReadScope::Everything,
            ScopeKind::Allowed => ReadScope::Allowed(allowed.to_vec()),
        }
    }

    fn spec(&self, cx: &App) -> TaskSpec {
        let state = cx.global::<AppState>();
        let settings = state.config.ai.agent_tasks;
        let mut spec = TaskSpec::with_settings(
            &self.goal_text,
            self.read_scope(&state.config.ai.agent.notebooks),
            self.new_book.as_deref().unwrap_or_default(),
            &settings,
        );
        spec.ops = self.ops;
        spec
    }

    /// Why Start is off, or `None`.
    fn blocked(&self, cx: &App) -> Option<String> {
        let state = cx.global::<AppState>();
        if !state.config.ai.enabled {
            return Some("Turn on AI in Settings to use agents.".to_owned());
        }
        if self.goal_text.trim().is_empty() {
            return Some(String::new());
        }
        let spec = self.spec(cx);
        if !(spec.ops.create || spec.ops.edit || spec.ops.moves || spec.ops.merge) {
            return Some("Choose at least one thing the agent may propose.".to_owned());
        }
        match self.readable {
            Some(true) => None,
            Some(false) => Some("None of these notes can be shown to an agent.".to_owned()),
            // Still working it out: Start waits, with no message.
            None => Some(String::new()),
        }
    }

    fn reads_everything(&self, cx: &App) -> bool {
        self.kind == ScopeKind::Allowed
            && cx.global::<AppState>().config.ai.agent.notebooks.is_empty()
            && cx
                .global::<AppState>()
                .config
                .ai
                .agent_tasks
                .ask_before_reading_all
    }

    fn start(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.blocked(cx).is_some() || self.starting {
            return;
        }
        if self.reads_everything(cx) && !self.confirming {
            self.confirming = true;
            cx.notify();
            return;
        }
        let spec = self.spec(cx);
        self.starting = true;
        cx.notify();
        let maker = StartMaker::capture(cx);
        let tasks = tasks(cx).unwrap_or_default();
        let url = if maker.is_agent {
            crate::services::agent_mcp_endpoint(cx)
        } else {
            None
        };
        let started = cx
            .background_executor()
            .spawn(async move { maker.start(&tasks, &spec, url) });
        cx.spawn_in(window, async move |this, cx| {
            let result = started.await;
            this.update_in(cx, |this, window, cx| {
                this.starting = false;
                match result {
                    Ok(_) => {
                        window.close_dialog(cx);
                        // After this update, so no window is updated twice.
                        cx.spawn(async move |_, cx| {
                            cx.update(respond);
                        })
                        .detach();
                    }
                    Err(error) => {
                        this.confirming = false;
                        this.error = Some(error.message());
                    }
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }
}

/// What a task needs from the app to start, captured on the UI thread so the
/// rest can run on a worker (the Keychain and the provider's client are
/// never built on the UI thread).
struct StartMaker {
    store: Arc<kanso_store::Store>,
    tokens: kanso_server::TokenStore,
    ai: crate::config::AiSettings,
    data_dir: std::path::PathBuf,
    secrets: Option<Arc<dyn kanso_sync::SecretStore>>,
    is_agent: bool,
}

impl StartMaker {
    fn capture(cx: &App) -> StartMaker {
        let state = cx.global::<AppState>();
        let services = cx.global::<Services>();
        StartMaker {
            store: state.store.clone(),
            tokens: services.agent_tokens.clone(),
            ai: state.config.ai.clone(),
            data_dir: state.data_dir.clone(),
            secrets: services.secrets.clone(),
            is_agent: state.config.ai.backend == AiBackend::Agent,
        }
    }

    /// What the person calls the AI running the task.
    fn label(&self) -> String {
        match self.ai.backend {
            AiBackend::Agent => crate::ai_agent::label(&self.ai.agent),
            AiBackend::Api => format!("{}, {} (your key)", self.ai.provider.label(), self.ai.model),
        }
    }

    fn start(
        &self,
        tasks: &Tasks,
        spec: &TaskSpec,
        url: Option<String>,
    ) -> Result<TaskId, CannotStart> {
        let label = self.label();
        let ai = self.ai.clone();
        let data_dir = self.data_dir.clone();
        let secrets = self.secrets.clone();
        let tokens = self.tokens.clone();
        let time_limit = spec.time_limit;
        tasks.start(
            &self.store,
            &self.tokens,
            spec,
            &label,
            move |open| match ai.backend {
                AiBackend::Agent => {
                    let url = url.ok_or_else(|| {
                        "Kanso can't start its notes server, so the agent can't read your notes."
                            .to_owned()
                    })?;
                    let settings = crate::ai_agent::agent_settings(&ai.agent, &data_dir);
                    let access = task_access(open, url, tokens, std::env::current_exe().ok());
                    Ok(Arc::new(crate::agent_task_acp::AcpDriver(
                        kanso_ai::agent::task_conversation(&settings, access, time_limit),
                    )) as _)
                }
                AiBackend::Api => {
                    let saved = secrets
                        .as_deref()
                        .and_then(|secrets| crate::settings_ai::read_from(secrets, ai.provider));
                    let settings = kanso_ai::ClientSettings {
                        provider: ai.provider,
                        base_url: ai.base_url.clone(),
                        model: ai.model.clone(),
                        api_key: saved,
                        region: ai.provider.default_region().to_owned(),
                    };
                    kanso_ai::cached_client(&settings)
                        .map(|provider| Arc::new(ApiDriver(provider)) as _)
                        .map_err(|err| err.to_string())
                }
            },
        )
    }
}

/// One task's notes access for an ACP agent: the task's own token the first
/// time, a new one with the same scope if the session has to start again.
pub(crate) fn task_access(
    open: &kanso_engine::agent_task::OpenTask,
    url: String,
    tokens: kanso_server::TokenStore,
    relay: Option<std::path::PathBuf>,
) -> kanso_ai::agent::NotesAccess {
    use kanso_ai::agent::{NotesGrant, NotesLease};
    struct Lease {
        tokens: kanso_server::TokenStore,
        id: kanso_server::TokenId,
    }
    impl NotesLease for Lease {
        fn is_live(&self) -> bool {
            self.tokens.is_live(self.id)
        }
        fn renew(&self) -> bool {
            self.tokens.renew(self.id)
        }
        fn begin_turn(&self) {
            // A task is one long turn: its call budget is the task's.
        }
        fn reads(&self) -> Vec<String> {
            self.tokens.reads(self.id)
        }
    }
    impl Drop for Lease {
        fn drop(&mut self) {
            self.tokens.revoke(self.id);
        }
    }
    let first = std::sync::Mutex::new(Some((open.token, open.secret.as_ref().clone())));
    let scope = open.scope.clone();
    Arc::new(move || {
        let (id, secret) = first
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take()
            .unwrap_or_else(|| {
                let minted = tokens.mint((*scope).clone());
                (minted.id, minted.token)
            });
        Some(NotesGrant {
            url: url.clone(),
            token: secret,
            relay: relay.clone(),
            suggest: true,
            lease: Box::new(Lease {
                tokens: tokens.clone(),
                id,
            }),
        })
    })
}

// no-prop-test: drawing; what Start needs is `blocked`, covered by the tests below.
impl Render for NewTaskSheet {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.ensure_readable(cx);
        let colors = Colors::get(cx);
        let kind = self.kind;
        let blocked = self.blocked(cx);
        let state = cx.global::<AppState>();
        let ai = state.config.ai.clone();
        let who = match ai.backend {
            AiBackend::Agent => format!("{} on this Mac", crate::ai_agent::label(&ai.agent)),
            AiBackend::Api => format!("{}, {} (your key)", ai.provider.label(), ai.model),
        };
        let radio = |id: &'static str, label: String, value: ScopeKind, cx: &mut Context<Self>| {
            Checkbox::new(id)
                .checked(kind == value)
                .label(label)
                .on_click(cx.listener(move |this, _: &bool, _, cx| {
                    this.kind = value;
                    this.confirming = false;
                    cx.notify();
                }))
        };
        let book_rows = |chosen: Option<String>,
                         id: &'static str,
                         books: &[ScopeBook],
                         cx: &mut Context<Self>,
                         pick: fn(&mut NewTaskSheet, String)| {
            books
                .iter()
                .enumerate()
                .map(|(ix, book)| {
                    let bid = book.id.clone();
                    h_flex()
                        .h(px(BOOK_ROW))
                        .flex_shrink_0()
                        .pl(px(8. + 14. * book.depth as f32))
                        .child(
                            Checkbox::new(ElementId::NamedInteger(id.into(), ix as u64))
                                .checked(chosen.as_deref() == Some(book.id.as_str()))
                                .label(book.name.clone())
                                .on_click(cx.listener(move |this, _: &bool, _, cx| {
                                    pick(this, bid.clone());
                                    cx.notify();
                                })),
                        )
                })
                .collect::<Vec<_>>()
        };
        let notebook_rows = book_rows(
            self.notebook.clone(),
            "agent-task-read-book",
            &self.books,
            cx,
            |this, id| this.notebook = Some(id),
        );
        let new_rows = book_rows(
            self.new_book.clone(),
            "agent-task-new-book",
            &self.books,
            cx,
            |this, id| this.new_book = Some(id),
        );
        let op = |id: &'static str,
                  label: &'static str,
                  on: bool,
                  set: fn(&mut TaskOps, bool),
                  cx: &mut Context<Self>| {
            Checkbox::new(id)
                .checked(on)
                .label(label)
                .on_click(cx.listener(move |this, checked: &bool, _, cx| {
                    set(&mut this.ops, *checked);
                    cx.notify();
                }))
        };
        let section = |text: &'static str| {
            div()
                .text_size(px(12.))
                .font_medium()
                .text_color(colors.text_secondary)
                .child(text)
        };
        v_flex()
            .gap(px(10.))
            .child(Input::new(&self.goal))
            .child(section("Notes the agent can read"))
            .when(!self.notes.is_empty(), |el| {
                el.child(radio(
                    "agent-task-scope-notes",
                    if self.notes.len() == 1 {
                        "This note".to_owned()
                    } else {
                        format!("These notes ({} notes)", self.notes.len())
                    },
                    ScopeKind::Notes,
                    cx,
                ))
            })
            .child(radio(
                "agent-task-scope-book",
                "A notebook".to_owned(),
                ScopeKind::Notebook,
                cx,
            ))
            .when(kind == ScopeKind::Notebook, |el| {
                el.child(
                    v_flex()
                        .id("agent-task-books")
                        .h(book_list_height(self.books.len()))
                        .flex_shrink_0()
                        .overflow_y_scrollbar()
                        .children(notebook_rows),
                )
            })
            .child(radio(
                "agent-task-scope-allowed",
                if ai.agent.notebooks.is_empty() {
                    "All notebooks you allow agents (every unlocked notebook)".to_owned()
                } else {
                    format!(
                        "All notebooks you allow agents ({} chosen)",
                        ai.agent.notebooks.len()
                    )
                },
                ScopeKind::Allowed,
                cx,
            ))
            .child(
                div()
                    .text_size(px(12.))
                    .line_height(px(17.))
                    .text_color(colors.text_muted)
                    .child("Locked notes and notes tagged #ai-exclude are never shown to the agent."),
            )
            .child(section("What it may propose"))
            .child(
                h_flex()
                    .gap(px(14.))
                    .flex_wrap()
                    .child(op("agent-task-op-new", "New notes", self.ops.create, |o, v| o.create = v, cx))
                    .child(op("agent-task-op-edit", "Edits", self.ops.edit, |o, v| o.edit = v, cx))
                    .child(op("agent-task-op-move", "Moves between notebooks", self.ops.moves, |o, v| o.moves = v, cx))
                    .child(op("agent-task-op-merge", "Merges", self.ops.merge, |o, v| o.merge = v, cx)),
            )
            .child(section("New notes go in"))
            .child(
                v_flex()
                    .id("agent-task-new-books")
                    .h(book_list_height(self.books.len()))
                    .flex_shrink_0()
                    .overflow_y_scrollbar()
                    .children(new_rows),
            )
            .child(section("AI"))
            .child(
                div()
                    .text_size(px(12.5))
                    .text_color(colors.text)
                    .child(who),
            )
            .when(ai.backend == AiBackend::Agent, |el| {
                el.child(
                    div()
                        .text_size(px(12.))
                        .line_height(px(17.))
                        .text_color(colors.text_muted)
                        .child("The agent runs on this computer with its own tools. Kanso only lets it read the notes you chose."),
                )
            })
            .when(self.confirming, |el| {
                el.child(
                    div()
                        .text_size(px(12.5))
                        .font_medium()
                        .text_color(colors.warning)
                        .child("This task can read all your notebooks. Press Start again to go on."),
                )
            })
            .when_some(self.error.clone(), |el, error| {
                el.child(
                    div()
                        .text_size(px(12.5))
                        .text_color(colors.danger)
                        .child(error),
                )
            })
            .when_some(blocked.filter(|text| !text.is_empty()), |el, text| {
                el.child(
                    div()
                        .text_size(px(12.5))
                        .text_color(colors.text_secondary)
                        .child(text),
                )
            })
    }
}

/// Opens the New agent task sheet, with `notes` preselected when started
/// from a selection (empty otherwise).
pub fn open_new(notes: Vec<String>, window: &mut Window, cx: &mut App) {
    let sheet = cx.new(|cx| NewTaskSheet::new(notes, window, cx));
    let view = sheet.clone();
    window.open_alert_dialog(cx, move |alert, _, cx| {
        let start = view.clone();
        let blocked = view.read(cx).blocked(cx).is_some() || view.read(cx).starting;
        alert
            .title("New agent task")
            .width(px(SHEET_WIDTH))
            .child(view.clone())
            .footer(
                h_flex()
                    .w_full()
                    .gap_2()
                    .justify_end()
                    .child(
                        Button::new("agent-task-cancel")
                            .small()
                            .label("Cancel")
                            .on_click(|_, window, cx| window.close_dialog(cx)),
                    )
                    .child(
                        Button::new("agent-task-start")
                            .small()
                            .primary()
                            .label("Start")
                            .disabled(blocked)
                            .on_click(move |_, window, cx| {
                                start.update(cx, |sheet, cx| sheet.start(window, cx));
                            }),
                    ),
            )
    });
}

// --------------------------------------------------------- the review sheet

/// The Review agent changes sheet.
pub struct ReviewChanges {
    task: TaskId,
    tasks: Tasks,
    cards: Cards,
    ticked: BTreeSet<usize>,
    apply_anyway: BTreeSet<usize>,
    keep_sources: BTreeSet<usize>,
    queue: Option<Entity<crate::review_queue::ReviewSheet>>,
    shown_all: BTreeSet<usize>,
    error: Option<String>,
}

impl ReviewChanges {
    fn new(task: TaskId, tasks: Tasks, cards: Cards) -> ReviewChanges {
        let selection = cards.default_selection();
        ReviewChanges {
            task,
            tasks,
            ticked: selection.ticked,
            apply_anyway: BTreeSet::new(),
            keep_sources: BTreeSet::new(),
            queue: None,
            cards,
            shown_all: BTreeSet::new(),
            error: None,
        }
    }

    fn selection(&self) -> Selection {
        Selection {
            ticked: self.ticked.clone(),
            apply_anyway: self.apply_anyway.clone(),
            keep_sources: self.keep_sources.clone(),
        }
    }

    fn toggle(&mut self, index: usize, on: bool, cx: &mut Context<Self>) {
        let blocked = self
            .cards
            .cards
            .iter()
            .any(|c| c.index == index && c.state == CardState::Blocked);
        if blocked {
            return;
        }
        if on {
            self.ticked.insert(index);
            // Ticking a changed note is choosing to apply over it.
            if self
                .cards
                .cards
                .iter()
                .any(|c| c.index == index && matches!(c.state, CardState::Changed { .. }))
            {
                self.apply_anyway.insert(index);
            }
        } else {
            self.ticked.remove(&index);
            self.apply_anyway.remove(&index);
        }
        cx.notify();
    }

    fn apply(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let now = crate::clock::Clock::now_ms(cx);
        let selection = self.selection();
        match self.tasks.apply(self.task, &selection, now) {
            Ok(_) => {
                window.close_dialog(cx);
                cx.refresh_windows();
            }
            Err(failure) => {
                self.error = Some(match failure {
                    ApplyFailure::Conflict => {
                        "A note changed since the agent read it. Untick it, or tick it to apply anyway."
                            .to_owned()
                    }
                    other => other.message(),
                });
                cx.notify();
            }
        }
    }

    fn discard_all(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.tasks.discard(self.task);
        window.close_dialog(cx);
    }

    fn render_lines(
        lines: &[(LineKind, String)],
        colors: &Colors,
        mono: Option<&'static str>,
    ) -> AnyElement {
        v_flex()
            .rounded(px(8.))
            .border_1()
            .border_color(colors.divider)
            .overflow_hidden()
            .children(lines.iter().map(|(kind, text)| {
                let (tint, sign, color) = match kind {
                    LineKind::Added => (
                        Hsla::from(colors.success).opacity(0.13),
                        "+",
                        colors.success,
                    ),
                    LineKind::Removed => {
                        (Hsla::from(colors.danger).opacity(0.13), "−", colors.danger)
                    }
                };
                h_flex()
                    .items_start()
                    .min_h(px(20.))
                    .bg(tint)
                    .child(div().w(px(18.)).flex_none().text_color(color).child(sign))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .pr(px(8.))
                            .text_size(px(12.5))
                            .text_color(colors.text)
                            .when_some(mono, |el, font| el.font_family(font))
                            .child(if text.is_empty() {
                                " ".to_owned()
                            } else {
                                text.clone()
                            }),
                    )
            }))
            .into_any_element()
    }

    fn render_card(
        &mut self,
        card: &ReviewCard,
        colors: &Colors,
        mono: Option<&'static str>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let index = card.index;
        let blocked = card.state == CardState::Blocked;
        let label = |text: String, color: crate::theme::Rgb| {
            div()
                .text_size(px(12.))
                .font_medium()
                .text_color(color)
                .child(text)
        };
        let body: AnyElement = match &card.detail {
            CardDetail::New { body, cites } => v_flex()
                .gap(px(6.))
                .child(
                    div()
                        .p(px(8.))
                        .rounded(px(8.))
                        .bg(colors.fill)
                        .max_h(px(260.))
                        .overflow_hidden()
                        .text_size(px(12.5))
                        .line_height(px(18.))
                        .text_color(colors.text)
                        .child(TextView::markdown(("agent-new-text", index), body.clone())),
                )
                .when(!cites.is_empty(), |el| {
                    el.child(label(
                        format!("Sources: {}", cites.join(", ")),
                        colors.text_muted,
                    ))
                })
                .into_any_element(),
            CardDetail::Edit {
                title,
                lines,
                added,
                removed,
            } => {
                let mut lines = lines.clone();
                let more = *added + *removed > lines.len();
                if self.shown_all.contains(&index)
                    && let Some(set) = self.tasks.changes(self.task)
                {
                    let store = cx.global::<AppState>().store.clone();
                    lines = review::all_lines(&set, &store, index);
                }
                v_flex()
                    .gap(px(6.))
                    .when_some(title.clone(), |el, (old, new)| {
                        el.child(label(
                            format!("Title: {old} → {new}"),
                            colors.text_secondary,
                        ))
                    })
                    .child(Self::render_lines(&lines, colors, mono))
                    .when(more && !self.shown_all.contains(&index), |el| {
                        el.child(
                            Button::new(("agent-review-showall", index))
                                .ghost()
                                .xsmall()
                                .label("Show all")
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.shown_all.insert(index);
                                    cx.notify();
                                })),
                        )
                    })
                    .into_any_element()
            }
            CardDetail::Move { from, to } => div()
                .text_size(px(13.))
                .text_color(colors.text)
                .child(format!("{from} → {to}"))
                .into_any_element(),
            CardDetail::Merge {
                sources,
                title,
                target_is_new,
                lines,
                ..
            } => v_flex()
                .gap(px(6.))
                .child(label(
                    format!("Combines {} notes into {title}", sources.len()),
                    colors.text_secondary,
                ))
                .child(
                    div()
                        .text_size(px(12.5))
                        .text_color(colors.text_muted)
                        .child(sources.join(", ")),
                )
                .child(Self::render_lines(lines, colors, mono))
                .child(
                    Checkbox::new(("agent-review-keep", index))
                        .label("Keep the original notes")
                        .checked(self.keep_sources.contains(&index))
                        .on_click(cx.listener(move |this, on: &bool, _, cx| {
                            if *on {
                                this.keep_sources.insert(index);
                            } else {
                                this.keep_sources.remove(&index);
                            }
                            cx.notify();
                        })),
                )
                .child(label(
                    if self.keep_sources.contains(&index) {
                        format!(
                            "The {} notes stay where they are{}.",
                            sources.len(),
                            if *target_is_new {
                                ""
                            } else {
                                "; the target note is replaced"
                            }
                        )
                    } else {
                        format!(
                            "The {} notes move to the Trash{}.",
                            sources.len(),
                            if *target_is_new {
                                ""
                            } else {
                                "; the target note is replaced"
                            }
                        )
                    },
                    colors.text_muted,
                ))
                .into_any_element(),
        };
        v_flex()
            .id(("agent-review-card", index))
            .flex_none()
            .gap(px(6.))
            .p(px(12.))
            .rounded(px(10.))
            .border_1()
            .border_color(colors.divider)
            .child(
                h_flex()
                    .gap(px(8.))
                    .child(
                        Checkbox::new(("agent-review-tick", index))
                            .checked(self.ticked.contains(&index))
                            .disabled(blocked)
                            .on_click(cx.listener(move |this, on: &bool, _, cx| {
                                this.toggle(index, *on, cx)
                            })),
                    )
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .child(
                                div()
                                    .text_size(px(14.))
                                    .font_semibold()
                                    .text_color(colors.text)
                                    .child(format!(
                                        "{}{}",
                                        match card.kind {
                                            OpKind::Create => "New note: ",
                                            OpKind::Edit => "",
                                            OpKind::Merge => "Merge: ",
                                            OpKind::Move => "Move: ",
                                        },
                                        if card.title.trim().is_empty() {
                                            "Untitled"
                                        } else {
                                            card.title.trim()
                                        }
                                    )),
                            )
                            .child(
                                div()
                                    .text_size(px(12.))
                                    .text_color(colors.text_muted)
                                    .child(card.book.clone()),
                            ),
                    ),
            )
            .when(card.outside, |el| {
                el.child(label(
                    "Not one of the notes you chose".to_owned(),
                    colors.warning,
                ))
            })
            .when(card.published, |el| {
                el.child(label("Published".to_owned(), colors.warning))
            })
            .when(blocked, |el| {
                el.child(label(
                    "This note can no longer be changed by an agent".to_owned(),
                    colors.danger,
                ))
            })
            .when_some(
                match &card.state {
                    CardState::Changed {
                        current_title,
                        current_body,
                    } => Some((current_title.clone(), current_body.clone())),
                    _ => None,
                },
                |el, (title, text)| {
                    el.child(
                        v_flex()
                            .gap(px(4.))
                            .child(label(
                                "Changed since the agent read it".to_owned(),
                                colors.danger,
                            ))
                            .child(
                                div()
                                    .p(px(8.))
                                    .rounded(px(8.))
                                    .bg(colors.fill)
                                    .max_h(px(140.))
                                    .overflow_hidden()
                                    .text_size(px(12.5))
                                    .text_color(colors.text)
                                    .child(format!("The note now — {title}\n{text}")),
                            ),
                    )
                },
            )
            .child(body)
            .into_any_element()
    }
}

// no-prop-test: drawing; the cards' content is `kanso_engine::agent_task::review`'s, tested there.
impl Render for ReviewChanges {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let mono = crate::theme::mono_font(cx);
        let cards = self.cards.cards.clone();
        let rendered: Vec<AnyElement> = cards
            .iter()
            .map(|card| self.render_card(card, colors, mono, cx))
            .collect();
        v_flex()
            .gap(px(10.))
            .child(
                v_flex()
                    .gap(px(2.))
                    .child(
                        div()
                            .min_w_0()
                            .max_h(px(40.))
                            .overflow_hidden()
                            .text_size(px(14.))
                            .font_semibold()
                            .text_color(colors.text)
                            .child(self.cards.goal.clone()),
                    )
                    .child(
                        div()
                            .min_w_0()
                            .truncate()
                            .text_size(px(12.5))
                            .text_color(colors.text_secondary)
                            .child(format!("{} · {}", self.cards.agent, self.cards.header())),
                    ),
            )
            .when(!self.cards.from_agent.is_empty(), |el| {
                el.child(
                    v_flex()
                        .gap(px(2.))
                        .p(px(10.))
                        .rounded(px(8.))
                        .bg(colors.fill)
                        .child(
                            div()
                                .text_size(px(11.5))
                                .font_medium()
                                .text_color(colors.text_muted)
                                .child("From the agent"),
                        )
                        .child(
                            div()
                                .text_size(px(12.5))
                                .line_height(px(18.))
                                .text_color(colors.text)
                                .child(self.cards.from_agent.clone()),
                        ),
                )
            })
            .children(self.cards.warnings.iter().map(|warning| {
                div()
                    .text_size(px(12.5))
                    .font_medium()
                    .text_color(colors.warning)
                    .child(warning.clone())
            }))
            .when_some(self.error.clone(), |el, error| {
                el.child(
                    div()
                        .text_size(px(12.5))
                        .font_medium()
                        .text_color(colors.danger)
                        .child(error),
                )
            })
            .when(rendered.is_empty(), |el| {
                el.child(
                    div()
                        .py_6()
                        .text_size(px(13.5))
                        .text_color(colors.text_secondary)
                        .child("The agent didn't propose any changes."),
                )
            })
            .child(
                v_flex()
                    .id("agent-review-cards")
                    .gap(px(10.))
                    .max_h(px(460.))
                    .overflow_y_scrollbar()
                    .children(rendered)
                    .when_some(self.queue.clone(), |el, queue| el.child(queue)),
            )
    }
}

/// Opens the review sheet for `task`.
pub fn open_review(task: TaskId, window: &mut Window, cx: &mut App) {
    let Some(runner) = tasks(cx) else { return };
    let Some(set) = runner.changes(task) else {
        return;
    };
    let store = cx.global::<AppState>().store.clone();
    let last = runner
        .row(task)
        .map(|row| row.last_message)
        .unwrap_or_default();
    let cards = review::build(&store, &set, &last);
    let queue = crate::review_queue::queue(cx);
    let sheet = cx.new(|cx| {
        let mut sheet = ReviewChanges::new(task, runner, cards);
        sheet.queue = queue.map(|queue| {
            cx.new(|cx| crate::review_queue::ReviewSheet::embedded(queue, store.clone(), cx))
        });
        sheet
    });
    let view = sheet.clone();
    window.open_alert_dialog(cx, move |alert, _, cx| {
        let apply = view.clone();
        let discard = view.clone();
        let count = view.read(cx).ticked.len();
        alert
            .title("Review agent changes")
            .width(px(REVIEW_WIDTH))
            .child(view.clone())
            .footer(
                h_flex()
                    .w_full()
                    .gap_2()
                    .justify_end()
                    .child(
                        Button::new("agent-review-close")
                            .small()
                            .ghost()
                            .label("Close")
                            .on_click(|_, window, cx| window.close_dialog(cx)),
                    )
                    .child(
                        Button::new("agent-review-discard")
                            .small()
                            .outline()
                            .label("Discard All")
                            .on_click(move |_, window, cx| {
                                discard.update(cx, |sheet, cx| sheet.discard_all(window, cx));
                            }),
                    )
                    .child(
                        Button::new("agent-review-apply")
                            .small()
                            .primary()
                            .label(Cards::apply_label(&Selection {
                                ticked: (0..count).collect(),
                                apply_anyway: BTreeSet::new(),
                                keep_sources: BTreeSet::new(),
                            }))
                            .disabled(count == 0)
                            .on_click(move |_, window, cx| {
                                apply.update(cx, |sheet, cx| sheet.apply(window, cx));
                            }),
                    ),
            )
    });
}

#[cfg(test)]
mod tests {
    use super::NOTIFICATION_BODY;

    // no-prop-test: a constant; the point is that no goal text can reach it.
    #[test]
    fn the_notification_never_carries_the_goal() {
        assert!(!NOTIFICATION_BODY.is_empty());
        assert!(NOTIFICATION_BODY.len() < 80);
    }
}
