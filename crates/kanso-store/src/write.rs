//! Writes: create, replace, merge, delete and import.

use std::collections::BTreeSet;

use base64::Engine as _;
use kanso_core::id::is_valid_id;
use kanso_core::model::FileMeta;
use kanso_core::model::{
    FILE_LOCK_FIELD, FileLock, KANSO_FILES_FIELD, LOCK_FIELD, MAX_ATTACHMENT_SIZE,
    MAX_SEALED_ATTACHMENT, NoteFile, OCR_FIELD, TRASH_BOOK_ID, UNSEALED_FIELD,
    note_files_from_extra, store_note_files,
};
use kanso_core::tasks::linked_file_ids;
use kanso_core::{DocType, Rev, new_id, now_ms};
use rusqlite::{OptionalExtension, Transaction, params};
use serde_json::{Map, Value, json};

use crate::doc::Attachment;
use crate::validate::{Refs, normalize, strip_reserved};
use crate::{Change, Origin, Result, Store, StoreError, schema};

/// How a POST treats an existing document.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Put {
    /// `POST /notes` and friends: create, or replace the whole document when
    /// `_id` exists, which then requires the current `_rev`.
    Upsert,
    /// `POST /<id>`: shallow-merge the given fields into the current document.
    /// `_rev` is optional and checked when given.
    Merge,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct WriteResult {
    pub id: String,
    pub rev: String,
}

/// Minimum gap between snapshots of UI autosaves of one document.
const UI_SNAPSHOT_INTERVAL_MS: i64 = 5 * 60 * 1000;
/// Snapshots kept per document.
pub(crate) const SNAPSHOTS_PER_DOC: i64 = 50;

/// A row of `docs` as the write path needs it.
pub(crate) struct Row {
    pub doc_type: DocType,
    pub rev: String,
    pub deleted: bool,
    pub fields: Map<String, Value>,
    pub synced_rev: Option<String>,
    /// The replay floor (`docs.synced_gen`).
    pub synced_gen: u64,
    /// The revision in a push whose answer wasn't processed (`docs.sent_rev`).
    pub sent_rev: Option<String>,
}

/// What happens to `synced_rev` when a row is written.
#[derive(Clone, Copy)]
pub(crate) enum Synced<'a> {
    Keep,
    Set(&'a str),
}

pub(crate) fn load_row(tx: &Transaction, id: &str) -> Result<Option<Row>> {
    let row = tx
        .query_row(
            "SELECT type, rev, deleted, json, synced_rev, synced_gen, sent_rev FROM docs WHERE id = ?",
            [id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, bool>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, i64>(5)?,
                    row.get::<_, Option<String>>(6)?,
                ))
            },
        )
        .optional()?;
    let Some((doc_type, rev, deleted, json, synced_rev, synced_gen, sent_rev)) = row else {
        return Ok(None);
    };
    let doc_type = DocType::from_name(&doc_type)
        .ok_or_else(|| StoreError::Invalid(format!("unknown document type {doc_type}")))?;
    let fields = match serde_json::from_str(&json)? {
        Value::Object(map) => map,
        _ => Map::new(),
    };
    Ok(Some(Row {
        doc_type,
        rev,
        deleted,
        fields,
        synced_rev,
        synced_gen: u64::try_from(synced_gen).unwrap_or(0),
        sent_rev,
    }))
}

/// Writes one row and its side tables, snapshotting the version it replaces.
#[allow(clippy::too_many_arguments)]
pub(crate) fn store_row(
    tx: &Transaction,
    id: &str,
    doc_type: DocType,
    rev: &str,
    deleted: bool,
    fields: &Map<String, Value>,
    previous: Option<&Row>,
    synced: Synced,
    origin: Origin,
    changes: &mut Vec<Change>,
) -> Result<()> {
    let is_note = doc_type == DocType::Note;
    // A locked note keeps only versions sealed under its current key, on
    // every device: locking it, or changing the lock password, anywhere
    // forgets its plain and old-key versions here too, however the new
    // version arrived (sync included). A note in a locked notebook is
    // locked the same way, sealed or not.
    let salt_now = (is_note && !deleted).then(|| lock_salt(fields)).flatten();
    let in_locked = is_note && !deleted && in_locked_book(tx, book_of(fields))?;
    let locked = salt_now.is_some() || in_locked;
    if is_note && !deleted && !matches!(origin, Origin::Sync | Origin::Collab) {
        guard_note(tx, fields, previous, in_locked)?;
        crate::collab::guard_shared(
            fields,
            previous.filter(|p| !p.deleted).map(|p| &p.fields),
            in_locked,
        )?;
    }
    if let Some(prev) = previous
        && !prev.deleted
        && (!locked || (salt_now.is_some() && salt_now == lock_salt(&prev.fields)))
    {
        snapshot(tx, id, prev, origin)?;
    }
    if locked {
        let was_locked = match previous.filter(|prev| !prev.deleted) {
            Some(prev) => {
                lock_salt(&prev.fields).is_some() || in_locked_book(tx, book_of(&prev.fields))?
            }
            // A new note has nothing to forget.
            None => true,
        };
        forget_text(tx, id, salt_now.as_deref(), !was_locked)?;
    }
    let book_ids_before = match doc_type {
        DocType::Book => Some(locked_book_ids(tx)?),
        _ => None,
    };
    let seq = schema::next_seq(tx)?;
    let json = serde_json::to_string(fields)?;
    let synced_rev = match synced {
        Synced::Keep => previous.and_then(|p| p.synced_rev.clone()),
        Synced::Set(rev) => Some(rev.to_owned()),
    };
    tx.execute(
        "INSERT INTO docs (id, type, rev, seq, deleted, json, synced_rev)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT (id) DO UPDATE SET type = excluded.type, rev = excluded.rev,
             seq = excluded.seq, deleted = excluded.deleted, json = excluded.json,
             synced_rev = excluded.synced_rev",
        params![id, doc_type.as_str(), rev, seq, deleted, json, synced_rev],
    )?;
    if is_note {
        tx.execute("DELETE FROM note_tags WHERE note_id = ?", [id])?;
        if !deleted && let Some(Value::Array(tags)) = fields.get("tags") {
            for tag in tags.iter().filter_map(Value::as_str) {
                tx.execute(
                    "INSERT OR IGNORE INTO note_tags (note_id, tag_id) VALUES (?1, ?2)",
                    params![id, tag],
                )?;
            }
        }
        index_note(tx, id, (!deleted).then_some(fields), locked, in_locked)?;
    }
    if let Some(before) = book_ids_before {
        books_changed(tx, before, origin)?;
    }
    if doc_type == DocType::File && deleted {
        tx.execute("DELETE FROM blobs WHERE file_id = ?", [id])?;
    }
    if doc_type == DocType::File {
        crate::ocr::write_file(tx, id, (!deleted).then_some(fields))?;
        // A sealed file's past versions may contain the plaintext document or
        // OCR text from before it was sealed. Keep no such history, however
        // the seal arrived (local write or sync).
        if !deleted && fields.contains_key(FILE_LOCK_FIELD) {
            tx.execute("DELETE FROM revisions WHERE doc_id = ?", [id])?;
        }
    }
    if is_note && deleted {
        crate::embed::forget_note(tx, id)?;
    }
    changes.push(Change {
        id: id.to_owned(),
        rev: rev.to_owned(),
        seq,
        deleted,
        origin,
    });
    Ok(())
}

