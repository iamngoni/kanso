//! Push notifications: telling the server where to wake this phone.
//!
//! The shell asks the OS for a token and hands it here ([`SyncHandle::set_push_token`]);
//! this crate keeps it in the library's own settings and registers it with the
//! account's server whenever the phone is signed in and the person hasn't
//! turned pushes off. The token is the OS's, not a secret of the account; the
//! session never leaves the secret store, and the settings below hold only a
//! one-way label of the login a token was registered for.
//!
//! Registering is best effort. A failed registration is an error from the
//! explicit `set_*` calls and is ignored by sign-in; [`SyncHandle::refresh_push`]
//! tries again. The server forgets a token when the session ends, so signing
//! out only forgets that it was registered, not the token itself.

use kanso_sync::{PushEnvironment, PushPlatform, PushRegistration, Session};

use kanso_engine::account::words::ErrorContext;

use crate::boundary::guard;
use crate::errors::KansoError;
use crate::model::{PushEnvironmentKind, PushPlatformKind};
use crate::syncing::{SyncHandle, account_error};

/// The library's push settings (its `meta` table).
pub(crate) mod keys {
    /// The OS token, as given.
    pub const TOKEN: &str = "push.token";
    /// Its service: `apns` or `fcm`.
    pub const PLATFORM: &str = "push.platform";
    /// Its environment: `production` or `development`.
    pub const ENVIRONMENT: &str = "push.environment";
    /// `0` when the person turned pushes off; absent or `1` is on.
    pub const ENABLED: &str = "push.enabled";
    /// [`kanso_sync::Session::login_label`] of the login the token was last
    /// registered for; empty or absent when the server holds none.
    pub const REGISTERED_FOR: &str = "push.registered_for";
}

impl SyncHandle {
    /// Remembers this phone's push `token` and, when signed in with pushes on,
    /// registers it with the server. A token already registered for this
    /// login is not sent again.
    ///
    /// [`KansoError::Invalid`] when the token's shape is one the server would
    /// refuse (the token is not stored); [`KansoError::Network`] and the
    /// like when the server couldn't be told. The token is remembered
    /// either way, and a later [`SyncHandle::refresh_push`] retries.
    pub fn set_push_token(
        &self,
        platform: PushPlatformKind,
        token: String,
        environment: PushEnvironmentKind,
    ) -> Result<(), KansoError> {
        guard(|| {
            let registration = PushRegistration {
                platform: wire_platform(platform),
                token,
                environment: wire_environment(environment),
            };
            if !registration.is_valid() {
                return Err(KansoError::invalid(
                    "That isn't a push token Kanso's server accepts.",
                ));
            }
            if self.remembered_push()?.as_ref() != Some(&registration) {
                self.remember_push(&registration)?;
                // A different token is not the one the server holds.
                self.set_push_meta(keys::REGISTERED_FOR, "")?;
            }
            self.refresh_push_inner()
        })
    }

    /// Turns pushes on or off for this phone, and remembers the choice. Off
    /// removes the token from the server (when signed in); on registers the
    /// remembered token again. Pushes are on until this is called.
    pub fn set_push_enabled(&self, enabled: bool) -> Result<(), KansoError> {
        guard(|| {
            self.set_push_meta(keys::ENABLED, if enabled { "1" } else { "0" })?;
            self.refresh_push_inner()
        })
    }

    /// Whether pushes are on. On when never set.
    pub fn push_enabled(&self) -> Result<bool, KansoError> {
        guard(|| self.push_is_enabled())
    }

    /// Brings the server's copy of this phone's push token in line with the
    /// settings: registers the remembered token when pushes are on and it
    /// isn't registered for this login yet, and removes it when they are off
    /// and it still is. Does nothing when signed out.
    ///
    /// Every path that ends in a signed-in session calls this
    /// ([`SyncHandle::sign_in_password`] does, ignoring its result so a
    /// network failure never fails a sign-in); the shell calls it again when
    /// the OS hands out a new token or the app starts, which is the retry.
    pub fn refresh_push(&self) -> Result<(), KansoError> {
        guard(|| self.refresh_push_inner())
    }

