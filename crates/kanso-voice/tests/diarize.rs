//! The real speaker models on a recording of two people taking turns.
//! Needs the downloaded files and a recording (16 kHz mono 16-bit WAV is
//! the easiest; anything Kanso reads works):
//!
//! ```sh
//! # Two voices, alternating, made with the system's own:
//! say -v Samantha -o a.aiff "The beta is on track for the sixth. The open question is onboarding."
//! say -v Daniel -o b.aiff "I will send the draft by the thirtieth. Then we review it with design."
//! # ...join a, b, a, b with afconvert/sox into two.wav, then:
//! KANSO_DIARIZE_SEGMENTATION_DIR=/path/to/models/diarize-segmentation-pyannote-3-0 \
//! KANSO_DIARIZE_EMBEDDING_DIR=/path/to/models/diarize-embedding-wespeaker-resnet34 \
//! KANSO_DIARIZE_WAV=/path/to/two.wav \
//! KANSO_DIARIZE_VOICES=2 \
//!   cargo test -p kanso-voice --test diarize -- --ignored --nocapture
//! ```

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::time::Instant;

use kanso_voice::diarize::Diarizer;
use kanso_voice::file::{self, Progress};

fn dir(var: &str) -> PathBuf {
    PathBuf::from(std::env::var(var).unwrap_or_else(|_| panic!("set {var}")))
}

#[test]
#[ignore = "needs the downloaded speaker models and a recording"]
fn two_people_taking_turns_are_two_voices() {
    let wav = dir("KANSO_DIARIZE_WAV");
    let want: usize = std::env::var("KANSO_DIARIZE_VOICES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(2);
    let cancel = AtomicBool::new(false);
    let started = Instant::now();
    let result = file::diarize_parts(
        vec![(
            file::from_path(&wav).unwrap(),
            wav.extension().map(|e| e.to_string_lossy().into_owned()),
        )],
        // A file named `*.opus` is taken for a recording made in Kanso
        // (microphone left, system audio right); anything else is a
        // dropped file, heard as its mix.
        wav.extension().is_some_and(|e| e == "opus"),
        None,
        &dir("KANSO_DIARIZE_SEGMENTATION_DIR"),
        &dir("KANSO_DIARIZE_EMBEDDING_DIR"),
        &cancel,
        &mut |at: Progress| {
            let _ = at;
        },
    )
    .unwrap();
    println!(
        "{} voices over {:.1}s in {:.1}s",
        result.speakers(),
        result.seconds,
        started.elapsed().as_secs_f64()
    );
    for turn in &result.turns {
        println!(
            "  {:6.1} - {:6.1}  voice {}",
            turn.start, turn.end, turn.speaker
        );
    }
    assert_eq!(result.speakers(), want, "{:?}", result.turns);
}

#[test]
#[ignore = "needs the downloaded speaker models"]
fn silence_has_nobody_in_it() {
    let mut diarizer = Diarizer::load(
        &dir("KANSO_DIARIZE_SEGMENTATION_DIR"),
        &dir("KANSO_DIARIZE_EMBEDDING_DIR"),
    )
    .unwrap();
    let cancel = AtomicBool::new(false);
    diarizer.push(&vec![0.0f32; 16_000 * 25], &cancel).unwrap();
    let result = diarizer.finish(&cancel).unwrap();
    assert!(result.turns.is_empty(), "{:?}", result.turns);
    assert!((result.seconds - 25.0).abs() < 0.01);
}
