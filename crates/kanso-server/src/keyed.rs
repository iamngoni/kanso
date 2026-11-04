//! What an API key may see and change (issue #243).
//!
//! Every key reads through the same [`Visibility`] an agent does: a locked
//! note, a note in a locked notebook, a trashed note and a note tagged
//! `#ai-exclude` read as "not found", and so does anything outside the key's
//! notebooks. Every document a key is shown goes through
//! [`scope::strip`], so no publish, share or lock fields reach it.
//!
//! | Access  | Reads                          | Writes                      |
//! |---------|--------------------------------|-----------------------------|
//! | read    | notes, notebooks, tags         | nothing                     |
//! | suggest | the same                       | MCP suggestions, for review |
//! | write   | the same, and attachments      | every route and tool        |
//!
//! Read and suggest keys never see attachments. A write key's writes are
//! checked against the same visibility ([`check_write`]), so a key limited
//! to some notebooks writes only there, and no key writes a locked note.

use std::sync::Arc;

use kanso_core::DocType;
use kanso_core::id::with_prefix;
use kanso_core::model::TRASH_BOOK_ID;
use kanso_store::{KeyAccess, Store};
use serde_json::{Map, Value};

use crate::api::{self, NoteQuery};
use crate::error::{ApiError, ApiResult};
use crate::key_model::KeyId;
use crate::scope::{self, AgentScope, Caller, Visibility};

/// Where a person fixes a key that is not enough.
const SETTINGS: &str = "Settings \u{203a} Local API \u{203a} API keys";

/// What a key that may not write is told, and how to fix it.
pub(crate) fn cannot_write(access: KeyAccess) -> String {
    match access {
        KeyAccess::Read => format!(
            "This key can read your notes but not change them. In Kanso, open {SETTINGS} and create a key that can read and write."
        ),
        _ => format!(
            "This key can suggest edits through MCP but not change notes directly. In Kanso, open {SETTINGS} and create a key that can read and write."
        ),
    }
}

/// What a key that may not use attachments is told.
pub(crate) fn cannot_attach(access: KeyAccess) -> String {
    let what = if access == KeyAccess::Read {
        "read"
    } else {
        "suggest edits"
    };
    format!(
        "This key can {what} but has no access to attachments. In Kanso, open {SETTINGS} and create a key that can read and write."
    )
}

/// The caller of a request, when it is an API key.
pub(crate) struct Key<'a> {
    pub id: &'a KeyId,
    pub access: KeyAccess,
    pub scope: &'a Arc<AgentScope>,
}

impl<'a> Key<'a> {
    /// The key behind `caller`. Anything else reaches a REST route only by
    /// a mistake in the gate, and is refused.
    pub fn of(caller: &'a Caller) -> ApiResult<Key<'a>> {
        match caller {
            Caller::Key { id, access, scope } => Ok(Key {
                id,
                access: *access,
                scope,
            }),
            Caller::Agent { .. } => Err(ApiError::unauthorized()),
        }
    }

    pub fn can_write(&self) -> bool {
        self.access == KeyAccess::Write
    }

    /// Whether this key sees attachments: only a key that can write does.
    pub fn sees_files(&self) -> bool {
        self.can_write()
    }

    /// Refuses a key that may not write.
    pub fn require_write(&self) -> ApiResult<()> {
        if self.can_write() {
            Ok(())
        } else {
            Err(ApiError::forbidden(cannot_write(self.access)))
        }
    }

    pub fn visibility<'s>(&self, store: &'s Store) -> ApiResult<Visibility<'s>> {
        Visibility::new(store, self.scope)
            .ok_or_else(|| ApiError::internal("Couldn't read the notes."))
    }

    /// Whether document `id` is one this key sees.
    pub fn sees(&self, vis: &Visibility, id: &str) -> bool {
        match DocType::of_id(id) {
            Some(DocType::Note) => vis.allows(id),
            Some(DocType::Book) => vis.book_visible(id),
            Some(DocType::Tag) => true,
            Some(DocType::File) => self.sees_files(),
            _ => false,
        }
    }
}

