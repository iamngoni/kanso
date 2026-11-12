//! What a [`KeyAccess::Capture`](kanso_store::KeyAccess::Capture) key may do
//! (issue #390, step 11).
//!
//! Capture is the smallest power that does the job of a quick-capture app:
//! it reads like a Read key, and it adds. It creates a note (saved at once,
//! not staged), appends a task and ticks a task. It cannot edit, delete,
//! move or untick anything, and it never reaches attachments.
//!
//! Every MCP tool and every REST route has an explicit decision in
//! [`TOOLS`] and [`ROUTES`]; a tool or route that is in neither is
//! [`Decision::Denied`], so a new one is refused until someone decides.
//! The tests fail when a tool or a route is added without a decision.
//! What a Capture key sees is its [`Visibility`](crate::scope::Visibility),
//! as for every key: not locked notes, not notebooks outside its scope.

use axum::http::Method;
use serde_json::{Map, Value};

use crate::error::{ApiError, ApiResult};

/// What Capture does with one tool or route.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Decision {
    /// Reads, through the key's visibility. Nothing is written.
    Read,
    /// Adds something new, or ticks. Checked by [`check_tool`] and by the
    /// tool's own target checks.
    Add,
    /// Refused.
    Denied,
    /// Takes no key at all, so no key's access applies: the pairing routes
    /// (`crate::pair`), which sit outside the key gate.
    Open,
}

use Decision::{Add, Denied, Open, Read};

/// The decision for every MCP tool. A tool missing here is [`Denied`].
pub(crate) const TOOLS: &[(&str, Decision)] = &[
    ("read-note", Read),
    ("read-backlinks", Denied),
    ("search-notes", Read),
    ("list-notes", Read),
    ("list-templates", Read),
    ("create-note", Add),
    ("update-note", Denied),
    ("patch-note", Denied),
    ("list-tags", Denied),
    ("read-tag", Denied),
    ("create-tag", Denied),
    ("update-tag", Denied),
    ("list-notebooks", Read),
    ("read-book", Denied),
    ("create-book", Denied),
    ("update-book", Denied),
    ("create-file", Denied),
    ("read-file", Denied),
    ("kanso-add-pdf-highlight", Denied),
    ("kanso-add-task", Add),
    ("kanso-ask-read-lines", Read),
    ("kanso-ask-search", Read),
    ("kanso-ask-when", Read),
    ("kanso-attach-file", Denied),
    ("kanso-create-canvas", Denied),
    ("kanso-create-daily-note", Denied),
    ("kanso-create-from-template", Denied),
    ("kanso-delete-pdf-highlight", Denied),
    ("kanso-edit-canvas", Denied),
    ("kanso-edit-pdf-comment", Denied),
    ("kanso-edit-pdf-highlight", Denied),
    ("kanso-export-html", Denied),
    ("kanso-get-outline", Read),
    ("kanso-get-properties", Read),
    ("kanso-list-attachments", Denied),
    ("kanso-list-backlinks", Read),
    ("kanso-list-canvases", Read),
    ("kanso-list-mentions", Read),
    ("kanso-list-note-links", Read),
    ("kanso-list-open-loops", Read),
    ("kanso-list-pdf-highlights", Denied),
    ("kanso-list-tags", Read),
    ("kanso-list-tasks", Read),
    ("kanso-list-versions", Read),
    ("kanso-move-note", Denied),
    ("kanso-parse-when", Read),
    ("kanso-read-attachment", Denied),
    ("kanso-read-canvas", Read),
    ("kanso-read-pdf-page", Denied),
    ("kanso-read-version", Read),
    ("kanso-related-notes", Read),
    ("kanso-resolve-wikilink", Read),
    ("kanso-run-query-view", Read),
    ("kanso-search-by-meaning", Read),
    ("kanso-search-pdf", Denied),
    ("kanso-set-task-done", Add),
    ("kanso-sync-now", Denied),
    ("kanso-sync-status", Read),
    // The agent tasks' own tools: no key has them.
    ("kanso-propose-move", Denied),
    ("kanso-propose-merge", Denied),
    ("kanso-list-proposed", Denied),
    ("kanso-task-done", Denied),
];

