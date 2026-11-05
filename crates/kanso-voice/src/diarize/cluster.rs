//! Putting voices together: embeddings that sound alike are one speaker.
//!
//! Average-linked agglomerative clustering on unit-length embeddings,
//! stopped at a distance. Nothing here knows how many people there are; the
//! distance decides. (Average linkage, not centroid: the voiceprints of one
//! person spread widely, and a centroid pulled toward the middle of two
//! people makes them look alike.)

/// The average straight-line distance between the unit embeddings of two
/// clusters above which they are different people. Chosen on recordings of
/// two to four people: one person's voiceprints average about 0.7 from each
/// other and two people's about 1.2.
pub const THRESHOLD: f32 = 0.95;

/// At most this many embeddings are clustered; the rest join the nearest
/// cluster afterwards. Keeps an eight-hour meeting from taking minutes.
pub const MAX_CLUSTERED: usize = 600;

/// Scales `v` to length one; a vector of nothing, or not a number, stays
/// as zeros. Pure.
pub fn unit(v: &[f32]) -> Vec<f32> {
    let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if !norm.is_finite() || norm == 0. {
        return vec![0.; v.len()];
    }
    v.iter().map(|x| x / norm).collect()
}

fn distance(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b)
        .map(|(x, y)| (x - y) * (x - y))
        .sum::<f32>()
        .sqrt()
}

/// One cluster while it grows.
struct Group {
    sum: Vec<f32>,
    count: usize,
    /// How much speech the members stand for.
    weight: f32,
    members: Vec<usize>,
}

impl Group {
    fn centroid(&self) -> Vec<f32> {
        self.sum.iter().map(|s| s / self.count as f32).collect()
    }
}

/// Which cluster each of `embeddings` belongs to: a number from 0 up, in
/// the order clusters first appear, so the same voices always get the same
/// numbers. Two clusters merge while the average distance between their
/// members is under `threshold`. A cluster of fewer than `min_size` embeddings is folded
/// into the nearest one that has enough, unless none has. Embeddings
/// beyond [`MAX_CLUSTERED`], taken evenly across the list, are not
/// clustered but placed with the nearest centroid. Pure.
pub fn cluster(embeddings: &[Vec<f32>], threshold: f32, min_size: usize) -> Vec<usize> {
    cluster_weighted(embeddings, &[], threshold, min_size as f32)
}

