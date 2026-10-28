//! Example tests for the native `/mcp` endpoint.

mod common;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use common::*;
use kanso_server::basic_header;
use serde_json::{Value, json};

fn mcp_request(body: &str, extra: &[(&str, &str)]) -> Request<Body> {
    let mut builder = Request::builder()
        .method(Method::POST)
        .uri("/mcp")
        .header(header::AUTHORIZATION, basic_header(USER, PASS))
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::ACCEPT, "application/json, text/event-stream");
    for (name, value) in extra {
        builder = builder.header(*name, *value);
    }
    builder.body(Body::from(body.to_owned())).unwrap()
}

#[tokio::test]
async fn initialize_negotiates_the_protocol_version() {
    let app = app();
    for (requested, expected) in [
        ("2025-06-18", "2025-06-18"),
        ("2024-11-05", "2024-11-05"),
        ("1999-01-01", "2025-11-25"),
    ] {
        let answer = rpc(
            &app.router,
            "initialize",
            json!({ "protocolVersion": requested, "capabilities": {}, "clientInfo": { "name": "t", "version": "0" } }),
        )
        .await;
        assert_eq!(answer["jsonrpc"], "2.0");
        assert_eq!(answer["id"], 1);
        assert_eq!(answer["result"]["protocolVersion"], expected);
        assert_eq!(answer["result"]["serverInfo"]["name"], "Kanso");
        assert!(answer["result"]["capabilities"]["tools"].is_object());
        assert!(answer["result"]["capabilities"]["resources"].is_object());
    }
}

#[tokio::test]
async fn lists_inkdrop_tools_before_kanso_tools_and_the_note_template() {
    let app = app();
    let answer = rpc(&app.router, "tools/list", json!({})).await;
    let tools = answer["result"]["tools"].as_array().unwrap();
    // The 17, then Kanso's own.
    assert_eq!(tools.len(), 38);
    assert!(
        tools[18..]
            .iter()
            .all(|t| t["name"].as_str().unwrap().starts_with("kanso-"))
    );
    for tool in tools {
        assert!(tool["description"].as_str().is_some_and(|d| !d.is_empty()));
        assert_eq!(tool["inputSchema"]["type"], "object");
        for (_, schema) in tool["inputSchema"]["properties"]
            .as_object()
            .into_iter()
            .flatten()
        {
            assert!(
                schema["description"]
                    .as_str()
                    .is_some_and(|d| !d.is_empty()),
                "{}",
                tool["name"]
            );
        }
    }
    let answer = rpc(&app.router, "resources/templates/list", json!({})).await;
    assert_eq!(
        answer["result"],
        json!({ "resourceTemplates": [{
            "name": "note", "uriTemplate": "inkdrop://note/{noteId}",
            "description": "A note data", "mimeType": "application/json"
        }] })
    );
    let answer = rpc(&app.router, "resources/list", json!({})).await;
    assert_eq!(answer["result"], json!({ "resources": [] }));
    assert_eq!(
        rpc(&app.router, "ping", json!({})).await["result"],
        json!({})
    );
}

#[tokio::test]
async fn kanso_tools_read_properties_links_query_views_and_versions() {
    let app = app();
    let (book_id, _) = write(&app.router, "/books", json!({ "name": "Library" })).await;
    let (target, _) = write(&app.router, "/notes", json!({
        "bookId": book_id, "title": "Target", "body": "---\npriority: 3\nlabels: [one, two]\n---\n# Heading\n"
    })).await;
    let (source, _) = write(&app.router, "/notes", json!({
        "bookId": book_id, "title": "Source", "body": format!("[[Target]]\ninkdrop://note/{}\n", target.trim_start_matches("note:"))
    })).await;

    let props = tool_json(
        &tool(
            &app.router,
            "kanso-get-properties",
            json!({ "noteId": target }),
        )
        .await,
    );
    assert_eq!(
        props["properties"],
        json!([
            { "key": "priority", "value": 3.0 },
            { "key": "labels", "value": ["one", "two"] }
        ])
    );
    let outgoing = tool_json(
        &tool(
            &app.router,
            "kanso-list-note-links",
            json!({ "noteId": source }),
        )
        .await,
    );
    assert_eq!(outgoing.as_array().unwrap().len(), 2);
    assert_eq!(outgoing[0]["target"]["noteId"], target);
    let backlinks = tool_json(
        &tool(
            &app.router,
            "kanso-list-backlinks",
            json!({ "noteId": target }),
        )
        .await,
    );
    assert_eq!(backlinks[0]["noteId"], source);

    let (view, _) = write(&app.router, "/notes", json!({
        "bookId": book_id, "title": "View", "body": "```query\nfrom: Target\ncolumns: priority\n```\n"
    })).await;
    let rows = tool_json(
        &tool(
            &app.router,
            "kanso-run-query-view",
            json!({ "noteId": view }),
        )
        .await,
    );
    assert_eq!(rows["columns"], json!(["Title", "priority"]));
    assert_eq!(rows["rows"][0]["noteId"], target);
    assert_eq!(rows["rows"][0]["cells"], json!([3.0]));

    let (history, rev) = write(
        &app.router,
        "/notes",
        json!({ "bookId": book_id, "title": "History", "body": "first" }),
    )
    .await;
    let updated = tool(
        &app.router,
        "update-note",
        json!({ "_id": history, "_rev": rev, "body": "second" }),
    )
    .await;
    assert_ne!(updated["isError"], true);
    let versions = tool_json(
        &tool(
            &app.router,
            "kanso-list-versions",
            json!({ "noteId": history }),
        )
        .await,
    );
    assert!(
        versions
            .as_array()
            .unwrap()
            .iter()
            .any(|version| version["rev"] == rev)
    );
    let old = tool_json(
        &tool(
            &app.router,
            "kanso-read-version",
            json!({ "noteId": history, "rev": rev }),
        )
        .await,
    );
    assert_eq!(old["body"], "first");
}

