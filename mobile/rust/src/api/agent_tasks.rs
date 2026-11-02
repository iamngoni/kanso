//! Agent tasks: the AI proposes changes to notes and the person approves
//! them. Polled, not streamed: Dart asks for the task list while the Ask
//! page or the task list is open.

use flutter_rust_bridge::frb;
use kanso_mobile::{
    AgentTasks, KansoError, TaskCard as CoreTaskCard, TaskReview as CoreTaskReview,
    TaskView as CoreTaskView,
};

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

#[frb(opaque)]
pub struct KansoAgentTasks(pub(super) AgentTasks);

impl KansoAgentTasks {
    /// Starts a task; an empty `note_ids` lets it read every unlocked note.
    pub fn start(
        &self,
        goal: String,
        note_ids: Vec<String>,
        new_note_book: String,
    ) -> Result<u64, KansoError> {
        self.0.start(&goal, note_ids, &new_note_book)
    }

    pub fn tasks(&self) -> Vec<TaskView> {
        self.0.tasks().into_iter().map(TaskView::from).collect()
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
    pub fn apply(&self, id: u64, ticked: Vec<u32>, now_ms: i64) -> Result<String, KansoError> {
        self.0.apply(id, ticked, now_ms)
    }

    pub fn undo(&self, id: u64, now_ms: i64) -> Result<String, KansoError> {
        self.0.undo(id, now_ms)
    }

    pub fn can_undo(&self, id: u64) -> bool {
        self.0.can_undo(id)
    }
}
