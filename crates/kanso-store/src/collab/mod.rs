//! Shared notes' side of the library (docs/ACCOUNTS.md › Collaboration):
//! local tables `kanso-collab` keeps its state in, the write that puts a
//! shared note's merged text back into the note, and the rules that keep a
//! shared note's text out of private sync.
//!
//! A shared note is an ordinary note with `kansoShare` set. Its `title` and
//! `body` are always the CRDT's text, written here with [`Origin::Collab`];
//! the CRDT's state, keys, outbox and comments live in the `collab_*`
//! tables, which never sync (each device rebuilds them from the server).

pub mod model;

use kanso_core::model::{LOCK_FIELD, SHARE_FIELD};
use kanso_core::{DocType, now_ms};
use rusqlite::{OptionalExtension, params};
use serde_json::{Map, Value};

use crate::write::{Synced, load_row, next_rev, store_row};
use crate::{Origin, Result, Store, StoreError, WriteResult};
use model::{
    CommentRecord, Contact, Outgoing, SealedKey, SharedKind, SharedNote, SharedStatus, StoredNotice,
};

/// Migration 9.
pub(crate) const MIGRATION: &str = r#"
CREATE TABLE collab_notes (
    share_id TEXT PRIMARY KEY NOT NULL,
    note_id TEXT NOT NULL UNIQUE,
    owner_id TEXT NOT NULL,
    owner_name TEXT NOT NULL,
    role TEXT NOT NULL,
    status TEXT NOT NULL,
    epoch INTEGER NOT NULL,
    list TEXT NOT NULL,
    list_sig TEXT NOT NULL,
    state BLOB NOT NULL,
    applied_seq INTEGER NOT NULL DEFAULT 0,
    since_snapshot INTEGER NOT NULL DEFAULT 0,
    bytes_since_snapshot INTEGER NOT NULL DEFAULT 0,
    counter INTEGER NOT NULL DEFAULT 0,
    needs_snapshot INTEGER NOT NULL DEFAULT 0,
    -- Who changed the note since its last saved version (a JSON array of
    -- names); the next version saved takes them as its author.
    authors TEXT NOT NULL DEFAULT '[]',
    updated_at INTEGER NOT NULL
);

CREATE TABLE collab_keys (
    share_id TEXT NOT NULL,
    epoch INTEGER NOT NULL,
    -- The note key sealed under vault_key.
    sealed BLOB NOT NULL,
    -- That epoch's member list and its signature, when this device saw it.
    list TEXT,
    list_sig TEXT,
    PRIMARY KEY (share_id, epoch)
) WITHOUT ROWID;

CREATE TABLE collab_outbox (
    id INTEGER PRIMARY KEY,
    share_id TEXT NOT NULL,
    kind TEXT NOT NULL,
    payload BLOB NOT NULL,
    created_at INTEGER NOT NULL
);
CREATE INDEX collab_outbox_share ON collab_outbox (share_id, id);

CREATE TABLE collab_comment_events (
    share_id TEXT NOT NULL,
    seq INTEGER NOT NULL,
    author TEXT NOT NULL,
    author_name TEXT NOT NULL,
    role TEXT NOT NULL,
    json TEXT NOT NULL,
    PRIMARY KEY (share_id, seq)
) WITHOUT ROWID;

CREATE TABLE collab_contacts (
    account_id TEXT PRIMARY KEY NOT NULL,
    username TEXT NOT NULL,
    sign_pub TEXT NOT NULL,
    box_pub TEXT NOT NULL,
    verified_at INTEGER,
    seen_at INTEGER NOT NULL
);

CREATE TABLE collab_notices (
    id TEXT PRIMARY KEY NOT NULL,
    share_id TEXT NOT NULL,
    comment_id TEXT NOT NULL,
    from_id TEXT NOT NULL,
    from_name TEXT NOT NULL,
    title TEXT NOT NULL,
    excerpt TEXT NOT NULL,
    at INTEGER NOT NULL,
    read INTEGER NOT NULL DEFAULT 0
);

ALTER TABLE revisions ADD COLUMN author TEXT;
"#;

/// The share a note's fields name, if it is a shared note.
pub(crate) fn share_of(fields: &Map<String, Value>) -> Option<&Map<String, Value>> {
    fields.get(SHARE_FIELD).and_then(Value::as_object)
}

