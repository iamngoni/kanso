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
    // The turn ended, not the session: the token waits, refusing every call.
    assert_eq!(tokens.live_count(), 1, "the session's token stays live");
    assert!(tokens.is_paused(tasks.token(id)));
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
    // Ending the session revokes the token.
    tasks.end_session(id);
    wait_until("the token to end", || tokens.live_count() == 0);
    assert!(!row(&tasks, id).live);
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
    // Stop ends the turn, not the session: the token waits, paused.
    assert!(stopped.live);
    assert!(tokens.is_live(stopped_token(&tasks, id)));
    assert!(tokens.is_paused(stopped_token(&tasks, id)));
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
    assert!(
        !row(&tasks, id).is_active(),
        "a question is not a running turn"
    );
    assert!(tasks.follow_up(id, "Yes, keep them"));
    wait_until("the end", || row(&tasks, id).phase == Phase::Ready);
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
    assert_eq!(r.phase, Phase::Idle);
    assert!(r.live && r.can_follow_up());
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
    // Only running turns count: a session between turns takes none.
    tasks.stop(a);
    wait_until("a to be idle", || !row(&tasks, a).is_active());
    let provider = Scripted::provider(vec![hang()]);
    let third = tasks
        .start(&lib.store, &tokens, &spec(&lib, "c"), "x", move |_| {
            Ok(Arc::new(ApiDriver(provider)))
        })
        .unwrap();
    tasks.stop(third);
    tasks.stop(b);
    wait_until("both to end", || tasks.active() == 0);
}

#[test]
fn a_session_is_unread_until_it_is_seen_and_can_be_found_and_renamed() {
    let lib = library();
    let tokens = TokenStore::new();
    let tasks = Tasks::default();
    let id = start_api(
        &lib,
        &tasks,
        &tokens,
        &spec(&lib, "Tidy the trips"),
        vec![Run {
            calls: vec![],
            says: "All tidy.".into(),
            hang: false,
        }],
    );
    wait_until("the end", || !row(&tasks, id).is_active());
    let r = row(&tasks, id);
    // The goal is the person's own; what the agent said is new.
    assert!(r.unread, "{r:?}");
    assert_eq!(tasks.by_key(&r.key), Some(id));
    tasks.mark_seen(id);
    assert!(!row(&tasks, id).unread);
    tasks.rename(id, "  Trips, tidied  ");
    assert_eq!(row(&tasks, id).title, "Trips, tidied");
    tasks.rename(id, "   ");
    assert_eq!(row(&tasks, id).title, "Trips, tidied");
}

// -------------------------------------------------------- follow-ups

fn idle(tasks: &Tasks, id: kanso_server::TaskId) -> bool {
    !row(tasks, id).is_active()
}

fn said(tasks: &Tasks, id: kanso_server::TaskId) -> Vec<String> {
    use kanso_engine::agent_task::session::Item;
    tasks
        .transcript(id)
        .unwrap()
        .items
        .into_iter()
        .filter_map(|item| match item {
            Item::Agent { text, .. } => Some(text),
            _ => None,
        })
        .collect()
}

/// An ACP agent task on `script`, and the conversation it runs on.
fn start_acp(
    lib: &Library,
    tasks: &Tasks,
    tokens: &TokenStore,
    script: Script,
) -> (kanso_server::TaskId, Arc<kanso_ai::agent::Conversation>) {
    let settings = AgentSettings {
        kind: AgentKind::Custom,
        // Any existing file passes the check; the starter connects the fake.
        command: Some(
            std::env::current_exe()
                .unwrap()
                .to_string_lossy()
                .into_owned(),
        ),
        args: Vec::new(),
        env: Vec::new(),
        workspace: lib.dir.path().join("agents").join("workspace"),
    };
    let conversation = Arc::new(Mutex::new(None));
    let keep = conversation.clone();
    let id = tasks
        .start(
            &lib.store,
            tokens,
            &spec(lib, "Plan the trip"),
            "Claude Agent",
            |open| {
                let access = crate::agent_task_panel::task_access(
                    open,
                    "http://127.0.0.1:9/mcp".to_owned(),
                    tokens.clone(),
                    None::<PathBuf>,
                );
                let made = kanso_ai::agent::task_conversation_with(
                    &settings,
                    Limits::default(),
                    starter(script),
                    access,
                    Duration::from_secs(60),
                );
                *keep.lock().unwrap() = Some(made.clone());
                Ok(Arc::new(crate::agent_task_acp::AcpDriver(made)))
            },
        )
        .unwrap();
    let made = conversation.lock().unwrap().take().unwrap();
    (id, made)
}

