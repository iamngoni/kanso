//! The theme gallery: what it lists and what is installed from it.

use flutter_rust_bridge::frb;
use kanso_mobile::{BuiltinTheme, Gallery, GalleryItem, KansoError, ThemeTokens};

#[frb(opaque)]
pub struct KansoGallery(Gallery);

impl KansoGallery {
    pub(super) fn new(gallery: Gallery) -> KansoGallery {
        KansoGallery(gallery)
    }

    pub fn index(&self, refresh: bool) -> Result<Vec<GalleryItem>, KansoError> {
        self.0.index(refresh)
    }

    pub fn install(&self, id: String) -> Result<Vec<GalleryItem>, KansoError> {
        self.0.install(id)
    }

    pub fn uninstall(&self, id: String) -> Result<Vec<GalleryItem>, KansoError> {
        self.0.uninstall(id)
    }

    pub fn installed_themes(&self) -> Result<Vec<BuiltinTheme>, KansoError> {
        self.0.installed_themes()
    }

    pub fn installed_theme_tokens(&self, id: String) -> Result<ThemeTokens, KansoError> {
        self.0.installed_theme_tokens(id)
    }
}
