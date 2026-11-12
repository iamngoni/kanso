//! Typed dates: `film Pixel review fri 3pm` is a task due on the next Friday
//! at 15:00. English only; anything else is left as text, with no guessing.
//!
//! The parser reads words from the end of the line and keeps going while a
//! word, or a few words together, is a day or a time of day. The first word
//! that is neither ends the phrase. A line whose last word is not a date or
//! time word has no phrase at all, so "call Fri about the fri shoot" is
//! plain prose.
//!
//! What it knows:
//!
//! - days: `today`, `tonight` (a day and 20:00), `tomorrow`, `tmr`, a
//!   weekday (`fri`, `friday`: the next one after today), `this fri` (the
//!   next one from today on), `next fri` (the one in next week, Monday to
//!   Sunday), `in 3 days`, `in 2 weeks`, `14 nov`, `nov 14`, `14th nov`,
//!   `14/11` (in the order the caller gives), `2026-11-14`. A day and month
//!   without a year mean their next turn, today included.
//! - times: `3pm`, `3:30pm`, `3 pm`, `15:00`, `1500 h`, `noon`, `midnight`,
//!   `morning` (09:00), `afternoon` (14:00), `evening` (18:00). A time may
//!   follow `at`.
//! - the canonical token that [`canonical`] writes, `@2026-11-14T15:00`.
//!
//! - a repeat after them: `every week`, `weekly`, `every 2 weeks on mon and
//!   thu`, `every month on the last day`, `every 3 days after done` (the
//!   grammar is in [`crate::repeat`]). It comes last, and needs a day or
//!   time before it.
//!
//! A day, a time, or both, in either order. A time with no day is today,
//! or tomorrow when it has already passed.
//!
//! Pure and local: `now` is the device's local wall-clock time, passed in so
//! a phrase always means the same thing in a test.

use crate::dates::{Date, DateTime, MONTHS};
use crate::due_phrase_model::{DateOrder, Parsed};
use crate::repeat::parse_repeat;
use crate::tasks::Due;

/// Finds the date and time at the end of `text`, with the repeat that
/// follows them (`fri 3pm every week`), or `None` when the text does not
/// end in one.
///
/// A repeat needs a date in front of it: `every week` alone, or after
/// words that are no date, is no phrase, and the parser does not pick a
/// first day for it.
pub fn parse(text: &str, now: DateTime, order: DateOrder) -> Option<Parsed> {
    let ws = words(text);
    let last = ws.last()?;
    // The longest repeat is some sixteen words; look no further back, so
    // a long line costs no more than a short one.
    let from = ws.len().saturating_sub(MAX_REPEAT_WORDS);
    for (ix, word) in ws.iter().enumerate().skip(from) {
        let Some((repeat, len)) = parse_repeat(&text[word.start..]) else {
            continue;
        };
        if word.start + len != last.end {
            continue;
        }
        // The date must end where the repeat starts, or there is no phrase.
        let mut parsed = parse_date(&text[..ws[ix.checked_sub(1)?].end], now, order)?;
        parsed.span.end = last.end;
        parsed.repeat = Some(repeat);
        return Some(parsed);
    }
    parse_date(text, now, order)
}

/// The most words a repeat takes: `every 2 weeks on mon, tue, wed, thu,
/// fri, sat and sun after done`, with room to spare.
const MAX_REPEAT_WORDS: usize = 20;

/// The date and time at the end of `text`, with no repeat.
fn parse_date(text: &str, now: DateTime, order: DateOrder) -> Option<Parsed> {
    let words = words(text);
    let ctx = Ctx { now, order };
    let mut pos = words.len();
    let mut date: Option<Date> = None;
    let mut time: Option<(u32, u32)> = None;
    while pos > 0 {
        let Some((len, unit)) = unit_ending_at(&words, pos, &ctx) else {
            break;
        };
        let fits = match unit {
            Unit::Day(day) if date.is_none() => {
                date = Some(day);
                true
            }
            Unit::Clock(h, m) if time.is_none() => {
                time = Some((h, m));
                // "at 3pm"
                if pos > len && words[pos - len - 1].lower == "at" {
                    pos -= 1;
                }
                true
            }
            Unit::Both(day, h, m) if date.is_none() && time.is_none() => {
                date = Some(day);
                time = Some((h, m));
                true
            }
            _ => false,
        };
        if !fits {
            break;
        }
        pos -= len;
    }
    if pos == words.len() {
        return None;
    }
    let date = match (date, time) {
        (Some(date), _) => date,
        (None, Some((h, m))) => {
            if (h, m) > (now.hour, now.minute) {
                now.date
            } else {
                now.date.add_days(1)
            }
        }
        (None, None) => return None,
    };
    Some(Parsed {
        span: words[pos].start..words[words.len() - 1].end,
        due: Due { date, time },
        repeat: None,
    })
}

