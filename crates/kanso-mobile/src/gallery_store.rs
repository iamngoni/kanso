//! The gallery's work on the library's folder: installing a checked catalog
//! entry under `<library folder>/gallery/`, reconciling what is on disk with
//! `state.json`, removing a theme, and turning a crash between a swap and a
//! state write into a recorded or discarded pending entry.
//!
//! This is the desktop's `kanso-app/src/gallery.rs` for the phone: the same
//! records, the same refusals (an edited copy is never overwritten, a file
//! the gallery did not write is never replaced, a built-in's name is never
//! shadowed, a color theme must read well), the same layout on disk. It
//! never touches the network.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use kanso_engine::theme::file::{ThemeFile, problems};
use kanso_gallery::state::{Pending, Record, now_rfc3339};
use kanso_gallery::{IconCaps, Kind, State, Theme, is_valid_name};

/// Installs run on the caller's thread, but they read and write one
/// `state.json`; one at a time keeps two halves of a pair from losing each
/// other's record.
static INSTALL_LOCK: Mutex<()> = Mutex::new(());

/// The icon theme file's schema version this build reads.
const ICON_SCHEMA_VERSION: u64 = 1;

/// The largest icon theme file (`<id>.json`).
const ICON_JSON_BYTES: u64 = 512 * 1024;

