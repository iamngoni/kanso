//! What `kanso api-key` and a running Kanso say to each other over the
//! control socket (issue #243): one JSON line each way. No key or hash is
//! ever part of a request; a reply carries a key only for `create`, and only
//! down the socket the caller opened.

use kanso_server::KeyAccess;
use serde::{Deserialize, Serialize};

use crate::api_key_cli::Command;

/// The longest line either side reads. A request is a name and a few
/// notebook names.
pub const MAX_LINE: usize = 64 * 1024;

/// A command for the running process, as it goes down the socket.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "verb", rename_all = "lowercase", deny_unknown_fields)]
pub enum Request {
    Create {
        name: String,
        access: String,
        #[serde(default)]
        scopes: Vec<String>,
    },
    List,
    Revoke {
        target: String,
    },
}

/// What the running process answers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Reply {
    /// Done: what the command prints on standard output and on standard
    /// error.
    Done { out: String, err: String },
    /// Refused or failed, in a sentence that says what to do.
    Failed { error: String },
}

impl Request {
    /// The request for `command`; `None` for help, which needs no process.
    pub fn of(command: &Command) -> Option<Request> {
        Some(match command {
            Command::Help => return None,
            Command::List => Request::List,
            Command::Revoke { target } => Request::Revoke {
                target: target.clone(),
            },
            Command::Create {
                name,
                access,
                scopes,
            } => Request::Create {
                name: name.clone(),
                access: access.as_str().to_owned(),
                scopes: scopes.clone(),
            },
        })
    }

    /// The command this asks for, or why it can't be one.
    pub fn command(self) -> Result<Command, String> {
        Ok(match self {
            Request::List => Command::List,
            Request::Revoke { target } => Command::Revoke { target },
            Request::Create {
                name,
                access,
                scopes,
            } => Command::Create {
                name,
                access: KeyAccess::parse(access.trim()).ok_or_else(|| {
                    format!("--access is read, suggest or write, not “{access}”.")
                })?,
                scopes,
            },
        })
    }
}

#[cfg(test)]
#[path = "control_model_tests.rs"]
mod tests;
