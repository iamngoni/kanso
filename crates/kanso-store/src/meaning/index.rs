//! The in-memory index of the notes' vectors that resurfacing searches.
//!
//! Resurfacing asks after every pause in typing, so reading and decoding
//! every vector from SQLite each time (as the inspector's Related tab can
//! afford to) is too slow. The index keeps each note's vector as int8 in
//! one array, loaded once on first use and kept up to date by
//! [`crate::Store::put_embeddings`]. A note that was deleted or locked
//! since may still be in it: [`crate::Store::resurface`] checks every
//! candidate against the database before showing it, and drops the ones
//! that no longer qualify.

use std::collections::HashMap;

use crate::passages::{dot_i8, quantize};

/// How many candidates are scored again in full precision.
pub const CANDIDATES: usize = 50;

/// The note vectors of one model, as int8.
#[derive(Default)]
pub struct Index {
    pub(crate) model: String,
    dim: usize,
    ids: Vec<String>,
    scales: Vec<f32>,
    numbers: Vec<i8>,
    at: HashMap<String, usize>,
}

impl Index {
    pub fn new(model: &str, dim: usize) -> Index {
        Index {
            model: model.to_owned(),
            dim,
            ..Index::default()
        }
    }

    pub fn len(&self) -> usize {
        self.ids.len()
    }

    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    pub fn dim(&self) -> usize {
        self.dim
    }

    /// Adds or replaces note `id`'s vector. A vector of the wrong length
    /// is left out.
    pub fn put(&mut self, id: &str, vector: &[f32]) {
        if self.dim == 0 {
            self.dim = vector.len();
        }
        if vector.len() != self.dim {
            self.remove(id);
            return;
        }
        let (scale, numbers) = quantize(vector);
        match self.at.get(id) {
            Some(&slot) => {
                self.scales[slot] = scale;
                self.numbers[slot * self.dim..(slot + 1) * self.dim].copy_from_slice(&numbers);
            }
            None => {
                self.at.insert(id.to_owned(), self.ids.len());
                self.ids.push(id.to_owned());
                self.scales.push(scale);
                self.numbers.extend_from_slice(&numbers);
            }
        }
    }

    /// Forgets note `id`.
    pub fn remove(&mut self, id: &str) {
        let Some(slot) = self.at.remove(id) else {
            return;
        };
        let last = self.ids.len() - 1;
        if slot != last {
            self.ids.swap(slot, last);
            self.scales.swap(slot, last);
            let (head, tail) = self.numbers.split_at_mut(last * self.dim);
            head[slot * self.dim..(slot + 1) * self.dim].swap_with_slice(&mut tail[..self.dim]);
            self.at.insert(self.ids[slot].clone(), slot);
        }
        self.ids.pop();
        self.scales.pop();
        self.numbers.truncate(last * self.dim);
    }

    /// The `k` notes nearest `vector` that score at least `floor`, apart
    /// from `skip`, nearest first (ties by ID).
    pub fn nearest(&self, vector: &[f32], skip: &str, floor: f32, k: usize) -> Vec<(String, f32)> {
        if self.dim == 0 || vector.len() != self.dim || k == 0 {
            return Vec::new();
        }
        let (scale, query) = quantize(vector);
        if scale == 0. {
            return Vec::new();
        }
        let mut best: Vec<(usize, f32)> = Vec::with_capacity(k + 1);
        let mut worst = f32::NEG_INFINITY;
        for (slot, row) in self.numbers.chunks_exact(self.dim).enumerate() {
            let score = dot_i8(&query, scale, row, self.scales[slot]);
            if score < floor || (best.len() == k && score <= worst) {
                continue;
            }
            if self.ids[slot] == skip {
                continue;
            }
            best.push((slot, score));
            if best.len() > k {
                best.sort_by(|a, b| b.1.total_cmp(&a.1));
                best.truncate(k);
            }
            if best.len() == k {
                worst = best.iter().map(|b| b.1).fold(f32::INFINITY, f32::min);
            }
        }
        let mut found: Vec<(String, f32)> = best
            .into_iter()
            .map(|(slot, score)| (self.ids[slot].clone(), score))
            .collect();
        found.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        found
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embed::cosine;
    use proptest::prelude::*;

    fn vector(seed: u64, dim: usize) -> Vec<f32> {
        let mut x = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (0..dim)
            .map(|_| {
                x = x
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                ((x >> 33) as f32 / (1u64 << 31) as f32) - 0.5
            })
            .collect()
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 48,
            failure_persistence: None,
            rng_seed: proptest::test_runner::RngSeed::Fixed(0x0069_6478),
            ..ProptestConfig::default()
        })]

        /// The index finds what an exact scan finds, never the skipped
        /// note, no more than k, nearest first, none below the floor.
        #[test]
        fn prop_nearest_matches_an_exact_scan(
            seeds in proptest::collection::vec(0u64..10_000, 1..40),
            query in 0u64..10_000,
            k in 1usize..8,
        ) {
            let mut index = Index::new("m", 32);
            let mut unique = std::collections::BTreeMap::new();
            for seed in seeds {
                unique.insert(format!("note:{seed}"), vector(seed, 32));
            }
            for (id, v) in &unique {
                index.put(id, v);
            }
            let q = vector(query, 32);
            let skip = unique.keys().next().cloned().unwrap();
            let found = index.nearest(&q, &skip, -1., k);
            prop_assert!(found.len() <= k);
            prop_assert!(found.iter().all(|(id, _)| *id != skip));
            prop_assert!(found.windows(2).all(|w| w[0].1 >= w[1].1));
            // The top hit's score is the exact cosine to within int8 error.
            if let Some((id, score)) = found.first() {
                prop_assert!((cosine(&q, &unique[id]) - score).abs() <= 0.02);
            }
            let high = index.nearest(&q, &skip, 0.5, k);
            prop_assert!(high.iter().all(|(_, s)| *s >= 0.5));
        }

        /// Removing a note leaves the others in place.
        #[test]
        fn prop_remove_keeps_the_others(count in 2usize..20, gone in 0usize..20) {
            let mut index = Index::new("m", 8);
            for n in 0..count {
                index.put(&format!("note:{n}"), &vector(n as u64, 8));
            }
            let gone = gone % count;
            index.remove(&format!("note:{gone}"));
            prop_assert_eq!(index.len(), count - 1);
            for n in (0..count).filter(|n| *n != gone) {
                let found = index.nearest(&vector(n as u64, 8), "", 0.95, 1);
                prop_assert_eq!(&found[0].0, &format!("note:{n}"));
            }
        }
    }

    /// The 50,000-note budget: a query scores them all in a few
    /// milliseconds (15 ms desktop, 40 ms phone). Run with `--ignored
    /// --release`; also in `kanso_engine::discovery::measure`.
    #[test]
    #[ignore = "benchmark: run with --release"]
    fn fifty_thousand_notes_are_scored_within_the_budget() {
        let mut index = Index::new("m", 256);
        for n in 0..50_000u64 {
            index.put(&format!("note:{n}"), &vector(n, 256));
        }
        let q = vector(99_999, 256);
        let start = std::time::Instant::now();
        let found = index.nearest(&q, "", 0.3, CANDIDATES);
        let took = start.elapsed();
        assert!(found.len() <= CANDIDATES);
        assert!(took.as_millis() <= 15, "took {took:?}");
    }
}
