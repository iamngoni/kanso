//! People & Projects on the phone (issue #200): the calls behind the list
//! of pages, the suggestions, the live sections below a person's or
//! project's note, and the settings. The pages, their sections, detection
//! and the AI writing are the engine's (`kanso_engine::entities`,
//! `kanso_engine::pages`), which the desktop uses too; this is the shell's
//! side: the settings file, the model call and the plain data.
//!
//! Nothing here makes or merges a page unless the person asked. A locked
//! note, a note in a locked notebook, a trashed note and, for AI, a note
//! tagged `#ai-exclude` are never a source.

use std::path::PathBuf;

use kanso_ai::Provider;
use kanso_core::entities::EntityKind;
use kanso_core::now_ms;
use kanso_engine::ask::Receipt;
use kanso_engine::entities::{
    self, EntityPage, EntitySettings, Suggestion, actions, ai_detect, suggest,
};
use kanso_engine::pages::{
    self, OpenOptions, PageSource as _, PageView, RefreshPolicy, Section, SectionKind,
    SectionState, write::WriteContext,
};
use kanso_store::Store;
use serde::{Deserialize, Serialize};

use crate::ai::AiHandle;
use crate::boundary::{guard, held};
use crate::errors::KansoError;
use crate::library::Library;
use crate::people_model::{
    PageHeaderInfo, PageHidden, PageItem, PagePanel, PageSection, PageSectionState, PageSource,
    PageTask, PeopleKind, PeopleMerged, PeoplePage, PeopleSettings, PeopleStatus, PeopleSuggestion,
};

/// The file the settings are in, next to the library's database.
const SETTINGS_FILE: &str = "people.json";

/// What `people.json` holds. Every field has a default, so a file from an
/// older or newer build still reads.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct SettingsFile {
    #[serde(flatten)]
    entities: EntitySettings,
    /// When detection last ran (ms since the epoch).
    last_detect: i64,
}

/// The People & Projects calls of one library.
#[derive(Clone)]
pub struct People {
    ai: AiHandle,
}

impl Library {
    /// People and projects: pages, suggestions, live sections, settings.
    pub fn people(&self) -> People {
        People { ai: self.ai() }
    }

    /// [`Library::people`] with the AI handle `ai` (a test's, which
    /// answers without a network).
    #[cfg(any(test, feature = "test-support"))]
    pub fn people_with_ai(&self, ai: AiHandle) -> People {
        People { ai }
    }
}

fn kind_of(kind: EntityKind) -> PeopleKind {
    match kind {
        EntityKind::Person => PeopleKind::Person,
        EntityKind::Project => PeopleKind::Project,
    }
}

fn engine_kind(kind: PeopleKind) -> EntityKind {
    match kind {
        PeopleKind::Person => EntityKind::Person,
        PeopleKind::Project => EntityKind::Project,
    }
}

fn source_of(receipt: &Receipt) -> PageSource {
    PageSource {
        note_id: receipt.note_id.clone(),
        title: receipt.title.clone(),
        updated: receipt.updated.clone().unwrap_or_default(),
        start: receipt.start,
        end: receipt.end,
        quote: receipt.quote.clone(),
        label: receipt.source_line(),
    }
}

fn section_of(section: &Section) -> PageSection {
    let (state, changed) = match section.state {
        SectionState::Fresh => (PageSectionState::Fresh, 0),
        SectionState::Stale { changed } => (PageSectionState::Stale, changed),
        SectionState::Pinned { changed, .. } => (PageSectionState::Pinned, changed),
        SectionState::Unwritten => (PageSectionState::Unwritten, 0),
        SectionState::Off(_) => (PageSectionState::Off, 0),
    };
    PageSection {
        key: section.key.clone(),
        title: section.title.clone(),
        ai: section.kind == SectionKind::Ai,
        items: section
            .items
            .iter()
            .map(|item| PageItem {
                text: item.text.clone(),
                who: item.who.clone().unwrap_or_default(),
                sources: item.sources.iter().map(source_of).collect(),
                meta: item.meta.clone().unwrap_or_default(),
                group: item.group.clone().unwrap_or_default(),
                task: item.task.as_ref().map(|t| PageTask {
                    note_id: t.note_id.clone(),
                    line: t.line,
                    text: t.text.clone(),
                    checked: t.checked,
                }),
                target: item.target.clone().unwrap_or_default(),
                tag: item.tag.clone().unwrap_or_default(),
                shared: item.shared,
            })
            .collect(),
        state,
        changed: changed as u32,
        written_ms: section.written_at.unwrap_or(0),
        total: section.total as u32,
        note: section.note.clone().unwrap_or_default(),
        empty_line: pages::empty_line(section).unwrap_or_default().to_owned(),
        stale_line: section.stale_line().unwrap_or_default(),
    }
}

