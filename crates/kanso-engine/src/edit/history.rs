//! Undo and redo for an editor that applies [`Edit`]s to its text, without a
//! window and without a clock of its own.
//!
//! Each step is one replacement and its inverse. Typing that is contiguous
//! and of one kind (inserting, or deleting) joins the step before it, until
//! a pause of [`PAUSE`], a jump of the caret, or a newline. The edits of one
//! IME composition, from the first letter to the commit, are one step
//! however long it takes. Everything else, an engine command, a completion
//! or a paste, is a step of its own. Undoing gives the edit that takes the step back, redoing the one
//! that puts it again; a new step drops what could have been redone.
//!
//! Offsets are UTF-8 byte offsets, as in [`Edit`].

use std::time::{Duration, Instant};

use super::{Edit, Selection};

/// Typing joins the step before it only if it comes within this long of the
/// last typing.
pub const PAUSE: Duration = Duration::from_secs(1);

/// How many steps are kept; the oldest go first.
pub const LIMIT: usize = 1_000;

/// How an edit relates to the steps before it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Grouping {
    /// Typed or deleted by hand: joins the step before it when contiguous.
    Typing,
    /// A command, a completion, a paste: always a step of its own, and the
    /// next typing starts another.
    Own,
    /// Text being composed by an input method, and the edit that commits it:
    /// each replaces the text composed so far, and they are one step.
    Composing,
}

/// One replacement: at byte `at`, `removed` became `inserted`.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Step {
    at: usize,
    removed: String,
    inserted: String,
    /// What was selected before the step, and after it.
    before: Selection,
    after: Selection,
}

impl Step {
    /// The edit that takes the step back, in the text after it.
    fn inverse(&self) -> Edit {
        Edit {
            range: self.at..self.at + self.inserted.len(),
            text: self.removed.clone(),
            selection: self.before.clone(),
        }
    }

    /// The edit that makes the step again, in the text before it.
    fn forward(&self) -> Edit {
        Edit {
            range: self.at..self.at + self.removed.len(),
            text: self.inserted.clone(),
            selection: self.after.clone(),
        }
    }
}

/// The kinds of typing that join, and where the next one must be.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Insert,
    Delete,
    Compose,
}

/// The last step, while more typing can still join it.
#[derive(Clone, Debug)]
struct Open {
    kind: Kind,
    at: Instant,
}

/// The undo and redo steps of one text.
#[derive(Clone, Debug)]
pub struct History {
    undo: Vec<Step>,
    redo: Vec<Step>,
    open: Option<Open>,
    limit: usize,
}

impl Default for History {
    fn default() -> Self {
        Self::with_limit(LIMIT)
    }
}

impl History {
    /// An empty history that keeps at most `limit` steps.
    pub fn with_limit(limit: usize) -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            open: None,
            limit: limit.max(1),
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// How many steps can be undone.
    pub fn undo_len(&self) -> usize {
        self.undo.len()
    }

    /// The next edit starts a step of its own: a composition was committed,
    /// or something else ended the typing.
    pub fn close_step(&mut self) {
        self.open = None;
    }

    /// Forgets every step: the text was replaced as a whole.
    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
        self.open = None;
    }

    /// The caret moved (or the selection changed) other than by an edit:
    /// the next typing starts a step of its own. Call it with every
    /// selection that doesn't come from an edit just recorded; `selection`
    /// is compared with where the last edit left it.
    pub fn selection_changed(&mut self, selection: &Selection) {
        if self.open.is_some() && self.undo.last().is_none_or(|step| step.after != *selection) {
            self.open = None;
        }
    }

    /// Records that `edit` was applied, replacing `removed` (the text of its
    /// range before), with `selection_before` selected, at time `now`.
    pub fn record(
        &mut self,
        removed: &str,
        edit: &Edit,
        selection_before: &Selection,
        grouping: Grouping,
        now: Instant,
    ) {
        if edit.range.is_empty() && edit.text.is_empty() {
            return;
        }
        self.redo.clear();
        let kind = match (grouping, edit.text.contains('\n')) {
            (Grouping::Composing, _) => Some(Kind::Compose),
            (Grouping::Own, _) | (_, true) => None,
            (Grouping::Typing, false) => Some(if edit.text.is_empty() {
                Kind::Delete
            } else {
                Kind::Insert
            }),
        };
        if let (Some(kind), Some(open)) = (kind, &self.open)
            && open.kind == kind
            // A composition goes on however long its pauses.
            && (kind == Kind::Compose || now.saturating_duration_since(open.at) <= PAUSE)
            && let Some(last) = self.undo.last_mut()
            && join(last, kind, edit, removed)
        {
            last.after = edit.selection.clone();
            self.open = Some(Open { kind, at: now });
            // A composition taken back to nothing leaves no step.
            if last.removed.is_empty() && last.inserted.is_empty() {
                self.undo.pop();
                self.open = None;
            }
            return;
        }
        self.undo.push(Step {
            at: edit.range.start,
            removed: removed.to_owned(),
            inserted: edit.text.clone(),
            before: selection_before.clone(),
            after: edit.selection.clone(),
        });
        if self.undo.len() > self.limit {
            self.undo.remove(0);
        }
        self.open = kind.map(|kind| Open { kind, at: now });
    }

    /// The edit that undoes the last step, to apply to the current text.
    pub fn undo(&mut self) -> Option<Edit> {
        let step = self.undo.pop()?;
        let edit = step.inverse();
        self.redo.push(step);
        self.open = None;
        Some(edit)
    }

    /// The edit that makes the last undone step again.
    pub fn redo(&mut self) -> Option<Edit> {
        let step = self.redo.pop()?;
        let edit = step.forward();
        self.undo.push(step);
        self.open = None;
        Some(edit)
    }
}

