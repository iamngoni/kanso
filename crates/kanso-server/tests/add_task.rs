//! `kanso-add-task`, `kanso-parse-when` and `repeat` in `kanso-list-tasks`
//! over `/mcp` (issue #390, step 10). Dates in the examples are written out
//! in full, so no test depends on the day it runs. The properties pin their
//! seed.

mod common;

use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, header};
use common::*;
use kanso_core::DocType;
use kanso_core::repeat::parse_repeat;
use kanso_server::KeyAccess;
use kanso_store::{Origin, Put, Store};
use proptest::prelude::*;
use proptest::test_runner::{Config, RngAlgorithm, RngSeed};
use serde_json::{Value, json};

const LIMITED: &str = "kcl_LLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLL";
const READER: &str = "kcl_RRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRR";

fn put(store: &Store, doc_type: DocType, doc: Value) -> String {
    store
        .put(doc_type, doc, Put::Upsert, Origin::Sync)
        .unwrap()
        .id
}

fn book(store: &Store, name: &str) -> String {
    put(store, DocType::Book, json!({ "name": name }))
}

fn note(store: &Store, book: &str, body: &str) -> String {
    put(
        store,
        DocType::Note,
        json!({ "title": "Plan", "body": body, "bookId": book }),
    )
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

async fn tool_as(router: &Router, key: &str, name: &str, arguments: Value) -> Value {
    let message = json!({
        "jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": { "name": name, "arguments": arguments },
    });
    let request = Request::builder()
        .method(Method::POST)
        .uri("/mcp")
        .header(header::AUTHORIZATION, bearer(key))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(message.to_string()))
        .unwrap();
    let answer = send(router, request).await.json();
    answer["result"].clone()
}

fn error_text(result: &Value) -> String {
    assert_eq!(result["isError"], true, "{result}");
    text(result, 0).to_owned()
}

fn body(store: &Store, id: &str) -> String {
    store.note(id).unwrap().unwrap().body
}

#[tokio::test]
async fn parse_when_reads_a_date_time_and_repeat() {
    let app = app();
    let got = tool_json(
        &tool(
            &app.router,
            "kanso-parse-when",
            json!({ "text": "film Pixel review 2030-03-04 3pm every week" }),
        )
        .await,
    );
    assert_eq!(
        got,
        json!({
            "text": "film Pixel review",
            "due": "2030-03-04T15:00",
            "repeat": "every week",
            "label": "Mon 4 Mar 2030, 15:00 \u{b7} every week",
        })
    );
    // A day and no time, and no repeat.
    let got = tool_json(
        &tool(
            &app.router,
            "kanso-parse-when",
            json!({ "text": "pay rent @2030-03-04" }),
        )
        .await,
    );
    assert_eq!(got["text"], "pay rent");
    assert_eq!(got["due"], "2030-03-04");
    assert_eq!(got["repeat"], Value::Null);
    // The order of a date with a slash is the caller's.
    for (text, order, reads) in [
        ("x 14/11", None, true),
        ("x 14/11", Some("mdy"), false),
        ("x 11/14", Some("mdy"), true),
    ] {
        let mut args = json!({ "text": text });
        if let Some(order) = order {
            args["dateOrder"] = json!(order);
        }
        let got = tool_json(&tool(&app.router, "kanso-parse-when", args).await);
        assert_eq!(
            got["due"].as_str().is_some_and(|d| d.ends_with("-11-14")),
            reads,
            "{text} {order:?}: {got}"
        );
    }
}

#[tokio::test]
async fn parse_when_leaves_other_text_alone() {
    let app = app();
    for input in [
        "call Fri about the fri shoot",
        "water plants every week",
        "just a task",
    ] {
        let got = tool_json(&tool(&app.router, "kanso-parse-when", json!({ "text": input })).await);
        assert_eq!(
            got,
            json!({ "text": input, "due": null, "repeat": null, "label": null }),
            "{input}"
        );
    }
    let relative = tool_json(
        &tool(
            &app.router,
            "kanso-parse-when",
            json!({ "text": "ring Sam tomorrow 9am" }),
        )
        .await,
    );
    assert_eq!(relative["text"], "ring Sam");
    assert!(relative["due"].as_str().unwrap().ends_with("T09:00"));
    let bad = tool(&app.router, "kanso-parse-when", json!({})).await;
    assert_eq!(bad["isError"], true);
}

