//! Back and forward between notes, as in a browser: every note a window
//! opens is a visit, going back and forward walks the visits, and opening
//! a note after going back drops the visits ahead. Each window keeps its
//! own history, in memory only.
//!
//! Notes trashed or deleted since they were visited are skipped, and so is
//! a visit to the note already open. The workspace records the visits and
//! opens what going back or forward lands on (`Workspace::go_back`).

use gpui_kit::{App, Context, KeyBinding};
use kanso_core::model::TRASH_BOOK_ID;
use kanso_store::Store;

use crate::models::note_visit::{NotePlace, Visit};
use crate::workspace::{GoBack, GoForward, OpenIntent, Workspace};

/// The most visits kept behind the open note; older ones are forgotten.
pub const MOST_VISITS: usize = 100;

/// One window's history: the visits behind the open note, the open note's
/// own visit, and the visits ahead of it, nearest last.
#[derive(Clone, Debug, Default)]
pub struct NavHistory {
    back: Vec<Visit>,
    current: Option<Visit>,
    forward: Vec<Visit>,
}

impl NavHistory {
    /// The note of the current visit.
    pub fn current(&self) -> Option<&str> {
        self.current.as_ref().map(|v| v.id.as_str())
    }

    /// Note `id` opened some way other than going back or forward: it
    /// becomes the current visit, and the visits ahead are dropped. The
    /// note already current stays as it is.
    pub fn visit(&mut self, id: &str) {
        if self.current() == Some(id) {
            return;
        }
        if let Some(current) = self.current.take() {
            self.back.push(current);
            if self.back.len() > MOST_VISITS {
                self.back.remove(0);
            }
        }
        self.forward.clear();
        self.current = Some(Visit::new(id));
    }

    /// The reader was at `place` in note `id` when they left it: kept on
    /// the current visit when it is that note's, to go back to.
    pub fn leave(&mut self, id: &str, place: NotePlace) {
        if let Some(current) = self.current.as_mut().filter(|v| v.id == id) {
            current.place = Some(place);
        }
    }

    /// Steps back to the nearest earlier visit whose note `live` says still
    /// exists, and returns it. The visits stepped over are dropped.
    pub fn back(&mut self, live: impl Fn(&str) -> bool) -> Option<Visit> {
        step(&mut self.back, &mut self.current, &mut self.forward, live)
    }

    /// Steps forward, as [`NavHistory::back`] steps back.
    pub fn forward(&mut self, live: impl Fn(&str) -> bool) -> Option<Visit> {
        step(&mut self.forward, &mut self.current, &mut self.back, live)
    }

    /// Whether going back would land anywhere.
    pub fn can_go_back(&self, live: impl Fn(&str) -> bool) -> bool {
        self.reachable(&self.back, live)
    }

    /// Whether going forward would land anywhere.
    pub fn can_go_forward(&self, live: impl Fn(&str) -> bool) -> bool {
        self.reachable(&self.forward, live)
    }

    fn reachable(&self, visits: &[Visit], live: impl Fn(&str) -> bool) -> bool {
        visits
            .iter()
            .any(|v| Some(v.id.as_str()) != self.current() && live(&v.id))
    }

    /// Every visit, oldest first, and where the current one is in them.
    #[cfg(test)]
    fn visits(&self) -> (Vec<Visit>, Option<usize>) {
        let mut all = self.back.clone();
        let at = self.current.as_ref().map(|current| {
            all.push(current.clone());
            all.len() - 1
        });
        all.extend(self.forward.iter().rev().cloned());
        (all, at)
    }
}

/// Moves from `current` to the nearest visit on `from` that is live and
/// another note, pushing `current` onto `to`. Pops what it passes over.
fn step(
    from: &mut Vec<Visit>,
    current: &mut Option<Visit>,
    to: &mut Vec<Visit>,
    live: impl Fn(&str) -> bool,
) -> Option<Visit> {
    let here = current.as_ref().map(|v| v.id.clone());
    while let Some(visit) = from.pop() {
        if Some(&visit.id) == here.as_ref() || !live(&visit.id) {
            continue;
        }
        if let Some(left) = current.replace(visit.clone()) {
            to.push(left);
        }
        return Some(visit);
    }
    None
}

