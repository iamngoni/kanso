//! Flashcards where the note is written (issue #198): the `::` of a line
//! like `question :: answer` is dimmed, and the question gets a quiet tint,
//! so a line that is a card reads as one without changing its text. Off in
//! Settings › Flashcards ("Show card icons in the editor").
//!
//! Like the due-date tints these are text decorations that follow edits and
//! are worked out again a moment after typing stops.

use std::time::Duration;

use gpui_kit::base::input::{TextDecoration, TextDecorationCollection};
use gpui_kit::component::input::EditorState;
use gpui_kit::*;

use crate::theme::Colors;
use crate::workspace::AppState;

/// How long the editor waits after typing stops to mark cards.
const MARK_DELAY: Duration = Duration::from_millis(200);

/// The decorations for the cards in `text`: the separator dimmed, the
/// question tinted. Pure over the text and the two colors.
// no-prop-test: delegates to kanso_core::cards::card_marks, property-tested
// there; this only picks the colors.
pub fn decorations(text: &str, dim: Hsla, tint: Hsla) -> Vec<TextDecoration> {
    let mut out = Vec::new();
    for mark in kanso_core::cards::card_marks(text) {
        let sep = mark.separator;
        // The question is the line up to the space before the `::`, past
        // any list marker, which the tint simply includes.
        let line = text[mark.line.start..sep.start].trim_end();
        let question = mark.line.start..mark.line.start + line.len();
        if question.start < question.end {
            out.push(TextDecoration::new(
                question,
                HighlightStyle {
                    background_color: Some(tint),
                    ..Default::default()
                },
            ));
        }
        out.push(TextDecoration::new(
            sep,
            HighlightStyle {
                color: Some(dim),
                ..Default::default()
            },
        ));
    }
    out
}

/// The card marks of one editor, kept as decorations that follow edits.
#[derive(Default)]
pub struct CardMarks {
    collection: Option<TextDecorationCollection>,
    task: Option<Task<()>>,
}

impl CardMarks {
    /// Marks the cards in `body`'s text now.
    pub fn refresh(&mut self, body: &Entity<EditorState>, cx: &mut App) {
        self.task = None;
        let show = cx
            .try_global::<AppState>()
            .and_then(|state| state.store.card_settings().ok())
            .is_none_or(|s| s.show_icons);
        let decorations = if show {
            let text = body.read(cx).value();
            let colors = Colors::get(cx);
            decorations(
                &text,
                Hsla::from(colors.text_muted),
                Hsla::from(colors.accent).opacity(0.10),
            )
        } else {
            Vec::new()
        };
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

    /// Marks the cards once typing pauses.
    pub fn refresh_soon<V: 'static>(
        &mut self,
        body: Entity<EditorState>,
        owner: WeakEntity<V>,
        pick: fn(&mut V) -> &mut CardMarks,
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

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use gpui_kit::Hsla;

    use super::decorations;

    #[test]
    fn a_card_line_gets_a_tint_and_a_dimmed_separator() {
        let d = decorations("Q :: A\nplain\n", Hsla::black(), Hsla::white());
        assert_eq!(d.len(), 2);
    }

    proptest! {
        /// However the text reads, every decoration lies inside it, on
        /// character boundaries.
        #[test]
        fn prop_decorations_stay_inside_the_text(text in "[a-z :]{0,60}") {
            for d in decorations(&text, Hsla::black(), Hsla::white()) {
                let r = d.range.clone();
                prop_assert!(r.end <= text.len());
                prop_assert!(text.is_char_boundary(r.start) && text.is_char_boundary(r.end));
            }
        }
    }
}
