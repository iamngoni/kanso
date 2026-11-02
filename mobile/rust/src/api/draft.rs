//! Drafting from notes: the sources, the streamed draft with its checked
//! citations, the review step and saving.

use flutter_rust_bridge::frb;
use kanso_mobile::{Drafting, KansoError};

use super::ai::KansoAiRun;
use super::draft_types::{
    DraftEvent, DraftLength, DraftPreview, DraftShape, DraftSourceRef, DraftSources,
};
use crate::frb_generated::StreamSink;

#[frb(opaque)]
pub struct KansoDrafting(Drafting);

impl KansoDrafting {
    pub(super) fn new(drafting: Drafting) -> KansoDrafting {
        KansoDrafting(drafting)
    }

    pub fn notes_of_notebook(&self, book_id: String) -> Result<Vec<String>, KansoError> {
        self.0.notes_of_notebook(book_id)
    }

    pub fn notes_of_tag(&self, tag_id: String) -> Result<Vec<String>, KansoError> {
        self.0.notes_of_tag(tag_id)
    }

    /// What a draft from these notes would read, and what is left out.
    pub fn sources(&self, note_ids: Vec<String>) -> Result<DraftSources, KansoError> {
        self.0.sources(note_ids)
    }

    /// The draft's events, until it ends or Dart stops listening. A draft
    /// that can't start (AI off, nothing to read) is one `Failed` event.
    pub fn write(
        &self,
        note_ids: Vec<String>,
        shape: DraftShape,
        request: String,
        length: DraftLength,
        run: &KansoAiRun,
        sink: StreamSink<DraftEvent>,
    ) {
        let refused = sink.clone();
        if let Err(err) = self.0.write(
            note_ids,
            shape,
            request,
            length,
            &run.0,
            Box::new(move |event| sink.add(event).is_ok()),
        ) {
            refused
                .add(DraftEvent::Failed {
                    message: err.to_string(),
                })
                .ok();
        }
    }

    /// The note a finished draft would be saved as.
    pub fn preview(
        &self,
        text: String,
        sources: Vec<DraftSourceRef>,
    ) -> Result<DraftPreview, KansoError> {
        self.0.preview(text, sources)
    }

    /// Saves the reviewed draft as a note; returns its ID.
    pub fn save(
        &self,
        title: String,
        markdown: String,
        book_id: String,
        source_count: u32,
    ) -> Result<String, KansoError> {
        self.0.save(title, markdown, book_id, source_count)
    }
}
