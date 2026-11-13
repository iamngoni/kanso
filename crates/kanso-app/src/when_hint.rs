//! Typed dates in the editor: on a task line that ends in a date phrase
//! (`film Pixel review fri 3pm every week`), a quiet hint after the line
//! says what it will become (`→ Fri 13 Nov 2026, 15:00 · every week  Tab`).
//! Tab writes the stored form (`@2026-11-13T15:00 every week`) in its
//! place, as one undo step; Escape sends the hint away. Nothing is
//! rewritten without Tab.
//!
//! The parser is [`kanso_core::due_phrase`]. Slash dates read in the
//! order of the system's locale. The hint is worked out a moment after
//! typing stops, as the due dates' tints are; Tab works it out again, so it
//! never writes from a stale hint.

use std::sync::OnceLock;
use std::time::Duration;

use gpui_kit::component::input::EditorState;
use gpui_kit::*;
use kanso_core::dates::DateTime;
use kanso_core::due_phrase::{canonical, label, parse};
use kanso_core::due_phrase_model::DateOrder;
use kanso_core::links::code_ranges;
use kanso_core::tasks::{due_token, task_items};
use kanso_engine::edit::Edit;

use crate::theme::Colors;
pub use crate::when_hint_model::WhenHint;
use crate::zoom::px;

/// How long the editor waits after typing stops to show the hint.
const HINT_DELAY: Duration = Duration::from_millis(150);
/// The hint's text size, against the editor's.
const HINT_SIZE: f32 = 0.8;
/// The room between the line's end and the hint, in ems of its size.
const HINT_GAP: f32 = 1.2;

// MARK: Locale

/// How a slash date reads in `locale` (`en_US`, `en-GB.UTF-8`): month
/// first for the United States and the few places that follow it, day
/// first everywhere else. Pure.
pub fn order_for_locale(locale: &str) -> DateOrder {
    let tag = locale.split(['.', '@']).next().unwrap_or("");
    let region = tag.split(['_', '-']).nth(1).unwrap_or("");
    match region.to_ascii_uppercase().as_str() {
        "US" | "PH" | "PR" | "GU" | "VI" | "AS" | "MP" | "FM" | "PW" | "MH" => DateOrder::MonthDay,
        _ => DateOrder::DayMonth,
    }
}

/// The system's date order, read once: the locale the user's environment
/// names, or on a Mac the one in its settings.
pub fn system_date_order() -> DateOrder {
    static ORDER: OnceLock<DateOrder> = OnceLock::new();
    *ORDER.get_or_init(|| {
        let from_env = ["LC_ALL", "LC_TIME", "LANG"]
            .iter()
            .filter_map(|name| std::env::var(name).ok())
            .find(|value| !value.is_empty() && value != "C" && value != "POSIX");
        let locale = from_env.or_else(mac_locale).unwrap_or_default();
        order_for_locale(&locale)
    })
}

#[cfg(target_os = "macos")]
fn mac_locale() -> Option<String> {
    let out = std::process::Command::new("defaults")
        .args(["read", "-g", "AppleLocale"])
        .output()
        .ok()?;
    Some(String::from_utf8_lossy(&out.stdout).trim().to_owned())
}

#[cfg(not(target_os = "macos"))]
fn mac_locale() -> Option<String> {
    None
}

// MARK: The hint

/// The hint for the task line the caret is on in `text`, at local time
/// `now`: the date phrase that ends the line, when there is one and the
/// line isn't already written in the stored form. `None` off a task line,
/// in code, or when the line already has a date and the phrase isn't it.
/// Pure.
pub fn hint_at(text: &str, caret: usize, now: DateTime, order: DateOrder) -> Option<WhenHint> {
    let caret = caret.min(text.len());
    let start = text[..caret].rfind('\n').map_or(0, |at| at + 1);
    let end = text[caret..].find('\n').map_or(text.len(), |at| caret + at);
    let line = text[start..end].trim_end_matches('\r');
    let line_end = start + line.len();
    if task_items(line).is_empty() {
        return None;
    }
    if code_ranges(text)
        .iter()
        .any(|code| code.start <= start && start < code.end)
    {
        return None;
    }
    let found = parse(line, now, order)?;
    let words = &line[found.span.clone()];
    let stored = canonical(&found);
    // Already the stored form: nothing to say.
    if words == stored {
        return None;
    }
    // A line with a date has no second one.
    if due_token(&line[..found.span.start]).is_some() {
        return None;
    }
    Some(WhenHint {
        phrase: start + found.span.start..start + found.span.end,
        line: start..line_end,
        label: label(&found, now.date),
        canonical: stored,
    })
}

/// The edit Tab makes: the phrase becomes its stored form, and the caret
/// goes after it. One replacement, so one undo step. Pure.
pub fn edit(hint: &WhenHint) -> Edit {
    let caret = hint.phrase.start + hint.canonical.len();
    Edit {
        range: hint.phrase.clone(),
        text: hint.canonical.clone(),
        selection: caret..caret,
    }
}

