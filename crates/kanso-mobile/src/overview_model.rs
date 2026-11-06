//! What Overview hands the phone. The rules, the order and the caps are
//! `kanso_engine::overview`, which the desktop runs too; these types are the
//! page with its words already said (`3d late`, `12 min ago`, `Now`), so the
//! screens draw and do not decide.

/// What an agenda row is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverviewRowKind {
    /// An unchecked task due before today.
    Overdue,
    Event,
    /// A task due today at a time.
    Reminder,
    /// A task due today without a time.
    Due,
}

/// A row of the agenda: a task or an event, with the words for its gutter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverviewAgendaRow {
    pub kind: OverviewRowKind,
    /// Names the task or the event.
    pub key: String,
    pub title: String,
    /// The line under the title.
    pub context: String,
    /// The time column: `09:30`, `Now`, `2d late` or `Today`.
    pub gutter: String,
    pub now: bool,
    pub free: bool,
    pub note_id: String,
    pub line: u32,
    /// A promise in a note, not a task: ticking it closes the loop.
    pub is_loop: bool,
    /// The task's words as written in its note, for ticking it.
    pub raw: String,
    /// The calendar's `#rrggbb`, or empty.
    pub color: String,
    /// The meeting note this event links to, or empty.
    pub meeting_note_id: String,
    /// `this`, `series`, or `last` for the linked event note.
    pub meeting_which: String,
    /// The local day of the prior meeting note, when `meeting_which` is `last`.
    pub meeting_day: String,
    /// The local system-calendar event as JSON, used to create a meeting note.
    pub event_json: String,
}

/// Which List.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverviewListKind {
    PickUp,
    Waiting,
    Activity,
}

/// A row of a List.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverviewListRow {
    pub kind: OverviewListKind,
    pub note_id: String,
    pub line: u32,
    pub title: String,
    pub detail: String,
    pub place: String,
    /// `2 h ago`.
    pub when_label: String,
    pub quiet_days: u32,
    /// Quiet for ten days or more.
    pub warn: bool,
    pub key: String,
    pub person: String,
    pub raw: String,
    /// A promise (closed with Done) rather than a task (ticked).
    pub is_loop: bool,
    /// Draw the caret after the words.
    pub caret: bool,
    pub count: u32,
    /// Edits waiting for review, not a person's change.
    pub agent: bool,
}

/// A List: three rows shown, then `more` rows (the last ones in `rows`)
/// that `N more` shows in place.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverviewList {
    pub kind: OverviewListKind,
    pub label: String,
    pub rows: Vec<OverviewListRow>,
    pub more: u32,
}

/// Which Line of *Also*.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverviewLineKind {
    Revisit,
    Captured,
    Promises,
    Themes,
    Meetings,
    Activity,
    Waiting,
    PickUp,
}

/// A note an *Also* line opens.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverviewItem {
    pub note_id: String,
    pub line: u32,
    pub text: String,
    /// A voice drop's length in seconds, or 0.
    pub voice_secs: u32,
}

/// A Line of *Also*.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverviewLine {
    pub kind: OverviewLineKind,
    pub label: String,
    pub count: u32,
    pub cards: u32,
    pub items: Vec<OverviewItem>,
    pub detail: Vec<String>,
}

/// The one note from the past.
#[derive(Clone, Debug, PartialEq)]
pub struct OverviewPast {
    pub note_id: String,
    pub title: String,
    pub excerpt: String,
    pub why: String,
}

/// The model's paragraph, with its citations already linked to notes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverviewAi {
    pub text: String,
    pub warning: Option<String>,
    /// `HH:MM` it was written.
    pub time_label: String,
}

/// How full the day is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverviewState {
    /// A library with nothing in it yet.
    New,
    Quiet,
    Busy,
}

/// The Today page.
#[derive(Clone, Debug, PartialEq)]
pub struct OverviewToday {
    /// `YYYY-MM-DD`.
    pub day: String,
    pub greeting: String,
    /// `WEDNESDAY 4 NOVEMBER`.
    pub overline: String,
    pub lede: String,
    pub state: OverviewState,
    pub earlier: Vec<OverviewAgendaRow>,
    pub agenda: Vec<OverviewAgendaRow>,
    pub agenda_more: u32,
    pub task_total: u32,
    pub lists: Vec<OverviewList>,
    pub also: Vec<OverviewLine>,
    pub past: Option<OverviewPast>,
    pub footer: String,
    pub ai: Option<OverviewAi>,
}

/// A notebook the week went into.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverviewProject {
    pub id: String,
    pub name: String,
    pub notes_changed: u32,
    pub last_title: String,
    pub last_note_id: String,
    pub next_task: String,
    pub next_due: String,
}

/// Something finished this week.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverviewFinished {
    pub note_id: String,
    pub text: String,
}

/// A day of *Ahead*.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverviewAheadDay {
    pub summary: String,
    pub rows: Vec<OverviewAgendaRow>,
}

/// The This week page.
#[derive(Clone, Debug, PartialEq)]
pub struct OverviewWeek {
    /// The Monday, `YYYY-MM-DD`.
    pub from: String,
    pub range_label: String,
    pub week_label: String,
    pub is_this_week: bool,
    pub lede: String,
    pub projects: Vec<OverviewProject>,
    pub finished: Vec<OverviewFinished>,
    pub finished_more: u32,
    pub waiting: Vec<OverviewListRow>,
    pub still_open: Vec<OverviewAgendaRow>,
    pub still_open_total: u32,
    pub ahead: Vec<OverviewAheadDay>,
    pub also: Vec<OverviewLine>,
    /// The saved review note's title, or empty.
    pub saved_as: String,
    pub ai: Option<OverviewAi>,
}

/// The choices of *Revisit in…*, in the menu's order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RevisitChoice {
    Tomorrow,
    InThreeDays,
    NextWeek,
    InTwoWeeks,
    InAMonth,
    InThreeMonths,
}

/// A date offered by *Revisit in…*, with its words.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RevisitOption {
    pub choice: RevisitChoice,
    pub label: String,
    /// `Fri 6 Nov`.
    pub day_label: String,
}
