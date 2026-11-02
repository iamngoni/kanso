//! The task runner end to end, without a window: an ACP agent (the fake)
//! that really calls the notes server, and scripted API-key models.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use kanso_acp::fake::{self, Script};
use kanso_acp::{AcpError, AgentProcess, Limits};
use kanso_ai::agent::{AgentKind, AgentSettings, Starter};
use kanso_ai::{AiError, ChatProvider, Event, Prompt, SharedProvider, ToolCall, ToolRegistry};
use kanso_core::DocType;
use kanso_engine::agent_task::{ReadScope, TaskSpec};
use kanso_server::{Selection, TokenStore};
use kanso_store::{Origin, Put, Store};
use serde_json::{Value, json};

use kanso_engine::agent_task::model::{EndedBy, Phase};
use kanso_engine::agent_task::runner::{ApiDriver, Tasks};

/// The agent host is one per app, so tests that start it take turns.
static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

struct Library {
    dir: tempfile::TempDir,
    store: Arc<Store>,
    trips: String,
    notes: Vec<(String, String)>,
}

fn library() -> Library {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    let trips = store
        .put(
            DocType::Book,
            json!({"name": "Trips"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    let notes = ["Lisbon flights", "Lisbon hotel", "Lisbon food"]
        .iter()
        .map(|title| {
            let w = store
                .put(
                    DocType::Note,
                    json!({"bookId": trips, "title": title, "body": format!("{title} lisbon notes")}),
                    Put::Upsert,
                    Origin::Ui,
                )
                .unwrap();
            (w.id, w.rev)
        })
        .collect();
    Library {
        dir,
        store,
        trips,
        notes,
    }
}

fn spec(lib: &Library, goal: &str) -> TaskSpec {
    TaskSpec::new(goal, ReadScope::Notebook(lib.trips.clone()), &lib.trips)
}

fn wait_until(what: &str, mut done: impl FnMut() -> bool) {
    let start = Instant::now();
    while !done() {
        assert!(
            start.elapsed() < Duration::from_secs(20),
            "timed out waiting for {what}"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn row(tasks: &Tasks, id: kanso_server::TaskId) -> kanso_engine::agent_task::model::TaskRow {
    tasks.row(id).unwrap()
}

// ---------------------------------------------------------- an ACP agent

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

#[test]
fn an_acp_agent_proposes_a_merge_through_the_real_notes_server() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let lib = library();
    let tokens = TokenStore::new();
    let listener = kanso_server::spawn_agents(lib.store.clone(), tokens.clone());
    let url = format!("http://{}/mcp", listener.local_addr().unwrap());
    let settings = AgentSettings {
        kind: AgentKind::Custom,
        command: Some(
            std::env::current_exe()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
        ),
        args: vec!["task-e2e".into()],
        env: Vec::new(),
        workspace: lib.dir.path().join("agents").join("workspace"),
    };
    let tasks = Tasks::default();
    let goal = "Merge the notes about lisbon";
    let id = tasks
        .start(
            &lib.store,
            &tokens,
            &spec(&lib, goal),
            "Claude Agent",
            |open| {
                let access = crate::agent_task_panel::task_access(
                    open,
                    url.clone(),
                    tokens.clone(),
                    None::<PathBuf>,
                );
                Ok(Arc::new(crate::agent_task_acp::AcpDriver(
                    kanso_ai::agent::task_conversation_with(
                        &settings,
                        Limits::default(),
                        starter(Script::Task),
                        access,
                        Duration::from_secs(60),
                    ),
                )))
            },
        )
        .unwrap();
    wait_until("the task to end", || !row(&tasks, id).is_active());
    let finished = row(&tasks, id);
    assert_eq!(finished.phase, Phase::Ready, "{finished:?}");
    assert_eq!(finished.counts.merges, 1);
    assert_eq!(
        finished.notes_read, 3,
        "the fake searched and found all three"
    );
    // Nothing was written, and the token ended with the task.
    let set = tasks.changes(id).unwrap();
    assert!(set.done);
    assert_eq!(set.summary, "Merged them.");
    assert_eq!(tokens.live_count(), 0, "the token is revoked at the end");
    for (note, _) in &lib.notes {
        assert_eq!(lib.store.note(note).unwrap().unwrap().book_id, lib.trips);
    }
    // Apply the merge, then undo it.
    let applied = tasks.apply(id, &Selection::all(1), 100).unwrap();
    assert_eq!(applied.created.len(), 1);
    assert_eq!(
        lib.store.note(&lib.notes[0].0).unwrap().unwrap().book_id,
        "trash"
    );
    assert!(tasks.can_undo(id));
    tasks.undo(id, 200).unwrap().unwrap();
    assert_eq!(
        lib.store.note(&lib.notes[0].0).unwrap().unwrap().book_id,
        lib.trips
    );
    assert_eq!(row(&tasks, id).phase, Phase::Undone);
    drop(listener);
}

// ------------------------------------------------- scripted API-key models

/// A model that calls its scripted tools, then waits to be stopped (or
/// ends right away).
struct Scripted {
    runs: Mutex<Vec<Run>>,
}

struct Run {
    calls: Vec<(String, Value)>,
    /// What the model says at the end of the run.
    says: String,
    /// Keep the run open until cancelled.
    hang: bool,
}

impl Scripted {
    fn provider(runs: Vec<Run>) -> SharedProvider {
        let mut runs = runs;
        runs.reverse();
        Arc::new(Mutex::new(Box::new(Scripted {
            runs: Mutex::new(runs),
        })))
    }
}

impl ChatProvider for Scripted {
    fn stream(&self, _prompt: &Prompt) -> Result<Box<dyn Iterator<Item = Event> + Send>, AiError> {
        Ok(Box::new(vec![Event::Done].into_iter()))
    }

    fn run_task(
        &self,
        _prompt: &Prompt,
        tools: ToolRegistry<()>,
        _max: usize,
        mut on_tool_completed: Box<dyn FnMut() + Send>,
        cancel: Arc<AtomicBool>,
    ) -> Result<Box<dyn Iterator<Item = Event> + Send>, AiError> {
        let run = self.runs.lock().unwrap().pop().expect("a scripted run");
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        for (i, (name, args)) in run.calls.iter().enumerate() {
            let call = ToolCall {
                id: format!("c{i}"),
                name: name.clone(),
                arguments: args.clone(),
            };
            runtime
                .block_on(tools.execute((), &call))
                .unwrap()
                .expect("the tool exists");
            on_tool_completed();
        }
        Ok(Box::new(Hang {
            says: Some(run.says),
            hang: run.hang,
            cancel,
        }))
    }
}

struct Hang {
    says: Option<String>,
    hang: bool,
    cancel: Arc<AtomicBool>,
}

impl Iterator for Hang {
    type Item = Event;

    fn next(&mut self) -> Option<Event> {
        if let Some(text) = self.says.take()
            && !text.is_empty()
        {
            return Some(Event::Text(text));
        }
        while self.hang && !self.cancel.load(Ordering::Relaxed) {
            std::thread::sleep(Duration::from_millis(10));
        }
        self.hang = false;
        None
    }
}

fn move_call(lib: &Library) -> (String, Value) {
    let (id, rev) = &lib.notes[0];
    (
        "update_note".into(),
        json!({"_id": id, "_rev": rev, "body": "new text"}),
    )
}

fn start_api(
    lib: &Library,
    tasks: &Tasks,
    tokens: &TokenStore,
    spec: &TaskSpec,
    runs: Vec<Run>,
) -> kanso_server::TaskId {
    let provider = Scripted::provider(runs);
    tasks
        .start(&lib.store, tokens, spec, "Anthropic", move |_| {
            Ok(Arc::new(ApiDriver(provider)))
        })
        .unwrap()
}

#[test]
fn stop_keeps_the_change_set() {
    let lib = library();
    let tokens = TokenStore::new();
    let tasks = Tasks::default();
    let id = start_api(
        &lib,
        &tasks,
        &tokens,
        &spec(&lib, "Tidy"),
        vec![Run {
            calls: vec![move_call(&lib)],
            says: "Working".into(),
            hang: true,
        }],
    );
    wait_until("a proposal", || row(&tasks, id).counts.edits == 1);
    assert!(row(&tasks, id).is_active());
    tasks.stop(id);
    wait_until("the stop", || !row(&tasks, id).is_active());
    let stopped = row(&tasks, id);
    assert_eq!(stopped.phase, Phase::Ready);
    assert_eq!(stopped.ended_by, Some(EndedBy::Stopped));
    assert_eq!(tasks.changes(id).unwrap().ops.len(), 1);
    assert!(!tokens.is_live(stopped_token(&tasks, id)));
    // Review it later: the edit applies.
    tasks.apply(id, &Selection::all(1), 5).unwrap();
    assert_eq!(
        lib.store.note(&lib.notes[0].0).unwrap().unwrap().body,
        "new text"
    );
}

fn stopped_token(tasks: &Tasks, id: kanso_server::TaskId) -> kanso_server::TokenId {
    tasks.token(id)
}

#[test]
fn a_task_past_its_time_limit_ends_and_keeps_its_changes() {
    let lib = library();
    let tokens = TokenStore::new();
    let tasks = Tasks::default();
    let mut s = spec(&lib, "Tidy");
    s.time_limit = Duration::from_millis(500);
    let id = start_api(
        &lib,
        &tasks,
        &tokens,
        &s,
        vec![Run {
            calls: vec![move_call(&lib)],
            says: String::new(),
            hang: true,
        }],
    );
    wait_until("the time limit", || !row(&tasks, id).is_active());
    let ended = row(&tasks, id);
    assert_eq!(ended.ended_by, Some(EndedBy::TimeLimit), "{ended:?}");
    assert_eq!(ended.phase, Phase::Ready);
    assert_eq!(tasks.changes(id).unwrap().ops.len(), 1);
    assert!(ended.status_line().contains("time limit"));
}

#[test]
fn a_question_waits_for_a_reply_that_continues_the_task() {
    let lib = library();
    let tokens = TokenStore::new();
    let tasks = Tasks::default();
    let id = start_api(
        &lib,
        &tasks,
        &tokens,
        &spec(&lib, "Plan the trip"),
        vec![
            Run {
                calls: vec![],
                says: "Should the plan keep the old dates?".into(),
                hang: false,
            },
            Run {
                calls: vec![move_call(&lib)],
                says: "Done.".into(),
                hang: false,
            },
        ],
    );
    wait_until("the question", || {
        matches!(row(&tasks, id).phase, Phase::Asking { .. })
    });
    assert!(row(&tasks, id).is_active(), "it waits, it hasn't ended");
    tasks.reply(id, "Yes, keep them");
    wait_until("the end", || !row(&tasks, id).is_active());
    assert_eq!(row(&tasks, id).phase, Phase::Ready);
    assert_eq!(row(&tasks, id).counts.edits, 1);
}

#[test]
fn a_model_that_proposes_nothing_says_so_and_a_failure_keeps_the_set() {
    let lib = library();
    let tokens = TokenStore::new();
    let tasks = Tasks::default();
    let id = start_api(
        &lib,
        &tasks,
        &tokens,
        &spec(&lib, "Do nothing"),
        vec![Run {
            calls: vec![],
            says: "Nothing to do.".into(),
            hang: false,
        }],
    );
    wait_until("the end", || !row(&tasks, id).is_active());
    let r = row(&tasks, id);
    assert_eq!(r.phase, Phase::Nothing);
    assert_eq!(r.status_line(), "The agent didn't propose any changes.");
    assert_eq!(r.last_message, "Nothing to do.");

    // A provider that can't use tools ends the task at once with the reason.
    struct NoTools;
    impl ChatProvider for NoTools {
        fn stream(&self, _: &Prompt) -> Result<Box<dyn Iterator<Item = Event> + Send>, AiError> {
            Ok(Box::new(vec![Event::Done].into_iter()))
        }
    }
    let provider: SharedProvider = Arc::new(Mutex::new(Box::new(NoTools)));
    let id = tasks
        .start(&lib.store, &tokens, &spec(&lib, "x"), "Ollama", move |_| {
            Ok(Arc::new(ApiDriver(provider)))
        })
        .unwrap();
    wait_until("the refusal", || !row(&tasks, id).is_active());
    let r = row(&tasks, id);
    assert!(
        matches!(&r.phase, Phase::Failed { message } if message.contains("can't use tools")),
        "{r:?}"
    );
}

#[test]
fn at_most_two_tasks_work_at_once() {
    let lib = library();
    let tokens = TokenStore::new();
    let tasks = Tasks::default();
    let hang = || Run {
        calls: vec![],
        says: String::new(),
        hang: true,
    };
    let a = start_api(&lib, &tasks, &tokens, &spec(&lib, "a"), vec![hang()]);
    let b = start_api(&lib, &tasks, &tokens, &spec(&lib, "b"), vec![hang()]);
    let provider = Scripted::provider(vec![hang()]);
    let third = tasks.start(&lib.store, &tokens, &spec(&lib, "c"), "x", move |_| {
        Ok(Arc::new(ApiDriver(provider)))
    });
    assert_eq!(
        third.err(),
        Some(kanso_engine::agent_task::runner::CannotStart::TooMany)
    );
    tasks.stop(a);
    tasks.stop(b);
    wait_until("both to end", || tasks.active() == 0);
}
