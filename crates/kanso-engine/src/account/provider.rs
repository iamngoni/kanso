//! Signing in with Apple, Google and GitHub through the Worker
//! (docs/ACCOUNTS.md › Sign in with Apple, Google and GitHub).
//!
//! The app never embeds a web view for sign-in and never sees a provider
//! token: the Worker exchanges the code and verifies the ID token. What this
//! module decides is the same on every platform: which routes a flow uses,
//! when to poll and how often, when a flow has expired or been cancelled, and
//! what a finished flow leaves to do. What it needs of the system's browser
//! is the [`Browser`] seam, which each app fills in: a loopback listener for
//! Google's redirect, the pages to open, and Apple's callback.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use base64::Engine as _;
use kanso_sync::api::LoginDevice;
use kanso_sync::keychain;
use kanso_sync::provider::{
    GithubStart, GoogleStart, IdentityIntent, LinkOutcome, ProofWire, Provider, ProviderSession,
    ProviderSignIn, ProviderStart,
};
use kanso_sync::{
    AccountError, Backend, DeviceIdentity, LocalLibrary, LockedSession, SecretError, Session, Step,
};
use sha2::Digest as _;

use super::words::ErrorContext;
use super::{Authenticated, Failure, Flow};
use crate::Secrets;

/// What can go wrong in the system's browser, as far as a flow cares.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrowserError {
    /// The person dismissed the browser.
    Cancelled,
    /// The provider's absolute flow deadline elapsed.
    Expired,
    /// The provider returned an explicit denial or cancellation.
    ProviderDenied,
    /// Anything else: the listener or the browser failed, or this platform
    /// has no such session.
    Failed,
}

/// A one-request listener for Google's redirect, bound before the Worker is
/// asked to start the flow so its address can be the redirect URI.
pub trait Listener: Send {
    /// The exact redirect URI.
    fn uri(&self) -> String;

    /// Waits for one valid callback carrying `state`, without blocking
    /// cancellation or the flow's absolute deadline; returns its `code`.
    fn wait_until(
        &self,
        state: &str,
        deadline: Instant,
        cancel: &AtomicBool,
    ) -> Result<String, BrowserError>;
}

/// Where Apple's callback URL arrives once the sign-in sheet has finished.
pub trait AppleWait: Send {
    fn poll(&self) -> ApplePoll;
}

/// What Apple's callback has done so far.
pub enum ApplePoll {
    Waiting,
    /// Nothing will arrive: the sheet's end went away.
    Closed,
    /// The callback URL, or why there is none.
    Answer(Result<String, BrowserError>),
}

/// What a provider sign-in asks of the system's browser. The desktop opens
/// the default browser and binds a loopback; a phone has its own sessions.
pub trait Browser: Send + Sync {
    /// Starts listening for Google's redirect.
    fn listen(&self) -> Result<Box<dyn Listener>, BrowserError>;
    /// Opens Google's consent page.
    fn open_google(&self, start: &GoogleStart) -> Result<(), BrowserError>;
    /// Opens GitHub's device page; the caller shows the code.
    fn open_github(&self, start: &GithubStart) -> Result<(), BrowserError>;
}

/// A provider ticket waiting for the user to choose a username. Kept only in
/// process until signup or cancellation; it is never persisted.
#[derive(Clone)]
pub struct ProviderSignup {
    pub server: String,
    pub provider: Provider,
    pub ticket: String,
    pub suggested_username: String,
    pub login_device: LoginDevice,
}

/// Which routes a provider flow uses: the public sign-in and sign-up ones,
/// or a signed-in session's link or proof (docs/ACCOUNTS.md › Sign-in routes
/// › `identities/<provider>/start`).
#[derive(Clone)]
pub enum ProviderRoute {
    Public {
        server: String,
    },
    Session {
        session: Arc<Session>,
        intent: IdentityIntent,
        proof: Option<ProofWire>,
    },
}

/// One poll or finish of a provider flow.
pub enum Polled {
    Done(ProviderDone),
    Pending { slow_down: bool },
}

/// A provider flow's answer: a sign-in's, or a link's or proof's.
pub enum ProviderDone {
    SignIn(ProviderSignIn),
    Link(LinkOutcome),
}