/// Joins typing `edit` (which replaced `removed`) to `last`, when it comes
/// right where `last` left off; says whether it did.
fn join(last: &mut Step, kind: Kind, edit: &Edit, removed: &str) -> bool {
    match kind {
        Kind::Compose => {
            // The composed text so far is replaced, in whole or in part.
            let region = last.at..last.at + last.inserted.len();
            if edit.range.start < region.start || edit.range.end > region.end {
                return false;
            }
            last.inserted.replace_range(
                edit.range.start - region.start..edit.range.end - region.start,
                &edit.text,
            );
            true
        }
        Kind::Insert => {
            // Typing goes in at the end of what was typed, replacing
            // nothing.
            if !edit.range.is_empty() || edit.range.start != last.at + last.inserted.len() {
                return false;
            }
            last.inserted.push_str(&edit.text);
            true
        }
        Kind::Delete => {
            // Only deleting joins deleting: the step so far inserted
            // nothing.
            if !last.inserted.is_empty() {
                return false;
            }
            if edit.range.end == last.at {
                // Backspace: what is removed now was before what was.
                last.at = edit.range.start;
                last.removed.insert_str(0, removed);
                true
            } else if edit.range.start == last.at {
                // Forward delete: what is removed now was after what was.
                last.removed.push_str(removed);
                true
            } else {
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    use super::*;

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    /// A text edited with a history, and a model of every state it can be
    /// in: the text after each number of steps.
    struct Run {
        text: String,
        history: History,
        /// `states[n]` is the text with `n` steps done.
        states: Vec<String>,
        start: Instant,
        clock: Duration,
    }

    impl Run {
        fn new(text: &str) -> Self {
            Self {
                text: text.to_owned(),
                history: History::default(),
                states: vec![text.to_owned()],
                start: Instant::now(),
                clock: Duration::ZERO,
            }
        }

        fn apply(&mut self, edit: &Edit, grouping: Grouping, wait: Duration) {
            // Nothing happened, so nothing is recorded.
            if edit.range.is_empty() && edit.text.is_empty() {
                return;
            }
            self.clock += wait;
            let selection = 0..0;
            self.history.record(
                &self.text[edit.range.clone()],
                edit,
                &selection,
                grouping,
                self.start + self.clock,
            );
            self.text = edit.apply(&self.text);
            let steps = self.history.undo_len();
            self.states.truncate(steps);
            self.states.push(self.text.clone());
            assert_eq!(self.states.len(), steps + 1);
        }

        fn undo(&mut self) -> bool {
            let Some(edit) = self.history.undo() else {
                return false;
            };
            self.text = edit.apply(&self.text);
            true
        }

        fn redo(&mut self) -> bool {
            let Some(edit) = self.history.redo() else {
                return false;
            };
            self.text = edit.apply(&self.text);
            true
        }
    }

    /// A random edit of `text`: `a` and `b` as fractions of its length,
    /// on character boundaries.
    fn edit_of(text: &str, a: f64, b: f64, inserted: &str) -> Edit {
        let at = |f: f64| {
            let mut at = (text.len() as f64 * f) as usize;
            while !text.is_char_boundary(at) {
                at -= 1;
            }
            at
        };
        let (a, b) = (at(a), at(b));
        let range = a.min(b)..a.max(b);
        let caret = range.start + inserted.len();
        Edit {
            range,
            text: inserted.to_owned(),
            selection: caret..caret,
        }
    }

    #[derive(Clone, Debug)]
    enum Op {
        Edit(f64, f64, String, u8, u64),
        Undo,
        Redo,
    }

    fn ops() -> impl Strategy<Value = Vec<Op>> {
        prop::collection::vec(
            prop_oneof![
                4 => (0f64..=1., 0f64..=1., "[a-zé日🙂\n ]{0,4}", 0u8..3, 0u64..1_500)
                    .prop_map(|(a, b, t, kind, wait)| Op::Edit(a, b, t, kind, wait)),
                2 => Just(Op::Undo),
                1 => Just(Op::Redo),
            ],
            0..30,
        )
    }

    #[test]
    fn typing_joins_until_a_pause_a_jump_or_a_newline() {
        let start = Instant::now();
        let mut history = History::default();
        let mut text = String::new();
        let type_at = |text: &mut String, at: usize, s: &str, ms: u64, h: &mut History| {
            let edit = Edit {
                range: at..at,
                text: s.into(),
                selection: at + s.len()..at + s.len(),
            };
            h.record(
                &text[edit.range.clone()],
                &edit,
                &(at..at),
                Grouping::Typing,
                start + Duration::from_millis(ms),
            );
            *text = edit.apply(text);
        };
        for (i, ch) in ["a", "b", "c"].into_iter().enumerate() {
            type_at(&mut text, i, ch, 100 * i as u64, &mut history);
        }
        assert_eq!(history.undo_len(), 1);
        // A pause longer than a second starts another step.
        type_at(&mut text, 3, "d", 1_500, &mut history);
        assert_eq!(history.undo_len(), 2);
        // A jump elsewhere does.
        type_at(&mut text, 0, "x", 1_600, &mut history);
        assert_eq!(history.undo_len(), 3);
        // A newline is a step of its own, and closes the one it is in.
        type_at(&mut text, 1, "\n", 1_700, &mut history);
        type_at(&mut text, 2, "y", 1_800, &mut history);
        assert_eq!(history.undo_len(), 5);
        assert_eq!(text, "x\nyabcd");
        let undo = history.undo().unwrap();
        text = undo.apply(&text);
        assert_eq!(text, "x\nabcd");
        // Moving the caret on its own ends the step.
        history.selection_changed(&(0..0));
        assert!(history.open.is_none());
    }

    #[test]
    fn deleting_backwards_and_forwards_joins() {
        let start = Instant::now();
        let mut history = History::default();
        let mut text = "hello world".to_owned();
        // Backspace three times from the end of "hello".
        for (i, range) in [(4..5), (3..4), (2..3)].into_iter().enumerate() {
            let edit = Edit {
                range: range.clone(),
                text: String::new(),
                selection: range.start..range.start,
            };
            history.record(
                &text[edit.range.clone()],
                &edit,
                &(range.end..range.end),
                Grouping::Typing,
                start + Duration::from_millis(50 * i as u64),
            );
            text = edit.apply(&text);
        }
        assert_eq!(text, "he world");
        assert_eq!(history.undo_len(), 1);
        let undo = history.undo().unwrap();
        assert_eq!(undo.selection, 5..5);
        assert_eq!(undo.apply(&text), "hello world");
        let redo = history.redo().unwrap();
        assert_eq!(redo.apply("hello world"), "he world");
        // Typing a letter then deleting it is two steps.
        let mut history = History::default();
        let typed = Edit {
            range: 0..0,
            text: "a".into(),
            selection: 1..1,
        };
        history.record("", &typed, &(0..0), Grouping::Typing, start);
        let deleted = Edit {
            range: 0..1,
            text: String::new(),
            selection: 0..0,
        };
        history.record("a", &deleted, &(1..1), Grouping::Typing, start);
        assert_eq!(history.undo_len(), 2);
    }

    #[test]
    fn a_new_step_drops_what_could_be_redone() {
        let start = Instant::now();
        let mut history = History::default();
        let one = Edit {
            range: 0..0,
            text: "a".into(),
            selection: 1..1,
        };
        history.record("", &one, &(0..0), Grouping::Own, start);
        history.undo().unwrap();
        assert!(history.can_redo());
        history.record("", &one, &(0..0), Grouping::Own, start);
        assert!(!history.can_redo() && history.can_undo());
    }

    proptest! {
        #![proptest_config(config())]

        /// Any mix of edits, undos and redos keeps the text equal to the
        /// model's text for the number of steps done; so undoing
        /// everything restores the original and redoing everything the
        /// final text.
        #[test]
        fn prop_undo_and_redo_match_a_model(text in "[a-zé日🙂\n ]{0,16}", ops in ops()) {
            let mut run = Run::new(&text);
            for op in ops {
                match op {
                    Op::Edit(a, b, inserted, kind, wait) => {
                        let edit = edit_of(&run.text, a, b, &inserted);
                        let grouping = [Grouping::Typing, Grouping::Own, Grouping::Composing][kind as usize];
                        run.apply(&edit, grouping, Duration::from_millis(wait));
                    }
                    Op::Undo => { run.undo(); }
                    Op::Redo => { run.redo(); }
                }
                prop_assert_eq!(&run.text, &run.states[run.history.undo_len()]);
            }
            let finished = run.text.clone();
            let redo_len = run.history.redo.len();
            while run.undo() {}
            prop_assert_eq!(&run.text, &text);
            prop_assert!(!run.history.can_undo());
            let mut redone = 0;
            while run.redo() { redone += 1; }
            prop_assert_eq!(redone, run.history.undo_len());
            prop_assert!(redone >= redo_len);
            // Everything redone: the text the last step made.
            prop_assert_eq!(&run.text, run.states.last().unwrap());
            if redo_len == 0 {
                prop_assert_eq!(&run.text, &finished);
            }
        }

        /// Single characters typed one after another within the pause are
        /// one step however many there are, and a pause between any two of
        /// them splits the step there.
        #[test]
        fn prop_contiguous_typing_is_one_step_up_to_a_pause(
            letters in "[a-zé日🙂]{1,12}",
            pauses in prop::collection::vec(any::<bool>(), 12),
        ) {
            let start = Instant::now();
            let mut history = History::default();
            let mut text = String::new();
            let mut clock = Duration::ZERO;
            let mut expected = 0;
            for (ix, ch) in letters.chars().enumerate() {
                let paused = pauses[ix] || ix == 0;
                clock += if paused { PAUSE + Duration::from_millis(1) } else { Duration::from_millis(80) };
                expected += usize::from(paused);
                let at = text.len();
                let edit = Edit {
                    range: at..at,
                    text: ch.to_string(),
                    selection: at + ch.len_utf8()..at + ch.len_utf8(),
                };
                history.record("", &edit, &(at..at), Grouping::Typing, start + clock);
                text = edit.apply(&text);
            }
            prop_assert_eq!(history.undo_len(), expected);
        }

        /// An input method's composition, however many edits it takes and
        /// however long its pauses, is one step: undoing it restores the
        /// text from before, redoing it gives the committed text.
        #[test]
        fn prop_a_composition_is_one_step(
            before in "[a-zé日]{0,6}",
            at in 0f64..=1.,
            composed in prop::collection::vec("[a-zé日ぁ-ん]{0,5}", 1..8),
            pauses in prop::collection::vec(0u64..5_000, 8),
        ) {
            let mut start = (before.len() as f64 * at) as usize;
            while !before.is_char_boundary(start) { start -= 1; }
            let origin = Instant::now();
            let mut history = History::default();
            let mut text = before.clone();
            let mut region = start..start;
            let mut clock = Duration::ZERO;
            for (ix, now) in composed.iter().enumerate() {
                clock += Duration::from_millis(pauses[ix]);
                let edit = Edit {
                    range: region.clone(),
                    text: now.clone(),
                    selection: start + now.len()..start + now.len(),
                };
                history.record(&text[edit.range.clone()], &edit, &(start..start), Grouping::Composing, origin + clock);
                text = edit.apply(&text);
                region = start..start + now.len();
            }
            history.close_step();
            let last = composed.last().unwrap();
            // Taken back to nothing at the end, there is no step at all.
            prop_assert_eq!(history.undo_len(), usize::from(!last.is_empty()));
            if history.can_undo() {
                let undo = history.undo().unwrap();
                prop_assert_eq!(undo.apply(&text), before.clone());
                let redo = history.redo().unwrap();
                prop_assert_eq!(redo.apply(&before), text.clone());
            } else {
                prop_assert_eq!(&text, &before);
            }
        }

        /// The history never keeps more steps than its limit, and what it
        /// keeps still undoes cleanly from the newest back.
        #[test]
        fn prop_the_limit_drops_the_oldest(limit in 1usize..6, count in 1usize..20) {
            let start = Instant::now();
            let mut history = History::with_limit(limit);
            let mut text = String::new();
            for _ in 0..count {
                let at = text.len();
                let edit = Edit { range: at..at, text: "a".into(), selection: at + 1..at + 1 };
                history.record("", &edit, &(at..at), Grouping::Own, start);
                text = edit.apply(&text);
            }
            prop_assert_eq!(history.undo_len(), count.min(limit));
            while let Some(edit) = history.undo() {
                text = edit.apply(&text);
            }
            prop_assert_eq!(text.len(), count - count.min(limit));
        }
    }
}
