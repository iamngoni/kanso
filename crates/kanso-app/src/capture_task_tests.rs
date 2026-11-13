//! The Note | Task toggle in the Quick Capture window, through the real
//! view: a task saves into today's daily note or the Inbox as Settings
//! says, and the last choice of the toggle is there the next time.

use std::sync::Arc;

use gpui_kit::{AnyWindowHandle, AppContext as _, Entity, TestAppContext};
use kanso_engine::capture_task::{INBOX_TASKS_ID, TaskTarget};
use kanso_store::Store;

use crate::capture::CaptureView;
use crate::clock::Clock;
use crate::config::AppConfig;
use crate::workspace::AppState;

/// Thursday 12 November 2026, 10:30 UTC.
const NOW_MS: i64 = 1_794_479_400_000;

struct Open {
    dir: tempfile::TempDir,
    store: Arc<Store>,
    window: AnyWindowHandle,
    view: Entity<CaptureView>,
}

fn open_with(cx: &mut TestAppContext, config: AppConfig) -> Open {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    let (app_store, data_dir) = (store.clone(), dir.path().to_path_buf());
    let (window, view) = cx.update(|cx| {
        gpui_kit::init(cx);
        cx.set_global(Clock::fixed(NOW_MS, 0));
        cx.set_global(AppState {
            store: app_store.clone(),
            data_dir,
            config,
        });
        gpui_kit::open_window(Default::default(), cx, |window, cx| {
            cx.new(|cx| CaptureView::new(app_store, window, cx))
        })
        .unwrap()
    });
    cx.run_until_parked();
    Open {
        dir,
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

fn capture_task(open: &Open, cx: &mut TestAppContext, text: &str) {
    with(open, cx, |view, window, cx| {
        view.set_task_mode(true, cx);
        view.set_text_for_test(text, window, cx);
        view.save(window, cx);
    });
}

// no-prop-test: drives the view; the engine's property tests cover the writing.
#[gpui_kit::test]
fn a_task_saves_into_todays_daily_note_by_default(cx: &mut TestAppContext) {
    let open = open_with(cx, AppConfig::default());
    capture_task(&open, cx, "film Pixel review fri 3pm every week");
    let dates = open.store.daily_dates().unwrap();
    assert_eq!(dates.len(), 1);
    assert_eq!(dates[0].iso(), "2026-11-12");
    let body = open
        .store
        .note(&kanso_core::roles::daily_id(dates[0]))
        .unwrap()
        .expect("today's daily note")
        .body;
    assert_eq!(
        body,
        "- [ ] film Pixel review @2026-11-13T15:00 every week\n"
    );
}

// no-prop-test: drives the view; the engine's property tests cover the writing.
#[gpui_kit::test]
fn the_inbox_setting_sends_a_task_to_the_inbox(cx: &mut TestAppContext) {
    let mut config = AppConfig::default();
    config.capture.task_target = TaskTarget::Inbox;
    let open = open_with(cx, config);
    capture_task(&open, cx, "buy milk tomorrow");
    assert!(open.store.daily_dates().unwrap().is_empty());
    let note = open.store.note(INBOX_TASKS_ID).unwrap().unwrap();
    assert_eq!(note.book_id, "book:inbox");
    assert_eq!(note.body, "- [ ] buy milk @2026-11-13\n");
}

// no-prop-test: two fixed choices through the toggle.
#[gpui_kit::test]
fn the_toggle_remembers_the_last_choice(cx: &mut TestAppContext) {
    let open = open_with(cx, AppConfig::default());
    assert!(!with(&open, cx, |view, _, _| view.is_task_mode()));
    with(&open, cx, |view, _, cx| view.set_task_mode(true, cx));
    assert!(cx.update(|cx| cx.global::<AppState>().config.capture.task_mode));
    // Saved to disk, and a window opened later starts in Task mode.
    let saved =
        AppConfig::load(&crate::paths::config_path(open.dir.path())).expect("the saved settings");
    assert!(saved.capture.task_mode);
    let next = open_with(cx, saved);
    assert!(with(&next, cx, |view, _, _| view.is_task_mode()));
    with(&next, cx, |view, _, cx| view.set_task_mode(false, cx));
    assert!(!cx.update(|cx| cx.global::<AppState>().config.capture.task_mode));
}
