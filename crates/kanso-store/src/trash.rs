//! Empty Trash (issue #44). Every note in the trash goes for good, locked
//! ones too, and so does each attachment that only trashed notes own, sealed
//! copies included: none is left behind as unreadable ciphertext. An
//! attachment a note outside the trash still owns is kept.

use std::collections::BTreeSet;

use kanso_core::DocType;
use rusqlite::Transaction;

use crate::trash_model::{Emptied, TrashCounts};
use crate::write::{delete_row, load_row};
use crate::{Origin, Result, Store};

/// Live notes in the trash, with whether each is locked.
const TRASHED: &str = "SELECT id, json_extract(json, '$.kansoLock') IS NOT NULL FROM docs \
                       WHERE type = 'note' AND deleted = 0 AND IFNULL(book_id, '') = 'trash' \
                       ORDER BY id";

fn trashed(conn: &rusqlite::Connection) -> Result<Vec<(String, bool)>> {
    let mut stmt = conn.prepare(TRASHED)?;
    let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

impl Store {
    /// How many notes the trash holds, and how many of them are locked.
    pub fn trash_counts(&self) -> Result<TrashCounts> {
        self.read(|conn| {
            let notes = trashed(conn)?;
            Ok(TrashCounts {
                notes: notes.len() as u32,
                locked: notes.iter().filter(|(_, locked)| *locked).count() as u32,
            })
        })
    }

    /// Deletes every note in the trash, locked ones too, then each file
    /// that only those notes owned, in one transaction.
    pub fn empty_trash(&self, origin: Origin) -> Result<Emptied> {
        self.write(|tx, changes| {
            let notes = trashed(tx)?;
            let mut files = BTreeSet::new();
            for (id, _) in &notes {
                let mut stmt = tx.prepare("SELECT file_id FROM note_files WHERE note_id = ?")?;
                for file in stmt.query_map([id], |row| row.get::<_, String>(0))? {
                    files.insert(file?);
                }
            }
            for (id, _) in &notes {
                let Some(prev) = load_row(tx, id)?.filter(|row| !row.deleted) else {
                    continue;
                };
                crate::cards::forget_note_reviews(tx, changes, id, origin)?;
                delete_row(tx, id, &prev, origin, changes)?;
            }
            let mut deleted_files = 0;
            for file in &files {
                if still_owned(tx, file)? {
                    continue;
                }
                let Some(prev) = load_row(tx, file)?.filter(|row| !row.deleted) else {
                    continue;
                };
                if prev.doc_type != DocType::File {
                    continue;
                }
                delete_row(tx, file, &prev, origin, changes)?;
                deleted_files += 1;
            }
            Ok(Emptied {
                notes: notes.len() as u32,
                locked: notes.iter().filter(|(_, locked)| *locked).count() as u32,
                files: deleted_files,
            })
        })
    }
}

/// Whether a live note still owns `file`. The trashed notes are deleted
/// by now, so any owner left is outside the trash.
fn still_owned(tx: &Transaction, file: &str) -> Result<bool> {
    Ok(tx.query_row(
        "SELECT EXISTS (SELECT 1 FROM note_files nf JOIN docs d ON d.id = nf.note_id
         WHERE nf.file_id = ?1 AND d.type = 'note' AND d.deleted = 0)",
        [file],
        |row| row.get(0),
    )?)
}
