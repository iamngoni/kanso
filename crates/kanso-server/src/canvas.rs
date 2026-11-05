//! Canvases through the local API and MCP (issue #45, step 12): listing
//! them as short Markdown, reading one as JSON, and changing one with a batch
//! of validated operations.
//!
//! A canvas is a note (`doctype: "canvas"`) whose body is one `canvas` fence.
//! Everything here goes through the note's own read and write paths, so a
//! locked canvas is served empty or as missing, and the store's rules (the
//! 1 MB budget, "a shared canvas's body is the CRDT's") hold for every
//! caller. An edit never rewrites the fence as text: it applies operations
//! to the parsed canvas, each stamped like an edit in the canvas view, and
//! writes it back as one revision with the API as its origin.

use std::collections::HashSet;
use std::sync::Arc;

use kanso_core::canvas::{self, Batch, BatchContext, Canvas, Op, Refusal};
use kanso_core::model::SHARE_FIELD;
use kanso_core::{DocType, Note, now_ms};
use kanso_store::{Origin, Put, Store, StoreError};
use serde_json::{Map, Value, json};

use crate::api;
use crate::error::{ApiError, ApiResult};
use crate::keyed::Key;
use crate::scope::Visibility;

/// How many times an edit without `ifRev` re-reads and re-applies when
/// someone else writes the note between its read and its write.
const RETRIES: usize = 5;

/// What [`SharedCanvases::edit`] asks of the canvas as the room has it: the
/// canvas wanted and the note's new title, if any, or why not.
pub type CanvasChange<'a> = dyn FnMut(&Canvas) -> Result<(Canvas, Option<String>), String> + 'a;

