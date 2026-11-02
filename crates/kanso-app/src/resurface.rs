//! Resurfacing related notes while you write: a quiet margin beside the
//! text that shows up to three past notes about the paragraph around the
//! caret, and "On this day" on daily notes.
//!
//! What is asked and how it is worded is the engine's
//! (`kanso_engine::discovery::resurface`); the searching is the store's
//! (`kanso-store`'s `meaning/resurface.rs`), on the on-device model related
//! notes already use. This is the state and the views: the margin
//! ([`Resurface`] as a view), and the same list in the inspector's Related
//! tab ([`item_list`]) when the pane is too narrow, the inspector is
//! open, or the setting says so.
//!
//! Calm by design: it asks only once typing has paused ([`SETTLE`]) or the
//! caret has moved to another paragraph, never makes a sound or a badge,
//! fades a new item in over 150 ms, keeps the items that stay relevant where
//! they are, and shows nothing at all, not even its title, when nothing is
//! close enough. Locked notes and notes tagged `#ai-exclude` are never
//! searched from or shown.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::base::input::{EditorState, InputEvent, InputState};
use gpui_kit::base::text::TextView;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{Icon, Sizable as _, StyledExt as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_engine::discovery::resurface::{Item, OnThisDay, SHOWN, on_this_day, present, query};
use kanso_store::{Resurfacing, Store};

use crate::clock::Clock;
use crate::related::{Related, SETTLE, Status};
use crate::theme::Colors;
use crate::workspace::AppState;
use crate::zoom::px;

/// The margin's width, in points.
pub const MARGIN_WIDTH: f32 = 240.;
/// The editor pane has to be this wide for the margin to show.
pub const MIN_PANE: f32 = 900.;
/// How long the pointer rests on an item before its peek shows.
const PEEK_DELAY: Duration = Duration::from_millis(400);
/// How soon the margin asks again after the caret moves to another
/// paragraph without any typing.
const MOVE_DELAY: Duration = Duration::from_millis(250);
/// How much of a note its peek shows, in characters.
const PEEK_CHARS: usize = 600;
/// The fade a new item comes in with.
const FADE: Duration = Duration::from_millis(150);

const CONTEXT: &str = "Resurface";

/// What the margin asks of the editor.
pub enum ResurfaceEvent {
    /// Put `[[Title]]` at the caret.
    Link(String),
    /// Open the note, in a new tab for ⌘-click.
    Open { id: String, new_tab: bool },
    /// Escape: the keyboard goes back to the text.
    Leave,
}

impl EventEmitter<ResurfaceEvent> for Resurface {}

/// An item with the generation it appeared in, for its fade.
#[derive(Clone)]
struct Shown {
    item: Item,
    born: u64,
}

/// A note's first words, shown beside its item.
struct Peek {
    id: String,
    pinned: bool,
    text: SharedString,
}

/// Everything one search needs, taken on the main thread.
struct Job {
    store: Arc<Store>,
    id: String,
    query: Option<String>,
    opts: kanso_store::ResurfaceOpts,
    now_ms: i64,
    offset: i32,
}

pub struct Resurface {
    store: Arc<Store>,
    body: Entity<EditorState>,
    title: Entity<InputState>,
    /// The open note, when the margin may work for it.
    note: Option<String>,
    /// Whether the margin has room: the pane is wide, the inspector shut.
    room: bool,
    items: Vec<Shown>,
    /// Notes written on this day in earlier years, on a daily note.
    day: Vec<OnThisDay>,
    day_task: Option<Task<()>>,
    /// Notes the margin is hidden for until another opens.
    hidden: HashSet<String>,
    /// What the last search was asked, so the same paragraph isn't asked
    /// twice.
    asked: Option<String>,
    /// The text length and caret the last change was seen at.
    seen: (usize, usize),
    generation: u64,
    task: Option<Task<()>>,
    peek: Option<Peek>,
    peek_task: Option<Task<()>>,
    menu: bool,
    /// The item the keyboard is on.
    cursor: Option<usize>,
    focus: FocusHandle,
    _subscriptions: Vec<Subscription>,
}

impl Resurface {
    pub fn new(
        store: Arc<Store>,
        body: Entity<EditorState>,
        title: Entity<InputState>,
        cx: &mut Context<Self>,
    ) -> Self {
        let subscriptions = vec![
            // Typing: ask once it has paused.
            cx.subscribe(&body, |this, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    this.changed(SETTLE, cx);
                }
            }),
            // The caret moving to another paragraph.
            cx.observe(&body, |this, _, cx| this.moved(cx)),
            // New embeddings, or related notes turned on or off.
            cx.observe_global::<Related>(|this, cx| {
                this.asked = None;
                this.changed(MOVE_DELAY, cx);
            }),
        ];
        Resurface {
            store,
            body,
            title,
            note: None,
            room: false,
            items: Vec::new(),
            day: Vec::new(),
            day_task: None,
            hidden: HashSet::new(),
            asked: None,
            seen: (0, 0),
            generation: 0,
            task: None,
            peek: None,
            peek_task: None,
            menu: false,
            cursor: None,
            focus: cx.focus_handle(),
            _subscriptions: subscriptions,
        }
    }

    /// The note open now (`None`: none, or one the margin must not work
    /// for: locked, or tagged `#ai-exclude`).
    pub fn follow(&mut self, note: Option<&str>, cx: &mut Context<Self>) {
        if self.note.as_deref() == note {
            return;
        }
        self.note = note.map(str::to_owned);
        self.items.clear();
        self.peek = None;
        self.cursor = None;
        self.menu = false;
        self.asked = None;
        self.task = None;
        self.seen = (0, 0);
        self.load_day(cx);
        if note.is_some() {
            self.changed(MOVE_DELAY, cx);
        }
        cx.notify();
    }

    /// Whether the margin has the room to show beside the text.
    pub fn set_room(&mut self, room: bool, cx: &mut Context<Self>) {
        if self.room != room {
            self.room = room;
            cx.notify();
        }
    }

    /// The settings as they are now.
    fn settings(cx: &App) -> crate::config::DiscoverySettings {
        // Without the app's state (the editor alone, in tests) related notes
        // are taken to be on, as the margin's own tests need.
        cx.try_global::<AppState>().map_or_else(
            || crate::config::DiscoverySettings {
                related_notes: true,
                ..Default::default()
            },
            |state| state.config.discovery,
        )
    }

    /// Whether the margin works at all now: related notes on, the setting
    /// on, a note open, and not hidden for it.
    fn enabled(&self, cx: &App) -> bool {
        let settings = Self::settings(cx);
        settings.related_notes
            && settings.resurface
            && self
                .note
                .as_deref()
                .is_some_and(|id| !self.hidden.contains(id))
    }

    /// Whether anything shows: items, or "On this day".
    pub fn has_content(&self, cx: &App) -> bool {
        (self.enabled(cx) && !self.items.is_empty()) || self.shows_day(cx)
    }

    fn shows_day(&self, cx: &App) -> bool {
        Self::settings(cx).on_this_day && !self.day.is_empty() && self.note.is_some()
    }

    /// Whether the margin is drawn beside the text now.
    pub fn in_margin(&self, cx: &App) -> bool {
        self.room && Self::settings(cx).resurface_margin && self.has_content(cx)
    }

    // MARK: Asking

    /// The text changed: ask after `delay`.
    fn changed(&mut self, delay: Duration, cx: &mut Context<Self>) {
        self.seen = self.signature(cx);
        self.schedule(delay, cx);
    }

    /// The caret may have moved: ask soon if it did, to another paragraph.
    fn moved(&mut self, cx: &mut Context<Self>) {
        if self.note.is_none() {
            return;
        }
        let now = self.signature(cx);
        if now == self.seen {
            return;
        }
        // Typing is told by its change event, which waits for a pause.
        if now.0 == self.seen.0 {
            self.seen = now;
            self.schedule(MOVE_DELAY, cx);
        } else {
            self.seen = now;
        }
    }

    fn signature(&self, cx: &App) -> (usize, usize) {
        let body = self.body.read(cx);
        (body.text().len(), body.cursor())
    }

    fn schedule(&mut self, delay: Duration, cx: &mut Context<Self>) {
        if !self.enabled(cx) {
            if !self.items.is_empty() {
                self.items.clear();
                cx.notify();
            }
            return;
        }
        self.task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(delay).await;
            let job = this.update(cx, |this, cx| this.job(cx)).ok().flatten();
            let Some(job) = job else { return };
            let found = cx
                .background_executor()
                .spawn(async move { job.run() })
                .await;
            this.update(cx, |this, cx| this.apply(found, cx)).ok();
        }));
    }

    /// What to search for now, or `None` when there is nothing to ask:
    /// the paragraph is the one already asked about, or too short (which
    /// also clears the items).
    fn job(&mut self, cx: &mut Context<Self>) -> Option<Job> {
        let id = self.note.clone()?;
        if !self.enabled(cx) {
            return None;
        }
        let (title, text, caret) = {
            let title = self.title.read(cx).value().to_string();
            let body = self.body.read(cx);
            (title, body.value().to_string(), body.cursor())
        };
        let query = query(&title, &text, caret);
        if query == self.asked {
            return None;
        }
        self.asked = query.clone();
        if query.is_none() {
            if !self.items.is_empty() {
                self.items.clear();
                cx.notify();
            }
            return None;
        }
        let now_ms = Clock::now_ms(cx);
        Some(Job {
            store: self.store.clone(),
            id,
            query,
            opts: Self::settings(cx).resurface_closeness.opts(),
            now_ms,
            offset: Clock::utc_offset_secs(cx, now_ms),
        })
    }

    /// What a search found: items that stay stay where they are, the gone
    /// go, new ones come in at the bottom.
    fn apply(&mut self, found: Vec<Item>, cx: &mut Context<Self>) {
        self.generation += 1;
        let generation = self.generation;
        let mut next: Vec<Shown> = Vec::new();
        for shown in &self.items {
            if let Some(item) = found.iter().find(|i| i.id == shown.item.id) {
                next.push(Shown {
                    item: item.clone(),
                    born: shown.born,
                });
            }
        }
        for item in found {
            if !next.iter().any(|s| s.item.id == item.id) {
                next.push(Shown {
                    item,
                    born: generation,
                });
            }
        }
        next.truncate(SHOWN);
        let same = next.len() == self.items.len()
            && next.iter().zip(&self.items).all(|(a, b)| a.item == b.item);
        if !same {
            if self.cursor.is_some_and(|c| c >= next.len()) {
                self.cursor = next.len().checked_sub(1);
            }
            self.items = next;
            cx.notify();
        }
    }

    /// "On this day" for the open note, when it is a daily note.
    fn load_day(&mut self, cx: &mut Context<Self>) {
        self.day.clear();
        self.day_task = None;
        let Some(id) = self.note.clone() else { return };
        let Some(date) = kanso_core::roles::daily_date(&id) else {
            return;
        };
        let store = self.store.clone();
        let zone = Clock::zone(cx);
        let read = cx
            .background_executor()
            .spawn(async move { on_this_day(&store, date, &*zone, SHOWN).unwrap_or_default() });
        self.day_task = Some(cx.spawn(async move |this, cx| {
            let found = read.await;
            this.update(cx, |this, cx| {
                this.day = found;
                cx.notify();
            })
            .ok();
        }));
    }

    // MARK: Acting

    /// "Not related": this note is not shown for the open one again.
    fn dismiss(&mut self, id: &str, cx: &mut Context<Self>) {
        let Some(note) = self.note.clone() else {
            return;
        };
        if let Err(err) = self.store.dismiss_related(&note, id) {
            eprintln!("kanso: couldn't keep that: {err}");
        }
        self.items.retain(|s| s.item.id != id);
        if self.peek.as_ref().is_some_and(|p| p.id == id) {
            self.peek = None;
        }
        if self.cursor.is_some_and(|c| c >= self.items.len()) {
            self.cursor = self.items.len().checked_sub(1);
        }
        cx.notify();
    }

    fn hide_for_note(&mut self, cx: &mut Context<Self>) {
        if let Some(note) = self.note.clone() {
            self.hidden.insert(note);
        }
        self.items.clear();
        self.menu = false;
        cx.notify();
    }

    fn hide_everywhere(&mut self, cx: &mut Context<Self>) {
        AppState::update_config(cx, |config| config.discovery.resurface = false);
        self.items.clear();
        self.menu = false;
        cx.refresh_windows();
    }

    fn open(&mut self, id: String, new_tab: bool, cx: &mut Context<Self>) {
        cx.emit(ResurfaceEvent::Open { id, new_tab });
    }

    fn link(&mut self, title: String, cx: &mut Context<Self>) {
        if !title.trim().is_empty() && title != "Untitled" {
            cx.emit(ResurfaceEvent::Link(title));
        }
    }

    /// Shows the note's first words beside its item.
    fn show_peek(&mut self, id: String, pinned: bool, cx: &mut Context<Self>) {
        if let Some(peek) = self.peek.as_mut()
            && peek.id == id
        {
            peek.pinned = pinned || peek.pinned;
            cx.notify();
            return;
        }
        let store = self.store.clone();
        let wanted = id.clone();
        let read = cx.background_executor().spawn(async move {
            store
                .note_excerpt(&wanted, PEEK_CHARS)
                .ok()
                .flatten()
                .unwrap_or_default()
        });
        self.peek_task = Some(cx.spawn(async move |this, cx| {
            let text = read.await;
            this.update(cx, |this, cx| {
                this.peek = Some(Peek {
                    id,
                    pinned,
                    text: text.into(),
                });
                cx.notify();
            })
            .ok();
        }));
    }

    fn hover(&mut self, id: String, over: bool, cx: &mut Context<Self>) {
        if over {
            self.peek_task = Some(cx.spawn(async move |this, cx| {
                cx.background_executor().timer(PEEK_DELAY).await;
                this.update(cx, |this, cx| this.show_peek(id, false, cx))
                    .ok();
            }));
        } else {
            self.peek_task = None;
            if self.peek.as_ref().is_some_and(|p| !p.pinned) {
                self.peek = None;
                cx.notify();
            }
        }
    }

    fn toggle_pinned(&mut self, id: String, cx: &mut Context<Self>) {
        if self.peek.as_ref().is_some_and(|p| p.id == id && p.pinned) {
            self.peek = None;
            cx.notify();
        } else {
            self.show_peek(id, true, cx);
        }
    }

    /// The keyboard moves to the margin, onto its first item.
    pub fn focus_margin(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.items.is_empty() {
            return;
        }
        self.cursor = Some(self.cursor.unwrap_or(0).min(self.items.len() - 1));
        self.focus.focus(window, cx);
        cx.notify();
    }

    fn step(&mut self, forward: bool, cx: &mut Context<Self>) {
        let last = self.items.len().saturating_sub(1);
        self.cursor = Some(match (self.cursor, forward) {
            (None, _) => 0,
            (Some(c), true) => (c + 1).min(last),
            (Some(c), false) => c.saturating_sub(1),
        });
        cx.notify();
    }

    /// Keys while the margin has the keyboard: the arrows move, Enter opens,
    /// L links, Backspace says "Not related", Escape goes back to the text.
    /// Whether the key was used.
    fn key(&mut self, key: &Keystroke, cx: &mut Context<Self>) -> bool {
        if key.modifiers.modified() {
            return false;
        }
        match key.key.as_str() {
            "down" => self.step(true, cx),
            "up" => self.step(false, cx),
            "enter" => {
                if let Some(id) = self.focused_item().map(|i| i.id.clone()) {
                    self.open(id, false, cx);
                }
            }
            "l" => {
                if let Some(title) = self.focused_item().map(|i| i.title.clone()) {
                    self.link(title, cx);
                }
            }
            "backspace" => {
                if let Some(id) = self.focused_item().map(|i| i.id.clone()) {
                    self.dismiss(&id, cx);
                }
            }
            "escape" => cx.emit(ResurfaceEvent::Leave),
            _ => return false,
        }
        true
    }

    fn focused_item(&self) -> Option<&Item> {
        self.cursor.and_then(|c| self.items.get(c)).map(|s| &s.item)
    }

    // MARK: Test access

    #[cfg(test)]
    pub fn items_for_test(&self) -> Vec<Item> {
        self.items.iter().map(|s| s.item.clone()).collect()
    }

    #[cfg(test)]
    pub fn day_for_test(&self) -> Vec<OnThisDay> {
        self.day.clone()
    }

    #[cfg(test)]
    pub fn dismiss_for_test(&mut self, id: &str, cx: &mut Context<Self>) {
        self.dismiss(id, cx);
    }

    /// Runs the settle early and applies a search made of `found`.
    #[cfg(test)]
    pub fn apply_for_test(&mut self, found: Vec<Item>, cx: &mut Context<Self>) {
        self.apply(found, cx);
    }
}

