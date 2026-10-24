//! Kanso: a Markdown notes app with a local REST API.
//!
//! `kanso` opens the app. Other commands run without a window; see `--help`.

// On Windows the release build is a GUI program, so opening Kanso shows no
// console window; the commands borrow the console they were run from.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod account_flow;
#[cfg(test)]
mod account_flow_tests;
mod account_page;
mod account_words;
mod ask;
mod ask_panel;
#[cfg(test)]
mod ask_tests;
mod assets;
mod board;
mod board_view;
mod book_colors;
mod book_icons;
mod calendar;
mod calendar_view;
mod capture;
mod capture_cli;
mod clipboard;
mod clock;
mod code_lang;
mod collab;
mod collab_model;
#[cfg(test)]
mod collab_tests;
mod collab_text;
mod commands;
mod config;
mod device_approval;
mod dialogs;
mod dictation;
mod due;
mod editor;
mod editor_footer;
mod empty_state;
mod export;
mod export_html;
mod export_style;
mod flavour;
mod fonts;
mod fuzzy;
mod gallery;
mod graph;
mod graph_view;
mod grouping;
mod history;
mod history_diff;
mod hotkey;
mod icon_theme_file;
mod icon_theme_import;
mod icon_themes;
mod image_size;
mod inspector;
mod journal;
mod keys;
#[cfg(test)]
mod latency_bench;
mod launch;
mod layout;
mod library;
mod library_import;
#[cfg(test)]
mod links_tests;
mod list_edit;
mod locked;
mod markdown_edit;
mod markdown_source;
mod mcp_cli;
mod mcp_setup;
mod model_fetch;
mod models;
mod nav_history;
#[cfg(test)]
mod nav_tests;
mod note_batch;
mod note_link;
mod note_list;
mod note_meaning;
mod note_menu;
mod note_window;
#[cfg(test)]
mod note_window_tests;
mod notebook_lock;
mod notify_support;
mod ocr;
mod origin;
mod pairing;
mod palette;
mod paths;
mod pouch_import;
mod preview;
mod preview_chunks;
mod preview_rich;
mod publish;
mod publish_site;
mod query_view_ui;
mod related;
mod reminders;
mod replace;
mod selection;
mod services;
mod settings;
mod settings_ai;
mod settings_capture;
mod settings_discovery;
mod settings_gallery;
mod settings_themes;
mod settings_voice;
#[cfg(test)]
mod settings_window_tests;
mod share_sheet;
mod shared_inbox;
mod sidebar;
mod site;
mod smart_paste;
#[cfg(test)]
mod snapshots;
mod spellcheck;
// Tests check with a word list instead.
#[cfg(all(target_os = "macos", not(test)))]
mod spellcheck_macos;
mod split;
mod sync_nudge;
mod tab_bar;
#[cfg(test)]
mod tab_bar_tests;
#[cfg(test)]
mod tab_tests;
mod table_format;
mod tabs;
mod tag_complete;
mod tasks_view;
mod text_match;
mod text_stats;
mod theme;
mod theme_file;
mod theme_import;
mod themes;
mod title_bar;
mod touch_id;
mod transcribe;
#[cfg(test)]
mod ui_tests;
mod updates;
mod vim;
mod voice;
mod voice_commands;
mod voice_state;
mod wikilinks;
mod word_goal;
mod workspace;
mod zoom;

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context as _, Result, bail};
use gpui_kit::*;
use kanso_store::{BackupPolicy, Store, lock_data_dir};

use crate::config::AppConfig;
use crate::services::Services;
use crate::workspace::{AppState, Quit, Workspace};

const USAGE: &str = "\
Usage: kanso [command] [--data-dir <dir>]

Commands:
  (none)            Open the app
  serve             Run only the local API (REST and /mcp), without a
                    window
  import <kind> [path]
                    Import a folder of Markdown or another app's export
                    [--dry-run]; Apple Notes finds its library itself.
                    `kanso import help` lists what each takes
  import pouchdb    Import the PouchDB notes library found on this computer
                    [--library-dir <dir>] [--credentials]
                    --credentials also takes its local API port, username
                    and password (macOS asks to allow Keychain access)
  capture [text]    Save a note to the Inbox, or [--book <name or id>];
                    with no text, from standard input. Works with
                    Kanso open or closed
  mcp               Serve Kanso's MCP tools over standard input and
                    output, for AI agents that start a program; needs
                    Kanso open with its local API on
  theme             Work with themes; `kanso theme help` lists how
  icon-theme        Work with icon themes; `kanso icon-theme help` lists
                    how
  help              Show this message

