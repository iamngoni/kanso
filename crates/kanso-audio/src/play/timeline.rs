//! The parts of a recording as one timeline, in microseconds.
//!
//! A recording longer than one part is several files played in order; the
//! player and the notes' `?t=` seconds see one clock. Pure.

/// Microseconds in a second.
pub const MICROS: u64 = 1_000_000;

/// Where a moment on the timeline falls in the parts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Place {
    /// The part, from 0.
    pub part: usize,
    /// Microseconds into that part.
    pub offset_us: u64,
}

/// The lengths of the parts, in order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Timeline {
    lengths_us: Vec<u64>,
}

impl Timeline {
    pub fn new(lengths_us: Vec<u64>) -> Self {
        Timeline { lengths_us }
    }

    pub fn parts(&self) -> usize {
        self.lengths_us.len()
    }

    /// The whole recording.
    pub fn total_us(&self) -> u64 {
        self.lengths_us.iter().sum()
    }

    /// Where part `part` starts on the timeline.
    pub fn start_of(&self, part: usize) -> u64 {
        self.lengths_us.iter().take(part).sum()
    }

    pub fn length_of(&self, part: usize) -> u64 {
        self.lengths_us.get(part).copied().unwrap_or(0)
    }

    /// `at` held within the recording.
    pub fn clamp(&self, at: u64) -> u64 {
        at.min(self.total_us())
    }

    /// The part holding `at` and the offset in it. A moment on a boundary
    /// belongs to the later part; the end belongs to the last. `None` for a
    /// recording with no parts.
    pub fn locate(&self, at: u64) -> Option<Place> {
        let at = self.clamp(at);
        let mut start = 0;
        let last = self.lengths_us.len().checked_sub(1)?;
        for (part, len) in self.lengths_us.iter().enumerate() {
            if at < start + len || part == last {
                return Some(Place {
                    part,
                    offset_us: (at - start).min(*len),
                });
            }
            start += len;
        }
        None
    }

    /// `at` moved by `delta_us` (negative goes back), held within the
    /// recording. Pure.
    pub fn shifted(&self, at: u64, delta_us: i64) -> u64 {
        let moved = if delta_us >= 0 {
            at.saturating_add(delta_us as u64)
        } else {
            at.saturating_sub(delta_us.unsigned_abs())
        };
        self.clamp(moved)
    }
}

/// Seconds as microseconds.
pub fn secs_to_us(secs: f64) -> u64 {
    if !secs.is_finite() || secs <= 0.0 {
        0
    } else {
        (secs * MICROS as f64).round() as u64
    }
}

/// `frames` at `rate` Hz as microseconds.
pub fn frames_to_us(frames: u64, rate: u32) -> u64 {
    if rate == 0 {
        return 0;
    }
    (u128::from(frames) * u128::from(MICROS) / u128::from(rate)) as u64
}

/// `us` as frames at `rate` Hz.
pub fn us_to_frames(us: u64, rate: u32) -> u64 {
    (u128::from(us) * u128::from(rate) / u128::from(MICROS)) as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::{FileFailurePersistence, RngSeed};

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256,
            rng_seed: RngSeed::Fixed(0x006b_616e_736f),
            failure_persistence: Some(Box::new(FileFailurePersistence::Off)),
            ..ProptestConfig::default()
        })]

        /// A located moment is inside its part, and adds back to the moment.
        #[test]
        fn prop_locate_round_trips(
            lengths in proptest::collection::vec(1u64..10_000_000, 1..6),
            at in 0u64..80_000_000,
        ) {
            let timeline = Timeline::new(lengths);
            let place = timeline.locate(at).unwrap();
            prop_assert!(place.part < timeline.parts());
            prop_assert!(place.offset_us <= timeline.length_of(place.part));
            prop_assert_eq!(
                timeline.start_of(place.part) + place.offset_us,
                timeline.clamp(at)
            );
        }

        /// Audio seeks and scrubs stay on the recording's frame clock across
        /// randomly sized parts, including the frame immediately around a seam.
        #[test]
        fn prop_frame_seek_and_scrub_cross_part_boundaries(
            lengths in proptest::collection::vec(1u64..2_000, 1..8),
            position in 0u32..=10_000,
        ) {
            const FRAME_US: u64 = 20_000;
            let timeline = Timeline::new(lengths.iter().map(|frames| frames * FRAME_US).collect());
            let total_frames = lengths.iter().sum::<u64>();
            let target = total_frames * u64::from(position) / 10_000;
            let place = timeline.locate(target * FRAME_US).unwrap();
            let landed = timeline.start_of(place.part) / FRAME_US + place.offset_us / FRAME_US;
            prop_assert!(landed <= target);
            prop_assert!(target - landed <= 1, "seek missed by {} frames", target - landed);

            let mut boundary = 0;
            for (part, frames) in lengths.iter().enumerate().take(lengths.len() - 1) {
                boundary += frames;
                let before = timeline.locate((boundary - 1) * FRAME_US).unwrap();
                let at = timeline.locate(boundary * FRAME_US).unwrap();
                prop_assert_eq!(before.part, part);
                prop_assert_eq!(at.part, part + 1);
                prop_assert_eq!(at.offset_us, 0);
                // A frame past the boundary is still in the next part only
                // when that part is longer than one frame.
                if lengths[part + 1] > 1 {
                    let after = timeline.locate((boundary + 1) * FRAME_US).unwrap();
                    prop_assert_eq!(after.part, part + 1);
                    prop_assert_eq!(after.offset_us, FRAME_US);
                }
            }
        }

        /// Moving never leaves the recording, and forward never goes back.
        #[test]
        fn prop_shifted_stays_inside(
            lengths in proptest::collection::vec(0u64..10_000_000, 0..6),
            at in 0u64..80_000_000,
            delta in -50_000_000i64..50_000_000,
        ) {
            let timeline = Timeline::new(lengths);
            let moved = timeline.shifted(at, delta);
            prop_assert!(moved <= timeline.total_us());
            if delta > 0 {
                prop_assert!(moved >= timeline.clamp(at));
            }
        }

        #[test]
        fn prop_frames_and_micros_agree(frames in 0u64..10_000_000_000, rate in 8_000u32..192_000) {
            let back = us_to_frames(frames_to_us(frames, rate), rate);
            prop_assert!(back <= frames);
            prop_assert!(frames - back <= 1 + u64::from(rate) / MICROS);
        }
    }

    #[test]
    fn a_boundary_belongs_to_the_later_part() {
        let t = Timeline::new(vec![1_000, 2_000]);
        assert_eq!(
            t.locate(1_000),
            Some(Place {
                part: 1,
                offset_us: 0
            })
        );
        assert_eq!(
            t.locate(999),
            Some(Place {
                part: 0,
                offset_us: 999
            })
        );
        assert_eq!(
            t.locate(99_999),
            Some(Place {
                part: 1,
                offset_us: 2_000
            })
        );
        assert_eq!(Timeline::default().locate(5), None);
    }
}
