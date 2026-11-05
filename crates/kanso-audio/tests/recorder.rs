//! The recorder on a real disk: the private folder, the sidecar, fsync of the
//! folder, the cap, the pipeline, and finding and finishing what a crash
//! leaves.

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use kanso_audio::recovery::{recover, scan};
use kanso_audio::session::{
    CHECKPOINT_SAMPLES, SESSION_FILE, in_progress_dir, parse_part_name, part_file_name,
    read_session, remove_session, valid_id,
};
use kanso_audio::{
    Channels, Limits, OpusReader, Pipeline, RecordSource, Recorder, RecorderOptions, SAMPLE_RATE,
    SessionState,
};
use proptest::prelude::*;

fn tone(samples: usize, rate: u32) -> Vec<f32> {
    let mut state = 7u32;
    (0..samples)
        .map(|i| {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let noise = (state >> 8) as f32 / (1u32 << 24) as f32 - 0.5;
            let t = i as f32 / rate as f32;
            0.4 * (std::f32::consts::TAU * 330.0 * t).sin() + 0.02 * noise
        })
        .collect()
}

fn options() -> RecorderOptions {
    RecorderOptions::new(Channels::Mono, vec![RecordSource::Microphone])
}

fn secs(n: usize) -> usize {
    n * SAMPLE_RATE as usize
}

#[cfg(unix)]
fn mode(path: &std::path::Path) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(path).unwrap().permissions().mode() & 0o777
}

#[test]
#[cfg(unix)]
fn the_in_progress_folder_and_the_session_are_private() {
    let data = tempfile::tempdir().unwrap();
    let root = in_progress_dir(data.path());
    let mut recorder = Recorder::create(&root, options()).unwrap();
    recorder.push(&tone(secs(2), SAMPLE_RATE)).unwrap();
    assert_eq!(mode(&root), 0o700);
    assert_eq!(mode(&data.path().join("recordings")), 0o700);
    assert_eq!(mode(recorder.dir()), 0o700);
    assert_eq!(mode(&recorder.dir().join(part_file_name(0))), 0o600);
    assert_eq!(mode(&recorder.dir().join(SESSION_FILE)), 0o600);
}

#[test]
fn the_sidecar_names_the_session_its_parts_and_its_note() {
    let data = tempfile::tempdir().unwrap();
    let root = in_progress_dir(data.path());
    let mut o = options();
    o.note_id = Some("note-1".into());
    let mut recorder = Recorder::create(&root, o).unwrap();
    let first = read_session(recorder.dir()).unwrap();
    assert_eq!(first.state, SessionState::Recording);
    assert_eq!(first.note_id.as_deref(), Some("note-1"));
    assert_eq!(first.sources, vec![RecordSource::Microphone]);
    assert_eq!(first.cap_samples, Limits::default().cap_samples);
    assert!(first.parts.is_empty());

    // Under 5 seconds of audio the sidecar is not rewritten.
    recorder.push(&tone(secs(3), SAMPLE_RATE)).unwrap();
    assert_eq!(read_session(recorder.dir()).unwrap().samples, 0);
    // Past 5 seconds it is, listing the open part.
    recorder.push(&tone(secs(3), SAMPLE_RATE)).unwrap();
    let mid = read_session(recorder.dir()).unwrap();
    assert_eq!(mid.samples, secs(6) as u64);
    assert_eq!(mid.parts.len(), 1);
    assert!(!mid.parts[0].closed);
    assert!(mid.bytes > 0);

    let finished = recorder.finish().unwrap();
    let last = read_session(&finished.dir).unwrap();
    assert_eq!(last, finished.session);
    assert_eq!(last.state, SessionState::Stopped);
    assert_eq!(last.samples, secs(6) as u64);
    assert!(last.parts.iter().all(|p| p.closed));
    assert_eq!(finished.parts.len(), 1);
}

