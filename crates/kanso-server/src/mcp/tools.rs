//! Kanso's native MCP implementation: the Inkdrop-compatible tools, the
//! note resource, and Kanso's own `kanso-…` tools after them.
//!
//! Each tool does what `@inkdropapp/mcp-server` 1.5.0 does with the HTTP API,
//! through the same operations the REST handlers use, and formats its result
//! the same way: documents as `JSON.stringify(value, null, 2)`, failed
//! requests as `Fetch error [<status>] <reason>`.
//!
//! For an agent, each read tool gets the request's [`Visibility`]: a note it
//! may not see is answered exactly as a missing one, results are filtered
//! before they are cut to their limit, and every note shown is noted.

use std::path::Path;

use base64::Engine as _;
use base64::engine::DecodePaddingMode;
use base64::engine::general_purpose::{GeneralPurpose, GeneralPurposeConfig, STANDARD};
use kanso_core::id::with_prefix;
use kanso_core::links::{LINK_SCHEME, NOTE_LINK_PREFIX};
use kanso_core::model::{MAX_ATTACHMENT_SIZE, SUPPORTED_IMAGE_TYPES};
use kanso_core::{DocType, now_ms};
use kanso_store::{Attachment, AttachmentState, MEANING_SIMILARITY, MeaningSearch, Store};
use serde_json::{Map, Value, json};

use super::RpcError;
use super::schema;
use crate::AppState;
use crate::api::{self, NoteQuery};
use crate::error::ApiError;
use crate::scope::{self, Visibility};

type Args = Map<String, Value>;

/// A tool failure, reported as an `isError` result with this text.
struct ToolError(String);

impl From<ApiError> for ToolError {
    fn from(err: ApiError) -> Self {
        ToolError(err.fetch_error_text())
    }
}

type ToolResult = Result<Value, ToolError>;

const SOURCE_TEMPLATE_DESCRIPTION: &str = "The template this note was created from. Read it to get the fill-out instructions and examples before completing this note.";

/// How many more rows a filtered search fetches, so that what an agent may
/// see still fills the limit.
const SCOPED_WIDEN: usize = 5;

/// Runs `tools/call`; `vis` is an agent's, when one is asking.
pub(crate) fn call(
    state: &AppState,
    vis: Option<&Visibility>,
    name: &str,
    arguments: Option<&Value>,
) -> Value {
    let Some(tool) = schema::tool(name) else {
        return error_result(format!("MCP error -32602: Tool {name} not found"));
    };
    let args = match tool.validate(arguments) {
        Ok(args) => args,
        Err(issues) => {
            return error_result(format!(
                "MCP error -32602: Input validation error: Invalid arguments for tool {name}: {issues}"
            ));
        }
    };
    let store = &*state.store;
    let result = match (name, vis) {
        ("read-note", vis) => read_note(store, vis, &args),
        ("search-notes", vis) => search_notes(store, vis, &args),
        ("list-notes", vis) => list_notes(store, vis, &args),
        ("list-templates", vis) => list_templates(store, vis, &args),
        ("list-notebooks", Some(vis)) => list_visible_books(store, vis),
        ("kanso-get-outline", vis) => get_outline(store, vis, &args),
        ("kanso-list-tasks", vis) => list_tasks(store, vis, &args),
        ("kanso-related-notes", vis) => related_notes(store, vis, &args),
        ("kanso-resolve-wikilink", vis) => resolve_wikilink(store, vis, &args),
        ("kanso-search-by-meaning", vis) => search_by_meaning(store, vis, &args),
        ("kanso-list-tags", vis) => list_visible_tags(store, vis),
        ("kanso-get-properties", vis) => get_properties(store, vis, &args),
        ("kanso-list-note-links", vis) => list_note_links(store, vis, &args),
        ("kanso-list-backlinks", vis) => list_backlinks(store, vis, &args),
        ("kanso-run-query-view", vis) => run_query_view(store, vis, &args),
        ("kanso-list-versions", vis) => list_versions(store, vis, &args),
        ("kanso-read-version", vis) => read_version(store, vis, &args),
        // Nothing else is ever an agent's; the scope is checked before this.
        (_, Some(_)) => Err(ToolError(super::READ_ONLY.to_owned())),
        (name, None) => call_full(state, name, args),
    };
    result.unwrap_or_else(|ToolError(message)| error_result(message))
}

/// The tools only the local API's own sign-in reaches.
fn call_full(state: &AppState, name: &str, args: Args) -> ToolResult {
    let store = &*state.store;
    match name {
        "read-backlinks" => read_backlinks(store, &args),
        "create-note" => upsert(store, DocType::Note, args),
        "update-note" => update_note(store, args),
        "patch-note" => patch_note(store, &args),
        "list-tags" => list(store, DocType::Tag),
        "read-tag" => read(store, DocType::Tag, &args, "tagId"),
        "create-tag" => upsert(store, DocType::Tag, args),
        "update-tag" => merge(store, args, false),
        "list-notebooks" => list(store, DocType::Book),
        "read-book" => read(store, DocType::Book, &args, "bookId"),
        "create-book" => upsert(store, DocType::Book, args),
        "update-book" => merge(store, args, true),
        "create-file" => create_file(store, state.local_files, &args),
        "read-file" => read_file(store, state.local_files, &args),
        "kanso-create-daily-note" => create_daily_note(store, &args),
        "kanso-create-from-template" => create_from_template(store, &args),
        "kanso-export-html" => export_html(store, &args),
        "kanso-set-task-done" => set_task_done(store, &args),
        "kanso-move-note" => move_note(store, &args),
        "kanso-attach-file" => attach_file(store, &args),
        "kanso-list-attachments" => list_attachments(store, &args),
        "kanso-read-attachment" => read_attachment(store, &args),
        _ => Err(ToolError(format!(
            "MCP error -32602: Tool {name} not found"
        ))),
    }
}

/// `get_doc` for a note an agent may not see: the same failure as a note
/// that isn't there.
fn visible_note(store: &Store, vis: Option<&Visibility>, id: &str) -> ToolResult {
    if let Some(vis) = vis {
        if !vis.allows(id) {
            return Err(ApiError::not_found().into());
        }
        let note = api::get_doc(store, id, None, false)?;
        vis.show(id);
        return Ok(scope::strip(note));
    }
    Ok(api::get_doc(store, id, None, false)?)
}

/// A note exposed by Kanso-native reads: live, unlocked, and not excluded.
fn native_visible_note(store: &Store, vis: Option<&Visibility>, id: &str) -> ToolResult {
    let note = visible_note(store, vis, id)?;
    if !native_note_is_visible(store, id) {
        return Err(ApiError::not_found().into());
    }
    Ok(note)
}

/// Whether a Kanso-native read may mention a note, regardless of caller kind.
fn native_note_is_visible(store: &Store, id: &str) -> bool {
    if store.note_is_locked(id).unwrap_or(true) || store.note_in_locked_book(id).unwrap_or(true) {
        return false;
    }
    let Ok(Some(note)) = store.note(id) else {
        return false;
    };
    if note.is_trashed() {
        return false;
    }
    let Ok(tags) = store.tags() else {
        return false;
    };
    !note.tags.iter().any(|tag| {
        kanso_core::tags::is_ai_exclude(tag)
            || tags
                .iter()
                .any(|known| known.id == *tag && kanso_core::tags::is_ai_exclude(&known.name))
    })
}

