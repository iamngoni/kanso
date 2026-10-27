//! Which task of a note a click on a box in the preview ticks, and whether
//! that task is still where the click found it. Pure.
//!
//! The preview is handed the note with wiki-links and due dates written as
//! ordinary links, which changes bytes inside a line but never a line, a
//! list marker or a fence. So a task is named by its place in the task list,
//! its line and whether it is ticked, never by its words or its byte.
//!
//! The text view reports the byte where the clicked item starts. That is
//! turned into the task's place in the note's task list
//! (`kanso_core::tasks::task_items`, the index `toggle_task` takes) by the
//! rule the HTML page's linked boxes follow (`kanso_render::html::task_boxes`),
//! so a box in a code fence, in front matter or in indented code is never
//! another task's. A box that can't be placed with certainty is left alone.

use kanso_core::tasks::{task_items, toggle_task};
use kanso_render::html::{TaskSlot, task_boxes};

use crate::preview_chunks::parse_options;

/// A task box ticked in the preview: the task as the preview read it, so the
/// editor can tell, against its own text, that it is still where it was.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskTick {
    /// The task's place in the note's task list.
    pub index: usize,
    /// The line (0-based) it is on.
    pub line: u32,
    /// Whether the box was ticked when it was clicked.
    pub checked: bool,
}

/// The task whose list item starts at byte `offset` of `note`, or `None`
/// when no box is there or its place in the task list can't be known with
/// certainty (a lone carriage return, a box the line scan doesn't see).
pub fn tick_at(note: &str, offset: usize) -> Option<TaskTick> {
    let root = kanso_render::markdown_safe::to_mdast(note, &parse_options()).ok()?;
    let items = task_items(note);
    let slots: Vec<TaskSlot> = items
        .iter()
        .map(|item| TaskSlot {
            line: item.line,
            checked: item.checked,
        })
        .collect();
    let task = task_boxes(&root, note, &slots)
        .into_iter()
        .find(|task| task.offset == offset)?;
    Some(TaskTick {
        index: task.slot?,
        line: task.line,
        checked: task.checked,
    })
}

