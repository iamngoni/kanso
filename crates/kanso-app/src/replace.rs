//! Replace in all notes (⇧⌘H): a sheet with what to find and what to put
//! instead, match case, whole word, regular expression and skip-code
//! toggles, and a live preview of every match, note by note, each of which
//! can be left out. "Replace N in M notes" changes them in one go under
//! one batch ID; a note changed elsewhere since the preview (by sync, say)
//! is left alone and reported. "Undo replace" puts the notes back, unless
//! they've been edited since. Trashed notes are only touched when asked.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::checkbox::Checkbox;
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::notification::Notification;
use gpui_kit::component::{
    Disableable as _, Icon, Selectable as _, Sizable as _, StyledExt as _, WindowExt as _, h_flex,
    v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_core::replace::{Matcher, Pattern, PatternError};
use kanso_store::{NoteMatches, Planned, ReplaceReport, Store};

use crate::clock::Clock;
use crate::editor::NoteEditor;
use crate::theme::{Colors, section_label};
use crate::workspace::{AppState, Workspace};
use crate::zoom::px;

actions!(replace, [ReplaceInNotes]);

pub const SHEET_WIDTH: f32 = 720.;
const PREVIEW_HEIGHT: f32 = 340.;
const PREVIEW_DELAY: Duration = Duration::from_millis(150);
/// Matches shown per note before "and N more".
const SHOWN_PER_NOTE: usize = 3;

pub fn bind_keys(cx: &mut App) {
    cx.bind_keys([KeyBinding::new("secondary-shift-h", ReplaceInNotes, None)]);
}

pub fn on_actions(el: Div, cx: &mut Context<Workspace>) -> Div {
    el.on_action(cx.listener(|this, _: &ReplaceInNotes, window, cx| {
        open_replace(this.editor.clone(), window, cx);
    }))
}

pub use kanso_engine::replace::{
    Options, apply_label, new_batch, report_text, undo_text, untitled,
};

pub struct ReplaceView {
    store: Arc<Store>,
    pub(crate) find: Entity<InputState>,
    pub(crate) with: Entity<InputState>,
    pub(crate) options: Options,
    pub(crate) found: Vec<NoteMatches>,
    pub(crate) excluded: HashSet<String>,
    error: Option<String>,
    searching: Option<Task<()>>,
    running: bool,
    _subscriptions: Vec<Subscription>,
}

impl ReplaceView {
    fn new(
        store: Arc<Store>,
        initial: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let find = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Find")
                .default_value(initial)
        });
        let with = cx.new(|cx| InputState::new(window, cx).placeholder("Replace with"));
        let subscriptions = vec![
            cx.subscribe_in(&find, window, |this, _, event: &InputEvent, _, cx| {
                if let InputEvent::Change = event {
                    this.search_soon(cx);
                }
            }),
            cx.subscribe_in(&with, window, |_, _, event: &InputEvent, _, cx| {
                if let InputEvent::Change = event {
                    cx.notify();
                }
            }),
        ];
        let mut view = ReplaceView {
            store,
            find,
            with,
            options: Options {
                skip_code: true,
                ..Options::default()
            },
            found: Vec::new(),
            excluded: HashSet::new(),
            error: None,
            searching: None,
            running: false,
            _subscriptions: subscriptions,
        };
        view.search_now(cx);
        view
    }

    fn pattern(&self, cx: &App) -> Pattern {
        self.options.pattern(&self.find.read(cx).value())
    }

    fn matcher(&self, cx: &App) -> Result<Matcher, PatternError> {
        Matcher::new(&self.pattern(cx))
    }

    fn search_soon(&mut self, cx: &mut Context<Self>) {
        self.searching = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(PREVIEW_DELAY).await;
            this.update(cx, |this, cx| this.search_now(cx)).ok();
        }));
    }

    /// Finds the matches again, off the main thread.
    pub(crate) fn search_now(&mut self, cx: &mut Context<Self>) {
        let matcher = match self.matcher(cx) {
            Ok(matcher) => matcher,
            Err(PatternError::Empty) => {
                self.found.clear();
                self.error = None;
                cx.notify();
                return;
            }
            Err(PatternError::Invalid(err)) => {
                self.found.clear();
                self.error = Some(err);
                cx.notify();
                return;
            }
        };
        self.error = None;
        let store = self.store.clone();
        let include_trash = self.options.include_trash;
        let task = cx
            .background_executor()
            .spawn(async move { store.replace_preview(&matcher, include_trash) });
        self.searching = Some(cx.spawn(async move |this, cx| {
            let found = task.await;
            this.update(cx, |this, cx| {
                match found {
                    Ok(found) => this.found = found,
                    Err(err) => this.error = Some(err.to_string()),
                }
                cx.notify();
            })
            .ok();
        }));
    }

    /// Reads the matches now, on this thread: for tests and snapshots.
    #[cfg(test)]
    pub(crate) fn search_blocking(&mut self, cx: &mut Context<Self>) {
        if let Ok(matcher) = self.matcher(cx) {
            self.found = self
                .store
                .replace_preview(&matcher, self.options.include_trash)
                .unwrap_or_default();
        }
        cx.notify();
    }

    fn set_option(&mut self, set: impl FnOnce(&mut Options), cx: &mut Context<Self>) {
        set(&mut self.options);
        self.search_now(cx);
    }

    fn toggle_note(&mut self, id: &str, cx: &mut Context<Self>) {
        if !self.excluded.remove(id) {
            self.excluded.insert(id.to_owned());
        }
        cx.notify();
    }

    /// The notes to change: those found and not left out.
    fn planned(&self) -> Vec<&NoteMatches> {
        self.found
            .iter()
            .filter(|n| !self.excluded.contains(&n.id))
            .collect()
    }

    /// Matches and notes to change.
    fn totals(&self) -> (usize, usize) {
        let planned = self.planned();
        (planned.iter().map(|n| n.count).sum(), planned.len())
    }

    /// Replaces in the planned notes, closes the sheet and offers to undo.
    fn apply(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.running {
            return;
        }
        if cx
            .try_global::<crate::services::Services>()
            .is_some_and(|s| s.writes_held)
        {
            self.error = Some("Notes can't be changed while the library is being replaced.".into());
            cx.notify();
            return;
        }
        let Ok(matcher) = self.matcher(cx) else {
            return;
        };
        // The open note's typing is saved first; it is then one of the
        // notes changed, or reported if it moved past the preview.
        crate::editor::peers::flush_everywhere(cx);
        let planned: Vec<Planned> = self
            .planned()
            .into_iter()
            .map(|n| Planned {
                id: n.id.clone(),
                rev: n.rev.clone(),
            })
            .collect();
        if planned.is_empty() {
            return;
        }
        let replacement = self.with.read(cx).value().to_string();
        let batch = new_batch();
        self.running = true;
        let result = self.store.replace_in_notes(
            &matcher,
            &replacement,
            &planned,
            &batch,
            Clock::now_ms(cx),
        );
        self.running = false;
        match result {
            Ok(report) => {
                window.close_dialog(cx);
                notify_replaced(report, batch, self.store.clone(), window, cx);
            }
            Err(err) => {
                self.error = Some(format!("Couldn't replace: {err}"));
                cx.notify();
            }
        }
    }

    // MARK: Drawing

    fn toggle(
        &self,
        id: &'static str,
        icon: Lucide,
        tip: &'static str,
        on: bool,
        set: fn(&mut Options),
        cx: &mut Context<Self>,
    ) -> Button {
        let colors = Colors::get(cx);
        Button::new(id)
            .ghost()
            .small()
            .icon(Icon::new(icon).size(px(15.)).text_color(if on {
                colors.accent
            } else {
                colors.text_secondary
            }))
            .selected(on)
            .tooltip(tip)
            .on_click(cx.listener(move |this, _, _, cx| this.set_option(set, cx)))
    }

    fn render_note(&self, note: &NoteMatches, cx: &mut Context<Self>) -> AnyElement {
        let colors = Colors::get(cx);
        let included = !self.excluded.contains(&note.id);
        let id = note.id.clone();
        let more = note.count.saturating_sub(SHOWN_PER_NOTE);
        let lines = note.matches.iter().take(SHOWN_PER_NOTE).map(|m| {
            let (before, hit, after) = (
                &m.text[..m.range.start],
                &m.text[m.range.clone()],
                &m.text[m.range.end..],
            );
            h_flex()
                .pl(px(28.))
                .gap(px(8.))
                .text_size(px(12.5))
                .line_height(px(18.))
                .child(
                    div()
                        .w(px(34.))
                        .flex_none()
                        .text_right()
                        .text_color(colors.text_muted)
                        .child((m.line + 1).to_string()),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .truncate()
                        .text_color(colors.text_secondary)
                        .child(
                            StyledText::new(format!("{before}{hit}{after}")).with_highlights([(
                                before.len()..before.len() + hit.len(),
                                HighlightStyle {
                                    color: Some(colors.text.into()),
                                    background_color: Some(
                                        Hsla::from(colors.warning).opacity(if included {
                                            0.42
                                        } else {
                                            0.14
                                        }),
                                    ),
                                    ..Default::default()
                                },
                            )]),
                        ),
                )
        });
        v_flex()
            .py(px(6.))
            .gap(px(3.))
            .border_b_1()
            .border_color(colors.divider)
            .when(!included, |el| el.opacity(0.55))
            .child(
                h_flex()
                    .gap(px(8.))
                    .child(
                        Checkbox::new(SharedString::from(format!("replace-in-{}", note.id)))
                            .checked(included)
                            .on_click(
                                cx.listener(move |this, _: &bool, _, cx| this.toggle_note(&id, cx)),
                            ),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .text_size(px(13.5))
                            .font_medium()
                            .text_color(colors.text)
                            .child(untitled(&note.title).to_owned()),
                    )
                    .when(note.trashed, |el| {
                        el.child(
                            div()
                                .text_size(px(11.5))
                                .text_color(colors.text_muted)
                                .child("In Trash"),
                        )
                    })
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(colors.text_muted)
                            .child(note.count.to_string()),
                    ),
            )
            .children(lines)
            .when(more > 0, |el| {
                el.child(
                    div()
                        .pl(px(70.))
                        .text_size(px(12.))
                        .text_color(colors.text_muted)
                        .child(format!("and {more} more")),
                )
            })
            .into_any_element()
    }
}

