//! Reminders for tasks with a due time.

use kanso_mobile::{KansoError, ReminderAnswer, ReminderPlan, TaskPlace};

use super::library::KansoLibrary;

impl KansoLibrary {
    pub fn reminders_plan(
        &self,
        now_utc_ms: i64,
        offset_ms: i64,
        can_notify: bool,
    ) -> Result<ReminderPlan, KansoError> {
        self.0.reminders_plan(now_utc_ms, offset_ms, can_notify)
    }

    pub fn reminders_forget(&self, keys: Vec<String>, now_utc_ms: i64) -> Result<(), KansoError> {
        self.0.reminders_forget(keys, now_utc_ms)
    }

    pub fn reminders_forget_future(&self, now_utc_ms: i64) -> Result<(), KansoError> {
        self.0.reminders_forget_future(now_utc_ms)
    }

    pub fn reminder_answer(
        &self,
        key: String,
        answer: ReminderAnswer,
        now_utc_ms: i64,
    ) -> Result<Option<TaskPlace>, KansoError> {
        self.0.reminder_answer(&key, answer, now_utc_ms)
    }
}
