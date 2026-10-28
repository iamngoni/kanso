//! When a sync runs and what its answer means (docs/ACCOUNTS.md ›
//! Services).
//!
//! [`Scheduler`] decides: a sync starts, queues behind the one running, or
//! is refused while a problem holds automatic syncs back, and a finished
//! sync says what to show, what to hold and whether to run again. The
//! caller owns the thread, the timers and the window: it asks the scheduler,
//! does what the answer says and reports back, passing the time in
//! milliseconds.

use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use kanso_store::{Change, Origin, Store};
use kanso_sync::keychain::PendingRecoveryKey;
use kanso_sync::{
    Backend, RecoveryKey, Schedule, Session, SignOutReason, SyncError, SyncOptions, SyncProblem,
    SyncReport,
};

use crate::Secrets;

pub mod model;
pub mod words;

pub use model::{AccountState, StatusLine, Tone};

/// How long after the last local edit to sync.
pub const AFTER_EDIT: Duration = Duration::from_secs(20);

/// The shortest time between automatic syncs.
const SHORTEST_INTERVAL_SECS: u64 = 60;

/// The time between automatic syncs for a setting of `secs` seconds: never
/// under a minute.
pub fn interval(secs: u64) -> Duration {
    Duration::from_secs(secs.max(SHORTEST_INTERVAL_SECS))
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum SyncState {
    /// Signed out, or signed in and not synced yet.
    #[default]
    Off,
    Syncing,
    Synced {
        at_ms: i64,
        /// Conflicts in that sync; the losing versions are in history.
        conflicts: usize,
        /// The documents that had them, to show and open the note.
        conflicted: Vec<String>,
        /// A problem the sync finished with (`report_problem`).
        problem: Option<SyncProblem>,
    },
    Failed(SyncProblem),
}

/// What keeps automatic syncs from running after a problem (the problems
/// table's last column). "Sync now" always runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hold {
    /// Until this time (ms): a paused account.
    Until(i64),
    /// Until Sync now.
    UntilSyncNow,
    /// Until the user signs in again.
    Stopped,
}

/// What the schedule does after `problem`.
pub fn hold_after(problem: Option<&SyncProblem>) -> Option<Hold> {
    match problem?.schedule() {
        Schedule::KeepsRunning => None,
        Schedule::WaitsUntil(until) => Some(Hold::Until(until)),
        Schedule::StopsUntilSyncNow => Some(Hold::UntilSyncNow),
        Schedule::Stops => Some(Hold::Stopped),
    }
}

/// Whether `hold` keeps an automatic sync from running at `now_ms`.
pub fn holds_at(hold: Option<Hold>, now_ms: i64) -> bool {
    match hold {
        None => false,
        Some(Hold::Until(until)) => now_ms < until,
        Some(Hold::UntilSyncNow | Hold::Stopped) => true,
    }
}

/// What the status shows after a sync: `None` for one that was cancelled,
/// which changes nothing.
pub fn sync_outcome(result: Result<SyncReport, SyncError>, now_ms: i64) -> Option<SyncState> {
    match result {
        Ok(report) => Some(SyncState::Synced {
            at_ms: now_ms,
            conflicts: report.conflicts(),
            problem: kanso_sync::report_problem(&report),
            conflicted: report.conflicted,
        }),
        Err(err) => kanso_sync::problem(&err).map(SyncState::Failed),
    }
}

/// Whether a sync changed what the account holds on the server: it pushed,
/// uploaded, or pulled what another Mac pushed.
pub fn changed_storage(report: &SyncReport) -> bool {
    report.pushed > 0 || report.files_up > 0 || report.pulled > 0
}

/// The problem a sync state shows, if any.
pub fn state_problem(state: &SyncState) -> Option<&SyncProblem> {
    match state {
        SyncState::Failed(problem)
        | SyncState::Synced {
            problem: Some(problem),
            ..
        } => Some(problem),
        _ => None,
    }
}

