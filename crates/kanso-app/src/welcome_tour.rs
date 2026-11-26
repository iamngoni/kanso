//! Welcome tour: a six-page first-run setup sheet on desktop (issue #510).
//!
//! Opens once on first launch for new installs and existing users upgrading to
//! the version that ships it (`config.welcome_tour_seen == false`), and any
//! time after from Help › Welcome Tour or the command palette.
//!
//! Every control writes through the exact same Settings setters and globals
//! (`settings_themes`, `settings`, `icon_themes`, `account_flow`, `voice`,
//! `settings_ai`, `related`, `services::key_store`) so there is no second code
//! path for any setting. Downloads and sign-in live in app globals and keep
//! running if the sheet is closed.

use std::collections::BTreeSet;
use std::sync::{Arc, OnceLock};

use gpui_component::scroll::ScrollableElement as _;
use gpui_kit::assets::IconName as Lucide;
use gpui_kit::component::button::{Button, ButtonVariants as _};
use gpui_kit::component::input::{Input, InputEvent, InputState};
use gpui_kit::component::menu::{DropdownMenu as _, PopupMenuItem};
use gpui_kit::component::notification::Notification;
use gpui_kit::component::searchable_list::SearchableVec;
use gpui_kit::component::select::{Select, SelectEvent, SelectState};
use gpui_kit::component::switch::Switch;
use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::component::{
    Disableable as _, Icon, Selectable as _, Sizable as _, StyledExt as _, WindowExt as _, h_flex,
    v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use kanso_ai::Provider as AiProvider;
use kanso_gallery::Kind as GalleryKind;
use kanso_server::KeyAccess;
use zeroize::Zeroizing;

use crate::account_flow;
use crate::account_words::server_label;
use crate::api_keys_model::{Client, new_key};
use crate::config::{ThemePreference, ViewMode, VoiceEngine};
use crate::icon_themes::{self, IconTheme, IconThemes};
use crate::mcp_setup::{Agent, Setup};
use crate::passkey_ceremony;
use crate::related::{self, Related, Status as RelatedStatus};
use crate::services::{self, AccountState, ApiState, Services};
use crate::settings::{self, Row, mcp_endpoint, section, segmented, size_stepper};
use crate::settings_ai;
use crate::settings_discovery::{meter, related_words};
use crate::settings_gallery::{self, ThemeGallery};
use crate::settings_google_fonts::{GoogleFontAdded, GoogleFontRemoved, GoogleFonts};
use crate::settings_themes::{self, Kind as ThemeKind, selected, ui_thumb};
use crate::settings_voice::model_words;
use crate::theme::Colors;
use crate::themes::{self, ThemeEntry, Themes};
use crate::voice::{self, ModelState, Voice};
use crate::welcome_tour_model::{
    AGENTS_LINE, DetectedAgent, SAMPLE_PARAGRAPH, StepOutcome, TourPage, TourState, WELCOME_LINES,
    body_max_height, detect_mcp_agents, dialog_width, should_show_on_launch, theme_mode_of,
};
use crate::workspace::AppState;
use crate::zoom::px;

actions!(kanso, [ShowWelcomeTour]);

/// Compact card width inside the tour sheet so 3–4 themes sit cleanly in a row.
const TOUR_CARD_WIDTH: f32 = 124.;
const TOUR_THUMB_HEIGHT: f32 = 76.;

/// Cached embedded app icon PNG for Page 1.
fn app_icon() -> Arc<Image> {
    static ICON: OnceLock<Arc<Image>> = OnceLock::new();
    ICON.get_or_init(|| {
        let bytes = include_bytes!("../../../assets/icon/kanso.png").to_vec();
        Arc::new(Image::from_bytes(ImageFormat::Png, bytes))
    })
    .clone()
}

/// Marks whether the welcome tour dialog is currently open so closing the
/// window while it is up also records `welcome_tour_seen = true`, and
/// reopening moves the existing sheet instead of stacking another dialog.
#[derive(Default)]
struct WelcomeTourOpen(Option<WeakEntity<WelcomeTourSheet>>);

impl Global for WelcomeTourOpen {}

/// Marks the tour as seen in `AppConfig`, persisting immediately.
pub fn mark_seen(cx: &mut App) {
    if cx.has_global::<WelcomeTourOpen>() {
        cx.global_mut::<WelcomeTourOpen>().0 = None;
    }
    if !cx.has_global::<AppState>() {
        return;
    }
    if !cx.global::<AppState>().config.welcome_tour_seen {
        AppState::update_config(cx, |config| config.welcome_tour_seen = true);
    }
}

/// Called when the main window is closing: if the tour is open, record it as
/// seen so it won't pop up again on next launch.
pub fn on_window_close(cx: &mut App) {
    if cx
        .try_global::<WelcomeTourOpen>()
        .is_some_and(|open| open.0.as_ref().is_some_and(|w| w.upgrade().is_some()))
    {
        themes::stop_preview(cx);
        mark_seen(cx);
    }
}

/// One-click MCP snippet generated for an agent on Page 6, wiped on drop.
struct AgentSnippet {
    agent: Agent,
    text: Zeroizing<String>,
    copied: bool,
}

type FontSelect = SelectState<SearchableVec<SharedString>>;

/// The six-page Welcome Tour modal view.
pub struct WelcomeTourSheet {
    pub(crate) state: TourState,
    focus_handle: FocusHandle,
    scroll: ScrollHandle,
    // Page 3: Writing
    writing_font: Entity<FontSelect>,
    google_fonts: Entity<GoogleFonts>,
    // Page 4: Account & sync
    self_hosted: bool,
    server_url: Entity<InputState>,
    server_error: Option<String>,
    account_error: Option<String>,
    // Page 5: AI & voice
    ai_key: Entity<InputState>,
    ai_key_saved: bool,
    ai_status: Option<String>,
    // Page 6: Agents & MCP
    detected_agents: Option<Vec<DetectedAgent>>,
    show_all_agents: bool,
    agent_snippet: Option<AgentSnippet>,
    agent_error: Option<String>,
    // Subscriptions & background detection
    _account_listen: Option<Task<()>>,
    _detect_task: Option<Task<()>>,
    _subscriptions: Vec<Subscription>,
}

impl WelcomeTourSheet {
    // no-prop-test: constructs GPUI controls; state transitions live in `welcome_tour_model`.
    fn new(initial_page: TourPage, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle().tab_stop(true);
        let config = cx.global::<AppState>().config.clone();

        // Page 3 font picker reusing `crate::fonts` + `settings::set_editor_font_family`.
        let choices = crate::fonts::choices(&crate::fonts::installed(cx));
        let at = crate::fonts::position(config.appearance.editor_font_family.as_deref(), &choices);
        let writing_font = cx.new(|cx| {
            SelectState::new(
                SearchableVec::new(choices),
                Some(gpui_kit::component::IndexPath::new(at)),
                window,
                cx,
            )
            .searchable(true)
        });
        let google_fonts = cx.new(|cx| GoogleFonts::new(window, cx));

        // Page 4 sync server input.
        let initial_server = config.sync.server.clone().unwrap_or_default();
        let self_hosted = !initial_server.is_empty();
        let server_url = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("https://sync.example.com")
                .default_value(initial_server)
        });

        // Page 5 AI API key input.
        let ai_key = cx.new(|cx| {
            InputState::new(window, cx)
                .masked(true)
                .placeholder("API key (optional)")
        });
        let ai_key_saved = settings_ai::read_key(cx, config.ai.provider).is_some();

        let mut subscriptions = Vec::new();
        subscriptions.push(cx.subscribe(
            &writing_font,
            |_, _, event: &SelectEvent<SearchableVec<SharedString>>, cx| {
                let SelectEvent::Confirm(entry) = event;
                let family = entry.as_deref().and_then(crate::fonts::picked);
                settings::set_editor_font_family(family, cx);
                cx.notify();
            },
        ));
        subscriptions.push(cx.subscribe_in(
            &google_fonts,
            window,
            |this, _, GoogleFontAdded(family), window, cx| {
                settings::add_google_font(family.as_str(), cx);
                this.reload_font_picker(window, cx);
                cx.notify();
            },
        ));
        subscriptions.push(cx.subscribe_in(
            &google_fonts,
            window,
            |this, _, GoogleFontRemoved(family), window, cx| {
                settings::remove_google_font(family, cx);
                this.reload_font_picker(window, cx);
                cx.notify();
            },
        ));
        subscriptions.push(cx.subscribe_in(
            &server_url,
            window,
            |this, _, event: &InputEvent, window, cx| match event {
                InputEvent::PressEnter { shift: false, .. } => {
                    this.apply_server_url(window, cx);
                }
                InputEvent::Change => {
                    this.server_error = None;
                    cx.notify();
                }
                _ => {}
            },
        ));
        subscriptions.push(cx.subscribe_in(
            &ai_key,
            window,
            |this, _, event: &InputEvent, window, cx| match event {
                InputEvent::PressEnter { shift: false, .. } => {
                    this.save_ai_key(window, cx);
                }
                InputEvent::Change => {
                    this.ai_status = None;
                    cx.notify();
                }
                _ => {}
            },
        ));

        let mut sheet = WelcomeTourSheet {
            state: TourState { page: initial_page },
            focus_handle,
            scroll: ScrollHandle::new(),
            writing_font,
            google_fonts,
            self_hosted,
            server_url,
            server_error: None,
            account_error: None,
            ai_key,
            ai_key_saved,
            ai_status: None,
            detected_agents: None,
            show_all_agents: false,
            agent_snippet: None,
            agent_error: None,
            _account_listen: None,
            _detect_task: None,
            _subscriptions: subscriptions,
        };
        sheet.detect_agents(cx);
        sheet
    }

    /// Detects installed MCP-capable agents off the main thread.
    fn detect_agents(&mut self, cx: &mut Context<Self>) {
        if cfg!(test) {
            return;
        }
        let found = cx.background_executor().spawn(async move {
            let home = crate::paths::home();
            let home_opt = (!home.as_os_str().is_empty()).then_some(home);
            let dirs = kanso_acp::detect::search_dirs();
            detect_mcp_agents(home_opt.as_deref(), &dirs)
        });
        self._detect_task = Some(cx.spawn(async move |this, cx| {
            let list = found.await;
            this.update(cx, |this, cx| {
                this.detected_agents = Some(list);
                this._detect_task = None;
                cx.notify();
            })
            .ok();
        }));
    }

    /// Sets deterministic agent detections for tests and snapshots.
    #[cfg(test)]
    pub(crate) fn set_detected_agents_for_test(
        &mut self,
        agents: Vec<DetectedAgent>,
        cx: &mut Context<Self>,
    ) {
        self.detected_agents = Some(agents);
        self._detect_task = None;
        cx.notify();
    }

    #[cfg(test)]
    pub(crate) fn server_url_for_test(&self) -> Entity<InputState> {
        self.server_url.clone()
    }

    #[cfg(test)]
    pub(crate) fn ai_key_for_test(&self) -> Entity<InputState> {
        self.ai_key.clone()
    }

    #[cfg(test)]
    pub(crate) fn has_agent_snippet_for_test(&self, agent: Agent) -> bool {
        self.agent_snippet
            .as_ref()
            .is_some_and(|s| s.agent == agent && !s.text.is_empty())
    }

    fn reload_font_picker(&self, window: &mut Window, cx: &mut Context<Self>) {
        let choices = crate::fonts::choices(&crate::fonts::installed(cx));
        let current = cx
            .global::<AppState>()
            .config
            .appearance
            .editor_font_family
            .clone();
        let at = crate::fonts::position(current.as_deref(), &choices);
        self.writing_font.update(cx, |select, cx| {
            select.set_items(SearchableVec::new(choices), window, cx);
            select.set_selected_index(Some(gpui_kit::component::IndexPath::new(at)), window, cx);
        });
    }

    #[cfg(test)]
    pub fn page(&self) -> TourPage {
        self.state.page
    }

    /// Moves to `page`, resetting scroll and focusing the sheet.
    pub fn go_to(&mut self, page: TourPage, window: &mut Window, cx: &mut Context<Self>) {
        self.state.jump(page);
        self.scroll.scroll_to_item(0);
        window.focus(&self.focus_handle, cx);
        cx.notify();
    }

    /// Advances to the next page, or finishes and closes on the last page.
    pub fn next_or_finish(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match self.state.advance() {
            StepOutcome::Moved(_) => {
                self.scroll.scroll_to_item(0);
                window.focus(&self.focus_handle, cx);
                cx.notify();
            }
            StepOutcome::Finished => {
                self.finish(window, cx);
            }
        }
    }

    /// Steps back one page if not on the first page.
    pub fn back(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.state.back() {
            self.scroll.scroll_to_item(0);
            window.focus(&self.focus_handle, cx);
            cx.notify();
        }
    }

    /// Skips setup, marks `welcome_tour_seen = true`, and closes the dialog.
    pub fn skip(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.close_and_mark_seen(window, cx);
    }

    /// Finishes setup on the last page, marks `welcome_tour_seen = true`, and
    /// closes the dialog.
    pub fn finish(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.close_and_mark_seen(window, cx);
    }

    fn close_and_mark_seen(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.agent_snippet = None;
        themes::stop_preview(cx);
        mark_seen(cx);
        window.close_dialog(cx);
    }

    fn input_is_focused(&self, window: &Window, cx: &App) -> bool {
        if self.google_fonts.read(cx).is_focused(window, cx) {
            return true;
        }
        let Some(focused) = window.focused(cx) else {
            return false;
        };
        [
            self.server_url.focus_handle(cx),
            self.ai_key.focus_handle(cx),
        ]
        .contains(&focused)
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let key = event.keystroke.key.as_str();
        let mods = &event.keystroke.modifiers;
        let plain = !mods.control && !mods.alt && !mods.platform && !mods.function;
        if key == "escape" && plain {
            cx.stop_propagation();
            self.skip(window, cx);
        } else if key == "enter" && plain && !mods.shift && !self.input_is_focused(window, cx) {
            cx.stop_propagation();
            self.next_or_finish(window, cx);
        }
    }

    // MARK: Page 4 actions (Account & Sync)

    pub(crate) fn stay_local_only(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        AppState::update_config(cx, |config| config.sync.enabled = false);
        self.next_or_finish(window, cx);
    }

    pub(crate) fn use_default_server(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.self_hosted = false;
        self.server_error = None;
        self.server_url
            .update(cx, |input, cx| input.set_value("", window, cx));
        account_flow::set_server(None, cx);
        cx.notify();
    }

    pub(crate) fn apply_server_url(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let raw = self.server_url.read(cx).value().trim().to_owned();
        if raw.is_empty() {
            self.self_hosted = false;
            self.server_error = None;
            account_flow::set_server(None, cx);
            cx.notify();
            return;
        }
        match kanso_sync::check_endpoint(&raw) {
            Ok(url) => {
                self.self_hosted = true;
                self.server_error = None;
                account_flow::set_server(Some(url), cx);
                cx.notify();
            }
            Err(err) => {
                self.server_error = Some(err.to_string());
                cx.notify();
            }
        }
    }

    fn listen_account(
        &mut self,
        progress: async_channel::Receiver<crate::dialogs::Progress>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.account_error = None;
        self._account_listen = Some(cx.spawn_in(window, async move |this, cx| {
            let mut outcome: Result<(), String> = Ok(());
            while let Ok(event) = progress.recv().await {
                match event {
                    crate::dialogs::Progress::Step(_) => {
                        if this.update(cx, |_, cx| cx.notify()).is_err() {
                            return;
                        }
                    }
                    crate::dialogs::Progress::Done(result) => {
                        outcome = result;
                        break;
                    }
                }
            }
            this.update(cx, |this, cx| {
                this._account_listen = None;
                this.account_error = outcome.err();
                cx.notify();
            })
            .ok();
        }));
    }

    fn start_provider_sign_in(
        &mut self,
        provider: kanso_sync::Provider,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Sign-in runs inside `Services` and continues even if the tour closes.
        let progress = account_flow::start_provider(provider, false, window.window_handle(), cx);
        self.listen_account(progress, window, cx);
        cx.notify();
    }

    fn start_passkey_sign_in(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let server = account_flow::form_server(cx);
        let progress = account_flow::passkey_sign_in(
            server,
            account_flow::PasskeyMethod::Platform {
                window: passkey_ceremony::native_window(window),
            },
            None,
            window.window_handle(),
            cx,
        );
        self.listen_account(progress, window, cx);
        cx.notify();
    }

    // MARK: Page 5 actions (AI & Voice)

    pub(crate) fn set_ai_provider(
        &mut self,
        provider: AiProvider,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        settings_ai::choose_provider(provider, cx);
        self.ai_key_saved = settings_ai::read_key(cx, provider).is_some();
        self.ai_key
            .update(cx, |input, cx| input.set_value("", window, cx));
        self.ai_status = None;
        cx.notify();
    }

    pub(crate) fn save_ai_key(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let provider = cx.global::<AppState>().config.ai.provider;
        let typed = self.ai_key.read(cx).value().trim().to_owned();
        if typed.is_empty() {
            if !provider.needs_key() || self.ai_key_saved {
                settings_ai::set_ai_enabled(true, cx);
                self.ai_status = Some(format!("{} enabled.", provider.label()));
            } else {
                self.ai_status = Some("Enter an API key or continue to set up later.".to_owned());
            }
            cx.notify();
            return;
        }
        match settings_ai::write_key(cx, provider, &typed) {
            Ok(()) => {
                self.ai_key_saved = true;
                self.ai_key
                    .update(cx, |input, cx| input.set_value("", window, cx));
                settings_ai::set_ai_enabled(true, cx);
                self.ai_status = Some(format!(
                    "{} key saved in Keychain and AI enabled.",
                    provider.label()
                ));
            }
            Err(err) => {
                self.ai_status = Some(format!("Couldn't save the key: {err}"));
            }
        }
        cx.notify();
    }

    pub(crate) fn start_voice_download(&mut self, cx: &mut Context<Self>) {
        AppState::update_config(cx, |config| config.voice.engine = VoiceEngine::OnDevice);
        if cx.has_global::<Voice>() {
            voice::start_download(cx);
        }
        cx.notify();
    }

    // MARK: Page 6 actions (Agents & MCP)

    /// One-click "Add to <Agent>": ensures the local API server is on, mints a
    /// scoped read-only API key for `agent` using the exact same `KeyStore`
    /// Settings › Local API uses, builds its MCP snippet, copies it as a
    /// transient secret, and displays the paste instructions.
    pub(crate) fn add_to_agent(
        &mut self,
        agent: Agent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.agent_error = None;
        if !cx.global::<AppState>().config.server.enabled {
            settings::set_server_enabled(true, cx);
        }
        let keys = match services::key_store(cx) {
            Ok(keys) => keys,
            Err(reason) => {
                self.agent_error = Some(reason);
                cx.notify();
                return;
            }
        };
        let client = Client::Agent(agent);
        let Some(request) = new_key(
            client,
            &client.default_name(),
            KeyAccess::Read,
            true,
            &BTreeSet::new(),
        ) else {
            return;
        };
        match keys.mint(request) {
            Ok(minted) => {
                let server = cx.global::<AppState>().config.server.clone();
                let setup = Setup {
                    endpoint: mcp_endpoint(&server.bind_address, server.port),
                    key: Zeroizing::new(minted.key),
                    program: std::env::current_exe()
                        .map(|path| path.display().to_string())
                        .unwrap_or_else(|_| "kanso".to_owned()),
                };
                let snippet = Zeroizing::new(agent.snippet(&setup));
                let copied = crate::clipboard::copy_secret(&snippet, cx);
                if copied {
                    window.push_notification(Notification::info(client.copied_message()), cx);
                }
                self.agent_snippet = Some(AgentSnippet {
                    agent,
                    text: snippet,
                    copied,
                });
            }
            Err(err) => {
                self.agent_error = Some(format!("Couldn't create an API key: {err}"));
            }
        }
        cx.notify();
    }

    // MARK: Page renderers

    fn render_welcome(&self, colors: &Colors) -> AnyElement {
        let icon = app_icon();
        let points = [Lucide::NotebookText, Lucide::CloudCheck, Lucide::Sparkles]
            .into_iter()
            .zip(WELCOME_LINES)
            .map(|(icon, (title, detail))| {
                h_flex()
                    .w_full()
                    .gap(px(12.))
                    .items_start()
                    .p(px(10.))
                    .rounded(px(8.))
                    .bg(colors.fill)
                    .child(
                        div()
                            .mt(px(2.))
                            .flex_none()
                            .child(Icon::new(icon).size(px(16.)).text_color(colors.accent)),
                    )
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .gap(px(2.))
                            .child(
                                div()
                                    .text_size(px(13.5))
                                    .font_medium()
                                    .text_color(colors.text)
                                    .child(title),
                            )
                            .child(
                                div()
                                    .text_size(px(12.5))
                                    .line_height(px(17.))
                                    .text_color(colors.text_secondary)
                                    .child(detail),
                            ),
                    )
            });

        v_flex()
            .w_full()
            .items_center()
            .gap(px(14.))
            .py(px(6.))
            .child(
                div()
                    .debug_selector(|| "welcome-tour-app-icon".into())
                    .size(px(68.))
                    .rounded(px(16.))
                    .overflow_hidden()
                    .child(img(icon).size(px(68.)).object_fit(ObjectFit::Contain)),
            )
            .child(
                v_flex()
                    .items_center()
                    .gap(px(4.))
                    .child(
                        div()
                            .text_size(px(20.))
                            .font_semibold()
                            .text_color(colors.text)
                            .child("Welcome to Kanso"),
                    )
                    .child(
                        div()
                            .text_size(px(13.))
                            .text_color(colors.text_secondary)
                            .text_center()
                            .child("Calm, local-first notes that work the way you think."),
                    ),
            )
            .child(v_flex().w_full().gap(px(8.)).children(points))
            .into_any_element()
    }

    fn render_look_and_feel(&self, colors: &Colors, cx: &mut Context<Self>) -> AnyElement {
        let appearance = cx.global::<AppState>().config.appearance.clone();
        let theme_settings = appearance.themes.clone();
        let active_dark = cx
            .try_global::<themes::ActiveTheme>()
            .map(|a| a.ui.dark())
            .unwrap_or_else(|| colors.editor.0 < 0x80_80_80);
        let current_mode = theme_mode_of(&theme_settings, active_dark);

        let mode_row = section(
            "Appearance",
            vec![
                Row::new(Lucide::SunMoon, "Mode")
                    .detail("Follow your system automatically, or stay in light or dark mode.")
                    .trailing(segmented(
                        "tour-theme-mode",
                        [
                            ("System", ThemePreference::System),
                            ("Light", ThemePreference::Light),
                            ("Dark", ThemePreference::Dark),
                        ],
                        current_mode,
                        settings_themes::apply_mode,
                        colors,
                    )),
            ],
            colors,
        );

        // Installed themes
        let installed_entries: Vec<Arc<ThemeEntry>> = cx
            .try_global::<Themes>()
            .map(|t| t.entries.clone())
            .unwrap_or_else(|| themes::builtins().to_vec());

        let mut theme_cards: Vec<AnyElement> = installed_entries
            .iter()
            .map(|entry| tour_theme_card(entry, &theme_settings, colors))
            .collect();

        // Gallery themes (when catalog is loaded) not already installed; lazily fetch if idle.
        if !cfg!(test)
            && cx
                .try_global::<ThemeGallery>()
                .is_some_and(|g| g.index.is_none() && g.status == settings_gallery::Status::Idle)
        {
            cx.defer(|cx| settings_gallery::refresh(false, cx));
        }
        let mut to_request = Vec::new();
        if let Some(gallery) = cx.try_global::<ThemeGallery>()
            && let Some(index) = &gallery.index
        {
            let installed_names: BTreeSet<&str> =
                installed_entries.iter().map(|e| e.name()).collect();
            let gallery_themes: Vec<_> = index
                .themes
                .iter()
                .filter(|t| {
                    t.kind == GalleryKind::Color && !installed_names.contains(t.name.as_str())
                })
                .take(6)
                .cloned()
                .collect();
            for theme in gallery_themes {
                let fetched = gallery.files.get(&theme.id).cloned();
                let previewing = gallery.preview.as_ref().is_some_and(|p| p.id == theme.id);
                if fetched.is_none()
                    && !gallery.requested.contains(&theme.id)
                    && !gallery.failed.contains(&theme.id)
                {
                    to_request.push(theme.clone());
                }
                theme_cards.push(tour_gallery_card(theme, fetched, previewing, colors));
            }
        }
        for req in to_request {
            cx.defer(move |cx| settings_gallery::request(req, cx));
        }

        // Icon themes
        let active_icons = icon_themes::active(cx);
        let icon_entries: Vec<Arc<IconTheme>> = cx
            .try_global::<IconThemes>()
            .map(|t| t.entries.clone())
            .unwrap_or_default();
        let icon_cards: Vec<AnyElement> = icon_entries
            .iter()
            .map(|theme| tour_icon_card(theme, theme.key == active_icons.key, colors))
            .collect();

        v_flex()
            .w_full()
            .gap(px(14.))
            .child(mode_row)
            .child(
                v_flex()
                    .gap(px(6.))
                    .child(
                        div()
                            .text_size(px(13.))
                            .font_medium()
                            .text_color(colors.text_secondary)
                            .child("Theme"),
                    )
                    .child(h_flex().flex_wrap().gap(px(10.)).children(theme_cards)),
            )
            .when(!icon_cards.is_empty(), |el| {
                el.child(
                    v_flex()
                        .gap(px(6.))
                        .child(
                            div()
                                .text_size(px(13.))
                                .font_medium()
                                .text_color(colors.text_secondary)
                                .child("Icon theme"),
                        )
                        .child(h_flex().flex_wrap().gap(px(10.)).children(icon_cards)),
                )
            })
            .into_any_element()
    }

    fn render_writing(&self, colors: &Colors, cx: &mut Context<Self>) -> AnyElement {
        let appearance = cx.global::<AppState>().config.appearance.clone();
        let view_mode = match appearance.view_mode {
            ViewMode::Live => ViewMode::Edit,
            other => other,
        };
        let stepper = size_stepper(
            ("tour-font-smaller", "tour-font-larger"),
            appearance.editor_font_size,
            settings::step_editor_size,
            colors,
        );

        let sample_box = {
            let mut text = div()
                .w_full()
                .text_size(px(appearance.editor_font_size))
                .line_height(px((appearance.editor_font_size * 1.55).round()))
                .text_color(colors.text)
                .child(SAMPLE_PARAGRAPH);
            if let Some(family) = appearance.editor_font_family.clone() {
                text = text.font_family(family);
            }
            if appearance.readable_width {
                text = text.max_w(px(360.)).mx_auto();
            }
            div()
                .debug_selector(|| "tour-writing-sample".into())
                .w_full()
                .p(px(14.))
                .rounded(px(8.))
                .bg(colors.editor)
                .border_1()
                .border_color(colors.divider)
                .child(text)
        };

        v_flex()
            .w_full()
            .gap(px(12.))
            .child(sample_box)
            .child(section(
                "Editor",
                vec![
                    Row::new(Lucide::CaseSensitive, "Writing font")
                        .detail("System fonts and any Google Fonts you add below.")
                        .trailing(
                            div()
                                .w(px(200.))
                                .debug_selector(|| "tour-font-writing".into())
                                .child(
                                    Select::new(&self.writing_font)
                                        .small()
                                        .search_placeholder("Search fonts")
                                        .menu_width(px(240.)),
                                ),
                        ),
                    Row::new(Lucide::ALargeSmall, "Text size")
                        .detail("Applies to Markdown prose in the editor.")
                        .trailing(stepper),
                    Row::new(Lucide::Ruler, "Readable width")
                        .detail("Keeps lines to a comfortable reading measure.")
                        .trailing(
                            div().debug_selector(|| "tour-readable-width".into()).child(
                                Switch::new("tour-readable-width")
                                    .checked(appearance.readable_width)
                                    .on_click(|on, _, cx| settings::set_readable_width(*on, cx)),
                            ),
                        ),
                    Row::new(Lucide::Columns2, "Default view")
                        .detail("Choose how notes open by default.")
                        .trailing(segmented(
                            "tour-view-mode",
                            [
                                ("Edit", ViewMode::Edit),
                                ("Split", ViewMode::Split),
                                ("Preview", ViewMode::Preview),
                            ],
                            view_mode,
                            |mode, _, cx| settings::set_view_mode(mode, cx),
                            colors,
                        )),
                ],
                colors,
            ))
            .child(GoogleFonts::section(&self.google_fonts, cx, colors))
            .into_any_element()
    }

    fn render_account_and_sync(&self, colors: &Colors, cx: &mut Context<Self>) -> AnyElement {
        let services = cx.try_global::<Services>();
        let account = services
            .map(|s| s.account.clone())
            .unwrap_or(AccountState::SignedOut);
        let provider_wait = services.and_then(|s| s.provider_wait.clone());
        let busy = services.is_some_and(|s| s.task.is_some());
        let server_cfg = cx.global::<AppState>().config.sync.server.clone();

        // Already signed in: show who they are and invite skipping ahead.
        if let AccountState::SignedIn { username } | AccountState::SignedInLocked { username } =
            &account
        {
            return v_flex()
                .w_full()
                .gap(px(12.))
                .child(section(
                    "Signed in",
                    vec![
                        Row::new(Lucide::CircleUser, format!("Signed in as @{username}")).detail(
                            format!(
                                "Connected to {}. You're all set — click Next to continue.",
                                server_label(server_cfg.as_deref())
                            ),
                        ),
                    ],
                    colors,
                ))
                .into_any_element();
        }

        let platform_passkey = services.is_some_and(|s| {
            !s.platform_passkeys_failed
                && s.platform_passkeys
                    .unwrap_or_else(passkey_ceremony::platform_available)
        });

        let sign_in_buttons = h_flex()
            .flex_wrap()
            .gap(px(8.))
            .child(
                Button::new("tour-sign-in-apple")
                    .debug_selector(|| "tour-sign-in-apple".into())
                    .outline()
                    .small()
                    .icon(Lucide::Apple)
                    .label("Sign in with Apple")
                    .disabled(busy)
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.start_provider_sign_in(kanso_sync::Provider::Apple, window, cx);
                    })),
            )
            .child(
                Button::new("tour-sign-in-github")
                    .debug_selector(|| "tour-sign-in-github".into())
                    .outline()
                    .small()
                    .icon(Lucide::Github)
                    .label("Sign in with GitHub")
                    .disabled(busy)
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.start_provider_sign_in(kanso_sync::Provider::Github, window, cx);
                    })),
            )
            .when(platform_passkey, |el| {
                el.child(
                    Button::new("tour-sign-in-passkey")
                        .debug_selector(|| "tour-sign-in-passkey".into())
                        .outline()
                        .small()
                        .icon(Lucide::KeyRound)
                        .label("Sign in with Passkey")
                        .disabled(busy)
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.start_passkey_sign_in(window, cx);
                        })),
                )
            })
            .child(
                Button::new("tour-stay-local")
                    .debug_selector(|| "tour-stay-local".into())
                    .ghost()
                    .small()
                    .label("Stay local-only")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.stay_local_only(window, cx);
                    })),
            );

        let waiting_box = provider_wait.map(|wait| {
            v_flex()
                .w_full()
                .gap(px(6.))
                .p(px(12.))
                .rounded(px(8.))
                .bg(colors.fill)
                .border_1()
                .border_color(colors.divider)
                .child(
                    div()
                        .text_size(px(13.))
                        .font_medium()
                        .text_color(colors.text)
                        .child(format!(
                            "Waiting for {} sign-in… (keeps running if you continue)",
                            wait.provider
                        )),
                )
                .when_some(wait.user_code, |el, code| {
                    el.child(
                        div()
                            .text_size(px(15.))
                            .font_semibold()
                            .text_color(colors.accent)
                            .child(format!("Code: {code}")),
                    )
                })
                .when_some(wait.verification_uri, |el, url| {
                    let open_url = url.clone();
                    el.child(
                        h_flex().gap(px(8.)).items_center().child(
                            Button::new("tour-open-verify-url")
                                .outline()
                                .xsmall()
                                .label(format!("Open {url}"))
                                .on_click(move |_, _, cx| cx.open_url(&open_url)),
                        ),
                    )
                })
        });

        let server_mode_buttons = h_flex()
            .gap(px(6.))
            .child(
                Button::new("tour-server-default")
                    .debug_selector(|| "tour-server-default".into())
                    .outline()
                    .small()
                    .selected(!self.self_hosted && server_cfg.is_none())
                    .label("Kanso Cloud (default)")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.use_default_server(window, cx);
                    })),
            )
            .child(
                Button::new("tour-server-self-hosted")
                    .debug_selector(|| "tour-server-self-hosted".into())
                    .outline()
                    .small()
                    .selected(self.self_hosted || server_cfg.is_some())
                    .label("Self-hosted")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.self_hosted = true;
                        this.server_url
                            .update(cx, |input, cx| input.focus(window, cx));
                        cx.notify();
                    })),
            );

        let mut server_rows = vec![
            Row::new(Lucide::Server, "Sync server")
                .detail(server_label(server_cfg.as_deref()))
                .trailing(server_mode_buttons),
        ];
        if self.self_hosted || server_cfg.is_some() {
            server_rows.push(
                Row::new(Lucide::Link, "Server URL")
                    .detail(self.server_error.clone().unwrap_or_else(|| {
                        "Enter your self-hosted Kanso sync server URL.".to_owned()
                    }))
                    .trailing(
                        div()
                            .w(px(210.))
                            .debug_selector(|| "tour-server-url".into())
                            .child(Input::new(&self.server_url)),
                    )
                    .trailing(
                        Button::new("tour-server-apply")
                            .debug_selector(|| "tour-server-apply".into())
                            .primary()
                            .small()
                            .label("Save")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.apply_server_url(window, cx);
                            })),
                    ),
            );
        }

        v_flex()
            .w_full()
            .gap(px(12.))
            .child(
                v_flex()
                    .w_full()
                    .gap(px(10.))
                    .p(px(12.))
                    .rounded(px(8.))
                    .bg(colors.fill)
                    .border_1()
                    .border_color(colors.divider)
                    .child(
                        div()
                            .text_size(px(13.))
                            .text_color(colors.text)
                            .child("Sign in to sync your notes end-to-end encrypted, or keep everything local."),
                    )
                    .child(sign_in_buttons)
                    .when_some(self.account_error.clone(), |el, err| {
                        el.child(div().text_size(px(12.5)).text_color(colors.danger).child(err))
                    }),
            )
            .children(waiting_box)
            .child(section("Server", server_rows, colors))
            .into_any_element()
    }

    fn render_ai_and_voice(&self, colors: &Colors, cx: &mut Context<Self>) -> AnyElement {
        let config = cx.global::<AppState>().config.clone();

        // 1. Voice model
        let voice_model = cx
            .try_global::<Voice>()
            .map(|v| v.model.clone())
            .unwrap_or(ModelState::Missing);
        let (voice_title, voice_detail) = model_words(&voice_model);
        let voice_action: AnyElement = match &voice_model {
            ModelState::Downloading { done, total } => h_flex()
                .gap(px(8.))
                .items_center()
                .child(meter(*done as f32 / (*total).max(1) as f32, colors))
                .child(
                    Button::new("tour-voice-stop")
                        .outline()
                        .small()
                        .label("Stop")
                        .on_click(|_, _, cx| voice::cancel_download(cx)),
                )
                .into_any_element(),
            ModelState::Ready => div()
                .text_size(px(12.5))
                .text_color(colors.accent)
                .child("Ready")
                .into_any_element(),
            ModelState::Failed(_) => Button::new("tour-voice-retry")
                .debug_selector(|| "tour-voice-download".into())
                .outline()
                .small()
                .label("Try Again")
                .on_click(cx.listener(|this, _, _, cx| this.start_voice_download(cx)))
                .into_any_element(),
            ModelState::Unknown | ModelState::Missing => Button::new("tour-voice-download")
                .debug_selector(|| "tour-voice-download".into())
                .outline()
                .small()
                .label("Download")
                .on_click(cx.listener(|this, _, _, cx| this.start_voice_download(cx)))
                .into_any_element(),
        };

        let mut voice_rows = vec![
            Row::new(Lucide::Mic, voice_title)
                .detail(format!(
                    "{voice_detail} Downloads continue in the background."
                ))
                .trailing(voice_action),
        ];
        if crate::dictation::AVAILABLE {
            voice_rows.push(
                Row::new(Lucide::Keyboard, "Voice engine")
                    .detail("On-device Parakeet model or system dictation.")
                    .trailing(segmented(
                        "tour-voice-engine",
                        [
                            ("On Device", VoiceEngine::OnDevice),
                            ("System", VoiceEngine::System),
                        ],
                        config.voice.engine,
                        |engine, _, cx| {
                            AppState::update_config(cx, |config| config.voice.engine = engine)
                        },
                        colors,
                    )),
            );
        }

        // 2. AI provider & key
        let view = cx.entity();
        let chosen_provider = config.ai.provider;
        let provider_menu = Button::new("tour-ai-provider")
            .debug_selector(|| "tour-ai-provider".into())
            .outline()
            .small()
            .label(chosen_provider.label())
            .dropdown_caret(true)
            .dropdown_menu(move |mut menu, _, _| {
                for option in AiProvider::ALL {
                    let view = view.clone();
                    menu = menu.item(
                        PopupMenuItem::new(option.label())
                            .checked(option == chosen_provider)
                            .on_click(move |_, window, cx| {
                                view.update(cx, |this, cx| {
                                    this.set_ai_provider(option, window, cx)
                                });
                            }),
                    );
                }
                menu
            });
        let key_detail = if let Some(status) = &self.ai_status {
            status.clone()
        } else if self.ai_key_saved {
            "Saved in the Keychain. Leave blank to keep it, or skip for later.".to_owned()
        } else if chosen_provider.needs_key() {
            "Kept in the Keychain. Leave blank to set up later in Settings › AI.".to_owned()
        } else {
            "Optional for local models. Leave blank to set up later.".to_owned()
        };
        let ai_rows = vec![
            Row::new(Lucide::Sparkles, "AI provider")
                .detail("Optional — Ask Your Notes and voice note tidying.")
                .trailing(provider_menu),
            Row::new(Lucide::Key, "API key")
                .detail(key_detail)
                .trailing(
                    div()
                        .w(px(205.))
                        .debug_selector(|| "tour-ai-key".into())
                        .child(Input::new(&self.ai_key).mask_toggle()),
                )
                .trailing(
                    Button::new("tour-ai-save")
                        .debug_selector(|| "tour-ai-save".into())
                        .outline()
                        .small()
                        .label("Save")
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.save_ai_key(window, cx);
                        })),
                ),
        ];

        // 3. Semantic search / Related notes model
        let related_on = config.discovery.related_notes;
        let related_status = cx
            .try_global::<Related>()
            .map(|r| r.status.clone())
            .unwrap_or_default();
        let mut discovery_rows = vec![
            Row::new(Lucide::Search, "Semantic search & related notes")
                .detail(
                    "Downloads a ~30 MB on-device language model once. Nothing leaves this device.",
                )
                .trailing(
                    div()
                        .debug_selector(|| "tour-semantic-search".into())
                        .child(
                            Switch::new("tour-semantic-search")
                                .checked(related_on)
                                .on_click(|on, _, cx| related::set_enabled(*on, cx)),
                        ),
                ),
        ];
        if related_on && let Some((title, detail)) = related_words(&related_status) {
            let mut row = Row::new(Lucide::Download, title).detail(detail);
            if let RelatedStatus::Downloading { done, total } = related_status {
                row = row.trailing(meter(done as f32 / total.max(1) as f32, colors));
            }
            if let RelatedStatus::Indexing { done, total } = related_status {
                row = row.trailing(meter(done as f32 / total.max(1) as f32, colors));
            }
            discovery_rows.push(row);
        }

        v_flex()
            .w_full()
            .gap(px(12.))
            .child(section("Voice", voice_rows, colors))
            .child(section("AI", ai_rows, colors))
            .child(section("Semantic search", discovery_rows, colors))
            .into_any_element()
    }

    fn render_agents_and_mcp(&self, colors: &Colors, cx: &mut Context<Self>) -> AnyElement {
        let server = cx.global::<AppState>().config.server.clone();
        let endpoint = mcp_endpoint(&server.bind_address, server.port);
        let api_state = cx
            .try_global::<Services>()
            .map(|s| s.api.clone())
            .unwrap_or(ApiState::Off);
        let status = settings::api_status_line(&api_state);

        let all_agents = self.detected_agents.clone().unwrap_or_else(|| {
            Agent::ALL
                .into_iter()
                .map(|agent| DetectedAgent {
                    agent,
                    detected: matches!(agent, Agent::ClaudeCode | Agent::Codex | Agent::Cursor),
                })
                .collect()
        });
        let detected_only: Vec<DetectedAgent> =
            all_agents.iter().copied().filter(|a| a.detected).collect();
        let shown_agents: Vec<DetectedAgent> = if self.show_all_agents || detected_only.is_empty() {
            all_agents
        } else {
            detected_only
        };

        let copy_endpoint = endpoint.clone();
        let server_section = section(
            "Local MCP server",
            vec![
                Row::new(Lucide::Plug, "Enable local MCP server")
                    .detail(format!("{} — {}", status.label, status.detail))
                    .trailing(
                        div().debug_selector(|| "tour-mcp-enabled".into()).child(
                            Switch::new("tour-mcp-enabled")
                                .checked(server.enabled)
                                .on_click(|on, _, cx| settings::set_server_enabled(*on, cx)),
                        ),
                    ),
                Row::new(Lucide::Link, "MCP endpoint")
                    .value(endpoint.clone())
                    .trailing(
                        Button::new("tour-copy-mcp")
                            .debug_selector(|| "tour-copy-mcp".into())
                            .outline()
                            .small()
                            .label("Copy")
                            .on_click(move |_, window, cx| {
                                settings::copy_to_clipboard(
                                    copy_endpoint.clone(),
                                    "MCP endpoint copied.",
                                    window,
                                    cx,
                                );
                            }),
                    ),
            ],
            colors,
        );

        let agent_buttons = shown_agents.into_iter().map(|entry| {
            let agent = entry.agent;
            let id = format!("tour-add-agent-{}", agent.id());
            let sel = id.clone();
            Button::new(SharedString::from(id))
                .debug_selector(move || sel.clone())
                .outline()
                .small()
                .icon(Lucide::PlugZap)
                .label(format!("Add to {}", agent.name()))
                .on_click(cx.listener(move |this, _, window, cx| {
                    this.add_to_agent(agent, window, cx);
                }))
        });

        let snippet_panel = self.agent_snippet.as_ref().map(|made| {
            let mono = crate::theme::mono_font(cx);
            let copy_text = made.text.as_str().to_owned();
            let agent = made.agent;
            v_flex()
                .debug_selector(|| "tour-agent-snippet".into())
                .w_full()
                .gap(px(6.))
                .p(px(12.))
                .rounded(px(8.))
                .bg(colors.fill)
                .border_1()
                .border_color(colors.divider)
                .child(
                    h_flex()
                        .justify_between()
                        .items_center()
                        .child(
                            div()
                                .text_size(px(13.))
                                .font_medium()
                                .text_color(colors.text)
                                .child(format!(
                                    "{} setup{}",
                                    agent.name(),
                                    if made.copied { " (copied)" } else { "" }
                                )),
                        )
                        .child(
                            Button::new("tour-copy-snippet")
                                .outline()
                                .xsmall()
                                .label("Copy again")
                                .on_click(move |_, window, cx| {
                                    if crate::clipboard::copy_secret(&copy_text, cx) {
                                        window.push_notification(
                                            Notification::info(
                                                Client::Agent(agent).copied_message(),
                                            ),
                                            cx,
                                        );
                                    }
                                }),
                        ),
                )
                .child(
                    div()
                        .text_size(px(12.))
                        .line_height(px(17.))
                        .text_color(colors.text_secondary)
                        .child(agent.instructions()),
                )
                .child(
                    div()
                        .w_full()
                        .p(px(8.))
                        .rounded(px(6.))
                        .bg(colors.editor)
                        .border_1()
                        .border_color(colors.divider)
                        .text_size(px(11.5))
                        .text_color(colors.text)
                        .when_some(mono, |el, family| el.font_family(family))
                        .child(made.text.as_str().to_owned()),
                )
        });

        v_flex()
            .w_full()
            .gap(px(12.))
            .child(
                div()
                    .text_size(px(13.))
                    .line_height(px(19.))
                    .text_color(colors.text_secondary)
                    .child(AGENTS_LINE),
            )
            .child(server_section)
            .child(
                v_flex()
                    .w_full()
                    .gap(px(8.))
                    .child(
                        h_flex()
                            .justify_between()
                            .items_center()
                            .child(
                                div()
                                    .text_size(px(13.))
                                    .font_medium()
                                    .text_color(colors.text_secondary)
                                    .child("Connect an agent"),
                            )
                            .child(
                                Button::new("tour-toggle-all-agents")
                                    .ghost()
                                    .xsmall()
                                    .label(if self.show_all_agents {
                                        "Detected only"
                                    } else {
                                        "Show all agents"
                                    })
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.show_all_agents = !this.show_all_agents;
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(h_flex().flex_wrap().gap(px(8.)).children(agent_buttons))
                    .when_some(self.agent_error.clone(), |el, err| {
                        el.child(
                            div()
                                .text_size(px(12.5))
                                .text_color(colors.danger)
                                .child(err),
                        )
                    })
                    .children(snippet_panel),
            )
            .into_any_element()
    }
}

/// Compact installed theme card for Page 2, clicking applies via
/// `settings_themes::pick_ui`.
fn tour_theme_card(
    entry: &Arc<ThemeEntry>,
    settings: &crate::config::ThemeSettings,
    colors: &Colors,
) -> AnyElement {
    let name = entry.name().to_owned();
    let (chosen, badge) = selected(settings, ThemeKind::Ui, Some(&name));
    let dark = entry.dark();
    let id = format!("tour-theme-{}", entry.id.to_lowercase().replace(' ', "-"));
    let sel = id.clone();
    v_flex()
        .id(SharedString::from(id))
        .debug_selector(move || sel.clone())
        .w(px(TOUR_CARD_WIDTH))
        .gap(px(4.))
        .cursor_pointer()
        .child(
            div()
                .p(px(2.))
                .rounded(px(8.))
                .border_2()
                .border_color(if chosen {
                    Hsla::from(colors.accent)
                } else {
                    transparent_black()
                })
                .child(
                    ui_thumb(entry)
                        .w_full()
                        .h(px(TOUR_THUMB_HEIGHT))
                        .rounded(px(6.))
                        .overflow_hidden()
                        .border_1()
                        .border_color(colors.divider),
                ),
        )
        .child(
            h_flex()
                .justify_center()
                .gap(px(4.))
                .text_size(px(11.5))
                .text_color(if chosen {
                    colors.text
                } else {
                    colors.text_secondary
                })
                .when(chosen, |el| el.font_medium())
                .child(div().truncate().child(name.clone()))
                .when_some(badge, |el, badge| {
                    el.child(
                        div()
                            .flex_none()
                            .px(px(4.))
                            .rounded(px(3.))
                            .bg(colors.fill)
                            .text_size(px(10.))
                            .text_color(colors.text_secondary)
                            .child(badge),
                    )
                }),
        )
        .on_click(move |_, window, cx| {
            settings_themes::pick_ui(&name, dark, window, cx);
        })
        .into_any_element()
}

/// Compact gallery theme card for Page 2: previews live on click (and installs
/// when clicked again or via its Install chip).
fn tour_gallery_card(
    theme: kanso_gallery::Theme,
    fetched: Option<Arc<ThemeEntry>>,
    previewing: bool,
    colors: &Colors,
) -> AnyElement {
    let id = format!("tour-gallery-theme-{}", theme.id);
    let sel = id.clone();
    let label = theme.name.clone();
    let preview_theme = theme.clone();
    let install_theme = theme.clone();
    let thumb = match &fetched {
        Some(entry) => ui_thumb(entry)
            .w_full()
            .h(px(TOUR_THUMB_HEIGHT))
            .rounded(px(6.))
            .overflow_hidden()
            .border_1()
            .border_color(colors.divider),
        None => div()
            .w_full()
            .h(px(TOUR_THUMB_HEIGHT))
            .rounded(px(6.))
            .bg(colors.fill)
            .border_1()
            .border_color(colors.divider)
            .flex()
            .items_center()
            .justify_center()
            .text_size(px(11.))
            .text_color(colors.text_muted)
            .child("Gallery"),
    };
    v_flex()
        .id(SharedString::from(id))
        .debug_selector(move || sel.clone())
        .w(px(TOUR_CARD_WIDTH))
        .gap(px(4.))
        .cursor_pointer()
        .tooltip(|window, cx| Tooltip::new("Click to preview from the gallery").build(window, cx))
        .child(
            div()
                .p(px(2.))
                .rounded(px(8.))
                .border_2()
                .border_color(if previewing {
                    Hsla::from(colors.accent)
                } else {
                    transparent_black()
                })
                .child(thumb),
        )
        .child(
            h_flex()
                .justify_center()
                .gap(px(4.))
                .text_size(px(11.5))
                .text_color(if previewing {
                    colors.text
                } else {
                    colors.text_secondary
                })
                .child(div().truncate().child(label))
                .child(
                    Button::new(SharedString::from(format!("tour-install-{}", theme.id)))
                        .ghost()
                        .xsmall()
                        .label("Install")
                        .on_click(move |_, _, cx| {
                            settings_gallery::install(install_theme.clone(), cx);
                        }),
                ),
        )
        .on_click(move |_, _, cx| {
            settings_gallery::preview(preview_theme.clone(), cx);
        })
        .into_any_element()
}

/// Compact icon theme card for Page 2, clicking applies via `icon_themes::choose`.
fn tour_icon_card(theme: &Arc<IconTheme>, chosen: bool, colors: &Colors) -> AnyElement {
    let icons = icon_themes::card_icons(theme)
        .into_iter()
        .take(5)
        .map(|drawn| {
            Icon::default()
                .path(icon_themes::unresolved(drawn))
                .size(px(15.))
                .text_color(colors.text_secondary)
        });
    let id = theme.id.clone();
    let selector = format!("tour-icon-theme-{id}");
    v_flex()
        .id(SharedString::from(selector.clone()))
        .debug_selector(move || selector.clone())
        .gap(px(4.))
        .cursor_pointer()
        .child(
            div()
                .p(px(2.))
                .rounded(px(8.))
                .border_2()
                .border_color(if chosen {
                    Hsla::from(colors.accent)
                } else {
                    transparent_black()
                })
                .child(
                    h_flex()
                        .h(px(36.))
                        .px(px(10.))
                        .gap(px(8.))
                        .items_center()
                        .rounded(px(6.))
                        .bg(colors.editor)
                        .border_1()
                        .border_color(colors.divider)
                        .children(icons),
                ),
        )
        .child(
            div()
                .text_center()
                .text_size(px(11.5))
                .text_color(if chosen {
                    colors.text
                } else {
                    colors.text_secondary
                })
                .when(chosen, |el| el.font_medium())
                .child(theme.name.clone()),
        )
        .on_click(move |_, _, cx| icon_themes::choose(Some(id.clone()), cx))
        .into_any_element()
}

// no-prop-test: layout; page state and sizing live in `welcome_tour_model`.
impl Render for WelcomeTourSheet {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = Colors::get(cx);
        let page = self.state.page;
        let viewport_h = f32::from(window.viewport_size().height);
        let max_body_h = body_max_height(viewport_h);

        let dots = h_flex()
            .debug_selector(|| "welcome-tour-dots".into())
            .gap(px(6.))
            .items_center()
            .children(TourPage::ALL.into_iter().map(|step| {
                let active = step == page;
                let ix = step.index();
                div()
                    .id(ElementId::NamedInteger("tour-dot".into(), ix as u64))
                    .debug_selector(move || format!("tour-dot-{ix}"))
                    .cursor_pointer()
                    .h(px(7.))
                    .w(px(if active { 20. } else { 7. }))
                    .rounded_full()
                    .bg(if active {
                        Hsla::from(colors.accent)
                    } else {
                        Hsla::from(colors.divider)
                    })
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.go_to(step, window, cx);
                    }))
            }));

        let content = match page {
            TourPage::Welcome => self.render_welcome(colors),
            TourPage::LookAndFeel => self.render_look_and_feel(colors, cx),
            TourPage::Writing => self.render_writing(colors, cx),
            TourPage::AccountAndSync => self.render_account_and_sync(colors, cx),
            TourPage::AiAndVoice => self.render_ai_and_voice(colors, cx),
            TourPage::AgentsAndMcp => self.render_agents_and_mcp(colors, cx),
        };

        v_flex()
            .id("welcome-tour-root")
            .debug_selector(move || page.selector().into())
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::on_key_down))
            .w_full()
            .gap(px(12.))
            .child(
                h_flex()
                    .w_full()
                    .pr(px(30.))
                    .justify_between()
                    .items_center()
                    .child(dots)
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(colors.text_muted)
                            .child(format!("{} of {}", page.index() + 1, TourPage::COUNT)),
                    ),
            )
            .when(!page.is_first(), |el| {
                el.child(
                    v_flex()
                        .gap(px(2.))
                        .child(
                            div()
                                .text_size(px(17.))
                                .font_semibold()
                                .text_color(colors.text)
                                .child(page.title()),
                        )
                        .child(
                            div()
                                .text_size(px(12.5))
                                .line_height(px(17.))
                                .text_color(colors.text_secondary)
                                .child(page.subtitle()),
                        ),
                )
            })
            .child(
                div()
                    .id("welcome-tour-body")
                    .w_full()
                    .max_h(px(max_body_h))
                    .overflow_y_scrollbar()
                    .track_scroll(&self.scroll)
                    .pr(px(4.))
                    .child(content),
            )
    }
}

