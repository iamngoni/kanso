//! The Schedule (issue #390): a month grid or a week, with the tasks and
//! the notes that have a date property on their days, over the window beside
//! the sidebar.
//!
//! Which items there are is `kanso_engine::schedule`, asked through
//! [`Filters`]; a repeating task shows on each day its repeat comes round
//! (marked ↻), never written. Clicking a task ticks it (a projected
//! occurrence ticks the real task, which writes the next one), clicking a
//! note opens it, and dragging a note to another day changes its date
//! property, keeping the time and zone; dragging a task changes its due
//! date. The keys are the Journal calendar's (arrows, Page Up and Down, T,
//! Return), with J and K for the items of the day, Space or X to tick, M
//! and W for month and week, Cmd-Z to take the last change back, Esc to
//! close.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenu, PopupMenuItem};
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::component::{
    Icon, Selectable as _, Sizable as _, StyledExt as _, WindowExt as _, h_flex, v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_core::dates::Date;
use kanso_core::properties;
use kanso_core::reschedule::move_value;
use kanso_core::schema::PropertyType;
use kanso_engine::schedule::{ScheduleItem, ScheduleKind, schedule};
use kanso_store::{CheckOff, Origin, Store};

use crate::calendar::{system_week_start, weekday_labels};
use crate::calendar_view::{
    GoToToday, NextDayCell, NextMonth, NextWeek, OpenDay, PreviousDayCell, PreviousMonth,
    PreviousWeek,
};
use crate::clock::Clock;
use crate::editor::NoteEditor;
use crate::layout::{BOTTOM_BAND_HEIGHT, TOP_BAND_HEIGHT};
use crate::library::{Library, LibraryEvent};
use crate::schedule_model::{
    Filters, Mode, Undo, by_day, can_drag, keep_selection, note_mark, range_of, step_period,
    step_selection, task_title, time_label, title, week_of,
};
use crate::tasks_view::writes_held;
use crate::theme::Colors;
use crate::workspace::Workspace;
use crate::zoom::px;

actions!(kanso, [ShowSchedule]);

/// How long changes to notes settle before the Schedule is read again.
const RELOAD_DELAY: Duration = Duration::from_millis(120);
/// The items a month's day shows before "+n more".
const MONTH_CHIPS: usize = 3;

/// What the view asks of the workspace.
pub enum ScheduleEvent {
    /// Open the note (a task's at its line).
    Open {
        note_id: String,
        line: Option<u32>,
    },
    Notice(String),
    Close,
}

/// An item being dragged to another day.
#[derive(Clone)]
pub struct DraggedScheduleItem {
    pub kind: ScheduleKind,
    pub note_id: String,
    pub line: Option<u32>,
    pub text: String,
    pub property: Option<String>,
    label: SharedString,
}

impl Render for DraggedScheduleItem {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        div()
            .max_w(px(220.))
            .px(px(10.))
            .py(px(5.))
            .rounded(px(6.))
            .border_1()
            .border_color(Hsla::from(colors.accent).opacity(0.6))
            .bg(colors.elevated)
            .shadow_md()
            .text_size(px(12.5))
            .font_medium()
            .text_color(colors.text)
            .truncate()
            .child(self.label.clone())
    }
}

/// The last change, and how to take it back.
struct Last {
    what: String,
    undo: Undo,
}

pub struct ScheduleView {
    library: Entity<Library>,
    editor: Entity<NoteEditor>,
    store: Arc<Store>,
    pub(crate) mode: Mode,
    pub(crate) focused: Date,
    today: Date,
    week_start: usize,
    pub(crate) filters: Filters,
    pub(crate) items: Vec<ScheduleItem>,
    days: BTreeMap<Date, Vec<usize>>,
    /// The keyboard's item among the focused day's.
    selected: Option<usize>,
    /// The date properties the filter can name.
    pub(crate) date_keys: Vec<String>,
    last: Option<Last>,
    /// Drawn on screen.
    pub visible: bool,
    generation: u64,
    focus: FocusHandle,
    pane_width: Pixels,
    leading_inset: Pixels,
    reload_task: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<ScheduleEvent> for ScheduleView {}

impl Focusable for ScheduleView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl ScheduleView {
    pub fn new(
        library: Entity<Library>,
        editor: Entity<NoteEditor>,
        cx: &mut Context<Self>,
    ) -> Self {
        let store = library.read(cx).store.clone();
        let subscriptions = vec![cx.subscribe(&library, |this, _, event, cx| {
            if let LibraryEvent::Changed(change) = event
                && this.visible
                && (change.id.starts_with("note:") || change.id.starts_with("book:"))
            {
                this.reload_soon(cx);
            }
        })];
        let today = crate::journal::now(cx).date;
        ScheduleView {
            library,
            editor,
            store,
            mode: Mode::Month,
            focused: today,
            today,
            week_start: system_week_start(),
            filters: Filters::default(),
            items: Vec::new(),
            days: BTreeMap::new(),
            selected: None,
            date_keys: Vec::new(),
            last: None,
            visible: false,
            generation: 0,
            focus: cx.focus_handle(),
            pane_width: px(900.),
            leading_inset: px(0.),
            reload_task: None,
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

    /// Opens the Schedule on today.
    pub fn show(&mut self, cx: &mut Context<Self>) {
        self.today = crate::journal::now(cx).date;
        self.focused = self.today;
        self.selected = None;
        self.visible = true;
        self.reload(cx);
    }

    fn reload_soon(&mut self, cx: &mut Context<Self>) {
        self.reload_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(RELOAD_DELAY).await;
            this.update(cx, |this, cx| this.reload(cx)).ok();
        }));
    }

