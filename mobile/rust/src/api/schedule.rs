//! The Schedule: tasks and dated notes in a range of days.

use kanso_mobile::{KansoError, ScheduleRequest, ScheduleRow};

use super::library::KansoLibrary;

impl KansoLibrary {
    /// What falls on each day of `request`'s range: tasks by due date (a
    /// repeating one on every day it comes round) and notes by their date
    /// properties, by day and time.
    pub fn schedule(&self, request: ScheduleRequest) -> Result<Vec<ScheduleRow>, KansoError> {
        self.0.schedule(request)
    }
}
