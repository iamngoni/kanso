//! Review on the desktop (issue #198): the cards that are due, one at a
//! time, over the window, and the stats of the log.
//!
//! The cards are lines of the notes (`question :: answer`); their schedule
//! is the store's, replayed from the synced log of answers. This is the
//! window's side: drawing, keys and the clock. Space shows the answer,
//! 1 to 4 (or Space again, for Good) record it, E opens the note at the
//! card, S suspends the card, Tab switches to Stats, Esc closes.

use std::sync::Arc;

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{Icon, Sizable as _, StyledExt as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_core::srs::Grade;
use kanso_store::{CardScope, CardStats, Store};

use crate::clock::Clock;
use crate::flashcards_model::{Command, Session, button_labels, command_for};
use crate::layout::{BOTTOM_BAND_HEIGHT, TOP_BAND_HEIGHT, inset_after};
use crate::note_list::quiet_empty;
use crate::theme::Colors;
use crate::workspace::{AppState, Workspace};
use crate::zoom::px;

actions!(kanso, [ShowFlashcards, CloseFlashcards]);

/// What the view asks of the workspace.
pub enum FlashcardsEvent {
    /// Open the note in the editor, closing Review.
    Open(String),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Review,
    Stats,
}

pub struct FlashcardsView {
    store: Arc<Store>,
    scope: CardScope,
    session: Session,
    intervals: [i64; 4],
    stats: Option<CardStats>,
    tab: Tab,
    /// Drawn on screen.
    pub visible: bool,
    loading: bool,
    error: Option<String>,
    device: String,
    focus: FocusHandle,
}

impl EventEmitter<FlashcardsEvent> for FlashcardsView {}

impl Focusable for FlashcardsView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

/// Now, and the start of the person's day, in the clock the views use.
fn clock(cx: &App) -> (i64, i64) {
    let now = Clock::now_ms(cx);
    let offset_minutes = Clock::utc_offset_secs(cx, now) / 60;
    (
        now,
        kanso_engine::flashcards::day_start_ms(now, offset_minutes),
    )
}

impl FlashcardsView {
    pub fn new(store: Arc<Store>, cx: &mut Context<Self>) -> Self {
        let device = store
            .meta("cards.device")
            .ok()
            .flatten()
            .unwrap_or_else(|| {
                let id = format!("d{:012x}", fastrand::u64(..) >> 16);
                store.set_meta("cards.device", &id).ok();
                id
            });
        FlashcardsView {
            store,
            scope: CardScope::All,
            session: Session::default(),
            intervals: [0; 4],
            stats: None,
            tab: Tab::Review,
            visible: false,
            loading: false,
            error: None,
            device,
            focus: cx.focus_handle(),
        }
    }

    /// Opens Review on `scope`: the due cards are read again.
    pub fn show(&mut self, scope: CardScope, cx: &mut Context<Self>) {
        self.scope = scope;
        self.visible = true;
        self.tab = Tab::Review;
        self.reload(cx);
    }

    fn reload(&mut self, cx: &mut Context<Self>) {
        let (now, day) = clock(cx);
        self.loading = false;
        match self.store.due_cards(&self.scope, now, day) {
            Ok(cards) => {
                self.session = Session::new(cards);
                self.error = None;
            }
            Err(err) => {
                self.session = Session::default();
                self.error = Some(err.to_string());
            }
        }
        self.stats = self.store.card_stats(&self.scope, now, day).ok();
        self.refresh_intervals(cx);
        cx.notify();
    }

    fn refresh_intervals(&mut self, cx: &App) {
        let (now, _) = clock(cx);
        self.intervals = self
            .session
            .current()
            .and_then(|card| self.store.card_intervals(&card.card_key, now).ok())
            .unwrap_or([0; 4]);
    }

    fn run(&mut self, command: Command, window: &mut Window, cx: &mut Context<Self>) {
        match command {
            Command::ShowAnswer => {
                self.session.show_answer();
                cx.notify();
            }
            Command::Answer(grade) => self.answer(grade, cx),
            Command::Edit => {
                if let Some(card) = self.session.current() {
                    cx.emit(FlashcardsEvent::Open(card.note_id.clone()));
                }
            }
            Command::Suspend => self.suspend(cx),
            Command::ToggleStats => {
                self.tab = match self.tab {
                    Tab::Review => Tab::Stats,
                    Tab::Stats => Tab::Review,
                };
                let (now, day) = clock(cx);
                self.stats = self.store.card_stats(&self.scope, now, day).ok();
                cx.notify();
            }
            Command::Close => window.dispatch_action(Box::new(CloseFlashcards), cx),
        }
    }

    fn answer(&mut self, grade: u8, cx: &mut Context<Self>) {
        let Some(card) = self.session.current().cloned() else {
            return;
        };
        let Some(g) = Grade::from_number(i64::from(grade)) else {
            return;
        };
        let (now, _) = clock(cx);
        match self
            .store
            .record_review(&card.note_id, &card.card_key, g, &self.device, now)
        {
            Ok(_) => {
                self.session.answered_with(grade);
                self.error = None;
            }
            Err(err) => self.error = Some(err.to_string()),
        }
        self.refresh_intervals(cx);
        cx.notify();
    }

    fn suspend(&mut self, cx: &mut Context<Self>) {
        let Some(card) = self.session.current().cloned() else {
            return;
        };
        let (now, _) = clock(cx);
        match self
            .store
            .suspend_card(&card.note_id, &card.card_key, true, now)
        {
            Ok(()) => {
                self.session.suspended();
                self.error = None;
            }
            Err(err) => self.error = Some(err.to_string()),
        }
        self.refresh_intervals(cx);
        cx.notify();
    }

    fn render_toolbar(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let title = match self.tab {
            Tab::Review => "Review".to_owned(),
            Tab::Stats => "Review · Stats".to_owned(),
        };
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
                    .child(title),
            )
            .child(div().flex_1())
            .child(
                Button::new("review-stats")
                    .ghost()
                    .small()
                    .label(if self.tab == Tab::Review {
                        "Stats"
                    } else {
                        "Cards"
                    })
                    .tooltip("Switch between the cards and the stats (Tab)")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.run(Command::ToggleStats, window, cx)
                    })),
            )
            .child(
                Button::new("review-close")
                    .ghost()
                    .small()
                    .icon(Lucide::Close)
                    .tooltip("Close (Esc)")
                    .on_click(cx.listener(|_, _, window, cx| {
                        window.dispatch_action(Box::new(CloseFlashcards), cx)
                    })),
            )
    }

    fn render_card(&self, cx: &mut Context<Self>) -> AnyElement {
        let colors = Colors::get(cx);
        let Some(card) = self.session.current() else {
            let none = self.session.answered() == 0;
            return quiet_empty(
                Icon::new(Lucide::Check),
                if let Some(err) = &self.error {
                    err.clone()
                } else if none {
                    "No cards due. Write a line like  What is FSRS? :: A scheduler  in any note."
                        .to_owned()
                } else {
                    "Done for today.".to_owned()
                },
                (!none).then(|| "Review More (R) pulls the cards due soon.".to_owned()),
                colors,
            )
            .into_any_element();
        };
        let shown = self.session.answer_shown();
        let labels = button_labels(&self.intervals);
        v_flex()
            .size_full()
            .items_center()
            .justify_center()
            .gap(px(18.))
            .px(px(40.))
            .child(
                div()
                    .text_size(px(12.))
                    .text_color(colors.text_muted)
                    .child(self.session.progress()),
            )
            .child(
                div()
                    .max_w(px(640.))
                    .text_size(px(22.))
                    .font_medium()
                    .text_color(colors.text)
                    .child(card.question.clone()),
            )
            .when(shown, |el| {
                el.child(
                    div()
                        .max_w(px(640.))
                        .text_size(px(16.))
                        .text_color(colors.text_secondary)
                        .child(card.answer.clone()),
                )
                .child(
                    div()
                        .id("review-source")
                        .cursor_pointer()
                        .text_size(px(12.))
                        .text_color(colors.link)
                        .child(card.note_title.clone())
                        .on_click(
                            cx.listener(|this, _, window, cx| this.run(Command::Edit, window, cx)),
                        ),
                )
            })
            .child(if shown {
                h_flex()
                    .gap(px(8.))
                    .children(labels.into_iter().enumerate().map(|(i, label)| {
                        Button::new(("review-grade", i))
                            .when(i == 2, |b| b.primary())
                            .outline()
                            .small()
                            .label(format!("{label}  ({})", i + 1))
                            .on_click(
                                cx.listener(move |this, _, _, cx| this.answer(i as u8 + 1, cx)),
                            )
                    }))
                    .into_any_element()
            } else {
                Button::new("review-show")
                    .primary()
                    .label("Show Answer (Space)")
                    .on_click(
                        cx.listener(|this, _, window, cx| {
                            this.run(Command::ShowAnswer, window, cx)
                        }),
                    )
                    .into_any_element()
            })
            .when_some(self.error.clone(), |el, err| {
                el.child(
                    div()
                        .text_size(px(12.))
                        .text_color(colors.danger)
                        .child(err),
                )
            })
            .into_any_element()
    }

    fn render_stats(&self, cx: &mut Context<Self>) -> AnyElement {
        let colors = Colors::get(cx);
        let Some(stats) = &self.stats else {
            return quiet_empty(
                Icon::new(Lucide::Check),
                "No stats yet.".to_owned(),
                None,
                colors,
            )
            .into_any_element();
        };
        let figure = |label: &'static str, value: usize| {
            v_flex()
                .flex_1()
                .items_center()
                .child(
                    div()
                        .text_size(px(22.))
                        .font_medium()
                        .text_color(colors.text)
                        .child(value.to_string()),
                )
                .child(
                    div()
                        .text_size(px(12.))
                        .text_color(colors.text_muted)
                        .child(label),
                )
        };
        let bars = |values: &[u32], color: Hsla| {
            let top = values.iter().copied().max().unwrap_or(0).max(1) as f32;
            h_flex()
                .h(px(64.))
                .items_end()
                .gap(px(2.))
                .children(values.iter().map(move |v| {
                    div()
                        .flex_1()
                        .h(px((*v as f32 / top * 64.).max(2.)))
                        .rounded(px(2.))
                        .bg(if *v == 0 { color.opacity(0.25) } else { color })
                }))
        };
        let heading = |text: &'static str| {
            div()
                .mt(px(18.))
                .mb(px(6.))
                .text_size(px(11.))
                .font_medium()
                .text_color(colors.text_muted)
                .child(text)
        };
        v_flex()
            .id("review-stats-body")
            .size_full()
            .overflow_y_scroll()
            .p(px(24.))
            .max_w(px(680.))
            .child(
                h_flex()
                    .child(figure("Cards", stats.counts.total))
                    .child(figure("Due today", stats.counts.due))
                    .child(figure("New", stats.counts.new))
                    .child(figure("Suspended", stats.counts.suspended)),
            )
            .child(heading("REVIEWS, LAST 30 DAYS"))
            .child(bars(&stats.per_day, Hsla::from(colors.accent)))
            .child(heading("RETENTION, LAST 30 DAYS"))
            .child(
                div()
                    .text_size(px(22.))
                    .text_color(colors.text)
                    .child(match stats.retention {
                        Some(r) => format!("{}%", (r * 100.).round()),
                        None => "Nothing reviewed yet.".to_owned(),
                    }),
            )
            .child(heading("DUE, NEXT 7 DAYS"))
            .child(bars(&stats.forecast, Hsla::from(colors.success)))
            .when(!stats.hardest.is_empty(), |el| {
                el.child(heading("HARDEST CARDS"))
                    .children(stats.hardest.iter().enumerate().map(|(i, card)| {
                        let id = card.note_id.clone();
                        h_flex()
                            .id(("review-hard", i))
                            .cursor_pointer()
                            .gap(px(8.))
                            .py(px(4.))
                            .child(
                                div()
                                    .flex_1()
                                    .truncate()
                                    .text_size(px(13.))
                                    .text_color(colors.text)
                                    .child(card.question.clone()),
                            )
                            .child(
                                div()
                                    .text_size(px(12.))
                                    .text_color(colors.text_muted)
                                    .child(format!("{} · ×{}", card.note_title, card.agains)),
                            )
                            .on_click(cx.listener(move |_, _, _, cx| {
                                cx.emit(FlashcardsEvent::Open(id.clone()))
                            }))
                    }))
            })
            .into_any_element()
    }
}

