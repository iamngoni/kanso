//! Flashcards: review sessions, answers, stats, the editor's card marks,
//! suggested cards, Quiz Me and the settings.

use flutter_rust_bridge::frb;
use kanso_mobile::{Cards, KansoError};

use super::ai::KansoAiRun;
use super::cards_types::{
    Answered, CardPair, CardScopeArg, CardSettingsView, GradeEvent, NoteCardView, QuizEvent,
    ReviewCard, ReviewCounts, ReviewStats, SuggestEvent,
};
use crate::frb_generated::StreamSink;

#[frb(opaque)]
pub struct KansoCards(Cards);

impl KansoCards {
    pub(super) fn new(cards: Cards) -> KansoCards {
        KansoCards(cards)
    }

    pub fn settings(&self) -> Result<CardSettingsView, KansoError> {
        self.0.settings()
    }

    pub fn set_settings(&self, input: CardSettingsView) -> Result<CardSettingsView, KansoError> {
        self.0.set_settings(input)
    }

    /// Counts for a scope; `utc_offset_minutes` decides where today starts.
    pub fn counts(
        &self,
        scope: CardScopeArg,
        utc_offset_minutes: i32,
    ) -> Result<ReviewCounts, KansoError> {
        self.0.counts(scope, utc_offset_minutes)
    }

    /// The cards to show now, at most `limit`.
    pub fn due(
        &self,
        scope: CardScopeArg,
        limit: u32,
        utc_offset_minutes: i32,
    ) -> Result<Vec<ReviewCard>, KansoError> {
        self.0.due(scope, limit, utc_offset_minutes)
    }

    /// Records an answer: 1 Again, 2 Hard, 3 Good, 4 Easy.
    pub fn answer(
        &self,
        note_id: String,
        card_key: String,
        grade: u8,
        utc_offset_minutes: i32,
    ) -> Result<Answered, KansoError> {
        self.0.answer(note_id, card_key, grade, utc_offset_minutes)
    }

    pub fn suspend(
        &self,
        note_id: String,
        card_key: String,
        suspended: bool,
    ) -> Result<(), KansoError> {
        self.0.suspend(note_id, card_key, suspended)
    }

    pub fn reset_history(&self) -> Result<u32, KansoError> {
        self.0.reset_history()
    }

    pub fn stats(
        &self,
        scope: CardScopeArg,
        utc_offset_minutes: i32,
    ) -> Result<ReviewStats, KansoError> {
        self.0.stats(scope, utc_offset_minutes)
    }

    pub fn note_cards(&self, note_id: String) -> Result<Vec<NoteCardView>, KansoError> {
        self.0.note_cards(note_id)
    }

    /// Why **Make Flashcards** is not offered for a note, or null.
    pub fn make_blocked(&self, note_id: String) -> Result<Option<String>, KansoError> {
        self.0.make_blocked(note_id)
    }

    /// Cards the model suggests for a note. A request that can't start is one
    /// `Failed` event with the reason.
    pub fn suggest(&self, note_id: String, run: &KansoAiRun, sink: StreamSink<SuggestEvent>) {
        let refused = sink.clone();
        if let Err(err) = self.0.suggest(
            note_id,
            &run.0,
            Box::new(move |event| sink.add(event).is_ok()),
        ) {
            refused
                .add(SuggestEvent::Failed {
                    message: err.to_string(),
                })
                .ok();
        }
    }

    /// Writes cards under `## Flashcards` in a note; one edit.
    pub fn add_cards(&self, note_id: String, cards: Vec<CardPair>) -> Result<u32, KansoError> {
        self.0.add_cards(note_id, cards)
    }

    /// Questions from the given notes (those that may be sent).
    pub fn quiz(&self, note_ids: Vec<String>, run: &KansoAiRun, sink: StreamSink<QuizEvent>) {
        let refused = sink.clone();
        if let Err(err) = self.0.quiz(
            note_ids,
            &run.0,
            Box::new(move |event| sink.add(event).is_ok()),
        ) {
            refused
                .add(QuizEvent::Failed {
                    message: err.to_string(),
                })
                .ok();
        }
    }

    /// Grades an answer against the note the question came from.
    pub fn grade(
        &self,
        question: String,
        note_id: String,
        answer: String,
        run: &KansoAiRun,
        sink: StreamSink<GradeEvent>,
    ) {
        let refused = sink.clone();
        if let Err(err) = self.0.grade(
            question,
            note_id,
            answer,
            &run.0,
            Box::new(move |event| sink.add(event).is_ok()),
        ) {
            refused
                .add(GradeEvent::Failed {
                    message: err.to_string(),
                })
                .ok();
        }
    }

    /// The daily reminder's words (a count), or null when nothing is due.
    pub fn reminder_text(&self, utc_offset_minutes: i32) -> Result<Option<String>, KansoError> {
        self.0.reminder_text(utc_offset_minutes)
    }
}