fn suggestion_of(s: &Suggestion) -> PeopleSuggestion {
    PeopleSuggestion {
        key: s.key.clone(),
        name: s.name.clone(),
        kind: kind_of(s.kind),
        detail: s.detail(),
        also_line: s.also_line().unwrap_or_default(),
        alias_of_id: s.alias_of.as_ref().map(|a| a.0.clone()).unwrap_or_default(),
        alias_of_title: s.alias_of.as_ref().map(|a| a.1.clone()).unwrap_or_default(),
    }
}

impl People {
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

    fn save(&self, file: &SettingsFile) -> Result<(), KansoError> {
        let path = self.settings_path();
        let bytes =
            serde_json::to_vec_pretty(file).map_err(|err| KansoError::storage(err.to_string()))?;
        let part = path.with_extension("json.part");
        std::fs::write(&part, bytes)
            .and_then(|()| std::fs::rename(&part, &path))
            .map_err(|err| KansoError::storage(err.to_string()))
    }

    /// The provider and address AI is set to, whether or not it is on.
    fn provider_info(&self) -> Result<(Provider, String, bool), KansoError> {
        let view = self.ai.settings()?;
        let provider = Provider::ALL
            .into_iter()
            .find(|p| p.key() == view.provider.trim())
            .unwrap_or_default();
        Ok((provider, view.base_url, view.enabled))
    }

    fn ai_on(&self, settings: &EntitySettings) -> bool {
        self.provider_info()
            .map(|(_, _, enabled)| settings.ai_sections(enabled))
            .unwrap_or(false)
    }

    // MARK: Settings

    /// The settings now.
    pub fn settings(&self) -> PeopleSettings {
        let file = self.load().entities;
        PeopleSettings {
            suggest: file.suggest,
            keep_in: file.keep_in.unwrap_or_default(),
            write_with_ai: file.write_with_ai,
            refresh: file.refresh.unwrap_or_else(|| "auto".to_owned()),
            detect_with_ai: file.detect_with_ai,
        }
    }

    /// Saves the settings and returns them as they are now.
    pub fn set_settings(&self, input: PeopleSettings) -> Result<PeopleSettings, KansoError> {
        guard(|| {
            let mut file = self.load();
            file.entities.suggest = input.suggest;
            file.entities.keep_in = Some(input.keep_in.trim().to_owned()).filter(|s| !s.is_empty());
            file.entities.write_with_ai = input.write_with_ai;
            file.entities.refresh = match input.refresh.as_str() {
                "open" | "press" => Some(input.refresh),
                _ => None,
            };
            file.entities.detect_with_ai = input.detect_with_ai;
            self.save(&file)?;
            Ok(self.settings())
        })
    }

    // MARK: Pages and suggestions

    /// How many pages and suggestions there are, for the drawer row.
    pub fn status(&self) -> Result<PeopleStatus, KansoError> {
        guard(|| {
            let file = self.load().entities;
            let pages = self.store().entities()?.len() as u32;
            let suggestions = suggest::suggestions(self.store(), &file)?.len();
            Ok(PeopleStatus {
                pages,
                suggestions: suggestions as u32,
                new_label: suggest::count_label(suggestions).unwrap_or_default(),
                suggest_on: file.suggest,
                ai_on: self.ai_on(&file),
            })
        })
    }