/// `list-notebooks` for an agent: only notebooks in scope and unlocked, so a
/// locked notebook's name doesn't show, and without note counts, which
/// count notes the agent may not see.
fn list_visible_books(store: &Store, vis: &Visibility) -> ToolResult {
    let books: Vec<Value> = api::list_docs(store, DocType::Book, None, 0)?
        .into_iter()
        .filter(|book| {
            book.get("_id")
                .and_then(Value::as_str)
                .is_some_and(|id| vis.book_visible(id))
        })
        .map(|book| {
            let mut book = scope::strip(book);
            if let Some(fields) = book.as_object_mut() {
                fields.remove("count");
            }
            book
        })
        .collect();
    result(vec![text(pretty(&Value::Array(books)))])
}

/// Search rows an agent may see, at most `limit`, each noted as shown.
fn visible_rows(vis: Option<&Visibility>, rows: Vec<Value>, limit: usize) -> Vec<Value> {
    let Some(vis) = vis else {
        return rows;
    };
    rows.into_iter()
        .filter(|row| {
            row.get("_id")
                .and_then(Value::as_str)
                .is_some_and(|id| vis.allows(id))
        })
        .take(limit)
        .inspect(|row| {
            if let Some(id) = row.get("_id").and_then(Value::as_str) {
                vis.show(id);
            }
        })
        .map(scope::strip)
        .collect()
}

/// Now on this computer's clock, in its time zone.
fn local_now() -> kanso_core::dates::DateTime {
    let local = chrono::Local::now()
        .naive_local()
        .and_utc()
        .timestamp_millis();
    kanso_core::dates::DateTime::from_local_ms(local)
}

/// What a note made or opened by a tool is, for its answer.
fn made_note(store: &Store, id: &str, created: Option<bool>) -> ToolResult {
    let note = api::get_doc(store, id, None, false)?;
    let mut answer = Map::new();
    answer.insert("noteId".into(), json!(id));
    answer.insert(
        "title".into(),
        note.get("title").cloned().unwrap_or_default(),
    );
    answer.insert(
        "bookId".into(),
        note.get("bookId").cloned().unwrap_or_default(),
    );
    answer.insert("uri".into(), json!(note_uri(id)));
    if let Some(created) = created {
        answer.insert("created".into(), json!(created));
    }
    result(vec![text(pretty(&Value::Object(answer)))])
}

/// `kanso-create-daily-note`: the day's note in the Journal, made from the
/// daily template when missing.
fn create_daily_note(store: &Store, args: &Args) -> ToolResult {
    let mut now = local_now();
    if let Some(date) = opt_str(args, "date") {
        now.date = kanso_core::dates::Date::parse_iso(date.trim())
            .ok_or_else(|| ToolError(format!("`{date}` is not a date: write it as YYYY-MM-DD.")))?;
    }
    let daily = store
        .open_daily(now, kanso_store::Origin::Api)
        .map_err(|err| ToolError(err.to_string()))?;
    made_note(store, &daily.id, Some(daily.created))
}

/// `kanso-create-from-template`: a note from a template, its variables
/// filled in.
fn create_from_template(store: &Store, args: &Args) -> ToolResult {
    let template = with_prefix(DocType::Note, str_arg(args, "templateId"));
    // The same failure as reading a note that isn't there.
    api::get_doc(store, &template, None, false)?;
    let book = opt_str(args, "bookId").map(|id| with_prefix(DocType::Book, id));
    let made = store
        .create_from_template(
            &template,
            Some(str_arg(args, "title")),
            book.as_deref(),
            local_now(),
            kanso_store::Origin::Api,
        )
        .map_err(|err| ToolError(err.to_string()))?;
    made_note(store, &made.id, None)
}

/// `kanso-export-html`: the note as a standalone page, in Kanso's light or
/// dark theme.
fn export_html(store: &Store, args: &Args) -> ToolResult {
    let id = with_prefix(DocType::Note, str_arg(args, "noteId"));
    let note = api::get_doc(store, &id, None, false)?;
    let palette = match opt_str(args, "theme") {
        Some("dark") => kanso_render::Palette::DARK,
        _ => kanso_render::Palette::LIGHT,
    };
    let attachments = |id: &str| {
        if store.attachment_state(id).ok() == Some(AttachmentState::Sealed) {
            return None;
        }
        store
            .attachment(id)
            .ok()
            .flatten()
            .map(|(content_type, attachment)| (content_type, attachment.data))
    };
    let html = kanso_render::html::page(
        &js_string(note.get("title")),
        &js_string(note.get("body")),
        &palette,
        &attachments,
    );
    result(vec![text(html)])
}

/// Lists tags with counts computed only from notes the caller may see.
fn list_visible_tags(store: &Store, vis: Option<&Visibility>) -> ToolResult {
    let tags = store.tags().map_err(|err| ToolError(err.to_string()))?;
    let notes = store
        .query_view_notes_with_body(false)
        .map_err(|err| ToolError(err.to_string()))?;
    let visible = notes.into_iter().filter(|note| {
        !note.locked
            && !note.trashed
            && native_note_is_visible(store, &note.id)
            && vis.is_none_or(|vis| vis.allows(&note.id))
    });
    let memberships = visible
        .flat_map(|note| {
            note.tag_ids
                .into_iter()
                .map(move |tag| (tag, note.id.clone()))
        })
        .fold(
            std::collections::HashMap::<String, Vec<String>>::new(),
            |mut memberships, (tag, note)| {
                memberships.entry(tag).or_default().push(note);
                memberships
            },
        );
    let rows: Vec<Value> = tags
        .into_iter()
        .filter_map(|tag| {
            let note_ids = memberships.get(&tag.id).cloned().unwrap_or_default();
            if note_ids.is_empty() {
                return None;
            }
            Some(json!({ "_id": tag.id, "name": tag.name, "color": tag.color, "count": note_ids.len(), "noteIds": note_ids }))
        })
        .collect();
    result(vec![text(pretty(&Value::Array(rows)))])
}

/// Returns Kanso properties parsed from a visible note's body.
fn get_properties(store: &Store, vis: Option<&Visibility>, args: &Args) -> ToolResult {
    let id = with_prefix(DocType::Note, str_arg(args, "noteId"));
    let note = native_visible_note(store, vis, &id)?;
    let body = note.get("body").and_then(Value::as_str).unwrap_or_default();
    let properties: Vec<Value> = kanso_core::properties::parse(body)
        .into_iter()
        .map(|(key, value)| json!({ "key": key, "value": property_value(&value) }))
        .collect();
    result(vec![text(pretty(
        &json!({ "noteId": id, "properties": properties }),
    ))])
}