/// Footer bar: Skip setup on the left, Back and Next/Done on the right.
struct TourFooter {
    sheet: Entity<WelcomeTourSheet>,
    _observing: Subscription,
}

// no-prop-test: layout and clicks; UI tests drive every button.
impl Render for TourFooter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let page = self.sheet.read(cx).state.page;
        let skip_target = self.sheet.clone();
        let back_target = self.sheet.clone();
        let next_target = self.sheet.clone();

        h_flex()
            .w_full()
            .items_center()
            .justify_between()
            .gap(px(8.))
            .child(
                Button::new("welcome-tour-skip")
                    .debug_selector(|| "welcome-tour-skip".into())
                    .ghost()
                    .small()
                    .label(page.skip_label())
                    .on_click(move |_, window, cx| {
                        skip_target.update(cx, |sheet, cx| sheet.skip(window, cx));
                    }),
            )
            .child(
                h_flex()
                    .gap(px(8.))
                    .when(!page.is_first(), |el| {
                        el.child(
                            Button::new("welcome-tour-back")
                                .debug_selector(|| "welcome-tour-back".into())
                                .outline()
                                .small()
                                .label("Back")
                                .on_click(move |_, window, cx| {
                                    back_target.update(cx, |sheet, cx| sheet.back(window, cx));
                                }),
                        )
                    })
                    .child(
                        Button::new("welcome-tour-next")
                            .debug_selector(|| "welcome-tour-next".into())
                            .primary()
                            .small()
                            .label(page.primary_label())
                            .on_click(move |_, window, cx| {
                                next_target
                                    .update(cx, |sheet, cx| sheet.next_or_finish(window, cx));
                            }),
                    ),
            )
    }
}