impl Polled {
    pub fn sign_in(step: kanso_sync::ProviderStep) -> Polled {
        use kanso_sync::ProviderStep;
        match step {
            ProviderStep::Session(session) => {
                Polled::Done(ProviderDone::SignIn(ProviderSignIn::Session(session)))
            }
            ProviderStep::NewAccount(account) => {
                Polled::Done(ProviderDone::SignIn(ProviderSignIn::NewAccount(account)))
            }
            ProviderStep::Pending { slow_down } => Polled::Pending { slow_down },
        }
    }

    pub fn link(outcome: LinkOutcome) -> Polled {
        match outcome {
            LinkOutcome::Pending { slow_down } => Polled::Pending { slow_down },
            done => Polled::Done(ProviderDone::Link(done)),
        }
    }
}

impl ProviderRoute {
    pub fn start(
        &self,
        backend: &dyn Backend,
        provider: &Provider,
        device: &LoginDevice,
        loopback: Option<&kanso_sync::Loopback>,
        challenge: Option<&str>,
        app_scheme: Option<&str>,
    ) -> Result<ProviderStart, AccountError> {
        match self {
            ProviderRoute::Public { server } => kanso_sync::start_provider_with_scheme(
                backend, server, provider, device, loopback, challenge, app_scheme,
            ),
            ProviderRoute::Session {
                session,
                intent,
                proof,
            } => kanso_sync::account::start_identity_with_scheme(
                backend,
                session,
                provider,
                *intent,
                proof.clone(),
                loopback,
                challenge,
                device,
                app_scheme,
            ),
        }
    }

    /// Apple's or GitHub's poll.
    pub fn poll(
        &self,
        backend: &dyn Backend,
        provider: &Provider,
        flow: &str,
        secret: Option<&str>,
        device: &LoginDevice,
    ) -> Result<Polled, AccountError> {
        match self {
            ProviderRoute::Public { server } => {
                kanso_sync::poll_provider(backend, server, provider, flow, secret, device)
                    .map(Polled::sign_in)
            }
            ProviderRoute::Session { session, .. } => kanso_sync::account::finish_identity(
                backend, session, provider, flow, secret, device,
            )
            .map(Polled::link),
        }
    }

    /// Google's finish.
    pub fn google(
        &self,
        backend: &dyn Backend,
        flow: &str,
        code: &str,
        verifier: &str,
        device: &LoginDevice,
    ) -> Result<Polled, AccountError> {
        match self {
            ProviderRoute::Public { server } => {
                kanso_sync::finish_google(backend, server, flow, code, verifier, device)
                    .map(Polled::sign_in)
            }
            ProviderRoute::Session { session, .. } => kanso_sync::account::finish_identity_google(
                backend,
                session,
                &Provider::Google,
                flow,
                code,
                verifier,
                device,
            )
            .map(Polled::link),
        }
    }
}

/// A flow the Worker has started, and what finishing it needs.
pub struct StartedProviderFlow {
    pub start: ProviderStart,
    loopback: Option<Box<dyn Listener>>,
    verifier: Option<String>,
    deadline: Instant,
}

/// How a provider sign-in task ended.
pub enum ProviderTaskResult {
    Cancelled,
    NewAccount(kanso_sync::NewAccount),
    Locked(Box<LockedSession>, DeviceIdentity),
}

/// Preserves the provider flow's exact device proof and raw eviction answer
/// in the durable locked item before the UI exposes the locked state.
pub fn locked_provider_session(
    server: String,
    login_device: LoginDevice,
    session: ProviderSession,
    library_id: &str,
) -> Result<LockedSession, AccountError> {
    let origin = kanso_sync::origin(&server).map_err(AccountError::Endpoint)?;
    let mut locked = LockedSession::new(
        server,
        origin,
        session.account_id,
        session.username,
        login_device.id.clone(),
        session.token,
        session.key_id,
        session.epoch,
        session.passkey_wrapped_key,
        None,
    )
    .with_has_password(session.has_password)
    .with_login_device(login_device)
    .with_evicted_wire_device(session.evicted_device);
    locked.set_library_id(library_id);
    Ok(locked)
}

/// Makes the provider token durable before it can enter the app's locked
/// state. A failed save revokes or queues that token instead of exposing an
/// unusable pending account.
pub fn save_provider_pending(
    backend: &dyn Backend,
    secrets: &dyn Secrets,
    locked: &LockedSession,
) -> Result<(), SecretError> {
    let Some(library_id) = locked.library_id() else {
        return Err(SecretError("the locked session has no library ID".into()));
    };
    match keychain::save_locked_pending(secrets, library_id, locked) {
        Ok(()) => Ok(()),
        Err(error) => {
            if let Err(cleanup) = kanso_sync::discard_locked_session(backend, secrets, locked) {
                eprintln!("kanso: discarding provider session after Keychain failure: {cleanup}");
            }
            Err(error)
        }
    }
}

