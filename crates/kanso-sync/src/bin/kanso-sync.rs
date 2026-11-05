//! Manual runs against a library's account (docs/ACCOUNTS.md › Command line).
//!
//! ```text
//! status   --data-dir DIR                             account, server, device, cursor, pending changes
//! sync     --data-dir DIR [--full]                    sync once with this library's session
//! verify   --data-dir DIR [--repair] [--legacy M.json --out R.json]
//! estimate --data-dir DIR                             charged document and file bytes this library needs
//! unbind   --data-dir DIR                             sign out locally and reset bookkeeping for the old app
//! ```
//!
//! Each command takes the data-directory lock, so quit Kanso first. `sync`
//! and `verify` read the library's `sync-account` item from the library's
//! secret store; `unbind` deletes it. The store is the system's (the
//! Keychain, which on macOS asks once) unless the library's `config.json`
//! says `"sync": {"secretStore": "file"}`: then it is `secrets.json` in the
//! data directory, sealed under the key in `$KANSO_SECRETS_KEY_FILE` when
//! that is set.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context, Result, anyhow, bail};
use kanso_store::Store;
use kanso_sync::{
    DeviceIdentity, FileSecrets, HttpBackend, Keychain, SECRETS_FILE, SECRETS_KEY_FILE_ENV,
    SecretStore, Session, SyncOptions, keychain, meta_keys, problem, report_problem,
};

const USAGE: &str = "usage:
  kanso-sync status   --data-dir DIR
  kanso-sync sync     --data-dir DIR [--full]
  kanso-sync verify   --data-dir DIR [--repair] [--legacy M.json --out R.json]
  kanso-sync estimate --data-dir DIR
  kanso-sync unbind   --data-dir DIR";

fn main() -> ExitCode {
    match parse(&std::env::args().skip(1).collect::<Vec<_>>()).and_then(run) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(err) => {
            eprintln!("kanso-sync: {err:#}");
            ExitCode::FAILURE
        }
    }
}

/// A parsed command line.
#[derive(Debug, PartialEq, Eq)]
enum Command {
    Help,
    Status {
        data_dir: PathBuf,
    },
    Sync {
        data_dir: PathBuf,
        full: bool,
    },
    Verify {
        data_dir: PathBuf,
        repair: bool,
        legacy: Option<(PathBuf, PathBuf)>,
    },
    Estimate {
        data_dir: PathBuf,
    },
    Unbind {
        data_dir: PathBuf,
    },
}

fn parse(args: &[String]) -> Result<Command> {
    let Some(command) = args.first() else {
        return Ok(Command::Help);
    };
    if matches!(command.as_str(), "-h" | "--help" | "help") {
        return Ok(Command::Help);
    }
    let mut data_dir = None;
    let (mut full, mut repair) = (false, false);
    let (mut legacy, mut out) = (None, None);
    let mut rest = args[1..].iter();
    while let Some(arg) = rest.next() {
        let mut value = |name: &str| -> Result<PathBuf> {
            Ok(rest
                .next()
                .with_context(|| format!("{name} needs a value"))?
                .into())
        };
        match arg.as_str() {
            "--data-dir" => data_dir = Some(value("--data-dir")?),
            "--full" if command == "sync" => full = true,
            "--repair" if command == "verify" => repair = true,
            "--legacy" if command == "verify" => legacy = Some(value("--legacy")?),
            "--out" if command == "verify" => out = Some(value("--out")?),
            other => bail!("unknown argument {other}\n{USAGE}"),
        }
    }
    let data_dir = data_dir.ok_or_else(|| anyhow!("--data-dir is required\n{USAGE}"))?;
    Ok(match command.as_str() {
        "status" => Command::Status { data_dir },
        "sync" => Command::Sync { data_dir, full },
        "verify" => {
            let legacy = match (legacy, out) {
                (Some(manifest), Some(out)) => Some((manifest, out)),
                (None, None) => None,
                _ => bail!("--legacy and --out go together\n{USAGE}"),
            };
            Command::Verify {
                data_dir,
                repair,
                legacy,
            }
        }
        "estimate" => Command::Estimate { data_dir },
        "unbind" => Command::Unbind { data_dir },
        other => bail!("unknown command {other}\n{USAGE}"),
    })
}

