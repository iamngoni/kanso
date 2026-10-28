//! Sign-in, sign-up, provider and passkey flows, locked sessions, device approval as the new device, recovery and the binding question.
//!
//! This is the desktop's `account_flow` without the windows: the same engine
//! calls ([`kanso_engine::account::Flow`], `after_auth`, `after_unlock`) make
//! the same decisions, and the questions the desktop asks in a dialog come
//! back as [`AccountState::Question`], answered with
//! [`AccountHandle::answer_binding`]. Nothing here invents a rule the engine
//! has.
//!
//! Where the account stands is read from what is durable, so a restart finds
//! a locked sign-in where it was ([`crate::account_runtime::current`]). One
//! account task runs at a time; a task that changes the session stops a
//! running sync first.
//!
//! Secrets stay here. A recovery key the person must write down comes back
//! as text, and a provider flow's pages as URLs; no data key, session
//! token, PRF output or password wrap is ever handed to the shell.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use kanso_core::now_ms;
use kanso_engine::account::approval::{WaitEnd, WaitPhase, WaitUpdate, WaitingState};
use kanso_engine::account::provider::{ProviderSignup, ProviderTaskResult};
use kanso_engine::account::restore::{Restored, candidate_from_local};
use kanso_engine::account::words::{self, ErrorContext};
use kanso_engine::account::{
    Authenticated, Failure, Flow as AccountFlow, Next, NextUnlock, PendingKey, SOMETHING_WRONG,
    UnlockedCandidate, after_auth, after_unlock,
};
use kanso_engine::sync::{SyncState, forget_ended_session};
use kanso_sync::api::LoginDevice;
use kanso_sync::keychain;
use kanso_sync::provider::ProviderStart;
use kanso_sync::{
    AccountError, Choice, DeviceIdentity, Flow as SyncFlow, LockedSession, PasskeySignIn, Password,
    Plan, RecoveryKey, SignOutReason, SignedIn, SignupState, meta_keys,
};

use crate::account_model::*;
use crate::account_provider::{Events, PhoneBrowser, kind_of, open_apple, provider_of};
use crate::account_runtime::{
    Busy, Held, Out, ProviderRun, SignUpLater, begin, begin_new, binding_question, current,
    failure_error, key_text, locked_session, publish,
};
use crate::boundary::{guard, held};
use crate::errors::KansoError;
use crate::library::Inner;
use crate::passkey_bridge::{self, NativeCeremony};
use crate::passkey_flow::{self, Context, Ended, RpRule};
use crate::syncing::{SyncHandle, account_error};

/// Sign-in, sign-up, provider and passkey flows, locked sessions, device approval as the new device, recovery and the binding question.
#[derive(Clone)]
pub struct AccountHandle {
    pub(crate) inner: Arc<Inner>,
}

/// What a sign-up does once the person has said the notes may be uploaded,
/// if they have to: make the account and bind the library. The recovery key
/// it made, or `None` when the person dismissed the passkey sheet.
type Create =
    Box<dyn FnOnce(&AccountHandle, &AccountFlow) -> Result<Option<RecoveryKey>, KansoError> + Send>;

/// The key the device name is kept under, so a task that isn't given a name
/// (unlocking, a restart) still names this phone to the account.
const DEVICE_NAME: &str = "mobile.device_name";

impl AccountHandle {
    // ------------------------------------------------------------- state

    /// Where the account stands. A locked sign-in survives a restart.
    pub fn state(&self) -> Result<AccountState, KansoError> {
        guard(|| {
            self.start_restore();
            current(&self.inner)
        })
    }

    /// Calls `sink` with the state now and again each time it changes, until
    /// the sink returns `false`.
    pub fn state_stream(&self, mut sink: impl FnMut(AccountState) -> bool + Send + 'static) {
        self.start_restore();
        if let Ok(state) = current(&self.inner) {
            if !sink(state.clone()) {
                return;
            }
            *held(&self.inner.account.published) = Some(state);
        }
        self.inner.account.listeners.add(sink);
    }

    /// What the server offers this phone: the ways in, and whether it takes
    /// new accounts.
    pub fn server_methods(&self, server: String) -> Result<ServerMethods, KansoError> {
        guard(|| {
            let backend = self.inner.backend.as_ref();
            let server = server.trim();
            let auth = kanso_sync::server_auth(backend, server)
                .map_err(|err| account_error(&err, ErrorContext::Server))?;
            let signup = kanso_sync::server_info(backend, server)
                .map_err(|err| account_error(&err, ErrorContext::Server))?;
            let rule = RpRule::platform();
            Ok(ServerMethods {
                signups_open: signup == SignupState::Open,
                password: auth.password,
                passkeys: auth
                    .passkeys
                    .is_some_and(|passkeys| rule.allows(&passkeys.rp_id, server)),
                providers: auth.providers.iter().filter_map(kind_of).collect(),
            })
        })
    }

    // ---------------------------------------------------------- passwords

    /// Makes `username`'s account on `server` with `password`, signs this
    /// phone in and returns the recovery key to write down.
    ///
    /// When this phone's notes came from another account the desktop asks
    /// before it uploads them; so does this: the state comes back as
    /// [`AccountState::Question`] with the key empty, nothing has been sent,
    /// and [`AccountHandle::answer_binding`] with `Add` makes the account.
    /// The key is then [`AccountHandle::pending_recovery_key`].
    pub fn sign_up_password(
        &self,
        server: String,
        username: String,
        password: String,
        device_name: String,
    ) -> Result<SignedUp, KansoError> {
        guard(|| {
            let (name, password) = words::check_sign_up(&username, &password, &password)
                .map_err(KansoError::invalid)?;
            let busy = begin_new(&self.inner)?;
            let server = server.trim().to_owned();
            let at = server.clone();
            let create: Create = Box::new(move |this, flow| {
                let (made, _) = flow
                    .create(&at, &name, &password, &|_| {})
                    .map_err(|failure| this.settle(failure, ErrorContext::SignUp))?;
                Ok(Some(made.recovery_key))
            });
            self.sign_up_with(busy, &server, &device_name, create)
        })
    }

