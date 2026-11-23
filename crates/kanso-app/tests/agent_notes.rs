//! End to end: the fake ACP agent answers an Ask question by searching and
//! reading the notes through Kanso's read-only notes server, over HTTP and
//! through `kanso mcp` (this package's own binary) over stdio, and cites the
//! note it read. A follow-up continues the same session.

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use kanso_acp::fake::{self, Script};
use kanso_acp::{AcpError, AgentProcess, Limits};
use kanso_ai::agent::{
    self, AgentKind, AgentSettings, NotesAccess, NotesGrant, NotesLease, Starter,
};
use kanso_ai::{Event, Turn};
use kanso_core::DocType;
use kanso_server::{AgentScope, TokenId, TokenStore};
use kanso_store::{Origin, Put, Store};
use serde_json::json;

/// The agent host is one per app, so these tests take turns.
static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

struct Lease {
    tokens: TokenStore,
    id: TokenId,
}

impl NotesLease for Lease {
    fn is_live(&self) -> bool {
        self.tokens.is_live(self.id)
    }

    fn renew(&self) -> bool {
        self.tokens.renew(self.id)
    }

    fn begin_turn(&self) {
        self.tokens.begin_turn(self.id);
    }

    fn reads(&self) -> Vec<String> {
        self.tokens.reads(self.id)
    }
}

impl Drop for Lease {
    fn drop(&mut self) {
        self.tokens.revoke(self.id);
    }
}

