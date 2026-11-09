//! How to add Kanso's MCP server to each AI agent, as the entry to paste
//! into the agent's config file. Never a command: a key in a command's
//! arguments is in the process list and the shell's history. Agents that
//! reach a local HTTP server get the endpoint and the API key as a header;
//! the rest start `kanso mcp` (see `mcp_cli`), which takes the key from
//! `KANSO_MCP_TOKEN` in the entry's environment and passes their requests
//! to the running app. Every snippet carries the key, so every one is a
//! secret.

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

    /// Where the snippet goes, said to the user. Every agent's key goes in
    /// a config file the agent reads, never in a command's arguments, where
    /// the process list and the shell's history would keep it.
    pub fn instructions(self) -> String {
        match self {
            Agent::ClaudeCode => format!(
                "Quit Claude Code, add the \"kanso\" entry to the mcpServers object in {CLAUDE_CODE_CONFIG} (your user settings, not a project's .mcp.json, which may be shared), then start it again. Claude Code rewrites that file while it runs, so an edit made meanwhile can be lost."
            ),
            Agent::ClaudeDesktop => format!(
                "Add this to {CLAUDE_DESKTOP_CONFIG} (Settings › Developer › Edit Config), then restart Claude."
            ),
            Agent::Codex => format!(
                "Add this to {CODEX_CONFIG}, then restart Codex. It starts Kanso with the key in its environment, so the key is not in a command."
            ),
            Agent::Cursor => {
                format!("Add this to {CURSOR_CONFIG} (Settings › MCP › Add new MCP server).")
            }
            Agent::VsCode => format!(
                "Add this to your {VS_CODE_CONFIG} (command palette › MCP: Open User Configuration)."
            ),
            Agent::GeminiCli => format!(
                "Add the \"kanso\" entry to the mcpServers object in {GEMINI_CLI_CONFIG}, then restart Gemini CLI."
            ),
            Agent::Windsurf => format!(
                "Add this to {WINDSURF_CONFIG} (Settings › Cascade › MCP servers › View raw config)."
            ),
            Agent::Zed => format!("Add this to Zed's {ZED_CONFIG}."),
            Agent::OpenCode => format!("Add this to {OPENCODE_CONFIG}."),
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
            Agent::ClaudeCode => config(json!({
                "mcpServers": { "kanso": { "type": "http", "url": endpoint, "headers": header } }
            })),
            Agent::GeminiCli => config(json!({
                "mcpServers": { "kanso": { "httpUrl": endpoint, "headers": header } }
            })),
            // TOML basic strings take the same escapes as JSON strings.
            Agent::Codex => format!(
                "[mcp_servers.kanso]\ncommand = {}\nargs = [\"mcp\"]\n\n[mcp_servers.kanso.env]\n{TOKEN_VAR} = {}",
                toml_string(program),
                toml_string(key)
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

// The agents' config files. Kanso only shows the entry for the user to
// paste; it never writes another program's config. Each agent's own docs
// say how it takes an MCP server and its key in a file: Claude Code's
// `mcpServers` entry with `type` and `headers`; Gemini CLI's `mcpServers`
// entry with `httpUrl` and `headers`; Codex's `[mcp_servers.<name>]` table
// with `command`, `args` and an `env` table.
/// Claude Code's user settings, where its user-scope servers are kept.
const CLAUDE_CODE_CONFIG: &str = "~/.claude.json";
/// Claude Desktop's config file.
const CLAUDE_DESKTOP_CONFIG: &str = "claude_desktop_config.json";
/// Codex's settings file.
const CODEX_CONFIG: &str = "~/.codex/config.toml";
/// Cursor's global MCP file.
const CURSOR_CONFIG: &str = "~/.cursor/mcp.json";
/// VS Code's user MCP file.
const VS_CODE_CONFIG: &str = "mcp.json";
/// Gemini CLI's user settings file.
const GEMINI_CLI_CONFIG: &str = "~/.gemini/settings.json";
/// Windsurf's MCP file.
const WINDSURF_CONFIG: &str = "mcp_config.json";
/// Zed's settings file.
const ZED_CONFIG: &str = "settings.json";
/// OpenCode's global config file.
const OPENCODE_CONFIG: &str = "~/.config/opencode/opencode.json";

/// `text` as a TOML string, escaped by TOML's own rules. A JSON string is
/// not always a TOML one: TOML refuses a bare U+007F, which JSON allows.
fn toml_string(text: &str) -> String {
    toml::Value::String(text.to_owned()).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 256,
            rng_seed: proptest::test_runner::RngSeed::Fixed(0x006b_616e_736f),
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
        let claude: Value = serde_json::from_str(&Agent::ClaudeCode.snippet(&setup)).unwrap();
        assert_eq!(
            claude["mcpServers"]["kanso"],
            json!({
                "type": "http",
                "url": "http://127.0.0.1:19840/mcp",
                "headers": { "Authorization": "Bearer kcl_example" },
            })
        );
        let gemini: Value = serde_json::from_str(&Agent::GeminiCli.snippet(&setup)).unwrap();
        assert_eq!(
            gemini["mcpServers"]["kanso"],
            json!({
                "httpUrl": "http://127.0.0.1:19840/mcp",
                "headers": { "Authorization": "Bearer kcl_example" },
            })
        );
        assert_eq!(
            Agent::Codex.snippet(&setup),
            "[mcp_servers.kanso]\ncommand = \"/Applications/Kanso.app/Contents/MacOS/kanso\"\nargs = [\"mcp\"]\n\n[mcp_servers.kanso.env]\nKANSO_MCP_TOKEN = \"kcl_example\""
        );
        let windows = Setup {
            program: r"C:\Program Files\Kanso\kanso.exe".into(),
            ..setup.clone()
        };
        let codex: toml::Table = Agent::Codex.snippet(&windows).parse().unwrap();
        assert_eq!(
            codex["mcp_servers"]["kanso"]["command"].as_str(),
            Some(r"C:\Program Files\Kanso\kanso.exe")
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

    /// What Copy puts on the clipboard is the snippet itself: the same text
    /// the sheet shows line by line, with the real key in full, and valid
    /// JSON (or TOML for Codex) for every client.
    // no-prop-test: one example per client; the key's shape is covered by
    // the properties below.
    #[test]
    fn the_copied_text_is_the_displayed_snippet_with_the_whole_key() {
        let key = "kcl_Ab3-xY_9QwErTy0123456789abcdefghijklmn";
        let setup = Setup {
            key: Zeroizing::new(key.to_owned()),
            ..setup()
        };
        for agent in Agent::ALL {
            let text = agent.snippet(&setup);
            let shown = text.lines().collect::<Vec<_>>().join("\n");
            assert_eq!(text, shown, "{}: the copy is what is shown", agent.name());
            assert_eq!(text.matches(key).count(), 1, "{}", agent.name());
            if agent == Agent::Codex {
                let table: toml::Table = text.parse().unwrap();
                assert_eq!(
                    table["mcp_servers"]["kanso"]["env"][TOKEN_VAR].as_str(),
                    Some(key)
                );
            } else {
                let json: Value = serde_json::from_str(&text)
                    .unwrap_or_else(|err| panic!("{}: {err}", agent.name()));
                assert!(json.is_object(), "{}", agent.name());
            }
        }
        assert!(setup.rest_snippet().contains(&format!("Bearer {key}")));
    }

    #[test]
    fn the_codex_entry_is_toml_whatever_the_path_holds() {
        // serde_json leaves U+007F bare, and TOML refuses it there.
        let odd = Setup {
            program: "/opt/k\u{7f}an\"so\\x\u{1}/kanso".into(),
            ..setup()
        };
        let table: toml::Table = Agent::Codex.snippet(&odd).parse().unwrap();
        assert_eq!(
            table["mcp_servers"]["kanso"]["command"].as_str(),
            Some(odd.program.as_str())
        );
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

    /// Every snippet carries the key, and none is a command: the key is
    /// never in a command's arguments, where the process list and the
    /// shell's history would keep it. Where an agent reaches the server over
    /// HTTP it names the endpoint; otherwise it starts this program. The
    /// snippet is a config file's text that reads back (TOML for Codex,
    /// JSON for the rest).
    fn check(agent: Agent, setup: &Setup) -> Result<(), TestCaseError> {
        let text = agent.snippet(setup);
        prop_assert!(text.contains(setup.key.as_str()));
        prop_assert!(!text.contains(" mcp add "), "{text}");
        for line in text.lines() {
            for cli in ["claude ", "codex ", "gemini "] {
                prop_assert!(!line.trim_start().starts_with(cli), "{line}");
            }
        }
        // The words say where the file is, and that is not a terminal.
        let words = agent.instructions();
        prop_assert!(!words.contains("terminal"), "{words}");
        if agent == Agent::Codex {
            let table: toml::Table = text
                .parse()
                .map_err(|e: toml::de::Error| TestCaseError::fail(e.to_string()))?;
            let server = &table["mcp_servers"]["kanso"];
            prop_assert_eq!(server["command"].as_str(), Some(setup.program.as_str()));
            prop_assert_eq!(server["env"][TOKEN_VAR].as_str(), Some(setup.key.as_str()));
            return Ok(());
        }
        let value: Value =
            serde_json::from_str(&text).map_err(|e| TestCaseError::fail(e.to_string()))?;
        let found = value.to_string();
        let needle = if agent == Agent::ClaudeDesktop {
            &setup.program
        } else {
            &setup.endpoint
        };
        prop_assert!(found.contains(&serde_json::to_string(needle).unwrap()));
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

        /// Whatever the program's path holds, the Codex entry reads back as
        /// TOML with that path and the key.
        #[test]
        fn prop_codex_entry_escapes_any_path(program in any::<String>()) {
            let setup = Setup { program, ..setup() };
            check(Agent::Codex, &setup)?;
        }
    }
}
