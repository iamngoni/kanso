use proptest::prelude::*;
use proptest::test_runner::{Config, RngAlgorithm, RngSeed};

use super::*;
use crate::dates::{Date, DateTime, MONTHS, WEEKDAYS};
use crate::due_phrase_model::DateOrder;
use crate::tasks::Due;

/// Thursday 12 November 2026, 10:30.
fn now() -> DateTime {
    DateTime {
        date: Date::new(2026, 11, 12).unwrap(),
        hour: 10,
        minute: 30,
        second: 0,
    }
}

fn due(date: Date, time: Option<(u32, u32)>) -> Due {
    Due { date, time }
}

/// Each row: the line, the phrase the parser takes from its end, and the
/// canonical token it writes. The calendar around `now()`:
///
/// ```text
/// Su Mo Tu We Th Fr Sa
/// 15 16 17 18 19 20 21   (Thu 12 is today, Fri 13, Sat 14)
/// 22 23 24 25 26 27 28
/// ```
const CORPUS: &[(&str, &str, &str)] = &[
    // the headline example and its neighbours
    ("film Pixel review fri 3pm", "fri 3pm", "@2026-11-13T15:00"),
    (
        "film Pixel review fri at 3pm",
        "fri at 3pm",
        "@2026-11-13T15:00",
    ),
    ("film Pixel review 3pm fri", "3pm fri", "@2026-11-13T15:00"),
    (
        "film Pixel review friday 3:30pm",
        "friday 3:30pm",
        "@2026-11-13T15:30",
    ),
    (
        "film Pixel review next fri 15:00",
        "next fri 15:00",
        "@2026-11-20T15:00",
    ),
    ("film Pixel review", "", ""),
    // weekdays alone: the next one after today
    ("x mon", "mon", "@2026-11-16"),
    ("x tue", "tue", "@2026-11-17"),
    ("x tues", "tues", "@2026-11-17"),
    ("x wed", "wed", "@2026-11-18"),
    ("x weds", "weds", "@2026-11-18"),
    ("x thu", "thu", "@2026-11-19"),
    ("x thur", "thur", "@2026-11-19"),
    ("x thurs", "thurs", "@2026-11-19"),
    ("x fri", "fri", "@2026-11-13"),
    ("x sat", "sat", "@2026-11-14"),
    ("x sun", "sun", "@2026-11-15"),
    ("x monday", "monday", "@2026-11-16"),
    ("x tuesday", "tuesday", "@2026-11-17"),
    ("x wednesday", "wednesday", "@2026-11-18"),
    ("x thursday", "thursday", "@2026-11-19"),
    ("x friday", "friday", "@2026-11-13"),
    ("x saturday", "saturday", "@2026-11-14"),
    ("x sunday", "sunday", "@2026-11-15"),
    ("x FRIDAY", "FRIDAY", "@2026-11-13"),
    ("x Fri", "Fri", "@2026-11-13"),
    // this <weekday>: from today on
    ("x this mon", "this mon", "@2026-11-16"),
    ("x this tue", "this tue", "@2026-11-17"),
    ("x this wed", "this wed", "@2026-11-18"),
    ("x this thu", "this thu", "@2026-11-12"),
    ("x this fri", "this fri", "@2026-11-13"),
    ("x this sat", "this sat", "@2026-11-14"),
    ("x this sunday", "this sunday", "@2026-11-15"),
    // next <weekday>: in next week, Monday to Sunday
    ("x next mon", "next mon", "@2026-11-16"),
    ("x next tue", "next tue", "@2026-11-17"),
    ("x next wed", "next wed", "@2026-11-18"),
    ("x next thu", "next thu", "@2026-11-19"),
    ("x next fri", "next fri", "@2026-11-20"),
    ("x next sat", "next sat", "@2026-11-21"),
    ("x next sun", "next sun", "@2026-11-22"),
    ("x Next Friday", "Next Friday", "@2026-11-20"),
    // today, tomorrow, tonight
    ("x today", "today", "@2026-11-12"),
    ("x tomorrow", "tomorrow", "@2026-11-13"),
    ("x tmr", "tmr", "@2026-11-13"),
    ("x TOMORROW", "TOMORROW", "@2026-11-13"),
    ("x tonight", "tonight", "@2026-11-12T20:00"),
    ("x today 9am", "today 9am", "@2026-11-12T09:00"),
    ("x today at 5pm", "today at 5pm", "@2026-11-12T17:00"),
    ("x tomorrow 9am", "tomorrow 9am", "@2026-11-13T09:00"),
    ("x tmr 8:15am", "tmr 8:15am", "@2026-11-13T08:15"),
    (
        "x tomorrow morning",
        "tomorrow morning",
        "@2026-11-13T09:00",
    ),
    (
        "x tomorrow afternoon",
        "tomorrow afternoon",
        "@2026-11-13T14:00",
    ),
    (
        "x tomorrow evening",
        "tomorrow evening",
        "@2026-11-13T18:00",
    ),
    ("x tomorrow noon", "tomorrow noon", "@2026-11-13T12:00"),
    (
        "x tomorrow midnight",
        "tomorrow midnight",
        "@2026-11-13T00:00",
    ),
    ("x sat morning", "sat morning", "@2026-11-14T09:00"),
    ("x evening sat", "evening sat", "@2026-11-14T18:00"),
    ("x mon noon", "mon noon", "@2026-11-16T12:00"),
    // in N days and weeks
    ("x in 1 day", "in 1 day", "@2026-11-13"),
    ("x in 3 days", "in 3 days", "@2026-11-15"),
    ("x in 10 days", "in 10 days", "@2026-11-22"),
    ("x in 1 week", "in 1 week", "@2026-11-19"),
    ("x in 2 weeks", "in 2 weeks", "@2026-11-26"),
    ("x in 3 weeks", "in 3 weeks", "@2026-12-03"),
    ("x in 20 days", "in 20 days", "@2026-12-02"),
    ("x in 50 days", "in 50 days", "@2027-01-01"),
    ("x IN 2 WEEKS", "IN 2 WEEKS", "@2026-11-26"),
    ("x in 2 weeks 3pm", "in 2 weeks 3pm", "@2026-11-26T15:00"),
    (
        "x in 3 days at 9:30am",
        "in 3 days at 9:30am",
        "@2026-11-15T09:30",
    ),
    ("x in 0 days", "", ""),
    ("x in days", "", ""),
    ("x in 2 months", "", ""),
    // day and month
    ("x 14 nov", "14 nov", "@2026-11-14"),
    ("x nov 14", "nov 14", "@2026-11-14"),
    ("x 14 november", "14 november", "@2026-11-14"),
    ("x November 14", "November 14", "@2026-11-14"),
    ("x 14th nov", "14th nov", "@2026-11-14"),
    ("x 1st dec", "1st dec", "@2026-12-01"),
    ("x 2nd dec", "2nd dec", "@2026-12-02"),
    ("x 3rd dec", "3rd dec", "@2026-12-03"),
    ("x dec 25th", "dec 25th", "@2026-12-25"),
    ("x 12 nov", "12 nov", "@2026-11-12"),
    ("x 11 nov", "11 nov", "@2027-11-11"),
    ("x nov 11", "nov 11", "@2027-11-11"),
    ("x 1 jan", "1 jan", "@2027-01-01"),
    ("x 31 dec", "31 dec", "@2026-12-31"),
    ("x 29 feb", "29 feb", "@2028-02-29"),
    ("x feb 29", "feb 29", "@2028-02-29"),
    ("x 30 feb", "", ""),
    ("x 31 apr", "", ""),
    ("x 0 nov", "", ""),
    ("x 32 nov", "", ""),
    ("x 14 sept", "14 sept", "@2027-09-14"),
    ("x 3 jan", "3 jan", "@2027-01-03"),
    ("x feb 3", "feb 3", "@2027-02-03"),
    ("x mar 1", "mar 1", "@2027-03-01"),
    ("x apr 2", "apr 2", "@2027-04-02"),
    ("x may 5", "may 5", "@2027-05-05"),
    ("x jun 6", "jun 6", "@2027-06-06"),
    ("x jul 7", "jul 7", "@2027-07-07"),
    ("x aug 8", "aug 8", "@2027-08-08"),
    ("x sep 9", "sep 9", "@2027-09-09"),
    ("x oct 10", "oct 10", "@2027-10-10"),
    ("x 20 nov 3pm", "20 nov 3pm", "@2026-11-20T15:00"),
    ("x nov 20 3pm", "nov 20 3pm", "@2026-11-20T15:00"),
    ("x 3pm 20 nov", "3pm 20 nov", "@2026-11-20T15:00"),
    ("x 3pm nov 20", "3pm nov 20", "@2026-11-20T15:00"),
    ("x 20 nov at 3pm", "20 nov at 3pm", "@2026-11-20T15:00"),
    ("x 20 nov 15:00", "20 nov 15:00", "@2026-11-20T15:00"),
    ("x 20 nov evening", "20 nov evening", "@2026-11-20T18:00"),
    // numeric dates, day first
    ("x 14/11", "14/11", "@2026-11-14"),
    ("x 4/12", "4/12", "@2026-12-04"),
    ("x 04/12", "04/12", "@2026-12-04"),
    ("x 1/1", "1/1", "@2027-01-01"),
    ("x 11/11", "11/11", "@2027-11-11"),
    ("x 12/11", "12/11", "@2026-11-12"),
    ("x 13/13", "", ""),
    ("x 11/14", "", ""),
    ("x 31/4", "", ""),
    ("x 14/11/2026", "14/11/2026", "@2026-11-14"),
    ("x 1/2/2030", "1/2/2030", "@2030-02-01"),
    ("x 14/11/26", "", ""),
    ("x 14/11/2026/1", "", ""),
    ("x 1/2/3/4", "", ""),
    ("x 14/11 3pm", "14/11 3pm", "@2026-11-14T15:00"),
    // iso dates and the canonical token
    ("x 2026-11-14", "2026-11-14", "@2026-11-14"),
    ("x 2027-02-28", "2027-02-28", "@2027-02-28"),
    ("x 2024-02-29", "2024-02-29", "@2024-02-29"),
    ("x 2026-02-29", "", ""),
    ("x 2026-13-01", "", ""),
    (
        "x 2026-11-14T15:00",
        "2026-11-14T15:00",
        "@2026-11-14T15:00",
    ),
    ("x @2026-11-14", "@2026-11-14", "@2026-11-14"),
    (
        "x @2026-11-14T15:00",
        "@2026-11-14T15:00",
        "@2026-11-14T15:00",
    ),
    ("x @2026-11-14T25:00", "", ""),
    ("x @2026-11-14 3pm", "@2026-11-14 3pm", "@2026-11-14T15:00"),
    ("x 2026-11-14 3pm", "2026-11-14 3pm", "@2026-11-14T15:00"),
    ("x 3pm 2026-11-14", "3pm 2026-11-14", "@2026-11-14T15:00"),
    ("x @2026-11-14T15:00 9am", "9am", "@2026-11-13T09:00"),
    // 12-hour times
    ("x 1am", "1am", "@2026-11-13T01:00"),
    ("x 9am", "9am", "@2026-11-13T09:00"),
    ("x 10am", "10am", "@2026-11-13T10:00"),
    ("x 11am", "11am", "@2026-11-12T11:00"),
    ("x 12pm", "12pm", "@2026-11-12T12:00"),
    ("x 12am", "12am", "@2026-11-13T00:00"),
    ("x 3pm", "3pm", "@2026-11-12T15:00"),
    ("x 3 pm", "3 pm", "@2026-11-12T15:00"),
    ("x 3PM", "3PM", "@2026-11-12T15:00"),
    ("x 3:30pm", "3:30pm", "@2026-11-12T15:30"),
    ("x 3:30 pm", "3:30 pm", "@2026-11-12T15:30"),
    ("x 03:30pm", "03:30pm", "@2026-11-12T15:30"),
    ("x 10:29am", "10:29am", "@2026-11-13T10:29"),
    ("x 10:30am", "10:30am", "@2026-11-13T10:30"),
    ("x 10:31am", "10:31am", "@2026-11-12T10:31"),
    ("x 11:59pm", "11:59pm", "@2026-11-12T23:59"),
    ("x 12:30am", "12:30am", "@2026-11-13T00:30"),
    ("x 13pm", "", ""),
    ("x 0pm", "", ""),
    ("x 0am", "", ""),
    ("x 3:60pm", "", ""),
    ("x 3:5pm", "", ""),
    ("x 3:005pm", "", ""),
    ("x 123pm", "", ""),
    ("x at 3pm", "at 3pm", "@2026-11-12T15:00"),
    ("x at 3 pm", "at 3 pm", "@2026-11-12T15:00"),
    // 24-hour times
    ("x 15:00", "15:00", "@2026-11-12T15:00"),
    ("x 9:00", "9:00", "@2026-11-13T09:00"),
    ("x 09:00", "09:00", "@2026-11-13T09:00"),
    ("x 00:00", "00:00", "@2026-11-13T00:00"),
    ("x 23:59", "23:59", "@2026-11-12T23:59"),
    ("x 10:30", "10:30", "@2026-11-13T10:30"),
    ("x 10:31", "10:31", "@2026-11-12T10:31"),
    ("x 24:00", "", ""),
    ("x 15:60", "", ""),
    ("x 15:0", "", ""),
    ("x 1500h", "1500h", "@2026-11-12T15:00"),
    ("x 1500 h", "1500 h", "@2026-11-12T15:00"),
    ("x 0900 h", "0900 h", "@2026-11-13T09:00"),
    ("x 0000h", "0000h", "@2026-11-13T00:00"),
    ("x 2359 h", "2359 h", "@2026-11-12T23:59"),
    ("x 2400 h", "", ""),
    ("x 1560 h", "", ""),
    ("x 15 h", "", ""),
    ("x 1500", "", ""),
    ("x fri 1500 h", "fri 1500 h", "@2026-11-13T15:00"),
    ("x fri 15:00", "fri 15:00", "@2026-11-13T15:00"),
    // named times of day alone
    ("x noon", "noon", "@2026-11-12T12:00"),
    ("x midnight", "midnight", "@2026-11-13T00:00"),
    ("x morning", "morning", "@2026-11-13T09:00"),
    ("x afternoon", "afternoon", "@2026-11-12T14:00"),
    ("x evening", "evening", "@2026-11-12T18:00"),
    ("x at noon", "at noon", "@2026-11-12T12:00"),
    ("x Noon", "Noon", "@2026-11-12T12:00"),
    // a phrase is only the end of the line, and only date or time words
    ("fri 3pm call", "", ""),
    ("call Fri about the fri shoot", "", ""),
    ("fri", "fri", "@2026-11-13"),
    ("3pm", "3pm", "@2026-11-12T15:00"),
    ("fri 3pm", "fri 3pm", "@2026-11-13T15:00"),
    ("  fri 3pm  ", "fri 3pm", "@2026-11-13T15:00"),
    ("x   fri    3pm", "fri    3pm", "@2026-11-13T15:00"),
    ("x\tfri\t3pm", "fri\t3pm", "@2026-11-13T15:00"),
    ("x\nfri 3pm", "fri 3pm", "@2026-11-13T15:00"),
    ("see you on fri", "fri", "@2026-11-13"),
    ("buy 3 apples fri", "fri", "@2026-11-13"),
    ("buy 3 apples", "", ""),
    ("chapter 12 nov 3pm", "12 nov 3pm", "@2026-11-12T15:00"),
    ("fri fri", "fri", "@2026-11-13"),
    ("fri sat", "sat", "@2026-11-14"),
    ("3pm 4pm", "4pm", "@2026-11-12T16:00"),
    ("tomorrow tomorrow", "tomorrow", "@2026-11-13"),
    ("tomorrow tonight", "tonight", "@2026-11-12T20:00"),
    ("tonight tomorrow", "tomorrow", "@2026-11-13"),
    ("fri tonight", "tonight", "@2026-11-12T20:00"),
    ("tonight 9pm", "9pm", "@2026-11-12T21:00"),
    ("fri 3pm.", "", ""),
    ("fri, 3pm", "3pm", "@2026-11-12T15:00"),
    ("(fri)", "", ""),
    ("fri!", "", ""),
    // words that are not dates
    ("", "", ""),
    ("   ", "", ""),
    ("hello", "", ""),
    ("in", "", ""),
    ("at", "", ""),
    ("next", "", ""),
    ("this", "", ""),
    ("pm", "", ""),
    ("14", "", ""),
    ("nov", "", ""),
    ("every week", "", ""),
    ("fridays", "", ""),
    ("friyay", "", ""),
    ("3 p.m.", "", ""),
    ("vendredi 15h", "", ""),
    ("morgen", "", ""),
    ("next week", "", ""),
    ("next month", "", ""),
    ("next friday please", "", ""),
    ("@", "", ""),
    ("@2026", "", ""),
    ("/", "", ""),
    ("//", "", ""),
    (":", "", ""),
    ("--", "", ""),
    ("\u{1F4A9}", "", ""),
    ("fri \u{1F4A9}", "", ""),
    ("\u{1F4A9} fri", "fri", "@2026-11-13"),
];

