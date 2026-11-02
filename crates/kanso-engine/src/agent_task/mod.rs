//! Agent tasks without a window (issue #196), shared by the desktop app and
//! the phone.
//!
//! A task is a goal, the notes the agent may read, and what it may propose.
//! [`open`] checks all three, mints the task's token and opens its change set
//! in the [`TokenStore`](kanso_server::TokenStore). An ACP agent reaches the
//! set through the notes server over MCP; a model with an API key is driven
//! here, by [`run_api`], whose tools call the same handlers in process
//! ([`kanso_server::agent_tools`]), so both stage the same ops under the same
//! scope. Nothing here writes a note: the person's Apply does
//! ([`kanso_server::ChangeSet::apply`]).
//!
//! [`review`] turns a finished set into the cards the review sheet and the
//! review page show.

pub mod review;
pub mod settings;

use std::collections::{BTreeSet, HashSet};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use kanso_ai::{AiError, ChatProvider, Event, NO_TOOLS, Prompt, RawTool, ToolRegistry};
use kanso_server::{
    AgentScope, ChangeSet, TaskId, TaskLimits, TaskOps, TokenId, TokenStore, agent_tools,
    readable_notes,
};
use kanso_store::Store;
use serde_json::Value;

/// The default time a task may run.
pub const DEFAULT_TIME_LIMIT: Duration = Duration::from_secs(20 * 60);
/// The choices Settings offers for the time limit, in minutes.
pub const TIME_LIMITS_MIN: [u64; 3] = [5, 20, 60];
/// The default tool-call limit.
pub const DEFAULT_MAX_CALLS: usize = 200;
/// The default and offered limits on how many notes one task may read.
pub const DEFAULT_MAX_NOTES: usize = 200;
pub const MAX_NOTES_CHOICES: [usize; 3] = [50, 200, 1000];
/// How long a task waits for the person's reply to a question before it
/// stops and keeps its change set.
pub const REPLY_WAIT: Duration = Duration::from_secs(30 * 60);

/// What the agent may read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReadScope {
    /// These notes, by ID.
    Notes(Vec<String>),
    /// A notebook and the notebooks inside it.
    Notebook(String),
    /// The notebooks the person allowed agents (Settings > AI > Agent).
    Allowed(Vec<String>),
    /// Every unlocked notebook (the allowed set is "all").
    Everything,
}

/// The New agent task sheet, as a value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskSpec {
    pub goal: String,
    pub read: ReadScope,
    pub ops: TaskOps,
    /// Where new notes go.
    pub new_note_book: String,
    pub time_limit: Duration,
    pub max_calls: usize,
    pub max_notes: usize,
}

impl TaskSpec {
    /// A spec with the limits and defaults `settings` give.
    pub fn with_settings(
        goal: &str,
        read: ReadScope,
        new_note_book: &str,
        settings: &settings::AgentTaskSettings,
    ) -> TaskSpec {
        let settings = settings.normalised();
        TaskSpec {
            ops: settings.ops(),
            time_limit: settings.time_limit(),
            max_notes: settings.max_notes,
            ..TaskSpec::new(goal, read, new_note_book)
        }
    }

    pub fn new(goal: &str, read: ReadScope, new_note_book: &str) -> TaskSpec {
        TaskSpec {
            goal: goal.trim().to_owned(),
            read,
            ops: TaskOps::default(),
            new_note_book: new_note_book.to_owned(),
            time_limit: DEFAULT_TIME_LIMIT,
            max_calls: DEFAULT_MAX_CALLS,
            max_notes: DEFAULT_MAX_NOTES,
        }
    }
}

/// Why a task can't start. Each is a whole sentence for the sheet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StartError {
    EmptyGoal,
    /// No note in scope can be shown to an agent.
    NothingReadable,
    /// Nothing is allowed to be proposed.
    NothingAllowed,
}

impl StartError {
    pub fn message(&self) -> &'static str {
        match self {
            StartError::EmptyGoal => "Say what the agent should do.",
            StartError::NothingReadable => "None of these notes can be shown to an agent.",
            StartError::NothingAllowed => "Choose at least one thing the agent may propose.",
        }
    }
}

