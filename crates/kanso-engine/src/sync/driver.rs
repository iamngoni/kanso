//! Sync without a window: the loop `kanso serve` and the app run
//! (docs/ACCOUNTS.md › Services). It does with plain threads: a sync at
//! start, one every `interval`, one `after_edit` after the last local edit,
//! and Sync now; [`Scheduler`] decides each time.
//!
//! One control thread owns the timers and starts the syncs. A sync runs on a
//! worker thread of its own, at most one at a time, and reports back to the
//! control thread; a feed thread turns the store's local edits into wakes.
//! The owner hears what happened through [`DriverEvent`]s, called on the
//! control thread, and asks for syncs with [`Driver::request`], which
//! answers once the control thread has decided.

use std::io;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, PoisonError, mpsc};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use kanso_store::{Change, Store};
use kanso_sync::{Backend, Session, SignOutReason, SyncError, SyncProblem, SyncReport};
use serde::{Deserialize, Serialize};

use super::{
    Hold, Request, Scheduler, forget_ended_session, is_local_edit, run_once, state_problem,
};
use crate::Secrets;

/// When the last sync that finished ended (ms), written after each sync.
pub const LAST_SYNC_MS: &str = "sync.v2.last_sync_ms";
/// How the last sync that finished went, as [`LastResult`] JSON.
pub const LAST_RESULT: &str = "sync.v2.last_result";

/// What the driver needs of the world; the app and `kanso serve` fill it.
pub struct DriverSeams {
    pub store: Arc<Store>,
    pub backend: Arc<dyn Backend>,
    pub secrets: Arc<dyn Secrets>,
    pub library_id: String,
    pub session: Arc<Session>,
    /// [`super::interval`] of the setting.
    pub interval: Duration,
    /// [`super::AFTER_EDIT`]; tests shorten it.
    pub after_edit: Duration,
    pub now_ms: Arc<dyn Fn() -> i64 + Send + Sync>,
    /// Run on the sync thread after each successful library sync and on
    /// each timer tick (shared notes in `kanso serve`: the collab engine's
    /// `sync_all`). kanso-engine can't depend on kanso-collab, hence a hook.
    /// An `Err` is reported as [`DriverEvent::Problem`] with
    /// [`SyncProblem::Other`], then [`DriverEvent::HookFailed`] with its
    /// text.
    pub after_sync: Option<Arc<dyn Fn() -> Result<(), String> + Send + Sync>>,
}

/// How a driver starts. [`Driver::start`] takes the default: automatic,
/// nothing held, not busy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DriverStart {
    /// Syncs on its own: at start, every `interval` and `after_edit` after
    /// a local edit. Without, only [`Driver::request`]s run (the app's
    /// Sync now with Sync automatically off).
    pub automatic: bool,
    /// The hold to start with: a driver replacing one of the same session
    /// keeps its [`Driver::hold`].
    pub hold: Option<Hold>,
    /// Whether an account task runs at start ([`Driver::set_busy`]).
    pub busy: bool,
}

impl Default for DriverStart {
    fn default() -> Self {
        DriverStart {
            automatic: true,
            hold: None,
            busy: false,
        }
    }
}

/// News the owner shows or logs. Carries no secrets. Rare enough that a
/// report kept inline costs nothing.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq)]
pub enum DriverEvent {
    Started,
    /// A library sync started; one of `Synced`, `Problem`, `Cancelled` or
    /// `SignedOut` follows, unless the driver stops first.
    SyncStarted,
    /// A sync finished; the owner decides whether it is worth a line.
    Synced(SyncReport),
    /// A sync failed, or finished with a problem.
    Problem(SyncProblem),
    /// A library sync was cancelled ([`Driver::stop_sync`]): it changed
    /// nothing, and whoever cancelled it says what happens next.
    Cancelled,
    /// Another device's changes were merged with a fresh start of the
    /// library.
    Remerged,
    /// The `after_sync` hook failed, with its text.
    HookFailed(String),
    /// The server ended the session: it is forgotten and syncing stopped.
    SignedOut(SignOutReason),
    Stopped,
}

/// [`LAST_RESULT`]'s JSON: what the last sync moved, and its problem's
/// label ([`super::words::problem_label`]) if it had one.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LastResult {
    pub pulled: usize,
    pub pushed: usize,
    pub files_up: usize,
    pub conflicts: usize,
    pub pending: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub problem: Option<String>,
}

impl LastResult {
    /// The line for a sync that ended with `report` (`None` when it
    /// failed) and `problem`.
    pub fn new(report: Option<&SyncReport>, problem: Option<&SyncProblem>) -> LastResult {
        let mut last = report.map_or_else(LastResult::default, |report| LastResult {
            pulled: report.pulled,
            pushed: report.pushed,
            files_up: report.files_up,
            conflicts: report.conflicts(),
            pending: report.pending,
            problem: None,
        });
        last.problem = problem.map(|p| super::words::problem_label(p).to_owned());
        last
    }
}

/// One sync, given its cancel flag.
type Runner = Arc<dyn Fn(Arc<AtomicBool>) -> Result<SyncReport, SyncError> + Send + Sync>;
type Hook = Arc<dyn Fn() -> Result<(), String> + Send + Sync>;

/// What reaches the control thread.
enum Msg {
    Edit,
    /// A sync asked for (`manual` for Sync now); the answer goes back.
    Request {
        manual: bool,
        answer: mpsc::Sender<Request>,
    },
    /// Cancel the running sync; what waits for it goes back.
    StopSync(mpsc::Sender<Option<SyncStop>>),
    Stop,
    Done(Box<Outcome>),
}

/// What a worker did: the library sync's answer (`Some(None)` when it
/// panicked, `None` when it didn't sync the library) and the hook's.
struct Outcome {
    library: Option<Option<Result<SyncReport, SyncError>>>,
    hook: Option<Result<(), String>>,
}

