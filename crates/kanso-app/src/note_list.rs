//! The middle column: the notebook and a filter menu on top, the search box,
//! the notes (under date headers when sorted by date), and New note with the
//! sort at the bottom.

use std::rc::Rc;
use std::time::Duration;

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::menu::{ContextMenuExt as _, DropdownMenu as _, PopupMenu, PopupMenuItem};
use gpui_kit::component::{
    ActiveTheme as _, Icon, IconName, Selectable as _, Sizable as _, StyledExt as _,
    VirtualListScrollHandle, h_flex, v_flex, v_virtual_list,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_core::model::TRASH_BOOK_ID;
use kanso_core::{Field, NoteStatus, Query};
use kanso_engine::list;
use kanso_store::{NoteSummary, Sort};

use crate::clock::Clock;
use crate::grouping::{Grouped, Row, RowKey, bucket_label, day_of, group_rows, row_time_label};
use crate::icon_themes::{Slot, library_icon};
use crate::layout::{BOTTOM_BAND_HEIGHT, TOP_BAND_HEIGHT};
use crate::library::{Filter, Library, LibraryEvent};
use crate::note_menu::NoteCommand;
use crate::selection::Click;
use crate::text_match::{highlight_terms, match_ranges};
use crate::theme::{Colors, section_label, status_glyph};
use crate::workspace::SearchAllNotes;
use crate::zoom::px;

const SEARCH_DELAY: Duration = Duration::from_millis(120);

/// A note row, top to bottom: a sliver between neighbouring fills, padding,
/// the title line, a gap, two lines of excerpt, padding, the sliver. The
/// test checks they add up to the row.
const ROW_HEIGHT: f32 = 80.;
const ROW_INSET_Y: f32 = 1.;
const ROW_PAD_Y: f32 = 9.;
const TITLE_LINE: f32 = 20.;
const ROW_GAP: f32 = 4.;
const EXCERPT_LINE: f32 = 18.;
const EXCERPT_LINES: f32 = 2.;
/// A date header: space above, the label, space below, a divider.
const HEADER_HEIGHT: f32 = 30.;
const HEADER_PAD_TOP: f32 = 8.;
const HEADER_LINE: f32 = 16.;
const HEADER_PAD_BOTTOM: f32 = 5.;
const DIVIDER: f32 = 1.;
/// Rows sit this far in from the column's edges, and their text this much
/// further, so a selected row's fill has room around it.
const ROW_INSET_X: f32 = 8.;
const ROW_PAD_X: f32 = 10.;
/// The status glyph's column, when any listed note has a status.
const GLYPH_COLUMN: f32 = 14.;
const GLYPH_GAP: f32 = 8.;

actions!(
    note_list,
    [
        SelectNext,
        SelectPrevious,
        SelectAllNotes,
        SelectOneNote,
        OpenFocusedInNewTab
    ]
);

pub fn bind_keys(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("down", SelectNext, Some("NoteList")),
        KeyBinding::new("up", SelectPrevious, Some("NoteList")),
        KeyBinding::new("j", SelectNext, Some("NoteList")),
        KeyBinding::new("k", SelectPrevious, Some("NoteList")),
        KeyBinding::new("secondary-a", SelectAllNotes, Some("NoteList")),
        KeyBinding::new("secondary-enter", OpenFocusedInNewTab, Some("NoteList")),
        KeyBinding::new("escape", SelectOneNote, Some("NoteList")),
    ]);
}

/// Note operations the workspace carries out.
pub enum NoteListEvent {
    NewNote,
    /// A choice from a note's menu, or its pin or trash button.
    Command(String, NoteCommand),
    /// A choice from the menu of several selected notes, for all of them.
    CommandSeveral(Vec<String>, NoteCommand),
    Restore(String),
    DeleteForever(String),
    /// Enter in the list or the search box: move focus to the editor.
    OpenSelected,
}

/// Notes being dragged from the list, and what the drag shows under the
/// pointer: the note's title, or how many notes.
#[derive(Clone)]
pub struct DraggedNotes {
    pub ids: Rc<[String]>,
    label: SharedString,
}

impl Render for DraggedNotes {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        div()
            .max_w(px(240.))
            .px(px(10.))
            .py(px(6.))
            .rounded(px(6.))
            .border_1()
            .border_color(Hsla::from(colors.accent).opacity(0.6))
            .bg(colors.elevated)
            .shadow_md()
            .text_size(px(13.))
            .font_medium()
            .text_color(colors.text)
            .truncate()
            .child(self.label.clone())
    }
}

/// What a drag of `count` notes shows: the one note's title, or the count.
pub(crate) fn drag_label(count: usize, title: &str) -> String {
    match count {
        1 if title.trim().is_empty() => "Untitled".to_owned(),
        1 => title.to_owned(),
        n => format!("{n} notes"),
    }
}

/// What the list's footer says while several notes are selected.
pub(crate) fn selected_label(count: usize) -> Option<String> {
    (count > 1).then(|| format!("{count} notes selected"))
}

/// The rows for one list of notes, kept until the library loads another.
#[derive(Default)]
struct ListLayout {
    /// The library's notes version, sort and whether pinned notes group.
    key: Option<(u64, Sort, bool)>,
    grouped: Grouped,
    sizes: Rc<Vec<Size<Pixels>>>,
    /// The zoom `sizes` were measured at.
    scale: f32,
    /// Whether any note has a status, so titles leave room for its glyph.
    status_column: bool,
}

/// A row about to be drawn.
enum Visible {
    Header(String),
    Note(NoteSummary),
}

/// What every visible row needs, worked out once per frame.
struct RowContext {
    now_local_ms: i64,
    today: i64,
    created: bool,
    status_column: bool,
    title_terms: Vec<String>,
    body_terms: Vec<String>,
}

pub struct NoteList {
    library: Entity<Library>,
    pub(crate) search: Entity<InputState>,
    scroll: VirtualListScrollHandle,
    focus: FocusHandle,
    search_task: Option<Task<()>>,
    layout: ListLayout,
    /// Brought into view once the rows for the current list are laid out,
    /// with the library's list version when it was asked for: a note that
    /// is selected as it is made shows only after the next reload.
    scroll_to: Option<(String, u64)>,
    /// The note under the pointer. Hover is kept as state rather than a
    /// style so a snapshot can show it.
    pub(crate) hovered: Option<String>,
    /// The note whose row has a press inside it held down. GPUI calls a row
    /// unhovered while a press inside it is held and the pointer moves, so
    /// without this its actions went away under a slightly wobbly click and
    /// the release selected the note instead of pinning or trashing it.
    /// The release settles the hover.
    pressed: Option<String>,
    /// Room kept at the left of the top band, for the window's traffic
    /// lights when the sidebar is hidden.
    leading_inset: Pixels,
    /// Whether the list has the keyboard, as of the last frame: its
    /// selected row then shows a focus ring.
    list_focused: bool,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<NoteListEvent> for NoteList {}

impl Focusable for NoteList {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl NoteList {
    pub fn new(library: Entity<Library>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Search notes…")
                .clean_on_escape()
        });
        let subscriptions = vec![
            cx.observe(&library, |_, _, cx| cx.notify()),
            // Invitations and mentions, over Shared with me.
            cx.observe_global::<crate::collab_model::Collab>(|this, cx| {
                if this.library.read(cx).filter == Filter::Shared {
                    cx.notify();
                }
            }),
            // Follow the selection when it moves, but not on every reload,
            // which would pull the list back while it's being scrolled.
            cx.subscribe(&library, |this, library, event, cx| {
                if let LibraryEvent::Selected { id: Some(id), .. } = event {
                    this.scroll_to = Some((id.clone(), library.read(cx).notes_version));
                    cx.notify();
                }
            }),
            cx.subscribe_in(&search, window, |this, search, event, _, cx| match event {
                InputEvent::Change => {
                    let text = search.read(cx).value().to_string();
                    let library = this.library.clone();
                    this.search_task = Some(cx.spawn(async move |_, cx| {
                        cx.background_executor().timer(SEARCH_DELAY).await;
                        library.update(cx, |library, cx| library.set_search(text, cx));
                    }));
                    cx.notify();
                }
                InputEvent::PressEnter { .. } => cx.emit(NoteListEvent::OpenSelected),
                // The syntax hint shows while the empty box has focus.
                InputEvent::Focus | InputEvent::Blur => cx.notify(),
            }),
        ];
        NoteList {
            library,
            search,
            scroll: VirtualListScrollHandle::new(),
            focus: cx.focus_handle(),
            search_task: None,
            layout: ListLayout::default(),
            scroll_to: None,
            hovered: None,
            list_focused: false,
            pressed: None,
            leading_inset: px(0.),
            _subscriptions: subscriptions,
        }
    }

    pub fn focus_search(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.search
            .update(cx, |search, cx| search.focus(window, cx));
    }

    pub fn focus_list(&self, window: &mut Window, cx: &mut App) {
        window.focus(&self.focus, cx);
    }

    /// Keeps `inset` free at the left of the top band, for the traffic
    /// lights when nothing else is to the list's left.
    pub fn set_leading_inset(&mut self, inset: Pixels, cx: &mut Context<Self>) {
        if self.leading_inset != inset {
            self.leading_inset = inset;
            cx.notify();
        }
    }

    /// Lays the notes out again when the library has loaded a new list.
    fn update_layout(&mut self, cx: &App) {
        let library = self.library.read(cx);
        // Recent is always newest first by edit time, whatever the sort, and
        // pinned notes stay among the days. A list of pinned notes only
        // groups by date too.
        let shape = list::shape(&library.filter, library.pinned_only, library.sort);
        let (sort, group_pinned) = (shape.sort, shape.group_pinned);
        let key = (library.notes_version, sort, group_pinned);
        if self.layout.key == Some(key) {
            return;
        }
        let created = shape.date == list::DateKey::Created;
        let grouped = group_rows(
            library.notes.iter().map(|note| RowKey {
                pinned: group_pinned && note.pinned,
                at_ms: if created {
                    note.created_at
                } else {
                    note.updated_at
                },
            }),
            |at| day_of(Clock::local_ms(cx, at)),
            shape.by_date,
        );
        self.layout = ListLayout {
            key: Some(key),
            sizes: Rc::new(item_sizes(&grouped.rows)),
            scale: crate::zoom::scale(),
            grouped,
            status_column: library.notes.iter().any(|n| n.status != NoteStatus::None),
        };
        // A note that left the list (trashed from its hover button, say)
        // leaves nothing hovered; the row now under the pointer picks it up.
        let listed = |id: &Option<String>| {
            id.as_ref()
                .is_none_or(|id| library.notes.iter().any(|n| &n.id == id))
        };
        if !listed(&self.hovered) {
            self.hovered = None;
        }
        if !listed(&self.pressed) {
            self.pressed = None;
        }
    }

