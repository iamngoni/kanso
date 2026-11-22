//! The ledger's rules (docs/METRICS.md › Events, Sending): what is due, and
//! what a send's answer settles. The caller keeps the [`Ledger`] (the desktop
//! in its config as `metrics`, the phones in the library's metadata) and
//! passes the day in, so every function here is pure. A batch is built by
//! [`Ledger::due`] and only [`Ledger::settle`] removes what it carried, so a
//! restart in between sends it again and a restart after never does.

use serde::Deserialize;

pub use crate::ledger_model::{Counted, Ended, Ledger, Opened, SessionsCounted};

use crate::batch::{Batch, Event, EventKind, MAX_EVENTS};
use crate::ranges::SessionLength;
use crate::send::Outcome;
use crate::values::{Active, Day, Device, Feature, Id, install_id};

/// The most `app_opened` a day counts per ID.
pub const MAX_OPENED_PER_DAY: u8 = 12;
/// The most `session` a day counts per install. Phones end a session each
/// time the app stays in the background past its grace period, so a busy
/// day could still flood the counts; twelve matches `app_opened`, which a
/// real sitting usually comes with.
pub const MAX_SESSIONS_PER_DAY: u8 = 12;
/// Sessions kept while they can't be sent.
const MAX_SESSIONS: usize = 8;
/// Count entries kept while they can't be sent.
const MAX_COUNTS: usize = 64;
/// A count stops growing here.
const COUNT_CAP: u32 = 1_000_000;

/// Reads whatever is there and keeps only what is valid: a damaged entry
/// costs a count, never the settings file it sits in. The first version's
/// keys (`featuresSent`, `pending`) are ignored.
impl<'de> Deserialize<'de> for Ledger {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Ledger, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        let text = |v: &serde_json::Value, key: &str| {
            v.get(key).and_then(|v| v.as_str()).map(str::to_owned)
        };
        let items = |key: &str| -> Vec<serde_json::Value> {
            value
                .get(key)
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default()
        };
        let number = |v: &serde_json::Value, key: &str| {
            v.get(key)
                .and_then(serde_json::Value::as_u64)
                .map(|n| u32::try_from(n).unwrap_or(u32::MAX).min(COUNT_CAP))
        };
        let mut ledger = Ledger {
            install_id: text(&value, "installId")
                .and_then(|id| Id::parse(&id))
                .filter(|id| id.as_str().starts_with("i_")),
            day: text(&value, "day").and_then(|d| Day::parse(&d)),
            launch_pending: value.get("launchPending").and_then(|v| v.as_bool()) == Some(true),
            sync_errors: number(&value, "syncErrors").unwrap_or(0),
            sessions_counted: value.get("sessionsCounted").and_then(|item| {
                let day = text(item, "day").and_then(|d| Day::parse(&d))?;
                let n = number(item, "n")?;
                Some(SessionsCounted {
                    day,
                    n: u8::try_from(n).unwrap_or(u8::MAX).min(MAX_SESSIONS_PER_DAY),
                })
            }),
            ..Ledger::default()
        };
        for id in items("activeSent") {
            if let Some(id) = id.as_str().and_then(Id::parse)
                && !ledger.active_sent.contains(&id)
            {
                ledger.active_sent.push(id);
            }
        }
        for item in items("openedSent") {
            let id = text(&item, "id").and_then(|id| Id::parse(&id));
            let n = number(&item, "n").map(|n| u8::try_from(n).unwrap_or(u8::MAX));
            if let (Some(id), Some(n)) = (id, n)
                && !ledger.opened_sent.iter().any(|o| o.id == id)
            {
                ledger.opened_sent.push(Opened { id, n });
            }
        }
        for item in items("counts") {
            let day = text(&item, "day").and_then(|d| Day::parse(&d));
            let feature = text(&item, "feature").and_then(|k| Feature::from_key(&k));
            if let (Some(day), Some(feature), Some(n)) = (day, feature, number(&item, "n"))
                && n > 0
            {
                ledger.add_count(day, &feature.key(), n);
            }
        }
        for item in items("sessions") {
            let day = text(&item, "day").and_then(|d| Day::parse(&d));
            let length = text(&item, "length")
                .and_then(|l| SessionLength::ALL.iter().copied().find(|s| s.as_str() == l));
            if let (Some(day), Some(length)) = (day, length)
                && ledger.sessions.len() < MAX_SESSIONS
            {
                ledger.sessions.push(Ended { day, length });
            }
        }
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

