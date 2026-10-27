//! The sidebar's rules as the phone's drawer uses them: which notebooks show
//! and are open, and the tag chips and tree.

mod common;

use std::collections::BTreeSet;

use common::*;
use kanso_mobile::{FoldState, Library, ListPlace, ListQuery, NoteChange};
use proptest::prelude::*;

fn names(shown: &[kanso_mobile::VisibleNotebook]) -> Vec<&str> {
    shown.iter().map(|v| v.row.name.as_str()).collect()
}

/// Top (a, b), a holds a1 which holds a1x; b holds b1. Returns the IDs by
/// name.
fn tree(library: &Library) -> std::collections::HashMap<&'static str, String> {
    let mut ids: std::collections::HashMap<&'static str, String> = std::collections::HashMap::new();
    for (name, parent) in [
        ("a", None),
        ("b", None),
        ("a1", Some("a")),
        ("a1x", Some("a1")),
        ("b1", Some("b")),
    ] {
        let parent = parent.map(|p| ids[p].clone());
        ids.insert(
            name,
            library.create_notebook(name.into(), parent).unwrap().id,
        );
    }
    ids
}

/// The IDs of the notebooks `id` is inside, nearest last: the tree `tree`
/// builds.
fn ancestors(ids: &std::collections::HashMap<&'static str, String>, id: &str) -> Vec<String> {
    let parents = [("a1", "a"), ("a1x", "a1"), ("b1", "b")];
    let mut here = ids
        .iter()
        .find(|(_, value)| value.as_str() == id)
        .map(|(name, _)| *name);
    let mut out = Vec::new();
    while let Some(child) = here {
        let Some((_, parent)) = parents.iter().find(|(name, _)| *name == child) else {
            break;
        };
        out.push(ids[parent].clone());
        here = Some(parent);
    }
    out
}

/// The tag names a tag is nested inside, from its name.
fn tag_ancestors(name: &str) -> Vec<&str> {
    let mut out = Vec::new();
    for (at, ch) in name.char_indices() {
        if ch == '/' {
            out.push(&name[..at]);
        }
    }
    out
}

// no-prop-test: a fixed tree, walked through folding and showing; the
// invariants over any tree are below.
#[test]
fn notebooks_fold_and_open_around_the_one_showing() {
    let phone = lone_phone();
    let library = &phone.library;
    let ids = tree(library);
    let folds = FoldState::default();
    // All folded: the top level, each a parent and shut.
    let shown = library.visible_notebooks(folds.clone(), None).unwrap();
    assert_eq!(names(&shown), ["a", "b"]);
    assert!(shown.iter().all(|v| v.parent && !v.open));
    // Opening a with its arrow shows a1 (itself a shut parent).
    let folds = library.toggle_fold(folds, ids["a"].clone(), None).unwrap();
    assert_eq!(folds.open, vec![ids["a"].clone()]);
    let shown = library.visible_notebooks(folds.clone(), None).unwrap();
    assert_eq!(names(&shown), ["a", "a1", "b"]);
    assert_eq!(shown[1].row.depth, 1);
    assert!(shown[0].open && shown[1].parent && !shown[1].open);
    // Showing a1x opens what is around it, with nothing opened by hand.
    let showing = Some(ids["a1x"].clone());
    let shown = library
        .visible_notebooks(FoldState::default(), showing.clone())
        .unwrap();
    assert_eq!(names(&shown), ["a", "a1", "a1x", "b"]);
    assert!(!shown[2].parent && !shown[2].open);
    // Folding a around it hides what is inside, and remembers it there.
    let folded = library
        .toggle_fold(FoldState::default(), ids["a"].clone(), showing.clone())
        .unwrap();
    assert_eq!(folded.folded_at, showing);
    assert_eq!(folded.folded, vec![ids["a"].clone()]);
    let shown = library
        .visible_notebooks(folded.clone(), showing.clone())
        .unwrap();
    assert_eq!(names(&shown), ["a", "b"]);
    // Showing another notebook opens it again.
    let shown = library
        .visible_notebooks(folded.clone(), Some(ids["b1"].clone()))
        .unwrap();
    assert_eq!(names(&shown), ["a", "b", "b1"]);
    // Tapping it again opens it.
    let reopened = library
        .toggle_fold(folded, ids["a"].clone(), showing.clone())
        .unwrap();
    let shown = library.visible_notebooks(reopened, showing).unwrap();
    assert_eq!(names(&shown), ["a", "a1", "a1x", "b"]);
    // A notebook that isn't there around nothing.
    let shown = library
        .visible_notebooks(FoldState::default(), Some("book:nope".into()))
        .unwrap();
    assert_eq!(names(&shown), ["a", "b"]);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    /// Invariants over any folds: every shown row's parent shows and is
    /// open; toggling twice leaves the notebook's openness as it was; the
    /// notebook showing shows unless one of the notebooks around it was
    /// folded with its arrow while it showed.
    #[test]
    fn prop_folds_show_only_what_is_open(
        open in proptest::collection::btree_set(0usize..5, 0..5),
        toggles in proptest::collection::vec(0usize..5, 0..4),
        showing in proptest::option::of(0usize..5),
    ) {
        let phone = lone_phone();
        let library = &phone.library;
        let ids = tree(library);
        let order = ["a", "b", "a1", "a1x", "b1"];
        let id = |ix: usize| ids[order[ix]].clone();
        let showing = showing.map(id);
        let mut folds = FoldState {
            open: open.iter().map(|ix| id(*ix)).collect(),
            ..FoldState::default()
        };
        for ix in toggles {
            folds = library.toggle_fold(folds, id(ix), showing.clone()).unwrap();
        }
        let shown = library.visible_notebooks(folds.clone(), showing.clone()).unwrap();
        let shown_ids: BTreeSet<&str> = shown.iter().map(|v| v.row.id.as_str()).collect();
        for v in &shown {
            if let Some(parent) = &v.row.parent_id {
                let above = shown.iter().find(|p| &p.row.id == parent);
                prop_assert!(above.is_some_and(|p| p.open), "{} under a shut parent", v.row.name);
            }
            prop_assert_eq!(v.parent, v.row.has_children);
            prop_assert!(!v.open || v.parent);
        }
        if let Some(showing) = &showing {
            let folded_here = folds.folded_at.as_deref() == Some(showing.as_str());
            let hidden = ancestors(&ids, showing).iter().any(|above| {
                folded_here
                    && folds.folded.iter().any(|folded| folded == above)
                    && !folds.open.iter().any(|open| open == above)
            });
            prop_assert_eq!(
                shown_ids.contains(showing.as_str()),
                !hidden,
                "{} hidden by a folded notebook around it",
                showing
            );
        }
        // Two taps on one arrow are no change to what shows.
        let target = id(0);
        let once = library.toggle_fold(folds.clone(), target.clone(), showing.clone()).unwrap();
        let twice = library.toggle_fold(once, target, showing.clone()).unwrap();
        let after = library.visible_notebooks(twice, showing.clone()).unwrap();
        prop_assert_eq!(after, shown);
    }
}

