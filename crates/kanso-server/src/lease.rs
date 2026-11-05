//! A request's hold on the API key that signed it in (issue #243).
//!
//! Revoking a key must be final for work already under way: no request the
//! key signed in may finish after the revoke returns. Every request holds a
//! [`Lease`] on its key's [`KeyGate`] from sign-in until its response is
//! made, and until any store call it started has returned. Revoking closes
//! the gate, wakes every request to drop it, and waits for the leases to be
//! let go.

use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::Duration;

use tokio::sync::watch;

/// How long a revoke waits for requests already inside the store to return.
/// A store call is short; this only bounds a stuck disk.
pub(crate) const DRAIN: Duration = Duration::from_secs(10);

#[derive(Default)]
struct State {
    revoked: bool,
    /// Leases alive: requests, and store calls they started.
    held: usize,
}

/// One key's gate: open until the key is revoked or leaves the library.
pub(crate) struct KeyGate {
    state: Mutex<State>,
    idle: Condvar,
    closed: watch::Sender<bool>,
}

impl KeyGate {
    pub(crate) fn new() -> Arc<KeyGate> {
        Arc::new(KeyGate {
            state: Mutex::new(State::default()),
            idle: Condvar::new(),
            closed: watch::channel(false).0,
        })
    }

    // The counters stay consistent through a panic: every update is one step.
    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|p| p.into_inner())
    }

    /// A lease, or `None` once the gate is closed.
    pub(crate) fn lease(self: &Arc<Self>) -> Option<Lease> {
        let mut state = self.lock();
        if state.revoked {
            return None;
        }
        state.held += 1;
        Some(Lease(self.clone()))
    }

    /// Closes the gate: no new lease, and every holder is told to stop.
    pub(crate) fn close(&self) {
        self.lock().revoked = true;
        self.closed.send_replace(true);
    }

    pub(crate) fn is_closed(&self) -> bool {
        self.lock().revoked
    }

    /// Waits until no lease is held, at most `limit`. `true` when none is.
    pub(crate) fn drain(&self, limit: Duration) -> bool {
        let state = self.lock();
        let (state, _) = self
            .idle
            .wait_timeout_while(state, limit, |state| state.held > 0)
            .unwrap_or_else(|p| p.into_inner());
        state.held == 0
    }

    #[cfg(test)]
    pub(crate) fn held(&self) -> usize {
        self.lock().held
    }
}

/// Held while a request signed in by a key is being answered.
pub(crate) struct Lease(Arc<KeyGate>);

impl Lease {
    /// Claims a store call before the gate closes, or refuses it after.
    ///
    /// The state lock makes this the ordering point with `close`: once
    /// `close` wins, no newly claimed store call can begin. A call that wins
    /// first is counted by this lease and must drain before revoke succeeds.
    pub(crate) fn begin_store_call(&self) -> bool {
        !self.0.is_closed()
    }

    /// Whether the key was revoked, or left the library, since sign-in.
    pub(crate) fn is_revoked(&self) -> bool {
        self.0.is_closed()
    }

    /// Resolves when the key is revoked.
    pub(crate) async fn revoked(&self) {
        let mut closed = self.0.closed.subscribe();
        let _ = closed.wait_for(|closed| *closed).await;
    }

    /// Another lease on the same key, for work that outlives the request's
    /// future: a store call on the blocking pool. Always granted, since the
    /// caller already holds one.
    pub(crate) fn also(&self) -> Lease {
        self.0.lock().held += 1;
        Lease(self.0.clone())
    }

    /// The lease of the request this task is answering, if a key signed it in.
    pub(crate) fn current() -> Option<Lease> {
        CURRENT.try_with(Lease::also).ok()
    }
}

impl Drop for Lease {
    fn drop(&mut self) {
        let mut state = self.0.lock();
        state.held -= 1;
        if state.held == 0 {
            self.0.idle.notify_all();
        }
    }
}

tokio::task_local! {
    /// The lease of the request being answered on this task.
    pub(crate) static CURRENT: Lease;
}

/// Runs `work` for the request on a blocking thread, refusing to begin it if
/// the key was revoked in the meantime. `None` means refused.
pub(crate) fn guarded<T>(lease: Option<Lease>, work: impl FnOnce() -> T) -> Option<T> {
    if lease
        .as_ref()
        .is_some_and(|lease| !lease.begin_store_call())
    {
        return None;
    }
    let out = work();
    drop(lease);
    Some(out)
}