#[test]
fn the_folder_is_synced_each_time_a_part_file_is_created() {
    let data = tempfile::tempdir().unwrap();
    let root = in_progress_dir(data.path());
    let mut o = options();
    o.encode.max_part_frames = 100; // two seconds
    let mut recorder = Recorder::create(&root, o).unwrap();
    assert_eq!(recorder.dir_syncs(), 0);
    recorder.push(&tone(secs(1), SAMPLE_RATE)).unwrap();
    assert_eq!(recorder.dir_syncs(), 1);
    recorder.push(&tone(secs(5), SAMPLE_RATE)).unwrap();
    assert_eq!(recorder.dir_syncs(), 3);
    let finished = recorder.finish().unwrap();
    assert_eq!(finished.parts.len(), 3);
    for (path, _) in &finished.parts {
        assert!(parse_part_name(path.file_name().unwrap().to_str().unwrap()).is_some());
    }
}

#[test]
fn a_part_boundary_rewrites_the_sidecar() {
    let data = tempfile::tempdir().unwrap();
    let root = in_progress_dir(data.path());
    let mut o = options();
    o.encode.max_part_frames = 100;
    let mut recorder = Recorder::create(&root, o).unwrap();
    // 2.5 s: one part closed, a second open; fewer than 5 s in total.
    recorder.push(&tone(secs(5) / 2, SAMPLE_RATE)).unwrap();
    let session = read_session(recorder.dir()).unwrap();
    assert_eq!(session.parts.len(), 2);
    assert!(session.parts[0].closed && !session.parts[1].closed);
}

#[test]
fn the_cap_stops_the_recording_exactly_and_the_warning_comes_once() {
    let data = tempfile::tempdir().unwrap();
    let root = in_progress_dir(data.path());
    let mut o = options();
    o.limits = Limits::from_secs(6, 5);
    let mut recorder = Recorder::create(&root, o).unwrap();
    let one = tone(secs(1), SAMPLE_RATE);
    let mut warnings = 0;
    for second in 1..=5 {
        let p = recorder.push(&one).unwrap();
        warnings += usize::from(p.warned);
        assert_eq!(p.samples, secs(second) as u64);
        assert!(!p.capped);
        assert_eq!(p.warned, second == 5);
    }
    // 2 s offered, 1 s of room.
    let p = recorder.push(&tone(secs(2), SAMPLE_RATE)).unwrap();
    warnings += usize::from(p.warned);
    assert!(p.capped);
    assert_eq!(p.samples, secs(6) as u64);
    let p = recorder.push(&one).unwrap();
    assert!(p.capped && !p.warned);
    assert_eq!(p.samples, secs(6) as u64);
    assert_eq!(warnings, 1);
    let finished = recorder.finish().unwrap();
    assert_eq!(finished.session.samples, secs(6) as u64);
}

#[test]
fn the_default_cap_is_six_hours_with_a_warning_at_five() {
    let l = Limits::default();
    assert_eq!(l.cap_samples, 6 * 3_600 * u64::from(SAMPLE_RATE));
    assert_eq!(l.warn_samples, 5 * 3_600 * u64::from(SAMPLE_RATE));
}

#[test]
fn a_session_id_is_a_folder_name_and_nothing_else() {
    let data = tempfile::tempdir().unwrap();
    let root = in_progress_dir(data.path());
    for bad in ["", "..", "a/b", "a\\b", "x y", &"a".repeat(65)] {
        assert!(!valid_id(bad));
        let mut o = options();
        o.id = Some(bad.into());
        assert!(Recorder::create(&root, o).is_err(), "{bad:?}");
    }
    let mut o = options();
    o.id = Some("abc-123".into());
    Recorder::create(&root, o.clone()).unwrap();
    // A second recorder never takes over a session's folder.
    assert!(Recorder::create(&root, o).is_err());
}

#[test]
fn discard_removes_everything() {
    let data = tempfile::tempdir().unwrap();
    let root = in_progress_dir(data.path());
    let mut recorder = Recorder::create(&root, options()).unwrap();
    recorder.push(&tone(secs(2), SAMPLE_RATE)).unwrap();
    let dir = recorder.dir().to_path_buf();
    recorder.discard().unwrap();
    assert!(!dir.exists());
    assert!(root.exists());
    remove_session(&dir).unwrap(); // already gone is fine
}

#[test]
fn a_finished_recording_plays_as_one() {
    let data = tempfile::tempdir().unwrap();
    let root = in_progress_dir(data.path());
    let mut o = options();
    o.encode.max_part_frames = 150; // three seconds
    let mut recorder = Recorder::create(&root, o).unwrap();
    for piece in tone(secs(8), SAMPLE_RATE).chunks(777) {
        recorder.push(piece).unwrap();
    }
    let finished = recorder.finish().unwrap();
    let mut total = 0;
    for (path, info) in &finished.parts {
        let reader = OpusReader::open(fs::File::open(path).unwrap()).unwrap();
        assert_eq!(reader.info().total_samples, info.samples);
        assert_eq!(fs::metadata(path).unwrap().len(), info.bytes);
        total += info.samples;
    }
    assert_eq!(total, secs(8) as u64);
}

