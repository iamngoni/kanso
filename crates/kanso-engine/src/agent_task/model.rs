//! What the Agent Tasks panel and the status bar say about a task. A model
//! file: values and the words for them, nothing that runs or draws.

use std::time::Duration;

use kanso_server::{Counts, TaskId, TaskOps};

use super::session::SessionKey;
use super::{ReadScope, TaskSpec};

/// Where a task is.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Phase {
    /// Another task is using the agent.
    Waiting,
    Running,
    /// The turn ended and the session is warm: a message goes to the same
    /// agent conversation.
    Idle,
    /// The agent asked a question and waits for the person (in the
    /// composer: it is the latest agent message).
    Asking {
        question: String,
    },
    /// Ended with changes to review.
    Ready,
    /// Ended without proposing anything.
    Nothing,
    /// The AI stopped with an error; the change set so far is kept.
    Failed {
        message: String,
    },
    /// Applied; `Undo` is offered for the session.
    Applied {
        changes: usize,
    },
    Undone,
    Discarded,
}

/// Why a running task ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EndedBy {
    /// The agent said it was done.
    Agent,
    Stopped,
    TimeLimit,
    CallLimit,
    /// Nobody wrote for 30 minutes, so the session was closed.
    IdleClose,
    /// The person ended the session.
    Ended,
}

/// One row of the Agent Tasks panel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskRow {
    pub id: TaskId,
    /// The session's own id, which a restart keeps.
    pub key: SessionKey,
    pub goal: String,
    /// The goal cut to one line: what the list calls the task.
    pub title: String,
    /// What the person calls the AI: "Claude Agent".
    pub agent: String,
    pub phase: Phase,
    pub ended_by: Option<EndedBy>,
    /// The agent's live line: "Reading Q3 planning".
    pub line: String,
    /// The agent's last message.
    pub last_message: String,
    pub notes_read: usize,
    pub counts: Counts,
    pub elapsed: Duration,
    /// When the transcript last grew, in Unix milliseconds.
    pub last_activity_ms: i64,
    /// New items since the session was last opened.
    pub unread: bool,
    /// The agent conversation is warm: a message can still be sent to it.
    pub live: bool,
    /// The round of proposals the session is in; 1 until a follow-up.
    pub round: u32,
    /// Read back from disk after a restart: nothing runs it until it is
    /// resumed or continued.
    pub restored: bool,
    /// A message sent to it can pick the agent's conversation up again.
    pub resumable: bool,
    /// The session this one carries on from.
    pub continued_from: Option<SessionKey>,
    /// The session that carries this one on.
    pub continued_in: Option<SessionKey>,
}

/// What the composer under a transcript does now.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Composer {
    /// Enter sends.
    Send,
    /// A turn is running: the button stops it, and what is typed waits as
    /// the next message.
    Stop,
    /// Can't send, and why: a whole sentence.
    Off(&'static str),
}

impl Composer {
    /// The composer for `row`; `ai_on` is whether AI is on in Settings.
    pub fn of(row: &TaskRow, ai_on: bool) -> Composer {
        if !ai_on {
            Composer::Off("AI is off in Settings.")
        } else if row.restored {
            // Read back after a restart: a message picks the conversation up
            // again, when the agent can.
            if row.resumable && !row.is_active() {
                Composer::Send
            } else {
                Composer::Off(
                    "This session can't be picked up again. Continue in a new session to keep going.",
                )
            }
        } else if !row.live {
            Composer::Off("This session ended. Continue in a new session to keep going.")
        } else if row.is_active() {
            Composer::Stop
        } else if matches!(row.phase, Phase::Failed { .. }) {
            Composer::Off("The agent stopped. Continue in a new session to keep going.")
        } else {
            Composer::Send
        }
    }
}

/// The colour of a row's status dot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dot {
    /// Blue and pulsing.
    Working,
    /// Amber: a question, changes to review.
    NeedsYou,
    /// Grey.
    Idle,
    /// Red.
    Failed,
}

/// Whether a session has items its viewer hasn't seen: the transcript's
/// version has moved past the one last opened.
pub fn is_unread(seen_version: u64, version: u64) -> bool {
    version > seen_version
}

