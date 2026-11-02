//! Flashcards on the phone (issue #198): the calls behind the Review page,
//! the stats, the editor's card marks, suggested cards and Quiz Me.
//!
//! The cards are lines of the notes (`question :: answer`); their schedule
//! is replayed from the synced log of answers (`kanso_store`'s `cards`);
//! what needs a model (suggested cards, Quiz Me) is `kanso_engine`'s
//! `flashcards`, which the desktop uses too. This is the shell's side: the
//! plain data, the clock and the streams.
//!
//! Locked notes and notes in locked notebooks never become cards, and no
//! note kept out of AI reaches a prompt. A notification built here
//! carries a count and never a question.

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use kanso_core::now_ms;
use kanso_core::srs::{Grade, interval_label};
use kanso_engine::flashcards;
use kanso_store::{CardScope, CardSettings, Store};

use crate::ai::{AiHandle, AiRun, Emitter, spawn};
use crate::boundary::{guard, held};
use crate::cards_model::{
    Answered, CardPair, CardScopeArg, CardScopeKind, CardSettingsView, GradeEvent, HardCardView,
    NoteCardView, QuizEvent, QuizItem, QuizVerdict, ReviewCard, ReviewCounts, ReviewStats,
    SuggestEvent, SuggestedCard,
};
use crate::errors::KansoError;
use crate::library::Library;
use crate::stream::Sink;

const DEVICE_KEY: &str = "cards.device";

/// The flashcard calls of one library.
#[derive(Clone)]
pub struct Cards {
    ai: AiHandle,
}

impl Library {
    /// Flashcards: review sessions, stats, suggested cards and Quiz Me.
    pub fn cards(&self) -> Cards {
        Cards { ai: self.ai() }
    }

    /// [`Library::cards`] with the AI handle `ai` (a test's).
    #[cfg(any(test, feature = "test-support"))]
    pub fn cards_with_ai(&self, ai: AiHandle) -> Cards {
        Cards { ai }
    }
}

fn scope_of(arg: &CardScopeArg) -> CardScope {
    match arg.kind {
        CardScopeKind::All => CardScope::All,
        CardScopeKind::Notebook => CardScope::Book(arg.id.clone()),
        CardScopeKind::Tag => CardScope::Tag(arg.id.clone()),
        CardScopeKind::Note => CardScope::Note(arg.id.clone()),
    }
}

fn view_of(settings: &CardSettings) -> CardSettingsView {
    CardSettingsView {
        new_per_day: settings.new_per_day,
        reviews_per_day: settings.reviews_per_day,
        retention: settings.retention,
        reminder: settings.reminder.clone(),
        show_icons: settings.show_icons,
    }
}

fn store_error(err: kanso_store::StoreError) -> KansoError {
    KansoError::storage(err.to_string())
}

impl Cards {
    fn store(&self) -> &Store {
        &self.ai.inner.store
    }

    /// This install's name in the log of answers.
    fn device(&self) -> Result<String, KansoError> {
        if let Some(id) = self.store().meta(DEVICE_KEY).map_err(store_error)? {
            return Ok(id);
        }
        let id = format!("d{:012x}", fastrand::u64(..) >> 16);
        self.store()
            .set_meta(DEVICE_KEY, &id)
            .map_err(store_error)?;
        Ok(id)
    }

    pub fn settings(&self) -> Result<CardSettingsView, KansoError> {
        guard(|| Ok(view_of(&self.store().card_settings().map_err(store_error)?)))
    }

    pub fn set_settings(&self, input: CardSettingsView) -> Result<CardSettingsView, KansoError> {
        guard(|| {
            let reminder = input.reminder.filter(|t| valid_time(t));
            let settings = CardSettings {
                new_per_day: input.new_per_day.min(1_000),
                reviews_per_day: input.reviews_per_day.min(10_000),
                retention: input.retention,
                reminder,
                show_icons: input.show_icons,
            };
            self.store()
                .set_card_settings(&settings)
                .map_err(store_error)?;
            Ok(view_of(&self.store().card_settings().map_err(store_error)?))
        })
    }

