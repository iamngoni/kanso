//! `kanso api-key create | list | revoke` against the real binary
//! (issue #243): a key made with no window works over REST and `/mcp` on
//! `kanso serve`, and is refused once it is revoked. The key is on standard
//! output alone, and the commands work while `kanso serve` runs: a key made
//! then works at once, and a key revoked then is refused at once while another
//! keeps working and the server never stops.

use std::io::{Read as _, Write as _};
use std::net::TcpStream;
use std::path::Path;
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

use kanso_store::lock_data_dir;
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

/// API keys are returned once to the caller; libraries, config and local
/// sidecars retain only the token hash.
fn assert_key_not_on_disk(data: &Path, key: &str) {
    let mut pending = vec![data.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let kind = entry.file_type().unwrap();
            if kind.is_dir() {
                pending.push(entry.path());
            } else if kind.is_file() {
                let bytes = std::fs::read(entry.path()).unwrap();
                assert!(
                    !bytes
                        .windows(key.len())
                        .any(|window| window == key.as_bytes()),
                    "the API key was written to {}",
                    entry.path().display()
                );
            }
        }
    }
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

/// Stops `kanso serve` the way a service manager does (SIGTERM), so it
/// cleans up after itself.
fn stop_gracefully(mut child: Child) {
    let asked = Command::new("kill")
        .args(["-TERM", &child.id().to_string()])
        .status()
        .unwrap();
    assert!(asked.success());
    assert!(child.wait().unwrap().success());
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
    assert_key_not_on_disk(data, &key);

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

    // `list`, `create` and `revoke` go on working while the server holds the
    // folder: they are done by the server itself, over a socket only this
    // user can open, and nothing stops.
    let other = make_key(data, "Other script");
    assert_key_not_on_disk(data, &other);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let mode = std::fs::metadata(data.join("control.sock"))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o077, 0, "the control socket is private: {mode:o}");
    }
    let live = make_key(data, "Made while serving");
    assert_eq!(rest_status(port, Some(&live)), 200);
    assert!(text(&kanso(data, &["list"]).stdout).contains("Made while serving"));

    let started = Instant::now();
    let revoked = kanso(data, &["revoke", "Backup script"]);
    assert!(revoked.status.success(), "{}", text(&revoked.stderr));
    assert_eq!(text(&revoked.stdout), "");
    // Refused the moment the command has returned, within a second of
    // asking, and over MCP as well.
    assert_eq!(rest_status(port, Some(&key)), 401);
    assert!(
        started.elapsed() < Duration::from_secs(1),
        "{:?}",
        started.elapsed()
    );
    let answer = mcp_answer(data, port, &key);
    assert_eq!(answer["error"]["code"], -32001);
    // The others keep working, and the server never restarted.
    assert_eq!(rest_status(port, Some(&other)), 200);
    assert_eq!(rest_status(port, Some(&live)), 200);
    assert_eq!(
        mcp_answer(data, port, &other)["result"]["serverInfo"]["name"],
        "Kanso"
    );
    assert!(server_alive(&server));

    // Revoking it again, or a stranger, says what to do; neither prints a key.
    let again = kanso(data, &["revoke", "Backup script"]);
    assert!(!again.status.success());
    assert!(text(&again.stderr).contains("kanso api-key list"));
    assert!(!text(&again.stderr).contains(&key));
    stop(server);

    // Gone from the list, and still refused after a restart.
    assert!(!text(&kanso(data, &["list"]).stdout).contains("Backup script"));
    let server = serve(data, port);
    assert_eq!(rest_status(port, Some(&key)), 401);
    assert_eq!(rest_status(port, Some(&other)), 200);
    // A server asked to stop takes its socket with it, and with nothing
    // running the commands work on the library themselves.
    stop_gracefully(server);
    assert!(!data.join("control.sock").exists());
    assert!(text(&kanso(data, &["list"]).stdout).contains("Other script"));
}

/// A key made with nothing running.
fn make_key(data: &Path, name: &str) -> String {
    let made = kanso(data, &["create", "--name", name]);
    assert!(made.status.success(), "{}", text(&made.stderr));
    let key = text(&made.stdout).trim_end().to_owned();
    assert!(!text(&made.stderr).contains(&key));
    assert_key_not_on_disk(data, &key);
    key
}

fn server_alive(child: &Child) -> bool {
    // `kill(pid, 0)` without a signal crate: the process is ours.
    Command::new("kill")
        .args(["-0", &child.id().to_string()])
        .status()
        .is_ok_and(|status| status.success())
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

#[test]
fn a_locked_folder_without_a_control_server_is_never_written_behind_its_owner() {
    let (dir, _) = data_folder();
    let data = dir.path();
    let lock = lock_data_dir(data).unwrap().unwrap();

    let made = kanso(data, &["create", "--name", "Must not exist"]);
    assert!(!made.status.success());
    assert!(text(&made.stderr).contains("a second process must not write"));
    assert_eq!(text(&made.stdout), "");

    drop(lock);
    let listed = kanso(data, &["list"]);
    assert!(listed.status.success(), "{}", text(&listed.stderr));
    assert_eq!(text(&listed.stdout), "");
    assert!(text(&listed.stderr).contains("No API keys"));
}
