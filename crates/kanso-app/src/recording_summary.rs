//! Summarise Recording (issue #38, step 12): the sheet that sends a
//! recording's transcript to the person's own AI and, once they have looked
//! it over, writes a summary and the action items they ticked above the
//! transcript.
//!
//! Nothing here runs by itself. The command opens a sheet that names where
//! the transcript goes and what is not sent ("The audio is not sent. The
//! transcript is."); only **Send** asks the model. What comes back is a
//! review, in the manner of the agent review queue: the summary and the
//! decisions as written, and each action item as a tick box, all ticked to
//! start. Only **Apply** writes, once, through
//! `kanso_engine::recording_summary::apply_summary`, and the notice has
//! Undo. A failure, or Cancel, leaves the note as it was.
//!
//! Every decision and action item has already been checked against the
//! transcript by `ground`: one the model could not quote is not offered.

use std::collections::BTreeSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::component::notification::Notification;
use gpui_kit::component::{
    Disableable as _, Sizable as _, StyledExt as _, WindowExt as _, h_flex, v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_engine::recording_note::stamp;
use kanso_engine::recording_summary::{
    SummaryDraft, SummaryError, SummaryItem, SummarySource, apply_summary, ground, send_words,
    summary_source,
};
use kanso_store::Origin;

use crate::ai_feature;
use crate::config::AiBackend;
use crate::theme::Colors;
use crate::workspace::AppState;
use crate::zoom::px;

/// The width of the sheet.
pub const WIDTH: f32 = 560.;
/// The most of the review that shows before it scrolls.
const REVIEW_HEIGHT: f32 = 400.;

/// Where the sheet is.
#[derive(Clone, Debug, PartialEq)]
pub enum Phase {
    /// Naming where the transcript goes; nothing has been sent.
    Confirm,
    /// The model is working.
    Working,
    /// The model answered; the person is looking.
    Review,
    /// The model could not be asked, or its answer was no use.
    Failed(String),
}

/// The sheet's state.
pub struct SummarySheet {
    note_id: String,
    source: SummarySource,
    /// What the Confirm phase says.
    send_words: String,
    phase: Phase,
    draft: Option<SummaryDraft>,
    /// The action items that will be written.
    ticked: BTreeSet<usize>,
    cancel: Arc<AtomicBool>,
}

impl SummarySheet {
    fn new(note_id: String, source: SummarySource, send_words: String) -> Self {
        SummarySheet {
            note_id,
            source,
            send_words,
            phase: Phase::Confirm,
            draft: None,
            ticked: BTreeSet::new(),
            cancel: Arc::new(AtomicBool::new(false)),
        }
    }

    #[cfg(test)]
    pub fn phase(&self) -> &Phase {
        &self.phase
    }

    /// What the Confirm phase says.
    #[cfg(test)]
    pub fn confirm_text(&self) -> &str {
        &self.send_words
    }

    /// Asks the model, on a worker. The note is read again through the gate
    /// first: one locked, trashed or tagged `#ai-exclude` since the sheet
    /// opened sends nothing.
    pub(crate) fn send(&mut self, cx: &mut Context<Self>) {
        if self.phase != Phase::Confirm && !matches!(self.phase, Phase::Failed(_)) {
            return;
        }
        let store = cx.global::<AppState>().store.clone();
        match summary_source(&store, &self.note_id) {
            Ok(source) => self.source = source,
            Err(err) => {
                self.phase = Phase::Failed(err.to_string());
                cx.notify();
                return;
            }
        }
        self.phase = Phase::Working;
        self.cancel = Arc::new(AtomicBool::new(false));
        let maker = ai_feature::maker(cx);
        crate::metrics::record(cx, crate::metrics::Feature::Ai);
        let (tx, rx) = async_channel::bounded(1);
        let source = self.source.clone();
        let cancel = self.cancel.clone();
        let spawned = std::thread::Builder::new()
            .name("kanso-recording-summary".into())
            .spawn(move || {
                let result = ai_feature::summarize_recording(
                    &maker,
                    &source.title,
                    source.date.as_deref(),
                    &source.prompt_text,
                    &cancel,
                )
                .map(|tidy| ground(&source, &tidy));
                tx.try_send(result).ok();
            });
        if spawned.is_err() {
            self.phase = Phase::Failed("Couldn't start.".to_owned());
            cx.notify();
            return;
        }
        cx.spawn(async move |this, cx| {
            let result = rx
                .recv()
                .await
                .unwrap_or_else(|_| Err("Stopped.".to_owned()));
            this.update(cx, |this, cx| this.arrived(result, cx)).ok();
        })
        .detach();
        cx.notify();
    }

    fn arrived(&mut self, result: Result<SummaryDraft, String>, cx: &mut Context<Self>) {
        if self.phase != Phase::Working {
            return;
        }
        match result {
            Ok(draft) => {
                let nothing = draft.overview.is_empty()
                    && draft.decisions.is_empty()
                    && draft.actions.is_empty();
                if nothing {
                    self.phase = Phase::Failed(SummaryError::Empty.to_string());
                } else {
                    self.ticked = (0..draft.actions.len()).collect();
                    self.draft = Some(draft);
                    self.phase = Phase::Review;
                }
            }
            Err(reason) => {
                self.phase = Phase::Failed(crate::capture_voice::trouble_words(&reason));
            }
        }
        cx.notify();
    }

    fn toggle(&mut self, index: usize, on: bool, cx: &mut Context<Self>) {
        if on {
            self.ticked.insert(index);
        } else {
            self.ticked.remove(&index);
        }
        cx.notify();
    }

    /// Stops the request and closes the sheet.
    fn stop(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.cancel.store(true, Ordering::Relaxed);
        window.close_dialog(cx);
    }

    /// Writes what was approved, once.
    fn apply(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(draft) = self.draft.as_ref().filter(|_| self.phase == Phase::Review) else {
            return;
        };
        let store = cx.global::<AppState>().store.clone();
        // Typing waiting in an editor is saved first, so the write is made
        // against the note as it is.
        crate::editor::peers::flush_everywhere(cx);
        match apply_summary(&store, &self.note_id, draft, &self.ticked, Origin::Ui) {
            Ok(applied) => {
                window.close_dialog(cx);
                let words = if applied.replaced {
                    "Summary replaced. Undo, or the note's history, brings the old one back."
                } else {
                    "Summary added above the transcript. Undo, or the note's history, takes it out."
                };
                crate::note_batch::say_with_undo(
                    &store,
                    vec![applied.edit],
                    words.to_owned(),
                    window,
                    cx,
                );
            }
            Err(err) => {
                let words = match err {
                    SummaryError::Changed => {
                        "The note changed while it was being summarised. Try again.".to_owned()
                    }
                    other => other.to_string(),
                };
                window.push_notification(Notification::error(words), cx);
            }
        }
    }

    /// Ticks or unticks an action item, as a click on its box does.
    #[cfg(test)]
    pub(crate) fn toggle_for_test(&mut self, index: usize, on: bool, cx: &mut Context<Self>) {
        self.toggle(index, on, cx);
    }

    /// Presses Apply.
    #[cfg(test)]
    pub(crate) fn apply_for_test(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.apply(window, cx);
    }

    fn item_meta(item: &SummaryItem) -> String {
        let mut parts: Vec<String> = Vec::new();
        parts.extend(item.who.clone());
        parts.extend(item.when.clone());
        parts.push(stamp(item.moment));
        parts.join(" · ")
    }
}

fn heading(text: &'static str, colors: &Colors) -> Div {
    div()
        .text_size(px(12.5))
        .font_medium()
        .text_color(colors.text_secondary)
        .child(text)
}

fn quiet(text: impl Into<SharedString>, colors: &Colors) -> Div {
    div()
        .text_size(px(12.5))
        .line_height(px(18.))
        .text_color(colors.text_muted)
        .child(text.into())
}

impl Render for SummarySheet {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let body = |text: String| {
            div()
                .text_size(px(14.))
                .line_height(px(21.))
                .text_color(colors.text)
                .child(text)
        };
        match &self.phase {
            Phase::Confirm => v_flex()
                .w_full()
                .gap(px(10.))
                .child(body(self.send_words.clone()))
                .child(quiet(
                    "Nothing is sent until you choose Send. You will see the summary and the action items before anything is written.",
                    colors,
                ))
                .when(self.source.replaces, |el| {
                    el.child(quiet(
                        "This note already has a summary. A new one replaces it.",
                        colors,
                    ))
                })
                .into_any_element(),
            Phase::Working => v_flex()
                .w_full()
                .gap(px(10.))
                .child(body("Summarising the transcript…".to_owned()))
                .child(quiet(
                    "Cancel stops the request. The note is not changed.",
                    colors,
                ))
                .into_any_element(),
            Phase::Failed(reason) => v_flex()
                .w_full()
                .gap(px(10.))
                .child(
                    div()
                        .text_size(px(13.5))
                        .line_height(px(20.))
                        .text_color(colors.warning)
                        .debug_selector(|| "recording-summary-error".into())
                        .child(format!("Couldn't summarise this recording. {reason}")),
                )
                .child(quiet("The note is not changed.", colors))
                .into_any_element(),
            Phase::Review => {
                let Some(draft) = &self.draft else {
                    return div().into_any_element();
                };
                let mut review = v_flex().w_full().gap(px(14.));
                if !draft.overview.is_empty() {
                    let mut points = v_flex().gap(px(4.));
                    for point in &draft.overview {
                        points = points.child(body(format!("•  {point}")));
                    }
                    review = review.child(
                        v_flex()
                            .gap(px(6.))
                            .child(heading("Summary", colors))
                            .child(points),
                    );
                }
                if !draft.decisions.is_empty() {
                    let mut lines = v_flex().gap(px(4.));
                    for item in &draft.decisions {
                        lines = lines.child(
                            h_flex()
                                .gap(px(8.))
                                .items_baseline()
                                .child(div().flex_1().min_w_0().child(body(item.text.clone())))
                                .child(quiet(stamp(item.moment), colors)),
                        );
                    }
                    review = review.child(
                        v_flex()
                            .gap(px(6.))
                            .child(heading("Decisions", colors))
                            .child(lines),
                    );
                }
                if !draft.actions.is_empty() {
                    let mut rows = v_flex().gap(px(8.));
                    for (index, item) in draft.actions.iter().enumerate() {
                        rows = rows.child(
                            v_flex()
                                .gap(px(2.))
                                .child(
                                    Checkbox::new(("recording-summary-action", index))
                                        .checked(self.ticked.contains(&index))
                                        .label(item.text.clone())
                                        .on_click(cx.listener(move |this, on: &bool, _, cx| {
                                            this.toggle(index, *on, cx)
                                        })),
                                )
                                .child(
                                    div()
                                        .pl(px(26.))
                                        .child(quiet(Self::item_meta(item), colors)),
                                ),
                        );
                    }
                    review = review.child(
                        v_flex()
                            .gap(px(6.))
                            .child(heading("Action items", colors))
                            .child(quiet(
                                "Only the ticked ones are written into the note.",
                                colors,
                            ))
                            .child(rows),
                    );
                }
                let mut notes: Vec<String> = Vec::new();
                if draft.dropped > 0 {
                    notes.push(format!(
                        "{} left out for want of a quote from the recording.",
                        if draft.dropped == 1 {
                            "One item was".to_owned()
                        } else {
                            format!("{} items were", draft.dropped)
                        }
                    ));
                }
                if draft.cut {
                    notes.push(
                        "The transcript was long, so only its first part was summarised.".into(),
                    );
                }
                if self.source.replaces {
                    notes.push("This replaces the summary already in the note.".into());
                }
                v_flex()
                    .w_full()
                    .gap(px(10.))
                    .child(
                        div()
                            .id("recording-summary-review")
                            .max_h(px(REVIEW_HEIGHT))
                            .overflow_y_scroll()
                            .child(review),
                    )
                    .children(notes.into_iter().map(|n| quiet(n, colors)))
                    .into_any_element()
            }
        }
    }
}