/// What the local API needs from the app to change a canvas that is shared:
/// its CRDT lives in the signed-in account's engine, which the server does
/// not hold.
pub trait SharedCanvases: Send + Sync {
    /// Applies `change` to shared canvas `note_id` as one change to its
    /// CRDT, so people drawing on it see all of it arrive at once, like a
    /// collaborator's. `change` is given the canvas as the CRDT has it (the
    /// library brought up to it first) and returns the canvas it wants and
    /// the note's new title, if any, or why not. `change` reads the library
    /// and must not call back into the host.
    ///
    /// `Ok(false)` when this device cannot reach that canvas's room (signed
    /// out), and nothing was changed.
    fn edit(&self, note_id: &str, change: &mut CanvasChange<'_>) -> Result<bool, String>;
}

/// What a caller's cards may point at, looked up before any edit: the notes
/// a batch names that the caller sees, and the pictures already on this
/// canvas (its attachments and the files its cards show) that the caller
/// could read. So a card never brings in another note's attachment, one
/// sealed with a locked note (#44), or one the key can't read.
struct Reach {
    notes: HashSet<String>,
    files: HashSet<String>,
}

impl Reach {
    fn of(store: &Store, key: &Key, vis: &Visibility, note: &Note, ops: &[Op]) -> ApiResult<Reach> {
        let notes = ops
            .iter()
            .filter_map(|op| match op {
                Op::AddNode {
                    file: Some(target), ..
                } if target.starts_with(canvas::NOTE_PREFIX) => Some(target.clone()),
                _ => None,
            })
            .filter(|target| vis.note(target).is_some())
            .collect();
        let mut held: HashSet<String> = store.files_of_note(&note.id)?.into_iter().collect();
        if let Ok(current) = canvas_of(note) {
            held.extend(
                current
                    .nodes
                    .iter()
                    .filter_map(|n| n.file.clone())
                    .filter(|f| f.starts_with(canvas::ATTACHMENT_PREFIX)),
            );
        }
        let files = held
            .into_iter()
            .filter(|file| key.sees(vis, file) && !sealed(store, file))
            .collect();
        Ok(Reach { notes, files })
    }
}

/// Whether attachment `id` is sealed with a locked note (#44). Fail closed.
fn sealed(store: &Store, id: &str) -> bool {
    match store.get(id) {
        Ok(Some(doc)) => doc.fields.contains_key(kanso_core::model::FILE_LOCK_FIELD),
        Ok(None) => false,
        Err(_) => true,
    }
}

/// The host, when the app gave one.
pub(crate) type Host = Option<Arc<dyn SharedCanvases>>;

/// What a person is told for a canvas that is shared when the app can't
/// reach its room.
const SHARED_OFFLINE: &str = "This canvas is shared, and it can only be changed through the live room: sign in to Kanso and try again, or change it in the app.";

/// A canvas note the caller may open: live, not locked, not in a locked
/// notebook. Anything else is "not found", as a missing note is.
fn open(store: &Store, id: &str) -> ApiResult<Note> {
    if DocType::of_id(id) != Some(DocType::Note) {
        return Err(ApiError::not_found());
    }
    if store.note_is_locked(id)? || store.note_in_locked_book(id)? {
        return Err(ApiError::not_found());
    }
    let note = store.note(id)?.ok_or_else(ApiError::not_found)?;
    if note.doctype != "canvas" {
        return Err(ApiError::bad_request(
            "The note isn't a canvas: read it with read-note.",
        ));
    }
    Ok(note)
}

/// The canvas in a canvas note's body. A new canvas has an empty body.
fn canvas_of(note: &Note) -> ApiResult<Canvas> {
    if note.body.trim().is_empty() {
        return Ok(Canvas::default());
    }
    canvas::parse(&note.body)
        .map_err(|why| ApiError::internal(format!("The canvas can't be read: {why}")))
}

fn is_shared(note: &Note) -> bool {
    note.extra
        .get(SHARE_FIELD)
        .and_then(Value::as_object)
        .is_some()
}

/// How a row of `GET /notes` shows a canvas: its body is the short Markdown
/// summary (not a megabyte of JSON), and `size` says how big the document
/// is. Other rows are left as they are. A locked canvas is served with an
/// empty body, so it has nothing to summarize.
pub(crate) fn summarize(mut row: Value) -> Value {
    if row.get("doctype").and_then(Value::as_str) != Some("canvas") {
        return row;
    }
    let Some(fields) = row.as_object_mut() else {
        return row;
    };
    let body = fields
        .get("body")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned();
    let summary = canvas::parse(&body)
        .map(|canvas| canvas.summary())
        .unwrap_or_default();
    fields.insert("size".into(), body.len().into());
    fields.insert("body".into(), Value::String(summary));
    row
}

/// `GET /note:<id>`: the stored note, and for a canvas `canvasJson`, the
/// fence's inner JSON, so an agent gets the document without stripping
/// Markdown. For a locked note both are empty.
pub(crate) fn with_canvas_json(mut doc: Value) -> Value {
    if doc.get("doctype").and_then(Value::as_str) != Some("canvas") {
        return doc;
    }
    let json = doc
        .get("body")
        .and_then(Value::as_str)
        .and_then(canvas::fenced)
        .unwrap_or_default()
        .to_owned();
    if let Some(fields) = doc.as_object_mut() {
        fields.insert("canvasJson".into(), Value::String(json));
    }
    doc
}

/// `GET /note:<id>/canvas[?group=<id>]`: the canvas as JSON, live items
/// only, each with its `kansoVersion`, and the note's `_rev`. With `group`,
/// only what lies inside that group's rectangle.
pub(crate) fn read(store: &Store, id: &str, group: Option<&str>) -> ApiResult<Value> {
    let note = open(store, id)?;
    let full = canvas_of(&note)?;
    let view = match group {
        None => full.live(),
        Some(group) => full
            .group_view(group)
            .ok_or_else(|| ApiError::not_found_because(format!("There is no group “{group}”.")))?,
    };
    Ok(json!({
        "_id": note.id,
        "_rev": note.rev,
        "title": note.title,
        "bookId": note.book_id,
        "shared": is_shared(&note),
        "size": note.body.len(),
        "canvas": view,
    }))
}

/// A new canvas note, empty or from a canvas document (`canvas`), through
/// the same write as any note, so it is validated like one.
pub(crate) fn create(
    store: &Store,
    vis: &Visibility,
    mut args: Map<String, Value>,
) -> ApiResult<Value> {
    let document = args.remove("canvas");
    let canvas = match document {
        None | Some(Value::Null) => Canvas::default(),
        Some(value) => {
            let mut canvas: Canvas = serde_json::from_value(value)
                .map_err(|err| ApiError::bad_request(format!("That isn't a canvas: {err}")))?;
            stamp(&mut canvas, now_ms());
            canvas.validate().map_err(|why| {
                ApiError::bad_request(format!("That isn't a valid canvas: {why}"))
            })?;
            check_new_cards(&canvas, vis)?;
            canvas
        }
    };
    args.insert("doctype".into(), "canvas".into());
    args.insert("body".into(), canvas::to_body(&canvas).into());
    if !args.contains_key("bookId") {
        let inbox = store.role_book(kanso_core::roles::Role::Inbox, Origin::Api)?;
        args.insert("bookId".into(), inbox.into());
    }
    api::upsert(store, DocType::Note, Value::Object(args))
}

/// A new canvas has no attachments, so a picture card on it would show
/// another note's: refused, as is a card to a note the caller can't see.
/// Add pictures with `kanso-attach-file` and `kanso-edit-canvas`.
fn check_new_cards(canvas: &Canvas, vis: &Visibility) -> ApiResult<()> {
    for node in canvas.nodes.iter().filter(|n| !n.deleted) {
        let Some(target) = node.file.as_deref() else {
            continue;
        };
        if target.starts_with(canvas::ATTACHMENT_PREFIX) {
            return Err(ApiError::bad_request(
                "A new canvas has no attachments, so it can't hold a picture card. Create the canvas, attach the file to it with kanso-attach-file, then add the card with kanso-edit-canvas.",
            ));
        }
        if target.starts_with(canvas::NOTE_PREFIX) && vis.note(target).is_none() {
            return Err(ApiError::bad_request(format!(
                "There is no note “{target}”."
            )));
        }
    }
    Ok(())
}

/// Gives items of a canvas that came without stamps the first version.
fn stamp(canvas: &mut Canvas, now: i64) {
    for node in &mut canvas.nodes {
        if node.kanso_version == 0 {
            node.kanso_version = 1;
            node.kanso_updated = now;
        }
        for stroke in &mut node.strokes {
            if stroke.kanso_version == 0 {
                stroke.kanso_version = 1;
                stroke.kanso_updated = now;
            }
        }
    }
    for edge in &mut canvas.edges {
        if edge.kanso_version == 0 {
            edge.kanso_version = 1;
            edge.kanso_updated = now;
        }
    }
}

/// `PATCH /note:<id>/canvas`: applies a batch of operations, all or none.
/// Answers `{ok, id, rev, created: {nodes, edges, strokes}}`. `key` and
/// `vis` say what the batch's cards may point at.
pub(crate) fn edit(
    store: &Store,
    host: &Host,
    key: &Key,
    vis: &Visibility,
    id: &str,
    body: &Value,
) -> ApiResult<Value> {
    let batch = Batch::from_json(body).map_err(ApiError::bad_request)?;
    let note = open(store, id)?;
    let reach = Reach::of(store, key, vis, &note, &batch.ops)?;
    let exists = |n: &str| reach.notes.contains(n);
    let attached = |f: &str| reach.files.contains(f);
    if is_shared(&note) {
        let context = BatchContext {
            now: now_ms(),
            shared: true,
            note_exists: &exists,
            attachment_ok: &attached,
        };
        return edit_shared(store, host, &note, &batch, &context);
    }
    for attempt in 0..RETRIES {
        let note = open(store, id)?;
        let rev = note.rev.clone().unwrap_or_default();
        if let Some(want) = &batch.if_rev
            && *want != rev
        {
            return Err(stale_rev(&note, &canvas_of(&note)?));
        }
        // It may have been shared since the first look.
        if is_shared(&note) {
            return Err(ApiError::conflict());
        }
        let current = canvas_of(&note)?;
        let context = BatchContext {
            now: now_ms(),
            shared: false,
            note_exists: &exists,
            attachment_ok: &attached,
        };
        let (next, made) =
            canvas::apply_batch(&current, &batch, &context).map_err(refused(&note))?;
        let mut fields = Map::new();
        fields.insert("_id".into(), id.into());
        fields.insert("_rev".into(), rev.into());
        fields.insert("body".into(), canvas::to_body(&next).into());
        if let Some(title) = &made.title {
            fields.insert("title".into(), title.clone().into());
        }
        let written = store.put_keeping(
            DocType::Note,
            Value::Object(fields),
            Put::Merge,
            Origin::Api,
            &api::app_only_names(DocType::Note),
        );
        match written {
            Ok(result) => return Ok(done(result.id, result.rev, &made)),
            // Someone wrote the note between the read and the write. An agent
            // that named a revision is told; one that didn't is applied again
            // on the newer canvas.
            Err(StoreError::Conflict) if batch.if_rev.is_none() && attempt + 1 < RETRIES => {}
            Err(StoreError::Conflict) => {
                let latest = open(store, id)?;
                return Err(stale_rev(&latest, &canvas_of(&latest)?));
            }
            Err(err) => return Err(err.into()),
        }
    }
    Err(ApiError::conflict())
}

/// A batch against a shared canvas: applied to its CRDT as one change, the
/// title with it. `ifRev` is checked against the library as the engine has
/// just brought it up to the room, under the room's lock.
fn edit_shared(
    store: &Store,
    host: &Host,
    note: &Note,
    batch: &Batch,
    context: &BatchContext,
) -> ApiResult<Value> {
    let Some(host) = host else {
        return Err(ApiError::bad_request(SHARED_OFFLINE));
    };
    let mut outcome: Option<canvas::Applied> = None;
    let mut refusal: Option<ApiError> = None;
    let done_edit = host
        .edit(&note.id, &mut |live: &Canvas| {
            let now = store.note(&note.id).ok().flatten();
            let rev = now.as_ref().and_then(|n| n.rev.clone()).unwrap_or_default();
            if let Some(want) = &batch.if_rev
                && *want != rev
            {
                let latest = now.as_ref().unwrap_or(note);
                let why = stale_rev(latest, &live.live());
                let text = why.reason.clone();
                refusal = Some(why);
                return Err(text);
            }
            match canvas::apply_batch(live, batch, context) {
                Ok((next, made)) => {
                    let title = made.title.clone();
                    outcome = Some(made);
                    Ok((next, title))
                }
                Err(why) => {
                    let text = why.message();
                    refusal = Some(refused(now.as_ref().unwrap_or(note))(why));
                    Err(text)
                }
            }
        })
        .map_err(ApiError::bad_request);
    if let Some(why) = refusal {
        return Err(why);
    }
    if !done_edit? {
        return Err(ApiError::bad_request(SHARED_OFFLINE));
    }
    let made = outcome.unwrap_or_default();
    let latest = store.note(&note.id)?.ok_or_else(ApiError::not_found)?;
    Ok(done(note.id.clone(), latest.rev.unwrap_or_default(), &made))
}

fn done(id: String, rev: String, made: &canvas::Applied) -> Value {
    json!({
        "ok": true,
        "id": id,
        "rev": rev,
        "created": { "nodes": made.nodes, "edges": made.edges, "strokes": made.strokes },
    })
}

/// A refusal as the API says it: stale items and revisions are 409 with the
/// versions now; a missing item is 404; the rest is 400.
fn refused(note: &Note) -> impl Fn(Refusal) -> ApiError + '_ {
    move |why| match why {
        Refusal::Stale(items) => {
            let versions: Map<String, Value> = items
                .iter()
                .map(|s| (s.id.clone(), s.version.into()))
                .collect();
            ApiError::stale(
                Refusal::Stale(items).message(),
                json!({ "rev": note.rev, "versions": versions }),
            )
        }
        Refusal::Missing(why) => ApiError::not_found_because(why),
        Refusal::Invalid(why) | Refusal::Full(why) => ApiError::bad_request(why),
    }
}

