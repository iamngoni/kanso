//! What this phone keeps while it answers other devices' requests to join
//! the account: its ephemeral key for each request and whether the request
//! was answered, and the listeners and the event socket's thread that
//! `Approvals::watch` runs.
//!
//! Memory only. The keys are zeroized when a request ends (they drop with
//! the entry), and nothing here is written to the secret store: a request
//! the app was closed on is answered again from scratch, which the
//! requester sees as a changed key (see `approvals`).

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use kanso_engine::account::approval::ApproverKey;
use kanso_sync::PendingApproval;

use crate::account_model::ApprovalPrompt;
use crate::stream::Listeners;

/// This phone's side of one request.
pub(crate) struct Tracked {
    /// The key made when the request was first seen, with the request as
    /// it read then: what the revealed key is checked against.
    pub(crate) key: ApproverKey,
    /// This phone has posted its key for the request.
    pub(crate) challenged: bool,
}

impl Tracked {
    /// The record for a request seen for the first time. Pins it as it
    /// reads now, before any key of this phone's is posted.
    pub(crate) fn new(pending: &PendingApproval) -> Tracked {
        Tracked {
            key: ApproverKey::new(pending),
            challenged: false,
        }
    }
}

/// The requests this phone has seen, by their IDs.
#[derive(Default)]
pub(crate) struct Requests {
    pub(crate) by_id: BTreeMap<String, Tracked>,
}

/// The approver of one library: held by `library::Inner`.
#[derive(Default)]
pub(crate) struct Approver {
    /// Held for the length of a read, a challenge, an approval or a
    /// decline: one at a time, so a request is never answered twice.
    pub(crate) requests: Mutex<Requests>,
    /// The stop flag of the thread that holds the event socket, while one
    /// runs.
    pub(crate) watching: Mutex<Option<Arc<AtomicBool>>>,
    /// Who hears the list of requests.
    pub(crate) listeners: Listeners<Vec<ApprovalPrompt>>,
    /// How many of the listeners are alive. The listeners themselves can't
    /// say while a list is being sent to them.
    pub(crate) sinks: Arc<AtomicUsize>,
}

/// One listener, counted: held by its sink, so it is gone when the sink is,
/// whether it said it was done, panicked or went with the library.
pub(crate) struct Counted(Arc<AtomicUsize>);

impl Counted {
    pub(crate) fn new(count: &Arc<AtomicUsize>) -> Counted {
        count.fetch_add(1, Ordering::SeqCst);
        Counted(count.clone())
    }
}

impl Drop for Counted {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}