/// Simulates `kill -9`: the recorder is forgotten without finishing, so the
/// page in flight is lost and no end of stream is written.
fn kill(recorder: Recorder) -> PathBuf {
    let dir = recorder.dir().to_path_buf();
    std::mem::forget(recorder);
    dir
}

#[test]
fn a_killed_recording_is_found_and_recovered() {
    let data = tempfile::tempdir().unwrap();
    let root = in_progress_dir(data.path());
    let mut o = options();
    o.id = Some("crashed-1".into());
    o.encode.max_part_frames = 400; // 8 s parts
    let mut recorder = Recorder::create(&root, o).unwrap();
    recorder.push(&tone(secs(21) + 123, SAMPLE_RATE)).unwrap();
    let dir = kill(recorder);

    // The active session is not offered for recovery.
    assert!(scan(&root, Some("crashed-1")).is_empty());
    let found = scan(&root, None);
    assert_eq!(found.len(), 1);
    let r = &found[0];
    assert_eq!(r.id, "crashed-1");
    assert_eq!(r.dir, dir);
    let session = r.session.as_ref().unwrap();
    assert_eq!(session.state, SessionState::Recording);
    // At most the last few seconds are gone: whole pages (1 s) are on disk.
    assert!(r.samples >= secs(20) as u64 - 312 && r.samples <= secs(21) as u64);
    assert_eq!(r.parts.len(), 3);

    // Garbage after the last page (a write the power cut tore) is cut off.
    let last = &r.parts.last().unwrap().0;
    let before = fs::metadata(last).unwrap().len();
    fs::OpenOptions::new()
        .append(true)
        .open(last)
        .unwrap()
        .write_all(b"OggS\0\x02 torn page")
        .unwrap();
    let finished = recover(r).unwrap();
    assert_eq!(fs::metadata(last).unwrap().len(), before);
    assert_eq!(finished.session.state, SessionState::Stopped);
    assert_eq!(finished.session.samples, r.samples);
    assert_eq!(finished.parts.len(), 3);
    for (path, info) in &finished.parts {
        let mut reader = OpusReader::open(fs::File::open(path).unwrap()).unwrap();
        assert_eq!(reader.read_to_end().unwrap().len() as u64, info.samples);
    }
    assert_eq!(read_session(&dir).unwrap(), finished.session);
}

#[test]
fn recovery_without_a_sidecar_and_with_empty_parts() {
    let data = tempfile::tempdir().unwrap();
    let root = in_progress_dir(data.path());
    let mut o = options();
    o.id = Some("nosidecar".into());
    let mut recorder = Recorder::create(&root, o).unwrap();
    recorder.push(&tone(secs(3), SAMPLE_RATE)).unwrap();
    let dir = kill(recorder);
    fs::remove_file(dir.join(SESSION_FILE)).unwrap();
    // A part cut before its headers were whole holds no audio.
    fs::write(dir.join(part_file_name(1)), b"Ogg").unwrap();
    // A folder with nothing playable is not offered.
    fs::create_dir_all(root.join("empty")).unwrap();
    fs::write(root.join("empty").join(part_file_name(0)), b"").unwrap();
    // Stray files and bad names are ignored.
    fs::create_dir_all(root.join("not a session")).unwrap();
    fs::write(root.join("stray.txt"), b"x").unwrap();

    let found = scan(&root, None);
    assert_eq!(found.len(), 1);
    assert!(found[0].session.is_none());
    let finished = recover(&found[0]).unwrap();
    assert_eq!(finished.parts.len(), 1);
    assert!(!dir.join(part_file_name(1)).exists());
    assert_eq!(finished.session.id, "nosidecar");
    assert_eq!(finished.session.channels, 1);
    assert!(finished.session.samples >= secs(2) as u64 - 312);
}

#[test]
fn scanning_a_missing_folder_finds_nothing() {
    let data = tempfile::tempdir().unwrap();
    assert!(scan(&in_progress_dir(data.path()), None).is_empty());
}

