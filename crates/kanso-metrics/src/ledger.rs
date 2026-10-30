//! The day's bookkeeping (docs/METRICS.md › Events, Sending): which IDs
//! were counted active today, which features were used and are waiting, and
//! which were already sent. The caller keeps it (the desktop in its config
//! as `metrics`) and passes the day in, so every function here is pure.

use serde::{Deserialize, Serialize};

use crate::batch::{Batch, Event, EventKind, MAX_EVENTS};
use crate::send::Outcome;
use crate::values::{Active, Day, Device, Feature, Id, install_id};

/// What has been counted on the day it holds. A day earlier than today
/// counts as empty: its waiting features are never sent.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ledger {
    /// This install's `i_` ID, made the first time it is needed.
    #[serde(skip_serializing_if = "Option::is_none")]
    install_id: Option<Id>,
    /// The UTC day the lists below are for.
    #[serde(skip_serializing_if = "Option::is_none")]
    day: Option<Day>,
    /// The IDs whose `app_active` that day is done.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    active_sent: Vec<Id>,
    /// The feature names whose `feature_used` that day is done.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    features_sent: Vec<String>,
    /// Features used that day and not yet sent (`import:<source>` for an
    /// import).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pending: Vec<String>,
}

/// Reads whatever is there and keeps only what is valid: a damaged entry
/// costs a count, never the settings file it sits in.
impl<'de> Deserialize<'de> for Ledger {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Ledger, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        let text = |key: &str| value.get(key).and_then(|v| v.as_str());
        let list = |key: &str| -> Vec<String> {
            value
                .get(key)
                .and_then(|v| v.as_array())
                .map(|items| {
                    items
                        .iter()
                        .filter_map(|v| v.as_str().map(str::to_owned))
                        .collect()
                })
                .unwrap_or_default()
        };
        let feature_names: Vec<&str> = Feature::all().map(Feature::name).collect();
        let mut ledger = Ledger {
            install_id: text("installId")
                .and_then(Id::parse)
                .filter(|id| id.as_str().starts_with("i_")),
            day: text("day").and_then(Day::parse),
            active_sent: list("activeSent")
                .iter()
                .filter_map(|id| Id::parse(id))
                .collect(),
            features_sent: list("featuresSent")
                .into_iter()
                .filter(|name| feature_names.contains(&name.as_str()))
                .collect(),
            pending: Vec::new(),
        };
        for key in list("pending") {
            if let Some(feature) = Feature::from_key(&key) {
                ledger.queue(feature);
            }
        }
        ledger.active_sent.dedup();
        ledger.features_sent.dedup();
        Ok(ledger)
    }
}

impl Ledger {
    /// This install's ID, made now if there is none (the caller saves the
    /// ledger after).
    pub fn install_id(&mut self) -> Id {
        self.install_id.get_or_insert_with(install_id).clone()
    }

    /// The install ID if one was made.
    pub fn existing_install_id(&self) -> Option<&Id> {
        self.install_id.as_ref()
    }

    /// Notes that `feature` was used `today`. Says whether the ledger
    /// changed: not for a feature already waiting or sent today, nor for a
    /// day before the one it holds (the clock went back).
    pub fn record(&mut self, today: Day, feature: Feature) -> bool {
        if !self.roll(today) {
            return false;
        }
        let name = feature.name();
        if self.features_sent.iter().any(|sent| sent == name) {
            return false;
        }
        self.queue(feature)
    }

    /// Adds a feature unless one of its name is waiting.
    fn queue(&mut self, feature: Feature) -> bool {
        let name = feature.name();
        if self
            .pending
            .iter()
            .any(|key| Feature::from_key(key).is_some_and(|f| f.name() == name))
        {
            return false;
        }
        self.pending.push(feature.key());
        true
    }

    /// Moves the ledger to `today`, emptying it when the day changed. False
    /// for a day before the one it holds.
    fn roll(&mut self, today: Day) -> bool {
        match self.day {
            Some(day) if day > today => false,
            Some(day) if day == today => true,
            _ => {
                *self = Ledger {
                    install_id: self.install_id.take(),
                    day: Some(today),
                    ..Ledger::default()
                };
                true
            }
        }
    }