/// Set once a worker thread is done.
#[derive(Default)]
struct Ended {
    done: Mutex<bool>,
    changed: Condvar,
}

impl Ended {
    fn set(&self) {
        *self.done.lock().unwrap_or_else(PoisonError::into_inner) = true;
        self.changed.notify_all();
    }
}

/// Marks its worker done when the worker's thread ends, panicking or not.
struct EndGuard(Arc<Ended>);

impl Drop for EndGuard {
    fn drop(&mut self) {
        self.0.set();
    }
}

/// A cancelled sync, to wait for ([`Driver::stop_sync`]).
pub struct SyncStop(Arc<Ended>);

impl SyncStop {
    /// Waits until the cancelled sync's thread is done: it stops between
    /// pages, batches and files.
    pub fn join(self) {
        let mut done = self.0.done.lock().unwrap_or_else(PoisonError::into_inner);
        while !*done {
            done = self
                .0
                .changed
                .wait(done)
                .unwrap_or_else(PoisonError::into_inner);
        }
    }
}

/// The running sync loop. Dropping it stops it, as [`Driver::stop`] does.
pub struct Driver {
    stop: Arc<AtomicBool>,
    finished: Arc<AtomicBool>,
    busy: Arc<AtomicBool>,
    scheduler: Arc<Mutex<Scheduler>>,
    wake: mpsc::Sender<Msg>,
    thread: Option<JoinHandle<()>>,
}

impl Driver {
    /// Starts the loop on a thread named `kanso-sync-driver`; `on` is called
    /// on that thread for every event.
    pub fn start(seams: DriverSeams, on: Box<dyn FnMut(DriverEvent) + Send>) -> io::Result<Driver> {
        Driver::start_from(seams, DriverStart::default(), on)
    }

    /// [`Driver::start`] from `start`: the app's settings, and what a
    /// driver it replaces left.
    pub fn start_from(
        seams: DriverSeams,
        start: DriverStart,
        on: Box<dyn FnMut(DriverEvent) + Send>,
    ) -> io::Result<Driver> {
        let store = seams.store.clone();
        let backend = seams.backend.clone();
        let session = seams.session.clone();
        let secrets = seams.secrets.clone();
        let library_id = seams.library_id.clone();
        let runner: Runner = Arc::new(move |cancel| {
            run_once(
                &store,
                backend.as_ref(),
                &session,
                Some((secrets.as_ref(), library_id.as_str())),
                cancel,
            )
        });
        Driver::start_with(seams, start, runner, on)
    }

    /// [`Driver::start_from`] with the sync itself given: tests count and
    /// script it.
    fn start_with(
        seams: DriverSeams,
        start: DriverStart,
        runner: Runner,
        on: Box<dyn FnMut(DriverEvent) + Send>,
    ) -> io::Result<Driver> {
        let stop = Arc::new(AtomicBool::new(false));
        let finished = Arc::new(AtomicBool::new(false));
        let busy = Arc::new(AtomicBool::new(start.busy));
        let mut scheduler = Scheduler::default();
        scheduler.set_hold(start.hold);
        let scheduler = Arc::new(Mutex::new(scheduler));
        let (wake, rx) = mpsc::channel();
        let changes = seams.store.subscribe();
        let feed_changes = changes.clone();
        let feed_wake = wake.clone();
        let feed = std::thread::Builder::new()
            .name("kanso-sync-feed".into())
            .spawn(move || feed_loop(feed_changes, feed_wake))?;
        let mut control = Control {
            hook: seams.after_sync.clone(),
            seams,
            runner,
            on,
            rx,
            tx: wake.clone(),
            stop: stop.clone(),
            finished: finished.clone(),
            busy: busy.clone(),
            automatic: start.automatic,
            scheduler: scheduler.clone(),
            worker: None,
            start_pending: None,
            hook_next: false,
            ended: false,
            next_tick: Instant::now(),
            edit_due: None,
            feed: Some((changes.clone(), feed)),
        };
        let thread = std::thread::Builder::new()
            .name("kanso-sync-driver".into())
            .spawn(move || control.run());
        match thread {
            Ok(thread) => Ok(Driver {
                stop,
                finished,
                busy,
                scheduler,
                wake,
                thread: Some(thread),
            }),
            Err(err) => {
                // The feed thread ends once its channel closes.
                changes.close();
                Err(err)
            }
        }
    }

    /// Sync now: a hold never stops it (and is lifted). Returns the
    /// scheduler's answer.
    pub fn sync_now(&self) -> Request {
        self.request(true)
    }

    /// Asks for a sync, `manual` for Sync now, and waits for the control
    /// thread's answer: a started sync has started, and
    /// [`DriverEvent::SyncStarted`] gone out, when this returns. An ended
    /// or stopped driver refuses.
    pub fn request(&self, manual: bool) -> Request {
        let (answer, answered) = mpsc::channel();
        if self.wake.send(Msg::Request { manual, answer }).is_err() {
            return Request::Refused;
        }
        answered.recv().unwrap_or(Request::Refused)
    }

    /// An account task runs (`true`) or none does: while one runs, nothing
    /// starts, not even Sync now, and a session that ends meanwhile leaves
    /// the unsaved recovery key to the owner (its own recovery may have
    /// made it). A sync already running runs on; [`Driver::stop_sync`]
    /// stops it.
    pub fn set_busy(&self, busy: bool) {
        self.busy.store(busy, Ordering::SeqCst);
    }

    /// Cancels the running sync, if any, and returns what waits for it to
    /// end; the loop runs on. A sync waiting to start is dropped.
    pub fn stop_sync(&self) -> Option<SyncStop> {
        let (answer, answered) = mpsc::channel();
        self.wake.send(Msg::StopSync(answer)).ok()?;
        answered.recv().ok().flatten()
    }

    /// What keeps automatic syncs from running now, if anything.
    pub fn hold(&self) -> Option<Hold> {
        self.scheduler().hold()
    }

