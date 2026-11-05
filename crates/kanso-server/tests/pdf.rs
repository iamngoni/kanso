//! PDF REST and MCP contract tests.
//!
//! These exercise the real store index and both transports. PDFium rendering
//! is reserved for the annotation worker path; page reads use the indexed
//! text that the reader and search already share.

mod common;

use axum::http::Method;
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use common::*;
use kanso_core::{DocType, roles::Role};
use kanso_store::{Origin, Put};
use serde_json::{Value, json};

fn indexed_pdf(app: &App) -> (String, String) {
    let file_id = app.store.put(
        DocType::File,
        json!({
            "_id": "file:pdf-contract",
            "name": "report.pdf",
            "contentType": "application/pdf",
            "_attachments": {"index": {"content_type": "application/pdf", "data": STANDARD.encode(b"%PDF-1.4 test")}}
        }),
        Put::Upsert,
        Origin::Api,
    ).unwrap().id;
    let inbox = app.store.role_book(Role::Inbox, Origin::Api).unwrap();
    let note_id = app.store.put(
        DocType::Note,
        json!({"_id":"note:pdf-contract", "title":"Report", "bookId":inbox, "body":format!("[Report](inkdrop://{file_id})"), "kansoFiles":[file_id]}),
        Put::Upsert,
        Origin::Api,
    ).unwrap().id;
    let md5 = app.store.get(&file_id).unwrap().unwrap().fields["md5digest"]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(
        app.store
            .set_pdf_text(
                &file_id,
                &md5,
                &[
                    "Cover".into(),
                    "Contents".into(),
                    "Summary".into(),
                    "Revenue rose 12 percent".into()
                ],
                "contract-test",
            )
            .unwrap()
    );
    (note_id, file_id)
}

async fn mcp_call(app: &App, name: &str, arguments: Value) -> Value {
    let request = request(
        Method::POST,
        "/mcp",
        Some(
            &json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":name,"arguments":arguments}}),
        ),
    );
    let response = send(&app.router, request).await;
    assert_eq!(response.status, axum::http::StatusCode::OK);
    let answer = response.json();
    assert_ne!(answer["result"]["isError"], true, "{answer}");
    serde_json::from_str(answer["result"]["content"][0]["text"].as_str().unwrap()).unwrap()
}

#[tokio::test]
async fn indexed_page_and_search_match_between_rest_and_mcp() {
    let app = app();
    let (note_id, file_id) = indexed_pdf(&app);

    let uri = format!("/notes/{note_id}/pdf/{file_id}/pages/4");
    let (status, rest_page) = get(&app.router, &uri).await;
    assert_eq!(status, axum::http::StatusCode::OK);
    let mcp_page = mcp_call(
        &app,
        "kanso-read-pdf-page",
        json!({"noteId":note_id,"fileId":file_id,"page":4}),
    )
    .await;
    assert_eq!(rest_page, mcp_page);
    assert_eq!(rest_page["text"], "Revenue rose 12 percent");

    let (status, rest_search) = get(&app.router, "/pdf/search?query=revenue&limit=20").await;
    assert_eq!(status, axum::http::StatusCode::OK);
    let mcp_search = mcp_call(
        &app,
        "kanso-search-pdf",
        json!({"query":"revenue","limit":20}),
    )
    .await;
    assert_eq!(rest_search, mcp_search);
    assert_eq!(rest_search["results"][0]["page"], 4);
}
