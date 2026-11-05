//! What the window for a library that won't open says (issue #243, step
//! 10): the plain message, the steps that fix it, and which buttons make
//! sense. No drawing here and no key: the text is made from the kind of
//! failure, and any text that came from another program (the keychain, the
//! database) goes through [`redact`] first.

use std::path::Path;

use kanso_store::StoreError;
use kanso_sync::OpenLibraryError;
use serde::{Deserialize, Serialize};

/// What went wrong when Kanso started, before any window opened.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Failure {
    /// The data folder couldn't be made.
    Folder(String),
    /// Another program holds the data-folder lock.
    Locked,
    /// The lock file couldn't be taken at all.
    LockFailed(String),
    /// `config.json` couldn't be read.
    Config(String),
    /// The library wouldn't open.
    Library(Cause),
}

/// Why the library wouldn't open.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Cause {
    WrongKey,
    /// A conversion marker is stuck or unexpected.
    Converting,
    DiskFull {
        needed_mb: u64,
    },
    /// The conversion failed and rolled back; the text is why.
    EncryptFailed(String),
    /// The keychain couldn't be read at all.
    Keychain(String),
    NoEncryption,
    /// A file couldn't be read or written.
    Io(String),
    Other(String),
}

/// A button the window can have. Show folder, Copy details and Quit are
/// always there.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    TryAgain,
    ShowFolder,
    CopyDetails,
    Quit,
}

/// The window's whole content. It crosses to the window's process as JSON,
/// so it holds nothing but text.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Screen {
    pub title: String,
    pub message: String,
    pub steps: Vec<String>,
    pub actions: Vec<Action>,
    /// What Copy details puts on the clipboard.
    pub details: String,
    /// The folder Show folder opens.
    pub folder: String,
}

impl Screen {
    pub fn retry(&self) -> bool {
        self.actions.contains(&Action::TryAgain)
    }

    /// Every piece of text on the screen, for checks.
    #[cfg(test)]
    pub fn texts(&self) -> impl Iterator<Item = &str> {
        [&self.title, &self.message, &self.details, &self.folder]
            .into_iter()
            .map(String::as_str)
            .chain(self.steps.iter().map(String::as_str))
    }
}

/// Text from outside Kanso with anything that looks like a key taken out.
///
/// A word is a run of letters, digits and `_ - + = /`. Each part of it
/// between slashes that is 32 or more characters long (a hex or base64url
/// key, a `kcl_` API key) becomes `[hidden]`, unless it is a UUID: a folder
/// can be named by one, and Kanso keeps no secret in that shape. A whole
/// word of 32 or more characters with a slash in it is standard base64, not
/// a path, when it ends in `=`, or when it doesn't start with a slash or go
/// on a path after one space or dot (`Application Support/…`) and has a `+` or
/// mixes digits with upper- and lowercase letters; it becomes `[hidden]`
/// too. A path keeps its ordinary folder names.
pub fn redact(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut word = String::new();
    // Whether the last word was in a path and one space or dot came after
    // it: the next word is then the rest of the path (`Application
    // Support/…`, `v1.2/…`).
    let mut path_goes_on = false;
    let mut between = 0;
    for c in text.chars() {
        if in_word(c) {
            word.push(c);
            continue;
        }
        if !word.is_empty() {
            let in_path = path_goes_on && between == 1;
            out.push_str(&redact_word(&word, in_path));
            path_goes_on = in_path || word.contains('/');
            between = 0;
            word.clear();
        }
        if matches!(c, ' ' | '.') {
            between += 1;
        } else {
            path_goes_on = false;
        }
        out.push(c);
    }
    out.push_str(&redact_word(&word, path_goes_on && between == 1));
    out
}

/// How long a run must be to count as a key.
const KEY_RUN: usize = 32;

fn in_word(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '+' | '=' | '/')
}

fn redact_word(word: &str, in_path: bool) -> String {
    if word.len() >= KEY_RUN && word.contains('/') && looks_like_base64(word, in_path) {
        return "[hidden]".to_owned();
    }
    word.split('/')
        .map(|part| {
            if part.len() >= KEY_RUN && !is_uuid(part) {
                "[hidden]"
            } else {
                part
            }
        })
        .collect::<Vec<_>>()
        .join("/")
}