/// A task that has started: its token and change set are open.
#[derive(Clone)]
pub struct OpenTask {
    pub id: TaskId,
    pub token: TokenId,
    pub scope: Arc<AgentScope>,
    pub tokens: TokenStore,
    pub spec: TaskSpec,
    pub agent: String,
    /// How many notes the agent may read, when it started.
    pub readable: usize,
    /// The token's secret, for the notes server grant an ACP agent gets. Never
    /// printed.
    pub secret: Arc<String>,
}

/// The scope a task with `spec` has (without minting anything).
pub fn scope_for(spec: &TaskSpec, agent: &str, id: TaskId) -> AgentScope {
    let limits = TaskLimits {
        lifetime: spec.time_limit + Duration::from_secs(60),
        calls: spec.max_calls,
        max_notes: spec.max_notes,
    };
    let (books, notes): (Option<HashSet<String>>, Option<HashSet<String>>) = match &spec.read {
        ReadScope::Notes(ids) => (None, Some(ids.iter().cloned().collect())),
        ReadScope::Notebook(book) => (Some(HashSet::from([book.clone()])), None),
        ReadScope::Allowed(books) => (Some(books.iter().cloned().collect()), None),
        ReadScope::Everything => (None, None),
    };
    AgentScope::task(books, notes, agent, id, spec.ops, limits)
}

/// The notebooks the person named, for the "moved into a notebook you
/// didn't name" warning.
fn named_books(spec: &TaskSpec) -> BTreeSet<String> {
    let mut named = BTreeSet::from([spec.new_note_book.clone()]);
    match &spec.read {
        ReadScope::Notebook(book) => {
            named.insert(book.clone());
        }
        ReadScope::Allowed(books) => named.extend(books.iter().cloned()),
        _ => {}
    }
    named
}

/// Starts a task: checks the sheet, mints the token and opens the change
/// set. `agent` is what the person calls the AI ("Claude Agent").
pub fn open(
    store: &Store,
    tokens: &TokenStore,
    spec: &TaskSpec,
    agent: &str,
) -> Result<OpenTask, StartError> {
    if spec.goal.trim().is_empty() {
        return Err(StartError::EmptyGoal);
    }
    if !(spec.ops.create || spec.ops.edit || spec.ops.moves || spec.ops.merge) {
        return Err(StartError::NothingAllowed);
    }
    let sets = tokens.changesets();
    let id = sets.next_id();
    let scope = scope_for(spec, agent, id);
    let readable = readable_notes(store, &scope).len();
    if readable == 0 {
        return Err(StartError::NothingReadable);
    }
    let minted = tokens.mint(scope.clone());
    let scope = Arc::new(scope);
    sets.open(ChangeSet::new(
        id,
        &spec.goal,
        agent,
        scope.clone(),
        named_books(spec),
    ));
    Ok(OpenTask {
        id,
        token: minted.id,
        scope,
        tokens: tokens.clone(),
        spec: spec.clone(),
        agent: agent.to_owned(),
        readable,
        secret: Arc::new(minted.token),
    })
}

impl OpenTask {
    /// A copy of the change set so far.
    pub fn changes(&self) -> Option<ChangeSet> {
        self.tokens.changesets().get(self.id)
    }

    /// The notes the agent has been shown, in order.
    pub fn reads(&self) -> Vec<String> {
        self.tokens.reads(self.token)
    }

    /// Ends the token. The change set stays for review.
    pub fn end(&self) {
        self.tokens.revoke(self.token);
    }

    /// Throws the change set away.
    pub fn discard(&self) {
        self.end();
        self.tokens.changesets().remove(self.id);
    }
}

/// The tool name an API-key model sees: `search-notes` is `search_notes`.
pub fn api_name(mcp_name: &str) -> String {
    mcp_name.replace('-', "_")
}

/// The MCP name for a tool name an API-key model called.
fn mcp_name(api_name: &str, known: &[String]) -> Option<String> {
    known
        .iter()
        .find(|n| api_name == n.replace('-', "_"))
        .cloned()
}

