//! The Tasks view, in the note list's place: every task-list item across
//! the notes, grouped Overdue, Today, Upcoming and No date (and Completed,
//! when shown), each with its note and notebook. A task is ticked off in
//! place (its note's text changes in one write, with ⌘Z to take it back)
//! and opens its note at its line.
//!
//! The grouping, the selection and the check-offs are the [`Board`], which
//! knows nothing of drawing, so a model-based test can drive it against
//! the store.

use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenu, PopupMenuItem};
use gpui_kit::component::{
    Icon, Selectable as _, Sizable as _, StyledExt as _, VirtualListScrollHandle, h_flex, v_flex,
    v_virtual_list,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_core::dates::DateTime;
use kanso_core::tasks::{Due, TaskGroup};
use kanso_store::{Store, TaskQuery};

use crate::clock::Clock;
use crate::due::{Tone, chip_colors, chip_label, tone};
use crate::editor::NoteEditor;
use crate::icon_themes::{Slot, library_icon};
use crate::layout::{BOTTOM_BAND_HEIGHT, TOP_BAND_HEIGHT};
use crate::library::{Filter, Library, LibraryEvent};
use crate::note_list::quiet_empty;
use crate::theme::{Colors, section_label};
use crate::zoom::px;

/// A task row: padding, its words, a gap, its note and notebook, padding.
const ROW_HEIGHT: f32 = 54.;
const HEADER_HEIGHT: f32 = 32.;
const ROW_INSET_X: f32 = 8.;
const ROW_PAD_X: f32 = 10.;
/// How long changes to notes settle before the tasks are read again.
const RELOAD_DELAY: Duration = Duration::from_millis(120);
/// How often the groups are worked out again (a task falls overdue as
/// time passes) and the overdue count read again.
const TICK: Duration = Duration::from_secs(60);

actions!(
    tasks_view,
    [
        SelectNextTask,
        SelectPreviousTask,
        ToggleTask,
        OpenTask,
        UndoCheckOff
    ]
);

pub fn bind_keys(cx: &mut App) {
    let context = Some("TasksView");
    cx.bind_keys([
        KeyBinding::new("down", SelectNextTask, context),
        KeyBinding::new("j", SelectNextTask, context),
        KeyBinding::new("up", SelectPreviousTask, context),
        KeyBinding::new("k", SelectPreviousTask, context),
        KeyBinding::new("space", ToggleTask, context),
        KeyBinding::new("x", ToggleTask, context),
        KeyBinding::new("enter", OpenTask, context),
        KeyBinding::new("secondary-z", UndoCheckOff, context),
    ]);
}

pub use kanso_engine::tasks::board::{Board, Entry, TaskId, Toggled};

pub use kanso_engine::tasks::display_text;

/// The workspace opens notes.
pub enum TasksEvent {
    /// Open the task's note at its line; `focus` puts the keyboard in it.
    Open {
        note_id: String,
        line: u32,
        focus: bool,
    },
    /// Something to tell the user.
    Notice(String),
}

pub struct TasksView {
    library: Entity<Library>,
    editor: Entity<NoteEditor>,
    store: Arc<Store>,
    pub(crate) board: Board,
    focus: FocusHandle,
    scroll: VirtualListScrollHandle,
    leading_inset: Pixels,
    /// The rows were read before the last change to the notes.
    stale: bool,
    reload_task: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
    _tick: Task<()>,
}

impl EventEmitter<TasksEvent> for TasksView {}

impl Focusable for TasksView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl TasksView {
    pub fn new(
        library: Entity<Library>,
        editor: Entity<NoteEditor>,
        cx: &mut Context<Self>,
    ) -> Self {
        let store = library.read(cx).store.clone();
        let subscriptions = vec![cx.subscribe(&library, |this, library, event, cx| {
            match event {
                LibraryEvent::Changed(change) if change.id.starts_with("note:") => {
                    this.stale = true;
                    if library.read(cx).filter == Filter::Tasks {
                        this.reload_soon(cx);
                    }
                }
                LibraryEvent::Shown if library.read(cx).filter == Filter::Tasks => {
                    // Opened afresh: what was ticked off last time goes.
                    this.board.kept.clear();
                    this.reload(cx);
                }
                _ => {}
            }
        })];
        let tick = cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(TICK).await;
                let alive = this.update(cx, |this, cx| {
                    this.library
                        .update(cx, |library, cx| library.refresh_overdue(cx));
                    cx.notify();
                });
                if alive.is_err() {
                    break;
                }
            }
        });
        let mut view = TasksView {
            library,
            editor,
            store,
            board: Board::default(),
            focus: cx.focus_handle(),
            scroll: VirtualListScrollHandle::new(),
            leading_inset: px(0.),
            stale: true,
            reload_task: None,
            _subscriptions: subscriptions,
            _tick: tick,
        };
        view.reload(cx);
        view
    }

    pub fn set_leading_inset(&mut self, inset: Pixels, cx: &mut Context<Self>) {
        if self.leading_inset != inset {
            self.leading_inset = inset;
            cx.notify();
        }
    }

    pub fn focus_list(&self, window: &mut Window, cx: &mut App) {
        window.focus(&self.focus, cx);
    }

    fn reload_soon(&mut self, cx: &mut Context<Self>) {
        self.reload_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(RELOAD_DELAY).await;
            this.update(cx, |this, cx| this.reload(cx)).ok();
        }));
    }

    /// Reads every task again, off the main thread.
    pub fn reload(&mut self, cx: &mut Context<Self>) {
        self.stale = false;
        let store = self.store.clone();
        let task = cx.background_executor().spawn(async move {
            store.tasks(&TaskQuery {
                include_done: true,
                ..Default::default()
            })
        });
        self.reload_task = Some(cx.spawn(async move |this, cx| {
            let rows = task.await;
            this.update(cx, |this, cx| {
                match rows {
                    Ok(rows) => this.board.set_rows(rows),
                    Err(err) => cx.emit(TasksEvent::Notice(format!(
                        "Couldn't read the tasks: {err}"
                    ))),
                }
                cx.notify();
            })
            .ok();
        }));
    }

    /// Reads the tasks now, on this thread: for tests and snapshots.
    #[cfg(test)]
    pub fn reload_now(&mut self, cx: &mut Context<Self>) {
        let rows = self
            .store
            .tasks(&TaskQuery {
                include_done: true,
                ..Default::default()
            })
            .unwrap_or_default();
        self.board.set_rows(rows);
        cx.notify();
    }

    fn now(cx: &App) -> DateTime {
        crate::journal::now(cx)
    }

    fn select_step(&mut self, delta: isize, cx: &mut Context<Self>) {
        self.board.step(delta, Self::now(cx));
        let entries = self.board.entries(Self::now(cx));
        if let Some(at) = self.board.selected_entry(&entries) {
            self.scroll.scroll_to_item(at, ScrollStrategy::Nearest);
        }
        cx.notify();
    }

    /// Ticks (or clears) task `id` in its note. The note's unsaved typing
    /// is saved first, so the write goes on top of it.
    pub fn toggle(&mut self, id: &TaskId, cx: &mut Context<Self>) {
        if writes_held(cx) {
            cx.emit(TasksEvent::Notice(
                "Tasks can't be changed while the library is being replaced.".into(),
            ));
            return;
        }
        self.editor.update(cx, |editor, cx| editor.flush(cx));
        let outcome = self.board.toggle(id, &self.store, Clock::now_ms(cx));
        self.after(outcome, cx);
    }

    fn undo(&mut self, cx: &mut Context<Self>) {
        if writes_held(cx) {
            return;
        }
        self.editor.update(cx, |editor, cx| editor.flush(cx));
        let outcome = self.board.undo(&self.store, Clock::now_ms(cx));
        self.after(outcome, cx);
    }

    fn after(&mut self, outcome: Toggled, cx: &mut Context<Self>) {
        match outcome {
            Toggled::Done(_) | Toggled::Nothing => {}
            Toggled::Moved => {
                cx.emit(TasksEvent::Notice(
                    "That task changed in its note. The list is up to date now.".into(),
                ));
                self.reload(cx);
            }
            Toggled::Failed(err) => cx.emit(TasksEvent::Notice(format!(
                "Couldn't change the task: {err}"
            ))),
        }
        cx.notify();
    }

    fn open(&mut self, id: &TaskId, focus: bool, cx: &mut Context<Self>) {
        self.board.selected = Some(id.clone());
        cx.emit(TasksEvent::Open {
            note_id: id.note_id.clone(),
            line: id.line,
            focus,
        });
        cx.notify();
    }

    fn toggle_group(&mut self, group: TaskGroup, cx: &mut Context<Self>) {
        if !self.board.collapsed.remove(&group) {
            self.board.collapsed.insert(group);
        }
        cx.notify();
    }

    fn set_book(&mut self, book: Option<String>, cx: &mut Context<Self>) {
        self.board.book = book;
        cx.notify();
    }

    fn set_show_done(&mut self, show: bool, cx: &mut Context<Self>) {
        self.board.show_done = show;
        cx.notify();
    }

    // MARK: Drawing

    fn render_top_band(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let open = self
            .board
            .rows
            .iter()
            .filter(|r| !r.checked && self.board.listed(r))
            .count();
        let library = self.library.read(cx);
        let book_name = self
            .board
            .book
            .as_ref()
            .and_then(|id| library.book(id))
            .map(|b| b.name.clone());
        let refined = self.board.book.is_some() || self.board.show_done;
        let view = cx.entity();
        let filter = Button::new("tasks-filter")
            .ghost()
            .small()
            .icon(
                Icon::new(Lucide::ListFilter)
                    .size(px(15.))
                    .text_color(colors.text_secondary),
            )
            .selected(refined)
            .tooltip("Filter tasks")
            .dropdown_menu_with_anchor(Anchor::TopRight, move |menu, _, cx| {
                filter_menu(menu, &view, cx)
            });
        h_flex()
            .h(px(TOP_BAND_HEIGHT))
            .flex_shrink_0()
            .pl(px(16.) + self.leading_inset)
            .pr(px(10.))
            .gap_1p5()
            .child(
                div()
                    .text_size(px(15.))
                    .font_semibold()
                    .text_color(colors.text)
                    .child("Tasks"),
            )
            .child(
                div()
                    .flex_shrink_0()
                    .text_size(px(12.))
                    .text_color(colors.text_muted)
                    .child(open.to_string()),
            )
            .when_some(book_name, |el, name| {
                el.child(
                    div()
                        .min_w_0()
                        .flex_shrink(1000.)
                        .truncate()
                        .text_size(px(12.))
                        .text_color(colors.text_muted)
                        .child(format!("· {name}")),
                )
            })
            .child(div().flex_1())
            .child(
                Button::new("tasks-board")
                    .ghost()
                    .small()
                    .icon(
                        library_icon(Slot::Board)
                            .size(px(15.))
                            .text_color(colors.text_secondary),
                    )
                    .tooltip("Tasks board")
                    .on_click(|_, window, cx| {
                        window.dispatch_action(Box::new(crate::board_view::ShowTaskBoard), cx)
                    }),
            )
            .child(filter)
    }

    fn render_header(
        &self,
        group: TaskGroup,
        count: usize,
        collapsed: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let colors = Colors::get(cx);
        let tint = if group == TaskGroup::Overdue {
            colors.danger
        } else {
            colors.text_muted
        };
        h_flex()
            .id(SharedString::from(format!("tasks-group-{}", group.label())))
            .h(px(HEADER_HEIGHT))
            .w_full()
            .px(px(ROW_INSET_X + ROW_PAD_X - 4.))
            .pt(px(8.))
            .gap(px(4.))
            .cursor_pointer()
            .child(
                library_icon(if collapsed {
                    Slot::FoldClosed
                } else {
                    Slot::FoldOpen
                })
                .size(px(13.))
                .text_color(colors.text_muted),
            )
            .child(
                section_label(div(), colors)
                    .text_color(tint)
                    .child(group.label()),
            )
            .child(
                div()
                    .text_size(px(11.5))
                    .text_color(colors.text_muted)
                    .child(count.to_string()),
            )
            .on_click(cx.listener(move |this, _, _, cx| this.toggle_group(group, cx)))
            .into_any_element()
    }

    fn render_task(
        &self,
        ix: usize,
        selected: bool,
        focused: bool,
        now: DateTime,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let colors = Colors::get(cx);
        let row = &self.board.rows[ix];
        let id = TaskId::of(row);
        let checked = row.checked;
        let due = row.due.as_deref().and_then(Due::parse);
        let library = self.library.read(cx);
        let book = library
            .book(&row.book_id)
            .map(|b| b.name.clone())
            .unwrap_or_default();
        let title = if row.note_title.trim().is_empty() {
            "Untitled".to_owned()
        } else {
            row.note_title.clone()
        };
        let text = display_text(&row.text);
        let toggle_id = id.clone();
        let open_id = id.clone();
        let check = Icon::new(if checked {
            Lucide::CircleCheck
        } else {
            Lucide::Circle
        })
        .size(px(16.))
        .text_color(if checked {
            colors.success
        } else {
            colors.text_muted
        });
        let chip = due.map(|due| {
            let tone = tone(due, checked, now);
            let (text, fill) = chip_colors(tone, colors);
            div()
                .flex_shrink_0()
                .px(px(6.))
                .h(px(18.))
                .flex()
                .items_center()
                .rounded(px(5.))
                .bg(fill)
                .text_size(px(11.))
                .font_medium()
                .text_color(text)
                .when(tone == Tone::Done, |el| el.line_through())
                .child(chip_label(due, checked, now))
        });
        div()
            .id(SharedString::from(format!(
                "task-{}-{}",
                row.note_id, row.line
            )))
            .h(px(ROW_HEIGHT))
            .w_full()
            .px(px(ROW_INSET_X))
            .py(px(1.))
            .child(
                h_flex()
                    .size_full()
                    .px(px(ROW_PAD_X))
                    .gap(px(10.))
                    .items_start()
                    .pt(px(9.))
                    .rounded(px(7.))
                    .when(selected, |el| el.bg(colors.row_active))
                    .when(!selected, |el| el.hover(|el| el.bg(colors.row_hover)))
                    .when(selected && focused, |el| {
                        el.border_1()
                            .border_color(Hsla::from(colors.accent).opacity(0.5))
                    })
                    .child(
                        div()
                            .id(SharedString::from(format!(
                                "task-check-{}-{}",
                                row.note_id, row.line
                            )))
                            .flex_none()
                            .pt(px(1.))
                            .cursor_pointer()
                            .child(check)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                cx.stop_propagation();
                                this.toggle(&toggle_id, cx);
                            })),
                    )
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .gap(px(3.))
                            .child(
                                h_flex()
                                    .gap(px(8.))
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_w_0()
                                            .truncate()
                                            .text_size(px(13.5))
                                            .line_height(px(18.))
                                            .text_color(if checked {
                                                colors.text_muted
                                            } else {
                                                colors.text
                                            })
                                            .when(checked, |el| el.line_through())
                                            .child(text),
                                    )
                                    .children(chip),
                            )
                            .child(
                                div()
                                    .truncate()
                                    .text_size(px(12.))
                                    .line_height(px(16.))
                                    .text_color(colors.quiet(selected))
                                    .child(if book.is_empty() {
                                        title
                                    } else {
                                        format!("{title} · {book}")
                                    }),
                            ),
                    ),
            )
            .on_click(cx.listener(move |this, _, window, cx| {
                window.focus(&this.focus, cx);
                this.open(&open_id, false, cx);
            }))
            .into_any_element()
    }

    fn render_rows(
        &mut self,
        range: std::ops::Range<usize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Vec<AnyElement> {
        let now = Self::now(cx);
        let entries = self.board.entries(now);
        let selected = self.board.selected_entry(&entries);
        let focused = self.focus.contains_focused(window, cx);
        range
            .filter_map(|at| {
                Some(match *entries.get(at)? {
                    Entry::Header {
                        group,
                        count,
                        collapsed,
                    } => self.render_header(group, count, collapsed, cx),
                    Entry::Task(ix) => self.render_task(ix, selected == Some(at), focused, now, cx),
                })
            })
            .collect()
    }

    fn render_empty(&self, cx: &App) -> AnyElement {
        let colors = Colors::get(cx);
        if !self.board.loaded {
            return div().into_any_element();
        }
        let any = self.board.rows.iter().any(|r| {
            self.board
                .book
                .as_ref()
                .is_none_or(|book| r.book_id == *book)
        });
        let (title, detail) = if !any && self.board.book.is_some() {
            let name = self
                .board
                .book
                .as_ref()
                .and_then(|id| self.library.read(cx).book(id))
                .map_or("this notebook".to_owned(), |b| b.name.clone());
            (format!("No tasks in {name}"), None)
        } else if !any {
            (
                "No tasks yet".to_owned(),
                Some(
                    "Write “- [ ] Call Sam @2026-10-25” in any note, and it shows here.".to_owned(),
                ),
            )
        } else {
            (
                "All done".to_owned(),
                Some("Every task is ticked off.".to_owned()),
            )
        };
        quiet_empty(Lucide::ListChecks, title, detail, colors).into_any_element()
    }

    fn render_bottom_band(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let last = self.board.last.clone();
        h_flex()
            .h(px(BOTTOM_BAND_HEIGHT))
            .flex_shrink_0()
            .px(px(12.))
            .gap_1()
            .border_t_1()
            .border_color(colors.divider)
            .text_size(px(12.))
            .text_color(colors.text_muted)
            .map(|el| match last {
                Some(last) => el
                    .child(div().flex_1().min_w_0().truncate().child(if last.checked {
                        format!("Ticked off “{}”", display_text(&last.id.text))
                    } else {
                        format!("Opened “{}” again", display_text(&last.id.text))
                    }))
                    .child(
                        Button::new("tasks-undo")
                            .ghost()
                            .xsmall()
                            .label("Undo")
                            .tooltip_with_action("Undo", &UndoCheckOff, Some("TasksView"))
                            .on_click(cx.listener(|this, _, _, cx| this.undo(cx))),
                    ),
                None => el.child(
                    div()
                        .flex_1()
                        .truncate()
                        .child("Space ticks off · Return opens the note"),
                ),
            })
    }
}

