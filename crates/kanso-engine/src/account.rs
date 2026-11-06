//! The account tasks without a window (docs/ACCOUNTS.md › Account flows, ›
//! Library binding).
//!
//! Every account task has two halves. The worker half is a blocking method
//! of [`Flow`], run on a thread of the caller's: it reaches the server, the
//! secret store and the library through the seams [`Flow`] holds, reports
//! its steps through a callback, and returns a result or a [`Failure`]. The
//! finishing half, on the caller's main thread, shows the result: the
//! windows, notifications and questions are the apps'. The decisions between
//! them that need no window ([`after_auth`], [`after_unlock`],
//! [`details_from`]) are here too, so the desktop and the mobile app make
//! the same ones.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use kanso_store::Store;
use kanso_sync::keychain::{self, PendingRecoveryKey};
use kanso_sync::{
    AccountError, Backend, BindError, Choice, DeviceIdentity, DeviceInfo, LocalLibrary,
    LockedSession, Machine, Password, Plan, RecoveryKey, RotationError, SecretError, Session,
    SignOutReason, SignedUp, Step, Usage,
};

use crate::Secrets;
use words::{ErrorContext, Question};

pub mod approval;
#[cfg(test)]
mod lab;
pub mod provider;
pub mod restore;
pub mod words;

pub use approval::{WaitEnd, WaitUpdate};
pub use provider::{ProviderDone, ProviderRoute, ProviderTaskResult, StartedProviderFlow};
pub use restore::Restored;

/// Whether `server` is the compiled-in default, however it is written, or
/// the same server under its first name.
pub fn is_default_server(server: &str) -> bool {
    kanso_sync::check_endpoint(server).is_ok_and(|server| kanso_sync::is_owner_server(&server))
}

/// When the library itself couldn't be changed.
pub const LIBRARY_TROUBLE: &str = "Kanso couldn't update this device's library. Check there is free space on this device, then try again.";
/// When a task's thread stopped without an answer.
pub const SOMETHING_WRONG: &str = "Something went wrong. Try again.";
/// When another account task is still running.
pub const BUSY: &str = "Kanso is still busy with your account. Try again in a moment.";

/// Why a task failed: the words to show, the reason when the server ended
/// this Mac's session on the way, and a new recovery key the server took
/// or may have taken, which shows all the same.
#[derive(Debug)]
pub struct Failure {
    pub words: String,
    pub signed_out: Option<SignOutReason>,
    pub recovery: Option<Box<PendingKey>>,
    /// The backup a Replace made before it failed: said all the same.
    pub backup: Option<PathBuf>,
    /// A locked candidate was successfully discarded and its pending item
    /// cleared after a bind failure.
    pub locked_cleanup: bool,
    /// Keychain cleanup or rollback failed; show the established warning.
    pub keychain_warning: bool,
}

impl Failure {
    pub fn account(err: AccountError, context: ErrorContext) -> Failure {
        // The details go to the log; the words never repeat them.
        eprintln!("kanso: account: {err}");
        let signed_out = match &err {
            AccountError::Unauthorized { reason } => Some(*reason),
            _ => None,
        };
        let keychain_warning = matches!(&err, AccountError::Keychain(_));
        Failure {
            words: words::error_words(&err, context),
            signed_out,
            recovery: None,
            backup: None,
            locked_cleanup: false,
            keychain_warning,
        }
    }

    /// A call that sets a new recovery key failed. The key it returns
    /// shows, with `note` when Kanso couldn't tell whether the server took
    /// it.
    pub fn rotation(err: RotationError, context: ErrorContext, note: &'static str) -> Failure {
        Failure {
            recovery: err
                .recovery_key
                .map(|item| Box::new(PendingKey::kept(*item, note))),
            ..Failure::account(err.error, context)
        }
    }

    /// The server took `key`, then this Mac failed: `words` say what
    /// happened instead, and the key shows with them.
    pub fn after_key(self, words: &'static str, session: &Session, key: RecoveryKey) -> Failure {
        Failure {
            words: words.to_owned(),
            recovery: Some(Box::new(PendingKey {
                key,
                username: session.username.clone(),
                server: session.server.clone(),
                note: Some(words),
            })),
            ..self
        }
    }

    pub fn keychain(err: SecretError) -> Failure {
        let mut failure = Failure::account(AccountError::Keychain(err), ErrorContext::Account);
        failure.keychain_warning = true;
        failure
    }

    pub fn library(err: impl std::fmt::Display) -> Failure {
        eprintln!("kanso: account: the library: {err}");
        Failure::words(LIBRARY_TROUBLE)
    }

    /// A failure that is only its words.
    pub fn words(words: &str) -> Failure {
        Failure {
            words: words.to_owned(),
            signed_out: None,
            recovery: None,
            backup: None,
            locked_cleanup: false,
            keychain_warning: false,
        }
    }

    pub fn bind(err: BindError) -> Failure {
        match err {
            BindError::Store(err) => Failure::library(err),
            BindError::Keychain(err) => Failure::keychain(err),
        }
    }
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

/// What the Account page shows of a fetch of the storage and the devices:
/// the details, or the reason the server ended this device's session (a 401
/// on either answer outranks anything else). Devices come out sorted, this
/// device first.
pub fn details_from(
    usage: Result<Usage, AccountError>,
    devices: Result<Vec<DeviceInfo>, AccountError>,
) -> Result<AccountDetails, SignOutReason> {
    let ended = [usage.as_ref().err(), devices.as_ref().err()]
        .into_iter()
        .flatten()
        .find_map(|err| match err {
            AccountError::Unauthorized { reason } => Some(*reason),
            _ => None,
        });
    if let Some(reason) = ended {
        return Err(reason);
    }
    let failed = [usage.as_ref().err(), devices.as_ref().err()]
        .into_iter()
        .flatten()
        .next()
        .map(|err| words::error_words(err, ErrorContext::Account));
    let mut details = AccountDetails {
        usage: usage.ok(),
        devices: devices.ok(),
        loading: false,
        stale: false,
        failed,
    };
    if let Some(devices) = &mut details.devices {
        words::sort_devices(devices);
    }
    Ok(details)
}

/// The signed-in account's ways in, as last fetched. A newer refresh hides
/// the retained success from the page until it has answered.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AccountMethodsState {
    pub last_successful: Option<kanso_sync::AccountMethods>,
    pub loading: bool,
    pub failed: Option<String>,
    /// Rejects answers from an earlier Account page visit.
    pub generation: u64,
    /// The server's passkeys, as its health last advertised them: which
    /// ceremony this device may run to add or prove one.
    pub passkeys: Option<kanso_sync::PasskeyMethods>,
}