#[tokio::test]
async fn inkdrop_template_tools_map_to_kansos_templates_notebook() {
    let app = app();
    let created = tool_json(
        &tool(
            &app.router,
            "create-note",
            json!({ "bookId": "template", "title": "Release", "body": "# Release" }),
        )
        .await,
    );
    let listed = tool_json(
        &tool(
            &app.router,
            "list-templates",
            json!({ "keyword": "Release" }),
        )
        .await,
    );
    assert_eq!(listed[0]["_id"], created["id"]);
    assert_eq!(listed[0]["title"], "Release");
    assert_eq!(
        get(
            &app.router,
            &format!("/{}", created["id"].as_str().unwrap()),
        )
        .await
        .1["bookId"],
        "book:templates"
    );
}

#[tokio::test]
async fn kanso_explicit_writes_move_notes_toggle_tasks_and_attach_files() {
    let app = app();
    let (book_a, _) = write(&app.router, "/books", json!({ "name": "A" })).await;
    let (book_b, _) = write(&app.router, "/books", json!({ "name": "B" })).await;
    let (note, _) = write(
        &app.router,
        "/notes",
        json!({ "bookId": book_a, "title": "Work", "body": "- [ ] Ship it\n" }),
    )
    .await;
    let task = tool(
        &app.router,
        "kanso-set-task-done",
        json!({ "noteId": note, "line": 0, "text": "Ship it", "done": true }),
    )
    .await;
    assert_ne!(task["isError"], true);
    let moved = tool(
        &app.router,
        "kanso-move-note",
        json!({ "noteId": note, "bookId": book_b }),
    )
    .await;
    assert_ne!(moved["isError"], true);
    assert_eq!(
        get(&app.router, &format!("/{note}")).await.1["bookId"],
        book_b
    );

    let attached = tool_json(&tool(&app.router, "kanso-attach-file", json!({
        "noteId": note, "name": "pixel.png", "contentType": "image/png", "data": STANDARD.encode(PNG)
    })).await);
    assert_eq!(attached["contentLength"], PNG.len());
    let listed = tool_json(
        &tool(
            &app.router,
            "kanso-list-attachments",
            json!({ "noteId": note }),
        )
        .await,
    );
    assert_eq!(listed[0]["fileId"], attached["fileId"]);
    let file = tool_json(
        &tool(
            &app.router,
            "kanso-read-attachment",
            json!({ "noteId": note, "fileId": attached["fileId"] }),
        )
        .await,
    );
    assert_eq!(file["_attachments"]["index"]["data"], STANDARD.encode(PNG));
}

#[tokio::test]
async fn transport_edge_cases() {
    let app = app();
    // Notifications and client responses get 202 with no body.
    for body in [
        r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
        r#"{"jsonrpc":"2.0","id":7,"result":{}}"#,
        r#"[{"jsonrpc":"2.0","method":"notifications/cancelled","params":{"requestId":1}}]"#,
    ] {
        let response = send(&app.router, mcp_request(body, &[])).await;
        assert_eq!(response.status, StatusCode::ACCEPTED, "{body}");
        assert!(response.bytes.is_empty());
    }
    // Unknown methods, bad JSON, batches, invalid requests.
    let response = send(
        &app.router,
        mcp_request(r#"{"jsonrpc":"2.0","id":"a","method":"prompts/list"}"#, &[]),
    )
    .await;
    assert_eq!(
        response.json(),
        json!({ "jsonrpc": "2.0", "id": "a", "error": { "code": -32601, "message": "Method not found" } })
    );
    let response = send(&app.router, mcp_request("{nope", &[])).await;
    assert_eq!(response.status, StatusCode::BAD_REQUEST);
    assert_eq!(response.json()["error"]["code"], -32700);
    let response = send(
        &app.router,
        mcp_request(
            r#"[{"jsonrpc":"2.0","id":1,"method":"ping"},{"jsonrpc":"2.0","method":"notifications/initialized"},{"jsonrpc":"2.0","id":2,"method":"ping"}]"#,
            &[],
        ),
    )
    .await;
    assert_eq!(
        response.json(),
        json!([{ "jsonrpc": "2.0", "id": 1, "result": {} }, { "jsonrpc": "2.0", "id": 2, "result": {} }])
    );
    for body in ["[]", "3", r#"{"id":1,"method":"ping"}"#] {
        let response = send(&app.router, mcp_request(body, &[])).await;
        assert_eq!(response.json()["error"]["code"], -32600, "{body}");
    }
    let response = send(
        &app.router,
        mcp_request(
            r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{}}"#,
            &[],
        ),
    )
    .await;
    assert_eq!(response.json()["error"]["code"], -32602);

    // Headers: protocol version and origin.
    let ping = r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#;
    let response = send(
        &app.router,
        mcp_request(ping, &[("mcp-protocol-version", "2025-06-18")]),
    )
    .await;
    assert_eq!(response.status, StatusCode::OK);
    let response = send(
        &app.router,
        mcp_request(ping, &[("mcp-protocol-version", "1.0")]),
    )
    .await;
    assert_eq!(response.status, StatusCode::BAD_REQUEST);
    let response = send(
        &app.router,
        mcp_request(ping, &[("origin", "http://localhost:6274")]),
    )
    .await;
    assert_eq!(response.status, StatusCode::OK);
    let response = send(
        &app.router,
        mcp_request(ping, &[("origin", "https://evil.example")]),
    )
    .await;
    assert_eq!(response.status, StatusCode::FORBIDDEN);

    // No event stream, no sessions.
    for method in [Method::GET, Method::DELETE] {
        let response = send(&app.router, request(method, "/mcp", None)).await;
        assert_eq!(response.status, StatusCode::METHOD_NOT_ALLOWED);
        assert_eq!(response.headers[header::ALLOW], "POST");
    }
}

