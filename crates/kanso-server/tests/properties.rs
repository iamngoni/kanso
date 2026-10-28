//! Kanso HTTP API and MCP property tests. CI pins the seed
//! with `PROPTEST_RNG_SEED`.

mod common;

use std::sync::{Arc, OnceLock};

use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use common::*;
use kanso_core::tasks::count_tasks;
use kanso_core::{DocType, NoteStatus};
use kanso_server::{
    AgentScope, READ_TOOLS, ServerConfig, TokenStore, basic_header, router_with_tokens,
};
use kanso_store::{Origin, Put, Store, StoreError};
use proptest::prelude::*;
use proptest::test_runner::{FileFailurePersistence, TestCaseError};
use serde_json::{Map, Value, json};

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

/// Fixed documents with explicit timestamps, so two stores seeded alike hold
/// identical revisions.
fn seed(store: &Store) {
    let docs = [
        (DocType::Book, json!({"_id": "book:b0", "name": "Alpha"})),
        (
            DocType::Book,
            json!({"_id": "book:b1", "name": "beta Book", "parentBookId": "book:b0"}),
        ),
        (
            DocType::Tag,
            json!({"_id": "tag:t0", "name": "red", "color": "red"}),
        ),
        (DocType::Tag, json!({"_id": "tag:t1", "name": "Green"})),
        (
            DocType::Note,
            json!({"_id": "note:n0", "bookId": "book:b0", "title": "N0", "body": "- [ ] x", "tags": ["tag:t0"]}),
        ),
        (
            DocType::Note,
            json!({"_id": "note:n1", "bookId": "trash", "title": "N1", "body": ""}),
        ),
        (
            DocType::File,
            json!({"_id": "file:f0", "name": "p.png", "contentType": "image/png",
                   "_attachments": {"index": {"content_type": "image/png", "data": STANDARD.encode(PNG)}}}),
        ),
    ];
    for (doc_type, mut doc) in docs {
        doc["createdAt"] = json!(1_700_000_000_000_i64);
        doc["updatedAt"] = json!(1_700_000_000_000_i64);
        store.put(doc_type, doc, Put::Upsert, Origin::Api).unwrap();
    }
}

fn seeded_app() -> App {
    let app = app();
    seed(&app.store);
    app
}

// ---------------------------------------------------------------------------
// Auth refinement

#[derive(Clone, Debug)]
enum Attempt {
    NoHeader,
    Raw(String),
    Creds(String, String),
}

fn attempt(user: String, pass: String) -> impl Strategy<Value = Attempt> {
    let exact = Attempt::Creds(user.clone(), pass.clone());
    prop_oneof![
        Just(exact),
        Just(Attempt::NoHeader),
        "[ -~]{0,24}".prop_map(Attempt::Raw),
        ("[^:]{0,10}", ".{0,10}").prop_map(|(u, p)| Attempt::Creds(u, p)),
        (0..4usize, any::<char>()).prop_map(move |(kind, c)| {
            let (mut u, mut p) = (user.clone(), pass.clone());
            match kind {
                0 => u.push(c),
                1 => p.push(c),
                2 => {
                    u.pop();
                }
                _ => {
                    p.pop();
                }
            }
            Attempt::Creds(u, p)
        }),
    ]
}

fn shared_store() -> Arc<Store> {
    static STORE: OnceLock<(tempfile::TempDir, Arc<Store>)> = OnceLock::new();
    STORE
        .get_or_init(|| {
            let dir = tempfile::tempdir().unwrap();
            let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
            (dir, store)
        })
        .1
        .clone()
}