/// The task token for a parsed phrase: `@2026-11-14T15:00`, or
/// `@2026-11-14T15:00 every week` with a repeat, the words that
/// [`crate::tasks::TaskItem`] reads back.
pub fn canonical(parsed: &Parsed) -> String {
    match &parsed.repeat {
        Some(repeat) => format!("@{} {repeat}", parsed.due.format()),
        None => format!("@{}", parsed.due.format()),
    }
}

/// How a parsed phrase reads to a person, as the hint under a field shows
/// it: `Fri 14 Nov, 15:00 · every week`. The year follows the month when it
/// is not `today`'s. Pure.
pub fn label(parsed: &Parsed, today: Date) -> String {
    let pattern = if parsed.due.date.year == today.year {
        "%a %e %b"
    } else {
        "%a %e %b %Y"
    };
    let mut out = crate::dates::format(&DateTime::at_midnight(parsed.due.date), pattern);
    if let Some((hour, minute)) = parsed.due.time {
        out.push_str(&format!(", {hour:02}:{minute:02}"));
    }
    if let Some(repeat) = &parsed.repeat {
        out.push_str(&format!(" \u{b7} {repeat}"));
    }
    out
}

struct Ctx {
    now: DateTime,
    order: DateOrder,
}

struct Word {
    start: usize,
    end: usize,
    lower: String,
}

fn words(text: &str) -> Vec<Word> {
    let mut out = Vec::new();
    let mut start: Option<usize> = None;
    for (at, c) in text.char_indices() {
        match (c.is_whitespace(), start) {
            (true, Some(s)) => {
                out.push(word(text, s, at));
                start = None;
            }
            (false, None) => start = Some(at),
            _ => {}
        }
    }
    if let Some(s) = start {
        out.push(word(text, s, text.len()));
    }
    out
}

fn word(text: &str, start: usize, end: usize) -> Word {
    Word {
        start,
        end,
        lower: text[start..end].to_lowercase(),
    }
}

enum Unit {
    Day(Date),
    Clock(u32, u32),
    /// A word that is a day and a time at once ("tonight").
    Both(Date, u32, u32),
}

/// The longest run of words ending just before `end` that is a day or a
/// time, with how many words it took.
fn unit_ending_at(words: &[Word], end: usize, ctx: &Ctx) -> Option<(usize, Unit)> {
    (1..=3).rev().filter(|len| *len <= end).find_map(|len| {
        let run: Vec<&str> = words[end - len..end]
            .iter()
            .map(|w| w.lower.as_str())
            .collect();
        match_unit(&run, ctx).map(|unit| (len, unit))
    })
}

fn match_unit(run: &[&str], ctx: &Ctx) -> Option<Unit> {
    let today = ctx.now.date;
    match *run {
        ["in", n, unit] => {
            let n = i64::from(plain_number(n, 3).filter(|n| *n >= 1)?);
            let days = match unit {
                "day" | "days" => n,
                "week" | "weeks" => n * 7,
                _ => return None,
            };
            Some(Unit::Day(today.add_days(days)))
        }
        [kind @ ("next" | "this"), name] => {
            let wd = weekday_of(name)?;
            let days = if kind == "this" {
                (wd + 7 - today.weekday()) % 7
            } else {
                // Monday of next week, then on to the weekday.
                (7 - today.weekday()) + wd
            };
            Some(Unit::Day(today.add_days(days as i64)))
        }
        [num, meridiem @ ("am" | "pm")] => {
            let (h, m) = clock_parts(num)?;
            twelve_hour(h, m, meridiem == "pm").map(|(h, m)| Unit::Clock(h, m))
        }
        [num, "h"] => hhmm(num).map(|(h, m)| Unit::Clock(h, m)),
        [a, b] => {
            let (month, day) = match (day_number(a), month_of(b)) {
                (Some(day), Some(month)) => (month, day),
                _ => (month_of(a)?, day_number(b)?),
            };
            next_turn(month, day, today).map(Unit::Day)
        }
        [one] => single(one, ctx),
        _ => None,
    }
}

fn single(word: &str, ctx: &Ctx) -> Option<Unit> {
    let today = ctx.now.date;
    match word {
        "today" => return Some(Unit::Day(today)),
        "tomorrow" | "tmr" => return Some(Unit::Day(today.add_days(1))),
        "tonight" => return Some(Unit::Both(today, 20, 0)),
        "noon" => return Some(Unit::Clock(12, 0)),
        "midnight" => return Some(Unit::Clock(0, 0)),
        "morning" => return Some(Unit::Clock(9, 0)),
        "afternoon" => return Some(Unit::Clock(14, 0)),
        "evening" => return Some(Unit::Clock(18, 0)),
        _ => {}
    }
    if let Some(wd) = weekday_of(word) {
        let ahead = (wd + 6 - today.weekday()) % 7 + 1;
        return Some(Unit::Day(today.add_days(ahead as i64)));
    }
    if let Some(unit) = clock_word(word) {
        return Some(unit);
    }
    numeric_date(word, ctx)
}

