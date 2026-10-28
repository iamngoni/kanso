//! Example tests for every REST endpoint and error case.

mod common;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use common::*;
use kanso_core::DocType;
use kanso_core::model::{FILE_LOCK_FIELD, FileLock};
use kanso_server::{ServerConfig, basic_header};
use kanso_store::{Attachment, Origin, Put};
use serde_json::{Value, json};

async fn book(app: &App, name: &str) -> String {
    write(&app.router, "/books", json!({ "name": name }))
        .await
        .0
}

async fn tag(app: &App, name: &str) -> String {
    write(&app.router, "/tags", json!({ "name": name })).await.0
}

async fn note(app: &App, book_id: &str, title: &str, body: &str) -> (String, String) {
    write(
        &app.router,
        "/notes",
        json!({ "bookId": book_id, "title": title, "body": body }),
    )
    .await
}

fn ids(docs: &Value) -> Vec<String> {
    docs.as_array()
        .unwrap()
        .iter()
        .map(|d| d["_id"].as_str().unwrap().to_owned())
        .collect()
}

#[tokio::test]
async fn root_returns_app_meta() {
    let app = app();
    let (status, meta) = get(&app.router, "/").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(meta["name"], "Kanso");
    assert_eq!(meta["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(meta["compatibleWith"], "inkdrop-local-server");
}

#[tokio::test]
async fn every_route_requires_basic_auth() {
    let app = app();
    for (method, uri) in [
        (Method::GET, "/"),
        (Method::GET, "/notes"),
        (Method::POST, "/notes"),
        (Method::GET, "/note:abc"),
        (Method::DELETE, "/note:abc"),
        (Method::GET, "/note/abc"),
        (Method::GET, "/_changes"),
        (Method::POST, "/mcp"),
        (Method::GET, "/no/such/route"),
    ] {
        for auth in [
            None,
            Some(basic_header(USER, "wrong")),
            Some(basic_header("x", PASS)),
        ] {
            let mut request = Request::builder().method(method.clone()).uri(uri);
            if let Some(auth) = &auth {
                request = request.header(header::AUTHORIZATION, auth);
            }
            let response = send(&app.router, request.body(Body::from("{}")).unwrap()).await;
            assert_eq!(response.status, StatusCode::UNAUTHORIZED, "{method} {uri}");
            assert_eq!(
                response.headers[header::WWW_AUTHENTICATE],
                "Basic realm=\"Kanso\""
            );
            assert_eq!(
                response.json(),
                json!({ "error": "unauthorized", "reason": "Name or password is incorrect." })
            );
        }
    }
}

#[tokio::test]
async fn notebooks_are_created_read_listed_and_merged() {
    let app = app();
    let parent = book(&app, "Parent").await;
    let (child, child_rev) = write(
        &app.router,
        "/books",
        json!({ "name": "Child", "parentBookId": parent }),
    )
    .await;
    note(&app, &child, "In child", "").await;

    let (status, doc) = get(&app.router, &format!("/{child}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(doc["name"], "Child");
    assert_eq!(doc["parentBookId"], parent);
    assert_eq!(doc["count"], 1);

    let (_, books) = get(&app.router, "/books").await;
    assert_eq!(ids(&books), [child.clone(), parent.clone()]);
    let (_, limited) = get(&app.router, "/books?limit=1&skip=1").await;
    assert_eq!(ids(&limited), std::slice::from_ref(&parent));

    // Partial merge without a rev keeps the other fields.
    let (_, new_rev) = write(
        &app.router,
        &format!("/{child}"),
        json!({ "name": "Renamed" }),
    )
    .await;
    assert_ne!(new_rev, child_rev);
    let (_, doc) = get(&app.router, &format!("/{child}")).await;
    assert_eq!(doc["name"], "Renamed");
    assert_eq!(doc["parentBookId"], parent);

    // A stale rev conflicts; a missing notebook is not found.
    let (status, err) = post(
        &app.router,
        &format!("/{child}"),
        json!({ "_rev": child_rev, "name": "Again" }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(
        err,
        json!({ "error": "conflict", "reason": "Document update conflict." })
    );
    let (status, err) = post(&app.router, "/book:missing", json!({ "name": "x" })).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(err, json!({ "error": "not_found", "reason": "missing" }));

    // Moving to the root with null.
    write(
        &app.router,
        &format!("/{child}"),
        json!({ "parentBookId": null }),
    )
    .await;
    let (_, doc) = get(&app.router, &format!("/{child}")).await;
    assert_eq!(doc["parentBookId"], Value::Null);
}

#[tokio::test]
async fn note_file_ownership_is_hidden_and_preserved_on_api_writes() {
    let app = app();
    let book_id = book(&app, "B").await;
    let note_id = "note:owned";
    app.store
        .put(
            DocType::Note,
            json!({
                "_id":note_id, "title":"Owner", "body":"", "bookId":book_id,
                "kansoLock":{"v":1},
                "kansoFiles":[{"file":"file:sealed", "v":1, "seal":"AAAAAAAAAAAAAAAAAAAAAA", "key":"wrapped"}]
            }),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();

    let (status, served) = get(&app.router, "/note:owned").await;
    assert_eq!(status, StatusCode::OK);
    assert!(served.get("kansoFiles").is_none(), "{served}");
    let mut update = served;
    update["title"] = "Renamed".into();
    let (status, _) = post(&app.router, "/notes", update).await;
    assert_eq!(status, StatusCode::OK);
    let stored = app.store.get(note_id).unwrap().unwrap();
    assert_eq!(stored.fields["kansoFiles"][0]["file"], "file:sealed");
}

#[tokio::test]
async fn tags_are_created_listed_and_validated() {
    let app = app();
    let (id, _) = write(
        &app.router,
        "/tags",
        json!({ "name": "work", "color": "blue" }),
    )
    .await;
    let (status, doc) = get(&app.router, &format!("/{id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(doc["color"], "blue");
    assert_eq!(doc["count"], 0);
    let (_, tags) = get(&app.router, "/tags").await;
    assert_eq!(ids(&tags), std::slice::from_ref(&id));

    let (status, err) = post(
        &app.router,
        "/tags",
        json!({ "name": "x", "color": "magenta" }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(err["error"], "bad_request");
    assert_eq!(err["reason"], "The tag has an invalid color \"magenta\".");

    write(&app.router, &format!("/{id}"), json!({ "color": "red" })).await;
    let (_, doc) = get(&app.router, &format!("/{id}")).await;
    assert_eq!(
        (doc["name"].as_str(), doc["color"].as_str()),
        (Some("work"), Some("red"))
    );
}

#[tokio::test]
async fn notes_follow_the_rev_and_conflict_rules() {
    let app = app();
    let book_id = book(&app, "Inbox").await;
    let tag_id = tag(&app, "t").await;
    let (id, rev) = write(
        &app.router,
        "/notes",
        json!({ "bookId": book_id, "title": "Hello", "body": "- [x] a\n- [ ] b", "tags": [tag_id], "status": "active" }),
    )
    .await;
    assert!(id.starts_with("note:"));

    let (status, doc) = get(&app.router, &format!("/{id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(doc["_rev"], rev);
    assert_eq!(doc["title"], "Hello");
    assert_eq!(doc["tags"], json!([tag_id]));
    assert_eq!(doc["numOfTasks"], 2);
    assert_eq!(doc["numOfCheckedTasks"], 1);

    // ?rev= finds only the current revision.
    let (status, _) = get(&app.router, &format!("/{id}?rev={rev}")).await;
    assert_eq!(status, StatusCode::OK);
    let (status, err) = get(
        &app.router,
        &format!("/{id}?rev=1-0123456789abcdef0123456789abcdef"),
    )
    .await;
    assert_eq!(
        (status, err["error"].as_str()),
        (StatusCode::NOT_FOUND, Some("not_found"))
    );

    // Full replace with the current rev.
    let mut replaced = doc.clone();
    replaced["title"] = json!("Replaced");
    let (_, rev2) = write(&app.router, "/notes", replaced.clone()).await;
    assert_ne!(rev2, rev);
    // The same body again is now stale; without a rev it conflicts too.
    let (status, _) = post(&app.router, "/notes", replaced.clone()).await;
    assert_eq!(status, StatusCode::CONFLICT);
    replaced.as_object_mut().unwrap().remove("_rev");
    let (status, _) = post(&app.router, "/notes", replaced).await;
    assert_eq!(status, StatusCode::CONFLICT);

    // An unknown _id creates the note with that ID.
    let (created, _) = write(
        &app.router,
        "/notes",
        json!({ "_id": "note:custom-1", "bookId": book_id, "title": "Mine", "body": "" }),
    )
    .await;
    assert_eq!(created, "note:custom-1");

    // Merge into a note; a stale rev conflicts.
    write(
        &app.router,
        &format!("/{id}"),
        json!({ "status": "onHold" }),
    )
    .await;
    let (_, doc) = get(&app.router, &format!("/{id}")).await;
    assert_eq!(
        (doc["status"].as_str(), doc["title"].as_str()),
        (Some("onHold"), Some("Replaced"))
    );
    let (status, _) = post(
        &app.router,
        &format!("/{id}"),
        json!({ "_rev": rev, "title": "x" }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);

    // Store validation errors come back as 400 with the store's reason.
    let (status, err) = post(
        &app.router,
        "/notes",
        json!({ "_id": "note:bad", "bookId": "book:nope", "title": "x", "body": "" }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(
        err,
        json!({ "error": "bad_request", "reason": "The note 'note:bad' has an invalid bookId." })
    );
}

#[tokio::test]
async fn malformed_and_oversized_bodies_are_rejected() {
    let app = app();
    let bad_json = Request::builder()
        .method(Method::POST)
        .uri("/notes")
        .header(header::AUTHORIZATION, basic_header(USER, PASS))
        .body(Body::from("{not json"))
        .unwrap();
    let response = send(&app.router, bad_json).await;
    assert_eq!(response.status, StatusCode::BAD_REQUEST);
    assert_eq!(response.json()["error"], "bad_request");

    let (status, err) = post(&app.router, "/books", json!(["not", "an", "object"])).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(err["reason"], "Expected a JSON object.");
    let (status, err) = post(&app.router, "/book:abc", json!(42)).await;
    assert_eq!(
        (status, err["reason"].as_str()),
        (StatusCode::BAD_REQUEST, Some("Expected a JSON object."))
    );

    let huge = format!(
        "{{\"bookId\":\"trash\",\"title\":\"x\",\"body\":\"{}\"}}",
        "a".repeat(kanso_server::MAX_BODY_BYTES)
    );
    for uri in ["/notes", "/note:abc", "/mcp"] {
        let request = Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header(header::AUTHORIZATION, basic_header(USER, PASS))
            .body(Body::from(huge.clone()))
            .unwrap();
        let response = send(&app.router, request).await;
        assert_eq!(response.status, StatusCode::PAYLOAD_TOO_LARGE, "{uri}");
        assert_eq!(response.json()["error"], "too_large");
    }

    // Just under the limit is accepted (and then judged by the store).
    let fits = format!(
        "{{\"bookId\":\"trash\",\"title\":\"x\",\"body\":\"{}\"}}",
        "a".repeat(1000)
    );
    let request = Request::builder()
        .method(Method::POST)
        .uri("/notes")
        .header(header::AUTHORIZATION, basic_header(USER, PASS))
        .body(Body::from(fits))
        .unwrap();
    assert_eq!(send(&app.router, request).await.status, StatusCode::OK);
}

#[tokio::test]
async fn note_lists_honour_keyword_limit_skip_and_sort() {
    let app = app();
    let alpha = book(&app, "Alpha").await;
    let beta = book(&app, "Beta Book").await;
    let mut created = Vec::new();
    for (i, title) in ["banana", "Apple", "cherry"].iter().enumerate() {
        let book_id = if i == 1 { &beta } else { &alpha };
        created.push(
            note(&app, book_id, title, &format!("body {i} shared"))
                .await
                .0,
        );
        tokio::time::sleep(std::time::Duration::from_millis(3)).await;
    }
    let (trashed, _) = note(&app, "trash", "trashed", "shared").await;

    // Default: updatedAt descending, trash excluded.
    let (status, notes) = get(&app.router, "/notes").await;
    assert_eq!(status, StatusCode::OK);
    let newest_first: Vec<String> = created.iter().rev().cloned().collect();
    assert_eq!(ids(&notes), newest_first);
    assert_eq!(notes[0]["body"], "body 2 shared");

    let (_, notes) = get(&app.router, "/notes?limit=2&skip=1").await;
    assert_eq!(ids(&notes), newest_first[1..3]);
    let (_, notes) = get(&app.router, "/notes?sort=title&descending=false").await;
    assert_eq!(
        ids(&notes),
        [created[1].clone(), created[0].clone(), created[2].clone()]
    );
    let (_, notes) = get(&app.router, "/notes?sort=createdAt&descending=false").await;
    assert_eq!(ids(&notes), created);
    // Unknown sorts fall back to updatedAt.
    for sort in ["rank", "tagCount"] {
        let (status, notes) = get(&app.router, &format!("/notes?sort={sort}")).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(ids(&notes), newest_first);
    }

    let (_, notes) = get(&app.router, "/notes?keyword=book%3A%22Beta%20Book%22").await;
    assert_eq!(ids(&notes), [created[1].clone()]);
    let (_, notes) = get(&app.router, "/notes?keyword=SHARED%20-title%3Aapple").await;
    assert_eq!(ids(&notes), [created[2].clone(), created[0].clone()]);
    let (_, notes) = get(&app.router, "/notes?keyword=bookId%3Atrash").await;
    assert_eq!(ids(&notes), [trashed]);

    for bad in ["limit=-1", "limit=ten", "skip=x", "descending=maybe"] {
        let (status, err) = get(&app.router, &format!("/notes?{bad}")).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{bad}");
        assert_eq!(err["error"], "bad_request");
    }
}

#[tokio::test]
async fn note_lists_default_to_fifty() {
    let app = app();
    for i in 0..55 {
        note(&app, "trash", &format!("n{i}"), "").await;
    }
    let (_, notes) = get(&app.router, "/notes?keyword=bookId:trash").await;
    assert_eq!(notes.as_array().unwrap().len(), 50);
    let (_, notes) = get(&app.router, "/notes?keyword=bookId:trash&limit=100").await;
    assert_eq!(notes.as_array().unwrap().len(), 55);
}

#[tokio::test]
async fn files_carry_attachments() {
    let app = app();
    let data = STANDARD.encode(PNG);
    let (id, _) = write(
        &app.router,
        "/files",
        json!({
            "name": "pixel.png", "contentType": "image/png", "contentLength": PNG.len(), "publicIn": [],
            "_attachments": { "index": { "content_type": "image/png", "data": data } }
        }),
    )
    .await;
    let (status, stub) = get(&app.router, &format!("/{id}")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(stub["contentLength"], PNG.len());
    assert_eq!(stub["_attachments"]["index"]["stub"], true);
    assert!(stub["_attachments"]["index"].get("data").is_none());

    let (_, full) = get(&app.router, &format!("/{id}?attachments=true")).await;
    assert_eq!(full["_attachments"]["index"]["data"], data);
    assert_eq!(full["_attachments"]["index"]["content_type"], "image/png");

    let (_, files) = get(&app.router, "/files").await;
    assert_eq!(ids(&files), [id]);

    let (status, err) = post(
        &app.router,
        "/files",
        json!({ "name": "x.bmp", "contentType": "image/bmp",
                "_attachments": { "index": { "content_type": "image/bmp", "data": data } } }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(err["reason"], "Unsupported file type 'image/bmp'.");
}

#[tokio::test]
async fn a_sealed_file_keeps_its_marker_from_the_local_api() {
    let app = app();
    let data = STANDARD.encode(PNG);
    let (id, _) = write(
        &app.router,
        "/files",
        json!({
            "name": "pixel.png", "contentType": "image/png", "publicIn": [],
            "_attachments": { "index": { "content_type": "image/png", "data": data } }
        }),
    )
    .await;
    // The app seals the file: the store writes the marker and the
    // container, and nothing else may.
    let mut file = app.store.file_meta(&id).unwrap().unwrap();
    FileLock::store(
        Some(&FileLock::sealed("AAAAAAAAAAAAAAAAAAAAAA".into())),
        &mut file.extra,
    );
    app.store
        .put_sealed_file(file, Attachment::new(vec![0x01, 0x01, 0x00, 0x03, 0, 0, 0]))
        .unwrap();

    // Served output never carries the marker, in either form.
    let (_, stub) = get(&app.router, &format!("/{id}")).await;
    assert!(stub.get(FILE_LOCK_FIELD).is_none(), "{stub}");
    let (_, files) = get(&app.router, "/files").await;
    assert!(files[0].get(FILE_LOCK_FIELD).is_none(), "{}", files[0]);
    let (_, asked) = get(&app.router, &format!("/{id}?attachments=true")).await;
    assert_eq!(asked["locked"], true, "{asked}");
    assert_eq!(asked["_attachments"]["index"]["stub"], true, "{asked}");
    assert!(
        asked["_attachments"]["index"].get("data").is_none(),
        "{asked}"
    );

    // A client's replace and merge keep the marker the store already has,
    // and a client's new bytes are refused.
    let (_, mut doc) = get(&app.router, &format!("/{id}")).await;
    doc["name"] = "renamed.png".into();
    write(&app.router, "/files", doc).await;
    let sealed = |store: &kanso_store::Store| {
        store
            .file_meta(&id)
            .unwrap()
            .unwrap()
            .extra
            .get(FILE_LOCK_FIELD)
            .cloned()
    };
    assert_eq!(
        app.store.file_meta(&id).unwrap().unwrap().name,
        "renamed.png"
    );
    assert!(sealed(&app.store).is_some());

    let (status, _) = post(
        &app.router,
        &format!("/{id}"),
        json!({ FILE_LOCK_FIELD: null }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(sealed(&app.store).is_some());

    let replacement = STANDARD.encode(b"plaintext");
    let (status, _) = post(
        &app.router,
        &format!("/{id}"),
        json!({ "_attachments": { "index": { "content_type": "image/png", "data": replacement } } }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(sealed(&app.store).is_some());
    assert_eq!(
        app.store.stored_attachment(&id).unwrap().unwrap().data,
        vec![0x01, 0x01, 0x00, 0x03, 0, 0, 0]
    );
}

#[tokio::test]
async fn deletes_follow_the_api_semantics() {
    let app = app();
    let book_id = book(&app, "B").await;
    let tag_id = tag(&app, "T").await;
    let (note_id, _) = write(
        &app.router,
        "/notes",
        json!({ "bookId": book_id, "title": "x", "body": "", "tags": [tag_id] }),
    )
    .await;
    let (other, _) = note(&app, &book_id, "y", "").await;

    let (status, res) = call(&app.router, Method::DELETE, &format!("/{other}"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(res["ok"], true);
    assert_eq!(res["id"], other);
    assert!(res["rev"].as_str().unwrap().starts_with("2-"));
    let (status, _) = get(&app.router, &format!("/{other}")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = call(&app.router, Method::DELETE, &format!("/{other}"), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Deleting a tag removes it from notes; deleting a notebook trashes its notes.
    call(&app.router, Method::DELETE, &format!("/{tag_id}"), None).await;
    let (_, doc) = get(&app.router, &format!("/{note_id}")).await;
    assert_eq!(doc["tags"], json!([]));
    call(&app.router, Method::DELETE, &format!("/{book_id}"), None).await;
    let (_, doc) = get(&app.router, &format!("/{note_id}")).await;
    assert_eq!(doc["bookId"], "trash");

    let (status, _) = call(&app.router, Method::DELETE, "/favicon.ico", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn slash_aliases_match_colon_ids() {
    let app = app();
    let book_id = book(&app, "B").await;
    let (note_id, _) = note(&app, &book_id, "x", "y").await;
    let bare = note_id.strip_prefix("note:").unwrap();

    let colon = send(
        &app.router,
        request(Method::GET, &format!("/{note_id}"), None),
    )
    .await;
    let slash = send(
        &app.router,
        request(Method::GET, &format!("/note/{bare}"), None),
    )
    .await;
    assert_eq!(colon.status, StatusCode::OK);
    assert_eq!(colon.bytes, slash.bytes);

    let book_bare = book_id.strip_prefix("book:").unwrap();
    write(
        &app.router,
        &format!("/book/{book_bare}"),
        json!({ "name": "Renamed" }),
    )
    .await;
    let (_, doc) = get(&app.router, &format!("/{book_id}")).await;
    assert_eq!(doc["name"], "Renamed");

    let (status, _) = call(&app.router, Method::DELETE, &format!("/note/{bare}"), None).await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = get(&app.router, &format!("/note/{bare}")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = get(&app.router, "/widget/abc").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn changes_feed_lists_writes_in_order() {
    let app = app();
    let book_id = book(&app, "B").await;
    let (note_id, _) = note(&app, &book_id, "x", "").await;
    call(&app.router, Method::DELETE, &format!("/{note_id}"), None).await;

    let (status, feed) = get(&app.router, "/_changes").await;
    assert_eq!(status, StatusCode::OK);
    let results = feed["results"].as_array().unwrap();
    assert_eq!(results.len(), 2, "{feed}");
    assert_eq!(results[0]["id"], book_id);
    assert_eq!(results[1]["id"], note_id);
    assert_eq!(results[1]["deleted"], true);
    assert!(results[0].get("deleted").is_none());
    assert!(results[0].get("doc").is_none());
    assert_eq!(feed["last_seq"], results[1]["seq"]);
    let rev = results[0]["changes"][0]["rev"].as_str().unwrap();
    assert!(rev.starts_with("1-"));

    let (_, feed) = get(&app.router, "/_changes?include_docs=true&limit=1").await;
    assert_eq!(feed["results"].as_array().unwrap().len(), 1);
    assert_eq!(feed["results"][0]["doc"]["name"], "B");
    assert_eq!(feed["last_seq"], feed["results"][0]["seq"]);

    let (_, feed) = get(&app.router, "/_changes?include_docs=true&descending=true").await;
    assert_eq!(feed["results"][0]["id"], note_id);
    assert_eq!(feed["results"][0]["doc"]["_deleted"], true);

    let last = results[1]["seq"].as_i64().unwrap();
    for since in [last.to_string(), "now".to_owned()] {
        let (_, feed) = get(&app.router, &format!("/_changes?since={since}")).await;
        assert_eq!(feed, json!({ "results": [], "last_seq": last }));
    }
    let (status, _) = get(&app.router, "/_changes?since=abc").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn unknown_routes_and_methods_answer_json() {
    let app = app();
    let (status, err) = get(&app.router, "/nope:abc").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(err, json!({ "error": "not_found", "reason": "missing" }));
    let (status, err) = get(&app.router, "/a/b/c").await;
    assert_eq!(
        (status, err["error"].as_str()),
        (StatusCode::NOT_FOUND, Some("not_found"))
    );
    let (status, err) = call(&app.router, Method::PUT, "/notes", Some(&json!({}))).await;
    assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(err["error"], "method_not_allowed");
}

#[test]
fn config_debug_redacts_the_password() {
    let config = ServerConfig {
        bind: SocketAddr::from(([127, 0, 0, 1], 1)),
        username: "user".into(),
        password: "hunter2-secret".into(),
    };
    let shown = format!("{config:?}");
    assert!(!shown.contains("hunter2-secret"), "{shown}");
    assert!(shown.contains("user"));
}

fn test_store() -> (tempfile::TempDir, Arc<kanso_store::Store>) {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(kanso_store::Store::open(dir.path().join("kanso.db")).unwrap());
    (dir, store)
}

#[test]
fn spawn_serves_on_its_own_runtime_and_stops() {
    let (_dir, store) = test_store();
    let handle = kanso_server::spawn(store, config());
    let addr = handle.local_addr().expect("bound");
    assert_ne!(addr.port(), 0);
    assert!(handle.is_running());
    let response = reqwest::blocking::Client::new()
        .get(format!("http://{addr}/"))
        .header(header::AUTHORIZATION.as_str(), basic_header(USER, PASS))
        .send()
        .unwrap();
    assert_eq!(response.status().as_u16(), 200);
    let meta: Value = response.json().unwrap();
    assert_eq!(meta["name"], "Kanso");
    handle.stop().unwrap();
    assert!(reqwest::blocking::get(format!("http://{addr}/")).is_err());
}

#[test]
fn a_stalled_client_cannot_block_stop() {
    use std::io::Write as _;
    let (_dir, store) = test_store();
    let handle = kanso_server::spawn(store, config());
    let addr = handle.local_addr().unwrap();
    // Half a request: headers never finish, so the request is in flight.
    let mut stalled = std::net::TcpStream::connect(addr).unwrap();
    write!(
        stalled,
        "POST /notes HTTP/1.1\r\nHost: x\r\nAuthorization: {}\r\nContent-Length: 100\r\n\r\n{{",
        basic_header(USER, PASS)
    )
    .unwrap();
    std::thread::sleep(std::time::Duration::from_millis(100));
    let started = std::time::Instant::now();
    handle.stop().unwrap();
    assert!(
        started.elapsed() < std::time::Duration::from_secs(8),
        "{:?}",
        started.elapsed()
    );
}

#[test]
fn a_taken_port_is_reported_as_addr_in_use() {
    let (_dir, store) = test_store();
    let taken = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let config = ServerConfig {
        bind: taken.local_addr().unwrap(),
        ..config()
    };
    let handle = kanso_server::spawn(store.clone(), config.clone());
    assert!(handle.is_addr_in_use(), "{handle:?}");
    assert!(handle.local_addr().is_none());
    assert!(!handle.is_running());
    handle.stop().unwrap();

    let err = common::runtime()
        .block_on(kanso_server::serve(store, config, async {}))
        .unwrap_err();
    assert!(kanso_server::is_addr_in_use(&err), "{err:?}");
}

#[tokio::test]
async fn serve_runs_until_shutdown() {
    let (_dir, store) = test_store();
    let probe = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = probe.local_addr().unwrap();
    drop(probe);
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    let server = tokio::spawn(kanso_server::serve(
        store,
        ServerConfig {
            bind: addr,
            ..config()
        },
        async move {
            let _ = rx.await;
        },
    ));
    let client = reqwest::Client::new();
    let mut status = None;
    for _ in 0..50 {
        match client
            .get(format!("http://{addr}/notes"))
            .header(header::AUTHORIZATION.as_str(), basic_header(USER, PASS))
            .send()
            .await
        {
            Ok(response) => {
                status = Some(response.status().as_u16());
                break;
            }
            Err(_) => tokio::time::sleep(std::time::Duration::from_millis(20)).await,
        }
    }
    assert_eq!(status, Some(200));
    tx.send(()).unwrap();
    server.await.unwrap().unwrap();
}

/// A note's published link as only the app's Publish sheet writes it.
fn published() -> Value {
    json!({ "shareId": "q3T9xW2LmN8vPz4RkY7aBw", "key": "app-key", "publishedAt": 1, "publishedRev": "1-a" })
}

/// Writes `note` as the app does (`Origin::Ui`), carrying `kansoPublish`.
fn app_note(app: &App, mut note: Value) -> (String, String) {
    note["kansoPublish"] = published();
    let res = app
        .store
        .put(
            kanso_core::DocType::Note,
            note,
            kanso_store::Put::Upsert,
            kanso_store::Origin::Ui,
        )
        .unwrap();
    (res.id, res.rev)
}

#[tokio::test]
async fn the_local_api_never_sets_a_notes_published_link() {
    let app = app();
    let planted = json!({ "shareId": "AAAAAAAAAAAAAAAAAAAAAA", "key": "attacker-key", "publishedAt": 2, "publishedRev": "1-b" });

    // Created through the API: whatever kansoPublish it's given is dropped, and a merge can't add one.
    let (fresh, _) = write(
        &app.router,
        "/notes",
        json!({ "bookId": "trash", "title": "t", "body": "b", "kansoPublish": planted }),
    )
    .await;
    let (_, doc) = get(&app.router, &format!("/{fresh}")).await;
    assert!(doc.get("kansoPublish").is_none(), "{doc}");
    write(
        &app.router,
        &format!("/{fresh}"),
        json!({ "kansoPublish": planted }),
    )
    .await;
    let (_, doc) = get(&app.router, &format!("/{fresh}")).await;
    assert!(doc.get("kansoPublish").is_none(), "{doc}");

    // A published note keeps the app's value through replace and merge, whatever is sent.
    let (id, rev) = app_note(
        &app,
        json!({ "bookId": "trash", "title": "t", "body": "b" }),
    );
    let (_, mut doc) = get(&app.router, &format!("/{id}")).await;
    assert_eq!(doc["_rev"], rev.as_str());
    doc["kansoPublish"] = planted.clone();
    doc["title"] = "replaced".into();
    write(&app.router, "/notes", doc).await;
    let (_, mut doc) = get(&app.router, &format!("/{id}")).await;
    assert_eq!(doc["title"], "replaced");
    assert_eq!(doc["kansoPublish"], published());
    // Leaving it out of a replace doesn't remove it either.
    doc.as_object_mut().unwrap().remove("kansoPublish");
    doc["title"] = "again".into();
    write(&app.router, "/notes", doc).await;
    for body in [
        json!({ "kansoPublish": planted }),
        json!({ "kansoPublish": null, "title": "merged" }),
    ] {
        write(&app.router, &format!("/{id}"), body).await;
        let (_, doc) = get(&app.router, &format!("/{id}")).await;
        assert_eq!(doc["kansoPublish"], published(), "{doc}");
    }
    let (_, doc) = get(&app.router, &format!("/{id}")).await;
    assert_eq!(doc["title"], "merged");
}

#[tokio::test]
async fn the_local_api_never_sets_a_notebooks_published_link() {
    let app = app();
    let planted = json!({ "shareId": "AAAAAAAAAAAAAAAAAAAAAA", "key": "attacker-key", "publishedAt": 2, "publishedRev": "" });
    // Made through the API: a link it is given is dropped, and a merge can't add one.
    let (id, _) = write(
        &app.router,
        "/books",
        json!({ "name": "Site", "kansoPublish": planted }),
    )
    .await;
    let (_, doc) = get(&app.router, &format!("/{id}")).await;
    assert!(doc.get("kansoPublish").is_none(), "{doc}");
    write(
        &app.router,
        &format!("/{id}"),
        json!({ "kansoPublish": planted }),
    )
    .await;
    let (_, doc) = get(&app.router, &format!("/{id}")).await;
    assert!(doc.get("kansoPublish").is_none(), "{doc}");
    // Published by the app: renames through the API keep the app's value.
    app.store
        .put(
            kanso_core::DocType::Book,
            json!({ "_id": id, "kansoPublish": published() }),
            kanso_store::Put::Merge,
            kanso_store::Origin::Ui,
        )
        .unwrap();
    for body in [
        json!({ "kansoPublish": planted, "name": "Renamed" }),
        json!({ "kansoPublish": null }),
    ] {
        write(&app.router, &format!("/{id}"), body).await;
        let (_, doc) = get(&app.router, &format!("/{id}")).await;
        assert_eq!(doc["kansoPublish"], published(), "{doc}");
    }
    let (_, doc) = get(&app.router, &format!("/{id}")).await;
    assert_eq!(doc["name"], "Renamed");
}

#[tokio::test]
async fn a_locked_note_keeps_its_sealed_text_from_the_local_api() {
    let app = app();
    let sealed = json!({ "v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "sealed" });
    let (id, _) = app_note(&app, json!({ "bookId": "trash", "title": "t", "body": "" }));
    let note = app.store.get_api(&id, None, false).unwrap();
    let mut locked = note.clone();
    locked["kansoLock"] = sealed.clone();
    app.store.lock_note(locked).unwrap();

    // Read with an empty body and without the sealed text, which would let
    // a client guess the password offline; a replace keeps the lock.
    let sealed_kept = |store: &kanso_store::Store| {
        store
            .note(&id)
            .unwrap()
            .unwrap()
            .extra
            .get("kansoLock")
            .cloned()
    };
    let (_, mut doc) = get(&app.router, &format!("/{id}")).await;
    assert_eq!(doc["body"], "");
    assert!(doc.get("kansoLock").is_none(), "{doc}");
    doc["title"] = "retitled".into();
    write(&app.router, "/notes", doc).await;
    let (_, doc) = get(&app.router, &format!("/{id}")).await;
    assert_eq!(doc["title"], "retitled");
    assert!(doc.get("kansoLock").is_none());
    assert_eq!(sealed_kept(&app.store), Some(sealed.clone()));

    // Text can't go beside the lock, the lock can't be taken off, and the
    // note can't be deleted outright.
    for body in [
        json!({ "body": "plain" }),
        json!({ "kansoLock": null, "body": "plain" }),
    ] {
        let (status, _) = post(&app.router, &format!("/{id}"), body).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }
    let (status, _) = call(&app.router, Method::DELETE, &format!("/{id}"), None).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (_, doc) = get(&app.router, &format!("/{id}")).await;
    assert_eq!(doc["body"], "");
    assert_eq!(sealed_kept(&app.store), Some(sealed));
    // Nor does the changes feed carry it.
    let (_, feed) = get(&app.router, "/_changes?include_docs=true").await;
    assert!(feed.to_string().contains(&id), "{feed}");
    assert!(!feed.to_string().contains("c2FsdA"), "{feed}");
}

#[tokio::test]
async fn a_locked_notebook_keeps_its_notes_text_and_its_lock_from_the_local_api_and_mcp() {
    let app = app();
    let lock = json!({ "v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "notebook" });
    let (book, _) = write(&app.router, "/books", json!({ "name": "Private" })).await;
    let (inner, _) = write(
        &app.router,
        "/books",
        json!({ "name": "Deeper", "parentBookId": book }),
    )
    .await;
    let (id, _) = write(
        &app.router,
        "/notes",
        json!({ "bookId": inner, "title": "Plans", "body": "zebracanary4471" }),
    )
    .await;
    // Locked on another device, before this note was sealed here.
    let mut locked = app.store.get_api(&book, None, false).unwrap();
    locked.as_object_mut().unwrap().remove("count");
    locked["kansoLock"] = lock.clone();
    app.store
        .put(
            kanso_core::DocType::Book,
            locked,
            kanso_store::Put::Upsert,
            kanso_store::Origin::Sync,
        )
        .unwrap();

    // The note reads with an empty body everywhere, MCP included.
    let (_, doc) = get(&app.router, &format!("/{id}")).await;
    assert_eq!(doc["body"], "");
    assert_eq!(doc["title"], "Plans");
    for uri in [
        "/notes?keyword=zebracanary4471".to_owned(),
        "/notes".to_owned(),
        "/_changes?include_docs=true".to_owned(),
    ] {
        let (_, found) = get(&app.router, &uri).await;
        assert!(
            !found.to_string().contains("zebracanary4471"),
            "{uri}: {found}"
        );
    }
    for (name, arguments) in [
        ("read-note", json!({ "noteId": id })),
        ("search-notes", json!({ "keyword": "Plans" })),
        ("list-notes", json!({ "bookId": inner })),
    ] {
        let answer = tool(&app.router, name, arguments).await;
        assert!(answer.to_string().contains("Plans"), "{name}: {answer}");
        assert!(
            !answer.to_string().contains("zebracanary4471"),
            "{name}: {answer}"
        );
    }
    // Nor is its text changed, emptied, moved out or deleted from here.
    for (uri, body) in [
        (format!("/{id}"), json!({ "body": "replaced" })),
        (format!("/{id}"), json!({ "body": "" })),
        (format!("/{id}"), json!({ "bookId": "trash" })),
        (
            "/notes".to_owned(),
            json!({ "bookId": book, "title": "New", "body": "plain" }),
        ),
    ] {
        let (status, _) = post(&app.router, &uri, body).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{uri}");
    }
    let (status, _) = call(&app.router, Method::DELETE, &format!("/{id}"), None).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // The notebook's lock never goes out, can't be taken off or planted,
    // and the notebook can't be deleted from here.
    for uri in [format!("/{book}"), "/books".to_owned()] {
        let (_, doc) = get(&app.router, &uri).await;
        assert!(!doc.to_string().contains("c2FsdA"), "{uri}: {doc}");
    }
    write(
        &app.router,
        &format!("/{book}"),
        json!({ "kansoLock": null, "name": "Renamed" }),
    )
    .await;
    assert_eq!(
        app.store
            .get(&book)
            .unwrap()
            .unwrap()
            .fields
            .get("kansoLock"),
        Some(&lock)
    );
    write(
        &app.router,
        &format!("/{inner}"),
        json!({ "kansoLock": { "v": 1, "salt": "planted", "m": 64, "t": 1, "p": 1, "data": "x" } }),
    )
    .await;
    assert!(
        app.store
            .get(&inner)
            .unwrap()
            .unwrap()
            .fields
            .get("kansoLock")
            .is_none()
    );
    for target in [&book, &inner] {
        let (status, _) = call(&app.router, Method::DELETE, &format!("/{target}"), None).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }
    assert!(
        app.store
            .note(&id)
            .unwrap()
            .unwrap()
            .body
            .contains("zebracanary4471")
    );
}
