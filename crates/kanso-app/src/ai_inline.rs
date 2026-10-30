//! The inline assistant in the editor: rewrite, shorten, fix, continue or
//! summarise the selected text.
//!
//! One suggestion is open at a time. It shows in a card above the text,
//! streaming in from the backend chosen in Settings, and changes nothing
//! until the person presses Accept. Accepting applies it in one undo step,
//! and only if the text it was made for is still there. The prompt carries
//! the selection and a little of the note before it; the model gets no
//! tools and no other note (`ai_feature.rs`).

use std::ops::Range;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::Disableable as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::{Icon, Sizable as _, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_ai::Event;
use kanso_ai::features::{self, InlineAction};

use super::{NoteEditor, TEXT_INSET};
use crate::ai_feature;
use crate::locked::LockView;
use crate::theme::Colors;
use crate::workspace::AppState;
use crate::zoom::px;

/// Runs an inline assistant action on the selection, from the context menu
/// or the palette.
#[derive(Clone, Debug, PartialEq, Action)]
#[action(namespace = kanso, no_json)]
pub struct InlineAssist {
    pub action: InlineAction,
}

/// What the card says when the text moved on while the suggestion was made.
pub const CHANGED: &str = "The text changed; the suggestion wasn't applied.";

/// One change to a note's text: `with` goes in place of `range`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Splice {
    pub range: Range<usize>,
    pub with: String,
}

impl Splice {
    /// `text` with this change made (the editor makes it itself, as an
    /// undoable edit).
    #[cfg(test)]
    pub fn apply(&self, text: &str) -> String {
        let mut out = String::with_capacity(text.len() + self.with.len());
        out.push_str(&text[..self.range.start]);
        out.push_str(&self.with);
        out.push_str(&text[self.range.end..]);
        out
    }
}

/// What accepting `suggestion` for `action` does to `text`, which had
/// `original` at `range` when the suggestion was asked for. A rewrite takes
/// the selection's place (its leading and trailing blank space stays); a
/// continuation or a summary goes after it, as a paragraph of its own.
/// Nothing when the text at `range` is no longer `original`, or the
/// suggestion is empty.
pub fn splice(
    text: &str,
    range: &Range<usize>,
    original: &str,
    action: InlineAction,
    suggestion: &str,
) -> Option<Splice> {
    if text.get(range.clone()) != Some(original) {
        return None;
    }
    let suggestion = suggestion.trim();
    if suggestion.is_empty() {
        return None;
    }
    if !action.replaces() {
        return Some(Splice {
            range: range.end..range.end,
            with: format!("\n\n{suggestion}"),
        });
    }
    let lead = original.len() - original.trim_start().len();
    let trail = original.trim_end().len().max(lead);
    Some(Splice {
        range: range.clone(),
        with: format!("{}{suggestion}{}", &original[..lead], &original[trail..]),
    })
}

/// The paragraph that ends at `caret`: from the blank line before it (or
/// the start of the text) to the caret. Empty when that is only blank space.
pub fn paragraph_before(text: &str, caret: usize) -> Range<usize> {
    let caret = caret.min(text.len());
    let mut start = text[..caret].rfind("\n\n").map_or(0, |at| at + 2);
    let blank = text[start..caret].len() - text[start..caret].trim_start().len();
    start += blank;
    if text[start..caret].trim().is_empty() {
        return caret..caret;
    }
    start..caret
}

/// One suggestion, from the moment it is asked for until it is accepted or
/// discarded. Dropping it stops the request.
pub(super) struct Suggestion {
    action: InlineAction,
    /// The note it was asked for.
    note_id: String,
    /// The text it was asked for, and where that was.
    range: Range<usize>,
    original: String,
    /// What has arrived.
    text: String,
    /// Something to say beside the label: a line of progress, or why the
    /// suggestion wasn't applied.
    notice: Option<String>,
    error: Option<String>,
    running: bool,
    /// The whole answer arrived. A rewrite that stopped or failed partway
    /// is never accepted: it would put half an answer in place of the text.
    complete: bool,
    cancel: Arc<AtomicBool>,
    task: Option<Task<()>>,
}

