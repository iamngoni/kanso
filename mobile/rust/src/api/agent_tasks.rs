//! Agent tasks: the AI proposes changes to notes and the person approves
//! them. Polled, not streamed: Dart asks for the task list while the Ask
//! page or the task list is open.

use flutter_rust_bridge::frb;
use kanso_mobile::{
    AgentTaskPrefs as CoreAgentTaskPrefs, AgentTasks, KansoError,
    SessionTranscript as CoreSessionTranscript, SessionView as CoreSessionView,
    TaskCard as CoreTaskCard, TaskReview as CoreTaskReview, TaskView as CoreTaskView,
    TranscriptItemView as CoreTranscriptItemView,
};

/// One session in the list: a task as a row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionView {
    pub id: u64,
    pub key: String,
    pub title: String,
    pub agent: String,
    pub phase: String,
    pub detail: String,
    pub dot: String,
    pub unread: bool,
    pub last_activity_ms: i64,
    pub notes_read: u32,
    pub changes: u32,
    pub can_stop: bool,
    pub can_review: bool,
    pub can_follow_up: bool,
}

impl From<CoreSessionView> for SessionView {
    fn from(s: CoreSessionView) -> Self {
        SessionView {
            id: s.id,
            key: s.key,
            title: s.title,
            agent: s.agent,
            phase: s.phase,
            detail: s.detail,
            dot: s.dot,
            unread: s.unread,
            last_activity_ms: s.last_activity_ms,
            notes_read: s.notes_read,
            changes: s.changes,
            can_stop: s.can_stop,
            can_review: s.can_review,
            can_follow_up: s.can_follow_up,
        }
    }
}

/// One thing said in a session; `kind` says which fields mean something.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TranscriptItem {
    pub kind: String,
    pub text: String,
    pub title: String,
    pub tool_kind: String,
    pub status: String,
    pub answer: String,
    pub round: u32,
    pub counts: String,
    pub state: String,
    pub at_ms: i64,
    pub thought_secs: u32,
}

impl From<CoreTranscriptItemView> for TranscriptItem {
    fn from(i: CoreTranscriptItemView) -> Self {
        TranscriptItem {
            kind: i.kind,
            text: i.text,
            title: i.title,
            tool_kind: i.tool_kind,
            status: i.status,
            answer: i.answer,
            round: i.round,
            counts: i.counts,
            state: i.state,
            at_ms: i.at_ms,
            thought_secs: i.thought_secs,
        }
    }
}

/// A session and everything said in it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionTranscript {
    pub session: SessionView,
    pub version: u64,
    pub items: Vec<TranscriptItem>,
}

