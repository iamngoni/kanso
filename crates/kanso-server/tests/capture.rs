//! The Capture access level over REST and `/mcp` (issue #390, step 11): a
//! key that reads, makes new notes, adds tasks and ticks them, and can
//! change or delete nothing that is already written. CI pins the seed with
//! `PROPTEST_RNG_SEED`.

mod common;

use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use common::*;
use kanso_core::DocType;
use kanso_server::{KeyAccess, NewKey};
use kanso_store::{Origin, Put, Store};
use proptest::prelude::*;
use proptest::test_runner::{FileFailurePersistence, TestCaseError};
use serde_json::{Value, json};

const ADDS: [&str; 3] = ["create-note", "kanso-add-task", "kanso-set-task-done"];

fn config(cases: u32) -> ProptestConfig {
    ProptestConfig {
        cases,
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
        ..ProptestConfig::default()
    }
}

type CaseResult = Result<(), TestCaseError>;

fn block_on(future: impl Future<Output = CaseResult>) -> CaseResult {
    runtime().block_on(future)
}

fn mint(app: &App, access: KeyAccess, books: Option<Vec<String>>) -> String {
    app.keys
        .mint(NewKey {
            name: "Raycast".into(),
            client: "other".into(),
            access,
            books,
        })
        .unwrap()
        .key
}

fn signed(key: &str, method: Method, uri: &str, body: Option<&Value>) -> Request<Body> {
    let builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::AUTHORIZATION, bearer(key));
    match body {
        Some(body) => builder
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap(),
        None => builder.body(Body::empty()).unwrap(),
    }
}

async fn call_as(
    router: &Router,
    key: &str,
    method: Method,
    uri: &str,
    body: Option<&Value>,
) -> (StatusCode, Value) {
    let response = send(router, signed(key, method, uri, body)).await;
    (response.status, response.json())
}

async fn tool_as(router: &Router, key: &str, name: &str, arguments: Value) -> Value {
    let message = json!({
        "jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": { "name": name, "arguments": arguments },
    });
    let (status, answer) = call_as(router, key, Method::POST, "/mcp", Some(&message)).await;
    assert_eq!(status, StatusCode::OK, "{answer}");
    answer["result"].clone()
}

