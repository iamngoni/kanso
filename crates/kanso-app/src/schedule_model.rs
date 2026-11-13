//! The Schedule view without a window (`schedule_view.rs` draws it): which
//! days it shows, what the filters ask the engine for, how the items sit on
//! their days, and what one change takes back. The days, items and filters
//! are plain values, so a property test can drive them.

use std::collections::BTreeMap;
use std::ops::Range;

use kanso_core::dates::{Date, MONTHS};
use kanso_engine::calendar::{MonthGrid, add_months};
use kanso_engine::schedule::{ScheduleItem, ScheduleKind, ScheduleQuery};
use kanso_store::{PropertyEdit, TickUndo};

/// How many days the view lays out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// The month's weeks, with the days of the neighbours that fill them.
    Month,
    /// The seven days of the focused day's week.
    Week,
}

/// The first day of `day`'s week, where weeks start on `week_start` (0 for
/// Monday through 6 for Sunday).
pub fn week_first(day: Date, week_start: usize) -> Date {
    let back = (day.weekday() + 7 - week_start % 7) % 7;
    day.add_days(-(back as i64))
}

/// The seven days of `day`'s week.
pub fn week_of(day: Date, week_start: usize) -> [Date; 7] {
    let first = week_first(day, week_start);
    std::array::from_fn(|ix| first.add_days(ix as i64))
}

/// The days the view lays out around `focused`: from the first to the one
/// before the last, as the engine's query counts them.
pub fn range_of(mode: Mode, focused: Date, week_start: usize) -> Range<Date> {
    match mode {
        Mode::Week => {
            let first = week_first(focused, week_start);
            first..first.add_days(7)
        }
        Mode::Month => {
            let grid = MonthGrid::new(focused.year, focused.month, week_start);
            let first = grid.weeks.first().map_or(focused, |week| week[0]);
            let last = grid.weeks.last().map_or(focused, |week| week[6]);
            first..last.add_days(1)
        }
    }
}

/// The day a step of `delta` periods from `focused`: a month (the same day
/// of it, or its last) or a week.
pub fn step_period(mode: Mode, focused: Date, delta: i32) -> Date {
    match mode {
        Mode::Month => add_months(focused, delta),
        Mode::Week => focused.add_days(7 * i64::from(delta)),
    }
}

/// What the header says the view shows: "November 2026", or the week as
/// "9 – 15 Nov 2026" (with each month when it crosses one).
pub fn title(mode: Mode, focused: Date, week_start: usize) -> String {
    match mode {
        Mode::Month => MonthGrid::new(focused.year, focused.month, week_start).title(),
        Mode::Week => {
            let days = week_of(focused, week_start);
            let (first, last) = (days[0], days[6]);
            let short = |month: u32| &MONTHS[(month as usize).clamp(1, 12) - 1][..3];
            if first.year != last.year {
                format!(
                    "{} {} {} – {} {} {}",
                    first.day,
                    short(first.month),
                    first.year,
                    last.day,
                    short(last.month),
                    last.year
                )
            } else if first.month != last.month {
                format!(
                    "{} {} – {} {} {}",
                    first.day,
                    short(first.month),
                    last.day,
                    short(last.month),
                    last.year
                )
            } else {
                format!(
                    "{} – {} {} {}",
                    first.day,
                    last.day,
                    short(last.month),
                    last.year
                )
            }
        }
    }
}

/// What the filters keep: the notebooks, the date properties, and whether
/// done tasks show. Empty lists keep everything. [`Filters::query`] is
/// what the engine is asked, so the grid shows what `schedule()` returns.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Filters {
    /// Only these notebooks' tasks and notes; every notebook when empty.
    pub books: Vec<String>,
    /// Only these date properties place a note; every date property when
    /// empty.
    pub keys: Vec<String>,
    /// Done tasks are hidden.
    pub hide_done: bool,
}

impl Filters {
    /// The engine's query for `range`.
    pub fn query(&self, range: Range<Date>) -> ScheduleQuery {
        let mut query = ScheduleQuery::new(range);
        query.books = (!self.books.is_empty()).then(|| self.books.clone());
        query.date_keys = (!self.keys.is_empty()).then(|| self.keys.clone());
        query.include_done = !self.hide_done;
        query
    }

    /// Whether any filter narrows the view.
    pub fn is_refined(&self) -> bool {
        !self.books.is_empty() || !self.keys.is_empty() || self.hide_done
    }

    /// Adds notebook `id`, or takes it away when it is there.
    pub fn toggle_book(&mut self, id: &str) {
        toggle(&mut self.books, id);
    }

    /// Adds date property `key`, or takes it away when it is there.
    pub fn toggle_key(&mut self, key: &str) {
        toggle(&mut self.keys, key);
    }
}

fn toggle(list: &mut Vec<String>, value: &str) {
    match list.iter().position(|item| item == value) {
        Some(at) => {
            list.remove(at);
        }
        None => list.push(value.to_owned()),
    }
}

/// The items by their day, each day's in the order the engine gave them.
/// `items` as indexes into the list, so a view keeps one copy.
pub fn by_day(items: &[ScheduleItem]) -> BTreeMap<Date, Vec<usize>> {
    let mut days: BTreeMap<Date, Vec<usize>> = BTreeMap::new();
    for (ix, item) in items.iter().enumerate() {
        days.entry(item.date).or_default().push(ix);
    }
    days
}

/// The small mark before a note's title: ▶ for what publishes, ⏸ for an
/// embargo, ◆ for any other date property. A task has its own box.
pub fn note_mark(property: Option<&str>) -> &'static str {
    match property {
        Some("publish") => "▶",
        Some("embargo") => "⏸",
        _ => "◆",
    }
}

/// An item's time as the grid shows it: `15:00`.
pub fn time_label(item: &ScheduleItem) -> Option<String> {
    item.time
        .map(|(hour, minute)| format!("{hour:02}:{minute:02}"))
}

/// What a task's words show on the grid: the words without their date and
/// repeat, as the Tasks view shows them.
pub fn task_title(item: &ScheduleItem) -> String {
    kanso_engine::tasks::display_text(&item.text)
}

/// Whether an item can be dragged to another day: a note, or the task's own
/// date. A projected occurrence is only the repeat coming round, and has no
/// date of its own to change.
pub fn can_drag(item: &ScheduleItem) -> bool {
    item.kind == ScheduleKind::Note || !item.projected
}

/// What the keyboard's selection is on after the focused day's items
/// change: the same place, or the last one, or none.
pub fn keep_selection(selected: Option<usize>, count: usize) -> Option<usize> {
    match (selected, count) {
        (_, 0) => None,
        (Some(at), count) => Some(at.min(count - 1)),
        (None, _) => None,
    }
}

/// Moves a selection by `delta` among `count` items, starting on the first
/// (or last) when there is none. Stops at the ends.
pub fn step_selection(selected: Option<usize>, count: usize, delta: isize) -> Option<usize> {
    if count == 0 {
        return None;
    }
    Some(match selected {
        None if delta < 0 => count - 1,
        None => 0,
        Some(at) => (at as isize + delta).clamp(0, count as isize - 1) as usize,
    })
}

/// What puts back the Schedule's last change.
#[derive(Clone, Debug)]
pub enum Undo {
    /// A date property or a task's due date, moved.
    Moved(PropertyEdit),
    /// A task ticked or cleared. A repeating task's tick wrote its next one,
    /// and `undo` takes both back.
    Ticked {
        note_id: String,
        line: u32,
        text: String,
        checked: bool,
        undo: Option<TickUndo>,
    },
}

#[cfg(test)]
#[path = "schedule_model_tests.rs"]
mod tests;