pub fn bind_keys(cx: &mut App) {
    // ⌘[ and ⌘] as in a browser, also in the note's text, where they
    // would otherwise indent (Tab and ⇧Tab still do); Alt+arrows elsewhere.
    #[cfg(target_os = "macos")]
    cx.bind_keys([
        KeyBinding::new("cmd-[", GoBack, None),
        KeyBinding::new("cmd-]", GoForward, None),
        KeyBinding::new("cmd-[", GoBack, Some("Input")),
        KeyBinding::new("cmd-]", GoForward, Some("Input")),
        KeyBinding::new("alt-cmd-h", crate::workspace::GoNotebookHome, None),
    ]);
    #[cfg(not(target_os = "macos"))]
    cx.bind_keys([
        KeyBinding::new("alt-left", GoBack, None),
        KeyBinding::new("alt-right", GoForward, None),
        KeyBinding::new("ctrl-alt-h", crate::workspace::GoNotebookHome, None),
    ]);
}

/// Whether note `id` can still be gone back to: it exists and isn't in the
/// trash. A visit to a notebook's homepage (its ID) is live while the
/// notebook is there and not locked.
fn live(store: &Store, id: &str) -> bool {
    if id.starts_with("book:") {
        return matches!(store.book_subtree(id, false), Ok(books) if !books.is_empty());
    }
    matches!(store.note(id), Ok(Some(note)) if note.book_id != TRASH_BOOK_ID)
}

// MARK: In the window

impl Workspace {
    /// After a note was selected: once the editor shows it, it is a visit,
    /// and the note the editor left keeps where the reader was in it.
    pub(crate) fn record_visit(&mut self, cx: &mut Context<Self>) {
        self.nav_reach = None;
        if let Some((id, place)) = self.editor.update(cx, |editor, _| editor.take_left_place()) {
            self.tabs.leave(&id, place);
        }
        if let Some(id) = self.editor.read(cx).note_id().map(str::to_owned)
            && let Some(nav) = self.tabs.nav_mut()
        {
            nav.visit(&id);
        }
    }

    /// Whether going back, and going forward, would land anywhere; asked
    /// of the store once per visit or change to the notes, not per frame.
    pub(crate) fn nav_reach(&mut self) -> (bool, bool) {
        if let Some(reach) = self.nav_reach {
            return reach;
        }
        let live = |id: &str| live(&self.store, id);
        let reach = self.tabs.nav().map_or((false, false), |nav| {
            (nav.can_go_back(live), nav.can_go_forward(live))
        });
        self.nav_reach = Some(reach);
        reach
    }

    pub(crate) fn go_back(&mut self, cx: &mut Context<Self>) {
        self.go(true, cx);
    }

    pub(crate) fn go_forward(&mut self, cx: &mut Context<Self>) {
        self.go(false, cx);
    }

