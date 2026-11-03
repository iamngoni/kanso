//! Time in a question: "what did I decide about pricing in October", "notes
//! from last week", "what is due before 3 March". English only; any other
//! language gives `None` and a question is searched as it was written.
//!
//! Pure and local. `today` and the first day of the week are passed in, so a
//! phrase always means the same thing in a test.

use std::ops::Range;

use crate::dates::{Date, MONTHS, WEEKDAYS, days_in_month};

/// A run of days, both ends inclusive. An end left open reaches past what
/// the question can mean ("after 3 March").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DateRange {
    pub start: Option<Date>,
    pub end: Option<Date>,
}

/// What kind of time a question asks about.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeIntent {
    /// When the note was written ("what did I decide in October").
    Written(DateRange),
    /// What the note is about ("what is due before the launch").
    About(DateRange),
    /// The question does not say; either may match.
    Either(DateRange),
}

impl TimeIntent {
    /// The range, whichever kind it is.
    pub fn range(&self) -> DateRange {
        match self {
            TimeIntent::Written(range) | TimeIntent::About(range) | TimeIntent::Either(range) => {
                *range
            }
        }
    }
}

impl DateRange {
    pub fn new(start: Date, end: Date) -> Self {
        Self {
            start: Some(start),
            end: Some(end),
        }
    }

    /// Whether `date` is in the range.
    pub fn contains(&self, date: Date) -> bool {
        self.start.is_none_or(|start| date >= start) && self.end.is_none_or(|end| date <= end)
    }

    /// The range's first and last millisecond since the epoch (UTC), the
    /// open ends as `None`. `offset_ms` is the zone's offset from UTC, so a
    /// local day is cut where the person's day is.
    pub fn bounds_ms(&self, offset_ms: i64) -> (Option<i64>, Option<i64>) {
        self.bounds_ms_by(&|_| offset_ms)
    }

    /// [`bounds_ms`](Self::bounds_ms) for a zone whose offset changes (a
    /// daylight-saving change): `offset_at` gives the offset from UTC in
    /// milliseconds at an instant in milliseconds since the epoch, and each
    /// end is cut with the offset in force at that end's own midnight.
    pub fn bounds_ms_by(&self, offset_at: &dyn Fn(i64) -> i64) -> (Option<i64>, Option<i64>) {
        const DAY: i64 = 86_400_000;
        // The UTC instant at which local time reads `local_ms`: guess with
        // the offset at the local reading taken as UTC, then again with the
        // offset at the guess.
        let utc_of = |local_ms: i64| {
            let guess = local_ms - offset_at(local_ms);
            local_ms - offset_at(guess)
        };
        (
            self.start.map(|d| utc_of(d.to_days() * DAY)),
            self.end.map(|d| utc_of((d.to_days() + 1) * DAY) - 1),
        )
    }

    /// The range as a chip reads it: `October 2026`, `Q3 2026`, `2026`,
    /// `3 March 2026`, `3 – 9 March 2026`, `Before 3 March 2026`.
    pub fn label(&self) -> String {
        let day = |d: Date| format!("{} {} {}", d.day, MONTHS[d.month as usize - 1], d.year);
        match (self.start, self.end) {
            (Some(s), Some(e)) if s == e => day(s),
            (Some(s), Some(e)) => {
                let month_end = days_in_month(e.year, e.month);
                if s.year == e.year && s.day == 1 && e.day == month_end {
                    if s.month == e.month {
                        return format!("{} {}", MONTHS[s.month as usize - 1], s.year);
                    }
                    if s.month == 1 && e.month == 12 {
                        return s.year.to_string();
                    }
                    if e.month == s.month + 2 && (s.month - 1) % 3 == 0 {
                        return format!("Q{} {}", (s.month - 1) / 3 + 1, s.year);
                    }
                }
                if s.year == e.year && s.month == e.month {
                    return format!("{} – {}", s.day, day(e));
                }
                format!("{} – {}", day(s), day(e))
            }
            (Some(s), None) => format!("Since {}", day(s)),
            (None, Some(e)) => format!("Until {}", day(e)),
            (None, None) => String::new(),
        }
    }
}

