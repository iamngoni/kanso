//! When the morning brief and the weekly review notify, and what the
//! notification says. Pure: the apps pass the time and what they have
//! already shown, and show the notifications themselves.
//!
//! A notification fires once a day per device, at the time chosen in
//! Settings, and only if it is still within [`GRACE_MS`] of that time
//! (a device that was off at 08:00 shows the brief when it starts at 09:30,
//! not at 14:00). The text is counts or a fixed string, never a note's
//! words, titles or a loop's sentence: lock screens show it.

use kanso_core::dates::{Date, DateTime};
use serde::{Deserialize, Serialize};

use super::Now;

const MINUTE_MS: i64 = 60_000;
const DAY_MS: i64 = 86_400_000;

/// How late a notice may still show: three hours past its time.
pub const GRACE_MS: i64 = 3 * 60 * MINUTE_MS;

/// How many days ahead a phone plans its notifications.
pub const PLAN_DAYS: i64 = 7;

/// The settings of Settings › Tasks › Brief and review.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct BriefSettings {
    /// Find open loops in notes (default on).
    pub loops: bool,
    /// The morning brief's notification.
    pub brief: bool,
    pub brief_hour: u32,
    pub brief_minute: u32,
    /// Only Monday to Friday.
    pub brief_weekdays_only: bool,
    /// The weekly review's notification.
    pub review: bool,
    /// 0 for Monday through 6 for Sunday.
    pub review_day: u32,
    pub review_hour: u32,
    pub review_minute: u32,
    /// Notify for the brief, when the system allows it.
    pub notify_brief: bool,
    /// Notify for the review.
    pub notify_review: bool,
    /// Add a tick after a loop's sentence in the note when it is done.
    pub mark_done_in_note: bool,
}

impl Default for BriefSettings {
    fn default() -> Self {
        BriefSettings {
            loops: true,
            brief: true,
            brief_hour: 8,
            brief_minute: 0,
            brief_weekdays_only: false,
            review: true,
            review_day: 6,
            review_hour: 18,
            review_minute: 0,
            notify_brief: true,
            notify_review: true,
            mark_done_in_note: false,
        }
    }
}

impl BriefSettings {
    /// The same settings with times and days clamped to what exists.
    pub fn sanitized(mut self) -> BriefSettings {
        self.brief_hour = self.brief_hour.min(23);
        self.brief_minute = self.brief_minute.min(59);
        self.review_hour = self.review_hour.min(23);
        self.review_minute = self.review_minute.min(59);
        self.review_day = self.review_day.min(6);
        self
    }
}

/// Which notification.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NoticeKind {
    Brief,
    Review,
}

impl NoticeKind {
    pub const ALL: [NoticeKind; 2] = [NoticeKind::Brief, NoticeKind::Review];

    pub fn as_str(self) -> &'static str {
        match self {
            NoticeKind::Brief => "brief",
            NoticeKind::Review => "review",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            NoticeKind::Brief => "Your morning brief",
            NoticeKind::Review => "Your weekly review",
        }
    }

    /// The text of a notification that can't know the counts (planned
    /// ahead on a phone, or the review).
    pub fn fixed_body(self) -> &'static str {
        match self {
            NoticeKind::Brief => "Your morning brief is ready",
            NoticeKind::Review => "Your weekly review is ready",
        }
    }
}

/// The store meta key recording that `kind` was shown for `date`.
pub fn shown_key(kind: NoticeKind, date: Date) -> String {
    format!("{}.shown.{}", kind.as_str(), date.iso())
}

fn applies(settings: &BriefSettings, kind: NoticeKind, date: Date) -> bool {
    match kind {
        NoticeKind::Brief => {
            settings.brief
                && settings.notify_brief
                && (!settings.brief_weekdays_only || date.weekday() < 5)
        }
        NoticeKind::Review => {
            settings.review && settings.notify_review && date.weekday() as u32 == settings.review_day
        }
    }
}

