//! Plugins in the app: the supervisor's side of each host, and what the
//! palette, the preview, the inspector and Settings ask of them.
//!
//! No plugin runs at launch, on a timer or in the background. A host is
//! started when one of the plugin's commands, fenced blocks or panel
//! buttons is used, and stopped once it has been idle for [`IDLE`] with no
//! panel of its open. Safe mode (`--safe-mode`, or Shift held at launch) is
//! decided before any host could start, and keeps every plugin off until
//! the person turns them back on one at a time.
//!
//! What a plugin may do is decided in `kanso-plugins`: the grant, the
//! router that answers its calls against the store, the staged batch. This
//! file holds the hosts (one lock each, so a slow plugin holds up only
//! itself), runs them off the UI thread, and brings results back: text in a
//! notification, a replaced selection as one Undo step, a panel tree, and a
//! batch that is applied at once or after the review sheet.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

use gpui_kit::component::WindowExt as _;
use gpui_kit::component::button::Button;
use gpui_kit::component::notification::Notification;
use gpui_kit::*;
use kanso_plugins::batch::{Batch, MAX_WRITTEN_PER_DAY, review};
use kanso_plugins::consent::Signed;
use kanso_plugins::model::{Disabled, Grant, Installed, Source};
use kanso_plugins::net::{
    EgressLog, Fetcher, HttpsTransport, NetPolicy, RateLimit, SystemResolver,
};
use kanso_plugins::panel_model::Panel;
use kanso_plugins::protocol::{Budgets, Failure, PluginInfo, Theme};
use kanso_plugins::registry_model::{Bundle, Entry, Index};
use kanso_plugins::router::{self, Editor, Network, NoNetwork, Outcome, Request, Router};
use kanso_plugins::state::State;
use kanso_plugins::supervisor::{Health, Host, Log, RunError};
use kanso_plugins::{bundle, grant, install, manifest, registry};
use kanso_store::{PluginWriter, Store};
use kanso_sync::keychain::SecretStore;
use serde_json::{Map, Value};

use crate::plugins_model::{Directory, InstallPlan, PanelState, PendingReview, PluginCommand};
use crate::services::notify_windows;

/// How long a host may sit unused before it is stopped.
pub const IDLE: Duration = Duration::from_secs(30);
/// How often idle hosts, developer folders and `kanso plugin dev` requests
/// are looked at.
const TICK: Duration = Duration::from_secs(2);
/// When the directory is first asked for revocations and updates, after
/// launch, and how often after that. Only with a plugin from it installed.
const FIRST_CHECK: Duration = Duration::from_secs(30);
const CHECK_EVERY: Duration = Duration::from_secs(6 * 60 * 60);
/// The longest text a command's answer may show in a notification.
const MAX_NOTICE: usize = 400;

/// The app's own version, which `minAppVersion` is held to.
pub fn app_version() -> semver::Version {
    semver::Version::parse(env!("CARGO_PKG_VERSION")).unwrap_or(semver::Version::new(0, 0, 0))
}

/// One plugin's host and what goes with it, behind a lock of its own.
#[derive(Default)]
struct Slot {
    host: Option<Host>,
    /// A developer folder's bundle hash when its host started: a change
    /// restarts it.
    dev_hash: Option<String>,
    last_used: Option<Instant>,
    health: Health,
    log: Log,
    egress: Option<EgressLog>,
    rate: Option<RateLimit>,
    /// The day (since the epoch) and the bytes written on it.
    written: (i64, usize),
}

/// Whether safe mode is on, and the plugins turned back on in it.
#[derive(Clone, Debug, Default)]
struct Gate {
    safe_mode: bool,
    session_on: HashSet<String>,
}

/// Everything the hosts need, shared with the drawing threads.
pub struct Shared {
    /// Itself, for the renderer hook, which must not keep it alive.
    me: std::sync::Weak<Shared>,
    store: Arc<Store>,
    plugins_dir: PathBuf,
    host_exe: Option<PathBuf>,
    state: RwLock<State>,
    slots: Mutex<HashMap<String, Arc<Mutex<Slot>>>>,
    gate: RwLock<Gate>,
    secrets: Arc<dyn SecretStore>,
    fetcher: Fetcher<SystemResolver, HttpsTransport>,
    /// The directory index this session has, for revocations and the
    /// lookalike warning.
    index: RwLock<Option<Index>>,
}

/// What a request left behind.
pub struct Report {
    pub result: Result<Value, RunError>,
    pub outcome: Outcome,
    /// Set when this request turned the plugin off.
    pub disabled: Option<Disabled>,
}

/// What a request asks the host to do.
#[derive(Clone, Debug)]
pub enum Run {
    Command {
        id: String,
        args: Value,
    },
    Press {
        button: String,
        values: Map<String, Value>,
    },
}

/// The Keychain item name of plugin `id`'s token `name`, under the app's
/// own service: `plugin <id> <name>`. Pure.
pub fn secret_item(id: &str, name: &str) -> String {
    format!("Kanso Plugin {id} {name}")
}

struct PluginSecrets<'a>(&'a dyn SecretStore);

impl router::Secrets for PluginSecrets<'_> {
    fn get(&self, plugin_id: &str, name: &str) -> Option<String> {
        self.0
            .get(&secret_item(plugin_id, name))
            .ok()
            .flatten()
            .map(|value| value.to_string())
    }
}

/// `kanso.fetch` through the proxy, logged in the plugin's egress log.
struct Net<'a> {
    fetcher: &'a Fetcher<SystemResolver, HttpsTransport>,
    egress: &'a mut EgressLog,
    rate: &'a mut RateLimit,
}

impl Network for Net<'_> {
    fn fetch(&mut self, _: &str, hosts: &BTreeSet<String>, args: &Value) -> Result<Value, Failure> {
        let request = kanso_plugins::net::request_from_args(args).map_err(|e| e.failure())?;
        let policy = NetPolicy {
            hosts: hosts.clone(),
        };
        self.fetcher
            .fetch(
                &policy,
                request,
                kanso_core::now_ms(),
                self.rate,
                self.egress,
            )
            .map(|response| response.to_json())
            .map_err(|e| e.failure())
    }
}

fn theme_of(palette: &kanso_render::Palette) -> Theme {
    Theme {
        dark: palette.dark,
        background: palette.background.hex(),
        text: palette.text.hex(),
        muted: palette.muted.hex(),
        accent: palette.accent.hex(),
        border: palette.border.hex(),
        surface: palette.surface.hex(),
    }
}

impl Shared {
    pub fn new(
        store: Arc<Store>,
        data_dir: &Path,
        safe_mode: bool,
        secrets: Arc<dyn SecretStore>,
    ) -> Arc<Shared> {
        let state = State::load(&store);
        let plugins_dir = install::plugins_dir(data_dir);
        let cache = registry::Cache::new(plugins_dir.join("directory"));
        let index = cache.load(&registry::trusted_keys(), state.last_serial);
        Arc::new_cyclic(|me| Shared {
            me: me.clone(),
            store,
            plugins_dir,
            host_exe: kanso_plugins::supervisor::find_host(),
            state: RwLock::new(state),
            slots: Mutex::new(HashMap::new()),
            gate: RwLock::new(Gate {
                safe_mode,
                session_on: HashSet::new(),
            }),
            secrets,
            fetcher: Fetcher::system(),
            index: RwLock::new(index),
        })
    }

    pub fn state(&self) -> State {
        self.state.read().map(|s| s.clone()).unwrap_or_default()
    }

    pub fn installed(&self, id: &str) -> Option<Installed> {
        self.state.read().ok()?.get(id).cloned()
    }

    /// Changes the state and keeps it in the library.
    fn update_state(&self, change: impl FnOnce(&mut State)) {
        let Ok(mut state) = self.state.write() else {
            return;
        };
        change(&mut state);
        if let Err(err) = state.save(&self.store) {
            eprintln!("kanso: couldn't save the plugins: {err}");
        }
    }

    fn slot(&self, id: &str) -> Arc<Mutex<Slot>> {
        let mut slots = self.slots.lock().expect("plugin slots poisoned");
        slots.entry(id.to_owned()).or_default().clone()
    }

