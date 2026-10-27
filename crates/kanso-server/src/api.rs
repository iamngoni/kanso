//! Kanso's HTTP API operations, shared by the REST handlers and the
//! native MCP tools so both answer identically. All of them are synchronous;
//! callers run them on a blocking thread.

use kanso_core::{DocType, Query};
use kanso_store::{Origin, Put, SearchParams, Sort, Store, WriteResult};
use serde_json::{Value, json};

use crate::error::{ApiError, ApiResult};

/// Notes returned by `GET /notes` when no `limit` is given.
pub(crate) const DEFAULT_NOTE_LIMIT: usize = 50;

/// The `{ok, id, rev}` answer to every write.
pub(crate) fn write_response(result: WriteResult) -> Value {
    json!({ "ok": true, "id": result.id, "rev": result.rev })
}

/// `GET /<id>`: a live document, optionally only at revision `rev`.
pub(crate) fn get_doc(
    store: &Store,
    id: &str,
    rev: Option<&str>,
    attachments: bool,
) -> ApiResult<Value> {
    if DocType::of_id(id).is_none() {
        return Err(ApiError::not_found());
    }
    Ok(store.get_api(id, rev, attachments)?)
}

/// The parameters of `GET /notes`.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NoteQuery {
    pub keyword: String,
    pub limit: Option<usize>,
    pub skip: usize,
    pub sort: Sort,
    pub descending: bool,
}

impl Default for NoteQuery {
    fn default() -> Self {
        NoteQuery {
            keyword: String::new(),
            limit: Some(DEFAULT_NOTE_LIMIT),
            skip: 0,
            sort: Sort::UpdatedAt,
            descending: true,
        }
    }
}

/// `GET /notes`: notes matching a keyword, with full bodies.
pub(crate) fn list_notes(store: &Store, query: &NoteQuery) -> ApiResult<Vec<Value>> {
    let params = SearchParams {
        query: Query::parse(&query.keyword),
        sort: query.sort,
        descending: query.descending,
        limit: query.limit,
        skip: query.skip,
        ocr: false,
    };
    Ok(store.search_api(&params)?)
}

/// `GET /books`, `/tags`, `/files`.
pub(crate) fn list_docs(
    store: &Store,
    doc_type: DocType,
    limit: Option<usize>,
    skip: usize,
) -> ApiResult<Vec<Value>> {
    Ok(store.list_api(doc_type, limit, skip)?)
}

/// Fields of a document the local API and MCP may not set: a note's or a
/// notebook's published link and key, which only the app's Publish sheets
/// write. A
/// write keeps the document's current value and ignores the one it is sent,
/// so a client can't plant a link (and a key it knows) for the app to
/// publish to, or move a published note's link elsewhere. Also the
/// reminders the app was told about (`kansoReminderAck`), which a client
/// replacing the whole note would otherwise drop, so they would ring again.
/// And a locked note's sealed text (`kansoLock`): a client reads a locked
/// note with an empty body and can't lock, unlock or replace its text (the
/// store refuses text beside the lock). A notebook's lock likewise: a client
/// can't lock or unlock a notebook, nor write text into the notes of a
/// locked one (the store refuses that too). And a shared note's mark
/// (`kansoShare`): only Kanso shares a note, and a client's writes to a
/// shared note's text reach the other members through the CRDT (the store
/// refuses them from a member who may only view or comment).
fn app_only_fields(doc_type: DocType) -> &'static [&'static str] {
    match doc_type {
        DocType::Note => &[
            kanso_core::model::PUBLISH_FIELD,
            kanso_core::reminders::ACK_FIELD,
            kanso_core::model::LOCK_FIELD,
            kanso_core::model::SHARE_FIELD,
        ],
        // A notebook's lock, and a published notebook's link and key.
        DocType::Book => &[
            kanso_core::model::LOCK_FIELD,
            kanso_core::model::PUBLISH_FIELD,
        ],
        // The words the app read in an image: a write of the file keeps them.
        DocType::File => &[kanso_core::model::OCR_FIELD],
        _ => &[],
    }
}

/// `POST /notes` and friends: create, or replace when `_id` exists.
pub(crate) fn upsert(store: &Store, doc_type: DocType, body: Value) -> ApiResult<Value> {
    let mut body = body;
    if doc_type == DocType::Note && body.get("bookId").and_then(Value::as_str) == Some("template") {
        let book = store.role_book(kanso_core::roles::Role::Templates, Origin::Api)?;
        if let Some(fields) = body.as_object_mut() {
            fields.insert("bookId".into(), Value::String(book));
        }
    }
    Ok(write_response(store.put_keeping(
        doc_type,
        body,
        Put::Upsert,
        Origin::Api,
        app_only_fields(doc_type),
    )?))
}

