use proptest::prelude::*;
use proptest::test_runner::{Config, RngAlgorithm, RngSeed};

use super::*;
use crate::tasks::task_items;

fn d(year: i32, month: u32, day: u32) -> Date {
    Date::new(year, month, day).unwrap()
}

fn due(date: Date, time: Option<(u32, u32)>) -> Due {
    Due { date, time }
}

fn rule(every: Every) -> Repeat {
    Repeat {
        every,
        after_done: false,
    }
}

fn set(days: &[usize]) -> WeekdaySet {
    let mut set = WeekdaySet::EMPTY;
    for &day in days {
        set.insert(day);
    }
    set
}

/// The next date when it is done on the day it was due.
fn next(repeat: Repeat, from: Date) -> Date {
    repeat.next(&due(from, None), from).date
}

// Thursday 12 November 2026.
const MON: usize = 0;
const TUE: usize = 1;
const WED: usize = 2;
const THU: usize = 3;
const FRI: usize = 4;

// MARK: Next date, by rule

#[test]
fn days() {
    assert_eq!(next(rule(Every::Days(1)), d(2026, 11, 12)), d(2026, 11, 13));
    assert_eq!(next(rule(Every::Days(3)), d(2026, 11, 12)), d(2026, 11, 15));
    assert_eq!(next(rule(Every::Days(1)), d(2026, 12, 31)), d(2027, 1, 1));
}

#[test]
fn weekdays_skip_the_weekend() {
    let every = rule(Every::Weekdays);
    assert_eq!(next(every, d(2026, 11, 12)), d(2026, 11, 13));
    assert_eq!(next(every, d(2026, 11, 13)), d(2026, 11, 16));
    // from a Saturday and a Sunday
    assert_eq!(next(every, d(2026, 11, 14)), d(2026, 11, 16));
    assert_eq!(next(every, d(2026, 11, 15)), d(2026, 11, 16));
}

#[test]
fn weeks() {
    assert_eq!(
        next(rule(Every::Weeks(1, WeekdaySet::EMPTY)), d(2026, 11, 12)),
        d(2026, 11, 19)
    );
    assert_eq!(
        next(rule(Every::Weeks(2, WeekdaySet::EMPTY)), d(2026, 11, 12)),
        d(2026, 11, 26)
    );
}

#[test]
fn named_weekdays() {
    let mon_thu = rule(Every::Weeks(1, set(&[MON, THU])));
    assert_eq!(next(mon_thu, d(2026, 11, 12)), d(2026, 11, 16));
    assert_eq!(next(mon_thu, d(2026, 11, 16)), d(2026, 11, 19));
    // from a day not in the set
    assert_eq!(next(mon_thu, d(2026, 11, 13)), d(2026, 11, 16));
    assert_eq!(next(mon_thu, d(2026, 11, 10)), d(2026, 11, 12));
}

#[test]
fn every_other_week_on_named_weekdays() {
    let every = rule(Every::Weeks(2, set(&[MON, WED])));
    assert_eq!(next(every, d(2026, 11, 16)), d(2026, 11, 18));
    // after the last one in its week, two weeks on from that week
    assert_eq!(next(every, d(2026, 11, 18)), d(2026, 11, 30));
    assert_eq!(next(every, d(2026, 11, 21)), d(2026, 11, 30));
}

#[test]
fn months_on_the_same_day() {
    let every = rule(Every::Months(1, MonthDay::Same));
    assert_eq!(next(every, d(2026, 11, 12)), d(2026, 12, 12));
    assert_eq!(next(every, d(2026, 12, 12)), d(2027, 1, 12));
    assert_eq!(
        next(rule(Every::Months(3, MonthDay::Same)), d(2026, 11, 12)),
        d(2027, 2, 12)
    );
}

