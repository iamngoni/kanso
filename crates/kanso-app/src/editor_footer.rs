//! The editor's bottom band: where the cursor is, and how long the note is.
//! In focus mode it also shows the note's word goal, as a small ring and
//! "640 / 1,000"; clicking it sets the goal.
//!
//! It is its own view so the editor doesn't redraw for it. The text editor
//! has no event for a moving cursor and notifies on every caret blink, so
//! the footer watches those notifications and only redraws when the line or
//! column changed. Words and characters are counted after typing pauses.

use std::time::Duration;

use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{EditorState, InputEvent, Position};
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenuItem};
use gpui_kit::component::{ActiveTheme as _, Sizable as _, h_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::layout::BOTTOM_BAND_HEIGHT;
use crate::preview::TEXT_INSET;
use crate::text_stats::{format_count, plural, word_count};
use crate::theme::Colors;
use crate::word_goal::{self, GoalSetting, PRESETS};
use crate::zoom::px;

/// How long typing pauses before the counts catch up.
const COUNT_DELAY: Duration = Duration::from_millis(250);
/// The band itself: in focus mode, hovering it shows the counts.
const FOOTER_GROUP: &str = "editor-footer";
/// The goal ring's size and the width of its line.
const RING: f32 = 12.;
const RING_LINE: f32 = 1.75;

/// The word goal chosen in the footer's menu.
pub enum FooterEvent {
    Goal(GoalSetting),
    /// "Custom…": the palette asks for a number.
    CustomGoal,
}

pub struct EditorFooter {
    body: Entity<EditorState>,
    cursor: Position,
    words: usize,
    chars: usize,
    /// Whether there is a note, and a cursor in it to report.
    has_note: bool,
    show_cursor: bool,
    /// In focus mode: no divider above it, the counts wait for the pointer,
    /// and the word goal shows.
    quiet: bool,
    /// The open note's own goal, and the default from Settings.
    goal: GoalSetting,
    default_goal: Option<u32>,
    /// With Vim keys on: the mode, and a command being typed.
    vim: Option<crate::vim::Status>,
    /// In Vim's Visual modes, its cursor (the editor's is at the
    /// selection's end).
    vim_cursor: Option<Position>,
    /// For a shared note: shared, the role, and whether it is live
    /// (`collab_text::share_footer`).
    share: Option<String>,
    /// Shows the counts in focus mode, as a hover would.
    #[cfg(test)]
    pub(crate) reveal: bool,
    count_task: Option<Task<()>>,
    /// Redraws when an agent task's row changes, for the status line.
    _agent_watch: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<FooterEvent> for EditorFooter {}

impl EditorFooter {
    pub fn new(body: Entity<EditorState>, cx: &mut Context<Self>) -> Self {
        let subscriptions = vec![
            cx.observe(&body, |this, body, cx| {
                let cursor = body.read(cx).cursor_position();
                if cursor != this.cursor {
                    this.cursor = cursor;
                    cx.notify();
                }
            }),
            cx.subscribe(&body, |this, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    this.count_soon(cx);
                }
            }),
        ];
        EditorFooter {
            body,
            cursor: Position::default(),
            words: 0,
            chars: 0,
            has_note: false,
            show_cursor: true,
            quiet: false,
            goal: GoalSetting::Default,
            default_goal: None,
            vim: None,
            vim_cursor: None,
            share: None,
            #[cfg(test)]
            reveal: false,
            count_task: None,
            _agent_watch: cx
                .try_global::<crate::services::Services>()
                .map(|services| services.tasks.subscribe())
                .map(|events| {
                    cx.spawn(async move |this, cx| {
                        while events.recv().await.is_ok() {
                            if this.update(cx, |_, cx| cx.notify()).is_err() {
                                break;
                            }
                        }
                    })
                }),
            _subscriptions: subscriptions,
        }
    }

    /// A note was opened (or closed): count it now.
    pub fn note_opened(&mut self, has_note: bool, cx: &mut Context<Self>) {
        self.has_note = has_note;
        self.count_task = None;
        self.count(cx);
        cx.notify();
    }

    /// The cursor means nothing in Preview, where there is no source.
    pub fn set_show_cursor(&mut self, show: bool, cx: &mut Context<Self>) {
        if self.show_cursor != show {
            self.show_cursor = show;
            cx.notify();
        }
    }

    /// Focus mode drops the divider above the counts, and shows them only
    /// while the pointer is over the band.
    pub fn set_quiet(&mut self, quiet: bool, cx: &mut Context<Self>) {
        if self.quiet != quiet {
            self.quiet = quiet;
            cx.notify();
        }
    }

    /// The open note's goal and the default.
    pub fn set_goal(&mut self, goal: GoalSetting, default: Option<u32>, cx: &mut Context<Self>) {
        if (self.goal, self.default_goal) != (goal, default) {
            (self.goal, self.default_goal) = (goal, default);
            cx.notify();
        }
    }

    /// The Vim mode and pending command to show, or `None` with Vim keys
    /// off; and in Visual modes, Vim's cursor for Ln and Col.
    pub fn set_vim(
        &mut self,
        vim: Option<crate::vim::Status>,
        cursor: Option<Position>,
        cx: &mut Context<Self>,
    ) {
        if (&self.vim, self.vim_cursor) != (&vim, cursor) {
            self.vim = vim;
            self.vim_cursor = cursor;
            cx.notify();
        }
    }

    /// What the band says about the open note's sharing, if it is shared.
    pub fn set_share(&mut self, share: Option<String>, cx: &mut Context<Self>) {
        if self.share != share {
            self.share = share;
            cx.notify();
        }
    }

    /// What the band says about sharing.
    #[cfg(test)]
    pub fn share_shown(&self) -> Option<&str> {
        self.share.as_deref()
    }

    /// The Vim mode and pending command shown.
    #[cfg(test)]
    pub fn vim_shown(&self) -> Option<&crate::vim::Status> {
        self.vim.as_ref()
    }

    /// The goal that applies to the open note.
    pub fn goal(&self) -> Option<u32> {
        word_goal::effective_goal(self.goal, self.default_goal)
    }

    fn count_soon(&mut self, cx: &mut Context<Self>) {
        self.count_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(COUNT_DELAY).await;
            this.update(cx, |this, cx| this.count(cx)).ok();
        }));
    }

    fn count(&mut self, cx: &mut Context<Self>) {
        let text = self.body.read(cx).value();
        let counts = (word_count(&text), text.chars().count());
        if counts != (self.words, self.chars) {
            (self.words, self.chars) = counts;
            cx.notify();
        }
    }

    pub fn label(&self) -> String {
        footer_label(
            self.show_cursor
                .then_some(self.vim_cursor.unwrap_or(self.cursor)),
            self.words,
            self.chars,
        )
    }

    /// The goal's ring and words, or "Word goal" with none; a click opens
    /// the goal menu.
    // no-prop-test: drawing and a menu; `word_goal` has the properties and
    // the goal snapshot shows it.
    fn render_goal(&self, cx: &mut Context<Self>) -> AnyElement {
        let colors = Colors::get(cx);
        let footer = cx.entity();
        let setting = self.goal;
        let default = self.default_goal;
        let goal = self.goal();
        let (content, tooltip): (AnyElement, &str) = match goal {
            Some(goal) => {
                let progress = word_goal::progress(self.words, goal);
                let met = progress >= 1.;
                let fill = if met { colors.success } else { colors.accent };
                (
                    h_flex()
                        .gap(px(6.))
                        .child(ring(progress, colors.divider.into(), fill.into()))
                        .child(
                            div()
                                .text_size(px(12.))
                                .text_color(if met {
                                    colors.text_secondary
                                } else {
                                    colors.text_muted
                                })
                                .child(word_goal::goal_label(self.words, goal)),
                        )
                        .into_any_element(),
                    "Word goal",
                )
            }
            None => (
                div()
                    .text_size(px(12.))
                    .text_color(colors.text_muted)
                    .child("Set word goal")
                    .into_any_element(),
                "Set a word goal for this note",
            ),
        };
        let emit =
            move |event: fn() -> FooterEvent, footer: &Entity<EditorFooter>, cx: &mut App| {
                footer.update(cx, |_, cx| cx.emit(event()));
            };
        Button::new("word-goal")
            .ghost()
            .xsmall()
            .tab_stop(false)
            .child(content)
            .tooltip(tooltip)
            .debug_selector(|| "word-goal".into())
            .dropdown_menu(move |mut menu, _, _| {
                menu = menu.label("Word goal");
                for preset in PRESETS {
                    let footer = footer.clone();
                    menu = menu.item(
                        PopupMenuItem::new(format!("{} words", format_count(preset as usize)))
                            .checked(setting == GoalSetting::Words(preset))
                            .on_click(move |_, _, cx| {
                                footer.update(cx, |_, cx| {
                                    cx.emit(FooterEvent::Goal(GoalSetting::Words(preset)))
                                });
                            }),
                    );
                }
                let custom = footer.clone();
                let custom_checked =
                    matches!(setting, GoalSetting::Words(n) if !PRESETS.contains(&n));
                menu = menu.separator().item(
                    PopupMenuItem::new("Custom…")
                        .checked(custom_checked)
                        .on_click(move |_, _, cx| emit(|| FooterEvent::CustomGoal, &custom, cx)),
                );
                let clear = footer.clone();
                match default {
                    Some(default) => {
                        let off = footer.clone();
                        menu.item(
                            PopupMenuItem::new(format!(
                                "Default ({} words)",
                                format_count(default as usize)
                            ))
                            .checked(setting == GoalSetting::Default)
                            .on_click(move |_, _, cx| {
                                emit(|| FooterEvent::Goal(GoalSetting::Default), &clear, cx)
                            }),
                        )
                        .item(
                            PopupMenuItem::new("No goal for this note")
                                .checked(setting == GoalSetting::Off)
                                .on_click(move |_, _, cx| {
                                    emit(|| FooterEvent::Goal(GoalSetting::Off), &off, cx)
                                }),
                        )
                    }
                    None => menu.item(
                        PopupMenuItem::new("No goal")
                            .checked(goal.is_none())
                            .on_click(move |_, _, cx| {
                                emit(|| FooterEvent::Goal(GoalSetting::Default), &clear, cx)
                            }),
                    ),
                }
            })
            .into_any_element()
    }
}