    /// Sets the hold, as a sync's problem would.
    pub fn set_hold(&self, hold: Option<Hold>) {
        self.scheduler().set_hold(hold);
    }

    fn scheduler(&self) -> std::sync::MutexGuard<'_, Scheduler> {
        self.scheduler
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    /// Sets the stop flag, cancels a running sync between pages, batches
    /// and files, waits for the loop to end and emits
    /// [`DriverEvent::Stopped`].
    pub fn stop(mut self) {
        self.shutdown();
    }

    /// Whether the loop ended: the session ended, or it was stopped.
    pub fn finished(&self) -> bool {
        self.finished.load(Ordering::SeqCst)
    }

    fn shutdown(&mut self) {
        let Some(thread) = self.thread.take() else {
            return;
        };
        self.stop.store(true, Ordering::SeqCst);
        self.wake.send(Msg::Stop).ok();
        thread.join().ok();
    }
}

impl Drop for Driver {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Turns local edits into wakes until the store's channel closes or the
/// control thread is gone.
fn feed_loop(changes: async_channel::Receiver<Change>, wake: mpsc::Sender<Msg>) {
    while let Ok(change) = changes.recv_blocking() {
        if is_local_edit(&change) && wake.send(Msg::Edit).is_err() {
            return;
        }
    }
}

/// The sync running on its own thread.
struct Worker {
    cancel: Arc<AtomicBool>,
    ended: Arc<Ended>,
    thread: JoinHandle<()>,
}

/// The control thread's state.
struct Control {
    seams: DriverSeams,
    runner: Runner,
    hook: Option<Hook>,
    on: Box<dyn FnMut(DriverEvent) + Send>,
    rx: mpsc::Receiver<Msg>,
    tx: mpsc::Sender<Msg>,
    stop: Arc<AtomicBool>,
    finished: Arc<AtomicBool>,
    busy: Arc<AtomicBool>,
    /// [`DriverStart::automatic`].
    automatic: bool,
    scheduler: Arc<Mutex<Scheduler>>,
    worker: Option<Worker>,
    /// A library sync the scheduler started while a hook-only worker ran:
    /// it starts when that one ends, running the hook if this is `true`.
    start_pending: Option<bool>,
    /// A timer tick queued behind a running sync: the next sync runs the
    /// hook whatever its result.
    hook_next: bool,
    /// The session ended: nothing runs any more.
    ended: bool,
    next_tick: Instant,
    edit_due: Option<Instant>,
    feed: Option<(async_channel::Receiver<Change>, JoinHandle<()>)>,
}

impl Control {
    fn run(&mut self) {
        (self.on)(DriverEvent::Started);
        self.next_tick = Instant::now() + self.seams.interval;
        if self.automatic {
            self.request(false, false);
        }
        while !self.stop.load(Ordering::SeqCst) {
            let msg = match self.deadline() {
                Some(at) => self
                    .rx
                    .recv_timeout(at.saturating_duration_since(Instant::now())),
                None => self
                    .rx
                    .recv()
                    .map_err(|_| mpsc::RecvTimeoutError::Disconnected),
            };
            match msg {
                Ok(Msg::Stop) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
                Ok(Msg::Edit) => {
                    if !self.ended && self.automatic {
                        self.edit_due = Some(Instant::now() + self.seams.after_edit);
                    }
                }
                Ok(Msg::Request { manual, answer }) => {
                    let request = self.request(manual, false);
                    answer.send(request).ok();
                }
                Ok(Msg::StopSync(answer)) => {
                    let stop = self.stop_sync();
                    answer.send(stop).ok();
                }
                Ok(Msg::Done(outcome)) => self.finish(*outcome),
                Err(mpsc::RecvTimeoutError::Timeout) => self.timers(),
            }
        }
        self.stop.store(true, Ordering::SeqCst);
        if let Some(worker) = self.worker.take() {
            worker.cancel.store(true, Ordering::SeqCst);
            worker.thread.join().ok();
        }
        self.close_feed();
        self.finished.store(true, Ordering::SeqCst);
        (self.on)(DriverEvent::Stopped);
    }

    fn scheduler(&self) -> std::sync::MutexGuard<'_, Scheduler> {
        self.scheduler
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    /// The next time a timer is due; `None` once the session ended, or
    /// when nothing syncs on its own.
    fn deadline(&self) -> Option<Instant> {
        if self.ended || !self.automatic {
            return None;
        }
        Some(match self.edit_due {
            Some(edit) => edit.min(self.next_tick),
            None => self.next_tick,
        })
    }

    fn timers(&mut self) {
        let now = Instant::now();
        if self.edit_due.is_some_and(|due| due <= now) {
            self.edit_due = None;
            self.request(false, false);
        }
        if self.next_tick <= now {
            self.next_tick = now + self.seams.interval;
            self.request(false, true);
        }
    }

    /// A sync is asked for: `manual` for Sync now, `tick` for the timer.
    fn request(&mut self, manual: bool, tick: bool) -> Request {
        if self.ended {
            return Request::Refused;
        }
        let now = (self.seams.now_ms)();
        let busy = self.busy.load(Ordering::SeqCst);
        let request = self.scheduler().request(manual, busy, now);
        match request {
            Request::Start => {
                let hook = tick || std::mem::take(&mut self.hook_next);
                if self.worker.is_some() {
                    // Only a hook-only worker runs while the scheduler has
                    // no sync running.
                    self.start_pending = Some(hook);
                } else {
                    self.spawn(true, hook);
                }
            }
            Request::Queued => self.hook_next |= tick,
            Request::Refused => {
                if tick && !busy && self.worker.is_none() && self.hook.is_some() {
                    self.spawn(false, true);
                }
            }
        }
        request
    }

    /// Cancels the running worker and drops a library sync waiting for
    /// it; returns what waits for the worker.
    fn stop_sync(&mut self) -> Option<SyncStop> {
        if self.start_pending.take().is_some() {
            self.scheduler().start_failed();
            (self.on)(DriverEvent::Cancelled);
        }
        let worker = self.worker.as_ref()?;
        worker.cancel.store(true, Ordering::SeqCst);
        Some(SyncStop(worker.ended.clone()))
    }