    /// How long laying out the current list takes, for the snapshot
    /// harness to report on a real library.
    #[cfg(test)]
    pub(crate) fn time_layout(&mut self, cx: &App) -> (usize, Duration) {
        self.layout.key = None;
        let start = std::time::Instant::now();
        self.update_layout(cx);
        (self.layout.grouped.rows.len(), start.elapsed())
    }

    /// How far the list is scrolled.
    #[cfg(test)]
    pub(crate) fn scroll_offset(&self) -> Point<Pixels> {
        self.scroll.base_handle().offset()
    }

    #[cfg(test)]
    pub(crate) fn set_scroll_offset(&self, offset: Point<Pixels>) {
        self.scroll.base_handle().set_offset(offset);
    }

    /// Scrolls the note asked for into view, with its date header when it
    /// is the first under one and the list moves up to it.
    fn scroll_to_pending(&mut self, cx: &App) {
        let Some((id, asked_at)) = self.scroll_to.clone() else {
            return;
        };
        let library = self.library.read(cx);
        let row = library
            .notes
            .iter()
            .position(|n| n.id == id)
            .and_then(|ix| self.layout.grouped.note_row(ix));
        match pending_scroll(row.is_some(), asked_at, library.notes_version) {
            PendingScroll::Scroll => self.scroll_to = None,
            PendingScroll::Wait => return,
            PendingScroll::Drop => {
                self.scroll_to = None;
                return;
            }
        }
        let Some(row) = row else {
            return;
        };
        let header_above = row
            .checked_sub(1)
            .filter(|&above| matches!(self.layout.grouped.rows[above], Row::Header(_)));
        let target = match header_above {
            Some(header) => {
                let top: f32 = self.layout.sizes[..header]
                    .iter()
                    .map(|s| f32::from(s.height))
                    .sum();
                if top + f32::from(self.scroll.offset().y) < 0. {
                    header
                } else {
                    row
                }
            }
            None => row,
        };
        self.scroll.scroll_to_item(target, ScrollStrategy::Nearest);
    }

    fn row_context(&self, cx: &App) -> RowContext {
        let library = self.library.read(cx);
        let now_local_ms = Clock::local_ms(cx, Clock::now_ms(cx));
        let query = Query::parse(&library.search);
        RowContext {
            now_local_ms,
            today: day_of(now_local_ms),
            created: library.filter != Filter::Recent && library.sort == Sort::CreatedAt,
            status_column: self.layout.status_column,
            title_terms: highlight_terms(&query, Field::Title),
            body_terms: highlight_terms(&query, Field::Body),
        }
    }

    fn render_rows(
        &mut self,
        range: std::ops::Range<usize>,
        cx: &mut Context<Self>,
    ) -> Vec<AnyElement> {
        let context = self.row_context(cx);
        let library = self.library.read(cx);
        let focus = library.selected.clone();
        // Worked out once for all the rows, which may be every note.
        let several = library.selection.is_several().then(|| {
            let ids: Rc<[String]> = library.selection.ids().into();
            let selected: std::collections::HashSet<&str> =
                ids.iter().map(String::as_str).collect();
            let all_pinned = library
                .notes
                .iter()
                .filter(|n| selected.contains(n.id.as_str()))
                .all(|n| n.pinned);
            (ids, all_pinned)
        });
        // Copied out first: drawing a row needs the context mutably.
        let rows: Vec<Visible> = range
            .filter_map(|ix| {
                Some(match *self.layout.grouped.rows.get(ix)? {
                    Row::Header(bucket) => Visible::Header(bucket_label(bucket, context.today)),
                    Row::Note(note) => Visible::Note(library.notes.get(note)?.clone()),
                })
            })
            .collect();
        rows.into_iter()
            .map(|row| match row {
                Visible::Header(label) => render_header(label, cx).into_any_element(),
                Visible::Note(note) => {
                    let is_focus = focus.as_deref() == Some(note.id.as_str());
                    let among = several.clone().filter(|(ids, _)| ids.contains(&note.id));
                    let is_selected = is_focus || among.is_some();
                    self.render_row(&note, is_selected, is_focus, among, &context, cx)
                        .into_any_element()
                }
            })
            .collect()
    }

