//! The ACP registry in Settings › AI: the agents beyond the four Kanso
//! features and tests, read from the registry in the background (Settings
//! never waits for it), and the one-click `npm install -g` an agent gets
//! after the person confirms the exact package. Never `npx -y`.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use kanso_acp::registry::{self, Distribution, RegistryAgent};
use kanso_ai::AgentKind;

use crate::config::AgentChoice;

/// How long a fetched registry is used before it is fetched again.
const FRESH: Duration = Duration::from_secs(60 * 60);
/// The most of the registry read; it is about 60 KB today.
const MAX_BYTES: u64 = 4 * 1024 * 1024;

static CACHE: Mutex<Option<(Instant, Vec<RegistryAgent>)>> = Mutex::new(None);

/// The registry fetched in the last hour, if any.
pub fn cached() -> Option<Vec<RegistryAgent>> {
    let cache = CACHE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    cache
        .as_ref()
        .filter(|(at, _)| at.elapsed() < FRESH)
        .map(|(_, agents)| agents.clone())
}

/// Keeps `agents` as the registry just read, for the snapshots.
#[cfg(test)]
pub fn seed(agents: Vec<RegistryAgent>) {
    *CACHE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some((Instant::now(), agents));
}

/// The registry, from the last hour's copy or fetched now. Blocks: run it
/// off the main thread.
pub fn fetch() -> Result<Vec<RegistryAgent>, String> {
    // no-prop-test: the network has no offline oracle; what is read from it
    // is tested in `kanso_acp::registry`.
    if let Some(agents) = cached() {
        return Ok(agents);
    }
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .https_only(true)
        .max_redirects(3)
        .timeout_global(Some(Duration::from_secs(20)))
        .user_agent("Kanso")
        .build()
        .into();
    let mut response = agent
        .get(registry::URL)
        .call()
        .map_err(|_| "Kanso couldn't reach the ACP registry.".to_owned())?;
    let body = response
        .body_mut()
        .with_config()
        .limit(MAX_BYTES)
        .read_to_string()
        .map_err(|_| "The ACP registry couldn't be read.".to_owned())?;
    let agents = registry::parse(&body)?;
    *CACHE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some((Instant::now(), agents.clone()));
    Ok(agents)
}

/// The registry's agents that aren't featured: "Not tested with Kanso".
pub fn others(agents: &[RegistryAgent]) -> Vec<RegistryAgent> {
    agents
        .iter()
        .filter(|agent| agent.featured().is_none())
        .cloned()
        .collect()
}

/// The package a featured agent installs: the registry's version when it is
/// known, else the latest.
pub fn featured_package(kind: AgentKind, agents: Option<&[RegistryAgent]>) -> Option<String> {
    let package = kind.npm_package()?;
    let pinned = agents
        .into_iter()
        .flatten()
        .filter(|agent| agent.featured() == Some(kind))
        .find_map(|agent| agent.npm_package())
        .filter(|pinned| registry::package_name(pinned) == package);
    Some(pinned.unwrap_or(package).to_owned())
}

/// The registry agent `choice` was picked from, if it was.
pub fn chosen<'a>(choice: &AgentChoice, agents: &'a [RegistryAgent]) -> Option<&'a RegistryAgent> {
    let id = choice.registry.as_deref()?;
    agents.iter().find(|agent| agent.id == id)
}

/// `current` with the agent changed to registry `agent`: a custom command,
/// with the arguments and variables its entry gives, and the command it is
/// installed as (if it is). Note access and its notebooks stay.
pub fn choice_for(agent: &RegistryAgent, current: &AgentChoice) -> AgentChoice {
    let (args, env) = match &agent.distribution {
        Distribution::Npm { args, env, .. } => (args.clone(), env.iter().cloned().collect()),
        Distribution::Other => (Vec::new(), Default::default()),
    };
    AgentChoice {
        kind: AgentKind::Custom,
        command: installed_command(agent),
        args,
        env,
        registry: Some(agent.id.clone()),
        ..current.clone()
    }
}

/// The command a registry agent is installed as, if Kanso can find it.
pub fn installed_command(agent: &RegistryAgent) -> Option<String> {
    let package = agent.npm_package()?;
    let roots = registry::global_roots(&kanso_acp::detect::search_dirs());
    registry::installed_bin(package, &roots)
}

