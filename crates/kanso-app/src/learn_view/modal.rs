//! Make Flashcards and Quiz Me as dialogs over the note, opened with the
//! same `open_alert_dialog` the Share and update sheets use. The note stays
//! where it is, dimmed behind the dialog: nothing in it is closed, moved or
//! re-created, and the dialog gives the keyboard back to whatever held it
//! when it closes, so the caret and scroll position are as they were.
//!
//! The dialog's own entrance (a short fade and settle, skipped when the
//! system asks for reduced motion) is the only animation.

use gpui_kit::component::dialog::DialogEntrance;

use super::*;

/// Quiz Me's card.
pub const QUIZ_WIDTH: f32 = 560.;
/// Make Flashcards, wide enough to read and edit a card.
pub const CARDS_WIDTH: f32 = 720.;
/// How much of the window's height Make Flashcards may take.
const HEIGHT_SHARE: f32 = 0.8;
/// Height of the title, footer and padding around the dialog's body.
const CHROME_HEIGHT: f32 = 150.;

/// The width a dialog for `mode` is opened at.
pub fn width_of(mode: Mode) -> f32 {
    match mode {
        Mode::Make => CARDS_WIDTH,
        Mode::Quiz | Mode::Draft => QUIZ_WIDTH,
    }
}

impl LearnView {
    /// Marks the view as a dialog's body.
    fn mark_modal(&mut self) {
        self.modal = true;
    }

    /// The dialog's body: the same content as the page, scrolling inside
    /// the share of the window the dialog may take.
    pub(super) fn render_modal(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let cap = (window.viewport_size().height * HEIGHT_SHARE - px(CHROME_HEIGHT)).max(px(160.));
        let body = self.body(window, cx);
        v_flex()
            .id("learn-modal")
            .debug_selector(|| "learn-modal".into())
            .key_context("LearnView")
            .track_focus(&self.focus)
            .w_full()
            .child(
                div()
                    .id("learn-body")
                    .w_full()
                    .max_h(cap)
                    .overflow_y_scrollbar()
                    .child(body),
            )
            .into_any_element()
    }
}

/// The dialog's buttons, kept in step with the view they belong to.
struct LearnFooter {
    view: Entity<LearnView>,
    _observing: Subscription,
}

impl Render for LearnFooter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let handle = self.view.clone();
        let this = self.view.read(cx);
        let hint = this.hint();
        let close = Button::new("learn-close")
            .outline()
            .small()
            .label("Close")
            .on_click(|_, window, cx| window.close_dialog(cx));
        let primary = match (this.mode, this.phase) {
            (Mode::Make, Phase::Ready) => Some(match (this.added, this.ids.first()) {
                (Some(_), Some(id)) => {
                    let (id, view) = (id.clone(), handle.clone());
                    Button::new("learn-open")
                        .primary()
                        .small()
                        .icon(Lucide::FileText)
                        .label("Open Note")
                        .on_click(move |_, _, cx| {
                            view.update(cx, |_, cx| cx.emit(LearnEvent::Open(id.clone())));
                        })
                }
                _ => {
                    let view = handle.clone();
                    Button::new("learn-add")
                        .primary()
                        .small()
                        .icon(Lucide::Plus)
                        .label(add_label(&this.picks))
                        .disabled(chosen(&this.picks).is_empty())
                        .tooltip("Write the ticked cards under Flashcards in the note")
                        .on_click(move |_, window, cx| {
                            view.update(cx, |this, cx| this.add_cards(window, cx));
                        })
                }
            }),
            (Mode::Quiz, Phase::Ready) if this.quiz.is_done() => {
                let view = handle.clone();
                Some(
                    Button::new("learn-again")
                        .primary()
                        .small()
                        .icon(Lucide::RefreshCw)
                        .label("Quiz Again")
                        .on_click(move |_, _, cx| {
                            view.update(cx, |this, cx| this.start(cx));
                        }),
                )
            }
            _ => None,
        };
        h_flex()
            .w_full()
            .gap(px(8.))
            .items_center()
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .text_size(px(12.))
                    .text_color(colors.text_muted)
                    .child(hint),
            )
            .child(close)
            .children(primary)
    }
}

/// What a window keeps of the Make Flashcards or Quiz Me dialog it shows.
/// The main window and every note window have one, so each opens the
/// dialog over its own note and never stacks a second one on the first.
#[derive(Default)]
pub(crate) struct ModalSlot {
    view: Option<WeakEntity<LearnView>>,
    /// Quiz Me is up: the window hides what is under the dialog.
    cover: bool,
    _subscriptions: Vec<Subscription>,
}

impl ModalSlot {
    /// The view in the open dialog, if one is open.
    pub(crate) fn view(&self) -> Option<Entity<LearnView>> {
        self.view.as_ref().and_then(|view| view.upgrade())
    }

    /// What the window draws over everything it has, under the dialog,
    /// while Quiz Me is open: the window's own paper, fully opaque. The
    /// note holds the answers, and the standard dialog overlay leaves it
    /// readable; even 96% paper left the text legible on a bright screen.
    pub(crate) fn cover(&self, cx: &App) -> Option<Div> {
        self.cover.then(|| {
            let paper = Colors::get(cx).app;
            div()
                .debug_selector(|| "learn-quiz-cover".into())
                .absolute()
                .inset_0()
                .bg(paper)
        })
    }
}

