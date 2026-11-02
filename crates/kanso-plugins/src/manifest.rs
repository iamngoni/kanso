//! Reading a manifest and holding a bundle to its rules, at install and
//! again at every launch.
//!
//! IDs are reverse-DNS; versions are semver; every path is relative, inside
//! the bundle, with no `..`, no absolute path and no backslash: the rule the
//! icon theme importer applies to its SVG paths
//! (`crates/kanso-app/src/icon_theme_file.rs`, `is_valid_svg_path()`). The
//! caps mirror the icon theme caps, scaled for code.
//!
//! A manifest comes from a stranger, so it is read as untrusted: capped in
//! size, parsed strictly (an unknown key is an error naming it), and every
//! field checked before anything else looks at it. A bundle's file list is
//! held to the same standard before a single file is unpacked.

use std::collections::BTreeSet;

use ed25519_dalek::VerifyingKey;
use semver::Version;

use crate::model::{self, EditorPermission, Manifest, Surface};

/// The longest ID.
pub const MAX_ID_LEN: usize = 128;
/// The longest path inside a bundle, in bytes.
pub const MAX_PATH_LEN: usize = 255;

/// Whether `id` is a plugin ID: `^[a-z0-9]+(\.[a-z0-9-]+){2,}$`, at most
/// [`MAX_ID_LEN`] bytes. Pure.
pub fn is_valid_id(id: &str) -> bool {
    if id.is_empty() || id.len() > MAX_ID_LEN {
        return false;
    }
    let mut parts = id.split('.');
    let first = parts.next().unwrap_or_default();
    if first.is_empty()
        || !first
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
    {
        return false;
    }
    let mut rest = 0;
    for part in parts {
        if part.is_empty()
            || !part
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        {
            return false;
        }
        rest += 1;
    }
    rest >= 2
}

/// Whether `path` names a file inside a bundle: relative, `/`-separated,
/// each segment of letters, digits, `.`, `_` and `-` that isn't `.` or
/// `..`, at most [`MAX_PATH_LEN`] bytes. Pure.
pub fn is_valid_bundle_path(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= MAX_PATH_LEN
        && path.split('/').all(|segment| {
            !segment.is_empty()
                && segment != "."
                && segment != ".."
                && segment
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
        })
}

/// The largest `manifest.json` read, in bytes.
pub const MAX_MANIFEST_BYTES: usize = 64 * 1024;
/// The most a bundle's files may add up to, uncompressed.
pub const MAX_BUNDLE_BYTES: u64 = 5 * 1024 * 1024;
/// The most files a bundle may hold.
pub const MAX_ENTRIES: usize = 200;
/// The largest entry script, uncompressed.
pub const MAX_ENTRY_SCRIPT_BYTES: u64 = 2 * 1024 * 1024;
/// The largest icon, uncompressed.
pub const MAX_ICON_BYTES: u64 = 64 * 1024;

const MAX_NAME_CHARS: usize = 48;
const MAX_DESCRIPTION_CHARS: usize = 200;
const MAX_PUBLISHER_CHARS: usize = 64;
const MAX_NOTE_SCOPES: usize = 64;
const MAX_SCOPE_BYTES: usize = 64;
const MAX_SECRETS: usize = 8;
const MAX_HOSTS: usize = 16;
const MAX_HOST_LEN: usize = 253;
const MAX_LABEL_LEN: usize = 63;
const MAX_FENCE_LEN: usize = 32;

/// Languages Kanso draws itself; a plugin can't take them over.
const RESERVED_FENCES: &[&str] = &["math", "mermaid", "excalidraw", "tex", "latex"];
/// What a bundle may hold.
const ALLOWED_EXTENSIONS: &[&str] = &["js", "json", "svg", "png", "md", "txt"];
/// Archives inside an archive are refused outright.
const ARCHIVE_EXTENSIONS: &[&str] = &["zip", "kansoplugin", "tar", "gz", "tgz", "jar"];
/// Names that only ever mean this machine or its network.
const LOCAL_SUFFIXES: &[&str] = &[
    ".localhost",
    ".local",
    ".internal",
    ".lan",
    ".home.arpa",
    ".test",
    ".invalid",
];

