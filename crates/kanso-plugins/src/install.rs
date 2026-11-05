//! Where plugins live on disk, and putting them there and taking them away.
//!
//! ```text
//! <data>/plugins/<id>/code/          the bundle's files (not for developer mode)
//! <data>/plugins/<id>/data/          its data store
//! ```
//!
//! A plugin's settings and the last 100 network requests Kanso made for it
//! are in the library's `meta` table, not in files: either can hold note
//! text, and the folder beside the library is not encrypted.
//!
//! Installing follows the theme installer: the bundle is written into a
//! hidden folder beside the others and moved into place whole, so a plugin
//! is never half there. Nothing is written outside `plugins/`. Removing
//! deletes the plugin's folder; sweeping its data from notes is the
//! store's ([`kanso_store::Store::remove_plugin_data`]). Nothing here
//! syncs.

use std::io::Write as _;
use std::path::{Path, PathBuf};

use kanso_store::Store;
use serde_json::Value;

use crate::manifest::is_valid_id;
use crate::registry_model::Bundle;

/// The folder all plugins live in.
pub fn plugins_dir(data_dir: &Path) -> PathBuf {
    data_dir.join("plugins")
}

/// Plugin `id`'s own folder. `id` must already be valid.
pub fn plugin_dir(plugins: &Path, id: &str) -> PathBuf {
    debug_assert!(is_valid_id(id));
    plugins.join(id)
}

/// Where an installed bundle's files are.
pub fn code_dir(plugin_dir: &Path) -> PathBuf {
    plugin_dir.join("code")
}

/// A random hidden name beside `parent`'s entries.
fn hidden(parent: &Path, what: &str) -> PathBuf {
    let mut bytes = [0u8; 8];
    getrandom::fill(&mut bytes).expect("the system's random source");
    let tag: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    parent.join(format!(".{what}-{tag}"))
}

/// Writes `bundle`'s files as plugin `bundle.manifest.id`'s code, replacing
/// any it had, and returns the plugin's folder. The files go into a hidden
/// folder first and are moved into place in one rename.
pub fn install_bundle(plugins: &Path, bundle: &Bundle) -> std::io::Result<PathBuf> {
    let id = &bundle.manifest.id;
    if !is_valid_id(id) {
        return Err(std::io::Error::other("not a plugin ID"));
    }
    std::fs::create_dir_all(plugins)?;
    let dir = plugin_dir(plugins, id);
    std::fs::create_dir_all(&dir)?;
    let staging = hidden(&dir, "code");
    std::fs::create_dir(&staging)?;
    let written = (|| {
        let manifest = serde_json::to_vec_pretty(&bundle.manifest)?;
        std::fs::write(staging.join("manifest.json"), manifest)?;
        for (path, bytes) in &bundle.files {
            if !crate::manifest::is_valid_bundle_path(path) || path == "manifest.json" {
                return Err(std::io::Error::other("a bundle path escapes its folder"));
            }
            let target = staging.join(path);
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&target, bytes)?;
        }
        Ok::<_, std::io::Error>(())
    })();
    if let Err(err) = written {
        std::fs::remove_dir_all(&staging).ok();
        return Err(err);
    }
    let code = code_dir(&dir);
    let old = hidden(&dir, "old");
    let had = code.exists();
    if had {
        std::fs::rename(&code, &old)?;
    }
    if let Err(err) = std::fs::rename(&staging, &code) {
        if had {
            std::fs::rename(&old, &code).ok();
        }
        std::fs::remove_dir_all(&staging).ok();
        return Err(err);
    }
    if had {
        std::fs::remove_dir_all(&old).ok();
    }
    std::fs::create_dir_all(dir.join("data"))?;
    Ok(dir)
}

/// Deletes plugin `id`'s folder: code, settings, data and egress log.
pub fn remove(plugins: &Path, id: &str) -> std::io::Result<()> {
    if !is_valid_id(id) {
        return Err(std::io::Error::other("not a plugin ID"));
    }
    let dir = plugin_dir(plugins, id);
    match std::fs::remove_dir_all(&dir) {
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        other => other,
    }
}

/// Writes `bytes` to `path` atomically, readable only by this user.
pub fn write_private(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let dir = path
        .parent()
        .ok_or_else(|| std::io::Error::other("no folder"))?;
    std::fs::create_dir_all(dir)?;
    let temp = hidden(dir, "write");
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt as _;
        options.mode(0o600);
    }
    let result = (|| {
        let mut file = options.open(&temp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        std::fs::rename(&temp, path)
    })();
    if result.is_err() {
        std::fs::remove_file(&temp).ok();
    }
    result
}