fn time_of(settings: &BriefSettings, kind: NoticeKind) -> (u32, u32) {
    match kind {
        NoticeKind::Brief => (settings.brief_hour, settings.brief_minute),
        NoticeKind::Review => (settings.review_hour, settings.review_minute),
    }
}

/// When `kind` fires on `date`, in local milliseconds, if it does.
pub fn fire_at(settings: &BriefSettings, kind: NoticeKind, date: Date) -> Option<i64> {
    if !applies(settings, kind, date) {
        return None;
    }
    let (hour, minute) = time_of(settings, kind);
    Some(date.to_days() * DAY_MS + i64::from(hour) * 60 * MINUTE_MS + i64::from(minute) * MINUTE_MS)
}

/// The notifications to show at `now_local_ms`: those whose time today has
/// come, no more than [`GRACE_MS`] ago, that `shown` says weren't shown
/// for today.
pub fn due_now(
    settings: &BriefSettings,
    now_local_ms: i64,
    shown: &dyn Fn(NoticeKind, Date) -> bool,
) -> Vec<NoticeKind> {
    let today = DateTime::from_local_ms(now_local_ms).date;
    NoticeKind::ALL
        .into_iter()
        .filter(|&kind| {
            fire_at(settings, kind, today).is_some_and(|at| {
                at <= now_local_ms && now_local_ms - at <= GRACE_MS && !shown(kind, today)
            })
        })
        .collect()
}

/// The next time after `now_local_ms` either notification fires (local
/// milliseconds), looking a week and a day ahead.
pub fn next_wake(settings: &BriefSettings, now_local_ms: i64) -> Option<i64> {
    let today = DateTime::from_local_ms(now_local_ms).date;
    (0..=PLAN_DAYS)
        .flat_map(|d| {
            let date = today.add_days(d);
            NoticeKind::ALL
                .into_iter()
                .filter_map(move |kind| fire_at(settings, kind, date))
        })
        .filter(|&at| at > now_local_ms)
        .min()
}

/// A notification a phone hands to the system ahead of time.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Planned {
    pub kind: NoticeKind,
    /// The local wall-clock date and time it should show at. The phone
    /// builds the instant from these in its own zone, so a change of
    /// offset (daylight saving) between now and then doesn't move it.
    pub date: Date,
    pub hour: u32,
    pub minute: u32,
    /// The instant with the zone's offset as it is now: a fallback.
    pub utc_ms: i64,
    pub title: &'static str,
    pub body: &'static str,
}

