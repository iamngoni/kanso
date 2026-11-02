//! Running agent tasks (issue #196): the runner. No window.
//!
//! A task is started with a goal, the notes the agent may read and what it
//! may propose ([`super::TaskSpec`]). It runs on its own thread until the
//! agent says it is done, the person presses Stop, the time limit (default
//! 20 minutes) passes, or the tool-call limit (default 200) is reached; in
//! every case the changes proposed so far stay for review. The agent never
//! writes: Apply writes the ticked ops in one transaction
//! ([`kanso_server::ChangeSet::apply`]) and one Undo takes them back.
//!
//! Who runs a task is a [`TaskDriver`]. [`ApiDriver`] drives a model with an
//! API key with Kanso's own tool loop and the same handlers in process
//! (desktop and phone); the desktop app adds one for an ACP agent, which
//! reaches the change set through the notes server over MCP. Everything a
//! window shows comes from [`TaskRow`]s.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use kanso_ai::{Event, Prompt, SharedProvider};
use kanso_server::{
    AppliedSet, ApplyFailure, ChangeSet, Selection, TaskId, TokenStore, undo_changes,
};
use kanso_store::{Store, UndoReport};

use super::model::{EndedBy, Phase, TaskRow};
use super::{OpenTask, REPLY_WAIT, StartError, TaskSpec};

/// The most tasks working at once.
pub const MAX_ACTIVE: usize = 2;

/// "Applied 12 changes from Claude Agent." with Undo, for 30 seconds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Notice {
    pub task: TaskId,
    pub text: String,
    pub undoable: bool,
    pub at: Instant,
}

/// How long the notice stays.
pub const NOTICE_FOR: Duration = Duration::from_secs(30);

/// What the window hears.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskEvent {
    /// A row changed.
    Changed,
    /// A task just ended with changes to review.
    Finished(TaskId),
}

/// Why a task didn't start.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CannotStart {
    Sheet(StartError),
    /// Two tasks are working already.
    TooMany,
    /// The AI can't start: a whole sentence.
    Ai(String),
}

impl CannotStart {
    pub fn message(&self) -> String {
        match self {
            CannotStart::Sheet(e) => e.message().to_owned(),
            CannotStart::TooMany => {
                "Two agent tasks are working. Wait for one to finish, or stop it.".to_owned()
            }
            CannotStart::Ai(text) => text.clone(),
        }
    }
}

/// What a task needs of whoever runs it: an API-key model, or an agent.
pub trait TaskDriver: Send + Sync {
    /// The prompt that starts `task`.
    fn first_prompt(&self, task: &OpenTask) -> Prompt;

    /// The prompt that continues `task` after the person answered a
    /// question. `said` is everything the AI said so far.
    fn reply_prompt(&self, task: &OpenTask, said: &str, reply: &str) -> Prompt;

    /// Starts one stretch of work on `prompt`. `progress` is called after
    /// each tool call or note read; setting `cancel` stops the stretch.
    fn begin(
        &self,
        prompt: &Prompt,
        task: &OpenTask,
        store: Arc<Store>,
        progress: Box<dyn FnMut() + Send>,
        cancel: Arc<AtomicBool>,
    ) -> Result<Box<dyn Iterator<Item = Event> + Send>, String>;

    /// Whether tasks on this driver wait their turn for it (an ACP agent
    /// is one process, one task at a time).
    fn queues(&self) -> bool {
        false
    }

    /// Whether the live line is made from the notes read ("Reading Q3
    /// planning") rather than from the AI's own status lines.
    fn names_reads(&self) -> bool {
        false
    }
}

/// Who runs a task.
pub type Backend = Arc<dyn TaskDriver>;

/// A model with an API key, driven by Kanso's tool loop.
pub struct ApiDriver(pub SharedProvider);

impl TaskDriver for ApiDriver {
    fn first_prompt(&self, task: &OpenTask) -> Prompt {
        super::api_prompt(task)
    }