    /// Signs this phone in to `username`'s account with its password, naming
    /// it `device_name` to the account's other devices.
    ///
    /// A library with notes of its own that aren't this account's isn't
    /// bound until the person answers: the state comes back as
    /// [`AccountState::Question`].
    pub fn sign_in_password(
        &self,
        server: String,
        username: String,
        password: String,
        device_name: String,
    ) -> Result<AccountState, KansoError> {
        guard(|| {
            let password = Password::new(&password, &username)
                .map_err(|err| KansoError::invalid(format!("For the password, {err}.")))?;
            let _busy = begin_new(&self.inner)?;
            let sync = self.sync();
            let _stopped = sync.stop_sync();
            let flow = self.flow(&device_name);
            let device = flow
                .device()
                .map_err(|failure| self.settle(failure, ErrorContext::SignIn))?;
            let signed = kanso_sync::sign_in(
                self.inner.backend.as_ref(),
                server.trim(),
                &username,
                &password,
                &device,
                &|_| {},
            )
            .map_err(|err| account_error(&err, ErrorContext::SignIn))?;
            let auth = flow
                .authenticated(device.library_id, signed.session, signed.evicted, None)
                .map_err(|failure| self.settle(failure, ErrorContext::SignIn))?;
            self.settle_auth(&flow, SyncFlow::SignIn, auth)
        })
    }

    /// Unlocks a locked account with its password. The state comes back as
    /// the question, when this phone's notes need one, or signed in.
    pub fn unlock_with_password(&self, password: String) -> Result<AccountState, KansoError> {
        guard(|| {
            self.stop_approval();
            let _busy = begin_new(&self.inner)?;
            let locked = self.require_locked()?;
            let password = Password::new(&password, &locked.username).map_err(|err| {
                KansoError::invalid(words::error_words(
                    &AccountError::Password(err),
                    ErrorContext::SignIn,
                ))
            })?;
            let sync = self.sync();
            let _stopped = sync.stop_sync();
            let flow = self.flow("");
            let candidate = flow
                .unlock_with_password(&locked, &password, &|_| {})
                .map_err(|failure| self.settle(failure, ErrorContext::SignIn))?;
            self.settle_unlock(&flow, candidate)
        })
    }

    /// Sets a new password with the recovery key and signs in: from a locked
    /// session (its `server` and `username` are used; those given are
    /// ignored) or signed out. Returns the new recovery key, which replaces
    /// the one used; the old one stops working.
    pub fn recover(
        &self,
        server: String,
        username: String,
        recovery_key: String,
        new_password: String,
        device_name: String,
    ) -> Result<SignedUp, KansoError> {
        guard(|| {
            self.stop_approval();
            let _busy = begin_new(&self.inner)?;
            let locked = if self.sync().load_session()?.is_none() {
                locked_session(&self.inner)?
            } else {
                None
            };
            let sync = self.sync();
            let _stopped = sync.stop_sync();
            let flow = self.flow(&device_name);
            // A locked phone recovers its own account. The engine's
            // `recover_locked` adopts the recovered session into the locked
            // item, which fails for a sign-in made with a login device of
            // its own (every passkey and provider one) after the server has
            // rotated the keys. Recovery ends every earlier session, the
            // locked one included, so the phone recovers as a signed-out
            // one would, and the locked item goes.
            let (server, username) = match &locked {
                Some(locked) => (locked.server.clone(), locked.username.clone()),
                None => (server, username),
            };
            let (name, key, password) =
                words::check_recovery(&username, &recovery_key, &new_password, &new_password)
                    .map_err(KansoError::invalid)?;
            let at = server.trim().to_owned();
            let auth = flow
                .authenticate(
                    Box::new(move |s, device, step| {
                        s.recover_with_key(&at, &name, &key, &password, device, step)
                    }),
                    &|_| {},
                )
                .map_err(|failure| self.settle(failure, ErrorContext::Recovery))?;
            if let Some(locked) = &locked
                && let Err(failure) = flow.end_locked(locked)
            {
                eprintln!(
                    "kanso: ending the recovered locked sign-in: {}",
                    failure.words
                );
            }
            // A used recovery key is gone: the new one shows at once, even if
            // the binding question is cancelled.
            let shown = auth
                .recovery_key
                .as_ref()
                .map(|key| self.show_key(key))
                .unwrap_or_default();
            let state = self
                .settle_auth(&flow, SyncFlow::Recover, auth)
                .map_err(with_key_note)?;
            Ok(SignedUp {
                state,
                recovery_key: shown,
            })
        })
    }

    /// Answers the binding question the state asked: `Add` or `Replace`, or
    /// `None` to cancel (the new session is signed out and nothing changes
    /// here). An error when no question waits.
    pub fn answer_binding(
        &self,
        answer: Option<BindingAnswer>,
    ) -> Result<AccountState, KansoError> {
        guard(|| {
            let busy = begin(&self.inner)?;
            let taken = {
                let mut waiting = held(&self.inner.account.held);
                match (waiting.as_ref(), answer) {
                    (None, _) => {
                        return Err(KansoError::invalid("There is no question to answer."));
                    }
                    (Some(Held::SignUp { .. }), Some(BindingAnswer::Replace)) => {
                        return Err(KansoError::invalid(
                            "A new account has no notes to replace this phone's with.",
                        ));
                    }
                    _ => waiting.take(),
                }
            };
            let result = match taken {
                Some(Held::Auth { auth, .. }) => self.answer_auth(*auth, answer),
                Some(Held::Unlock { candidate, .. }) => self.answer_unlock(*candidate, answer),
                Some(Held::SignUp { later, .. }) => self.answer_sign_up(later, answer, busy),
                None => Err(KansoError::invalid("There is no question to answer.")),
            };
            publish(&self.inner);
            result
        })
    }

    // --------------------------------------------------------- providers

