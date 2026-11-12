//! A wall-clock time with a time zone, as a date-time-with-zone property
//! holds it (see [`crate::zoned`]).

use crate::dates::DateTime;

/// A local date and time in a named zone, as written: `2026-11-18T09:00
/// America/New_York`. The zone is an IANA name, so it survives travel and
/// daylight saving changes; the same wall-clock time is a different instant
/// in each.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZonedTime {
    pub local: DateTime,
    pub zone: String,
}

/// How a wall-clock time sits in its zone's calendar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fit {
    /// The time happens once.
    Exact,
    /// The clocks skip it (spring forward). It means the same moment as the
    /// time after the gap: 02:30 where 02:00 jumps to 03:00 is 03:30.
    Gap,
    /// The clocks show it twice (fall back). It means the first.
    Ambiguous,
}

/// A wall-clock time placed on the timeline.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placed {
    /// Milliseconds since the Unix epoch.
    pub utc_ms: i64,
    pub fit: Fit,
}