#[test]
fn a_follow_up_goes_to_the_same_acp_session() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let lib = library();
    let tokens = TokenStore::new();
    let tasks = Tasks::default();
    let (id, conversation) = start_acp(&lib, &tasks, &tokens, Script::Session);
    wait_until("the first turn", || {
        idle(&tasks, id) && said(&tasks, id).len() == 1
    });
    assert_eq!(
        said(&tasks, id)[0],
        "session=fake-session-1 turn=1 sessions=1"
    );
    assert_eq!(row(&tasks, id).phase, Phase::Idle);
    assert!(conversation.has_session());
    assert!(tokens.is_paused(tasks.token(id)));

    assert!(tasks.follow_up(id, "Add a packing list"));
    assert_eq!(row(&tasks, id).round, 2);
    wait_until("the second turn", || {
        idle(&tasks, id) && said(&tasks, id).len() == 2
    });
    // The same session, its second prompt, and no second `session/new`.
    assert_eq!(
        said(&tasks, id)[1],
        "session=fake-session-1 turn=2 sessions=1"
    );
    assert!(!tasks.follow_up(id, "   "), "a blank message is not sent");
    tasks.end_session(id);
    wait_until("the session to close", || !conversation.has_session());
}

#[test]
fn stop_keeps_the_acp_session_and_end_session_closes_it() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let lib = library();
    let tokens = TokenStore::new();
    let tasks = Tasks::default();
    let (id, conversation) = start_acp(&lib, &tasks, &tokens, Script::Busy);
    // The agent has begun the turn once its first words are in.
    wait_until("the agent to start", || !said(&tasks, id).is_empty());
    assert_eq!(row(&tasks, id).phase, Phase::Running);
    tasks.stop(id);
    wait_until("the stop", || idle(&tasks, id));
    let stopped = row(&tasks, id);
    assert_eq!(stopped.phase, Phase::Idle, "{stopped:?}");
    assert_eq!(stopped.ended_by, Some(EndedBy::Stopped));
    assert!(stopped.live, "Stop keeps the session");
    assert!(conversation.has_session(), "and the agent conversation");
    assert!(tokens.is_paused(tasks.token(id)));
    assert!(stopped.can_follow_up());

    tasks.end_session(id);
    let ended = row(&tasks, id);
    assert!(!ended.live && !ended.can_follow_up());
    assert!(
        !tasks.follow_up(id, "More?"),
        "an ended session takes no message"
    );
    wait_until("the session to close", || !conversation.has_session());
    wait_until("the token to end", || tokens.live_count() == 0);
}

#[test]
fn a_session_idle_too_long_is_closed_and_keeps_its_transcript() {
    let _one = ONE_AT_A_TIME
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let lib = library();
    let tokens = TokenStore::new();
    let tasks = Tasks::default();
    tasks.set_idle_close(Duration::from_millis(600));
    let (id, conversation) = start_acp(&lib, &tasks, &tokens, Script::Session);
    wait_until("the turn", || {
        idle(&tasks, id) && said(&tasks, id).len() == 1
    });
    wait_until("the idle close", || !row(&tasks, id).live);
    assert_eq!(row(&tasks, id).ended_by, Some(EndedBy::IdleClose));
    wait_until("the conversation to close", || !conversation.has_session());
    assert_eq!(said(&tasks, id).len(), 1, "the transcript stays");
    assert_eq!(tokens.live_count(), 0);
}

/// A turn that edits `notes[ix]`.
fn edit_run(lib: &Library, ix: usize, says: &str) -> Run {
    let (id, rev) = &lib.notes[ix];
    Run {
        calls: vec![(
            "update_note".into(),
            json!({"_id": id, "_rev": rev, "body": format!("new text {ix}")}),
        )],
        says: says.into(),
        hang: false,
    }
}

#[test]
fn a_second_round_is_reviewed_and_undone_on_its_own() {
    use kanso_engine::agent_task::review;
    use kanso_engine::agent_task::session::{Item, RoundState};
    let lib = library();
    let tokens = TokenStore::new();
    let tasks = Tasks::default();
    let id = start_api(
        &lib,
        &tasks,
        &tokens,
        &spec(&lib, "Tidy"),
        vec![
            edit_run(&lib, 0, "Round one."),
            edit_run(&lib, 1, "Round two."),
        ],
    );
    wait_until("round 1", || row(&tasks, id).phase == Phase::Ready);
    tasks.apply(id, &Selection::all(1), 10).unwrap();
    assert_eq!(row(&tasks, id).phase, Phase::Applied { changes: 1 });
    assert!(tasks.changes(id).is_none(), "an applied set is closed");

    assert!(tasks.follow_up(id, "Do the next note too"));
    wait_until("round 2", || row(&tasks, id).phase == Phase::Ready);
    let set = tasks.changes(id).expect("a new set for round 2");
    assert_eq!((set.round, set.ops.len()), (2, 1));
    let sheet = review::build(&lib.store, &set, "");
    assert_eq!(sheet.cards.len(), 1);
    assert_eq!(sheet.cards[0].round, 2);
    assert!(!sheet.multi_round);
    let rounds: Vec<(u32, RoundState)> = tasks
        .transcript(id)
        .unwrap()
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Proposed { round, state, .. } => Some((*round, *state)),
            _ => None,
        })
        .collect();
    assert_eq!(
        rounds,
        [(1, RoundState::Applied), (2, RoundState::ToReview)]
    );

    tasks.apply(id, &Selection::all(1), 20).unwrap();
    let body = |ix: usize| lib.store.note(&lib.notes[ix].0).unwrap().unwrap().body;
    assert_eq!(
        (body(0).as_str(), body(1).as_str()),
        ("new text 0", "new text 1")
    );
    // Undo takes back round 2 alone, then round 1.
    assert_eq!(tasks.undo_round(id), Some(2));
    tasks.undo(id, 30).unwrap().unwrap();
    assert_eq!(body(1), "Lisbon hotel lisbon notes");
    assert_eq!(body(0), "new text 0");
    assert_eq!(tasks.undo_round(id), Some(1));
    tasks.undo(id, 40).unwrap().unwrap();
    assert_eq!(body(0), "Lisbon flights lisbon notes");
    assert!(!tasks.can_undo(id));
}

