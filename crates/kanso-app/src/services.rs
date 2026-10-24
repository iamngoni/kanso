//! Background services the window reports on: the local API, the account,
//! and sync (docs/ACCOUNTS.md › Services).
//!
//! The account's session lives here once loaded, shared with the sync
//! thread as an `Arc`. The backend and the Keychain are seams: `main.rs`
//! puts the real ones in; tests and snapshots leave them out (nothing then
//! reaches a server or the Keychain) or put `MemoryServer` and
//! `MemorySecrets` in.

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use gpui_kit::*;
use kanso_server::{ServerConfig, ServerHandle};
use kanso_store::{Change, Origin};
use kanso_sync::keychain::PendingRecoveryKey;
use kanso_sync::{
    Backend, DeviceInfo, Machine, PendingApproval, RecoveryKey, Schedule, SecretStore, Session,
    SignOutReason, SignupState, Step, SyncError, SyncOptions, SyncProblem, SyncReport, Usage,
};

use crate::config::ServerSettings;
use crate::workspace::AppState;

/// How often to try again for a port another app holds.
const PORT_RETRY: Duration = Duration::from_secs(15);
/// How long after the last local edit to sync.
const SYNC_AFTER_EDIT: Duration = Duration::from_secs(20);

#[derive(Clone, Debug, Default, PartialEq)]
pub enum ApiState {
    #[default]
    Off,
    Running(String),
    /// The port is taken, usually by another app serving the same API; Kanso
    /// keeps trying.
    PortInUse(u16),
    Failed(String),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum SyncState {
    /// Signed out, or signed in and not synced yet.
    #[default]
    Off,
    Syncing,
    Synced {
        at_ms: i64,
        /// Conflicts in that sync; the losing versions are in history.
        conflicts: usize,
        /// The documents that had them, to show and open the note.
        conflicted: Vec<String>,
        /// A problem the sync finished with (`report_problem`).
        problem: Option<SyncProblem>,
    },
    Failed(SyncProblem),
}

/// The account as the app shows it (docs/ACCOUNTS.md › Services).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum AccountState {
    #[default]
    SignedOut,
    SignedIn {
        username: String,
    },
    /// The server ended this Mac's session (a 401): the Sign in form shows
    /// under the reason.
    SessionEnded {
        username: String,
        reason: SignOutReason,
    },
    /// An account task is running, at this step.
    Busy(Step),
}

/// The account tasks, so the page knows whose button spins.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TaskKind {
    SignUp,
    SignIn,
    Recover,
    SignOut,
    ChangePassword,
    NewRecoveryKey,
    DeleteAccount,
    /// "Use a different server…" asking the server whether it is one.
    CheckServer,
    RevokeDevice,
    /// Asking the server once whether it took a shown key Kanso couldn't
    /// tell it took (review-verify 6).
    CheckKey,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RunningTask {
    pub kind: TaskKind,
    pub step: Step,
}

/// A recovery key the user hasn't confirmed saving: the Account page shows
/// it until "I've saved it" (its Keychain item lasts as long).
#[derive(Clone, Debug)]
pub struct PendingKey {
    pub key: RecoveryKey,
    pub username: String,
    pub server: String,
    /// What else the screen says: the task that made the key failed, or
    /// Kanso couldn't tell whether the server took it.
    pub note: Option<&'static str>,
}

impl PendingKey {
    /// The Keychain's unsaved key; `note` shows when the server may not
    /// have taken it.
    pub fn kept(item: PendingRecoveryKey, note: &'static str) -> PendingKey {
        PendingKey {
            note: item.unconfirmed.then_some(note),
            key: item.recovery_key,
            username: item.username,
            server: item.server,
        }
    }
}

/// The signed-in account's storage and devices, as last fetched.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AccountDetails {
    pub usage: Option<Usage>,
    pub devices: Option<Vec<DeviceInfo>>,
    pub loading: bool,
    /// A sync changed what the server holds while a fetch ran: that fetch's
    /// answer may be from before it, so it is fetched again.
    pub stale: bool,
    /// Why the last fetch failed, in plain words.
    pub failed: Option<String>,
}

/// The live account approval UI: what the approver's sheet shows, and the
/// new device's waiting page. The socket's stop flag lives here so a
/// sign-out or a new session can end it.
#[derive(Default)]
pub struct ApprovalUi {
    /// The open approver sheet, if any.
    pub sheet: Option<Entity<crate::device_approval::ApprovalSheet>>,
    /// The account's pending requests from other devices, as last read.
    pub pending: Vec<PendingApproval>,
    /// The new device's waiting page, once it asks to be approved.
    pub waiting: Option<crate::device_approval::WaitingState>,
    /// Stops the account event socket when set.
    pub event_stop: Option<Arc<AtomicBool>>,
}

/// What keeps automatic syncs from running after a problem (the problems
/// table's last column). "Sync now" always runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hold {
    /// Until this time (ms): a paused account.
    Until(i64),
    /// Until Sync now.
    UntilSyncNow,
    /// Until the user signs in again.
    Stopped,
}

/// What the schedule does after `problem`.
pub fn hold_after(problem: Option<&SyncProblem>) -> Option<Hold> {
    match problem?.schedule() {
        Schedule::KeepsRunning => None,
        Schedule::WaitsUntil(until) => Some(Hold::Until(until)),
        Schedule::StopsUntilSyncNow => Some(Hold::UntilSyncNow),
        Schedule::Stops => Some(Hold::Stopped),
    }
}

/// Whether `hold` keeps an automatic sync from running at `now_ms`.
pub fn holds_at(hold: Option<Hold>, now_ms: i64) -> bool {
    match hold {
        None => false,
        Some(Hold::Until(until)) => now_ms < until,
        Some(Hold::UntilSyncNow | Hold::Stopped) => true,
    }
}

