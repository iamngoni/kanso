//! The anonymous usage count (docs/METRICS.md) on the desktop: the notice
//! shown once per version, the features counted where they happen, and the
//! batch sent at launch, every hour after and when Kanso quits. `kanso_metrics` holds the rules; this
//! wires them to the settings, the account and the clock.
//!
//! Only a release build of the release flavour sends, and only once OK was
//! pressed on the notice. Until then nothing is counted either.

use std::sync::OnceLock;
use std::time::Duration;

use gpui_kit::component::{WindowExt as _, h_flex, v_flex};
use gpui_kit::*;
use kanso_core::DocType;
use kanso_engine::editor_settings::ViewMode;
use kanso_import::library::SourceKind;
use kanso_metrics::values::{
    ActiveExtras, AttachmentsMb, Channel, EditorMode, LocaleLang, NotebooksRange, PluginsRange,
    Theme, ThemeMode,
};
use kanso_metrics::{Active, Day, Device, Form, Id, Outcome, Platform, SyncMode};

pub use kanso_metrics::{Feature, ImportSource};

use crate::clock::Clock;
use crate::config::SyncSettings;
use crate::flavour;
use crate::services::Services;
use crate::theme::Colors;
use crate::workspace::AppState;
use crate::zoom::px;

pub const NOTICE_TITLE: &str = "Kanso counts, it doesn't watch";
pub const NOTICE_BODY: &str = "To know how many people use Kanso and on which devices, Kanso sends a small anonymous count every hour while it is open, when you open it and when you close it: the kind of device, Kanso's version, your language, theme and editor mode as a plain label, how many notes, notebooks and plugins you have and how much room attachments take as ranges, how long it was open, and how often features were used, as ranges. Never your notes, titles, tags, files, name or anything you type, and never the name of a theme or plugin. Devices signed in to the same account count as one person, with an ID that can't be traced back to the account.";
pub const NOTE_LINK: (&str, &str) = (
    "What's sent, and what isn't",
    "https://kansoapp.app/metrics",
);
pub const CODE_LINK: (&str, &str) = (
    "The code",
    "https://github.com/iamngoni/kanso/blob/master/docs/METRICS.md",
);

/// The notice's version: raised whenever what is sent changes, so everyone
/// sees the notice again once. Version 1 was the once-a-day count.
pub const NOTICE_VERSION: u8 = 2;

/// The first send waits for the app to settle.
const FIRST_SEND: Duration = Duration::from_secs(20);
/// Then once an hour: the day's `app_active` after midnight UTC, features
/// used since, and a later try for what didn't go.
const EVERY: Duration = Duration::from_secs(60 * 60);

/// Whether this build sends at all: a release build of Kanso (not Kanso
/// Dev), never a test.
pub fn enabled() -> bool {
    !cfg!(debug_assertions) && !cfg!(test) && !flavour::CURRENT.is_dev()
}

/// Whether the notice of the current version was acknowledged.
fn acknowledged(nudges: &crate::config::Nudges) -> bool {
    nudges.metrics_notice_seen && nudges.metrics_notice_version >= NOTICE_VERSION
}

/// Whether counting is on now: the build sends and the notice was seen.
fn counting(cx: &App) -> bool {
    enabled()
        && cx
            .try_global::<AppState>()
            .is_some_and(|state| acknowledged(&state.config.nudges))
}

/// The sender's state.
struct Metrics {
    /// A batch is on its way; the next tick waits for its answer.
    sending: bool,
    /// Notes edited in this batch window, so a note counts once however
    /// much it is typed in. Emptied each tick. IDs only, never kept.
    edited: std::collections::HashSet<String>,
    /// Uses counted in memory, written to the ledger on a debounce, before
    /// a send and on quit (so a batch of notes is one write, not hundreds).
    buffer: kanso_metrics::CountBuffer,
    /// A flush is scheduled.
    flush_scheduled: bool,
    /// When this launch began, for the session's length.
    started: std::time::Instant,
    /// This launch's `app_opened` was noted.
    launched: bool,
    _schedule: Option<Task<()>>,
}

