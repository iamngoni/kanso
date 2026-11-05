//! Recording one app's audio (issue #38, step 7): the plain logic around the
//! platform's capture, beside the platform-neutral [`crate::system_audio`].
//!
//! The capture itself is platform code (`system_audio_mac` is ScreenCaptureKit);
//! everything here is pure so it is tested without a sound card or a
//! permission: which apps the picker lists and in what order, turning the
//! platform's buffers into one mono channel, and the silence that keeps an
//! app that isn't playing on the same clock as the microphone.
//!
//! System audio is recorded as a track of its own, mono at 48 kHz, in its own
//! session folder next to the microphone's. Step 10 mixes the two onto one
//! timeline; this module only makes sure the track's clock is the wall
//! clock, so there is something to mix.

use std::time::Duration;

use crate::model::SAMPLE_RATE;

/// An app whose audio can be captured: what the picker lists.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct CaptureApp {
    /// What the system calls the app, such as `us.zoom.xos`.
    pub bundle_id: String,
    /// What the person calls it, such as `zoom.us`.
    pub name: String,
}

/// Apps people hold meetings in, by bundle ID. The picker lists these first.
/// Browsers are here because Meet, Teams and Zoom run in them; capturing a
/// browser captures every tab (the audio is per app, not per window).
pub const MEETING_APPS: &[&str] = &[
    "us.zoom.xos",
    "com.microsoft.teams2",
    "com.microsoft.teams",
    "com.tinyspeck.slackmacgap",
    "com.apple.FaceTime",
    "Cisco-Systems.Spark",
    "com.hnc.Discord",
    "com.skype.skype",
    "com.google.Chrome",
    "com.apple.Safari",
    "company.thebrowser.Browser",
    "org.mozilla.firefox",
    "com.microsoft.edgemac",
    "com.brave.Browser",
];

/// The picker's list: one entry per app, the meeting apps first (in the
/// order of [`MEETING_APPS`]), then the rest by name. Apps with no name or
/// no bundle ID can't be shown or captured and are left out. Pure.
pub fn picker_order(mut apps: Vec<CaptureApp>) -> Vec<CaptureApp> {
    apps.retain(|app| !app.bundle_id.is_empty() && !app.name.trim().is_empty());
    let rank = |app: &CaptureApp| {
        MEETING_APPS
            .iter()
            .position(|id| *id == app.bundle_id)
            .unwrap_or(MEETING_APPS.len())
    };
    apps.sort_by(|a, b| {
        rank(a)
            .cmp(&rank(b))
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
            .then_with(|| a.bundle_id.cmp(&b.bundle_id))
    });
    apps.dedup_by(|b, a| a.bundle_id == b.bundle_id);
    apps
}

/// One buffer of samples as the platform hands it over.
#[derive(Clone, Copy, Debug)]
pub struct PcmBuffer<'a> {
    pub samples: &'a [f32],
    /// Channels interleaved in this buffer: 1 for a planar layout, where
    /// every channel has a buffer of its own.
    pub channels: usize,
}

/// Appends one mono channel made from `buffers` to `out`: the mean of every
/// channel of every buffer, frame by frame. A planar stereo callback (two
/// buffers of one channel) and an interleaved one (one buffer of two) give
/// the same result. Buffers of unequal length are cut to the shortest, and a
/// buffer with no channels is ignored. Returns the frames appended. Pure.
pub fn append_mono(buffers: &[PcmBuffer<'_>], out: &mut Vec<f32>) -> usize {
    let buffers: Vec<PcmBuffer<'_>> = buffers.iter().copied().filter(|b| b.channels > 0).collect();
    let Some(frames) = buffers.iter().map(|b| b.samples.len() / b.channels).min() else {
        return 0;
    };
    let total_channels: usize = buffers.iter().map(|b| b.channels).sum();
    let scale = 1.0 / total_channels as f32;
    out.reserve(frames);
    for frame in 0..frames {
        let mut sum = 0.0f32;
        for buffer in &buffers {
            let at = frame * buffer.channels;
            sum += buffer.samples[at..at + buffer.channels].iter().sum::<f32>();
        }
        out.push(sum * scale);
    }
    frames
}

/// How far the track may lag the wall clock before silence is written: 100
/// ms, the design's drift limit.
pub const GAP_TOLERANCE: Duration = Duration::from_millis(100);

/// Keeps the system track on the wall clock.
///
/// An app that isn't playing may send no buffers at all, so a recording of
/// "nothing" would be shorter than the microphone's and every later
/// timestamp would be off. The filler counts time that passed while
/// recording (pause excluded, as the recorder's clock excludes it) and says
/// how much silence is owed to catch up. It never asks for the track to be
/// cut when it is ahead: aligning that is step 10's.
#[derive(Clone, Debug, Default)]
pub struct GapFiller {
    /// Samples that should have been written by now, at 48 kHz.
    due: f64,
}

impl GapFiller {
    pub fn new() -> Self {
        Self::default()
    }

    /// Time passed. Nothing is due while `paused`.
    pub fn advance(&mut self, dt: Duration, paused: bool) {
        if !paused {
            self.due += dt.as_secs_f64() * f64::from(SAMPLE_RATE);
        }
    }

    /// Samples of silence to write now, given the `written` so far (pause
    /// excluded). Zero within [`GAP_TOLERANCE`] of the clock.
    pub fn owed(&self, written: u64) -> usize {
        let tolerance = GAP_TOLERANCE.as_secs_f64() * f64::from(SAMPLE_RATE);
        let behind = self.due - written as f64;
        if behind > tolerance {
            behind.floor() as usize
        } else {
            0
        }
    }
}

/// Whether Screen Recording is allowed, and what the sheet's permission row
/// says about it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScreenAccess {
    Allowed,
    /// Not allowed, or not asked yet: the system doesn't tell the two apart
    /// without asking.
    NotAllowed,
}