impl Job {
    /// The search, on a background thread. Nothing for a note that may not
    /// be read.
    fn run(self) -> Vec<Item> {
        let Some(text) = self.query else {
            return Vec::new();
        };
        if !self.store.note_discoverable(&self.id).unwrap_or(false) {
            return Vec::new();
        }
        match self.store.resurface(&text, &self.id, self.opts) {
            Ok(Resurfacing::Found(found)) => present(&found, self.now_ms, self.offset),
            _ => Vec::new(),
        }
    }
}

// MARK: Views

/// The list of items, as the margin and the inspector draw it. Handlers
/// reach the state through `view`.
// no-prop-test: drawing; the resurface tests exercise the state it shows.
pub fn item_list(view: &Entity<Resurface>, cx: &App) -> Div {
    let colors = Colors::get(cx);
    let state = view.read(cx);
    let mut list = v_flex().w_full().gap(px(2.));
    if state.shows_day(cx) {
        list = list.child(day_section(&state.day, colors));
    }
    if state.enabled(cx) {
        for (ix, shown) in state.items.iter().enumerate() {
            let focused = state.cursor == Some(ix);
            let peek = state.peek.as_ref().filter(|p| p.id == shown.item.id);
            list = list.child(item_row(view, ix, shown, focused, peek, colors, cx));
        }
    }
    list
}

