//! Reminders for repeating tasks (#390, slice 5): each occurrence reminds
//! once, ticking moves the reminder to the next, the acknowledgements stay
//! bounded and replaying a sync doesn't duplicate. The seed is pinned.

use std::collections::BTreeMap;
use std::time::Duration;

use kanso_core::DocType;
use kanso_core::dates::Date;
use kanso_core::reminders::{
    ACK_KEEP_MS, Acks, GRACE_MS, key_due_ms, merge_acks, read_acks, task_key,
};
use kanso_core::tasks::Due;
use kanso_engine::tasks::reminders::{Answer, Shown, acknowledge, due, reminder, upcoming};
use kanso_store::{Origin, Put, Store};
use proptest::prelude::*;
use proptest::test_runner::{Config, RngAlgorithm, RngSeed};
use serde_json::json;

const MINUTE_MS: i64 = 60_000;
const FAR: Duration = Duration::from_secs(400 * 24 * 3600);

fn config() -> Config {
    Config {
        cases: 48,
        rng_seed: RngSeed::Fixed(0x0072_656d_696e),
        rng_algorithm: RngAlgorithm::ChaCha,
        failure_persistence: None,
        ..Config::default()
    }
}

fn store_with(body: &str) -> (tempfile::TempDir, Store) {
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
            json!({"_id": "note:r", "title": "Shoots", "bookId": "book:a", "body": body, "updatedAt": 1}),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();
    (dir, store)
}

fn acks_of(store: &Store) -> Acks {
    let note = store.note("note:r").unwrap().unwrap();
    read_acks(&note.extra)
}

proptest! {
    #![proptest_config(config())]

    /// Whatever the rule and however early or late each answer comes, the
    /// note has exactly one open timed occurrence, so exactly one pending
    /// reminder (due now, or still to come); ticking moves it to a later
    /// due with a new key; the log lines never remind and never repeat;
    /// and the acknowledgements stay bounded.
    #[test]
    fn prop_one_pending_reminder_per_series_and_acks_stay_bounded(
        rule in prop::sample::select(vec!["every day", "every week", "every 2 weeks", "every month", "every weekday"]),
        start_days in 20_000i64..21_000,
        time in (0u32..24, 0u32..60),
        offsets in prop::collection::vec(-1_440i64..1_440, 1..40),
        dismiss_first in prop::collection::vec(any::<bool>(), 40),
    ) {
        let first = Due { date: Date::from_days(start_days), time: Some(time) };
        let (_dir, store) = store_with(&format!("- [ ] film @{} {rule}\n", first.format()));
        let mut last_due = i64::MIN;
        for (offset, dismiss) in offsets.iter().zip(dismiss_first) {
            let tasks = store.reminder_tasks().unwrap();
            prop_assert_eq!(tasks.len(), 1, "one open timed occurrence");
            let task = &tasks[0];
            let at = Due::parse(&task.due).unwrap().local_ms();
            prop_assert!(at > last_due, "each occurrence is later");
            last_due = at;

            let now = at + offset * MINUTE_MS;
            let none = BTreeMap::new();
            let now_pending = due(&tasks, &none, now).len();
            let to_come = upcoming(&tasks, &none, now, FAR, 100).len();
            let missed = usize::from(now - at > GRACE_MS);
            prop_assert_eq!(now_pending + to_come + missed, 1);
            prop_assert!(reminder(task, Date::from_local_ms(now)).body.contains(rule));

            if dismiss {
                acknowledge(&store, &Shown::new(task, now), &task.key, Answer::Dismiss, now);
                let again = store.reminder_tasks().unwrap();
                prop_assert!(due(&again, &none, now).is_empty());
                prop_assert!(upcoming(&again, &none, now, FAR, 100).is_empty());
            }
            acknowledge(&store, &Shown::new(task, now), &task.key, Answer::MarkDone, now);
            let after = store.reminder_tasks().unwrap();
            prop_assert_eq!(after.len(), 1, "the next instance replaces it");
            prop_assert_ne!(&after[0].key, &task.key);
            prop_assert!(acks_of(&store).len() <= 2, "{:?}", acks_of(&store));
        }
        let body = store.note("note:r").unwrap().unwrap().body;
        for line in body.lines().filter(|l| l.starts_with("- [x]")) {
            prop_assert!(!line.contains("every"), "{line}");
        }
        prop_assert_eq!(body.lines().filter(|l| l.starts_with("- [ ]")).count(), 1);
    }

    /// Merging the same acknowledgements again (a sync replayed) adds
    /// nothing, nothing past the keep window survives, and everything
    /// inside it does.
    #[test]
    fn prop_sync_replay_does_not_duplicate_and_old_acks_go(
        weeks in prop::collection::btree_set(0i64..200, 0..40),
        split in 0usize..40,
        now_week in 0i64..220,
    ) {
        let due_of = |w: i64| format!("{}T09:00", Date::from_days(20_000 + w * 7).iso());
        let key = |w: i64| task_key("note:r", "film", &due_of(w));
        let all: Vec<i64> = weeks.iter().copied().collect();
        let (left, right) = all.split_at(split.min(all.len()));
        // Two devices that each saw part, and one that saw it all.
        let acks = |ws: &[i64]| -> Acks { ws.iter().map(|w| (key(*w), *w)).collect() };
        let now = Due { date: Date::from_days(20_000 + now_week * 7), time: Some((12, 0)) }.local_ms();
        let merged = merge_acks(&acks(left), &acks(right), now);
        prop_assert_eq!(&merged, &merge_acks(&acks(&all), &acks(&all), now));
        prop_assert_eq!(&merge_acks(&merged, &acks(left), now), &merged);
        prop_assert_eq!(&merge_acks(&merged, &merged, now), &merged);
        for w in &all {
            let age = now - Due::parse(&due_of(*w)).unwrap().local_ms();
            prop_assert_eq!(merged.contains_key(&key(*w)), age <= ACK_KEEP_MS);
        }
        for k in merged.keys() {
            prop_assert!(now - key_due_ms(k).unwrap() <= ACK_KEEP_MS);
        }
        let past = merged.keys().filter(|k| key_due_ms(k).unwrap() <= now).count();
        prop_assert!(past <= 5, "weekly keys in the last 30 days: {past}");
    }
}
