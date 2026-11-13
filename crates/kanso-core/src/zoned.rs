//! Date-time and date-time-with-zone property values: reading and writing
//! their YAML text, and zone math through the tz database.
//!
//! A date-time is `YYYY-MM-DDTHH:MM` (seconds allowed) in local wall-clock
//! time, like a task's due time. A date-time with a zone adds a space and an
//! IANA zone name: `2026-11-18T09:00 America/New_York`. Text that does not
//! fit, such as a zone that does not exist, is not one of these: the
//! property keeps it as text.

use std::str::FromStr;

use chrono::{
    Datelike, Duration, LocalResult, NaiveDate, NaiveDateTime, NaiveTime, Offset, TimeZone,
    Timelike,
};
use chrono_tz::Tz;

use crate::dates::{Date, DateTime};
pub use crate::zoned_model::{Fit, Placed, ZonedTime};

/// Reads `YYYY-MM-DDTHH:MM` or `YYYY-MM-DDTHH:MM:SS`.
pub fn parse_local(text: &str) -> Option<DateTime> {
    let (date, time) = text.split_once('T')?;
    let date = Date::parse_iso(date)?;
    let mut parts = time.split(':');
    let hour = digits(parts.next()?)?;
    let minute = digits(parts.next()?)?;
    let second = match parts.next() {
        Some(second) => digits(second)?,
        None => 0,
    };
    if parts.next().is_some() || hour > 23 || minute > 59 || second > 59 {
        return None;
    }
    Some(DateTime {
        date,
        hour,
        minute,
        second,
    })
}

/// Two ASCII digits as a number.
fn digits(text: &str) -> Option<u32> {
    if text.len() == 2 && text.bytes().all(|b| b.is_ascii_digit()) {
        text.parse().ok()
    } else {
        None
    }
}

/// A local date-time as `YYYY-MM-DDTHH:MM`, with seconds only when not zero.
pub fn local_text(local: &DateTime) -> String {
    let base = format!("{}T{:02}:{:02}", local.date.iso(), local.hour, local.minute);
    if local.second == 0 {
        base
    } else {
        format!("{base}:{:02}", local.second)
    }
}

/// The zone named `name`, when the tz database has it.
pub fn zone(name: &str) -> Option<Tz> {
    if name.is_empty() || name.trim() != name {
        return None;
    }
    Tz::from_str(name).ok()
}

/// Every zone the tz database names, in alphabetical order, for a picker.
pub fn zone_names() -> Vec<&'static str> {
    let mut names: Vec<&'static str> = chrono_tz::TZ_VARIANTS.iter().map(|tz| tz.name()).collect();
    names.sort_unstable();
    names.dedup();
    names
}

/// A zone as a person reads it: the city or region without its area and
/// underscores (`America/New_York` is `New York`). A name with no area
/// (`UTC`) stays as it is.
pub fn zone_label(name: &str) -> String {
    name.rsplit('/').next().unwrap_or(name).replace('_', " ")
}

/// Reads `YYYY-MM-DDTHH:MM Zone/Name`.
pub fn parse_zoned(text: &str) -> Option<ZonedTime> {
    let (local, zone_name) = text.split_once(' ')?;
    zone(zone_name)?;
    Some(ZonedTime {
        local: parse_local(local)?,
        zone: zone_name.to_owned(),
    })
}

impl ZonedTime {
    /// `2026-11-18T09:00 America/New_York`.
    pub fn text(&self) -> String {
        format!("{} {}", local_text(&self.local), self.zone)
    }

    /// Where the wall-clock time falls on the timeline. `None` when the
    /// zone is unknown (a value read from text always has a known one).
    pub fn place(&self) -> Option<Placed> {
        let tz = zone(&self.zone)?;
        let naive = naive(&self.local)?;
        Some(match tz.offset_from_local_datetime(&naive) {
            LocalResult::Single(offset) => Placed {
                utc_ms: ms(naive, offset.fix().local_minus_utc()),
                fit: Fit::Exact,
            },
            // Clocks fall back: the first of the two.
            LocalResult::Ambiguous(first, _) => Placed {
                utc_ms: ms(naive, first.fix().local_minus_utc()),
                fit: Fit::Ambiguous,
            },
            // Clocks spring forward: read the time with the offset from
            // before the jump, which lands it after the gap.
            LocalResult::None => {
                let before = (1..=3)
                    .find_map(|days| {
                        tz.offset_from_local_datetime(&(naive - Duration::days(days)))
                            .earliest()
                    })
                    .map(|offset| offset.fix().local_minus_utc())?;
                Placed {
                    utc_ms: ms(naive, before),
                    fit: Fit::Gap,
                }
            }
        })
    }

    /// The moment as milliseconds since the Unix epoch.
    pub fn utc_ms(&self) -> Option<i64> {
        self.place().map(|placed| placed.utc_ms)
    }

    /// The same moment on the wall clock of `zone_name`. `None` when either
    /// zone is unknown.
    pub fn in_zone(&self, zone_name: &str) -> Option<ZonedTime> {
        from_utc_ms(self.utc_ms()?, zone_name)
    }
}

