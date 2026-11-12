//! What a repeating task says: `every week`, `every 2 weeks on mon and thu`,
//! `every month on the last day`, `every 3 days after done`. The grammar,
//! the words and the next-date rule are in [`crate::repeat`].

/// A set of weekdays, one bit each: bit 0 is Monday and bit 6 is Sunday,
/// the order [`crate::dates::Date::weekday`] counts in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct WeekdaySet(pub u8);

impl WeekdaySet {
    pub const EMPTY: WeekdaySet = WeekdaySet(0);

    /// The set of one weekday (0 for Monday through 6 for Sunday).
    pub fn of(weekday: usize) -> WeekdaySet {
        WeekdaySet(1 << (weekday % 7))
    }

    pub fn contains(self, weekday: usize) -> bool {
        weekday < 7 && self.0 & (1 << weekday) != 0
    }

    pub fn insert(&mut self, weekday: usize) {
        self.0 |= 1 << (weekday % 7);
    }

    pub fn is_empty(self) -> bool {
        self.0 & 0x7f == 0
    }

    /// The weekdays in the set, Monday first.
    pub fn days(self) -> impl Iterator<Item = usize> {
        (0..7).filter(move |&d| self.contains(d))
    }
}

/// Which day a monthly rule lands on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MonthDay {
    /// The day of the month the task was due on (`every month`).
    Same,
    /// A day of the month, 1 to 31: a month with fewer days uses its last
    /// (`every month on the 31st`).
    Date(u8),
    /// The last day of the month.
    Last,
    /// The `n`th weekday of the month: `n` is 1 to 4, or -1 for the last
    /// one. The weekday is 0 for Monday through 6 for Sunday.
    Nth(i8, u8),
}

/// How often a task comes round.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Every {
    /// Every `n` days.
    Days(u16),
    /// Every `n` weeks, on the due date's weekday when the set is empty
    /// and on each weekday in the set otherwise.
    Weeks(u16, WeekdaySet),
    /// Every `n` months, on the day the rule names.
    Months(u16, MonthDay),
    /// Every `n` years, on the same day (29 February becomes 28 February
    /// in a year without one).
    Years(u16),
    /// Monday to Friday.
    Weekdays,
}

/// The `every …` after a task's due date.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Repeat {
    pub every: Every,
    /// Count from the day it was done, not from the day it was due
    /// (`every 3 days after done`).
    pub after_done: bool,
}
