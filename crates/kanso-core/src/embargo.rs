//! Embargoes: a note whose `embargo` property is still to come is held
//! back. It shows a banner, publishing it, making a share link to it or
//! exporting it asks first, and a reminder says when it lifts.
//!
//! The property is usually a date and time with a zone (`embargo:
//! 2026-11-18T09:00 America/New_York`), which lifts at the same moment on
//! every device. A date and time with no zone, or a day, lifts at that wall
//! clock time wherever the device is. Anything else is no embargo.
//!
//! An embargo is a guard rail, not access control: anyone who can read the
//! note can still copy its words out.

use crate::dates::{self, Date, DateTime};
pub use crate::embargo_model::{Ask, Embargo, Gate, Lift};
use crate::properties::{self, Value};
use crate::zoned;

/// The property that holds a note's embargo.
pub const KEY: &str = "embargo";

/// What a lift reminder's key has between the note's ID and the value.
const REMINDER_MARK: &str = "#embargo@";

const MINUTE_MS: i64 = 60_000;
const HOUR_MS: i64 = 60 * MINUTE_MS;
const DAY_MS: i64 = 24 * HOUR_MS;

/// The embargo `body`'s front matter sets, if it sets one Kanso can read.
pub fn of(body: &str) -> Option<Embargo> {
    let found = properties::parse(body);
    from_value(properties::get(&found, KEY)?)
}

/// The embargo a property value sets: a date and time with a zone, a date
/// and time, or a day. Text in one of those forms counts too (a notebook
/// that types the property as text). Anything else sets none.
pub fn from_value(value: &Value) -> Option<Embargo> {
    let text = match value {
        Value::DateTimeZone(text)
        | Value::DateTime(text)
        | Value::Date(text)
        | Value::Text(text) => text.trim(),
        _ => return None,
    };
    Some(Embargo {
        lift: lift_of(text)?,
        text: text.to_owned(),
    })
}

/// When `text` lifts, read as a zoned date and time, a date and time or a
/// day.
fn lift_of(text: &str) -> Option<Lift> {
    if let Some(at) = zoned::parse_zoned(text) {
        return Some(Lift::Zoned(at));
    }
    if let Some(at) = zoned::parse_local(text) {
        return Some(Lift::Local(at));
    }
    Date::parse_iso(text)
        .filter(|_| text.len() == 10)
        .map(|day| Lift::Local(DateTime::at_midnight(day)))
}

/// The embargo of `body` when it is still in force at `now_utc_ms` (local
/// time `now_local_ms`): `None` when the note has none or it has lifted.
/// What publish, share link and export ask about.
pub fn check(body: &str, now_utc_ms: i64, now_local_ms: i64) -> Option<Embargo> {
    of(body).filter(|embargo| embargo.is_active(now_utc_ms, now_local_ms))
}

/// Milliseconds since the epoch of a local date and time (as
/// [`DateTime::from_local_ms`] counts).
fn local_ms_of(at: &DateTime) -> i64 {
    at.date.to_days() * DAY_MS
        + i64::from(at.hour) * HOUR_MS
        + i64::from(at.minute) * MINUTE_MS
        + i64::from(at.second) * 1000
}

impl Embargo {
    /// How long until it lifts, from `now_utc_ms` (local `now_local_ms`).
    /// Zero or less once it has.
    pub fn remaining_ms(&self, now_utc_ms: i64, now_local_ms: i64) -> i64 {
        match &self.lift {
            Lift::Zoned(at) => at
                .utc_ms()
                .map_or(i64::MIN, |lifts| lifts.saturating_sub(now_utc_ms)),
            Lift::Local(at) => local_ms_of(at).saturating_sub(now_local_ms),
        }
    }

    /// Whether it is still in force at `now_utc_ms` (local `now_local_ms`).
    pub fn is_active(&self, now_utc_ms: i64, now_local_ms: i64) -> bool {
        self.remaining_ms(now_utc_ms, now_local_ms) > 0
    }

    /// When it lifts in this device's local time, in milliseconds since
    /// the epoch: a zoned one through `local_of` (UTC to local), any other
    /// as written. `None` for a zone the tz database does not know.
    pub fn lifts_local_ms(&self, local_of: &dyn Fn(i64) -> i64) -> Option<i64> {
        match &self.lift {
            Lift::Zoned(at) => at.utc_ms().map(local_of),
            Lift::Local(at) => Some(local_ms_of(at)),
        }
    }

