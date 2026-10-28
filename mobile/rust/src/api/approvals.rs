//! The requests of other devices to join the account, which this phone may
//! approve.

use crate::frb_generated::StreamSink;
use flutter_rust_bridge::frb;
use kanso_mobile::{ApprovalPrompt, Approvals, KansoError};

#[frb(opaque)]
pub struct KansoApprovals(Approvals);

impl KansoApprovals {
    pub(super) fn new(approvals: Approvals) -> KansoApprovals {
        KansoApprovals(approvals)
    }

    /// The requests waiting now and each time they change, until Dart stops
    /// listening.
    pub fn watch(&self, sink: StreamSink<Vec<ApprovalPrompt>>) {
        self.0.watch(move |prompts| sink.add(prompts).is_ok());
    }

    pub fn refresh(&self) -> Result<Vec<ApprovalPrompt>, KansoError> {
        self.0.refresh()
    }

    pub fn approve(&self, id: String) -> Result<Vec<ApprovalPrompt>, KansoError> {
        self.0.approve(id)
    }

    pub fn decline(&self, id: String) -> Result<Vec<ApprovalPrompt>, KansoError> {
        self.0.decline(id)
    }
}
