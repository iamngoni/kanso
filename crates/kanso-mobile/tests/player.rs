//! Playing a recording on a phone (issue #38, step 5): the note's recording
//! is found, its bytes are checked, a timestamp link is read, and the player
//! opens, seeks and stays inside the recording. The speaker is not needed:
//! the engine has its own tests in `kanso-audio`; a phone without one gets a
//! plain sentence from `player_open`.

mod common;

use common::*;
use kanso_audio::session::in_progress_dir;
use kanso_audio::{Channels, RecordSource, Recorder, RecorderOptions};
use kanso_mobile::{
    KansoError, NoteRecording, PaletteDto, PartsState, PreviewLink, PreviewOptions, Renderer,
};
use kanso_store::{Store, SyncDoc};
use proptest::prelude::*;
use proptest::test_runner::{Config, RngAlgorithm, RngSeed};

fn palette() -> PaletteDto {
    let theme = r##"{"name":"t","appearance":"light","ui":{"editor":"#ffffff","text":"#222222","accent":"#2185d0"},"syntax":{"keyword":"#aa0000"},"preview":{}}"##;
    Renderer::theme_tokens(theme.into()).unwrap().palette
}

/// A recording a crash left, recovered into a new note: `seconds` of sound.
fn recorded(phone: &Phone, id: &str, seconds: usize) -> (String, Vec<String>) {
    let data = phone.dir.path().to_path_buf();
    let mut options = RecorderOptions::new(Channels::Mono, vec![RecordSource::Microphone]);
    options.id = Some(id.into());
    let mut recorder = Recorder::create(&in_progress_dir(&data), options).unwrap();
    let tone: Vec<f32> = (0..48_000 * seconds)
        .map(|i| 0.4 * (std::f32::consts::TAU * 330.0 * i as f32 / 48_000.0).sin())
        .collect();
    recorder.push(&tone).unwrap();
    std::mem::forget(recorder);
    let saved = phone.library.recording_recover(&data, id).unwrap();
    (saved.note_id, saved.file_ids)
}

fn expect_recording(found: Option<NoteRecording>) -> NoteRecording {
    found.expect("the note holds a recording")
}

#[test]
fn a_notes_recording_is_found_with_its_length() {
    let phone = lone_phone();
    let (note_id, file_ids) = recorded(&phone, "play-1", 5);
    let found = expect_recording(phone.library.note_recording(&note_id).unwrap());
    assert_eq!(found.file_ids, file_ids);
    assert_eq!(found.state, PartsState::Ready);
    let ms = found.length_ms.expect("the length is read");
    assert!((4_900..=5_100).contains(&ms), "{ms} ms");
}

#[test]
fn a_note_without_audio_holds_no_recording() {
    let phone = lone_phone();
    let id = note(
        &phone.library,
        "Words",
        "Just words.\n\n[a link](https://example.com)\n",
    );
    assert_eq!(phone.library.note_recording(&id).unwrap(), None);
    assert!(matches!(
        phone.library.note_recording("note:nope"),
        Err(KansoError::NotFound)
    ));
}

#[test]
fn bytes_that_have_not_arrived_are_not_here_and_a_gone_file_is_missing() {
    let phone = lone_phone();
    // The account told this device about a file; its bytes are still coming.
    Store::open(phone.path())
        .unwrap()
        .apply_remote(
            &SyncDoc {
                id: "file:later-1".into(),
                rev: "1-aa".into(),
                deleted: false,
                fields: serde_json::json!({
                    "name": "Recording part 1.opus", "contentType": "audio/ogg",
                    "md5digest": "00000000000000000000000000000000", "createdAt": 1,
                })
                .as_object()
                .unwrap()
                .clone(),
                base_rev: None,
            },
            None,
            None,
        )
        .unwrap();
    let later = note(
        &phone.library,
        "Later",
        "[Audio · 3 min](inkdrop://file:later-1)\n",
    );
    let found = expect_recording(phone.library.note_recording(&later).unwrap());
    assert_eq!(found.state, PartsState::NotHere);
    assert_eq!(found.length_ms, None);
    assert!(matches!(
        phone.library.player_open(vec!["file:later-1".into()]),
        Err(KansoError::Invalid { .. })
    ));
    let gone = note(
        &phone.library,
        "Gone",
        "[Audio · 3 min](inkdrop://file:ghost-1)\n",
    );
    let found = expect_recording(phone.library.note_recording(&gone).unwrap());
    assert_eq!(found.state, PartsState::Missing);
    assert!(matches!(
        phone.library.player_open(vec!["file:ghost-1".into()]),
        Err(KansoError::NotFound)
    ));
}

