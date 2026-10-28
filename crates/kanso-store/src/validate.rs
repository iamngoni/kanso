//! Normalizes incoming documents: fills defaults, strips derived fields,
//! validates types and, for local writes, references to notebooks and tags.

use kanso_core::model::{
    MAX_BODY_LEN, MAX_NAME_LEN, MAX_TITLE_LEN, SUPPORTED_AUDIO_TYPES, SUPPORTED_DOCUMENT_TYPES,
    SUPPORTED_IMAGE_TYPES, TRASH_BOOK_ID,
};
use kanso_core::tasks::count_tasks;
use kanso_core::{DocType, NoteShare, NoteStatus, TagColor};
use rusqlite::{Connection, OptionalExtension};
use serde_json::{Map, Value};

use crate::{Result, StoreError};

/// Fields computed by the store and never stored as given.
pub const DERIVED_FIELDS: &[&str] = &["count", "numOfTasks", "numOfCheckedTasks"];

/// Whether to check that referenced notebooks and tags exist. Imports and
/// sync apply documents in arbitrary order, so they skip the check.
#[derive(Clone, Copy)]
pub enum Refs<'a> {
    Check(&'a Connection),
    Skip,
}

pub fn strip_reserved(fields: &mut Map<String, Value>) {
    fields.retain(|key, _| !key.starts_with('_') && !DERIVED_FIELDS.contains(&key.as_str()));
}

pub fn normalize(
    doc_type: DocType,
    id: &str,
    mut fields: Map<String, Value>,
    existing: Option<&Map<String, Value>>,
    now: i64,
    refs: Refs,
) -> Result<Map<String, Value>> {
    strip_reserved(&mut fields);
    let keep = |fields: &mut Map<String, Value>, key: &str| {
        if !fields.contains_key(key)
            && let Some(value) = existing.and_then(|e| e.get(key))
        {
            fields.insert(key.to_owned(), value.clone());
        }
    };
    keep(&mut fields, "createdAt");
    let created_at = timestamp(&fields, "createdAt")?.unwrap_or(now);
    fields.insert("createdAt".into(), created_at.into());

    match doc_type {
        DocType::Note => normalize_note(id, &mut fields, now, refs)?,
        DocType::Book => normalize_book(id, &mut fields, now, refs)?,
        DocType::Tag => normalize_tag(&mut fields, now)?,
        DocType::File => normalize_file(&mut fields, refs)?,
    }
    Ok(fields)
}

fn normalize_note(id: &str, f: &mut Map<String, Value>, now: i64, refs: Refs) -> Result<()> {
    let title = string_or(f, "title", "")?;
    if title.chars().count() > MAX_TITLE_LEN {
        return invalid(format!(
            "The note title is longer than {MAX_TITLE_LEN} characters."
        ));
    }
    let body = string_or(f, "body", "")?;
    if body.chars().count() > MAX_BODY_LEN {
        return invalid(format!(
            "The note body is longer than {MAX_BODY_LEN} characters."
        ));
    }
    let tasks = count_tasks(&body);
    f.insert("title".into(), title.into());
    f.insert("body".into(), body.into());

    let Some(book_id) = f.get("bookId").and_then(Value::as_str).map(str::to_owned) else {
        return invalid(format!("The note '{id}' has an invalid bookId."));
    };
    if let Refs::Check(conn) = refs
        && book_id != TRASH_BOOK_ID
        && !live_doc_exists(conn, DocType::Book, &book_id)?
    {
        return invalid(format!("The note '{id}' has an invalid bookId."));
    }

    let mut tags = Vec::new();
    match f.get("tags") {
        None | Some(Value::Null) => {}
        Some(Value::Array(items)) => {
            for item in items {
                let Some(tag) = item.as_str() else {
                    return invalid(format!("The note '{id}' has an invalid tag."));
                };
                if !tags.iter().any(|t: &String| t == tag) {
                    tags.push(tag.to_owned());
                }
            }
        }
        Some(_) => return invalid(format!("The note '{id}' has invalid tags.")),
    }
    if let Refs::Check(conn) = refs {
        for tag in &tags {
            if !live_doc_exists(conn, DocType::Tag, tag)? {
                return invalid(format!("The note '{id}' has an invalid tag '{tag}'."));
            }
        }
    }
    f.insert("tags".into(), tags.into());

    let status = string_or(f, "status", "none")?;
    let Some(status) = NoteStatus::parse(&status) else {
        return invalid(format!("The note '{id}' has an invalid status '{status}'."));
    };
    f.insert("status".into(), status.as_str().into());

    let share = string_or(f, "share", "private")?;
    let share = match share.as_str() {
        "private" => NoteShare::Private,
        "public" => NoteShare::Public,
        other => return invalid(format!("The note '{id}' has an invalid share '{other}'.")),
    };
    f.insert("share".into(), serde_json::to_value(share)?);

    let pinned = match f.get("pinned") {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(_) => return invalid(format!("The note '{id}' has an invalid pinned flag.")),
    };
    f.insert("pinned".into(), pinned.into());
    let doctype = string_or(f, "doctype", "markdown")?;
    f.insert("doctype".into(), doctype.into());

    let updated_at = timestamp(f, "updatedAt")?.unwrap_or(now);
    f.insert("updatedAt".into(), updated_at.into());
    let stamp = timestamp(f, "timestamp")?.unwrap_or(updated_at);
    f.insert("timestamp".into(), stamp.into());
    f.insert("numOfTasks".into(), tasks.total.into());
    f.insert("numOfCheckedTasks".into(), tasks.checked.into());
    Ok(())
}

