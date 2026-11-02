//! Settings › AI: whether AI is on, where answers come from (an API key or
//! an agent on this computer), and the key that cloud providers need. The
//! key is kept in the Keychain, never in `config.json`; an agent keeps its
//! own sign-in.

use std::sync::Arc;

use gpui_kit::component::Disableable as _;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenuItem};
use gpui_kit::component::switch::Switch;
use gpui_kit::component::{Sizable as _, h_flex, v_flex};
use gpui_kit::*;
use kanso_acp::Detection;
use kanso_acp::registry::{Distribution, RegistryAgent};
use kanso_ai::{AgentCheck, AgentKind, ClientSettings, Provider};
use kanso_sync::SecretStore;

use crate::config::{AiBackend, AiSettings};
use crate::services::Services;
use crate::settings::{Row, section};
use crate::theme::Colors;
use crate::workspace::AppState;
use crate::zoom::px;

/// How long Copy diagnostics says "Copied".
const COPIED_FOR: std::time::Duration = std::time::Duration::from_secs(2);

/// The Keychain item a provider's API key is kept in.
pub fn api_key_item(provider: Provider) -> String {
    format!("ai-key:{}", provider.key())
}

/// The API key saved for `provider`, if one is.
pub fn read_key(cx: &App, provider: Provider) -> Option<String> {
    let secrets = secrets(cx)?;
    read_from(secrets.as_ref(), provider)
}

/// Saves `key` for `provider` in the Keychain.
pub fn write_key(cx: &App, provider: Provider, key: &str) -> Result<(), String> {
    let secrets =
        secrets(cx).ok_or_else(|| "The Keychain isn't available in this build.".to_owned())?;
    write_to(secrets.as_ref(), provider, key)
}

fn secrets(cx: &App) -> Option<Arc<dyn SecretStore>> {
    cx.try_global::<Services>()?.secrets.clone()
}

pub(crate) fn read_from(secrets: &dyn SecretStore, provider: Provider) -> Option<String> {
    secrets
        .get(&api_key_item(provider))
        .ok()
        .flatten()
        .map(|key| key.to_string())
        .filter(|key| !key.trim().is_empty())
}

fn write_to(secrets: &dyn SecretStore, provider: Provider, key: &str) -> Result<(), String> {
    secrets
        .set(&api_key_item(provider), key.trim())
        .map_err(|err| err.to_string())
}

/// The plain line about where a question goes: the server it reaches, and
/// what is never sent. Settings and the Ask panel show the same line.
pub fn requests_line(provider: Provider, base_url: &str) -> String {
    let host = provider.host(base_url);
    if provider.needs_key() {
        format!(
            "Questions and the note excerpts they use go straight from this Mac to {host}. Nothing goes through Kanso's servers. Locked notes and notes tagged #ai-exclude are never sent."
        )
    } else {
        format!(
            "Questions and the note excerpts they use go straight from this Mac to the model at {host}. Nothing goes through Kanso's servers. Locked notes and notes tagged #ai-exclude are never sent."
        )
    }
}

/// The client settings the fields describe.
fn client_settings(ai: &AiSettings, typed_key: &str, saved_key: Option<&str>) -> ClientSettings {
    let typed = typed_key.trim();
    let api_key = if typed.is_empty() {
        saved_key.map(str::to_owned)
    } else {
        Some(typed.to_owned())
    };
    ClientSettings {
        provider: ai.provider,
        base_url: ai.base_url.clone(),
        model: ai.model.clone(),
        api_key,
        region: ai.provider.default_region().to_owned(),
    }
}

/// What the Test button found, or is doing.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) enum Test {
    #[default]
    NotRun,
    Running,
    Ok(String),
    Failed(String),
    /// The agent wants the person to sign in: the ways it offers.
    SignIn(Vec<(String, String)>),
    SigningIn,
}

/// What the one-click install is doing.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) enum Install {
    #[default]
    Idle,
    Running(String),
    Done(String),
    Failed(String),
}

pub struct AiPage {
    base_url: Entity<InputState>,
    model: Entity<InputState>,
    key: Entity<InputState>,
    /// Whether a key is in the Keychain for the chosen provider.
    has_key: bool,
    /// A custom agent's command, arguments and variables.
    agent_command: Entity<InputState>,
    agent_args: Entity<InputState>,
    agent_env: Entity<InputState>,
    /// Where each known agent was found, once looked for.
    detections: Option<Vec<(AgentKind, Detection)>>,
    detect_task: Option<Task<()>>,
    /// The ACP registry, once read; `Some(Err)` when it couldn't be.
    registry: Option<Result<Vec<RegistryAgent>, String>>,
    registry_task: Option<Task<()>>,
    install: Install,
    install_task: Option<Task<()>>,
    test: Test,
    /// Copy diagnostics was just pressed: its button says "Copied".
    diagnostics_copied: bool,
    copied_task: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
    task: Option<Task<()>>,
}

