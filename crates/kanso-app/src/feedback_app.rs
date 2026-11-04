//! In-app feedback on the desktop (issue #232): what the sheet, My feedback
//! and the update notice share. The words, the app info a report may carry,
//! who is sending, and the retry timer.
//!
//! Nothing here reads a note. The payload is built by
//! `kanso_engine::feedback::build_payload` from the person's text and
//! [`app_info`], and sent through `kanso_engine::feedback_flow`.

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use gpui_kit::*;
use kanso_engine::feedback::{
    self, AppInfo, Chip, DraftError, Kind, Platform, Screen, SyncMode, clean_build, clean_device,
    clean_locale, clean_os, scrub_log_line,
};
use kanso_engine::feedback_flow::{self, Wait};
use kanso_store::Store;
use kanso_sync::feedback::{Client, HttpTransport, Reporter, Transport};
use kanso_sync::{Machine, SecretStore, Session};

use crate::services::Services;
use crate::workspace::AppState;

// MARK: Words

pub const SENT: &str = "Thanks — feedback sent";
pub const SAVED: &str = "Saved — it will send when you're back online";
pub const PAUSED: &str = "Feedback is paused for now. Your report is saved and will send later.";
pub const RATE_LIMITED: &str = "You've sent a lot of feedback today. Try again tomorrow.";
pub const SCREENSHOT_TOO_BIG: &str = "That image is over 5 MB. Choose a smaller one.";
pub const NOT_AN_IMAGE: &str = "Choose a PNG or JPEG image.";
pub const SCREENSHOT_WARNING: &str = "Screenshots can show your notes. Check it before sending.";
pub const FOOTER: &str = "Your words and screenshot stay private with Kanso. A short summary may become a public GitHub issue. No notes are ever sent.";
pub const DELETE_PROMPT: &str = "Delete this report? Kanso removes your text, screenshot and app info from its server. A GitHub issue made from it stays, but holds no words of yours.";

/// The text area's placeholder for a kind of report.
pub fn placeholder(kind: Kind) -> &'static str {
    match kind {
        Kind::Idea => "What would you like Kanso to do?",
        Kind::Bug | Kind::Question => "What happened, and what did you expect?",
    }
}

/// The kind's label in the sheet's segmented control.
pub fn kind_label(kind: Kind) -> &'static str {
    match kind {
        Kind::Bug => "Something's wrong",
        Kind::Idea => "Idea",
        Kind::Question => "Question",
    }
}

/// What the sheet says when the draft can't be sent yet.
pub fn draft_error_text(err: &DraftError) -> String {
    match err {
        DraftError::TooShort => "Say a little more (at least 10 characters).".into(),
        DraftError::TooLong => "Keep it under 4,000 characters.".into(),
        DraftError::BadCharacter => "Take out the control characters and try again.".into(),
        DraftError::BadCredit => {
            "Use 1 to 39 letters, numbers, spaces, dots, dashes or underscores for your name."
                .into()
        }
        DraftError::BadScreenshot => SCREENSHOT_TOO_BIG.into(),
        DraftError::BadId | DraftError::BadApp(_) => {
            "Kanso couldn't build this report. Turn off app info and try again.".into()
        }
    }
}

/// The server refused a field: "Kanso couldn't send this report (<field>)…".
/// The field is a JSON path from the server; anything else in it is dropped,
/// and the value that failed is never shown.
pub fn rejected_text(field: &str) -> String {
    let safe: String = field
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '[' | ']'))
        .take(40)
        .collect();
    if safe.is_empty() {
        "Kanso couldn't send this report. Edit it and try again.".into()
    } else {
        format!("Kanso couldn't send this report ({safe}). Edit it and try again.")
    }
}

/// The notice for a report that is saved, not sent.
pub fn wait_text(wait: &Wait) -> &'static str {
    match wait {
        Wait::Paused => PAUSED,
        Wait::RateLimited { .. } => RATE_LIMITED,
        Wait::Offline | Wait::Later => SAVED,
    }
}

