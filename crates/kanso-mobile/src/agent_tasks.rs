//! Agent tasks on the phone (issue #196): a goal, the notes the agent may
//! read, and a change set the person reviews and applies in one step.
//!
//! The phone runs a task with the API key saved for Ask (there is no ACP
//! agent on a phone). The runner, the token scope that keeps locked and
//! `#ai-exclude` notes out, the review cards, apply and undo are the
//! desktop's own (`kanso_engine::agent_task`); this file is the handle the
//! bridge wraps, with plain data on the way out.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use kanso_engine::agent_task::review::{self, CardDetail, CardState, LineKind};
use kanso_engine::agent_task::runner::{ApiDriver, Tasks};
use kanso_engine::agent_task::settings::AgentTaskSettings;
use kanso_engine::agent_task::{ReadScope, TaskSpec};
use kanso_server::{OpKind, Selection, TaskId, TokenStore};

use crate::agent_tasks_model::{AgentTaskPrefs, TaskCard, TaskReview, TaskView};
use crate::ai::AiHandle;
use crate::errors::KansoError;

/// What starting a task says while Settings has agent tasks off.
pub const TASKS_OFF: &str = "Agent tasks are turned off in Settings.";

/// What starting a task says when it can read every notebook and Settings
/// ask first, and the person hasn't confirmed.
pub const CONFIRM_ALL: &str = "Confirm that this task may read all your notebooks.";

/// The file the preferences are kept in, beside the library.
const PREFS_FILE: &str = "agent-tasks.json";

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct PrefsFile {
    enabled: bool,
    new_notes: bool,
    edits: bool,
    moves: bool,
    merges: bool,
    time_limit_minutes: u32,
    max_notes: u32,
    ask_before_reading_all: bool,
}

impl Default for PrefsFile {
    fn default() -> Self {
        PrefsFile {
            enabled: true,
            new_notes: true,
            edits: true,
            moves: true,
            merges: true,
            time_limit_minutes: 20,
            max_notes: 200,
            ask_before_reading_all: true,
        }
    }
}

/// The agent's name on the phone.
const AGENT: &str = "Kanso AI";

struct Runtime {
    tasks: Tasks,
    tokens: TokenStore,
}

fn runtimes() -> &'static Mutex<HashMap<std::path::PathBuf, Arc<Runtime>>> {
    static R: OnceLock<Mutex<HashMap<std::path::PathBuf, Arc<Runtime>>>> = OnceLock::new();
    R.get_or_init(Default::default)
}

/// The agent tasks of one library.
#[derive(Clone)]
pub struct AgentTasks {
    ai: AiHandle,
}

impl AiHandle {
    /// Agent tasks: the AI proposes changes, the person approves them.
    pub fn agent_tasks(&self) -> AgentTasks {
        AgentTasks { ai: self.clone() }
    }
}

impl AgentTasks {
    fn runtime(&self) -> Arc<Runtime> {
        let path = self.ai.inner.path.clone();
        let mut all = runtimes().lock().unwrap_or_else(|e| e.into_inner());
        all.entry(path)
            .or_insert_with(|| {
                Arc::new(Runtime {
                    tasks: Tasks::default(),
                    tokens: TokenStore::new(),
                })
            })
            .clone()
    }

    fn prefs_path(&self) -> std::path::PathBuf {
        self.ai
            .inner
            .path
            .parent()
            .filter(|dir| !dir.as_os_str().is_empty())
            .unwrap_or_else(|| std::path::Path::new("."))
            .join(PREFS_FILE)
    }

    /// Settings > AI > Agent tasks.
    pub fn prefs(&self) -> AgentTaskPrefs {
        let file: PrefsFile = std::fs::read(self.prefs_path())
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default();
        AgentTaskPrefs {
            enabled: file.enabled,
            new_notes: file.new_notes,
            edits: file.edits,
            moves: file.moves,
            merges: file.merges,
            time_limit_minutes: file.time_limit_minutes,
            max_notes: file.max_notes,
            ask_before_reading_all: file.ask_before_reading_all,
        }
    }

