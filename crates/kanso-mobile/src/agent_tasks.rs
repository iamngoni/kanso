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
use kanso_engine::agent_task::{ReadScope, TaskSpec};
use kanso_server::{OpKind, Selection, TaskId, TokenStore};

use crate::agent_tasks_model::{TaskCard, TaskReview, TaskView};
use crate::ai::AiHandle;
use crate::errors::KansoError;

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

    /// Starts a task that may read `note_ids` (or every unlocked note when
    /// empty) and put new notes in `new_note_book`. Returns its id.
    pub fn start(
        &self,
        goal: &str,
        note_ids: Vec<String>,
        new_note_book: &str,
    ) -> Result<u64, KansoError> {
        let settings = self.ai.enabled()?;
        let provider = self.ai.provider_for(&settings)?;
        let read = if note_ids.is_empty() {
            ReadScope::Everything
        } else {
            ReadScope::Allowed(note_ids)
        };
        let spec = TaskSpec::new(goal, read, new_note_book);
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
    pub fn apply(&self, id: u64, ticked: Vec<u32>, now_ms: i64) -> Result<String, KansoError> {
        let runtime = self.runtime();
        let selection = Selection {
            ticked: ticked.iter().map(|i| *i as usize).collect(),
            apply_anyway: Default::default(),
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
            Some(Ok(_)) => Ok(review::undo_message(0)),
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
