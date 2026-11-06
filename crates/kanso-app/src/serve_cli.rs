//! `kanso serve`: the local API (REST and `/mcp`) without a window, and,
//! when the library is signed in, sync with its account while it runs
//! (docs/ACCOUNTS.md › Command line). Built for a headless server: no
//! display is touched, the session can live in `secrets.json`
//! (`kanso account sign-in --secrets file`), and SIGTERM stops it cleanly.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{Result, bail};
use kanso_engine::Secrets;
use kanso_engine::account::Flow;
use kanso_engine::sync::driver::{Driver, DriverEvent, DriverSeams};
use kanso_store::{Store, lock_data_dir};
use kanso_sync::{Backend, Machine, SignOutReason, SyncProblem, SyncReport};

use crate::config::AppConfig;

/// Where `serve`'s lines go: standard error in the real command, a list in
/// tests.
pub type Say = Arc<dyn Fn(String) + Send + Sync>;

/// `kanso serve [--verbose] [--as-service]`: runs until SIGINT or SIGTERM (Ctrl-C ends the
/// process on Windows).
pub fn run_cli(data_dir: PathBuf, rest: &[String]) -> Result<()> {
    let options = ServeOptions::parse(rest)?;
    #[cfg(windows)]
    if options.as_service {
        return crate::service_cli::run_windows_service(data_dir);
    }
    std::fs::create_dir_all(&data_dir)?;
    let Some(lock) = lock_data_dir(&data_dir)? else {
        bail!(
            "Kanso is already running with {}; its API is already on",
            data_dir.display()
        );
    };
    let mut config = crate::load_config(&data_dir)?;
    if let Some(related_notes) = options.related_notes {
        config.discovery.related_notes = related_notes;
        config.save(&crate::paths::config_path(&data_dir))?;
    }
    let server = crate::services::server_config(&config.server).map_err(anyhow::Error::msg)?;
    if let Some(warning) = crate::secrets::file_store_warning(
        config.sync.secret_store,
        crate::secrets::key_file().as_deref(),
        &data_dir,
    ) {
        eprintln!("{warning}");
    }
    let store = Arc::new(crate::library_open::open_store(&data_dir)?);
    if options.related_notes == Some(false) {
        kanso_engine::discovery::related::forget(&store)
            .map_err(|err| anyhow::anyhow!("turning related notes off: {err}"))?;
    }
    // A daily snapshot of the database, as the app takes, checked every
    // hour: a server runs for months.
    let backup_store = store.clone();
    let backup_folder = data_dir.clone();
    std::thread::Builder::new()
        .name("kanso-backup".into())
        .spawn(move || {
            loop {
                if let Err(err) = kanso_engine::backup::run_daily(
                    &backup_store,
                    &backup_folder,
                    kanso_core::now_ms(),
                ) {
                    eprintln!("kanso: backup failed: {err}");
                }
                std::thread::sleep(BACKUP_CHECK);
            }
        })?;
    let say: Say = Arc::new(|line| eprintln!("{line}"));
    let keys = kanso_server::KeyStore::open(store.clone())?;
    if let Some(notice) = no_keys_notice(&keys.list()?) {
        eprintln!("{notice}");
    }
    // `kanso api-key` reaches this server over the local control channel,
    // so keys can be made and revoked while it runs.
    let control_target = crate::control::ControlTarget::new(store.clone(), keys.clone());
    let control = match crate::control::ControlServer::start(&data_dir, control_target.clone()) {
        Ok(control) => Some(control),
        Err(err) => {
            eprintln!(
                "kanso serve: `kanso api-key` can't reach this server while it runs ({err}). To change keys, stop `kanso serve` first."
            );
            None
        }
    };
    let sync_status = kanso_server::SyncStatus::default();
    let handle = kanso_server::spawn_with_sync_status(
        store.clone(),
        server,
        kanso_server::TokenStore::new(),
        keys,
        sync_status.clone(),
    );
    let addr = match handle.bind_result() {
        Ok(addr) => addr,
        Err(err) => bail!(
            "couldn't listen on {}:{}: {err}",
            config.server.bind_address,
            config.server.port
        ),
    };
    let secrets = match crate::secrets::open_secrets(&config, &data_dir) {
        Ok(secrets) => Some(secrets),
        Err(err) => {
            say(format!("kanso serve: {err:#}; running signed out."));
            None
        }
    };
    let driver = Arc::new(Mutex::new(secrets.and_then(|secrets| {
        start_sync_with_status(
            SyncStart {
                config: &config,
                data_dir: &data_dir,
                store: store.clone(),
                secrets,
                backend: Arc::new(kanso_sync::HttpBackend::new()),
                machine: Machine::this_computer(),
                shared_notes: true,
                verbose: options.verbose,
                say: say.clone(),
            },
            sync_status.clone(),
        )
    })));
    let account_data_dir = data_dir.clone();
    let account_store = store.clone();
    let account_backend: Arc<dyn Backend> = Arc::new(kanso_sync::HttpBackend::new());
    let account_machine = Machine::this_computer();
    let account_driver = driver.clone();
    let account_status = sync_status.clone();
    let account_say = say.clone();
    let account_verbose = options.verbose;
    control_target.set_account_handler(move |args, console| {
        let reconfigure = args
            .first()
            .is_some_and(|command| matches!(command.as_str(), "sign-in" | "sign-out"));
        if reconfigure
            && let Some(previous) = account_driver
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .take()
        {
            previous.stop();
        }
        let result = crate::account_cli::run_control(&account_data_dir, args, console)
            .map_err(|refusal| refusal.0);
        if reconfigure && let Ok(config) = crate::load_config(&account_data_dir) {
            let next = match crate::secrets::open_secrets(&config, &account_data_dir) {
                Ok(secrets) => start_sync_with_status(
                    SyncStart {
                        config: &config,
                        data_dir: &account_data_dir,
                        store: account_store.clone(),
                        secrets,
                        backend: account_backend.clone(),
                        machine: account_machine.clone(),
                        shared_notes: true,
                        verbose: account_verbose,
                        say: account_say.clone(),
                    },
                    account_status.clone(),
                ),
                Err(error) => {
                    account_say(format!("kanso serve: {error:#}; running signed out."));
                    None
                }
            };
            let mut active = account_driver
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            *active = next;
        }
        result
    });
    let indexer = if config.discovery.related_notes {
        let indexer = kanso_engine::discovery::indexer::Indexer::start(
            store.clone(),
            data_dir.clone(),
            move |status| match status {
                kanso_engine::discovery::related::Status::Indexing { done, total } => {
                    eprintln!(
                        "kanso serve: related notes on; indexing {done} of {total} notes in the background."
                    );
                }
                kanso_engine::discovery::related::Status::Ready { notes } => {
                    eprintln!("kanso serve: related notes ready; {notes} notes indexed.");
                }
                kanso_engine::discovery::related::Status::Failed(error) => {
                    eprintln!("kanso serve: related notes couldn't start: {error}");
                }
                _ => {}
            },
        );
        Some(indexer)
    } else {
        None
    };
    if let Some(line) = exposed_warning(&config.server.bind_address) {
        say(line.to_owned());
    }
    eprintln!("Kanso API on http://{addr} (MCP at /mcp). Ctrl-C stops it.");
    wait_for_stop();
    say("kanso serve: stopping: finishing the sync…".to_owned());
    if let Some(driver) = driver
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .take()
    {
        driver.stop();
    }
    if let Some(indexer) = indexer
        && let Err(err) = indexer.stop(&store)
    {
        eprintln!("kanso serve: stopping related notes: {err}");
    }
    if let Err(err) = handle.stop() {
        eprintln!("kanso serve: stopping the API: {err}");
    }
    drop(control);
    drop(lock);
    say("kanso serve: stopped.".to_owned());
    Ok(())
}