#[tokio::test]
async fn add_task_appends_a_line_and_list_tasks_reads_it_back() {
    let app = app();
    let shelf = book(&app.store, "Work");
    let plan = note(&app.store, &shelf, "Intro\n- [ ] first\n");
    let added = tool_json(
        &tool(
            &app.router,
            "kanso-add-task",
            json!({ "noteId": plan, "text": "film Pixel review 2030-03-04 3pm every week" }),
        )
        .await,
    );
    assert_eq!(added["noteId"], json!(plan));
    assert_eq!(added["line"], 2);
    assert_eq!(
        added["task"],
        "- [ ] film Pixel review @2030-03-04T15:00 every week"
    );
    assert_eq!(added["due"], "2030-03-04T15:00");
    assert_eq!(added["repeat"], "every week");
    assert_eq!(
        body(&app.store, &plan),
        "Intro\n- [ ] first\n- [ ] film Pixel review @2030-03-04T15:00 every week\n"
    );
    let listed = tool_json(&tool(&app.router, "kanso-list-tasks", json!({})).await);
    let mine = listed
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["line"] == 2)
        .unwrap();
    assert_eq!(mine["due"], "2030-03-04T15:00");
    assert_eq!(mine["repeat"], "every week");
    assert_eq!(
        mine["text"],
        "film Pixel review @2030-03-04T15:00 every week"
    );
    let first = listed
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["line"] == 1)
        .unwrap();
    assert_eq!(first["repeat"], Value::Null);
    // Ticking it goes through the task's own line, as listed.
    let done = tool(
        &app.router,
        "kanso-set-task-done",
        json!({ "noteId": plan, "line": 2, "text": mine["text"], "done": true }),
    )
    .await;
    assert_ne!(done["isError"], true, "{done}");
}

#[tokio::test]
async fn a_task_with_no_date_is_added_as_written() {
    let app = app();
    let shelf = book(&app.store, "Work");
    let plan = note(&app.store, &shelf, "");
    let added = tool_json(
        &tool(
            &app.router,
            "kanso-add-task",
            json!({ "noteId": plan, "text": "  buy milk  " }),
        )
        .await,
    );
    assert_eq!(added["line"], 0);
    assert_eq!(added["due"], Value::Null);
    assert_eq!(added["repeat"], Value::Null);
    assert_eq!(added["label"], Value::Null);
    assert_eq!(body(&app.store, &plan), "- [ ] buy milk\n");
}

#[tokio::test]
async fn a_task_goes_to_the_daily_note_by_default_and_to_the_inbox_on_request() {
    let app = app();
    let daily = tool_json(
        &tool(
            &app.router,
            "kanso-add-task",
            json!({ "text": "stretch 2030-03-04" }),
        )
        .await,
    );
    let id = daily["noteId"].as_str().unwrap();
    assert!(id.starts_with("note:daily-"), "{id}");
    assert_eq!(daily["line"], 0);
    let again = tool_json(
        &tool(
            &app.router,
            "kanso-add-task",
            json!({ "text": "walk", "target": "daily" }),
        )
        .await,
    );
    assert_eq!(again["noteId"], daily["noteId"]);
    assert_eq!(again["line"], 1);
    let inbox = tool_json(
        &tool(
            &app.router,
            "kanso-add-task",
            json!({ "text": "sort post", "target": "inbox" }),
        )
        .await,
    );
    assert_ne!(inbox["noteId"], daily["noteId"]);
    let saved = app
        .store
        .note(inbox["noteId"].as_str().unwrap())
        .unwrap()
        .unwrap();
    assert_eq!(saved.title, "Tasks");
    assert_eq!(saved.book_id, "book:inbox");
    assert_eq!(saved.body, "- [ ] sort post\n");
}

