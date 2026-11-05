//! `kanso api-key create | list | revoke`: API keys without the app, for a
//! library on a headless server (`kanso serve` refuses every request until
//! the library has a key, and keys come only from here and from Settings ›
//! Local API › API keys; they never sync).
//!
//! It uses the same `KeyStore` as Settings. `create` prints the key, and
//! nothing else, on standard output; everything else it says goes to
//! standard error, so `KEY=$(kanso api-key create --name N)` holds exactly
//! the key. The key is never logged and never part of an error.
//!
//! It takes the data-folder lock, as `kanso serve` and the app do, so it
//! refuses while either is running: a running server keeps its keys in
//! memory, and a key made or revoked behind its back would not take effect
//! until it restarted.

use std::io::Write;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context as _, Result, bail};
use kanso_server::{KeyAccess, KeyId, KeyStore, NewKey};
use kanso_store::{ApiKeyRow, Store, lock_data_dir};

use crate::capture_cli::pick_book_for;

pub const USAGE: &str = "\
Usage: kanso api-key create --name <name> [--access read|suggest|write]
                            [--scope <notebook name or id>]… [--data-dir <dir>]
       kanso api-key list [--data-dir <dir>]
       kanso api-key revoke <key id or name> [--data-dir <dir>]

Makes, lists and revokes the API keys of a library, for `kanso serve` and
`kanso mcp` on a machine with no window. Stop Kanso and `kanso serve` first:
these commands need the data folder to themselves.

create   Prints the key, and only the key, on standard output. It is shown
         this once: Kanso keeps just a hash. --access is read (the
         default), suggest (edits wait for you to review them) or write.
         Repeat --scope to limit the key to those notebooks; without it the
         key reaches all of them. Locked notes are never shown to any key.
list     The keys that work, with what each may do and when it was last used.
revoke   Stops a key for good. It takes the key's id (from `list`) or its
         name.";

/// What was asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Help,
    Create {
        name: String,
        access: KeyAccess,
        /// Notebooks by name or id; empty is all of them.
        scopes: Vec<String>,
    },
    List,
    Revoke {
        target: String,
    },
}

/// The value of option `name` at `args[ix]`: `--name=value` or `--name
/// value`. Returns the value and how many arguments it took.
fn value_of(args: &[String], ix: usize, name: &str) -> Result<Option<(String, usize)>, String> {
    let arg = &args[ix];
    if arg == name {
        let value = args.get(ix + 1).ok_or_else(|| {
            format!("{name} needs a value. Add one after it, or see `kanso api-key help`.")
        })?;
        return Ok(Some((value.clone(), 2)));
    }
    Ok(arg
        .strip_prefix(name)
        .and_then(|rest| rest.strip_prefix('='))
        .map(|value| (value.to_owned(), 1)))
}

/// Reads the arguments after `api-key`. The error is a sentence that says
/// what to change. Pure.
pub fn parse(args: &[String]) -> Result<Command, String> {
    let Some((verb, rest)) = args.split_first() else {
        return Err("say what to do: create, list or revoke. See `kanso api-key help`.".into());
    };
    match verb.as_str() {
        "help" | "--help" | "-h" => Ok(Command::Help),
        "create" => parse_create(rest),
        "list" => match rest.first() {
            None => Ok(Command::List),
            Some(other) => Err(format!(
                "`list` doesn't take `{other}`. Run `kanso api-key list` on its own."
            )),
        },
        "revoke" => match rest {
            [target] if !target.starts_with('-') && !target.trim().is_empty() => {
                Ok(Command::Revoke {
                    target: target.trim().to_owned(),
                })
            }
            _ => Err(
                "`revoke` takes one key: its id or its name. Run `kanso api-key list` to see them."
                    .into(),
            ),
        },
        other => Err(format!(
            "`{other}` isn't a command. Use create, list or revoke."
        )),
    }
}