    /// Starts a worker: a library sync if `library`, then the hook after a
    /// successful one, or after any if `hook` (a timer tick).
    fn spawn(&mut self, library: bool, hook: bool) {
        let cancel = Arc::new(AtomicBool::new(false));
        let ended = Arc::new(Ended::default());
        let worker_cancel = cancel.clone();
        let guard = EndGuard(ended.clone());
        let runner = self.runner.clone();
        let after_sync = self.hook.clone();
        let tx = self.tx.clone();
        let thread = std::thread::Builder::new()
            .name("kanso-sync".into())
            .spawn(move || {
                let _guard = guard;
                let outcome = work(library, hook, &runner, after_sync.as_ref(), &worker_cancel);
                tx.send(Msg::Done(Box::new(outcome))).ok();
            });
        match thread {
            Ok(thread) => {
                self.worker = Some(Worker {
                    cancel,
                    ended,
                    thread,
                });
                if library {
                    (self.on)(DriverEvent::SyncStarted);
                }
            }
            Err(err) => {
                eprintln!("kanso: couldn't start the sync thread: {err}");
                if library {
                    self.scheduler().start_failed();
                    (self.on)(DriverEvent::Problem(SyncProblem::Other));
                }
            }
        }
    }

    fn finish(&mut self, outcome: Outcome) {
        if let Some(worker) = self.worker.take() {
            worker.thread.join().ok();
        }
        let Some(result) = outcome.library else {
            self.hook_events(outcome.hook);
            if let Some(hook) = self.start_pending.take() {
                self.spawn(true, hook);
            }
            return;
        };
        let report = match &result {
            Some(Ok(report)) => Some(report.clone()),
            _ => None,
        };
        let now = (self.seams.now_ms)();
        let finished = self.scheduler().finished(true, result, now);
        if let Some(reason) = finished.signed_out {
            self.end(reason);
            return;
        }
        if finished.cancelled {
            (self.on)(DriverEvent::Cancelled);
        } else {
            let problem = finished.state.as_ref().and_then(state_problem);
            self.write_last(now, &LastResult::new(report.as_ref(), problem));
            if let Some(report) = report {
                (self.on)(DriverEvent::Synced(report));
            }
            if let Some(problem) = problem.cloned() {
                (self.on)(DriverEvent::Problem(problem));
            }
        }
        if finished.remerged {
            (self.on)(DriverEvent::Remerged);
        }
        self.hook_events(outcome.hook);
        if let Some(manual) = finished.again {
            self.request(manual, false);
        }
    }

    fn hook_events(&mut self, hook: Option<Result<(), String>>) {
        if let Some(Err(text)) = hook {
            (self.on)(DriverEvent::Problem(SyncProblem::Other));
            (self.on)(DriverEvent::HookFailed(text));
        }
    }

    /// The server ended the session: forget it, say so once, and sync no
    /// more. While an account task runs, the recovery key is the owner's
    /// to judge ([`Driver::set_busy`]).
    fn end(&mut self, reason: SignOutReason) {
        self.ended = true;
        self.edit_due = None;
        self.start_pending = None;
        self.close_feed();
        forget_ended_session(
            self.seams.secrets.as_ref(),
            &self.seams.library_id,
            reason,
            Some(&self.seams.session),
            self.busy.load(Ordering::SeqCst),
        );
        self.finished.store(true, Ordering::SeqCst);
        (self.on)(DriverEvent::SignedOut(reason));
    }
    /// Writes the last-sync keys under the session's guard: a library bound
    /// elsewhere meanwhile keeps its own.
    fn write_last(&self, now: i64, last: &LastResult) {
        let pairs = kanso_sync::binding::guard_pairs(&self.seams.session);
        let guard = kanso_sync::binding::guard(&pairs);
        let json = serde_json::to_string(last).expect("the result serializes");
        for (key, value) in [(LAST_SYNC_MS, now.to_string()), (LAST_RESULT, json)] {
            match self.seams.store.set_meta_if(key, &value, &guard) {
                Ok(()) | Err(kanso_store::StoreError::GuardFailed) => {}
                Err(err) => eprintln!("kanso: saving the last sync: {err}"),
            }
        }
    }

