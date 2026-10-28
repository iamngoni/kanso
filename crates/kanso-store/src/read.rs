//! Reads: single documents, lists with derived counts, and note search.

use std::collections::HashMap;

use kanso_core::excerpt::{HEAD_CHARS, excerpt_around};
use kanso_core::model::{FILE_LOCK_FIELD, TRASH_BOOK_ID};
use kanso_core::query::{Clause, Field};
use kanso_core::{Book, DocType, FileMeta, Note, NoteStatus, Query, Tag};
use rusqlite::types::Value as SqlValue;
use rusqlite::{Connection, OptionalExtension, Row, params_from_iter};
use serde_json::{Map, Value};

use crate::doc::{Attachment, Doc};
use crate::write::load_blob;
use crate::{Result, Store, StoreError};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Sort {
    #[default]
    UpdatedAt,
    CreatedAt,
    Title,
}

impl Sort {
    pub fn parse(value: &str) -> Option<Sort> {
        match value {
            "updatedAt" => Some(Sort::UpdatedAt),
            "createdAt" => Some(Sort::CreatedAt),
            "title" => Some(Sort::Title),
            _ => None,
        }
    }

    fn sql(self) -> &'static str {
        match self {
            Sort::UpdatedAt => "d.updated_at",
            Sort::CreatedAt => "d.created_at",
            Sort::Title => "d.title COLLATE NOCASE",
        }
    }
}
/// What a file's bytes are: the plain bytes, a sealed container, or nothing
/// at all (never made, or deleted). The app and the API read it before
/// touching an attachment.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AttachmentState {
    /// The bytes in `blobs` are the attachment's plaintext.
    #[default]
    Plain,
    /// The bytes in `blobs` are a sealed container, opened only with the
    /// note's lock password; the file document carries `kansoFileLock`.
    Sealed,
    /// There is no live file document with this ID.
    Missing,
}

#[derive(Clone, Debug)]
pub struct SearchParams {
    pub query: Query,
    pub sort: Sort,
    pub descending: bool,
    pub limit: Option<usize>,
    pub skip: usize,
    /// Free-text terms also match the text read from the images a note
    /// shows (`ocr.rs`). The app's searches ask for it; the local API's
    /// search keeps to titles and bodies.
    pub ocr: bool,
}

impl Default for SearchParams {
    fn default() -> Self {
        SearchParams {
            query: Query::default(),
            sort: Sort::UpdatedAt,
            descending: true,
            limit: None,
            skip: 0,
            ocr: false,
        }
    }
}

/// What the note list shows, without the full body.
#[derive(Clone, Debug, PartialEq)]
pub struct NoteSummary {
    pub id: String,
    pub rev: String,
    pub title: String,
    pub snippet: String,
    pub book_id: String,
    pub status: NoteStatus,
    pub tags: Vec<String>,
    pub pinned: bool,
    pub updated_at: i64,
    pub created_at: i64,
    /// Task-list items: how many are done, of how many.
    pub tasks_done: u32,
    pub tasks_total: u32,
    /// Whether the note's text is sealed under the lock password, or the
    /// note is in a locked notebook; its snippet is then empty.
    pub locked: bool,
}

/// Note counts for the sidebar.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StatusCounts {
    pub all: u32,
    pub pinned: u32,
    pub trash: u32,
    pub by_status: HashMap<NoteStatus, u32>,
}

/// A saved earlier version of a document.
#[derive(Clone, Debug, PartialEq)]
pub struct Revision {
    pub rev: String,
    pub saved_at: i64,
    pub origin: String,
    pub fields: Map<String, Value>,
    /// Who made this version of a shared note (names, comma-separated).
    pub author: Option<String>,
    /// The agent whose suggested edit, applied by the person, replaced this
    /// version (origin `agent`).
    pub agent: Option<String>,
}

const DOC_COLUMNS: &str = "d.id, d.type, d.rev, d.seq, d.deleted, d.json, \
    d.type = 'note' AND IFNULL(d.book_id, '') IN (SELECT id FROM locked_books)";

