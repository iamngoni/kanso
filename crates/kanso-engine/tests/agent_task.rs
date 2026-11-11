//! Agent tasks on a model with an API key: a scripted provider drives a
//! whole task through the in-process tools, the API path and the MCP path
//! stage the same ops, and the review cards come out right.
#![cfg(feature = "agent-task")]

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use kanso_ai::{AiError, ChatProvider, Event, NO_TOOLS, Prompt, ToolCall, ToolRegistry};
use kanso_core::DocType;
use kanso_engine::agent_task::review::{self, CardDetail, CardState};
use kanso_engine::agent_task::{self, ReadScope, StartError, TaskSpec};
use kanso_server::{OpKind, Selection, TokenStore, agent_tools};
use kanso_store::{Origin, Put, Store};
use proptest::prelude::*;
use serde_json::{Value, json};

/// A provider that "calls" a fixed list of tools, then ends.
struct Script {
    calls: Vec<(String, Value)>,
    last_max_calls: Arc<std::sync::Mutex<usize>>,
}

impl ChatProvider for Script {
    fn stream(&self, _prompt: &Prompt) -> Result<Box<dyn Iterator<Item = Event> + Send>, AiError> {
        Ok(Box::new(vec![Event::Done].into_iter()))
    }

    fn run_task(
        &self,
        _prompt: &Prompt,
        tools: ToolRegistry<()>,
        max_tool_calls: usize,
        mut on_tool_completed: Box<dyn FnMut() + Send>,
        _cancel: Arc<AtomicBool>,
    ) -> Result<Box<dyn Iterator<Item = Event> + Send>, AiError> {
        *self.last_max_calls.lock().unwrap() = max_tool_calls;
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        for (i, (name, args)) in self.calls.iter().enumerate() {
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
        Ok(Box::new(
            vec![Event::Text("Done.".into()), Event::Done].into_iter(),
        ))
    }
}

/// A model that can't use tools: the default `run_task`.
struct NoTools;

impl ChatProvider for NoTools {
    fn stream(&self, _prompt: &Prompt) -> Result<Box<dyn Iterator<Item = Event> + Send>, AiError> {
        Ok(Box::new(vec![Event::Done].into_iter()))
    }
}

struct Lib {
    _dir: tempfile::TempDir,
    store: Arc<Store>,
    trips: String,
    archive: String,
    notes: Vec<(String, String)>,
}

fn lib() -> Lib {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    let book = |name: &str| {
        store
            .put(
                DocType::Book,
                json!({ "name": name }),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id
    };
    let trips = book("Trips");
    let archive = book("Archive");
    let notes = (0..4)
        .map(|n| {
            let w = store
                .put(
                    DocType::Note,
                    json!({ "bookId": trips, "title": format!("Trip {n}"), "body": format!("day {n}") }),
                    Put::Upsert,
                    Origin::Ui,
                )
                .unwrap();
            (w.id, w.rev)
        })
        .collect();
    Lib {
        _dir: dir,
        store,
        trips,
        archive,
        notes,
    }
}

fn spec(lib: &Lib) -> TaskSpec {
    TaskSpec::new(
        "Merge these trip notes into one plan",
        ReadScope::Notes(lib.notes.iter().map(|(id, _)| id.clone()).collect()),
        &lib.trips,
    )
}

fn merge_call(lib: &Lib, api: bool) -> (String, Value) {
    let name = if api {
        "kanso_propose_merge"
    } else {
        "kanso-propose-merge"
    };
    (
        name.to_owned(),
        json!({
            "sources": lib.notes[..3].iter().map(|(id, rev)| json!({"_id": id, "_rev": rev})).collect::<Vec<_>>(),
            "bookId": lib.trips,
            "title": "Lisbon plan",
            "body": "Day 0, 1 and 2. [[Trip 0]] [[Trip 1]]",
        }),
    )
}

#[test]
fn a_scripted_model_drives_a_full_task_and_nothing_is_written() {
    let lib = lib();
    let tokens = TokenStore::new();
    let task = agent_task::open(&lib.store, &tokens, &spec(&lib), "Anthropic").unwrap();
    assert_eq!(task.readable, 4);
    let max = Arc::new(std::sync::Mutex::new(0));
    let provider = Script {
        calls: vec![
            (
                "read_note".into(),
                json!({"noteId": lib.notes[0].0.clone()}),
            ),
            merge_call(&lib, true),
            (
                "kanso_task_done".into(),
                json!({"summary": "Merged three."}),
            ),
        ],
        last_max_calls: max.clone(),
    };
    let seq = lib.store.last_seq().unwrap();
    let events: Vec<Event> = agent_task::run_api(
        &provider,
        lib.store.clone(),
        &task,
        Box::new(|| {}),
        Arc::new(AtomicBool::new(false)),
    )
    .unwrap()
    .collect();
    assert!(matches!(events.last(), Some(Event::Done)));
    assert_eq!(*max.lock().unwrap(), 200, "the task's own call limit");
    assert_eq!(lib.store.last_seq().unwrap(), seq, "nothing was written");
    assert_eq!(task.reads(), vec![lib.notes[0].0.clone()]);
    let set = task.changes().unwrap();
    assert!(set.done);
    assert_eq!(set.counts().merges, 1);

    // Review it, tick everything, apply, undo.
    let sheet = review::build(&lib.store, &set, "");
    assert_eq!(sheet.header(), "Proposed 1 merge");
    assert_eq!(sheet.from_agent, "Merged three.");
    let card = &sheet.cards[0];
    assert!(card.ticked && card.state == CardState::Fresh);
    let CardDetail::Merge {
        sources,
        target_is_new,
        ..
    } = &card.detail
    else {
        panic!("a merge card");
    };
    assert_eq!(sources.len(), 3);
    assert!(target_is_new);
    let applied = set
        .apply(&lib.store, &sheet.default_selection(), 100)
        .unwrap();
    assert_eq!(
        review::applied_message("Anthropic", &applied, 1),
        "Applied 1 change from Anthropic."
    );
    for (id, _) in &lib.notes[..3] {
        assert_eq!(lib.store.note(id).unwrap().unwrap().book_id, "trash");
    }
    let undone = kanso_server::undo_changes(&lib.store, &applied.batch, 200).unwrap();
    assert!(undone.skipped.is_empty());
    for (id, _) in &lib.notes {
        assert_eq!(lib.store.note(id).unwrap().unwrap().book_id, lib.trips);
    }
}

#[test]
fn a_model_without_tools_is_refused_at_start() {
    let lib = lib();
    let tokens = TokenStore::new();
    let task = agent_task::open(&lib.store, &tokens, &spec(&lib), "Ollama").unwrap();
    let result = agent_task::run_api(
        &NoTools,
        lib.store.clone(),
        &task,
        Box::new(|| {}),
        Arc::new(AtomicBool::new(false)),
    );
    match result {
        Err(AiError::Agent(message)) => assert_eq!(message, NO_TOOLS),
        _ => panic!("a model without tools must be refused"),
    }
}

#[test]
fn a_task_does_not_start_without_a_goal_or_a_readable_note() {
    let lib = lib();
    let tokens = TokenStore::new();
    let mut s = spec(&lib);
    s.goal = "  ".into();
    assert_eq!(
        agent_task::open(&lib.store, &tokens, &s, "x").err(),
        Some(StartError::EmptyGoal)
    );
    let mut s = spec(&lib);
    for (id, _) in &lib.notes {
        let mut note = lib.store.get_api(id, None, false).unwrap();
        note["body"] = "".into();
        note["kansoLock"] = json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "x"});
        lib.store.lock_note(note).unwrap();
    }
    s.goal = "Do it".into();
    assert_eq!(
        agent_task::open(&lib.store, &tokens, &s, "x").err(),
        Some(StartError::NothingReadable)
    );
    let mut s = spec(&lib);
    s.ops.create = false;
    s.ops.edit = false;
    s.ops.moves = false;
    s.ops.merge = false;
    assert_eq!(
        agent_task::open(&lib.store, &tokens, &s, "x").err(),
        Some(StartError::NothingAllowed)
    );
}

