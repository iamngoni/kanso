//! A notebook's property list: saved with the notebook, checked against its
//! notes without rewriting them, and united one property at a time when two
//! devices change it apart.

use kanso_core::DocType;
use kanso_core::schema::{self, Mismatch, PropertyDef, PropertyType};
use kanso_store::{ApplyOutcome, Origin, Put, Store, SyncDoc};
use proptest::prelude::*;
use proptest::test_runner::FileFailurePersistence;
use serde_json::{Map, Value, json};

fn temp_store() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    (dir, store)
}

fn def(key: &str, kind: PropertyType, options: &[&str]) -> PropertyDef {
    PropertyDef {
        key: key.into(),
        kind,
        options: options.iter().map(|o| (*o).to_owned()).collect(),
    }
}

fn studio() -> Vec<PropertyDef> {
    vec![
        def(
            "stage",
            PropertyType::Select,
            &["Idea", "Scripting", "Shooting", "Editing", "Published"],
        ),
        def("embargo", PropertyType::DateTimeZone, &[]),
        def("publish", PropertyType::DateTime, &[]),
        def("sponsor", PropertyType::Text, &[]),
        def("owner", PropertyType::Person, &[]),
    ]
}

fn book(store: &Store, id: &str, updated: i64, list: Option<Value>) {
    let mut fields = json!({"_id": id, "name": "Studio", "updatedAt": updated});
    if let Some(list) = list {
        fields["kansoProperties"] = list;
    }
    store
        .put(DocType::Book, fields, Put::Upsert, Origin::Ui)
        .unwrap();
}

