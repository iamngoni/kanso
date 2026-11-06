//! Overview: Today and This week, and the small decisions on its rows.

use kanso_mobile::{
    AiSummary, KansoError, OverviewToday, OverviewWeek, RevisitChoice, RevisitOption,
};

use super::ai::{KansoAi, KansoAiRun};
use super::library::KansoLibrary;
use flutter_rust_bridge::frb;

impl KansoLibrary {
    /// Stores selected system-calendar events locally for Overview.
    pub fn overview_put_calendar_events(&self, events_json: String) -> Result<(), KansoError> {
        self.0.overview_put_calendar_events(&events_json)
    }

    /// Creates a Journal note linked to the single event in `event_json`.
    pub fn overview_create_meeting_note(
        &self,
        event_json: String,
        context: String,
    ) -> Result<String, KansoError> {
        self.0.overview_create_meeting_note(&event_json, &context)
    }

    pub fn overview_link_meeting_note(
        &self,
        note_id: String,
        event_json: String,
    ) -> Result<(), KansoError> {
        self.0.overview_link_meeting_note(&note_id, &event_json)
    }

    /// Today, worked out now. Kept as the next launch's first frame.
    pub fn overview_today(&self) -> Result<OverviewToday, KansoError> {
        self.0.overview_today()
    }

    /// The last Today kept, when it is today's: the first frame.
    #[frb(sync)]
    pub fn overview_snapshot(&self) -> Option<OverviewToday> {
        self.0.overview_snapshot()
    }

    /// The week of `from` (`YYYY-MM-DD`), or this week.
    pub fn overview_week(&self, from: Option<String>) -> Result<OverviewWeek, KansoError> {
        self.0.overview_week(from)
    }

    pub fn overview_hide_place(&self, note_id: String) -> Result<(), KansoError> {
        self.0.overview_hide_place(&note_id)
    }

    pub fn overview_snooze_waiting(&self, key: String) -> Result<(), KansoError> {
        self.0.overview_snooze_waiting(&key)
    }

    pub fn overview_dismiss_past(&self, note_id: String) -> Result<(), KansoError> {
        self.0.overview_dismiss_past(&note_id)
    }

    #[frb(sync)]
    pub fn revisit_options(&self) -> Vec<RevisitOption> {
        self.0.revisit_options()
    }

    /// Sets a note's revisit date; returns the day (`YYYY-MM-DD`).
    pub fn set_revisit(
        &self,
        note_id: String,
        choice: RevisitChoice,
    ) -> Result<String, KansoError> {
        self.0.set_revisit(&note_id, choice)
    }

    pub fn revisit_of(&self, note_id: String) -> Result<Option<String>, KansoError> {
        self.0.revisit_of(&note_id)
    }

    pub fn clear_revisit(&self, note_id: String) -> Result<(), KansoError> {
        self.0.clear_revisit(&note_id)
    }

    /// Typing in a note, for Pick up; the editor calls it every ten seconds
    /// at most.
    pub fn note_writing(
        &self,
        note_id: String,
        line: u32,
        column: u32,
        typed_chars: u32,
        body: String,
    ) -> Result<(), KansoError> {
        self.0
            .note_writing(&note_id, line, column, typed_chars, &body)
    }
}

impl KansoAi {
    /// A short paragraph over Today, or over the week of `from` when `week`.
    pub fn summarise_overview(
        &self,
        from: Option<String>,
        week: bool,
        run: &KansoAiRun,
    ) -> Result<AiSummary, KansoError> {
        self.0.summarise_overview(from, week, &run.0)
    }
}
