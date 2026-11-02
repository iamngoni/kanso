//! Reminders: a system notification when a task with a due time (`- [ ]
//! Call Sam @2026-10-25T09:00`) comes due. Clicking it opens the note at
//! the task; Mark Done ticks the task off; Dismiss lets it be. Either way
//! the note records the acknowledgement (`kansoReminderAck`), which syncs,
//! so the other devices don't remind again. What this device has shown
//! is kept locally, so a restart doesn't show it twice.
//!
//! Notifications go through GPUI: the notification centre on macOS (in an
//! app bundle; a bare binary shows none), toast notifications on Windows
//! and the desktop's notification daemon on Linux. Where none can be
//! shown, nothing else changes.

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use std::time::Duration;

use gpui_kit::component::switch::Switch;
use gpui_kit::*;
use kanso_core::dates::DateTime;
use kanso_engine::tasks::reminders::{
    self, ACTIONS, Answer, MAX_WAIT, SHOWN_META, Shown, parse_shown,
};
use kanso_store::{ReminderTask, Store};

use crate::clock::Clock;
use crate::notify_support::{self, Availability};
use crate::settings::{Row, section};
use crate::theme::Colors;
use crate::workspace::AppState;

pub struct Reminders {
    store: Arc<Store>,
    /// What is on screen now, by tag (the task key).
    on_screen: HashMap<String, Shown>,
    /// Shown on this device: key → when (UTC ms).
    shown: BTreeMap<String, i64>,
    open_window: fn(&mut App),
    /// Bumped when the tasks may have changed, to look again at once.
    generation: u64,
    /// Tests: the test platform's notifications, which need no bundle.
    assume_app: bool,
}

impl Global for Reminders {}

/// The notification for a task: its words, and when it is due and in
/// which note.
// no-prop-test: delegates to kanso_engine::tasks::reminders::reminder, tested there.
fn notification_for(task: &ReminderTask, today: kanso_core::dates::Date) -> SystemNotification {
    let said = reminders::reminder(task, today);
    SystemNotification {
        tag: said.tag.into(),
        title: said.title.into(),
        body: said.body.into(),
        actions: ACTIONS
            .iter()
            .map(|(id, label)| SystemNotificationAction {
                id: (*id).into(),
                label: (*label).into(),
            })
            .collect(),
    }
}

/// Whether notifications can reach the screen: never touched outside an
/// app bundle on macOS, and quiet when turned off for Kanso. Tests stand in
/// for the platform with `Reminders::assume_app`.
pub(crate) fn can_notify(cx: &App) -> bool {
    cx.try_global::<Reminders>().is_some_and(|r| r.assume_app)
        || notify_support::availability() == Availability::Ready
}

fn enabled(cx: &App) -> bool {
    cx.try_global::<AppState>()
        .is_none_or(|state| state.config.reminders.enabled)
}

/// Starts reminders: the app's identity for the notification centre, the
/// response handler and the loop that looks for due tasks.
/// `open_window` opens a main window when none is.
pub fn init(store: Arc<Store>, open_window: fn(&mut App), cx: &mut App) {
    cx.set_app_identity(
        crate::flavour::CURRENT.bundle_id,
        crate::flavour::CURRENT.name,
    );
    let shown = parse_shown(store.meta(SHOWN_META).ok().flatten().as_deref());
    cx.set_global(Reminders {
        store,
        on_screen: HashMap::new(),
        shown,
        open_window,
        generation: 0,
        assume_app: false,
    });
    // Registering for responses opens the platform's notification centre,
    // which aborts a macOS process that isn't an app bundle: only an app
    // goes near it (see `notify_support`).
    if notify_support::availability() != Availability::NotAnApp {
        cx.on_system_notification_response(respond);
        notify_support::refresh_permission();
    }
    cx.spawn(async move |cx| {
        loop {
            let wait = cx.update(check);
            let generation = cx.update(|cx| cx.global::<Reminders>().generation);
            // Sleep in short steps so a change to the tasks is seen soon.
            let mut left = wait;
            while !left.is_zero() {
                let step = left.min(Duration::from_secs(2));
                cx.background_executor().timer(step).await;
                left -= step;
                let now = cx.update(|cx| cx.global::<Reminders>().generation);
                if now != generation {
                    break;
                }
            }
        }
    })
    .detach();
}

/// The tasks may have changed: look again soon.
pub fn poke(cx: &mut App) {
    if cx.has_global::<Reminders>() {
        cx.global_mut::<Reminders>().generation += 1;
    }
}

