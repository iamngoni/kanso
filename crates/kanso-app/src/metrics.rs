//! The anonymous daily count (docs/METRICS.md) on the desktop: the notice
//! shown once, the features counted where they happen, and the batch sent
//! at launch and every hour after. `kanso_metrics` holds the rules; this
//! wires them to the settings, the account and the clock.
//!
//! Only a release build of the release flavour sends, and only once OK was
//! pressed on the notice. Until then nothing is counted either.

use std::sync::OnceLock;
use std::time::Duration;

use gpui_kit::component::{WindowExt as _, h_flex, v_flex};
use gpui_kit::*;
use kanso_core::DocType;
use kanso_import::library::SourceKind;
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
pub const NOTICE_BODY: &str = "To know how many people use Kanso and on which devices, Kanso sends a small anonymous count once a day: the kind of device, Kanso's version and which features were used. Never your notes, titles, tags, files, name or anything you type. Devices signed in to the same account count as one person, with an ID that can't be traced back to the account.";
pub const NOTE_LINK: (&str, &str) = (
    "What's sent, and what isn't",
    "https://kansoapp.app/metrics",
);
pub const CODE_LINK: (&str, &str) = (
    "The code",
    "https://github.com/iamngoni/kanso/blob/master/docs/METRICS.md",
);

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

/// Whether counting is on now: the build sends and the notice was seen.
fn counting(cx: &App) -> bool {
    enabled()
        && cx
            .try_global::<AppState>()
            .is_some_and(|state| state.config.nudges.metrics_notice_seen)
}

/// The sender's state.
#[derive(Default)]
struct Metrics {
    /// A batch is on its way; the next tick waits for its answer.
    sending: bool,
    _schedule: Option<Task<()>>,
}

impl Global for Metrics {}

/// Notes that `feature` was used, once a day; it goes with the next send.
// no-prop-test: glue; `Ledger::record` holds the rule and is tested.
pub fn record(cx: &mut App, feature: Feature) {
    if !counting(cx) {
        return;
    }
    let today = Day::from_unix_ms(Clock::now_ms(cx));
    let mut ledger = cx.global::<AppState>().config.metrics.clone();
    if ledger.record(today, feature) {
        AppState::update_config(cx, |config| config.metrics = ledger);
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
        _schedule: Some(task),
    });
}

/// Sends what is due, if anything, off the UI thread, and keeps how it
/// went in the settings.
// no-prop-test: wiring to the network; the ledger's properties cover it.
fn tick(cx: &mut App) {
    if !counting(cx) || !cx.has_global::<Metrics>() || cx.global::<Metrics>().sending {
        return;
    }
    let Some(platform) = Platform::current() else {
        return;
    };
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
    );
    let id = id(cx);
    let Some(batch) = cx
        .global::<AppState>()
        .config
        .metrics
        .due(today, &id, &device, &active)
    else {
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
            let mut ledger = cx.global::<AppState>().config.metrics.clone();
            if ledger.settle(&batch, outcome) {
                AppState::update_config(cx, |config| config.metrics = ledger);
            }
        });
    })
    .detach();
}

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
    if cx.global::<AppState>().config.nudges.metrics_notice_seen {
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
                AppState::update_config(cx, |config| config.nudges.metrics_notice_seen = true);
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
