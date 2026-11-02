//! Settings > AI > Agent tasks, as the config file keeps them (desktop) and
//! as the phone's preferences hand them over. A model file: values and the
//! rules that make them valid, nothing else.

use std::time::Duration;

use kanso_server::TaskOps;
use serde::{Deserialize, Serialize};

use super::{DEFAULT_MAX_NOTES, DEFAULT_TIME_LIMIT, MAX_NOTES_CHOICES, TIME_LIMITS_MIN};

/// What the person set for agent tasks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AgentTaskSettings {
    /// "Agents may propose": the defaults the New agent task sheet starts
    /// from.
    pub new_notes: bool,
    pub edits: bool,
    pub moves: bool,
    pub merges: bool,
    /// The time limit in minutes: 5, 20 (default) or 60.
    pub time_limit_minutes: u64,
    /// The most notes one task may read: 50, 200 (default) or 1,000.
    pub max_notes: usize,
    /// Ask before starting a task that can read all notebooks.
    pub ask_before_reading_all: bool,
}

impl Default for AgentTaskSettings {
    fn default() -> Self {
        AgentTaskSettings {
            new_notes: true,
            edits: true,
            moves: true,
            merges: true,
            time_limit_minutes: DEFAULT_TIME_LIMIT.as_secs() / 60,
            max_notes: DEFAULT_MAX_NOTES,
            ask_before_reading_all: true,
        }
    }
}

impl AgentTaskSettings {
    /// The settings with every value one of the choices Settings offers: a
    /// value from an old or edited file snaps to the nearest choice.
    pub fn normalised(mut self) -> AgentTaskSettings {
        self.time_limit_minutes = *TIME_LIMITS_MIN
            .iter()
            .min_by_key(|choice| choice.abs_diff(self.time_limit_minutes))
            .unwrap_or(&20);
        self.max_notes = *MAX_NOTES_CHOICES
            .iter()
            .min_by_key(|choice| choice.abs_diff(self.max_notes))
            .unwrap_or(&200);
        self
    }

    /// The time one task may run.
    pub fn time_limit(&self) -> Duration {
        Duration::from_secs(self.normalised().time_limit_minutes * 60)
    }

    /// What the sheet's checkboxes start as.
    pub fn ops(&self) -> TaskOps {
        TaskOps {
            create: self.new_notes,
            edit: self.edits,
            moves: self.moves,
            merge: self.merges,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn the_defaults_are_the_issues() {
        let s = AgentTaskSettings::default();
        assert_eq!(s.time_limit(), Duration::from_secs(20 * 60));
        assert_eq!(s.max_notes, 200);
        assert!(s.ask_before_reading_all);
        assert_eq!(s.ops(), TaskOps::default());
    }

    #[test]
    fn a_partial_file_keeps_the_defaults() {
        let s: AgentTaskSettings = serde_json::from_str(r#"{"edits": false}"#).unwrap();
        assert!(!s.edits && s.new_notes && s.merges);
        assert_eq!(s.time_limit_minutes, 20);
    }

    proptest! {
        /// Whatever a file holds, the limits end up as a choice Settings offers.
        #[test]
        fn prop_limits_snap_to_a_choice(minutes in 0u64..10_000, notes in 0usize..100_000) {
            let s = AgentTaskSettings { time_limit_minutes: minutes, max_notes: notes, ..Default::default() }.normalised();
            prop_assert!(TIME_LIMITS_MIN.contains(&s.time_limit_minutes));
            prop_assert!(MAX_NOTES_CHOICES.contains(&s.max_notes));
            prop_assert_eq!(s.normalised(), s);
        }
    }
}