/// Reads a time out of `question`, relative to `today`. Returns the intent
/// and the byte range of the words that expressed it (the preposition and
/// modifier included), which are left out of the search terms. `week_start`
/// is the first day of the week, 0 for Monday.
pub fn time_in(
    question: &str,
    today: Date,
    week_start: usize,
) -> Option<(TimeIntent, Range<usize>)> {
    let tokens = tokenize(question);
    let ahead = tokens.iter().any(|t| is_about_word(&t.word));
    let mut i = 0;
    while i < tokens.len() {
        if let Some((range, used)) = phrase_at(&tokens, i, today, week_start, ahead) {
            // A modifier or preposition just before the phrase belongs to it.
            let mut from = i;
            let mut range = range;
            if i > 0 {
                let before = tokens[i - 1].word.as_str();
                match before {
                    "before" => {
                        range = DateRange {
                            start: None,
                            end: range.start.map(|d| d.add_days(-1)),
                        };
                        from = i - 1;
                    }
                    "after" => {
                        range = DateRange {
                            start: range.end.map(|d| d.add_days(1)),
                            end: None,
                        };
                        from = i - 1;
                    }
                    "since" => {
                        range.end = None;
                        from = i - 1;
                    }
                    "until" | "till" | "by" => {
                        range.start = None;
                        from = i - 1;
                    }
                    "in" | "on" | "during" | "from" | "of" | "at" | "for" => from = i - 1,
                    _ => {}
                }
            }
            let span = tokens[from].span.start..tokens[i + used - 1].span.end;
            let intent = intent_of(&tokens, from..i + used, range);
            return Some((intent, span));
        }
        i += 1;
    }
    None
}

/// Every date written in `text`: `2026-03-03` (also after `@`), `3 March
/// 2026` and `March 3, 2026`. Dates that do not exist are skipped.
pub fn dates_in(text: &str) -> Vec<Date> {
    let tokens = tokenize(text);
    let mut found = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        if let Some(date) = iso_date(&tokens[i].word) {
            found.push(date);
            i += 1;
            continue;
        }
        if let Some((date, used)) = written_date(&tokens, i, true) {
            found.push(date);
            i += used;
            continue;
        }
        i += 1;
    }
    found
}

struct Token {
    /// Lowercase, without surrounding punctuation or a leading `@`.
    word: String,
    span: Range<usize>,
}

fn tokenize(text: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut start: Option<usize> = None;
    let push = |from: usize, to: usize, tokens: &mut Vec<Token>| {
        let raw = &text[from..to];
        let trimmed_start = raw.trim_start_matches(|c: char| !c.is_alphanumeric());
        let lead = raw.len() - trimmed_start.len();
        let word = trimmed_start.trim_end_matches(|c: char| !c.is_alphanumeric());
        if word.is_empty() {
            return;
        }
        let begin = from + lead;
        tokens.push(Token {
            word: word.to_lowercase(),
            span: begin..begin + word.len(),
        });
    };
    for (at, ch) in text.char_indices() {
        if ch.is_whitespace() {
            if let Some(from) = start.take() {
                push(from, at, &mut tokens);
            }
        } else if start.is_none() {
            start = Some(at);
        }
    }
    if let Some(from) = start {
        push(from, text.len(), &mut tokens);
    }
    tokens
}

fn month_number(word: &str) -> Option<u32> {
    MONTHS.iter().enumerate().find_map(|(ix, name)| {
        let name = name.to_lowercase();
        (word == name || (word.len() == 3 && name.starts_with(word)) || (word == "sept" && ix == 8))
            .then_some(ix as u32 + 1)
    })
}

fn weekday_number(word: &str) -> Option<usize> {
    WEEKDAYS.iter().position(|name| {
        let name = name.to_lowercase();
        word == name || (word.len() == 3 && name.starts_with(word))
    })
}

fn year_number(word: &str) -> Option<i32> {
    (word.len() == 4 && word.bytes().all(|b| b.is_ascii_digit()))
        .then(|| word.parse::<i32>().ok())?
        .filter(|year| (1900..=2100).contains(year))
}

fn day_number(word: &str) -> Option<u32> {
    let digits = word
        .strip_suffix("st")
        .or_else(|| word.strip_suffix("nd"))
        .or_else(|| word.strip_suffix("rd"))
        .or_else(|| word.strip_suffix("th"))
        .unwrap_or(word);
    (digits.len() <= 2 && !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()))
        .then(|| digits.parse::<u32>().ok())?
        .filter(|day| (1..=31).contains(day))
}

fn iso_date(word: &str) -> Option<Date> {
    Date::parse_iso(word)
}

fn iso_month(word: &str) -> Option<(i32, u32)> {
    let (year, month) = word.split_once('-')?;
    let year = year_number(year)?;
    (month.len() == 2 && month.bytes().all(|b| b.is_ascii_digit()))
        .then(|| month.parse::<u32>().ok())?
        .filter(|month| (1..=12).contains(month))
        .map(|month| (year, month))
}

