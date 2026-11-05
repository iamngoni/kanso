//! From window by window answers to who spoke when.
//!
//! The segmentation model sees ten seconds at a time, and its three
//! "speakers" in one window are not the three of the next: they are only
//! labels for that window. Clustering the voices tells which window label
//! is which person; here every window votes, frame by frame, for the people
//! it heard, and the majority makes the timeline.

use super::model::Turn;
use super::powerset::SPEAKERS;

/// Samples per segmentation frame at 16 kHz (the model's receptive field
/// shift).
pub const FRAME_SAMPLES: usize = 270;
/// Frames per second of audio, as a divisor: a frame lasts this many
/// seconds.
pub const FRAME_SECONDS: f64 = FRAME_SAMPLES as f64 / 16_000.;

/// Pauses shorter than this inside one person's speech are bridged (half a
/// second).
pub const MAX_GAP_FRAMES: usize = 30;
/// A stretch shorter than this is a cough or a click, not a turn (0.3 s).
pub const MIN_TURN_FRAMES: usize = 18;

/// What one segmentation window said, once its people are known.
#[derive(Clone, Debug, PartialEq)]
pub struct Window {
    /// The recording's frame where the window starts.
    pub first_frame: usize,
    /// For each of the window's frames, who is speaking among the window's
    /// three labels.
    pub active: Vec<[bool; SPEAKERS]>,
    /// Which person each of the window's three labels is, when known.
    pub person: [Option<usize>; SPEAKERS],
}

/// Which frames each of `people` is speaking in: a frame counts for a
/// person when at least half of the windows that cover it heard them.
/// Frames at or past `frames` are ignored. Pure.
pub fn votes(windows: &[Window], people: usize, frames: usize) -> Vec<Vec<bool>> {
    let mut covered = vec![0u32; frames];
    let mut heard = vec![vec![0u32; frames]; people];
    for window in windows {
        for (offset, labels) in window.active.iter().enumerate() {
            let frame = window.first_frame + offset;
            if frame >= frames {
                break;
            }
            covered[frame] += 1;
            for (label, &on) in labels.iter().enumerate() {
                if let (true, Some(person)) = (on, window.person[label])
                    && person < people
                {
                    heard[person][frame] += 1;
                }
            }
        }
    }
    heard
        .into_iter()
        .map(|counts| {
            counts
                .iter()
                .zip(&covered)
                .map(|(&h, &c)| h > 0 && h * 2 >= c)
                .collect()
        })
        .collect()
}

/// The stretches of `on`: gaps of `max_gap` frames or fewer are bridged,
/// then stretches shorter than `min_len` are dropped. Each is a range of
/// frames, in order. Pure.
pub fn runs(on: &[bool], max_gap: usize, min_len: usize) -> Vec<std::ops::Range<usize>> {
    let mut stretches: Vec<std::ops::Range<usize>> = Vec::new();
    let mut frame = 0;
    while frame < on.len() {
        if !on[frame] {
            frame += 1;
            continue;
        }
        let start = frame;
        while frame < on.len() && on[frame] {
            frame += 1;
        }
        match stretches.last_mut() {
            Some(last) if start - last.end <= max_gap => last.end = frame,
            _ => stretches.push(start..frame),
        }
    }
    stretches.retain(|r| r.len() >= min_len);
    stretches
}