/// `POST /<id>`: merge the given fields into the document.
pub(crate) fn merge(store: &Store, id: &str, body: Value) -> ApiResult<Value> {
    let doc_type = DocType::of_id(id).ok_or_else(ApiError::not_found)?;
    let Value::Object(mut fields) = body else {
        return Err(ApiError::bad_request("Expected a JSON object."));
    };
    fields.insert("_id".into(), Value::String(id.to_owned()));
    Ok(write_response(store.put_keeping(
        doc_type,
        Value::Object(fields),
        Put::Merge,
        Origin::Api,
        app_only_fields(doc_type),
    )?))
}

/// `DELETE /<id>`. A locked note, or a note in a locked notebook, can't be
/// deleted outright from here: its text would be gone without anyone who
/// knows the password agreeing. Moving it to the trash (`bookId: "trash"`)
/// still works for a sealed one. Nor can a locked notebook, or one inside
/// it, be deleted from here.
pub(crate) fn delete(store: &Store, id: &str) -> ApiResult<Value> {
    if DocType::of_id(id).is_none() {
        return Err(ApiError::not_found());
    }
    if DocType::of_id(id) == Some(DocType::Note) && store.note_is_locked(id)? {
        return Err(ApiError::bad_request(
            "The note is locked: delete it in Kanso.",
        ));
    }
    if DocType::of_id(id) == Some(DocType::Book) && store.locked_books()?.contains(id) {
        return Err(ApiError::bad_request(
            "The notebook is locked: delete it in Kanso.",
        ));
    }
    Ok(write_response(store.delete(id, Origin::Api)?))
}

/// Where `GET /_changes` starts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Since {
    Seq(i64),
    Now,
}

/// `GET /_changes`, in CouchDB's shape.
pub(crate) fn changes(
    store: &Store,
    since: Since,
    limit: Option<usize>,
    descending: bool,
    include_docs: bool,
) -> ApiResult<Value> {
    let since = match since {
        Since::Seq(seq) => seq,
        Since::Now => store.last_seq()?,
    };
    let docs = store.changes_since(since, limit, descending)?;
    let last_seq = match docs.last() {
        Some(doc) => doc.seq,
        None => store.last_seq()?,
    };
    let results: Vec<Value> = docs
        .into_iter()
        .map(|doc| {
            let mut row = json!({
                "id": doc.id,
                "seq": doc.seq,
                "changes": [{ "rev": doc.rev }],
            });
            if doc.deleted {
                row["deleted"] = Value::Bool(true);
            }
            if include_docs {
                row["doc"] = if doc.deleted {
                    json!({ "_id": doc.id, "_rev": doc.rev, "_deleted": true })
                } else {
                    doc.to_served(Default::default())
                };
            }
            row
        })
        .collect();
    Ok(json!({ "results": results, "last_seq": last_seq }))
}

/// A count parameter (`limit`, `skip`): a non-negative number, truncated.
pub(crate) fn count_param(name: &str, value: f64) -> ApiResult<usize> {
    if value.is_finite() && value >= 0.0 {
        Ok(value.trunc() as usize)
    } else {
        Err(ApiError::bad_request(format!(
            "'{name}' must be a non-negative number."
        )))
    }
}

/// [`count_param`] for a query-string value.
pub(crate) fn parse_count(name: &str, raw: &str) -> ApiResult<usize> {
    let value: f64 = raw
        .trim()
        .parse()
        .map_err(|_| ApiError::bad_request(format!("'{name}' must be a non-negative number.")))?;
    count_param(name, value)
}

/// A boolean query-string value.
pub(crate) fn parse_bool(name: &str, raw: &str) -> ApiResult<bool> {
    match raw.trim() {
        "true" | "1" => Ok(true),
        "false" | "0" => Ok(false),
        _ => Err(ApiError::bad_request(format!(
            "'{name}' must be true or false."
        ))),
    }
}

/// A `sort` value; anything Kanso doesn't sort by (`tagCount`, `rank`, …)
/// falls back to `updatedAt`.
pub(crate) fn parse_sort(raw: &str) -> Sort {
    Sort::parse(raw).unwrap_or(Sort::UpdatedAt)
}
