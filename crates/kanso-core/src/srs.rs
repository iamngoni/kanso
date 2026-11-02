//! Spaced repetition: FSRS-5 with the default weights.
//!
//! A card's state is never stored as the source of truth. It is replayed
//! from the log of answers ([`Review`]), sorted by `(at_ms, id)`, so two
//! devices that reviewed offline and merged their logs by union reach the
//! same state. Everything here is pure.

use serde::{Deserialize, Serialize};

pub const DAY_MS: i64 = 86_400_000;
const MINUTE_MS: i64 = 60_000;

/// FSRS-5 default weights.
const W: [f64; 19] = [
    0.40255, 1.18385, 3.173, 15.69105, 7.1949, 0.5345, 1.4604, 0.0046, 1.54575, 0.1192, 1.01925,
    1.9395, 0.11, 0.29605, 2.2698, 0.2315, 2.9898, 0.51655, 0.6621,
];
const DECAY: f64 = -0.5;
const FACTOR: f64 = 19.0 / 81.0;

/// Retention a card is scheduled for when settings say nothing.
pub const DEFAULT_RETENTION: f64 = 0.90;
/// The range settings may choose a target recall from.
pub const RETENTION_RANGE: (f64, f64) = (0.80, 0.97);
/// Short-term steps inside a session, before intervals count in days.
const LEARN_STEPS_MS: [i64; 2] = [MINUTE_MS, 10 * MINUTE_MS];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Grade {
    Again = 1,
    Hard = 2,
    Good = 3,
    Easy = 4,
}

impl Grade {
    pub const ALL: [Grade; 4] = [Grade::Again, Grade::Hard, Grade::Good, Grade::Easy];

    pub fn from_number(n: i64) -> Option<Self> {
        Self::ALL.into_iter().find(|g| *g as i64 == n)
    }

    pub fn number(self) -> u8 {
        self as u8
    }
}

/// One answer in the log. `id` is unique per answer; the log merges by it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Review {
    pub id: String,
    pub card: String,
    pub at_ms: i64,
    pub grade: Grade,
    pub device: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    New,
    /// Inside the short-term steps of a session.
    Learning,
    Review,
    /// Forgotten and being relearned.
    Relearning,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct CardState {
    pub stability: f64,
    pub difficulty: f64,
    pub due_ms: i64,
    pub last_ms: i64,
    pub reps: u32,
    pub lapses: u32,
    pub phase: Phase,
    /// Position in the short-term steps (0 = first).
    pub step: u8,
}

impl CardState {
    pub fn new() -> Self {
        CardState {
            stability: 0.0,
            difficulty: 0.0,
            due_ms: 0,
            last_ms: 0,
            reps: 0,
            lapses: 0,
            phase: Phase::New,
            step: 0,
        }
    }
}

impl Default for CardState {
    fn default() -> Self {
        Self::new()
    }
}

/// The retention to use: `retention` limited to the allowed range.
pub fn clamp_retention(retention: f64) -> f64 {
    if retention.is_nan() {
        return DEFAULT_RETENTION;
    }
    retention.clamp(RETENTION_RANGE.0, RETENTION_RANGE.1)
}

fn clamp_d(d: f64) -> f64 {
    d.clamp(1.0, 10.0)
}

fn init_stability(g: Grade) -> f64 {
    W[g as usize - 1].max(0.1)
}

fn init_difficulty(g: Grade) -> f64 {
    clamp_d(W[4] - (W[5] * (g as i32 as f64 - 1.0)).exp() + 1.0)
}

/// Probability of recall after `elapsed_days` at `stability`.
pub fn retrievability(elapsed_days: f64, stability: f64) -> f64 {
    (1.0 + FACTOR * elapsed_days.max(0.0) / stability.max(0.01)).powf(DECAY)
}

/// Days until recall falls to `retention`.
fn interval_days(stability: f64, retention: f64) -> f64 {
    stability / FACTOR * (retention.powf(1.0 / DECAY) - 1.0)
}

fn next_difficulty(d: f64, g: Grade) -> f64 {
    let delta = -W[6] * (g as i32 as f64 - 3.0);
    let damped = d + delta * (10.0 - d) / 9.0;
    // Mean reversion toward the difficulty of an "Easy" first answer.
    clamp_d(W[7] * init_difficulty(Grade::Easy) + (1.0 - W[7]) * damped)
}

