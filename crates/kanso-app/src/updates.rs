//! Updates from GitHub releases, as the window shows them
//! (docs/RELEASING.md › How updates work).
//!
//! Kanso checks a few seconds after it starts, every half hour (five
//! minutes after a check that failed, and within a minute of waking from
//! sleep), and when asked (the app menu, the palette, Settings › Updates). A newer release
//! downloads in the background, is checked against the signed feed and
//! staged beside the install; the sidebar then shows "Update ready ·
//! Restart". Restart now swaps it in and opens the new Kanso; quitting
//! installs it too. With automatic updates off, the sidebar says "Update
//! available · Download" instead.
//!
//! A copy apt installed (`/usr/bin`) is apt's to update: Kanso still says a
//! release is out, but its button copies `sudo apt upgrade` instead of
//! downloading anything or opening the release page.
//!
//! The work runs on threads with `kanso-update`; this module keeps the
//! state, words and views. Without a transport (tests and snapshots),
//! nothing reaches the network.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use gpui_kit::assets::IconName as Lucide;
use gpui_kit::base::text::TextView;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::notification::Notification;
use gpui_kit::component::switch::Switch;
use gpui_kit::component::{
    ActiveTheme as _, Icon, Sizable as _, StyledExt as _, WindowExt as _, h_flex, v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_update::stage::StageError;
use kanso_update::{
    Check, CheckError, FeedError, FetchError, Install, LocateOptions, NotInstallable, Pending,
    Platform, Transport, Update, UrlPolicy,
};
use serde::{Deserialize, Serialize};

use crate::clock::Clock;
use crate::flavour;
use crate::settings::{Row, StatusLine, section, status_row};
use crate::sidebar::Tone;
use crate::theme::Colors;
use crate::workspace::AppState;
use crate::zoom::px;

actions!(kanso, [CheckForUpdates, ShowUpdate, RestartToUpdate,]);

// The changes in the version running, from the Help menu and the palette.
actions!(kanso, [ShowWhatsNew]);

/// When the first check runs after Kanso starts, and how often after: by
/// the wall clock, looked at every minute, so a Mac that slept past a check
/// makes it up within a minute of waking (timers stop while it sleeps). A
/// check that failed, offline say, is tried again sooner.
const FIRST_CHECK: Duration = Duration::from_secs(8);
const CHECK_INTERVAL: Duration = Duration::from_secs(30 * 60);
const RETRY_INTERVAL: Duration = Duration::from_secs(5 * 60);
const TICK: Duration = Duration::from_secs(60);
/// The sheet's width and its notes' height.
pub const SHEET_WIDTH: f32 = 600.;
const NOTES_HEIGHT: f32 = 320.;

/// Store meta keys: the version that ran last, and the notes of the newest
/// release Kanso has heard of.
const META_LAST_RUN: &str = "update.last_run_version";
const META_NOTES: &str = "update.notes";

/// What this version is.
pub fn current_version() -> semver::Version {
    semver::Version::parse(env!("CARGO_PKG_VERSION")).expect("the crate's version is semver")
}

/// A release's notes, as the sheet and Settings show them.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseNotes {
    pub version: String,
    pub pub_date: String,
    pub release_url: String,
    pub notes: String,
}

impl From<&Update> for ReleaseNotes {
    fn from(update: &Update) -> Self {
        ReleaseNotes {
            version: update.version.clone(),
            pub_date: update.pub_date.clone(),
            release_url: update.release_url.clone(),
            notes: update.notes.clone(),
        }
    }
}

impl From<&Pending> for ReleaseNotes {
    fn from(pending: &Pending) -> Self {
        ReleaseNotes {
            version: pending.version.clone(),
            pub_date: pending.pub_date.clone(),
            release_url: pending.release_url.clone(),
            notes: pending.notes.clone(),
        }
    }
}

/// Why a check or a download didn't finish, in the few kinds a person can
/// tell apart and do something about.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Problem {
    Offline,
    RateLimited,
    /// No release is published yet.
    NoRelease,
    /// The feed or the download failed a check: never installed.
    Verification,
    NoSpace,
    /// The release needs a newer updater than this one.
    NewerSchema,
    Other,
}

impl Problem {
    /// One calm sentence for Settings › Updates.
    pub fn words(self) -> &'static str {
        match self {
            Problem::Offline => "Kanso couldn't reach GitHub. It tries again later.",
            Problem::RateLimited => "GitHub asked Kanso to wait. It tries again later.",
            Problem::NoRelease => "No release is published yet.",
            Problem::Verification => {
                "The update didn't pass Kanso's checks, so it wasn't installed. Kanso tries again later; the release page has the download."
            }
            Problem::NoSpace => "There isn't enough free space to download the update.",
            Problem::NewerSchema => "This release needs to be downloaded from its page.",
            Problem::Other => "Something went wrong getting the update. Kanso tries again later.",
        }
    }

    /// Whether the sidebar mentions it after a check nobody asked for:
    /// only when it needs a person (a failed check, a full disk), never for
    /// being offline.
    pub fn worth_showing(self) -> bool {
        matches!(
            self,
            Problem::Verification | Problem::NoSpace | Problem::NewerSchema
        )
    }
}

pub fn check_problem(err: &CheckError) -> Problem {
    match err {
        CheckError::Fetch(err) => fetch_problem(err),
        CheckError::Feed(FeedError::NewerSchema) => Problem::NewerSchema,
        CheckError::Feed(FeedError::NoBuildForPlatform) => Problem::NoRelease,
        CheckError::Feed(_) => Problem::Verification,
    }
}

fn fetch_problem(err: &FetchError) -> Problem {
    match err {
        FetchError::Offline => Problem::Offline,
        FetchError::RateLimited => Problem::RateLimited,
        FetchError::NotFound => Problem::NoRelease,
        FetchError::Policy(_) | FetchError::TooLarge | FetchError::TooManyRedirects => {
            Problem::Verification
        }
        FetchError::Io(io) if kanso_update::stage::is_no_space(io) => Problem::NoSpace,
        FetchError::Status(_) | FetchError::Io(_) => Problem::Offline,
    }
}

pub fn stage_problem(err: &StageError) -> Problem {
    match err {
        StageError::Fetch(err) => fetch_problem(err),
        StageError::NoSpace => Problem::NoSpace,
        StageError::SizeMismatch
        | StageError::ChecksumMismatch
        | StageError::BadArchive(_)
        | StageError::CodeSignature(_) => Problem::Verification,
        StageError::Cancelled | StageError::Io(_) => Problem::Other,
    }
}

/// Where the updater is.
#[derive(Clone, Debug, PartialEq)]
pub enum Phase {
    Idle,
    Checking,
    UpToDate,
    /// Newer, and waiting for "Download" (automatic updates are off, or
    /// this copy can't replace itself).
    Available(Update),
    Downloading {
        update: Update,
        done: u64,
        total: u64,
    },
    /// Staged: installs on restart or quit.
    Ready(Pending),
    Failed(Problem),
}

/// The app's updater.
pub struct Updates {
    pub phase: Phase,
    /// Whether someone asked for the check running or just finished.
    pub manual: bool,
    pub last_checked_ms: Option<i64>,
    /// Why this build never updates (Kanso Dev, no signing key, no release
    /// builds for this system).
    pub off: Option<String>,
    pub install: Result<Install, NotInstallable>,
    /// What's new in the version running, when Kanso heard of it.
    pub installed_notes: Option<ReleaseNotes>,
    /// Shown once after an update: "Updated to …".
    pub just_updated: Option<ReleaseNotes>,
    /// The network's seam: `HttpTransport` in the app; tests leave it out.
    pub transport: Option<Arc<dyn Transport>>,
    pub feed_url: String,
    pub policy: UrlPolicy,
    pub keys: Vec<kanso_update::VerifyingKey>,
    /// Open Kanso again once the quit that installs the update is done.
    restart: bool,
    running: bool,
    cancel: Arc<AtomicBool>,
    _schedule: Option<Task<()>>,
}

