//! What the Tasks place's open loops, the morning brief and the weekly
//! review hand the app. The rules are `kanso_core::loops` and
//! `kanso_engine::brief`, which the desktop runs too.

use crate::model::TaskItem;

/// What kind of promise a loop is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpenLoopKind {
    /// Something the writer will do.
    Mine,
    /// Something somebody else owes the writer.
    Waiting,
    /// Something to find out.
    Question,
}

/// How a loop is closed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoopAction {
    /// Done: remembered, and ticked in the note when Settings says so.
    Done,
    /// "Not a loop": hidden for good.
    NotALoop,
    /// The sentence becomes a task in its note.
    MakeTask,
}

/// An open loop, with the note it is in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenLoopItem {
    /// Names the loop to [`Library::close_loop`](crate::Library::close_loop).
    pub key: String,
    pub note_id: String,
    pub note_title: String,
    pub notebook_id: String,
    /// The line (0-based) in the note's text.
    pub line: u32,
    pub kind: OpenLoopKind,
    /// The sentence.
    pub text: String,
    /// A date the sentence names ("Fri 25 Oct"), if it does.
    pub due_label: Option<String>,
    /// How long ago the note was last changed: "3 days ago".
    pub age_label: String,
    /// Found by AI and approved, not by the phrase rules.
    pub from_ai: bool,
}

/// The open loops, newest notes first.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenLoopsBoard {
    pub items: Vec<OpenLoopItem>,
    /// How many are open in all (the list may be cut short).
    pub total: u32,
    /// Whether the detector is on in Settings.
    pub enabled: bool,
}

/// A timed task due later today.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BriefReminder {
    pub note_id: String,
    pub note_title: String,
    pub line: u32,
    pub text: String,
    /// "09:30".
    pub time_label: String,
}

/// A past note the brief suggests, and why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BriefNote {
    pub note_id: String,
    pub title: String,
    /// "A week ago today".
    pub why: String,
    pub excerpt: String,
}

/// The morning brief, worked out now.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BriefView {
    /// "Monday 2 November".
    pub date_label: String,
    /// Open tasks due today or earlier, overdue first.
    pub due: Vec<TaskItem>,
    pub loops: Vec<OpenLoopItem>,
    pub loops_total: u32,
    pub later_today: Vec<BriefReminder>,
    pub relevant: Vec<BriefNote>,
    pub wrote_yesterday: u32,
    pub finished_yesterday: u32,
    /// Nothing due and no open loops.
    pub is_empty: bool,
}

/// A note written or changed in the reviewed week.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewWritten {
    pub note_id: String,
    pub title: String,
    pub notebook_name: String,
    pub created: bool,
}

/// A task ticked or a loop closed in the reviewed week.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewFinished {
    pub note_id: String,
    pub note_title: String,
    pub text: String,
    /// Whether it was a loop (otherwise a task).
    pub was_loop: bool,
}

/// A recurring theme and how many notes it was in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewTheme {
    pub term: String,
    pub notes: u32,
    /// A tag (otherwise a word or phrase).
    pub is_tag: bool,
}

/// The weekly review of seven days.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewView {
    /// The first day, `YYYY-MM-DD`: what `Library::review` is asked with.
    pub from: String,
    /// "2026-W45".
    pub week_label: String,
    /// "2 Nov to 8 Nov 2026".
    pub range_label: String,
    /// "12 new, 31 changed".
    pub counts_text: String,
    pub written: Vec<ReviewWritten>,
    pub finished: Vec<ReviewFinished>,
    pub still_open: Vec<OpenLoopItem>,
    pub overdue: Vec<TaskItem>,
    pub themes: Vec<ReviewTheme>,
    pub next: Vec<TaskItem>,
    /// The note this week's review is already saved as, if it is.
    pub saved_note_id: Option<String>,
    /// This is the week of today.
    pub is_this_week: bool,
}

/// Where a saved review went.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReviewSaved {
    pub note_id: String,
    /// The week was already saved and nothing was written: ask whether to
    /// replace it.
    pub exists: bool,
}

/// Settings › Tasks › Brief and review.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BriefPrefs {
    pub loops: bool,
    pub brief: bool,
    pub brief_hour: u32,
    pub brief_minute: u32,
    pub brief_weekdays_only: bool,
    pub review: bool,
    /// 0 for Monday through 6 for Sunday.
    pub review_day: u32,
    pub review_hour: u32,
    pub review_minute: u32,
    pub notify_brief: bool,
    pub notify_review: bool,
    pub mark_done_in_note: bool,
}

/// A notification to hand the system ahead of time. Its text is fixed:
/// counts can't be known days ahead. It carries wall-clock fields; the app
/// builds the instant in its own zone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlannedNotice {
    /// Stable, so scheduling it again replaces it: "brief-2026-11-03".
    pub id: String,
    /// Whether it opens the review (otherwise the brief).
    pub is_review: bool,
    pub title: String,
    pub body: String,
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
}
