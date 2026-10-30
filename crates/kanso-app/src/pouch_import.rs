//! Moving over from a PouchDB notes library found on this computer:
//! importing its notes, and taking its local API port and credentials so
//! existing MCP clients keep working unchanged.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context as _, bail};
use gpui_kit::component::notification::Notification;
use gpui_kit::component::{ActiveTheme as _, WindowExt as _, v_flex};
use gpui_kit::*;
use kanso_import::safe_storage::{SAFE_STORAGE_SERVICE, keychain_secret};
use kanso_import::{
    ImportReport, LocalServerCredentials, Phase, Progress, find_pouch_db, read_local_server_config,
};
use kanso_store::Store;

use crate::config::ServerSettings;
use crate::paths;
use crate::workspace::AppState;

/// What [`copy_server_settings`] took from the library's settings.
#[derive(Debug, PartialEq, Eq)]
pub enum Copied {
    /// Port, address and credentials.
    Everything,
    /// Port and address; there were no credentials to copy.
    PortOnly,
}

/// Copies the library's local server port, address and credentials into
/// `settings`. `secret` supplies the Keychain secret when the credentials are
/// encrypted; it is only called then. On error `settings` is unchanged.
pub fn copy_server_settings(
    library_dir: &Path,
    settings: &mut ServerSettings,
    secret: impl FnOnce() -> anyhow::Result<String>,
) -> anyhow::Result<Copied> {
    let Some(mut server) = read_local_server_config(library_dir)? else {
        bail!(
            "there are no local server settings in {}",
            library_dir.display()
        );
    };
    if matches!(server.credentials, LocalServerCredentials::Encrypted { .. }) {
        server.decrypt_credentials(&secret()?)?;
    }
    if matches!(server.credentials, LocalServerCredentials::Unrecognized) {
        bail!("the credentials are in a format Kanso doesn't know");
    }
    settings.port = server.port;
    settings.bind_address = server.bind_address.clone();
    match (server.username(), server.password()) {
        (Some(username), Some(password)) => {
            settings.username = username.to_owned();
            settings.password = password.to_owned();
            Ok(Copied::Everything)
        }
        _ => Ok(Copied::PortOnly),
    }
}

/// The PouchDB library's data folder and database, when there is one to
/// import.
pub fn find_library() -> Option<(PathBuf, PathBuf)> {
    let dir = kanso_import::default_pouch_library_dir()?;
    let db = find_pouch_db(&dir).ok()?;
    Some((dir, db))
}

fn phase_label(phase: Phase) -> &'static str {
    match phase {
        Phase::Snapshot => "Copying the library",
        Phase::Books => "Notebooks",
        Phase::Tags => "Tags",
        Phase::Files => "Attachments",
        Phase::Notes => "Notes",
        Phase::Activities => "Activity log",
        Phase::Verify => "Checking",
    }
}

enum ImportState {
    Running(Progress),
    Done(Box<ImportReport>),
    Failed(String),
}

/// The import dialog's body: progress, then the result.
pub struct ImportView {
    state: ImportState,
    _task: Task<()>,
}

enum Message {
    Progress(Progress),
    Done(Result<Box<ImportReport>, String>),
}

impl ImportView {
    fn new(store: Arc<Store>, source: PathBuf, cx: &mut Context<Self>) -> Self {
        let (tx, rx) = async_channel::unbounded();
        std::thread::spawn(move || {
            let progress_tx = tx.clone();
            let result = kanso_import::import(&source, &store, move |progress| {
                progress_tx.try_send(Message::Progress(progress)).ok();
            });
            tx.try_send(Message::Done(
                result.map(Box::new).map_err(|err| format!("{err:#}")),
            ))
            .ok();
        });
        let task = cx.spawn(async move |this, cx| {
            while let Ok(message) = rx.recv().await {
                let alive = this.update(cx, |this, cx| {
                    this.state = match message {
                        Message::Progress(progress) => ImportState::Running(progress),
                        Message::Done(Ok(report)) => {
                            crate::metrics::record(
                                cx,
                                crate::metrics::Feature::Import(
                                    crate::metrics::ImportSource::Pouch,
                                ),
                            );
                            ImportState::Done(report)
                        }
                        Message::Done(Err(err)) => ImportState::Failed(err),
                    };
                    cx.notify();
                });
                if alive.is_err() {
                    break;
                }
            }
        });
        ImportView {
            state: ImportState::Running(Progress {
                phase: Phase::Snapshot,
                done: 0,
                total: 1,
            }),
            _task: task,
        }
    }
}

