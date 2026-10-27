//! Kanso agent tokens on `/mcp`: what they open, what they may read, and that
//! they write nothing. A note an agent may not see (locked, in a locked
//! notebook, trashed, deleted, tagged `#ai-exclude`, or outside its
//! notebooks) never shows in any answer, not even its title.

mod common;

use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use common::*;
use kanso_core::DocType;
use kanso_server::{AgentScope, READ_TOOLS, TokenStore, agents_router, router_with_tokens};
use kanso_store::{Origin, Put, Store};
use proptest::prelude::*;
use proptest::test_runner::{Config, FileFailurePersistence};
use serde_json::{Value, json};

struct Agents {
    _dir: tempfile::TempDir,
    store: Arc<Store>,
    api: Router,
    private: Router,
    tokens: TokenStore,
}

fn agents_with(tokens: TokenStore) -> Agents {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    let api = router_with_tokens(store.clone(), &config(), tokens.clone());
    let private = agents_router(store.clone(), tokens.clone());
    Agents {
        _dir: dir,
        store,
        api,
        private,
        tokens,
    }
}

fn agents() -> Agents {
    agents_with(TokenStore::new())
}

fn bearer_request(uri: &str, token: &str, body: &Value) -> Request<Body> {
    Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

/// A JSON-RPC request with an agent's token; the status and the answer.
async fn agent_rpc(
    router: &Router,
    token: &str,
    method: &str,
    params: Value,
) -> (StatusCode, Value) {
    let message = json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params });
    let response = send(router, bearer_request("/mcp", token, &message)).await;
    (response.status, response.json())
}

