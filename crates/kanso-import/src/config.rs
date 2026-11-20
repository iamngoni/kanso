//! The source app's local HTTP server settings, from its `config.json`.
//!
//! The credentials are the user's real password. Nothing here prints, logs
//! or stores them; `Debug` redacts them.

use std::fmt;
use std::path::Path;

use anyhow::{Context as _, bail};
use base64::Engine as _;
use serde_json::Value;

/// The local-server API's default port.
pub const DEFAULT_PORT: u16 = 19840;
/// The local-server API's default bind address.
pub const DEFAULT_BIND_ADDRESS: &str = "127.0.0.1";

/// `core.server` from the source app's `config.json`.
#[derive(Clone, PartialEq, Eq)]
pub struct LocalServerConfig {
    pub enabled: bool,
    pub port: u16,
    pub bind_address: String,
    pub credentials: LocalServerCredentials,
}

/// The `auth` value of the local server settings.
#[derive(Clone, PartialEq, Eq)]
pub enum LocalServerCredentials {
    /// No `auth` value.
    None,
    /// `"username:password"` (split at the first colon) or `{username, password}`.
    Basic { username: String, password: String },
    /// Electron `safeStorage` ciphertext: base64 of `v10`/`v11` followed by
    /// AES-CBC data. Only the matching macOS Keychain item can decrypt it, so
    /// the username and password are not available here.
    Encrypted { ciphertext: String },
    /// A value in a shape this importer doesn't recognize.
    Unrecognized,
}

impl LocalServerConfig {
    pub fn username(&self) -> Option<&str> {
        match &self.credentials {
            LocalServerCredentials::Basic { username, .. } => Some(username),
            _ => None,
        }
    }

    pub fn password(&self) -> Option<&str> {
        match &self.credentials {
            LocalServerCredentials::Basic { password, .. } => Some(password),
            _ => None,
        }
    }
}

impl fmt::Debug for LocalServerConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LocalServerConfig")
            .field("enabled", &self.enabled)
            .field("port", &self.port)
            .field("bind_address", &self.bind_address)
            .field("credentials", &self.credentials)
            .finish()
    }
}

impl fmt::Debug for LocalServerCredentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LocalServerCredentials::None => f.write_str("None"),
            LocalServerCredentials::Basic { username, .. } => f
                .debug_struct("Basic")
                .field("username", username)
                .field("password", &"<redacted>")
                .finish(),
            LocalServerCredentials::Encrypted { .. } => f
                .debug_struct("Encrypted")
                .field("ciphertext", &"<redacted>")
                .finish(),
            LocalServerCredentials::Unrecognized => f.write_str("Unrecognized"),
        }
    }
}

pub(crate) fn parse_credentials(auth: Option<&Value>) -> LocalServerCredentials {
    match auth {
        None | Some(Value::Null) => LocalServerCredentials::None,
        Some(Value::String(s)) if s.is_empty() => LocalServerCredentials::None,
        Some(Value::String(s)) => match s.split_once(':') {
            Some((username, password)) => LocalServerCredentials::Basic {
                username: username.to_owned(),
                password: password.to_owned(),
            },
            None => {
                let decoded = base64::engine::general_purpose::STANDARD.decode(s.trim());
                match decoded {
                    Ok(bytes) if bytes.starts_with(b"v10") || bytes.starts_with(b"v11") => {
                        LocalServerCredentials::Encrypted {
                            ciphertext: s.clone(),
                        }
                    }
                    _ => LocalServerCredentials::Unrecognized,
                }
            }
        },
        Some(Value::Object(map)) => match (
            map.get("username").and_then(Value::as_str),
            map.get("password").and_then(Value::as_str),
        ) {
            (Some(username), Some(password)) => LocalServerCredentials::Basic {
                username: username.to_owned(),
                password: password.to_owned(),
            },
            _ => LocalServerCredentials::Unrecognized,
        },
        Some(_) => LocalServerCredentials::Unrecognized,
    }
}

/// Parses the server settings from the text of the source app's `config.json`.
/// `Ok(None)` when it has no `core.server` section.
pub fn parse_local_server_config(json: &str) -> anyhow::Result<Option<LocalServerConfig>> {
    let config: Value = serde_json::from_str(json).context("config.json is not valid JSON")?;
    let Some(server) = config.pointer("/*/core/server") else {
        return Ok(None);
    };
    let Some(server) = server.as_object() else {
        bail!("core.server in config.json is not an object");
    };
    let enabled = match server.get("enabled") {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(_) => bail!("core.server.enabled in config.json is not a boolean"),
    };
    let port = match server.get("port") {
        None | Some(Value::Null) => DEFAULT_PORT,
        Some(v) => v
            .as_u64()
            .and_then(|p| u16::try_from(p).ok())
            .filter(|p| *p != 0)
            .context("core.server.port in config.json is not a valid port")?,
    };
    let bind_address = match server.get("bindAddress") {
        None | Some(Value::Null) => DEFAULT_BIND_ADDRESS.to_owned(),
        Some(Value::String(s)) if !s.trim().is_empty() => s.trim().to_owned(),
        Some(_) => bail!("core.server.bindAddress in config.json is not a valid address"),
    };
    Ok(Some(LocalServerConfig {
        enabled,
        port,
        bind_address,
        credentials: parse_credentials(server.get("auth")),
    }))
}

/// Reads the local server settings from `<library_dir>/config.json`.
/// `Ok(None)` when the file doesn't exist or has no server section.
pub fn read_local_server_config(library_dir: &Path) -> anyhow::Result<Option<LocalServerConfig>> {
    let path = library_dir.join("config.json");
    let json = match std::fs::read_to_string(&path) {
        Ok(json) => json,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(err) => return Err(err).with_context(|| format!("cannot read {}", path.display())),
    };
    parse_local_server_config(&json).with_context(|| format!("in {}", path.display()))
}
