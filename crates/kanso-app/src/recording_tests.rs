//! Recording in the app, headless: the notice a crash leaves, Recover and
//! Discard, and the transcription a quit cut off. The microphone itself is
//! not opened here; the recorder has its own tests in `kanso-audio`.

use std::path::PathBuf;
use std::sync::Arc;

use gpui_kit::component::WindowExt as _;
use gpui_kit::{AppContext as _, TestAppContext};
use kanso_audio::session::in_progress_dir;
use kanso_audio::{Channels, RecordSource, Recorder, RecorderOptions};
use kanso_store::Store;

use crate::config::AppConfig;
use crate::recording;
use crate::recording_model::Notice;
use crate::services::Services;
use crate::workspace::{AppState, Workspace};

struct Fixture {
    dir: tempfile::TempDir,
    store: Arc<Store>,
    window: gpui_kit::AnyWindowHandle,
}

fn setup(cx: &mut TestAppContext) -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    let data_dir = dir.path().to_path_buf();
    let app_store = store.clone();
    let window = cx.update(|cx| {
        gpui_kit::init(cx);
        cx.set_global(AppState {
            store: app_store,
            data_dir,
            config: AppConfig::default(),
        });
        cx.set_global(Services::default());
        gpui_kit::open_window(Default::default(), cx, |window, cx| {
            cx.new(|cx| Workspace::new(window, cx))
        })
        .unwrap()
        .0
    });
    cx.run_until_parked();
    Fixture { dir, store, window }
}

/// A recording a crash left: five seconds of sound, never finished.
fn leave_a_crash(root: &std::path::Path) -> String {
    leave_a_session(root, "crash-1", RecordSource::Microphone)
}

/// The same for session `id`, from `source`.
fn leave_a_session(root: &std::path::Path, id: &str, source: RecordSource) -> String {
    let mut options = RecorderOptions::new(Channels::Mono, vec![source]);
    options.id = Some(id.into());
    let mut recorder = Recorder::create(root, options).unwrap();
    let tone: Vec<f32> = (0..48_000 * 5)
        .map(|i| 0.4 * (std::f32::consts::TAU * 330.0 * i as f32 / 48_000.0).sin())
        .collect();
    recorder.push(&tone).unwrap();
    std::mem::forget(recorder);
    id.into()
}

fn notice(cx: &mut TestAppContext) -> Option<Notice> {
    cx.read(recording::notice)
}

#[gpui_kit::test]
fn nothing_shows_when_nothing_is_recording(cx: &mut TestAppContext) {
    let _f = setup(cx);
    cx.update(recording::scan);
    assert_eq!(notice(cx), None);
    assert!(!cx.read(recording::busy));
}

#[gpui_kit::test]
fn a_crash_is_offered_for_recovery(cx: &mut TestAppContext) {
    let f = setup(cx);
    let root: PathBuf = in_progress_dir(f.dir.path());
    let id = leave_a_crash(&root);
    cx.update(recording::scan);
    match notice(cx) {
        Some(Notice::Crash(banner)) => {
            assert_eq!(banner.id, id);
            assert!(
                banner.text.starts_with("Kanso closed while recording at "),
                "{}",
                banner.text
            );
            assert!(banner.text.ends_with("sec)."), "{}", banner.text);
        }
        other => panic!("expected the crash banner, got {other:?}"),
    }
}

#[gpui_kit::test]
fn recover_attaches_the_audio_to_a_new_note_and_clears_the_banner(cx: &mut TestAppContext) {
    let f = setup(cx);
    let root = in_progress_dir(f.dir.path());
    let id = leave_a_crash(&root);
    let found = kanso_audio::recovery::scan(&root, None).remove(0);
    let done = recording::recover_now(&f.store, &found).unwrap();
    let note_id = format!("note:rec-{id}");
    assert_eq!(done.saved.note_id, note_id);
    let note = f.store.note(&note_id).unwrap().unwrap();
    assert!(note.title.starts_with("Recording — "), "{}", note.title);
    assert!(note.body.contains("[Audio · "), "{}", note.body);
    assert!(f.store.get(&format!("file:rec-{id}-1")).unwrap().is_some());
    // The folder is gone: the bytes live in the library now.
    assert!(!root.join(&id).exists());
    // Nothing is left to recover.
    cx.update(recording::scan);
    assert_eq!(notice(cx), None);
}

