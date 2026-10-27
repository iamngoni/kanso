//! `config.json` in the data directory. It holds the API credentials, so it is
//! written with owner-only permissions and never logged.

use std::fmt;
use std::path::Path;

use serde::{Deserialize, Serialize};

pub use kanso_engine::editor_settings::{
    DEFAULT_SPLIT_RATIO, EditorFont, EditorSettings, Typewriter, ViewMode,
};

pub const DEFAULT_PORT: u16 = 19840;

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ServerSettings {
    pub enabled: bool,
    pub port: u16,
    pub bind_address: String,
    pub username: String,
    pub password: String,
}

impl Default for ServerSettings {
    fn default() -> Self {
        ServerSettings {
            enabled: true,
            port: crate::flavour::CURRENT.default_port,
            bind_address: "127.0.0.1".into(),
            username: String::new(),
            password: String::new(),
        }
    }
}

impl fmt::Debug for ServerSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ServerSettings")
            .field("enabled", &self.enabled)
            .field("port", &self.port)
            .field("bind_address", &self.bind_address)
            .field("username", &self.username)
            .field("password", &"<redacted>")
            .finish()
    }
}

/// Sync with an account (docs/ACCOUNTS.md › Config). The session itself is
/// in the Keychain; this only says which account the library is signed in
/// to, and where the forms point.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SyncSettings {
    /// Set on sign-up and sign-in; sync only runs while signed in too.
    pub enabled: bool,
    pub interval_secs: u64,
    /// The server the signed-out forms start with; `None` is
    /// [`kanso_sync::DEFAULT_SERVER`]. A signed-in session's server comes
    /// from its Keychain item, never from here.
    pub server: Option<String>,
    /// Some while signed in.
    pub account: Option<AccountRef>,
    /// The version 1 Worker URL, kept untouched so a rolled-back app finds
    /// it (docs/ACCOUNTS.md › Rollback).
    pub endpoint: Option<String>,
}

impl Default for SyncSettings {
    fn default() -> Self {
        SyncSettings {
            enabled: false,
            interval_secs: 300,
            server: None,
            account: None,
            endpoint: None,
        }
    }
}

/// The account a library is signed in to. With the Keychain item and the
/// library's binding, `(origin, id)` says whether two accounts are the same
/// (security re-check 3).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountRef {
    pub origin: String,
    pub id: String,
    pub username: String,
}

/// Light, dark or the system's: the only theme choice before version 3, still
/// written beside [`ThemeSettings`] so an older Kanso keeps the light or dark
/// it last showed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ThemePreference {
    #[default]
    System,
    Light,
    Dark,
}

/// Kanso's own themes, the ones older files and a missing theme fall back to.
pub const KANSO_LIGHT: &str = "Kanso Light";
pub const KANSO_DARK: &str = "Kanso Dark";

/// Which themes color the app, by name: one for the UI, and the editor's syntax and the preview each the UI's own unless one
/// is picked for them. Following the system switches the UI between a light
/// and a dark theme.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ThemeSettings {
    /// The UI theme while the system isn't followed.
    pub ui: String,
    /// `None`: the UI theme's.
    pub syntax: Option<String>,
    pub preview: Option<String>,
    pub follow_system: bool,
    /// The UI themes for light and dark mode while the system is followed.
    pub light: String,
    pub dark: String,
}

impl Default for ThemeSettings {
    fn default() -> Self {
        ThemeSettings::from_preference(ThemePreference::System)
    }
}

impl ThemeSettings {
    /// The settings an older file's light-or-dark choice means: Kanso's own
    /// light and dark themes, following the system or fixed.
    pub fn from_preference(preference: ThemePreference) -> ThemeSettings {
        ThemeSettings {
            ui: match preference {
                ThemePreference::Light => KANSO_LIGHT,
                ThemePreference::System | ThemePreference::Dark => KANSO_DARK,
            }
            .to_owned(),
            syntax: None,
            preview: None,
            follow_system: preference == ThemePreference::System,
            light: KANSO_LIGHT.to_owned(),
            dark: KANSO_DARK.to_owned(),
        }
    }

    /// The UI theme to show while the system is dark or light.
    pub fn ui_for(&self, system_dark: bool) -> &str {
        match (self.follow_system, system_dark) {
            (true, true) => &self.dark,
            (true, false) => &self.light,
            (false, _) => &self.ui,
        }
    }

    pub fn syntax_for(&self, system_dark: bool) -> &str {
        self.syntax
            .as_deref()
            .unwrap_or_else(|| self.ui_for(system_dark))
    }

    pub fn preview_for(&self, system_dark: bool) -> &str {
        self.preview
            .as_deref()
            .unwrap_or_else(|| self.ui_for(system_dark))
    }
}

/// What draws Mermaid diagrams in the preview and in exports.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DiagramRenderer {
    /// `mermaid-svg`, Kanso's own.
    Kanso,
    /// Pele, Obsidian's renderer: the default.
    #[default]
    Pele,
}

impl DiagramRenderer {
    pub fn engine(self) -> kanso_render::MermaidEngine {
        match self {
            DiagramRenderer::Kanso => kanso_render::MermaidEngine::MermaidSvg,
            DiagramRenderer::Pele => kanso_render::MermaidEngine::Pele,
        }
    }
}

/// The editor text size before version 1, kept by anyone who never changed it.
const OLD_DEFAULT_FONT_SIZE: f32 = 14.0;
/// The text size when settings don't say; a page's (`kanso_engine::export`)
/// is the same.
pub const DEFAULT_FONT_SIZE: f32 = kanso_engine::export::PageStyle::DEFAULT_SIZE;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Appearance {
    /// Kept for older versions; [`Self::themes`] is the choice.
    pub theme: ThemePreference,
    pub themes: ThemeSettings,
    pub editor_font: EditorFont,
    pub editor_font_size: f32,
    /// The installed font the editor writes prose in, and the one for its
    /// monospace face; `None` for Kanso's own.
    pub editor_font_family: Option<String>,
    pub editor_mono_family: Option<String>,
    /// The installed font the preview sets text in (`None` for Kanso's
    /// own), and its size.
    pub preview_font_family: Option<String>,
    pub preview_font_size: f32,
    /// Whether monospace text shows line numbers beside it.
    pub line_numbers: bool,
    /// Whether text keeps to a readable measure, or fills the pane.
    pub readable_width: bool,
    pub view_mode: ViewMode,
    /// The full sidebar, or (false) the icon rail it collapses to.
    pub show_sidebar: bool,
    /// The icon theme drawing the app's icons, by its folder's name;
    /// `None` for Kanso's own.
    pub icon_theme: Option<String>,
    /// The icon pack of versions before 4, which [`AppConfig::migrate`]
    /// makes the icon theme; never written.
    #[serde(rename = "iconPack", skip_serializing)]
    pub legacy_icon_pack: Option<String>,
    /// How large the whole app is drawn, in percent; see [`crate::zoom`].
    pub zoom: u32,
    pub diagram_renderer: DiagramRenderer,
}

