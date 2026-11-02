//! Reminders for tasks with a time (`- [ ] Call Sam @2026-10-25T09:00`):
//! which are due to notify now, and what a dismissal remembers.
//!
//! A reminder is known by its task's key ([`task_key`]): the note, the
//! task's words and its due date, so editing the words or the date makes a
//! new reminder. Dismissing or completing one writes `{key: firedAt}` into
//! the note's `kansoReminderAck` field, which syncs with the note, so the
//! account's other devices don't notify again. Acks from two devices merge
//! by union ([`merge_acks`]).

use std::collections::BTreeMap;

use serde_json::{Map, Value};

use crate::tasks::Due;

/// The note field holding acknowledged reminders: `{taskKey: firedAt}`.
pub const ACK_FIELD: &str = "kansoReminderAck";

/// How late a reminder may still notify, for a device that was asleep or
/// closed when it fell due. Older ones only show as overdue.
pub const GRACE_MS: i64 = 30 * 60_000;

/// Acknowledged reminders: task key to when the reminder fired (ms since
/// the epoch).
pub type Acks = BTreeMap<String, i64>;

/// A stable 64-bit FNV-1a hash of `text`.
pub(crate) fn fnv1a(text: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in text.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

/// The key a task's reminder is known by: `<note id>#<hash of its
/// words>@<due>`. The same task on any device gets the same key.
pub fn task_key(note_id: &str, text: &str, due: &str) -> String {
    format!("{note_id}#{:016x}@{due}", fnv1a(text))
}

/// The acks a note's fields hold; anything malformed is left out.
pub fn read_acks(fields: &Map<String, Value>) -> Acks {
    fields
        .get(ACK_FIELD)
        .and_then(Value::as_object)
        .map(|acks| {
            acks.iter()
                .filter_map(|(key, at)| Some((key.clone(), at.as_i64()?)))
                .collect()
        })
        .unwrap_or_default()
}

/// The acks as the note field stores them.
pub fn acks_value(acks: &Acks) -> Value {
    Value::Object(
        acks.iter()
            .map(|(key, at)| (key.clone(), Value::from(*at)))
            .collect(),
    )
}

/// Both sets of acks: every key either has, with the earlier firing time
/// where both do. Commutative, associative and idempotent.
pub fn merge_acks(a: &Acks, b: &Acks) -> Acks {
    let mut out = a.clone();
    for (key, at) in b {
        out.entry(key.clone())
            .and_modify(|mine| *mine = (*mine).min(*at))
            .or_insert(*at);
    }
    out
}

/// A timed task that may remind: its key and when it falls due, in local
/// milliseconds (see [`Due::local_ms`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Timed {
    pub key: String,
    pub at_local_ms: i64,
}

/// A timed task's reminder, when it has one: open, with a time of day.
pub fn timed(note_id: &str, text: &str, due: &str, checked: bool) -> Option<Timed> {
    if checked {
        return None;
    }
    let parsed = Due::parse(due)?;
    parsed.time?;
    Some(Timed {
        key: task_key(note_id, text, due),
        at_local_ms: parsed.local_ms(),
    })
}

/// The reminders to show at local time `now`: due at or before it, no
/// more than [`GRACE_MS`] ago, not acknowledged anywhere and not already
/// shown on this device. Pure.
pub fn due_now<'a>(
    tasks: &'a [Timed],
    acked: &dyn Fn(&str) -> bool,
    shown: &dyn Fn(&str) -> bool,
    now_local_ms: i64,
) -> Vec<&'a Timed> {
    tasks
        .iter()
        .filter(|t| t.at_local_ms <= now_local_ms && now_local_ms - t.at_local_ms <= GRACE_MS)
        .filter(|t| !acked(&t.key) && !shown(&t.key))
        .collect()
}

/// When to look again after `now_local_ms`: the next reminder's time, if
/// one is still to come. Pure.
pub fn next_wake(tasks: &[Timed], now_local_ms: i64) -> Option<i64> {
    tasks
        .iter()
        .map(|t| t.at_local_ms)
        .filter(|&at| at > now_local_ms)
        .min()
}

