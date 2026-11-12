//! One agent task as a conversation (issue #375): the header, and the
//! transcript with what the agent said, thought, ran, was allowed and
//! proposed. It reads the runner's transcript and draws it; the folding of
//! tool rows and the words are `kanso_engine::agent_task::blocks`.
//!
//! No composer yet: a question from the agent still gets its one-off reply
//! box here. A session read back after a restart has a box of its own, which
//! picks the agent's conversation up again, or "Continue in a new session"
//! when it can't, and links to the session it carries on from or in.

use std::collections::{HashMap, HashSet};
use std::time::Duration;

use gpui_component::scroll::ScrollableElement as _;
use gpui_kit::assets::IconName as Lucide;
use gpui_kit::base::text::TextView;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{Input, InputState};
use gpui_kit::component::{Disableable as _, Icon, Sizable as _, StyledExt as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_engine::agent_task::blocks::{self, Block, Offer, ToolLine};
use kanso_engine::agent_task::model::{TaskRow, elapsed_label, reads_label, scope_line};
use kanso_engine::agent_task::runner::{TaskEvent, Tasks};
use kanso_engine::agent_task::session::{Item, RoundState, SessionKey, ToolKind, ToolStatus};
use kanso_server::TaskId;

use crate::chat_parts::watch;
use crate::theme::Colors;
use crate::workspace::AppState;
use crate::zoom::px;

/// What the view tells whoever holds it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SessionEvent {
    /// "‹ Tasks": back to the list.
    Back,
    /// Show another session in its place (a linked one).
    Open(SessionKey),
}

impl EventEmitter<SessionEvent> for AgentSessionView {}

/// The view of one session.
pub struct AgentSessionView {
    tasks: Tasks,
    key: SessionKey,
    /// Whether the view is in the panel, with a way back to the list (and a
    /// way out to a window of its own).
    in_panel: bool,
    row: Option<TaskRow>,
    blocks: Vec<Block>,
    /// The transcript version `blocks` were made from.
    version: u64,
    /// The scope in a few words, and in full when the chip is open.
    scope_short: String,
    scope_full: String,
    scope_open: bool,
    /// Thoughts and tool runs the person opened, by block position.
    opened: HashSet<usize>,
    reads_open: bool,
    scroll: ScrollHandle,
    /// Whether the view follows the end of the transcript: until the person
    /// scrolls up.
    stick: bool,
    reply: Option<Entity<InputState>>,
    /// The box that picks a restored session up again.
    resume: Option<Entity<InputState>>,
    /// Why resuming or continuing didn't start.
    problem: Option<String>,
    /// A resume or a continue is starting.
    busy: bool,
    _watch: Task<()>,
}