/// Starts the server side of a provider flow. This runs on the worker; any
/// native authentication sheet is opened later by the main-thread callback.
pub fn provider_begin(
    backend: &dyn Backend,
    browser: &dyn Browser,
    route: &ProviderRoute,
    provider: &Provider,
    device: &LoginDevice,
    progress: &dyn Fn(Step),
    app_scheme: Option<&str>,
) -> Result<StartedProviderFlow, AccountError> {
    let (loopback, verifier, challenge) = if matches!(provider, Provider::Google) {
        let loopback = browser
            .listen()
            .map_err(|_| AccountError::ProviderUnavailable)?;
        let verifier: String = (0..64).map(|_| fastrand::alphanumeric()).collect();
        let digest = sha2::Sha256::digest(verifier.as_bytes());
        let challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(digest);
        (Some(loopback), Some(verifier), Some(challenge))
    } else {
        (None, None, None)
    };
    let wire_loopback = loopback.as_ref().map(|loopback| {
        kanso_sync::Loopback::parse(&loopback.uri()).expect("bound loopback has canonical URI")
    });
    let start = route.start(
        backend,
        provider,
        device,
        wire_loopback.as_ref(),
        challenge.as_deref(),
        app_scheme,
    )?;
    let deadline = provider_deadline(&start);
    progress(Step::Contacting);
    progress(Step::Contacting);
    Ok(StartedProviderFlow {
        start,
        loopback,
        verifier,
        deadline,
    })
}

/// Finishes a started flow off the UI thread. The optional Apple callback
/// was obtained by presenting the system's sign-in session on the main
/// thread. `None` is a flow cancelled on this device.
#[allow(clippy::too_many_arguments)] // The arguments are the explicit ceremony context.
pub fn provider_finish(
    backend: &dyn Backend,
    browser: &dyn Browser,
    route: &ProviderRoute,
    provider: &Provider,
    device: &LoginDevice,
    started: StartedProviderFlow,
    apple_callback: Option<Box<dyn AppleWait>>,
    cancel: Arc<AtomicBool>,
    _progress: &dyn Fn(Step),
) -> Result<Option<ProviderDone>, AccountError> {
    let StartedProviderFlow {
        start,
        loopback,
        verifier,
        deadline,
    } = started;
    let outcome = match (&start, provider) {
        (ProviderStart::Apple(start), Provider::Apple) => {
            let callback = apple_callback.ok_or(AccountError::ProviderUnavailable)?;
            let callback = loop {
                if wait_cancelled_or_expired(&cancel, deadline)? {
                    return Ok(None);
                }
                match callback.poll() {
                    ApplePoll::Answer(result) => {
                        break result.map_err(|err| match err {
                            BrowserError::Cancelled | BrowserError::ProviderDenied => {
                                AccountError::ProviderDenied
                            }
                            _ => AccountError::ProviderUnavailable,
                        })?;
                    }
                    ApplePoll::Closed => {
                        return Err(AccountError::ProviderUnavailable);
                    }
                    ApplePoll::Waiting => {
                        if wait_provider(&cancel, deadline, Duration::from_millis(50))? {
                            return Ok(None);
                        }
                    }
                }
            };
            // Only the callback for the flow this app started counts; its secret
            // proves to the Worker that this process finished it at Apple.
            let callback = kanso_sync::apple_callback_flow(&callback)
                .filter(|callback| callback.flow == start.flow_id)
                .ok_or(AccountError::ProviderDenied)?;
            loop {
                if wait_cancelled_or_expired(&cancel, deadline)? {
                    return Ok(None);
                }
                let answer = route.poll(
                    backend,
                    provider,
                    &callback.flow,
                    Some(&callback.secret),
                    device,
                )?;
                if wait_cancelled_or_expired(&cancel, deadline)? {
                    return Ok(None);
                }
                match answer {
                    Polled::Done(done) => break done,
                    Polled::Pending { .. } => {
                        if wait_provider(&cancel, deadline, Duration::from_secs(4))? {
                            return Ok(None);
                        }
                    }
                }
            }
        }
        (ProviderStart::Google(start), Provider::Google) => {
            browser
                .open_google(start)
                .map_err(|_| AccountError::ProviderUnavailable)?;
            let code = match loopback
                .as_ref()
                .ok_or(AccountError::ProviderUnavailable)?
                .wait_until(&start.state, deadline, &cancel)
            {
                Ok(code) => code,
                Err(BrowserError::Cancelled) if cancel.load(Ordering::Acquire) => {
                    return Ok(None);
                }
                Err(BrowserError::Expired) => {
                    return Err(AccountError::FlowExpired);
                }
                Err(_) => return Err(AccountError::ProviderUnavailable),
            };
            if wait_cancelled_or_expired(&cancel, deadline)? {
                return Ok(None);
            }
            let answer = route.google(
                backend,
                &start.flow_id,
                &code,
                verifier.as_deref().ok_or(AccountError::BadRequest)?,
                device,
            )?;
            if wait_cancelled_or_expired(&cancel, deadline)? {
                return Ok(None);
            }
            match answer {
                Polled::Done(done) => done,
                Polled::Pending { .. } => return Err(AccountError::ProviderUnavailable),
            }
        }
        (ProviderStart::Github(start), Provider::Github) => {
            browser
                .open_github(start)
                .map_err(|_| AccountError::ProviderUnavailable)?;
            let mut interval = github_poll_interval(start.interval);
            // The Worker rejects a poll before next_poll_at.
            if wait_provider(&cancel, deadline, Duration::from_secs(interval))? {
                return Ok(None);
            }
            loop {
                if wait_cancelled_or_expired(&cancel, deadline)? {
                    return Ok(None);
                }
                let answer = route.poll(backend, provider, &start.flow_id, None, device)?;
                if wait_cancelled_or_expired(&cancel, deadline)? {
                    return Ok(None);
                }
                match answer {
                    Polled::Done(done) => break done,
                    Polled::Pending { slow_down } => {
                        if slow_down {
                            interval = github_next_poll_interval(interval, slow_down);
                        }
                        if wait_provider(&cancel, deadline, Duration::from_secs(interval))? {
                            return Ok(None);
                        }
                    }
                }
            }
        }
        _ => return Err(AccountError::ProviderUnavailable),
    };
    Ok(Some(outcome))
}