    /// The app was launched: one `app_opened` waits. Says whether the ledger
    /// changed.
    pub fn launch(&mut self) -> bool {
        !std::mem::replace(&mut self.launch_pending, true)
    }

    /// Notes one use of `feature` on `today`. It is reported with the next
    /// batch, as a count. Always changes the ledger.
    pub fn record(&mut self, today: Day, feature: Feature) -> bool {
        self.record_n(today, feature, 1)
    }

    /// Notes `n` uses of `feature` on `today` at once (a batch of notes
    /// changed together). Says whether the ledger changed: not for none.
    pub fn record_n(&mut self, today: Day, feature: Feature, n: u32) -> bool {
        if n == 0 {
            return false;
        }
        self.prune(today);
        self.add_count(today, &feature.key(), n);
        true
    }

    /// A session ended on `today` after `secs` seconds. Says whether it was
    /// kept: not past [`MAX_SESSIONS_PER_DAY`], nor for a day before the
    /// latest one counted (the clock went back; it would only reopen a day
    /// whose cap was already spent).
    pub fn end_session(&mut self, today: Day, secs: u64) -> bool {
        let counted = match self.sessions_counted {
            Some(c) if c.day > today => return false,
            Some(c) if c.day == today => c.n,
            _ => 0,
        };
        if counted >= MAX_SESSIONS_PER_DAY {
            return false;
        }
        self.sessions_counted = Some(SessionsCounted {
            day: today,
            n: counted + 1,
        });
        self.prune(today);
        if self.sessions.len() >= MAX_SESSIONS {
            self.sessions.remove(0);
        }
        self.sessions.push(Ended {
            day: today,
            length: SessionLength::from_secs(secs),
        });
        true
    }

    /// A sync failed.
    pub fn record_sync_error(&mut self) -> bool {
        let before = self.sync_errors;
        self.sync_errors = (self.sync_errors + 1).min(COUNT_CAP);
        self.sync_errors != before
    }

    fn add_count(&mut self, day: Day, key: &str, n: u32) {
        if let Some(entry) = self
            .counts
            .iter_mut()
            .find(|c| c.day == day && c.feature == key)
        {
            entry.n = (entry.n + n).min(COUNT_CAP);
        } else {
            if self.counts.len() >= MAX_COUNTS {
                self.counts.remove(0);
            }
            self.counts.push(Counted {
                day,
                feature: key.to_owned(),
                n: n.min(COUNT_CAP),
            });
        }
    }

    /// Drops what is more than a day old: it would be refused as stale.
    fn prune(&mut self, today: Day) {
        let oldest = today.previous();
        self.counts.retain(|c| c.day >= oldest);
        self.sessions.retain(|s| s.day >= oldest);
    }

    /// Moves the lists that are per day to `today`, emptying them when the
    /// day changed. False for a day before the one held.
    fn roll(&mut self, today: Day) -> bool {
        match self.day {
            Some(day) if day > today => false,
            Some(day) if day == today => true,
            _ => {
                self.day = Some(today);
                self.active_sent.clear();
                self.opened_sent.clear();
                true
            }
        }
    }

    fn opened(&self, id: &Id) -> u8 {
        self.opened_sent
            .iter()
            .find(|o| &o.id == id)
            .map_or(0, |o| o.n)
    }