    // A model with an API key has no session: it starts again with what was
    // said, and `list_proposed` shows its own work.
    fn reply_prompt(&self, task: &OpenTask, said: &str, reply: &str) -> Prompt {
        let mut prompt = super::api_prompt(task);
        prompt.user = format!(
            "{}\n\nEarlier in this task you said:\n{}\n\nThe user replied:\n{}\n\nContinue the task. Call list_proposed to see what you have proposed already.",
            prompt.user,
            said.trim(),
            reply.trim()
        );
        prompt
    }

    fn begin(
        &self,
        prompt: &Prompt,
        task: &OpenTask,
        store: Arc<Store>,
        progress: Box<dyn FnMut() + Send>,
        cancel: Arc<AtomicBool>,
    ) -> Result<Box<dyn Iterator<Item = Event> + Send>, String> {
        let provider = lock(&self.0);
        super::run_api_with_prompt(&**provider, store, task, prompt.clone(), progress, cancel)
            .map_err(|e| e.to_string())
    }

    fn names_reads(&self) -> bool {
        true
    }
}

struct Entry {
    row: Mutex<TaskRow>,
    open: OpenTask,
    store: Arc<Store>,
    backend: Backend,
    started: Instant,
    /// Set by Stop, the watchdog and shutdown.
    cancel: Arc<AtomicBool>,
    /// Why the watchdog or Stop cancelled.
    reason: Mutex<Option<EndedBy>>,
    /// When the agent started waiting for a reply.
    asked_at: Mutex<Option<Instant>>,
    /// The last applied batch, for Undo.
    applied: Mutex<Option<AppliedSet>>,
    /// The words of the turn so far.
    said: Mutex<String>,
    /// Over the whole task, for an API-key reply (which restarts the loop).
    transcript: Mutex<String>,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// The running tasks, shared by the panel, the status bar and the quit
/// check. Cloning shares the same tasks.
#[derive(Clone, Default)]
pub struct Tasks {
    entries: Arc<Mutex<Vec<Arc<Entry>>>>,
    notice: Arc<Mutex<Option<Notice>>>,
    subscribers: Arc<Mutex<Vec<async_channel::Sender<TaskEvent>>>>,
}

impl Tasks {
    /// Hears of every change to a row.
    pub fn subscribe(&self) -> async_channel::Receiver<TaskEvent> {
        let (tx, rx) = async_channel::unbounded();
        lock(&self.subscribers).push(tx);
        rx
    }

    fn tell(&self, event: TaskEvent) {
        lock(&self.subscribers).retain(|tx| tx.try_send(event).is_ok());
    }

    /// The rows, oldest first, with their elapsed time brought up to date.
    pub fn rows(&self) -> Vec<TaskRow> {
        lock(&self.entries)
            .iter()
            .map(|entry| {
                let mut row = lock(&entry.row).clone();
                if row.is_active() {
                    row.elapsed = entry.started.elapsed();
                }
                row
            })
            .collect()
    }

    pub fn row(&self, id: TaskId) -> Option<TaskRow> {
        self.rows().into_iter().find(|row| row.id == id)
    }

    fn entry(&self, id: TaskId) -> Option<Arc<Entry>> {
        lock(&self.entries)
            .iter()
            .find(|entry| entry.open.id == id)
            .cloned()
    }

    /// How many tasks are working.
    pub fn active(&self) -> usize {
        self.rows().iter().filter(|row| row.is_active()).count()
    }

    /// A copy of the task's change set so far.
    pub fn changes(&self, id: TaskId) -> Option<ChangeSet> {
        self.entry(id)?.open.changes()
    }

    /// The task's token (it ends with the task).
    pub fn token(&self, id: TaskId) -> kanso_server::TokenId {
        self.entry(id).expect("the task").open.token
    }