/// Puts a note's words in the search index and its derived tables (links,
/// headings, tasks), or takes them out when `fields` is `None`. A locked
/// note's text is only ever the sealed one: whatever an older Kanso put
/// beside the lock, or a note in a locked notebook holds before it is
/// sealed, is kept as it came, but isn't searched, linked or listed as
/// tasks (and the API doesn't return it, `Doc::to_served`).
fn index_note(
    tx: &Transaction,
    id: &str,
    fields: Option<&Map<String, Value>>,
    locked: bool,
    in_locked_notebook: bool,
) -> Result<()> {
    let blanked;
    let indexed = match fields {
        Some(fields)
            if locked
                && fields
                    .get("body")
                    .and_then(Value::as_str)
                    .is_some_and(|b| !b.is_empty()) =>
        {
            blanked = {
                let mut blank = fields.clone();
                blank.insert("body".into(), Value::String(String::new()));
                blank
            };
            Some(&blanked)
        }
        other => other,
    };
    tx.execute("DELETE FROM notes_fts WHERE note_id = ?", [id])?;
    if let Some(indexed) = indexed
        && indexed.get("bookId").and_then(Value::as_str) != Some(TRASH_BOOK_ID)
    {
        tx.execute(
            "INSERT INTO notes_fts (note_id, title, body) VALUES (?1, ?2, ?3)",
            params![
                id,
                indexed.get("title").and_then(Value::as_str).unwrap_or(""),
                indexed.get("body").and_then(Value::as_str).unwrap_or(""),
            ],
        )?;
    }
    crate::derived::write_note(tx, id, indexed, fields, in_locked_notebook)
}

/// Forgets what a locked note said before: every saved version not sealed
/// under `salt` (all of them, for a note in a locked notebook that isn't
/// sealed), and, when it was `newly` locked, its embedding. The write-ahead
/// log is emptied after the write when anything went.
fn forget_text(tx: &Transaction, id: &str, salt: Option<&str>, newly: bool) -> Result<()> {
    let forgotten = match salt {
        Some(salt) => tx.execute(
            "DELETE FROM revisions WHERE doc_id = ?1
             AND json_extract(json, '$.kansoLock.salt') IS NOT ?2",
            params![id, salt],
        )?,
        None => tx.execute("DELETE FROM revisions WHERE doc_id = ?", [id])?,
    };
    if newly {
        crate::embed::forget_note(tx, id)?;
    }
    if forgotten > 0 || newly {
        FORGOT_TEXT.set(true);
    }
    Ok(())
}

/// A note's notebook.
fn book_of(fields: &Map<String, Value>) -> Option<&str> {
    fields.get("bookId").and_then(Value::as_str)
}

/// Whether a note's plain text isn't empty.
fn has_text(fields: &Map<String, Value>) -> bool {
    fields
        .get("body")
        .and_then(Value::as_str)
        .is_some_and(|b| !b.is_empty())
}

/// Whether notebook `book_id` is locked, or inside a locked notebook.
pub(crate) fn in_locked_book(conn: &rusqlite::Connection, book_id: Option<&str>) -> Result<bool> {
    let Some(book_id) = book_id else {
        return Ok(false);
    };
    Ok(conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM locked_books WHERE id = ?)",
        [book_id],
        |row| row.get(0),
    )?)
}

