//! API keys on the local API (issue #243, step 4): what each access level
//! reaches over REST and `/mcp`. CI pins the seed with `PROPTEST_RNG_SEED`.

mod common;

use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use base64::Engine as _;
use common::*;
use kanso_core::DocType;
use kanso_server::{KeyAccess, KeyId, NewKey};
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

type CaseResult = Result<(), TestCaseError>;

fn block_on(future: impl Future<Output = CaseResult>) -> CaseResult {
    runtime().block_on(future)
}

/// A key of `access` for `app`, limited to `books`.
fn mint(app: &App, access: KeyAccess, books: Option<Vec<String>>) -> (KeyId, String) {
    let minted = app
        .keys
        .mint(NewKey {
            name: "Test agent".into(),
            client: "other".into(),
            access,
            books,
        })
        .unwrap();
    (minted.id, minted.key)
}

/// A request signed with `key` (`None` sends no `Authorization`).
fn signed(key: Option<&str>, method: Method, uri: &str, body: Option<&Value>) -> Request<Body> {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(key) = key {
        builder = builder.header(header::AUTHORIZATION, bearer(key));
    }
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
) -> (StatusCode, Value, String) {
    let response = send(router, signed(Some(key), method, uri, body)).await;
    let text = String::from_utf8_lossy(&response.bytes).into_owned();
    (response.status, response.json(), text)
}

async fn tool_as(router: &Router, key: &str, name: &str, arguments: Value) -> Value {
    let message = json!({
        "jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": { "name": name, "arguments": arguments },
    });
    let (status, answer, _) = call_as(router, key, Method::POST, "/mcp", Some(&message)).await;
    assert_eq!(status, StatusCode::OK, "{answer}");
    answer["result"].clone()
}

