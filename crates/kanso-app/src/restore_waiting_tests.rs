//! A staged restore is applied by the app alone (issue #243 step 11 follow-up):
//! `kanso capture`, `serve`, `sticky`, `plugin`, the importers and the account
//! commands open the current library as it is and leave the restore for the
//! app, which can tell the person in a toast. The app applies it only while
//! the library is signed out. (`kanso-sync` has its own test, in
//! `crates/kanso-sync/tests/restore_waiting.rs`; `serve`'s binary is tested
//! in `crates/kanso-app/tests/serve_restore.rs`.)

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use kanso_sync::SecretStore as _;

use crate::library_open;
use crate::paths;

/// The app's open hands its outcome to a process-wide slot: the tests that
/// read it take turns.
static APP_OPENS: Mutex<()> = Mutex::new(());

/// A library with the note "later" written after a backup of it, and a
/// restore of that backup staged: applying it would drop "later". Gives back
/// the library's ID, which names its keychain items.
fn staged_library(dir: &Path) -> String {
    let (store, _) =
        library_open::open_store_reporting(dir, library_open::test_secrets(), &|_| {}).unwrap();
    let library_id = format!(
        "l_{:032x}",
        dir.to_string_lossy().bytes().fold(0u128, |acc, byte| acc
            .wrapping_mul(31)
            .wrapping_add(u128::from(byte)))
    );
    store
        .set_meta(kanso_sync::meta_keys::LIBRARY_ID, &library_id)
        .unwrap();
    let backup = store.backup_now(&paths::backups_dir(dir), 1_000).unwrap();
    store.set_meta("later", "yes").unwrap();
    let key = store.library_key().unwrap().clone();
    drop(store);
    kanso_store::stage_restore(&paths::db_path(dir), &backup, Some(&key), &key).unwrap();
    library_id
}

/// The staged restore still waits and the library is the current one.
fn assert_left_alone(dir: &Path) {
    assert!(
        kanso_store::restore_staged(&paths::db_path(dir)),
        "the restore is still waiting"
    );
    let store = library_open::open_store(dir).unwrap();
    assert_eq!(store.meta("later").unwrap().as_deref(), Some("yes"));
    assert!(
        kanso_store::restore_staged(&paths::db_path(dir)),
        "opening again changed nothing"
    );
}

#[test]
fn open_store_leaves_a_staged_restore_alone() {
    let dir = tempfile::tempdir().unwrap();
    staged_library(dir.path());
    drop(library_open::open_store(dir.path()).unwrap());
    assert_left_alone(dir.path());
}

#[test]
fn the_account_commands_leave_a_staged_restore_alone() {
    let dir = tempfile::tempdir().unwrap();
    staged_library(dir.path());
    drop(library_open::open_store_with(dir.path(), library_open::test_secrets()).unwrap());
    assert_left_alone(dir.path());
}

#[test]
fn capture_leaves_a_staged_restore_alone() {
    let dir = tempfile::tempdir().unwrap();
    staged_library(dir.path());
    crate::capture_cli::run_cli(dir.path(), &["a thought".into()]).unwrap();
    assert_left_alone(dir.path());
}

#[test]
fn sticky_leaves_a_staged_restore_alone() {
    let dir = tempfile::tempdir().unwrap();
    staged_library(dir.path());
    crate::sticky_cli::run_cli(dir.path(), &["a sticky".into()]).unwrap();
    assert_left_alone(dir.path());
}

#[test]
fn the_plugin_commands_leave_a_staged_restore_alone() {
    let dir = tempfile::tempdir().unwrap();
    staged_library(dir.path());
    crate::plugins::run_cli(dir.path(), &["list".into()]).unwrap();
    assert_left_alone(dir.path());
}

#[test]
fn the_importers_leave_a_staged_restore_alone() {
    let dir = tempfile::tempdir().unwrap();
    staged_library(dir.path());
    // An empty folder to read: the import finds nothing, but it opens the
    // library first, and that is the step under test.
    let source = tempfile::tempdir().unwrap();
    let args = [
        "apple-notes".to_owned(),
        source.path().display().to_string(),
    ];
    let _ = crate::library_import::run_cli(dir.path(), &args);
    assert_left_alone(dir.path());
}