impl Global for Updates {}

impl Default for Updates {
    /// Off, with nothing installed or reachable: what tests start from.
    fn default() -> Self {
        Updates {
            phase: Phase::Idle,
            manual: false,
            last_checked_ms: None,
            off: None,
            install: Err(NotInstallable::DevBuild),
            installed_notes: None,
            just_updated: None,
            transport: None,
            feed_url: kanso_update::FEED_URL.to_owned(),
            policy: UrlPolicy::RELEASE,
            keys: Vec::new(),
            restart: false,
            running: false,
            cancel: Arc::new(AtomicBool::new(false)),
            _schedule: None,
        }
    }
}

// MARK: Starting

/// Why this build never updates, if it doesn't: Kanso Dev, a build without
/// a signing key to check with, or a system releases aren't built for.
pub fn off_reason(
    flavour: &flavour::Flavour,
    has_keys: bool,
    platform: Option<Platform>,
) -> Option<String> {
    if !flavour.updates {
        Some(NotInstallable::DevFlavour.reason())
    } else if platform.is_none() {
        Some(NotInstallable::UnsupportedSystem.reason())
    } else if !has_keys {
        Some("This build has no key to check updates with, so it doesn't look for them.".into())
    } else {
        None
    }
}

/// The notes to announce after an update: when the version that ran last
/// is older than this one. The notes are the ones Kanso downloaded the
/// update with, or just the version and its release page.
pub fn just_updated(
    last_run: Option<&str>,
    current: &semver::Version,
    known: Option<ReleaseNotes>,
) -> Option<ReleaseNotes> {
    let last = semver::Version::parse(last_run?).ok()?;
    if last >= *current {
        return None;
    }
    let version = current.to_string();
    Some(match known {
        Some(notes) if notes.version == version => notes,
        _ => ReleaseNotes {
            release_url: format!(
                "https://github.com/{}/releases/tag/v{version}",
                kanso_update::REPOSITORY
            ),
            version,
            pub_date: String::new(),
            notes: String::new(),
        },
    })
}

/// Sets the updater up for the real app: the keys compiled in, GitHub over
/// HTTPS, and where Kanso is installed. Then clears whatever an earlier run
/// staged, notes an update that just happened, and starts checking.
// no-prop-test: wiring to the system; `off_reason`, `just_updated` and
// `kanso_update::install::locate` are tested.
pub fn init(cx: &mut App) {
    let current = current_version();
    let mut updates = Updates {
        keys: kanso_update::trusted_keys(),
        transport: Some(Arc::new(kanso_update::HttpTransport::new(&format!(
            "Kanso/{current}"
        )))),
        ..Updates::default()
    };
    // A debug build can be pointed at a local feed, which must still be
    // signed by a trusted key. Release builds ignore the variable.
    if cfg!(debug_assertions)
        && let Some(url) = std::env::var("KANSO_UPDATE_FEED")
            .ok()
            .filter(|u| !u.is_empty())
    {
        updates.feed_url = url;
        updates.policy.allow_loopback = true;
    }
    let platform = Platform::current();
    updates.off = off_reason(&flavour::CURRENT, !updates.keys.is_empty(), platform);
    updates.install = std::env::current_exe()
        .map_err(|_| NotInstallable::NotInBundle)
        .and_then(|exe| {
            kanso_update::install::locate(
                &exe,
                platform,
                LocateOptions {
                    dev_flavour: flavour::CURRENT.is_dev(),
                    debug_build: cfg!(debug_assertions),
                    appimage: cfg!(target_os = "linux") && std::env::var_os("APPIMAGE").is_some(),
                },
            )
        });
    cx.set_global(updates);
    after_launch(cx);
}

/// What happens at every start, whatever set the updater up: leftovers go
/// (an update staged by an earlier run included), a finished one is
/// announced once.
pub fn after_launch(cx: &mut App) {
    let current = current_version();
    let store = cx.global::<AppState>().store.clone();
    let known = store
        .meta(META_NOTES)
        .ok()
        .flatten()
        .and_then(|json| serde_json::from_str::<ReleaseNotes>(&json).ok());
    let last_run = store.meta(META_LAST_RUN).ok().flatten();
    let announce = just_updated(last_run.as_deref(), &current, known.clone());
    store.set_meta(META_LAST_RUN, &current.to_string()).ok();
    let off = {
        let updates = cx.global_mut::<Updates>();
        updates.installed_notes = known.filter(|n| n.version == current.to_string());
        if let Some(notes) = announce {
            log_line(&format!("updated to {}", notes.version));
            updates.installed_notes.get_or_insert_with(|| notes.clone());
            updates.just_updated = Some(notes);
        }
        // Whatever an earlier run left staged goes: nothing on disk can say
        // it is still what was checked, so a newer release is downloaded
        // and checked again instead (kanso_update::install::clean_up).
        if let Ok(install) = &updates.install {
            kanso_update::install::clean_up(install);
        }
        updates.off.is_some()
    };
    if !off {
        schedule(cx);
    }
}

/// Checks a few seconds from now, then whenever [`check_due`] says so.
fn schedule(cx: &mut App) {
    let task = cx.spawn(async move |cx| {
        cx.background_executor().timer(FIRST_CHECK).await;
        cx.update(|cx| check_now(false, cx));
        loop {
            cx.background_executor().timer(TICK).await;
            cx.update(|cx| {
                let updates = cx.global::<Updates>();
                let failed = matches!(updates.phase, Phase::Failed(_));
                if check_due(updates.last_checked_ms, failed, Clock::now_ms(cx)) {
                    check_now(false, cx);
                }
            });
        }
    });
    cx.global_mut::<Updates>()._schedule = Some(task);
}

/// Whether a background check is due at `now`: none has run, the last was
/// [`CHECK_INTERVAL`] ago ([`RETRY_INTERVAL`] if it failed), or the clock
/// went back past it. Pure.
pub(crate) fn check_due(last_checked_ms: Option<i64>, failed: bool, now: i64) -> bool {
    let Some(last) = last_checked_ms else {
        return true;
    };
    let every = if failed {
        RETRY_INTERVAL
    } else {
        CHECK_INTERVAL
    };
    now < last || now - last >= every.as_millis() as i64
}

/// Appends a line to the updater's log in the data folder (kept under
/// 256 KB) and to standard error.
fn log_line(line: &str) {
    eprintln!("kanso: update: {line}");
    let Some(dir) = LOG_DIR.get() else {
        return;
    };
    let path = dir.join("updates.log");
    if std::fs::metadata(&path).is_ok_and(|m| m.len() > 256 * 1024) {
        std::fs::rename(&path, dir.join("updates.log.1")).ok();
    }
    use std::io::Write as _;
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        let now = kanso_core::now_ms();
        writeln!(file, "{now} {line}").ok();
    }
}

static LOG_DIR: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();

/// Where [`log_line`] writes: `<data folder>/logs`.
pub fn set_log_dir(data_dir: &std::path::Path) {
    let dir = data_dir.join("logs");
    if std::fs::create_dir_all(&dir).is_ok() {
        LOG_DIR.set(dir).ok();
    }
}