impl Global for Metrics {}

/// Notes that `feature` was used; it goes with the next send as a count.
pub fn record(cx: &mut App, feature: Feature) {
    record_n(cx, feature, 1);
}

/// Notes `n` uses of `feature` at once (a change to many notes). Counted in
/// memory and written to the settings on a debounce, so many notes are one
/// write at most every [`kanso_metrics::FLUSH_EVERY_MS`].
// no-prop-test: glue; `CountBuffer` and `Ledger::record_n` hold the rules and are tested.
pub fn record_n(cx: &mut App, feature: Feature, n: u32) {
    if !counting(cx) || !cx.has_global::<Metrics>() {
        return;
    }
    let metrics = cx.global_mut::<Metrics>();
    metrics.buffer.add(feature, n);
    if metrics.flush_scheduled {
        return;
    }
    metrics.flush_scheduled = true;
    cx.spawn(async move |cx| {
        cx.background_executor()
            .timer(Duration::from_millis(
                u64::try_from(kanso_metrics::FLUSH_EVERY_MS).unwrap_or(30_000),
            ))
            .await;
        cx.update(|cx| {
            if cx.has_global::<Metrics>() {
                cx.global_mut::<Metrics>().flush_scheduled = false;
                flush(cx);
            }
        });
    })
    .detach();
}

/// Writes the counted uses to the ledger in the settings, once.
fn flush(cx: &mut App) {
    if !cx.has_global::<Metrics>() || cx.global::<Metrics>().buffer.is_empty() {
        return;
    }
    let now = Clock::now_ms(cx);
    let today = Day::from_unix_ms(now);
    let mut buffer = std::mem::take(&mut cx.global_mut::<Metrics>().buffer);
    let mut ledger = cx.global::<AppState>().config.metrics.clone();
    if buffer.flush(&mut ledger, today, now) {
        AppState::update_config(cx, |config| config.metrics = ledger);
    }
    cx.global_mut::<Metrics>().buffer = buffer;
}

/// Notes that the note `note_id` was edited: once per note per batch window,
/// however many keys. The ID is held in memory until the next send and is
/// never sent.
// no-prop-test: glue; `Ledger::record` holds the rule and is tested.
pub fn record_note_edited(cx: &mut App, note_id: &str) {
    if !counting(cx) || !cx.has_global::<Metrics>() {
        return;
    }
    if cx.global_mut::<Metrics>().edited.insert(note_id.to_owned()) {
        record(cx, Feature::NoteEdited);
    }
}

/// Starts sending: in a few seconds, then every hour. Nothing at all in a
/// build that never sends.
// no-prop-test: scheduling; what is sent is `Ledger::due`'s.
pub fn init(cx: &mut App) {
    if !enabled() {
        return;
    }
    let task = cx.spawn(async move |cx| {
        cx.background_executor().timer(FIRST_SEND).await;
        loop {
            cx.update(tick);
            cx.background_executor().timer(EVERY).await;
        }
    });
    cx.set_global(Metrics {
        sending: false,
        edited: std::collections::HashSet::new(),
        buffer: kanso_metrics::CountBuffer::default(),
        flush_scheduled: false,
        started: std::time::Instant::now(),
        launched: false,
        _schedule: Some(task),
    });
    // This launch counts now, not at the first send, so a short one does too.
    note_launch(cx);
}

/// Notes this launch (once), so the next send carries its `app_opened`.
fn note_launch(cx: &mut App) {
    if !counting(cx) || !cx.has_global::<Metrics>() || cx.global::<Metrics>().launched {
        return;
    }
    cx.global_mut::<Metrics>().launched = true;
    let mut ledger = cx.global::<AppState>().config.metrics.clone();
    if ledger.launch() {
        AppState::update_config(cx, |config| config.metrics = ledger);
    }
}

