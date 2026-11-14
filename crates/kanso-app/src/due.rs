//! Due dates on tasks (`- [ ] Pay rent @2026-10-25`, `@2026-10-25T09:00`)
//! where the note is written and read: a small chip in the preview
//! ("Tomorrow", "Fri 25 Oct", in a warm red once overdue), a quiet tint on
//! the token in the editor, and `@` autocomplete for the date in a task.
//!
//! The preview is handed each task's due token written as a
//! `kanso-due:` link (as wiki-links become ordinary links), which
//! [`DueChip`] draws. The chip works out its words when it is drawn, so
//! "Today" stays true.

use std::time::Duration;

use gpui_kit::base::input::{Rope, RopeExt as _, TextDecoration, TextDecorationCollection};
use gpui_kit::base::text::{
    InlineElement, InlineRenderContext, MarkdownNode, MarkdownParseContext, MarkdownPlugin,
    markdown_ast,
};
use gpui_kit::component::input::EditorState;
use gpui_kit::component::menu::{ContextMenuExt as _, PopupMenuItem};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_core::tasks::Due;
use lsp_types::{CompletionItem, CompletionItemKind, CompletionTextEdit, TextEdit};

use crate::theme::Colors;
use crate::zoom::px;

use kanso_core::repeat_model::Repeat;
pub use kanso_engine::tasks::{
    ChipColor, Tone, chip_label, date_choices, date_query, long_label, marks, parse_due_link,
    repeat_choices, repeat_query, tone,
};

/// The chip menu's item (and the editor's) that ends a task's repeat.
pub const STOP_REPEATING: &str = "Stop Repeating";

/// "Stop Repeating" from the editor's text menu, for the task on the
/// caret's line.
#[derive(Clone, Debug, Default, PartialEq, Action)]
#[action(namespace = kanso, no_json)]
pub struct StopRepeating;

/// What marks a repeating task's chip, after its date: `Fri 14 Nov 15:00 ↻`.
pub const REPEAT_MARK: &str = "\u{21bb}";

/// A chip's words with the mark of a repeat, when it has one.
pub fn repeat_label(label: String, repeat: Option<Repeat>) -> String {
    match repeat {
        Some(_) => format!("{label} {REPEAT_MARK}"),
        None => label,
    }
}

/// A chip's tooltip: the date in words, then `Repeats every week`.
pub fn chip_tooltip(due: Due, repeat: Option<Repeat>) -> String {
    match repeat {
        Some(repeat) => format!("{} \u{b7} Repeats {repeat}", long_label(due)),
        None => long_label(due),
    }
}

/// How long the editor waits after typing stops to tint new dates.
const MARK_DELAY: Duration = Duration::from_millis(150);

/// The note's Markdown with each task's due date, outside code, written as
/// a `kanso-due:` link for [`DueChip`] to draw, its repeat inside. Text
/// without dates comes back as it is, the same shared string.
// no-prop-test: delegates to kanso_engine::tasks::rewrite_for_preview_with_repeats, tested there.
pub fn rewrite_for_preview(text: &SharedString) -> SharedString {
    match kanso_engine::tasks::rewrite_for_preview_with_repeats(text) {
        std::borrow::Cow::Borrowed(_) => text.clone(),
        std::borrow::Cow::Owned(rewritten) => rewritten.into(),
    }
}

/// What a chip shows.
struct Chip {
    due: Due,
    checked: bool,
    repeat: Option<Repeat>,
    /// The whole line of the shown text the chip is on: what names its task
    /// to "Stop Repeating".
    line: String,
}

/// Draws the links due dates become: a small rounded chip with the date in
/// words, tinted by how near it is. An open repeating task's chip has a
/// menu with "Stop Repeating", which asks the editor through `edits`.
pub struct DueChip {
    edits: crate::preview::ResizeSender,
}

impl DueChip {
    pub const NAME: &'static str = "kanso-due-chip";

    pub fn new(edits: crate::preview::ResizeSender) -> Self {
        Self { edits }
    }
}