/// A ring `progress` of the way round, from the top clockwise, over a
/// faint full ring.
// no-prop-test: painting; the goal snapshot shows it.
fn ring(progress: f32, track: Hsla, fill: Hsla) -> impl IntoElement {
    canvas(
        |_, _, _| {},
        move |bounds, _, window, _| {
            let center = bounds.center();
            let radius =
                (crate::zoom::unscaled(bounds.size.width.min(bounds.size.height)) - RING_LINE) / 2.;
            let arc = |share: f32| -> Option<Path<Pixels>> {
                let steps = ((48. * share).ceil() as usize).max(2);
                let mut path = PathBuilder::stroke(px(RING_LINE));
                for step in 0..=steps {
                    let angle = std::f32::consts::TAU * share * step as f32 / steps as f32
                        - std::f32::consts::FRAC_PI_2;
                    let at = point(
                        center.x + px(radius * angle.cos()),
                        center.y + px(radius * angle.sin()),
                    );
                    if step == 0 {
                        path.move_to(at);
                    } else {
                        path.line_to(at);
                    }
                }
                path.build().ok()
            };
            if let Some(path) = arc(1.) {
                window.paint_path(path, track);
            }
            if progress > 0.
                && let Some(path) = arc(progress.min(1.))
            {
                window.paint_path(path, fill);
            }
        },
    )
    .flex_none()
    .size(px(RING))
}