/// The line the other programs print.
#[test]
fn the_waiting_line_says_to_open_kanso() {
    assert_eq!(
        library_open::RESTORE_WAITING,
        "A restore is waiting; open Kanso to apply it."
    );
}

/// The app's own open still applies it, and says so for its toast.
#[test]
fn the_app_still_applies_it() {
    let _turn = APP_OPENS
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let dir = tempfile::tempdir().unwrap();
    staged_library(dir.path());
    let (store, _) = library_open::open_store_progress(dir.path(), &|_| {}).unwrap();
    assert_eq!(store.meta("later").unwrap(), None);
    drop(store);
    assert!(!kanso_store::restore_staged(&paths::db_path(dir.path())));
    assert!(matches!(library_open::take_restore_outcome(), Some(Ok(()))));
}

/// Regression (PR #301 review): the sheet refuses while signed in, but a
/// sign-in after staging (the Account page, `kanso account sign-in`) was
/// still followed by the restore at the next launch, and the restored notes
/// synced with nobody asked. The app's launch leaves it waiting while the
/// keychain holds a session for the library, signed in or with a sign-in
/// waiting on its key, says why, and applies it once signed out.
#[test]
fn the_app_waits_while_the_library_is_signed_in() {
    let _turn = APP_OPENS
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    for item in [
        kanso_sync::keychain::account_item,
        kanso_sync::keychain::pending_item,
    ] {
        let dir = tempfile::tempdir().unwrap();
        let library_id = staged_library(dir.path());
        let secrets = library_open::test_secrets();
        secrets.set(&item(&library_id), "{}").unwrap();

        let (store, _) = library_open::open_store_progress(dir.path(), &|_| {}).unwrap();
        assert_eq!(store.meta("later").unwrap().as_deref(), Some("yes"));
        drop(store);
        assert!(kanso_store::restore_staged(&paths::db_path(dir.path())));
        let Some(Err((why, again))) = library_open::take_restore_outcome() else {
            panic!("the launch says the restore waits");
        };
        assert!(again, "it is tried again at the next launch");
        assert!(why.contains("Sign out"), "{why}");

        secrets.delete(&item(&library_id)).unwrap();
        let (store, _) = library_open::open_store_progress(dir.path(), &|_| {}).unwrap();
        assert_eq!(store.meta("later").unwrap(), None, "signed out, it applies");
        assert!(matches!(library_open::take_restore_outcome(), Some(Ok(()))));
    }
}

// MARK: Who may apply a restore

