//! A real `kanso-plugin-host` run by the supervisor, answering through the
//! router: a sample command reads the open note, a runaway loop and a huge
//! allocation are stopped while the app carries on, a hang past the wall
//! deadline is killed, and refused calls add up to the plugin being turned
//! off.

use std::path::PathBuf;
use std::time::Duration;

use kanso_core::DocType;
use kanso_plugins::model::{
    EditorPermission, Grant, Installed, Manifest, NotesGrant, Permissions, Publisher, Source,
};
use kanso_plugins::protocol::{Budgets, PluginInfo, Theme, ToHost};
use kanso_plugins::router::{Editor, NoNetwork, Request, Router, Secrets, Selection};
use kanso_plugins::supervisor::{Health, Host, Log, RunError};
use kanso_store::{Origin, Put, Store};
use serde_json::{Value, json};

struct NoSecrets;
impl Secrets for NoSecrets {
    fn get(&self, _: &str, _: &str) -> Option<String> {
        None
    }
}

/// The host binary, built if this test runs on its own.
fn host_exe() -> PathBuf {
    if let Some(path) = kanso_plugins::supervisor::find_host() {
        return path;
    }
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let status = std::process::Command::new(cargo)
        .args(["build", "-p", "kanso-plugin-host"])
        .status()
        .expect("building the plugin host");
    assert!(status.success());
    kanso_plugins::supervisor::find_host().expect("the plugin host after building it")
}

fn info() -> PluginInfo {
    PluginInfo {
        id: "com.example.wordcount".into(),
        name: "Word count".into(),
        version: "1.0.0".into(),
    }
}

fn start(
    code: &str,
    budgets: Budgets,
) -> Result<(Host, kanso_plugins::protocol::Registered), RunError> {
    let cwd = tempfile::tempdir().unwrap();
    Host::start_within(
        &host_exe(),
        cwd.path(),
        info(),
        code.into(),
        budgets,
        Duration::from_secs(3),
    )
}