/// The line of `source` that byte `at` is on, without its line break.
pub fn line_around(source: &str, at: usize) -> &str {
    let at = at.min(source.len());
    let start = source[..at].rfind('\n').map_or(0, |i| i + 1);
    let end = source[at..].find('\n').map_or(source.len(), |i| at + i);
    source[start..end].trim_end_matches('\r')
}

/// `note` with "Stop Repeating" done for a chip on the line of `shown` (the
/// preview's text of `note`, which has the same lines) that reads `line`:
/// the first such line whose task in `note` is open and repeats.
pub fn stop_repeating_shown(note: &str, shown: &str, line: &str) -> Option<String> {
    shown
        .split('\n')
        .enumerate()
        .filter(|(_, text)| text.trim_end_matches('\r') == line)
        .find_map(|(ix, _)| kanso_core::tick::stop_repeating(note, ix as u32))
}

/// A chip's text and fill colors for `tone`.
// no-prop-test: delegates to kanso_engine::tasks::chip_tint, tested there.
pub fn chip_colors(tone: Tone, colors: &Colors) -> (Hsla, Hsla) {
    let tint = kanso_engine::tasks::chip_tint(tone);
    let role = |color| {
        Hsla::from(match color {
            ChipColor::Danger => colors.danger,
            ChipColor::Link => colors.link,
            ChipColor::TextSecondary => colors.text_secondary,
            ChipColor::TextMuted => colors.text_muted,
            ChipColor::Fill => colors.fill,
        })
    };
    let fill = role(tint.fill);
    (
        role(tint.text),
        if tint.fill_alpha < 1.0 {
            fill.opacity(tint.fill_alpha)
        } else {
            fill
        },
    )
}

impl MarkdownPlugin for DueChip {
    fn name(&self) -> &str {
        Self::NAME
    }

    fn parse(
        &self,
        node: &markdown_ast::Node,
        cx: &MarkdownParseContext<'_>,
    ) -> Option<MarkdownNode> {
        let markdown_ast::Node::Link(link) = node else {
            return None;
        };
        let (due, checked, repeat) = parse_due_link(&link.url)?;
        let source = cx.node_source(node).unwrap_or_default().to_owned();
        let line = node
            .position()
            .map(|at| line_around(cx.source(), at.start.offset).to_owned())
            .unwrap_or_default();
        Some(
            MarkdownNode::new(
                Self::NAME,
                Chip {
                    due,
                    checked,
                    repeat,
                    line,
                },
            )
            .text(format!("@{}", due.format()))
            .markdown(source),
        )
    }

    fn render_inline(
        &self,
        node: &MarkdownNode,
        context: &InlineRenderContext,
        _: &mut Window,
        cx: &mut App,
    ) -> Option<InlineElement> {
        let chip = node.data::<Chip>()?;
        let now = crate::journal::now(cx);
        let colors = Colors::get(cx);
        let tone = tone(chip.due, chip.checked, now);
        let (text, fill) = chip_colors(tone, colors);
        let style = context.text_style();
        let size = context.font_size() * 0.8;
        let at = node.source_range().map_or(0, |r| r.start);
        let tooltip = chip_tooltip(chip.due, chip.repeat);
        let element = div()
            .id(ElementId::NamedInteger(Self::NAME.into(), at as u64))
            .h(context.line_height())
            .flex()
            .items_center()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(4.))
                    .px(size * 0.5)
                    .h(size * 1.55)
                    .rounded(size * 0.45)
                    .bg(fill)
                    .font_family(style.font_family.clone())
                    .font_weight(FontWeight::MEDIUM)
                    .text_size(size)
                    .line_height(size * 1.2)
                    .text_color(text)
                    .when(tone == Tone::Done, |el| el.line_through())
                    .child(
                        gpui_kit::component::Icon::new(gpui_kit::assets::IconName::Calendar)
                            .size(size * 0.95)
                            .text_color(text),
                    )
                    .child(repeat_label(
                        chip_label(chip.due, chip.checked, now),
                        chip.repeat,
                    )),
            )
            .tooltip(move |window, cx| {
                gpui_kit::component::tooltip::Tooltip::new(tooltip.clone()).build(window, cx)
            });
        // An open repeating task's chip menu: "Stop Repeating" keeps the
        // date and drops the `every …` words.
        let element = if chip.repeat.is_some() && !chip.checked && !chip.line.is_empty() {
            let edits = self.edits.clone();
            let line = chip.line.clone();
            element
                .context_menu(move |menu, _, _| {
                    let edits = edits.clone();
                    let line = line.clone();
                    menu.item(PopupMenuItem::new(STOP_REPEATING).on_click(move |_, _, _| {
                        edits
                            .try_send(crate::preview::PreviewEdit::StopRepeating {
                                shown_line: line.clone(),
                            })
                            .ok();
                    }))
                })
                .into_any_element()
        } else {
            element.into_any_element()
        };
        Some(InlineElement::new(element))
    }
}

