//! `kanso-release`: makes, signs and checks the update feed for a release
//! (docs/RELEASING.md).
//!
//! ```text
//! kanso-release keygen --secret-out <file>
//! kanso-release feed --version <v> --notes <file> --dist <dir> --pub-date <rfc3339> --out <latest.json>
//!                    [--whats-new <file.webp>] [--changelog <CHANGELOG.md>]
//! kanso-release sign --feed <latest.json> [--key-env KANSO_UPDATE_SIGNING_KEY]
//! kanso-release verify --feed <latest.json> [--sig <file>] [--key <base64>]...
//! kanso-release notes --changelog <CHANGELOG.md> --version <v> --out <file>
//! kanso-release check-notices [--changelog <CHANGELOG.md> --version <v>]
//! ```

use std::collections::BTreeMap;
use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use kanso_update::feed::{generate_signing_key, parse_signing_key, public_key_text};
use kanso_update::whats_new::{MAX_EARLIER, MAX_MEDIA_BYTES, looks_like_webp, media_name};
use kanso_update::{
    Asset, Earlier, Feed, Platform, SCHEMA, sign_feed, trusted_keys, verify_signature,
};
use sha2::{Digest, Sha256};

const USAGE: &str = "\
Usage:
  kanso-release keygen --secret-out <file>
      Makes a signing key. Writes its secret (base64) to <file>, readable
      only by you, and prints the public key for keys/trusted-keys.txt.
  kanso-release feed --version <v> --notes <file> --dist <dir>
                     --pub-date <rfc3339> --out <latest.json>
                     [--base-url <url>] [--release-url <url>]
                     [--whats-new <file.webp>] [--changelog <CHANGELOG.md>]
      Writes latest.json for the update archives in <dir>. --whats-new adds
      the release's What's new video (an animated WebP up to 8 MB, uploaded
      as Kanso-<v>-whats-new.webp); --changelog adds the notes of the
      releases before <v>, for people who skipped some.
  kanso-release sign --feed <latest.json> [--key-env <NAME>] [--allow-untrusted]
      Signs it with the base64 secret in $NAME (KANSO_UPDATE_SIGNING_KEY),
      writing <latest.json>.sig. Refuses a key the app doesn't trust.
  kanso-release verify --feed <latest.json> [--sig <file>] [--key <base64>]...
      Checks the signature against the given keys, or the trusted ones.
  kanso-release notes --changelog <CHANGELOG.md> --version <v> --out <file>
      Writes the changelog's section for <v>: the release's notes, for its
      page and for latest.json alike.
  kanso-release check-notices [--changelog <CHANGELOG.md> --version <v>]
      Checks the update notices the app is built with and, with --changelog,
      that every entry under Breaking in <v>'s section ends with
      (notice: <id>) for one of them. Fails the release when one doesn't.
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("keygen") => keygen(&Flags::parse(&args[1..])),
        Some("feed") => feed(&Flags::parse(&args[1..])),
        Some("sign") => sign(&Flags::parse(&args[1..])),
        Some("verify") => verify(&Flags::parse(&args[1..])),
        Some("notes") => notes(&Flags::parse(&args[1..])),
        Some("check-notices") => check_notices(&Flags::parse(&args[1..])),
        Some("help" | "-h" | "--help") => {
            print!("{USAGE}");
            Ok(())
        }
        _ => Err(format!("no command given\n\n{USAGE}")),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("kanso-release: {message}");
            ExitCode::FAILURE
        }
    }
}

/// `--name value` pairs and lone `--switch`es.
struct Flags {
    values: BTreeMap<String, Vec<String>>,
}

impl Flags {
    fn parse(args: &[String]) -> Flags {
        let mut values: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut i = 0;
        while i < args.len() {
            let name = args[i].trim_start_matches("--").to_owned();
            match args.get(i + 1).filter(|v| !v.starts_with("--")) {
                Some(value) => {
                    values.entry(name).or_default().push(value.clone());
                    i += 2;
                }
                None => {
                    values.entry(name).or_default();
                    i += 1;
                }
            }
        }
        Flags { values }
    }

    fn get(&self, name: &str) -> Option<&str> {
        self.values
            .get(name)
            .and_then(|v| v.last())
            .map(String::as_str)
    }

    fn need(&self, name: &str) -> Result<&str, String> {
        self.get(name)
            .ok_or_else(|| format!("--{name} is needed\n\n{USAGE}"))
    }

    fn all(&self, name: &str) -> &[String] {
        self.values.get(name).map_or(&[], Vec::as_slice)
    }

