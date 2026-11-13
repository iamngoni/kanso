//! Which editor a property gets in the inspector, and the values the
//! pickers write. Nothing here draws or reads a window: the controls are in
//! `inspector_properties` and `inspector_pickers`.
//!
//! A notebook's property list says what a key holds (`kansoProperties`);
//! a key it does not define still gets a picker when its value is already a
//! date, a date and time, or a date and time with a zone. A value that does
//! not fit its type is kept as written, and the picker starts empty.

use chrono::{Datelike as _, NaiveDate, NaiveDateTime, NaiveTime, Timelike as _};
use kanso_core::dates::{self, Date, DateTime};
use kanso_core::properties::Value;
use kanso_core::schema::{PropertyDef, PropertyType};
use kanso_core::zoned;

/// The control a property's value is edited with.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Pick {
    /// One of the notebook's options.
    Select(Vec<String>),
    /// One of a shared note's people.
    Person(Vec<String>),
    Date,
    DateTime,
    /// A date and time, and the zone they are in.
    DateTimeZone,
}

/// The control for a property of the open note, or `None` when it is edited
/// as text (or a switch, or in the note). `members` are the usernames in a
/// shared note; without any, a person is typed.
pub fn pick_for(def: Option<&PropertyDef>, value: &Value, members: &[String]) -> Option<Pick> {
    if value.is_raw() {
        return None;
    }
    if let Some(def) = def {
        return match def.kind {
            PropertyType::Select if !def.options.is_empty() => {
                Some(Pick::Select(def.options.clone()))
            }
            PropertyType::Person if !members.is_empty() => Some(Pick::Person(members.to_vec())),
            PropertyType::Date => Some(Pick::Date),
            PropertyType::DateTime => Some(Pick::DateTime),
            PropertyType::DateTimeZone => Some(Pick::DateTimeZone),
            _ => None,
        };
    }
    match value {
        Value::Date(_) => Some(Pick::Date),
        Value::DateTime(_) => Some(Pick::DateTime),
        Value::DateTimeZone(_) => Some(Pick::DateTimeZone),
        _ => None,
    }
}

/// The rows of a dropdown over `options`: the options in order, and the
/// current text first of all when it is not one of them, so a value the
/// notebook no longer lists is still shown (and warned about) rather than
/// replaced.
pub fn choices(options: &[String], current: &Value) -> Vec<String> {
    let mut rows: Vec<String> = Vec::with_capacity(options.len() + 1);
    let text = current.text();
    if !current.is_empty() && !options.contains(&text) {
        rows.push(text);
    }
    for option in options {
        if !rows.contains(option) {
            rows.push(option.clone());
        }
    }
    rows
}

/// Where `value` is among `rows`.
pub fn position(rows: &[String], value: &Value) -> Option<usize> {
    if value.is_empty() {
        return None;
    }
    let text = value.text();
    rows.iter().position(|row| *row == text)
}

/// The value a dropdown row sets: nothing set when no row is chosen.
pub fn chosen(row: Option<&str>) -> Value {
    Value::Text(row.unwrap_or_default().to_owned())
}

/// The date and time a picker starts on for `value`: what it holds, when it
/// holds a date of the picker's kind. A date-only value starts at midnight.
pub fn naive_of(value: &Value) -> Option<NaiveDateTime> {
    match value {
        Value::Date(text) => {
            let date = Date::parse_iso(text)?;
            Some(NaiveDateTime::new(naive_date(date)?, NaiveTime::MIN))
        }
        Value::DateTime(text) => {
            let local = zoned::parse_local(text)?;
            naive_of_local(&local)
        }
        Value::DateTimeZone(text) => {
            let at = zoned::parse_zoned(text)?;
            naive_of_local(&at.local)
        }
        _ => None,
    }
}

/// The zone a date-and-time value is in.
pub fn zone_of(value: &Value) -> Option<String> {
    match value {
        Value::DateTimeZone(text) => zoned::parse_zoned(text).map(|at| at.zone),
        _ => None,
    }
}

/// A picked day as a date value.
pub fn date_value(at: NaiveDateTime) -> Value {
    Value::Date(local_of(at).date.iso())
}

/// A picked day and minute as a date-and-time value.
pub fn datetime_value(at: NaiveDateTime) -> Value {
    Value::DateTime(zoned::local_text(&local_of(at)))
}

/// A picked day and minute in `zone`, the wall-clock time kept as it is
/// whichever zone it is read in (an embargo at 09:00 New York stays 09:00
/// when its zone changes to London: the zone says where that 09:00 is).
pub fn zoned_value(at: NaiveDateTime, zone: &str) -> Value {
    Value::DateTimeZone(
        zoned::ZonedTime {
            local: local_of(at),
            zone: zone.to_owned(),
        }
        .text(),
    )
}

/// The value after the zone of `value` is changed to `zone`: its wall-clock
/// time stays. An empty value has no time to keep, so it stays empty. The
/// zone is only taken when the tz database has it.
pub fn with_zone(value: &Value, zone: &str) -> Option<Value> {
    zoned::zone(zone)?;
    match naive_of(value) {
        Some(at) if matches!(value, Value::DateTimeZone(_)) => Some(zoned_value(at, zone)),
        _ => None,
    }
}

/// How a zoned value reads in the row: `Tue 18 Nov, 09:00 New York`.
pub fn zoned_label(value: &Value) -> Option<String> {
    let Value::DateTimeZone(text) = value else {
        return None;
    };
    let at = zoned::parse_zoned(text)?;
    Some(format!(
        "{} {}",
        dates::format(&at.local, "%a %e %b, %H:%M"),
        zoned::zone_label(&at.zone)
    ))
}

/// The dropdown rows for the zone picker, and where `zone` is among them.
/// The zone a value holds that the tz database does not name never reaches
/// here: such a value is kept as text.
pub fn zone_rows(zone: Option<&str>) -> (Vec<String>, Option<usize>) {
    let rows: Vec<String> = zoned::zone_names().into_iter().map(str::to_owned).collect();
    let at = zone.and_then(|zone| rows.iter().position(|row| row == zone));
    (rows, at)
}

fn naive_date(date: Date) -> Option<NaiveDate> {
    NaiveDate::from_ymd_opt(date.year, date.month, date.day)
}

fn naive_of_local(local: &DateTime) -> Option<NaiveDateTime> {
    let time = NaiveTime::from_hms_opt(local.hour, local.minute, 0)?;
    Some(NaiveDateTime::new(naive_date(local.date)?, time))
}

/// A picked time as the local date and time the properties write, to the
/// minute (the picker edits no seconds).
fn local_of(at: NaiveDateTime) -> DateTime {
    DateTime {
        date: Date {
            year: at.year(),
            month: at.month(),
            day: at.day(),
        },
        hour: at.hour(),
        minute: at.minute(),
        second: 0,
    }
}

#[cfg(test)]
#[path = "property_pick_tests.rs"]
mod tests;
