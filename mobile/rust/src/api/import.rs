//! Importing: the sources a phone can read and which a shared file could be.
//! The import itself, and opening files, are on the library
//! (`api/library.rs`).

use flutter_rust_bridge::frb;
use kanso_mobile::ImportSource;

/// Where notes can be imported from, in the order to list them.
#[frb(sync)]
pub fn import_sources() -> Vec<ImportSource> {
    kanso_mobile::import::import_sources()
}

/// What the file or folder at `path` could be, best first, as the `kind`
/// names the import takes. Empty when it is none of them.
#[frb(sync)]
pub fn import_kinds_for(path: String) -> Vec<String> {
    kanso_mobile::import::import_kinds_for(&path)
}