/// A day with a month name and (when `need_year`) a year: `3 March 2026`,
/// `3rd of March, 2026`, `March 3 2026`. Without a year, the day is
/// returned in `today`'s year by the caller; here `None` unless `need_year`.
fn written_date(tokens: &[Token], i: usize, need_year: bool) -> Option<(Date, usize)> {
    let word = |ix: usize| tokens.get(ix).map(|t| t.word.as_str());
    let after_preposition = i > 0 && PREPOSITIONS.contains(&tokens[i - 1].word.as_str());
    let ordinal = |w: &str| ["st", "nd", "rd", "th"].iter().any(|s| w.ends_with(s));
    // 3 [of] March [2026]
    if let Some(day) = word(i).and_then(day_number) {
        let mut at = i + 1;
        let mut with_of = false;
        if word(at) == Some("of") {
            at += 1;
            with_of = true;
        }
        if let Some(month) = word(at).and_then(month_number) {
            let month_word = word(at).unwrap_or_default();
            at += 1;
            if let Some(year) = word(at).and_then(year_number) {
                return Date::new(year, month, day).map(|d| (d, at + 1 - i));
            }
            // "chapter 3 may" is not the third of May.
            if AMBIGUOUS_MONTHS.contains(&month_word)
                && !(after_preposition || with_of || ordinal(word(i).unwrap_or_default()))
            {
                return None;
            }
            if !need_year {
                // Year filled in by the caller: 1900 marks "no year".
                return Date::new(2000, month, day).map(|d| (d, at - i));
            }
        }
        return None;
    }
    // March 3 [2026]
    if let Some(month) = word(i).and_then(month_number)
        && let Some(day) = word(i + 1).and_then(day_number)
    {
        if let Some(year) = word(i + 2).and_then(year_number) {
            return Date::new(year, month, day).map(|d| (d, 3));
        }
        // "may 3 times" is not a date either.
        if AMBIGUOUS_MONTHS.contains(&word(i).unwrap_or_default())
            && !(after_preposition || ordinal(word(i + 1).unwrap_or_default()))
        {
            return None;
        }
        if !need_year {
            return Date::new(2000, month, day).map(|d| (d, 2));
        }
    }
    None
}

/// The date of `day` `month` nearest `today` in the direction the question
/// looks (`ahead`, or back): this year's, or the closest year that has the
/// day (29 February is in leap years only). Never silently none for a day
/// that exists in some year.
fn nearest_year(month: u32, day: u32, today: Date, ahead: bool) -> Option<Date> {
    let mut candidates = (today.year - 8..=today.year + 8).filter_map(|y| Date::new(y, month, day));
    if ahead {
        candidates.find(|d| *d >= today)
    } else {
        candidates.rfind(|d| *d <= today)
    }
}

/// Words that are also months. Without a year they are dates only when
/// something says so: a preposition before, an ordinal (`3rd`), or `of`.
const AMBIGUOUS_MONTHS: &[&str] = &["may", "march"];

const PREPOSITIONS: &[&str] = &[
    "in", "during", "of", "since", "before", "after", "from", "until", "till", "by", "last",
    "this", "on",
];

