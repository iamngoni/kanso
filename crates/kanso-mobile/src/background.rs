//! The sync that runs with no app: what the system's background jobs (iOS's
//! `BGAppRefreshTask`, Android's `WorkManager`) call, within the time the
//! system gives, with no Dart anywhere in the process.
//!
//! [`run`] opens the library (or reuses the one the app has open, so the two
//! never sync one library at once), reads the saved session from the secret
//! store, and syncs until it is done or the budget is spent. The C entry
//! point that exports it, `kanso_background_sync`, is in `mobile/rust`: the
//! symbol has to be defined in the library the app loads.

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::time::Duration;

use kanso_sync::{Backend, SyncError};

use crate::Secrets;
use crate::boundary::{guard, panic_message};
use crate::errors::KansoError;
use crate::library::Library;
use crate::syncing::Ran;

/// How a background sync ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackgroundOutcome {
    /// Finished; `changed` says whether it pulled or pushed anything.
    Done { changed: bool },
    /// No account is signed in.
    NotSignedIn,
    /// The secret store refused (a locked Keychain, a Keystore error).
    SecretsUnavailable,
    /// The server couldn't be reached, or answered with a problem.
    Network,
    /// The budget ran out first; the sync stopped between steps and what it
    /// did is kept.
    OutOfTime,
    /// A bug: a panic was caught, or the library couldn't be opened.
    Internal,
}

impl BackgroundOutcome {
    /// The number the C entry returns: `0` done and nothing changed, `1`
    /// done and something changed, `-1` not signed in, `-2` secrets
    /// unavailable, `-3` network, `-4` out of time, `-5` internal.
    pub fn code(self) -> i32 {
        match self {
            BackgroundOutcome::Done { changed: false } => 0,
            BackgroundOutcome::Done { changed: true } => 1,
            BackgroundOutcome::NotSignedIn => -1,
            BackgroundOutcome::SecretsUnavailable => -2,
            BackgroundOutcome::Network => -3,
            BackgroundOutcome::OutOfTime => -4,
            BackgroundOutcome::Internal => -5,
        }
    }
}

/// Syncs the library at `path` (its database file) once, in at most
/// `budget`. A library whose sync settings turn automatic sync off is left
/// alone and answers `Done { changed: false }`. A panic anywhere in it is caught and returned as
/// [`BackgroundOutcome::Internal`].
pub fn run(
    path: &Path,
    budget: Duration,
    backend: Arc<dyn Backend>,
    secrets: Arc<dyn Secrets>,
) -> BackgroundOutcome {
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        sync_within(path, budget, backend, secrets)
    }));
    caught.unwrap_or_else(|payload| {
        eprintln!(
            "kanso: the background sync panicked and was caught: {}",
            panic_message(payload.as_ref())
        );
        BackgroundOutcome::Internal
    })
}

fn sync_within(
    path: &Path,
    budget: Duration,
    backend: Arc<dyn Backend>,
    secrets: Arc<dyn Secrets>,
) -> BackgroundOutcome {
    let library = match Library::existing(path) {
        Some(library) => library,
        None => match Library::open_with(path, secrets, backend) {
            Ok(library) => library,
            Err(err) => {
                eprintln!("kanso: the background sync couldn't open the library: {err}");
                return BackgroundOutcome::Internal;
            }
        },
    };
    // With "sync automatically" off, the system's job has nothing to do
    // (Sync now, in the app, still works). A signed-out library is still
    // told so. There is no code of its own for this: it is "done, nothing
    // changed", the C entry's `0`.
    let sync = library.sync();
    if matches!(sync.load_session(), Ok(Some(_)))
        && sync.settings().is_ok_and(|settings| !settings.enabled)
    {
        return BackgroundOutcome::Done { changed: false };
    }
    // An empty budget is spent before it starts.
    let cancel = Arc::new(AtomicBool::new(budget.is_zero()));
    let _deadline = Deadline::after(budget, cancel.clone());
    let ran = guard_ran(|| library.sync().run(cancel.clone()));
    outcome_of(ran, cancel.load(Ordering::SeqCst))
}

/// Runs the sync, catching a panic in it into an internal error.
fn guard_ran(sync: impl FnOnce() -> Ran) -> Result<Ran, KansoError> {
    let mut ran = None;
    guard(|| {
        ran = Some(sync());
        Ok(())
    })?;
    ran.ok_or_else(|| KansoError::Internal {
        message: "the sync didn't run".into(),
    })
}

