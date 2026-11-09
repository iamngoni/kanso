//! "Send feedback" from the window for a library that won't open.
//!
//! The app's feedback sender queues into the library's outbox, which is
//! the very thing that can't open here. This builds the same report
//! (`kanso_engine::feedback::build_payload`, the Worker's schema) from what
//! is at hand, with the error's text scrubbed, and sends it straight to
//! Kanso's Worker (`kanso_sync::feedback::Client`) as an anonymous
//! reporter. A failure is shown with Retry; nothing is queued.

use std::path::Path;

use kanso_engine::feedback::{
    self, AppInfo, Draft, DraftError, Kind, MAX_TEXT, Screen, SyncMode, new_id,
};
use kanso_sync::SecretStore;
use kanso_sync::feedback::{Client, FeedbackError, Reporter, Transport};

pub const SENT: &str = "Sent — thank you";
pub const PLACEHOLDER: &str = "Anything you'd like to add? (optional)";

/// Replaces the person's home folder in a path with `~`: `C:\Users\ana\…`,
/// `/Users/ana/…` and `/home/ana/…` all lose the name.
pub fn scrub_paths(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while !rest.is_empty() {
        let hit = ["\\Users\\", "\\users\\", "/Users/", "/home/"]
            .iter()
            .filter_map(|marker| rest.find(marker).map(|at| (at, *marker)))
            .min_by_key(|(at, _)| *at);
        let Some((at, marker)) = hit else {
            out.push_str(rest);
            break;
        };
        // A drive letter ("C:") before `\Users\` goes with the name.
        let mut start = at;
        if marker.starts_with('\\') {
            let before = &rest.as_bytes()[..at];
            if before.len() >= 2 && before[before.len() - 1] == b':' {
                start = at - 2;
            }
        }
        out.push_str(&rest[..start]);
        out.push('~');
        let after = &rest[at + marker.len()..];
        let sep = if marker.starts_with('\\') { '\\' } else { '/' };
        // The user's name runs to the next separator or the end of the path.
        let end = after
            .find(|c: char| c == sep || c.is_whitespace() || matches!(c, '"' | '\'' | ')' | ','))
            .unwrap_or(after.len());
        rest = &after[end..];
    }
    // Any other path that carries the name is not the one the person typed.
    out
}

/// The report's text: what the person added, then the error's details,
/// scrubbed of paths and anything that looks like a key.
pub fn report_text(details: &str, own_words: &str) -> String {
    let own = own_words.trim();
    let mut text = String::from("Kanso couldn't open the library.\n");
    if !own.is_empty() {
        text.push_str("\nFrom the person:\n");
        text.push_str(own);
        text.push('\n');
    }
    text.push_str("\nError window details:\n");
    let scrubbed = crate::startup_error_model::redact(&scrub_paths(details));
    // Control characters other than line feeds are refused by the Worker.
    let scrubbed: String = scrubbed
        .chars()
        .filter(|c| *c == '\n' || !c.is_control())
        .collect();
    text.push_str(&scrubbed);
    text.chars().take(MAX_TEXT).collect()
}

/// The payload exactly as the Worker takes it. `app` is the same app info
/// the sheet sends (version, OS, device, locale); it needs no library.
pub fn build(
    id: &str,
    details: &str,
    own_words: &str,
    app: &AppInfo,
) -> Result<String, DraftError> {
    let draft = Draft {
        kind: Kind::Bug,
        text: report_text(details, own_words),
        credit_name: None,
        screenshot: None,
    };
    feedback::build_payload(id, &draft, Some(app))
}

/// The app info for this computer: the app's own, with no sync and no
/// account (neither is known here).
pub fn app_info(data_dir: &Path) -> AppInfo {
    crate::feedback_app::app_info(data_dir, SyncMode::None, Screen::Other)
}

/// Why sending didn't work, in words.
pub fn failure_text(err: &FeedbackError) -> String {
    match err {
        FeedbackError::Offline => "Couldn't reach Kanso. Check your connection and retry.".into(),
        FeedbackError::RateLimited { .. } => crate::feedback_app::RATE_LIMITED.into(),
        FeedbackError::Closed => crate::feedback_app::PAUSED.into(),
        FeedbackError::Invalid { field } => crate::feedback_app::rejected_text(field),
        _ => "Kanso couldn't take the report just now. Retry in a moment.".into(),
    }
}

/// Sends one report. A fresh id per attempt: a retry after a failure is a
/// new request, and the server keeps the first if both got through.
pub fn send<T: Transport>(
    transport: T,
    secrets: &dyn SecretStore,
    details: &str,
    own_words: &str,
    app: &AppInfo,
) -> Result<(), String> {
    let mut entropy = [0u8; 16];
    getrandom::fill(&mut entropy).map_err(|err| err.to_string())?;
    let payload = build(&new_id(entropy), details, own_words, app)
        .map_err(|err| crate::feedback_app::draft_error_text(&err))?;
    let reporter = Reporter::anonymous(secrets)
        .map_err(|_| "Kanso couldn't reach the keychain to send this.".to_owned())?;
    Client::new(transport)
        .submit(&reporter, &payload)
        .map(drop)
        .map_err(|err| failure_text(&err))
}

#[cfg(test)]
#[path = "startup_feedback_tests.rs"]
mod tests;
