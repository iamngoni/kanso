//! Settings › Account › Sign-in methods: adding and removing a passkey,
//! linking and unlinking a provider (docs/ACCOUNTS.md › Account flows ›
//! Link a provider, › Changing a way in needs a fresh proof).
//!
//! Every change takes a fresh proof. On the phone that is the account's
//! current password for now (a passkey proof is a later improvement), so the
//! four calls need an account that has one. That also keeps the recovery key
//! out of them: the desktop asks for it only to remove the last way in, and
//! an account with a password always has another.
//!
//! The ceremony and the provider runner are the sign-in ones
//! ([`crate::passkey_bridge`], [`crate::account_provider`]); a link is the
//! provider flow on the signed-in session's route.

use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use kanso_engine::account::provider::{
    ProviderDone, ProviderRoute, provider_begin, provider_finish,
};
use kanso_engine::account::words::{self, ErrorContext};
use kanso_engine::account::{Failure, SOMETHING_WRONG};
use kanso_sync::passkey::PasskeyCeremony;
use kanso_sync::provider::{IdentityIntent, LinkOutcome, ProviderStart};
use kanso_sync::{Proof, Session};

use crate::account::{AccountHandle, RunGuard};
use crate::account_model::{AccountDetails, ProviderEvent, ProviderKind};
use crate::account_provider::{Events, PhoneBrowser, open_apple, provider_of};
use crate::account_runtime::{begin_new, current, publish};
use crate::account_settings::AccountSettings;
use crate::boundary::guard;
use crate::errors::KansoError;
use crate::passkey_flow::{Ended, PasskeyError, RpRule, ended};
use crate::syncing::account_error;

impl AccountSettings {
    /// Adds a passkey to the account, proving it with `current_password`.
    /// The native sheet runs through [`AccountHandle::passkey_requests`]. A
    /// dismissed sheet changes nothing and is not an error.
    pub fn add_passkey(&self, current_password: String) -> Result<AccountDetails, KansoError> {
        guard(|| {
            let account = self.account();
            let _busy = begin_new(&self.inner)?;
            self.inner
                .account
                .ceremony_cancel
                .store(false, std::sync::atomic::Ordering::Release);
            let session = account.require_session()?;
            let auth_key = self.prove(&session, &current_password)?;
            let proof = Proof::Password(&auth_key);
            let context = ErrorContext::CurrentPassword;
            let backend = self.inner.backend.as_ref();
            let start = kanso_sync::start_add_passkey(backend, &session, &proof)
                .map_err(|err| account_error(&err, context))?;
            if !RpRule::platform().allows_ceremony(
                &start.challenge.rp_id,
                &start.challenge.origin,
                &session.server,
            ) {
                return Err(KansoError::invalid(words::PASSKEY_WRONG_SERVER));
            }
            let registration = account.ceremony().register(&start.challenge);
            match registration {
                Ok(registration) => {
                    kanso_sync::add_passkey(backend, &session, &start, registration)
                        .map_err(|err| account_error(&err, context))?;
                }
                Err(error) => match ended(PasskeyError::Ceremony(error), ErrorContext::Passkey) {
                    Ended::Cancelled => {}
                    Ended::Failed(failure) => return Err(account.settle(failure, context)),
                    Ended::Error(error) => return Err(error),
                },
            }
            self.refreshed(&session)
        })
    }

    /// Removes the passkey `id` (the `id` of a [`PasskeyRow`]), proving it
    /// with `current_password`.
    pub fn remove_passkey(
        &self,
        id: String,
        current_password: String,
    ) -> Result<AccountDetails, KansoError> {
        guard(|| {
            let credential_id = URL_SAFE_NO_PAD
                .decode(&id)
                .map_err(|_| KansoError::invalid("That isn't one of this account's passkeys."))?;
            let _busy = begin_new(&self.inner)?;
            let session = self.account().require_session()?;
            let auth_key = self.prove(&session, &current_password)?;
            kanso_sync::remove_passkey(
                self.inner.backend.as_ref(),
                &session,
                &credential_id,
                &Proof::Password(&auth_key),
                None,
            )
            .map_err(|err| account_error(&err, ErrorContext::CurrentPassword))?;
            self.refreshed(&session)
        })
    }

