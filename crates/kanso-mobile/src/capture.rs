//! Saving what was shared into Kanso from other apps: the drops the share
//! extension, the App Intents and the Android share activity leave in the
//! library's `inbox/` folder (`mobile/docs/capture.md` says what a drop is).
//!
//! [`save_inbox`] opens the library (or reuses the one the app has open, so
//! the notes appear in the app's lists as they are saved) and saves every
//! finished drop in order, oldest first, as a note in the Inbox through
//! `kanso_engine::capture`. The C entry that exports it,
//! `kanso_capture_inbox`, is in `mobile/rust`.
//!
//! A drop's folder is removed once its note is saved, and also when the drop
//! can never be saved: the Inbox is locked, nothing in it is usable, or its
//! `drop.json` can't be read. A drop that failed for a reason that may pass
//! (a busy database, a disk error), or that a newer Kanso wrote, is kept and
//! tried again the next time. Saving a drop twice makes one note, so a crash
//! between the note and the removal costs nothing.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use kanso_core::dates::{Date, DateTime};
use kanso_core::model::{MAX_ATTACHMENT_SIZE, MAX_PDF_SIZE};
use kanso_engine::attach::too_large_file;
use kanso_engine::capture::{
    CaptureError, MAX_SHARED_FILES, Shared, SharedFile, is_drop_id, save_shared,
};
use kanso_engine::capture_task::{TaskTarget, save_task_line};
use kanso_import::library::notes_html::{placeholder, share_markdown};
use kanso_store::{Origin, Store};
use serde_json::Value;

use crate::boundary::guard;
use crate::errors::KansoError;
use crate::library::Library;
use crate::model::{CaptureOutcome, DropsSaved, LocalTime, Share};
use crate::{Backend, Secrets};

/// The `drop.json` version this reads.
const VERSION: i64 = 1;
/// A folder still being written (`<id>.partial`) older than this is
/// abandoned and removed.
const PARTIAL_LIFE: Duration = Duration::from_secs(24 * 60 * 60);
/// The largest `drop.json` read, in bytes.
const MANIFEST_LIMIT: u64 = 8 * 1024 * 1024;
/// A file larger than this is refused unread: converting it would not fit
/// the attachment limit in any case. PDFs may be up to 50 MiB, as on the
/// desktop, so the bound is never below that.
const READ_LIMIT: u64 = if MAX_PDF_SIZE > 4 * MAX_ATTACHMENT_SIZE {
    MAX_PDF_SIZE as u64
} else {
    4 * MAX_ATTACHMENT_SIZE as u64
};
/// The largest `html` a drop carries, in bytes; more is read as plain text.
const HTML_LIMIT: usize = 4 * 1024 * 1024;
/// The name of a drop's description.
const MANIFEST: &str = "drop.json";

impl Library {
    /// Saves every finished drop in `inbox` (a folder of drops) as a note in
    /// the Inbox, oldest first, and removes the folders of those saved or
    /// refused for good. See the module's docs. A folder that isn't there
    /// has no drops.
    pub fn save_drops(&self, inbox: &Path) -> Result<DropsSaved, KansoError> {
        guard(|| {
            let saved = save_drops_at(&self.inner.store, inbox, SystemTime::now())?;
            if !saved.notes.is_empty() || !saved.tasks.is_empty() {
                self.inner
                    .metrics()
                    .count(crate::metrics::Feature::QuickCapture);
            }
            Ok(saved)
        })
    }

    /// Saves something shared from inside the app (photos, the camera, a
    /// PDF) as a note in the Inbox, with its images and PDFs attached, and
    /// returns the note's ID and what was left out. It is saved under a new
    /// random drop ID, so each call makes a note.
    ///
    /// A locked Inbox refuses ([`KansoError::Locked`]) and nothing is
    /// written. A share with nothing Kanso can save is
    /// [`KansoError::Invalid`], saying why.
    pub fn capture(&self, share: Share) -> Result<CaptureOutcome, KansoError> {
        guard(|| {
            let captured = capture_at(&self.inner.store, share, &new_drop_id())?;
            self.inner
                .metrics()
                .count(crate::metrics::Feature::QuickCapture);
            Ok(captured)
        })
    }