    pub fn safe_mode(&self) -> bool {
        self.gate.read().is_ok_and(|g| g.safe_mode)
    }

    /// Whether plugin `id` may run now: installed, not turned off, and not
    /// held back by safe mode.
    pub fn may_run(&self, id: &str) -> bool {
        let Some(installed) = self.installed(id) else {
            return false;
        };
        let gate = self.gate.read().map(|g| g.clone()).unwrap_or_default();
        installed.may_run() && (!gate.safe_mode || gate.session_on.contains(id))
    }

    /// Plugin `id`'s folder.
    pub fn plugin_dir(&self, id: &str) -> PathBuf {
        install::plugin_dir(&self.plugins_dir, id)
    }

    /// Turns plugin `id` off for `why`, stopping its host.
    pub fn disable(&self, id: &str, why: Disabled) {
        self.stop(id);
        self.update_state(|state| {
            if let Some(plugin) = state.get_mut(id) {
                plugin.disabled = Some(why);
            }
        });
        self.refresh_renderer();
    }

    /// Stops plugin `id`'s host, if it is running.
    pub fn stop(&self, id: &str) {
        let slot = self.slot(id);
        let mut slot = slot.lock().expect("plugin slot poisoned");
        if let Some(host) = slot.host.take() {
            host.stop();
        }
    }

    /// Whether plugin `id`'s host is running.
    pub fn is_running(&self, id: &str) -> bool {
        let slot = self.slot(id);
        let Ok(mut slot) = slot.try_lock() else {
            // Busy with a request.
            return true;
        };
        slot.host.as_mut().is_some_and(Host::is_running)
    }

    /// What plugin `id` logged this session.
    pub fn log(&self, id: &str) -> Vec<(String, String)> {
        let slot = self.slot(id);
        let slot = slot.lock().expect("plugin slot poisoned");
        slot.log.lines().cloned().collect()
    }

    /// The network requests Kanso made for plugin `id`, newest first.
    pub fn egress(&self, id: &str) -> Vec<kanso_plugins::net::Egress> {
        let slot = self.slot(id);
        let mut slot = slot.lock().expect("plugin slot poisoned");
        let dir = self.plugin_dir(id);
        slot.egress
            .get_or_insert_with(|| install::read_egress(&dir))
            .entries()
    }

    /// The fenced languages enabled plugins draw, and tells the renderer.
    pub fn refresh_renderer(&self) {
        let mut languages = BTreeSet::new();
        for plugin in self.state().plugins {
            if self.may_run(plugin.id()) {
                languages.extend(plugin.grant.fences.iter().cloned());
            }
        }
        let shared = self.me.clone();
        let renderer: Arc<kanso_render::PluginRenderer> =
            Arc::new(move |language, source, palette| {
                let shared = shared.upgrade().ok_or("Kanso is closing")?;
                shared.render(language, source, palette)
            });
        kanso_render::set_plugin_renderer(Some(renderer), languages);
    }

    /// The plugin that draws `language`, if one may run.
    fn renderer_for(&self, language: &str) -> Option<String> {
        self.state()
            .plugins
            .iter()
            .filter(|p| {
                p.grant
                    .fences
                    .iter()
                    .any(|f| f.eq_ignore_ascii_case(language))
            })
            .map(|p| p.id().to_owned())
            .find(|id| self.may_run(id))
    }

    /// Draws a fenced block on the calling (drawing) thread. Blocks while
    /// the plugin's host answers.
    pub fn render(
        &self,
        language: &str,
        source: &str,
        palette: &kanso_render::Palette,
    ) -> Result<String, String> {
        let id = self
            .renderer_for(language)
            .ok_or_else(|| format!("No plugin draws {language} blocks"))?;
        let installed = self.installed(&id).ok_or("The plugin was removed")?;
        let slot = self.slot(&id);
        let mut slot = slot.lock().map_err(|_| "The plugin stopped".to_owned())?;
        if let Err(err) = self.ensure_host(&installed, &mut slot) {
            return Err(err.to_string());
        }
        let editor = Editor::default();
        let dir = self.plugin_dir(&id);
        let mut network = NoNetwork;
        let no_secrets = NoSecrets;
        let mut router = Router {
            store: &self.store,
            plugin: &installed,
            plugin_dir: &dir,
            editor: &editor,
            request: Request::Render,
            secrets: &no_secrets,
            network: &mut network,
            out: Outcome::default(),
        };
        let Slot { host, log, .. } = &mut *slot;
        let host = host.as_mut().ok_or("The plugin isn't running")?;
        let drawn = host.render(language, source, theme_of(palette), &mut router, log);
        let refused = router.out.refused;
        slot.last_used = Some(Instant::now());
        let disabled = self.settle(&id, &mut slot, drawn.as_ref().err(), refused);
        drop(slot);
        if let Some(why) = disabled {
            self.disable(&id, why);
        }
        drawn.map_err(|err| err.to_string())
    }

    /// Starts plugin `installed`'s host unless it is running (and, in
    /// developer mode, its folder is unchanged), recording what it
    /// registered.
    fn ensure_host(&self, installed: &Installed, slot: &mut Slot) -> Result<(), RunError> {
        let id = installed.id().to_owned();
        let dev_hash = match &installed.source {
            Source::Developer { path } => bundle::pack_dir(Path::new(path))
                .ok()
                .map(|bytes| bundle::sha256_hex(&bytes)),
            _ => None,
        };
        if let Some(host) = slot.host.as_mut()
            && host.is_running()
            && (dev_hash.is_none() || dev_hash == slot.dev_hash)
        {
            return Ok(());
        }
        if let Some(host) = slot.host.take() {
            host.stop();
        }
        let exe = self
            .host_exe
            .clone()
            .ok_or_else(|| RunError::FailedToStart("Kanso's plugin host is missing".into()))?;
        let dir = self.plugin_dir(&id);
        std::fs::create_dir_all(&dir).ok();
        let started = install::entry_script(&dir, installed)
            .map_err(RunError::FailedToStart)
            .and_then(|code| {
                Host::start(
                    &exe,
                    &dir,
                    PluginInfo {
                        id: id.clone(),
                        name: installed.manifest.name.clone(),
                        version: installed.manifest.version.clone(),
                    },
                    code,
                    Budgets::DEFAULT,
                )
            });
        match started {
            Ok((host, registered)) => {
                slot.host = Some(host);
                slot.dev_hash = dev_hash;
                if registered != installed.registered {
                    self.update_state(|state| {
                        if let Some(plugin) = state.get_mut(&id) {
                            plugin.registered = registered;
                            plugin.last_error = None;
                        }
                    });
                }
                Ok(())
            }
            Err(err) => {
                let message = err.to_string();
                self.update_state(|state| {
                    if let Some(plugin) = state.get_mut(&id) {
                        plugin.last_error = Some(message);
                    }
                });
                if let Some(why) = slot.health.failed_to_start() {
                    self.update_state(|state| {
                        if let Some(plugin) = state.get_mut(&id) {
                            plugin.disabled = Some(why);
                        }
                    });
                }
                Err(err)
            }
        }
    }

    /// Counts what a request cost the plugin's health; what it says to turn
    /// the plugin off with, if anything.
    fn settle(
        &self,
        id: &str,
        slot: &mut Slot,
        error: Option<&RunError>,
        refused: u32,
    ) -> Option<Disabled> {
        let mut disabled = slot.health.refused(refused);
        if let Some(err) = error {
            if err.lost_the_process() {
                slot.host = None;
                disabled = disabled.or(slot.health.crashed(Instant::now()));
            }
            let message = err.to_string();
            self.update_state(|state| {
                if let Some(plugin) = state.get_mut(id) {
                    plugin.last_error = Some(message);
                }
            });
        }
        disabled
    }