proptest! {
    #![proptest_config(config(128))]

    /// Auth refinement: only the configured username and password get past
    /// authentication; every other pair, a missing header or garbage gets 401.
    #[test]
    fn auth_refinement(
        (user, pass, attempt) in ("[^:]{0,10}", ".{0,10}")
            .prop_flat_map(|(u, p)| (Just(u.clone()), Just(p.clone()), attempt(u, p)))
    ) {
        let config = ServerConfig { username: user.clone(), password: pass.clone(), ..common::config() };
        let router = kanso_server::router(shared_store(), &config);
        let header_value = match &attempt {
            Attempt::NoHeader => None,
            Attempt::Raw(raw) => Some(raw.clone()),
            Attempt::Creds(u, p) => Some(basic_header(u, p)),
        };
        let exact = match &attempt {
            Attempt::Creds(u, p) => *u == user && *p == pass,
            Attempt::Raw(raw) => raw.trim() == basic_header(&user, &pass),
            Attempt::NoHeader => false,
        };
        block_on(async move {
            for (method, uri) in [(Method::GET, "/"), (Method::GET, "/note:missing"), (Method::POST, "/mcp")] {
                let mut request = Request::builder().method(method).uri(uri);
                if let Some(value) = &header_value {
                    request = request.header(header::AUTHORIZATION, value);
                }
                let body = r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#;
                let response = send(&router, request.body(Body::from(body)).unwrap()).await;
                prop_assert_eq!(response.status == StatusCode::UNAUTHORIZED, !exact, "{} {:?}", uri, attempt);
            }
            Ok(())
        })?;
    }
}

proptest! {
    #![proptest_config(config(16))]

    /// Scoped Kanso reads never return a locked note's title or body text.
    #[test]
    fn prop_scoped_native_reads_never_return_locked_note_text_or_title(
        title in "locked-title-[a-z]{1,16}",
        secret in "locked-body-[a-z]{1,16}",
    ) {
        let app = app();
        let templates = app.store.role_book(kanso_core::roles::Role::Templates, Origin::Sync).unwrap();
        let note_id = app.store.put(
            DocType::Note,
            json!({ "title": title, "body": format!("---\\nsecret: {secret}\\n---\\n{secret}"), "bookId": templates }),
            Put::Upsert,
            Origin::Sync,
        ).unwrap().id;
        app.store.put(
            DocType::Note,
            json!({ "_id": note_id, "kansoLock": { "salt": "test-salt" } }),
            Put::Merge,
            Origin::Sync,
        ).unwrap();

        let tokens = TokenStore::new();
        let minted = tokens.mint(AgentScope::with_tools(None, &READ_TOOLS));
        let router = router_with_tokens(app.store.clone(), &common::config(), tokens);
        let hidden_title = title.clone();
        let hidden_body = secret.clone();
        block_on(async move {
            let calls = [
                ("kanso-list-tags", json!({})),
                ("list-templates", json!({})),
                ("kanso-get-properties", json!({ "noteId": note_id })),
                ("kanso-list-note-links", json!({ "noteId": note_id })),
                ("kanso-list-backlinks", json!({ "noteId": note_id })),
                ("kanso-run-query-view", json!({ "noteId": note_id })),
                ("kanso-list-versions", json!({ "noteId": note_id })),
                ("kanso-read-version", json!({ "noteId": note_id, "rev": "missing" })),
            ];
            for (name, arguments) in calls {
                let request = Request::builder()
                    .method(Method::POST)
                    .uri("/mcp")
                    .header(header::AUTHORIZATION, format!("Bearer {}", minted.token))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(json!({
                        "jsonrpc": "2.0", "id": 1, "method": "tools/call",
                        "params": { "name": name, "arguments": arguments }
                    }).to_string())).unwrap();
                let answer = send(&router, request).await.json();
                let rendered = answer.to_string();
                prop_assert!(!rendered.contains(&hidden_title), "{name}: {rendered}");
                prop_assert!(!rendered.contains(&hidden_body), "{name}: {rendered}");
            }
            Ok(())
        })?;
    }
}

// ---------------------------------------------------------------------------
// HTTP round-trip

#[derive(Clone, Debug)]
struct NoteSpec {
    id: Option<String>,
    title: String,
    body: String,
    book: &'static str,
    tags: Vec<&'static str>,
    status: NoteStatus,
    share: &'static str,
    pinned: bool,
    extra: Option<Value>,
}

fn note_spec() -> impl Strategy<Value = NoteSpec> {
    (
        proptest::option::of("note:[A-Za-z0-9_-]{1,12}"),
        ".{0,40}",
        "(.|\n|- \\[[ x]\\] ){0,40}",
        prop::sample::select(vec!["book:b0", "book:b1", "trash"]),
        prop::sample::subsequence(vec!["tag:t0", "tag:t1"], 0..=2),
        prop::sample::select(NoteStatus::ALL.to_vec()),
        prop::sample::select(vec!["private", "public"]),
        any::<bool>(),
        proptest::option::of(prop_oneof![
            ".{0,10}".prop_map(Value::from),
            any::<i32>().prop_map(Value::from),
            Just(json!({"nested": [1, "two", null]})),
        ]),
    )
        .prop_map(
            |(id, title, body, book, tags, status, share, pinned, extra)| NoteSpec {
                id,
                title,
                body,
                book,
                tags,
                status,
                share,
                pinned,
                extra,
            },
        )
}