fn note(store: &Store, id: &str, book: &str, body: &str) {
    store
        .put(
            DocType::Note,
            json!({"_id": id, "title": "T", "body": body, "bookId": book}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
}

fn stored_list(store: &Store, id: &str) -> Vec<PropertyDef> {
    store.notebook_properties(id).unwrap()
}

/// A remote version of notebook `id`.
fn remote(id: &str, updated: i64, list: Option<Value>) -> SyncDoc {
    let mut fields = Map::new();
    fields.insert("name".into(), "Studio".into());
    fields.insert("parentBookId".into(), Value::Null);
    fields.insert("updatedAt".into(), updated.into());
    if let Some(list) = list {
        fields.insert("kansoProperties".into(), list);
    }
    SyncDoc {
        id: id.into(),
        rev: "5-bb".into(),
        deleted: false,
        fields,
        base_rev: None,
    }
}

#[test]
fn a_notebook_keeps_its_property_list() {
    let (_dir, store) = temp_store();
    book(&store, "book:s", 10, None);
    assert!(stored_list(&store, "book:s").is_empty());
    store
        .set_notebook_properties("book:s", &studio(), Origin::Ui)
        .unwrap();
    assert_eq!(stored_list(&store, "book:s"), studio());
    // Saved as the notebook's own field, in the documented shape.
    let doc = store.get("book:s").unwrap().unwrap();
    assert_eq!(
        doc.fields["kansoProperties"][1],
        json!({"key": "embargo", "type": "datetimeZone"})
    );
    // The notebook's name and place are untouched.
    assert_eq!(doc.fields["name"], "Studio");
    // An empty list clears it.
    store
        .set_notebook_properties("book:s", &[], Origin::Ui)
        .unwrap();
    assert!(stored_list(&store, "book:s").is_empty());
}

#[test]
fn a_list_that_would_confuse_a_note_is_refused_and_nothing_changes() {
    let (_dir, store) = temp_store();
    book(&store, "book:s", 10, None);
    store
        .set_notebook_properties("book:s", &studio(), Origin::Ui)
        .unwrap();
    for bad in [
        vec![def("", PropertyType::Text, &[])],
        vec![
            def("a", PropertyType::Text, &[]),
            def("A", PropertyType::Number, &[]),
        ],
        vec![def("a", PropertyType::Text, &["x"])],
        vec![def("a", PropertyType::Select, &["x", "x"])],
    ] {
        assert!(
            store
                .set_notebook_properties("book:s", &bad, Origin::Ui)
                .is_err()
        );
        assert_eq!(stored_list(&store, "book:s"), studio());
    }
    assert!(
        store
            .set_notebook_properties("book:missing", &studio(), Origin::Ui)
            .is_err()
    );
    // A note's id is not a notebook.
    note(&store, "note:n", "book:s", "x");
    assert!(
        store
            .set_notebook_properties("note:n", &studio(), Origin::Ui)
            .is_err()
    );
    // The stored field must be a list, whatever wrote it.
    assert!(
        store
            .put(
                DocType::Book,
                json!({"_id": "book:s", "kansoProperties": "stage"}),
                Put::Merge,
                Origin::Ui
            )
            .is_err()
    );
}

#[test]
fn a_value_that_does_not_fit_warns_and_the_note_is_never_rewritten() {
    let (_dir, store) = temp_store();
    book(&store, "book:s", 10, None);
    store
        .set_notebook_properties("book:s", &studio(), Origin::Ui)
        .unwrap();
    let body = "---\nstage: Filming\nembargo: 2026-11-18T09:00 Mars/Olympus\npublish: 2026-11-21T15:00\nowner: dan\n---\n# Brief\n";
    note(&store, "note:n", "book:s", body);
    let warnings = store.note_property_warnings("note:n").unwrap();
    assert_eq!(
        warnings,
        vec![
            (
                "stage".to_owned(),
                Mismatch::NotAnOption {
                    key: "stage".into()
                }
            ),
            ("embargo".to_owned(), Mismatch::NotADateTimeZone),
        ]
    );
    assert_eq!(warnings[0].1.message(), "Not one of this notebook's stages");
    // Defining the list, and checking against it, left the text alone.
    assert_eq!(store.note("note:n").unwrap().unwrap().body, body);
    // Fixing the value clears the warning; a note with no list has none.
    note(
        &store,
        "note:fixed",
        "book:s",
        &body.replace("Filming", "Shooting"),
    );
    assert_eq!(store.note_property_warnings("note:fixed").unwrap().len(), 1);
    book(&store, "book:plain", 10, None);
    note(&store, "note:p", "book:plain", body);
    assert!(store.note_property_warnings("note:p").unwrap().is_empty());
    assert!(
        store
            .note_property_warnings("note:none")
            .unwrap()
            .is_empty()
    );
}

#[test]
fn a_property_added_on_each_device_apart_is_kept_on_both() {
    let (_dir, store) = temp_store();
    book(
        &store,
        "book:s",
        1_000,
        Some(json!([
            {"key": "stage", "type": "select", "options": ["Idea"]},
            {"key": "sponsor", "type": "text"}
        ])),
    );
    // The other device renamed the notebook later and added `embargo`.
    let incoming = remote(
        "book:s",
        2_000,
        Some(json!([
            {"key": "embargo", "type": "datetimeZone"},
            {"key": "stage", "type": "select", "options": ["Idea", "Shot"]}
        ])),
    );
    let outcome = store.apply_remote(&incoming, None, None).unwrap();
    assert!(
        matches!(outcome, ApplyOutcome::LocalWon { .. }),
        "{outcome:?}"
    );
    assert_eq!(
        stored_list(&store, "book:s"),
        vec![
            def("embargo", PropertyType::DateTimeZone, &[]),
            def("stage", PropertyType::Select, &["Idea", "Shot"]),
            def("sponsor", PropertyType::Text, &[]),
        ]
    );
    // Both versions are in history, and what is pushed holds the union.
    assert!(store.revisions("book:s").unwrap().len() >= 2);
}

#[test]
fn a_newer_local_definition_wins_and_the_remote_only_property_is_added() {
    let (_dir, store) = temp_store();
    book(
        &store,
        "book:s",
        3_000,
        Some(json!([{"key": "stage", "type": "select", "options": ["Idea"]}])),
    );
    let incoming = remote(
        "book:s",
        2_000,
        Some(json!([
            {"key": "stage", "type": "select", "options": ["Old"]},
            {"key": "owner", "type": "person"}
        ])),
    );
    store.apply_remote(&incoming, None, None).unwrap();
    assert_eq!(
        stored_list(&store, "book:s"),
        vec![
            def("stage", PropertyType::Select, &["Idea"]),
            def("owner", PropertyType::Person, &[]),
        ]
    );
}

#[test]
fn a_list_from_a_newer_version_survives_a_merge_whole() {
    let (_dir, store) = temp_store();
    book(
        &store,
        "book:s",
        1_000,
        Some(json!([{"key": "a", "type": "text"}])),
    );
    let future = json!({"key": "x", "type": "hologram", "shape": [1, 2]});
    let incoming = remote("book:s", 2_000, Some(json!([future])));
    store.apply_remote(&incoming, None, None).unwrap();
    let doc = store.get("book:s").unwrap().unwrap();
    assert!(
        doc.fields["kansoProperties"]
            .as_array()
            .unwrap()
            .contains(&future)
    );
    assert_eq!(
        stored_list(&store, "book:s"),
        vec![def("a", PropertyType::Text, &[])]
    );
}

#[test]
fn the_same_list_on_both_sides_is_the_usual_last_writer_wins() {
    let (_dir, store) = temp_store();
    let list = json!([{"key": "stage", "type": "text"}]);
    book(&store, "book:s", 1_000, Some(list.clone()));
    let mut incoming = remote("book:s", 2_000, Some(list));
    incoming.fields.insert("name".into(), "Renamed".into());
    let outcome = store.apply_remote(&incoming, None, None).unwrap();
    assert!(matches!(outcome, ApplyOutcome::RemoteWon), "{outcome:?}");
    assert_eq!(
        store.get("book:s").unwrap().unwrap().fields["name"],
        "Renamed"
    );
}

fn any_list() -> impl Strategy<Value = Vec<PropertyDef>> {
    prop::collection::btree_map(
        "[a-z]{1,6}",
        (
            prop::sample::select(PropertyType::ALL.to_vec()),
            prop::collection::btree_set("[A-Z][a-z]{1,5}", 0..4),
        ),
        0..6,
    )
    .prop_map(|map| {
        map.into_iter()
            .map(|(key, (kind, options))| PropertyDef {
                key,
                kind,
                options: if kind == PropertyType::Select {
                    options.into_iter().collect()
                } else {
                    Vec::new()
                },
            })
            .collect()
    })
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 24,
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
        ..ProptestConfig::default()
    })]

    /// Whichever device is newer, after sync a notebook defines every
    /// property either device gave it, once each.
    #[test]
    fn prop_synced_lists_hold_every_property_either_side_defined(
        local in any_list(), incoming in any_list(), remote_newer in any::<bool>(),
    ) {
        let (_dir, store) = temp_store();
        let (l_up, r_up) = if remote_newer { (1_000, 2_000) } else { (2_000, 1_000) };
        book(&store, "book:s", l_up, schema::to_json(&local));
        let doc = remote("book:s", r_up, schema::to_json(&incoming));
        store.apply_remote(&doc, None, None).unwrap();
        let after = stored_list(&store, "book:s");
        let mut keys: Vec<&str> = after.iter().map(|d| d.key.as_str()).collect();
        let total = keys.len();
        keys.sort_unstable();
        keys.dedup();
        prop_assert_eq!(keys.len(), total);
        for def in local.iter().chain(&incoming) {
            prop_assert!(keys.contains(&def.key.as_str()), "{} missing", def.key);
        }
        // Where both defined a key, the newer device's definition is kept.
        let (newer, older) = if remote_newer { (&incoming, &local) } else { (&local, &incoming) };
        for def in newer {
            prop_assert!(after.contains(def));
        }
        // And what only the older device had is unchanged.
        for def in older {
            if !newer.iter().any(|n| n.key == def.key) {
                prop_assert!(after.contains(def));
            }
        }
    }
}
