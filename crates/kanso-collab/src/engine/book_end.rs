//! A shared notebook ending for an account: it left, was removed, or the
//! owner stopped sharing (issue #390 › Roles › Removing someone). Whichever
//! way, what the library holds stays: the notes become ordinary private
//! notes, and a notebook that isn't the owner's is renamed "<name> (no
//! longer shared)". The notebook's keys, logs and unsent edits go; its row
//! stays with the chain this device trusted, so joining again later is held
//! to that history.

use kanso_core::DocType;
use kanso_store::{Origin, Put, SharedStatus, SharedText};
use serde_json::json;

use super::Engine;
use super::book::unpinned_key;
use crate::api::{CollabError, Result};
use crate::book_copy::{copy_name, ended_notice, owner_notice};
use crate::book_model::{BookEndReason, BookEnded};

/// How a private copy is named, to find a name already given.
const COPY_SUFFIX: &str = " (no longer shared)";

impl Engine {
    /// Leaves a notebook someone else shared, keeping a private copy
    /// ([`crate::book_copy::leave_title`] asks first). The owner stops
    /// sharing instead.
    pub fn leave_book(&self, share_id: &str) -> Result<BookEnded> {
        let row = self.active_book(share_id)?;
        if row.owner_id == self.account.caller.account_id {
            return Err(CollabError::Refused(
                "The owner can't leave a notebook: stop sharing it instead.".into(),
            ));
        }
        // What came meanwhile is in the copy; offline, the copy is what
        // this device has.
        let _ = self.sync_book(share_id);
        match self.api.leave_book(&self.account.caller, share_id) {
            Ok(()) | Err(CollabError::Api { status: 404, .. }) => {}
            Err(err) => return Err(err),
        }
        self.end_book(share_id, BookEndReason::Left)
    }

    /// Stops sharing a notebook this account owns. Everyone keeps a private
    /// copy of its notes as they are now, and the share's id is retired.
    pub fn stop_book(&self, share_id: &str) -> Result<BookEnded> {
        let row = self.active_book(share_id)?;
        if row.owner_id != self.account.caller.account_id {
            return Err(CollabError::Refused(
                "Only the notebook's owner can stop sharing it.".into(),
            ));
        }
        // Whatever this device wrote reaches the others before it ends.
        let _ = self.sync_book(share_id);
        match self.api.stop_book(&self.account.caller, share_id) {
            Ok(()) | Err(CollabError::Api { status: 404, .. }) => {}
            Err(err) => return Err(err),
        }
        self.end_book(share_id, BookEndReason::Stopped)
    }

    fn active_book(&self, share_id: &str) -> Result<kanso_store::SharedBook> {
        self.store
            .shared_book(share_id)?
            .filter(|row| row.status == SharedStatus::Active)
            .ok_or_else(|| CollabError::Refused("The notebook isn't shared here.".into()))
    }

    /// The notebook stops being shared here: its notes become private, the
    /// notebook the owner's own or "<name> (no longer shared)", and what
    /// the device kept of it for syncing goes.
    pub(crate) fn end_book(&self, share_id: &str, reason: BookEndReason) -> Result<BookEnded> {
        let mut row = self
            .store
            .shared_book(share_id)?
            .ok_or_else(|| CollabError::Refused("The notebook isn't shared here.".into()))?;
        let mine = row.owner_id == self.account.caller.account_id;
        let book = self
            .store
            .books()?
            .into_iter()
            .find(|b| b.id == row.book_id);
        let shown = book.as_ref().map_or_else(
            || "Shared notebook".to_owned(),
            |b| {
                b.name
                    .strip_suffix(COPY_SUFFIX)
                    .unwrap_or(&b.name)
                    .to_owned()
            },
        );
        // Every note the notebook's logs name, and any that carry its mark.
        let mut notes: Vec<String> = self
            .store
            .book_streams(share_id)?
            .into_iter()
            .filter_map(|s| s.note_id)
            .collect();
        notes.extend(
            self.store
                .notes_marked_for_book(share_id)?
                .into_iter()
                .map(|(_, note, _)| note),
        );
        notes.sort();
        notes.dedup();
        let mut kept = 0;
        for id in notes {
            let Some(note) = self.store.note(&id)? else {
                continue;
            };
            self.store.write_shared_text(&SharedText {
                note_id: &note.id,
                title: &note.title,
                body: &note.body,
                mark: None,
                new_in: &note.book_id,
                expected_rev: None,
                authors: &[],
            })?;
            if note.book_id == row.book_id {
                kept += 1;
            }
        }
        let name = copy_name(&shown);
        if !mine
            && let Some(book) = &book
            && !book.name.ends_with(COPY_SUFFIX)
        {
            self.store.put(
                DocType::Book,
                json!({"_id": row.book_id, "name": name}),
                Put::Merge,
                Origin::Collab,
            )?;
        }
        row.status = match reason {
            BookEndReason::Stopped => SharedStatus::Stopped,
            _ => SharedStatus::Removed,
        };
        row.uploading = false;
        row.updated_at = kanso_core::now_ms();
        self.store.put_shared_book(&row)?;
        self.store.clear_shared_book_state(share_id)?;
        self.store.remove_meta(&unpinned_key(share_id))?;
        self.store
            .remove_meta(&super::book_roster::edits_key(share_id))?;
        Ok(BookEnded {
            share_id: share_id.to_owned(),
            reason,
            name: shown.clone(),
            book_id: row.book_id.clone(),
            copy_name: if mine { shown.clone() } else { name },
            notes: kept,
            message: if mine {
                owner_notice(&shown)
            } else {
                ended_notice(reason, &shown)
            },
        })
    }
}
