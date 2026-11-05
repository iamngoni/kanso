//! Giving each paragraph of a transcript the voice that spoke it, and a
//! name: "Speaker 1", "Speaker 2", in the order they first speak. The
//! person renames them afterwards, in the note.

use std::ops::Range;
use std::time::Duration;

use kanso_audio::Speaker;

use super::model::Diarization;

/// The label before the number: "Speaker 1".
pub const NAME: &str = "Speaker";

/// The name for the `n`th voice to speak, counting from 1. Pure.
pub fn speaker_name(n: usize) -> String {
    format!("{NAME} {n}")
}

/// Which voice spoke most of `span` (seconds), or `None` when nobody was
/// heard in it. The lowest-numbered voice wins a tie. Pure.
pub fn voice_in(diarization: &Diarization, span: &Range<f64>) -> Option<usize> {
    let mut best: Option<(usize, f64)> = None;
    let mut speakers: Vec<usize> = diarization.turns.iter().map(|t| t.speaker).collect();
    speakers.sort_unstable();
    speakers.dedup();
    for speaker in speakers {
        let heard: f64 = diarization
            .turns
            .iter()
            .filter(|t| t.speaker == speaker)
            .map(|t| t.overlap(span))
            .sum();
        if heard > 0. && best.is_none_or(|(_, most)| heard > most) {
            best = Some((speaker, heard));
        }
    }
    best.map(|(speaker, _)| speaker)
}