#[test]
fn corpus_of_phrases_at_a_pinned_now() {
    assert!(CORPUS.len() >= 200, "the corpus has {} rows", CORPUS.len());
    for (text, phrase, token) in CORPUS {
        let got = parse(text, now(), DateOrder::DayMonth);
        if phrase.is_empty() {
            assert_eq!(got, None, "{text:?} should not parse");
            continue;
        }
        let got = got.unwrap_or_else(|| panic!("{text:?} should parse"));
        assert_eq!(&text[got.span.clone()], *phrase, "phrase of {text:?}");
        assert_eq!(canonical(&got), *token, "token of {text:?}");
    }
}

#[test]
fn month_day_order_reads_the_slash_the_other_way() {
    let order = DateOrder::MonthDay;
    let day = |text| parse(text, now(), order).map(|p| p.due.date);
    assert_eq!(day("x 11/14"), Date::new(2026, 11, 14));
    assert_eq!(day("x 12/4"), Date::new(2026, 12, 4));
    assert_eq!(day("x 14/11"), None);
    assert_eq!(day("x 2/29"), Date::new(2028, 2, 29));
    assert_eq!(day("x 11/14/2026"), Date::new(2026, 11, 14));
}

#[test]
fn the_phrase_may_start_a_line_and_be_all_of_it() {
    let got = parse("fri 3pm", now(), DateOrder::DayMonth).unwrap();
    assert_eq!(got.span, 0..7);
}

