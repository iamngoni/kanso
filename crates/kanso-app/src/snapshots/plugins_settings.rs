//! Snapshot states for Settings › Plugins: two installed plugins with one
//! open, safe mode, a directory that isn't set up, a loaded directory
//! with an update, and the question before removing a plugin.

use std::collections::BTreeSet;

use gpui_kit::{AnyWindowHandle, Entity, HeadlessAppContext};
use kanso_core::DocType;
use kanso_plugins::model::{
    Disabled, EditorPermission, Fence, Grant, Installed, Manifest, NotesGrant, NotesRequest,
    Permissions, Publisher, ReadRequest, Source,
};
use kanso_plugins::net::{Egress, EgressLog};
use kanso_plugins::registry_model::{Entry, Index, Offer};
use kanso_store::{Origin, Put, Store};

use crate::plugins::Plugins;
use crate::plugins_model::Directory;
use crate::settings::SettingsPage;
use crate::workspace::{AppState, Workspace};

/// The id of the plugin from the directory.
const LISTED: &str = "com.example.clipper";
/// The id of the plugin in developer mode.
const DEVELOPER: &str = "dev.local.wordcount";

/// The shots: name (`dark-…` or `light-…`), how to set the window up,
/// and the width of a dialog it opens, if any.
pub const SHOTS: &[(&str, super::Prepare, Option<f32>)] = &[
    ("dark-settings-plugins", installed_open, None),
    ("light-settings-plugins", installed_open, None),
    ("dark-settings-plugins-developer", developer_open, None),
    ("light-settings-plugins-developer", developer_open, None),
    ("dark-settings-plugins-safe-mode", safe_mode, None),
    ("light-settings-plugins-safe-mode", safe_mode, None),
    ("dark-settings-plugins-not-set-up", not_set_up, None),
    ("light-settings-plugins-not-set-up", not_set_up, None),
    (
        "dark-settings-plugins-remove",
        remove_question,
        Some(crate::settings_plugins::SHEET_WIDTH),
    ),
    (
        "light-settings-plugins-remove",
        remove_question,
        Some(crate::settings_plugins::SHEET_WIDTH),
    ),
];

fn set(items: &[&str]) -> BTreeSet<String> {
    items.iter().map(|s| (*s).to_owned()).collect()
}

fn publisher(name: &str) -> Publisher {
    Publisher {
        name: name.into(),
        key: format!("ed25519:{}", "ab".repeat(32)),
    }
}

/// A web clipper from the directory: reads two notebooks, makes notes,
/// draws a block, uses a token and one host.
fn listed(books: &[String]) -> Installed {
    let manifest = Manifest {
        schema: 1,
        api_version: 1,
        id: LISTED.into(),
        name: "Web clipper".into(),
        version: "1.2.0".into(),
        entry: "main.js".into(),
        publisher: publisher("Example Labs"),
        min_app_version: "0.8.0".into(),
        icon: None,
        description: Some("Saves a page you paste into a note in your Inbox.".into()),
        permissions: Permissions {
            commands: true,
            editor: [EditorPermission::NoteRead].into(),
            notes: NotesRequest {
                read: Some(ReadRequest::default()),
                write: vec!["Inbox".into()],
                create: true,
            },
            fences: vec![Fence {
                language: "clip".into(),
            }],
            secrets: set(&["API_TOKEN"]),
            network: set(&["api.example.com"]),
            ..Permissions::default()
        },
    };
    Installed {
        grant: Grant {
            commands: true,
            editor: [EditorPermission::NoteRead].into(),
            notes: NotesGrant {
                read_books: books.iter().cloned().collect(),
                write_books: books.iter().take(1).cloned().collect(),
                create: true,
                ..NotesGrant::default()
            },
            fences: set(&["clip"]),
            panel: false,
            secrets: set(&["API_TOKEN"]),
            network: set(&["api.example.com"]),
            ask_before_changing: true,
        },
        manifest,
        source: Source::Registry {
            sha256: "0".repeat(64),
        },
        pinned_key: format!("ed25519:{}", "ab".repeat(32)),
        disabled: None,
        registered: Default::default(),
        installed_at: super::DEMO_NOW_MS - super::DAY,
        last_error: None,
    }
}

/// A word counter in developer mode that crashed twice.
fn developer() -> Installed {
    let manifest = Manifest {
        schema: 1,
        api_version: 1,
        id: DEVELOPER.into(),
        name: "Word count".into(),
        version: "0.1.0".into(),
        entry: "main.js".into(),
        publisher: publisher("You"),
        min_app_version: "0.8.0".into(),
        icon: None,
        description: None,
        permissions: Permissions {
            commands: true,
            editor: [EditorPermission::SelectionRead].into(),
            ..Permissions::default()
        },
    };
    Installed {
        grant: Grant {
            commands: true,
            editor: [EditorPermission::SelectionRead].into(),
            notes: NotesGrant::default(),
            fences: BTreeSet::new(),
            panel: false,
            secrets: BTreeSet::new(),
            network: BTreeSet::new(),
            ask_before_changing: true,
        },
        manifest,
        source: Source::Developer {
            path: "/Users/kanso/plugins/wordcount".into(),
        },
        pinned_key: String::new(),
        disabled: Some(Disabled::Crashed),
        registered: Default::default(),
        installed_at: super::DEMO_NOW_MS - 3 * super::DAY,
        last_error: Some("TypeError: selection is not a function (main.js:14)".into()),
    }
}

