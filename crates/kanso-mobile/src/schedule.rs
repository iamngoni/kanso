//! The sync schedule while the app is in front (docs/ACCOUNTS.md ›
//! Services): once when the app comes to the front, [`AFTER_EDIT`] after a
//! local edit, and every [`interval`] after the last one, honouring what the
//! last problem holds back, and shared notes after every account sync that
//! worked. Behind, nothing runs: the system's background jobs
//! (`background.rs`) carry on from there.
//!
//! The sync settings decide the rest: the interval is the person's choice,
//! and with "sync automatically" off the account is not synced on its own at
//! all (not on opening, after an edit or on the interval), as the desktop's
//! `apply_sync_settings` does; shared notes still sync on the interval, so
//! invitations and mentions come, and Sync now always works.
//!
//! [`Driver`] is the rules, pure and told the time, so a test can walk it
//! through any order of edits, trips to the back and ticks.
//! [`AutoSync`] is what a library keeps: the driver, and the two threads
//! that serve it. `kanso-sync-auto` waits for the driver's next deadline and
//! runs the sync; `kanso-sync-edits` hears the store's local edits. Both
//! end with the library, and neither is started before the app first comes
//! to the front.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, Weak};
use std::time::Duration;

use kanso_core::now_ms;
use kanso_engine::sync::{AFTER_EDIT, Finished, Request, Scheduler, interval, is_local_edit};
use kanso_store::Change;
use kanso_sync::{SyncError, SyncReport, meta_keys};

use crate::boundary::{held, panic_message};
use crate::library::Inner;
use crate::sharing::SharedNotes;
use crate::sync_model::SyncSettingsRow;
use crate::syncing::{Ran, SyncHandle};

/// How long to wait before looking again whether an account is signed in,
/// while none is.
const SIGNED_OUT_RECHECK_MS: i64 = 5_000;

/// The longest the scheduler thread sleeps with nothing due.
const IDLE_WAIT: Duration = Duration::from_secs(60);

/// How long after an edit, and how often, the schedule syncs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Timing {
    pub(crate) after_edit_ms: i64,
    pub(crate) interval_ms: i64,
}

impl Timing {
    /// The delays for these sync settings.
    pub(crate) fn of(settings: SyncSettingsRow) -> Timing {
        Timing {
            after_edit_ms: millis(AFTER_EDIT),
            interval_ms: millis(interval(u64::from(settings.interval_secs))),
        }
    }
}

impl Default for Timing {
    fn default() -> Timing {
        Timing::of(SyncSettingsRow::default())
    }
}

fn millis(duration: Duration) -> i64 {
    i64::try_from(duration.as_millis()).unwrap_or(i64::MAX)
}

/// The rules of the schedule. Pure state: the caller says what happened and
/// when (milliseconds), and asks whether a sync is due.
#[derive(Debug, Default)]
pub(crate) struct Driver {
    scheduler: Scheduler,
    timing: Timing,
    foreground: bool,
    /// "Sync automatically": with it off nothing starts an account sync on
    /// its own.
    enabled: bool,
    /// Only the shared notes are due: an interval came while automatic sync
    /// is off.
    shared_due: bool,
    /// A sync is wanted at once: the app came to the front, or one was
    /// asked for while another ran.
    now: bool,
    /// A local edit is waiting for its delay to pass.
    edit_due: Option<i64>,
    interval_due: Option<i64>,
}

impl Driver {
    pub(crate) fn new(timing: Timing) -> Driver {
        Driver {
            timing,
            enabled: true,
            ..Driver::default()
        }
    }

    /// The sync settings changed: new delays and whether sync runs on its
    /// own. Like the desktop's, the schedule starts over: in front, it is
    /// due at once (the account if automatic sync is on, else the shared
    /// notes) and then on the new interval.
    pub(crate) fn set_settings(&mut self, timing: Timing, enabled: bool, now: i64) {
        self.timing = timing;
        self.enabled = enabled;
        self.edit_due = None;
        if self.foreground {
            self.now = true;
            self.interval_due = Some(now + timing.interval_ms);
        }
    }