/// Runs a command; `Ok(false)` exits 1 without an error message.
fn run(command: Command) -> Result<bool> {
    match command {
        Command::Help => {
            println!("{USAGE}");
            Ok(true)
        }
        Command::Status { data_dir } => {
            let (_lock, store) = open(&data_dir)?;
            status(&store)?;
            Ok(true)
        }
        Command::Sync { data_dir, full } => {
            let (_lock, store) = open(&data_dir)?;
            let session = session(&store, &*secrets(&data_dir)?)?;
            if full {
                let pairs = kanso_sync::binding::guard_pairs(&session);
                store.set_meta_if(
                    meta_keys::LAST_SEQ,
                    "0",
                    &kanso_sync::binding::guard(&pairs),
                )?;
            }
            match kanso_sync::sync(&store, &HttpBackend, &session, &SyncOptions::default()) {
                Ok(report) => {
                    println!("{report:#?}");
                    if let Some(problem) = report_problem(&report) {
                        println!("problem: {problem:?}");
                    }
                    Ok(true)
                }
                Err(err) => {
                    let problem = problem(&err);
                    Err(anyhow!(err)).with_context(|| format!("sync failed ({problem:?})"))
                }
            }
        }
        Command::Verify {
            data_dir,
            repair,
            legacy,
        } => {
            let (_lock, store) = open(&data_dir)?;
            let session = session(&store, &*secrets(&data_dir)?)?;
            let manifest = legacy.as_ref().map(|(manifest, _)| manifest.as_path());
            let report = kanso_sync::verify(&store, &HttpBackend, &session, manifest, repair)?;
            println!("matched        {}", report.matched);
            println!("missing        {}", report.missing);
            println!("extra          {}", report.extra);
            println!("different      {}", report.different);
            println!("undecryptable  {}", report.undecryptable);
            println!("still pending  {}", report.pending);
            if repair {
                println!("repaired       {}", report.repaired);
            }
            if let (Some(result), Some((_, out))) = (&report.legacy, &legacy) {
                println!(
                    "legacy rows    {} checked, {} failed (manifest {})",
                    result.rows,
                    result.failed.len(),
                    result.manifest_sha256
                );
                std::fs::write(out, serde_json::to_vec_pretty(result)?)
                    .with_context(|| format!("writing {}", out.display()))?;
            }
            Ok(report.ok())
        }
        Command::Estimate { data_dir } => {
            let (_lock, store) = open(&data_dir)?;
            let estimate = kanso_sync::estimate(&store)?;
            println!(
                "documents   {} ({} charged)",
                estimate.docs,
                megabytes(estimate.doc_bytes)
            );
            println!(
                "files       {} ({} charged)",
                estimate.files,
                megabytes(estimate.file_bytes)
            );
            Ok(true)
        }
        Command::Unbind { data_dir } => {
            let (_lock, store) = open(&data_dir)?;
            kanso_sync::unbind(&store, &*secrets(&data_dir)?)?;
            println!(
                "Unbound {}: signed out locally, live documents will go to the old server again, and its cursor is 0.",
                data_dir.display()
            );
            Ok(true)
        }
    }
}

/// Takes the data-directory lock and opens the library.
fn open(data_dir: &Path) -> Result<(kanso_store::DataDirLock, Store)> {
    if !data_dir.join("kanso.db").exists() {
        bail!("{} has no kanso.db", data_dir.display());
    }
    let lock = kanso_store::lock_data_dir(data_dir)?
        .ok_or_else(|| anyhow!("{} is in use by Kanso; quit it first", data_dir.display()))?;
    let store = kanso_sync::open_library(&data_dir.join("kanso.db"), &*secrets(data_dir)?)?.store;
    Ok((lock, store))
}

/// The library's binding: (library ID, origin, account ID).
fn binding(store: &Store) -> Result<Option<(String, String, String)>> {
    Ok(
        match (
            store.meta(meta_keys::LIBRARY_ID)?,
            store.meta(meta_keys::ORIGIN)?,
            store.meta(meta_keys::ACCOUNT_ID)?,
        ) {
            (Some(library), Some(origin), Some(account)) => Some((library, origin, account)),
            _ => None,
        },
    )
}

/// Where a library's secrets live: `sync.secretStore` in its `config.json`.
#[derive(Debug, PartialEq, Eq)]
enum StoreKind {
    /// The Keychain, Credential Manager or Secret Service.
    System,
    /// `secrets.json` in the data directory.
    File,
}

/// The store `config.json` names, given its text (`None` when there is no
/// file). Only `"file"` picks the file; anything else, or nothing, is the
/// system's store, as before the key existed.
fn store_kind(config: Option<&str>) -> Result<StoreKind> {
    let Some(config) = config else {
        return Ok(StoreKind::System);
    };
    let config: serde_json::Value =
        serde_json::from_str(config).context("config.json isn't valid JSON")?;
    Ok(
        match config
            .pointer("/sync/secretStore")
            .and_then(|kind| kind.as_str())
        {
            Some("file") => StoreKind::File,
            _ => StoreKind::System,
        },
    )
}

/// The library's secret store, as its `config.json` says.
fn secrets(data_dir: &Path) -> Result<Box<dyn SecretStore>> {
    let path = data_dir.join("config.json");
    let config = match std::fs::read_to_string(&path) {
        Ok(text) => Some(text),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => None,
        Err(err) => return Err(err).with_context(|| format!("reading {}", path.display())),
    };
    Ok(match store_kind(config.as_deref())? {
        StoreKind::System => Box::new(Keychain::default()),
        StoreKind::File => {
            let key_file = std::env::var_os(SECRETS_KEY_FILE_ENV)
                .filter(|value| !value.is_empty())
                .map(PathBuf::from);
            Box::new(FileSecrets::open(
                data_dir.join(SECRETS_FILE),
                key_file.as_deref(),
            )?)
        }
    })
}

