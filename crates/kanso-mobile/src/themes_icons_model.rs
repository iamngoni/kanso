//! The plain data the icon theme calls return (`themes_icons.rs`).

use crate::themes_model::ThemeOrigin;

/// One icon theme on the phone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IconThemeRow {
    /// The theme's folder name.
    pub id: String,
    pub name: String,
    pub author: String,
    pub license: String,
    pub origin: ThemeOrigin,
    /// How many icons the theme draws.
    pub icon_count: u32,
}

/// What an icon theme import wrote.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IconThemeImport {
    pub ids: Vec<String>,
    pub names: Vec<String>,
    /// The sentence the desktop shows after an import: what was carried over
    /// and what was left to Kanso's own icons.
    pub summary: String,
}
