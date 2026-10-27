//! Requests from other devices to join the account, which this phone can approve.
//!
//! The approver's side of "Unlocking a new device" (docs/ACCOUNTS.md), as
//! the desktop's `device_approval` does it. The phone answers a request
//! with a key of its own the moment it first sees it, shows the requesting
//! device's name and the six-digit code once the device has named itself,
//! and sends the account's data key sealed to that device only when the
//! person approves. The revealed key is checked against the commitment the
//! device made before this phone's key existed; a request that fails any
//! check shows as [`PromptState::Changed`] and is never approved.
//!
//! The keys live in memory (`approver_model`) and are zeroized when a
//! request ends. A phone whose app was closed in the middle of a request
//! has lost its key for it, so the request reads as changed until it
//! expires or is declined; the device asking then starts again.
//!
//! [`Approvals::watch`] holds the account event socket open while a sink
//! listens. An event is only a hint: every connect and every event reads
//! the list again, and so does [`Approvals::refresh`].

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Weak};
use std::time::Duration;

use kanso_engine::account::approval::{FrontStep, front_step};
use kanso_engine::account::words::ErrorContext;
use kanso_sync::events::{self, ACCOUNT_EVENTS_PATH};
use kanso_sync::{
    AccountError, ApiError, EventError, EventSource, OpenedRequest, PendingApproval, Session,
    SignOutReason, Signal, account_events_path, challenge_request, decline_request, device_name,
    finish_approval, pending_approvals,
};

use crate::account_model::{ApprovalPrompt, PromptState};
use crate::account_settings::session_ended;
use crate::approver_model::{Counted, Requests, Tracked};
use crate::boundary::{guard, held};
use crate::errors::KansoError;
use crate::library::Inner;
use crate::syncing::{SyncHandle, account_error};

/// Requests from other devices to join the account, which this phone can approve.
#[derive(Clone)]
pub struct Approvals {
    pub(crate) inner: Arc<Inner>,
}

/// How often the socket's thread looks at the account being signed in:
/// every this many of its one-second reads.
const SESSION_CHECK_EVERY: u32 = 10;

impl Approvals {
    /// Calls `sink` with the requests now, and again on every account
    /// event, until it returns `false`. A thread named
    /// `kanso-account-events` holds the event socket open while a sink
    /// listens: it reads the list at the start and on every connect and
    /// event (a sink added while it runs hears the list at once, and so do
    /// the others), reconnects with backoff, and stops once every sink is gone,
    /// the library is closed or the account is signed out. A sink also
    /// hears the list after [`Approvals::refresh`], [`Approvals::approve`]
    /// and [`Approvals::decline`]. Calling `watch` again starts the thread
    /// again if it stopped.
    ///
    /// A sink must not call back into this handle: it runs while the
    /// request list is held.
    pub fn watch(&self, sink: impl FnMut(Vec<ApprovalPrompt>) -> bool + Send + 'static) {
        let approver = &self.inner.approver;
        let mut watching = held(&approver.watching);
        let mut sink = sink;
        let counted = Counted::new(&approver.sinks);
        approver.listeners.add(move |prompts| {
            let _counted = &counted;
            sink(prompts)
        });
        if watching.is_some() {
            // The thread is running: the new sink hears the list now too.
            let owner = Arc::downgrade(&self.inner);
            let spawned = std::thread::Builder::new()
                .name("kanso-account-refresh".into())
                .spawn(move || {
                    refresh_from(&owner);
                });
            if let Err(err) = spawned {
                eprintln!("kanso: couldn't read the device requests: {err}");
            }
            return;
        }
        let stop = Arc::new(AtomicBool::new(false));
        let owner = Arc::downgrade(&self.inner);
        let thread_stop = stop.clone();
        let spawned = std::thread::Builder::new()
            .name("kanso-account-events".into())
            .spawn(move || watch_events(&owner, &thread_stop));
        match spawned {
            Ok(_) => *watching = Some(stop),
            Err(err) => eprintln!("kanso: couldn't start the account event thread: {err}"),
        }
    }

