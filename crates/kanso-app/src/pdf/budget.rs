//! The pixel budget (issue #43, step 4): how much page bitmap the reader
//! keeps in memory, and which to let go when it is over. Pure, over any
//! payload, so the eviction is tested without textures.
//!
//! A page on screen is never let go (that would blank it); of the rest, the
//! one farthest from where the reader is goes first, the longest unused
//! among equals. Textures are never written to disk.

use std::collections::HashMap;
use std::ops::Range;

/// The most page bitmap kept for the column: 256 MB, some 60 pages of a
/// 2x display at Fit width.
pub const PAGE_BUDGET_BYTES: usize = 256 * 1024 * 1024;
/// The most the thumbnail rail keeps: 24 MB, a few hundred thumbnails.
pub const THUMB_BUDGET_BYTES: usize = 24 * 1024 * 1024;

struct Entry<V> {
    value: V,
    bytes: usize,
    used: u64,
}

/// What is kept, by page, within a limit.
pub struct PixelBudget<V> {
    limit: usize,
    total: usize,
    clock: u64,
    entries: HashMap<usize, Entry<V>>,
}

impl<V> PixelBudget<V> {
    pub fn new(limit: usize) -> Self {
        PixelBudget {
            limit,
            total: 0,
            clock: 0,
            entries: HashMap::new(),
        }
    }

    #[cfg(test)]
    pub fn limit(&self) -> usize {
        self.limit
    }

    /// The bytes held now.
    #[cfg(test)]
    pub fn used(&self) -> usize {
        self.total
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn get(&self, page: usize) -> Option<&V> {
        self.entries.get(&page).map(|entry| &entry.value)
    }

    /// Keeps `value` (`bytes` big) for `page`, handing back what it replaced.
    /// The caller evicts afterwards, knowing where the reader is.
    pub fn insert(&mut self, page: usize, value: V, bytes: usize) -> Option<V> {
        self.clock += 1;
        let old = self.entries.insert(
            page,
            Entry {
                value,
                bytes,
                used: self.clock,
            },
        );
        self.total += bytes;
        old.map(|old| {
            self.total -= old.bytes;
            old.value
        })
    }

    /// Notes that `page` was shown, so it is the last to go among equals.
    pub fn touch(&mut self, page: usize) {
        self.clock += 1;
        if let Some(entry) = self.entries.get_mut(&page) {
            entry.used = self.clock;
        }
    }

    /// Lets go of pages until the budget holds, never of those in `keep`
    /// (what is on screen), farthest from `focus` first and the longest
    /// unused among those as far. If `keep` alone is over the limit, it
    /// stays over: a page on screen is not blanked to save memory.
    pub fn evict(&mut self, keep: Range<usize>, focus: usize) -> Vec<V> {
        let mut gone = Vec::new();
        while self.total > self.limit {
            let victim = self
                .entries
                .iter()
                .filter(|(page, _)| !keep.contains(page))
                .max_by_key(|(page, entry)| (page.abs_diff(focus), u64::MAX - entry.used))
                .map(|(page, _)| *page);
            let Some(page) = victim else { break };
            if let Some(entry) = self.entries.remove(&page) {
                self.total -= entry.bytes;
                gone.push(entry.value);
            }
        }
        gone
    }

    /// Lets go of everything.
    pub fn clear(&mut self) -> Vec<V> {
        self.total = 0;
        self.entries.drain().map(|(_, entry)| entry.value).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// After eviction the budget holds, unless the pages on screen are
        /// alone over it; and the pages on screen are all still there.
        #[test]
        fn eviction_holds_the_budget_and_never_drops_the_visible(
            limit in 1usize..2000,
            sizes in prop::collection::vec(1usize..400, 1..60),
            visible_start in 0usize..60,
            visible_len in 0usize..5,
            focus in 0usize..60,
        ) {
            let mut budget = PixelBudget::new(limit);
            for (page, bytes) in sizes.iter().enumerate() {
                budget.insert(page, page, *bytes);
            }
            let keep = visible_start..visible_start + visible_len;
            let kept: usize = sizes
                .iter()
                .enumerate()
                .filter(|(page, _)| keep.contains(page))
                .map(|(_, bytes)| *bytes)
                .sum();
            budget.evict(keep.clone(), focus);
            prop_assert!(budget.used() <= limit.max(kept));
            for page in keep.filter(|page| *page < sizes.len()) {
                prop_assert!(budget.get(page).is_some(), "visible page {page} was dropped");
            }
            // The books agree with what is held.
            let held: usize = (0..sizes.len())
                .filter(|page| budget.get(*page).is_some())
                .map(|page| sizes[page])
                .sum();
            prop_assert_eq!(held, budget.used());
        }

        /// What goes is farther from the reader than what stays.
        #[test]
        fn the_farthest_pages_go_first(
            sizes in prop::collection::vec(1usize..100, 5..50),
            focus in 0usize..50,
            fraction in 0.2f64..0.9,
        ) {
            let total: usize = sizes.iter().sum();
            let mut budget = PixelBudget::new(((total as f64) * fraction) as usize);
            for (page, bytes) in sizes.iter().enumerate() {
                budget.insert(page, (), *bytes);
            }
            let before: Vec<usize> = (0..sizes.len()).collect();
            budget.evict(0..0, focus);
            let kept: Vec<usize> = before.iter().copied().filter(|p| budget.get(*p).is_some()).collect();
            let gone: Vec<usize> = before.iter().copied().filter(|p| budget.get(*p).is_none()).collect();
            if let (Some(far_kept), Some(near_gone)) = (
                kept.iter().map(|p| p.abs_diff(focus)).max(),
                gone.iter().map(|p| p.abs_diff(focus)).min(),
            ) {
                prop_assert!(far_kept <= near_gone, "kept {far_kept} away, let go {near_gone} away");
            }
        }

        /// Replacing a page's bitmap (a re-render at a new zoom) swaps its
        /// bytes; it does not add to them.
        #[test]
        fn replacing_a_page_swaps_its_bytes(a in 1usize..500, b in 1usize..500) {
            let mut budget = PixelBudget::new(10_000);
            budget.insert(3, 'a', a);
            let old = budget.insert(3, 'b', b);
            prop_assert_eq!(old, Some('a'));
            prop_assert_eq!(budget.used(), b);
            prop_assert_eq!(budget.len(), 1);
        }
    }

    #[test]
    fn among_equally_far_pages_the_longest_unused_goes() {
        let mut budget = PixelBudget::new(20);
        budget.insert(4, "four", 10);
        budget.insert(6, "six", 10);
        budget.insert(5, "five", 10);
        budget.touch(4);
        // 4 and 6 are as far from 5 as each other; 6 was used less recently.
        let gone = budget.evict(5..6, 5);
        assert_eq!(gone, vec!["six"]);
    }

    #[test]
    fn the_visible_stay_even_when_they_are_over_the_budget() {
        let mut budget = PixelBudget::new(10);
        budget.insert(0, (), 30);
        budget.insert(1, (), 30);
        budget.insert(2, (), 30);
        budget.evict(0..2, 0);
        assert_eq!(budget.len(), 2);
        assert_eq!(budget.used(), 60);
    }
}