/// The locked notebooks: each whose lock is set, and every notebook inside
/// one.
pub(crate) fn locked_book_ids(conn: &rusqlite::Connection) -> Result<BTreeSet<String>> {
    let mut stmt = conn.prepare("SELECT id FROM locked_books")?;
    let rows = stmt.query_map([], |row| row.get(0))?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

/// The plain text of a locked notebook stays in it until Kanso seals it:
/// nothing but sync may write plain text into a note in a locked notebook,
/// change or empty a note's text there without sealing it, or carry it out.
/// A note kept as it was (retagged, pinned, moved within the notebook) is
/// fine, and so is an empty one.
fn guard_note(
    tx: &Transaction,
    fields: &Map<String, Value>,
    previous: Option<&Row>,
    in_locked: bool,
) -> Result<()> {
    if fields.contains_key(LOCK_FIELD) {
        return Ok(());
    }
    let prev = previous.filter(|prev| !prev.deleted);
    let exposed = match prev {
        Some(prev) => {
            !prev.fields.contains_key(LOCK_FIELD)
                && has_text(&prev.fields)
                && in_locked_book_of(tx, &prev.fields)?
        }
        None => false,
    };
    let text_changed = prev.is_none_or(|prev| prev.fields.get("body") != fields.get("body"));
    let refused = if exposed {
        text_changed || !in_locked
    } else {
        in_locked && has_text(fields)
    };
    if refused {
        return Err(StoreError::Invalid(
            "The notebook is locked: its notes' text can only be changed in Kanso.".into(),
        ));
    }
    Ok(())
}

fn in_locked_book_of(tx: &Transaction, fields: &Map<String, Value>) -> Result<bool> {
    in_locked_book(tx, book_of(fields))
}

/// After a notebook was written: the notes of every notebook that became
/// locked, or stopped being locked, are indexed again as they now read. A
/// local write may not do this to a note with plain text ([`guard_note`]).
fn books_changed(tx: &Transaction, before: BTreeSet<String>, origin: Origin) -> Result<()> {
    let after = locked_book_ids(tx)?;
    // Only Kanso (or a lock taken off on another device) unlocks a
    // notebook: moving one out of a locked notebook would, too.
    if matches!(origin, Origin::Api | Origin::Import | Origin::Agent)
        && before.difference(&after).next().is_some()
    {
        return Err(StoreError::Invalid(
            "The notebook is locked: unlock it in Kanso first.".into(),
        ));
    }
    let moved: Vec<&String> = before.symmetric_difference(&after).collect();
    for book in moved {
        let notes: Vec<String> = tx
            .prepare(
                "SELECT id FROM docs WHERE type = 'note' AND deleted = 0 AND book_id = ? ORDER BY id",
            )?
            .query_map([book], |row| row.get(0))?
            .collect::<Result<_, _>>()?;
        let now_locked = after.contains(book);
        for note in notes {
            let Some(row) = load_row(tx, &note)? else {
                continue;
            };
            let salt = lock_salt(&row.fields);
            if origin != Origin::Sync && salt.is_none() && has_text(&row.fields) {
                return Err(StoreError::Invalid(if now_locked {
                    "The notebook is locked: notes with text move into it from Kanso, sealed."
                        .into()
                } else {
                    "The notebook is locked: unlock it in Kanso first.".into()
                }));
            }
            let locked = salt.is_some() || now_locked;
            if locked {
                forget_text(tx, &note, salt.as_deref(), now_locked && salt.is_none())?;
            }
            index_note(tx, &note, Some(&row.fields), locked, now_locked)?;
        }
    }
    Ok(())
}

thread_local! {
    /// Set by a write that overwrote a note's plain or old-key text, so the
    /// store empties the write-ahead log, where that text still is, once the
    /// write commits ([`forgot_text`]).
    // Clippy asks for the `const` this already has when it checks for Android.
    #[cfg_attr(target_os = "android", allow(clippy::missing_const_for_thread_local))]
    static FORGOT_TEXT: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Whether the write just made forgot a note's text; clears it.
pub(crate) fn forgot_text() -> bool {
    FORGOT_TEXT.replace(false)
}

/// The salt a locked note's text is sealed under; `None` when it isn't
/// locked.
fn lock_salt(fields: &Map<String, Value>) -> Option<String> {
    fields
        .get(LOCK_FIELD)?
        .get("salt")?
        .as_str()
        .map(str::to_owned)
}

fn snapshot(tx: &Transaction, id: &str, prev: &Row, origin: Origin) -> Result<()> {
    // A plugin's batch keeps every version itself, under the batch and by
    // the plugin's name (`plugin.rs`), never throttled.
    if origin == Origin::Plugin {
        return Ok(());
    }
    if matches!(origin, Origin::Ui | Origin::Collab) {
        let last: Option<i64> = tx.query_row(
            "SELECT MAX(saved_at) FROM revisions WHERE doc_id = ?",
            [id],
            |row| row.get(0),
        )?;
        if last.is_some_and(|at| now_ms() - at < UI_SNAPSHOT_INTERVAL_MS) {
            return Ok(());
        }
    }
    keep_snapshot(tx, id, prev, origin)
}

/// Puts `prev` in the document's history, unthrottled, then prunes it.
pub(crate) fn keep_snapshot(tx: &Transaction, id: &str, prev: &Row, origin: Origin) -> Result<()> {
    let now = crate::history::version_time(tx, id)?;
    // A shared note's version is by whoever changed it since the last one.
    let author = match prev.doc_type {
        DocType::Note => crate::collab::take_authors(tx, id)?,
        _ => None,
    };
    tx.execute(
        "INSERT INTO revisions (doc_id, rev, json, saved_at, origin, author) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            id,
            prev.rev,
            serde_json::to_string(&prev.fields)?,
            now,
            origin.as_str(),
            author
        ],
    )?;
    tx.execute(
        "DELETE FROM revisions WHERE doc_id = ?1 AND id NOT IN
         (SELECT id FROM revisions WHERE doc_id = ?1 ORDER BY saved_at DESC, id DESC LIMIT ?2)",
        params![id, SNAPSHOTS_PER_DOC],
    )?;
    Ok(())
}

/// Replaces live row `prev` with a tombstone and returns its revision. The
/// version it deletes always goes into history, however recent the last
/// snapshot: sync tells a replay of content this Mac held from another
/// Mac's edit it never saw by that history, and only the edit may beat the
/// deletion (docs/ACCOUNTS.md › Sync with encryption, review-verify 2).
pub(crate) fn delete_row(
    tx: &Transaction,
    id: &str,
    prev: &Row,
    origin: Origin,
    changes: &mut Vec<Change>,
) -> Result<String> {
    if origin == Origin::Ui {
        // `store_row`'s throttled snapshot then adds nothing.
        keep_snapshot(tx, id, prev, origin)?;
    }
    let rev = tombstone_rev(prev)?;
    store_row(
        tx,
        id,
        prev.doc_type,
        &rev,
        true,
        &Map::new(),
        Some(prev),
        Synced::Keep,
        origin,
        changes,
    )?;
    Ok(rev)
}

pub(crate) fn next_rev(previous: Option<&Row>, content: &Value) -> Result<String> {
    let previous = previous
        .map(|p| p.rev.parse::<Rev>())
        .transpose()
        .map_err(|e| StoreError::Invalid(e.to_string()))?;
    Ok(Rev::next(previous.as_ref(), content).to_string())
}

pub(crate) fn fresh_id(tx: &Transaction, doc_type: DocType) -> Result<String> {
    loop {
        let id = new_id(doc_type);
        let taken = tx
            .query_row("SELECT 1 FROM docs WHERE id = ?", [&id], |_| Ok(()))
            .optional()?
            .is_some();
        if !taken {
            return Ok(id);
        }
    }
}

fn tombstone_rev(previous: &Row) -> Result<String> {
    next_rev(Some(previous), &json!({ "deleted": true }))
}

impl Store {
    /// Creates, replaces or merges a document of `doc_type` from an API body.
    pub fn put(
        &self,
        doc_type: DocType,
        body: Value,
        mode: Put,
        origin: Origin,
    ) -> Result<WriteResult> {
        self.put_keeping(doc_type, body, mode, origin, &[])
    }

    /// [`Store::put`], except that the fields named in `kept` are never
    /// taken from `body`: the document keeps its current values for them (in
    /// the same transaction), and a new document gets none. The local API
    /// uses it for fields only the app may set.
    /// [`Store::put`] for an edit the agent called `agent` suggested and the
    /// person applied (`Origin::Agent`): the version it replaces is kept in
    /// the note's history, naming the agent. All in one transaction.
    pub fn put_by_agent(
        &self,
        doc_type: DocType,
        body: Value,
        mode: Put,
        agent: &str,
    ) -> Result<WriteResult> {
        let Value::Object(fields) = body else {
            return Err(StoreError::Invalid("Expected a JSON object.".into()));
        };
        self.write(|tx, changes| {
            let result = put_in(tx, changes, doc_type, fields, mode, Origin::Agent, PutRules::default())?;
            tx.execute(
                "UPDATE revisions SET agent = ?1 WHERE doc_id = ?2 AND origin = 'agent' AND agent IS NULL",
                params![agent, result.id],
            )?;
            Ok(result)
        })
    }

    pub fn put_keeping(
        &self,
        doc_type: DocType,
        body: Value,
        mode: Put,
        origin: Origin,
        kept: &[&str],
    ) -> Result<WriteResult> {
        let Value::Object(fields) = body else {
            return Err(StoreError::Invalid("Expected a JSON object.".into()));
        };
        self.write(|tx, changes| {
            put_in(
                tx,
                changes,
                doc_type,
                fields,
                mode,
                origin,
                PutRules {
                    kept,
                    ..PutRules::default()
                },
            )
        })
    }

    /// Writes `body`, a whole note the app has just sealed under the lock
    /// password, and forgets what the note said before: its saved versions
    /// (all in plain text until now), its embedding, and the pages SQLite
    /// freed for them, which are overwritten, and the write-ahead log, which
    /// is emptied once no reader holds it (`Store::write`). One transaction:
    /// there is no moment with the note locked and its old text still in
    /// history. Refused unless `body` is locked.
    pub fn lock_note(&self, body: Value) -> Result<WriteResult> {
        let Value::Object(fields) = body else {
            return Err(StoreError::Invalid("Expected a JSON object.".into()));
        };
        if !fields.contains_key(LOCK_FIELD) {
            return Err(StoreError::Invalid("The note isn't locked.".into()));
        }
        let result = self.write(|tx, changes| {
            validate_note_files(tx, &fields)?;
            let result = put_in(
                tx,
                changes,
                DocType::Note,
                fields,
                Put::Upsert,
                Origin::Ui,
                PutRules::default(),
            )?;
            tx.execute("DELETE FROM revisions WHERE doc_id = ?", [&result.id])?;
            crate::embed::forget_note(tx, &result.id)?;
            FORGOT_TEXT.set(true);
            Ok(result)
        })?;
        Ok(result)
    }

    /// Writes the deliberate transition from a locked note to a plain one.
    /// Only this path may keep a link to a sealed file still owned by another
    /// locked note; ordinary plain-note writes retain the sealed-file guard.
    pub fn unlock_note(&self, body: Value) -> Result<WriteResult> {
        let Value::Object(fields) = body else {
            return Err(StoreError::Invalid("Expected a JSON object.".into()));
        };
        let id = fields
            .get("_id")
            .and_then(Value::as_str)
            .ok_or_else(|| StoreError::Invalid("The note has no ID.".into()))?
            .to_owned();
        if fields.contains_key(LOCK_FIELD) {
            return Err(StoreError::Invalid("The note is still locked.".into()));
        }
        self.write(|tx, changes| {
            let previous = load_row(tx, &id)?.filter(|row| row.doc_type == DocType::Note && !row.deleted)
                .ok_or(StoreError::NotFound)?;
            if !previous.fields.contains_key(LOCK_FIELD)
                || fields.get("_rev").and_then(Value::as_str) != Some(previous.rev.as_str())
            {
                return Err(StoreError::Conflict);
            }
            let body = fields.get("body").and_then(Value::as_str).unwrap_or("");
            for file_id in linked_file_ids(body) {
                let sealed: bool = tx.query_row(
                    "SELECT EXISTS (SELECT 1 FROM docs WHERE id = ?1 AND type = 'file' AND deleted = 0
                     AND json_extract(json, '$.kansoFileLock') IS NOT NULL)",
                    [&file_id],
                    |row| row.get(0),
                )?;
                if !sealed { continue; }
                let other_owner: bool = tx.query_row(
                    "SELECT EXISTS (SELECT 1 FROM note_files nf JOIN docs d ON d.id = nf.note_id
                     WHERE nf.file_id = ?1 AND nf.note_id != ?2 AND d.type = 'note' AND d.deleted = 0)",
                    params![file_id, id],
                    |row| row.get(0),
                )?;
                if !other_owner {
                    return Err(StoreError::Invalid(
                        "A sealed attachment linked by an unlocked note must have another live owner.".into(),
                    ));
                }
            }
            put_in(tx, changes, DocType::Note, fields, Put::Upsert, Origin::Ui, PutRules { allow_sealed_note_links: true, ..PutRules::default() })
        })
    }

    /// Stores the attachment ownership list on a note. The app uses this
    /// while a note is still plain, before it seals the listed files and
    /// writes the locked note in the final transaction.
    pub fn set_note_files(&self, note_id: &str, files: &[NoteFile]) -> Result<WriteResult> {
        let doc = self
            .get(note_id)?
            .filter(|doc| doc.doc_type == DocType::Note)
            .ok_or(StoreError::NotFound)?;
        let mut fields = doc.fields;
        store_note_files(Some(files), &mut fields);
        self.put(
            DocType::Note,
            Value::Object({
                let mut body = fields;
                body.insert("_id".into(), Value::String(doc.id));
                body.insert("_rev".into(), Value::String(doc.rev));
                body
            }),
            Put::Upsert,
            Origin::Ui,
        )
    }

    /// Turns a sealed file back into a plain file after the app has opened
    /// its container. The marker names the seal generation so sync can
    /// distinguish this deliberate transition from an old plaintext replay.
    /// The plaintext must fit the ordinary attachment limit; the file's
    /// history and OCR text are dropped with the container.
    pub fn put_unsealed_file(&self, file: FileMeta, plaintext: Attachment) -> Result<WriteResult> {
        if plaintext.data.len() > MAX_ATTACHMENT_SIZE {
            return Err(StoreError::Invalid(format!(
                "The file is larger than {MAX_ATTACHMENT_SIZE} bytes."
            )));
        }
        let lock = checked_file_lock(&file)?;
        let Value::Object(mut fields) = serde_json::to_value(file)? else {
            return Err(StoreError::Invalid("Expected a JSON object.".into()));
        };
        let id = fields
            .get("_id")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| StoreError::Invalid("The file has no ID.".into()))?;
        let given_rev = fields
            .remove("_rev")
            .and_then(|v| v.as_str().map(str::to_owned));
        self.write(|tx, changes| {
            let previous = load_row(tx, &id)?;
            let Some(prev) = previous.as_ref().filter(|p| !p.deleted) else {
                return Err(StoreError::NotFound);
            };
            if prev.doc_type != DocType::File
                || FileLock::from_extra(&prev.fields).as_ref() != Some(&lock)
                || given_rev.as_deref() != Some(prev.rev.as_str())
            {
                return Err(StoreError::Conflict);
            }
            fields.remove(FILE_LOCK_FIELD);
            fields.remove(OCR_FIELD);
            fields.insert(UNSEALED_FIELD.into(), json!({"seal": lock.seal}));
            let mut normalized = normalize(
                DocType::File,
                &id,
                fields,
                Some(&prev.fields),
                now_ms(),
                Refs::Check(tx),
            )?;
            normalized.insert("contentLength".into(), plaintext.data.len().into());
            normalized.insert("md5digest".into(), plaintext.md5_hex().into());
            tx.execute(
                "INSERT INTO blobs (file_id, md5, data) VALUES (?1, ?2, ?3)
                 ON CONFLICT (file_id) DO UPDATE SET md5 = excluded.md5, data = excluded.data",
                params![id, plaintext.md5_hex(), plaintext.data],
            )?;
            let rev = next_rev(previous.as_ref(), &Value::Object(normalized.clone()))?;
            store_row(
                tx,
                &id,
                DocType::File,
                &rev,
                false,
                &normalized,
                previous.as_ref(),
                Synced::Keep,
                Origin::Ui,
                changes,
            )?;
            tx.execute("DELETE FROM revisions WHERE doc_id = ?", [&id])?;
            tx.execute("DELETE FROM file_text WHERE file_id = ?", [&id])?;
            FORGOT_TEXT.set(true);
            Ok(WriteResult { id, rev })
        })
    }

    /// Locks a notebook: writes `notes`, the notes in it the app has just
    /// sealed, as [`Store::lock_note`] does, then `book`, the whole notebook
    /// with its lock set, in one transaction. Every note in it then reads as
    /// locked; one with text left unsealed refuses the lock, so there is no
    /// moment with the notebook locked and a note's plain text readable.
    /// Refused unless `book` and every note in `notes` carry a lock.
    pub fn lock_book(&self, book: Value, notes: Vec<Value>) -> Result<WriteResult> {
        let Value::Object(book) = book else {
            return Err(StoreError::Invalid("Expected a JSON object.".into()));
        };
        if !book.contains_key(LOCK_FIELD) {
            return Err(StoreError::Invalid("The notebook isn't locked.".into()));
        }
        let mut sealed = Vec::with_capacity(notes.len());
        for note in notes {
            match note {
                Value::Object(note) if note.contains_key(LOCK_FIELD) => sealed.push(note),
                _ => return Err(StoreError::Invalid("A note isn't locked.".into())),
            }
        }
        self.write(|tx, changes| {
            for note in sealed {
                validate_note_files(tx, &note)?;
                let result = put_in(
                    tx,
                    changes,
                    DocType::Note,
                    note,
                    Put::Upsert,
                    Origin::Ui,
                    PutRules::default(),
                )?;
                tx.execute("DELETE FROM revisions WHERE doc_id = ?", [&result.id])?;
                crate::embed::forget_note(tx, &result.id)?;
                FORGOT_TEXT.set(true);
            }
            put_in(
                tx,
                changes,
                DocType::Book,
                book,
                Put::Upsert,
                Origin::Ui,
                PutRules::default(),
            )
        })
    }

    /// Writes a sealed attachment: the whole file document `file`, whose
    /// `kansoFileLock` marker must be set, with `container` as the stored
    /// bytes, in one transaction. The only write path that may create the
    /// marker or replace a sealed file's bytes — the ordinary [`Store::put`]
    /// path refuses both — so the app's lock steps seal through here. The
    /// stored `contentLength` and `md5digest` describe the container, and
    /// the container must fit [`MAX_SEALED_ATTACHMENT`]: the 10 MiB
    /// plaintext limit plus the container's overhead. The write-ahead log is
    /// checkpointed after the write ([`Store::write`]), as for a locked
    /// note's forgotten text, so the plaintext the container replaced
    /// doesn't linger there.
    pub fn put_sealed_file(&self, file: FileMeta, container: Attachment) -> Result<WriteResult> {
        checked_file_lock(&file)?;
        if container.data.len() > MAX_SEALED_ATTACHMENT {
            return Err(StoreError::Invalid(format!(
                "The sealed file is larger than {MAX_SEALED_ATTACHMENT} bytes."
            )));
        }
        let Value::Object(mut fields) = serde_json::to_value(file)? else {
            return Err(StoreError::Invalid("Expected a JSON object.".into()));
        };
        let id = fields
            .get("_id")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .ok_or_else(|| StoreError::Invalid("The file has no ID.".into()))?;
        if !is_valid_id(DocType::File, &id) {
            return Err(StoreError::Invalid(format!("Invalid document ID '{id}'.")));
        }
        let given_rev = fields
            .remove("_rev")
            .and_then(|v| v.as_str().map(str::to_owned));
        self.write(|tx, changes| {
            let previous = load_row(tx, &id)?;
            if let Some(prev) = &previous
                && prev.doc_type != DocType::File
            {
                return Err(StoreError::Conflict);
            }
            let live = previous.as_ref().filter(|p| !p.deleted);
            if let Some(prev) = live
                && given_rev.as_deref() != Some(prev.rev.as_str())
            {
                return Err(StoreError::Conflict);
            }
            fields.remove(OCR_FIELD);
            fields.remove(UNSEALED_FIELD);
            let mut normalized = normalize(
                DocType::File,
                &id,
                fields,
                live.map(|p| &p.fields),
                now_ms(),
                Refs::Check(tx),
            )?;
            normalized.insert("contentLength".into(), container.data.len().into());
            normalized.insert("md5digest".into(), container.md5_hex().into());
            tx.execute(
                "INSERT INTO blobs (file_id, md5, data) VALUES (?1, ?2, ?3)
                 ON CONFLICT (file_id) DO UPDATE SET md5 = excluded.md5, data = excluded.data",
                params![id, container.md5_hex(), container.data],
            )?;
            let rev = next_rev(previous.as_ref(), &Value::Object(normalized.clone()))?;
            store_row(
                tx,
                &id,
                DocType::File,
                &rev,
                false,
                &normalized,
                previous.as_ref(),
                Synced::Keep,
                Origin::Ui,
                changes,
            )?;
            tx.execute("DELETE FROM revisions WHERE doc_id = ?", [&id])?;
            tx.execute("DELETE FROM file_text WHERE file_id = ?", [&id])?;
            FORGOT_TEXT.set(true);
            Ok(WriteResult { id, rev })
        })
    }

    /// Merges each of `bodies` (with its `_id`) into its document of
    /// `doc_type`, in one transaction: every document changes, or none
    /// does. The app changes several selected notes at once with it.
    pub fn merge_many(
        &self,
        doc_type: DocType,
        bodies: Vec<Value>,
        origin: Origin,
    ) -> Result<Vec<WriteResult>> {
        let mut merges = Vec::with_capacity(bodies.len());
        for body in bodies {
            match body {
                Value::Object(fields) => merges.push(fields),
                _ => return Err(StoreError::Invalid("Expected a JSON object.".into())),
            }
        }
        self.write(|tx, changes| {
            merges
                .into_iter()
                .map(|fields| {
                    put_in(
                        tx,
                        changes,
                        doc_type,
                        fields,
                        Put::Merge,
                        origin,
                        PutRules::default(),
                    )
                })
                .collect()
        })
    }
}

