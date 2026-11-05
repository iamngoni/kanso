//! `kanso-import [--library-dir DIR] [--db FILE] --data-dir DIR`
//!
//! Imports a PouchDB notes library into `DIR/kanso.db` and prints a report.
//! The app that owns it may keep running; its files are only read.
//! Credentials from its config are never printed.

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Context as _, bail};
use kanso_import::safe_storage::SAFE_STORAGE_SERVICE;
use kanso_import::{
    LocalServerCredentials, Phase, default_pouch_library_dir, find_pouch_db, import,
    read_local_server_config,
};
use kanso_store::{LibraryFile, Store};

const USAGE: &str = "\
Usage: kanso-import [--library-dir DIR] [--db FILE] --data-dir DIR

Imports the PouchDB notes library found on this computer into
DIR/kanso.db. Safe to run again: documents already imported at the same
revision are left alone.

  --library-dir DIR  The library's data directory (default: the one found
                     on this computer)
  --db FILE          The library's database file (default: the single
                     db/<32 hex>.sqlite in the library directory)
  --data-dir DIR     Kanso's data directory (required)";

struct Args {
    library_dir: Option<PathBuf>,
    db: Option<PathBuf>,
    data_dir: PathBuf,
}

fn parse_args() -> anyhow::Result<Option<Args>> {
    let mut library_dir = None;
    let mut db = None;
    let mut data_dir = None;
    let mut args = std::env::args_os().skip(1);
    while let Some(arg) = args.next() {
        let slot = match arg.to_str() {
            Some("-h" | "--help") => return Ok(None),
            Some("--library-dir") => &mut library_dir,
            Some("--db") => &mut db,
            Some("--data-dir") => &mut data_dir,
            _ => bail!("unexpected argument {arg:?}\n\n{USAGE}"),
        };
        let value = args
            .next()
            .with_context(|| format!("{} needs a value", arg.to_string_lossy()))?;
        *slot = Some(PathBuf::from(value));
    }
    let data_dir = data_dir.with_context(|| format!("--data-dir is required\n\n{USAGE}"))?;
    Ok(Some(Args {
        library_dir,
        db,
        data_dir,
    }))
}

fn phase_name(phase: Phase) -> &'static str {
    match phase {
        Phase::Snapshot => "snapshot",
        Phase::Books => "notebooks",
        Phase::Tags => "tags",
        Phase::Files => "files",
        Phase::Notes => "notes",
        Phase::Activities => "activity log",
        Phase::Verify => "verify",
    }
}

fn describe(credentials: &LocalServerCredentials) -> String {
    match credentials {
        LocalServerCredentials::None => "none".to_owned(),
        LocalServerCredentials::Basic { .. } => "username and password present".to_owned(),
        LocalServerCredentials::Encrypted { .. } => format!(
            "encrypted with Electron safeStorage (needs the \"{SAFE_STORAGE_SERVICE}\" Keychain item)"
        ),
        LocalServerCredentials::Unrecognized => "in an unrecognized format".to_owned(),
    }
}

fn run() -> anyhow::Result<ExitCode> {
    let Some(args) = parse_args()? else {
        println!("{USAGE}");
        return Ok(ExitCode::SUCCESS);
    };
    let library_dir = match args.library_dir {
        Some(dir) => dir,
        None => default_pouch_library_dir().context("HOME is not set; pass --library-dir")?,
    };
    let source = match args.db {
        Some(db) => db,
        None => find_pouch_db(&library_dir)?,
    };
    let Some(_lock) = kanso_store::lock_data_dir(&args.data_dir)? else {
        bail!(
            "{} is in use by another Kanso process; quit it first",
            args.data_dir.display()
        );
    };
    // This tool has no access to the library key, so it only adds to a
    // plain library from before encryption; it never makes one, which would
    // leave a new library unencrypted (issue #243).
    let db = args.data_dir.join("kanso.db");
    match LibraryFile::of(&db) {
        LibraryFile::Plain => {}
        LibraryFile::New => bail!(
            "{} has no library; open Kanso once to make it, or use `kanso import`",
            args.data_dir.display()
        ),
        LibraryFile::Encrypted => bail!(
            "{} is encrypted and this tool has no key for it; use `kanso import`",
            db.display()
        ),
    }
    let store = Store::open(&db)?;

    eprintln!("Importing {}", source.display());
    let mut finished = None;
    let report = import(&source, &store, |progress| {
        if progress.done == progress.total && finished != Some(progress.phase) {
            finished = Some(progress.phase);
            eprintln!("  {}: {}", phase_name(progress.phase), progress.total);
        }
    })?;
    println!("{report}");

    match read_local_server_config(&library_dir) {
        Ok(Some(server)) => println!(
            "Local API server: {} on {}:{}; credentials {}.",
            if server.enabled {
                "enabled"
            } else {
                "disabled"
            },
            server.bind_address,
            server.port,
            describe(&server.credentials),
        ),
        Ok(None) => println!("Local API server: not configured."),
        Err(err) => eprintln!("Could not read the library's server settings: {err:#}"),
    }

    Ok(if report.is_clean() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(2)
    })
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(err) => {
            eprintln!("kanso-import: {err:#}");
            ExitCode::FAILURE
        }
    }
}