/// What to call whoever said each paragraph. `starts` are the paragraphs'
/// start times and `starts[i + 1]` is where paragraph `i` ends (the last
/// one ends with the recording). `channels` is what the two tracks of a
/// recording already said: `You` for the microphone, `Meeting` for the
/// system audio, nothing for a recording on one track.
///
/// - The microphone stays "You": the tracks know that better than a voice
///   model does.
/// - Everyone else becomes "Speaker 1", "Speaker 2"... by first appearance,
///   but only when at least two different voices were heard among them. One
///   voice is not worth a label that says nothing: "Meeting" stays
///   "Meeting", and a memo stays unlabelled.
/// - A paragraph where nobody was heard keeps what the tracks said.
///
/// Pure.
pub fn names(
    starts: &[Duration],
    diarization: &Diarization,
    channels: &[Option<Speaker>],
) -> Vec<Option<String>> {
    let kept = |i: usize| channels.get(i).copied().flatten().map(Speaker::label);
    let voices: Vec<Option<usize>> = (0..starts.len())
        .map(|i| {
            if channels.get(i).copied().flatten() == Some(Speaker::You) {
                return None;
            }
            let end = starts
                .get(i + 1)
                .map_or(diarization.seconds, Duration::as_secs_f64);
            let start = starts[i].as_secs_f64();
            voice_in(diarization, &(start..end.max(start)))
        })
        .collect();
    let mut order: Vec<usize> = Vec::new();
    for voice in voices.iter().flatten() {
        if !order.contains(voice) {
            order.push(*voice);
        }
    }
    (0..starts.len())
        .map(|i| match voices[i] {
            Some(voice) if order.len() >= 2 => order
                .iter()
                .position(|v| *v == voice)
                .map(|n| speaker_name(n + 1)),
            _ => kept(i).map(str::to_owned),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::super::model::Turn;
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::{FileFailurePersistence, RngSeed};

    fn secs(s: f64) -> Duration {
        Duration::from_secs_f64(s)
    }

    fn turn(start: f64, end: f64, speaker: usize) -> Turn {
        Turn {
            start,
            end,
            speaker,
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 96,
            rng_seed: RngSeed::Fixed(0x006b_616e_736f),
            failure_persistence: Some(Box::new(FileFailurePersistence::Off)),
            ..ProptestConfig::default()
        })]

        /// The microphone is always "You", and numbering is 1, 2, 3... with
        /// no gaps, in the order the voices first speak, whatever ids the
        /// model gave them.
        #[test]
        fn prop_names_are_dense_by_first_appearance_and_you_stays(
            heard in proptest::collection::vec((any::<bool>(), proptest::option::of(0usize..6)), 1..30),
        ) {
            // One 10-second paragraph each; the voice fills it.
            let starts: Vec<Duration> = (0..heard.len()).map(|i| secs(i as f64 * 10.)).collect();
            let turns = heard
                .iter()
                .enumerate()
                .filter_map(|(i, (_, v))| v.map(|v| turn(i as f64 * 10., i as f64 * 10. + 10., v)))
                .collect();
            let diarization = Diarization { turns, seconds: heard.len() as f64 * 10. };
            let channels: Vec<Option<Speaker>> = heard
                .iter()
                .map(|(you, _)| Some(if *you { Speaker::You } else { Speaker::Meeting }))
                .collect();
            let out = names(&starts, &diarization, &channels);
            prop_assert_eq!(out.len(), heard.len());
            let mut next = 1usize;
            for (i, name) in out.iter().enumerate() {
                if heard[i].0 {
                    prop_assert_eq!(name.as_deref(), Some("You"));
                } else if let Some(name) = name.as_deref().and_then(|n| n.strip_prefix("Speaker ")) {
                    let n: usize = name.parse().unwrap();
                    prop_assert!(n <= next);
                    if n == next { next += 1; }
                }
            }
        }

        /// Renaming the model's ids (any bijection) changes no label.
        #[test]
        fn prop_the_models_ids_do_not_matter(
            voices in proptest::collection::vec(0usize..4, 1..20),
            shift in 1usize..50,
        ) {
            let starts: Vec<Duration> = (0..voices.len()).map(|i| secs(i as f64 * 10.)).collect();
            let make = |map: &dyn Fn(usize) -> usize| Diarization {
                turns: voices
                    .iter()
                    .enumerate()
                    .map(|(i, &v)| turn(i as f64 * 10., i as f64 * 10. + 10., map(v)))
                    .collect(),
                seconds: voices.len() as f64 * 10.,
            };
            let none = vec![None; voices.len()];
            prop_assert_eq!(
                names(&starts, &make(&|v| v), &none),
                names(&starts, &make(&|v| 3 * v + shift), &none)
            );
        }

        /// With fewer than two voices heard, the tracks' labels pass
        /// through untouched.
        #[test]
        fn prop_one_voice_changes_nothing(
            tracks in proptest::collection::vec(proptest::option::of(any::<bool>()), 1..20),
        ) {
            let channels: Vec<Option<Speaker>> = tracks
                .iter()
                .map(|t| t.map(|you| if you { Speaker::You } else { Speaker::Meeting }))
                .collect();
            let starts: Vec<Duration> = (0..channels.len()).map(|i| secs(i as f64 * 10.)).collect();
            let diarization = Diarization {
                turns: vec![turn(0., channels.len() as f64 * 10., 0)],
                seconds: channels.len() as f64 * 10.,
            };
            let out = names(&starts, &diarization, &channels);
            for (name, channel) in out.iter().zip(&channels) {
                prop_assert_eq!(name.as_deref(), channel.map(Speaker::label));
            }
        }
    }

    #[test]
    fn two_voices_on_one_track_are_numbered_by_who_speaks_first() {
        let diarization = Diarization {
            turns: vec![turn(0., 20., 7), turn(20., 40., 2), turn(40., 60., 7)],
            seconds: 60.,
        };
        let starts = [secs(0.), secs(20.), secs(40.)];
        assert_eq!(
            names(&starts, &diarization, &[None, None, None]),
            vec![
                Some("Speaker 1".into()),
                Some("Speaker 2".into()),
                Some("Speaker 1".into())
            ]
        );
    }

    #[test]
    fn the_microphone_stays_you_and_the_meeting_is_split() {
        let diarization = Diarization {
            turns: vec![turn(10., 20., 0), turn(20., 30., 1)],
            seconds: 30.,
        };
        let starts = [secs(0.), secs(10.), secs(20.)];
        let channels = [
            Some(Speaker::You),
            Some(Speaker::Meeting),
            Some(Speaker::Meeting),
        ];
        assert_eq!(
            names(&starts, &diarization, &channels),
            vec![
                Some("You".into()),
                Some("Speaker 1".into()),
                Some("Speaker 2".into())
            ]
        );
    }

    #[test]
    fn silence_keeps_what_the_tracks_said_and_the_most_talking_wins() {
        let diarization = Diarization {
            turns: vec![turn(0., 6., 0), turn(6., 10., 1), turn(20., 30., 1)],
            seconds: 30.,
        };
        assert_eq!(voice_in(&diarization, &(0.0..10.0)), Some(0));
        assert_eq!(voice_in(&diarization, &(10.0..20.0)), None);
        let starts = [secs(0.), secs(10.), secs(20.)];
        let channels = [Some(Speaker::Meeting); 3];
        assert_eq!(
            names(&starts, &diarization, &channels),
            vec![
                Some("Speaker 1".into()),
                Some("Meeting".into()),
                Some("Speaker 2".into())
            ]
        );
    }
}