/// The explicit command-line overrides supported by `kanso serve`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ServeOptions {
    pub verbose: bool,
    pub related_notes: Option<bool>,
    pub as_service: bool,
}

impl ServeOptions {
    /// Parses serve flags once so invalid overrides never partially update
    /// the library configuration.
    pub fn parse(args: &[String]) -> Result<Self> {
        let mut options = ServeOptions::default();
        let mut index = 0;
        while index < args.len() {
            match args[index].as_str() {
                "--verbose" | "-v" => options.verbose = true,
                "--as-service" => options.as_service = true,
                "--related-notes" => {
                    index += 1;
                    options.related_notes = Some(match args.get(index).map(String::as_str) {
                        Some("on") => true,
                        Some("off") => false,
                        Some(_) => bail!("--related-notes must be `on` or `off`"),
                        None => bail!("--related-notes needs `on` or `off`"),
                    });
                }
                other => bail!("serve doesn't take `{other}`"),
            }
            index += 1;
        }
        Ok(options)
    }
}

/// How often `serve` checks whether the daily backup is due.
const BACKUP_CHECK: Duration = Duration::from_secs(60 * 60);

/// The warning for an API other computers can reach, or `None` on
/// loopback. Pure.
/// The same test as Settings' warning: anything but a loopback address,
/// including one that doesn't parse, warns.
pub fn exposed_warning(bind_address: &str) -> Option<&'static str> {
    crate::api_keys_model::is_lan_address(bind_address).then_some(
        "kanso serve: the API is reachable from other computers over plain HTTP; put TLS in front or use an SSH tunnel.",
    )
}

