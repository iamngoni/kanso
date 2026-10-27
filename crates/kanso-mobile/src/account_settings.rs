//! Settings › Account: the signed-in account's devices and ways in, its password and recovery key, signing out and deleting it.
//!
//! The same tasks as the desktop's Account page (`account_flow`), on the
//! engine's `Flow` and `kanso_sync`'s calls, with the desktop's words for
//! their failures. Anything that changes this phone's session stops a
//! running sync first and tells the status listeners after. A password is
//! the only proof for now; the account's recovery key is asked for where
//! the desktop asks for it (nowhere on these calls).
//!
//! A recovery key made here is kept in the secret store until the person
//! says they saved it (`AccountHandle::recovery_key_saved`), as the
//! desktop keeps it; [`AccountSettings::pending_recovery_key`] shows it
//! again after the app was closed or a call failed with the server maybe
//! having taken it.

use std::sync::Arc;

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use kanso_engine::account::words::{self, ErrorContext};
use kanso_engine::account::{Failure, details_from};
use kanso_engine::sync::{SyncState, forget_ended_session};
use kanso_sync::keychain;
use kanso_sync::provider::{AccountMethods, PasskeyInfo};
use kanso_sync::{
    AccountError, DeviceInfo, Password, Provider, RotationError, Session, SignOutReason, meta_keys,
    normalize_username,
};
use zeroize::Zeroizing;

use crate::account_model::{
    AccountDetails, DeviceRow, LinkedProviderRow, PasskeyRow, ProviderKind, RecoveryKeyText,
};
use crate::approvals;
use crate::boundary::{guard, held};
use crate::errors::KansoError;
use crate::library::Inner;
use crate::syncing::{SyncHandle, account_error, failure};

/// Settings › Account: the signed-in account's devices and ways in, its password and recovery key, signing out and deleting it.
#[derive(Clone)]
pub struct AccountSettings {
    pub(crate) inner: Arc<Inner>,
}

impl AccountSettings {
    /// The account's page: its devices (this phone first), the storage it
    /// uses, and the ways in. A server that ended this phone's session
    /// signs it out (`KansoError::SignedOut`).
    pub fn details(&self) -> Result<AccountDetails, KansoError> {
        guard(|| {
            let session = self.session()?;
            self.read_details(&session)
        })
    }

    /// Signs another device of the account out, and gives the page again.
    /// A device the server no longer knows is signed out already. This
    /// phone is signed out with [`AccountSettings::sign_out`], not here.
    pub fn revoke_device(&self, device_id: String) -> Result<AccountDetails, KansoError> {
        guard(|| {
            let session = self.session()?;
            if device_id == session.device_id {
                return Err(KansoError::invalid(
                    "That is this phone. Sign out of the account instead.",
                ));
            }
            match kanso_sync::revoke_device(self.inner.backend.as_ref(), &session, &device_id) {
                Ok(()) | Err(AccountError::NotFound) => {}
                Err(err) => return Err(self.fail(&session, &err, ErrorContext::Account)),
            }
            self.read_details(&session)
        })
    }

    /// Signs this phone out of its account. The server is told, or told
    /// later if it can't be reached; the library's notes stay. A sign-in
    /// that never opened (a locked session) ends too, with its device
    /// approval request. Signing out twice is fine.
    pub fn sign_out(&self) -> Result<(), KansoError> {
        guard(|| {
            let sync = self.sync();
            let _stopped = sync.stop_sync();
            let mut first = None;
            match sync.load_session() {
                Ok(Some(session)) => {
                    if let Err(err) = sync.flow("").sign_out(&session) {
                        first = Some(account_error(&err, ErrorContext::Account));
                    }
                }
                Ok(None) => {}
                Err(err) => first = Some(err),
            }
            // Whatever the session did, a locked sign-in goes too.
            if let Err(err) = self.end_locked() {
                first.get_or_insert(err);
            }
            *held(&self.inner.sync_state) = SyncState::Off;
            sync.notify();
            approvals::forget(&self.inner);
            first.map_or(Ok(()), Err)
        })
    }

