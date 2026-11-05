//! Kanso: a Markdown notes app with a local REST API.
//!
//! `kanso` opens the app. Other commands run without a window; see `--help`.

// On Windows the release build is a GUI program, so opening Kanso shows no
// console window; the commands borrow the console they were run from.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod account_cli;
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
mod agent_task_acp;
mod agent_task_panel;
#[cfg(test)]
mod agent_task_tests;
mod ai_agent;
mod ai_feature;
mod api_key_cli;
mod api_keys_model;

#[cfg(test)]
mod ask_agent_tests;
mod ask_panel;
#[cfg(test)]
mod ask_tests;
mod assets;
mod attachments;
mod board_view;
mod book_colors;
mod book_icons;
mod briefs;
mod calendar;
mod calendar_view;
mod canvas_state;
#[cfg(test)]
mod canvas_state_tests;
mod canvas_view;
mod capture;
mod capture_cli;
mod capture_request;
mod capture_voice;
#[cfg(test)]
mod capture_voice_tests;
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
mod control;
mod control_model;
mod device_approval;
mod dialogs;
mod dictation;
mod diff_view;
mod digest_view;
mod due;
mod editor;
mod editor_footer;
mod empty_state;
mod encrypt_progress;
mod export_html;
mod export_style;
mod feedback_app;
mod feedback_sheet;
#[cfg(test)]
mod feedback_sheet_tests;
mod flavour;
mod fonts;
mod fuzzy;
mod gallery;
mod graph;
mod graph_view;
mod grouping;
mod history;
mod history_diff;
mod home_data;
mod home_dialogs;
mod home_view;
mod hotkey;
mod icon_theme_file;
mod icon_theme_import;
mod icon_themes;
mod image_size;
mod ink_ocr;
mod inspector;
mod journal;
mod keys;
#[cfg(test)]
mod latency_bench;
mod launch;
mod layout;
mod library;
mod library_import;
mod library_key_model;
mod library_key_sheet;
#[cfg(test)]
mod library_key_tests;
mod library_open;
#[cfg(test)]
mod links_tests;
mod live_marks;
mod lock_flow;
mod locked;
mod markdown_edit;
mod markdown_source;
mod mcp_cli;
mod mcp_setup;
mod metrics;
mod metrics_hooks;
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
mod notice_views;
mod notices;
mod notify_support;
mod ocr;
mod open_documents;
mod origin;
mod page_chrome;
mod page_view;
mod pairing;
mod palette;
mod passkey_accounts;
mod passkey_ceremony;
mod paths;
mod pdf;
mod pen_input;
mod people_data;
mod people_view;
mod player;
mod player_model;
#[cfg(test)]
mod player_tests;
mod plugin_panel;
mod plugin_panel_model;
mod plugin_sheets;
mod plugin_sheets_model;
mod plugins;
mod plugins_model;
mod pouch_import;
mod presentation;
mod preview;
mod preview_canvas;
mod preview_chunks;
mod preview_rich;
mod preview_tasks;
#[allow(dead_code)]
mod proof_sheet;
mod provider_browser;
mod publish;
mod publish_site;
mod query_view_ui;
mod record_sheet;
mod recording;
mod recording_bar;
mod recording_model;
mod recording_summary;
#[cfg(test)]
mod recording_summary_tests;
#[cfg(test)]
mod recording_tests;
mod related;
mod reminders;
mod replace;
mod restore_sheet;
#[cfg(test)]
mod restore_waiting_tests;
mod resurface;
mod review_queue;
#[cfg(test)]
mod review_tests;
mod seal_migration;
mod secrets;
mod security_key_flow;
mod selection;
mod serve_cli;
mod services;
mod settings;
mod settings_ai;
mod settings_api_keys;
mod settings_briefs;
mod settings_capture;
mod settings_discovery;
mod settings_gallery;
mod settings_inbox;
mod settings_people;
mod settings_plugins;
mod settings_plugins_model;
mod settings_themes;
mod settings_voice;
#[cfg(test)]
mod settings_window_tests;
mod share_sheet;
mod shared_canvases;
mod shared_inbox;
#[cfg(test)]
mod shortcuts_parity_tests;
mod sidebar;
mod sketch_view;
mod smart_paste;
#[cfg(test)]
mod snapshots;
mod spellcheck;
// Tests check with a word list instead.
mod card_marks;
mod flashcards_model;
mod flashcards_view;
mod learn_model;
mod learn_view;
mod my_feedback;
mod settings_flashcards;
#[cfg(all(target_os = "macos", not(test)))]
mod spellcheck_macos;
mod split;
mod startup_error;
mod startup_error_model;
mod sticky;
mod sticky_cli;
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
mod trash_words;
mod triage_view;
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
mod voice_tidy;
mod wikilinks;
mod word_goal;
mod workspace;
mod zoom;

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context as _, Result, bail};
use gpui_kit::*;
use kanso_store::lock_data_dir;

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
                    window; syncs with your account when signed in
                    [--verbose]
  account           Sign this library in to a Kanso account from the
                    command line, see its status, list devices, sign
                    out; `kanso account help` lists how
  import <kind> [path]
                    Import a folder of Markdown or another app's export
                    [--dry-run]; Apple Notes finds its library itself.
                    `kanso import help` lists what each takes
  import pouchdb    Import the PouchDB notes library found on this computer
                    [--library-dir <dir>] [--credentials]
                    --credentials also takes its local API port and address
  capture [text]    Save a note to the Inbox, or [--book <name or id>];
                    with no text, from standard input. Works with
                    Kanso open or closed
  sticky [text]     Save a note to the Stickies notebook and show it as
                    a sticky note that stays above other windows;
                    [--book <name or id>] [--color <colour>]. Opens
                    Kanso if it's closed
  mcp               Serve Kanso's MCP tools over standard input and
                    output, for AI agents that start a program; needs
                    Kanso open with its local API on, or `kanso serve`
                    running
  api-key           Make, list and revoke API keys without the app, for
                    `kanso serve`; `kanso api-key help` lists how
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
        Some("serve") => serve_cli::run_cli(args.data_dir, &args.rest),
        // The window an encryption of the library shows; started by the app.
        Some("encrypt-progress") => encrypt_progress::run_child(),
        // The window for a library that won't open; started by the app.
        Some("startup-error") => startup_error::run_child(),
        Some("account") => account_cli::run_cli(&args.data_dir, &args.rest),
        Some("import") => library_import::run_cli(&args.data_dir, &args.rest),
        Some("theme") => theme_import::run_cli(&args.data_dir, &args.rest),
        Some("icon-theme") => icon_theme_import::run_cli(&args.data_dir, &args.rest),
        Some("capture") => capture_cli::run_cli(&args.data_dir, &args.rest),
        Some("sticky") => sticky_cli::run_cli(&args.data_dir, &args.rest),
        Some("mcp") => mcp_cli::run_cli(&args.data_dir, &args.rest),
        Some("api-key") => api_key_cli::run_cli(&args.data_dir, &args.rest),
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