/// What `serve` says at start-up when no key in `rows` works, so every
/// request would be refused. Pure.
pub fn no_keys_notice(rows: &[kanso_store::ApiKeyRow]) -> Option<&'static str> {
    rows.iter().all(|row| row.revoked_at.is_some()).then_some(
        "kanso serve: this library has no API keys, so every request is refused. Run `kanso api-key create --name <name>` to make one; it works while this server runs.",
    )
}

/// Blocks until SIGINT or SIGTERM.
// no-prop-test: thin OS signal registration adapter; the signal APIs require a live process session.
#[cfg(unix)]
fn wait_for_stop() {
    let stop = Arc::new(AtomicBool::new(false));
    for signal in [
        signal_hook::consts::SIGINT,
        signal_hook::consts::SIGTERM,
        signal_hook::consts::SIGHUP,
    ] {
        // A second signal while stopping ends the process at once.
        signal_hook::flag::register_conditional_shutdown(signal, 1, stop.clone()).ok();
        if let Err(err) = signal_hook::flag::register(signal, stop.clone()) {
            eprintln!("kanso serve: can't catch signal {signal}: {err}");
        }
    }
    while !stop.load(Ordering::Relaxed) {
        std::thread::sleep(Duration::from_millis(200));
    }
}

/// Windows: Ctrl-C, console close, and the termination event stop the server.
// no-prop-test: this is an OS registration adapter; testing the signal itself requires a Windows console session.
#[cfg(not(unix))]
fn wait_for_stop() {
    #[cfg(windows)]
    if let Some(stop) = crate::service_cli::windows_stop_flag() {
        while !stop.load(Ordering::Relaxed) {
            std::thread::sleep(Duration::from_millis(200));
        }
        return;
    }
    let stop = Arc::new(AtomicBool::new(false));
    let handler_stop = stop.clone();
    if let Err(err) = ctrlc::set_handler(move || handler_stop.store(true, Ordering::Relaxed)) {
        eprintln!("kanso serve: can't catch termination: {err}");
    }
    while !stop.load(Ordering::Relaxed) {
        std::thread::sleep(Duration::from_millis(200));
    }
}

/// What starting sync needs: the real world in `run_cli`, `MemoryServer`
/// and `MemorySecrets` in tests.
pub struct SyncStart<'a> {
    pub config: &'a AppConfig,
    pub data_dir: &'a Path,
    pub store: Arc<Store>,
    pub secrets: Arc<dyn Secrets>,
    pub backend: Arc<dyn Backend>,
    pub machine: Machine,
    /// Shared notes sync on the same cadence (the real servers only).
    pub shared_notes: bool,
    pub verbose: bool,
    pub say: Say,
}

fn sync_snapshot(
    store: &Store,
    session: &kanso_sync::Session,
    device: String,
) -> kanso_server::SyncSnapshot {
    let last_sync_at = store
        .meta(kanso_engine::sync::driver::LAST_SYNC_MS)
        .ok()
        .flatten()
        .and_then(|value| value.parse::<i64>().ok());
    let last_result = store
        .meta(kanso_engine::sync::driver::LAST_RESULT)
        .ok()
        .flatten()
        .and_then(|value| {
            serde_json::from_str::<kanso_engine::sync::driver::LastResult>(&value).ok()
        })
        .map(|result| kanso_server::SyncResult {
            pulled: result.pulled,
            pushed: result.pushed,
            files_up: result.files_up,
            conflicts: result.conflicts,
            pending: result.pending,
        });
    kanso_server::SyncSnapshot {
        signed_in: true,
        username: Some(session.username.clone()),
        server: Some(session.server.clone()),
        device: Some(device),
        state: kanso_server::SyncPhase::Idle,
        last_sync_at,
        last_result,
        problem: None,
        hold_until: None,
    }
}

fn update_sync_phase(
    status: &kanso_server::SyncStatus,
    phase: kanso_server::SyncPhase,
    problem: Option<String>,
) {
    let mut snapshot = status.snapshot();
    snapshot.state = phase;
    snapshot.problem = problem;
    status.set(snapshot);
}

#[cfg(test)]
pub(crate) fn start_sync(start: SyncStart<'_>) -> Option<Driver> {
    start_sync_with_status(start, kanso_server::SyncStatus::default())
}

