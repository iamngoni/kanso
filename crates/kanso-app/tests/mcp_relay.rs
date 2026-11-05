//! `kanso mcp` (this package's own binary) over stdio: a key is required.
//! Without one it answers `initialize` with error -32001 and exits 2; a key
//! Kanso doesn't know is answered with the fix-it message.

use std::io::Write as _;
use std::process::{Command, Stdio};
use std::sync::Arc;

use kanso_server::{KeyAccess, KeyStore, NewKey, ServerConfig};
use kanso_store::Store;
use serde_json::{Value, json};

const INITIALIZE: &str = r#"{"jsonrpc":"2.0","id":7,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"t","version":"1"}}}"#;

/// Runs `kanso mcp` with `env`, sends `lines`, and gives back its exit
/// status and what it wrote on standard output.
fn relay(env: &[(&str, String)], lines: &[&str]) -> (Option<i32>, Vec<Value>) {
    let data = tempfile::tempdir().unwrap();
    relay_in(data.path(), env, lines)
}

/// [`relay`] with data folder `data`.
fn relay_in(
    data: &std::path::Path,
    env: &[(&str, String)],
    lines: &[&str],
) -> (Option<i32>, Vec<Value>) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_kanso"));
    command
        .args(["mcp", "--data-dir"])
        .arg(data)
        .env_remove("KANSO_MCP_TOKEN")
        .env_remove("KANSO_MCP_URL")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    for (name, value) in env {
        command.env(name, value);
    }
    let mut child = command.spawn().unwrap();
    let mut stdin = child.stdin.take().unwrap();
    for line in lines {
        writeln!(stdin, "{line}").unwrap();
    }
    drop(stdin);
    let output = child.wait_with_output().unwrap();
    let answers = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    (output.status.code(), answers)
}

#[test]
fn a_relay_without_a_key_refuses_and_exits_2() {
    let (code, answers) = relay(&[], &[INITIALIZE]);
    assert_eq!(code, Some(2));
    assert_eq!(answers.len(), 1);
    assert_eq!(answers[0]["id"], 7);
    assert_eq!(answers[0]["error"]["code"], -32001);
    assert_eq!(
        answers[0]["error"]["message"],
        "Kanso needs a key for this agent. In Kanso, open Settings › Local API › API keys, choose New key…, and paste the setup it gives you into this agent's settings."
    );
    // An empty key is no key.
    let (code, answers) = relay(&[("KANSO_MCP_TOKEN", "  ".into())], &[INITIALIZE]);
    assert_eq!(code, Some(2));
    assert_eq!(answers[0]["error"]["code"], -32001);
}

#[test]
fn a_relay_passes_a_known_key_through_and_explains_an_unknown_one() {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    let keys = KeyStore::open(store.clone()).unwrap();
    let key = keys
        .mint(NewKey {
            name: "Relay test".into(),
            client: "other".into(),
            access: KeyAccess::Read,
            books: None,
        })
        .unwrap()
        .key;
    let server = kanso_server::spawn(
        store,
        ServerConfig {
            bind: "127.0.0.1:0".parse().unwrap(),
        },
        keys,
    );
    let url = format!("http://{}/mcp", server.local_addr().unwrap());

    let (code, answers) = relay(
        &[
            ("KANSO_MCP_TOKEN", key.clone()),
            ("KANSO_MCP_URL", url.clone()),
        ],
        &[INITIALIZE],
    );
    assert_eq!(code, Some(0));
    assert_eq!(answers[0]["id"], 7);
    assert_eq!(answers[0]["result"]["serverInfo"]["name"], "Kanso");

    let stranger = format!("kcl_{}", "B".repeat(43));
    let (code, answers) = relay(
        &[
            ("KANSO_MCP_TOKEN", stranger),
            ("KANSO_MCP_URL", url.clone()),
        ],
        &[INITIALIZE],
    );
    assert_eq!(code, Some(0));
    assert_eq!(answers[0]["error"]["code"], -32001);
    assert_eq!(
        answers[0]["error"]["message"],
        "Kanso doesn't recognise this key. It may have been revoked. Create a new one in Kanso › Settings › Local API › API keys."
    );

    // The key is never sent to an address off this computer, and the
    // agent is told how to fix the address.
    let (code, answers) = relay(
        &[
            ("KANSO_MCP_TOKEN", key),
            ("KANSO_MCP_URL", "http://example.com/mcp".into()),
        ],
        &[INITIALIZE],
    );
    assert_eq!(code, Some(2));
    assert_eq!(answers[0]["id"], 7);
    assert_eq!(answers[0]["error"]["code"], -32000);
    assert!(
        answers[0]["error"]["message"]
            .as_str()
            .unwrap()
            .contains("Unset it"),
        "{}",
        answers[0]
    );
    drop(json!(null));
    server.stop().unwrap();
}

/// The relay reads the address from `config.json` and never writes the
/// file: an old one, login and all, is left for Kanso's own migration.
#[test]
fn a_relay_never_rewrites_the_settings() {
    let data = tempfile::tempdir().unwrap();
    let path = data.path().join("config.json");
    let old = r#"{"version":0,"server":{"enabled":true,"port":1,"bind_address":"127.0.0.1","username":"kanso","password":"hunter2"}}"#;
    std::fs::write(&path, old).unwrap();
    let key = format!("kcl_{}", "A".repeat(43));
    let (code, answers) = relay_in(data.path(), &[("KANSO_MCP_TOKEN", key)], &[INITIALIZE]);
    assert_eq!(code, Some(0));
    // Nothing listens on port 1.
    assert_eq!(answers[0]["error"]["code"], -32000);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), old);
}