    fn has(&self, name: &str) -> bool {
        self.values.contains_key(name)
    }
}

fn keygen(flags: &Flags) -> Result<(), String> {
    let out = PathBuf::from(flags.need("secret-out")?);
    let (key, secret) = generate_signing_key();
    write_secret(&out, &secret).map_err(|e| format!("writing {}: {e}", out.display()))?;
    println!("Secret key written to {} (keep it private).", out.display());
    println!("Public key, for crates/kanso-update/keys/trusted-keys.txt:");
    println!("{}", public_key_text(&key.verifying_key()));
    Ok(())
}

/// Writes a new file only its owner can read; never overwrites one.
fn write_secret(path: &Path, secret: &str) -> std::io::Result<()> {
    use std::io::Write as _;
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    let mut file = options.open(path)?;
    file.write_all(secret.as_bytes())?;
    file.write_all(b"\n")
}

fn feed(flags: &Flags) -> Result<(), String> {
    let version = flags.need("version")?.trim_start_matches('v').to_owned();
    semver::Version::parse(&version).map_err(|e| format!("--version {version}: {e}"))?;
    let notes_path = flags.need("notes")?;
    let notes =
        std::fs::read_to_string(notes_path).map_err(|e| format!("reading {notes_path}: {e}"))?;
    let dist = PathBuf::from(flags.need("dist")?);
    let pub_date = flags.need("pub-date")?.to_owned();
    let out = PathBuf::from(flags.need("out")?);
    let repo = kanso_update::REPOSITORY;
    let base = flags
        .get("base-url")
        .map(str::to_owned)
        .unwrap_or_else(|| format!("https://github.com/{repo}/releases/download/v{version}"));
    let release_url = flags
        .get("release-url")
        .map(str::to_owned)
        .unwrap_or_else(|| format!("https://github.com/{repo}/releases/tag/v{version}"));
    let mut platforms = BTreeMap::new();
    for platform in Platform::ALL {
        let name = platform.asset_name(&version);
        let path = dist.join(&name);
        let (sha256, size) = hash_file(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        platforms.insert(
            platform.key().to_owned(),
            Asset {
                url: format!("{}/{name}", base.trim_end_matches('/')),
                sha256,
                size,
            },
        );
    }
    let whats_new = match flags.get("whats-new") {
        None => None,
        Some(path) => {
            let path = Path::new(path);
            let mut head = [0u8; 12];
            std::fs::File::open(path)
                .and_then(|mut f| f.read_exact(&mut head))
                .map_err(|e| format!("{}: {e}", path.display()))?;
            if !looks_like_webp(&head) {
                return Err(format!("{} isn't a WebP file", path.display()));
            }
            let (sha256, size) = hash_file(path).map_err(|e| format!("{}: {e}", path.display()))?;
            if size > MAX_MEDIA_BYTES {
                return Err(format!(
                    "{} is {size} bytes; the app takes up to {MAX_MEDIA_BYTES}",
                    path.display()
                ));
            }
            Some(Asset {
                url: format!("{}/{}", base.trim_end_matches('/'), media_name(&version)),
                sha256,
                size,
            })
        }
    };
    let earlier = match flags.get("changelog") {
        None => Vec::new(),
        Some(path) => {
            let changelog =
                std::fs::read_to_string(path).map_err(|e| format!("reading {path}: {e}"))?;
            kanso_update::notes::earlier_sections(&changelog, &version, MAX_EARLIER)
                .into_iter()
                .map(|(version, pub_date, notes)| Earlier {
                    version,
                    pub_date,
                    notes: kanso_update::notes::for_app(&notes),
                })
                .collect()
        }
    };
    let feed = Feed {
        schema: SCHEMA,
        version,
        pub_date,
        release_url,
        notes: notes.trim().to_owned(),
        platforms,
        whats_new,
        earlier: kanso_update::whats_new::checked_earlier(earlier),
    };
    let mut bytes = serde_json::to_vec_pretty(&feed).map_err(|e| e.to_string())?;
    bytes.push(b'\n');
    std::fs::write(&out, bytes).map_err(|e| format!("writing {}: {e}", out.display()))?;
    println!("Wrote {}", out.display());
    Ok(())
}

fn hash_file(path: &Path) -> std::io::Result<(String, u64)> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 1 << 16];
    let mut size = 0u64;
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        size += n as u64;
        hasher.update(&buf[..n]);
    }
    let hex = hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    Ok((hex, size))
}