/// Why a manifest or a bundle was refused, in words the person can read.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ManifestError {
    #[error("the manifest is larger than {} KiB", MAX_MANIFEST_BYTES / 1024)]
    TooLarge,
    #[error("the manifest can't be read: {0}")]
    Malformed(String),
    #[error("the manifest is for a layout this Kanso doesn't know")]
    WrongSchema,
    #[error("it needs a newer Kanso: {version} or later")]
    NeedsNewerKanso { version: String },
    #[error("it was made for a newer plugin API (version {0}) than this Kanso offers")]
    NewerApi(u32),
    #[error("it was made for a plugin API (version {0}) this Kanso no longer offers")]
    OldApi(u32),
    #[error("the plugin's ID isn't a reverse-DNS name like com.example.wordcount")]
    BadId,
    #[error("the plugin's name must be 1 to {MAX_NAME_CHARS} characters, with no line breaks")]
    BadName,
    #[error("the plugin's version isn't a semantic version like 1.2.3")]
    BadVersion,
    #[error("the entry must be a .js file inside the bundle")]
    BadEntry,
    #[error("the icon must be an .svg or .png file inside the bundle")]
    BadIcon,
    #[error("the description is too long or has control characters")]
    BadDescription,
    #[error("the publisher's name or key is wrong")]
    BadPublisher,
    #[error("a permission is wrong: {0}")]
    BadPermission(String),
    #[error("the bundle has no {0}")]
    MissingFile(String),
    #[error("{0:?} isn't a path or file type a bundle may hold")]
    BadPath(String),
    #[error("the bundle has more than {MAX_ENTRIES} files")]
    TooManyFiles,
    #[error("the bundle is larger than {} MiB", MAX_BUNDLE_BYTES / 1024 / 1024)]
    BundleTooLarge,
    #[error("{0} is too large")]
    FileTooLarge(String),
    #[error("{0} is an archive inside the bundle")]
    NestedArchive(String),
    #[error("{0} appears twice")]
    Duplicate(String),
}

/// Reads and checks a manifest. Pure.
pub fn parse(bytes: &[u8]) -> Result<Manifest, ManifestError> {
    if bytes.len() > MAX_MANIFEST_BYTES {
        return Err(ManifestError::TooLarge);
    }
    let manifest: Manifest =
        serde_json::from_slice(bytes).map_err(|e| ManifestError::Malformed(e.to_string()))?;
    validate(&manifest)?;
    Ok(manifest)
}

/// Holds a manifest to every rule above. Pure.
pub fn validate(m: &Manifest) -> Result<(), ManifestError> {
    if m.schema != model::SCHEMA {
        return Err(ManifestError::WrongSchema);
    }
    match m.api_version.cmp(&model::API_VERSION) {
        std::cmp::Ordering::Greater => return Err(ManifestError::NewerApi(m.api_version)),
        std::cmp::Ordering::Less => return Err(ManifestError::OldApi(m.api_version)),
        std::cmp::Ordering::Equal => {}
    }
    if !is_valid_id(&m.id) {
        return Err(ManifestError::BadId);
    }
    if m.name != m.name.trim()
        || m.name.is_empty()
        || m.name.chars().count() > MAX_NAME_CHARS
        || m.name.chars().any(char::is_control)
    {
        return Err(ManifestError::BadName);
    }
    if Version::parse(&m.version).is_err() || Version::parse(&m.min_app_version).is_err() {
        return Err(ManifestError::BadVersion);
    }
    if !is_valid_bundle_path(&m.entry) || !m.entry.ends_with(".js") {
        return Err(ManifestError::BadEntry);
    }
    if let Some(icon) = &m.icon
        && !(is_valid_bundle_path(icon) && (icon.ends_with(".svg") || icon.ends_with(".png")))
    {
        return Err(ManifestError::BadIcon);
    }
    if let Some(description) = &m.description
        && (description.chars().count() > MAX_DESCRIPTION_CHARS
            || description.chars().any(char::is_control))
    {
        return Err(ManifestError::BadDescription);
    }
    let publisher = &m.publisher;
    let name_len = publisher.name.chars().count();
    if name_len == 0
        || name_len > MAX_PUBLISHER_CHARS
        || publisher.name.chars().any(char::is_control)
        || !is_valid_key(&publisher.key)
    {
        return Err(ManifestError::BadPublisher);
    }
    validate_permissions(&m.permissions)
}

/// Whether `key` is `ed25519:` and 64 lowercase hex digits that make a
/// real public key. Pure.
fn is_valid_key(key: &str) -> bool {
    let Some(hex) = key.strip_prefix("ed25519:") else {
        return false;
    };
    if hex.len() != 64 || !hex.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f')) {
        return false;
    }
    let mut bytes = [0u8; 32];
    for (i, pair) in hex.as_bytes().chunks(2).enumerate() {
        let digit = |b: u8| if b <= b'9' { b - b'0' } else { b - b'a' + 10 };
        bytes[i] = digit(pair[0]) << 4 | digit(pair[1]);
    }
    VerifyingKey::from_bytes(&bytes).is_ok()
}