/// X11 in place of the session's Wayland, when `KANSO_BACKEND=x11` or
/// Settings › General asks for it (`KANSO_BACKEND=wayland` overrides the
/// setting): under XWayland, GNOME lets sticky notes stay on top. `None`
/// keeps what the session names, as does a session with no X server.
#[cfg(target_os = "linux")]
fn forced_x11(config: &AppConfig) -> Option<WindowingRequest> {
    let asked = std::env::var("KANSO_BACKEND").ok();
    let x11 = match asked.as_deref().map(str::trim) {
        Some(backend) if backend.eq_ignore_ascii_case("x11") => true,
        Some(backend) if backend.eq_ignore_ascii_case("wayland") => false,
        _ => config.linux.backend == config::LinuxBackend::X11,
    };
    let session = GraphicalEnvironment::detect();
    if !x11 || session.wayland_display.is_none() || session.x11_display.is_none() {
        return None;
    }
    Some(WindowingRequest::Windowed(GraphicalEnvironment {
        wayland_display: None,
        ..session
    }))
}

/// Loads the settings, bringing an older file up to date.
fn load_config(data_dir: &std::path::Path) -> Result<AppConfig> {
    let path = paths::config_path(data_dir);
    AppConfig::load_migrated(&path).with_context(|| format!("reading {}", path.display()))
}

/// Opens the library. A plain one is encrypted first, behind a small window
/// that says how far it is; the library window opens when that is done.
fn open_library_showing_progress(
    data_dir: &std::path::Path,
) -> Result<(kanso_store::Store, Option<String>)> {
    let reporter = if library_open::will_encrypt(data_dir) {
        encrypt_progress::Reporter::window()
    } else {
        encrypt_progress::Reporter::stderr()
    };
    let (store, plain_because) =
        library_open::open_store_progress(data_dir, &|progress| reporter.report(progress))?;
    drop(reporter);
    Ok((store, plain_because))
}

/// How long a Kanso opened by an update waits for the one that installed
/// it to let go of the data folder.
const AFTER_UPDATE_WAIT: std::time::Duration = std::time::Duration::from_secs(20);

/// What starting Kanso holds until the main window opens.
struct Started {
    lock: kanso_store::DataDirLock,
    config: AppConfig,
    store: kanso_store::Store,
    plain_because: Option<String>,
}

