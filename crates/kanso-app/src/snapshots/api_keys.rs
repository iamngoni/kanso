//! Snapshot states for Settings › Local API › API keys (issue #243, step
//! 7): the list empty and with keys, a refused request, the New key sheet,
//! the key shown once, and the question before a revoke.

use std::sync::Arc;

use gpui_kit::{AnyWindowHandle, Entity, HeadlessAppContext};
use kanso_server::{KeyAccess, KeyStore};
use kanso_store::{NewApiKey, Store};

use crate::api_keys_model::Client;
use crate::mcp_setup::{Agent, Setup};
use crate::settings::SettingsPage;
use crate::workspace::{AppState, Workspace};

/// The shots: name (`dark-…` or `light-…`), how to set the window up, and
/// the width of the dialog it opens, if any.
pub const SHOTS: &[(&str, super::Prepare, Option<f32>)] = &[
    ("dark-settings-api-keys-empty", empty, None),
    ("light-settings-api-keys-empty", empty, None),
    ("dark-settings-api-keys", with_keys, None),
    ("light-settings-api-keys", with_keys, None),
    ("dark-settings-api-keys-refused", refused, None),
    ("light-settings-api-keys-refused", refused, None),
    ("dark-settings-api-lan-warning", lan_warning, None),
    ("light-settings-api-lan-warning", lan_warning, None),
    ("dark-api-key-new", new_key, Some(WIDTH)),
    ("light-api-key-new", new_key, Some(WIDTH)),
    ("dark-api-key-shown", shown_once, Some(WIDTH)),
    ("light-api-key-shown", shown_once, Some(WIDTH)),
    ("dark-api-key-revoke", revoke_question, Some(448.)),
    ("light-api-key-revoke", revoke_question, Some(448.)),
];

const WIDTH: f32 = crate::settings_api_keys::SHEET_WIDTH;

/// A key as the demo shows it, 47 characters like a real one.
const DEMO_KEY: &str = "kcl_9fQ2mT7xLw0aZ4dVbN8cYhJr3sUe5oKpGiX1tA6yE2x";

/// A key to save: name, client, access, notebooks, minutes since it was
/// made, and minutes since it was last used.
struct Demo {
    name: &'static str,
    client: &'static str,
    access: KeyAccess,
    books: Option<Vec<&'static str>>,
    made: i64,
    used: Option<i64>,
}

fn book(store: &Store, name: &str) -> String {
    store
        .books()
        .unwrap()
        .into_iter()
        .find(|b| b.name == name)
        .map(|b| b.id)
        .unwrap_or_else(|| panic!("the demo library has a notebook {name}"))
}

/// Opens Settings › Local API with `keys` as the library's keys. `refused`
/// is when a request was refused, in minutes ago.
fn open(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
    keys: &[Demo],
    refused: Option<i64>,
) {
    let now = super::DEMO_NOW_MS;
    for (ix, demo) in keys.iter().enumerate() {
        let books = demo
            .books
            .as_ref()
            .map(|names| names.iter().map(|n| book(store, n)).collect());
        let id = format!("01900000-0000-7000-8000-00000000000{ix}");
        store
            .api_key_insert(&NewApiKey {
                id: id.clone(),
                name: demo.name.into(),
                client: demo.client.into(),
                token_hash: [ix as u8 + 1; 32],
                access: demo.access,
                books,
                created_at: now - demo.made * super::MINUTE,
            })
            .unwrap();
        if let Some(used) = demo.used {
            store
                .api_key_touch(&id, now - used * super::MINUTE)
                .unwrap();
        }
    }
    cx.update(|cx| {
        let store: Arc<Store> = cx.global::<AppState>().store.clone();
        let at = now - refused.unwrap_or(0) * super::MINUTE;
        let keys = KeyStore::with_clock(store, Arc::new(move || at)).unwrap();
        if refused.is_some() {
            keys.note_refused();
        }
        crate::services::set_keys_for_test(keys, cx);
    });
    super::show_settings(cx, window, workspace, SettingsPage::LocalApi);
}

/// The three keys of the design's list. They are saved oldest first and
/// listed newest first.
fn demo_keys() -> Vec<Demo> {
    vec![
        Demo {
            name: "Backup script",
            client: "other",
            access: KeyAccess::Read,
            books: None,
            made: 9 * 24 * 60,
            used: Some(26 * 60),
        },
        Demo {
            name: "Claude Desktop",
            client: "claude-desktop",
            access: KeyAccess::Suggest,
            books: Some(vec!["Engineering", "Product"]),
            made: 3 * 24 * 60,
            used: None,
        },
        Demo {
            name: "Codex on this Mac",
            client: "codex",
            access: KeyAccess::Read,
            books: None,
            made: 50,
            used: Some(2),
        },
    ]
}

fn empty(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    open(cx, window, workspace, store, &[], None);
}

fn with_keys(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    open(cx, window, workspace, store, &demo_keys(), None);
}

/// The API listens on a LAN address, so Settings warns that keys travel
/// unencrypted.
fn lan_warning(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    cx.update(|cx| {
        AppState::update_config(cx, |config| config.server.bind_address = "0.0.0.0".into());
    });
    open(cx, window, workspace, store, &demo_keys(), None);
}

/// A request was refused 40 minutes ago.
fn refused(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    open(cx, window, workspace, store, &demo_keys(), Some(40));
}

/// New key… clicked.
fn new_key(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    open(cx, window, workspace, store, &demo_keys(), None);
    cx.update_window(window, |_, window, cx| {
        let page = workspace.read(cx).settings.read(cx).api_keys.clone();
        let keys = crate::services::key_store(cx).unwrap();
        let _ = crate::settings_api_keys::open_new_key_sheet(page, keys, window, cx);
    })
    .unwrap();
}

/// Create key clicked for Codex: the setup, with the key in it.
fn shown_once(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    open(cx, window, workspace, store, &demo_keys(), None);
    cx.update_window(window, |_, window, cx| {
        let page = workspace.read(cx).settings.read(cx).api_keys.clone();
        let keys = crate::services::key_store(cx).unwrap();
        let setup = Setup {
            endpoint: "http://127.0.0.1:19840/mcp".into(),
            key: zeroize::Zeroizing::new(DEMO_KEY.into()),
            program: "/Applications/Kanso.app/Contents/MacOS/kanso".into(),
        };
        let client = Client::Agent(Agent::Codex);
        let text = Agent::Codex.snippet(&setup);
        crate::settings_api_keys::open_made_sheet_for_test(page, keys, client, text, window, cx);
    })
    .unwrap();
}

/// Revoke clicked on the newest key.
fn revoke_question(
    cx: &mut HeadlessAppContext,
    window: AnyWindowHandle,
    workspace: &Entity<Workspace>,
    store: &Store,
) {
    open(cx, window, workspace, store, &demo_keys(), None);
    cx.update_window(window, |_, window, cx| {
        let page = workspace.read(cx).settings.read(cx).api_keys.clone();
        page.update(cx, |page, cx| page.snapshot_ask_revoke(0, window, cx));
    })
    .unwrap();
}
