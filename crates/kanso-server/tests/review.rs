//! The review queue: an agent the person let suggest edits stages changes
//! and writes nothing. Only Apply writes, once, to a note that is still as
//! the agent read it and still one it may see.

mod common;

use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use common::*;
use kanso_core::DocType;
use kanso_server::{
    AgentScope, ApplyError, ChangeId, REVIEW_EXPIRY, ReviewQueue, TokenStore, agents_router,
};
use kanso_store::{Origin, Put, Store};
use proptest::prelude::*;
use proptest::test_runner::{Config, FileFailurePersistence};
use serde_json::{Value, json};

const AGENT: &str = "Claude Agent";
const QUEUED: &str = "Queued for review. The person will apply or discard it in Kanso.";

struct Setup {
    _dir: tempfile::TempDir,
    store: Arc<Store>,
    private: Router,
    tokens: TokenStore,
}

fn setup_with(tokens: TokenStore) -> Setup {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    let private = agents_router(store.clone(), tokens.clone());
    Setup {
        _dir: dir,
        store,
        private,
        tokens,
    }
}

fn setup() -> Setup {
    setup_with(TokenStore::new())
}

fn config_for(cases: u32) -> Config {
    Config {
        cases,
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
        ..Config::default()
    }
}

async fn agent_tool(router: &Router, token: &str, name: &str, arguments: Value) -> Value {
    let message = json!({
        "jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": { "name": name, "arguments": arguments },
    });
    let request = Request::builder()
        .method(Method::POST)
        .uri("/mcp")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(message.to_string()))
        .unwrap();
    let response = send(router, request).await;
    assert_eq!(response.status, StatusCode::OK);
    response.json()["result"].clone()
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

fn queued(result: &Value) -> bool {
    result["isError"] != json!(true) && text_of(result) == QUEUED
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
            Origin::Sync,
        )
        .unwrap()
        .id
}

fn rev_of(store: &Store, id: &str) -> String {
    store.note(id).unwrap().unwrap().rev.unwrap()
}

/// Everything in the store: each document with its revision and fields, and
/// the sequence of the last change.
fn snapshot(store: &Store) -> (i64, Vec<(String, String, String)>) {
    let docs = store
        .changes_since(0, None, false)
        .unwrap()
        .into_iter()
        .map(|doc| (doc.id, doc.rev, serde_json::to_string(&doc.fields).unwrap()))
        .collect();
    (store.last_seq().unwrap(), docs)
}

fn suggester(setup: &Setup, notebooks: Option<HashSet<String>>) -> kanso_server::Minted {
    setup
        .tokens
        .mint(AgentScope::ask_and_suggest(notebooks, AGENT))
}

fn update_args(store: &Store, id: &str, body: &str) -> Value {
    json!({ "_id": id, "_rev": rev_of(store, id), "body": body })
}

#[tokio::test]
async fn a_suggestion_is_queued_and_the_store_is_untouched() {
    let setup = setup();
    let shelf = book(&setup.store, "Shelf", None, false);
    let id = note(&setup.store, "Plan", "one two three", &shelf, vec![]);
    let minted = suggester(&setup, None);
    let before = snapshot(&setup.store);
    let result = agent_tool(
        &setup.private,
        &minted.token,
        "patch-note",
        json!({ "_id": id, "_rev": rev_of(&setup.store, &id), "old_string": "two", "new_string": "2" }),
    )
    .await;
    assert!(queued(&result), "{result}");
    assert_eq!(snapshot(&setup.store), before);
    let pending = setup.tokens.reviews().list();
    assert_eq!(pending.len(), 1);
    let change = &pending[0];
    assert_eq!(change.note_id.as_deref(), Some(id.as_str()));
    assert_eq!(change.old_body, "one two three");
    assert_eq!(change.new_body, "one 2 three");
    assert_eq!(change.new_title, "Plan");
    assert_eq!(change.agent, AGENT);
    assert_eq!(change.token, minted.id);

    let applied = setup
        .tokens
        .reviews()
        .apply(&setup.store, change.id)
        .unwrap();
    assert_eq!(applied.note_id, id);
    assert_eq!(setup.store.note(&id).unwrap().unwrap().body, "one 2 three");
    assert!(setup.tokens.reviews().is_empty());
}