    /// Today's daily note, made in the Journal (from its daily template
    /// when there is one) if it isn't there, and its ID. `now` is the
    /// phone's local date and time.
    ///
    /// In a locked Journal the note is made without text from any locked
    /// template: nothing of a locked note is copied out.
    pub fn today_note(&self, now: LocalTime) -> Result<String, KansoError> {
        self.journal_today(now).map(|made| made.id)
    }
}

/// [`Library::capture`] with the drop ID given.
fn capture_at(store: &Store, share: Share, drop_id: &str) -> Result<CaptureOutcome, KansoError> {
    let shared = Shared {
        subject: share.subject,
        text: share.text,
        url: share.url,
        files: share
            .files
            .into_iter()
            .map(|file| SharedFile {
                name: file.name,
                content_type: file.content_type,
                bytes: file.bytes,
            })
            .collect(),
    };
    match save_shared(store, &shared, drop_id, Origin::Ui) {
        Ok(captured) => Ok(CaptureOutcome {
            note_id: captured.id,
            problems: captured.problems,
        }),
        Err(CaptureError::Locked) => Err(KansoError::Locked),
        Err(CaptureError::Empty(skipped)) => {
            let mut message = String::from("There was nothing Kanso could save.");
            for problem in skipped {
                message.push(' ');
                message.push_str(&problem);
            }
            Err(KansoError::invalid(message))
        }
        Err(CaptureError::BadId) => Err(KansoError::Internal {
            message: "the share's ID was not valid".into(),
        }),
        Err(CaptureError::Store(err)) => Err(err.into()),
    }
}

/// 32 random lowercase hex characters: a drop's ID.
pub(crate) fn new_drop_id() -> String {
    (0..32)
        .map(|_| char::from_digit(fastrand::u32(0..16), 16).unwrap_or('0'))
        .collect()
}

/// [`Library::save_drops`] for the library whose database is `library`,
/// opened if the process doesn't have it open already: the call the C entry
/// makes, with the secret store and the server client given as the
/// background sync takes them. A panic is caught and returned as
/// [`KansoError::Internal`].
pub fn save_inbox(
    library: &Path,
    inbox: &Path,
    secrets: Arc<dyn Secrets>,
    backend: Arc<dyn Backend>,
) -> Result<DropsSaved, KansoError> {
    let library = match Library::existing(library) {
        Some(library) => library,
        None => Library::open_with(library, secrets, backend)?,
    };
    library.save_drops(inbox)
}

impl DropsSaved {
    /// The JSON the C entry returns:
    /// `{"notes": [...], "tasks": [...], "problems": [...], "kept": 0}`.
    pub fn to_json(&self) -> String {
        // Written in this order whatever the JSON library's map keeps.
        let list = |items: &[String]| Value::from(items).to_string();
        format!(
            r#"{{"notes":{},"tasks":{},"problems":{},"kept":{}}}"#,
            list(&self.notes),
            list(&self.tasks),
            list(&self.problems),
            self.kept
        )
    }
}

/// A finished drop found in the inbox.
struct Found {
    id: String,
    folder: PathBuf,
    /// When the drop was made, for the order: its `createdAt`, else the
    /// folder's own time.
    created_ms: i64,
    manifest: Result<Value, Unreadable>,
}

/// Why a drop's description couldn't be used.
enum Unreadable {
    /// It will never be: missing, not JSON, not an object.
    ForGood,
    /// It couldn't be read now.
    ForNow(String),
    /// A newer Kanso wrote it.
    Newer,
}

/// What became of one drop.
enum Fate {
    /// Saved, or already saved: the note's ID, and what was left out.
    Saved(String, Vec<String>),
    /// A task drop's line, written into a note.
    Task(String),
    /// Never to be saved; say why.
    Refused(String),
    /// Try again later; say why.
    Kept(String),
}

