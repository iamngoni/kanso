//! The writing tools of an agent task: `create-note`, `update-note`,
//! `patch-note`, `kanso-propose-move`, `kanso-propose-merge`,
//! `kanso-list-proposed` and `kanso-task-done`.
//!
//! None of them writes. Each stages an op in the task's
//! [`ChangeSet`](crate::changeset::ChangeSet) and answers with what is
//! proposed so far. Every note and notebook they name is checked through the
//! request's [`Visibility`], so one the agent may not see reads as missing
//! and no op can target it.

use serde_json::{Map, Value, json};

use super::schema;
use super::tools::{Occurrence, error_result, find_once};
use crate::changeset::{ChangeSet, ChangeSets, TaskId};
use crate::error::ApiError;
use crate::scope::Visibility;

const TEXT_AND_TITLE_ONLY: &str =
    "An agent can propose changes to a note's text and title only. Other fields aren't changed.";

type Args = Map<String, Value>;

fn str_arg<'a>(args: &'a Args, key: &str) -> &'a str {
    args.get(key).and_then(Value::as_str).unwrap_or_default()
}

fn missing() -> String {
    ApiError::not_found().fetch_error_text()
}

fn ok(text: impl Into<String>) -> Value {
    json!({ "content": [{ "type": "text", "text": text.into() }] })
}

/// Runs one task tool for `task`. Never writes the store.
pub(crate) fn call(
    sets: &ChangeSets,
    task: TaskId,
    vis: &Visibility,
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
    if matches!(name, "create-note" | "update-note" | "patch-note")
        && let Some(sent) = arguments.and_then(Value::as_object)
    {
        let allowed: &[&str] = match name {
            "create-note" => &["bookId", "title", "body"],
            "update-note" => &["_id", "_rev", "title", "body"],
            _ => &["_id", "_rev", "old_string", "new_string"],
        };
        if sent.keys().any(|key| !allowed.contains(&key.as_str())) {
            return error_result(TEXT_AND_TITLE_ONLY);
        }
    }
    let staged = sets.with(task, |set| match name {
        "create-note" => create(set, vis, &args),
        "update-note" => update(set, vis, &args),
        "patch-note" => patch(set, vis, &args),
        "kanso-propose-move" => propose_move(set, vis, &args),
        "kanso-propose-merge" => merge(set, vis, &args),
        "kanso-list-proposed" => {
            Ok(serde_json::to_string_pretty(&set.describe_for_agent()).unwrap_or_default())
        }
        "kanso-task-done" => {
            set.finish(str_arg(&args, "summary"));
            Ok("Thanks. The person will review what you proposed.".to_owned())
        }
        _ => Err(super::READ_ONLY.to_owned()),
    });
    match staged {
        Some(Ok(text)) => ok(text),
        Some(Err(text)) => error_result(text),
        None => error_result("This task has ended."),
    }
}

fn create(set: &mut ChangeSet, vis: &Visibility, args: &Args) -> Result<String, String> {
    let book = str_arg(args, "bookId");
    if !vis.book_visible(book) {
        return Err(missing());
    }
    let temp = set
        .stage_create(
            book,
            str_arg(args, "title"),
            str_arg(args, "body"),
            Vec::new(),
        )
        .map_err(|e| e.0)?;
    Ok(format!("{} The new note's ID is {temp}.", set.ack()))
}

/// The pending text of a temporary note of `set`.
fn temp_text(set: &ChangeSet, id: &str) -> Option<(String, String)> {
    set.ops.iter().find_map(|op| match op {
        crate::changeset::ProposedOp::Create {
            temp, title, body, ..
        } if temp == id => Some((title.clone(), body.clone())),
        _ => None,
    })
}

fn update(set: &mut ChangeSet, vis: &Visibility, args: &Args) -> Result<String, String> {
    if !args.contains_key("title") && !args.contains_key("body") {
        return Err("There is nothing to change: give a new `title` or `body`.".to_owned());
    }
    let id = str_arg(args, "_id");
    let title = args.get("title").and_then(Value::as_str);
    let body = args.get("body").and_then(Value::as_str);
    if ChangeSet::is_temp(id) {
        set.stage_edit(id, "", ("", ""), title, body)
            .map_err(|e| e.0)?;
        return Ok(set.ack());
    }
    let note = vis.note(id).ok_or_else(missing)?;
    if note.rev.as_deref() != Some(str_arg(args, "_rev")) {
        return Err(missing());
    }
    set.stage_edit(
        id,
        note.rev.as_deref().unwrap_or_default(),
        (&note.title, &note.body),
        title,
        body,
    )
    .map_err(|e| e.0)?;
    Ok(set.ack())
}