impl AccountMethodsState {
    /// Starts a refresh and makes every earlier answer stale.
    pub fn begin_refresh(&mut self) -> u64 {
        self.generation = self.generation.wrapping_add(1);
        self.loading = true;
        self.failed = None;
        self.generation
    }

    /// Keeps the previous success available internally if the latest fetch
    /// fails; the page uses `failed` to avoid presenting it as current.
    pub fn finish_failure(&mut self, generation: u64, reason: String) -> bool {
        if self.generation != generation {
            return false;
        }
        self.loading = false;
        self.failed = Some(reason);
        true
    }

    pub fn finish_success(&mut self, generation: u64, methods: kanso_sync::AccountMethods) -> bool {
        if self.generation != generation {
            return false;
        }
        self.last_successful = Some(methods);
        self.loading = false;
        self.failed = None;
        true
    }
}

/// Whether an answer to the forms' health request still belongs to the
/// request the form is waiting on: the same server shown, the same
/// generation and the same attempt.
pub fn auth_answer_is_current(
    server: &str,
    generation: u64,
    shown: &str,
    current_generation: u64,
    attempted: Option<&(String, u64)>,
    request: &(String, u64),
) -> bool {
    shown == server
        && current_generation == generation
        && attempted == Some(request)
        && request == &(server.to_owned(), generation)
}

/// What the account tasks reach the world through, taken from the app's
/// services on its main thread.
#[derive(Clone)]
pub struct Flow {
    pub backend: Arc<dyn Backend>,
    pub secrets: Arc<dyn Secrets>,
    pub store: Arc<Store>,
    pub machine: Option<Machine>,
    pub backups: PathBuf,
    /// Now, in milliseconds, for the binding's backup name.
    pub now_ms: i64,
}

/// An account this Mac has proved it may use, before anything is saved.
pub struct Authenticated {
    pub session: Session,
    pub evicted: Option<DeviceInfo>,
    pub library: LocalLibrary,
    pub library_id: String,
    /// A recovery's new key, in the Keychain already.
    pub recovery_key: Option<RecoveryKey>,
}

/// An unlocked candidate returned to the UI thread before any library bind,
/// config write, Keychain promotion, or sync start is allowed.
pub struct UnlockedCandidate {
    pub session: Session,
    pub locked_session: LockedSession,
    pub evicted: Option<DeviceInfo>,
    pub library_id: String,
    /// Local-key and approval unlocks reuse the provider token in the
    /// pending item. Password unlock and recovery create a new token that
    /// must be revoked if promotion cannot persist it.
    pub same_token_as_locked: bool,
}

/// The session, who it replaced, and the recovery key a sign-in produced.
pub type Signed = (Session, Option<DeviceInfo>, Option<RecoveryKey>);

/// How a sign-in proves itself: given the seams, this device and the step
/// callback, it gives the session.
pub type Authenticate =
    Box<dyn FnOnce(&Flow, &DeviceIdentity, &dyn Fn(Step)) -> Result<Signed, Failure> + Send>;

/// What binding gives: the session, the library's ID, and a Replace's
/// backup.
pub type Bound = (Session, String, Option<PathBuf>);

/// What to do once an account is authenticated and the library it would
/// bind is known.
#[derive(Debug)]
pub enum Next {
    /// Ask this question first; its answer chooses the binding.
    Ask(Question),
    /// Nothing to ask and no one to ask: log the new session out and leave
    /// the library as it is.
    Abandon,
    /// Bind now: `None` keeps the binding the library already has.
    Bind(Option<Choice>),
}

/// What to do after a sign-in, sign-up or recovery for `flow`, given the
/// library and whether a window can ask. A choice is never made for the
/// person: a question that needs asking, with no one to ask, abandons.
pub fn after_auth(
    flow: kanso_sync::Flow,
    library: &LocalLibrary,
    username: &str,
    can_ask: bool,
) -> Next {
    let plan = kanso_sync::plan(flow, library);
    let question = words::binding_question(&plan, username);
    match (plan, question) {
        (Plan::Ask { .. }, Some(question)) if can_ask => Next::Ask(question),
        (Plan::Ask { .. }, _) => Next::Abandon,
        (Plan::Resume, _) => Next::Bind(None),
        (Plan::Fresh | Plan::AskBeforeSignUp { .. }, _) => Next::Bind(Some(Choice::Add)),
    }
}

/// What to do with an unlocked candidate, given the library and whether a
/// window can ask.
#[derive(Debug)]
pub enum NextUnlock {
    /// Ask this question first; its answer chooses the binding.
    Ask(Question),
    /// Keep the pending session and the candidate for the Account page:
    /// the question needs a window and none is there (the launch check).
    Park,
    /// Promote now: `None` keeps the binding the library already has.
    Promote(Option<Choice>),
    /// Not a plan an unlock can follow: clean the candidate up.
    Reject,
}

/// What to do after any unlock path, for the library the candidate's
/// account would bind. Never picks a choice unasked.
pub fn after_unlock(library: &LocalLibrary, username: &str, can_ask: bool) -> NextUnlock {
    let plan = kanso_sync::plan(kanso_sync::Flow::SignIn, library);
    let question = words::binding_question(&plan, username);
    match (plan, question) {
        (Plan::Ask { .. }, Some(question)) if can_ask => NextUnlock::Ask(question),
        (Plan::Ask { .. }, _) => NextUnlock::Park,
        (Plan::Resume, _) => NextUnlock::Promote(None),
        (Plan::Fresh, _) => NextUnlock::Promote(Some(Choice::Add)),
        (Plan::AskBeforeSignUp { .. }, _) => NextUnlock::Reject,
    }
}