/// The gallery's folder inside the library's folder: `themes/`,
/// `icon-themes/`, `state.json`, the cached index and `meta.json`.
pub(crate) fn root_of(library_database: &Path) -> PathBuf {
    library_database
        .parent()
        .filter(|dir| !dir.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .join("gallery")
}

/// The digest of an installed record's bytes now: a color file's SHA-256, or
/// an icon folder's tree digest. `None` when it is missing.
pub(crate) fn current_digest(root: &Path, record: &Record) -> Option<String> {
    if !is_valid_name(&record.id) {
        return None;
    }
    match record.kind {
        Kind::Color => {
            let bytes =
                std::fs::read(root.join("themes").join(format!("{}.json", record.id))).ok()?;
            Some(kanso_gallery::color_digest(&bytes))
        }
        Kind::Icons => {
            let folder = root.join("icon-themes").join(&record.id);
            kanso_update::stage::payload_digest(&folder).ok()
        }
    }
}

/// Resolves a pending record from an earlier run and returns the state
/// reconciled against the bytes on disk. A swap that completed is recorded;
/// one that didn't is forgotten, so the old copy stays installed.
pub(crate) fn reconcile(root: &Path) -> State {
    let path = root.join("state.json");
    let mut state = State::load(&path);
    let known = state.installed.clone();
    let pending = state.pending.clone();
    let before = state.clone();
    state.reconcile(|id| {
        if let Some(record) = known.iter().find(|record| record.id == id) {
            return current_digest(root, record);
        }
        let swap = pending.iter().find(|swap| swap.id == id)?;
        let record = Record {
            id: swap.id.clone(),
            kind: swap.kind,
            version: swap.version,
            name: swap.name.clone(),
            family: swap.family.clone(),
            appearance: swap.appearance,
            digest: swap.digest.clone(),
            installed_at: swap.installed_at.clone(),
        };
        current_digest(root, &record)
    });
    if state != before {
        state.save(&path).ok();
    }
    state
}

/// The installed gallery themes, each with the digest of what is on disk
/// now, for the catalog's plan.
pub(crate) fn installed(root: &Path) -> Vec<kanso_gallery::Installed> {
    let state = reconcile(root);
    state.plan_records(|record| current_digest(root, record))
}

/// Keeps a revoked, edited theme on disk but out of the loaders. Returns
/// whether it was newly disabled, so a check that finds it again doesn't
/// announce it again.
pub(crate) fn disable(root: &Path, id: &str) -> bool {
    let _installing = INSTALL_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let path = root.join("state.json");
    let mut state = State::load(&path);
    if !state.disabled.iter().any(|other| other == id) {
        state.disabled.push(id.to_owned());
        state.disabled.sort();
        state.save(&path).ok();
        return true;
    }
    false
}

/// Installs a color theme the catalog pinned, after holding it to the same
/// parser and legibility checks as an imported theme. `reserved` holds the
/// lowercased names of the built-in themes, which a gallery install must not
/// shadow.
pub(crate) fn install_color(
    root: &Path,
    theme: &Theme,
    bytes: &[u8],
    reserved: &HashSet<String>,
    automatic: bool,
) -> Result<(), String> {
    if theme.kind != Kind::Color {
        return Err("that entry isn't a color theme".into());
    }
    check_id(&theme.id)?;
    let _installing = INSTALL_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let text = std::str::from_utf8(bytes).map_err(|_| "the theme isn't UTF-8".to_owned())?;
    let file = ThemeFile::parse(text).map_err(|err| format!("the theme {err}"))?;
    if file.name != theme.name {
        return Err(format!(
            "the catalog calls it \"{}\", but the file calls itself \"{}\"",
            theme.name, file.name
        ));
    }
    if reserved.contains(&file.name.to_lowercase()) {
        return Err(format!(
            "\"{}\" is already a built-in theme; it can't be installed",
            file.name
        ));
    }
    if let Some(problem) = problems(&file).first() {
        return Err(format!("it doesn't read well: {problem}"));
    }
    let path = root.join("state.json");
    let mut state = State::load(&path);
    check_replacement(root, &state, theme.kind, &theme.id, automatic)?;
    check_install_cap(&state, theme.kind, &theme.id)?;
    let pending = Pending {
        id: theme.id.clone(),
        kind: Kind::Color,
        version: theme.version,
        name: theme.name.clone(),
        family: theme.family.clone(),
        appearance: theme.appearance,
        digest: kanso_gallery::color_digest(bytes),
        installed_at: now_rfc3339(),
    };
    state.pending.push(pending.clone());
    state.save(&path).map_err(|err| err.to_string())?;
    if let Err(err) = kanso_gallery::install_color(root, theme, bytes) {
        forget_pending(&mut state, &theme.id);
        state.save(&path).ok();
        return Err(err.to_string());
    }
    record(&mut state, pending);
    state.save(&path).map_err(|err| err.to_string())
}

/// Installs an icon theme the catalog pinned, checks the unpacked folder,
/// and records it. `reserved_ids` and `reserved_names` are the built-in icon
/// themes' ids and lowercased display names.
pub(crate) fn install_icons(
    root: &Path,
    theme: &Theme,
    bundle: &[u8],
    reserved_ids: &HashSet<String>,
    reserved_names: &HashSet<String>,
    automatic: bool,
) -> Result<(), String> {
    if theme.kind != Kind::Icons {
        return Err("that entry isn't an icon theme".into());
    }
    check_id(&theme.id)?;
    let _installing = INSTALL_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if reserved_ids.contains(&theme.id) || reserved_names.contains(&theme.name.to_lowercase()) {
        return Err(format!(
            "\"{}\" is already a built-in icon theme",
            theme.name
        ));
    }
    let path = root.join("state.json");
    let mut state = State::load(&path);
    check_replacement(root, &state, theme.kind, &theme.id, automatic)?;
    check_install_cap(&state, theme.kind, &theme.id)?;
    let mut pending: Option<Pending> = None;
    let installed_at = now_rfc3339();
    let result = kanso_gallery::install_icons_with(
        root,
        theme,
        bundle,
        IconCaps::default(),
        &mut |folder, digest| {
            // Hold the staged folder to the reader's first rules before the
            // swap. A failure here leaves the working older copy installed.
            check_icon_folder(folder, &theme.id).map_err(std::io::Error::other)?;
            let record = Pending {
                id: theme.id.clone(),
                kind: Kind::Icons,
                version: theme.version,
                name: theme.name.clone(),
                family: theme.family.clone(),
                appearance: theme.appearance,
                digest: digest.to_owned(),
                installed_at: installed_at.clone(),
            };
            let mut with_pending = state.clone();
            with_pending.pending.push(record.clone());
            pending = Some(record);
            with_pending.save(&path)
        },
    );
    let digest = match result {
        Ok(digest) => digest,
        Err(err) => {
            if let Some(pending) = pending {
                state.pending.retain(|other| other.id != pending.id);
                state.save(&path).ok();
            }
            return Err(err.to_string());
        }
    };
    let Some(pending) = pending else {
        return Err("the icon theme wasn't staged".into());
    };
    debug_assert_eq!(pending.digest, digest);
    record(&mut state, pending);
    state.save(&path).map_err(|err| err.to_string())
}

/// Removes an installed gallery theme and its record. Anything else in the
/// library's folder is never touched.
pub(crate) fn remove(root: &Path, kind: Kind, id: &str) -> Result<(), String> {
    check_id(id)?;
    let _installing = INSTALL_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    kanso_gallery::remove_installed(root, kind, id).map_err(|err| err.to_string())?;
    let path = root.join("state.json");
    let mut state = State::load(&path);
    state.remove(id);
    state.disabled.retain(|other| other != id);
    state.save(&path).map_err(|err| err.to_string())
}

/// The folder and file name a theme is installed under come from the signed
/// catalog, which only lists valid ones; the check is repeated here so no
/// path is ever built from a name that could leave the gallery's folders.
fn check_id(id: &str) -> Result<(), String> {
    if is_valid_name(id) {
        Ok(())
    } else {
        Err(format!(
            "\"{id}\" isn't a name a theme can be installed under"
        ))
    }
}

/// The first rules of an icon theme's folder: its theme file reads, at the
/// schema this build knows. (The phone does not draw icon themes yet; the
/// shell's own reader will hold them to the rest.)
fn check_icon_folder(folder: &Path, id: &str) -> Result<(), String> {
    let file = folder.join(format!("{id}.json"));
    let bytes = kanso_gallery::install::read_capped(&file, ICON_JSON_BYTES)
        .map_err(|_| "the icon theme has no readable theme file".to_owned())?;
    let value: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|_| "the icon theme's file isn't valid JSON".to_owned())?;
    match value.get("schema_version").and_then(|v| v.as_u64()) {
        Some(ICON_SCHEMA_VERSION) => Ok(()),
        _ => Err("the icon theme's file has a schema this build doesn't read".into()),
    }
}