    /// Signs in with `provider`, naming this phone `device_name`. The whole
    /// flow runs on a thread of its own and its steps go to `sink` until the
    /// last one (`NeedsUsername`, `Finished`, `Failed` or `Cancelled`).
    /// One account task runs at a time.
    pub fn start_provider(
        &self,
        server: String,
        provider: ProviderKind,
        device_name: String,
        sink: impl FnMut(ProviderEvent) -> bool + Send + 'static,
    ) {
        let cancel = Arc::new(AtomicBool::new(false));
        let events = Events::new(sink, cancel.clone());
        let (this, thread_events) = (self.clone(), events.clone());
        let spawned = std::thread::Builder::new()
            .name("kanso-account".into())
            .spawn(move || {
                let end = match guard(|| {
                    Ok(this.provider_sign_in(
                        &server,
                        provider,
                        &device_name,
                        &thread_events,
                        &cancel,
                    ))
                }) {
                    Ok(event) => event,
                    Err(err) => ProviderEvent::Failed {
                        message: err.to_string(),
                    },
                };
                thread_events.close_page();
                thread_events.emit(end);
                publish(&this.inner);
            });
        if spawned.is_err() {
            events.emit(ProviderEvent::Failed {
                message: SOMETHING_WRONG.to_owned(),
            });
        }
    }

    /// The provider's sign-in session ended at a `kanso://` address (Apple),
    /// or was dismissed (`None`), which cancels the flow.
    pub fn provider_page_ended(&self, callback_url: Option<String>) {
        let running = held(&self.inner.account.provider);
        let Some(run) = running.as_ref() else {
            return;
        };
        match callback_url {
            Some(url) => {
                if let Some(apple) = &run.apple {
                    let _ = apple.send(url);
                }
            }
            None => run.cancel.store(true, Ordering::Release),
        }
    }

    /// Stops a provider flow, a link, or a passkey ceremony that is waiting
    /// for its sheet, and forgets a provider identity waiting for a username.
    pub fn cancel_provider(&self) {
        let account = &self.inner.account;
        if let Some(run) = held(&account.provider).as_ref() {
            run.cancel.store(true, Ordering::Release);
        }
        account.ceremony_cancel.store(true, Ordering::Release);
        *held(&account.signup) = None;
    }

    /// Makes the account a provider flow found no one for, with the
    /// `username` the person chose. Returns the recovery key to write down;
    /// a username that is taken fails and can be tried again.
    ///
    /// Like [`AccountHandle::sign_up_password`], it asks first when this
    /// phone's notes came from another account.
    pub fn finish_provider_sign_up(&self, username: String) -> Result<SignedUp, KansoError> {
        guard(|| {
            let pending: ProviderSignup = held(&self.inner.account.signup)
                .clone()
                .ok_or_else(|| KansoError::invalid("No sign-in is waiting for a username."))?;
            let busy = begin_new(&self.inner)?;
            let server = pending.server.clone();
            let create: Create = Box::new(move |this, flow| {
                let context = ErrorContext::SignUp;
                let library = flow
                    .provider_sign_up_library(&pending.server)
                    .map_err(|failure| this.settle(failure, context))?;
                let auth = flow
                    .create_provider_signup(
                        &pending.server,
                        &pending,
                        username.trim(),
                        library,
                        &|_| {},
                    )
                    .map_err(|failure| this.settle(failure, context))?;
                let key = auth.recovery_key.clone();
                this.bind_auth(flow, auth, Some(Choice::Add))?;
                *held(&this.inner.account.signup) = None;
                Ok(key)
            });
            self.sign_up_with(busy, &server, "", create)
        })
    }

    // ---------------------------------------------------------- passkeys

    /// Requests that the native passkey sheet must run, until the sink
    /// returns `false`. Each request carries a ticket; the platform's answer
    /// goes to Rust through `passkey_bridge::answer`, not through the shell.
    pub fn passkey_requests(&self, sink: impl FnMut(PasskeyRequest) -> bool + Send + 'static) {
        self.inner.account.passkey_sinks.add(sink);
    }

    /// The shell couldn't run the native ceremony for `ticket`.
    pub fn passkey_failed(&self, ticket: u64, failure: PasskeyFailure) {
        passkey_bridge::fail(ticket, failure);
    }

    /// Signs in with a passkey; blocks while the sheet is up. A dismissed
    /// sheet is not an error: the state comes back as it was. A passkey that
    /// can't unlock this phone leaves it [`AccountState::Locked`].
    pub fn passkey_sign_in(
        &self,
        server: String,
        device_name: String,
    ) -> Result<AccountState, KansoError> {
        guard(|| {
            let _busy = begin_new(&self.inner)?;
            self.inner
                .account
                .ceremony_cancel
                .store(false, Ordering::Release);
            let sync = self.sync();
            let _stopped = sync.stop_sync();
            let flow = self.flow(&device_name);
            let context = ErrorContext::Passkey;
            let device = flow
                .device()
                .map_err(|failure| self.settle(failure, context))?;
            let server = server.trim();
            let signed = {
                let ceremony = self.ceremony();
                let cx = self.passkey_context(server, &device);
                passkey_flow::sign_in(&cx, &ceremony)
            };
            let signed = match signed {
                Ok(signed) => signed,
                Err(error) => {
                    return self
                        .passkey_ended(error, context)
                        .and_then(|()| current(&self.inner));
                }
            };
            match signed {
                PasskeySignIn::Unlocked(SignedIn { session, evicted }) => {
                    let auth = flow
                        .authenticated(device.library_id, session, evicted, None)
                        .map_err(|failure| self.settle(failure, ErrorContext::SignIn))?;
                    self.settle_auth(&flow, SyncFlow::SignIn, auth)
                }
                PasskeySignIn::Locked(locked) => {
                    kanso_engine::account::provider::save_provider_pending(
                        self.inner.backend.as_ref(),
                        self.inner.secrets.as_ref(),
                        &locked,
                    )
                    .map_err(|err| self.settle(Failure::keychain(err), context))?;
                    self.entered_locked(&flow, locked, &device)
                }
            }
        })
    }

    /// Makes an account whose first way in is a new passkey, and signs this
    /// phone in; blocks while the sheet is up. A dismissed sheet is not an
    /// error: the state comes back as it was and the key is empty. Asks first
    /// when this phone's notes came from another account, as
    /// [`AccountHandle::sign_up_password`] does.
    pub fn passkey_sign_up(
        &self,
        server: String,
        username: String,
        device_name: String,
    ) -> Result<SignedUp, KansoError> {
        guard(|| {
            let name = kanso_sync::check_new_username(&username)
                .map_err(|err| KansoError::invalid(words::username_words(err)))?;
            let busy = begin_new(&self.inner)?;
            let server = server.trim().to_owned();
            let at = server.clone();
            let create: Create = Box::new(move |this, flow| this.passkey_create(flow, &at, &name));
            self.sign_up_with(busy, &server, &device_name, create)
        })
    }