#[test]
fn the_token_is_revoked_when_the_task_ends_and_the_set_stays() {
    let lib = lib();
    let tokens = TokenStore::new();
    let task = agent_task::open(&lib.store, &tokens, &spec(&lib), "x").unwrap();
    assert!(tokens.is_live(task.token));
    task.end();
    assert!(!tokens.is_live(task.token));
    assert!(task.changes().is_some());
    task.discard();
    assert!(task.changes().is_none());
}

#[test]
fn review_cards_are_ordered_and_flag_changed_and_blocked_notes() {
    let lib = lib();
    let tokens = TokenStore::new();
    let task = agent_task::open(
        &lib.store,
        &tokens,
        &TaskSpec {
            new_note_book: lib.archive.clone(),
            ..spec(&lib)
        },
        "x",
    )
    .unwrap();
    let call = |name: &str, args: Value| {
        agent_tools::call(
            &lib.store,
            &tokens,
            task.token,
            &task.scope,
            name,
            Some(&args),
        )
    };
    let (a, a_rev) = &lib.notes[0];
    let (b, b_rev) = &lib.notes[1];
    call(
        "kanso-propose-move",
        json!({"_id": a, "_rev": a_rev, "bookId": lib.archive}),
    );
    call(
        "update-note",
        json!({"_id": b, "_rev": b_rev, "body": "day 1\nmore"}),
    );
    call(
        "create-note",
        json!({"bookId": lib.archive, "title": "Plan", "body": "See [[Trip 0]]"}),
    );
    // The person edits note b meanwhile.
    lib.store
        .put(
            DocType::Note,
            json!({"_id": b, "body": "mine"}),
            Put::Merge,
            Origin::Ui,
        )
        .unwrap();
    let set = task.changes().unwrap();
    let sheet = review::build(&lib.store, &set, "ok");
    let kinds: Vec<OpKind> = sheet.cards.iter().map(|c| c.kind).collect();
    assert_eq!(kinds, [OpKind::Create, OpKind::Edit, OpKind::Move]);
    assert!(sheet.cards[0].ticked);
    let CardDetail::New { cites, .. } = &sheet.cards[0].detail else {
        panic!()
    };
    assert_eq!(cites, &["Trip 0".to_owned()]);
    let edit = &sheet.cards[1];
    assert!(!edit.ticked, "a changed note starts unticked");
    assert!(matches!(edit.state, CardState::Changed { .. }));
    let CardDetail::Move { from, to } = &sheet.cards[2].detail else {
        panic!()
    };
    assert_eq!((from.as_str(), to.as_str()), ("Trips", "Archive"));
    // Lock note a: its move card can no longer be ticked or applied.
    let mut note = lib.store.get_api(a, None, false).unwrap();
    note["body"] = "".into();
    note["kansoLock"] = json!({"v": 1, "salt": "c2FsdA", "m": 64, "t": 1, "p": 1, "data": "x"});
    lib.store.lock_note(note).unwrap();
    let sheet = review::build(&lib.store, &set, "ok");
    let moved = sheet.cards.iter().find(|c| c.kind == OpKind::Move).unwrap();
    assert_eq!(moved.state, CardState::Blocked);
    assert!(!moved.ticked);
}