/// The running sync's thread, and the flag that stops it between pages,
/// batches and files.
pub struct SyncHandle {
    cancel: Arc<AtomicBool>,
    thread: std::thread::JoinHandle<()>,
}

impl SyncHandle {
    /// Stops the sync and waits for its thread: every re-bind and sign-out
    /// does this first, off the main thread (fix 2).
    pub fn cancel_and_join(self) {
        self.cancel.store(true, Ordering::SeqCst);
        self.thread.join().ok();
    }
}

#[derive(Default)]
pub struct Services {
    pub api: ApiState,
    pub sync: SyncState,
    /// Signed out, in, or ended; never `Busy`, which [`Self::account_state`]
    /// adds while a task runs.
    pub account: AccountState,
    pub task: Option<RunningTask>,
    pub pending_key: Option<PendingKey>,
    /// The shown key is one Kanso couldn't tell the server took, and the
    /// server hasn't been asked about it yet: it is asked once, when no
    /// task runs (`recover/start` counts against a per-address limit).
    pub key_check_due: bool,
    pub details: AccountDetails,
    /// Device approval's live UI and event socket.
    pub approval: ApprovalUi,
    /// What a server last said about new accounts, for the Create account
    /// form: the server's URL and its answer.
    pub signup: Option<(String, SignupState)>,
    /// The server being asked about new accounts right now.
    pub signup_asking: Option<String>,
    /// The servers' seam: `HttpBackend` in the app, `MemoryServer` in tests.
    /// Without one, nothing reaches a server.
    pub backend: Option<Arc<dyn Backend>>,
    /// Published notes' seam: `HttpShares` in the app, `MemoryShares` in
    /// tests. Without one, nothing is published.
    pub shares: Option<Arc<dyn kanso_sync::share::ShareApi>>,
    /// Shared notes' seam: `HttpCollab` in the app, `MemoryCollab` in
    /// tests. Without one, nothing is shared (`crate::collab`).
    pub collab: Option<Arc<dyn kanso_collab::CollabApi>>,
    /// The Keychain's seam: `Keychain` in the app, `MemorySecrets` in tests.
    pub secrets: Option<Arc<dyn SecretStore>>,
    /// This Mac, as tests fake it; `None` asks the system (on a background
    /// thread).
    pub machine: Option<Machine>,
    /// The library's ID once known, naming its Keychain items.
    pub library_id: Option<String>,
    /// The launch check is reading the Keychain: account tasks wait (A4).
    pub restoring: bool,
    /// A Replace is backing the library up and emptying it: the editors
    /// keep their edits unsaved until it is done (A10).
    pub writes_held: bool,
    session: Option<Arc<Session>>,
    hold: Option<Hold>,
    server: Option<ServerHandle>,
    port_retry: Option<Task<()>>,
    sync_tasks: Vec<Task<()>>,
    sync_after_edit: Option<Task<()>>,
    sync_running: bool,
    /// A sync asked for while one ran, to run right after it: `Some(true)`
    /// when any of the requests was Sync now (sync 8).
    sync_again: Option<bool>,
    sync_handle: Option<SyncHandle>,
    /// The session whose sync an account task stopped: it starts again
    /// when the last task of the chain ends with it still signed in (A8).
    pub(crate) stopped_for: Option<Arc<Session>>,
    /// How many syncs started, for tests.
    #[cfg(test)]
    pub(crate) sync_starts: usize,
}

impl Global for Services {}

impl Services {
    pub fn api_status(&self) -> String {
        match &self.api {
            ApiState::Off => "API off".into(),
            ApiState::Running(addr) => format!("API on {addr}"),
            ApiState::PortInUse(port) => {
                format!("API off: port {port} is in use by another app")
            }
            ApiState::Failed(reason) => format!("API off: {reason}"),
        }
    }

    /// The account, `Busy` while a task runs.
    pub fn account_state(&self) -> AccountState {
        match self.task {
            Some(task) => AccountState::Busy(task.step),
            None => self.account.clone(),
        }
    }

    /// Whether a session is loaded: what Sync now and the signed-in
    /// commands need.
    pub fn signed_in(&self) -> bool {
        matches!(self.account, AccountState::SignedIn { .. })
    }

    pub fn session(&self) -> Option<Arc<Session>> {
        self.session.clone()
    }

    /// Sync's state in a few words, for a tooltip: the sidebar's line,
    /// with what a problem means.
    pub fn sync_status(&self, cx: &App) -> String {
        let account = self.account_state();
        let (_, label) = crate::sidebar::sync_line(&self.sync, &account, |at| {
            crate::clock::Clock::edited_label(cx, at)
        });
        let problem = match (&account, &self.sync) {
            (AccountState::SignedIn { .. }, SyncState::Failed(problem))
            | (
                AccountState::SignedIn { .. },
                SyncState::Synced {
                    problem: Some(problem),
                    ..
                },
            ) => Some(problem),
            _ => None,
        };
        match (&account, problem) {
            (AccountState::SessionEnded { reason, .. }, _) => {
                format!(
                    "{}\n{}",
                    label,
                    crate::account_words::signed_out_sentence(*reason)
                )
            }
            (_, Some(problem)) => format!(
                "{label}\n{}",
                crate::settings::problem_help(problem, |at| crate::account_words::resume_time(
                    crate::clock::Clock::local_ms(cx, at),
                    crate::clock::Clock::local_ms(cx, crate::clock::Clock::now_ms(cx)),
                ))
            ),
            (AccountState::SignedIn { username }, None) => {
                format!("{label}\nSigned in as {username}")
            }
            _ => label,
        }
    }