#[test]
fn a_leap_day_waits_for_its_year() {
    let late = DateTime {
        date: Date::new(2096, 3, 1).unwrap(),
        ..now()
    };
    // 2100 is not a leap year, so the next 29 Feb is in 2104.
    let got = parse("x 29 feb", late, DateOrder::DayMonth).unwrap();
    assert_eq!(got.due.date, Date::new(2104, 2, 29).unwrap());
}

#[test]
fn midnight_hour_boundaries() {
    let at = |hour, minute| DateTime {
        date: Date::new(2026, 11, 12).unwrap(),
        hour,
        minute,
        second: 0,
    };
    let p = |now| parse("x 00:00", now, DateOrder::DayMonth).unwrap().due.date;
    assert_eq!(p(at(0, 0)), Date::new(2026, 11, 13).unwrap());
    assert_eq!(p(at(23, 59)), Date::new(2026, 11, 13).unwrap());
    let q = |now| parse("x 23:59", now, DateOrder::DayMonth).unwrap().due.date;
    assert_eq!(q(at(23, 58)), Date::new(2026, 11, 12).unwrap());
    assert_eq!(q(at(23, 59)), Date::new(2026, 11, 13).unwrap());
}

fn arb_date() -> impl Strategy<Value = Date> {
    (0i32..=9999, 1u32..=12, 1u32..=31)
        .prop_filter_map("a real day", |(y, m, d)| Date::new(y, m, d))
}

