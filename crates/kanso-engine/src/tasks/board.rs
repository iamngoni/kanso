//! The Tasks board without a window: every task across the notes grouped
//! Overdue, Today, Upcoming and No date (and Completed, when shown), with a
//! notebook filter, the selection, check-offs and their undo. The desktop's
//! Tasks view and the phones' Tasks place both group through it.

use std::collections::{HashMap, HashSet};

use kanso_core::dates::DateTime;
use kanso_core::tasks::{Due, TaskGroup, task_group};
use kanso_store::{CheckOff, Origin, Store, StoreError, TaskRow};

pub use super::board_model::{CheckedOff, Entry, TaskId, Toggled};

/// The tasks, how they are grouped and filtered, the selection and the
/// last check-off. Pure apart from the store calls it is handed.
#[derive(Default)]
pub struct Board {
    pub rows: Vec<TaskRow>,
    pub loaded: bool,
    /// Only tasks in this notebook's notes.
    pub book: Option<String>,
    pub show_done: bool,
    pub collapsed: HashSet<TaskGroup>,
    pub selected: Option<TaskId>,
    /// Ticked off here since the view opened: they stay in their group,
    /// struck through, rather than vanish under the pointer.
    pub kept: HashSet<TaskId>,
    pub last: Option<CheckedOff>,
}

impl Board {
    /// The group a row shows in at local time `now`.
    pub fn group_of(&self, row: &TaskRow, now: DateTime) -> TaskGroup {
        let due = row.due.as_deref().and_then(Due::parse);
        let kept = !self.show_done && row.checked && self.kept.contains(&TaskId::of(row));
        task_group(due, row.checked && !kept, now)
    }

    /// Whether a row is listed at all: in the notebook asked for, and open
    /// (or done, when done ones show, or ticked off here).
    pub fn listed(&self, row: &TaskRow) -> bool {
        self.book.as_ref().is_none_or(|book| row.book_id == *book)
            && (!row.checked || self.show_done || self.kept.contains(&TaskId::of(row)))
    }

    /// The list's lines at local time `now`: each group with tasks, in
    /// order, its header and (unless collapsed) its tasks. Pure.
    pub fn entries(&self, now: DateTime) -> Vec<Entry> {
        let mut by_group: HashMap<TaskGroup, Vec<usize>> = HashMap::new();
        for (ix, row) in self.rows.iter().enumerate() {
            if self.listed(row) {
                by_group
                    .entry(self.group_of(row, now))
                    .or_default()
                    .push(ix);
            }
        }
        let mut entries = Vec::new();
        for group in TaskGroup::ALL {
            let Some(tasks) = by_group.remove(&group) else {
                continue;
            };
            let collapsed = self.collapsed.contains(&group);
            entries.push(Entry::Header {
                group,
                count: tasks.len(),
                collapsed,
            });
            if !collapsed {
                entries.extend(tasks.into_iter().map(Entry::Task));
            }
        }
        entries
    }

    /// The selected task's place in `entries`, if it shows.
    pub fn selected_entry(&self, entries: &[Entry]) -> Option<usize> {
        let selected = self.selected.as_ref()?;
        entries.iter().position(|entry| match entry {
            Entry::Task(ix) => TaskId::of(&self.rows[*ix]) == *selected,
            Entry::Header { .. } => false,
        })
    }

    /// Moves the selection `delta` tasks down (or up), over headers; the
    /// first task when none was selected.
    pub fn step(&mut self, delta: isize, now: DateTime) {
        let entries = self.entries(now);
        let tasks: Vec<usize> = entries
            .iter()
            .filter_map(|e| match e {
                Entry::Task(ix) => Some(*ix),
                Entry::Header { .. } => None,
            })
            .collect();
        if tasks.is_empty() {
            self.selected = None;
            return;
        }
        let current = self.selected.as_ref().and_then(|id| {
            tasks
                .iter()
                .position(|&ix| TaskId::of(&self.rows[ix]) == *id)
        });
        let next = match current {
            Some(at) => at.saturating_add_signed(delta).min(tasks.len() - 1),
            None => 0,
        };
        self.selected = Some(TaskId::of(&self.rows[tasks[next]]));
    }

    /// The selected task's row.
    pub fn selected_row(&self) -> Option<&TaskRow> {
        let id = self.selected.as_ref()?;
        self.rows.iter().find(|row| TaskId::of(row) == *id)
    }

    /// Takes newly read rows, keeping the selection on its task.
    pub fn set_rows(&mut self, rows: Vec<TaskRow>) {
        self.rows = rows;
        self.loaded = true;
        if let Some(id) = &self.selected
            && !self.rows.iter().any(|row| TaskId::of(row) == *id)
        {
            // The task's line moved: follow it by its words in its note.
            let moved = self
                .rows
                .iter()
                .find(|row| row.note_id == id.note_id && row.text == id.text)
                .map(TaskId::of);
            self.selected = moved;
        }
    }