/// What Capture may do with tool `name`: [`Denied`] for any not listed.
pub(crate) fn tool(name: &str) -> Decision {
    TOOLS
        .iter()
        .find(|(known, _)| *known == name)
        .map_or(Denied, |(_, decision)| *decision)
}

/// One REST method on one route.
#[cfg(test)]
pub(crate) struct Route {
    pub method: &'static str,
    /// The route as the router names it.
    pub pattern: &'static str,
    /// A path the route answers, for the tests.
    pub example: &'static str,
    pub decision: Decision,
}

#[cfg(test)]
const fn route(
    method: &'static str,
    pattern: &'static str,
    example: &'static str,
    decision: Decision,
) -> Route {
    Route {
        method,
        pattern,
        example,
        decision,
    }
}

/// The decision for every REST route, one row per method. `POST /notes` is
/// the only write; it makes a new note and never replaces one.
#[cfg(test)]
pub(crate) const ROUTES: &[Route] = &[
    route("GET", "/", "/", Read),
    route("GET", "/_changes", "/_changes", Read),
    route("GET", "/loops", "/loops", Read),
    route("GET", "/pdf/search", "/pdf/search", Denied),
    route(
        "GET",
        "/notes/{note_id}/pdf/{file_id}/pages/{page}",
        "/notes/note:a/pdf/file:b/pages/1",
        Denied,
    ),
    route(
        "GET",
        "/notes/{note_id}/pdf/{file_id}/highlights",
        "/notes/note:a/pdf/file:b/highlights",
        Denied,
    ),
    route(
        "POST",
        "/notes/{note_id}/pdf/{file_id}/highlights",
        "/notes/note:a/pdf/file:b/highlights",
        Denied,
    ),
    route(
        "PATCH",
        "/notes/{note_id}/pdf/{file_id}/highlights/{highlight_id}",
        "/notes/note:a/pdf/file:b/highlights/h1",
        Denied,
    ),
    route(
        "DELETE",
        "/notes/{note_id}/pdf/{file_id}/highlights/{highlight_id}",
        "/notes/note:a/pdf/file:b/highlights/h1",
        Denied,
    ),
    route("POST", "/pair", "/pair", Open),
    route("GET", "/pair/{request}", "/pair/abc", Open),
    route("GET", "/notes", "/notes", Read),
    route("POST", "/notes", "/notes", Add),
    route("GET", "/{collection}", "/books", Read),
    route("POST", "/{collection}", "/books", Denied),
    route("GET", "/{id}", "/note:a", Read),
    route("POST", "/{id}", "/note:a", Denied),
    route("DELETE", "/{id}", "/note:a", Denied),
    route("GET", "/{kind}/canvas", "/notes/canvas", Read),
    route("PATCH", "/{kind}/canvas", "/notes/canvas", Denied),
    route("GET", "/{kind}/{id}/canvas", "/notes/a/canvas", Read),
    route("PATCH", "/{kind}/{id}/canvas", "/notes/a/canvas", Denied),
    route("GET", "/{kind}/{id}", "/notes/a", Read),
    route("POST", "/{kind}/{id}", "/notes/a", Denied),
    route("DELETE", "/{kind}/{id}", "/notes/a", Denied),
];

/// What Capture may do with a REST request. Anything but a read or
/// `POST /notes` is [`Denied`], whatever route it reaches, so a route added
/// later is refused until it is decided here.
pub(crate) fn rest(method: &Method, path: &str) -> Decision {
    // Exactly the two pairing routes; the router serves them without a key.
    if *method == Method::POST && path == "/pair" {
        return Open;
    }
    if *method == Method::GET
        && path
            .strip_prefix("/pair/")
            .is_some_and(|rest| !rest.is_empty() && !rest.contains('/'))
    {
        return Open;
    }
    if *method == Method::POST && path == "/notes" {
        return Add;
    }
    let pdf = path == "/pdf/search" || path.contains("/pdf/");
    if (*method == Method::GET || *method == Method::HEAD) && !pdf {
        return Read;
    }
    Denied
}

