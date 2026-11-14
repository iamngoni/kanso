use super::*;
use crate::reminders::key_due_ms;
use crate::zoned::{from_utc_ms, parse_zoned};
use proptest::prelude::*;

/// Thursday 13 November 2026, 15:00 UTC.
const NOW: i64 = 1_794_582_000_000;

fn note(value: &str) -> String {
    format!("---\nstage: Editing\nembargo: {value}\n---\n# Pixel 11 review\n")
}

#[test]
fn the_documented_embargo_reads_as_the_studio_says() {
    let body = note("2026-11-18T09:00 America/New_York");
    let embargo = check(&body, NOW, NOW).expect("in force");
    assert_eq!(embargo.text, "2026-11-18T09:00 America/New_York");
    assert_eq!(embargo.label(), "Wed 18 Nov, 09:00 New York");
    assert_eq!(
        embargo.banner(),
        "Embargoed until Wed 18 Nov, 09:00 New York."
    );
    let ask = embargo.ask(Gate::Publish, NOW, NOW);
    assert_eq!(ask.title, "This note is under embargo");
    assert_eq!(
        ask.body,
        "Embargo lifts Wed 18 Nov, 09:00 New York (in 5 days). Publish anyway?"
    );
    assert_eq!(ask.confirm, "Publish anyway");
    assert_eq!(
        embargo.ask(Gate::ShareLink, NOW, NOW).confirm,
        "Make link anyway"
    );
    assert!(
        embargo
            .ask(Gate::Export, NOW, NOW)
            .body
            .ends_with("Export anyway?")
    );
    // 09:00 in New York is 14:00 UTC.
    let lifts = parse_zoned("2026-11-18T09:00 America/New_York")
        .unwrap()
        .utc_ms()
        .unwrap();
    assert!(check(&body, lifts - 1, lifts - 1).is_some());
    assert_eq!(check(&body, lifts, lifts), None, "lifted on the minute");
}

#[test]
fn other_values_and_notes_without_one_are_no_embargo() {
    assert_eq!(of("# No front matter\n"), None);
    assert_eq!(of(&note("soon")), None);
    assert_eq!(of(&note("2026-11-18T09:00 Mars/Olympus")), None);
    assert_eq!(of(&note("[2026-11-18]")), None);
    assert_eq!(of("---\nembargo:\n---\n"), None);
    // A plain date-time and a day lift on this device's wall clock.
    let local = of(&note("2026-11-18T09:00")).unwrap();
    assert_eq!(local.label(), "Wed 18 Nov, 09:00");
    let day = of(&note("2026-11-18")).unwrap();
    assert_eq!(day.label(), "Wed 18 Nov, 00:00");
    assert!(day.is_active(NOW, NOW));
}

#[test]
fn time_left_reads_in_the_largest_whole_unit() {
    assert_eq!(in_words(0), "now");
    assert_eq!(in_words(-5), "now");
    assert_eq!(in_words(30_000), "in under a minute");
    assert_eq!(in_words(60_000), "in 1 minute");
    assert_eq!(in_words(61_000), "in 2 minutes");
    assert_eq!(in_words(59 * 60_000), "in 59 minutes");
    assert_eq!(in_words(59 * 60_000 + 1), "in 1 hour");
    assert_eq!(in_words(5 * 3_600_000), "in 5 hours");
    assert_eq!(in_words(23 * 3_600_000 + 40 * 60_000), "in 1 day");
    assert_eq!(in_words(5 * 86_400_000 - 3_600_000), "in 5 days");
}

#[test]
fn the_lift_reminder_has_a_stable_key_and_title() {
    let embargo = of(&note("2026-11-18T09:00 America/New_York")).unwrap();
    let key = reminder_key("note:pixel", &embargo);
    assert_eq!(key, "note:pixel#embargo@2026-11-18T09:00 America/New_York");
    assert!(is_reminder_key(&key));
    assert!(!is_reminder_key(
        "note:pixel#0123456789abcdef@2026-11-18T09:00"
    ));
    assert!(key_due_ms(&key).is_some(), "old keys can be pruned");
    assert_eq!(
        lifted_title("Pixel 11 review"),
        "Embargo lifted: Pixel 11 review"
    );
    assert_eq!(lifted_title("  "), "Embargo lifted: Untitled");
    assert_eq!(due_text(0), "1970-01-01T00:00");
    assert_eq!(due_text(1), "1970-01-01T00:01");
}