/// A note may only be locked after every attachment in its ownership list
/// is a live sealed file with stored container bytes. A malformed list is a
/// refused lock, not an invitation to lock text whose files are still plain.
fn validate_note_files(tx: &Transaction, fields: &Map<String, Value>) -> Result<()> {
    if !fields.contains_key(KANSO_FILES_FIELD) {
        return Ok(());
    }
    let files = note_files_from_extra(fields)
        .ok_or_else(|| StoreError::Invalid("The note's attachment list is invalid.".into()))?;
    for file in files {
        if file.v != 1 || !is_valid_id(DocType::File, &file.file) {
            return Err(StoreError::Invalid(
                "The note's attachment list is invalid.".into(),
            ));
        }
        let sealed: bool = tx.query_row(
            "SELECT EXISTS (SELECT 1 FROM docs d JOIN blobs b ON b.file_id = d.id
             WHERE d.id = ?1 AND d.type = 'file' AND d.deleted = 0
             AND json_extract(d.json, '$.kansoFileLock') IS NOT NULL)",
            [&file.file],
            |row| row.get(0),
        )?;
        if !sealed {
            return Err(StoreError::Invalid(
                "Every attachment must be sealed before the note is locked.".into(),
            ));
        }
    }
    Ok(())
}

/// Checks the app-only `kansoFileLock` marker before a sealed write. The
/// marker is the only thing that tells readers `blobs.data` is a container,
/// so a malformed one must never reach storage through the dedicated writer.
fn checked_file_lock(file: &FileMeta) -> Result<FileLock> {
    let lock = FileLock::from_extra(&file.extra)
        .ok_or_else(|| StoreError::Invalid("The file isn't sealed.".into()))?;
    if lock.v != 1 || lock.kind != "file" || lock.meta {
        return Err(StoreError::Invalid(
            "The sealed file's marker is invalid.".into(),
        ));
    }
    let seal = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(&lock.seal)
        .map_err(|_| StoreError::Invalid("The sealed file's marker is invalid.".into()))?;
    if seal.len() != 16
        || base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&seal) != lock.seal
    {
        return Err(StoreError::Invalid(
            "The sealed file's marker is invalid.".into(),
        ));
    }
    Ok(lock)
}