/// Makes the data folder, takes its lock, reads the settings and opens the
/// library, or says which of them failed. Waits for the lock when an update
/// opened this Kanso.
fn start(
    data_dir: &std::path::Path,
    wait_for_update: bool,
) -> Result<Started, startup_error_model::Failure> {
    use startup_error_model::Failure;
    std::fs::create_dir_all(data_dir).map_err(|err| Failure::Folder(err.to_string()))?;
    let mut lock = lock_data_dir(data_dir).map_err(|err| Failure::LockFailed(err.to_string()))?;
    let started = std::time::Instant::now();
    while lock.is_none() && wait_for_update && started.elapsed() < AFTER_UPDATE_WAIT {
        std::thread::sleep(std::time::Duration::from_millis(200));
        lock = lock_data_dir(data_dir).map_err(|err| Failure::LockFailed(err.to_string()))?;
    }
    let Some(lock) = lock else {
        return Err(Failure::Locked);
    };
    let config = load_config(data_dir).map_err(|err| Failure::Config(format!("{err:#}")))?;
    let (store, plain_because) = open_library_showing_progress(data_dir)
        .map_err(|err| Failure::Library(startup_error_model::classify(&err)))?;
    Ok(Started {
        lock,
        config,
        store,
        plain_because,
    })
}

fn run_app(
    data_dir: PathBuf,
    after_update: bool,
    safe_mode: bool,
    files: Vec<PathBuf>,
) -> Result<()> {
    // What can stop Kanso before any window opens is shown in a window of
    // its own: launched from the Finder or the dock there is no terminal to
    // read standard error in. Try again goes round once more.
    let mut wait_for_update = after_update;
    let Started {
        lock,
        config,
        store,
        plain_because,
    } = loop {
        let failure = match start(&data_dir, wait_for_update) {
            Ok(started) => break started,
            Err(failure) => failure,
        };
        wait_for_update = false;
        let screen = startup_error_model::describe(&failure, &data_dir);
        eprintln!("kanso: {}. {}", screen.title, screen.message);
        for (ix, step) in screen.steps.iter().enumerate() {
            eprintln!("  {}. {step}", ix + 1);
        }
        match startup_error::show(&failure, &data_dir) {
            Some(startup_error::Choice::TryAgain) => {}
            Some(startup_error::Choice::Quit) | None => std::process::exit(1),
        }
    };
    let store = Arc::new(store);
    // Files to open: from the command line now, from the system's open
    // events (Finder) and from a second `kanso file.md` later. They wait
    // here until the main window exists.
    let (opener, opened) = open_documents::channel();
    opener.paths(files);
    // Stickies a second `kanso sticky` asks for: saved as they arrive,
    // shown once the windows are open.
    let (sticky_sender, stickies) = async_channel::unbounded();
    {
        let store = store.clone();
        url_callback::on_stickies(Arc::new(move |request| {
            let (id, _) = sticky_cli::save(&store, &request, kanso_store::Origin::Api)
                .map_err(|err| format!("{err:#}"))?;
            sticky_sender
                .try_send((id, request.color.unwrap_or_default()))
                .map_err(|_| "Kanso is quitting".to_owned())
        }));
    }
    // The local link, on every platform: Sign in with Apple returns through
    // the browser and `kanso://` outside macOS, and files, stickies and
    // notes to capture come from a second `kanso`; the second one forwards
    // them here. The link can only add notes, never read them.
    {
        let opener = opener.clone();
        let sink: url_callback::FileSink = Arc::new(move |paths| opener.paths(paths));
        let captures: url_callback::CaptureSink = {
            let store = store.clone();
            Arc::new(move |request| {
                capture_cli::add_note(&store, &request, kanso_store::Origin::Api).map(|(id, _)| id)
            })
        };
        if let Err(err) =
            url_callback::listen_with(&data_dir, &url_callback::ROUTER, sink, captures)
        {
            eprintln!(
                "kanso: `kanso capture` and `kanso sticky` can't reach this Kanso while it is open ({err}). Restart Kanso, or quit it to capture from the command line."
            );
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
    #[cfg(target_os = "linux")]
    let app = match forced_x11(&config) {
        Some(request) => app.with_windowing(request),
        None => app,
    };
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
        // A library that could not be encrypted says so in a notice.
        if let Some(why) = plain_because {
            cx.set_global(notices::PlainLibrary(why));
        }
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
        // Kanso Dev keeps its secrets under its own service; a library
        // signed in with `kanso account sign-in --secrets file` keeps them in
        // its data folder.
        let secrets = secrets::open_secrets_or_refuse(
            &cx.global::<AppState>().config,
            &cx.global::<AppState>().data_dir,
        );
        services.secrets = Some(secrets.clone());
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
        canvas_view::bind_keys(cx);
        pen_input::install();
        board_view::bind_keys(cx);
        workspace::set_menus(cx);
        cx.on_action(|_: &Quit, cx| quit(cx));
        cx.on_action(|_: &workspace::UndoAgentTask, cx| agent_task_panel::undo_latest(cx));
        watch_for_quit(cx);
        // A recording a crash left, or a transcription a quit cut off.
        recording::scan(cx);
        agent_permission::install(cx);
        settings::register_app_actions(cx, open_main_window);
        updates::register_app_actions(cx);
        services::apply_server_settings(cx);
        services::start_control(cx);
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
        // The anonymous daily count: only a release, once the notice was seen.
        metrics::init(cx);
        // Feedback reports that were waiting go once the session is back.
        feedback_app::start(cx);
        hotkey::init(cx);
        let store = cx.global::<AppState>().store.clone();
        // Plugins: none starts here. Safe mode is settled before any could.
        plugins::Plugins::init(
            store.clone(),
            &cx.global::<AppState>().data_dir.clone(),
            safe_mode,
            secrets,
            cx,
        );
        reminders::init(store.clone(), open_main_window, cx);
        briefs::init(store, open_main_window, cx);
        // Discovery, in the background: related notes, text in images and in PDFs.
        related::init(cx);
        voice::init(cx);
        ocr::init(cx);
        pdf::index::init(cx);
        ink_ocr::init(cx);
        open_main_window(cx);
        note_window::restore(cx);
        open_documents::serve(opened, open_main_window, cx);
        note_window::serve_stickies(stickies, cx);
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
        window_min_size: Some(size(
            px(layout::WINDOW_MIN_SIZE.0),
            px(layout::WINDOW_MIN_SIZE.1),
        )),
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
                    view.update(cx, |workspace, cx| workspace.launch_to_overview(window, cx));
                    window.on_window_should_close(cx, move |window, cx| {
                        // A sketch's ink from the last two seconds included.
                        editor.update(cx, |editor, cx| editor.flush_with_sketches(window, cx));
                        true
                    });
                    offer_first_import(window, cx);
                    updates::announce(window, cx);
                    metrics::show_notice(window, cx);
                    library_key_sheet::offer_at_launch(window, cx);
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
        // A quit that gave no chance to ask: the recording stops and its
        // folder stays for the next launch to recover.
        recording::stop_for_exit(cx);
        services::stop_server(cx);
        // Any agent Kanso started for AI answers stops with it, and its
        // note access ends.
        ai_agent::stop_agents(cx);
        // A staged update goes in now, and Kanso opens again if Restart
        // asked for it.
        updates::install_on_quit(cx);
        // The session's length and the last batch, before the config is saved.
        metrics::on_quit(cx);
        AppState::save_pending_config(cx);
        clipboard::clear_secret_on_quit();
        async {}
    })
    .detach();
}