/// A Rust source file reduced to its code: identifiers with the function
/// each sits in. Comments, strings, chars and raw strings are skipped, so a
/// name mentioned in prose or text never counts, and a name used anywhere in
/// code (a call, a `use … as` alias, a function pointer) always does.
fn code_names(source: &str) -> Vec<(String, Option<String>)> {
    let chars: Vec<char> = source.chars().collect();
    let mut found = Vec::new();
    // Function bodies open: (name, the depth its body opened at).
    let mut scopes: Vec<(String, usize)> = Vec::new();
    let mut pending_fn: Option<String> = None;
    let mut last_was_fn = false;
    let mut depth = 0usize;
    let mut i = 0;
    let at = |i: usize| chars.get(i).copied().unwrap_or('\0');
    while i < chars.len() {
        let c = chars[i];
        if c == '/' && at(i + 1) == '/' {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }
        if c == '/' && at(i + 1) == '*' {
            let mut nested = 1;
            i += 2;
            while i < chars.len() && nested > 0 {
                if chars[i] == '/' && at(i + 1) == '*' {
                    nested += 1;
                    i += 2;
                } else if chars[i] == '*' && at(i + 1) == '/' {
                    nested -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            continue;
        }
        // Raw strings: r"…", r#"…"#, br#"…"#.
        let raw_start = if c == 'r' {
            Some(i + 1)
        } else if c == 'b' && at(i + 1) == 'r' {
            Some(i + 2)
        } else {
            None
        };
        if let Some(mut j) = raw_start
            && (i == 0 || !is_ident(chars[i - 1]))
        {
            let mut hashes = 0;
            while at(j) == '#' {
                hashes += 1;
                j += 1;
            }
            if at(j) == '"' {
                j += 1;
                loop {
                    if j >= chars.len() {
                        break;
                    }
                    if chars[j] == '"' && (1..=hashes).all(|k| at(j + k) == '#') {
                        j += 1 + hashes;
                        break;
                    }
                    j += 1;
                }
                i = j;
                continue;
            }
        }
        if c == '"' {
            i += 1;
            while i < chars.len() && chars[i] != '"' {
                if chars[i] == '\\' {
                    i += 1;
                }
                i += 1;
            }
            i += 1;
            continue;
        }
        if c == '\'' {
            // A char literal ('x', '\n', '\u{1F600}'), not a lifetime.
            if at(i + 1) == '\\' {
                // Past the quote, the backslash and the escaped character
                // (itself maybe a quote).
                i += 3;
                while i < chars.len() && chars[i] != '\'' {
                    i += 1;
                }
                i += 1;
                continue;
            }
            if at(i + 2) == '\'' {
                i += 3;
                continue;
            }
            i += 1;
            // A lifetime's name is no code name.
            while i < chars.len() && is_ident(chars[i]) {
                i += 1;
            }
            continue;
        }
        if is_ident(c) && !c.is_ascii_digit() {
            let start = i;
            while i < chars.len() && is_ident(chars[i]) {
                i += 1;
            }
            let name: String = chars[start..i].iter().collect();
            if last_was_fn {
                pending_fn = Some(name.clone());
            }
            last_was_fn = name == "fn";
            found.push((name, scopes.last().map(|(name, _)| name.clone())));
            continue;
        }
        if c.is_ascii_digit() {
            while i < chars.len() && is_ident(chars[i]) {
                i += 1;
            }
            last_was_fn = false;
            continue;
        }
        if !c.is_whitespace() {
            last_was_fn = false;
        }
        match c {
            '{' => {
                if let Some(name) = pending_fn.take() {
                    scopes.push((name, depth));
                }
                depth += 1;
            }
            '}' => {
                depth = depth.saturating_sub(1);
                if scopes.last().is_some_and(|(_, opened)| *opened == depth) {
                    scopes.pop();
                }
            }
            // A declaration with no body.
            ';' => pending_fn = None,
            _ => {}
        }
        i += 1;
    }
    found
}

fn is_ident(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Every `.rs` file under `dir`, at any depth.
fn rust_files(dir: &Path, into: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, into);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            into.push(path);
        }
    }
}

/// Who may name each step of the path to an applied restore: the file
/// (relative to the crate's `src`) and the function the name sits in.
/// `None` is the function's own definition line, outside any body.
fn allowed() -> BTreeMap<&'static str, Vec<(&'static str, Option<&'static str>)>> {
    BTreeMap::from([
        (
            "apply_staged_restore",
            vec![("library_open.rs", Some("apply_pending_restore"))],
        ),
        (
            "apply_pending_restore",
            vec![
                ("library_open.rs", None),
                ("library_open.rs", Some("open_store_applying_restore")),
                // `library_open`'s own tests of the step.
                (
                    "library_open.rs",
                    Some("a_staged_restore_replaces_the_library_at_the_next_open"),
                ),
                (
                    "library_open.rs",
                    Some("a_restore_that_cannot_apply_now_waits_and_the_library_opens"),
                ),
            ],
        ),
        (
            "open_store_applying_restore",
            vec![
                ("library_open.rs", None),
                ("library_open.rs", Some("open_store_progress")),
            ],
        ),
        (
            "open_store_progress",
            vec![
                ("library_open.rs", None),
                ("main.rs", Some("open_library_showing_progress")),
                // This file's tests of the app's open.
                ("restore_waiting_tests.rs", Some("the_app_still_applies_it")),
                (
                    "restore_waiting_tests.rs",
                    Some("the_app_waits_while_the_library_is_signed_in"),
                ),
            ],
        ),
        (
            "open_library_showing_progress",
            vec![("main.rs", None), ("main.rs", Some("run_app"))],
        ),
    ])
}