#[test]
fn month_end_clamps() {
    let same = rule(Every::Months(1, MonthDay::Same));
    assert_eq!(next(same, d(2026, 10, 31)), d(2026, 11, 30));
    assert_eq!(next(same, d(2026, 1, 31)), d(2026, 2, 28));
    assert_eq!(next(same, d(2028, 1, 31)), d(2028, 2, 29));

    // a rule that names the 31st does not drift down
    let on_31 = rule(Every::Months(1, MonthDay::Date(31)));
    assert_eq!(next(on_31, d(2026, 1, 31)), d(2026, 2, 28));
    assert_eq!(next(on_31, d(2026, 2, 28)), d(2026, 3, 31));
    assert_eq!(next(on_31, d(2026, 10, 31)), d(2026, 11, 30));
    assert_eq!(next(on_31, d(2026, 11, 30)), d(2026, 12, 31));
    assert_eq!(next(on_31, d(2028, 1, 31)), d(2028, 2, 29));
}

#[test]
fn months_on_a_date_the_last_day_and_the_nth_weekday() {
    let on_15 = rule(Every::Months(1, MonthDay::Date(15)));
    assert_eq!(next(on_15, d(2026, 11, 12)), d(2026, 12, 15));
    // the 2026 calendar: 1 December is a Tuesday
    let last = rule(Every::Months(1, MonthDay::Last));
    assert_eq!(next(last, d(2026, 11, 12)), d(2026, 12, 31));
    assert_eq!(next(last, d(2026, 1, 15)), d(2026, 2, 28));
    assert_eq!(next(last, d(2028, 1, 15)), d(2028, 2, 29));
    let second_tue = rule(Every::Months(1, MonthDay::Nth(2, TUE as u8)));
    assert_eq!(next(second_tue, d(2026, 11, 12)), d(2026, 12, 8));
    let last_fri = rule(Every::Months(1, MonthDay::Nth(-1, FRI as u8)));
    assert_eq!(next(last_fri, d(2026, 11, 12)), d(2026, 12, 25));
    assert_eq!(next(last_fri, d(2026, 12, 25)), d(2027, 1, 29));
    // a month whose first day is the weekday asked for
    let first_tue = rule(Every::Months(1, MonthDay::Nth(1, TUE as u8)));
    assert_eq!(next(first_tue, d(2026, 11, 12)), d(2026, 12, 1));
}

#[test]
fn years_and_29_february() {
    let yearly = rule(Every::Years(1));
    assert_eq!(next(yearly, d(2026, 11, 12)), d(2027, 11, 12));
    assert_eq!(next(yearly, d(2028, 2, 29)), d(2029, 2, 28));
    assert_eq!(next(yearly, d(2027, 2, 28)), d(2028, 2, 28));
    assert_eq!(next(rule(Every::Years(4)), d(2028, 2, 29)), d(2032, 2, 29));
    // 2100 is not a leap year
    assert_eq!(next(rule(Every::Years(4)), d(2096, 2, 29)), d(2100, 2, 28));
}

#[test]
fn the_time_of_day_is_kept() {
    let every = rule(Every::Days(1));
    let at = due(d(2026, 11, 12), Some((15, 0)));
    assert_eq!(
        every.next(&at, d(2026, 11, 12)),
        due(d(2026, 11, 13), Some((15, 0)))
    );
    let at = due(d(2026, 11, 12), Some((0, 5)));
    assert_eq!(every.next(&at, d(2026, 11, 12)).time, Some((0, 5)));
}

#[test]
fn a_date_across_a_clock_change_is_still_one_calendar_day_on() {
    // Europe springs forward on 2026-03-29 (02:30 does not exist that day)
    // and the US falls back on 2026-11-01 (01:30 happens twice). The date
    // is local wall-clock, so the time stays as written and the day is
    // exactly one calendar day on: 24 hours are never assumed.
    for (day, time) in [
        (d(2026, 3, 28), (2, 30)),
        (d(2026, 3, 8), (2, 30)),
        (d(2026, 10, 31), (1, 30)),
        (d(2026, 10, 24), (2, 30)),
    ] {
        let before = due(day, Some(time));
        let after = rule(Every::Days(1)).next(&before, day);
        assert_eq!(after.time, Some(time));
        assert_eq!(after.date, day.add_days(1));
        assert_eq!(after.local_ms() - before.local_ms(), 86_400_000);
        let week = rule(Every::Weeks(1, WeekdaySet::EMPTY)).next(&before, day);
        assert_eq!(week.local_ms() - before.local_ms(), 7 * 86_400_000);
        assert_eq!(week.time, Some(time));
    }
}

