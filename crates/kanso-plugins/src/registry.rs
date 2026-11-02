//! The plugin directory: a signed `index.json`, what Kanso makes of it, and
//! how it is fetched and kept.
//!
//! The construction is the update feed's and the theme gallery's. The index
//! is signed as exact bytes with `index.json.sig`, the signature is checked
//! before the JSON is parsed, and the public keys are compiled in from
//! `keys/registry-keys.txt`, a list so a key can rotate. This list is
//! separate from the update keys and the gallery's: a key for one must never
//! be able to sign another's file.
//!
//! The index vouches for bundles by hash. A bundle is installed only when
//! its bytes match an entry's size and SHA-256, its author's signature over
//! that hash verifies, and its manifest says what the entry says. The index
//! never decides what a plugin may do; the manifest does, and the person
//! consents to it.
//!
//! Revocation fails closed: a version that can't be read, or that a rule
//! names, doesn't run. A short list is compiled in for Macs that have not been
//! online since a plugin was found to be harmful.

use std::collections::{BTreeMap, BTreeSet};
use std::io::Read as _;
use std::path::PathBuf;
use std::time::Duration;

use ed25519_dalek::VerifyingKey;
use kanso_update::feed::{is_sha256_hex, parse_key_list, verify_signature_with_cap};
use semver::{Version, VersionReq};
use url::Url;

use crate::bundle::{self, BundleError, MAX_ARCHIVE_BYTES};
use crate::consent;
use crate::manifest::is_valid_id;
use crate::model::Installed;
use crate::registry_model::{Bundle, Entry, INDEX_SCHEMA, Index, Offer, Revocation};

/// The repository the directory is published from, as `owner/name`.
pub const REPOSITORY: &str = "iamngoni/kanso-plugins";
/// The only host the app asks: GitHub's raw file server.
pub const HOST: &str = "raw.githubusercontent.com";
/// Where in the repository the published directory lives.
const DIRECTORY_PATH: &str = "main/directory";
/// The directory's index.
pub const INDEX_URL: &str =
    "https://raw.githubusercontent.com/iamngoni/kanso-plugins/main/directory/index.json";
/// Its signature.
pub const SIGNATURE_URL: &str =
    "https://raw.githubusercontent.com/iamngoni/kanso-plugins/main/directory/index.json.sig";
/// The largest index read, in bytes.
pub const MAX_INDEX_BYTES: u64 = 4 * 1024 * 1024;
/// The largest signature read, in bytes.
pub const MAX_SIGNATURE_BYTES: u64 = 1024;
/// The most entries, and the most revocations, an index may hold.
pub const MAX_INDEX_ENTRIES: usize = 4096;

/// Revoked on purpose and compiled in: `(id, version requirement, reason)`,
/// for Macs that haven't been online since. Empty until a real one is needed.
pub const COMPILED_REVOCATIONS: &[(&str, &str, &str)] = &[];

const MAX_REDIRECTS: usize = 3;
const MAX_NAME_CHARS: usize = 48;
const MAX_PUBLISHER_CHARS: usize = 64;
const MAX_DESCRIPTION_CHARS: usize = 300;
const MAX_REASON_CHARS: usize = 300;
const MAX_TIME_CHARS: usize = 40;
const BLOCKED: &str = "You blocked this plugin.";
const UNREADABLE: &str = "This version can't be checked, so it isn't allowed to run.";
const WITHDRAWN: &str = "The directory withdrew this version.";

/// The address of a published bundle. `id` and `version` must already be
/// valid; they are checked again where the index is checked. Pure.
pub fn bundle_url(id: &str, version: &str) -> String {
    format!("https://{HOST}/{REPOSITORY}/{DIRECTORY_PATH}/bundles/{id}/{version}.kansoplugin")
}

/// Why the directory, an index or a bundle was refused.
#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("the plugin directory isn't set up in this build")]
    NotConfigured,
    #[error("the directory's signature doesn't match")]
    BadSignature,
    #[error("the directory is older than one this Mac has already seen")]
    Replay,
    #[error("the directory can't be read: {0}")]
    Malformed(String),
    #[error("the directory lists a plugin wrongly: {0}")]
    BadEntry(String),
    #[error("the bundle isn't the one the directory lists")]
    HashMismatch,
    #[error("the bundle's size isn't the one the directory lists")]
    SizeMismatch,
    #[error("the author's signature doesn't match the bundle")]
    BadAuthorSignature,
    #[error("the bundle says something different from the directory: {0}")]
    ManifestMismatch(String),
    #[error(transparent)]
    Bundle(#[from] BundleError),
    #[error("couldn't reach the plugin directory: {0}")]
    Fetch(String),
    #[error("the answer is larger than expected")]
    TooLarge,
}

/// The keys compiled into this build, from `keys/registry-keys.txt`.
pub fn trusted_keys() -> Vec<VerifyingKey> {
    parse_key_list(include_str!("../keys/registry-keys.txt"))
}

/// Whether this build has a key to check the directory with. Without one,
/// Settings says so and nothing is fetched.
pub fn is_configured() -> bool {
    !trusted_keys().is_empty()
}

/// Checks a fetched index: the signature over its exact bytes first, then
/// the JSON, then every entry. `last_serial` is the newest serial this Mac
/// has seen; an older index is a replay.
pub fn check_index(
    bytes: &[u8],
    signature: &[u8],
    keys: &[VerifyingKey],
    last_serial: Option<u64>,
) -> Result<Index, RegistryError> {
    if keys.is_empty() {
        return Err(RegistryError::NotConfigured);
    }
    verify_signature_with_cap(bytes, signature, keys, MAX_INDEX_BYTES, MAX_SIGNATURE_BYTES)
        .map_err(|e| match e {
            kanso_update::feed::FeedError::TooLarge => RegistryError::TooLarge,
            _ => RegistryError::BadSignature,
        })?;
    let index: Index =
        serde_json::from_slice(bytes).map_err(|e| RegistryError::Malformed(e.to_string()))?;
    validate_index(&index)?;
    if last_serial.is_some_and(|last| index.serial < last) {
        return Err(RegistryError::Replay);
    }
    Ok(index)
}

