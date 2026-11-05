use std::io::Read as _;
use std::os::unix::fs::PermissionsExt as _;
use std::os::unix::net::UnixStream;
use std::sync::Arc;

use kanso_server::{KeyAccess, KeyStore};
use kanso_store::{Store, lock_data_dir};
use proptest::prelude::*;
use proptest::test_runner::RngSeed;

use super::*;
use crate::control_model::Request;

fn running() -> (tempfile::TempDir, ControlServer, KeyStore) {
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    let keys = KeyStore::open(store.clone()).unwrap();
    let server = ControlServer::start(dir.path(), ControlTarget::new(store, keys.clone())).unwrap();
    (dir, server, keys)
}

fn create(dir: &std::path::Path, name: &str) -> String {
    let request = Request::Create {
        name: name.into(),
        access: "read".into(),
        scopes: vec![],
    };
    match send(dir, &request).unwrap() {
        Sent::Done(out, err) => {
            let key = out.trim_end().to_owned();
            assert!(!err.contains(&key));
            key
        }
        other => panic!("{other:?}"),
    }
}

fn revoke(dir: &std::path::Path, target: &str) -> Sent {
    send(
        dir,
        &Request::Revoke {
            target: target.into(),
        },
    )
    .unwrap()
}

// no-prop-test: files and modes, not logic.
#[test]
fn the_socket_is_private_and_goes_with_the_server() {
    let (dir, server, _) = running();
    let path = dir.path().join("control.sock");
    assert_eq!(
        std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    drop(server);
    assert!(!path.exists());
    // With nothing listening there is nobody to ask.
    assert!(matches!(
        send(dir.path(), &Request::List).unwrap(),
        Sent::NoSocket
    ));
}

// no-prop-test: shutdown must drain accepted commands before the owner releases its folder lock.
#[test]
fn dropping_the_control_server_waits_for_an_accepted_client() {
    let dir = tempfile::tempdir().unwrap();
    let lock = lock_data_dir(dir.path()).unwrap().unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    let keys = KeyStore::open(store.clone()).unwrap();
    let server = ControlServer::start(dir.path(), ControlTarget::new(store, keys)).unwrap();
    let mut client = UnixStream::connect(dir.path().join("control.sock")).unwrap();
    client.write_all(b"{").unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    while server.accepted_clients() == 0 && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(
        server.accepted_clients(),
        1,
        "the client must be accepted before shutdown"
    );

    let started = std::time::Instant::now();
    drop(server);
    assert!(started.elapsed() >= std::time::Duration::from_secs(4));
    drop(client);
    drop(lock);
    assert!(lock_data_dir(dir.path()).unwrap().is_some());
}

// no-prop-test: verifies every concurrent bind produces an owner-only socket.
#[test]
fn concurrent_server_starts_each_create_a_private_socket() {
    const COUNT: usize = 8;
    let dir = tempfile::tempdir().unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    let keys = KeyStore::open(store.clone()).unwrap();
    let target = ControlTarget::new(store, keys);
    let ready = Arc::new(std::sync::Barrier::new(COUNT));
    let finished = Arc::new(std::sync::Barrier::new(COUNT));
    let threads: Vec<_> = (0..COUNT)
        .map(|_| {
            let (dir, target) = (dir.path().to_owned(), target.clone());
            let (ready, finished) = (ready.clone(), finished.clone());
            std::thread::spawn(move || {
                ready.wait();
                let server = ControlServer::start(&dir, target);
                let mode = server.as_ref().ok().map(|_| {
                    std::fs::metadata(dir.join("control.sock"))
                        .unwrap()
                        .permissions()
                        .mode()
                        & 0o777
                });
                finished.wait();
                match (server, mode) {
                    (Ok(server), Some(mode)) => {
                        drop(server);
                        Some(mode)
                    }
                    (Err(_), None) => None,
                    _ => unreachable!("a socket start either has a server and mode or neither"),
                }
            })
        })
        .collect();
    let mut succeeded = 0;
    for thread in threads {
        if let Some(mode) = thread.join().unwrap() {
            succeeded += 1;
            assert_eq!(mode, 0o600);
        }
    }
    assert_eq!(succeeded, 1, "only one server may own the same socket");
    assert!(!dir.path().join("control.sock").exists());
}

// no-prop-test: a created token must not become visible through Debug.
#[test]
fn sent_command_debug_redacts_a_created_key() {
    let token = "kcl_0123456789012345678901234567890123456789012";
    let sent = Sent::Done(token.to_owned(), String::new());
    assert!(!format!("{sent:?}").contains(token));
}

// no-prop-test: path types and directory permissions are security boundaries.
#[test]
fn a_symlink_or_regular_file_at_the_socket_path_is_never_replaced() {
    use std::os::unix::fs::symlink;

    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("outside");
    std::fs::write(&target, "keep this file").unwrap();
    let path = dir.path().join("control.sock");
    symlink(&target, &path).unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    let keys = KeyStore::open(store.clone()).unwrap();
    assert!(ControlServer::start(dir.path(), ControlTarget::new(store, keys)).is_err());
    assert_eq!(std::fs::read_to_string(target).unwrap(), "keep this file");
    assert!(
        std::fs::symlink_metadata(&path)
            .unwrap()
            .file_type()
            .is_symlink()
    );

    std::fs::remove_file(&path).unwrap();
    std::fs::write(&path, "keep this entry").unwrap();
    let store = Arc::new(Store::open(dir.path().join("another.db")).unwrap());
    let keys = KeyStore::open(store.clone()).unwrap();
    assert!(ControlServer::start(dir.path(), ControlTarget::new(store, keys)).is_err());
    assert_eq!(std::fs::read_to_string(path).unwrap(), "keep this entry");
}

// no-prop-test: another uid must not be able to replace a pathname socket.
#[test]
fn a_group_or_world_writable_data_directory_cannot_host_the_control_socket() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o777)).unwrap();
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    let keys = KeyStore::open(store.clone()).unwrap();
    let result = ControlServer::start(dir.path(), ControlTarget::new(store, keys));
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    assert!(result.is_err());
    assert!(!dir.path().join("control.sock").exists());
}

