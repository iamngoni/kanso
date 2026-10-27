//! Discovery: related notes on and off with their status, what the
//! embeddings say about a note, search by meaning, the graph, the words in
//! images, and what the model costs on the phone.

use crate::frb_generated::StreamSink;
use flutter_rust_bridge::frb;
use kanso_mobile::{
    Discovery, DiscoveryMeasure, GraphLayout, GraphRequest, ImageToRead, KansoError, MeaningResult,
    NoteMeaning, NoteView, RelatedRow, RelatedState,
};

#[frb(opaque)]
pub struct KansoDiscovery(Discovery);

impl KansoDiscovery {
    pub(super) fn new(discovery: Discovery) -> KansoDiscovery {
        KansoDiscovery(discovery)
    }

    /// Whether related notes are on.
    pub fn related_enabled(&self) -> Result<bool, KansoError> {
        self.0.related_enabled()
    }

    /// Turns related notes on (the model is downloaded once, then notes are
    /// embedded in the background) or off (the embeddings are forgotten).
    pub fn set_related_notes(&self, on: bool) -> Result<(), KansoError> {
        self.0.set_related_notes(on)
    }

    pub fn related_state(&self) -> Result<RelatedState, KansoError> {
        self.0.related_state()
    }

    /// Where related notes stand now and each time that changes, until Dart
    /// stops listening.
    pub fn related_stream(&self, sink: StreamSink<RelatedState>) {
        self.0.related_stream(move |state| sink.add(state).is_ok());
    }

    /// Where the model's files are kept.
    #[frb(sync)]
    pub fn model_dir(&self) -> String {
        self.0.model_dir().to_string_lossy().into_owned()
    }

    pub fn related_notes(&self, id: String, limit: u32) -> Result<Vec<RelatedRow>, KansoError> {
        self.0.related_notes(&id, limit)
    }

    pub fn note_meaning(&self, id: String) -> Result<NoteMeaning, KansoError> {
        self.0.note_meaning(&id)
    }

    pub fn dismiss_twin(&self, id: String, other_id: String) -> Result<(), KansoError> {
        self.0.dismiss_twin(&id, &other_id)
    }

    pub fn merge_notes(&self, keep_id: String, other_id: String) -> Result<NoteView, KansoError> {
        self.0.merge_notes(&keep_id, &other_id)
    }

    pub fn search_by_meaning(&self, query: String, k: u32) -> Result<MeaningResult, KansoError> {
        self.0.search_by_meaning(&query, k)
    }

    pub fn graph(&self, request: GraphRequest) -> Result<GraphLayout, KansoError> {
        self.0.graph(&request)
    }

    pub fn ocr_next(&self) -> Result<Option<ImageToRead>, KansoError> {
        self.0.ocr_next()
    }

    pub fn ocr_keep(
        &self,
        id: String,
        md5: String,
        text: String,
        engine: String,
    ) -> Result<bool, KansoError> {
        self.0.ocr_keep(&id, &md5, &text, &engine)
    }

    pub fn ocr_waiting(&self) -> Result<u32, KansoError> {
        self.0.ocr_waiting()
    }

    /// Loads the downloaded model and embeds sample notes, to say what it
    /// costs on this phone.
    pub fn measure(&self) -> Result<DiscoveryMeasure, KansoError> {
        self.0.measure()
    }
}

/// The words of `query` worth searching by meaning, or `None` when it is
/// too short or has qualifiers (`tag:`, `-`, quotes).
#[frb(sync)]
pub fn meaning_query(query: String) -> Option<String> {
    Discovery::meaning_query(&query)
}

/// The fewest letters and digits a search needs to be searched by meaning.
#[frb(sync)]
pub fn meaning_min_chars() -> u32 {
    Discovery::MIN_QUERY_CHARS
}