/// Holds a parsed index to every rule except its signature: the layout,
/// the caps, each entry and each revocation. The authoring tool runs it
/// before it signs. Pure.
pub fn validate_index(index: &Index) -> Result<(), RegistryError> {
    if index.schema != INDEX_SCHEMA {
        return Err(RegistryError::Malformed(
            "it is for a layout this Kanso doesn't know".to_owned(),
        ));
    }
    if index.plugins.len() > MAX_INDEX_ENTRIES || index.revoked.len() > MAX_INDEX_ENTRIES {
        return Err(RegistryError::Malformed(
            "it lists too many plugins".to_owned(),
        ));
    }
    let mut seen = BTreeSet::new();
    for entry in &index.plugins {
        check_entry(entry)?;
        if !seen.insert((entry.id.as_str(), entry.version.as_str())) {
            return Err(bad(entry, "listed twice"));
        }
    }
    for revocation in &index.revoked {
        check_revocation(revocation)?;
    }
    Ok(())
}

fn bad(entry: &Entry, why: &str) -> RegistryError {
    RegistryError::BadEntry(format!("{} {}: {why}", entry.id, entry.version))
}

fn check_entry(entry: &Entry) -> Result<(), RegistryError> {
    if !is_valid_id(&entry.id) {
        return Err(bad(entry, "its ID isn't valid"));
    }
    let version = Version::parse(&entry.version)
        .ok()
        .filter(|v| v.build.is_empty() && v.to_string() == entry.version);
    if version.is_none() {
        return Err(bad(entry, "its version isn't valid"));
    }
    if Version::parse(&entry.min_app_version).is_err() {
        return Err(bad(entry, "its minimum Kanso version isn't valid"));
    }
    if !is_sha256_hex(&entry.sha256) {
        return Err(bad(entry, "its hash isn't valid"));
    }
    if entry.size == 0 || entry.size > MAX_ARCHIVE_BYTES {
        return Err(bad(entry, "its size isn't valid"));
    }
    if bundle::parse_publisher_key(&entry.signer).is_err() {
        return Err(bad(entry, "its signer's key isn't valid"));
    }
    let signature_ok = base64_len(&entry.author_signature) == Some(64);
    if !signature_ok {
        return Err(bad(entry, "its author signature isn't valid"));
    }
    if !clean(&entry.name, 1, MAX_NAME_CHARS)
        || !clean(&entry.publisher, 1, MAX_PUBLISHER_CHARS)
        || !clean(&entry.description, 0, MAX_DESCRIPTION_CHARS)
        || !clean(&entry.published_at, 1, MAX_TIME_CHARS)
    {
        return Err(bad(entry, "its name, publisher or description isn't clean"));
    }
    Ok(())
}

fn check_revocation(revocation: &Revocation) -> Result<(), RegistryError> {
    let fail =
        |why: &str| RegistryError::BadEntry(format!("revocation of {}: {why}", revocation.id));
    if !is_valid_id(&revocation.id) {
        return Err(fail("the ID isn't valid"));
    }
    if VersionReq::parse(&revocation.versions).is_err() {
        return Err(fail("the versions aren't a valid requirement"));
    }
    if !clean(&revocation.reason, 0, MAX_REASON_CHARS) {
        return Err(fail("the reason isn't clean"));
    }
    Ok(())
}

fn base64_len(text: &str) -> Option<usize> {
    use base64::Engine as _;
    base64::engine::general_purpose::STANDARD
        .decode(text)
        .ok()
        .map(|b| b.len())
}

/// Text a person will read: within `min..=max` characters, trimmed, with no
/// control characters and none of the invisible ones that reorder or hide
/// text.
fn clean(text: &str, min: usize, max: usize) -> bool {
    let count = text.chars().count();
    count >= min
        && count <= max
        && text.trim() == text
        && !text.chars().any(|c| {
            c.is_control()
                || matches!(c, '\u{200B}'..='\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2060}'..='\u{2064}' | '\u{2066}'..='\u{2069}' | '\u{FEFF}')
        })
}

/// Checks a downloaded bundle against the entry that vouches for it: size and
/// hash, the author's signature, then the bundle itself, then that its
/// manifest says what the entry says.
pub fn verify_bundle(bytes: &[u8], entry: &Entry) -> Result<Bundle, RegistryError> {
    if bytes.len() as u64 != entry.size {
        return Err(RegistryError::SizeMismatch);
    }
    let hash = bundle::sha256_hex(bytes);
    if hash != entry.sha256 {
        return Err(RegistryError::HashMismatch);
    }
    bundle::verify_hash(&hash, &entry.author_signature, &entry.signer)
        .map_err(|_| RegistryError::BadAuthorSignature)?;
    let opened = bundle::open(bytes)?;
    let m = &opened.manifest;
    let mismatch = |what: &str| Err(RegistryError::ManifestMismatch(what.to_owned()));
    if m.id != entry.id {
        return mismatch("the ID");
    }
    if m.version != entry.version {
        return mismatch("the version");
    }
    if m.publisher.key != entry.signer {
        return mismatch("the author's key");
    }
    if m.permissions != entry.permissions {
        return mismatch("the permissions");
    }
    if m.min_app_version != entry.min_app_version {
        return mismatch("the minimum Kanso version");
    }
    Ok(opened)
}

/// The entry whose bundle has this hash: a `.kansoplugin` opened from Finder
/// is trusted only if the signed index lists it. Pure.
pub fn find_by_hash<'a>(index: &'a Index, sha256: &str) -> Option<&'a Entry> {
    index.plugins.iter().find(|entry| entry.sha256 == sha256)
}

/// Every reason a version may not run, read once so many versions can be
/// asked about cheaply.
struct Revoker<'a> {
    blocked: &'a BTreeSet<String>,
    /// Rules by ID; a requirement that can't be read matches everything.
    rules: BTreeMap<String, Vec<(Option<VersionReq>, String)>>,
    /// `(id, version)` pairs the index marks revoked.
    exact: BTreeSet<(String, String)>,
}