/// Set once the person chose to quit with unreviewed agent changes.
struct ConfirmedQuit;

impl Global for ConfirmedQuit {}

/// Asks whether to quit with unreviewed agent changes; Quit anyway quits.
fn confirm_quit(prompt: String, cx: &mut App) {
    let Some((handle, _)) = workspace::main_workspace(cx) else {
        cx.set_global(ConfirmedQuit);
        quit(cx);
        return;
    };
    handle
        .update(cx, |_, window, cx| {
            use gpui_kit::component::WindowExt as _;
            window.open_alert_dialog(cx, move |alert, _, _| {
                alert
                    .title("Quit Kanso?")
                    .child(prompt.clone())
                    .show_cancel(true)
                    .ok_text("Quit Anyway")
                    .on_ok(|_, _, cx| {
                        cx.set_global(ConfirmedQuit);
                        quit(cx);
                        true
                    })
            });
        })
        .ok();
}

/// Saves open notes, then quits. The Quit action arrives while its window
/// is being updated, and a window can't be reached from inside its own
/// update, so the work waits until that update ends.
fn quit(cx: &mut App) {
    // "Stop and keep the recording?" Stopping finalises it first.
    if recording::confirm_quit(cx) {
        recording::ask_before_quit(quit, cx);
        return;
    }
    if let Some(prompt) = cx
        .try_global::<services::Services>()
        .and_then(|services| services.tasks.quit_prompt())
        && !cx.has_global::<ConfirmedQuit>()
    {
        // "1 agent task has changes you haven't reviewed. Quit anyway?"
        // The change sets live in memory only, so quitting loses them.
        confirm_quit(prompt, cx);
        return;
    }
    cx.defer(|cx| {
        workspace::for_each_editor(cx, |editor, cx| editor.flush(cx));
        services::stop_server(cx);
        cx.quit();
    });
}
