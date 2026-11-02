//! Inbox triage on the phone (issue #195): the calls behind the Triage
//! pager. The suggestions, filing and its undo are the engine's
//! (`kanso_engine::triage`), which the desktop uses too; this is the
//! shell's side: the settings, the model call and the plain data the pager
//! shows.
//!
//! Nothing here changes a note until `file` is called with the chips a
//! person accepted. A rejection is remembered in the library's local,
//! unsynced table. A filed batch is kept under an Undo ticket (the same
//! `Library::undo` the swipe actions use), so the snackbar can take it back.
//! A locked note, a note in a locked notebook, a trashed note and a note
//! tagged `#ai-exclude` are never shown, never used as evidence and never
//! sent to a model.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use kanso_core::now_ms;
use kanso_core::roles::Role;
use kanso_engine::triage::{self, Card, Field, Filing, Source, Suggestion, TriageOptions};
use kanso_store::Store;
use serde::{Deserialize, Serialize};

use crate::ai::AiHandle;
use crate::boundary::guard;
use crate::errors::KansoError;
use crate::library::Library;
use crate::triage_model::{
    TriageCard, TriageChip, TriageField, TriageFiled, TriageFiling, TriageMissing, TriageNote,
    TriageSettings, TriageStatus,
};
use crate::undo::Undoable;

/// The file the settings are in, next to the library's database.
const SETTINGS_FILE: &str = "triage.json";
/// The most tags a card may suggest.
pub const MAX_TAGS_LIMIT: u32 = 5;
/// The most cards read at once.
const CARDS: usize = 200;

/// What `triage.json` holds. Every field has a default, so a file from an
/// older or newer build still reads.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct SettingsFile {
    show_bar: bool,
    suggest_titles: bool,
    suggest_links: bool,
    max_tags: u32,
    use_ai: bool,
}

impl Default for SettingsFile {
    fn default() -> Self {
        SettingsFile {
            show_bar: true,
            suggest_titles: true,
            suggest_links: true,
            max_tags: 3,
            use_ai: false,
        }
    }
}

/// The Inbox triage calls of one library.
#[derive(Clone)]
pub struct Triage {
    ai: AiHandle,
}

impl Library {
    /// Inbox triage: cards of suggestions for the Inbox's notes, filing
    /// what is accepted, and the settings.
    pub fn triage(&self) -> Triage {
        Triage { ai: self.ai() }
    }

    /// [`Library::triage`] with the AI handle `ai` (a test's, which answers
    /// without a network).
    #[cfg(any(test, feature = "test-support"))]
    pub fn triage_with_ai(&self, ai: AiHandle) -> Triage {
        Triage { ai }
    }
}

impl Triage {
    fn store(&self) -> &Store {
        &self.ai.inner.store
    }

    fn settings_path(&self) -> PathBuf {
        self.ai
            .inner
            .path
            .parent()
            .filter(|dir| !dir.as_os_str().is_empty())
            .unwrap_or_else(|| std::path::Path::new("."))
            .join(SETTINGS_FILE)
    }