/// A password unlock or recovery yields a fresh token; local-key and
/// approval unlocks reuse the locked token and must leave it usable if
/// promotion fails.
pub fn discard_fresh_candidate(
    backend: &dyn Backend,
    secrets: &dyn Secrets,
    session: &Session,
    same_token_as_locked: bool,
) -> bool {
    if same_token_as_locked {
        return false;
    }
    match kanso_sync::discard_session(backend, secrets, session) {
        Ok(()) => false,
        Err(error) => {
            eprintln!("kanso: discarding unpromoted session: {error}");
            matches!(&error, AccountError::Keychain(_))
        }
    }
}

// no-prop-test: the account tasks' workers, each a sequence of calls to
// kanso_sync and the secret store; tested end to end against MemoryServer and
// MemorySecrets, and the decisions between them by property.
impl Flow {
    /// This library's identity on this device (on the task's thread:
    /// reading a Mac runs `ioreg` and `scutil`).
    pub fn device(&self) -> Result<DeviceIdentity, Failure> {
        let machine = self.machine.clone().unwrap_or_else(Machine::this_computer);
        kanso_sync::device_identity(&self.store, &machine).map_err(Failure::library)
    }

    /// The library as it stands for the account of `origin` and
    /// `account_id` (empty for a new account, which is never the
    /// library's own).
    pub fn library(&self, origin: &str, account_id: &str) -> Result<LocalLibrary, Failure> {
        kanso_sync::local_library(&self.store, origin, account_id).map_err(Failure::library)
    }

    /// Saves `session` as the library's Keychain item, then binds the
    /// library with `choice` (A1, sync 7): a library is never bound to an
    /// account whose item is missing. When binding fails the item is put
    /// back as it was (gone, or another account's), and a backup a Replace
    /// made first comes back with the failure. `None` leaves the binding as
    /// it is.
    pub fn save_and_bind(
        &self,
        library_id: &str,
        session: &Session,
        choice: Option<Choice>,
    ) -> Result<Option<PathBuf>, Failure> {
        let item = keychain::account_item(library_id);
        let before = self.secrets.get(&item).map_err(Failure::keychain)?;
        keychain::save_session(self.secrets.as_ref(), library_id, session)
            .map_err(Failure::keychain)?;
        let Some(choice) = choice else {
            return Ok(None);
        };
        let backups_before = backups_in(&self.backups);
        match kanso_sync::bind(&self.store, session, choice, &self.backups, self.now_ms) {
            Ok(backup) => Ok(backup),
            Err(err) => {
                let put_back = match &before {
                    Some(raw) => self.secrets.set(&item, raw),
                    None => self.secrets.delete(&item),
                };
                if let Err(err) = put_back {
                    eprintln!("kanso: putting the library's Keychain item back: {err}");
                }
                let mut failure = Failure::bind(err);
                if choice == Choice::Replace {
                    failure.backup = backups_in(&self.backups)
                        .into_iter()
                        .find(|path| !backups_before.contains(path));
                }
                Err(failure)
            }
        }
    }

    /// Asks the server whether it took `shown`, a key Kanso couldn't tell it
    /// took (review-verify 6), and settles the Keychain's copy to match.
    pub fn check_kept_key(
        &self,
        shown: &PendingKey,
        library_id: Option<String>,
    ) -> Result<bool, AccountError> {
        let held = kanso_sync::recovery_key_held(
            self.backend.as_ref(),
            &shown.server,
            &shown.username,
            &shown.key,
        )?;
        self.settle_kept_key(library_id, &shown.key, held);
        Ok(held)
    }

    /// The Keychain's unsaved key once the server answered about `key`:
    /// confirmed when the server holds it, gone when it doesn't. An item that
    /// holds another key by then, or is confirmed already, stays as it is.
    pub fn settle_kept_key(&self, library_id: Option<String>, key: &RecoveryKey, held: bool) {
        let library_id = match library_id {
            Some(library_id) => library_id,
            None => match self.device() {
                Ok(device) => device.library_id,
                Err(_) => return,
            },
        };
        let secrets = self.secrets.as_ref();
        let settled =
            keychain::load_any_recovery_key(secrets, &library_id).and_then(|kept| match kept {
                Some(mut kept) if kept.unconfirmed && kept.recovery_key == *key => {
                    if held {
                        kept.unconfirmed = false;
                        keychain::save_recovery_key(secrets, &library_id, &kept)
                    } else {
                        keychain::delete_recovery_key(secrets, &library_id)
                    }
                }
                _ => Ok(()),
            });
        if let Err(err) = settled {
            eprintln!("kanso: settling the unconfirmed recovery key: {err}");
        }
    }

    /// "I've saved my recovery key": the Keychain's copy goes, unless the
    /// item holds another key by then.
    pub fn forget_saved_key(&self, library_id: Option<String>, saved: &RecoveryKey) {
        // Before the launch check, as after a failed sign-up in tests.
        let library_id = match library_id {
            Some(library_id) => library_id,
            None => match self.device() {
                Ok(device) => device.library_id,
                Err(_) => return,
            },
        };
        let secrets = self.secrets.as_ref();
        let removed =
            keychain::load_any_recovery_key(secrets, &library_id).and_then(|kept| match kept {
                Some(kept) if kept.recovery_key == *saved => {
                    keychain::delete_recovery_key(secrets, &library_id)
                }
                _ => Ok(()),
            });
        if let Err(err) = removed {
            eprintln!("kanso: removing the saved recovery key: {err}");
        }
    }

    /// Creates an account on `server` and binds this device's library to it
    /// as a fresh sign-up: the session, with the recovery key, and the
    /// library's ID.
    pub fn create(
        &self,
        server: &str,
        username: &str,
        password: &Password,
        step: &dyn Fn(Step),
    ) -> Result<(SignedUp, String), Failure> {
        let device = self.device()?;
        // The new key is in the Keychain before the request goes.
        let signed = kanso_sync::sign_up(
            self.backend.as_ref(),
            self.secrets.as_ref(),
            server,
            username,
            password,
            &device,
            step,
        )
        .map_err(|err| {
            Failure::rotation(err, ErrorContext::SignUp, words::KEY_UNCONFIRMED_SIGN_UP)
        })?;
        step(Step::Saving);
        let bound = self.save_and_bind(&device.library_id, &signed.session, Some(Choice::Add));
        if let Err(failure) = bound {
            // Nothing here uses the new session: the server forgets it.
            // The account stays, and so does its recovery key, in the
            // Keychain and on screen.
            kanso_sync::discard_session(
                self.backend.as_ref(),
                self.secrets.as_ref(),
                &signed.session,
            )
            .ok();
            let SignedUp {
                session,
                recovery_key,
            } = signed;
            return Err(failure.after_key(words::CREATED_NOT_SIGNED_IN, &session, recovery_key));
        }
        Ok((signed, device.library_id))
    }