";

#[cfg(target_os = "macos")]
const DATA_DIR_NOTE: &str = "The data folder defaults to ~/Library/Application Support/Kanso
(or $KANSO_DATA_DIR).";
#[cfg(target_os = "windows")]
const DATA_DIR_NOTE: &str = "The data folder defaults to %APPDATA%\\Kanso (or %KANSO_DATA_DIR%).";
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const DATA_DIR_NOTE: &str = "The data folder defaults to $XDG_DATA_HOME/kanso, ~/.local/share/kanso when\nthat isn't set (or $KANSO_DATA_DIR).";

struct Args {
    command: Option<String>,
    data_dir: PathBuf,
    rest: Vec<String>,
    /// Opened by an update: the Kanso that installed it may still be
    /// quitting, so wait for the data folder.
    after_update: bool,
}

fn parse_args() -> Result<Args> {
    parse_args_from(std::env::args().skip(1))
}

/// Reads the command line after the program name.
fn parse_args_from(args: impl IntoIterator<Item = String>) -> Result<Args> {
    let mut command = None;
    let mut data_dir = None;
    let mut rest = Vec::new();
    let mut after_update = false;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--data-dir" => {
                data_dir = Some(PathBuf::from(
                    args.next().context("--data-dir needs a folder")?,
                ))
            }
            "-h" | "--help" => command = Some("help".to_owned()),
            "--after-update" => after_update = true,
            // macOS adds this when launching from Finder.
            arg if arg.starts_with("-psn_") => {}
            _ if command.is_none() && !arg.starts_with('-') => command = Some(arg),
            _ => rest.push(arg),
        }
    }
    Ok(Args {
        command,
        data_dir: data_dir.unwrap_or_else(paths::default_data_dir),
        rest,
        after_update,
    })
}

fn main() -> Result<()> {
    #[cfg(windows)]
    if std::env::args().len() > 1 {
        attach_parent_console();
    }
    let args = parse_args()?;
    match args.command.as_deref() {
        None => run_app(args.data_dir, args.after_update),
        Some("serve") => serve(args.data_dir),
        Some("import") => library_import::run_cli(&args.data_dir, &args.rest),
        Some("theme") => theme_import::run_cli(&args.data_dir, &args.rest),
        Some("icon-theme") => icon_theme_import::run_cli(&args.data_dir, &args.rest),
        Some("capture") => capture_cli::run_cli(&args.data_dir, &args.rest),
        Some("mcp") => mcp_cli::run_cli(&args.data_dir, &args.rest),
        Some("help") => {
            println!("{USAGE}{DATA_DIR_NOTE}");
            Ok(())
        }
        Some(other) => bail!("unknown command `{other}`\n\n{USAGE}{DATA_DIR_NOTE}"),
    }
}

/// Writes the commands' output to the console Kanso was started from, if
/// any: a GUI program on Windows has none of its own.
#[cfg(windows)]
fn attach_parent_console() {
    use windows_sys::Win32::System::Console::{ATTACH_PARENT_PROCESS, AttachConsole};
    // SAFETY: takes a process ID and only fails when there's no console to
    // attach to or one is attached already, which leaves things as they were.
    unsafe {
        AttachConsole(ATTACH_PARENT_PROCESS);
    }
}

/// Runs the local API until the process is stopped.
fn serve(data_dir: PathBuf) -> Result<()> {
    std::fs::create_dir_all(&data_dir)?;
    let Some(_lock) = lock_data_dir(&data_dir)? else {
        bail!(
            "Kanso is already running with {}; its API is already on",
            data_dir.display()
        );
    };
    let config = load_config(&data_dir)?;
    let server = services::server_config(&config.server).map_err(anyhow::Error::msg)?;
    let store = Arc::new(Store::open(paths::db_path(&data_dir))?);
    let handle = kanso_server::spawn(store, server);
    match handle.bind_result() {
        Ok(addr) => eprintln!("Kanso API on http://{addr} (MCP at /mcp). Ctrl-C stops it."),
        Err(err) => bail!(
            "couldn't listen on {}:{}: {err}",
            config.server.bind_address,
            config.server.port
        ),
    }
    loop {
        std::thread::park();
    }
}

