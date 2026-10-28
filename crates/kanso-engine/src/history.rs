//! A note's saved versions: why each was kept, who made it, and what
//! changed since. Pure; the desktop's history dialog and the phones' history
//! screen say the same words and draw the same sections.

pub mod diff;

/// Why a version was kept, from the origin of the write that replaced it.
pub fn origin_label(origin: &str) -> &'static str {
    match origin {
        "ui" => "Edited in Kanso",
        "api" => "Replaced through the API",
        "sync" => "Replaced by sync",
        "import" => "Replaced by an import",
        "collab" => "Edited in a shared note",
        "agent" => "Suggested by an agent, applied by you",
        "plugin" => "Changed by a plugin",
        _ => "Saved version",
    }
}

/// "by user2, user3": who made a version of a shared note, from names
/// listed with commas; each once, in the order first named. `None` when
/// nobody is named. Pure.
pub fn authors_label<'a>(names: impl IntoIterator<Item = &'a str>) -> Option<String> {
    let mut seen: Vec<&str> = Vec::new();
    for name in names.into_iter().flat_map(|n| n.split(',')).map(str::trim) {
        if !name.is_empty() && !seen.contains(&name) {
            seen.push(name);
        }
    }
    (!seen.is_empty()).then(|| format!("by {}", seen.join(", ")))
}

#[cfg(test)]
mod tests {
    use super::{authors_label, origin_label};
    use proptest::prelude::*;

    #[test]
    fn versions_name_who_made_them() {
        assert_eq!(
            authors_label(Some("user2, user3")).as_deref(),
            Some("by user2, user3")
        );
        assert_eq!(
            authors_label(["user2", "user3,user2", " "]).as_deref(),
            Some("by user2, user3")
        );
        assert_eq!(authors_label(None), None);
        assert_eq!(authors_label([" , "]), None);
    }

    proptest! {
        /// Every name given shows once, in the order first named, and only
        /// those.
        #[test]
        fn prop_authors_label_names_each_once(names in prop::collection::vec("[a-z][a-z0-9]{0,6}", 0..6)) {
            let joined = names.join(", ");
            match authors_label([joined.as_str()]) {
                None => prop_assert!(names.is_empty()),
                Some(label) => {
                    let listed: Vec<&str> = label.strip_prefix("by ").unwrap().split(", ").collect();
                    let mut expected: Vec<&str> = Vec::new();
                    for name in &names {
                        if !expected.contains(&name.as_str()) {
                            expected.push(name);
                        }
                    }
                    prop_assert_eq!(listed, expected);
                }
            }
        }
    }

    #[test]
    fn every_origin_has_a_label() {
        for origin in [
            kanso_store::Origin::Ui,
            kanso_store::Origin::Api,
            kanso_store::Origin::Sync,
            kanso_store::Origin::Import,
            kanso_store::Origin::Collab,
            kanso_store::Origin::Agent,
            kanso_store::Origin::Plugin,
        ] {
            assert_ne!(origin_label(origin.as_str()), "Saved version");
        }
    }
}
