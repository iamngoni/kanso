//! Overview's data types, declared again for the generator, as `types.rs`
//! does for the rest. What the fields mean is said where the facade declares
//! them (`crates/kanso-mobile/src/overview_model.rs`).

use flutter_rust_bridge::frb;
// Public: the generated code names these types by this module's path.
pub use kanso_mobile::{
    OverviewAgendaRow, OverviewAheadDay, OverviewAi, OverviewFinished, OverviewItem, OverviewLine,
    OverviewLineKind, OverviewList, OverviewListKind, OverviewListRow, OverviewPast,
    OverviewProject, OverviewRowKind, OverviewState, OverviewToday, OverviewWeek, RevisitChoice,
    RevisitOption,
};

#[frb(mirror(OverviewRowKind))]
pub enum _OverviewRowKind {
    Overdue,
    Event,
    Reminder,
    Due,
}

#[frb(mirror(OverviewListKind))]
pub enum _OverviewListKind {
    PickUp,
    Waiting,
    Activity,
}

#[frb(mirror(OverviewLineKind))]
pub enum _OverviewLineKind {
    Revisit,
    Captured,
    Promises,
    Themes,
    Meetings,
    Activity,
    Waiting,
    PickUp,
}

#[frb(mirror(OverviewState))]
pub enum _OverviewState {
    New,
    Quiet,
    Busy,
}

#[frb(mirror(RevisitChoice))]
pub enum _RevisitChoice {
    Tomorrow,
    InThreeDays,
    NextWeek,
    InTwoWeeks,
    InAMonth,
    InThreeMonths,
}

#[frb(mirror(OverviewAgendaRow))]
pub struct _OverviewAgendaRow {
    pub kind: OverviewRowKind,
    pub key: String,
    pub title: String,
    pub context: String,
    pub gutter: String,
    pub now: bool,
    pub free: bool,
    pub note_id: String,
    pub line: u32,
    pub is_loop: bool,
    pub raw: String,
    pub color: String,
    pub meeting_note_id: String,
}

#[frb(mirror(OverviewListRow))]
pub struct _OverviewListRow {
    pub kind: OverviewListKind,
    pub note_id: String,
    pub line: u32,
    pub title: String,
    pub detail: String,
    pub place: String,
    pub when_label: String,
    pub quiet_days: u32,
    pub warn: bool,
    pub key: String,
    pub person: String,
    pub raw: String,
    pub is_loop: bool,
    pub caret: bool,
    pub count: u32,
    pub agent: bool,
}

#[frb(mirror(OverviewList))]
pub struct _OverviewList {
    pub kind: OverviewListKind,
    pub label: String,
    pub rows: Vec<OverviewListRow>,
    pub more: u32,
}

#[frb(mirror(OverviewItem))]
pub struct _OverviewItem {
    pub note_id: String,
    pub line: u32,
    pub text: String,
    pub voice_secs: u32,
}

#[frb(mirror(OverviewLine))]
pub struct _OverviewLine {
    pub kind: OverviewLineKind,
    pub label: String,
    pub count: u32,
    pub cards: u32,
    pub items: Vec<OverviewItem>,
    pub detail: Vec<String>,
}

#[frb(mirror(OverviewPast))]
pub struct _OverviewPast {
    pub note_id: String,
    pub title: String,
    pub excerpt: String,
    pub why: String,
}

#[frb(mirror(OverviewAi))]
pub struct _OverviewAi {
    pub text: String,
    pub warning: Option<String>,
    pub time_label: String,
}

#[frb(mirror(OverviewToday))]
pub struct _OverviewToday {
    pub day: String,
    pub greeting: String,
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

#[frb(mirror(OverviewProject))]
pub struct _OverviewProject {
    pub id: String,
    pub name: String,
    pub notes_changed: u32,
    pub last_title: String,
    pub last_note_id: String,
    pub next_task: String,
    pub next_due: String,
}

#[frb(mirror(OverviewFinished))]
pub struct _OverviewFinished {
    pub note_id: String,
    pub text: String,
}

#[frb(mirror(OverviewAheadDay))]
pub struct _OverviewAheadDay {
    pub summary: String,
    pub rows: Vec<OverviewAgendaRow>,
}

#[frb(mirror(OverviewWeek))]
pub struct _OverviewWeek {
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
    pub saved_as: String,
    pub ai: Option<OverviewAi>,
}

#[frb(mirror(RevisitOption))]
pub struct _RevisitOption {
    pub choice: RevisitChoice,
    pub label: String,
    pub day_label: String,
}