fn recall_stability(d: f64, s: f64, r: f64, g: Grade) -> f64 {
    let hard = if g == Grade::Hard { W[15] } else { 1.0 };
    let easy = if g == Grade::Easy { W[16] } else { 1.0 };
    s * (1.0
        + W[8].exp() * (11.0 - d) * s.powf(-W[9]) * (((1.0 - r) * W[10]).exp() - 1.0) * hard * easy)
}

fn forget_stability(d: f64, s: f64, r: f64) -> f64 {
    let v = W[11] * d.powf(-W[12]) * ((s + 1.0).powf(W[13]) - 1.0) * ((1.0 - r) * W[14]).exp();
    v.min(s).max(0.01)
}

fn short_term_stability(s: f64, g: Grade) -> f64 {
    let v = s * (W[17] * (g as i32 as f64 - 3.0 + W[18])).exp();
    if matches!(g, Grade::Good | Grade::Easy) {
        v.max(s)
    } else {
        v
    }
}

fn whole_days(days: f64) -> i64 {
    days.round().max(1.0) as i64
}

/// The state after one answer at `at_ms`.
pub fn step(state: &CardState, at_ms: i64, grade: Grade, retention: f64) -> CardState {
    let retention = clamp_retention(retention);
    let mut next = *state;
    next.reps += 1;
    next.last_ms = at_ms;
    match state.phase {
        Phase::New => {
            next.stability = init_stability(grade);
            next.difficulty = init_difficulty(grade);
            into_steps(&mut next, at_ms, grade, retention, Phase::Learning);
        }
        Phase::Learning | Phase::Relearning => {
            let same_day = at_ms - state.last_ms < DAY_MS;
            if same_day {
                next.stability = short_term_stability(state.stability, grade);
            }
            next.difficulty = next_difficulty(state.difficulty, grade);
            let phase = state.phase;
            into_steps(&mut next, at_ms, grade, retention, phase);
        }
        Phase::Review => {
            let elapsed = (at_ms - state.last_ms).max(0) as f64 / DAY_MS as f64;
            let r = retrievability(elapsed, state.stability);
            next.difficulty = next_difficulty(state.difficulty, grade);
            if grade == Grade::Again {
                next.lapses += 1;
                next.stability = forget_stability(state.difficulty, state.stability, r);
                next.phase = Phase::Relearning;
                next.step = 0;
                next.due_ms = at_ms + LEARN_STEPS_MS[0];
            } else {
                next.stability = recall_stability(state.difficulty, state.stability, r, grade);
                graduate(&mut next, at_ms, retention);
            }
        }
    }
    next
}

/// Move through the short-term steps; Good on the last step graduates.
fn into_steps(next: &mut CardState, at_ms: i64, grade: Grade, retention: f64, phase: Phase) {
    let last_step = LEARN_STEPS_MS.len() as u8 - 1;
    let from = if phase == Phase::New { 0 } else { next.step };
    match grade {
        Grade::Again => {
            next.phase = phase_or_learning(phase);
            next.step = 0;
            next.due_ms = at_ms + LEARN_STEPS_MS[0];
        }
        Grade::Hard => {
            next.phase = phase_or_learning(phase);
            next.step = from.min(last_step);
            next.due_ms = at_ms + LEARN_STEPS_MS[next.step as usize];
        }
        Grade::Good if from < last_step => {
            next.phase = phase_or_learning(phase);
            next.step = from + 1;
            next.due_ms = at_ms + LEARN_STEPS_MS[next.step as usize];
        }
        Grade::Good | Grade::Easy => graduate(next, at_ms, retention),
    }
}

fn phase_or_learning(phase: Phase) -> Phase {
    if phase == Phase::New {
        Phase::Learning
    } else {
        phase
    }
}

fn graduate(next: &mut CardState, at_ms: i64, retention: f64) {
    next.phase = Phase::Review;
    next.step = 0;
    let days = whole_days(interval_days(next.stability, retention));
    next.due_ms = at_ms + days * DAY_MS;
}