fn doc_from_row(row: &Row) -> rusqlite::Result<Doc> {
    let doc_type: String = row.get(1)?;
    let json: String = row.get(5)?;
    let fields = match serde_json::from_str(&json) {
        Ok(Value::Object(map)) => map,
        _ => Map::new(),
    };
    Ok(Doc {
        id: row.get(0)?,
        doc_type: DocType::from_name(&doc_type).unwrap_or(DocType::Note),
        rev: row.get(2)?,
        seq: row.get(3)?,
        deleted: row.get(4)?,
        fields,
        in_locked_notebook: row.get(6)?,
    })
}

fn live_doc(conn: &Connection, id: &str) -> Result<Option<Doc>> {
    Ok(conn
        .query_row(
            &format!("SELECT {DOC_COLUMNS} FROM docs d WHERE d.id = ? AND d.deleted = 0"),
            [id],
            doc_from_row,
        )
        .optional()?)
}

fn count_map(conn: &Connection, sql: &str) -> Result<HashMap<String, u32>> {
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?))
    })?;
    Ok(rows.collect::<rusqlite::Result<_>>()?)
}

const BOOK_COUNTS: &str = "SELECT book_id, COUNT(*) FROM docs
    WHERE type = 'note' AND deleted = 0 AND book_id IS NOT NULL GROUP BY book_id";
const TAG_COUNTS: &str = "SELECT nt.tag_id, COUNT(*) FROM note_tags nt
    JOIN docs d ON d.id = nt.note_id AND d.deleted = 0 AND d.book_id != 'trash'
    GROUP BY nt.tag_id";

fn with_count(count: u32) -> Map<String, Value> {
    let mut extra = Map::new();
    extra.insert("count".into(), count.into());
    extra
}

/// Builds the `WHERE` clause and parameters for a note search; with `ocr`,
/// a free-text term also matches a note that shows an image whose read
/// text contains it.
fn search_where(conn: &Connection, query: &Query, ocr: bool) -> Result<(String, Vec<SqlValue>)> {
    let includes_trash = query.includes_trash();
    let mut sql = String::from("d.type = 'note' AND d.deleted = 0");
    if !includes_trash {
        sql.push_str(" AND IFNULL(d.book_id, '') != 'trash'");
    }
    let mut params = Vec::new();
    for clause in &query.clauses {
        let mut expr = clause_sql(clause, !includes_trash, &mut params);
        if ocr && clause.field.is_none() && !clause.negated {
            let files = crate::ocr::files_matching(conn, &clause.value)?;
            if !files.is_empty() {
                let shows = files
                    .into_iter()
                    .map(|file| format!("instr({BODY}, {}) > 0", bind(&mut params, file)))
                    .collect::<Vec<_>>()
                    .join(" OR ");
                expr = format!("({expr}) OR {shows}");
            }
        }
        sql.push_str(if clause.negated { " AND NOT " } else { " AND " });
        sql.push_str(&format!("COALESCE(({expr}), 0)"));
    }
    Ok((sql, params))
}

const BODY: &str = crate::schema::readable_body!("d.");

fn bind(params: &mut Vec<SqlValue>, value: String) -> &'static str {
    params.push(SqlValue::Text(value));
    "?"
}