    fn render_row(
        &self,
        note: &NoteSummary,
        selected: bool,
        focus: bool,
        several: Option<(Rc<[String]>, bool)>,
        context: &RowContext,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let colors = Colors::get(cx);
        // A row among several selected acts for all of them: its menu and a
        // drag from it take the lot. The flag is whether all are pinned.
        let (several, all_pinned) = match several {
            Some((ids, all_pinned)) => (Some(ids), all_pinned),
            None => (None, false),
        };
        let dragged = match &several {
            Some(ids) => DraggedNotes {
                ids: ids.clone(),
                label: drag_label(ids.len(), "").into(),
            },
            None => DraggedNotes {
                ids: Rc::from([note.id.clone()]),
                label: drag_label(1, &note.title).into(),
            },
        };
        let hovered = self.hovered.as_deref() == Some(note.id.as_str());
        let id = note.id.clone();
        let library = self.library.clone();
        let list_focus = self.focus.clone();
        let in_trash = note.book_id == TRASH_BOOK_ID;
        let view = cx.entity();
        let menu_id = note.id.clone();
        let pinned = note.pinned;
        let locked = note.locked;
        let status = note.status;
        let at = if context.created {
            note.created_at
        } else {
            note.updated_at
        };
        let time = row_time_label(Clock::local_ms(cx, at), context.now_local_ms);
        let highlight = HighlightStyle {
            background_color: Some(Hsla::from(colors.accent).opacity(0.32)),
            color: Some(colors.text.into()),
            ..Default::default()
        };
        let untitled = note.title.trim().is_empty();
        let title: SharedString = if untitled {
            "Untitled".into()
        } else {
            note.title.clone().into()
        };
        // "Untitled" stands in for a title; the search never matched it.
        let title_matches = if untitled {
            Vec::new()
        } else {
            match_ranges(&title, &context.title_terms)
        };
        let excerpt_matches = match_ranges(&note.snippet, &context.body_terms);
        let hover_id = note.id.clone();
        let press_id = note.id.clone();
        let release_id = note.id.clone();

        // Dropped is as quiet as the row's time.
        let glyph = status_glyph(note.status, colors, colors.quiet(selected))
            .map(|(icon, tint)| Icon::new(icon).size(px(12.)).text_color(tint));
        let tasks = task_pill(note.tasks_done, note.tasks_total).map(|label| {
            let done = note.tasks_done >= note.tasks_total;
            div()
                .flex_shrink_0()
                .px(px(5.))
                .h(px(16.))
                .flex()
                .items_center()
                .rounded(px(8.))
                .border_1()
                .border_color(if done {
                    Hsla::from(colors.success).opacity(0.45)
                } else {
                    Hsla::from(colors.divider)
                })
                .text_size(px(11.))
                .line_height(px(14.))
                .text_color(if done {
                    colors.success
                } else {
                    colors.quiet(selected)
                })
                .debug_selector(|| "task-pill".into())
                .child(label)
        });
        let trailing = if hovered {
            self.render_row_actions(note, in_trash, cx)
                .into_any_element()
        } else {
            h_flex()
                .flex_shrink_0()
                .gap_1()
                .text_size(px(12.))
                .text_color(colors.quiet(selected))
                .children(tasks)
                .when(pinned, |el| {
                    el.child(library_icon(Slot::NotePinned).size(px(12.)))
                })
                .child(time)
                .into_any_element()
        };

        div()
            // Keyed by the note, so the pointer's hover belongs to the note
            // under it, not to whichever note last had this row.
            .id(SharedString::from(format!("note-{}", note.id)))
            .debug_selector({
                let id = note.id.clone();
                move || format!("note-{id}")
            })
            .h(px(ROW_HEIGHT))
            .w_full()
            .px(px(ROW_INSET_X))
            .py(px(ROW_INSET_Y))
            .child(
                v_flex()
                    .size_full()
                    // The ring's width comes out of the padding, so text
                    // stays put with and without it.
                    .px(px(ROW_PAD_X - 1.))
                    .py(px(ROW_PAD_Y - 1.))
                    .gap(px(ROW_GAP))
                    .rounded(px(6.))
                    .border_1()
                    .border_color(if focus && self.list_focused {
                        Hsla::from(colors.accent)
                    } else {
                        gpui_kit::transparent_black()
                    })
                    .when(selected, |el| el.bg(colors.row_active))
                    .when(hovered && !selected, |el| el.bg(colors.row_hover))
                    .child(
                        h_flex()
                            .h(px(TITLE_LINE))
                            .gap(px(GLYPH_GAP))
                            .when(context.status_column, |el| {
                                el.child(
                                    div()
                                        .w(px(GLYPH_COLUMN))
                                        .flex_shrink_0()
                                        .flex()
                                        .justify_center()
                                        .children(glyph),
                                )
                            })
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .truncate()
                                    .text_size(px(13.5))
                                    .line_height(px(TITLE_LINE))
                                    .font_semibold()
                                    .text_color(if untitled {
                                        colors.quiet(selected)
                                    } else {
                                        colors.text
                                    })
                                    .child(StyledText::new(title).with_highlights(
                                        title_matches.into_iter().map(|r| (r, highlight)),
                                    )),
                            )
                            .child(trailing),
                    )
                    .child(
                        div()
                            .h(px(EXCERPT_LINE * EXCERPT_LINES))
                            .when(context.status_column, |el| {
                                el.pl(px(GLYPH_COLUMN + GLYPH_GAP))
                            })
                            .text_size(px(12.5))
                            .line_height(px(EXCERPT_LINE))
                            .text_color(colors.text_secondary)
                            .line_clamp(EXCERPT_LINES as usize)
                            .text_ellipsis()
                            // A locked note's text isn't in the list.
                            .when(note.locked, |el| {
                                let (mark, word) = crate::locked::note_mark(cx);
                                el.child(
                                    h_flex()
                                        .gap_1()
                                        .text_color(colors.text_muted)
                                        .child(mark.size(px(12.)).text_color(colors.text_muted))
                                        .child(word),
                                )
                            })
                            .when(!note.locked, |el| {
                                el.child(StyledText::new(note.snippet.clone()).with_highlights(
                                    excerpt_matches.into_iter().map(|r| (r, highlight)),
                                ))
                            }),
                    ),
            )
            .cursor_pointer()
            .on_hover(cx.listener(move |this, hovering: &bool, _, cx| {
                if *hovering {
                    // Hovering any row also ends a press whose release never
                    // arrived, say when the window lost focus mid-press.
                    this.pressed = None;
                    this.hovered = Some(hover_id.clone());
                } else if this.pressed.as_deref() == Some(hover_id.as_str()) {
                    // Only the press hides the pointer; its release decides.
                    return;
                } else if this.hovered.as_deref() == Some(hover_id.as_str()) {
                    this.hovered = None;
                } else {
                    return;
                }
                cx.notify();
            }))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _: &MouseDownEvent, _, _| {
                    this.pressed = Some(press_id.clone());
                }),
            )
            // A middle click opens the note in a new tab, as a browser does.
            // It is its own button: the left-click handler below doesn't run.
            .on_mouse_down(MouseButton::Middle, {
                let view = view.clone();
                let id = id.clone();
                let focus = list_focus.clone();
                move |_: &MouseDownEvent, window, cx| {
                    window.focus(&focus, cx);
                    view.update(cx, |_, cx| {
                        cx.emit(NoteListEvent::Command(
                            id.clone(),
                            NoteCommand::OpenInNewTab,
                        ))
                    });
                }
            })
            // Both on the way in, before a hover button's click stops the
            // release going further: inside, the row stays hovered; outside,
            // the pointer has left it.
            .capture_any_mouse_up(cx.listener(|this, _: &MouseUpEvent, _, _| {
                this.pressed = None;
            }))
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(move |this, _: &MouseUpEvent, _, cx| {
                    if this.pressed.as_deref() == Some(release_id.as_str()) {
                        this.pressed = None;
                        if this.hovered.as_deref() == Some(release_id.as_str()) {
                            this.hovered = None;
                        }
                        cx.notify();
                    }
                }),
            )
            .on_drag(dragged, |notes, _, _, cx| cx.new(|_| notes.clone()))
            .on_click({
                let view = view.clone();
                move |event, window, cx| {
                    // ⌥-click: the note in a window of its own. (⌘ and ⇧
                    // are for choosing several rows.)
                    if event.modifiers().alt {
                        let id = id.clone();
                        view.update(cx, |_, cx| {
                            cx.emit(NoteListEvent::Command(id, NoteCommand::OpenInNewWindow))
                        });
                        return;
                    }
                    window.focus(&list_focus, cx);
                    let how = click_kind(event.modifiers());
                    library.update(cx, |library, cx| library.click(&id, how, cx));
                }
            })
            .context_menu(move |menu, window, cx| {
                let emit = |event: fn(String) -> NoteListEvent| {
                    let view = view.clone();
                    let id = menu_id.clone();
                    move |_: &ClickEvent, _: &mut Window, cx: &mut App| {
                        view.update(cx, |_, cx| cx.emit(event(id.clone())));
                    }
                };
                if let Some(ids) = several.clone().filter(|_| !in_trash) {
                    let view = view.clone();
                    let count = ids.len();
                    let run: crate::note_menu::Run = Rc::new(move |command, _, cx| {
                        let ids = ids.to_vec();
                        view.update(cx, |_, cx| {
                            cx.emit(NoteListEvent::CommandSeveral(ids, command))
                        });
                    });
                    crate::note_menu::several_items(menu, count, all_pinned, run, window, cx)
                } else if in_trash {
                    menu.item(PopupMenuItem::new("Restore").on_click(emit(NoteListEvent::Restore)))
                        .separator()
                        .item(
                            PopupMenuItem::new("Delete Permanently…")
                                .on_click(emit(NoteListEvent::DeleteForever)),
                        )
                } else {
                    let view = view.clone();
                    let id = menu_id.clone();
                    let run: crate::note_menu::Run = Rc::new(move |command, _, cx| {
                        view.update(cx, |_, cx| {
                            cx.emit(NoteListEvent::Command(id.clone(), command))
                        });
                    });
                    crate::note_menu::items(menu, pinned, locked, status, run, window, cx)
                }
            })
    }

    /// What hovering a row reveals in place of its time: pin and trash, or
    /// restore for a note in the trash.
    fn render_row_actions(
        &self,
        note: &NoteSummary,
        in_trash: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let colors = Colors::get(cx);
        let action = |name: &'static str,
                      icon: Icon,
                      tooltip: &'static str,
                      event: fn(String) -> NoteListEvent,
                      cx: &mut Context<Self>| {
            let id = note.id.clone();
            Button::new(name)
                .ghost()
                .xsmall()
                .icon(icon.size(px(13.)).text_color(colors.text_secondary))
                .tooltip(tooltip)
                .debug_selector(|| name.into())
                .on_click(cx.listener(move |_, _, _, cx| {
                    cx.stop_propagation();
                    cx.emit(event(id.clone()));
                }))
        };
        h_flex().flex_shrink_0().gap_0p5().map(|el| {
            if in_trash {
                el.child(action(
                    "row-restore",
                    Icon::new(Lucide::RotateCw),
                    "Restore",
                    NoteListEvent::Restore,
                    cx,
                ))
            } else {
                el.child(action(
                    "row-pin",
                    Icon::new(if note.pinned {
                        Lucide::PinOff
                    } else {
                        Lucide::Pin
                    }),
                    if note.pinned {
                        "Unpin note"
                    } else {
                        "Pin note"
                    },
                    |id| NoteListEvent::Command(id, NoteCommand::TogglePinned),
                    cx,
                ))
                .child(action(
                    "row-trash",
                    Icon::new(Lucide::Trash),
                    "Move to trash",
                    |id| NoteListEvent::Command(id, NoteCommand::Trash),
                    cx,
                ))
            }
        })
    }

    /// Notebook and filter menu, at the height of every pane's top band.
    fn render_top_band(&self, cx: &App) -> impl IntoElement {
        let colors = Colors::get(cx);
        let library = self.library.read(cx);
        let title = library.filter_title();
        let count = library.notes.len();
        let refined = library.is_refined();
        let narrowing = narrowing_label(library.status_only, library.pinned_only);
        let switch_library = self.library.clone();
        let filter_library = self.library.clone();

        let switcher = Button::new("list-notebook")
            .ghost()
            // Buttons don't shrink by default; a long name truncates.
            .flex_shrink(1.)
            .min_w_0()
            .px_1p5()
            .child(
                div()
                    .min_w_0()
                    .truncate()
                    .text_size(px(15.))
                    .font_semibold()
                    .text_color(colors.text)
                    .child(title),
            )
            .child(
                Icon::new(IconName::ChevronDown)
                    .size(px(14.))
                    .text_color(colors.text_muted),
            )
            .dropdown_menu(move |menu, _, cx| notebook_menu(menu, &switch_library, cx));

        let filter_menu = Button::new("list-filter")
            .ghost()
            .small()
            .icon(
                Icon::new(Lucide::ListFilter)
                    .size(px(15.))
                    .text_color(colors.text_secondary),
            )
            .selected(refined)
            .tooltip("Filter and sort")
            .dropdown_menu_with_anchor(Anchor::TopRight, move |menu, window, cx| {
                filter_menu(menu, &filter_library, window, cx)
            });

        h_flex()
            .h(px(TOP_BAND_HEIGHT))
            .flex_shrink_0()
            .pl(px(10.) + self.leading_inset)
            .pr(px(10.))
            .gap_1()
            .child(switcher)
            .child(
                div()
                    .flex_shrink_0()
                    .text_size(px(12.))
                    .text_color(colors.text_muted)
                    .child(count.to_string()),
            )
            // What the filter menu narrowed the list to, so a short list
            // doesn't read as the whole notebook.
            .when_some(narrowing, |el, narrowing| {
                el.child(
                    div()
                        .min_w_0()
                        .flex_shrink(1000.)
                        .truncate()
                        .text_size(px(12.))
                        .text_color(colors.text_muted)
                        .child(narrowing),
                )
            })
            .child(div().flex_1())
            // The trash has no board: its notes have no status to change.
            .when(library.filter != Filter::Trash, |el| {
                el.child(
                    Button::new("list-board")
                        .ghost()
                        .small()
                        .icon(
                            library_icon(Slot::Board)
                                .size(px(15.))
                                .text_color(colors.text_secondary),
                        )
                        .tooltip("Board")
                        .on_click(|_, window, cx| {
                            window.dispatch_action(Box::new(crate::board_view::ShowBoard), cx)
                        }),
                )
            })
            .child(filter_menu)
    }

    fn render_search(&self, window: &Window, cx: &App) -> impl IntoElement {
        let colors = Colors::get(cx);
        let empty = self.search.read(cx).value().is_empty();
        let focused = self.search.read(cx).focus_handle(cx).is_focused(window);
        // Global search's key, the same in every place: it brings the
        // keyboard here over all notes.
        let key = crate::keys::shortcut_chip(&SearchAllNotes, window, cx);
        v_flex()
            .flex_shrink_0()
            .px(px(12.))
            .pb(px(8.))
            .gap(px(8.))
            .child(
                Input::new(&self.search)
                    .cleanable(true)
                    .prefix(
                        Icon::new(IconName::Search)
                            .size(px(14.))
                            .text_color(colors.text_muted),
                    )
                    // The shortcut as bound, while there is room for it.
                    .when_some(key.filter(|_| empty), |input, key| input.suffix(key)),
            )
            .when(focused && empty, |el| {
                el.child(div().pl(px(2.)).child(query_tokens(colors, cx)))
            })
    }

    fn render_bottom_band(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let library = self.library.read(cx);
        let in_trash = library.filter == Filter::Trash;
        let sortable = sorts(&library.filter);
        let sort_library = self.library.clone();
        let several = selected_label(library.selection.count());
        h_flex()
            .h(px(BOTTOM_BAND_HEIGHT))
            .flex_shrink_0()
            .px(px(8.))
            .gap_1()
            .border_t_1()
            .border_color(colors.divider)
            .when(!in_trash, |el| {
                el.child(
                    Button::new("list-new-note")
                        .ghost()
                        .small()
                        .icon(IconName::Plus)
                        .label("New note")
                        .text_color(colors.text_secondary)
                        .tooltip_with_action("New note", &crate::workspace::NewNote, None)
                        .on_click(cx.listener(|_, _, _, cx| cx.emit(NoteListEvent::NewNote))),
                )
                // The other ways to start a note.
                .child(
                    Button::new("list-new-more")
                        .ghost()
                        .xsmall()
                        .icon(
                            Icon::new(IconName::ChevronDown)
                                .size(px(13.))
                                .text_color(colors.text_muted),
                        )
                        .tooltip("More ways to start a note")
                        .dropdown_menu_with_anchor(Anchor::BottomLeft, |menu, _, _| {
                            menu.menu(
                                "New from Template…",
                                Box::new(crate::journal::NewFromTemplate),
                            )
                            .menu("Today's Note", Box::new(crate::journal::OpenToday))
                            .menu(
                                "Journal Calendar…",
                                Box::new(crate::calendar_view::ShowCalendar),
                            )
                            .menu("Quick Note", Box::new(crate::journal::QuickCapture))
                            .separator()
                            .menu("New Template", Box::new(crate::journal::NewTemplate))
                        }),
                )
            })
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_size(px(12.))
                    .text_color(colors.text_muted)
                    .debug_selector(|| "list-selected-count".into())
                    .children(several),
            )
            .when(sortable, |el| {
                el.child(
                    Button::new("list-sort")
                        .ghost()
                        .small()
                        .icon(
                            Icon::new(Lucide::ArrowUpDown)
                                .size(px(15.))
                                .text_color(colors.text_secondary),
                        )
                        .tooltip("Sort")
                        .dropdown_menu_with_anchor(Anchor::BottomRight, move |menu, _, cx| {
                            sort_items(menu, &sort_library, cx)
                        }),
                )
            })
    }

    /// What the list says when it has no notes to show.
    fn render_empty(&self, cx: &App) -> AnyElement {
        let colors = Colors::get(cx);
        let library = self.library.read(cx);
        let search = library.search.trim().to_owned();
        if !search.is_empty() {
            return quiet_empty(
                IconName::Search,
                "No matches".into(),
                Some(format!("No notes match “{search}”.")),
                colors,
            )
            .child(
                div()
                    .mt(px(18.))
                    .text_size(px(11.5))
                    .text_color(colors.text_muted)
                    .child("Narrow a search with"),
            )
            .child(div().mt(px(6.)).child(query_tokens(colors, cx)))
            .into_any_element();
        }
        if library.is_refined() {
            let library = self.library.clone();
            return quiet_empty(
                Lucide::ListFilter,
                "No notes match these filters".into(),
                None,
                colors,
            )
            .child(
                Button::new("clear-refinements")
                    .ghost()
                    .small()
                    .mt(px(10.))
                    .label("Clear filters")
                    .on_click(move |_, _, cx| {
                        library.update(cx, |library, cx| library.clear_refinements(cx));
                    }),
            )
            .into_any_element();
        }
        match &library.filter {
            // Its notes are a level or more down: not a notebook to start.
            Filter::Book(id) if library.has_notes_inside(id) => quiet_empty(
                library_icon(Slot::Notebook),
                format!("No notes directly in {}", library.filter_title()),
                Some("Its notes are in the notebooks inside it.".into()),
                colors,
            )
            .into_any_element(),
            // The editor beside it shows the full block with the actions;
            // a second icon and sentence here would only repeat it.
            Filter::All | Filter::Book(_) => v_flex()
                .size_full()
                .items_center()
                .justify_center()
                .pb(px(TOP_BAND_HEIGHT))
                .text_size(px(12.5))
                .text_color(colors.text_muted)
                .child("No notes")
                .into_any_element(),
            Filter::Pinned => quiet_empty(
                library_icon(Slot::Pinned),
                "No pinned notes".into(),
                Some("Pin a note to keep it at the top.".into()),
                colors,
            )
            .into_any_element(),
            Filter::Recent => quiet_empty(
                library_icon(Slot::Recent),
                "No recent notes".into(),
                Some("Notes edited in the last 7 days show here.".into()),
                colors,
            )
            .into_any_element(),
            Filter::Tag(_) | Filter::TagPath(_) => quiet_empty(
                library_icon(Slot::Tag),
                format!("No notes tagged {}", library.filter_title()),
                None,
                colors,
            )
            .into_any_element(),
            Filter::Status(status) => quiet_empty(
                Lucide::CircleDot,
                format!("No notes marked {}", status.label()),
                None,
                colors,
            )
            .into_any_element(),
            Filter::Trash => quiet_empty(
                library_icon(Slot::Trash),
                "Trash is empty".into(),
                None,
                colors,
            )
            .into_any_element(),
            // The Tasks view shows in the list's place.
            Filter::Tasks => div().into_any_element(),
            Filter::Shared => quiet_empty(
                library_icon(Slot::Shared),
                "Nothing shared with you yet".into(),
                Some(
                    "Notes other people share with you show here. Invitations wait above them."
                        .into(),
                ),
                colors,
            )
            .into_any_element(),
        }
    }
}

