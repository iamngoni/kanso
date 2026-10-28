//! Reminders without a window: a notification when a task with a due time
//! (`- [ ] Call Sam @2026-10-25T09:00`) comes due. Clicking it opens the note
//! at the task; Mark Done ticks the task off; Dismiss lets it be. Either way
//! the note records the acknowledgement (`kansoReminderAck`), which syncs,
//! so the other devices don't remind again. What this device has shown
//! is kept locally ([`SHOWN_META`]), so a restart doesn't show it twice.
//!
//! The app shows the notifications and runs the loop; this decides what is
//! [`due`], how long to [`next_wait`], what each notification says
//! ([`reminder`]) and what an answer does ([`acknowledge`]). The tasks come
//! from the store's `reminder_tasks`, which leaves out locked notes (their
//! tasks are sealed with their text), the trash, and tasks that are done.

use std::collections::BTreeMap;
use std::time::Duration;

use kanso_core::dates::Date;
use kanso_core::reminders::{GRACE_MS, Timed, due_now, next_wake, timed};
use kanso_core::tasks::{Due, due_label};
use kanso_store::{Origin, ReminderTask, Store};

use super::display_text;

/// The local record of reminders shown on this device: key → when shown.
pub const SHOWN_META: &str = "app.remindersShown";
/// The longest wait between looks: edits to tasks are picked up by then.
pub const MAX_WAIT: Duration = Duration::from_secs(30);
/// The shortest wait between looks.
pub const MIN_WAIT: Duration = Duration::from_millis(500);
pub const MARK_DONE: &str = "mark-done";
pub const DISMISS: &str = "dismiss";
/// The buttons of a reminder, as `(id, label)`.
pub const ACTIONS: [(&str, &str); 2] = [(MARK_DONE, "Mark Done"), (DISMISS, "Dismiss")];

/// A reminder on screen, for its response.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shown {
    pub note_id: String,
    pub line: u32,
    pub text: String,
    pub at_ms: i64,
}

impl Shown {
    /// `task`'s reminder, shown at `at_ms` (UTC).
    pub fn new(task: &ReminderTask, at_ms: i64) -> Shown {
        Shown {
            note_id: task.note_id.clone(),
            line: task.line,
            text: task.text.clone(),
            at_ms,
        }
    }
}

/// What a notification says.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reminder {
    /// The task's key: what a response names.
    pub tag: String,
    pub title: String,
    pub body: String,
}

/// Reads the shown record, tolerating anything unreadable. Pure.
pub fn parse_shown(raw: Option<&str>) -> BTreeMap<String, i64> {
    raw.and_then(|raw| serde_json::from_str(raw).ok())
        .unwrap_or_default()
}

/// Forgets reminders shown more than twice the grace period ago: they
/// can't be due again. Pure.
pub fn prune_shown(shown: &mut BTreeMap<String, i64>, now_ms: i64) {
    shown.retain(|_, at| now_ms - *at <= 2 * GRACE_MS);
}

/// The notification for a task: its words, and when it is due and in
/// which note. Pure.
pub fn reminder(task: &ReminderTask, today: Date) -> Reminder {
    let when = Due::parse(&task.due)
        .map(|due| due_label(due, today))
        .unwrap_or_else(|| task.due.clone());
    let title = if task.note_title.trim().is_empty() {
        "Untitled".to_owned()
    } else {
        task.note_title.clone()
    };
    Reminder {
        tag: task.key.clone(),
        title: display_text(&task.text),
        body: format!("{when} · {title}"),
    }
}

/// The timed tasks among `tasks`, with their times.
fn timed_tasks(tasks: &[ReminderTask]) -> Vec<(Timed, &ReminderTask)> {
    tasks
        .iter()
        .filter_map(|t| Some((timed(&t.note_id, &t.text, &t.due, false)?, t)))
        .collect()
}