fn bad(why: impl Into<String>) -> ManifestError {
    ManifestError::BadPermission(why.into())
}

fn validate_scopes(what: &str, items: &[String]) -> Result<(), ManifestError> {
    if items.len() > MAX_NOTE_SCOPES {
        return Err(bad(format!("{what} lists more than {MAX_NOTE_SCOPES}")));
    }
    for item in items {
        if item.is_empty() || item.len() > MAX_SCOPE_BYTES || item.chars().any(char::is_control) {
            return Err(bad(format!("{what} has an entry that isn't a short name")));
        }
    }
    Ok(())
}

fn validate_permissions(p: &model::Permissions) -> Result<(), ManifestError> {
    if let Some(read) = &p.notes.read {
        validate_scopes("notes.read.books", &read.books)?;
        validate_scopes("notes.read.tags", &read.tags)?;
    }
    validate_scopes("notes.write", &p.notes.write)?;
    if p.notes.create && p.notes.write.is_empty() {
        return Err(bad("notes.create needs notes.write"));
    }
    let mut seen = BTreeSet::new();
    for fence in &p.fences {
        let lang = fence.language.as_str();
        if !is_valid_fence(lang) {
            return Err(bad(format!("{lang:?} isn't a fence language")));
        }
        if RESERVED_FENCES.contains(&lang) {
            return Err(bad(format!("Kanso draws {lang} blocks itself")));
        }
        if !seen.insert(lang) {
            return Err(bad(format!("{lang} is listed twice")));
        }
    }
    let editor_used = p.editor.iter().any(|e| {
        matches!(
            e,
            EditorPermission::NoteRead
                | EditorPermission::SelectionRead
                | EditorPermission::SelectionReplace
        )
    });
    if editor_used && !p.commands {
        return Err(bad("editor permissions need commands"));
    }
    if p.ui.contains(&Surface::Panel) && !p.commands {
        return Err(bad("a panel needs commands"));
    }
    if p.secrets.len() > MAX_SECRETS {
        return Err(bad(format!("more than {MAX_SECRETS} secrets")));
    }
    for secret in &p.secrets {
        let ok = !secret.is_empty()
            && secret.len() <= 64
            && secret
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-');
        if !ok {
            return Err(bad(format!("{secret:?} isn't a secret name")));
        }
    }
    if p.network.len() > MAX_HOSTS {
        return Err(bad(format!("more than {MAX_HOSTS} hosts")));
    }
    for host in &p.network {
        if !is_valid_host(host) {
            return Err(bad(format!("{host:?} isn't a host a plugin may reach")));
        }
    }
    Ok(())
}

