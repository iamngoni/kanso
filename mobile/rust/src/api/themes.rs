//! Themes: the built-in, the person's own and the gallery's, fitted as the
//! desktop fits them, and importing more.

use flutter_rust_bridge::frb;
use kanso_mobile::{
    IconThemeImport, IconThemeRow, KansoError, StyledRun, ThemeImport, ThemeNotice, ThemeRow,
    ThemeTokens, Themes,
};

/// The themes of one library.
#[frb(opaque)]
pub struct KansoThemes(Themes);

impl KansoThemes {
    pub(super) fn new(themes: Themes) -> KansoThemes {
        KansoThemes(themes)
    }

    pub fn list(&self) -> Result<Vec<ThemeRow>, KansoError> {
        self.0.list()
    }

    pub fn notices(&self) -> Result<Vec<ThemeNotice>, KansoError> {
        self.0.notices()
    }

    pub fn fit(
        &self,
        ui: String,
        syntax: Option<String>,
        preview: Option<String>,
    ) -> Result<ThemeTokens, KansoError> {
        self.0.fit(ui, syntax, preview)
    }

    pub fn styled_runs(
        &self,
        text: String,
        start: u32,
        end: u32,
        ui: String,
        syntax: Option<String>,
    ) -> Result<Vec<StyledRun>, KansoError> {
        self.0.styled_runs(text, start, end, ui, syntax)
    }

    pub fn import(&self, path: String) -> Result<ThemeImport, KansoError> {
        self.0.import(path)
    }

    pub fn icon_themes(&self) -> Result<Vec<IconThemeRow>, KansoError> {
        self.0.icon_themes()
    }

    pub fn import_icon_theme(&self, path: String) -> Result<IconThemeImport, KansoError> {
        self.0.import_icon_theme(path)
    }
}