fn day_section(day: &[OnThisDay], colors: &Colors) -> Div {
    v_flex()
        .w_full()
        .pb(px(6.))
        .child(
            div()
                .px(px(8.))
                .pb(px(2.))
                .text_size(px(11.))
                .text_color(colors.text_muted)
                .child("On this day"),
        )
        .children(day.iter().map(|note| {
            let title = if note.title.trim().is_empty() {
                "Untitled".to_owned()
            } else {
                note.title.clone()
            };
            h_flex()
                .w_full()
                .px(px(8.))
                .py(px(3.))
                .gap(px(6.))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .truncate()
                        .text_size(px(12.5))
                        .text_color(colors.text)
                        .child(title),
                )
                .child(
                    div()
                        .flex_none()
                        .text_size(px(11.))
                        .text_color(colors.text_muted)
                        .child(note.ago.clone()),
                )
        }))
}

fn item_row(
    view: &Entity<Resurface>,
    ix: usize,
    shown: &Shown,
    focused: bool,
    peek: Option<&Peek>,
    colors: &Colors,
    cx: &App,
) -> AnyElement {
    let item = &shown.item;
    let (open, hover, link, dismiss) = (view.clone(), view.clone(), view.clone(), view.clone());
    let (id, hover_id, link_title, dismiss_id) = (
        item.id.clone(),
        item.id.clone(),
        item.title.clone(),
        item.id.clone(),
    );
    let can_link = !(item.title.trim().is_empty() || item.title == "Untitled");
    let row = v_flex()
        .id(ElementId::NamedInteger("resurface-item".into(), ix as u64))
        .debug_selector(move || format!("resurface-item-{ix}"))
        .w_full()
        .gap(px(2.))
        .px(px(8.))
        .py(px(6.))
        .rounded(px(6.))
        .cursor_pointer()
        .when(focused, |el| el.bg(colors.row_hover))
        .hover(|el| el.bg(colors.row_hover))
        .on_hover(move |over, _, cx| {
            let id = hover_id.clone();
            let over = *over;
            hover.update(cx, |this, cx| this.hover(id, over, cx));
        })
        .on_click(move |event, _, cx| {
            let id = id.clone();
            let new_tab = event.modifiers().secondary();
            open.update(cx, |this, cx| {
                if new_tab {
                    this.open(id, true, cx);
                } else {
                    this.toggle_pinned(id, cx);
                }
            });
        })
        .child(
            h_flex()
                .w_full()
                .gap(px(6.))
                .items_baseline()
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .truncate()
                        .text_size(px(12.5))
                        .font_medium()
                        .text_color(colors.text)
                        .child(item.title.clone()),
                )
                .child(
                    div()
                        .flex_none()
                        .max_w(px(110.))
                        .truncate()
                        .text_size(px(11.))
                        .text_color(colors.text_muted)
                        .child(item.place.clone()),
                ),
        )
        .when(!item.snippet.is_empty(), |el| {
            el.child(
                div()
                    .w_full()
                    .max_h(px(48.))
                    .overflow_hidden()
                    .text_size(px(12.))
                    .line_height(relative(1.35))
                    .text_color(colors.text_secondary)
                    .child(item.snippet.clone()),
            )
        })
        .when_some(item.before.clone(), |el, before| {
            el.child(
                div()
                    .text_size(px(11.))
                    .text_color(colors.link)
                    .child(before),
            )
        })
        .child(
            h_flex()
                .gap(px(2.))
                .when(can_link, |el| {
                    el.child(
                        Button::new(ElementId::NamedInteger("resurface-link".into(), ix as u64))
                            .ghost()
                            .xsmall()
                            .label("Link")
                            .tooltip("Put a link to this note at the caret")
                            .on_click(move |_, _, cx| {
                                cx.stop_propagation();
                                let title = link_title.clone();
                                link.update(cx, |this, cx| this.link(title, cx));
                            }),
                    )
                })
                .child(
                    Button::new(ElementId::NamedInteger(
                        "resurface-dismiss".into(),
                        ix as u64,
                    ))
                    .ghost()
                    .xsmall()
                    .icon(Icon::new(Lucide::X).text_color(colors.text_muted))
                    .tooltip("Not related")
                    .on_click(move |_, _, cx| {
                        cx.stop_propagation();
                        let id = dismiss_id.clone();
                        dismiss.update(cx, |this, cx| this.dismiss(&id, cx));
                    }),
                ),
        );
    // A new item fades in once.
    let row = row.with_animation(
        ElementId::NamedInteger("resurface-fade".into(), shown.born * 8 + ix as u64),
        Animation::new(FADE),
        |el, t| el.opacity(t),
    );
    let mut group = v_flex().w_full().child(row);
    if let Some(peek) = peek {
        group = group.child(
            div()
                .mx(px(8.))
                .mb(px(4.))
                .px(px(10.))
                .py(px(8.))
                .max_h(px(220.))
                .overflow_hidden()
                .rounded(px(6.))
                .border_1()
                .border_color(colors.divider)
                .bg(colors.elevated)
                .child(
                    TextView::markdown(
                        ElementId::NamedInteger("resurface-peek".into(), ix as u64),
                        peek.text.clone(),
                    )
                    .text_size(px(12.))
                    .text_color(colors.text),
                ),
        );
    }
    let _ = cx;
    group.into_any_element()
}

