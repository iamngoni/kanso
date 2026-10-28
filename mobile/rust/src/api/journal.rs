//! The Journal and templates: today's note, any day's, stepping between
//! days, the month calendar, the daily template and notes from templates.

use flutter_rust_bridge::frb;
use kanso_mobile::{DayDate, KansoError, Library, LocalTime, MadeNote, MonthGrid, TemplateRow};

use super::library::KansoLibrary;

impl KansoLibrary {
    pub fn journal_today(&self, now: LocalTime) -> Result<MadeNote, KansoError> {
        self.0.journal_today(now)
    }

    pub fn journal_step(
        &self,
        open_note: Option<String>,
        now: LocalTime,
        forward: bool,
    ) -> Result<Option<MadeNote>, KansoError> {
        self.0.journal_step(open_note, now, forward)
    }

    pub fn journal_open_day(&self, day: DayDate, now: LocalTime) -> Result<MadeNote, KansoError> {
        self.0.journal_open_day(day, now)
    }

    pub fn journal_month(
        &self,
        year: i32,
        month: u32,
        week_start: u32,
        today: DayDate,
    ) -> Result<MonthGrid, KansoError> {
        self.0.journal_month(year, month, week_start, today)
    }

    #[frb(sync)]
    pub fn daily_date_of(note_id: String) -> Option<DayDate> {
        Library::daily_date_of(&note_id)
    }

    #[frb(sync)]
    pub fn journal_week_start(first_weekday: u32) -> u32 {
        Library::journal_week_start(first_weekday)
    }

    pub fn templates(&self) -> Result<Vec<TemplateRow>, KansoError> {
        self.0.templates()
    }

    pub fn new_from_template(
        &self,
        template_id: String,
        book_id: Option<String>,
        now: LocalTime,
    ) -> Result<MadeNote, KansoError> {
        self.0.new_from_template(&template_id, book_id, now)
    }

    pub fn new_template(&self) -> Result<MadeNote, KansoError> {
        self.0.new_template()
    }

    pub fn daily_template(&self) -> Result<Option<String>, KansoError> {
        self.0.daily_template()
    }

    pub fn set_daily_template(&self, template_id: Option<String>) -> Result<(), KansoError> {
        self.0.set_daily_template(template_id)
    }
}