async fn tool_names(router: &Router, key: &str) -> Vec<String> {
    let message = json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/list" });
    let (_, answer) = call_as(router, key, Method::POST, "/mcp", Some(&message)).await;
    answer["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tool| tool["name"].as_str().unwrap().to_owned())
        .collect()
}

fn error_text(result: &Value) -> String {
    assert_eq!(result["isError"], true, "{result}");
    text(result, 0).to_owned()
}

fn put(store: &Store, doc_type: DocType, doc: Value) -> String {
    store
        .put(doc_type, doc, Put::Upsert, Origin::Sync)
        .unwrap()
        .id
}

fn lock(store: &Store, id: &str) {
    store
        .put(
            DocType::Note,
            json!({ "_id": id, "kansoLock": { "salt": "c2FsdA" } }),
            Put::Merge,
            Origin::Sync,
        )
        .unwrap();
}

/// What a library holds: a notebook the key may use, one it may not, a
/// locked notebook, and notes in each.
struct World {
    work: String,
    other: String,
    vault: String,
    note: String,
    outside: String,
    locked: String,
    vaulted: String,
}

fn world(store: &Store) -> World {
    let work = put(store, DocType::Book, json!({ "name": "Work" }));
    let other = put(store, DocType::Book, json!({ "name": "Other" }));
    let vault = put(
        store,
        DocType::Book,
        json!({ "name": "Vault", "kansoLock": { "salt": "c2FsdA" } }),
    );
    let note = put(
        store,
        DocType::Note,
        json!({ "title": "Plan", "body": "- [ ] call Sam\n- [ ] book room", "bookId": work }),
    );
    let outside = put(
        store,
        DocType::Note,
        json!({ "title": "Elsewhere", "body": "- [ ] outside task", "bookId": other }),
    );
    let locked = put(
        store,
        DocType::Note,
        json!({ "title": "Sealed", "body": "- [ ] sealed task", "bookId": work }),
    );
    lock(store, &locked);
    let vaulted = put(
        store,
        DocType::Note,
        json!({ "title": "Vaulted", "body": "- [ ] vaulted task", "bookId": vault }),
    );
    World {
        work,
        other,
        vault,
        note,
        outside,
        locked,
        vaulted,
    }
}

fn rev(store: &Store, id: &str) -> Option<String> {
    store
        .get_api(id, None, false)
        .ok()
        .map(|doc| doc["_rev"].as_str().unwrap().to_owned())
}

fn body(store: &Store, id: &str) -> String {
    store.note(id).unwrap().unwrap().body
}

#[tokio::test]
async fn the_catalog_is_the_reads_and_the_three_that_add() {
    let app = app();
    let key = mint(&app, KeyAccess::Capture, None);
    let offered = tool_names(&app.router, &key).await;
    for name in ADDS {
        assert!(offered.iter().any(|n| n == name), "{name} not offered");
    }
    for name in &offered {
        assert!(
            ADDS.contains(&name.as_str())
                || (kanso_server::READ_TOOLS.contains(&name.as_str()) && !name.contains("pdf")),
            "{name} offered to a Capture key"
        );
    }
    // What a Read key is offered, Capture is offered too.
    let read = mint(&app, KeyAccess::Read, None);
    for name in tool_names(&app.router, &read).await {
        assert!(offered.contains(&name), "{name}");
    }
    assert!(offered.iter().any(|n| n == "kanso-add-task"));
    assert!(offered.iter().any(|n| n == "kanso-parse-when"));
}

#[tokio::test]
async fn a_capture_key_is_gated_on_routes_that_only_look_like_pairing() {
    let app = app();
    let key = mint(&app, KeyAccess::Capture, None);
    for (method, uri) in [
        (Method::PATCH, "/pair/x/canvas"),
        (Method::PATCH, "/pair/x/y/canvas"),
    ] {
        let (status, _) = call_as(&app.router, &key, method.clone(), uri, Some(&json!({}))).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{method} {uri}");
    }
    // `/pair/<id>` is the pairing route's own and answers GET only: any
    // other method is refused there, key or no key.
    for method in [Method::POST, Method::DELETE, Method::PATCH] {
        let (status, _) = call_as(
            &app.router,
            &key,
            method.clone(),
            "/pair/x",
            Some(&json!({})),
        )
        .await;
        assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED, "{method}");
    }
}

