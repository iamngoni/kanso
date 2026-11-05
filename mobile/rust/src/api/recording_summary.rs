//! Summarise Recording (issue #38, step 12): whether it is offered for a
//! note, asking the person's own AI for a summary and action items, and
//! writing the ones they ticked above the transcript.

use flutter_rust_bridge::frb;
use kanso_mobile::{KansoError, Library};

use super::ai::KansoAiRun;
use super::recording_summary_types::{SummaryApplied, SummaryDraftView, SummaryOffer};

#[frb(opaque)]
pub struct KansoRecordingSummary(Library);

impl KansoRecordingSummary {
    pub(super) fn new(library: Library) -> KansoRecordingSummary {
        KansoRecordingSummary(library)
    }

    /// Whether Summarise Recording is offered for note `id`, and what to say
    /// before anything is sent. Sends nothing.
    pub fn offer(&self, id: String) -> Result<SummaryOffer, KansoError> {
        self.0.ai().summary_offer(&id)
    }

    /// Asks the person's own AI for a summary of note `id`'s recording.
    /// Blocks until it has answered; `run` (from `KansoAi::new_run`) stops
    /// it. Nothing is written.
    pub fn summarize(&self, id: String, run: &KansoAiRun) -> Result<SummaryDraftView, KansoError> {
        self.0.ai().summarize_recording(&id, &run.0)
    }

    /// Writes `draft` with the action items at `ticked` above the
    /// transcript, in one change with an Undo ticket.
    pub fn apply(
        &self,
        id: String,
        draft: SummaryDraftView,
        ticked: Vec<u32>,
    ) -> Result<SummaryApplied, KansoError> {
        self.0.ai().apply_recording_summary(&id, draft, ticked)
    }
}