/// A sync failed: the next `app_active` says how many, as a range.
// no-prop-test: glue; `Ledger::record_sync_error` holds the rule and is tested.
pub fn record_sync_error(cx: &mut App) {
    if !counting(cx) {
        return;
    }
    let mut ledger = cx.global::<AppState>().config.metrics.clone();
    if ledger.record_sync_error() {
        AppState::update_config(cx, |config| config.metrics = ledger);
    }
}

/// The optional props of `app_active`, read from the settings and the
/// library as labels and ranges only. The language is English, the only one
/// the desktop app ships.
fn extras(cx: &App) -> ActiveExtras {
    let state = cx.global::<AppState>();
    let counts = state.store.counts().unwrap_or_default();
    let count =
        |kind: DocType| u64::try_from(counts.get(&kind).copied().unwrap_or(0)).unwrap_or(u64::MAX);
    let (theme_mode, theme) = match cx.try_global::<crate::themes::ActiveTheme>() {
        Some(active) => {
            let mode = if state.config.appearance.themes.follow_system {
                ThemeMode::System
            } else if active.ui.dark() {
                ThemeMode::Dark
            } else {
                ThemeMode::Light
            };
            let builtin = matches!(active.ui.origin, crate::origin::Origin::Builtin);
            (Some(mode), Some(Theme::new(&active.ui.id, builtin)))
        }
        None => (None, None),
    };
    let editor_mode = match state.config.appearance.view_mode {
        ViewMode::Edit => EditorMode::Edit,
        ViewMode::Live => EditorMode::Live,
        ViewMode::Preview => EditorMode::Preview,
        ViewMode::Split => EditorMode::Split,
    };
    ActiveExtras {
        locale_lang: Some(LocaleLang::new("en", SHIPPED_LANGUAGES)),
        theme_mode,
        theme,
        editor_mode: Some(editor_mode),
        plugins: Some(PluginsRange::from_count(
            u64::try_from(crate::plugins::installed(cx).len()).unwrap_or(u64::MAX),
        )),
        notebooks: Some(NotebooksRange::from_count(count(DocType::Book))),
        attachments_mb: state
            .store
            .attachments_bytes()
            .ok()
            .map(AttachmentsMb::from_bytes),
        channel: Some(channel()),
    }
}

/// The languages the desktop app's interface ships in (ISO 639-1).
const SHIPPED_LANGUAGES: &[&str] = &["en"];

/// `channel`: Kanso Dev is `dev`, a pre-release version `beta`, else `stable`.
fn channel() -> Channel {
    if flavour::CURRENT.is_dev() {
        Channel::Dev
    } else if env!("CARGO_PKG_VERSION").contains('-') {
        Channel::Beta
    } else {
        Channel::Stable
    }
}

/// What is due to send now, if anything.
fn due_batch(cx: &mut App) -> Option<kanso_metrics::Batch> {
    let platform = Platform::current()?;
    let today = Day::from_unix_ms(Clock::now_ms(cx));
    let device = Device::new(platform, Form::Desktop, env!("CARGO_PKG_VERSION"));
    let state = cx.global::<AppState>();
    let notes = state
        .store
        .counts()
        .ok()
        .and_then(|counts| counts.get(&DocType::Note).copied())
        .unwrap_or(0);
    let active = Active::new(
        os_version(),
        sync_mode(&state.config.sync),
        state.config.ai.enabled,
        u64::try_from(notes).unwrap_or(u64::MAX),
    )
    .with_extras(extras(cx));
    let id = id(cx);
    cx.global::<AppState>()
        .config
        .metrics
        .due(today, &id, &device, &active)
}