/// How one write treats fields and links the caller doesn't own.
#[derive(Clone, Copy, Default)]
struct PutRules<'a> {
    /// Fields a merge keeps from the stored document.
    kept: &'a [&'a str],
    /// True only for the deliberate unlock of a note: a plain note may keep
    /// a link to a sealed file another locked note still owns.
    allow_sealed_note_links: bool,
}

/// [`Store::put_keeping`] inside the writer's transaction.
fn put_in(
    tx: &Transaction,
    changes: &mut Vec<Change>,
    doc_type: DocType,
    mut fields: Map<String, Value>,
    mode: Put,
    origin: Origin,
    rules: PutRules<'_>,
) -> Result<WriteResult> {
    let attachment = if doc_type == DocType::File {
        take_attachment(&mut fields)?
    } else {
        None
    };
    let given_id = fields.get("_id").and_then(Value::as_str).map(str::to_owned);
    let given_rev = fields
        .get("_rev")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let id = match given_id {
        Some(id) if is_valid_id(doc_type, &id) => id,
        Some(id) => {
            return Err(StoreError::Invalid(format!("Invalid document ID '{id}'.")));
        }
        None if mode == Put::Merge => return Err(StoreError::NotFound),
        None => fresh_id(tx, doc_type)?,
    };
    let previous = load_row(tx, &id)?;
    if let Some(prev) = &previous
        && prev.doc_type != doc_type
    {
        return Err(StoreError::Conflict);
    }
    let live = previous.as_ref().filter(|p| !p.deleted);
    let merged = match (mode, live) {
        (Put::Upsert, Some(prev)) => {
            if given_rev.as_deref() != Some(prev.rev.as_str()) {
                return Err(StoreError::Conflict);
            }
            fields
        }
        (Put::Upsert, None) => fields,
        (Put::Merge, None) => return Err(StoreError::NotFound),
        (Put::Merge, Some(prev)) => {
            if given_rev.as_ref().is_some_and(|rev| *rev != prev.rev) {
                return Err(StoreError::Conflict);
            }
            strip_reserved(&mut fields);
            let mut merged = prev.fields.clone();
            merged.extend(fields);
            merged
        }
    };
    let mut merged = merged;
    for &key in rules.kept {
        match live.and_then(|p| p.fields.get(key)) {
            Some(value) => merged.insert(key.to_owned(), value.clone()),
            None => merged.remove(key),
        };
    }
    // A sealed file's marker is app-only, set and cleared only by
    // `put_sealed_file` (and, in a later step, the unseal path). The
    // ordinary write path refuses new bytes for a sealed file, keeps its
    // marker on every merge or replace, and never creates one: a client or
    // an older writer can neither unseal a file nor plant the marker.
    if doc_type == DocType::File {
        if let Some(marker) = live
            .as_ref()
            .and_then(|p| p.fields.get(FILE_LOCK_FIELD))
            .cloned()
        {
            if attachment.is_some() {
                return Err(StoreError::Invalid(
                    "The file is sealed: its bytes can only be changed in Kanso.".into(),
                ));
            }
            merged.insert(FILE_LOCK_FIELD.to_owned(), marker);
            merged.remove(OCR_FIELD);
        } else {
            merged.remove(FILE_LOCK_FIELD);
        }
    }
    if doc_type == DocType::Note && origin == Origin::Import {
        merged.remove(KANSO_FILES_FIELD);
    }
    let mut normalized = normalize(
        doc_type,
        &id,
        merged,
        live.map(|p| &p.fields),
        now_ms(),
        if origin == Origin::Sync {
            Refs::Skip
        } else {
            Refs::Check(tx)
        },
    )?;
    // A locked note's text is only in its sealed `kansoLock`. Sync takes
    // what another device wrote, even an older Kanso's; nothing local may
    // put text beside the lock, where search and the API would read it.
    if doc_type == DocType::Note
        && origin != Origin::Sync
        && normalized.contains_key(LOCK_FIELD)
        && normalized
            .get("body")
            .and_then(Value::as_str)
            .is_some_and(|b| !b.is_empty())
    {
        return Err(StoreError::Invalid(
            "The note is locked: its text can only be changed in Kanso.".into(),
        ));
    }
    if doc_type == DocType::Note
        && !matches!(origin, Origin::Sync | Origin::Import)
        && !normalized.contains_key(LOCK_FIELD)
        && !in_locked_book(tx, book_of(&normalized))?
        && !rules.allow_sealed_note_links
    {
        let body = normalized.get("body").and_then(Value::as_str).unwrap_or("");
        for file_id in linked_file_ids(body) {
            let sealed: bool = tx.query_row(
                "SELECT EXISTS (SELECT 1 FROM docs WHERE id = ?1 AND type = 'file' AND deleted = 0
                 AND json_extract(json, '$.kansoFileLock') IS NOT NULL)",
                [&file_id],
                |row| row.get(0),
            )?;
            if sealed {
                return Err(StoreError::Invalid(
                    "That attachment belongs to a locked note.".into(),
                ));
            }
        }
    }

    if doc_type == DocType::File {
        let blob = match attachment {
            Some(blob) => {
                if blob.data.len() > MAX_ATTACHMENT_SIZE {
                    return Err(StoreError::Invalid(format!(
                        "The file is larger than {MAX_ATTACHMENT_SIZE} bytes."
                    )));
                }
                blob
            }
            None => match live {
                Some(_) => load_blob(tx, &id)?.ok_or(StoreError::NotFound)?,
                None => {
                    return Err(StoreError::Invalid(
                        "The file has no attachment data.".into(),
                    ));
                }
            },
        };
        normalized.insert("contentLength".into(), blob.data.len().into());
        normalized.insert("md5digest".into(), blob.md5_hex().into());
        tx.execute(
            "INSERT INTO blobs (file_id, md5, data) VALUES (?1, ?2, ?3)
             ON CONFLICT (file_id) DO UPDATE SET md5 = excluded.md5, data = excluded.data",
            params![id, blob.md5_hex(), blob.data],
        )?;
    }

    if let Some(prev) = live
        && prev.fields == normalized
    {
        return Ok(WriteResult {
            id,
            rev: prev.rev.clone(),
        });
    }
    let rev = next_rev(previous.as_ref(), &Value::Object(normalized.clone()))?;
    store_row(
        tx,
        &id,
        doc_type,
        &rev,
        false,
        &normalized,
        previous.as_ref(),
        Synced::Keep,
        origin,
        changes,
    )?;
    Ok(WriteResult { id, rev })
}