/// SQL for one clause, ignoring negation. For ASCII text of 3+ characters
/// the trigram index narrows the candidates; `kanso_contains` then decides,
/// so results match `Query::matches` exactly.
fn clause_sql(clause: &Clause, fts_allowed: bool, params: &mut Vec<SqlValue>) -> String {
    let v = &clause.value;
    let use_fts = fts_allowed && !clause.negated && v.is_ascii() && v.chars().count() >= 3;
    let prefilter = |columns: &str, params: &mut Vec<SqlValue>| -> String {
        if !use_fts {
            return String::new();
        }
        let q = bind(
            params,
            format!("{columns} : \"{}\"", v.replace('"', "\"\"")),
        );
        format!("d.id IN (SELECT note_id FROM notes_fts WHERE notes_fts MATCH {q}) AND ")
    };
    match clause.field {
        None => {
            let pre = prefilter("{title body}", params);
            let a = bind(params, v.clone());
            let b = bind(params, v.clone());
            format!("{pre}(kanso_contains(d.title, {a}) OR kanso_contains({BODY}, {b}))")
        }
        Some(Field::Title) => {
            let pre = prefilter("title", params);
            let a = bind(params, v.clone());
            format!("{pre}kanso_contains(d.title, {a})")
        }
        Some(Field::Body) => {
            let pre = prefilter("body", params);
            let a = bind(params, v.clone());
            format!("{pre}kanso_contains({BODY}, {a})")
        }
        Some(Field::Book) => {
            let a = bind(params, v.clone());
            let b = bind(params, v.clone());
            format!(
                "d.book_id IN (SELECT id FROM docs WHERE type = 'book' AND deleted = 0 \
                 AND lower(name) = lower({a})) OR (d.book_id = '{TRASH_BOOK_ID}' AND lower({b}) = '{TRASH_BOOK_ID}')"
            )
        }
        Some(Field::BookId) => {
            let id = bind(params, clause.book_id());
            format!("d.book_id = {id}")
        }
        Some(Field::Tag) => {
            // The tag or any tag inside it: `tag:work` finds `work/acme`
            // (kanso_core::tags::is_within).
            let name = bind(params, v.clone());
            let prefix = bind(params, v.clone());
            let len = bind(params, v.clone());
            format!(
                "EXISTS (SELECT 1 FROM note_tags nt JOIN docs t ON t.id = nt.tag_id AND t.deleted = 0 \
                 WHERE nt.note_id = d.id AND (lower(t.name) = lower({name}) \
                 OR lower(substr(t.name, 1, length({len}) + 1)) = lower({prefix}) || '/'))"
            )
        }
        Some(Field::TagId) => {
            let id = bind(params, kanso_core::id::with_prefix(DocType::Tag, v));
            format!("EXISTS (SELECT 1 FROM note_tags WHERE note_id = d.id AND tag_id = {id})")
        }
        Some(Field::Status) => {
            let status = bind(params, v.clone());
            format!("lower(d.status) = lower({status})")
        }
    }
}

fn search_sql(
    conn: &Connection,
    columns: &str,
    params: &SearchParams,
) -> Result<(String, Vec<SqlValue>)> {
    let (where_sql, mut values) = search_where(conn, &params.query, params.ocr)?;
    let direction = if params.descending { "DESC" } else { "ASC" };
    // Notes without a title read "Untitled"; they go after the named ones
    // in either direction rather than first.
    let blank_last = if params.sort == Sort::Title {
        "(trim(IFNULL(d.title, ''), ' ' || char(9) || char(10) || char(13)) = '') ASC, "
    } else {
        ""
    };
    let sql = format!(
        "SELECT {columns} FROM docs d WHERE {where_sql} ORDER BY {blank_last}{} {direction}, d.id {direction} LIMIT ? OFFSET ?",
        params.sort.sql()
    );
    values.push(SqlValue::Integer(params.limit.map_or(-1, |n| n as i64)));
    values.push(SqlValue::Integer(params.skip as i64));
    Ok((sql, values))
}