impl AiPage {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let ai = cx.global::<AppState>().config.ai.clone();
        let mut field = |value: &str, placeholder: &'static str, cx: &mut Context<Self>| {
            cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder(placeholder)
                    .default_value(value)
            })
        };
        let base_url = field(&ai.base_url, "http://localhost:11434/v1", cx);
        let model = field(&ai.model, "Model name", cx);
        let agent_command = field(
            ai.agent.command.as_deref().unwrap_or(""),
            "/path/to/agent, or its command",
            cx,
        );
        let agent_args = field(&join_args(&ai.agent.args), "--acp", cx);
        let env_line = ai
            .agent
            .env
            .iter()
            .map(|(name, value)| join_args(&[format!("{name}={value}")]))
            .collect::<Vec<_>>()
            .join(" ");
        let agent_env = field(&env_line, "NAME=value", cx);
        let key = cx.new(|cx| {
            InputState::new(window, cx)
                .masked(true)
                .placeholder("Paste the key")
        });
        let subscriptions = [
            &base_url,
            &model,
            &key,
            &agent_command,
            &agent_args,
            &agent_env,
        ]
        .into_iter()
        .map(|input| {
            cx.subscribe_in(
                input,
                window,
                |this: &mut AiPage, _, event: &InputEvent, window, cx| match event {
                    InputEvent::PressEnter { shift: false, .. } => {
                        this.commit(window, cx);
                    }
                    InputEvent::Blur => this.commit(window, cx),
                    InputEvent::Change => {
                        this.test = Test::NotRun;
                        cx.notify();
                    }
                    _ => {}
                },
            )
        })
        .collect();
        let mut page = AiPage {
            base_url,
            model,
            key,
            has_key: read_key(cx, ai.provider).is_some(),
            agent_command,
            agent_args,
            agent_env,
            detections: None,
            detect_task: None,
            registry: crate::agent_registry::cached().map(Ok),
            registry_task: None,
            install: Install::Idle,
            install_task: None,
            test: Test::NotRun,
            diagnostics_copied: false,
            copied_task: None,
            _subscriptions: subscriptions,
            task: None,
        };
        if ai.enabled && ai.backend == AiBackend::Agent {
            page.find_agents(cx);
        }
        page
    }

    /// Fixed agent states for the snapshots: what was found, and what Test
    /// said.
    #[cfg(test)]
    pub(crate) fn snapshot_agents(&mut self, detections: Vec<(AgentKind, Detection)>, test: Test) {
        self.detections = Some(detections);
        self.detect_task = None;
        self.registry_task = None;
        self.test = test;
    }

    /// A fixed registry for the snapshots.
    #[cfg(test)]
    pub(crate) fn snapshot_registry(&mut self, agents: Vec<RegistryAgent>, install: Install) {
        crate::agent_registry::seed(agents.clone());
        self.registry = Some(Ok(agents));
        self.registry_task = None;
        self.install = install;
    }

    /// Looks for the known agents, off the main thread: a path search, and
    /// once the login shell's `PATH`. Nothing is started.
    /// Reads the ACP registry in the background, once an hour at most.
    /// Settings never waits for it.
    fn fetch_registry(&mut self, cx: &mut Context<Self>) {
        if matches!(self.registry, Some(Ok(_))) || self.registry_task.is_some() {
            return;
        }
        let fetched = cx
            .background_executor()
            .spawn(async move { crate::agent_registry::fetch() });
        self.registry_task = Some(cx.spawn(async move |this, cx| {
            let fetched = fetched.await;
            this.update(cx, |this, cx| {
                this.registry = Some(fetched);
                this.registry_task = None;
                cx.notify();
            })
            .ok();
        }));
    }

    fn registry_agents(&self) -> &[RegistryAgent] {
        match &self.registry {
            Some(Ok(agents)) => agents,
            _ => &[],
        }
    }

    fn find_agents(&mut self, cx: &mut Context<Self>) {
        self.fetch_registry(cx);
        let found = cx.background_executor().spawn(async move {
            let dirs = kanso_acp::detect::search_dirs();
            AgentKind::ALL
                .into_iter()
                .filter(|kind| *kind != AgentKind::Custom)
                .map(|kind| (kind, kanso_acp::detect::detect_in(kind, &dirs)))
                .collect::<Vec<_>>()
        });
        self.detect_task = Some(cx.spawn(async move |this, cx| {
            let found = found.await;
            this.update(cx, |this, cx| {
                this.detections = Some(found);
                this.detect_task = None;
                cx.notify();
            })
            .ok();
        }));
    }

    /// "Find again": looks for the agents again, and lets one that failed to
    /// start be tried again.
    fn find_again(&mut self, cx: &mut Context<Self>) {
        kanso_ai::agent::find_again();
        self.test = Test::NotRun;
        self.find_agents(cx);
        cx.notify();
    }

    /// Answers from an API key or an agent. The other side's settings stay
    /// as they were; a running agent stops.
    fn set_backend(&mut self, backend: AiBackend, cx: &mut Context<Self>) {
        AppState::update_config(cx, |config| config.ai.backend = backend);
        crate::ai_agent::stop_agents(cx);
        self.test = Test::NotRun;
        self.task = None;
        if backend == AiBackend::Agent && self.detections.is_none() {
            self.find_agents(cx);
        }
        cx.notify();
    }

    /// Picks which agent answers.
    /// Picks an agent from the ACP registry: a custom command made from its
    /// entry, not tested with Kanso.
    fn set_registry_agent(
        &mut self,
        agent: RegistryAgent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let current = cx.global::<AppState>().config.ai.agent.clone();
        let choice = crate::agent_registry::choice_for(&agent, &current);
        self.agent_command.update(cx, |field, cx| {
            field.set_value(choice.command.clone().unwrap_or_default(), window, cx)
        });
        self.agent_args.update(cx, |field, cx| {
            field.set_value(join_args(&choice.args), window, cx)
        });
        let env_line = choice
            .env
            .iter()
            .map(|(name, value)| join_args(&[format!("{name}={value}")]))
            .collect::<Vec<_>>()
            .join(" ");
        self.agent_env
            .update(cx, |field, cx| field.set_value(env_line, window, cx));
        AppState::update_config(cx, |config| config.ai.agent = choice);
        crate::ai_agent::stop_agents(cx);
        self.test = Test::NotRun;
        self.install = Install::Idle;
        self.task = None;
        cx.notify();
    }

    /// Lets the agent suggest edits for review, or not. Turning it off
    /// drops the suggestions still waiting.
    fn set_suggest_edits(&mut self, on: bool, cx: &mut Context<Self>) {
        AppState::update_config(cx, |config| config.ai.agent.suggest_edits = on);
        if !on && cx.has_global::<crate::services::Services>() {
            cx.global::<crate::services::Services>()
                .agent_tokens
                .reviews()
                .clear();
        }
        // Tokens already given end; the next question gets the new tools.
        crate::services::end_agent_access(cx);
        cx.notify();
    }

    /// Lets the agent search and read the notes itself, or not.
    fn set_note_access(&mut self, on: bool, cx: &mut Context<Self>) {
        AppState::update_config(cx, |config| config.ai.agent.mcp = on);
        // Tokens already given end; the next question follows the setting.
        crate::services::end_agent_access(cx);
        cx.notify();
    }

    /// What installing the chosen agent would run: its name, the package,
    /// the Node it needs and whether it is tested with Kanso.
    fn install_plan(&self, ai: &AiSettings) -> Option<(String, String, Option<u32>, bool)> {
        let agents = self.registry_agents();
        if ai.agent.kind == AgentKind::Custom {
            let agent = crate::agent_registry::chosen(&ai.agent, agents)?;
            if ai.agent.command.is_some() {
                return None;
            }
            let package = agent.npm_package()?.to_owned();
            return Some((agent.name.clone(), package, None, false));
        }
        if !matches!(
            self.detection(ai.agent.kind),
            Some(Detection::NotInstalled | Detection::OldClaudeAdapter(_))
        ) {
            return None;
        }
        let package = crate::agent_registry::featured_package(ai.agent.kind, Some(agents))?;
        Some((
            ai.agent.kind.label().to_owned(),
            package,
            ai.agent.kind.node(),
            true,
        ))
    }

    /// Asks before installing: the exact command, the Node it needs, and
    /// that it runs as the person. Nothing runs until Install is clicked.
    fn confirm_install(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let ai = cx.global::<AppState>().config.ai.clone();
        let Some((name, package, node, tested)) = self.install_plan(&ai) else {
            return;
        };
        let view = cx.entity();
        crate::dialogs::confirm(
            format!("Install {name}?"),
            crate::agent_registry::confirm_text(&name, &package, node, tested),
            "Install",
            window,
            cx,
            move |_, cx| {
                let (name, package) = (name.clone(), package.clone());
                view.update(cx, |this, cx| this.start_install(name, package, cx));
            },
        );
    }

    fn start_install(&mut self, name: String, package: String, cx: &mut Context<Self>) {
        self.install = Install::Running(format!("Installing {name} with npm…"));
        let worker = package.clone();
        let installed = cx
            .background_executor()
            .spawn(async move { crate::agent_registry::install(&worker) });
        self.install_task = Some(cx.spawn(async move |this, cx| {
            let installed = installed.await;
            this.update(cx, |this, cx| {
                this.install_task = None;
                this.install = match installed {
                    Ok(()) => Install::Done(format!(
                        "{name} is installed. Test checks that it's signed in."
                    )),
                    Err(why) => Install::Failed(why),
                };
                this.installed(cx);
                cx.notify();
            })
            .ok();
        }));
        cx.notify();
    }

    /// After an install: a featured agent is found again; a registry agent
    /// gets the command it was installed as.
    fn installed(&mut self, cx: &mut Context<Self>) {
        let ai = cx.global::<AppState>().config.ai.clone();
        if ai.agent.kind != AgentKind::Custom {
            kanso_ai::agent::find_again();
            self.find_agents(cx);
            return;
        }
        let command = crate::agent_registry::chosen(&ai.agent, self.registry_agents())
            .and_then(crate::agent_registry::installed_command);
        if let Some(command) = command {
            AppState::update_config(cx, |config| config.ai.agent.command = Some(command));
        }
    }

    fn set_agent(&mut self, kind: AgentKind, cx: &mut Context<Self>) {
        AppState::update_config(cx, |config| {
            config.ai.agent.kind = kind;
            config.ai.agent.registry = None;
        });
        // Another agent: the last one stops, and its note access ends.
        crate::ai_agent::stop_agents(cx);
        self.test = Test::NotRun;
        self.install = Install::Idle;
        self.task = None;
        cx.notify();
    }

    /// The page came into view: says whether a key is saved for the provider
    /// chosen now.
    pub fn shown(&mut self, cx: &mut Context<Self>) {
        let provider = cx.global::<AppState>().config.ai.provider;
        let has_key = read_key(cx, provider).is_some();
        if has_key != self.has_key {
            self.has_key = has_key;
            cx.notify();
        }
    }

    /// Turns AI on or off. Menus and any open Ask panel follow.
    pub fn set_enabled(&mut self, on: bool, cx: &mut Context<Self>) {
        AppState::update_config(cx, |config| config.ai.enabled = on);
        crate::settings_ai::ai_changed(cx);
        if !on {
            self.task = None;
            self.test = Test::NotRun;
            crate::ai_agent::stop_agents(cx);
        } else if cx.global::<AppState>().config.ai.backend == AiBackend::Agent
            && self.detections.is_none()
        {
            self.find_agents(cx);
        }
        cx.notify();
    }

    /// Picks `provider`, with the address and model it starts with.
    fn set_provider(&mut self, provider: Provider, window: &mut Window, cx: &mut Context<Self>) {
        AppState::update_config(cx, |config| config.ai.choose(provider));
        let ai = cx.global::<AppState>().config.ai.clone();
        self.base_url.update(cx, |field, cx| {
            field.set_value(ai.base_url.as_str(), window, cx)
        });
        self.model.update(cx, |field, cx| {
            field.set_value(ai.model.as_str(), window, cx)
        });
        self.key
            .update(cx, |field, cx| field.set_value("", window, cx));
        self.has_key = read_key(cx, provider).is_some();
        self.test = Test::NotRun;
        cx.notify();
    }

    /// Writes what the fields say into the config, and saves a typed key in
    /// the Keychain.
    fn commit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let base_url = self.base_url.read(cx).value().trim().to_owned();
        let model = self.model.read(cx).value().trim().to_owned();
        let typed_key = self.key.read(cx).value().trim().to_owned();
        let provider = cx.global::<AppState>().config.ai.provider;
        let command = self.agent_command.read(cx).value().trim().to_owned();
        let args = kanso_acp::detect::split_args(&self.agent_args.read(cx).value());
        let env = kanso_acp::detect::split_env(&self.agent_env.read(cx).value());
        let before = cx.global::<AppState>().config.ai.agent.clone();
        AppState::update_config(cx, |config| {
            if !base_url.is_empty() {
                config.ai.base_url = base_url;
            }
            config.ai.model = model;
            config.ai.agent.command = (!command.is_empty()).then_some(command);
            config.ai.agent.args = args;
            config.ai.agent.env = env.into_iter().collect();
        });
        if cx.global::<AppState>().config.ai.agent != before {
            // A different command is a different agent.
            crate::ai_agent::stop_agents(cx);
        }
        if !typed_key.is_empty() {
            match write_key(cx, provider, &typed_key) {
                Ok(()) => {
                    self.has_key = true;
                    self.key
                        .update(cx, |field, cx| field.set_value("", window, cx));
                }
                Err(err) => {
                    self.test = Test::Failed(format!("Couldn't save the key: {err}"));
                }
            }
        }
        cx.notify();
    }

    /// Sends a tiny request to check the settings, off the main thread.
    fn run_test(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.test == Test::Running {
            return;
        }
        self.commit(window, cx);
        let ai = cx.global::<AppState>().config.ai.clone();
        if ai.backend == AiBackend::Agent {
            self.run_agent_test(cx);
            return;
        }
        let typed_key = self.key.read(cx).value().trim().to_owned();
        let saved = read_key(cx, ai.provider);
        let settings = client_settings(&ai, &typed_key, saved.as_deref());
        self.test = Test::Running;
        cx.notify();
        let probe = cx
            .background_executor()
            .spawn(async move { kanso_ai::probe(&settings) });
        self.task = Some(cx.spawn(async move |this, cx| {
            let result = probe.await;
            this.update(cx, |this, cx| {
                this.task = None;
                this.test = match result {
                    Ok(said) => Test::Ok(format!("OK: {said}")),
                    Err(err) => Test::Failed(err.to_string()),
                };
                cx.notify();
            })
            .ok();
        }));
    }

    /// Starts the chosen agent if needed and asks it "Reply with OK.", with
    /// no note content.
    fn run_agent_test(&mut self, cx: &mut Context<Self>) {
        let state = cx.global::<AppState>();
        let settings = crate::ai_agent::agent_settings(&state.config.ai.agent, &state.data_dir);
        self.test = Test::Running;
        cx.notify();
        let check = cx
            .background_executor()
            .spawn(async move { kanso_ai::agent::check(&settings) });
        self.task = Some(cx.spawn(async move |this, cx| {
            let result = check.await;
            this.update(cx, |this, cx| {
                this.task = None;
                this.test = match result {
                    Ok(AgentCheck::Ready {
                        name,
                        version,
                        said,
                    }) => Test::Ok(ready_line(&name, version.as_deref(), &said)),
                    Ok(AgentCheck::SignIn { methods }) => Test::SignIn(methods),
                    Err(err) => Test::Failed(err.to_string()),
                };
                cx.notify();
            })
            .ok();
        }));
    }

    /// Puts the agent's diagnostics report, which holds no note content, on
    /// the clipboard and says so for a moment.
    fn copy_diagnostics(&mut self, cx: &mut Context<Self>) {
        let state = cx.global::<AppState>();
        let settings = crate::ai_agent::agent_settings(&state.config.ai.agent, &state.data_dir);
        let report = kanso_ai::agent::diagnostics(&settings, env!("CARGO_PKG_VERSION"));
        cx.write_to_clipboard(ClipboardItem::new_string(report));
        self.diagnostics_copied = true;
        cx.notify();
        self.copied_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(COPIED_FOR).await;
            this.update(cx, |this, cx| {
                this.diagnostics_copied = false;
                this.copied_task = None;
                cx.notify();
            })
            .ok();
        }));
    }

    /// Hands sign-in to the agent with its first offered way, then tests
    /// again: one attempt, no loop.
    fn sign_in(&mut self, method: String, cx: &mut Context<Self>) {
        let state = cx.global::<AppState>();
        let settings = crate::ai_agent::agent_settings(&state.config.ai.agent, &state.data_dir);
        self.test = Test::SigningIn;
        cx.notify();
        let signed = cx
            .background_executor()
            .spawn(async move { kanso_ai::agent::sign_in(&settings, &method) });
        self.task = Some(cx.spawn(async move |this, cx| {
            let result = signed.await;
            this.update(cx, |this, cx| {
                this.task = None;
                match result {
                    Ok(()) => this.run_agent_test(cx),
                    Err(err) => this.test = Test::Failed(err.to_string()),
                }
                cx.notify();
            })
            .ok();
        }));
    }

    fn backend_row(&self, ai: &AiSettings, cx: &mut Context<Self>) -> Row {
        let view = cx.entity();
        let chosen = ai.backend;
        let label = |backend: AiBackend| match backend {
            AiBackend::Api => "An API key",
            AiBackend::Agent => "An agent on this computer",
        };
        let button = Button::new("ai-backend")
            .debug_selector(|| "ai-backend".into())
            .outline()
            .small()
            .label(label(chosen))
            .dropdown_caret(true)
            .dropdown_menu(move |mut menu, _, _| {
                for option in [AiBackend::Api, AiBackend::Agent] {
                    let view = view.clone();
                    menu = menu.item(
                        PopupMenuItem::new(label(option))
                            .checked(option == chosen)
                            .on_click(move |_, _, cx| {
                                view.update(cx, |this, cx| this.set_backend(option, cx));
                            }),
                    );
                }
                menu
            });
        Row::new(gpui_kit::assets::IconName::Waypoints, "Answers come from")
            .detail(match chosen {
                AiBackend::Api => {
                    "A provider you have a key for, or a model on this Mac or your network."
                }
                AiBackend::Agent => {
                    "Claude Agent, Codex, Gemini CLI or OpenCode, signed in with your own subscription. It keeps its own sign-in, model and billing."
                }
            })
            .trailing(button)
    }

    fn detection(&self, kind: AgentKind) -> Option<&Detection> {
        self.detections
            .as_ref()?
            .iter()
            .find(|(found, _)| *found == kind)
            .map(|(_, detection)| detection)
    }

    fn agent_row(&self, ai: &AiSettings, cx: &mut Context<Self>) -> Row {
        let view = cx.entity();
        let chosen = ai.agent.kind;
        let words: Vec<(AgentKind, String)> = AgentKind::ALL
            .into_iter()
            .map(|kind| {
                let word = match (kind, self.detection(kind)) {
                    (AgentKind::Custom, _) => "Advanced".to_owned(),
                    (_, Some(detection)) => crate::ai_agent::status_word(detection).to_owned(),
                    (_, None) => "Looking…".to_owned(),
                };
                (kind, format!("{} · {word}", kind.label()))
            })
            .collect();
        let from_registry =
            crate::agent_registry::chosen(&ai.agent, self.registry_agents()).cloned();
        let others = crate::agent_registry::others(self.registry_agents());
        let registry_state = match &self.registry {
            Some(Ok(_)) => None,
            Some(Err(why)) => Some(why.clone()),
            None => Some("Looking up the ACP registry…".to_owned()),
        };
        let picked = ai.agent.registry.clone();
        let button = Button::new("ai-agent")
            .debug_selector(|| "ai-agent".into())
            .outline()
            .small()
            .label(
                from_registry
                    .as_ref()
                    .map(|agent| agent.name.clone())
                    .unwrap_or_else(|| crate::ai_agent::label(&ai.agent)),
            )
            .dropdown_caret(true)
            .dropdown_menu(move |mut menu, window, cx| {
                for (kind, text) in &words {
                    let view = view.clone();
                    let kind = *kind;
                    menu = menu.item(
                        PopupMenuItem::new(text.clone())
                            .checked(
                                kind == chosen && (kind != AgentKind::Custom || picked.is_none()),
                            )
                            .on_click(move |_, _, cx| {
                                view.update(cx, |this, cx| this.set_agent(kind, cx));
                            }),
                    );
                }
                let view = view.clone();
                let others = others.clone();
                let picked = picked.clone();
                let state = registry_state.clone();
                menu.separator().submenu(
                    "More agents from the ACP registry",
                    window,
                    cx,
                    move |mut menu, _, _| {
                        if let Some(state) = &state {
                            return menu.label(state.clone());
                        }
                        for agent in &others {
                            let view = view.clone();
                            let pick = agent.clone();
                            menu = menu.item(
                                PopupMenuItem::new(format!(
                                    "{} · Not tested with Kanso",
                                    agent.name
                                ))
                                .checked(picked.as_deref() == Some(agent.id.as_str()))
                                .on_click(move |_, window, cx| {
                                    let pick = pick.clone();
                                    view.update(cx, |this, cx| {
                                        this.set_registry_agent(pick, window, cx)
                                    });
                                }),
                            );
                        }
                        menu
                    },
                )
            });
        let detail = match (chosen, self.detection(chosen)) {
            (AgentKind::Custom, _) if let Some(agent) = &from_registry => {
                let about = if agent.description.is_empty() {
                    String::new()
                } else {
                    format!("{} ", agent.description.trim_end_matches('.'))
                        .trim_end()
                        .to_owned()
                        + ". "
                };
                format!(
                    "{about}From the ACP registry; not tested with Kanso. It runs as a program on this computer, as you."
                )
            }
            (AgentKind::Custom, _) => {
                "Any other program that speaks the Agent Client Protocol: an executable and its arguments, never a shell command. Not tested with Kanso.".to_owned()
            }
            (kind, Some(detection)) => crate::ai_agent::status_line(kind, detection),
            (_, None) => "Looking for it on this computer…".to_owned(),
        };
        let find = {
            let view = cx.entity();
            Button::new("ai-agent-find")
                .debug_selector(|| "ai-agent-find".into())
                .ghost()
                .small()
                .label("Find again")
                .disabled(self.detect_task.is_some())
                .on_click(move |_, _, cx| view.update(cx, |this, cx| this.find_again(cx)))
        };
        Row::new(gpui_kit::assets::IconName::Terminal, "Agent")
            .detail(detail)
            .trailing(h_flex().gap(px(6.)).child(find).child(button))
    }

    /// How to install the chosen agent when it isn't found: the command,
    /// with Copy, and Install… for an npm package, which runs that command
    /// only after the person confirms it. Kanso never installs anything by
    /// itself.
    fn install_row(&self, ai: &AiSettings, cx: &mut Context<Self>) -> Option<Row> {
        let plan = self.install_plan(ai);
        let command = match (&plan, ai.agent.kind) {
            (Some((_, package, _, _)), _) => format!("npm install -g {package}"),
            (None, AgentKind::Custom) => {
                let agent = crate::agent_registry::chosen(&ai.agent, self.registry_agents())?;
                if ai.agent.command.is_some() || agent.distribution != Distribution::Other {
                    return None;
                }
                let site = agent
                    .website
                    .clone()
                    .unwrap_or_else(|| "its website".to_owned());
                return Some(
                    Row::new(gpui_kit::assets::IconName::Package, "Install")
                        .detail(format!(
                            "Kanso can't install {} itself. Get it from {site}, then put its command under Command.",
                            agent.name
                        )),
                );
            }
            (None, kind) => {
                if !matches!(
                    self.detection(kind),
                    Some(Detection::NotInstalled | Detection::OldClaudeAdapter(_))
                ) {
                    return None;
                }
                kind.install()?.to_owned()
            }
        };
        let detail = match &self.install {
            Install::Idle if plan.is_some() => format!(
                "Install runs this after you confirm, or run it in your terminal and Find again: {command}"
            ),
            Install::Idle => format!("Run this in your terminal, then Find again: {command}"),
            Install::Running(line) | Install::Done(line) | Install::Failed(line) => line.clone(),
        };
        let copy = Button::new("ai-agent-copy-install")
            .debug_selector(|| "ai-agent-copy-install".into())
            .outline()
            .small()
            .label("Copy")
            .on_click(move |_, _, cx| {
                cx.write_to_clipboard(ClipboardItem::new_string(command.clone()));
            });
        let mut buttons = h_flex().gap(px(6.)).child(copy);
        if plan.is_some() {
            let view = cx.entity();
            let running = matches!(self.install, Install::Running(_));
            buttons = buttons.child(
                Button::new("ai-agent-install")
                    .debug_selector(|| "ai-agent-install".into())
                    .outline()
                    .small()
                    .label(if running {
                        "Installing…"
                    } else {
                        "Install…"
                    })
                    .disabled(running)
                    .on_click(move |_, window, cx| {
                        view.update(cx, |this, cx| this.confirm_install(window, cx));
                    }),
            );
        }
        Some(
            Row::new(gpui_kit::assets::IconName::Package, "Install")
                .detail(detail)
                .trailing(buttons),
        )
    }

    /// Whether the agent may search and read the notes itself, and which
    /// notebooks it may.
    fn note_access_rows(&self, ai: &AiSettings, cx: &mut Context<Self>) -> Vec<Row> {
        let view = cx.entity();
        let on = ai.agent.mcp;
        let label = crate::ai_agent::label(&ai.agent);
        let mut rows = vec![
            Row::new(gpui_kit::assets::IconName::BookOpen, "Note access")
                .detail(if on {
                    format!(
                        "{label} may also search and read your notes itself, read only. Every note it reads is listed under Used notes. Locked notes, the trash and notes tagged #ai-exclude never reach it."
                    )
                } else {
                    format!("{label} gets only the excerpts Kanso finds for each question.")
                })
                .trailing(div().debug_selector(|| "ai-agent-mcp".into()).child(
                    Switch::new("ai-agent-mcp").checked(on).on_click(move |on, _, cx| {
                        view.update(cx, |this, cx| this.set_note_access(*on, cx));
                    }),
                )),
        ];
        if on {
            let summary = if ai.agent.notebooks.is_empty() {
                crate::agent_scope::scope_summary(&[], &[])
            } else {
                let store = cx.global::<AppState>().store.clone();
                crate::agent_scope::scope_summary(
                    &ai.agent.notebooks,
                    &crate::agent_scope::scope_books(&store),
                )
            };
            rows.push(
                Row::new(gpui_kit::assets::IconName::Folder, "Notebooks")
                    .detail(summary)
                    .trailing(
                        Button::new("ai-agent-scope")
                            .debug_selector(|| "ai-agent-scope".into())
                            .outline()
                            .small()
                            .label("Choose…")
                            .on_click(|_, window, cx| {
                                crate::agent_scope::open_sheet(window, cx);
                            }),
                    ),
            );
            let suggest = ai.agent.suggest_edits;
            let view = cx.entity();
            rows.push(
                Row::new(gpui_kit::assets::IconName::Pencil, "Let the agent suggest edits")
                    .detail(format!(
                        "{label} may suggest changes to the text and title of these notes, or a new note. Nothing changes until you review a suggestion and apply it in Kanso. Locked notes, the trash and notes tagged #ai-exclude can't be suggested for."
                    ))
                    .trailing(div().debug_selector(|| "ai-agent-suggest".into()).child(
                        Switch::new("ai-agent-suggest")
                            .checked(suggest)
                            .on_click(move |on, _, cx| {
                                view.update(cx, |this, cx| this.set_suggest_edits(*on, cx));
                            }),
                    )),
            );
        }
        rows
    }

    /// Settings > AI > Agent tasks: what a task may propose by default, how
    /// long it may run, how many notes it may read.
    fn agent_task_rows(&self, ai: &AiSettings) -> Vec<Row> {
        use kanso_engine::agent_task::{MAX_NOTES_CHOICES, TIME_LIMITS_MIN};
        let tasks = ai.agent_tasks.normalised();
        let switch = |id: &'static str, on: bool, set: fn(&mut kanso_engine::agent_task::settings::AgentTaskSettings, bool)| {
            div().debug_selector(move || id.into()).child(
                Switch::new(id).checked(on).on_click(move |on, _, cx| {
                    AppState::update_config(cx, |config| set(&mut config.ai.agent_tasks, *on));
                }),
            )
        };
        let choice =
            |id: &'static str,
             label: String,
             options: Vec<(String, bool, Arc<dyn Fn(&mut App) + Send + Sync>)>| {
                Button::new(id)
                    .debug_selector(move || id.into())
                    .outline()
                    .small()
                    .label(label)
                    .dropdown_menu(move |mut menu, _, _| {
                        for (text, checked, pick) in options.clone() {
                            menu = menu.item(
                                PopupMenuItem::new(text)
                                    .checked(checked)
                                    .on_click(move |_, _, cx| pick(cx)),
                            );
                        }
                        menu
                    })
            };
        let time_options = TIME_LIMITS_MIN
            .iter()
            .map(|minutes| {
                let minutes = *minutes;
                let pick: Arc<dyn Fn(&mut App) + Send + Sync> = Arc::new(move |cx| {
                    AppState::update_config(cx, |config| {
                        config.ai.agent_tasks.time_limit_minutes = minutes
                    });
                });
                (
                    format!("{minutes} minutes"),
                    tasks.time_limit_minutes == minutes,
                    pick,
                )
            })
            .collect();
        let note_options = MAX_NOTES_CHOICES
            .iter()
            .map(|notes| {
                let notes = *notes;
                let pick: Arc<dyn Fn(&mut App) + Send + Sync> = Arc::new(move |cx| {
                    AppState::update_config(cx, |config| config.ai.agent_tasks.max_notes = notes);
                });
                (
                    crate::text_stats::format_count(notes),
                    tasks.max_notes == notes,
                    pick,
                )
            })
            .collect();
        vec![
            Row::new(gpui_kit::assets::IconName::FilePlus, "Agents may propose new notes")
                .detail("The defaults for the New agent task sheet. A task never writes: you review every change and apply it, with one Undo.")
                .trailing(switch("ai-task-new", tasks.new_notes, |t, on| t.new_notes = on)),
            Row::new(gpui_kit::assets::IconName::Pencil, "Agents may propose edits")
                .trailing(switch("ai-task-edits", tasks.edits, |t, on| t.edits = on)),
            Row::new(gpui_kit::assets::IconName::Folder, "Agents may propose moves between notebooks")
                .trailing(switch("ai-task-moves", tasks.moves, |t, on| t.moves = on)),
            Row::new(gpui_kit::assets::IconName::Merge, "Agents may propose merges")
                .detail("A merge combines notes into one; the notes it combined move to the Trash.")
                .trailing(switch("ai-task-merges", tasks.merges, |t, on| t.merges = on)),
            Row::new(gpui_kit::assets::IconName::Clock, "Time limit").trailing(choice(
                "ai-task-time",
                format!("{} minutes", tasks.time_limit_minutes),
                time_options,
            )),
            Row::new(gpui_kit::assets::IconName::FileText, "Most notes one task may read").trailing(
                choice(
                    "ai-task-notes",
                    crate::text_stats::format_count(tasks.max_notes),
                    note_options,
                ),
            ),
            Row::new(gpui_kit::assets::IconName::ShieldCheck, "Ask before starting a task that can read all notebooks")
                .trailing(switch("ai-task-ask-all", tasks.ask_before_reading_all, |t, on| {
                    t.ask_before_reading_all = on
                })),
        ]
    }

    fn custom_rows(&self) -> Vec<Row> {
        vec![
            Row::new(gpui_kit::assets::IconName::Command, "Command")
                .detail("The agent's executable: a full path, or a command Kanso finds on your PATH.")
                .trailing(
                    div()
                        .w(px(FIELD_WIDTH))
                        .debug_selector(|| "ai-agent-command".into())
                        .child(Input::new(&self.agent_command)),
                ),
            Row::new(gpui_kit::assets::IconName::List, "Arguments")
                .detail("Separated by spaces; quote one that has spaces in it.")
                .trailing(
                    div()
                        .w(px(FIELD_WIDTH))
                        .debug_selector(|| "ai-agent-args".into())
                        .child(Input::new(&self.agent_args)),
                ),
            Row::new(gpui_kit::assets::IconName::Key, "Environment")
                .detail("NAME=value pairs the agent needs. Kept in the settings file, so no secrets here.")
                .trailing(
                    div()
                        .w(px(FIELD_WIDTH))
                        .debug_selector(|| "ai-agent-env".into())
                        .child(Input::new(&self.agent_env)),
                ),
        ]
    }

    fn agent_test_row(&self, ai: &AiSettings, view: Entity<Self>) -> Row {
        let label = crate::ai_agent::label(&ai.agent);
        let detail = match &self.test {
            Test::NotRun => format!(
                "Starts {label} if needed and asks it to reply with OK. No notes are sent; it uses a little of your subscription or quota."
            ),
            Test::Running => format!("Asking {label}…"),
            Test::Ok(said) => said.clone(),
            Test::Failed(why) => why.clone(),
            Test::SignIn(_) => match ai.agent.kind.sign_in() {
                Some(command) => format!(
                    "{label} needs you to sign in. Sign in lets it open its own page; or run this in your terminal, then Test again: {command}"
                ),
                None => {
                    format!("{label} needs you to sign in. Sign in lets it do that its own way.")
                }
            },
            Test::SigningIn => format!("Waiting for {label} to sign you in…"),
        };
        let busy = matches!(self.test, Test::Running | Test::SigningIn);
        let sign_in = match &self.test {
            Test::SignIn(methods) => methods.first().map(|(id, _)| id.clone()),
            _ => None,
        };
        let mut buttons = h_flex().gap(px(6.));
        if let Some(method) = sign_in {
            let view = view.clone();
            buttons = buttons.child(
                Button::new("ai-agent-sign-in")
                    .debug_selector(|| "ai-agent-sign-in".into())
                    .outline()
                    .small()
                    .label("Sign in")
                    .on_click(move |_, _, cx| {
                        let method = method.clone();
                        view.update(cx, |this, cx| this.sign_in(method, cx));
                    }),
            );
        }
        let copy_view = view.clone();
        buttons = buttons
            .child(
                Button::new("ai-agent-copy-diagnostics")
                    .debug_selector(|| "ai-agent-copy-diagnostics".into())
                    .outline()
                    .small()
                    .label(if self.diagnostics_copied {
                        "Copied"
                    } else {
                        "Copy diagnostics"
                    })
                    .on_click(move |_, _, cx| {
                        copy_view.update(cx, |this, cx| this.copy_diagnostics(cx));
                    }),
            )
            .child(
                Button::new("ai-test")
                    .debug_selector(|| "ai-test".into())
                    .outline()
                    .small()
                    .label(if busy { "Testing…" } else { "Test" })
                    .disabled(busy || !ai.enabled)
                    .on_click(move |_, window, cx| {
                        view.update(cx, |this, cx| this.run_test(window, cx));
                    }),
            );
        Row::new(gpui_kit::assets::IconName::FlaskConical, "Test")
            .detail(detail)
            .trailing(buttons)
    }

    fn provider_row(&self, ai: &AiSettings, cx: &mut Context<Self>) -> Row {
        let view = cx.entity();
        let chosen = ai.provider;
        let provider = Button::new("ai-provider")
            .debug_selector(|| "ai-provider".into())
            .outline()
            .small()
            .label(ai.provider.label())
            .dropdown_caret(true)
            .dropdown_menu(move |mut menu, _, _| {
                for option in Provider::ALL {
                    let view = view.clone();
                    menu = menu.item(
                        PopupMenuItem::new(option.label())
                            .checked(option == chosen)
                            .on_click(move |_, window, cx| {
                                view.update(cx, |this, cx| this.set_provider(option, window, cx));
                            }),
                    );
                }
                menu
            });
        Row::new(gpui_kit::assets::IconName::Cpu, "Provider")
            .detail(provider_detail(ai.provider))
            .trailing(provider)
    }

    fn base_url_row(&self, ai: &AiSettings) -> Row {
        let editable = matches!(ai.provider, Provider::LocalModel | Provider::Custom);
        let row = Row::new(gpui_kit::assets::IconName::Link, "Server address");
        if editable {
            row.detail(if ai.provider == Provider::Custom {
                "The OpenAI-compatible server to ask, for example http://localhost:1234/v1."
            } else {
                "Ollama's OpenAI-compatible address. Change it for a model on another machine."
            })
            .trailing(
                div()
                    .w(px(FIELD_WIDTH))
                    .debug_selector(|| "ai-base-url".into())
                    .child(Input::new(&self.base_url)),
            )
        } else if ai.provider == Provider::Bedrock {
            row.value(Provider::Bedrock.host(""))
        } else {
            row.value(ai.base_url.clone())
        }
    }

    fn model_row(&self, ai: &AiSettings) -> Row {
        let placeholder = if ai.provider == Provider::Custom {
            "The model's name at that server"
        } else {
            "The model to ask"
        };
        Row::new(gpui_kit::assets::IconName::Bot, "Model")
            .detail(placeholder)
            .trailing(
                div()
                    .w(px(FIELD_WIDTH))
                    .debug_selector(|| "ai-model".into())
                    .child(Input::new(&self.model)),
            )
    }

    fn key_row(&self, provider: Provider) -> Row {
        let detail = if self.has_key {
            "Saved in the Keychain. Type a new key to replace it."
        } else if provider.needs_key() {
            "Kept in the Keychain, never in the settings file."
        } else {
            "Optional, for a server that asks for one. Kept in the Keychain, never in the settings file."
        };
        Row::new(gpui_kit::assets::IconName::Key, "API key")
            .detail(detail)
            .trailing(
                div()
                    .w(px(FIELD_WIDTH))
                    .debug_selector(|| "ai-api-key".into())
                    .child(Input::new(&self.key).mask_toggle()),
            )
    }

    fn test_row(&self, ai: &AiSettings, view: Entity<Self>) -> Row {
        let detail = match &self.test {
            Test::NotRun => {
                "Sends one short question to check the address, model and key.".to_owned()
            }
            Test::Running => "Asking the model…".to_owned(),
            Test::Ok(said) => said.clone(),
            Test::Failed(why) => why.clone(),
            Test::SignIn(_) | Test::SigningIn => String::new(),
        };
        let running = self.test == Test::Running;
        Row::new(gpui_kit::assets::IconName::FlaskConical, "Test")
            .detail(detail)
            .trailing(
                Button::new("ai-test")
                    .debug_selector(|| "ai-test".into())
                    .outline()
                    .small()
                    .label(if running { "Testing…" } else { "Test" })
                    .disabled(running || !ai.enabled)
                    .on_click({
                        let view = view.clone();
                        move |_, window, cx| {
                            let view = view.clone();
                            view.update(cx, |this, cx| this.run_test(window, cx));
                        }
                    }),
            )
    }
}