impl Store {
    /// Deletes a document with the local-server API's semantics: a notebook's notes move to
    /// the trash and its sub-notebooks are deleted; a tag is removed from its
    /// notes first; a note or file is deleted outright.
    pub fn delete(&self, id: &str, origin: Origin) -> Result<WriteResult> {
        self.write(|tx, changes| {
            let prev = load_row(tx, id)?
                .filter(|p| !p.deleted)
                .ok_or(StoreError::NotFound)?;
            let now = now_ms();
            match prev.doc_type {
                DocType::Book => {
                    let mut books = vec![id.to_owned()];
                    let mut i = 0;
                    while i < books.len() {
                        let mut stmt = tx.prepare(
                            "SELECT id FROM docs WHERE type = 'book' AND deleted = 0 AND parent_book_id = ?",
                        )?;
                        let children: Vec<String> = stmt
                            .query_map([&books[i]], |row| row.get(0))?
                            .collect::<Result<_, _>>()?;
                        books.extend(children);
                        i += 1;
                    }
                    for book in &books {
                        let notes: Vec<String> = tx
                            .prepare("SELECT id FROM docs WHERE type = 'note' AND deleted = 0 AND book_id = ?")?
                            .query_map([book], |row| row.get(0))?
                            .collect::<Result<_, _>>()?;
                        for note in notes {
                            rewrite_note(tx, &note, origin, changes, |fields| {
                                fields.insert("bookId".into(), TRASH_BOOK_ID.into());
                                fields.insert("updatedAt".into(), now.into());
                            })?;
                        }
                    }
                    for book in books.iter().skip(1) {
                        if let Some(child) = load_row(tx, book)? {
                            delete_row(tx, book, &child, origin, changes)?;
                        }
                    }
                }
                DocType::Tag => {
                    let notes: Vec<String> = tx
                        .prepare("SELECT note_id FROM note_tags WHERE tag_id = ?")?
                        .query_map([id], |row| row.get(0))?
                        .collect::<Result<_, _>>()?;
                    for note in notes {
                        rewrite_note(tx, &note, origin, changes, |fields| {
                            if let Some(Value::Array(tags)) = fields.get_mut("tags") {
                                tags.retain(|t| t.as_str() != Some(id));
                            }
                        })?;
                    }
                }
                DocType::Note | DocType::File => {}
            }
            let rev = delete_row(tx, id, &prev, origin, changes)?;
            Ok(WriteResult {
                id: id.to_owned(),
                rev,
            })
        })
    }