// MARK: Checking and downloading

/// Checks for an update now. `manual` when someone asked: the result shows
/// even when it is "up to date" or a problem a background check keeps
/// quiet about.
pub fn check_now(manual: bool, cx: &mut App) {
    let current = current_version();
    let updates = cx.global_mut::<Updates>();
    if manual {
        updates.manual = true;
    }
    if updates.off.is_some()
        || updates.running
        || matches!(updates.phase, Phase::Ready(_) | Phase::Downloading { .. })
    {
        cx.refresh_windows();
        return;
    }
    let Some(transport) = updates.transport.clone() else {
        return;
    };
    let Some(platform) = Platform::current() else {
        return;
    };
    updates.running = true;
    updates.manual = manual;
    updates.phase = Phase::Checking;
    let (policy, feed_url, keys) = (
        updates.policy,
        updates.feed_url.clone(),
        updates.keys.clone(),
    );
    let (tx, rx) = async_channel::bounded(1);
    let spawned = std::thread::Builder::new()
        .name("kanso-update-check".into())
        .spawn(move || {
            let result = kanso_update::check(
                transport.as_ref(),
                &policy,
                &feed_url,
                &keys,
                &current,
                platform,
            );
            tx.try_send(result).ok();
        });
    if spawned.is_err() {
        finish_check(None, cx);
        return;
    }
    cx.refresh_windows();
    cx.spawn(async move |cx| {
        let result = rx.recv().await.ok();
        cx.update(|cx| finish_check(result, cx));
    })
    .detach();
}

fn finish_check(result: Option<Result<Check, CheckError>>, cx: &mut App) {
    let now = Clock::now_ms(cx);
    let auto = cx.global::<AppState>().config.updates.auto_install;
    let store = cx.global::<AppState>().store.clone();
    let updates = cx.global_mut::<Updates>();
    updates.running = false;
    updates.last_checked_ms = Some(now);
    match result {
        Some(Ok(Check::UpToDate { latest })) => {
            log_line(&format!("up to date (latest release {latest})"));
            updates.phase = Phase::UpToDate;
        }
        Some(Ok(Check::Available(update))) => {
            log_line(&format!("{} is available", update.version));
            if let Ok(json) = serde_json::to_string(&ReleaseNotes::from(&update)) {
                store.set_meta(META_NOTES, &json).ok();
            }
            if auto && updates.install.is_ok() {
                start_download(update, cx);
                return;
            }
            updates.phase = Phase::Available(update);
        }
        Some(Err(err)) => {
            log_line(&format!("check failed: {err}"));
            updates.phase = Phase::Failed(check_problem(&err));
        }
        None => {
            log_line("check failed: the thread stopped");
            updates.phase = Phase::Failed(Problem::Other);
        }
    }
    cx.refresh_windows();
}

enum Progress {
    Bytes(u64, u64),
    Done(Box<Result<Pending, StageError>>),
}

/// Downloads, checks and stages `update` in the background. Without an
/// install Kanso can replace, it opens the release page instead.
pub fn start_download(update: Update, cx: &mut App) {
    let updates = cx.global_mut::<Updates>();
    let (Ok(install), Some(transport)) = (updates.install.clone(), updates.transport.clone())
    else {
        updates.phase = Phase::Available(update);
        cx.refresh_windows();
        return;
    };
    let cancel = Arc::new(AtomicBool::new(false));
    updates.cancel = cancel.clone();
    updates.running = true;
    updates.phase = Phase::Downloading {
        total: update.asset.size,
        done: 0,
        update: update.clone(),
    };
    let policy = updates.policy;
    let (tx, rx) = async_channel::unbounded();
    let spawned = std::thread::Builder::new()
        .name("kanso-update-download".into())
        .spawn(move || {
            let mut last_percent = None;
            let mut progress = |done: u64, total: u64| {
                let percent = percent(done, total);
                if last_percent != Some(percent) {
                    last_percent = Some(percent);
                    tx.try_send(Progress::Bytes(done, total)).ok();
                }
            };
            let verify = |payload: &std::path::Path| -> Result<(), String> {
                #[cfg(target_os = "macos")]
                return kanso_update::install::verify_bundle(payload, Some(&install.target));
                #[cfg(not(target_os = "macos"))]
                {
                    let _ = payload;
                    Ok(())
                }
            };
            let result = kanso_update::stage::prepare(
                transport.as_ref(),
                &policy,
                &update,
                &install,
                &mut progress,
                &cancel,
                &verify,
            );
            tx.try_send(Progress::Done(Box::new(result))).ok();
        });
    if spawned.is_err() {
        let updates = cx.global_mut::<Updates>();
        updates.running = false;
        updates.phase = Phase::Failed(Problem::Other);
        cx.refresh_windows();
        return;
    }
    cx.refresh_windows();
    cx.spawn(async move |cx| {
        while let Ok(message) = rx.recv().await {
            let done = matches!(message, Progress::Done(_));
            cx.update(|cx| progress(message, cx));
            if done {
                break;
            }
        }
    })
    .detach();
}

fn progress(message: Progress, cx: &mut App) {
    let updates = cx.global_mut::<Updates>();
    match message {
        Progress::Bytes(now_done, now_total) => {
            if let Phase::Downloading { done, total, .. } = &mut updates.phase {
                *done = now_done;
                *total = now_total;
            }
        }
        Progress::Done(result) => {
            updates.running = false;
            match *result {
                Ok(pending) => {
                    log_line(&format!(
                        "{} is staged at {}",
                        pending.version,
                        pending.payload.display()
                    ));
                    updates.phase = Phase::Ready(pending);
                }
                Err(StageError::Cancelled) => {}
                Err(err) => {
                    log_line(&format!("download failed: {err}"));
                    updates.phase = Phase::Failed(stage_problem(&err));
                }
            }
        }
    }
    cx.refresh_windows();
}

/// Whole percent of `done` out of `total`.
pub fn percent(done: u64, total: u64) -> u8 {
    if total == 0 {
        return 0;
    }
    (done.min(total).saturating_mul(100) / total) as u8
}

/// Whether apt installed this copy, so apt updates it.
pub fn package_managed(install: &Result<Install, NotInstallable>) -> bool {
    matches!(install, Err(NotInstallable::PackageManaged))
}

/// What the button for an available update says: Download, or for a copy
/// apt installed, the command to copy.
pub fn download_label(install: &Result<Install, NotInstallable>) -> &'static str {
    if package_managed(install) {
        "Copy Command"
    } else {
        "Download"
    }
}

/// The pill's words for this install: an apt copy's pill names apt.
pub fn pill_label(label: String, install: &Result<Install, NotInstallable>) -> String {
    match label.strip_suffix(" · Download") {
        Some(what) if package_managed(install) => format!("{what} · apt upgrade"),
        _ => label,
    }
}

/// "Download": stages the available update when this copy can replace
/// itself, copies `sudo apt upgrade` for a copy apt installed (never
/// downloading or opening anything), and otherwise opens its release page.
/// Returns whether it copied the command.
pub fn download(cx: &mut App) -> bool {
    let updates = cx.global::<Updates>();
    let Phase::Available(update) = updates.phase.clone() else {
        return false;
    };
    if package_managed(&updates.install) {
        cx.write_to_clipboard(ClipboardItem::new_string(
            kanso_update::install::APT_UPGRADE.to_owned(),
        ));
        return true;
    }
    if updates.install.is_ok() && updates.transport.is_some() {
        start_download(update, cx);
    } else {
        open_release_page(&update.release_url, cx);
    }
    false
}

