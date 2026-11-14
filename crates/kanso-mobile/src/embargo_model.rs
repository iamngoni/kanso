//! What the phone is told of a note's embargo. The rules are
//! `kanso_core::embargo`'s.

/// A note's embargo while it is still in force.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EmbargoView {
    /// When it lifts, as people read it: `Wed 18 Nov, 09:00 New York`, or
    /// `Wed 18 Nov, 09:00` with no zone.
    pub lifts: String,
    /// How far off that is: `in 5 days`, `in 3 hours`.
    pub lifts_in: String,
    /// The moment it lifts, in milliseconds since the epoch (UTC).
    pub lifts_at_ms: i64,
    /// The zone it is set in (`America/New_York`), when it names one.
    pub zone: Option<String>,
}