fn installed(grant: Grant) -> Installed {
    Installed {
        manifest: Manifest {
            schema: 1,
            api_version: 1,
            id: "com.example.wordcount".into(),
            name: "Word count".into(),
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
        grant,
        disabled: None,
        registered: Default::default(),
        installed_at: 0,
        last_error: None,
    }
}

fn editor_grant() -> Grant {
    Grant {
        commands: true,
        editor: [
            EditorPermission::NoteRead,
            EditorPermission::SelectionRead,
            EditorPermission::SelectionReplace,
        ]
        .into(),
        notes: NotesGrant::default(),
        fences: ["chart".to_owned()].into(),
        panel: false,
        secrets: Default::default(),
        network: Default::default(),
        ask_before_changing: true,
    }
}

const WORD_COUNT: &str = r#"
kanso.commands.register({
  id: "count-words",
  title: "Count words",
  when: "note-open",
  run() {
    const id = kanso.editor.noteId();
    const note = kanso.notes.get(id);
    const words = note.body.trim().split(/\s+/).filter(Boolean).length;
    return { text: `${words} words` };
  }
});
kanso.commands.register({
  id: "shout",
  title: "Shout the selection",
  when: "selection",
  run() {
    const s = kanso.editor.selection();
    kanso.editor.replaceSelection({ text: s.text.toUpperCase(), selection: { start: s.start, end: s.end } });
    return null;
  }
});
kanso.commands.register({
  id: "pry",
  title: "Try what isn't allowed",
  run() {
    for (const call of [() => kanso.fetch("https://example.com"), () => kanso.notes.create({ bookId: "book:x" }), () => kanso.secrets.get("x")]) {
      try { call(); } catch (e) { kanso.log("warn", e.code); }
    }
    return "done";
  }
});
kanso.commands.register({ id: "spin", title: "Spin", run() { while (true) {} } });
kanso.commands.register({ id: "hog", title: "Hog", run() { let s = "x"; while (true) { s = s + s; } } });
kanso.fences.register("chart", ({ source }) => `<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><text>${source.length}</text></svg>`);
"#;

#[test]
fn a_sample_plugin_runs_through_the_router() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    let book = store
        .put(
            DocType::Book,
            json!({ "name": "Work" }),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    let note = store
        .put(
            DocType::Note,
            json!({ "bookId": book, "title": "Draft", "body": "one two three four" }),
            Put::Upsert,
            Origin::Ui,
        )
        .unwrap()
        .id;
    let plugin = installed(editor_grant());
    let editor = Editor {
        note_id: Some(note.clone()),
        selection: Some(Selection {
            text: "two".into(),
            start: 4,
            end: 7,
        }),
    };
    let (mut host, registered) = start(WORD_COUNT, Budgets::DEFAULT).unwrap();
    assert_eq!(registered.commands.len(), 5);
    assert_eq!(registered.fences, vec!["chart"]);

    let plugin_dir = tempfile::tempdir().unwrap();
    let mut network = NoNetwork;
    let mut log = Log::default();
    let mut router = Router {
        store: &store,
        plugin: &plugin,
        plugin_dir: plugin_dir.path(),
        editor: &editor,
        request: Request::Command,
        secrets: &NoSecrets,
        network: &mut network,
        out: Default::default(),
    };
    let value = host
        .invoke("count-words", Value::Null, &mut router, &mut log)
        .unwrap();
    assert_eq!(value["text"], "4 words");

    host.invoke("shout", Value::Null, &mut router, &mut log)
        .unwrap();
    let edit = router.out.selection_edit.clone().unwrap();
    assert_eq!((edit.start, edit.end, edit.text.as_str()), (4, 7, "TWO"));
    assert_eq!(edit.note_id, note);

    // Three refused calls, logged by the plugin, counted by the router.
    assert_eq!(
        host.invoke("pry", Value::Null, &mut router, &mut log)
            .unwrap(),
        "done"
    );
    assert_eq!(router.out.refused, 3);
    assert!(log.lines().all(|(_, code)| code == "denied"));
    let mut health = Health::default();
    assert_eq!(
        health.refused(router.out.refused),
        Some(kanso_plugins::model::Disabled::Refused)
    );

    // A renderer draws, and still nothing it does reaches the store.
    let mut render_router = Router {
        request: Request::Render,
        out: Default::default(),
        ..router
    };
    let theme = Theme {
        dark: false,
        background: "#ffffff".into(),
        text: "#000000".into(),
        muted: "#666666".into(),
        accent: "#aa7700".into(),
        border: "#dddddd".into(),
        surface: "#eeeeee".into(),
    };
    let svg = host
        .render("chart", "abc", theme, &mut render_router, &mut log)
        .unwrap();
    assert!(svg.contains("<text>3</text>"));
    host.stop();
}

#[test]
fn runaway_code_is_stopped_and_the_host_carries_on() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    let plugin = installed(editor_grant());
    let editor = Editor::default();
    let budgets = Budgets {
        command_ms: 300,
        memory: 32 << 20,
        ..Budgets::DEFAULT
    };
    let (mut host, _) = start(WORD_COUNT, budgets).unwrap();
    let plugin_dir = tempfile::tempdir().unwrap();
    let mut network = NoNetwork;
    let mut log = Log::default();
    let mut router = Router {
        store: &store,
        plugin: &plugin,
        plugin_dir: plugin_dir.path(),
        editor: &editor,
        request: Request::Command,
        secrets: &NoSecrets,
        network: &mut network,
        out: Default::default(),
    };
    let began = std::time::Instant::now();
    let spun = host.invoke("spin", Value::Null, &mut router, &mut log);
    assert!(
        matches!(spun, Err(RunError::Failed(ref f)) if f.code == "timeout"),
        "{spun:?}"
    );
    assert!(began.elapsed() < Duration::from_secs(3));
    let hogged = host.invoke("hog", Value::Null, &mut router, &mut log);
    assert!(
        matches!(hogged, Err(RunError::Failed(ref f)) if f.code == "limit" || f.code == "thrown"),
        "{hogged:?}"
    );
    // Still answering.
    assert!(host.is_running());
    let mut editor_router = Router {
        out: Default::default(),
        ..router
    };
    let fine = host.invoke("pry", Value::Null, &mut editor_router, &mut log);
    assert_eq!(fine.unwrap(), "done");
}

#[test]
fn a_hang_past_the_wall_deadline_is_killed() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    let plugin = installed(editor_grant());
    let editor = Editor::default();
    // A CPU budget far beyond the wall deadline: only the kill can stop it.
    let budgets = Budgets {
        command_ms: 600_000,
        ..Budgets::DEFAULT
    };
    let (mut host, _) = start(WORD_COUNT, budgets).unwrap();
    let plugin_dir = tempfile::tempdir().unwrap();
    let mut network = NoNetwork;
    let mut log = Log::default();
    let mut router = Router {
        store: &store,
        plugin: &plugin,
        plugin_dir: plugin_dir.path(),
        editor: &editor,
        request: Request::Command,
        secrets: &NoSecrets,
        network: &mut network,
        out: Default::default(),
    };
    let began = std::time::Instant::now();
    let result = host.request(
        ToHost::Invoke {
            id: 99,
            command: "spin".into(),
            args: Value::Null,
        },
        99,
        Duration::from_millis(400),
        &mut router,
        &mut log,
    );
    assert_eq!(result, Err(RunError::Timeout));
    assert!(began.elapsed() < Duration::from_secs(2));
    assert!(!host.is_running());
    assert!(result.unwrap_err().lost_the_process());
}

#[test]
fn a_bad_start_never_hangs() {
    assert!(matches!(
        start("throw new Error('nope')", Budgets::DEFAULT),
        Err(RunError::FailedToStart(_))
    ));
    let slow = Budgets {
        startup_ms: 600_000,
        ..Budgets::DEFAULT
    };
    let cwd = tempfile::tempdir().unwrap();
    let began = std::time::Instant::now();
    let result = Host::start_within(
        &host_exe(),
        cwd.path(),
        info(),
        "while (true) {}".into(),
        slow,
        Duration::from_millis(400),
    );
    assert!(matches!(result, Err(RunError::FailedToStart(_))));
    assert!(began.elapsed() < Duration::from_secs(2));
    let mut health = Health::default();
    health.failed_to_start();
    health.failed_to_start();
    assert_eq!(
        health.failed_to_start(),
        Some(kanso_plugins::model::Disabled::FailedToStart)
    );
}

