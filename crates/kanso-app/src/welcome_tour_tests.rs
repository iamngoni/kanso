//! GPUI tests for the six-page Welcome Tour (issue #510).

use std::sync::Arc;

use gpui_kit::component::WindowExt as _;
use gpui_kit::{
    AnyWindowHandle, AppContext as _, Bounds, Entity, TestAppContext, VisualTestContext,
    WindowBounds, WindowOptions, px, size,
};
use kanso_ai::Provider as AiProvider;
use kanso_store::Store;
use kanso_sync::MemorySecrets;

use crate::commands;
use crate::config::{AppConfig, ThemePreference, ViewMode, VoiceEngine};
use crate::icon_themes;
use crate::mcp_setup::Agent;
use crate::related::Related;
use crate::services::{AccountState, Services};
use crate::settings;
use crate::settings_ai;
use crate::settings_themes;
use crate::themes;
use crate::voice::{ModelState, Voice};
use crate::welcome_tour::{self, ShowWelcomeTour, WelcomeTourSheet};
use crate::welcome_tour_model::{DetectedAgent, TourPage};
use crate::workspace::{AppState, Workspace};

struct Rig {
    window: AnyWindowHandle,
    _workspace: Entity<Workspace>,
    dir: tempfile::TempDir,
}

fn rig_with(cx: &mut TestAppContext, config: AppConfig, window_size: (f32, f32)) -> Rig {
    cx.executor().allow_parking();
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    let data_dir = dir.path().to_path_buf();
    let (window, workspace): (_, Entity<Workspace>) = cx.update(|cx| {
        gpui_kit::init(cx);
        let bounds = Bounds::centered(None, size(px(window_size.0), px(window_size.1)), cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            ..Default::default()
        };
        cx.set_global(AppState {
            store,
            data_dir,
            config,
        });
        let mut services = Services::default();
        services.secrets = Some(Arc::new(MemorySecrets::new()));
        cx.set_global(services);
        let mut voice = Voice::default();
        voice.model = ModelState::Missing;
        cx.set_global(voice);
        cx.set_global(Related::default());
        icon_themes::init(cx);
        gpui_kit::open_window(options, cx, |window, cx| {
            themes::apply(Some(window), cx);
            cx.new(|cx| Workspace::new(window, cx))
        })
        .unwrap()
    });
    cx.run_until_parked();
    Rig {
        window,
        _workspace: workspace,
        dir,
    }
}

fn rig(cx: &mut TestAppContext) -> Rig {
    rig_with(cx, AppConfig::default(), (1280., 800.))
}

/// Command palette exposes "Welcome tour" in the App group with no prerequisites.
#[test]
fn command_palette_includes_welcome_tour() {
    let cmd = commands::all()
        .iter()
        .find(|c| c.id == "welcome-tour")
        .expect("welcome-tour command");
    assert_eq!(cmd.label, "Welcome tour");
    assert_eq!(cmd.group, commands::Group::App);
    assert_eq!(cmd.needs, commands::Needs::Nothing);
    assert!(cmd.action().is_some_and(|a| a.partial_eq(&ShowWelcomeTour)));
}