impl Default for Appearance {
    fn default() -> Self {
        Appearance {
            theme: ThemePreference::System,
            themes: ThemeSettings::default(),
            editor_font: EditorFont::Auto,
            editor_font_size: DEFAULT_FONT_SIZE,
            editor_font_family: None,
            editor_mono_family: None,
            preview_font_family: None,
            preview_font_size: DEFAULT_FONT_SIZE,
            line_numbers: true,
            readable_width: false,
            view_mode: ViewMode::Split,
            show_sidebar: true,
            icon_theme: None,
            legacy_icon_pack: None,
            zoom: crate::zoom::DEFAULT_ZOOM,
            diagram_renderer: DiagramRenderer::Pele,
        }
    }
}

pub use kanso_engine::account::is_default_server;

/// The owner's Worker before it moved to its own domain; v1 files that name
/// it migrate to the default, like the default itself. It no longer answers.
const OLD_OWNER_SERVER: &str = "https://kanso-sync.imngonii.workers.dev";

/// Whether a v1 `endpoint` means "the owner's server": the default, or its
/// old workers.dev address.
fn is_owner_endpoint(endpoint: &str) -> bool {
    is_default_server(endpoint)
        || kanso_sync::check_endpoint(endpoint).as_deref() == Ok(OLD_OWNER_SERVER)
}

/// The inspector's tabs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InspectorTab {
    #[default]
    Outline,
    Backlinks,
    /// The notes around the open one, as a graph.
    Graph,
    /// Notes about the same things (`related.rs`).
    Related,
    /// A shared note's comments (`inspector_comments.rs`).
    Comments,
    /// Ask Your Notes: a question answered from the notes (`ask.rs`).
    Ask,
}

/// How the window's panes were left: focus mode, where the split divides,
/// whether the sidebar's Status group is open, and the inspector.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct WindowLayout {
    pub focus_mode: bool,
    /// The source's share of the editor in Split mode, between 0 and 1.
    pub split_ratio: f32,
    /// Open or closed by the user; `None` until they choose.
    pub status_open: Option<bool>,
    /// The outline and backlinks pane beside the editor.
    pub inspector_open: bool,
    pub inspector_width: f32,
    pub inspector_tab: InspectorTab,
    /// The sidebar's notebooks opened to show the ones inside them, by ID;
    /// the rest are folded.
    pub open_books: Vec<String>,
    /// The notes open in windows of their own (`note_window.rs`), to open
    /// again at the next launch where they were.
    pub note_windows: Vec<NoteWindowFrame>,
    /// The notes open as tabs in the main window (`tabs`), in the order
    /// they show, and the one showing.
    pub open_tabs: Vec<String>,
    pub active_tab: usize,
}

/// A note's own window: which note, and where the window was on screen, in
/// points from the top left of the main display.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteWindowFrame {
    pub note_id: String,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Default for WindowLayout {
    fn default() -> Self {
        WindowLayout {
            focus_mode: false,
            split_ratio: DEFAULT_SPLIT_RATIO,
            status_open: None,
            inspector_open: false,
            inspector_width: crate::layout::INSPECTOR_WIDTH,
            inspector_tab: InspectorTab::Outline,
            open_books: Vec::new(),
            note_windows: Vec::new(),
            open_tabs: Vec::new(),
            active_tab: 0,
        }
    }
}

/// Updates from GitHub releases (docs/RELEASING.md › How updates work).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct UpdateSettings {
    /// Download a new release in the background and install it on the
    /// next restart or quit. Off, Kanso only says one is out.
    pub auto_install: bool,
}

impl Default for UpdateSettings {
    fn default() -> Self {
        UpdateSettings { auto_install: true }
    }
}

/// The theme gallery (Settings › Themes).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct GallerySettings {
    /// Check the signed catalog on the app's update timer and update
    /// installed gallery themes. Off, nothing is checked behind a person's
    /// back; Refresh still works.
    pub auto_update: bool,
}

impl Default for GallerySettings {
    fn default() -> Self {
        GallerySettings { auto_update: true }
    }
}

/// The quick capture shortcut, which works from any app (Settings ›
/// General).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct CaptureSettings {
    pub enabled: bool,
    /// As GPUI writes keystrokes: `ctrl-alt-space` (see `hotkey`).
    pub shortcut: String,
}

/// ⌃⌥Space.
pub const DEFAULT_CAPTURE_SHORTCUT: &str = "ctrl-alt-space";

impl Default for CaptureSettings {
    fn default() -> Self {
        CaptureSettings {
            enabled: true,
            shortcut: DEFAULT_CAPTURE_SHORTCUT.to_owned(),
        }
    }
}

/// Notifications for tasks with a due time (Settings › General).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct RemindersSettings {
    pub enabled: bool,
}

impl Default for RemindersSettings {
    fn default() -> Self {
        RemindersSettings { enabled: true }
    }
}

/// Finding notes by their connections and their words (Settings ›
/// General): related notes, text in images, the inspector graph's reach.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct DiscoverySettings {
    /// Related notes: off until turned on, as it downloads a model.
    pub related_notes: bool,
    /// Reading the words in image attachments, where the system can
    /// (macOS): on by default there.
    pub image_text: bool,
    /// How many links out the inspector's graph reaches: 1 or 2.
    pub graph_hops: u8,
}

impl Default for DiscoverySettings {
    fn default() -> Self {
        DiscoverySettings {
            related_notes: false,
            image_text: cfg!(target_os = "macos"),
            graph_hops: 1,
        }
    }
}

/// How the editor's mic turns speech into text (`voice.rs`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum VoiceEngine {
    /// Not chosen yet: the mic's first press asks, as the on-device model
    /// is a large download.
    #[default]
    Ask,
    /// Parakeet, on this device, once its model is downloaded.
    OnDevice,
    /// The system's own dictation (macOS dictation, Windows voice typing).
    System,
}

/// Voice input (Settings › Editor).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct VoiceSettings {
    pub engine: VoiceEngine,
    /// Whether "comma", "new line", "delete that" and the like, said on
    /// their own, act instead of going in as words (`voice_commands.rs`).
    pub spoken_commands: bool,
}

impl Default for VoiceSettings {
    fn default() -> Self {
        VoiceSettings {
            engine: VoiceEngine::default(),
            spoken_commands: true,
        }
    }
}