#[test]
fn unreviewed_changes_stay_and_the_review_says_which_round() {
    use kanso_engine::agent_task::review;
    let lib = library();
    let tokens = TokenStore::new();
    let tasks = Tasks::default();
    let id = start_api(
        &lib,
        &tasks,
        &tokens,
        &spec(&lib, "Tidy"),
        vec![edit_run(&lib, 0, "One."), edit_run(&lib, 1, "Two.")],
    );
    wait_until("round 1", || row(&tasks, id).phase == Phase::Ready);
    assert!(tasks.follow_up(id, "And the hotel"));
    wait_until("round 2", || {
        let r = row(&tasks, id);
        r.phase == Phase::Ready && r.round == 2 && r.counts.edits == 2
    });
    let set = tasks.changes(id).unwrap();
    assert_eq!(set.rounds(), [1, 2]);
    let sheet = review::build(&lib.store, &set, "");
    assert!(sheet.multi_round);
    let mut labelled: Vec<u32> = sheet.cards.iter().map(|card| card.round).collect();
    labelled.sort_unstable();
    assert_eq!(labelled, [1, 2]);
    // Both rounds apply together, and one Undo takes both back.
    tasks.apply(id, &Selection::all(2), 10).unwrap();
    assert_eq!(tasks.undo_round(id), Some(2));
    tasks.undo(id, 20).unwrap().unwrap();
    assert_eq!(
        lib.store.note(&lib.notes[0].0).unwrap().unwrap().body,
        "Lisbon flights lisbon notes"
    );
}

#[test]
fn each_turn_gets_its_own_call_budget() {
    let lib = library();
    let tokens = TokenStore::new();
    let tasks = Tasks::default();
    let mut s = spec(&lib, "Tidy");
    s.max_calls = 1;
    let id = start_api(
        &lib,
        &tasks,
        &tokens,
        &s,
        vec![edit_run(&lib, 0, "One."), edit_run(&lib, 1, "Two.")],
    );
    wait_until("round 1", || row(&tasks, id).phase == Phase::Ready);
    assert!(tasks.follow_up(id, "One more"));
    wait_until("round 2", || {
        let r = row(&tasks, id);
        r.phase == Phase::Ready && r.round == 2
    });
    // The second call was within the second turn's budget of one.
    assert_eq!(row(&tasks, id).counts.edits, 2);
}

#[test]
fn a_follow_up_waits_while_two_turns_are_running() {
    let lib = library();
    let tokens = TokenStore::new();
    let tasks = Tasks::default();
    let hang = || Run {
        calls: vec![],
        says: String::new(),
        hang: true,
    };
    let quiet = start_api(
        &lib,
        &tasks,
        &tokens,
        &spec(&lib, "a"),
        vec![
            Run {
                calls: vec![],
                says: "Done.".into(),
                hang: false,
            },
            hang(),
        ],
    );
    wait_until("the first to idle", || {
        row(&tasks, quiet).phase == Phase::Idle
    });
    let b = start_api(&lib, &tasks, &tokens, &spec(&lib, "b"), vec![hang()]);
    let c = start_api(&lib, &tasks, &tokens, &spec(&lib, "c"), vec![hang()]);
    wait_until("two running", || tasks.active() == 2);
    assert!(tasks.follow_up(quiet, "Again"));
    assert_eq!(row(&tasks, quiet).phase, Phase::Waiting);
    std::thread::sleep(Duration::from_millis(400));
    assert_eq!(
        row(&tasks, quiet).phase,
        Phase::Waiting,
        "it waits its turn"
    );
    tasks.stop(b);
    wait_until("it takes b's place", || {
        row(&tasks, quiet).phase == Phase::Running
    });
    tasks.stop(c);
    tasks.stop(quiet);
    wait_until("all idle", || tasks.active() == 0);
}