    /// Runs `run` for plugin `id` with `editor` as the editor stands.
    /// Blocks; call it off the UI thread.
    pub fn run(&self, id: &str, run: Run, editor: Editor) -> Report {
        let fail = |err: RunError| Report {
            result: Err(err),
            outcome: Outcome::default(),
            disabled: None,
        };
        if !self.may_run(id) {
            return fail(RunError::FailedToStart("it's turned off".into()));
        }
        let Some(installed) = self.installed(id) else {
            return fail(RunError::FailedToStart("it was removed".into()));
        };
        let slot_lock = self.slot(id);
        let mut slot = slot_lock.lock().expect("plugin slot poisoned");
        if let Err(err) = self.ensure_host(&installed, &mut slot) {
            let disabled = self.installed(id).and_then(|p| p.disabled);
            return Report {
                result: Err(err),
                outcome: Outcome::default(),
                disabled,
            };
        }
        let dir = self.plugin_dir(id);
        let secrets = PluginSecrets(self.secrets.as_ref());
        let Slot {
            host,
            log,
            egress,
            rate,
            ..
        } = &mut *slot;
        let egress = egress.get_or_insert_with(|| install::read_egress(&dir));
        let before = egress.len();
        let rate = rate.get_or_insert_with(RateLimit::new);
        let mut network = Net {
            fetcher: &self.fetcher,
            egress,
            rate,
        };
        let request = match &run {
            Run::Command { .. } => Request::Command,
            Run::Press { .. } => Request::Press,
        };
        let mut router = Router {
            store: &self.store,
            plugin: &installed,
            plugin_dir: &dir,
            editor: &editor,
            request,
            secrets: &secrets,
            network: &mut network,
            out: Outcome::default(),
        };
        let host = host.as_mut().expect("the host was just started");
        let result = match run {
            Run::Command { id, args } => host.invoke(&id, args, &mut router, log),
            Run::Press { button, values } => host.press(&button, values, &mut router, log),
        };
        let outcome = std::mem::take(&mut router.out);
        drop(router);
        if let Some(egress) = &slot.egress
            && egress.len() != before
        {
            install::write_egress(&dir, egress).ok();
        }
        slot.last_used = Some(Instant::now());
        let disabled = self.settle(id, &mut slot, result.as_ref().err(), outcome.refused);
        drop(slot);
        if let Some(why) = &disabled {
            self.disable(id, why.clone());
        }
        Report {
            result,
            outcome,
            disabled,
        }
    }

    /// Applies a reviewed (or unreviewed, for the open note) batch for
    /// plugin `id`, held to its daily write budget.
    pub fn apply(&self, id: &str, batch: &Batch) -> Result<kanso_store::PluginReport, String> {
        let installed = self.installed(id).ok_or("The plugin was removed.")?;
        let slot = self.slot(id);
        let mut slot = slot.lock().map_err(|_| "The plugin stopped.".to_owned())?;
        let today = kanso_core::now_ms() / 86_400_000;
        if slot.written.0 != today {
            slot.written = (today, 0);
        }
        if slot.written.1 + batch.bytes > MAX_WRITTEN_PER_DAY {
            return Err(format!(
                "{} has written as much as a plugin may today.",
                installed.manifest.name
            ));
        }
        let writer = PluginWriter {
            plugin_id: installed.id(),
            plugin_name: &installed.manifest.name,
            batch: &batch.id,
        };
        let report = self
            .store
            .apply_plugin_batch(writer, &batch.edits, &batch.creates, kanso_core::now_ms())
            .map_err(|err| match err {
                kanso_store::StoreError::Conflict => {
                    "A note changed after the plugin read it, so nothing was changed. Run it again."
                        .to_owned()
                }
                other => other.to_string(),
            })?;
        slot.written.1 += batch.bytes;
        Ok(report)
    }