    /// Counts for `scope`. `utc_offset_minutes` is the phone's zone, which
    /// decides where today starts.
    pub fn counts(
        &self,
        scope: CardScopeArg,
        utc_offset_minutes: i32,
    ) -> Result<ReviewCounts, KansoError> {
        guard(|| {
            let (now, day) = clock(utc_offset_minutes);
            let scope = scope_of(&scope);
            let counts = self.store().card_counts(&scope, now).map_err(store_error)?;
            let waiting = self
                .store()
                .cards_waiting(&scope, now, day)
                .map_err(store_error)?;
            Ok(ReviewCounts {
                total: counts.total as u32,
                due: counts.due as u32,
                new_cards: counts.new as u32,
                suspended: counts.suspended as u32,
                waiting: waiting as u32,
            })
        })
    }

    /// The cards to show now, at most `limit`.
    pub fn due(
        &self,
        scope: CardScopeArg,
        limit: u32,
        utc_offset_minutes: i32,
    ) -> Result<Vec<ReviewCard>, KansoError> {
        guard(|| {
            let (now, day) = clock(utc_offset_minutes);
            let due = self
                .store()
                .due_cards(&scope_of(&scope), now, day)
                .map_err(store_error)?;
            due.into_iter()
                .take(limit as usize)
                .map(|card| {
                    let spans = self
                        .store()
                        .card_intervals(&card.card_key, now)
                        .map_err(store_error)?;
                    Ok(ReviewCard {
                        intervals: spans.iter().map(|ms| interval_label(*ms)).collect(),
                        card_key: card.card_key,
                        note_id: card.note_id,
                        note_title: card.note_title,
                        line: card.line,
                        question: card.question,
                        answer: card.answer,
                        is_new: card.is_new,
                    })
                })
                .collect()
        })
    }

    /// Records an answer: 1 Again, 2 Hard, 3 Good, 4 Easy.
    pub fn answer(
        &self,
        note_id: String,
        card_key: String,
        grade: u8,
        utc_offset_minutes: i32,
    ) -> Result<Answered, KansoError> {
        guard(|| {
            let grade = Grade::from_number(i64::from(grade))
                .ok_or_else(|| KansoError::invalid("That isn't an answer."))?;
            let (now, day) = clock(utc_offset_minutes);
            let state = self
                .store()
                .record_review(&note_id, &card_key, grade, &self.device()?, now)
                .map_err(store_error)?;
            let waiting = self
                .store()
                .cards_waiting(&CardScope::All, now, day)
                .map_err(store_error)?;
            Ok(Answered {
                due_ms: state.due_ms,
                waiting: waiting as u32,
            })
        })
    }

    /// Hides a card until it is resumed.
    pub fn suspend(
        &self,
        note_id: String,
        card_key: String,
        suspended: bool,
    ) -> Result<(), KansoError> {
        guard(|| {
            self.store()
                .suspend_card(&note_id, &card_key, suspended, now_ms())
                .map_err(store_error)
        })
    }

    /// Forgets every answer on this account.
    pub fn reset_history(&self) -> Result<u32, KansoError> {
        guard(|| Ok(self.store().reset_review_history().map_err(store_error)? as u32))
    }

    pub fn stats(
        &self,
        scope: CardScopeArg,
        utc_offset_minutes: i32,
    ) -> Result<ReviewStats, KansoError> {
        guard(|| {
            let (now, day) = clock(utc_offset_minutes);
            let scope = scope_of(&scope);
            let stats = self
                .store()
                .card_stats(&scope, now, day)
                .map_err(store_error)?;
            let waiting = self
                .store()
                .cards_waiting(&scope, now, day)
                .map_err(store_error)?;
            Ok(ReviewStats {
                counts: ReviewCounts {
                    total: stats.counts.total as u32,
                    due: stats.counts.due as u32,
                    new_cards: stats.counts.new as u32,
                    suspended: stats.counts.suspended as u32,
                    waiting: waiting as u32,
                },
                per_day: stats.per_day,
                retention: stats.retention,
                forecast: stats.forecast,
                hardest: stats
                    .hardest
                    .into_iter()
                    .map(|h| HardCardView {
                        note_id: h.note_id,
                        note_title: h.note_title,
                        line: h.line,
                        question: h.question,
                        agains: h.agains,
                    })
                    .collect(),
            })
        })
    }