/// The reminders to show at local time `now_local` (ms): due at or before
/// it, no more than the grace period ago, not acknowledged anywhere and
/// not in `shown`. Each task once, in the order they fall due. Pure.
pub fn due<'a>(
    tasks: &'a [ReminderTask],
    shown: &BTreeMap<String, i64>,
    now_local: i64,
) -> Vec<&'a ReminderTask> {
    let timed_tasks = timed_tasks(tasks);
    let list: Vec<Timed> = timed_tasks.iter().map(|(t, _)| t.clone()).collect();
    let acked = |key: &str| {
        tasks
            .iter()
            .any(|t| t.key == key && t.acks.contains_key(key))
    };
    let was_shown = |key: &str| shown.contains_key(key);
    let mut out: Vec<&ReminderTask> = Vec::new();
    for t in due_now(&list, &acked, &was_shown, now_local) {
        if let Some((_, task)) = timed_tasks.iter().find(|(timed, _)| timed.key == t.key)
            && !out.iter().any(|seen| seen.key == task.key)
        {
            out.push(task);
        }
    }
    out
}

/// A reminder still to come: its task, and when it falls due in local
/// milliseconds (see [`Due::local_ms`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Upcoming<'a> {
    pub task: &'a ReminderTask,
    pub at_local_ms: i64,
}

/// The reminders that fall due after local time `now_local` (ms) and no
/// later than `horizon` on: not acknowledged anywhere and not in `shown`.
/// Each task once, soonest first, at most `limit`. A phone has the system
/// show these at their times while the app is closed. Pure.
pub fn upcoming<'a>(
    tasks: &'a [ReminderTask],
    shown: &BTreeMap<String, i64>,
    now_local: i64,
    horizon: Duration,
    limit: usize,
) -> Vec<Upcoming<'a>> {
    let end = now_local.saturating_add(i64::try_from(horizon.as_millis()).unwrap_or(i64::MAX));
    let mut out: Vec<Upcoming<'a>> = Vec::new();
    for (timed, task) in timed_tasks(tasks) {
        let acked = tasks
            .iter()
            .any(|t| t.key == task.key && t.acks.contains_key(&task.key));
        if timed.at_local_ms <= now_local
            || timed.at_local_ms > end
            || acked
            || shown.contains_key(&task.key)
            || out.iter().any(|seen| seen.task.key == task.key)
        {
            continue;
        }
        out.push(Upcoming {
            task,
            at_local_ms: timed.at_local_ms,
        });
    }
    out.sort_by_key(|coming| coming.at_local_ms);
    out.truncate(limit);
    out
}

/// How long until the next look at local time `now_local` (ms): until the
/// next reminder falls due, kept between [`MIN_WAIT`] and [`MAX_WAIT`]. Pure.
pub fn next_wait(tasks: &[ReminderTask], now_local: i64) -> Duration {
    let list: Vec<Timed> = timed_tasks(tasks).into_iter().map(|(t, _)| t).collect();
    next_wake(&list, now_local)
        .map(|at| Duration::from_millis(u64::try_from(at - now_local).unwrap_or(0)))
        .unwrap_or(MAX_WAIT)
        .clamp(MIN_WAIT, MAX_WAIT)
}

/// Notes that reminders `keys` were shown at `now_ms`: adds them to
/// `shown`, forgets the old ones, and says the record as the text to keep
/// under [`SHOWN_META`]. Pure.
pub fn record(shown: &mut BTreeMap<String, i64>, keys: &[String], now_ms: i64) -> String {
    for key in keys {
        shown.insert(key.clone(), now_ms);
    }
    prune_shown(shown, now_ms);
    serde_json::to_string(shown).unwrap_or_default()
}

/// What the person did with a reminder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Answer {
    /// The Mark Done button: tick the task off.
    MarkDone,
    /// Any other button.
    Dismiss,
    /// A click on the notification itself: open the task.
    Open,
}