// MARK: In the editor

/// The hint the editor shows now, kept while typing rests, and what is
/// needed to work it out again a moment after the next change.
#[derive(Default)]
pub struct WhenHints {
    current: Option<WhenHint>,
    task: Option<Task<()>>,
}

impl WhenHints {
    /// The hint, when the caret is still on its line.
    pub fn shown(&self, caret: usize) -> Option<&WhenHint> {
        self.current
            .as_ref()
            .filter(|hint| hint.line.start <= caret && caret <= hint.line.end)
    }

    /// Sends the hint away, and the work to bring it back.
    pub fn clear(&mut self) {
        self.current = None;
        self.task = None;
    }

    /// Works the hint out now, from the text and the caret.
    pub fn refresh(&mut self, body: &Entity<EditorState>, cx: &mut App) {
        self.task = None;
        let state = body.read(cx);
        let selection = state.selected_range();
        self.current = if selection.is_empty() {
            let now = crate::journal::now(cx);
            hint_at(&state.value(), selection.end, now, system_date_order())
        } else {
            None
        };
    }

    /// The text changed: the hint goes at once, and comes back once typing
    /// pauses.
    pub fn refresh_soon<V: 'static>(
        &mut self,
        body: Entity<EditorState>,
        owner: WeakEntity<V>,
        pick: fn(&mut V) -> &mut WhenHints,
        cx: &mut App,
    ) {
        self.current = None;
        self.task = Some(cx.spawn(async move |cx| {
            cx.background_executor().timer(HINT_DELAY).await;
            cx.update(|cx| {
                if let Some(owner) = owner.upgrade() {
                    owner.update(cx, |owner, cx| {
                        let hints = pick(owner);
                        hints.refresh(&body, cx);
                        cx.notify();
                    });
                }
            });
        }));
    }
}

/// The hint drawn after the end of its line, over the text editor once it
/// has laid out: `→ Fri 13 Nov 2026, 15:00 · every week` and a small `Tab`.
/// It drops to the line below when there is no room to the right.
// no-prop-test: painting; `hint_at` has the properties.
pub fn overlay(
    body: Entity<EditorState>,
    hint: &WhenHint,
    font_size: f32,
    window: &Window,
    cx: &App,
) -> AnyElement {
    let colors = Colors::get(cx);
    let muted = Hsla::from(colors.text_muted);
    let key_fill = Hsla::from(colors.fill);
    let key_text = Hsla::from(colors.text_secondary);
    let font = window.text_style().font();
    let at = hint.line.end;
    let words: SharedString = format!("\u{2192} {}", hint.label).into();
    let size = font_size * HINT_SIZE;
    canvas(
        |_, _, _| {},
        move |bounds, _, window, cx| {
            let Some(end) = body.read(cx).range_to_bounds(&(at..at)) else {
                return;
            };
            let run = |len: usize, color: Hsla| TextRun {
                len,
                font: font.clone(),
                color,
                background_color: None,
                underline: None,
                strikethrough: None,
            };
            let shape = |text: SharedString, color: Hsla| {
                let len = text.len();
                window
                    .text_system()
                    .shape_line(text, px(size), &[run(len, color)], None)
            };
            let label = shape(words.clone(), muted);
            let key = shape("Tab".into(), key_text);
            let pad = px(size * 0.4);
            let key_width = key.width + pad * 2.;
            let gap = px(size * HINT_GAP);
            let total = label.width + px(size * 0.6) + key_width;
            let mut origin = point(end.origin.x + gap, end.origin.y);
            if origin.x + total > bounds.right() {
                // No room beside the line's end: under it, from its start.
                origin = point(bounds.left(), end.origin.y + end.size.height);
            }
            let line_height = end.size.height;
            window.with_content_mask(Some(ContentMask { bounds }), |window| {
                label
                    .paint(origin, line_height, TextAlign::Left, None, window, cx)
                    .ok();
                let key_height = px(size * 1.45);
                let key_box = Bounds::new(
                    point(
                        origin.x + label.width + px(size * 0.6),
                        origin.y + (line_height - key_height) / 2.,
                    ),
                    gpui_kit::size(key_width, key_height),
                );
                window.paint_quad(fill(key_box, key_fill).corner_radii(px(size * 0.35)));
                key.paint(
                    point(key_box.origin.x + pad, origin.y),
                    line_height,
                    TextAlign::Left,
                    None,
                    window,
                    cx,
                )
                .ok();
            });
        },
    )
    .absolute()
    .top_0()
    .left_0()
    .size_full()
    .into_any_element()
}

#[cfg(test)]
#[path = "when_hint_tests.rs"]
mod tests;