    /// Takes the running sync out, for an account task to stop and join it
    /// on its own thread. No sync starts while the task runs.
    pub fn take_sync_handle(&mut self) -> Option<SyncHandle> {
        self.sync_handle.take()
    }

    /// Signs the window in with `session` (after the binding, the Keychain
    /// item and the config are written).
    pub fn set_signed_in(&mut self, session: Arc<Session>) {
        self.account = AccountState::SignedIn {
            username: session.username.clone(),
        };
        self.session = Some(session);
        self.hold = None;
        self.details = AccountDetails::default();
    }

    /// Forgets the session (its keys are zeroized once the sync thread lets
    /// go of it too) and stops automatic syncs.
    pub fn clear_session(&mut self, account: AccountState) {
        self.session = None;
        self.account = account;
        self.hold = None;
        self.sync = SyncState::Off;
        self.details = AccountDetails::default();
        self.sync_tasks.clear();
        self.sync_after_edit = None;
    }
}

/// The server's listen address and credentials from the settings.
pub fn server_config(settings: &ServerSettings) -> Result<ServerConfig, String> {
    let ip: IpAddr = settings
        .bind_address
        .trim()
        .parse()
        .map_err(|_| format!("“{}” isn't an IP address", settings.bind_address))?;
    if settings.username.is_empty() || settings.password.is_empty() {
        return Err("set a username and password first".into());
    }
    Ok(ServerConfig {
        bind: SocketAddr::new(ip, settings.port),
        username: settings.username.clone(),
        password: settings.password.clone(),
    })
}

/// What the status bar says for a server that just tried to start.
pub fn api_state(handle: &ServerHandle, port: u16) -> ApiState {
    match handle.bind_result() {
        Ok(addr) => ApiState::Running(format!("http://{addr}")),
        Err(_) if handle.is_addr_in_use() => ApiState::PortInUse(port),
        Err(err) => ApiState::Failed(err.to_string()),
    }
}

/// Starts, stops or restarts the local API to match the settings.
pub fn apply_server_settings(cx: &mut App) {
    let state = cx.global::<AppState>();
    let settings = state.config.server.clone();
    let store = state.store.clone();
    let services = cx.global_mut::<Services>();
    services.port_retry = None;
    if let Some(old) = services.server.take() {
        // Waits for in-flight requests (at most a few seconds) so the port
        // is free before binding it again.
        if let Err(err) = old.stop() {
            eprintln!("kanso: the API server stopped with an error: {err:#}");
        }
    }
    services.api = if !settings.enabled {
        ApiState::Off
    } else {
        match server_config(&settings) {
            Err(reason) => ApiState::Failed(reason),
            Ok(config) => {
                let handle = kanso_server::spawn(store, config);
                let api = api_state(&handle, settings.port);
                if matches!(api, ApiState::Running(_)) {
                    services.server = Some(handle);
                }
                api
            }
        }
    };
    if matches!(services.api, ApiState::PortInUse(_)) {
        let retry = cx.spawn(async move |cx| {
            cx.background_executor().timer(PORT_RETRY).await;
            cx.update(apply_server_settings);
        });
        cx.global_mut::<Services>().port_retry = Some(retry);
    }
    cx.refresh_windows();
}

/// Before a Replace backs the library up and empties it (A10): every
/// editor saves now, so its edits are in the backup; then editor saves are
/// held, and the local API (with MCP) is taken out to be stopped off the
/// main thread. Returns it, and whether it is started again afterwards.
pub fn hold_writes(cx: &mut App) -> (Option<ServerHandle>, bool) {
    crate::workspace::for_each_editor(cx, |editor, cx| editor.flush(cx));
    let services = cx.global_mut::<Services>();
    services.writes_held = true;
    let restart = services.server.is_some() || services.port_retry.is_some();
    services.port_retry = None;
    (services.server.take(), restart)
}

/// After the Replace: the API starts again if it ran, and the editors save
/// what was typed meanwhile, unless their note went with the old library
/// (they ask about that themselves).
pub fn release_writes(restart: bool, cx: &mut App) {
    cx.global_mut::<Services>().writes_held = false;
    if restart {
        apply_server_settings(cx);
    }
    crate::workspace::for_each_editor(cx, |editor, cx| editor.save_held(cx));
}

/// Stops the API before quitting.
pub fn stop_server(cx: &mut App) {
    let services = cx.global_mut::<Services>();
    services.port_retry = None;
    if let Some(server) = services.server.take() {
        server.stop().ok();
    }
}

/// What the status shows after a sync: `None` for one that was cancelled,
/// which changes nothing.
pub fn sync_outcome(result: Result<SyncReport, SyncError>, now_ms: i64) -> Option<SyncState> {
    match result {
        Ok(report) => Some(SyncState::Synced {
            at_ms: now_ms,
            conflicts: report.conflicts(),
            problem: kanso_sync::report_problem(&report),
            conflicted: report.conflicted,
        }),
        Err(err) => kanso_sync::problem(&err).map(SyncState::Failed),
    }
}

/// Whether a sync changed what the account holds on the server: it pushed,
/// uploaded, or pulled what another Mac pushed.
pub fn changed_storage(report: &SyncReport) -> bool {
    report.pushed > 0 || report.files_up > 0 || report.pulled > 0
}

/// The problem a sync state shows, if any.
pub fn state_problem(state: &SyncState) -> Option<&SyncProblem> {
    match state {
        SyncState::Failed(problem)
        | SyncState::Synced {
            problem: Some(problem),
            ..
        } => Some(problem),
        _ => None,
    }
}

/// Whether a change should be sent to the server soon. Changes that came
/// from sync, and bulk imports, wait for the regular sync.
pub fn is_local_edit(change: &Change) -> bool {
    matches!(change.origin, Origin::Ui | Origin::Api)
}

