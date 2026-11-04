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
    let mut options = RecorderOptions::new(Channels::Mono, vec![RecordSource::Microphone]);
    options.id = Some("crash-1".into());
    let mut recorder = Recorder::create(root, options).unwrap();
    let tone: Vec<f32> = (0..48_000 * 5)
        .map(|i| 0.4 * (std::f32::consts::TAU * 330.0 * i as f32 / 48_000.0).sin())
        .collect();
    recorder.push(&tone).unwrap();
    std::mem::forget(recorder);
    "crash-1".into()
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
