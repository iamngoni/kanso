//! Drawing the board (`board.rs`) over the window, as the library's graph
//! is: the notes the list showed when it opened, in a column per status,
//! or their tasks in To do and Done. A card dragged to another column, or
//! moved with ⇧← and ⇧→, changes its note's status or ticks its task.
//! Return or a double-click opens the card's note.

use std::sync::Arc;
use std::time::Duration;

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonGroup, ButtonVariants as _};
use gpui_kit::component::notification::Notification;
use gpui_kit::component::{
    Icon, Selectable as _, Sizable as _, StyledExt as _, WindowExt as _, h_flex, v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_core::tasks::Due;
use kanso_engine::board::{Board, BoardKind, Card, CardId, Lane, Move, Written, cards, write_move};
use kanso_store::{Sort, Store, TaskQuery};

use crate::clock::Clock;
use crate::due::{Tone, chip_colors, chip_label, tone};
use crate::icon_themes::{Slot, library_icon};
use crate::layout::{BOTTOM_BAND_HEIGHT, TOP_BAND_HEIGHT, inset_after};
use crate::library::{Filter, Library, LibraryEvent, list_notes, search_params};
use crate::note_list::quiet_empty;
use crate::tasks_view::display_text;
use crate::theme::{Colors, section_label, status_glyph};
use crate::workspace::Workspace;
use crate::zoom::px;

const COLUMN_WIDTH: f32 = 272.;
/// How long the library stays still before a shown board is read again.
const RELOAD_DELAY: Duration = Duration::from_millis(150);
const DAY_MS: i64 = 86_400_000;

actions!(kanso, [ShowBoard, ShowTaskBoard, CloseBoardView]);
actions!(
    board_view,
    [
        SelectCardUp,
        SelectCardDown,
        SelectCardLeft,
        SelectCardRight,
        MoveCardLeft,
        MoveCardRight,
        OpenCard,
        CloseBoard
    ]
);

pub fn bind_keys(cx: &mut App) {
    let context = Some("BoardView");
    cx.bind_keys([
        KeyBinding::new("up", SelectCardUp, context),
        KeyBinding::new("k", SelectCardUp, context),
        KeyBinding::new("down", SelectCardDown, context),
        KeyBinding::new("j", SelectCardDown, context),
        KeyBinding::new("left", SelectCardLeft, context),
        KeyBinding::new("h", SelectCardLeft, context),
        KeyBinding::new("right", SelectCardRight, context),
        KeyBinding::new("l", SelectCardRight, context),
        KeyBinding::new("shift-left", MoveCardLeft, context),
        KeyBinding::new("shift-h", MoveCardLeft, context),
        KeyBinding::new("shift-right", MoveCardRight, context),
        KeyBinding::new("shift-l", MoveCardRight, context),
        KeyBinding::new("enter", OpenCard, context),
        KeyBinding::new("escape", CloseBoard, context),
    ]);
}

/// What the board asks of the workspace.
pub enum BoardEvent {
    /// Write a move to its note.
    Move(Move),
    /// Open a card's note (at a task's line).
    Open { note_id: String, line: Option<u32> },
}

/// A card being dragged, and what the drag shows under the pointer.
#[derive(Clone)]
pub struct DraggedCard {
    pub id: CardId,
    title: SharedString,
}

impl Render for DraggedCard {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        div()
            .w(px(COLUMN_WIDTH - 24.))
            .px(px(12.))
            .py(px(10.))
            .rounded(px(8.))
            .border_1()
            .border_color(Hsla::from(colors.accent).opacity(0.6))
            .bg(colors.elevated)
            .shadow_md()
            .text_size(px(13.5))
            .font_medium()
            .text_color(colors.text)
            .truncate()
            .child(self.title.clone())
    }
}

/// Which notes a board lays out: what the list showed when it opened.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoardScope {
    pub filter: Filter,
    pub search: String,
    pub sort: Sort,
    pub descending: bool,
}

