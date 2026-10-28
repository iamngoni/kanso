//! The Journal and templates: today's note, stepping between days, the
//! month calendar, the daily template and notes from templates.
//!
//! What to make and open is decided by `kanso_engine::daily`, and the month
//! is laid out by `kanso_engine::calendar`, which the desktop calls too. This
//! is the facade's side: the day the app means comes in as a [`LocalTime`],
//! cursor offsets go out in UTF-16 units.

use std::collections::HashSet;

use kanso_core::dates::{Date, DateTime};
use kanso_core::roles::daily_date;
use kanso_engine::text::byte_to_utf16;
use kanso_engine::{calendar, daily};
use kanso_store::Origin;

use crate::boundary::guard;
use crate::errors::KansoError;
use crate::library::Library;
use crate::model::{CalendarDay, DayDate, LocalTime, MadeNote, MonthGrid, TemplateRow};

/// The years a calendar day may be in.
const YEARS: std::ops::RangeInclusive<i32> = 1..=9999;

pub(crate) fn date_of(day: DayDate) -> Result<Date, KansoError> {
    if !YEARS.contains(&day.year) {
        return Err(KansoError::invalid("That year can't be shown."));
    }
    Date::new(day.year, day.month, day.day)
        .ok_or_else(|| KansoError::invalid("That isn't a day of the calendar."))
}

pub(crate) fn day_of(date: Date) -> DayDate {
    DayDate {
        year: date.year,
        month: date.month,
        day: date.day,
    }
}

pub(crate) fn time_of(time: LocalTime) -> Result<DateTime, KansoError> {
    let date = date_of(DayDate {
        year: time.year,
        month: time.month,
        day: time.day,
    })?;
    if time.hour > 23 || time.minute > 59 || time.second > 59 {
        return Err(KansoError::invalid("That isn't a time of day."));
    }
    Ok(DateTime {
        date,
        hour: time.hour,
        minute: time.minute,
        second: time.second,
    })
}

pub(crate) fn local_time_of(time: DateTime) -> LocalTime {
    LocalTime {
        year: time.date.year,
        month: time.date.month,
        day: time.date.day,
        hour: time.hour,
        minute: time.minute,
        second: time.second,
    }
}

impl Library {
    /// Today's note at the phone's local time `now`: found, or made from
    /// the daily template.
    pub fn journal_today(&self, now: LocalTime) -> Result<MadeNote, KansoError> {
        guard(|| {
            let now = time_of(now)?;
            self.made(daily::today(&self.inner.store, now)?)
        })
    }

    /// The day note to open when stepping from `open_note` (a daily note, or
    /// any other, which steps from today) to the day before or after that
    /// has a note, or to today, whichever is nearer. `None` when there is no
    /// such day. The day's note is made if it is today's.
    pub fn journal_step(
        &self,
        open_note: Option<String>,
        now: LocalTime,
        forward: bool,
    ) -> Result<Option<MadeNote>, KansoError> {
        guard(|| {
            let now = time_of(now)?;
            let Some(day) = daily::step(&self.inner.store, open_note.as_deref(), now.date, forward)
            else {
                return Ok(None);
            };
            self.open_day(day, now).map(Some)
        })
    }

    /// `day`'s note, made (from the daily template) when it doesn't exist
    /// yet. `now` is the phone's local time, for today and for the
    /// template's `{{time}}`.
    pub fn journal_open_day(&self, day: DayDate, now: LocalTime) -> Result<MadeNote, KansoError> {
        guard(|| self.open_day(date_of(day)?, time_of(now)?))
    }