    /// [`SyncHandle::refresh_push`] for a sign-in: the same, but its result is
    /// dropped.
    pub(crate) fn refresh_push_quietly(&self) {
        let _ = self.refresh_push();
    }

    /// Forgets that the server holds a token: it dropped its copy with the
    /// session. The remembered OS token stays.
    pub(crate) fn forget_push_registration(&self) -> Result<(), KansoError> {
        self.set_push_meta(keys::REGISTERED_FOR, "")
    }

    fn refresh_push_inner(&self) -> Result<(), KansoError> {
        let Some(session) = self.load_session()? else {
            return Ok(());
        };
        let label = session.login_label();
        let registered = self.push_meta(keys::REGISTERED_FOR)?.as_deref() == Some(label.as_str());
        if !self.push_is_enabled()? {
            if registered {
                self.unregister_push(&session)?;
            }
            return Ok(());
        }
        let Some(registration) = self.remembered_push()? else {
            return Ok(());
        };
        if registered {
            return Ok(());
        }
        kanso_sync::register_push(self.inner.backend.as_ref(), &session, &registration)
            .map_err(|err| account_error(&err, ErrorContext::Account))?;
        self.set_push_meta(keys::REGISTERED_FOR, &label)
    }

    fn unregister_push(&self, session: &Session) -> Result<(), KansoError> {
        kanso_sync::unregister_push(self.inner.backend.as_ref(), session)
            .map_err(|err| account_error(&err, ErrorContext::Account))?;
        self.forget_push_registration()
    }

    fn push_is_enabled(&self) -> Result<bool, KansoError> {
        Ok(self.push_meta(keys::ENABLED)?.as_deref() != Some("0"))
    }

    /// The remembered token, when all three settings are there and valid.
    fn remembered_push(&self) -> Result<Option<PushRegistration>, KansoError> {
        let (Some(token), Some(platform), Some(environment)) = (
            self.push_meta(keys::TOKEN)?,
            self.push_meta(keys::PLATFORM)?,
            self.push_meta(keys::ENVIRONMENT)?,
        ) else {
            return Ok(None);
        };
        let platform = match platform.as_str() {
            "apns" => PushPlatform::Apns,
            "fcm" => PushPlatform::Fcm,
            _ => return Ok(None),
        };
        let environment = match environment.as_str() {
            "production" => PushEnvironment::Production,
            "development" => PushEnvironment::Development,
            _ => return Ok(None),
        };
        let registration = PushRegistration {
            platform,
            token,
            environment,
        };
        Ok(registration.is_valid().then_some(registration))
    }

    fn remember_push(&self, registration: &PushRegistration) -> Result<(), KansoError> {
        self.set_push_meta(keys::TOKEN, &registration.token)?;
        self.set_push_meta(
            keys::PLATFORM,
            match registration.platform {
                PushPlatform::Apns => "apns",
                PushPlatform::Fcm => "fcm",
            },
        )?;
        self.set_push_meta(
            keys::ENVIRONMENT,
            match registration.environment {
                PushEnvironment::Production => "production",
                PushEnvironment::Development => "development",
            },
        )
    }

    /// A setting; an empty one is no setting (the store has no delete).
    fn push_meta(&self, key: &str) -> Result<Option<String>, KansoError> {
        Ok(self
            .inner
            .store
            .meta(key)
            .map_err(KansoError::from)?
            .filter(|value| !value.is_empty()))
    }

    fn set_push_meta(&self, key: &str, value: &str) -> Result<(), KansoError> {
        self.inner
            .store
            .set_meta(key, value)
            .map_err(KansoError::from)
    }
}

fn wire_platform(kind: PushPlatformKind) -> PushPlatform {
    match kind {
        PushPlatformKind::Apns => PushPlatform::Apns,
        PushPlatformKind::Fcm => PushPlatform::Fcm,
    }
}

fn wire_environment(kind: PushEnvironmentKind) -> PushEnvironment {
    match kind {
        PushEnvironmentKind::Production => PushEnvironment::Production,
        PushEnvironmentKind::Development => PushEnvironment::Development,
    }
}
