//! The desktop engine's word-goal and text-count helpers for mobile UI.

use flutter_rust_bridge::frb;
use kanso_mobile::{Library, format_count, goal_label, parse_goal, progress, word_count};

/// The word-goal presets offered by the desktop app.
#[frb(sync)]
pub fn word_goal_presets() -> Vec<u32> {
    Library::word_goal_presets()
}

/// Counts note words with the same Markdown-aware rules as the desktop.
#[frb(sync)]
pub fn word_goal_word_count(text: String) -> u32 {
    word_count(&text).min(u32::MAX as usize) as u32
}

/// Parses a typed goal such as `1,500`, `1.5k` or `2k words`.
#[frb(sync)]
pub fn word_goal_parse(input: String) -> Option<u32> {
    parse_goal(&input)
}

/// Formats a count with the desktop's thousands separators.
#[frb(sync)]
pub fn word_goal_format_count(count: u32) -> String {
    format_count(count as usize)
}

/// Formats the editor footer's current and target counts.
#[frb(sync)]
pub fn word_goal_label(words: u32, goal: u32) -> String {
    goal_label(words as usize, goal)
}

/// The goal progress, clamped to the range from zero to one.
#[frb(sync)]
pub fn word_goal_progress(words: u32, goal: u32) -> f32 {
    progress(words as usize, goal)
}