/// Starts or stops background sync to match the settings: it runs while
/// signed in with Sync automatically on.
pub fn apply_sync_settings(cx: &mut App) {
    let state = cx.global::<AppState>();
    let settings = state.config.sync.clone();
    let store = state.store.clone();
    let services = cx.global_mut::<Services>();
    services.sync_tasks.clear();
    services.sync_after_edit = None;
    if services.session.is_none() {
        services.sync = SyncState::Off;
        cx.refresh_windows();
        return;
    }
    let interval = Duration::from_secs(settings.interval_secs.max(60));
    // Shared notes sync on the same cadence, whether or not the library
    // syncs on its own: invitations and mentions come that way.
    let shared = crate::collab::periodic(interval, cx);
    if !settings.enabled {
        cx.global_mut::<Services>().sync_tasks = vec![shared];
        cx.refresh_windows();
        return;
    }
    let periodic = cx.spawn(async move |cx| {
        loop {
            cx.update(|cx| start_sync(false, cx));
            cx.background_executor().timer(interval).await;
        }
    });
    let changes = store.subscribe();
    let after_edits = cx.spawn(async move |cx| {
        while let Ok(change) = changes.recv().await {
            if !is_local_edit(&change) {
                continue;
            }
            cx.update(|cx| {
                let delayed = cx.spawn(async move |cx| {
                    cx.background_executor().timer(SYNC_AFTER_EDIT).await;
                    cx.update(|cx| start_sync(false, cx));
                });
                cx.global_mut::<Services>().sync_after_edit = Some(delayed);
            });
        }
    });
    let services = cx.global_mut::<Services>();
    services.sync_tasks = vec![periodic, after_edits, shared];
}

/// Syncs once, now, on its own thread ("Sync now"): it runs whatever held
/// automatic syncs back. A request during a sync runs right after it.
pub fn sync_now(cx: &mut App) {
    start_sync(true, cx);
    crate::collab::sync_soon(cx);
}

/// Starts an automatic sync again after an account task stopped one and
/// left the same session signed in: the holds still apply.
pub fn resume_sync(cx: &mut App) {
    start_sync(false, cx);
}

/// Starts a sync of the signed-in library. Nothing happens while signed
/// out, while an account task runs, or (for automatic syncs) while a
/// problem holds them.
fn start_sync(manual: bool, cx: &mut App) {
    let now = crate::clock::Clock::now_ms(cx);
    let store = cx.global::<AppState>().store.clone();
    let services = cx.global_mut::<Services>();
    let (Some(session), Some(backend)) = (services.session.clone(), services.backend.clone())
    else {
        return;
    };
    if services.task.is_some() || (!manual && holds_at(services.hold, now)) {
        return;
    }
    if manual {
        services.hold = None;
    }
    if services.sync_running {
        services.sync_again = Some(queued_again(services.sync_again, manual));
        return;
    }
    let secrets = services.secrets.clone();
    let library_id = services.library_id.clone();
    services.sync_running = true;
    services.sync = SyncState::Syncing;

    let cancel = Arc::new(AtomicBool::new(false));
    let options = SyncOptions {
        cancel: Some(cancel.clone()),
        ..SyncOptions::default()
    };
    let (tx, rx) = async_channel::bounded(1);
    let thread_session = session.clone();
    let thread = std::thread::Builder::new()
        .name("kanso-sync".into())
        .spawn(move || {
            if let (Some(secrets), Some(library_id)) = (&secrets, &library_id) {
                kanso_sync::flush_pending_logouts(backend.as_ref(), secrets.as_ref(), library_id);
            }
            let result = kanso_sync::sync(&store, backend.as_ref(), &thread_session, &options);
            tx.try_send(result).ok();
        });
    match thread {
        Ok(thread) => {
            services.sync_handle = Some(SyncHandle { cancel, thread });
            #[cfg(test)]
            {
                services.sync_starts += 1;
            }
        }
        Err(err) => {
            eprintln!("kanso: couldn't start the sync thread: {err}");
            services.sync_running = false;
            services.sync = SyncState::Failed(SyncProblem::Other);
            cx.refresh_windows();
            return;
        }
    }
    cx.refresh_windows();
    cx.spawn(async move |cx| {
        // A thread that panicked drops the sender: that reads as a failure.
        let result = rx.recv().await.ok();
        cx.update(|cx| finish_sync(session, result, cx));
    })
    .detach();
}

fn finish_sync(session: Arc<Session>, result: Option<Result<SyncReport, SyncError>>, cx: &mut App) {
    let now = crate::clock::Clock::now_ms(cx);
    let services = cx.global_mut::<Services>();
    services.sync_running = false;
    // The handle is gone already when an account task took it to stop us.
    if services
        .sync_handle
        .as_ref()
        .is_some_and(|handle| handle.thread.is_finished())
    {
        services.sync_handle = None;
    }
    let current = services
        .session
        .as_ref()
        .is_some_and(|s| Arc::ptr_eq(s, &session));
    let again = services.sync_again.take();
    if !current {
        // Signed out or in again meanwhile: this sync's news is stale, but
        // a sync asked for meanwhile is the current session's.
        cx.refresh_windows();
        if let Some(manual) = again {
            start_sync(manual, cx);
        }
        return;
    }
    let remerged = matches!(&result, Some(Ok(report)) if report.remerged);
    // What the server holds changed: the Account page's storage line is
    // fetched again. A sync that moved nothing asks for nothing (each
    // account has 120 requests a minute).
    let changed = matches!(&result, Some(Ok(report)) if changed_storage(report));
    let outcome = match result {
        Some(result) => sync_outcome(result, now),
        None => Some(SyncState::Failed(SyncProblem::Other)),
    };
    match outcome {
        // Cancelled: whatever cancelled it says what happens next.
        None => {
            if matches!(services.sync, SyncState::Syncing) {
                services.sync = SyncState::Off;
            }
        }
        Some(state) => {
            services.hold = hold_after(state_problem(&state));
            if let SyncState::Failed(SyncProblem::SignedOut(reason)) = state {
                end_session(reason, cx);
                return;
            }
            services.sync = state;
        }
    }
    if remerged {
        notify_windows(
            || {
                gpui_kit::component::notification::Notification::info(
                    crate::account_words::REMERGED,
                )
            },
            cx,
        );
    }
    if changed {
        crate::account_flow::refresh_details_after_sync(cx);
    }
    cx.refresh_windows();
    // Automatic requests stay automatic: the holds this sync just set
    // apply to them.
    if let Some(manual) = again {
        start_sync(manual, cx);
    }
}