    /// The requests now. Answers a new request with this phone's key
    /// first. The listeners hear the list too.
    pub fn refresh(&self) -> Result<Vec<ApprovalPrompt>, KansoError> {
        guard(|| {
            let mut requests = held(&self.inner.approver.requests);
            let seen = self.look(&mut requests, true);
            self.tell(&seen);
            seen.map(prompts)
        })
    }

    /// Sends the account's data key, sealed to the device that asked, and
    /// gives the requests that remain. Refuses a request that isn't
    /// [`PromptState::Ready`], and one whose checks fail now: the answer
    /// is read from the server again, never taken from an earlier list.
    pub fn approve(&self, id: String) -> Result<Vec<ApprovalPrompt>, KansoError> {
        guard(|| {
            let mut requests = held(&self.inner.approver.requests);
            let mut seen = self.look(&mut requests, true)?;
            let session = self.session()?;
            let at = seen
                .iter()
                .position(|one| one.prompt.id == id)
                .ok_or(KansoError::NotFound)?;
            match (seen[at].prompt.state, seen[at].opened.as_ref()) {
                (PromptState::Ready, Some(opened)) => {
                    finish_approval(
                        self.inner.backend.as_ref(),
                        &session,
                        &seen[at].pending,
                        &opened.approval,
                    )
                    .map_err(|err| self.fail(&session, &err))?;
                }
                (PromptState::Changed, _) => {
                    return Err(account_error(
                        &AccountError::ApprovalChanged,
                        ErrorContext::Account,
                    ));
                }
                _ => {
                    return Err(KansoError::invalid(
                        "That device hasn't shown its name and code yet. Wait a moment.",
                    ));
                }
            }
            seen.remove(at);
            requests.by_id.remove(&id);
            let left = prompts(seen);
            self.inner.approver.listeners.send(left.clone());
            Ok(left)
        })
    }

    /// Declines a request and gives the requests that remain. One that is
    /// already gone counts as declined.
    pub fn decline(&self, id: String) -> Result<Vec<ApprovalPrompt>, KansoError> {
        guard(|| {
            let mut requests = held(&self.inner.approver.requests);
            // Without answering the ones not yet answered: the person is
            // saying no.
            let mut seen = self.look(&mut requests, false)?;
            let session = self.session()?;
            decline_request(self.inner.backend.as_ref(), &session, &id)
                .map_err(|err| self.fail(&session, &err))?;
            seen.retain(|one| one.prompt.id != id);
            requests.by_id.remove(&id);
            let left = prompts(seen);
            self.inner.approver.listeners.send(left.clone());
            Ok(left)
        })
    }

    /// The signed-in, unlocked session. A phone that is locked or signed
    /// out has no data key to give, so it approves nothing.
    fn session(&self) -> Result<Session, KansoError> {
        SyncHandle {
            inner: self.inner.clone(),
        }
        .load_session()?
        .ok_or(KansoError::SignedOut)
    }

    /// A failed call as the shell hears it. A 401 means the server ended
    /// this phone's session.
    fn fail(&self, session: &Session, err: &AccountError) -> KansoError {
        if let AccountError::Unauthorized { reason } = err {
            session_ended(&self.inner, session, *reason);
        }
        account_error(err, ErrorContext::Account)
    }

    /// Reads the requests, and, when `answer` is set, answers the ones this
    /// phone hasn't. Called with the request list held. When the account is
    /// signed out, forgets every key.
    fn look(&self, requests: &mut Requests, answer: bool) -> Result<Vec<Seen>, KansoError> {
        let seen = self.look_inner(requests, answer);
        if matches!(seen, Err(KansoError::SignedOut)) {
            requests.by_id.clear();
        }
        seen
    }

