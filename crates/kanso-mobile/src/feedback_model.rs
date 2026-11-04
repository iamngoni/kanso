//! The data Send feedback and My feedback pass between the phone's shell and
//! the library (issue #232). Plain values; the rules are in
//! `kanso_engine::feedback`.

/// What the shell knows about the phone and the app. Nothing here comes from
/// a note: the shell reads the app's version and the system's, and says which
/// screen of the app the person was on.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct FeedbackContext {
    /// `0.9.4`; a build suffix is cut.
    pub version: String,
    /// The build number, digits only.
    pub build: String,
    /// The OS version, `18.1` or `15`.
    pub os: String,
    /// The device model, `iPhone15,2` or `Pixel 8`.
    pub device: String,
    /// `en-GB`.
    pub locale: String,
    /// One of the fixed screens (`settings`, `editor`…); anything else is `other`.
    pub screen: String,
    /// The type names of the crash reports waiting on the device.
    pub crash_types: Vec<String>,
}

/// A report as the person filled it in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FeedbackInput {
    /// `bug`, `idea` or `question`.
    pub kind: String,
    pub text: String,
    /// `Some` when "Credit me by name" is on.
    pub credit_name: Option<String>,
    /// Whether "Include app info" is on.
    pub include_app: bool,
    /// The chosen image's bytes (PNG or JPEG).
    pub screenshot: Option<Vec<u8>>,
    pub context: FeedbackContext,
}

/// Why a report can't be sent yet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeedbackProblem {
    TooShort,
    TooLong,
    BadCharacter,
    BadCredit,
    BadScreenshot,
    Other,
}

/// "Show what's sent": the exact JSON, or why there is none yet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FeedbackPreview {
    pub json: String,
    pub problem: Option<FeedbackProblem>,
}

/// How a send went.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FeedbackSent {
    /// The server has it.
    Sent,
    /// Saved; it goes when the phone is back online.
    Queued,
    /// Feedback is paused on the server; the report is saved.
    Paused,
    /// Too many today; the report is saved and waits.
    RateLimited,
    /// The server refused this field; nothing was kept.
    Rejected { field: String },
    /// The report can't be sent as written.
    Invalid { problem: FeedbackProblem },
}

/// What a report's chip says; the shell words it in its language.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FeedbackChipKind {
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

/// One report in My feedback.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FeedbackReport {
    pub id: String,
    pub created_ms: i64,
    /// `bug`, `idea` or `question`.
    pub kind: String,
    /// The first line of the text.
    pub preview: String,
    pub chip: FeedbackChipKind,
    /// The issue number for `FiledAs` and `SameAs`.
    pub number: Option<u32>,
    /// `0.9.2` for `ReleasedIn`.
    pub release: Option<String>,
    /// The pull request that fixes it, once there is one.
    pub pr: Option<u32>,
    /// Where the issue is on GitHub, once filed.
    pub issue_url: Option<String>,
    /// A delete is waiting for the server.
    pub deleting: bool,
    /// Released, and not yet looked at in My feedback.
    pub fresh: bool,
    /// Whether the report was sent (not still waiting on this phone).
    pub on_server: bool,
}