    /// Writes an imported document as-is, keeping its revision. Returns
    /// `false` when the store already has that revision, or a newer one made
    /// in Kanso. A replaced version is kept in the document's history.
    pub fn import_doc(
        &self,
        doc_type: DocType,
        id: &str,
        rev: &str,
        fields: Map<String, Value>,
        attachment: Option<Attachment>,
    ) -> Result<bool> {
        let incoming: Rev = rev
            .parse()
            .map_err(|e: kanso_core::rev::InvalidRev| StoreError::Invalid(e.to_string()))?;
        self.write(|tx, changes| {
            let previous = load_row(tx, id)?;
            if let Some(previous) = &previous {
                if previous.rev == rev && !previous.deleted {
                    return Ok(false);
                }
                // A copy edited or deleted in Kanso since the last import has
                // a generation at least as high; like CouchDB, it wins.
                let local: Option<Rev> = previous.rev.parse().ok();
                if local.is_some_and(|local| local.generation >= incoming.generation) {
                    return Ok(false);
                }
            }
            let created_at = fields
                .get("createdAt")
                .and_then(Value::as_i64)
                .unwrap_or_else(now_ms);
            // A sealed file's marker is app-only: an import takes the
            // document, never the seal.
            let mut fields = fields;
            fields.remove(FILE_LOCK_FIELD);
            if doc_type == DocType::Note {
                fields.remove(KANSO_FILES_FIELD);
            }
            let mut normalized = normalize(doc_type, id, fields, None, created_at, Refs::Skip)?;
            if let Some(blob) = &attachment {
                normalized.insert("contentLength".into(), blob.data.len().into());
                tx.execute(
                    "INSERT INTO blobs (file_id, md5, data) VALUES (?1, ?2, ?3)
                     ON CONFLICT (file_id) DO UPDATE SET md5 = excluded.md5, data = excluded.data",
                    params![id, blob.md5_hex(), blob.data],
                )?;
            }
            store_row(
                tx,
                id,
                doc_type,
                rev,
                false,
                &normalized,
                previous.as_ref(),
                Synced::Keep,
                Origin::Import,
                changes,
            )?;
            Ok(true)
        })
    }