/// Refuses to write over a copy that was edited, or over an unrecorded file.
/// A missing copy may be reinstalled by hand, never automatically. Checked
/// here, inside the install, so a change between the plan and the write
/// cannot slip through.
fn check_replacement(
    root: &Path,
    state: &State,
    kind: Kind,
    id: &str,
    automatic: bool,
) -> Result<(), String> {
    let target = installed_path(root, kind, id);
    match state.record(id) {
        Some(record) => match current_digest(root, record) {
            Some(digest) if digest == record.digest => Ok(()),
            None if !automatic => Ok(()),
            None => Err(format!(
                "\"{}\" is missing; install it again from Browse",
                record.name
            )),
            Some(_) => Err(format!(
                "\"{}\" was edited; the gallery won't overwrite your copy",
                record.name
            )),
        },
        None if target.exists() => Err(format!(
            "a file named \"{id}\" is already installed but not by the gallery"
        )),
        None => Ok(()),
    }
}

fn check_install_cap(state: &State, kind: Kind, id: &str) -> Result<(), String> {
    let limit = match kind {
        Kind::Color => kanso_gallery::MAX_GALLERY_COLOR_THEMES,
        Kind::Icons => kanso_gallery::MAX_GALLERY_ICON_THEMES,
    };
    let count = state
        .installed
        .iter()
        .filter(|record| record.kind == kind && record.id != id)
        .count();
    if count >= limit {
        return Err(format!(
            "there are already {limit} gallery {} themes installed; remove one first",
            kind.as_str()
        ));
    }
    Ok(())
}

fn installed_path(root: &Path, kind: Kind, id: &str) -> PathBuf {
    match kind {
        Kind::Color => root.join("themes").join(format!("{id}.json")),
        Kind::Icons => root.join("icon-themes").join(id),
    }
}

fn record(state: &mut State, pending: Pending) {
    state.pending.retain(|other| other.id != pending.id);
    state.insert(Record {
        id: pending.id,
        kind: pending.kind,
        version: pending.version,
        name: pending.name,
        family: pending.family,
        appearance: pending.appearance,
        digest: pending.digest,
        installed_at: pending.installed_at,
    });
}

fn forget_pending(state: &mut State, id: &str) {
    state.pending.retain(|other| other.id != id);
}

#[cfg(test)]
mod tests {
    use super::*;
    use kanso_gallery::{FileRef, sha256_hex};
    use proptest::prelude::*;

    fn color_theme(id: &str) -> (Theme, Vec<u8>) {
        let (_, text) = kanso_engine::theme::builtin::BUILTIN[0];
        let mut value: serde_json::Value = serde_json::from_str(text).unwrap();
        value["name"] = "Store Test".into();
        let bytes = serde_json::to_vec(&value).unwrap();
        (
            Theme {
                id: id.into(),
                kind: Kind::Color,
                family: None,
                version: 1,
                name: "Store Test".into(),
                appearance: Some(kanso_gallery::Appearance::Light),
                author: "Author".into(),
                license: "MIT".into(),
                source: "https://example.com".into(),
                description: "A test.".into(),
                min_kanso: "0.1.0".into(),
                file: Some(FileRef {
                    path: format!("{id}.json"),
                    sha256: sha256_hex(&bytes),
                    size: bytes.len() as u64,
                }),
                bundle: None,
                preview: Vec::new(),
                colors: None,
            },
            bytes,
        )
    }