/// Says the command was copied, when [`download`] copied it.
fn say_copied(copied: bool, window: &mut Window, cx: &mut App) {
    if copied {
        window.push_notification(
            Notification::info(format!(
                "Copied “{}”. Run it in a terminal to update Kanso.",
                kanso_update::install::APT_UPGRADE
            )),
            cx,
        );
    }
}

/// Opens a release page the feed named, if it is one of this repository's.
pub fn open_release_page(url: &str, cx: &mut App) {
    let policy = cx.global::<Updates>().policy;
    if policy.check_release_page(url).is_ok() {
        cx.open_url(url);
    }
}

// MARK: Installing

/// "Restart now": quits, installing the update on the way out, and opens
/// the new Kanso.
///
/// It quits directly rather than dispatching `Quit`: its callers run while
/// their window is being updated, and `App::dispatch_action` sends an action
/// through the active window, which GPUI can't update a second time. The
/// action was dropped (only logged), so Restart did nothing until the next
/// quit.
pub fn restart_now(cx: &mut App) {
    let updates = cx.global_mut::<Updates>();
    if !matches!(updates.phase, Phase::Ready(_)) {
        return;
    }
    updates.restart = true;
    crate::quit(cx);
}

/// On every quit: a staged update is swapped in, and Kanso opens again if
/// that's why it quit. A download still running stops.
// no-prop-test: the swap is `kanso_update::install::apply`, tested there
// and end to end; this only calls it on the way out.
pub fn install_on_quit(cx: &mut App) {
    let data_dir = cx.try_global::<AppState>().map(|s| s.data_dir.clone());
    if !cx.has_global::<Updates>() {
        return;
    }
    let updates = cx.global_mut::<Updates>();
    updates.cancel.store(true, Ordering::Relaxed);
    let (Phase::Ready(pending), Ok(install)) = (&updates.phase, &updates.install) else {
        return;
    };
    match kanso_update::install::apply(pending, install) {
        Ok(()) => {
            log_line(&format!("installed {}", pending.version));
            #[cfg(target_os = "macos")]
            kanso_update::install::remove_quarantine(&install.target);
        }
        Err(err) => log_line(&format!("installing {} failed: {err}", pending.version)),
    }
    if updates.restart {
        let args: Vec<String> = data_dir
            .map(|dir| vec!["--data-dir".to_owned(), dir.to_string_lossy().into_owned()])
            .unwrap_or_default();
        if let Err(err) = kanso_update::install::relaunch(install, &args) {
            log_line(&format!("couldn't open Kanso again: {err}"));
        }
    }
}

// MARK: Words

/// How the sidebar's pill looks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PillKind {
    /// Checking or downloading: quiet.
    Busy,
    /// Staged: the accent, the one that asks for something.
    Ready,
    /// Out, waiting for Download.
    Available,
    Failed,
}

/// The sidebar's pill: what it says and how it looks, if it shows at all.
/// It shows while there is something to do or see, and stays out of the
/// way otherwise: a check nobody asked for shows nothing until it finds an
/// update, and only failures that need a person.
pub fn pill(phase: &Phase, manual: bool) -> Option<(PillKind, String)> {
    match phase {
        Phase::Idle | Phase::UpToDate => None,
        Phase::Checking => manual.then(|| (PillKind::Busy, "Checking for updates…".into())),
        Phase::Downloading { done, total, .. } => Some((
            PillKind::Busy,
            format!("Downloading · {}%", percent(*done, *total)),
        )),
        Phase::Ready(_) => Some((PillKind::Ready, "Update ready · Restart".into())),
        Phase::Available(_) => Some((PillKind::Available, "Update available · Download".into())),
        Phase::Failed(problem) => (manual || problem.worth_showing())
            .then(|| (PillKind::Failed, "Couldn't update · Details".into())),
    }
}

/// Settings › Updates' status line.
pub fn status(
    phase: &Phase,
    off: Option<&str>,
    install: &Result<Install, NotInstallable>,
    current: &str,
    auto: bool,
) -> StatusLine {
    let line = |tone, label: String, detail: &str| StatusLine {
        tone,
        label,
        detail: detail.to_owned(),
    };
    if let Some(reason) = off {
        let label = reason.trim_end_matches('.').to_owned();
        let detail = if label.starts_with("Development build") {
            "Kanso Dev never replaces itself or the Kanso you installed. Releases update as Kanso."
        } else {
            "Download new releases from GitHub."
        };
        return line(Tone::Quiet, label, detail);
    }
    match phase {
        Phase::Idle => line(
            Tone::Quiet,
            format!("Kanso {current}"),
            "Kanso looks for updates shortly after it opens, and every half hour.",
        ),
        Phase::Checking => line(
            Tone::Busy,
            "Checking for updates…".into(),
            "Asking GitHub for the newest release.",
        ),
        Phase::UpToDate => line(
            Tone::Good,
            format!("Kanso {current} is up to date"),
            "Kanso looks again every half hour.",
        ),
        Phase::Available(update) => {
            let detail = match install {
                Err(reason) => reason.reason(),
                Ok(_) if auto => "Download it now, and it installs when you restart.".into(),
                Ok(_) => "Download it now, or turn on automatic updates below.".into(),
            };
            line(
                Tone::Busy,
                format!("Kanso {} is available", update.version),
                &detail,
            )
        }
        Phase::Downloading {
            update,
            done,
            total,
        } => line(
            Tone::Busy,
            format!(
                "Downloading Kanso {} · {}%",
                update.version,
                percent(*done, *total)
            ),
            "It installs when you restart or quit Kanso.",
        ),
        Phase::Ready(pending) => line(
            Tone::Good,
            format!("Kanso {} is ready to install", pending.version),
            "Restart Kanso to finish. Quitting installs it too.",
        ),
        Phase::Failed(problem) => line(Tone::Warning, "Couldn't update".into(), problem.words()),
    }
}

/// `2026-10-20T10:00:00Z` as "October 20, 2026", as the note list names
/// days; `None` for anything else.
pub fn release_date(pub_date: &str) -> Option<String> {
    const MONTHS: [&str; 12] = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    let date = pub_date.get(..10)?;
    let mut parts = date.split('-');
    let (y, m, d) = (parts.next()?, parts.next()?, parts.next()?);
    if y.len() != 4 || m.len() != 2 || d.len() != 2 {
        return None;
    }
    let (y, m, d): (u32, usize, u32) = (y.parse().ok()?, m.parse().ok()?, d.parse().ok()?);
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    Some(format!("{} {d}, {y}", MONTHS[m - 1]))
}

// MARK: Views

