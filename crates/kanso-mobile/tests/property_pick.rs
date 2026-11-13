//! The phone's property rows through an open note: the editor a row has
//! (from the notebook's property list), a pick written to the front matter,
//! and a pick dropped when the front matter changed under it.

mod common;

use std::sync::Arc;

use common::*;
use kanso_core::schema::{PropertyDef, PropertyType};
use kanso_mobile::{Moment, PickKind, PropertyChoice, PropertyKind};
use kanso_store::{Origin, Store};
use kanso_sync::MemoryServer;
use proptest::prelude::*;
use proptest::test_runner::{Config, RngAlgorithm, RngSeed};
use serde_json::json;

fn def(key: &str, kind: PropertyType, options: &[&str]) -> PropertyDef {
    PropertyDef {
        key: key.into(),
        kind,
        options: options.iter().map(|o| (*o).to_owned()).collect(),
    }
}

fn studio(body: &str) -> (Phone, String) {
    let mut id = String::new();
    let phone = phone_with(&Arc::new(MemoryServer::new()), |store: &Store| {
        let book_id = book(store, "Studio");
        store
            .set_notebook_properties(
                &book_id,
                &[
                    def("stage", PropertyType::Select, &["Idea", "Shooting"]),
                    def("embargo", PropertyType::DateTimeZone, &[]),
                    def("publish", PropertyType::DateTime, &[]),
                    def("due", PropertyType::Date, &[]),
                    def("owner", PropertyType::Person, &[]),
                ],
                Origin::Ui,
            )
            .unwrap();
        id = raw_note(
            store,
            json!({"title": "Pixel", "body": body, "bookId": book_id}),
        );
    });
    (phone, id)
}

const BODY: &str =
    "---\nstage: Idea\nembargo:\npublish: 2026-11-18T09:00\ndue: 2026-11-20\nowner:\n---\ntext";

fn moment(day: u32, hour: u32, minute: u32) -> Moment {
    Moment {
        year: 2026,
        month: 11,
        day,
        hour,
        minute,
    }
}

// no-prop-test: a fixed notebook and the editor each row must get.
#[test]
fn the_notebooks_property_list_gives_each_row_its_editor() {
    let (phone, id) = studio(BODY);
    let rows = phone
        .library
        .edit(&id)
        .unwrap()
        .snapshot()
        .unwrap()
        .properties;
    let kinds: Vec<Option<PickKind>> = rows
        .iter()
        .map(|row| row.pick.as_ref().map(|p| p.kind))
        .collect();
    // The person is typed: the note is not shared.
    assert_eq!(
        kinds,
        [
            Some(PickKind::Select),
            Some(PickKind::DateTimeZone),
            Some(PickKind::DateTime),
            Some(PickKind::Date),
            None
        ]
    );
    assert_eq!(rows[2].kind, PropertyKind::DateTime);
    assert_eq!(rows[0].pick.as_ref().unwrap().rows, ["Idea", "Shooting"]);
}

// no-prop-test: the written forms are property-tested in `property_pick`.
#[test]
fn a_pick_is_written_and_a_stale_one_is_dropped() {
    let (phone, id) = studio(BODY);
    let session = phone.library.edit(&id).unwrap();
    let pick = |index: u32, key: &str, original: &str, choice: PropertyChoice| {
        session
            .pick_property(index, key.into(), original.into(), choice)
            .unwrap()
    };
    let out = pick(
        0,
        "stage",
        "Idea",
        PropertyChoice::Option {
            text: "Shooting".into(),
        },
    );
    assert!(out.applied);
    assert_eq!(out.properties[0].text, "Shooting");
    let out = pick(
        1,
        "embargo",
        "",
        PropertyChoice::Zoned {
            moment: moment(18, 9, 0),
            zone: "America/New_York".into(),
        },
    );
    assert!(out.applied);
    assert_eq!(out.properties[1].text, "2026-11-18T09:00 America/New_York");
    assert_eq!(
        out.properties[1].pick.as_ref().unwrap().label.as_deref(),
        Some("Wed 18 Nov, 09:00 New York")
    );
    // The picker still shows "Idea" but the row holds "Shooting": dropped.
    let out = pick(
        0,
        "stage",
        "Idea",
        PropertyChoice::Option {
            text: "Idea".into(),
        },
    );
    assert!(!out.applied);
    assert_eq!(out.properties[0].text, "Shooting");
    // The row is another property now: dropped, nothing else changed.
    let out = pick(0, "due", "2026-11-20", PropertyChoice::Clear);
    assert!(!out.applied);
    assert_eq!(out.properties[3].text, "2026-11-20");
}

proptest! {
    #![proptest_config(Config {
        cases: 16,
        rng_seed: RngSeed::Fixed(20_261_017),
        rng_algorithm: RngAlgorithm::ChaCha,
        failure_persistence: None,
        ..Config::default()
    })]

    /// A picked date-time shows as the row's value and moment.
    #[test]
    fn prop_a_picked_time_is_the_rows_value(day in 1u32..29, hour in 0u32..24, minute in 0u32..60) {
        let (phone, id) = studio(BODY);
        let session = phone.library.edit(&id).unwrap();
        let out = session
            .pick_property(2, "publish".into(), "2026-11-18T09:00".into(),
                PropertyChoice::Moment { moment: moment(day, hour, minute) })
            .unwrap();
        prop_assert!(out.applied);
        prop_assert_eq!(
            out.properties[2].text.clone(),
            format!("2026-11-{day:02}T{hour:02}:{minute:02}")
        );
        prop_assert_eq!(
            out.properties[2].pick.as_ref().unwrap().moment,
            Some(moment(day, hour, minute))
        );
    }
}