    /// The month `year`-`month` in whole weeks starting on `week_start` (0
    /// for Monday through 6 for Sunday), each day with whether it has a
    /// daily note outside the trash and whether it is `today`.
    pub fn journal_month(
        &self,
        year: i32,
        month: u32,
        week_start: u32,
        today: DayDate,
    ) -> Result<MonthGrid, KansoError> {
        guard(|| {
            if !YEARS.contains(&year) || !(1..=12).contains(&month) || week_start > 6 {
                return Err(KansoError::invalid("That month can't be shown."));
            }
            let today = date_of(today)?;
            let with_notes: HashSet<Date> = self.inner.store.daily_dates()?.into_iter().collect();
            let start = week_start as usize;
            let grid = calendar::MonthGrid::new(year, month, start);
            Ok(MonthGrid {
                year: grid.year,
                month: grid.month,
                title: grid.title(),
                weekday_labels: calendar::weekday_labels(start)
                    .iter()
                    .map(|label| (*label).to_owned())
                    .collect(),
                weeks: grid
                    .weeks
                    .iter()
                    .map(|week| {
                        week.iter()
                            .map(|&date| CalendarDay {
                                date: day_of(date),
                                in_month: grid.in_month(date),
                                has_note: with_notes.contains(&date),
                                is_today: date == today,
                            })
                            .collect()
                    })
                    .collect(),
            })
        })
    }

    /// The day a daily note's ID names, when it is one.
    pub fn daily_date_of(note_id: &str) -> Option<DayDate> {
        daily_date(note_id).map(day_of)
    }

    /// The month calendar's first column for a platform calendar's first
    /// weekday (Foundation's and Java's count: 1 for Sunday through 7 for
    /// Saturday): 0 for Monday through 6 for Sunday. Anything else is
    /// Monday.
    pub fn journal_week_start(first_weekday: u32) -> u32 {
        calendar::from_first_weekday(first_weekday as usize) as u32
    }

    /// The notes in the Templates notebook, by title.
    pub fn templates(&self) -> Result<Vec<TemplateRow>, KansoError> {
        guard(|| {
            Ok(self
                .inner
                .store
                .templates()?
                .into_iter()
                .map(|(id, title)| TemplateRow { id, title })
                .collect())
        })
    }

    /// A note from template `template_id` with its variables filled in for
    /// `now`, in `notebook_id` (the notebook the list shows; the Inbox when
    /// that is the Templates notebook, or none).
    pub fn new_from_template(
        &self,
        template_id: &str,
        notebook_id: Option<String>,
        now: LocalTime,
    ) -> Result<MadeNote, KansoError> {
        guard(|| {
            let now = time_of(now)?;
            self.made(daily::from_template(
                &self.inner.store,
                template_id,
                notebook_id,
                now,
            )?)
        })
    }

    /// An empty note in the Templates notebook (made if needed), to write a
    /// template in.
    pub fn new_template(&self) -> Result<MadeNote, KansoError> {
        guard(|| {
            Ok(MadeNote {
                id: daily::new_template(&self.inner.store)?,
                cursor16: None,
                title_first: true,
            })
        })
    }

    /// The template new daily notes are made from, when it is still there.
    pub fn daily_template(&self) -> Result<Option<String>, KansoError> {
        guard(|| Ok(self.inner.store.daily_template()?))
    }

    /// Makes `template_id` (a note's ID; `None` for none) the template new
    /// daily notes are made from.
    pub fn set_daily_template(&self, template_id: Option<String>) -> Result<(), KansoError> {
        guard(|| {
            self.inner
                .store
                .set_daily_template(template_id.as_deref(), Origin::Ui)?;
            Ok(())
        })
    }

    /// Opens `day`'s note as the desktop's `open_day` does: today's through
    /// `today`, another day's by making it for that day at `now`'s time.
    fn open_day(&self, day: Date, now: DateTime) -> Result<MadeNote, KansoError> {
        let store = &self.inner.store;
        if day == now.date {
            return self.made(daily::today(store, now)?);
        }
        let made = store.open_daily(DateTime { date: day, ..now }, Origin::Ui)?;
        self.made(daily::Made {
            id: made.id,
            cursor: made.cursor,
            title_first: false,
        })
    }

    /// `made` for the app: the cursor in UTF-16 units of the note's text.
    fn made(&self, made: daily::Made) -> Result<MadeNote, KansoError> {
        let cursor16 = match made.cursor {
            Some(cursor) => {
                let body = self.inner.store.note(&made.id)?.map(|note| note.body);
                body.map(|body| byte_to_utf16(&body, cursor) as u32)
            }
            None => None,
        };
        Ok(MadeNote {
            id: made.id,
            cursor16,
            title_first: made.title_first,
        })
    }
}