/// Whether `word`, which has a slash, reads as standard base64 rather than
/// a path. `in_path`: it goes on a path that had a space in it.
fn looks_like_base64(word: &str, in_path: bool) -> bool {
    let has = |test: fn(&char) -> bool| word.chars().any(|c| test(&c));
    word.ends_with('=')
        || (!word.starts_with('/')
            && !in_path
            && (word.contains('+')
                || (has(char::is_ascii_digit)
                    && has(char::is_ascii_uppercase)
                    && has(char::is_ascii_lowercase))))
}

/// `8-4-4-4-12` hex digits.
fn is_uuid(part: &str) -> bool {
    let groups: Vec<&str> = part.split('-').collect();
    groups.len() == 5
        && groups
            .iter()
            .zip([8, 4, 4, 4, 12])
            .all(|(group, len)| group.len() == len && group.chars().all(|c| c.is_ascii_hexdigit()))
}

/// The kind of failure an error from opening the library is.
pub fn classify(error: &anyhow::Error) -> Cause {
    for cause in error.chain() {
        if let Some(OpenLibraryError::Keychain(why)) = cause.downcast_ref::<OpenLibraryError>() {
            return Cause::Keychain(redact(&why.0));
        }
        let store =
            cause
                .downcast_ref::<StoreError>()
                .or(match cause.downcast_ref::<OpenLibraryError>() {
                    Some(OpenLibraryError::Store(store)) => Some(store),
                    _ => None,
                });
        match store {
            Some(StoreError::WrongKey) => return Cause::WrongKey,
            Some(StoreError::Converting) => return Cause::Converting,
            Some(StoreError::DiskFull { needed_mb }) => {
                return Cause::DiskFull {
                    needed_mb: *needed_mb,
                };
            }
            Some(StoreError::EncryptFailed(why)) => return Cause::EncryptFailed(redact(why)),
            Some(StoreError::NoEncryption) => return Cause::NoEncryption,
            Some(StoreError::Io(why)) => return Cause::Io(redact(&why.to_string())),
            _ => {}
        }
    }
    Cause::Other(redact(&format!("{error:#}")))
}

fn file(data_dir: &Path, name: &str) -> String {
    data_dir.join(name).display().to_string()
}

const ADVICE: &str = "Choose Copy details and send them with a bug report if this keeps happening.";

/// The window for `failure`, in the data folder `data_dir`.
pub fn describe(failure: &Failure, data_dir: &Path) -> Screen {
    let folder = data_dir.display().to_string();
    let (title, message, steps, retry, technical): (&str, String, Vec<String>, bool, String) =
        match failure {
            Failure::Locked => (
                "Kanso is already open",
                format!("Another Kanso, or a Kanso command, is using the data folder {folder}."),
                vec![
                    "Look for the open Kanso in the Dock, the taskbar or the tray, and switch to it.".into(),
                    "If you can't find it, quit it from Activity Monitor or Task Manager, or restart this computer.".into(),
                    "Choose Try again.".into(),
                ],
                true,
                "the data-folder lock is held by another process".into(),
            ),
            Failure::Folder(why) => (
                "Kanso can't use its data folder",
                format!("Kanso couldn't create the data folder {folder}: {}", redact(why)),
                vec![
                    "Check that the disk the folder is on is connected and has free space.".into(),
                    "Check that you are allowed to write there, or start Kanso with --data-dir and a folder you can write to.".into(),
                    "Choose Try again.".into(),
                ],
                true,
                redact(why),
            ),
            Failure::LockFailed(why) => (
                "Kanso can't lock its data folder",
                format!("Kanso couldn't take the lock file in {folder}: {}", redact(why)),
                vec![
                    "Check that you are allowed to write in the folder and that its disk is connected.".into(),
                    "Choose Try again.".into(),
                ],
                true,
                redact(why),
            ),
            Failure::Config(why) => (
                "Kanso can't read its settings",
                format!("The settings file {} couldn't be read: {}", file(data_dir, "config.json"), redact(why)),
                vec![
                    "Open config.json in the Kanso folder and fix the problem it names. Choose Show folder to find it.".into(),
                    "Or move config.json out of the folder. Kanso then starts with its default settings.".into(),
                    "Choose Try again.".into(),
                ],
                true,
                redact(why),
            ),
            Failure::Library(cause) => return describe_library(cause, data_dir),
        };
    screen(title, message, steps, retry, &technical, &folder)
}