/// The state after replaying `reviews` (any order; sorted by `(at_ms, id)`
/// here). A card with no reviews is new.
pub fn replay(reviews: &[Review], retention: f64) -> CardState {
    let mut sorted: Vec<&Review> = reviews.iter().collect();
    sorted.sort_by(|a, b| (a.at_ms, &a.id).cmp(&(b.at_ms, &b.id)));
    sorted.into_iter().fold(CardState::new(), |state, r| {
        step(&state, r.at_ms, r.grade, retention)
    })
}

/// When the card falls due next for each of Again, Hard, Good and Easy,
/// as the number of milliseconds from `now_ms`.
pub fn preview(state: &CardState, now_ms: i64, retention: f64) -> [i64; 4] {
    let mut out = [0; 4];
    for (slot, grade) in out.iter_mut().zip(Grade::ALL) {
        *slot = step(state, now_ms, grade, retention).due_ms - now_ms;
    }
    // Longer answers never schedule sooner than shorter ones.
    for i in 1..4 {
        out[i] = out[i].max(out[i - 1]);
    }
    out
}

/// "10 min", "4 days", "2 months" for a span of milliseconds.
pub fn interval_label(ms: i64) -> String {
    let minutes = ms / MINUTE_MS;
    if minutes < 1 {
        return "now".into();
    }
    if minutes < 60 {
        return format!("{minutes} min");
    }
    let days = (ms as f64 / DAY_MS as f64).round() as i64;
    if days < 1 {
        return format!("{} h", minutes / 60);
    }
    if days < 31 {
        return format!("{days} {}", if days == 1 { "day" } else { "days" });
    }
    if days < 365 {
        let months = (days as f64 / 30.4).round() as i64;
        return format!("{months} {}", if months == 1 { "month" } else { "months" });
    }
    let years = days as f64 / 365.0;
    format!("{years:.1} years")
}

/// A mark in the log that is not an answer: suspend, resume or reset.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MarkKind {
    Suspend,
    Resume,
    Reset,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mark {
    pub id: String,
    pub card: String,
    pub at_ms: i64,
    pub kind: MarkKind,
}

/// Whether a card is suspended: its latest suspend/resume/reset mark (by
/// `(at_ms, id)`) is a suspend.
pub fn is_suspended(marks: &[Mark], card: &str) -> bool {
    marks
        .iter()
        .filter(|m| m.card == card && m.kind != MarkKind::Reset)
        .max_by(|a, b| (a.at_ms, &a.id).cmp(&(b.at_ms, &b.id)))
        .is_some_and(|m| m.kind == MarkKind::Suspend)
}