fn provider_deadline(start: &ProviderStart) -> Instant {
    let remaining = match start {
        ProviderStart::Github(start) => Duration::from_secs(start.expires_in),
        ProviderStart::Apple(start) => remaining_until_epoch_ms(start.expires_at),
        ProviderStart::Google(start) => remaining_until_epoch_ms(start.expires_at),
    };
    Instant::now() + remaining
}

fn remaining_until_epoch_ms(expires_at: i64) -> Duration {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;
    Duration::from_millis(expires_at.saturating_sub(now_ms).max(0) as u64)
}

/// Returns true only for local cancellation; expiry remains a user-visible flow error.
pub fn wait_cancelled_or_expired(
    cancel: &AtomicBool,
    deadline: Instant,
) -> Result<bool, AccountError> {
    if cancel.load(Ordering::Acquire) {
        return Ok(true);
    }
    if Instant::now() >= deadline {
        return Err(AccountError::FlowExpired);
    }
    Ok(false)
}

/// Waits in short slices so cancellation and expiry are observed promptly.
fn wait_provider(
    cancel: &AtomicBool,
    deadline: Instant,
    duration: Duration,
) -> Result<bool, AccountError> {
    let end = Instant::now() + duration;
    loop {
        if wait_cancelled_or_expired(cancel, deadline)? {
            return Ok(true);
        }
        let remaining = end.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Ok(false);
        }
        std::thread::sleep(remaining.min(Duration::from_millis(50)));
    }
}

/// The seconds between GitHub polls to start with: the server's, never
/// under one.
pub fn github_poll_interval(start_interval: u64) -> u64 {
    start_interval.max(1)
}

/// The seconds between GitHub polls after an answer: five more each time
/// GitHub says to slow down.
pub fn github_next_poll_interval(interval: u64, slow_down: bool) -> u64 {
    if slow_down {
        interval.saturating_add(5)
    } else {
        interval
    }
}