/// `[a-z][a-z0-9-]{0,31}`. Pure.
fn is_valid_fence(lang: &str) -> bool {
    let mut bytes = lang.bytes();
    bytes.next().is_some_and(|b| b.is_ascii_lowercase())
        && lang.len() <= MAX_FENCE_LEN
        && bytes.all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// Whether `host` is a public DNS name a plugin may be allowed to reach:
/// lowercase, at least one dot, no port, wildcard or IP address, and not a
/// name that only means this machine or its network. Pure.
pub fn is_valid_host(host: &str) -> bool {
    if host.is_empty() || host.len() > MAX_HOST_LEN || !host.contains('.') {
        return false;
    }
    if host == "localhost" || LOCAL_SUFFIXES.iter().any(|s| host.ends_with(s)) {
        return false;
    }
    let labels: Vec<&str> = host.split('.').collect();
    for label in &labels {
        let ok = !label.is_empty()
            && label.len() <= MAX_LABEL_LEN
            && !label.starts_with('-')
            && !label.ends_with('-')
            && label
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
        if !ok {
            return false;
        }
    }
    // A numeric last label would be (part of) an IP address.
    !labels
        .last()
        .is_some_and(|l| l.bytes().all(|b| b.is_ascii_digit()))
}

/// Refuses a plugin that needs a newer Kanso than `app`. Pure.
pub fn check_app_version(m: &Manifest, app: &Version) -> Result<(), ManifestError> {
    let needed = Version::parse(&m.min_app_version).map_err(|_| ManifestError::BadVersion)?;
    if needed > *app {
        return Err(ManifestError::NeedsNewerKanso {
            version: m.min_app_version.clone(),
        });
    }
    Ok(())
}

/// Holds a bundle's file list, `(path, uncompressed size)` with
/// `manifest.json` among them, to the rules, before anything is unpacked.
/// Pure.
pub fn check_files(m: &Manifest, files: &[(String, u64)]) -> Result<(), ManifestError> {
    if files.len() > MAX_ENTRIES {
        return Err(ManifestError::TooManyFiles);
    }
    let mut seen = BTreeSet::new();
    let mut total: u64 = 0;
    for (path, size) in files {
        if !is_valid_bundle_path(path) {
            return Err(ManifestError::BadPath(path.clone()));
        }
        let name = path.rsplit('/').next().unwrap_or(path);
        let ext = name
            .rsplit_once('.')
            .map(|(_, e)| e.to_ascii_lowercase())
            .unwrap_or_default();
        if ARCHIVE_EXTENSIONS.contains(&ext.as_str()) {
            return Err(ManifestError::NestedArchive(path.clone()));
        }
        if !ALLOWED_EXTENSIONS.contains(&ext.as_str()) {
            return Err(ManifestError::BadPath(path.clone()));
        }
        if !seen.insert(path.to_lowercase()) {
            return Err(ManifestError::Duplicate(path.clone()));
        }
        total = total.saturating_add(*size);
    }
    if total > MAX_BUNDLE_BYTES {
        return Err(ManifestError::BundleTooLarge);
    }
    let size_of = |wanted: &str| files.iter().find(|(p, _)| p == wanted).map(|(_, s)| *s);
    if size_of("manifest.json").is_none() {
        return Err(ManifestError::MissingFile("manifest.json".to_owned()));
    }
    if m.entry == "manifest.json" || !m.entry.ends_with(".js") {
        return Err(ManifestError::BadEntry);
    }
    match size_of(&m.entry) {
        None => return Err(ManifestError::MissingFile(m.entry.clone())),
        Some(size) if size > MAX_ENTRY_SCRIPT_BYTES => {
            return Err(ManifestError::FileTooLarge(m.entry.clone()));
        }
        Some(_) => {}
    }
    if let Some(icon) = &m.icon {
        match size_of(icon) {
            None => return Err(ManifestError::MissingFile(icon.clone())),
            Some(size) if size > MAX_ICON_BYTES => {
                return Err(ManifestError::FileTooLarge(icon.clone()));
            }
            Some(_) => {}
        }
    }
    Ok(())
}

/// The fewest single-character insertions, deletions and substitutions
/// that turn `a` into `b`. Pure.
pub fn edit_distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut cur = vec![i + 1];
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != *cb);
            cur.push((prev[j] + cost).min(prev[j + 1] + 1).min(cur[j] + 1));
        }
        prev = cur;
    }
    prev[b.len()]
}

