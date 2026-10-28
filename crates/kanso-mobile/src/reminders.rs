//! Reminders: what to show now and what to have the system show later, for
//! the tasks with a due time, and what an answer to one does.
//!
//! The rules are `kanso_engine::tasks::reminders`, which the desktop runs
//! too: what is due, what is still to come, what a notification says, what
//! this device has shown (kept in the library, locally) and what answering
//! writes to the note. A phone's notifications are the system's, so the plan
//! goes out ahead of time: `schedule` is the next 60 reminders in the next 30
//! days, and the app asks for it again on every library change.
//!
//! A reminder carries what the desktop's notification shows and nothing of
//! the note: the task's words, when it is due and the note's title. Locked
//! notes, notes in locked notebooks and the trash have none.

use std::collections::{BTreeMap, HashMap};
use std::time::Duration;

use kanso_core::dates::{Date, DateTime};
use kanso_core::tasks::Due;
use kanso_engine::tasks::reminders::{self, Answer, SHOWN_META, Shown, parse_shown};
use kanso_store::ReminderTask;

use crate::boundary::guard;
use crate::errors::KansoError;
use crate::journal::local_time_of;
use crate::library::Library;
use crate::model::{ReminderAnswer, ReminderNote, ReminderPlan, TaskPlace};

/// How far ahead the system is given reminders.
const HORIZON: Duration = Duration::from_secs(30 * 24 * 3600);
/// The most reminders handed to the system: iOS keeps 64 pending
/// notifications per app, and a few are left for the app's other needs.
pub const SCHEDULE_LIMIT: usize = 60;

impl Library {
    /// The reminders at `now_utc_ms`, in a phone whose local time is that
    /// plus `offset_ms`. `show_now` are due (at or before now, no more than
    /// 30 minutes ago, not answered on any device, not shown here); `schedule`
    /// are the next ones to fall due, for the system to show at their times
    /// in place of any it was given before.
    ///
    /// With `can_notify` false (reminders are off in Settings, or the system
    /// said no) the plan is empty and nothing is recorded. Otherwise the
    /// reminders in both lists are recorded as shown here, `schedule`'s at
    /// their times, so one the system showed isn't shown again when the app
    /// next opens.
    pub fn reminders_plan(
        &self,
        now_utc_ms: i64,
        offset_ms: i64,
        can_notify: bool,
    ) -> Result<ReminderPlan, KansoError> {
        guard(|| {
            let empty = ReminderPlan {
                show_now: Vec::new(),
                schedule: Vec::new(),
            };
            if !can_notify {
                return Ok(empty);
            }
            let store = &self.inner.store;
            let now_local = now_utc_ms.saturating_add(offset_ms);
            let today = DateTime::from_local_ms(now_local).date;
            let tasks = self.reminder_tasks()?;
            let kept = store.meta(SHOWN_META)?;
            let mut shown = parse_shown(kept.as_deref());

            let due = reminders::due(&tasks, &shown, now_local);
            // A reminder recorded for a time still to come is scheduled, not
            // shown: it is still to come.
            let been_shown: BTreeMap<String, i64> = shown
                .iter()
                .filter(|(_, at)| **at <= now_utc_ms)
                .map(|(key, at)| (key.clone(), *at))
                .collect();
            let coming =
                reminders::upcoming(&tasks, &been_shown, now_local, HORIZON, SCHEDULE_LIMIT);

            let show_now: Vec<ReminderNote> = due
                .iter()
                .map(|task| note_of(task, today, fire_time(task)))
                .collect();
            let schedule: Vec<ReminderNote> = coming
                .iter()
                .map(|coming| {
                    let at = DateTime::from_local_ms(coming.at_local_ms);
                    note_of(coming.task, at.date, at)
                })
                .collect();

            if !show_now.is_empty() || !schedule.is_empty() {
                for coming in &coming {
                    shown.insert(
                        coming.task.key.clone(),
                        coming.at_local_ms.saturating_sub(offset_ms),
                    );
                }
                // Last: it forgets what is too old to be due again.
                let keys: Vec<String> = show_now.iter().map(|n| n.key.clone()).collect();
                let record = reminders::record(&mut shown, &keys, now_utc_ms);
                if kept.as_deref() != Some(record.as_str()) {
                    store.set_meta(SHOWN_META, &record)?;
                }
            }
            Ok(ReminderPlan { show_now, schedule })
        })
    }

