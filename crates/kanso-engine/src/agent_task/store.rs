//! Agent sessions on disk (issue #375): one file per session in
//! `<data dir>/agent-sessions/<session key>.json`, so a session and the
//! proposal nobody reviewed survive a restart. No window; the runner calls
//! these.
//!
//! A file is written to a temporary name and renamed over the old one, so a
//! crash leaves the last whole file. A file that can't be read, or has
//! another `version`, is skipped, never deleted. A session over
//! [`MAX_BYTES`] loses its oldest thoughts, then its oldest tool calls
//! ([`TRIMMED_NOTICE`]); the unreviewed proposal is never trimmed.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde_json::Value;

use super::session::{Item, SavedSession, SessionKey};

/// The file format [`save`] writes and [`load_all`] accepts.
pub const VERSION: u32 = 1;
/// The most bytes one session's file may take (items are trimmed to fit).
pub const MAX_BYTES: usize = 2 << 20;
/// What the transcript says after items were dropped.
pub const TRIMMED_NOTICE: &str = "Older tool details were trimmed";

/// The folder sessions live in, inside the app's data folder.
pub fn dir(data_dir: &Path) -> PathBuf {
    data_dir.join("agent-sessions")
}

/// A key is the file's name, so it must be plain letters and digits.
fn valid_key(key: &SessionKey) -> bool {
    !key.0.is_empty() && key.0.len() <= 64 && key.0.chars().all(|c| c.is_ascii_alphanumeric())
}

fn file(dir: &Path, key: &SessionKey) -> PathBuf {
    dir.join(format!("{}.json", key.0))
}

fn item_len(item: &Item) -> usize {
    serde_json::to_vec(item).map_or(0, |bytes| bytes.len() + 1)
}

fn is_thought(item: &Item) -> bool {
    matches!(item, Item::Thought { .. })
}

fn is_tool(item: &Item) -> bool {
    matches!(item, Item::Tool { .. })
}

/// `session` with items dropped until it fits [`MAX_BYTES`]: the oldest
/// thoughts first, then the oldest tool calls, and a notice saying so.
pub fn trimmed(session: &SavedSession) -> SavedSession {
    let mut session = session.clone();
    let Ok(whole) = serde_json::to_vec(&session) else {
        return session;
    };
    if whole.len() <= MAX_BYTES {
        return session;
    }
    let notice = Item::Notice {
        text: TRIMMED_NOTICE.to_owned(),
        at_ms: session.last_activity_ms,
    };
    let mut excess = (whole.len() + item_len(&notice)).saturating_sub(MAX_BYTES);
    let mut dropped = vec![false; session.items.len()];
    for wanted in [is_thought as fn(&Item) -> bool, is_tool] {
        for (ix, item) in session.items.iter().enumerate() {
            if excess == 0 {
                break;
            }
            if wanted(item) && !dropped[ix] {
                dropped[ix] = true;
                excess = excess.saturating_sub(item_len(item));
            }
        }
    }
    if !dropped.iter().any(|d| *d) {
        return session;
    }
    let mut flags = dropped.into_iter();
    session.items.retain(|_| !flags.next().unwrap_or(false));
    let already = session
        .items
        .iter()
        .any(|item| matches!(item, Item::Notice { text, .. } if text == TRIMMED_NOTICE));
    if !already {
        // After the opening goal, so it reads where the old details were.
        let at = usize::from(matches!(session.items.first(), Some(Item::User { .. })));
        session.items.insert(at, notice);
    }
    session
}

/// Writes `session` to `dir`, atomically, trimmed to [`MAX_BYTES`].
pub fn save(dir: &Path, session: &SavedSession) -> io::Result<()> {
    if !valid_key(&session.key) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "a session key is letters and digits",
        ));
    }
    let mut session = trimmed(session);
    session.version = VERSION;
    let bytes = serde_json::to_vec(&session).map_err(io::Error::other)?;
    fs::create_dir_all(dir)?;
    let target = file(dir, &session.key);
    let temp = dir.join(format!(".{}.tmp", session.key.0));
    fs::write(&temp, bytes)?;
    fs::rename(&temp, &target).inspect_err(|_| {
        let _ = fs::remove_file(&temp);
    })
}

/// Every session in `dir` that can be read, oldest first. Files that are
/// corrupt, of another version, or named for another key are skipped.
pub fn load_all(dir: &Path) -> Vec<SavedSession> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut all: Vec<SavedSession> = entries
        .flatten()
        .filter_map(|entry| read(&entry.path()))
        .collect();
    all.sort_by(|a, b| (a.created_ms, &a.key).cmp(&(b.created_ms, &b.key)));
    all
}

fn read(path: &Path) -> Option<SavedSession> {
    if path.extension()?.to_str()? != "json" {
        return None;
    }
    let value: Value = serde_json::from_slice(&fs::read(path).ok()?).ok()?;
    if value.get("version")?.as_u64()? != u64::from(VERSION) {
        return None;
    }
    let session: SavedSession = serde_json::from_value(value).ok()?;
    let named = path.file_stem()?.to_str()?;
    (valid_key(&session.key) && session.key.0 == named).then_some(session)
}

/// Removes the file of session `key`. A file that isn't there is fine.
pub fn delete(dir: &Path, key: &SessionKey) -> io::Result<()> {
    if !valid_key(key) {
        return Ok(());
    }
    match fs::remove_file(file(dir, key)) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e),
        _ => Ok(()),
    }
}

/// How many session files `dir` holds.
pub fn count(dir: &Path) -> usize {
    fs::read_dir(dir).map_or(0, |entries| {
        entries
            .flatten()
            .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "json"))
            .count()
    })
}

/// Removes every session file; returns how many went.
pub fn delete_all(dir: &Path) -> usize {
    let Ok(entries) = fs::read_dir(dir) else {
        return 0;
    };
    entries
        .flatten()
        .filter(|entry| {
            let path = entry.path();
            let ours = path.extension().is_some_and(|ext| ext == "json")
                || entry.file_name().to_string_lossy().ends_with(".tmp");
            ours && fs::remove_file(&path).is_ok()
        })
        .count()
}