    /// What to send `today` as `id`: `app_active` unless it is done for this
    /// ID today, `app_opened` for a launch not yet reported (at most
    /// [`MAX_OPENED_PER_DAY`] a day), the sessions that ended, and each
    /// feature's count since the last batch. `None` when there is nothing.
    pub fn due(&self, today: Day, id: &Id, device: &Device, active: &Active) -> Option<Batch> {
        let fresh = match self.day {
            Some(day) if day > today => return None,
            Some(day) => day == today,
            None => false,
        };
        let (active_done, opened) = if fresh {
            (self.active_sent.contains(id), self.opened(id))
        } else {
            (false, 0)
        };
        let oldest = today.previous();
        let current = |day: Day| day >= oldest && day <= today;
        let mut events = Vec::new();
        if !active_done {
            let mut active = active.clone();
            active.sync_errors = self.sync_errors;
            events.push(Event {
                day: today,
                kind: EventKind::AppActive(active),
            });
        }
        if self.launch_pending && opened < MAX_OPENED_PER_DAY {
            events.push(Event {
                day: today,
                kind: EventKind::AppOpened,
            });
        }
        events.extend(
            self.sessions
                .iter()
                .filter(|s| current(s.day))
                .map(|s| Event {
                    day: s.day,
                    kind: EventKind::Session(s.length),
                }),
        );
        events.extend(
            self.counts
                .iter()
                .filter(|c| current(c.day))
                .filter_map(|c| {
                    Feature::from_key(&c.feature).map(|feature| Event {
                        day: c.day,
                        kind: EventKind::FeatureUsed(feature, c.n),
                    })
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
    /// (it would only fail again): what it carried is done. Kept for a later
    /// try otherwise. Says whether the ledger changed.
    pub fn settle(&mut self, batch: &Batch, outcome: Outcome) -> bool {
        if !matches!(outcome, Outcome::Accepted | Outcome::Rejected) {
            return false;
        }
        let before = self.clone();
        for event in &batch.events {
            match &event.kind {
                EventKind::AppActive(active) => {
                    if self.roll(event.day) && !self.active_sent.contains(&batch.id) {
                        self.active_sent.push(batch.id.clone());
                    }
                    self.sync_errors = self.sync_errors.saturating_sub(active.sync_errors);
                }
                EventKind::AppOpened => {
                    if self.roll(event.day) {
                        match self.opened_sent.iter_mut().find(|o| o.id == batch.id) {
                            Some(o) => o.n = o.n.saturating_add(1),
                            None => self.opened_sent.push(Opened {
                                id: batch.id.clone(),
                                n: 1,
                            }),
                        }
                    }
                    self.launch_pending = false;
                }
                EventKind::Session(length) => {
                    if let Some(at) = self
                        .sessions
                        .iter()
                        .position(|s| s.day == event.day && s.length == *length)
                    {
                        self.sessions.remove(at);
                    }
                }
                EventKind::FeatureUsed(feature, sent) => {
                    let key = feature.key();
                    if let Some(at) = self
                        .counts
                        .iter()
                        .position(|c| c.day == event.day && c.feature == key)
                    {
                        let left = self.counts[at].n.saturating_sub(*sent);
                        if left == 0 {
                            self.counts.remove(at);
                        } else {
                            self.counts[at].n = left;
                        }
                    }
                }
            }
        }
        // A launch that can't be counted any more (12 a day) is not kept.
        if self.launch_pending && self.opened(&batch.id) >= MAX_OPENED_PER_DAY {
            self.launch_pending = false;
        }
        *self != before
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::batch::MAX_BYTES;
    use crate::ranges::*;
    use crate::values::{ActiveExtras, Form, ImportSource, Platform, SyncMode};
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

    fn names(batch: &Batch) -> Vec<String> {
        batch
            .events
            .iter()
            .map(|e| match &e.kind {
                EventKind::AppActive(_) => "app_active".into(),
                EventKind::AppOpened => "app_opened".into(),
                EventKind::Session(_) => "session".into(),
                EventKind::FeatureUsed(f, _) => f.name().into(),
            })
            .collect()
    }

    #[test]
    fn a_day_of_use() {
        let mut ledger = Ledger::default();
        let id = ledger.install_id();
        assert_eq!(ledger.install_id(), id, "made once");
        let today = day("2026-10-30");

        // Launch: app_active and app_opened.
        ledger.launch();
        let batch = ledger.due(today, &id, &device(), &active()).unwrap();
        assert_eq!(names(&batch), ["app_active", "app_opened"]);
        // No answer: kept, and the next try has it again.
        assert!(!ledger.settle(&batch, Outcome::Retry));
        assert_eq!(
            ledger.due(today, &id, &device(), &active()),
            Some(batch.clone())
        );
        assert!(ledger.settle(&batch, Outcome::Accepted));
        assert_eq!(ledger.due(today, &id, &device(), &active()), None);

        // An import twice, three locks: counted per feature, with the source.
        ledger.record(today, Feature::Import(ImportSource::Bear));
        ledger.record(today, Feature::Import(ImportSource::Bear));
        for _ in 0..3 {
            ledger.record(today, Feature::Lock);
        }
        let batch = ledger.due(today, &id, &device(), &active()).unwrap();
        assert_eq!(names(&batch), ["import", "lock"]);
        assert_eq!(
            batch.events[1].kind,
            EventKind::FeatureUsed(Feature::Lock, 3)
        );
        // Used again while it is being sent: the later use stays.
        ledger.record(today, Feature::Lock);
        ledger.settle(&batch, Outcome::Accepted);
        let batch = ledger.due(today, &id, &device(), &active()).unwrap();
        assert_eq!(
            batch.events[0].kind,
            EventKind::FeatureUsed(Feature::Lock, 1)
        );
        ledger.settle(&batch, Outcome::Accepted);
        assert_eq!(ledger.due(today, &id, &device(), &active()), None);

        // Quitting sends the session.
        ledger.end_session(today, 90);
        let batch = ledger.due(today, &id, &device(), &active()).unwrap();
        assert_eq!(
            batch.events[0].kind,
            EventKind::Session(SessionLength::OneToFiveMinutes)
        );
        ledger.settle(&batch, Outcome::Accepted);
        assert_eq!(ledger.due(today, &id, &device(), &active()), None);

        // Signing in counts the account's app_active too that day.
        let batch = ledger.due(today, &account(), &device(), &active()).unwrap();
        assert_eq!(names(&batch), ["app_active"]);

        // A refused batch is dropped like a sent one.
        ledger.record(today, Feature::Export);
        let batch = ledger.due(today, &id, &device(), &active()).unwrap();
        ledger.settle(&batch, Outcome::Rejected);
        assert_eq!(ledger.due(today, &id, &device(), &active()), None);

        // The next day: app_active again; what is a day old still goes with
        // its own day, what is older is dropped.
        ledger.record(today, Feature::Ai);
        let tomorrow = today.next();
        let batch = ledger.due(tomorrow, &id, &device(), &active()).unwrap();
        assert_eq!(names(&batch), ["app_active", "ai"]);
        assert_eq!(batch.events[1].day, today);
        let later = tomorrow.next();
        let batch = ledger.due(later, &id, &device(), &active()).unwrap();
        assert_eq!(names(&batch), ["app_active"]);
        // And the clock going back sends nothing for the earlier day.
        ledger.settle(&batch, Outcome::Accepted);
        assert_eq!(ledger.due(today, &id, &device(), &active()), None);
        assert_eq!(ledger.install_id(), id, "the install ID outlives the days");
    }

    #[test]
    fn opens_at_most_twelve_a_day_and_errors_ride_with_app_active() {
        let mut ledger = Ledger::default();
        let id = ledger.install_id();
        let today = day("2026-10-30");
        for _ in 0..15 {
            ledger.launch();
            if let Some(batch) = ledger.due(today, &id, &device(), &active()) {
                ledger.settle(&batch, Outcome::Accepted);
            }
        }
        assert_eq!(ledger.opened(&id), MAX_OPENED_PER_DAY);
        ledger.launch();
        assert_eq!(
            ledger.due(today, &id, &device(), &active()),
            None,
            "a thirteenth launch is not sent"
        );

        let mut ledger = Ledger::default();
        for _ in 0..7 {
            ledger.record_sync_error();
        }
        let batch = ledger.due(today, &id, &device(), &active()).unwrap();
        let json = String::from_utf8(batch.to_json()).unwrap();
        assert!(json.contains(r#""sync_errors":"6+""#), "{json}");
        ledger.record_sync_error();
        ledger.settle(&batch, Outcome::Accepted);
        assert_eq!(ledger.sync_errors, 1, "the later error stays");
    }

    #[test]
    fn reads_damaged_settings() {
        let json = serde_json::json!({
            "installId": "i_00112233445566778899aabbccddeeff",
            "day": "2026-10-30",
            "activeSent": ["a_00112233445566778899aabbccddeeff", "nope", 5],
            "openedSent": [{"id": "a_00112233445566778899aabbccddeeff", "n": 3}, {"n": 1}],
            "counts": [{"day": "2026-10-30", "feature": "ai", "n": 4},
                       {"day": "2026-10-30", "feature": "spying", "n": 1},
                       {"day": "x", "feature": "ai", "n": 1}],
            "sessions": [{"day": "2026-10-30", "length": "1-5m"}, {"day": "2026-10-30", "length": "forever"}],
            "pending": ["ai"],
            "extra": true
        });
        let ledger: Ledger = serde_json::from_value(json).unwrap();
        assert_eq!(ledger.active_sent.len(), 1);
        assert_eq!(ledger.opened_sent.len(), 1);
        assert_eq!(ledger.counts.len(), 1);
        assert_eq!(ledger.sessions.len(), 1);
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
        Launch,
        EndSession(u64),
        SyncError,
        /// Builds the batch for one of the IDs and settles it.
        Send(usize, Outcome),
        /// Saves and loads the ledger, as a restart does.
        Restart,
    }

    fn feature() -> impl Strategy<Value = Feature> {
        (0usize..Feature::all().count()).prop_map(|ix| Feature::all().nth(ix).unwrap())
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
            4 => feature().prop_map(Op::Use),
            2 => Just(Op::Launch),
            2 => (0u64..20_000).prop_map(Op::EndSession),
            1 => Just(Op::SyncError),
            4 => (0usize..3, outcome()).prop_map(|(id, o)| Op::Send(id, o)),
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
            (
                prop::option::of("\\PC{0,8}"),
                prop::option::of(prop::sample::select(ThemeMode::ALL.to_vec())),
                prop::option::of(("\\PC{0,50}", any::<bool>())),
                prop::option::of(prop::sample::select(EditorMode::ALL.to_vec())),
                prop::option::of(any::<u64>()),
                prop::option::of(any::<u64>()),
                prop::option::of(any::<u64>()),
                prop::option::of(prop::sample::select(Channel::ALL.to_vec())),
            ),
        )
            .prop_map(|(os, sync, ai, notes, x)| {
                Active::new(&os, sync, ai, notes).with_extras(ActiveExtras {
                    locale_lang: x.0.map(|l| LocaleLang::new(&l, &["en", "fr", "de"])),
                    theme_mode: x.1,
                    theme: x.2.map(|(n, b)| Theme::new(&n, b)),
                    editor_mode: x.3,
                    plugins: x.4.map(PluginsRange::from_count),
                    notebooks: x.5.map(NotebooksRange::from_count),
                    attachments_mb: x.6.map(AttachmentsMb::from_bytes),
                    channel: x.7,
                })
            })
    }

    /// What the Worker checks (docs/METRICS.md › The batch), written from
    /// the page rather than from this crate's types.
    fn check_like_the_worker(body: &[u8]) -> Result<(), String> {
        let digits = |s: &str, lo: usize, hi: usize| {
            (lo..=hi).contains(&s.len()) && s.bytes().all(|b| b.is_ascii_digit())
        };
        let one_of =
            |v: &serde_json::Value, list: &[&str]| v.as_str().is_some_and(|s| list.contains(&s));
        let slug = |s: &str| {
            (1..=32).contains(&s.len())
                && s.bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        };
        if body.len() > MAX_BYTES {
            return Err("over 16 KiB".into());
        }
        let batch: serde_json::Value = serde_json::from_slice(body).map_err(|e| e.to_string())?;
        let top = batch.as_object().ok_or("not an object")?;
        let keys: HashSet<&str> = top.keys().map(String::as_str).collect();
        if keys != HashSet::from(["v", "id", "events"]) || batch["v"] != 1 {
            return Err("top level".into());
        }
        if Id::parse(batch["id"].as_str().ok_or("id")?).is_none() {
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
            let must: HashSet<&str> = common.into_iter().collect();
            match event["event"].as_str() {
                Some("app_active") => {
                    let required = ["os_version", "sync", "ai", "notes"];
                    let optional = [
                        "locale_lang",
                        "theme_mode",
                        "theme",
                        "editor_mode",
                        "plugins",
                        "notebooks",
                        "attachments_mb",
                        "sync_errors",
                        "channel",
                    ];
                    let must: HashSet<&str> = must.into_iter().chain(required).collect();
                    let may: HashSet<&str> = must.iter().copied().chain(optional).collect();
                    if !must.is_subset(&keys) || !keys.is_subset(&may) {
                        return Err("app_active props".into());
                    }
                    let os = props["os_version"].as_str().ok_or("os_version")?;
                    let ok = (1..=32).contains(&os.len())
                        && os
                            .bytes()
                            .all(|b| b.is_ascii_alphanumeric() || b".-_".contains(&b))
                        && one_of(&props["sync"], &["off", "kanso", "self_hosted"])
                        && props["ai"].is_boolean()
                        && one_of(&props["notes"], &["0", "1-9", "10-99", "100-999", "1000+"]);
                    let list = |key: &str, values: &[&str]| {
                        props.get(key).is_none_or(|v| one_of(v, values))
                    };
                    let lang = props.get("locale_lang").is_none_or(|v| {
                        v.as_str().is_some_and(|s| {
                            s == "other"
                                || (s.len() == 2 && s.bytes().all(|b| b.is_ascii_lowercase()))
                        })
                    });
                    let theme = props
                        .get("theme")
                        .is_none_or(|v| v.as_str().is_some_and(|s| s == "custom" || slug(s)));
                    if !(ok
                        && lang
                        && theme
                        && list("theme_mode", &["light", "dark", "system"])
                        && list("editor_mode", &["edit", "live", "preview", "split"])
                        && list("plugins", &["0", "1-2", "3-5", "6+"])
                        && list("notebooks", &["0", "1-4", "5-19", "20+"])
                        && list(
                            "attachments_mb",
                            &["0", "<10", "10-100", "100-1000", "1000+"],
                        )
                        && list("sync_errors", &["0", "1-5", "6+"])
                        && list("channel", &["stable", "beta", "dev", "nightly"]))
                    {
                        return Err("app_active values".into());
                    }
                }
                Some("app_opened") => {
                    if keys != must {
                        return Err("app_opened props".into());
                    }
                }
                Some("session") => {
                    let want: HashSet<&str> = must.into_iter().chain(["duration"]).collect();
                    if keys != want
                        || !one_of(
                            &props["duration"],
                            &["<1m", "1-5m", "5-15m", "15-60m", "1-3h", "3h+"],
                        )
                    {
                        return Err("session props".into());
                    }
                }
                Some("feature_used") => {
                    let features = crate::values::DOCUMENTED_FEATURES;
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
                    if !one_of(&props["feature"], features) {
                        return Err("feature".into());
                    }
                    let import = props["feature"] == "import";
                    let mut want: HashSet<&str> =
                        must.into_iter().chain(["feature", "count"]).collect();
                    if import {
                        want.insert("source");
                        if !one_of(&props["source"], &sources) {
                            return Err("source".into());
                        }
                    }
                    if keys != want || !one_of(&props["count"], &["1", "2-5", "6-20", "21+"]) {
                        return Err("feature_used props".into());
                    }
                }
                _ => return Err("event".into()),
            }
        }
        Ok(())
    }

    proptest! {
        #![proptest_config(ProptestConfig { rng_seed: proptest::test_runner::RngSeed::Fixed(0x006b_616e_736f), ..ProptestConfig::with_cases(256) })]

        /// Over any run of days, uses, launches, sessions, sends, answers and
        /// restarts: nothing recorded is accepted more than it was recorded
        /// (no double counting, restarts included); at most one `app_active`
        /// per ID per day and twelve `app_opened`; every event is for a day
        /// no later than today and at most a day old; and every batch is one
        /// the Worker takes (bar an `unknown` version), with every value in
        /// the documented sets.
        #[test]
        fn prop_never_counts_twice(
            start in 19_000i64..30_000,
            ops in prop::collection::vec(op(), 1..100),
            device in device_strategy(),
            active in active_strategy(),
        ) {
            let mut ledger = Ledger::default();
            let ids = [ledger.install_id(), account(), Id::parse(&format!("a_{}", "2".repeat(32))).unwrap()];
            let mut today = Day::from_unix_ms(start * 86_400_000);
            let mut accepted_active: HashMap<(Id, Day), usize> = HashMap::new();
            let mut accepted_opened: HashMap<(Id, Day), usize> = HashMap::new();
            let mut recorded: HashMap<String, u64> = HashMap::new();
            let mut accepted: HashMap<String, u64> = HashMap::new();
            let mut sessions = (0usize, 0usize);
            let mut sessions_per_day: HashMap<Day, usize> = HashMap::new();
            let mut errors = (0u64, 0u64);
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
                        *recorded.entry(feature.key()).or_default() += 1;
                    }
                    Op::Launch => {
                        ledger.launch();
                    }
                    Op::EndSession(secs) => {
                        ledger.end_session(today, secs);
                        sessions.0 += 1;
                    }
                    Op::SyncError => {
                        ledger.record_sync_error();
                        errors.0 += 1;
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
                            prop_assert!(event.day <= today && event.day >= today.previous());
                        }
                        ledger.settle(&batch, outcome);
                        if outcome == Outcome::Accepted {
                            for event in &batch.events {
                                match &event.kind {
                                    EventKind::AppActive(a) => {
                                        *accepted_active.entry((id.clone(), today)).or_default() += 1;
                                        errors.1 += u64::from(a.sync_errors);
                                    }
                                    EventKind::AppOpened => {
                                        *accepted_opened.entry((id.clone(), today)).or_default() += 1;
                                    }
                                    EventKind::Session(_) => {
                                        sessions.1 += 1;
                                        *sessions_per_day.entry(event.day).or_default() += 1;
                                    }
                                    EventKind::FeatureUsed(f, n) => {
                                        *accepted.entry(f.key()).or_default() += u64::from(*n);
                                    }
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
            prop_assert!(accepted_opened.values().all(|&n| n <= usize::from(MAX_OPENED_PER_DAY)));
            prop_assert!(sessions.1 <= sessions.0);
            prop_assert!(sessions_per_day.values().all(|&n| n <= usize::from(MAX_SESSIONS_PER_DAY)));
            // Three IDs can each report the same errors once.
            prop_assert!(errors.1 <= errors.0 * 3);
            for (key, n) in &accepted {
                prop_assert!(*n <= recorded.get(key).copied().unwrap_or(0), "{key}");
            }
            prop_assert_eq!(&ledger.install_id(), &ids[0]);
        }

        /// However many sessions end in a day, with sends and restarts in
        /// between, at most twelve are kept for it; the next day counts
        /// afresh, and a day the clock went back to counts none.
        #[test]
        fn prop_sessions_are_capped_per_day(
            ends in prop::collection::vec((0u64..20_000, any::<bool>(), any::<bool>()), 1..80),
        ) {
            let today = day("2026-10-30");
            let mut ledger = Ledger::default();
            let id = ledger.install_id();
            let mut kept = 0usize;
            for (secs, send, restart) in ends {
                if ledger.end_session(today, secs) {
                    kept += 1;
                }
                if send && let Some(batch) = ledger.due(today, &id, &device(), &active()) {
                    ledger.settle(&batch, Outcome::Accepted);
                }
                if restart {
                    let saved = serde_json::to_string(&ledger).unwrap();
                    ledger = serde_json::from_str(&saved).unwrap();
                }
            }
            prop_assert!(kept <= usize::from(MAX_SESSIONS_PER_DAY));
            prop_assert_eq!(kept == usize::from(MAX_SESSIONS_PER_DAY), !ledger.end_session(today, 1));
            prop_assert!(!ledger.end_session(today.previous(), 1));
            prop_assert!(ledger.end_session(today.next(), 1));
        }

        /// Restarting after every answer never sends what was accepted again.
        #[test]
        fn prop_accepted_is_never_resent_after_a_restart(
            uses in prop::collection::vec(feature(), 1..20),
            secs in 0u64..20_000,
        ) {
            let today = day("2026-10-30");
            let mut ledger = Ledger::default();
            let id = ledger.install_id();
            ledger.launch();
            ledger.end_session(today, secs);
            for f in uses { ledger.record(today, f); }
            while let Some(batch) = ledger.due(today, &id, &device(), &active()) {
                ledger.settle(&batch, Outcome::Accepted);
                let saved = serde_json::to_string(&ledger).unwrap();
                ledger = serde_json::from_str(&saved).unwrap();
            }
            prop_assert_eq!(ledger.due(today, &id, &device(), &active()), None);
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
            "lock|ai|import:bear|import:x|1-5m|2026-10-30|i_[0-9a-f]{32}|a_[0-9a-f]{32}|\\PC{0,8}"
                .prop_map(serde_json::Value::from),
        ];
        leaf.prop_recursive(3, 24, 6, |inner| {
            prop_oneof![
                prop::collection::vec(inner.clone(), 0..6).prop_map(serde_json::Value::from),
                prop::collection::hash_map(
                    "installId|day|activeSent|openedSent|launchPending|counts|sessions|sessionsCounted|syncErrors|id|n|feature|length|[a-z]{1,4}",
                    inner,
                    0..6
                )
                .prop_map(|m| serde_json::Value::Object(m.into_iter().collect())),
            ]
        })
    }
}