fn start_sync_with_status(
    start: SyncStart<'_>,
    sync_status: kanso_server::SyncStatus,
) -> Option<Driver> {
    let SyncStart {
        config,
        data_dir,
        store,
        secrets,
        backend,
        machine,
        shared_notes,
        verbose,
        say,
    } = start;
    let device = kanso_sync::device_name(&machine.computer_name);
    let flow = Flow {
        backend: backend.clone(),
        secrets: secrets.clone(),
        store: store.clone(),
        machine: Some(machine),
        backups: crate::paths::backups_dir(data_dir),
        now_ms: kanso_core::now_ms(),
    };
    let account = config
        .sync
        .account
        .as_ref()
        .map(|a| (a.origin.as_str(), a.id.as_str()));
    let Some(restored) = flow.restore(account) else {
        sync_status.set(kanso_server::SyncSnapshot::default());
        say(
            "kanso serve: couldn't read this library's device identity; running signed out."
                .to_owned(),
        );
        return None;
    };
    if restored.kept.is_some() {
        say("kanso serve: this library has a recovery key you haven't saved. Run `kanso account recovery-key`.".to_owned());
    }
    let session = match restored.loaded {
        Ok(Some(session)) => session,
        Ok(None) => {
            sync_status.set(kanso_server::SyncSnapshot::default());
            match restored.pending {
                Ok(Some(locked)) => say(format!(
                    "kanso serve: signed in to {}, but this computer isn't approved yet; run `kanso account sign-in` to finish.",
                    locked.username
                )),
                _ => say(NOT_SIGNED_IN.to_owned()),
            }
            return None;
        }
        Err(err) => {
            sync_status.set(kanso_server::SyncSnapshot::default());
            say(format!(
                "kanso serve: couldn't read the session ({err}); running signed out."
            ));
            return None;
        }
    };
    flow.flush_logouts(&restored.library_id);
    let server = session.server.clone();
    let username = session.username.clone();
    let initial = sync_snapshot(&store, &session, device);
    sync_status.set(initial);
    if !config.sync.enabled {
        say(format!(
            "kanso serve: signed in as {username} on {server}; sync is off in this library's settings."
        ));
        return None;
    }
    let interval = kanso_engine::sync::interval(config.sync.interval_secs);
    say(format!(
        "kanso serve: signed in as {username} on {server}; syncing {} and 20 s after a change.",
        kanso_engine::sync::words::sync_every(interval.as_secs() / 60)
    ));
    let session = Arc::new(session);
    let after_sync = shared_notes.then(|| shared_notes_hook(store.clone(), session.clone()));
    let seams = DriverSeams {
        store,
        backend,
        secrets,
        library_id: restored.library_id,
        session,
        interval,
        after_edit: kanso_engine::sync::AFTER_EDIT,
        now_ms: Arc::new(kanso_core::now_ms),
        after_sync,
        exclusive: None,
    };
    let config_path = crate::paths::config_path(data_dir);
    let mut last_problem = None;
    let event_status = sync_status.clone();
    let on = Box::new(move |event: DriverEvent| {
        if let DriverEvent::SignedOut(_) = &event {
            // As the app does: the config forgets the account, so the next
            // start reads as signed out. The API keeps serving the library.
            forget_account(&config_path);
        }
        // A problem that repeats each tick is said once, until it changes.
        if let DriverEvent::Problem(problem) = &event {
            if last_problem.as_ref() == Some(problem) && !verbose {
                return;
            }
            last_problem = Some(problem.clone());
        } else if matches!(event, DriverEvent::Synced(_)) {
            last_problem = None;
        }
        match &event {
            DriverEvent::SyncStarted => {
                update_sync_phase(&event_status, kanso_server::SyncPhase::Syncing, None)
            }
            DriverEvent::Synced(report) => {
                let mut snapshot = event_status.snapshot();
                snapshot.state = kanso_server::SyncPhase::Idle;
                snapshot.hold_until = None;
                snapshot.problem = None;
                snapshot.last_sync_at = Some(kanso_core::now_ms());
                snapshot.last_result = Some(kanso_server::SyncResult {
                    pulled: report.pulled,
                    pushed: report.pushed,
                    files_up: report.files_up,
                    conflicts: report.conflicts(),
                    pending: report.pending,
                });
                event_status.set(snapshot);
            }
            DriverEvent::Problem(problem) => {
                let mut snapshot = event_status.snapshot();
                snapshot.hold_until = None;
                snapshot.state = match problem.schedule() {
                    kanso_sync::Schedule::WaitsUntil(until) => {
                        snapshot.hold_until = Some(until);
                        kanso_server::SyncPhase::Held
                    }
                    _ => kanso_server::SyncPhase::Idle,
                };
                snapshot.problem =
                    Some(kanso_engine::sync::words::problem_label(problem).to_owned());
                event_status.set(snapshot);
            }
            DriverEvent::SignedOut(_) => event_status.set(kanso_server::SyncSnapshot {
                state: kanso_server::SyncPhase::SignedOut,
                ..kanso_server::SyncSnapshot::default()
            }),
            DriverEvent::Stopped => {
                update_sync_phase(&event_status, kanso_server::SyncPhase::Idle, None)
            }
            _ => {}
        }
        for line in event_lines(&event, verbose) {
            say(line);
        }
    });
    match Driver::start(seams, on) {
        Ok(driver) => {
            let requested = driver.sync_control();
            sync_status.set_sync_now(Arc::new(move || {
                requested.sync_now() != kanso_engine::sync::Request::Refused
            }));
            Some(driver)
        }
        Err(err) => {
            eprintln!("kanso serve: couldn't start syncing: {err}");
            None
        }
    }
}