/// A document as a key is shown it: without publish, share and lock fields,
/// and, for a notebook of a key limited to some notebooks, without the
/// count, which counts notes it may not see.
fn shaped(key: &Key, mut doc: Value) -> Value {
    doc = scope::strip(doc);
    if key.scope.notebooks.is_some()
        && doc
            .get("_id")
            .and_then(Value::as_str)
            .and_then(DocType::of_id)
            == Some(DocType::Book)
        && let Some(fields) = doc.as_object_mut()
    {
        fields.remove("count");
    }
    doc
}

/// `GET /<id>` for a key.
pub(crate) fn get_doc(
    store: &Store,
    key: &Key,
    id: &str,
    rev: Option<&str>,
    attachments: bool,
) -> ApiResult<Value> {
    let vis = key.visibility(store)?;
    if !key.sees(&vis, id) {
        return Err(ApiError::not_found());
    }
    // A sealed file is served as a stub, without its bytes, whoever asks.
    let attachments = attachments && key.sees_files();
    Ok(shaped(key, api::get_doc(store, id, rev, attachments)?))
}

/// `GET /notes` for a key: the notes it sees that match, `skip` and `limit`
/// counted among those.
pub(crate) fn list_notes(store: &Store, key: &Key, query: &NoteQuery) -> ApiResult<Vec<Value>> {
    const PAGE: usize = 200;
    let vis = key.visibility(store)?;
    let want = query.limit.unwrap_or(usize::MAX);
    let mut out = Vec::new();
    if want == 0 {
        return Ok(out);
    }
    let mut seen = 0usize;
    let mut offset = 0usize;
    loop {
        let page = api::list_notes(
            store,
            &NoteQuery {
                limit: Some(PAGE),
                skip: offset,
                ..query.clone()
            },
        )?;
        let fetched = page.len();
        for row in page {
            let visible = row
                .get("_id")
                .and_then(Value::as_str)
                .is_some_and(|id| vis.allows(id));
            if !visible {
                continue;
            }
            if seen >= query.skip {
                out.push(shaped(key, row));
                if out.len() == want {
                    return Ok(out);
                }
            }
            seen += 1;
        }
        if fetched < PAGE {
            return Ok(out);
        }
        offset += fetched;
    }
}

/// `GET /loops` for a key: loops in notes it sees.
pub(crate) fn list_loops(
    store: &Store,
    key: &Key,
    book_id: Option<String>,
    kind: Option<kanso_core::loops::LoopKind>,
    limit: Option<usize>,
) -> ApiResult<Vec<Value>> {
    let vis = key.visibility(store)?;
    let rows = api::list_loops(store, book_id, kind, None)?;
    Ok(rows
        .into_iter()
        .filter(|row| {
            row.get("noteId")
                .and_then(Value::as_str)
                .is_some_and(|id| vis.allows(id))
        })
        .take(limit.unwrap_or(usize::MAX))
        .collect())
}

/// `GET /books`, `/tags` and `/files` for a key.
pub(crate) fn list_docs(
    store: &Store,
    key: &Key,
    doc_type: DocType,
    limit: Option<usize>,
    skip: usize,
) -> ApiResult<Vec<Value>> {
    if doc_type == DocType::File && !key.sees_files() {
        return Ok(Vec::new());
    }
    let vis = key.visibility(store)?;
    let docs = api::list_docs(store, doc_type, None, 0)?;
    Ok(docs
        .into_iter()
        .filter(|doc| {
            doc.get("_id")
                .and_then(Value::as_str)
                .is_some_and(|id| key.sees(&vis, id))
                && !(doc_type == DocType::Tag
                    && doc
                        .get("name")
                        .and_then(Value::as_str)
                        .is_some_and(kanso_core::tags::is_ai_exclude))
        })
        .skip(skip)
        .take(limit.unwrap_or(usize::MAX))
        .map(|doc| shaped(key, doc))
        .collect())
}