/// Sends what is due, if anything, off the UI thread, and keeps how it
/// went in the settings.
// no-prop-test: wiring to the network; the ledger's properties cover it.
fn tick(cx: &mut App) {
    if !counting(cx) || !cx.has_global::<Metrics>() || cx.global::<Metrics>().sending {
        return;
    }
    note_launch(cx);
    flush(cx);
    // A note edited again after this send counts again in the next window.
    cx.global_mut::<Metrics>().edited.clear();
    let Some(batch) = due_batch(cx) else {
        return;
    };
    let (tx, rx) = async_channel::bounded(1);
    let sent = batch.clone();
    let spawned = std::thread::Builder::new()
        .name("kanso-metrics".into())
        .spawn(move || {
            tx.try_send(kanso_metrics::send(&sent, enabled())).ok();
        });
    if spawned.is_err() {
        return;
    }
    cx.global_mut::<Metrics>().sending = true;
    cx.spawn(async move |cx| {
        let outcome = rx.recv().await.unwrap_or(Outcome::Retry);
        cx.update(|cx| {
            cx.global_mut::<Metrics>().sending = false;
            settle(cx, &batch, outcome);
        });
    })
    .detach();
}

/// Keeps how sending `batch` went in the settings.
fn settle(cx: &mut App, batch: &kanso_metrics::Batch, outcome: Outcome) {
    let mut ledger = cx.global::<AppState>().config.metrics.clone();
    if ledger.settle(batch, outcome) {
        AppState::update_config(cx, |config| config.metrics = ledger);
    }
}

/// On quit: the session's length is noted and what is due goes now, waiting
/// a couple of seconds at most for the answer. Without one, the batch stays
/// in the ledger for the next launch.
// no-prop-test: wiring to the network and the clock; the ledger's properties cover it.
pub fn on_quit(cx: &mut App) {
    if !counting(cx) || !cx.has_global::<Metrics>() {
        return;
    }
    flush(cx);
    let secs = cx.global::<Metrics>().started.elapsed().as_secs();
    let today = Day::from_unix_ms(Clock::now_ms(cx));
    let mut ledger = cx.global::<AppState>().config.metrics.clone();
    if ledger.end_session(today, secs) {
        AppState::update_config(cx, |config| config.metrics = ledger);
    }
    // A batch already on its way is not sent twice: what it carries and the
    // session wait in the ledger for the next launch (a quit is no time to
    // wait on a slow answer).
    if cx.global::<Metrics>().sending {
        return;
    }
    let Some(batch) = due_batch(cx) else {
        return;
    };
    let (tx, rx) = std::sync::mpsc::channel();
    let sent = batch.clone();
    if std::thread::Builder::new()
        .name("kanso-metrics-quit".into())
        .spawn(move || {
            tx.send(kanso_metrics::send(&sent, enabled())).ok();
        })
        .is_err()
    {
        return;
    }
    if let Ok(outcome) = rx.recv_timeout(QUIT_WAIT) {
        settle(cx, &batch, outcome);
    }
}

/// How long quitting waits for the last batch to be answered.
const QUIT_WAIT: Duration = Duration::from_secs(2);

/// Who this device counts as: the account's `a_` ID while signed in with
/// its data key here, else this install's `i_` ID (made and saved the first
/// time).
fn id(cx: &mut App) -> Id {
    let account = cx
        .try_global::<Services>()
        .and_then(Services::session)
        .and_then(|session| Id::parse(&session.metrics_id()));
    if let Some(id) = account {
        return id;
    }
    if let Some(id) = cx.global::<AppState>().config.metrics.existing_install_id() {
        return id.clone();
    }
    let mut ledger = cx.global::<AppState>().config.metrics.clone();
    let id = ledger.install_id();
    AppState::update_config(cx, |config| config.metrics = ledger);
    id
}

/// `sync`: off unless signed in with sync on; `kanso` on Kanso's own server
/// under any of its names, else `self_hosted`.
pub fn sync_mode(sync: &SyncSettings) -> SyncMode {
    match &sync.account {
        Some(account) if sync.enabled => {
            if kanso_sync::is_owner_server(&account.origin) {
                SyncMode::Kanso
            } else {
                SyncMode::SelfHosted
            }
        }
        _ => SyncMode::Off,
    }
}

