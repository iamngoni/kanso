//! The account's page: devices, password, recovery key, passkeys, providers
//! and deleting the account.

use crate::frb_generated::StreamSink;
use flutter_rust_bridge::frb;
use kanso_mobile::{
    AccountDetails, AccountSettings, KansoError, ProviderEvent, ProviderKind, RecoveryKeyText,
};

#[frb(opaque)]
pub struct KansoAccountSettings(AccountSettings);

impl KansoAccountSettings {
    pub(super) fn new(settings: AccountSettings) -> KansoAccountSettings {
        KansoAccountSettings(settings)
    }

    pub fn details(&self) -> Result<AccountDetails, KansoError> {
        self.0.details()
    }

    pub fn revoke_device(&self, device_id: String) -> Result<AccountDetails, KansoError> {
        self.0.revoke_device(device_id)
    }

    pub fn sign_out(&self) -> Result<(), KansoError> {
        self.0.sign_out()
    }

    pub fn change_password(
        &self,
        current: String,
        new_password: String,
    ) -> Result<RecoveryKeyText, KansoError> {
        self.0.change_password(current, new_password)
    }

    pub fn new_recovery_key(
        &self,
        current_password: String,
    ) -> Result<RecoveryKeyText, KansoError> {
        self.0.new_recovery_key(current_password)
    }

    pub fn pending_recovery_key(&self) -> Result<Option<RecoveryKeyText>, KansoError> {
        self.0.pending_recovery_key()
    }

    pub fn delete_account(
        &self,
        password: String,
        typed_username: String,
    ) -> Result<(), KansoError> {
        self.0.delete_account(password, typed_username)
    }

    pub fn add_passkey(&self, current_password: String) -> Result<AccountDetails, KansoError> {
        self.0.add_passkey(current_password)
    }

    pub fn remove_passkey(
        &self,
        id: String,
        current_password: String,
    ) -> Result<AccountDetails, KansoError> {
        self.0.remove_passkey(id, current_password)
    }

    /// The steps of linking a provider, until it ends or Dart stops
    /// listening.
    pub fn link_provider(
        &self,
        provider: ProviderKind,
        current_password: String,
        sink: StreamSink<ProviderEvent>,
    ) {
        self.0
            .link_provider(provider, current_password, move |event| {
                sink.add(event).is_ok()
            });
    }

    pub fn unlink_provider(
        &self,
        provider: ProviderKind,
        current_password: String,
    ) -> Result<AccountDetails, KansoError> {
        self.0.unlink_provider(provider, current_password)
    }
}