fn sign(flags: &Flags) -> Result<(), String> {
    let feed_path = PathBuf::from(flags.need("feed")?);
    let var = flags.get("key-env").unwrap_or("KANSO_UPDATE_SIGNING_KEY");
    let secret = zeroize::Zeroizing::new(std::env::var(var).unwrap_or_default());
    if secret.trim().is_empty() {
        return Err(format!(
            "{var} is empty or not set; a release can't ship without a signed feed (docs/RELEASING.md › One-time setup)"
        ));
    }
    let key = parse_signing_key(&secret)
        .map_err(|_| format!("{var} isn't a base64 Ed25519 secret key"))?;
    let public = key.verifying_key();
    if !flags.has("allow-untrusted") && !trusted_keys().contains(&public) {
        return Err(format!(
            "the key in {var} (public {}) isn't in crates/kanso-update/keys/trusted-keys.txt, so this build's app would refuse the update",
            public_key_text(&public)
        ));
    }
    let bytes =
        std::fs::read(&feed_path).map_err(|e| format!("reading {}: {e}", feed_path.display()))?;
    serde_json::from_slice::<Feed>(&bytes)
        .map_err(|e| format!("{} isn't a feed: {e}", feed_path.display()))?;
    let sig_path = sig_path(&feed_path);
    std::fs::write(&sig_path, sign_feed(&bytes, &key)).map_err(|e| e.to_string())?;
    println!("Wrote {}", sig_path.display());
    Ok(())
}

fn sig_path(feed: &Path) -> PathBuf {
    let mut name = feed.file_name().unwrap_or_default().to_os_string();
    name.push(".sig");
    feed.with_file_name(name)
}

fn verify(flags: &Flags) -> Result<(), String> {
    let feed_path = PathBuf::from(flags.need("feed")?);
    let sig = flags
        .get("sig")
        .map_or_else(|| sig_path(&feed_path), PathBuf::from);
    let keys = if flags.all("key").is_empty() {
        trusted_keys()
    } else {
        flags
            .all("key")
            .iter()
            .map(|k| {
                kanso_update::parse_public_key(k)
                    .map_err(|_| format!("--key {k} isn't a public key"))
            })
            .collect::<Result<Vec<_>, _>>()?
    };
    let bytes =
        std::fs::read(&feed_path).map_err(|e| format!("reading {}: {e}", feed_path.display()))?;
    let signature = std::fs::read(&sig).map_err(|e| format!("reading {}: {e}", sig.display()))?;
    verify_signature(&bytes, &signature, &keys).map_err(|e| e.to_string())?;
    let feed: Feed = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    println!(
        "Signature OK: Kanso {} for {}",
        feed.version,
        feed.platforms
            .keys()
            .cloned()
            .collect::<Vec<_>>()
            .join(", ")
    );
    Ok(())
}

fn notes(flags: &Flags) -> Result<(), String> {
    let path = flags.need("changelog")?;
    let version = flags.need("version")?;
    let out = PathBuf::from(flags.need("out")?);
    let changelog = std::fs::read_to_string(path).map_err(|e| format!("reading {path}: {e}"))?;
    let section = kanso_update::notes::section(&changelog, version).ok_or_else(|| {
        format!("{path} has no section for {version}; run scripts/release.sh first")
    })?;
    std::fs::write(&out, format!("{section}\n"))
        .map_err(|e| format!("writing {}: {e}", out.display()))?;
    println!("Wrote {}", out.display());
    Ok(())
}

fn check_notices(flags: &Flags) -> Result<(), String> {
    use kanso_core::notices::NOTICES;
    use kanso_update::notice_gate::{check_list, check_section};
    let mut problems = check_list(NOTICES);
    if flags.has("changelog") || flags.has("version") {
        let path = flags.need("changelog")?;
        let version = flags.need("version")?.trim_start_matches('v');
        let changelog =
            std::fs::read_to_string(path).map_err(|e| format!("reading {path}: {e}"))?;
        let section = kanso_update::notes::section(&changelog, version).ok_or_else(|| {
            format!("{path} has no section for {version}; run scripts/release.sh first")
        })?;
        problems.extend(check_section(&section, version, NOTICES));
    }
    if problems.is_empty() {
        println!("Update notices OK.");
        return Ok(());
    }
    let list: Vec<String> = problems.iter().map(|p| format!("  - {p}")).collect();
    Err(format!(
        "a breaking change can't ship without an update notice (docs/RELEASING.md \u{203a} Breaking changes):\n{}",
        list.join("\n")
    ))
}
