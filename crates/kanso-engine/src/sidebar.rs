//! What the sidebar shows, decided without drawing it: which notebooks are
//! open and which rows of the notebook tree and of the tag tree show, which
//! tags are listed before "…", and whether the Status group is open. The
//! desktop sidebar and the phone's drawer both draw these.

use std::collections::HashSet;

use kanso_core::NoteStatus;
use kanso_core::tags::TagNode;

/// Tags shown before "…" when there are many: the most used ones.
pub const TAG_LIMIT: usize = 10;

/// Whether the Status group is open: as the user left it, or, until they
/// choose, while a status is the filter.
pub fn status_group_open(choice: Option<bool>, status_filter: bool) -> bool {
    choice.unwrap_or(status_filter)
}

/// The user's choice about the Status group once the filter changes:
/// picking a status from elsewhere opens a group they had closed; other
/// changes keep what they chose.
pub fn status_choice_after(
    choice: Option<bool>,
    was_status: bool,
    is_status: bool,
) -> Option<bool> {
    if is_status && !was_status && choice == Some(false) {
        None
    } else {
        choice
    }
}

/// The statuses the Status group lists, in its order.
pub const STATUS_PLACES: [NoteStatus; 4] = [
    NoteStatus::Active,
    NoteStatus::OnHold,
    NoteStatus::Completed,
    NoteStatus::Dropped,
];

/// The IDs of the tags that are neither nested nor have tags nested in
/// them: they show as chips, the rest in the tree. Pure.
pub fn flat_tag_ids(tree: &[TagNode]) -> HashSet<&str> {
    tree.iter()
        .filter(|node| node.children.is_empty())
        .filter_map(|node| node.tag_id.as_deref())
        .collect()
}

/// Which notebooks the sidebar shows open: those opened with their arrows,
/// and those around the notebook showing, unless folded with their arrows
/// while it shows.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BookFolds {
    /// Opened with their arrows, by ID; kept in the config.
    pub open: HashSet<String>,
    /// Notebooks around the one showing folded with their arrows, with the
    /// one showing then; they open around another one again.
    folded_around: Option<(String, HashSet<String>)>,
}

impl BookFolds {
    /// The folds of a sidebar that opens with `open` opened and nothing
    /// folded around a notebook showing.
    pub fn with_open(open: HashSet<String>) -> BookFolds {
        BookFolds {
            open,
            folded_around: None,
        }
    }

    /// The folds of a sidebar restored from what it kept: `open`, and the
    /// notebooks `folded` around notebook `folded_at` (see
    /// [`BookFolds::folded_around`]). Pure.
    pub fn with_state(
        open: HashSet<String>,
        folded_around: Option<(String, HashSet<String>)>,
    ) -> BookFolds {
        BookFolds {
            open,
            folded_around,
        }
    }

    /// The notebook that was showing when some notebooks around it were
    /// folded, and those notebooks. Pure.
    pub fn folded_around(&self) -> Option<(&str, &HashSet<String>)> {
        self.folded_around
            .as_ref()
            .map(|(at, folded)| (at.as_str(), folded))
    }

    /// Whether notebook `id` shows open while notebook `showing` (if one
    /// is) shows inside the notebooks `around`. Pure.
    pub fn is_open(&self, id: &str, showing: Option<&str>, around: &HashSet<String>) -> bool {
        self.open.contains(id) || (around.contains(id) && !self.folded(id, showing))
    }

    fn folded(&self, id: &str, showing: Option<&str>) -> bool {
        match (&self.folded_around, showing) {
            (Some((at, folded)), Some(showing)) => at == showing && folded.contains(id),
            _ => false,
        }
    }