/// How long ago `then_ms` was at `now_ms`, in a few words: `now`, `2m ago`,
/// `3h ago`, `yesterday`, `4d ago`.
pub fn when_label(now_ms: i64, then_ms: i64) -> String {
    let secs = (now_ms - then_ms).max(0) / 1000;
    match secs {
        0..60 => "now".to_owned(),
        60..3600 => format!("{}m ago", secs / 60),
        3600..86_400 => format!("{}h ago", secs / 3600),
        86_400..172_800 => "yesterday".to_owned(),
        _ => format!("{}d ago", secs / 86_400),
    }
}

/// The rows in the order the list shows them: latest activity first.
pub fn by_activity(rows: &mut [TaskRow]) {
    rows.sort_by(|a, b| {
        b.last_activity_ms
            .cmp(&a.last_activity_ms)
            .then(b.id.0.cmp(&a.id.0))
    });
}

/// A goal cut to one line for a title.
pub fn title_of(goal: &str) -> String {
    let line = goal.lines().next().unwrap_or_default().trim();
    let mut title: String = line.chars().take(80).collect();
    if line.chars().count() > 80 {
        title.push('…');
    }
    title
}

impl TaskRow {
    pub fn new(id: TaskId, goal: &str, agent: &str) -> TaskRow {
        TaskRow {
            id,
            key: SessionKey::default(),
            goal: goal.to_owned(),
            title: title_of(goal),
            agent: agent.to_owned(),
            phase: Phase::Running,
            ended_by: None,
            line: String::new(),
            last_message: String::new(),
            notes_read: 0,
            counts: Counts::default(),
            elapsed: Duration::ZERO,
            last_activity_ms: 0,
            unread: false,
            live: true,
            round: 1,
            restored: false,
            resumable: false,
            continued_from: None,
            continued_in: None,
        }
    }

    /// The status in a list row: `Working…`, `4 changes to review`, `Done`.
    pub fn list_status(&self) -> String {
        match &self.phase {
            Phase::Waiting => "Waiting".to_owned(),
            Phase::Running => "Working…".to_owned(),
            Phase::Idle => "Done".to_owned(),
            Phase::Asking { .. } => "Has a question".to_owned(),
            Phase::Ready => {
                let n = self.counts.total();
                format!("{n} change{} to review", if n == 1 { "" } else { "s" })
            }
            Phase::Nothing => "Done".to_owned(),
            Phase::Failed { .. } => "Failed".to_owned(),
            Phase::Applied { changes } => format!(
                "Applied {changes} change{}",
                if *changes == 1 { "" } else { "s" }
            ),
            Phase::Undone => "Undone".to_owned(),
            Phase::Discarded => "Discarded".to_owned(),
        }
    }

    /// The line under a list row's title: `Claude Agent · Working… · now`.
    pub fn list_line(&self, now_ms: i64) -> String {
        format!(
            "{} · {} · {}",
            self.agent,
            self.list_status(),
            when_label(now_ms, self.last_activity_ms)
        )
    }

    /// The row's status dot.
    pub fn dot(&self) -> Dot {
        match &self.phase {
            Phase::Waiting | Phase::Running => Dot::Working,
            Phase::Asking { .. } | Phase::Ready => Dot::NeedsYou,
            Phase::Failed { .. } => Dot::Failed,
            Phase::Idle
            | Phase::Nothing
            | Phase::Applied { .. }
            | Phase::Undone
            | Phase::Discarded => Dot::Idle,
        }
    }

    /// Whether a turn is running (or waiting to). A session that is only
    /// open, between turns, is not active.
    pub fn is_active(&self) -> bool {
        matches!(self.phase, Phase::Waiting | Phase::Running)
    }

    /// Whether a message can go to the agent now.
    pub fn can_follow_up(&self) -> bool {
        self.live && !self.is_active() && !matches!(self.phase, Phase::Failed { .. })
    }

    /// Whether End Session does anything.
    pub fn can_end(&self) -> bool {
        self.live
    }

