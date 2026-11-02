//! Voice in the Quick Capture window through the real view: a spoken text
//! saves with its record in the Inbox, Tidy replaces the words with the tidy
//! version and Undo tidy brings them back, a failed tidy changes nothing, and
//! Tidy isn't offered without AI.

use std::sync::Arc;
use std::time::Duration;

use gpui_kit::{AnyWindowHandle, AppContext as _, Entity, TestAppContext};
use kanso_store::Store;

use crate::ai_feature::testing::{Injected, Recorder};
use crate::capture::CaptureView;
use crate::config::AppConfig;
use crate::services::Services;
use crate::workspace::AppState;

const SAID: &str = "remind me to call the plumber about the leak on tuesday";
const REPLY: &str = r#"{"title": "Plumber", "summary": [], "decisions": [],
 "actions": [{"text": "Call the plumber about the leak", "who": null, "when": "Tuesday",
              "quotes": ["call the plumber about the leak"]},
             {"text": "Invented", "quotes": ["never said"]}],
 "sections": []}"#;

struct Open {
    _dir: tempfile::TempDir,
    store: Arc<Store>,
    window: AnyWindowHandle,
    view: Entity<CaptureView>,
}

fn open(cx: &mut TestAppContext, ai: bool, reply: &str) -> Open {
    // The tidy worker thread wakes the view's task from outside the scheduler.
    cx.executor().allow_parking();
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    let (app_store, data_dir) = (store.clone(), dir.path().to_path_buf());
    let recorder = Recorder::new(reply);
    let maker = recorder.maker();
    let (window, view) = cx.update(|cx| {
        gpui_kit::init(cx);
        let mut config = AppConfig::default();
        config.ai.enabled = ai;
        cx.set_global(AppState {
            store: app_store.clone(),
            data_dir,
            config,
        });
        cx.set_global(Services::default());
        cx.set_global(Injected(maker));
        gpui_kit::open_window(Default::default(), cx, |window, cx| {
            cx.new(|cx| CaptureView::new(app_store, window, cx))
        })
        .unwrap()
    });
    cx.run_until_parked();
    Open {
        _dir: dir,
        store,
        window,
        view,
    }
}

fn with<R>(
    open: &Open,
    cx: &mut TestAppContext,
    f: impl FnOnce(&mut CaptureView, &mut gpui_kit::Window, &mut gpui_kit::Context<CaptureView>) -> R,
) -> R {
    let view = open.view.clone();
    cx.update_window(open.window, |_, window, cx| {
        view.update(cx, |view, cx| f(view, window, cx))
    })
    .unwrap()
}

fn spoken(open: &Open, cx: &mut TestAppContext, text: &str, ms: u64) {
    with(open, cx, |view, window, cx| {
        view.set_text_for_test(text, window, cx);
        view.spoken_for_test(ms);
    });
}

fn settle(open: &Open, cx: &mut TestAppContext) -> &'static str {
    for _ in 0..1000 {
        let state = with(open, cx, |view, _, _| view.tidy_for_test().0);
        if state != "working" {
            return state;
        }
        std::thread::sleep(Duration::from_millis(10));
        cx.run_until_parked();
    }
    panic!("the tidy never settled");
}

// no-prop-test: drives the view; the renderer and the engine have the properties.
#[gpui_kit::test]
fn a_spoken_capture_saves_in_the_inbox_with_its_record(cx: &mut TestAppContext) {
    let open = open(cx, false, REPLY);
    spoken(&open, cx, SAID, 24_000);
    with(&open, cx, |view, window, cx| view.save(window, cx));
    let notes = open.store.notes_in_books(&["book:inbox".into()]).unwrap();
    assert_eq!(notes.len(), 1);
    let note = open.store.note(&notes[0].id).unwrap().unwrap();
    assert_eq!(note.extra["kansoCapture"]["kind"], "voice");
    assert_eq!(note.extra["kansoCapture"]["tidy"], "none");
    assert_eq!(note.extra["kansoCapture"]["duration_ms"], 24_000);
    assert!(note.id.starts_with("note:cap-"));
    assert_eq!(note.body, SAID);
}

// no-prop-test: drives the view; `render_tidy` has the properties.
#[gpui_kit::test]
fn tidy_replaces_the_words_and_undo_brings_them_back(cx: &mut TestAppContext) {
    let open = open(cx, true, REPLY);
    spoken(&open, cx, SAID, 24_000);
    assert!(with(&open, cx, |view, _, cx| view.can_tidy(cx)));
    with(&open, cx, |view, window, cx| view.tidy_now(window, cx));
    assert_eq!(settle(&open, cx), "undo");
    let text = with(&open, cx, |view, _, cx| view.text(cx));
    assert!(text.starts_with("# Plumber"), "{text}");
    assert!(text.contains("- [ ] Call the plumber about the leak — Tuesday"));
    assert!(
        !text.contains("Invented"),
        "an item without a real quote is dropped"
    );
    assert!(text.contains(SAID), "the transcript is kept, folded");
    assert!(!with(&open, cx, |view, _, cx| view.can_tidy(cx)));

    with(&open, cx, |view, window, cx| view.undo_tidy(window, cx));
    assert_eq!(with(&open, cx, |view, _, cx| view.text(cx)), SAID);

    // Tidy again and save: the note is titled by the tidy heading and marked done.
    with(&open, cx, |view, window, cx| view.tidy_now(window, cx));
    assert_eq!(settle(&open, cx), "undo");
    with(&open, cx, |view, window, cx| view.save(window, cx));
    let notes = open.store.notes_in_books(&["book:inbox".into()]).unwrap();
    let note = open.store.note(&notes[0].id).unwrap().unwrap();
    assert_eq!(note.title, "Plumber");
    assert_eq!(note.extra["kansoCapture"]["tidy"], "done");
    assert!(note.body.contains("<details><summary>Transcript"));
}

// no-prop-test: drives the view with one unreadable reply.
#[gpui_kit::test]
fn a_failed_tidy_leaves_the_words_alone_and_says_why(cx: &mut TestAppContext) {
    let open = open(cx, true, "not json at all");
    spoken(&open, cx, SAID, 5_000);
    with(&open, cx, |view, window, cx| view.tidy_now(window, cx));
    assert_eq!(settle(&open, cx), "failed");
    assert_eq!(with(&open, cx, |view, _, cx| view.text(cx)), SAID);
}

// no-prop-test: drives the view; the settings decide.
#[gpui_kit::test]
fn tidy_is_not_offered_without_ai_or_for_typed_text(cx: &mut TestAppContext) {
    let open = open(cx, false, REPLY);
    spoken(&open, cx, SAID, 5_000);
    assert!(!with(&open, cx, |view, _, cx| view.can_tidy(cx)));
    let open = self::open(cx, true, REPLY);
    with(&open, cx, |view, window, cx| {
        view.set_text_for_test(SAID, window, cx)
    });
    assert!(
        !with(&open, cx, |view, _, cx| view.can_tidy(cx)),
        "typed, not spoken"
    );
}