pub struct BoardView {
    library: Entity<Library>,
    store: Arc<Store>,
    pub(crate) board: Board,
    pub(crate) scope: BoardScope,
    /// Drawn on screen: only then is it read again on changes.
    pub visible: bool,
    stale: bool,
    focus: FocusHandle,
    lanes_scroll: ScrollHandle,
    column_scrolls: Vec<ScrollHandle>,
    reload_task: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<BoardEvent> for BoardView {}

impl Focusable for BoardView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl BoardView {
    pub fn new(
        store: Arc<Store>,
        library: Entity<Library>,
        kind: BoardKind,
        scope: BoardScope,
        cx: &mut Context<Self>,
    ) -> Self {
        let subscriptions = vec![cx.subscribe(&library, |this, _, event, cx| {
            if let LibraryEvent::Changed(change) = event
                && change.id.starts_with("note:")
            {
                this.stale = true;
                if this.visible {
                    this.reload_soon(cx);
                }
            }
        })];
        let mut view = BoardView {
            library,
            store,
            board: Board::new(kind),
            scope,
            visible: false,
            stale: true,
            focus: cx.focus_handle(),
            lanes_scroll: ScrollHandle::new(),
            column_scrolls: Vec::new(),
            reload_task: None,
            _subscriptions: subscriptions,
        };
        view.reload(cx);
        view
    }

    /// Lays out `kind` for `scope`, keeping the selection when nothing
    /// changed.
    pub fn show(&mut self, kind: BoardKind, scope: BoardScope, cx: &mut Context<Self>) {
        if self.board.kind != kind || self.scope != scope {
            self.board = Board::new(kind);
            self.scope = scope;
        }
        self.visible = true;
        self.reload(cx);
    }

    fn set_kind(&mut self, kind: BoardKind, cx: &mut Context<Self>) {
        if self.board.kind != kind {
            self.board = Board::new(kind);
            self.reload(cx);
            cx.notify();
        }
    }