/// The wall-clock time in `zone_name` at `utc_ms` milliseconds since the
/// Unix epoch.
pub fn from_utc_ms(utc_ms: i64, zone_name: &str) -> Option<ZonedTime> {
    let tz = zone(zone_name)?;
    let utc = chrono::DateTime::from_timestamp_millis(utc_ms)?;
    let local = utc.with_timezone(&tz).naive_local();
    Some(ZonedTime {
        local: from_naive(&local)?,
        zone: zone_name.to_owned(),
    })
}

/// The zone's offset from UTC, in seconds, at `utc_ms`.
pub fn offset_secs(zone_name: &str, utc_ms: i64) -> Option<i32> {
    let tz = zone(zone_name)?;
    let utc = chrono::DateTime::from_timestamp_millis(utc_ms)?;
    Some(
        tz.offset_from_utc_datetime(&utc.naive_utc())
            .fix()
            .local_minus_utc(),
    )
}

fn ms(naive: NaiveDateTime, offset_secs: i32) -> i64 {
    naive.and_utc().timestamp_millis() - i64::from(offset_secs) * 1000
}

fn naive(local: &DateTime) -> Option<NaiveDateTime> {
    Some(NaiveDateTime::new(
        NaiveDate::from_ymd_opt(local.date.year, local.date.month, local.date.day)?,
        NaiveTime::from_hms_opt(local.hour, local.minute, local.second)?,
    ))
}

