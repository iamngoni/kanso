//! Signing in and up, the locked account, recovery and device approval as
//! the new device.

use crate::frb_generated::StreamSink;
use flutter_rust_bridge::frb;
use kanso_mobile::{
    AccountHandle, AccountState, ApprovalWait, BindingAnswer, KansoError, PasskeyFailure,
    PasskeyRequest, ProviderEvent, ProviderKind, RecoveryKeyText, ServerMethods, SignedUp,
};

#[frb(opaque)]
pub struct KansoAccount(AccountHandle);

impl KansoAccount {
    pub(super) fn new(handle: AccountHandle) -> KansoAccount {
        KansoAccount(handle)
    }

    pub fn state(&self) -> Result<AccountState, KansoError> {
        self.0.state()
    }

    /// The state now and each time it changes, until Dart stops listening.
    pub fn state_stream(&self, sink: StreamSink<AccountState>) {
        self.0.state_stream(move |state| sink.add(state).is_ok());
    }

    pub fn server_methods(&self, server: String) -> Result<ServerMethods, KansoError> {
        self.0.server_methods(server)
    }

    pub fn sign_up_password(
        &self,
        server: String,
        username: String,
        password: String,
        device_name: String,
    ) -> Result<SignedUp, KansoError> {
        self.0
            .sign_up_password(server, username, password, device_name)
    }

    pub fn sign_in_password(
        &self,
        server: String,
        username: String,
        password: String,
        device_name: String,
    ) -> Result<AccountState, KansoError> {
        self.0
            .sign_in_password(server, username, password, device_name)
    }

    pub fn unlock_with_password(&self, password: String) -> Result<AccountState, KansoError> {
        self.0.unlock_with_password(password)
    }

    pub fn recover(
        &self,
        server: String,
        username: String,
        recovery_key: String,
        new_password: String,
        device_name: String,
    ) -> Result<SignedUp, KansoError> {
        self.0
            .recover(server, username, recovery_key, new_password, device_name)
    }

    pub fn answer_binding(
        &self,
        answer: Option<BindingAnswer>,
    ) -> Result<AccountState, KansoError> {
        self.0.answer_binding(answer)
    }

    /// The steps of a provider sign-in, until it ends or Dart stops listening.
    pub fn start_provider(
        &self,
        server: String,
        provider: ProviderKind,
        device_name: String,
        sink: StreamSink<ProviderEvent>,
    ) {
        self.0
            .start_provider(server, provider, device_name, move |event| {
                sink.add(event).is_ok()
            });
    }

    pub fn provider_page_ended(&self, callback_url: Option<String>) {
        self.0.provider_page_ended(callback_url);
    }

    pub fn cancel_provider(&self) {
        self.0.cancel_provider();
    }

    pub fn finish_provider_sign_up(&self, username: String) -> Result<SignedUp, KansoError> {
        self.0.finish_provider_sign_up(username)
    }

    /// The passkey sheets the phone is asked to show, until Dart stops
    /// listening.
    pub fn passkey_requests(&self, sink: StreamSink<PasskeyRequest>) {
        self.0
            .passkey_requests(move |request| sink.add(request).is_ok());
    }

    pub fn passkey_failed(&self, ticket: u64, failure: PasskeyFailure) {
        self.0.passkey_failed(ticket, failure);
    }

    pub fn passkey_sign_in(
        &self,
        server: String,
        device_name: String,
    ) -> Result<AccountState, KansoError> {
        self.0.passkey_sign_in(server, device_name)
    }

    pub fn passkey_sign_up(
        &self,
        server: String,
        username: String,
        device_name: String,
    ) -> Result<SignedUp, KansoError> {
        self.0.passkey_sign_up(server, username, device_name)
    }

    /// The wait for another device to approve this one, until it ends or Dart
    /// stops listening.
    pub fn request_approval(&self, device_name: String, sink: StreamSink<ApprovalWait>) {
        self.0
            .request_approval(device_name, move |wait| sink.add(wait).is_ok());
    }

    pub fn cancel_approval(&self) {
        self.0.cancel_approval();
    }

    pub fn cancel_locked(&self) -> Result<AccountState, KansoError> {
        self.0.cancel_locked()
    }

    pub fn pending_recovery_key(&self) -> Result<Option<RecoveryKeyText>, KansoError> {
        self.0.pending_recovery_key()
    }

    pub fn recovery_key_saved(&self) -> Result<(), KansoError> {
        self.0.recovery_key_saved()
    }
}