// no-prop-test: a writable non-sticky ancestor can replace the data folder.
#[test]
fn a_writable_parent_directory_cannot_host_the_control_socket() {
    let root = tempfile::tempdir().unwrap();
    let parent = root.path().join("shared");
    std::fs::create_dir(&parent).unwrap();
    let data = parent.join("data");
    std::fs::create_dir(&data).unwrap();
    std::fs::set_permissions(&parent, std::fs::Permissions::from_mode(0o777)).unwrap();
    std::fs::set_permissions(&data, std::fs::Permissions::from_mode(0o700)).unwrap();
    let store = Arc::new(Store::open(data.join("kanso.db")).unwrap());
    let keys = KeyStore::open(store.clone()).unwrap();
    let result = ControlServer::start(&data, ControlTarget::new(store, keys));
    std::fs::set_permissions(&parent, std::fs::Permissions::from_mode(0o700)).unwrap();
    assert!(result.is_err());
    assert!(!data.join("control.sock").exists());
}

// no-prop-test: Drop removes only the socket inode this server created.
#[test]
fn dropping_a_server_does_not_remove_a_replacement_path() {
    let (dir, server, _) = running();
    let path = dir.path().join("control.sock");
    std::fs::remove_file(&path).unwrap();
    std::fs::write(&path, "replacement").unwrap();
    drop(server);
    assert_eq!(std::fs::read_to_string(path).unwrap(), "replacement");
}

// no-prop-test: a dead process's socket file, and nothing else.
#[test]
fn a_dead_servers_socket_file_is_replaced() {
    let dir = tempfile::tempdir().unwrap();
    let stale = dir.path().join("control.sock");
    drop(std::os::unix::net::UnixListener::bind(&stale).unwrap());
    assert!(stale.exists());
    assert!(matches!(
        send(dir.path(), &Request::List).unwrap(),
        Sent::NoSocket
    ));
    let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
    let keys = KeyStore::open(store.clone()).unwrap();
    let _server = ControlServer::start(dir.path(), ControlTarget::new(store, keys)).unwrap();
    assert!(matches!(
        send(dir.path(), &Request::List).unwrap(),
        Sent::Done(..)
    ));
}

