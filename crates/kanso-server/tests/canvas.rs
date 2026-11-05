//! Canvases through the local API and MCP (issue #45, step 12): summaries in
//! lists, the raw document on a read, validated writes, and the canvas tools.
//! CI pins the seed with `PROPTEST_RNG_SEED`.

mod common;

use std::sync::{Arc, Mutex};

use axum::http::{Method, StatusCode};
use common::*;
use kanso_core::DocType;
use kanso_core::canvas::{self, Canvas, Node, Point, Stroke};
use kanso_server::{CanvasChange, SharedCanvases, TokenStore, router_with_canvases};
use kanso_store::{Origin, Put, Store};
use proptest::prelude::*;
use proptest::test_runner::{FileFailurePersistence, TestCaseError};
use serde_json::{Value, json};

fn config(cases: u32) -> ProptestConfig {
    ProptestConfig {
        cases,
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
        ..ProptestConfig::default()
    }
}

/// A canvas with a text card, a group around it and a layer of ink.
fn drawing() -> Canvas {
    let mut c = Canvas::default();
    c.put_node(
        Node {
            id: "t1".into(),
            kind: "text".into(),
            x: 10.0,
            y: 10.0,
            width: 100.0,
            height: 40.0,
            text: Some("Launch plan".into()),
            ..Node::default()
        },
        1,
    );
    c.put_node(
        Node {
            id: "g1".into(),
            kind: "group".into(),
            width: 400.0,
            height: 300.0,
            label: Some("Q4".into()),
            ..Node::default()
        },
        1,
    );
    c.put_node(
        Node {
            id: "far".into(),
            kind: "text".into(),
            x: 900.0,
            y: 900.0,
            width: 100.0,
            height: 40.0,
            text: Some("Elsewhere".into()),
            ..Node::default()
        },
        1,
    );
    c.put_node(
        Node {
            id: "i1".into(),
            kind: "ink".into(),
            width: 1.0,
            height: 1.0,
            ..Node::default()
        },
        1,
    );
    c.add_stroke(
        "i1",
        Stroke {
            id: "s1".into(),
            tool: "pen".into(),
            color: "#000000".into(),
            width: 2.0,
            points: vec![Point::new(20.0, 20.0, 0.5), Point::new(60.0, 60.0, 0.5)],
            ..Stroke::default()
        },
        1,
    );
    c.refit_ink();
    c
}

fn make_book(store: &Store) -> String {
    store
        .put(
            DocType::Book,
            json!({ "name": "Work" }),
            Put::Upsert,
            Origin::Sync,
        )
        .unwrap()
        .id
}