fn property_value(value: &kanso_core::properties::Value) -> Value {
    use kanso_core::properties::Value as Property;
    match value {
        Property::Text(value) | Property::Date(value) | Property::Raw(value) => json!(value),
        Property::Number(value) => json!(value),
        Property::Bool(value) => json!(value),
        Property::List(values) => Value::Array(values.iter().map(property_value).collect()),
    }
}

/// Lists resolved outgoing links in a visible note.
fn list_note_links(store: &Store, vis: Option<&Visibility>, args: &Args) -> ToolResult {
    let id = with_prefix(DocType::Note, str_arg(args, "noteId"));
    let note = native_visible_note(store, vis, &id)?;
    let body = note.get("body").and_then(Value::as_str).unwrap_or_default();
    let mut links = Vec::new();
    for link in kanso_core::links::note_links(body) {
        let resolved = match link.target {
            kanso_core::links::LinkTarget::Note(target) => store
                .note(&target).map_err(|err| ToolError(err.to_string()))?
                .filter(|note| native_note_is_visible(store, &note.id) && vis.is_none_or(|vis| vis.allows(&note.id)))
                .map(|note| json!({ "noteId": note.id, "title": note.title, "uri": note_uri(&note.id) })),
            kanso_core::links::LinkTarget::Title(title) => store.resolve_title(&title)
                .map_err(|err| ToolError(err.to_string()))?
                .filter(|(target, _)| native_note_is_visible(store, target) && vis.is_none_or(|vis| vis.allows(target)))
                .map(|(id, title)| json!({ "noteId": id, "title": title, "uri": note_uri(&id) })),
        };
        links.push(json!({ "line": link.line, "heading": link.heading, "target": resolved }));
    }
    result(vec![text(pretty(&Value::Array(links)))])
}

/// Lists backlinks only after checking both the requested note and each result.
fn list_backlinks(store: &Store, vis: Option<&Visibility>, args: &Args) -> ToolResult {
    let id = with_prefix(DocType::Note, str_arg(args, "noteId"));
    native_visible_note(store, vis, &id)?;
    let backlinks = store
        .backlinks(&id)
        .map_err(|err| ToolError(err.to_string()))?;
    let rows: Vec<Value> = backlinks.into_iter().filter(|link| native_note_is_visible(store, &link.id) && vis.is_none_or(|vis| vis.allows(&link.id)))
        .inspect(|link| if let Some(vis) = vis { vis.show(&link.id); })
        .map(|link| json!({ "noteId": link.id, "title": link.title, "uri": note_uri(&link.id), "line": link.line, "context": link.context }))
        .collect();
    result(vec![text(pretty(&Value::Array(rows)))])
}

/// Runs the first saved query fence in a visible note through the core evaluator.
fn run_query_view(store: &Store, vis: Option<&Visibility>, args: &Args) -> ToolResult {
    let id = with_prefix(DocType::Note, str_arg(args, "noteId"));
    let note = native_visible_note(store, vis, &id)?;
    let body = note.get("body").and_then(Value::as_str).unwrap_or_default();
    let fence = kanso_core::query_view::query_fences(body)
        .into_iter()
        .next()
        .ok_or_else(|| ToolError("This note has no saved query view.".into()))?;
    let spec =
        kanso_core::query_view::parse(&fence.source).map_err(|err| ToolError(err.to_string()))?;
    let notes = store
        .query_view_notes()
        .map_err(|err| ToolError(err.to_string()))?;
    let notes: Vec<kanso_core::query_view::ViewNote> = notes
        .into_iter()
        .filter(|row| {
            native_note_is_visible(store, &row.id) && vis.is_none_or(|vis| vis.allows(&row.id))
        })
        .map(Into::into)
        .collect();
    let rows = kanso_core::query_view::evaluate(&spec, &notes, Some(&id));
    for row in &rows {
        if let Some(vis) = vis {
            vis.show(&row.id);
        }
    }
    let output = json!({ "columns": spec.labels(), "rows": rows.iter().map(|row| json!({ "noteId": row.id, "title": row.title, "uri": note_uri(&row.id), "cells": row.cells.iter().map(|cell| cell.as_ref().map(property_value)).collect::<Vec<_>>() })).collect::<Vec<_>>() });
    result(vec![text(pretty(&output))])
}

/// Lists revision metadata without returning historical note text.
fn list_versions(store: &Store, vis: Option<&Visibility>, args: &Args) -> ToolResult {
    let id = with_prefix(DocType::Note, str_arg(args, "noteId"));
    native_visible_note(store, vis, &id)?;
    let versions = store
        .revisions(&id)
        .map_err(|err| ToolError(err.to_string()))?;
    let versions: Vec<Value> = versions.into_iter().map(|version| json!({ "rev": version.rev, "savedAt": version.saved_at, "origin": version.origin, "author": version.author, "agent": version.agent })).collect();
    result(vec![text(pretty(&Value::Array(versions)))])
}

/// Reads one saved revision after checking the current note's visibility.
fn read_version(store: &Store, vis: Option<&Visibility>, args: &Args) -> ToolResult {
    let id = with_prefix(DocType::Note, str_arg(args, "noteId"));
    native_visible_note(store, vis, &id)?;
    let rev = str_arg(args, "rev");
    let version = store
        .revisions(&id)
        .map_err(|err| ToolError(err.to_string()))?
        .into_iter()
        .find(|version| version.rev == rev)
        .ok_or_else(|| ToolError("That note version was not found.".into()))?;
    let mut fields = version.fields;
    fields.insert("_id".into(), id.into());
    fields.insert("_rev".into(), rev.into());
    fields.remove(kanso_core::model::LOCK_FIELD);
    result(vec![text(pretty(&Value::Object(fields)))])
}

/// Ticks or clears the identified task through the store's atomic task edit.
fn set_task_done(store: &Store, args: &Args) -> ToolResult {
    let id = with_prefix(DocType::Note, str_arg(args, "noteId"));
    let line = args.get("line").and_then(Value::as_u64).unwrap_or_default() as u32;
    let task_text = str_arg(args, "text");
    let done = bool_arg(args, "done");
    let change = store
        .set_task_checked(
            &id,
            line,
            task_text,
            done,
            now_ms(),
            kanso_store::Origin::Api,
        )
        .map_err(|err| ToolError(err.to_string()))?;
    match change {
        kanso_store::CheckOff::Changed { rev } => result(vec![text(format!(
            "Task updated in {id} at revision {rev}."
        ))]),
        kanso_store::CheckOff::Unchanged => result(vec![text("The task already has that status.")]),
        kanso_store::CheckOff::Missing => Err(ToolError(
            "The task changed or could not be identified; no note was changed.".into(),
        )),
    }
}

/// Moves a note to an explicitly named notebook.
fn move_note(store: &Store, args: &Args) -> ToolResult {
    let id = with_prefix(DocType::Note, str_arg(args, "noteId"));
    let mut note = native_visible_note(store, None, &id)?
        .as_object()
        .cloned()
        .ok_or_else(|| ToolError("The note could not be read.".into()))?;
    note.insert(
        "bookId".into(),
        json!(with_prefix(DocType::Book, str_arg(args, "bookId"))),
    );
    note.insert("updatedAt".into(), json!(now_ms()));
    let saved = api::upsert(store, DocType::Note, Value::Object(note))?;
    result(vec![text(pretty(&saved))])
}