#[tokio::test]
async fn history_of_an_applied_change_names_the_agent() {
    let setup = setup();
    let shelf = book(&setup.store, "Shelf", None, false);
    let id = note(&setup.store, "Plan", "before", &shelf, vec![]);
    let minted = suggester(&setup, None);
    let result = agent_tool(
        &setup.private,
        &minted.token,
        "update-note",
        json!({ "_id": id, "_rev": rev_of(&setup.store, &id), "title": "Plan B", "body": "after" }),
    )
    .await;
    assert!(queued(&result), "{result}");
    let change = setup.tokens.reviews().list().remove(0);
    setup
        .tokens
        .reviews()
        .apply(&setup.store, change.id)
        .unwrap();
    let now = setup.store.note(&id).unwrap().unwrap();
    assert_eq!((now.title.as_str(), now.body.as_str()), ("Plan B", "after"));
    let history = setup.store.revisions(&id).unwrap();
    let newest = &history[0];
    // The version kept is the one the agent's change replaced; its origin
    // says it was an agent's edit that the person applied.
    assert_eq!(newest.origin, "agent");
    assert_eq!(newest.agent.as_deref(), Some(AGENT));
    assert_eq!(newest.fields["body"], "before");
    assert_eq!(history.iter().filter(|r| r.origin == "agent").count(), 1);
}

#[tokio::test]
async fn a_new_note_waits_and_is_created_on_apply() {
    let setup = setup();
    let shelf = book(&setup.store, "Shelf", None, false);
    let minted = suggester(&setup, None);
    let before = snapshot(&setup.store);
    let result = agent_tool(
        &setup.private,
        &minted.token,
        "create-note",
        json!({ "bookId": shelf, "title": "Ideas", "body": "- one" }),
    )
    .await;
    assert!(queued(&result), "{result}");
    assert_eq!(snapshot(&setup.store), before);
    let change = setup.tokens.reviews().list().remove(0);
    assert!(change.is_new_note() && change.base_revision.is_none());
    let applied = setup
        .tokens
        .reviews()
        .apply(&setup.store, change.id)
        .unwrap();
    let made = setup.store.note(&applied.note_id).unwrap().unwrap();
    assert_eq!(
        (made.title.as_str(), made.body.as_str()),
        ("Ideas", "- one")
    );
    assert_eq!(made.book_id, shelf);
}

#[tokio::test]
async fn only_text_and_title_can_be_suggested() {
    let setup = setup();
    let shelf = book(&setup.store, "Shelf", None, false);
    let id = note(&setup.store, "Plan", "body", &shelf, vec![]);
    let minted = suggester(&setup, None);
    let rev = rev_of(&setup.store, &id);
    let calls = [
        (
            "update-note",
            json!({ "_id": id, "_rev": rev, "body": "x", "pinned": true }),
        ),
        (
            "update-note",
            json!({ "_id": id, "_rev": rev, "body": "x", "bookId": "trash" }),
        ),
        ("update-note", json!({ "_id": id, "_rev": rev })),
        (
            "create-note",
            json!({ "bookId": shelf, "title": "t", "body": "b", "status": "active" }),
        ),
        (
            "update-tag",
            json!({ "_id": "tag:x", "_rev": "1-x", "name": "y" }),
        ),
        (
            "update-book",
            json!({ "_id": shelf, "_rev": "1-x", "name": "y" }),
        ),
        ("create-tag", json!({ "name": "x" })),
        ("create-book", json!({ "name": "x" })),
        (
            "create-file",
            json!({ "name": "a.png", "contentType": "image/png", "data": "aGVsbG8=" }),
        ),
        ("read-file", json!({ "fileId": "file:x" })),
        ("kanso-create-daily-note", json!({})),
    ];
    let before = snapshot(&setup.store);
    for (name, arguments) in calls {
        let result = agent_tool(&setup.private, &minted.token, name, arguments).await;
        assert_eq!(result["isError"], true, "{name}: {result}");
    }
    assert!(setup.tokens.reviews().is_empty());
    assert_eq!(snapshot(&setup.store), before);
}

