//! An agent on this computer as the source of AI answers: what the settings
//! mean for `kanso-ai`, the notes access each of its sessions gets (read
//! only, and suggested edits that wait for review when the person allowed
//! them), and the plain lines Settings and the Ask panel show about what the
//! agent sees and who it sends questions to.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use gpui_kit::App;
use kanso_ai::AgentKind;
use kanso_ai::agent::{AgentSettings, NotesAccess, NotesGrant, NotesLease};
use kanso_server::{AgentScope, TokenId, TokenStore};

use crate::config::AgentChoice;

/// One session's notes token. Dropping it revokes the token.
struct Lease {
    tokens: TokenStore,
    id: TokenId,
}

impl NotesLease for Lease {
    fn is_live(&self) -> bool {
        self.tokens.is_live(self.id)
    }

    fn renew(&self) -> bool {
        self.tokens.renew(self.id)
    }

    fn begin_turn(&self) {
        self.tokens.begin_turn(self.id);
    }

    fn reads(&self) -> Vec<String> {
        self.tokens.reads(self.id)
    }

    fn seed(&self, sent: &[kanso_ai::SentExcerpt]) {
        self.tokens.seed_shown(
            self.id,
            sent.iter()
                .map(|excerpt| kanso_server::ShownNote {
                    number: excerpt.number,
                    id: excerpt.id.clone(),
                    title: excerpt.title.clone(),
                    updated: excerpt.updated.clone(),
                    lines: excerpt.lines.clone(),
                })
                .collect(),
        );
    }

    fn shown(&self) -> Vec<kanso_ai::SentExcerpt> {
        self.tokens
            .shown_notes(self.id)
            .into_iter()
            .map(|note| kanso_ai::SentExcerpt {
                number: note.number,
                id: note.id,
                title: note.title,
                updated: note.updated,
                lines: note.lines,
            })
            .collect()
    }
}

impl Drop for Lease {
    fn drop(&mut self) {
        self.tokens.revoke(self.id);
    }
}

/// The notebooks `choice` lets the agent read: `None` for every unlocked
/// one.
pub fn scope_notebooks(choice: &AgentChoice) -> Option<HashSet<String>> {
    (!choice.notebooks.is_empty()).then(|| choice.notebooks.iter().cloned().collect())
}

/// The scope each session of `choice` gets over `notebooks`: Ask's two read
/// tools, and the suggesting tools too when the person allowed suggested
/// edits. Those only queue a change for review; the agent never writes.
pub fn scope_for(
    choice: &AgentChoice,
    notebooks: Option<HashSet<String>>,
    agent: &str,
) -> AgentScope {
    if choice.suggest_edits {
        AgentScope::ask_and_suggest(notebooks, agent)
    } else {
        AgentScope::ask(notebooks)
    }
}

/// How each new session of `choice` gets access to the notes at `url`: a
/// token of its own over the notebooks `choice` allows, with Ask's five
/// tools (and the suggesting ones when allowed), and `relay` for an agent
/// without HTTP MCP.
pub fn notes_access_at(
    choice: &AgentChoice,
    url: String,
    tokens: TokenStore,
    relay: Option<PathBuf>,
) -> NotesAccess {
    let notebooks = scope_notebooks(choice);
    let choice = choice.clone();
    let agent = label(&choice);
    Arc::new(move || {
        let minted = tokens.mint(scope_for(&choice, notebooks.clone(), &agent));
        Some(NotesGrant {
            url: url.clone(),
            token: minted.token,
            relay: relay.clone(),
            suggest: choice.suggest_edits,
            lease: Box::new(Lease {
                tokens: tokens.clone(),
                id: minted.id,
            }),
        })
    })
}

/// The notes access the Ask panel's agent gets, or `None` when note access
/// is off in Settings or nothing can listen for it (then the agent has only
/// the excerpts). Starts the agents' private listener if the local API
/// isn't reachable from this computer alone.
pub fn notes_access(cx: &mut App) -> Option<NotesAccess> {
    let choice = cx
        .global::<crate::workspace::AppState>()
        .config
        .ai
        .agent
        .clone();
    if !choice.mcp {
        return None;
    }
    let url = crate::services::agent_mcp_endpoint(cx)?;
    let tokens = cx
        .global::<crate::services::Services>()
        .agent_tokens
        .clone();
    Some(notes_access_at(
        &choice,
        url,
        tokens,
        std::env::current_exe().ok(),
    ))
}

/// Stops every agent and ends its note access: AI was turned off, the
/// agent or backend changed, or Kanso is quitting.
pub fn stop_agents(cx: &mut App) {
    kanso_ai::agent::shutdown();
    if let Some(services) = cx.try_global::<crate::services::Services>() {
        services.tasks.shutdown();
    }
    crate::services::end_agent_access(cx);
}