fn parse_create(args: &[String]) -> Result<Command, String> {
    let mut name = None;
    let mut access = KeyAccess::Read;
    let mut scopes: Vec<String> = Vec::new();
    let mut ix = 0;
    while ix < args.len() {
        if let Some((value, took)) = value_of(args, ix, "--name")? {
            name = Some(value);
            ix += took;
        } else if let Some((value, took)) = value_of(args, ix, "--access")? {
            access = KeyAccess::parse(value.trim()).ok_or_else(|| {
                format!(
                    "--access is read, suggest or write, not “{}”.",
                    value.trim()
                )
            })?;
            ix += took;
        } else if let Some((value, took)) = value_of(args, ix, "--scope")? {
            let value = value.trim().to_owned();
            if value.is_empty() {
                return Err("--scope needs a notebook's name or id.".into());
            }
            if !scopes.contains(&value) {
                scopes.push(value);
            }
            ix += took;
        } else {
            return Err(format!(
                "`create` doesn't take `{}`. See `kanso api-key help`.",
                args[ix]
            ));
        }
    }
    let name = name
        .map(|name| name.trim().to_owned())
        .filter(|name| !name.is_empty())
        .ok_or("`create` needs --name, such as --name \"Backup script\". Name the program that will use the key.")?;
    Ok(Command::Create {
        name,
        access,
        scopes,
    })
}

/// The arguments that read back as `command`, for tests.
#[cfg(test)]
fn unparse(command: &Command) -> Vec<String> {
    match command {
        Command::Help => vec!["help".into()],
        Command::List => vec!["list".into()],
        Command::Revoke { target } => vec!["revoke".into(), target.clone()],
        Command::Create {
            name,
            access,
            scopes,
        } => {
            let mut args = vec![
                "create".to_owned(),
                format!("--name={name}"),
                "--access".to_owned(),
                access.as_str().to_owned(),
            ];
            for scope in scopes {
                args.push("--scope".into());
                args.push(scope.clone());
            }
            args
        }
    }
}

/// "2026-11-05 14:05 UTC".
fn when(ms: i64) -> String {
    chrono::DateTime::from_timestamp_millis(ms).map_or_else(
        || "unknown".to_owned(),
        |t| t.format("%Y-%m-%d %H:%M UTC").to_string(),
    )
}

/// One line of `list`: id, name, access, notebooks, created, last used.
fn list_line(row: &ApiKeyRow, books: &dyn Fn(&str) -> Option<String>) -> String {
    let scope = match &row.books {
        None => "all notebooks".to_owned(),
        Some(ids) => {
            let names: Vec<String> = ids
                .iter()
                .map(|id| books(id).unwrap_or_else(|| format!("(deleted notebook {id})")))
                .collect();
            names.join(", ")
        }
    };
    let used = row.last_used_at.map_or_else(
        || "never used".to_owned(),
        |at| format!("used {}", when(at)),
    );
    // The name last: it is free text, set wherever the key was made.
    format!(
        "{}\t{}\t{}\tcreated {}\t{}\t{}",
        row.id,
        row.access.as_str(),
        plain(&scope),
        when(row.created_at),
        used,
        plain(&row.name)
    )
}

/// `text` with every control character (tab, newline, the escape that
/// starts a terminal sequence) as a space, so a name stays on its line and
/// can't drive the terminal. Pure.
fn plain(text: &str) -> String {
    text.chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect()
}