    /// Every page for a person or project, most recently changed first.
    pub fn pages(&self) -> Result<Vec<PeoplePage>, KansoError> {
        guard(|| {
            let mut out = Vec::new();
            for page in self.store().entities()? {
                let (notes, last) = self.store().entity_mention_stats(&page.id)?;
                let mut detail = format!("{notes} {}", if notes == 1 { "note" } else { "notes" });
                if let Some(day) = (notes > 0)
                    .then(|| entities::page::date_label(last))
                    .flatten()
                {
                    detail.push_str(&format!(" \u{b7} last {day}"));
                }
                out.push(PeoplePage {
                    id: page.id,
                    title: page.title,
                    kind: kind_of(page.kind),
                    notes,
                    detail,
                });
            }
            Ok(out)
        })
    }

    /// Runs detection when it is due (or `force`d) and returns how many
    /// suggestions there are. Does nothing while suggestions are off.
    pub fn detect(&self, force: bool) -> Result<u32, KansoError> {
        guard(|| {
            let mut file = self.load();
            let mut last = file.last_detect;
            let ran = suggest::refresh(self.store(), &file.entities, now_ms(), &mut last, force)?;
            if ran.is_some() {
                file.last_detect = last;
                self.save(&file)?;
            }
            Ok(suggest::suggestions(self.store(), &file.entities)?.len() as u32)
        })
    }

    /// The suggestions to offer now.
    pub fn suggestions(&self) -> Result<Vec<PeopleSuggestion>, KansoError> {
        guard(|| {
            let file = self.load().entities;
            Ok(suggest::suggestions(self.store(), &file)?
                .iter()
                .map(suggestion_of)
                .collect())
        })
    }

    /// Asks the person's model which suggestions are people or projects.
    /// Refused unless AI and its switch here are on; once a day at most.
    pub fn detect_with_ai(&self) -> Result<u32, KansoError> {
        guard(|| {
            let mut file = self.load();
            if !file.entities.detect_with_ai {
                return Err(KansoError::invalid(
                    "Use AI to find people and projects is turned off.",
                ));
            }
            let settings = self.ai.enabled()?;
            if now_ms() - file.entities.last_ai_detect < ai_detect::MIN_GAP_MS
                && file.entities.last_ai_detect != 0
            {
                return Ok(suggest::suggestions(self.store(), &file.entities)?.len() as u32);
            }
            let provider = self.ai.provider_for(&settings)?;
            let left = {
                let provider = held(&provider);
                ai_detect::run(self.store(), provider.as_ref())
            }
            .map_err(|err| KansoError::invalid(err.to_string()))?;
            file.entities.last_ai_detect = now_ms();
            self.save(&file)?;
            Ok(left as u32)
        })
    }

    /// Makes the page for a name (and the notebook it goes in, on first
    /// use) and returns its note ID.
    pub fn create(
        &self,
        kind: PeopleKind,
        name: String,
        aliases: Vec<String>,
    ) -> Result<String, KansoError> {
        guard(|| {
            let file = self.load().entities;
            actions::create_page(
                self.store(),
                &file,
                engine_kind(kind),
                &name,
                &aliases,
                now_ms(),
            )
            .map_err(|err| KansoError::invalid(err.to_string()))
        })
    }

    /// "Not a person" / "Not a project" (`never`), or "Not now".
    pub fn dismiss(&self, key: String, never: bool) -> Result<(), KansoError> {
        guard(|| {
            let file = self.load().entities;
            let Some(found) = suggest::suggestions(self.store(), &file)?
                .into_iter()
                .find(|s| s.key == key)
            else {
                return Ok(());
            };
            Ok(suggest::dismiss(
                self.store(),
                &file,
                &found,
                never,
                now_ms(),
            )?)
        })
    }

    /// The names turned away for good, for Settings.
    pub fn dismissed(&self) -> Result<Vec<String>, KansoError> {
        guard(|| {
            Ok(self
                .store()
                .dismissed()?
                .into_iter()
                .filter(|d| d.kind == "never")
                .map(|d| d.name)
                .collect())
        })
    }

    /// "Suggest again".
    pub fn suggest_again(&self, name: String) -> Result<(), KansoError> {
        guard(|| {
            let file = self.load().entities;
            Ok(suggest::suggest_again(
                self.store(),
                &file,
                &name,
                now_ms(),
            )?)
        })
    }