    /// Signs in with the account's password.
    pub fn sign_in_password(
        &self,
        server: &str,
        username: &str,
        password: &Password,
        device: &DeviceIdentity,
        step: &dyn Fn(Step),
    ) -> Result<Signed, Failure> {
        let signed = kanso_sync::sign_in(
            self.backend.as_ref(),
            server,
            username,
            password,
            device,
            step,
        )
        .map_err(|err| Failure::account(err, ErrorContext::SignIn))?;
        Ok((signed.session, signed.evicted, None))
    }

    /// Sets a new password with the recovery key and signs in.
    pub fn recover_with_key(
        &self,
        server: &str,
        username: &str,
        key: &RecoveryKey,
        password: &Password,
        device: &DeviceIdentity,
        step: &dyn Fn(Step),
    ) -> Result<Signed, Failure> {
        // The new key is in the Keychain before the request goes.
        let recovered = kanso_sync::recover(
            self.backend.as_ref(),
            self.secrets.as_ref(),
            server,
            username,
            key,
            password,
            device,
            step,
        )
        .map_err(|err| {
            Failure::rotation(err, ErrorContext::Recovery, words::KEY_UNCONFIRMED_PASSWORD)
        })?;
        Ok((recovered.session, None, Some(recovered.recovery_key)))
    }

    /// Proves the account with `auth`, then reads the library it would
    /// bind.
    pub fn authenticate(
        &self,
        auth: Authenticate,
        step: &dyn Fn(Step),
    ) -> Result<Authenticated, Failure> {
        let device = self.device()?;
        let (session, evicted, recovery_key) = auth(self, &device, step)?;
        self.authenticated(device.library_id, session, evicted, recovery_key)
    }

    /// After a sign-in: the library the account would bind, or the failure
    /// (the new session is logged out).
    pub fn authenticated(
        &self,
        library_id: String,
        session: Session,
        evicted: Option<DeviceInfo>,
        recovery_key: Option<RecoveryKey>,
    ) -> Result<Authenticated, Failure> {
        let library = match self.library(&session.origin, &session.account_id) {
            Ok(library) => library,
            Err(failure) => {
                kanso_sync::discard_session(self.backend.as_ref(), self.secrets.as_ref(), &session)
                    .ok();
                return Err(match recovery_key {
                    Some(key) => failure.after_key(words::CHANGED_NOT_SIGNED_IN, &session, key),
                    None => failure,
                });
            }
        };
        Ok(Authenticated {
            session,
            evicted,
            library,
            library_id,
            recovery_key,
        })
    }

    /// Binds the library (unless `choice` is `None`: it already is) and
    /// saves the Keychain item. When it fails, nothing uses the new session
    /// and the library's Keychain items are as they were: maybe another
    /// account's.
    pub fn bind_and_save(
        &self,
        session: Session,
        library_id: String,
        choice: Option<Choice>,
        recovery_key: Option<RecoveryKey>,
    ) -> Result<Bound, Failure> {
        match self.save_and_bind(&library_id, &session, choice) {
            Ok(backup) => Ok((session, library_id, backup)),
            Err(failure) => {
                kanso_sync::discard_session(self.backend.as_ref(), self.secrets.as_ref(), &session)
                    .ok();
                // The password changed all the same.
                Err(match recovery_key {
                    Some(key) => failure.after_key(words::CHANGED_NOT_SIGNED_IN, &session, key),
                    None => failure,
                })
            }
        }
    }

    /// The binding question was cancelled: the new session is logged out,
    /// and nothing here changes. The library's Keychain items stay: they
    /// hold the account this Mac may still be signed in to, and a recovery
    /// key that may still be on screen.
    pub fn abandon(&self, session: &Session) {
        if let Err(err) =
            kanso_sync::discard_session(self.backend.as_ref(), self.secrets.as_ref(), session)
        {
            eprintln!("kanso: signing out the cancelled session: {err}");
        }
    }

    /// Signs this device out. The logout is tried, or queued, whatever the
    /// server says.
    pub fn sign_out(&self, session: &Session) -> Result<(), AccountError> {
        kanso_sync::sign_out(self.backend.as_ref(), self.secrets.as_ref(), session)
    }

    /// Changes the account's password, and with it the recovery key.
    pub fn change_password(
        &self,
        session: &Session,
        current: &Password,
        new: &Password,
        step: &dyn Fn(Step),
    ) -> Result<kanso_sync::PasswordChanged, Failure> {
        // The new key is in the Keychain before the request goes.
        kanso_sync::change_password(
            self.backend.as_ref(),
            self.secrets.as_ref(),
            session,
            current,
            new,
            step,
        )
        .map_err(|err| {
            Failure::rotation(
                err,
                ErrorContext::CurrentPassword,
                words::KEY_UNCONFIRMED_PASSWORD,
            )
        })
    }

    /// Replaces the recovery key.
    pub fn new_recovery_key(
        &self,
        session: &Session,
        current: &Password,
        step: &dyn Fn(Step),
    ) -> Result<RecoveryKey, Failure> {
        // The new key is in the Keychain before the request goes.
        kanso_sync::new_recovery_key(
            self.backend.as_ref(),
            self.secrets.as_ref(),
            session,
            current,
            step,
        )
        .map_err(|err| {
            Failure::rotation(err, ErrorContext::CurrentPassword, words::KEY_UNCONFIRMED)
        })
    }