fn tag_with_note(library: &Library, name: &str, notes: usize) -> String {
    let tag = library.create_tag(name.into()).unwrap().id;
    for i in 0..notes {
        let id = note(library, &format!("{name} {i}"), "x");
        library
            .change(
                vec![id],
                NoteChange::AddTag {
                    tag_id: tag.clone(),
                },
            )
            .unwrap();
    }
    tag
}

// no-prop-test: a fixed set of tags, nested and not.
#[test]
fn nested_tags_show_as_a_tree_and_the_rest_as_chips() {
    let phone = lone_phone();
    let library = &phone.library;
    let solo = tag_with_note(library, "solo", 1);
    let acme = tag_with_note(library, "work/acme", 2);
    tag_with_note(library, "work/home", 1);
    tag_with_note(library, "work/acme/q3", 1);
    let places = library.tag_places(false, None, vec![], None).unwrap();
    let chips: Vec<&str> = places.chips.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(chips, ["solo"]);
    assert!(!places.more);
    // The tree is folded: one row, `work`, which is only a level.
    assert_eq!(places.tree.len(), 1);
    let work = &places.tree[0];
    assert_eq!((work.path.as_str(), work.label.as_str()), ("work", "work"));
    assert_eq!(
        (work.tag_id.clone(), work.parent, work.open, work.depth),
        (None, true, false, 0)
    );
    assert_eq!(work.count, 4, "notes with a tag at or under the level");
    // Opened by hand (any case).
    let places = library
        .tag_places(false, None, vec!["WORK".into()], None)
        .unwrap();
    let rows: Vec<(&str, u32, bool)> = places
        .tree
        .iter()
        .map(|r| (r.label.as_str(), r.depth, r.open))
        .collect();
    assert_eq!(
        rows,
        [("work", 0, true), ("acme", 1, false), ("home", 1, false)]
    );
    let home = &places.tree[2];
    assert!(!home.parent && home.tag_id.is_some());
    // Showing a tag deep inside opens the levels around it, not itself.
    let places = library
        .tag_places(false, None, vec![], Some("work/acme/q3".into()))
        .unwrap();
    let rows: Vec<(&str, bool)> = places
        .tree
        .iter()
        .map(|r| (r.label.as_str(), r.open))
        .collect();
    assert_eq!(
        rows,
        [
            ("work", true),
            ("acme", true),
            ("q3", false),
            ("home", false)
        ]
    );
    // A level lists as a path, a leaf as its tag.
    let acme_row = places.tree.iter().find(|r| r.label == "acme").unwrap();
    assert_eq!(acme_row.tag_id.as_deref(), Some(acme.as_str()));
    let listed = library
        .list(ListQuery {
            place: ListPlace::TagPath {
                path: acme_row.path.clone(),
            },
            ..ListQuery::default()
        })
        .unwrap();
    assert_eq!(listed.rows.len(), 3);
    let _ = solo;
}