    /// Stops every host idle for [`IDLE`], except `keep`'s (its panel is
    /// open).
    pub fn reap(&self, keep: Option<&str>) {
        let slots: Vec<(String, Arc<Mutex<Slot>>)> = self
            .slots
            .lock()
            .map(|s| s.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default();
        for (id, slot) in slots {
            if Some(id.as_str()) == keep {
                continue;
            }
            let Ok(mut slot) = slot.try_lock() else {
                continue;
            };
            if slot.last_used.is_none_or(|at| at.elapsed() >= IDLE)
                && let Some(host) = slot.host.take()
            {
                host.stop();
            }
        }
    }

    /// Turns off every installed plugin the directory, the compiled list or
    /// this Mac's block list refuses, or that needs a newer Kanso, before
    /// any of them can start.
    pub fn check_revocations(self: &Arc<Self>) {
        let index = self.index.read().ok().and_then(|i| i.clone());
        let app = app_version();
        self.update_state(|state| {
            let blocked = state.blocked.clone();
            for plugin in &mut state.plugins {
                let id = plugin.manifest.id.clone();
                if blocked.contains(&id) {
                    plugin.disabled = Some(Disabled::Blocked);
                } else if !plugin.is_developer()
                    && let Some(reason) = registry::revocation_reason(
                        index.as_ref(),
                        &blocked,
                        &id,
                        &plugin.manifest.version,
                    )
                {
                    plugin.disabled = Some(Disabled::Revoked { reason });
                } else if manifest::check_app_version(&plugin.manifest, &app).is_err() {
                    plugin.disabled = Some(Disabled::NeedsNewerKanso {
                        version: plugin.manifest.min_app_version.clone(),
                    });
                } else if matches!(plugin.disabled, Some(Disabled::NeedsNewerKanso { .. })) {
                    plugin.disabled = None;
                }
            }
        });
        for plugin in self.state().plugins {
            if !plugin.may_run() {
                self.stop(plugin.id());
            }
        }
        self.refresh_renderer();
    }

    /// Plans installing `bundle` from `source`: the lookalike warning, the
    /// grant to start from (the current one, cut to the new manifest, for
    /// an update), and whether it replaces an installed plugin.
    pub fn plan(&self, bundle: Bundle, source: Source) -> Result<InstallPlan, String> {
        let id = bundle.manifest.id.clone();
        manifest::validate(&bundle.manifest).map_err(|e| e.to_string())?;
        manifest::check_app_version(&bundle.manifest, &app_version()).map_err(|e| e.to_string())?;
        let state = self.state();
        if state.blocked.contains(&id) {
            return Err("You blocked this plugin on this Mac.".into());
        }
        if !matches!(source, Source::Developer { .. }) {
            let index = self.index.read().ok().and_then(|i| i.clone());
            if let Some(reason) = registry::revocation_reason(
                index.as_ref(),
                &state.blocked,
                &id,
                &bundle.manifest.version,
            ) {
                return Err(format!("The directory withdrew this version: {reason}"));
            }
        }
        let replacing = state.get(&id).cloned();
        if let Some(old) = &replacing
            && !old.is_developer()
            && !matches!(source, Source::Developer { .. })
        {
            let old_version = semver::Version::parse(&old.manifest.version).ok();
            let new_version = semver::Version::parse(&bundle.manifest.version).ok();
            if new_version <= old_version {
                return Err(
                    "This is the version installed, or an older one: Kanso never goes back.".into(),
                );
            }
        }
        let mut known: Vec<String> = state.ids().map(str::to_owned).collect();
        if let Some(index) = self.index.read().ok().and_then(|i| i.clone()) {
            known.extend(index.plugins.into_iter().map(|e| e.id).take(100));
        }
        let lookalikes = manifest::lookalikes(&id, known.iter().map(String::as_str))
            .into_iter()
            .map(str::to_owned)
            .collect();
        let grant = match &replacing {
            Some(old) => grant::narrow_to(&old.grant, &bundle.manifest),
            None => grant::initial(&bundle.manifest),
        };
        let offer = replacing.as_ref().map(|old| {
            let (added, removed) = kanso_plugins::consent::diff(
                &old.manifest.permissions,
                &bundle.manifest.permissions,
            );
            kanso_plugins::registry_model::Offer {
                entry: Entry {
                    id: id.clone(),
                    name: bundle.manifest.name.clone(),
                    publisher: bundle.manifest.publisher.name.clone(),
                    description: bundle.manifest.description.clone().unwrap_or_default(),
                    version: bundle.manifest.version.clone(),
                    sha256: bundle.sha256.clone(),
                    size: 0,
                    signer: bundle.manifest.publisher.key.clone(),
                    author_signature: String::new(),
                    min_app_version: bundle.manifest.min_app_version.clone(),
                    permissions: bundle.manifest.permissions.clone(),
                    published_at: String::new(),
                    revoked: false,
                },
                added,
                removed,
                key_changed: !old.is_developer() && old.pinned_key != bundle.manifest.publisher.key,
            }
        });
        let signed = match source {
            Source::Developer { .. } => Signed::Developer,
            _ => Signed::Directory,
        };
        Ok(InstallPlan {
            bundle,
            source,
            signed,
            lookalikes,
            replacing,
            grant,
            offer,
        })
    }

    /// Installs `plan` with `grant` once the person has said so on the
    /// sheet, replacing the plugin if it was installed.
    pub fn install(self: &Arc<Self>, plan: &InstallPlan, grant: Grant) -> Result<(), String> {
        let id = plan.id().to_owned();
        let locked = self.store.locked_books().map_err(|e| e.to_string())?;
        grant::check(&grant, &plan.bundle.manifest, &locked).map_err(|e| e.to_string())?;
        self.stop(&id);
        if !matches!(plan.source, Source::Developer { .. }) {
            install::install_bundle(&self.plugins_dir, &plan.bundle).map_err(|e| e.to_string())?;
        } else {
            std::fs::create_dir_all(self.plugin_dir(&id)).map_err(|e| e.to_string())?;
        }
        let installed = Installed {
            manifest: plan.bundle.manifest.clone(),
            source: plan.source.clone(),
            // Trust on first use: the key pinned now is the one every
            // update must be signed by, or be confirmed in red.
            pinned_key: plan.bundle.manifest.publisher.key.clone(),
            grant,
            disabled: None,
            registered: Default::default(),
            installed_at: kanso_core::now_ms(),
            last_error: None,
        };
        self.update_state(|state| state.upsert(installed));
        let slot = self.slot(&id);
        slot.lock().expect("plugin slot poisoned").health.reset();
        self.refresh_renderer();
        Ok(())
    }

    /// Runs a newly installed plugin's entry script once, so its commands
    /// reach the palette. Blocks.
    pub fn introduce(&self, id: &str) -> Result<(), String> {
        let installed = self.installed(id).ok_or("It was removed.")?;
        let slot = self.slot(id);
        let mut slot = slot.lock().map_err(|_| "It stopped.".to_owned())?;
        self.ensure_host(&installed, &mut slot)
            .map_err(|e| e.to_string())?;
        slot.last_used = Some(Instant::now());
        Ok(())
    }

    /// Removes plugin `id`: its host, folder, tokens and state, and with
    /// `sweep` its data on notes. How many notes changed.
    pub fn remove(self: &Arc<Self>, id: &str, sweep: bool) -> Result<usize, String> {
        let installed = self.installed(id).ok_or("It isn't installed.")?;
        self.stop(id);
        install::remove(&self.plugins_dir, id).map_err(|e| e.to_string())?;
        for name in &installed.manifest.permissions.secrets {
            self.secrets.delete(&secret_item(id, name)).ok();
        }
        self.update_state(|state| {
            state.remove(id);
        });
        if let Ok(mut slots) = self.slots.lock() {
            slots.remove(id);
        }
        self.refresh_renderer();
        if sweep {
            self.store.remove_plugin_data(id).map_err(|e| e.to_string())
        } else {
            Ok(0)
        }
    }

    /// Gives plugin `id` `grant` instead: its host stops and starts again
    /// with it on next use.
    pub fn set_grant(self: &Arc<Self>, id: &str, grant: Grant) -> Result<(), String> {
        let installed = self.installed(id).ok_or("It isn't installed.")?;
        let locked = self.store.locked_books().map_err(|e| e.to_string())?;
        grant::check(&grant, &installed.manifest, &locked).map_err(|e| e.to_string())?;
        self.stop(id);
        self.update_state(|state| {
            if let Some(plugin) = state.get_mut(id) {
                plugin.grant = grant;
            }
        });
        self.refresh_renderer();
        Ok(())
    }

    /// Turns plugin `id` on (clearing any reason it was off, after Try
    /// again) or off.
    pub fn set_enabled(self: &Arc<Self>, id: &str, on: bool) {
        if !on {
            self.disable(id, Disabled::ByYou);
            return;
        }
        self.update_state(|state| {
            if let Some(plugin) = state.get_mut(id)
                && !matches!(
                    plugin.disabled,
                    Some(
                        Disabled::Revoked { .. }
                            | Disabled::Blocked
                            | Disabled::NeedsNewerKanso { .. }
                    )
                )
            {
                plugin.disabled = None;
            }
        });
        self.slot(id)
            .lock()
            .expect("plugin slot poisoned")
            .health
            .reset();
        if let Ok(mut gate) = self.gate.write()
            && gate.safe_mode
        {
            gate.session_on.insert(id.to_owned());
        }
        self.refresh_renderer();
    }

    /// Adds plugin `id` to this Mac's block list, turning it off.
    pub fn block(self: &Arc<Self>, id: &str) {
        self.update_state(|state| {
            state.blocked.insert(id.to_owned());
        });
        self.disable(id, Disabled::Blocked);
    }

    /// Saves (or with an empty value removes) plugin `id`'s token `name`.
    pub fn set_secret(&self, id: &str, name: &str, value: &str) -> Result<(), String> {
        let item = secret_item(id, name);
        if value.is_empty() {
            self.secrets.delete(&item)
        } else {
            self.secrets.set(&item, value)
        }
        .map_err(|e| e.to_string())
    }

    pub fn has_secret(&self, id: &str, name: &str) -> bool {
        self.secrets
            .get(&secret_item(id, name))
            .is_ok_and(|v| v.is_some())
    }

    /// Takes a freshly fetched directory index: checked against the keys
    /// and the newest serial seen, kept, and used for revocations.
    pub fn take_index(self: &Arc<Self>, bytes: &[u8], signature: &[u8]) -> Result<Index, String> {
        let last = self.state().last_serial;
        let index = registry::check_index(bytes, signature, &registry::trusted_keys(), last)
            .map_err(|e| e.to_string())?;
        registry::Cache::new(self.plugins_dir.join("directory"))
            .save(bytes, signature)
            .ok();
        let serial = index.serial;
        self.update_state(|state| state.last_serial = Some(serial));
        if let Ok(mut held) = self.index.write() {
            *held = Some(index.clone());
        }
        self.check_revocations();
        Ok(index)
    }

    pub fn index(&self) -> Option<Index> {
        self.index.read().ok().and_then(|i| i.clone())
    }
}

struct NoSecrets;

impl router::Secrets for NoSecrets {
    fn get(&self, _: &str, _: &str) -> Option<String> {
        None
    }
}

// MARK: The app's side

/// Plugins as the windows see them.
pub struct Plugins {
    pub shared: Arc<Shared>,
    /// A batch waiting for the review sheet.
    pub review: Option<PendingReview>,
    /// The panel in the inspector.
    pub panel: Option<PanelState>,
    pub directory: Directory,
    /// Updates the directory offers, by plugin ID.
    pub offers: Vec<kanso_plugins::registry_model::Offer>,
    /// Commands running now, as `plugin/command`.
    pub running: HashSet<String>,
    /// The last batch applied, which Undo plugin changes takes back.
    pub last_batch: Option<(String, String)>,
    /// `kanso plugin dev` folders waiting for the person to trust them.
    pub dev_requests: Vec<PathBuf>,
    _tick: Option<Task<()>>,
}

impl Global for Plugins {}

/// The plugins, when they were set up.
fn plugins_mut(cx: &mut App) -> Option<&mut Plugins> {
    cx.has_global::<Plugins>()
        .then(|| cx.global_mut::<Plugins>())
}

/// Whether to start in safe mode: `--safe-mode` given, or Shift held as
/// Kanso opens.
pub fn safe_mode_requested(args: &[String]) -> bool {
    args.iter().any(|a| a == "--safe-mode") || shift_held()
}

#[cfg(target_os = "macos")]
fn shift_held() -> bool {
    use objc2_app_kit::{NSEvent, NSEventModifierFlags};
    NSEvent::modifierFlags_class().contains(NSEventModifierFlags::Shift)
}

#[cfg(not(target_os = "macos"))]
fn shift_held() -> bool {
    false
}

/// Where `kanso plugin dev` leaves folders for the app to offer.
fn dev_requests_dir(data_dir: &Path) -> PathBuf {
    install::plugins_dir(data_dir).join("developer-requests")
}

impl Plugins {
    /// Sets plugins up for this session: no host starts here.
    pub fn init(
        store: Arc<Store>,
        data_dir: &Path,
        safe_mode: bool,
        secrets: Arc<dyn SecretStore>,
        cx: &mut App,
    ) {
        let shared = Shared::new(store, data_dir, safe_mode, secrets);
        shared.check_revocations();
        let directory = if registry::is_configured() {
            match shared.index() {
                Some(index) => Directory::Loaded(index),
                None => Directory::Idle,
            }
        } else {
            Directory::NotSetUp
        };
        let requests = dev_requests_dir(data_dir);
        let tick = cx.spawn(async move |cx| {
            let mut next_check = Instant::now() + FIRST_CHECK;
            loop {
                cx.background_executor().timer(TICK).await;
                if Instant::now() >= next_check {
                    next_check = Instant::now() + CHECK_EVERY;
                    cx.update(|cx| {
                        let wanted = cx
                            .try_global::<Plugins>()
                            .is_some_and(|p| checks_directory(&p.shared.state()));
                        if wanted && registry::is_configured() {
                            refresh_directory(cx);
                        }
                    });
                }
                let Some((shared, keep)) = cx.update(|cx| {
                    let plugins = cx.try_global::<Plugins>()?;
                    let keep = plugins.panel.as_ref().map(|p| p.panel.plugin_id.clone());
                    Some((plugins.shared.clone(), keep))
                }) else {
                    break;
                };
                let requests = requests.clone();
                let found = cx
                    .background_spawn(async move {
                        shared.reap(keep.as_deref());
                        take_dev_requests(&requests)
                    })
                    .await;
                if found.is_empty() {
                    continue;
                }
                cx.update(|cx| {
                    if let Some(plugins) = plugins_mut(cx) {
                        for path in found {
                            if !plugins.dev_requests.contains(&path) {
                                plugins.dev_requests.push(path);
                            }
                        }
                    }
                    cx.refresh_windows();
                });
            }
        });
        cx.set_global(Plugins {
            shared,
            review: None,
            panel: None,
            directory,
            offers: Vec::new(),
            running: HashSet::new(),
            last_batch: None,
            dev_requests: Vec::new(),
            _tick: Some(tick),
        });
    }

