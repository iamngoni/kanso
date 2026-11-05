//! `kanso api-key create | list | revoke` against the real binary
//! (issue #243): a key made with no window works over REST and `/mcp` on
//! `kanso serve`, and is refused once it is revoked. The key is on standard
//! output alone, and the commands refuse while the data folder is in use.

use std::io::{Read as _, Write as _};
use std::net::TcpStream;
use std::path::Path;
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

use serde_json::Value;

const INITIALIZE: &str = r#"{"jsonrpc":"2.0","id":7,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"t","version":"1"}}}"#;

/// A data folder whose library key lives in a file, never in the system's
/// Keychain, and whose API listens on a free port.
fn data_folder() -> (tempfile::TempDir, u16) {
    let dir = tempfile::tempdir().unwrap();
    let port = {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.local_addr().unwrap().port()
    };
    let config = format!(
        r#"{{"sync":{{"secretStore":"file"}},"server":{{"enabled":true,"port":{port},"bind_address":"127.0.0.1"}}}}"#
    );
    std::fs::write(dir.path().join("config.json"), config).unwrap();
    (dir, port)
}

fn kanso(data: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_kanso"))
        .arg("api-key")
        .args(args)
        .arg("--data-dir")
        .arg(data)
        .env_remove("KANSO_MCP_TOKEN")
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn serve(data: &Path, port: u16) -> Child {
    let child = Command::new(env!("CARGO_BIN_EXE_kanso"))
        .args(["serve", "--data-dir"])
        .arg(data)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let until = Instant::now() + Duration::from_secs(60);
    while TcpStream::connect(("127.0.0.1", port)).is_err() {
        assert!(Instant::now() < until, "kanso serve never listened");
        std::thread::sleep(Duration::from_millis(100));
    }
    child
}

fn stop(mut child: Child) {
    child.kill().unwrap();
    child.wait().unwrap();
}

/// `GET /notes` with `key` (if any): the status code.
fn rest_status(port: u16, key: Option<&str>) -> u16 {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
    let auth = key.map_or(String::new(), |k| format!("Authorization: Bearer {k}\r\n"));
    write!(
        stream,
        "GET /notes HTTP/1.1\r\nHost: 127.0.0.1\r\n{auth}Connection: close\r\n\r\n"
    )
    .unwrap();
    let mut reply = String::new();
    stream.read_to_string(&mut reply).unwrap();
    reply.split(' ').nth(1).unwrap().parse().unwrap()
}

/// `kanso mcp` with `key`: the answer to `initialize`.
fn mcp_answer(data: &Path, port: u16, key: &str) -> Value {
    let mut child = Command::new(env!("CARGO_BIN_EXE_kanso"))
        .args(["mcp", "--data-dir"])
        .arg(data)
        .env("KANSO_MCP_TOKEN", key)
        .env("KANSO_MCP_URL", format!("http://127.0.0.1:{port}/mcp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    writeln!(stdin, "{INITIALIZE}").unwrap();
    drop(stdin);
    let output = child.wait_with_output().unwrap();
    serde_json::from_str(text(&output.stdout).lines().next().unwrap()).unwrap()
}

#[test]
fn a_key_made_on_the_command_line_works_until_it_is_revoked() {
    let (dir, port) = data_folder();
    let data = dir.path();

    // Nothing to list yet, and no key is printed.
    let listed = kanso(data, &["list"]);
    assert!(listed.status.success(), "{}", text(&listed.stderr));
    assert_eq!(text(&listed.stdout), "");
    assert!(text(&listed.stderr).contains("kanso api-key create"));

    // The key is on standard output alone.
    let made = kanso(
        data,
        &["create", "--name", "Backup script", "--access", "read"],
    );
    assert!(made.status.success(), "{}", text(&made.stderr));
    let stdout = text(&made.stdout);
    let key = stdout.strip_suffix('\n').unwrap().to_owned();
    assert_eq!(key.len(), 47, "{stdout:?}");
    assert!(key.starts_with("kcl_"));
    assert!(!text(&made.stderr).contains(&key));

    let listed = kanso(data, &["list"]);
    let line = text(&listed.stdout);
    assert!(
        line.contains("Backup script") && line.contains("read"),
        "{line}"
    );
    assert!(!line.contains(&key));

    // It works over REST and MCP.
    let server = serve(data, port);
    assert_eq!(rest_status(port, Some(&key)), 200);
    assert_eq!(rest_status(port, None), 401);
    let answer = mcp_answer(data, port, &key);
    assert_eq!(answer["result"]["serverInfo"]["name"], "Kanso");

    // While the server holds the folder, a key can't be changed behind its
    // back; the refusal says what to do.
    let refused = kanso(data, &["revoke", "Backup script"]);
    assert!(!refused.status.success());
    let words = text(&refused.stderr);
    assert!(
        words.contains("Quit it") && words.contains("Settings"),
        "{words}"
    );
    assert_eq!(rest_status(port, Some(&key)), 200);
    stop(server);

    // Revoked, it is refused over both.
    let revoked = kanso(data, &["revoke", "Backup script"]);
    assert!(revoked.status.success(), "{}", text(&revoked.stderr));
    assert_eq!(text(&revoked.stdout), "");
    let server = serve(data, port);
    assert_eq!(rest_status(port, Some(&key)), 401);
    let answer = mcp_answer(data, port, &key);
    assert_eq!(answer["error"]["code"], -32001);
    stop(server);

    // And it is gone from the list.
    assert_eq!(text(&kanso(data, &["list"]).stdout), "");
}

#[test]
fn a_bad_command_says_how_to_fix_it() {
    let (dir, _) = data_folder();
    let missing = kanso(dir.path(), &["create"]);
    assert!(!missing.status.success());
    assert!(text(&missing.stderr).contains("--name"));
    assert_eq!(text(&missing.stdout), "");
    let scope = kanso(dir.path(), &["create", "--name", "x", "--scope", "Nowhere"]);
    assert!(!scope.status.success());
    assert!(text(&scope.stderr).contains("Nowhere"));
    assert_eq!(text(&scope.stdout), "");
}