/// `body` with the task of `tick` toggled, or `None` when `body` no longer
/// has that task at that place: the note changed since the click was read.
pub fn toggled(body: &str, tick: &TaskTick) -> Option<String> {
    let item = task_items(body).into_iter().nth(tick.index)?;
    let same = item.line == tick.line && item.checked == tick.checked;
    same.then(|| toggle_task(body, tick.index)).flatten()
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    /// The offset of the `n`-th occurrence of `needle` in `text`.
    fn nth(text: &str, needle: &str, n: usize) -> usize {
        text.match_indices(needle).nth(n).unwrap().0
    }

    #[test]
    fn a_click_finds_its_task_among_fences_and_nested_lists() {
        let note = "- [ ] a\n  - [x] b\n\n```\n- [ ] code\n```\n\n- [ ] c\n";
        let tick = |offset| tick_at(note, offset).map(|t| (t.index, t.line, t.checked));
        assert_eq!(tick(0), Some((0, 0, false)));
        assert_eq!(tick(nth(note, "- [x]", 0)), Some((1, 1, true)));
        assert_eq!(tick(nth(note, "- [ ] c\n", 0)), Some((2, 7, false)));
        // The fence's line is no task, and neither is a place that is no item.
        assert_eq!(tick(nth(note, "- [ ] code", 0)), None);
        assert_eq!(tick(1), None);
    }

    #[test]
    fn front_matter_and_indented_code_do_not_shift_the_task() {
        // The line scan counts the YAML and the indented lines; the click
        // still lands on the task that `toggle_task` toggles.
        let note = "---\ntags:\n  - [ ] yaml\n---\n\npara\n\n    - [ ] code\n\n- [ ] real\n";
        let tick = tick_at(note, nth(note, "- [ ] real", 0)).unwrap();
        assert_eq!((tick.index, tick.line), (2, 9));
        let after = toggled(note, &tick).unwrap();
        assert_eq!(after, note.replace("- [ ] real", "- [x] real"));
    }

    #[test]
    fn crlf_notes_toggle_the_clicked_task_only() {
        let note = "- [ ] a\r\n- [ ] b\r\n```\r\n- [ ] code\r\n```\r\n- [ ] c\r\n";
        let tick = tick_at(note, nth(note, "- [ ] c\r\n", 0)).unwrap();
        assert_eq!(tick.index, 2);
        assert_eq!(
            toggled(note, &tick).unwrap(),
            "- [ ] a\r\n- [ ] b\r\n```\r\n- [ ] code\r\n```\r\n- [x] c\r\n"
        );
    }

    #[test]
    fn a_lone_carriage_return_leaves_boxes_unclickable() {
        let note = "a\r- [ ] x\r- [ ] y\n";
        assert_eq!(tick_at(note, nth(note, "- [ ] x", 0)), None);
        assert_eq!(tick_at(note, nth(note, "- [ ] y", 0)), None);
    }

    #[test]
    fn a_task_that_moved_is_not_toggled() {
        let note = "- [ ] a\n- [ ] b\n";
        let tick = tick_at(note, nth(note, "- [ ] b", 0)).unwrap();
        // Typed above, ticked elsewhere, or gone since the click.
        assert_eq!(toggled("new\n- [ ] a\n- [ ] b\n", &tick), None);
        assert_eq!(toggled("- [x] a\n- [x] b\n", &tick), None);
        assert_eq!(toggled("- [ ] a\n", &tick), None);
        // Reworded in place, it is still that task.
        assert!(toggled("- [ ] a\n- [ ] bb\n", &tick).is_some());
        assert!(toggled(note, &tick).is_some());
    }

    /// A box written with an uppercase `X` comes back with a lowercase one,
    /// the one form Kanso writes.
    #[test]
    fn an_uppercase_x_comes_back_lowercase() {
        let note = "1. [X] done\n";
        let tick = tick_at(note, 0).unwrap();
        let once = toggled(note, &tick).unwrap();
        assert_eq!(once, "1. [ ] done\n");
        let again = tick_at(&once, 0).unwrap();
        assert_eq!(toggled(&once, &again).unwrap(), "1. [x] done\n");
    }

    /// Note lines, with the awkward ones: fences, quotes, indented code,
    /// front matter fences and prose.
    fn note_lines() -> impl Strategy<Value = Vec<&'static str>> {
        prop::collection::vec(
            prop_oneof![
                Just("- [ ] open"),
                Just("- [x] done"),
                Just("  - [ ] nested"),
                Just("1. [x] numbered"),
                Just("> - [ ] quoted"),
                Just("```"),
                Just("~~~"),
                Just("    - [ ] indented"),
                Just("- not a task"),
                Just("---"),
                Just(""),
                Just("text"),
            ],
            0..14,
        )
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        })]

        /// Round trip: ticking the same box twice gives back the note's
        /// bytes, line endings included.
        #[test]
        fn prop_ticking_a_box_twice_restores_the_bytes(
            lines in note_lines(),
            eol in prop_oneof![Just("\n"), Just("\r\n")],
        ) {
            let note: String = lines.iter().map(|line| format!("{line}{eol}")).collect();
            for (offset, _) in note.match_indices(['-', '1']) {
                let Some(first) = tick_at(&note, offset) else { continue };
                let once = toggled(&note, &first).unwrap();
                // Only the one box's mark changed.
                prop_assert_eq!(once.len(), note.len());
                prop_assert_eq!(
                    once.bytes().zip(note.bytes()).filter(|(a, b)| a != b).count(),
                    1
                );
                let again = tick_at(&once, offset).unwrap();
                prop_assert_eq!((again.index, again.line), (first.index, first.line));
                prop_assert_eq!(again.checked, !first.checked);
                prop_assert_eq!(toggled(&once, &again).unwrap(), note.clone());
            }
        }

        /// Agreement: a click's task is the task of that line in the note,
        /// whatever the lines around it are.
        #[test]
        fn prop_a_click_ticks_the_task_on_its_own_line(lines in note_lines()) {
            let note: String = lines.iter().map(|line| format!("{line}\n")).collect();
            let mut line_start = 0;
            for (ix, line) in note.split('\n').enumerate() {
                if let Some(tick) = tick_at(&note, line_start) {
                    prop_assert_eq!(tick.line as usize, ix, "{:?}", note);
                    let after = toggled(&note, &tick).unwrap();
                    let changed: Vec<usize> = note
                        .split('\n')
                        .zip(after.split('\n'))
                        .enumerate()
                        .filter(|(_, (a, b))| a != b)
                        .map(|(ix, _)| ix)
                        .collect();
                    prop_assert_eq!(changed, vec![ix], "{:?}", note);
                }
                line_start += line.len() + 1;
            }
        }
    }
}