    #[cfg(any(test, feature = "test-support"))]
    pub(crate) fn set_timing(&mut self, timing: Timing, now: i64) {
        self.timing = timing;
        if self.foreground {
            self.edit_due = self.edit_due.map(|_| now + timing.after_edit_ms);
            self.interval_due = Some(now + timing.interval_ms);
        }
    }

    /// The app came to the front (`true`): sync now, then on the interval.
    /// It went behind: nothing is due any more. A sync that is running is
    /// the caller's to stop.
    pub(crate) fn set_foreground(&mut self, foreground: bool, now: i64) {
        if foreground == self.foreground {
            return;
        }
        self.foreground = foreground;
        self.edit_due = None;
        if foreground {
            self.now = true;
            self.interval_due = Some(now + self.timing.interval_ms);
        } else {
            self.now = false;
            self.interval_due = None;
        }
    }

    /// A local edit: a sync in [`Timing::after_edit_ms`]. A second edit
    /// before then moves it, as the desktop's does.
    pub(crate) fn edited(&mut self, now: i64) {
        if self.foreground && self.enabled {
            self.edit_due = Some(now + self.timing.after_edit_ms);
        }
    }

    /// Whether anything has come due. Takes nothing; [`Driver::poll`] does.
    pub(crate) fn is_due(&self, now: i64) -> bool {
        self.foreground
            && (self.now
                || self.edit_due.is_some_and(|at| at <= now)
                || self.interval_due.is_some_and(|at| at <= now))
    }

    /// When the next deadline falls, if there is one.
    pub(crate) fn next_wake(&self) -> Option<i64> {
        if !self.foreground {
            return None;
        }
        self.edit_due.into_iter().chain(self.interval_due).min()
    }

    /// Takes what has come due and says whether a sync starts now. Never in
    /// the background, never signed out, never while one runs (a request
    /// then runs right after it), never while a problem holds the schedule
    /// back.
    pub(crate) fn poll(&mut self, signed_in: bool, now: i64) -> bool {
        if !self.is_due(now) {
            return false;
        }
        self.now = false;
        self.edit_due = None;
        if self.interval_due.is_some_and(|at| at <= now) {
            self.interval_due = Some(now + self.timing.interval_ms);
        }
        if !signed_in {
            // Look again soon: signing in is the shell's call, not ours.
            self.interval_due = Some(now + SIGNED_OUT_RECHECK_MS.min(self.timing.interval_ms));
            return false;
        }
        if !self.enabled {
            self.shared_due = true;
            return false;
        }
        matches!(self.scheduler.request(false, false, now), Request::Start)
    }

    /// Takes whether the shared notes alone are due, as they are on the
    /// interval while automatic sync is off.
    pub(crate) fn take_shared_due(&mut self) -> bool {
        std::mem::take(&mut self.shared_due)
    }

    /// The sync [`Driver::poll`] started could not run, or was refused by
    /// the library (another sync held it): `retry` asks again after the
    /// edit delay.
    pub(crate) fn start_failed(&mut self, retry: bool, now: i64) {
        self.scheduler.start_failed();
        if retry && self.foreground {
            self.edit_due = Some(now + self.timing.after_edit_ms);
        }
    }

    /// The sync ended with `result` (`None` when its thread died). Answers
    /// as the desktop's scheduler does; a sync asked for meanwhile runs next
    /// if the app is still in front.
    pub(crate) fn finished(
        &mut self,
        result: Option<Result<SyncReport, SyncError>>,
        now: i64,
    ) -> Finished {
        let finished = self.scheduler.finished(true, result, now);
        if finished.again.is_some() && finished.signed_out.is_none() && self.foreground {
            self.now = true;
        }
        finished
    }

    /// Whether a sync this driver started is still running.
    pub(crate) fn running(&self) -> bool {
        self.scheduler.running()
    }

    /// Whether the app is in front.
    pub(crate) fn in_front(&self) -> bool {
        self.foreground
    }
}

/// What the scheduler thread and the library share.
struct Shared {
    state: Mutex<State>,
    wake: Condvar,
}