    fn reload_soon(&mut self, cx: &mut Context<Self>) {
        self.reload_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(RELOAD_DELAY).await;
            this.update(cx, |this, cx| this.reload(cx)).ok();
        }));
    }

    /// Reads the cards again, off the main thread.
    pub fn reload(&mut self, cx: &mut Context<Self>) {
        self.stale = false;
        let store = self.store.clone();
        let scope = self.scope.clone();
        let kind = self.board.kind;
        let recent_since = Clock::now_ms(cx) - crate::library::RECENT_DAYS * DAY_MS;
        let task = cx
            .background_executor()
            .spawn(async move { read_cards(&store, kind, &scope, recent_since) });
        self.reload_task = Some(cx.spawn(async move |this, cx| {
            let read = task.await;
            this.update(cx, |this, cx| {
                // A board of the other kind was asked for meanwhile.
                if this.board.kind == kind {
                    match read {
                        Ok(cards) => this.board.set_cards(cards),
                        Err(err) => eprintln!("kanso: board: {err}"),
                    }
                }
                cx.notify();
            })
            .ok();
        }));
    }

    /// Reads the cards now, on this thread: for tests and snapshots.
    #[cfg(test)]
    pub fn reload_now(&mut self, cx: &mut Context<Self>) {
        let recent_since = Clock::now_ms(cx) - crate::library::RECENT_DAYS * DAY_MS;
        if let Ok(cards) = read_cards(&self.store, self.board.kind, &self.scope, recent_since) {
            self.board.set_cards(cards);
        }
        cx.notify();
    }

    fn scroll_to_selection(&self) {
        if let Some((col, row)) = self.board.selected_position() {
            self.lanes_scroll.scroll_to_item(col);
            if let Some(scroll) = self.column_scrolls.get(col) {
                scroll.scroll_to_item(row);
            }
        }
    }

    fn step(&mut self, down: isize, across: isize, cx: &mut Context<Self>) {
        if across != 0 {
            self.board.step_across(across);
        } else {
            self.board.step_down(down);
        }
        self.scroll_to_selection();
        cx.notify();
    }

    fn move_selected(&mut self, delta: isize, cx: &mut Context<Self>) {
        if let Some(mv) = self.board.move_selected(delta) {
            cx.emit(BoardEvent::Move(mv));
        }
        self.scroll_to_selection();
        cx.notify();
    }

    fn drop_card(&mut self, id: &CardId, lane: Lane, cx: &mut Context<Self>) {
        if let Some(mv) = self.board.move_card(id, lane) {
            cx.emit(BoardEvent::Move(mv));
        }
        cx.notify();
    }

    fn open(&mut self, id: &CardId, cx: &mut Context<Self>) {
        self.board.selected = Some(id.clone());
        let Some(card) = self.board.selected_card() else {
            return;
        };
        let line = match card {
            Card::Task(row) => Some(row.line),
            Card::Note(_) => None,
        };
        cx.emit(BoardEvent::Open {
            note_id: card.note_id().to_owned(),
            line,
        });
    }

    // MARK: Drawing

    fn render_toolbar(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let library = self.library.read(cx);
        let place = library.title_of(&self.scope.filter);
        let search = self.scope.search.trim();
        let place = if search.is_empty() {
            place
        } else {
            format!("{place} · matching “{search}”")
        };
        let count = match self.board.kind {
            BoardKind::Notes => crate::text_stats::plural(self.board.cards.len(), "note", "notes"),
            BoardKind::Tasks => crate::text_stats::plural(self.board.cards.len(), "task", "tasks"),
        };
        let kind = self.board.kind;
        let kinds = ButtonGroup::new("board-kind")
            .outline()
            .small()
            .child(
                Button::new("board-notes")
                    .label("Notes")
                    .selected(kind == BoardKind::Notes),
            )
            .child(
                Button::new("board-tasks")
                    .label("Tasks")
                    .selected(kind == BoardKind::Tasks),
            )
            .on_click(cx.listener(|this, clicked: &Vec<usize>, _, cx| {
                let kind = if clicked.first() == Some(&1) {
                    BoardKind::Tasks
                } else {
                    BoardKind::Notes
                };
                this.set_kind(kind, cx);
            }));
        h_flex()
            .h(px(TOP_BAND_HEIGHT))
            .flex_none()
            .pl(px(inset_after(0., window.is_fullscreen())))
            .pr(px(12.))
            .gap(px(10.))
            .border_b_1()
            .border_color(colors.divider)
            .bg(colors.list)
            .child(
                div()
                    .text_size(px(14.))
                    .font_medium()
                    .text_color(colors.text)
                    .child("Board"),
            )
            .child(
                div()
                    .min_w_0()
                    .flex_shrink(1000.)
                    .truncate()
                    .text_size(px(12.5))
                    .text_color(colors.text_muted)
                    .child(format!("{place} · {count}")),
            )
            .child(div().flex_1())
            .child(kinds)
            .child(
                Button::new("board-close")
                    .ghost()
                    .small()
                    .icon(Lucide::Close)
                    .tooltip("Close (Esc)")
                    .on_click(cx.listener(|_, _, window, cx| {
                        window.dispatch_action(Box::new(CloseBoard), cx)
                    })),
            )
    }

    fn render_column(
        &self,
        col: usize,
        lane: Lane,
        cards: &[usize],
        selected: Option<(usize, usize)>,
        focused: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let colors = Colors::get(cx);
        let glyph = match lane {
            Lane::Status(status) => status_glyph(status, colors, colors.text_muted),
            Lane::Todo => Some((Lucide::Circle, colors.text_muted)),
            Lane::Done => Some((Lucide::CircleCheck, colors.success)),
        };
        let now = crate::journal::now(cx);
        let scroll = self.column_scrolls.get(col).cloned().unwrap_or_default();
        let accent = Hsla::from(colors.accent);
        v_flex()
            .id(SharedString::from(format!("board-column-{}", lane.label())))
            .w(px(COLUMN_WIDTH))
            .flex_none()
            .h_full()
            .rounded(px(10.))
            .bg(colors.list)
            .border_1()
            .border_color(colors.divider)
            .drag_over::<DraggedCard>(move |style, _, _, _| {
                style
                    .border_color(accent.opacity(0.7))
                    .bg(accent.opacity(0.06))
            })
            .on_drop(cx.listener(move |this, card: &DraggedCard, _, cx| {
                this.drop_card(&card.id, lane, cx);
            }))
            .child(
                h_flex()
                    .flex_none()
                    .h(px(36.))
                    .px(px(12.))
                    .gap(px(6.))
                    .when_some(glyph, |el, (icon, color)| {
                        el.child(Icon::new(icon).size(px(14.)).text_color(color))
                    })
                    .child(section_label(div(), colors).child(lane.label()))
                    .child(
                        div()
                            .text_size(px(11.5))
                            .text_color(colors.text_muted)
                            .child(cards.len().to_string()),
                    ),
            )
            .child(
                v_flex()
                    .id(SharedString::from(format!("board-cards-{}", lane.label())))
                    .flex_1()
                    .min_h_0()
                    .px(px(8.))
                    .pb(px(8.))
                    .gap(px(6.))
                    .overflow_y_scroll()
                    .track_scroll(&scroll)
                    .children(cards.iter().enumerate().map(|(row, &ix)| {
                        let selected = selected == Some((col, row));
                        self.render_card(ix, selected, focused, now, cx)
                    })),
            )
            .into_any_element()
    }

    fn render_card(
        &self,
        ix: usize,
        selected: bool,
        focused: bool,
        now: kanso_core::dates::DateTime,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let colors = Colors::get(cx);
        let card = &self.board.cards[ix];
        let id = card.id();
        let library = self.library.read(cx);
        let untitled = |title: &str| {
            if title.trim().is_empty() {
                "Untitled".to_owned()
            } else {
                title.to_owned()
            }
        };
        let (title, body): (String, AnyElement) = match card {
            Card::Note(note) => {
                let book = library
                    .book(&note.book_id)
                    .map(|b| b.name.clone())
                    .filter(|_| !matches!(self.scope.filter, Filter::Book(_)));
                let tasks = (note.tasks_total > 0)
                    .then(|| format!("{}/{} tasks", note.tasks_done, note.tasks_total));
                let footer: Vec<String> = book.into_iter().chain(tasks).collect();
                let title = untitled(&note.title);
                let (mark, word) = crate::locked::note_mark(cx);
                let snippet = if note.locked {
                    word.to_owned()
                } else {
                    note.snippet.trim().to_owned()
                };
                let body = v_flex()
                    .gap(px(4.))
                    .child(
                        h_flex()
                            .gap(px(6.))
                            .when(note.locked, |el| {
                                el.child(mark.size(px(12.)).text_color(colors.text_muted))
                            })
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .truncate()
                                    .text_size(px(13.5))
                                    .font_medium()
                                    .text_color(colors.text)
                                    .child(title.clone()),
                            ),
                    )
                    .when(!snippet.is_empty() && !note.locked, |el| {
                        el.child(
                            div()
                                .text_size(px(12.))
                                .line_height(px(16.))
                                .text_color(colors.quiet(selected))
                                .line_clamp(2)
                                .child(snippet),
                        )
                    })
                    .when(!footer.is_empty(), |el| {
                        el.child(
                            div()
                                .truncate()
                                .text_size(px(11.5))
                                .text_color(colors.text_muted)
                                .child(footer.join(" · ")),
                        )
                    })
                    .into_any_element();
                (title, body)
            }
            Card::Task(row) => {
                let text = display_text(&row.text);
                let note_title = untitled(&row.note_title);
                let book = library
                    .book(&row.book_id)
                    .map(|b| b.name.clone())
                    .unwrap_or_default();
                let due = row.due.as_deref().and_then(Due::parse);
                let chip = due.map(|due| {
                    let tone = tone(due, row.checked, now);
                    let (text, fill) = chip_colors(tone, colors);
                    div()
                        .flex_none()
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
                        .child(chip_label(due, row.checked, now))
                });
                let body = v_flex()
                    .gap(px(4.))
                    .child(
                        div()
                            .text_size(px(13.5))
                            .line_height(px(18.))
                            .line_clamp(3)
                            .text_color(if row.checked {
                                colors.text_muted
                            } else {
                                colors.text
                            })
                            .when(row.checked, |el| el.line_through())
                            .child(text.clone()),
                    )
                    .child(
                        h_flex()
                            .gap(px(6.))
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .truncate()
                                    .text_size(px(11.5))
                                    .text_color(colors.text_muted)
                                    .child(if book.is_empty() {
                                        note_title
                                    } else {
                                        format!("{note_title} · {book}")
                                    }),
                            )
                            .children(chip),
                    )
                    .into_any_element();
                (text, body)
            }
        };
        let dragged = DraggedCard {
            id: id.clone(),
            title: title.into(),
        };
        let open_id = id.clone();
        div()
            .id(SharedString::from(format!("board-card-{ix}")))
            .flex_none()
            .w_full()
            .px(px(12.))
            .py(px(10.))
            .rounded(px(8.))
            .bg(if selected {
                colors.row_active
            } else {
                colors.editor
            })
            .border_1()
            .border_color(if selected && focused {
                Hsla::from(colors.accent).opacity(0.6)
            } else {
                colors.divider.into()
            })
            .when(!selected, |el| el.hover(|el| el.bg(colors.row_hover)))
            .cursor_grab()
            .child(body)
            .on_drag(dragged, |card, _, _, cx| cx.new(|_| card.clone()))
            .on_click(cx.listener(move |this, event: &ClickEvent, window, cx| {
                window.focus(&this.focus, cx);
                if event.click_count() >= 2 {
                    this.open(&open_id, cx);
                } else {
                    this.board.selected = Some(open_id.clone());
                    cx.notify();
                }
            }))
            .into_any_element()
    }

    fn render_empty(&self, cx: &App) -> AnyElement {
        let colors = Colors::get(cx);
        if !self.board.loaded {
            return div().into_any_element();
        }
        let (title, detail) = match self.board.kind {
            BoardKind::Notes => ("No notes here".to_owned(), None),
            BoardKind::Tasks => (
                "No tasks in these notes".to_owned(),
                Some("Write “- [ ] Call Sam” in a note, and it shows here.".to_owned()),
            ),
        };
        quiet_empty(library_icon(Slot::Board), title, detail, colors).into_any_element()
    }
}

