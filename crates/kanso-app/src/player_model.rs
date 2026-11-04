//! What the recording player card shows, and the arithmetic behind its
//! scrubber (issue #38, step 5). Pure: the view only draws these.

use kanso_audio::play::SPEEDS;

/// Where the recording's bytes stand on this device.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Not opened yet: the card shows its controls and opens on Play.
    Idle,
    /// Reading the parts and opening the speaker.
    Opening,
    /// Open: it plays.
    Ready,
    /// A part's bytes haven't arrived by sync.
    NotHere,
    /// A part is gone, or the note is locked.
    Missing,
    /// It couldn't be opened or played; the words say why.
    Failed(String),
}

/// What the card tells the editor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayerEvent {
    /// A timestamp link was followed: the card has to be on show.
    Moment,
}

/// What the card draws: compared with the last, to redraw only on change.
#[derive(Clone, Debug, PartialEq)]
pub struct Shown {
    pub phase: Phase,
    pub playing: bool,
    pub position_secs: u64,
    pub length_secs: u64,
    pub speed_milli: u32,
    pub volume_milli: u32,
}

/// How far along a track `x` is, 0 to 1, for a track starting at `left` and
/// `width` wide. A drag past either end holds at the end. Pure.
pub fn fraction(x: f32, left: f32, width: f32) -> f32 {
    if !(width.is_finite() && width > 0.0 && x.is_finite() && left.is_finite()) {
        return 0.0;
    }
    ((x - left) / width).clamp(0.0, 1.0)
}

/// The moment `fraction` of the way along a recording, in microseconds.
/// Pure.
pub fn at_fraction(fraction: f32, length_us: u64) -> u64 {
    let fraction = if fraction.is_finite() {
        fraction.clamp(0.0, 1.0)
    } else {
        0.0
    };
    ((length_us as f64 * f64::from(fraction)).round() as u64).min(length_us)
}

/// How far along the recording is, 0 to 1. Pure.
pub fn progress(position_us: u64, length_us: u64) -> f32 {
    if length_us == 0 {
        return 0.0;
    }
    (position_us as f64 / length_us as f64).clamp(0.0, 1.0) as f32
}

/// The speed after `current` in the list the button cycles through; an
/// unlisted speed goes to the next one up. Pure.
pub fn next_speed(current: f32) -> f32 {
    SPEEDS
        .iter()
        .copied()
        .find(|s| *s > current + 0.001)
        .unwrap_or(SPEEDS[0])
}

/// `1×`, `0.75×`, `1.25×`. Pure.
pub fn speed_label(speed: f32) -> String {
    let text = format!("{speed:.2}");
    let text = text.trim_end_matches('0').trim_end_matches('.');
    format!("{text}×")
}

/// What `attachment` bytes of this type are called to the decoder.
/// Ogg Opus is told by its first page, so its hint is only a hint. Pure.
pub fn extension_for(content_type: &str) -> Option<&'static str> {
    match content_type.split(';').next().unwrap_or("").trim() {
        "audio/ogg" | "audio/opus" | "application/ogg" => Some("opus"),
        "audio/mp4" | "audio/x-m4a" | "audio/m4a" | "audio/aac" => Some("m4a"),
        "audio/mpeg" | "audio/mp3" => Some("mp3"),
        "audio/wav" | "audio/x-wav" | "audio/wave" => Some("wav"),
        "audio/flac" | "audio/x-flac" => Some("flac"),
        "audio/aiff" | "audio/x-aiff" => Some("aiff"),
        "audio/x-caf" => Some("caf"),
        _ => None,
    }
}

/// Whether a file of this type is one the player plays. Pure.
pub fn is_audio(content_type: &str) -> bool {
    content_type
        .trim()
        .to_ascii_lowercase()
        .starts_with("audio/")
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

        /// A drag anywhere lands inside the recording, and a point on the
        /// track lands in proportion.
        #[test]
        fn prop_scrubbing_stays_inside(
            x in -5_000f32..5_000.0,
            left in 0f32..1_000.0,
            width in 1f32..2_000.0,
            length in 0u64..40_000_000_000,
        ) {
            let f = fraction(x, left, width);
            prop_assert!((0.0..=1.0).contains(&f));
            let at = at_fraction(f, length);
            prop_assert!(at <= length);
            if x >= left && x <= left + width && length > 1_000_000 {
                let back = progress(at, length);
                prop_assert!((back - f).abs() < 1e-3);
            }
        }

        #[test]
        fn prop_nonsense_is_held_to_the_ends(x in any::<f32>(), left in any::<f32>(), width in any::<f32>(), f in any::<f32>(), length in any::<u64>()) {
            prop_assert!((0.0..=1.0).contains(&fraction(x, left, width)));
            prop_assert!(at_fraction(f, length) <= length);
            prop_assert!((0.0..=1.0).contains(&progress(length / 2, length)));
        }

        /// The speed button visits every speed and comes back.
        #[test]
        fn prop_the_speed_button_cycles(start in 0usize..5) {
            let mut speed = SPEEDS[start];
            let mut seen = Vec::new();
            for _ in 0..SPEEDS.len() {
                speed = next_speed(speed);
                seen.push(speed);
            }
            prop_assert_eq!(speed, SPEEDS[start]);
            seen.sort_by(|a, b| a.total_cmp(b));
            seen.dedup();
            prop_assert_eq!(seen.len(), SPEEDS.len());
        }
    }

    #[test]
    fn labels_and_types() {
        assert_eq!(speed_label(1.0), "1×");
        assert_eq!(speed_label(0.75), "0.75×");
        assert_eq!(speed_label(1.25), "1.25×");
        assert_eq!(speed_label(2.0), "2×");
        assert_eq!(extension_for("audio/ogg; codecs=opus"), Some("opus"));
        assert_eq!(extension_for("audio/mp4"), Some("m4a"));
        assert_eq!(extension_for("image/png"), None);
        assert!(is_audio("audio/ogg") && !is_audio("application/pdf"));
        assert_eq!(next_speed(1.1), 1.25);
        assert_eq!(next_speed(2.0), 0.75);
    }
}