struct State {
    driver: Driver,
    /// Set by the library when it closes: the threads end.
    closed: bool,
    /// Stops the sync this schedule started, if one runs.
    cancel: Option<Arc<AtomicBool>>,
    started: bool,
}

/// The schedule of one library.
pub(crate) struct AutoSync {
    shared: Arc<Shared>,
}

impl AutoSync {
    /// The schedule for a library with these sync settings.
    pub(crate) fn new(settings: SyncSettingsRow) -> AutoSync {
        let mut driver = Driver::new(Timing::of(settings));
        driver.enabled = settings.enabled;
        AutoSync {
            shared: Arc::new(Shared {
                state: Mutex::new(State {
                    driver,
                    closed: false,
                    cancel: None,
                    started: false,
                }),
                wake: Condvar::new(),
            }),
        }
    }
}

impl Default for AutoSync {
    fn default() -> AutoSync {
        AutoSync::new(SyncSettingsRow::default())
    }
}

impl Drop for AutoSync {
    fn drop(&mut self) {
        held(&self.shared.state).closed = true;
        self.shared.wake.notify_all();
    }
}

impl AutoSync {
    /// The app came to the front or went behind. Starts the threads the
    /// first time it is in front.
    pub(crate) fn set_foreground(&self, library: &Arc<Inner>, foreground: bool) {
        let mut state = held(&self.shared.state);
        state.driver.set_foreground(foreground, now_ms());
        if !foreground && let Some(cancel) = state.cancel.as_ref() {
            cancel.store(true, Ordering::SeqCst);
        }
        if foreground && !state.started {
            state.started = true;
            self.start(library);
        }
        drop(state);
        self.shared.wake.notify_all();
    }

    /// The sync settings changed: the schedule follows at once.
    pub(crate) fn apply_settings(&self, settings: SyncSettingsRow) {
        held(&self.shared.state).driver.set_settings(
            Timing::of(settings),
            settings.enabled,
            now_ms(),
        );
        self.shared.wake.notify_all();
    }

    /// Different delays, for tests that can't wait for the real ones.
    #[cfg(any(test, feature = "test-support"))]
    pub(crate) fn set_timing(&self, timing: Timing) {
        held(&self.shared.state).driver.set_timing(timing, now_ms());
        self.shared.wake.notify_all();
    }

    fn start(&self, library: &Arc<Inner>) {
        let changes = library.store.subscribe();
        let edits = self.shared.clone();
        if let Err(err) = std::thread::Builder::new()
            .name("kanso-sync-edits".into())
            .spawn(move || hear_edits(&edits, &changes))
        {
            eprintln!("kanso: couldn't start the edit listener: {err}");
        }
        let shared = self.shared.clone();
        let library = Arc::downgrade(library);
        if let Err(err) = std::thread::Builder::new()
            .name("kanso-sync-auto".into())
            .spawn(move || run(&shared, &library))
        {
            eprintln!("kanso: couldn't start the sync schedule: {err}");
        }
    }
}

/// Tells the driver about every local edit the store reports, until the
/// store goes or the library closes.
fn hear_edits(shared: &Shared, changes: &async_channel::Receiver<Change>) {
    while let Ok(change) = changes.recv_blocking() {
        if !is_local_edit(&change) {
            continue;
        }
        let mut state = held(&shared.state);
        if state.closed {
            return;
        }
        state.driver.edited(now_ms());
        drop(state);
        shared.wake.notify_all();
    }
}