    fn range(&self) -> std::ops::Range<Date> {
        range_of(self.mode, self.focused, self.week_start)
    }

    /// Reads the days again, off the main thread.
    pub fn reload(&mut self, cx: &mut Context<Self>) {
        self.generation += 1;
        let generation = self.generation;
        self.read_date_keys();
        let query = self.filters.query(self.range());
        let store = self.store.clone();
        let task = cx
            .background_executor()
            .spawn(async move { schedule(&store, &query) });
        self.reload_task = Some(cx.spawn(async move |this, cx| {
            let found = task.await;
            this.update(cx, |this, cx| {
                if this.generation != generation {
                    return;
                }
                match found {
                    Ok(items) => this.set_items(items),
                    Err(err) => cx.emit(ScheduleEvent::Notice(format!(
                        "Couldn't read the Schedule: {err}"
                    ))),
                }
                cx.notify();
            })
            .ok();
        }));
    }

    /// Reads the days now, on this thread: for tests and snapshots.
    #[cfg(test)]
    pub fn reload_now(&mut self, cx: &mut Context<Self>) {
        self.generation += 1;
        self.read_date_keys();
        let query = self.filters.query(self.range());
        if let Ok(items) = schedule(&self.store, &query) {
            self.set_items(items);
        }
        cx.notify();
    }

    fn set_items(&mut self, items: Vec<ScheduleItem>) {
        self.days = by_day(&items);
        self.items = items;
        let count = self.day_items().len();
        self.selected = keep_selection(self.selected, count);
    }

    /// The date properties of the notebooks in view, for the filter.
    fn read_date_keys(&mut self) {
        let books: Vec<String> = if self.filters.books.is_empty() {
            self.store
                .books()
                .unwrap_or_default()
                .into_iter()
                .map(|book| book.id)
                .collect()
        } else {
            self.filters.books.clone()
        };
        let mut keys: Vec<String> = books
            .iter()
            .flat_map(|id| self.store.notebook_properties(id).unwrap_or_default())
            .filter(|def| {
                matches!(
                    def.kind,
                    PropertyType::Date | PropertyType::DateTime | PropertyType::DateTimeZone
                )
            })
            .map(|def| def.key)
            .collect();
        keys.sort();
        keys.dedup();
        self.date_keys = keys;
    }

    /// The indexes of the focused day's items.
    fn day_items(&self) -> Vec<usize> {
        self.days.get(&self.focused).cloned().unwrap_or_default()
    }

    /// The item the keyboard is on.
    fn selected_item(&self) -> Option<usize> {
        self.selected
            .and_then(|at| self.day_items().get(at).copied())
    }

    // MARK: Moving about

    pub(crate) fn set_focus(&mut self, day: Date, cx: &mut Context<Self>) {
        let before = self.range();
        self.focused = day;
        self.selected = None;
        if self.range() != before {
            self.reload(cx);
        }
        cx.notify();
    }

    fn step_days(&mut self, days: i64, cx: &mut Context<Self>) {
        self.set_focus(self.focused.add_days(days), cx);
    }

    fn step_periods(&mut self, delta: i32, cx: &mut Context<Self>) {
        self.set_focus(step_period(self.mode, self.focused, delta), cx);
    }

    pub(crate) fn set_mode(&mut self, mode: Mode, cx: &mut Context<Self>) {
        if self.mode != mode {
            self.mode = mode;
            self.reload(cx);
            cx.notify();
        }
    }

    pub(crate) fn step_selected(&mut self, delta: isize, cx: &mut Context<Self>) {
        self.selected = step_selection(self.selected, self.day_items().len(), delta);
        cx.notify();
    }

