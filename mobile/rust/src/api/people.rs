//! People & Projects: pages, suggestions, the live sections below a
//! person's or project's note, and the settings.

use flutter_rust_bridge::frb;
use kanso_mobile::{KansoError, People};

use super::people_types::{
    PagePanel, PageSection, PeopleKind, PeopleMerged, PeoplePage, PeopleSettings, PeopleStatus,
    PeopleSuggestion,
};

#[frb(opaque)]
pub struct KansoPeople(People);

impl KansoPeople {
    pub(super) fn new(people: People) -> KansoPeople {
        KansoPeople(people)
    }

    pub fn settings(&self) -> PeopleSettings {
        self.0.settings()
    }

    pub fn set_settings(&self, input: PeopleSettings) -> Result<PeopleSettings, KansoError> {
        self.0.set_settings(input)
    }

    /// How many pages and suggestions there are.
    pub fn status(&self) -> Result<PeopleStatus, KansoError> {
        self.0.status()
    }

    /// Every page for a person or project.
    pub fn pages(&self) -> Result<Vec<PeoplePage>, KansoError> {
        self.0.pages()
    }

    /// Looks for names worth a page when it is due (or `force`d).
    pub fn detect(&self, force: bool) -> Result<u32, KansoError> {
        self.0.detect(force)
    }

    pub fn suggestions(&self) -> Result<Vec<PeopleSuggestion>, KansoError> {
        self.0.suggestions()
    }

    /// Asks the person's model to label the suggestions.
    pub fn detect_with_ai(&self) -> Result<u32, KansoError> {
        self.0.detect_with_ai()
    }

    /// Makes a page and returns its note ID.
    pub fn create(
        &self,
        kind: PeopleKind,
        name: String,
        aliases: Vec<String>,
    ) -> Result<String, KansoError> {
        self.0.create(kind, name, aliases)
    }

    pub fn dismiss(&self, key: String, never: bool) -> Result<(), KansoError> {
        self.0.dismiss(key, never)
    }

    pub fn dismissed(&self) -> Result<Vec<String>, KansoError> {
        self.0.dismissed()
    }

    pub fn suggest_again(&self, name: String) -> Result<(), KansoError> {
        self.0.suggest_again(name)
    }

    pub fn confirm_alias(&self, key: String, page_id: String) -> Result<(), KansoError> {
        self.0.confirm_alias(key, page_id)
    }

    pub fn add_alias(&self, page_id: String, alias: String) -> Result<bool, KansoError> {
        self.0.add_alias(page_id, alias)
    }

    pub fn merge(&self, into: String, from: String) -> Result<PeopleMerged, KansoError> {
        self.0.merge(into, from)
    }

    /// The live sections of a note, or nothing when it isn't a page.
    pub fn page(&self, id: String) -> Result<Option<PagePanel>, KansoError> {
        self.0.page(&id)
    }

    pub fn show_all(&self, id: String, key: String) -> Result<Option<PageSection>, KansoError> {
        self.0.show_all(&id, &key)
    }

    pub fn refresh_section(
        &self,
        id: String,
        key: String,
    ) -> Result<Option<PagePanel>, KansoError> {
        self.0.refresh_section(&id, &key)
    }

    pub fn refresh_all(&self, id: String) -> Result<Option<PagePanel>, KansoError> {
        self.0.refresh_all(&id)
    }

    pub fn pin(&self, id: String, key: String, on: bool) -> Result<Option<PagePanel>, KansoError> {
        self.0.pin(&id, &key, on)
    }

    pub fn hide(
        &self,
        id: String,
        key: String,
        hidden: bool,
    ) -> Result<Option<PagePanel>, KansoError> {
        self.0.hide(&id, &key, hidden)
    }

    pub fn show_hidden(&self, id: String) -> Result<Option<PagePanel>, KansoError> {
        self.0.show_hidden(&id)
    }

    pub fn not_right(
        &self,
        id: String,
        key: String,
        index: u32,
    ) -> Result<Option<PagePanel>, KansoError> {
        self.0.not_right(&id, &key, index)
    }

    pub fn copy_into_note(&self, id: String, key: String) -> Result<bool, KansoError> {
        self.0.copy_into_note(&id, &key)
    }

    pub fn section_markdown(&self, id: String, key: String) -> Result<String, KansoError> {
        self.0.section_markdown(&id, &key)
    }
}
