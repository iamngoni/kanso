//! In-app feedback: Send feedback and My feedback (issue #232). The rules,
//! the outbox and the retry live in `kanso-mobile` / `kanso-engine`; this is
//! only the thin bridge to Dart. The data types are declared again for the
//! generator; what they mean is said where the facade declares them
//! (`crates/kanso-mobile/src/feedback_model.rs`).

use flutter_rust_bridge::frb;
// Public: the generated code names these types by this module's path.
pub use kanso_mobile::{
    FeedbackChipKind, FeedbackContext, FeedbackInput, FeedbackPreview, FeedbackProblem,
    FeedbackReport, FeedbackSent,
};
use kanso_mobile::{KansoError, MobileFeedback};

#[frb(mirror(FeedbackContext))]
pub struct _FeedbackContext {
    pub version: String,
    pub build: String,
    pub os: String,
    pub device: String,
    pub locale: String,
    pub screen: String,
    pub crash_types: Vec<String>,
}

#[frb(mirror(FeedbackInput))]
pub struct _FeedbackInput {
    pub kind: String,
    pub text: String,
    pub credit_name: Option<String>,
    pub include_app: bool,
    pub screenshot: Option<Vec<u8>>,
    pub context: FeedbackContext,
}

#[frb(mirror(FeedbackProblem))]
pub enum _FeedbackProblem {
    TooShort,
    TooLong,
    BadCharacter,
    BadCredit,
    BadScreenshot,
    Other,
}

#[frb(mirror(FeedbackPreview))]
pub struct _FeedbackPreview {
    pub json: String,
    pub problem: Option<FeedbackProblem>,
}

#[frb(mirror(FeedbackSent))]
pub enum _FeedbackSent {
    Sent,
    Queued,
    Paused,
    RateLimited,
    Rejected { field: String },
    Invalid { problem: FeedbackProblem },
}

#[frb(mirror(FeedbackChipKind))]
pub enum _FeedbackChipKind {
    WaitingToSend,
    Received,
    FiledAs,
    Building,
    FixProposed,
    Merged,
    ReleasedIn,
    NotFiled,
    SameAs,
    Closed,
    Deleting,
}

#[frb(mirror(FeedbackReport))]
pub struct _FeedbackReport {
    pub id: String,
    pub created_ms: i64,
    pub kind: String,
    pub preview: String,
    pub chip: FeedbackChipKind,
    pub number: Option<u32>,
    pub release: Option<String>,
    pub pr: Option<u32>,
    pub issue_url: Option<String>,
    pub deleting: bool,
    pub fresh: bool,
    pub on_server: bool,
}

/// Feedback for one library.
#[frb(opaque)]
pub struct KansoFeedback(MobileFeedback);

impl KansoFeedback {
    pub(super) fn new(feedback: MobileFeedback) -> KansoFeedback {
        KansoFeedback(feedback)
    }

    /// The exact JSON "Show what's sent" shows, or why there is none yet.
    pub fn preview(&self, input: FeedbackInput) -> Result<FeedbackPreview, KansoError> {
        self.0.preview(input)
    }

    /// Saves the report to the outbox and tries to send it.
    pub fn send(&self, input: FeedbackInput) -> Result<FeedbackSent, KansoError> {
        self.0.send(input)
    }

    /// Sends what is waiting (app resume).
    pub fn flush(&self) -> Result<(), KansoError> {
        self.0.flush()
    }

    /// Makes waiting reports due again (connectivity returned).
    pub fn retry_now(&self) -> Result<(), KansoError> {
        self.0.retry_now()
    }

    /// The reports, newest first; `refresh` asks the server first.
    pub fn reports(&self, refresh: bool) -> Result<Vec<FeedbackReport>, KansoError> {
        self.0.reports(refresh)
    }

    /// Whether the launch refresh is due (at most every 6 hours).
    #[frb(sync)]
    pub fn refresh_due(&self) -> bool {
        self.0.refresh_due()
    }

    /// Deletes a report here and on the server.
    pub fn delete(&self, id: String) -> Result<(), KansoError> {
        self.0.delete(id)
    }

    /// The released reports were looked at.
    pub fn mark_seen(&self) -> Result<(), KansoError> {
        self.0.mark_seen()
    }
}
