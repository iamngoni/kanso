//! What happened in an agent task, as a conversation (issue #375): the
//! person's goal and follow-ups, the agent's words, each tool it ran and how
//! that went, what it was allowed, what it proposed. A model file: values and
//! the rules for adding to them, nothing that runs or draws.

use kanso_server::{ChangeSet, Counts};
use serde::{Deserialize, Serialize};

use super::TaskSpec;
use super::model::Phase;

/// A session's own id. A [`kanso_server::TaskId`] restarts with each launch;
/// this one does not, so it can name a file on disk. 26 characters of
/// Crockford base32, ULID-shaped: 48 bits of time, then 80 random bits, so
/// ids sort by when they were made.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SessionKey(pub String);

const CROCKFORD: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

impl SessionKey {
    /// A fresh key for a session started at `now_ms` (Unix milliseconds).
    pub fn new(now_ms: i64) -> SessionKey {
        let time = u64::try_from(now_ms).unwrap_or(0) & ((1 << 48) - 1);
        let mut text = String::with_capacity(26);
        for ix in (0..10).rev() {
            text.push(char::from(CROCKFORD[((time >> (ix * 5)) & 31) as usize]));
        }
        for _ in 0..16 {
            text.push(char::from(CROCKFORD[fastrand::usize(0..32)]));
        }
        SessionKey(text)
    }
}

/// What a tool call does, as the agent says (ACP's `kind`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolKind {
    Read,
    Edit,
    Delete,
    Move,
    Search,
    Execute,
    Think,
    Fetch,
    SwitchMode,
    #[default]
    Other,
}

impl ToolKind {
    /// The kind a protocol word names; anything unknown is `Other`.
    pub fn parse(text: &str) -> ToolKind {
        match text {
            "read" => ToolKind::Read,
            "edit" => ToolKind::Edit,
            "delete" => ToolKind::Delete,
            "move" => ToolKind::Move,
            "search" => ToolKind::Search,
            "execute" => ToolKind::Execute,
            "think" => ToolKind::Think,
            "fetch" => ToolKind::Fetch,
            "switch_mode" => ToolKind::SwitchMode,
            _ => ToolKind::Other,
        }
    }
}

/// How a tool call went.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolStatus {
    Pending,
    #[default]
    InProgress,
    Completed,
    Failed,
}

/// What the person answered when the agent asked for a tool.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionAnswer {
    AllowedOnce,
    AllowedForSession,
    AllowedAlways,
    Rejected,
}

/// Where a round's proposal stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoundState {
    ToReview,
    Applied,
    Undone,
    Discarded,
    /// The change set is gone (Kanso quit before it was reviewed).
    Lost,
}

/// One thing in the conversation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Item {
    User {
        text: String,
        at_ms: i64,
    },
    /// Streamed chunks append to the last Agent item.
    Agent {
        text: String,
        at_ms: i64,
    },
    Thought {
        text: String,
        ms: u64,
    },
    Tool {
        id: String,
        title: String,
        kind: ToolKind,
        status: ToolStatus,
    },
    Permission {
        title: String,
        kind: ToolKind,
        answer: PermissionAnswer,
    },
    Proposed {
        round: u32,
        counts: Counts,
        state: RoundState,
    },
    Notice {
        text: String,
        at_ms: i64,
    },
}

/// The conversation so far. `version` moves with every change, so a view
/// can tell whether it needs to look again.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transcript {
    pub items: Vec<Item>,
    pub version: u64,
}

impl Transcript {
    pub fn push(&mut self, item: Item) {
        self.items.push(item);
        self.version += 1;
    }

    pub fn user(&mut self, text: &str, at_ms: i64) {
        self.push(Item::User {
            text: text.to_owned(),
            at_ms,
        });
    }

    /// The person answered a request to use a tool.
    pub fn permission(&mut self, title: &str, kind: ToolKind, answer: PermissionAnswer) {
        self.push(Item::Permission {
            title: title.to_owned(),
            kind,
            answer,
        });
    }

    pub fn notice(&mut self, text: &str, at_ms: i64) {
        self.push(Item::Notice {
            text: text.to_owned(),
            at_ms,
        });
    }

    /// The agent said `chunk`: it goes on the end of the last item when that
    /// is the agent's words, and starts a new item otherwise.
    pub fn agent_text(&mut self, chunk: &str, at_ms: i64) {
        if chunk.is_empty() {
            return;
        }
        if let Some(Item::Agent { text, .. }) = self.items.last_mut() {
            text.push_str(chunk);
            self.version += 1;
        } else {
            self.push(Item::Agent {
                text: chunk.to_owned(),
                at_ms,
            });
        }
    }

    /// The agent thought `chunk`, `ms` into thinking.
    pub fn thought(&mut self, chunk: &str, ms: u64) {
        if chunk.is_empty() {
            return;
        }
        if let Some(Item::Thought { text, ms: was }) = self.items.last_mut() {
            text.push_str(chunk);
            *was = ms;
            self.version += 1;
        } else {
            self.push(Item::Thought {
                text: chunk.to_owned(),
                ms,
            });
        }
    }