// no-prop-test: drawing; the replace sheet snapshots show it in both themes.
impl Render for ReplaceView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let o = self.options;
        let toggles = h_flex()
            .gap(px(2.))
            .child(self.toggle(
                "replace-case",
                Lucide::CaseSensitive,
                "Match case",
                o.match_case,
                |o| o.match_case = !o.match_case,
                cx,
            ))
            .child(self.toggle(
                "replace-word",
                Lucide::WholeWord,
                "Whole word",
                o.whole_word,
                |o| o.whole_word = !o.whole_word,
                cx,
            ))
            .child(self.toggle(
                "replace-regex",
                Lucide::Regex,
                "Regular expression ($1 in the replacement for a group)",
                o.regex,
                |o| o.regex = !o.regex,
                cx,
            ))
            .child(self.toggle(
                "replace-code",
                Lucide::CodeXml,
                "Skip code blocks",
                o.skip_code,
                |o| o.skip_code = !o.skip_code,
                cx,
            ))
            .child(self.toggle(
                "replace-trash",
                Lucide::Trash,
                "Include notes in the Trash",
                o.include_trash,
                |o| o.include_trash = !o.include_trash,
                cx,
            ));
        let (matches, notes) = self.totals();
        let found: usize = self.found.iter().map(|n| n.count).sum();
        let summary = if let Some(error) = &self.error {
            div().text_color(colors.danger).child(error.clone())
        } else if self.find.read(cx).value().is_empty() {
            div().child("Type what to find.")
        } else if self.found.is_empty() {
            div().child("No matches.")
        } else {
            div().child(format!(
                "{found} {} in {} {}",
                if found == 1 { "match" } else { "matches" },
                self.found.len(),
                if self.found.len() == 1 {
                    "note"
                } else {
                    "notes"
                }
            ))
        };
        let label = apply_label(matches, notes);
        let can_apply = label.is_some() && self.error.is_none();
        let rows: Vec<AnyElement> = self
            .found
            .iter()
            .map(|note| self.render_note(note, cx))
            .collect();
        v_flex()
            .gap(px(10.))
            .child(
                h_flex()
                    .gap(px(8.))
                    .child(div().flex_1().child(Input::new(&self.find).cleanable(true)))
                    .child(toggles),
            )
            .child(Input::new(&self.with))
            .child(
                h_flex()
                    .pt(px(4.))
                    .child(section_label(div().flex_1(), colors).child("Preview"))
                    .child(summary.text_size(px(12.)).text_color(colors.text_muted)),
            )
            .child(
                div()
                    .id("replace-preview")
                    .h(px(PREVIEW_HEIGHT))
                    .overflow_y_scroll()
                    .rounded(px(8.))
                    .border_1()
                    .border_color(colors.divider)
                    .bg(colors.list)
                    .px(px(12.))
                    .children(rows),
            )
            .child(
                h_flex()
                    .pt(px(4.))
                    .gap(px(8.))
                    .child(div().flex_1())
                    .child(
                        Button::new("replace-cancel")
                            .ghost()
                            .label("Cancel")
                            .on_click(|_, window, cx| window.close_dialog(cx)),
                    )
                    .child(
                        Button::new("replace-apply")
                            .primary()
                            .label(label.unwrap_or_else(|| "Replace".to_owned()))
                            .disabled(!can_apply)
                            .loading(self.running)
                            .debug_selector(|| "replace-apply".into())
                            .on_click(cx.listener(|this, _, window, cx| this.apply(window, cx))),
                    ),
            )
    }
}

