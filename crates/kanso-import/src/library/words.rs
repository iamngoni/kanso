//! The words an import says, shared by the desktop and the phones: what the
//! picker offers, the summary of a report, the lines for what was skipped or
//! refused, and the label and fraction of a progress bar. All pure, so both
//! apps say the same thing.

use crate::library::{LibraryPhase, LibraryProgress, LibraryReport, SourceKind};

/// What the picker should offer for `kind`: `(files, folders)`. Pure.
pub fn picks(kind: SourceKind) -> (bool, bool) {
    match kind {
        SourceKind::Markdown | SourceKind::Obsidian | SourceKind::Logseq | SourceKind::UpNote => {
            (false, true)
        }
        SourceKind::StandardNotes | SourceKind::Simplenote | SourceKind::Roam => (true, false),
        SourceKind::Notion
        | SourceKind::Bear
        | SourceKind::Evernote
        | SourceKind::Joplin
        | SourceKind::AppleNotes
        | SourceKind::AppleJournal
        | SourceKind::GoogleKeep
        | SourceKind::Html
        | SourceKind::TextBundle
        | SourceKind::Tomboy => (true, true),
    }
}

/// The name of a phase in a progress label. Pure.
pub fn phase_label(phase: LibraryPhase) -> &'static str {
    match phase {
        LibraryPhase::Reading => "Reading notes",
        LibraryPhase::Notebooks => "Notebooks",
        LibraryPhase::Notes => "Notes and attachments",
    }
}