/// The cards of a `kind` board over `scope`'s notes. The trash never
/// shows.
fn read_cards(
    store: &Store,
    kind: BoardKind,
    scope: &BoardScope,
    recent_since: i64,
) -> Result<Vec<Card>, kanso_store::StoreError> {
    let params = search_params(&scope.filter, &scope.search, scope.sort, scope.descending);
    let mut notes = list_notes(store, &scope.filter, &params, false, recent_since)?;
    notes.retain(|note| note.book_id != kanso_core::model::TRASH_BOOK_ID);
    let tasks = match kind {
        BoardKind::Notes => Vec::new(),
        BoardKind::Tasks => store.tasks(&TaskQuery {
            include_done: true,
            ..Default::default()
        })?,
    };
    Ok(cards(kind, notes, tasks))
}

impl Render for BoardView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.visible && self.stale && self.reload_task.is_none() {
            self.reload(cx);
        }
        let colors = Colors::get(cx);
        let columns = self.board.columns();
        self.column_scrolls
            .resize_with(columns.len(), ScrollHandle::new);
        let selected = self.board.selected_position();
        let focused = self.focus.contains_focused(window, cx);
        let body = if self.board.cards.is_empty() {
            self.render_empty(cx)
        } else {
            h_flex()
                .id("board-lanes")
                .size_full()
                .items_start()
                .gap(px(12.))
                .p(px(16.))
                .overflow_x_scroll()
                .track_scroll(&self.lanes_scroll)
                .children(columns.iter().enumerate().map(|(col, (lane, cards))| {
                    self.render_column(col, *lane, cards, selected, focused, cx)
                }))
                .into_any_element()
        };
        let hint = match self.board.kind {
            BoardKind::Notes => {
                "Drag a card to change its status · Arrows choose · ⇧← ⇧→ move · Return opens"
            }
            BoardKind::Tasks => {
                "Drag a task to Done to tick it off · Arrows choose · ⇧← ⇧→ move · Return opens"
            }
        };
        v_flex()
            .id("board-view")
            .key_context("BoardView")
            .track_focus(&self.focus)
            .size_full()
            .bg(colors.editor)
            .on_action(cx.listener(|this, _: &SelectCardUp, _, cx| this.step(-1, 0, cx)))
            .on_action(cx.listener(|this, _: &SelectCardDown, _, cx| this.step(1, 0, cx)))
            .on_action(cx.listener(|this, _: &SelectCardLeft, _, cx| this.step(0, -1, cx)))
            .on_action(cx.listener(|this, _: &SelectCardRight, _, cx| this.step(0, 1, cx)))
            .on_action(cx.listener(|this, _: &MoveCardLeft, _, cx| this.move_selected(-1, cx)))
            .on_action(cx.listener(|this, _: &MoveCardRight, _, cx| this.move_selected(1, cx)))
            .on_action(cx.listener(|this, _: &OpenCard, _, cx| {
                if let Some(id) = this.board.selected.clone() {
                    this.open(&id, cx);
                }
            }))
            .on_action(cx.listener(|this, _: &CloseBoard, window, cx| {
                this.visible = false;
                window.dispatch_action(Box::new(CloseBoardView), cx);
            }))
            // Presses here are the board's, not the window's below.
            .on_any_mouse_down(|_, _, cx| cx.stop_propagation())
            .child(self.render_toolbar(window, cx))
            .child(div().flex_1().min_h_0().child(body))
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
                    .child(div().truncate().child(hint)),
            )
    }
}

