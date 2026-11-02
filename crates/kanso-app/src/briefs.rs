//! The morning brief's and the weekly review's notifications: one a day at
//! the time chosen in Settings, through the same path as reminders. Clicking
//! one opens the brief or the review.
//!
//! What is shown, and when, is `kanso_engine::brief` (`due_now`,
//! `next_wake`): a notice fires once a day per device, at most three hours
//! late. That it was shown is kept in the library's local meta
//! (`brief.shown.<date>`), so a restart doesn't show it twice. The text is
//! counts or a fixed sentence, never a note's words. Nothing is acknowledged
//! across devices.
//!
//! Where no notification can reach the screen (a macOS binary that isn't an
//! app bundle, permission denied) nothing is shown and nothing else changes.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use gpui_kit::*;
use kanso_core::dates::DateTime;
use kanso_engine::brief::{self, BriefSettings, NoticeKind, Now};
use kanso_store::Store;

use crate::clock::Clock;
use crate::workspace::AppState;

/// The longest wait between looks: a change in Settings is seen by then.
const MAX_WAIT: Duration = Duration::from_secs(30);
const MIN_WAIT: Duration = Duration::from_millis(500);
/// Tags of these notifications start with this, so the one handler of
/// notification responses can tell them from reminders.
const TAG_PREFIX: &str = "kanso-digest-";

pub struct Briefs {
    store: Arc<Store>,
    /// What is on screen now, by tag.
    on_screen: HashMap<String, NoticeKind>,
    open_window: fn(&mut App),
    /// Bumped when the settings may have changed, to look again at once.
    generation: u64,
    /// Tests: the test platform's notifications, which need no bundle.
    assume_app: bool,
}

impl Global for Briefs {}

/// Whether a notification response is for the brief or the review.
pub fn owns(tag: &str) -> bool {
    tag.starts_with(TAG_PREFIX)
}

fn settings(cx: &App) -> BriefSettings {
    cx.try_global::<AppState>()
        .map(|state| state.config.briefs.clone())
        .unwrap_or_default()
        .sanitized()
}

/// Starts the notifications: the loop that looks for a time to notify. The
/// response handler is reminders', which hands ours over (`owns`).
pub fn init(store: Arc<Store>, open_window: fn(&mut App), cx: &mut App) {
    cx.set_global(Briefs {
        store,
        on_screen: HashMap::new(),
        open_window,
        generation: 0,
        assume_app: false,
    });
    cx.spawn(async move |cx| {
        loop {
            let wait = cx.update(check);
            let generation = cx.update(|cx| cx.global::<Briefs>().generation);
            let mut left = wait;
            while !left.is_zero() {
                let step = left.min(Duration::from_secs(2));
                cx.background_executor().timer(step).await;
                left -= step;
                if cx.update(|cx| cx.global::<Briefs>().generation) != generation {
                    break;
                }
            }
        }
    })
    .detach();
}

/// The settings may have changed: look again soon.
pub fn poke(cx: &mut App) {
    if cx.has_global::<Briefs>() {
        cx.global_mut::<Briefs>().generation += 1;
    }
}

fn can_notify(cx: &App) -> bool {
    cx.try_global::<Briefs>().is_some_and(|b| b.assume_app) || crate::reminders::can_notify(cx)
}

/// What the notification for `kind` says on `now`.
fn body_of(store: &Store, kind: NoticeKind, now: Now) -> String {
    match kind {
        NoticeKind::Brief => brief::morning_brief(store, now, None)
            .map(|b| b.notification_text())
            .unwrap_or_else(|_| kind.fixed_body().to_owned()),
        NoticeKind::Review => kind.fixed_body().to_owned(),
    }
}