/// Where answers come from: a provider reached with an API key, or an agent
/// on this computer that keeps its own sign-in and billing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AiBackend {
    #[default]
    Api,
    Agent,
}

/// The agent that answers when [`AiBackend::Agent`] is chosen. Known agents
/// are found at run time; only a custom agent's command is kept. Nothing
/// here is secret: no token or running scope is ever written.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AgentChoice {
    pub kind: kanso_ai::AgentKind,
    /// A custom agent's executable: a path or a command name, never a shell
    /// string.
    pub command: Option<String>,
    pub args: Vec<String>,
    /// A custom agent's own variables.
    pub env: std::collections::BTreeMap<String, String>,
    /// Whether the agent may search and read the notes itself, through
    /// Kanso's read-only notes server. On unless turned off.
    pub mcp: bool,
    /// The notebooks it may read there, by ID, with the ones inside them;
    /// empty is every unlocked notebook. Names aren't kept: they change.
    pub notebooks: Vec<String>,
    /// Whether the agent may suggest edits to the notes it may read. Off
    /// unless turned on. A suggestion changes nothing: it waits in the review
    /// queue until the person applies it. Needs `mcp`.
    pub suggest_edits: bool,
    /// The ACP registry entry a custom agent was picked from, by its ID.
    pub registry: Option<String>,
}

impl Default for AgentChoice {
    fn default() -> Self {
        AgentChoice {
            kind: kanso_ai::AgentKind::default(),
            command: None,
            args: Vec::new(),
            env: Default::default(),
            mcp: true,
            notebooks: Vec::new(),
            suggest_edits: false,
            registry: None,
        }
    }
}

/// AI in Settings › AI. Off until turned on: nothing asks a model or shows a
/// question box before then. The API key is not here; it lives in the
/// Keychain (`settings_ai.rs`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AiSettings {
    pub enabled: bool,
    /// An API key or an agent. Choosing an agent leaves the provider, its
    /// address, model and key as they were, so switching back loses nothing.
    pub backend: AiBackend,
    pub provider: kanso_ai::Provider,
    /// The provider's address; local and custom servers can change it.
    pub base_url: String,
    pub model: String,
    pub agent: AgentChoice,
}

impl Default for AiSettings {
    fn default() -> Self {
        AiSettings::for_provider(kanso_ai::Provider::default())
    }
}

impl AiSettings {
    /// The settings a provider starts with: its own address and model.
    pub fn for_provider(provider: kanso_ai::Provider) -> AiSettings {
        AiSettings {
            enabled: false,
            backend: AiBackend::Api,
            provider,
            base_url: provider.default_base_url().to_owned(),
            model: provider.default_model().to_owned(),
            agent: AgentChoice::default(),
        }
    }

    /// Picks `provider`, with the address and model it starts with.
    pub fn choose(&mut self, provider: kanso_ai::Provider) {
        self.provider = provider;
        self.base_url = provider.default_base_url().to_owned();
        self.model = provider.default_model().to_owned();
    }
}

/// Locked notes on this device.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct LockedSettings {
    /// Whether Touch ID (Windows Hello on Windows) unlocks them here: `None`
    /// until Kanso has offered it, so it offers once.
    pub touch_id: Option<bool>,
}

/// The layout [`AppConfig::migrate`] brings older files up to.
pub const CONFIG_VERSION: u32 = 5;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct AppConfig {
    /// Files written before versions existed read as 0.
    #[serde(default)]
    pub version: u32,
    pub server: ServerSettings,
    pub sync: SyncSettings,
    pub appearance: Appearance,
    pub launch_at_login: bool,
    pub last_note: Option<String>,
    pub layout: WindowLayout,
    pub updates: UpdateSettings,
    pub gallery: GallerySettings,
    pub editor: EditorSettings,
    pub capture: CaptureSettings,
    pub reminders: RemindersSettings,
    pub discovery: DiscoverySettings,
    pub voice: VoiceSettings,
    pub locked: LockedSettings,
    pub ai: AiSettings,
    pub nudges: Nudges,
}

/// What Kanso has told someone about, so it says it once.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Nudges {
    /// When Kanso was first opened with this config (ms), for the sync offer.
    pub first_seen_ms: Option<i64>,
    /// The note list's sync offer was put away.
    pub sync_offer_dismissed: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            version: CONFIG_VERSION,
            server: ServerSettings::default(),
            sync: SyncSettings::default(),
            appearance: Appearance::default(),
            launch_at_login: false,
            last_note: None,
            layout: WindowLayout::default(),
            updates: UpdateSettings::default(),
            gallery: GallerySettings::default(),
            editor: EditorSettings::default(),
            capture: CaptureSettings::default(),
            reminders: RemindersSettings::default(),
            discovery: DiscoverySettings::default(),
            voice: VoiceSettings::default(),
            locked: LockedSettings::default(),
            ai: AiSettings::default(),
            nudges: Nudges::default(),
        }
    }
}

impl AppConfig {
    /// Brings a config from an older version up to [`CONFIG_VERSION`], and
    /// says whether anything changed. Version 1 raised the default editor
    /// size from 14 to 16; a size someone chose stays. Version 2 brought
    /// accounts: a Worker URL that isn't the default becomes the forms'
    /// server, and the URL itself, `enabled` and the (absent) account stay
    /// as they were, so the Mac starts signed out with its notes in place.
    /// Version 3 brought themes: light, dark or system becomes Kanso's own
    /// light and dark themes, fixed or following the system. Version 4
    /// replaced icon packs with icon themes: a pack chosen for the app's
    /// icons becomes the icon theme of the same folder name, which draws
    /// Kanso's own icons until there is one. Version 5 added where AI
    /// answers come from; an older file means an API key, as before.
    pub fn migrate(&mut self) -> bool {
        if self.version >= CONFIG_VERSION {
            return false;
        }
        if self.version < 1 && self.appearance.editor_font_size == OLD_DEFAULT_FONT_SIZE {
            self.appearance.editor_font_size = DEFAULT_FONT_SIZE;
        }
        if self.version < 2 && self.sync.server.is_none() {
            self.sync.server = self
                .sync
                .endpoint
                .clone()
                .filter(|endpoint| !is_owner_endpoint(endpoint));
        }
        if self.version < 3 {
            self.appearance.themes = ThemeSettings::from_preference(self.appearance.theme);
        }
        if let Some(pack) = self.appearance.legacy_icon_pack.take()
            && self.version < 4
            && self.appearance.icon_theme.is_none()
        {
            self.appearance.icon_theme = Some(pack);
        }
        self.version = CONFIG_VERSION;
        true
    }