/// The `meta` key holding plugin `id`'s settings. Settings are whatever
/// the plugin saves, which can be note text, so they live in the library
/// (encrypted with it), never in a file beside it.
fn settings_key(id: &str) -> String {
    format!("plugin.settings.{id}")
}

/// The `meta` key holding plugin `id`'s egress log. A request's path can
/// carry note text, so the log lives in the library too.
fn egress_key(id: &str) -> String {
    format!("plugin.egress.{id}")
}

/// Reads `file` in `plugin_dir` once, from before these lived in the
/// library: stores it under `key` and deletes the file.
fn adopt_legacy(store: &Store, plugin_dir: &Path, file: &str, key: &str) -> Option<String> {
    let path = plugin_dir.join(file);
    let text = std::fs::read_to_string(&path).ok()?;
    store.set_meta(key, &text).ok()?;
    std::fs::remove_file(&path).ok();
    Some(text)
}

/// Plugin `id`'s settings, or `null` when there are none or they can't be
/// read. `plugin_dir` is only checked for settings an older Kanso left in
/// a file; they move into the library on the first read.
pub fn read_settings(store: &Store, plugin_dir: &Path, id: &str) -> Value {
    let key = settings_key(id);
    store
        .meta(&key)
        .ok()
        .flatten()
        .or_else(|| adopt_legacy(store, plugin_dir, "settings.json", &key))
        .filter(|text| text.len() <= crate::router::MAX_SETTINGS_BYTES)
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or(Value::Null)
}

/// Saves plugin `id`'s settings in the library.
pub fn write_settings(store: &Store, id: &str, value: &Value) -> std::io::Result<()> {
    store
        .set_meta(&settings_key(id), &serde_json::to_string(value)?)
        .map_err(std::io::Error::other)
}

/// The egress log Kanso keeps for plugin `id`.
pub fn read_egress(store: &Store, plugin_dir: &Path, id: &str) -> crate::net::EgressLog {
    let key = egress_key(id);
    let entries: Vec<crate::net::Egress> = store
        .meta(&key)
        .ok()
        .flatten()
        .or_else(|| adopt_legacy(store, plugin_dir, "egress.json", &key))
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default();
    let mut log = crate::net::EgressLog::new();
    // Stored newest first; pushed oldest first.
    for entry in entries.into_iter().rev() {
        log.push(entry);
    }
    log
}

/// Saves the egress log in the library.
pub fn write_egress(store: &Store, id: &str, log: &crate::net::EgressLog) -> std::io::Result<()> {
    store
        .set_meta(&egress_key(id), &serde_json::to_string(&log.entries())?)
        .map_err(std::io::Error::other)
}

/// Forgets plugin `id`'s settings and egress log in the library.
pub fn forget(store: &Store, id: &str) {
    store.remove_meta(&settings_key(id)).ok();
    store.remove_meta(&egress_key(id)).ok();
}