impl Suggestion {
    /// Whether Accept may put it in the note: it has text, and a rewrite
    /// arrived whole. A continuation or a summary that stopped partway only
    /// adds text, so what arrived may still go in.
    fn acceptable(&self) -> bool {
        !self.running && !self.text.trim().is_empty() && (self.complete || !self.action.replaces())
    }

    /// The note it was asked for.
    pub(super) fn note_id(&self) -> &str {
        &self.note_id
    }
}

impl Drop for Suggestion {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Relaxed);
    }
}

impl NoteEditor {
    /// Whether the inline assistant applies to the open note's text now: AI
    /// is on, the note can be edited and isn't locked, and nothing keeps it
    /// out of AI.
    pub(super) fn inline_available(&self, cx: &App) -> bool {
        let Some(note) = self.note.as_ref() else {
            return false;
        };
        cx.try_global::<AppState>()
            .is_some_and(|state| state.config.ai.enabled)
            && !self.text_read_only
            && self.lock == LockView::Plain
            && ai_feature::eligible(&self.store, &note.id)
    }

    /// The inline assistant's items for the right-click menu: all of them
    /// while text is selected and the assistant applies, else none.
    pub(super) fn inline_menu(&self, cx: &App) -> Vec<InlineAction> {
        if self.body.read(cx).selected_range().is_empty() || !self.inline_available(cx) {
            return Vec::new();
        }
        InlineAction::ALL.to_vec()
    }

    /// Asks for `action` on the selection (or, for Continue with nothing
    /// selected, the paragraph before the caret).
    pub fn start_inline(&mut self, action: InlineAction, cx: &mut Context<Self>) {
        let Some(id) = self.note.as_ref().map(|note| note.id.clone()) else {
            return;
        };
        if !cx
            .try_global::<AppState>()
            .is_some_and(|state| state.config.ai.enabled)
        {
            return;
        }
        if self.lock != LockView::Plain || !ai_feature::eligible(&self.store, &id) {
            self.notice(ai_feature::KEPT_OUT, cx);
            return;
        }
        if self.text_read_only {
            self.notice("This note can't be edited.", cx);
            return;
        }
        let (text, selection) = {
            let body = self.body.read(cx);
            (body.value().to_string(), body.selected_range())
        };
        let range = if !selection.is_empty() {
            selection
        } else if action == InlineAction::Continue {
            let range = paragraph_before(&text, selection.end);
            if range.is_empty() {
                self.notice("Put the cursor after some text to continue it.", cx);
                return;
            }
            range
        } else {
            self.notice("Select some text first.", cx);
            return;
        };
        if text
            .get(range.clone())
            .is_none_or(|part| part.trim().is_empty())
        {
            self.notice("Select some text first.", cx);
            return;
        }
        let original = text[range.clone()].to_owned();
        let title = self.title.read(cx).value().to_string();
        let prompt = features::inline_prompt(action, &title, &text[..range.start], &original);
        let maker = ai_feature::maker(cx);
        crate::metrics::record(cx, crate::metrics::Feature::Ai);
        let cancel = Arc::new(AtomicBool::new(false));
        let (tx, rx) = async_channel::unbounded::<Event>();
        let worker_cancel = cancel.clone();
        let spawned = std::thread::Builder::new()
            .name("kanso-inline".into())
            .spawn(move || {
                ai_feature::stream(&maker, &prompt, &worker_cancel, |event| {
                    tx.try_send(event).ok();
                });
            });
        if spawned.is_err() {
            self.notice("Couldn't start the request.", cx);
            return;
        }
        let task = cx.spawn(async move |this, cx| {
            let mut settled = false;
            while let Ok(event) = rx.recv().await {
                let last = matches!(event, Event::Done | Event::Error(_));
                let alive = this
                    .update(cx, |this, cx| this.inline_event(event, cx))
                    .is_ok();
                if last || !alive {
                    settled = true;
                    break;
                }
            }
            // The worker ended without Done or Error.
            if !settled {
                this.update(cx, |this, cx| {
                    this.inline_event(Event::Error(kanso_ai::CUT_OFF.to_owned()), cx)
                })
                .ok();
            }
        });
        self.inline = Some(Suggestion {
            action,
            note_id: id,
            range,
            original,
            text: String::new(),
            notice: None,
            error: None,
            running: true,
            complete: false,
            cancel,
            task: Some(task),
        });
        cx.notify();
    }