/// Creates an attachment from base64 and explicitly adds its image link to a note.
fn attach_file(store: &Store, args: &Args) -> ToolResult {
    let id = with_prefix(DocType::Note, str_arg(args, "noteId"));
    let mut note = api::get_doc(store, &id, None, false)?
        .as_object()
        .cloned()
        .ok_or_else(|| ToolError("The note could not be read.".into()))?;
    let bytes = decode_base64_leniently(str_arg(args, "data"));
    if bytes.len() > MAX_ATTACHMENT_SIZE {
        return Err(ToolError(format!(
            "The file is {} bytes, over the {} byte limit.",
            bytes.len(),
            MAX_ATTACHMENT_SIZE
        )));
    }
    let content_type = str_arg(args, "contentType");
    if !SUPPORTED_IMAGE_TYPES.contains(&content_type) {
        return Err(ToolError(format!(
            "Unsupported attachment type. Supported types: {}.",
            SUPPORTED_IMAGE_TYPES.join(", ")
        )));
    }
    let name = str_arg(args, "name");
    let encoded = STANDARD.encode(&bytes);
    let content_length = bytes.len();
    let file = api::upsert(
        store,
        DocType::File,
        json!({ "name": name, "contentType": content_type, "contentLength": content_length, "publicIn": [], "md5digest": Attachment::new(bytes).md5_hex(), "_attachments": { "index": { "content_type": content_type, "data": encoded } } }),
    )?;
    let file_id = js_string(file.get("id"));
    let body = note.get("body").and_then(Value::as_str).unwrap_or_default();
    let separator = if body.is_empty() || body.ends_with('\n') {
        ""
    } else {
        "\n"
    };
    note.insert(
        "body".into(),
        json!(format!(
            "{body}{separator}\n![{name}]({})\n",
            file_uri(&file_id)
        )),
    );
    note.insert("updatedAt".into(), json!(now_ms()));
    api::upsert(store, DocType::Note, Value::Object(note))?;
    result(vec![text(pretty(
        &json!({ "noteId": id, "fileId": file_id, "name": name, "contentType": content_type, "contentLength": content_length, "uri": file_uri(&file_id) }),
    ))])
}

/// Lists attachment metadata for files embedded in a visible note.
fn list_attachments(store: &Store, args: &Args) -> ToolResult {
    let id = with_prefix(DocType::Note, str_arg(args, "noteId"));
    let note = native_visible_note(store, None, &id)?;
    let body = note.get("body").and_then(Value::as_str).unwrap_or_default();
    let files: Vec<Value> = kanso_core::tasks::linked_file_ids(body)
        .into_iter()
        .filter_map(|file_id| {
            api::get_doc(store, &file_id, None, false).ok().map(|file| {
                json!({
                    "fileId": file_id,
                    "name": file.get("name"),
                    "contentType": file.get("contentType"),
                    "contentLength": file.get("contentLength"),
                    "uri": file_uri(&file_id),
                })
            })
        })
        .collect();
    result(vec![text(pretty(&Value::Array(files)))])
}

/// Reads an attachment's metadata and base64 bytes.
fn read_attachment(store: &Store, args: &Args) -> ToolResult {
    let note_id = with_prefix(DocType::Note, str_arg(args, "noteId"));
    let note = native_visible_note(store, None, &note_id)?;
    let id = with_prefix(DocType::File, str_arg(args, "fileId"));
    let body = note.get("body").and_then(Value::as_str).unwrap_or_default();
    if !kanso_core::tasks::linked_file_ids(body).contains(&id) {
        return Err(ApiError::not_found().into());
    }
    ensure_plain_attachment(store, &id)?;
    let file = api::get_doc(store, &id, None, true)?;
    result(vec![text(pretty(&file))])
}

/// Runs `resources/read` for a note link ([`NOTE_LINK_PREFIX`]`{noteId}`);
/// for an agent, only a note `vis` lets it see.
pub(crate) fn read_resource(
    state: &AppState,
    vis: Option<&Visibility>,
    uri: &str,
) -> Result<Value, RpcError> {
    let note_id = uri
        .get(..LINK_SCHEME.len())
        .filter(|scheme| scheme.eq_ignore_ascii_case(LINK_SCHEME))
        .and_then(|_| uri[LINK_SCHEME.len()..].strip_prefix("note/"))
        .filter(|id| !id.is_empty() && !id.contains(['/', ',']))
        .ok_or_else(|| {
            RpcError::invalid_params(format!("MCP error -32602: Resource {uri} not found"))
        })?;
    let id = with_prefix(DocType::Note, note_id);
    let note =
        visible_note(&state.store, vis, &id).map_err(|ToolError(text)| RpcError::internal(text))?;
    Ok(json!({
        "contents": [{ "uri": uri, "text": pretty(&note), "mimeType": "application/json" }]
    }))
}

fn text(text: impl Into<String>) -> Value {
    json!({ "type": "text", "text": text.into() })
}

fn result(blocks: Vec<Value>) -> ToolResult {
    Ok(json!({ "content": blocks }))
}

pub(crate) fn error_result(message: impl Into<String>) -> Value {
    json!({ "content": [text(message)], "isError": true })
}

/// `JSON.stringify(value, null, 2)`.
fn pretty(value: &Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_default()
}

fn str_arg<'a>(args: &'a Args, key: &str) -> &'a str {
    opt_str(args, key).unwrap_or_default()
}

fn opt_str<'a>(args: &'a Args, key: &str) -> Option<&'a str> {
    args.get(key).and_then(Value::as_str)
}

fn bool_arg(args: &Args, key: &str) -> bool {
    args.get(key).and_then(Value::as_bool).unwrap_or(false)
}

/// [`NOTE_LINK_PREFIX`]`<id without the note: prefix>`.
fn note_uri(id: &str) -> String {
    format!(
        "{NOTE_LINK_PREFIX}{}",
        id.strip_prefix("note:").unwrap_or(id)
    )
}

/// [`LINK_SCHEME`]`file:<id>`.
fn file_uri(id: &str) -> String {
    format!("{LINK_SCHEME}{}", with_prefix(DocType::File, id))
}

/// JavaScript truthiness.
fn js_truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|x| x != 0.0),
        Value::String(s) => !s.is_empty(),
        Value::Array(_) | Value::Object(_) => true,
    }
}

/// JavaScript's `String(value)`, with `None` as `undefined`.
fn js_string(value: Option<&Value>) -> String {
    match value {
        None => "undefined".into(),
        Some(Value::Null) => "null".into(),
        Some(Value::String(s)) => s.clone(),
        Some(Value::Bool(b)) => b.to_string(),
        Some(Value::Number(n)) => n.to_string(),
        Some(Value::Array(items)) => items
            .iter()
            .map(|item| match item {
                Value::Null => String::new(),
                item => js_string(Some(item)),
            })
            .collect::<Vec<_>>()
            .join(","),
        Some(Value::Object(_)) => "[object Object]".into(),
    }
}