/// The sync to run after the current one, given what was queued and a new
/// request: manual if any request was.
fn queued_again(queued: Option<bool>, manual: bool) -> bool {
    queued.unwrap_or(false) || manual
}

#[cfg(test)]
impl Services {
    pub fn set_hold(&mut self, hold: Option<Hold>) {
        self.hold = hold;
    }

    pub fn hold(&self) -> Option<Hold> {
        self.hold
    }
}

/// Shows `notification` in every main window, for news that comes from no
/// window in particular (a sync, an account task finishing).
pub fn notify_windows(
    notification: impl Fn() -> gpui_kit::component::notification::Notification + 'static,
    cx: &mut App,
) {
    use gpui_kit::component::WindowExt as _;
    // A window can't be reached from inside its own update.
    cx.defer(move |cx| {
        for handle in cx.windows() {
            handle
                .update(cx, |_, window, cx| {
                    window.push_notification(notification(), cx)
                })
                .ok();
        }
    });
}

/// The server ended this Mac's session (a 401): the Keychain item goes,
/// the page shows the reason over the Sign in form, and syncing stops.
///
/// This Mac's own "Forgot your password?" ends every session the account
/// had, this one too, so a request still in flight on it gets a 401
/// (`unknown` once the new session replaced its row, or `recovered`)
/// before the recovery has signed in again (review-verify 3). The
/// recovery's new key is then no stale key: while a recovery runs, the key
/// stays, in the Keychain and on screen. And the session item goes only
/// while it still holds the ended session, never the one the recovery
/// saved meanwhile.
pub fn end_session(reason: SignOutReason, cx: &mut App) {
    crate::device_approval::stop_events(cx);
    let services = cx.global_mut::<Services>();
    let own_recovery = services
        .task
        .is_some_and(|task| task.kind == TaskKind::Recover);
    let username = match &services.account {
        AccountState::SignedIn { username } => username.clone(),
        AccountState::SessionEnded { username, .. } => username.clone(),
        _ => services
            .session
            .as_ref()
            .map(|s| s.username.clone())
            .unwrap_or_default(),
    };
    let secrets = services.secrets.clone();
    let library_id = services.library_id.clone();
    let ended = services.session.clone();
    services.clear_session(AccountState::SessionEnded { username, reason });
    crate::collab::signed_out(cx);
    if let (Some(secrets), Some(library_id)) = (secrets, library_id) {
        cx.spawn(async move |cx| {
            let stale = cx
                .background_spawn(async move {
                    let deleted = match ended.as_deref() {
                        Some(ended) => kanso_sync::keychain::delete_ended_session(
                            secrets.as_ref(),
                            &library_id,
                            ended,
                        ),
                        None => kanso_sync::keychain::delete_session(secrets.as_ref(), &library_id),
                    };
                    if let Err(err) = deleted {
                        eprintln!("kanso: couldn't remove the ended session: {err}");
                    }
                    if own_recovery {
                        return None;
                    }
                    drop_stale_key(secrets.as_ref(), &library_id, reason, ended.as_deref())
                })
                .await;
            if let Some(stale) = stale {
                cx.update(|cx| {
                    let services = cx.global_mut::<Services>();
                    if services
                        .pending_key
                        .as_ref()
                        .is_some_and(|shown| shown.key == stale)
                    {
                        services.pending_key = None;
                    }
                    cx.refresh_windows();
                });
            }
        })
        .detach();
    }
    AppState::update_config(cx, |config| config.sync.account = None);
    cx.refresh_windows();
}

/// Whether `kept` is a recovery key the server no longer holds, because the
/// session of `origin`'s `account_id` ended with `reason`: another device
/// recovered the account or changed its password, and either replaced the
/// key. A key the server took before that is stale. One whose request went
/// unanswered stays: the server may have taken it after, so it may be the
/// only working key (sec-client 3).
pub fn stale_key(
    kept: &PendingRecoveryKey,
    reason: SignOutReason,
    origin: &str,
    account_id: &str,
) -> bool {
    matches!(
        reason,
        SignOutReason::PasswordChanged | SignOutReason::Recovered
    ) && !kept.unconfirmed
        && kept.origin == origin
        && kept.account_id == account_id
}

/// Deletes the library's unsaved recovery key when it is [`stale_key`];
/// returns the key it deleted.
fn drop_stale_key(
    secrets: &dyn SecretStore,
    library_id: &str,
    reason: SignOutReason,
    ended: Option<&Session>,
) -> Option<RecoveryKey> {
    let ended = ended?;
    let kept = match kanso_sync::keychain::load_any_recovery_key(secrets, library_id) {
        Ok(kept) => kept?,
        Err(err) => {
            eprintln!("kanso: reading the unsaved recovery key: {err}");
            return None;
        }
    };
    if !stale_key(&kept, reason, &ended.origin, &ended.account_id) {
        return None;
    }
    match kanso_sync::keychain::delete_recovery_key(secrets, library_id) {
        Ok(()) => Some(kept.recovery_key),
        Err(err) => {
            eprintln!("kanso: removing a recovery key replaced elsewhere: {err}");
            None
        }
    }
}

