//! How to add Kanso's MCP server to each AI agent, as text to paste: a
//! command for agents that have one, otherwise the entry for their config
//! file. Agents that reach a local HTTP server get the endpoint and the
//! API key as a header; the rest start `kanso mcp` (see `mcp_cli`), which
//! takes the key from `KANSO_MCP_TOKEN` and passes their requests to the
//! running app. Every snippet carries the key, so every one is a secret.

use serde_json::{Value, json};
use zeroize::Zeroizing;

use crate::mcp_cli::TOKEN_VAR;

/// An agent Settings has instructions for, in the order they're listed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Agent {
    ClaudeCode,
    ClaudeDesktop,
    Codex,
    Cursor,
    VsCode,
    GeminiCli,
    Windsurf,
    Zed,
    OpenCode,
}

/// What every snippet is made from.
#[derive(Clone, PartialEq, Eq)]
pub struct Setup {
    /// The MCP endpoint, as `mcp_endpoint` gives it.
    pub endpoint: String,
    /// The API key the snippet carries, wiped when the setup is dropped.
    pub key: Zeroizing<String>,
    /// This Kanso program, for agents that start `kanso mcp` themselves.
    pub program: String,
}

// The key is a secret: it stays out of any debug print.
impl std::fmt::Debug for Setup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Setup")
            .field("endpoint", &self.endpoint)
            .field("key", &"<redacted>")
            .field("program", &self.program)
            .finish()
    }
}

impl Setup {
    /// The `Authorization` header's value.
    pub fn authorization(&self) -> String {
        format!("Bearer {}", self.key.as_str())
    }

    /// What a script or another app needs: the endpoint and the two ways to
    /// send the key.
    pub fn rest_snippet(&self) -> String {
        let base = self.endpoint.strip_suffix("/mcp").unwrap_or(&self.endpoint);
        format!(
            "Endpoint: {base}\nMCP: {mcp}\n\nAuthorization: {auth}\n\nTools that only accept a username and password: use any username, with the key as the password.",
            mcp = self.endpoint,
            auth = self.authorization(),
        )
    }
}