    pub(crate) fn set_book(&mut self, id: Option<&str>, cx: &mut Context<Self>) {
        match id {
            Some(id) => self.filters.toggle_book(id),
            None => self.filters.books.clear(),
        }
        self.reload(cx);
        cx.notify();
    }

    pub(crate) fn set_key(&mut self, key: Option<&str>, cx: &mut Context<Self>) {
        match key {
            Some(key) => self.filters.toggle_key(key),
            None => self.filters.keys.clear(),
        }
        self.reload(cx);
        cx.notify();
    }

    pub(crate) fn set_hide_done(&mut self, hide: bool, cx: &mut Context<Self>) {
        self.filters.hide_done = hide;
        self.reload(cx);
        cx.notify();
    }

    // MARK: Changing things

    fn notice(&self, message: impl Into<String>, cx: &mut Context<Self>) {
        cx.emit(ScheduleEvent::Notice(message.into()));
    }

    fn can_write(&self, cx: &mut Context<Self>) -> bool {
        if writes_held(cx) {
            self.notice(
                "The Schedule can't be changed while the library is being replaced.",
                cx,
            );
            return false;
        }
        // The note's unsaved typing is saved first, so the write goes on top.
        self.editor.update(cx, |editor, cx| editor.flush(cx));
        true
    }

    /// Ticks task item `ix` (or clears it). A projected occurrence is the
    /// real task, so this ticks it, and a repeating one writes its next.
    pub fn tick(&mut self, ix: usize, cx: &mut Context<Self>) {
        let Some(item) = self.items.get(ix).cloned() else {
            return;
        };
        let (ScheduleKind::Task, Some(line)) = (item.kind, item.line) else {
            return;
        };
        if !self.can_write(cx) {
            return;
        }
        let checked = !item.done;
        let now_ms = Clock::now_ms(cx);
        let today = crate::journal::now(cx).date;
        match self.store.tick_task(
            &item.note_id,
            line,
            &item.text,
            checked,
            today,
            now_ms,
            Origin::Ui,
        ) {
            Ok(result) => match result.off {
                CheckOff::Changed { .. } => {
                    crate::metrics::record(cx, crate::metrics::Feature::TaskToggled);
                    let title = task_title(&item);
                    self.last = Some(Last {
                        what: if checked {
                            format!("Ticked off “{title}”")
                        } else {
                            format!("Opened “{title}” again")
                        },
                        undo: Undo::Ticked {
                            note_id: item.note_id.clone(),
                            line,
                            text: item.text.clone(),
                            checked,
                            undo: result.undo,
                        },
                    });
                }
                CheckOff::Unchanged | CheckOff::Missing => {
                    self.notice(
                        "That task changed in its note. The Schedule is up to date now.",
                        cx,
                    );
                }
            },
            Err(err) => self.notice(format!("Couldn't change the task: {err}"), cx),
        }
        self.reload(cx);
        cx.notify();
    }

    /// Something was dropped on `day`: a note's date property or a task's
    /// due date becomes that day.
    pub fn drop_on(&mut self, day: Date, dragged: &DraggedScheduleItem, cx: &mut Context<Self>) {
        if !self.can_write(cx) {
            return;
        }
        let now_ms = Clock::now_ms(cx);
        let nice = day.long_label();
        let result = match (dragged.kind, &dragged.property, dragged.line) {
            (ScheduleKind::Note, Some(key), _) => {
                self.move_note(&dragged.note_id, key, day, now_ms)
            }
            (ScheduleKind::Task, _, Some(line)) => self
                .store
                .set_task_due(
                    &dragged.note_id,
                    line,
                    &dragged.text,
                    day,
                    now_ms,
                    Origin::Ui,
                )
                .map_err(|err| err.to_string())
                .and_then(|edit| edit.ok_or_else(|| "That task changed in its note.".to_owned())),
            _ => return,
        };
        match result {
            Ok(edit) if edit.changed => {
                self.last = Some(Last {
                    what: format!("Moved “{}” to {nice}", dragged.label),
                    undo: Undo::Moved(edit),
                });
            }
            Ok(_) => {}
            Err(message) => self.notice(message, cx),
        }
        self.reload(cx);
        cx.notify();
    }

