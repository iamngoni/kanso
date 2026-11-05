//! `kanso serve`: the local API (REST and `/mcp`) without a window, and,
//! when the library is signed in, sync with its account while it runs
//! (docs/ACCOUNTS.md › Command line). Built for a headless server: no
//! display is touched, the session can live in `secrets.json`
//! (`kanso account sign-in --secrets file`), and SIGTERM stops it cleanly.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
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

/// `kanso serve [--verbose]`: runs until SIGINT or SIGTERM (Ctrl-C ends the
/// process on Windows).
pub fn run_cli(data_dir: PathBuf, rest: &[String]) -> Result<()> {
    let mut verbose = false;
    for arg in rest {
        match arg.as_str() {
            "--verbose" | "-v" => verbose = true,
            other => bail!("serve doesn't take `{other}`"),
        }
    }
    std::fs::create_dir_all(&data_dir)?;
    let Some(lock) = lock_data_dir(&data_dir)? else {
        bail!(
            "Kanso is already running with {}; its API is already on",
            data_dir.display()
        );
    };
    let config = crate::load_config(&data_dir)?;
    let server = crate::services::server_config(&config.server).map_err(anyhow::Error::msg)?;
    if let Some(warning) = crate::secrets::file_store_warning(
        config.sync.secret_store,
        crate::secrets::key_file().as_deref(),
        &data_dir,
    ) {
        eprintln!("{warning}");
    }
    let store = Arc::new(crate::library_open::open_store(&data_dir)?);
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
    let handle = kanso_server::spawn(store.clone(), server, keys);
    match handle.bind_result() {
        Ok(addr) => eprintln!("Kanso API on http://{addr} (MCP at /mcp). Ctrl-C stops it."),
        Err(err) => bail!(
            "couldn't listen on {}:{}: {err}",
            config.server.bind_address,
            config.server.port
        ),
    }
    if let Some(line) = exposed_warning(&config.server.bind_address) {
        say(line.to_owned());
    }
    let secrets = match crate::secrets::open_secrets(&config, &data_dir) {
        Ok(secrets) => Some(secrets),
        Err(err) => {
            say(format!("kanso serve: {err:#}; running signed out."));
            None
        }
    };
    let driver = secrets.and_then(|secrets| {
        start_sync(SyncStart {
            config: &config,
            data_dir: &data_dir,
            store: store.clone(),
            secrets,
            backend: Arc::new(kanso_sync::HttpBackend::new()),
            machine: Machine::this_mac(),
            shared_notes: true,
            verbose,
            say: say.clone(),
        })
    });
    wait_for_stop();
    say("kanso serve: stopping: finishing the sync…".to_owned());
    if let Some(driver) = driver {
        driver.stop();
    }
    if let Err(err) = handle.stop() {
        eprintln!("kanso serve: stopping the API: {err}");
    }
    drop(lock);
    say("kanso serve: stopped.".to_owned());
    Ok(())
}

/// How often `serve` checks whether the daily backup is due.
const BACKUP_CHECK: Duration = Duration::from_secs(60 * 60);

/// The warning for an API other computers can reach, or `None` on
/// loopback. Pure.
/// The same test as Settings' warning: anything but a loopback address,
/// including one that doesn't parse, warns.
pub fn exposed_warning(bind_address: &str) -> Option<&'static str> {
    crate::api_keys_model::is_lan_address(bind_address).then_some(
        "kanso serve: binding to a LAN address sends keys unencrypted, so anyone on the network can read them. Set server.bind_address in config.json to 127.0.0.1 to keep the API on this computer, or put TLS in front or use an SSH tunnel.",
    )
}

/// What `serve` says at start-up when no key in `rows` works, so every
/// request would be refused. Pure.
pub fn no_keys_notice(rows: &[kanso_store::ApiKeyRow]) -> Option<&'static str> {
    rows.iter().all(|row| row.revoked_at.is_some()).then_some(
        "kanso serve: this library has no API keys, so every request is refused. Stop `kanso serve` and run `kanso api-key create --name <name>` to make one.",
    )
}

/// Blocks until SIGINT or SIGTERM.
#[cfg(unix)]
fn wait_for_stop() {
    let stop = Arc::new(AtomicBool::new(false));
    for signal in [signal_hook::consts::SIGINT, signal_hook::consts::SIGTERM] {
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

/// Windows: Ctrl-C ends the process, as before.
#[cfg(not(unix))]
fn wait_for_stop() {
    loop {
        std::thread::park();
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

/// Restores the library's session and starts the sync driver when it is
/// signed in and sync is on; says why not otherwise.
pub fn start_sync(start: SyncStart<'_>) -> Option<Driver> {
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
        say(
            "kanso serve: couldn't read this library's device identity; running signed out."
                .to_owned(),
        );
        return None;
    };
    if restored.kept.is_some() {
        say("kanso serve: an unsaved recovery key is kept for this library; open this library in Kanso to see it.".to_owned());
    }
    let session = match restored.loaded {
        Ok(Some(session)) => session,
        Ok(None) => {
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
            say(format!(
                "kanso serve: couldn't read the session ({err}); running signed out."
            ));
            return None;
        }
    };
    flow.flush_logouts(&restored.library_id);
    let server = session.server.clone();
    let username = session.username.clone();
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
        for line in event_lines(&event, verbose) {
            say(line);
        }
    });
    match Driver::start(seams, on) {
        Ok(driver) => Some(driver),
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