/// Two notebooks, and the plugins set up over the store.
fn setup(cx: &mut HeadlessAppContext, store: &Store) {
    let put = |name: &str| {
        store
            .put(
                DocType::Book,
                serde_json::json!({ "name": name }),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id
    };
    let books = vec![put("Inbox"), put("Reading")];
    let plugins = vec![listed(&books), developer()];
    cx.update(|cx| {
        let state = cx.global::<AppState>();
        let (arc, dir) = (state.store.clone(), state.data_dir.clone());
        crate::plugins::init_for_test(arc, &dir, plugins, cx);
        let shared = cx.global::<Plugins>().shared.clone();
        shared
            .set_secret(LISTED, "API_TOKEN", "test-token")
            .unwrap();
        for (level, text) in [
            ("info", "started: 3 commands, 1 renderer"),
            ("warn", "notes.get: not_found"),
            (
                "error",
                "count-words: Cannot read properties of null (reading 'body')",
            ),
        ] {
            crate::plugins::push_log_for_test(DEVELOPER, level, text, cx);
        }
        // Three requests, the newest first when read back.
        let mut log = EgressLog::new();
        for (ago_min, path, status, sent, received) in [
            (42, "/v1/clips", 201, 2_300, 180),
            (9, "/v1/clips", 200, 0, 5_400),
            (2, "/v1/me", 200, 0, 612),
        ] {
            log.push(Egress {
                at_ms: super::DEMO_NOW_MS - ago_min * super::MINUTE,
                method: if sent > 0 { "POST" } else { "GET" }.into(),
                host: "api.example.com".into(),
                url_path: path.into(),
                status: Some(status),
                sent,
                received,
                refused: None,
            });
        }
        let dir = shared.plugin_dir(LISTED);
        std::fs::create_dir_all(&dir).unwrap();
        kanso_plugins::install::write_egress(&dir, &log).unwrap();
    });
}

/// A loaded directory with the clipper, a newer version of it, and one
/// plugin not installed.
fn loaded_directory() -> (Directory, Vec<Offer>) {
    let entry = |id: &str, name: &str, version: &str, description: &str| Entry {
        id: id.into(),
        name: name.into(),
        publisher: "Example Labs".into(),
        description: description.into(),
        version: version.into(),
        sha256: "1".repeat(64),
        size: 12_000,
        signer: format!("ed25519:{}", "ab".repeat(32)),
        author_signature: String::new(),
        min_app_version: "0.8.0".into(),
        permissions: Permissions::default(),
        published_at: "2026-05-01T00:00:00Z".into(),
        revoked: false,
    };
    let clipper = entry(
        LISTED,
        "Web clipper",
        "1.3.0",
        "Saves a page you paste into a note.",
    );
    let index = Index {
        schema: 1,
        serial: 4,
        plugins: vec![
            clipper.clone(),
            entry(
                "com.example.mermaid",
                "Diagrams",
                "2.0.1",
                "Draws flowcharts from text in your notes.",
            ),
        ],
        revoked: Vec::new(),
    };
    let offer = Offer {
        entry: clipper,
        added: vec!["connect to cdn.example.com".into()],
        removed: Vec::new(),
        key_changed: false,
    };
    (Directory::Loaded(index), vec![offer])
}

fn show(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    expand: Option<&str>,
) {
    super::show_settings(cx, window, workspace, SettingsPage::Plugins);
    cx.update(|cx| {
        workspace.update(cx, |workspace, cx| {
            workspace
                .settings
                .read(cx)
                .plugins
                .clone()
                .update(cx, |page, cx| {
                    if let Some(id) = expand {
                        page.snapshot_expand(id);
                    }
                    cx.notify();
                });
        });
    });
}

/// The clipper open, with the directory loaded and an update waiting.
fn installed_open(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    setup(cx, store);
    let (directory, offers) = loaded_directory();
    cx.update(|cx| {
        let plugins = cx.global_mut::<Plugins>();
        plugins.directory = directory;
        plugins.offers = offers;
    });
    show(cx, window, workspace, Some(LISTED));
}

/// The plugin in developer mode open: it crashed, and Try again is offered.
fn developer_open(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    setup(cx, store);
    cx.update(|cx| {
        cx.global_mut::<Plugins>()
            .dev_requests
            .push("/Users/kanso/plugins/outline".into());
    });
    show(cx, window, workspace, Some(DEVELOPER));
}

/// Safe mode: every plugin is off for the session.
fn safe_mode(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    setup(cx, store);
    cx.update(|cx| crate::plugins::set_safe_mode_for_test(true, cx));
    show(cx, window, workspace, None);
}

/// A build without a directory key: Browse says so.
fn not_set_up(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    setup(cx, store);
    show(cx, window, workspace, None);
}

/// Remove… clicked.
fn remove_question(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    setup(cx, store);
    show(cx, window, workspace, Some(LISTED));
    cx.update_window(window, |_, window, cx| {
        let page = workspace.read(cx).settings.read(cx).plugins.clone();
        crate::settings_plugins::open_remove_sheet(
            LISTED.into(),
            "Web clipper".into(),
            page,
            window,
            cx,
        );
    })
    .unwrap();
}