    /// What the request says as it goes.
    fn inline_event(&mut self, event: Event, cx: &mut Context<Self>) {
        let Some(suggestion) = self.inline.as_mut() else {
            return;
        };
        match event {
            Event::Text(text) => {
                suggestion.notice = None;
                suggestion.text.push_str(&text);
            }
            Event::Status(line) => suggestion.notice = Some(line),
            Event::Done => {
                suggestion.running = false;
                suggestion.complete = true;
                suggestion.text = features::clean_suggestion(&suggestion.text);
                if suggestion.text.is_empty() {
                    suggestion.error = Some("Nothing came back.".to_owned());
                }
            }
            Event::Error(message) => {
                suggestion.running = false;
                suggestion.error = Some(message);
            }
        }
        cx.notify();
    }

    /// Stops the request. What arrived stays, to accept or discard.
    pub fn stop_inline(&mut self, cx: &mut Context<Self>) {
        let Some(suggestion) = self.inline.as_mut() else {
            return;
        };
        if !suggestion.running {
            return;
        }
        suggestion.cancel.store(true, Ordering::Relaxed);
        suggestion.task = None;
        suggestion.running = false;
        suggestion.text = features::clean_suggestion(&suggestion.text);
        suggestion.notice = Some(if suggestion.action.replaces() {
            "Stopped. A partial rewrite can't be applied.".to_owned()
        } else {
            "Stopped.".to_owned()
        });
        cx.notify();
    }

    /// Drops the suggestion and stops its request.
    pub fn discard_inline(&mut self, cx: &mut Context<Self>) {
        if self.inline.take().is_some() {
            cx.notify();
        }
    }

    /// Puts the suggestion in the note, as one undo step, if the text it was
    /// made for is still there. If not, the card stays and says so.
    pub fn accept_inline(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(suggestion) = self.inline.as_ref() else {
            return;
        };
        if !suggestion.acceptable() {
            return;
        }
        if self.note_id() != Some(suggestion.note_id.as_str())
            || self.text_read_only
            || self.lock != LockView::Plain
        {
            self.inline = None;
            cx.notify();
            return;
        }
        let text = self.body.read(cx).value().to_string();
        let Some(change) = splice(
            &text,
            &suggestion.range,
            &suggestion.original,
            suggestion.action,
            &suggestion.text,
        ) else {
            if let Some(suggestion) = self.inline.as_mut() {
                suggestion.notice = Some(CHANGED.to_owned());
            }
            cx.notify();
            return;
        };
        self.inline = None;
        self.body.update(cx, |state, cx| {
            state.set_selected_range(change.range.clone(), cx);
            state.replace(change.with.clone(), window, cx);
        });
        cx.notify();
    }