/// [`Library::save_drops`] at time `now`.
fn save_drops_at(store: &Store, inbox: &Path, now: SystemTime) -> Result<DropsSaved, KansoError> {
    let mut saved = DropsSaved::default();
    let entries = match fs::read_dir(inbox) {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(saved),
        Err(err) => {
            return Err(KansoError::storage(format!(
                "The folder of shared items couldn't be read: {err}"
            )));
        }
    };
    let mut found: Vec<Found> = Vec::new();
    for entry in entries.flatten() {
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        let folder = entry.path();
        // A link is not a folder this app made.
        let Ok(meta) = fs::symlink_metadata(&folder) else {
            continue;
        };
        if !meta.is_dir() {
            continue;
        }
        if let Some(id) = name.strip_suffix(".partial") {
            let old = meta
                .modified()
                .ok()
                .and_then(|made| now.duration_since(made).ok())
                .is_some_and(|age| age > PARTIAL_LIFE);
            if is_drop_id(id) && old {
                if has_recording(&folder) {
                    // A voice recording cut short: never deleted, kept
                    // aside for the person to find.
                    crate::voice_capture::quarantine_partial(&dir_of_inbox(&folder), &name);
                } else {
                    remove(&folder);
                }
            }
            continue;
        }
        if !is_drop_id(&name) {
            continue;
        }
        let modified_ms = meta.modified().ok().map_or(0, millis);
        let manifest = read_manifest(&folder);
        // A recording waits for the app to transcribe it (`voice_capture`):
        // it is neither a share nor a failure, so it is left where it is.
        if manifest
            .as_ref()
            .is_ok_and(|json| json.get("kind").and_then(Value::as_str) == Some("voice"))
        {
            continue;
        }
        let created_ms = manifest
            .as_ref()
            .ok()
            .and_then(|json| json.get("createdAt")?.as_i64())
            .unwrap_or(modified_ms);
        found.push(Found {
            id: name,
            folder,
            created_ms,
            manifest,
        });
    }
    found.sort_by(|a, b| (a.created_ms, &a.id).cmp(&(b.created_ms, &b.id)));

    for drop in found {
        match save_one(store, &drop) {
            Fate::Saved(note, problems) => {
                remove(&drop.folder);
                saved.notes.push(note);
                saved.problems.extend(problems);
            }
            Fate::Task(line) => {
                remove(&drop.folder);
                saved.tasks.push(line);
            }
            Fate::Refused(problem) => {
                remove(&drop.folder);
                saved.problems.push(problem);
            }
            Fate::Kept(problem) => {
                saved.kept += 1;
                saved.problems.push(problem);
            }
        }
    }
    Ok(saved)
}

fn save_one(store: &Store, drop: &Found) -> Fate {
    let manifest = match &drop.manifest {
        Ok(manifest) => manifest,
        Err(Unreadable::ForGood) => {
            return Fate::Refused("A shared item couldn't be read and was discarded.".into());
        }
        Err(Unreadable::ForNow(err)) => {
            return Fate::Kept(format!(
                "A shared item couldn't be read yet ({err}). It will be tried again."
            ));
        }
        Err(Unreadable::Newer) => {
            return Fate::Kept(
                "A shared item was made by a newer Kanso and was kept: update Kanso to save it."
                    .into(),
            );
        }
    };
    if manifest.get("kind").and_then(Value::as_str) == Some(TASK_KIND) {
        return save_task_drop(store, manifest);
    }
    let (shared, mut problems) = match shared_from(manifest, &drop.folder) {
        Ok(read) => read,
        Err(err) => {
            return Fate::Kept(format!(
                "A shared item's files couldn't be read yet ({err}). It will be tried again."
            ));
        }
    };
    match save_shared(store, &shared, &drop.id, Origin::Ui) {
        Ok(captured) => {
            problems.extend(captured.problems);
            Fate::Saved(captured.id, problems)
        }
        Err(err) if err.is_final() => {
            let why = match &err {
                CaptureError::Locked => {
                    "The Inbox is locked, so a shared item wasn't saved and was discarded."
                        .to_owned()
                }
                CaptureError::Empty(skipped) => {
                    problems.extend(skipped.iter().cloned());
                    "A shared item had nothing Kanso could save and was discarded.".to_owned()
                }
                other => format!("A shared item couldn't be saved ({other}) and was discarded."),
            };
            problems.push(why);
            Fate::Refused(problems.join(" "))
        }
        Err(err) => Fate::Kept(format!(
            "A shared item couldn't be saved yet ({err}). It will be tried again."
        )),
    }
}

/// What a task drop's `drop.json` says it is.
const TASK_KIND: &str = "task";

/// What a task drop asks for: its line, written at its target in the daily
/// note of the day it was captured.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct TaskDrop {
    pub line: String,
    pub target: TaskTarget,
    pub at: DateTime,
}

