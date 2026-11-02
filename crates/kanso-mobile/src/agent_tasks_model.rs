//! The plain data of agent tasks on the phone: a task's row and the cards
//! to review. No logic beyond building a row from the engine's.

use kanso_engine::agent_task::model::{Phase, TaskRow};

/// Settings > AI > Agent tasks on the phone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AgentTaskPrefs {
    /// Agent tasks are on (when AI is on).
    pub enabled: bool,
    /// "Agents may propose".
    pub new_notes: bool,
    pub edits: bool,
    pub moves: bool,
    pub merges: bool,
    /// The most minutes one task may run: 5, 20 or 60.
    pub time_limit_minutes: u32,
    /// The most notes one task may read: 50, 200 or 1000.
    pub max_notes: u32,
    /// Ask before starting a task that can read every notebook.
    pub ask_before_reading_all: bool,
}

/// One task, as the Ask page and the task list show it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskView {
    pub id: u64,
    pub goal: String,
    pub agent: String,
    /// `waiting`, `running`, `asking`, `ready`, `nothing`, `failed`,
    /// `applied`, `undone` or `discarded`.
    pub phase: String,
    /// The question, the error or the number of changes, by phase.
    pub detail: String,
    /// "Reading Q3 planning".
    pub line: String,
    pub last_message: String,
    pub notes_read: u32,
    pub changes: u32,
    pub elapsed_secs: u32,
}

impl TaskView {
    pub(crate) fn from_row(row: &TaskRow) -> TaskView {
        let (phase, detail) = match &row.phase {
            Phase::Waiting => ("waiting", String::new()),
            Phase::Running => ("running", String::new()),
            Phase::Asking { question } => ("asking", question.clone()),
            Phase::Ready => ("ready", String::new()),
            Phase::Nothing => ("nothing", String::new()),
            Phase::Failed { message } => ("failed", message.clone()),
            Phase::Applied { changes } => ("applied", changes.to_string()),
            Phase::Undone => ("undone", String::new()),
            Phase::Discarded => ("discarded", String::new()),
        };
        TaskView {
            id: row.id.0,
            goal: row.goal.clone(),
            agent: row.agent.clone(),
            phase: phase.to_owned(),
            detail,
            line: row.line.clone(),
            last_message: row.last_message.clone(),
            notes_read: row.notes_read as u32,
            changes: row.counts.total() as u32,
            elapsed_secs: row.elapsed.as_secs() as u32,
        }
    }
}

/// One change to approve.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskCard {
    pub index: u32,
    /// `create`, `edit`, `move` or `merge`.
    pub kind: String,
    pub title: String,
    pub book: String,
    pub ticked: bool,
    /// The note changed since the agent read it.
    pub changed_since: bool,
    /// Not one of the notes the person chose.
    pub outside: bool,
    pub published: bool,
    /// `+ added`, `- removed` or plain lines.
    pub lines: Vec<String>,
}

/// The review sheet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskReview {
    pub header: String,
    pub from_agent: String,
    pub warnings: Vec<String>,
    pub cards: Vec<TaskCard>,
}