    fn move_note(
        &self,
        id: &str,
        key: &str,
        day: Date,
        now_ms: i64,
    ) -> Result<kanso_store::PropertyEdit, String> {
        let note = self
            .store
            .note(id)
            .map_err(|err| err.to_string())?
            .ok_or_else(|| "That note is gone.".to_owned())?;
        let value = properties::parse(&note.body)
            .into_iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value)
            .ok_or_else(|| format!("That note no longer has a {key} date."))?;
        let moved = move_value(&value, day)
            .ok_or_else(|| format!("The {key} property is no longer a date."))?;
        self.store
            .set_note_property(id, key, &moved, now_ms, Origin::Ui)
            .map_err(|err| err.to_string())
    }

    /// Takes the last change back. `true` when there was one.
    pub fn undo(&mut self, cx: &mut Context<Self>) -> bool {
        let Some(last) = self.last.take() else {
            return false;
        };
        if !self.can_write(cx) {
            return true;
        }
        let now_ms = Clock::now_ms(cx);
        let done = match &last.undo {
            Undo::Moved(edit) => self
                .store
                .undo_note_property(edit, Origin::Ui)
                .map_err(|err| err.to_string()),
            Undo::Ticked {
                note_id,
                line,
                text,
                checked,
                undo,
            } => match undo {
                Some(undo) => self
                    .store
                    .undo_tick(note_id, undo, now_ms, Origin::Ui)
                    .map(|off| matches!(off, CheckOff::Changed { .. }))
                    .map_err(|err| err.to_string()),
                None => self
                    .store
                    .set_task_checked(note_id, *line, text, !checked, now_ms, Origin::Ui)
                    .map(|off| matches!(off, CheckOff::Changed { .. }))
                    .map_err(|err| err.to_string()),
            },
        };
        match done {
            Ok(true) => {}
            Ok(false) => self.notice("That changed since, so it was left as it is.", cx),
            Err(err) => self.notice(format!("Couldn't undo: {err}"), cx),
        }
        self.reload(cx);
        cx.notify();
        true
    }

    fn open_item(&mut self, ix: usize, cx: &mut Context<Self>) {
        if let Some(item) = self.items.get(ix) {
            cx.emit(ScheduleEvent::Open {
                note_id: item.note_id.clone(),
                line: item.line,
            });
        }
    }

    /// Return: opens the item the keyboard is on, or the day's first.
    fn open_selected(&mut self, cx: &mut Context<Self>) {
        let ix = self
            .selected_item()
            .or_else(|| self.day_items().first().copied());
        if let Some(ix) = ix {
            self.open_item(ix, cx);
        }
    }

    // MARK: Drawing

    fn render_chip(
        &self,
        ix: usize,
        compact: bool,
        selected: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let colors = Colors::get(cx);
        let item = &self.items[ix];
        let is_task = item.kind == ScheduleKind::Task;
        let accent = Hsla::from(colors.accent);
        let title = if is_task {
            task_title(item)
        } else {
            item.note_title.clone()
        };
        let mut tip = title.clone();
        if is_task {
            tip.push_str(&format!(" · in {}", item.note_title));
        } else if let Some(key) = &item.property {
            tip.push_str(&format!(" · {key}"));
        }
        if let Some(time) = time_label(item) {
            tip.push_str(&format!(" · {time}"));
            if let Some(zone) = &item.zone {
                tip.push_str(&format!(" {}", kanso_core::zoned::zone_label(zone)));
            }
        }
        if let Some(repeat) = &item.repeat {
            tip.push_str(&format!(" · {repeat}"));
        }
        let dragged = DraggedScheduleItem {
            kind: item.kind,
            note_id: item.note_id.clone(),
            line: item.line,
            text: item.text.clone(),
            property: item.property.clone(),
            label: title.clone().into(),
        };
        let draggable = can_drag(item);
        let done = item.done;
        let projected = item.projected;
        let size = if compact { 11.5 } else { 12.5 };
        let leading: AnyElement = if is_task {
            Icon::new(if done {
                Lucide::CircleCheck
            } else {
                Lucide::Circle
            })
            .size(px(12.))
            .text_color(if done {
                colors.success
            } else {
                colors.text_muted
            })
            .into_any_element()
        } else {
            div()
                .flex_none()
                .text_size(px(size - 1.))
                .text_color(colors.accent)
                .child(note_mark(item.property.as_deref()))
                .into_any_element()
        };
        let time = time_label(item);
        let repeating = item.repeat.is_some();
        h_flex()
            .id(SharedString::from(format!("schedule-item-{ix}")))
            .debug_selector(move || format!("schedule-item-{ix}"))
            .w_full()
            .flex_none()
            .min_w_0()
            .gap(px(4.))
            .px(px(5.))
            .py(px(2.))
            .rounded(px(5.))
            .border_1()
            .border_color(if selected {
                accent.opacity(0.7)
            } else {
                Hsla::transparent_black()
            })
            .bg(if is_task {
                Hsla::from(colors.fill)
            } else {
                accent.opacity(0.14)
            })
            .when(projected, |el| el.opacity(0.7))
            .hover(|el| el.bg(colors.hover))
            .text_size(px(size))
            .text_color(if done { colors.text_muted } else { colors.text })
            .cursor_pointer()
            .child(leading)
            .when_some(time, |el, time| {
                el.child(div().flex_none().text_color(colors.text_muted).child(time))
            })
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .when(done, |el| el.line_through())
                    .child(title),
            )
            .when(repeating, |el| {
                el.child(div().flex_none().text_color(colors.text_muted).child("↻"))
            })
            .when(draggable, |el| {
                el.on_drag(dragged, |dragged, _, _, cx| cx.new(|_| dragged.clone()))
            })
            .tooltip(move |window, cx| Tooltip::new(tip.clone()).build(window, cx))
            .on_click(cx.listener(move |this, _: &ClickEvent, window, cx| {
                cx.stop_propagation();
                window.focus(&this.focus, cx);
                let (kind, day) = (this.items[ix].kind, this.items[ix].date);
                this.focused = day;
                match kind {
                    ScheduleKind::Task => this.tick(ix, cx),
                    ScheduleKind::Note => this.open_item(ix, cx),
                }
            }))
            .into_any_element()
    }

    fn render_day(&self, day: Date, month: Option<u32>, cx: &mut Context<Self>) -> AnyElement {
        let colors = Colors::get(cx);
        let accent = Hsla::from(colors.accent);
        let compact = self.mode == Mode::Month;
        let inside = month.is_none_or(|month| day.month == month);
        let today = day == self.today;
        let focused_day = day == self.focused;
        let selected_ix = self.selected_item();
        let ixs = self.days.get(&day).cloned().unwrap_or_default();
        let shown = if compact { MONTH_CHIPS } else { ixs.len() };
        let more = ixs.len().saturating_sub(shown);
        let chips: Vec<AnyElement> = ixs
            .iter()
            .take(shown)
            .map(|&ix| self.render_chip(ix, compact, focused_day && selected_ix == Some(ix), cx))
            .collect();
        let number = div()
            .size(px(22.))
            .flex()
            .items_center()
            .justify_center()
            .rounded_full()
            .text_size(px(12.))
            .when(today, |el| el.bg(colors.accent).font_semibold())
            .text_color(if today {
                colors.button_text
            } else if inside {
                colors.text
            } else {
                colors.text_muted
            })
            .child(day.day.to_string());
        let heading = if compact {
            h_flex().child(number).into_any_element()
        } else {
            h_flex()
                .gap(px(6.))
                .child(
                    div()
                        .text_size(px(11.5))
                        .font_medium()
                        .text_color(colors.text_muted)
                        .child(weekday_labels(0)[day.weekday()]),
                )
                .child(number)
                .into_any_element()
        };
        v_flex()
            .id(SharedString::from(format!("schedule-day-{}", day.iso())))
            .debug_selector(move || format!("schedule-day-{}", day.iso()))
            .flex_1()
            .h_full()
            .min_w_0()
            .min_h_0()
            .gap(px(2.))
            .p(px(4.))
            .overflow_hidden()
            .border_r_1()
            .border_b_1()
            .border_color(colors.divider)
            .when(focused_day, |el| el.bg(accent.opacity(0.07)))
            .when(!inside, |el| el.opacity(0.6))
            .drag_over::<DraggedScheduleItem>(move |style, _, _, _| style.bg(accent.opacity(0.18)))
            .on_drop(
                cx.listener(move |this, dragged: &DraggedScheduleItem, _, cx| {
                    this.drop_on(day, dragged, cx)
                }),
            )
            .on_click(cx.listener(move |this, _: &ClickEvent, window, cx| {
                window.focus(&this.focus, cx);
                this.set_focus(day, cx);
            }))
            .child(heading)
            .child(
                v_flex()
                    .id(SharedString::from(format!("schedule-chips-{}", day.iso())))
                    .gap(px(2.))
                    .min_h_0()
                    .when(!compact, |el| el.overflow_y_scroll())
                    .children(chips),
            )
            .when(more > 0, |el| {
                el.child(
                    div()
                        .id(SharedString::from(format!("schedule-more-{}", day.iso())))
                        .px(px(5.))
                        .text_size(px(11.))
                        .text_color(colors.text_muted)
                        .cursor_pointer()
                        .child(format!("+{more} more"))
                        .on_click(cx.listener(move |this, _: &ClickEvent, window, cx| {
                            cx.stop_propagation();
                            window.focus(&this.focus, cx);
                            this.focused = day;
                            this.set_mode(Mode::Week, cx);
                        })),
                )
            })
            .into_any_element()
    }

    fn render_grid(&self, cx: &mut Context<Self>) -> AnyElement {
        let colors = Colors::get(cx);
        let labels = h_flex()
            .flex_none()
            .children(weekday_labels(self.week_start).map(|label| {
                div()
                    .flex_1()
                    .px(px(8.))
                    .py(px(4.))
                    .text_size(px(11.5))
                    .font_medium()
                    .text_color(colors.text_muted)
                    .child(label)
            }));
        let rows: Vec<AnyElement> = match self.mode {
            Mode::Month => {
                let grid = kanso_engine::calendar::MonthGrid::new(
                    self.focused.year,
                    self.focused.month,
                    self.week_start,
                );
                grid.weeks
                    .iter()
                    .map(|week| {
                        h_flex()
                            .flex_1()
                            .min_h_0()
                            .items_stretch()
                            .children(
                                week.iter()
                                    .map(|&day| self.render_day(day, Some(grid.month), cx)),
                            )
                            .into_any_element()
                    })
                    .collect()
            }
            Mode::Week => vec![
                h_flex()
                    .flex_1()
                    .min_h_0()
                    .items_stretch()
                    .children(
                        week_of(self.focused, self.week_start)
                            .iter()
                            .map(|&day| self.render_day(day, None, cx)),
                    )
                    .into_any_element(),
            ],
        };
        v_flex()
            .id("schedule-grid")
            .flex_1()
            .min_h_0()
            .border_t_1()
            .border_l_1()
            .border_color(colors.divider)
            .when(self.mode == Mode::Month, |el| el.child(labels))
            .children(rows)
            .into_any_element()
    }

    fn render_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let view = cx.entity();
        let library = self.library.read(cx);
        let books_label = match self.filters.books.as_slice() {
            [] => "All notebooks".to_owned(),
            [id] => library
                .book(id)
                .map_or("1 notebook".to_owned(), |book| book.name.clone()),
            more => format!("{} notebooks", more.len()),
        };
        let keys_label = match self.filters.keys.as_slice() {
            [] => "All dates".to_owned(),
            [key] => key.clone(),
            more => format!("{} properties", more.len()),
        };
        let books_view = view.clone();
        let keys_view = view.clone();
        let nav = |id: &'static str, icon: Lucide, tip: &'static str| {
            Button::new(id)
                .ghost()
                .small()
                .icon(icon)
                .tooltip(tip)
                .debug_selector(move || id.into())
        };
        let mode_button = |id: &'static str, label: &'static str, mode: Mode| {
            Button::new(id)
                .ghost()
                .small()
                .label(label)
                .selected(self.mode == mode)
                .debug_selector(move || id.into())
                .on_click(cx.listener(move |this, _, _, cx| this.set_mode(mode, cx)))
        };
        h_flex()
            .h(px(TOP_BAND_HEIGHT))
            .flex_shrink_0()
            .w_full()
            .pl(px(16.) + self.leading_inset)
            .pr(px(10.))
            .gap(px(4.))
            .border_b_1()
            .border_color(colors.divider)
            .bg(colors.editor)
            .child(
                div()
                    .min_w_0()
                    .truncate()
                    .text_size(px(14.))
                    .font_semibold()
                    .text_color(colors.text)
                    .debug_selector(|| "schedule-title".into())
                    .child(title(self.mode, self.focused, self.week_start)),
            )
            .child(
                nav(
                    "schedule-previous",
                    Lucide::ChevronLeft,
                    "Previous (Page Up)",
                )
                .on_click(cx.listener(|this, _, _, cx| this.step_periods(-1, cx))),
            )
            .child(
                Button::new("schedule-today")
                    .ghost()
                    .small()
                    .label("Today")
                    .tooltip("Go to today (T)")
                    .debug_selector(|| "schedule-today".into())
                    .on_click(cx.listener(|this, _, _, cx| {
                        let today = this.today;
                        this.set_focus(today, cx)
                    })),
            )
            .child(
                nav("schedule-next", Lucide::ChevronRight, "Next (Page Down)")
                    .on_click(cx.listener(|this, _, _, cx| this.step_periods(1, cx))),
            )
            .child(div().flex_1())
            .child(mode_button("schedule-month", "Month", Mode::Month))
            .child(mode_button("schedule-week", "Week", Mode::Week))
            .when(self.filters.is_refined(), |el| {
                el.child(
                    Button::new("schedule-clear")
                        .ghost()
                        .small()
                        .label("Clear filters")
                        .debug_selector(|| "schedule-clear".into())
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.filters = Filters::default();
                            this.reload(cx);
                            cx.notify();
                        })),
                )
            })
            .child(
                Button::new("schedule-books")
                    .ghost()
                    .small()
                    .label(books_label)
                    .selected(!self.filters.books.is_empty() || self.filters.hide_done)
                    .tooltip("Filter by notebook")
                    .debug_selector(|| "schedule-books".into())
                    .dropdown_menu_with_anchor(Anchor::TopRight, move |menu, _, cx| {
                        books_menu(menu, &books_view, cx)
                    }),
            )
            .child(
                Button::new("schedule-keys")
                    .ghost()
                    .small()
                    .label(keys_label)
                    .selected(!self.filters.keys.is_empty())
                    .tooltip("Filter by date property")
                    .debug_selector(|| "schedule-keys".into())
                    .dropdown_menu_with_anchor(Anchor::TopRight, move |menu, _, cx| {
                        keys_menu(menu, &keys_view, cx)
                    }),
            )
            .child(
                Button::new("schedule-close")
                    .ghost()
                    .small()
                    .icon(Lucide::Close)
                    .tooltip("Close (Esc)")
                    .debug_selector(|| "schedule-close".into())
                    .on_click(cx.listener(|_, _, _, cx| cx.emit(ScheduleEvent::Close))),
            )
    }

    fn render_footer(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let what = self.last.as_ref().map(|last| last.what.clone());
        h_flex()
            .h(px(BOTTOM_BAND_HEIGHT))
            .flex_none()
            .px(px(18.))
            .gap_1()
            .border_t_1()
            .border_color(colors.divider)
            .bg(colors.editor)
            .text_size(px(12.))
            .text_color(colors.text_muted)
            .map(|el| match what {
                Some(what) => el.child(div().flex_1().min_w_0().truncate().child(what)).child(
                    Button::new("schedule-undo")
                        .ghost()
                        .xsmall()
                        .label("Undo")
                        .debug_selector(|| "schedule-undo".into())
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.undo(cx);
                        })),
                ),
                None => el.child(div().truncate().child(
                    "Click a task to tick it · drag onto a day to move it · ↻ repeats · arrows move, T today, M month, W week, Esc close",
                )),
            })
    }
}