    /// Deletes the account and its server data, then unbinds the library.
    /// Notes on this device stay.
    pub fn delete_account(
        &self,
        session: &Session,
        password: &Password,
        typed: &str,
        library_id: &str,
        step: &dyn Fn(Step),
    ) -> Result<(), Failure> {
        kanso_sync::delete_account(self.backend.as_ref(), session, password, typed, step)
            .map_err(|err| Failure::account(err, ErrorContext::CurrentPassword))?;
        step(Step::Saving);
        kanso_sync::unbind(&self.store, self.secrets.as_ref()).map_err(Failure::bind)?;
        keychain::delete_recovery_key(self.secrets.as_ref(), library_id).ok();
        Ok(())
    }

    /// Signs another device of the account out. A device the server no
    /// longer knows is signed out already.
    pub fn revoke_device(&self, session: &Session, device_id: &str) -> Result<(), Failure> {
        match kanso_sync::revoke_device(self.backend.as_ref(), session, device_id) {
            Ok(()) | Err(AccountError::NotFound) => Ok(()),
            Err(err) => Err(Failure::account(err, ErrorContext::Account)),
        }
    }

    /// Asks `endpoint` whether it is a Kanso server, and whether it takes
    /// new accounts.
    pub fn check_server(
        &self,
        endpoint: String,
    ) -> Result<(String, kanso_sync::SignupState), Failure> {
        kanso_sync::server_info(self.backend.as_ref(), &endpoint)
            .map(|signup| (endpoint, signup))
            .map_err(|err| Failure::account(err, ErrorContext::Server))
    }

    /// Ends a locked provider sign-in whose credentials can't be used: the
    /// pending item and the token go. Not the same as `cancel_locked`, which
    /// also cancels an approval.
    pub fn end_locked(&self, locked: &LockedSession) -> Result<(), Failure> {
        kanso_sync::sign_out_locked(self.backend.as_ref(), self.secrets.as_ref(), locked)
            .map_err(|error| Failure::account(error, ErrorContext::Account))
    }

    /// Cancels a locked sign-in: its approval request when it has one, and
    /// the pending item and token. Both are attempted even when one fails;
    /// the first error is the answer.
    pub fn cancel_locked(&self, locked: &LockedSession) -> Result<(), AccountError> {
        // The request is in the secret store: a session the caller loaded
        // before it was made has no resume state to go by.
        let has_request = kanso_sync::saved_locked_pending(self.secrets.as_ref(), locked)
            .map(|saved| saved.approval_resume().is_some())
            .unwrap_or_else(|_| locked.approval_resume().is_some());
        let approval = if has_request {
            kanso_sync::cancel_saved_approval(self.backend.as_ref(), self.secrets.as_ref(), locked)
        } else {
            Ok(())
        };
        let sign_out = kanso_sync::discard_locked_session(
            self.backend.as_ref(),
            self.secrets.as_ref(),
            locked,
        );
        sign_out.and(approval)
    }

    /// Removes the pending item and token of a locked sign-in whose result
    /// went stale before it was shown.
    pub fn discard_locked(&self, locked: &LockedSession) {
        if let Err(error) =
            kanso_sync::discard_locked_session(self.backend.as_ref(), self.secrets.as_ref(), locked)
        {
            eprintln!("kanso: discarding a stale provider session: {error}");
        }
    }

    /// Uses the persisted provider login device to replace the locked token
    /// with a password-authenticated session, then returns it to UI-thread
    /// consent.
    pub fn unlock_with_password(
        &self,
        locked: &LockedSession,
        password: &Password,
        step: &dyn Fn(Step),
    ) -> Result<UnlockedCandidate, Failure> {
        let device = self.device()?;
        let unlocked = kanso_sync::unlock_with_password(
            self.backend.as_ref(),
            locked,
            password,
            &device,
            step,
        )
        .map_err(|error| Failure::account(error, ErrorContext::SignIn))?;
        let mut locked_session = locked.clone();
        if let Err(error) = locked_session
            .adopt_unlocked_session(&unlocked.session)
            .and_then(|()| {
                keychain::update_locked_pending(
                    self.secrets.as_ref(),
                    &device.library_id,
                    &locked_session,
                )
                .map_err(AccountError::Keychain)
            })
        {
            let mut failure = Failure::account(error, ErrorContext::SignIn);
            failure.keychain_warning |= discard_fresh_candidate(
                self.backend.as_ref(),
                self.secrets.as_ref(),
                &unlocked.session,
                false,
            );
            return Err(failure);
        }
        Ok(UnlockedCandidate {
            session: unlocked.session,
            locked_session,
            evicted: unlocked.evicted,
            library_id: device.library_id,
            same_token_as_locked: true,
        })
    }

    /// Recovers a locked provider session using its own server and
    /// username. Rotation uses the existing recovery transaction; the new
    /// key comes back with the candidate, to show before binding consent is
    /// asked.
    pub fn recover_locked(
        &self,
        locked: &LockedSession,
        key: &RecoveryKey,
        password: &Password,
        step: &dyn Fn(Step),
    ) -> Result<(UnlockedCandidate, RecoveryKey), Failure> {
        let device = self.device()?;
        let recovered = kanso_sync::recover(
            self.backend.as_ref(),
            self.secrets.as_ref(),
            &locked.server,
            &locked.username,
            key,
            password,
            &device,
            step,
        )
        .map_err(|error| {
            Failure::rotation(
                error,
                ErrorContext::Recovery,
                words::KEY_UNCONFIRMED_PASSWORD,
            )
        })?;
        let mut locked_session = locked.clone();
        if let Err(error) = locked_session
            .adopt_unlocked_session(&recovered.session)
            .and_then(|()| {
                keychain::update_locked_pending(
                    self.secrets.as_ref(),
                    &device.library_id,
                    &locked_session,
                )
                .map_err(AccountError::Keychain)
            })
        {
            let mut failure = Failure::account(error, ErrorContext::Recovery);
            failure.keychain_warning |= discard_fresh_candidate(
                self.backend.as_ref(),
                self.secrets.as_ref(),
                &recovered.session,
                false,
            );
            // Recovery rotated the server's keys and ended every session
            // that came before it, including this locked provider token.
            // The updated pending item could not be written, so remove the
            // old one and do not leave the app presenting a dead token.
            if let Err(cleanup) =
                kanso_sync::sign_out_locked(self.backend.as_ref(), self.secrets.as_ref(), locked)
            {
                eprintln!("kanso: clearing a recovery's stale locked session: {cleanup}");
                failure.keychain_warning |= matches!(&cleanup, AccountError::Keychain(_));
            }
            failure.locked_cleanup = true;
            return Err(failure.after_key(
                words::CHANGED_NOT_SIGNED_IN,
                &recovered.session,
                recovered.recovery_key,
            ));
        }
        Ok((
            UnlockedCandidate {
                session: recovered.session,
                locked_session,
                evicted: locked.evicted.clone(),
                library_id: device.library_id,
                same_token_as_locked: true,
            },
            recovered.recovery_key,
        ))
    }