    /// Whether "Continue in a new session" is the way on: it ended (or was
    /// read back after a restart), can't be resumed, and has no successor.
    pub fn can_continue(&self) -> bool {
        !self.is_active()
            && !self.resumable
            && self.continued_in.is_none()
            && (self.restored || !self.live)
    }

    pub fn can_stop(&self) -> bool {
        self.is_active()
    }

    pub fn can_review(&self) -> bool {
        self.phase == Phase::Ready
            || (matches!(self.phase, Phase::Failed { .. }) && self.counts.total() > 0)
    }

    pub fn can_discard(&self) -> bool {
        self.can_review() || self.phase == Phase::Nothing
    }

    /// Whether this task has changes nobody has reviewed.
    pub fn has_unreviewed(&self) -> bool {
        self.can_review()
            || ((self.is_active() || matches!(self.phase, Phase::Asking { .. }))
                && self.counts.total() > 0)
    }

    /// "Ready to review: 3 new notes, 5 edits, 6 moved".
    pub fn ready_line(&self) -> String {
        ready_line(&self.counts)
    }

    /// The line under the goal.
    pub fn status_line(&self) -> String {
        match &self.phase {
            Phase::Waiting => "Waiting for the agent to finish another task".to_owned(),
            Phase::Running if self.line.is_empty() => "Working…".to_owned(),
            Phase::Running => self.line.clone(),
            Phase::Idle => "Ask for more, or end the session.".to_owned(),
            Phase::Asking { .. } => "The agent has a question".to_owned(),
            Phase::Ready => format!(
                "{}{}",
                self.ready_line(),
                match self.ended_by {
                    Some(EndedBy::TimeLimit) => " (stopped at the time limit)",
                    Some(EndedBy::CallLimit) => " (stopped at the tool-call limit)",
                    Some(EndedBy::Stopped) => " (stopped)",
                    Some(EndedBy::IdleClose) => " (closed after 30 minutes idle)",
                    Some(EndedBy::Ended) => " (session ended)",
                    _ => "",
                }
            ),
            Phase::Nothing => "The agent didn't propose any changes.".to_owned(),
            Phase::Failed { message } => format!("The AI stopped with an error: {message}"),
            Phase::Applied { changes } => format!(
                "Applied {changes} change{}",
                if *changes == 1 { "" } else { "s" }
            ),
            Phase::Undone => "Undone".to_owned(),
            Phase::Discarded => "Discarded".to_owned(),
        }
    }

    /// "Notes read: 14 · Changes proposed: 2".
    pub fn counts_line(&self) -> String {
        format!(
            "Notes read: {} · Changes proposed: {}",
            self.notes_read,
            self.counts.total()
        )
    }
}

/// "Ready to review: 3 new notes, 5 edits, 6 moved".
pub fn ready_line(counts: &Counts) -> String {
    let moved = if counts.moves == 0 {
        String::new()
    } else {
        format!("{} moved", counts.moves)
    };
    let without_moves = Counts {
        moves: 0,
        ..*counts
    };
    let mut parts = Vec::new();
    if without_moves.total() > 0 {
        parts.push(without_moves.describe());
    }
    if !moved.is_empty() {
        parts.push(moved);
    }
    if parts.is_empty() {
        "Ready to review: no changes".to_owned()
    } else {
        format!("Ready to review: {}", parts.join(", "))
    }
}

/// "12:03" or "1:02:03": how long a task has run.
pub fn elapsed_label(elapsed: Duration) -> String {
    let secs = elapsed.as_secs();
    let (h, m, s) = (secs / 3600, secs % 3600 / 60, secs % 60);
    if h > 0 {
        format!("{h}:{m:02}:{s:02}")
    } else {
        format!("{m}:{s:02}")
    }
}

/// The status bar's line for the tasks running, or `None` when none are:
/// "Agent: 14 notes read, 2 changes".
pub fn status_bar_line(rows: &[TaskRow]) -> Option<String> {
    let active: Vec<&TaskRow> = rows.iter().filter(|row| row.is_active()).collect();
    if active.is_empty() {
        return None;
    }
    let notes: usize = active.iter().map(|row| row.notes_read).sum();
    let changes: usize = active.iter().map(|row| row.counts.total()).sum();
    Some(format!(
        "Agent: {notes} note{} read, {changes} change{}",
        if notes == 1 { "" } else { "s" },
        if changes == 1 { "" } else { "s" }
    ))
}