/// Reads a task drop (`"kind": "task"`, see `mobile/docs/capture.md`):
/// `line`, the task line the parser wrote; `day`, the phone's local date
/// it was captured on (`YYYY-MM-DD`), with `time` (`HH:MM`) when given;
/// `target`, `dailyNote` (the default) or `inbox`. `None` when the line
/// isn't one task line or the day isn't a date. Pure.
pub(crate) fn task_drop(manifest: &Value) -> Option<TaskDrop> {
    let text = |key: &str| manifest.get(key).and_then(Value::as_str);
    let line = kanso_engine::capture_task::checked_task_line(text("line")?)?.to_owned();
    let date = Date::parse_iso(text("day")?)?;
    let (hour, minute) = match text("time") {
        None => (0, 0),
        Some(time) => {
            let (hour, minute) = time.split_once(':')?;
            let (hour, minute) = (hour.parse::<u32>().ok()?, minute.parse::<u32>().ok()?);
            if hour > 23 || minute > 59 {
                return None;
            }
            (hour, minute)
        }
    };
    let target = match text("target") {
        None | Some("dailyNote") => TaskTarget::DailyNote,
        Some("inbox") => TaskTarget::Inbox,
        Some(_) => return None,
    };
    Some(TaskDrop {
        line,
        target,
        at: DateTime {
            date,
            hour,
            minute,
            second: 0,
        },
    })
}

/// Writes a task drop's line. A drop that can't be read, or whose place is
/// locked, is refused and the task's words are said, so the person can add
/// it by hand; a failure that may pass keeps it.
///
/// Unlike a share, a task written twice is two lines: a crash between the
/// write and the folder's removal leaves a duplicate the person can see and
/// delete, never a lost task.
fn save_task_drop(store: &Store, manifest: &Value) -> Fate {
    let Some(task) = task_drop(manifest) else {
        return Fate::Refused("A task from Shortcuts couldn't be read and was discarded.".into());
    };
    match save_task_line(store, &task.line, task.target, task.at, Origin::Ui) {
        Ok(_) => Fate::Task(task.line),
        Err(CaptureError::Locked) => {
            let place = match task.target {
                TaskTarget::DailyNote => "The Journal",
                TaskTarget::Inbox => "The Inbox",
            };
            let words = task.line.trim_start_matches("- [ ] ");
            Fate::Refused(format!(
                "{place} is locked, so a task from Shortcuts wasn't added: “{words}”."
            ))
        }
        Err(err) if err.is_final() => Fate::Refused(format!(
            "A task from Shortcuts couldn't be added ({err}) and was discarded."
        )),
        Err(err) => Fate::Kept(format!(
            "A task from Shortcuts couldn't be added yet ({err}). It will be tried again."
        )),
    }
}

/// Reads `drop.json`.
fn read_manifest(folder: &Path) -> Result<Value, Unreadable> {
    let path = folder.join(MANIFEST);
    let meta = match fs::symlink_metadata(&path) {
        Ok(meta) => meta,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Err(Unreadable::ForGood),
        Err(err) => return Err(Unreadable::ForNow(err.to_string())),
    };
    if !meta.is_file() || meta.len() > MANIFEST_LIMIT {
        return Err(Unreadable::ForGood);
    }
    let bytes = fs::read(&path).map_err(|err| Unreadable::ForNow(err.to_string()))?;
    let json: Value = serde_json::from_slice(&bytes).map_err(|_| Unreadable::ForGood)?;
    if !json.is_object() {
        return Err(Unreadable::ForGood);
    }
    match json.get("version").and_then(Value::as_i64) {
        Some(VERSION) => Ok(json),
        Some(version) if version > VERSION => Err(Unreadable::Newer),
        _ => Err(Unreadable::ForGood),
    }
}