    /// "Yes, add alias": the name becomes another name of the page.
    pub fn confirm_alias(&self, key: String, page_id: String) -> Result<(), KansoError> {
        guard(|| {
            let file = self.load().entities;
            let Some(found) = suggest::suggestions(self.store(), &file)?
                .into_iter()
                .find(|s| s.key == key)
            else {
                return Ok(());
            };
            actions::add_alias(self.store(), &page_id, &found.name, now_ms())
                .map_err(|err| KansoError::invalid(err.to_string()))?;
            Ok(())
        })
    }

    /// Adds `alias` to a page. Returns whether anything changed.
    pub fn add_alias(&self, page_id: String, alias: String) -> Result<bool, KansoError> {
        guard(|| {
            actions::add_alias(self.store(), &page_id, &alias, now_ms())
                .map_err(|err| KansoError::invalid(err.to_string()))
        })
    }

    /// Merges page `from` into page `into`.
    pub fn merge(&self, into: String, from: String) -> Result<PeopleMerged, KansoError> {
        guard(|| {
            let done = actions::merge(self.store(), &into, &from, now_ms())
                .map_err(|err| KansoError::invalid(err.to_string()))?;
            Ok(PeopleMerged {
                into: done.into,
                links_retitled: done.links_retitled as u32,
            })
        })
    }

    // MARK: An open page

    fn options(&self, file: &EntitySettings) -> OpenOptions {
        OpenOptions {
            ai_on: self.ai_on(file),
            now: now_ms(),
        }
    }

    fn policy(&self, file: &EntitySettings) -> RefreshPolicy {
        match self.provider_info() {
            Ok((provider, base_url, _)) => file.policy(provider, &base_url),
            Err(_) => RefreshPolicy::OnPress,
        }
    }

    fn panel(&self, page: &EntityPage, show_all: &[&str]) -> Result<PagePanel, KansoError> {
        let file = self.load().entities;
        let options = self.options(&file);
        let view: PageView = pages::open(self.store(), page, options, show_all)?;
        let header = page.header(self.store())?;
        let mut line = String::new();
        if !header.aliases.is_empty() {
            line.push_str(&format!(
                "Also known as {} \u{b7} ",
                header.aliases.join(", ")
            ));
        }
        line.push_str(&format!(
            "{} {}",
            header.notes,
            if header.notes == 1 { "note" } else { "notes" }
        ));
        if let Some(last) = &header.last_mentioned {
            line.push_str(&format!(" \u{b7} last mentioned {last}"));
        }
        Ok(PagePanel {
            header: PageHeaderInfo {
                title: header.title,
                kind: kind_of(header.kind),
                aliases: header.aliases,
                notes: header.notes,
                last_mentioned: header.last_mentioned.unwrap_or_default(),
                line,
            },
            sections: view.sections.iter().map(section_of).collect(),
            hidden: view
                .hidden
                .iter()
                .map(|(key, title)| PageHidden {
                    key: key.clone(),
                    title: title.clone(),
                })
                .collect(),
            ai_on: options.ai_on,
            due: pages::due_for_refresh(&view, self.policy(&file), options),
        })
    }

    /// The live sections of note `id`, or `None` when it is not a person or
    /// project page (or is locked: its own text is all it shows).
    pub fn page(&self, id: &str) -> Result<Option<PagePanel>, KansoError> {
        guard(|| {
            let Some(page) = EntityPage::load(self.store(), id)? else {
                return Ok(None);
            };
            Ok(Some(self.panel(&page, &[])?))
        })
    }

    /// A computed section with all its lines ("Show all 14").
    pub fn show_all(&self, id: &str, key: &str) -> Result<Option<PageSection>, KansoError> {
        guard(|| {
            let Some(page) = EntityPage::load(self.store(), id)? else {
                return Ok(None);
            };
            Ok(Some(section_of(&page.compute(self.store(), key, true)?)))
        })
    }