// MARK: The editor

/// The tints the editor gives the due dates in its text, kept as text
/// decorations that follow edits, and worked out again a moment after
/// typing stops.
#[derive(Default)]
pub struct DueMarks {
    collection: Option<TextDecorationCollection>,
    task: Option<Task<()>>,
    /// How many dates are tinted now.
    count: usize,
}

impl DueMarks {
    /// Tints the dates in `body`'s text now.
    pub fn refresh(&mut self, body: &Entity<EditorState>, cx: &mut App) {
        self.task = None;
        let text = body.read(cx).value();
        let now = crate::journal::now(cx);
        let colors = Colors::get(cx);
        let decorations: Vec<TextDecoration> = marks(&text, now)
            .into_iter()
            .map(|(range, tone)| {
                let (color, fill) = chip_colors(tone, colors);
                TextDecoration::new(
                    range,
                    HighlightStyle {
                        color: Some(color),
                        background_color: Some(fill),
                        ..Default::default()
                    },
                )
            })
            .collect();
        self.count = decorations.len();
        match &self.collection {
            Some(collection) => collection.set(decorations, cx),
            None if decorations.is_empty() => {}
            None => {
                self.collection = Some(body.update(cx, |state, cx| {
                    state.create_decorations_collection(decorations, cx)
                }));
            }
        }
    }

    /// How many dates are tinted now.
    #[cfg(test)]
    pub fn count(&self) -> usize {
        self.count
    }

    /// Tints the dates once typing pauses. The decorations already there
    /// follow the edits meanwhile.
    pub fn refresh_soon<V: 'static>(
        &mut self,
        body: Entity<EditorState>,
        owner: WeakEntity<V>,
        pick: fn(&mut V) -> &mut DueMarks,
        cx: &mut App,
    ) {
        self.task = Some(cx.spawn(async move |cx| {
            cx.background_executor().timer(MARK_DELAY).await;
            cx.update(|cx| {
                if let Some(owner) = owner.upgrade() {
                    owner.update(cx, |owner, cx| {
                        let marks = pick(owner);
                        marks.task = None;
                        marks.refresh(&body, cx);
                    });
                }
            });
        }));
    }
}

// MARK: Autocomplete

/// `@` autocomplete for the date of the task the caret is in: the dates
/// [`date_choices`] offers, each replacing the `@` and what follows it; and
/// once a date is there, the repeats [`repeat_choices`] offers after it.
/// `None` when the caret isn't after an `@` in a task, or just past a date.
pub fn completions(text: &Rope, offset: usize, cx: &App) -> Option<Vec<CompletionItem>> {
    let offset = offset.min(text.len());
    let position = text.offset_to_position(offset);
    let line_start = text.line_start_offset(position.line as usize);
    let before = text.slice(line_start..offset).to_string();
    if let Some((from, query)) = repeat_query(&before) {
        let range = lsp_types::Range {
            start: text.offset_to_position(line_start + from),
            end: text.offset_to_position(offset),
        };
        let items = repeat_choices(query)
            .into_iter()
            .map(|(label, words)| CompletionItem {
                label,
                detail: Some(words.clone()),
                kind: Some(CompletionItemKind::VALUE),
                filter_text: Some(String::new()),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                    range,
                    new_text: words,
                })),
                ..Default::default()
            })
            .collect::<Vec<_>>();
        return (!items.is_empty()).then_some(items);
    }
    let (at, query) = date_query(&before)?;
    let today = crate::journal::now(cx).date;
    let range = lsp_types::Range {
        start: text.offset_to_position(line_start + at),
        end: text.offset_to_position(offset),
    };
    let items = date_choices(query, today)
        .into_iter()
        .map(|(label, iso)| CompletionItem {
            label,
            detail: Some(iso.clone()),
            kind: Some(CompletionItemKind::VALUE),
            filter_text: Some(String::new()),
            text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                range,
                new_text: format!("@{iso}"),
            })),
            ..Default::default()
        })
        .collect::<Vec<_>>();
    (!items.is_empty()).then_some(items)
}