    /// The cards of one note with their schedule, for the editor's marks.
    pub fn note_cards(&self, note_id: String) -> Result<Vec<NoteCardView>, KansoError> {
        guard(|| {
            let cards = self.store().note_cards(&note_id).map_err(store_error)?;
            // Marks follow the lines as written: a repeated question is
            // marked "duplicate" beside the card that counts.
            let body = self
                .store()
                .note(&note_id)
                .map_err(store_error)?
                .map(|n| n.body)
                .unwrap_or_default();
            let mut seen = std::collections::HashSet::new();
            let mut out: Vec<NoteCardView> = cards
                .into_iter()
                .map(|c| {
                    seen.insert(c.card_key.clone());
                    NoteCardView {
                        card_key: c.card_key,
                        line: c.line,
                        question: c.question,
                        answer: c.answer,
                        due_ms: c.due_ms,
                        suspended: c.suspended,
                        duplicate: false,
                    }
                })
                .collect();
            for line in kanso_core::cards::card_lines(&body) {
                let key = kanso_core::cards::card_key(&note_id, &line.question);
                if out.iter().all(|c| c.line != line.line) && seen.contains(&key) {
                    out.push(NoteCardView {
                        card_key: key,
                        line: line.line,
                        question: line.question,
                        answer: line.answer,
                        due_ms: None,
                        suspended: false,
                        duplicate: true,
                    });
                }
            }
            out.sort_by_key(|c| c.line);
            Ok(out)
        })
    }

    /// Why **Make Flashcards** is not offered for a note, or `None`.
    pub fn make_blocked(&self, note_id: String) -> Result<Option<String>, KansoError> {
        guard(|| {
            if let Some(why) = flashcards::cannot_add(self.store(), &note_id) {
                return Ok(Some(why.to_owned()));
            }
            if kanso_engine::assist::note_for_ai(self.store(), &note_id).is_none() {
                return Ok(Some("This note is excluded from AI.".to_owned()));
            }
            Ok(None)
        })
    }

    /// Asks the model for cards from note `note_id`. Only that note is
    /// sent. The suggestions arrive on `sink` and change nothing until
    /// `add_cards`.
    pub fn suggest(
        &self,
        note_id: String,
        run: &AiRun,
        sink: Sink<SuggestEvent>,
    ) -> Result<(), KansoError> {
        guard(|| {
            let settings = self.ai.enabled()?;
            let provider = self.ai.provider_for(&settings)?;
            self.ai.inner.metrics().count(crate::metrics::Feature::Ai);
            let cancel = run.cancel_flag();
            let emitter = Arc::new(Emitter::new(sink, cancel.clone()));
            let store = self.ai.inner.store.clone();
            spawn("kanso-cards", move || {
                emitter.send(SuggestEvent::Status {
                    line: "Reading the note…".into(),
                });
                let result = {
                    let provider = held(&provider);
                    flashcards::suggest(&store, provider.as_ref(), &note_id, &cancel)
                };
                finish(&emitter, &cancel, result, |cards| SuggestEvent::Done {
                    cards: cards
                        .into_iter()
                        .map(|s| SuggestedCard {
                            question: s.question,
                            answer: s.answer,
                            source: s.source,
                            line: s.line,
                            source_missing: s.source_missing,
                        })
                        .collect(),
                });
            })
        })
    }

    /// Writes cards under `## Flashcards` in note `note_id`: one edit.
    /// Returns how many were added.
    pub fn add_cards(&self, note_id: String, cards: Vec<CardPair>) -> Result<u32, KansoError> {
        guard(|| {
            let pairs: Vec<(String, String)> =
                cards.into_iter().map(|c| (c.question, c.answer)).collect();
            flashcards::add_cards(self.store(), &note_id, &pairs)
                .map(|n| n as u32)
                .map_err(KansoError::invalid)
        })
    }

    /// Five questions from the notes in `note_ids` (those that may be
    /// sent), asked of the model.
    pub fn quiz(
        &self,
        note_ids: Vec<String>,
        run: &AiRun,
        sink: Sink<QuizEvent>,
    ) -> Result<(), KansoError> {
        guard(|| {
            let settings = self.ai.enabled()?;
            let provider = self.ai.provider_for(&settings)?;
            let excerpts = flashcards::quiz_excerpts(self.store(), &note_ids);
            if excerpts.is_empty() {
                return Err(KansoError::invalid(crate::ai::KEPT_OUT));
            }
            self.ai.inner.metrics().count(crate::metrics::Feature::Ai);
            let cancel = run.cancel_flag();
            let emitter = Arc::new(Emitter::new(sink, cancel.clone()));
            spawn("kanso-quiz", move || {
                emitter.send(QuizEvent::Status {
                    line: "Thinking of questions…".into(),
                });
                let result = {
                    let provider = held(&provider);
                    flashcards::quiz(provider.as_ref(), &excerpts, &cancel)
                };
                finish(&emitter, &cancel, result, |asked| QuizEvent::Questions {
                    items: asked
                        .into_iter()
                        .map(|q| {
                            let excerpt = &excerpts[q.excerpt - 1];
                            QuizItem {
                                question: q.question,
                                note_id: excerpt.id.clone(),
                                note_title: excerpt.title.clone(),
                                source: q.source,
                            }
                        })
                        .collect(),
                });
            })
        })
    }