/// Shows the notifications that are due, and says how long until the next
/// look.
// no-prop-test: delegates to kanso_engine::brief::{due_now, next_wake}, tested there.
pub fn check(cx: &mut App) -> Duration {
    if !cx.has_global::<Briefs>() || !can_notify(cx) {
        return MAX_WAIT;
    }
    let settings = settings(cx);
    let now_ms = Clock::now_ms(cx);
    let now_local = Clock::local_ms(cx, now_ms);
    let store = cx.global::<Briefs>().store.clone();
    let shown = |kind: NoticeKind, date| {
        store
            .meta(&brief::shown_key(kind, date))
            .ok()
            .flatten()
            .is_some()
    };
    let due = brief::due_now(&settings, now_local, &shown);
    if !due.is_empty() {
        let now = Now {
            utc_ms: now_ms,
            offset_ms: now_local - now_ms,
        };
        let today = DateTime::from_local_ms(now_local).date;
        for kind in due {
            store
                .set_meta(&brief::shown_key(kind, today), &now_ms.to_string())
                .ok();
            let tag = format!("{TAG_PREFIX}{}-{}", kind.as_str(), today.iso());
            let body = body_of(&store, kind, now);
            cx.global_mut::<Briefs>()
                .on_screen
                .insert(tag.clone(), kind);
            cx.show_system_notification(SystemNotification {
                tag: tag.into(),
                title: kind.title().into(),
                body: body.into(),
                actions: Vec::new(),
            });
        }
        crate::notify_support::refresh_permission();
    }
    let cards = flashcard_reminder(&store, now_ms, now_local, cx);
    let wait = match brief::next_wake(&settings, now_local) {
        Some(at) => Duration::from_millis((at - now_local).max(0) as u64).clamp(MIN_WAIT, MAX_WAIT),
        None => MAX_WAIT,
    };
    wait.min(cards)
}

/// Tags of the flashcard reminder start with this (after `TAG_PREFIX`).
const CARDS_TAG: &str = "kanso-digest-flashcards-";

/// The daily flashcard reminder (Settings › Flashcards): once a day, at the
/// time chosen, saying how many cards are waiting. A count, never a
/// question or an answer. Nothing when none are due. Returns how long until
/// the next look.
// no-prop-test: the timing is `kanso_engine::flashcards::reminder_due` and
// `reminder_wait` (property-tested there); this only shows it.
fn flashcard_reminder(store: &Store, now_ms: i64, now_local: i64, cx: &mut App) -> Duration {
    let Some(at) = store
        .card_settings()
        .ok()
        .and_then(|s| s.reminder)
        .and_then(|t| kanso_engine::flashcards::parse_reminder(&t))
    else {
        return MAX_WAIT;
    };
    let date = DateTime::from_local_ms(now_local);
    let today = date.date;
    let now_min = (now_local.rem_euclid(86_400_000) / 60_000) as u32;
    let key = format!("cards.reminder.{}", today.iso());
    let shown = store.meta(&key).ok().flatten().is_some();
    if kanso_engine::flashcards::reminder_due(at, now_min, shown) {
        let offset = ((now_local - now_ms) / 60_000) as i32;
        let day = kanso_engine::flashcards::day_start_ms(now_ms, offset);
        let waiting = store
            .cards_waiting(&kanso_store::CardScope::All, now_ms, day)
            .unwrap_or(0);
        // Marked either way: with nothing due there is nothing to say today.
        store.set_meta(&key, &now_ms.to_string()).ok();
        if let Some(body) = kanso_engine::flashcards::reminder_text(waiting) {
            cx.show_system_notification(SystemNotification {
                tag: format!("{CARDS_TAG}{}", today.iso()).into(),
                title: "Flashcards".into(),
                body: body.into(),
                actions: Vec::new(),
            });
            crate::notify_support::refresh_permission();
        }
    }
    let minutes = kanso_engine::flashcards::reminder_wait(at, now_min).max(1);
    Duration::from_secs(u64::from(minutes) * 60).clamp(MIN_WAIT, MAX_WAIT)
}

/// A click on a brief or review notification: opens it.
// no-prop-test: window plumbing; the click is covered by the test below.
pub fn respond(response: SystemNotificationResponse, cx: &mut App) {
    if !cx.has_global::<Briefs>() {
        return;
    }
    let tag = response.tag.to_string();
    if tag.starts_with(CARDS_TAG) {
        let open_window = cx.global::<Briefs>().open_window;
        if cx.windows().is_empty() {
            open_window(cx);
        }
        cx.activate(true);
        for handle in cx.windows() {
            let opened = handle
                .update(cx, |_, window, cx| {
                    window.dispatch_action(Box::new(crate::flashcards_view::ShowFlashcards), cx)
                })
                .is_ok();
            if opened {
                handle
                    .update(cx, |_, window, _| window.activate_window())
                    .ok();
                break;
            }
        }
        cx.dismiss_system_notification(&tag);
        return;
    }
    let Some(kind) = cx.global_mut::<Briefs>().on_screen.remove(&tag) else {
        return;
    };
    let open_window = cx.global::<Briefs>().open_window;
    if cx.windows().is_empty() {
        open_window(cx);
    }
    cx.activate(true);
    for handle in cx.windows() {
        let opened = handle
            .update(cx, |_, window, cx| match kind {
                NoticeKind::Brief => {
                    window.dispatch_action(Box::new(crate::digest_view::OpenMorningBrief), cx)
                }
                NoticeKind::Review => {
                    window.dispatch_action(Box::new(crate::digest_view::OpenWeeklyReview), cx)
                }
            })
            .is_ok();
        if opened {
            handle
                .update(cx, |_, window, _| window.activate_window())
                .ok();
            break;
        }
    }
    cx.dismiss_system_notification(&tag);
}