    /// Cancels or rejects a candidate without touching the previous full
    /// item, binding, recovery item, or library contents: the locked
    /// sign-in (`locked`, or just its pending item) and the candidate's
    /// session go. Returns whether the Keychain refused any of it.
    pub fn cleanup_candidate(
        &self,
        locked: Option<&LockedSession>,
        candidate: &UnlockedCandidate,
    ) -> bool {
        let mut keychain_failed = false;
        let locked_cleanup = match locked {
            Some(locked) => {
                kanso_sync::sign_out_locked(self.backend.as_ref(), self.secrets.as_ref(), locked)
            }
            None => keychain::delete_locked_pending(self.secrets.as_ref(), &candidate.library_id)
                .map_err(AccountError::Keychain),
        };
        if let Err(error) = locked_cleanup {
            eprintln!("kanso: clearing locked pending session: {error}");
            keychain_failed |= matches!(&error, AccountError::Keychain(_));
        }
        if let Err(error) = kanso_sync::discard_session(
            self.backend.as_ref(),
            self.secrets.as_ref(),
            &candidate.session,
        ) {
            eprintln!("kanso: discarding unlocked candidate: {error}");
            keychain_failed |= matches!(&error, AccountError::Keychain(_));
        }
        keychain_failed
    }

    /// Promotes an unlocked candidate: its session becomes the library's
    /// Keychain item, the locked pending item goes, and the library binds
    /// with `choice` (`None` keeps the binding it has). Every step that
    /// fails puts the Keychain back as it was and cleans the candidate up.
    pub fn promote(
        &self,
        session: Session,
        library_id: String,
        same_token_as_locked: bool,
        choice: Option<Choice>,
        locked: Option<&LockedSession>,
    ) -> Result<(Session, String), Failure> {
        let item = keychain::account_item(&library_id);
        let before = self.secrets.get(&item).map_err(Failure::keychain)?;
        let rollback = |secrets: &dyn Secrets| match &before {
            Some(raw) => secrets.set(&item, raw),
            None => secrets.delete(&item),
        };
        if let Err(error) = keychain::save_session(self.secrets.as_ref(), &library_id, &session) {
            let mut failure = Failure::keychain(error);
            if let Err(cleanup) = rollback(self.secrets.as_ref()) {
                eprintln!(
                    "kanso: restoring the previous account item after save failure: {cleanup}"
                );
                failure.keychain_warning = true;
            }
            failure.keychain_warning |= discard_fresh_candidate(
                self.backend.as_ref(),
                self.secrets.as_ref(),
                &session,
                same_token_as_locked,
            );
            return Err(failure);
        }
        if let Err(error) = keychain::delete_locked_pending(self.secrets.as_ref(), &library_id) {
            let mut failure = Failure::keychain(error);
            if let Err(cleanup) = rollback(self.secrets.as_ref()) {
                eprintln!(
                    "kanso: restoring the previous account item after pending-delete failure: {cleanup}"
                );
                failure.keychain_warning = true;
            }
            failure.keychain_warning |= discard_fresh_candidate(
                self.backend.as_ref(),
                self.secrets.as_ref(),
                &session,
                same_token_as_locked,
            );
            return Err(failure);
        }
        if let Some(choice) = choice
            && let Err(error) =
                kanso_sync::bind(&self.store, &session, choice, &self.backups, self.now_ms)
        {
            let rollback_error = rollback(self.secrets.as_ref()).err();
            let mut cleanup_failed = false;
            if let Some(locked) = locked
                && let Err(cleanup) = kanso_sync::sign_out_locked(
                    self.backend.as_ref(),
                    self.secrets.as_ref(),
                    locked,
                )
            {
                eprintln!("kanso: discarding failed binding's locked session: {cleanup}");
                cleanup_failed |= matches!(&cleanup, AccountError::Keychain(_));
            } else if let Err(cleanup) =
                keychain::delete_locked_pending(self.secrets.as_ref(), &library_id)
            {
                eprintln!("kanso: clearing failed binding's locked session: {cleanup}");
                cleanup_failed = true;
            }
            if let Err(cleanup) =
                kanso_sync::discard_session(self.backend.as_ref(), self.secrets.as_ref(), &session)
            {
                eprintln!("kanso: discarding failed binding candidate: {cleanup}");
                cleanup_failed |= matches!(&cleanup, AccountError::Keychain(_));
            }
            let mut failure = Failure::bind(error);
            failure.locked_cleanup = true;
            failure.keychain_warning |= cleanup_failed;
            if let Some(error) = rollback_error {
                eprintln!("kanso: restoring the previous account item after bind failure: {error}");
                let keychain_failure = Failure::keychain(error);
                failure.words = keychain_failure.words;
                failure.keychain_warning = true;
            }
            return Err(failure);
        }
        Ok((session, library_id))
    }
}

/// The files in the backups folder, to find the one a failed Replace made.
fn backups_in(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut paths: Vec<PathBuf> = entries.flatten().map(|entry| entry.path()).collect();
    paths.sort();
    paths
}

#[cfg(test)]
mod tests {
    use kanso_sync::{LocalLibrary, MemorySecrets, MemoryServer, Quota, meta_keys};
    use proptest::prelude::*;