/// A shared note's role for this account, as its `kansoShare` says.
pub(crate) fn shared_role(fields: &Map<String, Value>) -> Option<&str> {
    share_of(fields)?.get("role").and_then(Value::as_str)
}

fn is_canvas(fields: &Map<String, Value>) -> bool {
    fields.get("doctype").and_then(Value::as_str) == Some("canvas")
}

/// What private sync sends of a shared note: everything but its text, which
/// the CRDT carries to everyone (this account's devices included).
pub(crate) fn private_part(fields: &mut Map<String, Value>) {
    if share_of(fields).is_some() {
        fields.insert("title".into(), Value::String(String::new()));
        fields.insert("body".into(), Value::String(String::new()));
    }
}

/// A shared note arriving through private sync takes the text this device
/// already has for it: only `kanso-collab` writes a shared note's text.
pub(crate) fn keep_shared_text(
    fields: &mut Map<String, Value>,
    local: Option<&Map<String, Value>>,
) {
    let Some(share) = share_of(fields).and_then(|s| s.get("shareId")).cloned() else {
        return;
    };
    let same = local.and_then(|l| share_of(l)?.get("shareId")) == Some(&share);
    for key in ["title", "body"] {
        let value = if same {
            local
                .and_then(|l| l.get(key))
                .cloned()
                .unwrap_or_else(|| Value::String(String::new()))
        } else {
            Value::String(String::new())
        };
        fields.insert(key.into(), value);
    }
}

/// The store's rules for shared notes, on every write but sync's and the
/// collab engine's own: a `view` or `comment` member can't change the text,
/// and a shared note can't be locked or moved into a locked notebook.
pub(crate) fn guard_shared(
    fields: &Map<String, Value>,
    previous: Option<&Map<String, Value>>,
    in_locked: bool,
) -> Result<()> {
    if let Some(prev) = previous
        && let Some(role) = shared_role(prev)
        && !matches!(role, "owner" | "edit")
        && (prev.get("body") != fields.get("body") || prev.get("title") != fields.get("title"))
    {
        return Err(StoreError::Invalid(
            "The note was shared with you to read: its text can't be changed here.".into(),
        ));
    }
    // A shared canvas's body is the projection of its CRDT, written by the
    // engine only: nobody else may change it, whatever their role, and the
    // note can't stop being a canvas while it is shared.
    if let Some(prev) = previous
        && share_of(prev).is_some()
        && is_canvas(prev)
        && (prev.get("body") != fields.get("body") || !is_canvas(fields))
    {
        return Err(StoreError::Invalid(
            "The canvas is shared: it can only be changed by drawing on it, not by writing its text."
                .into(),
        ));
    }
    // Nor can a shared text note become a canvas: its CRDT is text, so the
    // canvas's JSON would be merged character by character.
    if let Some(prev) = previous
        && share_of(prev).is_some()
        && !is_canvas(prev)
        && is_canvas(fields)
    {
        return Err(StoreError::Invalid(
            "The note is shared as text: stop sharing it before making it a canvas.".into(),
        ));
    }
    if share_of(fields).is_some() && (fields.contains_key(LOCK_FIELD) || in_locked) {
        return Err(StoreError::Invalid(
            "The note is shared: stop sharing it before locking it or moving it into a locked notebook."
                .into(),
        ));
    }
    Ok(())
}

/// What [`Store::write_shared_text`] answers for a note that is locked, or
/// in a locked notebook.
pub const LOCKED_SHARED: &str =
    "The shared note is locked, or in a locked notebook: its shared text isn't written to it.";

/// Takes the names that changed a shared note since its last saved version,
/// leaving none: the version being saved now is theirs.
pub(crate) fn take_authors(tx: &rusqlite::Transaction, note_id: &str) -> Result<Option<String>> {
    let authors: Option<String> = tx
        .query_row(
            "SELECT authors FROM collab_notes WHERE note_id = ?",
            [note_id],
            |row| row.get(0),
        )
        .optional()?;
    let Some(authors) = authors else {
        return Ok(None);
    };
    let names: Vec<String> = serde_json::from_str(&authors).unwrap_or_default();
    if names.is_empty() {
        return Ok(None);
    }
    tx.execute(
        "UPDATE collab_notes SET authors = '[]' WHERE note_id = ?",
        [note_id],
    )?;
    Ok(Some(names.join(", ")))
}