impl Render for FlashcardsView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let body = match self.tab {
            Tab::Review => self.render_card(cx),
            Tab::Stats => self.render_stats(cx),
        };
        v_flex()
            .id("flashcards-view")
            .key_context("FlashcardsView")
            .track_focus(&self.focus)
            .size_full()
            .bg(colors.editor)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                let k = &event.keystroke;
                if k.modifiers.platform || k.modifiers.control {
                    return;
                }
                if let Some(command) = command_for(&k.key, this.session.answer_shown()) {
                    cx.stop_propagation();
                    this.run(command, window, cx);
                } else if k.key == "r" && this.session.is_done() {
                    cx.stop_propagation();
                    this.reload(cx);
                }
            }))
            // Presses here are the view's, not the window's below.
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
                    .child(div().truncate().child(
                        "Space show answer · 1 Again · 2 Hard · 3 Good · 4 Easy · E open note · S suspend · Tab stats · Esc close",
                    )),
            )
    }
}

// MARK: Workspace

/// The workspace's part: opening and closing Review.
impl Workspace {
    pub(crate) fn show_flashcards(
        &mut self,
        scope: CardScope,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.close_settings(window, cx);
        self.close_graph(window, cx);
        self.close_board(window, cx);
        self.close_triage(window, cx);
        self.set_focus_mode(false, window, cx);
        self.flush(cx);
        let view = match &self.flashcards {
            Some(view) => view.clone(),
            None => {
                let store = cx.global::<AppState>().store.clone();
                let view = cx.new(|cx| FlashcardsView::new(store, cx));
                self._flashcards_events = Some(cx.subscribe_in(
                    &view,
                    window,
                    |this, _, event, window, cx| this.on_flashcards_event(event, window, cx),
                ));
                self.flashcards = Some(view.clone());
                view
            }
        };
        view.update(cx, |view, cx| view.show(scope, cx));
        self.flashcards_open = true;
        window.focus(&view.focus_handle(cx), cx);
        cx.notify();
    }