// no-prop-test: the chip limit with fixed counts; the invariants over any
// counts are below.
#[test]
fn many_tags_show_the_most_used_and_the_selected_one() {
    let phone = lone_phone();
    let library = &phone.library;
    // 12 flat tags: t0 .. t11, used 12 - i times is too slow; use 1 note
    // each for the first ten, none for the last two.
    let ids: Vec<String> = (0..12)
        .map(|i| {
            let tag = library.create_tag(format!("t{i:02}")).unwrap().id;
            if i < 10 {
                tag_with_note_for(library, &tag, i);
            }
            tag
        })
        .collect();
    let few = library.tag_places(false, None, vec![], None).unwrap();
    assert_eq!(few.chips.len(), 10);
    assert!(few.more);
    let unused = &ids[11];
    let with_selected = library
        .tag_places(false, Some(unused.clone()), vec![], None)
        .unwrap();
    assert_eq!(with_selected.chips.len(), 11);
    assert!(with_selected.chips.iter().any(|c| &c.id == unused));
    let all = library.tag_places(true, None, vec![], None).unwrap();
    assert_eq!(all.chips.len(), 12);
    let sorted: Vec<&str> = all.chips.iter().map(|c| c.name.as_str()).collect();
    let mut by_name = sorted.clone();
    by_name.sort_by_key(|n| n.to_lowercase());
    assert_eq!(sorted, by_name, "in name order");
}

fn tag_with_note_for(library: &Library, tag: &str, i: usize) {
    let id = note(library, &format!("n{i}"), "x");
    library
        .change(
            vec![id],
            NoteChange::AddTag {
                tag_id: tag.to_owned(),
            },
        )
        .unwrap();
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(16))]

    /// Invariants over any tag names: the chips never show a nested tag, a
    /// tag is a chip or in the tree but not both lost, chips are at most
    /// the limit unless all are asked for, and the selected chip is shown.
    #[test]
    fn prop_every_tag_is_a_chip_or_in_the_tree(
        names in proptest::collection::btree_set("[a-c]{1,2}(/[a-c]{1,2}){0,2}", 1..16),
        all in any::<bool>(),
        pick in 0usize..16,
    ) {
        let phone = lone_phone();
        let library = &phone.library;
        for name in &names {
            let _ = library.create_tag(name.clone());
        }
        let tags = library.tags().unwrap();
        let selected = tags[pick % tags.len()].id.clone();
        let opened = library
            .tag_places(true, None, tags.iter().map(|t| t.name.clone()).collect(), None)
            .unwrap();
        let in_tree: BTreeSet<&str> =
            opened.tree.iter().filter_map(|r| r.tag_id.as_deref()).collect();
        let every = library.tag_places(true, None, vec![], None).unwrap();
        for chip in &every.chips {
            prop_assert!(!in_tree.contains(chip.id.as_str()), "{} in both", chip.name);
        }
        for tag in &tags {
            let chip = every.chips.iter().any(|c| c.id == tag.id);
            // A level folded above a nested tag still holds it: unfolding its
            // row shows the tag, as the desktop's tree does.
            let reachable = every.tree.iter().any(|row| {
                tag_ancestors(&tag.name)
                    .iter()
                    .any(|above| above.eq_ignore_ascii_case(&row.path))
            });
            prop_assert!(
                chip || in_tree.contains(tag.id.as_str()) || reachable,
                "{} nowhere",
                tag.name
            );
        }
        let places = library.tag_places(all, Some(selected.clone()), vec![], None).unwrap();
        if !all {
            prop_assert!(places.chips.len() <= 11);
        }
        if every.chips.iter().any(|c| c.id == selected) {
            prop_assert!(places.chips.iter().any(|c| c.id == selected));
        }
        prop_assert_eq!(places.more, every.chips.len() > 10);
        for row in &places.tree {
            prop_assert!(!row.open || row.parent);
        }
    }
}