fn normalize_book(id: &str, f: &mut Map<String, Value>, now: i64, refs: Refs) -> Result<()> {
    let name = required_name(f, "notebook")?;
    f.insert("name".into(), name.into());
    let parent = match f.get("parentBookId") {
        None | Some(Value::Null) => None,
        Some(Value::String(p)) => Some(p.clone()),
        Some(_) => return invalid("The notebook has an invalid parentBookId.".into()),
    };
    if let (Refs::Check(conn), Some(parent)) = (refs, &parent) {
        if !live_doc_exists(conn, DocType::Book, parent)? {
            return invalid(format!("The notebook '{id}' has an invalid parentBookId."));
        }
        // Walk up from the new parent; reaching this notebook would make a cycle.
        let mut cursor = Some(parent.clone());
        let mut steps = 0;
        while let Some(current) = cursor {
            if current == id || steps > 1000 {
                return invalid(format!(
                    "The notebook '{id}' cannot be nested inside itself."
                ));
            }
            cursor = book_parent(conn, &current)?;
            steps += 1;
        }
    }
    f.insert(
        "parentBookId".into(),
        parent.map_or(Value::Null, Value::String),
    );
    if let Some(order) = f.get("order")
        && !order.is_null()
        && order.as_i64().is_none()
    {
        return invalid("The notebook has an invalid order.".into());
    }
    let updated_at = timestamp(f, "updatedAt")?.unwrap_or(now);
    f.insert("updatedAt".into(), updated_at.into());
    Ok(())
}

fn normalize_tag(f: &mut Map<String, Value>, now: i64) -> Result<()> {
    let name = required_name(f, "tag")?;
    f.insert("name".into(), name.into());
    let color = f.get("color").cloned().unwrap_or(Value::from("default"));
    let color: TagColor = serde_json::from_value(color.clone())
        .map_err(|_| StoreError::Invalid(format!("The tag has an invalid color {color}.")))?;
    f.insert("color".into(), serde_json::to_value(color)?);
    let updated_at = timestamp(f, "updatedAt")?.unwrap_or(now);
    f.insert("updatedAt".into(), updated_at.into());
    Ok(())
}

fn normalize_file(f: &mut Map<String, Value>, refs: Refs) -> Result<()> {
    let name = string_or(f, "name", "")?;
    if name.is_empty() || name.chars().count() > 128 {
        return invalid("The file needs a name of 1 to 128 characters.".into());
    }
    let content_type = string_or(f, "contentType", "")?;
    let sealed_metadata = kanso_core::model::FileLock::from_extra(f).is_some_and(|lock| lock.meta);
    let supported = (sealed_metadata
        && content_type == kanso_core::model::SEALED_FILE_CONTENT_TYPE)
        || SUPPORTED_IMAGE_TYPES.contains(&content_type.as_str())
        || SUPPORTED_AUDIO_TYPES.contains(&content_type.as_str())
        || SUPPORTED_DOCUMENT_TYPES.contains(&content_type.as_str());
    if matches!(refs, Refs::Check(_)) && !supported {
        return invalid(format!("Unsupported file type '{content_type}'."));
    }
    f.insert("contentType".into(), content_type.into());
    if !matches!(f.get("publicIn"), Some(Value::Array(_))) {
        f.insert("publicIn".into(), Value::Array(Vec::new()));
    }
    Ok(())
}

fn required_name(f: &Map<String, Value>, what: &str) -> Result<String> {
    let name = string_or(f, "name", "")?;
    let trimmed = name.trim();
    if trimmed.is_empty() || trimmed.chars().count() > MAX_NAME_LEN {
        return invalid(format!(
            "The {what} needs a name of 1 to {MAX_NAME_LEN} characters."
        ));
    }
    Ok(trimmed.to_owned())
}

fn string_or(f: &Map<String, Value>, key: &str, default: &str) -> Result<String> {
    match f.get(key) {
        None | Some(Value::Null) => Ok(default.to_owned()),
        Some(Value::String(s)) => Ok(s.clone()),
        Some(_) => invalid(format!("'{key}' must be a string.")),
    }
}

fn timestamp(f: &Map<String, Value>, key: &str) -> Result<Option<i64>> {
    match f.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => v
            .as_i64()
            .or_else(|| v.as_f64().filter(|x| x.is_finite()).map(|x| x as i64))
            .map(Some)
            .ok_or_else(|| StoreError::Invalid(format!("'{key}' must be a timestamp."))),
    }
}

fn invalid<T>(message: String) -> Result<T> {
    Err(StoreError::Invalid(message))
}

pub fn live_doc_exists(conn: &Connection, doc_type: DocType, id: &str) -> Result<bool> {
    Ok(conn
        .query_row(
            "SELECT 1 FROM docs WHERE id = ? AND type = ? AND deleted = 0",
            [id, doc_type.as_str()],
            |_| Ok(()),
        )
        .optional()?
        .is_some())
}

fn book_parent(conn: &Connection, id: &str) -> Result<Option<String>> {
    Ok(conn
        .query_row(
            "SELECT parent_book_id FROM docs WHERE id = ? AND type = 'book' AND deleted = 0",
            [id],
            |row| row.get::<_, Option<String>>(0),
        )
        .optional()?
        .flatten())
}