/// What a tool answer says, as one string for the model.
pub fn answer_text(result: &Value) -> String {
    let text = result
        .get("content")
        .and_then(Value::as_array)
        .map(|parts| {
            parts
                .iter()
                .filter_map(|p| p.get("text").and_then(Value::as_str))
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default();
    if result.get("isError") == Some(&Value::Bool(true)) {
        format!("Error: {text}")
    } else {
        text
    }
}

/// The tools of `task` for a model with an API key: the definitions an ACP
/// agent gets from the notes server, under underscore names, each running
/// the same handler in process.
pub fn registry(store: Arc<Store>, task: &OpenTask) -> ToolRegistry<()> {
    let mut registry = ToolRegistry::new();
    let definitions = agent_tools::definitions_for(&task.scope);
    let known: Arc<Vec<String>> = Arc::new(
        definitions
            .iter()
            .filter_map(|d| d.get("name").and_then(Value::as_str).map(str::to_owned))
            .collect(),
    );
    for definition in definitions {
        let Some(name) = definition.get("name").and_then(Value::as_str) else {
            continue;
        };
        let description = definition
            .get("description")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let mut schema = definition
            .get("inputSchema")
            .cloned()
            .unwrap_or(Value::Null);
        if let Some(object) = schema.as_object_mut() {
            object.remove("$schema");
        }
        let (store, tokens, token, scope) = (
            store.clone(),
            task.tokens.clone(),
            task.token,
            task.scope.clone(),
        );
        let known = known.clone();
        let mcp = name.to_owned();
        registry.register(RawTool::new(
            api_name(name),
            description,
            schema,
            Arc::new(move |arguments: Value| {
                let name = mcp_name(&api_name(&mcp), &known).unwrap_or_else(|| mcp.clone());
                Value::String(answer_text(&agent_tools::call(
                    &store,
                    &tokens,
                    token,
                    &scope,
                    &name,
                    Some(&arguments),
                )))
            }),
        ));
    }
    registry
}

/// The prompt an API-key model gets for `task`.
pub fn api_prompt(task: &OpenTask) -> Prompt {
    // The model sees the tools under underscore names.
    let mut system = kanso_ai::prompt::AGENT_TASK_SYSTEM.to_owned();
    for name in [
        "create-note",
        "update-note",
        "patch-note",
        "kanso-propose-move",
        "kanso-propose-merge",
        "kanso-task-done",
    ] {
        system = system.replace(name, &api_name(name));
    }
    Prompt {
        system,
        user: format!("Task: {}", task.spec.goal.trim()),
    }
}

/// Runs `task` on a model with an API key. The events are the model's words
/// and its end; the change set fills as the tools run. A model that can't use
/// tools ends with [`NO_TOOLS`].
pub fn run_api(
    provider: &dyn ChatProvider,
    store: Arc<Store>,
    task: &OpenTask,
    on_tool_completed: Box<dyn FnMut() + Send>,
    cancel: Arc<AtomicBool>,
) -> Result<Box<dyn Iterator<Item = Event> + Send>, AiError> {
    run_api_with_prompt(
        provider,
        store,
        task,
        api_prompt(task),
        on_tool_completed,
        cancel,
    )
}

/// [`run_api`] with another prompt: a reply to the model's question
/// continues the task with what was said.
pub fn run_api_with_prompt(
    provider: &dyn ChatProvider,
    store: Arc<Store>,
    task: &OpenTask,
    prompt: Prompt,
    on_tool_completed: Box<dyn FnMut() + Send>,
    cancel: Arc<AtomicBool>,
) -> Result<Box<dyn Iterator<Item = Event> + Send>, AiError> {
    provider.run_task(
        &prompt,
        registry(store, task),
        task.spec.max_calls,
        on_tool_completed,
        cancel,
    )
}

/// Whether a task's last words say the model can't use tools.
pub fn is_no_tools(message: &str) -> bool {
    message == NO_TOOLS
}