#[cfg(test)]
mod stop_tests {
    use super::{line_around, rewrite_for_preview, stop_repeating_shown};
    use gpui_kit::SharedString;
    use proptest::prelude::*;
    use proptest::test_runner::{Config, RngAlgorithm, RngSeed};

    fn shown(note: &str) -> String {
        rewrite_for_preview(&SharedString::from(note.to_owned())).to_string()
    }

    #[test]
    fn a_chip_names_its_task_by_its_shown_line() {
        let note = "# Week\n- [ ] meds @2026-11-13 every day\n- [ ] walk @2026-11-13 every day\n";
        let page = shown(note);
        let line = page.lines().nth(2).unwrap();
        let at = page.find(line).unwrap() + line.find('@').unwrap();
        assert_eq!(line_around(&page, at), line);
        assert_eq!(
            stop_repeating_shown(note, &page, line).unwrap(),
            "# Week\n- [ ] meds @2026-11-13 every day\n- [ ] walk @2026-11-13\n"
        );
        // A line that isn't there, or doesn't repeat, does nothing.
        assert_eq!(stop_repeating_shown(note, &page, "# Week"), None);
        assert_eq!(stop_repeating_shown(note, &page, "gone"), None);
    }

    fn config() -> Config {
        Config {
            cases: 256,
            rng_seed: RngSeed::Fixed(20_261_017),
            rng_algorithm: RngAlgorithm::ChaCha,
            failure_persistence: None,
            ..Config::default()
        }
    }

    fn gen_line() -> impl Strategy<Value = String> {
        (
            prop_oneof![Just("- [ ] "), Just("- [x] "), Just("text ")],
            "[a-z]{1,6}",
            prop_oneof![
                Just(""),
                Just(" every week"),
                Just(" every 2 days after done")
            ],
        )
            .prop_map(|(lead, words, rule)| format!("{lead}{words} @2026-11-13{rule}"))
    }

    proptest! {
        #![proptest_config(config())]

        /// A chip's shown line stops the repeat of a task on a line that
        /// reads the same and nothing else.
        #[test]
        fn prop_a_shown_line_stops_its_own_task(
            lines in prop::collection::vec(gen_line(), 1..6),
            pick in any::<prop::sample::Index>(),
        ) {
            let note = format!("{}\n", lines.join("\n"));
            let page = shown(&note);
            let ix = pick.index(lines.len());
            let line = page.lines().nth(ix).unwrap();
            let offset: usize = page.lines().take(ix).map(|l| l.len() + 1).sum();
            prop_assert_eq!(line_around(&page, offset + line.len() / 2), line);
            match stop_repeating_shown(&note, &page, line) {
                Some(out) => {
                    let changed: Vec<usize> = note.lines().zip(out.lines()).enumerate()
                        .filter(|(_, (a, b))| a != b).map(|(i, _)| i).collect();
                    prop_assert_eq!(changed.len(), 1);
                    prop_assert_eq!(page.lines().nth(changed[0]).unwrap(), line);
                    prop_assert!(!out.lines().nth(changed[0]).unwrap().contains("every"));
                }
                None => {
                    for (i, shown_line) in page.lines().enumerate() {
                        if shown_line == line {
                            let raw = note.lines().nth(i).unwrap();
                            prop_assert!(!(raw.starts_with("- [ ] ") && raw.contains("every")));
                        }
                    }
                }
            }
        }
    }
}