#[gpui_kit::test]
fn a_crash_with_system_audio_is_one_recording_with_two_lines(cx: &mut TestAppContext) {
    let f = setup(cx);
    let root = in_progress_dir(f.dir.path());
    leave_a_session(&root, "call-1", RecordSource::Microphone);
    leave_a_session(&root, "call-1-sys", RecordSource::SystemAudio);
    // One banner for the pair, under the microphone's name.
    cx.update(recording::scan);
    match notice(cx) {
        Some(Notice::Crash(banner)) => assert_eq!(banner.id, "call-1"),
        other => panic!("expected the crash banner, got {other:?}"),
    }
    let mut found = kanso_audio::recovery::scan(&root, None);
    let track = found.remove(found.iter().position(|r| r.id == "call-1-sys").unwrap());
    let mic = found.remove(0);
    let done = recording::recover_with(&f.store, &mic, Some(&track)).unwrap();
    let note = f.store.note(&done.saved.note_id).unwrap().unwrap();
    assert!(note.title.starts_with("Meeting — "), "{}", note.title);
    let lines = kanso_engine::recording_player::link_lines(&note.body);
    assert_eq!(lines.len(), 2, "{}", note.body);
    assert_eq!(done.system_file_ids, vec!["file:rec-call-1-sys-1"]);
    assert!(note.body.contains("[System audio · "), "{}", note.body);
    // Both folders went with it; nothing is left to offer.
    assert!(!root.join("call-1").exists() && !root.join("call-1-sys").exists());
    cx.update(recording::scan);
    assert_eq!(notice(cx), None);
}

#[gpui_kit::test]
fn a_mixed_recording_a_crash_left_is_one_meeting_with_both_voices(cx: &mut TestAppContext) {
    let f = setup(cx);
    let root = in_progress_dir(f.dir.path());
    let mut options = RecorderOptions::new(
        Channels::Stereo,
        vec![RecordSource::Microphone, RecordSource::SystemAudio],
    );
    options.id = Some("call-9".into());
    let recorder = Recorder::create(&root, options).unwrap();
    let mut pipeline = kanso_audio::MixPipeline::new(recorder, 48_000, 48_000).unwrap();
    let tone = |hz: f32| -> Vec<f32> {
        (0..48_000 * 5)
            .map(|i| 0.4 * (std::f32::consts::TAU * hz * i as f32 / 48_000.0).sin())
            .collect()
    };
    pipeline.feed_you(&tone(300.0));
    pipeline.feed_meeting(&tone(500.0));
    pipeline.flush().unwrap();
    std::mem::forget(pipeline);
    // One banner, and Recover makes one meeting note with one audio line.
    cx.update(recording::scan);
    match notice(cx) {
        Some(Notice::Crash(banner)) => assert_eq!(banner.id, "call-9"),
        other => panic!("expected the crash banner, got {other:?}"),
    }
    let found = kanso_audio::recovery::scan(&root, None).remove(0);
    let done = recording::recover_now(&f.store, &found).unwrap();
    let note = f.store.note(&done.saved.note_id).unwrap().unwrap();
    assert!(note.title.starts_with("Meeting — "), "{}", note.title);
    assert_eq!(
        kanso_engine::recording_player::link_lines(&note.body).len(),
        1,
        "{}",
        note.body
    );
    // The stereo recording itself is what is transcribed, so your voice
    // (the left channel) is in the transcript with the meeting's.
    assert!(done.system_file_ids.is_empty());
    assert_eq!(done.saved.file_ids, vec!["file:rec-call-9-1"]);
    assert!(!root.join("call-9").exists());
}

#[gpui_kit::test]
fn a_system_track_left_alone_joins_its_recordings_note(cx: &mut TestAppContext) {
    let f = setup(cx);
    let root = in_progress_dir(f.dir.path());
    leave_a_session(&root, "call-2", RecordSource::Microphone);
    let mic = kanso_audio::recovery::scan(&root, None).remove(0);
    let done = recording::recover_now(&f.store, &mic).unwrap();
    // The track's folder turns up later (or was recovered separately).
    leave_a_session(&root, "call-2-sys", RecordSource::SystemAudio);
    let track = kanso_audio::recovery::scan(&root, None).remove(0);
    let again = recording::recover_now(&f.store, &track).unwrap();
    assert_eq!(again.saved.note_id, done.saved.note_id);
    assert!(!again.saved.created);
    let body = f.store.note(&done.saved.note_id).unwrap().unwrap().body;
    assert!(body.contains("[System audio · "), "{body}");
    assert_eq!(kanso_engine::recording_player::link_lines(&body).len(), 2);
    // A track with no recording to join is a note of its own.
    leave_a_session(&root, "call-3-sys", RecordSource::SystemAudio);
    let alone = kanso_audio::recovery::scan(&root, None).remove(0);
    let made = recording::recover_now(&f.store, &alone).unwrap();
    let note = f.store.note(&made.saved.note_id).unwrap().unwrap();
    assert!(note.title.starts_with("Meeting — "), "{}", note.title);
}

#[gpui_kit::test]
fn a_transcription_a_quit_cut_off_is_offered_again(cx: &mut TestAppContext) {
    let f = setup(cx);
    let dir = f.dir.path().join("recordings");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("pending-transcript.json"),
        br#"{"note_id":"note:rec-a","file_ids":["file:rec-a-1"],"duration_secs":90}"#,
    )
    .unwrap();
    cx.update(recording::scan);
    assert!(matches!(notice(cx), Some(Notice::Pending(p)) if p.note_id == "note:rec-a"));
    cx.update(recording::clear_pending);
    assert_eq!(notice(cx), None);
    assert!(!dir.join("pending-transcript.json").exists());
}