    /// Keeps an imported document Kanso doesn't use, verbatim.
    pub fn archive(&self, id: &str, json: &Value) -> Result<()> {
        self.write(|tx, _| {
            tx.execute(
                "INSERT INTO archive (id, json) VALUES (?1, ?2)
                 ON CONFLICT (id) DO UPDATE SET json = excluded.json",
                params![id, json.to_string()],
            )?;
            Ok(())
        })
    }
}

/// Applies `edit` to a live note and stores it under a new revision.
pub(crate) fn rewrite_note(
    tx: &Transaction,
    id: &str,
    origin: Origin,
    changes: &mut Vec<Change>,
    edit: impl FnOnce(&mut Map<String, Value>),
) -> Result<()> {
    let Some(prev) = load_row(tx, id)?.filter(|p| !p.deleted) else {
        return Ok(());
    };
    let mut fields = prev.fields.clone();
    edit(&mut fields);
    let fields = normalize(
        DocType::Note,
        id,
        fields,
        Some(&prev.fields),
        now_ms(),
        Refs::Skip,
    )?;
    let rev = next_rev(Some(&prev), &Value::Object(fields.clone()))?;
    store_row(
        tx,
        id,
        DocType::Note,
        &rev,
        false,
        &fields,
        Some(&prev),
        Synced::Keep,
        origin,
        changes,
    )
}

pub(crate) fn load_blob(tx: &rusqlite::Connection, id: &str) -> Result<Option<Attachment>> {
    Ok(tx
        .query_row("SELECT data FROM blobs WHERE file_id = ?", [id], |row| {
            row.get::<_, Vec<u8>>(0)
        })
        .optional()?
        .map(Attachment::new))
}

/// Removes `_attachments` from a file body, returning the decoded bytes when
/// `index.data` is present.
fn take_attachment(fields: &mut Map<String, Value>) -> Result<Option<Attachment>> {
    let Some(attachments) = fields.remove("_attachments") else {
        return Ok(None);
    };
    let data = attachments
        .get("index")
        .and_then(|index| index.get("data"))
        .and_then(Value::as_str);
    let Some(data) = data else { return Ok(None) };
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(data.trim())
        .map_err(|_| StoreError::Invalid("The attachment data is not valid base64.".into()))?;
    if let Some(content_type) = attachments["index"]
        .get("content_type")
        .and_then(Value::as_str)
        && !fields.contains_key("contentType")
    {
        fields.insert("contentType".into(), content_type.into());
    }
    Ok(Some(Attachment::new(bytes)))
}
