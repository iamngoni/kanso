//! A notebook's property list (`kansoProperties`): the properties its notes
//! are meant to have, each with a type.
//!
//! The list never rewrites a note. A value that does not fit its property's
//! type stays exactly as written and is shown as text with a warning
//! ([`check`]). The list travels with the notebook through sync, where two
//! versions made apart are united one property at a time ([`merge`]).

use std::collections::HashSet;

use serde_json::{Map, Value as Json, json};

use crate::dates::Date;
use crate::properties::{self, Value};
pub use crate::schema_model::{
    FIELD, MAX_OPTIONS, MAX_PROPERTIES, Mismatch, PropertyDef, PropertyType, SchemaError,
};
use crate::zoned;

/// The property list a notebook's `kansoProperties` holds. A malformed
/// entry (no key, a type this version does not know) is skipped, and a
/// second definition of a key is ignored, so a list written by a newer
/// version never stops this one reading the rest.
pub fn from_json(field: Option<&Json>) -> Vec<PropertyDef> {
    let Some(Json::Array(entries)) = field else {
        return Vec::new();
    };
    let mut seen = HashSet::new();
    let mut defs = Vec::new();
    for entry in entries {
        let Some(key) = entry.get("key").and_then(Json::as_str) else {
            continue;
        };
        let Some(kind) = entry
            .get("type")
            .and_then(Json::as_str)
            .and_then(PropertyType::parse)
        else {
            continue;
        };
        if key.trim().is_empty() || !seen.insert(key.to_lowercase()) {
            continue;
        }
        let mut options: Vec<String> = Vec::new();
        if kind == PropertyType::Select
            && let Some(listed) = entry.get("options").and_then(Json::as_array)
        {
            for option in listed.iter().filter_map(Json::as_str) {
                let option = option.trim();
                if !option.is_empty() && !options.iter().any(|o| o == option) {
                    options.push(option.to_owned());
                }
            }
        }
        defs.push(PropertyDef {
            key: key.to_owned(),
            kind,
            options,
        });
    }
    defs
}

/// The `kansoProperties` field for `defs`: `None` for no properties.
pub fn to_json(defs: &[PropertyDef]) -> Option<Json> {
    if defs.is_empty() {
        return None;
    }
    Some(Json::Array(
        defs.iter()
            .map(|def| {
                let mut entry = Map::new();
                entry.insert("key".into(), json!(def.key));
                entry.insert("type".into(), json!(def.kind.as_str()));
                if def.kind == PropertyType::Select {
                    entry.insert("options".into(), json!(def.options));
                }
                Json::Object(entry)
            })
            .collect(),
    ))
}

/// Whether `defs` can be saved as a notebook's property list.
pub fn validate(defs: &[PropertyDef]) -> Result<(), SchemaError> {
    if defs.len() > MAX_PROPERTIES {
        return Err(SchemaError::TooManyProperties);
    }
    let mut seen = HashSet::new();
    for def in defs {
        if def.key.trim().is_empty() {
            return Err(SchemaError::EmptyKey);
        }
        if !seen.insert(def.key.to_lowercase()) {
            return Err(SchemaError::DuplicateKey(def.key.clone()));
        }
        if def.kind != PropertyType::Select && !def.options.is_empty() {
            return Err(SchemaError::OptionsOnNonSelect(def.key.clone()));
        }
        if def.options.len() > MAX_OPTIONS {
            return Err(SchemaError::TooManyOptions(def.key.clone()));
        }
        let mut options = HashSet::new();
        for option in &def.options {
            if option.trim().is_empty() {
                return Err(SchemaError::EmptyOption(def.key.clone()));
            }
            if !options.insert(option.as_str()) {
                return Err(SchemaError::DuplicateOption(
                    def.key.clone(),
                    option.clone(),
                ));
            }
        }
    }
    Ok(())
}

