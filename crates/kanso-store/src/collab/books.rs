//! Shared notebooks' side of the library (issue #390 › Shared notebooks):
//! the local tables `kanso-collab` keeps a notebook's state in. Like the
//! shared notes' they never sync; each device rebuilds them from the server.

use kanso_core::now_ms;
use rusqlite::{OptionalExtension, params};

use super::books_model::{BookOutgoing, BookStream, SharedBook};
use super::model::{SealedKey, SharedStatus};
use crate::{Result, Store};

/// Migration 28.
pub(crate) const MIGRATION: &str = r#"
CREATE TABLE collab_books (
    share_id TEXT PRIMARY KEY NOT NULL,
    book_id TEXT NOT NULL,
    owner_id TEXT NOT NULL,
    owner_name TEXT NOT NULL,
    role TEXT NOT NULL,
    status TEXT NOT NULL,
    epoch INTEGER NOT NULL,
    chain TEXT NOT NULL,
    head INTEGER NOT NULL DEFAULT 0,
    counter INTEGER NOT NULL DEFAULT 0,
    uploading INTEGER NOT NULL DEFAULT 0,
    updated_at INTEGER NOT NULL
);
CREATE INDEX collab_books_book ON collab_books (book_id);

CREATE TABLE collab_book_keys (
    share_id TEXT NOT NULL,
    epoch INTEGER NOT NULL,
    -- The notebook key sealed under vault_key.
    sealed BLOB NOT NULL,
    PRIMARY KEY (share_id, epoch)
) WITHOUT ROWID;

CREATE TABLE collab_book_streams (
    share_id TEXT NOT NULL,
    stream TEXT NOT NULL,
    note_id TEXT,
    state BLOB NOT NULL,
    applied_seq INTEGER NOT NULL DEFAULT 0,
    on_server INTEGER NOT NULL DEFAULT 0,
    base TEXT NOT NULL DEFAULT '',
    PRIMARY KEY (share_id, stream)
) WITHOUT ROWID;
CREATE INDEX collab_book_streams_note ON collab_book_streams (note_id);

CREATE TABLE collab_book_outbox (
    id INTEGER PRIMARY KEY,
    share_id TEXT NOT NULL,
    stream TEXT NOT NULL,
    payload BLOB NOT NULL,
    created_at INTEGER NOT NULL
);
CREATE INDEX collab_book_outbox_share ON collab_book_outbox (share_id, id);
"#;

const BOOK_COLUMNS: &str = "share_id, book_id, owner_id, owner_name, role, status, epoch, chain, head, counter, uploading, updated_at";

fn book_from_row(row: &rusqlite::Row) -> rusqlite::Result<SharedBook> {
    let status: String = row.get(5)?;
    Ok(SharedBook {
        share_id: row.get(0)?,
        book_id: row.get(1)?,
        owner_id: row.get(2)?,
        owner_name: row.get(3)?,
        role: row.get(4)?,
        status: SharedStatus::parse(&status).unwrap_or(SharedStatus::Removed),
        epoch: row.get(6)?,
        chain: row.get(7)?,
        head: row.get::<_, i64>(8)? as u64,
        counter: row.get::<_, i64>(9)? as u64,
        uploading: row.get(10)?,
        updated_at: row.get(11)?,
    })
}

fn stream_from_row(row: &rusqlite::Row) -> rusqlite::Result<BookStream> {
    Ok(BookStream {
        share_id: row.get(0)?,
        stream: row.get(1)?,
        note_id: row.get(2)?,
        state: row.get(3)?,
        applied_seq: row.get::<_, i64>(4)? as u64,
        on_server: row.get(5)?,
        base: row.get(6)?,
    })
}

const STREAM_COLUMNS: &str = "share_id, stream, note_id, state, applied_seq, on_server, base";

