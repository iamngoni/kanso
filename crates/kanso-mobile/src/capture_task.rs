//! Quick Capture's Task mode on the phone: a task typed into the capture
//! sheet, written as a task line into today's daily note or the Inbox's
//! task note by the engine's [`kanso_engine::capture_task`], the code the
//! desktop's Quick Capture runs.
//!
//! [`Library::capture_task_hint`] is the line under the field (`Fri 13 Nov,
//! 15:00 · every week`), [`Library::capture_task`] saves. Nothing is written
//! into a locked notebook or a locked note.

use kanso_core::due_phrase_model::DateOrder;
use kanso_engine::capture::CaptureError;
use kanso_engine::capture_task::{self, TaskTarget};
use kanso_store::Origin;

use crate::boundary::guard;
use crate::capture_task_model::{CaptureTarget, TaskCaptured};
use crate::errors::KansoError;
use crate::journal::time_of;
use crate::library::Library;
use crate::model::LocalTime;

fn order(month_first: bool) -> DateOrder {
    if month_first {
        DateOrder::MonthDay
    } else {
        DateOrder::DayMonth
    }
}

fn target_of(target: CaptureTarget) -> TaskTarget {
    match target {
        CaptureTarget::DailyNote => TaskTarget::DailyNote,
        CaptureTarget::Inbox => TaskTarget::Inbox,
    }
}

impl Library {
    /// How the date and repeat at the end of `text` read in Task mode, such
    /// as `Fri 13 Nov, 15:00 · every week`, at the phone's local time
    /// `now`; `month_first` reads `11/14` as November 14th. `None` when
    /// there is none to show.
    pub fn capture_task_hint(
        text: &str,
        now: LocalTime,
        month_first: bool,
    ) -> Result<Option<String>, KansoError> {
        guard(|| {
            Ok(capture_task::task_hint(
                text,
                time_of(now)?,
                order(month_first),
            ))
        })
    }

    /// Saves `text` as one task at `target` and returns where it went and
    /// the line written. `now` is the phone's local date and time, for
    /// today's note and for what a phrase means. Text with no task in it is
    /// [`KansoError::Invalid`]; a locked Journal, Inbox or note is
    /// [`KansoError::Locked`], and nothing is written.
    pub fn capture_task(
        &self,
        text: &str,
        target: CaptureTarget,
        now: LocalTime,
        month_first: bool,
    ) -> Result<TaskCaptured, KansoError> {
        guard(|| {
            let now = time_of(now)?;
            let order = order(month_first);
            let Some(line) = capture_task::task_line(text, now, order) else {
                return Err(KansoError::invalid("There is no task in that text."));
            };
            let note_id = match capture_task::save_task(
                &self.inner.store,
                text,
                target_of(target),
                now,
                order,
                Origin::Ui,
            ) {
                Ok(Some(id)) => id,
                Ok(None) => return Err(KansoError::invalid("There is no task in that text.")),
                Err(CaptureError::Locked) => return Err(KansoError::Locked),
                Err(CaptureError::Store(err)) => return Err(err.into()),
                Err(other) => {
                    return Err(KansoError::Internal {
                        message: other.to_string(),
                    });
                }
            };
            self.inner
                .metrics()
                .count(crate::metrics::Feature::QuickCapture);
            Ok(TaskCaptured { note_id, line })
        })
    }
}
