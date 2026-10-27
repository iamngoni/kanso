//! Icon themes on the phone: the ones on disk and importing another.
//!
//! The icon theme format and importer are still the desktop's own; they move
//! into the engine with the phones' icon themes, and these two calls come
//! alive then. Until then they say so.

use crate::boundary::guard;
use crate::errors::KansoError;
use crate::themes::Themes;
use crate::themes_icons_model::{IconThemeImport, IconThemeRow};

/// Why the calls below fail for now.
const NOT_YET: &str = "Icon theme import arrives with the icon themes.";

impl Themes {
    /// The icon themes in the library's `icon-themes/` folder.
    pub fn icon_themes(&self) -> Result<Vec<IconThemeRow>, KansoError> {
        guard(|| Err(KansoError::invalid(NOT_YET)))
    }

    /// Imports the icon theme at `path` (a Zed icon theme, a theme package or
    /// a Kanso icon theme folder) into the library's `icon-themes/` folder.
    pub fn import_icon_theme(&self, path: String) -> Result<IconThemeImport, KansoError> {
        guard(|| {
            let _ = path;
            Err(KansoError::invalid(NOT_YET))
        })
    }
}
