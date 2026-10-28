//! The theme gallery's plain data: what the Browse page lists.

/// Whether a gallery theme colors the app or draws its icons.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GalleryKind {
    ColorTheme,
    IconTheme,
}

/// The five colors a card draws before the theme itself is fitted, each
/// `0xRRGGBB`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GalleryColors {
    pub editor: u32,
    pub text: u32,
    pub accent: u32,
    pub sidebar: u32,
    pub list: u32,
}

/// One theme in the gallery: what the signed catalog says about it, and
/// what this phone has done with it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GalleryItem {
    /// The theme file's name without `.json`, or an icon theme's folder.
    pub id: String,
    pub name: String,
    pub kind: GalleryKind,
    pub author: String,
    /// Published from Kanso's own account (its `source` is under
    /// `github.com/iamngoni/`).
    pub official: bool,
    /// The catalog's version of the theme, a whole number.
    pub version: String,
    /// How many times it was installed. The signed catalog does not count
    /// installs, so this is 0 for now.
    pub installs: u64,
    /// When the catalog listing it was generated, in milliseconds since the
    /// epoch (a theme has no date of its own); for a theme the catalog no
    /// longer lists, when it was installed. 0 when unknown.
    pub updated_ms: i64,
    /// Where the theme's page is; opened in the browser, never fetched. `None`
    /// for a theme the catalog no longer lists.
    pub source: Option<String>,
    /// A color theme is a dark one. Always `false` for icon themes.
    pub dark: bool,
    /// The colors a card draws before the theme is installed.
    pub headline: Option<GalleryColors>,
    /// This phone has it, whatever state it is in.
    pub installed: bool,
    /// The catalog has a newer version than the installed copy, and the copy
    /// is the one the gallery wrote.
    pub update_available: bool,
    /// This build is new enough to install it.
    pub runs_here: bool,
    /// What the theme is, in a sentence or two.
    pub description: String,
    /// An SPDX licence expression.
    pub license: String,
    /// The family a light and a dark theme pair under, when they do.
    pub family: Option<String>,
    /// The installed copy is not the one the gallery wrote; the gallery will
    /// not overwrite it.
    pub edited: bool,
    /// The catalog withdrew an installed theme the person had edited: it
    /// stays on disk and is no longer drawn.
    pub disabled: bool,
}