impl<'a> Revoker<'a> {
    fn new(index: Option<&Index>, blocked: &'a BTreeSet<String>) -> Revoker<'a> {
        let mut rules: BTreeMap<String, Vec<(Option<VersionReq>, String)>> = BTreeMap::new();
        for (id, req, reason) in COMPILED_REVOCATIONS {
            rules
                .entry((*id).to_owned())
                .or_default()
                .push((VersionReq::parse(req).ok(), (*reason).to_owned()));
        }
        let mut exact = BTreeSet::new();
        if let Some(index) = index {
            for r in &index.revoked {
                rules.entry(r.id.clone()).or_default().push((
                    VersionReq::parse(&r.versions).ok(),
                    reason_or_default(&r.reason),
                ));
            }
            for entry in index.plugins.iter().filter(|e| e.revoked) {
                exact.insert((entry.id.clone(), normal(&entry.version)));
            }
        }
        Revoker {
            blocked,
            rules,
            exact,
        }
    }

    fn reason(&self, id: &str, version: &str) -> Option<String> {
        if self.blocked.contains(id) {
            return Some(BLOCKED.to_owned());
        }
        let Ok(parsed) = Version::parse(version) else {
            return Some(UNREADABLE.to_owned());
        };
        if let Some(rules) = self.rules.get(id) {
            for (req, reason) in rules {
                if req.as_ref().is_none_or(|req| req_matches(req, &parsed)) {
                    return Some(reason.clone());
                }
            }
        }
        if self.exact.contains(&(id.to_owned(), normal(version))) {
            return Some(WITHDRAWN.to_owned());
        }
        None
    }
}

fn reason_or_default(reason: &str) -> String {
    if reason.is_empty() {
        WITHDRAWN.to_owned()
    } else {
        reason.to_owned()
    }
}

fn normal(version: &str) -> String {
    Version::parse(version).map_or_else(|_| version.to_owned(), |v| v.to_string())
}

/// Whether `req` names `version`. A requirement like `*` skips pre-releases
/// by semver's rules, which would let `1.0.0-beta` slip past a revocation of
/// everything, so the release it belongs to is asked too.
fn req_matches(req: &VersionReq, version: &Version) -> bool {
    req.matches(version) || req.matches(&Version::new(version.major, version.minor, version.patch))
}

/// Why this version of a plugin must not run, if it must not: the compiled
/// list, the index's revocations and revoked entries, and this Mac's block
/// list. A version that can't be read counts as revoked. Pure.
pub fn revocation_reason(
    index: Option<&Index>,
    blocked: &BTreeSet<String>,
    id: &str,
    version: &str,
) -> Option<String> {
    Revoker::new(index, blocked).reason(id, version)
}

/// The update on offer for an installed plugin, if there is one: the highest
/// version of it that isn't revoked, is strictly newer than the installed
/// one and runs on `app`. Developer installs are never offered updates.
/// Pure.
pub fn offer(
    installed: &Installed,
    index: &Index,
    app: &Version,
    blocked: &BTreeSet<String>,
) -> Option<Offer> {
    if installed.is_developer() {
        return None;
    }
    let current = Version::parse(&installed.manifest.version).ok()?;
    let revoker = Revoker::new(Some(index), blocked);
    let (_, entry) = index
        .plugins
        .iter()
        .filter(|entry| entry.id == installed.manifest.id)
        .filter_map(|entry| Some((Version::parse(&entry.version).ok()?, entry)))
        .filter(|(version, entry)| {
            *version > current
                && Version::parse(&entry.min_app_version).is_ok_and(|min| min <= *app)
                && revoker.reason(&entry.id, &entry.version).is_none()
        })
        .max_by(|a, b| a.0.cmp(&b.0))?;
    let (added, removed) = consent::diff(&installed.manifest.permissions, &entry.permissions);
    Some(Offer {
        key_changed: entry.signer != installed.pinned_key,
        entry: entry.clone(),
        added,
        removed,
    })
}

/// The highest version of each plugin that isn't revoked.
fn latest_each<'a>(
    index: &'a Index,
    only: Option<&str>,
) -> BTreeMap<&'a str, (Version, &'a Entry)> {
    let none = BTreeSet::new();
    let revoker = Revoker::new(Some(index), &none);
    let mut best: BTreeMap<&str, (Version, &Entry)> = BTreeMap::new();
    for entry in &index.plugins {
        if only.is_some_and(|id| id != entry.id) {
            continue;
        }
        let Ok(version) = Version::parse(&entry.version) else {
            continue;
        };
        if revoker.reason(&entry.id, &entry.version).is_some() {
            continue;
        }
        match best.get(entry.id.as_str()) {
            Some((have, _)) if *have >= version => {}
            _ => {
                best.insert(entry.id.as_str(), (version, entry));
            }
        }
    }
    best
}

/// The highest version of `id` that isn't revoked. Pure.
pub fn latest<'a>(index: &'a Index, id: &str) -> Option<&'a Entry> {
    latest_each(index, Some(id)).remove(id).map(|(_, e)| e)
}

/// The latest version of each plugin whose name or ID contains `query`
/// (ignoring case), sorted by name. An empty query lists them all. Pure.
pub fn search<'a>(index: &'a Index, query: &str) -> Vec<&'a Entry> {
    let query = query.trim().to_lowercase();
    let mut found: Vec<&Entry> = latest_each(index, None)
        .into_values()
        .map(|(_, entry)| entry)
        .filter(|entry| {
            entry.name.to_lowercase().contains(&query) || entry.id.to_lowercase().contains(&query)
        })
        .collect();
    found.sort_by(|a, b| {
        a.name
            .to_lowercase()
            .cmp(&b.name.to_lowercase())
            .then_with(|| a.id.cmp(&b.id))
    });
    found
}

/// The last good index on disk, so the directory can be browsed offline and
/// revocations apply at launch. It is checked again on every load.
pub struct Cache {
    dir: PathBuf,
}

const CACHE_INDEX: &str = "index.json";
const CACHE_SIGNATURE: &str = "index.json.sig";

impl Cache {
    /// A cache in `dir`, made on first save.
    pub fn new(dir: PathBuf) -> Cache {
        Cache { dir }
    }

    /// Writes the index and its signature, each through a temporary file and
    /// a rename, so a reader never sees half of one.
    pub fn save(&self, index_bytes: &[u8], signature: &[u8]) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.dir)?;
        write_atomic(&self.dir, CACHE_INDEX, index_bytes)?;
        write_atomic(&self.dir, CACHE_SIGNATURE, signature)
    }

    /// The cached index, if it is still signed by one of `keys` and not
    /// older than `last_serial`.
    pub fn load(&self, keys: &[VerifyingKey], last_serial: Option<u64>) -> Option<Index> {
        let bytes = read_capped(&self.dir.join(CACHE_INDEX), MAX_INDEX_BYTES)?;
        let signature = read_capped(&self.dir.join(CACHE_SIGNATURE), MAX_SIGNATURE_BYTES)?;
        check_index(&bytes, &signature, keys, last_serial).ok()
    }
}

fn read_capped(path: &std::path::Path, cap: u64) -> Option<Vec<u8>> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .ok()?
        .take(cap + 1)
        .read_to_end(&mut bytes)
        .ok()?;
    (bytes.len() as u64 <= cap).then_some(bytes)
}