/// The notebook filter: which notebooks, and whether done tasks show.
fn books_menu(menu: PopupMenu, view: &Entity<ScheduleView>, cx: &App) -> PopupMenu {
    let state = view.read(cx);
    let chosen = state.filters.books.clone();
    let hide_done = state.filters.hide_done;
    let books: Vec<(usize, String, String)> = state
        .library
        .read(cx)
        .book_tree()
        .into_iter()
        .map(|(depth, book)| (depth, book.id.clone(), book.name.clone()))
        .collect();
    let all = view.clone();
    let mut menu = menu.item(
        PopupMenuItem::new("All notebooks")
            .checked(chosen.is_empty())
            .on_click(move |_, _, cx| all.update(cx, |this, cx| this.set_book(None, cx))),
    );
    for (depth, id, name) in books {
        let view = view.clone();
        let checked = chosen.contains(&id);
        menu = menu.item(
            PopupMenuItem::new(format!("{}{name}", "   ".repeat(depth)))
                .checked(checked)
                .on_click(move |_, _, cx| {
                    let id = id.clone();
                    view.update(cx, |this, cx| this.set_book(Some(&id), cx))
                }),
        );
    }
    let done = view.clone();
    menu.separator().item(
        PopupMenuItem::new("Show Completed")
            .checked(!hide_done)
            .on_click(move |_, _, cx| {
                done.update(cx, |this, cx| this.set_hide_done(!hide_done, cx))
            }),
    )
}