proptest! {
    #![proptest_config(config(64))]

    /// HTTP round-trip: POST a note, then GET returns the same fields, the
    /// write's rev, and the task counts derived from the body.
    #[test]
    fn http_round_trip(spec in note_spec()) {
        let app = seeded_app();
        block_on(async move {
            let mut doc = json!({
                "title": spec.title, "body": spec.body, "bookId": spec.book, "tags": spec.tags,
                "status": spec.status.as_str(), "share": spec.share, "pinned": spec.pinned,
            });
            if let Some(id) = &spec.id {
                doc["_id"] = json!(id);
            }
            if let Some(extra) = &spec.extra {
                doc["sourceTemplateId"] = extra.clone();
            }
            let (status, res) = post(&app.router, "/notes", doc.clone()).await;
            prop_assert_eq!(status, StatusCode::OK, "{}", res);
            let id = res["id"].as_str().unwrap().to_owned();
            if let Some(given) = &spec.id {
                prop_assert_eq!(&id, given);
            }
            let (status, got) = get(&app.router, &format!("/{}", encode(&id))).await;
            prop_assert_eq!(status, StatusCode::OK);
            prop_assert_eq!(&got["_id"], &json!(id));
            prop_assert_eq!(&got["_rev"], &res["rev"]);
            for key in ["title", "body", "bookId", "tags", "status", "share", "pinned"] {
                prop_assert_eq!(&got[key], &doc[key], "{}", key);
            }
            if let Some(extra) = &spec.extra {
                prop_assert_eq!(&got["sourceTemplateId"], extra);
            }
            let tasks = count_tasks(&spec.body);
            prop_assert_eq!(&got["numOfTasks"], &json!(tasks.total));
            prop_assert_eq!(&got["numOfCheckedTasks"], &json!(tasks.checked));
            prop_assert_eq!(&got["doctype"], &json!("markdown"));
            Ok(())
        })?;
    }
}

// ---------------------------------------------------------------------------
// Status-code invariant

#[derive(Clone, Copy, Debug)]
enum RevPick {
    Absent,
    Current,
    Stale,
    Garbage,
}

#[derive(Clone, Debug)]
enum Op {
    Get {
        id: &'static str,
        rev: RevPick,
    },
    Upsert {
        doc_type: DocType,
        id: Option<&'static str>,
        rev: RevPick,
        body: Value,
    },
    Merge {
        id: &'static str,
        rev: RevPick,
        body: Value,
    },
    Delete {
        id: &'static str,
    },
}

const IDS: [&str; 13] = [
    "note:n0",
    "note:n1",
    "note:missing",
    "book:b0",
    "book:b1",
    "book:missing",
    "tag:t0",
    "tag:missing",
    "file:f0",
    "note:",
    "bogus",
    "note:bad!id",
    "tag:n0",
];

/// Request bodies mixing valid and invalid values for every document kind.
fn body() -> impl Strategy<Value = Value> {
    let field = |key: &'static str, values: Vec<Value>| {
        proptest::option::of(prop::sample::select(values)).prop_map(move |v| (key, v))
    };
    let fields = (
        field(
            "bookId",
            vec![
                json!("book:b0"),
                json!("book:missing"),
                json!("trash"),
                json!(7),
            ],
        ),
        field("title", vec![json!("T"), json!("x".repeat(129)), json!(5)]),
        field("body", vec![json!(""), json!("b"), json!(null)]),
        field("status", vec![json!("active"), json!("later")]),
        field(
            "tags",
            vec![json!(["tag:t0"]), json!(["tag:missing"]), json!("tag:t0")],
        ),
        field(
            "name",
            vec![json!("Name"), json!(""), json!("n".repeat(65))],
        ),
        field(
            "parentBookId",
            vec![json!("book:b0"), json!("book:missing"), json!(null)],
        ),
        field("color", vec![json!("blue"), json!("magenta")]),
        field("pinned", vec![json!(true), json!("yes")]),
        field("contentType", vec![json!("image/png"), json!("text/plain")]),
    );
    prop_oneof![
        9 => fields.prop_map(|fields| {
            let (a, b, c, d, e, f, g, h, i, j) = fields;
            let mut map = Map::new();
            for (key, value) in [a, b, c, d, e, f, g, h, i, j] {
                if let Some(value) = value {
                    map.insert(key.to_owned(), value);
                }
            }
            Value::Object(map)
        }),
        1 => prop::sample::select(vec![json!([1, 2]), json!("text"), json!(3), json!(null)]),
    ]
}

