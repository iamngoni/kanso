//! The lede: the one or two sentences under Overview's greeting. A pure
//! function of the day's counts, the same on every platform, never AI.
//!
//! Today's lede spells out one to nine ("Three meetings and four tasks
//! today, one of them two days late."). The week's lede keeps digits inside
//! a sentence and spells a number at the start ("Eleven meetings, eight
//! with notes."). Plurals agree and zero parts drop out.

use super::model::clock;

const WORDS: [&str; 21] = [
    "zero", "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten",
    "eleven", "twelve", "thirteen", "fourteen", "fifteen", "sixteen", "seventeen", "eighteen",
    "nineteen", "twenty",
];

/// `n` in words when it is small, in digits otherwise; capitalised at the
/// start of a sentence.
pub fn spell(n: u32, max_word: u32, start: bool) -> String {
    let text = if n <= max_word && (n as usize) < WORDS.len() {
        WORDS[n as usize].to_owned()
    } else {
        n.to_string()
    };
    if start {
        capital(&text)
    } else {
        text
    }
}

fn capital(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn plural<'a>(n: u32, one: &'a str, many: &'a str) -> &'a str {
    if n == 1 { one } else { many }
}

/// What is coming next today.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Next {
    /// Later than an hour from now: `Next: Call with Tendai at 11:30.`
    At { title: String, at_min: u32 },
    /// Within the hour: `Next: Design review in 20 minutes.`
    In { title: String, minutes: u32 },
    /// Happening now: `Now: Call with Tendai, until 12:00.`
    Now { title: String, until_min: u32 },
}

/// The counts the lede is written from.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TodayFacts {
    pub meetings: u32,
    /// Tasks due today and overdue ones.
    pub tasks: u32,
    /// How many of those are overdue, and how late the oldest is.
    pub late: u32,
    pub oldest_late_days: u32,
    pub next: Option<Next>,
    /// *Pick up where you left off* has something.
    pub pick_up: bool,
}

fn days_phrase(days: u32) -> String {
    if days == 1 {
        "a day".to_owned()
    } else {
        format!("{} days", spell(days, 9, false))
    }
}

/// Today's lede.
pub fn today_lede(f: &TodayFacts) -> String {
    let mut parts = Vec::new();
    if f.meetings > 0 {
        parts.push(format!(
            "{} {}",
            spell(f.meetings, 9, false),
            plural(f.meetings, "meeting", "meetings")
        ));
    }
    if f.tasks > 0 {
        parts.push(format!(
            "{} {}",
            spell(f.tasks, 9, false),
            plural(f.tasks, "task", "tasks")
        ));
    }
    if parts.is_empty() {
        let mut text = "Nothing on today.".to_owned();
        if f.pick_up {
            text.push_str(" Pick up where you left off below.");
        }
        return text;
    }
    let mut first = format!("{} today", parts.join(" and "));
    if f.late > 0 && f.tasks > 0 {
        if f.late == 1 {
            first.push_str(&format!(", one of them {} late", days_phrase(f.oldest_late_days)));
        } else if f.late >= f.tasks {
            first.push_str(", all of them late");
        } else {
            first.push_str(&format!(", {} of them late", spell(f.late, 9, false)));
        }
    }
    first.push('.');
    let first = capital(&first);
    match &f.next {
        Some(Next::At { title, at_min }) => {
            format!("{first} Next: {title} at {}.", clock(*at_min))
        }
        Some(Next::In { title, minutes: 0 }) => format!("{first} Next: {title}, starting now."),
        Some(Next::In { title, minutes }) => format!(
            "{first} Next: {title} in {minutes} {}.",
            plural(*minutes, "minute", "minutes")
        ),
        Some(Next::Now { title, until_min }) => {
            format!("{first} Now: {title}, until {}.", clock(*until_min))
        }
        None => first,
    }
}

/// What the lede for a week is written from.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WeekFacts {
    pub wrote: u32,
    pub changed: u32,
    pub finished: u32,
    /// The notebooks and projects the week went into most, up to two.
    pub top: Vec<String>,
    /// Calendar meetings in the week, and how many have a note; `None` when
    /// no calendar is on.
    pub meetings: Option<(u32, u32)>,
}

fn join_names(names: &[String]) -> String {
    match names {
        [] => String::new(),
        [one] => one.clone(),
        [a, b] => format!("{a} and {b}"),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    }
}

