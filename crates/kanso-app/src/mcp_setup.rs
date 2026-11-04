//! How to add Kanso's MCP server to each AI agent, as text to paste: a
//! command for agents that have one, otherwise the entry for their config
//! file. Agents that reach a local HTTP server get the endpoint and the
//! local API's sign-in; the rest start `kanso mcp` (see `mcp_cli`), which
//! passes their requests to the running app and needs no sign-in of its own.

use serde_json::{Value, json};

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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Setup {
    /// The MCP endpoint, as `mcp_endpoint` gives it.
    pub endpoint: String,
    /// The `Authorization` header's value for the local API's sign-in.
    pub authorization: String,
    /// This Kanso program, for agents that start `kanso mcp` themselves.
    pub program: String,
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

    /// Where the snippet goes, said to the user.
    pub fn instructions(self) -> &'static str {
        match self {
            Agent::ClaudeCode => "Run this in a terminal.",
            Agent::ClaudeDesktop => {
                "Add this to claude_desktop_config.json (Settings › Developer › Edit Config), then restart Claude."
            }
            Agent::Codex => "Run this in a terminal.",
            Agent::Cursor => {
                "Add this to ~/.cursor/mcp.json (Settings › MCP › Add new MCP server)."
            }
            Agent::VsCode => {
                "Add this to your mcp.json (command palette › MCP: Open User Configuration)."
            }
            Agent::GeminiCli => "Run this in a terminal.",
            Agent::Windsurf => {
                "Add this to mcp_config.json (Settings › Cascade › MCP servers › View raw config)."
            }
            Agent::Zed => "Add this to Zed's settings.json.",
            Agent::OpenCode => "Add this to ~/.config/opencode/opencode.json.",
        }
    }

    /// Whether the snippet carries the local API's sign-in, and so should be
    /// copied as a secret.
    pub fn has_sign_in(self) -> bool {
        !matches!(self, Agent::ClaudeDesktop | Agent::Codex)
    }

    /// The text to paste.
    pub fn snippet(self, setup: &Setup) -> String {
        let Setup {
            endpoint,
            authorization,
            program,
        } = setup;
        let header = json!({ "Authorization": authorization });
        let config =
            |value: Value| serde_json::to_string_pretty(&value).expect("JSON values print");
        match self {
            Agent::ClaudeCode => format!(
                "claude mcp add --transport http --scope user kanso {endpoint} --header \"Authorization: {authorization}\""
            ),
            Agent::GeminiCli => format!(
                "gemini mcp add --transport http --scope user --header \"Authorization: {authorization}\" kanso {endpoint}"
            ),
            Agent::Codex => format!("codex mcp add kanso -- {} mcp", quoted(program)),
            Agent::ClaudeDesktop => config(json!({
                "mcpServers": { "kanso": { "command": program, "args": ["mcp"] } }
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
            authorization: "Bearer kcl_example".to_owned(),
            program: "/Applications/Kanso.app/Contents/MacOS/kanso".into(),
        }
    }

    #[test]
    fn snippet_examples() {
        let setup = setup();
        assert_eq!(
            Agent::ClaudeCode.snippet(&setup),
            "claude mcp add --transport http --scope user kanso http://127.0.0.1:19840/mcp --header \"Authorization: Bearer kcl_example\""
        );
        assert_eq!(
            Agent::Codex.snippet(&setup),
            "codex mcp add kanso -- /Applications/Kanso.app/Contents/MacOS/kanso mcp"
        );
        let windows = Setup {
            program: r"C:\Program Files\Kanso\kanso.exe".into(),
            ..setup.clone()
        };
        assert_eq!(
            Agent::Codex.snippet(&windows),
            r#"codex mcp add kanso -- "C:\Program Files\Kanso\kanso.exe" mcp"#
        );
        let desktop: Value = serde_json::from_str(&Agent::ClaudeDesktop.snippet(&windows)).unwrap();
        assert_eq!(
            desktop["mcpServers"]["kanso"],
            json!({"command": r"C:\Program Files\Kanso\kanso.exe", "args": ["mcp"]})
        );
        let vscode: Value = serde_json::from_str(&Agent::VsCode.snippet(&setup)).unwrap();
        assert_eq!(vscode["servers"]["kanso"]["type"], "http");
    }

    /// Where an agent reaches the server over HTTP, its snippet names the
    /// endpoint and carries the sign-in; otherwise it starts this program
    /// and carries no sign-in. Config snippets are JSON that reads back.
    fn check(agent: Agent, setup: &Setup) -> Result<(), TestCaseError> {
        let text = agent.snippet(setup);
        prop_assert_eq!(text.contains(&setup.authorization), agent.has_sign_in());
        if agent.has_sign_in() {
            prop_assert!(text.contains(&setup.endpoint));
        }
        let is_command = matches!(agent, Agent::ClaudeCode | Agent::GeminiCli | Agent::Codex);
        if !is_command {
            let value: Value =
                serde_json::from_str(&text).map_err(|e| TestCaseError::fail(e.to_string()))?;
            let found = value.to_string();
            if agent.has_sign_in() {
                prop_assert!(found.contains(&serde_json::to_string(&setup.endpoint).unwrap()));
            } else {
                prop_assert!(found.contains(&serde_json::to_string(&setup.program).unwrap()));
            }
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
                authorization: format!("Bearer {key}"),
                program,
            };
            for agent in Agent::ALL {
                check(agent, &setup)?;
            }
        }
    }
}