/// Everyone's turns, in start order. Pure.
pub fn turns(windows: &[Window], people: usize, frames: usize) -> Vec<Turn> {
    let mut out: Vec<Turn> = votes(windows, people, frames)
        .iter()
        .enumerate()
        .flat_map(|(speaker, on)| {
            runs(on, MAX_GAP_FRAMES, MIN_TURN_FRAMES)
                .into_iter()
                .map(move |r| Turn {
                    start: r.start as f64 * FRAME_SECONDS,
                    end: r.end as f64 * FRAME_SECONDS,
                    speaker,
                })
        })
        .collect();
    out.sort_by(|a, b| a.start.total_cmp(&b.start).then(a.speaker.cmp(&b.speaker)));
    out
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

    proptest! {
        #![proptest_config(config(96))]

        /// Stretches are in order, apart by more than the gap, at least the
        /// minimum long, and start and end on a frame that was on.
        #[test]
        fn prop_runs_are_ordered_apart_and_long_enough(
            on in proptest::collection::vec(any::<bool>(), 0..200),
            gap in 0usize..6,
            min in 1usize..6,
        ) {
            let found = runs(&on, gap, min);
            for pair in found.windows(2) {
                prop_assert!(pair[1].start > pair[0].end + gap);
            }
            for r in &found {
                prop_assert!(r.len() >= min);
                prop_assert!(on[r.start] && on[r.end - 1]);
                prop_assert!(r.end <= on.len());
            }
        }

        /// Every frame that was on and sits in a stretch that is long
        /// enough to keep is inside some found stretch.
        #[test]
        fn prop_a_long_solid_stretch_is_always_kept(
            lead in 0usize..10, solid in 6usize..20, tail in 0usize..10,
        ) {
            let mut on = vec![false; lead];
            on.extend(std::iter::repeat_n(true, solid));
            on.extend(std::iter::repeat_n(false, tail));
            prop_assert_eq!(runs(&on, 2, 5), vec![lead..lead + solid]);
        }

        /// Turns come out in start order, inside the recording, and never
        /// negative in length.
        #[test]
        fn prop_turns_are_in_order_and_inside_the_recording(
            frames in 1usize..400,
            shapes in proptest::collection::vec(
                (0usize..300, proptest::collection::vec(any::<[bool; 3]>(), 1..120), any::<[Option<u8>; 3]>()),
                0..8,
            ),
        ) {
            let windows: Vec<Window> = shapes
                .into_iter()
                .map(|(first_frame, active, person)| Window {
                    first_frame,
                    active,
                    person: person.map(|p| p.map(|p| usize::from(p % 3))),
                })
                .collect();
            let found = turns(&windows, 3, frames);
            let end = frames as f64 * FRAME_SECONDS + 1e-9;
            for t in &found {
                prop_assert!(t.start < t.end && t.end <= end && t.speaker < 3);
            }
            for pair in found.windows(2) {
                prop_assert!(pair[0].start <= pair[1].start);
            }
        }

        /// Swapping the names of two people swaps their turns and changes
        /// nothing else.
        #[test]
        fn prop_naming_people_differently_only_renames_turns(
            active in proptest::collection::vec(any::<[bool; 3]>(), 30..90),
        ) {
            let make = |person| vec![Window { first_frame: 0, active: active.clone(), person }];
            let a = turns(&make([Some(0), Some(1), Some(2)]), 3, 90);
            let b = turns(&make([Some(2), Some(1), Some(0)]), 3, 90);
            let swapped: Vec<_> = a
                .iter()
                .map(|t| Turn { speaker: 2 - t.speaker, ..t.clone() })
                .collect();
            let mut b = b;
            b.sort_by(|x, y| x.start.total_cmp(&y.start).then(x.speaker.cmp(&y.speaker)));
            let mut swapped = swapped;
            swapped.sort_by(|x, y| x.start.total_cmp(&y.start).then(x.speaker.cmp(&y.speaker)));
            prop_assert_eq!(b, swapped);
        }
    }

    #[test]
    fn overlapping_windows_vote_and_the_majority_wins() {
        // Three windows cover frame 10; two heard person 0, one did not.
        let heard = |on| Window {
            first_frame: 8,
            active: vec![[on, false, false]; 5],
            person: [Some(0), None, None],
        };
        let found = votes(&[heard(true), heard(true), heard(false)], 1, 20);
        assert!(found[0][10] && found[0][8] && !found[0][7] && !found[0][13]);
        let found = votes(&[heard(true), heard(false), heard(false)], 1, 20);
        assert!(!found[0][10]);
    }

    #[test]
    fn a_label_without_a_person_votes_for_nobody() {
        let window = Window {
            first_frame: 0,
            active: vec![[true, true, true]; 4],
            person: [None, None, None],
        };
        assert!(votes(&[window], 2, 4).iter().all(|p| p.iter().all(|&b| !b)));
    }

    #[test]
    fn a_turn_is_timed_in_seconds() {
        let window = Window {
            first_frame: 0,
            active: vec![[true, false, false]; 100],
            person: [Some(0), None, None],
        };
        let found = turns(&[window], 1, 100);
        assert_eq!(found.len(), 1);
        assert!((found[0].end - 100. * FRAME_SECONDS).abs() < 1e-9);
    }
}