/// The workspace's part: opening and closing the board, and writing its
/// moves.
impl Workspace {
    pub(crate) fn show_board(
        &mut self,
        kind: BoardKind,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.close_settings(window, cx);
        self.close_graph(window, cx);
        self.set_focus_mode(false, window, cx);
        self.flush(cx);
        let scope = self.board_scope(cx);
        let board = match &self.board {
            Some(board) => board.clone(),
            None => {
                let store = self.library.read(cx).store.clone();
                let library = self.library.clone();
                let scope = scope.clone();
                let board = cx.new(|cx| BoardView::new(store, library, kind, scope, cx));
                self._board_events = Some(cx.subscribe_in(
                    &board,
                    window,
                    |this, _, event, window, cx| this.on_board_event(event, window, cx),
                ));
                self.board = Some(board.clone());
                board
            }
        };
        board.update(cx, |board, cx| board.show(kind, scope, cx));
        self.board_open = true;
        window.focus(&board.focus_handle(cx), cx);
        cx.notify();
    }

    /// What the list shows, for a board: its notebook, tag or place and
    /// its search, but every status. From Tasks, the notebook its tasks
    /// were narrowed to.
    fn board_scope(&self, cx: &App) -> BoardScope {
        let library = self.library.read(cx);
        let filter = match &library.filter {
            Filter::Tasks => match &self.tasks.read(cx).board.book {
                Some(book) => Filter::Book(book.clone()),
                None => Filter::All,
            },
            other => crate::library::board_scope(other),
        };
        let search = match library.filter {
            Filter::Tasks => String::new(),
            _ => library.search.clone(),
        };
        BoardScope {
            filter,
            search,
            sort: library.sort,
            descending: library.descending,
        }
    }