// MARK: After done, overdue

#[test]
fn after_done_counts_from_the_day_it_was_done() {
    let every = Repeat {
        every: Every::Days(3),
        after_done: true,
    };
    let at = due(d(2026, 11, 12), Some((9, 0)));
    // done late
    assert_eq!(
        every.next(&at, d(2026, 11, 20)),
        due(d(2026, 11, 23), Some((9, 0)))
    );
    // done on the day
    assert_eq!(every.next(&at, d(2026, 11, 12)).date, d(2026, 11, 15));
    // done early: counted from the due date, never before it
    assert_eq!(every.next(&at, d(2026, 11, 10)).date, d(2026, 11, 15));
    let monthly = Repeat {
        every: Every::Months(1, MonthDay::Same),
        after_done: true,
    };
    assert_eq!(
        monthly.next(&due(d(2026, 1, 5), None), d(2026, 1, 31)).date,
        d(2026, 2, 28)
    );
}

#[test]
fn an_overdue_task_skips_to_the_next_future_date() {
    let weekly = rule(Every::Weeks(1, WeekdaySet::EMPTY));
    let late = due(d(2026, 10, 1), Some((9, 0)));
    // five weeks late, done on a Thursday: the next Thursday, not five catch-ups
    assert_eq!(
        weekly.next(&late, d(2026, 11, 12)),
        due(d(2026, 11, 19), Some((9, 0)))
    );
    assert_eq!(weekly.next(&late, d(2026, 11, 13)).date, d(2026, 11, 19));
    assert_eq!(weekly.next(&late, d(2026, 11, 11)).date, d(2026, 11, 12));
    // done early or on time is one step
    assert_eq!(weekly.next(&late, d(2026, 9, 1)).date, d(2026, 10, 8));
    let daily = rule(Every::Days(1));
    assert_eq!(daily.next(&late, d(2026, 11, 12)).date, d(2026, 11, 13));
    let monthly = rule(Every::Months(1, MonthDay::Date(15)));
    assert_eq!(monthly.next(&late, d(2026, 11, 16)).date, d(2026, 12, 15));
}

// MARK: Words

const CORPUS: &[(&str, Repeat)] = &[
    ("every day", rule_const(Every::Days(1), false)),
    ("daily", rule_const(Every::Days(1), false)),
    ("every 3 days", rule_const(Every::Days(3), false)),
    ("every weekday", rule_const(Every::Weekdays, false)),
    (
        "every week",
        rule_const(Every::Weeks(1, WeekdaySet::EMPTY), false),
    ),
    (
        "weekly",
        rule_const(Every::Weeks(1, WeekdaySet::EMPTY), false),
    ),
    (
        "EVERY WEEK",
        rule_const(Every::Weeks(1, WeekdaySet::EMPTY), false),
    ),
    (
        "every 2 weeks",
        rule_const(Every::Weeks(2, WeekdaySet::EMPTY), false),
    ),
    (
        "every mon",
        rule_const(Every::Weeks(1, WeekdaySet(1)), false),
    ),
    (
        "every monday",
        rule_const(Every::Weeks(1, WeekdaySet(1)), false),
    ),
    (
        "every mon and thu",
        rule_const(Every::Weeks(1, WeekdaySet(0b0001001)), false),
    ),
    (
        "every mon, thu",
        rule_const(Every::Weeks(1, WeekdaySet(0b0001001)), false),
    ),
    (
        "every mon, wed and fri",
        rule_const(Every::Weeks(1, WeekdaySet(0b0010101)), false),
    ),
    (
        "every 2 weeks on mon and thu",
        rule_const(Every::Weeks(2, WeekdaySet(0b0001001)), false),
    ),
    (
        "every month",
        rule_const(Every::Months(1, MonthDay::Same), false),
    ),
    (
        "monthly",
        rule_const(Every::Months(1, MonthDay::Same), false),
    ),
    (
        "every 3 months",
        rule_const(Every::Months(3, MonthDay::Same), false),
    ),
    (
        "every month on the 15th",
        rule_const(Every::Months(1, MonthDay::Date(15)), false),
    ),
    (
        "every month on the 1st",
        rule_const(Every::Months(1, MonthDay::Date(1)), false),
    ),
    (
        "every month on the 31st",
        rule_const(Every::Months(1, MonthDay::Date(31)), false),
    ),
    (
        "every month on the last day",
        rule_const(Every::Months(1, MonthDay::Last), false),
    ),
    (
        "every month on the 2nd tue",
        rule_const(Every::Months(1, MonthDay::Nth(2, 1)), false),
    ),
    (
        "every 2 months on the last fri",
        rule_const(Every::Months(2, MonthDay::Nth(-1, 4)), false),
    ),
    ("every year", rule_const(Every::Years(1), false)),
    ("yearly", rule_const(Every::Years(1), false)),
    ("every 2 years", rule_const(Every::Years(2), false)),
    ("every 3 days after done", rule_const(Every::Days(3), true)),
    (
        "every week after done",
        rule_const(Every::Weeks(1, WeekdaySet::EMPTY), true),
    ),
    (
        "every month on the last day after done",
        rule_const(Every::Months(1, MonthDay::Last), true),
    ),
];