/// What a finished (or failed) attempt is to the system's job. `spent` says
/// the budget ran out.
fn outcome_of(ran: Result<Ran, KansoError>, spent: bool) -> BackgroundOutcome {
    match ran {
        Err(_) => BackgroundOutcome::Internal,
        Ok(Ran::NotSignedIn) => BackgroundOutcome::NotSignedIn,
        Ok(Ran::SecretsUnavailable(_)) => BackgroundOutcome::SecretsUnavailable,
        // The app is syncing this library right now: nothing for this job.
        Ok(Ran::Queued) => BackgroundOutcome::Done { changed: false },
        Ok(Ran::Done(report)) => BackgroundOutcome::Done {
            changed: report.pulled + report.pushed + report.files_up + report.files_down > 0,
        },
        Ok(Ran::Failed(SyncError::Cancelled)) if spent => BackgroundOutcome::OutOfTime,
        Ok(Ran::Failed(err)) => match kanso_sync::problem(&err) {
            // The server ended this device's session.
            Some(kanso_sync::SyncProblem::SignedOut(_)) => BackgroundOutcome::NotSignedIn,
            _ => BackgroundOutcome::Network,
        },
    }
}

/// Sets a flag once a time has passed, and is dismissed by being dropped.
struct Deadline {
    /// Dropping it wakes the thread to end.
    _done: mpsc::Sender<()>,
}

impl Deadline {
    fn after(budget: Duration, flag: Arc<AtomicBool>) -> Deadline {
        let (done, wait) = mpsc::channel::<()>();
        let spawned = std::thread::Builder::new()
            .name("kanso-deadline".into())
            .spawn(move || {
                // Disconnected means the sync ended first; a timeout means
                // the time is up.
                if wait.recv_timeout(budget) == Err(mpsc::RecvTimeoutError::Timeout) {
                    flag.store(true, Ordering::SeqCst);
                }
            });
        if let Err(err) = spawned {
            eprintln!("kanso: couldn't start the background deadline: {err}");
        }
        Deadline { _done: done }
    }
}

#[cfg(test)]
mod tests {
    use kanso_sync::SyncReport;
    use proptest::prelude::*;

    use super::*;

    proptest! {
        /// The codes are the C entry's contract: each outcome has its own,
        /// and they stay within -5..=1.
        #[test]
        fn prop_a_finished_sync_is_zero_or_one_by_whether_it_changed_anything(
            pulled in 0usize..3, pushed in 0usize..3, up in 0usize..3, down in 0usize..3,
            conflicts in 0usize..3,
        ) {
            let report = SyncReport {
                pulled, pushed, files_up: up, files_down: down,
                conflicts_local: conflicts,
                ..SyncReport::default()
            };
            let changed = pulled + pushed + up + down > 0;
            let outcome = outcome_of(Ok(Ran::Done(Box::new(report))), false);
            prop_assert_eq!(outcome, BackgroundOutcome::Done { changed });
            prop_assert_eq!(outcome.code(), i32::from(changed));
        }
    }

    // no-prop-test: a table of the contract's numbers.
    #[test]
    fn every_outcome_has_its_number() {
        let codes = [
            (BackgroundOutcome::Done { changed: false }, 0),
            (BackgroundOutcome::Done { changed: true }, 1),
            (BackgroundOutcome::NotSignedIn, -1),
            (BackgroundOutcome::SecretsUnavailable, -2),
            (BackgroundOutcome::Network, -3),
            (BackgroundOutcome::OutOfTime, -4),
            (BackgroundOutcome::Internal, -5),
        ];
        for (outcome, code) in codes {
            assert_eq!(outcome.code(), code);
        }
        assert_eq!(
            outcome_of(Ok(Ran::Failed(SyncError::Cancelled)), true),
            BackgroundOutcome::OutOfTime
        );
        assert_eq!(
            outcome_of(Ok(Ran::Failed(SyncError::Cancelled)), false),
            BackgroundOutcome::Network
        );
        assert_eq!(
            outcome_of(Err(KansoError::Locked), false),
            BackgroundOutcome::Internal
        );
    }

    // no-prop-test: the deadline fires once and is dismissed by dropping it.
    #[test]
    fn the_deadline_sets_its_flag_on_time_and_not_when_dismissed() {
        let flag = Arc::new(AtomicBool::new(false));
        let kept = Deadline::after(Duration::from_millis(20), flag.clone());
        std::thread::sleep(Duration::from_millis(300));
        assert!(flag.load(Ordering::SeqCst));
        drop(kept);

        let flag = Arc::new(AtomicBool::new(false));
        drop(Deadline::after(Duration::from_millis(100), flag.clone()));
        std::thread::sleep(Duration::from_millis(300));
        assert!(!flag.load(Ordering::SeqCst));
    }
}