    pub(crate) fn close_flashcards(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.flashcards_open {
            return;
        }
        self.flashcards_open = false;
        if let Some(view) = &self.flashcards {
            view.update(cx, |view, _| view.visible = false);
        }
        window.focus(&self.focus, cx);
        cx.notify();
    }

    fn on_flashcards_event(
        &mut self,
        event: &FlashcardsEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            FlashcardsEvent::Open(id) => {
                self.close_flashcards(window, cx);
                let id = id.clone();
                self.library
                    .update(cx, |library, cx| library.select(Some(id), cx));
                self.editor
                    .update(cx, |editor, cx| editor.focus_body(window, cx));
            }
        }
    }
}

/// The workspace's actions for Review.
pub fn on_actions(el: Div, cx: &mut Context<Workspace>) -> Div {
    el.on_action(cx.listener(|this, _: &ShowFlashcards, window, cx| {
        this.show_flashcards(CardScope::All, window, cx)
    }))
    .on_action(
        cx.listener(|this, _: &CloseFlashcards, window, cx| this.close_flashcards(window, cx)),
    )
}

/// Review over the window, while open.
pub fn overlay(workspace: &Workspace) -> Option<impl IntoElement> {
    let view = workspace
        .flashcards
        .clone()
        .filter(|_| workspace.flashcards_open)?;
    Some(
        div()
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .debug_selector(|| "flashcards-overlay".into())
            .child(view),
    )
}