/// A report's chip in words.
pub fn chip_text(chip: &Chip) -> String {
    match chip {
        Chip::WaitingToSend => "Waiting to send".into(),
        Chip::Received => "Received".into(),
        Chip::FiledAs(n) => format!("Filed as #{n}"),
        Chip::Building => "Building".into(),
        Chip::FixProposed => "Fix proposed".into(),
        Chip::Merged => "Merged".into(),
        Chip::ReleasedIn(release) => format!("Released in {}", feedback::release_name(release)),
        Chip::NotFiled => "Not filed".into(),
        Chip::SameAs(n) => format!("Same as #{n}"),
        Chip::Closed => "Closed".into(),
        Chip::Deleting => "Deleting…".into(),
    }
}

/// A longer line under a chip, when there is more to say.
pub fn chip_detail(chip: &Chip) -> Option<&'static str> {
    match chip {
        Chip::NotFiled => Some(
            "Not filed — it didn't look like a bug or an idea. Still kept privately for the owner to read.",
        ),
        Chip::SameAs(_) => Some("Your report was added to it."),
        _ => None,
    }
}

/// The update notice's message: "See what changed." and, when the release
/// fixed some of this person's reports, how many.
pub fn update_message(fixed: usize) -> String {
    match fixed {
        0 => "See what changed.".into(),
        1 => "See what changed. Fixes your report.".into(),
        n => format!("See what changed. Fixes {n} of your reports."),
    }
}

// MARK: App info

/// How the library syncs, for the report: not at all, with Kanso's server,
/// or with another.
pub fn sync_mode(session: Option<&Session>) -> SyncMode {
    match session {
        None => SyncMode::None,
        Some(session) if session.origin == "https://sync.kansoapp.app" => SyncMode::Kanso,
        Some(_) => SyncMode::SelfHosted,
    }
}

/// The version without a pre-release tail: `0.9.4-dev.abc` is `0.9.4`.
pub fn core_version(version: &semver::Version) -> String {
    format!("{}.{}.{}", version.major, version.minor, version.patch)
}

fn command(program: &str, args: &[&str]) -> String {
    std::process::Command::new(program)
        .args(args)
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_owned())
        .unwrap_or_default()
}

#[cfg(target_os = "macos")]
fn system() -> (String, String, String) {
    (
        command("sw_vers", &["-productVersion"]),
        command("sysctl", &["-n", "hw.model"]),
        command("defaults", &["read", "-g", "AppleLocale"]),
    )
}

#[cfg(target_os = "windows")]
fn system() -> (String, String, String) {
    let version = command("cmd", &["/C", "ver"]);
    let digits: String = version
        .rsplit(' ')
        .next()
        .unwrap_or("")
        .trim_end_matches(']')
        .to_owned();
    (
        digits,
        "pc".into(),
        std::env::var("LANG").unwrap_or_default(),
    )
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn system() -> (String, String, String) {
    let os = std::fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|text| {
            text.lines()
                .find_map(|l| l.strip_prefix("VERSION_ID="))
                .map(|v| v.trim_matches('"').to_owned())
        })
        .unwrap_or_default();
    let device = std::fs::read_to_string("/sys/devices/virtual/dmi/id/product_name")
        .unwrap_or_default()
        .trim()
        .to_owned();
    let locale = std::env::var("LC_ALL")
        .or_else(|_| std::env::var("LANG"))
        .unwrap_or_default();
    (os, device, locale)
}

/// The last 20 lines of the updater's and the theme gallery's logs that are
/// exactly the log grammar (`scrub_log_line`); the rest are dropped.
pub fn recent_errors(data_dir: &Path) -> Vec<String> {
    let mut lines = Vec::new();
    for (file, source) in [("updates.log", "update"), ("themes.log", "theme")] {
        let Ok(text) = std::fs::read_to_string(data_dir.join("logs").join(file)) else {
            continue;
        };
        for line in text.lines().rev().take(60) {
            // "<ms> <message>": the time goes, the source is named.
            let message = line.split_once(' ').map_or("", |(_, rest)| rest);
            if let Some(kept) = scrub_log_line(&format!("{source}: {message}")) {
                lines.push(kept);
            }
        }
    }
    feedback::scrub_log_lines(lines.iter().map(String::as_str))
}

/// The app info a report may carry. Blocking (it runs three small system
/// commands): call it off the UI thread.
pub fn app_info(data_dir: &Path, sync: SyncMode, screen: Screen) -> AppInfo {
    let (os, device, locale) = system();
    AppInfo {
        platform: Platform::current(),
        version: core_version(&crate::updates::current_version()),
        build: clean_build(option_env!("KANSO_BUILD").unwrap_or("0")),
        patch: None,
        os: clean_os(&os),
        device: clean_device(&device),
        locale: clean_locale(&locale),
        screen,
        sync,
        errors: recent_errors(data_dir),
    }
}