fn arb_now() -> impl Strategy<Value = DateTime> {
    (1900i32..=2200, 1u32..=12, 1u32..=31, 0u32..24, 0u32..60).prop_filter_map(
        "a real day",
        |(y, m, d, hour, minute)| {
            Date::new(y, m, d).map(|date| DateTime {
                date,
                hour,
                minute,
                second: 0,
            })
        },
    )
}

fn arb_order() -> impl Strategy<Value = DateOrder> {
    prop_oneof![Just(DateOrder::DayMonth), Just(DateOrder::MonthDay)]
}

/// Strings built from the words the parser knows, so random text reaches
/// the interesting paths and not only the first rejected word.
fn arb_phrase_soup() -> impl Strategy<Value = String> {
    let word = prop_oneof![
        Just("fri".to_owned()),
        Just("next".to_owned()),
        Just("this".to_owned()),
        Just("in".to_owned()),
        Just("at".to_owned()),
        Just("days".to_owned()),
        Just("weeks".to_owned()),
        Just("nov".to_owned()),
        Just("tonight".to_owned()),
        Just("noon".to_owned()),
        Just("pm".to_owned()),
        Just("am".to_owned()),
        Just("h".to_owned()),
        Just("@".to_owned()),
        "[0-9]{1,4}",
        "[0-9]{1,2}:[0-9]{1,3}",
        "[0-9]{1,2}/[0-9]{1,2}(/[0-9]{1,5})?",
        "[0-9]{1,5}-[0-9]{1,3}-[0-9]{1,3}(T[0-9]{1,3}:[0-9]{1,3})?",
        "[0-9]{1,3}(st|nd|rd|th|am|pm|h)",
        "\\PC{1,6}",
    ];
    proptest::collection::vec(
        (
            word,
            prop_oneof![Just(" "), Just("  "), Just("\t"), Just("")],
        ),
        0..8,
    )
    .prop_map(|parts| parts.into_iter().map(|(w, s)| format!("{w}{s}")).collect())
}

