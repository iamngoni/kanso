//! Which editor a phone property row gets and what its pickers write: the
//! table, and the properties of writing a pick into front matter.

use kanso_core::properties::{self, Value};
use kanso_core::schema::{PropertyDef, PropertyType};
use proptest::prelude::*;
use proptest::test_runner::{Config, RngAlgorithm, RngSeed};

use super::*;

fn def(kind: PropertyType, options: &[&str]) -> PropertyDef {
    def_for("k", kind, options)
}

fn def_for(key: &str, kind: PropertyType, options: &[&str]) -> PropertyDef {
    PropertyDef {
        key: key.into(),
        kind,
        options: options.iter().map(|o| (*o).to_owned()).collect(),
    }
}

fn ctx(defs: Vec<PropertyDef>, members: &[&str]) -> PickContext {
    PickContext {
        defs,
        members: members.iter().map(|m| (*m).to_owned()).collect(),
        device_zone: Some("Africa/Harare".into()),
    }
}

fn kind_of(def: PropertyDef, value: &Value, members: &[&str]) -> Option<PickKind> {
    pick_for(Some(&def), value, &ctx(vec![], members)).map(|p| p.kind)
}

fn at(day: u32, hour: u32, minute: u32) -> Moment {
    Moment {
        year: 2026,
        month: 11,
        day,
        hour,
        minute,
    }
}

fn row_pick(key: &str, value: &Value, c: &PickContext) -> PropertyPick {
    row(key.into(), value, c).pick.unwrap()
}

#[test]
fn a_notebook_type_picks_the_editor() {
    let empty = Value::Text(String::new());
    let stage = def(PropertyType::Select, &["Idea", "Shooting"]);
    let pick = pick_for(Some(&stage), &empty, &ctx(vec![], &[])).unwrap();
    assert_eq!(pick.kind, PickKind::Select);
    assert_eq!(pick.rows, ["Idea", "Shooting"]);
    assert!(!pick.unlisted);
    assert_eq!(kind_of(def(PropertyType::Select, &[]), &empty, &[]), None);
    assert_eq!(
        kind_of(def(PropertyType::Date, &[]), &empty, &[]),
        Some(PickKind::Date)
    );
    assert_eq!(
        kind_of(def(PropertyType::DateTime, &[]), &empty, &[]),
        Some(PickKind::DateTime)
    );
    assert_eq!(
        kind_of(def(PropertyType::DateTimeZone, &[]), &empty, &[]),
        Some(PickKind::DateTimeZone)
    );
    let person = def(PropertyType::Person, &[]);
    let pick = pick_for(Some(&person), &empty, &ctx(vec![], &["amy", "bo"])).unwrap();
    assert_eq!(pick.kind, PickKind::Person);
    assert_eq!(pick.rows, ["amy", "bo"]);
    // Outside a shared note a person is typed.
    assert_eq!(kind_of(person, &empty, &[]), None);
    assert_eq!(kind_of(def(PropertyType::Text, &[]), &empty, &[]), None);
}

#[test]
fn a_date_value_picks_without_a_notebook_type() {
    let c = ctx(vec![], &[]);
    let kind = |v: &Value| pick_for(None, v, &c).map(|p| p.kind);
    assert_eq!(
        kind(&Value::Date("2026-11-18".into())),
        Some(PickKind::Date)
    );
    assert_eq!(
        kind(&Value::DateTime("2026-11-18T09:00".into())),
        Some(PickKind::DateTime)
    );
    assert_eq!(
        kind(&Value::DateTimeZone(
            "2026-11-18T09:00 America/New_York".into()
        )),
        Some(PickKind::DateTimeZone)
    );
    assert_eq!(kind(&Value::Text("x".into())), None);
    let raw = Value::Raw("a: 1".into());
    assert_eq!(kind_of(def(PropertyType::Date, &[]), &raw, &[]), None);
}

#[test]
fn a_value_the_notebook_no_longer_lists_stays_in_the_rows() {
    let stage = def(PropertyType::Select, &["Idea", "Shooting"]);
    let value = Value::Text("Archived".into());
    let pick = pick_for(Some(&stage), &value, &ctx(vec![], &[])).unwrap();
    assert_eq!(pick.rows, ["Archived", "Idea", "Shooting"]);
    assert!(pick.unlisted);
}