/// `source` of an import from a library kind. The kinds the page doesn't
/// list are `other`.
pub fn import_source(kind: SourceKind) -> ImportSource {
    match kind {
        SourceKind::Markdown => ImportSource::Markdown,
        SourceKind::Obsidian => ImportSource::Obsidian,
        SourceKind::Logseq => ImportSource::Logseq,
        SourceKind::Notion => ImportSource::Notion,
        SourceKind::Bear => ImportSource::Bear,
        SourceKind::Evernote => ImportSource::Evernote,
        SourceKind::Joplin => ImportSource::Joplin,
        SourceKind::StandardNotes => ImportSource::StandardNotes,
        SourceKind::Simplenote => ImportSource::Simplenote,
        SourceKind::UpNote => ImportSource::Upnote,
        SourceKind::AppleNotes => ImportSource::AppleNotes,
        SourceKind::AppleJournal
        | SourceKind::GoogleKeep
        | SourceKind::Html
        | SourceKind::Roam
        | SourceKind::TextBundle
        | SourceKind::Tomboy => ImportSource::Other,
    }
}

/// The system's version, as far as Kanso can read it without asking: the
/// macOS product version, the Linux kernel's release, and `unknown` on
/// Windows. Checked again by `kanso_metrics`.
// On Windows `read` is always `None`.
#[cfg_attr(
    not(any(target_os = "macos", target_os = "linux")),
    allow(clippy::unnecessary_literal_unwrap)
)]
fn os_version() -> &'static str {
    static VERSION: OnceLock<String> = OnceLock::new();
    VERSION.get_or_init(|| {
        #[cfg(target_os = "macos")]
        let read = std::fs::read_to_string("/System/Library/CoreServices/SystemVersion.plist")
            .ok()
            .and_then(|plist| product_version(&plist));
        #[cfg(target_os = "linux")]
        let read = std::fs::read_to_string("/proc/sys/kernel/osrelease")
            .ok()
            .map(|release| release.trim().to_owned());
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        let read: Option<String> = None;
        read.unwrap_or_else(|| kanso_metrics::values::UNKNOWN.to_owned())
    })
}

/// `ProductVersion` from macOS's SystemVersion.plist. Pure.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn product_version(plist: &str) -> Option<String> {
    let after = &plist[plist.find("<key>ProductVersion</key>")? + 25..];
    let start = after.find("<string>")? + 8;
    let end = after[start..].find("</string>")? + start;
    Some(after[start..end].trim().to_owned())
}

/// The notice, once: shown at launch until OK is pressed (a new install and
/// an update alike).
// no-prop-test: a dialog; a snapshot shows it.
pub fn show_notice(window: &mut Window, cx: &mut App) {
    if acknowledged(&cx.global::<AppState>().config.nudges) {
        return;
    }
    open_notice(window, cx);
}

/// The notice itself. Only OK closes it, and only OK starts the count.
pub fn open_notice(window: &mut Window, cx: &mut App) {
    window.open_alert_dialog(cx, move |alert, _, cx| {
        let colors = Colors::get(cx);
        let link = |(label, url): (&'static str, &'static str), id: &'static str| {
            div()
                .id(id)
                .debug_selector(move || id.into())
                .text_color(colors.link)
                .cursor_pointer()
                .hover(|el| el.underline())
                .child(label)
                .on_click(move |_, _, cx| cx.open_url(url))
        };
        alert
            .title(NOTICE_TITLE)
            .width(px(crate::dialogs::FORM_WIDTH))
            .description(
                v_flex().gap(px(10.)).child(NOTICE_BODY).child(
                    h_flex()
                        .gap(px(6.))
                        .flex_wrap()
                        .child(link(NOTE_LINK, "metrics-note"))
                        .child(div().text_color(colors.text_muted).child("\u{b7}"))
                        .child(link(CODE_LINK, "metrics-code")),
                ),
            )
            .keyboard(false)
            .ok_text("OK")
            .on_ok(|_, _, cx| {
                AppState::update_config(cx, |config| {
                    config.nudges.metrics_notice_seen = true;
                    config.nudges.metrics_notice_version = NOTICE_VERSION;
                });
                // The day's count goes now rather than at the next tick.
                tick(cx);
                true
            })
    });
}