    // ---------------------------------------------------- device approval

    /// Asks the account's other devices to approve this phone, or resumes the
    /// request it made earlier, and reports each step to `sink` on a thread
    /// of its own. When a device approves, this phone is unlocked and signed
    /// in (or asks its binding question). Dropping the stream cancels the
    /// request.
    pub fn request_approval(
        &self,
        device_name: String,
        sink: impl FnMut(ApprovalWait) -> bool + Send + 'static,
    ) {
        let out = Arc::new(Out::new(sink));
        let (this, thread_out) = (self.clone(), out.clone());
        let spawned = std::thread::Builder::new()
            .name("kanso-account".into())
            .spawn(move || {
                let ended = guard(|| {
                    this.wait_for_approval(&device_name, &thread_out);
                    Ok(())
                });
                if let Err(err) = ended {
                    thread_out.emit(failed_wait(&err.to_string()));
                }
                publish(&this.inner);
            });
        if spawned.is_err() {
            out.emit(failed_wait(SOMETHING_WRONG));
        }
    }

    /// Cancels the approval request being waited for, at the server too.
    pub fn cancel_approval(&self) {
        if let Some(cancel) = held(&self.inner.account.approval).as_ref() {
            cancel.store(true, Ordering::SeqCst);
        }
    }

    /// Gives up on a locked sign-in: its approval request, if it has one,
    /// and its pending session are ended, and the notes stay.
    pub fn cancel_locked(&self) -> Result<AccountState, KansoError> {
        guard(|| {
            self.stop_approval();
            if matches!(
                held(&self.inner.account.held).as_ref(),
                Some(Held::Unlock { .. })
            ) {
                return self.answer_binding(None);
            }
            let _busy = begin(&self.inner)?;
            if let Some(locked) = locked_session(&self.inner)? {
                let flow = self.flow("");
                if let Err(err) = flow.cancel_locked(&locked) {
                    eprintln!("kanso: cancelling the locked sign-in: {err}");
                }
            }
            publish(&self.inner);
            current(&self.inner)
        })
    }

    /// Test only: makes this phone's locked sign-in the way a provider
    /// sign-in leaves it, without a provider. Signs in to `username`'s
    /// account with `password`, then keeps only what a locked session holds
    /// (the token, no data key) as the pending item, so the state is
    /// [`AccountState::Locked`] and the phone has to be approved or recover.
    #[cfg(any(test, feature = "test-support"))]
    pub fn lock_for_test(
        &self,
        server: String,
        username: String,
        password: String,
        device_name: String,
    ) -> Result<AccountState, KansoError> {
        use kanso_sync::SessionToken;

        guard(|| {
            let password = Password::new(&password, &username)
                .map_err(|err| KansoError::invalid(format!("For the password, {err}.")))?;
            let _busy = begin_new(&self.inner)?;
            let flow = self.flow(&device_name);
            let device = flow
                .device()
                .map_err(|failure| self.settle(failure, ErrorContext::SignIn))?;
            let signed = kanso_sync::sign_in(
                self.inner.backend.as_ref(),
                server.trim(),
                &username,
                &password,
                &device,
                &|_| {},
            )
            .map_err(|err| account_error(&err, ErrorContext::SignIn))?;
            let session = signed.session;
            let token = SessionToken::new(
                session
                    .authorization()
                    .trim_start_matches("Bearer ")
                    .to_owned(),
            );
            let mut locked = LockedSession::new(
                session.server.clone(),
                session.origin.clone(),
                session.account_id.clone(),
                session.username.clone(),
                session.device_id.clone(),
                token,
                session.key_id(),
                "0000000000000000".to_owned(),
                None,
                None,
            )
            .with_has_password(true)
            .with_login_device(LoginDevice::for_identity(
                &device,
                &session.origin,
                &session.account_id,
            ));
            locked.set_library_id(&device.library_id);
            keychain::save_locked_pending(
                self.inner.secrets.as_ref(),
                &device.library_id,
                &locked,
            )?;
            publish(&self.inner);
            current(&self.inner)
        })
    }

    // ------------------------------------------------------ recovery key

    /// The recovery key shown last that hasn't been confirmed saved, for a
    /// screen that shows it again (after a failure, or a restart). A key the
    /// server may not hold is asked about first.
    pub fn pending_recovery_key(&self) -> Result<Option<RecoveryKeyText>, KansoError> {
        guard(|| {
            let Some(library_id) = self.inner.store.meta(meta_keys::LIBRARY_ID)? else {
                return Ok(None);
            };
            let Some(item) =
                keychain::load_any_recovery_key(self.inner.secrets.as_ref(), &library_id)?
            else {
                return Ok(None);
            };
            if item.unconfirmed {
                let shown = PendingKey::kept(item.clone(), words::KEY_UNCONFIRMED);
                match self.flow("").check_kept_key(&shown, Some(library_id)) {
                    Ok(false) => return Ok(None),
                    Ok(true) => {}
                    Err(err) => eprintln!("kanso: asking whether the server took the key: {err}"),
                }
            }
            Ok(Some(self.show_key(&item.recovery_key)))
        })
    }

    /// The person said they saved the recovery key shown last: its copy in
    /// the secret store goes, unless that holds another key by now.
    pub fn recovery_key_saved(&self) -> Result<(), KansoError> {
        guard(|| {
            let library_id = self.inner.store.meta(meta_keys::LIBRARY_ID)?;
            // The key this handle showed, or else the one kept in the secret
            // store (a password or key change in `AccountSettings` keeps its
            // key there and shows it itself).
            let saved = held(&self.inner.account.shown_key).take();
            let saved = match (saved, &library_id) {
                (Some(saved), _) => Some(saved),
                (None, Some(library_id)) => {
                    keychain::load_any_recovery_key(self.inner.secrets.as_ref(), library_id)?
                        .map(|kept| kept.recovery_key)
                }
                (None, None) => None,
            };
            if let Some(saved) = saved {
                self.flow("").forget_saved_key(library_id, &saved);
            }
            Ok(())
        })
    }
}

// The pieces the flows are made of.
impl AccountHandle {
    fn sync(&self) -> SyncHandle {
        SyncHandle {
            inner: self.inner.clone(),
        }
    }