impl Render for Resurface {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Nothing to say: no title, no width.
        if !self.in_margin(cx) {
            return div().w(px(0.)).h(px(0.)).into_any_element();
        }
        let colors = Colors::get(cx);
        let status = cx
            .try_global::<Related>()
            .map(|r| r.status.clone())
            .unwrap_or_default();
        let reading = match status {
            Status::Indexing { done, total } if total > 0 => Some(format!(
                "Still reading your notes ({}%)",
                (done * 100 / total).min(100)
            )),
            _ => None,
        };
        let view = cx.entity();
        let menu = self.menu;
        let (hide_note, hide_all, toggle) = (view.clone(), view.clone(), view.clone());
        v_flex()
            .id("resurface")
            .key_context(CONTEXT)
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _, cx| {
                if this.key(&event.keystroke, cx) {
                    cx.stop_propagation();
                }
            }))
            .debug_selector(|| "resurface".into())
            .flex_none()
            .w(px(MARGIN_WIDTH))
            .h_full()
            .overflow_y_scroll()
            .pt(px(8.))
            .pr(px(10.))
            .pb(px(8.))
            .child(
                h_flex()
                    .w_full()
                    .px(px(8.))
                    .pb(px(6.))
                    .gap(px(4.))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .text_size(px(11.))
                            .text_color(colors.text_muted)
                            .child(reading.unwrap_or_else(|| "From your notes".to_owned())),
                    )
                    .child(
                        Button::new("resurface-menu")
                            .ghost()
                            .xsmall()
                            .icon(Icon::new(Lucide::Ellipsis).text_color(colors.text_muted))
                            .tooltip("Margin options")
                            .on_click(move |_, _, cx| {
                                toggle.update(cx, |this, cx| {
                                    this.menu = !this.menu;
                                    cx.notify();
                                })
                            }),
                    ),
            )
            .when(menu, |el| {
                el.child(
                    v_flex()
                        .px(px(8.))
                        .pb(px(6.))
                        .gap(px(2.))
                        .child(
                            Button::new("resurface-hide-note")
                                .ghost()
                                .xsmall()
                                .label("Hide for this note")
                                .on_click(move |_, _, cx| {
                                    hide_note.update(cx, |this, cx| this.hide_for_note(cx))
                                }),
                        )
                        .child(
                            Button::new("resurface-hide-everywhere")
                                .ghost()
                                .xsmall()
                                .label("Hide everywhere")
                                .on_click(move |_, _, cx| {
                                    hide_all.update(cx, |this, cx| this.hide_everywhere(cx))
                                }),
                        ),
                )
            })
            .child(item_list(&view, cx))
            .into_any_element()
    }
}

/// Whether the editor pane is wide enough for the margin (`width` in
/// points).
pub fn pane_has_room(width: f32, inspector_open: bool) -> bool {
    width >= MIN_PANE && !inspector_open
}

#[cfg(test)]
mod tests {
    use super::{MIN_PANE, pane_has_room};
    use proptest::prelude::*;

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 64,
            failure_persistence: None,
            rng_seed: proptest::test_runner::RngSeed::Fixed(0x0072_6d67),
            ..ProptestConfig::default()
        }
    }

    proptest! {
        #![proptest_config(config())]

        /// The margin needs a wide pane and a shut inspector.
        #[test]
        fn prop_the_margin_needs_room(width in 0f32..3000., open in any::<bool>()) {
            prop_assert_eq!(pane_has_room(width, open), width >= MIN_PANE && !open);
        }
    }
}