impl Agent {
    pub const ALL: [Agent; 9] = [
        Agent::ClaudeCode,
        Agent::ClaudeDesktop,
        Agent::Codex,
        Agent::Cursor,
        Agent::VsCode,
        Agent::GeminiCli,
        Agent::Windsurf,
        Agent::Zed,
        Agent::OpenCode,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Agent::ClaudeCode => "Claude Code",
            Agent::ClaudeDesktop => "Claude Desktop",
            Agent::Codex => "Codex",
            Agent::Cursor => "Cursor",
            Agent::VsCode => "VS Code",
            Agent::GeminiCli => "Gemini CLI",
            Agent::Windsurf => "Windsurf",
            Agent::Zed => "Zed",
            Agent::OpenCode => "OpenCode",
        }
    }

    /// The `client` a key made for this agent is saved with.
    pub fn id(self) -> &'static str {
        match self {
            Agent::ClaudeCode => "claude-code",
            Agent::ClaudeDesktop => "claude-desktop",
            Agent::Codex => "codex",
            Agent::Cursor => "cursor",
            Agent::VsCode => "vscode",
            Agent::GeminiCli => "gemini-cli",
            Agent::Windsurf => "windsurf",
            Agent::Zed => "zed",
            Agent::OpenCode => "opencode",
        }
    }

    /// The agent a saved `client` names, if it names one.
    pub fn from_id(id: &str) -> Option<Agent> {
        Agent::ALL.into_iter().find(|agent| agent.id() == id)
    }

    /// Where the snippet goes, said to the user.
    pub fn instructions(self) -> &'static str {
        match self {
            Agent::ClaudeCode => TERMINAL,
            Agent::ClaudeDesktop => {
                "Add this to claude_desktop_config.json (Settings › Developer › Edit Config), then restart Claude."
            }
            Agent::Codex => TERMINAL,
            Agent::Cursor => {
                "Add this to ~/.cursor/mcp.json (Settings › MCP › Add new MCP server)."
            }
            Agent::VsCode => {
                "Add this to your mcp.json (command palette › MCP: Open User Configuration)."
            }
            Agent::GeminiCli => TERMINAL,
            Agent::Windsurf => {
                "Add this to mcp_config.json (Settings › Cascade › MCP servers › View raw config)."
            }
            Agent::Zed => "Add this to Zed's settings.json.",
            Agent::OpenCode => "Add this to ~/.config/opencode/opencode.json.",
        }
    }

    /// The text to paste.
    pub fn snippet(self, setup: &Setup) -> String {
        let Setup {
            endpoint,
            key,
            program,
        } = setup;
        let key = key.as_str();
        let authorization = setup.authorization();
        let header = json!({ "Authorization": authorization });
        let config =
            |value: Value| serde_json::to_string_pretty(&value).expect("JSON values print");
        match self {
            Agent::ClaudeCode => format!(
                "{HISTORY_SKIP}claude mcp add --transport http --scope user kanso {endpoint} --header \"Authorization: {authorization}\""
            ),
            Agent::GeminiCli => format!(
                "{HISTORY_SKIP}gemini mcp add --transport http --scope user --header \"Authorization: {authorization}\" kanso {endpoint}"
            ),
            Agent::Codex => format!(
                "{HISTORY_SKIP}codex mcp add kanso --env {TOKEN_VAR}={key} -- {} mcp",
                quoted(program)
            ),
            Agent::ClaudeDesktop => config(json!({
                "mcpServers": { "kanso": {
                    "command": program,
                    "args": ["mcp"],
                    "env": { TOKEN_VAR: key },
                } }
            })),
            Agent::Cursor => config(json!({
                "mcpServers": { "kanso": { "url": endpoint, "headers": header } }
            })),
            Agent::VsCode => config(json!({
                "servers": { "kanso": { "type": "http", "url": endpoint, "headers": header } }
            })),
            Agent::Windsurf => config(json!({
                "mcpServers": { "kanso": { "serverUrl": endpoint, "headers": header } }
            })),
            Agent::Zed => config(json!({
                "context_servers": { "kanso": { "url": endpoint, "headers": header } }
            })),
            Agent::OpenCode => config(json!({
                "$schema": "https://opencode.ai/config.json",
                "mcp": {
                    "kanso": { "type": "remote", "url": endpoint, "enabled": true, "headers": header }
                }
            })),
        }
    }
}

/// Where a terminal command goes, said to the user.
const TERMINAL: &str = "Run this in a terminal. It starts with a space, so a shell set to skip such lines (fish, or bash and zsh with ignorespace) keeps the key out of its history.";

/// What a terminal command starts with: a space. bash with `ignorespace`,
/// zsh with `HIST_IGNORE_SPACE`, and fish leave such a line out of their
/// history, so the key in it isn't saved there. The design's commands put
/// the key in the command's arguments (the agents' CLIs take it no other
/// way); this keeps it out of the history file where the shell allows.
const HISTORY_SKIP: &str = " ";