/// The sheet's buttons, which follow its phase.
struct SummaryFooter {
    view: Entity<SummarySheet>,
    _observing: Subscription,
}

impl Render for SummaryFooter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sheet = self.view.read(cx);
        let phase = sheet.phase.clone();
        let nothing_ticked = sheet.ticked.is_empty()
            && sheet
                .draft
                .as_ref()
                .is_some_and(|d| d.overview.is_empty() && d.decisions.is_empty());
        let view = self.view.clone();
        let row = h_flex().w_full().gap_2().justify_end();
        match phase {
            Phase::Confirm => row
                .child(
                    Button::new("recording-summary-cancel")
                        .small()
                        .label("Cancel")
                        .on_click(|_, window, cx| window.close_dialog(cx)),
                )
                .child(
                    Button::new("recording-summary-send")
                        .debug_selector(|| "recording-summary-send".into())
                        .small()
                        .primary()
                        .label("Send")
                        .on_click(move |_, _, cx| view.update(cx, |sheet, cx| sheet.send(cx))),
                ),
            Phase::Working => row.child(
                Button::new("recording-summary-stop")
                    .small()
                    .label("Cancel")
                    .on_click(move |_, window, cx| {
                        view.update(cx, |sheet, cx| sheet.stop(window, cx))
                    }),
            ),
            Phase::Review => row
                .child(
                    Button::new("recording-summary-discard")
                        .small()
                        .label("Discard")
                        .on_click(|_, window, cx| window.close_dialog(cx)),
                )
                .child(
                    Button::new("recording-summary-apply")
                        .debug_selector(|| "recording-summary-apply".into())
                        .small()
                        .primary()
                        .label("Apply")
                        .disabled(nothing_ticked)
                        .on_click(move |_, window, cx| {
                            view.update(cx, |sheet, cx| sheet.apply(window, cx))
                        }),
                ),
            Phase::Failed(_) => row
                .child(
                    Button::new("recording-summary-close")
                        .small()
                        .label("Close")
                        .on_click(|_, window, cx| window.close_dialog(cx)),
                )
                .child(
                    Button::new("recording-summary-retry")
                        .small()
                        .primary()
                        .label("Try again")
                        .on_click(move |_, _, cx| view.update(cx, |sheet, cx| sheet.send(cx))),
                ),
        }
    }
}

