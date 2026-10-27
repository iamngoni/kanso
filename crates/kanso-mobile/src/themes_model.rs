//! The plain data the theme calls return and take (`themes.rs`): rows, notices
//! and import reports. No logic lives here.

use crate::model::ThemeTokens;

/// Where a theme came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeOrigin {
    /// Compiled in, reviewed with the release.
    Builtin,
    /// A file in the library's `themes/` folder, imported or put there.
    Own,
    /// Installed from the gallery, under the library's `gallery/themes/`.
    Gallery,
}

/// One theme the phone can show.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThemeRow {
    /// The theme file's name without `.json`; what [`crate::Themes::fit`]
    /// takes. Unique among the rows.
    pub id: String,
    pub name: String,
    pub author: String,
    pub license: String,
    pub source: Option<String>,
    pub dark: bool,
    pub origin: ThemeOrigin,
    /// The theme fitted alone, as one theme for the interface, the editor and
    /// the preview.
    pub tokens: ThemeTokens,
}

/// A theme file that couldn't be used, and why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThemeNotice {
    /// The file's name.
    pub file: String,
    /// The notice as a sentence, as the desktop's Themes page words it.
    pub sentence: String,
}

/// What an import wrote.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThemeImport {
    /// The IDs of the themes written, which [`crate::Themes::fit`] takes.
    pub ids: Vec<String>,
    /// Their names, in the same order.
    pub names: Vec<String>,
}

/// What draws Mermaid diagrams.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DiagramRenderer {
    /// Pele, Obsidian's renderer: the default.
    #[default]
    Pele,
    /// `mermaid-svg`, Kanso's own.
    Kanso,
}