/// The sidebar's pill, above the footer, when [`pill`] says it shows.
// no-prop-test: drawing; `pill` is tested and the snapshots show each state.
pub fn render_pill(cx: &App) -> Option<AnyElement> {
    let updates = cx.try_global::<Updates>()?;
    let (kind, label) = pill(&updates.phase, updates.manual)?;
    let label = pill_label(label, &updates.install);
    let colors = Colors::get(cx);
    let (bg, fg, icon) = match kind {
        PillKind::Ready => (colors.button, colors.button_text, Lucide::CircleArrowDown),
        PillKind::Available => (colors.fill, colors.link, Lucide::CircleArrowDown),
        PillKind::Busy => (colors.fill, colors.text_secondary, Lucide::LoaderCircle),
        PillKind::Failed => (colors.fill, colors.warning, Lucide::TriangleAlert),
    };
    let opens_sheet = matches!(kind, PillKind::Ready | PillKind::Available);
    // "Update ready" on the left, what a click does on the right.
    let (what, act) = match label.split_once(" · ") {
        Some((what, act)) => (what.to_owned(), Some(act.to_owned())),
        None => (label, None),
    };
    Some(
        h_flex()
            .id("update-pill")
            .debug_selector(|| "update-pill".into())
            .flex_none()
            .mx(px(8.))
            .mb(px(10.))
            .h(px(30.))
            .px(px(10.))
            .gap(px(6.))
            .rounded(px(15.))
            .bg(bg)
            .text_color(fg)
            .text_size(px(12.))
            .cursor_pointer()
            .hover(|el| el.opacity(0.9))
            .child(Icon::new(icon).size(px(13.)).text_color(fg))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .font_medium()
                    .child(what),
            )
            .children(act.map(|act| {
                div()
                    .flex_none()
                    .whitespace_nowrap()
                    .font_semibold()
                    .child(act)
            }))
            .on_click(move |_, window, cx| {
                if opens_sheet {
                    window.dispatch_action(Box::new(ShowUpdate), cx)
                } else {
                    window.dispatch_action(Box::new(crate::settings::OpenUpdateSettings), cx)
                }
            })
            .into_any_element(),
    )
}

/// The rail's button for the same states: the pill's icon alone.
// no-prop-test: drawing; the rail snapshots show it.
pub fn render_rail_button(cx: &App) -> Option<AnyElement> {
    let updates = cx.try_global::<Updates>()?;
    let (kind, label) = pill(&updates.phase, updates.manual)?;
    let label = pill_label(label, &updates.install);
    let colors = Colors::get(cx);
    let (color, icon) = match kind {
        PillKind::Ready => (colors.accent, Lucide::CircleArrowDown),
        PillKind::Available => (colors.link, Lucide::CircleArrowDown),
        PillKind::Busy => (colors.text_secondary, Lucide::LoaderCircle),
        PillKind::Failed => (colors.warning, Lucide::TriangleAlert),
    };
    let opens_sheet = matches!(kind, PillKind::Ready | PillKind::Available);
    Some(
        div()
            .id("rail-update")
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .size(px(34.))
            .rounded(px(6.))
            .cursor_pointer()
            .hover(|el| el.bg(colors.sidebar_hover))
            .child(Icon::new(icon).size(px(16.)).text_color(color))
            .tooltip(move |window, cx| {
                gpui_kit::component::tooltip::Tooltip::new(label.clone()).build(window, cx)
            })
            .on_click(move |_, window, cx| {
                if opens_sheet {
                    window.dispatch_action(Box::new(ShowUpdate), cx)
                } else {
                    window.dispatch_action(Box::new(crate::settings::OpenUpdateSettings), cx)
                }
            })
            .into_any_element(),
    )
}

/// What the sheet is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SheetKind {
    Ready,
    Available,
    /// After an update, or from Settings: what's new in a release.
    Installed,
}

/// The sheet for the update in hand: ready to restart into, or out and
/// waiting for Download. With neither, What's new for this version.
pub fn open_sheet(window: &mut Window, cx: &mut App) {
    let updates = cx.global::<Updates>();
    let (notes, kind) = match &updates.phase {
        Phase::Ready(pending) => (ReleaseNotes::from(pending), SheetKind::Ready),
        Phase::Available(update) | Phase::Downloading { update, .. } => {
            (ReleaseNotes::from(update), SheetKind::Available)
        }
        _ => match &updates.installed_notes {
            Some(notes) => (notes.clone(), SheetKind::Installed),
            None => return,
        },
    };
    open_notes(notes, kind, window, cx);
}

/// A release's notes in a sheet: its version and date, the changelog with
/// its commit links, and the buttons for `kind`.
// no-prop-test: drawing and buttons; the snapshots show each kind.
pub fn open_notes(notes: ReleaseNotes, kind: SheetKind, window: &mut Window, cx: &mut App) {
    let title = match kind {
        SheetKind::Ready => format!("Kanso {} is ready", notes.version),
        SheetKind::Available => format!("Kanso {} is available", notes.version),
        SheetKind::Installed => format!("What's new in Kanso {}", notes.version),
    };
    let body = kanso_update::notes::for_app(&notes.notes);
    let date = release_date(&notes.pub_date);
    let release_url = notes.release_url.clone();
    window.open_alert_dialog(cx, move |alert, _, cx| {
        let colors = Colors::get(cx);
        let dark = cx.theme().is_dark();
        let lead = match (kind, &date) {
            (SheetKind::Ready, Some(date)) => format!(
                "Released {date}. It's downloaded and checked; restart Kanso to start using it."
            ),
            (SheetKind::Ready, None) => {
                "It's downloaded and checked; restart Kanso to start using it.".to_owned()
            }
            (_, Some(date)) => format!("Released {date}."),
            (_, None) => String::new(),
        };
        let shown = if body.is_empty() {
            "This release's notes are on its page.".to_owned()
        } else {
            crate::preview_chunks::showable(&body).into_owned()
        };
        let release_url = release_url.clone();
        let page = release_url.clone();
        let content = v_flex()
            .gap(px(12.))
            .when(!lead.is_empty(), |el| {
                el.child(
                    div()
                        .text_size(px(13.))
                        .line_height(px(19.))
                        .text_color(colors.text_secondary)
                        .child(lead.clone()),
                )
            })
            .child(
                div()
                    .id("update-notes")
                    .max_h(px(NOTES_HEIGHT))
                    .overflow_y_scroll()
                    .border_1()
                    .border_color(colors.divider)
                    .rounded(px(8.))
                    .bg(colors.editor)
                    .px(px(16.))
                    .py(px(12.))
                    .child(
                        TextView::markdown("update-notes-text", shown)
                            .style(crate::preview::text_style(
                                colors,
                                &crate::themes::preview_colors(cx),
                                dark,
                            ))
                            .text_size(px(13.5))
                            .line_height(relative(1.55))
                            .text_color(colors.text)
                            .on_link_click(|url, _, _, cx| {
                                if crate::preview::is_openable(url) {
                                    cx.open_url(url);
                                }
                            }),
                    ),
            )
            .child(
                h_flex().child(
                    Button::new("release-page")
                        .ghost()
                        .small()
                        .icon(Lucide::ExternalLink)
                        .label("Release page")
                        .on_click(move |_, _, cx| open_release_page(&page, cx)),
                ),
            );
        let alert = alert
            .title(title.clone())
            .width(px(SHEET_WIDTH))
            .child(content);
        match kind {
            SheetKind::Ready => alert
                .show_cancel(true)
                .cancel_text("Later")
                .ok_text("Restart now")
                .on_ok(|_, _, cx| {
                    restart_now(cx);
                    true
                }),
            SheetKind::Available => alert
                .show_cancel(true)
                .cancel_text("Later")
                .ok_text(download_label(&cx.global::<Updates>().install))
                .on_ok(|_, window, cx| {
                    let copied = download(cx);
                    say_copied(copied, window, cx);
                    true
                }),
            SheetKind::Installed => alert.show_cancel(false).ok_text("Done"),
        }
    });
}