/// The week's lede.
pub fn week_lede(f: &WeekFacts) -> String {
    let mut sentences = Vec::new();
    let mut did = Vec::new();
    if f.wrote > 0 {
        did.push(format!("wrote {} {}", f.wrote, plural(f.wrote, "note", "notes")));
    }
    if f.changed > 0 {
        let noun = if f.wrote > 0 {
            String::new()
        } else {
            format!(" {}", plural(f.changed, "note", "notes"))
        };
        did.push(format!("changed {}{noun}", f.changed));
    }
    if f.finished > 0 {
        did.push(format!(
            "finished {} {}",
            f.finished,
            plural(f.finished, "task", "tasks")
        ));
    }
    if did.is_empty() {
        sentences.push("A quiet week.".to_owned());
    } else {
        let joined = match did.as_slice() {
            [one] => one.clone(),
            [a, b] => format!("{a} and {b}"),
            [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
            [] => String::new(),
        };
        sentences.push(format!("You {joined}."));
    }
    if !f.top.is_empty() && !did.is_empty() {
        let top: Vec<String> = f.top.iter().take(2).cloned().collect();
        sentences.push(format!("Most of it went into {}.", join_names(&top)));
    }
    if let Some((meetings, with_notes)) = f.meetings
        && meetings > 0
    {
        let first = format!(
            "{} {}",
            spell(meetings, 20, true),
            plural(meetings, "meeting", "meetings")
        );
        if with_notes > 0 {
            sentences.push(format!("{first}, {} with notes.", spell(with_notes, 20, false)));
        } else {
            sentences.push(format!("{first}."));
        }
    }
    sentences.join(" ")
}

/// The greeting for an hour of the day (0 to 23).
pub fn greeting(hour: u32) -> &'static str {
    match hour {
        5..=11 => "Good morning",
        12..=17 => "Good afternoon",
        _ => "Good evening",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn facts(meetings: u32, tasks: u32, late: u32, days: u32) -> TodayFacts {
        TodayFacts { meetings, tasks, late, oldest_late_days: days, ..TodayFacts::default() }
    }

    #[test]
    fn the_spec_examples() {
        let mut f = facts(3, 4, 1, 2);
        f.next = Some(Next::At { title: "Call with Tendai".into(), at_min: 11 * 60 + 30 });
        assert_eq!(
            today_lede(&f),
            "Three meetings and four tasks today, one of them two days late. Next: Call with Tendai at 11:30."
        );
        assert_eq!(today_lede(&facts(0, 4, 0, 0)), "Four tasks today.");
        assert_eq!(today_lede(&facts(2, 0, 0, 0)), "Two meetings today.");
        assert_eq!(today_lede(&facts(0, 0, 0, 0)), "Nothing on today.");
        let mut quiet = facts(0, 0, 0, 0);
        quiet.pick_up = true;
        assert_eq!(today_lede(&quiet), "Nothing on today. Pick up where you left off below.");
        let mut soon = facts(1, 0, 0, 0);
        soon.next = Some(Next::In { title: "Design review".into(), minutes: 20 });
        assert_eq!(today_lede(&soon), "One meeting today. Next: Design review in 20 minutes.");
        let mut now = facts(1, 0, 0, 0);
        now.next = Some(Next::Now { title: "Call with Tendai".into(), until_min: 720 });
        assert_eq!(today_lede(&now), "One meeting today. Now: Call with Tendai, until 12:00.");
        assert_eq!(today_lede(&facts(0, 1, 1, 1)), "One task today, one of them a day late.");
        assert_eq!(today_lede(&facts(0, 12, 0, 0)), "12 tasks today.");
    }

    #[test]
    fn the_week_examples() {
        let f = WeekFacts {
            wrote: 16,
            changed: 3,
            finished: 11,
            top: vec!["Kanso".into(), "S4VO".into()],
            meetings: Some((11, 8)),
        };
        assert_eq!(
            week_lede(&f),
            "You wrote 16 notes, changed 3 and finished 11 tasks. Most of it went into Kanso and S4VO. Eleven meetings, eight with notes."
        );
        assert_eq!(week_lede(&WeekFacts::default()), "A quiet week.");
        let only = WeekFacts { changed: 1, ..WeekFacts::default() };
        assert_eq!(week_lede(&only), "You changed 1 note.");
    }

    proptest! {
        #![proptest_config(ProptestConfig { rng_seed: proptest::test_runner::RngSeed::Fixed(0x006b_616e_736f), ..ProptestConfig::with_cases(512) })]

        /// Today's lede is at most two sentences, starts with a capital,
        /// ends with a full stop, and says nothing about a count of zero.
        #[test]
        fn today_lede_is_short_and_agrees(
            meetings in 0u32..30, tasks in 0u32..30, late in 0u32..30, days in 1u32..40,
            next in prop::option::of((0u32..1440, 0u32..60, 0usize..3)), pick in any::<bool>(),
        ) {
            let late = late.min(tasks);
            let mut f = facts(meetings, tasks, late, days);
            f.pick_up = pick;
            f.next = next.map(|(at, minutes, which)| match which {
                0 => Next::At { title: "X".into(), at_min: at },
                1 => Next::In { title: "X".into(), minutes },
                _ => Next::Now { title: "X".into(), until_min: at },
            });
            let text = today_lede(&f);
            let first = text.chars().next().unwrap();
            prop_assert!(first.is_uppercase() || first.is_ascii_digit());
            prop_assert!(text.ends_with('.'));
            prop_assert!(text.matches(". ").count() <= 2);
            prop_assert!(!text.contains("zero") && !text.contains(" 0 "));
            if meetings == 1 { prop_assert!(!text.contains("one meetings")); }
            if tasks == 1 { prop_assert!(!text.contains("one tasks")); }
            if meetings > 1 { prop_assert!(!text.contains(" meeting ")); }
            if tasks > 1 { prop_assert!(!text.contains(" task ")); }
            if meetings == 0 && tasks == 0 { prop_assert!(text.starts_with("Nothing on today.")); }
        }

        /// The week's lede never mentions a zero.
        #[test]
        fn week_lede_drops_zero_parts(wrote in 0u32..50, changed in 0u32..50, finished in 0u32..50, m in prop::option::of((0u32..30, 0u32..30))) {
            let f = WeekFacts { wrote, changed, finished, top: vec!["A".into()], meetings: m };
            let text = week_lede(&f);
            prop_assert!(text.ends_with('.'));
            prop_assert!(!text.contains(" 0 ") && !text.contains("zero"));
        }
    }
}
