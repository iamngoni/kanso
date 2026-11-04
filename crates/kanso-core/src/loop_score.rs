//! How sure Kanso is that a sentence is a promise, and what the person's
//! dismissals teach it (issue #240, Overview).
//!
//! The detection rules in [`crate::loops`] stay as they are. This module
//! sits beside them: it names the phrase family a sentence belongs to,
//! scores the sentence from 0 to 1, and says whether a family has been
//! dismissed often enough that Overview should stop showing its guesses.
//! Everything here is pure.

use crate::loops::LoopKind;

/// A wording guess is shown on Overview only at or above this score.
pub const SHOW_AT: f32 = 0.75;
/// How many dismissals within [`WINDOW_DAYS`] hide a family.
pub const SUPPRESS_AFTER: usize = 2;
/// The window the dismissals are counted in, in days.
pub const WINDOW_DAYS: i64 = 60;

const DAY_MS: i64 = 86_400_000;

/// The group of phrases a loop's sentence starts with.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Family {
    Will,
    Need,
    MustShould,
    FollowUp,
    Remember,
    Todo,
    Check,
    Ask,
    WaitingPhrase,
    NameThen,
}

impl Family {
    pub const ALL: [Family; 10] = [
        Family::Will,
        Family::Need,
        Family::MustShould,
        Family::FollowUp,
        Family::Remember,
        Family::Todo,
        Family::Check,
        Family::Ask,
        Family::WaitingPhrase,
        Family::NameThen,
    ];

    /// The name stored and shown.
    pub fn as_str(self) -> &'static str {
        match self {
            Family::Will => "will",
            Family::Need => "need",
            Family::MustShould => "must-should",
            Family::FollowUp => "follow-up",
            Family::Remember => "remember",
            Family::Todo => "todo",
            Family::Check => "check",
            Family::Ask => "ask",
            Family::WaitingPhrase => "waiting-phrase",
            Family::NameThen => "name-then",
        }
    }

    pub fn parse(text: &str) -> Option<Family> {
        Family::ALL.into_iter().find(|f| f.as_str() == text)
    }

    /// The phrase the banner quotes: `should …`.
    pub fn banner_phrase(self) -> &'static str {
        match self {
            Family::Will => "I'll …",
            Family::Need => "need to …",
            Family::MustShould => "should …",
            Family::FollowUp => "follow up …",
            Family::Remember => "remember to …",
            Family::Todo => "TODO …",
            Family::Check => "check whether …",
            Family::Ask => "ask …",
            Family::WaitingPhrase => "waiting on …",
            Family::NameThen => "<name> will …",
        }
    }
}

fn fold(text: &str) -> String {
    text.chars()
        .map(|c| if c == '\u{2019}' { '\'' } else { c })
        .flat_map(char::to_lowercase)
        .collect()
}

/// The sentence without bullets, quote marks and an "and"/"also" joiner.
fn words(text: &str) -> String {
    let mut rest = fold(text.trim());
    loop {
        let trimmed = rest
            .trim_start_matches([' ', '\t', '-', '*', '_', '+', '>', '"', '\u{201c}'])
            .to_owned();
        let stripped = ["and ", "also ", "then "]
            .iter()
            .find_map(|j| trimmed.strip_prefix(j))
            .map(str::to_owned);
        match stripped {
            Some(next) => rest = next,
            None => return trimmed,
        }
    }
}

fn starts_with_phrase(text: &str, phrases: &[&str]) -> bool {
    phrases.iter().any(|p| {
        text.strip_prefix(p).is_some_and(|rest| {
            p.ends_with(':') || rest.chars().next().is_none_or(|c| !c.is_alphanumeric())
        })
    })
}