/// Loads the settings, bringing an older file up to date and filling in API
/// credentials on first run.
fn load_config(data_dir: &std::path::Path) -> Result<AppConfig> {
    let path = paths::config_path(data_dir);
    let mut config =
        AppConfig::load(&path).with_context(|| format!("reading {}", path.display()))?;
    let migrated = config.migrate();
    if config.ensure_credentials() || migrated {
        config.save(&path)?;
    }
    Ok(config)
}

/// How long a Kanso opened by an update waits for the one that installed
/// it to let go of the data folder.
const AFTER_UPDATE_WAIT: std::time::Duration = std::time::Duration::from_secs(20);

fn run_app(data_dir: PathBuf, after_update: bool) -> Result<()> {
    std::fs::create_dir_all(&data_dir)
        .with_context(|| format!("creating {}", data_dir.display()))?;
    let mut lock = lock_data_dir(&data_dir)?;
    let started = std::time::Instant::now();
    while lock.is_none() && after_update && started.elapsed() < AFTER_UPDATE_WAIT {
        std::thread::sleep(std::time::Duration::from_millis(200));
        lock = lock_data_dir(&data_dir)?;
    }
    let Some(lock) = lock else {
        bail!(
            "Kanso is already running with the data folder {}",
            data_dir.display()
        );
    };
    let config = load_config(&data_dir)?;
    let store = Arc::new(Store::open(paths::db_path(&data_dir))?);

    // A daily snapshot of the database, off the main thread.
    let backup_store = store.clone();
    let policy = BackupPolicy::daily(paths::backups_dir(&data_dir));
    std::thread::spawn(move || {
        if let Err(err) = backup_store.backup_if_due(&policy, kanso_core::now_ms()) {
            eprintln!("kanso: backup failed: {err}");
        }
    });

    let app = application().with_assets(assets::KansoAssets);
    // The Dock's icon brings the main window back, even while notes are
    // open in windows of their own.
    app.on_reopen(|cx| {
        if workspace::main_workspace(cx).is_none() {
            open_main_window(cx);
        }
    });
    app.run(move |cx| {
        gpui_kit::init(cx);
        cx.set_global(AppState {
            store,
            data_dir,
            config,
        });
        // The real servers and the Keychain; tests and snapshots leave both
        // out.
        let mut services = Services::default();
        services.backend = Some(Arc::new(kanso_sync::HttpBackend::new()));
        services.shares = Some(Arc::new(kanso_sync::share::HttpShares));
        services.collab = Some(Arc::new(kanso_collab::http::HttpCollab));
        // Kanso Dev keeps its secrets under its own service.
        services.secrets = Some(Arc::new(flavour::CURRENT.keychain()));
        cx.set_global(services);
        cx.set_global(touch_id::TouchId(Arc::new(touch_id::SystemTouchId)));
        // The saved zoom, before the first window is drawn.
        zoom::set(cx.global::<AppState>().config.appearance.zoom, cx);
        zoom::register(cx);
        kanso_render::set_mermaid_engine(
            cx.global::<AppState>()
                .config
                .appearance
                .diagram_renderer
                .engine(),
        );
        workspace::bind_keys(cx);
        note_list::bind_keys(cx);
        note_window::bind_keys(cx);
        capture::bind_keys(cx);
        graph_view::bind_keys(cx);
        board_view::bind_keys(cx);
        workspace::set_menus(cx);
        cx.on_action(|_: &Quit, cx| quit(cx));
        watch_for_quit(cx);
        settings::register_app_actions(cx, open_main_window);
        updates::register_app_actions(cx);
        services::apply_server_settings(cx);
        crate::settings_gallery::init(cx);
        crate::settings_gallery::set_log_dir(&cx.global::<AppState>().data_dir);
        crate::settings_gallery::schedule(cx);
        themes::watch(cx);
        // The chosen icon theme draws every icon, from the first window on.
        icon_themes::init(cx);
        icon_themes::watch(cx);
        // Signed in again once the config, the binding and the Keychain
        // agree; sync starts then.
        account_flow::restore(cx);
        updates::set_log_dir(&cx.global::<AppState>().data_dir.clone());
        updates::init(cx);
        hotkey::init(cx);
        let store = cx.global::<AppState>().store.clone();
        reminders::init(store, open_main_window, cx);
        // Discovery, in the background: related notes and text in images.
        related::init(cx);
        voice::init(cx);
        ocr::init(cx);
        open_main_window(cx);
        note_window::restore(cx);
        cx.activate(true);
        // Held until the process exits so no second Kanso opens this folder.
        let _ = &lock;
        std::mem::forget(lock);
    });
    Ok(())
}