/// Printed once when the library isn't signed in.
pub const NOT_SIGNED_IN: &str = "kanso serve: not signed in. This library syncs with nothing.\n             Run `kanso account sign-in` to sync it with your Kanso account.";

/// Clears the signed-in account from the config after the server ended the
/// session.
fn forget_account(config_path: &Path) {
    let saved = AppConfig::load(config_path).and_then(|mut config| {
        config.sync.account = None;
        config.save(config_path)
    });
    if let Err(err) = saved {
        eprintln!("kanso serve: couldn't update the config: {err:#}");
    }
}

/// Shared notes' sync, run on the driver's thread after each library sync.
fn shared_notes_hook(
    store: Arc<Store>,
    session: Arc<kanso_sync::Session>,
) -> Arc<dyn Fn() -> Result<(), String> + Send + Sync> {
    let engine = kanso_collab::Engine::new(
        store,
        Arc::new(kanso_collab::http::HttpCollab),
        kanso_collab::Account::of(&session),
    );
    Arc::new(move || engine.sync_all().map(|_| ()).map_err(|err| err.to_string()))
}

/// What `serve` prints for a driver event: nothing for a sync that moved
/// nothing (unless `verbose`), and never a token, key, title or body: the
/// events carry only counts and problems. Pure.
pub fn event_lines(event: &DriverEvent, verbose: bool) -> Vec<String> {
    match event {
        DriverEvent::Started
        | DriverEvent::SyncStarted
        | DriverEvent::Cancelled
        | DriverEvent::Stopped => Vec::new(),
        DriverEvent::Synced(report) => {
            if moved(report) || verbose {
                vec![format!(
                    "kanso serve: synced, {} pulled, {} pushed.",
                    kanso_engine::account::words::format_count(report.pulled as u64),
                    kanso_engine::account::words::format_count(report.pushed as u64)
                )]
            } else {
                Vec::new()
            }
        }
        DriverEvent::Problem(problem) => vec![problem_line(problem)],
        DriverEvent::Remerged => vec![
            "kanso serve: another device's changes were merged with a fresh start of this library."
                .to_owned(),
        ],
        DriverEvent::HookFailed(text) if verbose => {
            vec![format!("kanso serve: shared notes didn't sync: {text}")]
        }
        DriverEvent::HookFailed(_) => Vec::new(),
        DriverEvent::SignedOut(reason) => signed_out_lines(*reason),
    }
}

/// Whether a sync moved anything.
fn moved(report: &SyncReport) -> bool {
    report.pulled + report.pushed + report.files_up + report.files_down > 0
}

/// A problem in a line: the sidebar's label, then what it means.
pub fn problem_line(problem: &SyncProblem) -> String {
    let help = kanso_engine::sync::words::problem_help(problem, |until| {
        chrono::DateTime::from_timestamp_millis(until)
            .map(|at| format!("at {}", at.with_timezone(&chrono::Local).format("%H:%M")))
            .unwrap_or_else(|| "later".to_owned())
    });
    format!(
        "kanso serve: {}: {help}",
        kanso_engine::sync::words::problem_label(problem).to_lowercase()
    )
}

/// The server ended this computer's session.
pub fn signed_out_lines(reason: SignOutReason) -> Vec<String> {
    vec![
        format!(
            "kanso serve: sync stopped: {}",
            kanso_engine::account::words::signed_out_sentence(reason)
        ),
        "             The library stays and the API keeps working. Run `kanso account sign-in` to sync again.".to_owned(),
    ]
}

#[cfg(test)]
#[path = "serve_cli_tests.rs"]
mod tests;