impl Render for ImportView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let muted = cx.theme().muted_foreground;
        match &self.state {
            ImportState::Running(progress) => {
                let fraction = if progress.total == 0 {
                    1.0
                } else {
                    progress.done as f32 / progress.total as f32
                };
                v_flex()
                    .gap_2()
                    .child(format!(
                        "{}… {} of {}",
                        phase_label(progress.phase),
                        progress.done,
                        progress.total
                    ))
                    .child(
                        div()
                            .h_1p5()
                            .w_full()
                            .rounded_full()
                            .bg(cx.theme().muted)
                            .child(
                                div()
                                    .h_full()
                                    .rounded_full()
                                    .bg(cx.theme().primary)
                                    .w(relative(fraction.clamp(0.0, 1.0))),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(muted)
                            .child("The library's files are only read; its app can keep running."),
                    )
            }
            ImportState::Done(report) => v_flex()
                .gap_1()
                .child(format!(
                    "Imported {} notes, {} notebooks, {} tags and {} attachments.",
                    report.notes, report.books, report.tags, report.files
                ))
                .when(report.unchanged > 0, |el| {
                    el.child(
                        div()
                            .text_sm()
                            .text_color(muted)
                            .child(format!("{} were already up to date.", report.unchanged)),
                    )
                })
                .when(!report.is_clean(), |el| {
                    el.child(div().text_sm().text_color(cx.theme().danger).child(format!(
                        "{} problems: {} rejected, {} attachments didn't match their checksum.",
                        report.rejected.len()
                            + report.attachment_mismatches.len()
                            + report.unverified.len(),
                        report.rejected.len(),
                        report.attachment_mismatches.len(),
                    )))
                }),
            ImportState::Failed(err) => v_flex().gap_1().child("The import failed.").child(
                div()
                    .text_sm()
                    .text_color(cx.theme().danger)
                    .child(err.clone()),
            ),
        }
    }
}

use gpui_kit::prelude::FluentBuilder as _;

/// Imports the library's notes, showing progress in a dialog.
pub fn start_import(window: &mut Window, cx: &mut App) {
    let Some((_, source)) = find_library() else {
        let message = match kanso_import::default_pouch_library_dir() {
            Some(dir) => format!("No notes library was found in {}.", dir.display()),
            None => "No notes library was found.".to_owned(),
        };
        window.push_notification(Notification::warning(message), cx);
        return;
    };
    let store = cx.global::<AppState>().store.clone();
    let view = cx.new(|cx| ImportView::new(store, source, cx));
    window.open_alert_dialog(cx, move |alert, _, _| {
        alert
            .title("Import a notes library")
            .child(view.clone())
            .ok_text("Close")
    });
}

/// Takes the library's API port and credentials, after the user agrees to the
/// Keychain prompt. `on_done` runs after the settings are saved.
pub fn start_copy_credentials(
    window: &mut Window,
    cx: &mut App,
    on_done: impl Fn(&mut Window, &mut App) + 'static,
) {
    let Some(library_dir) = kanso_import::default_pouch_library_dir() else {
        return;
    };
    let mut server = cx.global::<AppState>().config.server.clone();
    // Reading the Keychain waits for the user to answer macOS's prompt.
    let copy = cx.background_spawn(async move {
        copy_server_settings(&library_dir, &mut server, || {
            keychain_secret(SAFE_STORAGE_SERVICE)
        })
        .map(|copied| (copied, server))
    });
    window
        .spawn(cx, async move |cx| {
            let result = copy.await;
            cx.update(|window, cx| match result {
                Ok((copied, server)) => {
                    AppState::update_config(cx, |config| config.server = server);
                    let message = match copied {
                        Copied::Everything => {
                            "Kanso now uses the copied API port, username and password."
                        }
                        Copied::PortOnly => "Kanso now uses the copied API port.",
                    };
                    window.push_notification(Notification::success(message), cx);
                    on_done(window, cx);
                }
                Err(err) => {
                    window.push_notification(
                        Notification::error(format!("Couldn't copy the API settings: {err:#}")),
                        cx,
                    );
                }
            })
            .ok();
        })
        .detach();
}

