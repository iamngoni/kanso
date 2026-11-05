//! `kanso serve` (this package's binary) leaves a staged restore alone: only
//! the app applies one (issue #243 step 11, PR #301). It says so, serves the
//! library as it is, and the restore still waits for the app.

use std::io::{BufRead as _, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use kanso_sync::{FileSecrets, SECRETS_FILE, SECRETS_KEY_FILE_ENV};

const WAITING: &str = "A restore is waiting; open Kanso to apply it.";

/// The library in `dir`, opened with the secrets file its config names.
fn open(dir: &Path) -> kanso_store::Store {
    let secrets = FileSecrets::open(dir.join(SECRETS_FILE), None).unwrap();
    kanso_sync::open_library(&dir.join("kanso.db"), &secrets)
        .unwrap()
        .store
}

/// A port nothing listens on right now.
fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

#[test]
fn serve_leaves_a_staged_restore_alone() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("config.json"),
        format!(
            r#"{{"server": {{"enabled": true, "port": {}, "bindAddress": "127.0.0.1"}},
                "sync": {{"secretStore": "file"}}}}"#,
            free_port()
        ),
    )
    .unwrap();
    let db = dir.path().join("kanso.db");
    {
        let store = open(dir.path());
        let backup = store
            .backup_now(&dir.path().join("backups"), 1_000)
            .unwrap();
        // Written after the backup: applying the restore would drop it.
        store.set_meta("later", "yes").unwrap();
        let key = store.library_key().unwrap().clone();
        drop(store);
        kanso_store::stage_restore(&db, &backup, Some(&key), &key).unwrap();
    }

    let mut child = Command::new(env!("CARGO_BIN_EXE_kanso"))
        .args(["serve", "--data-dir"])
        .arg(dir.path())
        .env_remove(SECRETS_KEY_FILE_ENV)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let stderr = child.stderr.take().unwrap();
    let (lines, read) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        for line in BufReader::new(stderr).lines().map_while(Result::ok) {
            if lines.send(line).is_err() {
                break;
            }
        }
    });
    let mut said = Vec::new();
    let started = Instant::now();
    while started.elapsed() < Duration::from_secs(60) {
        match read.recv_timeout(Duration::from_millis(200)) {
            Ok(line) => {
                let listening = line.starts_with("Kanso API on");
                said.push(line);
                if listening {
                    break;
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                if child.try_wait().unwrap().is_some() {
                    break;
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
    let _ = child.kill();
    let _ = child.wait();

    assert!(
        said.iter().any(|line| line.starts_with("Kanso API on")),
        "serve started: {said:?}"
    );
    assert!(said.iter().any(|line| line == WAITING), "{said:?}");
    assert!(kanso_store::restore_staged(&db), "the restore still waits");
    assert_eq!(
        open(dir.path()).meta("later").unwrap().as_deref(),
        Some("yes"),
        "serve served the library as it was"
    );
}