/// Shows the reminders that are due, and says how long until the next
/// look.
// no-prop-test: delegates to kanso_engine::tasks::reminders::{due, record, next_wait}, tested there.
pub fn check(cx: &mut App) -> Duration {
    if !cx.has_global::<Reminders>() || !enabled(cx) || !can_notify(cx) {
        return MAX_WAIT;
    }
    let now_ms = Clock::now_ms(cx);
    let now_local = Clock::local_ms(cx, now_ms);
    let today = DateTime::from_local_ms(now_local).date;
    let store = cx.global::<Reminders>().store.clone();
    let tasks = store.reminder_tasks().unwrap_or_default();
    let notes: Vec<(SystemNotification, Shown)> =
        reminders::due(&tasks, &cx.global::<Reminders>().shown, now_local)
            .into_iter()
            .map(|task| (notification_for(task, today), Shown::new(task, now_ms)))
            .collect();
    if !notes.is_empty() {
        let state = cx.global_mut::<Reminders>();
        let keys: Vec<String> = notes.iter().map(|(n, _)| n.tag.to_string()).collect();
        let record = reminders::record(&mut state.shown, &keys, now_ms);
        for (notification, shown) in &notes {
            state
                .on_screen
                .insert(notification.tag.to_string(), shown.clone());
        }
        store.set_meta(SHOWN_META, &record).ok();
        for (notification, _) in notes {
            cx.show_system_notification(notification);
        }
        // The first one asks for permission; hear the answer.
        notify_support::refresh_permission();
    }
    reminders::next_wait(&tasks, now_local)
}

/// A click on a reminder or one of its buttons.
// no-prop-test: delegates to kanso_engine::tasks::reminders::acknowledge, tested there.
fn respond(response: SystemNotificationResponse, cx: &mut App) {
    // The morning brief's and the weekly review's notifications share this
    // handler.
    if crate::agent_task_panel::owns(&response.tag) {
        crate::agent_task_panel::respond(cx);
        return;
    }
    if crate::briefs::owns(&response.tag) {
        crate::briefs::respond(response, cx);
        return;
    }
    if !cx.has_global::<Reminders>() {
        return;
    }
    let reminders = cx.global_mut::<Reminders>();
    let key = response.tag.to_string();
    let Some(shown) = reminders.on_screen.remove(&key) else {
        return;
    };
    let store = reminders.store.clone();
    let open_window = reminders.open_window;
    let now_ms = Clock::now_ms(cx);
    let answer = Answer::from_action(response.action_id.as_deref());
    reminders::acknowledge(&store, &shown, &key, answer, now_ms);
    if answer == Answer::Open {
        open_task(&shown.note_id, shown.line, open_window, cx);
    }
    cx.dismiss_system_notification(&key);
}

/// Brings a main window forward with the task's note open at its line.
fn open_task(note_id: &str, line: u32, open_window: fn(&mut App), cx: &mut App) {
    if cx.windows().is_empty() {
        open_window(cx);
    }
    cx.activate(true);
    let note_id = note_id.to_owned();
    let mut done = false;
    for handle in cx.windows() {
        if done {
            break;
        }
        handle
            .update(cx, |root, window, cx| {
                let Ok(root) = root.downcast::<gpui_kit::base::Root>() else {
                    return;
                };
                let view = root.read(cx).view().clone();
                let Ok(workspace) = view.downcast::<crate::workspace::Workspace>() else {
                    return;
                };
                workspace.update(cx, |workspace, cx| {
                    workspace.open_note_at(&note_id, line, false, window, cx)
                });
                window.activate_window();
                done = true;
            })
            .ok();
    }
}

/// Settings › General › Reminders.
// no-prop-test: drawing; the settings snapshots show it.
pub fn reminders_section(cx: &App) -> AnyElement {
    let colors = Colors::get(cx);
    let on = enabled(cx);
    section(
        "Reminders",
        vec![
            Row::new(gpui_kit::assets::IconName::Bell, "Remind me of tasks")
                .detail(
                    notify_support::settings_note(
                        notify_support::availability(),
                        crate::flavour::CURRENT.name,
                    )
                    .unwrap_or_else(|| {
                        "A notification when a task with a time comes due, like “- [ ] Call Sam @2026-10-25T09:00”.".to_owned()
                    }),
                )
                .trailing(Switch::new("reminders").checked(on).on_click(|on, _, cx| {
                    let on = *on;
                    AppState::update_config(cx, |config| config.reminders.enabled = on);
                    poke(cx);
                })),
        ],
        colors,
    )
}