/// JavaScript's `s.substring(0, n)`, counting UTF-16 code units. A surrogate
/// pair cut in half is dropped rather than split.
fn js_prefix(s: &str, n: usize) -> &str {
    let mut units = 0;
    for (i, c) in s.char_indices() {
        units += c.len_utf16();
        if units > n {
            return &s[..i];
        }
    }
    s
}

/// JavaScript's `String.prototype.trim`.
fn js_trim(s: &str) -> &str {
    s.trim_matches(|c: char| c.is_whitespace() || c == '\u{feff}')
}

/// A note as `search-notes` and `list-notes` show it: body cut to 200 characters.
fn summary(mut note: Value) -> Value {
    if let Some(Value::String(body)) = note.get_mut("body") {
        let cut = js_prefix(body, 200).len();
        body.truncate(cut);
    }
    note
}

fn read_note(store: &Store, vis: Option<&Visibility>, args: &Args) -> ToolResult {
    let id = with_prefix(DocType::Note, str_arg(args, "noteId"));
    let note = visible_note(store, vis, &id)?;
    let source = note
        .get("sourceTemplateId")
        .filter(|v| js_truthy(v))
        .map(|v| js_string(Some(v)))
        // A template an agent may not see is as good as missing.
        .filter(|source| vis.is_none_or(|vis| vis.allows(source)));
    let template = match &source {
        // A missing template (e.g. deleted) only skips the embed.
        Some(source) if bool_arg(args, "includeTemplate") => {
            visible_note(store, vis, source).ok().map(|t| {
                format!(
                    "# Source template: {}\n\n{}",
                    js_string(t.get("title")),
                    js_string(t.get("body"))
                )
            })
        }
        _ => None,
    };
    let mut blocks = vec![text(pretty(&note))];
    if let Some(source) = &source {
        blocks.push(json!({
            "type": "resource_link",
            "uri": note_uri(source),
            "name": "Source template",
            "mimeType": "application/json",
            "description": SOURCE_TEMPLATE_DESCRIPTION,
        }));
    }
    if let Some(template) = template {
        blocks.push(text(template));
    }
    result(blocks)
}

fn read_backlinks(store: &Store, args: &Args) -> ToolResult {
    let link = note_uri(&with_prefix(DocType::Note, str_arg(args, "noteId")));
    let query = NoteQuery {
        keyword: link.clone(),
        limit: Some(100),
        ..NoteQuery::default()
    };
    let notes = api::list_notes(store, &query)?;
    if notes.is_empty() {
        return result(vec![text(format!("No backlinks found for {link}."))]);
    }
    result(
        notes
            .iter()
            .map(|note| {
                let name = note
                    .get("title")
                    .filter(|t| js_truthy(t))
                    .map_or_else(|| "Untitled note".to_owned(), |t| js_string(Some(t)));
                json!({
                    "type": "resource_link",
                    "uri": note_uri(&js_string(note.get("_id"))),
                    "name": name,
                    "mimeType": "application/json",
                })
            })
            .collect(),
    )
}

/// Lists custom template notes from Kanso's Templates notebook.
fn list_templates(store: &Store, vis: Option<&Visibility>, args: &Args) -> ToolResult {
    let Some(book) = store
        .find_role_book(kanso_core::roles::Role::Templates)
        .map_err(|err| ToolError(err.to_string()))?
    else {
        return summaries(Vec::new());
    };
    let query = NoteQuery {
        keyword: list_notes_keyword(Some(&book), &[], opt_str(args, "keyword")),
        limit: Some(100),
        ..NoteQuery::default()
    };
    let notes = api::list_notes(store, &query)?
        .into_iter()
        .filter(|note| {
            note.get("_id").and_then(Value::as_str).is_some_and(|id| {
                native_note_is_visible(store, id) && vis.is_none_or(|vis| vis.allows(id))
            })
        })
        .inspect(|note| {
            if let (Some(vis), Some(id)) = (vis, note.get("_id").and_then(Value::as_str)) {
                vis.show(id);
            }
        })
        .collect();
    summaries(notes)
}

/// `kanso-get-outline`: the note's headings as JSON.
fn get_outline(store: &Store, vis: Option<&Visibility>, args: &Args) -> ToolResult {
    let id = with_prefix(DocType::Note, str_arg(args, "noteId"));
    // The same failure as reading a note that isn't there.
    visible_note(store, vis, &id)?;
    let headings = store
        .outline(&id)
        .map_err(|err| ToolError(err.to_string()))?;
    if headings.is_empty() {
        return result(vec![text(format!("No headings in {}.", note_uri(&id)))]);
    }
    let headings = serde_json::to_value(headings).unwrap_or_default();
    result(vec![text(pretty(&headings))])
}

/// `kanso-list-tasks`: tasks across notes, with their notes.
fn list_tasks(store: &Store, vis: Option<&Visibility>, args: &Args) -> ToolResult {
    let status = str_arg(args, "status");
    let limit = args.get("limit").and_then(Value::as_f64).unwrap_or(100.0);
    let book_id = opt_str(args, "bookId").map(|id| with_prefix(DocType::Book, id));
    if let (Some(vis), Some(book)) = (vis, &book_id)
        && !vis.book_visible(book)
    {
        return result(vec![text("No tasks match.")]);
    }
    let query = kanso_store::TaskQuery {
        book_id,
        include_done: status != "open",
        due_before: opt_str(args, "dueBefore").map(str::to_owned),
        due_after: opt_str(args, "dueAfter").map(str::to_owned),
        // Done ones, and an agent's unseen ones, are left out below, after
        // the limit would cut.
        limit: (status != "done" && vis.is_none())
            .then(|| api::count_param("limit", limit))
            .transpose()?,
    };
    let limit = api::count_param("limit", limit)?;
    let tasks: Vec<Value> = store
        .tasks(&query)
        .map_err(|err| ToolError(err.to_string()))?
        .into_iter()
        .filter(|t| status != "done" || t.checked)
        .filter(|t| {
            native_note_is_visible(store, &t.note_id)
                && vis.is_none_or(|vis| vis.allows(&t.note_id))
        })
        .take(limit)
        .inspect(|t| {
            if let Some(vis) = vis {
                vis.show(&t.note_id);
            }
        })
        .map(|t| {
            json!({
                "text": t.text,
                "done": t.checked,
                "due": t.due,
                "line": t.line,
                "noteId": t.note_id,
                "noteTitle": t.note_title,
                "bookId": t.book_id,
                "uri": note_uri(&t.note_id),
            })
        })
        .collect();
    if tasks.is_empty() {
        return result(vec![text("No tasks match.")]);
    }
    result(vec![text(pretty(&Value::Array(tasks)))])
}