#[tokio::test]
async fn a_capture_key_makes_a_note_over_rest_and_it_is_saved_at_once() {
    let app = app();
    let w = world(&app.store);
    let key = mint(&app, KeyAccess::Capture, None);
    let (status, answer) = call_as(
        &app.router,
        &key,
        Method::POST,
        "/notes",
        Some(&json!({ "bookId": w.work, "title": "Idea", "body": "a video about pixels" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{answer}");
    let id = answer["id"].as_str().unwrap();
    assert_eq!(body(&app.store, id), "a video about pixels");
    let (status, got) = call_as(&app.router, &key, Method::GET, &format!("/{id}"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(got["title"], "Idea");
}

#[tokio::test]
async fn a_capture_key_makes_a_note_adds_a_task_and_ticks_it_over_mcp() {
    let app = app();
    let w = world(&app.store);
    let key = mint(&app, KeyAccess::Capture, None);

    let made = tool_as(
        &app.router,
        &key,
        "create-note",
        json!({ "bookId": w.work, "title": "Idea", "body": "pixels" }),
    )
    .await;
    assert_ne!(made["isError"], true, "{made}");

    let added = tool_as(
        &app.router,
        &key,
        "kanso-add-task",
        json!({ "text": "film Pixel review", "noteId": w.note }),
    )
    .await;
    assert_ne!(added["isError"], true, "{added}");
    assert_eq!(
        body(&app.store, &w.note),
        "- [ ] call Sam\n- [ ] book room\n- [ ] film Pixel review\n"
    );

    let ticked = tool_as(
        &app.router,
        &key,
        "kanso-set-task-done",
        json!({ "noteId": w.note, "line": 0, "text": "call Sam", "done": true }),
    )
    .await;
    assert_ne!(ticked["isError"], true, "{ticked}");
    assert!(body(&app.store, &w.note).starts_with("- [x] call Sam"));

    // A tick is never cleared: that would change what is written.
    let cleared = tool_as(
        &app.router,
        &key,
        "kanso-set-task-done",
        json!({ "noteId": w.note, "line": 0, "text": "call Sam", "done": false }),
    )
    .await;
    assert!(error_text(&cleared).contains("can tick a task but not clear"));
    assert!(body(&app.store, &w.note).starts_with("- [x] call Sam"));

    // The daily note is made when missing: a task with no place to go.
    let daily = tool_as(
        &app.router,
        &key,
        "kanso-add-task",
        json!({ "text": "water plants tomorrow" }),
    )
    .await;
    assert_ne!(daily["isError"], true, "{daily}");
}

#[tokio::test]
async fn a_capture_key_reads_but_does_not_see_locked_notes() {
    let app = app();
    let w = world(&app.store);
    let key = mint(&app, KeyAccess::Capture, None);
    let found = tool_as(&app.router, &key, "list-notes", json!({})).await;
    let listing = text(&found, 0).to_owned();
    assert!(listing.contains(&w.note), "{listing}");
    assert!(!listing.contains(&w.locked) && !listing.contains(&w.vaulted));
    for id in [&w.locked, &w.vaulted] {
        let (status, _) = call_as(&app.router, &key, Method::GET, &format!("/{id}"), None).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{id}");
        let read = tool_as(&app.router, &key, "read-note", json!({ "noteId": id })).await;
        assert_eq!(read["isError"], true, "{read}");
    }
}

#[tokio::test]
async fn a_capture_key_cannot_touch_a_locked_note_or_a_locked_notebook() {
    let app = app();
    let w = world(&app.store);
    let key = mint(&app, KeyAccess::Capture, None);
    let before = (rev(&app.store, &w.locked), rev(&app.store, &w.vaulted));
    for note in [&w.locked, &w.vaulted] {
        let added = tool_as(
            &app.router,
            &key,
            "kanso-add-task",
            json!({ "text": "sneak", "noteId": note }),
        )
        .await;
        assert_eq!(added["isError"], true, "{added}");
        let ticked = tool_as(
            &app.router,
            &key,
            "kanso-set-task-done",
            json!({ "noteId": note, "line": 0, "text": "sealed task", "done": true }),
        )
        .await;
        assert_eq!(ticked["isError"], true, "{ticked}");
    }
    // Nor a new note into the locked notebook.
    let made = tool_as(
        &app.router,
        &key,
        "create-note",
        json!({ "bookId": w.vault, "title": "t", "body": "b" }),
    )
    .await;
    assert_eq!(made["isError"], true, "{made}");
    let (status, _) = call_as(
        &app.router,
        &key,
        Method::POST,
        "/notes",
        Some(&json!({ "bookId": w.vault, "title": "t", "body": "b" })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(
        before,
        (rev(&app.store, &w.locked), rev(&app.store, &w.vaulted))
    );
}

#[tokio::test]
async fn a_capture_key_for_one_notebook_adds_only_there() {
    let app = app();
    let w = world(&app.store);
    let key = mint(&app, KeyAccess::Capture, Some(vec![w.work.clone()]));
    let before = rev(&app.store, &w.outside);

    // Outside its notebook, a note is not found and a destination is refused.
    let tick = tool_as(
        &app.router,
        &key,
        "kanso-set-task-done",
        json!({ "noteId": w.outside, "line": 0, "text": "outside task", "done": true }),
    )
    .await;
    assert_eq!(tick["isError"], true, "{tick}");
    let add = tool_as(
        &app.router,
        &key,
        "kanso-add-task",
        json!({ "text": "sneak", "noteId": w.outside }),
    )
    .await;
    assert_eq!(add["isError"], true, "{add}");
    let made = tool_as(
        &app.router,
        &key,
        "create-note",
        json!({ "bookId": w.other, "title": "t", "body": "b" }),
    )
    .await;
    assert!(error_text(&made).contains("limited to some notebooks"));
    let (status, _) = call_as(
        &app.router,
        &key,
        Method::POST,
        "/notes",
        Some(&json!({ "bookId": w.other, "title": "t", "body": "b" })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    // A task with no note named goes to the daily note, which is not in
    // its notebook.
    let daily = tool_as(&app.router, &key, "kanso-add-task", json!({ "text": "x" })).await;
    assert_eq!(daily["isError"], true, "{daily}");
    assert_eq!(before, rev(&app.store, &w.outside));

    // Inside it, the same calls work.
    let ok = tool_as(
        &app.router,
        &key,
        "create-note",
        json!({ "bookId": w.work, "title": "t", "body": "b" }),
    )
    .await;
    assert_ne!(ok["isError"], true, "{ok}");
}

#[tokio::test]
async fn a_capture_key_cannot_replace_an_existing_note_or_set_anything_but_a_new_note() {
    let app = app();
    let w = world(&app.store);
    let key = mint(&app, KeyAccess::Capture, None);
    let before = rev(&app.store, &w.note);
    for extra in [
        json!({ "_id": w.note }),
        json!({ "_id": "note:fresh" }),
        json!({ "_rev": "1-x" }),
        json!({ "_deleted": true }),
        json!({ "kansoLock": { "salt": "c2FsdA" } }),
        json!({ "pinned": true }),
    ] {
        let mut note = json!({ "bookId": w.work, "title": "Hijack", "body": "gone" });
        for (field, value) in extra.as_object().unwrap() {
            note[field] = value.clone();
        }
        let (status, answer) =
            call_as(&app.router, &key, Method::POST, "/notes", Some(&note)).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{note}: {answer}");
        // The tool's schema drops what a note is not made with, so a note
        // is made, new; the one that is there is not touched.
        let result = tool_as(&app.router, &key, "create-note", note.clone()).await;
        if result["isError"] != true {
            let made = tool_json(&result);
            assert_ne!(made["id"], json!(w.note), "{note}: {result}");
            assert_ne!(made["id"], json!("note:fresh"), "{note}: {result}");
            assert_eq!(body(&app.store, made["id"].as_str().unwrap()), "gone");
        }
        assert_eq!(before, rev(&app.store, &w.note), "{note}");
    }
    for book in ["trash", "template"] {
        let (status, _) = call_as(
            &app.router,
            &key,
            Method::POST,
            "/notes",
            Some(&json!({ "bookId": book, "title": "t", "body": "b" })),
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{book}");
    }
    assert_eq!(before, rev(&app.store, &w.note));
    assert_eq!(body(&app.store, &w.note), "- [ ] call Sam\n- [ ] book room");
}

#[tokio::test]
async fn every_other_rest_write_and_tool_is_refused_with_how_to_fix_it() {
    let app = app();
    let w = world(&app.store);
    let key = mint(&app, KeyAccess::Capture, None);
    let before = app.store.last_seq().unwrap();
    let doc = json!({ "title": "x", "name": "x", "body": "x", "bookId": w.work });
    let mut requests = vec![
        (Method::POST, "/books".to_owned()),
        (Method::POST, "/tags".to_owned()),
        (Method::POST, "/files".to_owned()),
        (Method::POST, format!("/{}", w.note)),
        (Method::POST, format!("/notes/{}", w.note)),
        (Method::PUT, format!("/{}", w.note)),
        (Method::PATCH, format!("/{}", w.note)),
        (Method::DELETE, format!("/{}", w.note)),
        (Method::DELETE, format!("/{}", w.work)),
        (Method::DELETE, format!("/notes/{}", w.note)),
        (Method::PATCH, format!("/{}/canvas", w.note)),
        (
            Method::POST,
            format!("/notes/{}/pdf/file:f/highlights", w.note),
        ),
        (Method::PUT, "/notes".to_owned()),
        (Method::DELETE, "/notes".to_owned()),
    ];
    requests.push((Method::POST, "/notes/".to_owned()));
    for (method, uri) in requests {
        let (status, answer) = call_as(&app.router, &key, method.clone(), &uri, Some(&doc)).await;
        assert!(
            status == StatusCode::FORBIDDEN || status == StatusCode::NOT_FOUND,
            "{method} {uri}: {status} {answer}"
        );
        if status == StatusCode::FORBIDDEN {
            assert!(
                answer["reason"]
                    .as_str()
                    .unwrap()
                    .contains("Settings \u{203a} Local API \u{203a} API keys"),
                "{answer}"
            );
        }
    }
    for (name, arguments) in [
        (
            "update-note",
            json!({ "_id": w.note, "title": "x", "body": "x" }),
        ),
        (
            "patch-note",
            json!({ "_id": w.note, "old_string": "call", "new_string": "x" }),
        ),
        (
            "kanso-move-note",
            json!({ "noteId": w.note, "bookId": w.other }),
        ),
        ("update-book", json!({ "_id": w.work, "name": "x" })),
        ("create-book", json!({ "name": "x" })),
        ("create-tag", json!({ "name": "x" })),
        ("update-tag", json!({ "_id": "tag:x", "name": "y" })),
        ("kanso-create-daily-note", json!({})),
        (
            "kanso-create-from-template",
            json!({ "templateId": w.note }),
        ),
        ("kanso-create-canvas", json!({ "title": "x" })),
        ("kanso-attach-file", json!({ "noteId": w.note })),
        (
            "create-file",
            json!({ "name": "x", "contentType": "image/png", "data": "AAAA" }),
        ),
        ("not-a-tool-yet", json!({})),
    ] {
        let result = tool_as(&app.router, &key, name, arguments).await;
        let message = error_text(&result);
        assert!(!message.is_empty(), "{name}");
        if name != "not-a-tool-yet" {
            assert!(
                message.contains("Settings \u{203a} Local API"),
                "{name}: {message}"
            );
        }
    }
    assert_eq!(app.store.last_seq().unwrap(), before);
}

#[tokio::test]
async fn a_capture_key_gets_no_attachments_or_pdf_routes() {
    let app = app();
    let w = world(&app.store);
    let key = mint(&app, KeyAccess::Capture, None);
    for uri in [
        "/pdf/search?query=x".to_owned(),
        format!("/notes/{}/pdf/file:f/highlights", w.note),
        format!("/notes/{}/pdf/file:f/pages/1", w.note),
    ] {
        let (status, _) = call_as(&app.router, &key, Method::GET, &uri, None).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{uri}");
    }
    for name in ["kanso-read-attachment", "read-file", "kanso-search-pdf"] {
        let result = tool_as(
            &app.router,
            &key,
            name,
            json!({ "noteId": w.note, "fileId": "file:f", "query": "x" }),
        )
        .await;
        assert_eq!(result["isError"], true, "{name}");
    }
}

#[tokio::test]
async fn a_revoked_capture_key_is_refused_at_once() {
    let app = app();
    let w = world(&app.store);
    let minted = app
        .keys
        .mint(NewKey {
            name: "Raycast".into(),
            client: "other".into(),
            access: KeyAccess::Capture,
            books: None,
        })
        .unwrap();
    let note = json!({ "bookId": w.work, "title": "t", "body": "b" });
    let (status, _) = call_as(
        &app.router,
        &minted.key,
        Method::POST,
        "/notes",
        Some(&note),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    app.keys.revoke(&minted.id).unwrap();
    let (status, _) = call_as(
        &app.router,
        &minted.key,
        Method::POST,
        "/notes",
        Some(&note),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

/// Every line of `old` is in `new` at the same place, as it was or ticked.
fn only_added_or_ticked(old: &str, new: &str) -> bool {
    let new_lines: Vec<&str> = new.split('\n').collect();
    let old_lines: Vec<&str> = old.split('\n').collect();
    new_lines.len() >= old_lines.len()
        && old_lines
            .iter()
            .zip(&new_lines)
            .all(|(old, new)| old == new || old.replacen("- [ ]", "- [x]", 1) == *new)
}

fn ids(w: &World) -> Vec<String> {
    vec![
        w.note.clone(),
        w.outside.clone(),
        w.locked.clone(),
        w.vaulted.clone(),
        w.work.clone(),
        w.other.clone(),
        w.vault.clone(),
        "note:missing".into(),
        "bogus".into(),
    ]
}

proptest! {
    #![proptest_config(config(24))]

    /// Whatever it is sent, a Capture key changes no document in a way it
    /// may not: every tool that does not add leaves the library as it was,
    /// and the three that add never touch a locked note, a note in a
    /// locked or other notebook, a notebook or a tag, and only add lines
    /// to a note it may see or tick one.
    #[test]
    fn capture_never_edits_deletes_moves_or_reaches_outside_its_scope(
        picks in prop::collection::vec(
            (0usize..80, 0usize..9, 0usize..9, any::<bool>(), "[a-zA-Z0-9 ]{0,12}", 0u32..4),
            1..12,
        ),
        limited in any::<bool>(),
    ) {
        let app = app();
        let w = world(&app.store);
        let books = limited.then(|| vec![w.work.clone()]);
        let key = mint(&app, KeyAccess::Capture, books);
        let ids = ids(&w);
        let mut guarded: Vec<String> = [&w.locked, &w.vaulted, &w.work, &w.other, &w.vault]
            .iter().map(|id| (*id).clone()).collect();
        // Only a key for one notebook is kept from the other.
        if limited {
            guarded.push(w.outside.clone());
        }
        let before: Vec<Option<String>> = guarded.iter().map(|id| rev(&app.store, id)).collect();
        let old_body = body(&app.store, &w.note);
        let seq = app.store.last_seq().unwrap();
        block_on(async {
            let all = tool_names(&app.router, KEY).await;
            for (tool_ix, note_ix, book_ix, done, text, line) in picks {
                let name = &all[tool_ix % all.len()];
                let arguments = json!({
                    "_id": ids[note_ix], "noteId": ids[note_ix], "bookId": ids[book_ix],
                    "title": text, "body": text, "text": text, "name": text, "done": done,
                    "line": line, "old_string": "call", "new_string": text,
                    "templateId": ids[note_ix], "fileId": "file:f0", "filePath": "/etc/hosts",
                    "outputPath": "/tmp/kanso-never-capture",
                });
                let before_call = app.store.last_seq().unwrap();
                let result = tool_as(&app.router, &key, name, arguments).await;
                if !ADDS.contains(&name.as_str()) {
                    let reads_only = kanso_server::READ_TOOLS.contains(&name.as_str());
                    if !reads_only {
                        prop_assert_eq!(&result["isError"], &json!(true), "{}: {}", name, result);
                    }
                    prop_assert_eq!(app.store.last_seq().unwrap(), before_call, "{} wrote", name);
                }
            }
            Ok(())
        })?;
        let after: Vec<Option<String>> = guarded.iter().map(|id| rev(&app.store, id)).collect();
        prop_assert_eq!(before, after, "a note or notebook out of reach changed");
        prop_assert!(only_added_or_ticked(&old_body, &body(&app.store, &w.note)),
            "{:?} became {:?}", old_body, body(&app.store, &w.note));
        prop_assert!(app.store.last_seq().unwrap() >= seq);
        prop_assert!(!std::path::Path::new("/tmp/kanso-never-capture").exists());
    }

    /// Every REST write route, aimed at any document, is refused with 403
    /// and changes nothing; only `POST /notes` with a new note passes.
    #[test]
    fn capture_rest_writes_are_refused_except_a_new_note(
        id in prop::sample::select(vec!["note:n0", "note:missing", "book:b0", "tag:t0", "file:f0", "bogus"]),
        method in prop::sample::select(vec![Method::POST, Method::PUT, Method::PATCH, Method::DELETE]),
        prefix in prop::sample::select(vec!["", "notes/", "note/", "books/", "tags/", "files/"]),
        suffix in prop::sample::select(vec!["", "/canvas", "/pdf/file:f/highlights"]),
        title in "[a-zA-Z ]{0,10}",
    ) {
        let app = app();
        let w = world(&app.store);
        let key = mint(&app, KeyAccess::Capture, None);
        let before = app.store.last_seq().unwrap();
        let uri = format!("/{prefix}{id}{suffix}");
        let payload = json!({ "_id": w.note, "title": title, "name": title, "body": title, "bookId": w.work });
        block_on(async {
            let (status, answer) =
                call_as(&app.router, &key, method.clone(), &uri, Some(&payload)).await;
            prop_assert!(
                status == StatusCode::FORBIDDEN || status == StatusCode::NOT_FOUND,
                "{} {}: {} {}", method, uri, status, answer
            );
            Ok(())
        })?;
        prop_assert_eq!(app.store.last_seq().unwrap(), before);
    }
}