// MARK: Who is sending

/// What a feedback job needs from the app, captured on the UI thread so the
/// job can run on a worker.
#[derive(Clone)]
pub struct Env {
    pub store: Arc<Store>,
    pub data_dir: std::path::PathBuf,
    session: Option<Arc<Session>>,
    secrets: Option<Arc<dyn SecretStore>>,
    machine: Option<Machine>,
    transport: Arc<dyn Transport>,
    /// The real network, not a test's stand-in.
    real: bool,
}

/// A test's stand-in for Kanso's Worker.
pub struct TransportOverride(pub Arc<dyn Transport>);

impl Global for TransportOverride {}

impl Env {
    pub fn capture(cx: &App) -> Option<Env> {
        let state = cx.try_global::<AppState>()?;
        let services = cx.try_global::<Services>();
        Some(Env {
            store: state.store.clone(),
            data_dir: state.data_dir.clone(),
            session: services.and_then(Services::session),
            secrets: services.and_then(|s| s.secrets.clone()),
            machine: services.and_then(|s| s.machine.clone()),
            transport: cx
                .try_global::<TransportOverride>()
                .map(|t| t.0.clone())
                .unwrap_or_else(|| Arc::new(HttpTransport::new())),
            real: cx.try_global::<TransportOverride>().is_none(),
        })
    }

    /// The account's name, for "Credit me by name".
    pub fn username(&self) -> Option<String> {
        self.session.as_ref().map(|s| s.username.clone())
    }

    pub fn sync_mode(&self) -> SyncMode {
        sync_mode(self.session.as_deref())
    }

    /// Signed in, or with the install's anonymous key. Blocking when the
    /// machine has to be read.
    pub fn reporter(&self) -> Option<Reporter> {
        if let Some(session) = &self.session {
            let machine = self.machine.clone().unwrap_or_else(Machine::this_mac);
            let device = kanso_sync::device_identity(&self.store, &machine).ok()?;
            return Some(Reporter::from_session(session, &device));
        }
        Reporter::anonymous(self.secrets.as_deref()?).ok()
    }

    pub fn client(&self) -> Client<Arc<dyn Transport>> {
        Client::new(self.transport.clone())
    }

    /// The same, giving up after `seconds`.
    pub fn quick_client(&self, seconds: u64) -> Client<Arc<dyn Transport>> {
        let transport: Arc<dyn Transport> = if self.real {
            Arc::new(HttpTransport::quick(seconds))
        } else {
            self.transport.clone()
        };
        Client::new(transport)
    }
}

// MARK: Retry

/// A retry is already waiting.
#[derive(Default)]
struct Scheduled(bool);

impl Global for Scheduled {}

/// Sends what is waiting, off the UI thread, then waits for the next retry
/// (1, 5, then 30 minutes; after that, the next launch).
// no-prop-test: wiring over the timer and the network; `retry_delay_ms`
// and the flow are tested.
pub fn flush_soon(cx: &mut App, after: Duration) {
    if cx.default_global::<Scheduled>().0 {
        return;
    }
    let Some(env) = Env::capture(cx) else {
        return;
    };
    cx.default_global::<Scheduled>().0 = true;
    cx.spawn(async move |cx| {
        if !after.is_zero() {
            cx.background_executor().timer(after).await;
        }
        let job = {
            let env = env.clone();
            cx.background_executor().spawn(async move {
                let reporter = env.reporter()?;
                feedback_flow::flush(&env.store, &env.client(), &reporter, kanso_core::now_ms())
                    .ok()?;
                // The next wait, from the longest-failing report.
                let attempts = env
                    .store
                    .feedback_all()
                    .ok()?
                    .iter()
                    .filter(|r| r.state != kanso_store::OutboxState::Sent)
                    .map(|r| r.attempts)
                    .max()?;
                feedback::retry_delay_ms(attempts)
            })
        };
        let next = job.await;
        cx.update(|cx| {
            cx.default_global::<Scheduled>().0 = false;
            if let Some(ms) = next.filter(|ms| *ms > 0) {
                flush_soon(cx, Duration::from_millis(ms as u64));
            }
        });
    })
    .detach();
}