fn op() -> impl Strategy<Value = Op> {
    let id = || prop::sample::select(IDS.to_vec());
    let rev = || {
        prop::sample::select(vec![
            RevPick::Absent,
            RevPick::Current,
            RevPick::Stale,
            RevPick::Garbage,
        ])
    };
    let doc_type = prop::sample::select(DocType::ALL.to_vec());
    prop_oneof![
        (id(), rev()).prop_map(|(id, rev)| Op::Get { id, rev }),
        (doc_type, proptest::option::of(id()), rev(), body()).prop_map(
            |(doc_type, id, rev, body)| Op::Upsert {
                doc_type,
                id,
                rev,
                body
            }
        ),
        (id(), rev(), body()).prop_map(|(id, rev, body)| Op::Merge { id, rev, body }),
        id().prop_map(|id| Op::Delete { id }),
    ]
}

fn rev_value(store: &Store, id: Option<&str>, pick: RevPick) -> Option<String> {
    match pick {
        RevPick::Absent => None,
        RevPick::Current => Some(id.and_then(|id| store.get(id).ok().flatten()).map_or_else(
            || "1-00000000000000000000000000000000".to_owned(),
            |d| d.rev,
        )),
        RevPick::Stale => Some("1-00000000000000000000000000000000".to_owned()),
        RevPick::Garbage => Some("garbage".to_owned()),
    }
}

fn with_fields(mut body: Value, id: Option<&str>, rev: Option<&str>) -> Value {
    if let Value::Object(map) = &mut body {
        if let Some(id) = id {
            map.insert("_id".into(), json!(id));
        }
        if let Some(rev) = rev {
            map.insert("_rev".into(), json!(rev));
        }
    }
    body
}

/// What the store itself says about an operation, called directly.
fn direct(store: &Store, op: &Op) -> Result<Option<Value>, StoreError> {
    let typed = |id: &str| DocType::of_id(id).ok_or(StoreError::NotFound);
    match op {
        Op::Get { id, rev } => {
            typed(id)?;
            let rev = rev_value(store, Some(id), *rev);
            store.get_api(id, rev.as_deref(), false).map(Some)
        }
        Op::Upsert {
            doc_type,
            id,
            rev,
            body,
        } => {
            let rev = rev_value(store, *id, *rev);
            let body = with_fields(body.clone(), *id, rev.as_deref());
            store
                .put(*doc_type, body, Put::Upsert, Origin::Api)
                .map(|_| None)
        }
        Op::Merge { id, rev, body } => {
            let doc_type = typed(id)?;
            let rev = rev_value(store, Some(id), *rev);
            let body = with_fields(body.clone(), Some(id), rev.as_deref());
            store
                .put(doc_type, body, Put::Merge, Origin::Api)
                .map(|_| None)
        }
        Op::Delete { id } => {
            typed(id)?;
            store.delete(id, Origin::Api).map(|_| None)
        }
    }
}

async fn over_http(router: &Router, store: &Store, op: &Op) -> (StatusCode, Value) {
    match op {
        Op::Get { id, rev } => {
            let query = rev_value(store, Some(id), *rev)
                .map(|rev| format!("?rev={}", encode(&rev)))
                .unwrap_or_default();
            get(router, &format!("/{id}{query}")).await
        }
        Op::Upsert {
            doc_type,
            id,
            rev,
            body,
        } => {
            let rev = rev_value(store, *id, *rev);
            let body = with_fields(body.clone(), *id, rev.as_deref());
            post(router, &format!("/{}", doc_type.collection()), body).await
        }
        Op::Merge { id, rev, body } => {
            let rev = rev_value(store, Some(id), *rev);
            post(
                router,
                &format!("/{id}"),
                with_fields(body.clone(), None, rev.as_deref()),
            )
            .await
        }
        Op::Delete { id } => call(router, Method::DELETE, &format!("/{id}"), None).await,
    }
}

