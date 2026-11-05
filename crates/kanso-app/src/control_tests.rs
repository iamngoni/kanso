use std::io::{Read as _, Write as _};
use std::os::unix::fs::PermissionsExt as _;
use std::os::unix::net::UnixStream;
use std::sync::Arc;

use kanso_server::{KeyAccess, KeyStore};
use kanso_store::Store;
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
        std::fs::metadata(&path).unwrap().permissions().mode() & 0o077,
        0
    );
    drop(server);
    assert!(!path.exists());
    // With nothing listening there is nobody to ask.
    assert!(matches!(
        send(dir.path(), &Request::List).unwrap(),
        Sent::NoSocket
    ));
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