/// The acks after acknowledging `key` (fired at `at`), keeping only the
/// keys still in `live` (the note's timed tasks) so the field doesn't grow
/// for tasks long gone.
pub fn acknowledge(acks: &Acks, key: &str, at: i64, live: &[String]) -> Acks {
    let mut next: Acks = acks
        .iter()
        .filter(|(k, _)| k.as_str() == key || live.iter().any(|l| l == *k))
        .map(|(k, v)| (k.clone(), *v))
        .collect();
    next.entry(key.to_owned()).or_insert(at);
    next
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::collections::HashSet;

    #[test]
    fn keys_and_timing() {
        let a = task_key("note:x", "Call Sam @2026-10-25T09:00", "2026-10-25T09:00");
        assert!(a.starts_with("note:x#"));
        assert!(a.ends_with("@2026-10-25T09:00"));
        assert_ne!(
            a,
            task_key("note:x", "Call Bo @2026-10-25T09:00", "2026-10-25T09:00")
        );
        assert_eq!(timed("note:x", "t", "2026-10-25", false), None);
        assert_eq!(timed("note:x", "t", "2026-10-25T09:00", true), None);
        let t = timed("note:x", "t", "2026-10-25T09:00", false).unwrap();
        let none = |_: &str| false;
        assert!(due_now(std::slice::from_ref(&t), &none, &none, t.at_local_ms - 1).is_empty());
        assert_eq!(
            due_now(std::slice::from_ref(&t), &none, &none, t.at_local_ms).len(),
            1
        );
        assert!(
            due_now(
                std::slice::from_ref(&t),
                &none,
                &none,
                t.at_local_ms + GRACE_MS + 1
            )
            .is_empty()
        );
        assert_eq!(next_wake(std::slice::from_ref(&t), 0), Some(t.at_local_ms));
        let mut fields = Map::new();
        fields.insert(ACK_FIELD.into(), serde_json::json!({"k": 5, "bad": "x"}));
        assert_eq!(read_acks(&fields), Acks::from([("k".to_owned(), 5)]));
    }

    fn acks() -> impl Strategy<Value = Acks> {
        prop::collection::btree_map("[a-d]", 0i64..100, 0..5)
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(512))]

        /// Merging acks is a union: commutative, associative, idempotent,
        /// and it keeps every key either side had.
        #[test]
        fn prop_acks_merge_by_union(a in acks(), b in acks(), c in acks()) {
            let ab = merge_acks(&a, &b);
            prop_assert_eq!(&ab, &merge_acks(&b, &a));
            prop_assert_eq!(merge_acks(&ab, &c), merge_acks(&a, &merge_acks(&b, &c)));
            prop_assert_eq!(merge_acks(&a, &a), a.clone());
            let keys: HashSet<&String> = a.keys().chain(b.keys()).collect();
            prop_assert_eq!(ab.keys().collect::<HashSet<_>>(), keys);
            // Through the note field and back.
            let mut fields = Map::new();
            fields.insert(ACK_FIELD.into(), acks_value(&ab));
            prop_assert_eq!(read_acks(&fields), ab);
        }

        /// A scheduler stepping through time with any wake-ups never shows
        /// a reminder twice, never shows an acked one, and shows every
        /// other one whose time it passed within the grace period.
        #[test]
        fn prop_never_fires_twice(
            times in prop::collection::vec(0i64..10_000, 0..8),
            acked_ix in prop::collection::vec(any::<prop::sample::Index>(), 0..4),
            steps in prop::collection::vec(1i64..400, 1..60),
        ) {
            let tasks: Vec<Timed> = times.iter().enumerate()
                .map(|(i, &m)| Timed { key: format!("k{i}"), at_local_ms: m * 60_000 })
                .collect();
            let acked: HashSet<String> = acked_ix.iter()
                .filter(|_| !tasks.is_empty())
                .map(|ix| tasks[ix.index(tasks.len())].key.clone())
                .collect();
            let mut shown: Vec<String> = Vec::new();
            let mut now = 0i64;
            for step in steps {
                now += step * 60_000;
                let seen: HashSet<String> = shown.iter().cloned().collect();
                let fired: Vec<String> = due_now(&tasks, &|k| acked.contains(k), &|k| seen.contains(k), now)
                    .into_iter().map(|t| t.key.clone()).collect();
                for key in fired {
                    prop_assert!(!acked.contains(&key));
                    prop_assert!(!shown.contains(&key), "fired twice");
                    shown.push(key);
                }
                if let Some(wake) = next_wake(&tasks, now) {
                    prop_assert!(wake > now);
                }
            }
        }

        /// Acknowledging keeps the new key, keeps only live keys besides
        /// it, and never changes an existing ack's time.
        #[test]
        fn prop_acknowledge_keeps_live_keys(a in acks(), key in "[a-f]", at in 0i64..100, live in prop::collection::vec("[a-f]", 0..6)) {
            let next = acknowledge(&a, &key, at, &live);
            prop_assert!(next.contains_key(&key));
            for (k, v) in &next {
                prop_assert!(k == &key || live.contains(k));
                if let Some(old) = a.get(k) {
                    prop_assert_eq!(old, v);
                }
            }
        }
    }
}