/// Strategies for the account's and sync's states, shared by the tests of
/// the views that show them.
#[cfg(test)]
pub(crate) mod strategies {
    use super::{AccountState, SyncState};
    use kanso_sync::{QuotaKind, SignOutReason, Step, SyncProblem};
    use proptest::prelude::*;

    pub fn problem() -> impl Strategy<Value = SyncProblem> {
        prop_oneof![
            prop::sample::select(SignOutReason::ALL.to_vec()).prop_map(SyncProblem::SignedOut),
            Just(SyncProblem::AccountDisabled),
            (0i64..1_000_000).prop_map(|until| SyncProblem::Paused { until }),
            (0i64..1_000_000).prop_map(|until| SyncProblem::ServerBusy { until }),
            (0i64..1_000_000).prop_map(|until| SyncProblem::BackOff { until }),
            prop::sample::select(QuotaKind::ALL.to_vec()).prop_map(SyncProblem::QuotaExceeded),
            Just(SyncProblem::ServerFull),
            (1usize..100).prop_map(|count| SyncProblem::Skipped { count }),
            Just(SyncProblem::UpdateKanso),
            Just(SyncProblem::WrongKey),
            Just(SyncProblem::ServerOutdated),
            Just(SyncProblem::Offline),
            Just(SyncProblem::Other),
        ]
    }

    pub fn step() -> impl Strategy<Value = Step> {
        prop::sample::select(vec![
            Step::StoppingSync,
            Step::DerivingKeys,
            Step::Contacting,
            Step::Saving,
        ])
    }

    pub fn sync_state() -> impl Strategy<Value = SyncState> {
        prop_oneof![
            Just(SyncState::Off),
            Just(SyncState::Syncing),
            (0i64..1_000_000_000, 0usize..4, prop::option::of(problem())).prop_map(
                |(at_ms, conflicts, problem)| SyncState::Synced {
                    at_ms,
                    conflicts,
                    conflicted: Vec::new(),
                    problem,
                }
            ),
            problem().prop_map(SyncState::Failed),
        ]
    }

    pub fn account() -> impl Strategy<Value = AccountState> {
        prop_oneof![
            Just(AccountState::SignedOut),
            "[a-z]{3,12}".prop_map(|username| AccountState::SignedIn { username }),
            (
                "[a-z]{3,12}",
                prop::sample::select(SignOutReason::ALL.to_vec())
            )
                .prop_map(|(username, reason)| AccountState::SessionEnded { username, reason }),
            step().prop_map(AccountState::Busy),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AccountState, ApiState, Hold, SyncState, api_state, hold_after, holds_at, is_local_edit,
        queued_again, server_config, stale_key, state_problem, sync_outcome,
    };
    use crate::config::ServerSettings;
    use kanso_sync::RecoveryKey;
    use kanso_sync::keychain::PendingRecoveryKey;
    use kanso_sync::{ApiError, SignOutReason, SyncError, SyncProblem, SyncReport};
    use proptest::prelude::*;
    use std::net::IpAddr;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// A sync asked for during another runs once after it, manual if
        /// any of the requests was.
        #[test]
        fn prop_queued_syncs_are_manual_only_if_one_was(requests in prop::collection::vec(any::<bool>(), 1..12)) {
            let queued = requests
                .iter()
                .fold(None, |queued, &manual| Some(queued_again(queued, manual)));
            prop_assert_eq!(queued, Some(requests.iter().any(|&manual| manual)));
        }

        /// Round-trip: the listen address keeps the configured IP and port,
        /// IPv6 included.
        #[test]
        fn prop_server_config_keeps_ip_and_port(ip in any::<IpAddr>(), port in any::<u16>()) {
            let settings = ServerSettings {
                bind_address: ip.to_string(),
                port,
                username: "u".into(),
                password: "p".into(),
                ..ServerSettings::default()
            };
            let config = server_config(&settings).unwrap();
            prop_assert_eq!((config.bind.ip(), config.bind.port()), (ip, port));
        }

        /// Sec-client 3: an unsaved key is dropped only when the session of
        /// its own account ended because another device replaced the key,
        /// and only when the server had taken it.
        #[test]
        fn prop_only_a_key_replaced_elsewhere_is_stale(
            reason in prop::sample::select(SignOutReason::ALL.to_vec()),
            unconfirmed: bool,
            same_origin: bool,
            same_account: bool,
        ) {
            let kept = PendingRecoveryKey {
                origin: "https://a.example".into(),
                account_id: "acct".into(),
                username: "alice".into(),
                server: "https://a.example".into(),
                recovery_key: RecoveryKey::from_bytes([7; 16]),
                unconfirmed,
            };
            let origin = if same_origin { "https://a.example" } else { "https://b.example" };
            let account = if same_account { "acct" } else { "other" };
            let replaced = matches!(reason, SignOutReason::PasswordChanged | SignOutReason::Recovered);
            prop_assert_eq!(
                stale_key(&kept, reason, origin, account),
                replaced && !unconfirmed && same_origin && same_account
            );
        }

        /// Refinement: text that isn't an IP address is refused.
        #[test]
        fn prop_bad_addresses_are_refused(text in "[a-z]{1,10}") {
            let settings = ServerSettings { bind_address: text, ..ServerSettings::default() };
            prop_assert!(server_config(&settings).is_err());
        }

        /// The schedule after a problem follows the problems table: what
        /// keeps running never holds, a pause holds exactly until its end,
        /// and the rest hold until someone acts.
        #[test]
        fn prop_holds_follow_the_problems_table(problem in super::strategies::problem(), now in 0i64..2_000_000) {
            let hold = hold_after(Some(&problem));
            match problem.schedule() {
                kanso_sync::Schedule::KeepsRunning => prop_assert_eq!(hold, None),
                kanso_sync::Schedule::WaitsUntil(until) => {
                    prop_assert_eq!(hold, Some(Hold::Until(until)));
                    prop_assert_eq!(holds_at(hold, now), now < until);
                }
                kanso_sync::Schedule::StopsUntilSyncNow => prop_assert_eq!(hold, Some(Hold::UntilSyncNow)),
                kanso_sync::Schedule::Stops => prop_assert_eq!(hold, Some(Hold::Stopped)),
            }
            if hold.is_some() && !matches!(hold, Some(Hold::Until(_))) {
                prop_assert!(holds_at(hold, now));
            }
            prop_assert_eq!(hold_after(None), None);
            prop_assert!(!holds_at(None, now));
        }

        /// A finished sync is Synced, with the problem `report_problem`
        /// finds; a failed one shows `problem`'s; a cancelled one changes
        /// nothing.
        #[test]
        fn prop_sync_outcomes_follow_the_report(
            conflicts_local in 0usize..3,
            conflicts_remote in 0usize..3,
            skipped in 0usize..3,
            too_new: bool,
            paused in prop::option::of(0i64..1_000),
            at in 0i64..1_000_000,
        ) {
            let report = SyncReport {
                conflicts_local,
                conflicts_remote,
                skipped: (0..skipped).map(|n| format!("note:{n}")).collect(),
                too_new,
                paused_until: paused,
                ..SyncReport::default()
            };
            let state = sync_outcome(Ok(report.clone()), at).unwrap();
            let SyncState::Synced { at_ms, conflicts, problem, .. } = &state else {
                panic!("{state:?}");
            };
            prop_assert_eq!(*at_ms, at);
            prop_assert_eq!(*conflicts, conflicts_local + conflicts_remote);
            prop_assert_eq!(problem.clone(), kanso_sync::report_problem(&report));
            prop_assert_eq!(state_problem(&state), problem.as_ref());
        }
    }