impl From<CoreSessionTranscript> for SessionTranscript {
    fn from(t: CoreSessionTranscript) -> Self {
        SessionTranscript {
            session: t.session.into(),
            version: t.version,
            items: t.items.into_iter().map(TranscriptItem::from).collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentTaskPrefs {
    pub enabled: bool,
    pub new_notes: bool,
    pub edits: bool,
    pub moves: bool,
    pub merges: bool,
    pub time_limit_minutes: u32,
    pub max_notes: u32,
    pub ask_before_reading_all: bool,
}

impl From<CoreAgentTaskPrefs> for AgentTaskPrefs {
    fn from(p: CoreAgentTaskPrefs) -> Self {
        AgentTaskPrefs {
            enabled: p.enabled,
            new_notes: p.new_notes,
            edits: p.edits,
            moves: p.moves,
            merges: p.merges,
            time_limit_minutes: p.time_limit_minutes,
            max_notes: p.max_notes,
            ask_before_reading_all: p.ask_before_reading_all,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskView {
    pub id: u64,
    pub goal: String,
    pub agent: String,
    pub phase: String,
    pub detail: String,
    pub line: String,
    pub last_message: String,
    pub notes_read: u32,
    pub changes: u32,
    pub elapsed_secs: u32,
}

impl From<CoreTaskView> for TaskView {
    fn from(v: CoreTaskView) -> Self {
        TaskView {
            id: v.id,
            goal: v.goal,
            agent: v.agent,
            phase: v.phase,
            detail: v.detail,
            line: v.line,
            last_message: v.last_message,
            notes_read: v.notes_read,
            changes: v.changes,
            elapsed_secs: v.elapsed_secs,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskCard {
    pub index: u32,
    pub kind: String,
    pub title: String,
    pub book: String,
    pub ticked: bool,
    pub changed_since: bool,
    pub outside: bool,
    pub published: bool,
    pub lines: Vec<String>,
}

impl From<CoreTaskCard> for TaskCard {
    fn from(c: CoreTaskCard) -> Self {
        TaskCard {
            index: c.index,
            kind: c.kind,
            title: c.title,
            book: c.book,
            ticked: c.ticked,
            changed_since: c.changed_since,
            outside: c.outside,
            published: c.published,
            lines: c.lines,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskReview {
    pub header: String,
    pub from_agent: String,
    pub warnings: Vec<String>,
    pub cards: Vec<TaskCard>,
}

impl From<CoreTaskReview> for TaskReview {
    fn from(r: CoreTaskReview) -> Self {
        TaskReview {
            header: r.header,
            from_agent: r.from_agent,
            warnings: r.warnings,
            cards: r.cards.into_iter().map(TaskCard::from).collect(),
        }
    }
}

/// Unix milliseconds now.
fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| {
            i64::try_from(since.as_millis()).unwrap_or(i64::MAX)
        })
}

#[frb(opaque)]
pub struct KansoAgentTasks(pub(super) AgentTasks);

impl KansoAgentTasks {
    /// Starts a task; an empty `note_ids` lets it read every unlocked note,
    /// which needs `confirmed_all` when Settings ask first.
    pub fn start(
        &self,
        goal: String,
        note_ids: Vec<String>,
        new_note_book: String,
        confirmed_all: bool,
    ) -> Result<u64, KansoError> {
        self.0.start(&goal, note_ids, &new_note_book, confirmed_all)
    }

    pub fn prefs(&self) -> AgentTaskPrefs {
        self.0.prefs().into()
    }

    pub fn set_prefs(&self, prefs: AgentTaskPrefs) -> Result<AgentTaskPrefs, KansoError> {
        self.0
            .set_prefs(CoreAgentTaskPrefs {
                enabled: prefs.enabled,
                new_notes: prefs.new_notes,
                edits: prefs.edits,
                moves: prefs.moves,
                merges: prefs.merges,
                time_limit_minutes: prefs.time_limit_minutes,
                max_notes: prefs.max_notes,
                ask_before_reading_all: prefs.ask_before_reading_all,
            })
            .map(AgentTaskPrefs::from)
    }

    pub fn tasks(&self) -> Vec<TaskView> {
        self.0.tasks().into_iter().map(TaskView::from).collect()
    }

    /// The sessions, latest activity first.
    pub fn agent_sessions(&self) -> Vec<SessionView> {
        self.0
            .sessions(now_ms())
            .into_iter()
            .map(SessionView::from)
            .collect()
    }

    /// What was said in session `key`; opening it clears its unread dot.
    /// `None` when the session is no longer listed.
    pub fn agent_transcript(&self, key: String) -> Option<SessionTranscript> {
        self.0
            .transcript(&key, now_ms())
            .map(SessionTranscript::from)
    }

    /// Sends a message into session `key`.
    pub fn agent_follow_up(&self, key: String, text: String) -> Result<(), KansoError> {
        self.0.follow_up(&key, &text)
    }

    /// Ends session `key`; what it did stays.
    pub fn agent_end_session(&self, key: String) -> Result<(), KansoError> {
        self.0.end_session(&key)
    }

    pub fn stop(&self, id: u64) {
        self.0.stop(id)
    }

    pub fn reply(&self, id: u64, text: String) {
        self.0.reply(id, &text)
    }

    pub fn discard(&self, id: u64) {
        self.0.discard(id)
    }

    pub fn review(&self, id: u64) -> Result<TaskReview, KansoError> {
        self.0.review(id).map(TaskReview::from)
    }

    /// Applies the ticked cards in one step; returns the sentence to show.
    pub fn apply(
        &self,
        id: u64,
        ticked: Vec<u32>,
        keep: Vec<u32>,
        now_ms: i64,
    ) -> Result<String, KansoError> {
        self.0.apply(id, ticked, keep, now_ms)
    }

    pub fn undo(&self, id: u64, now_ms: i64) -> Result<String, KansoError> {
        self.0.undo(id, now_ms)
    }

    pub fn can_undo(&self, id: u64) -> bool {
        self.0.can_undo(id)
    }
}