/// The prompt when quitting with unreviewed change sets, or `None`.
pub fn quit_prompt(rows: &[TaskRow]) -> Option<String> {
    let n = rows.iter().filter(|row| row.has_unreviewed()).count();
    match n {
        0 => None,
        1 => Some("1 agent task has changes you haven't reviewed. Quit anyway?".to_owned()),
        n => Some(format!(
            "{n} agent tasks have changes you haven't reviewed. Quit anyway?"
        )),
    }
}

/// What a session may read, in a few words: `Campus Pilot`, `3 notes`.
pub fn reads_label(read: &ReadScope, book_name: &dyn Fn(&str) -> String) -> String {
    match read {
        ReadScope::Notes(ids) => format!(
            "{} note{}",
            ids.len(),
            if ids.len() == 1 { "" } else { "s" }
        ),
        ReadScope::Notebook(id) => book_name(id),
        ReadScope::Allowed(_) => "Notebooks you allowed".to_owned(),
        ReadScope::Everything => "All notebooks".to_owned(),
    }
}

/// What a session may propose: `new notes, edits, moves, merges`.
pub fn ops_label(ops: &TaskOps) -> String {
    let mut parts = Vec::new();
    if ops.create {
        parts.push("new notes");
    }
    if ops.edit {
        parts.push("edits");
    }
    if ops.moves {
        parts.push("moves");
    }
    if ops.merge_allowed() {
        parts.push("merges");
    }
    if parts.is_empty() {
        "nothing".to_owned()
    } else {
        parts.join(", ")
    }
}

/// The scope in full, for the header chip:
/// `Reads: Campus Pilot · May propose: new notes, edits · New notes go in: Campus Pilot`.
pub fn scope_line(spec: &TaskSpec, book_name: &dyn Fn(&str) -> String) -> String {
    format!(
        "Reads: {} · May propose: {} · New notes go in: {}",
        reads_label(&spec.read, book_name),
        ops_label(&spec.ops),
        book_name(&spec.new_note_book)
    )
}