/// "Ln 12, Col 4 · 412 words · 2,841 chars", from a 0-based cursor.
// no-prop-test: joins fixed words around `format_count`, whose properties
// cover the numbers; `footer_label_examples` checks the layout.
pub fn footer_label(cursor: Option<Position>, words: usize, chars: usize) -> String {
    let counts = format!(
        "{} · {}",
        plural(words, "word", "words"),
        plural(chars, "char", "chars")
    );
    match cursor {
        Some(cursor) => format!(
            "Ln {}, Col {} · {counts}",
            format_count(cursor.line as usize + 1),
            format_count(cursor.character as usize + 1)
        ),
        None => counts,
    }
}

impl Render for EditorFooter {
    // no-prop-test: layout only; the snapshots show it.
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        // "Agent: 14 notes read, 2 changes" while a task works.
        let agent_line = cx
            .try_global::<crate::services::Services>()
            .and_then(|services| crate::agent_task_model::status_bar_line(&services.tasks.rows()));
        #[cfg(test)]
        let hidden = self.quiet && !self.reveal;
        #[cfg(not(test))]
        let hidden = self.quiet;
        let item = |el: Div| {
            el.when(hidden, |el| {
                el.opacity(0.)
                    .group_hover(FOOTER_GROUP, |style| style.opacity(1.))
            })
        };
        // In focus mode a goal stays in sight; with none, the way to set
        // one waits with the counts.
        let goal = (self.quiet && self.has_note).then(|| {
            let shown = self.goal().is_some();
            let el = div().flex_none().child(self.render_goal(cx));
            if shown { el } else { item(el) }
        });
        h_flex()
            .id("editor-footer")
            .group(FOOTER_GROUP)
            .h(px(BOTTOM_BAND_HEIGHT))
            .w_full()
            .flex_none()
            .px(px(TEXT_INSET))
            .gap_4()
            .when(!self.quiet, |el| {
                el.border_t_1().border_color(colors.divider)
            })
            .bg(colors.editor)
            .text_size(px(12.))
            .text_color(colors.text_muted)
            // The Vim mode stays in sight in focus mode too: it says what
            // the next key will do.
            .when_some(self.vim.as_ref().filter(|_| self.has_note), |el, vim| {
                el.child(
                    h_flex()
                        .flex_none()
                        .gap(px(6.))
                        .debug_selector(|| "vim-mode".into())
                        .child(
                            div()
                                .text_size(px(11.))
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(colors.text_secondary)
                                .child(vim.mode.label()),
                        )
                        .when(!vim.pending.is_empty(), |el| {
                            el.child(
                                div()
                                    .font_family(cx.theme().mono_font_family.clone())
                                    .text_size(px(11.))
                                    .child(vim.pending.clone()),
                            )
                        }),
                )
            })
            .when(self.has_note, |el| {
                el.child(
                    item(div())
                        .flex_1()
                        .min_w_0()
                        .truncate()
                        .child(self.label()),
                )
                .when_some(self.share.clone(), |el, share| {
                    el.child(
                        div()
                            .flex_none()
                            .debug_selector(|| "share-state".into())
                            .child(share),
                    )
                })
                .when(!self.quiet, |el| {
                    el.child(item(div()).flex_none().child("Markdown"))
                })
            })
            .when_some(agent_line, |el, line| {
                el.child(
                    div()
                        .flex_none()
                        .debug_selector(|| "agent-status".into())
                        .text_color(colors.text_secondary)
                        .child(line),
                )
            })
            .children(goal)
    }
}

#[cfg(test)]
mod tests {
    use super::footer_label;
    use gpui_kit::component::input::Position;

    #[test]
    fn footer_label_examples() {
        assert_eq!(
            footer_label(Some(Position::new(11, 3)), 412, 2841),
            "Ln 12, Col 4 · 412 words · 2,841 chars"
        );
        assert_eq!(footer_label(None, 1, 1), "1 word · 1 char");
    }
}