    /// Reads the config, falling back to defaults for a missing file or field.
    pub fn load(path: &Path) -> anyhow::Result<AppConfig> {
        match std::fs::read(path) {
            Ok(bytes) => Ok(serde_json::from_slice(&bytes)?),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(AppConfig::default()),
            Err(err) => Err(err.into()),
        }
    }

    /// Writes the config atomically with owner-only permissions (on
    /// Windows, the data folder's own: the user's profile is theirs alone).
    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        use std::io::Write as _;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let tmp = path.with_extension("json.tmp");
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create(true).truncate(true);
        #[cfg(unix)]
        std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
        let mut file = options.open(&tmp)?;
        // `mode` only applies to a new file; a leftover one keeps its own.
        #[cfg(unix)]
        file.set_permissions(std::os::unix::fs::PermissionsExt::from_mode(0o600))?;
        file.write_all(&serde_json::to_vec_pretty(self)?)?;
        file.sync_all()?;
        std::fs::rename(&tmp, path)?;
        Ok(())
    }

    /// Fills in API credentials on first run so the server never runs open.
    pub fn ensure_credentials(&mut self) -> bool {
        let mut changed = false;
        if self.server.username.is_empty() {
            self.server.username = "kanso".into();
            changed = true;
        }
        if self.server.password.is_empty() {
            self.server.password = (0..24).map(|_| fastrand::alphanumeric()).collect();
            changed = true;
        }
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    /// Any config, with the window layout too.
    fn gen_config() -> impl Strategy<Value = AppConfig> {
        (
            gen_settings(),
            gen_layout(),
            any::<bool>(),
            gen_editor(),
            (any::<bool>(), "(ctrl-|alt-|shift-|cmd-){1,3}[a-z0-9]"),
            (any::<bool>(), any::<bool>(), 1u8..=2),
            (
                prop::sample::select(vec![
                    VoiceEngine::Ask,
                    VoiceEngine::OnDevice,
                    VoiceEngine::System,
                ]),
                any::<bool>(),
            ),
            gen_ai(),
        )
            .prop_map(
                |(
                    config,
                    layout,
                    auto_install,
                    editor,
                    (enabled, shortcut),
                    (related_notes, image_text, graph_hops),
                    (engine, spoken_commands),
                    ai,
                )| AppConfig {
                    voice: VoiceSettings {
                        engine,
                        spoken_commands,
                    },
                    ai,
                    layout,
                    updates: UpdateSettings { auto_install },
                    gallery: GallerySettings::default(),
                    editor,
                    capture: CaptureSettings { enabled, shortcut },
                    discovery: DiscoverySettings {
                        related_notes,
                        image_text,
                        graph_hops,
                    },
                    ..config
                },
            )
    }

    fn gen_editor() -> impl Strategy<Value = EditorSettings> {
        (
            any::<bool>(),
            any::<bool>(),
            any::<bool>(),
            prop::sample::select(vec![
                Typewriter::Off,
                Typewriter::FocusMode,
                Typewriter::Always,
            ]),
            prop::option::of(1u32..100_000),
            any::<bool>(),
            any::<bool>(),
            any::<bool>(),
        )
            .prop_map(
                |(
                    auto_pair,
                    format_tables,
                    smart_paste,
                    typewriter,
                    word_goal,
                    inline_tags,
                    vim,
                    spellcheck,
                )| {
                    EditorSettings {
                        auto_pair,
                        format_tables,
                        smart_paste,
                        typewriter,
                        word_goal,
                        inline_tags,
                        vim,
                        spellcheck,
                    }
                },
            )
    }

    fn gen_layout() -> impl Strategy<Value = WindowLayout> {
        (
            any::<bool>(),
            0.0f32..=1.0,
            prop::option::of(any::<bool>()),
            any::<bool>(),
            crate::layout::INSPECTOR_RANGE.0..=crate::layout::INSPECTOR_RANGE.1,
            prop::sample::select(vec![
                InspectorTab::Outline,
                InspectorTab::Backlinks,
                InspectorTab::Graph,
                InspectorTab::Related,
                InspectorTab::Comments,
                InspectorTab::Ask,
            ]),
            prop::collection::vec(
                (
                    "note:[a-z0-9]{1,8}",
                    -4000.0f32..4000.,
                    -4000.0f32..4000.,
                    0.0f32..4000.,
                    0.0f32..4000.,
                ),
                0..3,
            ),
            prop::collection::vec("note:[a-z0-9]{1,8}", 0..4),
            0..6usize,
        )
            .prop_map(
                |(
                    focus_mode,
                    split_ratio,
                    status_open,
                    inspector_open,
                    inspector_width,
                    inspector_tab,
                    note_windows,
                    open_tabs,
                    active_tab,
                )| {
                    WindowLayout {
                        focus_mode,
                        split_ratio,
                        status_open,
                        inspector_open,
                        inspector_width,
                        inspector_tab,
                        open_books: Vec::new(),
                        note_windows: note_windows
                            .into_iter()
                            .map(|(note_id, x, y, width, height)| NoteWindowFrame {
                                note_id,
                                x,
                                y,
                                width,
                                height,
                            })
                            .collect(),
                        open_tabs,
                        active_tab,
                    }
                },
            )
    }

    fn gen_agent() -> impl Strategy<Value = AgentChoice> {
        (
            prop::sample::select(kanso_ai::AgentKind::ALL.to_vec()),
            prop::option::of("[a-zA-Z0-9._/ -]{1,30}"),
            prop::collection::vec("[ -~]{0,12}", 0..4),
            prop::collection::btree_map("[A-Z_]{1,10}", "[ -~]{0,12}", 0..3),
            any::<bool>(),
            prop::collection::vec("book:[a-zA-Z0-9]{8}", 0..3),
            prop::option::of("[a-z-]{1,12}"),
            any::<bool>(),
        )
            .prop_map(
                |(kind, command, args, env, mcp, notebooks, registry, suggest_edits)| AgentChoice {
                    kind,
                    command,
                    args,
                    env,
                    mcp,
                    notebooks,
                    suggest_edits,
                    registry,
                },
            )
    }

    fn gen_ai() -> impl Strategy<Value = AiSettings> {
        (
            any::<bool>(),
            prop::sample::select(vec![AiBackend::Api, AiBackend::Agent]),
            prop::sample::select(kanso_ai::Provider::ALL.to_vec()),
            "https?://[a-z.:0-9/-]{1,40}",
            "[a-zA-Z0-9._/-]{0,40}",
            gen_agent(),
        )
            .prop_map(
                |(enabled, backend, provider, base_url, model, agent)| AiSettings {
                    enabled,
                    backend,
                    provider,
                    base_url,
                    model,
                    agent,
                },
            )
    }

    fn gen_settings() -> impl Strategy<Value = AppConfig> {
        (
            any::<bool>(),
            any::<u16>(),
            "[a-z0-9.:]{0,15}",
            "\\PC{0,12}",
            "\\PC{0,24}",
            gen_sync(),
            (
                prop::sample::select(vec![
                    ThemePreference::System,
                    ThemePreference::Light,
                    ThemePreference::Dark,
                ]),
                gen_font(),
                prop_oneof![
                    Just(OLD_DEFAULT_FONT_SIZE),
                    Just(DEFAULT_FONT_SIZE),
                    8.0f32..32.0
                ],
                any::<bool>(),
                gen_mode(),
                any::<bool>(),
                any::<bool>(),
            ),
            any::<bool>(),
            prop::option::of("note:[a-zA-Z0-9]{8}"),
            0u32..=CONFIG_VERSION + 1,
            (
                gen_themes(),
                gen_fonts(),
                gen_zoom(),
                prop::sample::select(vec![DiagramRenderer::Kanso, DiagramRenderer::Pele]),
            ),
        )
            .prop_map(
                |(
                    enabled,
                    port,
                    bind,
                    user,
                    pass,
                    sync,
                    (theme, font, size, numbers, mode, sidebar, readable),
                    login,
                    last,
                    version,
                    (themes, (writing, mono, preview, preview_size), zoom, diagrams),
                )| AppConfig {
                    version,
                    server: ServerSettings {
                        enabled,
                        port,
                        bind_address: bind,
                        username: user,
                        password: pass,
                    },
                    sync,
                    appearance: Appearance {
                        theme,
                        themes,
                        editor_font: font,
                        editor_font_size: size,
                        editor_font_family: writing,
                        editor_mono_family: mono,
                        preview_font_family: preview,
                        preview_font_size: preview_size,
                        line_numbers: numbers,
                        readable_width: readable,
                        view_mode: mode,
                        show_sidebar: sidebar,
                        icon_theme: None,
                        legacy_icon_pack: None,
                        zoom,
                        diagram_renderer: diagrams,
                    },
                    launch_at_login: login,
                    last_note: last,
                    layout: WindowLayout::default(),
                    updates: UpdateSettings::default(),
                    gallery: GallerySettings::default(),
                    editor: EditorSettings::default(),
                    capture: CaptureSettings::default(),
                    reminders: super::RemindersSettings::default(),
                    discovery: DiscoverySettings::default(),
                    voice: VoiceSettings::default(),
                    locked: LockedSettings::default(),
                    ai: AiSettings::default(),
                    nudges: super::Nudges {
                        first_seen_ms: login.then_some(i64::from(version) * 1_000),
                        sync_offer_dismissed: version % 2 == 0,
                    },
                },
            )
    }

    /// Sync settings, with Worker URLs that are the default (written
    /// several ways), another server, or not a URL at all.
    fn gen_sync() -> impl Strategy<Value = SyncSettings> {
        let url = prop_oneof![
            Just(kanso_sync::DEFAULT_SERVER.to_owned()),
            Just(format!("{}/", kanso_sync::DEFAULT_SERVER)),
            Just("https://KANSO-SYNC.imngonii.workers.dev".to_owned()),
            "https://[a-z]{1,8}[.][a-z]{2,3}",
            "[a-z ]{0,8}",
        ];
        let account = ("https://[a-z]{1,8}[.]dev", "a_[0-9a-f]{32}", "[a-z]{3,8}").prop_map(
            |(origin, id, username)| AccountRef {
                origin,
                id,
                username,
            },
        );
        (
            any::<bool>(),
            1u64..100_000,
            prop::option::of("https://[a-z.]{1,20}"),
            prop::option::of(account),
            prop::option::of(url),
        )
            .prop_map(
                |(enabled, interval_secs, server, account, endpoint)| SyncSettings {
                    enabled,
                    interval_secs,
                    server,
                    account,
                    endpoint,
                },
            )
    }

    fn gen_theme_name() -> impl Strategy<Value = String> {
        prop_oneof![
            Just(KANSO_LIGHT.to_owned()),
            Just(KANSO_DARK.to_owned()),
            "[A-Za-z ]{1,12}"
        ]
    }

    fn gen_themes() -> impl Strategy<Value = ThemeSettings> {
        (
            gen_theme_name(),
            prop::option::of(gen_theme_name()),
            prop::option::of(gen_theme_name()),
            any::<bool>(),
            gen_theme_name(),
            gen_theme_name(),
        )
            .prop_map(
                |(ui, syntax, preview, follow_system, light, dark)| ThemeSettings {
                    ui,
                    syntax,
                    preview,
                    follow_system,
                    light,
                    dark,
                },
            )
    }

    /// A version 2 file's light theme becomes Kanso Light, fixed; a file
    /// with no theme follows the system.
    #[test]
    fn old_theme_choices_migrate() {
        let mut light: AppConfig =
            serde_json::from_str(r#"{"version": 2, "appearance": {"theme": "light"}}"#).unwrap();
        assert!(light.migrate());
        assert!(!light.appearance.themes.follow_system);
        assert_eq!(light.appearance.themes.ui, KANSO_LIGHT);
        let mut none: AppConfig = serde_json::from_str(r#"{"version": 2}"#).unwrap();
        none.migrate();
        assert!(none.appearance.themes.follow_system);
        assert_eq!(none.appearance.themes.ui_for(false), KANSO_LIGHT);
        assert_eq!(none.appearance.themes.ui_for(true), KANSO_DARK);
        let chosen: AppConfig = serde_json::from_str(
            r#"{"version": 3, "appearance": {"themes": {"ui": "Nord", "followSystem": false, "syntax": "Dracula"}}}"#,
        )
        .unwrap();
        assert_eq!(chosen.appearance.themes.ui_for(true), "Nord");
        assert_eq!(chosen.appearance.themes.syntax_for(true), "Dracula");
        assert_eq!(chosen.appearance.themes.preview_for(true), "Nord");
        assert_eq!(chosen.appearance.themes.light, KANSO_LIGHT);
    }

    /// The fonts picked for writing, monospace and the preview, and the
    /// preview's size.
    #[allow(clippy::type_complexity)]
    fn gen_fonts() -> impl Strategy<Value = (Option<String>, Option<String>, Option<String>, f32)> {
        let family = || prop::option::of("[A-Za-z][A-Za-z0-9 ]{0,15}");
        (
            family(),
            family(),
            family(),
            prop_oneof![Just(DEFAULT_FONT_SIZE), 10.0f32..28.0],
        )
    }

    fn gen_font() -> impl Strategy<Value = EditorFont> {
        prop::sample::select(vec![EditorFont::Auto, EditorFont::Sans, EditorFont::Mono])
    }

    /// Zooms: every step, and values a hand-edited file might hold.
    fn gen_zoom() -> impl Strategy<Value = u32> {
        prop_oneof![
            (0u32..=7).prop_map(|step| crate::zoom::ZOOM_RANGE.0 + step * crate::zoom::ZOOM_STEP),
            any::<u32>(),
        ]
    }

    fn gen_mode() -> impl Strategy<Value = ViewMode> {
        prop::sample::select(vec![ViewMode::Edit, ViewMode::Split, ViewMode::Preview])
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        })]

        /// Migration ends at the current version and is idempotent. It
        /// lifts a never-changed old default size (from before version 1),
        /// makes a Worker URL that isn't the default the forms' server
        /// (from before version 2, unless one is set), turns light, dark or
        /// system into Kanso's own themes (from before version 3), and
        /// changes nothing
        /// else: the URL, `enabled` and the account stay as they were.
        /// Current files are left alone.
        #[test]
        fn prop_migration_lifts_old_files_to_v3(config in gen_config()) {
            let mut migrated = config.clone();
            let changed = migrated.migrate();
            prop_assert_eq!(migrated.version, CONFIG_VERSION.max(config.version));
            prop_assert_eq!(changed, config.version < CONFIG_VERSION);
            let expected_size = if config.version < 1 && config.appearance.editor_font_size == OLD_DEFAULT_FONT_SIZE {
                DEFAULT_FONT_SIZE
            } else {
                config.appearance.editor_font_size
            };
            prop_assert_eq!(migrated.appearance.editor_font_size, expected_size);
            let expected_server = if config.version < 2 && config.sync.server.is_none() {
                config.sync.endpoint.clone().filter(|e| !is_owner_endpoint(e))
            } else {
                config.sync.server.clone()
            };
            prop_assert_eq!(&migrated.sync.server, &expected_server);
            prop_assert_eq!(&migrated.sync.endpoint, &config.sync.endpoint);
            prop_assert_eq!(migrated.sync.enabled, config.sync.enabled);
            prop_assert_eq!(&migrated.sync.account, &config.sync.account);
            let expected_themes = if config.version < 3 {
                ThemeSettings::from_preference(config.appearance.theme)
            } else {
                config.appearance.themes.clone()
            };
            prop_assert_eq!(&migrated.appearance.themes, &expected_themes);
            let mut rest = migrated.clone();
            rest.appearance.themes = config.appearance.themes.clone();
            rest.version = config.version;
            rest.appearance.editor_font_size = config.appearance.editor_font_size;
            rest.sync.server = config.sync.server.clone();
            prop_assert_eq!(&rest, &config);
            let mut again = migrated.clone();
            prop_assert!(!again.migrate());
            prop_assert_eq!(again, migrated);
        }

        /// Following the system picks its light or dark theme and nothing
        /// else does; syntax and preview are the UI theme's unless chosen.
        /// A file from before themes means Kanso's own, following the
        /// system only for System.
        #[test]
        fn prop_theme_choice_follows_the_system_only_when_asked(themes in gen_themes(), dark: bool, pref in prop::sample::select(vec![ThemePreference::System, ThemePreference::Light, ThemePreference::Dark])) {
            let ui = themes.ui_for(dark);
            if themes.follow_system {
                prop_assert_eq!(ui, if dark { &themes.dark } else { &themes.light });
            } else {
                prop_assert_eq!(ui, &themes.ui);
                prop_assert_eq!(ui, themes.ui_for(!dark));
            }
            prop_assert_eq!(themes.syntax_for(dark), themes.syntax.as_deref().unwrap_or(ui));
            prop_assert_eq!(themes.preview_for(dark), themes.preview.as_deref().unwrap_or(ui));
            let old = ThemeSettings::from_preference(pref);
            prop_assert_eq!(old.follow_system, pref == ThemePreference::System);
            let expected = match (pref, dark) {
                (ThemePreference::Light, _) | (ThemePreference::System, false) => KANSO_LIGHT,
                _ => KANSO_DARK,
            };
            prop_assert_eq!(old.ui_for(dark), expected);
            prop_assert_eq!(old.syntax_for(dark), expected);
        }

        /// The default server is recognized however it is written, and no
        /// other server is taken for it.
        #[test]
        fn prop_only_the_default_server_is_the_default(host in "[a-z]{1,10}", slash: bool, upper: bool) {
            let default = kanso_sync::DEFAULT_SERVER;
            let written = if upper { default.replace("kanso-sync", "KANSO-SYNC") } else { default.to_owned() };
            let written = if slash { format!("{written}/") } else { written };
            prop_assert!(is_default_server(&written));
            let other = format!("https://{}.example", host);
            prop_assert!(!is_default_server(&other));
            prop_assert!(!is_default_server(&host));
        }

        /// Line numbers only ever come with the monospace face, and turning
        /// the setting off removes them without changing the face.
        #[test]
        fn prop_line_numbers_need_mono_and_the_setting(font in gen_font(), mode in gen_mode(), allowed: bool) {
            let face = font.face(mode);
            let shown = face.with_line_numbers(allowed);
            prop_assert_eq!(shown.mono, face.mono);
            prop_assert_eq!(shown.line_numbers, face.mono && allowed);
            prop_assert_eq!(face.with_line_numbers(true), face);
        }

        /// A file without a version (written before versions existed) loads
        /// as version 0, so its default size is migrated.
        #[test]
        fn prop_unversioned_files_migrate(config in gen_config()) {
            let mut value = serde_json::to_value(&config).unwrap();
            value.as_object_mut().unwrap().remove("version");
            let mut loaded: AppConfig = serde_json::from_value(value).unwrap();
            prop_assert_eq!(loaded.version, 0);
            loaded.migrate();
            prop_assert_eq!(loaded.version, CONFIG_VERSION);
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(128))]

        /// Round-trip: saving then loading gives the same config.
        #[test]
        fn prop_config_save_load_roundtrip(config in gen_config()) {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("config.json");
            config.save(&path).unwrap();
            prop_assert_eq!(AppConfig::load(&path).unwrap(), config);
        }

        /// Refinement: debug output never contains the password.
        #[test]
        fn prop_debug_redacts_password(password in "[a-zA-Z0-9]{8,24}") {
            let settings = ServerSettings { password: password.clone(), ..ServerSettings::default() };
            let debug = format!("{settings:?}");
            prop_assert!(!debug.contains(&password));
        }
    }

    /// Auto writes in the proportional face and switches to monospace with
    /// line numbers beside the preview; the fixed choices hold in every mode.
    #[test]
    fn editor_faces() {
        use ViewMode::{Edit, Preview, Split};
        let face = |font: EditorFont, mode| {
            let face = font.face(mode);
            assert_eq!(face.line_numbers, face.mono);
            face.mono
        };
        assert!(!face(EditorFont::Auto, Edit));
        assert!(face(EditorFont::Auto, Split));
        assert!(!face(EditorFont::Auto, Preview));
        for mode in [Edit, Split, Preview] {
            assert!(!face(EditorFont::Sans, mode));
            assert!(face(EditorFont::Mono, mode));
        }
    }

    /// A version 4 file means an API key: its provider, address, model and
    /// switch are kept, and saving writes no key or token.
    #[test]
    fn a_version_4_file_answers_with_its_api_key_provider() {
        let mut old: AppConfig = serde_json::from_str(
            r#"{"version": 4, "ai": {"enabled": true, "provider": "anthropic", "baseUrl": "https://api.anthropic.com", "model": "claude-haiku-5-5"}}"#,
        )
        .unwrap();
        assert!(old.migrate());
        assert_eq!(old.version, 5);
        assert_eq!(old.ai.backend, AiBackend::Api);
        assert!(old.ai.enabled);
        assert_eq!(old.ai.provider, kanso_ai::Provider::Anthropic);
        assert_eq!(old.ai.base_url, "https://api.anthropic.com");
        assert_eq!(old.ai.model, "claude-haiku-5-5");
        // Note access is on for an agent unless turned off, over every
        // unlocked notebook.
        assert!(old.ai.agent.mcp);
        assert!(old.ai.agent.notebooks.is_empty());
        // Suggested edits are off until turned on; no version bump is needed
        // for a file that never had the field.
        assert!(!old.ai.agent.suggest_edits);
        assert_eq!(old.version, CONFIG_VERSION);
        old.ai.backend = AiBackend::Agent;
        old.ai.agent.kind = kanso_ai::AgentKind::Codex;
        let written = serde_json::to_value(&old).unwrap();
        assert_eq!(written["ai"]["backend"], "agent");
        assert_eq!(written["ai"]["agent"]["kind"], "codex");
        let ai = written["ai"].to_string().to_ascii_lowercase();
        for secret in ["key", "token", "secret", "password"] {
            assert!(!ai.contains(secret), "{secret} written");
        }
    }

    /// An icon pack chosen before version 4 becomes the icon theme of the
    /// same folder name, and isn't written back; a later file keeps its
    /// icon theme.
    #[test]
    fn an_old_icon_pack_becomes_the_icon_theme() {
        let mut old: AppConfig =
            serde_json::from_str(r#"{"version": 3, "appearance": {"iconPack": "phosphor"}}"#)
                .unwrap();
        assert!(old.migrate());
        assert_eq!(old.appearance.icon_theme.as_deref(), Some("phosphor"));
        assert_eq!(old.appearance.legacy_icon_pack, None);
        let written = serde_json::to_value(&old).unwrap();
        assert_eq!(written["appearance"]["iconTheme"], "phosphor");
        assert!(written["appearance"].get("iconPack").is_none());
        let mut current: AppConfig = serde_json::from_str(
            r#"{"version": 5, "appearance": {"iconTheme": "mine", "iconPack": "old"}}"#,
        )
        .unwrap();
        assert!(!current.migrate());
        assert_eq!(current.appearance.icon_theme.as_deref(), Some("mine"));
        let mut none: AppConfig = serde_json::from_str(r#"{"version": 3}"#).unwrap();
        assert!(none.migrate());
        assert_eq!(none.appearance.icon_theme, None);
    }

    /// A file from before zoom opens at actual size; a chosen zoom is
    /// written as `zoom` and read back.
    #[test]
    fn zoom_defaults_to_actual_size_and_is_kept() {
        let old: AppConfig =
            serde_json::from_str(r#"{"version": 4, "appearance": {"editorFontSize": 15}}"#)
                .unwrap();
        assert_eq!(old.appearance.zoom, 100);
        assert_eq!(old.appearance.editor_font_size, 15.0);
        let mut zoomed = AppConfig::default();
        zoomed.appearance.zoom = 120;
        let value = serde_json::to_value(&zoomed).unwrap();
        assert_eq!(value["appearance"]["zoom"], 120);
        let back: AppConfig = serde_json::from_value(value).unwrap();
        assert_eq!(back.appearance.zoom, 120);
    }

    #[test]
    fn old_default_size_is_migrated_once() {
        let mut old: AppConfig =
            serde_json::from_str(r#"{"appearance": {"editorFontSize": 14}}"#).unwrap();
        assert!(old.migrate());
        assert_eq!(old.appearance.editor_font_size, 16.0);
        assert_eq!(old.appearance.editor_font, EditorFont::Auto);
        let mut chosen: AppConfig =
            serde_json::from_str(r#"{"appearance": {"editorFontSize": 15}}"#).unwrap();
        assert!(chosen.migrate());
        assert_eq!(chosen.appearance.editor_font_size, 15.0);
        // Someone who picks 14 after the migration keeps it, through the
        // next version's migration too.
        let mut current: AppConfig =
            serde_json::from_str(r#"{"version": 1, "appearance": {"editorFontSize": 14}}"#)
                .unwrap();
        assert!(current.migrate());
        assert_eq!(current.appearance.editor_font_size, 14.0);
        assert!(!current.migrate());
        assert_eq!(AppConfig::default().version, CONFIG_VERSION);
    }

    /// The owner's Mac before accounts: its Worker URL was the default, so
    /// the forms start at the default; another Mac's own Worker is kept for
    /// its forms. Both start signed out, with `endpoint` and `enabled`
    /// untouched for a rolled-back app.
    #[test]
    fn version_1_files_start_signed_out() {
        let owner = r#"{"version": 1, "sync": {"enabled": true, "endpoint": "https://kanso-sync.imngonii.workers.dev", "intervalSecs": 300}}"#;
        let mut config: AppConfig = serde_json::from_str(owner).unwrap();
        assert!(config.migrate());
        assert_eq!(config.version, CONFIG_VERSION);
        assert_eq!(config.sync.server, None);
        assert_eq!(config.sync.account, None);
        assert!(config.sync.enabled);
        assert_eq!(
            config.sync.endpoint.as_deref(),
            Some("https://kanso-sync.imngonii.workers.dev")
        );
        let custom = r#"{"version": 1, "sync": {"endpoint": "https://sync.example.dev"}}"#;
        let mut config: AppConfig = serde_json::from_str(custom).unwrap();
        assert!(config.migrate());
        assert_eq!(
            config.sync.server.as_deref(),
            Some("https://sync.example.dev")
        );
        assert_eq!(
            config.sync.endpoint.as_deref(),
            Some("https://sync.example.dev")
        );
        // An older app (which ignores fields it doesn't know) still finds
        // the URL where it left it.
        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["sync"]["endpoint"], "https://sync.example.dev");
        assert!(json["sync"]["account"].is_null());
        // A signed-in file keeps its account.
        config.sync.account = Some(AccountRef {
            origin: "https://sync.example.dev".into(),
            id: format!("a_{}", "0".repeat(32)),
            username: "ngoni".into(),
        });
        let back: AppConfig =
            serde_json::from_value(serde_json::to_value(&config).unwrap()).unwrap();
        assert_eq!(back, config);
    }

    /// AI is off in a new file, and a file from before AI takes the same
    /// defaults. Changing the provider brings its own address and model.
    #[test]
    fn ai_defaults_off_and_follows_its_provider() {
        let config = AppConfig::default();
        assert!(!config.ai.enabled);
        assert_eq!(config.ai.provider, kanso_ai::Provider::LocalModel);
        assert_eq!(config.ai.base_url, "http://localhost:11434/v1");
        assert_eq!(config.ai.model, "llama3.2");
        let old: AppConfig = serde_json::from_str(r#"{"version": 4}"#).unwrap();
        assert_eq!(old.ai, AiSettings::default());
        let mut ai = AiSettings::default();
        ai.choose(kanso_ai::Provider::Anthropic);
        assert_eq!(ai.base_url, "https://api.anthropic.com");
        assert_eq!(ai.model, "claude-haiku-5-5");
        ai.choose(kanso_ai::Provider::OpenRouter);
        assert_eq!(ai.model, "openai/gpt-4o-mini");
    }

    /// The config has no place for an API key: whatever the AI settings
    /// hold, saving them writes no secret.
    #[test]
    fn api_keys_are_never_written_to_config() {
        let mut config = AppConfig::default();
        config.ai.enabled = true;
        config.ai.provider = kanso_ai::Provider::OpenAi;
        config.ai.model = "gpt-4o-mini".into();
        let json = serde_json::to_string(&config).unwrap();
        assert!(!json.contains("apiKey"));
        assert!(!json.contains("api_key"));
        assert!(!json.contains("sk-"));
        let value = serde_json::to_value(&config).unwrap();
        assert_eq!(
            value["ai"]["provider"], "openAi",
            "the provider is written by name"
        );
    }

    #[test]
    fn missing_fields_take_defaults_and_file_is_private() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        std::fs::write(&path, r#"{"server": {"port": 1234}}"#).unwrap();
        let mut config = AppConfig::load(&path).unwrap();
        assert_eq!(config.server.port, 1234);
        assert_eq!(config.server.bind_address, "127.0.0.1");
        assert_eq!(config.appearance.view_mode, ViewMode::Split);
        // Files from before the setting keep their line numbers; text fills
        // the pane unless the readable measure is turned on.
        assert!(config.appearance.line_numbers);
        assert!(!config.appearance.readable_width);
        assert_eq!(config.appearance.diagram_renderer, DiagramRenderer::Pele);
        // Spelling is checked, though the file predates the setting.
        assert!(config.editor.spellcheck);
        assert!(config.ensure_credentials());
        assert!(!config.ensure_credentials());
        config.save(&path).unwrap();
        use std::os::unix::fs::PermissionsExt as _;
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert_eq!(
            AppConfig::load(&dir.path().join("missing.json")).unwrap(),
            AppConfig::default()
        );
    }

    /// Files from before the window layout was kept load with its defaults,
    /// and a partial one keeps what it has.
    #[test]
    fn older_files_take_the_default_layout() {
        let old: AppConfig =
            serde_json::from_str(r#"{"version": 1, "appearance": {"showSidebar": false}}"#)
                .unwrap();
        assert_eq!(old.layout, WindowLayout::default());
        assert!(!old.appearance.show_sidebar);
        assert_eq!(old.layout.split_ratio, DEFAULT_SPLIT_RATIO);
        let partial: AppConfig =
            serde_json::from_str(r#"{"layout": {"focusMode": true}}"#).unwrap();
        assert!(partial.layout.focus_mode);
        assert_eq!(partial.layout.split_ratio, DEFAULT_SPLIT_RATIO);
        assert_eq!(partial.layout.status_open, None);
        assert!(!partial.layout.inspector_open);
        assert_eq!(
            partial.layout.inspector_width,
            crate::layout::INSPECTOR_WIDTH
        );
        assert_eq!(partial.layout.inspector_tab, InspectorTab::Outline);
    }

    /// Files from before updates install themselves turn them on; turning
    /// them off is kept.
    #[test]
    fn older_files_get_the_writing_aids() {
        let config: AppConfig = serde_json::from_str(r#"{"version": 3}"#).unwrap();
        assert_eq!(config.editor, EditorSettings::default());
        let editor = config.editor;
        assert!(editor.auto_pair && editor.format_tables && editor.smart_paste);
        assert_eq!(editor.typewriter, Typewriter::FocusMode);
        assert_eq!(editor.word_goal, None);
        assert!(Typewriter::FocusMode.centres(true) && !Typewriter::FocusMode.centres(false));
        assert!(Typewriter::Always.centres(false) && !Typewriter::Off.centres(true));
    }

    #[test]
    fn older_files_install_updates_automatically() {
        let old: AppConfig = serde_json::from_str(r#"{"version": 3}"#).unwrap();
        assert!(old.updates.auto_install);
        let off: AppConfig =
            serde_json::from_str(r#"{"updates": {"autoInstall": false}}"#).unwrap();
        assert!(!off.updates.auto_install);
    }

    /// Spoken commands are on unless turned off, also for files written
    /// before there was the setting, which keep their engine.
    #[test]
    fn older_files_take_spoken_commands() {
        let old: AppConfig = serde_json::from_str(r#"{"voice": {"engine": "onDevice"}}"#).unwrap();
        assert_eq!(old.voice.engine, VoiceEngine::OnDevice);
        assert!(old.voice.spoken_commands);
        let off: AppConfig =
            serde_json::from_str(r#"{"voice": {"spokenCommands": false}}"#).unwrap();
        assert!(!off.voice.spoken_commands);
        assert_eq!(off.voice.engine, VoiceEngine::Ask);
    }

    #[test]
    fn a_leftover_temporary_file_does_not_widen_permissions() {
        use std::os::unix::fs::PermissionsExt as _;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, "{}").unwrap();
        std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o644)).unwrap();
        AppConfig::default().save(&path).unwrap();
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
}
