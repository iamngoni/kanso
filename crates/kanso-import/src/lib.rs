//! Imports other note apps' libraries into Kanso's store.
//!
//! [`library`] reads folders of Markdown (plain, Obsidian vaults, Logseq
//! graphs) and other apps' exports (Notion, Evernote, Bear, Joplin,
//! Standard Notes, Simplenote, UpNote), and is the importer behind
//! `kanso import` and the app's Import… window.
//!
//! The rest of the crate imports a PouchDB notes library: an Electron notes
//! app that keeps its documents in PouchDB on SQLite, in
//! `<library dir>/db/<32 hex>.sqlite`. The importer snapshots that file
//! read-only (the app may have it open), then copies the winning revision of
//! every note, notebook, tag and file, keeping their IDs and revisions.
//! Attachment bytes are checked against their MD5 digests; the library's
//! `activity:` log is archived verbatim; design documents, `config` and
//! `_local` documents are skipped.

mod config;
mod import;
pub mod library;
pub mod pouch;
mod report;
pub mod safe_storage;

use std::path::{Path, PathBuf};

use anyhow::{Context as _, bail};

pub use config::{
    DEFAULT_BIND_ADDRESS, DEFAULT_PORT, LocalServerConfig, LocalServerCredentials,
    parse_local_server_config, read_local_server_config,
};
pub use import::{Phase, Progress, import};
pub use report::{ImportReport, SourceCounts};

/// Whether `name` is a main database file name: 32 hex digits and `.sqlite`.
/// Excludes `*-mrview-*` view databases and `-wal`/`-shm` companions.
fn is_main_db_name(name: &str) -> bool {
    name.strip_suffix(".sqlite")
        .is_some_and(|stem| stem.len() == 32 && stem.bytes().all(|b| b.is_ascii_hexdigit()))
}

/// The main database in `<library_dir>/db`: the single `<32 hex>.sqlite` file.
pub fn find_pouch_db(library_dir: &Path) -> anyhow::Result<PathBuf> {
    let db_dir = library_dir.join("db");
    let entries =
        std::fs::read_dir(&db_dir).with_context(|| format!("cannot list {}", db_dir.display()))?;
    let mut found = Vec::new();
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if entry.file_name().to_str().is_some_and(is_main_db_name) && path.is_file() {
            found.push(path);
        }
    }
    found.sort();
    match found.len() {
        0 => bail!(
            "no PouchDB library (<32 hex digits>.sqlite) in {}",
            db_dir.display()
        ),
        1 => Ok(found.remove(0)),
        _ => bail!(
            "several PouchDB libraries in {}: {}",
            db_dir.display(),
            found
                .iter()
                .filter_map(|p| p.file_name()?.to_str())
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main_db_names() {
        assert!(is_main_db_name("5c172db466294e6a0b7855cdba0b8d18.sqlite"));
        assert!(!is_main_db_name(
            "5c172db466294e6a0b7855cdba0b8d18-mrview-21f5a21417c6d244c49767ad979917ae.sqlite"
        ));
        assert!(!is_main_db_name(
            "5c172db466294e6a0b7855cdba0b8d18.sqlite-wal"
        ));
        assert!(!is_main_db_name("5c172db466294e6a0b7855cdba0b8d18"));
        assert!(!is_main_db_name("fts.db"));
    }
}
