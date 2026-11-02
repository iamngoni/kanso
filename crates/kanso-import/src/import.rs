//! Copies the winning revision of every document in a PouchDB library into
//! the store.

use std::collections::{BTreeSet, HashMap};
use std::path::Path;

use anyhow::Context as _;
use kanso_core::tasks::{linked_file_ids, linked_note_ids};
use kanso_core::{DocType, TRASH_BOOK_ID};
use kanso_store::{Attachment, Store, StoreError};
use rusqlite::{Connection, OpenFlags};
use serde_json::{Map, Value};

use crate::pouch::{self, SourceDoc};
use crate::report::ImportReport;

/// The stage an import is in, reported through the progress callback.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Snapshot,
    Books,
    Tags,
    Files,
    Notes,
    Activities,
    Verify,
}

/// Progress through one phase: `done` of `total` items.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Progress {
    pub phase: Phase,
    pub done: usize,
    pub total: usize,
}

enum Kind {
    Doc(DocType),
    Activity,
    DesignOrConfig,
    Unknown,
}

fn kind_of(id: &str) -> Kind {
    if id.starts_with("_design/") || id == "config" {
        Kind::DesignOrConfig
    } else if id.starts_with("activity:") {
        Kind::Activity
    } else if let Some(doc_type) = DocType::of_id(id).filter(|t| *t != DocType::Review) {
        Kind::Doc(doc_type)
    } else {
        Kind::Unknown
    }
}