impl AgentSessionView {
    pub fn new(key: SessionKey, in_panel: bool, cx: &mut Context<Self>) -> AgentSessionView {
        let tasks = crate::agent_task_panel::tasks(cx).unwrap_or_default();
        let (scope_short, scope_full) = tasks
            .by_key(&key)
            .and_then(|id| tasks.spec(id))
            .map(|spec| {
                let names: HashMap<String, String> = cx
                    .global::<AppState>()
                    .store
                    .books()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|book| (book.id, book.name))
                    .collect();
                let name = move |id: &str| {
                    names
                        .get(id)
                        .cloned()
                        .unwrap_or_else(|| "a notebook".to_owned())
                };
                (
                    format!("Reads: {}", reads_label(&spec.read, &name)),
                    scope_line(&spec, &name),
                )
            })
            .unwrap_or_default();
        let events = tasks.subscribe();
        let watching = watch(
            events,
            cx,
            |this: &mut AgentSessionView, event, cx| {
                let mine = match event {
                    TaskEvent::Changed(Some(id)) | TaskEvent::Finished(id) => this.id() == Some(id),
                    TaskEvent::Changed(None) => true,
                };
                if mine {
                    this.refresh(cx);
                }
                false
            },
            |_, _| {},
        );
        let mut view = AgentSessionView {
            tasks,
            key,
            in_panel,
            row: None,
            blocks: Vec::new(),
            version: u64::MAX,
            scope_short,
            scope_full,
            scope_open: false,
            opened: HashSet::new(),
            reads_open: false,
            scroll: ScrollHandle::new(),
            stick: true,
            reply: None,
            resume: None,
            problem: None,
            busy: false,
            _watch: watching,
        };
        view.refresh(cx);
        view
    }

    fn id(&self) -> Option<TaskId> {
        self.tasks.by_key(&self.key)
    }

    /// Looks at the session again: the row, and the transcript when it grew.
    fn refresh(&mut self, cx: &mut Context<Self>) {
        let Some(id) = self.id() else {
            self.row = None;
            cx.notify();
            return;
        };
        self.row = self.tasks.row(id);
        if let Some(transcript) = self.tasks.transcript(id)
            && transcript.version != self.version
        {
            // Follow the end, unless the person scrolled up from it.
            self.stick = self.stick_now();
            let undoable = if self.tasks.can_undo(id) {
                transcript.items.iter().rev().find_map(|item| match item {
                    Item::Proposed {
                        round,
                        state: RoundState::Applied,
                        ..
                    } => Some(*round),
                    _ => None,
                })
            } else {
                None
            };
            self.blocks = blocks::fold(&transcript.items, undoable);
            self.version = transcript.version;
        }
        // Looking at it is reading it.
        self.tasks.mark_seen(id);
        cx.notify();
    }

    /// Whether the transcript is scrolled to its end (or has nothing to
    /// scroll).
    fn stick_now(&self) -> bool {
        let max = self.scroll.max_offset().y;
        let at = -self.scroll.offset().y;
        max <= px(1.) || at >= max - px(24.)
    }

    fn reply_box(&mut self, window: &mut Window, cx: &mut Context<Self>) -> Entity<InputState> {
        self.reply
            .get_or_insert_with(|| {
                cx.new(|cx| InputState::new(window, cx).placeholder("Reply to the agent"))
            })
            .clone()
    }

    fn send_reply(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let (Some(id), Some(input)) = (self.id(), self.reply.clone()) else {
            return;
        };
        let text = input.read(cx).value().to_string();
        if text.trim().is_empty() {
            return;
        }
        self.tasks.reply(id, &text);
        input.update(cx, |input, cx| input.set_value("", window, cx));
        self.stick = true;
        cx.notify();
    }

    fn resume_box(&mut self, window: &mut Window, cx: &mut Context<Self>) -> Entity<InputState> {
        self.resume
            .get_or_insert_with(|| {
                cx.new(|cx| InputState::new(window, cx).placeholder("Ask for more…"))
            })
            .clone()
    }

    /// Sends the message in the resume box: the agent's conversation is
    /// picked up again where it left off.
    fn send_resume(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let (Some(id), Some(input)) = (self.id(), self.resume.clone()) else {
            return;
        };
        let text = input.read(cx).value().to_string();
        if text.trim().is_empty() || self.busy {
            return;
        }
        self.problem = None;
        self.busy = true;
        let job = crate::agent_task_panel::resume_job(id, text, cx);
        input.update(cx, |input, cx| input.set_value("", window, cx));
        self.stick = true;
        cx.spawn(async move |this, cx| {
            let result = job.await;
            this.update(cx, |this, cx| {
                this.busy = false;
                if let Err(message) = result {
                    this.problem = Some(message);
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
        cx.notify();
    }

    /// "Continue in a new session": the same scope, with a summary of this
    /// one, linked both ways; the new session opens in place of this one.
    fn continue_in_new(&mut self, cx: &mut Context<Self>) {
        let Some(id) = self.id() else { return };
        if self.busy {
            return;
        }
        self.problem = None;
        self.busy = true;
        let job = crate::agent_task_panel::continue_job(id, cx);
        cx.spawn(async move |this, cx| {
            let result = job.await;
            this.update(cx, |this, cx| {
                this.busy = false;
                match result {
                    Ok(key) => this.go_to(key, cx),
                    Err(message) => this.problem = Some(message),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
        cx.notify();
    }

    /// Shows session `key`: in the panel in place of this one, from a window
    /// in a window of its own.
    fn go_to(&mut self, key: SessionKey, cx: &mut Context<Self>) {
        if self.in_panel {
            cx.emit(SessionEvent::Open(key));
        } else {
            open_in_window(key, cx);
        }
    }

    fn toggle(&mut self, ix: usize, cx: &mut Context<Self>) {
        if !self.opened.remove(&ix) {
            self.opened.insert(ix);
        }
        cx.notify();
    }

    fn undo(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = self.id() {
            let now = crate::clock::Clock::now_ms(cx);
            self.tasks.undo(id, now);
        }
        cx.notify();
    }

    // ------------------------------------------------------------ drawing

    /// "Continued from …" and "Continued in …", as buttons that open the
    /// other session (plain words once it is no longer listed).
    fn links(&mut self, cx: &mut Context<Self>) -> Option<Vec<AnyElement>> {
        let colors = Colors::get(cx);
        let row = self.row.clone()?;
        let mut links = Vec::new();
        for (ix, (word, key)) in [
            ("Continued from", row.continued_from),
            ("Continued in", row.continued_in),
        ]
        .into_iter()
        .enumerate()
        {
            let Some(key) = key else { continue };
            let title = self
                .tasks
                .by_key(&key)
                .and_then(|id| self.tasks.row(id))
                .map(|row| row.title);
            links.push(match title {
                Some(title) => Button::new(("session-link", ix))
                    .ghost()
                    .xsmall()
                    .label(format!("{word} {title} ›"))
                    .on_click(cx.listener(move |this, _, _, cx| this.go_to(key.clone(), cx)))
                    .into_any_element(),
                None => div()
                    .text_size(px(12.))
                    .text_color(colors.text_muted)
                    .child(format!("{word} a session that was deleted"))
                    .into_any_element(),
            });
        }
        (!links.is_empty()).then_some(links)
    }

    fn header(&mut self, ix_base: &'static str, cx: &mut Context<Self>) -> AnyElement {
        let colors = Colors::get(cx);
        let row = self.row.clone();
        let id = self.id();
        let title = row
            .as_ref()
            .map_or_else(|| "Session".to_owned(), |row| row.title.clone());
        let agent = row
            .as_ref()
            .map(|row| row.agent.clone())
            .unwrap_or_default();
        let counts_line = row.as_ref().map(|row| row.counts_line());
        let reads_n = row.as_ref().map_or(0, |row| row.notes_read);
        let elapsed = row
            .as_ref()
            .filter(|row| row.is_active())
            .map(|row| elapsed_label(row.elapsed));
        let (can_stop, can_review, can_discard) = row.as_ref().map_or((false, false, false), |r| {
            (r.can_stop(), r.can_review(), r.can_discard())
        });
        let can_undo = id.is_some_and(|id| self.tasks.can_undo(id));
        let links = self.links(cx);
        let tasks = self.tasks.clone();
        let open_window = {
            let key = self.key.clone();
            move |_: &ClickEvent, _: &mut Window, cx: &mut App| open_in_window(key.clone(), cx)
        };
        v_flex()
            .id(ix_base)
            .debug_selector(|| "session-header".into())
            .flex_none()
            .gap(px(6.))
            .p(px(12.))
            .border_b_1()
            .border_color(colors.divider)
            .when(self.in_panel, |el| {
                el.child(
                    h_flex().child(
                        Button::new("session-back")
                            .ghost()
                            .xsmall()
                            .icon(Lucide::ChevronLeft)
                            .label("Tasks")
                            .on_click(cx.listener(|_, _, _, cx| cx.emit(SessionEvent::Back))),
                    ),
                )
            })
            .child(
                h_flex()
                    .gap(px(8.))
                    .items_start()
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .child(
                                div()
                                    .min_w_0()
                                    .text_size(px(14.))
                                    .line_height(px(19.))
                                    .font_semibold()
                                    .text_color(colors.text)
                                    .child(title),
                            )
                            .child(
                                div()
                                    .min_w_0()
                                    .truncate()
                                    .text_size(px(12.))
                                    .text_color(colors.text_muted)
                                    .child(match elapsed {
                                        Some(time) => format!("{agent} · {time}"),
                                        None => agent,
                                    }),
                            ),
                    )
                    .when(self.in_panel, |el| {
                        el.child(
                            Button::new("session-window")
                                .ghost()
                                .xsmall()
                                .icon(Lucide::ExternalLink)
                                .tooltip("Open in Window")
                                .on_click(open_window),
                        )
                    }),
            )
            .when_some(links, |el, links| {
                el.child(h_flex().gap(px(6.)).flex_wrap().children(links))
            })
            .child(
                h_flex()
                    .gap(px(6.))
                    .flex_wrap()
                    .child(
                        Button::new("session-scope")
                            .outline()
                            .xsmall()
                            .label(self.scope_short.clone())
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.scope_open = !this.scope_open;
                                cx.notify();
                            })),
                    )
                    .when(reads_n > 0, |el| {
                        let open = self.reads_open;
                        el.child(
                            Button::new("session-reads")
                                .ghost()
                                .xsmall()
                                .label(format!(
                                    "Notes read: {reads_n}{}",
                                    if open { " ▾" } else { " ▸" }
                                ))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.reads_open = !this.reads_open;
                                    cx.notify();
                                })),
                        )
                    }),
            )
            .when(self.scope_open, |el| {
                el.child(
                    div()
                        .p(px(8.))
                        .rounded(px(8.))
                        .bg(colors.fill)
                        .text_size(px(12.))
                        .line_height(px(17.))
                        .text_color(colors.text_secondary)
                        .child(self.scope_full.clone()),
                )
            })
            .when(self.reads_open, |el| {
                let store = cx.global::<AppState>().store.clone();
                let titles: Vec<String> = id
                    .map(|id| self.tasks.reads(id))
                    .unwrap_or_default()
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
                    .collect();
                el.child(
                    v_flex()
                        .id("session-reads-list")
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
            .when_some(counts_line.filter(|_| can_stop), |el, line| {
                el.child(
                    div()
                        .text_size(px(12.))
                        .text_color(colors.text_muted)
                        .child(line),
                )
            })
            .child(
                h_flex()
                    .gap(px(8.))
                    .justify_end()
                    .when(can_stop, |el| {
                        let tasks = tasks.clone();
                        el.child(
                            Button::new("session-stop")
                                .outline()
                                .small()
                                .label("Stop")
                                .on_click(move |_, _, _| {
                                    if let Some(id) = id {
                                        tasks.stop(id);
                                    }
                                }),
                        )
                    })
                    .when(can_undo, |el| {
                        el.child(
                            Button::new("session-undo")
                                .outline()
                                .small()
                                .label("Undo")
                                .on_click(cx.listener(|this, _, _, cx| this.undo(cx))),
                        )
                    })
                    .when(can_discard, |el| {
                        let tasks = tasks.clone();
                        el.child(
                            Button::new("session-discard")
                                .outline()
                                .small()
                                .label("Discard")
                                .on_click(move |_, _, _| {
                                    if let Some(id) = id {
                                        tasks.discard(id);
                                    }
                                }),
                        )
                    })
                    .when(can_review, |el| {
                        el.child(
                            Button::new("session-review")
                                .primary()
                                .small()
                                .label("Review…")
                                .on_click(move |_, window, cx| {
                                    if let Some(id) = id {
                                        crate::agent_task_panel::open_review(id, window, cx);
                                    }
                                }),
                        )
                    }),
            )
            .into_any_element()
    }

    fn block(&mut self, ix: usize, block: Block, cx: &mut Context<Self>) -> AnyElement {
        let colors = Colors::get(cx);
        match block {
            Block::You { text } => v_flex()
                .id(("session-you", ix))
                .debug_selector(|| format!("session-you-{ix}"))
                .gap(px(2.))
                .child(label("You", colors))
                .child(
                    div()
                        .p(px(8.))
                        .rounded(px(8.))
                        .bg(colors.fill)
                        .text_size(px(13.))
                        .line_height(px(19.))
                        .text_color(colors.text)
                        .child(text),
                )
                .into_any_element(),
            Block::Agent { text } => v_flex()
                .id(("session-agent", ix))
                .debug_selector(|| format!("session-agent-{ix}"))
                .gap(px(2.))
                .child(label("Agent", colors))
                .child(TextView::markdown(("session-agent-text", ix), text).selectable(true))
                .into_any_element(),
            Block::Thought { label: said, text } => {
                let open = self.opened.contains(&ix);
                v_flex()
                    .id(("session-thought", ix))
                    .debug_selector(|| format!("session-thought-{ix}"))
                    .gap(px(4.))
                    .child(
                        Button::new(("session-thought-toggle", ix))
                            .ghost()
                            .xsmall()
                            .label(format!("{said}{}", if open { " ▾" } else { " ▸" }))
                            .on_click(cx.listener(move |this, _, _, cx| this.toggle(ix, cx))),
                    )
                    .when(open, |el| {
                        el.child(
                            div()
                                .pl(px(10.))
                                .border_l_2()
                                .border_color(colors.divider)
                                .text_size(px(12.5))
                                .line_height(px(18.))
                                .text_color(colors.text_muted)
                                .child(text),
                        )
                    })
                    .into_any_element()
            }
            Block::Tool(tool) => div()
                .id(("session-tool", ix))
                .debug_selector(|| format!("session-tool-{ix}"))
                .child(tool_row(("session-tool-row", ix), &tool, colors))
                .into_any_element(),
            Block::Tools { label, tools } => {
                let open = self.opened.contains(&ix);
                let working = tools.iter().any(is_going);
                let failed = tools.iter().any(|t| t.status == ToolStatus::Failed);
                v_flex()
                    .id(("session-tools", ix))
                    .debug_selector(|| format!("session-tools-{ix}"))
                    .gap(px(2.))
                    .child(
                        h_flex()
                            .gap(px(6.))
                            .child(
                                Button::new(("session-tools-toggle", ix))
                                    .ghost()
                                    .xsmall()
                                    .label(format!("{label}{}", if open { " ▾" } else { " ▸" }))
                                    .on_click(
                                        cx.listener(move |this, _, _, cx| this.toggle(ix, cx)),
                                    ),
                            )
                            .when(working, |el| {
                                el.child(spinner(("session-tools-spin", ix), colors))
                            })
                            .when(failed && !working, |el| {
                                el.child(
                                    Icon::new(Lucide::Close)
                                        .size(px(12.))
                                        .text_color(colors.danger),
                                )
                            }),
                    )
                    .when(open, |el| {
                        el.child(v_flex().pl(px(10.)).gap(px(2.)).children(
                            tools.iter().enumerate().map(|(n, tool)| {
                                tool_row(("session-tools-row", ix * 1000 + n), tool, colors)
                            }),
                        ))
                    })
                    .into_any_element()
            }
            Block::Permission { text } => div()
                .id(("session-permission", ix))
                .debug_selector(|| format!("session-permission-{ix}"))
                .text_size(px(12.))
                .line_height(px(17.))
                .text_color(colors.text_muted)
                .child(text)
                .into_any_element(),
            Block::Proposal {
                round,
                label,
                state,
                offer,
            } => {
                let id = self.id();
                let tone = match state {
                    RoundState::ToReview => colors.warning,
                    _ => colors.divider,
                };
                h_flex()
                    .id(("session-proposal", ix))
                    .debug_selector(|| format!("session-proposal-{ix}"))
                    .gap(px(8.))
                    .p(px(10.))
                    .rounded(px(10.))
                    .border_1()
                    .border_color(tone)
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .text_size(px(13.))
                            .text_color(colors.text)
                            .child(label),
                    )
                    .when(offer == Offer::Review, |el| {
                        el.child(
                            Button::new(("session-proposal-review", round as usize))
                                .primary()
                                .small()
                                .label("Review…")
                                .on_click(move |_, window, cx| {
                                    if let Some(id) = id {
                                        crate::agent_task_panel::open_review(id, window, cx);
                                    }
                                }),
                        )
                    })
                    .when(offer == Offer::Undo, |el| {
                        el.child(
                            Button::new(("session-proposal-undo", round as usize))
                                .outline()
                                .small()
                                .label("Undo")
                                .on_click(cx.listener(|this, _, _, cx| this.undo(cx))),
                        )
                    })
                    .into_any_element()
            }
            Block::Notice { text } => div()
                .id(("session-notice", ix))
                .debug_selector(|| format!("session-notice-{ix}"))
                .text_size(px(12.))
                .text_color(colors.text_muted)
                .child(text)
                .into_any_element(),
        }
    }
}

/// A small muted heading over a message.
fn label(text: &'static str, colors: &Colors) -> impl IntoElement {
    div()
        .text_size(px(11.5))
        .font_semibold()
        .text_color(colors.text_muted)
        .child(text)
}

fn is_going(tool: &ToolLine) -> bool {
    matches!(tool.status, ToolStatus::Pending | ToolStatus::InProgress)
}

/// The icon for what a tool call does.
fn kind_icon(kind: ToolKind) -> Lucide {
    match kind {
        ToolKind::Read => Lucide::Eye,
        ToolKind::Edit => Lucide::Pencil,
        ToolKind::Delete => Lucide::Trash,
        ToolKind::Move => Lucide::FolderOpen,
        ToolKind::Search => Lucide::Search,
        ToolKind::Execute => Lucide::Code,
        ToolKind::Think => Lucide::Sparkles,
        ToolKind::Fetch => Lucide::Globe,
        ToolKind::SwitchMode => Lucide::RefreshCw,
        ToolKind::Other => Lucide::Puzzle,
    }
}

fn spinner(id: impl Into<ElementId>, colors: &Colors) -> impl IntoElement {
    Icon::new(Lucide::RefreshCcw)
        .size(px(12.))
        .text_color(colors.text_muted)
        .with_animation(
            id,
            Animation::new(Duration::from_millis(1100)).repeat(),
            |icon, t| icon.transform(Transformation::rotate(percentage(t))),
        )
}

/// One tool call: its kind, its title and how it went.
fn tool_row(id: (&'static str, usize), tool: &ToolLine, colors: &Colors) -> impl IntoElement {
    h_flex()
        .id(id)
        .gap(px(6.))
        .items_center()
        .child(
            Icon::new(kind_icon(tool.kind))
                .size(px(12.))
                .text_color(colors.text_muted),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .truncate()
                .text_size(px(12.5))
                .text_color(colors.text_secondary)
                .child(tool.title.clone()),
        )
        .child(match tool.status {
            ToolStatus::Pending | ToolStatus::InProgress => {
                spinner((id.0, id.1 + 1_000_000), colors).into_any_element()
            }
            ToolStatus::Completed => Icon::new(Lucide::Check)
                .size(px(12.))
                .text_color(colors.success)
                .into_any_element(),
            ToolStatus::Failed => Icon::new(Lucide::Close)
                .size(px(12.))
                .text_color(colors.danger)
                .into_any_element(),
        })
}

// no-prop-test: drawing; the blocks and their words are `kanso_engine::agent_task::blocks`.
impl Render for AgentSessionView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let asking = matches!(
            self.row.as_ref().map(|row| &row.phase),
            Some(kanso_engine::agent_task::model::Phase::Asking { .. })
        );
        let reply = asking.then(|| self.reply_box(window, cx));
        let resumable = self
            .row
            .as_ref()
            .is_some_and(|row| row.restored && row.resumable && !row.is_active());
        let resume = resumable.then(|| self.resume_box(window, cx));
        let can_continue = self.row.as_ref().is_some_and(TaskRow::can_continue);
        let problem = self.problem.clone();
        let busy = self.busy;
        if self.stick {
            self.scroll.scroll_to_bottom();
        }
        let header = self.header("session-header", cx);
        let blocks = self.blocks.clone();
        let items: Vec<AnyElement> = blocks
            .into_iter()
            .enumerate()
            .map(|(ix, block)| self.block(ix, block, cx))
            .collect();
        let gone = self.row.is_none();
        v_flex()
            .id("agent-session")
            .size_full()
            .min_h_0()
            .child(header)
            .child(
                div()
                    .id("session-transcript")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scrollbar()
                    .track_scroll(&self.scroll)
                    .on_scroll_wheel(cx.listener(|_, _, _, cx| {
                        // After the wheel has moved the transcript.
                        cx.spawn(async move |this, cx| {
                            this.update(cx, |this, _| this.stick = this.stick_now())
                                .ok();
                        })
                        .detach();
                    }))
                    .child(
                        v_flex()
                            .gap(px(12.))
                            .p(px(12.))
                            .when(gone, |el| {
                                el.child(
                                    div()
                                        .text_size(px(12.5))
                                        .text_color(colors.text_muted)
                                        .child("This session is no longer listed."),
                                )
                            })
                            .children(items),
                    ),
            )
            .when_some(problem, |el, problem| {
                el.child(
                    div()
                        .flex_none()
                        .px(px(12.))
                        .py(px(6.))
                        .text_size(px(12.))
                        .text_color(colors.danger)
                        .child(problem),
                )
            })
            .when(can_continue, |el| {
                el.child(
                    h_flex()
                        .debug_selector(|| "session-continue-bar".into())
                        .flex_none()
                        .gap(px(8.))
                        .p(px(12.))
                        .border_t_1()
                        .border_color(colors.divider)
                        .child(
                            div()
                                .flex_1()
                                .text_size(px(12.))
                                .text_color(colors.text_muted)
                                .child("This session can't be picked up again."),
                        )
                        .child(
                            Button::new("session-continue")
                                .small()
                                .primary()
                                .disabled(busy)
                                .label("Continue in a new session")
                                .on_click(cx.listener(|this, _, _, cx| this.continue_in_new(cx))),
                        ),
                )
            })
            .when_some(resume, |el, input| {
                el.child(
                    h_flex()
                        .debug_selector(|| "session-resume-bar".into())
                        .flex_none()
                        .gap(px(8.))
                        .p(px(12.))
                        .border_t_1()
                        .border_color(colors.divider)
                        .child(div().flex_1().child(Input::new(&input)))
                        .child(
                            Button::new("session-resume")
                                .small()
                                .primary()
                                .disabled(busy)
                                .label("Send")
                                .on_click(
                                    cx.listener(|this, _, window, cx| this.send_resume(window, cx)),
                                ),
                        ),
                )
            })
            .when_some(reply, |el, input| {
                el.child(
                    h_flex()
                        .flex_none()
                        .gap(px(8.))
                        .p(px(12.))
                        .border_t_1()
                        .border_color(colors.divider)
                        .child(div().flex_1().child(Input::new(&input)))
                        .child(
                            Button::new("session-send")
                                .small()
                                .primary()
                                .label("Send")
                                .on_click(
                                    cx.listener(|this, _, window, cx| this.send_reply(window, cx)),
                                ),
                        ),
                )
            })
    }
}

/// The session in a window of its own.
pub fn open_in_window(key: SessionKey, cx: &mut App) {
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
            None,
            size(px(520.), px(680.)),
            cx,
        ))),
        titlebar: Some(TitlebarOptions {
            title: Some("Agent Session".into()),
            appears_transparent: false,
            traffic_light_position: None,
        }),
        #[cfg(target_os = "linux")]
        app_id: Some(crate::flavour::CURRENT.xdg_name.into()),
        ..Default::default()
    };
    let opened = gpui_kit::open_window(options, cx, |window, cx| {
        crate::themes::apply(Some(window), cx);
        cx.new(|cx| AgentSessionView::new(key, false, cx))
    });
    if let Err(err) = opened {
        eprintln!("kanso: couldn't open the session: {err:#}");
    }
}

#[cfg(test)]
impl AgentSessionView {
    /// A view of `items` with no runner behind it.
    pub(crate) fn from_items(
        items: &[Item],
        in_panel: bool,
        cx: &mut Context<Self>,
    ) -> AgentSessionView {
        let mut view = AgentSessionView::new(SessionKey::default(), in_panel, cx);
        view.row = Some(TaskRow::new(TaskId(1), "Merge my trips", "Claude Agent"));
        view.blocks = blocks::fold(items, None);
        view
    }

    /// Changes the row the view shows, to draw a session in another state.
    pub(crate) fn change_row(&mut self, change: impl FnOnce(&mut TaskRow)) {
        if let Some(row) = self.row.as_mut() {
            change(row);
        }
    }
}