    /// Starts a task. `backend` builds whoever runs it from the opened task
    /// (an ACP agent needs the task's token for its session).
    pub fn start(
        &self,
        store: &Arc<Store>,
        tokens: &TokenStore,
        spec: &TaskSpec,
        agent: &str,
        backend: impl FnOnce(&OpenTask) -> Result<Backend, String>,
    ) -> Result<TaskId, CannotStart> {
        if self.active() >= MAX_ACTIVE {
            return Err(CannotStart::TooMany);
        }
        let open = super::open(store, tokens, spec, agent).map_err(CannotStart::Sheet)?;
        let backend = match backend(&open) {
            Ok(backend) => backend,
            Err(text) => {
                open.discard();
                return Err(CannotStart::Ai(text));
            }
        };
        let id = open.id;
        let mut row = TaskRow::new(id, &spec.goal, agent);
        if backend.queues() && self.active() > 0 {
            row.phase = Phase::Waiting;
        }
        let entry = Arc::new(Entry {
            row: Mutex::new(row),
            open,
            store: store.clone(),
            backend,
            started: Instant::now(),
            cancel: Arc::new(AtomicBool::new(false)),
            reason: Mutex::new(None),
            asked_at: Mutex::new(None),
            applied: Mutex::new(None),
            said: Mutex::new(String::new()),
            transcript: Mutex::new(String::new()),
        });
        lock(&self.entries).push(entry.clone());
        self.tell(TaskEvent::Changed);
        self.watch(entry.clone(), spec.time_limit);
        let prompt = entry.backend.first_prompt(&entry.open);
        self.segment(entry, prompt);
        Ok(id)
    }

    /// The watchdog: ends the task at its time limit or tool-call limit, and
    /// a task that waited [`REPLY_WAIT`] for a reply.
    fn watch(&self, entry: Arc<Entry>, limit: Duration) {
        let tasks = self.clone();
        std::thread::Builder::new()
            .name("kanso-agent-task-watch".into())
            .spawn(move || {
                loop {
                    std::thread::sleep(Duration::from_millis(200));
                    let row = lock(&entry.row).clone();
                    if !row.is_active() {
                        return;
                    }
                    if let Phase::Asking { .. } = row.phase {
                        let waited = lock(&entry.asked_at).map(|at| at.elapsed());
                        if waited.is_some_and(|waited| waited >= REPLY_WAIT) {
                            tasks.conclude(&entry, Some(EndedBy::NoReply), None);
                            return;
                        }
                        continue;
                    }
                    let reason = if entry.started.elapsed() >= limit {
                        Some(EndedBy::TimeLimit)
                    } else if entry.open.tokens.calls(entry.open.token) >= entry.open.spec.max_calls
                    {
                        Some(EndedBy::CallLimit)
                    } else {
                        None
                    };
                    if let Some(reason) = reason {
                        lock(&entry.reason).get_or_insert(reason);
                        entry.cancel.store(true, Ordering::Relaxed);
                        // A hung agent is let go of after a moment.
                        std::thread::sleep(Duration::from_secs(5));
                        if lock(&entry.row).is_active() {
                            tasks.conclude(&entry, Some(reason), None);
                        }
                        return;
                    }
                }
            })
            .ok();
    }

    /// Runs one stretch of the task: the goal, or the person's reply.
    fn segment(&self, entry: Arc<Entry>, prompt: Prompt) {
        let tasks = self.clone();
        entry.cancel.store(false, Ordering::Relaxed);
        {
            let mut row = lock(&entry.row);
            if matches!(row.phase, Phase::Asking { .. }) {
                row.phase = Phase::Running;
            }
        }
        lock(&entry.said).clear();
        std::thread::Builder::new()
            .name("kanso-agent-task".into())
            .spawn(move || tasks.run_segment(&entry, &prompt))
            .ok();
    }

    fn refresh(&self, entry: &Entry, line: Option<String>) {
        let notes = entry.open.reads();
        let counts = entry.open.changes().map(|set| set.counts());
        {
            let mut row = lock(&entry.row);
            if !row.is_active() {
                return;
            }
            if row.phase == Phase::Waiting {
                row.phase = Phase::Running;
            }
            row.notes_read = notes.len();
            if let Some(counts) = counts {
                row.counts = counts;
            }
            row.elapsed = entry.started.elapsed();
            if let Some(line) = line {
                row.line = line;
            } else if let Some(last) = notes.last()
                && entry.backend.names_reads()
                && let Ok(Some(note)) = entry.store.note(last)
            {
                row.line = format!("Reading {}", note.title);
            }
        }
        self.tell(TaskEvent::Changed);
    }