    fn look_inner(&self, requests: &mut Requests, answer: bool) -> Result<Vec<Seen>, KansoError> {
        let session = self.session()?;
        let backend = self.inner.backend.as_ref();
        let list = pending_approvals(backend, &session).map_err(|err| self.fail(&session, &err))?;
        // A request that ended takes its key with it.
        requests
            .by_id
            .retain(|id, _| list.iter().any(|pending| pending.id == *id));
        let mut seen = Vec::with_capacity(list.len());
        for pending in list {
            // Another device answered first: not this phone's to show.
            if answered_elsewhere(&session, &pending) {
                requests.by_id.remove(&pending.id);
                continue;
            }
            let tracked = requests
                .by_id
                .entry(pending.id.clone())
                .or_insert_with(|| Tracked::new(&pending));
            let mut step = front_step(&session, &pending, &tracked.key, tracked.challenged);
            if answer && matches!(step, FrontStep::Challenge) {
                match challenge_request(backend, &session, &pending, &tracked.key.keys) {
                    Ok(()) => {
                        tracked.challenged = true;
                        step = FrontStep::Wait;
                    }
                    Err(err @ AccountError::Unauthorized { .. }) => {
                        return Err(self.fail(&session, &err));
                    }
                    Err(
                        AccountError::ApprovalUsed
                        | AccountError::ApprovalExpired
                        | AccountError::NotFound,
                    ) => {
                        requests.by_id.remove(&pending.id);
                        continue;
                    }
                    // Not answered yet; the next read tries again.
                    Err(err) => eprintln!("kanso: answering a device request: {err}"),
                }
            }
            seen.push(Seen::of(pending, step));
        }
        Ok(seen)
    }

    /// Tells the listeners where the requests stand: the list, or none
    /// when the account is signed out. A failed read says nothing.
    fn tell(&self, seen: &Result<Vec<Seen>, KansoError>) {
        let listeners = &self.inner.approver.listeners;
        match seen {
            Ok(seen) => listeners.send(seen.iter().map(|one| one.prompt.clone()).collect()),
            Err(KansoError::SignedOut) => listeners.send(Vec::new()),
            Err(_) => {}
        }
    }
}

/// Forgets every request's key, stops the event socket's thread and tells
/// the listeners there is nothing to approve: the account is gone from this
/// phone. The sinks stay until they say they are done.
pub(crate) fn forget(inner: &Inner) {
    if let Some(stop) = held(&inner.approver.watching).take() {
        stop.store(true, Ordering::SeqCst);
    }
    held(&inner.approver.requests).by_id.clear();
    inner.approver.listeners.send(Vec::new());
}

/// A request as this phone sees it.
pub(crate) struct Seen {
    pub(crate) pending: PendingApproval,
    pub(crate) prompt: ApprovalPrompt,
    /// The name and approval, once the request checks out.
    pub(crate) opened: Option<OpenedRequest>,
}

impl Seen {
    /// The request after `step`: what the shell shows, and what approving
    /// needs.
    pub(crate) fn of(pending: PendingApproval, step: FrontStep) -> Seen {
        let (state, opened) = match step {
            FrontStep::Challenge => (PromptState::New, None),
            FrontStep::Wait => (PromptState::Challenged, None),
            FrontStep::Open(opened) => (PromptState::Ready, Some(opened)),
            FrontStep::Refuse(_) => (PromptState::Changed, None),
        };
        let prompt = ApprovalPrompt {
            id: pending.id.clone(),
            device_name: opened.as_ref().map(|opened| device_name(&opened.name)),
            asked_at_ms: pending.created_at,
            expires_at_ms: pending.expires_at,
            code: opened.as_ref().map(|opened| opened.approval.code()),
            state,
        };
        Seen {
            pending,
            prompt,
            opened,
        }
    }
}

fn prompts(seen: Vec<Seen>) -> Vec<ApprovalPrompt> {
    seen.into_iter().map(|one| one.prompt).collect()
}

/// Whether a device other than this phone has already answered `pending`.
fn answered_elsewhere(session: &Session, pending: &PendingApproval) -> bool {
    pending
        .approver_device_id
        .as_deref()
        .is_some_and(|device| device != session.device_id)
}

// ---------------------------------------------------------------------------
// The event socket
// ---------------------------------------------------------------------------

