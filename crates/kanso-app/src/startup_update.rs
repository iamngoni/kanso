//! Updating from the window for a library that won't open.
//!
//! A version that can't open its library can't reach the updater in the
//! app, so the fix would need a hand reinstall. This is the updater without
//! the app: the same feed, signature check, URL policy and staging
//! (`kanso_update`), driven by a small state machine ([`Recovery`]) that
//! needs no library, no store and no settings. [`Backend`] is its seam to
//! the network and the disk; tests use a fake.

use std::sync::atomic::AtomicBool;

use kanso_update::{
    Check, CheckError, Install, NotInstallable, Platform, Transport, Update, UrlPolicy,
};

use crate::updates::{Problem, check_problem, stage_problem};

/// How this copy takes an update.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Way {
    /// Kanso replaces itself and restarts.
    Replace,
    /// apt installed it: show the upgrade command.
    Apt,
    /// It can't replace itself: open the release page.
    Page,
}

impl Way {
    pub fn of(install: &Result<Install, NotInstallable>) -> Way {
        match install {
            Ok(_) => Way::Replace,
            Err(NotInstallable::PackageManaged) => Way::Apt,
            Err(_) => Way::Page,
        }
    }
}

/// Where the window's update section is.
#[derive(Clone, Debug, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum Phase {
    Checking,
    Available(Update),
    Latest,
    /// The check failed; Retry checks again.
    CheckFailed(Problem),
    Installing {
        done: u64,
        total: u64,
    },
    /// The download or install failed; Retry checks again.
    InstallFailed(Problem),
}

/// What the check and the install need from outside.
pub trait Backend: Send + Sync + 'static {
    fn check(&self) -> Result<Check, CheckError>;
    /// Downloads, verifies, swaps in and restarts into `update`. On success
    /// the caller ends the window.
    fn install(
        &self,
        update: &Update,
        progress: &mut dyn FnMut(u64, u64),
        cancel: &AtomicBool,
    ) -> Result<(), Problem>;
}

/// The state machine.
#[derive(Clone, Debug, PartialEq)]
pub struct Recovery {
    pub phase: Phase,
    pub way: Way,
    /// The error is a bug in this version, so the window owns it.
    pub bug: bool,
    pub current: String,
}

impl Recovery {
    pub fn new(way: Way, bug: bool, current: &semver::Version) -> Self {
        Recovery {
            phase: Phase::Checking,
            way,
            bug,
            current: current.to_string(),
        }
    }

    /// Retry, or the first check.
    pub fn begin_check(&mut self) {
        self.phase = Phase::Checking;
    }

    pub fn checked(&mut self, result: Result<Check, CheckError>) {
        if !matches!(self.phase, Phase::Checking) {
            return;
        }
        self.phase = match result {
            Ok(Check::UpToDate { .. }) => Phase::Latest,
            Ok(Check::Available(update)) => Phase::Available(update),
            Err(err) => Phase::CheckFailed(check_problem(&err)),
        };
    }

    /// "Update now": the update to install, when one is waiting and this
    /// copy can replace itself.
    pub fn update_now(&mut self) -> Option<Update> {
        match (&self.phase, self.way) {
            (Phase::Available(update), Way::Replace) => {
                let update = update.clone();
                self.phase = Phase::Installing {
                    done: 0,
                    total: update.asset.size,
                };
                Some(update)
            }
            _ => None,
        }
    }

    pub fn progress(&mut self, now_done: u64, now_total: u64) {
        if let Phase::Installing { done, total } = &mut self.phase {
            *done = now_done;
            *total = now_total;
        }
    }

    pub fn installed(&mut self, result: Result<(), Problem>) {
        if matches!(self.phase, Phase::Installing { .. })
            && let Err(problem) = result
        {
            self.phase = Phase::InstallFailed(problem);
        }
    }

    /// Whether Retry is offered.
    pub fn can_retry(&self) -> bool {
        matches!(self.phase, Phase::CheckFailed(_) | Phase::InstallFailed(_))
    }