    /// The commands the palette offers now.
    pub fn commands(cx: &App) -> Vec<PluginCommand> {
        let Some(plugins) = cx.try_global::<Plugins>() else {
            return Vec::new();
        };
        let shared = &plugins.shared;
        shared
            .state()
            .plugins
            .iter()
            .filter(|p| p.grant.commands && shared.may_run(p.id()))
            .flat_map(|p| {
                p.registered.commands.iter().map(|spec| PluginCommand {
                    plugin_id: p.id().to_owned(),
                    plugin_name: p.manifest.name.clone(),
                    spec: spec.clone(),
                })
            })
            .collect()
    }
}

/// Whether the directory is worth asking for revocations and updates: a
/// plugin from it is installed. Developer folders never are. Pure.
// no-prop-test: one `any` over `Installed::is_developer`, which has its own tests.
pub fn checks_directory(state: &State) -> bool {
    state.plugins.iter().any(|p| !p.is_developer())
}

/// Reads and clears the folders `kanso plugin dev` asked for.
fn take_dev_requests(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut found = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if let Ok(text) = std::fs::read_to_string(&path)
            && let Ok(folder) = serde_json::from_str::<String>(&text)
        {
            found.push(PathBuf::from(folder));
        }
        std::fs::remove_file(&path).ok();
    }
    found
}

/// A few words of a command's answer for a notification: its `text`, or the
/// answer itself when it is text. Pure.
pub fn notice_text(value: &Value) -> Option<String> {
    let text = match value {
        Value::String(text) => text.clone(),
        Value::Object(fields) => fields.get("text")?.as_str()?.to_owned(),
        _ => return None,
    };
    let text = text.trim();
    if text.is_empty() {
        return None;
    }
    let mut shown: String = text.chars().take(MAX_NOTICE).collect();
    if text.chars().count() > MAX_NOTICE {
        shown.push('…');
    }
    Some(shown)
}

/// Where the editor stands, for a plugin: its open note and selection.
pub fn editor_context(editor: &Entity<crate::editor::NoteEditor>, cx: &App) -> Editor {
    let editor = editor.read(cx);
    let note_id = editor.note_id().map(str::to_owned);
    let selection = editor
        .plugin_selection(cx)
        .map(|(start, end, text)| router::Selection { text, start, end });
    Editor { note_id, selection }
}

/// Runs plugin command `command` of `plugin_id` for the editor as it is.
pub fn run_command(
    plugin_id: String,
    command: String,
    editor: Entity<crate::editor::NoteEditor>,
    cx: &mut App,
) {
    // The plugin reads the library, so what was typed is saved first.
    editor.update(cx, |editor, cx| editor.flush(cx));
    let context = editor_context(&editor, cx);
    let seen = context.selection.clone();
    let key = format!("{plugin_id}/{command}");
    let Some(plugins) = plugins_mut(cx) else {
        return;
    };
    if !plugins.running.insert(key.clone()) {
        return;
    }
    let shared = plugins.shared.clone();
    let run = Run::Command {
        id: command,
        args: Value::Null,
    };
    let id = plugin_id.clone();
    let task = cx.background_spawn(async move { shared.run(&id, run, context) });
    crate::metrics::record(cx, crate::metrics::Feature::PluginCommand);
    cx.spawn(async move |cx| {
        let report = task.await;
        cx.update(|cx| {
            if let Some(plugins) = plugins_mut(cx) {
                plugins.running.remove(&key);
            }
            finish(&plugin_id, report, Some(editor), seen, cx);
        });
    })
    .detach();
}

/// Presses `button` in the open panel, with its inputs as they stand.
pub fn press(button: String, cx: &mut App) {
    let Some(plugins) = plugins_mut(cx) else {
        return;
    };
    let Some(panel) = plugins.panel.as_mut() else {
        return;
    };
    if panel.busy {
        return;
    }
    panel.busy = true;
    let held: Map<String, Value> = panel.values.clone().into_iter().collect();
    let values = router::press_values(&panel.panel.root, &held);
    let id = panel.panel.plugin_id.clone();
    let shared = plugins.shared.clone();
    let run = Run::Press { button, values };
    let editor = crate::workspace::main_workspace(cx)
        .map(|(_, workspace)| workspace.read(cx).editor.clone());
    if let Some(editor) = &editor {
        editor.update(cx, |editor, cx| editor.flush(cx));
    }
    let context = editor
        .as_ref()
        .map(|e| editor_context(e, cx))
        .unwrap_or_default();
    let seen = context.selection.clone();
    let plugin = id.clone();
    let task = cx.background_spawn(async move { shared.run(&plugin, run, context) });
    cx.spawn(async move |cx| {
        let report = task.await;
        cx.update(|cx| {
            if let Some(panel) = plugins_mut(cx).and_then(|p| p.panel.as_mut()) {
                panel.busy = false;
            }
            finish(&id, report, editor, seen, cx);
        });
    })
    .detach();
}