fn write_atomic(dir: &std::path::Path, name: &str, bytes: &[u8]) -> std::io::Result<()> {
    use std::io::Write as _;
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let temp = dir.join(format!(".{name}.tmp-{}-{nanos}", std::process::id()));
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    let result = (|| {
        let mut file = options.open(&temp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        std::fs::rename(&temp, dir.join(name))
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temp);
    }
    result
}

// ---- fetching ----

/// Where the directory is fetched from: the fixed https address, or in a
/// debug build the loopback `KANSO_PLUGIN_INDEX=<base url>`.
#[derive(Clone, Debug)]
pub struct Endpoint {
    base: String,
    /// Set only by the debug override: the one loopback origin allowed.
    loopback: Option<Url>,
}

impl Endpoint {
    /// The real directory.
    pub fn directory() -> Endpoint {
        Endpoint {
            base: format!("https://{HOST}/{REPOSITORY}/{DIRECTORY_PATH}"),
            loopback: None,
        }
    }

    /// A loopback test server at `base` (`http://127.0.0.1:<port>`).
    pub fn loopback(base: &str) -> Result<Endpoint, RegistryError> {
        let url = Url::parse(base).map_err(|_| RegistryError::Fetch("bad address".to_owned()))?;
        let loopback_host = match url.host() {
            Some(url::Host::Ipv4(ip)) => ip.is_loopback(),
            Some(url::Host::Ipv6(ip)) => ip.is_loopback(),
            Some(url::Host::Domain(name)) => name.eq_ignore_ascii_case("localhost"),
            None => false,
        };
        if url.scheme() != "http" || !loopback_host || !url.username().is_empty() {
            return Err(RegistryError::Fetch(
                "the override must be a loopback http address".to_owned(),
            ));
        }
        Ok(Endpoint {
            base: base.trim_end_matches('/').to_owned(),
            loopback: Some(url),
        })
    }

    /// This build's endpoint: the directory, or a debug build's
    /// `KANSO_PLUGIN_INDEX`.
    pub fn from_env() -> Result<Endpoint, RegistryError> {
        if cfg!(debug_assertions)
            && let Ok(base) = std::env::var("KANSO_PLUGIN_INDEX")
            && !base.is_empty()
        {
            return Endpoint::loopback(&base);
        }
        Ok(Endpoint::directory())
    }

    fn index_url(&self) -> String {
        format!("{}/index.json", self.base)
    }

    fn signature_url(&self) -> String {
        format!("{}/index.json.sig", self.base)
    }

    fn bundle_url(&self, id: &str, version: &str) -> String {
        format!("{}/bundles/{id}/{version}.kansoplugin", self.base)
    }

    /// Whether a request (the first, or any redirect hop) may be made.
    fn allows(&self, url: &Url) -> bool {
        if !url.username().is_empty() || url.password().is_some() || url.fragment().is_some() {
            return false;
        }
        let low = url.as_str().to_ascii_lowercase();
        if low.contains("%2e") || low.contains("%2f") || low.contains("%5c") {
            return false;
        }
        match &self.loopback {
            Some(only) => {
                url.scheme() == "http"
                    && url.host() == only.host()
                    && url.port_or_known_default() == only.port_or_known_default()
            }
            // That host, under this repository only: another repository on
            // the same server is anyone's.
            None => {
                url.scheme() == "https"
                    && url.port().is_none()
                    && url.host_str().is_some_and(|h| h.eq_ignore_ascii_case(HOST))
                    && url
                        .path()
                        .strip_prefix(&format!("/{REPOSITORY}/"))
                        .is_some_and(|rest| !rest.is_empty())
                    && url.query().is_none()
            }
        }
    }
}

/// An HTTP client for the directory: no redirects of its own (each hop is
/// checked here) and statuses left to the caller.
pub fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .http_status_as_error(false)
        .max_redirects(0)
        .timeout_connect(Some(Duration::from_secs(20)))
        .timeout_recv_body(Some(Duration::from_secs(60)))
        .user_agent("Kanso")
        .build()
        .into()
}

/// Fetches `index.json` and its signature from this build's directory.
pub fn fetch_index(agent: &ureq::Agent) -> Result<(Vec<u8>, Vec<u8>), RegistryError> {
    fetch_index_from(agent, &Endpoint::from_env()?)
}

/// Fetches a bundle: the entry's, at its fixed address.
pub fn fetch_bundle(agent: &ureq::Agent, entry: &Entry) -> Result<Vec<u8>, RegistryError> {
    fetch_bundle_from(agent, &Endpoint::from_env()?, entry)
}

/// [`fetch_index`] from a given endpoint.
pub fn fetch_index_from(
    agent: &ureq::Agent,
    endpoint: &Endpoint,
) -> Result<(Vec<u8>, Vec<u8>), RegistryError> {
    let index = get(agent, endpoint, &endpoint.index_url(), MAX_INDEX_BYTES)?;
    let signature = get(
        agent,
        endpoint,
        &endpoint.signature_url(),
        MAX_SIGNATURE_BYTES,
    )?;
    Ok((index, signature))
}

/// [`fetch_bundle`] from a given endpoint.
pub fn fetch_bundle_from(
    agent: &ureq::Agent,
    endpoint: &Endpoint,
    entry: &Entry,
) -> Result<Vec<u8>, RegistryError> {
    let version_ok = Version::parse(&entry.version).is_ok_and(|v| v.build.is_empty());
    if !is_valid_id(&entry.id) || !version_ok {
        return Err(RegistryError::Fetch(
            "the entry's address isn't valid".to_owned(),
        ));
    }
    let cap = entry.size.min(MAX_ARCHIVE_BYTES);
    get(
        agent,
        endpoint,
        &endpoint.bundle_url(&entry.id, &entry.version),
        cap,
    )
}

