//! Release notes as the app shows them.
//!
//! The release page and `latest.json` carry the same changelog section
//! (cliff.toml). In the app, "Under the hood" (builds, CI, tests, docs) is
//! left out: it is there on the release page, which the sheet links to.

/// The heading of the section the app leaves out.
pub const UNDER_THE_HOOD: &str = "Under the hood";

/// `notes` without its "Under the hood" section, from that heading to the
/// next heading of the same level or above. Whitespace at the ends goes.
pub fn for_app(notes: &str) -> String {
    let mut out = Vec::new();
    let mut skipping: Option<usize> = None;
    let mut in_fence = false;
    for line in notes.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
        }
        let level = if in_fence { None } else { heading_level(line) };
        if let (Some(depth), Some(level)) = (skipping, level)
            && level <= depth
        {
            skipping = None;
        }
        if skipping.is_none()
            && let Some(level) = level
            && heading_text(line).eq_ignore_ascii_case(UNDER_THE_HOOD)
        {
            skipping = Some(level);
            continue;
        }
        if skipping.is_none() {
            out.push(line);
        }
    }
    out.join("\n").trim().to_owned()
}

/// The section of a changelog for `version` (with or without a leading
/// `v`): from its `## ` heading, whose text starts with the version or
/// `[version]`, to the next `## `. `None` when there is none.
pub fn section(changelog: &str, version: &str) -> Option<String> {
    let version = version.trim_start_matches('v');
    let mut out: Vec<&str> = Vec::new();
    let mut inside = false;
    let mut in_fence = false;
    for line in changelog.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
        }
        if !in_fence && heading_level(line) == Some(2) {
            if inside {
                break;
            }
            let text = heading_text(line);
            let named = text
                .strip_prefix('[')
                .unwrap_or(text)
                .trim_start_matches('v');
            inside = named
                .strip_prefix(version)
                .is_some_and(|rest| rest.is_empty() || rest.starts_with([']', ' ']));
        }
        if inside {
            out.push(line);
        }
    }
    (!out.is_empty()).then(|| out.join("\n").trim().to_owned())
}

/// The sections of the releases before `version` in a changelog, newest
/// first, at most `max`: each one's version, its date (the heading's text
/// after ` - `, or empty) and its section.
pub fn earlier_sections(
    changelog: &str,
    version: &str,
    max: usize,
) -> Vec<(String, String, String)> {
    let version = version.trim_start_matches('v');
    let Ok(current) = semver::Version::parse(version) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    let mut in_fence = false;
    for line in changelog.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
        }
        if in_fence || heading_level(line) != Some(2) {
            continue;
        }
        let text = heading_text(line);
        let named = text
            .strip_prefix('[')
            .unwrap_or(text)
            .trim_start_matches('v');
        let end = named.find([']', ' ']).unwrap_or(named.len());
        let Ok(found) = semver::Version::parse(&named[..end]) else {
            continue;
        };
        if found >= current {
            continue;
        }
        let date = text
            .rsplit_once(" - ")
            .map(|(_, d)| d.trim().to_owned())
            .unwrap_or_default();
        if let Some(section) = section(changelog, &found.to_string()) {
            out.push((found.to_string(), date, section));
        }
        if out.len() >= max {
            break;
        }
    }
    out
}

/// Notes without the release's own `## ` heading, which the version chip
/// and date say instead. Pure.
pub fn without_heading(notes: &str) -> String {
    let trimmed = notes.trim_start();
    match trimmed.strip_prefix("## ") {
        Some(rest) => rest
            .split_once('\n')
            .map(|(_, body)| body.trim().to_owned())
            .unwrap_or_default(),
        None => notes.trim().to_owned(),
    }
}