/// Whether a change should be sent to the server soon. Changes that came
/// from sync, and bulk imports, wait for the regular sync.
pub fn is_local_edit(change: &Change) -> bool {
    matches!(change.origin, Origin::Ui | Origin::Api)
}

/// What to do with a request for a sync.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Request {
    /// Start one now; report back with [`Scheduler::finished`] (or
    /// [`Scheduler::start_failed`] when it couldn't start).
    Start,
    /// One is running: this one runs right after it.
    Queued,
    /// Nothing runs: an account task is busy, or a problem holds automatic
    /// syncs.
    Refused,
}

/// What a finished sync leaves to do. Everything is its default for a sync
/// of a session that is no longer the current one.
#[derive(Debug, Default, PartialEq)]
pub struct Finished {
    /// The status to show; `None` when it stays as it is.
    pub state: Option<SyncState>,
    /// The sync was cancelled: whatever cancelled it says what happens
    /// next, and a status still reading "Syncing" goes back to off.
    pub cancelled: bool,
    /// The server ended the session: sign out for this reason.
    pub signed_out: Option<SignOutReason>,
    /// Another device's changes were merged with a fresh start of the
    /// library: tell the person.
    pub remerged: bool,
    /// What the server holds changed: the account's storage is fetched
    /// again.
    pub storage_changed: bool,
    /// A sync asked for while this one ran: `Some(true)` if any of the
    /// requests was Sync now. Request it again.
    pub again: Option<bool>,
}

/// Decides when syncs run: at most one at a time, a request during a sync
/// runs right after it, and a problem holds automatic syncs back until it
/// has passed. Pure state; the caller runs the sync and tells it how it
/// went.
#[derive(Debug, Default)]
pub struct Scheduler {
    hold: Option<Hold>,
    running: bool,
    /// A sync asked for while one ran, to run right after it: `Some(true)`
    /// when any of the requests was Sync now.
    again: Option<bool>,
}

impl Scheduler {
    /// A sync is asked for: `manual` for Sync now, which a hold never stops
    /// (and lifts); `busy` while an account task runs. Call it once a
    /// session and a server are in hand.
    pub fn request(&mut self, manual: bool, busy: bool, now_ms: i64) -> Request {
        if busy || (!manual && holds_at(self.hold, now_ms)) {
            return Request::Refused;
        }
        if manual {
            self.hold = None;
        }
        if self.running {
            self.again = Some(queued_again(self.again, manual));
            return Request::Queued;
        }
        self.running = true;
        Request::Start
    }

    /// The sync [`Request::Start`] allowed couldn't start.
    pub fn start_failed(&mut self) {
        self.running = false;
    }

    /// A sync ended. `current` says its session is still the signed-in one;
    /// `result` is its answer, `None` when its thread died.
    pub fn finished(
        &mut self,
        current: bool,
        result: Option<Result<SyncReport, SyncError>>,
        now_ms: i64,
    ) -> Finished {
        self.running = false;
        let again = self.again.take();
        if !current {
            // Signed out or in again meanwhile: this sync's news is stale,
            // but a sync asked for meanwhile is the current session's.
            return Finished {
                again,
                ..Finished::default()
            };
        }
        let remerged = matches!(&result, Some(Ok(report)) if report.remerged);
        // What the server holds changed: the Account page's storage line is
        // fetched again. A sync that moved nothing asks for nothing (each
        // account has 120 requests a minute).
        let storage_changed = matches!(&result, Some(Ok(report)) if changed_storage(report));
        let outcome = match result {
            Some(result) => sync_outcome(result, now_ms),
            None => Some(SyncState::Failed(SyncProblem::Other)),
        };
        match outcome {
            None => Finished {
                cancelled: true,
                remerged,
                storage_changed,
                again,
                ..Finished::default()
            },
            Some(state) => {
                self.hold = hold_after(state_problem(&state));
                if let SyncState::Failed(SyncProblem::SignedOut(reason)) = state {
                    // Signing out ends syncing: nothing queued runs.
                    return Finished {
                        signed_out: Some(reason),
                        ..Finished::default()
                    };
                }
                Finished {
                    state: Some(state),
                    remerged,
                    storage_changed,
                    again,
                    ..Finished::default()
                }
            }
        }
    }

