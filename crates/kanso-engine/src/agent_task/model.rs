//! What the Agent Tasks panel and the status bar say about a task. A model
//! file: values and the words for them, nothing that runs or draws.

use std::time::Duration;

use kanso_server::{Counts, TaskId};

/// Where a task is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Another task is using the agent.
    Waiting,
    Running,
    /// The agent asked a question and waits for the person.
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
    /// Waited 30 minutes for a reply.
    NoReply,
}

/// One row of the Agent Tasks panel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskRow {
    pub id: TaskId,
    pub goal: String,
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
}

impl TaskRow {
    pub fn new(id: TaskId, goal: &str, agent: &str) -> TaskRow {
        TaskRow {
            id,
            goal: goal.to_owned(),
            agent: agent.to_owned(),
            phase: Phase::Running,
            ended_by: None,
            line: String::new(),
            last_message: String::new(),
            notes_read: 0,
            counts: Counts::default(),
            elapsed: Duration::ZERO,
        }
    }

    /// Whether the task is still working (or waiting to).
    pub fn is_active(&self) -> bool {
        matches!(
            self.phase,
            Phase::Waiting | Phase::Running | Phase::Asking { .. }
        )
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
        self.can_review() || (self.is_active() && self.counts.total() > 0)
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
            Phase::Asking { .. } => "The agent has a question".to_owned(),
            Phase::Ready => format!(
                "{}{}",
                self.ready_line(),
                match self.ended_by {
                    Some(EndedBy::TimeLimit) => " (stopped at the time limit)",
                    Some(EndedBy::CallLimit) => " (stopped at the tool-call limit)",
                    Some(EndedBy::Stopped) => " (stopped)",
                    Some(EndedBy::NoReply) => " (no reply, so it stopped)",
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
    fn elapsed_time_is_minutes_and_seconds() {
        assert_eq!(elapsed_label(Duration::from_secs(5)), "0:05");
        assert_eq!(elapsed_label(Duration::from_secs(125)), "2:05");
        assert_eq!(elapsed_label(Duration::from_secs(3725)), "1:02:05");
    }

    proptest! {
        /// A row only offers Stop while active, only offers Review when
        /// something is waiting, and never both.
        #[test]
        fn prop_actions_follow_the_phase(c in (0usize..5, 0usize..5, 0usize..5, 0usize..5), pick in 0u8..9) {
            let phase = match pick {
                0 => Phase::Waiting,
                1 => Phase::Running,
                2 => Phase::Asking { question: "?".into() },
                3 => Phase::Ready,
                4 => Phase::Nothing,
                5 => Phase::Failed { message: "x".into() },
                6 => Phase::Applied { changes: 1 },
                7 => Phase::Undone,
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