/// New install and existing users upgrading (`welcome_tour_seen == false`) see
/// the tour on first launch; once seen, subsequent launches do not pop it up;
/// Help › Welcome Tour (`ShowWelcomeTour`) reopens it any time.
#[gpui_kit::test]
fn launch_visibility_skip_and_reopen(cx: &mut TestAppContext) {
    let rig = rig(cx);

    // First launch: unseen => opens at Page 1.
    let sheet = rig
        .window
        .update(cx, |_, window, cx| {
            welcome_tour::offer_at_launch(window, cx)
        })
        .unwrap()
        .expect("tour should open when unseen");
    cx.run_until_parked();
    assert_eq!(cx.read(|cx| sheet.read(cx).page()), TourPage::Welcome);
    assert!(
        rig.window
            .update(cx, |_, window, cx| window.has_active_dialog(cx))
            .unwrap()
    );

    // Skipping marks `welcome_tour_seen = true`, persists to config.json, and closes.
    rig.window
        .update(cx, |_, window, cx| {
            sheet.update(cx, |s, cx| s.skip(window, cx));
        })
        .unwrap();
    cx.run_until_parked();
    assert!(cx.read(|cx| cx.global::<AppState>().config.welcome_tour_seen));
    let persisted = AppConfig::load(&crate::paths::config_path(rig.dir.path())).unwrap();
    assert!(persisted.welcome_tour_seen);
    assert!(
        !rig.window
            .update(cx, |_, window, cx| window.has_active_dialog(cx))
            .unwrap()
    );

    // Second launch: already seen => does not open.
    let second = rig
        .window
        .update(cx, |_, window, cx| {
            welcome_tour::offer_at_launch(window, cx)
        })
        .unwrap();
    assert!(second.is_none());

    // Help menu / command palette action reopens it even after seen.
    rig.window
        .update(cx, |_, window, cx| {
            window.dispatch_action(Box::new(ShowWelcomeTour), cx);
        })
        .unwrap();
    cx.run_until_parked();
    assert!(
        rig.window
            .update(cx, |_, window, cx| window.has_active_dialog(cx))
            .unwrap()
    );
}

/// Closing the window while the tour is open marks `welcome_tour_seen = true`.
#[gpui_kit::test]
fn closing_window_marks_tour_seen(cx: &mut TestAppContext) {
    let rig = rig(cx);
    let _sheet = rig
        .window
        .update(cx, |_, window, cx| {
            welcome_tour::offer_at_launch(window, cx)
        })
        .unwrap()
        .expect("opens");
    cx.run_until_parked();
    assert!(!cx.read(|cx| cx.global::<AppState>().config.welcome_tour_seen));

    cx.update(welcome_tour::on_window_close);
    assert!(cx.read(|cx| cx.global::<AppState>().config.welcome_tour_seen));
}

/// Keyboard: Return advances through all 6 pages and finishes on the last;
/// Escape skips immediately and marks `welcome_tour_seen = true`.
#[gpui_kit::test]
fn keyboard_return_and_escape(cx: &mut TestAppContext) {
    let rig = rig(cx);
    let sheet = rig
        .window
        .update(cx, |_, window, cx| welcome_tour::open(window, cx))
        .unwrap();
    cx.run_until_parked();

    let mut visual = VisualTestContext::from_window(rig.window, cx);

    // Return walks pages 1 -> 2 -> 3 -> 4 -> 5 -> 6.
    for expected in &TourPage::ALL[1..] {
        visual.simulate_keystrokes("enter");
        visual.run_until_parked();
        assert_eq!(visual.read(|cx| sheet.read(cx).page()), *expected);
    }

    // Return on the last page finishes and marks seen.
    visual.simulate_keystrokes("enter");
    visual.run_until_parked();
    assert!(visual.read(|cx| cx.global::<AppState>().config.welcome_tour_seen));
    assert!(
        !rig.window
            .update(&mut visual, |_, window, cx| window.has_active_dialog(cx))
            .unwrap()
    );

    // Reopen with seen reset to false and press Escape -> closes & marks seen.
    visual.update(|_, cx| {
        AppState::update_config(cx, |c| c.welcome_tour_seen = false);
    });
    let _sheet2 = rig
        .window
        .update(&mut visual, |_, window, cx| welcome_tour::open(window, cx))
        .unwrap();
    visual.run_until_parked();
    visual.simulate_keystrokes("escape");
    visual.run_until_parked();
    assert!(visual.read(|cx| cx.global::<AppState>().config.welcome_tour_seen));
    assert!(
        !rig.window
            .update(&mut visual, |_, window, cx| window.has_active_dialog(cx))
            .unwrap()
    );
}