    /// Saves the preferences.
    pub fn set_prefs(&self, prefs: AgentTaskPrefs) -> Result<AgentTaskPrefs, KansoError> {
        let file = PrefsFile {
            enabled: prefs.enabled,
            new_notes: prefs.new_notes,
            edits: prefs.edits,
            moves: prefs.moves,
            merges: prefs.merges,
            time_limit_minutes: prefs.time_limit_minutes,
            max_notes: prefs.max_notes,
            ask_before_reading_all: prefs.ask_before_reading_all,
        };
        let bytes =
            serde_json::to_vec_pretty(&file).map_err(|err| KansoError::storage(err.to_string()))?;
        let path = self.prefs_path();
        let part = path.with_extension("json.part");
        std::fs::write(&part, bytes)
            .and_then(|()| std::fs::rename(&part, &path))
            .map_err(|err| KansoError::storage(err.to_string()))?;
        Ok(prefs)
    }

    /// Starts a task that may read `note_ids` (or every unlocked note when
    /// empty, which the person must have confirmed when Settings ask first)
    /// and put new notes in `new_note_book`. Returns its id.
    pub fn start(
        &self,
        goal: &str,
        note_ids: Vec<String>,
        new_note_book: &str,
        confirmed_all: bool,
    ) -> Result<u64, KansoError> {
        let prefs = self.prefs();
        if !prefs.enabled {
            return Err(KansoError::invalid(TASKS_OFF));
        }
        let read = if note_ids.is_empty() {
            if prefs.ask_before_reading_all && !confirmed_all {
                return Err(KansoError::invalid(CONFIRM_ALL));
            }
            ReadScope::Everything
        } else {
            ReadScope::Notes(note_ids)
        };
        let settings = self.ai.enabled()?;
        let provider = self.ai.provider_for(&settings)?;
        // New notes go where the person said, and the Inbox when they
        // didn't.
        let new_note_book = match new_note_book.trim() {
            "" => self
                .ai
                .inner
                .store
                .role_book(kanso_core::roles::Role::Inbox, kanso_store::Origin::Ui)
                .map_err(|err| KansoError::storage(err.to_string()))?,
            chosen => chosen.to_owned(),
        };
        let spec = TaskSpec::with_settings(
            goal,
            read,
            &new_note_book,
            &AgentTaskSettings {
                new_notes: prefs.new_notes,
                edits: prefs.edits,
                moves: prefs.moves,
                merges: prefs.merges,
                time_limit_minutes: u64::from(prefs.time_limit_minutes),
                max_notes: prefs.max_notes as usize,
                ask_before_reading_all: prefs.ask_before_reading_all,
            },
        );
        let runtime = self.runtime();
        runtime
            .tasks
            .start(
                &self.ai.inner.store,
                &runtime.tokens,
                &spec,
                AGENT,
                move |_| Ok(Arc::new(ApiDriver(provider)) as _),
            )
            .map(|id| id.0)
            .map_err(|err| KansoError::invalid(err.message()))
    }

    /// Every task this session, newest last.
    pub fn tasks(&self) -> Vec<TaskView> {
        self.runtime()
            .tasks
            .rows()
            .iter()
            .map(TaskView::from_row)
            .collect()
    }

    /// Stops a task; what it proposed stays for review.
    pub fn stop(&self, id: u64) {
        self.runtime().tasks.stop(TaskId(id));
    }

    /// Answers a question the agent asked.
    pub fn reply(&self, id: u64, text: &str) {
        self.runtime().tasks.reply(TaskId(id), text);
    }

    /// Forgets a task and what it proposed.
    pub fn discard(&self, id: u64) {
        self.runtime().tasks.discard(TaskId(id));
    }