    fn run_segment(&self, entry: &Arc<Entry>, prompt: &Prompt) {
        let tasks = self.clone();
        let watching = entry.clone();
        let events = entry.backend.begin(
            prompt,
            &entry.open,
            entry.store.clone(),
            Box::new(move || tasks.refresh(&watching, None)),
            entry.cancel.clone(),
        );
        let events = match events {
            Ok(events) => events,
            Err(message) => return self.conclude(entry, None, Some(message)),
        };
        let mut failure = None;
        for event in events {
            match event {
                Event::Text(text) => {
                    lock(&entry.said).push_str(&text);
                    self.refresh(entry, None);
                }
                Event::Status(line) => self.refresh(entry, Some(line)),
                Event::Done => break,
                Event::Error(message) => {
                    failure = Some(message);
                    break;
                }
            }
            if entry.cancel.load(Ordering::Relaxed) {
                break;
            }
        }
        let said = std::mem::take(&mut *lock(&entry.said));
        lock(&entry.transcript).push_str(&said);
        if !said.trim().is_empty() {
            lock(&entry.row).last_message = said.trim().to_owned();
        }
        if failure.is_none() && self.wants_reply(entry, &said) {
            return;
        }
        self.conclude(entry, None, failure);
    }

    /// Whether the agent ended its turn with a question for the person (and
    /// not with `kanso-task-done`). Then the task waits for a reply.
    fn wants_reply(&self, entry: &Arc<Entry>, said: &str) -> bool {
        let done = entry.open.changes().is_some_and(|set| set.done);
        let stopped = entry.cancel.load(Ordering::Relaxed);
        let question = said.trim();
        if done || stopped || !question.ends_with('?') {
            return false;
        }
        *lock(&entry.asked_at) = Some(Instant::now());
        lock(&entry.row).phase = Phase::Asking {
            question: question.to_owned(),
        };
        self.refresh(entry, None);
        true
    }

    /// Ends the task: the token ends, the change set stays.
    fn conclude(&self, entry: &Arc<Entry>, forced: Option<EndedBy>, failure: Option<String>) {
        let reason = forced.or(*lock(&entry.reason));
        let set = entry.open.changes();
        let counts = set.as_ref().map(|s| s.counts()).unwrap_or_default();
        let summary = set.as_ref().map(|s| s.summary.clone()).unwrap_or_default();
        let finished = {
            let mut row = lock(&entry.row);
            if !row.is_active() {
                return;
            }
            row.counts = counts;
            row.notes_read = entry.open.reads().len();
            row.elapsed = entry.started.elapsed();
            if !summary.trim().is_empty() {
                row.last_message = summary;
            }
            row.ended_by = Some(match reason {
                Some(reason) => reason,
                None if entry.cancel.load(Ordering::Relaxed) => EndedBy::Stopped,
                None => EndedBy::Agent,
            });
            row.phase = match failure {
                Some(message) => Phase::Failed { message },
                None if counts.total() > 0 => Phase::Ready,
                None => Phase::Nothing,
            };
            row.line.clear();
            row.can_review()
        };
        entry.open.end();
        entry.cancel.store(true, Ordering::Relaxed);
        self.tell(TaskEvent::Changed);
        if finished {
            self.tell(TaskEvent::Finished(entry.open.id));
        }
    }

    /// Stops the task. Its changes so far stay for review.
    pub fn stop(&self, id: TaskId) {
        let Some(entry) = self.entry(id) else { return };
        lock(&entry.reason).get_or_insert(EndedBy::Stopped);
        entry.cancel.store(true, Ordering::Relaxed);
        if matches!(
            lock(&entry.row).phase,
            Phase::Asking { .. } | Phase::Waiting
        ) {
            self.conclude(&entry, Some(EndedBy::Stopped), None);
        }
    }

