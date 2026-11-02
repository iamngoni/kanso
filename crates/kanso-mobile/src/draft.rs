//! Drafting from notes on the phone (issue #198): pick notes, say what to
//! write, stream a draft with its citations checked, review it and save it
//! as a note.
//!
//! The sources are chosen, ranked and checked by `kanso_engine::draft`; the
//! prompts, the map-reduce and the citation check are `kanso_ai::draft`.
//! This is the shell's side: the plain data, the model call on a thread,
//! and UTF-16 offsets for the spans (what Dart indexes strings by).
//!
//! Nothing is saved until `save` is called with a draft the person
//! reviewed. A locked note, a note in a locked notebook, a trashed note and
//! a note tagged `#ai-exclude` are never a source.

use std::sync::Arc;
use std::sync::atomic::Ordering;

use kanso_ai::draft::SourceRef;
use kanso_ai::draft_model::{CheckedDraft, Length, Shape, SpanKind};
use kanso_engine::draft as engine;

use crate::ai::{AiHandle, AiRun, Emitter, spawn};
use crate::boundary::{guard, held};
use crate::draft_model::{
    DraftCite, DraftEvent, DraftLength, DraftPreview, DraftShape, DraftSourceNote, DraftSourceRef,
    DraftSources, DraftSpan, DraftView,
};
use crate::errors::KansoError;
use crate::library::Library;
use crate::stream::Sink;

/// The drafting calls of one library.
#[derive(Clone)]
pub struct Drafting {
    ai: AiHandle,
}

impl Library {
    /// Drafting from notes: sources, the streamed draft, review and save.
    pub fn drafting(&self) -> Drafting {
        Drafting { ai: self.ai() }
    }

    /// [`Library::drafting`] with the AI handle `ai` (a test's).
    #[cfg(any(test, feature = "test-support"))]
    pub fn drafting_with_ai(&self, ai: AiHandle) -> Drafting {
        Drafting { ai }
    }
}

fn store_error(err: kanso_store::StoreError) -> KansoError {
    KansoError::storage(err.to_string())
}

fn shape_of(shape: DraftShape) -> Shape {
    match shape {
        DraftShape::Report => Shape::Report,
        DraftShape::Essay => Shape::Essay,
        DraftShape::Update => Shape::Update,
        DraftShape::Outline => Shape::Outline,
        DraftShape::Custom => Shape::Custom,
    }
}

fn length_of(length: DraftLength) -> Length {
    match length {
        DraftLength::Short => Length::Short,
        DraftLength::Medium => Length::Medium,
        DraftLength::Long => Length::Long,
    }
}

/// UTF-16 units before byte `at` of `text`.
fn utf16(text: &str, at: usize) -> u32 {
    text[..at.min(text.len())]
        .chars()
        .map(|c| c.len_utf16() as u32)
        .sum()
}

fn view_of(checked: &CheckedDraft) -> DraftView {
    DraftView {
        spans: checked
            .spans
            .iter()
            .map(|span| {
                let (cites, unsourced) = match &span.kind {
                    SpanKind::Cited(cites) => (
                        cites
                            .iter()
                            .map(|c| DraftCite {
                                number: c.number as u32,
                                heading: c.heading.clone(),
                            })
                            .collect(),
                        false,
                    ),
                    SpanKind::Unsourced => (Vec::new(), true),
                };
                DraftSpan {
                    start: utf16(&checked.text, span.start),
                    end: utf16(&checked.text, span.end),
                    cites,
                    unsourced,
                }
            })
            .collect(),
        removed: checked.removed as u32,
        not_covered: engine::not_covered(&checked.text),
        text: checked.text.clone(),
    }
}

impl Drafting {
    /// The notes of a notebook (and the notebooks inside it), newest
    /// first, at most 200.
    pub fn notes_of_notebook(&self, book_id: String) -> Result<Vec<String>, KansoError> {
        guard(|| engine::notes_of_book(&self.ai.inner.store, &book_id).map_err(store_error))
    }

    /// The notes with a tag, newest first, at most 200.
    pub fn notes_of_tag(&self, tag_id: String) -> Result<Vec<String>, KansoError> {
        guard(|| engine::notes_of_tag(&self.ai.inner.store, &tag_id).map_err(store_error))
    }

    /// What a draft from `note_ids` would read, and what is left out.
    pub fn sources(&self, note_ids: Vec<String>) -> Result<DraftSources, KansoError> {
        guard(|| {
            let list = engine::resolve(&self.ai.inner.store, &note_ids);
            Ok(DraftSources {
                summary: list.summary(),
                left_out_locked: list.left_out_locked as u32,
                left_out_excluded: list.left_out_excluded as u32,
                too_many: list.too_many,
                notes: list
                    .notes
                    .into_iter()
                    .map(|n| DraftSourceNote {
                        id: n.id,
                        title: n.title,
                        words: n.words as u32,
                    })
                    .collect(),
            })
        })
    }