    /// The cards to review, none ticked that the person would not expect.
    pub fn review(&self, id: u64) -> Result<TaskReview, KansoError> {
        let runtime = self.runtime();
        let set = runtime
            .tasks
            .changes(TaskId(id))
            .ok_or_else(|| KansoError::invalid("There is nothing to review."))?;
        let from = runtime
            .tasks
            .row(TaskId(id))
            .map(|row| row.last_message)
            .unwrap_or_default();
        let sheet = review::build(&self.ai.inner.store, &set, &from);
        let default = sheet.default_selection();
        Ok(TaskReview {
            header: sheet.header(),
            from_agent: sheet.from_agent.clone(),
            warnings: sheet.warnings.clone(),
            cards: sheet
                .cards
                .iter()
                .map(|card| TaskCard {
                    index: card.index as u32,
                    kind: kind_word(card.kind).to_owned(),
                    title: card.title.clone(),
                    book: card.book.clone(),
                    ticked: default.ticked.contains(&card.index),
                    changed_since: matches!(card.state, CardState::Changed { .. }),
                    outside: card.outside,
                    published: card.published,
                    lines: lines_of(&card.detail),
                })
                .collect(),
        })
    }

    /// Writes the ticked cards in one transaction. Returns the sentence to
    /// show ("Applied 12 changes from Kanso AI.").
    pub fn apply(
        &self,
        id: u64,
        ticked: Vec<u32>,
        keep: Vec<u32>,
        now_ms: i64,
    ) -> Result<String, KansoError> {
        let runtime = self.runtime();
        let ticked: std::collections::BTreeSet<usize> =
            ticked.iter().map(|i| *i as usize).collect();
        // Ticking a card that changed since the agent read it is choosing to
        // apply over the change, as on the desktop.
        let apply_anyway = match runtime.tasks.changes(TaskId(id)) {
            Some(set) => review::build(&self.ai.inner.store, &set, "")
                .cards
                .iter()
                .filter(|card| {
                    matches!(card.state, CardState::Changed { .. }) && ticked.contains(&card.index)
                })
                .map(|card| card.index)
                .collect(),
            None => Default::default(),
        };
        let selection = Selection {
            ticked,
            apply_anyway,
            keep_sources: keep.iter().map(|i| *i as usize).collect(),
        };
        runtime
            .tasks
            .apply(TaskId(id), &selection, now_ms)
            .map_err(|err| KansoError::invalid(err.message()))?;
        Ok(runtime
            .tasks
            .notice()
            .map(|notice| notice.text)
            .unwrap_or_default())
    }

    /// Takes back the last apply of task `id`, in one transaction.
    pub fn undo(&self, id: u64, now_ms: i64) -> Result<String, KansoError> {
        match self.runtime().tasks.undo(TaskId(id), now_ms) {
            Some(Ok(report)) => Ok(review::undo_message(report.skipped.len())),
            Some(Err(message)) => Err(KansoError::storage(message)),
            None => Err(KansoError::invalid("There is nothing to undo.")),
        }
    }

    /// Whether the person can still undo task `id`.
    pub fn can_undo(&self, id: u64) -> bool {
        self.runtime().tasks.can_undo(TaskId(id))
    }

    /// Stops every task: the app is going away.
    pub fn shutdown(&self) {
        self.runtime().tasks.shutdown();
    }
}

fn kind_word(kind: OpKind) -> &'static str {
    match kind {
        OpKind::Create => "create",
        OpKind::Edit => "edit",
        OpKind::Move => "move",
        OpKind::Merge => "merge",
    }
}

/// A card's body as lines: `+ added`, `- removed`, or plain text.
fn lines_of(detail: &CardDetail) -> Vec<String> {
    let diff = |lines: &[(LineKind, String)]| -> Vec<String> {
        lines
            .iter()
            .map(|(kind, text)| match kind {
                LineKind::Added => format!("+ {text}"),
                LineKind::Removed => format!("- {text}"),
            })
            .collect()
    };
    match detail {
        CardDetail::New { body, .. } => body.lines().map(str::to_owned).collect(),
        CardDetail::Edit { lines, .. } => diff(lines),
        CardDetail::Move { from, to } => vec![format!("From {from}"), format!("To {to}")],
        CardDetail::Merge { sources, lines, .. } => {
            let mut all = vec![format!("Combines {}", sources.join(", "))];
            all.extend(diff(lines));
            all
        }
    }
}
