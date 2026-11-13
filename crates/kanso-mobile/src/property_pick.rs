//! Which editor a property row gets on the phone, and the values its pickers
//! write: the rules of the desktop inspector's pickers
//! (`kanso-app/src/property_pick.rs`), in terms of the rows the bridge hands
//! the shell. Nothing here draws.
//!
//! A notebook's property list says what a key holds (`kansoProperties`); a
//! key it does not define still gets a picker when its value is already a
//! date, a date and time, or a date and time with a zone. A value that does
//! not fit its type is kept as written, and the picker starts empty.
//!
//! A pick is written under the same check as on the desktop: the row must
//! still name the key and hold the value the picker showed, or the pick is
//! dropped (the shell says so) rather than written to another property.

use chrono::NaiveDate;
use kanso_core::dates::{self, Date, DateTime};
use kanso_core::properties::{self, Value};
use kanso_core::schema::{PropertyDef, PropertyType};
use kanso_core::zoned;

use crate::errors::KansoError;
use crate::model::{Property, PropertyKind};
use crate::property_pick_model::{Moment, PickKind, PropertyChoice, PropertyPick, ZoneRow};

/// What decides a row's editor besides its value: the notebook's property
/// list, the people of a shared note, and the zone this device is in.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct PickContext {
    pub defs: Vec<PropertyDef>,
    /// The usernames of a shared note; none outside one.
    pub members: Vec<String>,
    pub device_zone: Option<String>,
}

/// The zone this device is in, when the tz database names it.
pub(crate) fn device_zone() -> Option<String> {
    iana_time_zone::get_timezone()
        .ok()
        .filter(|name| zoned::zone(name).is_some())
}

/// The row for a property of a note's front matter.
pub(crate) fn row(key: String, value: &Value, ctx: &PickContext) -> Property {
    let (kind, items) = match value {
        Value::Text(_) => (PropertyKind::Text, Vec::new()),
        Value::DateTime(_) => (PropertyKind::DateTime, Vec::new()),
        Value::DateTimeZone(_) => (PropertyKind::DateTimeZone, Vec::new()),
        Value::Number(_) => (PropertyKind::Number, Vec::new()),
        Value::Bool(_) => (PropertyKind::Bool, Vec::new()),
        Value::Date(_) => (PropertyKind::Date, Vec::new()),
        Value::List(items) => (PropertyKind::List, items.iter().map(Value::text).collect()),
        Value::Raw(_) => (PropertyKind::Raw, Vec::new()),
    };
    let def = ctx
        .defs
        .iter()
        .find(|def| def.key.eq_ignore_ascii_case(&key));
    Property {
        pick: pick_for(def, value, ctx),
        key,
        kind,
        text: value.text(),
        items,
    }
}

/// The editor for a property, or `None` when it is edited as text (or a
/// switch, or in the note).
pub(crate) fn pick_for(
    def: Option<&PropertyDef>,
    value: &Value,
    ctx: &PickContext,
) -> Option<PropertyPick> {
    if value.is_raw() {
        return None;
    }
    let kind = match def {
        Some(def) => match def.kind {
            PropertyType::Select if !def.options.is_empty() => PickKind::Select,
            PropertyType::Person if !ctx.members.is_empty() => PickKind::Person,
            PropertyType::Date => PickKind::Date,
            PropertyType::DateTime => PickKind::DateTime,
            PropertyType::DateTimeZone => PickKind::DateTimeZone,
            _ => return None,
        },
        None => match value {
            Value::Date(_) => PickKind::Date,
            Value::DateTime(_) => PickKind::DateTime,
            Value::DateTimeZone(_) => PickKind::DateTimeZone,
            _ => return None,
        },
    };
    let mut pick = PropertyPick {
        kind,
        rows: Vec::new(),
        unlisted: false,
        moment: None,
        zone: None,
        zone_set: false,
        label: None,
    };
    match kind {
        PickKind::Select | PickKind::Person => {
            let options = match (kind, def) {
                (PickKind::Select, Some(def)) => def.options.as_slice(),
                _ => ctx.members.as_slice(),
            };
            pick.rows = choices(options, value);
            pick.unlisted = !value.is_empty() && !options.contains(&value.text());
        }
        PickKind::Date | PickKind::DateTime | PickKind::DateTimeZone => {
            pick.moment = moment_of(value);
            if kind == PickKind::DateTimeZone {
                let own = zone_of(value);
                pick.zone_set = own.is_some();
                pick.zone = own.or_else(|| ctx.device_zone.clone());
                pick.label = zoned_label(value);
            }
        }
    }
    Some(pick)
}