    /// Grades `answer` against the note the question came from. Nothing is
    /// stored and the schedule does not change.
    pub fn grade(
        &self,
        question: String,
        note_id: String,
        answer: String,
        run: &AiRun,
        sink: Sink<GradeEvent>,
    ) -> Result<(), KansoError> {
        guard(|| {
            let settings = self.ai.enabled()?;
            let provider = self.ai.provider_for(&settings)?;
            let excerpt = flashcards::quiz_excerpts(self.store(), &[note_id])
                .into_iter()
                .next()
                .ok_or_else(|| KansoError::invalid(crate::ai::KEPT_OUT))?;
            let cancel = run.cancel_flag();
            let emitter = Arc::new(Emitter::new(sink, cancel.clone()));
            spawn("kanso-grade", move || {
                let result = {
                    let provider = held(&provider);
                    flashcards::grade(provider.as_ref(), &question, &excerpt, &answer, &cancel)
                };
                finish(&emitter, &cancel, result, |g| GradeEvent::Graded {
                    verdict: match g.verdict {
                        kanso_ai::draft_model::Verdict::Right => QuizVerdict::Right,
                        kanso_ai::draft_model::Verdict::Partly => QuizVerdict::Partly,
                        kanso_ai::draft_model::Verdict::Wrong => QuizVerdict::Wrong,
                    },
                    label: g.verdict.label().to_owned(),
                    comment: g.comment,
                });
            })
        })
    }

    /// The daily reminder's words for the cards waiting now; `None` when
    /// there are none. A count only, never a question.
    pub fn reminder_text(&self, utc_offset_minutes: i32) -> Result<Option<String>, KansoError> {
        guard(|| {
            let (now, day) = clock(utc_offset_minutes);
            let waiting = self
                .store()
                .cards_waiting(&CardScope::All, now, day)
                .map_err(store_error)?;
            Ok(flashcards::reminder_text(waiting))
        })
    }
}

/// The end of a request: the result as the event, or why not.
fn finish<R, T: Clone + 'static>(
    emitter: &Emitter<T>,
    cancel: &AtomicBool,
    result: Result<R, String>,
    done: impl FnOnce(R) -> T,
) where
    T: EndEvents,
{
    if cancel.load(std::sync::atomic::Ordering::Relaxed) {
        emitter.finish(T::stopped());
        return;
    }
    match result {
        Ok(value) => emitter.finish(done(value)),
        Err(message) => emitter.finish(T::failed(message)),
    }
}

/// The failure and stop events every flashcard stream has.
trait EndEvents {
    fn failed(message: String) -> Self;
    fn stopped() -> Self;
}

impl EndEvents for SuggestEvent {
    fn failed(message: String) -> Self {
        SuggestEvent::Failed { message }
    }
    fn stopped() -> Self {
        SuggestEvent::Stopped
    }
}

impl EndEvents for QuizEvent {
    fn failed(message: String) -> Self {
        QuizEvent::Failed { message }
    }
    fn stopped() -> Self {
        QuizEvent::Stopped
    }
}

impl EndEvents for GradeEvent {
    fn failed(message: String) -> Self {
        GradeEvent::Failed { message }
    }
    fn stopped() -> Self {
        GradeEvent::Stopped
    }
}

/// Now, and the start of the person's day.
fn clock(utc_offset_minutes: i32) -> (i64, i64) {
    let now = now_ms();
    (now, flashcards::day_start_ms(now, utc_offset_minutes))
}

fn valid_time(text: &str) -> bool {
    let Some((h, m)) = text.split_once(':') else {
        return false;
    };
    m.len() == 2
        && matches!((h.parse::<u32>(), m.parse::<u32>()), (Ok(h), Ok(m)) if h < 24 && m < 60)
}