async fn tool_names(router: &Router, key: &str) -> Vec<String> {
    let message = json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/list" });
    let (_, answer, _) = call_as(router, key, Method::POST, "/mcp", Some(&message)).await;
    answer["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tool| tool["name"].as_str().unwrap().to_owned())
        .collect()
}

/// A notebook, a note in it and a note nobody may read.
struct Seeded {
    book: String,
    note: String,
    locked: String,
    locked_book_note: String,
    secret: String,
}

fn seed(store: &Store, secret: &str) -> Seeded {
    let put = |doc_type, doc: Value| {
        store
            .put(doc_type, doc, Put::Upsert, Origin::Sync)
            .unwrap()
            .id
    };
    let book = put(DocType::Book, json!({ "name": "Work" }));
    let note = put(
        DocType::Note,
        json!({ "title": "Open note", "body": "plain text", "bookId": book }),
    );
    let locked = put(
        DocType::Note,
        json!({ "title": format!("title-{secret}"), "body": format!("body-{secret}"), "bookId": book }),
    );
    store
        .put(
            DocType::Note,
            json!({ "_id": locked, "kansoLock": { "salt": "c2FsdA" } }),
            Put::Merge,
            Origin::Sync,
        )
        .unwrap();
    let vault = put(
        DocType::Book,
        json!({ "name": format!("vault-{secret}"), "kansoLock": { "salt": "c2FsdA" } }),
    );
    let locked_book_note = put(
        DocType::Note,
        json!({ "title": format!("vaulted-{secret}"), "body": format!("vaulted body {secret}"), "bookId": vault }),
    );
    Seeded {
        book,
        note,
        locked,
        locked_book_note,
        secret: secret.to_owned(),
    }
}

fn access() -> impl Strategy<Value = KeyAccess> {
    prop::sample::select(vec![KeyAccess::Read, KeyAccess::Suggest, KeyAccess::Write])
}

proptest! {
    #![proptest_config(config(24))]

    /// No key is ever shown a locked note, or a note in a locked notebook,
    /// or either one's notebook: not in any list, search, feed, document
    /// or tool answer, at any access level, with or without a notebook
    /// limit.
    #[test]
    fn locked_notes_are_never_returned_to_any_key(
        secret in "[a-z]{12}",
        access in access(),
        limited in any::<bool>(),
    ) {
        let app = app();
        let seeded = seed(&app.store, &secret);
        let books = limited.then(|| vec![seeded.book.clone()]);
        let (_, key) = mint(&app, access, books);
        let bare = |id: &str| id.split_once(':').unwrap().1.to_owned();
        block_on(async move {
            let mut uris = vec![
                "/notes".to_owned(),
                format!("/notes?keyword={}", secret),
                "/notes?limit=500&sort=title".to_owned(),
                "/_changes?include_docs=true".to_owned(),
                "/_changes".to_owned(),
                "/books".to_owned(),
                "/tags".to_owned(),
                "/files".to_owned(),
                "/loops".to_owned(),
            ];
            for id in [&seeded.locked, &seeded.locked_book_note] {
                uris.push(format!("/{id}"));
                uris.push(format!("/note/{}", bare(id)));
                uris.push(format!("/{id}?rev=1-x"));
            }
            for uri in &uris {
                let (status, _, text) = call_as(&app.router, &key, Method::GET, uri, None).await;
                prop_assert!(
                    !text.contains(&seeded.secret) && !text.contains("c2FsdA") && !text.contains("kansoLock"),
                    "{} {} {}", status, uri, text
                );
                if uri.starts_with(&format!("/{}", seeded.locked)) || uri.contains(&bare(&seeded.locked)) || uri.contains(&bare(&seeded.locked_book_note)) || uri.starts_with(&format!("/{}", seeded.locked_book_note)) {
                    prop_assert_eq!(status, StatusCode::NOT_FOUND, "{}", uri);
                }
            }
            let reads = [
                ("read-note", json!({ "noteId": seeded.locked })),
                ("read-note", json!({ "noteId": seeded.locked_book_note })),
                ("search-notes", json!({ "keyword": secret })),
                ("list-notes", json!({})),
                ("list-notebooks", json!({})),
                ("kanso-list-tags", json!({})),
                ("kanso-get-outline", json!({ "noteId": seeded.locked })),
                ("kanso-list-versions", json!({ "noteId": seeded.locked })),
                ("kanso-related-notes", json!({ "noteId": seeded.note })),
                ("kanso-list-backlinks", json!({ "noteId": seeded.locked })),
                ("kanso-search-by-meaning", json!({ "query": secret })),
                ("kanso-ask-search", json!({ "query": secret })),
                ("read-backlinks", json!({ "noteId": seeded.locked })),
                ("kanso-list-attachments", json!({ "noteId": seeded.locked })),
                ("kanso-export-html", json!({ "noteId": seeded.locked })),
                ("read-book", json!({ "bookId": seeded.locked_book_note })),
            ];
            for (name, arguments) in reads {
                let result = tool_as(&app.router, &key, name, arguments.clone()).await;
                let shown = result.to_string();
                prop_assert!(
                    !shown.contains(&seeded.secret) && !shown.contains("c2FsdA"),
                    "{} {} {}", name, arguments, shown
                );
            }
            let resource = json!({
                "jsonrpc": "2.0", "id": 2, "method": "resources/read",
                "params": { "uri": format!("inkdrop://note/{}", bare(&seeded.locked)) },
            });
            let (_, _, text) = call_as(&app.router, &key, Method::POST, "/mcp", Some(&resource)).await;
            prop_assert!(!text.contains(&seeded.secret), "{}", text);
            Ok(())
        })?;
    }
}

/// The write routes, with a document of each kind to aim at.
fn write_requests(seeded: &Seeded, id: &str, body: &Value) -> Vec<(Method, String, Option<Value>)> {
    let bare = id.split_once(':').map_or(id, |(_, rest)| rest).to_owned();
    vec![
        (Method::POST, "/notes".into(), Some(body.clone())),
        (Method::POST, "/books".into(), Some(body.clone())),
        (Method::POST, "/tags".into(), Some(body.clone())),
        (Method::POST, "/files".into(), Some(body.clone())),
        (Method::POST, format!("/{id}"), Some(body.clone())),
        (Method::POST, format!("/note/{bare}"), Some(body.clone())),
        (
            Method::POST,
            format!("/{}", seeded.note),
            Some(body.clone()),
        ),
        (
            Method::POST,
            format!("/{}", seeded.book),
            Some(body.clone()),
        ),
        (Method::DELETE, format!("/{id}"), None),
        (Method::DELETE, format!("/{}", seeded.note), None),
        (Method::DELETE, format!("/{}", seeded.book), None),
        (Method::DELETE, format!("/book/{bare}"), None),
        (Method::PUT, format!("/{}", seeded.note), Some(body.clone())),
        (
            Method::PATCH,
            format!("/{}", seeded.note),
            Some(body.clone()),
        ),
        (
            Method::PATCH,
            format!("/{}/canvas", seeded.note),
            Some(json!({ "ops": [{ "op": "rename", "title": "x" }] })),
        ),
        (
            Method::PATCH,
            format!("/note/{bare}/canvas"),
            Some(json!({ "ops": [] })),
        ),
    ]
}

fn small_body() -> impl Strategy<Value = Value> {
    (
        proptest::option::of("[a-zA-Z ]{0,12}"),
        proptest::option::of("[a-zA-Z ]{0,12}"),
        any::<bool>(),
    )
        .prop_map(|(title, body, trash)| {
            let mut doc = json!({ "name": "Renamed" });
            if let Some(title) = title {
                doc["title"] = json!(title);
            }
            if let Some(body) = body {
                doc["body"] = json!(body);
            }
            if trash {
                doc["bookId"] = json!("trash");
            }
            doc
        })
}

proptest! {
    #![proptest_config(config(24))]

    /// A Read or Suggest key never reaches a write route or tool: every
    /// write route refuses it with 403, every tool but the suggesting ones
    /// refuses it, and the library does not change by a single revision.
    /// A Suggest key's suggestions are only staged for review.
    #[test]
    fn read_and_suggest_keys_never_write(
        suggest in any::<bool>(),
        body in small_body(),
        id in prop::sample::select(vec!["note:n0", "note:missing", "book:b0", "tag:t0", "file:f0", "bogus"]),
        tool_arguments in prop::collection::vec(("[a-zA-Z_]{1,8}", "[a-zA-Z0-9 ]{0,12}"), 0..4),
    ) {
        let app = app();
        let seeded = seed(&app.store, "zzzzzzzzzzzz");
        let access = if suggest { KeyAccess::Suggest } else { KeyAccess::Read };
        let (_, key) = mint(&app, access, None);
        let before = app.store.last_seq().unwrap();
        block_on(async move {
            for (method, uri, payload) in write_requests(&seeded, id, &body) {
                let (status, answer, _) = call_as(&app.router, &key, method.clone(), &uri, payload.as_ref()).await;
                prop_assert_eq!(status, StatusCode::FORBIDDEN, "{} {}: {}", method, uri, answer);
                prop_assert_eq!(&answer["error"], &json!("forbidden"));
                // The answer says how to fix it.
                prop_assert!(
                    answer["reason"].as_str().unwrap().contains("Settings \u{203a} Local API \u{203a} API keys"),
                    "{}", answer
                );
            }
            // Every tool the full catalog offers, with arguments that name
            // real documents and arguments that are noise.
            let all = tool_names(&app.router, KEY).await;
            let offered = tool_names(&app.router, &key).await;
            for name in &offered {
                prop_assert!(
                    kanso_server::READ_TOOLS.contains(&name.as_str())
                        || (suggest && kanso_server::SUGGEST_TOOLS.contains(&name.as_str())),
                    "{} offered to {:?}", name, access
                );
            }
            let mut arguments = json!({
                "noteId": seeded.note, "_id": seeded.note, "bookId": seeded.book, "_rev": "1-x",
                "title": "t", "body": "b", "name": "n", "old_string": "plain", "new_string": "x",
                "data": "AAAA", "contentType": "image/png", "filePath": "/etc/hosts",
                "outputPath": "/tmp/kanso-never", "fileId": "file:f0",
            });
            for (name, value) in &tool_arguments {
                arguments[name] = json!(value);
            }
            for name in &all {
                let allowed = offered.contains(name);
                let result = tool_as(&app.router, &key, name, arguments.clone()).await;
                if !allowed {
                    prop_assert_eq!(&result["isError"], &json!(true), "{}: {}", name, result);
                }
            }
            prop_assert_eq!(app.store.last_seq().unwrap(), before, "a read or suggest key changed the library");
            prop_assert!(!std::path::Path::new("/tmp/kanso-never").exists());
            Ok(())
        })?;
    }

    /// A key that was never minted, and a key that was revoked, get 401 on
    /// every route; the same key was accepted before its revoke.
    #[test]
    fn a_revoked_key_gets_401(access in access(), uri in prop::sample::select(vec!["/", "/notes", "/books", "/_changes", "/note:x", "/mcp"])) {
        let app = app();
        let (id, key) = mint(&app, access, None);
        let method = if uri == "/mcp" { Method::POST } else { Method::GET };
        let payload = json!({ "jsonrpc": "2.0", "id": 1, "method": "ping" });
        let body = (uri == "/mcp").then_some(&payload);
        block_on(async move {
            let (status, _, _) = call_as(&app.router, &key, method.clone(), uri, body).await;
            prop_assert_ne!(status, StatusCode::UNAUTHORIZED, "{}", uri);
            app.keys.revoke(&id).unwrap();
            let (status, answer, _) = call_as(&app.router, &key, method.clone(), uri, body).await;
            prop_assert_eq!(status, StatusCode::UNAUTHORIZED, "{}", uri);
            prop_assert_eq!(
                &answer["reason"],
                &json!("Kanso needs an API key. Create one in Settings \u{203a} Local API \u{203a} API keys.")
            );
            // The same key as a Basic password is refused too.
            let basic = format!("Basic {}", base64_of(&format!("anyone:{key}")));
            let request = Request::builder()
                .method(method)
                .uri(uri)
                .header(header::AUTHORIZATION, basic)
                .body(Body::empty())
                .unwrap();
            prop_assert_eq!(send(&app.router, request).await.status, StatusCode::UNAUTHORIZED);
            Ok(())
        })?;
    }
}

fn base64_of(text: &str) -> String {
    use base64::Engine as _;
    base64::engine::general_purpose::STANDARD.encode(text)
}

#[tokio::test]
async fn read_keys_read_suggest_keys_suggest_and_write_keys_write() {
    let app = app();
    let seeded = seed(&app.store, "abcdefghijkl");
    let (_, read) = mint(&app, KeyAccess::Read, None);
    let (_, suggest) = mint(&app, KeyAccess::Suggest, None);
    let (_, write_key) = mint(&app, KeyAccess::Write, None);

    for key in [&read, &suggest, &write_key] {
        let (status, doc, _) = call_as(
            &app.router,
            key,
            Method::GET,
            &format!("/{}", seeded.note),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(doc["title"], "Open note");
        let (_, notes, _) = call_as(&app.router, key, Method::GET, "/notes", None).await;
        let found: Vec<&str> = notes
            .as_array()
            .unwrap()
            .iter()
            .map(|n| n["_id"].as_str().unwrap())
            .collect();
        assert_eq!(found, [seeded.note.as_str()]);
    }

    // Read and suggest keys see no attachments, whatever the route.
    let file = app
        .store
        .put(
            DocType::File,
            json!({ "name": "p.png", "contentType": "image/png",
                    "_attachments": { "index": { "content_type": "image/png", "data": base64_of("png") } } }),
            Put::Upsert,
            Origin::Api,
        )
        .unwrap()
        .id;
    for key in [&read, &suggest] {
        let (_, files, _) = call_as(&app.router, key, Method::GET, "/files", None).await;
        assert_eq!(files, json!([]));
        let (status, _, _) = call_as(
            &app.router,
            key,
            Method::GET,
            &format!("/{file}?attachments=true"),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        let (_, feed, text) = call_as(
            &app.router,
            key,
            Method::GET,
            "/_changes?include_docs=true",
            None,
        )
        .await;
        assert!(!text.contains(&file), "{feed}");
        for name in [
            "create-file",
            "read-file",
            "kanso-attach-file",
            "kanso-list-attachments",
            "kanso-read-attachment",
        ] {
            let result = tool_as(
                &app.router,
                key,
                name,
                json!({ "fileId": file, "noteId": seeded.note }),
            )
            .await;
            assert_eq!(result["isError"], true, "{name}");
            assert!(
                result.to_string().contains("no access to attachments"),
                "{name}: {result}"
            );
        }
    }
    let (status, doc, _) = call_as(
        &app.router,
        &write_key,
        Method::GET,
        &format!("/{file}?attachments=true"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(doc["_attachments"]["index"]["data"].is_string());

    // A read key's write attempt says how to get a key that can.
    let (status, answer, _) = call_as(
        &app.router,
        &read,
        Method::POST,
        "/notes",
        Some(&json!({ "bookId": seeded.book, "title": "x" })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(
        answer["reason"]
            .as_str()
            .unwrap()
            .contains("create a key that can read and write")
    );
    let result = tool_as(&app.router, &read, "update-note", json!({})).await;
    assert!(
        result
            .to_string()
            .contains("create a key that can read and write")
    );

    // A suggest key suggests: the library is untouched.
    let before = app.store.last_seq().unwrap();
    let note = app.store.get_api(&seeded.note, None, false).unwrap();
    let result = tool_as(
        &app.router,
        &suggest,
        "update-note",
        json!({ "_id": seeded.note, "_rev": note["_rev"], "title": "Suggested" }),
    )
    .await;
    assert_ne!(result["isError"], true, "{result}");
    assert!(result.to_string().contains("Queued for review"));
    assert_eq!(app.store.last_seq().unwrap(), before);
    // The review queue is the tokens'; the key's name is on it.
    let queued = tool_names(&app.router, &suggest).await;
    assert!(queued.contains(&"update-note".to_owned()));
    assert!(!queued.contains(&"update-book".to_owned()));

    // A write key writes right away, and still never reaches what is locked.
    let result = tool_as(
        &app.router,
        &write_key,
        "update-note",
        json!({ "_id": seeded.note, "_rev": note["_rev"], "title": "Written" }),
    )
    .await;
    assert_ne!(result["isError"], true, "{result}");
    assert_eq!(
        app.store.get_api(&seeded.note, None, false).unwrap()["title"],
        "Written"
    );
    for id in [&seeded.locked, &seeded.locked_book_note] {
        let (status, _, _) = call_as(
            &app.router,
            &write_key,
            Method::POST,
            &format!("/{id}"),
            Some(&json!({ "title": "hacked" })),
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        let (status, _, _) = call_as(
            &app.router,
            &write_key,
            Method::DELETE,
            &format!("/{id}"),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        let result = tool_as(
            &app.router,
            &write_key,
            "update-note",
            json!({ "_id": id, "_rev": "1-x", "title": "hacked" }),
        )
        .await;
        assert_eq!(result["isError"], true);
        assert_ne!(
            app.store.get_api(id, None, false).unwrap()["title"],
            "hacked"
        );
    }
}

#[tokio::test]
async fn a_key_limited_to_a_notebook_reads_and_writes_only_there() {
    let app = app();
    let seeded = seed(&app.store, "abcdefghijkl");
    let other = app
        .store
        .put(
            DocType::Book,
            json!({ "name": "Elsewhere" }),
            Put::Upsert,
            Origin::Api,
        )
        .unwrap()
        .id;
    let elsewhere = app
        .store
        .put(
            DocType::Note,
            json!({ "title": "Elsewhere note", "body": "x", "bookId": other }),
            Put::Upsert,
            Origin::Api,
        )
        .unwrap()
        .id;
    let (_, key) = mint(&app, KeyAccess::Write, Some(vec![seeded.book.clone()]));

    let (status, _, _) = call_as(
        &app.router,
        &key,
        Method::GET,
        &format!("/{elsewhere}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (_, books, _) = call_as(&app.router, &key, Method::GET, "/books", None).await;
    assert_eq!(books.as_array().unwrap().len(), 1);
    // The open note only: the locked one beside it isn't counted.
    assert_eq!(books[0]["count"], 1);
    // Not written into, moved into, or edited there.
    let (status, answer, _) = call_as(
        &app.router,
        &key,
        Method::POST,
        "/notes",
        Some(&json!({ "bookId": other, "title": "x", "body": "" })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "{answer}");
    assert!(answer["reason"].as_str().unwrap().contains("Settings"));
    let (status, _, _) = call_as(
        &app.router,
        &key,
        Method::POST,
        &format!("/{elsewhere}"),
        Some(&json!({ "title": "y" })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _, _) = call_as(
        &app.router,
        &key,
        Method::POST,
        &format!("/{}", seeded.note),
        Some(&json!({ "bookId": other })),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let result = tool_as(
        &app.router,
        &key,
        "kanso-move-note",
        json!({ "noteId": seeded.note, "bookId": other }),
    )
    .await;
    assert_eq!(result["isError"], true);
    let result = tool_as(&app.router, &key, "kanso-create-daily-note", json!({})).await;
    assert_eq!(result["isError"], true);
    assert_eq!(
        app.store.get_api(&elsewhere, None, false).unwrap()["title"],
        "Elsewhere note"
    );
    assert_eq!(
        app.store.get_api(&seeded.note, None, false).unwrap()["bookId"],
        json!(seeded.book)
    );
    // In its own notebook it writes.
    let (status, answer, _) = call_as(
        &app.router,
        &key,
        Method::POST,
        "/notes",
        Some(&json!({ "bookId": seeded.book, "title": "mine", "body": "" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{answer}");
}

#[tokio::test]
async fn a_session_token_opens_mcp_alone_and_a_key_opens_both() {
    let app = app();
    let (_, key) = mint(&app, KeyAccess::Read, None);
    // Basic with the key as the password works for tools that only take a login.
    for user in ["", "anyone", "kanso"] {
        let request = Request::builder()
            .uri("/notes")
            .header(
                header::AUTHORIZATION,
                format!("Basic {}", base64_of(&format!("{user}:{key}"))),
            )
            .body(Body::empty())
            .unwrap();
        assert_eq!(send(&app.router, request).await.status, StatusCode::OK);
    }
    // A wrong password, an old login and a half key are refused.
    for bad in [
        "Basic a2Fuc286cGFzcw==".to_owned(),
        format!("Basic {}", base64_of(&format!("anyone:{}", &key[..40]))),
        format!("Bearer {}x", key),
        format!("Token {key}"),
    ] {
        let request = Request::builder()
            .uri("/notes")
            .header(header::AUTHORIZATION, bad.clone())
            .body(Body::empty())
            .unwrap();
        assert_eq!(
            send(&app.router, request).await.status,
            StatusCode::UNAUTHORIZED,
            "{bad}"
        );
    }
}

/// A refused request is noted for Settings (time only); an accepted one is not.
#[tokio::test]
async fn a_refused_request_is_noted_and_an_accepted_one_is_not() {
    let app = app();
    let (_, key) = mint(&app, KeyAccess::Read, None);
    assert_eq!(app.keys.last_refused(), None);
    let (status, _, _) = call_as(&app.router, &key, Method::GET, "/notes", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(app.keys.last_refused(), None);
    let request = Request::builder()
        .uri("/notes")
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        send(&app.router, request).await.status,
        StatusCode::UNAUTHORIZED
    );
    assert!(app.keys.last_refused().is_some());
}

/// An attachment, uploaded with a key for every notebook. Nothing holds it
/// until a note links it.
async fn upload_png(app: &App) -> String {
    let (_, writer) = mint(app, KeyAccess::Write, None);
    let data = base64::engine::general_purpose::STANDARD.encode(PNG);
    let (status, answer, _) = call_as(
        &app.router,
        &writer,
        Method::POST,
        "/files",
        Some(&json!({
            "name": "pixel.png", "contentType": "image/png", "contentLength": PNG.len(), "publicIn": [],
            "_attachments": { "index": { "content_type": "image/png", "data": data } }
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{answer}");
    answer["id"].as_str().unwrap().to_owned()
}

fn link(file: &str) -> String {
    format!(
        "![pixel]({}{})",
        kanso_core::links::FILE_LINK_PREFIX,
        file.trim_start_matches("file:")
    )
}

/// A key that can write sees an attachment only when it sees every note
/// that holds it: never one held by a note in a locked notebook, or by a
/// note outside its notebooks, over REST, `_changes` or a tool.
#[tokio::test]
async fn a_write_key_never_reaches_an_attachment_of_a_note_it_cannot_see() {
    let app = app();
    let seeded = seed(&app.store, "qwertyuiopas");
    let vaulted = upload_png(&app).await;
    let elsewhere_file = upload_png(&app).await;
    let orphan = upload_png(&app).await;
    let mine = upload_png(&app).await;
    let put = |doc: Value| {
        app.store
            .put(DocType::Note, doc, Put::Upsert, Origin::Sync)
            .unwrap()
            .id
    };
    let vault = app
        .store
        .get_api(&seeded.locked_book_note, None, false)
        .unwrap()["bookId"]
        .as_str()
        .unwrap()
        .to_owned();
    put(json!({ "title": "in the vault", "body": link(&vaulted), "bookId": vault }));
    let other = app
        .store
        .put(
            DocType::Book,
            json!({ "name": "Elsewhere" }),
            Put::Upsert,
            Origin::Api,
        )
        .unwrap()
        .id;
    put(json!({ "title": "elsewhere", "body": link(&elsewhere_file), "bookId": other }));
    put(json!({ "title": "mine", "body": link(&mine), "bookId": seeded.book }));

    let (_, everywhere) = mint(&app, KeyAccess::Write, None);
    let (_, limited) = mint(&app, KeyAccess::Write, Some(vec![seeded.book.clone()]));
    // (key, files it sees, files it doesn't)
    let cases = [
        (
            &everywhere,
            vec![&elsewhere_file, &orphan, &mine],
            vec![&vaulted],
        ),
        (
            &limited,
            vec![&mine],
            vec![&vaulted, &elsewhere_file, &orphan],
        ),
    ];
    for (key, shown, hidden) in cases {
        let (_, files, _) = call_as(&app.router, key, Method::GET, "/files", None).await;
        let listed: Vec<&str> = files
            .as_array()
            .unwrap()
            .iter()
            .map(|file| file["_id"].as_str().unwrap())
            .collect();
        let (_, feed, _) = call_as(&app.router, key, Method::GET, "/_changes", None).await;
        let fed: Vec<&str> = feed["results"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row["id"].as_str().unwrap())
            .collect();
        for file in &shown {
            assert!(listed.contains(&file.as_str()), "{file} in {listed:?}");
            let (status, _, _) =
                call_as(&app.router, key, Method::GET, &format!("/{file}"), None).await;
            assert_eq!(status, StatusCode::OK, "{file}");
        }
        for file in &hidden {
            assert!(!listed.contains(&file.as_str()), "{file}");
            assert!(!fed.contains(&file.as_str()), "{file}");
            let (status, _, _) = call_as(
                &app.router,
                key,
                Method::GET,
                &format!("/{file}?attachments=true"),
                None,
            )
            .await;
            assert_eq!(status, StatusCode::NOT_FOUND, "{file}");
            let read = tool_as(&app.router, key, "read-file", json!({ "fileId": file })).await;
            assert_eq!(read["isError"], true, "{file}: {read}");
            // Nor overwritten or deleted.
            let (status, _, _) =
                call_as(&app.router, key, Method::DELETE, &format!("/{file}"), None).await;
            assert_eq!(status, StatusCode::NOT_FOUND, "{file}");
        }
    }
    // Linking a hidden attachment into a note it sees doesn't open it.
    let (status, _, _) = call_as(
        &app.router,
        &limited,
        Method::POST,
        &format!("/{}", seeded.note),
        Some(&json!({ "body": link(&elsewhere_file) })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let read = tool_as(
        &app.router,
        &limited,
        "kanso-read-attachment",
        json!({ "noteId": seeded.note, "fileId": elsewhere_file }),
    )
    .await;
    assert_eq!(read["isError"], true, "{read}");
    let listed = tool_as(
        &app.router,
        &limited,
        "kanso-list-attachments",
        json!({ "noteId": seeded.note }),
    )
    .await;
    assert!(!listed.to_string().contains(&elsewhere_file), "{listed}");
}

/// No key reads, renames or deletes the `#ai-exclude` tag, which would show
/// the notes it hides; and no key hears of a tag only hidden notes carry,
/// or a count that includes them.
#[tokio::test]
async fn keys_never_reach_the_ai_exclude_tag_or_hidden_counts() {
    let app = app();
    let seeded = seed(&app.store, "zxcvbnmasdfg");
    let tag = |name: &str| {
        app.store
            .put(
                DocType::Tag,
                json!({ "name": name }),
                Put::Upsert,
                Origin::Sync,
            )
            .unwrap()
            .id
    };
    let exclude = tag("ai-exclude");
    let secret_tag = tag("divorce-lawyer");
    let open_tag = tag("work");
    let put = |doc: Value| {
        app.store
            .put(DocType::Note, doc, Put::Upsert, Origin::Sync)
            .unwrap()
            .id
    };
    let excluded = put(json!({
        "title": "excluded", "body": "private", "bookId": seeded.book, "tags": [exclude]
    }));
    // The tag a locked note alone carries.
    app.store
        .put(
            DocType::Note,
            json!({ "_id": seeded.locked, "tags": [secret_tag] }),
            Put::Merge,
            Origin::Sync,
        )
        .unwrap();
    app.store
        .put(
            DocType::Note,
            json!({ "_id": seeded.note, "tags": [open_tag] }),
            Put::Merge,
            Origin::Sync,
        )
        .unwrap();
    let (_, key) = mint(&app, KeyAccess::Write, None);

    let (_, tags, text) = call_as(&app.router, &key, Method::GET, "/tags", None).await;
    assert!(!text.contains("ai-exclude"), "{text}");
    assert!(!text.contains("divorce-lawyer"), "{text}");
    let work: Vec<&Value> = tags
        .as_array()
        .unwrap()
        .iter()
        .filter(|tag| tag["_id"] == json!(open_tag))
        .collect();
    assert_eq!(work.len(), 1);
    let (_, feed, _) = call_as(&app.router, &key, Method::GET, "/_changes", None).await;
    assert!(!feed.to_string().contains(&exclude));
    assert!(!feed.to_string().contains(&secret_tag));
    let (_, books, _) = call_as(&app.router, &key, Method::GET, "/books", None).await;
    let work_book = books
        .as_array()
        .unwrap()
        .iter()
        .find(|book| book["_id"] == json!(seeded.book))
        .unwrap();
    // The open note alone: not the locked or the excluded one.
    assert_eq!(work_book["count"], 1, "{work_book}");
    let listed = tool_as(&app.router, &key, "list-tags", json!({})).await;
    assert!(!listed.to_string().contains("divorce-lawyer"), "{listed}");
    assert!(!listed.to_string().contains("ai-exclude"), "{listed}");

    for hidden in [&exclude, &secret_tag] {
        let (status, _, _) =
            call_as(&app.router, &key, Method::GET, &format!("/{hidden}"), None).await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{hidden}");
    }
    let (status, _, _) = call_as(
        &app.router,
        &key,
        Method::POST,
        &format!("/{exclude}"),
        Some(&json!({ "name": "renamed" })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _, _) = call_as(
        &app.router,
        &key,
        Method::DELETE,
        &format!("/{exclude}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let renamed = tool_as(
        &app.router,
        &key,
        "update-tag",
        json!({ "_id": exclude, "name": "renamed" }),
    )
    .await;
    assert_eq!(renamed["isError"], true, "{renamed}");
    assert_eq!(
        app.store.get_api(&exclude, None, false).unwrap()["name"],
        "ai-exclude"
    );
    let (status, _, _) = call_as(
        &app.router,
        &key,
        Method::GET,
        &format!("/{excluded}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

/// A key limited to some notebooks can't restore a trashed note, which may
/// have come from any notebook, into one of its own; nor can any key reach
/// a trashed `#ai-exclude` note.
#[tokio::test]
async fn a_trashed_note_is_written_only_by_a_key_for_every_notebook() {
    let app = app();
    let seeded = seed(&app.store, "poiuytrewqlk");
    let exclude = app
        .store
        .put(
            DocType::Tag,
            json!({ "name": "ai-exclude" }),
            Put::Upsert,
            Origin::Sync,
        )
        .unwrap()
        .id;
    let put = |doc: Value| {
        app.store
            .put(DocType::Note, doc, Put::Upsert, Origin::Sync)
            .unwrap()
            .id
    };
    let trashed = put(json!({ "title": "from elsewhere", "body": "x", "bookId": "trash" }));
    let trashed_excluded = put(json!({
        "title": "excluded", "body": "y", "bookId": "trash", "tags": [exclude]
    }));
    let (_, limited) = mint(&app, KeyAccess::Write, Some(vec![seeded.book.clone()]));
    let (_, everywhere) = mint(&app, KeyAccess::Write, None);
    let restore = json!({ "bookId": seeded.book });
    let (status, _, _) = call_as(
        &app.router,
        &limited,
        Method::POST,
        &format!("/{trashed}"),
        Some(&restore),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _, _) = call_as(
        &app.router,
        &everywhere,
        Method::POST,
        &format!("/{trashed_excluded}"),
        Some(&json!({ "tags": [] })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(app.store.note(&trashed).unwrap().unwrap().book_id, "trash");
    // A key for every notebook restores an ordinary one.
    let (status, answer, _) = call_as(
        &app.router,
        &everywhere,
        Method::POST,
        &format!("/{trashed}"),
        Some(&restore),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{answer}");
}