#[cfg(test)]
mod tests {
    use super::{Reminders, check, respond};
    use crate::clock::Clock;
    use gpui_kit::{App, SystemNotificationResponse, TestAppContext};
    use kanso_core::DocType;
    use kanso_engine::tasks::reminders::{DISMISS, MARK_DONE};
    use kanso_store::{Origin, Put, Store};
    use serde_json::json;
    use std::collections::{BTreeMap, HashMap};
    use std::sync::Arc;

    // Thursday 22 October 2026, 09:00:30 UTC.
    const NOW: i64 = 1_792_659_630_000;

    fn setup(cx: &mut TestAppContext) -> (tempfile::TempDir, Arc<Store>) {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
        store
            .put(
                DocType::Book,
                json!({"_id": "book:a", "name": "A"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
        store
            .put(
                DocType::Note,
                json!({"_id": "note:a", "title": "Plans", "bookId": "book:a", "body": "- [ ] Call Sam @2026-10-22T09:00\n- [ ] Later @2026-10-22T18:00\n"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
        let reminders_store = store.clone();
        cx.update(|cx: &mut App| {
            cx.set_global(Clock::fixed(NOW, 0));
            cx.set_app_identity("app.kanso.Test", "Kanso Test");
            cx.set_global(Reminders {
                store: reminders_store,
                on_screen: HashMap::new(),
                shown: BTreeMap::new(),
                open_window: |_| {},
                generation: 0,
                assume_app: true,
            });
            cx.on_system_notification_response(respond);
        });
        (dir, store)
    }

    #[gpui_kit::test]
    fn a_due_task_is_shown_once_and_mark_done_ticks_it(cx: &mut TestAppContext) {
        let (_dir, store) = setup(cx);
        let wait = cx.update(check);
        let shown = cx.shown_system_notifications();
        assert_eq!(shown.len(), 1, "only the 09:00 task is due");
        assert_eq!(shown[0].title.as_ref(), "Call Sam");
        assert_eq!(shown[0].body.as_ref(), "Today 09:00 · Plans");
        assert!(wait.as_secs() <= 30);
        // Looking again shows nothing new.
        cx.update(check);
        assert_eq!(cx.shown_system_notifications().len(), 1);
        cx.simulate_system_notification_response(SystemNotificationResponse {
            tag: shown[0].tag.clone(),
            action_id: Some(MARK_DONE.into()),
        });
        cx.run_until_parked();
        let note = store.note("note:a").unwrap().unwrap();
        let body = note.body.clone();
        assert!(body.starts_with("- [x] Call Sam"), "{body}");
        let ack = note.extra.get("kansoReminderAck").cloned().unwrap();
        assert_eq!(
            ack.get(shown[0].tag.as_ref()).and_then(|v| v.as_i64()),
            Some(NOW)
        );
    }

    /// The test binary, like `cargo run`, isn't an app bundle: without the
    /// test stand-in, reminders show nothing and leave the platform's
    /// notification centre alone.
    #[gpui_kit::test]
    fn outside_an_app_bundle_nothing_is_shown(cx: &mut TestAppContext) {
        let (_dir, _store) = setup(cx);
        cx.update(|cx: &mut App| cx.global_mut::<Reminders>().assume_app = false);
        assert!(!crate::notify_support::is_app() || !cfg!(target_os = "macos"));
        if cfg!(target_os = "macos") {
            cx.update(check);
            assert!(cx.shown_system_notifications().is_empty());
        }
    }

    #[gpui_kit::test]
    fn dismiss_acknowledges_and_another_device_stays_quiet(cx: &mut TestAppContext) {
        let (_dir, store) = setup(cx);
        cx.update(check);
        let tag = cx.shown_system_notifications()[0].tag.clone();
        cx.simulate_system_notification_response(SystemNotificationResponse {
            tag: tag.clone(),
            action_id: Some(DISMISS.into()),
        });
        cx.run_until_parked();
        let note = store.note("note:a").unwrap().unwrap();
        assert!(note.body.starts_with("- [ ] Call Sam"));
        // A device that never showed it reads the ack and stays quiet.
        cx.update(|cx: &mut App| cx.global_mut::<Reminders>().shown.clear());
        cx.update(check);
        assert_eq!(cx.shown_system_notifications().len(), 1);
    }
}
