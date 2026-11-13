//! The Schedule's data types, declared again for the generator, as
//! `types.rs` does for the rest. What the fields mean is said where the
//! facade declares them (`crates/kanso-mobile/src/schedule_view_model.rs`).

use flutter_rust_bridge::frb;
// Public: the generated code names these types by this module's path.
pub use kanso_mobile::{DayDate, ScheduleRequest, ScheduleRow, ScheduleRowKind};

#[frb(mirror(ScheduleRowKind))]
pub enum _ScheduleRowKind {
    Task,
    Note,
}

#[frb(mirror(ScheduleRequest))]
pub struct _ScheduleRequest {
    pub from: DayDate,
    pub until: DayDate,
    pub books: Option<Vec<String>>,
    pub date_keys: Option<Vec<String>>,
    pub include_done: bool,
}

#[frb(mirror(ScheduleRow))]
pub struct _ScheduleRow {
    pub kind: ScheduleRowKind,
    pub note_id: String,
    pub note_title: String,
    pub book_id: String,
    pub line: Option<u32>,
    pub text: String,
    pub date: DayDate,
    pub time: Option<String>,
    pub zone: Option<String>,
    pub repeat: Option<String>,
    pub done: bool,
    pub property: Option<String>,
    pub projected: bool,
}