fn describe_library(cause: &Cause, data_dir: &Path) -> Screen {
    let folder = data_dir.display().to_string();
    let (title, message, steps, retry, technical): (&str, String, Vec<String>, bool, String) =
        match cause {
            Cause::WrongKey => (
                "Kanso can't open your library",
                format!(
                    "Your library {} is encrypted, and this computer's key doesn't open it. Nothing was changed and your notes are not lost.",
                    file(data_dir, "kanso.db")
                ),
                vec![
                    "If you moved this library here from another computer, copy it back, or restore a backup with the 24-word library key you saved.".into(),
                    "If you use a different user account than before, or this computer's keychain was reset, sign in to the account that made the library.".into(),
                    "Keep kanso.db and kanso.db.keyid in the Kanso folder as they are. Don't delete them.".into(),
                ],
                false,
                "wrong or missing library key".into(),
            ),
            Cause::Converting => (
                "Kanso can't finish encrypting your library",
                format!(
                    "Kanso was encrypting your library, or was stopped while doing it, and left a marker ({}) that it can't finish from. Your notes are kept.",
                    file(data_dir, "kanso.db.encrypting")
                ),
                vec![
                    "Quit any other Kanso using this folder, then choose Try again. Kanso picks up where it stopped, or puts your library back as it was.".into(),
                    "If it comes back, keep kanso.db, kanso.db.encrypting, kanso.db.enc and kanso.db.plain in the Kanso folder. They hold your notes. Don't delete them.".into(),
                    ADVICE.into(),
                ],
                true,
                "encryption marker is stuck".into(),
            ),
            Cause::DiskFull { needed_mb } => (
                "There isn't enough free space",
                format!(
                    "There isn't enough free space to encrypt your library. Kanso needs about {needed_mb} MB free on the disk that holds {folder}. Your notes are untouched."
                ),
                vec![
                    format!("Free about {needed_mb} MB on the disk that holds the Kanso folder."),
                    "Choose Try again.".into(),
                ],
                true,
                format!("not enough disk space, about {needed_mb} MB needed"),
            ),
            Cause::EncryptFailed(why) => (
                "Kanso couldn't encrypt your library",
                format!("Encrypting stopped and was undone, so your library is as it was. The reason: {why}"),
                vec![
                    "Check that the disk that holds the Kanso folder has free space about the size of your library.".into(),
                    "Choose Try again.".into(),
                    ADVICE.into(),
                ],
                true,
                why.clone(),
            ),
            Cause::Keychain(why) => (
                "Kanso can't reach the keychain",
                format!("Kanso can't read this library's key from the system keychain: {why}"),
                vec![
                    "Unlock the keychain, and let Kanso use it if it asks. On Linux, run a Secret Service such as GNOME Keyring or KeePassXC.".into(),
                    "Choose Try again.".into(),
                ],
                true,
                why.clone(),
            ),
            Cause::NoEncryption => (
                "This Kanso can't open an encrypted library",
                "This build of Kanso has no encryption, but your library is encrypted. Nothing was changed.".into(),
                vec!["Install the official Kanso for this computer from the download page, then open it again.".into()],
                false,
                "build without encryption".into(),
            ),
            Cause::Io(why) => (
                "Kanso can't read your library",
                format!("A file in {folder} couldn't be read or written: {why}"),
                vec![
                    "Check that the disk the folder is on is connected, and that you are allowed to write there.".into(),
                    "Choose Try again.".into(),
                    ADVICE.into(),
                ],
                true,
                why.clone(),
            ),
            Cause::Other(why) => (
                "Kanso can't open your library",
                format!("Opening {} failed: {why}", file(data_dir, "kanso.db")),
                vec![
                    "Your notes were not changed. Quit Kanso and open it again.".into(),
                    ADVICE.into(),
                ],
                false,
                why.clone(),
            ),
        };
    screen(title, message, steps, retry, &technical, &folder)
}

fn screen(
    title: &str,
    message: String,
    steps: Vec<String>,
    retry: bool,
    technical: &str,
    folder: &str,
) -> Screen {
    let mut actions = vec![Action::ShowFolder, Action::CopyDetails, Action::Quit];
    if retry {
        actions.push(Action::TryAgain);
    }
    // The details hold what a person needs to report this and nothing else:
    // what happened and where, never a key.
    let details = format!(
        "{} {} on {}\nProblem: {title}\nData folder: {folder}\nDetails: {technical}",
        crate::flavour::CURRENT.name,
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
    );
    Screen {
        title: title.to_owned(),
        message,
        steps,
        actions,
        details,
        folder: folder.to_owned(),
    }
}

#[cfg(test)]
#[path = "startup_error_tests.rs"]
mod tests;