fn zones() -> impl Strategy<Value = &'static str> {
    prop::sample::select(vec![
        "America/New_York",
        "Europe/London",
        "Africa/Harare",
        "Asia/Tokyo",
        "Australia/Lord_Howe",
        "UTC",
    ])
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 256,
        ..ProptestConfig::default()
    })]

    /// A zoned embargo is in force exactly while its moment is still to
    /// come, whatever the device's own offset.
    #[test]
    fn prop_a_zoned_embargo_holds_until_its_moment(
        lifts in 1_700_000_000_000i64..1_900_000_000_000,
        zone in zones(),
        before in -400_000_000i64..400_000_000,
        offset in -14i64 * 3_600_000..14 * 3_600_000,
    ) {
        let written = from_utc_ms(lifts - lifts.rem_euclid(60_000), zone).unwrap();
        let body = note(&written.text());
        let moment = written.utc_ms().unwrap();
        let now = moment - before;
        let embargo = of(&body).expect("readable");
        prop_assert_eq!(embargo.is_active(now, now + offset), before > 0);
        prop_assert_eq!(check(&body, now, now + offset).is_some(), before > 0);
        prop_assert_eq!(embargo.remaining_ms(now, now + offset), before);
        // Local time is the moment with the device's offset.
        prop_assert_eq!(embargo.lifts_local_ms(&|utc| utc + offset), Some(moment + offset));
    }

    /// An embargo with no zone lifts on the device's wall clock: only local
    /// time matters.
    #[test]
    fn prop_a_local_embargo_follows_the_wall_clock(
        minute in 28_000_000i64..32_000_000,
        before in -400_000_000i64..400_000_000,
        utc in any::<i64>(),
    ) {
        let at = DateTime::from_local_ms(minute * 60_000);
        let text = format!("{}T{:02}:{:02}", at.date.iso(), at.hour, at.minute);
        let embargo = of(&note(&text)).expect("readable");
        let local = minute * 60_000;
        prop_assert_eq!(embargo.is_active(utc, local - before), before > 0);
        prop_assert_eq!(embargo.lifts_local_ms(&|_| 0), Some(local));
        prop_assert_eq!(due_text(local), text.clone());
        prop_assert_eq!(key_due_ms(&reminder_key("note:x", &embargo)), Some(local));
    }

    /// The words for time left never shrink as more time is left, and say
    /// "now" only when none is.
    #[test]
    fn prop_time_left_words_grow_with_time(a in -1_000i64..2_000_000_000, b in -1_000i64..2_000_000_000) {
        fn rank(words: &str) -> (u8, i64) {
            let unit = if words == "now" { 0 }
                else if words == "in under a minute" { 1 }
                else if words.contains("minute") { 2 }
                else if words.contains("hour") { 3 }
                else { 4 };
            let n = words.split(' ').nth(1).and_then(|n| n.parse().ok()).unwrap_or(0);
            (unit, n)
        }
        let (short, long) = if a <= b { (a, b) } else { (b, a) };
        prop_assert!(rank(&in_words(short)) <= rank(&in_words(long)));
        prop_assert_eq!(in_words(a) == "now", a <= 0);
    }

    /// The lift notice comes on the minute the embargo lifts, never before.
    #[test]
    fn prop_the_notice_is_never_early(local in -10_000_000_000_000i64..10_000_000_000_000) {
        let due = crate::tasks::Due::parse(&due_text(local)).expect("a timed due");
        let at = due.local_ms();
        prop_assert!(at >= local && at - local < 60_000);
    }

    /// Asking always names when it lifts and how far off that is.
    #[test]
    fn prop_asking_names_the_lift(
        zone in zones(),
        ahead in 1i64..30 * 86_400_000,
        gate in prop::sample::select(vec![Gate::Publish, Gate::ShareLink, Gate::Export]),
    ) {
        let written = from_utc_ms(NOW + ahead, zone).unwrap();
        let embargo = of(&note(&written.text())).unwrap();
        let ask = embargo.ask(gate, NOW, NOW);
        prop_assert!(ask.body.contains(&embargo.label()));
        let remaining = embargo.remaining_ms(NOW, NOW);
        prop_assert!(ask.body.contains(&in_words(remaining)));
        prop_assert_eq!(ask.confirm, confirm_label(gate));
        prop_assert!(embargo.banner().contains(&embargo.label()));
    }
}