/// Why `value` does not fit `def`, or `None` when it does. A value that says
/// nothing (an empty field) fits every type: the property is just not set.
pub fn check(def: &PropertyDef, value: &Value) -> Option<Mismatch> {
    if value.is_empty() {
        return None;
    }
    let fits = match def.kind {
        PropertyType::Text => matches!(
            value,
            Value::Text(_)
                | Value::Number(_)
                | Value::Bool(_)
                | Value::Date(_)
                | Value::DateTime(_)
                | Value::DateTimeZone(_)
        ),
        PropertyType::Number => matches!(value, Value::Number(_)),
        PropertyType::Bool => matches!(value, Value::Bool(_)),
        PropertyType::Date => matches!(value, Value::Date(_)),
        PropertyType::DateTime => matches!(value, Value::DateTime(_)),
        PropertyType::DateTimeZone => matches!(value, Value::DateTimeZone(_)),
        PropertyType::Select => {
            return match value {
                Value::Text(_)
                | Value::Number(_)
                | Value::Bool(_)
                | Value::Date(_)
                | Value::DateTime(_)
                | Value::DateTimeZone(_)
                    if def.options.contains(&value.text()) =>
                {
                    None
                }
                _ => Some(Mismatch::NotAnOption {
                    key: def.key.clone(),
                }),
            };
        }
        PropertyType::List => matches!(value, Value::List(_)),
        PropertyType::Person => matches!(value, Value::Text(_)),
    };
    (!fits).then(|| match def.kind {
        PropertyType::Text | PropertyType::Person => Mismatch::NotText,
        PropertyType::Number => Mismatch::NotANumber,
        PropertyType::Bool => Mismatch::NotYesNo,
        PropertyType::Date => Mismatch::NotADate,
        PropertyType::DateTime => Mismatch::NotADateTime,
        PropertyType::DateTimeZone => Mismatch::NotADateTimeZone,
        PropertyType::Select => Mismatch::NotAnOption {
            key: def.key.clone(),
        },
        PropertyType::List => Mismatch::NotAList,
    })
}

/// The warnings for a note's properties under `defs`: each property the list
/// defines whose value does not fit, in the note's order. Properties the
/// list does not define are not checked.
pub fn check_all(defs: &[PropertyDef], properties: &[(String, Value)]) -> Vec<(String, Mismatch)> {
    properties
        .iter()
        .filter_map(|(key, value)| {
            let def = defs.iter().find(|d| d.key.eq_ignore_ascii_case(key))?;
            check(def, value).map(|mismatch| (key.clone(), mismatch))
        })
        .collect()
}

/// The value a field of type `kind` should set for the text typed into it.
/// Text that does not fit is kept as text, never dropped or rewritten: the
/// warning shows it and the person fixes it.
pub fn coerce(kind: PropertyType, text: &str) -> Value {
    let text = text.trim();
    if text.is_empty() {
        return Value::Text(String::new());
    }
    let kept = || Value::Text(text.to_owned());
    match kind {
        PropertyType::Text | PropertyType::Select | PropertyType::Person => kept(),
        PropertyType::Number => text
            .parse::<f64>()
            .ok()
            .filter(|n| n.is_finite())
            .map_or_else(kept, Value::Number),
        PropertyType::Bool => match text.to_ascii_lowercase().as_str() {
            "true" | "yes" => Value::Bool(true),
            "false" | "no" => Value::Bool(false),
            _ => kept(),
        },
        PropertyType::Date => Date::parse_iso(text).map_or_else(kept, |d| Value::Date(d.iso())),
        PropertyType::DateTime => {
            zoned::parse_local(text).map_or_else(kept, |at| Value::DateTime(zoned::local_text(&at)))
        }
        PropertyType::DateTimeZone => {
            zoned::parse_zoned(text).map_or_else(kept, |at| Value::DateTimeZone(at.text()))
        }
        PropertyType::List => properties::typed(text, Some(&Value::List(Vec::new()))),
    }
}

