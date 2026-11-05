//! A real system audio capture on a Mac (issue #38, step 7). It needs the
//! Screen Recording permission for the program that runs the test, and sound
//! playing while it records, so it is ignored by default:
//!
//! ```sh
//! KANSO_LIVE_SECS=6 cargo test -p kanso-audio --test system_live -- --ignored --nocapture
//! ```
//!
//! Play something (`afplay /System/Library/Sounds/Submarine.aiff` in a loop,
//! or any app) for the length of the run. `KANSO_LIVE_APP=<bundle id>`
//! captures one app instead of everything the Mac plays. The test fails if
//! the recording is silent.
#![cfg(target_os = "macos")]

use std::time::Duration;

use kanso_audio::{
    Capture, Channels, RecordSource, RecorderOptions, ScreenAccess, SystemAudioTarget, open_bytes,
};

#[test]
#[ignore = "needs Screen Recording permission and sound playing"]
fn a_real_capture_is_not_silent() {
    assert_eq!(
        kanso_audio::screen_access(),
        ScreenAccess::Allowed,
        "Screen Recording isn't allowed for this program"
    );
    let secs: u64 = std::env::var("KANSO_LIVE_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(6);
    let target = match std::env::var("KANSO_LIVE_APP") {
        Ok(id) => SystemAudioTarget::App(id),
        Err(_) => SystemAudioTarget::Default,
    };
    let dir = tempfile::tempdir().unwrap();
    let mut options = RecorderOptions::new(Channels::Mono, vec![RecordSource::SystemAudio]);
    options.id = Some("live-sys".into());
    let capture = Capture::start_system(dir.path(), options, target).expect("the capture starts");
    std::thread::sleep(Duration::from_secs(secs));
    let status = capture.status();
    let stopped = capture.stop().expect("the capture stops");
    let (path, _) = stopped.finished.parts.first().expect("a part was written");
    let mut source =
        open_bytes(std::fs::read(path).unwrap(), Some("opus")).expect("the part opens");
    let mut samples = Vec::new();
    let mut chunk = Vec::new();
    while source.read(&mut chunk).expect("the part decodes").is_some() {
        samples.extend_from_slice(&chunk);
    }
    let peak = samples.iter().fold(0.0f32, |m, s| m.max(s.abs()));
    let rms = (samples.iter().map(|s| f64::from(*s).powi(2)).sum::<f64>()
        / samples.len().max(1) as f64)
        .sqrt();
    let seconds = samples.len() as f64 / 48_000.0;
    println!(
        "captured {seconds:.2} s, peak {peak:.4}, rms {rms:.5}, meter rms {:.5}",
        status.rms
    );
    assert!(
        (seconds - secs as f64).abs() < 1.0,
        "the track is {seconds:.2} s, not about {secs} s"
    );
    assert!(peak > 0.01, "the capture is silent (peak {peak})");
}