    /// The person's answer to the agent's question: the task goes on.
    pub fn reply(&self, id: TaskId, text: &str) {
        let Some(entry) = self.entry(id) else { return };
        if !matches!(lock(&entry.row).phase, Phase::Asking { .. }) || text.trim().is_empty() {
            return;
        }
        *lock(&entry.asked_at) = None;
        let said = lock(&entry.transcript).clone();
        let prompt = entry.backend.reply_prompt(&entry.open, &said, text);
        self.segment(entry, prompt);
    }

    /// Writes the ticked ops in one transaction. `Undo` is offered after.
    pub fn apply(
        &self,
        id: TaskId,
        selection: &Selection,
        now_ms: i64,
    ) -> Result<AppliedSet, ApplyFailure> {
        let entry = self.entry(id).ok_or(ApplyFailure::NothingToApply)?;
        let set = entry.open.changes().ok_or(ApplyFailure::NothingToApply)?;
        let applied = set.apply(&entry.store, selection, now_ms)?;
        entry.open.tokens.changesets().remove(id);
        *lock(&entry.applied) = Some(applied.clone());
        let changes = selection.ticked.len().saturating_sub(applied.dropped.len());
        lock(&entry.row).phase = Phase::Applied { changes };
        let agent = lock(&entry.row).agent.clone();
        self.say(
            id,
            super::review::applied_message(&agent, &applied, selection.ticked.len()),
            true,
        );
        Ok(applied)
    }

    /// Takes back what Apply wrote, in one transaction.
    pub fn undo(&self, id: TaskId, now_ms: i64) -> Option<Result<UndoReport, String>> {
        let entry = self.entry(id)?;
        let applied = lock(&entry.applied).take()?;
        let report = undo_changes(&entry.store, &applied.batch, now_ms).map_err(|e| e.to_string());
        if let Ok(done) = &report {
            lock(&entry.row).phase = Phase::Undone;
            self.say(id, super::review::undo_message(done.skipped.len()), false);
        }
        Some(report)
    }

    /// The notice to show now, if it hasn't run out.
    pub fn notice(&self) -> Option<Notice> {
        let mut notice = lock(&self.notice);
        if notice
            .as_ref()
            .is_some_and(|n| n.at.elapsed() >= NOTICE_FOR)
        {
            *notice = None;
        }
        notice.clone()
    }

    fn say(&self, task: TaskId, text: String, undoable: bool) {
        *lock(&self.notice) = Some(Notice {
            task,
            text,
            undoable,
            at: Instant::now(),
        });
        self.tell(TaskEvent::Changed);
    }

    /// Whether task `id` has something Undo would take back.
    pub fn can_undo(&self, id: TaskId) -> bool {
        self.entry(id)
            .is_some_and(|entry| lock(&entry.applied).is_some())
    }

    /// Throws the task's change set away.
    pub fn discard(&self, id: TaskId) {
        let Some(entry) = self.entry(id) else { return };
        entry.cancel.store(true, Ordering::Relaxed);
        entry.open.discard();
        let mut row = lock(&entry.row);
        if row.is_active() || row.can_discard() {
            row.phase = Phase::Discarded;
        }
        drop(row);
        self.tell(TaskEvent::Changed);
    }

    /// Takes a finished row off the panel.
    pub fn dismiss(&self, id: TaskId) {
        let mut entries = lock(&self.entries);
        entries.retain(|entry| {
            let keep = entry.open.id != id || lock(&entry.row).is_active();
            if !keep {
                entry.open.discard();
            }
            keep
        });
        drop(entries);
        self.tell(TaskEvent::Changed);
    }

    /// Stops every task and revokes every token: AI was turned off or Kanso is
    /// quitting. Change sets are lost with the app.
    pub fn shutdown(&self) {
        for entry in lock(&self.entries).iter() {
            entry.cancel.store(true, Ordering::Relaxed);
            entry.open.end();
        }
    }

    /// The prompt for quitting with unreviewed changes, if any.
    pub fn quit_prompt(&self) -> Option<String> {
        super::model::quit_prompt(&self.rows())
    }
}
