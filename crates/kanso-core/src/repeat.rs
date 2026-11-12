//! Repeating tasks: the `every …` words after a due date, and the next
//! date they give.
//!
//! ```text
//! repeat := "every" rule ["after done"]        (or daily, weekly, monthly, yearly)
//! rule   := day | week | weekday | month | year | N days | N weeks | N months | N years
//!         | days                                   every mon and thu
//!         | N weeks "on" days                      every 2 weeks on mon, wed and fri
//!         | month|N months "on the" (15th | last day | 2nd tue | last fri)
//! ```
//!
//! English only. The words are read only right after a due token
//! (`@2026-11-14T15:00 every week`); anything the grammar does not know
//! is not a repeat, so there is no guessing. Dates are local wall-clock
//! dates, so a time of day is kept as written across a daylight-saving
//! change: the zone never enters.

use std::fmt;
use std::ops::Range;

use crate::dates::{Date, days_in_month};
use crate::repeat_model::{Every, MonthDay, Repeat, WeekdaySet};
use crate::tasks::{Due, due_token};

const DAYS: [&str; 7] = ["mon", "tue", "wed", "thu", "fri", "sat", "sun"];
const FULL_DAYS: [&str; 7] = [
    "monday",
    "tuesday",
    "wednesday",
    "thursday",
    "friday",
    "saturday",
    "sunday",
];

impl Repeat {
    /// The due date of the task written when one due `due` is done on
    /// `done_on`. The time of day is kept. The date is always after
    /// `due`'s.
    ///
    /// Counted from the due date, it keeps stepping until the date is
    /// after `done_on`, so an overdue weekly task gives the next future
    /// date, not a run of catch-up ones. With `after_done` it counts from
    /// `done_on` (or from the due date when that is later, for a task
    /// done early).
    ///
    /// A rule that names no day (`every month`, `every year`) keeps the
    /// day it is given, shortened in a short month, so the 31st becomes
    /// the 30th and stays there. `every month on the 31st` does not drift.
    pub fn next(&self, due: &Due, done_on: Date) -> Due {
        let mut date = if self.after_done {
            self.step(done_on.max(due.date))
        } else {
            let mut date = self.step(due.date);
            while date <= done_on {
                date = self.step(date);
            }
            date
        };
        // A rule always moves forward; this is a guard, not a path.
        if date <= due.date {
            date = due.date.add_days(1);
        }
        Due {
            date,
            time: due.time,
        }
    }

    /// The first date the rule gives after `base`.
    fn step(&self, base: Date) -> Date {
        match self.every {
            Every::Days(n) => base.add_days(i64::from(n.max(1))),
            Every::Weeks(n, set) => step_weeks(base, n.max(1), set),
            Every::Months(n, rule) => step_months(base, n.max(1), rule),
            Every::Years(n) => {
                let year = base.year.saturating_add(i32::from(n.max(1)));
                let day = base.day.min(days_in_month(year, base.month));
                Date {
                    year,
                    month: base.month,
                    day,
                }
            }
            Every::Weekdays => {
                let mut date = base.add_days(1);
                while date.weekday() >= 5 {
                    date = date.add_days(1);
                }
                date
            }
        }
    }
}

fn step_weeks(base: Date, n: u16, set: WeekdaySet) -> Date {
    if set.is_empty() {
        return base.add_days(7 * i64::from(n));
    }
    let weekday = base.weekday();
    if let Some(later) = set.days().find(|&d| d > weekday) {
        return base.add_days((later - weekday) as i64);
    }
    // The first listed weekday of the week `n` weeks on.
    let first = set.days().next().unwrap_or(weekday);
    base.add_days(7 * i64::from(n) - weekday as i64 + first as i64)
}

fn step_months(base: Date, n: u16, rule: MonthDay) -> Date {
    let index = i64::from(base.year) * 12 + i64::from(base.month) - 1 + i64::from(n);
    let year = index.div_euclid(12) as i32;
    let month = index.rem_euclid(12) as u32 + 1;
    let length = days_in_month(year, month);
    let day = match rule {
        MonthDay::Same => base.day.min(length),
        MonthDay::Date(d) => u32::from(d).clamp(1, 31).min(length),
        MonthDay::Last => length,
        MonthDay::Nth(nth, weekday) => return nth_weekday(year, month, nth, usize::from(weekday)),
    };
    Date { year, month, day }
}

fn nth_weekday(year: i32, month: u32, nth: i8, weekday: usize) -> Date {
    let length = days_in_month(year, month);
    let first = Date {
        year,
        month,
        day: 1,
    };
    let mut day = 1 + ((weekday % 7 + 7 - first.weekday()) % 7) as u32;
    if nth < 0 {
        while day + 7 <= length {
            day += 7;
        }
    } else {
        day += 7 * (nth.clamp(1, 4) as u32 - 1);
    }
    Date {
        year,
        month,
        day: day.min(length),
    }
}