#[test]
fn the_host_gets_no_environment() {
    // SAFETY: tests in this file don't read the environment concurrently
    // in a way this affects; the variable only needs to exist for the
    // check below.
    unsafe { std::env::set_var("KANSO_SECRET_PROBE", "leak") };
    let code = r#"
kanso.commands.register({ id: "env", title: "Env", run() {
  return [typeof process, typeof std, typeof os, typeof require, typeof console, typeof setTimeout];
} });
"#;
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    let plugin = installed(editor_grant());
    let editor = Editor::default();
    let (mut host, _) = start(code, Budgets::DEFAULT).unwrap();
    let plugin_dir = tempfile::tempdir().unwrap();
    let mut network = NoNetwork;
    let mut log = Log::default();
    let mut router = Router {
        store: &store,
        plugin: &plugin,
        plugin_dir: plugin_dir.path(),
        editor: &editor,
        request: Request::Command,
        secrets: &NoSecrets,
        network: &mut network,
        out: Default::default(),
    };
    let value = host
        .invoke("env", Value::Null, &mut router, &mut log)
        .unwrap();
    assert_eq!(value, json!(vec!["undefined"; 6]));
}

/// The sample plugins (published in `iamngoni/kanso-plugins`), as `kanso plugin dev` opens them.
fn sample(name: &str) -> (kanso_plugins::registry_model::Bundle, String) {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    let bundle = kanso_plugins::bundle::open_dir(&dir).unwrap();
    kanso_plugins::manifest::validate(&bundle.manifest).unwrap();
    let entry = bundle.manifest.entry.clone();
    let code = bundle
        .files
        .iter()
        .find(|(path, _)| *path == entry)
        .map(|(_, bytes)| String::from_utf8(bytes.clone()).unwrap())
        .unwrap();
    (bundle, code)
}

#[test]
fn the_documented_samples_run() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(dir.path().join("kanso.db")).unwrap();
    let book = store
        .put(
            DocType::Book,
            json!({ "name": "Work" }),
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

    // Word count: a command, the selection, and a panel of what it may read.
    let (bundle, code) = sample("word-count");
    let (mut host, registered) = start(&code, Budgets::DEFAULT).unwrap();
    let ids: Vec<&str> = registered.commands.iter().map(|c| c.id.as_str()).collect();
    assert_eq!(ids, ["count-words", "append-count", "recent-notes"]);
    let mut plugin = installed(Grant {
        notes: NotesGrant {
            read_books: [book.clone()].into(),
            ..NotesGrant::default()
        },
        panel: true,
        fences: Default::default(),
        ..editor_grant()
    });
    plugin.manifest = bundle.manifest.clone();
    let editor = Editor {
        note_id: Some(note.clone()),
        selection: Some(Selection {
            text: "two".into(),
            start: 4,
            end: 7,
        }),
    };
    let plugin_dir = tempfile::tempdir().unwrap();
    let mut network = NoNetwork;
    let mut log = Log::default();
    let mut router = Router {
        store: &store,
        plugin: &plugin,
        plugin_dir: plugin_dir.path(),
        editor: &editor,
        request: Request::Command,
        secrets: &NoSecrets,
        network: &mut network,
        out: Default::default(),
    };
    let counted = host
        .invoke("count-words", Value::Null, &mut router, &mut log)
        .unwrap();
    assert_eq!(counted["text"], "3 words");
    host.invoke("append-count", Value::Null, &mut router, &mut log)
        .unwrap();
    assert_eq!(
        router.out.selection_edit.as_ref().unwrap().text,
        "two (1 word)"
    );
    host.invoke("recent-notes", Value::Null, &mut router, &mut log)
        .unwrap();
    let panel = serde_json::to_value(router.out.panel.as_ref().unwrap()).unwrap();
    assert!(panel.to_string().contains("Draft"), "{panel}");
    assert_eq!(
        router.out.refused,
        0,
        "{:?}",
        log.lines().collect::<Vec<_>>()
    );
    host.stop();

    // Bar chart: SVG from the block's text, in the theme's colors.
    let (_, code) = sample("bar-chart");
    let (mut host, registered) = start(&code, Budgets::DEFAULT).unwrap();
    assert_eq!(registered.fences, ["barchart"]);
    let theme = Theme {
        dark: false,
        background: "#ffffff".into(),
        text: "#111111".into(),
        muted: "#777777".into(),
        accent: "#3366ff".into(),
        border: "#dddddd".into(),
        surface: "#f4f4f4".into(),
    };
    let mut router = Router {
        request: Request::Render,
        out: Default::default(),
        ..router
    };
    let svg = host
        .render("barchart", "Mon: 3\nTue: 5", theme, &mut router, &mut log)
        .unwrap();
    assert!(svg.starts_with("<svg") && svg.contains("#3366ff") && svg.contains("Tue"));
}