/// `3pm`, `3:30pm`, `15:00`, `1500h`.
fn clock_word(word: &str) -> Option<Unit> {
    for (suffix, pm) in [("am", false), ("pm", true)] {
        if let Some(num) = word.strip_suffix(suffix) {
            let (h, m) = clock_parts(num)?;
            return twelve_hour(h, m, pm).map(|(h, m)| Unit::Clock(h, m));
        }
    }
    if let Some(num) = word.strip_suffix('h') {
        return hhmm(num).map(|(h, m)| Unit::Clock(h, m));
    }
    let (h, m) = colon_time(word)?;
    (h < 24).then_some(Unit::Clock(h, m))
}

/// `14/11`, `14/11/2026`, `2026-11-14` and `2026-11-14T15:00`, with or
/// without the `@` of the canonical token.
fn numeric_date(word: &str, ctx: &Ctx) -> Option<Unit> {
    let bare = word.strip_prefix('@').unwrap_or(word);
    if bare.contains('-') {
        // `lower` made the `T` small.
        let due = Due::parse(&bare.to_ascii_uppercase())?;
        return Some(match due.time {
            Some((h, m)) => Unit::Both(due.date, h, m),
            None => Unit::Day(due.date),
        });
    }
    let parts: Vec<&str> = word.split('/').collect();
    if parts.len() < 2 || parts.len() > 3 {
        return None;
    }
    let (a, b) = (plain_number(parts[0], 2)?, plain_number(parts[1], 2)?);
    let (day, month) = match ctx.order {
        DateOrder::DayMonth => (a, b),
        DateOrder::MonthDay => (b, a),
    };
    match parts.get(2) {
        None => next_turn(month, day, ctx.now.date).map(Unit::Day),
        Some(year) if year.len() == 4 => {
            let year = i32::try_from(plain_number(year, 4)?).ok()?;
            Date::new(year, month, day).map(Unit::Day)
        }
        Some(_) => None,
    }
}

/// The first day on or after `today` that falls on `month`/`day`.
fn next_turn(month: u32, day: u32, today: Date) -> Option<Date> {
    if !(1..=12).contains(&month) || day == 0 || day > 31 {
        return None;
    }
    // A leap day may be four years off, across a century year eight.
    (today.year..=today.year + 8)
        .filter_map(|year| Date::new(year, month, day))
        .find(|date| *date >= today)
}

/// Digits only, at most `max` of them.
fn plain_number(text: &str, max: usize) -> Option<u32> {
    (!text.is_empty() && text.len() <= max && text.bytes().all(|b| b.is_ascii_digit()))
        .then(|| text.parse().ok())?
}

/// A day of the month: `14`, `14th`.
fn day_number(text: &str) -> Option<u32> {
    let digits = ["st", "nd", "rd", "th"]
        .iter()
        .find_map(|suffix| text.strip_suffix(suffix))
        .unwrap_or(text);
    plain_number(digits, 2)
}

/// `H:MM` or `HH:MM`.
fn colon_time(text: &str) -> Option<(u32, u32)> {
    let (h, m) = text.split_once(':')?;
    if m.len() != 2 {
        return None;
    }
    let (h, m) = (plain_number(h, 2)?, plain_number(m, 2)?);
    (m < 60).then_some((h, m))
}

/// The hour and minute of a 12-hour clock before its `am` or `pm`: `3`,
/// `3:30`.
fn clock_parts(text: &str) -> Option<(u32, u32)> {
    if text.contains(':') {
        colon_time(text)
    } else {
        plain_number(text, 2).map(|h| (h, 0))
    }
}

fn twelve_hour(hour: u32, minute: u32, pm: bool) -> Option<(u32, u32)> {
    ((1..=12).contains(&hour) && minute < 60)
        .then_some((hour % 12 + if pm { 12 } else { 0 }, minute))
}

/// Four digits, `1500`.
fn hhmm(text: &str) -> Option<(u32, u32)> {
    if text.len() != 4 {
        return None;
    }
    let n = plain_number(text, 4)?;
    let (h, m) = (n / 100, n % 100);
    (h < 24 && m < 60).then_some((h, m))
}

/// 0 for Monday through 6 for Sunday.
fn weekday_of(text: &str) -> Option<usize> {
    Some(match text {
        "mon" | "monday" => 0,
        "tue" | "tues" | "tuesday" => 1,
        "wed" | "weds" | "wednesday" => 2,
        "thu" | "thur" | "thurs" | "thursday" => 3,
        "fri" | "friday" => 4,
        "sat" | "saturday" => 5,
        "sun" | "sunday" => 6,
        _ => return None,
    })
}

/// 1 for January. Full names, three-letter forms and `sept`.
fn month_of(text: &str) -> Option<u32> {
    if text == "sept" {
        return Some(9);
    }
    MONTHS
        .iter()
        .position(|name| {
            let name = name.to_lowercase();
            text == name || (text.len() == 3 && name.starts_with(text))
        })
        .map(|ix| ix as u32 + 1)
}

#[cfg(test)]
#[path = "due_phrase_tests.rs"]
mod tests;