    /// Writes one AI section again and returns the page as it is now.
    /// Refused while AI is off.
    pub fn refresh_section(&self, id: &str, key: &str) -> Result<Option<PagePanel>, KansoError> {
        guard(|| {
            let Some(page) = EntityPage::load(self.store(), id)? else {
                return Ok(None);
            };
            let settings = self.ai.enabled()?;
            let provider = self.ai.provider_for(&settings)?;
            {
                let provider = held(&provider);
                let context = WriteContext::new(provider.as_ref(), &settings.model, now_ms());
                pages::refresh_section(self.store(), &page, key, &context)
                    .map_err(|err| KansoError::invalid(err.to_string()))?;
            }
            Ok(Some(self.panel(&page, &[])?))
        })
    }

    /// Writes every AI section that is not kept again.
    pub fn refresh_all(&self, id: &str) -> Result<Option<PagePanel>, KansoError> {
        guard(|| {
            let Some(page) = EntityPage::load(self.store(), id)? else {
                return Ok(None);
            };
            let settings = self.ai.enabled()?;
            let provider = self.ai.provider_for(&settings)?;
            {
                let provider = held(&provider);
                let context = WriteContext::new(provider.as_ref(), &settings.model, now_ms());
                pages::refresh_all(self.store(), &page, &context)
                    .map_err(|err| KansoError::invalid(err.to_string()))?;
            }
            Ok(Some(self.panel(&page, &[])?))
        })
    }

    /// Keeps a section's current text (`on`), or stops keeping it.
    pub fn pin(&self, id: &str, key: &str, on: bool) -> Result<Option<PagePanel>, KansoError> {
        guard(|| {
            let Some(page) = EntityPage::load(self.store(), id)? else {
                return Ok(None);
            };
            pages::pin(self.store(), &page, key, on, now_ms())?;
            Ok(Some(self.panel(&page, &[])?))
        })
    }

    /// Hides a section on this page, or shows it again.
    pub fn hide(&self, id: &str, key: &str, hidden: bool) -> Result<Option<PagePanel>, KansoError> {
        guard(|| {
            let Some(page) = EntityPage::load(self.store(), id)? else {
                return Ok(None);
            };
            pages::hide(self.store(), &page, key, hidden, now_ms())?;
            Ok(Some(self.panel(&page, &[])?))
        })
    }

    /// Shows every hidden section again.
    pub fn show_hidden(&self, id: &str) -> Result<Option<PagePanel>, KansoError> {
        guard(|| {
            let Some(page) = EntityPage::load(self.store(), id)? else {
                return Ok(None);
            };
            pages::show_hidden(self.store(), &page, now_ms())?;
            Ok(Some(self.panel(&page, &[])?))
        })
    }

    /// "Not right" on line `index` of an AI section: it goes, and the next
    /// rewrite is told.
    pub fn not_right(
        &self,
        id: &str,
        key: &str,
        index: u32,
    ) -> Result<Option<PagePanel>, KansoError> {
        guard(|| {
            let Some(page) = EntityPage::load(self.store(), id)? else {
                return Ok(None);
            };
            pages::not_right(self.store(), &page, key, index as usize)?;
            Ok(Some(self.panel(&page, &[])?))
        })
    }

    /// Copies a section's lines, with their sources as links, onto the end
    /// of the person's own text. The section stays live.
    pub fn copy_into_note(&self, id: &str, key: &str) -> Result<bool, KansoError> {
        guard(|| {
            let Some(page) = EntityPage::load(self.store(), id)? else {
                return Ok(false);
            };
            let file = self.load().entities;
            let view = pages::open(self.store(), &page, self.options(&file), &[])?;
            let Some(section) = view.sections.iter().find(|s| s.key == key) else {
                return Ok(false);
            };
            actions::copy_into_note(
                self.store(),
                id,
                &section.title,
                &pages::section_markdown(section),
                now_ms(),
            )
            .map_err(|err| KansoError::invalid(err.to_string()))
        })
    }

    /// The section as Markdown, for Copy.
    pub fn section_markdown(&self, id: &str, key: &str) -> Result<String, KansoError> {
        guard(|| {
            let Some(page) = EntityPage::load(self.store(), id)? else {
                return Ok(String::new());
            };
            let file = self.load().entities;
            let view = pages::open(self.store(), &page, self.options(&file), &[])?;
            Ok(view
                .sections
                .iter()
                .find(|s| s.key == key)
                .map(pages::section_markdown)
                .unwrap_or_default())
        })
    }
}