    /// When it lifts, as people read it: `Tue 18 Nov, 09:00 New York`, or
    /// `Tue 18 Nov, 09:00` with no zone.
    pub fn label(&self) -> String {
        match &self.lift {
            Lift::Zoned(at) => format!(
                "{} {}",
                dates::format(&at.local, "%a %e %b, %H:%M"),
                zoned::zone_label(&at.zone)
            ),
            Lift::Local(at) => dates::format(at, "%a %e %b, %H:%M"),
        }
    }

    /// The banner under the note's title: "Embargoed until Tue 18 Nov,
    /// 09:00 New York."
    pub fn banner(&self) -> String {
        format!("Embargoed until {}.", self.label())
    }

    /// What asking before `gate` says, at `now_utc_ms` (local
    /// `now_local_ms`).
    pub fn ask(&self, gate: Gate, now_utc_ms: i64, now_local_ms: i64) -> Ask {
        let question = match gate {
            Gate::Publish => "Publish anyway?",
            Gate::ShareLink => "Make a share link anyway?",
            Gate::Export => "Export anyway?",
        };
        Ask {
            title: "This note is under embargo".to_owned(),
            body: format!(
                "Embargo lifts {} ({}). {question}",
                self.label(),
                in_words(self.remaining_ms(now_utc_ms, now_local_ms))
            ),
            confirm: confirm_label(gate).to_owned(),
        }
    }
}

/// The button that goes ahead past an embargo.
pub fn confirm_label(gate: Gate) -> &'static str {
    match gate {
        Gate::Publish => "Publish anyway",
        Gate::ShareLink => "Make link anyway",
        Gate::Export => "Export anyway",
    }
}

/// How long `ms` is from now, in words: "in 5 days", "in 3 hours", "in 12
/// minutes", "in under a minute", or "now" for none left. Days and hours
/// are rounded to the nearest; minutes up.
pub fn in_words(ms: i64) -> String {
    fn count(n: i64, unit: &str) -> String {
        if n == 1 {
            format!("in 1 {unit}")
        } else {
            format!("in {n} {unit}s")
        }
    }
    if ms <= 0 {
        return "now".to_owned();
    }
    if ms < MINUTE_MS {
        return "in under a minute".to_owned();
    }
    let minutes = (ms + MINUTE_MS - 1) / MINUTE_MS;
    if minutes < 60 {
        return count(minutes, "minute");
    }
    let hours = ((ms + HOUR_MS / 2) / HOUR_MS).max(1);
    if hours < 24 {
        return count(hours, "hour");
    }
    count(((ms + DAY_MS / 2) / DAY_MS).max(1), "day")
}

/// The key of the reminder that says note `note_id`'s embargo lifted:
/// `<note id>#embargo@<value as written>`. The same on every device, and a
/// new one when the date changes.
pub fn reminder_key(note_id: &str, embargo: &Embargo) -> String {
    format!("{note_id}{REMINDER_MARK}{}", embargo.text)
}

/// Whether reminder `key` is an embargo's lift notice rather than a task's.
pub fn is_reminder_key(key: &str) -> bool {
    key.contains(REMINDER_MARK)
}

/// The lift notice's title: "Embargo lifted: Pixel 11 review".
pub fn lifted_title(note_title: &str) -> String {
    let title = note_title.trim();
    if title.is_empty() {
        "Embargo lifted: Untitled".to_owned()
    } else {
        format!("Embargo lifted: {title}")
    }
}

/// A local time as a timed task's due (`YYYY-MM-DDTHH:MM`), rounded up to
/// the minute so the notice never comes before the embargo lifts.
pub fn due_text(local_ms: i64) -> String {
    let minute = local_ms.div_euclid(MINUTE_MS) + i64::from(local_ms.rem_euclid(MINUTE_MS) != 0);
    let at = DateTime::from_local_ms(minute * MINUTE_MS);
    format!("{}T{:02}:{:02}", at.date.iso(), at.hour, at.minute)
}

#[cfg(test)]
#[path = "embargo_tests.rs"]
mod tests;