/// `GET /_changes` for a key: the changes to documents it sees. The
/// sequence numbers are the library's, so a client that follows `last_seq`
/// loses nothing it may see.
pub(crate) fn changes(
    store: &Store,
    key: &Key,
    since: api::Since,
    limit: Option<usize>,
    descending: bool,
    include_docs: bool,
) -> ApiResult<Value> {
    let vis = key.visibility(store)?;
    let mut answer = api::changes(store, since, limit, descending, include_docs)?;
    if let Some(results) = answer.get_mut("results").and_then(Value::as_array_mut) {
        results.retain(|row| {
            let Some(id) = row.get("id").and_then(Value::as_str) else {
                return false;
            };
            // A deleted document is only an ID, and a client that mirrors
            // the library needs to hear of it.
            let gone = row.get("deleted").and_then(Value::as_bool) == Some(true);
            if gone {
                return match DocType::of_id(id) {
                    Some(DocType::File) => key.sees_files(),
                    Some(DocType::Note | DocType::Book | DocType::Tag) => true,
                    _ => false,
                };
            }
            key.sees(&vis, id)
        });
        for row in results.iter_mut() {
            if let Some(doc) = row.get_mut("doc") {
                *doc = shaped(key, doc.take());
            }
        }
    }
    Ok(answer)
}

/// Whether a write may touch note `id`: it is not there yet, or it is not
/// locked and the key may see it (a trashed note counts, so it can be
/// restored or emptied).
fn note_target_ok(store: &Store, vis: &Visibility, id: &str) -> bool {
    match store.note(id) {
        Ok(None) => true,
        Ok(Some(note)) => {
            !store.note_is_locked(id).unwrap_or(true)
                && !store.note_in_locked_book(id).unwrap_or(true)
                && (note.is_trashed() || vis.note(id).is_some())
        }
        Err(_) => false,
    }
}

/// Whether a write may touch notebook `id`.
fn book_target_ok(store: &Store, vis: &Visibility, id: &str) -> bool {
    store.get_api(id, None, false).is_err() || vis.book_visible(id)
}

/// Whether a note may be put in notebook `book`: one the key sees, the
/// trash, or the Templates notebook for a key that sees every notebook.
pub(crate) fn dest_ok(vis: &Visibility, scope: &AgentScope, book: &str) -> bool {
    book == TRASH_BOOK_ID
        || (book == "template" && scope.notebooks.is_none())
        || vis.book_visible(book)
}

/// Checks a write by a key that can write: `doc_type` document `id` (when
/// the write names one) with `fields`. A note or notebook the key may not
/// see is "not found", as it is to a read; a destination outside the key's
/// notebooks is refused.
pub(crate) fn check_write(
    store: &Store,
    key: &Key,
    vis: &Visibility,
    doc_type: DocType,
    id: Option<&str>,
    fields: &Map<String, Value>,
) -> ApiResult<()> {
    // An ID of another kind, or a book that is not there, is the store's to
    // refuse, in its own words.
    if let Some(id) = id
        && DocType::of_id(id) == Some(doc_type)
    {
        let ok = match doc_type {
            DocType::Note => note_target_ok(store, vis, id),
            DocType::Book => book_target_ok(store, vis, id),
            DocType::Tag | DocType::File => true,
            _ => false,
        };
        if !ok {
            return Err(ApiError::not_found());
        }
    }
    let outside = |book: &str| {
        Err(ApiError::forbidden(format!(
            "This key is limited to some notebooks, and {book} is not one of them. In Kanso, open {SETTINGS} and create a key for the notebooks you need."
        )))
    };
    match doc_type {
        DocType::Note => {
            if let Some(book) = fields.get("bookId").and_then(Value::as_str)
                && !dest_ok(vis, key.scope, book)
                && store.get_api(book, None, false).is_ok()
            {
                return outside(book);
            }
        }
        DocType::Book => {
            if let Some(parent) = fields.get("parentBookId").and_then(Value::as_str)
                && !vis.book_visible(parent)
                && store.get_api(parent, None, false).is_ok()
            {
                return outside(parent);
            }
        }
        _ => {}
    }
    Ok(())
}

/// The notebook ID a `kanso-…` tool argument names.
pub(crate) fn book_arg(raw: &str) -> String {
    with_prefix(DocType::Book, raw)
}