/// After an update, once: "Updated to Kanso 0.2.0", with What's new. When
/// the person has sent feedback, the server is asked (for a few seconds at
/// most) whether this release fixed any of it, and the notice says so.
// no-prop-test: a notification; `just_updated` decides when, and a
// snapshot shows it. `update_message` is tested.
pub fn announce(window: &mut Window, cx: &mut App) {
    if !cx.has_global::<Updates>() {
        return;
    }
    let Some(notes) = cx.global_mut::<Updates>().just_updated.take() else {
        return;
    };
    let env = crate::feedback_app::Env::capture(cx)
        .filter(|env| env.store.feedback_all().is_ok_and(|rows| !rows.is_empty()));
    let Some(env) = env else {
        push_update_notice(notes, 0, window, cx);
        return;
    };
    let version = notes.version.to_string();
    let task = cx.background_executor().spawn(async move {
        let reporter = env.reporter()?;
        let client = env.quick_client(4);
        let views = kanso_engine::feedback_flow::refresh(
            &env.store,
            &client,
            &reporter,
            kanso_core::now_ms(),
        )
        .ok()?;
        Some(kanso_engine::feedback::fixed_in(&views, &version))
    });
    window
        .spawn(cx, async move |cx| {
            let fixed = task.await.unwrap_or(0);
            cx.update(|window, cx| push_update_notice(notes, fixed, window, cx))
                .ok();
        })
        .detach();
}

/// The "Updated to Kanso X" notice, with a second button when the release
/// fixed `fixed` of this person's reports.
fn push_update_notice(notes: ReleaseNotes, fixed: usize, window: &mut Window, cx: &mut App) {
    let title = format!("Updated to Kanso {}", notes.version);
    let message = crate::feedback_app::update_message(fixed);
    let open = move |notes: ReleaseNotes| {
        Button::new("whats-new")
            .small()
            .primary()
            .label("What's new")
            .on_click(move |_, window, cx| {
                open_notes(notes.clone(), SheetKind::Installed, window, cx)
            })
    };
    let notification = Notification::new()
        .title(title)
        .icon(Icon::new(Lucide::Sparkles));
    let notification = if fixed == 0 {
        notification
            .message(message)
            .action(move |_, _, _| open(notes.clone()))
    } else {
        // Two buttons: the notice's own content holds both.
        notification.content(move |_, _, cx| {
            let colors = Colors::get(cx);
            v_flex()
                .gap(px(8.))
                .child(
                    div()
                        .text_size(px(13.))
                        .text_color(colors.text_secondary)
                        .child(message.clone()),
                )
                .child(
                    h_flex().gap_2().child(open(notes.clone())).child(
                        Button::new("my-feedback")
                            .small()
                            .label("My feedback")
                            .on_click(|_, window, cx| {
                                window.dispatch_action(
                                    Box::new(crate::feedback_sheet::MyFeedback),
                                    cx,
                                )
                            }),
                    ),
                )
                .into_any_element()
        })
    };
    window.push_notification(notification, cx);
}

/// Settings › Updates.
// no-prop-test: layout; `status` is tested and the snapshots show the page.
pub fn page(cx: &mut Context<crate::settings::SettingsView>) -> Vec<AnyElement> {
    let colors = Colors::get(cx);
    let auto = cx.global::<AppState>().config.updates.auto_install;
    let current = current_version().to_string();
    let Some(updates) = cx.try_global::<Updates>() else {
        return Vec::new();
    };
    let status_line = status(
        &updates.phase,
        updates.off.as_deref(),
        &updates.install,
        &current,
        auto,
    );
    let off = updates.off.is_some();
    let action: Option<AnyElement> = (!off).then(|| match &updates.phase {
        Phase::Ready(_) => Button::new("restart-update")
            .primary()
            .small()
            .label("Restart now")
            .on_click(|_, _, cx| restart_now(cx))
            .into_any_element(),
        Phase::Available(_) => Button::new("download-update")
            .primary()
            .small()
            .label(download_label(&updates.install))
            .on_click(|_, window, cx| {
                let copied = download(cx);
                say_copied(copied, window, cx);
            })
            .into_any_element(),
        phase => Button::new("check-updates")
            .outline()
            .small()
            .label("Check now")
            .debug_selector(|| "check-updates".into())
            .loading(matches!(phase, Phase::Checking))
            .on_click(|_, _, cx| check_now(true, cx))
            .into_any_element(),
    });
    let last_checked = match updates.last_checked_ms {
        Some(at) => Clock::edited_label(cx, at),
        None => "Not yet".to_owned(),
    };
    let notes = match &updates.phase {
        Phase::Ready(pending) => Some(ReleaseNotes::from(pending)),
        Phase::Available(update) | Phase::Downloading { update, .. } => {
            Some(ReleaseNotes::from(update))
        }
        _ => updates.installed_notes.clone(),
    };
    let cannot_install = match (&updates.install, off) {
        (Err(reason), false) => Some(reason.reason()),
        _ => None,
    };
    let mut rows = vec![
        Row::new(Lucide::Info, "Version").value(current.clone()),
        Row::new(Lucide::Clock, "Last checked").value(if off {
            "Never: updates are off".to_owned()
        } else {
            last_checked
        }),
    ];
    if !off {
        rows.push(
            Row::new(Lucide::RefreshCw, "Download and install updates automatically")
                .detail("New releases download in the background, are checked, and install when you restart or quit Kanso.")
                .trailing(
                    div().debug_selector(|| "auto-update".into()).child(
                        Switch::new("auto-update")
                            .checked(auto)
                            .on_click(|on, _, cx| set_auto_install(*on, cx)),
                    ),
                ),
        );
    }
    rows.push(match notes {
        Some(notes) => {
            let label = format!("The changes in Kanso {}.", notes.version);
            let kind = match &updates.phase {
                Phase::Ready(_) => SheetKind::Ready,
                Phase::Available(_) | Phase::Downloading { .. } => SheetKind::Available,
                _ => SheetKind::Installed,
            };
            Row::new(Lucide::Sparkles, "What's new")
                .detail(label)
                .trailing(
                    Button::new("whats-new")
                        .outline()
                        .small()
                        .label("What's new")
                        .on_click(move |_, window, cx| open_notes(notes.clone(), kind, window, cx)),
                )
        }
        None => Row::new(Lucide::Sparkles, "What's new")
            .detail("Every release's changes, with links to each commit, are on GitHub.")
            .trailing(
                Button::new("all-releases")
                    .outline()
                    .small()
                    .label("Releases")
                    .on_click(|_, _, cx| {
                        cx.open_url(&format!(
                            "https://github.com/{}/releases",
                            kanso_update::REPOSITORY
                        ))
                    }),
            ),
    });
    if let Some(reason) = cannot_install {
        rows.push(Row::new(Lucide::TriangleAlert, "Installing updates").detail(reason));
    }
    vec![
        status_row(&status_line, status_line.label.clone(), action, colors),
        section("", rows, colors),
    ]
}

fn set_auto_install(on: bool, cx: &mut App) {
    AppState::update_config(cx, |config| config.updates.auto_install = on);
    // Turned on with an update waiting: it downloads now.
    if on
        && let Some(Phase::Available(update)) = cx.try_global::<Updates>().map(|u| u.phase.clone())
        && cx.global::<Updates>().install.is_ok()
    {
        start_download(update, cx);
    }
    cx.refresh_windows();
}

/// The workspace's handlers for the sheet and Restart.
// no-prop-test: wiring; the UI tests send each action.
pub fn on_actions(el: Div, cx: &mut Context<crate::workspace::Workspace>) -> Div {
    el.on_action(cx.listener(|_, _: &ShowUpdate, window, cx| open_sheet(window, cx)))
        .on_action(cx.listener(|_, _: &RestartToUpdate, _, cx| restart_now(cx)))
        .on_action(cx.listener(|_, _: &ShowWhatsNew, window, cx| show_whats_new(window, cx)))
        .on_action(cx.listener(|this, _: &CheckForUpdates, window, cx| {
            this.open_settings(Some(crate::settings::SettingsPage::Updates), window, cx);
            check_now(true, cx);
        }))
}