/// The reviews of `card` that count: those after its latest reset mark.
pub fn since_reset<'a>(reviews: &'a [Review], marks: &[Mark], card: &str) -> Vec<&'a Review> {
    let reset = marks
        .iter()
        .filter(|m| m.card == card && m.kind == MarkKind::Reset)
        .map(|m| m.at_ms)
        .max();
    reviews
        .iter()
        .filter(|r| r.card == card && reset.is_none_or(|t| r.at_ms > t))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn rv(id: u32, at: i64, g: Grade) -> Review {
        Review {
            id: format!("r{id:04}"),
            card: "c".into(),
            at_ms: at,
            grade: g,
            device: "d".into(),
        }
    }

    #[test]
    fn first_answers_follow_the_published_initial_values() {
        // FSRS-5: S0 is w[grade-1]; D0 = w4 - exp(w5 (g-1)) + 1.
        assert!((init_stability(Grade::Good) - 3.173).abs() < 1e-9);
        assert!((init_difficulty(Grade::Good) - 5.2824).abs() < 1e-2);
        assert!((init_difficulty(Grade::Again) - 7.1949).abs() < 1e-3);
        assert!((init_difficulty(Grade::Easy) - 3.2245).abs() < 1e-2);
    }

    #[test]
    fn retrievability_is_ninety_percent_at_the_stability() {
        assert!((retrievability(5.0, 5.0) - 0.9).abs() < 1e-9);
        assert!((interval_days(5.0, 0.9) - 5.0).abs() < 1e-9);
    }

    #[test]
    fn a_new_card_learns_in_steps_then_graduates() {
        let s = step(&CardState::new(), 0, Grade::Good, 0.9);
        assert_eq!((s.phase, s.due_ms), (Phase::Learning, 10 * MINUTE_MS));
        let s = step(&s, 10 * MINUTE_MS, Grade::Good, 0.9);
        assert_eq!(s.phase, Phase::Review);
        assert_eq!(s.due_ms - 10 * MINUTE_MS, 4 * DAY_MS);
        let easy = step(&CardState::new(), 0, Grade::Easy, 0.9);
        assert_eq!(easy.phase, Phase::Review);
        assert_eq!(easy.due_ms, 16 * DAY_MS);
    }

    #[test]
    fn a_lapse_relearns_in_minutes() {
        let s = replay(
            &[rv(1, 0, Grade::Easy), rv(2, 16 * DAY_MS, Grade::Again)],
            0.9,
        );
        assert_eq!(s.phase, Phase::Relearning);
        assert_eq!(s.lapses, 1);
        assert_eq!(s.due_ms, 16 * DAY_MS + MINUTE_MS);
    }

    #[test]
    fn preview_labels_read_well() {
        assert_eq!(interval_label(10 * MINUTE_MS), "10 min");
        assert_eq!(interval_label(DAY_MS), "1 day");
        assert_eq!(interval_label(4 * DAY_MS), "4 days");
        assert_eq!(interval_label(90 * DAY_MS), "3 months");
    }

    #[test]
    fn suspend_and_reset_marks() {
        let m = |id: &str, at, kind| Mark {
            id: id.into(),
            card: "c".into(),
            at_ms: at,
            kind,
        };
        assert!(!is_suspended(&[], "c"));
        assert!(is_suspended(&[m("a", 1, MarkKind::Suspend)], "c"));
        assert!(!is_suspended(
            &[m("a", 1, MarkKind::Suspend), m("b", 2, MarkKind::Resume)],
            "c"
        ));
        let log = [rv(1, 5, Grade::Good), rv(2, 20, Grade::Good)];
        assert_eq!(since_reset(&log, &[m("x", 10, MarkKind::Reset)], "c").len(), 1);
    }

    fn grade() -> impl Strategy<Value = Grade> {
        prop::sample::select(Grade::ALL.to_vec())
    }

    fn log() -> impl Strategy<Value = Vec<Review>> {
        prop::collection::vec((0i64..400, grade()), 0..25).prop_map(|v| {
            v.into_iter()
                .enumerate()
                .map(|(i, (day, g))| rv(i as u32, day * DAY_MS / 4, g))
                .collect()
        })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(512))]

        /// The state does not depend on the order the log is held in.
        #[test]
        fn prop_replay_is_order_independent_after_sort(
            reviews in log(), seed in any::<u64>(),
        ) {
            let mut shuffled = reviews.clone();
            let n = shuffled.len().max(1);
            shuffled.rotate_left((seed as usize) % n);
            shuffled.reverse();
            prop_assert_eq!(replay(&reviews, 0.9), replay(&shuffled, 0.9));
        }

        /// A better answer never schedules the card sooner.
        #[test]
        fn prop_due_is_monotonic_in_grade(reviews in log(), extra in 0i64..30) {
            let state = replay(&reviews, 0.9);
            let now = state.last_ms + extra * DAY_MS / 3;
            let due = preview(&state, now, 0.9);
            prop_assert!(due[0] <= due[1] && due[1] <= due[2] && due[2] <= due[3]);
            prop_assert!(due.iter().all(|d| *d > 0));
        }

        /// State stays finite and in range whatever the history.
        #[test]
        fn prop_state_stays_sane(reviews in log(), retention in 0.0f64..1.5) {
            let s = replay(&reviews, retention);
            prop_assert!(s.stability.is_finite() && s.difficulty.is_finite());
            prop_assert!(s.difficulty == 0.0 || (1.0..=10.0).contains(&s.difficulty));
            prop_assert_eq!(s.reps as usize, reviews.len());
        }

        /// A higher target recall never gives a longer interval.
        #[test]
        fn prop_higher_retention_is_not_longer(s in 0.1f64..400.0, a in 0.80f64..0.97, b in 0.80f64..0.97) {
            let (lo, hi) = if a < b { (a, b) } else { (b, a) };
            prop_assert!(interval_days(s, hi) <= interval_days(s, lo) + 1e-9);
        }
    }
}