#[tokio::test]
async fn tool_errors_read_like_the_node_server() {
    let app = app();
    let result = tool(&app.router, "nope", json!({})).await;
    assert_eq!(
        result,
        json!({ "content": [{ "type": "text", "text": "MCP error -32602: Tool nope not found" }], "isError": true })
    );
    let result = tool(
        &app.router,
        "create-note",
        json!({ "bookId": "x", "title": "a", "body": "b" }),
    )
    .await;
    assert_eq!(result["isError"], true);
    assert_eq!(
        text(&result, 0),
        "MCP error -32602: Input validation error: Invalid arguments for tool create-note: \
         Too small: expected string to have >=5 characters at bookId\n\
         Invalid string: must match pattern /^(book:|trash$|template$)/ at bookId"
    );
    let result = tool(
        &app.router,
        "read-note",
        json!({ "noteId": "note:missing" }),
    )
    .await;
    assert_eq!(
        result,
        json!({ "content": [{ "type": "text", "text": "Fetch error [404] Not Found" }], "isError": true })
    );
    let result = tool(
        &app.router,
        "create-note",
        json!({ "bookId": "book:missing", "title": "a", "body": "b" }),
    )
    .await;
    assert_eq!(text(&result, 0), "Fetch error [400] Bad Request");
}

#[tokio::test]
async fn notes_round_trip_through_the_tools() {
    let app = app();
    let book_id =
        tool_json(&tool(&app.router, "create-book", json!({ "name": "Work" })).await)["id"]
            .as_str()
            .unwrap()
            .to_owned();
    let created = tool_json(&tool(&app.router, "create-tag", json!({ "name": "urgent" })).await);
    let tag_id = created["id"].as_str().unwrap().to_owned();
    assert_eq!(created["ok"], true);
    let (_, tag_doc) = get(&app.router, &format!("/{tag_id}")).await;
    assert_eq!(tag_doc["color"], "default");

    let long_body = format!("{}😀tail", "x".repeat(199));
    let note = tool_json(
        &tool(
            &app.router,
            "create-note",
            json!({ "bookId": book_id, "title": "Plan", "body": long_body, "tags": [tag_id], "status": "active" }),
        )
        .await,
    );
    let note_id = note["id"].as_str().unwrap().to_owned();

    // read-note accepts the ID with or without its prefix.
    let read = tool(&app.router, "read-note", json!({ "noteId": note_id })).await;
    let (_, http_doc) = get(&app.router, &format!("/{note_id}")).await;
    assert_eq!(
        text(&read, 0),
        serde_json::to_string_pretty(&http_doc).unwrap()
    );
    assert_eq!(read["content"].as_array().unwrap().len(), 1);
    let bare = note_id.strip_prefix("note:").unwrap();
    let read_bare = tool(&app.router, "read-note", json!({ "noteId": bare })).await;
    assert_eq!(read, read_bare);

    // Summaries cut bodies at 200 UTF-16 units; the emoji straddles the cut.
    let found = tool_json(&tool(&app.router, "search-notes", json!({ "keyword": "tail" })).await);
    assert_eq!(found[0]["_id"], note_id);
    assert_eq!(found[0]["body"], "x".repeat(199));
    let listed = tool_json(
        &tool(
            &app.router,
            "list-notes",
            json!({ "bookId": book_id, "tagIds": [tag_id] }),
        )
        .await,
    );
    assert_eq!(listed.as_array().unwrap().len(), 1);
    let none = tool_json(
        &tool(
            &app.router,
            "list-notes",
            json!({ "keyword": "absent-word" }),
        )
        .await,
    );
    assert_eq!(none, json!([]));

    // update-note merges given fields into the current note.
    let rev = http_doc["_rev"].as_str().unwrap();
    let updated = tool_json(
        &tool(
            &app.router,
            "update-note",
            json!({ "_id": note_id, "_rev": rev, "title": "Plan v2" }),
        )
        .await,
    );
    let (_, doc) = get(&app.router, &format!("/{note_id}")).await;
    assert_eq!(doc["title"], "Plan v2");
    assert_eq!(doc["status"], "active");
    assert_eq!(doc["_rev"], updated["rev"]);
    // A stale rev fails like the Node server: the GET with ?rev= is a 404.
    let stale = tool(
        &app.router,
        "update-note",
        json!({ "_id": note_id, "_rev": rev, "title": "x" }),
    )
    .await;
    assert_eq!(text(&stale, 0), "Fetch error [404] Not Found");
    assert_eq!(stale["isError"], true);
}