/// Two versions of a notebook's property list made apart, united one
/// property at a time: every property either side defines is kept, and where
/// both define one, the newer side's definition wins. The newer side's order
/// comes first. A property removed on one side and untouched on the other
/// comes back, since the two versions share no history to say who removed it.
///
/// Entries are united as stored, so a definition a newer version wrote that
/// this one cannot read is carried along whole.
pub fn merge(local: Option<&Json>, remote: Option<&Json>, remote_newer: bool) -> Option<Json> {
    let (first, second) = if remote_newer {
        (remote, local)
    } else {
        (local, remote)
    };
    let entries = |field: Option<&Json>| -> Vec<Json> {
        field.and_then(Json::as_array).cloned().unwrap_or_default()
    };
    let key_of = |entry: &Json| {
        entry
            .get("key")
            .and_then(Json::as_str)
            .map(str::to_lowercase)
    };
    let mut merged = entries(first);
    let mut seen: HashSet<String> = merged.iter().filter_map(key_of).collect();
    for entry in entries(second) {
        match key_of(&entry) {
            Some(key) if seen.contains(&key) => {}
            Some(key) => {
                seen.insert(key);
                merged.push(entry);
            }
            None => merged.push(entry),
        }
    }
    (!merged.is_empty()).then_some(Json::Array(merged))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

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

    #[test]
    fn the_studio_list_reads_back_from_its_stored_form() {
        let stored = json!([
            {"key": "stage", "type": "select", "options": ["Idea","Scripting","Shooting","Editing","Published"]},
            {"key": "embargo", "type": "datetimeZone"},
            {"key": "publish", "type": "datetime"},
            {"key": "sponsor", "type": "text"},
            {"key": "owner", "type": "person"}
        ]);
        assert_eq!(from_json(Some(&stored)), studio());
        assert_eq!(to_json(&studio()), Some(stored));
        assert_eq!(to_json(&[]), None);
    }

    #[test]
    fn a_list_this_version_cannot_fully_read_is_read_as_far_as_it_goes() {
        let stored = json!([
            {"key": "a", "type": "hologram"},
            {"type": "text"},
            "nope",
            {"key": " ", "type": "text"},
            {"key": "b", "type": "text"},
            {"key": "B", "type": "number"},
            {"key": "c", "type": "select", "options": [" x ", "x", "", 4, "y"]}
        ]);
        assert_eq!(
            from_json(Some(&stored)),
            vec![
                def("b", PropertyType::Text, &[]),
                def("c", PropertyType::Select, &["x", "y"]),
            ]
        );
        assert!(from_json(None).is_empty());
        assert!(from_json(Some(&json!({"key": "a"}))).is_empty());
    }

    #[test]
    fn a_list_is_refused_when_it_would_confuse_a_note() {
        assert_eq!(validate(&studio()), Ok(()));
        let one = |d| validate(&[d]);
        assert_eq!(
            one(def(" ", PropertyType::Text, &[])),
            Err(SchemaError::EmptyKey)
        );
        assert_eq!(
            validate(&[
                def("a", PropertyType::Text, &[]),
                def("A", PropertyType::Number, &[])
            ]),
            Err(SchemaError::DuplicateKey("A".into()))
        );
        assert_eq!(
            one(def("a", PropertyType::Text, &["x"])),
            Err(SchemaError::OptionsOnNonSelect("a".into()))
        );
        assert_eq!(
            one(def("a", PropertyType::Select, &["x", "x"])),
            Err(SchemaError::DuplicateOption("a".into(), "x".into()))
        );
        assert_eq!(
            one(def("a", PropertyType::Select, &[""])),
            Err(SchemaError::EmptyOption("a".into()))
        );
    }

    #[test]
    fn a_value_that_does_not_fit_gets_the_warning_and_stays_as_written() {
        let defs = studio();
        let stage = &defs[0];
        assert_eq!(check(stage, &Value::Text("Shooting".into())), None);
        let wrong = check(stage, &Value::Text("Filming".into())).unwrap();
        assert_eq!(wrong.message(), "Not one of this notebook's stages");
        assert_eq!(check(stage, &Value::Text(String::new())), None);
        // The embargo wants a zone; a bare date-time or a made-up zone is
        // only text.
        let embargo = &defs[1];
        let note = "---\nembargo: 2026-11-18T09:00\npublish: 2026-11-21T15:00 Mars/Olympus\n---\n";
        let props = properties::parse(note);
        let warnings = check_all(&defs, &props);
        assert_eq!(
            warnings,
            vec![
                ("embargo".to_owned(), Mismatch::NotADateTimeZone),
                ("publish".to_owned(), Mismatch::NotADateTime),
            ]
        );
        assert_eq!(
            check(
                embargo,
                &Value::DateTimeZone("2026-11-18T09:00 America/New_York".into())
            ),
            None
        );
        // Nothing was rewritten: the note is as it was.
        assert_eq!(
            properties::set_property(note, "stage", &Value::Text("Idea".into())),
            "---\nembargo: 2026-11-18T09:00\npublish: 2026-11-21T15:00 Mars/Olympus\nstage: Idea\n---\n"
        );
    }

    #[test]
    fn typed_input_that_does_not_fit_is_kept_as_text() {
        use PropertyType::*;
        assert_eq!(coerce(Number, "4.5"), Value::Number(4.5));
        assert_eq!(coerce(Number, "four"), Value::Text("four".into()));
        assert_eq!(coerce(Bool, "Yes"), Value::Bool(true));
        assert_eq!(coerce(Bool, "maybe"), Value::Text("maybe".into()));
        assert_eq!(coerce(Date, "2026-11-18"), Value::Date("2026-11-18".into()));
        assert_eq!(coerce(Date, "tomorrow"), Value::Text("tomorrow".into()));
        assert_eq!(
            coerce(DateTime, "2026-11-18T09:00"),
            Value::DateTime("2026-11-18T09:00".into())
        );
        assert_eq!(
            coerce(DateTimeZone, "2026-11-18T09:00 Mars/Olympus"),
            Value::Text("2026-11-18T09:00 Mars/Olympus".into())
        );
        assert_eq!(coerce(DateTimeZone, "  "), Value::Text(String::new()));
    }

    #[test]
    fn two_lists_made_apart_unite_one_property_at_a_time() {
        let local = json!([
            {"key": "stage", "type": "select", "options": ["Idea"]},
            {"key": "sponsor", "type": "text"}
        ]);
        let remote = json!([
            {"key": "embargo", "type": "datetimeZone"},
            {"key": "Stage", "type": "select", "options": ["Idea", "Shot"]}
        ]);
        let keys = |v: &Json| -> Vec<String> {
            v.as_array()
                .unwrap()
                .iter()
                .map(|e| e["key"].as_str().unwrap().to_owned())
                .collect()
        };
        // The newer remote list comes first and wins on `stage`.
        let merged = merge(Some(&local), Some(&remote), true).unwrap();
        assert_eq!(keys(&merged), ["embargo", "Stage", "sponsor"]);
        assert_eq!(merged[1]["options"], json!(["Idea", "Shot"]));
        // A newer local list keeps its own `stage`.
        let merged = merge(Some(&local), Some(&remote), false).unwrap();
        assert_eq!(keys(&merged), ["stage", "sponsor", "embargo"]);
        assert_eq!(merged[0]["options"], json!(["Idea"]));
        // A side with no list adds nothing and loses nothing.
        assert_eq!(merge(Some(&local), None, true), Some(local.clone()));
        assert_eq!(merge(None, None, true), None);
        // A definition this version cannot read is carried whole.
        let future = json!([{"key": "x", "type": "hologram", "shape": 1}]);
        assert_eq!(
            merge(Some(&local), Some(&future), false).unwrap()[2],
            future[0]
        );
    }

    fn any_kind() -> impl Strategy<Value = PropertyType> {
        prop::sample::select(PropertyType::ALL.to_vec())
    }

    fn any_def() -> impl Strategy<Value = PropertyDef> {
        (
            "[a-z][a-z0-9_ ]{0,8}[a-z0-9]",
            any_kind(),
            prop::collection::btree_set("[A-Za-z][A-Za-z ]{0,8}[A-Za-z]", 0..6),
        )
            .prop_map(|(key, kind, options)| PropertyDef {
                key,
                kind,
                options: if kind == PropertyType::Select {
                    options.into_iter().collect()
                } else {
                    Vec::new()
                },
            })
    }

    fn any_list() -> impl Strategy<Value = Vec<PropertyDef>> {
        prop::collection::vec(any_def(), 0..8).prop_map(|defs| {
            let mut seen = HashSet::new();
            defs.into_iter()
                .filter(|d| seen.insert(d.key.to_lowercase()))
                .collect()
        })
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        })]

        /// A saved list reads back as itself.
        #[test]
        fn prop_a_property_list_round_trips(defs in any_list()) {
            prop_assert_eq!(validate(&defs), Ok(()));
            let stored = to_json(&defs);
            prop_assert_eq!(from_json(stored.as_ref()), defs);
        }

        /// Uniting never loses a property either side has, keeps one entry
        /// per key, and a list united with itself is itself.
        #[test]
        fn prop_merge_keeps_every_key_once(
            a in any_list(), b in any_list(), remote_newer in any::<bool>(),
        ) {
            let (ja, jb) = (to_json(&a), to_json(&b));
            let merged = merge(ja.as_ref(), jb.as_ref(), remote_newer);
            let keys: Vec<String> = from_json(merged.as_ref())
                .iter()
                .map(|d| d.key.to_lowercase())
                .collect();
            let unique: HashSet<_> = keys.iter().cloned().collect();
            prop_assert_eq!(unique.len(), keys.len());
            for def in a.iter().chain(&b) {
                prop_assert!(unique.contains(&def.key.to_lowercase()));
            }
            // The newer side's definitions win where both have one.
            let (newer, _) = if remote_newer { (&b, &a) } else { (&a, &b) };
            let merged_defs = from_json(merged.as_ref());
            for def in newer {
                prop_assert!(merged_defs.contains(def));
            }
            prop_assert_eq!(merge(ja.as_ref(), ja.as_ref(), true), ja);
        }

        /// Text typed into a field of any type is never lost: it comes back
        /// as the typed value or as the same text.
        #[test]
        fn prop_coerce_never_drops_text(kind in any_kind(), text in "[ -~]{0,24}") {
            let value = coerce(kind, &text);
            let trimmed = text.trim();
            match &value {
                Value::Text(kept) => prop_assert_eq!(kept.as_str(), trimmed),
                other => {
                    let written = properties::set_property("", "k", other);
                    let back = properties::parse(&written);
                    prop_assert_eq!(properties::get(&back, "k"), Some(other));
                }
            }
        }
    }
}