fn open_main_window(cx: &mut App) {
    let bounds = Bounds::centered(None, size(px(1440.), px(900.)), cx);
    // On macOS the panes run to the top edge; the traffic lights sit in the
    // leftmost pane's top band, and the app moves the window from the bands
    // itself (see `layout::BandDrag`). On Windows Kanso draws the whole
    // title bar, menus and window buttons included (see `title_bar`). On
    // Linux the system's title bar stays, with Kanso's menus under it.
    #[cfg(target_os = "macos")]
    let titlebar = TitlebarOptions {
        title: Some(flavour::CURRENT.name.into()),
        appears_transparent: true,
        traffic_light_position: Some(point(
            px(layout::TRAFFIC_LIGHTS.0),
            px(layout::TRAFFIC_LIGHTS.1),
        )),
    };
    #[cfg(not(target_os = "macos"))]
    let titlebar = TitlebarOptions {
        title: Some(flavour::CURRENT.name.into()),
        appears_transparent: cfg!(target_os = "windows"),
        traffic_light_position: None,
    };
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: Some(titlebar),
        app_owns_titlebar_drag: cfg!(target_os = "macos"),
        window_min_size: Some(size(px(1100.), px(720.))),
        // Matches kanso.desktop, so the dock and the window list show its
        // name and icon.
        #[cfg(target_os = "linux")]
        app_id: Some(flavour::CURRENT.xdg_name.into()),
        #[cfg(target_os = "linux")]
        window_decorations: Some(WindowDecorations::Server),
        ..Default::default()
    };
    let opened = gpui_kit::open_window(options, cx, |window, cx| {
        themes::apply(Some(window), cx);
        cx.new(|cx| Workspace::new(window, cx))
    });
    match opened {
        Ok((window, view)) => {
            window
                .update(cx, |_, window, cx| {
                    let editor = view.read(cx).editor.clone();
                    window.on_window_should_close(cx, move |_, cx| {
                        editor.update(cx, |editor, cx| editor.flush(cx));
                        true
                    });
                    offer_first_import(window, cx);
                    updates::announce(window, cx);
                })
                .ok();
        }
        Err(err) => eprintln!("kanso: couldn't open a window: {err:#}"),
    }
}

/// On a first run with an empty library, offers to bring notes over from
/// a PouchDB notes library found on this computer.
fn offer_first_import(window: &mut Window, cx: &mut App) {
    let state = cx.global::<AppState>();
    let empty = state
        .store
        .counts()
        .is_ok_and(|counts| counts.get(&kanso_core::DocType::Note).copied().unwrap_or(0) == 0);
    if !empty
        || state.store.meta("import_offered").ok().flatten().is_some()
        || pouch_import::find_library().is_none()
    {
        return;
    }
    state.store.set_meta("import_offered", "1").ok();
    dialogs::confirm(
        "Import your notes?",
        "Kanso found a notes library on this computer. Its notes, notebooks, tags and attachments can be copied in now; its files are only read.",
        "Import",
        window,
        cx,
        pouch_import::start_import,
    );
}

/// Cleans up on any quit, including ones without the Quit action (the
/// Dock, logging out): a recovery key copied in the last minute leaves the
/// clipboard. Open notes save themselves.
fn watch_for_quit(cx: &mut App) {
    cx.on_app_quit(|cx| {
        services::stop_server(cx);
        // A staged update goes in now, and Kanso opens again if Restart
        // asked for it.
        updates::install_on_quit(cx);
        AppState::save_pending_config(cx);
        clipboard::clear_secret_on_quit();
        async {}
    })
    .detach();
}

/// Saves open notes, then quits. The Quit action arrives while its window
/// is being updated, and a window can't be reached from inside its own
/// update, so the work waits until that update ends.
fn quit(cx: &mut App) {
    cx.defer(|cx| {
        workspace::for_each_editor(cx, |editor, cx| editor.flush(cx));
        services::stop_server(cx);
        cx.quit();
    });
}
