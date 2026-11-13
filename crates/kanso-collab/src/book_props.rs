//! A shared notebook's property list in its manifest (issue #390 › Shared
//! notebooks). The list (`kansoProperties`) is not one field: each property
//! is its own manifest key, `prop:<lowercased key>`, holding that property's
//! definition as JSON, so two people adding or changing different properties
//! both win and only a change to the same property conflicts (the later
//! write wins, as for every manifest field). The rules here are pure; the
//! engine reads and writes the library and the manifest around them.

use std::collections::BTreeMap;

use serde_json::Value as Json;

/// The manifest key prefix of a property.
pub const PROP_PREFIX: &str = "prop:";

/// The manifest key of the property `key`.
pub fn prop_key(key: &str) -> String {
    format!("{PROP_PREFIX}{}", key.to_lowercase())
}

/// A notebook's `kansoProperties` as one entry per property, by manifest
/// key. Entries are kept as stored, so a definition a newer version wrote
/// that this one can't read is carried along whole. An entry without a key,
/// or a second one for a key, is left out.
pub fn split(field: Option<&Json>) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for entry in field.and_then(Json::as_array).into_iter().flatten() {
        let Some(key) = entry.get("key").and_then(Json::as_str) else {
            continue;
        };
        if key.trim().is_empty() {
            continue;
        }
        out.entry(prop_key(key))
            .or_insert_with(|| entry.to_string());
    }
    out
}

/// The property entries among a manifest's (or a notebook's) fields.
pub fn of_fields(fields: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    fields
        .iter()
        .filter(|(key, _)| key.starts_with(PROP_PREFIX))
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect()
}

/// Three-way merge, one property at a time: what changed here since `base`
/// stays, whatever else came from `remote` (a property added, changed or
/// removed there) is taken. Both sides changing one property keeps the local
/// one, which is then written to the manifest and so wins there too.
pub fn merge(
    local: &BTreeMap<String, String>,
    base: &BTreeMap<String, String>,
    remote: &BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for key in local.keys().chain(base.keys()).chain(remote.keys()) {
        let kept = if local.get(key) == base.get(key) {
            remote.get(key)
        } else {
            local.get(key)
        };
        if let Some(value) = kept {
            out.insert(key.clone(), value.clone());
        }
    }
    out
}

/// The list `merged` makes, as `kansoProperties` holds it: the properties of
/// `order` (the list as it is now) keep their places, the others follow by
/// key. `None` for no properties.
pub fn join(merged: &BTreeMap<String, String>, order: Option<&Json>) -> Option<Json> {
    let mut entries: Vec<Json> = Vec::new();
    let mut placed = std::collections::BTreeSet::new();
    for entry in order.and_then(Json::as_array).into_iter().flatten() {
        let Some(key) = entry.get("key").and_then(Json::as_str) else {
            continue;
        };
        let key = prop_key(key);
        if let Some(value) = merged.get(&key)
            && placed.insert(key)
            && let Ok(parsed) = serde_json::from_str(value)
        {
            entries.push(parsed);
        }
    }
    for (key, value) in merged {
        if placed.contains(key) {
            continue;
        }
        if let Ok(parsed) = serde_json::from_str(value) {
            entries.push(parsed);
        }
    }
    (!entries.is_empty()).then_some(Json::Array(entries))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use serde_json::json;

    fn def(key: &str, kind: &str) -> Json {
        json!({"key": key, "type": kind})
    }

    #[test]
    fn a_list_splits_per_property_and_joins_back() {
        let list = json!([def("Stage", "text"), def("Due", "date")]);
        let parts = split(Some(&list));
        assert_eq!(
            parts.keys().map(String::as_str).collect::<Vec<_>>(),
            ["prop:due", "prop:stage"]
        );
        assert_eq!(join(&parts, Some(&list)), Some(list));
        assert_eq!(join(&BTreeMap::new(), None), None);
        // A repeated key and an entry without one are left out.
        let messy = json!([def("a", "text"), def("A", "number"), {"type": "text"}]);
        assert_eq!(split(Some(&messy)).len(), 1);
    }

    #[test]
    fn a_property_one_member_adds_reaches_the_other_and_neither_loses_theirs() {
        let base = BTreeMap::new();
        let mine = split(Some(&json!([def("stage", "text")])));
        let theirs = split(Some(&json!([def("due", "date")])));
        let a = merge(&mine, &base, &theirs);
        assert_eq!(a.len(), 2);
        // The other member, merging with what the first one wrote.
        let b = merge(&theirs, &base, &mine);
        assert_eq!(a, b);
    }

    fn arb_props() -> impl Strategy<Value = BTreeMap<String, String>> {
        prop::collection::btree_map("prop:[a-d]", "[xyz]{1,2}", 0..5)
    }

    proptest! {
        #![proptest_config(crate::tests::config(64))]

        /// With nothing changed here, the notebook is the manifest's.
        #[test]
        fn prop_untouched_local_takes_remote(base in arb_props(), remote in arb_props()) {
            prop_assert_eq!(merge(&base, &base, &remote), remote);
        }

        /// With nothing changed there, the manifest takes what was changed here.
        #[test]
        fn prop_untouched_remote_keeps_local(base in arb_props(), local in arb_props()) {
            prop_assert_eq!(merge(&local, &base, &base), local);
        }

        /// Merging again with the same manifest changes nothing, and a
        /// property changed on one side only is never lost.
        #[test]
        fn prop_merge_is_stable_and_keeps_one_sided_changes(base in arb_props(), local in arb_props(), remote in arb_props()) {
            let once = merge(&local, &base, &remote);
            prop_assert_eq!(merge(&once, &base, &remote), once.clone());
            for key in local.keys().chain(base.keys()).chain(remote.keys()) {
                let (l, b, r) = (local.get(key), base.get(key), remote.get(key));
                if l == b {
                    prop_assert_eq!(once.get(key), r);
                } else if r == b {
                    prop_assert_eq!(once.get(key), l);
                }
            }
        }

        /// Two members who each changed different properties end with the
        /// same list, whichever merged first.
        #[test]
        fn prop_disjoint_changes_converge(base in arb_props(), a_edits in arb_props(), b_edits in arb_props()) {
            let mut a = base.clone();
            let mut b = base.clone();
            for (k, v) in a_edits {
                if !k.ends_with('c') && !k.ends_with('d') {
                    a.insert(k, v);
                }
            }
            for (k, v) in b_edits {
                if k.ends_with('c') || k.ends_with('d') {
                    b.insert(k, v);
                }
            }
            prop_assert_eq!(merge(&a, &base, &b), merge(&b, &base, &a));
        }

        /// Splitting and joining a list gives the same list back, in order,
        /// whatever the keys' case.
        #[test]
        fn prop_split_join_round_trips(keys in prop::collection::vec("[a-d]{1,3}", 0..6)) {
            let mut seen = std::collections::HashSet::new();
            let entries: Vec<Json> = keys
                .iter()
                .filter(|k| seen.insert(k.to_lowercase()))
                .map(|k| def(k, "text"))
                .collect();
            let list = (!entries.is_empty()).then_some(Json::Array(entries));
            prop_assert_eq!(join(&split(list.as_ref()), list.as_ref()), list);
        }
    }
}