/// What the Confirm phase says, from the settings as they are now.
fn confirm_words(source: &SummarySource, cx: &App) -> String {
    let ai = &cx.global::<AppState>().config.ai;
    let agent = (ai.backend == AiBackend::Agent).then(|| crate::ai_agent::label(&ai.agent));
    send_words(source, ai.provider, &ai.base_url, agent.as_deref())
}

/// Opens the sheet for note `id`, or says why it can't.
// no-prop-test: glue over `summary_source`, `ground` and `apply_summary`,
// which have the properties.
pub fn open(id: String, window: &mut Window, cx: &mut App) -> Option<Entity<SummarySheet>> {
    if !cx.global::<AppState>().config.ai.enabled {
        return None;
    }
    let store = cx.global::<AppState>().store.clone();
    let source = match summary_source(&store, &id) {
        Ok(source) => source,
        Err(err) => {
            let words = match err {
                SummaryError::NoTranscript => {
                    "Put a recording with a transcript in this note first: Record…, or Transcribe Recording on a recording you dropped in.".to_owned()
                }
                other => other.to_string(),
            };
            window.push_notification(Notification::info(words), cx);
            return None;
        }
    };
    let words = confirm_words(&source, cx);
    Some(open_with(id, source, words, window, cx))
}

fn open_with(
    id: String,
    source: SummarySource,
    words: String,
    window: &mut Window,
    cx: &mut App,
) -> Entity<SummarySheet> {
    let sheet = cx.new(|_| SummarySheet::new(id, source, words));
    show(sheet, window, cx)
}

