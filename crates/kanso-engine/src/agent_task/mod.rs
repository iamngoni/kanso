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

pub mod blocks;
pub mod model;
pub mod review;
pub mod runner;
pub mod session;
pub mod settings;

use std::collections::{BTreeSet, HashSet, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;

use kanso_ai::{AiError, ChatProvider, Event, NO_TOOLS, Prompt, RawTool, ToolRegistry, ToolStatus};
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
/// How long a warm session waits for the next message before it is closed.
/// Its transcript and any changes to review stay.
pub const IDLE_CLOSE: Duration = Duration::from_secs(30 * 60);

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
    /// The note the agent should start from (the one open when the task was
    /// made), if any. It only names a place to begin: it widens nothing.
    pub start_note: Option<String>,
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
            start_note: None,
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

/// The notebooks a task may write into: the new-note notebook, the notebooks
/// of the notes it was given, or the notebooks it may read (with the ones
/// inside). `None` for a task that may read every notebook. A note's text
/// can never widen it.
fn write_books(store: &Store, spec: &TaskSpec) -> Option<HashSet<String>> {
    let mut books: HashSet<String> = HashSet::new();
    if !spec.new_note_book.trim().is_empty() {
        books.insert(spec.new_note_book.trim().to_owned());
    }
    let roots: Vec<String> = match &spec.read {
        ReadScope::Everything => return None,
        ReadScope::Notes(ids) => {
            books.extend(
                ids.iter()
                    .filter_map(|id| store.note(id).ok().flatten())
                    .map(|note| note.book_id),
            );
            return Some(books);
        }
        ReadScope::Notebook(book) => vec![book.clone()],
        ReadScope::Allowed(allowed) => allowed.clone(),
    };
    let all = store.books().unwrap_or_default();
    let mut queue = roots;
    let mut seen = HashSet::new();
    while let Some(id) = queue.pop() {
        if seen.insert(id.clone()) {
            queue.extend(
                all.iter()
                    .filter(|b| b.parent_book_id.as_deref() == Some(id.as_str()))
                    .map(|b| b.id.clone()),
            );
        }
    }
    books.extend(seen);
    Some(books)
}

/// The scope a task with `spec` has (without minting anything).
pub fn scope_for(store: &Store, spec: &TaskSpec, agent: &str, id: TaskId) -> AgentScope {
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
        .with_writes(write_books(store, spec), &spec.new_note_book)
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
    let scope = scope_for(store, spec, agent, id);
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
        self.tokens.task_reads(self.id)
    }

    /// The limits one turn of the session has: its own time and calls.
    pub fn limits(&self) -> TaskLimits {
        TaskLimits {
            lifetime: self.spec.time_limit + Duration::from_secs(60),
            calls: self.spec.max_calls,
            max_notes: self.spec.max_notes,
        }
    }

    /// The turn ended: the token stays live but refuses every call.
    pub fn pause(&self) {
        self.tokens.pause_task(self.id);
    }

    /// A follow-up begins a turn: the token takes calls again, with a
    /// fresh clock and budget. `false` when it was evicted or revoked.
    pub fn resume(&self) -> bool {
        self.tokens.resume_task(self.id, self.limits())
    }

    /// A new, empty change set for round `round`, when there is none.
    pub fn open_round(&self, round: u32) {
        let sets = self.tokens.changesets();
        if sets.with(self.id, |set| set.begin_round(round)).is_none() {
            let mut set = ChangeSet::new(
                self.id,
                &self.spec.goal,
                &self.agent,
                self.scope.clone(),
                named_books(&self.spec),
            );
            set.round = round;
            sets.open(set);
        }
    }

    /// Ends the session's tokens (the one it started with, and any it
    /// started again with). The change set stays for review.
    pub fn end(&self) {
        self.tokens.revoke(self.token);
        self.tokens.revoke_task(self.id);
    }

    /// Throws the change set away. The session goes on.
    pub fn discard_changes(&self) {
        self.tokens.changesets().remove(self.id);
    }

    /// Ends the session and throws everything of it away.
    pub fn discard(&self) {
        self.end();
        self.discard_changes();
        self.tokens.forget_task(self.id);
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
    registry_with_events(store, task, None)
}

/// The kind of tool a Kanso tool is, in the agent protocol's words.
fn tool_kind(mcp_name: &str) -> &'static str {
    let name = mcp_name.trim_start_matches("kanso-");
    if name.contains("search") {
        "search"
    } else if name.starts_with("propose-move") {
        "move"
    } else if ["create", "update", "patch", "propose"]
        .iter()
        .any(|verb| name.starts_with(verb))
    {
        "edit"
    } else if ["read", "get", "list"].iter().any(|v| name.starts_with(v)) {
        "read"
    } else {
        "other"
    }
}