/// Every place in `src` that names a step of the path to an applied
/// restore and isn't allowed to: (file, name, function).
fn trespassers(src: &Path) -> Vec<(String, String, Option<String>)> {
    let rules = allowed();
    let mut files = Vec::new();
    rust_files(src, &mut files);
    files.sort();
    let mut found = Vec::new();
    for path in files {
        let file = path
            .strip_prefix(src)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let text = std::fs::read_to_string(&path).unwrap();
        for (name, inside) in code_names(&text) {
            let Some(allowed) = rules.get(name.as_str()) else {
                continue;
            };
            let ok = allowed
                .iter()
                .any(|(f, scope)| *f == file && *scope == inside.as_deref());
            if !ok {
                found.push((file.clone(), name, inside));
            }
        }
    }
    found
}

/// `kanso serve`, and anything else that opens the library through the
/// app's own call, would apply a restore without the app's checks and
/// toast. Only the app's launch reaches it: `main.rs`'s `run_app`, through
/// `open_library_showing_progress`, `open_store_progress`,
/// `open_store_applying_restore` and `apply_pending_restore`, to the store's
/// `apply_staged_restore`. Every `.rs` file in the crate, at any depth, is
/// read as code.
#[test]
fn only_the_app_opens_the_library_applying_a_restore() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(trespassers(&src), []);
    // And each step is still there to check (a rename would empty the scan).
    let main = std::fs::read_to_string(src.join("main.rs")).unwrap();
    assert!(code_names(&main).iter().any(|(name, inside)| {
        name == "open_store_progress" && inside.as_deref() == Some("open_library_showing_progress")
    }));
}

/// Nothing outside the store and the app's launch may apply a restore:
/// every other crate in the workspace (`kanso-sync`, the engine, the phones)
/// never names `apply_staged_restore` in code.
#[test]
fn no_other_crate_applies_a_restore() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let mut offenders = Vec::new();
    for entry in std::fs::read_dir(crates).unwrap().flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if matches!(name.as_str(), "kanso-store" | "kanso-app") {
            continue;
        }
        let src = entry.path().join("src");
        if !src.is_dir() {
            continue;
        }
        let mut files = Vec::new();
        rust_files(&src, &mut files);
        for path in files {
            let text = std::fs::read_to_string(&path).unwrap();
            if code_names(&text)
                .iter()
                .any(|(name, _)| name == "apply_staged_restore")
            {
                offenders.push(path.display().to_string());
            }
        }
    }
    assert_eq!(offenders, Vec::<String>::new());
}

/// The scanner can't be fooled: names in comments, docs, strings, raw
/// strings and chars don't count; a call from another function, an alias
/// and a function pointer do, each in the function it sits in.
#[test]
fn the_scanner_reads_code_not_text() {
    let source = r##"
        // open_store_progress in a comment
        /* open_store_progress /* nested */ still a comment */
        /// open_store_progress in a doc
        fn quiet<'a>(x: &'a str) -> char {
            let _ = "open_store_progress \" escaped";
            let _ = r#"open_store_progress "quoted""#;
            let _ = b"open_store_progress";
            let _ = ['\'', '"', '\u{7b}'];
            let _ = "open_store_progress";
            'o'
        }
        fn sneaky() {
            use crate::library_open::open_store_progress as innocent;
            let f = open_store_progress;
            if true { innocent(); }
        }
        fn after() { open_store_progress(); }
    "##;
    let names: Vec<(String, Option<String>)> = code_names(source)
        .into_iter()
        .filter(|(name, _)| name == "open_store_progress")
        .collect();
    assert_eq!(
        names,
        [
            ("open_store_progress".to_owned(), Some("sneaky".to_owned())),
            ("open_store_progress".to_owned(), Some("sneaky".to_owned())),
            ("open_store_progress".to_owned(), Some("after".to_owned())),
        ]
    );
}
