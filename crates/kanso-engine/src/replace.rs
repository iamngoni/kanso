//! Replace in all notes without a window: the options as the toggles have
//! them, the pattern they make, and the words the button and the reports
//! say. The store finds and replaces ([`kanso_store::Store::replace_preview`],
//! [`kanso_store::Store::replace_in_notes`]); the desktop's sheet and the
//! phones' screen say the same things.

use kanso_core::replace::Pattern;
use kanso_store::{ReplaceReport, UndoReport};

/// The options as the toggles have them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Options {
    pub match_case: bool,
    pub whole_word: bool,
    pub regex: bool,
    pub skip_code: bool,
    pub include_trash: bool,
}

impl Options {
    /// What to look for: `find` as these options have it. Pure.
    pub fn pattern(&self, find: &str) -> Pattern {
        Pattern {
            find: find.to_owned(),
            match_case: self.match_case,
            whole_word: self.whole_word,
            regex: self.regex,
            skip_code: self.skip_code,
        }
    }
}

/// What the Replace button says, or `None` when there is nothing to do.
/// Pure.
pub fn apply_label(matches: usize, notes: usize) -> Option<String> {
    (matches > 0).then(|| {
        format!(
            "Replace {matches} in {notes} {}",
            if notes == 1 { "note" } else { "notes" }
        )
    })
}

/// What the notification says after replacing. Pure.
pub fn report_text(report: &ReplaceReport) -> String {
    let notes = report.notes.len();
    let mut text = format!(
        "Replaced {} {} in {notes} {}.",
        report.replaced,
        if report.replaced == 1 {
            "match"
        } else {
            "matches"
        },
        if notes == 1 { "note" } else { "notes" }
    );
    match report.skipped.len() {
        0 => {}
        1 => text.push_str(&format!(
            " “{}” changed elsewhere since the preview and was left alone.",
            untitled(&report.skipped[0].title)
        )),
        n => text.push_str(&format!(
            " {n} notes changed elsewhere since the preview and were left alone."
        )),
    }
    text
}

/// What the notification says after undoing. Pure.
pub fn undo_text(report: &UndoReport) -> String {
    let restored = report.restored.len();
    let mut text = format!(
        "Put back {restored} {}.",
        if restored == 1 { "note" } else { "notes" }
    );
    match report.skipped.len() {
        0 => {}
        1 => text.push_str(&format!(
            " “{}” was edited since and kept its changes.",
            untitled(&report.skipped[0].title)
        )),
        n => text.push_str(&format!(
            " {n} notes were edited since and kept their changes."
        )),
    }
    text
}

pub fn untitled(title: &str) -> &str {
    if title.trim().is_empty() {
        "Untitled"
    } else {
        title
    }
}

/// A new batch ID for one Replace.
pub fn new_batch() -> String {
    kanso_core::new_id(kanso_core::DocType::Note).replacen("note:", "replace:", 1)
}

#[cfg(test)]
mod tests {
    use super::{Options, apply_label, new_batch, report_text, undo_text, untitled};
    use kanso_store::{ReplaceReport, Skipped, UndoReport};

    #[test]
    fn labels_and_reports() {
        assert_eq!(apply_label(0, 0), None);
        assert_eq!(apply_label(1, 1).as_deref(), Some("Replace 1 in 1 note"));
        assert_eq!(apply_label(7, 3).as_deref(), Some("Replace 7 in 3 notes"));
        let mut report = ReplaceReport {
            replaced: 4,
            notes: vec!["note:a".into(), "note:b".into()],
            skipped: vec![],
        };
        assert_eq!(report_text(&report), "Replaced 4 matches in 2 notes.");
        report.skipped.push(Skipped {
            id: "note:c".into(),
            title: "".into(),
        });
        assert_eq!(
            report_text(&report),
            "Replaced 4 matches in 2 notes. “Untitled” changed elsewhere since the preview and was left alone."
        );
        let undo = UndoReport {
            restored: vec!["note:a".into()],
            skipped: vec![
                Skipped {
                    id: "note:b".into(),
                    title: "B".into(),
                },
                Skipped {
                    id: "note:c".into(),
                    title: "C".into(),
                },
            ],
        };
        assert_eq!(
            undo_text(&undo),
            "Put back 1 note. 2 notes were edited since and kept their changes."
        );
    }

    // no-prop-test: a table of the words the toggles map to.
    #[test]
    fn options_make_the_pattern_they_name() {
        let options = Options {
            match_case: true,
            regex: true,
            skip_code: true,
            ..Options::default()
        };
        let pattern = options.pattern("a+");
        assert_eq!(pattern.find, "a+");
        assert!(pattern.match_case && pattern.regex && pattern.skip_code);
        assert!(!pattern.whole_word);
    }

    // no-prop-test: a fixed shape.
    #[test]
    fn a_batch_id_is_new_every_time() {
        let (a, b) = (new_batch(), new_batch());
        assert!(a.starts_with("replace:"));
        assert_ne!(a, b);
        assert_eq!(untitled("  "), "Untitled");
        assert_eq!(untitled("Plans"), "Plans");
    }
}