fn writes_held(cx: &App) -> bool {
    cx.try_global::<crate::services::Services>()
        .is_some_and(|services| services.writes_held)
}

/// The filter menu: which notebook's tasks, and whether done ones show.
fn filter_menu(menu: PopupMenu, view: &Entity<TasksView>, cx: &App) -> PopupMenu {
    let board = &view.read(cx).board;
    let current = board.book.clone();
    let show_done = board.show_done;
    let library = view.read(cx).library.read(cx);
    let books: Vec<(usize, String, String)> = library
        .book_tree()
        .into_iter()
        .map(|(depth, b)| (depth, b.id.clone(), b.name.clone()))
        .collect();
    let all = view.clone();
    let mut menu = menu.item(
        PopupMenuItem::new("All notebooks")
            .checked(current.is_none())
            .on_click(move |_, _, cx| all.update(cx, |this, cx| this.set_book(None, cx))),
    );
    for (depth, id, name) in books {
        let view = view.clone();
        let label = format!("{}{name}", "   ".repeat(depth));
        let checked = current.as_deref() == Some(id.as_str());
        menu = menu.item(
            PopupMenuItem::new(label)
                .checked(checked)
                .on_click(move |_, _, cx| {
                    let id = id.clone();
                    view.update(cx, |this, cx| this.set_book(Some(id), cx))
                }),
        );
    }
    let done = view.clone();
    menu.separator().item(
        PopupMenuItem::new("Show Completed")
            .checked(show_done)
            .on_click(move |_, _, cx| {
                done.update(cx, |this, cx| this.set_show_done(!show_done, cx))
            }),
    )
}