/// What the notification says when a task finishes while Kanso is behind
/// other windows.
pub const NOTIFICATION: &str = "Agent task ready to review";

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn counts(creates: usize, edits: usize, moves: usize, merges: usize) -> Counts {
        Counts {
            creates,
            edits,
            moves,
            merges,
        }
    }

    fn row(phase: Phase, counts: Counts) -> TaskRow {
        TaskRow {
            phase,
            counts,
            ..TaskRow::new(TaskId(1), "Merge trips", "Claude Agent")
        }
    }

    #[test]
    fn list_rows_say_the_status_of_each_phase() {
        let c = counts(2, 2, 0, 0);
        let said = |phase: Phase| row(phase, c).list_status();
        assert_eq!(said(Phase::Waiting), "Waiting");
        assert_eq!(said(Phase::Running), "Working…");
        assert_eq!(
            said(Phase::Asking {
                question: "?".into()
            }),
            "Has a question"
        );
        assert_eq!(said(Phase::Ready), "4 changes to review");
        assert_eq!(
            row(Phase::Ready, counts(1, 0, 0, 0)).list_status(),
            "1 change to review"
        );
        assert_eq!(said(Phase::Nothing), "Done");
        assert_eq!(said(Phase::Idle), "Done");
        assert_eq!(
            said(Phase::Failed {
                message: "x".into()
            }),
            "Failed"
        );
        assert_eq!(said(Phase::Applied { changes: 1 }), "Applied 1 change");
        assert_eq!(said(Phase::Undone), "Undone");
        assert_eq!(said(Phase::Discarded), "Discarded");
    }

    #[test]
    fn the_line_under_a_title_has_agent_status_and_time() {
        let mut working = row(Phase::Running, Counts::default());
        working.last_activity_ms = 1_000_000;
        assert_eq!(
            working.list_line(1_010_000),
            "Claude Agent · Working… · now"
        );
        let mut ready = row(Phase::Ready, counts(4, 0, 0, 0));
        ready.last_activity_ms = 1_000_000;
        assert_eq!(
            ready.list_line(1_000_000 + 120_000),
            "Claude Agent · 4 changes to review · 2m ago"
        );
    }

    #[test]
    fn times_read_like_a_person() {
        assert_eq!(when_label(5_000, 5_000), "now");
        assert_eq!(when_label(0, 5_000), "now");
        assert_eq!(when_label(3_600_000, 0), "1h ago");
        assert_eq!(when_label(86_400_000, 0), "yesterday");
        assert_eq!(when_label(3 * 86_400_000, 0), "3d ago");
    }

    #[test]
    fn dots_follow_what_the_person_must_do() {
        assert_eq!(row(Phase::Running, counts(0, 0, 0, 0)).dot(), Dot::Working);
        assert_eq!(row(Phase::Waiting, counts(0, 0, 0, 0)).dot(), Dot::Working);
        assert_eq!(row(Phase::Ready, counts(1, 0, 0, 0)).dot(), Dot::NeedsYou);
        assert_eq!(
            row(
                Phase::Asking {
                    question: "?".into()
                },
                counts(0, 0, 0, 0)
            )
            .dot(),
            Dot::NeedsYou
        );
        assert_eq!(row(Phase::Nothing, counts(0, 0, 0, 0)).dot(), Dot::Idle);
        assert_eq!(
            row(Phase::Applied { changes: 1 }, counts(1, 0, 0, 0)).dot(),
            Dot::Idle
        );
        assert_eq!(
            row(
                Phase::Failed {
                    message: "x".into()
                },
                counts(0, 0, 0, 0)
            )
            .dot(),
            Dot::Failed
        );
    }

    #[test]
    fn the_scope_reads_in_one_line() {
        let name = |id: &str| format!("Book {id}");
        let mut spec = TaskSpec::new("g", ReadScope::Notebook("1".into()), "2");
        assert_eq!(
            scope_line(&spec, &name),
            "Reads: Book 1 · May propose: new notes, edits, moves, merges · New notes go in: Book 2"
        );
        spec.read = ReadScope::Notes(vec!["a".into()]);
        spec.ops.merge = false;
        spec.ops.moves = false;
        assert_eq!(
            scope_line(&spec, &name),
            "Reads: 1 note · May propose: new notes, edits · New notes go in: Book 2"
        );
        spec.ops = TaskOps {
            create: false,
            edit: false,
            moves: false,
            merge: false,
        };
        assert!(scope_line(&spec, &name).contains("May propose: nothing"));
    }

    #[test]
    fn a_session_is_unread_until_its_version_is_seen() {
        assert!(!is_unread(3, 3));
        assert!(is_unread(3, 4));
        assert!(!is_unread(4, 3));
    }

    #[test]
    fn the_list_is_newest_activity_first() {
        let at = |id: u64, ms: i64| TaskRow {
            last_activity_ms: ms,
            ..TaskRow::new(TaskId(id), "g", "a")
        };
        let mut rows = vec![at(1, 10), at(2, 30), at(3, 20)];
        by_activity(&mut rows);
        assert_eq!(rows.iter().map(|r| r.id.0).collect::<Vec<_>>(), [2, 3, 1]);
    }

    #[test]
    fn the_ready_line_reads_as_the_issue_says() {
        assert_eq!(
            ready_line(&counts(3, 5, 6, 0)),
            "Ready to review: 3 new notes, 5 edits, 6 moved"
        );
        assert_eq!(
            ready_line(&counts(1, 0, 0, 0)),
            "Ready to review: 1 new note"
        );
        assert_eq!(
            ready_line(&counts(0, 0, 0, 0)),
            "Ready to review: no changes"
        );
    }

    #[test]
    fn the_status_bar_counts_only_active_tasks() {
        let mut running = row(Phase::Running, counts(0, 2, 0, 0));
        running.notes_read = 14;
        let ready = row(Phase::Ready, counts(9, 9, 9, 9));
        assert_eq!(
            status_bar_line(&[running.clone(), ready.clone()]).as_deref(),
            Some("Agent: 14 notes read, 2 changes")
        );
        assert_eq!(status_bar_line(&[ready]), None);
        running.notes_read = 1;
        running.counts = counts(1, 0, 0, 0);
        assert_eq!(
            status_bar_line(&[running]).as_deref(),
            Some("Agent: 1 note read, 1 change")
        );
    }

    #[test]
    fn quitting_asks_only_about_unreviewed_changes() {
        let ready = row(Phase::Ready, counts(1, 0, 0, 0));
        let applied = row(Phase::Applied { changes: 1 }, counts(1, 0, 0, 0));
        let nothing = row(Phase::Nothing, counts(0, 0, 0, 0));
        assert_eq!(quit_prompt(&[applied, nothing]), None);
        assert_eq!(
            quit_prompt(std::slice::from_ref(&ready)).as_deref(),
            Some("1 agent task has changes you haven't reviewed. Quit anyway?")
        );
        assert!(
            quit_prompt(&[ready.clone(), ready])
                .unwrap()
                .starts_with("2 agent tasks")
        );
    }

    #[test]
    fn a_failed_task_keeps_its_changes_for_review() {
        let failed = row(
            Phase::Failed {
                message: "timeout".into(),
            },
            counts(0, 1, 0, 0),
        );
        assert!(failed.can_review() && failed.can_discard());
        assert!(
            failed
                .status_line()
                .contains("The AI stopped with an error: timeout")
        );
        let empty = row(
            Phase::Failed {
                message: "x".into(),
            },
            counts(0, 0, 0, 0),
        );
        assert!(!empty.can_review());
    }

    #[test]
    fn the_composer_says_why_it_cannot_send() {
        let mut idle = row(Phase::Idle, Counts::default());
        assert_eq!(Composer::of(&idle, true), Composer::Send);
        assert_eq!(
            Composer::of(&idle, false),
            Composer::Off("AI is off in Settings.")
        );
        let running = row(Phase::Running, Counts::default());
        assert_eq!(Composer::of(&running, true), Composer::Stop);
        idle.live = false;
        assert!(matches!(Composer::of(&idle, true), Composer::Off(why) if why.contains("ended")));
        let asking = row(
            Phase::Asking {
                question: "?".into(),
            },
            counts(1, 0, 0, 0),
        );
        assert_eq!(Composer::of(&asking, true), Composer::Send);
        assert!(asking.has_unreviewed(), "a question keeps its changes");
        assert!(!asking.is_active(), "a question is not a running turn");
    }

    #[test]
    fn elapsed_time_is_minutes_and_seconds() {
        assert_eq!(elapsed_label(Duration::from_secs(5)), "0:05");
        assert_eq!(elapsed_label(Duration::from_secs(125)), "2:05");
        assert_eq!(elapsed_label(Duration::from_secs(3725)), "1:02:05");
    }

    proptest! {
        /// A row only offers Stop while active, only offers Review when
        /// something is waiting, and never both.
        #[test]
        fn prop_actions_follow_the_phase(c in (0usize..5, 0usize..5, 0usize..5, 0usize..5), pick in 0u8..10) {
            let phase = match pick {
                0 => Phase::Waiting,
                1 => Phase::Running,
                2 => Phase::Asking { question: "?".into() },
                3 => Phase::Ready,
                4 => Phase::Nothing,
                5 => Phase::Failed { message: "x".into() },
                6 => Phase::Applied { changes: 1 },
                7 => Phase::Undone,
                8 => Phase::Idle,
                _ => Phase::Discarded,
            };
            let r = row(phase, counts(c.0, c.1, c.2, c.3));
            prop_assert!(!(r.can_stop() && r.can_review()));
            prop_assert_eq!(r.can_stop(), r.is_active());
            prop_assert!(!r.status_line().is_empty());
        }

        /// The ready line always starts the same and the moved count is the
        /// moves.
        #[test]
        fn prop_ready_line_counts_moves(m in 0usize..50, e in 0usize..50) {
            let line = ready_line(&counts(0, e, m, 0));
            prop_assert!(line.starts_with("Ready to review: "));
            prop_assert_eq!(line.contains(&format!("{m} moved")), m > 0);
        }
    }
}
