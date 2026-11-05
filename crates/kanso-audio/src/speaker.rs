//! Who is speaking, from the two channels of a recording with both sources
//! (issue #38, step 10): left is you, right is the meeting.
//!
//! This is the cheap and honest version of speaker labels: for each stretch
//! of speech the louder channel wins. It fails in crosstalk and for an
//! in-person meeting on one microphone, and says nothing when the channels
//! are equally loud (including both silent). Real diarization is step 13.

/// Who a stretch of a stereo recording is attributed to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Speaker {
    /// The microphone.
    You,
    /// System audio: the other people.
    Meeting,
}

impl Speaker {
    /// The word shown before the paragraph.
    pub fn label(self) -> &'static str {
        match self {
            Speaker::You => "You",
            Speaker::Meeting => "Meeting",
        }
    }
}

/// The sum of squares of `samples`, the energy that [`speaker_of`] compares.
pub fn energy(samples: &[f32]) -> f64 {
    samples.iter().map(|&s| f64::from(s) * f64::from(s)).sum()
}

/// The speaker for a stretch with `you` energy on the left and `meeting`
/// energy on the right: whichever has more, and nobody when they are equal
/// or not numbers. Pure.
pub fn speaker_of(you: f64, meeting: f64) -> Option<Speaker> {
    if !you.is_finite() || !meeting.is_finite() {
        return None;
    }
    match you.partial_cmp(&meeting)? {
        std::cmp::Ordering::Greater => Some(Speaker::You),
        std::cmp::Ordering::Less => Some(Speaker::Meeting),
        std::cmp::Ordering::Equal => None,
    }
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

        /// The label follows the channel with more energy, whatever the
        /// channels hold.
        #[test]
        fn prop_label_follows_the_louder_channel(
            you in proptest::collection::vec(-1.0f32..1.0, 1..200),
            meeting in proptest::collection::vec(-1.0f32..1.0, 1..200),
        ) {
            let (a, b) = (energy(&you), energy(&meeting));
            match speaker_of(a, b) {
                Some(Speaker::You) => prop_assert!(a > b),
                Some(Speaker::Meeting) => prop_assert!(b > a),
                None => prop_assert!(a == b),
            }
        }

        /// Swapping the channels swaps the label.
        #[test]
        fn prop_swapping_channels_swaps_the_label(a in 0.0f64..1e6, b in 0.0f64..1e6) {
            let swapped = speaker_of(b, a);
            match speaker_of(a, b) {
                Some(Speaker::You) => prop_assert_eq!(swapped, Some(Speaker::Meeting)),
                Some(Speaker::Meeting) => prop_assert_eq!(swapped, Some(Speaker::You)),
                None => prop_assert_eq!(swapped, None),
            }
        }

        /// Scaling both channels by the same gain never changes who wins.
        #[test]
        fn prop_gain_does_not_change_the_label(
            a in 0.0f64..1e6, b in 0.0f64..1e6, gain in 0.01f64..100.0,
        ) {
            prop_assert_eq!(speaker_of(a, b), speaker_of(a * gain * gain, b * gain * gain));
        }
    }

    #[test]
    fn silence_and_nonsense_have_no_speaker() {
        assert_eq!(speaker_of(0.0, 0.0), None);
        assert_eq!(speaker_of(f64::NAN, 1.0), None);
        assert_eq!(speaker_of(1.0, f64::INFINITY), None);
        assert_eq!(Speaker::You.label(), "You");
        assert_eq!(Speaker::Meeting.label(), "Meeting");
    }
}
