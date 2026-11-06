//! A display-free command binary for servers. Keep this entry point separate
//! from GPUI so Linux releases can ship a small process with no window-system
//! or graphics runtime dependencies.

// Shared app modules contain UI helpers unused by this entry point. This
// binary intentionally compiles only the CLI paths that its dispatcher uses.
#![allow(dead_code, unused_imports)]

#[path = "account_cli.rs"]
mod account_cli;
#[path = "api_key_cli.rs"]
mod api_key_cli;
#[path = "api_keys_model.rs"]
mod api_keys_model;
#[path = "grouping.rs"]
mod grouping;
#[path = "mcp_setup.rs"]
mod mcp_setup;
mod capture_cli {
    pub fn pick_book_for<'a>(
        books: &'a [(String, String)],
        wanted: &str,
        option: &str,
    ) -> Result<&'a (String, String), String> {
        let wanted = wanted.trim();
        if let Some(book) = books.iter().find(|(id, _)| id == wanted) {
            return Ok(book);
        }
        let named: Vec<_> = books
            .iter()
            .filter(|(_, name)| name.trim().eq_ignore_ascii_case(wanted))
            .collect();
        match named.as_slice() {
            [book] => Ok(book),
            [] => Err(format!(
                "there's no notebook named “{wanted}”. Check its spelling, or leave {option} off"
            )),
            several => Err(format!(
                "{} notebooks are named “{wanted}”; use the ID of one: {}",
                several.len(),
                several
                    .iter()
                    .map(|(id, _)| id.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
        }
    }
}
#[path = "config.rs"]
mod config;
#[path = "control.rs"]
mod control;
#[path = "control_model.rs"]
mod control_model;
#[path = "flavour.rs"]
mod flavour;
mod library_open {
    use std::path::Path;

    use anyhow::Result;
    use kanso_store::{ReadOnlyStore, Store};
    use kanso_sync::keychain::SecretStore;

    use crate::{config::AppConfig, paths};

    pub fn open_store(data_dir: &Path) -> Result<Store> {
        let config = AppConfig::load(&paths::config_path(data_dir)).unwrap_or_default();
        let secrets = crate::secrets::open_secrets(&config, data_dir)?;
        open_store_with(data_dir, secrets.as_ref())
    }

    pub fn open_store_with(data_dir: &Path, secrets: &dyn SecretStore) -> Result<Store> {
        let db = paths::db_path(data_dir);
        if kanso_store::restore_staged(&db) {
            eprintln!("A restore is waiting; open Kanso to apply it.");
        }
        let opened = kanso_sync::open_library(&db, secrets)?;
        Ok(opened.store)
    }

    pub fn open_read_only(data_dir: &Path, secrets: &dyn SecretStore) -> Result<ReadOnlyStore> {
        let db = paths::db_path(data_dir);
        let key = match kanso_store::LibraryFile::of(&db) {
            kanso_store::LibraryFile::Encrypted => {
                let id = std::fs::read_to_string(kanso_sync::library_key::key_id_path(&db))?;
                kanso_sync::library_key::load_key(secrets, id.trim())?
                    .ok_or(kanso_store::StoreError::WrongKey)?
                    .into()
            }
            _ => None,
        };
        Ok(Store::open_read_only_with_key(db, key.as_ref())?)
    }
}
#[path = "mcp_cli.rs"]
mod mcp_cli;
#[path = "paths.rs"]
mod paths;
#[path = "secrets.rs"]
mod secrets;
#[path = "serve_cli.rs"]
mod serve_cli;
#[path = "service_cli.rs"]
mod service_cli;
mod services {
    use std::net::{IpAddr, SocketAddr};

    use kanso_server::ServerConfig;

    use crate::config::ServerSettings;

    pub fn server_config(settings: &ServerSettings) -> Result<ServerConfig, String> {
        let ip: IpAddr = settings
            .bind_address
            .trim()
            .parse()
            .map_err(|_| format!("“{}” isn't an IP address", settings.bind_address))?;
        Ok(ServerConfig {
            bind: SocketAddr::new(ip, settings.port),
        })
    }
}
mod layout {
    pub const INSPECTOR_WIDTH: f32 = 300.;
    pub const INSPECTOR_RANGE: (f32, f32) = (260., 480.);
}
mod launch {
    pub fn label_for(flavour: &crate::flavour::Flavour) -> &'static str {
        flavour.bundle_id
    }
}
mod zoom {
    pub const DEFAULT_ZOOM: u32 = 100;
    pub const ZOOM_RANGE: (u32, u32) = (80, 150);
    pub const ZOOM_STEP: u32 = 10;
}
mod settings {
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

