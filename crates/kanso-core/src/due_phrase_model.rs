//! What the typed-date parser ([`crate::due_phrase`]) takes and gives back.

use std::ops::Range;

use crate::tasks::Due;

/// How a date written as two numbers with a slash reads: `14/11` is the
/// 14th of November in [`DateOrder::DayMonth`] and not a date at all in
/// [`DateOrder::MonthDay`], where `11/14` is the same day. The caller
/// passes the system's order; the parser never tries the other one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum DateOrder {
    #[default]
    DayMonth,
    MonthDay,
}

/// A date and time found at the end of a line of text.
///
/// Repeats ("every week") are not part of it yet: they arrive with the
/// repeat grammar in `tasks.rs`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parsed {
    /// The bytes of the phrase in the input, from its first word to its
    /// last. Only whitespace follows it.
    pub span: Range<usize>,
    /// The day, and the time of day when the phrase named one.
    pub due: Due,
}