/// The pill a row shows for its task list: "3/7", or nothing for a note
/// without tasks. Pure.
pub(crate) fn task_pill(done: u32, total: u32) -> Option<String> {
    (total > 0).then(|| format!("{}/{total}", done.min(total)))
}

/// Each row's height for the virtual list (only heights are read).
fn item_sizes(rows: &[Row]) -> Vec<Size<Pixels>> {
    rows.iter()
        .map(|row| {
            let height = match row {
                Row::Header(_) => HEADER_HEIGHT,
                Row::Note(_) => ROW_HEIGHT,
            };
            size(px(0.), px(height))
        })
        .collect()
}

/// Whether the list can be sorted: Recent is always newest first.
// no-prop-test: one comparison; `the_list_menus_build` covers Recent.
/// What a click with `modifiers` does: ⇧ selects a run, ⌘ (Ctrl on Windows
/// and Linux) adds or takes away one note.
fn click_kind(modifiers: Modifiers) -> Click {
    if modifiers.shift {
        Click::Range
    } else if modifiers.secondary() {
        Click::Toggle
    } else {
        Click::Plain
    }
}

fn sorts(filter: &Filter) -> bool {
    *filter != Filter::Recent
}

/// What the filter menu narrowed the list to, after its count: "· On hold",
/// "· Pinned" or both.
// no-prop-test: a finite domain; `narrowing_labels` checks every case.
fn narrowing_label(status: Option<NoteStatus>, pinned: bool) -> Option<String> {
    let mut label = String::new();
    if let Some(status) = status {
        label.push_str(" · ");
        label.push_str(status.label());
    }
    if pinned {
        label.push_str(" · Pinned");
    }
    (!label.is_empty()).then(|| label.trim_start().to_owned())
}

/// What becomes of a request to bring a note into view.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PendingScroll {
    /// The note is listed: scroll to it.
    Scroll,
    /// Not listed yet; a reload may still bring it.
    Wait,
    /// Two newer lists came without it: it isn't coming.
    Drop,
}

/// Whether to scroll to a note asked for at list version `asked_at`, now
/// that the list is at `version` and has the note or not. A note selected
/// as it is made arrives with the reload after next at the latest.
fn pending_scroll(listed: bool, asked_at: u64, version: u64) -> PendingScroll {
    if listed {
        PendingScroll::Scroll
    } else if version > asked_at.saturating_add(1) {
        PendingScroll::Drop
    } else {
        PendingScroll::Wait
    }
}

// no-prop-test: layout only; the snapshots show it.
fn render_header(label: String, cx: &App) -> impl IntoElement {
    let colors = Colors::get(cx);
    v_flex()
        .id(SharedString::from(format!("header-{label}")))
        .h(px(HEADER_HEIGHT))
        .w_full()
        .px(px(ROW_INSET_X + ROW_PAD_X))
        .pt(px(HEADER_PAD_TOP))
        .child(
            section_label(div(), colors)
                .h(px(HEADER_LINE))
                .line_height(px(HEADER_LINE))
                .child(label),
        )
        .child(
            div()
                .mt(px(HEADER_PAD_BOTTOM))
                .h(px(DIVIDER))
                .bg(colors.divider),
        )
}

/// The search language's qualifiers, under the empty search box and after a
/// search that found nothing.
fn query_tokens(colors: &Colors, cx: &App) -> impl IntoElement {
    let mono = cx.theme().mono_font_family.clone();
    h_flex()
        .flex_wrap()
        .gap_1()
        .text_size(px(11.5))
        .line_height(px(18.))
        .children(
            ["book:", "tag:", "status:", "title:", "-word"].map(|token| {
                div()
                    .px_1()
                    .rounded(px(4.))
                    .bg(colors.fill)
                    .font_family(mono.clone())
                    .text_color(colors.text_secondary)
                    .child(token)
            }),
        )
}

/// A short message in the middle of an empty list, as in the design, a
/// little above the centre (by the top band's height) so it sits level with
/// the editor's block beside it.
pub(crate) fn quiet_empty(
    icon: impl Into<Icon>,
    title: String,
    detail: Option<String>,
    colors: &Colors,
) -> Div {
    v_flex()
        .size_full()
        .items_center()
        .justify_center()
        .pb(px(TOP_BAND_HEIGHT))
        .px(px(24.))
        .child(icon.into().size(px(22.)).text_color(colors.text_muted))
        .child(
            div()
                .mt(px(10.))
                .text_size(px(13.5))
                .font_medium()
                .text_color(colors.text_secondary)
                .text_center()
                .child(title),
        )
        .when_some(detail, |el, detail| {
            el.child(
                div()
                    .mt(px(4.))
                    .text_size(px(12.5))
                    .text_color(colors.text_muted)
                    .text_center()
                    .child(detail),
            )
        })
}

/// The title's menu: All Notes, Pinned, Recent and every notebook, to
/// switch to.
fn notebook_menu(menu: PopupMenu, library: &Entity<Library>, cx: &App) -> PopupMenu {
    let (current, books) = {
        let library = library.read(cx);
        let books: Vec<(usize, String, String)> = library
            .book_tree()
            .into_iter()
            .map(|(depth, book)| (depth, book.id.clone(), book.name.clone()))
            .collect();
        (library.filter.clone(), books)
    };
    let item = |label: String, filter: Filter| {
        let library = library.clone();
        let checked = current == filter;
        PopupMenuItem::new(label)
            .checked(checked)
            .on_click(move |_, _, cx| {
                let filter = filter.clone();
                library.update(cx, |library, cx| library.set_filter(filter, cx));
            })
    };
    let mut menu = menu
        .max_h(px(420.))
        .scrollable(true)
        .item(item("All Notes".into(), Filter::All))
        .item(item("Pinned".into(), Filter::Pinned))
        .item(item("Recent".into(), Filter::Recent));
    if !books.is_empty() {
        menu = menu.separator().label("Notebooks");
    }
    for (depth, id, name) in books {
        let indent = "    ".repeat(depth);
        menu = menu.item(item(format!("{indent}{name}"), Filter::Book(id)));
    }
    menu
}