    #[test]
    fn sync_errors_become_problems() {
        assert_eq!(sync_outcome(Err(SyncError::Cancelled), 1), None);
        assert_eq!(
            sync_outcome(Err(SyncError::WrongKey), 1),
            Some(SyncState::Failed(SyncProblem::WrongKey))
        );
        assert_eq!(
            sync_outcome(
                Err(SyncError::Api(ApiError::Unauthorized {
                    reason: SignOutReason::PasswordChanged
                })),
                1
            ),
            Some(SyncState::Failed(SyncProblem::SignedOut(
                SignOutReason::PasswordChanged
            )))
        );
        assert_eq!(
            sync_outcome(Err(SyncError::Api(ApiError::Transport("reset".into()))), 1),
            Some(SyncState::Failed(SyncProblem::Offline))
        );
        assert_eq!(
            sync_outcome(Err(SyncError::Api(ApiError::NotFound)), 1),
            Some(SyncState::Failed(SyncProblem::ServerOutdated))
        );
        // A clean sync has no problem, and holds nothing back.
        let clean = sync_outcome(Ok(SyncReport::default()), 7).unwrap();
        assert_eq!(
            clean,
            SyncState::Synced {
                at_ms: 7,
                conflicts: 0,
                conflicted: Vec::new(),
                problem: None,
            }
        );
        assert_eq!(hold_after(state_problem(&clean)), None);
    }

    #[test]
    fn a_taken_port_is_reported_as_in_use() {
        let taken = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = taken.local_addr().unwrap().port();
        let dir = tempfile::tempdir().unwrap();
        let store = std::sync::Arc::new(kanso_store::Store::open(dir.path().join("k.db")).unwrap());
        let settings = ServerSettings {
            port,
            username: "u".into(),
            password: "p".into(),
            ..ServerSettings::default()
        };
        let handle = kanso_server::spawn(store.clone(), server_config(&settings).unwrap());
        assert_eq!(api_state(&handle, port), ApiState::PortInUse(port));
        drop(taken);
        let settings = ServerSettings {
            port: 0,
            ..settings
        };
        let handle = kanso_server::spawn(store, server_config(&settings).unwrap());
        assert!(matches!(api_state(&handle, 0), ApiState::Running(_)));
        handle.stop().unwrap();
    }

    /// A library on `server` for `username`, signed in from Mac `n` and
    /// bound fresh: its store, session and folder.
    fn library(
        server: &kanso_sync::MemoryServer,
        username: &str,
        n: u8,
        sign_up: bool,
    ) -> (kanso_store::Store, kanso_sync::Session, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let store = kanso_store::Store::open(dir.path().join("k.db")).unwrap();
        let device = kanso_sync::DeviceIdentity::from_parts(
            &format!("l_{n:032x}"),
            &format!("Mac {n}"),
            [n; 32],
        );
        let password = kanso_sync::Password::new("correct horse battery", username).unwrap();
        let server_url = kanso_sync::DEFAULT_SERVER;
        let session = if sign_up {
            kanso_sync::sign_up(
                server,
                &kanso_sync::MemorySecrets::new(),
                server_url,
                username,
                &password,
                &device,
                &|_| {},
            )
            .unwrap()
            .session
        } else {
            kanso_sync::sign_in(server, server_url, username, &password, &device, &|_| {})
                .unwrap()
                .session
        };
        kanso_sync::bind(
            &store,
            &session,
            kanso_sync::Choice::Add,
            &dir.path().join("backups"),
            0,
        )
        .unwrap();
        (store, session, dir)
    }