/// The scheduler thread.
fn run(shared: &Shared, library: &Weak<Inner>) {
    loop {
        let mut state = held(&shared.state);
        loop {
            if state.closed {
                return;
            }
            let now = now_ms();
            if state.driver.is_due(now) {
                break;
            }
            let wait = state.driver.next_wake().map_or(IDLE_WAIT, |at| {
                Duration::from_millis(u64::try_from((at - now).max(1)).unwrap_or(1))
            });
            state = shared
                .wake
                .wait_timeout(state, wait.min(IDLE_WAIT))
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .0;
        }
        drop(state);
        let Some(inner) = library.upgrade() else {
            return;
        };
        // A bug in one round must not end the schedule.
        let round = catch_unwind(AssertUnwindSafe(|| round(shared, &inner)));
        if let Err(payload) = round {
            eprintln!(
                "kanso: the sync schedule's round panicked and was caught: {}",
                panic_message(payload.as_ref())
            );
            let mut state = held(&shared.state);
            state.cancel = None;
            if state.driver.running() {
                state.driver.start_failed(false, now_ms());
            }
        }
    }
}

/// One due moment: sync if the rules allow, then the shared notes.
fn round(shared: &Shared, inner: &Arc<Inner>) {
    let signed_in = matches!(inner.store.meta(meta_keys::ACCOUNT_ID), Ok(Some(_)));
    let cancel = {
        let mut state = held(&shared.state);
        if !state.driver.poll(signed_in, now_ms()) {
            let shared_only = state.driver.take_shared_due();
            drop(state);
            if shared_only {
                sync_shared(inner);
            }
            return;
        }
        let cancel = Arc::new(AtomicBool::new(false));
        state.cancel = Some(cancel.clone());
        cancel
    };
    let ran = SyncHandle {
        inner: inner.clone(),
    }
    .run(cancel);
    let mut worked = false;
    {
        let mut state = held(&shared.state);
        state.cancel = None;
        let now = now_ms();
        match ran {
            Ran::Done(report) => {
                worked = true;
                state.driver.finished(Some(Ok(*report)), now);
            }
            Ran::Failed(err) => {
                state.driver.finished(Some(Err(err)), now);
            }
            // A sync of someone else's (Sync now) holds the library: it
            // covers what this one would; ask again after the delay.
            Ran::Queued => state.driver.start_failed(true, now),
            Ran::NotSignedIn | Ran::SecretsUnavailable(_) => {
                state.driver.start_failed(false, now);
            }
        }
        worked &= state.driver.in_front();
    }
    if worked {
        sync_shared(inner);
    }
}

