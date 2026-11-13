//! The library as the phone's system actions see it: the notes iOS's "Open
//! Kanso note" App Intent offers (#390).
//!
//! An intent runs outside the app's screens, so it is shown nothing a lock
//! hides: a locked note, or any note in a locked notebook, is never offered,
//! open or not, and neither is a note in the trash. The intent itself only
//! opens `kanso://open/note/<id>`; the app does the rest behind its own
//! locks.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use kanso_core::due_phrase_model::DateOrder;
use kanso_engine::capture_task::task_preview;
use serde_json::{Value, json};

use crate::boundary::guard;
use crate::errors::KansoError;
use crate::intents_model::IntentNote;
use crate::journal::time_of;
use crate::library::Library;
use crate::model::{ListQuery, LocalTime, NoteRow};
use crate::{Backend, Secrets};

/// The most notes one ask returns.
pub const MAX_INTENT_NOTES: usize = 50;

impl Library {
    /// The notes "Open Kanso note" offers for `query`, newest edit first:
    /// every word of `query` is in the title, ignoring case (an empty query
    /// matches every note). At most `limit` (and never more than
    /// [`MAX_INTENT_NOTES`]). No locked note, no note in a locked notebook.
    pub fn intent_notes(&self, query: &str, limit: usize) -> Result<Vec<IntentNote>, KansoError> {
        guard(|| {
            let rows = self.list(ListQuery::default())?.rows;
            let books: HashMap<String, String> = self
                .notebooks()?
                .into_iter()
                .map(|book| (book.id, book.name))
                .collect();
            Ok(matching_notes(&rows, &books, query, limit))
        })
    }
}

/// The notes of `rows` (in their order) that may be offered and match
/// `query`, at most `limit`. Pure.
pub fn matching_notes(
    rows: &[NoteRow],
    books: &HashMap<String, String>,
    query: &str,
    limit: usize,
) -> Vec<IntentNote> {
    let words: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();
    rows.iter()
        .filter(|row| !row.locked)
        .filter(|row| {
            let title = row.title.to_lowercase();
            words.iter().all(|word| title.contains(word.as_str()))
        })
        .take(limit.min(MAX_INTENT_NOTES))
        .map(|row| IntentNote {
            id: row.id.clone(),
            title: row.title.clone(),
            notebook: books.get(&row.notebook_id).cloned().unwrap_or_default(),
        })
        .collect()
}

/// [`Library::intent_notes`] for the library whose database is `library`,
/// opened if the process doesn't have it open already, as
/// [`crate::capture::save_inbox`] does.
pub fn intent_notes_at(
    library: &Path,
    query: &str,
    limit: usize,
    secrets: Arc<dyn Secrets>,
    backend: Arc<dyn Backend>,
) -> Result<Vec<IntentNote>, KansoError> {
    let library = match Library::existing(library) {
        Some(library) => library,
        None => Library::open_with(library, secrets, backend)?,
    };
    library.intent_notes(query, limit)
}

/// "Add Kanso task"'s reading of `text` at the phone's local time `now`,
/// as JSON for the C entry: `{"line": …, "words": …, "when": … or null,
/// "confirmation": "Added: …"}`, or `null` when there is no task in it or
/// `now` isn't a time. `month_first` reads `11/14` as November 14th. The
/// parser is the desktop's and Raycast's
/// ([`kanso_engine::capture_task::task_preview`]).
pub fn task_preview_json(text: &str, now: LocalTime, month_first: bool) -> String {
    let order = if month_first {
        DateOrder::MonthDay
    } else {
        DateOrder::DayMonth
    };
    let Ok(now) = time_of(now) else {
        return Value::Null.to_string();
    };
    match task_preview(text, now, order) {
        None => Value::Null.to_string(),
        Some(preview) => json!({
            "line": preview.line,
            "words": preview.words,
            "when": preview.when,
            "confirmation": preview.confirmation(),
        })
        .to_string(),
    }
}

/// The JSON the C entry returns:
/// `{"notes": [{"id": …, "title": …, "notebook": …}]}`.
pub fn intent_notes_json(notes: &[IntentNote]) -> String {
    let notes: Vec<Value> = notes
        .iter()
        .map(|note| json!({"id": note.id, "title": note.title, "notebook": note.notebook}))
        .collect();
    json!({ "notes": notes }).to_string()
}