/// The phrase family of a loop's `text`, given its `kind`.
pub fn family(text: &str, kind: LoopKind) -> Family {
    let w = words(text);
    if starts_with_phrase(
        &w,
        &[
            "i'm waiting on",
            "i am waiting on",
            "i'm waiting for",
            "i am waiting for",
            "still waiting on",
            "still waiting for",
            "waiting on",
            "waiting for",
            "pending from",
        ],
    ) {
        return Family::WaitingPhrase;
    }
    if starts_with_phrase(&w, &["i'll", "i will"]) {
        return Family::Will;
    }
    if starts_with_phrase(&w, &["i need to", "i have to", "need to", "have to"]) {
        return Family::Need;
    }
    if starts_with_phrase(&w, &["i must", "i should", "must", "should"]) {
        return Family::MustShould;
    }
    if starts_with_phrase(&w, &["follow up with", "follow up on"]) {
        return Family::FollowUp;
    }
    if starts_with_phrase(&w, &["remember to", "don't forget to", "do not forget to"]) {
        return Family::Remember;
    }
    if starts_with_phrase(
        &w,
        &["todo:", "todo", "fixme:", "fixme", "to do:", "action:"],
    ) {
        return Family::Todo;
    }
    if starts_with_phrase(&w, &["check whether", "check if", "find out"]) {
        return Family::Check;
    }
    if starts_with_phrase(&w, &["ask"]) {
        return Family::Ask;
    }
    match kind {
        LoopKind::Waiting => Family::NameThen,
        LoopKind::Question => Family::Check,
        LoopKind::Mine => Family::Need,
    }
}

/// What the score is made from, besides the sentence itself.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Context {
    /// The sentence names a person, a project or a date.
    pub names_something: bool,
    /// The note changed in the last 14 days.
    pub note_recent: bool,
    /// The sentence sits in a quote or a list of other people's words.
    pub quoted: bool,
    /// Its phrase family is suppressed.
    pub suppressed: bool,
}

fn base_strength(text: &str, kind: LoopKind, fam: Family) -> f32 {
    let w = words(text);
    let first_person = starts_with_phrase(
        &w,
        &[
            "i'll",
            "i will",
            "i need to",
            "i have to",
            "i must",
            "i should",
            "remember to",
            "don't forget to",
            "do not forget to",
            "follow up with",
            "follow up on",
            "todo:",
            "todo",
            "to do:",
            "action:",
        ],
    );
    if first_person {
        0.6
    } else if kind == LoopKind::Question || fam == Family::Check || fam == Family::Ask {
        0.3
    } else if matches!(fam, Family::Need | Family::MustShould) {
        0.4
    } else if kind == LoopKind::Waiting {
        0.4
    } else {
        0.3
    }
}

/// The score of a sentence, always within `0.0..=1.0`.
pub fn score(text: &str, kind: LoopKind, ctx: Context) -> f32 {
    let fam = family(text, kind);
    let mut s = base_strength(text, kind, fam);
    if ctx.names_something {
        s += 0.15;
    }
    let count = text.split_whitespace().count();
    if (5..=25).contains(&count) {
        s += 0.1;
    }
    if ctx.note_recent {
        s += 0.1;
    }
    if ctx.quoted {
        s -= 0.2;
    }
    if ctx.suppressed {
        s -= 0.3;
    }
    s.clamp(0.0, 1.0)
}

/// Whether a score is high enough for Overview's Promises line.
pub fn is_confident(score: f32) -> bool {
    score >= SHOW_AT - 1e-6
}

/// What the person did with a loop, as feedback for its family.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Feedback {
    /// "Not a loop".
    Dismissed,
    /// Ticked done: the family was right that time.
    Done,
}

impl Feedback {
    pub fn as_str(self) -> &'static str {
        match self {
            Feedback::Dismissed => "dismissed",
            Feedback::Done => "done",
        }
    }

    pub fn parse(text: &str) -> Option<Feedback> {
        match text {
            "dismissed" => Some(Feedback::Dismissed),
            "done" => Some(Feedback::Done),
            _ => None,
        }
    }
}