    /// Changes the account's password, and with it the recovery key: the
    /// account's other devices are signed out and ask for the new
    /// password. The new key shows once; it is kept until the person says
    /// they saved it. If the call fails after the server may have taken a
    /// key, [`AccountSettings::pending_recovery_key`] has it.
    pub fn change_password(
        &self,
        current: String,
        new: String,
    ) -> Result<RecoveryKeyText, KansoError> {
        guard(|| {
            let current = Zeroizing::new(current);
            let new = Zeroizing::new(new);
            let sync = self.sync();
            let _stopped = sync.stop_sync();
            let session = self.session()?;
            let current = typed_password(&current, &session, "the password")?;
            let new = typed_password(&new, &session, "the new password")?;
            let changed = kanso_sync::change_password(
                self.inner.backend.as_ref(),
                self.inner.secrets.as_ref(),
                &session,
                &current,
                &new,
                &|_| {},
            )
            .map_err(|err| self.rotation_failed(&session, err))?;
            sync.notify();
            Ok(key_text(&changed.recovery_key))
        })
    }

    /// Replaces the recovery key and gives the new one, to show once. The
    /// one the person saved stops working. Kept like
    /// [`AccountSettings::change_password`]'s.
    pub fn new_recovery_key(
        &self,
        current_password: String,
    ) -> Result<RecoveryKeyText, KansoError> {
        guard(|| {
            let current = Zeroizing::new(current_password);
            let sync = self.sync();
            let _stopped = sync.stop_sync();
            let session = self.session()?;
            let current = typed_password(&current, &session, "the password")?;
            let key = kanso_sync::new_recovery_key(
                self.inner.backend.as_ref(),
                self.inner.secrets.as_ref(),
                &session,
                &current,
                &|_| {},
            )
            .map_err(|err| self.rotation_failed(&session, err))?;
            sync.notify();
            Ok(key_text(&key))
        })
    }

    /// The recovery key shown last and not yet confirmed saved, if there
    /// is one: what the page shows again after the app was closed, or
    /// after a password or key change failed with the server maybe having
    /// taken the new key.
    pub fn pending_recovery_key(&self) -> Result<Option<RecoveryKeyText>, KansoError> {
        guard(|| {
            let session = self.session()?;
            let Some(library_id) = session.library_id() else {
                return Ok(None);
            };
            let kept = keychain::load_recovery_key(
                self.inner.secrets.as_ref(),
                library_id,
                &session.origin,
                &session.account_id,
            )?;
            Ok(kept.map(|kept| key_text(&kept.recovery_key)))
        })
    }

    /// Deletes the account and its server data, then unbinds this phone's
    /// library: the notes stay. `typed_username` must be the account's
    /// name, as the desktop asks.
    pub fn delete_account(
        &self,
        password: String,
        typed_username: String,
    ) -> Result<(), KansoError> {
        guard(|| {
            let proof = Zeroizing::new(password);
            let sync = self.sync();
            let _stopped = sync.stop_sync();
            let session = self.session()?;
            let typed = normalize_username(&typed_username)
                .ok()
                .filter(|typed| *typed == session.username);
            let Some(typed) = typed else {
                return Err(KansoError::invalid(words::error_words(
                    &AccountError::UsernameMismatch,
                    ErrorContext::CurrentPassword,
                )));
            };
            let proof = typed_password(&proof, &session, "the password")?;
            kanso_sync::delete_account(
                self.inner.backend.as_ref(),
                &session,
                &proof,
                &typed,
                &|_| {},
            )
            .map_err(|err| self.fail(&session, &err, ErrorContext::CurrentPassword))?;
            kanso_sync::unbind(&self.inner.store, self.inner.secrets.as_ref())
                .map_err(|err| failure(Failure::bind(err)))?;
            if let Some(library_id) = session.library_id() {
                // The key belonged to the account that is gone.
                let _ = keychain::delete_recovery_key(self.inner.secrets.as_ref(), library_id);
            }
            *held(&self.inner.sync_state) = SyncState::Off;
            sync.notify();
            approvals::forget(&self.inner);
            Ok(())
        })
    }