#[test]
fn a_zoned_row_says_when_and_where() {
    let value = Value::DateTimeZone("2026-11-18T09:00 America/New_York".into());
    let shown = row("embargo".into(), &value, &ctx(vec![], &[]));
    assert_eq!(shown.kind, PropertyKind::DateTimeZone);
    let pick = shown.pick.unwrap();
    assert_eq!(pick.moment, Some(at(18, 9, 0)));
    assert_eq!(pick.zone.as_deref(), Some("America/New_York"));
    assert!(pick.zone_set);
    assert_eq!(pick.label.as_deref(), Some("Wed 18 Nov, 09:00 New York"));
    // No zone yet: the device's is offered, and not marked as the value's.
    let plain = Value::DateTime("2026-11-18T09:00".into());
    let both = ctx(
        vec![def_for("publish", PropertyType::DateTimeZone, &[])],
        &[],
    );
    let pick = row_pick("publish", &plain, &both);
    assert_eq!(pick.zone.as_deref(), Some("Africa/Harare"));
    assert!(!pick.zone_set);
    // A value that does not fit is kept as written and the picker is empty.
    let pick = row_pick("publish", &Value::Text("soon".into()), &both);
    assert_eq!(pick.moment, None);
}

#[test]
fn picks_write_the_documented_forms() {
    let body = "---\nwhen:\n---\n";
    let go = |choice: PropertyChoice| apply(body, 0, "when", "", &choice).unwrap().unwrap();
    let read = |hidden: &str| properties::parse(hidden).remove(0).1;
    assert_eq!(
        read(&go(PropertyChoice::Day {
            moment: at(18, 9, 5)
        })),
        Value::Date("2026-11-18".into())
    );
    assert_eq!(
        read(&go(PropertyChoice::Moment {
            moment: at(18, 9, 5)
        })),
        Value::DateTime("2026-11-18T09:05".into())
    );
    assert_eq!(
        read(&go(PropertyChoice::Zoned {
            moment: at(18, 9, 5),
            zone: "America/New_York".into()
        })),
        Value::DateTimeZone("2026-11-18T09:05 America/New_York".into())
    );
    assert_eq!(
        read(&go(PropertyChoice::Option {
            text: "Idea".into()
        })),
        Value::Text("Idea".into())
    );
}

#[test]
fn changing_the_zone_keeps_the_wall_clock() {
    let body = "---\nwhen: 2026-11-18T09:00 America/New_York\n---\n";
    let original = "2026-11-18T09:00 America/New_York";
    let london = PropertyChoice::Zone {
        zone: "Europe/London".into(),
    };
    let next = apply(body, 0, "when", original, &london).unwrap().unwrap();
    assert_eq!(
        properties::parse(&next).remove(0).1,
        Value::DateTimeZone("2026-11-18T09:00 Europe/London".into())
    );
    // A zone for no time yet waits: nothing is written.
    let empty = "---\nwhen:\n---\n";
    let utc = PropertyChoice::Zone { zone: "UTC".into() };
    let same = apply(empty, 0, "when", "", &utc).unwrap();
    assert_eq!(same.as_deref(), Some(empty));
    // A zone the tz database lacks is refused.
    let nowhere = PropertyChoice::Zone {
        zone: "Nowhere/Land".into(),
    };
    assert!(apply(body, 0, "when", original, &nowhere).is_err());
}

