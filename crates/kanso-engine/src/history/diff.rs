//! What changed between a kept version of a note and the note now: the
//! changed sections (hunks) with a few unchanged lines around each, line
//! by line, with the changed words inside a changed line marked. Pure; the
//! history dialogs draw it.

use std::ops::Range;

use similar::{ChangeTag, TextDiff};

/// Unchanged lines kept around each change.
pub const CONTEXT_LINES: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineKind {
    Same,
    /// In the note now, not in the version.
    Added,
    /// In the version, not in the note now.
    Removed,
}

/// A line of a section: its text (without the line break), its number in
/// the version and now (from 1), and the byte ranges of words that
/// changed within it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiffLine {
    pub kind: LineKind,
    pub text: String,
    pub old_line: Option<usize>,
    pub new_line: Option<usize>,
    pub changed: Vec<Range<usize>>,
}

/// A changed section.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hunk {
    pub lines: Vec<DiffLine>,
}

impl Hunk {
    /// The section as the version has it: what "Copy section" copies.
    pub fn old_text(&self) -> String {
        self.lines
            .iter()
            .filter(|l| l.kind != LineKind::Added)
            .map(|l| format!("{}\n", l.text))
            .collect()
    }

    /// "Lines 4–9" of the version, for the section's header.
    pub fn old_span(&self) -> Option<(usize, usize)> {
        let mut lines = self.lines.iter().filter_map(|l| l.old_line);
        let first = lines.next()?;
        Some((first, lines.next_back().unwrap_or(first)))
    }

    pub fn counts(&self) -> (usize, usize) {
        let added = self
            .lines
            .iter()
            .filter(|l| l.kind == LineKind::Added)
            .count();
        let removed = self
            .lines
            .iter()
            .filter(|l| l.kind == LineKind::Removed)
            .count();
        (added, removed)
    }
}

fn strip_newline(s: &str) -> &str {
    s.strip_suffix('\n')
        .map(|s| s.strip_suffix('\r').unwrap_or(s))
        .unwrap_or(s)
}

/// The changed sections from `old` (the version) to `new` (now). Pure.
pub fn hunks(old: &str, new: &str) -> Vec<Hunk> {
    let diff = TextDiff::from_lines(old, new);
    diff.grouped_ops(CONTEXT_LINES)
        .iter()
        .map(|group| {
            let mut lines = Vec::new();
            for op in group {
                for change in diff.iter_inline_changes(op) {
                    let kind = match change.tag() {
                        ChangeTag::Equal => LineKind::Same,
                        ChangeTag::Insert => LineKind::Added,
                        ChangeTag::Delete => LineKind::Removed,
                    };
                    let mut text = String::new();
                    let mut changed = Vec::new();
                    for (emphasized, part) in change.iter_strings_lossy() {
                        let start = text.len();
                        text.push_str(&part);
                        if emphasized && kind != LineKind::Same {
                            let end = text.len().min(strip_newline(&text).len());
                            if start < end {
                                changed.push(start..end);
                            }
                        }
                    }
                    let text = strip_newline(&text).to_owned();
                    // A line changed all through needs no word marks.
                    if changed.len() == 1 && changed[0] == (0..text.len()) {
                        changed.clear();
                    }
                    lines.push(DiffLine {
                        kind,
                        text,
                        old_line: change.old_index().map(|i| i + 1),
                        new_line: change.new_index().map(|i| i + 1),
                        changed,
                    });
                }
            }
            Hunk { lines }
        })
        .collect()
}

/// The whole of both texts, line by line, as the diff has them: the
/// version's lines (same and removed) and now's (same and added). Pure;
/// for the tests, which put both texts back together from it.
#[cfg(any(test, feature = "test-support"))]
pub fn sides(old: &str, new: &str) -> (Vec<String>, Vec<String>) {
    let diff = TextDiff::from_lines(old, new);
    let mut before = Vec::new();
    let mut after = Vec::new();
    for change in diff.iter_all_changes() {
        let text = change.to_string_lossy().into_owned();
        match change.tag() {
            ChangeTag::Equal => {
                before.push(text.clone());
                after.push(text);
            }
            ChangeTag::Delete => before.push(text),
            ChangeTag::Insert => after.push(text),
        }
    }
    (before, after)
}

#[cfg(test)]
mod tests {
    use super::{CONTEXT_LINES, LineKind, hunks, sides};
    use proptest::prelude::*;

    #[test]
    fn one_word_changed() {
        let old = "# Plan\n\nBuy milk\nCall Sam\nWalk\n";
        let new = "# Plan\n\nBuy oat milk\nCall Sam\nWalk\nRead\n";
        let found = hunks(old, new);
        assert_eq!(found.len(), 1);
        let hunk = &found[0];
        let removed: Vec<_> = hunk
            .lines
            .iter()
            .filter(|l| l.kind == LineKind::Removed)
            .collect();
        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0].text, "Buy milk");
        let added: Vec<_> = hunk
            .lines
            .iter()
            .filter(|l| l.kind == LineKind::Added)
            .collect();
        assert_eq!(added[0].text, "Buy oat milk");
        assert!(
            added[0]
                .changed
                .iter()
                .any(|r| added[0].text[r.clone()].contains("oat"))
        );
        assert_eq!(added[1].text, "Read");
        assert!(added[1].changed.is_empty());
        assert_eq!(hunk.counts(), (2, 1));
        assert_eq!(hunk.old_span(), Some((1, 5)));
        assert_eq!(hunk.old_text(), old);
        assert!(hunks("same\n", "same\n").is_empty());
    }

    fn text() -> impl Strategy<Value = String> {
        prop::collection::vec(
            prop_oneof!["[a-c ]{0,6}", Just(String::new()), "x[a-z]{0,3}"],
            0..14,
        )
        .prop_map(|lines| {
            lines
                .into_iter()
                .map(|l| format!("{l}\n"))
                .collect::<String>()
        })
    }

    proptest! {
        /// The diff loses nothing: its two sides are the two texts, each
        /// section's lines are numbered in order, and every changed line is
        /// in some section with context around it no wider than allowed.
        #[test]
        fn prop_diff_rebuilds_both_texts(old in text(), new in text()) {
            let (before, after) = sides(&old, &new);
            prop_assert_eq!(before.concat(), old.clone());
            prop_assert_eq!(after.concat(), new.clone());
            let found = hunks(&old, &new);
            prop_assert_eq!(found.is_empty(), old == new);
            let changed: usize = found
                .iter()
                .map(|h| h.lines.iter().filter(|l| l.kind != LineKind::Same).count())
                .sum();
            let expected = similar::TextDiff::from_lines(&old, &new)
                .iter_all_changes()
                .filter(|c| c.tag() != similar::ChangeTag::Equal)
                .count();
            prop_assert_eq!(changed, expected);
            for hunk in &found {
                let olds: Vec<usize> = hunk.lines.iter().filter_map(|l| l.old_line).collect();
                prop_assert!(olds.windows(2).all(|w| w[0] < w[1]));
                let news: Vec<usize> = hunk.lines.iter().filter_map(|l| l.new_line).collect();
                prop_assert!(news.windows(2).all(|w| w[0] < w[1]));
                let lead = hunk.lines.iter().take_while(|l| l.kind == LineKind::Same).count();
                prop_assert!(lead <= CONTEXT_LINES);
                for line in &hunk.lines {
                    for r in &line.changed {
                        prop_assert!(r.end <= line.text.len() && r.start < r.end);
                    }
                }
            }
        }
    }
}