fn add_authors(tx: &rusqlite::Transaction, note_id: &str, names: &[String]) -> Result<()> {
    if names.is_empty() {
        return Ok(());
    }
    let authors: Option<String> = tx
        .query_row(
            "SELECT authors FROM collab_notes WHERE note_id = ?",
            [note_id],
            |row| row.get(0),
        )
        .optional()?;
    let Some(authors) = authors else {
        return Ok(());
    };
    let mut list: Vec<String> = serde_json::from_str(&authors).unwrap_or_default();
    for name in names {
        if !list.contains(name) {
            list.push(name.clone());
        }
    }
    tx.execute(
        "UPDATE collab_notes SET authors = ?2 WHERE note_id = ?1",
        params![note_id, serde_json::to_string(&list)?],
    )?;
    Ok(())
}

fn shared_from_row(row: &rusqlite::Row) -> rusqlite::Result<SharedNote> {
    let status: String = row.get(5)?;
    Ok(SharedNote {
        share_id: row.get(0)?,
        note_id: row.get(1)?,
        owner_id: row.get(2)?,
        owner_name: row.get(3)?,
        role: row.get(4)?,
        status: SharedStatus::parse(&status).unwrap_or(SharedStatus::Removed),
        epoch: row.get(6)?,
        list: row.get(7)?,
        list_sig: row.get(8)?,
        state: row.get(9)?,
        applied_seq: row.get::<_, i64>(10)? as u64,
        since_snapshot: row.get::<_, i64>(11)? as u64,
        bytes_since_snapshot: row.get::<_, i64>(12)? as u64,
        counter: row.get::<_, i64>(13)? as u64,
        needs_snapshot: row.get(14)?,
        updated_at: row.get(15)?,
        kind: SharedKind::parse(&row.get::<_, String>(16)?),
    })
}

const SHARED_COLUMNS: &str = "share_id, note_id, owner_id, owner_name, role, status, epoch, list, \
    list_sig, state, applied_seq, since_snapshot, bytes_since_snapshot, counter, needs_snapshot, updated_at, kind";

/// What [`Store::write_shared_text`] writes into a shared note.
#[derive(Clone, Debug)]
pub struct SharedText<'a> {
    pub note_id: &'a str,
    pub title: &'a str,
    pub body: &'a str,
    /// The note's `kansoShare`, or `None` to take it off (a note no longer
    /// shared, kept as a private copy).
    pub mark: Option<Value>,
    /// The notebook a note that doesn't exist yet goes into.
    pub new_in: &'a str,
    /// The revision the caller read; the write is refused with
    /// [`StoreError::Conflict`] when the note moved since. `None` writes
    /// whatever the note holds.
    pub expected_rev: Option<&'a str>,
    /// Who made this change, for the note's history.
    pub authors: &'a [String],
}

/// What [`Store::write_shared_canvas`] writes into a shared canvas.
#[derive(Clone, Debug)]
pub struct SharedCanvas<'a> {
    pub note_id: &'a str,
    pub title: &'a str,
    pub canvas: &'a kanso_core::canvas::Canvas,
    pub mark: Option<Value>,
    pub new_in: &'a str,
    pub expected_rev: Option<&'a str>,
    pub authors: &'a [String],
}