/// The empty private folder agents start in, inside Kanso's data folder.
pub fn workspace(data_dir: &Path) -> PathBuf {
    data_dir.join("agents").join("workspace")
}

/// The agent `choice` names, as `kanso-ai` runs it.
pub fn agent_settings(choice: &AgentChoice, data_dir: &Path) -> AgentSettings {
    AgentSettings {
        kind: choice.kind,
        command: choice.command.clone(),
        args: choice.args.clone(),
        env: choice
            .env
            .iter()
            .map(|(name, value)| (name.clone(), value.clone()))
            .collect(),
        workspace: workspace(data_dir),
    }
}

/// What the person calls the chosen agent: a registry agent by its name
/// there, once the registry has been read.
pub fn label(choice: &AgentChoice) -> String {
    let registry = crate::agent_registry::cached().unwrap_or_default();
    match crate::agent_registry::chosen(choice, &registry) {
        Some(agent) => agent.name.clone(),
        None => agent_settings(choice, Path::new("")).label(),
    }
}

/// Where a question goes when an agent answers. Settings and the Ask panel
/// show the same line; it never says notes stay on this computer.
pub fn requests_line(choice: &AgentChoice) -> String {
    let label = label(choice);
    let vendor = choice.kind.vendor();
    let what = if choice.mcp {
        "Questions, the note excerpts they use (with their titles, dates and notebook names), and the notes it searches or reads"
    } else {
        "Questions and the note excerpts they use, with their titles, dates and notebook names,"
    };
    let suggest = if choice.mcp && choice.suggest_edits {
        format!(" {label} may also suggest edits to notes; nothing changes until you apply one.")
    } else {
        String::new()
    };
    format!(
        "{what} go to {label} on this computer, which sends them to {vendor} under your own account. Kanso doesn't proxy or keep them. Locked notes, notes in the trash and notes tagged #ai-exclude are never sent. {label} may keep its own history of the conversation.{suggest}"
    )
}

/// The trust line: what Kanso refuses, and what it can't control.
pub fn trust_line(choice: &AgentChoice) -> String {
    let label = label(choice);
    format!(
        "{label} runs as a program on this computer, as you. Kanso gives it no access to files or a terminal and refuses its requests for them. It can still read files and run commands with its own tools: Kanso asks before it uses one (Allow once or Reject), but can't see or limit what the tool then does. Notes are data, not instructions: Kanso asks the agent to ignore instructions inside them, but a note can still steer an answer. Check the citations."
    )
}

/// A one-line status for a known agent from its detection. How to install
/// it is the Install row's.
pub fn status_line(kind: AgentKind, detection: &kanso_acp::Detection) -> String {
    use kanso_acp::Detection;
    match detection {
        Detection::Found(path) => format!(
            "Found at {}. Test checks that it's signed in.",
            path.display()
        ),
        Detection::NotInstalled => {
            let node = kind
                .node()
                .map(|major| format!(" It needs Node {major} or newer."))
                .unwrap_or_default();
            format!("Not installed on this computer.{node}")
        }
        Detection::OldClaudeAdapter(_) => format!(
            "Only the old Claude Code adapter is installed, which Kanso doesn't start. Install the new one with: {}",
            kind.install().unwrap_or_default()
        ),
    }
}

/// A short status for a menu: Found, or Not installed.
pub fn status_word(detection: &kanso_acp::Detection) -> &'static str {
    match detection {
        kanso_acp::Detection::Found(_) => "Found",
        kanso_acp::Detection::NotInstalled => "Not installed",
        kanso_acp::Detection::OldClaudeAdapter(_) => "Update needed",
    }
}

#[cfg(test)]
mod tests {
    use super::{agent_settings, label, requests_line, status_line, trust_line, workspace};
    use crate::config::AgentChoice;
    use kanso_acp::Detection;
    use kanso_ai::AgentKind;
    use proptest::prelude::*;
    use std::path::Path;

    #[test]
    fn the_workspace_is_inside_the_data_folder_and_not_the_notes() {
        let dir = Path::new("/data/Kanso");
        assert_eq!(workspace(dir), dir.join("agents").join("workspace"));
        let settings = agent_settings(&AgentChoice::default(), dir);
        assert_eq!(settings.workspace, workspace(dir));
        assert_eq!(settings.kind, AgentKind::ClaudeAgent);
    }