/// The share a `drop.json` describes, with its files read, and what was said
/// about the files left out. An error is a read that failed and may work
/// later.
fn shared_from(manifest: &Value, folder: &Path) -> Result<(Shared, Vec<String>), String> {
    let text = |key: &str| manifest.get(key).and_then(Value::as_str).map(str::to_owned);
    let subject = text("subject");
    // A note shared from Apple Notes brings its HTML (or RTF as HTML):
    // checklists become tasks and the title is not said twice.
    let html = text("html").filter(|html| html.len() <= HTML_LIMIT);
    let mut shared = Shared {
        text: if html.is_some() {
            share_markdown(subject.as_deref(), html.as_deref(), text("text").as_deref())
        } else {
            text("text")
        },
        subject,
        url: text("url"),
        files: Vec::new(),
    };
    let mut problems = Vec::new();
    let listed = manifest.get("files").and_then(Value::as_array);
    // Past the limit the engine names the files left out; reading them would
    // only fill the phone's memory.
    for entry in listed.into_iter().flatten().take(MAX_SHARED_FILES) {
        let Some(path) = entry.get("path").and_then(Value::as_str) else {
            problems.push("A shared file had no path and was left out.".into());
            continue;
        };
        let name = entry
            .get("name")
            .and_then(Value::as_str)
            .filter(|name| !name.trim().is_empty())
            .unwrap_or(path);
        if !is_plain_name(path) {
            problems.push(format!("{name}: its path isn't a file in the shared item."));
            continue;
        }
        let file = folder.join(path);
        let meta = match fs::symlink_metadata(&file) {
            Ok(meta) => meta,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                problems.push(format!("{name}: the file is missing."));
                continue;
            }
            Err(err) => return Err(err.to_string()),
        };
        if !meta.is_file() {
            problems.push(format!("{name}: its path isn't a file in the shared item."));
            continue;
        }
        if meta.len() > READ_LIMIT {
            problems.push(format!(
                "{name}: {}",
                too_large_file(
                    "file",
                    meta.len() as usize,
                    if name.to_ascii_lowercase().ends_with(".pdf") {
                        MAX_PDF_SIZE
                    } else {
                        MAX_ATTACHMENT_SIZE
                    }
                )
            ));
            continue;
        }
        let bytes = fs::read(&file).map_err(|err| err.to_string())?;
        shared.files.push(SharedFile {
            name: name.to_owned(),
            content_type: entry.get("type").and_then(Value::as_str).map(str::to_owned),
            bytes,
        });
    }
    // A titled note that is only a drawing or a scan says which. Apple
    // Notes is iOS only; on Android many apps title a plain photo share
    // (EXTRA_SUBJECT), which must not read as a drawing.
    if cfg!(not(target_os = "android"))
        && shared.subject.is_some()
        && shared.text.is_none()
        && shared.url.is_none()
        && let Some(first) = shared.files.first()
    {
        let pdf = first.content_type.as_deref() == Some("application/pdf")
            || first.name.to_ascii_lowercase().ends_with(".pdf");
        shared.text = Some(placeholder(pdf).to_owned());
    }
    if listed.is_some_and(|files| files.len() > MAX_SHARED_FILES) {
        problems.push(format!(
            "A share can have up to {MAX_SHARED_FILES} files: the last {} weren't attached.",
            listed.map_or(0, Vec::len) - MAX_SHARED_FILES
        ));
    }
    Ok((shared, problems))
}

/// Whether `path` names a file directly inside the drop's folder: not empty,
/// no separator, not `.` or `..`, not a drive or an absolute path, and not
/// the description itself.
pub(crate) fn is_plain_name(path: &str) -> bool {
    !path.is_empty()
        && path != "."
        && path != ".."
        && path != MANIFEST
        && !path.contains(['/', '\\', '\0', ':'])
        && Path::new(path).components().count() == 1
        && Path::new(path).is_relative()
}

/// Removes a drop's folder. Left behind, it is saved again, to the same
/// note, and removed then.
fn remove(folder: &Path) {
    if let Err(err) = fs::remove_dir_all(folder) {
        eprintln!(
            "kanso: couldn't remove the shared item {}: {err}",
            folder.display()
        );
    }
}

fn millis(time: SystemTime) -> i64 {
    time.duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, |since| since.as_millis() as i64)
}

/// Whether `folder` holds a recording (`rec.*`) that is not empty.
fn has_recording(folder: &Path) -> bool {
    fs::read_dir(folder).is_ok_and(|entries| {
        entries.flatten().any(|entry| {
            entry.file_name().to_string_lossy().starts_with("rec.")
                && entry.metadata().is_ok_and(|m| m.len() > 0)
        })
    })
}

fn dir_of_inbox(folder: &Path) -> std::path::PathBuf {
    folder.parent().map(Path::to_path_buf).unwrap_or_default()
}