/// What a provider is, in a line.
fn provider_detail(provider: Provider) -> &'static str {
    match provider {
        Provider::LocalModel => {
            "Ollama or another OpenAI-compatible server, on this Mac or your network. No key unless the server asks for one."
        }
        Provider::OpenAi => "OpenAI's own models, with a key from your OpenAI account.",
        Provider::Anthropic => "Claude models, with a key from your Anthropic account.",
        Provider::Gemini => "Google Gemini models, with a key from Google AI Studio.",
        Provider::OpenRouter => "Many providers through OpenRouter, with a key from your account.",
        Provider::Groq => "Groq's fast models, with a key from your Groq account.",
        Provider::DeepSeek => "DeepSeek's models, with a key from your DeepSeek account.",
        Provider::Xai => "xAI's Grok models, with a key from your xAI account.",
        Provider::Mistral => "Mistral's models, with a key from your Mistral account.",
        Provider::Cohere => "Cohere's models, with a key from your Cohere account.",
        Provider::Bedrock => "AWS Bedrock with a bearer token, in the region us-east-1.",
        Provider::Kimi => "Moonshot's Kimi models, with a key from your Moonshot account.",
        Provider::Custom => {
            "Another OpenAI-compatible server, such as LM Studio, at an address you type. A key is optional."
        }
    }
}

