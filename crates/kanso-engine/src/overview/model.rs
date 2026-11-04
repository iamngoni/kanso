//! What Overview holds, as plain data: the page for Today and the page for
//! This week. Every type is serialisable, because the first frame draws from
//! a snapshot of the last page kept in the library's local meta.
//!
//! Times are raw (UTC milliseconds, minutes into the local day); the screens
//! turn them into words, with [`ago`] and [`clock`] shared so the desktop and
//! the phone say the same thing.

use serde::{Deserialize, Serialize};

/// The snapshot format. A snapshot of another version is ignored.
pub const VERSION: u32 = 1;

/// A calendar event, as the agenda needs it. Read from the device's own
/// calendars or an ICS address, kept in local tables, never synced.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    /// The iCalendar UID. Together with `start` it names one instance.
    pub uid: String,
    /// The instance's start (UTC ms).
    pub start: i64,
    pub end: i64,
    pub title: String,
    pub location: String,
    pub conference_url: String,
    pub attendees: Vec<Attendee>,
    pub calendar: String,
    /// The calendar's colour as `#rrggbb`, or empty.
    pub color: String,
    pub all_day: bool,
    /// Marked "free" rather than busy.
    pub free: bool,
    /// The person declined it.
    pub declined: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attendee {
    pub name: String,
    pub email: String,
    /// The People page it resolves to, when there is one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,
}

/// What an agenda row is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgendaKind {
    /// An unchecked task due before today.
    Overdue,
    Event,
    /// A task due today at a time.
    Reminder,
    /// A task due today without a time.
    Due,
}

/// A meeting note an event row links to.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeetingLink {
    pub note_id: String,
    /// `this` for the instance's own note, `last` for the newest earlier
    /// instance's, `series` for a note of the whole series.
    pub which: String,
    /// For `last`: the day it was for (`YYYY-MM-DD`).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub day: String,
    pub locked: bool,
}

/// One row of the agenda.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgendaRow {
    pub kind: AgendaKind,
    /// The task's key, or the event's `uid@start`.
    pub key: String,
    pub title: String,
    /// The line under the title: the task's note, or the event's time,
    /// attendees and place.
    pub context: String,
    /// Minutes into the local day it starts at, when it has a time.
    pub at_min: Option<u32>,
    pub end_min: Option<u32>,
    /// For an overdue row, how many days late.
    pub days_late: u32,
    /// An event happening now.
    pub now: bool,
    pub free: bool,
    pub note_id: String,
    pub line: u32,
    /// A promise in a note whose date passed ("promised 3 Nov"), not a task:
    /// ticking it closes the loop.
    pub is_loop: bool,
    /// The task's due text (`YYYY-MM-DD[THH:MM]`).
    pub due: String,
    pub meeting: Option<MeetingLink>,
    pub color: String,
    /// The event's UID and start, for linking it to a note.
    pub event: Option<(String, i64)>,
}

impl AgendaRow {
    pub fn is_task(&self) -> bool {
        self.kind != AgendaKind::Event
    }
}

/// The lead section: today's rows, in order.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Agenda {
    /// Events that are over, folded into one line at the top.
    pub earlier: Vec<AgendaRow>,
    /// At most [`crate::overview::density::AGENDA_CAP`] rows from now on.
    pub rows: Vec<AgendaRow>,
    /// How many more rows there were.
    pub more: u32,
    /// How many of those were tasks (`All 12 in Tasks`).
    pub more_tasks: u32,
    /// How many tasks are on the agenda in all.
    pub task_total: u32,
}

impl Agenda {
    pub fn is_empty(&self) -> bool {
        self.earlier.is_empty() && self.rows.is_empty()
    }
}

/// The Lists.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ListKind {
    PickUp,
    Waiting,
    Activity,
}

impl ListKind {
    pub fn label(self) -> &'static str {
        match self {
            ListKind::PickUp => "Pick up where you left off",
            ListKind::Waiting => "Waiting on others",
            ListKind::Activity => "Agent and shared activity",
        }
    }
}

/// What a List row is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RowKind {
    PickUp,
    WaitingLoop,
    WaitingTask,
    /// Edits an agent suggested, waiting for review.
    AgentSuggestion,
    /// An agent task with changes nobody has reviewed.
    AgentTask,
    /// A shared note changed by someone else.
    SharedEdit,
    /// A comment that mentions the person.
    Mention,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListRow {
    pub kind: RowKind,
    pub note_id: String,
    pub line: u32,
    /// The title (a note's, or a person's name for waiting).
    pub title: String,
    /// The second line: the paragraph, the sentence, the activity.
    pub detail: String,
    /// The notebook, or the person the row is about.
    pub place: String,
    /// When (UTC ms): the end of the writing session, the last change.
    pub at: i64,
    /// For a waiting row: days quiet.
    pub quiet_days: u32,
    /// The row's key (a loop's or a task's), for Done and Not yet.
    pub key: String,
    /// The People page, for a waiting row.
    pub person: String,
    /// Where the caret was in `detail`, in characters, to draw the caret.
    pub caret: bool,
    /// For activity: how many changes or suggestions.
    pub count: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct List {
    pub kind: ListKind,
    /// At most [`crate::overview::density::LIST_CAP`] rows.
    pub rows: Vec<ListRow>,
    /// How many more rows there were.
    pub more: u32,
}

/// The Lines of *Also*.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlsoKind {
    /// A List folded into a Line.
    Folded(ListKind),
    Revisit,
    Captured,
    Promises,
    /// Week: the themes.
    Themes,
    /// Week: meetings without notes.
    MeetingsWithoutNotes,
}