/// The property filter: which date properties place a note.
fn keys_menu(menu: PopupMenu, view: &Entity<ScheduleView>, cx: &App) -> PopupMenu {
    let state = view.read(cx);
    let chosen = state.filters.keys.clone();
    let keys = state.date_keys.clone();
    let all = view.clone();
    let mut menu = menu.item(
        PopupMenuItem::new("All date properties")
            .checked(chosen.is_empty())
            .on_click(move |_, _, cx| all.update(cx, |this, cx| this.set_key(None, cx))),
    );
    if keys.is_empty() {
        menu = menu.item(PopupMenuItem::new("No date properties yet").disabled(true));
    }
    for key in keys {
        let view = view.clone();
        let checked = chosen.contains(&key);
        menu = menu.item(PopupMenuItem::new(key.clone()).checked(checked).on_click(
            move |_, _, cx| {
                let key = key.clone();
                view.update(cx, |this, cx| this.set_key(Some(&key), cx))
            },
        ));
    }
    menu
}

// no-prop-test: drawing and wiring; `schedule_model` carries the logic with
// its properties, and the UI tests and snapshots drive and show the view.
impl Render for ScheduleView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        v_flex()
            .id("schedule-view")
            .key_context("Calendar")
            .track_focus(&self.focus)
            .size_full()
            .bg(colors.editor)
            .on_action(cx.listener(|this, _: &PreviousDayCell, _, cx| this.step_days(-1, cx)))
            .on_action(cx.listener(|this, _: &NextDayCell, _, cx| this.step_days(1, cx)))
            .on_action(cx.listener(|this, _: &PreviousWeek, _, cx| this.step_days(-7, cx)))
            .on_action(cx.listener(|this, _: &NextWeek, _, cx| this.step_days(7, cx)))
            .on_action(cx.listener(|this, _: &PreviousMonth, _, cx| this.step_periods(-1, cx)))
            .on_action(cx.listener(|this, _: &NextMonth, _, cx| this.step_periods(1, cx)))
            .on_action(cx.listener(|this, _: &GoToToday, _, cx| {
                let today = this.today;
                this.set_focus(today, cx)
            }))
            .on_action(cx.listener(|this, _: &OpenDay, _, cx| this.open_selected(cx)))
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                let k = &event.keystroke;
                if k.modifiers.platform || k.modifiers.control || k.modifiers.alt {
                    return;
                }
                match k.key.as_str() {
                    "escape" => cx.emit(ScheduleEvent::Close),
                    "j" => this.step_selected(1, cx),
                    "k" => this.step_selected(-1, cx),
                    "space" | "x" => {
                        if let Some(ix) = this.selected_item() {
                            this.tick(ix, cx);
                        }
                    }
                    "m" => this.set_mode(Mode::Month, cx),
                    "w" => this.set_mode(Mode::Week, cx),
                    _ => return,
                }
                cx.stop_propagation();
            }))
            // Presses here are the view's, not the window's below.
            .on_any_mouse_down(|_, _, cx| cx.stop_propagation())
            .child(self.render_header(cx))
            .child(self.render_grid(cx))
            .child(self.render_footer(cx))
    }
}