/// Turns the AI on or off everywhere it is visible: the View menu, and an
/// Ask panel that was left open.
pub fn ai_changed(cx: &mut App) {
    // The click is inside a window's update; the windows and the menus are
    // reached once it has finished.
    cx.defer(|cx| {
        let enabled = cx
            .try_global::<AppState>()
            .is_some_and(|state| state.config.ai.enabled);
        crate::workspace::set_menus(cx);
        crate::workspace::for_each_workspace(cx, |workspace, cx| {
            workspace.inspector.update(cx, |inspector, cx| {
                if !enabled {
                    inspector.stop_ask(cx);
                    if inspector.tab == crate::config::InspectorTab::Ask {
                        inspector.set_tab(crate::config::InspectorTab::Outline, cx);
                    }
                }
            });
        });
    });
}

/// What Test says when an agent answered.
fn ready_line(name: &str, version: Option<&str>, said: &str) -> String {
    let version = version
        .map(|version| format!(" {version}"))
        .unwrap_or_default();
    format!(
        "Ready · {name}{version} said “{said}”. Billing is with the agent's provider, not Kanso."
    )
}

/// Arguments as one line [`kanso_acp::detect::split_args`] reads back.
fn join_args(args: &[String]) -> String {
    args.iter()
        .map(|arg| {
            if arg.is_empty() || arg.chars().any(char::is_whitespace) {
                format!("\"{arg}\"")
            } else {
                arg.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// The width of a page's fields, as Account's are.
const FIELD_WIDTH: f32 = 260.;

impl Render for AiPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let ai = cx.global::<AppState>().config.ai.clone();
        let view = cx.entity();
        let mut rows = vec![
            Row::new(gpui_kit::assets::IconName::Sparkles, "AI")
                .detail(if ai.enabled {
                    "Answers come from your own notes, with links to the notes they used."
                } else {
                    "Off. Nothing asks a model, and nothing about AI shows, until you turn this on."
                })
                .trailing(div().debug_selector(|| "ai-enabled".into()).child(
                    Switch::new("ai-enabled").checked(ai.enabled).on_click({
                        let view = view.clone();
                        move |on, _, cx| {
                            view.update(cx, |this, cx| this.set_enabled(*on, cx));
                        }
                    }),
                )),
        ];
        if ai.enabled {
            rows.push(
                Row::new(
                    gpui_kit::assets::IconName::Inbox,
                    "Use AI for Inbox suggestions",
                )
                .detail(
                    "Sends the note and the names of likely notebooks and tags to your AI provider.",
                )
                .trailing(div().debug_selector(|| "ai-inbox".into()).child(
                    Switch::new("ai-inbox")
                        .checked(ai.inbox_suggestions)
                        .on_click(|on, _, cx| {
                            AppState::update_config(cx, |config| {
                                config.ai.inbox_suggestions = *on
                            });
                        }),
                )),
            );
            rows.push(self.backend_row(&ai, cx));
            if ai.backend == AiBackend::Agent {
                rows.push(self.agent_row(&ai, cx));
                if let Some(row) = self.install_row(&ai, cx) {
                    rows.push(row);
                }
                if ai.agent.kind == AgentKind::Custom {
                    rows.extend(self.custom_rows());
                }
                rows.extend(self.note_access_rows(&ai, cx));
                rows.push(self.agent_test_row(&ai, view.clone()));
            } else {
                rows.push(self.provider_row(&ai, cx));
                rows.push(self.base_url_row(&ai));
                rows.push(self.model_row(&ai));
                if ai.provider.accepts_key() {
                    rows.push(self.key_row(ai.provider));
                }
                rows.push(self.test_row(&ai, view.clone()));
            }
        }
        let mut page = v_flex().w_full().child(section("AI", rows, colors));
        if ai.enabled {
            page = page.child(v_flex().mt(px(12.)).child(section(
                "Agent tasks",
                self.agent_task_rows(&ai),
                colors,
            )));
        }
        if ai.enabled {
            let lines = if ai.backend == AiBackend::Agent {
                vec![
                    crate::ai_agent::requests_line(&ai.agent),
                    crate::ai_agent::trust_line(&ai.agent),
                ]
            } else {
                vec![requests_line(ai.provider, &ai.base_url)]
            };
            page = page.child(
                v_flex()
                    .mt(px(12.))
                    .gap(px(8.))
                    .text_size(px(12.5))
                    .line_height(px(18.))
                    .text_color(colors.text_secondary)
                    .debug_selector(|| "ai-requests-line".into())
                    .children(lines),
            );
        }
        page
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Test, api_key_item, client_settings, join_args, read_from, ready_line, requests_line,
        write_to,
    };
    use crate::config::AiSettings;
    use kanso_ai::Provider;
    use kanso_sync::keychain::MemorySecrets;
    use proptest::prelude::*;

    #[test]
    fn each_provider_keeps_its_key_apart() {
        assert_eq!(api_key_item(Provider::OpenAi), "ai-key:openai");
        assert_eq!(api_key_item(Provider::Anthropic), "ai-key:anthropic");
        assert_ne!(
            api_key_item(Provider::OpenRouter),
            api_key_item(Provider::OpenAi)
        );
    }

    #[test]
    fn a_key_goes_to_the_keychain_and_comes_back() {
        let secrets = MemorySecrets::default();
        assert_eq!(read_from(&secrets, Provider::Anthropic), None);
        write_to(&secrets, Provider::Anthropic, "  sk-ant-1  ").unwrap();
        assert_eq!(
            read_from(&secrets, Provider::Anthropic).as_deref(),
            Some("sk-ant-1")
        );
        // Another provider has no key of its own.
        assert_eq!(read_from(&secrets, Provider::OpenAi), None);
    }

    #[test]
    fn a_typed_key_is_used_before_the_saved_one() {
        let ai = AiSettings::for_provider(Provider::OpenAi);
        assert_eq!(
            client_settings(&ai, "typed", Some("saved"))
                .api_key
                .as_deref(),
            Some("typed")
        );
        assert_eq!(
            client_settings(&ai, "  ", Some("saved")).api_key.as_deref(),
            Some("saved")
        );
        assert_eq!(client_settings(&ai, "", None).api_key, None);
        assert_eq!(Test::default(), Test::NotRun);
    }

    #[test]
    fn the_line_names_the_server_and_the_exclusions() {
        let line = requests_line(Provider::Anthropic, "https://api.anthropic.com");
        assert!(line.contains("api.anthropic.com"), "{line}");
        assert!(line.contains("#ai-exclude"), "{line}");
        assert!(line.contains("Locked notes"), "{line}");
        assert!(line.contains("Kanso's servers"), "{line}");
    }

    #[test]
    fn a_ready_agent_names_itself_and_where_billing_is() {
        let line = ready_line("Claude Agent", Some("0.94.0"), "OK");
        assert!(line.starts_with("Ready · Claude Agent 0.94.0"), "{line}");
        assert!(line.contains("not Kanso"), "{line}");
        assert_eq!(
            join_args(&["--acp".into(), "two words".into(), String::new()]),
            "--acp \"two words\" \"\""
        );
    }

    proptest! {
        /// A custom agent's arguments, shown in their field and read back,
        /// are the arguments it had.
        #[test]
        fn prop_agent_arguments_survive_the_field(
            args in prop::collection::vec("[A-Za-z0-9 ._/=-]{0,12}", 0..6),
        ) {
            prop_assert_eq!(kanso_acp::detect::split_args(&join_args(&args)), args);
        }

        /// The line always names the server the address points at, says
        /// locked notes and `#ai-exclude` are never sent, and has no line
        /// breaks.
        #[test]
        fn prop_the_line_always_says_the_same(
            provider in prop::sample::select(Provider::ALL.to_vec()),
            host in "[a-z0-9.-]{1,30}",
            scheme in prop::sample::select(vec!["http", "https"]),
        ) {
            let address = format!("{scheme}://{host}/v1");
            let line = requests_line(provider, &address);
            prop_assert!(line.contains(&host));
            prop_assert!(line.contains("#ai-exclude"));
            prop_assert!(line.contains("Locked notes"));
            prop_assert!(!line.contains('\n'));
        }
    }
}