    /// Opens the note going back (or forward) lands on, where the reader
    /// left it. Selecting it records no visit: it is already current.
    fn go(&mut self, back: bool, cx: &mut Context<Self>) {
        self.nav_reach = None;
        if let Some((id, place)) = self.editor.read(cx).place(cx) {
            self.tabs.leave(&id, place);
        }
        let store = self.store.clone();
        let live = |id: &str| live(&store, id);
        let Some(visit) = self.tabs.step_active(back, live) else {
            return;
        };
        if let Some(place) = visit.place {
            let id = visit.id.clone();
            self.editor
                .update(cx, |editor, _| editor.place_after_open(id, place));
        }
        // A notebook's homepage is a visit too.
        if visit.id.starts_with("book:") {
            self.open_home_of(Some(visit.id.clone()), cx);
            return;
        }
        // In the tab it started in, as a browser goes back: another tab
        // holding the same note is not switched to.
        self.select_note(&visit.id, OpenIntent::InPlace, cx);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    use super::*;

    fn place(line: u32) -> NotePlace {
        NotePlace {
            caret_line: line,
            scroll_y: -(line as f32) * 10.,
            ..NotePlace::default()
        }
    }

    fn ids(history: &NavHistory) -> Vec<String> {
        history.visits().0.into_iter().map(|v| v.id).collect()
    }

    #[test]
    fn back_and_forward_walk_the_visits() {
        let mut history = NavHistory::default();
        for id in ["a", "b", "c"] {
            history.visit(id);
        }
        assert_eq!(history.back(|_| true).unwrap().id, "b");
        assert_eq!(history.back(|_| true).unwrap().id, "a");
        assert_eq!(history.back(|_| true), None);
        assert_eq!(history.current(), Some("a"));
        assert_eq!(history.forward(|_| true).unwrap().id, "b");
        assert_eq!(history.forward(|_| true).unwrap().id, "c");
        assert_eq!(history.forward(|_| true), None);
    }

    #[test]
    fn opening_a_note_after_going_back_drops_the_visits_ahead() {
        let mut history = NavHistory::default();
        for id in ["a", "b", "c"] {
            history.visit(id);
        }
        history.back(|_| true);
        history.visit("d");
        assert_eq!(ids(&history), ["a", "b", "d"]);
        assert!(!history.can_go_forward(|_| true));
        assert_eq!(history.back(|_| true).unwrap().id, "b");
    }

    #[test]
    fn opening_the_open_note_again_is_not_a_visit() {
        let mut history = NavHistory::default();
        history.visit("a");
        history.visit("a");
        assert_eq!(ids(&history), ["a"]);
        assert!(!history.can_go_back(|_| true));
    }

    #[test]
    fn going_back_skips_notes_trashed_or_deleted() {
        let mut history = NavHistory::default();
        for id in ["a", "b", "c", "d"] {
            history.visit(id);
        }
        let live = |id: &str| id != "b" && id != "c";
        assert!(history.can_go_back(live));
        assert_eq!(history.back(live).unwrap().id, "a");
        assert_eq!(history.forward(live).unwrap().id, "d");
        // With everything behind gone, there is nowhere to go back to.
        assert!(!history.can_go_back(|id| id == "d"));
        assert_eq!(history.back(|id| id == "d"), None);
        assert_eq!(history.current(), Some("d"));
    }

    #[test]
    fn skipping_a_deleted_note_never_lands_on_the_open_one() {
        let mut history = NavHistory::default();
        for id in ["a", "b", "a"] {
            history.visit(id);
        }
        // Behind "a" are "b" (deleted) and "a" itself.
        assert_eq!(history.back(|id| id != "b"), None);
        assert_eq!(history.current(), Some("a"));
    }

    #[test]
    fn going_back_returns_the_place_left() {
        let mut history = NavHistory::default();
        history.visit("a");
        history.leave("a", place(12));
        history.visit("b");
        history.leave("b", place(3));
        let back = history.back(|_| true).unwrap();
        assert_eq!(
            back,
            Visit {
                id: "a".into(),
                place: Some(place(12))
            }
        );
        let forward = history.forward(|_| true).unwrap();
        assert_eq!(forward.place, Some(place(3)));
    }

    #[test]
    fn leaving_another_note_changes_nothing() {
        let mut history = NavHistory::default();
        history.visit("a");
        history.leave("b", place(4));
        assert_eq!(history.visits().0, [Visit::new("a")]);
    }

    #[test]
    fn the_oldest_visits_are_forgotten() {
        let mut history = NavHistory::default();
        for n in 0..MOST_VISITS + 5 {
            history.visit(&n.to_string());
        }
        let (visits, at) = history.visits();
        assert_eq!(visits.len(), MOST_VISITS + 1);
        assert_eq!(visits[0].id, "4");
        assert_eq!(at, Some(MOST_VISITS));
    }

    // MARK: Against a model

    /// The same history kept the plain way: one list of visits and the
    /// index of the current one.
    #[derive(Default)]
    struct Model {
        visits: Vec<Visit>,
        at: Option<usize>,
    }

    impl Model {
        fn current(&self) -> Option<&str> {
            self.at.map(|at| self.visits[at].id.as_str())
        }

        fn visit(&mut self, id: &str) {
            if self.current() == Some(id) {
                return;
            }
            let keep = self.at.map_or(0, |at| at + 1);
            self.visits.truncate(keep);
            self.visits.push(Visit::new(id));
            if self.visits.len() > MOST_VISITS + 1 {
                self.visits.remove(0);
            }
            self.at = Some(self.visits.len() - 1);
        }

        fn leave(&mut self, id: &str, place: NotePlace) {
            if let Some(at) = self.at.filter(|&at| self.visits[at].id == id) {
                self.visits[at].place = Some(place);
            }
        }

        /// Steps one way, `back` towards the start, else towards the end,
        /// to the nearest live visit to another note, dropping the visits
        /// passed over (all of that side's when there is none).
        fn step(&mut self, back: bool, live: &dyn Fn(&str) -> bool) -> Option<Visit> {
            let at = self.at?;
            let here = self.visits[at].id.clone();
            let wanted = |v: &Visit| v.id != here && live(&v.id);
            if back {
                let landing = (0..at).rev().find(|&ix| wanted(&self.visits[ix]));
                let first_passed = landing.map_or(0, |ix| ix + 1);
                self.visits.drain(first_passed..at);
                self.at = Some(first_passed);
                let landing = landing?;
                self.at = Some(landing);
            } else {
                let landing = (at + 1..self.visits.len()).find(|&ix| wanted(&self.visits[ix]));
                let past = landing.unwrap_or(self.visits.len());
                self.visits.drain(at + 1..past);
                landing?;
                self.at = Some(at + 1);
            }
            self.at.map(|at| self.visits[at].clone())
        }
    }

    #[derive(Clone, Debug)]
    enum Op {
        Open(u8),
        Leave(u8, u32),
        Back,
        Forward,
        Trash(u8),
        Restore(u8),
    }

    fn op() -> impl Strategy<Value = Op> {
        prop_oneof![
            4 => (0..6u8).prop_map(Op::Open),
            2 => (0..6u8, 0..50u32).prop_map(|(n, line)| Op::Leave(n, line)),
            3 => Just(Op::Back),
            3 => Just(Op::Forward),
            1 => (0..6u8).prop_map(Op::Trash),
            1 => (0..6u8).prop_map(Op::Restore),
        ]
    }

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 512,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    proptest! {
        #![proptest_config(config())]

        /// Any run of opening, leaving, going back and forward, and notes
        /// trashed and restored, leaves the history as the model has it.
        #[test]
        fn history_matches_the_model(ops in prop::collection::vec(op(), 0..120)) {
            let mut history = NavHistory::default();
            let mut model = Model::default();
            let mut gone: BTreeSet<String> = BTreeSet::new();
            for op in ops {
                let live = |id: &str| !gone.contains(id);
                match op {
                    Op::Open(n) => {
                        let id = n.to_string();
                        // Only a live note opens.
                        if live(&id) {
                            history.visit(&id);
                            model.visit(&id);
                        }
                    }
                    Op::Leave(n, line) => {
                        history.leave(&n.to_string(), place(line));
                        model.leave(&n.to_string(), place(line));
                    }
                    Op::Back | Op::Forward => {
                        let back = matches!(op, Op::Back);
                        let before = history.current().map(str::to_owned);
                        let could = if back {
                            history.can_go_back(live)
                        } else {
                            history.can_go_forward(live)
                        };
                        let got = if back { history.back(live) } else { history.forward(live) };
                        let expected = model.step(back, &live);
                        prop_assert_eq!(could, got.is_some());
                        if let Some(landed) = &got {
                            prop_assert!(live(&landed.id), "landed on a note that is gone");
                            prop_assert_ne!(Some(&landed.id), before.as_ref());
                            prop_assert_eq!(history.current(), Some(landed.id.as_str()));
                        } else {
                            prop_assert_eq!(history.current().map(str::to_owned), before);
                        }
                        prop_assert_eq!(got, expected);
                    }
                    Op::Trash(n) => {
                        gone.insert(n.to_string());
                    }
                    Op::Restore(n) => {
                        gone.remove(&n.to_string());
                    }
                }
                let (visits, at) = history.visits();
                prop_assert_eq!(&visits, &model.visits);
                prop_assert_eq!(at, model.at);
                prop_assert!(history.back.len() <= MOST_VISITS);
            }
        }

        /// With nothing gone, going back n times and forward n times comes
        /// back to the same visit and leaves the history as it was.
        #[test]
        fn back_then_forward_comes_back(
            opens in prop::collection::vec(0..8u8, 1..30),
            steps in 0..30usize,
        ) {
            let mut history = NavHistory::default();
            for n in opens {
                history.visit(&n.to_string());
            }
            let before = history.visits();
            let mut taken = 0;
            while taken < steps && history.back(|_| true).is_some() {
                taken += 1;
            }
            for _ in 0..taken {
                prop_assert!(history.forward(|_| true).is_some());
            }
            prop_assert_eq!(history.visits(), before);
        }
    }
}
