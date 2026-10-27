//! The note actions of [`Library`] beyond editing a note's text: changing
//! several notes at once (with the desktop's notice and Undo), restoring
//! and deleting for good, emptying the trash, duplicating a note and copying
//! its link. Each matches what the desktop does for the same command.

use std::collections::HashSet;
use std::time::Instant;

use kanso_core::model::TRASH_BOOK_ID;
use kanso_core::{DocType, Note, NoteStatus, new_id, now_ms};
use kanso_engine::batch::{self, Change, Edit};
use kanso_engine::links::{copy_message, link_for};
use kanso_engine::list::{self, Filter};
use kanso_engine::notes::copy_title;
use kanso_store::{NoteSummary, Origin, Put, Sort};
use serde_json::json;

use crate::boundary::{guard, held};
use crate::errors::KansoError;
use crate::library::Library;
use crate::model::{CopiedLink, NoteChange, Saved, StatusChoice};
use crate::notes::parse_status;

/// What a change did, before it is told to the shell.
pub(crate) struct Applied {
    pub(crate) count: u32,
    pub(crate) edits: Vec<Edit>,
    pub(crate) notice: String,
}

impl Library {
    /// The statuses a note can be set to, in the desktop's order, with the
    /// labels it shows.
    pub fn statuses() -> Vec<StatusChoice> {
        NoteStatus::ALL
            .into_iter()
            .map(|status| StatusChoice {
                status: status.as_str().to_owned(),
                label: status.label().to_owned(),
            })
            .collect()
    }

    /// Deletes every note in the trash for good, locked ones too, and says
    /// how many.
    pub fn empty_trash(&self) -> Result<u32, KansoError> {
        guard(|| {
            let store = &self.inner.store;
            let params = list::search_params(&Filter::Trash, "", Sort::UpdatedAt, true);
            let notes = store.search_summaries(&params)?;
            for note in &notes {
                store.delete(&note.id, Origin::Ui)?;
            }
            Ok(notes.len() as u32)
        })
    }

    /// Copies note `id` as the desktop does: "`title` (copy)", the same text,
    /// notebook, tags and status. A locked note needs its lock open
    /// ([`KansoError::Locked`] otherwise), and its copy is locked too, sealed
    /// for its own ID; so is the copy of a note in a locked notebook.
    pub fn duplicate(&self, id: &str) -> Result<Saved, KansoError> {
        guard(|| {
            let store = &self.inner.store;
            let mut note = self.stored(id)?;
            if self.reveal(&mut note).is_closed() {
                return Err(KansoError::Locked);
            }
            let title = copy_title(&note.title);
            if store.locked_books()?.contains(&note.book_id) {
                // As `create` makes a note in a locked notebook.
                let now = now_ms();
                let mut copy: Note = serde_json::from_value(json!({
                    "_id": new_id(DocType::Note),
                    "title": title,
                    "body": note.body,
                    "bookId": note.book_id,
                    "tags": note.tags,
                    "status": note.status.as_str(),
                    "createdAt": now,
                    "updatedAt": now,
                }))
                .map_err(|err| KansoError::invalid(err.to_string()))?;
                let stored = held(&self.inner.keys)
                    .seal_for_notebook(&mut copy, store)
                    .map_err(|_| KansoError::Locked)?;
                let value = serde_json::to_value(&stored)
                    .map_err(|err| KansoError::storage(err.to_string()))?;
                let written = store.put(DocType::Note, value, Put::Upsert, Origin::Ui)?;
                return Ok(Saved {
                    id: written.id,
                    rev: written.rev,
                });
            }
            let sealed = note.is_locked();
            let created = store.put(
                DocType::Note,
                json!({
                    "title": title,
                    "body": if sealed { "" } else { note.body.as_str() },
                    "bookId": note.book_id,
                    "tags": note.tags,
                    "status": note.status.as_str(),
                }),
                Put::Upsert,
                Origin::Ui,
            )?;
            if !sealed {
                return Ok(Saved {
                    id: created.id,
                    rev: created.rev,
                });
            }
            // A locked note's copy is locked too, sealed for its own ID.
            let locked = self.lock_copy(&created.id, note.body);
            match locked {
                Ok(written) => Ok(Saved {
                    id: written.id,
                    rev: written.rev,
                }),
                Err(err) => {
                    // No plain, empty copy is left behind.
                    let _ = store.delete(&created.id, Origin::Ui);
                    Err(err)
                }
            }
        })
    }

    /// The link to note `id` to paste into another note, and what to tell the
    /// person: its wiki-link (`[[Title]]`), or with `kanso` Kanso's own link
    /// by ID, which also stands in when the title can't be a wiki-link or an
    /// older note has it. A locked note's title is no secret, so it works
    /// closed or open.
    pub fn note_link(&self, id: &str, kanso: bool) -> Result<CopiedLink, KansoError> {
        guard(|| {
            let note = self.stored(id)?;
            let resolves_to = self
                .inner
                .store
                .resolve_title(&note.title)
                .ok()
                .flatten()
                .map(|(other, _)| other);
            let (text, instead) = link_for(id, &note.title, kanso, resolves_to.as_deref());
            Ok(CopiedLink {
                text,
                message: copy_message(&note.title, instead).to_owned(),
            })
        })
    }

