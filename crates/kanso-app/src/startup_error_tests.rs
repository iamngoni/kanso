use std::path::{Path, PathBuf};

use kanso_store::StoreError;
use kanso_sync::{OpenLibraryError, SecretError};
use proptest::prelude::*;

use super::*;

fn dir() -> &'static Path {
    Path::new("/Users/me/Library/Application Support/Kanso")
}

fn opening(error: impl Into<OpenLibraryError>) -> anyhow::Error {
    anyhow::Error::new(error.into()).context("opening /Users/me/kanso.db")
}

/// Each error `open_library` gives maps to its message, and to the buttons
/// that make sense: Try again only where trying again can work.
#[test]
fn each_open_library_error_maps_to_its_message_and_buttons() {
    let cases: Vec<(anyhow::Error, Cause, &str, bool)> = vec![
        (
            opening(StoreError::WrongKey),
            Cause::WrongKey,
            "this computer's key doesn't open it",
            false,
        ),
        (
            opening(StoreError::Converting),
            Cause::Converting,
            "kanso.db.encrypting",
            true,
        ),
        (
            opening(StoreError::DiskFull { needed_mb: 412 }),
            Cause::DiskFull { needed_mb: 412 },
            "about 412 MB free",
            true,
        ),
        (
            opening(StoreError::EncryptFailed("copy failed".into())),
            Cause::EncryptFailed("copy failed".into()),
            "copy failed",
            true,
        ),
        (
            opening(OpenLibraryError::Keychain(SecretError("locked".into()))),
            Cause::Keychain("locked".into()),
            "system keychain: locked",
            true,
        ),
        (
            opening(StoreError::NoEncryption),
            Cause::NoEncryption,
            "has no encryption",
            false,
        ),
        (
            opening(StoreError::Io(std::io::Error::other("disk gone"))),
            Cause::Io("disk gone".into()),
            "disk gone",
            true,
        ),
        (
            opening(StoreError::Invalid("odd".into())),
            Cause::Other("opening /Users/me/kanso.db: odd".into()),
            "odd",
            false,
        ),
    ];
    for (error, cause, says, retry) in cases {
        assert_eq!(classify(&error), cause);
        let screen = describe(&Failure::Library(cause.clone()), dir());
        assert!(
            screen.message.contains(says),
            "{cause:?}: {}",
            screen.message
        );
        assert_eq!(screen.retry(), retry, "{cause:?}");
        for always in [Action::ShowFolder, Action::CopyDetails, Action::Quit] {
            assert!(screen.actions.contains(&always), "{cause:?}");
        }
        assert!(!screen.steps.is_empty());
        assert_eq!(screen.folder, dir().display().to_string());
    }
}

/// The failures before the library opens say what to do and which of them
/// Try again can cure.
#[test]
fn startup_failures_before_the_library_map_too() {
    let locked = describe(&Failure::Locked, dir());
    assert!(locked.retry());
    assert!(locked.message.contains("Application Support/Kanso"));
    assert!(locked.steps.iter().any(|step| step.contains("Try again")));
    for failure in [
        Failure::Folder("denied".into()),
        Failure::LockFailed("denied".into()),
        Failure::Config("expected value at line 3".into()),
    ] {
        let screen = describe(&failure, dir());
        assert!(screen.retry(), "{failure:?}");
        assert!(screen.message.contains(failure_why(&failure)));
    }
    let config = describe(&Failure::Config("x".into()), dir());
    assert!(config.message.contains("config.json"));
}

fn failure_why(failure: &Failure) -> &str {
    match failure {
        Failure::Folder(why) | Failure::LockFailed(why) | Failure::Config(why) => why,
        _ => "",
    }
}

/// A wrong key and a held lock, as the real library gives them, reach the
/// window as such.
#[test]
fn real_open_errors_reach_the_window() {
    let data = tempfile::tempdir().unwrap();
    let first = kanso_sync::MemorySecrets::new();
    crate::library_open::open_store_reporting(data.path(), &first, &|_| {}).unwrap();
    let other = kanso_sync::MemorySecrets::new();
    let error = crate::library_open::open_store_reporting(data.path(), &other, &|_| {})
        .err()
        .expect("an empty keychain doesn't open an encrypted library");
    assert_eq!(classify(&error), Cause::WrongKey);

    let _held = kanso_store::lock_data_dir(data.path()).unwrap().unwrap();
    assert!(kanso_store::lock_data_dir(data.path()).unwrap().is_none());
}

