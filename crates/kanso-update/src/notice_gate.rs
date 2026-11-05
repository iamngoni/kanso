//! The release gate for update notices (issue #243, "Update notices").
//!
//! A change that breaks something people rely on must ship a notice with the
//! steps to fix it (`kanso_core::notices`). A `CHANGELOG.md` entry under
//! `### Breaking` has to end with `(notice: <id>)`, and the ID has to be in
//! the list the app is built with. [`check_list`] checks the list itself and
//! [`check_section`] checks one release's changelog section against it.
//! `kanso-release check-notices` runs both from `scripts/release.sh` and the
//! release workflow, so a breaking change with no notice cannot ship.
//!
//! The changelog is generated (cliff.toml): a commit marked breaking (`feat!:`
//! or a `BREAKING CHANGE` footer) lands under Breaking, and a `Notice: <id>`
//! footer on it prints the `(notice: <id>)` the gate looks for.

use std::collections::BTreeSet;

use kanso_core::notices::Notice;

/// The changelog heading whose entries each need a notice.
pub const BREAKING: &str = "Breaking";

/// The words in front of a notice's ID at the end of an entry.
const MARKER: &str = "(notice: ";

/// One thing the gate refuses, worded with how to fix it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Problem(pub String);

impl std::fmt::Display for Problem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Whether `id` is a well-formed notice ID: lower-case letters, digits,
/// dots and hyphens, starting with a letter or digit (`api-keys-0.10`).
pub fn is_valid_id(id: &str) -> bool {
    let ok = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit();
    id.chars().next().is_some_and(ok) && id.chars().all(|c| ok(c) || c == '.' || c == '-')
}

/// What is wrong with the notices the app is built with: an ID that is
/// malformed or repeated, or a notice with no title, no explanation or no
/// steps to follow.
pub fn check_list(list: &[Notice]) -> Vec<Problem> {
    let mut problems = Vec::new();
    let mut seen = BTreeSet::new();
    for notice in list {
        let id = notice.id;
        if !is_valid_id(id) {
            problems.push(Problem(format!(
                "notice \u{201c}{id}\u{201d} has an ID that isn't lower-case letters, digits, dots and hyphens; rename it in crates/kanso-core/src/notices.rs"
            )));
        }
        if !seen.insert(id) {
            problems.push(Problem(format!(
                "notice ID \u{201c}{id}\u{201d} is used twice; IDs are never reused, so give the newer notice its own in crates/kanso-core/src/notices.rs"
            )));
        }
        if notice.title.trim().is_empty() {
            problems.push(Problem(format!(
                "notice {id} has no title; add one in crates/kanso-core/src/notices.rs"
            )));
        }
        if notice.what_changed.trim().is_empty() {
            problems.push(Problem(format!(
                "notice {id} doesn't say what changed; fill in `what_changed` in crates/kanso-core/src/notices.rs"
            )));
        }
        if notice.steps.is_empty() || notice.steps.iter().any(|s| s.trim().is_empty()) {
            problems.push(Problem(format!(
                "notice {id} has no steps to fix what broke, or an empty one; give it numbered `steps` in crates/kanso-core/src/notices.rs"
            )));
        }
    }
    problems
}

/// The entries under `### Breaking` in a changelog section: each bullet
/// line, joined with the lines that continue it.
fn breaking_entries(section: &str) -> Vec<String> {
    let mut entries: Vec<String> = Vec::new();
    let mut inside = false;
    let mut in_fence = false;
    for line in section.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        let hashes = trimmed.bytes().take_while(|b| *b == b'#').count();
        if (1..=3).contains(&hashes) && trimmed[hashes..].starts_with(' ') {
            inside = hashes == 3 && trimmed[hashes..].trim().eq_ignore_ascii_case(BREAKING);
            continue;
        }
        if !inside {
            continue;
        }
        if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            entries.push(trimmed.to_owned());
        } else if !trimmed.is_empty()
            && let Some(last) = entries.last_mut()
        {
            last.push(' ');
            last.push_str(trimmed);
        }
    }
    entries
}

/// The ID an entry ends with, when it ends `(notice: <id>)`.
pub fn notice_id(entry: &str) -> Option<&str> {
    let body = entry.trim_end().strip_suffix(')')?;
    let at = body.rfind(MARKER)?;
    let id = body[at + MARKER.len()..].trim();
    (!id.is_empty() && !id.contains(['(', ')'])).then_some(id)
}

/// What is wrong with one release's changelog section (`version` is the
/// release's): a Breaking entry that doesn't end with `(notice: <id>)`, an ID
/// that isn't in `list`, or a notice that the release would not yet show.
pub fn check_section(section: &str, version: &str, list: &[Notice]) -> Vec<Problem> {
    let mut problems = Vec::new();
    let release = semver::Version::parse(version.trim_start_matches('v')).ok();
    for entry in breaking_entries(section) {
        let shown = entry.trim_start_matches(['-', '*', ' ']);
        let Some(id) = notice_id(&entry) else {
            problems.push(Problem(format!(
                "this breaking change has no update notice: \u{201c}{shown}\u{201d}. Add a notice to NOTICES in crates/kanso-core/src/notices.rs, then end the entry with (notice: <id>); for a generated entry, add a \u{201c}Notice: <id>\u{201d} footer to its commit message"
            )));
            continue;
        };
        match list.iter().find(|n| n.id == id) {
            None => problems.push(Problem(format!(
                "this entry names notice \u{201c}{id}\u{201d}, which isn't in NOTICES: \u{201c}{shown}\u{201d}. Add it in crates/kanso-core/src/notices.rs, or correct the ID"
            ))),
            Some(notice) => {
                if release.as_ref().is_some_and(|v| *v < notice.since) {
                    problems.push(Problem(format!(
                        "notice {id} starts at {} but this release is {version}, so nobody on {version} would see it; set `since` to {version} or lower in crates/kanso-core/src/notices.rs",
                        notice.since
                    )));
                }
            }
        }
    }
    problems
}