// no-prop-test: the account tasks' workers, each a sequence of calls to
// kanso_sync and the secret store; tested end to end against MemoryServer and
// MemorySecrets, and the decisions between them by property.
impl Flow {
    /// The worker half of starting a provider sign-in on `server`.
    pub fn start_provider(
        &self,
        browser: &dyn Browser,
        server: &str,
        provider: &Provider,
        device: &LoginDevice,
        step: &dyn Fn(Step),
    ) -> Result<StartedProviderFlow, Failure> {
        self.start_provider_with_scheme(browser, server, provider, device, step, None)
    }

    /// [`Flow::start_provider`] with the app's callback scheme.
    pub fn start_provider_with_scheme(
        &self,
        browser: &dyn Browser,
        server: &str,
        provider: &Provider,
        device: &LoginDevice,
        step: &dyn Fn(Step),
        app_scheme: Option<&str>,
    ) -> Result<StartedProviderFlow, Failure> {
        provider_begin(
            self.backend.as_ref(),
            browser,
            &ProviderRoute::Public {
                server: server.to_owned(),
            },
            provider,
            device,
            step,
            app_scheme,
        )
        .map_err(|err| Failure::account(err, ErrorContext::SignIn))
    }

    /// The worker half of finishing a provider sign-in: waits for the
    /// person, then keeps a session durably as a locked one, or hands back
    /// the ticket for a new account.
    #[allow(clippy::too_many_arguments)] // The arguments are the explicit ceremony context.
    pub fn finish_provider(
        &self,
        browser: &dyn Browser,
        server: &str,
        provider: &Provider,
        login_device: &LoginDevice,
        started: StartedProviderFlow,
        apple_callback: Option<Box<dyn AppleWait>>,
        cancel: Arc<AtomicBool>,
        step: &dyn Fn(Step),
    ) -> Result<ProviderTaskResult, Failure> {
        let answer = provider_finish(
            self.backend.as_ref(),
            browser,
            &ProviderRoute::Public {
                server: server.to_owned(),
            },
            provider,
            login_device,
            started,
            apple_callback,
            cancel,
            step,
        )
        .map_err(|err| Failure::account(err, ErrorContext::SignIn))?;
        match answer {
            None => Ok(ProviderTaskResult::Cancelled),
            Some(ProviderDone::Link(_)) => Err(Failure::account(
                AccountError::Protocol("a link answer to a sign-in".into()),
                ErrorContext::SignIn,
            )),
            Some(ProviderDone::SignIn(ProviderSignIn::NewAccount(account))) => {
                Ok(ProviderTaskResult::NewAccount(account))
            }
            Some(ProviderDone::SignIn(ProviderSignIn::Session(session))) => {
                let device = self.device()?;
                let locked = locked_provider_session(
                    server.to_owned(),
                    login_device.clone(),
                    session,
                    &device.library_id,
                )
                .map_err(|error| Failure::account(error, ErrorContext::SignIn))?;
                if let Err(error) =
                    save_provider_pending(self.backend.as_ref(), self.secrets.as_ref(), &locked)
                {
                    return Err(Failure::keychain(error));
                }
                Ok(ProviderTaskResult::Locked(Box::new(locked), device))
            }
        }
    }

    /// The library a provider sign-up would bind, for the question about
    /// notes already here. A new account is never the library's own.
    pub fn provider_sign_up_library(&self, server: &str) -> Result<LocalLibrary, Failure> {
        let origin = kanso_sync::origin(server)
            .map_err(|err| Failure::account(AccountError::Endpoint(err), ErrorContext::SignUp))?;
        self.library(&origin, "")
    }