    /// Writes a draft from `note_ids`. Returns once it has started, or at
    /// once with why it can't: AI is off, nothing may be read, the request
    /// is empty. The draft arrives as [`DraftEvent`]s on `sink`.
    pub fn write(
        &self,
        note_ids: Vec<String>,
        shape: DraftShape,
        request: String,
        length: DraftLength,
        run: &AiRun,
        sink: Sink<DraftEvent>,
    ) -> Result<(), KansoError> {
        guard(|| {
            let request = request.trim().to_owned();
            if request.is_empty() && shape == DraftShape::Custom {
                return Err(KansoError::invalid("Say what the draft should be."));
            }
            let settings = self.ai.enabled()?;
            let provider = self.ai.provider_for(&settings)?;
            let store = self.ai.inner.store.clone();
            let list = engine::resolve(&store, &note_ids);
            if list.too_many {
                return Err(KansoError::invalid(
                    "That's more than a draft can read. Pick up to 200 notes.",
                ));
            }
            let sources = engine::sources(&store, &note_ids, &request);
            if sources.is_empty() {
                return Err(KansoError::invalid(
                    "None of these notes can be sent to AI. Locked notes and notes tagged #ai-exclude are never sent.",
                ));
            }
            self.ai.inner.metrics().count(crate::metrics::Feature::Ai);
            let cancel = run.cancel_flag();
            let emitter = Arc::new(Emitter::new(sink, cancel.clone()));
            let refs: Vec<DraftSourceRef> = engine::refs(&sources)
                .into_iter()
                .map(|r| DraftSourceRef {
                    number: r.number as u32,
                    id: r.id,
                    title: r.title,
                })
                .collect();
            spawn("kanso-draft", move || {
                let mut said = String::new();
                let mut ended = false;
                {
                    let provider = held(&provider);
                    kanso_ai::draft::write_draft(
                        provider.as_ref(),
                        shape_of(shape),
                        &request,
                        length_of(length),
                        &sources,
                        &cancel,
                        |event| match event {
                            kanso_ai::Event::Text(piece) => {
                                said.push_str(&piece);
                                emitter.send(DraftEvent::Text {
                                    draft: view_of(&engine::prepare(&said, &sources)),
                                });
                            }
                            kanso_ai::Event::Status(line) => {
                                emitter.send(DraftEvent::Status { line });
                            }
                            kanso_ai::Event::Done => {
                                ended = true;
                                if said.trim().is_empty() {
                                    emitter.finish(DraftEvent::Failed {
                                        message: "Nothing came back.".into(),
                                    });
                                } else {
                                    emitter.finish(DraftEvent::Done {
                                        draft: view_of(&engine::prepare(&said, &sources)),
                                        sources: refs.clone(),
                                    });
                                }
                            }
                            kanso_ai::Event::Error(message) => {
                                ended = true;
                                if message == kanso_ai::CUT_OFF {
                                    emitter.finish(DraftEvent::CutOff { message });
                                } else {
                                    emitter.finish(DraftEvent::Failed { message });
                                }
                            }
                        },
                    );
                }
                if !ended {
                    if cancel.load(Ordering::Relaxed) {
                        emitter.finish(DraftEvent::Stopped);
                    } else {
                        emitter.finish(DraftEvent::CutOff {
                            message: kanso_ai::CUT_OFF.to_owned(),
                        });
                    }
                }
            })
        })
    }

    /// The note `text` (a finished draft) would be saved as: its title, the
    /// Markdown with citations as links and a Sources section, and the
    /// notebook to offer.
    pub fn preview(
        &self,
        text: String,
        sources: Vec<DraftSourceRef>,
    ) -> Result<DraftPreview, KansoError> {
        guard(|| {
            let refs = refs_of(sources);
            Ok(DraftPreview {
                title: engine::title_of(&text),
                markdown: engine::final_markdown(&text, &refs),
                default_book_id: engine::default_book(&self.ai.inner.store, &refs),
            })
        })
    }

    /// Saves the reviewed draft as a new note in `book_id`. One write; the
    /// note's history names it "Drafted by AI from N notes". Returns the
    /// note's ID.
    pub fn save(
        &self,
        title: String,
        markdown: String,
        book_id: String,
        source_count: u32,
    ) -> Result<String, KansoError> {
        guard(|| {
            engine::save(
                &self.ai.inner.store,
                &title,
                &markdown,
                &book_id,
                source_count as usize,
            )
            .map_err(store_error)
        })
    }
}

fn refs_of(sources: Vec<DraftSourceRef>) -> Vec<SourceRef> {
    sources
        .into_iter()
        .map(|s| SourceRef {
            number: s.number as usize,
            id: s.id,
            title: s.title,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // no-prop-test: two fixed strings with astral characters.
    #[test]
    fn offsets_count_utf16_units() {
        assert_eq!(utf16("ab", 1), 1);
        assert_eq!(utf16("a😀b", 5), 3);
        assert_eq!(utf16("é", 2), 1);
    }
}