    /// A click on notebook `id`'s arrow: folds it if it shows open, else
    /// opens it. Pure.
    pub fn toggle(&mut self, id: &str, showing: Option<&str>, around: &HashSet<String>) {
        if !self.is_open(id, showing, around) {
            self.open.insert(id.to_owned());
            if let Some((_, folded)) = &mut self.folded_around {
                folded.remove(id);
            }
            return;
        }
        self.open.remove(id);
        if let Some(showing) = showing
            && around.contains(id)
        {
            match &mut self.folded_around {
                Some((at, folded)) if at == showing => {
                    folded.insert(id.to_owned());
                }
                _ => {
                    self.folded_around = Some((showing.to_owned(), HashSet::from([id.to_owned()])))
                }
            }
        }
    }
}

/// The notebook tree's rows that show, top to bottom, from the depth of
/// each notebook in the tree's order: every notebook whose parents are all
/// open, with whether it has notebooks inside it. `open` is asked by index.
/// Pure.
pub fn visible_book_rows(depths: &[usize], open: &dyn Fn(usize) -> bool) -> Vec<(usize, bool)> {
    let mut rows = Vec::new();
    // Rows deeper than this are inside a folded notebook.
    let mut folded_at: Option<usize> = None;
    for (ix, &depth) in depths.iter().enumerate() {
        if folded_at.is_some_and(|folded| depth > folded) {
            continue;
        }
        folded_at = None;
        let parent = depths.get(ix + 1).is_some_and(|&next| next > depth);
        rows.push((ix, parent));
        if parent && !open(ix) {
            folded_at = Some(depth);
        }
    }
    rows
}

/// The tree's rows top to bottom with their depth: every level whose
/// parents are all open. Pure.
pub fn visible_tag_rows<'a>(
    tree: &'a [TagNode],
    open: &dyn Fn(&str) -> bool,
) -> Vec<(usize, &'a TagNode)> {
    fn walk<'a>(
        nodes: &'a [TagNode],
        depth: usize,
        open: &dyn Fn(&str) -> bool,
        rows: &mut Vec<(usize, &'a TagNode)>,
    ) {
        for node in nodes {
            rows.push((depth, node));
            if open(&node.path) {
                walk(&node.children, depth + 1, open, rows);
            }
        }
    }
    let mut rows = Vec::new();
    walk(tree, 0, open, &mut rows);
    rows
}

