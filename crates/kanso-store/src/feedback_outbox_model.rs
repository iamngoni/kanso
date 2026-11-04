//! The rows of the feedback outbox (issue #232).

/// Where a report is on this device.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutboxState {
    /// Saved here, not yet accepted by the server.
    Queued,
    /// The server has it; the row now mirrors its status.
    Sent,
    /// The person deleted it and the server hasn't confirmed yet.
    Deleting,
}

impl OutboxState {
    pub fn as_str(self) -> &'static str {
        match self {
            OutboxState::Queued => "queued",
            OutboxState::Sent => "sent",
            OutboxState::Deleting => "deleting",
        }
    }

    pub fn parse(text: &str) -> Option<OutboxState> {
        match text {
            "queued" => Some(OutboxState::Queued),
            "sent" => Some(OutboxState::Sent),
            "deleting" => Some(OutboxState::Deleting),
            _ => None,
        }
    }
}

/// A report to save before it goes anywhere.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutboxNew {
    /// `fb_` and 26 base32 characters.
    pub id: String,
    /// The exact JSON the person was shown.
    pub payload: String,
    /// `bug`, `idea` or `question`.
    pub kind: String,
    /// The first line of the person's text, for the list.
    pub preview: String,
    /// A copy of the screenshot in the data folder, deleted after upload.
    pub screenshot_path: Option<String>,
    pub created_ms: i64,
    /// Who sends it (`anonymous` or `account:<id>`): the server only lets
    /// that reporter list, upload to or delete it.
    pub sender: String,
}

/// One report on this device.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OutboxRow {
    pub id: String,
    pub payload: String,
    pub kind: String,
    pub preview: String,
    pub screenshot_path: Option<String>,
    /// The screenshot's bytes are up (or there is none to send).
    pub screenshot_sent: bool,
    pub state: OutboxState,
    /// The server's last status word.
    pub server_status: Option<String>,
    pub issue: Option<u32>,
    pub duplicate_of: Option<u32>,
    pub pr: Option<u32>,
    pub release: Option<String>,
    pub attempts: u32,
    pub next_try_ms: i64,
    pub created_ms: i64,
    /// Who sent it; empty for a row that doesn't say.
    pub sender: String,
}

/// What the server last said about a report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerStatus {
    pub status: String,
    pub issue: Option<u32>,
    pub duplicate_of: Option<u32>,
    pub pr: Option<u32>,
    pub release: Option<String>,
}
