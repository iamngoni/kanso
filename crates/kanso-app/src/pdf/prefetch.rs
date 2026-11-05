//! The order pages are asked of the worker in (issue #43, step 4). Pure, so
//! the order is tested without a worker.
//!
//! What is on screen comes first, top to bottom, since that is how it is
//! read. Then the pages the reader is heading for, nearest first, with the
//! one behind interleaved so scrolling back is quick too. Anything that is
//! already drawn at the right size is left out.

use std::ops::Range;

/// How many pages past the window are drawn ahead, in the direction of the
/// scroll.
pub const AHEAD: usize = 3;
/// How many are drawn behind it.
pub const BEHIND: usize = 1;

/// Which way the reader is going.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Heading {
    #[default]
    Down,
    Up,
}

impl Heading {
    /// The heading a move from `before` to `after` pixels down the column
    /// means, or `current` when it did not move.
    pub fn after(current: Heading, before: f32, after: f32) -> Heading {
        if after > before {
            Heading::Down
        } else if after < before {
            Heading::Up
        } else {
            current
        }
    }
}

/// The pages to draw, in the order to ask for them: the visible ones, then
/// [`AHEAD`] pages along `heading` and [`BEHIND`] against it, leaving out
/// those `have` says are already right. Only pages in `0..pages`, each once.
pub fn plan(
    visible: Range<usize>,
    pages: usize,
    heading: Heading,
    have: impl Fn(usize) -> bool,
) -> Vec<usize> {
    let visible = visible.start.min(pages)..visible.end.min(pages);
    let mut order: Vec<usize> = visible.clone().collect();
    if !visible.is_empty() {
        let (before, after) = (visible.start, visible.end);
        // `forward` is the side the reader is heading to.
        let (forward, backward): (Vec<usize>, Vec<usize>) = match heading {
            Heading::Down => (
                (after..(after + AHEAD).min(pages)).collect(),
                (before.saturating_sub(BEHIND)..before).rev().collect(),
            ),
            Heading::Up => (
                (before.saturating_sub(AHEAD)..before).rev().collect(),
                (after..(after + BEHIND).min(pages)).collect(),
            ),
        };
        // Nearest first, the side being headed for leading.
        let mut forward = forward.into_iter();
        let mut backward = backward.into_iter();
        loop {
            let (a, b) = (forward.next(), backward.next());
            if a.is_none() && b.is_none() {
                break;
            }
            order.extend(a.into_iter().chain(b));
        }
    }
    order.retain(|page| !have(*page));
    order
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn heading() -> impl Strategy<Value = Heading> {
        prop_oneof![Just(Heading::Down), Just(Heading::Up)]
    }

    proptest! {
        /// Every page asked for exists, none twice, and the visible ones that
        /// are missing come first, top to bottom, before any other.
        #[test]
        fn the_plan_is_in_range_unique_and_visible_first(
            pages in 0usize..200,
            start in 0usize..220,
            len in 0usize..8,
            heading in heading(),
            have in prop::collection::hash_set(0usize..200, 0..40),
        ) {
            let visible = start..start + len;
            let order = plan(visible.clone(), pages, heading, |p| have.contains(&p));
            let mut seen = std::collections::HashSet::new();
            for page in &order {
                prop_assert!(*page < pages);
                prop_assert!(seen.insert(*page), "{page} twice");
                prop_assert!(!have.contains(page));
            }
            let wanted: Vec<usize> = visible
                .filter(|p| *p < pages && !have.contains(p))
                .collect();
            let asked: Vec<usize> = order.iter().copied().take(wanted.len()).collect();
            prop_assert_eq!(asked, wanted);
        }

        /// Nothing is asked for beyond the lookahead.
        #[test]
        fn the_plan_stays_near_the_window(
            pages in 1usize..200,
            start in 0usize..200,
            len in 1usize..8,
            heading in heading(),
        ) {
            let from = start.min(pages - 1);
            let visible = from..(from + len).min(pages);
            let order = plan(visible.clone(), pages, heading, |_| false);
            for page in order {
                prop_assert!(
                    page + AHEAD >= visible.start && page < visible.end + AHEAD,
                    "{page} is far from {visible:?}"
                );
            }
        }

        /// Pages ahead are asked for nearest first.
        #[test]
        fn the_pages_ahead_come_nearest_first(pages in 20usize..100, start in 0usize..10) {
            let visible = start..start + 2;
            let order = plan(visible.clone(), pages, Heading::Down, |_| false);
            let ahead: Vec<usize> = order.into_iter().filter(|p| *p >= visible.end).collect();
            let mut sorted = ahead.clone();
            sorted.sort_unstable();
            prop_assert_eq!(ahead, sorted);
        }
    }

    #[test]
    fn heading_down_draws_more_ahead_and_heading_up_more_behind() {
        let down = plan(10..12, 50, Heading::Down, |_| false);
        assert_eq!(down, vec![10, 11, 12, 9, 13, 14]);
        let up = plan(10..12, 50, Heading::Up, |_| false);
        assert_eq!(up, vec![10, 11, 9, 12, 8, 7]);
    }

    #[test]
    fn what_is_drawn_already_is_not_asked_again() {
        let order = plan(0..2, 5, Heading::Down, |page| page == 1 || page == 3);
        assert_eq!(order, vec![0, 2, 4]);
    }

    #[test]
    fn the_heading_follows_the_scroll() {
        assert_eq!(Heading::after(Heading::Down, 10., 5.), Heading::Up);
        assert_eq!(Heading::after(Heading::Up, 5., 10.), Heading::Down);
        assert_eq!(Heading::after(Heading::Up, 5., 5.), Heading::Up);
    }
}