async fn agent_tool(router: &Router, token: &str, name: &str, arguments: Value) -> Value {
    let (status, answer) = agent_rpc(
        router,
        token,
        "tools/call",
        json!({ "name": name, "arguments": arguments }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{answer}");
    answer["result"].clone()
}

fn text_of(result: &Value) -> String {
    result["content"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|block| block["text"].as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

fn book(store: &Store, name: &str, parent: Option<&str>, locked: bool) -> String {
    let mut fields = json!({ "name": name });
    if let Some(parent) = parent {
        fields["parentBookId"] = json!(parent);
    }
    if locked {
        fields["kansoLock"] = json!({ "salt": "abc" });
    }
    store
        .put(DocType::Book, fields, Put::Upsert, Origin::Ui)
        .unwrap()
        .id
}

fn note(store: &Store, title: &str, body: &str, book: &str, tags: Vec<String>) -> String {
    store
        .put(
            DocType::Note,
            json!({ "title": title, "body": body, "bookId": book, "tags": tags }),
            Put::Upsert,
            // As another device would: the text of a note in a locked
            // notebook isn't sealed yet.
            Origin::Sync,
        )
        .unwrap()
        .id
}

fn seal(store: &Store, id: &str) {
    store
        .put(
            DocType::Note,
            json!({ "_id": id, "kansoLock": { "salt": "abc" } }),
            Put::Merge,
            Origin::Sync,
        )
        .unwrap();
}

#[tokio::test]
async fn an_ask_token_lists_two_tools_and_opens_mcp_alone() {
    let agents = agents();
    let minted = agents.tokens.mint(AgentScope::ask(None));
    for router in [&agents.api, &agents.private] {
        let (status, answer) = agent_rpc(router, &minted.token, "tools/list", json!({})).await;
        assert_eq!(status, StatusCode::OK);
        let names: Vec<&str> = answer["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|tool| tool["name"].as_str().unwrap())
            .collect();
        assert_eq!(names, ["read-note", "search-notes"]);
        // A tool it isn't given is refused, as read-only.
        let result = agent_tool(router, &minted.token, "list-notebooks", json!({})).await;
        assert_eq!(result["isError"], true);
        assert!(text_of(&result).contains("read-only"));
    }
    // The token opens no REST route, on either server.
    for router in [&agents.api, &agents.private] {
        for uri in ["/notes", "/books", "/", "/_changes"] {
            let response = send(
                router,
                Request::builder()
                    .uri(uri)
                    .header(header::AUTHORIZATION, format!("Bearer {}", minted.token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await;
            assert_eq!(response.status, StatusCode::UNAUTHORIZED, "{uri}");
        }
    }
    // The private listener takes no Basic sign-in at all.
    let (status, _) = call(&agents.private, Method::GET, "/notes", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = post(
        &agents.private,
        "/mcp",
        json!({ "jsonrpc": "2.0", "id": 1, "method": "ping" }),
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn expired_and_revoked_tokens_get_401() {
    let now = Arc::new(Mutex::new(Instant::now()));
    let clock = now.clone();
    let agents = agents_with(TokenStore::with_clock(Arc::new(move || {
        *clock.lock().unwrap()
    })));
    let ping = json!({});
    let idle = agents.tokens.mint(AgentScope::ask(None));
    let revoked = agents.tokens.mint(AgentScope::ask(None));
    for minted in [&idle, &revoked] {
        let (status, _) = agent_rpc(&agents.private, &minted.token, "ping", ping.clone()).await;
        assert_eq!(status, StatusCode::OK);
    }
    agents.tokens.revoke(revoked.id);
    let (status, _) = agent_rpc(&agents.private, &revoked.token, "ping", ping.clone()).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    *now.lock().unwrap() += Duration::from_secs(5 * 60);
    for router in [&agents.api, &agents.private] {
        let (status, _) = agent_rpc(router, &idle.token, "ping", ping.clone()).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }
    let fresh = agents.tokens.mint(AgentScope::ask(None));
    *now.lock().unwrap() += Duration::from_secs(60);
    let (status, _) = agent_rpc(&agents.api, &fresh.token, "ping", ping.clone()).await;
    assert_eq!(status, StatusCode::OK);
    agents.tokens.revoke_all();
    let (status, _) = agent_rpc(&agents.api, &fresh.token, "ping", ping).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn an_agent_reads_what_its_scope_holds_and_nothing_else() {
    let agents = agents();
    let store = &agents.store;
    let work = book(store, "Work", None, false);
    let clients = book(store, "Clients", Some(&work), false);
    let home = book(store, "Home", None, false);
    let vault = book(store, "Vault", Some(&work), true);
    let tag = store
        .put(
            DocType::Tag,
            json!({ "name": "#AI-Exclude" }),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    let pricing = note(store, "Pricing", "zebra yearly plan", &clients, vec![]);
    let groceries = note(store, "Groceries", "zebra milk", &home, vec![]);
    let hidden = note(store, "Hidden plan", "zebra secret", &work, vec![tag]);
    let in_vault = note(store, "Vault plan", "zebra vault", &vault, vec![]);
    let sealed = note(store, "Sealed plan", "zebra sealed", &work, vec![]);
    seal(store, &sealed);
    let trashed = note(store, "Trashed plan", "zebra trashed", "trash", vec![]);
    let scope = AgentScope::with_tools(Some(HashSet::from([work.clone()])), &READ_TOOLS);
    let minted = agents.tokens.mint(scope);
    let token = &minted.token;

    let found = text_of(
        &agent_tool(
            &agents.api,
            token,
            "search-notes",
            json!({ "keyword": "zebra" }),
        )
        .await,
    );
    assert!(found.contains(&pricing), "{found}");
    for id in [&groceries, &hidden, &in_vault, &sealed, &trashed] {
        assert!(!found.contains(id.as_str()), "{id} in {found}");
    }
    let read = agent_tool(
        &agents.api,
        token,
        "read-note",
        json!({ "noteId": pricing }),
    )
    .await;
    assert!(text_of(&read).contains("zebra yearly plan"));
    // Denied reads exactly like missing.
    let missing = agent_tool(
        &agents.api,
        token,
        "read-note",
        json!({ "noteId": "note:nope" }),
    )
    .await;
    for id in [&groceries, &hidden, &in_vault, &sealed, &trashed] {
        let denied = agent_tool(&agents.api, token, "read-note", json!({ "noteId": id })).await;
        assert_eq!(denied, missing, "{id}");
    }
    let books = text_of(&agent_tool(&agents.api, token, "list-notebooks", json!({})).await);
    assert!(books.contains("Work") && books.contains("Clients"));
    assert!(
        !books.contains("Vault") && !books.contains("Home"),
        "{books}"
    );
    let link = text_of(
        &agent_tool(
            &agents.api,
            token,
            "kanso-resolve-wikilink",
            json!({ "link": "[[Vault plan]]" }),
        )
        .await,
    );
    assert!(!link.contains(&in_vault));
    let (_, resource) = agent_rpc(
        &agents.api,
        token,
        "resources/read",
        json!({ "uri": format!("inkdrop://note/{}", sealed.trim_start_matches("note:")) }),
    )
    .await;
    assert!(resource.get("error").is_some(), "{resource}");
    assert!(!resource.to_string().contains("Sealed plan"));
    // What it saw is what Used notes lists.
    assert_eq!(
        agents.tokens.reads(minted.id),
        std::slice::from_ref(&pricing)
    );

    // The local API's own sign-in still sees everything.
    let full = tool(&agents.api, "search-notes", json!({ "keyword": "zebra" })).await;
    assert!(text_of(&full).contains(&groceries));
}

/// Every tool a full sign-in has, with arguments it would accept.
fn write_calls(ids: &[String], books: &[String]) -> Vec<(&'static str, Value)> {
    let note = ids.first().cloned().unwrap_or_default();
    let book = books.first().cloned().unwrap_or_default();
    vec![
        (
            "create-note",
            json!({ "title": "x", "body": "y", "bookId": book, "status": "none" }),
        ),
        (
            "update-note",
            json!({ "_id": note, "_rev": "1-x", "title": "x", "body": "y", "bookId": book, "status": "none" }),
        ),
        (
            "patch-note",
            json!({ "_id": note, "_rev": "1-x", "old_string": "a", "new_string": "b" }),
        ),
        ("create-tag", json!({ "name": "x" })),
        (
            "update-tag",
            json!({ "_id": "tag:x", "_rev": "1-x", "name": "y" }),
        ),
        ("create-book", json!({ "name": "x" })),
        (
            "update-book",
            json!({ "_id": book, "_rev": "1-x", "name": "y" }),
        ),
        (
            "create-file",
            json!({ "name": "a.png", "contentType": "image/png", "data": "aGVsbG8=" }),
        ),
        ("read-file", json!({ "fileId": "file:x" })),
        (
            "kanso-attach-file",
            json!({ "noteId": note, "name": "a.png", "contentType": "image/png", "data": "aGVsbG8=" }),
        ),
        ("kanso-list-attachments", json!({ "noteId": note })),
        (
            "kanso-read-attachment",
            json!({ "noteId": note, "fileId": "file:x" }),
        ),
        ("kanso-move-note", json!({ "noteId": note, "bookId": book })),
        (
            "kanso-set-task-done",
            json!({ "noteId": note, "line": 0, "text": "task", "done": true }),
        ),
        ("kanso-create-daily-note", json!({})),
        (
            "kanso-create-from-template",
            json!({ "templateId": note, "title": "x" }),
        ),
        ("kanso-export-html", json!({ "noteId": note })),
        ("read-backlinks", json!({ "noteId": note })),
        ("read-tag", json!({ "tagId": "tag:x" })),
        ("read-book", json!({ "bookId": book })),
        ("list-tags", json!({})),
        ("no-such-tool", json!({})),
    ]
}

fn config_for(cases: u32) -> Config {
    Config {
        cases,
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
        ..Config::default()
    }
}

proptest! {
    #![proptest_config(config_for(48))]

    /// Every tool that isn't a read is refused for an agent, whichever read
    /// tools its scope names and even if its scope names write tools too,
    /// and nothing in the store changes.
    #[test]
    fn prop_read_only_token_denies_every_write_tool(
        notes in 0usize..4,
        named in prop::collection::vec(prop::sample::select(vec![
            "create-note", "update-note", "patch-note", "create-book", "read-file",
            "search-notes", "read-note", "list-notebooks", "kanso-list-tasks",
        ]), 0..6),
    ) {
        let agents = agents();
        let store = &agents.store;
        let shelf = book(store, "Shelf", None, false);
        let ids: Vec<String> = (0..notes)
            .map(|ix| note(store, &format!("Note {ix}"), "a body", &shelf, vec![]))
            .collect();
        let mut scope = AgentScope::with_tools(None, &named);
        // A scope built by hand with write tools in it still writes nothing.
        scope.tools.extend(named.iter().map(|tool| (*tool).to_owned()));
        let minted = agents.tokens.mint(scope);
        let before = store.last_seq().unwrap();
        let full = runtime().block_on(rpc(&agents.api, "tools/list", json!({})));
        let every: Vec<String> = full["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|tool| tool["name"].as_str().unwrap().to_owned())
            .collect();
        let calls = write_calls(&ids, &[shelf]);
        // Every tool that isn't a read is in the calls above.
        for name in &every {
            prop_assert!(
                READ_TOOLS.contains(&name.as_str()) || calls.iter().any(|(tool, _)| tool == name),
                "{name} isn't tried"
            );
        }
        for router in [&agents.api, &agents.private] {
            for (name, arguments) in &calls {
                let result = runtime().block_on(agent_tool(router, &minted.token, name, arguments.clone()));
                prop_assert_eq!(&result["isError"], &json!(true), "{} ran: {}", name, result);
                prop_assert!(text_of(&result).contains("read-only"), "{}", result);
            }
        }
        prop_assert_eq!(store.last_seq().unwrap(), before);
        prop_assert!(agents.tokens.reads(minted.id).is_empty());
    }
}

/// What a note in the generated store is.
#[derive(Clone, Copy, Debug)]
enum Kind {
    Plain,
    Sealed,
    Trashed,
    Deleted,
    Excluded,
}

fn gen_kind() -> impl Strategy<Value = Kind> {
    prop_oneof![
        3 => Just(Kind::Plain),
        1 => Just(Kind::Sealed),
        1 => Just(Kind::Trashed),
        1 => Just(Kind::Deleted),
        1 => Just(Kind::Excluded),
    ]
}

proptest! {
    #![proptest_config(config_for(48))]

    /// Random notebooks (nested, some locked), random notes in them (sealed,
    /// trashed, deleted, `#ai-exclude`d or plain), a random notebook scope,
    /// and random calls of every read tool and `resources/read`: no answer
    /// names a note outside the scope or any excluded note, by id, title or
    /// text, and Used notes lists only notes in scope.
    #[test]
    fn prop_scoped_mcp_never_returns_a_note_outside_scope(
        parents in prop::collection::vec((any::<prop::sample::Index>(), any::<bool>(), 0u8..4), 1..6),
        kinds in prop::collection::vec((gen_kind(), any::<prop::sample::Index>()), 1..12),
        scoped in prop::option::of(prop::collection::vec(any::<prop::sample::Index>(), 0..3)),
        calls in prop::collection::vec((prop::sample::select(READ_TOOLS.to_vec()), any::<prop::sample::Index>()), 1..14),
        resources in prop::collection::vec(any::<prop::sample::Index>(), 0..4),
    ) {
        let agents = agents();
        let store = &agents.store;
        let tag = store
            .put(DocType::Tag, json!({ "name": "ai-exclude" }), Put::Upsert, Origin::Ui)
            .unwrap()
            .id;
        // Notebooks, each maybe inside an earlier one, some locked.
        let mut books: Vec<(String, Option<usize>, bool)> = Vec::new();
        for (ix, (parent, locked, nest)) in parents.iter().enumerate() {
            let parent = (ix > 0 && *nest > 0).then(|| parent.index(ix));
            let locked = *locked && *nest == 3;
            let id = book(
                store,
                &format!("shelf~{ix}~"),
                parent.map(|at| books[at].0.as_str()),
                locked,
            );
            books.push((id, parent, locked));
        }
        let locked_at = |mut at: usize| loop {
            if books[at].2 {
                break true;
            }
            match books[at].1 {
                Some(parent) => at = parent,
                None => break false,
            }
        };
        let roots: Option<HashSet<String>> = scoped.as_ref().map(|picks| {
            picks.iter().map(|pick| books[pick.index(books.len())].0.clone()).collect()
        });
        let in_scope = |at: usize| {
            let Some(roots) = &roots else { return true };
            let mut at = Some(at);
            while let Some(here) = at {
                if roots.contains(&books[here].0) {
                    return true;
                }
                at = books[here].1;
            }
            false
        };
        // Notes, each with a word of its own in its title and body.
        let mut visible = Vec::new();
        let mut hidden: Vec<(String, String)> = Vec::new();
        let mut every = Vec::new();
        for (ix, (kind, place)) in kinds.iter().enumerate() {
            let at = place.index(books.len());
            let word = format!("~w{ix}~");
            let title = format!("zebra {word} title");
            let body = format!("zebra {word} body\n# Heading {word}\n- [ ] task {word}");
            let (book_id, tags) = match kind {
                Kind::Trashed => ("trash".to_owned(), vec![]),
                Kind::Excluded => (books[at].0.clone(), vec![tag.clone()]),
                _ => (books[at].0.clone(), vec![]),
            };
            let id = note(store, &title, &body, &book_id, tags);
            match kind {
                Kind::Sealed => seal(store, &id),
                Kind::Deleted => {
                    store.delete(&id, Origin::Ui).unwrap();
                }
                _ => {}
            }
            every.push(id.clone());
            let shows = matches!(kind, Kind::Plain) && !locked_at(at) && in_scope(at);
            if shows {
                visible.push(id);
            } else {
                hidden.push((id, word));
            }
        }
        let hidden_books: Vec<String> = books
            .iter()
            .enumerate()
            .filter(|(at, _)| locked_at(*at) || !in_scope(*at))
            .map(|(at, _)| format!("shelf~{at}~"))
            .collect();
        let minted = agents.tokens.mint(AgentScope::with_tools(roots.clone(), &READ_TOOLS));
        let mut answers = Vec::new();
        for (name, pick) in &calls {
            let id = every[pick.index(every.len())].clone();
            let book_id = books[pick.index(books.len())].0.clone();
            // What the agent itself sent comes back in an error; that is not
            // a note shown.
            let mut echoed = None;
            let arguments = match *name {
                "search-notes" => json!({ "keyword": "zebra" }),
                "read-note" | "kanso-get-outline" | "kanso-related-notes"
                | "kanso-get-properties" | "kanso-list-note-links" | "kanso-list-backlinks"
                | "kanso-run-query-view" | "kanso-list-versions" => json!({ "noteId": id }),
                "kanso-read-version" => json!({ "noteId": id, "rev": "missing" }),
                "list-notes" => json!({ "bookId": book_id }),
                "kanso-list-tags" => json!({}),
                "kanso-list-tasks" => json!({}),
                "kanso-search-by-meaning" => json!({ "query": "zebra" }),
                "kanso-resolve-wikilink" => {
                    let title = store.note(&id).ok().flatten().map(|n| n.title).unwrap_or_default();
                    echoed = Some(title.clone());
                    json!({ "link": format!("[[{title}]]") })
                }
                _ => json!({}),
            };
            let result = runtime().block_on(agent_tool(&agents.private, &minted.token, name, arguments));
            let mut answer = result.to_string();
            if let Some(echoed) = echoed.filter(|_| result["isError"] == json!(true)) {
                answer = answer.replace(&echoed, "");
            }
            answers.push(answer);
        }
        for pick in &resources {
            let id = &every[pick.index(every.len())];
            let uri = format!("inkdrop://note/{}", id.trim_start_matches("note:"));
            let (_, answer) = runtime().block_on(agent_rpc(
                &agents.private,
                &minted.token,
                "resources/read",
                json!({ "uri": uri }),
            ));
            answers.push(answer.to_string());
        }
        for answer in &answers {
            for (id, word) in &hidden {
                let bare = id.trim_start_matches("note:");
                prop_assert!(!answer.contains(bare), "{} shown: {}", id, answer);
                prop_assert!(!answer.contains(word.as_str()), "{} shown: {}", word, answer);
            }
            if calls.iter().any(|(name, _)| *name == "list-notebooks") {
                for name in &hidden_books {
                    prop_assert!(!answer.contains(name.as_str()), "{} shown: {}", name, answer);
                }
            }
            prop_assert!(!answer.contains("kansoLock"));
        }
        for read in agents.tokens.reads(minted.id) {
            prop_assert!(visible.contains(&read), "{} listed as used", read);
        }
        // Not vacuous: a search that may see a note finds it.
        let searched = runtime().block_on(agent_tool(
            &agents.private,
            &minted.token,
            "search-notes",
            json!({ "keyword": "zebra" }),
        ));
        for id in visible.iter().filter(|_| visible.len() <= 10) {
            prop_assert!(searched.to_string().contains(id.as_str()), "{} not found", id);
        }
    }
}