/// A canvas note with `canvas` as its body.
fn make_canvas(store: &Store, book: &str, title: &str, canvas: &Canvas) -> (String, String) {
    let done = store
        .put(
            DocType::Note,
            json!({
                "title": title, "bookId": book, "doctype": "canvas",
                "body": canvas::to_body(canvas),
            }),
            Put::Upsert,
            Origin::Sync,
        )
        .unwrap();
    (done.id, done.rev)
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

async fn patch(app: &App, id: &str, body: Value) -> (StatusCode, Value) {
    call(
        &app.router,
        Method::PATCH,
        &format!("/{id}/canvas"),
        Some(&body),
    )
    .await
}

fn stored(app: &App, id: &str) -> Canvas {
    canvas::parse(&app.store.note(id).unwrap().unwrap().body).unwrap()
}

// -------------------------------------------------------------------- lists

#[tokio::test]
async fn lists_and_searches_show_a_summary_not_the_json() {
    let app = app();
    let book = make_book(&app.store);
    let (id, _) = make_canvas(&app.store, &book, "Board", &drawing());
    let (_, notes) = get(&app.router, "/notes").await;
    let row = notes
        .as_array()
        .unwrap()
        .iter()
        .find(|n| n["_id"] == id.as_str())
        .unwrap();
    let body = row["body"].as_str().unwrap();
    assert!(
        body.contains("Launch plan") && body.contains("Group: Q4"),
        "{body}"
    );
    assert!(
        !body.contains("kansoVersion") && !body.contains("```"),
        "{body}"
    );
    assert!(row["size"].as_u64().unwrap() > 100);
    // Keyword search goes through the same path.
    let (_, found) = get(&app.router, "/notes?keyword=Elsewhere").await;
    assert!(found[0]["body"].as_str().unwrap().contains("Elsewhere"));
    // MCP's list and search show it whole, not cut to 200 characters.
    let mut long = drawing();
    for n in 0..40 {
        long.put_node(
            Node {
                id: format!("w{n}"),
                kind: "text".into(),
                width: 100.0,
                height: 40.0,
                text: Some(format!("a line of words for card number {n} on the board")),
                ..Node::default()
            },
            2,
        );
    }
    make_canvas(&app.store, &book, "Long board", &long);
    let listed = tool_json(&tool(&app.router, "list-notes", json!({ "keyword": "Long" })).await);
    let summary = listed[0]["body"].as_str().unwrap();
    assert!(
        summary.len() > 200 && summary.contains("card number 39"),
        "{summary}"
    );
    let searched =
        tool_json(&tool(&app.router, "search-notes", json!({ "keyword": "Long" })).await);
    assert!(searched[0]["body"].as_str().unwrap().len() > 200);
    assert!(searched[0]["body"].as_str().unwrap().len() <= canvas::SUMMARY_CAP);
}

#[tokio::test]
async fn get_note_keeps_the_fence_and_adds_the_json() {
    let app = app();
    let book = make_book(&app.store);
    let (id, _) = make_canvas(&app.store, &book, "Board", &drawing());
    let (status, doc) = get(&app.router, &format!("/{id}")).await;
    assert_eq!(status, StatusCode::OK);
    let body = doc["body"].as_str().unwrap();
    assert!(body.starts_with("```canvas"));
    let json: Value = serde_json::from_str(doc["canvasJson"].as_str().unwrap()).unwrap();
    assert_eq!(json["nodes"].as_array().unwrap().len(), 4);
    assert_eq!(
        canvas::fenced(body).unwrap(),
        doc["canvasJson"].as_str().unwrap()
    );
    // read-note returns the same.
    let read = tool_json(&tool(&app.router, "read-note", json!({ "noteId": id })).await);
    assert_eq!(read["canvasJson"], doc["canvasJson"]);
    // A Markdown note has none.
    let (md, _) = write(
        &app.router,
        "/notes",
        json!({ "bookId": book, "title": "Plain", "body": "text" }),
    )
    .await;
    let (_, plain) = get(&app.router, &format!("/{md}")).await;
    assert!(plain.get("canvasJson").is_none());
    // Reading a canvas and writing it back whole does not store the JSON twice.
    let mut whole = doc.clone();
    whole["title"] = "Renamed".into();
    let (_, res) = post(&app.router, "/notes", whole).await;
    assert_eq!(res["ok"], true, "{res}");
    let (_, again) = get(&app.router, &format!("/{id}")).await;
    assert_eq!(again["title"], "Renamed");
    assert_eq!(again["canvasJson"], doc["canvasJson"]);
}

#[tokio::test]
async fn writes_are_validated_as_canvases() {
    let app = app();
    let book = make_book(&app.store);
    let (id, rev) = make_canvas(&app.store, &book, "Board", &drawing());
    for body in [
        "not a canvas",
        "```canvas\n{broken\n```",
        "```canvas\n{\"nodes\":[{\"id\":\"a\"}]}\n```",
    ] {
        let (status, answer) = post(
            &app.router,
            "/notes",
            json!({ "_id": id, "_rev": rev, "title": "Board", "bookId": book, "doctype": "canvas", "body": body }),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{body}: {answer}");
    }
    let (status, _) = post(
        &app.router,
        "/notes",
        json!({ "title": "New", "bookId": book, "doctype": "canvas", "body": "nope" }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    // The canvas is as it was.
    assert_eq!(stored(&app, &id), drawing());
}

// ------------------------------------------------------------- canvas tools

#[tokio::test]
async fn list_canvases_lists_canvases_only_and_not_locked_ones() {
    let app = app();
    let book = make_book(&app.store);
    let (open, _) = make_canvas(&app.store, &book, "Open board", &drawing());
    let (sealed, _) = make_canvas(&app.store, &book, "Sealed board", &drawing());
    lock(&app.store, &sealed);
    write(
        &app.router,
        "/notes",
        json!({ "bookId": book, "title": "Plain", "body": "text" }),
    )
    .await;

    let (_, rows) = get(&app.router, "/notes?doctype=canvas").await;
    let ids: Vec<_> = rows
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["_id"].clone())
        .collect();
    assert!(ids.contains(&json!(open)) && !ids.contains(&json!("note:plain")));
    assert!(
        !ids.contains(&json!(sealed)),
        "a locked canvas is listed: {rows}"
    );
    assert!(
        rows.as_array()
            .unwrap()
            .iter()
            .all(|r| r["doctype"] == "canvas")
    );
    let (status, _) = get(&app.router, "/notes?doctype=bogus").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    let listed = tool_json(&tool(&app.router, "kanso-list-canvases", json!({})).await);
    let rows = listed.as_array().unwrap();
    assert_eq!(rows.len(), 1, "{listed}");
    assert_eq!(rows[0]["_id"], open.as_str());
    assert_eq!(rows[0]["title"], "Open board");
    assert_eq!(rows[0]["bookId"], book.as_str());
    assert!(rows[0]["tags"].is_array());
    assert!(rows[0]["size"].as_u64().unwrap() > 0);
    assert!(rows[0]["summary"].as_str().unwrap().contains("Launch plan"));
    // The same parameters as list-notes.
    let by_keyword = tool_json(
        &tool(
            &app.router,
            "kanso-list-canvases",
            json!({ "keyword": "nothing-here" }),
        )
        .await,
    );
    assert!(by_keyword.as_array().unwrap().is_empty());
    let by_book = tool_json(
        &tool(
            &app.router,
            "kanso-list-canvases",
            json!({ "bookId": book }),
        )
        .await,
    );
    assert_eq!(by_book.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn read_canvas_returns_versions_rev_and_groups() {
    let app = app();
    let book = make_book(&app.store);
    let (id, rev) = make_canvas(&app.store, &book, "Board", &drawing());
    let (status, whole) = get(&app.router, &format!("/{id}/canvas")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(whole["_rev"], rev.as_str());
    let nodes = whole["canvas"]["nodes"].as_array().unwrap();
    assert!(
        nodes
            .iter()
            .all(|n| n["kansoVersion"].as_u64().unwrap() >= 1)
    );
    assert_eq!(nodes.len(), 4);
    // The alias path and the MCP tool answer the same.
    let (_, alias) = get(
        &app.router,
        &format!("/note/{}/canvas", id.trim_start_matches("note:")),
    )
    .await;
    assert_eq!(alias, whole);
    let via_tool =
        tool_json(&tool(&app.router, "kanso-read-canvas", json!({ "noteId": id })).await);
    assert_eq!(via_tool, whole);

    let (_, group) = get(&app.router, &format!("/{id}/canvas?group=g1")).await;
    let ids: Vec<_> = group["canvas"]["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["id"].as_str().unwrap().to_owned())
        .collect();
    assert!(
        ids.contains(&"t1".to_owned()) && !ids.contains(&"far".to_owned()),
        "{ids:?}"
    );
    let (status, _) = get(&app.router, &format!("/{id}/canvas?group=nope")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Not a canvas.
    let (md, _) = write(
        &app.router,
        "/notes",
        json!({ "bookId": book, "title": "Plain", "body": "text" }),
    )
    .await;
    let (status, _) = get(&app.router, &format!("/{md}/canvas")).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn edit_canvas_applies_a_batch_as_one_revision() {
    let app = app();
    let book = make_book(&app.store);
    let (id, rev) = make_canvas(&app.store, &book, "Board", &drawing());
    let before = app.store.revisions(&id).unwrap().len();
    let (status, done) = patch(
        &app,
        &id,
        json!({ "ifRev": rev, "ops": [
            { "op": "addNode", "type": "text", "x": 5, "y": 5, "text": "from an agent" },
            { "op": "moveNode", "id": "t1", "x": 200, "y": 200, "ifVersion": 1 },
            { "op": "addStrokes", "node": "i1", "strokes": [{ "points": [[1, 1, 0.5], [9, 9, 0.5]] }] },
            { "op": "rename", "title": "Renamed by an agent" },
        ] }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{done}");
    assert_eq!(done["created"]["nodes"].as_array().unwrap().len(), 1);
    assert_eq!(done["created"]["strokes"].as_array().unwrap().len(), 1);
    let note = app.store.note(&id).unwrap().unwrap();
    assert_eq!(note.rev.as_deref(), done["rev"].as_str());
    assert_eq!(note.title, "Renamed by an agent");
    let now = stored(&app, &id);
    assert_eq!(now.node("t1").unwrap().x, 200.0);
    assert_eq!(now.node("t1").unwrap().kanso_version, 2);
    // Exactly one revision for the whole batch.
    let revisions = app.store.revisions(&id).unwrap();
    assert_eq!(revisions.len(), before + 1);
    // And history says an API client made it, so one undo reverts the batch.
    assert!(revisions.iter().any(|r| r.origin == "api"), "{revisions:?}");
    // Search finds the new text: the index reads the canvas's words.
    let (_, found) = get(&app.router, "/notes?keyword=agent").await;
    assert!(!found.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn a_stale_rev_or_version_is_a_409_with_the_current_versions() {
    let app = app();
    let book = make_book(&app.store);
    let (id, rev) = make_canvas(&app.store, &book, "Board", &drawing());
    let ok = json!({ "ops": [{ "op": "moveNode", "id": "t1", "x": 1, "y": 1 }] });
    assert_eq!(patch(&app, &id, ok).await.0, StatusCode::OK);

    // The old revision is refused, and the answer says what is there now.
    let (status, answer) = patch(
        &app,
        &id,
        json!({ "ifRev": rev, "ops": [{ "op": "moveNode", "id": "far", "x": 1, "y": 1 }] }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{answer}");
    assert_ne!(answer["rev"], rev.as_str());
    assert_eq!(answer["versions"]["t1"], 2);
    // The old version of one card is refused, and names it.
    let (status, answer) = patch(
        &app,
        &id,
        json!({ "ops": [
            { "op": "moveNode", "id": "far", "x": 7, "y": 7 },
            { "op": "updateNode", "id": "t1", "text": "overwrite", "ifVersion": 1 },
        ] }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{answer}");
    assert_eq!(answer["versions"], json!({ "t1": 2 }));
    // Nothing from the refused batch was applied, not even the first operation.
    let now = stored(&app, &id);
    assert_eq!(now.node("far").unwrap().x, 900.0);
    assert_eq!(now.node("t1").unwrap().text.as_deref(), Some("Launch plan"));
}

#[tokio::test]
async fn refused_batches_say_why_and_change_nothing() {
    let app = app();
    let book = make_book(&app.store);
    let (id, _) = make_canvas(&app.store, &book, "Board", &drawing());
    let before = app.store.note(&id).unwrap().unwrap();
    for (ops, status) in [
        (
            json!([{ "op": "moveNode", "id": "ghost", "x": 1, "y": 1 }]),
            StatusCode::NOT_FOUND,
        ),
        (
            json!([{ "op": "addNode", "type": "text", "x": 1, "y": 1, "width": -3 }]),
            StatusCode::BAD_REQUEST,
        ),
        (
            json!([{ "op": "addNode", "type": "file", "x": 1, "y": 1, "file": "note:ghost" }]),
            StatusCode::NOT_FOUND,
        ),
        (
            json!([{ "op": "addNode", "type": "file", "x": 1, "y": 1, "file": "file:foreign" }]),
            StatusCode::BAD_REQUEST,
        ),
        (json!([{ "op": "explode" }]), StatusCode::BAD_REQUEST),
    ] {
        let (got, answer) = patch(&app, &id, json!({ "ops": ops })).await;
        assert_eq!(got, status, "{ops}: {answer}");
        assert!(answer["reason"].as_str().unwrap().len() > 8);
    }
    let after = app.store.note(&id).unwrap().unwrap();
    assert_eq!(before.rev, after.rev);
    // A batch that would pass the budget is refused with it.
    let (status, answer) = patch(
        &app,
        &id,
        json!({ "ops": [{ "op": "addNode", "type": "text", "x": 0, "y": 0, "text": "x".repeat(99_000) }] }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{answer}");
    let mut blocked = 0;
    for _ in 0..12 {
        let (status, answer) = patch(
            &app,
            &id,
            json!({ "ops": [{ "op": "addNode", "type": "text", "x": 0, "y": 0, "text": "x".repeat(99_000) }] }),
        )
        .await;
        if status == StatusCode::BAD_REQUEST {
            assert!(
                answer["reason"].as_str().unwrap().contains("1000000"),
                "{answer}"
            );
            blocked += 1;
        }
    }
    assert!(blocked > 0);
    assert!(app.store.note(&id).unwrap().unwrap().body.len() < canvas::MAX_BYTES);
}

#[tokio::test]
async fn the_canvas_tools_work_over_mcp() {
    let app = app();
    let book = make_book(&app.store);
    let created = tool_json(
        &tool(
            &app.router,
            "kanso-create-canvas",
            json!({ "title": "Fresh", "bookId": book }),
        )
        .await,
    );
    let id = created["id"].as_str().unwrap().to_owned();
    assert_eq!(stored(&app, &id), Canvas::default());
    let from_doc = tool_json(
        &tool(
            &app.router,
            "kanso-create-canvas",
            json!({ "title": "From JSON", "bookId": book, "canvas": serde_json::to_value(drawing()).unwrap() }),
        )
        .await,
    );
    assert!(
        canvas::parse(
            &app.store
                .note(from_doc["id"].as_str().unwrap())
                .unwrap()
                .unwrap()
                .body
        )
        .is_ok()
    );
    let broken = tool(
        &app.router,
        "kanso-create-canvas",
        json!({ "title": "Broken", "bookId": book, "canvas": { "nodes": [{ "id": "a", "type": "text", "width": -1, "height": 1 }] } }),
    )
    .await;
    assert_eq!(broken["isError"], true, "{broken}");

    let read = tool_json(&tool(&app.router, "kanso-read-canvas", json!({ "noteId": id })).await);
    let edited = tool_json(
        &tool(
            &app.router,
            "kanso-edit-canvas",
            json!({ "noteId": id, "ifRev": read["_rev"], "ops": [
                { "op": "addNode", "type": "text", "x": 0, "y": 0, "text": "hello", "id": "mine" }
            ] }),
        )
        .await,
    );
    assert_eq!(edited["created"]["nodes"], json!(["mine"]));
    // A stale edit is an error an agent can act on.
    let stale = tool(
        &app.router,
        "kanso-edit-canvas",
        json!({ "noteId": id, "ifRev": read["_rev"], "ops": [{ "op": "removeNode", "id": "mine" }] }),
    )
    .await;
    assert_eq!(stale["isError"], true);
    let words = text(&stale, 0);
    assert!(
        words.contains("changed since you read it") && words.contains("versions"),
        "{words}"
    );
    let missing = tool(
        &app.router,
        "kanso-edit-canvas",
        json!({ "noteId": id, "ops": [{ "op": "moveNode", "id": "ghost", "x": 1, "y": 1 }] }),
    )
    .await;
    assert!(text(&missing, 0).contains("ghost"));
    // The tools are listed.
    let listed = rpc(&app.router, "tools/list", json!({})).await;
    let names: Vec<_> = listed["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap().to_owned())
        .collect();
    for name in [
        "kanso-list-canvases",
        "kanso-read-canvas",
        "kanso-edit-canvas",
        "kanso-create-canvas",
    ] {
        assert!(names.contains(&name.to_owned()), "{name}");
    }
}

// ------------------------------------------------------------------- locked

#[tokio::test]
async fn a_locked_canvas_is_empty_or_missing_everywhere() {
    let app = app();
    let book = make_book(&app.store);
    let mut secret = drawing();
    secret.node_mut("t1").unwrap().text = Some("zebra-secret".into());
    let (id, _) = make_canvas(&app.store, &book, "Locked board", &secret);
    lock(&app.store, &id);
    let before = app.store.note(&id).unwrap().unwrap();

    // A key is told there is no such note.
    let (status, doc) = get(&app.router, &format!("/{id}")).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{doc}");
    let (_, notes) = get(&app.router, "/notes").await;
    for row in notes.as_array().unwrap() {
        assert!(!row.to_string().contains("zebra-secret"), "{row}");
    }
    for uri in [format!("/{id}/canvas"), format!("/{id}/canvas?group=g1")] {
        assert_eq!(get(&app.router, &uri).await.0, StatusCode::NOT_FOUND);
    }
    let read = tool(&app.router, "kanso-read-canvas", json!({ "noteId": id })).await;
    assert_eq!(read["isError"], true);
    assert!(!read.to_string().contains("zebra-secret"));
    let (status, _) = patch(
        &app,
        &id,
        json!({ "ops": [{ "op": "rename", "title": "x" }] }),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let listed = tool(&app.router, "kanso-list-canvases", json!({})).await;
    assert!(
        !listed.to_string().contains("zebra-secret")
            && !listed.to_string().contains("Locked board")
    );
    let found = tool(
        &app.router,
        "search-notes",
        json!({ "keyword": "zebra-secret" }),
    )
    .await;
    assert!(!found.to_string().contains("zebra-secret"));
    assert_eq!(app.store.note(&id).unwrap().unwrap().rev, before.rev);
}

fn hostile_op() -> impl Strategy<Value = Value> {
    let id = prop::sample::select(vec!["t1", "g1", "far", "i1", "s1", "ghost"]);
    let n = prop_oneof![-10.0f64..1500.0, Just(1e13), Just(0.0)];
    prop_oneof![
        (n.clone(), n.clone()).prop_map(|(x, y)| json!({ "op": "addNode", "type": "text", "x": x, "y": y, "text": "q" })),
        (id.clone(), n.clone(), n.clone()).prop_map(|(i, x, y)| json!({ "op": "moveNode", "id": i, "x": x, "y": y })),
        id.clone().prop_map(|i| json!({ "op": "removeNode", "id": i })),
        (id.clone(), id.clone()).prop_map(|(a, b)| json!({ "op": "addEdge", "fromNode": a, "toNode": b })),
        (id.clone(), n.clone()).prop_map(|(i, p)| json!({ "op": "addStrokes", "node": i, "strokes": [{ "points": [[p, p, 0.5]] }] })),
        id.clone().prop_map(|i| json!({ "op": "updateNode", "id": i, "text": "zebra-secret" })),
        id.prop_map(|i| json!({ "op": "removeStrokes", "node": "i1", "strokes": [i] })),
        Just(json!({ "op": "rename", "title": "zebra-secret" })),
    ]
}

proptest! {
    #![proptest_config(config(24))]

    /// No sequence of operations stores a canvas that doesn't open, or one
    /// over the budget; a refused batch leaves the note as it was; and a
    /// locked canvas is never reached, whatever is asked of it.
    #[test]
    fn no_operation_sequence_breaks_a_canvas_or_reaches_a_locked_one(
        batches in prop::collection::vec(prop::collection::vec(hostile_op(), 0..8), 1..5),
        via_tool in any::<bool>(),
    ) {
        let app = app();
        let book = make_book(&app.store);
        let (open, _) = make_canvas(&app.store, &book, "Open", &drawing());
        let (sealed, _) = make_canvas(&app.store, &book, "Sealed", &drawing());
        lock(&app.store, &sealed);
        let sealed_before = app.store.note(&sealed).unwrap().unwrap();
        let result: Result<(), TestCaseError> = runtime().block_on(async {
            for ops in &batches {
                let before = app.store.note(&open).unwrap().unwrap();
                let (status, answer) = if via_tool {
                    let result = tool(&app.router, "kanso-edit-canvas", json!({ "noteId": open, "ops": ops })).await;
                    let failed = result["isError"] == true;
                    (if failed { StatusCode::BAD_REQUEST } else { StatusCode::OK }, result)
                } else {
                    patch(&app, &open, json!({ "ops": ops })).await
                };
                let after = app.store.note(&open).unwrap().unwrap();
                if status == StatusCode::OK {
                    prop_assert!(after.body.len() < canvas::MAX_BYTES);
                } else {
                    prop_assert_eq!(&before.rev, &after.rev, "{}", answer);
                    prop_assert_eq!(&before.body, &after.body);
                }
                // Whatever is stored opens and passes the canvas rules.
                let parsed = canvas::parse(&after.body);
                prop_assert!(parsed.is_ok(), "{:?}", parsed);
                // The same batch against the locked canvas finds nothing.
                let (sealed_status, sealed_answer) = patch(&app, &sealed, json!({ "ops": ops })).await;
                prop_assert_eq!(sealed_status, StatusCode::NOT_FOUND);
                prop_assert!(!sealed_answer.to_string().contains("Sealed"));
                let tool_answer = tool(&app.router, "kanso-edit-canvas", json!({ "noteId": sealed, "ops": ops })).await;
                prop_assert_eq!(&tool_answer["isError"], &json!(true));
                let read = get(&app.router, &format!("/{sealed}/canvas")).await;
                prop_assert_eq!(read.0, StatusCode::NOT_FOUND);
            }
            Ok(())
        });
        result?;
        let sealed_after = app.store.note(&sealed).unwrap().unwrap();
        prop_assert_eq!(sealed_before.rev, sealed_after.rev);
        prop_assert_eq!(sealed_before.body, sealed_after.body);
    }
}

// ------------------------------------------------------------------- shared

/// A shared canvas as the app's engine would hold it: the canvas itself, and
/// a log of what was done to it.
struct Room {
    canvas: Mutex<Canvas>,
    calls: Mutex<usize>,
    title: Mutex<Option<String>>,
}

impl SharedCanvases for Room {
    fn edit(&self, _note_id: &str, change: &mut CanvasChange<'_>) -> Result<bool, String> {
        let mut canvas = self.canvas.lock().unwrap();
        let (next, title) = change(&canvas)?;
        *canvas = next;
        if title.is_some() {
            *self.title.lock().unwrap() = title;
        }
        *self.calls.lock().unwrap() += 1;
        Ok(true)
    }
}

fn share(store: &Store, id: &str) {
    store
        .put(
            DocType::Note,
            json!({ "_id": id, "kansoShare": { "id": "share-1", "role": "owner", "owner": "me" } }),
            Put::Merge,
            Origin::Collab,
        )
        .unwrap();
}

fn with_room(app: &App, room: Option<Arc<Room>>) -> axum::Router {
    router_with_canvases(
        app.store.clone(),
        &common::config(),
        TokenStore::new(),
        app.keys.clone(),
        room.map(|r| r as Arc<dyn SharedCanvases>),
    )
}

#[tokio::test]
async fn a_shared_canvas_is_changed_through_its_room_and_never_by_text() {
    let app = app();
    let book = make_book(&app.store);
    let (id, _) = make_canvas(&app.store, &book, "Shared board", &drawing());
    share(&app.store, &id);
    let rev = app.store.note(&id).unwrap().unwrap().rev.unwrap();
    let room = Arc::new(Room {
        canvas: Mutex::new(drawing()),
        calls: Mutex::new(0),
        title: Mutex::new(None),
    });
    let router = with_room(&app, Some(room.clone()));

    // A whole-body write is refused, as before.
    let (status, _) = call(
        &router,
        Method::POST,
        "/notes",
        Some(&json!({ "_id": id, "_rev": rev, "title": "Shared board", "bookId": book, "doctype": "canvas",
            "body": canvas::to_body(&Canvas::default()) })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(room.canvas.lock().unwrap().nodes.len(), 4);

    // A batch goes to the room as one transaction.
    let (status, done) = call(
        &router,
        Method::PATCH,
        &format!("/{id}/canvas"),
        Some(&json!({ "ops": [
            { "op": "addNode", "type": "text", "x": 1, "y": 2, "text": "agent card", "id": "agent1" },
            { "op": "moveNode", "id": "t1", "x": 77, "y": 88 },
        ] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{done}");
    assert_eq!(*room.calls.lock().unwrap(), 1);
    let live = room.canvas.lock().unwrap().clone();
    assert!(live.node("agent1").is_some());
    assert_eq!(live.node("t1").unwrap().x, 77.0);
    // The note's own body was not rewritten by the API.
    assert_eq!(stored(&app, &id), drawing());

    // A refused batch changes nothing in the room, and says why.
    let (status, answer) = call(
        &router,
        Method::PATCH,
        &format!("/{id}/canvas"),
        Some(&json!({ "ops": [
            { "op": "moveNode", "id": "far", "x": 1, "y": 1 },
            { "op": "updateNode", "id": "t1", "text": "x", "ifVersion": 1 },
        ] })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{answer}");
    assert_eq!(room.canvas.lock().unwrap().node("far").unwrap().x, 900.0);
    // Pictures are refused until shared canvases carry attachments.
    let (status, answer) = call(
        &router,
        Method::PATCH,
        &format!("/{id}/canvas"),
        Some(&json!({ "ops": [{ "op": "addNode", "type": "file", "x": 0, "y": 0, "file": "file:pic" }] })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(answer["reason"].as_str().unwrap().contains("shared canvas"));

    // Without a room (signed out) it is refused, and says how to fix it.
    let (status, answer) = call(
        &with_room(&app, None),
        Method::PATCH,
        &format!("/{id}/canvas"),
        Some(&json!({ "ops": [{ "op": "rename", "title": "x" }] })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(answer["reason"].as_str().unwrap().contains("sign in"));
}

#[tokio::test]
async fn a_shared_rename_goes_with_the_change_and_a_stale_rev_is_refused() {
    let app = app();
    let book = make_book(&app.store);
    let (id, _) = make_canvas(&app.store, &book, "Shared board", &drawing());
    share(&app.store, &id);
    let rev = app.store.note(&id).unwrap().unwrap().rev.unwrap();
    let room = Arc::new(Room {
        canvas: Mutex::new(drawing()),
        calls: Mutex::new(0),
        title: Mutex::new(None),
    });
    let router = with_room(&app, Some(room.clone()));
    let (status, answer) = call(
        &router,
        Method::PATCH,
        &format!("/{id}/canvas"),
        Some(&json!({ "ifRev": "1-stale", "ops": [{ "op": "moveNode", "id": "t1", "x": 5, "y": 5 }] })),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{answer}");
    assert_eq!(answer["rev"], rev.as_str());
    assert_eq!(answer["versions"]["t1"], 1);
    assert_eq!(room.canvas.lock().unwrap().node("t1").unwrap().x, 10.0);
    // The new title is part of the same change to the room: one call, and
    // the library is not written beside it.
    let (status, done) = call(
        &router,
        Method::PATCH,
        &format!("/{id}/canvas"),
        Some(&json!({ "ifRev": rev, "ops": [
            { "op": "moveNode", "id": "t1", "x": 5, "y": 5 },
            { "op": "rename", "title": "Renamed" },
        ] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{done}");
    assert_eq!(*room.calls.lock().unwrap(), 1);
    assert_eq!(room.title.lock().unwrap().as_deref(), Some("Renamed"));
    assert_eq!(app.store.note(&id).unwrap().unwrap().title, "Shared board");
}

/// A key of `access` for `app`, limited to `books`.
fn mint(app: &App, access: kanso_server::KeyAccess, books: Option<Vec<String>>) -> String {
    app.keys
        .mint(kanso_server::NewKey {
            name: "Scoped".into(),
            client: "other".into(),
            access,
            books,
        })
        .unwrap()
        .key
}

/// Calls a tool through `/mcp` signed with `key`.
async fn tool_as(app: &App, key: &str, name: &str, arguments: Value) -> Value {
    let message = json!({
        "jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": { "name": name, "arguments": arguments },
    });
    let request = axum::http::Request::builder()
        .method(Method::POST)
        .uri("/mcp")
        .header(axum::http::header::AUTHORIZATION, bearer(key))
        .header(axum::http::header::CONTENT_TYPE, "application/json")
        .body(axum::body::Body::from(message.to_string()))
        .unwrap();
    send(&app.router, request).await.json()["result"].clone()
}

fn notes_in(store: &Store, book: &str) -> usize {
    store.notes_in_books(&[book.to_owned()]).unwrap().len()
}

#[tokio::test]
async fn a_key_for_some_notebooks_must_say_where_a_canvas_goes() {
    let app = app();
    let book = make_book(&app.store);
    let key = mint(
        &app,
        kanso_server::KeyAccess::Write,
        Some(vec![book.clone()]),
    );
    let inbox = app
        .store
        .role_book(kanso_core::roles::Role::Inbox, Origin::Sync)
        .unwrap();
    let before = notes_in(&app.store, &inbox);
    let answer = tool_as(
        &app,
        &key,
        "kanso-create-canvas",
        json!({ "title": "Where?" }),
    )
    .await;
    assert_eq!(answer["isError"], true, "{answer}");
    assert!(text(&answer, 0).contains("bookId"), "{answer}");
    assert_eq!(notes_in(&app.store, &inbox), before);
    // In one of its notebooks, it is made.
    let made = tool_as(
        &app,
        &key,
        "kanso-create-canvas",
        json!({ "title": "Here", "bookId": book }),
    )
    .await;
    assert_ne!(made["isError"], true, "{made}");
    assert_eq!(notes_in(&app.store, &book), 1);
}

/// Lists `files` as attachments of note `id`.
fn attach(store: &Store, id: &str, files: &[&str]) {
    let files: Vec<kanso_core::model::NoteFile> = files
        .iter()
        .map(|file| kanso_core::model::NoteFile {
            file: (*file).to_owned(),
            v: 1,
            seal: "AAAAAAAAAAAAAAAAAAAAAA".into(),
            key: "wrapped".into(),
            extra: serde_json::Map::new(),
        })
        .collect();
    store.set_note_files(id, &files).unwrap();
}

fn picture(file: &str) -> Value {
    json!({ "ops": [{ "op": "addNode", "type": "file", "x": 0, "y": 0, "file": file }] })
}

#[tokio::test]
async fn a_card_points_only_at_what_the_key_could_already_read() {
    let app = app();
    let book = make_book(&app.store);
    // The board already shows two pictures.
    let mut board = drawing();
    for (card, file) in [("p1", "file:mine"), ("p2", "file:both")] {
        board.put_node(
            Node {
                id: card.into(),
                kind: "file".into(),
                x: 500.0,
                y: 500.0,
                width: 50.0,
                height: 50.0,
                file: Some(file.into()),
                ..Node::default()
            },
            1,
        );
    }
    let (id, _) = make_canvas(&app.store, &book, "Board", &board);
    // Another note's picture.
    let other = app
        .store
        .put(
            DocType::Note,
            json!({ "title": "Other", "bookId": book, "body": "![x](inkdrop://file:theirs)" }),
            Put::Upsert,
            Origin::Sync,
        )
        .unwrap()
        .id;
    // `file:both` is also held by a locked note, so it is sealed with it.
    let (vault, _) = make_canvas(&app.store, &book, "Vault", &Canvas::default());
    attach(&app.store, &vault, &["file:both"]);
    lock(&app.store, &vault);
    // The board's card shows it too, so the board holds it as well.
    let mut owners = app.store.file_owners("file:both").unwrap();
    owners.sort();
    let mut want = vec![vault.clone(), id.clone()];
    want.sort();
    assert_eq!(owners, want);
    assert_eq!(
        app.store.file_owners("file:theirs").unwrap(),
        vec![other.clone()]
    );
    let before = app.store.note(&id).unwrap().unwrap().rev;

    // Another note's attachment, and one sealed with a locked note: refused.
    for file in ["file:theirs", "file:both"] {
        let (status, answer) = patch(&app, &id, picture(file)).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{file}: {answer}");
    }
    // A card to a locked note is a card to a note that isn't there.
    let (status, answer) = patch(&app, &id, picture(&vault)).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{answer}");
    assert!(!answer.to_string().contains("Vault"));
    assert_eq!(app.store.note(&id).unwrap().unwrap().rev, before);
    // The canvas's own attachment, and a note the key sees, are fine.
    let (status, answer) = patch(&app, &id, picture("file:mine")).await;
    assert_eq!(status, StatusCode::OK, "{answer}");
    let (status, answer) = patch(&app, &id, picture(&other)).await;
    assert_eq!(status, StatusCode::OK, "{answer}");

    // A new canvas has no attachments: a picture card on it is refused.
    let made = tool(
        &app.router,
        "kanso-create-canvas",
        json!({ "title": "New", "bookId": book, "canvas": { "nodes": [
            { "id": "p", "type": "file", "file": "file:theirs", "x": 0, "y": 0, "width": 10, "height": 10 }
        ], "edges": [] } }),
    )
    .await;
    assert_eq!(made["isError"], true, "{made}");
    let made = tool(
        &app.router,
        "kanso-create-canvas",
        json!({ "title": "New", "bookId": book, "canvas": { "nodes": [
            { "id": "p", "type": "file", "file": vault, "x": 0, "y": 0, "width": 10, "height": 10 }
        ], "edges": [] } }),
    )
    .await;
    assert_eq!(made["isError"], true, "{made}");
}

#[tokio::test]
async fn read_and_suggest_keys_read_canvases_but_never_change_them() {
    let app = app();
    let book = make_book(&app.store);
    let (id, rev) = make_canvas(&app.store, &book, "Board", &drawing());
    for access in [
        kanso_server::KeyAccess::Read,
        kanso_server::KeyAccess::Suggest,
    ] {
        let key = mint(&app, access, None);
        let read = tool_as(&app, &key, "kanso-read-canvas", json!({ "noteId": id })).await;
        assert_ne!(read["isError"], true, "{read}");
        for (name, args) in [
            (
                "kanso-edit-canvas",
                json!({ "noteId": id, "ops": [{ "op": "rename", "title": "x" }] }),
            ),
            (
                "kanso-create-canvas",
                json!({ "title": "x", "bookId": book }),
            ),
        ] {
            let answer = tool_as(&app, &key, name, args).await;
            assert_eq!(answer["isError"], true, "{name} {access:?}: {answer}");
        }
        let request = axum::http::Request::builder()
            .method(Method::PATCH)
            .uri(format!("/{id}/canvas"))
            .header(axum::http::header::AUTHORIZATION, bearer(&key))
            .header(axum::http::header::CONTENT_TYPE, "application/json")
            .body(axum::body::Body::from(
                json!({ "ops": [{ "op": "rename", "title": "x" }] }).to_string(),
            ))
            .unwrap();
        assert_eq!(
            send(&app.router, request).await.status,
            StatusCode::FORBIDDEN
        );
    }
    assert_eq!(app.store.note(&id).unwrap().unwrap().rev.unwrap(), rev);
    assert_eq!(notes_in(&app.store, &book), 1);
}