    /// Links `provider` to the account, proving it with `current_password`,
    /// and reports the flow's steps to `sink` on a thread of its own, as
    /// [`AccountHandle::start_provider`] does. `Finished` carries the state
    /// (still signed in). The pages end as a sign-in's do, through
    /// [`AccountHandle::provider_page_ended`] and
    /// [`AccountHandle::cancel_provider`].
    pub fn link_provider(
        &self,
        provider: ProviderKind,
        current_password: String,
        sink: impl FnMut(ProviderEvent) -> bool + Send + 'static,
    ) {
        let cancel = Arc::new(AtomicBool::new(false));
        let events = Events::new(sink, cancel.clone());
        let (this, thread_events) = (self.clone(), events.clone());
        let spawned = std::thread::Builder::new()
            .name("kanso-account".into())
            .spawn(move || {
                let end = match guard(|| {
                    Ok(this.link(provider, &current_password, &thread_events, &cancel))
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

    /// Unlinks `provider` from the account, proving it with
    /// `current_password`.
    pub fn unlink_provider(
        &self,
        provider: ProviderKind,
        current_password: String,
    ) -> Result<AccountDetails, KansoError> {
        guard(|| {
            let _busy = begin_new(&self.inner)?;
            let session = self.account().require_session()?;
            let auth_key = self.prove(&session, &current_password)?;
            kanso_sync::unlink_identity(
                self.inner.backend.as_ref(),
                &session,
                &provider_of(provider),
                Proof::Password(&auth_key).to_wire(),
                None,
            )
            .map_err(|err| account_error(&err, ErrorContext::CurrentPassword))?;
            self.refreshed(&session)
        })
    }
}

impl AccountSettings {
    fn account(&self) -> AccountHandle {
        AccountHandle {
            inner: self.inner.clone(),
        }
    }

    /// The password's auth key for one change.
    fn prove(
        &self,
        session: &Session,
        current_password: &str,
    ) -> Result<kanso_sync::crypto::AuthKey, KansoError> {
        let password = words::check_current_password(current_password, &session.username)
            .map_err(KansoError::invalid)?;
        kanso_sync::password_proof(self.inner.backend.as_ref(), session, &password, &|_| {})
            .map_err(|err| account_error(&err, ErrorContext::CurrentPassword))
    }

    /// The account's page as it is now, fetched again after a change.
    fn refreshed(&self, _session: &Session) -> Result<AccountDetails, KansoError> {
        self.details()
    }

    /// A link: the password proof, then the provider flow on this session's
    /// route.
    fn link(
        &self,
        kind: ProviderKind,
        current_password: &str,
        events: &Arc<Events>,
        cancel: &Arc<AtomicBool>,
    ) -> ProviderEvent {
        let account = self.account();
        let context = ErrorContext::Account;
        let failed = |err: &KansoError| ProviderEvent::Failed {
            message: err.to_string(),
        };
        let _busy = match begin_new(&self.inner) {
            Ok(busy) => busy,
            Err(err) => return failed(&err),
        };
        let session = match account.require_session() {
            Ok(session) => Arc::new(session),
            Err(err) => return failed(&err),
        };
        let auth_key = match self.prove(&session, current_password) {
            Ok(key) => key,
            Err(err) => return failed(&err),
        };
        let provider = provider_of(kind);
        let flow = account.flow("");
        let login = match flow.device() {
            Ok(device) => session.login_device(&device),
            Err(failure) => return failed(&account.settle(failure, context)),
        };
        let _running = RunGuard::install(&self.inner, cancel.clone());
        let route = ProviderRoute::Session {
            session: session.clone(),
            intent: IdentityIntent::Link,
            proof: Some(Proof::Password(&auth_key).to_wire()),
        };
        let browser = PhoneBrowser {
            events: events.clone(),
        };
        let backend = self.inner.backend.as_ref();
        let started = match provider_begin(backend, &browser, &route, &provider, &login, &|_| {}) {
            Ok(started) => started,
            Err(err) => {
                return failed(&account.settle(Failure::account(err, context), context));
            }
        };
        let apple = match &started.start {
            ProviderStart::Apple(start) => {
                match open_apple(events, &self.inner.account.provider, start) {
                    Ok(wait) => Some(wait),
                    Err(err) => {
                        return failed(&account.settle(Failure::account(err, context), context));
                    }
                }
            }
            _ => None,
        };
        let finished = provider_finish(
            backend,
            &browser,
            &route,
            &provider,
            &login,
            started,
            apple,
            cancel.clone(),
            &|_| {},
        );
        events.close_page();
        match finished {
            Err(err) => failed(&account.settle(Failure::account(err, context), context)),
            Ok(None) => ProviderEvent::Cancelled,
            Ok(Some(ProviderDone::Link(LinkOutcome::Linked { .. }))) => {
                match current(&self.inner) {
                    Ok(state) => ProviderEvent::Finished(state),
                    Err(err) => failed(&err),
                }
            }
            Ok(Some(_)) => ProviderEvent::Failed {
                message: SOMETHING_WRONG.to_owned(),
            },
        }
    }
}
