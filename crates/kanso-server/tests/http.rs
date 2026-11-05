//! Example tests for every REST endpoint and error case.

mod common;

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use common::*;
use kanso_core::DocType;
use kanso_core::model::{FILE_LOCK_FIELD, FileLock};
use kanso_server::ServerConfig;
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

/// A document straight from the library, as the app holds it.
fn raw(app: &App, id: &str) -> Value {
    app.store.get_api(id, None, false).unwrap()
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
async fn every_route_requires_a_key() {
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
            Some(bearer("kcl_BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB")),
            Some(bearer("not-a-key")),
            Some(basic_with(
                "x",
                "kcl_BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB",
            )),
            Some(basic_with("kanso-test", "pa:ss word")),
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
                json!({
                    "error": "unauthorized",
                    "reason": "Kanso needs an API key. Create one in Settings \u{203a} Local API \u{203a} API keys."
                })
            );
        }
    }
}

fn basic_with(user: &str, password: &str) -> String {
    format!("Basic {}", STANDARD.encode(format!("{user}:{password}")))
}

#[tokio::test]
async fn the_key_works_as_a_bearer_token_or_as_a_basic_password() {
    let app = app();
    for header_value in [
        common::auth(),
        basic_with("anything", KEY),
        basic_with("", KEY),
        basic_with("kanso-test", KEY),
    ] {
        let request = Request::builder()
            .uri("/")
            .header(header::AUTHORIZATION, header_value)
            .body(Body::empty())
            .unwrap();
        assert_eq!(send(&app.router, request).await.status, StatusCode::OK);
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

    // A locked note does not exist for a key: it is not served, and a
    // write to it is refused as not found.
    let (status, _) = get(&app.router, "/note:owned").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = post(
        &app.router,
        "/notes",
        json!({ "_id": note_id, "title": "Renamed", "body": "", "bookId": book_id }),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let stored = app.store.get(note_id).unwrap().unwrap();
    assert_eq!(stored.fields["title"], "Owner");
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
        .header(header::AUTHORIZATION, common::auth())
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
            .header(header::AUTHORIZATION, common::auth())
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
        .header(header::AUTHORIZATION, common::auth())
        .body(Body::from(fits))
        .unwrap();
    assert_eq!(send(&app.router, request).await.status, StatusCode::OK);
}

/// A 50 MiB PDF in base64 is over the 16 MiB every other route takes: the
/// attachment routes alone take up to `MAX_ATTACHMENT_BODY_BYTES`, and the
/// store then holds the file to its own limit.
// no-prop-test: fixed sizes either side of the two body limits.
#[tokio::test]
async fn attachment_uploads_take_a_fifty_mib_pdf() {
    let app = app();
    let mut pdf = b"%PDF-1.7\n".to_vec();
    pdf.resize(kanso_core::model::MAX_PDF_SIZE, b' ');
    let file = |pdf: &[u8]| {
        json!({
            "name": "big.pdf", "contentType": "application/pdf", "publicIn": [],
            "_attachments": { "index": { "content_type": "application/pdf", "data": STANDARD.encode(pdf) } }
        })
        .to_string()
    };
    let upload = |uri: &str, body: String| {
        Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header(header::AUTHORIZATION, common::auth())
            .body(Body::from(body))
            .unwrap()
    };
    let body = file(&pdf);
    assert!(body.len() > kanso_server::MAX_BODY_BYTES);
    let response = send(&app.router, upload("/files", body.clone())).await;
    assert_eq!(response.status, StatusCode::OK, "{:?}", response.json());
    let id = response.json()["id"].as_str().unwrap().to_owned();
    for uri in [
        format!("/{id}"),
        format!("/file/{}", &id[5..]),
        format!("/file%3A{}", &id[5..]),
    ] {
        let response = send(&app.router, upload(&uri, body.clone())).await;
        assert_ne!(response.status, StatusCode::PAYLOAD_TOO_LARGE, "{uri}");
    }

    // The same bytes to a note route are still too large.
    let response = send(&app.router, upload("/notes", body)).await;
    assert_eq!(response.status, StatusCode::PAYLOAD_TOO_LARGE);

    // One byte over the PDF limit: the store refuses it in words.
    pdf.push(b' ');
    let response = send(&app.router, upload("/files", file(&pdf))).await;
    assert_eq!(
        response.status,
        StatusCode::BAD_REQUEST,
        "{:?}",
        response.json()
    );

    // Over the attachment body limit: refused before the store sees it.
    let huge = "a".repeat(kanso_server::MAX_ATTACHMENT_BODY_BYTES + 1);
    let response = send(&app.router, upload("/files", huge)).await;
    assert_eq!(response.status, StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(response.json()["error"], "too_large");
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
    // The trash is not visible to a key.
    let (_, notes) = get(&app.router, "/notes?keyword=bookId%3Atrash").await;
    assert!(ids(&notes).is_empty(), "{trashed}");

    for bad in ["limit=-1", "limit=ten", "skip=x", "descending=maybe"] {
        let (status, err) = get(&app.router, &format!("/notes?{bad}")).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{bad}");
        assert_eq!(err["error"], "bad_request");
    }
}

#[tokio::test]
async fn note_lists_default_to_fifty() {
    let app = app();
    let book_id = book(&app, "B").await;
    for i in 0..55 {
        note(&app, &book_id, &format!("n{i}"), "").await;
    }
    let (_, notes) = get(&app.router, "/notes").await;
    assert_eq!(notes.as_array().unwrap().len(), 50);
    let (_, notes) = get(&app.router, "/notes?limit=100").await;
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
    let mut lock = FileLock::sealed("AAAAAAAAAAAAAAAAAAAAAA".into());
    lock.meta = true;
    FileLock::store(Some(&lock), &mut file.extra);
    app.store
        .put_sealed_file(file, Attachment::new(vec![0x01, 0x01, 0x00, 0x03, 0, 0, 0]))
        .unwrap();
    let raw = app.store.get(&id).unwrap().unwrap();
    assert_eq!(raw.fields["name"], "Locked attachment");
    assert_eq!(raw.fields["contentType"], "application/octet-stream");
    assert_eq!(raw.fields["publicIn"], json!([]));
    assert!(
        !serde_json::to_string(&raw.fields)
            .unwrap()
            .contains("pixel.png")
    );

    // Served output never carries the marker, in either form.
    let (_, stub) = get(&app.router, &format!("/{id}")).await;
    assert!(stub.get(FILE_LOCK_FIELD).is_none(), "{stub}");
    assert_eq!(stub["name"], "Locked attachment", "{stub}");
    assert_eq!(stub["contentType"], "application/octet-stream", "{stub}");
    assert_eq!(stub["publicIn"], json!([]), "{stub}");
    let (_, files) = get(&app.router, "/files").await;
    assert!(files[0].get(FILE_LOCK_FIELD).is_none(), "{}", files[0]);
    assert_eq!(files[0]["name"], "Locked attachment", "{}", files[0]);
    assert_eq!(
        files[0]["contentType"], "application/octet-stream",
        "{}",
        files[0]
    );
    assert_eq!(
        files[0]["_attachments"]["index"]["stub"], true,
        "{}",
        files[0]
    );
    assert!(files[0]["_attachments"]["index"].get("data").is_none());
    let (_, asked) = get(&app.router, &format!("/{id}?attachments=true")).await;
    assert_eq!(asked["locked"], true, "{asked}");
    assert_eq!(asked["name"], "Locked attachment", "{asked}");
    assert_eq!(asked["contentType"], "application/octet-stream", "{asked}");
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
        "Locked attachment"
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
    let doc = raw(&app, &note_id);
    assert_eq!(doc["tags"], json!([]));
    call(&app.router, Method::DELETE, &format!("/{book_id}"), None).await;
    let doc = raw(&app, &note_id);
    assert_eq!(doc["bookId"], "trash");
    // A trashed note is not visible to a key.
    let (status, _) = get(&app.router, &format!("/{note_id}")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

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

fn test_store() -> (tempfile::TempDir, Arc<kanso_store::Store>) {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(kanso_store::Store::open(dir.path().join("kanso.db")).unwrap());
    (dir, store)
}

#[test]
fn spawn_serves_on_its_own_runtime_and_stops() {
    let (_dir, store) = test_store();
    let handle = kanso_server::spawn(store.clone(), config(), keys_for(&store));
    let addr = handle.local_addr().expect("bound");
    assert_ne!(addr.port(), 0);
    assert!(handle.is_running());
    let response = reqwest::blocking::Client::new()
        .get(format!("http://{addr}/"))
        .header(header::AUTHORIZATION.as_str(), common::auth())
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
    let handle = kanso_server::spawn(store.clone(), config(), keys_for(&store));
    let addr = handle.local_addr().unwrap();
    // Half a request: headers never finish, so the request is in flight.
    let mut stalled = std::net::TcpStream::connect(addr).unwrap();
    write!(
        stalled,
        "POST /notes HTTP/1.1\r\nHost: x\r\nAuthorization: {}\r\nContent-Length: 100\r\n\r\n{{",
        common::auth()
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
    };
    let handle = kanso_server::spawn(store.clone(), config.clone(), keys_for(&store));
    assert!(handle.is_addr_in_use(), "{handle:?}");
    assert!(handle.local_addr().is_none());
    assert!(!handle.is_running());
    handle.stop().unwrap();

    let err = common::runtime()
        .block_on(kanso_server::serve(
            store.clone(),
            config,
            keys_for(&store),
            async {},
        ))
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
    let keys = keys_for(&store);
    let server = tokio::spawn(kanso_server::serve(
        store,
        ServerConfig { bind: addr },
        keys,
        async move {
            let _ = rx.await;
        },
    ));
    let client = reqwest::Client::new();
    let mut status = None;
    for _ in 0..50 {
        match client
            .get(format!("http://{addr}/notes"))
            .header(header::AUTHORIZATION.as_str(), common::auth())
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
    let doc = raw(&app, &fresh);
    assert!(doc.get("kansoPublish").is_none(), "{doc}");
    write(
        &app.router,
        &format!("/{fresh}"),
        json!({ "kansoPublish": planted }),
    )
    .await;
    let doc = raw(&app, &fresh);
    assert!(doc.get("kansoPublish").is_none(), "{doc}");

    // A published note keeps the app's value through replace and merge, whatever is sent.
    let (id, rev) = app_note(
        &app,
        json!({ "bookId": "trash", "title": "t", "body": "b" }),
    );
    let mut doc = raw(&app, &id);
    assert_eq!(doc["_rev"], rev.as_str());
    doc["kansoPublish"] = planted.clone();
    doc["title"] = "replaced".into();
    write(&app.router, "/notes", doc).await;
    let mut doc = raw(&app, &id);
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
        let doc = raw(&app, &id);
        assert_eq!(doc["kansoPublish"], published(), "{doc}");
    }
    let doc = raw(&app, &id);
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
    let doc = raw(&app, &id);
    assert!(doc.get("kansoPublish").is_none(), "{doc}");
    write(
        &app.router,
        &format!("/{id}"),
        json!({ "kansoPublish": planted }),
    )
    .await;
    let doc = raw(&app, &id);
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
        let doc = raw(&app, &id);
        assert_eq!(doc["kansoPublish"], published(), "{doc}");
    }
    let doc = raw(&app, &id);
    assert_eq!(doc["name"], "Renamed");
}

#[tokio::test]
async fn a_locked_note_does_not_exist_for_a_key() {
    let app = app();
    let sealed = json!({ "v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "sealed" });
    let book_id = book(&app, "B").await;
    let (id, _) = app_note(&app, json!({ "bookId": book_id, "title": "t", "body": "" }));
    let note = app.store.get_api(&id, None, false).unwrap();
    let mut locked = note.clone();
    locked["kansoLock"] = sealed.clone();
    app.store.lock_note(locked).unwrap();
    let sealed_kept = |store: &kanso_store::Store| {
        store
            .note(&id)
            .unwrap()
            .unwrap()
            .extra
            .get("kansoLock")
            .cloned()
    };

    // It is not read, listed, replaced, merged into or deleted from here:
    // every try answers not found, and the note stays as it was.
    let (status, _) = get(&app.router, &format!("/{id}")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (_, listed) = get(&app.router, "/notes").await;
    assert!(!listed.to_string().contains(&id), "{listed}");
    let mut replacement = raw(&app, &id);
    replacement["title"] = "retitled".into();
    let (status, _) = post(&app.router, "/notes", replacement).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    for body in [
        json!({ "body": "plain" }),
        json!({ "kansoLock": null, "body": "plain" }),
    ] {
        let (status, _) = post(&app.router, &format!("/{id}"), body).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }
    let (status, _) = call(&app.router, Method::DELETE, &format!("/{id}"), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(app.store.note(&id).unwrap().unwrap().title, "t");
    assert_eq!(sealed_kept(&app.store), Some(sealed));
    // Nor does the changes feed carry it.
    let (_, feed) = get(&app.router, "/_changes?include_docs=true").await;
    assert!(!feed.to_string().contains(&id), "{feed}");
    assert!(!feed.to_string().contains("c2FsdA"), "{feed}");
}

#[tokio::test]
async fn a_locked_notebook_and_its_notes_do_not_exist_for_a_key() {
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

    // The note is nowhere: not read, found, listed or in the feed, over
    // REST or MCP.
    let (status, _) = get(&app.router, &format!("/{id}")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    for uri in [
        "/notes?keyword=zebracanary4471".to_owned(),
        "/notes?keyword=Plans".to_owned(),
        "/notes".to_owned(),
        "/_changes?include_docs=true".to_owned(),
        "/books".to_owned(),
    ] {
        let (_, found) = get(&app.router, &uri).await;
        let found = found.to_string();
        assert!(!found.contains("zebracanary4471"), "{uri}: {found}");
        assert!(!found.contains("Plans"), "{uri}: {found}");
        assert!(!found.contains("Private"), "{uri}: {found}");
        assert!(!found.contains("c2FsdA"), "{uri}: {found}");
    }
    for (name, arguments) in [
        ("read-note", json!({ "noteId": id })),
        ("search-notes", json!({ "keyword": "Plans" })),
        ("list-notes", json!({ "bookId": inner })),
        ("list-notebooks", json!({})),
    ] {
        let answer = tool(&app.router, name, arguments).await;
        assert!(!answer.to_string().contains("Plans"), "{name}: {answer}");
        assert!(!answer.to_string().contains("Private"), "{name}: {answer}");
        assert!(
            !answer.to_string().contains("zebracanary4471"),
            "{name}: {answer}"
        );
    }
    // Nor is its text changed, emptied, moved out or deleted from here, and
    // no note is added to the notebook.
    for (uri, body, expected) in [
        (
            format!("/{id}"),
            json!({ "body": "replaced" }),
            StatusCode::NOT_FOUND,
        ),
        (
            format!("/{id}"),
            json!({ "body": "" }),
            StatusCode::NOT_FOUND,
        ),
        (
            format!("/{id}"),
            json!({ "bookId": "trash" }),
            StatusCode::NOT_FOUND,
        ),
        (
            "/notes".to_owned(),
            json!({ "bookId": book, "title": "New", "body": "plain" }),
            StatusCode::FORBIDDEN,
        ),
    ] {
        let (status, _) = post(&app.router, &uri, body).await;
        assert_eq!(status, expected, "{uri}");
    }
    let (status, _) = call(&app.router, Method::DELETE, &format!("/{id}"), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // The notebook's lock never goes out, can't be taken off or planted,
    // and the notebook can't be deleted from here.
    for target in [&book, &inner] {
        let (status, _) = get(&app.router, &format!("/{target}")).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        let (status, _) = post(
            &app.router,
            &format!("/{target}"),
            json!({ "kansoLock": null, "name": "Renamed" }),
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        let (status, _) = call(&app.router, Method::DELETE, &format!("/{target}"), None).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }
    assert_eq!(
        app.store
            .get(&book)
            .unwrap()
            .unwrap()
            .fields
            .get("kansoLock"),
        Some(&lock)
    );
    assert_eq!(
        app.store.get(&book).unwrap().unwrap().fields["name"],
        "Private"
    );
    assert!(
        app.store
            .note(&id)
            .unwrap()
            .unwrap()
            .body
            .contains("zebracanary4471")
    );
}