    /// The engine's account tasks for this library. A name given is kept,
    /// and used again when a task is not given one.
    pub(crate) fn flow(&self, device_name: &str) -> AccountFlow {
        let given = device_name.trim();
        let name = if given.is_empty() {
            self.inner
                .store
                .meta(DEVICE_NAME)
                .ok()
                .flatten()
                .unwrap_or_default()
        } else {
            if self.inner.store.meta(DEVICE_NAME).ok().flatten().as_deref() != Some(given)
                && let Err(err) = self.inner.store.set_meta(DEVICE_NAME, given)
            {
                eprintln!("kanso: remembering the device name: {err}");
            }
            given.to_owned()
        };
        self.sync().flow(&name)
    }

    /// What a failed task is to the shell, after what the failure asks for:
    /// a session the server ended is forgotten.
    pub(crate) fn settle(&self, failure: Failure, context: ErrorContext) -> KansoError {
        if let Some(reason) = failure.signed_out {
            self.end_session(reason);
        }
        let error = failure_error(&failure, context);
        publish(&self.inner);
        error
    }

    /// The server ended this phone's session: it is signed out here too.
    fn end_session(&self, reason: SignOutReason) {
        let sync = self.sync();
        let (Ok(Some(session)), Ok(Some(library_id))) = (
            sync.load_session(),
            self.inner.store.meta(meta_keys::LIBRARY_ID),
        ) else {
            return;
        };
        forget_ended_session(
            self.inner.secrets.as_ref(),
            &library_id,
            reason,
            Some(&session),
            false,
        );
        *held(&self.inner.sync_state) = SyncState::Off;
        sync.notify();
    }

    /// A new account or sign-in is the library's now: sync starts over.
    fn signed_in_now(&self) {
        *held(&self.inner.sync_state) = SyncState::Off;
        self.sync().notify();
        publish(&self.inner);
        self.publish_identity();
    }

    /// Publishes this account's sharing identity, as the desktop does at
    /// sign-up, so others can share with it before its first sync. Off this
    /// thread and best effort: the next shared-notes sync tries again.
    fn publish_identity(&self) {
        let shared = crate::SharedNotes {
            inner: self.inner.clone(),
        };
        let spawned = std::thread::Builder::new()
            .name("kanso-identity".into())
            .spawn(move || {
                let published = crate::boundary::guard(|| {
                    shared
                        .engine()?
                        .identity()
                        .map(|_| ())
                        .map_err(|err| crate::sharing::collab_error(&err))
                });
                if let Err(err) = published {
                    eprintln!("kanso: publishing the sharing identity: {err}");
                }
            });
        if let Err(err) = spawned {
            eprintln!("kanso: couldn't start publishing the sharing identity: {err}");
        }
    }

    /// Remembers `key` as the one shown, and writes it down for the shell.
    fn show_key(&self, key: &RecoveryKey) -> RecoveryKeyText {
        *held(&self.inner.account.shown_key) = Some(key.clone());
        key_text(key)
    }

    /// The signed-in session, or [`KansoError::SignedOut`].
    pub(crate) fn require_session(&self) -> Result<kanso_sync::Session, KansoError> {
        self.sync().load_session()?.ok_or(KansoError::SignedOut)
    }

    fn require_locked(&self) -> Result<LockedSession, KansoError> {
        locked_session(&self.inner)?
            .ok_or_else(|| KansoError::invalid("This phone isn't waiting to be unlocked."))
    }