#[test]
fn the_pipeline_resamples_pauses_and_keeps_the_clock() {
    let data = tempfile::tempdir().unwrap();
    let root = in_progress_dir(data.path());
    let recorder = Recorder::create(&root, options()).unwrap();
    let mut pipeline = Pipeline::new(recorder, 44_100).unwrap();
    let one_second = tone(44_100, 44_100);
    let p = pipeline.feed(&one_second).unwrap();
    // The resampler holds back a little; the clock is within 50 ms of 1 s.
    assert!(p.samples > secs(1) as u64 - 2_400 && p.samples <= secs(1) as u64);
    assert!(pipeline.rms() > 0.1);
    // Paused: nothing is written and the clock does not move.
    pipeline.set_paused(true).unwrap();
    let paused = pipeline.feed(&one_second).unwrap();
    assert_eq!(paused.samples, p.samples);
    pipeline.set_paused(false).unwrap();
    pipeline.feed(&one_second).unwrap();
    let finished = pipeline.finish().unwrap();
    // Two seconds of sound went in while recording; the pause is not in it.
    assert_eq!(finished.session.samples, secs(2) as u64);
}

#[test]
fn the_pipeline_needs_a_mono_recorder() {
    let data = tempfile::tempdir().unwrap();
    let root = in_progress_dir(data.path());
    let recorder = Recorder::create(
        &root,
        RecorderOptions::new(Channels::Stereo, vec![RecordSource::Microphone]),
    )
    .unwrap();
    assert!(Pipeline::new(recorder, 48_000).is_err());
}

#[test]
fn checkpoints_are_five_seconds_of_audio() {
    assert_eq!(CHECKPOINT_SAMPLES, 5 * u64::from(SAMPLE_RATE));
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 12,
        rng_seed: proptest::test_runner::RngSeed::Fixed(0x006b_616e_736f),
        failure_persistence: Some(Box::new(proptest::test_runner::FileFailurePersistence::Off)),
        ..ProptestConfig::default()
    })]

    /// However the sound is cut up, a stopped recording holds exactly the
    /// samples pushed, up to the cap, and its sidecar agrees.
    #[test]
    fn prop_a_recording_holds_what_was_pushed_up_to_the_cap(
        total in 0usize..200_000,
        chunk in 1usize..9_000,
        cap_ms in 100u64..4_000,
    ) {
        let data = tempfile::tempdir().unwrap();
        let root = in_progress_dir(data.path());
        let mut o = options();
        o.limits = Limits {
            cap_samples: cap_ms * u64::from(SAMPLE_RATE) / 1000,
            warn_samples: 0,
        };
        let cap = o.limits.cap_samples;
        let mut recorder = Recorder::create(&root, o).unwrap();
        for piece in tone(total, SAMPLE_RATE).chunks(chunk) {
            recorder.push(piece).unwrap();
        }
        let expected = (total as u64).min(cap);
        prop_assert_eq!(recorder.samples(), expected);
        let finished = recorder.finish().unwrap();
        prop_assert_eq!(finished.session.samples, expected);
        let from_parts: u64 = finished.parts.iter().map(|(_, p)| p.samples).sum();
        prop_assert_eq!(from_parts, expected);
    }

    /// A crash anywhere loses less than the page in flight plus the
    /// encoder's delay, and recovery then agrees with the scan.
    #[test]
    fn prop_a_crash_loses_less_than_a_page(total in 96_000usize..240_000, chunk in 100usize..7_000) {
        let data = tempfile::tempdir().unwrap();
        let root = in_progress_dir(data.path());
        let mut recorder = Recorder::create(&root, options()).unwrap();
        for piece in tone(total, SAMPLE_RATE).chunks(chunk) {
            recorder.push(piece).unwrap();
        }
        kill(recorder);
        let found = scan(&root, None);
        prop_assert_eq!(found.len(), 1);
        let lost = total as u64 - found[0].samples;
        prop_assert!(lost <= u64::from(SAMPLE_RATE) + 960 + 312, "lost {}", lost);
        let finished = recover(&found[0]).unwrap();
        prop_assert_eq!(finished.session.samples, found[0].samples);
    }

    /// Part names round-trip and nothing else parses.
    #[test]
    fn prop_part_names_round_trip(index in 0u32..100_000, junk in "[a-z0-9.-]{0,12}") {
        prop_assert_eq!(parse_part_name(&part_file_name(index)), Some(index));
        if !junk.starts_with("part-") {
            prop_assert_eq!(parse_part_name(&junk), None);
        }
    }
}