#[cfg(test)]
mod tests {
    use super::{
        CODE_LINK, ImportSource, NOTE_LINK, NOTICE_BODY, NOTICE_TITLE, SourceKind, SyncMode,
        SyncSettings, enabled, import_source, os_version, product_version, sync_mode,
    };
    use crate::config::AccountRef;
    use proptest::prelude::*;

    #[test]
    fn this_build_never_sends() {
        assert!(!enabled());
    }

    #[test]
    fn sync_is_named_by_its_server() {
        let signed_in = |origin: &str, enabled| SyncSettings {
            enabled,
            account: Some(AccountRef {
                origin: origin.into(),
                id: "a_1".into(),
                username: "ngoni".into(),
            }),
            ..SyncSettings::default()
        };
        assert_eq!(sync_mode(&SyncSettings::default()), SyncMode::Off);
        assert_eq!(
            sync_mode(&signed_in(kanso_sync::OWNER_ORIGIN, true)),
            SyncMode::Kanso
        );
        assert_eq!(
            sync_mode(&signed_in("https://sync.kansoapp.app", true)),
            SyncMode::Kanso
        );
        assert_eq!(
            sync_mode(&signed_in("https://notes.example.org", true)),
            SyncMode::SelfHosted
        );
        assert_eq!(
            sync_mode(&signed_in(kanso_sync::OWNER_ORIGIN, false)),
            SyncMode::Off
        );
    }

    #[test]
    fn reads_the_macos_version() {
        let plist = "<plist><dict><key>ProductBuildVersion</key><string>24B83</string><key>ProductVersion</key>\n\t<string>15.1</string></dict></plist>";
        assert_eq!(product_version(plist).as_deref(), Some("15.1"));
        assert_eq!(product_version("<plist/>"), None);
        assert_eq!(
            kanso_metrics::values::os_version(os_version()),
            os_version()
        );
    }

    #[test]
    fn the_notice_says_what_the_page_says() {
        let spec = include_str!("../../../docs/METRICS.md");
        let notice = &spec[spec.find("## The notice").unwrap()..];
        assert!(notice.contains(&format!("**{NOTICE_TITLE}**")));
        let quoted: String = notice
            .lines()
            .skip_while(|line| !line.starts_with("> To know"))
            .take_while(|line| *line != ">")
            .map(|line| line.trim_start_matches('>').trim())
            .collect::<Vec<_>>()
            .join(" ");
        assert_eq!(quoted, NOTICE_BODY);
        for (label, url) in [NOTE_LINK, CODE_LINK] {
            assert!(notice.contains(&format!("[{label}]({url})")), "{label}");
        }
    }

    #[test]
    fn an_older_notice_shows_again() {
        use super::{NOTICE_VERSION, acknowledged};
        use crate::config::Nudges;
        let seen = |seen, version| Nudges {
            metrics_notice_seen: seen,
            metrics_notice_version: version,
            ..Nudges::default()
        };
        assert!(!acknowledged(&seen(true, 0)), "the daily-count notice");
        assert!(!acknowledged(&seen(false, NOTICE_VERSION)));
        assert!(acknowledged(&seen(true, NOTICE_VERSION)));
        assert!(acknowledged(&seen(true, NOTICE_VERSION + 1)));
    }

    proptest! {
        /// Every library kind has a source from the closed list, and only
        /// the kinds the page names are named.
        #[test]
        fn prop_every_kind_has_a_source(ix in 0usize..SourceKind::ALL.len()) {
            let kind = SourceKind::ALL[ix];
            let source = import_source(kind);
            prop_assert!(ImportSource::ALL.contains(&source));
            prop_assert_ne!(source, ImportSource::Pouch);
        }
    }
}
