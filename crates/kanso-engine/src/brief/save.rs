//! "Save as Note": a weekly review kept as an ordinary note in the Journal.

use kanso_core::DocType;
use kanso_core::dates::Date;
use kanso_core::roles::Role;
use kanso_store::{Origin, Put, Result, Store};
use serde_json::json;

use super::{Review, iso_week_label, review_markdown, review_title};

/// What saving a review did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Saved {
    /// The note was written (made, or replaced).
    Written { id: String },
    /// A review for that week is saved already, and replacing it wasn't
    /// asked for: the caller asks "Replace the saved review for 2026-W45?"
    /// and saves again with `replace`.
    Exists { id: String },
}

/// The ID a week's saved review has: one note per ISO week, so saving the
/// same week again is a replacement, not a second note.
pub fn review_note_id(from: Date) -> String {
    format!("note:weekly-review-{}", iso_week_label(from).to_lowercase())
}

/// Whether a review is saved for the week starting `from` (and not in the
/// trash).
pub fn saved_review(store: &Store, from: Date) -> Result<Option<String>> {
    let id = review_note_id(from);
    Ok(store
        .note(&id)?
        .filter(|note| !note.is_trashed())
        .map(|_| id))
}

/// Writes `review` into the Journal as "Weekly review — 2026-W45".
pub fn save_review(store: &Store, review: &Review, replace: bool, origin: Origin) -> Result<Saved> {
    let id = review_note_id(review.from);
    let existing = store.note(&id)?;
    if let Some(note) = &existing
        && !note.is_trashed()
        && !replace
    {
        return Ok(Saved::Exists { id });
    }
    let book = store.role_book(Role::Journal, origin)?;
    let mut doc = json!({
        "_id": id,
        "title": review_title(review.from),
        "body": review_markdown(review),
        "bookId": book,
    });
    if let Some(note) = existing {
        doc["_rev"] = json!(note.rev);
    }
    let written = store.put(DocType::Note, doc, Put::Upsert, origin)?;
    Ok(Saved::Written { id: written.id })
}
