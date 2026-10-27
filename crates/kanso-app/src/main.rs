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
mod agent_permission;
#[cfg(test)]
mod agent_permission_tests;
mod agent_registry;
mod agent_scope;
mod ai_agent;
mod ai_feature;
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
mod diff_view;
mod due;
mod editor;
mod editor_footer;
mod empty_state;
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
mod open_documents;
mod origin;
mod pairing;
mod palette;
mod passkey_accounts;
mod passkey_ceremony;
mod paths;
mod plugin_panel;
mod plugin_panel_model;
mod plugin_sheets;
mod plugin_sheets_model;
mod plugins;
mod plugins_model;
mod pouch_import;
mod preview;
mod preview_chunks;
mod preview_rich;
mod preview_tasks;
#[allow(dead_code)]
mod proof_sheet;
mod provider_browser;
mod publish;
mod publish_site;
mod query_view_ui;
mod related;
mod reminders;
mod replace;
mod review_queue;
#[cfg(test)]
mod review_tests;
mod security_key_flow;
mod selection;
mod services;
mod settings;
mod settings_ai;
mod settings_capture;
mod settings_discovery;
mod settings_gallery;
mod settings_plugins;
mod settings_plugins_model;
mod settings_themes;
mod settings_voice;
#[cfg(test)]
mod settings_window_tests;
mod share_sheet;
mod shared_inbox;
mod sidebar;
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
// macOS finishes Apple's sign-in in its own sheet: only the forwarding of a
// `kanso://` argument and the tests reach the router there.
#[cfg_attr(target_os = "macos", allow(dead_code))]
mod url_callback;
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
use kanso_store::{Store, lock_data_dir};

use crate::config::AppConfig;
use crate::services::Services;
use crate::workspace::{AppState, Quit, Workspace};

const USAGE: &str = "\
Usage: kanso [command] [--data-dir <dir>] [--safe-mode]
       kanso <file.md> [more files]

--safe-mode (or Shift held as Kanso opens) keeps every plugin off for the
session.

Commands:
  (none)            Open the app
  <file.md> ...     Open Markdown or text files as notes in the Inbox,
                    in the running Kanso if there is one
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
  plugin            Work with plugins; `kanso plugin help` lists how
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
    /// Markdown or text files to open: every argument was one.
    files: Vec<PathBuf>,
    /// Opened by an update: the Kanso that installed it may still be
    /// quitting, so wait for the data folder.
    after_update: bool,
}

fn parse_args() -> Result<Args> {
    parse_args_from(std::env::args().skip(1))
}

/// Reads the command line after the program name.
///
/// A `kanso://` link counts only when it is the whole command line, the way
/// the desktop entry and the installer pass it (`kanso <url>`): the
/// operating system passes the link, so flags beside it, such as a
/// `--data-dir` naming the folder to forward to, can't come from Kanso.
///
/// Arguments that are all existing Markdown or text files (or `file://` URLs
/// for them, which is how a Linux file manager passes them) are files to
/// open, and mean nothing else.
fn parse_args_from(args: impl IntoIterator<Item = String>) -> Result<Args> {
    let mut command = None;
    let mut data_dir = None;
    let mut rest = Vec::new();
    let mut after_update = false;
    let args: Vec<String> = args.into_iter().collect();
    if let Some(files) = file_arguments(&args) {
        return Ok(Args {
            command: None,
            data_dir: paths::default_data_dir(),
            rest,
            files,
            after_update,
        });
    }
    let mut args = args.into_iter().peekable();
    if let Some(url) = args.next_if(|arg| url_callback::is_callback_argument(arg)) {
        if args.next().is_some() {
            bail!("a kanso:// link is passed on its own");
        }
        return Ok(Args {
            command: Some(url),
            data_dir: paths::default_data_dir(),
            rest,
            files: Vec::new(),
            after_update,
        });
    }
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
            arg if command.is_none() && url_callback::is_callback_argument(arg) => {
                bail!("a kanso:// link is passed on its own")
            }
            _ if command.is_none() && !arg.starts_with('-') => command = Some(arg),
            _ => rest.push(arg),
        }
    }
    Ok(Args {
        command,
        data_dir: data_dir.unwrap_or_else(paths::default_data_dir),
        rest,
        files: Vec::new(),
        after_update,
    })
}

/// The absolute paths of `args` when there is at least one and every one is
/// an existing file named like a note, or a `file://` URL for one.
fn file_arguments(args: &[String]) -> Option<Vec<PathBuf>> {
    if args.is_empty() {
        return None;
    }
    args.iter()
        .map(|arg| {
            let path = if arg
                .get(..7)
                .is_some_and(|p| p.eq_ignore_ascii_case("file://"))
            {
                open_documents::paths_from_urls(std::slice::from_ref(arg))
                    .pop()
                    .filter(|path| path.is_absolute())?
            } else {
                std::path::absolute(arg).ok()?
            };
            open_documents::is_note_file(&path).then_some(path)
        })
        .collect()
}