/// Clicks what `selector` names in the main window, as a person would: the
/// handler runs inside the window's own update.
fn click(f: &Fixture, cx: &mut TestAppContext, selector: &'static str) {
    f.window
        .update(cx, |_, window, cx| {
            window.refresh();
            window.draw(cx).clear(cx);
        })
        .unwrap();
    cx.run_until_parked();
    let mut vcx = gpui_kit::VisualTestContext::from_window(f.window, cx);
    let bounds = vcx
        .debug_bounds(selector)
        .unwrap_or_else(|| panic!("{selector} is drawn"));
    vcx.simulate_click(bounds.center(), gpui_kit::Modifiers::none());
    cx.run_until_parked();
}

fn open_note(cx: &mut TestAppContext) -> Option<String> {
    cx.update(|cx| {
        let (_, workspace) = crate::workspace::main_workspace(cx)?;
        let editor = workspace.read(cx).editor.clone();
        editor.read(cx).note_id().map(str::to_owned)
    })
}

#[gpui_kit::test]
fn finish_transcribing_from_the_main_window_opens_the_note(cx: &mut TestAppContext) {
    let f = setup(cx);
    let book = f
        .store
        .role_book(kanso_core::roles::Role::Inbox, kanso_store::Origin::Ui)
        .unwrap();
    let made = f
        .store
        .put(
            kanso_core::DocType::Note,
            serde_json::json!({"title": "Memo", "body": "x", "bookId": book}),
            kanso_store::Put::Upsert,
            kanso_store::Origin::Ui,
        )
        .unwrap();
    let dir = f.dir.path().join("recordings");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("pending-transcript.json"),
        serde_json::to_vec(&serde_json::json!({
            "note_id": made.id, "file_ids": ["file:rec-a-1"], "duration_secs": 90
        }))
        .unwrap(),
    )
    .unwrap();
    cx.update(recording::scan);
    click(&f, cx, "recording-finish-transcribing");
    assert_eq!(open_note(cx).as_deref(), Some(made.id.as_str()));
}

#[gpui_kit::test]
fn the_quit_question_opens_when_quit_comes_from_the_main_window(cx: &mut TestAppContext) {
    use std::sync::atomic::{AtomicBool, Ordering};
    static QUIT: AtomicBool = AtomicBool::new(false);
    let f = setup(cx);
    // As ⌘Q: the action runs inside the key window's update.
    f.window
        .update(cx, |_, _, cx| {
            recording::ask_before_quit(|_| QUIT.store(true, Ordering::SeqCst), cx)
        })
        .unwrap();
    cx.run_until_parked();
    assert!(!QUIT.load(Ordering::SeqCst), "quit without asking");
    assert!(
        f.window
            .update(cx, |_, window, cx| window.has_active_dialog(cx))
            .unwrap()
    );
}

#[gpui_kit::test]
fn a_system_track_left_for_a_locked_note_is_recovered_into_a_note_of_its_own(
    cx: &mut TestAppContext,
) {
    let f = setup(cx);
    let root = in_progress_dir(f.dir.path());
    // The recording was for a note that is locked by the time of Recover.
    let book = f
        .store
        .role_book(kanso_core::roles::Role::Inbox, kanso_store::Origin::Ui)
        .unwrap();
    let made = f
        .store
        .put(
            kanso_core::DocType::Note,
            serde_json::json!({"title": "Locked", "body": "secret", "bookId": book}),
            kanso_store::Put::Upsert,
            kanso_store::Origin::Ui,
        )
        .unwrap();
    let locked = f
        .store
        .lock_note(serde_json::json!({
            "_id": made.id,
            "_rev": made.rev,
            "title": "Locked",
            "body": "",
            "bookId": book,
            (kanso_core::model::LOCK_FIELD): {"v": 1},
        }))
        .unwrap();
    assert!(f.store.note(&locked.id).unwrap().unwrap().is_locked());
    let mut options = RecorderOptions::new(Channels::Mono, vec![RecordSource::SystemAudio]);
    options.id = Some("call-4-sys".into());
    options.note_id = Some(locked.id.clone());
    let mut recorder = Recorder::create(&root, options).unwrap();
    recorder.push(&vec![0.2f32; 48_000]).unwrap();
    std::mem::forget(recorder);
    let track = kanso_audio::recovery::scan(&root, None).remove(0);
    // Not an error that the banner would offer again and again.
    let done = recording::recover_now(&f.store, &track).unwrap();
    assert_ne!(done.saved.note_id, locked.id);
    assert_eq!(f.store.note(&locked.id).unwrap().unwrap().body, "");
    assert!(!root.join("call-4-sys").exists());
}