const fn rule_const(every: Every, after_done: bool) -> Repeat {
    Repeat { every, after_done }
}

#[test]
fn the_grammar_reads_each_form() {
    for (text, want) in CORPUS {
        let (got, len) = parse_repeat(text).unwrap_or_else(|| panic!("{text}"));
        assert_eq!(got, *want, "{text}");
        assert_eq!(len, text.len(), "{text}");
    }
}

#[test]
fn canonical_words() {
    for (text, repeat) in [
        ("every day", rule(Every::Days(1))),
        ("every 3 days", rule(Every::Days(3))),
        ("every weekday", rule(Every::Weekdays)),
        ("every week", rule(Every::Weeks(1, WeekdaySet::EMPTY))),
        ("every mon", rule(Every::Weeks(1, set(&[MON])))),
        ("every mon and thu", rule(Every::Weeks(1, set(&[MON, THU])))),
        (
            "every mon, wed and fri",
            rule(Every::Weeks(1, set(&[MON, WED, FRI]))),
        ),
        (
            "every 2 weeks on mon and thu",
            rule(Every::Weeks(2, set(&[MON, THU]))),
        ),
        ("every month", rule(Every::Months(1, MonthDay::Same))),
        (
            "every month on the 22nd",
            rule(Every::Months(1, MonthDay::Date(22))),
        ),
        (
            "every month on the 11th",
            rule(Every::Months(1, MonthDay::Date(11))),
        ),
        (
            "every 2 months on the 3rd wed",
            rule(Every::Months(2, MonthDay::Nth(3, WED as u8))),
        ),
        (
            "every month on the last day after done",
            Repeat {
                every: Every::Months(1, MonthDay::Last),
                after_done: true,
            },
        ),
        ("every year", rule(Every::Years(1))),
    ] {
        assert_eq!(repeat.to_string(), text);
    }
}

#[test]
fn what_is_not_a_repeat_is_not_read() {
    for text in [
        "",
        "every",
        "every blue moon",
        "every 0 days",
        "every 2 day",
        "every days",
        "every month on",
        "every month on the 32nd",
        "every month on the 0th",
        "every month on the 5th tue",
        "every month on 15th",
        "every week on the",
        "every 2 weeks on",
        "each week",
        "twice a week",
        "week",
        "everyday",
    ] {
        assert_eq!(parse_repeat(text), None, "{text:?}");
    }
}

#[test]
fn a_repeat_ends_where_the_grammar_does() {
    let (repeat, len) = parse_repeat("every mon and then lunch").unwrap();
    assert_eq!(repeat, rule(Every::Weeks(1, set(&[MON]))));
    assert_eq!(len, "every mon".len());
    let (_, len) = parse_repeat("every week after lunch").unwrap();
    assert_eq!(len, "every week".len());
    let (_, len) = parse_repeat("every week after done, thanks").unwrap();
    assert_eq!(len, "every week after done".len());
}