/// `kanso import pouchdb [--library-dir DIR] [--credentials]`
pub fn run_cli(data_dir: &Path, args: &[String]) -> anyhow::Result<()> {
    let mut library_dir = None;
    let mut credentials = false;
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--library-dir" => {
                library_dir = Some(PathBuf::from(
                    args.next().context("--library-dir needs a folder")?,
                ))
            }
            "--credentials" => credentials = true,
            other => bail!("unexpected argument `{other}`"),
        }
    }
    let library_dir = match library_dir {
        Some(dir) => dir,
        None => kanso_import::default_pouch_library_dir()
            .context("HOME is not set; pass --library-dir")?,
    };
    let source = find_pouch_db(&library_dir)?;
    std::fs::create_dir_all(data_dir)?;
    let Some(_lock) = kanso_store::lock_data_dir(data_dir)? else {
        bail!(
            "Kanso is running with {}; quit it first",
            data_dir.display()
        );
    };
    let store = Store::open(paths::db_path(data_dir))?;
    eprintln!("Importing {}", source.display());
    let report = kanso_import::import(&source, &store, |_| {})?;
    println!("{report}");
    if credentials {
        let config_path = paths::config_path(data_dir);
        let mut config = crate::config::AppConfig::load(&config_path)?;
        let copied = copy_server_settings(&library_dir, &mut config.server, || {
            eprintln!("macOS will ask to allow access to “{SAFE_STORAGE_SERVICE}”.");
            keychain_secret(SAFE_STORAGE_SERVICE)
        })?;
        config.save(&config_path)?;
        println!(
            "Copied the library's API {} (port {}).",
            if copied == Copied::Everything {
                "port and credentials"
            } else {
                "port"
            },
            config.server.port
        );
    }
    if !report.is_clean() {
        bail!("the import reported problems; see above");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Copied, copy_server_settings};
    use crate::config::ServerSettings;
    use base64::Engine as _;
    use proptest::prelude::*;

    fn library_dir(auth: serde_json::Value, port: u16) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let config = serde_json::json!({"*": {"core": {"server": {
            "enabled": true, "port": port, "bindAddress": "127.0.0.1", "auth": auth
        }}}});
        std::fs::write(dir.path().join("config.json"), config.to_string()).unwrap();
        dir
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        /// Round-trip: credentials the source app encrypted come out as Kanso's
        /// server settings unchanged.
        #[test]
        fn prop_encrypted_credentials_are_copied(user in "[a-z]{1,12}", pass in "[ -~]{1,24}", secret in "[A-Za-z0-9]{16}", port in 1024u16..65535) {
            let data = kanso_import::safe_storage::encrypt(format!("{user}:{pass}").as_bytes(), &secret);
            let auth = base64::engine::general_purpose::STANDARD.encode(data);
            let dir = library_dir(serde_json::json!(auth), port);
            let mut settings = ServerSettings::default();
            let copied = copy_server_settings(dir.path(), &mut settings, || Ok(secret.clone())).unwrap();
            prop_assert_eq!(copied, Copied::Everything);
            prop_assert_eq!((settings.port, settings.username, settings.password), (port, user, pass));
        }
    }

    #[test]
    fn a_failed_keychain_read_changes_nothing() {
        let data = kanso_import::safe_storage::encrypt(b"a:b", "s");
        let dir = library_dir(
            serde_json::json!(base64::engine::general_purpose::STANDARD.encode(data)),
            4000,
        );
        let mut settings = ServerSettings::default();
        let before = settings.clone();
        assert!(
            copy_server_settings(dir.path(), &mut settings, || anyhow::bail!("denied")).is_err()
        );
        assert_eq!(settings, before);
    }

    #[test]
    fn plain_credentials_need_no_keychain() {
        let dir = library_dir(serde_json::json!("me:pw"), 19841);
        let mut settings = ServerSettings::default();
        let copied =
            copy_server_settings(dir.path(), &mut settings, || panic!("no keychain needed"))
                .unwrap();
        assert_eq!(copied, Copied::Everything);
        assert_eq!(settings.port, 19841);
        assert_eq!(settings.username, "me");
    }
}
