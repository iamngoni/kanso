//! Ask through an agent: the prompt it is sent, the notes it is and is not
//! sent, and the receipts its answer is checked against. The agent is the
//! fake ACP agent, in process.

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use kanso_ai::{Event, Prompt};
use kanso_core::DocType;
use kanso_engine::ask::{AskRequest, Shown, finalize, prepare, stream_events};
use kanso_store::{Origin, Put, Store};
use serde_json::json;

fn open() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    (dir, store)
}

fn note(store: &Store, title: &str, body: &str, tags: Vec<String>) -> String {
    let book = store
        .put(
            DocType::Book,
            json!({"name": "Notes"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    store
        .put(
            DocType::Note,
            json!({"title": title, "body": body, "bookId": book, "tags": tags}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id
}

fn today() -> kanso_core::dates::Date {
    kanso_core::dates::Date::new(2026, 10, 20).unwrap()
}

/// An agent on this computer through the fake ACP agent, in process.
fn fake_agent(script: kanso_acp::fake::Script, name: &str) -> kanso_ai::SharedProvider {
    let settings = kanso_ai::agent::AgentSettings {
        kind: kanso_ai::AgentKind::Custom,
        command: Some(
            std::env::current_exe()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
        ),
        args: Vec::new(),
        env: Vec::new(),
        workspace: std::env::temp_dir().join(format!("kanso-ask-agent-{name}")),
    };
    kanso_ai::agent::agent_client_with(
        &settings,
        kanso_acp::Limits::default(),
        Arc::new(move |spec, limits| {
            let (from_agent, to_agent) = kanso_acp::fake::in_process(script.clone())
                .map_err(|err| kanso_acp::AcpError::Spawn(err.to_string()))?;
            kanso_acp::AgentProcess::connect(
                &spec.label,
                from_agent,
                to_agent,
                limits,
                kanso_acp::reject_all(),
            )
        }),
    )
}

/// Asks `provider` the way the panel does, returning the answer's text.
fn ask_through(provider: &kanso_ai::SharedProvider, prompt: &Prompt) -> String {
    let provider = provider.lock().unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    let events = provider.ask(
        prompt,
        &[],
        kanso_ai::ToolRegistry::new(),
        Box::new(|| {}),
        cancel.clone(),
        &kanso_ai::Budget::default(),
    );
    let mut text = String::new();
    let mut done = false;
    stream_events(events, &cancel, |event| match event {
        Event::Text(piece) => text.push_str(&piece),
        Event::Done => done = true,
        _ => {}
    });
    assert!(done, "the agent's answer ended");
    text
}

// no-prop-test: drives the fake agent end to end; the checking has its own properties.
#[test]
fn an_agent_answer_is_checked_against_the_numbered_excerpts() {
    let (_dir, store) = open();
    let id = note(
        &store,
        "Pricing",
        "# Pricing\nWe decided the yearly plan costs 40 dollars.",
        vec![],
    );
    let prepared = prepare(
        &store,
        &AskRequest::new("What did we decide about pricing?", today()),
    )
    .unwrap();
    let provider = fake_agent(
        kanso_acp::fake::Script::Answer(
            "The yearly plan costs 40 dollars [1:L2]. It also floats [1:L99].".into(),
        ),
        "receipts",
    );
    let answer = ask_through(&provider, &prepared.numbered.prompt);
    let done = finalize(&answer, &prepared.shown, true, &prepared.retrieval);
    assert_eq!(done.receipts.len(), 1, "{:?}", done.markdown);
    assert_eq!(done.receipts[0].note_id, id);
    assert_eq!((done.receipts[0].start, done.receipts[0].end), (1, 1));
    assert_eq!(done.unverified, 1, "the invented line is flagged");
    // The shown record is what the checker used: the second marker named a
    // line that was never sent.
    assert!(!Shown::new().sent().iter().any(|s| s.id == id));
}

// no-prop-test: drives the fake agent end to end; eligibility has its own properties.
#[test]
fn an_agent_is_sent_only_eligible_notes() {
    let (_dir, store) = open();
    note(
        &store,
        "Pricing",
        "We decided the yearly plan costs 40 dollars.",
        vec![],
    );
    let locked = note(
        &store,
        "Locked pricing",
        "The locked pricing is 99.",
        vec![],
    );
    store
        .put(
            DocType::Note,
            json!({"_id": locked, "kansoLock": {"salt": "abc"}}),
            Put::Merge,
            Origin::Sync,
        )
        .unwrap();
    let tag = store
        .put(
            DocType::Tag,
            json!({"name": "AI-Exclude"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    note(
        &store,
        "Excluded pricing",
        "The excluded pricing is 77.",
        vec![tag],
    );
    let trashed = note(
        &store,
        "Trashed pricing",
        "The trashed pricing is 55.",
        vec![],
    );
    store.delete(&trashed, Origin::Ui).unwrap();
    let prepared = prepare(&store, &AskRequest::new("What is the pricing?", today())).unwrap();
    // The fake agent says back exactly what it was sent.
    let provider = fake_agent(kanso_acp::fake::Script::Echo, "eligible");
    let sent = ask_through(&provider, &prepared.numbered.prompt);
    assert!(sent.contains("yearly plan"), "{sent}");
    assert!(sent.starts_with(kanso_ai::agent::AGENT_SYSTEM));
    for hidden in [
        "99",
        "77",
        "55",
        "Locked pricing",
        "Excluded pricing",
        "Trashed pricing",
    ] {
        assert!(!sent.contains(hidden), "{hidden} reached the agent");
    }
    assert!(sent.trim_end().ends_with("Question: What is the pricing?"));
}