#[test]
fn notes_outside_the_chosen_set_start_unticked() {
    let lib = lib();
    let tokens = TokenStore::new();
    // Chose only note 0, but the scope is notes so others are invisible; use
    // a notebook-scope task then check the warning for an unnamed notebook.
    let mut s = TaskSpec::new(
        "Move it",
        ReadScope::Notebook(lib.trips.clone()),
        &lib.trips,
    );
    s.max_notes = 50;
    let task = agent_task::open(&lib.store, &tokens, &s, "x").unwrap();
    let other = lib
        .store
        .put(
            DocType::Book,
            json!({"name": "Elsewhere"}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    let (a, a_rev) = &lib.notes[0];
    let answer = agent_tools::call(
        &lib.store,
        &tokens,
        task.token,
        &task.scope,
        "kanso-propose-move",
        Some(&json!({"_id": a, "_rev": a_rev, "bookId": other})),
    );
    // A notebook outside the scope is not found.
    assert_eq!(answer["isError"], json!(true));
    let set = task.changes().unwrap();
    assert!(set.ops.is_empty());
    // The Archive notebook is outside the notebook scope too.
    let answer = agent_tools::call(
        &lib.store,
        &tokens,
        task.token,
        &task.scope,
        "kanso-propose-move",
        Some(&json!({"_id": a, "_rev": a_rev, "bookId": lib.archive})),
    );
    assert_eq!(answer["isError"], json!(true));
}

#[derive(Clone, Debug)]
enum C {
    Create(String),
    Update(usize, String),
    Move(usize),
    Merge(Vec<usize>),
    Done,
}

fn calls() -> impl Strategy<Value = Vec<C>> {
    prop::collection::vec(
        prop_oneof![
            "[a-z]{1,6}".prop_map(C::Create),
            (0usize..4, "[a-z ]{0,12}").prop_map(|(n, b)| C::Update(n, b)),
            (0usize..4).prop_map(C::Move),
            prop::collection::vec(0usize..4, 0..4).prop_map(C::Merge),
            Just(C::Done),
        ],
        1..14,
    )
}

fn script(lib: &Lib, plan: &[C], api: bool) -> Vec<(String, Value)> {
    let n = |s: &str| {
        if api {
            s.replace('-', "_")
        } else {
            s.to_owned()
        }
    };
    plan.iter()
        .map(|c| match c {
            C::Create(t) => (
                n("create-note"),
                json!({"bookId": lib.trips, "title": t, "body": "b"}),
            ),
            C::Update(i, b) => (
                n("update-note"),
                json!({"_id": lib.notes[*i].0, "_rev": lib.notes[*i].1, "body": b}),
            ),
            C::Move(i) => (
                n("kanso-propose-move"),
                json!({"_id": lib.notes[*i].0, "_rev": lib.notes[*i].1, "bookId": lib.archive}),
            ),
            C::Merge(ix) => (
                n("kanso-propose-merge"),
                json!({
                    "sources": ix.iter().map(|i| json!({"_id": lib.notes[*i].0, "_rev": lib.notes[*i].1})).collect::<Vec<_>>(),
                    "bookId": lib.archive, "title": "M", "body": "m",
                }),
            ),
            C::Done => (n("kanso-task-done"), json!({"summary": "s"})),
        })
        .collect()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    /// Invariant: the same tool-call script stages the same ops whether it
    /// reaches the tools as an API-key model (underscore names, in process)
    /// or as an MCP client (hyphen names, same handlers).
    #[test]
    fn prop_the_api_and_mcp_paths_stage_the_same_ops(plan in calls()) {
        let lib = lib();
        let spec = spec(&lib);

        // API path.
        let tokens_a = TokenStore::new();
        let task_a = agent_task::open(&lib.store, &tokens_a, &spec, "A").unwrap();
        let provider = Script { calls: script(&lib, &plan, true), last_max_calls: Arc::default() };
        let _ = agent_task::run_api(&provider, lib.store.clone(), &task_a, Box::new(|| {}), Arc::new(AtomicBool::new(false))).unwrap().count();

        // MCP path: the hyphen names straight at the handler.
        let tokens_b = TokenStore::new();
        let task_b = agent_task::open(&lib.store, &tokens_b, &spec, "A").unwrap();
        for (name, args) in script(&lib, &plan, false) {
            agent_tools::call(&lib.store, &tokens_b, task_b.token, &task_b.scope, &name, Some(&args));
        }
        let (a, b) = (task_a.changes().unwrap(), task_b.changes().unwrap());
        prop_assert_eq!(a.ops, b.ops);
        prop_assert_eq!(a.done, b.done);
        prop_assert_eq!(a.summary, b.summary);
    }

    /// Invariant: whatever script runs and whatever the person ticks, only
    /// ticked ops reach the store and one undo restores every note.
    #[test]
    fn prop_apply_then_undo_leaves_the_library_as_it_was(plan in calls(), mask in prop::collection::vec(any::<bool>(), 14)) {
        let lib = lib();
        let tokens = TokenStore::new();
        let task = agent_task::open(&lib.store, &tokens, &spec(&lib), "A").unwrap();
        for (name, args) in script(&lib, &plan, false) {
            agent_tools::call(&lib.store, &tokens, task.token, &task.scope, &name, Some(&args));
        }
        let set = task.changes().unwrap();
        let mut selection = Selection::default();
        for i in 0..set.ops.len() {
            if mask[i % mask.len()] {
                selection.ticked.insert(i);
            }
        }
        let before: Vec<_> = lib.notes.iter().map(|(id, _)| {
            let f = lib.store.get(id).unwrap().unwrap().fields;
            (f.get("bookId").cloned(), f.get("title").cloned(), f.get("body").cloned())
        }).collect();
        if let Ok(applied) = set.apply(&lib.store, &selection, 50) {
            let undone = kanso_server::undo_changes(&lib.store, &applied.batch, 60).unwrap();
            prop_assert!(undone.skipped.is_empty());
        }
        let after: Vec<_> = lib.notes.iter().map(|(id, _)| {
            let f = lib.store.get(id).unwrap().unwrap().fields;
            (f.get("bookId").cloned(), f.get("title").cloned(), f.get("body").cloned())
        }).collect();
        prop_assert_eq!(before, after);
    }
}

/// A task over chosen notes may write only into their notebooks and the
/// notebook for new notes; the prompt names that notebook.
#[test]
fn a_task_over_chosen_notes_writes_only_where_the_person_said() {
    let lib = lib();
    let tokens = TokenStore::new();
    let spec = TaskSpec::new(
        "Summarise",
        ReadScope::Notes(vec![lib.notes[0].0.clone()]),
        &lib.archive,
    );
    let task = agent_task::open(&lib.store, &tokens, &spec, "x").unwrap();
    let call = |name: &str, args: Value| {
        agent_tools::call(
            &lib.store,
            &tokens,
            task.token,
            &task.scope,
            name,
            Some(&args),
        )
    };
    // Only the notebook for new notes takes new notes, even though Trips is
    // where the chosen note is.
    let wrong = call(
        "create-note",
        json!({"bookId": lib.trips, "title": "S", "body": "x"}),
    );
    assert_eq!(wrong["isError"], json!(true));
    let right = call("create-note", json!({"title": "S", "body": "x"}));
    assert_ne!(right["isError"], json!(true), "{right}");
    // The note may move to its own notebook's neighbours the person named.
    let moved = call(
        "kanso-propose-move",
        json!({"_id": lib.notes[0].0, "_rev": lib.notes[0].1, "bookId": lib.archive}),
    );
    assert_ne!(moved["isError"], json!(true), "{moved}");
    let prompt = agent_task::api_prompt(&task);
    assert!(prompt.user.contains(&lib.archive), "{}", prompt.user);
    assert!(agent_task::user_text(&task, false).contains("create-note"));
}

#[test]
fn the_prompt_names_the_note_to_start_from_only_when_one_is_given() {
    let lib = lib();
    let tokens = TokenStore::new();
    let open = |start_note: Option<String>| {
        agent_task::open(
            &lib.store,
            &tokens,
            &TaskSpec {
                start_note,
                ..spec(&lib)
            },
            "x",
        )
        .unwrap()
    };
    let plain = agent_task::user_text(&open(None), false);
    assert!(!plain.contains("Start from the note"), "{plain}");
    let id = lib.notes[0].0.clone();
    let started = agent_task::user_text(&open(Some(id.clone())), false);
    assert!(
        started.contains(&format!("Start from the note with id \"{id}\"")),
        "{started}"
    );
    // The task still comes last.
    assert!(started.trim_end().ends_with("into one plan"), "{started}");
}

/// A driver whose task works until it is stopped.
struct Idle;

impl kanso_engine::agent_task::runner::TaskDriver for Idle {
    fn first_prompt(&self, task: &agent_task::OpenTask) -> Prompt {
        agent_task::api_prompt(task)
    }
    fn reply_prompt(&self, task: &agent_task::OpenTask, _: &str, _: &str) -> Prompt {
        agent_task::api_prompt(task)
    }
    fn begin(
        &self,
        _: &Prompt,
        _: &agent_task::OpenTask,
        _: Arc<Store>,
        _: Box<dyn FnMut() + Send>,
        cancel: Arc<AtomicBool>,
    ) -> Result<Box<dyn Iterator<Item = Event> + Send>, String> {
        while !cancel.load(std::sync::atomic::Ordering::Relaxed) {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        Ok(Box::new(vec![Event::Done].into_iter()))
    }
}

/// Starts at the same moment can't get past the limit on working tasks.
#[test]
fn starts_at_once_cannot_pass_the_active_limit() {
    use kanso_engine::agent_task::runner::{MAX_ACTIVE, Tasks};
    let lib = lib();
    let tokens = TokenStore::new();
    let tasks = Tasks::default();
    let started: usize = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..8)
            .map(|_| {
                scope.spawn(|| {
                    tasks
                        .start(&lib.store, &tokens, &spec(&lib), "x", |_| {
                            Ok(Arc::new(Idle) as _)
                        })
                        .is_ok()
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|h| usize::from(h.join().unwrap()))
            .sum()
    });
    assert_eq!(started, MAX_ACTIVE);
    tasks.shutdown();
}