impl Answer {
    /// The answer for the id of the button pressed, if one was.
    pub fn from_action(id: Option<&str>) -> Answer {
        match id {
            Some(MARK_DONE) => Answer::MarkDone,
            Some(_) => Answer::Dismiss,
            None => Answer::Open,
        }
    }
}

/// Records `answer` to the reminder `key` that was `shown`: the note
/// remembers the acknowledgement whatever the answer, so no device reminds
/// again, and Mark Done also ticks the task off (at `now_ms`). A note
/// changed or gone since leaves nothing to do. Opening the task is the
/// caller's.
pub fn acknowledge(store: &Store, shown: &Shown, key: &str, answer: Answer, now_ms: i64) {
    if answer == Answer::MarkDone {
        store
            .set_task_checked(
                &shown.note_id,
                shown.line,
                &shown.text,
                true,
                now_ms,
                Origin::Ui,
            )
            .ok();
    }
    store
        .ack_reminder(&shown.note_id, key, shown.at_ms, Origin::Ui)
        .ok();
}

#[cfg(test)]
mod tests {
    use kanso_core::DocType;
    use kanso_core::reminders::task_key;
    use kanso_store::Put;
    use proptest::prelude::*;
    use serde_json::json;

    use super::*;

    // Thursday 22 October 2026, 09:00:30 UTC.
    const NOW: i64 = 1_792_659_630_000;