/// At launch: every waiting report is due again, and goes.
// no-prop-test: wiring.
pub fn start(cx: &mut App) {
    let Some(env) = Env::capture(cx) else {
        return;
    };
    if env
        .store
        .feedback_all()
        .map_or(true, |rows| rows.is_empty())
    {
        return;
    }
    feedback_flow::store_retry_now(&env.store).ok();
    // After the launch check has had time to bring the session back: a
    // signed-in report sent as an anonymous one would not be accepted.
    flush_soon(cx, Duration::from_secs(20));
}

#[cfg(test)]
mod tests {
    use super::{
        PAUSED, RATE_LIMITED, SAVED, chip_text, core_version, placeholder, recent_errors,
        rejected_text, sync_mode, update_message, wait_text,
    };
    use kanso_engine::feedback::SyncMode;
    use kanso_engine::feedback::{self, Chip, Kind};
    use kanso_engine::feedback_flow::Wait;
    use proptest::prelude::*;

    #[test]
    fn the_words_match_the_spec() {
        assert_eq!(
            placeholder(Kind::Bug),
            "What happened, and what did you expect?"
        );
        assert_eq!(placeholder(Kind::Idea), "What would you like Kanso to do?");
        assert_eq!(chip_text(&Chip::FiledAs(241)), "Filed as #241");
        assert_eq!(
            chip_text(&Chip::ReleasedIn("v0.9.2".into())),
            "Released in 0.9.2"
        );
        assert_eq!(chip_text(&Chip::SameAs(198)), "Same as #198");
        assert_eq!(update_message(0), "See what changed.");
        assert_eq!(
            update_message(2),
            "See what changed. Fixes 2 of your reports."
        );
        assert_eq!(
            rejected_text("app.version"),
            "Kanso couldn't send this report (app.version). Edit it and try again."
        );
        assert_eq!(wait_text(&Wait::Paused), PAUSED);
        assert_eq!(
            wait_text(&Wait::RateLimited { retry_after_ms: 1 }),
            RATE_LIMITED
        );
        assert_eq!(wait_text(&Wait::Offline), SAVED);
    }

    #[test]
    fn versions_lose_their_tail() {
        let v = semver::Version::parse("0.9.4-dev.abc").unwrap();
        assert_eq!(core_version(&v), "0.9.4");
        assert!(feedback::is_version(&core_version(&v)));
    }

    #[test]
    fn only_grammar_lines_leave_the_logs() {
        let dir = tempfile::tempdir().unwrap();
        let logs = dir.path().join("logs");
        std::fs::create_dir_all(&logs).unwrap();
        std::fs::write(
            logs.join("updates.log"),
            "1730000000000 check failed: timeout\n1730000001000 downloading /Users/me/x\n1730000002000 status 503\n",
        )
        .unwrap();
        std::fs::write(
            logs.join("themes.log"),
            "1730000003000 install failed: signature\n",
        )
        .unwrap();
        let lines = recent_errors(dir.path());
        assert!(lines.contains(&"update: check failed: timeout".to_owned()));
        assert!(lines.contains(&"theme: install failed: signature".to_owned()));
        assert_eq!(lines.len(), 2);
        assert!(recent_errors(&dir.path().join("nothing")).is_empty());
    }

    #[test]
    fn sync_is_named_by_its_server() {
        assert_eq!(sync_mode(None), SyncMode::None);
    }

    proptest! {
        /// A server's field never carries anything but a JSON path into the
        /// words, whatever it sent; nothing is echoed with a quote, a space or markup.
        #[test]
        fn prop_rejected_text_shows_only_a_path(field in ".{0,80}") {
            let text = rejected_text(&field);
            let inner = text.split_once('(').map(|(_, rest)| rest.split_once(')').map_or("", |(f, _)| f)).unwrap_or("");
            prop_assert!(inner.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '[' | ']')));
            prop_assert!(inner.len() <= 40);
            prop_assert!(text.starts_with("Kanso couldn't send this report"));
        }

        /// The update message names exactly the count it was given.
        #[test]
        fn prop_update_message_counts(n in 0usize..1000) {
            let text = update_message(n);
            prop_assert!(text.starts_with("See what changed."));
            if n >= 2 {
                let counted = format!("{n} of your reports");
                prop_assert!(text.contains(&counted));
            } else if n == 0 {
                prop_assert!(!text.contains("Fixes"));
            }
        }
    }
}