/// The library's session from its secret store.
fn session(store: &Store, secrets: &dyn SecretStore) -> Result<Session> {
    let (library, origin, account) =
        binding(store)?.ok_or_else(|| anyhow!("this library isn't bound to an account"))?;
    keychain::load_session(secrets, &library, &origin, &account)?
        .ok_or_else(|| anyhow!("this library is signed out; sign in from Kanso first"))
}

fn status(store: &Store) -> Result<()> {
    let pending = store.pending_count()?;
    let Some((library, origin, account)) = binding(store)? else {
        println!("account   none (not signed in)");
        println!("pending   {pending} changes");
        return Ok(());
    };
    let meta = |key: &str| -> Result<String> { Ok(store.meta(key)?.unwrap_or_else(|| "-".into())) };
    println!("account   {} ({account})", meta(meta_keys::USERNAME)?);
    println!("server    {} ({origin})", meta(meta_keys::SERVER)?);
    if let Some(secret) = store
        .meta(meta_keys::DEVICE_SECRET)?
        .and_then(|hex| parse_hex32(&hex))
    {
        let device = DeviceIdentity::from_parts(&library, "", secret);
        println!("device    {}", device.device_id(&account));
    }
    println!("library   {library}");
    println!("cursor    {}", meta(meta_keys::LAST_SEQ)?);
    println!("epoch     {}", meta(meta_keys::EPOCH)?);
    println!("pending   {pending} changes not on the server");
    Ok(())
}

fn parse_hex32(hex: &str) -> Option<[u8; 32]> {
    if hex.len() != 64 {
        return None;
    }
    let mut out = [0u8; 32];
    for (i, byte) in out.iter_mut().enumerate() {
        *byte = u8::from_str_radix(hex.get(2 * i..2 * i + 2)?, 16).ok()?;
    }
    Some(out)
}

fn megabytes(bytes: u64) -> String {
    format!("{:.1} MB", bytes as f64 / 1_000_000.0)
}

#[cfg(test)]
mod tests {
    // no-prop-test: argument and config plumbing; the commands' bodies are
    // tested in the library (sync, verify, estimate, unbind), and the stores
    // in `keychain`. Nothing here touches the Keychain.
    use super::*;

    fn args(line: &str) -> Vec<String> {
        line.split_whitespace().map(str::to_owned).collect()
    }

    #[test]
    fn commands_parse() {
        assert_eq!(parse(&[]).unwrap(), Command::Help);
        assert_eq!(
            parse(&args("sync --data-dir /d --full")).unwrap(),
            Command::Sync {
                data_dir: "/d".into(),
                full: true
            }
        );
        assert_eq!(
            parse(&args(
                "verify --data-dir /d --repair --legacy m.json --out r.json"
            ))
            .unwrap(),
            Command::Verify {
                data_dir: "/d".into(),
                repair: true,
                legacy: Some(("m.json".into(), "r.json".into()))
            }
        );
        assert_eq!(
            parse(&args("estimate --data-dir /d")).unwrap(),
            Command::Estimate {
                data_dir: "/d".into()
            }
        );
        assert_eq!(
            parse(&args("unbind --data-dir /d")).unwrap(),
            Command::Unbind {
                data_dir: "/d".into()
            }
        );
        assert_eq!(
            parse(&args("status --data-dir /d")).unwrap(),
            Command::Status {
                data_dir: "/d".into()
            }
        );
    }

    #[test]
    fn bad_command_lines_are_refused() {
        assert!(parse(&args("sync")).is_err());
        assert!(parse(&args("status --data-dir /d --full")).is_err());
        assert!(parse(&args("verify --data-dir /d --legacy m.json")).is_err());
        assert!(parse(&args("frobnicate --data-dir /d")).is_err());
        assert!(parse(&args("sync --data-dir")).is_err());
    }

    #[test]
    fn the_config_picks_the_secret_store() {
        assert_eq!(store_kind(None).unwrap(), StoreKind::System);
        assert_eq!(store_kind(Some("{}")).unwrap(), StoreKind::System);
        assert_eq!(
            store_kind(Some(r#"{"sync":{"enabled":true}}"#)).unwrap(),
            StoreKind::System
        );
        assert_eq!(
            store_kind(Some(r#"{"sync":{"secretStore":"system"}}"#)).unwrap(),
            StoreKind::System
        );
        assert_eq!(
            store_kind(Some(r#"{"sync":{"secretStore":"file"}}"#)).unwrap(),
            StoreKind::File
        );
        assert_eq!(
            store_kind(Some(r#"{"sync":{"secretStore":1}}"#)).unwrap(),
            StoreKind::System
        );
        assert!(store_kind(Some("{")).is_err());
    }

    #[test]
    fn hex_secrets_parse() {
        assert_eq!(parse_hex32(&"ab".repeat(32)), Some([0xab; 32]));
        assert_eq!(parse_hex32("ab"), None);
        assert_eq!(parse_hex32(&"zz".repeat(32)), None);
    }
}