/// The notifications for the next [`PLAN_DAYS`] days (today's included if
/// still ahead), with fixed text: counts can't be known days ahead.
pub fn plan(settings: &BriefSettings, now: Now) -> Vec<Planned> {
    let today = now.today();
    let mut out: Vec<Planned> = Vec::new();
    for d in 0..=PLAN_DAYS {
        let date = today.add_days(d);
        for kind in NoticeKind::ALL {
            let Some(at) = fire_at(settings, kind, date) else {
                continue;
            };
            if at <= now.local_ms() {
                continue;
            }
            let (hour, minute) = time_of(settings, kind);
            out.push(Planned {
                kind,
                date,
                hour,
                minute,
                utc_ms: at - now.offset_ms,
                title: kind.title(),
                body: kind.fixed_body(),
            });
        }
    }
    out.sort_by_key(|p| (p.date, p.hour, p.minute, p.kind as u8));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::collections::HashSet;

    fn date(y: i32, m: u32, d: u32) -> Date {
        Date::new(y, m, d).unwrap()
    }

    fn local(date: Date, hour: u32, minute: u32) -> i64 {
        date.to_days() * DAY_MS + i64::from(hour) * 60 * MINUTE_MS + i64::from(minute) * MINUTE_MS
    }

    #[test]
    fn defaults_are_the_documented_ones() {
        let s = BriefSettings::default();
        assert!(s.loops && s.brief && s.review && s.notify_brief && s.notify_review);
        assert_eq!((s.brief_hour, s.brief_minute), (8, 0));
        assert_eq!((s.review_day, s.review_hour, s.review_minute), (6, 18, 0));
        assert!(!s.mark_done_in_note);
        // Old config files without the keys read as defaults.
        let read: BriefSettings = serde_json::from_str("{}").unwrap();
        assert_eq!(read, s);
    }

    #[test]
    fn weekdays_only_skips_the_weekend_and_the_review_has_its_day() {
        let s = BriefSettings {
            brief_weekdays_only: true,
            ..BriefSettings::default()
        };
        let monday = date(2026, 11, 2);
        assert!(fire_at(&s, NoticeKind::Brief, monday).is_some());
        assert!(fire_at(&s, NoticeKind::Brief, date(2026, 11, 7)).is_none());
        assert!(fire_at(&s, NoticeKind::Review, monday).is_none());
        assert_eq!(
            fire_at(&s, NoticeKind::Review, date(2026, 11, 8)),
            Some(local(date(2026, 11, 8), 18, 0))
        );
    }

    #[test]
    fn fires_once_per_day_across_restarts() {
        let s = BriefSettings::default();
        let day = date(2026, 11, 2);
        let mut shown: HashSet<String> = HashSet::new();
        let mut fired = 0;
        // Look every five minutes all day, "restarting" (forgetting memory
        // but not the recorded keys) every hour.
        let mut at = local(day, 0, 0);
        while at < local(day.add_days(1), 0, 0) {
            let keys = shown.clone();
            let due = due_now(&s, at, &|k, d| keys.contains(&shown_key(k, d)));
            for kind in due {
                fired += 1;
                shown.insert(shown_key(kind, day));
            }
            at += 5 * MINUTE_MS;
        }
        assert_eq!(fired, 1, "the brief fires once, the review isn't on a Monday");
    }

    #[test]
    fn missed_by_more_than_three_hours_is_skipped() {
        let s = BriefSettings::default();
        let day = date(2026, 11, 2);
        let none = |_: NoticeKind, _: Date| false;
        assert!(due_now(&s, local(day, 7, 59), &none).is_empty());
        assert_eq!(due_now(&s, local(day, 8, 0), &none), [NoticeKind::Brief]);
        assert_eq!(due_now(&s, local(day, 11, 0), &none), [NoticeKind::Brief]);
        assert!(due_now(&s, local(day, 11, 1), &none).is_empty());
        // The 09:30 start still shows it.
        assert_eq!(due_now(&s, local(day, 9, 30), &none), [NoticeKind::Brief]);
    }

    #[test]
    fn off_switches_silence_each() {
        let none = |_: NoticeKind, _: Date| false;
        let sunday = date(2026, 11, 8);
        let both = |s: &BriefSettings| due_now(s, local(sunday, 18, 5), &none);
        assert_eq!(both(&BriefSettings::default()), [NoticeKind::Review]);
        let s = BriefSettings {
            notify_review: false,
            ..BriefSettings::default()
        };
        assert!(both(&s).is_empty());
        let s = BriefSettings {
            review: false,
            ..BriefSettings::default()
        };
        assert!(both(&s).is_empty());
    }

    #[test]
    fn next_wake_is_the_next_fire() {
        let s = BriefSettings::default();
        let day = date(2026, 11, 2);
        assert_eq!(next_wake(&s, local(day, 7, 0)), Some(local(day, 8, 0)));
        assert_eq!(
            next_wake(&s, local(day, 9, 0)),
            Some(local(day.add_days(1), 8, 0))
        );
        let off = BriefSettings {
            brief: false,
            review: false,
            ..BriefSettings::default()
        };
        assert_eq!(next_wake(&off, local(day, 7, 0)), None);
    }

    /// Daylight saving moves the offset between now and a planned day; the
    /// plan carries wall-clock times, so every notice is at 08:00 whatever
    /// the zone does.
    #[test]
    fn the_plan_keeps_wall_clock_times_across_a_change_of_offset() {
        let s = BriefSettings {
            review: false,
            ..BriefSettings::default()
        };
        // Saturday 7 Nov 2026, 22:00 in a zone at UTC-4 that falls back to
        // UTC-5 the next night.
        let offset_ms = -4 * 3_600_000;
        let now = Now {
            utc_ms: local(date(2026, 11, 7), 22, 0) - offset_ms,
            offset_ms,
        };
        let planned = plan(&s, now);
        assert_eq!(planned.len(), 7);
        assert!(planned.iter().all(|p| (p.hour, p.minute) == (8, 0)));
        let days: Vec<Date> = planned.iter().map(|p| p.date).collect();
        assert_eq!(days[0], date(2026, 11, 8));
        assert_eq!(days[6], date(2026, 11, 14));
        // The fallback instant uses today's offset.
        assert_eq!(planned[0].utc_ms, local(date(2026, 11, 8), 8, 0) + 4 * 3_600_000);
    }

    #[test]
    fn notification_text_holds_no_note_text() {
        for kind in NoticeKind::ALL {
            assert!(!kind.title().is_empty() && !kind.fixed_body().is_empty());
        }
        assert_eq!(super::super::notice_text(3, 5), "3 due, 5 open loops");
        assert_eq!(super::super::notice_text(0, 1), "1 open loop");
        assert_eq!(super::super::notice_text(0, 0), "Nothing due today");
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// However the settings are and whenever it looks, a scheduler
        /// that records what it shows never shows the same notice twice
        /// for a day, and only shows one within the grace period.
        #[test]
        fn prop_never_fires_twice_in_a_day(
            hour in 0u32..24, minute in 0u32..60, weekdays in any::<bool>(),
            review_day in 0u32..7,
            steps in prop::collection::vec(1i64..400, 1..120),
        ) {
            let s = BriefSettings { brief_hour: hour, brief_minute: minute,
                brief_weekdays_only: weekdays, review_day, ..BriefSettings::default() };
            let mut shown: HashSet<String> = HashSet::new();
            let mut now = local(date(2026, 11, 2), 0, 0);
            for step in steps {
                now += step * MINUTE_MS;
                let keys = shown.clone();
                for kind in due_now(&s, now, &|k, d| keys.contains(&shown_key(k, d))) {
                    let today = DateTime::from_local_ms(now).date;
                    let at = fire_at(&s, kind, today).unwrap();
                    prop_assert!(at <= now && now - at <= GRACE_MS);
                    prop_assert!(shown.insert(shown_key(kind, today)), "twice");
                }
                if let Some(wake) = next_wake(&s, now) {
                    prop_assert!(wake > now);
                }
            }
        }

        /// The plan only holds future notices, in order, within the horizon.
        #[test]
        fn prop_the_plan_is_ahead_and_ordered(
            hour in 0u32..24, minute in 0u32..60, offset_h in -12i64..14, at_min in 0i64..(60 * 24 * 30),
        ) {
            let s = BriefSettings { brief_hour: hour, brief_minute: minute, ..BriefSettings::default() };
            let offset_ms = offset_h * 3_600_000;
            let now = Now { utc_ms: local(date(2026, 11, 1), 0, 0) + at_min * MINUTE_MS - offset_ms, offset_ms };
            let planned = plan(&s, now);
            let mut last = None;
            for p in &planned {
                prop_assert!(p.utc_ms > now.utc_ms);
                prop_assert!(p.date <= now.today().add_days(PLAN_DAYS));
                let key = (p.date, p.hour, p.minute);
                prop_assert!(last.is_none_or(|l| l <= key));
                last = Some(key);
            }
        }
    }
}