    pub(crate) fn ceremony(&self) -> NativeCeremony<'_> {
        NativeCeremony::new(
            &self.inner.account.passkey_sinks,
            &self.inner.account.ceremony_cancel,
        )
    }

    fn passkey_context<'a>(&'a self, server: &'a str, device: &'a DeviceIdentity) -> Context<'a> {
        Context {
            backend: self.inner.backend.as_ref(),
            secrets: self.inner.secrets.as_ref(),
            server,
            device,
            rule: RpRule::platform(),
            progress: &|_| {},
        }
    }

    /// A passkey task that ended without an answer: nothing for a dismissed
    /// sheet, the failure settled, or the shell's error.
    fn passkey_ended(
        &self,
        error: passkey_flow::PasskeyError,
        context: ErrorContext,
    ) -> Result<(), KansoError> {
        match passkey_flow::ended(error, context) {
            Ended::Cancelled => Ok(()),
            Ended::Failed(failure) => Err(self.settle(failure, context)),
            Ended::Error(error) => Err(error),
        }
    }

    /// The launch check, once: retries logouts the server hasn't heard, and
    /// opens a locked sign-in with a session this phone already holds.
    fn start_restore(&self) {
        if !self.inner.account.begin_restore() {
            return;
        }
        let this = self.clone();
        let spawned = std::thread::Builder::new()
            .name("kanso-account".into())
            .spawn(move || {
                let _ = guard(|| {
                    this.restore();
                    Ok(())
                });
                this.inner.account.end_restore();
                publish(&this.inner);
            });
        if spawned.is_err() {
            self.inner.account.end_restore();
        }
    }

    fn restore(&self) {
        let bound = |key| self.inner.store.meta(key).ok().flatten();
        let account = bound(meta_keys::ORIGIN).zip(bound(meta_keys::ACCOUNT_ID));
        let flow = self.flow("");
        let Some(Restored {
            library_id,
            loaded,
            pending,
            local,
            ..
        }) = flow.restore(
            account
                .as_ref()
                .map(|(origin, id)| (origin.as_str(), id.as_str())),
        )
        else {
            return;
        };
        // A session that agrees with the binding wins over a locked one.
        if !matches!(loaded, Ok(Some(_)))
            && let Ok(Some(locked)) = pending
            && let Some(candidate) = candidate_from_local(&locked, &library_id, local)
        {
            let sync = self.sync();
            let _stopped = sync.stop_sync();
            if let Err(err) = self.settle_unlock(&flow, candidate) {
                eprintln!("kanso: opening a locked sign-in with a saved key: {err}");
            }
        }
        flow.flush_logouts(&library_id);
    }

    // ------------------------------------------------- binding questions

    /// After an account is proved: the question when the library needs one,
    /// then the binding.
    fn settle_auth(
        &self,
        flow: &AccountFlow,
        kind: SyncFlow,
        auth: Authenticated,
    ) -> Result<AccountState, KansoError> {
        match after_auth(kind, &auth.library, &auth.session.username, true) {
            Next::Ask(question) => {
                let question = binding_question(question);
                *held(&self.inner.account.held) = Some(Held::Auth {
                    auth: Box::new(auth),
                    question: question.clone(),
                });
                publish(&self.inner);
                Ok(AccountState::Question(question))
            }
            Next::Abandon => {
                flow.abandon(&auth.session);
                Err(KansoError::invalid(
                    "This phone's library already has notes that aren't this account's. \
                     Sign in from an empty library.",
                ))
            }
            Next::Bind(choice) => {
                self.bind_auth(flow, auth, choice)?;
                current(&self.inner)
            }
        }
    }

    /// Binds the library (unless `choice` is `None`: it already is) and saves
    /// the session.
    fn bind_auth(
        &self,
        flow: &AccountFlow,
        auth: Authenticated,
        choice: Option<Choice>,
    ) -> Result<(), KansoError> {
        let Authenticated {
            session,
            library_id,
            recovery_key,
            ..
        } = auth;
        flow.bind_and_save(session, library_id, choice, recovery_key)
            .map_err(|failure| self.settle(failure, ErrorContext::SignIn))?;
        self.signed_in_now();
        Ok(())
    }

    /// After any unlock: the question when the library needs one, then the
    /// promotion.
    fn settle_unlock(
        &self,
        flow: &AccountFlow,
        candidate: UnlockedCandidate,
    ) -> Result<AccountState, KansoError> {
        let library = match kanso_sync::local_library(
            &self.inner.store,
            &candidate.session.origin,
            &candidate.session.account_id,
        ) {
            Ok(library) => library,
            Err(err) => {
                eprintln!("kanso: reading the library before an unlock: {err}");
                self.discard(flow, &candidate);
                return Err(KansoError::storage(kanso_engine::account::LIBRARY_TROUBLE));
            }
        };
        match after_unlock(&library, &candidate.session.username, true) {
            NextUnlock::Ask(question) => {
                let question = binding_question(question);
                *held(&self.inner.account.held) = Some(Held::Unlock {
                    candidate: Box::new(candidate),
                    question: question.clone(),
                });
                publish(&self.inner);
                Ok(AccountState::Question(question))
            }
            NextUnlock::Promote(choice) => {
                self.promote(flow, candidate, choice)?;
                current(&self.inner)
            }
            // A plan an unlock can't follow; the phone has no window to park
            // one for, and a question always comes with its words.
            NextUnlock::Park | NextUnlock::Reject => {
                self.discard(flow, &candidate);
                Err(KansoError::invalid(SOMETHING_WRONG))
            }
        }
    }

    fn promote(
        &self,
        flow: &AccountFlow,
        candidate: UnlockedCandidate,
        choice: Option<Choice>,
    ) -> Result<(), KansoError> {
        let UnlockedCandidate {
            session,
            locked_session,
            library_id,
            same_token_as_locked,
            ..
        } = candidate;
        flow.promote(
            session,
            library_id,
            same_token_as_locked,
            choice,
            Some(&locked_session),
        )
        .map_err(|failure| self.settle(failure, ErrorContext::SignIn))?;
        self.signed_in_now();
        Ok(())
    }

    /// Ends a candidate nobody will use: its sign-in and its session go.
    fn discard(&self, flow: &AccountFlow, candidate: &UnlockedCandidate) {
        if flow.cleanup_candidate(Some(&candidate.locked_session), candidate) {
            eprintln!("kanso: the secret store refused to forget an unlocked candidate");
        }
        publish(&self.inner);
    }

    fn answer_auth(
        &self,
        auth: Authenticated,
        answer: Option<BindingAnswer>,
    ) -> Result<AccountState, KansoError> {
        let flow = self.flow("");
        match answer {
            // The new session is signed out; the library's items stay.
            None => flow.abandon(&auth.session),
            Some(answer) => {
                let sync = self.sync();
                let _stopped = sync.stop_sync();
                self.bind_auth(&flow, auth, Some(choice_of(answer)))?;
            }
        }
        current(&self.inner)
    }

    fn answer_unlock(
        &self,
        candidate: UnlockedCandidate,
        answer: Option<BindingAnswer>,
    ) -> Result<AccountState, KansoError> {
        let flow = self.flow("");
        match answer {
            None => self.discard(&flow, &candidate),
            Some(answer) => {
                let sync = self.sync();
                let _stopped = sync.stop_sync();
                self.promote(&flow, candidate, Some(choice_of(answer)))?;
            }
        }
        current(&self.inner)
    }

    fn answer_sign_up(
        &self,
        later: SignUpLater,
        answer: Option<BindingAnswer>,
        busy: Busy,
    ) -> Result<AccountState, KansoError> {
        if answer.is_none() {
            return current(&self.inner);
        }
        Ok(later(busy)?.state)
    }

    // ---------------------------------------------------------- sign-ups

    /// A sign-up: asks first when this phone's notes came from another
    /// account, otherwise makes the account with `create`.
    fn sign_up_with(
        &self,
        busy: Busy,
        server: &str,
        device_name: &str,
        create: Create,
    ) -> Result<SignedUp, KansoError> {
        let flow = self.flow(device_name);
        let origin = kanso_sync::origin(server).unwrap_or_default();
        let library = flow
            .library(&origin, "")
            .map_err(|failure| self.settle(failure, ErrorContext::SignUp))?;
        if let Plan::AskBeforeSignUp { from, .. } = kanso_sync::plan(SyncFlow::SignUp, &library) {
            let (title, body, _) = words::upload_question(&from);
            let question = BindingQuestion {
                title,
                body,
                answers: vec![BindingAnswer::Add],
            };
            let (this, name) = (self.clone(), device_name.to_owned());
            *held(&self.inner.account.held) = Some(Held::SignUp {
                later: Box::new(move |busy| this.make_account(busy, &name, create)),
                question: question.clone(),
            });
            drop(busy);
            publish(&self.inner);
            return Ok(SignedUp {
                state: AccountState::Question(question),
                recovery_key: RecoveryKeyText::default(),
            });
        }
        self.make_account(busy, device_name, create)
    }

    /// Makes the account and binds the library, with sync stopped.
    fn make_account(
        &self,
        busy: Busy,
        device_name: &str,
        create: Create,
    ) -> Result<SignedUp, KansoError> {
        let _busy = busy;
        let sync = self.sync();
        let _stopped = sync.stop_sync();
        let flow = self.flow(device_name);
        let made = create(self, &flow)?;
        let recovery_key = made
            .as_ref()
            .map(|key| self.show_key(key))
            .unwrap_or_default();
        if made.is_some() {
            self.signed_in_now();
        }
        Ok(SignedUp {
            state: current(&self.inner)?,
            recovery_key,
        })
    }

    /// The passkey sign-up's account half. `None` when the sheet was
    /// dismissed.
    fn passkey_create(
        &self,
        flow: &AccountFlow,
        server: &str,
        username: &str,
    ) -> Result<Option<RecoveryKey>, KansoError> {
        self.inner
            .account
            .ceremony_cancel
            .store(false, Ordering::Release);
        let context = ErrorContext::SignUp;
        let device = flow
            .device()
            .map_err(|failure| self.settle(failure, context))?;
        let made = {
            let ceremony = self.ceremony();
            let cx = self.passkey_context(server, &device);
            passkey_flow::sign_up(&cx, username, &ceremony)
        };
        let (made, _can_unlock) = match made {
            Ok(made) => made,
            Err(error) => return self.passkey_ended(error, context).map(|()| None),
        };
        let kanso_sync::SignedUp {
            session,
            recovery_key,
        } = made;
        if let Err(failure) = flow.save_and_bind(&device.library_id, &session, Some(Choice::Add)) {
            // As after a password sign-up: the account and its key stay.
            kanso_sync::discard_session(
                self.inner.backend.as_ref(),
                self.inner.secrets.as_ref(),
                &session,
            )
            .ok();
            let failure =
                failure.after_key(words::CREATED_NOT_SIGNED_IN_PASSKEY, &session, recovery_key);
            return Err(self.settle(failure, context));
        }
        Ok(Some(recovery_key))
    }

    // ---------------------------------------------------------- providers

    /// A provider sign-in's whole flow, to its last event.
    fn provider_sign_in(
        &self,
        server: &str,
        kind: ProviderKind,
        device_name: &str,
        events: &Arc<Events>,
        cancel: &Arc<AtomicBool>,
    ) -> ProviderEvent {
        let context = ErrorContext::SignIn;
        let _busy = match begin_new(&self.inner) {
            Ok(busy) => busy,
            Err(err) => return failed_event(&err),
        };
        let _running = RunGuard::install(&self.inner, cancel.clone());
        *held(&self.inner.account.signup) = None;
        let server = server.trim();
        let provider = provider_of(kind);
        let flow = self.flow(device_name);
        let login = LoginDevice::ephemeral();
        let browser = PhoneBrowser {
            events: events.clone(),
        };
        let started = match flow.start_provider(&browser, server, &provider, &login, &|_| {}) {
            Ok(started) => started,
            Err(failure) => return failed_event(&self.settle(failure, context)),
        };
        let apple = match &started.start {
            ProviderStart::Apple(start) => {
                match open_apple(events, &self.inner.account.provider, start) {
                    Ok(wait) => Some(wait),
                    Err(err) => {
                        let failure = Failure::account(err, context);
                        return failed_event(&self.settle(failure, context));
                    }
                }
            }
            _ => None,
        };
        let finished = flow.finish_provider(
            &browser,
            server,
            &provider,
            &login,
            started,
            apple,
            cancel.clone(),
            &|_| {},
        );
        events.close_page();
        match finished {
            Err(failure) => failed_event(&self.settle(failure, context)),
            Ok(ProviderTaskResult::Cancelled) => ProviderEvent::Cancelled,
            Ok(ProviderTaskResult::NewAccount(account)) => {
                if cancel.load(Ordering::Acquire) {
                    return ProviderEvent::Cancelled;
                }
                *held(&self.inner.account.signup) = Some(ProviderSignup {
                    server: server.to_owned(),
                    provider,
                    ticket: account.ticket,
                    suggested_username: account.suggested_username.clone(),
                    login_device: login,
                });
                ProviderEvent::NeedsUsername {
                    suggested: account.suggested_username,
                }
            }
            Ok(ProviderTaskResult::Locked(locked, device)) => {
                if cancel.load(Ordering::Acquire) {
                    // Made durable on the way; nobody will use it.
                    flow.discard_locked(&locked);
                    return ProviderEvent::Cancelled;
                }
                let sync = self.sync();
                let _stopped = sync.stop_sync();
                match self.entered_locked(&flow, *locked, &device) {
                    Ok(state) => ProviderEvent::Finished(state),
                    Err(err) => failed_event(&err),
                }
            }
        }
    }

    /// A locked sign-in is saved: a session this phone already holds for the
    /// account may open it without asking anyone.
    fn entered_locked(
        &self,
        flow: &AccountFlow,
        locked: LockedSession,
        device: &DeviceIdentity,
    ) -> Result<AccountState, KansoError> {
        match flow.local_unlock(&locked, &device.library_id) {
            Some(candidate) => self.settle_unlock(flow, candidate),
            None => {
                publish(&self.inner);
                current(&self.inner)
            }
        }
    }

    // ---------------------------------------------------- device approval

    /// Ends a device approval wait that is running, and waits for it to
    /// finish, so a task that is about to change the locked sign-in doesn't
    /// race it.
    fn stop_approval(&self) {
        let Some(cancel) = held(&self.inner.account.approval).clone() else {
            return;
        };
        cancel.store(true, Ordering::SeqCst);
        for _ in 0..500 {
            if held(&self.inner.account.approval).is_none() {
                return;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    /// The approval wait, on its own thread.
    fn wait_for_approval(&self, device_name: &str, out: &Out<ApprovalWait>) {
        let cancel = {
            let mut waiting = held(&self.inner.account.approval);
            if waiting.is_some() {
                out.emit(failed_wait(kanso_engine::account::BUSY));
                return;
            }
            let cancel = Arc::new(AtomicBool::new(false));
            *waiting = Some(cancel.clone());
            cancel
        };
        let _waiting = ApprovalSlot(self.inner.clone());
        let locked = match self.require_locked() {
            Ok(locked) => locked,
            Err(err) => {
                out.emit(failed_wait(&err.to_string()));
                return;
            }
        };
        let flow = self.flow(device_name);
        let device = match flow.device() {
            Ok(device) => device,
            Err(failure) => {
                out.emit(failed_wait(&failure.words));
                return;
            }
        };
        let state = std::cell::RefCell::new(WaitingState::requesting(now_ms()));
        let last = std::cell::RefCell::new(wait_view(&state.borrow()));
        if !out.emit(last.borrow().clone()) {
            return;
        }
        let tell = |update: WaitUpdate| {
            let view = {
                let mut state = state.borrow_mut();
                state.apply(update, now_ms());
                wait_view(&state)
            };
            if view != *last.borrow() {
                *last.borrow_mut() = view.clone();
                if !out.emit(view) {
                    cancel.store(true, Ordering::SeqCst);
                }
            }
        };
        let result = flow.wait_for_approval(&locked, &device, Some(&*cancel), &tell, &|pause| {
            pause_unless(&cancel, pause);
        });
        let show = |phase: WaitPhase| {
            let view = {
                let mut state = state.borrow_mut();
                state.phase = phase;
                wait_view(&state)
            };
            out.emit(view);
        };
        match result {
            Ok(candidate) => {
                show(WaitPhase::Unlocking);
                match self.promote_approved(&flow, candidate) {
                    Ok(()) => show(WaitPhase::Approved),
                    Err(err) => show(WaitPhase::Failed(err.to_string())),
                }
            }
            Err(end) => {
                if let WaitEnd::Failed(failure) = &end
                    && failure.locked_cleanup
                    && let Err(err) = flow.end_locked(&locked)
                {
                    eprintln!(
                        "kanso: ending a locked sign-in that can't be used: {}",
                        err.words
                    );
                }
                let view = {
                    let mut state = state.borrow_mut();
                    state.end(&end);
                    wait_view(&state)
                };
                out.emit(view);
            }
        }
    }

    /// A device approved: unlock this phone, as every unlock does.
    fn promote_approved(
        &self,
        flow: &AccountFlow,
        candidate: UnlockedCandidate,
    ) -> Result<(), KansoError> {
        // Another task may hold the slot for a moment.
        let mut waited = 0;
        let _busy = loop {
            match begin(&self.inner) {
                Ok(busy) => break busy,
                Err(err) if waited >= 200 => {
                    self.discard(flow, &candidate);
                    return Err(err);
                }
                Err(_) => {
                    waited += 1;
                    std::thread::sleep(Duration::from_millis(100));
                }
            }
        };
        let sync = self.sync();
        let _stopped = sync.stop_sync();
        self.settle_unlock(flow, candidate).map(|_| ())
    }
}

/// The provider flow that is running, registered for as long as this lives.
pub(crate) struct RunGuard(Arc<Inner>);

impl RunGuard {
    pub(crate) fn install(inner: &Arc<Inner>, cancel: Arc<AtomicBool>) -> RunGuard {
        *held(&inner.account.provider) = Some(ProviderRun {
            cancel,
            apple: None,
        });
        RunGuard(inner.clone())
    }
}

impl Drop for RunGuard {
    fn drop(&mut self) {
        *held(&self.0.account.provider) = None;
    }
}

/// The approval wait that is running, registered for as long as this lives.
struct ApprovalSlot(Arc<Inner>);

impl Drop for ApprovalSlot {
    fn drop(&mut self) {
        *held(&self.0.account.approval) = None;
    }
}

/// Sleeps `pause` in short slices, ending early once `cancel` is set.
fn pause_unless(cancel: &AtomicBool, pause: Duration) {
    let end = std::time::Instant::now() + pause;
    while !cancel.load(Ordering::SeqCst) {
        let left = end.saturating_duration_since(std::time::Instant::now());
        if left.is_zero() {
            return;
        }
        std::thread::sleep(left.min(Duration::from_millis(50)));
    }
}

/// What the waiting page shows of the engine's state.
pub(crate) fn wait_view(state: &WaitingState) -> ApprovalWait {
    ApprovalWait {
        phase: match &state.phase {
            WaitPhase::Requesting => ApprovalPhase::Requesting,
            WaitPhase::Waiting => ApprovalPhase::Waiting,
            WaitPhase::Compare { .. } => ApprovalPhase::Compare,
            WaitPhase::Unlocking => ApprovalPhase::Unlocking,
            WaitPhase::Approved => ApprovalPhase::Approved,
            WaitPhase::NoDevices => ApprovalPhase::NoDevices,
            WaitPhase::Cancelled => ApprovalPhase::Cancelled,
            WaitPhase::Expired => ApprovalPhase::Expired,
            WaitPhase::Failed(message) => ApprovalPhase::Failed {
                message: message.clone(),
            },
        },
        code: match &state.phase {
            WaitPhase::Compare { code } => Some(code.clone()),
            _ => state.code.clone(),
        },
        other_devices: u32::try_from(state.other_devices).unwrap_or(u32::MAX),
        offer_recovery: state.recovery_offered,
    }
}

fn failed_wait(message: &str) -> ApprovalWait {
    ApprovalWait {
        phase: ApprovalPhase::Failed {
            message: message.to_owned(),
        },
        code: None,
        other_devices: 0,
        offer_recovery: true,
    }
}

fn failed_event(err: &KansoError) -> ProviderEvent {
    ProviderEvent::Failed {
        message: err.to_string(),
    }
}

fn choice_of(answer: BindingAnswer) -> Choice {
    match answer {
        BindingAnswer::Add => Choice::Add,
        BindingAnswer::Replace => Choice::Replace,
    }
}

/// A failure after a recovery key was made: the key is saved on this
/// device, and the words say so.
fn with_key_note(err: KansoError) -> KansoError {
    const NOTE: &str = " The new recovery key is saved on this device.";
    match err {
        KansoError::Invalid { message } => KansoError::Invalid {
            message: message + NOTE,
        },
        KansoError::Storage { message } => KansoError::Storage {
            message: message + NOTE,
        },
        KansoError::Network { message } => KansoError::Network {
            message: message + NOTE,
        },
        KansoError::Auth { message } => KansoError::Auth {
            message: message + NOTE,
        },
        other => other,
    }
}