/// One item inside a Line: a note to open.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item {
    pub note_id: String,
    pub line: u32,
    pub text: String,
    /// A voice drop's length in seconds, or 0.
    pub voice_secs: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlsoLine {
    pub kind: AlsoKind,
    /// The count the line leads with (`Inbox 7`, `12 flashcards`).
    pub count: u32,
    /// Flashcards due, for the revisit line.
    pub cards: u32,
    /// At most two items shown inline.
    pub items: Vec<Item>,
    /// Revisit notes with their reasons, when the line is a List.
    pub detail: Vec<String>,
}

/// The one note from the past.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PastNote {
    pub note_id: String,
    pub title: String,
    pub excerpt: String,
    /// `Related to your 11:30 with Tendai`.
    pub why: String,
    pub score: f32,
    /// The seed notes it was chosen for (what "Not useful" dismisses).
    pub seeds: Vec<String>,
}

/// Which page state to draw.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum State {
    /// A library with nothing in it yet.
    New,
    Quiet,
    Busy,
}

/// The Today page.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Today {
    pub greeting: String,
    /// `WEDNESDAY 4 NOVEMBER`.
    pub overline: String,
    pub all_day: Vec<String>,
    pub lede: String,
    pub state: State,
    pub agenda: Agenda,
    /// At most two Lists, in fixed order.
    pub lists: Vec<List>,
    /// At most four Lines.
    pub also: Vec<AlsoLine>,
    pub past: Option<PastNote>,
    /// `Yesterday you wrote 4 notes and finished 6 tasks.`
    pub footer: String,
    /// Wording guesses hidden since the person dismissed two of a kind.
    pub suppressed: Vec<String>,
}

/// A notebook or project the week went into.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectRow {
    pub id: String,
    pub name: String,
    /// `book` or `project`.
    pub kind: String,
    pub notes_changed: u32,
    pub sessions: u32,
    pub last_title: String,
    pub last_note: String,
    pub last_at: i64,
    pub next_task: String,
    pub next_due: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FinishedItem {
    pub note_id: String,
    pub text: String,
    pub at: i64,
    /// `task` or `loop`.
    pub kind: String,
}

/// A day in *Ahead*.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AheadDay {
    /// `YYYY-MM-DD`.
    pub day: String,
    /// The past days of this week are one line.
    pub summary: String,
    pub rows: Vec<AgendaRow>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Week {
    /// The first day (`YYYY-MM-DD`).
    pub from: String,
    pub to: String,
    /// `2 – 8 NOVEMBER`.
    pub range_label: String,
    /// `Week 45`.
    pub week_label: String,
    pub lede: String,
    pub projects: Vec<ProjectRow>,
    pub finished: Vec<FinishedItem>,
    pub finished_more: u32,
    pub waiting: Vec<ListRow>,
    pub still_open: Vec<AgendaRow>,
    pub still_open_total: u32,
    pub ahead: Vec<AheadDay>,
    pub also: Vec<AlsoLine>,
    pub wrote: u32,
    pub changed: u32,
    pub finished_count: u32,
    /// The saved review note's title when this week is already saved.
    pub saved_as: String,
}

/// The whole page as the screens draw it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Overview {
    pub version: u32,
    /// When it was assembled (UTC ms).
    pub at: i64,
    /// Today as `YYYY-MM-DD`.
    pub day: String,
    pub today: Today,
}

/// How long ago `at` was, as the rows say it: `just now`, `12 min ago`,
/// `2 h ago`, `Yesterday`, `3 d ago`, `Mon 26 Oct` past a week.
pub fn ago(at: i64, now: i64, offset_ms: i64) -> String {
    const MIN: i64 = 60_000;
    const HOUR: i64 = 60 * MIN;
    const DAY: i64 = 24 * HOUR;
    let delta = now - at;
    if delta < MIN {
        return "just now".into();
    }
    if delta < HOUR {
        return format!("{} min ago", delta / MIN);
    }
    let today = (now + offset_ms).div_euclid(DAY);
    let then = (at + offset_ms).div_euclid(DAY);
    match today - then {
        0 => format!("{} h ago", delta / HOUR),
        1 => "Yesterday".into(),
        2..=6 => format!("{} d ago", today - then),
        _ => {
            let d = kanso_core::dates::Date::from_days(then);
            kanso_core::dates::format(
                &kanso_core::dates::DateTime::at_midnight(d),
                "%e %b",
            )
        }
    }
}

/// `09:05` from minutes into the day.
pub fn clock(min: u32) -> String {
    format!("{:02}:{:02}", (min / 60) % 24, min % 60)
}

/// `2d late`, `1d late`.
pub fn late_label(days: u32) -> String {
    format!("{days}d late")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ago_reads_like_a_person() {
        let h = 3_600_000;
        let base = 100 * 86_400_000 + 12 * h;
        assert_eq!(ago(base - 10_000, base, 0), "just now");
        assert_eq!(ago(base - 12 * 60_000, base, 0), "12 min ago");
        assert_eq!(ago(base - 2 * h, base, 0), "2 h ago");
        assert_eq!(ago(base - 20 * h, base, 0), "Yesterday");
        assert_eq!(ago(base - 3 * 86_400_000, base, 0), "3 d ago");
        assert_eq!(clock(11 * 60 + 5), "11:05");
        assert_eq!(late_label(2), "2d late");
    }
}
