//! Which editor a property gets and what the pickers write: the table, and
//! the properties of writing a picked value into front matter and reading
//! it back.

use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use kanso_core::properties::{self, Value};
use kanso_core::schema::{PropertyDef, PropertyType};
use proptest::prelude::*;
use proptest::test_runner::{Config, RngAlgorithm, RngSeed};

use super::*;

fn def(kind: PropertyType, options: &[&str]) -> PropertyDef {
    PropertyDef {
        key: "k".into(),
        kind,
        options: options.iter().map(|o| (*o).to_owned()).collect(),
    }
}

fn at(text: &str) -> NaiveDateTime {
    NaiveDateTime::parse_from_str(text, "%Y-%m-%d %H:%M").unwrap()
}

#[test]
fn a_notebook_type_picks_the_editor() {
    let stage = def(PropertyType::Select, &["Idea", "Shooting"]);
    let empty = Value::Text(String::new());
    assert_eq!(
        pick_for(Some(&stage), &empty, &[]),
        Some(Pick::Select(vec!["Idea".into(), "Shooting".into()]))
    );
    // A select with no options has nothing to pick from.
    assert_eq!(
        pick_for(Some(&def(PropertyType::Select, &[])), &empty, &[]),
        None
    );
    assert_eq!(
        pick_for(Some(&def(PropertyType::Date, &[])), &empty, &[]),
        Some(Pick::Date)
    );
    assert_eq!(
        pick_for(Some(&def(PropertyType::DateTime, &[])), &empty, &[]),
        Some(Pick::DateTime)
    );
    assert_eq!(
        pick_for(Some(&def(PropertyType::DateTimeZone, &[])), &empty, &[]),
        Some(Pick::DateTimeZone)
    );
    let people = ["amy".to_owned(), "bo".to_owned()];
    let person = def(PropertyType::Person, &[]);
    assert_eq!(
        pick_for(Some(&person), &empty, &people),
        Some(Pick::Person(people.to_vec()))
    );
    // Outside a shared note a person is typed.
    assert_eq!(pick_for(Some(&person), &empty, &[]), None);
    assert_eq!(
        pick_for(Some(&def(PropertyType::Text, &[])), &empty, &[]),
        None
    );
}

#[test]
fn a_date_value_picks_without_a_notebook_type() {
    let date = Value::Date("2026-11-18".into());
    let when = Value::DateTime("2026-11-18T09:00".into());
    let zoned = Value::DateTimeZone("2026-11-18T09:00 America/New_York".into());
    assert_eq!(pick_for(None, &date, &[]), Some(Pick::Date));
    assert_eq!(pick_for(None, &when, &[]), Some(Pick::DateTime));
    assert_eq!(pick_for(None, &zoned, &[]), Some(Pick::DateTimeZone));
    assert_eq!(pick_for(None, &Value::Text("x".into()), &[]), None);
    // A value Kanso does not understand is only ever edited in the note.
    let raw = Value::Raw("a: 1".into());
    assert_eq!(
        pick_for(Some(&def(PropertyType::Date, &[])), &raw, &[]),
        None
    );
}

#[test]
fn a_value_the_notebook_no_longer_lists_stays_in_the_dropdown() {
    let options = ["Idea".to_owned(), "Shooting".to_owned()];
    let rows = choices(&options, &Value::Text("Archived".into()));
    assert_eq!(rows, ["Archived", "Idea", "Shooting"]);
    assert_eq!(position(&rows, &Value::Text("Archived".into())), Some(0));
    let rows = choices(&options, &Value::Text("Shooting".into()));
    assert_eq!(rows, ["Idea", "Shooting"]);
    assert_eq!(position(&rows, &Value::Text("Shooting".into())), Some(1));
    // Nothing set: no row is picked and none is added.
    let empty = Value::Text(String::new());
    assert_eq!(choices(&options, &empty), ["Idea", "Shooting"]);
    assert_eq!(position(&rows, &empty), None);
    assert_eq!(chosen(None), Value::Text(String::new()));
    assert_eq!(chosen(Some("Idea")), Value::Text("Idea".into()));
}

#[test]
fn pickers_write_the_documented_forms() {
    let when = at("2026-11-18 09:05");
    assert_eq!(date_value(when), Value::Date("2026-11-18".into()));
    assert_eq!(
        datetime_value(when),
        Value::DateTime("2026-11-18T09:05".into())
    );
    assert_eq!(
        zoned_value(when, "America/New_York"),
        Value::DateTimeZone("2026-11-18T09:05 America/New_York".into())
    );
    assert_eq!(
        zoned_label(&zoned_value(when, "America/New_York")).as_deref(),
        Some("Wed 18 Nov, 09:05 New York")
    );
    assert_eq!(zoned_label(&date_value(when)), None);
}

