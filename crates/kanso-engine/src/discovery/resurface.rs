//! Resurfacing related notes while you write: the pure parts.
//!
//! What is asked of the store while a note is written ([`context`] and
//! [`query`]), how close a match has to be ([`Closeness`]), what "On this
//! day" means in calendar terms ([`day_ranges`], [`daily_ids`]), and the
//! words each item is told in ([`present`]). The store does the searching
//! (`kanso-store`'s `meaning/resurface.rs`); the desktop's margin and the
//! phones' pill show what [`present`] makes, so the two say the same.

use kanso_core::dates::{Date, format, is_leap};
use kanso_core::roles::daily_id;
use kanso_store::{
    DayNote, ResurfaceOpts, Resurfaced, SIMILARITY_CLOSER, SIMILARITY_DEFAULT, SIMILARITY_LOOSER,
    plain_text,
};

/// The paragraph being written needs at least this many words.
pub const MIN_QUERY_WORDS: usize = 6;
/// A paragraph with fewer words than this brings the one before it along.
pub const SHORT_PARAGRAPH_WORDS: usize = 12;
/// How many notes the margin and the pill show.
pub const SHOWN: usize = 3;
/// How many earlier years "On this day" looks back.
pub const YEARS_BACK: i32 = 30;

const DAY_MS: i64 = 86_400_000;

/// "How close": how alike a note must be to the writing to be shown.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Closeness {
    /// Fewer, nearer notes.
    Closer,
    #[default]
    Default,
    /// More, looser notes.
    Looser,
}

impl Closeness {
    pub const ALL: [Closeness; 3] = [Closeness::Closer, Closeness::Default, Closeness::Looser];

    pub fn threshold(self) -> f32 {
        match self {
            Closeness::Closer => SIMILARITY_CLOSER,
            Closeness::Default => SIMILARITY_DEFAULT,
            Closeness::Looser => SIMILARITY_LOOSER,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Closeness::Closer => "Closer",
            Closeness::Default => "Default",
            Closeness::Looser => "Looser",
        }
    }

    /// The name settings keep it under.
    pub fn key(self) -> &'static str {
        match self {
            Closeness::Closer => "closer",
            Closeness::Default => "default",
            Closeness::Looser => "looser",
        }
    }

    pub fn from_key(key: &str) -> Closeness {
        match key {
            "closer" => Closeness::Closer,
            "looser" => Closeness::Looser,
            _ => Closeness::Default,
        }
    }

    pub fn opts(self) -> ResurfaceOpts {
        ResurfaceOpts {
            threshold: self.threshold(),
            k: SHOWN,
        }
    }
}

/// The paragraph (blank-line separated block) of `body` the caret is in,
/// as plain words, with the one before it when it has fewer than
/// [`SHORT_PARAGRAPH_WORDS`] words. `None` unless that is at least
/// [`MIN_QUERY_WORDS`] words. A caret on a blank line belongs to the
/// paragraph above it. `caret` is a byte offset; one past the end or inside
/// a character is taken as the nearest place before it.
pub fn context(body: &str, caret: usize) -> Option<String> {
    let mut caret = caret.min(body.len());
    while !body.is_char_boundary(caret) {
        caret -= 1;
    }
    let paragraphs = paragraphs(body);
    // The paragraph the caret is in, or the last one before it.
    let at = paragraphs
        .iter()
        .rposition(|(start, _)| *start <= caret)
        .or(if paragraphs.is_empty() { None } else { Some(0) })?;
    let own = plain_text(&body[paragraphs[at].0..paragraphs[at].1]);
    let mut text = own.clone();
    if own.split_whitespace().count() < SHORT_PARAGRAPH_WORDS && at > 0 {
        let before = plain_text(&body[paragraphs[at - 1].0..paragraphs[at - 1].1]);
        if !before.is_empty() {
            text = format!("{before} {own}");
        }
    }
    (text.split_whitespace().count() >= MIN_QUERY_WORDS).then_some(text)
}

/// [`context`] with the note's title, which is what the store is asked
/// about. `None` when there is no context.
pub fn query(title: &str, body: &str, caret: usize) -> Option<String> {
    let text = context(body, caret)?;
    let title = title.trim();
    Some(if title.is_empty() {
        text
    } else {
        format!("{title}\n{text}")
    })
}