/// A program path as one word for the shell, in double quotes when it has a
/// space (as `C:\Program Files\…` does), which sh, PowerShell and cmd read
/// alike.
fn quoted(path: &str) -> String {
    if path.contains(' ') {
        format!("\"{path}\"")
    } else {
        path.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 256,
            ..ProptestConfig::default()
        }
    }

    fn setup() -> Setup {
        Setup {
            endpoint: "http://127.0.0.1:19840/mcp".into(),
            key: Zeroizing::new("kcl_example".to_owned()),
            program: "/Applications/Kanso.app/Contents/MacOS/kanso".into(),
        }
    }

    #[test]
    fn snippet_examples() {
        let setup = setup();
        assert_eq!(
            Agent::ClaudeCode.snippet(&setup),
            " claude mcp add --transport http --scope user kanso http://127.0.0.1:19840/mcp --header \"Authorization: Bearer kcl_example\""
        );
        assert_eq!(
            Agent::Codex.snippet(&setup),
            " codex mcp add kanso --env KANSO_MCP_TOKEN=kcl_example -- /Applications/Kanso.app/Contents/MacOS/kanso mcp"
        );
        let windows = Setup {
            program: r"C:\Program Files\Kanso\kanso.exe".into(),
            ..setup.clone()
        };
        assert_eq!(
            Agent::Codex.snippet(&windows),
            r#" codex mcp add kanso --env KANSO_MCP_TOKEN=kcl_example -- "C:\Program Files\Kanso\kanso.exe" mcp"#
        );
        let desktop: Value = serde_json::from_str(&Agent::ClaudeDesktop.snippet(&windows)).unwrap();
        assert_eq!(
            desktop["mcpServers"]["kanso"],
            json!({
                "command": r"C:\Program Files\Kanso\kanso.exe",
                "args": ["mcp"],
                "env": { "KANSO_MCP_TOKEN": "kcl_example" },
            })
        );
        let vscode: Value = serde_json::from_str(&Agent::VsCode.snippet(&setup)).unwrap();
        assert_eq!(vscode["servers"]["kanso"]["type"], "http");
    }

    #[test]
    fn the_rest_snippet_gives_the_endpoint_and_both_ways_to_send_the_key() {
        let text = setup().rest_snippet();
        assert!(text.contains("Endpoint: http://127.0.0.1:19840\n"));
        assert!(text.contains("Authorization: Bearer kcl_example"));
        assert!(text.contains("any username, with the key as the password"));
    }

    #[test]
    fn a_setup_never_prints_its_key() {
        assert!(!format!("{:?}", setup()).contains("kcl_example"));
    }

    #[test]
    fn every_agent_has_its_own_id_and_is_found_by_it() {
        for agent in Agent::ALL {
            assert_eq!(Agent::from_id(agent.id()), Some(agent));
        }
        assert_eq!(Agent::from_id("other"), None);
    }

    /// Every snippet carries the key. Where an agent reaches the server over
    /// HTTP it names the endpoint; otherwise it starts this program. Config
    /// snippets are JSON that reads back.
    fn check(agent: Agent, setup: &Setup) -> Result<(), TestCaseError> {
        let text = agent.snippet(setup);
        prop_assert!(text.contains(setup.key.as_str()));
        let starts_program = matches!(agent, Agent::ClaudeDesktop | Agent::Codex);
        let is_command = matches!(agent, Agent::ClaudeCode | Agent::GeminiCli | Agent::Codex);
        // A command starts with a space, so shells that skip such lines
        // keep the key out of their history.
        prop_assert_eq!(text.starts_with(HISTORY_SKIP), is_command);
        prop_assert!(agent.instructions().contains("space") == is_command);
        if !is_command {
            let value: Value =
                serde_json::from_str(&text).map_err(|e| TestCaseError::fail(e.to_string()))?;
            let found = value.to_string();
            let needle = if starts_program {
                &setup.program
            } else {
                &setup.endpoint
            };
            prop_assert!(found.contains(&serde_json::to_string(needle).unwrap()));
        } else if !starts_program {
            prop_assert!(text.contains(&setup.endpoint));
        }
        Ok(())
    }

    proptest! {
        #![proptest_config(config())]

        #[test]
        fn prop_snippets_carry_what_they_need(
            port in 1u16..,
            key in "kcl_[A-Za-z0-9_-]{43}",
            program in r"(/[a-zA-Z0-9 ._-]{1,12}){1,4}|[A-Z]:(\\[a-zA-Z0-9 ._-]{1,12}){1,4}",
        ) {
            let setup = Setup {
                endpoint: format!("http://127.0.0.1:{port}/mcp"),
                key: Zeroizing::new(key),
                program,
            };
            for agent in Agent::ALL {
                check(agent, &setup)?;
            }
        }
    }
}