    /// What to send `today` as `id`: `app_active` unless it is done for
    /// this ID today, and each feature waiting from today. `None` when there
    /// is nothing. Waiting features from an earlier day are left out.
    pub fn due(&self, today: Day, id: &Id, device: &Device, active: &Active) -> Option<Batch> {
        let (sent, pending): (&[Id], &[String]) = match self.day {
            Some(day) if day == today => (&self.active_sent, &self.pending),
            Some(day) if day > today => return None,
            _ => (&[], &[]),
        };
        let mut events = Vec::new();
        if !sent.contains(id) {
            events.push(Event {
                day: today,
                kind: EventKind::AppActive(active.clone()),
            });
        }
        events.extend(
            pending
                .iter()
                .filter_map(|key| Feature::from_key(key))
                .map(|feature| Event {
                    day: today,
                    kind: EventKind::FeatureUsed(feature),
                }),
        );
        events.truncate(MAX_EVENTS);
        (!events.is_empty()).then(|| Batch {
            id: id.clone(),
            device: device.clone(),
            events,
        })
    }

    /// Takes in how sending `batch` went. Accepted, or refused as invalid
    /// (it would only fail again): its events are done for their day. Kept
    /// for a later try otherwise. Says whether the ledger changed.
    pub fn settle(&mut self, batch: &Batch, outcome: Outcome) -> bool {
        if !matches!(outcome, Outcome::Accepted | Outcome::Rejected) {
            return false;
        }
        let mut changed = false;
        for event in &batch.events {
            if !self.roll(event.day) {
                continue;
            }
            match &event.kind {
                EventKind::AppActive(_) => {
                    if !self.active_sent.contains(&batch.id) {
                        self.active_sent.push(batch.id.clone());
                        changed = true;
                    }
                }
                EventKind::FeatureUsed(feature) => {
                    let name = feature.name();
                    let before = self.pending.len();
                    self.pending
                        .retain(|key| Feature::from_key(key).is_none_or(|f| f.name() != name));
                    changed |= self.pending.len() != before;
                    if !self.features_sent.iter().any(|sent| sent == name) {
                        self.features_sent.push(name.to_owned());
                        changed = true;
                    }
                }
            }
        }
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::batch::MAX_BYTES;
    use crate::values::{Form, ImportSource, Platform, SyncMode};
    use proptest::prelude::*;
    use std::collections::{HashMap, HashSet};

    fn day(text: &str) -> Day {
        Day::parse(text).unwrap()
    }

    fn device() -> Device {
        Device::new(Platform::Macos, Form::Desktop, "0.9.0")
    }

    fn active() -> Active {
        Active::new("15.1", SyncMode::Off, false, 3)
    }

    fn account() -> Id {
        Id::parse(&format!("a_{}", "1".repeat(32))).unwrap()
    }

    fn names(batch: &Batch) -> Vec<&'static str> {
        batch
            .events
            .iter()
            .map(|e| match &e.kind {
                EventKind::AppActive(_) => "app_active",
                EventKind::FeatureUsed(f) => f.name(),
            })
            .collect()
    }

    #[test]
    fn a_day_of_use() {
        let mut ledger = Ledger::default();
        let id = ledger.install_id();
        assert_eq!(ledger.install_id(), id, "made once");
        let today = day("2026-10-30");

        // Launch: app_active only.
        let batch = ledger.due(today, &id, &device(), &active()).unwrap();
        assert_eq!(names(&batch), ["app_active"]);
        // No answer: kept, and the next try has it again.
        assert!(!ledger.settle(&batch, Outcome::Retry));
        assert_eq!(
            ledger.due(today, &id, &device(), &active()),
            Some(batch.clone())
        );
        assert!(ledger.settle(&batch, Outcome::Accepted));
        assert_eq!(ledger.due(today, &id, &device(), &active()), None);

        // Two imports and a lock: one import (the first source) and the lock.
        assert!(ledger.record(today, Feature::Import(ImportSource::Bear)));
        assert!(!ledger.record(today, Feature::Import(ImportSource::Notion)));
        assert!(ledger.record(today, Feature::Lock));
        let batch = ledger.due(today, &id, &device(), &active()).unwrap();
        assert_eq!(names(&batch), ["import", "lock"]);
        assert_eq!(
            batch.events[0].kind,
            EventKind::FeatureUsed(Feature::Import(ImportSource::Bear))
        );
        // Used again while it is being sent: still once.
        assert!(!ledger.record(today, Feature::Lock));
        ledger.settle(&batch, Outcome::Accepted);
        assert!(!ledger.record(today, Feature::Lock));
        assert_eq!(ledger.due(today, &id, &device(), &active()), None);

        // Signing in counts the account too that day, without the features
        // already counted.
        let batch = ledger.due(today, &account(), &device(), &active()).unwrap();
        assert_eq!(names(&batch), ["app_active"]);

        // A refused batch is dropped like a sent one.
        ledger.record(today, Feature::Export);
        let batch = ledger.due(today, &account(), &device(), &active()).unwrap();
        ledger.settle(&batch, Outcome::Rejected);
        assert_eq!(ledger.due(today, &account(), &device(), &active()), None);

        // The next day starts over; what waited from yesterday is dropped.
        ledger.record(today, Feature::Ai);
        let tomorrow = today.next();
        let batch = ledger.due(tomorrow, &id, &device(), &active()).unwrap();
        assert_eq!(names(&batch), ["app_active"]);
        assert!(batch.events.iter().all(|e| e.day == tomorrow));
        // And the clock going back sends nothing for the earlier day.
        ledger.record(tomorrow, Feature::Ai);
        assert_eq!(ledger.due(today, &id, &device(), &active()), None);
        assert!(!ledger.record(today, Feature::Publish));
        assert_eq!(ledger.install_id(), id, "the install ID outlives the days");
    }