/// Which live key `target` names: by id, else by name ignoring case.
/// The error says what to do. Pure.
pub fn pick_key<'a>(rows: &'a [ApiKeyRow], target: &str) -> Result<&'a ApiKeyRow, String> {
    if let Some(row) = rows.iter().find(|row| row.id == target) {
        return Ok(row);
    }
    let named: Vec<&ApiKeyRow> = rows
        .iter()
        .filter(|row| row.name.trim().to_lowercase() == target.to_lowercase())
        .collect();
    match named.as_slice() {
        [row] => Ok(row),
        [] => Err(format!(
            "there's no working key with the id or name “{target}”. Run `kanso api-key list` to see them."
        )),
        several => Err(format!(
            "{} keys are named “{target}”. Revoke one by its id: {}",
            several.len(),
            several
                .iter()
                .map(|row| row.id.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }
}

/// Runs `command` against `store`. The key, when one is made, goes to `out`
/// and nothing else does; messages go to `err`.
pub fn execute(
    store: &Arc<Store>,
    command: Command,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> Result<()> {
    let keys = KeyStore::open(store.clone()).context("reading the API keys")?;
    let books: Vec<(String, String)> = store
        .books()
        .context("reading the notebooks")?
        .into_iter()
        .map(|b| (b.id, b.name))
        .collect();
    match command {
        Command::Help => writeln!(err, "{USAGE}")?,
        Command::Create {
            name,
            access,
            scopes,
        } => {
            let mut ids: Vec<String> = Vec::new();
            for wanted in &scopes {
                let (id, _) = pick_book_for(&books, wanted, "--scope").map_err(|reason| {
                    anyhow::anyhow!("{reason}. Without --scope the key reaches every notebook.")
                })?;
                if !ids.contains(id) {
                    ids.push(id.clone());
                }
            }
            let minted = keys
                .mint(NewKey {
                    name: name.clone(),
                    client: crate::api_keys_model::OTHER_ID.to_owned(),
                    access,
                    books: (!ids.is_empty()).then_some(ids),
                })
                .context("saving the key")?;
            writeln!(out, "{}", minted.key)?;
            writeln!(
                err,
                "Made the key “{name}” ({}). It is above, and only above: Kanso keeps just a hash, so it can't be shown again. Put it in the program's KANSO_MCP_TOKEN, or send it as “Authorization: Bearer <key>”.",
                minted.id.as_str()
            )?;
        }
        Command::List => {
            let rows: Vec<ApiKeyRow> = keys
                .list()
                .context("reading the API keys")?
                .into_iter()
                .filter(|row| row.revoked_at.is_none())
                .collect();
            if rows.is_empty() {
                writeln!(
                    err,
                    "No API keys. Make one with `kanso api-key create --name <name>`."
                )?;
            }
            let name_of = |id: &str| {
                books
                    .iter()
                    .find(|(book, _)| book == id)
                    .map(|(_, name)| name.clone())
            };
            for row in &rows {
                writeln!(out, "{}", list_line(row, &name_of))?;
            }
        }
        Command::Revoke { target } => {
            let rows: Vec<ApiKeyRow> = keys
                .list()
                .context("reading the API keys")?
                .into_iter()
                .filter(|row| row.revoked_at.is_none())
                .collect();
            let row = pick_key(&rows, &target).map_err(anyhow::Error::msg)?;
            if !keys
                .revoke(&KeyId(row.id.clone()))
                .context("revoking the key")?
            {
                bail!("the key “{}” was already revoked", row.name);
            }
            writeln!(
                err,
                "Revoked the key “{}”. It no longer works, and nothing brings it back.",
                row.name
            )?;
        }
    }
    Ok(())
}

/// What a data folder in use is told.
const IN_USE: &str = "Kanso, or `kanso serve`, is running with this data folder, and it keeps its keys in memory, so a key changed from here would not take effect. Quit it (stop `kanso serve`), then run this again. With the app open, use Settings › Local API › API keys instead.";

/// `kanso api-key …`.
pub fn run_cli(data_dir: &Path, rest: &[String]) -> Result<()> {
    let command = parse(rest).map_err(|reason| anyhow::anyhow!("api-key: {reason}\n\n{USAGE}"))?;
    if command == Command::Help {
        println!("{USAGE}");
        return Ok(());
    }
    let Some(_lock) = lock_data_dir(data_dir)? else {
        bail!("{IN_USE}");
    };
    let store = Arc::new(crate::library_open::open_store(data_dir)?);
    execute(
        &store,
        command,
        &mut std::io::stdout().lock(),
        &mut std::io::stderr().lock(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use kanso_core::DocType;
    use kanso_store::{Origin, Put};
    use proptest::prelude::*;
    use proptest::test_runner::RngSeed;

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 256,
            rng_seed: RngSeed::Fixed(0x006b_616e_736f),
            ..ProptestConfig::default()
        }
    }

    fn fresh() -> (tempfile::TempDir, Arc<Store>) {
        let dir = tempfile::tempdir().unwrap();
        let store = Arc::new(Store::open(dir.path().join("kanso.db")).unwrap());
        (dir, store)
    }

    fn args(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| (*s).to_owned()).collect()
    }

    fn book(store: &Store, name: &str) -> String {
        store
            .put(
                DocType::Book,
                serde_json::json!({"name": name}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id
    }

    fn run(store: &Arc<Store>, command: Command) -> (Result<()>, String, String) {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let result = execute(store, command, &mut out, &mut err);
        (
            result,
            String::from_utf8(out).unwrap(),
            String::from_utf8(err).unwrap(),
        )
    }

    fn make(store: &Arc<Store>, name: &str, access: KeyAccess, scopes: &[&str]) -> String {
        let (result, out, _) = run(
            store,
            Command::Create {
                name: name.into(),
                access,
                scopes: scopes.iter().map(|s| (*s).to_owned()).collect(),
            },
        );
        result.unwrap();
        out.trim_end().to_owned()
    }

    #[test]
    fn parse_examples() {
        assert_eq!(
            parse(&args(&[
                "create",
                "--name",
                "Backup",
                "--access",
                "write",
                "--scope",
                "Work",
                "--scope=Ideas"
            ]))
            .unwrap(),
            Command::Create {
                name: "Backup".into(),
                access: KeyAccess::Write,
                scopes: vec!["Work".into(), "Ideas".into()],
            }
        );
        // Read is the default.
        assert_eq!(
            parse(&args(&["create", "--name=x"])).unwrap(),
            Command::Create {
                name: "x".into(),
                access: KeyAccess::Read,
                scopes: vec![]
            }
        );
        assert!(parse(&args(&["create"])).unwrap_err().contains("--name"));
        assert!(
            parse(&args(&["create", "--name", "x", "--access", "admin"]))
                .unwrap_err()
                .contains("read, suggest or write")
        );
        assert!(
            parse(&args(&["create", "--name"]))
                .unwrap_err()
                .contains("needs a value")
        );
        assert!(parse(&args(&["revoke"])).unwrap_err().contains("list"));
        assert!(
            parse(&args(&["frobnicate"]))
                .unwrap_err()
                .contains("create, list or revoke")
        );
        assert!(parse(&args(&[])).unwrap_err().contains("api-key help"));
    }

    #[test]
    fn create_prints_only_the_key_on_stdout() {
        let (_dir, store) = fresh();
        let (result, out, err) = run(
            &store,
            Command::Create {
                name: "Backup script".into(),
                access: KeyAccess::Read,
                scopes: vec![],
            },
        );
        result.unwrap();
        let key = out.strip_suffix('\n').unwrap();
        assert_eq!(key.len(), 47);
        assert!(key.starts_with("kcl_"));
        // The messages never repeat it.
        assert!(!err.contains(key), "{err}");
    }

    #[test]
    fn a_made_key_verifies_and_a_revoked_one_does_not() {
        let (_dir, store) = fresh();
        let key = make(&store, "Script", KeyAccess::Suggest, &[]);
        assert!(
            KeyStore::open(store.clone())
                .unwrap()
                .verify(&key)
                .is_some()
        );
        let (result, out, err) = run(
            &store,
            Command::Revoke {
                target: "script".into(),
            },
        );
        result.unwrap();
        assert_eq!(out, "");
        assert!(!err.contains(&key));
        assert!(
            KeyStore::open(store.clone())
                .unwrap()
                .verify(&key)
                .is_none()
        );
        // Gone from the list, and revoking again says what to do.
        let (_, listed, _) = run(&store, Command::List);
        assert_eq!(listed, "");
        let (result, _, _) = run(
            &store,
            Command::Revoke {
                target: "Script".into(),
            },
        );
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("kanso api-key list")
        );
    }

    #[test]
    fn scopes_resolve_to_notebooks_and_a_stranger_is_refused() {
        let (_dir, store) = fresh();
        let work = book(&store, "Work");
        make(&store, "Scoped", KeyAccess::Read, &["work", &work]);
        let rows = store.api_keys_all().unwrap();
        assert_eq!(rows[0].books, Some(vec![work]));
        let (result, out, _) = run(
            &store,
            Command::Create {
                name: "Bad".into(),
                access: KeyAccess::Read,
                scopes: vec!["Nowhere".into()],
            },
        );
        let message = result.unwrap_err().to_string();
        assert!(
            message.contains("Nowhere")
                && message.contains("leave --scope off")
                && !message.contains("--book"),
            "{message}"
        );
        assert_eq!(out, "");
        assert_eq!(store.api_keys_all().unwrap().len(), 1);
    }

    #[test]
    fn list_shows_neither_key_nor_hash_and_keeps_names_on_their_line() {
        let (_dir, store) = fresh();
        let key = make(
            &store,
            "Evil\u{1b}]0;x\u{7}\rname\nnext",
            KeyAccess::Read,
            &[],
        );
        let (result, out, _) = run(&store, Command::List);
        result.unwrap();
        assert_eq!(out.lines().count(), 1, "{out:?}");
        assert!(
            !out.chars()
                .any(|c| c.is_control() && c != '\t' && c != '\n')
        );
        assert!(!out.contains(&key));
        let rows = store.api_keys_live().unwrap();
        let hash: String = rows[0]
            .token_hash
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert!(!out.contains(&hash), "{out}");
    }

    #[test]
    fn revoke_by_a_shared_name_asks_for_the_id() {
        let (_dir, store) = fresh();
        make(&store, "Twin", KeyAccess::Read, &[]);
        make(&store, "Twin", KeyAccess::Read, &[]);
        let (result, _, _) = run(
            &store,
            Command::Revoke {
                target: "Twin".into(),
            },
        );
        let message = result.unwrap_err().to_string();
        assert!(
            message.contains("2 keys are named") && message.contains("id"),
            "{message}"
        );
    }

    #[test]
    fn it_refuses_while_the_data_folder_is_held() {
        let dir = tempfile::tempdir().unwrap();
        let _held = lock_data_dir(dir.path()).unwrap().unwrap();
        let error = run_cli(dir.path(), &args(&["list"]))
            .unwrap_err()
            .to_string();
        assert!(
            error.contains("Quit it") && error.contains("Settings"),
            "{error}"
        );
    }

    proptest! {
        #![proptest_config(config())]

        /// Any command reads back from its own arguments.
        #[test]
        fn prop_parse_round_trips(
            name in "[A-Za-z0-9][A-Za-z0-9 _.-]{0,15}[A-Za-z0-9]",
            access in prop::sample::select(KeyAccess::ALL.to_vec()),
            scopes in prop::collection::vec("[A-Za-z0-9][A-Za-z0-9 ]{0,8}[A-Za-z0-9]", 0..4),
        ) {
            let mut unique: Vec<String> = Vec::new();
            for scope in scopes {
                if !unique.contains(&scope) {
                    unique.push(scope);
                }
            }
            let command = Command::Create { name, access, scopes: unique };
            prop_assert_eq!(parse(&unparse(&command)), Ok(command));
        }

        /// Whatever is typed, parsing never panics, and a refusal always
        /// names something to do (a command, an option or the help).
        #[test]
        fn prop_parse_never_panics(words in prop::collection::vec("[ -~]{0,12}", 0..6)) {
            if let Err(reason) = parse(&words) {
                prop_assert!(!reason.is_empty());
            }
        }

        /// A made key works exactly until it is revoked, and the key is on
        /// stdout alone.
        #[test]
        fn prop_made_keys_work_until_revoked(
            names in prop::collection::vec("[A-Za-z][A-Za-z0-9]{0,8}", 1..4),
            access in prop::sample::select(KeyAccess::ALL.to_vec()),
        ) {
            let (_dir, store) = fresh();
            let unique: std::collections::BTreeSet<String> =
                names.iter().map(|n| n.to_lowercase()).collect();
            let mut made = Vec::new();
            for name in &unique {
                let (result, out, err) = run(&store, Command::Create {
                    name: name.clone(), access, scopes: vec![],
                });
                prop_assert!(result.is_ok());
                let key = out.trim_end().to_owned();
                prop_assert!(!err.contains(&key));
                made.push((name.clone(), key));
            }
            let keys = KeyStore::open(store.clone()).unwrap();
            for (_, key) in &made {
                prop_assert!(keys.verify(key).is_some());
            }
            let (revoked_name, revoked_key) = made.remove(0);
            let (result, _, _) = run(&store, Command::Revoke { target: revoked_name });
            prop_assert!(result.is_ok());
            let keys = KeyStore::open(store.clone()).unwrap();
            prop_assert!(keys.verify(&revoked_key).is_none());
            for (_, key) in &made {
                prop_assert!(keys.verify(key).is_some());
            }
        }
    }
}