// MARK: Workspace

/// The workspace's part: opening and closing the Schedule.
impl Workspace {
    pub(crate) fn show_schedule(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.close_settings(window, cx);
        self.close_graph(window, cx);
        self.close_board(window, cx);
        self.close_triage(window, cx);
        self.close_flashcards(window, cx);
        self.set_focus_mode(false, window, cx);
        self.flush(cx);
        let view = match &self.schedule {
            Some(view) => view.clone(),
            None => {
                let library = self.library.clone();
                let editor = self.editor.clone();
                let view = cx.new(|cx| ScheduleView::new(library, editor, cx));
                self._schedule_events = Some(cx.subscribe_in(
                    &view,
                    window,
                    |this, _, event, window, cx| this.on_schedule_event(event, window, cx),
                ));
                self.schedule = Some(view.clone());
                view
            }
        };
        view.update(cx, |view, cx| view.show(cx));
        self.schedule_open = true;
        window.focus(&view.focus_handle(cx), cx);
        cx.notify();
    }

    pub(crate) fn close_schedule(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.schedule_open {
            return;
        }
        self.schedule_open = false;
        if let Some(view) = &self.schedule {
            view.update(cx, |view, _| view.visible = false);
        }
        window.focus(&self.focus, cx);
        cx.notify();
    }

    fn on_schedule_event(
        &mut self,
        event: &ScheduleEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            ScheduleEvent::Close => self.close_schedule(window, cx),
            ScheduleEvent::Notice(message) => window.push_notification(
                gpui_kit::component::notification::Notification::info(message.clone()),
                cx,
            ),
            ScheduleEvent::Open { note_id, line } => {
                self.close_schedule(window, cx);
                self.open_note_at(note_id, line.unwrap_or(0), true, window, cx);
            }
        }
    }
}

/// The workspace's action for the Schedule.
pub fn on_actions(el: Div, cx: &mut Context<Workspace>) -> Div {
    el.on_action(cx.listener(|this, _: &ShowSchedule, window, cx| this.show_schedule(window, cx)))
}

/// The Schedule over the window, beside the sidebar, while open.
pub fn overlay(workspace: &Workspace, leading: f32) -> Option<impl IntoElement> {
    let view = workspace
        .schedule
        .clone()
        .filter(|_| workspace.schedule_open)?;
    Some(
        div()
            .absolute()
            .top_0()
            .left(px(leading))
            .right_0()
            .bottom_0()
            .debug_selector(|| "schedule-overlay".into())
            .child(view),
    )
}