/// Drives every page's settings controls and verifies each writes through the
/// real Settings setter into `AppConfig` / `SecretStore` / `KeyStore`.
#[gpui_kit::test]
fn every_page_writes_its_real_settings(cx: &mut TestAppContext) {
    let rig = rig(cx);
    let sheet: Entity<WelcomeTourSheet> = rig
        .window
        .update(cx, |_, window, cx| welcome_tour::open(window, cx))
        .unwrap();
    cx.run_until_parked();

    // Page 2: Look and feel (mode, theme card, icon theme).
    rig.window
        .update(cx, |_, window, cx| {
            sheet.update(cx, |s, cx| s.go_to(TourPage::LookAndFeel, window, cx));
            settings_themes::apply_mode(ThemePreference::Light, window, cx);
            settings_themes::pick_ui("Nord", true, window, cx);
            if let Some(alt) = cx
                .try_global::<icon_themes::IconThemes>()
                .and_then(|t| t.entries.iter().find(|e| e.id != icon_themes::DEFAULT_ID))
                .map(|e| e.id.clone())
            {
                icon_themes::choose(Some(alt), cx);
            }
        })
        .unwrap();
    cx.run_until_parked();
    cx.read(|cx| {
        let appearance = &cx.global::<AppState>().config.appearance;
        assert!(!appearance.themes.follow_system);
        assert_eq!(appearance.themes.ui, "Nord");
        assert_eq!(appearance.theme, ThemePreference::Dark);
    });

    // Page 3: Writing (font family, font size stepper, readable width, default view).
    let before_size = cx.read(|cx| cx.global::<AppState>().config.appearance.editor_font_size);
    rig.window
        .update(cx, |_, window, cx| {
            sheet.update(cx, |s, cx| s.go_to(TourPage::Writing, window, cx));
            settings::set_editor_font_family(Some("Georgia".into()), cx);
            settings::step_editor_size(2, cx);
            settings::set_readable_width(true, cx);
            settings::set_view_mode(ViewMode::Preview, cx);
        })
        .unwrap();
    cx.run_until_parked();
    cx.read(|cx| {
        let appearance = &cx.global::<AppState>().config.appearance;
        assert_eq!(appearance.editor_font_family.as_deref(), Some("Georgia"));
        assert_eq!(appearance.editor_font_size, before_size + 2.);
        assert!(appearance.readable_width);
        assert_eq!(appearance.view_mode, ViewMode::Preview);
    });

    // Page 4: Account and sync (self-hosted URL, default server reset, stay local-only).
    rig.window
        .update(cx, |_, window, cx| {
            sheet.update(cx, |s, cx| s.go_to(TourPage::AccountAndSync, window, cx));
            // Set a valid self-hosted URL.
            let url_input = cx.read_entity(&sheet, |s, _| s.server_url_for_test());
            url_input.update(cx, |input, cx| {
                input.set_value("https://notes.example.com", window, cx)
            });
            sheet.update(cx, |s, cx| s.apply_server_url(window, cx));
        })
        .unwrap();
    cx.run_until_parked();
    assert_eq!(
        cx.read(|cx| cx.global::<AppState>().config.sync.server.clone()),
        Some("https://notes.example.com".to_owned())
    );
    // Reset to default server, then Stay local-only (advances to Page 5).
    rig.window
        .update(cx, |_, window, cx| {
            sheet.update(cx, |s, cx| s.use_default_server(window, cx));
            sheet.update(cx, |s, cx| s.stay_local_only(window, cx));
        })
        .unwrap();
    cx.run_until_parked();
    assert_eq!(
        cx.read(|cx| cx.global::<AppState>().config.sync.server.clone()),
        None
    );
    assert!(!cx.read(|cx| cx.global::<AppState>().config.sync.enabled));
    assert_eq!(cx.read(|cx| sheet.read(cx).page()), TourPage::AiAndVoice);

    // Page 5: AI and voice (provider, key saved to SecretStore & AI enabled, voice engine, semantic search).
    rig.window
        .update(cx, |_, window, cx| {
            sheet.update(cx, |s, cx| {
                s.set_ai_provider(AiProvider::Anthropic, window, cx)
            });
            let key_input = cx.read_entity(&sheet, |s, _| s.ai_key_for_test());
            key_input.update(cx, |input, cx| {
                input.set_value("sk-ant-test-123", window, cx)
            });
            sheet.update(cx, |s, cx| s.save_ai_key(window, cx));
            AppState::update_config(cx, |c| c.voice.engine = VoiceEngine::OnDevice);
            AppState::update_config(cx, |c| c.discovery.related_notes = true);
        })
        .unwrap();
    cx.run_until_parked();
    cx.read(|cx| {
        let cfg = &cx.global::<AppState>().config;
        assert!(cfg.ai.enabled);
        assert_eq!(cfg.ai.provider, AiProvider::Anthropic);
        assert_eq!(
            settings_ai::read_key(cx, AiProvider::Anthropic).as_deref(),
            Some("sk-ant-test-123")
        );
        assert_eq!(cfg.voice.engine, VoiceEngine::OnDevice);
        assert!(cfg.discovery.related_notes);
    });

    // Page 6: Agents and MCP (toggle server, one-click Add to Claude Code mints a key & snippet).
    rig.window
        .update(cx, |_, window, cx| {
            sheet.update(cx, |s, cx| {
                s.go_to(TourPage::AgentsAndMcp, window, cx);
                s.set_detected_agents_for_test(
                    vec![
                        DetectedAgent {
                            agent: Agent::ClaudeCode,
                            detected: true,
                        },
                        DetectedAgent {
                            agent: Agent::Codex,
                            detected: true,
                        },
                    ],
                    cx,
                );
            });
            settings::set_server_enabled(false, cx);
        })
        .unwrap();
    cx.run_until_parked();
    assert!(!cx.read(|cx| cx.global::<AppState>().config.server.enabled));

    rig.window
        .update(cx, |_, window, cx| {
            sheet.update(cx, |s, cx| s.add_to_agent(Agent::ClaudeCode, window, cx));
        })
        .unwrap();
    cx.run_until_parked();
    // Server turned back on and an API key was minted for Claude Code.
    assert!(cx.read(|cx| cx.global::<AppState>().config.server.enabled));
    assert!(cx.read(|cx| sheet.read(cx).has_agent_snippet_for_test(Agent::ClaudeCode)));
    let keys = cx.update(|cx| crate::services::key_store(cx).unwrap().list().unwrap());
    assert!(keys.iter().any(|k| k.client == Agent::ClaudeCode.id()));

    // Finish on Page 6 closes the tour and marks seen.
    rig.window
        .update(cx, |_, window, cx| {
            sheet.update(cx, |s, cx| s.next_or_finish(window, cx));
        })
        .unwrap();
    cx.run_until_parked();
    assert!(cx.read(|cx| cx.global::<AppState>().config.welcome_tour_seen));
    assert!(
        !rig.window
            .update(cx, |_, window, cx| window.has_active_dialog(cx))
            .unwrap()
    );
}