impl Store {
    pub fn shared_books(&self) -> Result<Vec<SharedBook>> {
        self.read(|conn| {
            let mut stmt = conn.prepare(&format!(
                "SELECT {BOOK_COLUMNS} FROM collab_books ORDER BY share_id"
            ))?;
            let rows = stmt.query_map([], book_from_row)?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }

    pub fn shared_book(&self, share_id: &str) -> Result<Option<SharedBook>> {
        self.read(|conn| {
            Ok(conn
                .query_row(
                    &format!("SELECT {BOOK_COLUMNS} FROM collab_books WHERE share_id = ?"),
                    [share_id],
                    book_from_row,
                )
                .optional()?)
        })
    }

    /// The shared notebook a library notebook is, if any.
    pub fn shared_book_of(&self, book_id: &str) -> Result<Option<SharedBook>> {
        self.read(|conn| {
            Ok(conn
                .query_row(
                    &format!(
                        "SELECT {BOOK_COLUMNS} FROM collab_books WHERE book_id = ? AND status = 'active'"
                    ),
                    [book_id],
                    book_from_row,
                )
                .optional()?)
        })
    }

    /// Creates or replaces a shared notebook's row.
    pub fn put_shared_book(&self, book: &SharedBook) -> Result<()> {
        self.write(|tx, _| {
            tx.execute(
                "INSERT INTO collab_books (share_id, book_id, owner_id, owner_name, role, status, epoch,
                     chain, head, counter, uploading, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
                 ON CONFLICT (share_id) DO UPDATE SET book_id = excluded.book_id,
                     owner_id = excluded.owner_id, owner_name = excluded.owner_name,
                     role = excluded.role, status = excluded.status, epoch = excluded.epoch,
                     chain = excluded.chain, head = excluded.head, counter = excluded.counter,
                     uploading = excluded.uploading, updated_at = excluded.updated_at",
                params![
                    book.share_id,
                    book.book_id,
                    book.owner_id,
                    book.owner_name,
                    book.role,
                    book.status.as_str(),
                    book.epoch,
                    book.chain,
                    book.head as i64,
                    book.counter as i64,
                    book.uploading,
                    book.updated_at,
                ],
            )?;
            Ok(())
        })
    }

    /// Forgets a shared notebook's local state: its row, keys, logs and
    /// outbox. The notebook and its notes stay.
    pub fn forget_shared_book(&self, share_id: &str) -> Result<()> {
        self.write(|tx, _| {
            for table in [
                "collab_books",
                "collab_book_keys",
                "collab_book_streams",
                "collab_book_outbox",
            ] {
                tx.execute(
                    &format!("DELETE FROM {table} WHERE share_id = ?"),
                    [share_id],
                )?;
            }
            Ok(())
        })
    }

    /// Forgets what a notebook that ended here (removed, left, stopped)
    /// kept: keys, logs and outbox. The row stays, with the chain this
    /// device trusted, so joining again is held to that history.
    pub fn clear_shared_book_state(&self, share_id: &str) -> Result<()> {
        self.write(|tx, _| {
            for table in [
                "collab_book_keys",
                "collab_book_streams",
                "collab_book_outbox",
            ] {
                tx.execute(
                    &format!("DELETE FROM {table} WHERE share_id = ?"),
                    [share_id],
                )?;
            }
            Ok(())
        })
    }

    /// Keeps a notebook key (sealed) for one epoch.
    pub fn put_book_key(&self, share_id: &str, epoch: u32, sealed: &[u8]) -> Result<()> {
        self.write(|tx, _| {
            tx.execute(
                "INSERT INTO collab_book_keys (share_id, epoch, sealed) VALUES (?1, ?2, ?3)
                 ON CONFLICT (share_id, epoch) DO UPDATE SET sealed = excluded.sealed",
                params![share_id, epoch, sealed],
            )?;
            Ok(())
        })
    }

    /// Every notebook key kept, oldest epoch first (`list` and
    /// `list_sig` are always `None`: a notebook's lists are its chain).
    pub fn book_keys(&self, share_id: &str) -> Result<Vec<SealedKey>> {
        self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT epoch, sealed FROM collab_book_keys WHERE share_id = ? ORDER BY epoch",
            )?;
            let rows = stmt.query_map([share_id], |row| {
                Ok(SealedKey {
                    epoch: row.get(0)?,
                    sealed: row.get(1)?,
                    list: None,
                    list_sig: None,
                })
            })?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }

    pub fn book_stream(&self, share_id: &str, stream: &str) -> Result<Option<BookStream>> {
        self.read(|conn| {
            Ok(conn
                .query_row(
                    &format!(
                        "SELECT {STREAM_COLUMNS} FROM collab_book_streams WHERE share_id = ?1 AND stream = ?2"
                    ),
                    params![share_id, stream],
                    stream_from_row,
                )
                .optional()?)
        })
    }

    /// The log of a library note, if it is in a shared notebook.
    pub fn book_stream_of_note(&self, note_id: &str) -> Result<Option<BookStream>> {
        self.read(|conn| {
            Ok(conn
                .query_row(
                    &format!(
                        "SELECT {STREAM_COLUMNS} FROM collab_book_streams WHERE note_id = ? LIMIT 1"
                    ),
                    [note_id],
                    stream_from_row,
                )
                .optional()?)
        })
    }

    /// The library notes carrying a notebook's `kansoShare`, with the log
    /// each names: (stream, note id, notebook id).
    pub fn notes_marked_for_book(&self, share_id: &str) -> Result<Vec<(String, String, String)>> {
        self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT json_extract(json, '$.kansoShare.stream'), id, IFNULL(book_id, '') FROM docs
                 WHERE type = 'note' AND deleted = 0
                 AND json_extract(json, '$.kansoShare.shareId') = ?
                 AND json_extract(json, '$.kansoShare.stream') IS NOT NULL",
            )?;
            let rows = stmt.query_map([share_id], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }

    /// Every log of a notebook, the manifest included, by id.
    pub fn book_streams(&self, share_id: &str) -> Result<Vec<BookStream>> {
        self.read(|conn| {
            let mut stmt = conn.prepare(&format!(
                "SELECT {STREAM_COLUMNS} FROM collab_book_streams WHERE share_id = ? ORDER BY stream"
            ))?;
            let rows = stmt.query_map([share_id], stream_from_row)?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }

    pub fn put_book_stream(&self, stream: &BookStream) -> Result<()> {
        self.write(|tx, _| {
            tx.execute(
                "INSERT INTO collab_book_streams (share_id, stream, note_id, state, applied_seq, on_server, base)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                 ON CONFLICT (share_id, stream) DO UPDATE SET note_id = excluded.note_id,
                     state = excluded.state, applied_seq = excluded.applied_seq,
                     on_server = excluded.on_server, base = excluded.base",
                params![
                    stream.share_id,
                    stream.stream,
                    stream.note_id,
                    stream.state,
                    stream.applied_seq as i64,
                    stream.on_server,
                    stream.base,
                ],
            )?;
            Ok(())
        })
    }

    /// Forgets a log that never went up (its note left the notebook, was
    /// deleted or locked before it did).
    pub fn drop_book_stream(&self, share_id: &str, stream: &str) -> Result<()> {
        self.write(|tx, _| {
            tx.execute(
                "DELETE FROM collab_book_streams WHERE share_id = ?1 AND stream = ?2",
                params![share_id, stream],
            )?;
            Ok(())
        })
    }

    /// Queues something written on this device for a notebook's log.
    pub fn push_book_outbox(&self, share_id: &str, stream: &str, payload: &[u8]) -> Result<i64> {
        self.write(|tx, _| {
            tx.execute(
                "INSERT INTO collab_book_outbox (share_id, stream, payload, created_at) VALUES (?1, ?2, ?3, ?4)",
                params![share_id, stream, payload, now_ms()],
            )?;
            Ok(tx.last_insert_rowid())
        })
    }

    /// What this device wrote that the server hasn't taken, oldest first.
    pub fn book_outbox(&self, share_id: &str) -> Result<Vec<BookOutgoing>> {
        self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, share_id, stream, payload, created_at FROM collab_book_outbox
                 WHERE share_id = ? ORDER BY id",
            )?;
            let rows = stmt.query_map([share_id], |row| {
                Ok(BookOutgoing {
                    id: row.get(0)?,
                    share_id: row.get(1)?,
                    stream: row.get(2)?,
                    payload: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }

    pub fn drop_book_outbox(&self, id: i64) -> Result<()> {
        self.write(|tx, _| {
            tx.execute("DELETE FROM collab_book_outbox WHERE id = ?", [id])?;
            Ok(())
        })
    }
}