/// What the install sheet says before anything runs: the exact command,
/// the Node it needs, and that it runs as the person.
pub fn confirm_text(name: &str, package: &str, node: Option<u32>, tested: bool) -> String {
    let node = match node {
        Some(major) => format!("It needs Node {major} or newer, with npm."),
        None => "It needs Node.js, with npm.".to_owned(),
    };
    let tested = if tested {
        String::new()
    } else {
        format!(" {name} is not tested with Kanso.")
    };
    format!(
        "Kanso will run this command, as you: npm install -g {package}. {node} The package comes from npm, not from Kanso.{tested} Once installed, {name} runs as a program on this computer with your permissions."
    )
}

/// Installs `package` with the `npm` Kanso finds, in a scrubbed
/// environment. Blocks for as long as npm takes (at most ten minutes).
pub fn install(package: &str) -> Result<(), String> {
    // no-prop-test: runs npm; which packages may run is a property tested in
    // `kanso_acp::registry`.
    let dirs = kanso_acp::detect::search_dirs();
    let npm = kanso_acp::detect::which_in("npm", &dirs).ok_or(
        "npm isn't on this computer. Install Node.js (it comes with npm), then Find again.",
    )?;
    let path = kanso_acp::detect::agent_path(&npm);
    let env = kanso_acp::env::agent_env(std::env::vars(), path, &[]);
    registry::npm_install(package, &dirs, &env)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn agents() -> Vec<RegistryAgent> {
        registry::parse(
            r#"{"agents": [
                {"id": "claude-acp", "name": "Claude Agent", "version": "0.95.0",
                 "distribution": {"npx": {"package": "@agentclientprotocol/claude-agent-acp@0.95.0"}}},
                {"id": "github-copilot-cli", "name": "Copilot", "version": "1.0.103",
                 "distribution": {"npx": {"package": "@github/copilot@1.0.103", "args": ["--acp"]}}},
                {"id": "goose", "name": "goose", "distribution": {"binary": {}}}
            ]}"#,
        )
        .unwrap()
    }

    #[test]
    fn featured_agents_install_the_registrys_version() {
        let agents = agents();
        assert_eq!(
            featured_package(AgentKind::ClaudeAgent, Some(&agents)).as_deref(),
            Some("@agentclientprotocol/claude-agent-acp@0.95.0")
        );
        assert_eq!(
            featured_package(AgentKind::Codex, Some(&agents)).as_deref(),
            Some("@agentclientprotocol/codex-acp")
        );
        assert_eq!(featured_package(AgentKind::OpenCode, Some(&agents)), None);
        let others = others(&agents);
        let ids: Vec<&str> = others.iter().map(|agent| agent.id.as_str()).collect();
        assert_eq!(ids, ["github-copilot-cli", "goose"]);
    }

    #[test]
    fn picking_a_registry_agent_keeps_note_access() {
        let agents = agents();
        let current = AgentChoice {
            mcp: false,
            notebooks: vec!["book:a".into()],
            ..AgentChoice::default()
        };
        let choice = choice_for(&agents[1], &current);
        assert_eq!(choice.kind, AgentKind::Custom);
        assert_eq!(choice.registry.as_deref(), Some("github-copilot-cli"));
        assert_eq!(choice.args, ["--acp"]);
        assert!(!choice.mcp);
        assert_eq!(choice.notebooks, ["book:a"]);
        assert_eq!(
            chosen(&choice, &agents).map(|agent| agent.name.as_str()),
            Some("Copilot")
        );
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(128))]

        /// The install sheet always names the exact package, the Node it
        /// needs and that it runs as the person; an untested agent says so.
        #[test]
        fn prop_the_install_sheet_names_what_will_run(
            name in "[A-Za-z ]{1,20}",
            package in "@?[a-z]{1,8}(/[a-z]{1,8})?(@[0-9]\\.[0-9])?",
            node in prop::option::of(18u32..30),
            tested in any::<bool>(),
        ) {
            // A package that is itself called `npx` would put the word in the
            // sheet legitimately; the property is about the install command.
            prop_assume!(!package.contains("npx"));
            let text = confirm_text(&name, &package, node, tested);
            let command = format!("npm install -g {package}");
            prop_assert!(text.contains(&command));
            prop_assert!(text.contains("Node"));
            prop_assert!(text.contains("as you"));
            prop_assert_eq!(text.contains("not tested with Kanso"), !tested);
            prop_assert!(!text.contains("npx"));
        }
    }
}