/// Sort options and direction, shared by the filter menu and the sort
/// button.
fn sort_items(mut menu: PopupMenu, library: &Entity<Library>, cx: &App) -> PopupMenu {
    let (sort, descending) = {
        let library = library.read(cx);
        (library.sort, library.descending)
    };
    menu = menu.label("Sort by");
    for (label, option) in [
        ("Date Updated", Sort::UpdatedAt),
        ("Date Created", Sort::CreatedAt),
        ("Title", Sort::Title),
    ] {
        let library = library.clone();
        menu = menu.item(PopupMenuItem::new(label).checked(sort == option).on_click(
            move |_, _, cx| {
                // Dates read newest first, titles A to Z.
                let descending = option != Sort::Title;
                library.update(cx, |library, cx| library.set_sort(option, descending, cx));
            },
        ));
    }
    let (first, second) = if sort == Sort::Title {
        ("A to Z", "Z to A")
    } else {
        ("Newest First", "Oldest First")
    };
    menu = menu.separator();
    // For titles "first" is ascending; for dates it is descending.
    for (label, wants_descending) in [(first, sort != Sort::Title), (second, sort == Sort::Title)] {
        let library = library.clone();
        menu = menu.item(
            PopupMenuItem::new(label)
                .checked(descending == wants_descending)
                .on_click(move |_, _, cx| {
                    library.update(cx, |library, cx| {
                        library.set_sort(sort, wants_descending, cx)
                    });
                }),
        );
    }
    menu
}

/// The header's one menu: sort, a status to narrow to, and pinned only.
fn filter_menu(
    menu: PopupMenu,
    library: &Entity<Library>,
    window: &mut Window,
    cx: &mut Context<PopupMenu>,
) -> PopupMenu {
    let (status_only, pinned_only, filter) = {
        let library = library.read(cx);
        (
            library.status_only,
            library.pinned_only,
            library.filter.clone(),
        )
    };
    let status_library = library.clone();
    // Recent is always newest first by edit time: nothing to sort.
    let menu = if sorts(&filter) {
        sort_items(menu, library, cx).separator()
    } else {
        menu
    };
    let menu = menu.submenu("Status", window, cx, move |mut menu, _, _| {
        let choices = [None]
            .into_iter()
            .chain(NoteStatus::ALL[1..].iter().copied().map(Some));
        for choice in choices {
            let library = status_library.clone();
            let label = choice.map_or("Any Status", |s| s.label());
            menu = menu.item(
                PopupMenuItem::new(label)
                    .checked(status_only == choice)
                    .on_click(move |_, _, cx| {
                        library.update(cx, |library, cx| library.set_status_only(choice, cx));
                    }),
            );
        }
        menu
    });
    if filter == Filter::Pinned {
        return menu;
    }
    let library = library.clone();
    menu.item(
        PopupMenuItem::new("Pinned Only")
            .checked(pinned_only)
            .on_click(move |_, _, cx| {
                library.update(cx, |library, cx| library.set_pinned_only(!pinned_only, cx));
            }),
    )
}

impl Render for NoteList {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.update_layout(cx);
        if self.layout.scale != crate::zoom::scale() {
            // Zoomed: the same rows at another height.
            self.layout.sizes = Rc::new(item_sizes(&self.layout.grouped.rows));
            self.layout.scale = crate::zoom::scale();
        }
        self.scroll_to_pending(cx);
        // Keyboard focus shows on the selected row, so it is clear which
        // pane the arrows move.
        self.list_focused = self.focus.contains_focused(window, cx);
        let library = self.library.read(cx);
        let error = library.load_error.clone();
        let empty = library.notes.is_empty();
        let shared_list = library.filter == Filter::Shared;
        let colors = Colors::get(cx);

        let body = if empty {
            self.render_empty(cx)
        } else {
            v_virtual_list(
                cx.entity(),
                "notes",
                self.layout.sizes.clone(),
                |this, range, _, cx| this.render_rows(range, cx),
            )
            .size_full()
            .track_scroll(&self.scroll)
            .into_any_element()
        };

        v_flex()
            .size_full()
            // The resize handle draws the divider on the right.
            .bg(colors.list)
            .child(self.render_top_band(cx))
            .child(self.render_search(window, cx))
            .when(shared_list, |el| {
                el.children(crate::shared_inbox::render(&self.library, cx))
            })
            .when_some(error, |el, error| {
                el.child(
                    div()
                        .px_3()
                        .py_1()
                        .text_xs()
                        .text_color(cx.theme().danger)
                        .child(error),
                )
            })
            .child(
                // What has focus when the list does: the arrows and j and k
                // move through the notes here.
                div()
                    .flex_1()
                    .min_h_0()
                    .key_context("NoteList")
                    .track_focus(&self.focus)
                    .on_action(cx.listener(|this, _: &SelectNext, _, cx| {
                        this.library
                            .update(cx, |library, cx| library.select_relative(1, cx));
                    }))
                    .on_action(cx.listener(|this, _: &SelectPrevious, _, cx| {
                        this.library
                            .update(cx, |library, cx| library.select_relative(-1, cx));
                    }))
                    .on_action(cx.listener(|this, _: &SelectAllNotes, _, cx| {
                        this.library
                            .update(cx, |library, cx| library.select_all(cx));
                    }))
                    // ⌘↩ opens the focused note in a new tab, leaving the
                    // note showing where it is.
                    .on_action(cx.listener(|this, _: &OpenFocusedInNewTab, _, cx| {
                        if let Some(id) = this.library.read(cx).selected.clone() {
                            cx.emit(NoteListEvent::Command(id, NoteCommand::OpenInNewTab));
                        }
                    }))
                    // Escape gives way (to leave focus mode, say) unless
                    // there are several notes to go back to one from.
                    .on_action(cx.listener(|this, _: &SelectOneNote, _, cx| {
                        if this.library.read(cx).selection.is_several() {
                            this.library
                                .update(cx, |library, cx| library.select_one(cx));
                        } else {
                            cx.propagate();
                        }
                    }))
                    .child(body),
            )
            .when(self.offers_sync(cx), |el| {
                el.child(crate::sync_nudge::render(Colors::get(cx)))
            })
            .child(self.render_bottom_band(cx))
    }
}

impl NoteList {
    /// Whether the foot of the list offers sync (see `sync_nudge`).
    fn offers_sync(&self, cx: &App) -> bool {
        let signed_out = cx
            .try_global::<crate::services::Services>()
            .is_some_and(|services| {
                matches!(
                    services.account_state(),
                    crate::services::AccountState::SignedOut
                )
            });
        let nudges = &cx.global::<crate::workspace::AppState>().config.nudges;
        crate::sync_nudge::offer_sync(
            signed_out,
            nudges.sync_offer_dismissed,
            self.library.read(cx).counts.all,
            nudges.first_seen_ms,
            Clock::now_ms(cx),
        )
    }
}