/// The range a phrase starting at token `i` means, and how many tokens it
/// uses.
fn phrase_at(
    tokens: &[Token],
    i: usize,
    today: Date,
    week_start: usize,
    ahead: bool,
) -> Option<(DateRange, usize)> {
    let word = |ix: usize| tokens.get(ix).map(|t| t.word.as_str());
    let w = word(i)?;
    let after_preposition = i > 0 && PREPOSITIONS.contains(&tokens[i - 1].word.as_str());
    let day = |d: Date| DateRange::new(d, d);

    match w {
        "yesterday" => return Some((day(today.add_days(-1)), 1)),
        "today" => return Some((day(today), 1)),
        "tomorrow" => return Some((day(today.add_days(1)), 1)),
        _ => {}
    }

    // last / past / previous / this / current / next + unit
    if matches!(
        w,
        "last" | "past" | "previous" | "this" | "current" | "next"
    ) {
        let rolling = word(i + 1).and_then(|n| n.parse::<i64>().ok());
        if let (Some(n), Some(unit)) = (rolling, word(i + 2))
            && (1..=3660).contains(&n)
            && matches!(w, "last" | "past" | "previous")
        {
            let days = match unit.trim_end_matches('s') {
                "day" => Some(n),
                "week" => Some(n * 7),
                "month" => Some(n * 30),
                "year" => Some(n * 365),
                _ => None,
            };
            if let Some(days) = days {
                return Some((DateRange::new(today.add_days(-days), today), 3));
            }
        }
        if let Some(unit) = word(i + 1) {
            let step = match w {
                "last" | "past" | "previous" => -1,
                "next" => 1,
                _ => 0,
            };
            // "past week" is the seven days up to today, "past year" the 365:
            // rolling, where "last week" is the calendar week before this.
            if w == "past" {
                let rolling = match unit {
                    "day" => Some(1),
                    "week" => Some(7),
                    "month" => Some(30),
                    "year" => Some(365),
                    _ => None,
                };
                if let Some(days) = rolling {
                    return Some((DateRange::new(today.add_days(-days), today), 2));
                }
            }
            let kind = match unit {
                "week" => Some(Unit::Week),
                "month" => Some(Unit::Month),
                "year" => Some(Unit::Year),
                "quarter" => Some(Unit::Quarter),
                _ => None,
            };
            if let Some(kind) = kind {
                return Some((period(kind, step, today, week_start), 2));
            }
            // last Friday, next Monday
            if let Some(weekday) = weekday_number(unit)
                && w != "this"
                && w != "current"
            {
                let date = if step < 0 {
                    let back = (today.weekday() + 7 - weekday) % 7;
                    today.add_days(-(if back == 0 { 7 } else { back as i64 }))
                } else {
                    let ahead = (weekday + 7 - today.weekday()) % 7;
                    today.add_days(if ahead == 0 { 7 } else { ahead as i64 })
                };
                return Some((day(date), 2));
            }
        }
    }

    // "on Friday": the most recent one, today included.
    if let Some(weekday) = weekday_number(w)
        && i > 0
        && tokens[i - 1].word == "on"
        && w.len() >= 6
    {
        let back = (today.weekday() + 7 - weekday) % 7;
        return Some((day(today.add_days(-(back as i64))), 1));
    }

    if let Some(date) = iso_date(w) {
        return Some((day(date), 1));
    }
    if let Some((year, month)) = iso_month(w) {
        return Some((month_range(year, month), 1));
    }

    // Q3 [2026]
    if w.len() == 2
        && w.starts_with('q')
        && let Some(q) = w[1..].parse::<u32>().ok().filter(|q| (1..=4).contains(q))
    {
        if let Some(year) = word(i + 1).and_then(year_number) {
            return Some((quarter_range(year, q), 2));
        }
        let mut year = today.year;
        if ahead {
            if quarter_range(year, q).end.is_some_and(|e| e < today) {
                year += 1;
            }
        } else if quarter_range(year, q).start.is_some_and(|s| s > today) {
            year -= 1;
        }
        return Some((quarter_range(year, q), 1));
    }

    // 3 March [2026], March 3 [2026]
    if let Some((date, used)) = written_date(tokens, i, false) {
        let has_year = word(i + used - 1).and_then(year_number).is_some();
        let date = if has_year {
            date
        } else {
            nearest_year(date.month, date.day, today, ahead)?
        };
        return Some((day(date), used));
    }

    // March 2026, in March
    if let Some(month) = month_number(w) {
        if let Some(year) = word(i + 1).and_then(year_number) {
            return Some((month_range(year, month), 2));
        }
        // A bare month is only a month after a preposition: "may" and
        // "march" are also ordinary words.
        if after_preposition {
            let mut year = today.year;
            if ahead {
                if month_range(year, month).end.is_some_and(|e| e < today) {
                    year += 1;
                }
            } else if month_range(year, month).start.is_some_and(|s| s > today) {
                year -= 1;
            }
            return Some((month_range(year, month), 1));
        }
        return None;
    }

    // in 2025
    if let Some(year) = year_number(w)
        && after_preposition
    {
        return Some((
            DateRange::new(Date::new(year, 1, 1)?, Date::new(year, 12, 31)?),
            1,
        ));
    }
    None
}

#[derive(Clone, Copy)]
enum Unit {
    Week,
    Month,
    Quarter,
    Year,
}