/// Redaction takes out key-shaped runs and leaves words and paths.
#[test]
fn redact_hides_keys_and_keeps_the_rest() {
    let key = "9f".repeat(32);
    let text = format!("bad key {key} at /Users/me/Kanso/kanso.db, id kcl_{key}");
    let out = redact(&text);
    assert!(!out.contains(&key));
    assert!(out.contains("/Users/me/Kanso/kanso.db"));
    assert!(out.contains("bad key [hidden] at"));
    assert_eq!(redact("short words only"), "short words only");
}

/// The screen survives the line it travels in.
#[test]
fn the_screen_round_trips_as_json() {
    let screen = describe(&Failure::Library(Cause::Converting), dir());
    let json = serde_json::to_string(&screen).unwrap();
    assert_eq!(serde_json::from_str::<Screen>(&json).unwrap(), screen);
}

fn secret() -> impl Strategy<Value = String> {
    prop_oneof!["[0-9a-f]{64}", "[A-Za-z0-9+_=-]{44}", "kcl_[A-Za-z0-9]{40}",]
}

fn why() -> impl Strategy<Value = (String, String)> {
    (secret(), "[a-z ]{0,20}", "[a-z .:]{0,20}")
        .prop_map(|(key, before, after)| (key.clone(), format!("{before} {key} {after}")))
}

fn folder() -> impl Strategy<Value = PathBuf> {
    "[A-Za-z0-9 ._/-]{1,40}".prop_map(|path| PathBuf::from(format!("/{path}")))
}

proptest! {
    #![proptest_config(ProptestConfig {
        rng_seed: proptest::test_runner::RngSeed::Fixed(0x006b_616e_736f),
        ..ProptestConfig::with_cases(256)
    })]

    /// Invariants: whatever the failure and the folder, no piece of the
    /// window is empty, the buttons always include the way out, and a key
    /// that came in an error's text is in none of it (nor in the copied
    /// details).
    #[test]
    fn prop_no_message_is_empty_and_none_holds_a_secret(
        (key, text) in why(),
        folder in folder(),
        pick in 0usize..12,
    ) {
        let failure = nth_failure(pick, &text);
        let screen = describe(&failure, &folder);
        for piece in screen.texts() {
            prop_assert!(!piece.trim().is_empty());
            prop_assert!(!piece.contains(&key), "secret in {piece}");
        }
        prop_assert!(!screen.steps.is_empty());
        prop_assert!(screen.steps.iter().all(|step| !step.trim().is_empty()));
        prop_assert!(screen.actions.contains(&Action::Quit));
        prop_assert!(screen.actions.contains(&Action::ShowFolder));
        prop_assert!(screen.actions.contains(&Action::CopyDetails));
        // The same through an error as the library gives it.
        let error = opening(StoreError::EncryptFailed(text.clone()));
        let screen = describe(&Failure::Library(classify(&error)), &folder);
        for piece in screen.texts() {
            prop_assert!(!piece.contains(&key), "secret in {piece}");
        }
    }
}

fn nth_failure(pick: usize, text: &str) -> Failure {
    match pick {
        0 => Failure::Locked,
        1 => Failure::Folder(text.into()),
        2 => Failure::LockFailed(text.into()),
        3 => Failure::Config(text.into()),
        4 => Failure::Library(Cause::WrongKey),
        5 => Failure::Library(Cause::Converting),
        6 => Failure::Library(Cause::DiskFull { needed_mb: 7 }),
        7 => Failure::Library(Cause::EncryptFailed(redact(text))),
        8 => Failure::Library(Cause::Keychain(redact(text))),
        9 => Failure::Library(Cause::NoEncryption),
        10 => Failure::Library(Cause::Io(redact(text))),
        _ => Failure::Library(Cause::Other(redact(text))),
    }
}