    #[test]
    fn each_session_gets_its_own_token_which_ends_with_it() {
        use super::{notes_access_at, scope_notebooks};
        let tokens = kanso_server::TokenStore::new();
        let mut choice = AgentChoice::default();
        assert_eq!(scope_notebooks(&choice), None);
        choice.notebooks = vec!["book:a".into()];
        assert_eq!(
            scope_notebooks(&choice),
            Some(["book:a".to_owned()].into_iter().collect())
        );
        let access = notes_access_at(
            &choice,
            "http://127.0.0.1:1/mcp".into(),
            tokens.clone(),
            None,
        );
        let first = access().unwrap();
        let second = access().unwrap();
        assert_ne!(first.token, second.token);
        assert_eq!(tokens.live_count(), 2);
        let Some(kanso_server::Caller::Agent { scope, .. }) = tokens.verify(&first.token) else {
            panic!("the token is an agent's");
        };
        assert!(scope.read_only && !scope.allow_attachments);
        assert_eq!(scope.notebooks, scope_notebooks(&choice));
        assert!(scope.allows_tool("read-note") && !scope.allows_tool("create-note"));
        assert!(scope.suggests_as.is_none());
        assert!(first.lease.is_live());
        drop(first);
        assert_eq!(tokens.live_count(), 1);
        assert!(tokens.verify(&second.token).is_some());
    }

    #[test]
    fn suggested_edits_are_off_unless_allowed_and_never_write() {
        use super::{notes_access_at, scope_for};
        let mut choice = AgentChoice::default();
        assert!(!choice.suggest_edits, "off by default");
        let off = scope_for(&choice, None, "Claude Agent");
        assert!(!off.allows_tool("patch-note") && off.suggests_as.is_none());
        choice.suggest_edits = true;
        let on = scope_for(&choice, None, "Claude Agent");
        for tool in ["patch-note", "update-note", "create-note", "read-note"] {
            assert!(on.allows_tool(tool), "{tool}");
        }
        // Tags and notebooks stay denied; nothing in it writes the store.
        for tool in [
            "update-tag",
            "update-book",
            "create-tag",
            "create-book",
            "create-file",
        ] {
            assert!(!on.allows_tool(tool), "{tool}");
        }
        assert!(on.read_only && !on.allow_attachments);
        assert_eq!(on.suggests_as.as_deref(), Some("Claude Agent"));
        // A session's token carries the scope the setting meant.
        let tokens = kanso_server::TokenStore::new();
        let access = notes_access_at(
            &choice,
            "http://127.0.0.1:1/mcp".into(),
            tokens.clone(),
            None,
        );
        let grant = access().unwrap();
        assert!(grant.suggest, "the agent is told it may suggest");
        let Some(kanso_server::Caller::Agent { scope, .. }) = tokens.verify(&grant.token) else {
            panic!("the token is an agent's");
        };
        assert!(scope.allows_tool("patch-note"));
        assert!(scope.suggests_as.is_some());
    }

    #[test]
    fn a_missing_agent_says_what_it_needs() {
        let line = status_line(AgentKind::ClaudeAgent, &Detection::NotInstalled);
        assert!(line.starts_with("Not installed"), "{line}");
        assert!(line.contains("Node 22"), "{line}");
        let line = status_line(
            AgentKind::ClaudeAgent,
            &Detection::OldClaudeAdapter("/usr/local/bin/claude-code-acp".into()),
        );
        assert!(line.contains("old Claude Code adapter"), "{line}");
    }

    fn gen_choice() -> impl Strategy<Value = AgentChoice> {
        (
            prop::sample::select(AgentKind::ALL.to_vec()),
            prop::option::of("[a-z/-]{1,20}"),
        )
            .prop_map(|(kind, command)| AgentChoice {
                kind,
                command,
                ..AgentChoice::default()
            })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(128))]

        /// Whatever the agent, the lines name it and who it sends to, say
        /// locked and excluded notes are never sent, never promise the notes
        /// stay on this computer, and say Kanso doesn't sandbox it.
        #[test]
        fn prop_the_agent_lines_never_overpromise(choice in gen_choice()) {
            let line = requests_line(&choice);
            let label = label(&choice);
            prop_assert!(line.contains(&label));
            prop_assert!(line.contains(choice.kind.vendor()));
            prop_assert!(line.contains("#ai-exclude"));
            prop_assert!(line.contains("Locked notes"));
            prop_assert!(!line.to_lowercase().contains("never leave"));
            prop_assert!(!line.contains('\n'));
            let trust = trust_line(&choice);
            prop_assert!(trust.contains("its own tools"));
            prop_assert!(trust.contains("as you"));
            prop_assert!(trust.contains("Allow once or Reject"));
            prop_assert!(!trust.to_lowercase().contains("never leave"));
        }

        /// Suggested edits are mentioned only when they are on, and say that
        /// nothing changes until the person applies them.
        #[test]
        fn prop_the_requests_line_mentions_suggested_edits_only_when_on(
            choice in gen_choice(),
            mcp in any::<bool>(),
            suggest in any::<bool>(),
        ) {
            let choice = AgentChoice { mcp, suggest_edits: suggest, ..choice };
            let line = requests_line(&choice);
            prop_assert_eq!(line.contains("suggest edits"), mcp && suggest);
            if mcp && suggest {
                prop_assert!(line.contains("nothing changes until you apply one"));
            }
            prop_assert!(line.contains("notes in the trash"));
        }
    }
}