/// Opens Make Flashcards or Quiz Me for the note `id` as a dialog in
/// `window`, kept in the `slot` of `this`, the view that owns the window.
/// `on_open` runs, once the dialog has closed, when its Open Note button is
/// pressed. With one already open, that one is returned and nothing new
/// opens, so two ways in can't stack two dialogs.
pub(crate) fn open<T: 'static>(
    this: &mut T,
    slot: fn(&mut T) -> &mut ModalSlot,
    mode: Mode,
    id: String,
    window: &mut Window,
    cx: &mut Context<T>,
    on_open: impl Fn(&mut T, &str, &mut Window, &mut Context<T>) + 'static,
) -> Option<Entity<LearnView>> {
    if !cx.global::<AppState>().config.ai.enabled {
        window.push_notification(
            Notification::info("Turn on AI in Settings › AI to use this."),
            cx,
        );
        return None;
    }
    // A view still alive with no dialog up (something kept a handle to it)
    // is stale: open a new one rather than refuse.
    if let Some(view) = slot(this).view()
        && window.has_active_dialog(cx)
    {
        return Some(view);
    }
    let store = cx.global::<AppState>().store.clone();
    let view = cx.new(|cx| {
        let mut view = LearnView::new(store, window, cx);
        view.mark_modal();
        view
    });
    let opened = cx.subscribe_in(
        &view,
        window,
        move |this, _, event: &LearnEvent, window, cx| match event {
            LearnEvent::Open(id) => {
                window.close_dialog(cx);
                on_open(this, id, window, cx);
            }
        },
    );
    // The dialog owns the view: when it closes, the view goes, and with it
    // the cover.
    let closed = cx.observe_release(&view, move |this, _, cx| {
        let slot = slot(this);
        slot.view = None;
        slot.cover = false;
        cx.notify();
    });
    *slot(this) = ModalSlot {
        view: Some(view.downgrade()),
        cover: mode == Mode::Quiz,
        _subscriptions: vec![opened, closed],
    };
    view.update(cx, |view, cx| view.show(mode, vec![id], window, cx));
    let footer = cx.new(|cx| LearnFooter {
        view: view.clone(),
        _observing: cx.observe(&view, |_, _, cx| cx.notify()),
    });
    let body = view.clone();
    window.open_alert_dialog(cx, move |alert, window, _| {
        let height = window.viewport_size().height;
        // Centred in the window for the short card; the tall one keeps
        // a tenth of the window above it and takes up to 0.8 of it.
        let top = match mode {
            Mode::Make => height * 0.1,
            _ => height * 0.16,
        };
        alert
            .title(mode.title())
            .width(px(width_of(mode)))
            .margin_top(top)
            .close_button(true)
            .entrance(DialogEntrance::FadeSlide)
            .child(body.clone())
            .footer(footer.clone())
            .on_cancel(|_, _, _| true)
    });
    cx.notify();
    Some(view)
}

impl Workspace {
    /// Opens Make Flashcards or Quiz Me for the note `id` as a dialog over
    /// the open note. Returns the view in the dialog, for tests and
    /// snapshots to drive.
    pub(crate) fn show_learn_modal(
        &mut self,
        mode: Mode,
        id: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<Entity<LearnView>> {
        // The model reads the saved note, so what was just typed is saved.
        self.flush(cx);
        open(
            self,
            |this| &mut this.learn_modal,
            mode,
            id,
            window,
            cx,
            |this, id, window, cx| {
                let id = id.to_owned();
                this.library
                    .update(cx, |library, cx| library.select(Some(id), cx));
                this.editor
                    .update(cx, |editor, cx| editor.focus_body(window, cx));
            },
        )
    }
}

#[cfg(test)]
impl LearnView {
    /// Shows a finished state for snapshots, with no model involved: the
    /// quiz with its first answer graded, or the cards with the second one
    /// open for editing.
    pub(crate) fn snapshot_state(
        &mut self,
        second: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        use kanso_ai::draft_model::{Graded, QuizQuestion, Suggestion, Verdict};

        self.stop();
        self.phase = Phase::Ready;
        if self.mode == Mode::Make {
            let card = |question: &str, answer: &str, quote: &str, line, missing| Suggestion {
                question: question.into(),
                answer: answer.into(),
                source: Some(quote.into()),
                line,
                source_missing: missing,
            };
            self.picks = picks_from(vec![
                card(
                    "What did we decide about pricing?",
                    "The yearly plan, at 40 dollars.",
                    "We chose the yearly plan at 40 dollars.",
                    Some(3),
                    false,
                ),
                card(
                    "Why not the monthly plan?",
                    "It churned too fast.",
                    "Sam argued the monthly plan churned too fast.",
                    Some(4),
                    false,
                ),
                card(
                    "Who approved the launch price?",
                    "Priya.",
                    "Priya signed it off in the call.",
                    None,
                    true,
                ),
            ]);
            if second {
                self.begin_edit(1, window, cx);
            }
        } else {
            self.excerpts = vec![kanso_ai::prompt::Excerpt {
                id: "note:pricing".into(),
                title: "Pricing decision".into(),
                ..Default::default()
            }];
            self.quiz = Quiz::new(vec![QuizQuestion {
                question: "What did we decide about pricing, and why?".into(),
                excerpt: 1,
                source: None,
            }]);
            if second {
                self.quiz.grade(Graded {
                    verdict: Verdict::Partly,
                    comment: "You have the plan, but not why the monthly one lost (line 4).".into(),
                });
            }
        }
        cx.notify();
    }
}
