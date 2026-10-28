//! A word-count goal for a note, shown in focus mode's footer as a ring and
//! "640 / 1,000". A note's own goal is kept in the note, in
//! `kansoWordGoal`, so it travels with it (end-to-end encrypted, like the
//! rest of the note); notes without one use the default from Settings.

use kanso_core::Note;
use serde_json::Value;

use crate::text_stats::format_count;

/// Where a note keeps its goal, beside the API's own fields.
pub const GOAL_KEY: &str = "kansoWordGoal";

/// The goals offered in the footer's menu and the palette.
pub const PRESETS: [u32; 7] = [250, 500, 750, 1000, 1500, 2000, 5000];

/// The largest goal Kanso takes.
pub const MAX_GOAL: u32 = 1_000_000;

/// A note's own goal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GoalSetting {
    /// The default from Settings (nothing kept in the note).
    Default,
    /// No goal for this note, whatever the default (kept as 0).
    Off,
    /// This note's own goal, in words.
    Words(u32),
}

/// The goal `note` keeps. A value Kanso can't use counts as none kept.
pub fn note_goal(note: &Note) -> GoalSetting {
    let Some(value) = note.extra.get(GOAL_KEY) else {
        return GoalSetting::Default;
    };
    match value.as_u64() {
        Some(0) => GoalSetting::Off,
        Some(n) if n <= u64::from(MAX_GOAL) => GoalSetting::Words(n as u32),
        _ => GoalSetting::Default,
    }
}

/// Keeps `setting` in `note`.
pub fn set_note_goal(note: &mut Note, setting: GoalSetting) {
    let value: Value = match setting {
        GoalSetting::Default => {
            note.extra.remove(GOAL_KEY);
            return;
        }
        GoalSetting::Off => 0.into(),
        GoalSetting::Words(n) => n.into(),
    };
    note.extra.insert(GOAL_KEY.into(), value);
}

/// The goal that applies: the note's own, else the default.
pub fn effective_goal(setting: GoalSetting, default: Option<u32>) -> Option<u32> {
    match setting {
        GoalSetting::Default => default,
        GoalSetting::Off => None,
        GoalSetting::Words(n) => Some(n),
    }
}

/// How far along the note is, from 0 to 1.
pub fn progress(words: usize, goal: u32) -> f32 {
    if goal == 0 {
        return 1.;
    }
    (words as f32 / goal as f32).clamp(0., 1.)
}

/// "640 / 1,000".
pub fn goal_label(words: usize, goal: u32) -> String {
    format!("{} / {}", format_count(words), format_count(goal as usize))
}

/// A goal typed by hand: "1000", "1,000", "1 000", "1.5k", "2k words".
pub fn parse_goal(input: &str) -> Option<u32> {
    let text = input.trim().to_ascii_lowercase();
    let text = text
        .strip_suffix("words")
        .or_else(|| text.strip_suffix("word"))
        .unwrap_or(&text)
        .trim();
    let (number, scale) = match text.strip_suffix('k') {
        Some(n) => (n.trim(), 1000.),
        None => (text, 1.),
    };
    let digits: String = number
        .chars()
        .filter(|c| !matches!(c, ',' | ' ' | '_'))
        .collect();
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit() || c == '.') {
        return None;
    }
    let value: f64 = digits.parse().ok()?;
    let goal = (value * scale).round();
    (goal >= 1. && goal <= f64::from(MAX_GOAL)).then_some(goal as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;
    use serde_json::json;

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 512,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    fn note(extra: Value) -> Note {
        let mut raw = json!({"_id": "note:a", "title": "t", "body": "", "bookId": "book:b"});
        raw.as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        serde_json::from_value(raw).unwrap()
    }

    #[test]
    fn goals_in_notes() {
        use GoalSetting::*;
        assert_eq!(
            note_goal(&note(json!({"kansoWordGoal": 1000}))),
            Words(1000)
        );
        assert_eq!(note_goal(&note(json!({"kansoWordGoal": 0}))), Off);
        assert_eq!(note_goal(&note(json!({"kansoWordGoal": "1000"}))), Default);
        assert_eq!(note_goal(&note(json!({"kansoWordGoal": -3}))), Default);
        assert_eq!(note_goal(&note(json!({}))), Default);
        let mut n = note(json!({}));
        set_note_goal(&mut n, Words(750));
        let raw = serde_json::to_value(&n).unwrap();
        assert_eq!(raw["kansoWordGoal"], 750);
        set_note_goal(&mut n, Default);
        assert_eq!(note_goal(&n), Default);
        assert!(
            !serde_json::to_value(&n)
                .unwrap()
                .as_object()
                .unwrap()
                .contains_key(GOAL_KEY)
        );
    }

    #[test]
    fn labels_and_parsing() {
        use GoalSetting::*;
        assert_eq!(goal_label(640, 1000), "640 / 1,000");
        assert_eq!(progress(640, 1000), 0.64);
        assert_eq!(progress(2000, 1000), 1.);
        assert_eq!(effective_goal(Default, Some(500)), Some(500));
        assert_eq!(effective_goal(Words(300), Some(500)), Some(300));
        assert_eq!(effective_goal(Off, Some(500)), None);
        assert_eq!(parse_goal("1,000"), Some(1000));
        assert_eq!(parse_goal("1.5k"), Some(1500));
        assert_eq!(parse_goal(" 2k words"), Some(2000));
        assert_eq!(parse_goal("800 words"), Some(800));
        assert_eq!(parse_goal("0"), None);
        assert_eq!(parse_goal("lots"), None);
        assert_eq!(parse_goal("-5"), None);
    }

    proptest! {
        #![proptest_config(config())]

        /// Round trip: a goal written as the label writes it reads back.
        #[test]
        fn prop_goals_parse_back(goal in 1u32..=MAX_GOAL) {
            prop_assert_eq!(parse_goal(&format_count(goal as usize)), Some(goal));
            prop_assert_eq!(parse_goal(&goal.to_string()), Some(goal));
        }

        /// Progress is a share: within 0 and 1, rising with the words, and
        /// full once the goal is met.
        #[test]
        fn prop_progress_is_a_share(words in 0usize..20_000, more in 0usize..500, goal in 1u32..10_000) {
            let p = progress(words, goal);
            prop_assert!((0. ..=1.).contains(&p));
            prop_assert!(progress(words + more, goal) >= p);
            prop_assert_eq!(progress(goal as usize + more, goal), 1.);
        }

        /// A goal set on a note survives the note's trip through JSON, as
        /// the store and sync carry it.
        #[test]
        fn prop_goals_survive_json(goal in prop_oneof![
            Just(GoalSetting::Default),
            Just(GoalSetting::Off),
            (1u32..=MAX_GOAL).prop_map(GoalSetting::Words),
        ]) {
            let mut n = note(json!({}));
            set_note_goal(&mut n, goal);
            let back: Note = serde_json::from_value(serde_json::to_value(&n).unwrap()).unwrap();
            prop_assert_eq!(note_goal(&back), goal);
        }
    }
}