/// How many tokens are live once the conversation's last turn thread has let
/// go of it, which it does just after its last event.
fn live_after_drop(tokens: &TokenStore) -> usize {
    for _ in 0..100 {
        if tokens.live_count() == 0 {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    tokens.live_count()
}

fn starter(script: Script) -> Starter {
    Arc::new(move |spec, limits| {
        let (from_agent, to_agent) =
            fake::in_process(script.clone()).map_err(|err| AcpError::Spawn(err.to_string()))?;
        AgentProcess::connect(
            &spec.label,
            from_agent,
            to_agent,
            limits,
            kanso_acp::reject_all(),
        )
    })
}

fn put(store: &Store, doc_type: DocType, fields: serde_json::Value) -> String {
    store
        .put(doc_type, fields, Put::Upsert, Origin::Sync)
        .unwrap()
        .id
}

/// A library with one note about zebras the agent may read, and two it may
/// not: a sealed one and one tagged `#ai-exclude`.
fn library() -> (tempfile::TempDir, Arc<Store>, String, Vec<String>) {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    let book = put(&store, DocType::Book, json!({ "name": "Animals" }));
    let tag = put(&store, DocType::Tag, json!({ "name": "ai-exclude" }));
    let sealed = put(
        &store,
        DocType::Note,
        json!({ "title": "Zebra secret", "body": "zebra hidden", "bookId": book }),
    );
    store
        .put(
            DocType::Note,
            json!({ "_id": sealed, "kansoLock": { "salt": "abc" } }),
            Put::Merge,
            Origin::Sync,
        )
        .unwrap();
    let excluded = put(
        &store,
        DocType::Note,
        json!({ "title": "Zebra private", "body": "zebra private", "bookId": book, "tags": [tag] }),
    );
    std::thread::sleep(Duration::from_millis(5));
    let facts = put(
        &store,
        DocType::Note,
        json!({ "title": "Zebra facts", "body": "Zebras have stripes.", "bookId": book }),
    );
    (dir, store, facts, vec![sealed, excluded])
}

fn ask(
    conversation: &Arc<agent::Conversation>,
    question: &str,
    history: &[Turn],
) -> (String, Vec<String>) {
    let read = Arc::new(Mutex::new(Vec::new()));
    let on_read = {
        let read = read.clone();
        Box::new(move |id: &str| read.lock().unwrap().push(id.to_owned()))
    };
    let prompt = kanso_ai::build_prompt(question, &[], kanso_ai::DEFAULT_BUDGET);
    let events: Vec<Event> = conversation
        .ask(&prompt, history, on_read, Arc::new(AtomicBool::new(false)))
        .unwrap()
        .collect();
    let text = events
        .iter()
        .filter_map(|event| match event {
            Event::Text(text) => Some(text.as_str()),
            Event::Error(error) => panic!("{error}"),
            _ => None,
        })
        .collect();
    let read = read.lock().unwrap().clone();
    (text, read)
}

fn answers_through(script: Script, via: &str) {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let (dir, store, facts, hidden) = library();
    let tokens = TokenStore::new();
    let listener = kanso_server::spawn_agents(store, tokens.clone());
    let url = format!("http://{}/mcp", listener.local_addr().unwrap());
    let notes: NotesAccess = {
        let tokens = tokens.clone();
        Arc::new(move || {
            let minted = tokens.mint(AgentScope::ask(None));
            Some(NotesGrant {
                suggest: false,
                url: url.clone(),
                token: minted.token,
                relay: Some(PathBuf::from(env!("CARGO_BIN_EXE_kanso"))),
                lease: Box::new(Lease {
                    tokens: tokens.clone(),
                    id: minted.id,
                }),
            })
        })
    };
    let settings = AgentSettings {
        kind: AgentKind::Custom,
        // Any existing file passes the custom command check; the starter
        // connects the fake instead of running it.
        command: Some(
            std::env::current_exe()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
        ),
        args: vec![via.to_owned()],
        model: None,
        env: Vec::new(),
        workspace: dir.path().join("agents").join("workspace"),
    };
    let conversation =
        agent::conversation_with(&settings, Limits::default(), starter(script), Some(notes));

    let (said, read) = ask(&conversation, "What do I know about zebra?", &[]);
    assert!(
        said.contains("Zebras have stripes. [[Zebra facts]]"),
        "{said}"
    );
    assert!(said.contains(&format!("(via {via}, turn 1)")), "{said}");
    assert_eq!(read, std::slice::from_ref(&facts));
    for id in &hidden {
        assert!(!said.contains(id.trim_start_matches("note:")));
    }
    assert!(!said.contains("hidden") && !said.contains("private"));
    assert_eq!(tokens.live_count(), 1);

    // A follow-up continues the same session, on the same token.
    let history = [Turn {
        question: "What do I know about zebra?".into(),
        answer: said,
    }];
    let (said, read) = ask(&conversation, "Tell me more about zebra?", &history);
    assert!(said.contains(&format!("(via {via}, turn 2)")), "{said}");
    assert_eq!(read, [facts]);
    assert_eq!(tokens.live_count(), 1);

    // Ending the conversation revokes its token.
    assert!(conversation.has_session());
    drop(conversation);
    assert_eq!(live_after_drop(&tokens), 0);
    drop(listener);
}

#[test]
fn an_agent_answers_from_the_notes_over_http() {
    answers_through(Script::Notes, "http");
}

#[test]
fn an_agent_answers_from_the_notes_over_stdio() {
    answers_through(Script::NotesStdio, "stdio");
}

#[test]
fn an_expired_token_starts_a_new_session_rather_than_widening_the_old() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let (dir, store, facts, _) = library();
    let now = Arc::new(Mutex::new(std::time::Instant::now()));
    let clock = now.clone();
    let tokens = TokenStore::with_clock(Arc::new(move || *clock.lock().unwrap()));
    let listener = kanso_server::spawn_agents(store, tokens.clone());
    let url = format!("http://{}/mcp", listener.local_addr().unwrap());
    let minted_count = Arc::new(Mutex::new(0));
    let notes: NotesAccess = {
        let tokens = tokens.clone();
        let minted_count = minted_count.clone();
        Arc::new(move || {
            *minted_count.lock().unwrap() += 1;
            let minted = tokens.mint(AgentScope::ask(None));
            Some(NotesGrant {
                suggest: false,
                url: url.clone(),
                token: minted.token,
                relay: None,
                lease: Box::new(Lease {
                    tokens: tokens.clone(),
                    id: minted.id,
                }),
            })
        })
    };
    let settings = AgentSettings {
        kind: AgentKind::Custom,
        command: Some(
            std::env::current_exe()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
        ),
        args: vec!["expiry".to_owned()],
        env: Vec::new(),
        workspace: dir.path().join("agents").join("workspace"),
        model: None,
    };
    let conversation = agent::conversation_with(
        &settings,
        Limits::default(),
        starter(Script::Notes),
        Some(notes),
    );
    let (said, _) = ask(&conversation, "What about zebra?", &[]);
    assert!(said.contains("turn 1"), "{said}");
    // Idle past the token's limit: the next question opens a new session
    // with a new token, and the old token stays dead.
    *now.lock().unwrap() += Duration::from_secs(6 * 60);
    let history = [Turn {
        question: "What about zebra?".into(),
        answer: said,
    }];
    let (said, read) = ask(&conversation, "More on zebra?", &history);
    assert!(
        said.contains("[[Zebra facts]]") && said.contains("turn 1"),
        "{said}"
    );
    assert_eq!(read, [facts]);
    assert_eq!(*minted_count.lock().unwrap(), 2);
    assert_eq!(tokens.live_count(), 1);
    drop(conversation);
    assert_eq!(live_after_drop(&tokens), 0);
}