proptest! {
    #![proptest_config(Config {
        cases: 512,
        rng_seed: RngSeed::Fixed(0x0064_6174_6573),
        rng_algorithm: RngAlgorithm::ChaCha,
        failure_persistence: None,
        ..Config::default()
    })]

    /// What `canonical` writes parses back to the same day and time, as the
    /// whole of the text, whatever `now` and the date order are.
    #[test]
    fn canonical_round_trips(
        date in arb_date(),
        time in proptest::option::of((0u32..24, 0u32..60)),
        now in arb_now(),
        order in arb_order(),
        lead in "[a-z ]{0,12}",
    ) {
        let parsed = Parsed { span: 0..0, due: due(date, time) };
        let token = canonical(&parsed);
        let again = parse(&token, now, order).expect("canonical parses");
        prop_assert_eq!(again.due, parsed.due);
        prop_assert_eq!(&again.span, &(0..token.len()));
        prop_assert_eq!(canonical(&again), token.clone());

        // After words of its own, the token is still found at the end.
        let line = format!("{lead}x {token}");
        let found = parse(&line, now, order).expect("canonical parses after text");
        prop_assert_eq!(found.due, parsed.due);
        prop_assert_eq!(&line[found.span], token.as_str());
    }

    /// Arbitrary text never panics, and a phrase it finds is a real slice
    /// of the input that ends the line and parses to the same thing alone.
    #[test]
    fn arbitrary_text_never_panics_and_spans_are_sound(
        text in prop_oneof![arb_phrase_soup(), "\\PC{0,40}", "[ -~\\s]{0,60}"],
        now in arb_now(),
        order in arb_order(),
    ) {
        if let Some(found) = parse(&text, now, order) {
            prop_assert!(found.span.start < found.span.end);
            prop_assert!(text.is_char_boundary(found.span.start));
            prop_assert!(text.is_char_boundary(found.span.end));
            let phrase = &text[found.span.clone()];
            prop_assert!(text[found.span.end..].trim().is_empty());
            prop_assert!(!phrase.starts_with(char::is_whitespace));
            prop_assert!(!phrase.ends_with(char::is_whitespace));
            let alone = parse(phrase, now, order).expect("the phrase parses alone");
            prop_assert_eq!(alone.due, found.due);
            prop_assert_eq!(alone.span, 0..phrase.len());
            // Text before the phrase never changes what it means.
            let prefixed = format!("zzz {text}");
            let again = parse(&prefixed, now, order).expect("still found");
            prop_assert_eq!(again.due, found.due);
        }
    }

    /// A weekday on its own is strictly after today and within a week;
    /// "this" is today or later within a week; "next" is in the following
    /// Monday-to-Sunday week. All name the weekday that was asked for.
    #[test]
    fn weekday_words_land_on_the_right_day(
        now in arb_now(),
        wd in 0usize..7,
        long in any::<bool>(),
    ) {
        let name = if long {
            WEEKDAYS[wd].to_lowercase()
        } else {
            WEEKDAYS[wd][..3].to_lowercase()
        };
        let today = now.date;
        let day = |text: String| parse(&text, now, DateOrder::DayMonth).unwrap().due.date;

        let plain = day(format!("x {name}"));
        prop_assert_eq!(plain.weekday(), wd);
        prop_assert!((1..=7).contains(&(plain.to_days() - today.to_days())));

        let this = day(format!("x this {name}"));
        prop_assert_eq!(this.weekday(), wd);
        prop_assert!((0..=6).contains(&(this.to_days() - today.to_days())));

        let next = day(format!("x next {name}"));
        prop_assert_eq!(next.weekday(), wd);
        let monday_next = today.add_days(7 - today.weekday() as i64);
        prop_assert!(next >= monday_next && next <= monday_next.add_days(6));
    }

    /// The same day written as "14 nov", "nov 14", "14th nov" and "14/11"
    /// (or "11/14" in month-first order) is one day: the next turn of that
    /// date, today included, and never in the past.
    #[test]
    fn one_day_in_every_written_form(
        now in arb_now(),
        month in 1u32..=12,
        day in 1u32..=28,
    ) {
        let name = MONTHS[month as usize - 1];
        let long = name.to_lowercase();
        let short = &long[..3];
        let mut forms = vec![
            (format!("x {day} {short}"), DateOrder::DayMonth),
            (format!("x {short} {day}"), DateOrder::DayMonth),
            (format!("x {day} {long}"), DateOrder::DayMonth),
            (format!("x {long} {day}"), DateOrder::DayMonth),
            (format!("x {day}/{month}"), DateOrder::DayMonth),
            (format!("x {month}/{day}"), DateOrder::MonthDay),
            (format!("x {day:02}/{month:02}"), DateOrder::DayMonth),
        ];
        forms.push((format!("x {day}{} {short}", ordinal(day)), DateOrder::DayMonth));
        let want = parse(&forms[0].0, now, forms[0].1).unwrap().due.date;
        prop_assert!(want >= now.date);
        prop_assert_eq!((want.month, want.day), (month, day));
        prop_assert!(want.to_days() - now.date.to_days() < 366);
        for (text, order) in &forms {
            let got = parse(text, now, *order);
            prop_assert_eq!(got.map(|p| p.due), Some(due(want, None)), "{}", text);
        }
    }

    /// A time written on a 12-hour clock, on a 24-hour clock and as
    /// "1500 h" is the same time of day.
    #[test]
    fn one_time_in_every_written_form(
        now in arb_now(),
        hour in 0u32..24,
        minute in 0u32..60,
        spaced in any::<bool>(),
    ) {
        let h12 = if hour % 12 == 0 { 12 } else { hour % 12 };
        let ampm = if hour >= 12 { "pm" } else { "am" };
        let gap = if spaced { " " } else { "" };
        let mut forms = vec![
            format!("x {hour}:{minute:02}"),
            format!("x {hour:02}:{minute:02}"),
            format!("x {hour:02}{minute:02}{gap}h"),
            format!("x {h12}:{minute:02}{gap}{ampm}"),
            format!("x at {h12}:{minute:02}{gap}{ampm}"),
            format!("x {h12}:{minute:02}{gap}{}", ampm.to_uppercase()),
        ];
        if minute == 0 {
            forms.push(format!("x {h12}{gap}{ampm}"));
        }
        let want = Some((hour, minute));
        let first = parse(&forms[0], now, DateOrder::DayMonth).unwrap();
        prop_assert_eq!(first.due.time, want);
        // With no day, the time is the next time it is that time of day.
        let later = (hour, minute) > (now.hour, now.minute);
        let expect_day = if later { now.date } else { now.date.add_days(1) };
        prop_assert_eq!(first.due.date, expect_day);
        for text in &forms {
            let got = parse(text, now, DateOrder::DayMonth).map(|p| p.due);
            prop_assert_eq!(got, Some(first.due), "{}", text);
        }
    }

    /// "in N days" and "in N weeks" count from today.
    #[test]
    fn in_n_counts_from_today(now in arb_now(), n in 1u32..=999) {
        let days = parse(&format!("x in {n} days"), now, DateOrder::DayMonth).unwrap();
        prop_assert_eq!(days.due.date, now.date.add_days(i64::from(n)));
        let weeks = parse(&format!("x in {n} weeks"), now, DateOrder::DayMonth).unwrap();
        prop_assert_eq!(weeks.due.date, now.date.add_days(i64::from(n) * 7));
        prop_assert_eq!(days.due.time, None);
    }

    /// Order does not matter between a day and a time.
    #[test]
    fn day_and_time_in_either_order(
        now in arb_now(),
        wd in 0usize..7,
        hour in 0u32..24,
        minute in 0u32..60,
    ) {
        let name = WEEKDAYS[wd][..3].to_lowercase();
        let time = format!("{hour:02}:{minute:02}");
        let a = parse(&format!("x {name} {time}"), now, DateOrder::DayMonth).unwrap();
        let b = parse(&format!("x {time} {name}"), now, DateOrder::DayMonth).unwrap();
        prop_assert_eq!(a.due, b.due);
        prop_assert_eq!(a.due.time, Some((hour, minute)));
    }
}

fn ordinal(day: u32) -> &'static str {
    match (day % 10, day % 100) {
        (_, 11..=13) => "th",
        (1, _) => "st",
        (2, _) => "nd",
        (3, _) => "rd",
        _ => "th",
    }
}