#[test]
fn a_pick_is_dropped_when_the_front_matter_changed_under_it() {
    let choice = PropertyChoice::Option {
        text: "Idea".into(),
    };
    let body = "---\nstage: Shooting\nowner: amy\n---\n";
    // The row moved: another property is there now.
    assert_eq!(apply(body, 0, "owner", "amy", &choice).unwrap(), None);
    // The value changed.
    assert_eq!(apply(body, 0, "stage", "Editing", &choice).unwrap(), None);
    // The row is gone.
    assert_eq!(apply(body, 5, "stage", "Shooting", &choice).unwrap(), None);
    // Still what the picker showed (the key's case may differ).
    let next = apply(body, 0, "STAGE", "Shooting", &choice)
        .unwrap()
        .unwrap();
    assert!(next.contains("stage: Idea") && next.contains("owner: amy"));
    // A day the calendar lacks is refused.
    let bad = PropertyChoice::Day {
        moment: Moment {
            year: 2026,
            month: 2,
            day: 31,
            hour: 0,
            minute: 0,
        },
    };
    assert!(apply(body, 0, "stage", "Shooting", &bad).is_err());
}

#[test]
fn the_zone_rows_are_labelled() {
    let rows = zone_rows();
    assert!(rows.windows(2).all(|pair| pair[0].name < pair[1].name));
    let harare = rows.iter().find(|r| r.name == "Africa/Harare").unwrap();
    assert_eq!(harare.label, "Harare");
}

fn moments() -> impl Strategy<Value = Moment> {
    (1000i32..9999, 1u32..13, 1u32..29, 0u32..24, 0u32..60).prop_map(
        |(year, month, day, hour, minute)| Moment {
            year,
            month,
            day,
            hour,
            minute,
        },
    )
}

fn zone() -> impl Strategy<Value = &'static str> {
    proptest::sample::select(kanso_core::zoned::zone_names())
}

proptest! {
    #![proptest_config(Config {
        cases: 256,
        rng_seed: RngSeed::Fixed(20_261_017),
        rng_algorithm: RngAlgorithm::ChaCha,
        failure_persistence: None,
        ..Config::default()
    })]

    /// A pick reads back through the front matter as the moment picked, and
    /// the other properties are untouched.
    #[test]
    fn prop_a_pick_reads_back(at in moments(), zone in zone(), which in 0usize..3) {
        let choice = match which {
            0 => PropertyChoice::Day { moment: at },
            1 => PropertyChoice::Moment { moment: at },
            _ => PropertyChoice::Zoned { moment: at, zone: zone.to_owned() },
        };
        let body = "---\nstage: Idea\nwhen:\n---\n";
        let next = apply(body, 1, "when", "", &choice).unwrap().unwrap();
        let rows = properties::parse(&next);
        prop_assert_eq!(&rows[0].1, &Value::Text("Idea".into()));
        let pick = row_pick("when", &rows[1].1, &ctx(vec![], &[]));
        let back = pick.moment.expect("a pick reads back");
        prop_assert_eq!((back.year, back.month, back.day), (at.year, at.month, at.day));
        if which > 0 {
            prop_assert_eq!((back.hour, back.minute), (at.hour, at.minute));
        }
        prop_assert_eq!(pick.zone_set, which == 2);
    }

    /// A zone change keeps the wall clock, whatever the two zones.
    #[test]
    fn prop_a_zone_change_keeps_the_wall_clock(at in moments(), from in zone(), to in zone()) {
        let text = format!(
            "{:04}-{:02}-{:02}T{:02}:{:02} {from}",
            at.year, at.month, at.day, at.hour, at.minute
        );
        let value = Value::DateTimeZone(text.clone());
        let body = format!("---\nwhen: {text}\n---\n");
        let to_zone = PropertyChoice::Zone { zone: to.to_owned() };
        let next = apply(&body, 0, "when", &text, &to_zone).unwrap().unwrap();
        let pick = row_pick("when", &properties::parse(&next)[0].1, &ctx(vec![], &[]));
        prop_assert_eq!(pick.moment, moment_of(&value));
        prop_assert_eq!(pick.zone.as_deref(), Some(to));
    }

    /// A pick whose original is not what the row holds never writes.
    #[test]
    fn prop_a_stale_pick_writes_nothing(held in "[a-z]{1,8}", shown in "[a-z]{1,8}") {
        prop_assume!(held != shown);
        let body = format!("---\nstage: {held}\n---\n");
        let choice = PropertyChoice::Option { text: "x".into() };
        prop_assert_eq!(apply(&body, 0, "stage", &shown, &choice).unwrap(), None);
    }
}