fn stale_rev(note: &Note, current: &Canvas) -> ApiError {
    ApiError::stale(
        "The canvas changed since you read it. Read it again and redo the edit.".to_owned(),
        json!({ "rev": note.rev, "versions": versions_of(current) }),
    )
}

/// The version of every live card, arrow and stroke, by ID.
fn versions_of(canvas: &Canvas) -> Value {
    let mut out = Map::new();
    for node in canvas.nodes.iter().filter(|n| !n.deleted) {
        out.insert(node.id.clone(), node.kanso_version.into());
        for stroke in node.strokes.iter().filter(|s| !s.deleted) {
            out.insert(stroke.id.clone(), stroke.kanso_version.into());
        }
    }
    for edge in canvas.edges.iter().filter(|e| !e.deleted) {
        out.insert(edge.id.clone(), edge.kanso_version.into());
    }
    Value::Object(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(doctype: &str, body: &str) -> Value {
        json!({ "_id": "note:a", "doctype": doctype, "body": body, "title": "t" })
    }

    #[test]
    fn a_canvas_row_is_summarized_and_others_are_left_alone() {
        let mut canvas = Canvas::default();
        canvas.put_node(
            kanso_core::canvas::Node {
                id: "t".into(),
                kind: "text".into(),
                width: 10.0,
                height: 10.0,
                text: Some("remember the milk".into()),
                ..Default::default()
            },
            1,
        );
        let body = canvas::to_body(&canvas);
        let listed = summarize(row("canvas", &body));
        assert_eq!(listed["body"], "- remember the milk");
        assert_eq!(listed["size"], body.len());
        let plain = row("markdown", "```canvas\nnot touched\n```");
        assert_eq!(summarize(plain.clone()), plain);
    }

    #[test]
    fn a_sealed_canvas_has_no_summary_and_no_json() {
        // A locked note is served with an empty body.
        let listed = summarize(row("canvas", ""));
        assert_eq!(listed["body"], "");
        assert_eq!(listed["size"], 0);
        let served = with_canvas_json(row("canvas", ""));
        assert_eq!(served["canvasJson"], "");
        assert_eq!(served["body"], "");
    }

    #[test]
    fn the_json_is_the_fences_inside() {
        let body = canvas::to_body(&Canvas::default());
        let served = with_canvas_json(row("canvas", &body));
        assert_eq!(served["canvasJson"], canvas::fenced(&body).unwrap());
        assert_eq!(served["body"], body.as_str());
    }
}
