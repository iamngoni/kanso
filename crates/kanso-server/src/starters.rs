//! Notebook starters over the API (issue #514): `POST /books` and MCP
//! `create-book` take an optional `starter`, and `kanso-list-starters` lists
//! the choices. The catalogue and the rules that make one are the store's
//! (`kanso_store::starters`), the same ones New Notebook uses.

use kanso_store::starters::{NotebookKind, SHOT_LIST, VIDEO_BRIEF, create_notebook_at};
use kanso_store::{Origin, Store};
use serde_json::{Map, Value, json};

use crate::error::{ApiError, ApiResult};

/// What an unknown or mistyped `starter` fails with.
pub(crate) const UNKNOWN_STARTER: &str = "'starter' must be one of blank, product, knowledge, side-projects, journal, reading, publishing, video-studio.";

/// The kind a `starter` value names, if it is one.
fn kind_of(value: &Value) -> ApiResult<NotebookKind> {
    value
        .as_str()
        .and_then(NotebookKind::parse)
        .ok_or_else(|| ApiError::bad_request(UNKNOWN_STARTER))
}

/// Refuses a starter that makes notes inside a locked notebook, in the
/// store's words. A key can't see a locked notebook, so this comes before
/// the key's own check, which would only say the notebook isn't its own.
pub(crate) fn refuse_locked(store: &Store, fields: &Map<String, Value>) -> ApiResult<()> {
    let kind = fields
        .get("starter")
        .and_then(Value::as_str)
        .and_then(NotebookKind::parse);
    let parent = fields.get("parentBookId").and_then(Value::as_str);
    if let (Some(kind), Some(parent)) = (kind, parent)
        && kind.makes_notes()
        && store.locked_books()?.contains(parent)
    {
        return Err(ApiError::bad_request(kanso_store::LOCKED_STARTER));
    }
    Ok(())
}

/// The notebook a body with a `starter` asks for, or None when the body has
/// none (or says null), which is the plain notebook write. Blank is a plain
/// notebook too, so the rest of the body still applies.
pub(crate) fn make(store: &Store, body: &mut Value) -> ApiResult<Option<Value>> {
    let Some(fields) = body.as_object_mut() else {
        return Ok(None);
    };
    let Some(starter) = fields.remove("starter").filter(|value| !value.is_null()) else {
        return Ok(None);
    };
    let kind = kind_of(&starter)?;
    if kind == NotebookKind::Blank {
        return Ok(None);
    }
    let name = fields.get("name").and_then(Value::as_str).unwrap_or("");
    let parent = fields
        .get("parentBookId")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let now = chrono::Local::now().naive_local();
    let made = create_notebook_at(store, name, parent, kind, Origin::Api, now)?;
    let rev = store
        .get_api(&made.book_id, None, false)?
        .get("_rev")
        .cloned()
        .unwrap_or(Value::Null);
    Ok(Some(json!({
        "ok": true,
        "id": made.book_id,
        "rev": rev,
        "subBookIds": made.child_ids,
        "tagIds": made.tag_ids,
        "noteIds": made.note_ids,
    })))
}

/// `kanso-list-starters`: every choice with what it makes.
pub(crate) fn catalogue() -> Value {
    let today = chrono::Local::now().date_naive();
    let starters: Vec<Value> = NotebookKind::ALL
        .into_iter()
        .map(|kind| {
            let mut entry = Map::new();
            entry.insert("key".into(), kind.key().into());
            entry.insert("label".into(), kind.label().into());
            entry.insert("description".into(), kind.description().into());
            entry.insert("icon".into(), kind.icon().into());
            entry.insert("makesNotes".into(), kind.makes_notes().into());
            let starter = kind.starter(today);
            let books = starter.as_ref().map(|s| s.children.as_slice());
            entry.insert(
                "subNotebooks".into(),
                books
                    .unwrap_or_default()
                    .iter()
                    .map(|book| {
                        json!({
                            "name": book.name,
                            "icon": book.icon,
                            "template": book.templates.first().map(|t| t.title),
                        })
                    })
                    .collect(),
            );
            entry.insert(
                "tags".into(),
                starter
                    .iter()
                    .flat_map(|s| &s.tags)
                    .map(|tag| json!({"name": tag.name, "color": tag.color}))
                    .collect(),
            );
            let templates: Vec<&str> = if kind == NotebookKind::VideoStudio {
                vec![VIDEO_BRIEF, SHOT_LIST]
            } else {
                books
                    .unwrap_or_default()
                    .iter()
                    .flat_map(|book| &book.templates)
                    .map(|template| template.title)
                    .collect()
            };
            entry.insert("templates".into(), json!(templates));
            Value::Object(entry)
        })
        .collect();
    Value::Array(starters)
}