impl Store {
    /// Every shared note this library holds.
    pub fn shared_notes(&self) -> Result<Vec<SharedNote>> {
        self.read(|conn| {
            let mut stmt = conn.prepare(&format!(
                "SELECT {SHARED_COLUMNS} FROM collab_notes ORDER BY share_id"
            ))?;
            let rows = stmt.query_map([], shared_from_row)?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }

    pub fn shared_note(&self, share_id: &str) -> Result<Option<SharedNote>> {
        self.read(|conn| {
            Ok(conn
                .query_row(
                    &format!("SELECT {SHARED_COLUMNS} FROM collab_notes WHERE share_id = ?"),
                    [share_id],
                    shared_from_row,
                )
                .optional()?)
        })
    }

    /// The shared note a note is, if any.
    pub fn shared_note_of(&self, note_id: &str) -> Result<Option<SharedNote>> {
        self.read(|conn| {
            Ok(conn
                .query_row(
                    &format!("SELECT {SHARED_COLUMNS} FROM collab_notes WHERE note_id = ?"),
                    [note_id],
                    shared_from_row,
                )
                .optional()?)
        })
    }

    /// Creates or replaces a shared note's row (its `authors` are kept).
    pub fn put_shared_note(&self, note: &SharedNote) -> Result<()> {
        self.write(|tx, _| {
            tx.execute(
                "INSERT INTO collab_notes (share_id, note_id, owner_id, owner_name, role, status, epoch,
                     list, list_sig, state, applied_seq, since_snapshot, bytes_since_snapshot, counter,
                     needs_snapshot, updated_at, kind)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)
                 ON CONFLICT (share_id) DO UPDATE SET note_id = excluded.note_id,
                     owner_id = excluded.owner_id, owner_name = excluded.owner_name, role = excluded.role,
                     status = excluded.status, epoch = excluded.epoch, list = excluded.list,
                     list_sig = excluded.list_sig, state = excluded.state,
                     applied_seq = excluded.applied_seq, since_snapshot = excluded.since_snapshot,
                     bytes_since_snapshot = excluded.bytes_since_snapshot, counter = excluded.counter,
                     needs_snapshot = excluded.needs_snapshot, updated_at = excluded.updated_at",
                params![
                    note.share_id,
                    note.note_id,
                    note.owner_id,
                    note.owner_name,
                    note.role,
                    note.status.as_str(),
                    note.epoch,
                    note.list,
                    note.list_sig,
                    note.state,
                    note.applied_seq as i64,
                    note.since_snapshot as i64,
                    note.bytes_since_snapshot as i64,
                    note.counter as i64,
                    note.needs_snapshot,
                    note.updated_at,
                    note.kind.as_str(),
                ],
            )?;
            Ok(())
        })
    }

    /// Forgets a shared note's local state: its row, keys, outbox and
    /// comments. The note itself stays.
    pub fn forget_shared_note(&self, share_id: &str) -> Result<()> {
        self.write(|tx, _| {
            for table in [
                "collab_notes",
                "collab_keys",
                "collab_outbox",
                "collab_comment_events",
            ] {
                tx.execute(
                    &format!("DELETE FROM {table} WHERE share_id = ?"),
                    [share_id],
                )?;
            }
            tx.execute("DELETE FROM collab_notices WHERE share_id = ?", [share_id])?;
            Ok(())
        })
    }

    /// Writes a shared note's merged text (and its `kansoShare`), creating
    /// the note when this library doesn't have it yet, as [`Origin::Collab`].
    /// The names in `authors` are added to who changed it since its last
    /// saved version.
    pub fn write_shared_text(&self, text: &SharedText) -> Result<WriteResult> {
        self.write_shared(text, None)
    }

    /// Writes a shared canvas's projection (and its `kansoShare`) as the
    /// note's body, creating the note when this library doesn't have it yet,
    /// as [`Origin::Collab`]. The canvas is checked as every canvas body is:
    /// one that doesn't validate, or is past the size cap, is refused with
    /// [`StoreError::Invalid`] (never cut short), and the note keeps its last
    /// good body. The note's `doctype` is set to `canvas`.
    pub fn write_shared_canvas(&self, canvas: &SharedCanvas) -> Result<WriteResult> {
        canvas.canvas.validate().map_err(|why| {
            StoreError::Invalid(format!("The shared canvas can't be saved: {why}"))
        })?;
        let body = kanso_core::canvas::to_body(canvas.canvas);
        kanso_core::canvas::check_body(&body).map_err(StoreError::Invalid)?;
        self.write_shared(
            &SharedText {
                note_id: canvas.note_id,
                title: canvas.title,
                body: &body,
                mark: canvas.mark.clone(),
                new_in: canvas.new_in,
                expected_rev: canvas.expected_rev,
                authors: canvas.authors,
            },
            Some("canvas"),
        )
    }