/// [`cluster`] where each embedding stands for `weights[i]` of speech
/// (seconds, say; 1 for any missing), and a cluster with less than
/// `min_weight` altogether is folded into the nearest one with enough. A
/// phantom "voice" that is only a second or two of crosstalk is not a
/// person. Pure.
pub fn cluster_weighted(
    embeddings: &[Vec<f32>],
    weights: &[f32],
    threshold: f32,
    min_weight: f32,
) -> Vec<usize> {
    let n = embeddings.len();
    if n == 0 {
        return Vec::new();
    }
    let unit_vectors: Vec<Vec<f32>> = embeddings.iter().map(|e| unit(e)).collect();
    // Which embeddings are clustered, evenly spread.
    let chosen: Vec<usize> = if n <= MAX_CLUSTERED {
        (0..n).collect()
    } else {
        (0..MAX_CLUSTERED).map(|i| i * n / MAX_CLUSTERED).collect()
    };
    let mut groups: Vec<Option<Group>> = chosen
        .iter()
        .map(|&i| {
            Some(Group {
                sum: unit_vectors[i].clone(),
                count: 1,
                weight: weights.get(i).copied().unwrap_or(1.),
                members: vec![i],
            })
        })
        .collect();
    // Average distances between live groups; merged ones are None.
    let m = groups.len();
    let mut dist = vec![f32::INFINITY; m * m];
    for a in 0..m {
        for b in a + 1..m {
            let d = distance(&unit_vectors[chosen[a]], &unit_vectors[chosen[b]]);
            dist[a * m + b] = d;
            dist[b * m + a] = d;
        }
    }
    loop {
        let mut best = (f32::INFINITY, 0usize, 0usize);
        for a in 0..m {
            if groups[a].is_none() {
                continue;
            }
            for b in a + 1..m {
                if groups[b].is_some() && dist[a * m + b] < best.0 {
                    best = (dist[a * m + b], a, b);
                }
            }
        }
        if best.0.is_nan() || best.0 >= threshold {
            break;
        }
        let (_, a, b) = best;
        let Some(gone) = groups[b].take() else { break };
        let Some(keep) = groups[a].as_mut() else {
            break;
        };
        let (na, nb) = (keep.count as f32, gone.count as f32);
        for (s, t) in keep.sum.iter_mut().zip(&gone.sum) {
            *s += t;
        }
        keep.count += gone.count;
        keep.weight += gone.weight;
        keep.members.extend(gone.members);
        // The average distance to the merged cluster is the members'
        // average (Lance-Williams).
        for c in 0..m {
            if c != a && groups[c].is_some() {
                let d = (na * dist[a * m + c] + nb * dist[b * m + c]) / (na + nb);
                dist[a * m + c] = d;
                dist[c * m + a] = d;
            }
        }
    }
    let mut live: Vec<Group> = groups.into_iter().flatten().collect();
    // Small clusters join the nearest big one, if there is a big one.
    if live.iter().any(|g| g.weight >= min_weight) {
        let (big, small): (Vec<Group>, Vec<Group>) =
            live.into_iter().partition(|g| g.weight >= min_weight);
        live = big;
        for g in small {
            let c = g.centroid();
            let nearest = (0..live.len())
                .min_by(|&x, &y| {
                    distance(&c, &live[x].centroid()).total_cmp(&distance(&c, &live[y].centroid()))
                })
                .unwrap_or(0);
            live[nearest].members.extend(g.members);
        }
    }
    // Everything not clustered goes to the nearest centroid.
    let live_centroids: Vec<Vec<f32>> = live.iter().map(Group::centroid).collect();
    let mut label = vec![usize::MAX; n];
    for (k, g) in live.iter().enumerate() {
        for &i in &g.members {
            label[i] = k;
        }
    }
    for i in 0..n {
        if label[i] == usize::MAX {
            label[i] = (0..live_centroids.len())
                .min_by(|&x, &y| {
                    distance(&unit_vectors[i], &live_centroids[x])
                        .total_cmp(&distance(&unit_vectors[i], &live_centroids[y]))
                })
                .unwrap_or(0);
        }
    }
    // Number clusters by first appearance.
    let mut order: Vec<usize> = Vec::new();
    for &l in &label {
        if !order.contains(&l) {
            order.push(l);
        }
    }
    label
        .iter()
        .map(|l| order.iter().position(|o| o == l).unwrap_or(0))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::{FileFailurePersistence, RngSeed};

    fn config(cases: u32) -> ProptestConfig {
        ProptestConfig {
            cases,
            rng_seed: RngSeed::Fixed(0x006b_616e_736f),
            failure_persistence: Some(Box::new(FileFailurePersistence::Off)),
            ..ProptestConfig::default()
        }
    }

    /// `count` points near the axis `axis` of 8 dimensions.
    fn near_axis(axis: usize, count: usize, wobble: &[f32]) -> Vec<Vec<f32>> {
        (0..count)
            .map(|i| {
                let mut v = vec![0.0f32; 8];
                v[axis] = 1.;
                v[(axis + 1) % 8] = wobble[i % wobble.len()] * 0.05;
                v
            })
            .collect()
    }

    proptest! {
        #![proptest_config(config(48))]

        /// One label per embedding, numbered 0 up without gaps, and by first
        /// appearance.
        #[test]
        fn prop_labels_are_dense_and_ordered(
            vectors in proptest::collection::vec(proptest::collection::vec(-1.0f32..1.0, 6), 0..40),
            threshold in 0.0f32..2.0,
            min_size in 1usize..5,
        ) {
            let labels = cluster(&vectors, threshold, min_size);
            prop_assert_eq!(labels.len(), vectors.len());
            let mut seen = 0usize;
            for &l in &labels {
                prop_assert!(l <= seen);
                if l == seen { seen += 1; }
            }
        }

        /// Groups pointing in clearly different directions come out as
        /// exactly those groups, whatever their size and wobble.
        #[test]
        fn prop_separate_voices_stay_separate(
            sizes in proptest::collection::vec(1usize..8, 2..6),
            wobble in proptest::collection::vec(-1.0f32..1.0, 1..8),
        ) {
            let mut vectors = Vec::new();
            let mut truth = Vec::new();
            for (axis, &size) in sizes.iter().enumerate() {
                vectors.extend(near_axis(axis, size, &wobble));
                truth.extend(std::iter::repeat_n(axis, size));
            }
            let labels = cluster(&vectors, THRESHOLD, 1);
            for i in 0..labels.len() {
                for j in 0..labels.len() {
                    prop_assert_eq!(labels[i] == labels[j], truth[i] == truth[j]);
                }
            }
        }

        /// Louder or quieter embeddings are the same voice: only direction
        /// counts.
        #[test]
        fn prop_length_does_not_matter(
            vectors in proptest::collection::vec(proptest::collection::vec(0.1f32..1.0, 4), 1..20),
            gains in proptest::collection::vec(0.1f32..10.0, 20),
        ) {
            let scaled: Vec<Vec<f32>> = vectors
                .iter()
                .enumerate()
                .map(|(i, v)| v.iter().map(|x| x * gains[i % gains.len()]).collect())
                .collect();
            prop_assert_eq!(
                cluster(&vectors, THRESHOLD, 1),
                cluster(&scaled, THRESHOLD, 1)
            );
        }

        /// Once any voice has enough speech, none that is left has less: a
        /// stretch of crosstalk is never a person of its own.
        #[test]
        fn prop_no_voice_is_left_below_the_floor(
            vectors in proptest::collection::vec(proptest::collection::vec(-1.0f32..1.0, 5), 1..30),
            weights in proptest::collection::vec(0.1f32..5.0, 30),
            floor in 0.5f32..8.0,
        ) {
            let labels = cluster_weighted(&vectors, &weights, THRESHOLD, floor);
            prop_assert_eq!(labels.len(), vectors.len());
            let k = labels.iter().max().map_or(0, |m| m + 1);
            let mut held = vec![0.0f32; k];
            for (i, &l) in labels.iter().enumerate() { held[l] += weights[i]; }
            if held.iter().any(|&w| w >= floor) {
                prop_assert!(held.iter().all(|&w| w >= floor - 1e-3), "{held:?} vs {floor}");
            }
        }

        /// A distance above the farthest two unit vectors can be (2) makes
        /// one voice of everything.
        #[test]
        fn prop_a_huge_threshold_is_one_voice(
            vectors in proptest::collection::vec(proptest::collection::vec(-1.0f32..1.0, 5), 1..30),
        ) {
            let labels = cluster(&vectors, 10., 1);
            prop_assert!(labels.iter().all(|&l| l == 0));
        }
    }

    #[test]
    fn nothing_in_nothing_out_and_a_lone_embedding_is_one_voice() {
        assert!(cluster(&[], THRESHOLD, 1).is_empty());
        assert_eq!(cluster(&[vec![1., 0.]], THRESHOLD, 3), vec![0]);
    }

    #[test]
    fn a_tiny_cluster_is_folded_into_a_big_one() {
        let mut vectors = near_axis(0, 6, &[0.1]);
        vectors.extend(near_axis(3, 1, &[0.1]));
        // Without a floor the stray is its own voice; with one it joins.
        assert_eq!(cluster(&vectors, THRESHOLD, 1).iter().max(), Some(&1));
        assert_eq!(cluster(&vectors, THRESHOLD, 3), vec![0; 7]);
    }

    #[test]
    fn a_long_meeting_is_clustered_on_a_sample_and_the_rest_placed() {
        let mut vectors = Vec::new();
        let mut truth = Vec::new();
        for i in 0..1_500 {
            let axis = (i / 3) % 2;
            vectors.extend(near_axis(axis, 1, &[(i % 7) as f32 / 7.]));
            truth.push(axis);
        }
        let labels = cluster(&vectors, THRESHOLD, 1);
        assert_eq!(labels.len(), 1_500);
        for i in 0..labels.len() {
            assert_eq!(labels[i] == labels[0], truth[i] == truth[0]);
        }
    }
}
