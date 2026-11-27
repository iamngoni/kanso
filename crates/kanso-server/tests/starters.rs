//! Notebook starters over MCP and REST (issue #514): `create-book` and
//! `POST /books` with a `starter`, and `kanso-list-starters`.

mod common;

use axum::http::StatusCode;
use common::*;
use kanso_core::DocType;
use serde_json::{Value, json};

const UNKNOWN: &str = "'starter' must be one of blank, product, knowledge, side-projects, journal, reading, publishing, video-studio.";

async fn doc(app: &App, id: &str) -> Value {
    get(&app.router, &format!("/{id}")).await.1
}

#[tokio::test]
async fn create_book_with_a_starter_makes_the_layout_and_answers_the_ids() {
    let app = app();
    let made = tool_json(
        &tool(
            &app.router,
            "create-book",
            json!({ "name": "Acme", "starter": "product" }),
        )
        .await,
    );
    assert_eq!(made["ok"], true);
    assert_eq!(made["subBookIds"].as_array().unwrap().len(), 9);
    assert_eq!(made["noteIds"].as_array().unwrap().len(), 9);
    assert_eq!(made["tagIds"].as_array().unwrap().len(), 6);
    let root = doc(&app, made["id"].as_str().unwrap()).await;
    assert_eq!(root["name"], "Acme");
    assert_eq!(root["kansoIcon"], "rocket");
    let first = doc(&app, made["subBookIds"][0].as_str().unwrap()).await;
    assert_eq!(first["name"], "Features");
    assert_eq!(first["parentBookId"], made["id"]);
    let note = doc(&app, made["noteIds"][0].as_str().unwrap()).await;
    assert_eq!(note["title"], "Feature plan");
    assert_eq!(note["status"], "active");
}

#[tokio::test]
async fn rest_post_books_takes_the_same_field() {
    let app = app();
    let (status, made) = post(
        &app.router,
        "/books",
        json!({ "name": "Shelf", "starter": "reading" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{made}");
    assert_eq!(made["subBookIds"].as_array().unwrap().len(), 2);
    assert_eq!(made["tagIds"].as_array().unwrap().len(), 2);
    assert!(made["rev"].is_string());
    // The templates are in the Templates notebook, once each.
    let templates = tool_json(&tool(&app.router, "list-templates", json!({})).await);
    assert_eq!(templates.as_array().unwrap().len(), 2);
    // A second one reuses them and the tags.
    let (_, again) = post(
        &app.router,
        "/books",
        json!({ "name": "Shelf 2", "starter": "reading" }),
    )
    .await;
    assert_eq!(again["tagIds"], made["tagIds"]);
    let templates = tool_json(&tool(&app.router, "list-templates", json!({})).await);
    assert_eq!(templates.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn blank_and_no_starter_are_the_plain_notebook() {
    let app = app();
    for body in [
        json!({ "name": "A" }),
        json!({ "name": "B", "starter": "blank" }),
        json!({ "name": "C", "starter": null }),
    ] {
        let (status, made) = post(&app.router, "/books", body).await;
        assert_eq!(status, StatusCode::OK);
        assert!(made.get("subBookIds").is_none(), "{made}");
        assert!(made["id"].as_str().unwrap().starts_with("book:"));
    }
}

#[tokio::test]
async fn an_unknown_starter_is_refused_in_the_specs_words() {
    let app = app();
    for starter in [json!("nope"), json!(3), json!("Product")] {
        let (status, body) = post(
            &app.router,
            "/books",
            json!({ "name": "X", "starter": starter }),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["reason"], UNKNOWN);
    }
    let result = tool(
        &app.router,
        "create-book",
        json!({ "name": "X", "starter": "nope" }),
    )
    .await;
    assert_eq!(result["isError"], true);
    assert!(text(&result, 0).contains(UNKNOWN), "{}", text(&result, 0));
    assert!(
        app.store
            .list_api(DocType::Book, None, 0)
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn a_name_of_spaces_is_refused() {
    let app = app();
    let (status, body) = post(
        &app.router,
        "/books",
        json!({ "name": "   ", "starter": "journal" }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["reason"], "A notebook needs a name.");
}

#[tokio::test]
async fn a_starter_that_makes_notes_is_refused_in_a_locked_notebook() {
    let app = app();
    let secret =
        tool_json(&tool(&app.router, "create-book", json!({ "name": "Secret" })).await)["id"]
            .as_str()
            .unwrap()
            .to_owned();
    let mut book = app.store.get_api(&secret, None, false).unwrap();
    book.as_object_mut().unwrap().remove("count");
    book["kansoLock"] =
        json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "notebook"});
    app.store.lock_book(book, Vec::new()).unwrap();

    let message = "Starters can't be made inside a locked notebook. Make it at the top level, then move it in.";
    let (status, body) = post(
        &app.router,
        "/books",
        json!({ "name": "In", "parentBookId": secret, "starter": "product" }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    assert_eq!(body["reason"], message);
    let result = tool(
        &app.router,
        "create-book",
        json!({ "name": "In", "parentBookId": secret, "starter": "journal" }),
    )
    .await;
    assert_eq!(result["isError"], true);
    assert!(text(&result, 0).contains(message));
    // A key never sees a locked notebook, so anything else in it is refused
    // as for every write (Video studio makes no notes, so the store would take it).
    let (status, _) = post(
        &app.router,
        "/books",
        json!({ "name": "Studio", "parentBookId": secret, "starter": "video-studio" }),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn list_starters_returns_the_catalogue() {
    let app = app();
    let listed = tool_json(&tool(&app.router, "kanso-list-starters", json!({})).await);
    let starters = listed.as_array().unwrap();
    let keys: Vec<&str> = starters
        .iter()
        .map(|s| s["key"].as_str().unwrap())
        .collect();
    assert_eq!(
        keys,
        [
            "blank",
            "product",
            "knowledge",
            "side-projects",
            "journal",
            "reading",
            "publishing",
            "video-studio"
        ]
    );
    let product = &starters[1];
    assert_eq!(product["label"], "Product development");
    assert_eq!(product["makesNotes"], true);
    assert_eq!(product["subNotebooks"].as_array().unwrap().len(), 9);
    assert_eq!(product["subNotebooks"][1]["name"], "Bugs");
    assert_eq!(product["subNotebooks"][1]["template"], "Bug report");
    assert_eq!(product["tags"][0], json!({"name": "web", "color": "blue"}));
    assert_eq!(product["templates"][1], "Bug report");
    assert_eq!(starters[0]["makesNotes"], false);
    assert_eq!(
        starters[7]["templates"],
        json!(["Video brief", "Shot list"])
    );
    // Every key it lists is one create-book accepts.
    for key in keys {
        let (status, _) = post(
            &app.router,
            "/books",
            json!({ "name": format!("From {key}"), "starter": key }),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{key}");
    }
}