/// Brings a request's results to the windows.
fn finish(
    plugin_id: &str,
    report: Report,
    editor: Option<Entity<crate::editor::NoteEditor>>,
    seen: Option<router::Selection>,
    cx: &mut App,
) {
    let Some(plugins) = cx.try_global::<Plugins>() else {
        return;
    };
    let shared = plugins.shared.clone();
    let Some(installed) = shared.installed(plugin_id) else {
        return;
    };
    let name = installed.manifest.name.clone();
    let Report {
        result,
        outcome,
        disabled,
    } = report;
    if let Some(why) = disabled {
        let words = crate::plugins_model::status_line(
            &Installed {
                disabled: Some(why),
                ..installed.clone()
            },
            false,
            false,
        );
        let name = name.clone();
        notify_windows(
            move || {
                Notification::warning(format!("{name}: {words}.")).action(|_, _, _| {
                    Button::new("plugin-settings")
                        .label("Settings")
                        .on_click(|_, window, cx| {
                            window
                                .dispatch_action(Box::new(crate::settings::OpenPluginSettings), cx)
                        })
                })
            },
            cx,
        );
    }
    match &result {
        Ok(value) => {
            if let Some(text) = notice_text(value) {
                let text = format!("{name}: {text}");
                notify_windows(move || Notification::info(text.clone()), cx);
            }
        }
        Err(err) => {
            let text = format!("{name}: {err}");
            notify_windows(move || Notification::error(text.clone()), cx);
        }
    }
    if let (Some(edit), Some(editor), Some(seen)) = (outcome.selection_edit, &editor, seen) {
        let editor = editor.clone();
        let name = name.clone();
        cx.defer(move |cx| {
            let Some((handle, _)) = crate::workspace::main_workspace(cx) else {
                return;
            };
            let replaced = handle
                .update(cx, |_, window, cx| {
                    editor.update(cx, |editor, cx| {
                        editor.replace_for_plugin(
                            &edit.note_id,
                            edit.start..edit.end,
                            &seen.text,
                            &edit.text,
                            window,
                            cx,
                        )
                    })
                })
                .unwrap_or(false);
            if !replaced {
                let text = format!("{name} didn't change the selection: it changed while it ran.");
                notify_windows(move || Notification::warning(text.clone()), cx);
            }
        });
    }
    if let Some(root) = outcome.panel {
        let plugins = cx.global_mut::<Plugins>();
        let values = match &plugins.panel {
            Some(open) if open.panel.plugin_id == plugin_id => open.values.clone(),
            _ => Default::default(),
        };
        let values = crate::plugin_panel::initial_values(&root, &values);
        plugins.panel = Some(PanelState {
            panel: Panel {
                plugin_id: plugin_id.to_owned(),
                plugin_name: name.clone(),
                plugin_version: installed.manifest.version.clone(),
                root,
            },
            developer: installed.is_developer(),
            values,
            busy: false,
        });
        crate::plugin_panel::reveal(cx);
    }
    let batch = outcome.batch;
    if batch.is_empty() {
        cx.refresh_windows();
        return;
    }
    let open = editor
        .as_ref()
        .and_then(|e| e.read(cx).note_id().map(str::to_owned));
    if !batch.needs_review(open.as_deref(), installed.grant.ask_before_changing) {
        // Only the open note's text: an edit in the editor, so Undo takes
        // it back like typing.
        if let (Some(editor), [edit]) = (&editor, batch.edits.as_slice())
            && batch.creates.is_empty()
            && edit.title.is_none()
            && edit.tags.is_none()
            && edit.status.is_none()
            && edit.data.is_none()
            && let Some(body) = edit.body.clone()
        {
            let editor = editor.clone();
            let id = edit.id.clone();
            let plugin_id = plugin_id.to_owned();
            let current = shared.store.get(&id).ok().flatten().map(|doc| doc.rev);
            let unchanged = current.as_deref() == Some(edit.base_rev.as_str());
            cx.defer(move |cx| {
                let replaced = unchanged
                    && crate::workspace::main_workspace(cx).is_some_and(|(handle, _)| {
                        handle
                            .update(cx, |_, window, cx| {
                                editor.update(cx, |editor, cx| {
                                    editor.replace_body_for_plugin(&id, &body, window, cx)
                                })
                            })
                            .unwrap_or(false)
                    });
                // Otherwise through the library, which checks the note
                // against what the plugin read.
                if !replaced {
                    apply_batch(&plugin_id, &name, batch, cx);
                }
            });
            return;
        }
        apply_batch(plugin_id, &name, batch, cx);
        return;
    }
    let notes = review(&shared.store, plugin_id, &batch);
    cx.global_mut::<Plugins>().review = Some(PendingReview {
        plugin_id: plugin_id.to_owned(),
        plugin_name: name,
        batch,
        notes,
    });
    cx.defer(|cx| {
        if let Some((handle, _)) = crate::workspace::main_workspace(cx) {
            handle
                .update(cx, |_, window, cx| {
                    crate::plugin_sheets::open_review(window, cx)
                })
                .ok();
        }
    });
}

/// Applies `batch` for plugin `plugin_id` and offers Undo.
fn apply_batch(plugin_id: &str, name: &str, batch: Batch, cx: &mut App) {
    let shared = cx.global::<Plugins>().shared.clone();
    match shared.apply(plugin_id, &batch) {
        Ok(report) => {
            let count = report.changed.len() + report.created.len();
            cx.global_mut::<Plugins>().last_batch = Some((batch.id.clone(), name.to_owned()));
            let text = format!(
                "{name} changed {count} {}.",
                if count == 1 { "note" } else { "notes" }
            );
            let batch_id = batch.id.clone();
            notify_windows(
                move || {
                    let batch_id = batch_id.clone();
                    Notification::success(text.clone()).action(move |_, _, _| {
                        let batch_id = batch_id.clone();
                        Button::new("plugin-undo")
                            .label("Undo")
                            .on_click(move |_, _, cx| undo(&batch_id, cx))
                    })
                },
                cx,
            );
        }
        Err(message) => notify_windows(move || Notification::error(message.clone()), cx),
    }
    cx.refresh_windows();
}

/// Applies the batch on the review sheet.
pub fn apply_review(cx: &mut App) {
    let Some(review) = cx.global_mut::<Plugins>().review.take() else {
        return;
    };
    apply_batch(&review.plugin_id, &review.plugin_name, review.batch, cx);
}

/// Drops the batch on the review sheet: nothing changes.
pub fn cancel_review(cx: &mut App) {
    cx.global_mut::<Plugins>().review = None;
    cx.refresh_windows();
}

/// Takes back plugin batch `batch`.
pub fn undo(batch: &str, cx: &mut App) {
    let store = cx.global::<Plugins>().shared.store.clone();
    match store.undo_plugin_batch(batch, kanso_core::now_ms()) {
        Ok(report) => {
            let mut text = format!(
                "Put back {} {}.",
                report.restored.len(),
                if report.restored.len() == 1 {
                    "note"
                } else {
                    "notes"
                }
            );
            if !report.skipped.is_empty() {
                text.push_str(&format!(
                    " {} changed again since, so they were left as they are.",
                    report.skipped.len()
                ));
            }
            notify_windows(move || Notification::info(text.clone()), cx);
        }
        Err(err) => {
            let text = format!("Couldn't undo the plugin's changes: {err}");
            notify_windows(move || Notification::error(text.clone()), cx);
        }
    }
    let plugins = cx.global_mut::<Plugins>();
    if plugins.last_batch.as_ref().is_some_and(|(b, _)| b == batch) {
        plugins.last_batch = None;
    }
    cx.refresh_windows();
}

/// Closes the panel; its plugin's host stops once idle.
pub fn close_panel(cx: &mut App) {
    if let Some(plugins) = plugins_mut(cx) {
        plugins.panel = None;
    }
    cx.refresh_windows();
}

/// Keeps what the person put in panel input `id`.
pub fn set_panel_value(id: &str, value: Value, cx: &mut App) {
    if let Some(panel) = plugins_mut(cx).and_then(|p| p.panel.as_mut()) {
        panel.values.insert(id.to_owned(), value);
    }
}

// MARK: Installing

/// Shows the install sheet for `plan`.
fn offer_plan(plan: Result<InstallPlan, String>, window: &mut Window, cx: &mut App) {
    match plan {
        Ok(plan) => crate::plugin_sheets::open_install(plan, window, cx),
        Err(message) => window.push_notification(
            Notification::error(format!("Couldn't install the plugin: {message}")),
            cx,
        ),
    }
}