// no-prop-test: path length must give the locked CLI a concrete recovery step.
#[test]
fn an_overlong_socket_path_tells_the_cli_to_use_a_shorter_data_dir() {
    let root = tempfile::tempdir().unwrap();
    let mut data = root.path().to_path_buf();
    for name in [
        "a".repeat(32),
        "b".repeat(32),
        "c".repeat(32),
        "d".repeat(32),
    ] {
        data.push(name);
    }
    std::fs::create_dir_all(&data).unwrap();
    let path = data.join("control.sock");
    assert!(path.as_os_str().len() > 100);

    let client_error = send(&data, &Request::List).expect_err("long paths cannot use a socket");
    assert!(client_error.to_string().contains("shorter --data-dir"));

    let server_error = match ControlServer::start(&data, ControlTarget::default()) {
        Ok(server) => {
            drop(server);
            panic!("long paths cannot host a socket");
        }
        Err(error) => error,
    };
    assert!(server_error.to_string().contains("shorter --data-dir"));
    assert!(!path.exists());
}

// no-prop-test: hostile input over a real socket.
#[test]
fn nonsense_is_refused_and_the_server_goes_on() {
    let (dir, _server, _) = running();
    let path = dir.path().join("control.sock");
    for line in [
        b"not json\n".to_vec(),
        b"{\"verb\":\"shell\",\"cmd\":\"x\"}\n".to_vec(),
        vec![b'a'; 200_000],
        b"{\"verb\":\"list\",\"extra\":1}\n".to_vec(),
    ] {
        let mut stream = UnixStream::connect(&path).unwrap();
        let _ = stream.write_all(&line);
        let _ = stream.write_all(b"\n");
        let mut reply = String::new();
        let _ = stream.read_to_string(&mut reply);
        assert!(!reply.contains("kcl_"), "{reply}");
    }
    assert!(matches!(
        send(dir.path(), &Request::List).unwrap(),
        Sent::Done(..)
    ));
}

// no-prop-test: one fixed end-to-end sequence.
#[test]
fn a_revoke_over_the_socket_is_final_in_the_servers_own_keys() {
    let (dir, _server, keys) = running();
    let key = create(dir.path(), "Script");
    let other = create(dir.path(), "Other");
    // The server's cache already knows both, with no reload.
    assert!(keys.verify(&key).is_some() && keys.verify(&other).is_some());
    let sent = revoke(dir.path(), "script");
    assert!(matches!(sent, Sent::Done(..)), "{sent:?}");
    assert!(keys.verify(&key).is_none());
    assert!(keys.verify(&other).is_some());
    // Revoking it again says what to do.
    match revoke(dir.path(), "script") {
        Sent::Failed(error) => assert!(error.contains("kanso api-key list"), "{error}"),
        other => panic!("{other:?}"),
    }
    let listed = match send(dir.path(), &Request::List).unwrap() {
        Sent::Done(out, _) => out,
        other => panic!("{other:?}"),
    };
    assert!(listed.contains("Other") && !listed.contains("Script"));
    assert!(!listed.contains(&key) && !listed.contains(&other));
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 24,
        rng_seed: RngSeed::Fixed(0x0063_6f6e_7472),
        ..ProptestConfig::default()
    })]

    /// Whatever keys are made and revoked over the socket, the server's own
    /// cache accepts exactly the ones not revoked, at once.
    #[test]
    fn prop_the_cache_follows_the_socket(
        names in prop::collection::btree_set("[a-z][a-z0-9]{0,6}", 1..5),
        revoked in prop::collection::vec(any::<bool>(), 5),
        access in prop::sample::select(KeyAccess::ALL.to_vec()),
    ) {
        let (dir, _server, keys) = running();
        let mut made = Vec::new();
        for name in &names {
            let request = Request::Create {
                name: name.clone(),
                access: access.as_str().into(),
                scopes: vec![],
            };
            let Sent::Done(out, _) = send(dir.path(), &request).unwrap() else {
                return Err(TestCaseError::fail("create failed"));
            };
            made.push((name.clone(), out.trim_end().to_owned()));
        }
        for (ix, (name, key)) in made.iter().enumerate() {
            if revoked[ix] {
                prop_assert!(matches!(revoke(dir.path(), name), Sent::Done(..)));
                prop_assert!(keys.verify(key).is_none());
            }
        }
        for (ix, (_, key)) in made.iter().enumerate() {
            prop_assert_eq!(keys.verify(key).is_some(), !revoked[ix]);
        }
    }
}