/// Known IDs within two edits of `id` but not equal to it, sorted, for the
/// "looks like" warning against typosquats. Pure.
pub fn lookalikes<'a>(id: &str, known: impl IntoIterator<Item = &'a str>) -> Vec<&'a str> {
    known
        .into_iter()
        .filter(|k| *k != id && edit_distance(id, k) <= 2)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Fence, NotesRequest, Permissions, Publisher, ReadRequest};
    use proptest::prelude::*;

    /// The RFC 8032 test vector's public key: a real curve point.
    pub(crate) const KEY: &str =
        "ed25519:d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a";

    type Change<T> = Box<dyn Fn(&mut T)>;

    fn manifest() -> Manifest {
        Manifest {
            schema: 1,
            api_version: 1,
            id: "com.samrivera.wordcount".into(),
            name: "Word count".into(),
            version: "1.4.0".into(),
            entry: "main.js".into(),
            publisher: Publisher {
                name: "Sam Rivera".into(),
                key: KEY.into(),
            },
            min_app_version: "0.1.0".into(),
            icon: Some("icon.svg".into()),
            description: Some("Counts words.".into()),
            permissions: Permissions::default(),
        }
    }

    fn files(m: &Manifest) -> Vec<(String, u64)> {
        let mut v = vec![("manifest.json".to_owned(), 500), (m.entry.clone(), 1000)];
        if let Some(i) = &m.icon {
            v.push((i.clone(), 100));
        }
        v
    }

    #[test]
    fn a_plain_manifest_is_accepted() {
        let m = manifest();
        assert_eq!(validate(&m), Ok(()));
        let bytes = serde_json::to_vec(&m).unwrap();
        assert_eq!(parse(&bytes).unwrap(), m);
    }

    #[test]
    fn unknown_keys_are_named() {
        let mut v = serde_json::to_value(manifest()).unwrap();
        v["permissions"]["root"] = true.into();
        let err = parse(&serde_json::to_vec(&v).unwrap()).unwrap_err();
        assert!(err.to_string().contains("root"), "{err}");
        v["permissions"].as_object_mut().unwrap().remove("root");
        v["surprise"] = 1.into();
        assert!(matches!(
            parse(&serde_json::to_vec(&v).unwrap()),
            Err(ManifestError::Malformed(_))
        ));
    }

    #[test]
    fn oversized_and_garbled_input_is_refused() {
        assert_eq!(
            parse(&vec![b' '; MAX_MANIFEST_BYTES + 1]),
            Err(ManifestError::TooLarge)
        );
        assert!(matches!(parse(b"{"), Err(ManifestError::Malformed(_))));
    }

    #[test]
    fn schema_and_api_must_match() {
        let mut m = manifest();
        m.schema = 2;
        assert_eq!(validate(&m), Err(ManifestError::WrongSchema));
        m.schema = 1;
        m.api_version = 2;
        assert_eq!(validate(&m), Err(ManifestError::NewerApi(2)));
        m.api_version = 0;
        assert_eq!(validate(&m), Err(ManifestError::OldApi(0)));
    }

    #[test]
    fn fields_are_checked() {
        let cases: Vec<(Change<Manifest>, ManifestError)> = vec![
            (Box::new(|m| m.id = "com.x".into()), ManifestError::BadId),
            (
                Box::new(|m| m.name = " Padded".into()),
                ManifestError::BadName,
            ),
            (Box::new(|m| m.name = "a\nb".into()), ManifestError::BadName),
            (
                Box::new(|m| m.name = "x".repeat(49)),
                ManifestError::BadName,
            ),
            (
                Box::new(|m| m.version = "1.4".into()),
                ManifestError::BadVersion,
            ),
            (
                Box::new(|m| m.min_app_version = "x".into()),
                ManifestError::BadVersion,
            ),
            (
                Box::new(|m| m.entry = "main.ts".into()),
                ManifestError::BadEntry,
            ),
            (
                Box::new(|m| m.entry = "../main.js".into()),
                ManifestError::BadEntry,
            ),
            (
                Box::new(|m| m.icon = Some("icon.gif".into())),
                ManifestError::BadIcon,
            ),
            (
                Box::new(|m| m.description = Some("x".repeat(201))),
                ManifestError::BadDescription,
            ),
            (
                Box::new(|m| m.publisher.name = String::new()),
                ManifestError::BadPublisher,
            ),
            (
                Box::new(|m| m.publisher.key = "ed25519:00".into()),
                ManifestError::BadPublisher,
            ),
            (
                Box::new(|m| m.publisher.key = KEY.to_uppercase().replace("ED25519", "ed25519")),
                ManifestError::BadPublisher,
            ),
        ];
        for (change, want) in cases {
            let mut m = manifest();
            change(&mut m);
            assert_eq!(validate(&m), Err(want));
        }
    }

    #[test]
    fn permissions_are_checked() {
        let tweaks: Vec<Change<Permissions>> = vec![
            Box::new(|p| p.notes.create = true),
            Box::new(|p| {
                p.fences = vec![Fence {
                    language: "mermaid".into(),
                }]
            }),
            Box::new(|p| {
                p.fences = vec![Fence {
                    language: "Chart".into(),
                }]
            }),
            Box::new(|p| {
                p.fences = vec![
                    Fence {
                        language: "chart".into(),
                    },
                    Fence {
                        language: "chart".into(),
                    },
                ]
            }),
            Box::new(|p| {
                p.editor.insert(EditorPermission::NoteRead);
            }),
            Box::new(|p| {
                p.ui.insert(Surface::Panel);
            }),
            Box::new(|p| {
                p.secrets.insert("bad name".into());
            }),
            Box::new(|p| p.secrets = (0..9).map(|i| format!("s{i}")).collect()),
            Box::new(|p| p.notes.write = vec!["x".into(); 65]),
            Box::new(|p| {
                p.notes.read = Some(ReadRequest {
                    books: vec!["y".repeat(65)],
                    ..Default::default()
                })
            }),
        ];
        for tweak in tweaks {
            let mut m = manifest();
            tweak(&mut m.permissions);
            assert!(matches!(validate(&m), Err(ManifestError::BadPermission(_))));
        }
        let mut m = manifest();
        m.permissions.commands = true;
        m.permissions.editor.insert(EditorPermission::SelectionRead);
        m.permissions.ui.insert(Surface::Panel);
        m.permissions.notes = NotesRequest {
            write: vec!["Work".into()],
            create: true,
            read: None,
        };
        m.permissions.fences = vec![Fence {
            language: "chart".into(),
        }];
        assert_eq!(validate(&m), Ok(()));
    }

    #[test]
    fn hosts_are_public_names_only() {
        for ok in [
            "api.clip.example",
            "a.io",
            "xn--abc.com",
            "a-b.c-d.org",
            "my.internal-tools.com",
        ] {
            assert!(is_valid_host(ok), "{ok}");
        }
        for bad in [
            "",
            "localhost",
            "a.localhost",
            "printer.local",
            "x.internal",
            "x.lan",
            "x.home.arpa",
            "x.test",
            "x.invalid",
            "host",
            "1.2.3.4",
            "a.1",
            "Host.com",
            "a.com:443",
            "*.a.com",
            "-a.com",
            "a-.com",
            "a..com",
            ".a.com",
            "a.com.",
            "a b.com",
            "http://a.com",
            "a.com/x",
            "a_b.com",
        ] {
            assert!(!is_valid_host(bad), "{bad}");
        }
        assert!(!is_valid_host(&format!("{}.com", "a".repeat(64))));
        assert!(!is_valid_host(&format!("{}.com", "a.".repeat(130))));
    }

    #[test]
    fn too_many_hosts_are_refused() {
        let mut m = manifest();
        m.permissions.network = (0..17).map(|i| format!("h{i}.example")).collect();
        assert!(matches!(validate(&m), Err(ManifestError::BadPermission(_))));
    }

    #[test]
    fn the_app_must_be_new_enough() {
        let mut m = manifest();
        m.min_app_version = "0.5.0".into();
        assert_eq!(check_app_version(&m, &Version::new(0, 5, 0)), Ok(()));
        assert_eq!(
            check_app_version(&m, &Version::new(0, 4, 9)),
            Err(ManifestError::NeedsNewerKanso {
                version: "0.5.0".into()
            })
        );
    }

    #[test]
    fn bundles_are_checked() {
        let m = manifest();
        assert_eq!(check_files(&m, &files(&m)), Ok(()));
        let with = |extra: (&str, u64)| {
            let mut f = files(&m);
            f.push((extra.0.to_owned(), extra.1));
            f
        };
        assert!(matches!(
            check_files(&m, &with(("a/../b.js", 1))),
            Err(ManifestError::BadPath(_))
        ));
        assert!(matches!(
            check_files(&m, &with(("x.exe", 1))),
            Err(ManifestError::BadPath(_))
        ));
        assert!(matches!(
            check_files(&m, &with(("x.zip", 1))),
            Err(ManifestError::NestedArchive(_))
        ));
        assert!(matches!(
            check_files(&m, &with(("x.KansoPlugin", 1))),
            Err(ManifestError::NestedArchive(_))
        ));
        assert!(matches!(
            check_files(&m, &with(("MAIN.js", 1))),
            Err(ManifestError::Duplicate(_))
        ));
        assert!(matches!(
            check_files(&m, &with(("main.js", 1))),
            Err(ManifestError::Duplicate(_))
        ));
        assert_eq!(
            check_files(&m, &with(("big.txt", MAX_BUNDLE_BYTES))),
            Err(ManifestError::BundleTooLarge)
        );
        let mut f = files(&m);
        f.retain(|(p, _)| p != "manifest.json");
        assert!(matches!(
            check_files(&m, &f),
            Err(ManifestError::MissingFile(_))
        ));
        let mut f = files(&m);
        f.retain(|(p, _)| p != "main.js");
        assert!(matches!(
            check_files(&m, &f),
            Err(ManifestError::MissingFile(_))
        ));
        let mut f = files(&m);
        f[1].1 = MAX_ENTRY_SCRIPT_BYTES + 1;
        assert!(matches!(
            check_files(&m, &f),
            Err(ManifestError::FileTooLarge(_))
        ));
        let mut f = files(&m);
        f[2].1 = MAX_ICON_BYTES + 1;
        assert!(matches!(
            check_files(&m, &f),
            Err(ManifestError::FileTooLarge(_))
        ));
        let many: Vec<(String, u64)> = (0..=MAX_ENTRIES)
            .map(|i| (format!("f{i}.txt"), 1))
            .collect();
        assert_eq!(check_files(&m, &many), Err(ManifestError::TooManyFiles));
        let mut m2 = manifest();
        m2.entry = "manifest.json".into();
        assert_eq!(check_files(&m2, &files(&m)), Err(ManifestError::BadEntry));
    }

    #[test]
    fn lookalikes_find_near_ids() {
        let known = [
            "com.samrivera.wordcount",
            "com.samrivera.wordcounts",
            "org.other.thing",
        ];
        assert_eq!(
            lookalikes("com.samriviera.wordcount", known),
            vec!["com.samrivera.wordcount", "com.samrivera.wordcounts"]
        );
        assert!(lookalikes("com.samrivera.wordcount", known).contains(&"com.samrivera.wordcounts"));
        assert!(!lookalikes("com.samrivera.wordcount", known).contains(&"com.samrivera.wordcount"));
        assert_eq!(edit_distance("kitten", "sitting"), 3);
        assert_eq!(edit_distance("", "abc"), 3);
    }

    fn arb_manifest() -> impl Strategy<Value = Manifest> {
        let tld = prop_oneof![Just("com"), Just("org"), Just("dev"), Just("io")];
        (
            "[a-z0-9]{1,8}(\\.[a-z0-9-]{1,8}){2,3}",
            "[A-Za-z0-9]{1,48}",
            (0u64..20, 0u64..20, 0u64..20),
            "[a-z]{1,8}\\.js",
            proptest::option::of("[a-z]{1,8}\\.(svg|png)"),
            proptest::collection::btree_set(
                ("[a-z]{1,10}", tld).prop_map(|(a, t)| format!("{a}.{t}")),
                0..5,
            ),
            proptest::collection::btree_set("[A-Za-z0-9_-]{1,12}", 0..4),
            any::<bool>(),
            proptest::collection::vec("[A-Za-z0-9 ]{1,20}", 0..4),
        )
            .prop_map(
                |(id, name, (a, b, c), entry, icon, hosts, secrets, create, write)| {
                    let mut m = manifest();
                    m.id = id;
                    m.name = name;
                    m.version = format!("{a}.{b}.{c}");
                    m.entry = entry;
                    m.icon = icon;
                    m.permissions.commands = true;
                    m.permissions.network = hosts;
                    m.permissions.secrets = secrets;
                    m.permissions.notes.create = create && !write.is_empty();
                    m.permissions.notes.write = write;
                    m
                },
            )
    }

    proptest! {
        /// Invariant: a manifest that passes validation re-serialises and
        /// re-parses to the same manifest.
        #[test]
        fn prop_valid_manifests_round_trip(m in arb_manifest()) {
            prop_assert_eq!(validate(&m), Ok(()));
            let bytes = serde_json::to_vec(&m).unwrap();
            prop_assert_eq!(parse(&bytes).unwrap(), m);
        }

        /// Invariant: breaking the ID is always refused.
        #[test]
        fn prop_bad_ids_are_refused(m in arb_manifest(), tail in prop_oneof![Just("_"), Just("A"), Just("/"), Just(" "), Just(".."), Just(".")]) {
            let mut m = m;
            m.id.push_str(tail);
            prop_assert_eq!(validate(&m), Err(ManifestError::BadId));
        }

        /// Invariant: an entry that climbs out, isn't a script or is the
        /// manifest is always refused.
        #[test]
        fn prop_bad_entries_are_refused(m in arb_manifest(), which in 0usize..5) {
            let mut m = m;
            m.entry = match which {
                0 => format!("../{}", m.entry),
                1 => format!("/{}", m.entry),
                2 => "a\\b.js".to_owned(),
                3 => m.entry.replace(".js", ".ts"),
                _ => "manifest.json".to_owned(),
            };
            prop_assert_eq!(validate(&m), Err(ManifestError::BadEntry));
        }

        /// Invariant: a host that is local, an address, has a port or a
        /// wildcard is always refused.
        #[test]
        fn prop_bad_hosts_are_refused(m in arb_manifest(), host in prop_oneof![
            Just("localhost"), Just("a.b:80"), Just("*.example.com"), Just("1.2.3.4"),
            Just("x.local"), Just("-a.com"), Just("a..com"), Just("host"),
            Just("192.168.0.1"), Just("Host.com"), Just("a.home.arpa"), Just("a.lan")]) {
            let mut m = m;
            m.permissions.network.insert(host.to_owned());
            prop_assert!(matches!(validate(&m), Err(ManifestError::BadPermission(_))));
        }

        /// Invariant: a permission key the schema doesn't know is always refused.
        #[test]
        fn prop_unknown_permission_keys_are_refused(m in arb_manifest(), key in "x_[a-z]{1,8}") {
            let mut v = serde_json::to_value(&m).unwrap();
            v["permissions"][key.as_str()] = true.into();
            let err = parse(&serde_json::to_vec(&v).unwrap()).unwrap_err();
            prop_assert!(matches!(err, ManifestError::Malformed(_)));
            prop_assert!(err.to_string().contains(&key));
        }

        /// Invariant: no file list with a `..` segment is accepted.
        #[test]
        fn prop_check_files_never_accepts_dotdot(prefix in "[a-z]{0,5}", suffix in "[a-z]{0,5}\\.js") {
            let m = manifest();
            let mut f = files(&m);
            f.push((format!("{prefix}/../{suffix}"), 1));
            prop_assert!(check_files(&m, &f).is_err());
        }

        /// Invariant: a bundle whose files add up past the cap is refused,
        /// and an accepted one is within every cap.
        #[test]
        fn prop_check_files_enforces_caps(sizes in proptest::collection::vec(0u64..3_000_000, 0..12)) {
            let m = manifest();
            let mut f = files(&m);
            for (i, s) in sizes.iter().enumerate() {
                f.push((format!("d{i}.txt"), *s));
            }
            let total: u64 = f.iter().map(|(_, s)| s).sum();
            let result = check_files(&m, &f);
            if total > MAX_BUNDLE_BYTES {
                prop_assert_eq!(result, Err(ManifestError::BundleTooLarge));
            } else {
                prop_assert_eq!(result, Ok(()));
            }
        }

        /// Invariant: edit distance is a metric bounded by the longer string.
        #[test]
        fn prop_edit_distance_is_a_metric(a in "[a-c.]{0,10}", b in "[a-c.]{0,10}", c in "[a-c.]{0,10}") {
            prop_assert_eq!(edit_distance(&a, &a), 0);
            prop_assert_eq!(edit_distance(&a, &b), edit_distance(&b, &a));
            prop_assert!(edit_distance(&a, &c) <= edit_distance(&a, &b) + edit_distance(&b, &c));
            prop_assert!(edit_distance(&a, &b) <= a.chars().count().max(b.chars().count()));
            if a != b {
                prop_assert!(edit_distance(&a, &b) > 0);
            }
        }

        /// Invariant: lookalikes are sorted, unique, never the ID itself and
        /// always within two edits.
        #[test]
        fn prop_lookalikes_are_near_and_distinct(id in "[a-c.]{0,8}", known in proptest::collection::vec("[a-c.]{0,8}", 0..12)) {
            let got = lookalikes(&id, known.iter().map(String::as_str));
            prop_assert!(got.windows(2).all(|w| w[0] < w[1]));
            for k in &got {
                prop_assert!(*k != id && edit_distance(&id, k) <= 2);
            }
            for k in &known {
                if *k != id && edit_distance(&id, k) <= 2 {
                    prop_assert!(got.contains(&k.as_str()));
                }
            }
        }
    }
}

