//! Two edits of one text made apart (issue #390 › Shared notebooks): the
//! text typed in an editor and what a collaborator's edit made of the same
//! version. When no change of one touches a change of the other, both are
//! kept, each where its text is; otherwise it is a real conflict and the
//! editor asks which version to keep, as the desktop does.

use crate::doc::{apply_splices, splices};
use crate::model::Splice;

/// `mine` and `theirs`, both made from `base`, merged: `Some` with both
/// sets of changes when none of one touches (overlaps or abuts) any of the
/// other, `None` when they do. With one side unchanged it is the other.
/// Pure.
pub fn merge_apart(base: &str, mine: &str, theirs: &str) -> Option<String> {
    if mine == base || mine == theirs {
        return Some(theirs.to_owned());
    }
    if theirs == base {
        return Some(mine.to_owned());
    }
    let ours = splices(base, mine);
    let other = splices(base, theirs);
    let touch = |a: &Splice, b: &Splice| a.at <= b.at + b.delete && b.at <= a.at + a.delete;
    if ours.iter().any(|a| other.iter().any(|b| touch(a, b))) {
        return None;
    }
    // Apart, every splice has offsets in `base`: made from the end, each
    // leaves the offsets before it as they were.
    let mut all: Vec<Splice> = ours.into_iter().chain(other).collect();
    all.sort_by_key(|splice| std::cmp::Reverse(splice.at));
    let mut out = base.to_owned();
    apply_splices(&mut out, &all);
    Some(out)
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn edits_apart_merge_and_edits_on_the_same_words_dont() {
        assert_eq!(
            merge_apart(
                "Pixel review",
                "Pixel review from Alice",
                "Bob's pixel review"
            )
            .as_deref(),
            Some("Bob's pixel review from Alice")
        );
        assert_eq!(
            merge_apart(
                "one\ntwo\nthree\n",
                "one!\ntwo\nthree\n",
                "one\ntwo\nthree?\n"
            )
            .as_deref(),
            Some("one!\ntwo\nthree?\n")
        );
        assert_eq!(merge_apart("cat", "cut", "cot"), None);
        assert_eq!(merge_apart("ab", "aXb", "aYb"), None);
        assert_eq!(merge_apart("a", "a", "b").as_deref(), Some("b"));
        assert_eq!(merge_apart("a", "b", "a").as_deref(), Some("b"));
        assert_eq!(merge_apart("a", "b", "b").as_deref(), Some("b"));
        assert_eq!(
            merge_apart("héllo wörld", "héllo wörld ✓", "Héllo wörld").as_deref(),
            Some("Héllo wörld ✓")
        );
    }

    /// Text without the separator.
    fn piece() -> impl Strategy<Value = String> {
        proptest::string::string_regex("[a-cé✓ \n]{0,6}").unwrap()
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256,
            failure_persistence: None,
            ..ProptestConfig::default()
        })]

        /// One side unchanged: the merge is the other side.
        #[test]
        fn one_side_unchanged_is_the_other(base in piece(), edit in piece()) {
            prop_assert_eq!(merge_apart(&base, &base, &edit), Some(edit.clone()));
            prop_assert_eq!(merge_apart(&base, &edit, &base), Some(edit));
        }

        /// It doesn't matter whose edit is whose.
        #[test]
        fn the_merge_is_symmetric(base in piece(), a in piece(), b in piece()) {
            prop_assert_eq!(merge_apart(&base, &a, &b), merge_apart(&base, &b, &a));
        }

        /// Paragraphs between separators, mine edits some and theirs the
        /// others (never the same one): the merge has each paragraph as the
        /// side that edited it wrote it.
        #[test]
        fn edits_of_different_paragraphs_both_stay(
            parts in proptest::collection::vec((piece(), piece(), 0u8..3), 1..6),
        ) {
            let join = |pick: &dyn Fn(&(String, String, u8)) -> String| {
                parts.iter().map(pick).collect::<Vec<_>>().join("|")
            };
            let base = join(&|(b, _, _)| b.clone());
            let mine = join(&|(b, e, who)| if *who == 1 { e.clone() } else { b.clone() });
            let theirs = join(&|(b, e, who)| if *who == 2 { e.clone() } else { b.clone() });
            let both = join(&|(b, e, who)| if *who == 0 { b.clone() } else { e.clone() });
            // Two neighbours both edited may meet at the separator's edge
            // (a diff can take it in): then it is fine to ask.
            match merge_apart(&base, &mine, &theirs) {
                Some(merged) => prop_assert_eq!(merged, both),
                None => prop_assert!(parts.windows(2).any(|w| w[0].2 != 0 && w[1].2 != 0 && w[0].2 != w[1].2)),
            }
        }

        /// Two different words typed at one place are a conflict, never one
        /// lost.
        #[test]
        fn two_insertions_at_one_place_ask(
            base in piece(), at in 0usize..8, a in "[xy]{1,3}", b in "[z]{1,3}",
        ) {
            let at = base.char_indices().map(|(i, _)| i).chain([base.len()]).nth(at).unwrap_or(base.len());
            let mut mine = base.clone();
            mine.insert_str(at, &a);
            let mut theirs = base.clone();
            theirs.insert_str(at, &b);
            prop_assert_eq!(merge_apart(&base, &mine, &theirs), None);
        }
    }
}