/// Each line's height for the virtual list.
fn item_sizes(entries: &[Entry]) -> Rc<Vec<Size<Pixels>>> {
    Rc::new(
        entries
            .iter()
            .map(|entry| {
                let height = match entry {
                    Entry::Header { .. } => HEADER_HEIGHT,
                    Entry::Task(_) => ROW_HEIGHT,
                };
                size(px(0.), px(height))
            })
            .collect(),
    )
}

impl Render for TasksView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.stale && self.reload_task.is_none() {
            self.reload(cx);
        }
        let colors = Colors::get(cx);
        let entries = self.board.entries(Self::now(cx));
        let body = if entries.is_empty() {
            self.render_empty(cx)
        } else {
            v_virtual_list(
                cx.entity(),
                "tasks",
                item_sizes(&entries),
                |this, range, window, cx| this.render_rows(range, window, cx),
            )
            .size_full()
            .track_scroll(&self.scroll)
            .into_any_element()
        };
        let _ = window;
        v_flex()
            .size_full()
            .bg(colors.list)
            .child(self.render_top_band(cx))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .key_context("TasksView")
                    .track_focus(&self.focus)
                    .on_action(
                        cx.listener(|this, _: &SelectNextTask, _, cx| this.select_step(1, cx)),
                    )
                    .on_action(
                        cx.listener(|this, _: &SelectPreviousTask, _, cx| this.select_step(-1, cx)),
                    )
                    .on_action(cx.listener(|this, _: &ToggleTask, _, cx| {
                        if let Some(row) = this.board.selected_row() {
                            let id = TaskId::of(row);
                            this.toggle(&id, cx);
                        }
                    }))
                    .on_action(cx.listener(|this, _: &OpenTask, _, cx| {
                        if let Some(row) = this.board.selected_row() {
                            let id = TaskId::of(row);
                            this.open(&id, true, cx);
                        }
                    }))
                    .on_action(cx.listener(|this, _: &UndoCheckOff, _, cx| this.undo(cx)))
                    .child(body),
            )
            .child(self.render_bottom_band(cx))
    }
}
