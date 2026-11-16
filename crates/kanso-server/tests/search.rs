//! Keyword search through `/mcp` and `GET /notes` (#449): unquoted words
//! are ANDed, and no keyword ever makes the search fail.

mod common;

use axum::http::StatusCode;
use common::*;
use serde_json::{Value, json};

async fn seed(app: &App) {
    let book = tool_json(&tool(&app.router, "create-book", json!({ "name": "Work" })).await);
    let book_id = book["_id"]
        .as_str()
        .or(book["id"].as_str())
        .unwrap()
        .to_owned();
    for (title, body) in [
        (
            "BuzzFi daily audit",
            "Checked the BuzzFi ledger for the daily audit.",
        ),
        ("Weekly audit", "Nothing to see here, only weekly."),
    ] {
        tool_json(
            &tool(
                &app.router,
                "create-note",
                json!({ "bookId": book_id, "title": title, "body": body, "status": "none" }),
            )
            .await,
        );
    }
}

fn titles(found: &Value) -> Vec<String> {
    let mut titles: Vec<String> = found
        .as_array()
        .unwrap_or_else(|| panic!("not a list: {found}"))
        .iter()
        .map(|n| n["title"].as_str().unwrap().to_owned())
        .collect();
    titles.sort();
    titles
}

async fn search(app: &App, keyword: &str) -> Value {
    tool(&app.router, "search-notes", json!({ "keyword": keyword })).await
}

#[tokio::test]
async fn unquoted_words_are_anded() {
    let app = app();
    seed(&app).await;
    for keyword in ["daily audit", "BuzzFi daily audit", "audit   ledger"] {
        let found = tool_json(&search(&app, keyword).await);
        assert_eq!(titles(&found), ["BuzzFi daily audit"], "{keyword}");
    }
    let found = tool_json(&search(&app, "\"BuzzFi daily audit\"").await);
    assert_eq!(titles(&found), ["BuzzFi daily audit"]);
    let found = tool_json(&search(&app, "audit").await);
    assert_eq!(titles(&found), ["BuzzFi daily audit", "Weekly audit"]);
    let found = tool_json(&search(&app, "audit weekly").await);
    assert_eq!(titles(&found), ["Weekly audit"]);
}

#[tokio::test]
async fn words_mix_with_filters_quotes_and_punctuation() {
    let app = app();
    seed(&app).await;
    let found = tool_json(&search(&app, "ledger audit status:none").await);
    assert_eq!(titles(&found), ["BuzzFi daily audit"]);
    let found = tool_json(&search(&app, "daily tag:Missing audit").await);
    assert!(found.as_array().unwrap().is_empty(), "{found}");
    let found = tool_json(&search(&app, "\"daily audit\" BuzzFi").await);
    assert_eq!(titles(&found), ["BuzzFi daily audit"]);
    for keyword in [
        "daily, audit.",
        "daily-audit!",
        "BuzzFi's (daily) audit?",
        "\"unclosed daily audit",
        "daily AND audit OR NOT -x",
        "a* b: c^ d\" * : - +",
        "NEAR(daily audit)",
        "日本 語 audit",
    ] {
        let result = search(&app, keyword).await;
        assert_ne!(result["isError"], true, "{keyword}: {result}");
        let (status, body) = get(&app.router, &format!("/notes?keyword={}", encode(keyword))).await;
        assert_eq!(status, StatusCode::OK, "{keyword}: {body}");
    }
}

#[tokio::test]
async fn rest_search_ands_unquoted_words() {
    let app = app();
    seed(&app).await;
    let (status, found) = get(
        &app.router,
        &format!("/notes?keyword={}", encode("daily audit")),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{found}");
    assert_eq!(titles(&found), ["BuzzFi daily audit"]);
}

/// The report in #449: the search index damaged while `kanso serve` runs.
/// `BuzzFi` and the quoted phrase missed the damage; `daily audit` hit it
/// and came back as `Fetch error [500]`. It is repaired and answered now.
#[tokio::test]
async fn a_damaged_index_is_repaired_rather_than_a_500() {
    let app = app();
    seed(&app).await;
    let conn = rusqlite::Connection::open(app.store.path()).unwrap();
    conn.execute(
        "DELETE FROM notes_fts_content WHERE c1 = 'Weekly audit'",
        [],
    )
    .unwrap();
    drop(conn);
    let found = tool_json(&search(&app, "BuzzFi").await);
    assert_eq!(titles(&found), ["BuzzFi daily audit"]);
    let found = tool_json(&search(&app, "BuzzFi daily audit").await);
    assert_eq!(titles(&found), ["BuzzFi daily audit"]);
    let found = tool_json(&search(&app, "audit").await);
    assert_eq!(titles(&found), ["BuzzFi daily audit", "Weekly audit"]);
    let (status, found) = get(
        &app.router,
        &format!("/notes?keyword={}", encode("audit weekly")),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{found}");
    assert_eq!(titles(&found), ["Weekly audit"]);
}
