//! The density system that keeps a page with ten possible sections calm:
//! four weights, caps per section, and one fold. Pure.
//!
//! - **Lead**: the agenda. Full rows with a time gutter.
//! - **Lists**: a label and up to three full rows.
//! - **Lines**: one block, *Also*, of at most four one-line items.
//! - **Coda**: one pull-quote at the end.
//!
//! When more than two Lists have something to say, the third becomes a Line
//! in *Also*, so a busy day stays a short page.

use super::model::{AlsoKind, AlsoLine, List, ListKind};

/// The agenda shows at most this many rows from now on.
pub const AGENDA_CAP: usize = 8;
/// Each List shows at most this many rows.
pub const LIST_CAP: usize = 3;
/// *Also* holds at most this many lines.
pub const ALSO_CAP: usize = 4;
/// At most this many Lists are drawn as Lists.
pub const LISTS_SHOWN: usize = 2;

/// How a section is drawn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Weight {
    Lead,
    Lists,
    Lines,
    Coda,
}

/// The sections of Today, in the fixed order they appear in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Section {
    Agenda,
    PickUp,
    Waiting,
    Activity,
    Also,
    Past,
}

impl Section {
    pub const ORDER: [Section; 6] = [
        Section::Agenda,
        Section::PickUp,
        Section::Waiting,
        Section::Activity,
        Section::Also,
        Section::Past,
    ];

    pub fn weight(self) -> Weight {
        match self {
            Section::Agenda => Weight::Lead,
            Section::PickUp | Section::Waiting | Section::Activity => Weight::Lists,
            Section::Also => Weight::Lines,
            Section::Past => Weight::Coda,
        }
    }
}

/// Which Lists are drawn as Lists and which one folded into a Line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Arrangement {
    pub lists: Vec<ListKind>,
    pub folded: Option<ListKind>,
}

/// Decides from the Lists that have content, in any order: the first two
/// in the fixed order stay Lists, a third folds. Nothing moves up out of
/// turn: a missing List simply closes its gap.
pub fn arrange(present: &[ListKind]) -> Arrangement {
    let mut kinds: Vec<ListKind> = present.to_vec();
    kinds.sort();
    kinds.dedup();
    let folded = (kinds.len() > LISTS_SHOWN).then(|| kinds.remove(LISTS_SHOWN));
    Arrangement {
        lists: kinds,
        folded,
    }
}

/// Keeps `cap` rows and counts the rest.
pub fn cap_rows<T>(mut rows: Vec<T>, cap: usize) -> (Vec<T>, u32) {
    let more = rows.len().saturating_sub(cap) as u32;
    rows.truncate(cap);
    (rows, more)
}

/// Puts the folded List's line first among the Lines (it is the most
/// timely), then the others, capped at [`ALSO_CAP`].
pub fn also_block(folded: Option<AlsoLine>, mut lines: Vec<AlsoLine>) -> Vec<AlsoLine> {
    lines.retain(|l| l.count > 0 || !l.items.is_empty());
    if let Some(line) = folded {
        lines.insert(0, line);
    }
    lines.truncate(ALSO_CAP);
    lines
}

/// The Lists as the page draws them: capped rows, empty ones dropped, at
/// most [`LISTS_SHOWN`], and the one that folded (if any).
pub fn lay_out(lists: Vec<List>) -> (Vec<List>, Option<List>) {
    let present: Vec<ListKind> = lists
        .iter()
        .filter(|l| !l.rows.is_empty())
        .map(|l| l.kind)
        .collect();
    let arrangement = arrange(&present);
    let mut shown = Vec::new();
    let mut folded = None;
    for mut list in lists {
        if list.rows.is_empty() {
            continue;
        }
        let (rows, extra) = cap_rows(std::mem::take(&mut list.rows), LIST_CAP);
        list.rows = rows;
        list.more += extra;
        if Some(list.kind) == arrangement.folded {
            folded = Some(list);
        } else {
            shown.push(list);
        }
    }
    shown.sort_by_key(|l| l.kind);
    (shown, folded)
}

/// The Line a folded List becomes.
pub fn fold_line(list: &List) -> AlsoLine {
    AlsoLine {
        kind: AlsoKind::Folded(list.kind),
        count: list.rows.len() as u32 + list.more,
        cards: 0,
        items: Vec::new(),
        detail: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn kinds() -> impl Strategy<Value = Vec<ListKind>> {
        prop::collection::vec(
            prop_oneof![
                Just(ListKind::PickUp),
                Just(ListKind::Waiting),
                Just(ListKind::Activity)
            ],
            0..6,
        )
    }

    proptest! {
        #![proptest_config(ProptestConfig { rng_seed: proptest::test_runner::RngSeed::Fixed(0x006b_616e_736f), ..ProptestConfig::with_cases(512) })]

        /// At most two Lists are drawn, in the fixed order, and with three
        /// the third folds; nothing is lost or invented.
        #[test]
        fn at_most_two_lists_and_the_third_folds(present in kinds()) {
            let a = arrange(&present);
            prop_assert!(a.lists.len() <= LISTS_SHOWN);
            prop_assert!(a.lists.windows(2).all(|w| w[0] < w[1]));
            let mut all: Vec<ListKind> = present.clone();
            all.sort();
            all.dedup();
            let mut back = a.lists.clone();
            back.extend(a.folded);
            back.sort();
            prop_assert_eq!(back, all.clone());
            prop_assert_eq!(a.folded.is_some(), all.len() > LISTS_SHOWN);
            // The folded one is the last in the fixed order.
            if let Some(f) = a.folded {
                prop_assert!(a.lists.iter().all(|k| *k < f));
            }
        }

        /// A cap keeps the first rows and counts the rest exactly.
        #[test]
        fn a_cap_counts_what_it_cuts(n in 0usize..40, cap in 0usize..12) {
            let (rows, more) = cap_rows((0..n).collect::<Vec<_>>(), cap);
            prop_assert_eq!(rows.len() + more as usize, n);
            prop_assert!(rows.len() <= cap);
            prop_assert!(rows.iter().enumerate().all(|(i, v)| i == *v));
        }
    }

    #[test]
    fn weights_follow_the_design() {
        assert_eq!(Section::Agenda.weight(), Weight::Lead);
        assert_eq!(Section::Waiting.weight(), Weight::Lists);
        assert_eq!(Section::Also.weight(), Weight::Lines);
        assert_eq!(Section::Past.weight(), Weight::Coda);
        assert!(Section::ORDER.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn also_holds_four_lines_with_the_fold_first() {
        let line = |kind| AlsoLine { kind, count: 1, cards: 0, items: vec![], detail: vec![] };
        let block = also_block(
            Some(line(AlsoKind::Folded(ListKind::Activity))),
            vec![
                line(AlsoKind::Revisit),
                line(AlsoKind::Captured),
                line(AlsoKind::Promises),
                line(AlsoKind::Themes),
            ],
        );
        assert_eq!(block.len(), ALSO_CAP);
        assert_eq!(block[0].kind, AlsoKind::Folded(ListKind::Activity));
    }
}