/// Workspace action wiring for Help › Welcome Tour and the command palette.
// no-prop-test: wiring; UI tests dispatch `ShowWelcomeTour`.
pub fn on_actions(el: Div, cx: &mut Context<crate::workspace::Workspace>) -> Div {
    el.on_action(cx.listener(|_, _: &ShowWelcomeTour, window, cx| {
        open(window, cx);
    }))
}

/// Opens the Welcome Tour automatically at launch if `welcome_tour_seen` is
/// `false`. Returns `Some(sheet)` when opened.
// no-prop-test: launch gate; `should_show_on_launch` is tested directly and
// UI tests verify first launch vs repeat launch.
pub fn offer_at_launch(window: &mut Window, cx: &mut App) -> Option<Entity<WelcomeTourSheet>> {
    let seen = cx
        .try_global::<AppState>()
        .is_none_or(|state| state.config.welcome_tour_seen);
    if !should_show_on_launch(seen) {
        return None;
    }
    Some(open(window, cx))
}

/// Opens the Welcome Tour starting at Page 1.
pub fn open(window: &mut Window, cx: &mut App) -> Entity<WelcomeTourSheet> {
    open_at(TourPage::Welcome, window, cx)
}

/// Opens the Welcome Tour at `initial_page`. Returns the sheet entity so tests
/// and snapshots can inspect or drive it.
// no-prop-test: opens a modal dialog; driven by `welcome_tour_tests` and snapshots.
pub fn open_at(
    initial_page: TourPage,
    window: &mut Window,
    cx: &mut App,
) -> Entity<WelcomeTourSheet> {
    if window.has_active_dialog(cx)
        && let Some(existing) = cx
            .try_global::<WelcomeTourOpen>()
            .and_then(|open| open.0.as_ref())
            .and_then(|weak| weak.upgrade())
    {
        existing.update(cx, |sheet, cx| sheet.go_to(initial_page, window, cx));
        return existing;
    }
    let sheet = cx.new(|cx| WelcomeTourSheet::new(initial_page, window, cx));
    cx.set_global(WelcomeTourOpen(Some(sheet.downgrade())));
    let footer = cx.new(|cx| TourFooter {
        _observing: cx.observe(&sheet, |_, _, cx| cx.notify()),
        sheet: sheet.clone(),
    });
    let focus = sheet.read(cx).focus_handle.clone();
    let first_build = std::cell::Cell::new(true);
    let body = sheet.clone();
    let ok_sheet = sheet.clone();
    window.open_alert_dialog(cx, move |alert, window, cx| {
        if first_build.replace(false) {
            window.focus(&focus, cx);
        }
        let width = dialog_width(f32::from(window.viewport_size().width));
        let top = (f32::from(window.viewport_size().height) * 0.06).clamp(12., 56.);
        let ok_sheet = ok_sheet.clone();
        alert
            .width(px(width))
            .margin_top(px(top))
            .close_button(true)
            .keyboard(true)
            .child(body.clone())
            .footer(footer.clone())
            .on_ok(move |_, window, cx| {
                ok_sheet.update(cx, |sheet, cx| {
                    if !sheet.input_is_focused(window, cx) {
                        sheet.next_or_finish(window, cx);
                    }
                });
                false
            })
            .on_cancel(|_, _, cx| {
                themes::stop_preview(cx);
                mark_seen(cx);
                true
            })
            .on_close(|_, _, cx| {
                themes::stop_preview(cx);
                mark_seen(cx);
            })
    });
    sheet
}