/// Replaces document IDs quoted in an error reason with `<id>`.
fn mask_quoted_ids(body: &mut Value) {
    if let Some(reason) = body["reason"].as_str() {
        let masked: Vec<String> = reason
            .split('\'')
            .enumerate()
            .map(|(i, part)| {
                if i % 2 == 1 && DocType::of_id(part).is_some() {
                    "<id>".to_owned()
                } else {
                    part.to_owned()
                }
            })
            .collect();
        body["reason"] = json!(masked.join("'"));
    }
}

/// The documented mapping from store errors to HTTP answers.
fn expected_error(err: &StoreError) -> (StatusCode, Value) {
    match err {
        StoreError::NotFound => (
            StatusCode::NOT_FOUND,
            json!({"error": "not_found", "reason": "missing"}),
        ),
        StoreError::Conflict => (
            StatusCode::CONFLICT,
            json!({"error": "conflict", "reason": "Document update conflict."}),
        ),
        StoreError::Invalid(reason) => (
            StatusCode::BAD_REQUEST,
            json!({"error": "bad_request", "reason": reason}),
        ),
        other => panic!("unexpected store error {other}"),
    }
}

proptest! {
    #![proptest_config(config(256))]

    /// Status-code invariant: an HTTP request answers exactly what the store
    /// answers for the same operation on an identical store: 200 when it
    /// succeeds, otherwise the status and body its error maps to.
    #[test]
    fn status_code_invariant(op in op()) {
        let http_app = seeded_app();
        let twin = seeded_app();
        let expected = direct(&twin.store, &op);
        block_on(async move {
            let (status, body) = over_http(&http_app.router, &http_app.store, &op).await;
            match expected {
                Ok(doc) => {
                    prop_assert_eq!(status, StatusCode::OK, "{:?}: {}", op, body);
                    match doc {
                        Some(doc) => prop_assert_eq!(body, doc),
                        None => prop_assert_eq!(&body["ok"], &json!(true)),
                    }
                }
                Err(err) => {
                    let (expected_status, mut expected_body) = expected_error(&err);
                    prop_assert_eq!(status, expected_status, "{:?}: {}", op, body);
                    let mut body = body;
                    if matches!(op, Op::Upsert { id: None, .. }) {
                        // Each store picks its own fresh ID and may quote it.
                        mask_quoted_ids(&mut body);
                        mask_quoted_ids(&mut expected_body);
                    }
                    prop_assert_eq!(body, expected_body);
                }
            }
            Ok(())
        })?;
    }
}

// ---------------------------------------------------------------------------
// Alias equivalence

proptest! {
    #![proptest_config(config(128))]

    /// Alias equivalence: `/<kind>/<id>` answers byte for byte what
    /// `/<kind>:<id>` answers, for existing and missing documents alike.
    #[test]
    fn alias_equivalence(
        kind in prop::sample::select(vec!["note", "book", "tag", "file", "widget"]),
        suffix in prop::sample::select(vec!["n0", "n1", "b0", "b1", "t0", "f0", "missing", "note:n0", "book:b0"]),
        rev in prop::sample::select(vec![None, Some("current"), Some("1-00000000000000000000000000000000")]),
        attachments in any::<bool>(),
    ) {
        let app = seeded_app();
        block_on(async move {
            let colon_id = if suffix.starts_with(&format!("{kind}:")) {
                suffix.to_owned()
            } else {
                format!("{kind}:{suffix}")
            };
            let rev = match rev {
                Some("current") => app.store.get(&colon_id).unwrap().map(|d| d.rev),
                other => other.map(str::to_owned),
            };
            let mut query = format!("?attachments={attachments}");
            if let Some(rev) = rev {
                query.push_str(&format!("&rev={rev}"));
            }
            let colon = send(&app.router, request(Method::GET, &format!("/{colon_id}{query}"), None)).await;
            let slash = send(&app.router, request(Method::GET, &format!("/{kind}/{suffix}{query}"), None)).await;
            prop_assert_eq!(colon.status, slash.status);
            prop_assert_eq!(colon.bytes, slash.bytes);
            Ok(())
        })?;
    }
}

// ---------------------------------------------------------------------------
// MCP vs HTTP oracle

const WORDS: [&str; 10] = [
    "apple",
    "Banana",
    "cherry",
    "go",
    "über",
    "😀",
    "inkdrop://note/n0",
    "a-b",
    "lorem-ipsum-dolor-sit-amet-consectetur-adipiscing",
    "\n",
];

