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
use kanso_engine::sync::Request;
use kanso_server::{ServerConfig, ServerHandle, TokenStore};
use kanso_sync::{
    Backend, Machine, PendingApproval, SecretStore, Session, SignOutReason, SignupState, Step,
    SyncError, SyncProblem, SyncReport,
};

use crate::config::ServerSettings;
use crate::workspace::AppState;

/// How often to try again for a port another app holds.
const PORT_RETRY: Duration = Duration::from_secs(15);
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

pub use kanso_engine::sync::SyncState;

pub use kanso_engine::sync::AccountState;

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
    /// A change in Settings › Account › Sign-in methods, after its proof.
    ChangeMethod,
    /// Asking the server once whether it took a shown key Kanso couldn't
    /// tell it took (review-verify 6).
    CheckKey,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RunningTask {
    pub kind: TaskKind,
    pub step: Step,
}

pub use kanso_engine::account::provider::ProviderSignup;

/// A provider flow the account worker is waiting on. Cancellation is logical:
/// Apple may leave its native system sheet open until the user dismisses it.
#[derive(Clone)]
pub struct ProviderWait {
    pub provider: &'static str,
    pub user_code: Option<String>,
    pub verification_uri: Option<String>,
    pub cancel: Arc<std::sync::atomic::AtomicBool>,
}

pub use kanso_engine::account::{AccountDetails, AccountMethodsState, PendingKey};

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