/// Invitations, notes shared with this account, their logs and mentions come
/// with the account's own sync, and on the interval while automatic sync is
/// off. Offline or signed out, there is nothing to say: the next round tries
/// again.
fn sync_shared(inner: &Arc<Inner>) {
    let _ = crate::boundary::guard(|| {
        SharedNotes {
            inner: inner.clone(),
        }
        .sync_shared()
    });
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    const TIMING: Timing = Timing {
        after_edit_ms: 20,
        interval_ms: 300,
    };

    /// What the rules were told, in any order.
    #[derive(Clone, Debug)]
    enum Step {
        Edit,
        Foreground,
        Background,
        Tick(i64),
        /// The running sync ends: well, or with a problem.
        Finish(Ending),
        SignIn(bool),
        /// "Sync automatically" turned on or off.
        Automatic(bool),
    }

    #[derive(Clone, Copy, Debug)]
    enum Ending {
        Well,
        Cancelled,
        BackOff,
        Died,
    }

    fn step() -> impl Strategy<Value = Step> {
        prop_oneof![
            3 => Just(Step::Edit),
            2 => Just(Step::Foreground),
            2 => Just(Step::Background),
            6 => (0i64..400).prop_map(Step::Tick),
            3 => prop_oneof![
                Just(Ending::Well),
                Just(Ending::Cancelled),
                Just(Ending::BackOff),
                Just(Ending::Died),
            ]
            .prop_map(Step::Finish),
            1 => any::<bool>().prop_map(Step::SignIn),
            1 => any::<bool>().prop_map(Step::Automatic),
        ]
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(96))]

        /// Whatever happens in whatever order: a sync starts only with the
        /// app in front and an account signed in, never while one runs, and
        /// every sync that starts is one the model has not already running.
        #[test]
        fn prop_at_most_one_sync_and_none_behind_or_signed_out(
            steps in prop::collection::vec(step(), 1..80)
        ) {
            let mut driver = Driver::new(TIMING);
            let mut now = 1_000i64;
            let mut foreground = false;
            let mut signed_in = true;
            let mut automatic = true;
            let mut running = false;
            let mut starts = 0usize;
            for step in steps {
                match step {
                    Step::Edit => driver.edited(now),
                    Step::Foreground => {
                        foreground = true;
                        driver.set_foreground(true, now);
                    }
                    Step::Background => {
                        foreground = false;
                        driver.set_foreground(false, now);
                    }
                    Step::SignIn(state) => signed_in = state,
                    Step::Automatic(on) => {
                        automatic = on;
                        driver.set_settings(TIMING, on, now);
                    }
                    Step::Tick(by) => {
                        now += by;
                        if driver.poll(signed_in, now) {
                            prop_assert!(foreground, "a sync started behind");
                            prop_assert!(signed_in, "a sync started signed out");
                            prop_assert!(automatic, "an automatic sync started while it is off");
                            prop_assert!(!running, "two syncs at once");
                            running = true;
                            starts += 1;
                        }
                    }
                    Step::Finish(ending) => {
                        if running {
                            let result = match ending {
                                Ending::Well => Some(Ok(SyncReport::default())),
                                Ending::Cancelled => Some(Err(SyncError::Cancelled)),
                                Ending::BackOff => {
                                    Some(Err(SyncError::BackOff { until: now + 10_000 }))
                                }
                                Ending::Died => None,
                            };
                            driver.finished(result, now);
                            running = false;
                        }
                    }
                }
                prop_assert_eq!(driver.running(), running);
                if !foreground {
                    prop_assert!(!driver.is_due(now), "something is due behind");
                    prop_assert_eq!(driver.next_wake(), None);
                }
            }
            // Nothing started more often than the app could have asked.
            prop_assert!(starts <= 80);
        }

        /// A burst of edits makes one sync, [`Timing::after_edit_ms`] after
        /// the last of them; one edit alone, after its own delay.
        #[test]
        fn prop_a_burst_of_edits_is_one_sync_after_the_last(
            gaps in prop::collection::vec(0i64..20, 1..12)
        ) {
            let mut driver = Driver::new(TIMING);
            driver.set_foreground(true, 0);
            // The sync the front starts, finished.
            prop_assert!(driver.poll(true, 0));
            driver.finished(Some(Ok(SyncReport::default())), 0);
            let mut now = 0;
            for gap in &gaps {
                now += gap;
                driver.edited(now);
                // Nothing is due before the delay has passed.
                prop_assert!(!driver.poll(true, now));
            }
            prop_assert!(!driver.poll(true, now + TIMING.after_edit_ms - 1));
            prop_assert!(driver.poll(true, now + TIMING.after_edit_ms));
            driver.finished(Some(Ok(SyncReport::default())), now + TIMING.after_edit_ms);
            prop_assert!(!driver.poll(true, now + TIMING.after_edit_ms + 1));
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(96))]

        /// The interval it is given is the interval it keeps: after the
        /// sync the front starts, the next is due exactly one interval
        /// later, and a change of setting moves it to the new interval from
        /// then.
        #[test]
        fn prop_the_schedule_keeps_the_interval_it_is_given(
            first in prop::sample::select(kanso_engine::sync::words::SYNC_INTERVALS.to_vec()),
            second in prop::sample::select(kanso_engine::sync::words::SYNC_INTERVALS.to_vec()),
        ) {
            let settings = |minutes: u64| SyncSettingsRow {
                enabled: true,
                interval_secs: u32::try_from(minutes * 60).unwrap(),
            };
            let timing = Timing::of(settings(first));
            prop_assert_eq!(timing.interval_ms, i64::try_from(first * 60_000).unwrap());
            let mut driver = Driver::new(timing);
            driver.set_foreground(true, 0);
            prop_assert!(driver.poll(true, 0));
            driver.finished(Some(Ok(SyncReport::default())), 0);
            prop_assert!(!driver.poll(true, timing.interval_ms - 1));
            prop_assert!(driver.poll(true, timing.interval_ms));
            driver.finished(Some(Ok(SyncReport::default())), timing.interval_ms);
            // The setting changes: due at once, then on the new interval.
            let now = timing.interval_ms + 10;
            let changed = Timing::of(settings(second));
            driver.set_settings(changed, true, now);
            prop_assert!(driver.poll(true, now));
            driver.finished(Some(Ok(SyncReport::default())), now);
            prop_assert!(!driver.poll(true, now + changed.interval_ms - 1));
            prop_assert!(driver.poll(true, now + changed.interval_ms));
        }

        /// With automatic sync off nothing starts on its own however long
        /// the app stays in front or whatever is edited, the shared notes
        /// are due on the interval, and turning it on again syncs at once.
        #[test]
        fn prop_a_schedule_that_is_off_never_syncs_by_itself(
            minutes in prop::sample::select(kanso_engine::sync::words::SYNC_INTERVALS.to_vec()),
            edits in prop::collection::vec(0i64..100_000, 0..10),
            rounds in 1usize..6,
        ) {
            let timing = Timing::of(SyncSettingsRow {
                enabled: false,
                interval_secs: u32::try_from(minutes * 60).unwrap(),
            });
            let mut driver = Driver::new(timing);
            driver.set_settings(timing, false, 0);
            driver.set_foreground(true, 0);
            let mut now = 0;
            // The front is due at once, for the shared notes only.
            prop_assert!(!driver.poll(true, now));
            prop_assert!(driver.take_shared_due());
            for at in edits {
                driver.edited(now + at);
            }
            for _ in 0..rounds {
                now += timing.interval_ms;
                prop_assert!(!driver.poll(true, now), "started while off");
                prop_assert!(driver.take_shared_due(), "the shared notes missed their interval");
                prop_assert!(!driver.take_shared_due());
            }
            driver.set_settings(timing, true, now);
            prop_assert!(driver.poll(true, now));
        }
    }

    // no-prop-test: one walk through the interval, the signed-out look-again
    // and a held schedule; the generated order is covered above.
    #[test]
    fn the_interval_repeats_and_a_hold_keeps_automatic_syncs_back() {
        let mut driver = Driver::new(TIMING);
        driver.set_foreground(true, 0);
        assert!(driver.poll(true, 0), "the front syncs at once");
        driver.finished(Some(Ok(SyncReport::default())), 0);
        assert!(!driver.poll(true, 299));
        assert!(driver.poll(true, 300), "and again on the interval");
        // A server asking for time holds automatic syncs until then.
        driver.finished(Some(Err(SyncError::BackOff { until: 5_000 })), 300);
        assert!(!driver.poll(true, 600), "held");
        driver.edited(700);
        assert!(!driver.poll(true, 720), "an edit does not lift a hold");
        assert!(driver.poll(true, 5_000 + 300), "after the wait it runs");
        driver.finished(Some(Ok(SyncReport::default())), 5_300);
        // Signed out: nothing starts, and it looks again in a few seconds.
        assert!(!driver.poll(false, 5_600));
        let again = 5_600 + SIGNED_OUT_RECHECK_MS.min(TIMING.interval_ms);
        assert_eq!(driver.next_wake(), Some(again));
        assert!(driver.poll(true, again));
    }

    // no-prop-test: a request that arrives while a sync runs, in a fixed
    // order; the queue's rules are the engine scheduler's.
    #[test]
    fn an_edit_during_a_sync_syncs_again_when_it_ends_and_not_behind() {
        let mut driver = Driver::new(TIMING);
        driver.set_foreground(true, 0);
        assert!(driver.poll(true, 0));
        driver.edited(10);
        assert!(!driver.poll(true, 30), "one runs: this one waits");
        driver.finished(Some(Ok(SyncReport::default())), 40);
        assert!(driver.poll(true, 40), "and runs right after it");
        // Behind, the same request is forgotten.
        driver.edited(50);
        assert!(!driver.poll(true, 70));
        driver.set_foreground(false, 80);
        driver.finished(Some(Ok(SyncReport::default())), 90);
        assert!(!driver.is_due(10_000));
    }
}