/// `n thing`/`n things`. Pure.
pub fn count(n: usize, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// The summary lines of a report: what the source holds, then what the
/// import did (or would do), then anything left alone. Pure.
pub fn summary(report: &LibraryReport) -> Vec<String> {
    let found = &report.found;
    let mut lines = vec![format!(
        "{}, {}, {}, {}.",
        count(found.notes, "note", "notes"),
        count(found.notebooks, "notebook", "notebooks"),
        count(found.tags, "tag", "tags"),
        count(found.attachments, "attachment", "attachments"),
    )];
    let notes = &report.notes;
    let again = notes.updated + notes.unchanged + notes.kept() > 0;
    let verb = if report.dry_run { "will be" } else { "were" };
    if again {
        lines.push(format!(
            "{} new, {} {verb} updated, {} unchanged since the last import.",
            notes.created, notes.updated, notes.unchanged
        ));
    } else if notes.created > 0 {
        lines.push(format!(
            "{} {verb} added.",
            count(notes.created, "note", "notes")
        ));
    }
    if notes.kept_edited > 0 {
        lines.push(format!(
            "{} changed in Kanso since {} kept as they are.",
            count(notes.kept_edited, "note", "notes"),
            if report.dry_run { "will be" } else { "were" }
        ));
    }
    if notes.kept_deleted > 0 {
        lines.push(format!(
            "{} deleted in Kanso {} stay deleted.",
            count(notes.kept_deleted, "note", "notes"),
            if report.dry_run { "will" } else { "did" }
        ));
    }
    if report.tags_created > 0 {
        lines.push(format!(
            "{} {}.",
            count(report.tags_created, "new tag", "new tags"),
            if report.dry_run {
                "will be made"
            } else {
                "made"
            }
        ));
    }
    if report.unresolved_links > 0 {
        lines.push(format!(
            "{} to notes or files that aren't in it.",
            count(report.unresolved_links, "link points", "links point")
        ));
    }
    lines
}

/// The skipped files, a line per reason with a few examples. Pure.
pub fn skipped_lines(report: &LibraryReport) -> Vec<String> {
    report
        .skipped_by_reason()
        .into_iter()
        .map(|(reason, n, paths)| {
            let more = if n > paths.len() { ", …" } else { "" };
            format!("{n} skipped — {reason}: {}{more}", paths.join(", "))
        })
        .collect()
}

/// How many refused documents [`rejected_lines`] names.
pub const REJECTED_SHOWN: usize = 5;

/// What the store refused: nothing when nothing was, else a header and the
/// first few reasons. Pure.
pub fn rejected_lines(report: &LibraryReport) -> Vec<String> {
    if report.rejected.is_empty() {
        return Vec::new();
    }
    std::iter::once(format!(
        "{} couldn't be imported:",
        count(report.rejected.len(), "item", "items")
    ))
    .chain(report.rejected.iter().take(REJECTED_SHOWN).cloned())
    .collect()
}

/// The label and the filled fraction (0 to 1) of a progress bar. `None` is
/// an import that hasn't reported yet. The fraction is 0 while the total is
/// unknown. Pure.
pub fn progress_label(progress: Option<LibraryProgress>) -> (String, f32) {
    match progress {
        None => ("Starting…".to_owned(), 0.0),
        Some(p) if p.total == 0 => (format!("{}… {}", phase_label(p.phase), p.done), 0.0),
        Some(p) => (
            format!("{}… {} of {}", phase_label(p.phase), p.done, p.total),
            p.done as f32 / p.total as f32,
        ),
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;
    use crate::library::Skipped;

    fn phase(ix: u8) -> LibraryPhase {
        match ix % 3 {
            0 => LibraryPhase::Reading,
            1 => LibraryPhase::Notebooks,
            _ => LibraryPhase::Notes,
        }
    }

    #[test]
    fn a_fresh_import_says_how_many_are_added() {
        let mut report = LibraryReport {
            dry_run: true,
            ..LibraryReport::default()
        };
        report.found.notes = 2;
        report.notes.created = 2;
        assert_eq!(
            summary(&report),
            [
                "2 notes, 0 notebooks, 0 tags, 0 attachments.",
                "2 notes will be added."
            ]
        );
    }

    #[test]
    fn progress_examples() {
        assert_eq!(progress_label(None), ("Starting…".to_owned(), 0.0));
        let reading = LibraryProgress {
            phase: LibraryPhase::Reading,
            done: 4,
            total: 0,
        };
        assert_eq!(
            progress_label(Some(reading)),
            ("Reading notes… 4".to_owned(), 0.0)
        );
        let notes = LibraryProgress {
            phase: LibraryPhase::Notes,
            done: 1,
            total: 4,
        };
        assert_eq!(
            progress_label(Some(notes)),
            ("Notes and attachments… 1 of 4".to_owned(), 0.25)
        );
    }

    #[test]
    fn rejected_examples() {
        let mut report = LibraryReport::default();
        assert!(rejected_lines(&report).is_empty());
        report.rejected = vec!["a: too long".into()];
        assert_eq!(
            rejected_lines(&report),
            ["1 item couldn't be imported:", "a: too long"]
        );
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(128))]

        /// The summary's first line always counts what was found, and a
        /// preview never says anything was done.
        #[test]
        fn prop_summary_tense(created in 0usize..5, updated in 0usize..5, unchanged in 0usize..5, dry in any::<bool>()) {
            let mut report = LibraryReport { dry_run: dry, ..LibraryReport::default() };
            report.found.notes = created + updated + unchanged;
            report.notes.created = created;
            report.notes.updated = updated;
            report.notes.unchanged = unchanged;
            let lines = summary(&report);
            let notes_phrase = if report.found.notes == 1 { "1 note," } else { "notes," };
            prop_assert!(lines[0].contains(notes_phrase));
            let past = lines.iter().any(|l| l.contains(" were "));
            prop_assert_eq!(past, !dry && report.found.notes > 0);
        }

        /// No rejected documents, no lines; else a header counting them all
        /// and at most five of them, in order.
        #[test]
        fn prop_rejected_lines_name_the_first_five(reasons in proptest::collection::vec("[a-z ]{1,12}", 0..12)) {
            let report = LibraryReport { rejected: reasons.clone(), ..LibraryReport::default() };
            let lines = rejected_lines(&report);
            if reasons.is_empty() {
                prop_assert!(lines.is_empty());
            } else {
                prop_assert_eq!(lines.len(), 1 + reasons.len().min(REJECTED_SHOWN));
                let counted = format!("{} ", reasons.len());
                prop_assert!(lines[0].starts_with(&counted));
                prop_assert!(lines[0].ends_with("couldn't be imported:"));
                prop_assert_eq!(&lines[1..], &reasons[..reasons.len().min(REJECTED_SHOWN)]);
            }
        }

        /// The fraction is within 0 and 1, is 0 while the total is unknown,
        /// and the label names the phase and how far along it is.
        #[test]
        fn prop_progress_label_and_fraction(ix in 0u8..3, done in 0usize..1000, extra in 0usize..1000, unknown in any::<bool>()) {
            let total = if unknown { 0 } else { done + extra };
            let p = LibraryProgress { phase: phase(ix), done, total };
            let (label, fraction) = progress_label(Some(p));
            prop_assert!((0.0..=1.0).contains(&fraction));
            prop_assert!(label.starts_with(phase_label(p.phase)));
            if total == 0 {
                prop_assert_eq!(fraction, 0.0);
                prop_assert_eq!(label, format!("{}… {done}", phase_label(p.phase)));
            } else {
                prop_assert_eq!(label, format!("{}… {done} of {total}", phase_label(p.phase)));
            }
        }

        /// Skipped lines: one per reason, each naming how many.
        #[test]
        fn prop_skipped_lines_one_per_reason(reasons in proptest::collection::vec(0u8..4, 0..20)) {
            let mut report = LibraryReport::default();
            for (ix, r) in reasons.iter().enumerate() {
                report.skipped.push(Skipped::new(format!("f{ix}"), format!("reason {r}")));
            }
            let distinct: std::collections::BTreeSet<_> = reasons.iter().collect();
            let lines = skipped_lines(&report);
            prop_assert_eq!(lines.len(), distinct.len());
            let total: usize = lines.iter().map(|l| l.split(' ').next().unwrap().parse::<usize>().unwrap()).sum();
            prop_assert_eq!(total, reasons.len());
        }

        /// A source is picked as files or folders or both, never neither.
        #[test]
        fn prop_every_kind_picks_something(ix in 0usize..SourceKind::ALL.len()) {
            let (files, folders) = picks(SourceKind::ALL[ix]);
            prop_assert!(files || folders);
        }
    }
}