/// What `Capture` falls back on when the disk refuses a write: one folder,
/// found and finished, with a torn page in flight cut off.
#[test]
fn one_session_folder_is_found_and_finished_after_a_refused_write() {
    let data = tempfile::tempdir().unwrap();
    let root = in_progress_dir(data.path());
    let mut o = options();
    o.id = Some("disk-full".into());
    let mut recorder = Recorder::create(&root, o).unwrap();
    recorder.push(&tone(secs(6), SAMPLE_RATE)).unwrap();
    let dir = kill(recorder);
    let part = dir.join(kanso_audio::session::part_file_name(0));
    let whole = fs::metadata(&part).unwrap().len();
    fs::OpenOptions::new()
        .append(true)
        .open(&part)
        .unwrap()
        .write_all(b"OggS\0\x00 half a page")
        .unwrap();

    assert!(kanso_audio::recovery::find(&root.join("missing")).is_none());
    let found = kanso_audio::recovery::find(&dir).unwrap();
    assert_eq!(found.id, "disk-full");
    let finished = recover(&found).unwrap();
    assert_eq!(fs::metadata(&part).unwrap().len(), whole);
    assert!(finished.session.samples >= secs(5) as u64);
    assert_eq!(finished.session.state, SessionState::Stopped);
}

/// A tone at `hz` of `samples` samples at `rate`.
fn pure_tone(samples: usize, rate: u32, hz: f32, level: f32) -> Vec<f32> {
    (0..samples)
        .map(|i| level * (std::f32::consts::TAU * hz * i as f32 / rate as f32).sin())
        .collect()
}

/// Two sources on different rates and clocks become one stereo recording:
/// you on the left, the meeting on the right, as long as the clock says.
#[test]
fn both_sources_are_written_as_left_you_and_right_meeting() {
    use kanso_audio::MixPipeline;
    let data = tempfile::tempdir().unwrap();
    let root = in_progress_dir(data.path());
    let mut o = RecorderOptions::new(
        Channels::Stereo,
        vec![RecordSource::Microphone, RecordSource::SystemAudio],
    );
    o.id = Some("both".into());
    let recorder = Recorder::create(&root, o).unwrap();
    let mut pipeline = MixPipeline::new(recorder, 44_100, 48_000).unwrap();
    // You say something loud for five seconds; the meeting is quiet.
    let you = pure_tone(44_100 * 5, 44_100, 300.0, 0.6);
    let meeting = pure_tone(48_000 * 5, 48_000, 500.0, 0.05);
    for (a, b) in you.chunks(882).zip(meeting.chunks(960)) {
        pipeline.feed_you(a);
        pipeline.feed_meeting(b);
        pipeline.flush().unwrap();
    }
    let finished = pipeline.finish().unwrap();
    assert_eq!(finished.session.channels, 2);
    assert_eq!(finished.session.sources.len(), 2);
    assert_eq!(finished.session.samples, secs(5) as u64);
    let mut reader = OpusReader::open(fs::File::open(&finished.parts[0].0).unwrap()).unwrap();
    assert_eq!(reader.info().channels, Channels::Stereo);
    let pcm = reader.read_to_end().unwrap();
    let (mut left, mut right) = (0.0f64, 0.0f64);
    for &[l, r] in pcm.as_chunks::<2>().0 {
        left += f64::from(l).powi(2);
        right += f64::from(r).powi(2);
    }
    assert!(left > right * 20.0, "{left} vs {right}");
    assert_eq!(
        kanso_audio::speaker_of(left, right),
        Some(kanso_audio::Speaker::You)
    );
}

/// A mono recorder is refused by the stereo pipeline.
#[test]
fn the_stereo_pipeline_wants_a_stereo_recorder() {
    let data = tempfile::tempdir().unwrap();
    let root = in_progress_dir(data.path());
    let recorder = Recorder::create(&root, options()).unwrap();
    assert!(kanso_audio::MixPipeline::new(recorder, 48_000, 48_000).is_err());
}