#[derive(Clone, Debug)]
struct RandomNote {
    title: Vec<usize>,
    body: Vec<usize>,
    book: &'static str,
    tags: Vec<&'static str>,
    status: NoteStatus,
}

fn random_note() -> impl Strategy<Value = RandomNote> {
    (
        prop::collection::vec(0..WORDS.len(), 0..3),
        prop::collection::vec(0..WORDS.len(), 0..14),
        prop::sample::select(vec!["book:b0", "book:b1", "trash"]),
        prop::sample::subsequence(vec!["tag:t0", "tag:t1"], 0..=2),
        prop::sample::select(NoteStatus::ALL.to_vec()),
    )
        .prop_map(|(title, body, book, tags, status)| RandomNote {
            title,
            body,
            book,
            tags,
            status,
        })
}

fn keyword() -> impl Strategy<Value = String> {
    let clause = prop_oneof![
        prop::sample::select(WORDS[..8].to_vec()).prop_map(str::to_owned),
        prop::sample::select(vec!["app", "RRY", "ber", "ipsum dolor"])
            .prop_map(|t| format!("\"{t}\"")),
        prop::sample::select(vec![
            "book:Alpha",
            "book:\"beta Book\"",
            "bookId:b0",
            "bookId:trash",
            "tag:red",
            "tagId:tag:t1",
            "status:active",
            "title:an",
            "body:lorem",
        ])
        .prop_map(str::to_owned),
    ];
    prop::collection::vec((any::<bool>(), clause), 0..3).prop_map(|clauses| {
        clauses
            .into_iter()
            .map(|(negated, c)| if negated { format!("-{c}") } else { c })
            .collect::<Vec<_>>()
            .join(" ")
    })
}

#[derive(Clone, Debug)]
struct ListArgs {
    book_id: Option<&'static str>,
    tag_ids: Vec<&'static str>,
    keyword: Option<String>,
    sort: &'static str,
    descending: bool,
    limit: u32,
}

fn list_args() -> impl Strategy<Value = ListArgs> {
    (
        proptest::option::of(prop::sample::select(vec![
            "book:b0", "book:b1", "trash", "",
        ])),
        prop::sample::subsequence(vec!["tag:t0", "tag:t1"], 0..=2),
        proptest::option::of(keyword()),
        prop::sample::select(vec!["updatedAt", "createdAt", "title"]),
        any::<bool>(),
        0..30u32,
    )
        .prop_map(
            |(book_id, tag_ids, keyword, sort, descending, limit)| ListArgs {
                book_id,
                tag_ids,
                keyword,
                sort,
                descending,
                limit,
            },
        )
}

/// `list-notes`'s keyword, as @inkdropapp/mcp-server 1.5.0 builds it.
fn node_keyword(args: &ListArgs) -> String {
    let book_filter = match args.book_id {
        Some(book) if !book.is_empty() => {
            format!("bookId:{}", book.split(':').nth(1).unwrap_or("undefined"))
        }
        _ => String::new(),
    };
    let tag_filter: Vec<String> = args.tag_ids.iter().map(|t| format!("tagId:{t}")).collect();
    format!(
        "{book_filter} {} {}",
        tag_filter.join(" "),
        args.keyword.as_deref().unwrap_or("")
    )
    .trim()
    .to_owned()
}

/// JavaScript's `body.substring(0, 200)`, minus any half surrogate pair.
fn js_substring_200(body: &str) -> String {
    let units: Vec<u16> = body.encode_utf16().take(200).collect();
    char::decode_utf16(units).map_while(Result::ok).collect()
}

fn summaries_match(mcp: &Value, http: &Value) -> CaseResult {
    let (mcp, http) = (mcp.as_array().unwrap(), http.as_array().unwrap());
    prop_assert_eq!(mcp.len(), http.len());
    for (m, h) in mcp.iter().zip(http) {
        let mut expected = h.clone();
        expected["body"] = json!(js_substring_200(h["body"].as_str().unwrap()));
        prop_assert_eq!(m, &expected);
    }
    Ok(())
}