/// One line saying what a release holds, from its notes' sections: "2 new
/// features, 1 improvement and 3 fixes." Empty when it has none of them.
/// Pure.
pub fn summary(notes: &str) -> String {
    let mut counts = [0usize; 4];
    let mut section: Option<usize> = None;
    for line in notes.lines() {
        if let Some(heading) = line.strip_prefix("### ") {
            section = match heading.trim().to_ascii_lowercase().as_str() {
                "breaking" => Some(0),
                "new" => Some(1),
                "improved" => Some(2),
                "fixed" => Some(3),
                _ => None,
            };
        } else if line.starts_with("## ") || line.starts_with("# ") {
            section = None;
        } else if let Some(ix) = section
            && (line.starts_with("- ") || line.starts_with("* "))
        {
            counts[ix] += 1;
        }
    }
    let words = [
        ("breaking change", "breaking changes"),
        ("new feature", "new features"),
        ("improvement", "improvements"),
        ("fix", "fixes"),
    ];
    let parts: Vec<String> = counts
        .iter()
        .zip(words)
        .filter(|(n, _)| **n > 0)
        .map(|(n, (one, many))| format!("{n} {}", if *n == 1 { one } else { many }))
        .collect();
    match parts.as_slice() {
        [] => String::new(),
        [one] => format!("{one}."),
        [rest @ .., last] => format!("{} and {last}.", rest.join(", ")),
    }
}

/// A release's notes as plain lists for a screen without Markdown: each
/// `### ` section's title and its top-level items, with links reduced to
/// their text and commit links (`([abc1234](…))`) left out. Pure.
pub fn plain_sections(notes: &str) -> Vec<(String, Vec<String>)> {
    let mut out: Vec<(String, Vec<String>)> = Vec::new();
    for line in for_app(notes).lines() {
        if let Some(heading) = line.strip_prefix("### ") {
            out.push((heading.trim().to_owned(), Vec::new()));
        } else if let Some(item) = line.strip_prefix("- ").or_else(|| line.strip_prefix("* "))
            && let Some((_, items)) = out.last_mut()
        {
            let text = plain_text(item);
            if !text.is_empty() {
                items.push(text);
            }
        }
    }
    out.retain(|(_, items)| !items.is_empty());
    out
}

