//! Recording on a phone (issue #38, step 4): a recording a crash left is
//! offered, recovered into a note with its audio attached exactly once, or
//! discarded; a locked note refuses a recording. The microphone is not
//! opened; the recorder has its own tests in `kanso-audio`.

mod common;

use std::path::Path;

use common::*;
use kanso_audio::session::in_progress_dir;
use kanso_audio::{Channels, RecordSource, Recorder, RecorderOptions};
use kanso_mobile::KansoError;
use proptest::prelude::*;
use proptest::test_runner::{Config, RngAlgorithm, RngSeed};

/// A recording a crash left: `seconds` of sound, never finished.
fn leave(data_dir: &Path, id: &str, seconds: usize, note_id: Option<String>) {
    let mut options = RecorderOptions::new(Channels::Mono, vec![RecordSource::Microphone]);
    options.id = Some(id.into());
    options.note_id = note_id;
    let mut recorder = Recorder::create(&in_progress_dir(data_dir), options).unwrap();
    let tone: Vec<f32> = (0..48_000 * seconds)
        .map(|i| 0.4 * (std::f32::consts::TAU * 330.0 * i as f32 / 48_000.0).sin())
        .collect();
    recorder.push(&tone).unwrap();
    std::mem::forget(recorder);
}

#[test]
fn a_left_behind_recording_is_offered_and_recovered_once() {
    let phone = lone_phone();
    let data = phone.dir.path().to_path_buf();
    assert!(phone.library.recording_found(&data).is_empty());
    leave(&data, "crash-1", 5, None);
    let found = phone.library.recording_found(&data);
    assert_eq!(found.len(), 1);
    assert!(
        found[0].text.starts_with("Kanso closed while recording"),
        "{}",
        found[0].text
    );
    let saved = phone.library.recording_recover(&data, "crash-1").unwrap();
    assert!(saved.created);
    assert_eq!(saved.file_ids, vec!["file:rec-crash-1-1"]);
    let note = phone.library.note(&saved.note_id).unwrap();
    let body = note.body.clone().unwrap();
    assert!(body.contains("inkdrop://file:rec-crash-1-1"), "{body}");
    assert!(note.title.starts_with("Recording — "), "{}", note.title);
    // Nothing is left to offer, and a second recover finds nothing.
    assert!(phone.library.recording_found(&data).is_empty());
    assert!(matches!(
        phone.library.recording_recover(&data, "crash-1"),
        Err(KansoError::NotFound)
    ));
}

#[test]
fn a_recording_goes_back_into_its_note_when_it_is_still_there() {
    let phone = lone_phone();
    let data = phone.dir.path().to_path_buf();
    let id = note(&phone.library, "Memo", "My prose");
    leave(&data, "crash-2", 4, Some(id.clone()));
    let saved = phone.library.recording_recover(&data, "crash-2").unwrap();
    assert!(!saved.created);
    assert_eq!(saved.note_id, id);
    let body = phone.library.note(&id).unwrap().body.unwrap();
    assert!(body.starts_with("My prose\n\n[Audio · "), "{body}");
}

#[test]
fn discarding_deletes_the_folder_and_only_that_folder() {
    let phone = lone_phone();
    let data = phone.dir.path().to_path_buf();
    leave(&data, "crash-3", 3, None);
    leave(&data, "crash-4", 3, None);
    phone
        .library
        .recording_discard_found(&data, "crash-3")
        .unwrap();
    let left: Vec<String> = phone
        .library
        .recording_found(&data)
        .into_iter()
        .map(|f| f.id)
        .collect();
    assert_eq!(left, vec!["crash-4"]);
    assert!(
        phone
            .library
            .recording_discard_found(&data, "../escape")
            .is_err()
    );
}

proptest! {
    #![proptest_config(Config {
        cases: 16,
        rng_seed: RngSeed::Fixed(0x0072_6563_3338),
        rng_algorithm: RngAlgorithm::ChaCha,
        failure_persistence: None,
        ..Config::default()
    })]

    /// No id that is not a plain folder name reaches the file system.
    #[test]
    fn prop_discard_refuses_anything_but_a_session_name(id in "[^a-zA-Z0-9-]{1,12}") {
        let phone = lone_phone();
        let data = phone.dir.path().to_path_buf();
        prop_assert!(phone.library.recording_discard_found(&data, &id).is_err());
    }
}