    use super::lab::{Lab, PASSWORD, SERVER, password};
    use super::*;

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
    fn methods_refresh_rejects_stale_answers_and_marks_retained_success_failed() {
        let mut state = AccountMethodsState::default();
        let first = state.begin_refresh();
        assert!(state.finish_success(first, account_methods(true)));

        let second = state.begin_refresh();
        assert!(!state.finish_success(first, account_methods(false)));
        assert!(state.finish_failure(second, "offline".into()));
        assert_eq!(state.last_successful, Some(account_methods(true)));
        assert!(!state.loading);
        assert_eq!(state.failed.as_deref(), Some("offline"));
    }

    #[test]
    fn creating_an_account_binds_the_library_and_keeps_the_session() {
        let server = Arc::new(MemoryServer::new());
        let lab = Lab::new(&server, "Studio", 2);
        let (signed, library_id) = lab
            .flow
            .create(SERVER, "alice", &password("alice"), &|_| {})
            .unwrap();
        // The library is bound to the new account, with its notes.
        let session = &signed.session;
        assert_eq!(
            kanso_sync::local_library(&lab.store, &session.origin, &session.account_id).unwrap(),
            LocalLibrary::Same
        );
        assert_eq!(library_id, lab.flow.device().unwrap().library_id);
        // The session is in the secret store, for the next launch.
        let kept = keychain::load_session(
            lab.secrets.as_ref(),
            &library_id,
            &session.origin,
            &session.account_id,
        )
        .unwrap()
        .expect("the session item");
        assert_eq!(kept.account_id, session.account_id);
        // And the recovery key waits there until it is saved.
        let key = keychain::load_any_recovery_key(lab.secrets.as_ref(), &library_id)
            .unwrap()
            .expect("the unsaved recovery key");
        assert_eq!(key.recovery_key, signed.recovery_key);
    }

    #[test]
    fn a_second_device_signs_in_asks_nothing_and_binds() {
        let server = Arc::new(MemoryServer::new());
        let first = Lab::new(&server, "Studio", 1);
        first
            .flow
            .create(SERVER, "alice", &password("alice"), &|_| {})
            .unwrap();
        let second = Lab::new(&server, "Laptop", 0);
        let auth = second
            .flow
            .authenticate(
                Box::new(|flow, device, step| {
                    flow.sign_in_password(SERVER, "alice", &password("alice"), device, step)
                }),
                &|_| {},
            )
            .unwrap();
        assert_eq!(auth.library, LocalLibrary::Empty);
        let Next::Bind(choice) =
            after_auth(kanso_sync::Flow::SignIn, &auth.library, "alice", false)
        else {
            panic!("an empty library binds without a question");
        };
        assert_eq!(choice, Some(Choice::Add));
        let (session, library_id, backup) = second
            .flow
            .bind_and_save(auth.session, auth.library_id, choice, None)
            .unwrap();
        assert_eq!(backup, None);
        assert_eq!(
            second.store.meta(meta_keys::ACCOUNT_ID).unwrap().as_deref(),
            Some(session.account_id.as_str())
        );
        assert!(
            second
                .secrets
                .items()
                .contains(&keychain::account_item(&library_id))
        );
    }

    #[test]
    fn a_wrong_password_fails_in_plain_words_and_binds_nothing() {
        let server = Arc::new(MemoryServer::new());
        let first = Lab::new(&server, "Studio", 0);
        first
            .flow
            .create(SERVER, "alice", &password("alice"), &|_| {})
            .unwrap();
        let second = Lab::new(&server, "Laptop", 3);
        let wrong = Password::new("not the right passphrase", "alice").unwrap();
        let err = second
            .flow
            .authenticate(
                Box::new(move |flow, device, step| {
                    flow.sign_in_password(SERVER, "alice", &wrong, device, step)
                }),
                &|_| {},
            )
            .err()
            .expect("a wrong password fails");
        assert_eq!(
            err.words,
            words::error_words(&AccountError::InvalidCredentials, ErrorContext::SignIn)
        );
        assert!(err.recovery.is_none() && err.signed_out.is_none());
        assert_eq!(second.store.meta(meta_keys::ACCOUNT_ID).unwrap(), None);
        assert!(second.secrets.items().is_empty());
    }

    fn libraries() -> impl Strategy<Value = LocalLibrary> {
        prop_oneof![
            Just(LocalLibrary::Empty),
            Just(LocalLibrary::Same),
            (1usize..50, 0usize..5)
                .prop_map(|(notes, unsynced)| LocalLibrary::Unbound { notes, unsynced }),
            ("[a-z]{3,8}", 1usize..50, 0usize..5).prop_map(|(username, notes, unsynced)| {
                LocalLibrary::Other {
                    username,
                    notes,
                    unsynced,
                }
            }),
        ]
    }

    fn flows() -> impl Strategy<Value = kanso_sync::Flow> {
        prop::sample::select(vec![
            kanso_sync::Flow::SignUp,
            kanso_sync::Flow::SignIn,
            kanso_sync::Flow::Recover,
        ])
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Refinement of the plan table: after authenticating, a question is
        /// asked only when the plan asks one and a window can; it is never
        /// answered for the person (no Replace or Cancel is picked), and a
        /// question with no one to ask abandons.
        #[test]
        fn prop_after_auth_never_picks_a_choice_unasked(
            flow in flows(),
            library in libraries(),
            can_ask: bool,
        ) {
            let plan = kanso_sync::plan(flow, &library);
            let next = after_auth(flow, &library, "alice", can_ask);
            match (&plan, &next) {
                (Plan::Ask { .. }, Next::Ask(question)) => {
                    prop_assert!(can_ask);
                    prop_assert!(!question.answers.is_empty());
                }
                (Plan::Ask { .. }, Next::Abandon) => prop_assert!(!can_ask),
                (Plan::Resume, Next::Bind(None)) => {}
                (Plan::Fresh | Plan::AskBeforeSignUp { .. }, Next::Bind(Some(Choice::Add))) => {}
                other => prop_assert!(false, "{other:?}"),
            }
            if let Next::Bind(Some(choice)) = next {
                prop_assert_eq!(choice, Choice::Add);
            }
        }

        /// The same for any unlock: the plan is a sign-in's, a window asks
        /// the question, one without a window parks the candidate, and no
        /// choice is made for the person.
        #[test]
        fn prop_after_unlock_never_picks_a_choice_unasked(
            library in libraries(),
            can_ask: bool,
        ) {
            let plan = kanso_sync::plan(kanso_sync::Flow::SignIn, &library);
            let next = after_unlock(&library, "alice", can_ask);
            match (&plan, &next) {
                (Plan::Ask { .. }, NextUnlock::Ask(_)) => prop_assert!(can_ask),
                (Plan::Ask { .. }, NextUnlock::Park) => prop_assert!(!can_ask),
                (Plan::Resume, NextUnlock::Promote(None)) => {}
                (Plan::Fresh, NextUnlock::Promote(Some(Choice::Add))) => {}
                other => prop_assert!(false, "{other:?}"),
            }
        }
    }