// MARK: Words

impl fmt::Display for Repeat {
    /// The canonical words, which [`parse_repeat`] reads back.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let unit = |n: u16, one: &str, f: &mut fmt::Formatter<'_>| match n {
            0 | 1 => write!(f, "every {one}"),
            n => write!(f, "every {n} {one}s"),
        };
        match self.every {
            Every::Days(n) => unit(n, "day", f)?,
            Every::Weeks(n, set) if set.is_empty() => unit(n, "week", f)?,
            Every::Weeks(n, set) if n <= 1 => write!(f, "every {}", day_list(set))?,
            Every::Weeks(n, set) => write!(f, "every {n} weeks on {}", day_list(set))?,
            Every::Months(n, rule) => {
                unit(n, "month", f)?;
                match rule {
                    MonthDay::Same => {}
                    MonthDay::Date(d) => write!(f, " on the {}", ordinal(u32::from(d)))?,
                    MonthDay::Last => write!(f, " on the last day")?,
                    MonthDay::Nth(-1, d) => write!(f, " on the last {}", day_name(d))?,
                    MonthDay::Nth(k, d) => write!(
                        f,
                        " on the {} {}",
                        ordinal(k.unsigned_abs().into()),
                        day_name(d)
                    )?,
                }
            }
            Every::Years(n) => unit(n, "year", f)?,
            Every::Weekdays => write!(f, "every weekday")?,
        }
        if self.after_done {
            write!(f, " after done")?;
        }
        Ok(())
    }
}

fn day_name(weekday: u8) -> &'static str {
    DAYS[usize::from(weekday) % 7]
}

fn day_list(set: WeekdaySet) -> String {
    let names: Vec<&str> = set.days().map(|d| DAYS[d]).collect();
    match names.as_slice() {
        [] => String::new(),
        [one] => (*one).to_owned(),
        [head @ .., last] => format!("{} and {last}", head.join(", ")),
    }
}

fn ordinal(n: u32) -> String {
    let suffix = match (n % 10, n % 100) {
        (_, 11..=13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    };
    format!("{n}{suffix}")
}

struct Word<'a> {
    range: Range<usize>,
    text: &'a str,
}

impl Word<'_> {
    fn lower(&self) -> String {
        self.text.to_ascii_lowercase()
    }
}

/// Words split on spaces, tabs and commas (a comma is only a separator).
fn words(text: &str) -> Vec<Word<'_>> {
    let mut out = Vec::new();
    let mut start = None;
    for (at, c) in text.char_indices() {
        let sep = c == ',' || c.is_whitespace();
        match (sep, start) {
            (true, Some(s)) => {
                out.push(Word {
                    range: s..at,
                    text: &text[s..at],
                });
                start = None;
            }
            (false, None) => start = Some(at),
            _ => {}
        }
    }
    if let Some(s) = start {
        out.push(Word {
            range: s..text.len(),
            text: &text[s..],
        });
    }
    out
}

fn weekday_of(word: &str) -> Option<usize> {
    let w = word.to_ascii_lowercase();
    (0..7).find(|&d| {
        w == DAYS[d] || w == FULL_DAYS[d] || (d == 1 && w == "tues") || (d == 3 && w == "thur")
    })
}

fn count(word: &str) -> Option<u16> {
    let n: u16 = word.parse().ok()?;
    (n >= 1).then_some(n)
}

/// `15th` is 15: one of st, nd, rd, th after the digits.
fn ordinal_of(word: &str) -> Option<u32> {
    let w = word.to_ascii_lowercase();
    let digits = w.trim_end_matches(|c: char| c.is_ascii_alphabetic());
    if !matches!(&w[digits.len()..], "st" | "nd" | "rd" | "th") {
        return None;
    }
    digits.parse().ok()
}

/// The unit a word names and whether it is the plural (which needs the
/// number before it).
fn unit(word: &str) -> Option<(char, bool)> {
    match word.to_ascii_lowercase().as_str() {
        "day" => Some(('d', false)),
        "days" => Some(('d', true)),
        "week" => Some(('w', false)),
        "weeks" => Some(('w', true)),
        "month" => Some(('m', false)),
        "months" => Some(('m', true)),
        "year" => Some(('y', false)),
        "years" => Some(('y', true)),
        _ => None,
    }
}