/// The byte ranges of the blank-line separated blocks of `body`.
fn paragraphs(body: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut open: Option<(usize, usize)> = None;
    let mut offset = 0;
    for line in body.split_inclusive('\n') {
        let start = offset;
        offset += line.len();
        let content = line.trim_end_matches(['\n', '\r']);
        if content.trim().is_empty() {
            out.extend(open.take());
            continue;
        }
        let end = start + content.len();
        open = Some(match open {
            Some((first, _)) => (first, end),
            None => (start, end),
        });
    }
    out.extend(open);
    out
}

/// The start of a local day in UTC milliseconds, given how far the zone is
/// from UTC (in seconds) at a UTC instant in milliseconds.
fn day_start(date: Date, offset_secs: &dyn Fn(i64) -> i32) -> i64 {
    let local = date.to_days() * DAY_MS;
    let first = local - i64::from(offset_secs(local)) * 1000;
    local - i64::from(offset_secs(first)) * 1000
}

/// The days of `year` that count as `today`'s month and day: that day, and
/// on February 28 of a year without a February 29, the February 29 of a
/// leap year too (a note from that day shows on the 28th).
fn same_days(today: Date, year: i32) -> Vec<Date> {
    let mut days = Vec::new();
    if let Some(date) = Date::new(year, today.month, today.day) {
        days.push(date);
    }
    if today.month == 2 && today.day == 28 && !is_leap(today.year) && is_leap(year) {
        days.extend(Date::new(year, 2, 29));
    }
    days
}

/// The earlier years' days that are `today`'s month and day, newest year
/// first, each with its year and the UTC millisecond range of the local day
/// (`start` up to but not including `end`).
pub fn day_ranges(today: Date, offset_secs: &dyn Fn(i64) -> i32) -> Vec<(i32, (i64, i64))> {
    let mut out = Vec::new();
    for year in (today.year - YEARS_BACK..today.year).rev() {
        for date in same_days(today, year) {
            let start = day_start(date, offset_secs);
            let end = day_start(date.add_days(1), offset_secs);
            out.push((year, (start, end)));
        }
    }
    out
}

/// The IDs of the daily notes for `today`'s month and day in earlier
/// years, newest year first.
pub fn daily_ids(today: Date) -> Vec<String> {
    let mut out = Vec::new();
    for year in (today.year - YEARS_BACK..today.year).rev() {
        for date in same_days(today, year) {
            out.push(daily_id(date));
        }
    }
    out
}

/// A note for "On this day", with how long ago.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OnThisDay {
    pub id: String,
    pub title: String,
    pub book: String,
    /// "1 year ago", "3 years ago".
    pub ago: String,
}

/// "1 year ago", "3 years ago".
pub fn years_ago(years: i32) -> String {
    match years {
        ..=0 => "This year".to_owned(),
        1 => "1 year ago".to_owned(),
        n => format!("{n} years ago"),
    }
}

/// Up to [`SHOWN`] notes written on `today`'s month and day in earlier
/// years, daily notes first, newest year first. Needs no embeddings, so it
/// works with related notes off.
pub fn on_this_day(
    store: &kanso_store::Store,
    today: Date,
    offset_secs: &dyn Fn(i64) -> i32,
    k: usize,
) -> kanso_store::Result<Vec<OnThisDay>> {
    let ranges: Vec<(i64, i64)> = day_ranges(today, offset_secs)
        .into_iter()
        .map(|(_, range)| range)
        .collect();
    let notes = store.on_this_day(&ranges, &daily_ids(today), k)?;
    Ok(notes
        .into_iter()
        .map(|note| {
            let year = note_year(&note, offset_secs);
            OnThisDay {
                ago: years_ago(today.year - year),
                id: note.id,
                title: note.title,
                book: note.book,
            }
        })
        .collect())
}

/// The year a note was written in, by the zone's calendar; a daily note's
/// own date is its year.
fn note_year(note: &DayNote, offset_secs: &dyn Fn(i64) -> i32) -> i32 {
    if let Some(date) = kanso_core::roles::daily_date(&note.id) {
        return date.year;
    }
    let local = note.created_at + i64::from(offset_secs(note.created_at)) * 1000;
    Date::from_local_ms(local).year
}

/// "March 2024".
pub fn month_year(created_at: i64, offset_secs: i32) -> String {
    let local = created_at + i64::from(offset_secs) * 1000;
    let dt = kanso_core::dates::DateTime::from_local_ms(local);
    format(&dt, "%B %Y")
}