    /// A tool call started or moved on. An update names its call by `id` and
    /// changes only what it carries; an id not seen before is a new call.
    pub fn tool(
        &mut self,
        id: &str,
        title: Option<&str>,
        kind: Option<ToolKind>,
        status: Option<ToolStatus>,
    ) {
        let found = self
            .items
            .iter_mut()
            .rev()
            .find(|item| matches!(item, Item::Tool { id: have, .. } if have == id));
        if let Some(Item::Tool {
            title: have_title,
            kind: have_kind,
            status: have_status,
            ..
        }) = found
        {
            if let Some(title) = title {
                *have_title = title.to_owned();
            }
            if let Some(kind) = kind {
                *have_kind = kind;
            }
            if let Some(status) = status {
                *have_status = status;
            }
            self.version += 1;
        } else {
            self.push(Item::Tool {
                id: id.to_owned(),
                title: title.unwrap_or("Using a tool").to_owned(),
                kind: kind.unwrap_or_default(),
                status: status.unwrap_or_default(),
            });
        }
    }

    /// Where round `round`'s proposal stands now.
    pub fn set_round_state(&mut self, round: u32, state: RoundState) {
        let found = self.items.iter_mut().rev().find_map(|item| match item {
            Item::Proposed {
                round: have, state, ..
            } if *have == round => Some(state),
            _ => None,
        });
        if let Some(have) = found
            && *have != state
        {
            *have = state;
            self.version += 1;
        }
    }

    /// Everything the agent said, its items joined with nothing between.
    pub fn agent_said(&self) -> String {
        self.items
            .iter()
            .filter_map(|item| match item {
                Item::Agent { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect()
    }
}

/// A session as it is kept on disk (`store.rs`): everything needed to show
/// it again after a restart. Never holds a token or an API key.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedSession {
    /// The file format; a file of another version is skipped.
    pub version: u32,
    pub key: SessionKey,
    pub title: String,
    /// What the person calls the agent.
    pub agent: String,
    /// Which agent runs it, so a resume starts the same one. Empty when not
    /// known.
    pub agent_key: String,
    /// The agent's own session id, for `session/load`.
    pub acp_session_id: Option<String>,
    pub spec: TaskSpec,
    pub created_ms: i64,
    pub last_activity_ms: i64,
    pub phase: Phase,
    /// The round of proposals the task was in.
    pub round: u32,
    pub items: Vec<Item>,
    /// The proposal nobody reviewed yet.
    pub pending: Option<ChangeSet>,
    /// The session this one carries on from ("Continued from …").
    #[serde(default)]
    pub continued_from: Option<SessionKey>,
    /// The session that carries this one on.
    #[serde(default)]
    pub continued_in: Option<SessionKey>,
}

/// The most messages of an old session a new one's first message carries.
pub const SUMMARY_MESSAGES: usize = 20;
/// The most bytes of that summary.
pub const SUMMARY_BYTES: usize = 24 * 1024;

/// `text` cut to at most `max` bytes, on a character boundary, with "…" when
/// something went.
fn cut(text: &str, max: usize) -> String {
    if text.len() <= max {
        return text.to_owned();
    }
    let room = max.saturating_sub('…'.len_utf8());
    let mut end = room.min(text.len());
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…", &text[..end])
}

/// What an old session says for a new one to start from: its last
/// [`SUMMARY_MESSAGES`] messages (the person's and the agent's), with the
/// tool calls and proposals between them as one line each, oldest first and
/// at most [`SUMMARY_BYTES`]. When it doesn't fit, the oldest lines go first;
/// a single line over the cap is cut.
pub fn summary(items: &[Item]) -> String {
    let mut lines: Vec<String> = Vec::new();
    let mut messages = 0;
    for item in items.iter().rev() {
        let line = match item {
            Item::User { text, .. } => {
                messages += 1;
                format!("You: {}", text.trim())
            }
            Item::Agent { text, .. } => {
                messages += 1;
                format!("Agent: {}", text.trim())
            }
            Item::Tool { title, status, .. } => format!(
                "Tool: {title}{}",
                if *status == ToolStatus::Failed {
                    " (failed)"
                } else {
                    ""
                }
            ),
            Item::Proposed { counts, state, .. } => format!(
                "Proposed {} change{} ({})",
                counts.total(),
                if counts.total() == 1 { "" } else { "s" },
                match state {
                    RoundState::ToReview => "not reviewed",
                    RoundState::Applied => "applied",
                    RoundState::Undone => "applied, then undone",
                    RoundState::Discarded => "discarded",
                    RoundState::Lost => "lost",
                },
            ),
            Item::Thought { .. } | Item::Permission { .. } | Item::Notice { .. } => continue,
        };
        if messages > SUMMARY_MESSAGES {
            break;
        }
        lines.push(line);
    }
    lines.reverse();
    let mut kept: Vec<String> = Vec::new();
    let mut size = 0;
    for line in lines.into_iter().rev() {
        let line = cut(&line, SUMMARY_BYTES - 1);
        let cost = line.len() + 1;
        if size + cost > SUMMARY_BYTES {
            break;
        }
        size += cost;
        kept.push(line);
    }
    kept.reverse();
    kept.join("\n")
}