    #[test]
    fn reads_damaged_settings() {
        let json = serde_json::json!({
            "installId": "i_00112233445566778899aabbccddeeff",
            "day": "2026-10-30",
            "activeSent": ["a_00112233445566778899aabbccddeeff", "nope", 5],
            "featuresSent": ["lock", "spying"],
            "pending": ["ai", "ai", "import:bear", "import:nowhere", {"x": 1}],
            "extra": true
        });
        let ledger: Ledger = serde_json::from_value(json).unwrap();
        assert_eq!(ledger.active_sent.len(), 1);
        assert_eq!(ledger.features_sent, ["lock"]);
        assert_eq!(ledger.pending, ["ai", "import:bear"]);
        let back: Ledger = serde_json::from_slice(&serde_json::to_vec(&ledger).unwrap()).unwrap();
        assert_eq!(back, ledger);
        // An account ID is never kept as the install's.
        let ledger: Ledger =
            serde_json::from_value(serde_json::json!({"installId": account().as_str()})).unwrap();
        assert_eq!(ledger.existing_install_id(), None);
        for odd in ["null", "[]", "3", "\"x\"", "{}"] {
            assert!(serde_json::from_str::<Ledger>(odd).is_ok(), "{odd}");
        }
        assert_eq!(serde_json::to_string(&Ledger::default()).unwrap(), "{}");
    }

    // MARK: Properties

    #[derive(Clone, Debug)]
    enum Op {
        /// Moves the clock this many days (back for a negative number).
        Clock(i8),
        Use(Feature),
        /// Builds the batch for one of the IDs and settles it.
        Send(usize, Outcome),
        /// Saves and loads the ledger, as a restart does.
        Restart,
    }

    fn feature() -> impl Strategy<Value = Feature> {
        (0usize..21).prop_map(|ix| Feature::all().nth(ix).unwrap())
    }

    fn outcome() -> impl Strategy<Value = Outcome> {
        prop::sample::select(vec![
            Outcome::Accepted,
            Outcome::Rejected,
            Outcome::Retry,
            Outcome::Off,
        ])
    }

    fn op() -> impl Strategy<Value = Op> {
        prop_oneof![
            1 => (-1i8..=2).prop_map(Op::Clock),
            3 => feature().prop_map(Op::Use),
            3 => (0usize..3, outcome()).prop_map(|(id, o)| Op::Send(id, o)),
            1 => Just(Op::Restart),
        ]
    }

    fn device_strategy() -> impl Strategy<Value = Device> {
        (
            prop::sample::select(Platform::ALL.to_vec()),
            prop::sample::select(vec![Form::Desktop, Form::Phone, Form::Tablet]),
            "\\PC{0,20}|[0-9]{1,5}\\.[0-9]{1,5}\\.[0-9]{1,5}",
        )
            .prop_map(|(p, f, v)| Device::new(p, f, &v))
    }

    fn active_strategy() -> impl Strategy<Value = Active> {
        (
            "\\PC{0,40}|[0-9.]{1,10}",
            prop::sample::select(vec![SyncMode::Off, SyncMode::Kanso, SyncMode::SelfHosted]),
            any::<bool>(),
            any::<u64>(),
        )
            .prop_map(|(os, sync, ai, notes)| Active::new(&os, sync, ai, notes))
    }

