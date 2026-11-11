//! The Agent Client Protocol, from Kanso's side: Kanso is the client, and an
//! agent the person already has (Claude Agent, Codex, Gemini CLI, OpenCode,
//! or their own command) answers.
//!
//! An agent is a child process speaking JSON-RPC 2.0 over newline-delimited
//! JSON on stdio. Kanso advertises no client capabilities (no file system, no
//! terminal), refuses every method it doesn't offer, rejects permission
//! requests unless a person allows one, and starts the agent in an empty
//! private folder with a scrubbed environment. None of that is a sandbox:
//! the agent runs as the person, with its own tools.
//!
//! This crate knows nothing about notes, the store or the UI. Everything is
//! synchronous: a reader thread per agent routes its messages, and a prompt
//! turn is a blocking iterator of [`AcpEvent`]s.

mod connection;
pub mod detect;
pub mod env;
pub mod fake;
pub mod framing;
pub mod process;
pub mod registry;

pub use connection::{
    AgentInfo, AgentProcess, AuthMethod, McpServer, PermissionPolicy, PermissionReply,
    PermissionRequest, Session, SessionMode, Turn, parse_tool_call, reject_all,
};
pub use detect::{AgentKind, Detection};
pub use process::AgentSpec;

use std::time::Duration;

/// The protocol version Kanso speaks.
pub const PROTOCOL_VERSION: u64 = 1;

/// How long each part of an exchange may take.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    /// From starting the process to the answer to `initialize`.
    pub initialize: Duration,
    /// The same when the command is `npx`, which may download first.
    pub initialize_cold: Duration,
    pub session_new: Duration,
    /// Before the first words: after this the turn says it is still working,
    /// but doesn't fail.
    pub first_text: Duration,
    /// The whole answer, after which it is cancelled.
    pub answer: Duration,
    /// From `session/cancel` to the agent's stop reason, after which the
    /// agent is killed.
    pub cancel_grace: Duration,
    /// How long a permission request waits for a person before it is
    /// rejected.
    pub permission: Duration,
    /// A full event queue for this long means the agent is killed.
    pub stuck: Duration,
    /// The longest one protocol line may be.
    pub max_line: usize,
    /// How much of the agent's stderr is kept.
    pub stderr: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Limits {
            initialize: Duration::from_secs(20),
            initialize_cold: Duration::from_secs(30),
            session_new: Duration::from_secs(20),
            first_text: Duration::from_secs(60),
            answer: Duration::from_secs(10 * 60),
            cancel_grace: Duration::from_secs(3),
            permission: Duration::from_secs(60),
            stuck: Duration::from_secs(10),
            max_line: 8 * 1024 * 1024,
            stderr: 64 * 1024,
        }
    }
}

/// Why an agent's turn ended.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StopReason {
    EndTurn,
    MaxTokens,
    MaxTurnRequests,
    Refusal,
    Cancelled,
    Other(String),
}

impl StopReason {
    fn parse(text: &str) -> StopReason {
        match text {
            "end_turn" => StopReason::EndTurn,
            "max_tokens" => StopReason::MaxTokens,
            "max_turn_requests" => StopReason::MaxTurnRequests,
            "refusal" => StopReason::Refusal,
            "cancelled" => StopReason::Cancelled,
            other => StopReason::Other(other.to_owned()),
        }
    }
}

/// How one of the agent's own tool calls is going.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

impl ToolStatus {
    /// `pending`, `in_progress`, `completed` or `failed`; anything else is
    /// not a status.
    pub fn parse(text: &str) -> Option<ToolStatus> {
        match text {
            "pending" => Some(ToolStatus::Pending),
            "in_progress" => Some(ToolStatus::InProgress),
            "completed" => Some(ToolStatus::Completed),
            "failed" => Some(ToolStatus::Failed),
            _ => None,
        }
    }
}

/// Something an agent's turn says.
#[derive(Clone, Debug, PartialEq)]
pub enum AcpEvent {
    /// Words of the answer, to add to what came before.
    Text(String),
    /// The agent's reasoning, not part of the answer.
    Thought(String),
    /// The agent started one of its own tools or reported on it
    /// (`tool_call` / `tool_call_update`). Only `id` is always there: an
    /// update that carries no title updates the call it names.
    ToolCall {
        id: String,
        title: Option<String>,
        kind: Option<String>,
        status: Option<ToolStatus>,
    },
    /// What the answer has cost so far, when the agent says.
    Cost { amount: f64, currency: String },
    /// A line for the person that isn't part of the answer.
    Notice(String),
    /// No words yet after [`Limits::first_text`]; the turn goes on.
    StillWorking,
    /// The turn is over.
    Done(StopReason),
    /// The turn failed; what arrived before stays.
    Error(AcpError),
}

/// The stage a timeout happened in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    Initialize,
    SessionNew,
    Request,
    Answer,
}

/// What went wrong with an agent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AcpError {
    /// The process couldn't be started.
    Spawn(String),
    Timeout(Stage),
    /// The agent speaks another protocol version.
    ProtocolVersion(u64),
    /// The agent needs the person to sign in.
    AuthRequired(String),
    /// The agent answered a request with an error.
    Agent {
        code: i64,
        message: String,
    },
    /// The agent sent something that isn't the protocol.
    Protocol(String),
    /// One line was longer than [`Limits::max_line`].
    TooLarge,
    /// The process ended, with its exit code if it had one.
    Exited(Option<i32>),
    /// Kanso closed the connection.
    Closed,
    /// The agent didn't stop within [`Limits::cancel_grace`] of a cancel.
    CancelIgnored,
    /// The agent wrote faster than Kanso could take its words.
    Stuck,
}

/// JSON-RPC's code for a method nobody offers.
pub const METHOD_NOT_FOUND: i64 = -32601;
/// ACP's code for "authentication required".
pub const AUTH_REQUIRED: i64 = -32000;

impl AcpError {
    /// A plain sentence about the error, naming the agent as `label`.
    pub fn describe(&self, label: &str) -> String {
        match self {
            AcpError::Spawn(why) => format!("{label} didn't start. {why}"),
            AcpError::Timeout(Stage::Initialize) => {
                format!("{label} didn't finish starting. Its log may say why.")
            }
            AcpError::Timeout(Stage::SessionNew) => format!("{label} didn't open a conversation."),
            AcpError::Timeout(Stage::Request) => format!("{label} didn't answer in time."),
            AcpError::Timeout(Stage::Answer) => {
                format!("{label} took longer than 10 minutes, so Kanso stopped it.")
            }
            AcpError::ProtocolVersion(version) => format!(
                "This agent speaks ACP version {version}; Kanso needs version {PROTOCOL_VERSION}. Update the agent."
            ),
            AcpError::AuthRequired(_) => format!("{label} needs you to sign in."),
            AcpError::Agent { message, .. } => format!("The agent reported an error: {message}"),
            AcpError::Protocol(why) => {
                format!("{label} sent something Kanso couldn't read ({why}).")
            }
            AcpError::TooLarge => "The agent sent an answer that was too large.".to_owned(),
            AcpError::Exited(Some(code)) => {
                format!("{label} stopped unexpectedly (exit code {code}).")
            }
            AcpError::Exited(None) => format!("{label} stopped unexpectedly."),
            AcpError::Closed => format!("{label} was closed."),
            AcpError::CancelIgnored => "Stopped. The agent did not finish cancelling.".to_owned(),
            AcpError::Stuck => {
                format!("{label} sent more than Kanso could take, so it was stopped.")
            }
        }
    }
}

impl std::fmt::Display for AcpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.describe("The agent"))
    }
}

impl std::error::Error for AcpError {}