    /// Seals `body` into the new note `id`, a locked note's copy.
    fn lock_copy(&self, id: &str, body: String) -> Result<kanso_store::WriteResult, KansoError> {
        let mut copy = self.stored(id)?;
        copy.body = body;
        held(&self.inner.keys)
            .seal(&mut copy, Instant::now())
            .map_err(|_| KansoError::Locked)?;
        let value =
            serde_json::to_value(&copy).map_err(|err| KansoError::storage(err.to_string()))?;
        Ok(self.inner.store.lock_note(value)?)
    }

    /// Plans and writes `change`, and words the notice for it.
    pub(crate) fn apply(
        &self,
        ids: Vec<String>,
        change: NoteChange,
    ) -> Result<Applied, KansoError> {
        let store = &self.inner.store;
        let mut seen = HashSet::new();
        let ids: Vec<String> = ids
            .into_iter()
            .filter(|id| seen.insert(id.clone()))
            .collect();
        let locked_books = store.locked_books()?;
        let mut notes = Vec::with_capacity(ids.len());
        let mut in_locked_book = HashSet::new();
        for id in &ids {
            let note = self.stored(id)?;
            let sealed = store.note_is_locked(id)?;
            if locked_books.contains(&note.book_id) {
                in_locked_book.insert(note.id.clone());
            }
            notes.push(NoteSummary {
                locked: sealed || locked_books.contains(&note.book_id),
                id: note.id,
                rev: note.rev.unwrap_or_default(),
                title: note.title,
                snippet: String::new(),
                book_id: note.book_id,
                status: note.status,
                tags: note.tags,
                pinned: note.pinned,
                updated_at: note.updated_at,
                created_at: note.created_at,
                tasks_done: 0,
                tasks_total: 0,
            });
        }
        let books = store.books()?;
        let book_name = |id: &str| books.iter().find(|b| b.id == id).map(|b| b.name.clone());
        let tag_name = |id: &str| -> Result<String, KansoError> {
            store
                .tags()?
                .into_iter()
                .find(|tag| tag.id == id)
                .map(|tag| tag.name)
                .ok_or(KansoError::NotFound)
        };
        let mut place = String::new();
        let planned = match &change {
            NoteChange::TogglePin => Change::TogglePin,
            NoteChange::Trash => Change::Move(TRASH_BOOK_ID.to_owned()),
            NoteChange::Move { notebook_id } => {
                if notebook_id != TRASH_BOOK_ID && book_name(notebook_id).is_none() {
                    return Err(KansoError::NotFound);
                }
                place = book_name(notebook_id).unwrap_or_default();
                Change::Move(notebook_id.clone())
            }
            NoteChange::SetStatus { status } => {
                let status = parse_status(status)?;
                place = status.label().to_owned();
                notes.retain(|note| !note.locked);
                Change::Status(status)
            }
            NoteChange::AddTag { tag_id } | NoteChange::RemoveTag { tag_id } => {
                place = tag_name(tag_id)?;
                notes.retain(|note| !note.locked);
                if matches!(change, NoteChange::AddTag { .. }) {
                    Change::AddTag(tag_id.clone())
                } else {
                    Change::RemoveTag(tag_id.clone())
                }
            }
            NoteChange::Restore => {
                let first = books.first().ok_or_else(|| {
                    KansoError::invalid("Create a notebook to restore notes into.")
                })?;
                place = first.name.clone();
                notes.retain(|note| note.book_id == TRASH_BOOK_ID);
                Change::Move(first.id.clone())
            }
            NoteChange::DeleteForever => {
                if notes.iter().any(|note| note.book_id != TRASH_BOOK_ID) {
                    return Err(KansoError::invalid(
                        "Only notes in the trash can be deleted permanently.",
                    ));
                }
                for note in &notes {
                    store.delete(&note.id, Origin::Ui)?;
                }
                let count = notes.len();
                return Ok(Applied {
                    count: count as u32,
                    edits: Vec::new(),
                    notice: if count == 0 {
                        String::new()
                    } else {
                        deleted_notice(count)
                    },
                });
            }
        };
        if batch::needs_one_by_one(&notes, &planned, &locked_books) {
            return Err(KansoError::Locked);
        }
        let edits = batch::plan(&notes, &planned);
        if let Change::Move(book) = &planned
            && book != TRASH_BOOK_ID
            && edits.iter().any(|edit| in_locked_book.contains(&edit.id))
        {
            return Err(KansoError::Locked);
        }
        if edits.is_empty() {
            return Ok(Applied {
                count: 0,
                edits,
                notice: String::new(),
            });
        }
        let on = batch::turned_on(&planned, &edits);
        let changes = edits.iter().map(|edit| edit.change.clone()).collect();
        store.merge_many(DocType::Note, changes, Origin::Ui)?;
        Ok(Applied {
            count: edits.len() as u32,
            notice: batch::summary(&planned, edits.len(), &place, on),
            edits,
        })
    }
}

/// The notice for deleting for good, which is not a batch change.
fn deleted_notice(count: usize) -> String {
    let notes = if count == 1 {
        "1 note".to_owned()
    } else {
        format!("{count} notes")
    };
    format!("Deleted {notes} permanently.")
}