/// `kanso-related-notes`: the notes nearest a note by the embeddings the
/// app keeps, without loading a model. An empty list, and why, while
/// related notes are off or the note isn't indexed.
fn related_notes(store: &Store, vis: Option<&Visibility>, args: &Args) -> ToolResult {
    let id = with_prefix(DocType::Note, str_arg(args, "noteId"));
    // The same failure as reading a note that isn't there.
    native_visible_note(store, vis, &id)?;
    let limit = args
        .get("limit")
        .and_then(Value::as_i64)
        .unwrap_or(5)
        .clamp(1, 20) as usize;
    let model = store
        .embedding_model()
        .map_err(|err| ToolError(err.to_string()))?;
    let Some(model) = model else {
        return result(vec![
            text("[]"),
            text(
                "Related notes are off. The user can turn them on in Kanso's Settings › General › Related notes; they run on the user's device.",
            ),
        ]);
    };
    let wide = if vis.is_some() {
        limit * SCOPED_WIDEN
    } else {
        limit
    };
    let found = store
        .related_notes(&id, &model, wide)
        .map_err(|err| ToolError(err.to_string()))?;
    let Some(found) = found else {
        return result(vec![
            text("[]"),
            text(format!(
                "{} isn't indexed for related notes yet; Kanso indexes notes in the background while it is open.",
                note_uri(&id)
            )),
        ]);
    };
    let notes: Vec<Value> = found
        .into_iter()
        .filter(|note| {
            native_note_is_visible(store, &note.id) && vis.is_none_or(|vis| vis.allows(&note.id))
        })
        .take(limit)
        .inspect(|note| {
            if let Some(vis) = vis {
                vis.show(&note.id);
            }
        })
        .map(|note| {
            json!({
                "noteId": note.id,
                "title": note.title,
                "uri": note_uri(&note.id),
                "similarity": (f64::from(note.score.max(0.)) * 1000.).round() / 1000.,
            })
        })
        .collect();
    result(vec![text(pretty(&Value::Array(notes)))])
}

/// `kanso-search-by-meaning`: the notes nearest a search by the embeddings
/// the app keeps, with the model the app lends the store. An empty list,
/// and why, while related notes are off or no model is loaded here.
fn search_by_meaning(store: &Store, vis: Option<&Visibility>, args: &Args) -> ToolResult {
    let query = str_arg(args, "query").trim();
    if query.is_empty() {
        return Err(ToolError("Say what the notes should be about.".into()));
    }
    let limit = args
        .get("limit")
        .and_then(Value::as_i64)
        .unwrap_or(5)
        .clamp(1, 20) as usize;
    let wide = if vis.is_some() {
        limit * SCOPED_WIDEN
    } else {
        limit
    };
    let found = store
        .search_by_meaning(query, wide)
        .map_err(|err| ToolError(err.to_string()))?;
    let found = match found {
        MeaningSearch::Off => {
            return result(vec![
                text("[]"),
                text(
                    "Search by meaning uses related notes, which are off. The user can turn them on in Kanso's Settings › General › Related notes; they run on the user's device. search-notes finds notes by their words meanwhile.",
                ),
            ]);
        }
        MeaningSearch::NoModel => {
            return result(vec![
                text("[]"),
                text(
                    "Kanso's language model isn't loaded where this API runs: it loads in the Kanso app, a little after it opens, and a server started with `kanso serve` has none. search-notes finds notes by their words meanwhile.",
                ),
            ]);
        }
        MeaningSearch::Found(found) => found,
    };
    let found = found
        .into_iter()
        .filter(|note| {
            native_note_is_visible(store, &note.id) && vis.is_none_or(|vis| vis.allows(&note.id))
        })
        .collect();
    let notes: Vec<Value> = kanso_store::blend_meaning(&[], found, MEANING_SIMILARITY, limit)
        .into_iter()
        .inspect(|note| {
            if let Some(vis) = vis {
                vis.show(&note.id);
            }
        })
        .map(|note| {
            json!({
                "noteId": note.id,
                "title": note.title,
                "uri": note_uri(&note.id),
                "similarity": (f64::from(note.score.max(0.)) * 1000.).round() / 1000.,
            })
        })
        .collect();
    if notes.is_empty() {
        return result(vec![
            text("[]"),
            text("No note is close enough in meaning."),
        ]);
    }
    result(vec![text(pretty(&Value::Array(notes)))])
}

/// `kanso-resolve-wikilink`: the note a wiki-link names.
fn resolve_wikilink(store: &Store, vis: Option<&Visibility>, args: &Args) -> ToolResult {
    let link = str_arg(args, "link").trim();
    let inner = link
        .strip_prefix("[[")
        .and_then(|rest| rest.strip_suffix("]]"))
        .unwrap_or(link);
    let Some((title, heading, alias)) = kanso_core::links::parse_target(inner) else {
        return Err(ToolError(format!(
            "`{link}` is not a wiki-link: it needs a title, without brackets or line breaks inside."
        )));
    };
    let found = store
        .resolve_title(&title)
        .map_err(|err| ToolError(err.to_string()))?
        .filter(|(id, _)| {
            native_note_is_visible(store, id) && vis.is_none_or(|vis| vis.allows(id))
        });
    let Some((id, note_title)) = found else {
        return Err(ToolError(format!("No note is titled \"{title}\".")));
    };
    if let Some(vis) = vis {
        vis.show(&id);
    }
    let mut answer = Map::new();
    answer.insert("noteId".into(), json!(id));
    answer.insert("title".into(), json!(note_title));
    answer.insert("uri".into(), json!(note_uri(&id)));
    if let Some(alias) = alias {
        answer.insert("alias".into(), json!(alias));
    }
    if let Some(heading) = heading {
        let outline = store
            .outline(&id)
            .map_err(|err| ToolError(err.to_string()))?;
        let found = kanso_core::outline::find_heading(&outline, &heading);
        answer.insert(
            "heading".into(),
            match found {
                Some(h) => json!({ "text": h.text, "level": h.level, "line": h.line }),
                None => json!({ "text": heading, "found": false }),
            },
        );
    }
    result(vec![text(pretty(&Value::Object(answer)))])
}

fn summaries(notes: Vec<Value>) -> ToolResult {
    let notes = Value::Array(notes.into_iter().map(summary).collect());
    result(vec![text(pretty(&notes))])
}

fn search_notes(store: &Store, vis: Option<&Visibility>, args: &Args) -> ToolResult {
    const LIMIT: usize = 10;
    let query = NoteQuery {
        keyword: str_arg(args, "keyword").to_owned(),
        limit: Some(if vis.is_some() {
            LIMIT * SCOPED_WIDEN
        } else {
            LIMIT
        }),
        ..NoteQuery::default()
    };
    summaries(visible_rows(vis, api::list_notes(store, &query)?, LIMIT))
}

/// The keyword `list-notes` searches for: `bookId:<id> tagId:<tag>… <keyword>`.
pub(crate) fn list_notes_keyword(
    book_id: Option<&str>,
    tag_ids: &[Value],
    keyword: Option<&str>,
) -> String {
    let book_filter = match book_id {
        Some(id) if !id.is_empty() => {
            format!("bookId:{}", id.split(':').nth(1).unwrap_or("undefined"))
        }
        _ => String::new(),
    };
    let tag_filter = tag_ids
        .iter()
        .map(|tag| format!("tagId:{}", js_string(Some(tag))))
        .collect::<Vec<_>>()
        .join(" ");
    js_trim(&format!(
        "{book_filter} {tag_filter} {}",
        keyword.unwrap_or_default()
    ))
    .to_owned()
}