    fn write_shared(&self, text: &SharedText, doctype: Option<&str>) -> Result<WriteResult> {
        self.write(|tx, changes| {
            let previous = load_row(tx, text.note_id)?.filter(|row| !row.deleted);
            if let (Some(expected), Some(prev)) = (text.expected_rev, &previous)
                && prev.rev != expected
            {
                return Err(StoreError::Conflict);
            }
            // A note that is locked, or in a locked notebook, never gets
            // shared text in plain: however it got there (private sync from
            // another device), it is refused, and the engine says so.
            if let Some(prev) = &previous
                && text.mark.is_some()
                && (prev.fields.contains_key(LOCK_FIELD)
                    || crate::write::in_locked_book(
                        tx,
                        prev.fields.get("bookId").and_then(Value::as_str),
                    )?)
            {
                return Err(StoreError::Invalid(LOCKED_SHARED.into()));
            }
            let now = now_ms();
            let mut fields = match &previous {
                Some(prev) => prev.fields.clone(),
                None => {
                    let mut fields = Map::new();
                    fields.insert("bookId".into(), Value::String(text.new_in.to_owned()));
                    fields.insert("createdAt".into(), now.into());
                    fields
                }
            };
            let unchanged = previous.as_ref().is_some_and(|prev| {
                prev.fields.get("title").and_then(Value::as_str) == Some(text.title)
                    && prev.fields.get("body").and_then(Value::as_str) == Some(text.body)
                    && prev.fields.get(SHARE_FIELD) == text.mark.as_ref()
                    && doctype.is_none_or(|kind| {
                        prev.fields.get("doctype").and_then(Value::as_str) == Some(kind)
                    })
            });
            if unchanged {
                let prev = previous.as_ref().expect("checked");
                add_authors(tx, text.note_id, text.authors)?;
                return Ok(WriteResult {
                    id: text.note_id.to_owned(),
                    rev: prev.rev.clone(),
                });
            }
            fields.insert("title".into(), Value::String(text.title.to_owned()));
            fields.insert("body".into(), Value::String(text.body.to_owned()));
            if let Some(kind) = doctype {
                fields.insert("doctype".into(), Value::String(kind.to_owned()));
            }
            match &text.mark {
                Some(mark) => {
                    fields.insert(SHARE_FIELD.into(), mark.clone());
                }
                None => {
                    fields.remove(SHARE_FIELD);
                }
            }
            fields.insert("updatedAt".into(), now.into());
            let created = fields
                .get("createdAt")
                .and_then(Value::as_i64)
                .unwrap_or(now);
            let fields = crate::validate::normalize(
                DocType::Note,
                text.note_id,
                fields,
                None,
                created,
                crate::validate::Refs::Check(tx),
            )?;
            let rev = next_rev(previous.as_ref(), &Value::Object(fields.clone()))?;
            store_row(
                tx,
                text.note_id,
                DocType::Note,
                &rev,
                false,
                &fields,
                previous.as_ref(),
                Synced::Keep,
                Origin::Collab,
                changes,
            )?;
            add_authors(tx, text.note_id, text.authors)?;
            Ok(WriteResult {
                id: text.note_id.to_owned(),
                rev,
            })
        })
    }

    /// Who changed a shared note since its last saved version: the history
    /// shows them against the note as it is now.
    pub fn shared_authors(&self, note_id: &str) -> Result<Vec<String>> {
        self.read(|conn| {
            let authors: Option<String> = conn
                .query_row(
                    "SELECT authors FROM collab_notes WHERE note_id = ?",
                    [note_id],
                    |row| row.get(0),
                )
                .optional()?;
            Ok(authors
                .and_then(|a| serde_json::from_str(&a).ok())
                .unwrap_or_default())
        })
    }

    /// Adds names to who changed a shared note since its last saved version.
    pub fn add_shared_authors(&self, note_id: &str, names: &[String]) -> Result<()> {
        self.write(|tx, _| add_authors(tx, note_id, names))
    }

    /// Keeps a note key (sealed) and, when known, its epoch's member list.
    /// A list already kept isn't dropped by a later call without one.
    pub fn put_collab_key(
        &self,
        share_id: &str,
        epoch: u32,
        sealed: &[u8],
        list: Option<(&str, &str)>,
    ) -> Result<()> {
        self.write(|tx, _| {
            tx.execute(
                "INSERT INTO collab_keys (share_id, epoch, sealed, list, list_sig) VALUES (?1, ?2, ?3, ?4, ?5)
                 ON CONFLICT (share_id, epoch) DO UPDATE SET sealed = excluded.sealed,
                     list = coalesce(excluded.list, collab_keys.list),
                     list_sig = coalesce(excluded.list_sig, collab_keys.list_sig)",
                params![share_id, epoch, sealed, list.map(|l| l.0), list.map(|l| l.1)],
            )?;
            Ok(())
        })
    }