    fn sync(&self) -> SyncHandle {
        SyncHandle {
            inner: self.inner.clone(),
        }
    }

    /// The session of the account this phone is signed in to. A locked
    /// sign-in has none: the account page needs the data key.
    fn session(&self) -> Result<Session, KansoError> {
        match self.sync().load_session()? {
            Some(session) => Ok(session),
            None if self.locked_waiting()? => Err(KansoError::auth(
                "This phone hasn't opened the account yet. Unlock it first.",
            )),
            None => Err(KansoError::SignedOut),
        }
    }

    /// Whether a sign-in that never opened is saved for this library.
    fn locked_waiting(&self) -> Result<bool, KansoError> {
        let Some(library_id) = self.inner.store.meta(meta_keys::LIBRARY_ID)? else {
            return Ok(false);
        };
        Ok(keychain::load_locked_pending(self.inner.secrets.as_ref(), &library_id)?.is_some())
    }

    /// Ends the saved locked sign-in, if there is one: its device approval
    /// request, then its token and item.
    fn end_locked(&self) -> Result<(), KansoError> {
        let flow = self.sync().flow("");
        let device = flow.device().map_err(failure)?;
        let secrets = self.inner.secrets.as_ref();
        let Some(locked) = keychain::load_locked_pending(secrets, &device.library_id)? else {
            return Ok(());
        };
        if locked.approval_resume().is_some() {
            // The request expires on its own if the server can't be asked.
            let _ =
                kanso_sync::cancel_saved_approval(self.inner.backend.as_ref(), secrets, &locked);
        }
        flow.end_locked(&locked).map_err(failure)
    }

    /// The account's page for `session`.
    fn read_details(&self, session: &Session) -> Result<AccountDetails, KansoError> {
        let backend = self.inner.backend.as_ref();
        let usage = kanso_sync::usage(backend, session);
        let devices = kanso_sync::devices(backend, session);
        let fetched = match details_from(usage, devices) {
            Ok(fetched) => fetched,
            Err(reason) => {
                session_ended(&self.inner, session, reason);
                approvals::forget(&self.inner);
                return Err(KansoError::SignedOut);
            }
        };
        let Some(devices) = fetched.devices else {
            let said = fetched
                .failed
                .unwrap_or_else(|| words::CANT_REACH.to_owned());
            return Err(KansoError::network(said));
        };
        let methods = kanso_sync::account::methods(backend, session)
            .map_err(|err| self.fail(session, &err, ErrorContext::Account))?;
        let library_id = session.library_id();
        let unsaved = match library_id {
            Some(library_id) => keychain::load_recovery_key(
                self.inner.secrets.as_ref(),
                library_id,
                &session.origin,
                &session.account_id,
            )?
            .is_some(),
            None => false,
        };
        Ok(AccountDetails {
            username: session.username.clone(),
            server: session.server.clone(),
            devices: device_rows(devices),
            storage_line: fetched.usage.as_ref().map(words::storage_line),
            has_password: methods.has_password,
            providers: provider_rows(&methods),
            passkeys: passkey_rows(&methods.passkeys),
            recovery_key_unsaved: unsaved,
        })
    }

    /// A failed call as the shell hears it. A 401 means the server ended
    /// this phone's session: it is signed out here too.
    fn fail(&self, session: &Session, err: &AccountError, context: ErrorContext) -> KansoError {
        if let AccountError::Unauthorized { reason } = err {
            session_ended(&self.inner, session, *reason);
            approvals::forget(&self.inner);
        }
        account_error(err, context)
    }