fn list_notes(store: &Store, vis: Option<&Visibility>, args: &Args) -> ToolResult {
    if let (Some(vis), Some(book)) = (vis, opt_str(args, "bookId"))
        && !book.is_empty()
        && !vis.book_visible(book)
    {
        return summaries(Vec::new());
    }
    let tag_ids = args
        .get("tagIds")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    let limit = args.get("limit").and_then(Value::as_f64).unwrap_or(100.0);
    let limit = api::count_param("limit", limit)?;
    let query = NoteQuery {
        keyword: list_notes_keyword(opt_str(args, "bookId"), tag_ids, opt_str(args, "keyword")),
        limit: Some(if vis.is_some() {
            limit.saturating_mul(SCOPED_WIDEN)
        } else {
            limit
        }),
        skip: 0,
        sort: api::parse_sort(str_arg(args, "sort")),
        descending: args
            .get("descending")
            .and_then(Value::as_bool)
            .unwrap_or(true),
    };
    summaries(visible_rows(vis, api::list_notes(store, &query)?, limit))
}

fn upsert(store: &Store, doc_type: DocType, args: Args) -> ToolResult {
    let res = api::upsert(store, doc_type, Value::Object(args))?;
    result(vec![text(pretty(&res))])
}

/// `update-tag` and `update-book`: post the given fields to `/<_id>`.
fn merge(store: &Store, mut args: Args, stamp: bool) -> ToolResult {
    let id = js_string(args.remove("_id").as_ref());
    if stamp {
        args.insert("updatedAt".into(), now_ms().into());
    }
    let res = api::merge(store, &id, Value::Object(args))?;
    result(vec![text(pretty(&res))])
}

fn update_note(store: &Store, args: Args) -> ToolResult {
    let existing = api::get_doc(
        store,
        str_arg(&args, "_id"),
        Some(str_arg(&args, "_rev")),
        false,
    )?;
    let mut note = match existing {
        Value::Object(note) => note,
        _ => Map::new(),
    };
    note.extend(args);
    note.insert("updatedAt".into(), now_ms().into());
    upsert_note(store, note)
}

fn upsert_note(store: &Store, note: Args) -> ToolResult {
    let res = api::upsert(store, DocType::Note, Value::Object(note))?;
    result(vec![text(pretty(&res))])
}

/// Where `old` occurs in `body`, when it occurs exactly once. Overlapping
/// occurrences count, as with JavaScript's `indexOf(old, first + 1)`.
pub(crate) enum Occurrence {
    None,
    Once(usize),
    Many,
}

pub(crate) fn find_once(body: &str, old: &str) -> Occurrence {
    let Some(first) = body.find(old) else {
        return Occurrence::None;
    };
    let next = first + body[first..].chars().next().map_or(1, char::len_utf8);
    if body[next..].contains(old) {
        Occurrence::Many
    } else {
        Occurrence::Once(first)
    }
}

fn patch_note(store: &Store, args: &Args) -> ToolResult {
    let existing = api::get_doc(
        store,
        str_arg(args, "_id"),
        Some(str_arg(args, "_rev")),
        false,
    )?;
    let mut note = match existing {
        Value::Object(note) => note,
        _ => Map::new(),
    };
    let body = note.get("body").and_then(Value::as_str).unwrap_or_default();
    let old = str_arg(args, "old_string");
    let new = str_arg(args, "new_string");
    let patched = match find_once(body, old) {
        Occurrence::None => {
            return Err(ToolError(
                "Failed to patch: `old_string` was not found in the note body.".into(),
            ));
        }
        Occurrence::Many => {
            return Err(ToolError(
                "Failed to patch: `old_string` matches multiple locations in the note body. Provide more surrounding context to make it unique.".into(),
            ));
        }
        Occurrence::Once(at) => format!("{}{new}{}", &body[..at], &body[at + old.len()..]),
    };
    note.insert("body".into(), patched.into());
    note.insert("updatedAt".into(), now_ms().into());
    upsert_note(store, note)
}

fn list(store: &Store, doc_type: DocType) -> ToolResult {
    let docs = Value::Array(api::list_docs(store, doc_type, None, 0)?);
    result(vec![text(pretty(&docs))])
}

fn read(store: &Store, doc_type: DocType, args: &Args, key: &str) -> ToolResult {
    let id = with_prefix(doc_type, str_arg(args, key));
    let doc = api::get_doc(store, &id, None, false)?;
    result(vec![text(pretty(&doc))])
}

/// Node's `Buffer.from(data, 'base64')`: accepts both alphabets, padding or
/// not, and skips characters outside them.
fn decode_base64_leniently(data: &str) -> Vec<u8> {
    const LENIENT: GeneralPurpose = GeneralPurpose::new(
        &base64::alphabet::STANDARD,
        GeneralPurposeConfig::new()
            .with_decode_padding_mode(DecodePaddingMode::Indifferent)
            .with_decode_allow_trailing_bits(true),
    );
    let mut clean: String = data
        .split('=')
        .next()
        .unwrap_or_default()
        .chars()
        .filter_map(|c| match c {
            '-' => Some('+'),
            '_' => Some('/'),
            c if c.is_ascii_alphanumeric() || c == '+' || c == '/' => Some(c),
            _ => None,
        })
        .collect();
    if clean.len() % 4 == 1 {
        clean.pop();
    }
    LENIENT.decode(clean).unwrap_or_default()
}

/// The image type for a path's extension, as `inferImageContentType` does.
fn infer_image_type(path: &str) -> Option<&'static str> {
    let ext = Path::new(path).extension()?.to_str()?.to_lowercase();
    match ext.as_str() {
        "png" => Some("image/png"),
        "jpeg" | "jpg" => Some("image/jpeg"),
        "svg" | "svg+xml" => Some("image/svg+xml"),
        "gif" => Some("image/gif"),
        "heic" => Some("image/heic"),
        "heif" => Some("image/heif"),
        _ => None,
    }
}

const NO_LOCAL_FILES: &str =
    "local file access is only available when the server is bound to a loopback address";