#[test]
fn a_timestamp_link_names_its_recording_and_moment() {
    let phone = lone_phone();
    let (note_id, file_ids) = recorded(&phone, "play-2", 3);
    let r = phone.library.renderer();
    let id = &file_ids[0];
    let url = format!("kanso-audio://{id}?t=12");
    assert_eq!(
        r.link_target(&note_id, &url).unwrap(),
        PreviewLink::Audio {
            file_id: id.clone(),
            seconds: 12
        }
    );
    assert_eq!(
        r.link_target(&note_id, &format!("kanso-audio://{id}"))
            .unwrap(),
        PreviewLink::Audio {
            file_id: id.clone(),
            seconds: 0
        }
    );
    // Not a moment: nothing happens, and the system is never asked.
    for bad in [
        "kanso-audio://file:?t=1",
        "kanso-audio://note:abc?t=1",
        "kanso-audio://file:abc?t=-1",
        "kanso-audio://file:abc?t=x",
    ] {
        assert_eq!(
            r.link_target(&note_id, bad).unwrap(),
            PreviewLink::Nothing,
            "{bad}"
        );
    }
}

#[test]
fn the_timestamps_of_the_note_become_links_in_its_preview() {
    let phone = lone_phone();
    let (_, file_ids) = recorded(&phone, "play-3", 3);
    let id = &file_ids[0];
    let body =
        format!("[Audio · 3 sec](inkdrop://{id})\n\n[00:02](kanso-audio://{id}?t=2) We started.\n");
    let note_id = note(&phone.library, "Timed", &body);
    let blocks = phone
        .library
        .renderer()
        .preview_blocks(&note_id, palette(), PreviewOptions::default())
        .unwrap();
    let urls = format!("{blocks:?}");
    assert!(urls.contains(&format!("kanso-audio://{id}?t=2")), "{urls}");
}

#[test]
fn the_player_opens_paused_and_stays_inside_the_recording() {
    let phone = lone_phone();
    let (_, file_ids) = recorded(&phone, "play-4", 4);
    let player = match phone.library.player_open(file_ids) {
        Ok(player) => player,
        // A machine with no speaker says so in words.
        Err(KansoError::Invalid { message }) => {
            assert!(message.contains("play"), "{message}");
            return;
        }
        Err(other) => panic!("{other:?}"),
    };
    player.set_volume(0.0);
    let status = player.status();
    assert!(!status.playing);
    assert_eq!(status.position_ms, 0);
    assert!(
        (3_900..=4_100).contains(&status.length_ms),
        "{}",
        status.length_ms
    );
    player.seek_ms(2_500);
    assert_eq!(player.status().position_ms, 2_500);
    player.seek_ms(99_000_000);
    assert_eq!(player.status().position_ms, player.status().length_ms);
    player.skip_ms(-1_000);
    assert_eq!(
        player.status().position_ms,
        player.status().length_ms - 1_000
    );
    player.set_speed(1.5);
    assert_eq!(player.status().speed, 1.5);
}

proptest! {
    #![proptest_config(Config {
        cases: 12,
        rng_algorithm: RngAlgorithm::ChaCha,
        rng_seed: RngSeed::Fixed(0x006b_616e_736f),
        failure_persistence: None,
        ..Config::default()
    })]

    /// However it is moved about, the player's position is never past the
    /// length, and seeking lands exactly where asked (inside the recording).
    #[test]
    fn prop_the_position_stays_inside(
        seeks in proptest::collection::vec(0u64..9_000, 1..8),
        skips in proptest::collection::vec(-6_000i64..6_000, 1..8),
    ) {
        let phone = lone_phone();
        let (_, file_ids) = recorded(&phone, "play-5", 3);
        let Ok(player) = phone.library.player_open(file_ids) else {
            return Ok(());
        };
        player.set_volume(0.0);
        let length = player.status().length_ms;
        for (at, by) in seeks.iter().zip(skips.iter().cycle()) {
            player.seek_ms(*at);
            prop_assert_eq!(player.status().position_ms, (*at).min(length));
            player.skip_ms(*by);
            prop_assert!(player.status().position_ms <= length);
        }
    }
}