/// Imports the PouchDB library at `source_sqlite` (the `db/<hash>.sqlite`
/// file) into `store`.
///
/// The source is opened read-only and copied with SQLite's online backup API
/// into a temporary file first, so the app that owns it may keep running and its files are
/// never written. Running it again imports nothing new.
pub fn import(
    source_sqlite: &Path,
    store: &Store,
    mut progress: impl FnMut(Progress),
) -> anyhow::Result<ImportReport> {
    progress(Progress {
        phase: Phase::Snapshot,
        done: 0,
        total: 1,
    });
    let temp = tempfile::Builder::new()
        .prefix("kanso-import-")
        .tempdir()
        .context("cannot create a temporary directory")?;
    let snapshot_path = temp.path().join("library.sqlite");
    pouch::snapshot(source_sqlite, &snapshot_path)?;
    let conn = Connection::open_with_flags(
        &snapshot_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    pouch::check_layout(&conn)?;
    progress(Progress {
        phase: Phase::Snapshot,
        done: 1,
        total: 1,
    });

    let mut report = ImportReport::default();
    let mut books = Vec::new();
    let mut tags = Vec::new();
    let mut files = Vec::new();
    let mut notes = Vec::new();
    let mut activities = Vec::new();
    for doc in pouch::winning_docs(&conn)? {
        let doc = match doc {
            Ok(doc) => doc,
            Err(unreadable) => {
                report
                    .rejected
                    .push(format!("{}: {}", unreadable.id, unreadable.reason));
                continue;
            }
        };
        match kind_of(&doc.id) {
            Kind::DesignOrConfig => report.skipped_design_and_config += 1,
            _ if doc.deleted => report.skipped_deleted += 1,
            Kind::Unknown => report.skipped_unknown += 1,
            Kind::Activity => activities.push(doc),
            Kind::Doc(DocType::Book) => books.push(doc),
            Kind::Doc(DocType::Tag) => tags.push(doc),
            Kind::Doc(DocType::File) => files.push(doc),
            Kind::Doc(DocType::Note) => notes.push(doc),
            // Review logs are Kanso's own; no source library has them.
            Kind::Doc(DocType::Review) => report.skipped_unknown += 1,
        }
    }

    report.source.books = books.len();
    report.source.tags = tags.len();
    report.source.files = files.len();
    report.source.notes = notes.len();
    report.source.trashed_notes = notes.iter().filter(|n| is_trashed(&n.fields)).count();
    report.source.activities = activities.len();

    let mut linked_notes = BTreeSet::new();
    let mut linked_files = BTreeSet::new();
    for note in &notes {
        let body = note
            .fields
            .get("body")
            .and_then(Value::as_str)
            .unwrap_or("");
        let note_links = linked_note_ids(body);
        if !note_links.is_empty() {
            report.notes_linking_notes += 1;
            linked_notes.extend(note_links);
        }
        let file_links = linked_file_ids(body);
        if !file_links.is_empty() {
            report.notes_embedding_files += 1;
            linked_files.extend(file_links);
        }
    }

    let mut imported: Vec<(String, String)> = Vec::new();
    for (phase, doc_type, docs) in [
        (Phase::Books, DocType::Book, books),
        (Phase::Tags, DocType::Tag, tags),
        (Phase::Files, DocType::File, files),
        (Phase::Notes, DocType::Note, notes),
    ] {
        let total = docs.len();
        progress(Progress {
            phase,
            done: 0,
            total,
        });
        for (done, doc) in docs.into_iter().enumerate() {
            import_one(&conn, store, doc_type, doc, &mut report, &mut imported)?;
            progress(Progress {
                phase,
                done: done + 1,
                total,
            });
        }
    }

    let archived = archived_docs(store.path())?;
    let total = activities.len();
    progress(Progress {
        phase: Phase::Activities,
        done: 0,
        total,
    });
    for (done, doc) in activities.into_iter().enumerate() {
        let mut json = Map::new();
        json.insert("_id".into(), Value::String(doc.id.clone()));
        json.insert("_rev".into(), Value::String(doc.rev));
        json.extend(doc.fields);
        let json = Value::Object(json);
        if archived.get(&doc.id) == Some(&json) {
            report.unchanged += 1;
        } else {
            store
                .archive(&doc.id, &json)
                .with_context(|| format!("cannot archive {}", doc.id))?;
            report.archived += 1;
        }
        progress(Progress {
            phase: Phase::Activities,
            done: done + 1,
            total,
        });
    }

    let total = imported.len();
    progress(Progress {
        phase: Phase::Verify,
        done: 0,
        total,
    });
    for (id, rev) in &imported {
        if store.get(id)?.is_none_or(|doc| doc.rev != *rev) {
            report.unverified.push(id.clone());
        }
    }
    for id in &linked_notes {
        if store.get(id)?.is_none() {
            report.dangling_note_links += 1;
        }
    }
    for id in &linked_files {
        if store.get(id)?.is_none() {
            report.dangling_file_links += 1;
        }
    }
    progress(Progress {
        phase: Phase::Verify,
        done: total,
        total,
    });
    Ok(report)
}

fn is_trashed(fields: &Map<String, Value>) -> bool {
    fields.get("bookId").and_then(Value::as_str) == Some(TRASH_BOOK_ID)
}

/// Writes one document. `imported` collects `(id, rev)` of every document the
/// store holds afterwards, for verification.
fn import_one(
    conn: &Connection,
    store: &Store,
    doc_type: DocType,
    doc: SourceDoc,
    report: &mut ImportReport,
    imported: &mut Vec<(String, String)>,
) -> anyhow::Result<()> {
    let SourceDoc {
        id,
        rev,
        mut fields,
        ..
    } = doc;
    let attachment = if doc_type == DocType::File {
        match verified_attachment(conn, &mut fields)? {
            Ok(attachment) => Some(attachment),
            Err(reason) => {
                report.attachment_mismatches.push(format!("{id}: {reason}"));
                return Ok(());
            }
        }
    } else {
        None
    };
    let trashed = doc_type == DocType::Note && is_trashed(&fields);
    let previous = store.get(&id)?;
    match store.import_doc(doc_type, &id, &rev, fields, attachment) {
        Ok(true) => {
            match doc_type {
                DocType::Note => report.notes += 1,
                DocType::Book => report.books += 1,
                DocType::Tag => report.tags += 1,
                DocType::File => report.files += 1,
                DocType::Review => {}
            }
            report.trashed_notes += usize::from(trashed);
            report.replaced += usize::from(previous.is_some());
        }
        Ok(false) => report.unchanged += 1,
        Err(StoreError::Invalid(reason)) => {
            report.rejected.push(format!("{id}: {reason}"));
            return Ok(());
        }
        Err(err) => return Err(err).with_context(|| format!("cannot import {id}")),
    }
    imported.push((id, rev));
    Ok(())
}

/// Removes `_attachments` from a file document and returns its bytes once they
/// are found and match the recorded digest and length. `Ok(Err(reason))` when
/// they don't.
fn verified_attachment(
    conn: &Connection,
    fields: &mut Map<String, Value>,
) -> anyhow::Result<Result<Attachment, String>> {
    let attachments = fields.remove("_attachments");
    let index = attachments
        .as_ref()
        .and_then(Value::as_object)
        .and_then(|map| {
            map.get("index")
                .or_else(|| (map.len() == 1).then(|| map.values().next()).flatten())
        });
    let Some(index) = index else {
        return Ok(Err("it has no attachment".to_owned()));
    };
    let Some(digest) = index.get("digest").and_then(Value::as_str) else {
        return Ok(Err("its attachment has no digest".to_owned()));
    };
    let bytes = match pouch::attachment_bytes(conn, digest)? {
        None => return Ok(Err(format!("no attachment body is stored under {digest}"))),
        Some(Err(reason)) => return Ok(Err(reason)),
        Some(Ok(bytes)) => bytes,
    };
    let attachment = Attachment::new(bytes);
    if attachment.digest() != digest {
        return Ok(Err(format!(
            "digest mismatch: expected {digest}, got {}",
            attachment.digest()
        )));
    }
    if let Some(length) = index.get("length").and_then(Value::as_u64)
        && length != attachment.data.len() as u64
    {
        return Ok(Err(format!(
            "length mismatch: expected {length} bytes, got {}",
            attachment.data.len()
        )));
    }
    if !fields.contains_key("contentType")
        && let Some(content_type) = index.get("content_type").and_then(Value::as_str)
    {
        fields.insert("contentType".into(), content_type.into());
    }
    Ok(Ok(attachment))
}

/// The documents already in the store's archive table.
fn archived_docs(store_path: &Path) -> anyhow::Result<HashMap<String, Value>> {
    let conn = Connection::open_with_flags(
        store_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .with_context(|| format!("cannot read {}", store_path.display()))?;
    let mut stmt = conn.prepare("SELECT id, json FROM archive")?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    let mut docs = HashMap::new();
    for row in rows {
        let (id, json) = row?;
        if let Ok(value) = serde_json::from_str(&json) {
            docs.insert(id, value);
        }
    }
    Ok(docs)
}