/// The tags the sidebar lists, as indexes into `counts` (each tag's note
/// count, in name order), in name order: all of them while `all` is set or
/// there are no more than `limit`, else the `limit` most used and, on top,
/// the `selected` one if it isn't among them. Pure.
pub fn visible_tags(
    counts: &[u32],
    limit: usize,
    selected: Option<usize>,
    all: bool,
) -> Vec<usize> {
    if all || counts.len() <= limit {
        return (0..counts.len()).collect();
    }
    let mut by_use: Vec<usize> = (0..counts.len()).collect();
    by_use.sort_by_key(|&ix| std::cmp::Reverse(counts[ix]));
    let mut shown: Vec<usize> = by_use.into_iter().take(limit).collect();
    if let Some(selected) = selected.filter(|&ix| ix < counts.len() && !shown.contains(&ix)) {
        shown.push(selected);
    }
    shown.sort_unstable();
    shown
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use kanso_core::tags::{TagInfo, tag_tree};
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    use super::{
        BookFolds, flat_tag_ids, status_choice_after, status_group_open, visible_book_rows,
        visible_tag_rows, visible_tags,
    };

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 512,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    /// The bug this replaces: while a status was the filter, a click on
    /// the header changed nothing.
    #[test]
    fn status_group_examples() {
        assert!(!status_group_open(None, false));
        assert!(status_group_open(None, true));
        // Closed by the user while Active is the filter.
        assert!(!status_group_open(Some(false), true));
        // Leaving the status keeps it closed; picking one opens it again.
        assert_eq!(status_choice_after(Some(false), true, false), Some(false));
        assert_eq!(status_choice_after(Some(false), false, true), None);
        // Opened by hand, it stays open after a status comes and goes.
        let picked = status_choice_after(Some(true), false, true);
        assert_eq!(status_choice_after(picked, true, false), Some(true));
    }

    proptest! {
        #![proptest_config(config())]

        /// Invariants: clicking the Status header always shows or hides its
        /// rows, whatever the filter; a status picked from elsewhere opens
        /// it; and leaving a status never undoes what the user chose.
        #[test]
        fn prop_the_status_header_always_toggles(choice in prop::option::of(any::<bool>()), was: bool, is: bool) {
            let open = status_group_open(choice, is);
            prop_assert_eq!(status_group_open(Some(!open), is), !open);
            let after = status_choice_after(choice, was, is);
            if is && !was {
                prop_assert!(status_group_open(after, is));
            }
            if choice.is_some() && !is {
                prop_assert_eq!(after, choice);
            }
        }


        /// The visible tags: in name order without repeats, the selected
        /// one always among them, and no hidden tag used more than a shown
        /// one (other than the selected).
        #[test]
        fn prop_visible_tags_are_the_most_used(
            counts in prop::collection::vec(0u32..20, 0..30),
            limit in 1usize..12,
            selected in prop::option::of(0usize..30),
            all in any::<bool>(),
        ) {
            let shown = visible_tags(&counts, limit, selected, all);
            prop_assert!(shown.windows(2).all(|w| w[0] < w[1]));
            prop_assert!(shown.iter().all(|&ix| ix < counts.len()));
            if all || counts.len() <= limit {
                prop_assert_eq!(shown.len(), counts.len());
            } else {
                prop_assert!(shown.len() <= limit + 1);
                if let Some(selected) = selected.filter(|&s| s < counts.len()) {
                    prop_assert!(shown.contains(&selected));
                }
                let least_shown = shown.iter().filter(|&&ix| Some(ix) != selected).map(|&ix| counts[ix]).min();
                for ix in (0..counts.len()).filter(|ix| !shown.contains(ix)) {
                    prop_assert!(least_shown.is_none_or(|least| counts[ix] <= least));
                }
                // The most used fill the limit; the selected one may come on top.
                let extra = shown.len() - limit;
                prop_assert!(extra <= 1);
                if extra == 1 {
                    prop_assert!(selected.is_some_and(|s| shown.contains(&s)));
                }
            }
        }
    }

    #[test]
    fn tag_tree_rows() {
        let tags: Vec<TagInfo> = ["work", "work/acme", "work/acme/q3", "home", "read"]
            .iter()
            .enumerate()
            .map(|(ix, name)| TagInfo {
                id: format!("tag:{ix}"),
                name: (*name).into(),
            })
            .collect();
        let tree = tag_tree(&tags, &[]);
        let flat = flat_tag_ids(&tree);
        assert_eq!(flat.len(), 2);
        assert!(flat.contains("tag:3") && flat.contains("tag:4"));
        let nested: Vec<_> = tree
            .into_iter()
            .filter(|n| !n.children.is_empty())
            .collect();
        let labels = |open: &dyn Fn(&str) -> bool| {
            visible_tag_rows(&nested, open)
                .into_iter()
                .map(|(depth, node)| format!("{depth}{}", node.label))
                .collect::<Vec<_>>()
        };
        assert_eq!(labels(&|_| false), ["0work"]);
        assert_eq!(labels(&|p| p == "work"), ["0work", "1acme"]);
        assert_eq!(labels(&|_| true), ["0work", "1acme", "2q3"]);
    }

    proptest::proptest! {
        /// With every level open, each nested tag has exactly one row, and
        /// together with the chips every tag shows once.
        #[test]
        fn prop_every_tag_shows_once(names in proptest::collection::btree_set("[ab]{1,2}(/[ab]{1,2}){0,2}", 0..10)) {
            let tags: Vec<TagInfo> = names
                .iter()
                .enumerate()
                .map(|(ix, name)| TagInfo { id: format!("tag:{ix}"), name: name.clone() })
                .collect();
            let tree = tag_tree(&tags, &[]);
            let flat = flat_tag_ids(&tree);
            let nested: Vec<_> = tree.iter().filter(|n| !n.children.is_empty()).cloned().collect();
            let mut shown: Vec<String> = visible_tag_rows(&nested, &|_| true)
                .into_iter()
                .filter_map(|(_, node)| node.tag_id.clone())
                .collect();
            shown.extend(flat.iter().map(|id| (*id).to_owned()));
            shown.sort();
            let mut all: Vec<String> = tags.iter().map(|t| t.id.clone()).collect();
            all.sort();
            proptest::prop_assert_eq!(shown, all);
        }
    }

    #[test]
    fn visible_tags_examples() {
        assert_eq!(visible_tags(&[5, 1, 9, 3], 2, None, false), vec![0, 2]);
        assert_eq!(
            visible_tags(&[5, 1, 9, 3], 2, Some(1), false),
            vec![0, 1, 2]
        );
        assert_eq!(visible_tags(&[5, 1, 9, 3], 2, None, true), vec![0, 1, 2, 3]);
        assert_eq!(visible_tags(&[1, 1, 1], 2, None, false), vec![0, 1]);
    }

    /// The sidebar's notebooks as in the screenshot that asked for this:
    /// folded, only the top levels show; opening one shows its children,
    /// and a folded child keeps its own children hidden.
    #[test]
    fn notebooks_fold_like_an_accordion() {
        // My Vault › (Agents, Clients › TDi, Etapath › (Driver App, Reviews › Filters)), Notes
        let depths = [0, 1, 1, 2, 1, 2, 2, 3, 0];
        let shown = |open: &[usize]| -> Vec<(usize, bool)> {
            visible_book_rows(&depths, &|ix| open.contains(&ix))
        };
        assert_eq!(shown(&[]), vec![(0, true), (8, false)]);
        assert_eq!(
            shown(&[0]),
            vec![(0, true), (1, false), (2, true), (4, true), (8, false)]
        );
        assert_eq!(
            shown(&[0, 4]),
            vec![
                (0, true),
                (1, false),
                (2, true),
                (4, true),
                (5, false),
                (6, true),
                (8, false)
            ]
        );
        // A child opened under a folded parent stays out of sight.
        assert_eq!(shown(&[4]), shown(&[]));
        assert_eq!(visible_book_rows(&[], &|_| true), vec![]);
    }

    /// Showing Filters (inside Reviews, inside Etapath, inside My Vault):
    /// the notebooks around it open, and an arrow among them folds it, as
    /// it shows, until another notebook shows.
    #[test]
    fn a_notebook_around_the_one_showing_folds_with_its_arrow() {
        let around: HashSet<String> = ["vault", "etapath", "reviews"]
            .into_iter()
            .map(str::to_owned)
            .collect();
        let showing = Some("filters");
        let mut folds = BookFolds::default();
        assert!(folds.is_open("etapath", showing, &around));
        folds.toggle("etapath", showing, &around);
        assert!(!folds.is_open("etapath", showing, &around));
        assert!(folds.is_open("vault", showing, &around));
        assert!(folds.open.is_empty(), "nothing opened is kept");
        // Opened again, it stays open, now by its own choice.
        folds.toggle("etapath", showing, &around);
        assert!(folds.is_open("etapath", showing, &around));
        assert!(folds.open.contains("etapath"));
        folds.toggle("etapath", showing, &around);
        assert!(!folds.is_open("etapath", showing, &around));
        // Another notebook showing inside it opens it around that one.
        assert!(folds.is_open("etapath", Some("driver-app"), &around));
        // Away from the notebooks, only those opened show open.
        assert!(!folds.is_open("etapath", None, &HashSet::new()));
    }

    /// Depths of a notebook tree in its order: each one at most one deeper
    /// than the one before it.
    fn tree_depths() -> impl Strategy<Value = Vec<usize>> {
        prop::collection::vec(any::<bool>(), 0..40).prop_map(|steps| {
            let mut depths: Vec<usize> = Vec::new();
            for (ix, deeper) in steps.into_iter().enumerate() {
                let last = depths.last().copied().unwrap_or(0);
                depths.push(match (ix, deeper) {
                    (0, _) => 0,
                    (_, true) => last + 1,
                    (_, false) => last.saturating_sub(ix % 3),
                });
            }
            depths
        })
    }

    proptest! {
        #![proptest_config(config())]

        /// Invariants: a notebook shows exactly when every notebook it is
        /// inside is open; rows keep the tree's order; a row has an arrow
        /// exactly when the next notebook is inside it; and with everything
        /// open, every notebook shows.
        #[test]
        fn prop_a_notebook_shows_when_its_parents_are_open(
            depths in tree_depths(),
            open in prop::collection::vec(any::<bool>(), 40),
        ) {
            let is_open = |ix: usize| open[ix];
            let rows = visible_book_rows(&depths, &is_open);
            let parents_open = |ix: usize| {
                // Walk back to each ancestor: the nearest shallower row.
                let mut depth = depths[ix];
                (0..ix).rev().all(|up| {
                    if depths[up] < depth {
                        depth = depths[up];
                        is_open(up)
                    } else {
                        true
                    }
                })
            };
            let expected: Vec<usize> = (0..depths.len()).filter(|&ix| parents_open(ix)).collect();
            let shown: Vec<usize> = rows.iter().map(|(ix, _)| *ix).collect();
            prop_assert_eq!(shown, expected);
            for (ix, parent) in &rows {
                prop_assert_eq!(*parent, depths.get(ix + 1).is_some_and(|&next| next > depths[*ix]));
            }
            prop_assert_eq!(visible_book_rows(&depths, &|_| true).len(), depths.len());
        }

        /// Invariants: whichever notebook shows and whatever was opened or
        /// folded before, a click on a row's arrow flips whether that row
        /// shows open, and so whether the next row, inside it, shows; no
        /// other notebook changes; and a second click puts it back.
        #[test]
        fn prop_an_arrow_flips_what_shows(
            depths in tree_depths(),
            opened in prop::collection::vec(any::<bool>(), 40),
            showing in any::<prop::sample::Index>(),
            clicks in prop::collection::vec(any::<prop::sample::Index>(), 1..6),
        ) {
            prop_assume!(!depths.is_empty());
            let id = |ix: usize| format!("book:{ix}");
            let showing_ix = showing.index(depths.len());
            // The notebooks the one showing is inside: each nearest
            // shallower one, walking back.
            let mut around = HashSet::new();
            let mut depth = depths[showing_ix];
            for up in (0..showing_ix).rev() {
                if depths[up] < depth {
                    depth = depths[up];
                    around.insert(id(up));
                }
            }
            let showing = id(showing_ix);
            let mut folds = BookFolds::with_open(
                (0..depths.len()).filter(|&ix| opened[ix]).map(id).collect(),
            );
            for click in clicks {
                let state = |folds: &BookFolds| -> Vec<bool> {
                    (0..depths.len())
                        .map(|ix| folds.is_open(&id(ix), Some(&showing), &around))
                        .collect()
                };
                let rows = visible_book_rows(&depths, &|ix| state(&folds)[ix]);
                let parents: Vec<usize> =
                    rows.iter().filter(|(_, parent)| *parent).map(|(ix, _)| *ix).collect();
                if parents.is_empty() {
                    break;
                }
                let target = parents[click.index(parents.len())];
                let before = state(&folds);
                let mut after_folds = folds.clone();
                after_folds.toggle(&id(target), Some(&showing), &around);
                let after = state(&after_folds);
                for ix in 0..depths.len() {
                    prop_assert_eq!(after[ix], before[ix] != (ix == target), "{}", ix);
                }
                let shown_after: Vec<usize> = visible_book_rows(&depths, &|ix| after[ix])
                    .into_iter()
                    .map(|(ix, _)| ix)
                    .collect();
                prop_assert_eq!(shown_after.contains(&(target + 1)), after[target]);
                let mut back = after_folds.clone();
                back.toggle(&id(target), Some(&showing), &around);
                prop_assert_eq!(state(&back), before);
                folds = after_folds;
            }
        }
    }
}