#[tokio::test]
async fn add_task_refuses_what_it_cannot_write() {
    let app = app();
    let shelf = book(&app.store, "Work");
    let plan = note(&app.store, &shelf, "- [ ] a\n");
    let cases = [
        (
            json!({ "text": "x", "noteId": plan, "target": "inbox" }),
            "not both",
        ),
        (json!({ "text": "x", "noteId": "note:nope" }), "not found"),
        (json!({ "text": "two\nlines", "noteId": plan }), "one line"),
        (json!({ "text": "fri 3pm", "noteId": plan }), "only a date"),
        (json!({ "text": "", "noteId": plan }), "Invalid"),
        (json!({ "text": "x", "target": "nowhere" }), "Invalid"),
    ];
    for (arguments, expected) in cases {
        let got = error_text(&tool(&app.router, "kanso-add-task", arguments.clone()).await);
        assert!(
            got.to_lowercase().contains(&expected.to_lowercase()),
            "{arguments}: {got}"
        );
    }
    assert_eq!(body(&app.store, &plan), "- [ ] a\n");
}

#[tokio::test]
async fn a_locked_note_gets_no_task_and_does_not_show_in_the_error() {
    let app = app();
    let shelf = book(&app.store, "Work");
    let secret = note(&app.store, &shelf, "sealed canary words\n");
    lock(&app.store, &secret);
    let got = error_text(
        &tool(
            &app.router,
            "kanso-add-task",
            json!({ "noteId": secret, "text": "leak" }),
        )
        .await,
    );
    assert!(!got.contains("canary"), "{got}");
    assert!(!body(&app.store, &secret).contains("leak"));
    // A note in a locked notebook is as unreachable.
    let vault = put(
        &app.store,
        DocType::Book,
        json!({ "name": "Vault", "kansoLock": { "salt": "c2FsdA" } }),
    );
    let vaulted = note(&app.store, &vault, "vaulted\n");
    let got = error_text(
        &tool(
            &app.router,
            "kanso-add-task",
            json!({ "noteId": vaulted, "text": "leak" }),
        )
        .await,
    );
    assert!(!got.contains("vaulted"), "{got}");
    assert!(!body(&app.store, &vaulted).contains("leak"));
}

#[tokio::test]
async fn a_key_limited_to_notebooks_adds_only_inside_them() {
    let app = app();
    let mine = book(&app.store, "Mine");
    let theirs = book(&app.store, "Theirs");
    let inside = note(&app.store, &mine, "");
    let outside = note(&app.store, &theirs, "");
    save_key(
        &app.store,
        LIMITED,
        KeyAccess::Write,
        Some(vec![mine.clone()]),
    );
    let keys = kanso_server::KeyStore::open(app.store.clone()).unwrap();
    let router = kanso_server::router(app.store.clone(), &config(), keys);
    let ok = tool_as(
        &router,
        LIMITED,
        "kanso-add-task",
        json!({ "noteId": inside, "text": "ok" }),
    )
    .await;
    assert_ne!(ok["isError"], true, "{ok}");
    let hidden = error_text(
        &tool_as(
            &router,
            LIMITED,
            "kanso-add-task",
            json!({ "noteId": outside, "text": "no" }),
        )
        .await,
    );
    assert!(hidden.to_lowercase().contains("not found"), "{hidden}");
    assert_eq!(body(&app.store, &outside), "");
    // The Journal and the Inbox are not among its notebooks.
    for target in ["daily", "inbox"] {
        let got = error_text(
            &tool_as(
                &router,
                LIMITED,
                "kanso-add-task",
                json!({ "text": "no", "target": target }),
            )
            .await,
        );
        assert!(got.contains("limited to some notebooks"), "{got}");
    }
}