#[test]
fn changing_the_zone_keeps_the_wall_clock_time() {
    let value = zoned_value(at("2026-11-18 09:00"), "America/New_York");
    assert_eq!(
        with_zone(&value, "Europe/London"),
        Some(Value::DateTimeZone("2026-11-18T09:00 Europe/London".into()))
    );
    assert_eq!(with_zone(&value, "Nowhere/Land"), None);
    // No time yet: the zone has nothing to attach to.
    assert_eq!(with_zone(&Value::Text(String::new()), "UTC"), None);
    assert_eq!(zone_of(&value).as_deref(), Some("America/New_York"));
    assert_eq!(zone_of(&date_value(at("2026-11-18 09:00"))), None);
}

#[test]
fn the_zone_rows_find_the_zone() {
    let (rows, at) = zone_rows(Some("Africa/Harare"));
    assert_eq!(rows[at.unwrap()], "Africa/Harare");
    assert_eq!(zone_rows(None).1, None);
    assert_eq!(zone_rows(Some("Nowhere/Land")).1, None);
}

fn moment() -> impl Strategy<Value = NaiveDateTime> {
    (
        1000i32..9999,
        1u32..13,
        1u32..29,
        0u32..24,
        0u32..60,
        0u32..60,
    )
        .prop_map(|(y, mo, d, h, mi, s)| {
            NaiveDateTime::new(
                NaiveDate::from_ymd_opt(y, mo, d).unwrap(),
                NaiveTime::from_hms_opt(h, mi, s).unwrap(),
            )
        })
}

fn zone() -> impl Strategy<Value = &'static str> {
    proptest::sample::select(kanso_core::zoned::zone_names())
}

fn word() -> impl Strategy<Value = String> {
    "[A-Za-z][A-Za-z0-9 ]{0,10}[A-Za-z0-9]"
}

proptest! {
    #![proptest_config(Config {
        cases: 256,
        rng_seed: RngSeed::Fixed(20_261_017),
        rng_algorithm: RngAlgorithm::ChaCha,
        failure_persistence: None,
        ..Config::default()
    })]

    /// A picked day reads back as the same day, at midnight.
    #[test]
    fn prop_a_picked_day_reads_back(at in moment()) {
        let value = date_value(at);
        let back = naive_of(&value).expect("a date value reads");
        prop_assert_eq!(back.date(), at.date());
        prop_assert_eq!(back.time(), NaiveTime::MIN);
        prop_assert_eq!(pick_for(None, &value, &[]), Some(Pick::Date));
    }

    /// A picked time reads back to the minute, and drops the seconds the
    /// picker does not edit.
    #[test]
    fn prop_a_picked_time_reads_back_to_the_minute(at in moment()) {
        let value = datetime_value(at);
        let back = naive_of(&value).expect("a date-time value reads");
        prop_assert_eq!(back, at.with_second(0).unwrap());
        prop_assert_eq!(pick_for(None, &value, &[]), Some(Pick::DateTime));
    }

    /// A picked time in a zone reads back, with the zone, and changing the
    /// zone to another keeps the wall clock.
    #[test]
    fn prop_a_zoned_time_keeps_its_wall_clock(at in moment(), from in zone(), to in zone()) {
        let value = zoned_value(at, from);
        prop_assert_eq!(naive_of(&value), Some(at.with_second(0).unwrap()));
        prop_assert_eq!(zone_of(&value), Some(from.to_owned()));
        let moved = with_zone(&value, to).expect("a listed zone is taken");
        prop_assert_eq!(naive_of(&moved), naive_of(&value));
        prop_assert_eq!(zone_of(&moved), Some(to.to_owned()));
        prop_assert!(zoned_label(&moved).is_some_and(|label| !label.is_empty()));
    }

    /// What a picker writes goes through the front matter and comes back as
    /// the same value, and the rest of the note is untouched.
    #[test]
    fn prop_a_picked_value_survives_the_front_matter(
        at in moment(), zone in zone(), which in 0usize..3,
    ) {
        let value = match which {
            0 => date_value(at),
            1 => datetime_value(at),
            _ => zoned_value(at, zone),
        };
        let body = "---\nstage: Idea\nwhen:\n---\n# Note\n\nBody\n";
        let next = properties::set_property(body, "when", &value);
        let parsed = properties::parse(&next);
        prop_assert_eq!(properties::get(&parsed, "when"), Some(&value));
        prop_assert!(next.ends_with("---\n# Note\n\nBody\n"));
        prop_assert_eq!(
            properties::get(&parsed, "stage"),
            Some(&Value::Text("Idea".into()))
        );
    }

    /// A dropdown always shows the current value, lists every option once
    /// and in order, and finds the current value's row.
    #[test]
    fn prop_a_dropdown_keeps_the_current_value(
        mut options in proptest::collection::vec(word(), 0..8),
        current in proptest::option::of(word()),
    ) {
        options.sort();
        options.dedup();
        let current = Value::Text(current.unwrap_or_default());
        let rows = choices(&options, &current);
        let mut seen = rows.clone();
        seen.sort();
        seen.dedup();
        prop_assert_eq!(seen.len(), rows.len());
        let listed: Vec<String> = rows.iter().filter(|r| options.contains(r)).cloned().collect();
        prop_assert_eq!(listed, options.clone());
        match position(&rows, &current) {
            Some(ix) => prop_assert_eq!(&rows[ix], &current.text()),
            None => prop_assert!(current.is_empty()),
        }
    }
}