impl Store {
    /// Attachment IDs owned by a live note, including ownership retained
    /// while the note is in the trash. A deleted note has no ownership.
    pub fn files_of_note(&self, note_id: &str) -> Result<Vec<String>> {
        self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT nf.file_id FROM note_files nf JOIN docs d ON d.id = nf.note_id
                 WHERE nf.note_id = ?1 AND d.type = 'note' AND d.deleted = 0
                 ORDER BY nf.file_id",
            )?;
            Ok(stmt
                .query_map([note_id], |row| row.get(0))?
                .collect::<rusqlite::Result<_>>()?)
        })
    }

    /// Live note IDs that own `file_id`, including notes in the trash.
    pub fn file_owners(&self, file_id: &str) -> Result<Vec<String>> {
        self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT nf.note_id FROM note_files nf JOIN docs d ON d.id = nf.note_id
                 WHERE nf.file_id = ?1 AND d.type = 'note' AND d.deleted = 0
                 ORDER BY nf.note_id",
            )?;
            Ok(stmt
                .query_map([file_id], |row| row.get(0))?
                .collect::<rusqlite::Result<_>>()?)
        })
    }

    /// The live document with `id`.
    pub fn get(&self, id: &str) -> Result<Option<Doc>> {
        self.read(|conn| live_doc(conn, id))
    }

    /// A document as the HTTP API returns it. With `rev`, only that exact
    /// current revision is found. With `attachments`, files carry their bytes.
    pub fn get_api(&self, id: &str, rev: Option<&str>, attachments: bool) -> Result<Value> {
        self.read(|conn| {
            let doc = live_doc(conn, id)?.ok_or(StoreError::NotFound)?;
            if rev.is_some_and(|rev| rev != doc.rev) {
                return Err(StoreError::NotFound);
            }
            Ok(match doc.doc_type {
                DocType::Note => doc.to_served(Map::new()),
                DocType::Book => {
                    let counts = count_map(conn, BOOK_COUNTS)?;
                    doc.to_served(with_count(counts.get(id).copied().unwrap_or(0)))
                }
                DocType::Tag => {
                    let counts = count_map(conn, TAG_COUNTS)?;
                    doc.to_served(with_count(counts.get(id).copied().unwrap_or(0)))
                }
                DocType::File => {
                    let blob = load_blob(conn, id)?.unwrap_or_else(|| Attachment::new(Vec::new()));
                    let content_type = doc.str_field("contentType").unwrap_or_default().to_owned();
                    doc.file_to_api(&content_type, &blob, attachments)
                }
            })
        })
    }

    /// All live notebooks, tags or files in API form.
    pub fn list_api(
        &self,
        doc_type: DocType,
        limit: Option<usize>,
        skip: usize,
    ) -> Result<Vec<Value>> {
        self.list(doc_type, limit, skip, true)
    }

    /// [`Store::list_api`], as served (`served`) or as the app reads it,
    /// with a notebook's lock.
    fn list(
        &self,
        doc_type: DocType,
        limit: Option<usize>,
        skip: usize,
        served: bool,
    ) -> Result<Vec<Value>> {
        let shown = |doc: Doc, extra: Map<String, Value>| {
            if served {
                doc.to_served(extra)
            } else {
                doc.to_api(extra)
            }
        };
        self.read(|conn| {
            let order = match doc_type {
                DocType::Book | DocType::Tag => "d.name COLLATE NOCASE, d.id",
                DocType::Note => "d.updated_at DESC, d.id",
                DocType::File => "d.created_at, d.id",
            };
            let mut stmt = conn.prepare(&format!(
                "SELECT {DOC_COLUMNS} FROM docs d WHERE d.type = ? AND d.deleted = 0 ORDER BY {order} LIMIT ? OFFSET ?"
            ))?;
            let docs: Vec<Doc> = stmt
                .query_map(
                    rusqlite::params![doc_type.as_str(), limit.map_or(-1, |n| n as i64), skip as i64],
                    doc_from_row,
                )?
                .collect::<rusqlite::Result<_>>()?;
            let counts = match doc_type {
                DocType::Book => count_map(conn, BOOK_COUNTS)?,
                DocType::Tag => count_map(conn, TAG_COUNTS)?,
                _ => HashMap::new(),
            };
            docs.into_iter()
                .map(|doc| match doc.doc_type {
                    DocType::Book | DocType::Tag => {
                        let count = counts.get(&doc.id).copied().unwrap_or(0);
                        Ok(shown(doc, with_count(count)))
                    }
                    DocType::File => {
                        let blob = load_blob(conn, &doc.id)?.unwrap_or_else(|| Attachment::new(Vec::new()));
                        let content_type = doc.str_field("contentType").unwrap_or_default().to_owned();
                        Ok(doc.file_to_api(&content_type, &blob, false))
                    }
                    DocType::Note => Ok(shown(doc, Map::new())),
                })
                .collect()
        })
    }

    /// Notes matching a search, with full bodies.
    pub fn search(&self, params: &SearchParams) -> Result<Vec<Doc>> {
        self.read(|conn| {
            let (sql, values) = search_sql(conn, DOC_COLUMNS, params)?;
            let mut stmt = conn.prepare(&sql)?;
            let docs = stmt
                .query_map(params_from_iter(values), doc_from_row)?
                .collect::<rusqlite::Result<_>>()?;
            Ok(docs)
        })
    }

    /// Notes matching a search, in API form.
    pub fn search_api(&self, params: &SearchParams) -> Result<Vec<Value>> {
        Ok(self
            .search(params)?
            .into_iter()
            .map(|doc| doc.to_served(Map::new()))
            .collect())
    }

    /// Notes matching a search, as list rows with a short snippet. For a
    /// search with words to find in the body, the snippet starts near the
    /// first of them, so the list can show why a note matched.
    pub fn search_summaries(&self, params: &SearchParams) -> Result<Vec<NoteSummary>> {
        let terms: Vec<&str> = params.query.terms_for(Field::Body).collect();
        self.read(|conn| {
            let window = if terms.is_empty() {
                "NULL".to_owned()
            } else {
                format!("kanso_window({BODY}, ?)")
            };
            let locked = crate::schema::note_locked!("d.");
            // A note in a locked notebook counts no tasks, as a sealed one.
            let columns = format!(
                "d.id, d.rev, d.title, substr({BODY}, 1, {HEAD_CHARS}), \
                 d.book_id, d.status, json_extract(d.json, '$.tags'), d.pinned, d.updated_at, d.created_at, \
                 {window}, CASE WHEN {locked} THEN 0 ELSE json_extract(d.json, '$.numOfCheckedTasks') END, \
                 CASE WHEN {locked} THEN 0 ELSE json_extract(d.json, '$.numOfTasks') END, {locked}"
            );
            let (sql, mut values) = search_sql(conn, &columns, params)?;
            // The window's term comes first: its column is before the rest.
            if let Some(first) = terms.first() {
                values.insert(0, SqlValue::Text((*first).to_owned()));
            }
            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt.query_map(params_from_iter(values), |row| {
                let tags: Option<String> = row.get(6)?;
                let status: Option<String> = row.get(5)?;
                let title = row.get::<_, Option<String>>(2)?.unwrap_or_default();
                let head = row.get::<_, Option<String>>(3)?.unwrap_or_default();
                let window = row.get::<_, Option<String>>(10)?;
                Ok(NoteSummary {
                    id: row.get(0)?,
                    rev: row.get(1)?,
                    snippet: excerpt_around(&title, &head, window.as_deref(), &terms),
                    title,
                    book_id: row.get::<_, Option<String>>(4)?.unwrap_or_default(),
                    status: status.as_deref().and_then(NoteStatus::parse).unwrap_or_default(),
                    tags: tags
                        .and_then(|t| serde_json::from_str(&t).ok())
                        .unwrap_or_default(),
                    pinned: row.get::<_, Option<bool>>(7)?.unwrap_or(false),
                    updated_at: row.get::<_, Option<i64>>(8)?.unwrap_or(0),
                    created_at: row.get::<_, Option<i64>>(9)?.unwrap_or(0),
                    tasks_done: row.get::<_, Option<u32>>(11)?.unwrap_or(0),
                    tasks_total: row.get::<_, Option<u32>>(12)?.unwrap_or(0),
                    locked: row.get(13)?,
                })
            })?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }

    /// How many live notes outside the trash changed at or after
    /// `since_ms`: what Recent lists.
    pub fn count_recent(&self, since_ms: i64) -> Result<u32> {
        self.read(|conn| {
            Ok(conn.query_row(
                "SELECT COUNT(*) FROM docs WHERE type = 'note' AND deleted = 0 \
                 AND IFNULL(book_id, '') != 'trash' AND updated_at >= ?",
                [since_ms],
                |row| row.get(0),
            )?)
        })
    }

    pub fn note(&self, id: &str) -> Result<Option<Note>> {
        self.typed(id)
    }

    /// Whether note `id` is read as locked: sealed itself, or in a locked
    /// notebook (or one inside it). `false` for no such live note.
    pub fn note_is_locked(&self, id: &str) -> Result<bool> {
        self.read(|conn| {
            Ok(conn
                .query_row(
                    concat!(
                        "SELECT ",
                        crate::schema::note_locked!(""),
                        " FROM docs WHERE id = ? AND type = 'note' AND deleted = 0"
                    ),
                    [id],
                    |row| row.get(0),
                )
                .optional()?
                .unwrap_or(false))
        })
    }

    /// Whether live note `id` is in a locked notebook (or one inside it).
    pub fn note_in_locked_book(&self, id: &str) -> Result<bool> {
        self.read(|conn| {
            Ok(conn
                .query_row(
                    "SELECT IFNULL(book_id, '') IN (SELECT id FROM locked_books)
                     FROM docs WHERE id = ? AND type = 'note' AND deleted = 0",
                    [id],
                    |row| row.get(0),
                )
                .optional()?
                .unwrap_or(false))
        })
    }

    /// The locked notebooks: each one whose lock is set, and every notebook
    /// inside one.
    pub fn locked_books(&self) -> Result<std::collections::HashSet<String>> {
        self.read(|conn| crate::write::locked_book_ids(conn).map(|ids| ids.into_iter().collect()))
    }

    /// The live notes in locked notebooks whose text isn't sealed yet: made
    /// there empty, or put there by another device or an older Kanso. The
    /// app seals them once it holds the key.
    pub fn unsealed_in_locked_books(&self) -> Result<Vec<Note>> {
        let ids: Vec<String> = self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id FROM docs WHERE type = 'note' AND deleted = 0
                 AND json_extract(json, '$.kansoLock') IS NULL
                 AND IFNULL(book_id, '') IN (SELECT id FROM locked_books) ORDER BY id",
            )?;
            let rows = stmt.query_map([], |row| row.get(0))?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })?;
        ids.iter()
            .filter_map(|id| self.note(id).transpose())
            .collect()
    }

    /// The live notes in `books`, as the app reads them: what locking a
    /// notebook seals.
    pub fn notes_in_books(&self, books: &[String]) -> Result<Vec<Note>> {
        let ids: Vec<String> = self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id FROM docs WHERE type = 'note' AND deleted = 0 AND book_id = ? ORDER BY id",
            )?;
            let mut ids = Vec::new();
            for book in books {
                let rows = stmt.query_map([book], |row| row.get::<_, String>(0))?;
                for id in rows {
                    ids.push(id?);
                }
            }
            Ok(ids)
        })?;
        ids.iter()
            .filter_map(|id| self.note(id).transpose())
            .collect()
    }

    /// Every live locked note, in the trash too: what changing the lock
    /// password seals again.
    pub fn locked_notes(&self) -> Result<Vec<Note>> {
        let ids: Vec<String> = self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id FROM docs WHERE type = 'note' AND deleted = 0
                 AND json_extract(json, '$.kansoLock') IS NOT NULL ORDER BY id",
            )?;
            let rows = stmt.query_map([], |row| row.get(0))?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })?;
        ids.iter()
            .filter_map(|id| self.note(id).transpose())
            .collect()
    }

    pub fn file_meta(&self, id: &str) -> Result<Option<FileMeta>> {
        self.typed(id)
    }

    fn typed<T: serde::de::DeserializeOwned>(&self, id: &str) -> Result<Option<T>> {
        let Some(doc) = self.get(id)? else {
            return Ok(None);
        };
        Ok(Some(serde_json::from_value(doc.to_api(Map::new()))?))
    }

    /// All live notebooks with note counts, sorted by name.
    pub fn books(&self) -> Result<Vec<Book>> {
        self.list(DocType::Book, None, 0, false)?
            .into_iter()
            .map(|v| Ok(serde_json::from_value(v)?))
            .collect()
    }

    /// All live tags with note counts, sorted by name.
    pub fn tags(&self) -> Result<Vec<Tag>> {
        self.list_api(DocType::Tag, None, 0)?
            .into_iter()
            .map(|v| Ok(serde_json::from_value(v)?))
            .collect()
    }

    /// Whether a file's document is live and plain, live and sealed, or
    /// gone (`Missing` covers a file that was never made and one deleted).
    /// The API and the app's readers check this before touching bytes: a
    /// sealed file's stored bytes are a container, never the plaintext.
    pub fn attachment_state(&self, id: &str) -> Result<AttachmentState> {
        self.read(|conn| {
            let Some(doc) = live_doc(conn, id)?.filter(|d| d.doc_type == DocType::File) else {
                return Ok(AttachmentState::Missing);
            };
            Ok(if doc.fields.contains_key(FILE_LOCK_FIELD) {
                AttachmentState::Sealed
            } else {
                AttachmentState::Plain
            })
        })
    }

    /// The attachment's bytes as stored in `blobs`: for a sealed file, the
    /// raw container (with its digest), never the plaintext. Sync's upload
    /// reads this, so the sealed bytes travel as they are; the app's
    /// unsealing path is the only reader that decrypts.
    pub fn stored_attachment(&self, id: &str) -> Result<Option<Attachment>> {
        self.read(|conn| {
            if live_doc(conn, id)?
                .filter(|d| d.doc_type == DocType::File)
                .is_none()
            {
                return Ok(None);
            }
            load_blob(conn, id)
        })
    }

    /// An attachment's content type and bytes. A sealed file returns `None`:
    /// its stored bytes are a container, and only the app's lock-aware helper
    /// may open them. Sync uses [`Store::stored_attachment`] for the raw bytes.
    pub fn attachment(&self, id: &str) -> Result<Option<(String, Attachment)>> {
        self.read(|conn| {
            let Some(doc) = live_doc(conn, id)?.filter(|d| d.doc_type == DocType::File) else {
                return Ok(None);
            };
            if doc.fields.contains_key(FILE_LOCK_FIELD) {
                return Ok(None);
            }
            let content_type = doc.str_field("contentType").unwrap_or_default().to_owned();
            Ok(load_blob(conn, id)?.map(|blob| (content_type, blob)))
        })
    }

    /// Note counts for the sidebar.
    pub fn status_counts(&self) -> Result<StatusCounts> {
        self.read(|conn| {
            let mut counts = StatusCounts::default();
            let mut stmt = conn.prepare(
                "SELECT book_id = 'trash', status, pinned, COUNT(*) FROM docs
                 WHERE type = 'note' AND deleted = 0 GROUP BY 1, 2, 3",
            )?;
            let rows = stmt.query_map([], |row| {
                Ok((
                    row.get::<_, Option<bool>>(0)?.unwrap_or(false),
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<bool>>(2)?.unwrap_or(false),
                    row.get::<_, u32>(3)?,
                ))
            })?;
            for row in rows {
                let (trash, status, pinned, n) = row?;
                if trash {
                    counts.trash += n;
                    continue;
                }
                counts.all += n;
                if pinned {
                    counts.pinned += n;
                }
                let status = status
                    .as_deref()
                    .and_then(NoteStatus::parse)
                    .unwrap_or_default();
                *counts.by_status.entry(status).or_default() += n;
            }
            Ok(counts)
        })
    }

    /// Saved earlier versions of a document, newest first.
    pub fn revisions(&self, id: &str) -> Result<Vec<Revision>> {
        self.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT rev, saved_at, origin, json, author, agent FROM revisions WHERE doc_id = ? AND json != 'null' ORDER BY saved_at DESC, id DESC",
            )?;
            let rows = stmt.query_map([id], |row| {
                let json: String = row.get(3)?;
                Ok(Revision {
                    rev: row.get(0)?,
                    saved_at: row.get(1)?,
                    origin: row.get(2)?,
                    fields: match serde_json::from_str(&json) {
                        Ok(Value::Object(map)) => map,
                        _ => Map::new(),
                    },
                    author: row.get(4)?,
                    agent: row.get(5)?,
                })
            })?;
            Ok(rows.collect::<rusqlite::Result<_>>()?)
        })
    }

    /// Documents changed after `since`, including deletions, oldest first.
    pub fn changes_since(
        &self,
        since: i64,
        limit: Option<usize>,
        descending: bool,
    ) -> Result<Vec<Doc>> {
        self.read(|conn| {
            let direction = if descending { "DESC" } else { "ASC" };
            let mut stmt = conn.prepare(&format!(
                "SELECT {DOC_COLUMNS} FROM docs d WHERE d.seq > ? ORDER BY d.seq {direction} LIMIT ?"
            ))?;
            let docs = stmt
                .query_map(rusqlite::params![since, limit.map_or(-1, |n| n as i64)], doc_from_row)?
                .collect::<rusqlite::Result<_>>()?;
            Ok(docs)
        })
    }

    /// The highest change sequence number.
    pub fn last_seq(&self) -> Result<i64> {
        self.read(crate::schema::last_seq)
    }

    /// Live documents per type.
    pub fn counts(&self) -> Result<HashMap<DocType, usize>> {
        self.read(|conn| {
            let mut stmt =
                conn.prepare("SELECT type, COUNT(*) FROM docs WHERE deleted = 0 GROUP BY type")?;
            let rows = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize))
            })?;
            let mut counts = HashMap::new();
            for row in rows {
                let (doc_type, n) = row?;
                if let Some(doc_type) = DocType::from_name(&doc_type) {
                    counts.insert(doc_type, n);
                }
            }
            Ok(counts)
        })
    }
}
