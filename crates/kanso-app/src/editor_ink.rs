//! The open editor and handwriting read in the background (`ink_ocr.rs`).
//!
//! A background read writes the words it found into the canvas body
//! (`kansoInkOcr`). To the editor that is a change made elsewhere, and
//! taken as one it would reload the canvas (dropping its undo steps and a
//! stroke in progress) or ask which version to keep. It is neither: when
//! nothing but the words changed, the editor takes them in place. And the
//! reader asks [`editing`] first, so it never writes over a note someone is
//! working on right now.

use gpui_kit::*;
use kanso_core::Note;
use kanso_core::canvas::{self, InkOcr};
use kanso_engine::saving::same_content;

use super::NoteEditor;
use super::peers::open_editors;

/// Whether note `id` is being edited in any open editor right now: typing
/// or a canvas change not saved yet, a gesture under way, or a question
/// about a change made elsewhere open. The background reader leaves such a
/// note until it is still again.
pub fn editing(id: &str, cx: &App) -> bool {
    open_editors(cx).into_iter().any(|editor| {
        let editor = editor.read(cx);
        editor.note_id() == Some(id)
            && (editor.has_unsaved_edits(cx)
                || editor.incoming.is_some()
                || editor.canvas.read(cx).is_drawing())
    })
}

/// The words `latest` holds, when it is `note` with nothing changed but
/// the handwriting a background read kept. Pure.
pub(super) fn ink_text_only(note: &Note, latest: &Note) -> Option<Vec<InkOcr>> {
    if note.doctype != "canvas"
        || !same_content(
            &Note {
                body: latest.body.clone(),
                ..note.clone()
            },
            latest,
        )
    {
        return None;
    }
    canvas::ink_text_change(&note.body, &latest.body)
}

/// `body` with its handwriting words replaced by `words`, when it is a
/// canvas.
fn with_words(body: &str, words: &[InkOcr]) -> Option<String> {
    let mut doc = canvas::parse(body).ok()?;
    doc.kanso_ink_ocr = words.to_vec();
    Some(canvas::to_body(&doc))
}

impl NoteEditor {
    /// The store has `latest` for the open canvas, which only adds the
    /// handwriting words a background read kept: the canvas takes them in
    /// place, unsaved drawing and all, and `latest` becomes the note as last
    /// saved. Says whether it did.
    pub(super) fn adopt_ink_text(&mut self, latest: &Note, cx: &mut Context<Self>) -> bool {
        let (Some(note), Some(shown)) = (&self.note, &self.canvas_body) else {
            return false;
        };
        let Some(words) = ink_text_only(note, latest) else {
            return false;
        };
        let shown = if *shown == note.body {
            latest.body.clone()
        } else {
            match with_words(shown, &words) {
                Some(body) => body,
                None => return false,
            }
        };
        self.canvas_body = Some(shown);
        self.canvas.update(cx, |view, _| view.take_ink_text(words));
        self.note = Some(latest.clone());
        cx.notify();
        true
    }

    /// A save of the open canvas met `latest`, which `base` became through
    /// nothing but a background read's words: `note`, the version being
    /// saved, takes those words and goes on top of `latest`. Says whether
    /// it did.
    pub(super) fn save_over_ink_text(
        &mut self,
        base: &Note,
        latest: &Note,
        note: &mut Note,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.canvas_body.is_none() {
            return false;
        }
        let Some(words) = ink_text_only(base, latest) else {
            return false;
        };
        let Some(body) = with_words(&note.body, &words) else {
            return false;
        };
        note.body = body.clone();
        note.rev = latest.rev.clone();
        self.canvas_body = Some(body);
        self.canvas.update(cx, |view, _| view.take_ink_text(words));
        true
    }
}

#[cfg(test)]
mod tests {
    use super::ink_text_only;
    use kanso_core::Note;
    use kanso_core::canvas::{self, Canvas, INK, Node, Point, Stroke};
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    fn board(strokes: usize) -> Canvas {
        let mut c = Canvas::default();
        c.put_node(
            Node {
                id: "i".into(),
                kind: INK.into(),
                width: 10.0,
                height: 10.0,
                ..Node::default()
            },
            1,
        );
        for s in 0..strokes {
            c.add_stroke(
                "i",
                Stroke {
                    id: format!("s{s}"),
                    tool: "pen".into(),
                    width: 2.0,
                    points: (0..4)
                        .map(|p| Point::new(f64::from(p as u32 * 3), s as f64, 0.5))
                        .collect(),
                    ..Stroke::default()
                },
                2,
            );
        }
        c
    }

    fn note(body: String) -> Note {
        serde_json::from_value(serde_json::json!({
            "_id": "note:a", "_rev": "1-a", "title": "Board", "doctype": "canvas",
            "body": body, "bookId": "book:a", "updatedAt": 5,
        }))
        .unwrap()
    }

    fn read(mut c: Canvas) -> Canvas {
        for job in c.ink_to_read() {
            c.keep_ink_text(&job.node, &job.md5, "fake", "words");
        }
        c
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 32,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        })]

        /// Only a version that adds nothing but the words is taken in
        /// place: a title, a stroke or a later edit time is a real change.
        #[test]
        fn prop_only_the_words_are_taken_in_place(strokes in 1usize..5) {
            let c = board(strokes);
            let open = note(canvas::to_body(&c));
            let mut latest = note(canvas::to_body(&read(c.clone())));
            latest.rev = Some("2-b".into());
            prop_assert!(ink_text_only(&open, &latest).is_some());
            let mut retitled = latest.clone();
            retitled.title = "Other".into();
            prop_assert!(ink_text_only(&open, &retitled).is_none());
            let mut later = latest.clone();
            later.updated_at = 9;
            prop_assert!(ink_text_only(&open, &later).is_none());
            let drawn = note(canvas::to_body(&read(board(strokes + 1))));
            prop_assert!(ink_text_only(&open, &drawn).is_none());
            let mut text = open.clone();
            text.doctype = String::new();
            prop_assert!(ink_text_only(&text, &latest).is_none());
        }
    }
}