/// "Search notes" for `search-notes`.
fn tool_title(mcp_name: &str) -> String {
    let words = mcp_name
        .trim_start_matches("kanso-")
        .replace(['-', '_'], " ");
    let mut chars = words.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().chain(chars).collect())
        .unwrap_or(words)
}

/// [`registry`] whose tools also tell `events` that they started and how
/// they ended, as [`Event::Tool`]s.
fn registry_with_events(
    store: Arc<Store>,
    task: &OpenTask,
    events: Option<mpsc::Sender<Event>>,
) -> ToolRegistry<()> {
    let events = events.map(|tx| Arc::new(Mutex::new(tx)));
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
        let events = events.clone();
        let mcp = name.to_owned();
        registry.register(RawTool::new(
            api_name(name),
            description,
            schema,
            Arc::new(move |arguments: Value| {
                let name = mcp_name(&api_name(&mcp), &known).unwrap_or_else(|| mcp.clone());
                let call = format!("api-{}", TOOL_CALLS.fetch_add(1, Ordering::Relaxed));
                let tell = |title: Option<String>, kind: Option<&str>, status: ToolStatus| {
                    if let Some(events) = &events {
                        let tx = events.lock().unwrap_or_else(|e| e.into_inner());
                        tx.send(Event::Tool {
                            id: call.clone(),
                            title,
                            kind: kind.map(str::to_owned),
                            status: Some(status),
                        })
                        .ok();
                    }
                };
                tell(
                    Some(tool_title(&name)),
                    Some(tool_kind(&name)),
                    ToolStatus::InProgress,
                );
                let result =
                    agent_tools::call(&store, &tokens, token, &scope, &name, Some(&arguments));
                let failed = result.get("isError") == Some(&Value::Bool(true));
                tell(
                    None,
                    None,
                    if failed {
                        ToolStatus::Failed
                    } else {
                        ToolStatus::Completed
                    },
                );
                Value::String(answer_text(&result))
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
        user: user_text(task, true),
    }
}

/// What follows the system prompt: the goal, and where new notes go.
pub fn user_text(task: &OpenTask, api_names: bool) -> String {
    let goal = format!("Task: {}", task.spec.goal.trim());
    let mut lines = Vec::new();
    let book = task.spec.new_note_book.trim();
    if !book.is_empty() {
        let tool = if api_names {
            "create_note"
        } else {
            "create-note"
        };
        lines.push(format!(
            "New notes go in the notebook with bookId \"{book}\" (leave `bookId` out of {tool} to use it). Notes can only be created and moved into the notebooks of the notes you were given and that one."
        ));
    }
    if let Some(note) = task
        .spec
        .start_note
        .as_deref()
        .map(str::trim)
        .filter(|note| !note.is_empty())
    {
        lines.push(format!(
            "Start from the note with id \"{note}\": read it first. This only says where to begin."
        ));
    }
    // The task comes last, as the system prompt says.
    lines.push(goal);
    lines.join("\n\n")
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
    let (tx, rx) = mpsc::channel();
    let inner = provider.run_task(
        &prompt,
        registry_with_events(store, task, Some(tx)),
        task.spec.max_calls,
        on_tool_completed,
        cancel,
    )?;
    Ok(Box::new(WithToolEvents {
        inner,
        rx,
        queue: VecDeque::new(),
    }))
}

/// Counts Kanso's own tool calls, for the ids of their [`Event::Tool`]s.
static TOOL_CALLS: AtomicU64 = AtomicU64::new(0);

/// The model's events with the tool events its tools told, each placed
/// before the event that was waiting when they came.
struct WithToolEvents {
    inner: Box<dyn Iterator<Item = Event> + Send>,
    rx: mpsc::Receiver<Event>,
    queue: VecDeque<Event>,
}

impl Iterator for WithToolEvents {
    type Item = Event;

    fn next(&mut self) -> Option<Event> {
        if self.queue.is_empty() {
            let event = self.inner.next();
            self.queue.extend(self.rx.try_iter());
            self.queue.extend(event);
        }
        self.queue.pop_front()
    }
}

/// Whether a task's last words say the model can't use tools.
pub fn is_no_tools(message: &str) -> bool {
    message == NO_TOOLS
}