#[cfg(test)]
mod tests {
    use super::{Briefs, check};
    use crate::clock::Clock;
    use gpui_kit::{App, TestAppContext};
    use kanso_core::DocType;
    use kanso_store::Store;
    use kanso_store::{Origin, Put};
    use serde_json::json;
    use std::collections::HashMap;
    use std::sync::Arc;

    // Monday 2 November 2026, 08:30:00 UTC.
    const NOW: i64 = 1_793_608_200_000;

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
                json!({"_id": "note:a", "title": "Plans canary", "bookId": "book:a",
                       "body": "- [ ] pay canaryrent @2026-11-01\nI'll call canaryperson about it.\n"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
        let held = store.clone();
        cx.update(|cx: &mut App| {
            cx.set_global(Clock::fixed(NOW, 0));
            cx.set_app_identity("app.kanso.Test", "Kanso Test");
            cx.set_global(Briefs {
                store: held,
                on_screen: HashMap::new(),
                open_window: |_| {},
                generation: 0,
                assume_app: true,
            });
        });
        (dir, store)
    }

    #[gpui_kit::test]
    fn the_brief_is_shown_once_a_day_with_counts_only(cx: &mut TestAppContext) {
        let (_dir, _store) = setup(cx);
        let wait = cx.update(check);
        let shown = cx.shown_system_notifications();
        assert_eq!(shown.len(), 1, "the review isn't on a Monday");
        assert_eq!(shown[0].title.as_ref(), "Your morning brief");
        assert_eq!(shown[0].body.as_ref(), "1 due, 1 open loop");
        for text in [shown[0].title.as_ref(), shown[0].body.as_ref()] {
            assert!(!text.contains("canary"), "{text}");
        }
        assert!(wait.as_secs() <= 30);
        // Looking again, or after a restart, shows nothing more.
        cx.update(check);
        cx.update(|cx: &mut App| cx.global_mut::<Briefs>().on_screen.clear());
        cx.update(check);
        assert_eq!(cx.shown_system_notifications().len(), 1);
    }

    #[gpui_kit::test]
    fn turned_off_or_too_late_shows_nothing(cx: &mut TestAppContext) {
        let (_dir, _store) = setup(cx);
        cx.update(|cx: &mut App| {
            cx.set_global(Clock::fixed(NOW + 4 * 3_600_000, 0));
        });
        cx.update(check);
        assert!(
            cx.shown_system_notifications().is_empty(),
            "12:30 is more than three hours after 08:00"
        );
    }

    #[gpui_kit::test]
    fn the_flashcard_reminder_says_a_count_once_a_day(cx: &mut TestAppContext) {
        let (_dir, store) = setup(cx);
        store
            .put(
                DocType::Note,
                json!({"_id": "note:cards", "title": "Deck", "bookId": "book:a",
                       "body": "secretquestion :: secretanswer\n"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
        let mut settings = store.card_settings().unwrap();
        settings.reminder = Some("08:00".into());
        store.set_card_settings(&settings).unwrap();
        cx.update(check);
        let shown = cx.shown_system_notifications();
        let cards: Vec<_> = shown
            .iter()
            .filter(|n| n.title.as_ref() == "Flashcards")
            .collect();
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].body.as_ref(), "1 flashcard is due.");
        for text in [cards[0].title.as_ref(), cards[0].body.as_ref()] {
            assert!(!text.contains("secret"), "{text}");
        }
        cx.update(check);
        let again = cx
            .shown_system_notifications()
            .iter()
            .filter(|n| n.title.as_ref() == "Flashcards")
            .count();
        assert_eq!(again, 1, "once a day");
    }
}