    /// End to end through the Worker's rules (in memory): two libraries of
    /// one account sync through `kanso_sync::sync`, encrypted, and the status
    /// reflects the result.
    #[test]
    fn syncing_two_libraries_reports_synced() {
        use kanso_core::DocType;
        use kanso_store::{Origin, Put};
        let server = kanso_sync::MemoryServer::new();
        let (a, a_session, _a_dir) = library(&server, "alice", 1, true);
        let (b, b_session, _b_dir) = library(&server, "alice", 2, false);
        let book = a
            .put(
                DocType::Book,
                serde_json::json!({"name": "B"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
        let note = a
            .put(
                DocType::Note,
                serde_json::json!({"title": "hello", "bookId": book.id}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
        let options = kanso_sync::SyncOptions::default();
        let state = sync_outcome(kanso_sync::sync(&a, &server, &a_session, &options), 7);
        assert_eq!(
            state,
            Some(SyncState::Synced {
                at_ms: 7,
                conflicts: 0,
                conflicted: Vec::new(),
                problem: None,
            })
        );
        // The server holds both, under opaque IDs.
        let view = server.plain_view(&a_session);
        assert_eq!(view.docs().len(), 2);
        assert!(
            view.wire_docs()
                .iter()
                .all(|doc| doc.id != note.id && doc.id != book.id)
        );

        let state = sync_outcome(kanso_sync::sync(&b, &server, &b_session, &options), 8);
        assert!(matches!(
            state,
            Some(SyncState::Synced { problem: None, .. })
        ));
        assert_eq!(b.note(&note.id).unwrap().unwrap().title, "hello");
    }

    /// The server's own daily cap (`writes-global`) reads "Server busy", not
    /// the account's "Sync paused", and holds automatic syncs until the time
    /// the server gave.
    #[test]
    fn a_server_wide_writes_429_reads_server_busy() {
        let server = kanso_sync::MemoryServer::new();
        let (a, session, _dir) = library(&server, "alice", 1, true);
        // A cap of 0 leaves every account above its fair share (0 / 50).
        server.set_setting("global_writes_per_day", "0");
        a.put(
            kanso_core::DocType::Book,
            serde_json::json!({"name": "Inbox"}),
            kanso_store::Put::Upsert,
            kanso_store::Origin::Ui,
        )
        .unwrap();
        let now = kanso_core::now_ms();
        let state = sync_outcome(
            kanso_sync::sync(&a, &server, &session, &kanso_sync::SyncOptions::default()),
            now,
        )
        .expect("a finished sync");
        let problem = state_problem(&state).cloned().expect("a problem");
        let SyncProblem::ServerBusy { until } = problem else {
            panic!("{problem:?}");
        };
        assert!(until > now);
        let signed_in = AccountState::SignedIn {
            username: "alice".into(),
        };
        let (_, label) = crate::sidebar::sync_line(&state, &signed_in, |_| String::new());
        assert_eq!(label, "Server busy");
        let help = crate::settings::problem_help(&problem, |_| "at midnight".into());
        assert!(
            help.starts_with("The server has taken all the changes"),
            "{help}"
        );
        let hold = hold_after(Some(&problem));
        assert_eq!(hold, Some(Hold::Until(until)));
        assert!(holds_at(hold, until - 1));
        assert!(!holds_at(hold, until));
    }

    /// An account past its day of upload bytes (`uploads-account`) reads
    /// "Sync paused" until the time the server gave, and holds automatic
    /// syncs until then.
    #[test]
    fn an_upload_budget_429_reads_sync_paused() {
        let server = kanso_sync::MemoryServer::new();
        let (a, session, _dir) = library(&server, "alice", 1, true);
        server.set_upload_bytes_per_day(1);
        a.put(
            kanso_core::DocType::File,
            serde_json::json!({
                "_id": "file:img1",
                "name": "a.png",
                "contentType": "image/png",
                "createdAt": 1,
                "_attachments": {"index": {"content_type": "image/png", "data": "Ynl0ZXM="}},
            }),
            kanso_store::Put::Upsert,
            kanso_store::Origin::Ui,
        )
        .unwrap();
        let now = kanso_core::now_ms();
        let state = sync_outcome(
            kanso_sync::sync(&a, &server, &session, &kanso_sync::SyncOptions::default()),
            now,
        )
        .expect("a finished sync");
        let problem = state_problem(&state).cloned().expect("a problem");
        let SyncProblem::Paused { until } = problem else {
            panic!("{problem:?}");
        };
        assert!(until > now);
        let signed_in = AccountState::SignedIn {
            username: "alice".into(),
        };
        let (_, label) = crate::sidebar::sync_line(&state, &signed_in, |_| String::new());
        assert_eq!(label, "Sync paused");
        let hold = hold_after(Some(&problem));
        assert_eq!(hold, Some(Hold::Until(until)));
        assert!(holds_at(hold, until - 1));
        assert!(!holds_at(hold, until));
    }

    #[test]
    fn only_local_edits_trigger_a_quick_sync() {
        let change = |origin| kanso_store::Change {
            id: "note:a".into(),
            rev: "1-a".into(),
            seq: 1,
            deleted: false,
            origin,
        };
        assert!(is_local_edit(&change(kanso_store::Origin::Ui)));
        assert!(is_local_edit(&change(kanso_store::Origin::Api)));
        assert!(!is_local_edit(&change(kanso_store::Origin::Sync)));
        assert!(!is_local_edit(&change(kanso_store::Origin::Import)));
    }
}