fn create_file(store: &Store, local_files: bool, args: &Args) -> ToolResult {
    let file_path = opt_str(args, "filePath");
    let data = opt_str(args, "data");
    if file_path.is_some() == data.is_some() {
        return Err(ToolError(
            "Failed to create the attachment: provide exactly one of `filePath` or `data`.".into(),
        ));
    }
    let mut name = opt_str(args, "name").map(str::to_owned);
    let mut content_type = opt_str(args, "contentType").map(str::to_owned);
    let bytes = match (file_path, data) {
        (Some(path), _) => {
            if !local_files {
                return Err(ToolError(format!(
                    "Failed to read the file at {path}: {NO_LOCAL_FILES}"
                )));
            }
            let bytes = std::fs::read(path)
                .map_err(|err| ToolError(format!("Failed to read the file at {path}: {err}")))?;
            content_type = content_type.or_else(|| infer_image_type(path).map(str::to_owned));
            name = name.or_else(|| {
                Path::new(path)
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
            });
            bytes
        }
        (None, Some(data)) => decode_base64_leniently(data),
        (None, None) => unreachable!("checked above"),
    };
    let Some(content_type) = content_type else {
        let from = file_path.map(|p| format!(" from {p}")).unwrap_or_default();
        return Err(ToolError(format!(
            "Failed to create the attachment: could not infer the content type{from}. Pass `contentType` explicitly. Supported types: {}.",
            SUPPORTED_IMAGE_TYPES.join(", ")
        )));
    };
    let encoded = STANDARD.encode(&bytes);
    let content_length = encoded.len() * 3 / 4;
    if content_length > MAX_ATTACHMENT_SIZE {
        return Err(ToolError(format!(
            "Failed to create the attachment: the file is {content_length} bytes, over the {MAX_ATTACHMENT_SIZE} byte limit."
        )));
    }
    let name = name.unwrap_or_else(|| {
        format!(
            "image.{}",
            content_type.strip_prefix("image/").unwrap_or(&content_type)
        )
    });
    let body = json!({
        "name": name,
        "contentType": content_type,
        "contentLength": content_length,
        "publicIn": [],
        "md5digest": Attachment::new(bytes).md5_hex(),
        "_attachments": { "index": { "content_type": content_type, "data": encoded } },
    });
    let res = api::upsert(store, DocType::File, body)?;
    let embed = format!(
        "Markdown to embed this attachment in a note body:\n\n![{name}]({})",
        file_uri(&js_string(res.get("id")))
    );
    result(vec![text(pretty(&res)), text(embed)])
}

/// Image types an MCP `image` block can carry.
fn is_viewable_image(content_type: &str) -> bool {
    matches!(
        content_type,
        "image/png" | "image/jpeg" | "image/jpg" | "image/gif"
    )
}

/// Refuse a sealed attachment before any tool reads or writes its bytes.
fn ensure_plain_attachment(store: &Store, id: &str) -> Result<(), ToolError> {
    match store
        .attachment_state(id)
        .map_err(|err| ToolError(err.to_string()))?
    {
        AttachmentState::Sealed => Err(ToolError(
            "That attachment is in a locked note. Open the note in Kanso and unlock it to use the file."
                .into(),
        )),
        AttachmentState::Plain | AttachmentState::Missing => Ok(()),
    }
}

fn read_file(store: &Store, local_files: bool, args: &Args) -> ToolResult {
    let id = with_prefix(DocType::File, str_arg(args, "fileId"));
    ensure_plain_attachment(store, &id)?;
    let output_path = opt_str(args, "outputPath");
    let include_image = args
        .get("includeImage")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let needs_data = include_image || output_path.is_some();
    let file = api::get_doc(store, &id, None, needs_data)?;
    // `const { _attachments, ...metadata } = file`, keeping the key order.
    let metadata: Map<String, Value> = file
        .as_object()
        .into_iter()
        .flatten()
        .filter(|(key, _)| *key != "_attachments")
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect();
    let metadata = Value::Object(metadata);
    let data = file
        .get("_attachments")
        .and_then(|a| a.get("index"))
        .and_then(|index| index.get("data"))
        .and_then(Value::as_str);
    let mut blocks = vec![text(pretty(&metadata))];
    if needs_data {
        let content_type = js_string(file.get("contentType"));
        blocks.push(match (data, output_path) {
            (None, _) => text("The attachment data could not be read from the document."),
            (Some(data), Some(path)) => {
                if !local_files {
                    return Err(ToolError(format!(
                        "Failed to write the attachment to {path}: {NO_LOCAL_FILES}"
                    )));
                }
                std::fs::write(path, decode_base64_leniently(data)).map_err(|err| {
                    ToolError(format!("Failed to write the attachment to {path}: {err}"))
                })?;
                text(format!("Attachment written to {path}"))
            }
            (Some(_), None) if !is_viewable_image(&content_type) => text(format!(
                "The attachment is a {content_type} file, which cannot be displayed inline. Pass `outputPath` to write it to disk."
            )),
            (Some(data), None) => json!({
                "type": "image",
                "data": data,
                "mimeType": if content_type == "image/jpg" { "image/jpeg" } else { content_type.as_str() },
            }),
        });
    }
    result(blocks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefixes_count_utf16_units() {
        assert_eq!(js_prefix("abc", 2), "ab");
        assert_eq!(js_prefix("ab", 5), "ab");
        // 😀 is two UTF-16 units; a cut through it drops it.
        assert_eq!(js_prefix("a😀b", 2), "a");
        assert_eq!(js_prefix("a😀b", 3), "a😀");
        assert_eq!(js_prefix("ééé", 2), "éé");
    }

    #[test]
    fn occurrences_overlap_like_index_of() {
        assert!(matches!(find_once("aaa", "aa"), Occurrence::Many));
        assert!(matches!(find_once("abc", "b"), Occurrence::Once(1)));
        assert!(matches!(find_once("abc", "x"), Occurrence::None));
        assert!(matches!(find_once("é😀é", "é"), Occurrence::Many));
        assert!(matches!(find_once("😀ab", "ab"), Occurrence::Once(4)));
    }

    #[test]
    fn list_notes_keywords_match_the_node_server() {
        let tags = [json!("tag:a"), json!("tag:b")];
        assert_eq!(
            list_notes_keyword(Some("book:xyz"), &tags, Some("hello")),
            "bookId:xyz tagId:tag:a tagId:tag:b hello"
        );
        assert_eq!(list_notes_keyword(None, &[], None), "");
        assert_eq!(list_notes_keyword(Some(""), &[], Some("x")), "x");
        assert_eq!(
            list_notes_keyword(Some("trash"), &[], None),
            "bookId:undefined"
        );
        assert_eq!(
            list_notes_keyword(None, &tags, None),
            "tagId:tag:a tagId:tag:b"
        );
    }

    #[test]
    fn base64_decoding_is_as_lenient_as_node() {
        assert_eq!(decode_base64_leniently("aGVsbG8="), b"hello");
        assert_eq!(decode_base64_leniently("aGVsbG8"), b"hello");
        assert_eq!(decode_base64_leniently("aGVs\nbG8="), b"hello");
        assert_eq!(decode_base64_leniently("-_8="), [0xfb, 0xff]);
    }

    #[test]
    fn image_types_are_inferred_from_extensions() {
        assert_eq!(infer_image_type("/a/b.PNG"), Some("image/png"));
        assert_eq!(infer_image_type("x.jpg"), Some("image/jpeg"));
        assert_eq!(infer_image_type("x.svg"), Some("image/svg+xml"));
        assert_eq!(infer_image_type(".png"), None);
        assert_eq!(infer_image_type("x.bmp"), None);
    }
}