/// "6 months ago", "1 year ago", "2 years ago": whole months under two
/// years, whole years after.
pub fn age_label(created_at: i64, now_ms: i64) -> String {
    let days = (now_ms - created_at).max(0) / DAY_MS;
    let months = days / 30;
    if months < 12 {
        return match months {
            0 | 1 => "1 month ago".to_owned(),
            n => format!("{n} months ago"),
        };
    }
    years_ago((days / 365).max(1) as i32)
}

/// A match as it is shown.
#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    pub id: String,
    pub title: String,
    pub book: String,
    /// "Home · March 2024" without the notebook when it has none.
    pub place: String,
    /// "March 2024".
    pub when: String,
    pub snippet: String,
    /// "You wrote about this before · 2 years ago" when the match is close
    /// and old enough.
    pub before: Option<String>,
}

/// The words of each match: `offset_secs` is the zone now, `now_ms` the
/// time now.
pub fn present(found: &[Resurfaced], now_ms: i64, offset_secs: i32) -> Vec<Item> {
    found
        .iter()
        .map(|note| {
            let when = month_year(note.created_at, offset_secs);
            let title = if note.title.trim().is_empty() {
                "Untitled".to_owned()
            } else {
                note.title.clone()
            };
            Item {
                id: note.id.clone(),
                place: if note.book.is_empty() {
                    when.clone()
                } else {
                    format!("{} \u{b7} {when}", note.book)
                },
                title,
                book: note.book.clone(),
                when,
                snippet: note.snippet.clone(),
                before: note.wrote_before(now_ms).then(|| {
                    format!(
                        "You wrote about this before \u{b7} {}",
                        age_label(note.created_at, now_ms)
                    )
                }),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use kanso_core::dates::days_in_month as last_day;
    use proptest::prelude::*;

    /// The letters and digits of `text`, in order.
    fn letters_of(text: &str) -> Vec<char> {
        text.chars().filter(|c| c.is_alphanumeric()).collect()
    }

    /// Whether `small` can be read off `big` by leaving characters out.
    fn is_subsequence(small: &[char], big: &[char]) -> bool {
        let mut rest = big.iter();
        small.iter().all(|c| rest.any(|b| b == c))
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256,
            failure_persistence: None,
            rng_seed: proptest::test_runner::RngSeed::Fixed(0x0063_7478),
            ..ProptestConfig::default()
        })]

        /// The context is made of the body's own words, whatever the
        /// caret; any caret is fine; and the same input gives the same
        /// answer.
        #[test]
        fn prop_context_uses_only_the_bodys_words(
            body in "([a-zA-Z]{1,7}[ ,.]{1,2}|\n\n|\n|é|# |- |\\*\\*|```\n){0,60}",
            caret in 0usize..400,
        ) {
            let found = context(&body, caret);
            prop_assert_eq!(&found, &context(&body, caret));
            if let Some(text) = found {
                prop_assert!(
                    is_subsequence(&letters_of(&text), &letters_of(&body)),
                    "{:?} is not made of {:?}", text, body
                );
                prop_assert!(text.split_whitespace().count() >= MIN_QUERY_WORDS);
            }
        }

        /// Days of earlier years only, in the zone's calendar: a range is
        /// one local day, and the same month and day as today (or the
        /// leap day beside the 28th).
        #[test]
        fn prop_day_ranges_are_earlier_years_on_this_day(
            year in 1990i32..2060,
            month in 1u32..=12,
            day in 1u32..=31,
            offset in -14i32 * 3600..=14 * 3600,
        ) {
            let day = day.min(last_day(year, month));
            let today = Date::new(year, month, day).unwrap();
            let zone = |_: i64| offset;
            let ranges = day_ranges(today, &zone);
            prop_assert!(ranges.len() <= (YEARS_BACK as usize) * 2);
            let mut last_year = i32::MAX;
            for (y, (start, end)) in &ranges {
                prop_assert!(*y < year && *y >= last_year.min(year) - YEARS_BACK);
                prop_assert!(*y <= last_year);
                last_year = *y;
                prop_assert_eq!(end - start, DAY_MS);
                let local = Date::from_local_ms(start + i64::from(offset) * 1000);
                prop_assert_eq!(local.year, *y);
                if local.month == month && local.day == day {
                    // The same day.
                } else {
                    // Only a February 29 beside a February 28.
                    prop_assert!(
                        month == 2 && day == 28 && local.month == 2 && local.day == 29
                    );
                }
                // The last instant of the range is still the same day.
                let last = Date::from_local_ms(end - 1 + i64::from(offset) * 1000);
                prop_assert_eq!(last, local);
            }
            prop_assert_eq!(daily_ids(today).len(), ranges.len());
        }
    }

    #[test]
    fn the_caret_picks_the_paragraph_and_short_ones_bring_the_one_before() {
        let body = "First paragraph has quite a few words in it to count here.\n\nShort one here now ok yes.\n\nA third paragraph with enough words to stand alone without help from others.";
        let first = body.find("First").unwrap();
        assert_eq!(
            context(body, first + 3).unwrap(),
            "First paragraph has quite a few words in it to count here."
        );
        let second = body.find("Short").unwrap();
        assert_eq!(
            context(body, second + 2).unwrap(),
            "First paragraph has quite a few words in it to count here. Short one here now ok yes."
        );
        let third = body.find("A third").unwrap();
        assert!(context(body, third).unwrap().starts_with("A third"));
        // On the blank line above it, the paragraph above.
        assert!(context(body, third - 1).unwrap().ends_with("ok yes."));
        // Too short.
        assert_eq!(context("only four words here", 3), None);
        assert_eq!(context("", 0), None);
        assert_eq!(
            query("Boiler", "one two three four five six seven", 2).unwrap(),
            "Boiler\none two three four five six seven"
        );
    }

    #[test]
    fn february_the_twenty_ninth_shows_on_the_twenty_eighth_of_other_years() {
        let zone = |_: i64| 0;
        // 2027 has no 29th: 2024's leap day shows on the 28th, with 2024's own 28th.
        let ranges = day_ranges(Date::new(2027, 2, 28).unwrap(), &zone);
        let in_2024: Vec<_> = ranges.iter().filter(|(y, _)| *y == 2024).collect();
        assert_eq!(in_2024.len(), 2);
        // In a leap year the 28th is just the 28th.
        let ranges = day_ranges(Date::new(2028, 2, 28).unwrap(), &zone);
        assert_eq!(ranges.iter().filter(|(y, _)| *y == 2024).count(), 1);
        // And the leap day itself skips years without one.
        let ranges = day_ranges(Date::new(2028, 2, 29).unwrap(), &zone);
        assert!(ranges.iter().all(|(y, _)| is_leap(*y)));
    }

    #[test]
    fn the_words_say_how_old_and_how_far() {
        let day = DAY_MS;
        assert_eq!(age_label(0, 200 * day), "6 months ago");
        assert_eq!(age_label(0, 400 * day), "1 year ago");
        assert_eq!(age_label(0, 800 * day), "2 years ago");
        assert_eq!(years_ago(1), "1 year ago");
        assert_eq!(years_ago(3), "3 years ago");
        // 2024-03-10 12:00 UTC.
        assert_eq!(month_year(1_710_072_000_000, 0), "March 2024");
        let near = Resurfaced {
            id: "note:a".into(),
            title: "".into(),
            book: "Home".into(),
            created_at: 1_710_072_000_000,
            score: 0.7,
            passage: (0, 3),
            snippet: "the valve".into(),
        };
        let items = present(
            std::slice::from_ref(&near),
            1_710_072_000_000 + 800 * day,
            0,
        );
        assert_eq!(items[0].title, "Untitled");
        assert_eq!(items[0].place, "Home \u{b7} March 2024");
        assert_eq!(
            items[0].before.as_deref(),
            Some("You wrote about this before \u{b7} 2 years ago")
        );
        let recent = present(&[near], 1_710_072_000_000 + 10 * day, 0);
        assert_eq!(recent[0].before, None);
    }

    #[test]
    fn closeness_maps_to_the_three_steps() {
        assert_eq!(Closeness::Closer.threshold(), 0.60);
        assert_eq!(Closeness::Default.threshold(), 0.50);
        assert_eq!(Closeness::Looser.threshold(), 0.42);
        for c in Closeness::ALL {
            assert_eq!(Closeness::from_key(c.key()), c);
        }
        assert_eq!(Closeness::from_key("nonsense"), Closeness::Default);
    }
}