fn show(sheet: Entity<SummarySheet>, window: &mut Window, cx: &mut App) -> Entity<SummarySheet> {
    let footer = cx.new(|cx| SummaryFooter {
        view: sheet.clone(),
        _observing: cx.observe(&sheet, |_, _, cx| cx.notify()),
    });
    let body = sheet.clone();
    let running = sheet.clone();
    window.open_alert_dialog(cx, move |alert, _, _| {
        let running = running.clone();
        alert
            .title("Summarise Recording")
            .width(px(WIDTH))
            .child(body.clone())
            .footer(footer.clone())
            // Escape stops a request in flight, as Cancel does.
            .on_cancel(move |_, _, cx| {
                running.update(cx, |sheet, _| sheet.cancel.store(true, Ordering::Relaxed));
                true
            })
    });
    sheet
}

/// Opens the sheet in `phase` over fixed content, for the snapshots.
// no-prop-test: drawing and glue.
#[cfg(test)]
pub fn open_for_snapshot(
    phase: Phase,
    source: SummarySource,
    draft: Option<SummaryDraft>,
    words: String,
    window: &mut Window,
    cx: &mut App,
) -> Entity<SummarySheet> {
    let sheet = cx.new(|_| {
        let mut sheet = SummarySheet::new("note:snapshot".into(), source, words);
        sheet.ticked = draft
            .as_ref()
            .map(|d| (0..d.actions.len()).collect())
            .unwrap_or_default();
        sheet.draft = draft;
        sheet.phase = phase;
        sheet
    });
    show(sheet, window, cx)
}