    fn usages() -> impl Strategy<Value = Usage> {
        (0u64..1000, 0u64..1000).prop_map(|(docs, doc_bytes)| Usage {
            docs,
            doc_bytes,
            files: 0,
            file_bytes: 0,
            writes_today: 0,
            quota: Quota::default(),
        })
    }

    fn device_lists() -> impl Strategy<Value = Vec<DeviceInfo>> {
        prop::collection::vec(
            (0usize..8, 0i64..1_000, any::<bool>()).prop_map(|(n, last_seen_at, current)| {
                DeviceInfo {
                    id: format!("d_{n}"),
                    name: Some(format!("Device {n}")),
                    created_at: 0,
                    last_seen_at,
                    current,
                }
            }),
            0..6,
        )
    }

    /// How one of the two answers went.
    #[derive(Clone, Copy, Debug)]
    enum Went {
        Fine,
        NotFound,
        Offline,
        Ended(SignOutReason),
    }

    impl Went {
        fn error(self) -> Option<AccountError> {
            match self {
                Went::Fine => None,
                Went::NotFound => Some(AccountError::NotFound),
                Went::Offline => Some(AccountError::Transport("reset".into())),
                Went::Ended(reason) => Some(AccountError::Unauthorized { reason }),
            }
        }
    }

    fn answer_errors() -> impl Strategy<Value = Went> {
        prop_oneof![
            Just(Went::Fine),
            Just(Went::NotFound),
            Just(Went::Offline),
            prop::sample::select(SignOutReason::ALL.to_vec()).prop_map(Went::Ended),
        ]
    }

    proptest! {
        /// Whatever else the two answers say, a 401 on either ends the
        /// session, for the first one's reason; otherwise the details show
        /// what arrived, the first failure's words when one failed, and the
        /// devices with this one first.
        #[test]
        fn prop_details_from_ends_the_session_on_any_401(
            usage in usages(),
            devices in device_lists(),
            usage_error in answer_errors(),
            devices_error in answer_errors(),
        ) {
            let reason_of = |went: Went| match went {
                Went::Ended(reason) => Some(reason),
                _ => None,
            };
            let expected = reason_of(usage_error).or(reason_of(devices_error));
            let usage_answer = match usage_error.error() {
                Some(err) => Err(err),
                None => Ok(usage),
            };
            let devices_answer = match devices_error.error() {
                Some(err) => Err(err),
                None => Ok(devices.clone()),
            };
            let usage_fine = matches!(usage_error, Went::Fine);
            let devices_fine = matches!(devices_error, Went::Fine);
            match details_from(usage_answer, devices_answer) {
                Err(reason) => prop_assert_eq!(Some(reason), expected),
                Ok(details) => {
                    prop_assert!(expected.is_none());
                    prop_assert!(!details.loading && !details.stale);
                    prop_assert_eq!(details.usage, usage_fine.then_some(usage));
                    prop_assert_eq!(details.failed.is_some(), !usage_fine || !devices_fine);
                    match (devices_fine, details.devices) {
                        (true, Some(shown)) => {
                            prop_assert_eq!(shown.len(), devices.len());
                            // This device first, then the most recently active.
                            for pair in shown.windows(2) {
                                prop_assert!(pair[0].current >= pair[1].current);
                                if pair[0].current == pair[1].current {
                                    prop_assert!(pair[0].last_seen_at >= pair[1].last_seen_at);
                                }
                            }
                        }
                        (false, None) => {}
                        other => prop_assert!(false, "{other:?}"),
                    }
                }
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(32))]

        /// Invariant: a binding that fails leaves the library's account item
        /// as it was, whether there was none or a stranger's, and says the
        /// library couldn't be changed. (A trigger makes the library refuse
        /// the binding, as a full disk would.)
        #[test]
        fn prop_a_failed_binding_leaves_the_account_item_as_it_was(
            prior in prop::option::of("\\PC{0,40}"),
            replace: bool,
        ) {
            let server = Arc::new(MemoryServer::new());
            let lab = Lab::new(&server, "Studio", 1);
            let library_id = lab.flow.device().unwrap().library_id;
            let session = kanso_sync::sign_up(
                server.as_ref(),
                &MemorySecrets::new(),
                SERVER,
                "alice",
                &password("alice"),
                &lab.flow.device().unwrap(),
                &|_| {},
            )
            .unwrap()
            .session;
            let item = keychain::account_item(&library_id);
            if let Some(raw) = &prior {
                lab.secrets.set(&item, raw).unwrap();
            }
            let conn = rusqlite::Connection::open(lab.dir.path().join("kanso.db")).unwrap();
            conn.execute_batch(&format!(
                "CREATE TRIGGER refuse BEFORE INSERT ON meta WHEN NEW.key = '{}' \
                 BEGIN SELECT RAISE(ABORT, 'refused'); END;",
                meta_keys::ACCOUNT_ID
            ))
            .unwrap();
            let choice = if replace { Choice::Replace } else { Choice::Add };
            let failure = lab
                .flow
                .save_and_bind(&library_id, &session, Some(choice))
                .expect_err("the library refuses");
            prop_assert_eq!(failure.words.as_str(), LIBRARY_TROUBLE);
            let now = lab.secrets.get(&item).unwrap().map(|raw| raw.to_string());
            prop_assert_eq!(now, prior);
        }
    }

    #[test]
    fn the_password_used_in_the_tests_is_long_enough() {
        assert!(Password::new(PASSWORD, "alice").is_ok());
    }
}