    /// Ticks the task (or clears it) in its note, in one write, and
    /// remembers it for [`Board::undo`].
    pub fn toggle(&mut self, id: &TaskId, store: &Store, now_ms: i64) -> Toggled {
        let Some(row) = self.rows.iter().find(|row| TaskId::of(row) == *id) else {
            return Toggled::Nothing;
        };
        let checked = !row.checked;
        match self.set(id, checked, store, now_ms) {
            Toggled::Done(done) => {
                self.last = Some(done.clone());
                Toggled::Done(done)
            }
            other => other,
        }
    }

    /// Takes the last check-off back.
    pub fn undo(&mut self, store: &Store, now_ms: i64) -> Toggled {
        let Some(last) = self.last.take() else {
            return Toggled::Nothing;
        };
        self.set(&last.id, !last.checked, store, now_ms)
    }

    fn set(&mut self, id: &TaskId, checked: bool, store: &Store, now_ms: i64) -> Toggled {
        match store.set_task_checked(&id.note_id, id.line, &id.text, checked, now_ms, Origin::Ui) {
            Ok(CheckOff::Changed { .. }) => {
                for row in self.rows.iter_mut().filter(|row| TaskId::of(row) == *id) {
                    row.checked = checked;
                }
                if checked {
                    self.kept.insert(id.clone());
                }
                Toggled::Done(CheckedOff {
                    id: id.clone(),
                    checked,
                })
            }
            Ok(CheckOff::Unchanged) => Toggled::Nothing,
            Ok(CheckOff::Missing) | Err(StoreError::NotFound) => Toggled::Moved,
            Err(err) => Toggled::Failed(err.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    // Not `super::*`: GPUI's own `test` attribute would shadow the built-in
    // one inside `proptest!`.
    use super::{Board, Entry, TaskId, Toggled};
    use kanso_core::DocType;
    use kanso_core::dates::{Date, DateTime};
    use kanso_core::tasks::TaskGroup;
    use kanso_store::{Origin, Put, Store, TaskQuery};
    use proptest::prelude::*;
    use serde_json::json;
    use std::collections::HashSet;

    fn now() -> DateTime {
        DateTime {
            date: Date::new(2026, 10, 20).unwrap(),
            hour: 10,
            minute: 0,
            second: 0,
        }
    }

    fn store_with(bodies: &[&str]) -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("kanso.db")).unwrap();
        store
            .put(
                DocType::Book,
                json!({"_id": "book:a", "name": "A"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
        for (ix, body) in bodies.iter().enumerate() {
            store
                .put(
                    DocType::Note,
                    json!({"_id": format!("note:n{ix}"), "title": format!("N{ix}"), "body": body, "bookId": "book:a"}),
                    Put::Upsert,
                    Origin::Ui,
                )
                .unwrap();
        }
        (dir, store)
    }

    fn read(store: &Store) -> Vec<kanso_store::TaskRow> {
        store
            .tasks(&TaskQuery {
                include_done: true,
                ..Default::default()
            })
            .unwrap()
    }

    #[test]
    fn groups_selection_and_undo() {
        let (_dir, store) = store_with(&[
            "- [ ] Late @2026-10-19\n- [ ] Now @2026-10-20\n- [ ] Next @2026-10-22\n- [ ] Plain\n- [x] Old\n",
        ]);
        let mut board = Board::default();
        board.set_rows(read(&store));
        let groups: Vec<TaskGroup> = board
            .entries(now())
            .iter()
            .filter_map(|e| match e {
                Entry::Header { group, .. } => Some(*group),
                Entry::Task(_) => None,
            })
            .collect();
        assert_eq!(
            groups,
            [
                TaskGroup::Overdue,
                TaskGroup::Today,
                TaskGroup::Upcoming,
                TaskGroup::NoDate
            ]
        );
        board.show_done = true;
        assert_eq!(board.entries(now()).len(), 10);
        board.show_done = false;
        board.collapsed.insert(TaskGroup::Upcoming);
        assert_eq!(board.entries(now()).len(), 7);
        // Down from nothing selects the first task, then steps over headers.
        board.step(1, now());
        assert_eq!(board.selected_row().unwrap().text, "Late @2026-10-19");
        board.step(1, now());
        assert_eq!(board.selected_row().unwrap().text, "Now @2026-10-20");
        board.step(5, now());
        assert_eq!(board.selected_row().unwrap().text, "Plain");
        // Ticking off keeps the task where it was, struck through.
        let id = board.selected.clone().unwrap();
        assert!(matches!(board.toggle(&id, &store, 1), Toggled::Done(_)));
        board.set_rows(read(&store));
        let still: Vec<usize> = board
            .entries(now())
            .iter()
            .filter_map(|e| match e {
                Entry::Task(ix) => Some(*ix),
                Entry::Header { .. } => None,
            })
            .collect();
        assert!(still.iter().any(|&ix| board.rows[ix].text == "Plain"));
        assert!(matches!(board.undo(&store, 2), Toggled::Done(_)));
        assert!(read(&store).iter().any(|r| r.text == "Plain" && !r.checked));
        assert_eq!(board.undo(&store, 3), Toggled::Nothing);
        // A task whose note changed underneath is reported, not guessed at.
        let gone = TaskId {
            note_id: "note:n0".into(),
            line: 0,
            text: "Late @2026-10-19".into(),
        };
        store
            .put(
                DocType::Note,
                json!({"_id": "note:n0", "body": "- [ ] Renamed\n"}),
                Put::Merge,
                Origin::Api,
            )
            .unwrap();
        assert_eq!(board.toggle(&gone, &store, 4), Toggled::Moved);
    }

    #[derive(Clone, Debug)]
    enum Op {
        Step(isize),
        Toggle,
        Undo,
        ShowDone,
        Collapse(usize),
        Reload,
    }

    fn op() -> impl Strategy<Value = Op> {
        prop_oneof![
            (-3isize..=3).prop_map(Op::Step),
            Just(Op::Toggle),
            Just(Op::Toggle),
            Just(Op::Undo),
            Just(Op::ShowDone),
            (0usize..5).prop_map(Op::Collapse),
            Just(Op::Reload),
        ]
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        /// Model-based: check-offs, undo, moving the selection, collapsing
        /// groups and showing done tasks, in any order, against a model
        /// that is just a flag per task and the last check-off. The store
        /// always agrees with the model, every listed task shows in exactly
        /// one group, and a task ticked off here stays listed.
        #[test]
        fn prop_check_offs_follow_the_model(
            dates in prop::collection::vec(prop::option::of(15u32..26), 1..7),
            ops in prop::collection::vec(op(), 0..30),
        ) {
            let body: String = dates.iter().enumerate().map(|(i, d)| match d {
                Some(day) => format!("- [ ] task {i} @2026-10-{day:02}\n"),
                None => format!("- [ ] task {i}\n"),
            }).collect();
            let (_dir, store) = store_with(&[&body]);
            let mut board = Board::default();
            board.set_rows(read(&store));
            let mut model = vec![false; dates.len()];
            let mut last: Option<(usize, bool)> = None;
            let index = |text: &str| -> usize {
                text.split(' ').nth(1).unwrap().parse().unwrap()
            };
            for op in ops {
                match op {
                    Op::Step(d) => board.step(d, now()),
                    Op::Toggle => {
                        if let Some(row) = board.selected_row() {
                            let i = index(&row.text);
                            let id = board.selected.clone().unwrap();
                            let result = board.toggle(&id, &store, 10);
                            prop_assert!(matches!(result, Toggled::Done(_)), "{:?}", result);
                            model[i] = !model[i];
                            last = Some((i, model[i]));
                        }
                    }
                    Op::Undo => {
                        let result = board.undo(&store, 11);
                        match last.take() {
                            Some((i, set)) => {
                                prop_assert!(matches!(result, Toggled::Done(_)));
                                model[i] = !set;
                            }
                            None => prop_assert_eq!(result, Toggled::Nothing),
                        }
                    }
                    Op::ShowDone => board.show_done = !board.show_done,
                    Op::Collapse(g) => {
                        let group = TaskGroup::ALL[g];
                        if !board.collapsed.remove(&group) {
                            board.collapsed.insert(group);
                        }
                    }
                    Op::Reload => board.set_rows(read(&store)),
                }
                let rows = read(&store);
                for row in &rows {
                    prop_assert_eq!(row.checked, model[index(&row.text)]);
                }
                // Every listed task in one group; collapsed groups hide theirs.
                let entries = board.entries(now());
                let mut seen = std::collections::HashSet::new();
                for entry in &entries {
                    if let Entry::Task(ix) = entry {
                        prop_assert!(seen.insert(*ix), "listed twice");
                        let group = board.group_of(&board.rows[*ix], now());
                        prop_assert!(!board.collapsed.contains(&group));
                    }
                }
                let headed: usize = entries.iter().map(|e| match e {
                    Entry::Header { count, .. } => *count,
                    Entry::Task(_) => 0,
                }).sum();
                prop_assert_eq!(headed, board.rows.iter().filter(|r| board.listed(r)).count());
                // Ticked off here and not shown as done: still listed.
                for (ix, row) in board.rows.iter().enumerate() {
                    if row.checked && board.kept.contains(&TaskId { note_id: row.note_id.clone(), line: row.line, text: row.text.clone() }) {
                        prop_assert!(board.listed(&board.rows[ix]));
                    }
                }
            }
        }
    }

    fn row(ix: usize, day: Option<u32>, checked: bool, book: &str) -> kanso_store::TaskRow {
        kanso_store::TaskRow {
            note_id: format!("note:n{}", ix / 3),
            note_title: String::new(),
            book_id: book.into(),
            line: ix as u32,
            checked,
            text: match day {
                Some(d) => format!("task {ix} @2026-10-{d:02}"),
                None => format!("task {ix}"),
            },
            due: day.map(|d| format!("2026-10-{d:02}")),
            note_updated_at: 0,
        }
    }

    fn rows_strategy() -> impl Strategy<Value = Vec<kanso_store::TaskRow>> {
        prop::collection::vec(
            (prop::option::of(15u32..26), any::<bool>(), 0usize..2),
            0..16,
        )
        .prop_map(|specs| {
            specs
                .into_iter()
                .enumerate()
                .map(|(ix, (day, checked, book))| {
                    row(
                        ix,
                        day,
                        checked,
                        if book == 0 { "book:a" } else { "book:b" },
                    )
                })
                .collect()
        })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        /// Grouping: every listed task is in exactly one group, the groups
        /// come in the desktop's order, a group shows only when it has
        /// tasks, and Completed holds done tasks only when they are shown.
        #[test]
        fn prop_grouping_is_a_partition_in_order(
            rows in rows_strategy(),
            show_done in any::<bool>(),
            book in prop::option::of(Just("book:a".to_string())),
            kept in prop::collection::vec(0usize..16, 0..6),
        ) {
            let mut board = Board { show_done, book: book.clone(), ..Board::default() };
            board.set_rows(rows.clone());
            for ix in kept {
                if let Some(row) = rows.get(ix) {
                    board.kept.insert(TaskId::of(row));
                }
            }
            let entries = board.entries(now());
            let mut seen = HashSet::new();
            let mut order = Vec::new();
            let mut current: Option<TaskGroup> = None;
            for entry in &entries {
                match entry {
                    Entry::Header { group, count, collapsed } => {
                        prop_assert!(!collapsed);
                        prop_assert!(*count > 0);
                        order.push(*group);
                        current = Some(*group);
                    }
                    Entry::Task(ix) => {
                        prop_assert!(seen.insert(*ix), "listed twice");
                        prop_assert_eq!(Some(board.group_of(&board.rows[*ix], now())), current);
                    }
                }
            }
            // Headers are in ALL order, each once.
            let positions: Vec<usize> = order
                .iter()
                .map(|g| TaskGroup::ALL.iter().position(|a| a == g).unwrap())
                .collect();
            prop_assert!(positions.windows(2).all(|w| w[0] < w[1]));
            // Exactly the listed rows are shown, in the notebook asked for.
            let listed: HashSet<usize> = board
                .rows
                .iter()
                .enumerate()
                .filter(|(_, r)| board.listed(r))
                .map(|(ix, _)| ix)
                .collect();
            prop_assert_eq!(&seen, &listed);
            for ix in &seen {
                let row = &board.rows[*ix];
                if let Some(book) = &book {
                    prop_assert_eq!(&row.book_id, book);
                }
                // Completed holds only done tasks that show as done.
                if board.group_of(row, now()) == TaskGroup::Done {
                    prop_assert!(show_done && row.checked);
                }
            }
            if !show_done {
                prop_assert!(!order.contains(&TaskGroup::Done));
            }
        }

        /// A task ticked off here and not shown as done stays in the group
        /// it had open, and a reload with the same rows leaves it there.
        #[test]
        fn prop_a_ticked_task_stays_put_until_reload(
            rows in rows_strategy(),
            pick in 0usize..16,
        ) {
            prop_assume!(!rows.is_empty());
            let open: Vec<kanso_store::TaskRow> = rows
                .into_iter()
                .map(|mut r| { r.checked = false; r })
                .collect();
            let mut board = Board::default();
            board.set_rows(open.clone());
            let ix = pick % open.len();
            let before = board.group_of(&board.rows[ix], now());
            // Ticked here: the store now says done.
            let id = TaskId::of(&open[ix]);
            board.kept.insert(id.clone());
            let mut done = open;
            done[ix].checked = true;
            board.set_rows(done);
            prop_assert_eq!(board.group_of(&board.rows[ix], now()), before);
            prop_assert!(board.listed(&board.rows[ix]));
            // With done tasks shown it moves to Completed.
            board.show_done = true;
            prop_assert_eq!(board.group_of(&board.rows[ix], now()), TaskGroup::Done);
            // A fresh board (a reload of the view) drops it.
            let mut fresh = Board::default();
            fresh.set_rows(board.rows.clone());
            prop_assert!(!fresh.listed(&fresh.rows[ix]));
        }
    }
}
