//! `kanso-release sign` and `verify` with `--nightly`: Kanso Nightly's feed
//! is held to the keys Kanso Nightly compiles in, not the release's, so a
//! feed signed with a key the app would refuse never gets published.

use std::path::Path;
use std::process::{Command, Output};

use kanso_update::{nightly_trusted_keys, trusted_keys};

const SECRET_VAR: &str = "KANSO_TEST_SIGNING_KEY";

fn release(args: &[&str], secret: Option<&str>) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_kanso-release"));
    cmd.args(args).env_remove(SECRET_VAR);
    if let Some(secret) = secret {
        cmd.env(SECRET_VAR, secret);
    }
    cmd.output().unwrap()
}

/// A fresh key: its secret and its public half.
fn keygen(dir: &Path) -> (String, String) {
    let file = dir.join("secret.key");
    let out = release(&["keygen", "--secret-out", file.to_str().unwrap()], None);
    assert!(out.status.success());
    let public = String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .last()
        .unwrap()
        .trim()
        .to_owned();
    (std::fs::read_to_string(file).unwrap(), public)
}

fn feed(dir: &Path) -> String {
    let path = dir.join("latest.json");
    std::fs::write(
        &path,
        r#"{"schema":1,"version":"0.1.0-nightly.202611230200","pub_date":"2026-11-23T02:00:00Z","release_url":"https://github.com/iamngoni/kanso/releases/tag/nightly","notes":"n","platforms":{}}"#,
    )
    .unwrap();
    path.to_str().unwrap().to_owned()
}

#[test]
fn the_nightly_list_and_the_release_list_are_different() {
    let nightly = nightly_trusted_keys();
    assert!(!nightly.is_empty());
    assert!(trusted_keys().iter().all(|key| !nightly.contains(key)));
}

#[test]
fn nightly_signing_refuses_a_key_the_nightly_app_does_not_trust() {
    let dir = tempfile::tempdir().unwrap();
    let (secret, _) = keygen(dir.path());
    let feed = feed(dir.path());
    let args = [
        "sign",
        "--feed",
        &feed,
        "--key-env",
        SECRET_VAR,
        "--nightly",
    ];
    let out = release(&args, Some(&secret));
    assert!(!out.status.success());
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("nightly-keys.txt"),
        "the refusal names the nightly key list"
    );
    assert!(!Path::new(&format!("{feed}.sig")).exists());
}

#[test]
fn nightly_verify_needs_a_nightly_trusted_signature_and_the_extra_key() {
    let dir = tempfile::tempdir().unwrap();
    let (secret, public) = keygen(dir.path());
    let feed = feed(dir.path());
    // Signed with --allow-untrusted, as the workflow used to.
    let signed = release(
        &[
            "sign",
            "--feed",
            &feed,
            "--key-env",
            SECRET_VAR,
            "--allow-untrusted",
        ],
        Some(&secret),
    );
    assert!(signed.status.success());
    // The key itself checks out, but it isn't one Kanso Nightly trusts.
    let by_key = release(&["verify", "--feed", &feed, "--key", &public], None);
    assert!(by_key.status.success());
    let nightly = release(
        &["verify", "--feed", &feed, "--key", &public, "--nightly"],
        None,
    );
    assert!(!nightly.status.success());
    let nightly_only = release(&["verify", "--feed", &feed, "--nightly"], None);
    assert!(!nightly_only.status.success());
}