/// The thread behind [`Approvals::watch`]: reads the list, then keeps the
/// account event socket open until `stop` is set or the loop ends for good.
fn watch_events(owner: &Weak<Inner>, stop: &Arc<AtomicBool>) {
    if refresh_from(owner) {
        let mut connect = || connect(owner, stop);
        let mut on = |signal: Signal| match signal {
            Signal::Connected | Signal::Event(_) => {
                if !refresh_from(owner) {
                    stop.store(true, Ordering::SeqCst);
                }
            }
            Signal::Ended(err) => {
                eprintln!("kanso: the account event socket ended: {err:?}");
                stop.store(true, Ordering::SeqCst);
            }
        };
        let sleep = |wait: Duration| events::sleep_unless_stopped(stop, wait);
        events::run(&mut connect, stop, &mut on, &sleep);
    }
    // Over: let the next `watch` start a thread, and let go of the keys
    // of an account that is gone.
    let Some(inner) = owner.upgrade() else {
        return;
    };
    let mut watching = held(&inner.approver.watching);
    if watching
        .as_ref()
        .is_some_and(|flag| Arc::ptr_eq(flag, stop))
    {
        *watching = None;
    }
    drop(watching);
    let signed_in = matches!(
        SyncHandle {
            inner: inner.clone()
        }
        .load_session(),
        Ok(Some(_))
    );
    if !signed_in {
        held(&inner.approver.requests).by_id.clear();
    }
}

/// Reads the list for the listeners. `false` when the thread should end:
/// the library is closed or the account is signed out.
fn refresh_from(owner: &Weak<Inner>) -> bool {
    let Some(inner) = owner.upgrade() else {
        return false;
    };
    let approvals = Approvals { inner };
    let mut requests = held(&approvals.inner.approver.requests);
    let seen = guard(|| approvals.look(&mut requests, true));
    approvals.tell(&seen);
    !matches!(seen, Err(KansoError::SignedOut))
}

/// Opens the socket for the session saved now: it may not be the one the
/// thread started with.
fn connect(
    owner: &Weak<Inner>,
    stop: &Arc<AtomicBool>,
) -> Result<Box<dyn EventSource>, EventError> {
    let Some(inner) = owner.upgrade() else {
        stop.store(true, Ordering::SeqCst);
        return Err(EventError::Dropped("the library was closed".into()));
    };
    let sync = SyncHandle {
        inner: inner.clone(),
    };
    let session = match sync.load_session() {
        Ok(Some(session)) => session,
        Ok(None) => {
            return Err(EventError::Refused(ApiError::Unauthorized {
                reason: SignOutReason::SignedOut,
            }));
        }
        Err(err) => return Err(EventError::Dropped(err.to_string())),
    };
    let device = sync
        .flow("")
        .device()
        .map_err(|failure| EventError::Dropped(failure.words))?;
    let path = account_events_path(inner.backend.as_ref(), &session.server)
        .ok()
        .flatten()
        .unwrap_or_else(|| ACCOUNT_EVENTS_PATH.to_owned());
    let source = inner.backend.events(&session, &device, &path)?;
    Ok(Box::new(Guarded {
        inner: source,
        owner: owner.clone(),
        stop: stop.clone(),
        reads: 0,
    }))
}

/// An event socket that ends its thread by itself: once the library is
/// closed, no sink listens, or the account is signed out. The loop only
/// looks at its stop flag between reads, so this is where it learns.
struct Guarded {
    inner: Box<dyn EventSource>,
    owner: Weak<Inner>,
    stop: Arc<AtomicBool>,
    reads: u32,
}

impl Guarded {
    fn alive(&mut self) -> bool {
        let Some(inner) = self.owner.upgrade() else {
            return false;
        };
        {
            // Under the lock `watch` adds a sink with, so a sink added
            // just now is either counted or finds the thread gone and
            // starts another.
            let mut watching = held(&inner.approver.watching);
            if inner.approver.sinks.load(Ordering::SeqCst) == 0 {
                if watching
                    .as_ref()
                    .is_some_and(|flag| Arc::ptr_eq(flag, &self.stop))
                {
                    *watching = None;
                }
                return false;
            }
        }
        self.reads = self.reads.wrapping_add(1);
        if self.reads.is_multiple_of(SESSION_CHECK_EVERY) {
            let session = SyncHandle { inner }.load_session();
            return !matches!(session, Ok(None));
        }
        true
    }
}

impl EventSource for Guarded {
    fn next(&mut self, wait: Duration) -> Result<Option<events::Frame>, EventError> {
        if !self.alive() {
            self.stop.store(true, Ordering::SeqCst);
            return Ok(None);
        }
        self.inner.next(wait)
    }

    fn ping(&mut self) -> Result<(), EventError> {
        self.inner.ping()
    }
}

#[cfg(test)]
mod tests;
