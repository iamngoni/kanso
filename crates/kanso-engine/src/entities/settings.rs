//! What a person chooses in Settings › People & Projects, on the desktop and
//! on phones.

use kanso_ai::Provider;
use serde::{Deserialize, Serialize};

use crate::pages::RefreshPolicy;

/// The People & Projects settings.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct EntitySettings {
    /// "Suggest people and projects". Detection without AI is local and free,
    /// so it is on whether or not AI is.
    pub suggest: bool,
    /// "Keep pages in": a notebook ID. `None` means the notebook named
    /// [`DEFAULT_NOTEBOOK`], made on first use.
    pub keep_in: Option<String>,
    /// "Write summaries with AI". Off keeps computed sections only.
    pub write_with_ai: bool,
    /// "Refresh AI sections": `open` or `press`. `None` until chosen: a
    /// model on this device or network or an agent refreshes when a page
    /// opens, a provider with an API key only when Refresh is pressed.
    pub refresh: Option<String>,
    /// "Use AI to find people and projects". Off by default.
    pub detect_with_ai: bool,
    /// When AI last re-ranked the suggestions (ms since the epoch).
    pub last_ai_detect: i64,
    /// "Show notebook homepages" (issue #201). Off restores the plain note
    /// list and its empty-state line.
    pub notebook_homepages: bool,
    /// "Write notebook summaries with AI". The refresh setting above is
    /// shared with people's pages.
    pub notebook_summaries: bool,
}

impl Default for EntitySettings {
    fn default() -> Self {
        EntitySettings {
            suggest: true,
            keep_in: None,
            write_with_ai: true,
            refresh: None,
            detect_with_ai: false,
            last_ai_detect: 0,
            notebook_homepages: true,
            notebook_summaries: true,
        }
    }
}

/// The notebook pages are kept in until another is chosen.
pub const DEFAULT_NOTEBOOK: &str = "People & Projects";

impl EntitySettings {
    /// The refresh policy in force for `provider` at `base_url`.
    pub fn policy(&self, provider: Provider, base_url: &str) -> RefreshPolicy {
        match self.refresh.as_deref() {
            Some("open") => RefreshPolicy::OnOpen,
            Some("press") => RefreshPolicy::OnPress,
            _ if provider.needs_key() && provider.is_remote(base_url) => RefreshPolicy::OnPress,
            _ => RefreshPolicy::OnOpen,
        }
    }

    /// Chooses when AI sections are rewritten.
    pub fn choose_refresh(&mut self, policy: RefreshPolicy) {
        self.refresh = Some(
            match policy {
                RefreshPolicy::OnOpen => "open",
                RefreshPolicy::OnPress => "press",
            }
            .to_owned(),
        );
    }

    /// Whether AI sections are written at all: AI is on and the person
    /// wants summaries.
    pub fn ai_sections(&self, ai_on: bool) -> bool {
        ai_on && self.write_with_ai
    }

    /// Whether a notebook homepage's AI sections are written.
    pub fn notebook_ai(&self, ai_on: bool) -> bool {
        ai_on && self.notebook_summaries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // no-prop-test: two providers against one default.
    #[test]
    fn local_models_refresh_on_open_and_keyed_providers_on_press() {
        let settings = EntitySettings::default();
        assert_eq!(
            settings.policy(Provider::LocalModel, "http://localhost:11434/v1"),
            RefreshPolicy::OnOpen
        );
        assert_eq!(
            settings.policy(Provider::OpenAi, "https://api.openai.com/v1"),
            RefreshPolicy::OnPress
        );
        let mut chosen = settings.clone();
        chosen.choose_refresh(RefreshPolicy::OnOpen);
        assert_eq!(
            chosen.policy(Provider::OpenAi, "https://api.openai.com/v1"),
            RefreshPolicy::OnOpen
        );
    }

    // no-prop-test: a literal old file.
    #[test]
    fn an_old_file_reads_with_defaults() {
        let read: EntitySettings = serde_json::from_str("{}").unwrap();
        assert_eq!(read, EntitySettings::default());
    }
}
