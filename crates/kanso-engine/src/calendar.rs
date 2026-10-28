//! The Journal's month calendar without a window: a month laid out in weeks
//! from the first day of the week the person's system uses, and month
//! arithmetic. The desktop draws it in `calendar_view.rs` and the mobile
//! app in Flutter; the first weekday of a platform's calendar is the
//! caller's to ask (see [`from_first_weekday`]).

use kanso_core::dates::{Date, MONTHS, WEEKDAYS, days_in_month};

/// A month as the calendar shows it: whole weeks, starting on the week's
/// first day, so the days before the 1st and after the last day come from
/// the months either side.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MonthGrid {
    pub year: i32,
    pub month: u32,
    pub weeks: Vec<[Date; 7]>,
}

impl MonthGrid {
    /// `week_start` is the week's first day, 0 for Monday through 6 for
    /// Sunday, as [`Date::weekday`] counts.
    pub fn new(year: i32, month: u32, week_start: usize) -> MonthGrid {
        let first = Date::new(year, month, 1).unwrap_or(Date {
            year,
            month: 1,
            day: 1,
        });
        let last = first.add_days(i64::from(days_in_month(year, first.month)) - 1);
        let back = (first.weekday() + 7 - week_start % 7) % 7;
        let mut start = first.add_days(-(back as i64));
        let mut weeks = Vec::new();
        while start <= last {
            weeks.push(std::array::from_fn(|ix| start.add_days(ix as i64)));
            start = start.add_days(7);
        }
        MonthGrid {
            year,
            month: first.month,
            weeks,
        }
    }

    /// Whether `date` is in the month itself, not a neighbour's.
    pub fn in_month(&self, date: Date) -> bool {
        date.year == self.year && date.month == self.month
    }

    /// "October 2026".
    pub fn title(&self) -> String {
        format!(
            "{} {}",
            MONTHS[(self.month as usize).clamp(1, 12) - 1],
            self.year
        )
    }
}

/// The weekdays' short names in the grid's order: "Mon" first, or "Sun".
pub fn weekday_labels(week_start: usize) -> [&'static str; 7] {
    std::array::from_fn(|ix| &WEEKDAYS[(week_start + ix) % 7][..3])
}

/// The month `delta` months after (or before) `year`-`month`.
pub fn shift_month(year: i32, month: u32, delta: i32) -> (i32, u32) {
    let index = year * 12 + (month as i32 - 1) + delta;
    (index.div_euclid(12), index.rem_euclid(12) as u32 + 1)
}

/// The same day `delta` months away, or that month's last day when it is
/// shorter (31 January and one month on is 28 or 29 February).
pub fn add_months(date: Date, delta: i32) -> Date {
    let (year, month) = shift_month(date.year, date.month, delta);
    Date {
        year,
        month,
        day: date.day.min(days_in_month(year, month)),
    }
}

/// A platform calendar's first weekday (Foundation's and Java's count: 1 for
/// Sunday through 7 for Saturday) as the grid counts it (0 for Monday
/// through 6 for Sunday). Anything else is Monday.
pub fn from_first_weekday(first: usize) -> usize {
    match first {
        1..=7 => (first + 5) % 7,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::{MonthGrid, add_months, from_first_weekday, shift_month, weekday_labels};
    use kanso_core::dates::{Date, days_in_month};
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 512,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    #[test]
    fn october_2026() {
        // 1 October 2026 is a Thursday.
        let monday = MonthGrid::new(2026, 10, 0);
        assert_eq!(monday.title(), "October 2026");
        assert_eq!(monday.weeks.len(), 5);
        assert_eq!(monday.weeks[0][0], Date::new(2026, 9, 28).unwrap());
        assert_eq!(monday.weeks[0][3], Date::new(2026, 10, 1).unwrap());
        assert_eq!(monday.weeks[4][6], Date::new(2026, 11, 1).unwrap());
        let sunday = MonthGrid::new(2026, 10, 6);
        assert_eq!(sunday.weeks[0][0], Date::new(2026, 9, 27).unwrap());
        assert_eq!(weekday_labels(6)[..2], ["Sun", "Mon"]);
        assert_eq!(weekday_labels(0)[6], "Sun");
        // February 2027 starts on a Monday and has exactly four weeks.
        assert_eq!(MonthGrid::new(2027, 2, 0).weeks.len(), 4);
        assert_eq!(shift_month(2026, 1, -1), (2025, 12));
        assert_eq!(shift_month(2026, 12, 1), (2027, 1));
        assert_eq!(
            add_months(Date::new(2024, 1, 31).unwrap(), 1),
            Date::new(2024, 2, 29).unwrap()
        );
        assert_eq!(from_first_weekday(1), 6, "Sunday");
        assert_eq!(from_first_weekday(2), 0, "Monday");
        assert_eq!(from_first_weekday(7), 5, "Saturday");
        assert_eq!(from_first_weekday(0), 0);
    }

    proptest! {
        #![proptest_config(config())]

        /// Invariants: every day of the month shows exactly once in it,
        /// weeks are seven days in a row, each column is one weekday, the
        /// first column the week's first day, and there are four to six
        /// weeks, none of them wholly outside the month.
        #[test]
        fn prop_month_grid_holds_each_day_once(
            year in 1600i32..2600,
            month in 1u32..=12,
            week_start in 0usize..7,
        ) {
            let grid = MonthGrid::new(year, month, week_start);
            prop_assert!((4..=6).contains(&grid.weeks.len()));
            let days: Vec<Date> = grid.weeks.iter().flatten().copied().collect();
            for pair in days.windows(2) {
                prop_assert_eq!(pair[1], pair[0].add_days(1));
            }
            for week in &grid.weeks {
                prop_assert!(week.iter().any(|d| grid.in_month(*d)));
                for (col, day) in week.iter().enumerate() {
                    prop_assert_eq!(day.weekday(), (week_start + col) % 7);
                }
            }
            let inside: Vec<u32> = days.iter().filter(|d| grid.in_month(**d)).map(|d| d.day).collect();
            prop_assert_eq!(inside, (1..=days_in_month(year, month)).collect::<Vec<_>>());
        }

        /// Shifting months adds up, and back again is where it began.
        #[test]
        fn prop_shift_month_adds_up(
            year in 1600i32..2600,
            month in 1u32..=12,
            a in -40i32..40,
            b in -40i32..40,
        ) {
            let (y, m) = shift_month(year, month, a);
            prop_assert!((1..=12).contains(&m));
            prop_assert_eq!(shift_month(y, m, b), shift_month(year, month, a + b));
            prop_assert_eq!(shift_month(y, m, -a), (year, month));
        }

        /// A month on is a real date in the next month, on the same day
        /// unless that month is shorter.
        #[test]
        fn prop_add_months_lands_in_the_month(days in -200_000i64..200_000, delta in -30i32..30) {
            let date = Date::from_days(days);
            let moved = add_months(date, delta);
            prop_assert!(Date::new(moved.year, moved.month, moved.day).is_some());
            prop_assert_eq!((moved.year, moved.month), shift_month(date.year, date.month, delta));
            prop_assert_eq!(moved.day, date.day.min(days_in_month(moved.year, moved.month)));
        }
    }
}
