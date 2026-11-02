//! Inbox triage: cards of suggestions for the Inbox's notes, rejecting and
//! editing chips, the model's ranking, filing what was accepted, and the
//! settings.

use flutter_rust_bridge::frb;
use kanso_mobile::{KansoError, Triage};

use super::triage_types::{
    TriageCard, TriageChip, TriageField, TriageFiled, TriageFiling, TriageSettings, TriageStatus,
};

#[frb(opaque)]
pub struct KansoTriage(Triage);

impl KansoTriage {
    pub(super) fn new(triage: Triage) -> KansoTriage {
        KansoTriage(triage)
    }

    pub fn settings(&self) -> TriageSettings {
        self.0.settings()
    }

    pub fn set_settings(&self, input: TriageSettings) -> Result<TriageSettings, KansoError> {
        self.0.set_settings(input)
    }

    /// How many notes wait, and what is switched on.
    pub fn status(&self) -> Result<TriageStatus, KansoError> {
        self.0.status()
    }

    /// Cards for the notes Triage shows, newest first.
    pub fn cards(&self, limit: u32) -> Result<Vec<TriageCard>, KansoError> {
        self.0.cards(limit)
    }

    /// One note's card, made again after it changed elsewhere.
    pub fn card(&self, note_id: String) -> Result<Option<TriageCard>, KansoError> {
        self.0.card(&note_id)
    }

    /// The card's chips ranked by the person's AI; they cite their notes.
    pub fn refine(&self, note_id: String) -> Result<Vec<TriageChip>, KansoError> {
        self.0.refine(&note_id)
    }

    pub fn reject(
        &self,
        note_id: String,
        field: TriageField,
        value: String,
    ) -> Result<(), KansoError> {
        self.0.reject(&note_id, field, &value)
    }

    pub fn unreject(
        &self,
        note_id: String,
        field: TriageField,
        value: String,
    ) -> Result<(), KansoError> {
        self.0.unreject(&note_id, field, &value)
    }

    /// The chip a typed value makes, or why it can't.
    pub fn resolve(&self, field: TriageField, text: String) -> Result<TriageChip, KansoError> {
        self.0.resolve(field, &text)
    }

    /// What filing would do, without writing anything.
    pub fn plan(&self, filings: Vec<TriageFiling>) -> Result<TriageFiled, KansoError> {
        self.0.plan(filings)
    }

    /// Files the accepted chips in one write; `undo` is the ticket for
    /// `KansoLibrary.undo`.
    pub fn file(&self, filings: Vec<TriageFiling>) -> Result<TriageFiled, KansoError> {
        self.0.file(filings)
    }
}