/// Whether a family is suppressed at `now_ms`, given its feedback as
/// `(at_ms, what)`. Only feedback in the last [`WINDOW_DAYS`] counts; each
/// *Done* cancels one dismissal. Two dismissals suppress a family.
pub fn suppressed(events: &[(i64, Feedback)], now_ms: i64) -> bool {
    let from = now_ms - WINDOW_DAYS * DAY_MS;
    let in_window = events.iter().filter(|(at, _)| *at >= from && *at <= now_ms);
    let (mut dismissed, mut done) = (0usize, 0usize);
    for (_, what) in in_window {
        match what {
            Feedback::Dismissed => dismissed += 1,
            Feedback::Done => done += 1,
        }
    }
    dismissed.saturating_sub(done) >= SUPPRESS_AFTER
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig { rng_seed: proptest::test_runner::RngSeed::Fixed(0x006b_616e_736f), ..ProptestConfig::with_cases(512) })]

        /// A score is always within 0 to 1, whatever the sentence.
        #[test]
        fn score_is_in_unit_range(
            text in ".{0,120}",
            kind in prop_oneof![Just(LoopKind::Mine), Just(LoopKind::Waiting), Just(LoopKind::Question)],
            names in any::<bool>(), recent in any::<bool>(), quoted in any::<bool>(), supp in any::<bool>(),
        ) {
            let s = score(&text, kind, Context { names_something: names, note_recent: recent, quoted, suppressed: supp });
            prop_assert!((0.0..=1.0).contains(&s));
        }

        /// Suppression turns on at exactly two dismissals inside the window
        /// and never with fewer, and old ones do not count.
        #[test]
        fn suppression_after_exactly_two_in_sixty_days(n in 0usize..6, age in 0i64..59, old in 0usize..5) {
            let now = 1_000 * DAY_MS;
            let mut events: Vec<(i64, Feedback)> =
                (0..n).map(|i| (now - (age * DAY_MS) - i as i64, Feedback::Dismissed)).collect();
            events.extend((0..old).map(|i| (now - (WINDOW_DAYS + 1 + i as i64) * DAY_MS, Feedback::Dismissed)));
            prop_assert_eq!(suppressed(&events, now), n >= SUPPRESS_AFTER);
        }

        /// A done tick takes back one dismissal.
        #[test]
        fn done_cancels_one_dismissal(dismissals in 0usize..6, dones in 0usize..6) {
            let now = 500 * DAY_MS;
            let mut events: Vec<(i64, Feedback)> = (0..dismissals).map(|i| (now - 1000 - i as i64, Feedback::Dismissed)).collect();
            events.extend((0..dones).map(|i| (now - 500 - i as i64, Feedback::Done)));
            prop_assert_eq!(suppressed(&events, now), dismissals.saturating_sub(dones) >= SUPPRESS_AFTER);
        }
    }

    #[test]
    fn families_follow_the_phrase() {
        assert_eq!(
            family("I'll send Farai the design.", LoopKind::Mine),
            Family::Will
        );
        assert_eq!(
            family("need to book the venue", LoopKind::Mine),
            Family::Need
        );
        assert_eq!(
            family("We should revisit the plan", LoopKind::Mine),
            Family::Need
        );
        assert_eq!(
            family("should call Tendai", LoopKind::Mine),
            Family::MustShould
        );
        assert_eq!(
            family("Waiting on Rudo for the quote.", LoopKind::Waiting),
            Family::WaitingPhrase
        );
        assert_eq!(family("TODO: renew passport", LoopKind::Mine), Family::Todo);
        assert_eq!(
            family("Check whether the protocol survived", LoopKind::Question),
            Family::Check
        );
        assert_eq!(
            family("Ask Tendai about MT5", LoopKind::Question),
            Family::Ask
        );
        assert_eq!(
            family("Rudo will reply by Friday.", LoopKind::Waiting),
            Family::NameThen
        );
        assert_eq!(
            family("remember to water the plants", LoopKind::Mine),
            Family::Remember
        );
        assert_eq!(
            family("follow up with Farai", LoopKind::Mine),
            Family::FollowUp
        );
    }

    #[test]
    fn first_person_with_a_name_is_confident() {
        let ctx = Context {
            names_something: true,
            note_recent: true,
            ..Context::default()
        };
        let s = score(
            "I'll send Farai the kit_capture design today.",
            LoopKind::Mine,
            ctx,
        );
        assert!(is_confident(s), "{s}");
        let weak = score("Check if it works?", LoopKind::Question, Context::default());
        assert!(!is_confident(weak));
        let hushed = score(
            "I'll send Farai the kit_capture design today.",
            LoopKind::Mine,
            Context {
                suppressed: true,
                ..ctx
            },
        );
        assert!(!is_confident(hushed), "{hushed}");
    }
}