/// Where to turn the permission on.
pub const SETTINGS_PATH: &str =
    "System Settings › Privacy & Security › Screen & System Audio Recording";

/// What the permission row says. `asked` is whether the person has been
/// asked in this sheet already. Pure.
pub fn access_words(access: ScreenAccess, asked: bool) -> String {
    match access {
        ScreenAccess::Allowed => {
            "Screen Recording is allowed. Kanso uses it only to hear the app's audio and never records the screen."
                .to_owned()
        }
        ScreenAccess::NotAllowed if !asked => {
            "macOS needs your permission to let Kanso hear another app. It asks once, and Kanso never records the screen."
                .to_owned()
        }
        ScreenAccess::NotAllowed => format!(
            "Kanso can't hear other apps yet. Turn Kanso on in {SETTINGS_PATH}, then reopen Kanso. The microphone records either way."
        ),
    }
}

/// The notice when a recording that asked for system audio can't have it
/// and goes on with the microphone alone. Pure.
pub fn denied_words() -> String {
    format!(
        "Kanso isn't allowed to record other apps' audio, so only the microphone is recording. Turn Kanso on in {SETTINGS_PATH}, then reopen Kanso."
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::{FileFailurePersistence, RngSeed};

    fn app(id: &str, name: &str) -> CaptureApp {
        CaptureApp {
            bundle_id: id.into(),
            name: name.into(),
        }
    }

    #[test]
    fn meeting_apps_come_first_then_the_rest_by_name() {
        let listed = picker_order(vec![
            app("com.apple.Notes", "Notes"),
            app("com.google.Chrome", "Google Chrome"),
            app("us.zoom.xos", "zoom.us"),
            app("com.apple.Music", "Music"),
        ]);
        let names: Vec<_> = listed.iter().map(|a| a.name.as_str()).collect();
        assert_eq!(names, ["zoom.us", "Google Chrome", "Music", "Notes"]);
    }

    #[test]
    fn planar_and_interleaved_stereo_mix_the_same() {
        let (l, r) = ([0.2f32, 0.4, 0.6], [0.0f32, 0.2, 0.4]);
        let inter = [0.2f32, 0.0, 0.4, 0.2, 0.6, 0.4];
        let mut planar = Vec::new();
        append_mono(
            &[
                PcmBuffer {
                    samples: &l,
                    channels: 1,
                },
                PcmBuffer {
                    samples: &r,
                    channels: 1,
                },
            ],
            &mut planar,
        );
        let mut interleaved = Vec::new();
        append_mono(
            &[PcmBuffer {
                samples: &inter,
                channels: 2,
            }],
            &mut interleaved,
        );
        assert_eq!(planar.len(), 3);
        for (a, b) in planar.iter().zip(&interleaved) {
            assert!((a - b).abs() < 1e-6);
        }
        assert!((planar[0] - 0.1).abs() < 1e-6);
    }

    #[test]
    fn nothing_owed_until_the_track_lags_by_more_than_the_tolerance() {
        let mut filler = GapFiller::new();
        filler.advance(Duration::from_millis(90), false);
        assert_eq!(filler.owed(0), 0);
        filler.advance(Duration::from_millis(60), false);
        assert_eq!(filler.owed(0), 7_200);
        assert_eq!(filler.owed(7_200), 0);
    }

    #[test]
    fn a_pause_owes_nothing() {
        let mut filler = GapFiller::new();
        filler.advance(Duration::from_secs(30), true);
        assert_eq!(filler.owed(0), 0);
    }

    #[test]
    fn the_words_name_where_to_turn_it_on() {
        let denied = access_words(ScreenAccess::NotAllowed, true);
        assert!(
            denied.contains("Screen & System Audio Recording"),
            "{denied}"
        );
        assert!(denied.contains("microphone"), "{denied}");
        assert!(denied_words().contains("only the microphone is recording"));
        assert!(access_words(ScreenAccess::NotAllowed, false).contains("asks once"));
    }

    fn app_strategy() -> impl Strategy<Value = CaptureApp> {
        (
            prop_oneof![
                Just(String::new()),
                "[a-z]{1,3}\\.[a-z]{1,3}",
                proptest::sample::select(MEETING_APPS).prop_map(str::to_owned),
            ],
            "[ A-Za-z]{0,8}",
        )
            .prop_map(|(bundle_id, name)| CaptureApp { bundle_id, name })
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 128,
            rng_seed: RngSeed::Fixed(0x006b_616e_736f),
            failure_persistence: Some(Box::new(FileFailurePersistence::Off)),
            ..ProptestConfig::default()
        })]

        /// The picker lists each usable app once, meeting apps before the
        /// others, and nothing it was not given.
        #[test]
        fn prop_picker_lists_each_usable_app_once_meeting_apps_first(
            apps in proptest::collection::vec(app_strategy(), 0..24),
        ) {
            let listed = picker_order(apps.clone());
            let mut ids: Vec<_> = listed.iter().map(|a| a.bundle_id.clone()).collect();
            let before = ids.len();
            ids.sort();
            ids.dedup();
            prop_assert_eq!(before, ids.len());
            for shown in &listed {
                prop_assert!(!shown.bundle_id.is_empty() && !shown.name.trim().is_empty());
                prop_assert!(apps.iter().any(|a| a.bundle_id == shown.bundle_id));
            }
            for usable in apps.iter().filter(|a| !a.bundle_id.is_empty() && !a.name.trim().is_empty()) {
                prop_assert!(listed.iter().any(|a| a.bundle_id == usable.bundle_id));
            }
            let is_meeting = |a: &CaptureApp| MEETING_APPS.contains(&a.bundle_id.as_str());
            let first_other = listed.iter().position(|a| !is_meeting(a)).unwrap_or(listed.len());
            prop_assert!(listed[first_other..].iter().all(|a| !is_meeting(a)));
            // Idempotent: listing the list changes nothing.
            prop_assert_eq!(picker_order(listed.clone()), listed);
        }

        /// A mix has one sample per whole frame of the shortest buffer, each
        /// the mean of every channel, and stays within the inputs' range.
        #[test]
        fn prop_mono_is_the_mean_of_every_channel(
            buffers in proptest::collection::vec(
                (1usize..4, proptest::collection::vec(-1.0f32..=1.0, 0..64)),
                1..4,
            ),
        ) {
            let views: Vec<PcmBuffer<'_>> = buffers
                .iter()
                .map(|(channels, samples)| PcmBuffer { samples, channels: *channels })
                .collect();
            let mut out = Vec::new();
            let frames = append_mono(&views, &mut out);
            let expect = views.iter().map(|b| b.samples.len() / b.channels).min().unwrap();
            prop_assert_eq!(frames, expect);
            prop_assert_eq!(out.len(), expect);
            let total: usize = views.iter().map(|b| b.channels).sum();
            for (frame, got) in out.iter().enumerate() {
                let sum: f32 = views
                    .iter()
                    .map(|b| b.samples[frame * b.channels..(frame + 1) * b.channels].iter().sum::<f32>())
                    .sum();
                prop_assert!((got - sum / total as f32).abs() < 1e-4);
                prop_assert!(got.abs() <= 1.0 + 1e-6);
            }
        }

        /// However time and writes interleave, feeding what is owed leaves
        /// the track within the tolerance of the clock, never ahead of it,
        /// and never owes anything while paused.
        #[test]
        fn prop_filling_what_is_owed_keeps_the_track_on_the_clock(
            steps in proptest::collection::vec((0u64..400, any::<bool>(), 0u64..4_000), 1..60),
        ) {
            let tolerance = (GAP_TOLERANCE.as_secs_f64() * f64::from(SAMPLE_RATE)) as i64 + 1;
            let mut filler = GapFiller::new();
            let mut written: u64 = 0;
            let mut active_ms: u64 = 0;
            for (ms, paused, real) in steps {
                filler.advance(Duration::from_millis(ms), paused);
                if !paused {
                    active_ms += ms;
                    // Whatever real sound arrived is written first.
                    written += real;
                } else {
                    prop_assert_eq!(filler.owed(written), 0);
                }
                let owed = filler.owed(written) as u64;
                written += owed;
                let due = (active_ms as f64 * 48.0) as i64;
                let lag = due - written as i64;
                prop_assert!(lag <= tolerance, "lag {lag}");
                if owed > 0 {
                    // Silence is only written to catch up, never past the clock.
                    prop_assert!(written as i64 <= due + 1);
                }
            }
        }
    }
}