fn period(unit: Unit, step: i32, today: Date, week_start: usize) -> DateRange {
    match unit {
        Unit::Week => {
            let back = (today.weekday() + 7 - week_start % 7) % 7;
            let start = today.add_days(-(back as i64) + 7 * i64::from(step));
            DateRange::new(start, start.add_days(6))
        }
        Unit::Month => {
            let index = today.year * 12 + today.month as i32 - 1 + step;
            month_range(index.div_euclid(12), index.rem_euclid(12) as u32 + 1)
        }
        Unit::Quarter => {
            let q = (today.month - 1) / 3;
            let index = today.year * 4 + q as i32 + step;
            quarter_range(index.div_euclid(4), index.rem_euclid(4) as u32 + 1)
        }
        Unit::Year => {
            let year = today.year + step;
            DateRange::new(
                Date::new(year, 1, 1).unwrap_or(today),
                Date::new(year, 12, 31).unwrap_or(today),
            )
        }
    }
}

fn month_range(year: i32, month: u32) -> DateRange {
    let first = Date::new(year, month, 1).unwrap_or(Date {
        year,
        month: 1,
        day: 1,
    });
    DateRange::new(
        first,
        Date::new(year, month, days_in_month(year, month)).unwrap_or(first),
    )
}

fn quarter_range(year: i32, quarter: u32) -> DateRange {
    let first = (quarter - 1) * 3 + 1;
    let start = month_range(year, first).start;
    let end = month_range(year, first + 2).end;
    DateRange {
        start: start.or(end),
        end,
    }
}

/// Words that say the question is about what a note mentions, not when it
/// was written.
fn is_about_word(word: &str) -> bool {
    [
        "due",
        "happens",
        "scheduled",
        "schedule",
        "deadline",
        "deadlines",
        "expires",
        "upcoming",
    ]
    .contains(&word)
}

