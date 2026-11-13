//! The Schedule: tasks and dated notes in a range of days.

use kanso_mobile::{CardMoved, DayDate, KansoError, ScheduleRequest, ScheduleRow};

use super::library::KansoLibrary;

impl KansoLibrary {
    /// What falls on each day of `request`'s range: tasks by due date (a
    /// repeating one on every day it comes round) and notes by their date
    /// properties, by day and time.
    pub fn schedule(&self, request: ScheduleRequest) -> Result<Vec<ScheduleRow>, KansoError> {
        self.0.schedule(request)
    }

    /// Moves a note's date property (`property`) or a task (`line`, `text`)
    /// to day `to`, keeping its time, zone and repeat. For a projected
    /// occurrence the caller passes the real task's line and words. The
    /// change can be taken back with `undo` under the ticket it returns.
    pub fn move_scheduled(
        &self,
        note_id: String,
        line: Option<u32>,
        text: String,
        property: Option<String>,
        to: DayDate,
    ) -> Result<CardMoved, KansoError> {
        self.0.move_scheduled(&note_id, line, &text, property, to)
    }

    /// The date properties the notebooks `books` define (every notebook
    /// when `None`): what the Schedule's property filter offers.
    pub fn schedule_date_keys(
        &self,
        books: Option<Vec<String>>,
    ) -> Result<Vec<String>, KansoError> {
        self.0.schedule_date_keys(books)
    }
}