    #[test]
    fn shown_record_reads_and_prunes() {
        assert!(parse_shown(None).is_empty());
        assert!(parse_shown(Some("not json")).is_empty());
        let mut shown = parse_shown(Some(r#"{"a":10,"b":5000000}"#));
        prune_shown(&mut shown, 10 + 2 * GRACE_MS + 1);
        assert_eq!(shown.keys().collect::<Vec<_>>(), ["b"]);
    }

    fn task(note: &str, text: &str, due: &str) -> ReminderTask {
        ReminderTask {
            note_id: note.into(),
            note_title: "Plans".into(),
            line: 0,
            text: text.into(),
            due: due.into(),
            key: task_key(note, text, due),
            acks: BTreeMap::new(),
        }
    }

    fn local(due: &str) -> i64 {
        Due::parse(due).unwrap().local_ms()
    }

    #[test]
    fn a_reminder_says_what_and_when() {
        let task = task("note:a", "Call **Sam**", "2026-10-22T09:00");
        let today = Date::new(2026, 10, 22).unwrap();
        let said = reminder(&task, today);
        assert_eq!(said.tag, task.key);
        assert_eq!(said.title, "Call Sam");
        assert_eq!(said.body, "Today 09:00 · Plans");
        let mut untitled = task.clone();
        untitled.note_title = "  ".into();
        assert_eq!(reminder(&untitled, today).body, "Today 09:00 · Untitled");
    }

    #[test]
    fn what_is_due_is_shown_once_and_acks_silence_it() {
        let mut tasks = vec![
            task("note:a", "Call Sam", "2026-10-22T09:00"),
            task("note:a", "Later", "2026-10-22T18:00"),
            task("note:a", "No time", "2026-10-22"),
        ];
        let now = local("2026-10-22T09:00") + 30_000;
        let keys = |due: Vec<&ReminderTask>| due.iter().map(|t| t.text.clone()).collect::<Vec<_>>();
        let mut shown = BTreeMap::new();
        assert_eq!(keys(due(&tasks, &shown, now)), ["Call Sam"]);
        // Shown here: not again.
        record(&mut shown, &[tasks[0].key.clone()], NOW);
        assert!(due(&tasks, &shown, now).is_empty());
        // Acknowledged on another device: not here either.
        shown.clear();
        let key = tasks[0].key.clone();
        tasks[0].acks.insert(key, NOW);
        assert!(due(&tasks, &shown, now).is_empty());
        // Too long ago to notify.
        assert!(
            due(
                &tasks[1..],
                &shown,
                local("2026-10-22T18:00") + GRACE_MS + 1
            )
            .is_empty()
        );
    }

    #[test]
    fn the_wait_runs_to_the_next_reminder() {
        let tasks = vec![task("note:a", "Later", "2026-10-22T18:00")];
        let at = local("2026-10-22T18:00");
        assert_eq!(next_wait(&tasks, at - 5_000), Duration::from_secs(5));
        assert_eq!(next_wait(&tasks, at - 100), MIN_WAIT);
        assert_eq!(next_wait(&tasks, at - 3_600_000), MAX_WAIT);
        assert_eq!(next_wait(&tasks, at + 1), MAX_WAIT);
        assert_eq!(next_wait(&[], 0), MAX_WAIT);
    }

    #[test]
    fn upcoming_is_what_falls_due_next_within_the_horizon() {
        let mut tasks = vec![
            task("note:a", "Evening", "2026-10-22T18:00"),
            task("note:a", "Morning", "2026-10-22T09:30"),
            task("note:a", "Far off", "2026-12-01T09:00"),
            task("note:a", "Past", "2026-10-22T08:00"),
            task("note:a", "No time", "2026-10-22"),
        ];
        let now = local("2026-10-22T09:00");
        let day = Duration::from_secs(86_400);
        let names =
            |got: Vec<Upcoming>| got.iter().map(|c| c.task.text.clone()).collect::<Vec<_>>();
        let mut shown = BTreeMap::new();
        assert_eq!(
            names(upcoming(&tasks, &shown, now, day, 10)),
            ["Morning", "Evening"]
        );
        assert_eq!(names(upcoming(&tasks, &shown, now, day, 1)), ["Morning"]);
        assert_eq!(
            upcoming(&tasks, &shown, now, day, 10)[0].at_local_ms,
            local("2026-10-22T09:30")
        );
        // Not at the very moment it is due: that is `due`'s.
        assert!(
            upcoming(
                &tasks,
                &shown,
                local("2026-10-22T09:30"),
                Duration::ZERO,
                10
            )
            .is_empty()
        );
        shown.insert(tasks[1].key.clone(), NOW);
        let key = tasks[0].key.clone();
        tasks[0].acks.insert(key, NOW);
        assert!(upcoming(&tasks, &shown, now, day, 10).is_empty());
    }

    #[test]
    fn answers_follow_the_button() {
        assert_eq!(Answer::from_action(Some(MARK_DONE)), Answer::MarkDone);
        assert_eq!(Answer::from_action(Some(DISMISS)), Answer::Dismiss);
        assert_eq!(Answer::from_action(Some("anything")), Answer::Dismiss);
        assert_eq!(Answer::from_action(None), Answer::Open);
        assert_eq!(ACTIONS[0].0, MARK_DONE);
    }

    fn store_with_note(body: &str) -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("kanso.db")).unwrap();
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
                json!({"_id": "note:a", "title": "Plans", "bookId": "book:a", "body": body}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
        (dir, store)
    }

    #[test]
    fn mark_done_ticks_the_task_and_every_answer_acks() {
        let body = "- [ ] Call Sam @2026-10-22T09:00\n- [ ] Later @2026-10-22T18:00\n";
        let (_dir, store) = store_with_note(body);
        let tasks = store.reminder_tasks().unwrap();
        let first = tasks.iter().find(|t| t.text.starts_with("Call")).unwrap();
        let shown = Shown::new(first, NOW);
        acknowledge(&store, &shown, &first.key, Answer::MarkDone, NOW);
        let note = store.note("note:a").unwrap().unwrap();
        assert!(note.body.starts_with("- [x] Call Sam"), "{}", note.body);
        let ack = note.extra.get("kansoReminderAck").cloned().unwrap();
        assert_eq!(ack.get(&first.key).and_then(|v| v.as_i64()), Some(NOW));

        // Dismissing leaves the task open, and still acknowledges it.
        let (_dir, store) = store_with_note(body);
        let tasks = store.reminder_tasks().unwrap();
        acknowledge(
            &store,
            &Shown::new(&tasks[0], NOW),
            &tasks[0].key,
            Answer::Dismiss,
            NOW,
        );
        let note = store.note("note:a").unwrap().unwrap();
        assert!(note.body.starts_with("- [ ] Call Sam"));
        assert!(
            due(
                &store.reminder_tasks().unwrap(),
                &BTreeMap::new(),
                local("2026-10-22T09:00") + 1
            )
            .is_empty()
        );
    }

    /// A locked note's tasks are sealed with its text: no reminder is read
    /// from it, so none is due, shown or answered.
    #[test]
    fn locked_notes_give_no_reminders() {
        use kanso_sync::KdfParams;
        use kanso_sync::lock::{Envelope, LockKey, LockPassword};

        let body = "- [ ] Secret call @2026-10-22T09:00\n";
        let (_dir, store) = store_with_note(body);
        assert_eq!(store.reminder_tasks().unwrap().len(), 1);
        let key = LockKey::create(
            &LockPassword::new("a lock password").unwrap(),
            KdfParams::TEST,
        )
        .unwrap();
        let mut note = store.note("note:a").unwrap().unwrap();
        Envelope::store(Some(&key.seal("note:a", &note.body)), &mut note.extra);
        note.body = String::new();
        store
            .lock_note(serde_json::to_value(&note).unwrap())
            .unwrap();
        let tasks = store.reminder_tasks().unwrap();
        assert!(tasks.is_empty(), "{tasks:?}");
        assert!(due(&tasks, &BTreeMap::new(), local("2026-10-22T09:00") + 1).is_empty());
    }

    fn arb_task() -> impl Strategy<Value = ReminderTask> {
        (
            "[ab]",
            "[a-c]{1,3}",
            prop_oneof![
                Just("2026-10-22T09:00".to_owned()),
                Just("2026-10-22T09:05".to_owned()),
                Just("2026-10-22T18:00".to_owned()),
                Just("2026-10-22".to_owned()),
            ],
            any::<bool>(),
        )
            .prop_map(|(note, text, due, acked)| {
                let mut task = task(&format!("note:{note}"), &text, &due);
                if acked {
                    let key = task.key.clone();
                    task.acks.insert(key, NOW);
                }
                task
            })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// What is due is a subset of the timed tasks, each once, with
        /// nothing acknowledged or already shown, and agrees with the
        /// rule in `kanso_core::reminders::due_now`.
        #[test]
        fn prop_due_is_a_clean_subset(
            tasks in prop::collection::vec(arb_task(), 0..10),
            shown_keys in prop::collection::vec(0usize..10, 0..4),
            offset in -60_000i64..4_000_000,
        ) {
            let now = local("2026-10-22T09:00") + offset;
            let shown: BTreeMap<String, i64> = shown_keys
                .iter()
                .filter_map(|&i| tasks.get(i))
                .map(|t| (t.key.clone(), NOW))
                .collect();
            let got = due(&tasks, &shown, now);
            let mut seen = std::collections::HashSet::new();
            for t in &got {
                prop_assert!(seen.insert(&t.key), "twice: {}", t.key);
                prop_assert!(tasks.iter().any(|x| x.key == t.key));
                prop_assert!(Due::parse(&t.due).is_some_and(|d| d.time.is_some()));
                prop_assert!(!shown.contains_key(&t.key));
                prop_assert!(!t.acks.contains_key(&t.key));
            }
            let list: Vec<Timed> = timed_tasks(&tasks).into_iter().map(|(t, _)| t).collect();
            let acked = |key: &str| tasks.iter().any(|t| t.key == key && t.acks.contains_key(key));
            let was_shown = |key: &str| shown.contains_key(key);
            let mut distinct: Vec<&str> = Vec::new();
            for t in due_now(&list, &acked, &was_shown, now) {
                if !distinct.contains(&t.key.as_str()) {
                    distinct.push(&t.key);
                }
            }
            prop_assert_eq!(got.iter().map(|t| t.key.as_str()).collect::<Vec<_>>(), distinct);
        }

        /// What is upcoming is a subset of the timed tasks, each once,
        /// after now and within the horizon, soonest first, never past the
        /// limit, never acknowledged or shown, and never also due. With a
        /// limit to spare it is every task that qualifies.
        #[test]
        fn prop_upcoming_is_a_clean_prefix(
            tasks in prop::collection::vec(arb_task(), 0..10),
            shown_keys in prop::collection::vec(0usize..10, 0..4),
            offset in -60_000i64..40_000_000,
            horizon_s in 0u64..100_000,
            limit in 0usize..12,
        ) {
            let now = local("2026-10-22T09:00") + offset;
            let horizon = Duration::from_secs(horizon_s);
            let shown: BTreeMap<String, i64> = shown_keys
                .iter()
                .filter_map(|&i| tasks.get(i))
                .map(|t| (t.key.clone(), NOW))
                .collect();
            let got = upcoming(&tasks, &shown, now, horizon, limit);
            let all = upcoming(&tasks, &shown, now, horizon, usize::MAX);
            prop_assert_eq!(got.len(), all.len().min(limit));
            prop_assert_eq!(&got[..], &all[..got.len()]);
            let in_due: Vec<String> = due(&tasks, &shown, now).iter().map(|t| t.key.clone()).collect();
            let mut seen = std::collections::HashSet::new();
            for coming in &got {
                let t = coming.task;
                prop_assert!(seen.insert(&t.key));
                prop_assert_eq!(coming.at_local_ms, Due::parse(&t.due).unwrap().local_ms());
                prop_assert!(coming.at_local_ms > now);
                prop_assert!(coming.at_local_ms - now <= horizon_s as i64 * 1000);
                prop_assert!(!shown.contains_key(&t.key) && !t.acks.contains_key(&t.key));
                prop_assert!(!in_due.contains(&t.key));
            }
            for pair in all.windows(2) {
                prop_assert!(pair[0].at_local_ms <= pair[1].at_local_ms);
            }
            // Nothing that qualifies is left out when the limit allows.
            for t in &tasks {
                let qualifies = Due::parse(&t.due).is_some_and(|d| {
                    d.time.is_some() && d.local_ms() > now && d.local_ms() - now <= horizon_s as i64 * 1000
                }) && !shown.contains_key(&t.key)
                    && !tasks.iter().any(|x| x.key == t.key && x.acks.contains_key(&x.key));
                prop_assert_eq!(qualifies, all.iter().any(|c| c.task.key == t.key));
            }
        }

        /// The wait always lies between the shortest and longest look.
        #[test]
        fn prop_the_wait_is_bounded(
            tasks in prop::collection::vec(arb_task(), 0..10),
            offset in -10_000_000i64..10_000_000,
        ) {
            let wait = next_wait(&tasks, local("2026-10-22T09:00") + offset);
            prop_assert!(wait >= MIN_WAIT && wait <= MAX_WAIT);
        }

        /// The record reads back as the pruned map, and holds each key it
        /// was given at the time it was given.
        #[test]
        fn prop_record_reads_back(
            before in prop::collection::btree_map("[a-d]", -10_000_000i64..10_000_000, 0..5),
            keys in prop::collection::vec("[a-f]", 0..4),
            now in 0i64..10_000_000,
        ) {
            let mut shown = before;
            let text = record(&mut shown, &keys, now);
            prop_assert_eq!(parse_shown(Some(&text)), shown.clone());
            for key in &keys {
                prop_assert_eq!(shown.get(key), Some(&now));
            }
            prop_assert!(shown.values().all(|at| now - at <= 2 * GRACE_MS));
        }
    }
}
