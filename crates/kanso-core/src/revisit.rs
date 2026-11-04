//! "Revisit in…": a date on a note when it should come back (issue #240).
//!
//! The date lives in the note's `kansoRevisit` field, beside `kansoLoops`
//! and `kansoWordGoal`: `{ "on": "2026-11-18", "setAt": 1762240000000,
//! "why": "check the protocol survived" }`. It syncs with the note, sealed,
//! and is written without counting as an edit. Pure: reading the field,
//! the dates the menu offers and the words for a revisit that is late.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::dates::{Date, days_in_month};

/// The note field a revisit date is stored in.
pub const REVISIT_FIELD: &str = "kansoRevisit";
/// The longest reason kept, in characters.
pub const MAX_WHY_CHARS: usize = 200;
/// After this many days late, a revisit says "Overdue since".
pub const OVERDUE_AFTER_DAYS: i64 = 7;

/// A revisit date and the reason the person gave.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Revisit {
    /// `YYYY-MM-DD`.
    pub on: String,
    /// When it was set (ms since the epoch).
    #[serde(rename = "setAt")]
    pub set_at: i64,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub why: String,
}

impl Revisit {
    pub fn new(on: Date, set_at: i64, why: &str) -> Revisit {
        Revisit {
            on: on.iso(),
            set_at,
            why: why.trim().chars().take(MAX_WHY_CHARS).collect(),
        }
    }

    pub fn date(&self) -> Option<Date> {
        Date::parse_iso(&self.on)
    }
}

/// The revisit a note's fields hold; anything malformed is none.
pub fn read(fields: &Map<String, Value>) -> Option<Revisit> {
    let mut revisit: Revisit = serde_json::from_value(fields.get(REVISIT_FIELD)?.clone()).ok()?;
    Date::parse_iso(&revisit.on)?;
    revisit.why = revisit.why.chars().take(MAX_WHY_CHARS).collect();
    Some(revisit)
}

/// The field value for `revisit`.
pub fn to_value(revisit: &Revisit) -> Value {
    serde_json::to_value(revisit).unwrap_or(Value::Null)
}

/// One choice in the *Revisit in…* menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Choice {
    Tomorrow,
    InThreeDays,
    NextWeek,
    InTwoWeeks,
    InAMonth,
    InThreeMonths,
}

impl Choice {
    pub const ALL: [Choice; 6] = [
        Choice::Tomorrow,
        Choice::InThreeDays,
        Choice::NextWeek,
        Choice::InTwoWeeks,
        Choice::InAMonth,
        Choice::InThreeMonths,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Choice::Tomorrow => "Tomorrow",
            Choice::InThreeDays => "In 3 days",
            Choice::NextWeek => "Next week",
            Choice::InTwoWeeks => "In 2 weeks",
            Choice::InAMonth => "In a month",
            Choice::InThreeMonths => "In 3 months",
        }
    }

    /// The day this choice means when picked on `today`.
    pub fn date(self, today: Date) -> Date {
        match self {
            Choice::Tomorrow => today.add_days(1),
            Choice::InThreeDays => today.add_days(3),
            // The coming Monday, a week away at most.
            Choice::NextWeek => today.add_days(7 - today.weekday() as i64),
            Choice::InTwoWeeks => today.add_days(14),
            Choice::InAMonth => add_months(today, 1),
            Choice::InThreeMonths => add_months(today, 3),
        }
    }
}

/// `date` moved forward by `months`, the day kept or cut to the month's end.
pub fn add_months(date: Date, months: u32) -> Date {
    let index = i64::from(date.year) * 12 + (i64::from(date.month) - 1) + i64::from(months);
    let year = index.div_euclid(12) as i32;
    let month = index.rem_euclid(12) as u32 + 1;
    let day = date.day.min(days_in_month(year, month));
    Date::new(year, month, day).unwrap_or(date)
}

/// How a revisit that has come stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Standing {
    /// Not yet.
    Later,
    /// Due today or a few days late.
    Due,
    /// More than [`OVERDUE_AFTER_DAYS`] late: reads `Overdue since 4 Nov`.
    Overdue,
}

pub fn standing(revisit: &Revisit, today: Date) -> Standing {
    let Some(on) = revisit.date() else {
        return Standing::Later;
    };
    let late = today.to_days() - on.to_days();
    if late > OVERDUE_AFTER_DAYS {
        Standing::Overdue
    } else if late >= 0 {
        Standing::Due
    } else {
        Standing::Later
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn date() -> impl Strategy<Value = Date> {
        (1990i32..2100, 1u32..=12, 1u32..=28).prop_map(|(y, m, d)| Date::new(y, m, d).unwrap())
    }

    proptest! {
        #![proptest_config(ProptestConfig { rng_seed: proptest::test_runner::RngSeed::Fixed(0x006b_616e_736f), ..ProptestConfig::with_cases(512) })]

        /// Every choice lands after today, in order, and Next week is a
        /// Monday within a week.
        #[test]
        fn choices_are_ahead_and_ordered(today in date()) {
            let days: Vec<i64> = Choice::ALL.iter().map(|c| c.date(today).to_days() - today.to_days()).collect();
            prop_assert!(days.iter().all(|d| *d >= 1));
            prop_assert!(days[0] < days[1]);
            prop_assert!(days[3] < days[4]);
            prop_assert!(days[4] < days[5]);
            let monday = Choice::NextWeek.date(today);
            prop_assert_eq!(monday.weekday(), 0);
            prop_assert!(days[2] >= 1 && days[2] <= 7);
        }

        /// The field survives a round trip, and the reason is capped.
        #[test]
        fn field_round_trips(on in date(), at in 0i64..4_000_000_000_000, why in ".{0,300}") {
            let revisit = Revisit::new(on, at, &why);
            let mut fields = Map::new();
            fields.insert(REVISIT_FIELD.into(), to_value(&revisit));
            let back = read(&fields).unwrap();
            prop_assert!(back.why.chars().count() <= MAX_WHY_CHARS);
            prop_assert_eq!(back, revisit);
        }
    }

    #[test]
    fn the_menu_matches_the_spec_on_a_wednesday() {
        let today = Date::new(2026, 11, 4).unwrap();
        let got: Vec<String> = Choice::ALL.iter().map(|c| c.date(today).iso()).collect();
        assert_eq!(
            got,
            [
                "2026-11-05",
                "2026-11-07",
                "2026-11-09",
                "2026-11-18",
                "2026-12-04",
                "2027-02-04"
            ]
        );
    }

    #[test]
    fn a_month_from_the_thirty_first_is_cut() {
        assert_eq!(
            add_months(Date::new(2026, 1, 31).unwrap(), 1).iso(),
            "2026-02-28"
        );
    }

    #[test]
    fn standing_turns_overdue_after_a_week() {
        let r = Revisit::new(Date::new(2026, 11, 4).unwrap(), 0, "");
        assert_eq!(
            standing(&r, Date::new(2026, 11, 3).unwrap()),
            Standing::Later
        );
        assert_eq!(standing(&r, Date::new(2026, 11, 4).unwrap()), Standing::Due);
        assert_eq!(
            standing(&r, Date::new(2026, 11, 11).unwrap()),
            Standing::Due
        );
        assert_eq!(
            standing(&r, Date::new(2026, 11, 12).unwrap()),
            Standing::Overdue
        );
    }
}
