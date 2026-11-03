//! Notebook homepages: the card above a notebook's notes, the page a tap
//! opens, its map, the Home note, customising and the settings.

use flutter_rust_bridge::frb;
use kanso_mobile::{KansoError, NotebookHome};

use super::notebook_home_types::{
    HomeCandidate, HomeCard, HomeMap, HomePanel, HomeSectionSetting, HomeSettings, HomeWords,
};

#[frb(opaque)]
pub struct KansoNotebookHome(NotebookHome);

impl KansoNotebookHome {
    pub(super) fn new(home: NotebookHome) -> KansoNotebookHome {
        KansoNotebookHome(home)
    }

    /// The words both platforms show.
    pub fn words(&self) -> HomeWords {
        self.0.words()
    }

    pub fn settings(&self) -> HomeSettings {
        self.0.settings()
    }

    pub fn set_settings(&self, input: HomeSettings) -> Result<HomeSettings, KansoError> {
        self.0.set_settings(input)
    }

    /// The card above notebook `book_id`'s notes.
    pub fn card(&self, book_id: String) -> Result<HomeCard, KansoError> {
        self.0.card(&book_id)
    }

    /// Notebook `book_id`'s homepage; nothing when it is gone.
    pub fn open(&self, book_id: String) -> Result<Option<HomePanel>, KansoError> {
        self.0.open(&book_id)
    }

    pub fn show_all(&self, book_id: String, key: String) -> Result<Option<HomePanel>, KansoError> {
        self.0.show_all(&book_id, &key)
    }

    /// The map on a 320 by 200 canvas.
    pub fn map(&self, book_id: String) -> Result<HomeMap, KansoError> {
        self.0.map(&book_id)
    }

    pub fn refresh_section(
        &self,
        book_id: String,
        key: String,
    ) -> Result<Option<HomePanel>, KansoError> {
        self.0.refresh_section(&book_id, &key)
    }

    pub fn refresh_all(&self, book_id: String) -> Result<Option<HomePanel>, KansoError> {
        self.0.refresh_all(&book_id)
    }

    pub fn pin(
        &self,
        book_id: String,
        key: String,
        on: bool,
    ) -> Result<Option<HomePanel>, KansoError> {
        self.0.pin(&book_id, &key, on)
    }

    pub fn hide(
        &self,
        book_id: String,
        key: String,
        hidden: bool,
    ) -> Result<Option<HomePanel>, KansoError> {
        self.0.hide(&book_id, &key, hidden)
    }

    pub fn show_hidden(&self, book_id: String) -> Result<Option<HomePanel>, KansoError> {
        self.0.show_hidden(&book_id)
    }

    pub fn not_right(
        &self,
        book_id: String,
        key: String,
        index: u32,
    ) -> Result<Option<HomePanel>, KansoError> {
        self.0.not_right(&book_id, &key, index)
    }

    pub fn section_markdown(&self, book_id: String, key: String) -> Result<String, KansoError> {
        self.0.section_markdown(&book_id, &key)
    }

    /// Appends a section to the Home note (made if need be); returns its ID.
    pub fn copy_into_home(&self, book_id: String, key: String) -> Result<String, KansoError> {
        self.0.copy_into_home(&book_id, &key)
    }

    pub fn candidates(&self, book_id: String) -> Result<Vec<HomeCandidate>, KansoError> {
        self.0.candidates(&book_id)
    }

    pub fn choose_home(&self, book_id: String, note_id: String) -> Result<(), KansoError> {
        self.0.choose_home(&book_id, &note_id)
    }

    /// Makes "About <Notebook>" as the Home note; returns its ID.
    pub fn new_home(&self, book_id: String) -> Result<String, KansoError> {
        self.0.new_home(&book_id)
    }

    pub fn clear_home(&self, book_id: String) -> Result<bool, KansoError> {
        self.0.clear_home(&book_id)
    }

    /// After a note moved: true when its Home note mark was removed.
    pub fn after_move(&self, note_id: String) -> Result<bool, KansoError> {
        self.0.after_move(&note_id)
    }

    pub fn sections(&self, book_id: String) -> Result<Vec<HomeSectionSetting>, KansoError> {
        self.0.sections(&book_id)
    }

    pub fn customise(
        &self,
        book_id: String,
        order: Vec<String>,
        hidden: Vec<String>,
    ) -> Result<Option<HomePanel>, KansoError> {
        self.0.customise(&book_id, order, hidden)
    }

    pub fn reset_layout(&self, book_id: String) -> Result<Option<HomePanel>, KansoError> {
        self.0.reset_layout(&book_id)
    }
}