/// `text` with `[label](url)` as `label`, `(label)` groups that held only a
/// commit link removed, and `**bold**` markers dropped.
fn plain_text(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(open) = rest.find('[') {
        let Some(close) = rest[open..].find("](").map(|i| open + i) else {
            break;
        };
        let Some(end) = rest[close..].find(')').map(|i| close + i) else {
            break;
        };
        let label = &rest[open + 1..close];
        let before = &rest[..open];
        // A commit link wrapped in its own parentheses: `([abc1234](url))`.
        let is_commit = label.len() >= 7 && label.bytes().all(|b| b.is_ascii_hexdigit());
        if is_commit && before.ends_with('(') && rest[end + 1..].starts_with(')') {
            out.push_str(before.trim_end_matches('(').trim_end());
            rest = &rest[end + 2..];
        } else {
            out.push_str(before);
            out.push_str(label);
            rest = &rest[end + 1..];
        }
    }
    out.push_str(rest);
    out.replace("**", "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// The level of an ATX heading (`## Title` is 2).
fn heading_level(line: &str) -> Option<usize> {
    let hashes = line.bytes().take_while(|b| *b == b'#').count();
    ((1..=6).contains(&hashes) && line[hashes..].starts_with(' ')).then_some(hashes)
}

fn heading_text(line: &str) -> &str {
    line.trim_start_matches('#').trim()
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    const NOTES: &str = "## [0.2.0](https://github.com/iamngoni/kanso/compare/v0.1.0...v0.2.0) - 2026-10-20\n\n### New\n\n- Updates install themselves ([abc1234](https://github.com/iamngoni/kanso/commit/abc1234))\n\n### Under the hood\n\n- Pin git-cliff ([def5678](https://github.com/iamngoni/kanso/commit/def5678))\n\n### Fixed\n\n- A crash\n";

    #[test]
    fn summaries_count_what_a_release_holds() {
        assert_eq!(
            summary("### New\n\n- a\n- b\n\n### Improved\n\n- c\n\n### Fixed\n\n- d\n- e\n- f\n"),
            "2 new features, 1 improvement and 3 fixes."
        );
        assert_eq!(summary("### Fixed\n\n- one\n"), "1 fix.");
        assert_eq!(summary("### Under the hood\n\n- x\n"), "");
        assert_eq!(summary(""), "");
    }

    #[test]
    fn the_release_heading_comes_off() {
        assert_eq!(without_heading(NOTES).lines().next(), Some("### New"));
        assert_eq!(without_heading("### Fixed\n\n- a"), "### Fixed\n\n- a");
        assert_eq!(without_heading("## [0.1.0](x)"), "");
    }

    #[test]
    fn plain_sections_drop_links_and_under_the_hood() {
        let got = plain_sections(NOTES);
        assert_eq!(
            got,
            vec![
                (
                    "New".to_owned(),
                    vec!["Updates install themselves".to_owned()]
                ),
                ("Fixed".to_owned(), vec!["A crash".to_owned()]),
            ]
        );
        assert_eq!(
            plain_text(
                "**app:** See [the docs](https://x.y) now ([abcdef1](https://github.com/c)) by @me"
            ),
            "app: See the docs now by @me"
        );
    }

    proptest! {
        #![proptest_config(ProptestConfig { rng_seed: proptest::test_runner::RngSeed::Fixed(0x6b616e736f), ..ProptestConfig::default() })]

        /// The summary counts each item once, whatever the order of sections.
        #[test]
        fn prop_summary_counts_items(new in 0usize..5, improved in 0usize..5, fixed in 0usize..5) {
            let mut notes = String::new();
            for (title, n) in [("Fixed", fixed), ("New", new), ("Improved", improved)] {
                if n > 0 {
                    notes.push_str(&format!("### {title}\n\n"));
                    for i in 0..n {
                        notes.push_str(&format!("- item {i}\n"));
                    }
                }
            }
            let line = summary(&notes);
            prop_assert_eq!(line.is_empty(), new + improved + fixed == 0);
            for (n, one, many) in [(new, "new feature", "new features"), (improved, "improvement", "improvements"), (fixed, "fix", "fixes")] {
                if n == 1 {
                    let needle = format!("1 {one}");
                    prop_assert!(line.contains(&needle));
                } else if n > 1 {
                    let needle = format!("{n} {many}");
                    prop_assert!(line.contains(&needle));
                }
            }
        }

        /// Plain text never keeps Markdown link syntax.
        #[test]
        fn prop_plain_text_has_no_links(label in "[a-z ]{1,12}", url in "[a-z]{1,8}", sha in "[0-9a-f]{7}") {
            let text = plain_text(&format!("Fix [{label}](https://{url}) ([{sha}](https://github.com/{url}))"));
            prop_assert!(!text.contains("](") && !text.contains(&sha));
        }
    }

    #[test]
    fn earlier_sections_are_the_releases_before() {
        let log = format!(
            "# Changelog\n\n## Unreleased\n\n- soon\n\n## [0.3.0](y) - 2026-10-21\n\n- third\n\n{NOTES}\n## [0.1.0](x) - 2026-10-19\n\n- first\n"
        );
        let got = earlier_sections(&log, "0.3.0", 10);
        assert_eq!(got.len(), 2);
        assert_eq!(got[0].0, "0.2.0");
        assert_eq!(got[0].1, "2026-10-20");
        assert!(got[0].2.contains("Updates install themselves"));
        assert_eq!(
            got[1],
            (
                "0.1.0".into(),
                "2026-10-19".into(),
                "## [0.1.0](x) - 2026-10-19\n\n- first".into()
            )
        );
        assert_eq!(earlier_sections(&log, "0.3.0", 1).len(), 1);
        assert!(earlier_sections(&log, "0.1.0", 10).is_empty());
    }

    #[test]
    fn under_the_hood_is_left_out() {
        let shown = for_app(NOTES);
        assert!(shown.contains("### New"));
        assert!(shown.contains("### Fixed"));
        assert!(shown.contains("- A crash"));
        assert!(!shown.contains("Under the hood"));
        assert!(!shown.contains("git-cliff"));
        // The compare link stays.
        assert!(shown.contains("compare/v0.1.0...v0.2.0"));
    }

    #[test]
    fn a_release_section_is_found_by_its_version() {
        let log = format!(
            "# Changelog\n\n## Unreleased\n\n- soon\n\n{NOTES}\n## [0.1.0](x) - 2026-10-19\n\n- first\n"
        );
        let s = section(&log, "v0.2.0").unwrap();
        assert!(s.starts_with("## [0.2.0]"));
        assert!(s.contains("Updates install themselves"));
        assert!(!s.contains("first") && !s.contains("soon"));
        assert_eq!(
            section(&log, "0.1.0").unwrap(),
            "## [0.1.0](x) - 2026-10-19\n\n- first"
        );
        assert_eq!(
            section(&log, "0.1"),
            None,
            "not a prefix of another version"
        );
        assert_eq!(section(&log, "9.9.9"), None);
        // The shipped changelog has the first release.
        assert!(section(include_str!("../../../CHANGELOG.md"), "0.1.0").is_some());
    }

    #[test]
    fn a_heading_inside_code_is_text() {
        let notes = "### New\n\n```\n### Under the hood\n```\n- kept\n";
        assert_eq!(for_app(notes), notes.trim());
    }

    fn notes_section() -> impl Strategy<Value = (String, Vec<String>)> {
        (
            prop::sample::select(vec!["New", "Improved", "Fixed", UNDER_THE_HOOD]),
            prop::collection::vec("[a-z]{1,8}( [a-z]{1,8}){0,2}", 0..4),
        )
            .prop_map(|(title, items)| (title.to_owned(), items))
    }

    proptest! {
        /// Every other section comes through whole and in order; nothing
        /// from "Under the hood" does.
        #[test]
        fn prop_only_under_the_hood_goes(sections in prop::collection::vec(notes_section(), 0..6)) {
            let mut notes = String::from("## 1.0.0\n");
            for (title, items) in &sections {
                notes.push_str(&format!("\n### {title}\n\n"));
                for item in items {
                    let marker = if title == UNDER_THE_HOOD { "hood" } else { "kept" };
                    notes.push_str(&format!("- {marker}: {item}\n"));
                }
            }
            let shown = for_app(&notes);
            prop_assert!(!shown.contains("hood:"));
            prop_assert!(!shown.contains(UNDER_THE_HOOD));
            let kept: Vec<_> = sections.iter().filter(|(t, _)| t != UNDER_THE_HOOD).flat_map(|(_, items)| items.iter()).collect();
            let mut at = 0;
            for item in kept {
                let line = format!("- kept: {item}");
                let found = shown[at..].find(&line);
                prop_assert!(found.is_some(), "missing {}", line);
                at += found.unwrap() + line.len();
            }
            // Leaving out a section twice changes nothing more.
            prop_assert_eq!(for_app(&shown), shown);
        }

        /// In a changelog of several releases, each one's section is its
        /// own: its heading first, and no other release's lines.
        #[test]
        fn prop_sections_hold_one_release(count in 1usize..6, pick in any::<prop::sample::Index>()) {
            let versions: Vec<String> = (0..count).map(|i| format!("0.{}.0", count - i)).collect();
            let mut log = String::from("# Changelog\n\n");
            for v in &versions {
                log.push_str(&format!("## [{v}](https://x/compare) - 2026-01-01\n\n### New\n\n- change in {v}\n\n"));
            }
            let v = &versions[pick.index(count)];
            let s = section(&log, v).unwrap();
            let heading = format!("## [{v}]");
            prop_assert!(s.starts_with(&heading));
            let line = format!("- change in {v}");
            prop_assert!(s.contains(&line));
            prop_assert_eq!(s.matches("## [").count(), 1);
        }
    }
}