/// Reads a repeat from the start of `text` (`every week`, `weekly`,
/// `every 2 weeks after done`), and how many bytes it takes. It ends at
/// a word boundary; what follows is not its concern.
pub fn parse_repeat(text: &str) -> Option<(Repeat, usize)> {
    let ws = words(text);
    let first = ws.first()?.lower();
    let (every, mut used) = match first.as_str() {
        "daily" => (Every::Days(1), 1),
        "weekly" => (Every::Weeks(1, WeekdaySet::EMPTY), 1),
        "monthly" => (Every::Months(1, MonthDay::Same), 1),
        "yearly" => (Every::Years(1), 1),
        "every" => parse_rule(&ws)?,
        _ => return None,
    };
    let mut after_done = false;
    if ws.get(used).map(Word::lower).as_deref() == Some("after")
        && ws.get(used + 1).map(Word::lower).as_deref() == Some("done")
    {
        after_done = true;
        used += 2;
    }
    Some((Repeat { every, after_done }, ws[used - 1].range.end))
}

/// The rule after `every`, and the words used in all, `every` included.
fn parse_rule(ws: &[Word<'_>]) -> Option<(Every, usize)> {
    let w = |i: usize| ws.get(i).map(Word::lower);
    let second = w(1)?;
    if second == "weekday" || second == "weekdays" {
        return Some((Every::Weekdays, 2));
    }
    // every mon, thu and fri
    if weekday_of(&second).is_some() {
        let (set, used) = parse_days(ws, 1)?;
        return Some((Every::Weeks(1, set), used));
    }
    let (n, unit_at) = match count(&second) {
        Some(n) => (n, 2),
        None => (1, 1),
    };
    let (kind, plural) = unit(&w(unit_at)?)?;
    if plural != (unit_at == 2) {
        return None;
    }
    let mut used = unit_at + 1;
    let every = match kind {
        'd' => Every::Days(n),
        'y' => Every::Years(n),
        'w' => {
            let mut set = WeekdaySet::EMPTY;
            if w(used).as_deref() == Some("on") {
                let (days, end) = parse_days(ws, used + 1)?;
                set = days;
                used = end;
            }
            Every::Weeks(n, set)
        }
        _ => {
            let mut rule = MonthDay::Same;
            if w(used).as_deref() == Some("on") {
                if w(used + 1).as_deref() != Some("the") {
                    return None;
                }
                let (day, end) = parse_month_day(ws, used + 2)?;
                rule = day;
                used = end;
            }
            Every::Months(n, rule)
        }
    };
    Some((every, used))
}

/// A list of weekdays from word `at`: `mon`, `mon and thu`, `mon, wed and fri`.
fn parse_days(ws: &[Word<'_>], at: usize) -> Option<(WeekdaySet, usize)> {
    let mut set = WeekdaySet::EMPTY;
    let mut i = at;
    loop {
        set.insert(weekday_of(ws.get(i)?.text)?);
        i += 1;
        let next = match ws.get(i).map(Word::lower) {
            Some(w) if w == "and" => ws
                .get(i + 1)
                .is_some_and(|d| weekday_of(d.text).is_some())
                .then_some(i + 1),
            Some(w) if weekday_of(&w).is_some() => Some(i),
            _ => None,
        };
        match next {
            Some(next) => i = next,
            None => return Some((set, i)),
        }
    }
}

fn parse_month_day(ws: &[Word<'_>], at: usize) -> Option<(MonthDay, usize)> {
    let first = ws.get(at)?.lower();
    if first == "last" {
        let second = ws.get(at + 1)?.text;
        if second.eq_ignore_ascii_case("day") {
            return Some((MonthDay::Last, at + 2));
        }
        return Some((MonthDay::Nth(-1, weekday_of(second)? as u8), at + 2));
    }
    let n = ordinal_of(&first)?;
    if let Some(weekday) = ws.get(at + 1).and_then(|w| weekday_of(w.text)) {
        return (1..=4)
            .contains(&n)
            .then_some((MonthDay::Nth(n as i8, weekday as u8), at + 2));
    }
    (1..=31)
        .contains(&n)
        .then_some((MonthDay::Date(n as u8), at + 1))
}

// MARK: In a task's text

/// The repeat of a task's text and where it is: the byte range of the
/// `every …` words, which come right after the due token and a space.
/// `None` without a due date, or when the words are not a repeat.
pub fn repeat_token(text: &str) -> Option<(Range<usize>, Repeat)> {
    let due = due_token(text)?;
    let rest = &text[due.end..];
    let trimmed = rest.trim_start_matches([' ', '\t']);
    if trimmed.len() == rest.len() {
        return None;
    }
    let start = due.end + (rest.len() - trimmed.len());
    let (repeat, len) = parse_repeat(trimmed)?;
    Some((start..start + len, repeat))
}

#[cfg(test)]
#[path = "repeat_tests.rs"]
mod tests;