/// Says what was replaced, with Undo replace.
fn notify_replaced(
    report: ReplaceReport,
    batch: String,
    store: Arc<Store>,
    window: &mut Window,
    cx: &mut App,
) {
    let text = report_text(&report);
    if report.notes.is_empty() {
        window.push_notification(Notification::info(text), cx);
        return;
    }
    let notification = Notification::info(text).action(move |_, _, _| {
        let (batch, store) = (batch.clone(), store.clone());
        Button::new("undo-replace")
            .label("Undo replace")
            .on_click(move |_, window, cx| {
                let message = match store.undo_replace(&batch, Clock::now_ms(cx)) {
                    Ok(report) => Notification::info(undo_text(&report)),
                    Err(err) => Notification::error(format!("Couldn't undo the replace: {err}")),
                };
                window.push_notification(message, cx);
            })
    });
    window.push_notification(notification, cx);
}

/// Opens the sheet, with the editor's selection to find when there is one.
pub fn open_replace(
    editor: Entity<NoteEditor>,
    window: &mut Window,
    cx: &mut App,
) -> Entity<ReplaceView> {
    let store = cx.global::<AppState>().store.clone();
    // The search sees what was just typed.
    editor.update(cx, |editor, cx| editor.flush(cx));
    let initial = editor.read(cx).selected_text(cx).unwrap_or_default();
    let initial = if initial.contains('\n') {
        String::new()
    } else {
        initial
    };
    let view = cx.new(|cx| ReplaceView::new(store, initial, window, cx));
    let dialog_view = view.clone();
    let find = view.read(cx).find.clone();
    window.open_alert_dialog(cx, move |alert, _, _| {
        alert
            .title("Replace in all notes")
            .width(px(SHEET_WIDTH))
            .child(dialog_view.clone())
            .footer(div())
    });
    find.update(cx, |find, cx| find.focus(window, cx));
    view
}