    /// Forgets plan entries the platform failed to show or schedule, so the
    /// next plan can retry them. Successfully shown reminders remain recorded.
    pub fn reminders_forget(&self, keys: Vec<String>, now_utc_ms: i64) -> Result<(), KansoError> {
        guard(|| {
            let store = &self.inner.store;
            let kept = store.meta(SHOWN_META)?;
            let mut shown = parse_shown(kept.as_deref());
            for key in keys {
                shown.remove(&key);
            }
            let record = reminders::record(&mut shown, &[], now_utc_ms);
            if kept.as_deref() != Some(record.as_str()) {
                store.set_meta(SHOWN_META, &record)?;
            }
            Ok(())
        })
    }

    /// Clears planned future reminders after notifications are disabled or
    /// permission is revoked. Past shown reminders and synced answers stay.
    pub fn reminders_forget_future(&self, now_utc_ms: i64) -> Result<(), KansoError> {
        guard(|| {
            let store = &self.inner.store;
            let kept = store.meta(SHOWN_META)?;
            let mut shown = parse_shown(kept.as_deref());
            shown.retain(|_, at| *at <= now_utc_ms);
            let record = reminders::record(&mut shown, &[], now_utc_ms);
            if kept.as_deref() != Some(record.as_str()) {
                store.set_meta(SHOWN_META, &record)?;
            }
            Ok(())
        })
    }

    /// Records the answer to reminder `key`: the note remembers it was
    /// answered, so no device reminds again, and Mark Done ticks the task
    /// off. For `Open` it says where to go. `None` when the reminder is gone
    /// (the task was edited, done or locked meanwhile): nothing is written.
    pub fn reminder_answer(
        &self,
        key: &str,
        answer: ReminderAnswer,
        now_utc_ms: i64,
    ) -> Result<Option<TaskPlace>, KansoError> {
        guard(|| {
            let tasks = self.reminder_tasks()?;
            let Some(task) = tasks.iter().find(|task| task.key == key) else {
                return Ok(None);
            };
            let engine = match answer {
                ReminderAnswer::Open => Answer::Open,
                ReminderAnswer::MarkDone => Answer::MarkDone,
                ReminderAnswer::Dismiss => Answer::Dismiss,
            };
            reminders::acknowledge(
                &self.inner.store,
                &Shown::new(task, now_utc_ms),
                key,
                engine,
                now_utc_ms,
            );
            Ok((engine == Answer::Open).then(|| TaskPlace {
                note_id: task.note_id.clone(),
                line: task.line,
            }))
        })
    }

    /// The open tasks with a time that can remind: not those of locked
    /// notes, whether sealed or in a locked notebook that hasn't sealed them
    /// yet.
    pub(crate) fn reminder_tasks(&self) -> Result<Vec<ReminderTask>, KansoError> {
        let store = &self.inner.store;
        let locked_books = store.locked_books()?;
        let mut open: HashMap<String, bool> = HashMap::new();
        let mut tasks = store.reminder_tasks()?;
        let mut failed = None;
        tasks.retain(|task| {
            *open
                .entry(task.note_id.clone())
                .or_insert_with(|| match store.note(&task.note_id) {
                    Ok(Some(note)) => !locked_books.contains(&note.book_id),
                    Ok(None) => false,
                    Err(err) => {
                        failed = Some(err);
                        false
                    }
                })
        });
        match failed {
            Some(err) => Err(err.into()),
            None => Ok(tasks),
        }
    }
}

/// When a task falls due, in local time.
fn fire_time(task: &ReminderTask) -> DateTime {
    Due::parse(&task.due)
        .map(|due| DateTime::from_local_ms(due.local_ms()))
        .unwrap_or_else(|| DateTime::at_midnight(Date::from_days(0)))
}

/// The reminder as the app is given it, its words worked out for `day`.
fn note_of(task: &ReminderTask, day: Date, at: DateTime) -> ReminderNote {
    let said = reminders::reminder(task, day);
    ReminderNote {
        key: said.tag,
        note_id: task.note_id.clone(),
        line: task.line,
        title: said.title,
        body: said.body,
        fire_at: local_time_of(at),
    }
}
