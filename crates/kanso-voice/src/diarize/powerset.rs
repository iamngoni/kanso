//! What the segmentation model says, as who is speaking.
//!
//! pyannote 3.0 does not give one yes or no per person. For each frame it
//! scores seven classes: silence; each of three people alone; and each pair
//! of the three talking together. The best-scoring class is the frame's
//! answer.

/// How many people the model tracks within one window.
pub const SPEAKERS: usize = 3;
/// Silence, three alone, three pairs.
pub const CLASSES: usize = 7;

/// Who is speaking in class `class`: nobody for 0, one person for 1 to 3,
/// two for 4 to 6 (0 and 1, 0 and 2, 1 and 2). Anything higher is nobody.
pub fn who(class: usize) -> [bool; SPEAKERS] {
    match class {
        1 => [true, false, false],
        2 => [false, true, false],
        3 => [false, false, true],
        4 => [true, true, false],
        5 => [true, false, true],
        6 => [false, true, true],
        _ => [false; SPEAKERS],
    }
}

/// The best class of one frame's scores: the first when they tie, and
/// silence when a score is not a number.
pub fn best(scores: &[f32]) -> usize {
    let mut best = (0usize, f32::NEG_INFINITY);
    for (class, &score) in scores.iter().enumerate().take(CLASSES) {
        if score.is_nan() {
            return 0;
        }
        if score > best.1 {
            best = (class, score);
        }
    }
    best.0
}

/// Who is speaking in each of the frames in `scores` (frames times
/// [`CLASSES`] numbers, frame by frame). A short last frame is ignored.
pub fn decode(scores: &[f32]) -> Vec<[bool; SPEAKERS]> {
    scores
        .as_chunks::<CLASSES>()
        .0
        .iter()
        .map(|frame| who(best(frame)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::{FileFailurePersistence, RngSeed};

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 64,
            rng_seed: RngSeed::Fixed(0x006b_616e_736f),
            failure_persistence: Some(Box::new(FileFailurePersistence::Off)),
            ..ProptestConfig::default()
        })]

        /// Never more than two people at once, and the answer is the class
        /// with the top score.
        #[test]
        fn prop_the_best_class_wins(scores in proptest::collection::vec(-20.0f32..20.0, CLASSES)) {
            let class = best(&scores);
            prop_assert!(class < CLASSES);
            prop_assert!(scores.iter().all(|&s| s <= scores[class]));
            prop_assert!(who(class).iter().filter(|&&on| on).count() <= 2);
        }

        /// One answer per whole frame, whatever the scores.
        #[test]
        fn prop_one_answer_per_frame(frames in 0usize..40, extra in 0usize..CLASSES) {
            let scores = vec![0.5f32; frames * CLASSES + extra];
            prop_assert_eq!(decode(&scores).len(), frames);
        }
    }

    #[test]
    fn the_classes_mean_what_pyannote_says() {
        assert_eq!(who(0), [false, false, false]);
        assert_eq!(who(2), [false, true, false]);
        assert_eq!(who(4), [true, true, false]);
        assert_eq!(who(6), [false, true, true]);
        assert_eq!(who(99), [false; 3]);
        assert_eq!(best(&[0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 0.0]), 2);
        assert_eq!(best(&[0.0, f32::NAN, 1.0, 0.0, 0.0, 0.0, 0.0]), 0);
    }
}
