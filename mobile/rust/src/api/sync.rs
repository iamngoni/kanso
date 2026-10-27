//! Signing in, sync and the status the sidebar shows.

use crate::frb_generated::StreamSink;
use flutter_rust_bridge::frb;
use kanso_mobile::{
    KansoError, PushEnvironmentKind, PushPlatformKind, ServerInfo, SyncHandle, SyncIntervalOption,
    SyncReportRow, SyncSettingsRow, SyncStatus, SyncStatusLine,
};

#[frb(opaque)]
pub struct KansoSync(SyncHandle);

impl KansoSync {
    pub(super) fn new(handle: SyncHandle) -> KansoSync {
        KansoSync(handle)
    }

    pub fn default_server() -> String {
        SyncHandle::default_server()
    }

    pub fn status(&self) -> Result<SyncStatus, KansoError> {
        self.0.status()
    }

    /// The status now and each time it changes, until Dart stops listening.
    pub fn status_stream(&self, sink: StreamSink<SyncStatus>) {
        self.0.status_stream(move |status| sink.add(status).is_ok());
    }

    pub fn check_server(&self, url: String) -> Result<ServerInfo, KansoError> {
        self.0.check_server(url)
    }

    pub fn sign_in_password(
        &self,
        server: String,
        username: String,
        password: String,
        device_name: String,
    ) -> Result<SyncStatus, KansoError> {
        self.0
            .sign_in_password(server, username, password, device_name)
    }

    pub fn sign_out(&self) -> Result<(), KansoError> {
        self.0.sign_out()
    }

    /// The app is in front (`true`) or not.
    pub fn set_foreground(&self, foreground: bool) {
        self.0.set_foreground(foreground);
    }

    pub fn sync_now(&self) -> Result<SyncReportRow, KansoError> {
        self.0.sync_now()
    }

    /// The status line in the desktop's words. `when` and `resume` are the
    /// shell's words for `last_synced_ms` and `resume_at_ms`.
    #[frb(sync)]
    pub fn status_line(
        status: SyncStatus,
        settings: SyncSettingsRow,
        when: String,
        resume: String,
    ) -> SyncStatusLine {
        SyncHandle::status_line(status, settings, when, resume)
    }

    /// The choices for how often sync runs.
    #[frb(sync)]
    pub fn interval_options() -> Vec<SyncIntervalOption> {
        SyncHandle::interval_options()
    }

    #[frb(sync)]
    pub fn interval_label(minutes: u32) -> String {
        SyncHandle::interval_label(minutes)
    }

    #[frb(sync)]
    pub fn interval_phrase(minutes: u32) -> String {
        SyncHandle::interval_phrase(minutes)
    }

    pub fn settings(&self) -> Result<SyncSettingsRow, KansoError> {
        self.0.settings()
    }

    pub fn set_settings(
        &self,
        enabled: bool,
        interval_secs: u32,
    ) -> Result<SyncSettingsRow, KansoError> {
        self.0.set_settings(enabled, interval_secs)
    }
    /// Hands over the token the OS issued for this phone; it is registered
    /// with the server when signed in and pushes are on.
    pub fn set_push_token(
        &self,
        platform: PushPlatformKind,
        token: String,
        environment: PushEnvironmentKind,
    ) -> Result<(), KansoError> {
        self.0.set_push_token(platform, token, environment)
    }

    pub fn set_push_enabled(&self, enabled: bool) -> Result<(), KansoError> {
        self.0.set_push_enabled(enabled)
    }

    pub fn push_enabled(&self) -> Result<bool, KansoError> {
        self.0.push_enabled()
    }

    /// Registers (or removes) the remembered token to match the settings;
    /// the retry after a failed registration.
    pub fn refresh_push(&self) -> Result<(), KansoError> {
        self.0.refresh_push()
    }
}
