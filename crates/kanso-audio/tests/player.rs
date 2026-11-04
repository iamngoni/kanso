//! The player on real recordings, with no sound card: parts written by the
//! encoder, decoded by libopus, seeked through the page index, stretched and
//! drained the way the audio callback drains them.

use std::io::{self, Write};
use std::sync::{Arc, Mutex};

use kanso_audio::encode::{DurableWrite, PartWriter};
use kanso_audio::play::{Engine, PartInput};
use kanso_audio::{Channels, EncodeConfig, SAMPLE_RATE};
use proptest::prelude::*;
use proptest::test_runner::{FileFailurePersistence, RngSeed};

#[derive(Clone, Default)]
struct Buf(Arc<Mutex<Vec<u8>>>);

impl Write for Buf {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(data);
        Ok(data.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl DurableWrite for Buf {
    fn sync(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// The frequency of the tone playing in second `k`: 200, 300, 400…
fn hz(second: usize) -> f32 {
    200.0 + 100.0 * second as f32
}

/// A recording of `seconds` seconds in which each second has its own pitch,
/// cut into parts of `part_secs`, as the bytes of each part.
fn recording(seconds: usize, part_secs: usize) -> Vec<PartInput> {
    let bufs = Arc::new(Mutex::new(Vec::<Buf>::new()));
    let sink_bufs = bufs.clone();
    let mut config = EncodeConfig::new(Channels::Mono);
    config.max_part_frames = part_secs as u64 * 50;
    let mut writer = PartWriter::new(
        move |_index: u32| -> io::Result<Buf> {
            let buf = Buf::default();
            sink_bufs.lock().unwrap().push(buf.clone());
            Ok(buf)
        },
        config,
    )
    .unwrap();
    let mut phase = 0.0f32;
    for second in 0..seconds {
        let step = std::f32::consts::TAU * hz(second) / SAMPLE_RATE as f32;
        let pcm: Vec<f32> = (0..SAMPLE_RATE)
            .map(|_| {
                phase += step;
                0.5 * phase.sin()
            })
            .collect();
        writer.push(&pcm).unwrap();
    }
    writer.finish().unwrap();
    let parts = bufs.lock().unwrap();
    parts
        .iter()
        .map(|b| PartInput::new(b.0.lock().unwrap().clone(), Some("opus")))
        .collect()
}

/// Hz of a mono-in-stereo signal, by rising zero crossings.
fn pitch(stereo: &[f32], rate: u32) -> f32 {
    let left: Vec<f32> = stereo.chunks_exact(2).map(|f| f[0]).collect();
    let crossings = left
        .windows(2)
        .filter(|w| w[0] < 0.0 && w[1] >= 0.0)
        .count();
    crossings as f32 * rate as f32 / left.len() as f32
}

/// Plays to the end, returning every frame rendered.
fn play_through(engine: &mut Engine, block: usize) -> Vec<f32> {
    engine.control.play();
    let mut all = Vec::new();
    for _ in 0..100_000 {
        let out = engine.render(block);
        let status = engine.control.status();
        all.extend_from_slice(&out);
        if status.ended {
            return all;
        }
    }
    panic!("never ended");
}

fn seconds(us: u64) -> f64 {
    us as f64 / 1e6
}

#[test]
fn a_recording_in_two_parts_is_one_timeline_that_plays_to_its_end() {
    let parts = recording(5, 3);
    assert_eq!(parts.len(), 2);
    let mut engine = Engine::headless(parts, SAMPLE_RATE).unwrap();
    let length = engine.control.length_us();
    assert!((seconds(length) - 5.0).abs() < 0.03, "{}", seconds(length));
    let all = play_through(&mut engine, 480);
    let played = all.len() as f64 / 2.0 / f64::from(SAMPLE_RATE);
    assert!((played - 5.0).abs() < 0.1, "{played} s came out");
    let status = engine.control.status();
    assert_eq!(status.position_us, length);
    assert!(!status.playing && status.ended);
}

#[test]
fn parts_join_without_a_gap() {
    // One steady tone over two parts: nothing near silence at the seam.
    let parts = {
        let mut one = recording(4, 2);
        // Seconds 0 and 1 are 200 and 300 Hz; only the seam matters here.
        one.truncate(2);
        one
    };
    let mut engine = Engine::headless(parts, SAMPLE_RATE).unwrap();
    let all = play_through(&mut engine, 256);
    let left: Vec<f32> = all.chunks_exact(2).map(|f| f[0]).collect();
    let seam = 2 * SAMPLE_RATE as usize;
    let around = &left[seam - 2_400..seam + 2_400];
    let mut run = 0;
    let mut longest = 0;
    for v in around {
        if v.abs() < 0.002 {
            run += 1;
            longest = longest.max(run);
        } else {
            run = 0;
        }
    }
    // A 300 Hz sine spends about 1 ms around each zero crossing.
    assert!(longest < 96, "{longest} frames of near silence at the seam");
}

#[test]
fn a_seek_plays_the_moment_asked_for() {
    let mut engine = Engine::headless(recording(6, 3), SAMPLE_RATE).unwrap();
    for target in [0usize, 1, 2, 3, 4, 5] {
        engine.control.play();
        engine.control.seek_us(target as u64 * 1_000_000 + 250_000);
        engine.render(480);
        // Skip a little after the seek, then listen for 300 ms.
        let heard = engine.render(14_400);
        let got = pitch(&heard, SAMPLE_RATE);
        assert!(
            (got - hz(target)).abs() < 12.0,
            "at {target}.25 s wanted {} Hz, heard {got} Hz",
            hz(target)
        );
    }
}

#[test]
fn a_resampled_card_keeps_the_clock_and_the_pitch() {
    let mut engine = Engine::headless(recording(4, 3), 44_100).unwrap();
    engine.control.play();
    engine.control.seek_us(1_200_000);
    engine.render(4_410);
    let heard = engine.render(13_230);
    let got = pitch(&heard, 44_100);
    assert!((got - hz(1)).abs() < 12.0, "heard {got} Hz");
    let all = play_through(&mut engine, 441);
    let played = all.len() as f64 / 2.0 / 44_100.0;
    // From 1.2 s and 0.4 s already heard, to the end at 4 s.
    assert!((played - 2.4).abs() < 0.15, "{played}");
}

#[test]
fn speed_shortens_the_wait_but_not_the_recording() {
    for (speed, wanted) in [(2.0f32, 2.5f64), (1.5, 3.333), (0.75, 6.667)] {
        let mut engine = Engine::headless(recording(5, 3), SAMPLE_RATE).unwrap();
        engine.control.set_speed(speed);
        let all = play_through(&mut engine, 480);
        let played = all.len() as f64 / 2.0 / f64::from(SAMPLE_RATE);
        assert!(
            (played - wanted).abs() < 0.25,
            "{speed}x took {played} s, wanted {wanted}"
        );
        // The position still reports recording time.
        assert_eq!(engine.control.position_us(), engine.control.length_us());
    }
}

#[test]
fn speed_keeps_the_pitch_and_the_position_in_recording_time() {
    let mut engine = Engine::headless(recording(6, 3), SAMPLE_RATE).unwrap();
    engine.control.set_speed(1.5);
    engine.control.play();
    engine.control.seek_us(2_000_000);
    engine.render(480);
    let mut heard = Vec::new();
    for _ in 0..30 {
        heard.extend(engine.render(480));
    }
    // 310 ms of listening covered 465 ms of the third second (400 Hz).
    let got = pitch(&heard, SAMPLE_RATE);
    assert!((got - hz(2)).abs() < 20.0, "heard {got} Hz");
    let at = seconds(engine.control.position_us());
    assert!((at - 2.465).abs() < 0.05, "position {at} s");
}

#[test]
fn volume_and_pause() {
    let mut engine = Engine::headless(recording(3, 3), SAMPLE_RATE).unwrap();
    engine.control.play();
    engine.render(4_800);
    let loud = engine.render(4_800);
    assert!(loud.iter().any(|v| v.abs() > 0.1));
    engine.control.set_volume(0.0);
    engine.render(4_800);
    let quiet = engine.render(4_800);
    assert!(quiet.iter().all(|v| v.abs() < 1e-6));
    engine.control.pause();
    let before = engine.control.position_us();
    engine.render(4_800);
    assert_eq!(engine.control.position_us(), before);
}

#[test]
fn a_part_that_is_not_audio_is_refused() {
    let bad = vec![PartInput::new(vec![1, 2, 3, 4, 5, 6, 7, 8], Some("opus"))];
    assert!(Engine::headless(bad, SAMPLE_RATE).is_err());
}

#[derive(Clone, Debug)]
enum Op {
    Play,
    Pause,
    Toggle,
    Seek(u64),
    Skip(i64),
    Speed(usize),
    Render(usize),
}

fn ops() -> impl Strategy<Value = Vec<Op>> {
    proptest::collection::vec(
        prop_oneof![
            Just(Op::Play),
            Just(Op::Pause),
            Just(Op::Toggle),
            (0u64..9_000_000).prop_map(Op::Seek),
            (-40_000_000i64..40_000_000).prop_map(Op::Skip),
            (0usize..5).prop_map(Op::Speed),
            (1usize..6_000).prop_map(Op::Render),
        ],
        1..25,
    )
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 24,
        rng_seed: RngSeed::Fixed(0x006b_616e_736f),
        failure_persistence: Some(Box::new(FileFailurePersistence::Off)),
        ..ProptestConfig::default()
    })]

    /// Whatever is done, in whatever order, the position is never past the
    /// length, and a recording that has ended says so at its end.
    #[test]
    fn prop_position_is_never_past_the_length(ops in ops()) {
        let mut engine = Engine::headless(recording(5, 2), SAMPLE_RATE).unwrap();
        let length = engine.control.length_us();
        let speeds = [0.75f32, 1.0, 1.25, 1.5, 2.0];
        for op in ops {
            match op {
                Op::Play => engine.control.play(),
                Op::Pause => engine.control.pause(),
                Op::Toggle => engine.control.toggle(),
                Op::Seek(at) => engine.control.seek_us(at),
                Op::Skip(by) => engine.control.skip_us(by),
                Op::Speed(i) => engine.control.set_speed(speeds[i]),
                Op::Render(n) => { engine.render(n); }
            }
            let status = engine.control.status();
            prop_assert!(status.position_us <= length, "{} > {length}", status.position_us);
            if status.ended {
                prop_assert_eq!(status.position_us, length);
                prop_assert!(!status.playing);
            }
        }
    }

    /// After a seek the position is at the target at once, and a short
    /// buffer later is still within 20 ms of it (it only moves as far as
    /// the audio played).
    #[test]
    fn prop_seek_then_position_lands_within_20_ms(
        target in 0u64..5_100_000,
        play in any::<bool>(),
        speed in prop::sample::select(vec![0.75f32, 1.0, 1.5, 2.0]),
        block in 1usize..470,
    ) {
        let mut engine = Engine::headless(recording(5, 2), SAMPLE_RATE).unwrap();
        let length = engine.control.length_us();
        engine.control.set_speed(speed);
        if play {
            engine.control.play();
        }
        engine.render(1_000);
        engine.control.seek_us(target);
        let want = target.min(length);
        prop_assert_eq!(engine.control.position_us(), want);
        engine.render(block);
        let got = engine.control.position_us();
        prop_assert!(
            got >= want && got - want <= 20_000,
            "sought {want} us, then {got} us after {block} frames at {speed}x"
        );
    }
}
