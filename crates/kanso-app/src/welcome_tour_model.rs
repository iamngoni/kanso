//! Pure model for the six-page desktop Welcome Tour (issue #510): page
//! navigation, bounds, seen-flag logic, appearance mode mapping, responsive
//! dialog sizing, and agent detection for MCP setup.

use std::path::{Path, PathBuf};

use crate::config::{KANSO_DARK, KANSO_LIGHT, ThemePreference, ThemeSettings};
use crate::mcp_setup::Agent;

/// The six pages of the first-run setup, in order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TourPage {
    #[default]
    Welcome,
    LookAndFeel,
    Writing,
    AccountAndSync,
    AiAndVoice,
    AgentsAndMcp,
}

impl TourPage {
    pub const ALL: [TourPage; 6] = [
        TourPage::Welcome,
        TourPage::LookAndFeel,
        TourPage::Writing,
        TourPage::AccountAndSync,
        TourPage::AiAndVoice,
        TourPage::AgentsAndMcp,
    ];

    pub const COUNT: usize = Self::ALL.len();

    /// Zero-based position in [`Self::ALL`].
    pub fn index(self) -> usize {
        match self {
            TourPage::Welcome => 0,
            TourPage::LookAndFeel => 1,
            TourPage::Writing => 2,
            TourPage::AccountAndSync => 3,
            TourPage::AiAndVoice => 4,
            TourPage::AgentsAndMcp => 5,
        }
    }

    /// The page at `index`, clamped into `0..COUNT`.
    #[cfg(test)]
    pub fn from_index(index: usize) -> TourPage {
        Self::ALL[index.min(Self::COUNT - 1)]
    }

    /// The next page, or `None` on the last page.
    pub fn next(self) -> Option<TourPage> {
        let next = self.index() + 1;
        (next < Self::COUNT).then(|| Self::ALL[next])
    }

    /// The previous page, or `None` on the first page.
    pub fn prev(self) -> Option<TourPage> {
        self.index().checked_sub(1).map(|ix| Self::ALL[ix])
    }

    pub fn is_first(self) -> bool {
        self == TourPage::Welcome
    }

    #[cfg(test)]
    pub fn is_last(self) -> bool {
        self == TourPage::AgentsAndMcp
    }

    pub fn title(self) -> &'static str {
        match self {
            TourPage::Welcome => "Welcome to Kanso",
            TourPage::LookAndFeel => "Look and feel",
            TourPage::Writing => "Writing",
            TourPage::AccountAndSync => "Account and sync",
            TourPage::AiAndVoice => "AI and voice",
            TourPage::AgentsAndMcp => "Agents and MCP",
        }
    }

    pub fn subtitle(self) -> &'static str {
        match self {
            TourPage::Welcome => {
                "Set up Kanso the way you work. Every choice can be changed later in Settings."
            }
            TourPage::LookAndFeel => {
                "Pick a mode, theme, and icon style. Changes preview behind this sheet."
            }
            TourPage::Writing => "Choose your font, size, line measure, and default editor layout.",
            TourPage::AccountAndSync => {
                "Sync notes end-to-end encrypted across devices, or stay local-only."
            }
            TourPage::AiAndVoice => {
                "Optionally download the on-device voice model, connect AI, or enable semantic search."
            }
            TourPage::AgentsAndMcp => {
                "Let local AI agents search, read, and suggest edits to your notes over MCP."
            }
        }
    }

    pub fn primary_label(self) -> &'static str {
        match self {
            TourPage::Welcome => "Set up in a minute",
            TourPage::AgentsAndMcp => "Done",
            _ => "Next",
        }
    }

    pub fn skip_label(self) -> &'static str {
        match self {
            TourPage::Welcome => "Skip",
            _ => "Skip setup",
        }
    }

    pub fn selector(self) -> &'static str {
        match self {
            TourPage::Welcome => "welcome-tour-page-welcome",
            TourPage::LookAndFeel => "welcome-tour-page-look-and-feel",
            TourPage::Writing => "welcome-tour-page-writing",
            TourPage::AccountAndSync => "welcome-tour-page-account-and-sync",
            TourPage::AiAndVoice => "welcome-tour-page-ai-and-voice",
            TourPage::AgentsAndMcp => "welcome-tour-page-agents-and-mcp",
        }
    }
}