/// The rows of a dropdown over `options`: the options in order, and the
/// current text first of all when it is not one of them, so a value the
/// notebook no longer lists is still shown rather than replaced.
fn choices(options: &[String], current: &Value) -> Vec<String> {
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

/// The day and time a picker starts on for `value`: what it holds, when it
/// holds a date of any kind. A date-only value starts at midnight.
fn moment_of(value: &Value) -> Option<Moment> {
    match value {
        Value::Date(text) => {
            let date = Date::parse_iso(text)?;
            moment(&DateTime {
                date,
                hour: 0,
                minute: 0,
                second: 0,
            })
        }
        Value::DateTime(text) => moment(&zoned::parse_local(text)?),
        Value::DateTimeZone(text) => moment(&zoned::parse_zoned(text)?.local),
        _ => None,
    }
}

fn moment(local: &DateTime) -> Option<Moment> {
    // A date the calendar has (`2026-02-31` is not one) or nothing.
    NaiveDate::from_ymd_opt(local.date.year, local.date.month, local.date.day)?;
    Some(Moment {
        year: local.date.year,
        month: local.date.month,
        day: local.date.day,
        hour: local.hour,
        minute: local.minute,
    })
}

/// The zone a date-and-time value is in.
fn zone_of(value: &Value) -> Option<String> {
    match value {
        Value::DateTimeZone(text) => zoned::parse_zoned(text).map(|at| at.zone),
        _ => None,
    }
}

/// How a zoned value reads in the row: `Tue 18 Nov, 09:00 New York`.
fn zoned_label(value: &Value) -> Option<String> {
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

/// Every zone of the picker, in order.
pub fn zone_rows() -> Vec<ZoneRow> {
    zoned::zone_names()
        .into_iter()
        .map(|name| ZoneRow {
            name: name.to_owned(),
            label: zoned::zone_label(name),
        })
        .collect()
}

/// The local date and time a moment is, checked: a day the calendar has, an
/// hour and a minute the clock has.
fn local_of(at: &Moment) -> Result<DateTime, KansoError> {
    let valid = NaiveDate::from_ymd_opt(at.year, at.month, at.day).is_some()
        && at.hour < 24
        && at.minute < 60;
    if !valid {
        return Err(KansoError::invalid("That is not a date and time."));
    }
    Ok(DateTime {
        date: Date {
            year: at.year,
            month: at.month,
            day: at.day,
        },
        hour: at.hour,
        minute: at.minute,
        second: 0,
    })
}

fn zoned_value(local: DateTime, zone: &str) -> Result<Value, KansoError> {
    if zoned::zone(zone).is_none() {
        return Err(KansoError::invalid("That time zone is not known."));
    }
    Ok(Value::DateTimeZone(
        zoned::ZonedTime {
            local,
            zone: zone.to_owned(),
        }
        .text(),
    ))
}

/// The value a pick writes over `current`, or `None` when nothing is to be
/// written (a zone picked before the value has a time to put in it).
fn picked(current: &Value, choice: &PropertyChoice) -> Result<Option<Value>, KansoError> {
    Ok(Some(match choice {
        PropertyChoice::Clear => Value::Text(String::new()),
        PropertyChoice::Option { text } => Value::Text(text.clone()),
        PropertyChoice::Day { moment } => Value::Date(local_of(moment)?.date.iso()),
        PropertyChoice::Moment { moment } => Value::DateTime(zoned::local_text(&local_of(moment)?)),
        PropertyChoice::Zoned { moment, zone } => zoned_value(local_of(moment)?, zone)?,
        // The wall-clock time stays whichever zone it is read in: an embargo
        // at 09:00 New York stays 09:00 when its zone becomes London.
        PropertyChoice::Zone { zone } => {
            let held = match current {
                Value::DateTime(text) => zoned::parse_local(text),
                Value::DateTimeZone(text) => zoned::parse_zoned(text).map(|at| at.local),
                _ => None,
            };
            match held {
                Some(local) => zoned_value(local, zone)?,
                None => {
                    if zoned::zone(zone).is_none() {
                        return Err(KansoError::invalid("That time zone is not known."));
                    }
                    return Ok(None);
                }
            }
        }
    }))
}

/// Writes a pick into the front matter `hidden`: the new front matter, or
/// `None` when the pick is dropped because row `index` is not the one named
/// `key` holding `original` (the text the picker showed). A pick that
/// changes nothing returns the front matter as it is.
pub(crate) fn apply(
    hidden: &str,
    index: usize,
    key: &str,
    original: &str,
    choice: &PropertyChoice,
) -> Result<Option<String>, KansoError> {
    let Some((found, like)) = properties::parse(hidden).into_iter().nth(index) else {
        return Ok(None);
    };
    if !found.eq_ignore_ascii_case(key) || like.text() != original {
        return Ok(None);
    }
    Ok(Some(match picked(&like, choice)? {
        Some(value) => properties::set_property_at(hidden, index, &value),
        None => hidden.to_owned(),
    }))
}

#[cfg(test)]
#[path = "property_pick_tests.rs"]
mod tests;