    pub fn mcp_endpoint(bind_address: &str, port: u16) -> String {
        let ip = bind_address
            .trim()
            .parse::<IpAddr>()
            .unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST));
        let ip = match ip {
            IpAddr::V4(v4) if v4.is_unspecified() => IpAddr::V4(Ipv4Addr::LOCALHOST),
            IpAddr::V6(v6) if v6.is_unspecified() => IpAddr::V6(Ipv6Addr::LOCALHOST),
            ip => ip,
        };
        format!("http://{}/mcp", SocketAddr::new(ip, port))
    }
}

fn load_config(data_dir: &std::path::Path) -> anyhow::Result<config::AppConfig> {
    let path = paths::config_path(data_dir);
    config::AppConfig::load_migrated(&path)
        .map_err(|error| anyhow::anyhow!("reading {}: {error}", path.display()))
}

const USAGE: &str = "Usage: kanso <serve|account|service|api-key|mcp> [--data-dir <dir>] [options]\n\nRun Kanso's API and account tools without a desktop runtime.\n";

struct Args {
    command: Option<String>,
    data_dir: std::path::PathBuf,
    rest: Vec<String>,
}

fn parse_args(args: impl IntoIterator<Item = String>) -> anyhow::Result<Args> {
    let mut args = args.into_iter().peekable();
    let mut data_dir = paths::default_data_dir();
    let mut command = None;
    let mut rest = Vec::new();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--data-dir" => {
                data_dir = args
                    .next()
                    .map(std::path::PathBuf::from)
                    .ok_or_else(|| anyhow::anyhow!("--data-dir needs a folder"))?;
            }
            "--help" | "-h" if command.is_none() => command = Some("help".to_owned()),
            _ if command.is_none() => command = Some(arg),
            _ => rest.push(arg),
        }
    }
    Ok(Args {
        command,
        data_dir,
        rest,
    })
}

fn main() -> anyhow::Result<()> {
    let args = parse_args(std::env::args().skip(1))?;
    let data_dir = args.data_dir;
    let rest = args.rest;
    match args.command.as_deref() {
        Some("serve") => serve_cli::run_cli(data_dir, &rest),
        Some("service") => service_cli::run_cli(&rest),
        Some("account") => account_cli::run_cli(&data_dir, &rest),
        Some("api-key") => api_key_cli::run_cli(&data_dir, &rest),
        Some("mcp") => mcp_cli::run_cli(&data_dir, &rest),
        Some("help") | None => {
            println!("{USAGE}");
            Ok(())
        }
        Some(other) => anyhow::bail!("unknown command `{other}`\n\n{USAGE}"),
    }
}

#[cfg(test)]
mod tests {
    use super::parse_args;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_global_data_dir_is_removed_without_changing_command_args(
            path in "/tmp/[a-zA-Z0-9_-]{1,48}"
        ) {
            let args = parse_args([
                "serve".to_owned(),
                "--verbose".to_owned(),
                "--data-dir".to_owned(),
                path.clone(),
                "--related-notes".to_owned(),
                "on".to_owned(),
            ]).unwrap();
            prop_assert_eq!(args.command.as_deref(), Some("serve"));
            prop_assert_eq!(args.data_dir, std::path::PathBuf::from(path));
            prop_assert_eq!(args.rest, ["--verbose", "--related-notes", "on"]);
        }
    }
}