#[cfg(test)]
mod tests {
    // Not `super::*`: GPUI's own `test` attribute would shadow the built-in
    // one inside `proptest!`.
    use super::{
        DIVIDER, EXCERPT_LINE, EXCERPT_LINES, HEADER_HEIGHT, HEADER_LINE, HEADER_PAD_BOTTOM,
        HEADER_PAD_TOP, PendingScroll, ROW_GAP, ROW_HEIGHT, ROW_INSET_Y, ROW_PAD_Y, TITLE_LINE,
        drag_label, item_sizes, narrowing_label, pending_scroll, selected_label,
    };
    use crate::grouping::{Bucket, Row, RowKey, group_rows};
    use kanso_core::NoteStatus;
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        })]

        /// Invariants: a listed note is scrolled to whenever it shows up;
        /// one that isn't is waited for a reload or two and then given up,
        /// for good.
        #[test]
        fn prop_pending_scrolls_wait_then_give_up(asked_at in 0u64..1_000, later in 0u64..6) {
            let version = asked_at + later;
            prop_assert_eq!(pending_scroll(true, asked_at, version), PendingScroll::Scroll);
            let waiting = pending_scroll(false, asked_at, version);
            prop_assert_eq!(waiting == PendingScroll::Wait, later <= 1);
            if waiting == PendingScroll::Drop {
                prop_assert_eq!(pending_scroll(false, asked_at, version + 1), PendingScroll::Drop);
            }
        }
    }

    /// A new note shows only after the store's next reload, so asking for it
    /// at once mustn't lose the request.
    #[test]
    fn pending_scroll_examples() {
        assert_eq!(pending_scroll(false, 7, 7), PendingScroll::Wait);
        assert_eq!(pending_scroll(false, 7, 8), PendingScroll::Wait);
        assert_eq!(pending_scroll(true, 7, 8), PendingScroll::Scroll);
        assert_eq!(pending_scroll(false, 7, 9), PendingScroll::Drop);
    }

    #[test]
    fn narrowing_labels() {
        assert_eq!(narrowing_label(None, false), None);
        assert_eq!(narrowing_label(None, true).as_deref(), Some("· Pinned"));
        for status in NoteStatus::ALL {
            let alone = narrowing_label(Some(status), false).unwrap();
            assert_eq!(alone, format!("· {}", status.label()));
            let both = narrowing_label(Some(status), true).unwrap();
            assert_eq!(both, format!("· {} · Pinned", status.label()));
        }
    }

    /// A row's and a header's parts fill them exactly, so nothing is
    /// clipped and nothing floats.
    #[test]
    fn rows_and_headers_fit_their_content() {
        assert_eq!(
            2. * ROW_INSET_Y + 2. * ROW_PAD_Y + TITLE_LINE + ROW_GAP + EXCERPT_LINE * EXCERPT_LINES,
            ROW_HEIGHT
        );
        assert_eq!(
            HEADER_PAD_TOP + HEADER_LINE + HEADER_PAD_BOTTOM + DIVIDER,
            HEADER_HEIGHT
        );
        let grouped = group_rows(
            [
                RowKey {
                    pinned: true,
                    at_ms: 0,
                },
                RowKey {
                    pinned: false,
                    at_ms: 0,
                },
            ],
            |_| 0,
            true,
        );
        assert_eq!(
            grouped.rows,
            [
                Row::Header(Bucket::Pinned),
                Row::Note(0),
                Row::Header(Bucket::Day(0)),
                Row::Note(1),
            ]
        );
        let total: f32 = item_sizes(&grouped.rows)
            .iter()
            .map(|s| f32::from(s.height))
            .sum();
        assert_eq!(total, 2. * ROW_HEIGHT + 2. * HEADER_HEIGHT);
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        })]

        /// Invariants: the footer speaks up exactly when several notes are
        /// selected, with their number; a drag shows the one note's title
        /// (or Untitled), and the number of notes for several.
        #[test]
        fn prop_selection_labels_say_how_many(count in 0usize..10_000, title in "[a-z ]{0,12}") {
            let label = selected_label(count);
            prop_assert_eq!(label.is_some(), count > 1);
            if let Some(label) = label {
                prop_assert_eq!(label, format!("{count} notes selected"));
            }
            let dragged = drag_label(count.max(1), &title);
            if count > 1 {
                prop_assert_eq!(dragged, format!("{count} notes"));
            } else if title.trim().is_empty() {
                prop_assert_eq!(dragged, "Untitled");
            } else {
                prop_assert_eq!(dragged, title);
            }
        }
    }

    /// The list in a real window: a workspace over a store with notes on
    /// two days in "Work" and an empty "Research".
    mod in_a_window {
        use std::sync::Arc;

        use gpui_kit::{
            AnyWindowHandle, AppContext as _, Entity, Modifiers, MouseButton, Pixels, Point,
            TestAppContext, VisualTestContext, point, px,
        };
        use kanso_core::DocType;
        use kanso_store::{Origin, Put, Store};
        use serde_json::json;

        use crate::clock::Clock;
        use crate::config::AppConfig;
        use crate::library::{Filter, Library};
        use crate::palette::Mode;
        use crate::services::Services;
        use crate::workspace::{AppState, NewNote, OpenCommandPalette, Workspace};

        const NOW: i64 = 1_779_954_300_000;
        const DAY: i64 = 86_400_000;

        struct Fixture {
            _dir: tempfile::TempDir,
            store: Arc<Store>,
            window: AnyWindowHandle,
            workspace: Entity<Workspace>,
            research: String,
        }

        fn setup(cx: &mut TestAppContext) -> Fixture {
            let dir = tempfile::tempdir().unwrap();
            let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
            let book = |name: &str| {
                store
                    .put(
                        DocType::Book,
                        json!({"name": name}),
                        Put::Upsert,
                        Origin::Ui,
                    )
                    .unwrap()
                    .id
            };
            let work = book("Work");
            let research = book("Research");
            // Newest first: two today, then one yesterday, so a header sits
            // between the second and third.
            for (title, at) in [("a", NOW - 60_000), ("b", NOW - 120_000), ("c", NOW - DAY)] {
                store
                    .put(
                        DocType::Note,
                        json!({"title": title, "body": "", "bookId": work,
                            "createdAt": at, "updatedAt": at}),
                        Put::Upsert,
                        Origin::Ui,
                    )
                    .unwrap();
            }
            let data_dir = dir.path().to_path_buf();
            let app_store = store.clone();
            let (window, workspace) = cx.update(|cx| {
                gpui_kit::init(cx);
                super::super::bind_keys(cx);
                cx.set_global(Clock::fixed(NOW, 0));
                cx.set_global(AppState {
                    store: app_store,
                    data_dir,
                    config: AppConfig::default(),
                });
                cx.set_global(Services::default());
                gpui_kit::open_window(Default::default(), cx, |window, cx| {
                    cx.new(|cx| Workspace::new(window, cx))
                })
                .unwrap()
            });
            cx.run_until_parked();
            Fixture {
                _dir: dir,
                store,
                window,
                workspace,
                research,
            }
        }

        impl Fixture {
            fn library(&self, cx: &mut TestAppContext) -> Entity<Library> {
                cx.read(|cx| self.workspace.read(cx).library.clone())
            }

            fn selected_title(&self, cx: &mut TestAppContext) -> Option<String> {
                let library = self.library(cx);
                cx.read(|cx| {
                    let library = library.read(cx);
                    let id = library.selected.as_ref()?;
                    let note = library.notes.iter().find(|n| &n.id == id)?;
                    Some(note.title.clone())
                })
            }

            /// Draws, so focus and actions find the views' elements.
            fn draw(&self, cx: &mut TestAppContext) {
                cx.run_until_parked();
                self.window
                    .update(cx, |_, window, cx| {
                        window.refresh();
                        window.draw(cx).clear(cx);
                    })
                    .unwrap();
            }
        }

        // no-prop-test: drives one window through keystrokes; the row
        // layout's properties are in grouping.rs.
        #[gpui_kit::test]
        fn j_and_k_step_over_date_headers(cx: &mut TestAppContext) {
            let f = setup(cx);
            let list = cx.read(|cx| f.workspace.read(cx).note_list.clone());
            f.draw(cx);
            f.window
                .update(cx, |_, window, cx| {
                    let focus = list.read(cx).focus.clone();
                    window.focus(&focus, cx);
                })
                .unwrap();
            let library = f.library(cx);
            let first = cx.read(|cx| library.read(cx).notes[0].id.clone());
            library.update(cx, |library, cx| library.select(Some(first), cx));
            f.draw(cx);
            assert_eq!(f.selected_title(cx).as_deref(), Some("a"));
            cx.simulate_keystrokes(f.window, "j j");
            assert_eq!(f.selected_title(cx).as_deref(), Some("c"), "past Yesterday");
            cx.simulate_keystrokes(f.window, "j");
            assert_eq!(
                f.selected_title(cx).as_deref(),
                Some("c"),
                "stops at the end"
            );
            cx.simulate_keystrokes(f.window, "k down up up");
            assert_eq!(f.selected_title(cx).as_deref(), Some("a"));
        }

        #[gpui_kit::test]
        fn j_and_k_type_into_the_search_box(cx: &mut TestAppContext) {
            let f = setup(cx);
            let list = cx.read(|cx| f.workspace.read(cx).note_list.clone());
            f.draw(cx);
            f.window
                .update(cx, |_, window, cx| {
                    list.update(cx, |list, cx| list.focus_search(window, cx))
                })
                .unwrap();
            let before = f.selected_title(cx);
            cx.simulate_keystrokes(f.window, "j k");
            f.draw(cx);
            let typed = cx.read(|cx| list.read(cx).search.read(cx).value().to_string());
            assert_eq!(typed, "jk");
            assert_eq!(f.selected_title(cx), before);
        }

        /// A note made while the list narrows to a status or to pinned
        /// notes gets them, so it shows in the list it was made from.
        #[gpui_kit::test]
        fn a_new_note_keeps_to_the_narrowing(cx: &mut TestAppContext) {
            use kanso_core::NoteStatus;

            let f = setup(cx);
            let library = f.library(cx);
            library.update(cx, |library, cx| {
                library.set_status_only(Some(NoteStatus::OnHold), cx);
                library.set_pinned_only(true, cx);
            });
            f.draw(cx);
            assert!(cx.read(|cx| library.read(cx).notes.is_empty()));
            let focus = cx.read(|cx| f.workspace.read(cx).note_list.read(cx).focus.clone());
            f.window
                .update(cx, |_, window, cx| {
                    focus.dispatch_action(&NewNote, window, cx)
                })
                .unwrap();
            cx.run_until_parked();
            let listed: Vec<(bool, NoteStatus)> = cx.read(|cx| {
                library
                    .read(cx)
                    .notes
                    .iter()
                    .map(|n| (n.pinned, n.status))
                    .collect()
            });
            assert_eq!(listed, [(true, NoteStatus::OnHold)]);
        }

        /// Recent lists by edit time under day headers, whatever the sort,
        /// and a pinned note stays among its day's notes.
        #[gpui_kit::test]
        fn recent_groups_by_day_only(cx: &mut TestAppContext) {
            use crate::grouping::{Bucket, Row};
            use kanso_store::Sort;

            let f = setup(cx);
            let library = f.library(cx);
            let list = cx.read(|cx| f.workspace.read(cx).note_list.clone());
            let pinned = cx.read(|cx| {
                let library = library.read(cx);
                library
                    .notes
                    .iter()
                    .find(|n| n.title == "b")
                    .unwrap()
                    .id
                    .clone()
            });
            f.store
                .put(
                    DocType::Note,
                    json!({"_id": pinned, "pinned": true, "updatedAt": NOW - 120_000}),
                    Put::Merge,
                    Origin::Ui,
                )
                .unwrap();
            library.update(cx, |library, cx| {
                library.set_sort(Sort::Title, false, cx);
                library.set_filter(Filter::Recent, cx);
            });
            f.draw(cx);
            let rows = cx.read(|cx| list.read(cx).layout.grouped.rows.clone());
            let titles: Vec<String> = cx.read(|cx| {
                library
                    .read(cx)
                    .notes
                    .iter()
                    .map(|n| n.title.clone())
                    .collect()
            });
            assert_eq!(titles, ["a", "b", "c"], "newest first");
            assert_eq!(
                rows,
                [
                    Row::Header(Bucket::Day(NOW / DAY)),
                    Row::Note(0),
                    Row::Note(1),
                    Row::Header(Bucket::Day(NOW / DAY - 1)),
                    Row::Note(2),
                ]
            );
        }

        /// The menus are built only when opened, so build each once here.
        #[gpui_kit::test]
        fn the_list_menus_build(cx: &mut TestAppContext) {
            use gpui_kit::component::menu::PopupMenu;

            let f = setup(cx);
            let library = f.library(cx);
            library.update(cx, |library, cx| library.set_pinned_only(true, cx));
            cx.run_until_parked();
            f.window
                .update(cx, |_, window, cx| {
                    let notebooks = PopupMenu::build(window, cx, |menu, _, cx| {
                        super::super::notebook_menu(menu, &library, cx)
                    });
                    let filter = PopupMenu::build(window, cx, |menu, window, cx| {
                        super::super::filter_menu(menu, &library, window, cx)
                    });
                    let sort = PopupMenu::build(window, cx, |menu, _, cx| {
                        super::super::sort_items(menu, &library, cx)
                    });
                    for menu in [notebooks, filter, sort] {
                        assert!(!menu.read(cx).is_empty());
                    }
                })
                .unwrap();
            // Recent has no sort: its menu starts at the status.
            library.update(cx, |library, cx| library.set_filter(Filter::Recent, cx));
            cx.run_until_parked();
            assert!(!super::super::sorts(&Filter::Recent));
            assert!(super::super::sorts(&Filter::All));
            f.window
                .update(cx, |_, window, cx| {
                    let filter = PopupMenu::build(window, cx, |menu, window, cx| {
                        super::super::filter_menu(menu, &library, window, cx)
                    });
                    assert!(!filter.read(cx).is_empty());
                })
                .unwrap();
        }

        /// Right after launch nothing inside has been clicked; the window's
        /// shortcuts still work.
        #[gpui_kit::test]
        fn shortcuts_work_at_launch(cx: &mut TestAppContext) {
            let f = setup(cx);
            cx.update(crate::workspace::bind_keys);
            f.draw(cx);
            let palette = cx.read(|cx| f.workspace.read(cx).palette.clone());
            cx.simulate_keystrokes(f.window, "cmd-k");
            cx.run_until_parked();
            assert_eq!(cx.read(|cx| palette.read(cx).mode()), Some(Mode::Commands));
            // With nothing at all focused, the workspace takes the keyboard
            // back at the next frame.
            cx.simulate_keystrokes(f.window, "escape");
            f.window
                .update(cx, |_, window, cx| window.blur(cx))
                .unwrap();
            f.draw(cx);
            cx.simulate_keystrokes(f.window, "cmd-p");
            cx.run_until_parked();
            assert_eq!(cx.read(|cx| palette.read(cx).mode()), Some(Mode::Notes));
        }

        /// A hovered note that leaves the list, trashed from its own hover
        /// button, leaves nothing hovered: its actions don't stay on the
        /// note that moves up under the pointer.
        #[gpui_kit::test]
        fn hover_leaves_with_its_note(cx: &mut TestAppContext) {
            let f = setup(cx);
            let library = f.library(cx);
            let list = cx.read(|cx| f.workspace.read(cx).note_list.clone());
            let first = cx.read(|cx| library.read(cx).notes[0].id.clone());
            list.update(cx, |list, cx| {
                list.hovered = Some(first.clone());
                cx.notify();
            });
            f.draw(cx);
            assert_eq!(
                cx.read(|cx| list.read(cx).hovered.clone()),
                Some(first.clone())
            );
            f.store
                .put(
                    DocType::Note,
                    json!({"_id": first, "bookId": "trash"}),
                    Put::Merge,
                    Origin::Ui,
                )
                .unwrap();
            f.draw(cx);
            assert_eq!(cx.read(|cx| list.read(cx).hovered.clone()), None);
        }

        /// A click on a row's hover button counts though the pointer moves a
        /// pixel while it is held. GPUI calls the row unhovered then; that
        /// used to take the buttons away, and the release selected the note
        /// instead. A press that ends outside the row leaves it unhovered.
        #[gpui_kit::test]
        fn a_wobbly_click_on_a_row_button_still_counts(cx: &mut TestAppContext) {
            use kanso_core::model::TRASH_BOOK_ID;

            let f = setup(cx);
            f.window
                .update(cx, |_, window, _| window.activate_window())
                .unwrap();
            let library = f.library(cx);
            let list = cx.read(|cx| f.workspace.read(cx).note_list.clone());
            let first_note =
                |cx: &mut TestAppContext| cx.read(|cx| library.read(cx).notes[0].id.clone());
            let state = |cx: &mut TestAppContext| {
                cx.read(|cx| {
                    let list = list.read(cx);
                    (list.hovered.clone(), list.pressed.clone())
                })
            };
            let book = |cx: &mut TestAppContext, id: &str| {
                cx.run_until_parked();
                f.store.note(id).unwrap().unwrap().book_id
            };
            // Shows the first row's buttons to find its trash button, then
            // puts the pointer on it, as moving there would.
            let point_at_trash = |cx: &mut TestAppContext, id: &str| {
                list.update(cx, |list, cx| {
                    list.hovered = Some(id.to_owned());
                    cx.notify();
                });
                f.draw(cx);
                let mut vcx = VisualTestContext::from_window(f.window, cx);
                let trash = vcx
                    .debug_bounds("row-trash")
                    .expect("the hovered row's buttons are drawn")
                    .center();
                vcx.simulate_mouse_move(trash, None, Modifiers::none());
                f.draw(cx);
                trash
            };

            let first = first_note(cx);
            // Another note open, so a click that reached the row would show.
            let other = cx.read(|cx| library.read(cx).notes[1].id.clone());
            library.update(cx, |library, cx| library.select(Some(other.clone()), cx));
            let trash = point_at_trash(cx, &first);
            let mut vcx = VisualTestContext::from_window(f.window, cx);
            vcx.simulate_mouse_down(trash, MouseButton::Left, Modifiers::none());
            let wobble = point(trash.x + px(1.), trash.y);
            vcx.simulate_mouse_move(wobble, MouseButton::Left, Modifiers::none());
            // A frame between the move and the release, as on screen.
            f.draw(cx);
            vcx.simulate_mouse_up(wobble, MouseButton::Left, Modifiers::none());
            assert_eq!(book(cx, &first), TRASH_BOOK_ID, "trashed, not selected");
            assert_eq!(cx.read(|cx| library.read(cx).selected.clone()), Some(other));

            // Pressed, dragged off the row and let go: nothing happens, and
            // the row the pointer left isn't hovered any more.
            f.draw(cx);
            let next = first_note(cx);
            let trash = point_at_trash(cx, &next);
            vcx.simulate_mouse_down(trash, MouseButton::Left, Modifiers::none());
            assert_eq!(state(cx), (Some(next.clone()), Some(next.clone())));
            let away = point(trash.x + px(400.), trash.y);
            vcx.simulate_mouse_move(away, MouseButton::Left, Modifiers::none());
            f.draw(cx);
            assert_eq!(
                state(cx),
                (Some(next.clone()), Some(next.clone())),
                "held: the press keeps the row's buttons"
            );
            vcx.simulate_mouse_up(away, MouseButton::Left, Modifiers::none());
            f.draw(cx);
            assert_ne!(book(cx, &next), TRASH_BOOK_ID);
            assert_eq!(state(cx), (None, None));
        }

        #[gpui_kit::test]
        fn empty_place_cases(cx: &mut TestAppContext) {
            let f = setup(cx);
            let library = f.library(cx);
            let place = |cx: &mut TestAppContext| cx.read(|cx| library.read(cx).empty_place());
            assert_eq!(place(cx), None, "all notes has notes");
            let research = f.research.clone();
            library.update(cx, |library, cx| {
                library.set_filter(Filter::Book(research), cx)
            });
            cx.run_until_parked();
            assert_eq!(place(cx).as_deref(), Some("No notes in Research"));
            // Empty for another reason: a search, a narrowing, another place.
            library.update(cx, |library, cx| library.set_search("x".into(), cx));
            cx.run_until_parked();
            assert_eq!(place(cx), None);
            library.update(cx, |library, cx| {
                library.set_search(String::new(), cx);
                library.set_pinned_only(true, cx);
            });
            cx.run_until_parked();
            assert_eq!(place(cx), None);
            library.update(cx, |library, cx| library.set_filter(Filter::Pinned, cx));
            cx.run_until_parked();
            assert!(cx.read(|cx| library.read(cx).notes.is_empty()));
            assert_eq!(place(cx), None);
            // A notebook of notebooks: none of its own, but the one inside
            // it has notes, so it isn't an empty notebook to start.
            let book = |name: &str, parent: Option<&str>| {
                f.store
                    .put(
                        DocType::Book,
                        json!({"name": name, "parentBookId": parent}),
                        Put::Upsert,
                        Origin::Ui,
                    )
                    .unwrap()
                    .id
            };
            let clients = book("Clients", None);
            let acme = book("Acme", Some(&clients));
            f.store
                .put(
                    DocType::Note,
                    json!({"title": "Kickoff", "bookId": acme}),
                    Put::Upsert,
                    Origin::Ui,
                )
                .unwrap();
            library.update(cx, |library, cx| {
                library.set_filter(Filter::Book(clients.clone()), cx)
            });
            cx.run_until_parked();
            assert!(cx.read(|cx| library.read(cx).notes.is_empty()));
            assert_eq!(place(cx), None);
            assert!(cx.read(|cx| library.read(cx).has_notes_inside(&clients)));
            assert_eq!(cx.read(|cx| library.read(cx).book_total_count(&clients)), 1);
            // Only empty notebooks inside: still empty.
            let projects = book("Projects", None);
            book("Q3", Some(&projects));
            library.update(cx, |library, cx| {
                library.set_filter(Filter::Book(projects), cx)
            });
            cx.run_until_parked();
            assert_eq!(place(cx).as_deref(), Some("No notes in Projects"));
        }

        /// First run is a library with no notes anywhere: not one whose
        /// notes are all in the trash, nor a place or search that is empty.
        #[gpui_kit::test]
        fn first_run_cases(cx: &mut TestAppContext) {
            let f = setup(cx);
            let library = f.library(cx);
            let empty = |cx: &mut TestAppContext| cx.read(|cx| library.read(cx).is_empty_library());
            assert!(!empty(cx), "three notes");
            let research = f.research.clone();
            library.update(cx, |library, cx| {
                library.set_filter(Filter::Book(research), cx);
                library.set_search("nothing matches".into(), cx);
            });
            cx.run_until_parked();
            assert!(cx.read(|cx| library.read(cx).notes.is_empty()));
            assert!(!empty(cx), "an empty notebook and search");
            let params = crate::library::search_params(&Filter::All, "", Default::default(), true);
            let ids = f.store.search_summaries(&params).unwrap();
            assert_eq!(ids.len(), 3);
            for note in &ids {
                f.store
                    .put(
                        DocType::Note,
                        json!({"_id": note.id, "bookId": "trash"}),
                        Put::Merge,
                        Origin::Ui,
                    )
                    .unwrap();
            }
            cx.run_until_parked();
            assert!(!empty(cx), "every note in the trash");
            for note in &ids {
                f.store.delete(&note.id, Origin::Ui).unwrap();
            }
            cx.run_until_parked();
            assert!(empty(cx), "nothing left anywhere");
            // Notebooks alone don't count as notes.
            assert!(cx.read(|cx| !library.read(cx).books.is_empty()));
            f.store
                .put(
                    DocType::Note,
                    json!({"title": "First", "body": "", "bookId": f.research}),
                    Put::Upsert,
                    Origin::Ui,
                )
                .unwrap();
            cx.run_until_parked();
            assert!(!empty(cx), "the first note");
        }

        #[gpui_kit::test]
        fn the_empty_notebook_adds_its_first_note(cx: &mut TestAppContext) {
            let f = setup(cx);
            let library = f.library(cx);
            let research = f.research.clone();
            library.update(cx, |library, cx| {
                library.set_filter(Filter::Book(research.clone()), cx)
            });
            f.draw(cx);
            assert!(cx.read(|cx| library.read(cx).notes.is_empty()));
            // What the editor pane's block sends: its palette row opens the
            // palette, and its New note button adds a note here.
            let focus = cx.read(|cx| f.workspace.read(cx).editor.read(cx).empty_focus.clone());
            let palette = cx.read(|cx| f.workspace.read(cx).palette.clone());
            f.window
                .update(cx, |_, window, cx| {
                    focus.dispatch_action(&OpenCommandPalette, window, cx)
                })
                .unwrap();
            cx.run_until_parked();
            assert_eq!(cx.read(|cx| palette.read(cx).mode()), Some(Mode::Commands));
            f.window
                .update(cx, |_, window, cx| {
                    palette.update(cx, |palette, cx| palette.close(window, cx));
                    focus.dispatch_action(&NewNote, window, cx)
                })
                .unwrap();
            cx.run_until_parked();
            let notes = f.store.search_summaries(&Default::default()).unwrap();
            assert_eq!(notes.iter().filter(|n| n.book_id == research).count(), 1);
            assert_eq!(cx.read(|cx| library.read(cx).notes.len()), 1);
        }

        /// The middle of what is drawn with the debug selector `selector`.
        fn middle_of(f: &Fixture, cx: &mut TestAppContext, selector: &str) -> Point<Pixels> {
            f.draw(cx);
            // Selectors are looked up by `&'static str`.
            let selector: &'static str = Box::leak(selector.to_owned().into_boxed_str());
            let mut vcx = VisualTestContext::from_window(f.window, cx);
            vcx.debug_bounds(selector)
                .unwrap_or_else(|| panic!("{selector} is drawn"))
                .center()
        }

        fn click(f: &Fixture, cx: &mut TestAppContext, id: &str, modifiers: Modifiers) {
            let at = middle_of(f, cx, &format!("note-{id}"));
            VisualTestContext::from_window(f.window, cx).simulate_click(at, modifiers);
            f.draw(cx);
        }

        /// Presses on the row of note `id`, moves to `to` and lets go there,
        /// a frame drawn at each step as on screen.
        fn drag(f: &Fixture, cx: &mut TestAppContext, id: &str, to: Point<Pixels>) -> bool {
            let from = middle_of(f, cx, &format!("note-{id}"));
            let mut vcx = VisualTestContext::from_window(f.window, cx);
            vcx.simulate_mouse_down(from, MouseButton::Left, Modifiers::none());
            let partway = point(from.x + px(30.), from.y + px(4.));
            vcx.simulate_mouse_move(partway, MouseButton::Left, Modifiers::none());
            f.draw(cx);
            let mut vcx = VisualTestContext::from_window(f.window, cx);
            vcx.simulate_mouse_move(to, MouseButton::Left, Modifiers::none());
            f.draw(cx);
            let dragging = cx.read(|cx| cx.has_active_drag());
            let mut vcx = VisualTestContext::from_window(f.window, cx);
            vcx.simulate_mouse_up(to, MouseButton::Left, Modifiers::none());
            f.draw(cx);
            dragging
        }

        fn selection(f: &Fixture, cx: &mut TestAppContext) -> (Vec<String>, Option<String>) {
            let library = f.library(cx);
            cx.read(|cx| {
                let library = library.read(cx);
                let mut ids = library.selection.ids().to_vec();
                ids.sort();
                (ids, library.selected.clone())
            })
        }

        fn listed(f: &Fixture, cx: &mut TestAppContext) -> Vec<String> {
            let library = f.library(cx);
            cx.read(|cx| {
                library
                    .read(cx)
                    .notes
                    .iter()
                    .map(|n| n.id.clone())
                    .collect()
            })
        }

        fn sorted(ids: &[&String]) -> Vec<String> {
            let mut ids: Vec<String> = ids.iter().map(|id| (*id).clone()).collect();
            ids.sort();
            ids
        }

        // no-prop-test: drives the window with clicks and keys; the
        // selection's properties are in selection.rs.
        #[gpui_kit::test]
        fn shift_and_cmd_clicks_select_several_notes(cx: &mut TestAppContext) {
            let f = setup(cx);
            f.window
                .update(cx, |_, window, _| window.activate_window())
                .unwrap();
            let ids = listed(&f, cx);
            let (a, b, c) = (&ids[0], &ids[1], &ids[2]);
            let shift = Modifiers::shift();
            let cmd = Modifiers::secondary_key();

            click(&f, cx, a, Modifiers::none());
            click(&f, cx, c, shift);
            assert_eq!(selection(&f, cx), (sorted(&[a, b, c]), Some(c.clone())));
            let library = f.library(cx);
            let count = cx.read(|cx| library.read(cx).selection.count());
            assert_eq!(
                super::super::selected_label(count).as_deref(),
                Some("3 notes selected")
            );

            click(&f, cx, b, cmd);
            assert_eq!(
                selection(&f, cx),
                (sorted(&[a, c]), Some(c.clone())),
                "⌘ takes b away"
            );
            click(&f, cx, b, cmd);
            assert_eq!(
                selection(&f, cx),
                (sorted(&[a, b, c]), Some(b.clone())),
                "and puts it back"
            );

            // Escape, with the list focused by the clicks: the shown note alone.
            cx.simulate_keystrokes(f.window, "escape");
            f.draw(cx);
            assert_eq!(selection(&f, cx), (vec![b.clone()], Some(b.clone())));
            cx.simulate_keystrokes(f.window, "escape");
            assert_eq!(
                selection(&f, cx),
                (vec![b.clone()], Some(b.clone())),
                "nothing to undo"
            );

            let select_all = if cfg!(target_os = "macos") {
                "cmd-a"
            } else {
                "ctrl-a"
            };
            cx.simulate_keystrokes(f.window, select_all);
            f.draw(cx);
            assert_eq!(selection(&f, cx), (sorted(&[a, b, c]), Some(b.clone())));

            click(&f, cx, a, Modifiers::none());
            assert_eq!(
                selection(&f, cx),
                (vec![a.clone()], Some(a.clone())),
                "a plain click: one"
            );
        }

        // no-prop-test: drives the window; the batch's properties are in
        // note_batch.rs.
        #[gpui_kit::test]
        fn several_notes_change_together_and_undo_together(cx: &mut TestAppContext) {
            use kanso_core::NoteStatus;
            use kanso_core::model::TRASH_BOOK_ID;

            use crate::commands::Picker;
            use crate::workspace::{TogglePinned, TrashNote};

            let f = setup(cx);
            f.window
                .update(cx, |_, window, _| window.activate_window())
                .unwrap();
            let ids = listed(&f, cx);
            let library = f.library(cx);
            let note = |cx: &mut TestAppContext, id: &str| {
                cx.run_until_parked();
                f.store.note(id).unwrap().unwrap()
            };
            click(&f, cx, &ids[0], Modifiers::none());
            click(&f, cx, &ids[1], Modifiers::secondary_key());

            // ⌘⇧P, from the menu bar or the palette, pins both.
            cx.dispatch_action(f.window, TogglePinned);
            assert!(note(cx, &ids[0]).pinned && note(cx, &ids[1]).pinned);
            assert!(!note(cx, &ids[2]).pinned, "the note not selected stays");

            // The palette's status list sets both.
            let palette = cx.read(|cx| f.workspace.read(cx).palette.clone());
            f.window
                .update(cx, |_, window, cx| {
                    palette.update(cx, |palette, cx| {
                        palette.open(Mode::Pick(Picker::Status), window, cx);
                        // In NoteStatus::ALL's order: None, then Active.
                        palette.choose_for_test(1, window, cx);
                    })
                })
                .unwrap();
            assert_eq!(note(cx, &ids[0]).status, NoteStatus::Active);
            assert_eq!(note(cx, &ids[1]).status, NoteStatus::Active);
            assert_eq!(note(cx, &ids[2]).status, NoteStatus::None);

            // Trash both at once; Undo brings both back.
            let work = note(cx, &ids[0]).book_id;
            cx.dispatch_action(f.window, TrashNote);
            assert_eq!(note(cx, &ids[0]).book_id, TRASH_BOOK_ID);
            assert_eq!(note(cx, &ids[1]).book_id, TRASH_BOOK_ID);
            assert_eq!(note(cx, &ids[2]).book_id, work);
            // Once the notice has slid in, which takes real time.
            f.draw(cx);
            std::thread::sleep(std::time::Duration::from_millis(600));
            let undo = middle_of(&f, cx, "undo-notes");
            VisualTestContext::from_window(f.window, cx).simulate_click(undo, Modifiers::none());
            assert_eq!(note(cx, &ids[0]).book_id, work);
            assert_eq!(note(cx, &ids[1]).book_id, work);
            assert_eq!(cx.read(|cx| library.read(cx).notes.len()), 3);
        }

        // no-prop-test: drives the window with drags; the batch's properties
        // are in note_batch.rs.
        #[gpui_kit::test]
        fn notes_dragged_to_the_sidebar_move_or_take_the_tag(cx: &mut TestAppContext) {
            let f = setup(cx);
            f.window
                .update(cx, |_, window, _| window.activate_window())
                .unwrap();
            let tag = f
                .store
                .put(
                    DocType::Tag,
                    json!({"name": "ideas"}),
                    Put::Upsert,
                    Origin::Ui,
                )
                .unwrap()
                .id;
            cx.run_until_parked();
            let ids = listed(&f, cx);
            let (a, b, c) = (&ids[0], &ids[1], &ids[2]);
            let note = |cx: &mut TestAppContext, id: &str| {
                cx.run_until_parked();
                f.store.note(id).unwrap().unwrap()
            };
            let work = note(cx, a).book_id;

            // Two selected, dragged by one of them onto Research: both go.
            click(&f, cx, a, Modifiers::none());
            click(&f, cx, b, Modifiers::secondary_key());
            let research = middle_of(&f, cx, &format!("book-{}", f.research));
            assert!(drag(&f, cx, b, research), "the row drags");
            assert_eq!(note(cx, a).book_id, f.research);
            assert_eq!(note(cx, b).book_id, f.research);
            assert_eq!(note(cx, c).book_id, work, "not selected, not moved");

            // Back to Work, as the real app does it: a run by ⇧-click, one
            // ⌘-clicked out, then a drag from a row still selected takes
            // the selected rows and not the one just clicked out.
            let work_row = middle_of(&f, cx, &format!("book-{work}"));
            click(&f, cx, c, Modifiers::none());
            click(&f, cx, a, Modifiers::shift());
            click(&f, cx, b, Modifiers::secondary_key());
            assert_eq!(selection(&f, cx).0, sorted(&[a, c]));
            assert!(drag(&f, cx, c, work_row));
            assert_eq!(note(cx, a).book_id, work);
            assert_eq!(note(cx, b).book_id, f.research, "clicked out: stays");
            assert_eq!(note(cx, c).book_id, work);
            click(&f, cx, a, Modifiers::none());
            click(&f, cx, b, Modifiers::secondary_key());

            // A note outside the selection drags alone, onto a tag.
            let chip = middle_of(&f, cx, &format!("tag-{tag}"));
            assert!(drag(&f, cx, c, chip));
            assert_eq!(note(cx, c).tags, vec![tag.clone()]);
            assert!(note(cx, a).tags.is_empty() && note(cx, b).tags.is_empty());
            assert_eq!(
                selection(&f, cx).0,
                sorted(&[a, b]),
                "dragging leaves the selection as it was"
            );

            // Let go somewhere that takes no notes: nothing happens.
            let editor = point(px(900.), px(300.));
            assert!(drag(&f, cx, c, editor));
            assert_eq!(note(cx, c).book_id, work);
            assert_eq!(note(cx, c).tags, vec![tag]);
        }
    }
}