#[tokio::test]
async fn with_the_setting_off_the_write_tools_are_read_only() {
    let setup = setup();
    let shelf = book(&setup.store, "Shelf", None, false);
    let id = note(&setup.store, "Plan", "body", &shelf, vec![]);
    let minted = setup.tokens.mint(AgentScope::ask(None));
    let before = snapshot(&setup.store);
    for (name, arguments) in [
        (
            "patch-note",
            json!({ "_id": id, "_rev": rev_of(&setup.store, &id), "old_string": "body", "new_string": "x" }),
        ),
        ("update-note", update_args(&setup.store, &id, "x")),
        (
            "create-note",
            json!({ "bookId": shelf, "title": "t", "body": "b" }),
        ),
    ] {
        let result = agent_tool(&setup.private, &minted.token, name, arguments).await;
        assert_eq!(result["isError"], true, "{name}");
        assert!(text_of(&result).contains("read-only"), "{result}");
    }
    assert!(setup.tokens.reviews().is_empty());
    assert_eq!(snapshot(&setup.store), before);
    // And they aren't listed either.
    let message = json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/list" });
    let request = Request::builder()
        .method(Method::POST)
        .uri("/mcp")
        .header(header::AUTHORIZATION, format!("Bearer {}", minted.token))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(message.to_string()))
        .unwrap();
    let listed = send(&setup.private, request).await.json();
    let names: Vec<&str> = listed["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tool| tool["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        names,
        [
            "read-note",
            "search-notes",
            "kanso-ask-read-lines",
            "kanso-ask-search",
            "kanso-ask-when"
        ]
    );
}

#[tokio::test]
async fn a_second_edit_to_the_same_note_replaces_the_first() {
    let setup = setup();
    let shelf = book(&setup.store, "Shelf", None, false);
    let id = note(&setup.store, "Plan", "a b c", &shelf, vec![]);
    let minted = suggester(&setup, None);
    let rev = rev_of(&setup.store, &id);
    for (old, new) in [("a", "A"), ("c", "C")] {
        let result = agent_tool(
            &setup.private,
            &minted.token,
            "patch-note",
            json!({ "_id": id, "_rev": rev, "old_string": old, "new_string": new }),
        )
        .await;
        assert!(queued(&result), "{result}");
    }
    let pending = setup.tokens.reviews().list();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].old_body, "a b c");
    assert_eq!(pending[0].new_body, "A b C");
}