/// Opens a `.kansoplugin` file: installed only when the signed directory
/// lists its exact bytes.
pub fn open_file(path: PathBuf, window: &mut Window, cx: &mut App) {
    let shared = cx.global::<Plugins>().shared.clone();
    let plan = (|| {
        let meta = std::fs::metadata(&path).map_err(|e| e.to_string())?;
        if meta.len() > bundle::MAX_ARCHIVE_BYTES {
            return Err("the file is too large to be a plugin".to_owned());
        }
        let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
        let sha = bundle::sha256_hex(&bytes);
        let index = shared
            .index()
            .ok_or("Kanso can only install plugins the plugin directory lists, and it hasn't been reached yet")?;
        let entry = registry::find_by_hash(&index, &sha)
            .ok_or("this file isn't one the Kanso plugin directory lists. Only listed plugins, or folders in developer mode, can be installed")?
            .clone();
        let bundle = registry::verify_bundle(&bytes, &entry).map_err(|e| e.to_string())?;
        shared.plan(bundle, Source::File { sha256: sha })
    })();
    offer_plan(plan, window, cx);
}

/// Installs `entry` from the directory: fetched, checked against its
/// signatures, then the install sheet.
pub fn install_from_directory(entry: Entry, window: &mut Window, cx: &mut App) {
    let shared = cx.global::<Plugins>().shared.clone();
    let task = cx.background_spawn(async move {
        let agent = registry::agent();
        let bytes = registry::fetch_bundle(&agent, &entry).map_err(|e| e.to_string())?;
        let bundle = registry::verify_bundle(&bytes, &entry).map_err(|e| e.to_string())?;
        shared.plan(
            bundle,
            Source::Registry {
                sha256: entry.sha256.clone(),
            },
        )
    });
    let handle = window.window_handle();
    cx.spawn(async move |cx| {
        let plan = task.await;
        handle
            .update(cx, |_, window, cx| offer_plan(plan, window, cx))
            .ok();
    })
    .detach();
}

/// Offers an unpacked folder in developer mode.
pub fn add_developer_folder(path: PathBuf, window: &mut Window, cx: &mut App) {
    let shared = cx.global::<Plugins>().shared.clone();
    let plan = bundle::open_dir(&path)
        .map_err(|e| e.to_string())
        .and_then(|bundle| {
            shared.plan(
                bundle,
                Source::Developer {
                    path: path.to_string_lossy().into_owned(),
                },
            )
        });
    offer_plan(plan, window, cx);
}

/// Installs `plan` with `grant`, as the sheet confirmed, then runs its
/// script once to learn its commands.
pub fn confirm_install(plan: InstallPlan, grant: Grant, cx: &mut App) -> Result<(), String> {
    let shared = cx.global::<Plugins>().shared.clone();
    shared.install(&plan, grant)?;
    crate::metrics::record(cx, crate::metrics::Feature::PluginInstalled);
    let name = plan.bundle.manifest.name.clone();
    let id = plan.id().to_owned();
    let plugins = cx.global_mut::<Plugins>();
    plugins.offers.retain(|o| o.entry.id != id);
    let task = cx.background_spawn(async move { shared.introduce(&id) });
    cx.spawn(async move |cx| {
        let started = task.await;
        cx.update(|cx| {
            let text = match started {
                Ok(()) => format!("Installed {name}."),
                Err(why) => format!("Installed {name}, but it didn't start: {why}"),
            };
            notify_windows(move || Notification::success(text.clone()), cx);
            cx.refresh_windows();
        });
    })
    .detach();
    Ok(())
}

/// Removes plugin `id`, and with `sweep` its data from notes.
pub fn remove(id: &str, sweep: bool, cx: &mut App) {
    let plugins = cx.global_mut::<Plugins>();
    if plugins
        .panel
        .as_ref()
        .is_some_and(|p| p.panel.plugin_id == id)
    {
        plugins.panel = None;
    }
    let name = plugins
        .shared
        .installed(id)
        .map(|p| p.manifest.name)
        .unwrap_or_else(|| id.to_owned());
    let text = match plugins.shared.remove(id, sweep) {
        Ok(0) => format!("Removed {name}."),
        Ok(n) => format!(
            "Removed {name}, and its data from {n} {}.",
            if n == 1 { "note" } else { "notes" }
        ),
        Err(err) => format!("Couldn't remove {name}: {err}"),
    };
    notify_windows(move || Notification::info(text.clone()), cx);
    cx.refresh_windows();
}

/// Fetches the directory, when this build has its key.
pub fn refresh_directory(cx: &mut App) {
    if !registry::is_configured() {
        cx.global_mut::<Plugins>().directory = Directory::NotSetUp;
        return;
    }
    let plugins = cx.global_mut::<Plugins>();
    if matches!(plugins.directory, Directory::Loading) {
        return;
    }
    plugins.directory = Directory::Loading;
    let shared = plugins.shared.clone();
    let task = cx.background_spawn(async move {
        let agent = registry::agent();
        let (bytes, signature) = registry::fetch_index(&agent).map_err(|e| e.to_string())?;
        shared.take_index(&bytes, &signature)
    });
    cx.spawn(async move |cx| {
        let index = task.await;
        cx.update(|cx| {
            let plugins = cx.global_mut::<Plugins>();
            match index {
                Ok(index) => {
                    let app = app_version();
                    let state = plugins.shared.state();
                    plugins.offers = state
                        .plugins
                        .iter()
                        .filter_map(|p| registry::offer(p, &index, &app, &state.blocked))
                        .collect();
                    plugins.directory = Directory::Loaded(index);
                }
                Err(why) => plugins.directory = Directory::Failed(why),
            }
            cx.refresh_windows();
        });
    })
    .detach();
}

// MARK: The command line

const USAGE: &str = "\
Usage: kanso plugin <command>

Commands:
  dev <folder>      Offer an unpacked plugin folder to Kanso in developer
                    mode. Kanso asks you to type the folder's path and tick
                    \"I trust this code\" before it runs; it reloads the
                    folder whenever a file changes
  list              List the installed plugins
";

/// `kanso plugin …`.
pub fn run_cli(data_dir: &Path, rest: &[String]) -> anyhow::Result<()> {
    match rest.first().map(String::as_str) {
        Some("dev") => {
            let folder = rest.get(1).ok_or_else(|| anyhow::anyhow!("{USAGE}"))?;
            let folder = std::fs::canonicalize(folder)?;
            bundle::open_dir(&folder).map_err(|e| anyhow::anyhow!("{}: {e}", folder.display()))?;
            let dir = dev_requests_dir(data_dir);
            std::fs::create_dir_all(&dir)?;
            let name = bundle::sha256_hex(folder.to_string_lossy().as_bytes());
            install::write_private(
                &dir.join(format!("{}.json", &name[..16])),
                serde_json::to_string(&folder.to_string_lossy())?.as_bytes(),
            )?;
            println!(
                "Kanso will offer {} in developer mode the next time it is open.",
                folder.display()
            );
            Ok(())
        }
        Some("list") => {
            let _lock = kanso_store::lock_data_dir(data_dir)?
                .ok_or_else(|| anyhow::anyhow!("Kanso is open; see Settings › Plugins"))?;
            let store = Store::open(crate::paths::db_path(data_dir))?;
            for plugin in State::load(&store).plugins {
                println!(
                    "{}  {}  {}{}",
                    plugin.manifest.id,
                    plugin.manifest.version,
                    plugin.manifest.name,
                    if plugin.may_run() { "" } else { "  (off)" }
                );
            }
            Ok(())
        }
        Some("help") | Some("--help") | None => {
            println!("{USAGE}");
            Ok(())
        }
        Some(other) => anyhow::bail!("unknown plugin command `{other}`\n\n{USAGE}"),
    }
}

/// Plugin `id`'s manifest as the person sees it, for Settings: what it was
/// given, by permission.
pub fn installed(cx: &App) -> Vec<Installed> {
    cx.try_global::<Plugins>()
        .map(|p| p.shared.state().plugins)
        .unwrap_or_default()
}