#[tokio::test]
async fn a_read_key_parses_but_cannot_add() {
    let app = app();
    let shelf = book(&app.store, "Work");
    let plan = note(&app.store, &shelf, "");
    save_key(&app.store, READER, KeyAccess::Read, None);
    let keys = kanso_server::KeyStore::open(app.store.clone()).unwrap();
    let router = kanso_server::router(app.store.clone(), &config(), keys);
    let parsed = tool_as(
        &router,
        READER,
        "kanso-parse-when",
        json!({ "text": "x 2030-03-04" }),
    )
    .await;
    assert_ne!(parsed["isError"], true, "{parsed}");
    let refused = tool_as(
        &router,
        READER,
        "kanso-add-task",
        json!({ "noteId": plan, "text": "no" }),
    )
    .await;
    assert_eq!(refused["isError"], true, "{refused}");
    assert_eq!(body(&app.store, &plan), "");
}

const REPEATS: [&str; 9] = [
    "every day",
    "every week",
    "every weekday",
    "every month",
    "every 2 weeks",
    "every 3 days after done",
    "every month on the last day",
    "every mon and thu",
    "every year",
];

proptest! {
    #![proptest_config(Config {
        cases: 48,
        rng_seed: RngSeed::Fixed(0x0074_6173_6b31),
        rng_algorithm: RngAlgorithm::ChaCha,
        failure_persistence: None,
        ..Config::default()
    })]

    /// What `kanso-add-task` writes, `kanso-list-tasks` reads back with the
    /// same due date and repeat, and `kanso-parse-when` read them the same
    /// way from the same text.
    #[test]
    fn prop_add_task_then_list_tasks_agree(
        words in "q[a-z]{3,7}( q[a-z]{3,7}){0,2}",
        year in 2030i32..2040,
        month in 1u32..=12,
        day in 1u32..=28,
        time in prop::option::of((0u32..24, 0u32..60)),
        repeat in prop::option::of(prop::sample::select(REPEATS.to_vec())),
    ) {
        let (due, input) = {
            let mut due = format!("{year:04}-{month:02}-{day:02}");
            if let Some((h, m)) = time {
                due.push_str(&format!("T{h:02}:{m:02}"));
            }
            let mut input = format!("{words} {}", due.replace('T', " "));
            if let Some(repeat) = repeat {
                input.push(' ');
                input.push_str(repeat);
            }
            (due, input)
        };
        let expected_repeat = repeat.map(|r| parse_repeat(r).unwrap().0.to_string());
        let app = app();
        let shelf = book(&app.store, "Work");
        let plan = note(&app.store, &shelf, "- [ ] before\n");
        let (parsed, added, listed) = runtime().block_on(async {
            let parsed = tool_json(
                &tool(&app.router, "kanso-parse-when", json!({ "text": input })).await,
            );
            let added = tool_json(
                &tool(
                    &app.router,
                    "kanso-add-task",
                    json!({ "noteId": plan, "text": input }),
                )
                .await,
            );
            let listed = tool_json(
                &tool(&app.router, "kanso-list-tasks", json!({ "status": "all" })).await,
            );
            (parsed, added, listed)
        });
        prop_assert_eq!(&parsed["text"], &json!(words));
        prop_assert_eq!(&parsed["due"], &json!(due));
        prop_assert_eq!(&parsed["repeat"], &json!(expected_repeat));
        prop_assert_eq!(&added["due"], &parsed["due"]);
        prop_assert_eq!(&added["repeat"], &parsed["repeat"]);
        prop_assert_eq!(&added["label"], &parsed["label"]);
        prop_assert_eq!(&added["line"], &json!(1));
        let listed = listed
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["noteId"] == json!(plan) && t["line"] == 1)
            .cloned()
            .unwrap();
        prop_assert_eq!(&listed["due"], &json!(due));
        prop_assert_eq!(&listed["repeat"], &json!(expected_repeat));
        let line = body(&app.store, &plan).lines().nth(1).unwrap().to_owned();
        prop_assert_eq!(line, added["task"].as_str().unwrap());
    }
}