/// Result of advancing from the current page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepOutcome {
    Moved(TourPage),
    Finished,
}

/// Pure navigation state for the Welcome Tour.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TourState {
    pub page: TourPage,
}

impl TourState {
    #[cfg(test)]
    pub fn new() -> Self {
        TourState {
            page: TourPage::Welcome,
        }
    }

    pub fn advance(&mut self) -> StepOutcome {
        match self.page.next() {
            Some(next) => {
                self.page = next;
                StepOutcome::Moved(next)
            }
            None => StepOutcome::Finished,
        }
    }

    pub fn back(&mut self) -> bool {
        match self.page.prev() {
            Some(prev) => {
                self.page = prev;
                true
            }
            None => false,
        }
    }

    pub fn jump(&mut self, page: TourPage) {
        self.page = page;
    }
}

/// Whether the tour should open automatically at launch.
pub fn should_show_on_launch(welcome_tour_seen: bool) -> bool {
    !welcome_tour_seen
}

/// The three headline lines on Page 1 (local-first notes, sync, agents).
pub const WELCOME_LINES: [(&str, &str); 3] = [
    (
        "Local-first Markdown notes",
        "Your notes live in a fast, encrypted local library and stay portable as plain Markdown.",
    ),
    (
        "End-to-end encrypted sync",
        "Sync across Mac, Windows, Linux, iOS and Android, or keep everything on this device alone.",
    ),
    (
        "Built for local AI and agents",
        "Connect your own AI or local MCP agents while locked notes and #ai-exclude stay private.",
    ),
];

/// Sample prose shown on Page 3 (Writing) so font, size and line measure
/// changes are immediately visible.
pub const SAMPLE_PARAGRAPH: &str = "Good notes stay quiet until you need them. Write in plain Markdown with wikilinks like [[Project Plan]], code blocks, and tasks — fast on your machine and readable years from now.";

/// Short summary shown on Page 6 (Agents and MCP).
pub const AGENTS_LINE: &str = "Agents connected over MCP can search, read, and suggest edits to your notes — never touching locked notes, the trash, or notes tagged #ai-exclude.";

/// Current appearance mode (`System`, `Light`, or `Dark`) derived from
/// `ThemeSettings` and whether the active UI theme is dark.
pub fn theme_mode_of(settings: &ThemeSettings, ui_is_dark: bool) -> ThemePreference {
    if settings.follow_system {
        ThemePreference::System
    } else if ui_is_dark {
        ThemePreference::Dark
    } else {
        ThemePreference::Light
    }
}

/// Applies a Light / Dark / System choice to `ThemeSettings`, keeping the
/// chosen light and dark theme names intact.
pub fn apply_theme_mode(settings: &ThemeSettings, mode: ThemePreference) -> ThemeSettings {
    let mut next = settings.clone();
    match mode {
        ThemePreference::System => {
            next.follow_system = true;
        }
        ThemePreference::Light => {
            next.follow_system = false;
            next.ui = if next.light.is_empty() {
                KANSO_LIGHT.to_owned()
            } else {
                next.light.clone()
            };
        }
        ThemePreference::Dark => {
            next.follow_system = false;
            next.ui = if next.dark.is_empty() {
                KANSO_DARK.to_owned()
            } else {
                next.dark.clone()
            };
        }
    }
    next
}

/// Desired dialog width in points, fitting cleanly even at `WINDOW_MIN_SIZE`
/// (520 × 420).
pub const SHEET_WIDTH: f32 = 620.;

pub fn dialog_width(viewport_width: f32) -> f32 {
    if !viewport_width.is_finite() {
        return SHEET_WIDTH;
    }
    (viewport_width - 32.).clamp(320., SHEET_WIDTH)
}

/// Maximum height of the scrollable body inside the sheet so header and footer
/// stay visible even at the smallest window height (420 px).
pub fn body_max_height(viewport_height: f32) -> f32 {
    if !viewport_height.is_finite() {
        return 500.;
    }
    (viewport_height - 190.).clamp(160., 540.)
}

/// Detection result for a known MCP agent on this machine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DetectedAgent {
    pub agent: Agent,
    pub detected: bool,
}