    /// The sentence the section opens with.
    pub fn headline(&self) -> String {
        match &self.phase {
            Phase::Checking => "Checking for updates…".into(),
            Phase::Available(update) if self.bug => format!(
                "Sorry — this is a bug in Kanso {}, not a problem with your notes. Your notes are safe. Kanso {} fixes it.",
                self.current, update.version
            ),
            Phase::Available(update) => {
                format!("Kanso {} is available — it may fix this.", update.version)
            }
            Phase::Latest if self.bug => "Sorry — Kanso hit a bug opening your library. Your notes haven't been changed. You're on the latest version.".into(),
            Phase::Latest => "You're on the latest version.".into(),
            Phase::CheckFailed(problem) => {
                format!("Couldn't check for updates. {}", problem.words())
            }
            Phase::Installing { done, total } => format!(
                "Downloading the update… {}%",
                crate::updates::percent(*done, *total)
            ),
            Phase::InstallFailed(problem) => {
                format!("The update didn't install. {}", problem.words())
            }
        }
    }
}

/// The real thing: the app's own feed, keys and policy, and this install.
pub struct Real {
    pub transport: Box<dyn Transport>,
    pub policy: UrlPolicy,
    pub feed_url: String,
    pub keys: Vec<kanso_update::VerifyingKey>,
    pub install: Result<Install, NotInstallable>,
    pub platform: Platform,
    pub current: semver::Version,
    /// Opened again with it, as the app does after an update.
    pub data_dir: String,
}

impl Real {
    /// `None` when this build never updates (Kanso Dev, no key, no builds
    /// for this system): the window then says nothing about updates.
    pub fn new(data_dir: &str) -> Option<Real> {
        let current = crate::updates::current_version();
        let keys = kanso_update::trusted_keys();
        let platform = Platform::current();
        if crate::updates::off_reason(&crate::flavour::CURRENT, !keys.is_empty(), platform)
            .is_some()
        {
            return None;
        }
        let mut policy = UrlPolicy::RELEASE;
        let mut feed_url = kanso_update::FEED_URL.to_owned();
        if cfg!(debug_assertions)
            && let Some(url) = std::env::var("KANSO_UPDATE_FEED")
                .ok()
                .filter(|u| !u.is_empty())
        {
            feed_url = url;
            policy.allow_loopback = true;
        }
        Some(Real {
            transport: Box::new(kanso_update::HttpTransport::new(&format!(
                "Kanso/{current}"
            ))),
            policy,
            feed_url,
            keys,
            install: crate::updates::locate_install(platform),
            platform: platform?,
            current,
            data_dir: data_dir.to_owned(),
        })
    }
}

impl Backend for Real {
    fn check(&self) -> Result<Check, CheckError> {
        kanso_update::check(
            self.transport.as_ref(),
            &self.policy,
            &self.feed_url,
            &self.keys,
            &self.current,
            self.platform,
        )
    }

    fn install(
        &self,
        update: &Update,
        progress: &mut dyn FnMut(u64, u64),
        cancel: &AtomicBool,
    ) -> Result<(), Problem> {
        let install = self.install.as_ref().map_err(|_| Problem::Other)?;
        let verify = |payload: &std::path::Path| -> Result<(), String> {
            #[cfg(target_os = "macos")]
            return kanso_update::install::verify_bundle(payload, Some(&install.target));
            #[cfg(not(target_os = "macos"))]
            {
                let _ = payload;
                Ok(())
            }
        };
        let pending = kanso_update::stage::prepare(
            self.transport.as_ref(),
            &self.policy,
            update,
            install,
            progress,
            cancel,
            &verify,
        )
        .map_err(|err| stage_problem(&err))?;
        kanso_update::install::apply(&pending, install).map_err(|_| Problem::Other)?;
        #[cfg(target_os = "macos")]
        kanso_update::install::remove_quarantine(&install.target);
        let args = ["--data-dir".to_owned(), self.data_dir.clone()];
        kanso_update::install::relaunch(install, &args).map_err(|_| Problem::Other)
    }
}

#[cfg(test)]
#[path = "startup_update_tests.rs"]
mod tests;