/// Which kind of time the verbs around the phrase ask for.
fn intent_of(tokens: &[Token], phrase: Range<usize>, range: DateRange) -> TimeIntent {
    let mut written = false;
    let mut about = false;
    for (ix, token) in tokens.iter().enumerate() {
        if phrase.contains(&ix) {
            continue;
        }
        let w = token.word.as_str();
        if [
            "wrote", "writ", "noted", "notes", "decid", "discuss", "met", "meet", "jot", "record",
            "logged", "journal", "said", "captur",
        ]
        .iter()
        .any(|verb| w == *verb || (verb.len() > 3 && w.starts_with(verb)))
        {
            written = true;
        }
        if is_about_word(w) {
            about = true;
        }
    }
    match (written, about) {
        (true, false) => TimeIntent::Written(range),
        (false, true) => TimeIntent::About(range),
        _ => TimeIntent::Either(range),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    /// Tuesday 20 October 2026.
    fn today() -> Date {
        Date::new(2026, 10, 20).unwrap()
    }

    fn d(y: i32, m: u32, day: u32) -> Date {
        Date::new(y, m, day).unwrap()
    }

    fn range(q: &str) -> Option<DateRange> {
        time_in(q, today(), 0).map(|(intent, _)| intent.range())
    }

    fn span(q: &str) -> Option<String> {
        time_in(q, today(), 0).map(|(_, r)| q[r].to_owned())
    }

    #[test]
    fn a_table_of_phrases_against_a_fixed_today() {
        type Bounds = Option<(Option<Date>, Option<Date>)>;
        let cases: &[(&str, Bounds)] = &[
            (
                "what did I decide about pricing in October",
                Some((Some(d(2026, 10, 1)), Some(d(2026, 10, 31)))),
            ),
            (
                "pricing in september",
                Some((Some(d(2026, 9, 1)), Some(d(2026, 9, 30)))),
            ),
            (
                "pricing in November",
                Some((Some(d(2025, 11, 1)), Some(d(2025, 11, 30)))),
            ),
            (
                "what happened in December 2024",
                Some((Some(d(2024, 12, 1)), Some(d(2024, 12, 31)))),
            ),
            (
                "notes from last week",
                Some((Some(d(2026, 10, 12)), Some(d(2026, 10, 18)))),
            ),
            (
                "notes from this week",
                Some((Some(d(2026, 10, 19)), Some(d(2026, 10, 25)))),
            ),
            (
                "what is due next week",
                Some((Some(d(2026, 10, 26)), Some(d(2026, 11, 1)))),
            ),
            (
                "notes from last month",
                Some((Some(d(2026, 9, 1)), Some(d(2026, 9, 30)))),
            ),
            ("last year's budget", None),
            (
                "what did I write last year",
                Some((Some(d(2025, 1, 1)), Some(d(2025, 12, 31)))),
            ),
            (
                "this year",
                Some((Some(d(2026, 1, 1)), Some(d(2026, 12, 31)))),
            ),
            (
                "this month",
                Some((Some(d(2026, 10, 1)), Some(d(2026, 10, 31)))),
            ),
            (
                "what did I note yesterday",
                Some((Some(d(2026, 10, 19)), Some(d(2026, 10, 19)))),
            ),
            (
                "what is due today",
                Some((Some(d(2026, 10, 20)), Some(d(2026, 10, 20)))),
            ),
            (
                "what is due tomorrow",
                Some((Some(d(2026, 10, 21)), Some(d(2026, 10, 21)))),
            ),
            (
                "what happened on 3 March",
                Some((Some(d(2026, 3, 3)), Some(d(2026, 3, 3)))),
            ),
            (
                "what happened on 3rd of March 2025",
                Some((Some(d(2025, 3, 3)), Some(d(2025, 3, 3)))),
            ),
            (
                "what happened on March 3, 2025",
                Some((Some(d(2025, 3, 3)), Some(d(2025, 3, 3)))),
            ),
            (
                "what happened on 25 December",
                Some((Some(d(2025, 12, 25)), Some(d(2025, 12, 25)))),
            ),
            (
                "meeting on 2026-03-03",
                Some((Some(d(2026, 3, 3)), Some(d(2026, 3, 3)))),
            ),
            (
                "in 2026-03",
                Some((Some(d(2026, 3, 1)), Some(d(2026, 3, 31)))),
            ),
            (
                "revenue in Q3 2026",
                Some((Some(d(2026, 7, 1)), Some(d(2026, 9, 30)))),
            ),
            (
                "revenue in Q4",
                Some((Some(d(2026, 10, 1)), Some(d(2026, 12, 31)))),
            ),
            (
                "revenue in Q4 2025",
                Some((Some(d(2025, 10, 1)), Some(d(2025, 12, 31)))),
            ),
            (
                "revenue in Q1",
                Some((Some(d(2026, 1, 1)), Some(d(2026, 3, 31)))),
            ),
            (
                "revenue last quarter",
                Some((Some(d(2026, 7, 1)), Some(d(2026, 9, 30)))),
            ),
            (
                "what did we ship in 2025",
                Some((Some(d(2025, 1, 1)), Some(d(2025, 12, 31)))),
            ),
            ("before 3 March 2026", Some((None, Some(d(2026, 3, 2))))),
            (
                "what is due before 1 November",
                Some((None, Some(d(2026, 10, 31)))),
            ),
            ("after 3 March 2026", Some((Some(d(2026, 3, 4)), None))),
            ("since 3 March", Some((Some(d(2026, 3, 3)), None))),
            ("since October", Some((Some(d(2026, 10, 1)), None))),
            ("until 2026-12-01", Some((None, Some(d(2026, 12, 1))))),
            (
                "in the last 3 days",
                Some((Some(d(2026, 10, 17)), Some(d(2026, 10, 20)))),
            ),
            (
                "past 2 weeks",
                Some((Some(d(2026, 10, 6)), Some(d(2026, 10, 20)))),
            ),
            (
                "last friday",
                Some((Some(d(2026, 10, 16)), Some(d(2026, 10, 16)))),
            ),
            (
                "last tuesday",
                Some((Some(d(2026, 10, 13)), Some(d(2026, 10, 13)))),
            ),
            (
                "next monday",
                Some((Some(d(2026, 10, 26)), Some(d(2026, 10, 26)))),
            ),
            (
                "on Tuesday",
                Some((Some(d(2026, 10, 20)), Some(d(2026, 10, 20)))),
            ),
            (
                "on monday",
                Some((Some(d(2026, 10, 19)), Some(d(2026, 10, 19)))),
            ),
            (
                "what did I decide in Oct",
                Some((Some(d(2026, 10, 1)), Some(d(2026, 10, 31)))),
            ),
            (
                "what did I decide in Sept 2025",
                Some((Some(d(2025, 9, 1)), Some(d(2025, 9, 30)))),
            ),
            (
                "during January",
                Some((Some(d(2026, 1, 1)), Some(d(2026, 1, 31)))),
            ),
            // Not times.
            ("what is the plan", None),
            ("we may need a march of progress", None),
            ("what is due before the launch", None),
            ("pricing", None),
            ("", None),
            ("a number 1234 and 99999", None),
            ("in 1850", None),
            // A day that does not exist falls back to the month it names.
            (
                "on 31 February 2026",
                Some((Some(d(2026, 2, 1)), Some(d(2026, 2, 28)))),
            ),
            ("in 2026-13-40", None),
        ];
        for (question, expected) in cases {
            let got = range(question).map(|r| (r.start, r.end));
            assert_eq!(&got, expected, "{question}");
        }
    }

    #[test]
    fn the_week_can_start_on_sunday() {
        let (intent, _) = time_in("last week", today(), 6).unwrap();
        // Sunday 11 Oct to Saturday 17 Oct.
        assert_eq!(
            intent.range(),
            DateRange::new(d(2026, 10, 11), d(2026, 10, 17))
        );
    }

    #[test]
    fn the_verb_decides_the_kind_of_time() {
        let kind = |q: &str| match time_in(q, today(), 0).unwrap().0 {
            TimeIntent::Written(_) => "written",
            TimeIntent::About(_) => "about",
            TimeIntent::Either(_) => "either",
        };
        assert_eq!(
            kind("what did I decide about pricing in October"),
            "written"
        );
        assert_eq!(kind("what did we discuss in October"), "written");
        assert_eq!(kind("who did I meet last week"), "written");
        assert_eq!(kind("notes from last week"), "written");
        assert_eq!(kind("what is due before 1 November"), "about");
        assert_eq!(kind("what is scheduled for next week"), "about");
        assert_eq!(kind("pricing in October"), "either");
        assert_eq!(kind("what happened on 3 March"), "either");
    }

    #[test]
    fn the_matched_words_are_reported_so_they_can_leave_the_search() {
        assert_eq!(
            span("pricing in October please").as_deref(),
            Some("in October")
        );
        assert_eq!(
            span("due before 3 March 2026 ok").as_deref(),
            Some("before 3 March 2026")
        );
        assert_eq!(
            span("notes from last week").as_deref(),
            Some("from last week")
        );
        assert_eq!(span("Q3 2026 revenue").as_deref(), Some("Q3 2026"));
        assert_eq!(
            span("what happened yesterday?").as_deref(),
            Some("yesterday")
        );
    }

    #[test]
    fn an_open_range_has_open_bounds() {
        let (from, to) = range("after 3 March 2026").unwrap().bounds_ms(0);
        assert!(from.is_some() && to.is_none());
    }

    #[test]
    fn ranges_read_as_a_chip_would() {
        assert_eq!(range("in October").unwrap().label(), "October 2026");
        assert_eq!(range("in Q3 2026").unwrap().label(), "Q3 2026");
        assert_eq!(range("in 2025").unwrap().label(), "2025");
        assert_eq!(range("on 3 March 2025").unwrap().label(), "3 March 2025");
        assert_eq!(range("last week").unwrap().label(), "12 – 18 October 2026");
        assert_eq!(
            range("before 3 March 2026").unwrap().label(),
            "Until 2 March 2026"
        );
        assert_eq!(
            range("since 3 March").unwrap().label(),
            "Since 3 March 2026"
        );
    }

    #[test]
    fn dates_are_found_in_note_text() {
        let text = "Due @2026-03-03, then 4 April 2026 and April 5, 2026; not 31 February 2026 or 2026-13-01.";
        assert_eq!(
            dates_in(text),
            vec![d(2026, 3, 3), d(2026, 4, 4), d(2026, 4, 5)]
        );
        assert!(dates_in("no dates 1 2 3").is_empty());
    }

    #[test]
    fn the_range_in_milliseconds_follows_the_zone() {
        let r = DateRange::new(d(1970, 1, 2), d(1970, 1, 2));
        assert_eq!(r.bounds_ms(0), (Some(86_400_000), Some(2 * 86_400_000 - 1)));
        // UTC+2: the local day starts two hours earlier in UTC.
        assert_eq!(r.bounds_ms(7_200_000).0, Some(86_400_000 - 7_200_000));
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// A phrase about the past never starts after today.
        #[test]
        fn prop_range_is_never_in_the_future_for_past_phrases(
            days in 0i64..40_000,
            month in 0usize..12,
            week_start in 0usize..7,
            phrase in 0usize..7,
        ) {
            let today = Date::from_days(days + 10_000);
            let name = MONTHS[month];
            let text = match phrase {
                0 => format!("what did I decide in {name}"),
                1 => "notes from last week".to_owned(),
                2 => "notes from last month".to_owned(),
                3 => "what did I write last year".to_owned(),
                4 => "what happened yesterday".to_owned(),
                5 => "revenue in Q2".to_owned(),
                _ => "what happened on 29 February".to_owned(),
            };
            if let Some((intent, _)) = time_in(&text, today, week_start) {
                let r = intent.range();
                prop_assert!(r.start.unwrap() <= today, "{text} on {today:?}: {r:?}");
                prop_assert!(r.start.unwrap() <= r.end.unwrap());
            }
        }

        /// Whatever it is asked, the module doesn't panic and its span is in
        /// the question on character boundaries.
        #[test]
        fn prop_time_in_never_panics_and_the_span_is_valid(text in "\\PC{0,60}", days in 0i64..30_000) {
            let today = Date::from_days(days);
            if let Some((_, span)) = time_in(&text, today, 0) {
                prop_assert!(text.get(span).is_some());
            }
            let _ = dates_in(&text);
        }
    }
}

#[cfg(test)]
mod review_tests {
    use super::*;

    /// Tuesday 20 October 2026.
    fn today() -> Date {
        Date::new(2026, 10, 20).unwrap()
    }

    fn d(y: i32, m: u32, day: u32) -> Date {
        Date::new(y, m, day).unwrap()
    }

    fn range(q: &str) -> Option<(Option<Date>, Option<Date>)> {
        time_in(q, today(), 0).map(|(intent, _)| {
            let r = intent.range();
            (r.start, r.end)
        })
    }

    // no-prop-test: a table, as the main table is.
    #[test]
    fn a_table_of_the_phrases_review_found() {
        type Bounds = Option<(Option<Date>, Option<Date>)>;
        let cases: &[(&str, Bounds)] = &[
            // "past" is rolling, "last" is the calendar.
            (
                "notes from the past week",
                Some((Some(d(2026, 10, 13)), Some(d(2026, 10, 20)))),
            ),
            (
                "what did I write in the past year",
                Some((Some(d(2025, 10, 20)), Some(d(2026, 10, 20)))),
            ),
            (
                "notes from last week",
                Some((Some(d(2026, 10, 12)), Some(d(2026, 10, 18)))),
            ),
            (
                "what did I write last year",
                Some((Some(d(2025, 1, 1)), Some(d(2025, 12, 31)))),
            ),
            // 29 February with no year goes to the nearest leap year.
            (
                "what happened on 29 February",
                Some((Some(d(2024, 2, 29)), Some(d(2024, 2, 29)))),
            ),
            (
                "what is due on 29 Feb",
                Some((Some(d(2028, 2, 29)), Some(d(2028, 2, 29)))),
            ),
            // "may" and "march" are dates only with something to say so.
            ("chapter 3 may need work", None),
            ("see chapter 3 may", None),
            ("it may 3 times", None),
            (
                "what happened on 3 may",
                Some((Some(d(2026, 5, 3)), Some(d(2026, 5, 3)))),
            ),
            (
                "the 3rd may",
                Some((Some(d(2026, 5, 3)), Some(d(2026, 5, 3)))),
            ),
            (
                "notes from 3 May 2025",
                Some((Some(d(2025, 5, 3)), Some(d(2025, 5, 3)))),
            ),
            (
                "what did I do on may 3",
                Some((Some(d(2026, 5, 3)), Some(d(2026, 5, 3)))),
            ),
        ];
        for (question, expected) in cases {
            assert_eq!(&range(question), expected, "{question}");
        }
    }

    // no-prop-test: one fixed zone with a spring-forward change.
    #[test]
    fn a_range_across_a_daylight_saving_change_uses_each_ends_own_offset() {
        const HOUR: i64 = 3_600_000;
        const DAY: i64 = 86_400_000;
        // New York, 2026: clocks go forward at 07:00 UTC on 8 March.
        let change = d(2026, 3, 8).to_days() * DAY + 7 * HOUR;
        let zone = |at: i64| if at < change { -5 * HOUR } else { -4 * HOUR };
        let r = DateRange::new(d(2026, 3, 7), d(2026, 3, 9));
        let (from, to) = r.bounds_ms_by(&zone);
        // 7 March 00:00 EST is 05:00 UTC; the end of 9 March EDT is 04:00 UTC
        // on 10 March, less a millisecond.
        assert_eq!(from, Some(d(2026, 3, 7).to_days() * DAY + 5 * HOUR));
        assert_eq!(to, Some(d(2026, 3, 10).to_days() * DAY + 4 * HOUR - 1));
        // One fixed offset would have put the end an hour out.
        assert_ne!(to, r.bounds_ms(-5 * HOUR).1);
        // The fixed form is the same as a zone that never changes.
        assert_eq!(r.bounds_ms(HOUR), r.bounds_ms_by(&|_| HOUR));
    }
}