// MARK: In a task's text

#[test]
fn a_repeat_follows_the_due_date() {
    let text = "film Pixel review @2026-11-14T15:00 every week";
    let (range, repeat) = repeat_token(text).unwrap();
    assert_eq!(&text[range], "every week");
    assert_eq!(repeat, rule(Every::Weeks(1, WeekdaySet::EMPTY)));

    let text = "upload @2026-11-17  every 2 weeks #sponsor";
    let (range, repeat) = repeat_token(text).unwrap();
    assert_eq!(&text[range], "every 2 weeks");
    assert_eq!(repeat, rule(Every::Weeks(2, WeekdaySet::EMPTY)));

    let text = "water plants @2026-11-12 every 3 days after done";
    let (range, repeat) = repeat_token(text).unwrap();
    assert_eq!(&text[range], "every 3 days after done");
    assert!(repeat.after_done);
}

#[test]
fn a_repeat_anywhere_else_is_prose() {
    for text in [
        "every week",
        "every week @2026-11-14",
        "film every week @2026-11-14",
        "film @2026-11-14 and every week",
        "film @2026-11-14every week",
        "film @2026-11-14 every other week",
        "film @2026-11-14",
        "film @2026-11-14 weekly-ish",
    ] {
        assert_eq!(repeat_token(text), None, "{text}");
    }
}

#[test]
fn task_items_carry_the_repeat() {
    let body = "- [ ] film @2026-11-14T15:00 every week\n\
                - [x] done @2026-11-13 every mon and thu\n\
                - [ ] plain @2026-11-13\n\
                - [ ] every week\n";
    let items = task_items(body);
    assert_eq!(items.len(), 4);
    assert_eq!(
        items[0].repeat,
        Some(rule(Every::Weeks(1, WeekdaySet::EMPTY)))
    );
    assert_eq!(items[0].due.as_deref(), Some("2026-11-14T15:00"));
    assert_eq!(
        items[1].repeat,
        Some(rule(Every::Weeks(1, set(&[MON, THU]))))
    );
    assert_eq!(items[2].repeat, None);
    assert_eq!(items[3].repeat, None);
}

// MARK: Properties

fn config() -> Config {
    Config {
        cases: 512,
        rng_seed: RngSeed::Fixed(0x0072_7065_6174),
        rng_algorithm: RngAlgorithm::ChaCha,
        failure_persistence: None,
        ..Config::default()
    }
}

fn gen_date() -> impl Strategy<Value = Date> {
    (-20_000i64..60_000).prop_map(Date::from_days)
}

fn gen_set() -> impl Strategy<Value = WeekdaySet> {
    prop_oneof![Just(WeekdaySet::EMPTY), (1u8..128).prop_map(WeekdaySet)]
}

fn gen_month_day() -> impl Strategy<Value = MonthDay> {
    prop_oneof![
        Just(MonthDay::Same),
        (1u8..=31).prop_map(MonthDay::Date),
        Just(MonthDay::Last),
        (prop_oneof![Just(-1i8), 1i8..=4], 0u8..7).prop_map(|(n, day)| MonthDay::Nth(n, day)),
    ]
}

fn gen_every() -> impl Strategy<Value = Every> {
    prop_oneof![
        (1u16..=400).prop_map(Every::Days),
        (1u16..=60, gen_set()).prop_map(|(n, set)| Every::Weeks(n, set)),
        (1u16..=48, gen_month_day()).prop_map(|(n, rule)| Every::Months(n, rule)),
        (1u16..=60).prop_map(Every::Years),
        Just(Every::Weekdays),
    ]
}

fn gen_repeat() -> impl Strategy<Value = Repeat> {
    (gen_every(), any::<bool>()).prop_map(|(every, after_done)| Repeat { every, after_done })
}

fn gen_time() -> impl Strategy<Value = Option<(u32, u32)>> {
    prop::option::of((0u32..24, 0u32..60))
}