#[tokio::test]
async fn patch_note_needs_exactly_one_occurrence() {
    let app = app();
    let (note_id, rev) = write(
        &app.router,
        "/notes",
        json!({ "bookId": "trash", "title": "t", "body": "one two two three" }),
    )
    .await;
    let patch = |old: &str, rev: &str| json!({ "_id": note_id, "_rev": rev, "old_string": old, "new_string": "2" });
    let missing = tool(&app.router, "patch-note", patch("four", &rev)).await;
    assert_eq!(
        missing,
        json!({ "content": [{ "type": "text", "text": "Failed to patch: `old_string` was not found in the note body." }], "isError": true })
    );
    let many = tool(&app.router, "patch-note", patch("two", &rev)).await;
    assert_eq!(
        text(&many, 0),
        "Failed to patch: `old_string` matches multiple locations in the note body. Provide more surrounding context to make it unique."
    );
    let ok = tool_json(&tool(&app.router, "patch-note", patch("one", &rev)).await);
    let (_, doc) = get(&app.router, &format!("/{note_id}")).await;
    assert_eq!(doc["body"], "2 two two three");
    assert_eq!(doc["_rev"], ok["rev"]);
}

#[tokio::test]
async fn backlinks_and_templates() {
    let app = app();
    let (target, _) = write(
        &app.router,
        "/notes",
        json!({ "bookId": "trash", "title": "Target", "body": "" }),
    )
    .await;
    let bare = target.strip_prefix("note:").unwrap().to_owned();
    let result = tool(&app.router, "read-backlinks", json!({ "noteId": target })).await;
    assert_eq!(
        text(&result, 0),
        format!("No backlinks found for inkdrop://note/{bare}.")
    );

    let (book_id, _) = write(&app.router, "/books", json!({ "name": "B" })).await;
    let (linker, _) = write(
        &app.router,
        "/notes",
        json!({ "bookId": book_id, "title": "", "body": format!("see [t](inkdrop://note/{bare})") }),
    )
    .await;
    let result = tool(&app.router, "read-backlinks", json!({ "noteId": bare })).await;
    assert_eq!(
        result,
        json!({ "content": [{
            "type": "resource_link",
            "uri": format!("inkdrop://note/{}", linker.strip_prefix("note:").unwrap()),
            "name": "Untitled note",
            "mimeType": "application/json",
        }] })
    );

    // A note created from a template links to it, and embeds it on request.
    let (template, _) = write(
        &app.router,
        "/notes",
        json!({ "bookId": book_id, "title": "Weekly", "body": "Fill in the goals." }),
    )
    .await;
    let (from_template, _) = write(
        &app.router,
        "/notes",
        json!({ "bookId": book_id, "title": "Week 1", "body": "", "sourceTemplateId": template }),
    )
    .await;
    let result = tool(&app.router, "read-note", json!({ "noteId": from_template })).await;
    let blocks = result["content"].as_array().unwrap();
    assert_eq!(blocks.len(), 2);
    assert_eq!(blocks[1]["type"], "resource_link");
    assert_eq!(
        blocks[1]["uri"],
        format!("inkdrop://note/{}", template.strip_prefix("note:").unwrap())
    );
    assert_eq!(blocks[1]["name"], "Source template");
    let result = tool(
        &app.router,
        "read-note",
        json!({ "noteId": from_template, "includeTemplate": true }),
    )
    .await;
    assert_eq!(
        text(&result, 2),
        "# Source template: Weekly\n\nFill in the goals."
    );

    // The note resource.
    let answer = rpc(
        &app.router,
        "resources/read",
        json!({ "uri": format!("inkdrop://note/{bare}") }),
    )
    .await;
    let contents = &answer["result"]["contents"][0];
    assert_eq!(contents["uri"], format!("inkdrop://note/{bare}"));
    assert_eq!(contents["mimeType"], "application/json");
    let doc: Value = serde_json::from_str(contents["text"].as_str().unwrap()).unwrap();
    assert_eq!(doc["_id"], target);
    let answer = rpc(
        &app.router,
        "resources/read",
        json!({ "uri": "inkdrop://book/abc" }),
    )
    .await;
    assert_eq!(answer["error"]["code"], -32602);
    assert_eq!(
        answer["error"]["message"],
        "MCP error -32602: Resource inkdrop://book/abc not found"
    );
    let answer = rpc(
        &app.router,
        "resources/read",
        json!({ "uri": "inkdrop://note/missing" }),
    )
    .await;
    assert_eq!(
        answer["error"],
        json!({ "code": -32603, "message": "Fetch error [404] Not Found" })
    );
}