/// The GitHub page with `version`'s changes: its release, or every
/// release for a build between releases (`0.3.0-dev.abc`). Pure.
pub fn release_page(version: &semver::Version) -> String {
    let repository = kanso_update::REPOSITORY;
    if version.pre.is_empty() {
        format!("https://github.com/{repository}/releases/tag/v{version}")
    } else {
        format!("https://github.com/{repository}/releases")
    }
}

/// What's new: this version's notes when Kanso has them, otherwise its
/// release page.
// no-prop-test: a sheet or a link; `release_page` is tested.
pub fn show_whats_new(window: &mut Window, cx: &mut App) {
    let notes = cx
        .try_global::<Updates>()
        .and_then(|updates| updates.installed_notes.clone());
    match notes {
        Some(notes) => open_notes(notes, SheetKind::Installed, window, cx),
        None => cx.open_url(&release_page(&current_version())),
    }
}

/// Check for Updates… from the menu bar with no window to take it.
// no-prop-test: wiring.
pub fn register_app_actions(cx: &mut App) {
    cx.on_action(|_: &ShowWhatsNew, cx| cx.open_url(&release_page(&current_version())));
    cx.on_action(|_: &CheckForUpdates, cx| {
        check_now(true, cx);
        // Deferred: this runs inside the window's update when the menu or
        // the palette sends it, and a nested dispatch would be dropped.
        cx.defer(|cx| cx.dispatch_action(&crate::settings::OpenUpdateSettings));
    });
}

#[cfg(test)]
mod tests {
    use super::{
        CheckError, FeedError, FetchError, Install, NotInstallable, Pending, Phase, PillKind,
        Platform, Problem, ReleaseNotes, StageError, Tone, Update, check_due, check_problem,
        flavour, just_updated, off_reason, percent, pill, release_date, stage_problem, status,
    };
    use super::{download_label, pill_label};
    use kanso_update::Asset;
    use proptest::prelude::*;

    #[test]
    fn whats_new_opens_the_versions_release() {
        let page = |v: &str| super::release_page(&semver::Version::parse(v).unwrap());
        assert_eq!(
            page("0.4.1"),
            format!(
                "https://github.com/{}/releases/tag/v0.4.1",
                kanso_update::REPOSITORY
            )
        );
        assert_eq!(
            page("0.4.1-dev.abc123"),
            format!("https://github.com/{}/releases", kanso_update::REPOSITORY)
        );
    }

    fn update(version: &str) -> Update {
        Update {
            version: version.into(),
            pub_date: "2026-10-20T10:00:00Z".into(),
            release_url: format!("https://github.com/iamngoni/kanso/releases/tag/v{version}"),
            notes: "### New\n\n- Updates".into(),
            asset: Asset {
                url: String::new(),
                sha256: "ab".repeat(32),
                size: 100,
            },
        }
    }

    fn pending(version: &str) -> Pending {
        Pending {
            version: version.into(),
            payload: "/x/.kanso-update/unpacked/Kanso.app".into(),
            target: "/x/Kanso.app".into(),
            pub_date: String::new(),
            release_url: String::new(),
            notes: String::new(),
            seal: Default::default(),
        }
    }

    fn installable() -> Result<Install, NotInstallable> {
        Ok(Install {
            target: "/Applications/Kanso.app".into(),
            stage_dir: "/Applications/.kanso-update".into(),
            platform: Platform::MacosArm64,
        })
    }

    fn phase() -> impl Strategy<Value = Phase> {
        let fixed = prop::sample::select(vec![
            Phase::Idle,
            Phase::Checking,
            Phase::UpToDate,
            Phase::Available(update("0.2.0")),
            Phase::Ready(pending("0.2.0")),
        ]);
        let downloading = (0u64..=200, 1u64..=200).prop_map(|(done, total)| Phase::Downloading {
            update: update("0.2.0"),
            done,
            total,
        });
        let failed = prop::sample::select(vec![
            Problem::Offline,
            Problem::RateLimited,
            Problem::NoRelease,
            Problem::Verification,
            Problem::NoSpace,
            Problem::NewerSchema,
            Problem::Other,
        ])
        .prop_map(Phase::Failed);
        prop_oneof![fixed, downloading, failed]
    }

    #[test]
    fn the_pill_reads_as_designed() {
        assert_eq!(
            pill(&Phase::Ready(pending("0.2.0")), false),
            Some((PillKind::Ready, "Update ready · Restart".into()))
        );
        assert_eq!(
            pill(&Phase::Available(update("0.2.0")), false),
            Some((PillKind::Available, "Update available · Download".into()))
        );
        let downloading = Phase::Downloading {
            update: update("0.2.0"),
            done: 42,
            total: 100,
        };
        assert_eq!(
            pill(&downloading, false),
            Some((PillKind::Busy, "Downloading · 42%".into()))
        );
        assert_eq!(
            pill(&Phase::Checking, false),
            None,
            "a background check is quiet"
        );
        assert!(pill(&Phase::Checking, true).is_some());
        assert_eq!(pill(&Phase::Failed(Problem::Offline), false), None);
        assert!(pill(&Phase::Failed(Problem::Offline), true).is_some());
        assert!(pill(&Phase::Failed(Problem::Verification), false).is_some());
    }

    #[test]
    fn each_failure_has_its_words() {
        let all = [
            Problem::Offline,
            Problem::RateLimited,
            Problem::NoRelease,
            Problem::Verification,
            Problem::NoSpace,
            Problem::NewerSchema,
            Problem::Other,
        ];
        for (i, a) in all.iter().enumerate() {
            assert!(a.words().ends_with('.'));
            for b in &all[i + 1..] {
                assert_ne!(a.words(), b.words());
            }
        }
        assert_eq!(
            check_problem(&CheckError::Fetch(FetchError::Offline)),
            Problem::Offline
        );
        assert_eq!(
            check_problem(&CheckError::Fetch(FetchError::RateLimited)),
            Problem::RateLimited
        );
        assert_eq!(
            check_problem(&CheckError::Fetch(FetchError::NotFound)),
            Problem::NoRelease
        );
        assert_eq!(
            check_problem(&CheckError::Feed(FeedError::BadSignature)),
            Problem::Verification
        );
        assert_eq!(
            check_problem(&CheckError::Feed(FeedError::NewerSchema)),
            Problem::NewerSchema
        );
        assert_eq!(stage_problem(&StageError::NoSpace), Problem::NoSpace);
        assert_eq!(
            stage_problem(&StageError::ChecksumMismatch),
            Problem::Verification
        );
        assert_eq!(
            stage_problem(&StageError::CodeSignature("bad".into())),
            Problem::Verification
        );
        assert_eq!(
            stage_problem(&StageError::Fetch(FetchError::Io(std::io::Error::from(
                std::io::ErrorKind::StorageFull
            )))),
            Problem::NoSpace
        );
    }

    #[test]
    fn dev_builds_say_updates_are_off() {
        let off = off_reason(&flavour::DEV, true, Some(Platform::MacosArm64));
        let line = status(
            &Phase::Idle,
            off.as_deref(),
            &Err(NotInstallable::DevFlavour),
            "0.1.0",
            true,
        );
        assert_eq!(line.label, "Development build — updates are off");
        assert_eq!(line.tone, Tone::Quiet);
        assert_eq!(
            off_reason(&flavour::RELEASE, true, Some(Platform::MacosArm64)),
            None
        );
        assert!(off_reason(&flavour::RELEASE, false, Some(Platform::MacosArm64)).is_some());
        assert!(off_reason(&flavour::RELEASE, true, None).is_some());
    }

