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

/// The entries under `### Breaking` in a changelog section: each top-level
/// bullet, joined with the lines that continue it. A bullet nested under an
/// entry (and what follows it, up to the next entry) is detail, not an entry
/// of its own, so it is the entry's own text that has to end with the notice.
fn breaking_entries(section: &str) -> Vec<String> {
    let mut entries: Vec<String> = Vec::new();
    let mut inside = false;
    let mut in_fence = false;
    let mut in_detail = false;
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
        let bullet = trimmed.starts_with("- ") || trimmed.starts_with("* ");
        let indent = line.len() - trimmed.len();
        if bullet && indent < 2 {
            entries.push(trimmed.to_owned());
            in_detail = false;
        } else if bullet {
            in_detail = true;
        } else if !trimmed.is_empty()
            && !in_detail
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
    let release = match semver::Version::parse(version.trim_start_matches('v')) {
        Ok(v) => Some(v),
        Err(_) => {
            problems.push(Problem(format!(
                "\u{201c}{version}\u{201d} isn't a version like 0.10.0 or 0.10.0-beta.1, so no notice's `since` can be checked against it; pass the release's version to --version"
            )));
            None
        }
    };
    for entry in breaking_entries(section) {
        let shown = entry.trim_start_matches(['-', '*', ' ']);
        let Some(id) = notice_id(&entry) else {
            problems.push(Problem(format!(
                "this breaking change has no update notice: \u{201c}{shown}\u{201d}. Add a notice to NOTICES in crates/kanso-core/src/notices.rs, then make the entry end with (notice: <id>): give its commit a \u{201c}Notice: <id>\u{201d} footer (lower-case letters, digits, dots and hyphens; any other ID isn't printed), or, for a commit already on master, write this release's section in CHANGELOG.md ahead and end the entry with that text yourself (docs/RELEASING.md \u{203a} Breaking changes)"
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
    fn a_nested_bullet_is_detail_not_an_entry() {
        let list = [notice("a-1", &["Step."])];
        let nested = "### Breaking\n\n- Keys change (notice: a-1)\n  - Old logins stop working\n  - Scripts need keys\n    still more\n- Second (notice: a-1)\n";
        assert_eq!(check_section(nested, "0.10.0", &list), vec![]);
        // A notice in the detail doesn't stand for the entry's own.
        let in_detail = "### Breaking\n\n- Keys change\n  - Old logins stop (notice: a-1)\n";
        assert_eq!(check_section(in_detail, "0.10.0", &list).len(), 1);
    }

    #[test]
    fn a_version_that_isnt_one_is_refused() {
        let list = [notice("a-1", &["Step."])];
        let ok = section(&["Keys (notice: a-1)"]);
        let problems = check_section(&ok, "next", &list);
        assert_eq!(problems.len(), 1, "{problems:?}");
        assert!(problems[0].0.contains("--version"));
    }

    #[test]
    fn a_prerelease_is_before_its_release() {
        let list = [notice("a-1", &["Step."])];
        let ok = section(&["Keys (notice: a-1)"]);
        // The app shows a notice from `since` on, and 0.10.0-beta.1 comes
        // before 0.10.0, so a beta can't ship the 0.10.0 notice.
        let beta = check_section(&ok, "0.10.0-beta.1", &list);
        assert_eq!(beta.len(), 1);
        assert!(beta[0].0.contains("set `since` to 0.10.0-beta.1 or lower"));
        assert_eq!(check_section(&ok, "v0.10.0", &list), vec![]);
        assert_eq!(check_section(&ok, "0.10.1-rc.1", &list), vec![]);
        assert_eq!(check_section(&ok, "1.0.0", &list), vec![]);
    }

    /// The release's section of a whole changelog, as `check-notices` reads it.
    const LOG: &str = "# Changelog\n\nEvery release of Kanso.\n\n\
        ## [0.10.0](https://x/compare/v0.10.0-beta.1...v0.10.0) - 2026-11-05\n\n\
        ### Breaking\n\n- Keys ([abc1234](https://x/commit/abc1234)) **(breaking)** by @sam (notice: a-1)\n\n\
        ### New\n\n- A feature\n\n\
        ## [0.10.0-beta.1](https://x/compare/v0.9.4...v0.10.0-beta.1) - 2026-11-01\n\n\
        ### Breaking\n\n- Keys, early ([def5678](https://x/commit/def5678)) **(breaking)**\n\n\
        ## [0.9.4](https://x/compare/v0.9.3...v0.9.4) - 2026-10-30\n\n### Fixed\n\n- A crash\n";

    fn gate(version: &str) -> Option<Vec<Problem>> {
        let list = [notice("a-1", &["Step."])];
        crate::notes::section(LOG, version).map(|s| check_section(&s, version, &list))
    }

    #[test]
    fn the_first_section_is_read_alone() {
        // The newest section (where release.sh puts it) stops at the next
        // release's heading, so the beta's entry with no notice isn't its.
        assert_eq!(gate("0.10.0"), Some(vec![]));
        // The beta's own section is found apart from 0.10.0's, and fails.
        assert_eq!(gate("0.10.0-beta.1").map(|p| p.len()), Some(1));
        assert_eq!(gate("0.9.4"), Some(vec![]));
    }

    #[test]
    fn a_missing_section_is_none() {
        // `check-notices` turns this into "has no section for …; run
        // scripts/release.sh first".
        assert_eq!(gate("0.10.1"), None);
        assert_eq!(gate("0.10"), None);
        assert_eq!(gate("0.10.0-beta"), None);
    }

    #[test]
    fn the_shipped_changelog_passes_the_gate() {
        let log = include_str!("../../../CHANGELOG.md");
        let versions: Vec<&str> = log
            .lines()
            .filter_map(|l| l.strip_prefix("## [")?.split(']').next())
            .collect();
        assert!(versions.len() > 5, "{versions:?}");
        for v in versions {
            let section = crate::notes::section(log, v).unwrap();
            assert_eq!(check_section(&section, v, NOTICES), vec![], "{v}");
        }
    }

    /// What one generated Breaking entry is, for the parser property.
    #[derive(Clone, Copy, Debug)]
    enum Entry {
        /// Ends with the known notice.
        Known,
        /// Ends with the known notice, wrapped over two lines.
        Wrapped,
        /// Ends with the known notice, with nested detail under it.
        WithDetail,
        /// Ends with an ID that isn't in the list.
        Unknown,
        /// Has no notice.
        Missing,
        /// Names the notice only in its nested detail.
        OnlyInDetail,
    }

    fn entry_strategy() -> impl Strategy<Value = Entry> {
        prop_oneof![
            Just(Entry::Known),
            Just(Entry::Wrapped),
            Just(Entry::WithDetail),
            Just(Entry::Unknown),
            Just(Entry::Missing),
            Just(Entry::OnlyInDetail),
        ]
    }

    fn render(entry: Entry, text: &str) -> String {
        let link = "([abc1234](https://x/commit/abc1234)) **(breaking)**";
        match entry {
            Entry::Known => format!("- {text} {link} (notice: known-1)\n"),
            Entry::Wrapped => format!("- {text}\n  {link} (notice: known-1)\n"),
            Entry::WithDetail => {
                format!("- {text} {link} (notice: known-1)\n  - detail {text}\n    more\n")
            }
            Entry::Unknown => format!("- {text} {link} (notice: other-2)\n"),
            Entry::Missing => format!("- {text} {link}\n"),
            Entry::OnlyInDetail => format!("- {text} {link}\n  - detail (notice: known-1)\n"),
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            rng_seed: proptest::test_runner::RngSeed::Fixed(0x006b_616e_736f),
            ..ProptestConfig::with_cases(256)
        })]

        /// Over a whole changelog of releases and prereleases, each with its
        /// own groups and entries, the gate on one release's section flags
        /// exactly that release's Breaking entries without a known notice it
        /// can show, and nothing from any other release.
        #[test]
        fn prop_the_gate_reads_exactly_the_releases_breaking_entries(
            releases in prop::collection::vec(
                (
                    any::<bool>(),
                    prop::collection::vec(
                        (prop::sample::select(vec!["New", "Breaking", "Fixed", "Under the hood"]),
                         prop::collection::vec((entry_strategy(), "[a-z]{1,8}( [a-z]{1,8}){0,3}"), 0..4)),
                        0..4,
                    ),
                ),
                1..6,
            ),
            pick in any::<prop::sample::Index>(),
        ) {
            let list = [notice("known-1", &["Step."])];
            // Newest first: 0.11.0, 0.11.0-beta.1, 0.10.0, ... An odd one is
            // the beta of the release above it, so their headings share a
            // prefix. The notice starts at 0.10.0, so the oldest ones (from
            // 0.10.0-beta.1 down) can't show it.
            let version = |i: usize| {
                let minor = 11 - i / 2;
                if i.is_multiple_of(2) { format!("0.{minor}.0") } else { format!("0.{minor}.0-beta.1") }
            };
            let mut log = String::from("# Changelog\n\nEvery release of Kanso.\n\n");
            for (i, (linked, groups)) in releases.iter().enumerate() {
                let v = version(i);
                if *linked {
                    log.push_str(&format!("## [{v}](https://x/compare/a...v{v}) - 2026-11-05\n\n"));
                } else {
                    log.push_str(&format!("## {v} - 2026-11-05\n\n"));
                }
                for (group, entries) in groups {
                    log.push_str(&format!("### {group}\n\n"));
                    for (entry, text) in entries {
                        log.push_str(&render(*entry, text));
                    }
                    log.push('\n');
                }
            }
            let i = pick.index(releases.len());
            let v = version(i);
            let release = semver::Version::parse(&v).unwrap();
            let shown = release >= semver::Version::new(0, 10, 0);
            let expected: usize = releases[i]
                .1
                .iter()
                .filter(|(group, _)| *group == "Breaking")
                .flat_map(|(_, entries)| entries)
                .filter(|(entry, _)| match entry {
                    Entry::Known | Entry::Wrapped | Entry::WithDetail => !shown,
                    Entry::Unknown | Entry::Missing | Entry::OnlyInDetail => true,
                })
                .count();
            let section = crate::notes::section(&log, &v).unwrap();
            prop_assert_eq!(check_section(&section, &v, &list).len(), expected, "{}", section);
        }

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