fn main() -> Result<()> {
    #[cfg(windows)]
    if std::env::args().len() > 1 {
        attach_parent_console();
    }
    let args = parse_args()?;
    match args.command.as_deref() {
        None if forwarded_files(&args.data_dir, &args.files) => Ok(()),
        None => {
            let safe_mode = plugins::safe_mode_requested(&args.rest);
            run_app(args.data_dir, args.after_update, safe_mode, args.files)
        }
        Some(url) if url_callback::is_callback_argument(url) => {
            forward_callback(&args.data_dir, url);
            Ok(())
        }
        Some("serve") => serve(args.data_dir),
        Some("import") => library_import::run_cli(&args.data_dir, &args.rest),
        Some("theme") => theme_import::run_cli(&args.data_dir, &args.rest),
        Some("icon-theme") => icon_theme_import::run_cli(&args.data_dir, &args.rest),
        Some("capture") => capture_cli::run_cli(&args.data_dir, &args.rest),
        Some("mcp") => mcp_cli::run_cli(&args.data_dir, &args.rest),
        Some("plugin") => plugins::run_cli(&args.data_dir, &args.rest),
        Some("help") => {
            println!("{USAGE}{DATA_DIR_NOTE}");
            Ok(())
        }
        Some(other) => bail!("unknown command `{other}`\n\n{USAGE}{DATA_DIR_NOTE}"),
    }
}

/// The browser opened `kanso://…` (Sign in with Apple on Windows and Linux)
/// and the system started this process with it: hand it to the Kanso that
/// is running, which accepts it only for a sign-in it is waiting on. The URL
/// carries a secret, so it is never printed.
fn forward_callback(data_dir: &std::path::Path, url: &str) {
    if let Err(error) = url_callback::forward(data_dir, url) {
        eprintln!("kanso: couldn't pass the sign-in back to Kanso: {error}");
    }
}

/// `kanso file.md` while another Kanso holds the data folder: hands it the
/// files and returns true. Anywhere else, or with nothing to hand over,
/// false and this process opens them itself. macOS opens documents through
/// the system, which sends them to the running app.
#[cfg(not(target_os = "macos"))]
fn forwarded_files(data_dir: &std::path::Path, files: &[PathBuf]) -> bool {
    if files.is_empty() {
        return false;
    }
    match url_callback::forward_files(data_dir, files) {
        Ok(()) => true,
        Err(url_callback::ForwardError::NotRunning) => false,
        Err(error) => {
            eprintln!("kanso: couldn't pass the files to Kanso: {error}");
            true
        }
    }
}

#[cfg(target_os = "macos")]
fn forwarded_files(_: &std::path::Path, _: &[PathBuf]) -> bool {
    false
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

fn run_app(
    data_dir: PathBuf,
    after_update: bool,
    safe_mode: bool,
    files: Vec<PathBuf>,
) -> Result<()> {
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
    // Files to open: from the command line now, from the system's open
    // events (Finder) and from a second `kanso file.md` later. They wait
    // here until the main window exists.
    let (opener, opened) = open_documents::channel();
    opener.paths(files);
    // Sign in with Apple returns through the browser and `kanso://` outside
    // macOS, and files come from a second `kanso`; the second one forwards
    // them here.
    #[cfg(not(target_os = "macos"))]
    {
        let opener = opener.clone();
        let sink: url_callback::FileSink = Arc::new(move |paths| opener.paths(paths));
        if let Err(err) = url_callback::listen(&data_dir, &url_callback::ROUTER, sink) {
            eprintln!("kanso: Sign in with Apple and opening files can't reach Kanso: {err}");
        }
    }

    // A daily snapshot of the database, off the main thread.
    let backup_store = store.clone();
    let backup_folder = data_dir.clone();
    std::thread::spawn(move || {
        if let Err(err) =
            kanso_engine::backup::run_daily(&backup_store, &backup_folder, kanso_core::now_ms())
        {
            eprintln!("kanso: backup failed: {err}");
        }
    });

    let app = application().with_assets(assets::KansoAssets);
    // The Dock's icon brings the main window back, even while notes are
    // open in windows of their own.
    app.on_open_urls({
        let opener = opener.clone();
        move |urls| opener.urls(&urls)
    });
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
        agent_permission::install(cx);
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
        // Plugins: none starts here. Safe mode is settled before any could.
        plugins::Plugins::init(
            store.clone(),
            &cx.global::<AppState>().data_dir.clone(),
            safe_mode,
            Arc::new(flavour::CURRENT.keychain()),
            cx,
        );
        reminders::init(store, open_main_window, cx);
        // Discovery, in the background: related notes and text in images.
        related::init(cx);
        voice::init(cx);
        ocr::init(cx);
        open_main_window(cx);
        note_window::restore(cx);
        open_documents::serve(opened, open_main_window, cx);
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
        // Any agent Kanso started for AI answers stops with it, and its
        // note access ends.
        ai_agent::stop_agents(cx);
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