    /// What the Worker checks (docs/METRICS.md › The batch), written from
    /// the page rather than from this crate's types.
    fn check_like_the_worker(body: &[u8]) -> Result<(), String> {
        let digits = |s: &str, lo: usize, hi: usize| {
            (lo..=hi).contains(&s.len()) && s.bytes().all(|b| b.is_ascii_digit())
        };
        let one_of =
            |v: &serde_json::Value, list: &[&str]| v.as_str().is_some_and(|s| list.contains(&s));
        if body.len() > MAX_BYTES {
            return Err("over 16 KiB".into());
        }
        let batch: serde_json::Value = serde_json::from_slice(body).map_err(|e| e.to_string())?;
        let top = batch.as_object().ok_or("not an object")?;
        let keys: HashSet<&str> = top.keys().map(String::as_str).collect();
        if keys != HashSet::from(["v", "id", "events"]) || batch["v"] != 1 {
            return Err("top level".into());
        }
        let id = batch["id"].as_str().ok_or("id")?;
        if Id::parse(id).is_none() {
            return Err("id".into());
        }
        let events = batch["events"].as_array().ok_or("events")?;
        if events.is_empty() || events.len() > 32 {
            return Err("event count".into());
        }
        let common = ["platform", "form", "app_version"];
        for event in events {
            let fields: HashSet<&str> = event
                .as_object()
                .ok_or("event")?
                .keys()
                .map(String::as_str)
                .collect();
            if fields != HashSet::from(["event", "day", "props"]) {
                return Err("event fields".into());
            }
            let day = event["day"].as_str().ok_or("day")?;
            let b = day.as_bytes();
            if b.len() != 10
                || b[4] != b'-'
                || b[7] != b'-'
                || !digits(&day[..4], 4, 4)
                || !digits(&day[5..7], 2, 2)
                || !digits(&day[8..], 2, 2)
            {
                return Err("day".into());
            }
            let props = event["props"].as_object().ok_or("props")?;
            let keys: HashSet<&str> = props.keys().map(String::as_str).collect();
            if !one_of(
                &props["platform"],
                &["macos", "windows", "linux", "ios", "android"],
            ) || !one_of(&props["form"], &["desktop", "phone", "tablet"])
            {
                return Err("platform or form".into());
            }
            let version = props["app_version"].as_str().ok_or("app_version")?;
            let parts: Vec<&str> = version.split('.').collect();
            if parts.len() != 3 || !parts.iter().all(|p| digits(p, 1, 4)) {
                // `unknown` is refused, as the page says.
                return Err("app_version".into());
            }
            match event["event"].as_str() {
                Some("app_active") => {
                    let want: HashSet<&str> = common
                        .into_iter()
                        .chain(["os_version", "sync", "ai", "notes"])
                        .collect();
                    let os = props["os_version"].as_str().ok_or("os_version")?;
                    if keys != want
                        || !((1..=32).contains(&os.len())
                            && os
                                .bytes()
                                .all(|b| b.is_ascii_alphanumeric() || b".-_".contains(&b)))
                        || !one_of(&props["sync"], &["off", "kanso", "self_hosted"])
                        || !props["ai"].is_boolean()
                        || !one_of(&props["notes"], &["0", "1-9", "10-99", "100-999", "1000+"])
                    {
                        return Err("app_active props".into());
                    }
                }
                Some("feature_used") => {
                    let features = [
                        "note_created",
                        "voice_input",
                        "publish",
                        "import",
                        "lock",
                        "ai",
                        "quick_capture",
                        "shared_note",
                        "export",
                    ];
                    let sources = [
                        "markdown",
                        "obsidian",
                        "logseq",
                        "notion",
                        "evernote",
                        "bear",
                        "joplin",
                        "standard_notes",
                        "simplenote",
                        "upnote",
                        "apple_notes",
                        "pouch",
                        "other",
                    ];
                    if !one_of(&props["feature"], &features) {
                        return Err("feature".into());
                    }
                    let import = props["feature"] == "import";
                    let mut want: HashSet<&str> = common.into_iter().chain(["feature"]).collect();
                    if import {
                        want.insert("source");
                        if !one_of(&props["source"], &sources) {
                            return Err("source".into());
                        }
                    }
                    if keys != want {
                        return Err("feature_used props".into());
                    }
                }
                _ => return Err("event".into()),
            }
        }
        Ok(())
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Over any run of days, uses, sends, answers and restarts: at most
        /// one `app_active` per ID per day and one `feature_used` per
        /// feature per day is ever accepted; every event is for the day it
        /// was sent on; a waiting feature is never sent on a later day; and
        /// every batch is one the Worker takes (bar an `unknown` version).
        #[test]
        fn prop_counts_once_a_day_and_never_late(
            start in 19_000i64..30_000,
            ops in prop::collection::vec(op(), 1..80),
            device in device_strategy(),
            active in active_strategy(),
        ) {
            let mut ledger = Ledger::default();
            let ids = [ledger.install_id(), account(), Id::parse(&format!("a_{}", "2".repeat(32))).unwrap()];
            let mut today = Day::from_unix_ms(start * 86_400_000);
            let mut accepted_active: HashMap<(Id, Day), usize> = HashMap::new();
            let mut accepted_feature: HashMap<(&'static str, Day), usize> = HashMap::new();
            // When each waiting feature name was last used.
            let mut used_on: HashMap<&'static str, Vec<Day>> = HashMap::new();
            for op in ops {
                match op {
                    Op::Clock(days) => {
                        today = if days < 0 {
                            today.previous()
                        } else {
                            (0..days).fold(today, |d, _| d.next())
                        };
                    }
                    Op::Use(feature) => {
                        ledger.record(today, feature);
                        used_on.entry(feature.name()).or_default().push(today);
                    }
                    Op::Send(which, outcome) => {
                        let id = &ids[which];
                        let Some(batch) = ledger.due(today, id, &device, &active) else { continue };
                        prop_assert!(!batch.events.is_empty() && batch.events.len() <= MAX_EVENTS);
                        let body = batch.to_json();
                        prop_assert!(body.len() <= MAX_BYTES);
                        if device.app_version() != crate::values::UNKNOWN {
                            prop_assert_eq!(check_like_the_worker(&body), Ok(()));
                        }
                        for event in &batch.events {
                            prop_assert_eq!(event.day, today);
                            if let EventKind::FeatureUsed(f) = &event.kind {
                                // It was used today, not only on an earlier day.
                                prop_assert!(used_on[f.name()].contains(&today));
                            }
                        }
                        ledger.settle(&batch, outcome);
                        if outcome == Outcome::Accepted {
                            for event in &batch.events {
                                match &event.kind {
                                    EventKind::AppActive(_) => *accepted_active.entry((id.clone(), today)).or_default() += 1,
                                    EventKind::FeatureUsed(f) => *accepted_feature.entry((f.name(), today)).or_default() += 1,
                                }
                            }
                        }
                    }
                    Op::Restart => {
                        let saved = serde_json::to_string(&ledger).unwrap();
                        ledger = serde_json::from_str(&saved).unwrap();
                    }
                }
            }
            prop_assert!(accepted_active.values().all(|&n| n == 1));
            prop_assert!(accepted_feature.values().all(|&n| n == 1));
            prop_assert_eq!(&ledger.install_id(), &ids[0]);
        }

        /// Any JSON at all reads as a ledger, and a ledger reads back as
        /// itself.
        #[test]
        fn prop_any_settings_read(json in any_json(), uses in prop::collection::vec(feature(), 0..12), at in 0i64..40_000) {
            let ledger: Ledger = serde_json::from_value(json).unwrap();
            let back: Ledger = serde_json::from_str(&serde_json::to_string(&ledger).unwrap()).unwrap();
            prop_assert_eq!(&back, &ledger);
            let mut ledger = ledger;
            let today = Day::from_unix_ms(at * 86_400_000);
            for feature in uses {
                ledger.record(today, feature);
            }
            let back: Ledger = serde_json::from_str(&serde_json::to_string(&ledger).unwrap()).unwrap();
            prop_assert_eq!(back, ledger);
        }
    }

    fn any_json() -> impl Strategy<Value = serde_json::Value> {
        let leaf = prop_oneof![
            Just(serde_json::Value::Null),
            any::<bool>().prop_map(serde_json::Value::from),
            any::<i32>().prop_map(serde_json::Value::from),
            "lock|ai|import:bear|import:x|2026-10-30|i_[0-9a-f]{32}|a_[0-9a-f]{32}|\\PC{0,8}"
                .prop_map(serde_json::Value::from),
        ];
        leaf.prop_recursive(3, 24, 6, |inner| {
            prop_oneof![
                prop::collection::vec(inner.clone(), 0..6).prop_map(serde_json::Value::from),
                prop::collection::hash_map(
                    "installId|day|activeSent|featuresSent|pending|[a-z]{1,4}",
                    inner,
                    0..6
                )
                .prop_map(|m| serde_json::Value::Object(m.into_iter().collect())),
            ]
        })
    }
}