    /// Whether a sync is running.
    pub fn running(&self) -> bool {
        self.running
    }

    /// What keeps automatic syncs from running now, if anything.
    pub fn hold(&self) -> Option<Hold> {
        self.hold
    }

    pub fn set_hold(&mut self, hold: Option<Hold>) {
        self.hold = hold;
    }

    /// Lifts the hold: a new session, or none.
    pub fn clear_hold(&mut self) {
        self.hold = None;
    }
}

/// The sync to run after the current one, given what was queued and a new
/// request: manual if any request was.
fn queued_again(queued: Option<bool>, manual: bool) -> bool {
    queued.unwrap_or(false) || manual
}

/// One sync, on the thread that calls it: first any logouts the server
/// hasn't heard (`logouts` holds the secrets and the library's ID), then the
/// sync itself. Setting `cancel` stops it between pages, batches and files.
pub fn run_once(
    store: &Store,
    backend: &dyn Backend,
    session: &Session,
    logouts: Option<(&dyn Secrets, &str)>,
    cancel: Arc<AtomicBool>,
) -> Result<SyncReport, SyncError> {
    if let Some((secrets, library_id)) = logouts {
        kanso_sync::flush_pending_logouts(backend, secrets, library_id);
    }
    let options = SyncOptions {
        cancel: Some(cancel),
        ..SyncOptions::default()
    };
    kanso_sync::sync(store, backend, session, &options)
}

/// Whether `kept` is a recovery key the server no longer holds, because the
/// session of `origin`'s `account_id` ended with `reason`: another device
/// recovered the account or changed its password, and either replaced the
/// key. A key the server took before that is stale. One whose request went
/// unanswered stays: the server may have taken it after, so it may be the
/// only working key (sec-client 3).
pub fn stale_key(
    kept: &PendingRecoveryKey,
    reason: SignOutReason,
    origin: &str,
    account_id: &str,
) -> bool {
    matches!(
        reason,
        SignOutReason::PasswordChanged | SignOutReason::Recovered
    ) && !kept.unconfirmed
        && kept.origin == origin
        && kept.account_id == account_id
}

/// Deletes the library's unsaved recovery key when it is [`stale_key`];
/// returns the key it deleted.
pub fn drop_stale_key(
    secrets: &dyn Secrets,
    library_id: &str,
    reason: SignOutReason,
    ended: Option<&Session>,
) -> Option<RecoveryKey> {
    let ended = ended?;
    let kept = match kanso_sync::keychain::load_any_recovery_key(secrets, library_id) {
        Ok(kept) => kept?,
        Err(err) => {
            eprintln!("kanso: reading the unsaved recovery key: {err}");
            return None;
        }
    };
    if !stale_key(&kept, reason, &ended.origin, &ended.account_id) {
        return None;
    }
    match kanso_sync::keychain::delete_recovery_key(secrets, library_id) {
        Ok(()) => Some(kept.recovery_key),
        Err(err) => {
            eprintln!("kanso: removing a recovery key replaced elsewhere: {err}");
            None
        }
    }
}