#[tokio::test]
async fn a_changed_note_refuses_apply_and_keeps_the_suggestion() {
    let setup = setup();
    let shelf = book(&setup.store, "Shelf", None, false);
    let id = note(&setup.store, "Plan", "agent read this", &shelf, vec![]);
    let minted = suggester(&setup, None);
    let result = agent_tool(
        &setup.private,
        &minted.token,
        "update-note",
        update_args(&setup.store, &id, "agent's version"),
    )
    .await;
    assert!(queued(&result));
    // The person edits the note meanwhile.
    setup
        .store
        .put(
            DocType::Note,
            json!({ "_id": id, "body": "person's version" }),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    let before = snapshot(&setup.store);
    let change = setup.tokens.reviews().list().remove(0);
    let refused = setup
        .tokens
        .reviews()
        .apply(&setup.store, change.id)
        .unwrap_err();
    assert_eq!(
        refused,
        ApplyError::Conflict {
            current_title: "Plan".into(),
            current_body: "person's version".into(),
        }
    );
    assert_eq!(
        refused.message(),
        "The note changed since the agent read it."
    );
    assert_eq!(snapshot(&setup.store), before);
    // It stays, with both versions to compare, until it is discarded.
    let kept = setup.tokens.reviews().get(change.id).unwrap();
    assert_eq!(kept.new_body, "agent's version");
    assert!(setup.tokens.reviews().discard(change.id));
    assert!(!setup.tokens.reviews().discard(change.id));
}

/// How a note can stop being one an agent may change.
#[derive(Clone, Copy, Debug)]
enum Hide {
    Sealed,
    Trashed,
    Excluded,
    BookLocked,
    Deleted,
}

fn hide(store: &Store, id: &str, book: &str, how: Hide) {
    match how {
        Hide::Sealed => {
            store
                .put(
                    DocType::Note,
                    json!({ "_id": id, "kansoLock": { "salt": "abc" } }),
                    Put::Merge,
                    Origin::Sync,
                )
                .unwrap();
        }
        Hide::Trashed => {
            store
                .put(
                    DocType::Note,
                    json!({ "_id": id, "bookId": "trash" }),
                    Put::Merge,
                    Origin::Ui,
                )
                .unwrap();
        }
        Hide::Excluded => {
            let tag = store
                .put(
                    DocType::Tag,
                    json!({ "name": "ai-exclude" }),
                    Put::Upsert,
                    Origin::Ui,
                )
                .unwrap()
                .id;
            store
                .put(
                    DocType::Note,
                    json!({ "_id": id, "tags": [tag] }),
                    Put::Merge,
                    Origin::Ui,
                )
                .unwrap();
        }
        Hide::BookLocked => {
            // Kanso seals the notebook's notes before it locks the notebook.
            store
                .put(
                    DocType::Note,
                    json!({ "_id": id, "kansoLock": { "salt": "abc" } }),
                    Put::Merge,
                    Origin::Sync,
                )
                .unwrap();
            store
                .put(
                    DocType::Book,
                    json!({ "_id": book, "kansoLock": { "salt": "abc" } }),
                    Put::Merge,
                    Origin::Ui,
                )
                .unwrap();
        }
        Hide::Deleted => {
            store.delete(id, Origin::Ui).unwrap();
        }
    }
}

#[tokio::test]
async fn locking_trashing_or_excluding_drops_the_pending_change() {
    for how in [
        Hide::Sealed,
        Hide::Trashed,
        Hide::Excluded,
        Hide::BookLocked,
        Hide::Deleted,
    ] {
        let setup = setup();
        let shelf = book(&setup.store, "Shelf", None, false);
        let id = note(&setup.store, "Plan", "original", &shelf, vec![]);
        let minted = suggester(&setup, None);
        let result = agent_tool(
            &setup.private,
            &minted.token,
            "update-note",
            update_args(&setup.store, &id, "changed"),
        )
        .await;
        assert!(queued(&result), "{how:?}");
        hide(&setup.store, &id, &shelf, how);
        let before = snapshot(&setup.store);
        let change = setup.tokens.reviews().list().remove(0);
        let refused = setup.tokens.reviews().apply(&setup.store, change.id);
        assert_eq!(refused, Err(ApplyError::NotAllowed), "{how:?}");
        assert_eq!(snapshot(&setup.store), before, "{how:?}");
        assert!(setup.tokens.reviews().is_empty(), "{how:?}: dropped");
    }
}

#[tokio::test]
async fn unappliable_changes_are_dropped_when_asked() {
    let setup = setup();
    let shelf = book(&setup.store, "Shelf", None, false);
    let kept = note(&setup.store, "Kept", "x", &shelf, vec![]);
    let hidden = note(&setup.store, "Hidden", "y", &shelf, vec![]);
    let minted = suggester(&setup, None);
    for id in [&kept, &hidden] {
        let result = agent_tool(
            &setup.private,
            &minted.token,
            "update-note",
            update_args(&setup.store, id, "z"),
        )
        .await;
        assert!(queued(&result));
    }
    assert_eq!(setup.tokens.reviews().drop_unappliable(&setup.store), 0);
    hide(&setup.store, &hidden, &shelf, Hide::Excluded);
    assert_eq!(setup.tokens.reviews().drop_unappliable(&setup.store), 1);
    let left = setup.tokens.reviews().list();
    assert_eq!(left.len(), 1);
    assert_eq!(left[0].note_id.as_deref(), Some(kept.as_str()));
}

#[tokio::test]
async fn a_pending_change_outlives_its_token() {
    let setup = setup();
    let shelf = book(&setup.store, "Shelf", None, false);
    let id = note(&setup.store, "Plan", "x", &shelf, vec![]);
    let minted = suggester(&setup, None);
    let result = agent_tool(
        &setup.private,
        &minted.token,
        "update-note",
        update_args(&setup.store, &id, "y"),
    )
    .await;
    assert!(queued(&result));
    // Clearing the Ask conversation revokes the session's token.
    setup.tokens.revoke(minted.id);
    setup.tokens.revoke_all();
    assert_eq!(setup.tokens.reviews().len(), 1);
}

#[tokio::test]
async fn suggestions_expire_after_an_hour() {
    let now = Arc::new(Mutex::new(Instant::now()));
    let clock = now.clone();
    let setup = setup_with(TokenStore::with_clock(Arc::new(move || {
        *clock.lock().unwrap()
    })));
    let shelf = book(&setup.store, "Shelf", None, false);
    let id = note(&setup.store, "Plan", "x", &shelf, vec![]);
    let minted = suggester(&setup, None);
    let result = agent_tool(
        &setup.private,
        &minted.token,
        "update-note",
        update_args(&setup.store, &id, "y"),
    )
    .await;
    assert!(queued(&result));
    let waiting = setup.tokens.reviews().list().remove(0);
    let events = setup.tokens.reviews().subscribe();
    assert_eq!(REVIEW_EXPIRY, Duration::from_secs(3600));
    *now.lock().unwrap() += REVIEW_EXPIRY - Duration::from_secs(1);
    assert_eq!(setup.tokens.reviews().len(), 1);
    *now.lock().unwrap() += Duration::from_secs(1);
    let before = snapshot(&setup.store);
    assert_eq!(setup.tokens.reviews().len(), 0);
    assert_eq!(
        setup.tokens.reviews().apply(&setup.store, waiting.id),
        Err(ApplyError::Gone)
    );
    assert_eq!(snapshot(&setup.store), before);
    // Subscribers were told it went.
    assert!(events.try_recv().is_ok());
}

#[tokio::test]
async fn subscribers_hear_of_arrivals_and_departures() {
    let setup = setup();
    let events = setup.tokens.reviews().subscribe();
    let shelf = book(&setup.store, "Shelf", None, false);
    let id = note(&setup.store, "Plan", "x", &shelf, vec![]);
    let minted = suggester(&setup, None);
    agent_tool(
        &setup.private,
        &minted.token,
        "update-note",
        update_args(&setup.store, &id, "y"),
    )
    .await;
    let change = setup.tokens.reviews().list().remove(0);
    assert_eq!(
        events.try_recv().unwrap(),
        kanso_server::ReviewEvent::Arrived(change.id)
    );
    setup.tokens.reviews().discard(change.id);
    assert_eq!(
        events.try_recv().unwrap(),
        kanso_server::ReviewEvent::Changed
    );
}

#[test]
fn two_applies_at_once_write_once() {
    let setup = setup();
    let shelf = book(&setup.store, "Shelf", None, false);
    let id = note(&setup.store, "Plan", "x", &shelf, vec![]);
    let minted = suggester(&setup, None);
    let result = runtime().block_on(agent_tool(
        &setup.private,
        &minted.token,
        "update-note",
        update_args(&setup.store, &id, "y"),
    ));
    assert!(queued(&result));
    let change = setup.tokens.reviews().list().remove(0).id;
    let outcomes: Vec<bool> = std::thread::scope(|scope| {
        let tries: Vec<_> = (0..4)
            .map(|_| scope.spawn(|| setup.tokens.reviews().apply(&setup.store, change).is_ok()))
            .collect();
        tries.into_iter().map(|t| t.join().unwrap()).collect()
    });
    assert_eq!(outcomes.iter().filter(|ok| **ok).count(), 1);
    let agent_versions = setup
        .store
        .revisions(&id)
        .unwrap()
        .iter()
        .filter(|r| r.origin == "agent")
        .count();
    assert_eq!(agent_versions, 1);
}

/// What a generated agent call is.
#[derive(Clone, Copy, Debug)]
enum Call {
    Patch,
    Update,
    Create,
    /// Stale revision.
    Stale,
    /// A note that isn't there.
    Missing,
}

fn gen_call() -> impl Strategy<Value = Call> {
    prop_oneof![
        Just(Call::Patch),
        Just(Call::Update),
        Just(Call::Create),
        Just(Call::Stale),
        Just(Call::Missing),
    ]
}

proptest! {
    #![proptest_config(config_for(32))]

    /// Random stores and random agent write calls, good and bad: the store,
    /// every note, notebook and tag with its revision, is exactly as it was
    /// until a change is applied, and then only that note differs.
    #[test]
    fn prop_pending_write_changes_nothing_until_apply(
        bodies in prop::collection::vec("[a-z ]{1,20}", 1..5),
        calls in prop::collection::vec((gen_call(), any::<prop::sample::Index>(), "[a-z]{0,10}"), 1..10),
        pick in any::<prop::sample::Index>(),
    ) {
        let setup = setup();
        let shelf = book(&setup.store, "Shelf", None, false);
        let ids: Vec<String> = bodies
            .iter()
            .enumerate()
            .map(|(ix, body)| note(&setup.store, &format!("Note {ix}"), body, &shelf, vec![]))
            .collect();
        let minted = suggester(&setup, None);
        let before = snapshot(&setup.store);
        for (call, at, text) in &calls {
            let id = &ids[at.index(ids.len())];
            let current = setup.store.note(id).unwrap().unwrap();
            let rev = current.rev.clone().unwrap();
            let (name, arguments) = match call {
                Call::Patch => ("patch-note", json!({ "_id": id, "_rev": rev, "old_string": current.body, "new_string": text })),
                Call::Update => ("update-note", json!({ "_id": id, "_rev": rev, "title": format!("T {text}"), "body": text })),
                Call::Create => ("create-note", json!({ "bookId": shelf, "title": "N", "body": text })),
                Call::Stale => ("update-note", json!({ "_id": id, "_rev": "9-stale", "body": text })),
                Call::Missing => ("update-note", json!({ "_id": "note:nosuchnote", "_rev": rev, "body": text })),
            };
            let result = runtime().block_on(agent_tool(&setup.private, &minted.token, name, arguments));
            if matches!(call, Call::Patch) {
                // A patch builds on the agent's own waiting change to the
                // note, so its text may no longer be there to find.
                prop_assert!(queued(&result) || result["isError"] == json!(true), "{}", result);
            } else {
                let should_queue = matches!(call, Call::Update | Call::Create);
                prop_assert_eq!(queued(&result), should_queue, "{:?}: {}", call, result);
            }
            prop_assert_eq!(snapshot(&setup.store), before.clone());
        }
        let pending = setup.tokens.reviews().list();
        if let Some(change) = pending.get(pick.index(pending.len().max(1))) {
            let applied = setup.tokens.reviews().apply(&setup.store, change.id).unwrap();
            let after = snapshot(&setup.store);
            // Only the applied note differs (a created note is new).
            let changed: Vec<&String> = after.1.iter()
                .filter(|doc| !before.1.contains(doc))
                .map(|doc| &doc.0)
                .collect();
            prop_assert_eq!(changed, vec![&applied.note_id]);
        }
    }
}

proptest! {
    #![proptest_config(config_for(32))]

    /// Random sequences of apply, discard and apply-again over changes to
    /// different notes: each change is written at most once, only the first
    /// action on it counts, and a note nothing applied is as it was.
    #[test]
    fn prop_apply_writes_exactly_once(
        count in 1usize..5,
        ops in prop::collection::vec((any::<prop::sample::Index>(), any::<bool>()), 1..14),
    ) {
        let setup = setup();
        let shelf = book(&setup.store, "Shelf", None, false);
        let ids: Vec<String> = (0..count)
            .map(|ix| note(&setup.store, &format!("Note {ix}"), "original", &shelf, vec![]))
            .collect();
        let minted = suggester(&setup, None);
        let mut changes: Vec<ChangeId> = Vec::new();
        for id in &ids {
            let result = runtime().block_on(agent_tool(
                &setup.private,
                &minted.token,
                "update-note",
                update_args(&setup.store, id, "agent text"),
            ));
            prop_assert!(queued(&result));
        }
        for id in &ids {
            let change = setup.tokens.reviews().list().into_iter()
                .find(|c| c.note_id.as_deref() == Some(id.as_str()))
                .unwrap();
            changes.push(change.id);
        }
        // `Some(true)` applied, `Some(false)` discarded, `None` untouched.
        let mut outcome: Vec<Option<bool>> = vec![None; count];
        for (index, apply) in &ops {
            let at = index.index(count);
            let first = outcome[at].is_none();
            if *apply {
                let result = setup.tokens.reviews().apply(&setup.store, changes[at]);
                prop_assert_eq!(result.is_ok(), first);
                if !first {
                    prop_assert_eq!(result, Err(ApplyError::Gone));
                }
            } else {
                prop_assert_eq!(setup.tokens.reviews().discard(changes[at]), first);
            }
            if first {
                outcome[at] = Some(*apply);
            }
        }
        for (ix, id) in ids.iter().enumerate() {
            let stored = setup.store.note(id).unwrap().unwrap();
            let versions = setup.store.revisions(id).unwrap().iter()
                .filter(|r| r.origin == "agent")
                .count();
            if outcome[ix] == Some(true) {
                prop_assert_eq!(stored.body, "agent text");
                prop_assert_eq!(versions, 1);
            } else {
                prop_assert_eq!(stored.body, "original");
                prop_assert_eq!(versions, 0);
            }
        }
        let waiting = outcome.iter().filter(|o| o.is_none()).count();
        prop_assert_eq!(setup.tokens.reviews().len(), waiting);
    }
}

/// How a generated note sits.
#[derive(Clone, Copy, Debug)]
enum Place {
    Plain,
    Sealed,
    Trashed,
    Excluded,
    InLockedBook,
    OutOfScope,
}

fn gen_place() -> impl Strategy<Value = Place> {
    prop_oneof![
        3 => Just(Place::Plain),
        1 => Just(Place::Sealed),
        1 => Just(Place::Trashed),
        1 => Just(Place::Excluded),
        1 => Just(Place::InLockedBook),
        1 => Just(Place::OutOfScope),
    ]
}

#[test]
fn a_locked_trashed_excluded_or_out_of_scope_note_cannot_be_targeted() {
    // An example of each, tried through every suggesting tool.
    let setup = setup();
    let inside = book(&setup.store, "Inside", None, false);
    let outside = book(&setup.store, "Outside", None, false);
    let vault = book(&setup.store, "Vault", None, true);
    let tag = setup
        .store
        .put(
            DocType::Tag,
            json!({ "name": "ai-exclude" }),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    let plain = note(&setup.store, "Plain", "x", &inside, vec![]);
    let sealed = note(&setup.store, "Sealed", "x", &inside, vec![]);
    setup
        .store
        .put(
            DocType::Note,
            json!({ "_id": sealed, "kansoLock": { "salt": "abc" } }),
            Put::Merge,
            Origin::Sync,
        )
        .unwrap();
    let trashed = note(&setup.store, "Trashed", "x", "trash", vec![]);
    let excluded = note(&setup.store, "Excluded", "x", &inside, vec![tag]);
    let in_vault = note(&setup.store, "In vault", "x", &vault, vec![]);
    let elsewhere = note(&setup.store, "Elsewhere", "x", &outside, vec![]);
    let minted = suggester(&setup, Some([inside.clone()].into()));
    for id in [&sealed, &trashed, &excluded, &in_vault, &elsewhere] {
        let rev = rev_of(&setup.store, id);
        for (name, arguments) in [
            (
                "update-note",
                json!({ "_id": id, "_rev": rev, "body": "y" }),
            ),
            (
                "patch-note",
                json!({ "_id": id, "_rev": rev, "old_string": "x", "new_string": "y" }),
            ),
        ] {
            let result =
                runtime().block_on(agent_tool(&setup.private, &minted.token, name, arguments));
            assert_eq!(result["isError"], true, "{id} {name}: {result}");
            assert!(text_of(&result).contains("404"), "{result}");
        }
    }
    // Nor can a new note go into a notebook it can't see.
    for target in [
        &vault,
        &outside,
        &"trash".to_owned(),
        &"book:nosuchbook".to_owned(),
    ] {
        let result = runtime().block_on(agent_tool(
            &setup.private,
            &minted.token,
            "create-note",
            json!({ "bookId": target, "title": "t", "body": "b" }),
        ));
        assert_eq!(result["isError"], true, "{target}: {result}");
    }
    assert!(setup.tokens.reviews().is_empty());
    // The one it may see queues.
    let result = runtime().block_on(agent_tool(
        &setup.private,
        &minted.token,
        "update-note",
        update_args(&setup.store, &plain, "y"),
    ));
    assert!(queued(&result), "{result}");
}

proptest! {
    #![proptest_config(config_for(32))]

    /// Random notes, each plain, sealed, trashed, `#ai-exclude`d, in a locked
    /// notebook or outside the agent's notebooks, and every suggesting tool
    /// tried on each: only the plain ones in scope are queued, and every
    /// change waiting targets a note the agent may see.
    #[test]
    fn prop_a_pending_change_never_targets_a_hidden_note(
        places in prop::collection::vec(gen_place(), 1..10),
        tool in prop::sample::select(vec!["update-note", "patch-note", "create-note"]),
    ) {
        let setup = setup();
        let inside = book(&setup.store, "Inside", None, false);
        let outside = book(&setup.store, "Outside", None, false);
        let vault = book(&setup.store, "Vault", None, true);
        let tag = setup.store
            .put(DocType::Tag, json!({ "name": "ai-exclude" }), Put::Upsert, Origin::Ui)
            .unwrap()
            .id;
        let mut expected = 0;
        let mut ids = Vec::new();
        for (ix, place) in places.iter().enumerate() {
            let title = format!("Note {ix}");
            let id = match place {
                Place::Plain => { expected += 1; note(&setup.store, &title, "x", &inside, vec![]) }
                Place::Sealed => {
                    let id = note(&setup.store, &title, "x", &inside, vec![]);
                    setup.store.put(DocType::Note, json!({ "_id": id, "kansoLock": { "salt": "abc" } }), Put::Merge, Origin::Sync).unwrap();
                    id
                }
                Place::Trashed => note(&setup.store, &title, "x", "trash", vec![]),
                Place::Excluded => note(&setup.store, &title, "x", &inside, vec![tag.clone()]),
                Place::InLockedBook => note(&setup.store, &title, "x", &vault, vec![]),
                Place::OutOfScope => note(&setup.store, &title, "x", &outside, vec![]),
            };
            ids.push(id);
        }
        let minted = suggester(&setup, Some([inside.clone()].into()));
        let before = snapshot(&setup.store);
        let notebooks = [inside.clone(), outside.clone(), vault.clone(), "trash".to_owned()];
        if tool == "create-note" {
            for target in &notebooks {
                let result = runtime().block_on(agent_tool(
                    &setup.private, &minted.token, tool,
                    json!({ "bookId": target, "title": "t", "body": "b" }),
                ));
                prop_assert_eq!(queued(&result), target == &inside, "{}: {}", target, result);
            }
            prop_assert_eq!(setup.tokens.reviews().len(), 1);
        } else {
            for id in &ids {
                let rev = rev_of(&setup.store, id);
                let arguments = if tool == "update-note" {
                    json!({ "_id": id, "_rev": rev, "body": "y" })
                } else {
                    json!({ "_id": id, "_rev": rev, "old_string": "x", "new_string": "y" })
                };
                runtime().block_on(agent_tool(&setup.private, &minted.token, tool, arguments));
            }
            prop_assert_eq!(setup.tokens.reviews().len(), expected);
            for change in setup.tokens.reviews().list() {
                let id = change.note_id.unwrap();
                let at = ids.iter().position(|seen| seen == &id).unwrap();
                prop_assert!(matches!(places[at], Place::Plain));
            }
        }
        prop_assert_eq!(snapshot(&setup.store), before);
    }
}

#[test]
fn the_queue_is_full_after_fifty() {
    let setup = setup();
    let shelf = book(&setup.store, "Shelf", None, false);
    let mut minted = suggester(&setup, None);
    for ix in 0..60 {
        // A token allows only so many calls per question.
        if ix % 20 == 0 {
            minted = suggester(&setup, None);
        }
        let result = runtime().block_on(agent_tool(
            &setup.private,
            &minted.token,
            "create-note",
            json!({ "bookId": shelf, "title": format!("N{ix}"), "body": "b" }),
        ));
        assert_eq!(queued(&result), ix < 50, "{ix}: {result}");
    }
    assert_eq!(setup.tokens.reviews().len(), 50);
}

#[test]
fn a_default_queue_is_empty_and_clones_share() {
    let queue = ReviewQueue::new();
    assert!(queue.is_empty());
    let other = queue.clone();
    assert_eq!(other.len(), 0);
}