    pub(crate) fn close_board(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.board_open {
            return;
        }
        self.board_open = false;
        if let Some(board) = &self.board {
            board.update(cx, |board, _| board.visible = false);
        }
        window.focus(&self.focus, cx);
        cx.notify();
    }

    fn on_board_event(&mut self, event: &BoardEvent, window: &mut Window, cx: &mut Context<Self>) {
        match event {
            BoardEvent::Move(mv) => self.write_board_move(mv, window, cx),
            BoardEvent::Open { note_id, line } => {
                self.close_board(window, cx);
                match line {
                    Some(line) => self.open_note_at(note_id, *line, true, window, cx),
                    None => {
                        let id = note_id.clone();
                        self.library
                            .update(cx, |library, cx| library.select(Some(id), cx));
                        self.editor
                            .update(cx, |editor, cx| editor.focus_body(window, cx));
                    }
                }
            }
        }
    }

    /// Writes a board's move: the open note's status through the editor,
    /// so its unsaved typing stays; anything else straight to the store,
    /// after the open note's typing is saved.
    fn write_board_move(&mut self, mv: &Move, window: &mut Window, cx: &mut Context<Self>) {
        let held = cx
            .try_global::<crate::services::Services>()
            .is_some_and(|services| services.writes_held);
        if held {
            self.notify_error(
                "Notes can't be changed while the library is being replaced.",
                window,
                cx,
            );
            self.reload_board(cx);
            return;
        }
        if let (CardId::Note(id), Lane::Status(status)) = (&mv.id, mv.to)
            && self.editor.read(cx).note_id() == Some(id.as_str())
        {
            self.editor
                .update(cx, |editor, cx| editor.set_status(status, cx));
            return;
        }
        self.flush(cx);
        let now = Clock::now_ms(cx);
        match write_move(&self.store, mv, now) {
            Ok(Written::Done) => {}
            Ok(Written::Moved) => {
                window.push_notification(
                    Notification::info(
                        "That task changed in its note. The board is up to date now.",
                    ),
                    cx,
                );
                self.reload_board(cx);
            }
            Err(err) => {
                self.notify_error(format!("Couldn't move the card: {err}"), window, cx);
                self.reload_board(cx);
            }
        }
    }

    fn reload_board(&self, cx: &mut Context<Self>) {
        if let Some(board) = &self.board {
            board.update(cx, |board, cx| board.reload(cx));
        }
    }
}

/// The workspace's actions for the board.
pub fn on_actions(el: Div, cx: &mut Context<Workspace>) -> Div {
    el.on_action(
        cx.listener(|this, _: &ShowBoard, window, cx| {
            this.show_board(BoardKind::Notes, window, cx)
        }),
    )
    .on_action(cx.listener(|this, _: &ShowTaskBoard, window, cx| {
        this.show_board(BoardKind::Tasks, window, cx)
    }))
    .on_action(cx.listener(|this, _: &CloseBoardView, window, cx| this.close_board(window, cx)))
}

/// The board over the window, while open.
pub fn overlay(workspace: &Workspace) -> Option<impl IntoElement> {
    let board = workspace.board.clone().filter(|_| workspace.board_open)?;
    Some(
        div()
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .debug_selector(|| "board-overlay".into())
            .child(board),
    )
}