proptest! {
    #![proptest_config(config(64))]

    /// MCP vs HTTP oracle: `search-notes` and `list-notes` through `/mcp`
    /// return the notes `GET /notes` returns for the keyword the Node server
    /// would send, in the same order, with bodies cut to 200 characters.
    #[test]
    fn mcp_http_oracle(
        notes in prop::collection::vec(random_note(), 0..14),
        search in keyword(),
        list in list_args(),
    ) {
        let app = seeded_app();
        block_on(async move {
            for (i, note) in notes.iter().enumerate() {
                let words = |ix: &[usize]| ix.iter().map(|&w| WORDS[w]).collect::<Vec<_>>().join(" ");
                write(&app.router, "/notes", json!({
                    "title": words(&note.title), "body": words(&note.body), "bookId": note.book,
                    "tags": note.tags, "status": note.status.as_str(), "updatedAt": 1_000 + i % 5,
                    "createdAt": 2_000 - i % 3,
                })).await;
            }

            let mcp = tool_json(&tool(&app.router, "search-notes", json!({ "keyword": search })).await);
            let (status, http) = get(&app.router, &format!("/notes?keyword={}&limit=10", encode(&search))).await;
            prop_assert_eq!(status, StatusCode::OK);
            summaries_match(&mcp, &http)?;

            let mut args = json!({
                "tagIds": list.tag_ids, "sort": list.sort, "descending": list.descending, "limit": list.limit,
            });
            if let Some(book) = list.book_id {
                args["bookId"] = json!(book);
            }
            if let Some(keyword) = &list.keyword {
                args["keyword"] = json!(keyword);
            }
            let mcp = tool_json(&tool(&app.router, "list-notes", args).await);
            let uri = format!(
                "/notes?keyword={}&sort={}&descending={}&limit={}",
                encode(&node_keyword(&list)), list.sort, list.descending, list.limit
            );
            let (status, http) = get(&app.router, &uri).await;
            prop_assert_eq!(status, StatusCode::OK);
            summaries_match(&mcp, &http)?;
            Ok(())
        })?;
    }
}

// ---------------------------------------------------------------------------
// patch-note

/// Occurrences of `needle` in `haystack` counted in UTF-16 code units,
/// overlapping, as repeated JavaScript `indexOf` calls find them.
fn js_occurrences(haystack: &str, needle: &str) -> Vec<usize> {
    let h: Vec<u16> = haystack.encode_utf16().collect();
    let n: Vec<u16> = needle.encode_utf16().collect();
    (0..h.len().saturating_sub(n.len() - 1))
        .filter(|&i| h[i..].starts_with(&n))
        .collect()
}

proptest! {
    #![proptest_config(config(128))]

    /// patch-note refinement: the patch applies exactly when `old_string`
    /// occurs once (overlaps included) and then replaces that occurrence;
    /// otherwise the note is untouched and the Node server's message returned.
    #[test]
    fn patch_note_refinement(
        body in "[ab😀é]{0,16}",
        old in "[ab😀é]{1,3}",
        new in "[xy😀]{0,3}",
    ) {
        let app = app();
        block_on(async move {
            let (id, rev) = write(&app.router, "/notes", json!({ "bookId": "trash", "title": "", "body": body })).await;
            let result = tool(&app.router, "patch-note", json!({ "_id": id, "_rev": rev, "old_string": old, "new_string": new })).await;
            let (_, doc) = get(&app.router, &format!("/{id}")).await;
            let occurrences = js_occurrences(&body, &old);
            match occurrences.len() {
                1 => {
                    prop_assert_ne!(&result["isError"], &json!(true), "{}", result);
                    let units: Vec<u16> = body.encode_utf16().collect();
                    let at = occurrences[0];
                    let old_len = old.encode_utf16().count();
                    let mut expected: Vec<u16> = units[..at].to_vec();
                    expected.extend(new.encode_utf16());
                    expected.extend(&units[at + old_len..]);
                    prop_assert_eq!(&doc["body"], &json!(String::from_utf16(&expected).unwrap()));
                }
                n => {
                    prop_assert_eq!(&result["isError"], &json!(true));
                    let message = if n == 0 {
                        "Failed to patch: `old_string` was not found in the note body."
                    } else {
                        "Failed to patch: `old_string` matches multiple locations in the note body. Provide more surrounding context to make it unique."
                    };
                    prop_assert_eq!(text(&result, 0), message);
                    prop_assert_eq!(&doc["_rev"], &json!(rev));
                }
            }
            Ok(())
        })?;
    }
}