    fn close_feed(&mut self) {
        if let Some((changes, thread)) = self.feed.take() {
            changes.close();
            thread.join().ok();
        }
    }
}

/// A worker's job, on its thread.
fn work(
    library: bool,
    hook: bool,
    runner: &Runner,
    after_sync: Option<&Hook>,
    cancel: &Arc<AtomicBool>,
) -> Outcome {
    let result = library.then(|| catch_unwind(AssertUnwindSafe(|| runner(cancel.clone()))).ok());
    let run_hook = match &result {
        None => true,
        Some(Some(Ok(_))) => true,
        Some(Some(Err(err))) => {
            hook && !matches!(
                kanso_sync::problem(err),
                None | Some(SyncProblem::SignedOut(_))
            )
        }
        Some(None) => hook,
    };
    let hook = match after_sync {
        Some(after_sync) if run_hook && !cancel.load(Ordering::SeqCst) => Some(
            catch_unwind(AssertUnwindSafe(|| after_sync()))
                .unwrap_or_else(|_| Err("the shared-notes sync panicked".into())),
        ),
        _ => None,
    };
    Outcome {
        library: result,
        hook,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;
    use std::sync::atomic::AtomicUsize;

    use kanso_core::DocType;
    use kanso_store::{Origin, Put};
    use kanso_sync::{ApiError, MemorySecrets, MemoryServer};
    use proptest::prelude::*;

    use super::*;
    use crate::sync::tests::library;

    const SLACK: Duration = Duration::from_secs(5);

    struct Lib {
        server: Arc<MemoryServer>,
        store: Arc<Store>,
        session: Arc<Session>,
        secrets: Arc<MemorySecrets>,
        _dir: tempfile::TempDir,
    }

    fn lib() -> Lib {
        let server = Arc::new(MemoryServer::new());
        let (store, session, dir) = library(&server, "alice", 1, true);
        let secrets = Arc::new(MemorySecrets::new());
        kanso_sync::keychain::save_session(secrets.as_ref(), "l_test", &session).unwrap();
        Lib {
            server,
            store: Arc::new(store),
            session: Arc::new(session),
            secrets,
            _dir: dir,
        }
    }

    fn seams(lib: &Lib, interval: Duration, after_edit: Duration) -> DriverSeams {
        DriverSeams {
            store: lib.store.clone(),
            backend: lib.server.clone(),
            secrets: lib.secrets.clone(),
            library_id: "l_test".into(),
            session: lib.session.clone(),
            interval,
            after_edit,
            now_ms: Arc::new(|| 1_234),
            after_sync: None,
        }
    }

    type Events = mpsc::Receiver<DriverEvent>;

    fn listen() -> (Box<dyn FnMut(DriverEvent) + Send>, Events) {
        let (tx, rx) = mpsc::channel();
        (
            Box::new(move |event| {
                tx.send(event).ok();
            }),
            rx,
        )
    }

    /// Waits for the first event `want` accepts, returning what came before.
    fn wait_for(events: &Events, want: impl Fn(&DriverEvent) -> bool) -> Vec<DriverEvent> {
        let mut seen = Vec::new();
        loop {
            let event = events.recv_timeout(SLACK).expect("the event came");
            let done = want(&event);
            seen.push(event);
            if done {
                return seen;
            }
        }
    }

    fn synced(event: &DriverEvent) -> bool {
        matches!(event, DriverEvent::Synced(_))
    }

    fn put(store: &Store, origin: Origin) {
        store
            .put(
                DocType::Book,
                serde_json::json!({"name": "B"}),
                Put::Upsert,
                origin,
            )
            .unwrap();
    }

    #[test]
    fn an_api_write_reaches_the_server_after_the_wait() {
        let lib = lib();
        let after_edit = Duration::from_millis(300);
        let (on, events) = listen();
        let driver = Driver::start(seams(&lib, Duration::from_secs(600), after_edit), on).unwrap();
        wait_for(&events, synced);
        let docs = || lib.server.plain_view(&lib.session).docs().len();
        let before = docs();
        let at = Instant::now();
        put(&lib.store, Origin::Api);
        while docs() == before {
            assert!(at.elapsed() < after_edit + SLACK, "the edit never synced");
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(at.elapsed() >= after_edit, "synced before the wait");
        driver.stop();
        assert_eq!(events.try_iter().last(), Some(DriverEvent::Stopped));
    }

    #[test]
    fn sync_writes_dont_wake_the_driver() {
        let lib = lib();
        let after_edit = Duration::from_millis(50);
        let (on, events) = listen();
        let driver = Driver::start(seams(&lib, Duration::from_secs(600), after_edit), on).unwrap();
        wait_for(&events, synced);
        put(&lib.store, Origin::Sync);
        std::thread::sleep(after_edit * 6);
        driver.stop();
        let rest: Vec<_> = events.try_iter().collect();
        assert_eq!(rest, vec![DriverEvent::Stopped]);
    }

    #[test]
    fn the_last_sync_is_written_to_meta() {
        let lib = lib();
        let (on, events) = listen();
        let driver = Driver::start(seams(&lib, Duration::from_secs(600), AFTER), on).unwrap();
        let seen = wait_for(&events, synced);
        assert_eq!(seen.first(), Some(&DriverEvent::Started));
        assert_eq!(
            lib.store.meta(LAST_SYNC_MS).unwrap().as_deref(),
            Some("1234")
        );
        let last: LastResult =
            serde_json::from_str(&lib.store.meta(LAST_RESULT).unwrap().unwrap()).unwrap();
        assert_eq!(last.problem, None);
        assert!(
            !lib.store
                .meta(LAST_RESULT)
                .unwrap()
                .unwrap()
                .contains("problem")
        );
        driver.stop();
    }

    const AFTER: Duration = Duration::from_secs(20);

    #[test]
    fn an_ended_session_signs_out_once() {
        let lib = lib();
        lib.server.revoke_all("alice", SignOutReason::Revoked);
        let (on, events) = listen();
        let driver = Driver::start(seams(&lib, Duration::from_millis(20), AFTER), on).unwrap();
        let seen = wait_for(&events, |e| matches!(e, DriverEvent::SignedOut(_)));
        assert_eq!(
            seen.last(),
            Some(&DriverEvent::SignedOut(SignOutReason::Revoked))
        );
        assert!(driver.finished());
        assert!(
            lib.secrets
                .items()
                .iter()
                .all(|item| !item.starts_with("sync-account"))
        );
        // Nothing runs any more: not the timer, not an edit, not Sync now.
        assert_eq!(driver.sync_now(), Request::Refused);
        put(&lib.store, Origin::Api);
        std::thread::sleep(Duration::from_millis(100));
        driver.stop();
        let rest: Vec<_> = events.try_iter().collect();
        assert_eq!(rest, vec![DriverEvent::Stopped]);
        assert_eq!(lib.store.meta(LAST_SYNC_MS).unwrap(), None);
    }

    /// A sync that runs until it is cancelled.
    fn until_cancelled(started: mpsc::Sender<()>) -> Runner {
        Arc::new(move |cancel: Arc<AtomicBool>| {
            started.send(()).ok();
            while !cancel.load(Ordering::SeqCst) {
                std::thread::sleep(Duration::from_millis(2));
            }
            Err(SyncError::Cancelled)
        })
    }

    #[test]
    fn stop_cancels_a_running_sync_and_returns_quickly() {
        let lib = lib();
        let (on, events) = listen();
        let (started, running) = mpsc::channel();
        let driver = Driver::start_with(
            seams(&lib, Duration::from_secs(600), AFTER),
            DriverStart::default(),
            until_cancelled(started),
            on,
        )
        .unwrap();
        running.recv_timeout(SLACK).unwrap();
        let at = Instant::now();
        driver.stop();
        assert!(
            at.elapsed() < Duration::from_millis(500),
            "{:?}",
            at.elapsed()
        );
        let all: Vec<_> = events.try_iter().collect();
        assert_eq!(
            all,
            vec![
                DriverEvent::Started,
                DriverEvent::SyncStarted,
                DriverEvent::Stopped
            ]
        );
    }

    #[test]
    fn a_failing_hook_is_a_problem_with_its_text() {
        let lib = lib();
        let (on, events) = listen();
        let mut seams = seams(&lib, Duration::from_secs(600), AFTER);
        seams.after_sync = Some(Arc::new(|| Err("shared notes: offline".into())));
        let driver = Driver::start(seams, on).unwrap();
        let seen = wait_for(&events, |e| matches!(e, DriverEvent::HookFailed(_)));
        assert_eq!(seen[1], DriverEvent::SyncStarted);
        assert!(matches!(seen[2], DriverEvent::Synced(_)), "{seen:?}");
        assert_eq!(
            seen[3..],
            [
                DriverEvent::Problem(SyncProblem::Other),
                DriverEvent::HookFailed("shared notes: offline".into())
            ]
        );
        driver.stop();
    }

    #[test]
    fn last_results_name_the_problem() {
        let report = SyncReport {
            pulled: 3,
            pushed: 2,
            files_up: 1,
            conflicts_local: 1,
            conflicts_remote: 1,
            pending: 4,
            ..SyncReport::default()
        };
        let last = LastResult::new(Some(&report), None);
        let json = serde_json::to_value(&last).unwrap();
        assert_eq!(
            json,
            serde_json::json!({"pulled":3,"pushed":2,"files_up":1,"conflicts":2,"pending":4})
        );
        let failed = LastResult::new(None, Some(&SyncProblem::Offline));
        assert_eq!(failed.problem.as_deref(), Some("Offline"));
        assert_eq!(failed.pulled, 0);
    }

    /// How a scripted sync ends.
    #[derive(Clone, Copy, Debug)]
    enum Ending {
        Clean,
        Pushed,
        Offline,
        BackOff,
        Outdated,
        SignedOut,
    }

    impl Ending {
        fn result(self) -> Result<SyncReport, SyncError> {
            match self {
                Ending::Clean => Ok(SyncReport::default()),
                Ending::Pushed => Ok(SyncReport {
                    pushed: 1,
                    ..SyncReport::default()
                }),
                Ending::Offline => Err(SyncError::Api(ApiError::Transport("reset".into()))),
                Ending::BackOff => Err(SyncError::BackOff { until: i64::MAX }),
                Ending::Outdated => Err(SyncError::Api(ApiError::NotFound)),
                Ending::SignedOut => Err(SyncError::Api(ApiError::Unauthorized {
                    reason: SignOutReason::Expired,
                })),
            }
        }
    }

    #[derive(Clone, Copy, Debug)]
    enum Wake {
        Edit,
        SyncWrite,
        Manual,
        Pause(u64),
    }

    fn ending() -> impl Strategy<Value = Ending> {
        prop_oneof![
            4 => Just(Ending::Clean),
            2 => Just(Ending::Pushed),
            2 => Just(Ending::Offline),
            1 => Just(Ending::BackOff),
            1 => Just(Ending::Outdated),
            1 => Just(Ending::SignedOut),
        ]
    }

    fn wake() -> impl Strategy<Value = Wake> {
        prop_oneof![
            Just(Wake::Edit),
            Just(Wake::SyncWrite),
            Just(Wake::Manual),
            (0u64..8).prop_map(Wake::Pause),
        ]
    }

    /// Counts the syncs and hooks running at once.
    #[derive(Default)]
    struct Gauge {
        now: AtomicUsize,
        most: AtomicUsize,
        runs: AtomicUsize,
    }

    impl Gauge {
        fn during<T>(&self, f: impl FnOnce() -> T) -> T {
            let now = self.now.fetch_add(1, Ordering::SeqCst) + 1;
            self.most.fetch_max(now, Ordering::SeqCst);
            let value = f();
            self.now.fetch_sub(1, Ordering::SeqCst);
            value
        }
    }

    /// A runner that counts its syncs, each taking `ms`, and stops early
    /// when cancelled.
    fn counting(runs: Arc<AtomicUsize>, ms: u64) -> Runner {
        Arc::new(move |cancel: Arc<AtomicBool>| {
            runs.fetch_add(1, Ordering::SeqCst);
            let until = Instant::now() + Duration::from_millis(ms);
            while Instant::now() < until {
                if cancel.load(Ordering::SeqCst) {
                    return Err(SyncError::Cancelled);
                }
                std::thread::sleep(Duration::from_millis(1));
            }
            Ok(SyncReport::default())
        })
    }

    fn manual_only() -> DriverStart {
        DriverStart {
            automatic: false,
            ..DriverStart::default()
        }
    }

    #[test]
    fn a_started_sync_has_started_when_the_request_returns() {
        let lib = lib();
        let (on, events) = listen();
        let runs = Arc::new(AtomicUsize::new(0));
        let driver = Driver::start_with(
            seams(&lib, Duration::from_secs(600), AFTER),
            manual_only(),
            counting(runs.clone(), 200),
            on,
        )
        .unwrap();
        assert_eq!(driver.request(false), Request::Start);
        assert_eq!(
            events.try_iter().collect::<Vec<_>>(),
            vec![DriverEvent::Started, DriverEvent::SyncStarted]
        );
        assert_eq!(driver.request(false), Request::Queued);
        assert_eq!(driver.sync_now(), Request::Queued);
        let seen = wait_for(&events, synced);
        assert_eq!(seen.len(), 1, "{seen:?}");
        // The queued requests run once, after it.
        assert_eq!(wait_for(&events, synced)[0], DriverEvent::SyncStarted);
        std::thread::sleep(Duration::from_millis(50));
        assert_eq!(runs.load(Ordering::SeqCst), 2);
        driver.stop();
    }

    #[test]
    fn without_automatic_syncs_only_requests_run() {
        let lib = lib();
        let (on, events) = listen();
        let runs = Arc::new(AtomicUsize::new(0));
        let seams = seams(&lib, Duration::from_millis(5), Duration::from_millis(2));
        let driver =
            Driver::start_with(seams, manual_only(), counting(runs.clone(), 0), on).unwrap();
        put(&lib.store, Origin::Api);
        std::thread::sleep(Duration::from_millis(60));
        assert_eq!(
            runs.load(Ordering::SeqCst),
            0,
            "no timer, no edit, no start"
        );
        assert_eq!(driver.sync_now(), Request::Start);
        wait_for(&events, synced);
        assert_eq!(runs.load(Ordering::SeqCst), 1);
        driver.stop();
    }

    #[test]
    fn a_driver_keeps_the_hold_it_starts_with() {
        let lib = lib();
        let (on, events) = listen();
        let runs = Arc::new(AtomicUsize::new(0));
        let start = DriverStart {
            hold: Some(Hold::UntilSyncNow),
            ..DriverStart::default()
        };
        let driver = Driver::start_with(
            seams(&lib, Duration::from_secs(600), AFTER),
            start,
            counting(runs.clone(), 0),
            on,
        )
        .unwrap();
        assert_eq!(driver.request(false), Request::Refused);
        assert_eq!(driver.hold(), Some(Hold::UntilSyncNow));
        assert_eq!(runs.load(Ordering::SeqCst), 0, "the start sync waits too");
        assert_eq!(driver.sync_now(), Request::Start);
        assert_eq!(driver.hold(), None, "Sync now lifts it");
        wait_for(&events, synced);
        driver.set_hold(Some(Hold::Stopped));
        assert_eq!(driver.request(false), Request::Refused);
        driver.stop();
    }

    #[test]
    fn busy_refuses_even_sync_now_until_it_ends() {
        let lib = lib();
        let (on, events) = listen();
        let runs = Arc::new(AtomicUsize::new(0));
        let start = DriverStart {
            busy: true,
            ..DriverStart::default()
        };
        let driver = Driver::start_with(
            seams(&lib, Duration::from_secs(600), AFTER),
            start,
            counting(runs.clone(), 0),
            on,
        )
        .unwrap();
        assert_eq!(driver.sync_now(), Request::Refused);
        assert_eq!(driver.request(false), Request::Refused);
        assert_eq!(runs.load(Ordering::SeqCst), 0);
        driver.set_busy(false);
        assert_eq!(driver.request(false), Request::Start);
        wait_for(&events, synced);
        driver.stop();
    }

    #[test]
    fn stop_sync_cancels_the_running_sync_and_the_loop_runs_on() {
        let lib = lib();
        let (on, events) = listen();
        let (started, running) = mpsc::channel();
        let driver = Driver::start_with(
            seams(&lib, Duration::from_secs(600), AFTER),
            DriverStart::default(),
            until_cancelled(started),
            on,
        )
        .unwrap();
        running.recv_timeout(SLACK).unwrap();
        driver.set_busy(true);
        let stop = driver.stop_sync().expect("a sync runs");
        let at = Instant::now();
        stop.join();
        assert!(at.elapsed() < Duration::from_millis(500));
        let seen = wait_for(&events, |e| *e == DriverEvent::Cancelled);
        assert_eq!(
            seen,
            vec![
                DriverEvent::Started,
                DriverEvent::SyncStarted,
                DriverEvent::Cancelled
            ]
        );
        assert!(driver.stop_sync().is_none(), "nothing runs now");
        assert!(!driver.finished());
        driver.set_busy(false);
        assert_eq!(driver.request(false), Request::Start);
        running.recv_timeout(SLACK).unwrap();
        driver.stop();
    }

    /// What an owner does to a running driver.
    #[derive(Clone, Copy, Debug)]
    enum Op {
        Request(bool),
        Busy(bool),
        StopSync,
        Pause(u64),
    }

    fn op() -> impl Strategy<Value = Op> {
        prop_oneof![
            any::<bool>().prop_map(Op::Request),
            any::<bool>().prop_map(Op::Busy),
            Just(Op::StopSync),
            (0u64..4).prop_map(Op::Pause),
        ]
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(12))]

        /// For any wakes (timer, edits, Sync now, sync writes) and endings:
        /// at most one sync or hook runs at a time; the first SignedOut is
        /// the only one, ends the loop and leaves no session item; a hold
        /// lets Sync now through; and stop ends with one Stopped.
        #[test]
        fn prop_one_sync_at_a_time(
            endings in prop::collection::vec(ending(), 1..8),
            wakes in prop::collection::vec(wake(), 0..16),
            interval in 3u64..15,
            after_edit in 1u64..10,
        ) {
            let lib = lib();
            let gauge = Arc::new(Gauge::default());
            let script = Arc::new(Mutex::new(endings.clone().into_iter()));
            let runner: Runner = {
                let gauge = gauge.clone();
                Arc::new(move |_cancel| {
                    gauge.during(|| {
                        gauge.runs.fetch_add(1, Ordering::SeqCst);
                        std::thread::sleep(Duration::from_millis(1));
                        script.lock().unwrap().next().unwrap_or(Ending::Clean).result()
                    })
                })
            };
            let mut seams = seams(
                &lib,
                Duration::from_millis(interval),
                Duration::from_millis(after_edit),
            );
            let hook_gauge = gauge.clone();
            seams.after_sync = Some(Arc::new(move || hook_gauge.during(|| Ok(()))));
            // The syncs that had started when the session ended.
            let runs_at_sign_out = Arc::new(AtomicUsize::new(usize::MAX));
            let (tx, events) = mpsc::channel();
            let on: Box<dyn FnMut(DriverEvent) + Send> = {
                let gauge = gauge.clone();
                let runs_at_sign_out = runs_at_sign_out.clone();
                Box::new(move |event| {
                    if matches!(event, DriverEvent::SignedOut(_)) {
                        runs_at_sign_out.store(gauge.runs.load(Ordering::SeqCst), Ordering::SeqCst);
                    }
                    tx.send(event).ok();
                })
            };
            let driver = Driver::start_with(seams, DriverStart::default(), runner, on).unwrap();
            for wake in wakes {
                match wake {
                    Wake::Edit => put(&lib.store, Origin::Api),
                    Wake::SyncWrite => put(&lib.store, Origin::Sync),
                    Wake::Manual => {
                        driver.sync_now();
                    }
                    Wake::Pause(ms) => std::thread::sleep(Duration::from_millis(ms)),
                }
            }
            std::thread::sleep(Duration::from_millis(20));
            let at = Instant::now();
            driver.stop();
            prop_assert!(at.elapsed() < Duration::from_millis(500));
            let all: Vec<_> = events.try_iter().collect();
            prop_assert_eq!(gauge.most.load(Ordering::SeqCst), 1);
            prop_assert_eq!(all.first(), Some(&DriverEvent::Started));
            prop_assert_eq!(all.last(), Some(&DriverEvent::Stopped));
            prop_assert_eq!(all.iter().filter(|e| **e == DriverEvent::Stopped).count(), 1);
            let signed_out = all.iter().filter(|e| matches!(e, DriverEvent::SignedOut(_))).count();
            prop_assert!(signed_out <= 1);
            let has_item = lib.secrets.items().iter().any(|i| i.starts_with("sync-account"));
            if signed_out == 1 {
                // Nothing ran after it: SignedOut, then Stopped.
                prop_assert!(matches!(all[all.len() - 2], DriverEvent::SignedOut(_)));
                prop_assert_eq!(
                    runs_at_sign_out.load(Ordering::SeqCst),
                    gauge.runs.load(Ordering::SeqCst)
                );
                prop_assert!(!has_item);
            } else {
                prop_assert!(has_item);
            }
        }

        /// A hold refuses the timer and edits, and lets Sync now through.
        #[test]
        fn prop_a_hold_lets_only_sync_now_through(edits in 0usize..4, manual: bool) {
            let lib = lib();
            let gauge = Arc::new(Gauge::default());
            let runner: Runner = {
                let gauge = gauge.clone();
                Arc::new(move |_cancel| {
                    gauge.runs.fetch_add(1, Ordering::SeqCst);
                    Ending::Outdated.result()
                })
            };
            let seams = seams(&lib, Duration::from_millis(5), Duration::from_millis(2));
            let (on, events) = listen();
            let driver = Driver::start_with(seams, DriverStart::default(), runner, on).unwrap();
            wait_for(&events, |e| matches!(e, DriverEvent::Problem(_)));
            for _ in 0..edits {
                put(&lib.store, Origin::Api);
            }
            std::thread::sleep(Duration::from_millis(30));
            prop_assert_eq!(gauge.runs.load(Ordering::SeqCst), 1);
            if manual {
                driver.sync_now();
                wait_for(&events, |e| matches!(e, DriverEvent::Problem(_)));
                prop_assert_eq!(gauge.runs.load(Ordering::SeqCst), 2);
            }
            driver.stop();
        }
        /// For any requests, busy changes and stops: busy refuses every
        /// request and an idle driver never does; each sync that ran was
        /// announced by one SyncStarted; once a sync stopped while busy is
        /// joined, nothing runs; and at most one runs at once.
        #[test]
        fn prop_requests_busy_and_stops(ops in prop::collection::vec(op(), 0..24)) {
            let lib = lib();
            let gauge = Arc::new(Gauge::default());
            let runner: Runner = {
                let gauge = gauge.clone();
                Arc::new(move |cancel: Arc<AtomicBool>| {
                    gauge.during(|| {
                        gauge.runs.fetch_add(1, Ordering::SeqCst);
                        for _ in 0..3 {
                            if cancel.load(Ordering::SeqCst) {
                                return Err(SyncError::Cancelled);
                            }
                            std::thread::sleep(Duration::from_millis(1));
                        }
                        Ok(SyncReport::default())
                    })
                })
            };
            let (on, events) = listen();
            let seams = seams(&lib, Duration::from_secs(600), AFTER);
            let driver = Driver::start_with(seams, manual_only(), runner, on).unwrap();
            let mut busy = false;
            for op in ops {
                match op {
                    Op::Request(manual) => {
                        let answer = driver.request(manual);
                        prop_assert_eq!(answer == Request::Refused, busy, "{:?}", answer);
                    }
                    Op::Busy(now) => {
                        busy = now;
                        driver.set_busy(now);
                    }
                    Op::StopSync => {
                        // As an account task does: busy first, so nothing
                        // queued starts once the stopped sync ends.
                        driver.set_busy(true);
                        if let Some(stop) = driver.stop_sync() {
                            stop.join();
                            prop_assert_eq!(gauge.now.load(Ordering::SeqCst), 0);
                        }
                        driver.set_busy(busy);
                    }
                    Op::Pause(ms) => std::thread::sleep(Duration::from_millis(ms)),
                }
            }
            driver.stop();
            let all: Vec<_> = events.try_iter().collect();
            let started = all.iter().filter(|e| **e == DriverEvent::SyncStarted).count();
            prop_assert_eq!(started, gauge.runs.load(Ordering::SeqCst));
            prop_assert!(gauge.most.load(Ordering::SeqCst) <= 1);
            prop_assert_eq!(all.last(), Some(&DriverEvent::Stopped));
        }
    }
}