#[cfg(test)]
pub use kanso_engine::sync::Hold;

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
    pub account_methods: AccountMethodsState,
    /// Device approval's live UI and event socket.
    pub approval: ApprovalUi,
    /// What a server last said about new accounts, for the Create account
    /// form: the server's URL and its answer.
    pub signup: Option<(String, SignupState)>,
    /// The server being asked about new accounts right now.
    pub signup_asking: Option<String>,
    /// The provider methods advertised by the current form server. An old
    /// server (or a failed health request) leaves this empty, preserving the
    /// username/password form.
    pub auth: Option<(String, kanso_sync::ServerAuth)>,
    /// Health request in flight, tagged with a generation to reject A→B→A
    /// stale answers.
    pub auth_asking: Option<(String, u64)>,
    /// The server/generation already attempted for the currently open form.
    pub auth_attempted: Option<(String, u64)>,
    pub auth_generation: u64,
    pub provider_signup: Option<ProviderSignup>,
    pub provider_wait: Option<ProviderWait>,
    /// The platform's passkey ceremony failed as unavailable this launch:
    /// the passkey buttons go, and the page said why (docs/ACCOUNTS.md ›
    /// Where the ceremony runs).
    pub platform_passkeys_failed: bool,
    /// Whether the platform can run a passkey ceremony, as tests and
    /// snapshots set it; `None` asks the platform.
    pub platform_passkeys: Option<bool>,
    /// The security keys attached, as the sign-in page last enumerated them
    /// (Linux; always empty elsewhere).
    pub security_keys: Vec<kanso_fido::SecurityKeyDevice>,
    /// Tests: the ceremony passkey tasks run instead of the platform's or a
    /// security key's.
    #[cfg(test)]
    pub test_ceremony: Option<Arc<dyn kanso_sync::PasskeyCeremony + Send + Sync>>,
    /// A provider-authenticated token awaiting a local key, approval, or
    /// recovery. It stays outside `AccountState` and is never a sync session.
    pub locked_session: Option<Arc<kanso_sync::LockedSession>>,
    /// A completed unlock awaiting the Account page's binding-consent dialog
    /// when launch restore completed before any window was available.
    pub(crate) pending_unlocked_candidate: Option<crate::account_flow::UnlockedCandidate>,
    /// Password availability for a locked account; refreshed opportunistically.
    pub locked_has_password: Option<bool>,
    /// Briefly true only while an explicitly confirmed Replace flushes open
    /// editors into the backup before a locked account is promoted.
    locked_replace_write_consent: bool,
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
    /// When syncs run, queue or are held back (`kanso_engine::sync`).
    scheduler: kanso_engine::sync::Scheduler,
    server: Option<ServerHandle>,
    /// The agents' read-only notes tokens (`ai_agent.rs`), accepted on the
    /// local API's `/mcp` while it listens on this computer only, and on
    /// the agents' private listener.
    pub(crate) agent_tokens: TokenStore,
    /// The agents' private listener, started while the local API is off (or
    /// reachable from other computers) and an agent may read the notes.
    pub(crate) agent_mcp: Option<ServerHandle>,
    port_retry: Option<Task<()>>,
    sync_tasks: Vec<Task<()>>,
    sync_after_edit: Option<Task<()>>,
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

    /// Whether this library is in the app-side locked lifecycle, including
    /// while an account task temporarily displays `Busy`.
    pub fn has_locked_session(&self) -> bool {
        self.locked_session.is_some() || matches!(self.account, AccountState::SignedInLocked { .. })
    }

    /// Whether the UI is inside the synchronous, consented Replace flush.
    pub fn locked_replace_write_consent(&self) -> bool {
        self.locked_replace_write_consent
    }

    pub(crate) fn begin_locked_replace_write_consent(&mut self) {
        self.locked_replace_write_consent = true;
    }

    pub(crate) fn end_locked_replace_write_consent(&mut self) {
        self.locked_replace_write_consent = false;
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
        self.locked_session = None;
        self.pending_unlocked_candidate = None;
        self.locked_has_password = None;
        self.approval.waiting = None;
        self.locked_replace_write_consent = false;
        self.account = AccountState::SignedIn {
            username: session.username.clone(),
        };
        self.session = Some(session);
        self.scheduler.clear_hold();
        self.details = AccountDetails::default();
        self.account_methods = AccountMethodsState::default();
    }

    /// Forgets the session (its keys are zeroized once the sync thread lets
    /// go of it too) and stops automatic syncs.
    pub fn clear_session(&mut self, account: AccountState) {
        self.session = None;
        self.locked_session = None;
        self.pending_unlocked_candidate = None;
        self.locked_has_password = None;
        self.approval.waiting = None;
        self.locked_replace_write_consent = false;
        self.account = account;
        self.scheduler.clear_hold();
        self.sync = SyncState::Off;
        self.details = AccountDetails::default();
        self.account_methods = AccountMethodsState::default();
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
                // Agents' tokens open `/mcp` only where every client is on
                // this computer; otherwise agents use their own listener.
                let tokens = if config.bind.ip().is_loopback() {
                    services.agent_tokens.clone()
                } else {
                    TokenStore::new()
                };
                let handle = kanso_server::spawn_with_tokens(store, config, tokens);
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

/// Where an agent reaches the notes server: the local API's `/mcp` when it
/// runs on this computer only, or else the agents' private listener,
/// started now. `None` when neither can listen.
pub fn agent_mcp_endpoint(cx: &mut App) -> Option<String> {
    let store = cx.global::<AppState>().store.clone();
    let services = cx.global_mut::<Services>();
    if let Some(addr) = services.server.as_ref().and_then(ServerHandle::local_addr)
        && addr.ip().is_loopback()
    {
        return Some(format!("http://{addr}/mcp"));
    }
    if services
        .agent_mcp
        .as_ref()
        .is_none_or(|listener| !listener.is_running())
    {
        let listener = kanso_server::spawn_agents(store, services.agent_tokens.clone());
        services.agent_mcp = listener.local_addr().is_some().then_some(listener);
    }
    let addr = services.agent_mcp.as_ref()?.local_addr()?;
    Some(format!("http://{addr}/mcp"))
}

/// Ends every agent's notes token and stops the private listener: AI was
/// turned off, the agent changed, or Kanso is quitting.
pub fn end_agent_access(cx: &mut App) {
    if !cx.has_global::<Services>() {
        return;
    }
    let services = cx.global_mut::<Services>();
    services.agent_tokens.revoke_all();
    // Stopping waits for in-flight requests; a stopping listener is let go.
    services.agent_mcp = None;
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
    let interval = kanso_engine::sync::interval(settings.interval_secs);
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
            if !kanso_engine::sync::is_local_edit(&change) {
                continue;
            }
            cx.update(|cx| {
                let delayed = cx.spawn(async move |cx| {
                    cx.background_executor()
                        .timer(kanso_engine::sync::AFTER_EDIT)
                        .await;
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
    let busy = services.task.is_some();
    match services.scheduler.request(manual, busy, now) {
        Request::Refused | Request::Queued => return,
        Request::Start => {}
    }
    let secrets = services.secrets.clone();
    let library_id = services.library_id.clone();
    services.sync = SyncState::Syncing;

    let cancel = Arc::new(AtomicBool::new(false));
    let thread_cancel = cancel.clone();
    let (tx, rx) = async_channel::bounded(1);
    let thread_session = session.clone();
    let thread = std::thread::Builder::new()
        .name("kanso-sync".into())
        .spawn(move || {
            let logouts = match (&secrets, &library_id) {
                (Some(secrets), Some(library_id)) => Some((secrets.as_ref(), library_id.as_str())),
                _ => None,
            };
            let result = kanso_engine::sync::run_once(
                &store,
                backend.as_ref(),
                &thread_session,
                logouts,
                thread_cancel,
            );
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
            services.scheduler.start_failed();
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
    let finished = services.scheduler.finished(current, result, now);
    if !current {
        cx.refresh_windows();
        if let Some(manual) = finished.again {
            start_sync(manual, cx);
        }
        return;
    }
    if let Some(reason) = finished.signed_out {
        end_session(reason, cx);
        return;
    }
    if finished.cancelled {
        // Cancelled: whatever cancelled it says what happens next.
        if matches!(services.sync, SyncState::Syncing) {
            services.sync = SyncState::Off;
        }
    } else if let Some(state) = finished.state {
        services.sync = state;
    }
    if finished.remerged {
        notify_windows(
            || {
                gpui_kit::component::notification::Notification::info(
                    crate::account_words::REMERGED,
                )
            },
            cx,
        );
    }
    if finished.storage_changed {
        crate::account_flow::refresh_details_after_sync(cx);
    }
    cx.refresh_windows();
    // Automatic requests stay automatic: the holds this sync just set
    // apply to them.
    if let Some(manual) = finished.again {
        start_sync(manual, cx);
    }
}

#[cfg(test)]
impl Services {
    // no-prop-test: delegates to kanso_engine::sync::Scheduler, tested there.
    pub fn set_hold(&mut self, hold: Option<Hold>) {
        self.scheduler.set_hold(hold);
    }

    // no-prop-test: delegates to kanso_engine::sync::Scheduler, tested there.
    pub fn hold(&self) -> Option<Hold> {
        self.scheduler.hold()
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
                    kanso_engine::sync::forget_ended_session(
                        secrets.as_ref(),
                        &library_id,
                        reason,
                        ended.as_deref(),
                        own_recovery,
                    )
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

/// Strategies for the account's and sync's states, shared by the tests of
/// the views that show them. The sync ones are the engine's.
#[cfg(test)]
pub(crate) mod strategies {
    use super::AccountState;
    use kanso_sync::SignOutReason;
    use proptest::prelude::*;

    pub(crate) use kanso_engine::strategies::{problem, step, sync_state};

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
        AccountMethodsState, AccountState, ApiState, Hold, Services, api_state, server_config,
    };
    use crate::config::ServerSettings;
    use kanso_engine::sync::{hold_after, holds_at, state_problem, sync_outcome};
    use kanso_sync::SyncProblem;
    use proptest::prelude::*;
    use std::net::IpAddr;

    fn account_methods(has_password: bool) -> kanso_sync::AccountMethods {
        kanso_sync::AccountMethods {
            has_password,
            rp_id: None,
            available: Vec::new(),
            linked: Vec::new(),
            passkeys: Vec::new(),
        }
    }

    #[test]
    fn signing_out_clears_account_methods() {
        let mut services = Services::default();
        let generation = services.account_methods.begin_refresh();
        services
            .account_methods
            .finish_success(generation, account_methods(true));
        services.clear_session(AccountState::SignedOut);
        assert_eq!(services.account_methods, AccountMethodsState::default());
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

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

        /// Refinement: text that isn't an IP address is refused.
        #[test]
        fn prop_bad_addresses_are_refused(text in "[a-z]{1,10}") {
            let settings = ServerSettings { bind_address: text, ..ServerSettings::default() };
            prop_assert!(server_config(&settings).is_err());
        }
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
}