/// Checks which MCP-compatible agents Settings › Local API knows about appear
/// to be installed on this machine, without spawning any external process.
/// Detected agents are sorted first while keeping `Agent::ALL` order within
/// each group.
pub fn detect_mcp_agents(home: Option<&Path>, path_dirs: &[PathBuf]) -> Vec<DetectedAgent> {
    let mut list: Vec<DetectedAgent> = Agent::ALL
        .into_iter()
        .map(|agent| DetectedAgent {
            agent,
            detected: is_agent_installed(agent, home, path_dirs),
        })
        .collect();
    // Stable sort puts detected agents first while preserving `Agent::ALL` order.
    list.sort_by_key(|entry| !entry.detected);
    list
}

fn has_exe(name: &str, dirs: &[PathBuf]) -> bool {
    kanso_acp::detect::which_in(name, dirs).is_some()
}

fn home_has(home: Option<&Path>, rel: &str) -> bool {
    home.is_some_and(|h| h.join(rel).exists())
}

fn app_exists(path: &str) -> bool {
    cfg!(target_os = "macos") && Path::new(path).exists()
}

fn is_agent_installed(agent: Agent, home: Option<&Path>, dirs: &[PathBuf]) -> bool {
    match agent {
        Agent::ClaudeCode => {
            has_exe("claude", dirs) || home_has(home, ".claude.json") || home_has(home, ".claude")
        }
        Agent::ClaudeDesktop => {
            home_has(home, "Library/Application Support/Claude")
                || home_has(home, ".config/Claude")
                || app_exists("/Applications/Claude.app")
        }
        Agent::Codex => has_exe("codex", dirs) || home_has(home, ".codex"),
        Agent::Cursor => {
            has_exe("cursor", dirs)
                || home_has(home, ".cursor")
                || app_exists("/Applications/Cursor.app")
        }
        Agent::VsCode => {
            has_exe("code", dirs)
                || home_has(home, ".vscode")
                || app_exists("/Applications/Visual Studio Code.app")
        }
        Agent::GeminiCli => has_exe("gemini", dirs) || home_has(home, ".gemini"),
        Agent::Windsurf => {
            has_exe("windsurf", dirs)
                || home_has(home, ".codeium/windsurf")
                || app_exists("/Applications/Windsurf.app")
        }
        Agent::Zed => {
            has_exe("zed", dirs)
                || home_has(home, ".config/zed")
                || app_exists("/Applications/Zed.app")
        }
        Agent::OpenCode => has_exe("opencode", dirs) || home_has(home, ".config/opencode"),
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    fn proptest_cfg() -> ProptestConfig {
        ProptestConfig {
            cases: 128,
            rng_seed: proptest::test_runner::RngSeed::Fixed(20_261_017),
            ..ProptestConfig::default()
        }
    }

    #[test]
    fn tour_pages_walk_forward_and_back_cleanly() {
        let mut state = TourState::new();
        assert_eq!(state.page, TourPage::Welcome);
        assert!(state.page.is_first());
        assert!(!state.back());
        assert_eq!(state.page.primary_label(), "Set up in a minute");

        for expected in &TourPage::ALL[1..] {
            assert_eq!(state.advance(), StepOutcome::Moved(*expected));
        }
        assert!(state.page.is_last());
        assert_eq!(state.page.primary_label(), "Done");
        assert_eq!(state.advance(), StepOutcome::Finished);

        for expected in TourPage::ALL[..TourPage::COUNT - 1].iter().rev() {
            assert!(state.back());
            assert_eq!(state.page, *expected);
        }
        assert!(!state.back());
    }

    #[test]
    fn launch_gate_shows_only_when_unseen() {
        assert!(should_show_on_launch(false));
        assert!(!should_show_on_launch(true));
    }

    #[test]
    fn theme_mode_switches_and_preserves_pair() {
        let base = ThemeSettings {
            ui: "Nord".into(),
            syntax: None,
            preview: None,
            follow_system: false,
            light: "Solarized Light".into(),
            dark: "Nord".into(),
        };
        let sys = apply_theme_mode(&base, ThemePreference::System);
        assert!(sys.follow_system);
        assert_eq!(theme_mode_of(&sys, true), ThemePreference::System);
        assert_eq!(theme_mode_of(&sys, false), ThemePreference::System);

        let light = apply_theme_mode(&sys, ThemePreference::Light);
        assert!(!light.follow_system);
        assert_eq!(light.ui, "Solarized Light");
        assert_eq!(theme_mode_of(&light, false), ThemePreference::Light);

        let dark = apply_theme_mode(&light, ThemePreference::Dark);
        assert!(!dark.follow_system);
        assert_eq!(dark.ui, "Nord");
        assert_eq!(theme_mode_of(&dark, true), ThemePreference::Dark);
    }

    #[test]
    fn detect_mcp_agents_finds_config_dirs_and_sorts_detected_first() {
        let home = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(home.path().join(".codex")).unwrap();
        std::fs::write(home.path().join(".claude.json"), "{}").unwrap();

        let detected = detect_mcp_agents(Some(home.path()), &[]);
        assert_eq!(detected.len(), Agent::ALL.len());
        let found: Vec<Agent> = detected
            .iter()
            .filter(|a| a.detected)
            .map(|a| a.agent)
            .collect();
        assert!(found.contains(&Agent::ClaudeCode));
        assert!(found.contains(&Agent::Codex));
        // All detected agents come before all undetected agents.
        let first_undetected = detected.iter().position(|a| !a.detected);
        let last_detected = detected.iter().rposition(|a| a.detected);
        if let (Some(first_u), Some(last_d)) = (first_undetected, last_detected) {
            assert!(last_d < first_u);
        }
    }

    #[test]
    fn sizing_fits_smallest_window() {
        let (min_w, min_h) = crate::layout::WINDOW_MIN_SIZE;
        assert!(dialog_width(min_w) <= min_w - 16.);
        assert!(body_max_height(min_h) <= min_h - 160.);
    }

    proptest! {
        #![proptest_config(proptest_cfg())]

        #[test]
        fn prop_index_roundtrip_and_clamping(ix in 0usize..100) {
            let page = TourPage::from_index(ix);
            prop_assert!(page.index() < TourPage::COUNT);
            if ix < TourPage::COUNT {
                prop_assert_eq!(page.index(), ix);
            } else {
                prop_assert_eq!(page, TourPage::ALL[TourPage::COUNT - 1]);
            }
        }

        #[test]
        fn prop_navigation_bounds(ops in prop::collection::vec(0u8..3, 0..64)) {
            let mut state = TourState::new();
            for op in ops {
                match op {
                    0 => {
                        let before = state.page.index();
                        match state.advance() {
                            StepOutcome::Moved(p) => prop_assert_eq!(p.index(), before + 1),
                            StepOutcome::Finished => prop_assert_eq!(before, TourPage::COUNT - 1),
                        }
                    }
                    1 => {
                        let before = state.page.index();
                        let moved = state.back();
                        if before == 0 {
                            prop_assert!(!moved);
                            prop_assert_eq!(state.page.index(), 0);
                        } else {
                            prop_assert!(moved);
                            prop_assert_eq!(state.page.index(), before - 1);
                        }
                    }
                    _ => {
                        let target = TourPage::from_index(state.page.index());
                        state.jump(target);
                        prop_assert_eq!(state.page, target);
                    }
                }
                prop_assert!(state.page.index() < TourPage::COUNT);
            }
        }

        #[test]
        fn prop_dialog_sizing_is_bounded(w in -500.0f32..5000.0, h in -500.0f32..5000.0) {
            let dw = dialog_width(w);
            let bh = body_max_height(h);
            prop_assert!((320.0..=SHEET_WIDTH).contains(&dw));
            prop_assert!((160.0..=540.0).contains(&bh));
        }

        #[test]
        fn prop_apply_theme_mode_respects_preference(
            follow in any::<bool>(),
            mode_ix in 0usize..3,
            light_name in "[A-Za-z ]{0,16}",
            dark_name in "[A-Za-z ]{0,16}",
        ) {
            let mode = [ThemePreference::System, ThemePreference::Light, ThemePreference::Dark][mode_ix];
            let settings = ThemeSettings {
                ui: KANSO_DARK.into(),
                syntax: None,
                preview: None,
                follow_system: follow,
                light: light_name,
                dark: dark_name,
            };
            let next = apply_theme_mode(&settings, mode);
            prop_assert_eq!(next.follow_system, mode == ThemePreference::System);
            prop_assert_eq!(next.light, settings.light);
            prop_assert_eq!(next.dark, settings.dark);
            if mode != ThemePreference::System {
                prop_assert!(!next.ui.is_empty());
            }
        }
    }
}