    /// A failed password or key change. The key the Keychain kept stays
    /// there for [`AccountSettings::pending_recovery_key`].
    fn rotation_failed(&self, session: &Session, err: RotationError) -> KansoError {
        self.fail(session, &err.error, ErrorContext::CurrentPassword)
    }
}

/// What the library forgets once the server ended `session` (a 401): the
/// saved session, and the sync state with it. Tells the status listeners.
pub(crate) fn session_ended(inner: &Arc<Inner>, session: &Session, reason: SignOutReason) {
    if let Some(library_id) = session.library_id() {
        forget_ended_session(
            inner.secrets.as_ref(),
            library_id,
            reason,
            Some(session),
            false,
        );
    }
    *held(&inner.sync_state) = SyncState::Off;
    SyncHandle {
        inner: inner.clone(),
    }
    .notify();
}

/// A typed password as the engine takes it, or why it can't be one.
fn typed_password(text: &str, session: &Session, what: &str) -> Result<Password, KansoError> {
    Password::new(text, &session.username)
        .map_err(|err| KansoError::invalid(format!("For {what}, {err}.")))
}

/// A recovery key as the person writes it down.
fn key_text(key: &kanso_sync::RecoveryKey) -> RecoveryKeyText {
    RecoveryKeyText { key: key.display() }
}

/// The devices as the page lists them: this phone first, then the most
/// recently seen. A device that was never seen has no time.
pub(crate) fn device_rows(mut devices: Vec<DeviceInfo>) -> Vec<DeviceRow> {
    words::sort_devices(&mut devices);
    devices
        .iter()
        .map(|device| DeviceRow {
            id: device.id.clone(),
            title: words::device_title(device),
            this_device: device.current,
            last_seen_ms: (device.last_seen_at > 0).then_some(device.last_seen_at),
        })
        .collect()
}

/// The sign-in provider this app has a button for.
pub(crate) fn kind_of(provider: &Provider) -> Option<ProviderKind> {
    match provider {
        Provider::Apple => Some(ProviderKind::Apple),
        Provider::Google => Some(ProviderKind::Google),
        Provider::Github => Some(ProviderKind::Github),
        Provider::Other(_) => None,
    }
}

/// The providers the server offers, and the account's linked ones the
/// server no longer offers, each with whether it is linked. Names this
/// app doesn't know are left out.
pub(crate) fn provider_rows(methods: &AccountMethods) -> Vec<LinkedProviderRow> {
    let linked = |provider: &Provider| {
        methods
            .linked
            .iter()
            .any(|linked| &linked.provider == provider)
    };
    let mut rows: Vec<LinkedProviderRow> = Vec::new();
    let others = methods.linked.iter().map(|linked| &linked.provider);
    for provider in methods.available.iter().chain(others) {
        let Some(kind) = kind_of(provider) else {
            continue;
        };
        if rows.iter().all(|row| row.provider != kind) {
            rows.push(LinkedProviderRow {
                provider: kind,
                linked: linked(provider),
            });
        }
    }
    rows
}

/// A passkey's credential ID as the page and the calls that change a
/// passkey name it: unpadded base64url.
pub(crate) fn credential_id_text(credential_id: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(credential_id)
}

/// The account's passkeys as the page lists them, oldest first.
pub(crate) fn passkey_rows(passkeys: &[PasskeyInfo]) -> Vec<PasskeyRow> {
    let mut rows: Vec<PasskeyRow> = passkeys
        .iter()
        .map(|passkey| PasskeyRow {
            id: credential_id_text(&passkey.credential_id),
            label: match passkey.manager() {
                Some(manager) => format!("Passkey in {manager}"),
                None => "Passkey".to_owned(),
            },
            added_ms: passkey.created_at,
            last_used_ms: passkey.last_used_at,
            can_unlock: passkey.prf,
        })
        .collect();
    rows.sort_by(|a, b| a.added_ms.cmp(&b.added_ms).then_with(|| a.id.cmp(&b.id)));
    rows
}

#[cfg(test)]
mod tests;