    /// Every path under `dir`, relative and sorted, with file sizes.
    fn tree(dir: &Path) -> Vec<(PathBuf, u64)> {
        fn walk(base: &Path, at: &Path, out: &mut Vec<(PathBuf, u64)>) {
            let Ok(read) = std::fs::read_dir(at) else {
                return;
            };
            for entry in read.flatten() {
                let path = entry.path();
                let meta = std::fs::symlink_metadata(&path).unwrap();
                out.push((path.strip_prefix(base).unwrap().to_owned(), meta.len()));
                if meta.is_dir() {
                    walk(base, &path, out);
                }
            }
        }
        let mut out = Vec::new();
        walk(dir, dir, &mut out);
        out.sort();
        out
    }

    // no-prop-test: one fixed sequence, install then remove, with the
    // record checked at both ends.
    #[test]
    fn installing_records_what_was_written_and_removing_forgets_it() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("gallery");
        let (theme, bytes) = color_theme("store-test");
        install_color(&root, &theme, &bytes, &HashSet::new(), false).unwrap();
        let records = installed(&root);
        assert_eq!(records.len(), 1);
        assert!(records[0].is_unedited());
        remove(&root, Kind::Color, "store-test").unwrap();
        assert!(installed(&root).is_empty());
        assert!(!root.join("themes/store-test.json").exists());
    }

    // no-prop-test: a built-in's name, an edit and a stranger's file are
    // three fixed refusals.
    #[test]
    fn the_desktops_refusals_hold() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("gallery");
        let (theme, bytes) = color_theme("store-test");
        let reserved: HashSet<String> = ["store test".to_owned()].into();
        assert!(install_color(&root, &theme, &bytes, &reserved, false).is_err());

        install_color(&root, &theme, &bytes, &HashSet::new(), false).unwrap();
        std::fs::write(root.join("themes/store-test.json"), b"{\"edited\":true}").unwrap();
        let err = install_color(&root, &theme, &bytes, &HashSet::new(), false).unwrap_err();
        assert!(err.contains("edited"), "{err}");

        let (other, other_bytes) = color_theme("stranger");
        std::fs::write(root.join("themes/stranger.json"), b"{}").unwrap();
        let err = install_color(&root, &other, &other_bytes, &HashSet::new(), false).unwrap_err();
        assert!(err.contains("not by the gallery"), "{err}");
    }

    proptest! {
        /// Whatever name the catalog (or a bug) hands in, an install or a
        /// removal writes nothing outside the gallery's own folders: the
        /// library folder's tree is unchanged on a refusal, and a success
        /// stays under `gallery/themes` and `gallery/staging`.
        #[test]
        fn prop_hostile_names_never_write_outside_the_gallery(id in prop_oneof![".{0,40}", "[a-z0-9][a-z0-9_-]{0,12}"]) {
            let dir = tempfile::tempdir().unwrap();
            let library = dir.path().join("library");
            std::fs::create_dir_all(&library).unwrap();
            std::fs::write(library.join("library.db"), b"db").unwrap();
            let root = library.join("gallery");
            let (mut theme, bytes) = color_theme("store-test");
            theme.id = id.clone();
            if let Some(file) = theme.file.as_mut() {
                file.path = format!("{id}.json");
            }
            let before = tree(dir.path());
            let result = install_color(&root, &theme, &bytes, &HashSet::new(), false);
            let removed = remove(&root, Kind::Color, &id);
            if is_valid_name(&id) {
                prop_assert!(result.is_ok());
                prop_assert!(removed.is_ok());
            } else {
                prop_assert!(result.is_err());
                prop_assert!(removed.is_err());
                prop_assert_eq!(before, tree(dir.path()));
            }
            for (path, _) in tree(dir.path()) {
                prop_assert!(
                    path.starts_with("library"),
                    "{path:?} is outside the library"
                );
                prop_assert!(
                    path == Path::new("library")
                        || path == Path::new("library/library.db")
                        || path.starts_with("library/gallery"),
                    "{path:?} isn't the database or the gallery's"
                );
            }
        }
    }
}