/// The fields a note made by a Capture key may carry. An `_id` or `_rev`
/// would replace a note that is there, and a field of the other kinds
/// (a lock, a share, a publish) is not capture.
const NEW_NOTE_FIELDS: [&str; 5] = ["bookId", "title", "body", "status", "tags"];

/// Refuses a new note by a Capture key that would be more than a new note:
/// one with an `_id` or `_rev`, any field a note is not made with, or a
/// destination that is the trash or the Templates notebook.
pub(crate) fn check_new_note(fields: &Map<String, Value>) -> ApiResult<()> {
    if let Some(field) = fields
        .keys()
        .find(|field| !NEW_NOTE_FIELDS.contains(&field.as_str()))
    {
        return Err(ApiError::forbidden(format!(
            "This key can only add a new note, with a notebook, title, text, status and tags. It can't set `{field}`, or change a note that is there."
        )));
    }
    if matches!(
        fields.get("bookId").and_then(Value::as_str),
        Some("trash" | "template")
    ) {
        return Err(ApiError::forbidden(
            "This key can add notes to your notebooks, not to the trash or the templates.",
        ));
    }
    Ok(())
}

/// The checks of a tool of [`Add`] beyond those every write makes: a task
/// is ticked, never cleared, and a note is only ever new.
pub(crate) fn check_tool(name: &str, args: &Map<String, Value>) -> ApiResult<()> {
    match name {
        "create-note" => check_new_note(args),
        "kanso-set-task-done" if args.get("done").and_then(Value::as_bool) != Some(true) => {
            Err(ApiError::forbidden(
                "This key can tick a task but not clear its checkbox. Pass `done: true`.",
            ))
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tool_names() -> Vec<String> {
        crate::mcp::schema::agent_definitions()
            .as_array()
            .unwrap()
            .iter()
            .map(|tool| tool["name"].as_str().unwrap().to_owned())
            .collect()
    }

    #[test]
    fn every_tool_has_a_decision() {
        for name in tool_names() {
            assert!(
                TOOLS.iter().any(|(known, _)| *known == name),
                "tool {name} has no Capture decision: add it to capture::TOOLS"
            );
        }
        for (name, _) in TOOLS {
            assert!(
                tool_names().iter().any(|known| known == name),
                "capture::TOOLS names {name}, which is not a tool"
            );
        }
    }

    #[test]
    fn tool_decisions_are_listed_once() {
        let mut names: Vec<_> = TOOLS.iter().map(|(name, _)| *name).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), TOOLS.len());
    }

    #[test]
    fn a_tool_that_is_not_listed_is_denied() {
        assert_eq!(tool("kanso-brand-new"), Denied);
        assert_eq!(tool(""), Denied);
    }

    #[test]
    fn capture_adds_only_these_and_reads_no_attachments() {
        let adds: Vec<_> = TOOLS
            .iter()
            .filter(|(_, decision)| *decision == Add)
            .map(|(name, _)| *name)
            .collect();
        assert_eq!(
            adds,
            ["create-note", "kanso-add-task", "kanso-set-task-done"]
        );
        for (name, decision) in TOOLS {
            if *decision == Read {
                assert!(
                    crate::scope::READ_TOOLS.contains(name),
                    "{name} is a Capture read but not a read tool"
                );
                assert!(
                    !crate::agent_tools::ATTACHMENT_TOOLS.contains(name),
                    "{name} reaches attachments"
                );
            }
        }
        // What a Read key reads, Capture reads.
        for name in crate::scope::READ_TOOLS {
            let expected = if crate::agent_tools::ATTACHMENT_TOOLS.contains(&name) {
                Denied
            } else {
                Read
            };
            assert_eq!(tool(name), expected, "{name}");
        }
    }

    /// The route patterns the router is built with, read from the source.
    fn router_patterns() -> Vec<String> {
        let mut patterns = Vec::new();
        for source in [
            include_str!("rest.rs"),
            include_str!("lib.rs"),
            include_str!("pair.rs"),
        ] {
            let production = source.split("#[cfg(test)]").next().unwrap();
            for chunk in production.split(".route(").skip(1) {
                let chunk = chunk.trim_start();
                let pattern = if let Some(rest) = chunk.strip_prefix('"') {
                    rest.split('"').next().unwrap().to_owned()
                } else if chunk.starts_with("&format!(\"/{}\"") {
                    "/{collection}".to_owned()
                } else if chunk.starts_with("auth::MCP_PATH") {
                    "/mcp".to_owned()
                } else {
                    panic!(
                        "a route this test cannot read: {}",
                        &chunk[..40.min(chunk.len())]
                    );
                };
                patterns.push(pattern);
            }
        }
        patterns
    }

    #[test]
    fn every_route_has_a_decision() {
        let patterns = router_patterns();
        assert!(patterns.len() > 10, "{patterns:?}");
        for pattern in &patterns {
            if pattern == "/mcp" {
                // `/mcp` is POST for every key; its tools are decided one
                // by one in TOOLS.
                continue;
            }
            assert!(
                ROUTES.iter().any(|route| route.pattern == pattern),
                "route {pattern} has no Capture decision: add it to capture::ROUTES"
            );
        }
        for route in ROUTES {
            assert!(
                patterns.iter().any(|pattern| pattern == route.pattern),
                "capture::ROUTES names {}, which is not a route",
                route.pattern
            );
        }
    }

    #[test]
    fn the_gate_agrees_with_the_route_table() {
        for route in ROUTES {
            let method: Method = route.method.parse().unwrap();
            assert_eq!(
                rest(&method, route.example),
                route.decision,
                "{} {}",
                route.method,
                route.pattern
            );
        }
    }

    #[test]
    fn only_a_read_or_a_new_note_passes_the_gate() {
        for method in [
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
            Method::TRACE,
        ] {
            for path in ["/notes", "/note:a", "/notes/a", "/books", "/mcp", "/x"] {
                assert_eq!(rest(&method, path), Denied, "{method} {path}");
            }
        }
        assert_eq!(rest(&Method::POST, "/notes/note:a"), Denied);
        assert_eq!(rest(&Method::POST, "/books"), Denied);
        assert_eq!(rest(&Method::POST, "/notes/"), Denied);
    }

    #[test]
    fn only_the_two_pairing_routes_are_open() {
        assert_eq!(rest(&Method::POST, "/pair"), Open);
        assert_eq!(rest(&Method::GET, "/pair/abc"), Open);
        // Anything else under /pair is an ordinary route, so a Capture key
        // is gated on it like on any other.
        assert_eq!(rest(&Method::PATCH, "/pair/x/canvas"), Denied);
        assert_eq!(rest(&Method::GET, "/pair/x/canvas"), Read);
        assert_eq!(rest(&Method::POST, "/pair/x"), Denied);
        assert_eq!(rest(&Method::DELETE, "/pair/x"), Denied);
        assert_eq!(rest(&Method::PUT, "/pair"), Denied);
        assert_eq!(rest(&Method::GET, "/pair/"), Read);
    }

    #[test]
    fn a_new_note_carries_no_id_and_no_lock() {
        let ok = serde_json::json!({ "bookId": "book:a", "title": "t", "body": "b", "tags": [] });
        assert!(check_new_note(ok.as_object().unwrap()).is_ok());
        for field in [
            "_id",
            "_rev",
            "_deleted",
            "kansoLock",
            "kansoShare",
            "pinned",
            "createdAt",
        ] {
            let mut bad = ok.clone();
            bad[field] = serde_json::json!("x");
            assert!(check_new_note(bad.as_object().unwrap()).is_err(), "{field}");
        }
        for book in ["trash", "template"] {
            let mut bad = ok.clone();
            bad["bookId"] = serde_json::json!(book);
            assert!(check_new_note(bad.as_object().unwrap()).is_err(), "{book}");
        }
    }

    #[test]
    fn a_tick_is_never_cleared() {
        let tick = serde_json::json!({ "done": true });
        let clear = serde_json::json!({ "done": false });
        assert!(check_tool("kanso-set-task-done", tick.as_object().unwrap()).is_ok());
        assert!(check_tool("kanso-set-task-done", clear.as_object().unwrap()).is_err());
        assert!(check_tool("kanso-set-task-done", &Map::new()).is_err());
    }
}