/// The entry script of a plugin: from its installed code, or, in developer
/// mode, read and checked from its folder again.
pub fn entry_script(
    plugin_dir: &Path,
    installed: &crate::model::Installed,
) -> Result<String, String> {
    let bundle_files = match &installed.source {
        crate::model::Source::Developer { path } => {
            let bundle = crate::bundle::open_dir(Path::new(path)).map_err(|e| e.to_string())?;
            if bundle.manifest.id != installed.manifest.id {
                return Err("the folder's manifest has another ID now".into());
            }
            bundle.files
        }
        _ => {
            let code = code_dir(plugin_dir);
            let path = code.join(&installed.manifest.entry);
            // The path was checked at install; keep to the folder anyway.
            if !crate::manifest::is_valid_bundle_path(&installed.manifest.entry) {
                return Err("the entry isn't inside the plugin".into());
            }
            let meta = std::fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
            if !meta.is_file() || meta.len() > crate::manifest::MAX_ENTRY_SCRIPT_BYTES {
                return Err("the entry script is missing or too large".into());
            }
            let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
            return String::from_utf8(bytes).map_err(|_| "the entry script isn't UTF-8".into());
        }
    };
    let bytes = bundle_files
        .into_iter()
        .find(|(path, _)| *path == installed.manifest.entry)
        .map(|(_, bytes)| bytes)
        .ok_or("the entry script is missing")?;
    String::from_utf8(bytes).map_err(|_| "the entry script isn't UTF-8".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Manifest, Permissions, Publisher};

    fn bundle(files: Vec<(&str, &str)>) -> Bundle {
        Bundle {
            manifest: Manifest {
                schema: 1,
                api_version: 1,
                id: "com.example.wordcount".into(),
                name: "Word count".into(),
                version: "1.0.0".into(),
                entry: "main.js".into(),
                publisher: Publisher {
                    name: "Sam".into(),
                    key: format!("ed25519:{}", "ab".repeat(32)),
                },
                min_app_version: "0.1.0".into(),
                icon: None,
                description: None,
                permissions: Permissions::default(),
            },
            files: files
                .into_iter()
                .map(|(p, b)| (p.to_owned(), b.as_bytes().to_vec()))
                .collect(),
            sha256: String::new(),
        }
    }

    #[test]
    fn install_replaces_whole_and_remove_deletes_everything() {
        let data = tempfile::tempdir().unwrap();
        let plugins = plugins_dir(data.path());
        let dir =
            install_bundle(&plugins, &bundle(vec![("main.js", "1"), ("lib/a.js", "a")])).unwrap();
        let store = Store::open(data.path().join("kanso.db")).unwrap();
        let id = "com.example.wordcount";
        write_settings(&store, id, &serde_json::json!({ "x": 1 })).unwrap();
        assert_eq!(read_settings(&store, &dir, id)["x"], 1);
        install_bundle(&plugins, &bundle(vec![("main.js", "2")])).unwrap();
        let code = code_dir(&dir);
        assert_eq!(std::fs::read_to_string(code.join("main.js")).unwrap(), "2");
        assert!(!code.join("lib/a.js").exists());
        // Settings survive an update.
        assert_eq!(read_settings(&store, &dir, id)["x"], 1);
        // No hidden leftovers.
        let names: Vec<String> = std::fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert!(names.iter().all(|n| !n.starts_with('.')), "{names:?}");
        remove(&plugins, "com.example.wordcount").unwrap();
        assert!(!dir.exists());
        remove(&plugins, "com.example.wordcount").unwrap();
        assert!(remove(&plugins, "../evil").is_err());
    }

    #[test]
    fn settings_and_egress_live_in_the_library_not_in_files() {
        let data = tempfile::tempdir().unwrap();
        let store = Store::open(data.path().join("kanso.db")).unwrap();
        let dir = data.path().join("plugins").join("com.example.x");
        let id = "com.example.x";
        write_settings(&store, id, &serde_json::json!({ "recent": "canary" })).unwrap();
        let mut log = crate::net::EgressLog::new();
        log.push(crate::net::Egress {
            at_ms: 1,
            method: "GET".into(),
            host: "example.com".into(),
            url_path: "/canary".into(),
            status: Some(200),
            sent: 0,
            received: 0,
            refused: None,
        });
        write_egress(&store, id, &log).unwrap();
        assert!(!dir.exists());
        assert_eq!(read_settings(&store, &dir, id)["recent"], "canary");
        assert_eq!(read_egress(&store, &dir, id).len(), 1);
        forget(&store, id);
        assert!(read_settings(&store, &dir, id).is_null());
        assert_eq!(read_egress(&store, &dir, id).len(), 0);
    }

    #[test]
    fn files_an_older_kanso_left_move_into_the_library() {
        let data = tempfile::tempdir().unwrap();
        let store = Store::open(data.path().join("kanso.db")).unwrap();
        let dir = data.path().join("plugins").join("com.example.x");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("settings.json"), r#"{"a":1}"#).unwrap();
        assert_eq!(read_settings(&store, &dir, "com.example.x")["a"], 1);
        assert!(!dir.join("settings.json").exists());
        assert_eq!(read_settings(&store, &dir, "com.example.x")["a"], 1);
    }

    #[test]
    fn a_bad_path_writes_nothing() {
        let data = tempfile::tempdir().unwrap();
        let plugins = plugins_dir(data.path());
        assert!(install_bundle(&plugins, &bundle(vec![("../x.js", "1")])).is_err());
        assert!(!data.path().join("x.js").exists());
        let dir = plugin_dir(&plugins, "com.example.wordcount");
        assert!(!code_dir(&dir).exists());
    }
}