    /// A copy apt installed is told a release is out, and its button and
    /// pill point at apt, not a download.
    #[test]
    fn an_apt_copy_points_at_apt_upgrade() {
        let apt = Err(NotInstallable::PackageManaged);
        let line = status(
            &Phase::Available(update("0.2.0")),
            None,
            &apt,
            "0.1.0",
            true,
        );
        assert_eq!(line.label, "Kanso 0.2.0 is available");
        assert!(line.detail.contains("sudo apt upgrade"), "{}", line.detail);
        assert_eq!(download_label(&apt), "Copy Command");
        assert_eq!(download_label(&installable()), "Download");
        assert_eq!(
            download_label(&Err(NotInstallable::NotWritable("/opt".into()))),
            "Download"
        );
        let (_, label) = pill(&Phase::Available(update("0.2.0")), false).unwrap();
        assert_eq!(
            pill_label(label.clone(), &apt),
            "Update available · apt upgrade"
        );
        assert_eq!(pill_label(label.clone(), &installable()), label);
        let ready = "Update ready · Restart".to_owned();
        assert_eq!(pill_label(ready.clone(), &apt), ready);
    }

    #[test]
    fn a_copy_that_cant_replace_itself_says_why() {
        let line = status(
            &Phase::Available(update("0.2.0")),
            None,
            &Err(NotInstallable::Translocated),
            "0.1.0",
            true,
        );
        assert_eq!(line.label, "Kanso 0.2.0 is available");
        assert_eq!(line.detail, NotInstallable::Translocated.reason());
    }

    #[test]
    fn updates_are_announced_once_after_they_happen() {
        let v = semver::Version::new(0, 2, 0);
        let notes = ReleaseNotes::from(&update("0.2.0"));
        assert_eq!(
            just_updated(Some("0.1.0"), &v, Some(notes.clone())),
            Some(notes)
        );
        // No earlier run: a new install, not an update.
        assert_eq!(just_updated(None, &v, None), None);
        assert_eq!(just_updated(Some("0.2.0"), &v, None), None);
        // Notes for another version give way to the release page.
        let other = ReleaseNotes::from(&update("0.3.0"));
        let shown = just_updated(Some("0.1.0"), &v, Some(other)).unwrap();
        assert_eq!(shown.version, "0.2.0");
        assert_eq!(
            shown.release_url,
            "https://github.com/iamngoni/kanso/releases/tag/v0.2.0"
        );
    }

    #[test]
    fn release_dates_read_as_the_list_writes_days() {
        assert_eq!(
            release_date("2026-10-20T10:00:00Z").as_deref(),
            Some("October 20, 2026")
        );
        assert_eq!(release_date(""), None);
        assert_eq!(release_date("2026-13-01"), None);
        assert_eq!(release_date("yesterday!"), None);
    }

    proptest! {
        /// Every state has a status line with words; the ready and
        /// available ones name the version; nothing but Failed is a
        /// warning.
        #[test]
        fn prop_status_lines_say_where_things_are(phase in phase(), auto in any::<bool>(), installable_ in any::<bool>()) {
            let install = if installable_ { installable() } else { Err(NotInstallable::DevBuild) };
            let line = status(&phase, None, &install, "0.1.0", auto);
            prop_assert!(!line.label.is_empty());
            prop_assert!(!line.detail.is_empty());
            prop_assert_eq!(line.tone == Tone::Warning, matches!(phase, Phase::Failed(_)));
            if matches!(phase, Phase::Ready(_) | Phase::Available(_) | Phase::Downloading { .. }) {
                prop_assert!(line.label.contains("0.2.0"));
            }
        }

        /// The pill shows exactly while there is something to do or see:
        /// always for an update in hand, a background check never, and a
        /// background failure only when it needs a person.
        #[test]
        fn prop_the_pill_shows_when_it_matters(phase in phase(), manual in any::<bool>()) {
            let shown = pill(&phase, manual);
            let expected = match &phase {
                Phase::Idle | Phase::UpToDate => false,
                Phase::Checking => manual,
                Phase::Downloading { .. } | Phase::Ready(_) | Phase::Available(_) => true,
                Phase::Failed(problem) => manual || problem.worth_showing(),
            };
            prop_assert_eq!(shown.is_some(), expected);
            if let Some((kind, label)) = shown {
                prop_assert_eq!(kind == PillKind::Ready, label.contains("Restart"));
                prop_assert!(label.chars().count() <= 28, "{} is too long for the sidebar", label);
            }
        }

    }

    #[test]
    fn checks_every_half_hour_and_sooner_after_a_failure() {
        const MINUTE: i64 = 60_000;
        let last = 1_000_000 * MINUTE;
        assert!(check_due(None, false, last));
        assert!(!check_due(Some(last), false, last + 29 * MINUTE));
        assert!(check_due(Some(last), false, last + 30 * MINUTE));
        assert!(!check_due(Some(last), true, last + 4 * MINUTE));
        assert!(check_due(Some(last), true, last + 5 * MINUTE));
        // Slept overnight: due on the first tick after waking.
        assert!(check_due(Some(last), false, last + 9 * 60 * MINUTE));
        // The clock went back (a time zone or manual change): check again.
        assert!(check_due(Some(last), false, last - MINUTE));
    }

    proptest! {
        /// A check is due exactly when the wait since the last one is up,
        /// and a failure never waits longer than a success.
        #[test]
        fn prop_check_due_follows_the_wait(last in 0i64..1 << 40, gap in -(1i64 << 30)..1 << 32, failed in any::<bool>()) {
            let now = last + gap;
            let every = if failed { super::RETRY_INTERVAL } else { super::CHECK_INTERVAL };
            prop_assert_eq!(check_due(Some(last), failed, now), gap < 0 || gap >= every.as_millis() as i64);
            if check_due(Some(last), false, now) {
                prop_assert!(check_due(Some(last), true, now));
            }
        }

        /// Progress never goes past 100% or backwards as bytes arrive.
        #[test]
        fn prop_percent_is_monotonic(total in 1u64..1_000_000, a in 0u64..2_000_000, b in 0u64..2_000_000) {
            let (lo, hi) = (a.min(b), a.max(b));
            prop_assert!(percent(lo, total) <= percent(hi, total));
            prop_assert!(percent(hi, total) <= 100);
            prop_assert_eq!(percent(total, total), 100);
        }

        /// Any real date reads back as its month, day and year.
        #[test]
        fn prop_release_dates_round_trip(y in 2000u32..2100, m in 1usize..=12, d in 1u32..=28) {
            let text = release_date(&format!("{y}-{m:02}-{d:02}T00:00:00Z")).unwrap();
            prop_assert!(text.ends_with(&format!(" {d}, {y}")), "{}", text);
        }

        /// Only a strictly newer version than the last run is announced.
        #[test]
        fn prop_only_newer_runs_are_announced(last in (0u64..3, 0u64..5, 0u64..5), now in (0u64..3, 0u64..5, 0u64..5)) {
            let last = semver::Version::new(last.0, last.1, last.2);
            let now = semver::Version::new(now.0, now.1, now.2);
            let shown = just_updated(Some(&last.to_string()), &now, None);
            prop_assert_eq!(shown.is_some(), now > last);
        }
    }
}
