//! What a person chooses about Ask in Settings › AI, on the desktop and on
//! phones: how much one question may send, whether answers are checked
//! against the notes, and whether links of the best notes are followed.

use kanso_ai::{Budget, BudgetSize, Provider};
use serde::{Deserialize, Serialize};

/// The Ask settings. Defaults: the budget the model suits, checking on,
/// links followed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AskSettings {
    /// "Most each question may send". `None` until chosen: a model on this
    /// device or network starts at Small, any other at Standard.
    pub budget: Option<String>,
    /// Check that each cited line was sent and supports its sentence. Off,
    /// answers show as the model wrote them, with a warning in Settings.
    pub check_sources: bool,
    /// Look up related notes through links and backlinks of the best notes.
    pub use_links: bool,
}

impl Default for AskSettings {
    fn default() -> Self {
        AskSettings {
            budget: None,
            check_sources: true,
            use_links: true,
        }
    }
}

impl AskSettings {
    /// The size in force for `provider` at `base_url`.
    pub fn size(&self, provider: Provider, base_url: &str) -> BudgetSize {
        self.budget
            .as_deref()
            .and_then(BudgetSize::from_key)
            .unwrap_or(if provider.is_remote(base_url) {
                BudgetSize::Standard
            } else {
                BudgetSize::Small
            })
    }

    /// The ceilings in force for `provider` at `base_url`.
    pub fn budget_for(&self, provider: Provider, base_url: &str) -> Budget {
        Budget::of(self.size(provider, base_url))
    }

    /// Chooses `size` explicitly.
    pub fn choose(&mut self, size: BudgetSize) {
        self.budget = Some(size.key().to_owned());
    }

    /// The warning shown beside Check sources.
    pub const CHECK_WARNING: &'static str =
        "Without checking, an answer may cite lines that were never sent.";
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn a_local_model_starts_small_and_a_remote_one_standard() {
        let settings = AskSettings::default();
        assert_eq!(
            settings.size(Provider::LocalModel, "http://localhost:11434/v1"),
            BudgetSize::Small
        );
        assert_eq!(
            settings.size(Provider::OpenAi, "https://api.openai.com/v1"),
            BudgetSize::Standard
        );
    }

    #[test]
    fn a_chosen_size_wins_everywhere() {
        let mut settings = AskSettings::default();
        settings.choose(BudgetSize::Large);
        assert_eq!(
            settings.size(Provider::LocalModel, "http://localhost:11434/v1"),
            BudgetSize::Large
        );
        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("\"checkSources\":true"), "{json}");
        let back: AskSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(back, settings);
        // An old file without the section reads as the defaults.
        let old: AskSettings = serde_json::from_str("{}").unwrap();
        assert_eq!(old, AskSettings::default());
    }

    proptest! {
        /// An unknown saved size never breaks Ask: it falls back to the
        /// size the model suits.
        #[test]
        fn prop_an_unknown_saved_size_falls_back(key in "[a-z]{0,8}") {
            prop_assume!(BudgetSize::from_key(&key).is_none());
            let settings = AskSettings { budget: Some(key), ..AskSettings::default() };
            let size = settings.size(Provider::OpenAi, "https://api.openai.com/v1");
            prop_assert_eq!(size, BudgetSize::Standard);
        }
    }
}