fn from_naive(naive: &NaiveDateTime) -> Option<DateTime> {
    Some(DateTime {
        date: Date::new(naive.year(), naive.month(), naive.day())?,
        hour: naive.hour(),
        minute: naive.minute(),
        second: naive.second(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    fn at(text: &str) -> ZonedTime {
        parse_zoned(text).unwrap()
    }

    #[test]
    fn reads_and_writes_the_documented_forms() {
        let t = at("2026-11-18T09:00 America/New_York");
        assert_eq!(t.text(), "2026-11-18T09:00 America/New_York");
        // 09:00 EST is 14:00 UTC.
        assert_eq!(t.in_zone("UTC").unwrap().text(), "2026-11-18T14:00 UTC");
        assert_eq!(
            t.in_zone("Africa/Harare").unwrap().text(),
            "2026-11-18T16:00 Africa/Harare"
        );
        assert_eq!(
            local_text(&parse_local("2026-11-21T15:00").unwrap()),
            "2026-11-21T15:00"
        );
        assert_eq!(
            local_text(&parse_local("2026-11-21T15:00:07").unwrap()),
            "2026-11-21T15:00:07"
        );
        for bad in [
            "2026-11-18T25:00",
            "2026-11-18T09:60",
            "2026-02-30T09:00",
            "2026-11-18 09:00",
            "2026-11-18T9:00",
            "2026-11-18T09:00Z",
            "2026-11-18T09:00:00:00",
        ] {
            assert_eq!(parse_local(bad), None, "{bad}");
        }
        for bad in [
            "2026-11-18T09:00 Mars/Olympus",
            "2026-11-18T09:00",
            "2026-11-18T09:00 ",
            "2026-11-18T09:00  UTC",
            "2026-11-18T09:00 America/New_York x",
        ] {
            assert_eq!(parse_zoned(bad), None, "{bad}");
        }
    }

    #[test]
    fn the_same_wall_clock_is_a_different_moment_across_a_dst_change() {
        // New York is UTC-4 until 1 Nov 2026, then UTC-5.
        let before = at("2026-10-31T09:00 America/New_York").utc_ms().unwrap();
        let after = at("2026-11-01T09:00 America/New_York").utc_ms().unwrap();
        // Two wall-clock mornings apart is 25 hours of elapsed time.
        assert_eq!(after - before, 25 * 3_600_000);
        // Europe changes a week earlier (25 Oct 2026).
        let london = at("2026-10-25T09:00 Europe/London").utc_ms().unwrap();
        let day_before = at("2026-10-24T09:00 Europe/London").utc_ms().unwrap();
        assert_eq!(london - day_before, 25 * 3_600_000);
        // The distance between the two zones changes with them: while only
        // the US is still on summer time, New York 09:00 is 13:00 in London.
        let ny = at("2026-10-28T09:00 America/New_York");
        assert_eq!(ny.in_zone("Europe/London").unwrap().local.hour, 13);
        let ny = at("2026-11-18T09:00 America/New_York");
        assert_eq!(ny.in_zone("Europe/London").unwrap().local.hour, 14);
    }

    #[test]
    fn a_skipped_time_lands_after_the_gap_and_a_repeated_one_is_the_first() {
        // 8 Mar 2026: 02:00 jumps to 03:00 in New York.
        let gap = at("2026-03-08T02:30 America/New_York").place().unwrap();
        assert_eq!(gap.fit, Fit::Gap);
        assert_eq!(
            from_utc_ms(gap.utc_ms, "America/New_York").unwrap().text(),
            "2026-03-08T03:30 America/New_York"
        );
        // 1 Nov 2026: 01:30 happens twice; the first is still summer time.
        let twice = at("2026-11-01T01:30 America/New_York").place().unwrap();
        assert_eq!(twice.fit, Fit::Ambiguous);
        assert_eq!(
            from_utc_ms(twice.utc_ms, "UTC").unwrap().text(),
            "2026-11-01T05:30 UTC"
        );
        assert_eq!(
            at("2026-06-01T12:00 America/New_York").place().unwrap().fit,
            Fit::Exact
        );
        // A zone with no daylight saving has no gap.
        assert_eq!(
            at("2026-03-08T02:30 Africa/Harare").place().unwrap().fit,
            Fit::Exact
        );
    }

    #[test]
    fn zones_are_listed_in_order_and_labelled_for_people() {
        let names = zone_names();
        assert!(names.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(names.contains(&"America/New_York") && names.contains(&"UTC"));
        assert_eq!(zone_label("America/New_York"), "New York");
        assert_eq!(zone_label("America/Argentina/Buenos_Aires"), "Buenos Aires");
        assert_eq!(zone_label("UTC"), "UTC");
    }

    #[test]
    fn offsets_follow_the_season() {
        let winter = at("2026-01-15T12:00 UTC").utc_ms().unwrap();
        let summer = at("2026-07-15T12:00 UTC").utc_ms().unwrap();
        assert_eq!(offset_secs("America/New_York", winter), Some(-5 * 3600));
        assert_eq!(offset_secs("America/New_York", summer), Some(-4 * 3600));
        assert_eq!(offset_secs("Asia/Kolkata", summer), Some(19_800));
        assert_eq!(offset_secs("Nowhere/Land", summer), None);
    }

    fn zones() -> impl Strategy<Value = &'static str> {
        prop_oneof![
            Just("America/New_York"),
            Just("Europe/London"),
            Just("Australia/Lord_Howe"),
            Just("Asia/Kolkata"),
            Just("Africa/Harare"),
            Just("Pacific/Apia"),
            Just("UTC"),
        ]
    }

    fn local() -> impl Strategy<Value = DateTime> {
        (1990i32..2060, 1u32..13, 1u32..29, 0u32..24, 0u32..60).prop_map(|(y, mo, d, h, mi)| {
            DateTime {
                date: Date::new(y, mo, d).unwrap(),
                hour: h,
                minute: mi,
                second: 0,
            }
        })
    }

    fn local_ms(shown: &DateTime) -> i64 {
        (shown.date.to_days() * 86_400
            + i64::from(shown.hour) * 3600
            + i64::from(shown.minute) * 60
            + i64::from(shown.second))
            * 1000
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        })]

        /// Every listed zone is one the reader accepts, and reads as a
        /// label with no area or underscore, so a picker's rows are never
        /// blank.
        #[test]
        fn prop_every_listed_zone_is_readable_and_labelled(
            name in proptest::sample::select(zone_names()),
        ) {
            prop_assert!(zone(name).is_some());
            let label = zone_label(name);
            prop_assert!(!label.is_empty());
            prop_assert!(!label.contains('_') && !label.contains('/'));
        }

        /// The text form reads back as the same value.
        #[test]
        fn prop_text_round_trips(local in local(), zone in zones()) {
            let value = ZonedTime { local, zone: zone.to_owned() };
            prop_assert_eq!(parse_zoned(&value.text()), Some(value));
        }

        /// A moment shown in another zone is that zone's reading of the same
        /// moment: the reading minus the zone's offset is the moment, and it
        /// places back on it unless it is a repeated hour (which names the
        /// first of two).
        #[test]
        fn prop_a_moment_survives_a_trip_through_another_zone(
            local in local(), from in zones(), through in zones(),
        ) {
            let start = ZonedTime { local, zone: from.to_owned() };
            let placed_start = start.place().unwrap();
            let there = start.in_zone(through).unwrap();
            let offset = i64::from(offset_secs(through, placed_start.utc_ms).unwrap());
            prop_assert_eq!(local_ms(&there.local) - offset * 1000, placed_start.utc_ms);
            let placed = there.place().unwrap();
            if placed.fit == Fit::Ambiguous {
                prop_assert!(placed.utc_ms <= placed_start.utc_ms);
                prop_assert!(placed_start.utc_ms - placed.utc_ms <= 2 * 3_600_000);
            } else {
                prop_assert_eq!(placed.utc_ms, placed_start.utc_ms);
            }
        }

        /// Within one zone, a later wall-clock time that sits once on the
        /// clock is a later moment, across any daylight saving change.
        #[test]
        fn prop_wall_clock_order_is_moment_order(
            a in local(), b in local(), zone in zones(),
        ) {
            let a = ZonedTime { local: a, zone: zone.to_owned() };
            let b = ZonedTime { local: b, zone: zone.to_owned() };
            let (pa, pb) = (a.place().unwrap(), b.place().unwrap());
            if pa.fit == Fit::Exact && pb.fit == Fit::Exact && a.local < b.local {
                prop_assert!(pa.utc_ms < pb.utc_ms);
            }
        }
    }
}