#[cfg(test)]
mod tests {
    use super::*;
    use kanso_core::notices::{NOTICES, NoticeContext, Platforms};
    use proptest::prelude::*;

    fn notice(id: &'static str, steps: &'static [&'static str]) -> Notice {
        Notice {
            id,
            since: semver::Version::new(0, 10, 0),
            title: "Title",
            what_changed: "What changed",
            steps,
            action: None,
            applies: |_: &NoticeContext| true,
            platforms: Platforms::Both,
        }
    }

    fn section(entries: &[&str]) -> String {
        let mut s =
            String::from("## [0.10.0](x) - 2026-11-05\n\n### New\n\n- A feature (abc1234)\n");
        if !entries.is_empty() {
            s.push_str("\n### Breaking\n\n");
            for e in entries {
                s.push_str(&format!("- {e}\n"));
            }
        }
        s.push_str("\n### Fixed\n\n- A crash (def5678)\n");
        s
    }

    #[test]
    fn the_shipped_list_passes() {
        assert_eq!(check_list(NOTICES), vec![]);
    }

    #[test]
    fn a_bad_list_is_refused_with_the_fix() {
        let list = [
            notice("a-1", &["Do this."]),
            notice("a-1", &["Do this."]),
            notice("Bad ID", &["Do this."]),
            notice("no-steps", &[]),
            notice("blank-step", &["  "]),
        ];
        let problems = check_list(&list);
        assert_eq!(problems.len(), 4, "{problems:?}");
        assert!(problems.iter().all(|p| p.0.contains("notices.rs")));
    }

    #[test]
    fn a_breaking_entry_needs_its_notice() {
        let list = [notice("api-keys-0.10", &["Step."])];
        let ok = section(&["Keys (abc) (notice: api-keys-0.10)"]);
        assert_eq!(check_section(&ok, "0.10.0", &list), vec![]);
        assert_eq!(check_section(&section(&[]), "0.10.0", &list), vec![]);
        let missing = check_section(&section(&["Keys (abc)"]), "0.10.0", &list);
        assert_eq!(missing.len(), 1);
        assert!(missing[0].0.contains("Notice: <id>"));
        let unknown = check_section(&section(&["Keys (notice: nope)"]), "0.10.0", &list);
        assert!(unknown[0].0.contains("isn't in NOTICES"));
        let early = check_section(&ok, "0.9.4", &list);
        assert!(early[0].0.contains("starts at 0.10.0"));
    }

    #[test]
    fn the_id_must_end_the_entry_and_headings_are_respected() {
        let list = [notice("a-1", &["Step."])];
        // A mention in the middle of an entry doesn't count.
        let middle = section(&["Keys (notice: a-1) more words"]);
        assert_eq!(check_section(&middle, "0.10.0", &list).len(), 1);
        // A continuation line carries the ID.
        let wrapped =
            "### Breaking\n\n- Keys change\n  and more (notice: a-1)\n\n### New\n\n- Plain\n";
        assert_eq!(check_section(wrapped, "0.10.0", &list), vec![]);
        // The same words in a code fence are text.
        let fenced = "### New\n\n```\n### Breaking\n- x\n```\n";
        assert_eq!(check_section(fenced, "0.10.0", &list), vec![]);
    }

    #[test]
    fn the_shipped_changelog_passes_the_gate() {
        let log = include_str!("../../../CHANGELOG.md");
        let top = crate::notes::section(log, "0.1.0").unwrap();
        assert_eq!(check_section(&top, "0.1.0", NOTICES), vec![]);
    }

    proptest! {
        /// Whatever the entries, the gate flags exactly those that don't end
        /// with a known notice's ID.
        #[test]
        fn prop_flags_exactly_the_entries_without_a_known_notice(
            entries in prop::collection::vec(("[a-z]{1,8}( [a-z]{1,8}){0,3}", 0u8..3), 0..8)
        ) {
            let list = [notice("known-1", &["Step."])];
            let lines: Vec<String> = entries
                .iter()
                .map(|(text, kind)| match kind {
                    0 => format!("{text} (notice: known-1)"),
                    1 => format!("{text} (notice: other-2)"),
                    _ => text.clone(),
                })
                .collect();
            let refs: Vec<&str> = lines.iter().map(String::as_str).collect();
            let bad = entries.iter().filter(|(_, k)| *k != 0).count();
            prop_assert_eq!(check_section(&section(&refs), "0.10.0", &list).len(), bad);
        }

        /// IDs from the allowed alphabet are valid; anything with another
        /// character is not.
        #[test]
        fn prop_id_alphabet(id in "[a-z0-9][a-z0-9.-]{0,20}", bad in "[A-Z _/]") {
            prop_assert!(is_valid_id(&id));
            let spoiled = format!("{id}{bad}");
            prop_assert!(!is_valid_id(&spoiled));
        }
    }
}
