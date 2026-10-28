//! Where Kanso keeps its files.

use std::path::{Path, PathBuf};

/// Kanso's data folder, or `$KANSO_DATA_DIR` when set:
/// `~/Library/Application Support/Kanso` on macOS, `%APPDATA%\Kanso` on
/// Windows, and `$XDG_DATA_HOME/kanso` (`~/.local/share/kanso`) elsewhere.
pub fn default_data_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("KANSO_DATA_DIR") {
        return PathBuf::from(dir);
    }
    platform_data_dir()
}

#[cfg(target_os = "macos")]
fn platform_data_dir() -> PathBuf {
    home()
        .join("Library/Application Support")
        .join(crate::flavour::CURRENT.data_folder)
}

#[cfg(target_os = "windows")]
fn platform_data_dir() -> PathBuf {
    non_empty_var("APPDATA")
        .map_or_else(|| home().join("AppData").join("Roaming"), PathBuf::from)
        .join(crate::flavour::CURRENT.data_folder)
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn platform_data_dir() -> PathBuf {
    xdg_dir("XDG_DATA_HOME", ".local/share").join(crate::flavour::CURRENT.xdg_name)
}

/// An XDG base directory: `$var` when it is an absolute path, otherwise
/// `fallback` under the home folder.
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn xdg_dir(var: &str, fallback: &str) -> PathBuf {
    non_empty_var(var)
        .map(PathBuf::from)
        .filter(|dir| dir.is_absolute())
        .unwrap_or_else(|| home().join(fallback))
}

#[cfg(not(target_os = "macos"))]
fn non_empty_var(name: &str) -> Option<std::ffi::OsString> {
    std::env::var_os(name).filter(|value| !value.is_empty())
}

/// The user's home folder: `$HOME`, or `%USERPROFILE%` on Windows.
pub fn home() -> PathBuf {
    #[cfg(target_os = "windows")]
    if let Some(profile) = non_empty_var("USERPROFILE") {
        return PathBuf::from(profile);
    }
    std::env::var_os("HOME").map_or_else(|| PathBuf::from("/"), PathBuf::from)
}

pub fn db_path(data_dir: &Path) -> PathBuf {
    data_dir.join("kanso.db")
}

pub fn config_path(data_dir: &Path) -> PathBuf {
    data_dir.join("config.json")
}

pub fn backups_dir(data_dir: &Path) -> PathBuf {
    kanso_engine::backup::backups_dir(data_dir)
}

/// What starts Kanso at login: the LaunchAgent on macOS, an XDG autostart
/// entry on Linux.
#[cfg(target_os = "macos")]
pub fn launch_agent_path() -> PathBuf {
    home()
        .join("Library/LaunchAgents")
        .join(format!("{}.plist", crate::flavour::CURRENT.bundle_id))
}

/// What starts Kanso at login: the LaunchAgent on macOS, an XDG autostart
/// entry on Linux. (Windows uses the Run key instead; see `launch`.)
#[cfg(not(target_os = "macos"))]
pub fn launch_agent_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    let config = home().join("AppData").join("Roaming");
    #[cfg(not(target_os = "windows"))]
    let config = xdg_dir("XDG_CONFIG_HOME", ".config");
    config
        .join("autostart")
        .join(format!("{}.desktop", crate::flavour::CURRENT.xdg_name))
}

/// The button that shows a file in the system's file manager.
#[cfg(target_os = "macos")]
pub const SHOW_IN_FILE_MANAGER: &str = "Show in Finder";
#[cfg(target_os = "windows")]
pub const SHOW_IN_FILE_MANAGER: &str = "Show in Explorer";
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub const SHOW_IN_FILE_MANAGER: &str = "Show in Files";