fn patch(set: &mut ChangeSet, vis: &Visibility, args: &Args) -> Result<String, String> {
    let id = str_arg(args, "_id");
    let old = str_arg(args, "old_string");
    let new = str_arg(args, "new_string");
    let replace = |body: &str| {
        match find_once(body, old) {
        Occurrence::None => {
            Err("Failed to patch: `old_string` was not found in the note body.".to_owned())
        }
        Occurrence::Many => Err("Failed to patch: `old_string` matches multiple locations in the note body. Provide more surrounding context to make it unique.".to_owned()),
        Occurrence::Once(at) => Ok(format!("{}{new}{}", &body[..at], &body[at + old.len()..])),
    }
    };
    if ChangeSet::is_temp(id) {
        let (_, body) = temp_text(set, id).ok_or_else(missing)?;
        let patched = replace(&body)?;
        set.stage_edit(id, "", ("", ""), None, Some(&patched))
            .map_err(|e| e.0)?;
        return Ok(set.ack());
    }
    let note = vis.note(id).ok_or_else(missing)?;
    let rev = note.rev.clone().unwrap_or_default();
    if rev != str_arg(args, "_rev") {
        return Err(missing());
    }
    // Build on the text already proposed for this note, if any.
    let base = set
        .pending_text(id, &rev)
        .map_or_else(|| note.body.clone(), |(_, body)| body);
    let patched = replace(&base)?;
    set.stage_edit(id, &rev, (&note.title, &note.body), None, Some(&patched))
        .map_err(|e| e.0)?;
    Ok(set.ack())
}

fn propose_move(set: &mut ChangeSet, vis: &Visibility, args: &Args) -> Result<String, String> {
    let id = str_arg(args, "_id");
    let to = str_arg(args, "bookId");
    if !vis.book_visible(to) {
        return Err(missing());
    }
    if ChangeSet::is_temp(id) {
        set.stage_move(id, "", "", to).map_err(|e| e.0)?;
        return Ok(set.ack());
    }
    let note = vis.note(id).ok_or_else(missing)?;
    if note.rev.as_deref() != Some(str_arg(args, "_rev")) {
        return Err(missing());
    }
    set.stage_move(
        id,
        note.rev.as_deref().unwrap_or_default(),
        &note.book_id,
        to,
    )
    .map_err(|e| e.0)?;
    Ok(set.ack())
}

/// `{_id, _rev}` of a note the agent may see at that revision.
fn visible_ref(vis: &Visibility, value: &Value) -> Result<(String, String), String> {
    let (Some(id), Some(rev)) = (
        value.get("_id").and_then(Value::as_str),
        value.get("_rev").and_then(Value::as_str),
    ) else {
        return Err("Each note needs an `_id` and a `_rev`.".to_owned());
    };
    let note = vis.note(id).ok_or_else(missing)?;
    if note.rev.as_deref() != Some(rev) {
        return Err(missing());
    }
    Ok((id.to_owned(), rev.to_owned()))
}

fn merge(set: &mut ChangeSet, vis: &Visibility, args: &Args) -> Result<String, String> {
    let sources = args
        .get("sources")
        .and_then(Value::as_array)
        .ok_or("`sources` must be a list of notes.")?
        .iter()
        .map(|source| visible_ref(vis, source))
        .collect::<Result<Vec<_>, _>>()?;
    let into = match args.get("into") {
        Some(target) => Some(visible_ref(vis, target)?),
        None => None,
    };
    let book = args.get("bookId").and_then(Value::as_str);
    if let Some(book) = book
        && !vis.book_visible(book)
    {
        return Err(missing());
    }
    let temp = set
        .stage_merge(
            sources,
            into,
            book.map(str::to_owned),
            str_arg(args, "title"),
            str_arg(args, "body"),
        )
        .map_err(|e| e.0)?;
    Ok(match temp {
        Some(temp) => format!("{} The merged note's ID is {temp}.", set.ack()),
        None => set.ack(),
    })
}