    /// Creates a provider-backed passwordless account after the user
    /// confirms an editable username, then reads the library as for a
    /// password sign-up.
    pub fn create_provider_signup(
        &self,
        server: &str,
        pending: &ProviderSignup,
        username: &str,
        library: LocalLibrary,
        step: &dyn Fn(Step),
    ) -> Result<Authenticated, Failure> {
        let device = self.device()?;
        let signed = kanso_sync::sign_up_with_provider(
            self.backend.as_ref(),
            self.secrets.as_ref(),
            server,
            &pending.provider,
            &pending.ticket,
            username,
            &device,
            &pending.login_device,
            step,
        )
        .map_err(|err| {
            Failure::rotation(
                err,
                ErrorContext::SignUp,
                super::words::KEY_UNCONFIRMED_SIGN_UP,
            )
        })?;
        Ok(Authenticated {
            session: signed.session,
            evicted: None,
            library,
            library_id: device.library_id,
            recovery_key: Some(signed.recovery_key),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use kanso_sync::{AccountError, LocalLibrary, MemoryServer, ProviderStep};
    use proptest::prelude::*;

    use super::super::lab::{Lab, SERVER, password};
    use super::super::words;
    use super::*;

    /// A browser that opens nothing: it keeps what it was asked to open and
    /// answers Google's redirect with `code`.
    struct FakeBrowser {
        opened: Mutex<Vec<String>>,
        code: Result<String, BrowserError>,
    }

    impl FakeBrowser {
        fn new(code: Result<&str, BrowserError>) -> FakeBrowser {
            FakeBrowser {
                opened: Mutex::new(Vec::new()),
                code: code.map(str::to_owned),
            }
        }

        fn opened(&self) -> Vec<String> {
            self.opened.lock().unwrap().clone()
        }
    }

    struct FakeListener(Result<String, BrowserError>);

    impl Listener for FakeListener {
        fn uri(&self) -> String {
            "http://127.0.0.1:43123".to_owned()
        }

        fn wait_until(&self, _: &str, _: Instant, _: &AtomicBool) -> Result<String, BrowserError> {
            self.0.clone()
        }
    }

    impl Browser for FakeBrowser {
        fn listen(&self) -> Result<Box<dyn Listener>, BrowserError> {
            Ok(Box::new(FakeListener(self.code.clone())))
        }

        fn open_google(&self, start: &GoogleStart) -> Result<(), BrowserError> {
            self.opened.lock().unwrap().push(start.auth_url.clone());
            Ok(())
        }

        fn open_github(&self, start: &GithubStart) -> Result<(), BrowserError> {
            self.opened
                .lock()
                .unwrap()
                .push(start.verification_uri.clone());
            Ok(())
        }
    }

    /// Apple's callback, already arrived.
    struct FakeApple(Mutex<Option<Result<String, BrowserError>>>);

    impl AppleWait for FakeApple {
        fn poll(&self) -> ApplePoll {
            match self.0.lock().unwrap().take() {
                Some(answer) => ApplePoll::Answer(answer),
                None => ApplePoll::Closed,
            }
        }
    }

    fn no_step(_: Step) {}

    fn not_cancelled() -> Arc<AtomicBool> {
        Arc::new(AtomicBool::new(false))
    }

    #[test]
    fn provider_wait_observes_cancellation_and_deadline_without_sleeping() {
        let cancel = AtomicBool::new(false);
        let deadline = Instant::now() + Duration::from_secs(60);
        assert!(matches!(
            wait_cancelled_or_expired(&cancel, deadline),
            Ok(false)
        ));

        cancel.store(true, Ordering::Release);
        assert!(matches!(
            wait_cancelled_or_expired(&cancel, deadline),
            Ok(true)
        ));

        cancel.store(false, Ordering::Release);
        assert!(matches!(
            wait_cancelled_or_expired(&cancel, Instant::now() - Duration::from_millis(1)),
            Err(AccountError::FlowExpired)
        ));
    }

    #[test]
    fn github_poll_schedule_starts_at_server_interval_and_slow_down_is_cumulative() {
        let mut interval = github_poll_interval(6);
        assert_eq!(interval, 6);
        interval = github_next_poll_interval(interval, true);
        assert_eq!(interval, 11);
        interval = github_next_poll_interval(interval, true);
        assert_eq!(interval, 16);
        assert_eq!(github_next_poll_interval(interval, false), 16);
        assert_eq!(github_poll_interval(0), 1);
    }

    proptest! {
        /// The poll schedule never asks faster than once a second, slows by
        /// exactly five seconds each time GitHub says so, and never wraps.
        #[test]
        fn prop_github_polling_never_speeds_up(start in any::<u64>(), slows in prop::collection::vec(any::<bool>(), 0..8)) {
            let mut interval = github_poll_interval(start);
            prop_assert!(interval >= 1);
            for slow in slows {
                let next = github_next_poll_interval(interval, slow);
                prop_assert!(next >= interval);
                prop_assert_eq!(next, if slow { interval.saturating_add(5) } else { interval });
                interval = next;
            }
        }

        /// A pending answer stays pending, whichever route asked and whether
        /// it must slow down; anything else is done.
        #[test]
        fn prop_a_poll_is_pending_only_when_the_server_says_pending(slow_down: bool) {
            let pending = Polled::sign_in(ProviderStep::Pending { slow_down });
            let Polled::Pending { slow_down: got } = pending else {
                return Err(TestCaseError::fail("a pending poll stays pending"));
            };
            prop_assert_eq!(got, slow_down);
            let linking = Polled::link(LinkOutcome::Pending { slow_down });
            let Polled::Pending { slow_down: got } = linking else {
                return Err(TestCaseError::fail("a pending link stays pending"));
            };
            prop_assert_eq!(got, slow_down);
            let done = Polled::link(LinkOutcome::Linked {
                provider: Provider::Github,
                created_at: 1,
            });
            prop_assert!(matches!(done, Polled::Done(ProviderDone::Link(_))));
        }
    }

    #[test]
    fn a_github_sign_in_with_a_new_identity_offers_a_username() {
        let server = Arc::new(MemoryServer::new());
        server.set_providers(vec![Provider::Github]);
        // A poll interval of a second: the wait is real, but short.
        server.set_github_device_flow("octocat", "ABCD-1234", 1);
        server.push_github_answer("authorized");
        let lab = Lab::new(&server, "Studio", 0);
        let browser = FakeBrowser::new(Err(BrowserError::Failed));
        let login = LoginDevice::ephemeral();
        let started = lab
            .flow
            .start_provider(&browser, SERVER, &Provider::Github, &login, &no_step)
            .unwrap();
        let ProviderStart::Github(github) = &started.start else {
            panic!("GitHub's start");
        };
        assert_eq!(github.user_code, "ABCD-1234");
        let result = lab
            .flow
            .finish_provider(
                &browser,
                SERVER,
                &Provider::Github,
                &login,
                started,
                None,
                not_cancelled(),
                &no_step,
            )
            .unwrap();
        let ProviderTaskResult::NewAccount(account) = result else {
            panic!("a GitHub identity no account has is offered a new one");
        };
        assert!(!account.ticket.is_empty());
        assert_eq!(browser.opened().len(), 1, "the device page opened once");
        assert!(browser.opened()[0].contains("github.com"));

        // The person chooses a name: the account is made, its recovery key
        // is the one thing to keep, and the library it would bind is read.
        let library = lab.flow.provider_sign_up_library(SERVER).unwrap();
        assert_eq!(library, LocalLibrary::Empty);
        let pending = ProviderSignup {
            server: SERVER.to_owned(),
            provider: Provider::Github,
            ticket: account.ticket,
            suggested_username: account.suggested_username,
            login_device: login,
        };
        let auth = lab
            .flow
            .create_provider_signup(SERVER, &pending, "octocat", library, &no_step)
            .unwrap();
        assert_eq!(auth.session.username, "octocat");
        assert!(auth.recovery_key.is_some());
        assert_eq!(auth.library_id, lab.flow.device().unwrap().library_id);
    }

    #[test]
    fn a_cancelled_provider_sign_in_ends_quietly_and_keeps_nothing() {
        let server = Arc::new(MemoryServer::new());
        server.set_providers(vec![Provider::Github]);
        server.set_github_device_flow("octocat", "ABCD-1234", 1);
        let lab = Lab::new(&server, "Studio", 0);
        let browser = FakeBrowser::new(Err(BrowserError::Failed));
        let login = LoginDevice::ephemeral();
        let started = lab
            .flow
            .start_provider(&browser, SERVER, &Provider::Github, &login, &no_step)
            .unwrap();
        let cancel = Arc::new(AtomicBool::new(true));
        let result = lab
            .flow
            .finish_provider(
                &browser,
                SERVER,
                &Provider::Github,
                &login,
                started,
                None,
                cancel,
                &no_step,
            )
            .unwrap();
        assert!(matches!(result, ProviderTaskResult::Cancelled));
        assert!(lab.secrets.items().is_empty());
    }

    /// An account on `server` that a provider can sign in to.
    fn alice(server: &Arc<MemoryServer>) -> String {
        let first = Lab::new(server, "Studio", 0);
        let (signed, _) = first
            .flow
            .create(SERVER, "alice", &password("alice"), &|_| {})
            .unwrap();
        signed.session.account_id
    }

    #[test]
    fn a_google_sign_in_keeps_the_session_locked_until_the_key_arrives() {
        let server = Arc::new(MemoryServer::new());
        let account_id = alice(&server);
        server.set_providers(vec![Provider::Google]);
        server.set_provider_outcomes(Provider::Google, [format!("session:{account_id}")]);
        let lab = Lab::new(&server, "Laptop", 0);
        let browser = FakeBrowser::new(Ok("the-code"));
        let login = LoginDevice::ephemeral();
        let started = lab
            .flow
            .start_provider(&browser, SERVER, &Provider::Google, &login, &no_step)
            .unwrap();
        let result = lab
            .flow
            .finish_provider(
                &browser,
                SERVER,
                &Provider::Google,
                &login,
                started,
                None,
                not_cancelled(),
                &no_step,
            )
            .unwrap();
        let ProviderTaskResult::Locked(locked, device) = result else {
            panic!("a known identity signs in, locked");
        };
        assert_eq!(locked.username, "alice");
        assert_eq!(browser.opened().len(), 1);
        // The token is durable before anyone is told: a crash can't lose it.
        let saved = keychain::load_locked_pending(lab.secrets.as_ref(), &device.library_id)
            .unwrap()
            .expect("the pending item");
        assert_eq!(saved.account_id, locked.account_id);
    }

    #[test]
    fn an_apple_sign_in_follows_the_callback_for_its_own_flow_only() {
        let server = Arc::new(MemoryServer::new());
        let account_id = alice(&server);
        server.set_providers(vec![Provider::Apple]);
        let lab = Lab::new(&server, "Laptop", 0);
        let browser = FakeBrowser::new(Err(BrowserError::Failed));
        let callback = |flow: &str| {
            let secret = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode([5u8; 32]);
            format!("kanso://auth/apple?flow={flow}&s={secret}")
        };
        let finish = |answer: Result<String, BrowserError>, outcomes: Vec<String>| {
            server.set_provider_outcomes(Provider::Apple, outcomes);
            let login = LoginDevice::ephemeral();
            let started = lab
                .flow
                .start_provider(&browser, SERVER, &Provider::Apple, &login, &no_step)
                .unwrap();
            let answer = answer.map(|_| callback(started.start.flow_id()));
            let waiting = Box::new(FakeApple(Mutex::new(Some(answer))));
            lab.flow.finish_provider(
                &browser,
                SERVER,
                &Provider::Apple,
                &login,
                started,
                Some(waiting),
                not_cancelled(),
                &no_step,
            )
        };
        // Its own flow's callback signs in.
        let result = finish(Ok(String::new()), vec![format!("session:{account_id}")]).unwrap();
        assert!(matches!(result, ProviderTaskResult::Locked(..)));
        // Dismissing the sheet is a denial, in plain words.
        let denied = finish(Err(BrowserError::Cancelled), vec!["pending".into()])
            .err()
            .expect("dismissed");
        assert_eq!(
            denied.words,
            words::error_words(&AccountError::ProviderDenied, ErrorContext::SignIn)
        );
        // A callback for another flow is no one's.
        let login = LoginDevice::ephemeral();
        let started = lab
            .flow
            .start_provider(&browser, SERVER, &Provider::Apple, &login, &no_step)
            .unwrap();
        let other = callback(&base64::engine::general_purpose::URL_SAFE_NO_PAD.encode([9u8; 32]));
        let denied = lab
            .flow
            .finish_provider(
                &browser,
                SERVER,
                &Provider::Apple,
                &login,
                started,
                Some(Box::new(FakeApple(Mutex::new(Some(Ok(other)))))),
                not_cancelled(),
                &no_step,
            )
            .err()
            .expect("another flow's callback");
        assert_eq!(
            denied.words,
            words::error_words(&AccountError::ProviderDenied, ErrorContext::SignIn)
        );
    }

    #[test]
    fn a_pending_provider_session_the_secret_store_refuses_fails_as_keychain_trouble() {
        let server = Arc::new(MemoryServer::new());
        let account_id = alice(&server);
        server.set_providers(vec![Provider::Google]);
        server.set_provider_outcomes(Provider::Google, [format!("session:{account_id}")]);
        let lab = Lab::new(&server, "Laptop", 0);
        lab.secrets.fail_writes_to(Some("sync-pending"));
        let browser = FakeBrowser::new(Ok("the-code"));
        let login = LoginDevice::ephemeral();
        let started = lab
            .flow
            .start_provider(&browser, SERVER, &Provider::Google, &login, &no_step)
            .unwrap();
        let failure = lab
            .flow
            .finish_provider(
                &browser,
                SERVER,
                &Provider::Google,
                &login,
                started,
                None,
                not_cancelled(),
                &no_step,
            )
            .err()
            .expect("the token can't be kept");
        assert!(failure.keychain_warning);
        assert!(failure.signed_out.is_none());
    }
}
