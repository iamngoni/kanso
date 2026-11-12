//! The plain data of agent tasks on the phone: a task's row and the cards
//! to review. No logic beyond building a row from the engine's.

use kanso_engine::agent_task::model::{Dot, Phase, TaskRow, when_label};
use kanso_engine::agent_task::session::{
    Item, PermissionAnswer, RoundState, ToolKind, ToolStatus, Transcript,
};

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

/// One session in the list (issue #375): a task as a row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionView {
    /// The task's id for this launch (Review, Stop and Apply take it).
    pub id: u64,
    /// The session's own id, which a restart keeps.
    pub key: String,
    /// The goal cut to one line, or the name the person gave.
    pub title: String,
    pub agent: String,
    /// `waiting`, `running`, `asking`, `ready`, `nothing`, `failed`,
    /// `applied`, `undone` or `discarded`.
    pub phase: String,
    /// The question, the error or the number of changes, by phase.
    pub detail: String,
    /// "4 changes to review".
    pub status: String,
    /// "now", "2m ago", "yesterday".
    pub when: String,
    /// `working`, `needs_you`, `idle` or `failed`.
    pub dot: String,
    /// New items since the session was last opened.
    pub unread: bool,
    pub last_activity_ms: i64,
    pub notes_read: u32,
    pub changes: u32,
    pub can_stop: bool,
    pub can_review: bool,
    /// Whether the composer can send now.
    pub can_follow_up: bool,
}

impl SessionView {
    pub(crate) fn from_row(row: &TaskRow, now_ms: i64) -> SessionView {
        let task = TaskView::from_row(row);
        SessionView {
            id: task.id,
            key: row.key.0.clone(),
            title: row.title.clone(),
            agent: task.agent,
            phase: task.phase,
            detail: task.detail,
            status: row.list_status(),
            when: when_label(now_ms, row.last_activity_ms),
            dot: match row.dot() {
                Dot::Working => "working",
                Dot::NeedsYou => "needs_you",
                Dot::Idle => "idle",
                Dot::Failed => "failed",
            }
            .to_owned(),
            unread: row.unread,
            last_activity_ms: row.last_activity_ms,
            notes_read: task.notes_read,
            changes: task.changes,
            can_stop: row.can_stop(),
            can_review: row.can_review(),
            // Only a question can be answered for now.
            // TODO(#375 step 4): also when the session is idle, once
            // `Tasks::follow_up` and `Phase::Idle` are in master.
            can_follow_up: matches!(row.phase, Phase::Asking { .. }),
        }
    }
}

/// One thing in a session's transcript, flat so the bridge carries it as
/// plain data. Which fields mean something depends on `kind`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TranscriptItemView {
    /// `user`, `agent`, `thought`, `tool`, `permission`, `proposed` or
    /// `notice`.
    pub kind: String,
    /// The words of a user, agent, thought or notice item.
    pub text: String,
    /// A tool's or permission's title.
    pub title: String,
    /// A tool's kind: `read`, `edit`, `delete`, `move`, `search`, `execute`,
    /// `think`, `fetch`, `switch_mode` or `other`.
    pub tool_kind: String,
    /// A tool's state: `pending`, `in_progress`, `completed` or `failed`.
    pub status: String,
    /// A permission's answer: `allowed_once`, `allowed_for_session`,
    /// `allowed_always` or `rejected`.
    pub answer: String,
    /// A proposal's round.
    pub round: u32,
    /// A proposal's counts: "3 new notes, 1 edit".
    pub counts: String,
    /// A proposal's state: `to_review`, `applied`, `undone`, `discarded` or
    /// `lost`.
    pub state: String,
    pub at_ms: i64,
    /// Seconds spent thinking.
    pub thought_secs: u32,
}

impl TranscriptItemView {
    fn of(item: &Item) -> TranscriptItemView {
        let base = TranscriptItemView::default();
        match item {
            Item::User { text, at_ms } => TranscriptItemView {
                kind: "user".into(),
                text: text.clone(),
                at_ms: *at_ms,
                ..base
            },
            Item::Agent { text, at_ms } => TranscriptItemView {
                kind: "agent".into(),
                text: text.clone(),
                at_ms: *at_ms,
                ..base
            },
            Item::Thought { text, ms } => TranscriptItemView {
                kind: "thought".into(),
                text: text.clone(),
                thought_secs: u32::try_from(ms / 1000).unwrap_or(u32::MAX),
                ..base
            },
            Item::Tool {
                title,
                kind,
                status,
                ..
            } => TranscriptItemView {
                kind: "tool".into(),
                title: title.clone(),
                tool_kind: tool_kind_word(*kind).into(),
                status: tool_status_word(*status).into(),
                ..base
            },
            Item::Permission {
                title,
                kind,
                answer,
            } => TranscriptItemView {
                kind: "permission".into(),
                title: title.clone(),
                tool_kind: tool_kind_word(*kind).into(),
                answer: match answer {
                    PermissionAnswer::AllowedOnce => "allowed_once",
                    PermissionAnswer::AllowedForSession => "allowed_for_session",
                    PermissionAnswer::AllowedAlways => "allowed_always",
                    PermissionAnswer::Rejected => "rejected",
                }
                .into(),
                ..base
            },
            Item::Proposed {
                round,
                counts,
                state,
            } => TranscriptItemView {
                kind: "proposed".into(),
                round: *round,
                counts: counts.describe(),
                state: match state {
                    RoundState::ToReview => "to_review",
                    RoundState::Applied => "applied",
                    RoundState::Undone => "undone",
                    RoundState::Discarded => "discarded",
                    RoundState::Lost => "lost",
                }
                .into(),
                ..base
            },
            Item::Notice { text, at_ms } => TranscriptItemView {
                kind: "notice".into(),
                text: text.clone(),
                at_ms: *at_ms,
                ..base
            },
        }
    }
}

fn tool_kind_word(kind: ToolKind) -> &'static str {
    match kind {
        ToolKind::Read => "read",
        ToolKind::Edit => "edit",
        ToolKind::Delete => "delete",
        ToolKind::Move => "move",
        ToolKind::Search => "search",
        ToolKind::Execute => "execute",
        ToolKind::Think => "think",
        ToolKind::Fetch => "fetch",
        ToolKind::SwitchMode => "switch_mode",
        ToolKind::Other => "other",
    }
}

fn tool_status_word(status: ToolStatus) -> &'static str {
    match status {
        ToolStatus::Pending => "pending",
        ToolStatus::InProgress => "in_progress",
        ToolStatus::Completed => "completed",
        ToolStatus::Failed => "failed",
    }
}

/// A session and everything said in it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionTranscript {
    pub session: SessionView,
    /// Moves with every change, so the screen can tell whether to redraw.
    pub version: u64,
    pub items: Vec<TranscriptItemView>,
}

impl SessionTranscript {
    pub(crate) fn from_parts(
        row: &TaskRow,
        transcript: &Transcript,
        now_ms: i64,
    ) -> SessionTranscript {
        SessionTranscript {
            session: SessionView::from_row(row, now_ms),
            version: transcript.version,
            items: transcript
                .items
                .iter()
                .map(TranscriptItemView::of)
                .collect(),
        }
    }
}