/// What the Keychain forgets once the server ended this device's session
/// (a 401): the session item (the ended one, never one saved since), and,
/// unless the end belongs to this device's own recovery, an unsaved
/// recovery key the server no longer holds. Returns that key, for the
/// screen showing it to drop.
pub fn forget_ended_session(
    secrets: &dyn Secrets,
    library_id: &str,
    reason: SignOutReason,
    ended: Option<&Session>,
    own_recovery: bool,
) -> Option<RecoveryKey> {
    let deleted = match ended {
        Some(ended) => kanso_sync::keychain::delete_ended_session(secrets, library_id, ended),
        None => kanso_sync::keychain::delete_session(secrets, library_id),
    };
    if let Err(err) = deleted {
        eprintln!("kanso: couldn't remove the ended session: {err}");
    }
    if own_recovery {
        return None;
    }
    drop_stale_key(secrets, library_id, reason, ended)
}

#[cfg(test)]
mod tests {
    use kanso_sync::{ApiError, RecoveryKey};
    use proptest::prelude::*;

    use super::*;
    use crate::strategies;

    /// A library on `server` for `username`, signed in from Mac `n` and
    /// bound fresh: its store, session and folder.
    fn library(
        server: &kanso_sync::MemoryServer,
        username: &str,
        n: u8,
        sign_up: bool,
    ) -> (Store, Session, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("k.db")).unwrap();
        let device = kanso_sync::DeviceIdentity::from_parts(
            &format!("l_{n:032x}"),
            &format!("Mac {n}"),
            [n; 32],
        );
        let password = kanso_sync::Password::new("correct horse battery", username).unwrap();
        let server_url = kanso_sync::DEFAULT_SERVER;
        let session = if sign_up {
            kanso_sync::sign_up(
                server,
                &kanso_sync::MemorySecrets::new(),
                server_url,
                username,
                &password,
                &device,
                &|_| {},
            )
            .unwrap()
            .session
        } else {
            kanso_sync::sign_in(server, server_url, username, &password, &device, &|_| {})
                .unwrap()
                .session
        };
        kanso_sync::bind(
            &store,
            &session,
            kanso_sync::Choice::Add,
            &dir.path().join("backups"),
            0,
        )
        .unwrap();
        (store, session, dir)
    }

    /// End to end through the Worker's rules (in memory): two libraries of
    /// one account sync through `run_once`, encrypted, and the status
    /// reflects the result.
    #[test]
    fn syncing_two_libraries_reports_synced() {
        use kanso_core::DocType;
        use kanso_store::Put;
        let server = kanso_sync::MemoryServer::new();
        let (a, a_session, _a_dir) = library(&server, "alice", 1, true);
        let (b, b_session, _b_dir) = library(&server, "alice", 2, false);
        let book = a
            .put(
                DocType::Book,
                serde_json::json!({"name": "B"}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
        let note = a
            .put(
                DocType::Note,
                serde_json::json!({"title": "hello", "bookId": book.id}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
        let cancel = || Arc::new(AtomicBool::new(false));
        let state = sync_outcome(run_once(&a, &server, &a_session, None, cancel()), 7);
        assert_eq!(
            state,
            Some(SyncState::Synced {
                at_ms: 7,
                conflicts: 0,
                conflicted: Vec::new(),
                problem: None,
            })
        );
        // The server holds both, under opaque IDs.
        let view = server.plain_view(&a_session);
        assert_eq!(view.docs().len(), 2);
        assert!(
            view.wire_docs()
                .iter()
                .all(|doc| doc.id != note.id && doc.id != book.id)
        );

        let state = sync_outcome(run_once(&b, &server, &b_session, None, cancel()), 8);
        assert!(matches!(
            state,
            Some(SyncState::Synced { problem: None, .. })
        ));
        assert_eq!(b.note(&note.id).unwrap().unwrap().title, "hello");
    }

    #[test]
    fn a_sync_set_to_cancel_ends_cancelled() {
        let server = kanso_sync::MemoryServer::new();
        let (a, session, _dir) = library(&server, "alice", 1, true);
        let cancel = Arc::new(AtomicBool::new(true));
        let result = run_once(&a, &server, &session, None, cancel);
        assert!(matches!(result, Err(SyncError::Cancelled)), "{result:?}");
        assert_eq!(sync_outcome(result, 1), None);
    }

    #[test]
    fn sync_errors_become_problems() {
        assert_eq!(sync_outcome(Err(SyncError::Cancelled), 1), None);
        assert_eq!(
            sync_outcome(Err(SyncError::WrongKey), 1),
            Some(SyncState::Failed(SyncProblem::WrongKey))
        );
        assert_eq!(
            sync_outcome(
                Err(SyncError::Api(ApiError::Unauthorized {
                    reason: SignOutReason::PasswordChanged
                })),
                1
            ),
            Some(SyncState::Failed(SyncProblem::SignedOut(
                SignOutReason::PasswordChanged
            )))
        );
        assert_eq!(
            sync_outcome(Err(SyncError::Api(ApiError::Transport("reset".into()))), 1),
            Some(SyncState::Failed(SyncProblem::Offline))
        );
        assert_eq!(
            sync_outcome(Err(SyncError::Api(ApiError::NotFound)), 1),
            Some(SyncState::Failed(SyncProblem::ServerOutdated))
        );
        // A clean sync has no problem, and holds nothing back.
        let clean = sync_outcome(Ok(SyncReport::default()), 7).unwrap();
        assert_eq!(
            clean,
            SyncState::Synced {
                at_ms: 7,
                conflicts: 0,
                conflicted: Vec::new(),
                problem: None,
            }
        );
        assert_eq!(hold_after(state_problem(&clean)), None);
    }

    #[test]
    fn only_local_edits_trigger_a_quick_sync() {
        let change = |origin| Change {
            id: "note:a".into(),
            rev: "1-a".into(),
            seq: 1,
            deleted: false,
            origin,
        };
        assert!(is_local_edit(&change(Origin::Ui)));
        assert!(is_local_edit(&change(Origin::Api)));
        assert!(!is_local_edit(&change(Origin::Sync)));
        assert!(!is_local_edit(&change(Origin::Import)));
    }

    #[test]
    fn the_interval_is_never_under_a_minute() {
        assert_eq!(interval(0), Duration::from_secs(60));
        assert_eq!(interval(59), Duration::from_secs(60));
        assert_eq!(interval(300), Duration::from_secs(300));
    }

    /// How a sync ended, in the cases whose holds the problems table fixes.
    #[derive(Clone, Copy, Debug)]
    enum Ending {
        Clean,
        Remerged,
        Pushed,
        Cancelled,
        Died,
        BackOff(i64),
        WrongKey,
        Outdated,
        SignedOut,
    }

    impl Ending {
        fn result(self) -> Option<Result<SyncReport, SyncError>> {
            Some(match self {
                Ending::Clean => Ok(SyncReport::default()),
                Ending::Remerged => Ok(SyncReport {
                    remerged: true,
                    ..SyncReport::default()
                }),
                Ending::Pushed => Ok(SyncReport {
                    pushed: 1,
                    ..SyncReport::default()
                }),
                Ending::Cancelled => Err(SyncError::Cancelled),
                Ending::Died => return None,
                Ending::BackOff(until) => Err(SyncError::BackOff { until }),
                Ending::WrongKey => Err(SyncError::WrongKey),
                Ending::Outdated => Err(SyncError::Api(ApiError::NotFound)),
                Ending::SignedOut => Err(SyncError::Api(ApiError::Unauthorized {
                    reason: SignOutReason::Recovered,
                })),
            })
        }

        /// The hold it leaves, from the problems table; `None` leaves the
        /// old one when cancelled.
        fn hold(self, old: Option<Hold>) -> Option<Hold> {
            match self {
                Ending::Cancelled => old,
                Ending::BackOff(until) => Some(Hold::Until(until)),
                Ending::WrongKey | Ending::SignedOut => Some(Hold::Stopped),
                Ending::Outdated => Some(Hold::UntilSyncNow),
                Ending::Clean | Ending::Remerged | Ending::Pushed | Ending::Died => None,
            }
        }
    }

    #[derive(Clone, Debug)]
    enum Op {
        Request {
            manual: bool,
            busy: bool,
            now: i64,
        },
        Finish {
            current: bool,
            ending: Ending,
            now: i64,
        },
    }

    fn ending() -> impl Strategy<Value = Ending> {
        prop_oneof![
            Just(Ending::Clean),
            Just(Ending::Remerged),
            Just(Ending::Pushed),
            Just(Ending::Cancelled),
            Just(Ending::Died),
            (0i64..2_000).prop_map(Ending::BackOff),
            Just(Ending::WrongKey),
            Just(Ending::Outdated),
            Just(Ending::SignedOut),
        ]
    }

    fn ops() -> impl Strategy<Value = Vec<Op>> {
        let op = prop_oneof![
            (any::<bool>(), any::<bool>(), 0i64..2_000)
                .prop_map(|(manual, busy, now)| Op::Request { manual, busy, now }),
            (any::<bool>(), ending(), 0i64..2_000).prop_map(|(current, ending, now)| Op::Finish {
                current,
                ending,
                now
            }),
        ];
        prop::collection::vec(op, 0..60)
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// A sync asked for during another runs once after it, manual if
        /// any of the requests was.
        #[test]
        fn prop_queued_syncs_are_manual_only_if_one_was(requests in prop::collection::vec(any::<bool>(), 1..12)) {
            let queued = requests
                .iter()
                .fold(None, |queued, &manual| Some(queued_again(queued, manual)));
            prop_assert_eq!(queued, Some(requests.iter().any(|&manual| manual)));
        }

        /// Sec-client 3: an unsaved key is dropped only when the session of
        /// its own account ended because another device replaced the key,
        /// and only when the server had taken it.
        #[test]
        fn prop_only_a_key_replaced_elsewhere_is_stale(
            reason in prop::sample::select(SignOutReason::ALL.to_vec()),
            unconfirmed: bool,
            same_origin: bool,
            same_account: bool,
        ) {
            let kept = PendingRecoveryKey {
                origin: "https://a.example".into(),
                account_id: "acct".into(),
                username: "alice".into(),
                server: "https://a.example".into(),
                recovery_key: RecoveryKey::from_bytes([7; 16]),
                unconfirmed,
            };
            let origin = if same_origin { "https://a.example" } else { "https://b.example" };
            let account = if same_account { "acct" } else { "other" };
            let replaced = matches!(reason, SignOutReason::PasswordChanged | SignOutReason::Recovered);
            prop_assert_eq!(
                stale_key(&kept, reason, origin, account),
                replaced && !unconfirmed && same_origin && same_account
            );
        }

        /// The schedule after a problem follows the problems table: what
        /// keeps running never holds, a pause holds exactly until its end,
        /// and the rest hold until someone acts.
        #[test]
        fn prop_holds_follow_the_problems_table(problem in strategies::problem(), now in 0i64..2_000_000) {
            let hold = hold_after(Some(&problem));
            match problem.schedule() {
                Schedule::KeepsRunning => prop_assert_eq!(hold, None),
                Schedule::WaitsUntil(until) => {
                    prop_assert_eq!(hold, Some(Hold::Until(until)));
                    prop_assert_eq!(holds_at(hold, now), now < until);
                }
                Schedule::StopsUntilSyncNow => prop_assert_eq!(hold, Some(Hold::UntilSyncNow)),
                Schedule::Stops => prop_assert_eq!(hold, Some(Hold::Stopped)),
            }
            if hold.is_some() && !matches!(hold, Some(Hold::Until(_))) {
                prop_assert!(holds_at(hold, now));
            }
            prop_assert_eq!(hold_after(None), None);
            prop_assert!(!holds_at(None, now));
        }

        /// A finished sync is Synced, with the problem `report_problem`
        /// finds; a failed one shows `problem`'s; a cancelled one changes
        /// nothing.
        #[test]
        fn prop_sync_outcomes_follow_the_report(
            conflicts_local in 0usize..3,
            conflicts_remote in 0usize..3,
            skipped in 0usize..3,
            too_new: bool,
            paused in prop::option::of(0i64..1_000),
            at in 0i64..1_000_000,
        ) {
            let report = SyncReport {
                conflicts_local,
                conflicts_remote,
                skipped: (0..skipped).map(|n| format!("note:{n}")).collect(),
                too_new,
                paused_until: paused,
                ..SyncReport::default()
            };
            let state = sync_outcome(Ok(report.clone()), at).unwrap();
            let SyncState::Synced { at_ms, conflicts, problem, .. } = &state else {
                panic!("{state:?}");
            };
            prop_assert_eq!(*at_ms, at);
            prop_assert_eq!(*conflicts, conflicts_local + conflicts_remote);
            prop_assert_eq!(problem.clone(), kanso_sync::report_problem(&report));
            prop_assert_eq!(state_problem(&state), problem.as_ref());
        }

        /// Model: against a plain list of what was asked, the scheduler
        /// never starts a sync while one runs; queues what asked meanwhile,
        /// manual if any was; refuses an automatic one only while a hold
        /// says so, and a manual one only while busy; lifts the hold on a
        /// manual one; drops what is queued when the session ends; and
        /// still runs what a stale session's sync left queued.
        #[test]
        fn prop_scheduler_follows_its_model(ops in ops()) {
            let mut scheduler = Scheduler::default();
            let mut running = false;
            let mut queued: Vec<bool> = Vec::new();
            let mut hold: Option<Hold> = None;
            for op in ops {
                match op {
                    Op::Request { manual, busy, now } => {
                        let held = match hold {
                            Some(Hold::Until(until)) => now < until,
                            Some(_) => true,
                            None => false,
                        };
                        let got = scheduler.request(manual, busy, now);
                        if busy || (!manual && held) {
                            prop_assert_eq!(got, Request::Refused);
                        } else {
                            if manual {
                                hold = None;
                            }
                            if running {
                                queued.push(manual);
                                prop_assert_eq!(got, Request::Queued);
                            } else {
                                running = true;
                                prop_assert_eq!(got, Request::Start);
                            }
                        }
                    }
                    Op::Finish { current, ending, now } => {
                        if !running {
                            // Only a sync that started can finish.
                            continue;
                        }
                        running = false;
                        let again = if queued.is_empty() {
                            None
                        } else {
                            Some(queued.iter().any(|&manual| manual))
                        };
                        queued.clear();
                        let got = scheduler.finished(current, ending.result(), now);
                        if !current {
                            prop_assert_eq!(got, Finished { again, ..Finished::default() });
                        } else {
                            hold = ending.hold(hold);
                            match ending {
                                Ending::SignedOut => {
                                    prop_assert_eq!(got.signed_out, Some(SignOutReason::Recovered));
                                    prop_assert_eq!(got.again, None);
                                    prop_assert_eq!(got.state, None);
                                }
                                Ending::Cancelled => {
                                    prop_assert!(got.cancelled);
                                    prop_assert_eq!(got.state, None);
                                    prop_assert_eq!(got.again, again);
                                }
                                _ => {
                                    prop_assert!(!got.cancelled && got.signed_out.is_none());
                                    prop_assert!(got.state.is_some());
                                    prop_assert_eq!(got.again, again);
                                    prop_assert_eq!(got.remerged, matches!(ending, Ending::Remerged));
                                    prop_assert_eq!(got.storage_changed, matches!(ending, Ending::Pushed));
                                }
                            }
                        }
                    }
                }
                prop_assert_eq!(scheduler.running(), running);
                prop_assert_eq!(scheduler.hold(), hold);
            }
        }
    }
}