proptest! {
    #![proptest_config(config())]

    /// The canonical words read back as the same repeat, whole.
    #[test]
    fn prop_canonical_form_round_trips(repeat in gen_repeat()) {
        let words = repeat.to_string();
        prop_assert_eq!(parse_repeat(&words), Some((repeat, words.len())));
        // and once more: the canonical form is a fixed point
        let (again, _) = parse_repeat(&words).unwrap();
        prop_assert_eq!(again.to_string(), words);
    }

    /// In a task, after its due date and before other words, the repeat is
    /// the one written and the range is exactly its words.
    #[test]
    fn prop_repeat_in_a_task_round_trips(
        repeat in gen_repeat(),
        date in gen_date(),
        time in gen_time(),
        tail in prop_oneof![Just(""), Just(" #sponsor"), Just(" soon")],
    ) {
        let words = repeat.to_string();
        let due = Due { date, time };
        let text = format!("film review @{} {words}{tail}", due.format());
        let (range, found) = repeat_token(&text).unwrap();
        prop_assert_eq!(found, repeat);
        prop_assert_eq!(&text[range], words.as_str());
        let items = task_items(&format!("- [ ] {text}"));
        prop_assert_eq!(items[0].repeat, Some(repeat));
    }

    /// The next date is always strictly after the current one, and after the
    /// day it was done, however late; and the time of day is kept.
    #[test]
    fn prop_next_is_strictly_after(
        repeat in gen_repeat(),
        date in gen_date(),
        time in gen_time(),
        offset in -400i64..3000,
    ) {
        let current = Due { date, time };
        let done_on = date.add_days(offset);
        let next = repeat.next(&current, done_on);
        prop_assert!(next.date > current.date);
        prop_assert!(next.date > done_on);
        prop_assert_eq!(next.time, time);
        prop_assert!(next.local_ms() > current.local_ms());
    }

    /// Every step of a chain is after the one before: a task never repeats
    /// into the past, whichever rule and whichever ticking day.
    #[test]
    fn prop_a_chain_only_moves_forward(repeat in gen_repeat(), date in gen_date()) {
        let mut current = Due { date, time: Some((9, 0)) };
        for _ in 0..40 {
            let next = repeat.next(&current, current.date);
            prop_assert!(next > current);
            current = next;
        }
    }

    /// Not after done: the dates are the rule's own, so the result is the
    /// first one after both the due date and the day it was done, and the
    /// step before it is not after that day. An overdue task never gets a run
    /// of catch-up instances.
    #[test]
    fn prop_overdue_skips_to_the_first_future_date(
        n in 1u16..=30,
        date in gen_date(),
        late in 0i64..1000,
    ) {
        let repeat = rule(Every::Days(n));
        let done_on = date.add_days(late);
        let next = repeat.next(&Due { date, time: None }, done_on);
        prop_assert!(next.date > done_on);
        prop_assert!(next.date.add_days(-i64::from(n)) <= done_on);
        prop_assert_eq!((next.date.to_days() - date.to_days()) % i64::from(n), 0);
    }

    /// After done counts from the day it was done: exactly `n` days on.
    #[test]
    fn prop_after_done_counts_from_done(n in 1u16..=60, date in gen_date(), late in 0i64..500) {
        let repeat = Repeat { every: Every::Days(n), after_done: true };
        let done_on = date.add_days(late);
        let next = repeat.next(&Due { date, time: None }, done_on);
        prop_assert_eq!(next.date, done_on.add_days(i64::from(n)));
    }

    /// Month ends: a rule that names a date lands on it, or the month's last
    /// day when the month is short, and never drifts down for good.
    #[test]
    fn prop_a_monthly_date_never_drifts(
        day in 1u8..=31,
        n in 1u16..=12,
        date in gen_date(),
    ) {
        let repeat = rule(Every::Months(n, MonthDay::Date(day)));
        let mut current = Due { date, time: None };
        for _ in 0..30 {
            current = repeat.next(&current, current.date);
            let length = days_in_month(current.date.year, current.date.month);
            prop_assert_eq!(current.date.day, u32::from(day).min(length));
        }
    }

    /// Every month on the same day: the day is the due date's, or the
    /// month's last when it is short.
    #[test]
    fn prop_same_day_of_month_clamps(n in 1u16..=12, date in gen_date()) {
        let next = next(rule(Every::Months(n, MonthDay::Same)), date);
        let length = days_in_month(next.year, next.month);
        prop_assert_eq!(next.day, date.day.min(length));
        let months = |x: Date| i64::from(x.year) * 12 + i64::from(x.month);
        prop_assert_eq!(months(next) - months(date), i64::from(n));
    }

    /// The last day, and the nth weekday, land where they say.
    #[test]
    fn prop_last_day_and_nth_weekday(
        date in gen_date(),
        nth in prop_oneof![Just(-1i8), 1i8..=4],
        weekday in 0u8..7,
    ) {
        let last = next(rule(Every::Months(1, MonthDay::Last)), date);
        prop_assert_eq!(last.day, days_in_month(last.year, last.month));
        let found = next(rule(Every::Months(1, MonthDay::Nth(nth, weekday))), date);
        prop_assert_eq!(found.weekday(), usize::from(weekday));
        prop_assert_eq!((found.year, found.month) == (last.year, last.month), true);
        if nth > 0 {
            prop_assert_eq!(i64::from((found.day - 1) / 7 + 1), i64::from(nth));
        } else {
            prop_assert!(found.day + 7 > days_in_month(found.year, found.month));
        }
    }

    /// 29 February: a year rule keeps the month and the day, or the 28th in
    /// a year with no 29th; leap days come back on a leap year.
    #[test]
    fn prop_years_keep_the_day_or_clamp(n in 1u16..=40, date in gen_date()) {
        let next = next(rule(Every::Years(n)), date);
        prop_assert_eq!(next.month, date.month);
        prop_assert_eq!(next.year, date.year + i32::from(n));
        prop_assert_eq!(next.day, date.day.min(days_in_month(next.year, next.month)));
    }

    /// Every weekday lands Monday to Friday and misses nothing on the way:
    /// every day it skipped is a weekend day.
    #[test]
    fn prop_every_weekday_skips_only_the_weekend(date in gen_date()) {
        let next = next(rule(Every::Weekdays), date);
        prop_assert!(next.weekday() < 5);
        for skipped in 1..(next.to_days() - date.to_days()) {
            prop_assert!(date.add_days(skipped).weekday() >= 5);
        }
    }

    /// Every n weeks is n times seven days on, the same weekday.
    #[test]
    fn prop_every_n_weeks_is_seven_n_days(n in 1u16..=52, date in gen_date()) {
        let next = next(rule(Every::Weeks(n, WeekdaySet::EMPTY)), date);
        prop_assert_eq!(next.to_days() - date.to_days(), 7 * i64::from(n));
        prop_assert_eq!(next.weekday(), date.weekday());
    }

    /// Named weekdays land on one of them, and on every one of them in
    /// turn: nothing in the set is passed over within the week.
    #[test]
    fn prop_named_weekdays_land_on_the_set(set in (1u8..128).prop_map(WeekdaySet), date in gen_date()) {
        let next = next(rule(Every::Weeks(1, set)), date);
        prop_assert!(set.contains(next.weekday()));
        for skipped in 1..(next.to_days() - date.to_days()) {
            prop_assert!(!set.contains(date.add_days(skipped).weekday()));
        }
    }

    /// Daylight saving: dates are wall-clock, so a day rule is exactly one
    /// calendar day, a week rule seven, and the time of day does not move,
    /// on any date, the clock-change weekends included.
    #[test]
    fn prop_a_clock_change_changes_nothing(
        date in (20_000i64..21_500).prop_map(Date::from_days),
        time in (0u32..24, 0u32..60),
    ) {
        let current = Due { date, time: Some(time) };
        let day = rule(Every::Days(1)).next(&current, date);
        prop_assert_eq!(day.local_ms() - current.local_ms(), 86_400_000);
        prop_assert_eq!(day.time, Some(time));
        let week = rule(Every::Weeks(1, WeekdaySet::EMPTY)).next(&current, date);
        prop_assert_eq!(week.local_ms() - current.local_ms(), 7 * 86_400_000);
    }
}