#[cfg(test)]
mod seed_tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn ids_follow_the_design() {
        assert!(is_valid_id("com.samrivera.wordcount"));
        assert!(is_valid_id("app.kanso.chart-2"));
        assert!(!is_valid_id("com.example"));
        assert!(!is_valid_id("Com.example.x"));
        assert!(!is_valid_id("com..example.x"));
        assert!(!is_valid_id("com.example.x."));
        assert!(!is_valid_id("com-x.example.y"));
    }

    #[test]
    fn paths_stay_inside() {
        assert!(is_valid_bundle_path("main.js"));
        assert!(is_valid_bundle_path("lib/chart.js"));
        for bad in [
            "",
            "/main.js",
            "../main.js",
            "a/../b.js",
            "a\\b.js",
            "./a.js",
            "a//b.js",
        ] {
            assert!(!is_valid_bundle_path(bad), "{bad}");
        }
    }

    proptest! {
        /// Invariant: an accepted path never climbs out or starts at the root.
        #[test]
        fn prop_paths_never_escape(path in "[a-z./\\\\_-]{0,24}") {
            if is_valid_bundle_path(&path) {
                prop_assert!(!path.starts_with('/'));
                prop_assert!(!path.split('/').any(|s| s == ".." || s == "." || s.is_empty()));
                prop_assert!(!path.contains('\\'));
            }
        }

        /// Invariant: an accepted ID has at least three dot-separated parts.
        #[test]
        fn prop_ids_have_three_parts(id in "[a-z0-9.-]{0,20}") {
            if is_valid_id(&id) {
                prop_assert!(id.split('.').count() >= 3);
                prop_assert!(!id.starts_with('-'));
            }
        }
    }
}