/// Signed-in users on Page 4 see their account and can advance cleanly.
#[gpui_kit::test]
fn signed_in_user_renders_account_page_and_skips_ahead(cx: &mut TestAppContext) {
    let rig = rig(cx);
    cx.update(|cx| {
        cx.global_mut::<Services>().account = AccountState::SignedIn {
            username: "ada".to_owned(),
        };
    });
    let sheet = rig
        .window
        .update(cx, |_, window, cx| {
            welcome_tour::open_at(TourPage::AccountAndSync, window, cx)
        })
        .unwrap();
    cx.run_until_parked();
    assert_eq!(
        cx.read(|cx| sheet.read(cx).page()),
        TourPage::AccountAndSync
    );

    rig.window
        .update(cx, |_, window, cx| {
            sheet.update(cx, |s, cx| s.next_or_finish(window, cx));
        })
        .unwrap();
    cx.run_until_parked();
    assert_eq!(cx.read(|cx| sheet.read(cx).page()), TourPage::AiAndVoice);
}

/// Renders all six pages cleanly at the smallest window size (520 × 420).
#[gpui_kit::test]
fn renders_all_pages_at_smallest_window_size(cx: &mut TestAppContext) {
    let rig = rig_with(cx, AppConfig::default(), crate::layout::WINDOW_MIN_SIZE);
    let sheet = rig
        .window
        .update(cx, |_, window, cx| welcome_tour::open(window, cx))
        .unwrap();
    cx.run_until_parked();
    for page in TourPage::ALL {
        rig.window
            .update(cx, |_, window, cx| {
                sheet.update(cx, |s, cx| s.go_to(page, window, cx));
            })
            .unwrap();
        cx.run_until_parked();
        assert_eq!(cx.read(|cx| sheet.read(cx).page()), page);
    }
}