#[tokio::test]
async fn tags_and_notebooks_through_the_tools() {
    let app = app();
    let parent =
        tool_json(&tool(&app.router, "create-book", json!({ "name": "Parent" })).await)["id"]
            .as_str()
            .unwrap()
            .to_owned();
    let child = tool_json(
        &tool(
            &app.router,
            "create-book",
            json!({ "name": "Child", "parentBookId": parent }),
        )
        .await,
    )["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let books = tool_json(&tool(&app.router, "list-notebooks", json!({})).await);
    let (_, http_books) = get(&app.router, "/books").await;
    assert_eq!(books, http_books);
    let book = tool_json(
        &tool(
            &app.router,
            "read-book",
            json!({ "bookId": child.strip_prefix("book:").unwrap() }),
        )
        .await,
    );
    assert_eq!(book["parentBookId"], parent);
    tool_json(
        &tool(
            &app.router,
            "update-book",
            json!({ "_id": child, "parentBookId": null, "name": "Top" }),
        )
        .await,
    );
    let (_, doc) = get(&app.router, &format!("/{child}")).await;
    assert_eq!(
        (doc["name"].as_str(), &doc["parentBookId"]),
        (Some("Top"), &Value::Null)
    );

    let tag_id = tool_json(
        &tool(
            &app.router,
            "create-tag",
            json!({ "name": "x", "color": "teal" }),
        )
        .await,
    )["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let (_, tag_doc) = get(&app.router, &format!("/{tag_id}")).await;
    let stale_rev = tag_doc["_rev"].as_str().unwrap().to_owned();
    tool_json(
        &tool(
            &app.router,
            "update-tag",
            json!({ "_id": tag_id, "color": "pink" }),
        )
        .await,
    );
    let tag = tool_json(&tool(&app.router, "read-tag", json!({ "tagId": tag_id })).await);
    assert_eq!(
        (tag["name"].as_str(), tag["color"].as_str()),
        (Some("x"), Some("pink"))
    );
    let conflict = tool(
        &app.router,
        "update-tag",
        json!({ "_id": tag_id, "_rev": stale_rev, "name": "y" }),
    )
    .await;
    assert_eq!(text(&conflict, 0), "Fetch error [409] Conflict");
    let tags = tool_json(&tool(&app.router, "list-tags", json!({})).await);
    assert_eq!(tags.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn files_through_the_tools() {
    let app = app();
    let dir = tempfile::tempdir().unwrap();
    let png_path = dir.path().join("pixel.png");
    std::fs::write(&png_path, PNG).unwrap();

    let neither = tool(&app.router, "create-file", json!({})).await;
    assert_eq!(
        text(&neither, 0),
        "Failed to create the attachment: provide exactly one of `filePath` or `data`."
    );
    let missing = tool(
        &app.router,
        "create-file",
        json!({ "filePath": "/no/such/file.png" }),
    )
    .await;
    assert!(text(&missing, 0).starts_with("Failed to read the file at /no/such/file.png: "));
    let untyped = tool(
        &app.router,
        "create-file",
        json!({ "data": STANDARD.encode(PNG) }),
    )
    .await;
    assert!(text(&untyped, 0).starts_with(
        "Failed to create the attachment: could not infer the content type. Pass `contentType` explicitly."
    ));

    let created = tool(&app.router, "create-file", json!({ "filePath": png_path })).await;
    let res = tool_json(&created);
    let file_id = res["id"].as_str().unwrap().to_owned();
    assert_eq!(
        text(&created, 1),
        format!(
            "Markdown to embed this attachment in a note body:\n\n![pixel.png](inkdrop://{file_id})"
        )
    );
    let (_, doc) = get(&app.router, &format!("/{file_id}?attachments=true")).await;
    assert_eq!(doc["name"], "pixel.png");
    assert_eq!(doc["contentType"], "image/png");
    assert_eq!(doc["_attachments"]["index"]["data"], STANDARD.encode(PNG));

    let read = tool(&app.router, "read-file", json!({ "fileId": file_id })).await;
    let metadata: Value = serde_json::from_str(text(&read, 0)).unwrap();
    assert!(metadata.get("_attachments").is_none());
    assert_eq!(metadata["_id"], file_id);
    assert_eq!(
        read["content"][1],
        json!({ "type": "image", "data": STANDARD.encode(PNG), "mimeType": "image/png" })
    );
    let meta_only = tool(
        &app.router,
        "read-file",
        json!({ "fileId": file_id, "includeImage": false }),
    )
    .await;
    assert_eq!(meta_only["content"].as_array().unwrap().len(), 1);
    let out = dir.path().join("out.png");
    let written = tool(
        &app.router,
        "read-file",
        json!({ "fileId": file_id, "outputPath": out }),
    )
    .await;
    assert_eq!(
        text(&written, 1),
        format!("Attachment written to {}", out.display())
    );
    assert_eq!(std::fs::read(&out).unwrap(), PNG);

    // Inline data; SVG cannot be shown inline.
    let svg = "<svg xmlns=\"http://www.w3.org/2000/svg\"/>";
    let created = tool(
        &app.router,
        "create-file",
        json!({ "data": STANDARD.encode(svg), "contentType": "image/svg+xml" }),
    )
    .await;
    let svg_id = tool_json(&created)["id"].as_str().unwrap().to_owned();
    assert!(text(&created, 1).contains("![image.svg+xml](inkdrop://"));
    let read = tool(&app.router, "read-file", json!({ "fileId": svg_id })).await;
    assert_eq!(
        text(&read, 1),
        "The attachment is a image/svg+xml file, which cannot be displayed inline. Pass `outputPath` to write it to disk."
    );
}

#[tokio::test]
async fn local_files_are_off_for_non_loopback_binds() {
    let dir = tempfile::tempdir().unwrap();
    let store = std::sync::Arc::new(kanso_store::Store::open(dir.path().join("k.db")).unwrap());
    let config = kanso_server::ServerConfig {
        bind: "0.0.0.0:0".parse().unwrap(),
        ..config()
    };
    let router = kanso_server::router(store, &config);
    let png_path = dir.path().join("pixel.png");
    std::fs::write(&png_path, PNG).unwrap();
    let result = tool(&router, "create-file", json!({ "filePath": png_path })).await;
    assert_eq!(result["isError"], true);
    assert!(
        text(&result, 0).contains("only available when the server is bound to a loopback address")
    );
}

#[tokio::test]
async fn notes_export_as_standalone_pages() {
    let app = app();
    let book_id =
        tool_json(&tool(&app.router, "create-book", json!({ "name": "Work" })).await)["id"]
            .as_str()
            .unwrap()
            .to_owned();
    let body = "Area $\\pi r^2$.\n\n```mermaid\ngraph TD\n  A --> B\n```\n\n<script>x()</script>\n";
    let note = tool_json(
        &tool(
            &app.router,
            "create-note",
            json!({ "bookId": book_id, "title": "Circles", "body": body }),
        )
        .await,
    );
    let note_id = note["id"].as_str().unwrap().to_owned();
    for theme in ["light", "dark"] {
        let page = tool(
            &app.router,
            "kanso-export-html",
            json!({ "noteId": note_id, "theme": theme }),
        )
        .await;
        let html = text(&page, 0);
        assert!(html.starts_with("<!doctype html>"), "{html}");
        assert!(html.contains("<title>Circles</title>"));
        assert!(html.contains("kanso-math") && html.contains("kanso-diagram"));
        assert!(!html.contains("<script"));
    }
    let missing = tool(
        &app.router,
        "kanso-export-html",
        json!({ "noteId": "note:nope" }),
    )
    .await;
    assert_eq!(missing["isError"], true);
    let bad = tool(
        &app.router,
        "kanso-export-html",
        json!({ "noteId": note_id, "theme": "sepia" }),
    )
    .await;
    assert_eq!(bad["isError"], true);
}

#[tokio::test]
async fn outlines_and_wiki_links_through_kanso_tools() {
    let app = app();
    let (book_id, _) = write(&app.router, "/books", json!({ "name": "B" })).await;
    let (plan, _) = write(
        &app.router,
        "/notes",
        json!({ "bookId": book_id, "title": "Project Plan", "body": "# Goals\ntext\n```\n# not\n```\n## Budget Lines\n" }),
    )
    .await;
    let outline =
        tool_json(&tool(&app.router, "kanso-get-outline", json!({ "noteId": plan })).await);
    assert_eq!(
        outline,
        json!([
            { "level": 1, "text": "Goals", "line": 0 },
            { "level": 2, "text": "Budget Lines", "line": 5 },
        ])
    );
    let missing = tool(
        &app.router,
        "kanso-get-outline",
        json!({ "noteId": "note:missing" }),
    )
    .await;
    assert_eq!(missing["isError"], true);

    let bare = plan.strip_prefix("note:").unwrap();
    let found = tool_json(
        &tool(
            &app.router,
            "kanso-resolve-wikilink",
            json!({ "link": "[[project  plan#budget lines|money]]" }),
        )
        .await,
    );
    assert_eq!(
        found,
        json!({
            "noteId": plan,
            "title": "Project Plan",
            "uri": format!("inkdrop://note/{bare}"),
            "alias": "money",
            "heading": { "text": "Budget Lines", "level": 2, "line": 5 },
        })
    );
    let unknown = tool(
        &app.router,
        "kanso-resolve-wikilink",
        json!({ "link": "Nothing" }),
    )
    .await;
    assert_eq!(unknown["isError"], true);
    assert_eq!(text(&unknown, 0), "No note is titled \"Nothing\".");
    let bad = tool(
        &app.router,
        "kanso-resolve-wikilink",
        json!({ "link": "[[]]" }),
    )
    .await;
    assert_eq!(bad["isError"], true);
}

#[tokio::test]
async fn tasks_through_kanso_list_tasks() {
    let app = app();
    let (book_id, _) = write(&app.router, "/books", json!({ "name": "B" })).await;
    let (other, _) = write(&app.router, "/books", json!({ "name": "C" })).await;
    let (plan, _) = write(
        &app.router,
        "/notes",
        json!({ "bookId": book_id, "title": "Plan", "body": "- [ ] Pay rent @2026-11-01\n- [x] Book flights @2026-10-02\n- [ ] Someday\n```\n- [ ] not a task\n```\n" }),
    )
    .await;
    write(
        &app.router,
        "/notes",
        json!({ "bookId": other, "title": "Calls", "body": "- [ ] Call Sam @2026-10-25T09:00\n" }),
    )
    .await;
    write(
        &app.router,
        "/notes",
        json!({ "bookId": "trash", "title": "Old", "body": "- [ ] Gone @2026-01-01\n" }),
    )
    .await;
    let open = tool_json(&tool(&app.router, "kanso-list-tasks", json!({})).await);
    let texts: Vec<&str> = open
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["text"].as_str().unwrap())
        .collect();
    assert_eq!(
        texts,
        [
            "Call Sam @2026-10-25T09:00",
            "Pay rent @2026-11-01",
            "Someday"
        ]
    );
    let bare = plan.strip_prefix("note:").unwrap();
    assert_eq!(
        open[1],
        json!({
            "text": "Pay rent @2026-11-01",
            "done": false,
            "due": "2026-11-01",
            "line": 0,
            "noteId": plan,
            "noteTitle": "Plan",
            "bookId": book_id,
            "uri": format!("inkdrop://note/{bare}"),
        })
    );
    let done = tool_json(&tool(&app.router, "kanso-list-tasks", json!({ "status": "done" })).await);
    assert_eq!(done.as_array().unwrap().len(), 1);
    assert_eq!(done[0]["text"], "Book flights @2026-10-02");
    let all = tool_json(
        &tool(
            &app.router,
            "kanso-list-tasks",
            json!({ "status": "all", "bookId": book_id }),
        )
        .await,
    );
    assert_eq!(all.as_array().unwrap().len(), 3);
    let window = tool_json(
        &tool(
            &app.router,
            "kanso-list-tasks",
            json!({ "dueAfter": "2026-10-24", "dueBefore": "2026-11-01", "limit": 5 }),
        )
        .await,
    );
    assert_eq!(window.as_array().unwrap().len(), 1);
    assert_eq!(window[0]["due"], "2026-10-25T09:00");
    let none = tool(
        &app.router,
        "kanso-list-tasks",
        json!({ "dueBefore": "2000-01-01" }),
    )
    .await;
    assert_eq!(text(&none, 0), "No tasks match.");
    let bad = tool(
        &app.router,
        "kanso-list-tasks",
        json!({ "status": "later" }),
    )
    .await;
    assert_eq!(bad["isError"], true);
}

#[tokio::test]
async fn related_notes_through_kanso_tools() {
    let app = app();
    let (book_id, _) = write(&app.router, "/books", json!({ "name": "B" })).await;
    let mut ids = Vec::new();
    for (title, body) in [
        ("Apples", "apples and pears"),
        ("Orchard", "apples, pears, plums"),
        ("Engines", "pistons and gears"),
    ] {
        let (id, _) = write(
            &app.router,
            "/notes",
            json!({ "bookId": book_id, "title": title, "body": body }),
        )
        .await;
        ids.push(id);
    }
    // Off: an empty list, and why.
    let off = tool(
        &app.router,
        "kanso-related-notes",
        json!({ "noteId": ids[0] }),
    )
    .await;
    assert_eq!(text(&off, 0), "[]");
    assert!(text(&off, 1).contains("Related notes are off"));
    // On, before the note is indexed.
    app.store.set_embedding_model(Some("test")).unwrap();
    let waiting = tool(
        &app.router,
        "kanso-related-notes",
        json!({ "noteId": ids[0] }),
    )
    .await;
    assert_eq!(text(&waiting, 0), "[]");
    assert!(text(&waiting, 1).contains("isn't indexed"));
    // Indexed: the closest first, the note itself left out.
    let vectors = [vec![1., 0.1], vec![0.9, 0.2], vec![0., 1.]];
    let rows: Vec<(String, String, Vec<f32>)> = ids
        .iter()
        .zip(vectors)
        .map(|(id, v)| (id.clone(), hash_of(&app.store, id), v))
        .collect();
    app.store.put_embeddings("test", &rows).unwrap();
    let found = tool_json(
        &tool(
            &app.router,
            "kanso-related-notes",
            json!({ "noteId": ids[0], "limit": 1 }),
        )
        .await,
    );
    assert_eq!(found.as_array().unwrap().len(), 1);
    assert_eq!(found[0]["noteId"], ids[1]);
    assert_eq!(found[0]["title"], "Orchard");
    assert!(found[0]["similarity"].as_f64().unwrap() > 0.9);
    let missing = tool(
        &app.router,
        "kanso-related-notes",
        json!({ "noteId": "note:missing" }),
    )
    .await;
    assert_eq!(missing["isError"], true);
}

/// A stand-in model that knows two subjects: fruit and machines.
struct Initials;

impl kanso_store::Embedder for Initials {
    fn model(&self) -> &str {
        "test"
    }
    fn dim(&self) -> usize {
        2
    }
    fn embed(&self, texts: &[String]) -> Vec<Vec<f32>> {
        texts
            .iter()
            .map(|t| {
                let fruit = t.matches("fruit").count() as f32;
                let machine = t.matches("machine").count() as f32;
                vec![fruit, machine]
            })
            .collect()
    }
}

#[tokio::test]
async fn search_by_meaning_through_kanso_tools() {
    let app = app();
    let (book_id, _) = write(&app.router, "/books", json!({ "name": "B" })).await;
    let mut ids = Vec::new();
    for (title, body) in [
        ("Apples", "apples and pears"),
        ("Engines", "pistons and gears"),
    ] {
        let (id, _) = write(
            &app.router,
            "/notes",
            json!({ "bookId": book_id, "title": title, "body": body }),
        )
        .await;
        ids.push(id);
    }
    let search = |query: &str| {
        tool(
            &app.router,
            "kanso-search-by-meaning",
            json!({ "query": query, "limit": 5 }),
        )
    };
    // Off: an empty list, and why.
    let off = search("fruit").await;
    assert_eq!(text(&off, 0), "[]");
    assert!(text(&off, 1).contains("related notes, which are off"));
    // On, but no model where the API runs (as under `kanso serve`).
    app.store.set_embedding_model(Some("test")).unwrap();
    let rows = vec![
        (ids[0].clone(), hash_of(&app.store, &ids[0]), vec![1., 0.]),
        (ids[1].clone(), hash_of(&app.store, &ids[1]), vec![0., 1.]),
    ];
    app.store.put_embeddings("test", &rows).unwrap();
    let no_model = search("fruit").await;
    assert_eq!(text(&no_model, 0), "[]");
    assert!(text(&no_model, 1).contains("isn't loaded"));
    // The app lends its model: the note about fruit, though it never says
    // "fruit", and not the one about machines.
    app.store.lend_embedder(Some(std::sync::Arc::new(Initials)));
    let found = tool_json(&search("fruit").await);
    assert_eq!(found.as_array().unwrap().len(), 1);
    assert_eq!(found[0]["noteId"], ids[0]);
    assert_eq!(found[0]["title"], "Apples");
    assert_eq!(found[0]["similarity"], 1.0);
    let nothing = search("weather").await;
    assert!(text(&nothing, 1).contains("close enough"));
    let empty = tool(
        &app.router,
        "kanso-search-by-meaning",
        json!({ "query": "  " }),
    )
    .await;
    assert_eq!(empty["isError"], true);
}

#[tokio::test]
async fn mcp_note_writes_never_set_a_published_link() {
    let app = app();
    let planted = json!({ "shareId": "AAAAAAAAAAAAAAAAAAAAAA", "key": "attacker-key", "publishedAt": 2, "publishedRev": "1-b" });
    let created = tool_json(
        &tool(
            &app.router,
            "create-note",
            json!({ "bookId": "trash", "title": "t", "body": "b", "status": "none", "kansoPublish": planted }),
        )
        .await,
    );
    let (_, doc) = get(
        &app.router,
        &format!("/{}", created["id"].as_str().unwrap()),
    )
    .await;
    assert!(doc.get("kansoPublish").is_none(), "{doc}");

    // A note the app published keeps its link through update-note.
    let ours = json!({ "shareId": "q3T9xW2LmN8vPz4RkY7aBw", "key": "app-key", "publishedAt": 1, "publishedRev": "1-a" });
    let res = app
        .store
        .put(
            kanso_core::DocType::Note,
            json!({ "bookId": "trash", "title": "t", "body": "b", "kansoPublish": ours }),
            kanso_store::Put::Upsert,
            kanso_store::Origin::Ui,
        )
        .unwrap();
    tool_json(
        &tool(
            &app.router,
            "update-note",
            json!({ "_id": res.id, "_rev": res.rev, "title": "Plan v2", "kansoPublish": planted }),
        )
        .await,
    );
    let (_, doc) = get(&app.router, &format!("/{}", res.id)).await;
    assert_eq!(doc["title"], "Plan v2");
    assert_eq!(doc["kansoPublish"], ours, "{doc}");
}

#[tokio::test]
async fn daily_notes_and_templates_through_kanso_tools() {
    let app = app();
    let made = tool_json(
        &tool(
            &app.router,
            "kanso-create-daily-note",
            json!({ "date": "2026-10-20" }),
        )
        .await,
    );
    assert_eq!(
        made,
        json!({
            "noteId": "note:daily-2026-10-20",
            "title": "2026-10-20",
            "bookId": "book:journal",
            "uri": "inkdrop://note/daily-2026-10-20",
            "created": true,
        })
    );
    let again = tool_json(
        &tool(
            &app.router,
            "kanso-create-daily-note",
            json!({ "date": "2026-10-20" }),
        )
        .await,
    );
    assert_eq!(again["created"], false);
    let today = tool_json(&tool(&app.router, "kanso-create-daily-note", json!({})).await);
    assert!(
        today["noteId"]
            .as_str()
            .unwrap()
            .starts_with("note:daily-20")
    );
    let bad = tool(
        &app.router,
        "kanso-create-daily-note",
        json!({ "date": "20 October" }),
    )
    .await;
    assert_eq!(bad["isError"], true);

    let (book_id, _) = write(
        &app.router,
        "/books",
        json!({ "_id": "book:templates", "name": "Templates", "kansoRole": "templates" }),
    )
    .await;
    let (template, _) = write(
        &app.router,
        "/notes",
        json!({ "bookId": book_id, "title": "Meeting", "body": "# {{title}}\n\n{{cursor}}{{who}}" }),
    )
    .await;
    let made = tool_json(
        &tool(
            &app.router,
            "kanso-create-from-template",
            json!({ "templateId": template, "title": "Standup" }),
        )
        .await,
    );
    assert_eq!(made["title"], "Standup");
    assert_eq!(made["bookId"], "book:inbox");
    let note = tool_json(
        &tool(
            &app.router,
            "read-note",
            json!({ "noteId": made["noteId"] }),
        )
        .await,
    );
    assert_eq!(note["body"], "# Standup\n\n{{who}}");
    assert_eq!(note["sourceTemplateId"], template);
    let missing = tool(
        &app.router,
        "kanso-create-from-template",
        json!({ "templateId": "note:missing", "title": "x" }),
    )
    .await;
    assert_eq!(missing["isError"], true);
}

/// The hash of note `id`'s words as the embedding worker would send it.
fn hash_of(store: &kanso_store::Store, id: &str) -> String {
    store
        .embedding_queue("test")
        .unwrap()
        .pending
        .into_iter()
        .find(|pending| pending.id == id)
        .unwrap()
        .hash
}