    /// A note key for one epoch (sealed), with that epoch's list if kept.
    pub fn collab_key(&self, share_id: &str, epoch: u32) -> Result<Option<SealedKey>> {
        self.read(|conn| {
            Ok(conn
                .query_row(
                    "SELECT sealed, list, list_sig FROM collab_keys WHERE share_id = ?1 AND epoch = ?2",
                    params![share_id, epoch],
                    |row| {
                        Ok(SealedKey {
                            epoch,
                            sealed: row.get(0)?,
                            list: row.get(1)?,
                            list_sig: row.get(2)?,
                        })
                    },
                )
                .optional()?)
        })
    }

    /// Every note key kept for a note, oldest epoch first.
    pub fn collab_keys(&self, share_id: &str) -> Result<Vec<SealedKey>> {
        self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT epoch, sealed, list, list_sig FROM collab_keys WHERE share_id = ? ORDER BY epoch",
            )?;
            let rows = stmt.query_map([share_id], |row| {
                Ok(SealedKey {
                    epoch: row.get(0)?,
                    sealed: row.get(1)?,
                    list: row.get(2)?,
                    list_sig: row.get(3)?,
                })
            })?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }

    /// The note carrying `kansoShare.shareId = share_id`, if this library has one.
    pub fn note_with_share(&self, share_id: &str) -> Result<Option<String>> {
        self.read(|conn| {
            Ok(conn
                .query_row(
                    "SELECT id FROM docs WHERE type = 'note' AND deleted = 0
                     AND json_extract(json, '$.kansoShare.shareId') = ? LIMIT 1",
                    [share_id],
                    |row| row.get(0),
                )
                .optional()?)
        })
    }

    /// Queues something written on this device for the server.
    pub fn push_outbox(&self, share_id: &str, kind: &str, payload: &[u8]) -> Result<i64> {
        self.write(|tx, _| {
            tx.execute(
                "INSERT INTO collab_outbox (share_id, kind, payload, created_at) VALUES (?1, ?2, ?3, ?4)",
                params![share_id, kind, payload, now_ms()],
            )?;
            Ok(tx.last_insert_rowid())
        })
    }

    /// What this device wrote to a note that the server hasn't taken, oldest
    /// first.
    pub fn outbox(&self, share_id: &str) -> Result<Vec<Outgoing>> {
        self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, share_id, kind, payload, created_at FROM collab_outbox
                 WHERE share_id = ? ORDER BY id",
            )?;
            let rows = stmt.query_map([share_id], |row| {
                Ok(Outgoing {
                    id: row.get(0)?,
                    share_id: row.get(1)?,
                    kind: row.get(2)?,
                    payload: row.get(3)?,
                    created_at: row.get(4)?,
                })
            })?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }

    pub fn drop_outbox(&self, id: i64) -> Result<()> {
        self.write(|tx, _| {
            tx.execute("DELETE FROM collab_outbox WHERE id = ?", [id])?;
            Ok(())
        })
    }

    /// Keeps a comment event the log delivered (once per sequence).
    pub fn add_comment_event(&self, share_id: &str, record: &CommentRecord) -> Result<()> {
        self.write(|tx, _| {
            tx.execute(
                "INSERT OR IGNORE INTO collab_comment_events (share_id, seq, author, author_name, role, json)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    share_id,
                    record.seq as i64,
                    record.author,
                    record.author_name,
                    record.role,
                    record.json
                ],
            )?;
            Ok(())
        })
    }

    pub fn comment_events(&self, share_id: &str) -> Result<Vec<CommentRecord>> {
        self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT seq, author, author_name, role, json FROM collab_comment_events
                 WHERE share_id = ? ORDER BY seq",
            )?;
            let rows = stmt.query_map([share_id], |row| {
                Ok(CommentRecord {
                    seq: row.get::<_, i64>(0)? as u64,
                    author: row.get(1)?,
                    author_name: row.get(2)?,
                    role: row.get(3)?,
                    json: row.get(4)?,
                })
            })?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }

    pub fn contact(&self, account_id: &str) -> Result<Option<Contact>> {
        self.read(|conn| {
            Ok(conn
                .query_row(
                    "SELECT account_id, username, sign_pub, box_pub, verified_at, seen_at
                     FROM collab_contacts WHERE account_id = ?",
                    [account_id],
                    |row| {
                        Ok(Contact {
                            account_id: row.get(0)?,
                            username: row.get(1)?,
                            sign_pub: row.get(2)?,
                            box_pub: row.get(3)?,
                            verified_at: row.get(4)?,
                            seen_at: row.get(5)?,
                        })
                    },
                )
                .optional()?)
        })
    }

    pub fn put_contact(&self, contact: &Contact) -> Result<()> {
        self.write(|tx, _| {
            tx.execute(
                "INSERT INTO collab_contacts (account_id, username, sign_pub, box_pub, verified_at, seen_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT (account_id) DO UPDATE SET username = excluded.username,
                     sign_pub = excluded.sign_pub, box_pub = excluded.box_pub,
                     verified_at = excluded.verified_at, seen_at = excluded.seen_at",
                params![
                    contact.account_id,
                    contact.username,
                    contact.sign_pub,
                    contact.box_pub,
                    contact.verified_at,
                    contact.seen_at
                ],
            )?;
            Ok(())
        })
    }

    /// Keeps a notification (once per ID).
    pub fn put_notice(&self, notice: &StoredNotice) -> Result<()> {
        self.write(|tx, _| {
            tx.execute(
                "INSERT OR IGNORE INTO collab_notices (id, share_id, comment_id, from_id, from_name, title,
                     excerpt, at, read) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    notice.id,
                    notice.share_id,
                    notice.comment_id,
                    notice.from_id,
                    notice.from_name,
                    notice.title,
                    notice.excerpt,
                    notice.at,
                    notice.read
                ],
            )?;
            Ok(())
        })
    }

    /// Notifications, newest first.
    pub fn notices(&self) -> Result<Vec<StoredNotice>> {
        self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, share_id, comment_id, from_id, from_name, title, excerpt, at, read
                 FROM collab_notices ORDER BY at DESC, id",
            )?;
            let rows = stmt.query_map([], |row| {
                Ok(StoredNotice {
                    id: row.get(0)?,
                    share_id: row.get(1)?,
                    comment_id: row.get(2)?,
                    from_id: row.get(3)?,
                    from_name: row.get(4)?,
                    title: row.get(5)?,
                    excerpt: row.get(6)?,
                    at: row.get(7)?,
                    read: row.get(8)?,
                })
            })?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }

    pub fn mark_notices_read(&self, share_id: Option<&str>) -> Result<()> {
        self.write(|tx, _| {
            match share_id {
                Some(share) => tx.execute(
                    "UPDATE collab_notices SET read = 1 WHERE share_id = ?",
                    [share],
                )?,
                None => tx.execute("UPDATE collab_notices SET read = 1", [])?,
            };
            Ok(())
        })
    }

    /// Notes others shared with this account (`kansoShare` names another
    /// owner), live and not in the trash: the "Shared with me" list.
    pub fn shared_with_me(&self) -> Result<Vec<String>> {
        self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT n.note_id FROM collab_notes n JOIN docs d ON d.id = n.note_id
                 WHERE d.deleted = 0 AND n.role != 'owner' AND IFNULL(d.book_id, '') != 'trash'
                 ORDER BY d.updated_at DESC",
            )?;
            let rows = stmt.query_map([], |row| row.get(0))?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn private_sync_sends_no_shared_text() {
        let mut fields =
            serde_json::json!({"title": "t", "body": "secret", "kansoShare": {"shareId": "s"}})
                .as_object()
                .unwrap()
                .clone();
        private_part(&mut fields);
        assert_eq!(fields["body"], "");
        assert_eq!(fields["title"], "");
        let mut plain = serde_json::json!({"title": "t", "body": "mine"})
            .as_object()
            .unwrap()
            .clone();
        private_part(&mut plain);
        assert_eq!(plain["body"], "mine");
    }

    #[test]
    fn arriving_envelopes_keep_the_local_text() {
        let local = serde_json::json!({"title": "T", "body": "B", "kansoShare": {"shareId": "s"}})
            .as_object()
            .unwrap()
            .clone();
        let mut incoming = serde_json::json!({"title": "", "body": "", "kansoShare": {"shareId": "s"}, "pinned": true})
            .as_object()
            .unwrap()
            .clone();
        keep_shared_text(&mut incoming, Some(&local));
        assert_eq!(incoming["body"], "B");
        assert_eq!(incoming["title"], "T");
        let mut other = serde_json::json!({"body": "x", "kansoShare": {"shareId": "z"}})
            .as_object()
            .unwrap()
            .clone();
        keep_shared_text(&mut other, Some(&local));
        assert_eq!(other["body"], "");
    }
}