/// Sets plugins up for a test or snapshot with `plugins` installed, in
/// memory secrets and no host. Nothing runs.
#[cfg(test)]
pub fn init_for_test(store: Arc<Store>, data_dir: &Path, plugins: Vec<Installed>, cx: &mut App) {
    let secrets: Arc<dyn SecretStore> = Arc::new(kanso_sync::keychain::MemorySecrets::default());
    let shared = Shared::new(store, data_dir, false, secrets);
    shared.update_state(|state| {
        for plugin in plugins {
            state.upsert(plugin);
        }
    });
    cx.set_global(Plugins {
        shared,
        review: None,
        panel: None,
        directory: Directory::NotSetUp,
        offers: Vec::new(),
        running: HashSet::new(),
        last_batch: None,
        dev_requests: Vec::new(),
        _tick: None,
    });
}

/// Adds a line to plugin `id`'s log, for a test or snapshot.
#[cfg(test)]
pub fn push_log_for_test(id: &str, level: &str, text: &str, cx: &App) {
    if let Some(plugins) = cx.try_global::<Plugins>() {
        let slot = plugins.shared.slot(id);
        slot.lock()
            .expect("plugin slot poisoned")
            .log
            .push(level, text);
    }
}

/// Turns safe mode on or off for a test or snapshot.
#[cfg(test)]
pub fn set_safe_mode_for_test(on: bool, cx: &mut App) {
    if let Some(plugins) = cx.try_global::<Plugins>()
        && let Ok(mut gate) = plugins.shared.gate.write()
    {
        gate.safe_mode = on;
    }
}

/// The live directory, through this service, as Kanso uses it: install a
/// sample, take its update, then find it withdrawn. Needs the network and a
/// persistent `KANSO_LIVE_DIR`; `KANSO_LIVE_STEP` is `install`, `update` or
/// `revoked`, run in that order as the directory publishes each change.
#[cfg(test)]
mod live {
    use std::sync::Arc;

    use kanso_core::DocType;
    use kanso_plugins::model::{Disabled, Source};
    use kanso_plugins::registry;
    use kanso_plugins::router::Editor;
    use kanso_store::{Origin, Put, Store};
    use serde_json::{Value, json};

    use super::{Run, Shared, app_version};

    const WORD_COUNT: &str = "app.kanso.sample.wordcount";

    fn fetch(shared: &Arc<Shared>) -> kanso_plugins::registry_model::Index {
        let agent = registry::agent();
        let (bytes, signature) = registry::fetch_index(&agent).expect("the directory answers");
        shared
            .take_index(&bytes, &signature)
            .expect("the directory verifies")
    }

    fn install(shared: &Arc<Shared>, entry: &kanso_plugins::registry_model::Entry) {
        let bytes =
            registry::fetch_bundle(&registry::agent(), entry).expect("the bundle downloads");
        let bundle = registry::verify_bundle(&bytes, entry).expect("the bundle verifies");
        let plan = shared
            .plan(
                bundle,
                Source::Registry {
                    sha256: entry.sha256.clone(),
                },
            )
            .expect("Kanso offers to install it");
        assert!(!plan.key_changed(), "the author's key is the pinned one");
        let grant = plan.grant.clone();
        shared.install(&plan, grant).expect("it installs");
        shared.introduce(&entry.id).expect("its host starts");
    }

    fn count_words(shared: &Arc<Shared>, store: &Store) -> Value {
        let book = store
            .put(
                DocType::Book,
                json!({ "name": "Drafts" }),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        let note = store
            .put(
                DocType::Note,
                json!({ "bookId": book, "title": "Draft", "body": "one two three" }),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        let editor = Editor {
            note_id: Some(note),
            selection: None,
        };
        let run = Run::Command {
            id: "count-words".into(),
            args: Value::Null,
        };
        shared
            .run(WORD_COUNT, run, editor)
            .result
            .expect("the command runs")
    }

    #[test]
    #[ignore = "uses the live plugin directory; see the module doc"]
    fn the_live_directory() {
        let dir =
            std::path::PathBuf::from(std::env::var("KANSO_LIVE_DIR").expect("KANSO_LIVE_DIR"));
        let step = std::env::var("KANSO_LIVE_STEP").expect("KANSO_LIVE_STEP");
        std::fs::create_dir_all(&dir).unwrap();
        let store = Arc::new(Store::open(dir.join("kanso.db")).unwrap());
        let secrets: Arc<dyn kanso_sync::keychain::SecretStore> =
            Arc::new(kanso_sync::keychain::MemorySecrets::default());
        let shared = Shared::new(store.clone(), &dir, false, secrets);
        assert!(
            registry::is_configured(),
            "this build trusts the directory key"
        );
        let index = fetch(&shared);
        eprintln!("directory serial {}", index.serial);
        match step.as_str() {
            "install" => {
                let entry = registry::latest(&index, WORD_COUNT).expect("Word count is listed");
                install(&shared, entry);
                assert_eq!(count_words(&shared, &store)["text"], "3 words");
                eprintln!("installed {} and ran it", entry.version);
            }
            "update" => {
                let installed = shared
                    .installed(WORD_COUNT)
                    .expect("installed at the first step");
                let offer =
                    registry::offer(&installed, &index, &app_version(), &Default::default())
                        .expect("the directory offers an update");
                assert!(!offer.key_changed);
                eprintln!(
                    "update offered: {} -> {}",
                    installed.manifest.version, offer.entry.version
                );
                install(&shared, &offer.entry);
                let now = shared.installed(WORD_COUNT).unwrap();
                assert_eq!(now.manifest.version, offer.entry.version);
                assert_eq!(count_words(&shared, &store)["text"], "3 words");
            }
            "revoked" => {
                let installed = shared
                    .installed(WORD_COUNT)
                    .expect("installed at the earlier steps");
                assert!(
                    matches!(installed.disabled, Some(Disabled::Revoked { .. })),
                    "the withdrawn version is off: {:?}",
                    installed.disabled
                );
                assert!(!shared.may_run(WORD_COUNT));
                let run = Run::Command {
                    id: "count-words".into(),
                    args: Value::Null,
                };
                assert!(
                    shared
                        .run(WORD_COUNT, run, Editor::default())
                        .result
                        .is_err()
                );
                eprintln!(
                    "{} {} is withdrawn: {:?}",
                    WORD_COUNT, installed.manifest.version, installed.disabled
                );
            }
            other => panic!("unknown step {other}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use proptest::prelude::*;
    use serde_json::json;

    use super::{MAX_NOTICE, notice_text, secret_item, take_dev_requests};

    #[test]
    fn notices_show_text_and_nothing_else() {
        assert_eq!(
            notice_text(&json!({ "text": "4 words" })).as_deref(),
            Some("4 words")
        );
        assert_eq!(notice_text(&json!("hi")).as_deref(), Some("hi"));
        assert_eq!(notice_text(&json!({ "other": 1 })), None);
        assert_eq!(notice_text(&json!(null)), None);
        assert_eq!(notice_text(&json!("   ")), None);
    }

    #[test]
    fn secrets_are_named_per_plugin() {
        assert_ne!(
            secret_item("com.a.b", "token"),
            secret_item("com.a.c", "token")
        );
        assert!(secret_item("com.a.b", "token").starts_with("Kanso Plugin com.a.b "));
    }

    #[test]
    fn dev_requests_are_taken_once() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.json"), "\"/tmp/plugin\"").unwrap();
        std::fs::write(dir.path().join("b.json"), "not json").unwrap();
        assert_eq!(
            take_dev_requests(dir.path()),
            vec![PathBuf::from("/tmp/plugin")]
        );
        assert!(take_dev_requests(dir.path()).is_empty());
    }

    proptest! {
        /// Invariant: a notice is never longer than the cap (and its
        /// ellipsis), and is the answer's text when it fits.
        #[test]
        fn prop_notices_are_bounded(text in "\\PC{0,600}") {
            match notice_text(&json!({ "text": text })) {
                Some(shown) => {
                    prop_assert!(shown.chars().count() <= MAX_NOTICE + 1);
                    if text.trim().chars().count() <= MAX_NOTICE {
                        prop_assert_eq!(shown, text.trim());
                    }
                }
                None => prop_assert!(text.trim().is_empty()),
            }
        }
    }
}