    /// The suggestion's card, above the text.
    pub(super) fn render_suggestion(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
        let suggestion = self.inline.as_ref()?;
        let colors = Colors::get(cx);
        let running = suggestion.running;
        let (status, status_color) = match (&suggestion.error, &suggestion.notice) {
            (Some(error), _) => (Some(error.clone()), colors.danger),
            (None, Some(notice)) => (Some(notice.clone()), colors.text_secondary),
            (None, None) if running && suggestion.text.is_empty() => {
                (Some("Thinking…".to_owned()), colors.text_secondary)
            }
            (None, None) => (None, colors.text_secondary),
        };
        let ready = suggestion.acceptable();
        Some(
            v_flex()
                .flex_none()
                .gap_1()
                .mx(px(TEXT_INSET - 12.))
                .my_1()
                .px(px(12.))
                .py(px(8.))
                .rounded(px(6.))
                .bg(colors.fill)
                .text_size(px(12.5))
                .text_color(colors.text_secondary)
                .debug_selector(|| "inline-suggestion".into())
                .child(
                    h_flex()
                        .gap_2()
                        .child(
                            Icon::new(Lucide::Sparkles)
                                .size(px(13.))
                                .text_color(colors.text_muted),
                        )
                        .child(
                            div()
                                .flex_none()
                                .text_color(colors.text)
                                .child(suggestion.action.noun()),
                        )
                        .when_some(status, |el, status| {
                            el.child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .truncate()
                                    .text_color(status_color)
                                    .child(status),
                            )
                        }),
                )
                .when(!suggestion.text.is_empty(), |el| {
                    el.child(
                        div()
                            .id("inline-suggestion-text")
                            .max_h(px(200.))
                            .overflow_y_scroll()
                            .text_size(px(13.))
                            .line_height(px(19.))
                            .text_color(colors.text)
                            .child(suggestion.text.clone()),
                    )
                })
                .child(
                    h_flex()
                        .gap_2()
                        .justify_end()
                        .when(running, |el| {
                            el.child(
                                Button::new("inline-stop")
                                    .ghost()
                                    .xsmall()
                                    .label("Stop")
                                    .on_click(cx.listener(|this, _, _, cx| this.stop_inline(cx))),
                            )
                        })
                        .child(
                            Button::new("inline-discard")
                                .ghost()
                                .xsmall()
                                .label("Discard")
                                .on_click(cx.listener(|this, _, _, cx| this.discard_inline(cx))),
                        )
                        .child(
                            Button::new("inline-accept")
                                .primary()
                                .xsmall()
                                .label("Accept")
                                .disabled(!ready)
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.accept_inline(window, cx)
                                })),
                        ),
                )
                .into_any_element(),
        )
    }

    /// The editor's handler for the menu's and the palette's actions.
    pub(super) fn inline_assist(
        &mut self,
        action: &InlineAssist,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.start_inline(action.action, cx);
    }

    /// A finished suggestion for the open note, as the snapshots draw it,
    /// with no request behind it.
    #[cfg(test)]
    pub fn show_for_snapshot(&mut self, action: InlineAction, text: &str, cx: &mut Context<Self>) {
        let Some(note_id) = self.note_id().map(str::to_owned) else {
            return;
        };
        self.inline = Some(Suggestion {
            action,
            note_id,
            range: 0..0,
            original: String::new(),
            text: text.to_owned(),
            notice: None,
            error: None,
            running: false,
            complete: true,
            cancel: Arc::default(),
            task: None,
        });
        cx.notify();
    }

    /// The suggestion's text, for tests.
    #[cfg(test)]
    pub(super) fn suggestion_for_test(&self) -> Option<(String, bool, Option<String>)> {
        self.inline.as_ref().map(|s| {
            (
                s.text.clone(),
                s.running,
                s.notice.clone().or(s.error.clone()),
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::{InlineAction, Suggestion};
    use proptest::prelude::*;

    fn suggestion(action: InlineAction, text: &str, running: bool, complete: bool) -> Suggestion {
        Suggestion {
            action,
            note_id: "note:a".to_owned(),
            range: 0..0,
            original: String::new(),
            text: text.to_owned(),
            notice: None,
            error: None,
            running,
            complete,
            cancel: Arc::default(),
            task: None,
        }
    }

    #[test]
    fn a_rewrite_that_stopped_partway_is_never_accepted() {
        assert!(!suggestion(InlineAction::Improve, "Half an ans", false, false).acceptable());
        assert!(suggestion(InlineAction::Improve, "Whole answer.", false, true).acceptable());
        // Adding text that stopped partway still adds only what arrived.
        assert!(suggestion(InlineAction::Continue, "Half", false, false).acceptable());
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(128))]

        /// Invariant: a replacing action is acceptable only once its whole
        /// answer arrived, and nothing is acceptable while running or empty.
        #[test]
        fn prop_only_a_whole_rewrite_is_accepted(
            action in prop::sample::select(InlineAction::ALL.to_vec()),
            text in "\\PC{0,20}",
            (running, complete) in (any::<bool>(), any::<bool>()),
        ) {
            let accepted = suggestion(action, &text, running, complete).acceptable();
            if accepted {
                prop_assert!(!running && !text.trim().is_empty());
                prop_assert!(complete || !action.replaces());
            }
        }
    }
}