/// One GET, redirects followed a hop at a time with every address checked,
/// the body read up to `cap` bytes.
fn get(
    agent: &ureq::Agent,
    endpoint: &Endpoint,
    url: &str,
    cap: u64,
) -> Result<Vec<u8>, RegistryError> {
    let config = agent.config();
    if config.max_redirects() != 0 || config.http_status_as_error() {
        return Err(RegistryError::Fetch(
            "the HTTP client must leave redirects to the directory code".to_owned(),
        ));
    }
    let refused = || RegistryError::Fetch("an address isn't allowed".to_owned());
    let mut current = Url::parse(url).map_err(|_| refused())?;
    for _ in 0..=MAX_REDIRECTS {
        if !endpoint.allows(&current) {
            return Err(refused());
        }
        let response = agent.get(current.as_str()).call().map_err(|_| {
            RegistryError::Fetch("offline or the server couldn't be reached".to_owned())
        })?;
        let status = response.status().as_u16();
        match status {
            200 => {
                let mut bytes = Vec::new();
                response
                    .into_body()
                    .into_reader()
                    .take(cap + 1)
                    .read_to_end(&mut bytes)
                    .map_err(|e| RegistryError::Fetch(e.to_string()))?;
                if bytes.len() as u64 > cap {
                    return Err(RegistryError::TooLarge);
                }
                return Ok(bytes);
            }
            301 | 302 | 303 | 307 | 308 => {
                let location = response
                    .headers()
                    .get("location")
                    .and_then(|v| v.to_str().ok())
                    .ok_or_else(|| RegistryError::Fetch("a redirect had no address".to_owned()))?;
                current = current.join(location).map_err(|_| refused())?;
            }
            other => {
                return Err(RegistryError::Fetch(format!(
                    "the directory answered {other}"
                )));
            }
        }
    }
    Err(RegistryError::Fetch("too many redirects".to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bundle::test_support::{bundle_bytes, key};
    use crate::bundle::{publisher_key_text, sign_hash};
    use crate::model::{Grant, NotesGrant, Permissions, Source};
    use base64::Engine as _;
    use ed25519_dalek::SigningKey;
    use kanso_update::feed::sign_feed;
    use proptest::prelude::*;

    const ID: &str = "com.example.words";
    const REG: u8 = 90;
    const AUTHOR: u8 = 1;

    fn reg_key() -> SigningKey {
        key(REG)
    }

    fn entry_for(bytes: &[u8], id: &str, version: &str, seed: u8) -> Entry {
        let hash = bundle::sha256_hex(bytes);
        Entry {
            id: id.to_owned(),
            name: "Word Count".to_owned(),
            publisher: "Tester".to_owned(),
            description: "Counts words.".to_owned(),
            version: version.to_owned(),
            sha256: hash.clone(),
            size: bytes.len() as u64,
            signer: publisher_key_text(&key(seed).verifying_key()),
            author_signature: sign_hash(&hash, &key(seed)),
            min_app_version: "0.1.0".to_owned(),
            permissions: Permissions {
                commands: true,
                ..Permissions::default()
            },
            published_at: "2026-10-20T10:00:00Z".to_owned(),
            revoked: false,
        }
    }

    /// An entry that passes `check_entry`, without a real bundle behind it.
    fn plain(id: &str, version: &str) -> Entry {
        entry_for(version.as_bytes(), id, version, AUTHOR)
    }

    fn index_of(plugins: Vec<Entry>, serial: u64) -> Index {
        Index {
            schema: INDEX_SCHEMA,
            serial,
            plugins,
            revoked: Vec::new(),
        }
    }

    fn signed(index: &Index) -> (Vec<u8>, Vec<u8>) {
        let bytes = serde_json::to_vec_pretty(index).unwrap();
        let sig = sign_feed(&bytes, &reg_key()).into_bytes();
        (bytes, sig)
    }

    fn keys() -> Vec<VerifyingKey> {
        vec![reg_key().verifying_key()]
    }

    fn installed(version: &str, pinned_seed: u8) -> Installed {
        let bytes = bundle_bytes(ID, version, pinned_seed);
        let manifest = bundle::open(&bytes).unwrap().manifest;
        Installed {
            pinned_key: manifest.publisher.key.clone(),
            manifest,
            source: Source::Registry {
                sha256: "0".repeat(64),
            },
            grant: Grant {
                commands: true,
                editor: Default::default(),
                notes: NotesGrant::default(),
                fences: Default::default(),
                panel: false,
                secrets: Default::default(),
                network: Default::default(),
                ask_before_changing: true,
            },
            disabled: None,
            registered: Default::default(),
            installed_at: 0,
            last_error: None,
        }
    }

    fn no_blocks() -> BTreeSet<String> {
        BTreeSet::new()
    }

    fn app(v: &str) -> Version {
        Version::parse(v).unwrap()
    }

    #[test]
    fn the_shipped_key_list_is_well_formed_and_separate() {
        let text = include_str!("../keys/registry-keys.txt");
        let lines: Vec<&str> = text
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .collect();
        assert_eq!(
            lines.len(),
            trusted_keys().len(),
            "every line must be a key"
        );
        let update = kanso_update::trusted_keys();
        for key in trusted_keys() {
            assert!(
                !update.contains(&key),
                "a registry key is also an update key"
            );
        }
        assert_eq!(is_configured(), !lines.is_empty());
    }

    #[test]
    fn urls_are_fixed() {
        assert_eq!(
            bundle_url(ID, "1.2.0"),
            "https://raw.githubusercontent.com/iamngoni/kanso-plugins/main/directory/bundles/com.example.words/1.2.0.kansoplugin"
        );
        assert_eq!(SIGNATURE_URL, format!("{INDEX_URL}.sig"));
    }

    #[test]
    fn a_signed_index_is_taken() {
        let index = index_of(vec![plain(ID, "1.0.0")], 5);
        let (bytes, sig) = signed(&index);
        assert_eq!(check_index(&bytes, &sig, &keys(), Some(5)).unwrap(), index);
    }

    #[test]
    fn no_keys_means_not_configured() {
        let (bytes, sig) = signed(&index_of(vec![], 1));
        assert!(matches!(
            check_index(&bytes, &sig, &[], None),
            Err(RegistryError::NotConfigured)
        ));
    }

    #[test]
    fn another_key_or_a_replay_is_refused() {
        let (bytes, sig) = signed(&index_of(vec![], 3));
        let other = vec![key(91).verifying_key()];
        assert!(matches!(
            check_index(&bytes, &sig, &other, None),
            Err(RegistryError::BadSignature)
        ));
        assert!(matches!(
            check_index(&bytes, &sig, &keys(), Some(4)),
            Err(RegistryError::Replay)
        ));
    }

    #[test]
    fn bad_entries_are_refused() {
        let good = plain(ID, "1.0.0");
        let cases: Vec<(&str, Entry)> = vec![
            (
                "id",
                Entry {
                    id: "nope".into(),
                    ..good.clone()
                },
            ),
            (
                "version",
                Entry {
                    version: "1.0".into(),
                    ..good.clone()
                },
            ),
            (
                "build",
                Entry {
                    version: "1.0.0+x".into(),
                    ..good.clone()
                },
            ),
            (
                "min",
                Entry {
                    min_app_version: "x".into(),
                    ..good.clone()
                },
            ),
            (
                "hash",
                Entry {
                    sha256: "AB".repeat(32),
                    ..good.clone()
                },
            ),
            (
                "size",
                Entry {
                    size: MAX_ARCHIVE_BYTES + 1,
                    ..good.clone()
                },
            ),
            (
                "signer",
                Entry {
                    signer: "ed25519:00".into(),
                    ..good.clone()
                },
            ),
            (
                "sig",
                Entry {
                    author_signature: "AAAA".into(),
                    ..good.clone()
                },
            ),
            (
                "name",
                Entry {
                    name: "Evil\nName".into(),
                    ..good.clone()
                },
            ),
            (
                "bidi",
                Entry {
                    publisher: "a\u{202E}b".into(),
                    ..good.clone()
                },
            ),
            (
                "long",
                Entry {
                    description: "x".repeat(301),
                    ..good.clone()
                },
            ),
            (
                "empty",
                Entry {
                    name: String::new(),
                    ..good.clone()
                },
            ),
        ];
        for (what, entry) in cases {
            let (bytes, sig) = signed(&index_of(vec![entry], 1));
            assert!(
                matches!(
                    check_index(&bytes, &sig, &keys(), None),
                    Err(RegistryError::BadEntry(_))
                ),
                "{what}"
            );
        }
        let (bytes, sig) = signed(&index_of(vec![good.clone(), good], 1));
        assert!(matches!(
            check_index(&bytes, &sig, &keys(), None),
            Err(RegistryError::BadEntry(_))
        ));
    }

    #[test]
    fn bad_revocations_and_unknown_fields_are_refused() {
        let mut index = index_of(vec![], 1);
        index.revoked.push(Revocation {
            id: ID.into(),
            versions: "not a req".into(),
            reason: "x".into(),
        });
        let (bytes, sig) = signed(&index);
        assert!(check_index(&bytes, &sig, &keys(), None).is_err());
        let text = br#"{"schema":1,"serial":1,"plugins":[],"extra":1}"#;
        let sig = sign_feed(text, &reg_key()).into_bytes();
        assert!(matches!(
            check_index(text, &sig, &keys(), None),
            Err(RegistryError::Malformed(_))
        ));
    }

    #[test]
    fn a_bundle_matching_its_entry_verifies() {
        let bytes = bundle_bytes(ID, "1.0.0", AUTHOR);
        let entry = entry_for(&bytes, ID, "1.0.0", AUTHOR);
        assert_eq!(verify_bundle(&bytes, &entry).unwrap().manifest.id, ID);
    }

    #[test]
    fn a_bundle_that_differs_is_refused() {
        let bytes = bundle_bytes(ID, "1.0.0", AUTHOR);
        let entry = entry_for(&bytes, ID, "1.0.0", AUTHOR);
        let mut tampered = bytes.clone();
        let mid = tampered.len() / 2;
        tampered[mid] ^= 1;
        assert!(verify_bundle(&tampered, &entry).is_err());
        let shorter = Entry {
            size: entry.size + 1,
            ..entry.clone()
        };
        assert!(matches!(
            verify_bundle(&bytes, &shorter),
            Err(RegistryError::SizeMismatch)
        ));
        let other_hash = Entry {
            sha256: "0".repeat(64),
            ..entry.clone()
        };
        assert!(matches!(
            verify_bundle(&bytes, &other_hash),
            Err(RegistryError::HashMismatch)
        ));
        // Signed by someone other than the entry's signer.
        let forged = Entry {
            author_signature: sign_hash(&entry.sha256, &key(44)),
            ..entry
        };
        assert!(matches!(
            verify_bundle(&bytes, &forged),
            Err(RegistryError::BadAuthorSignature)
        ));
    }

    proptest! {
        /// Invariant: the real endpoint allows a URL only on that host,
        /// over HTTPS, under the directory's repository.
        #[test]
        fn prop_the_endpoint_stays_in_its_repository(owner in "[a-z-]{1,12}", repo in "[a-z-]{1,16}", rest in "[a-z0-9/._-]{0,30}") {
            let url = format!("https://raw.githubusercontent.com/{owner}/{repo}/{rest}");
            let Ok(parsed) = Url::parse(&url) else { return Ok(()) };
            let allowed = Endpoint::directory().allows(&parsed);
            if allowed {
                let under = parsed.path().strip_prefix("/iamngoni/kanso-plugins/");
                prop_assert!(under.is_some_and(|r| !r.is_empty()), "{}", url);
            }
            if format!("{owner}/{repo}") != REPOSITORY {
                prop_assert!(!allowed, "{}", url);
            }
        }

        /// Invariant: a bundle whose manifest differs from its entry in ID,
        /// version, author key or permissions never verifies, even when the
        /// hash and signature are the entry's own.
        #[test]
        fn prop_manifest_must_match_entry(which in 0usize..4) {
            let bytes = bundle_bytes(ID, "1.0.0", AUTHOR);
            let mut entry = entry_for(&bytes, ID, "1.0.0", AUTHOR);
            match which {
                0 => entry.id = "com.example.other".to_owned(),
                1 => entry.version = "1.0.1".to_owned(),
                2 => {
                    // The entry names another signer, who signed the hash.
                    entry.signer = publisher_key_text(&key(55).verifying_key());
                    entry.author_signature = sign_hash(&entry.sha256, &key(55));
                }
                _ => entry.permissions.network.insert("evil.example".to_owned()).then_some(()).unwrap(),
            }
            prop_assert!(matches!(
                verify_bundle(&bytes, &entry),
                Err(RegistryError::ManifestMismatch(_))
            ));
        }

        /// Invariant: flipping any single byte of the index or of its
        /// signature makes `check_index` fail.
        #[test]
        fn prop_a_flipped_byte_fails(pos in any::<prop::sample::Index>(), in_sig in any::<bool>(), bit in 0u8..8) {
            let index = index_of(vec![plain(ID, "1.0.0"), plain("com.example.two", "2.0.0")], 9);
            let (mut bytes, mut sig) = signed(&index);
            let target = if in_sig { &mut sig } else { &mut bytes };
            let at = pos.index(target.len());
            target[at] ^= 1 << bit;
            prop_assert!(check_index(&bytes, &sig, &keys(), None).is_err());
        }

        /// Invariant: an index with a serial below the last seen is always
        /// a replay, and one at or above never is.
        #[test]
        fn prop_serial_below_last_is_replay(serial in 0u64..1000, last in 0u64..1000) {
            let (bytes, sig) = signed(&index_of(vec![], serial));
            let got = check_index(&bytes, &sig, &keys(), Some(last));
            if serial < last {
                prop_assert!(matches!(got, Err(RegistryError::Replay)));
            } else {
                prop_assert!(got.is_ok());
            }
        }

        /// Invariant: an offer is never a version at or below the installed
        /// one, never a revoked or blocked one, never one needing a newer
        /// app, and `key_changed` is true exactly when the signer differs.
        #[test]
        fn prop_offers_are_safe(
            installed_minor in 0u64..6,
            versions in prop::collection::btree_set((0u64..8, 0u64..3), 1..8),
            revoked_at in prop::collection::vec(any::<bool>(), 8),
            needs_new in prop::collection::vec(any::<bool>(), 8),
            req in prop::sample::select(vec!["", "<1.3.0", "=1.4.0", "*", ">=1.5.0"]),
            signer_seed in 1u8..4,
            app_minor in 0u64..4,
        ) {
            let mut plugins = Vec::new();
            for (i, (minor, patch)) in versions.iter().enumerate() {
                let version = format!("1.{minor}.{patch}");
                let mut e = entry_for(version.as_bytes(), ID, &version, signer_seed);
                e.revoked = revoked_at[i];
                if needs_new[i] {
                    e.min_app_version = "9.0.0".to_owned();
                }
                plugins.push(e);
            }
            let mut index = index_of(plugins, 1);
            if !req.is_empty() {
                index.revoked.push(Revocation { id: ID.into(), versions: req.into(), reason: "r".into() });
            }
            let inst = installed(&format!("1.{installed_minor}.0"), AUTHOR);
            let current = Version::parse(&inst.manifest.version).unwrap();
            let app = Version::new(0, app_minor + 1, 0);
            if let Some(offer) = offer(&inst, &index, &app, &no_blocks()) {
                let v = Version::parse(&offer.entry.version).unwrap();
                prop_assert!(v > current);
                prop_assert!(!offer.entry.revoked);
                prop_assert!(revocation_reason(Some(&index), &no_blocks(), ID, &offer.entry.version).is_none());
                prop_assert!(Version::parse(&offer.entry.min_app_version).unwrap() <= app);
                prop_assert_eq!(offer.key_changed, offer.entry.signer != inst.pinned_key);
                // And it is the highest such version.
                for e in &index.plugins {
                    let ev = Version::parse(&e.version).unwrap();
                    let ok = ev > current
                        && Version::parse(&e.min_app_version).unwrap() <= app
                        && revocation_reason(Some(&index), &no_blocks(), ID, &e.version).is_none();
                    if ok {
                        prop_assert!(ev <= v);
                    }
                }
            }
            let blocked: BTreeSet<String> = [ID.to_owned()].into();
            prop_assert!(offer(&inst, &index, &app, &blocked).is_none());
        }

        /// Invariant: a version string that isn't semver is always revoked,
        /// whatever the index and block list say.
        #[test]
        fn prop_unreadable_versions_fail_closed(version in "[a-z0-9.+_ -]{0,12}") {
            if Version::parse(&version).is_err() {
                prop_assert!(revocation_reason(None, &no_blocks(), ID, &version).is_some());
                let index = index_of(vec![], 1);
                prop_assert!(revocation_reason(Some(&index), &no_blocks(), ID, &version).is_some());
            }
        }

        /// Invariant: `*` revokes every version of the ID, pre-releases
        /// included, and no other ID.
        #[test]
        fn prop_star_revokes_everything_of_that_id(
            major in 0u64..5, minor in 0u64..5, pre in prop::option::of("[a-z]{1,4}"),
        ) {
            let mut index = index_of(vec![], 1);
            index.revoked.push(Revocation { id: ID.into(), versions: "*".into(), reason: "bad".into() });
            let version = match pre {
                Some(p) => format!("{major}.{minor}.0-{p}"),
                None => format!("{major}.{minor}.0"),
            };
            prop_assert_eq!(
                revocation_reason(Some(&index), &no_blocks(), ID, &version),
                Some("bad".to_owned())
            );
            prop_assert!(revocation_reason(Some(&index), &no_blocks(), "com.example.other", &version).is_none());
        }
    }

    #[test]
    fn revocation_reasons_cover_every_source() {
        let mut index = index_of(vec![], 1);
        let mut gone = plain(ID, "1.2.0");
        gone.revoked = true;
        index.plugins.push(gone);
        index.revoked.push(Revocation {
            id: ID.into(),
            versions: "<1.0.0".into(),
            reason: "Stole notes.".into(),
        });
        assert_eq!(
            revocation_reason(Some(&index), &no_blocks(), ID, "0.9.0").as_deref(),
            Some("Stole notes.")
        );
        assert!(revocation_reason(Some(&index), &no_blocks(), ID, "1.2.0").is_some());
        assert!(revocation_reason(Some(&index), &no_blocks(), ID, "1.3.0").is_none());
        assert!(revocation_reason(None, &no_blocks(), ID, "0.9.0").is_none());
        let blocked: BTreeSet<String> = [ID.to_owned()].into();
        assert_eq!(
            revocation_reason(None, &blocked, ID, "1.0.0").as_deref(),
            Some("You blocked this plugin.")
        );
    }

    #[test]
    fn offers_report_permissions_and_keys() {
        let bytes = b"new";
        let mut newer = entry_for(bytes, ID, "1.1.0", 2);
        newer
            .permissions
            .network
            .insert("api.example.com".to_owned());
        let index = index_of(vec![plain(ID, "1.0.0"), newer], 1);
        let inst = installed("1.0.0", AUTHOR);
        let offer = offer(&inst, &index, &app("1.0.0"), &no_blocks()).unwrap();
        assert_eq!(offer.entry.version, "1.1.0");
        assert!(offer.key_changed);
        assert!(!offer.added.is_empty());
        assert!(offer.needs_confirmation());
        // Same version, or a developer install, gets nothing.
        let same = installed("1.1.0", 2);
        assert!(super::offer(&same, &index, &app("1.0.0"), &no_blocks()).is_none());
        let mut dev = installed("1.0.0", AUTHOR);
        dev.source = Source::Developer { path: "/x".into() };
        assert!(super::offer(&dev, &index, &app("1.0.0"), &no_blocks()).is_none());
    }

    #[test]
    fn latest_and_search_skip_revoked_and_sort() {
        let mut bad = plain("com.example.zed", "2.0.0");
        bad.revoked = true;
        let mut zed1 = plain("com.example.zed", "1.0.0");
        zed1.name = "Zed".into();
        let mut alpha = plain("com.example.alpha", "1.0.0");
        alpha.name = "alpha tool".into();
        let index = index_of(
            vec![
                bad,
                zed1,
                alpha,
                plain(ID, "1.0.0"),
                plain(ID, "1.10.0"),
                plain(ID, "1.9.0"),
            ],
            1,
        );
        assert_eq!(latest(&index, "com.example.zed").unwrap().version, "1.0.0");
        assert_eq!(latest(&index, ID).unwrap().version, "1.10.0");
        assert!(latest(&index, "com.example.none").is_none());
        let names: Vec<_> = search(&index, "").iter().map(|e| e.name.clone()).collect();
        assert_eq!(names, ["alpha tool", "Word Count", "Zed"]);
        assert_eq!(search(&index, "ALPHA").len(), 1);
        assert_eq!(search(&index, "com.example.w").len(), 1);
    }

    #[test]
    fn find_by_hash_finds_listed_bundles_only() {
        let bytes = bundle_bytes(ID, "1.0.0", AUTHOR);
        let entry = entry_for(&bytes, ID, "1.0.0", AUTHOR);
        let index = index_of(vec![entry.clone()], 1);
        assert_eq!(find_by_hash(&index, &entry.sha256), Some(&entry));
        assert!(find_by_hash(&index, &"f".repeat(64)).is_none());
    }

    #[test]
    fn the_cache_round_trips_and_rechecks() {
        let dir = tempfile::tempdir().unwrap();
        let cache = Cache::new(dir.path().join("cache"));
        assert!(cache.load(&keys(), None).is_none());
        let index = index_of(vec![plain(ID, "1.0.0")], 4);
        let (bytes, sig) = signed(&index);
        cache.save(&bytes, &sig).unwrap();
        assert_eq!(cache.load(&keys(), Some(4)), Some(index));
        assert!(cache.load(&keys(), Some(5)).is_none(), "older than seen");
        assert!(
            cache.load(&[key(91).verifying_key()], None).is_none(),
            "key rotated out"
        );
        // A cache edited on disk no longer verifies.
        let mut edited = bytes.clone();
        edited[0] ^= 1;
        std::fs::write(dir.path().join("cache/index.json"), edited).unwrap();
        assert!(cache.load(&keys(), None).is_none());
    }

    #[test]
    fn the_endpoint_allows_only_its_own_origin() {
        let real = Endpoint::directory();
        let ok = |e: &Endpoint, u: &str| e.allows(&Url::parse(u).unwrap());
        assert!(ok(&real, INDEX_URL));
        assert!(ok(&real, &bundle_url(ID, "1.0.0")));
        for bad in [
            "http://raw.githubusercontent.com/iamngoni/kanso-plugins/main/directory/index.json",
            "https://raw.githubusercontent.com:8443/iamngoni/kanso-plugins/main/directory/index.json",
            "https://evil.example/iamngoni/kanso-plugins/main/directory/index.json",
            "https://raw.githubusercontent.com.evil.example/iamngoni/kanso-plugins/main/index.json",
            "https://user@raw.githubusercontent.com/iamngoni/kanso-plugins/main/index.json",
            "https://raw.githubusercontent.com/someone/kanso-plugins/main/directory/index.json",
            "https://raw.githubusercontent.com/iamngoni/kanso-plugins-evil/main/index.json",
            "https://raw.githubusercontent.com/iamngoni/kanso-plugins/../other/main/index.json",
            "https://raw.githubusercontent.com/iamngoni/kanso-plugins/%2e%2e/other/index.json",
            "https://raw.githubusercontent.com/iamngoni/kanso-plugins/",
            "https://raw.githubusercontent.com/iamngoni/kanso-plugins/main/index.json?x=1",
            "http://127.0.0.1/index.json",
        ] {
            assert!(!ok(&real, bad), "{bad}");
        }
        assert!(Endpoint::loopback("https://127.0.0.1:1").is_err());
        assert!(Endpoint::loopback("http://example.com:1").is_err());
        let local = Endpoint::loopback("http://127.0.0.1:4000").unwrap();
        assert!(ok(&local, "http://127.0.0.1:4000/index.json"));
        assert!(!ok(&local, "http://127.0.0.1:4001/index.json"));
        assert!(!ok(&local, INDEX_URL));
    }

    #[test]
    fn the_whole_trip_over_loopback() {
        use std::sync::Arc;
        let bundle = bundle_bytes(ID, "1.0.0", AUTHOR);
        let entry = entry_for(&bundle, ID, "1.0.0", AUTHOR);
        let (index_bytes, sig) = signed(&index_of(vec![entry.clone()], 2));
        let server = Arc::new(tiny_http::Server::http("127.0.0.1:0").unwrap());
        let port = server.server_addr().to_ip().unwrap().port();
        let bundle_path = format!("/bundles/{ID}/1.0.0.kansoplugin");
        let serving = {
            let server = Arc::clone(&server);
            let (index_bytes, sig, bundle) = (index_bytes.clone(), sig.clone(), bundle.clone());
            std::thread::spawn(move || {
                for request in server.incoming_requests().take(5) {
                    let body = match request.url() {
                        "/index.json" => Some(index_bytes.clone()),
                        "/index.json.sig" => Some(sig.clone()),
                        u if u == bundle_path => Some(bundle.clone()),
                        _ => None,
                    };
                    let _ = match body {
                        Some(b) => request.respond(tiny_http::Response::from_data(b)),
                        None => request.respond(tiny_http::Response::empty(404)),
                    };
                }
            })
        };
        let endpoint = Endpoint::loopback(&format!("http://127.0.0.1:{port}")).unwrap();
        let agent = agent();
        let (got_index, got_sig) = fetch_index_from(&agent, &endpoint).unwrap();
        let index = check_index(&got_index, &got_sig, &keys(), None).unwrap();
        let listed = latest(&index, ID).unwrap();
        let fetched = fetch_bundle_from(&agent, &endpoint, listed).unwrap();
        let opened = verify_bundle(&fetched, listed).unwrap();
        assert_eq!(opened.sha256, entry.sha256);
        // An entry that doesn't exist is a plain failure, not a panic.
        let missing = Entry {
            version: "9.9.9".into(),
            ..entry.clone()
        };
        assert!(fetch_bundle_from(&agent, &endpoint, &missing).is_err());
        // A bigger body than the entry says is cut off.
        let small = Entry { size: 10, ..entry };
        assert!(matches!(
            fetch_bundle_from(&agent, &endpoint, &small),
            Err(RegistryError::TooLarge)
        ));
        let _ = serving.join();
    }

    #[test]
    fn a_client_that_follows_redirects_is_not_used() {
        let eager: ureq::Agent = ureq::Agent::config_builder().build().into();
        let endpoint = Endpoint::loopback("http://127.0.0.1:1").unwrap();
        assert!(matches!(
            fetch_index_from(&eager, &endpoint),
            Err(RegistryError::Fetch(_))
        ));
    }

    #[test]
    fn base64_signature_helper_is_strict() {
        assert_eq!(
            base64_len(&base64::engine::general_purpose::STANDARD.encode([0u8; 64])),
            Some(64)
        );
        assert_eq!(base64_len("!!"), None);
    }
}