    fn load(&self) -> SettingsFile {
        std::fs::read(self.settings_path())
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    fn options(&self) -> TriageOptions {
        let file = self.load();
        TriageOptions {
            inbox_id: Role::Inbox.book_id().to_owned(),
            suggest_titles: file.suggest_titles,
            suggest_links: file.suggest_links,
            max_tags: file.max_tags.clamp(1, MAX_TAGS_LIMIT) as usize,
        }
    }

    // MARK: Settings

    /// The settings now.
    pub fn settings(&self) -> TriageSettings {
        let file = self.load();
        TriageSettings {
            show_bar: file.show_bar,
            suggest_titles: file.suggest_titles,
            suggest_links: file.suggest_links,
            max_tags: file.max_tags.clamp(1, MAX_TAGS_LIMIT),
            use_ai: file.use_ai,
        }
    }

    /// Saves the settings, by writing beside them and renaming, and returns
    /// them as they are now.
    pub fn set_settings(&self, input: TriageSettings) -> Result<TriageSettings, KansoError> {
        guard(|| {
            let file = SettingsFile {
                show_bar: input.show_bar,
                suggest_titles: input.suggest_titles,
                suggest_links: input.suggest_links,
                max_tags: input.max_tags.clamp(1, MAX_TAGS_LIMIT),
                use_ai: input.use_ai,
            };
            let path = self.settings_path();
            let bytes = serde_json::to_vec_pretty(&file)
                .map_err(|err| KansoError::storage(err.to_string()))?;
            let part = path.with_extension("json.part");
            std::fs::write(&part, bytes)
                .and_then(|()| std::fs::rename(&part, &path))
                .map_err(|err| KansoError::storage(err.to_string()))?;
            Ok(self.settings())
        })
    }

    // MARK: Cards

    /// How many notes wait, and what is switched on.
    pub fn status(&self) -> Result<TriageStatus, KansoError> {
        guard(|| {
            let waiting = self.store().inbox_waiting(Role::Inbox.book_id())?.len() as u32;
            let ai = self.ai.settings()?;
            Ok(TriageStatus {
                waiting,
                related_on: self.store().embedding_model()?.is_some(),
                ai_on: ai.enabled && self.load().use_ai,
            })
        })
    }

    /// Cards for the notes Triage shows, newest first.
    pub fn cards(&self, limit: u32) -> Result<Vec<TriageCard>, KansoError> {
        guard(|| {
            let limit = (limit as usize).clamp(1, CARDS);
            let cards = triage::cards(self.store(), &self.options(), limit)?;
            cards
                .into_iter()
                .map(|card| self.card_of(card))
                .collect::<Result<_, _>>()
        })
    }

    /// The card for one note, made again (after it changed elsewhere).
    pub fn card(&self, note_id: &str) -> Result<Option<TriageCard>, KansoError> {
        guard(|| {
            triage::card(self.store(), note_id, &self.options())?
                .map(|card| self.card_of(card))
                .transpose()
        })
    }

    fn card_of(&self, card: Card) -> Result<TriageCard, KansoError> {
        let note = self.store().note(&card.note_id)?;
        let (title, snippet, updated_ms) = note.map_or_else(Default::default, |n| {
            (
                n.title.clone(),
                kanso_core::links::plain_line(n.body.trim_start())
                    .chars()
                    .take(280)
                    .collect::<String>(),
                n.updated_at,
            )
        });
        let chips = card
            .suggestions
            .iter()
            .map(|s| self.chip_of(s))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(TriageCard {
            note_id: card.note_id,
            rev: card.rev,
            title,
            snippet,
            updated_ms,
            chips,
            missing: match card.missing {
                None => TriageMissing::Nothing,
                Some(triage::Reason::RelatedOff) => TriageMissing::RelatedOff,
                Some(triage::Reason::NotEmbedded) => TriageMissing::NotRead,
            },
        })
    }

    fn chip_of(&self, s: &Suggestion) -> Result<TriageChip, KansoError> {
        let because = s
            .because
            .iter()
            .filter_map(|id| match self.store().note(id) {
                Ok(Some(n)) => Some(Ok(TriageNote {
                    id: id.clone(),
                    title: n.title,
                })),
                Ok(None) => None,
                Err(err) => Some(Err(KansoError::from(err))),
            })
            .collect::<Result<_, _>>()?;
        Ok(TriageChip {
            field: field_of(s.field),
            value: s.value.clone(),
            label: s.label.clone(),
            score: s.score,
            because,
            ai: s.source == Source::Ai,
            is_new: s.is_new,
        })
    }

    // MARK: Model

    /// The card for `note_id` refined by the person's AI: it ranks the
    /// library's candidates and may suggest a title, and every chip cites
    /// notes. Refused while AI or its Inbox switch is off, and for a note
    /// kept out of AI (nothing is sent either way). The error is a plain
    /// reason; the shell keeps the card's library chips and says "AI
    /// suggestions unavailable: <reason>".
    pub fn refine(&self, note_id: &str) -> Result<Vec<TriageChip>, KansoError> {
        guard(|| {
            if !self.load().use_ai {
                return Err(KansoError::invalid(
                    "Use AI for Inbox suggestions is turned off.",
                ));
            }
            let opts = self.options();
            let card = triage::card(self.store(), note_id, &opts)?
                .ok_or_else(|| KansoError::invalid("That note isn't in the Inbox."))?;
            let note = self
                .store()
                .note(note_id)?
                .ok_or_else(|| KansoError::invalid("That note isn't in the Inbox."))?;
            let provider = self.ai.triage_provider()?;
            let cancel = Arc::new(AtomicBool::new(false));
            let suggestions = {
                let provider = crate::boundary::held(&provider);
                triage::ai::refine(
                    self.store(),
                    provider.as_ref(),
                    &card,
                    &note.title,
                    &note.body,
                    &opts,
                    &cancel,
                )
            }
            .map_err(KansoError::invalid)?;
            suggestions.iter().map(|s| self.chip_of(s)).collect()
        })
    }

    // MARK: Rejecting and editing

    /// Remembers that the person rejected a chip, so it doesn't come back.
    pub fn reject(&self, note_id: &str, field: TriageField, value: &str) -> Result<(), KansoError> {
        guard(|| {
            Ok(self.store().dismiss_triage(
                note_id,
                engine_field(field).as_str(),
                value,
                now_ms(),
            )?)
        })
    }

    /// Takes a rejection back.
    pub fn unreject(
        &self,
        note_id: &str,
        field: TriageField,
        value: &str,
    ) -> Result<(), KansoError> {
        guard(|| {
            Ok(self
                .store()
                .restore_triage(note_id, engine_field(field).as_str(), value)?)
        })
    }

    /// The chip a typed value makes for `field` (a notebook, tag, title or
    /// note), or why it can't.
    pub fn resolve(&self, field: TriageField, text: &str) -> Result<TriageChip, KansoError> {
        guard(|| {
            let s = triage::resolve(self.store(), engine_field(field), text)
                .map_err(KansoError::invalid)?;
            self.chip_of(&s)
        })
    }

    // MARK: Filing

    /// What filing `filings` would do, without writing anything: for "File
    /// 5 notes? Moves 4 to notebooks, adds 9 tags…".
    pub fn plan(&self, filings: Vec<TriageFiling>) -> Result<TriageFiled, KansoError> {
        guard(|| {
            let filed = triage::file::plan(self.store(), &self.filings(filings), now_ms())?;
            Ok(filed_of(&filed, 0))
        })
    }

    /// Files `filings` in one write: every note changes, or none does. A
    /// note changed since its card was read is left alone (`stale`). The
    /// returned `undo` ticket takes the whole batch back with
    /// [`Library::undo`].
    pub fn file(&self, filings: Vec<TriageFiling>) -> Result<TriageFiled, KansoError> {
        guard(|| {
            let filed = triage::file::file(self.store(), &self.filings(filings), now_ms())?;
            let ticket = if filed.edits.is_empty() {
                0
            } else {
                Library {
                    inner: self.ai.inner.clone(),
                }
                .keep_undo(Undoable::Fields(filed.edits.clone()))
            };
            Ok(filed_of(&filed, ticket))
        })
    }

    fn filings(&self, filings: Vec<TriageFiling>) -> Vec<Filing> {
        filings
            .into_iter()
            .map(|f| Filing {
                note_id: f.note_id,
                rev: f.rev,
                accepted: f.accepted.into_iter().map(suggestion_of).collect(),
            })
            .collect()
    }
}

fn field_of(field: Field) -> TriageField {
    match field {
        Field::Notebook => TriageField::Notebook,
        Field::Tag => TriageField::Tag,
        Field::Title => TriageField::Title,
        Field::Link => TriageField::Link,
    }
}

fn engine_field(field: TriageField) -> Field {
    match field {
        TriageField::Notebook => Field::Notebook,
        TriageField::Tag => Field::Tag,
        TriageField::Title => Field::Title,
        TriageField::Link => Field::Link,
    }
}

fn suggestion_of(chip: TriageChip) -> Suggestion {
    Suggestion {
        field: engine_field(chip.field),
        value: chip.value,
        label: chip.label,
        score: chip.score.clamp(0., 1.),
        because: chip.because.into_iter().map(|n| n.id).collect(),
        source: if chip.ai { Source::Ai } else { Source::Library },
        is_new: chip.is_new,
    }
}

fn filed_of(filed: &triage::Filed, undo: u64) -> TriageFiled {
    TriageFiled {
        undo,
        filed: filed.edits.len() as u32,
        stale: filed.stale.clone(),
        moved: filed.moved as u32,
        tags: filed.tags as u32,
        titles: filed.titles as u32,
        links: filed.links as u32,
        sentence: filed.sentence(),
    }
}
