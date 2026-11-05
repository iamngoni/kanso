//! Note text stays in the library (#243 step 12). Once `kanso.db` is
//! encrypted, nothing else in the data folder may hold a note's words. A
//! canary note is written, a plugin saves it into its settings, a request
//! for it is logged, and the whole data folder (the database files aside)
//! is scanned for it. A positive control proves the scan can find it.

use std::path::Path;

use kanso_core::DocType;
use kanso_plugins::install::{self, plugins_dir};
use kanso_plugins::model::{Grant, Installed, Manifest, Permissions, Publisher, Source};
use kanso_plugins::net::{Egress, EgressLog};
use kanso_plugins::router::{Editor, NoNetwork, Request, Router, Secrets};
use kanso_store::{Origin, Put, Store};
use serde_json::json;

const CANARY: &str = "canary-7f3a9c1e-zebra-quartz";
const ID: &str = "com.example.canary";

struct NoSecrets;
impl Secrets for NoSecrets {
    fn get(&self, _: &str, _: &str) -> Option<String> {
        None
    }
}

fn plugin() -> Installed {
    Installed {
        manifest: Manifest {
            schema: 1,
            api_version: 1,
            id: ID.into(),
            name: "Canary".into(),
            version: "1.0.0".into(),
            entry: "main.js".into(),
            publisher: Publisher {
                name: "Sam".into(),
                key: format!("ed25519:{}", "ab".repeat(32)),
            },
            min_app_version: "0.1.0".into(),
            icon: None,
            description: None,
            permissions: Permissions::default(),
        },
        source: Source::Developer { path: "/x".into() },
        pinned_key: String::new(),
        grant: Grant {
            commands: true,
            editor: Default::default(),
            notes: Default::default(),
            fences: Default::default(),
            panel: false,
            secrets: Default::default(),
            network: Default::default(),
            ask_before_changing: true,
        },
        disabled: None,
        registered: Default::default(),
        installed_at: 0,
        last_error: None,
    }
}

/// Every file under `dir` other than the library's own that holds `needle`.
fn stray_files(dir: &Path, needle: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut pending = vec![dir.to_path_buf()];
    while let Some(folder) = pending.pop() {
        for entry in std::fs::read_dir(&folder).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
                continue;
            }
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            if name.starts_with("kanso.db") {
                continue;
            }
            let bytes = std::fs::read(&path).unwrap();
            if bytes
                .windows(needle.len())
                .any(|window| window == needle.as_bytes())
            {
                found.push(path.display().to_string());
            }
        }
    }
    found
}

fn entry(path: &str) -> Egress {
    Egress {
        at_ms: 1,
        method: "GET".into(),
        host: "api.example.com".into(),
        url_path: path.into(),
        status: Some(200),
        sent: 0,
        received: 0,
        refused: None,
    }
}

#[test]
fn no_note_text_is_left_in_the_data_folder_outside_the_library() {
    let data = tempfile::tempdir().unwrap();
    let store = Store::open(data.path().join("kanso.db")).unwrap();
    let book = store
        .put(
            DocType::Book,
            json!({ "name": "Canary book" }),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    store
        .put(
            DocType::Note,
            json!({ "title": CANARY, "body": format!("{CANARY} body"), "bookId": book }),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap();

    // The folders the app makes beside the library.
    for folder in ["logs", "gallery/themes", "agents/workspace"] {
        std::fs::create_dir_all(data.path().join(folder)).unwrap();
    }
    let plugins = plugins_dir(data.path());
    let dir = install::plugin_dir(&plugins, ID);
    std::fs::create_dir_all(&dir).unwrap();

    // A plugin saves what it likes into its settings, and the proxy logs a
    // request whose path carries the same words.
    let plugin = plugin();
    let editor = Editor {
        note_id: None,
        selection: None,
    };
    let mut network = NoNetwork;
    let mut router = Router {
        store: &store,
        plugin: &plugin,
        plugin_dir: &dir,
        editor: &editor,
        request: Request::Command,
        secrets: &NoSecrets,
        network: &mut network,
        out: Default::default(),
    };
    router
        .call("settings.set", json!({ "value": { "recent": CANARY } }))
        .unwrap();
    assert_eq!(
        router.call("settings.get", json!({})).unwrap()["recent"],
        CANARY
    );
    let mut log = EgressLog::new();
    log.push(entry(&format!("/translate/{CANARY}")));
    install::write_egress(&store, ID, &log).unwrap();
    assert_eq!(install::read_egress(&store, &dir, ID).len(), 1);

    assert_eq!(stray_files(data.path(), CANARY), Vec::<String>::new());

    // Positive control: the same words in a plain file are found.
    std::fs::write(
        dir.join("settings.json"),
        format!("{{\"recent\":\"{CANARY}\"}}"),
    )
    .unwrap();
    let found = stray_files(data.path(), CANARY);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].ends_with("settings.json"));

    // An older Kanso's file is taken into the library, and the file goes.
    store.remove_meta(&format!("plugin.settings.{ID}")).unwrap();
    assert_eq!(install::read_settings(&store, &dir, ID)["recent"], CANARY);
    assert_eq!(stray_files(data.path(), CANARY), Vec::<String>::new());
}
