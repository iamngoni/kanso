//! The gallery's fetching: the transport that trusts the phone's certificate
//! store, and the fetch-then-verify of the signed catalog, exactly as the
//! desktop does it (`kanso-app/src/settings_gallery.rs`).

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use kanso_gallery::{GalleryPolicy, Index, check_index};
use kanso_update::fetch::{Fetched, Response};
use kanso_update::{FetchError, Transport, VerifyingKey};
use semver::Version;

/// Where the gallery reads from and what it trusts: the real thing in the
/// app, a scripted server and a test key in the tests.
#[derive(Clone)]
pub(crate) struct Source {
    pub(crate) transport: Arc<dyn Transport>,
    pub(crate) keys: Vec<VerifyingKey>,
    pub(crate) policy: GalleryPolicy,
    pub(crate) index_url: String,
}

impl Source {
    /// What shipped builds use: the themes keys compiled into the gallery
    /// crate, its fixed repository and the phone's certificate store.
    pub(crate) fn production() -> Source {
        Source {
            transport: Arc::new(PhoneTransport::new()),
            keys: kanso_gallery::trusted_keys(),
            policy: kanso_gallery::policy::policy(),
            index_url: kanso_gallery::policy::index_url(),
        }
    }

    pub(crate) fn signature_url(&self) -> String {
        format!("{}.sig", self.index_url)
    }
}

/// The updater's transport (blocking `ureq`, no redirects of its own, so the
/// gallery's address rules see every hop) with the certificate choice
/// `kanso-tls` makes: the phone's store on iOS and Android.
pub(crate) struct PhoneTransport {
    agent: ureq::Agent,
}

impl PhoneTransport {
    pub(crate) fn new() -> PhoneTransport {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .max_redirects(0)
            .timeout_connect(Some(Duration::from_secs(20)))
            .timeout_recv_body(Some(Duration::from_secs(60)))
            .user_agent(concat!("Kanso/", env!("CARGO_PKG_VERSION")))
            .tls_config(kanso_tls::ureq_tls())
            .build()
            .into();
        PhoneTransport { agent }
    }
}

impl Transport for PhoneTransport {
    fn get(&self, url: &str) -> Result<Response, FetchError> {
        self.get_if_none_match(url, None)
    }

    fn get_if_none_match(&self, url: &str, etag: Option<&str>) -> Result<Response, FetchError> {
        // Before the handshake: an Android app that hasn't handed over its
        // Context gets an error, not a panic inside the verifier.
        kanso_tls::ready_for(url).map_err(|_| FetchError::Offline)?;
        let mut request = self.agent.get(url);
        if let Some(etag) = etag {
            request = request.header("If-None-Match", etag);
        }
        let response = request.call().map_err(|err| match err {
            ureq::Error::Io(io) => FetchError::Io(io),
            ureq::Error::BadUri(_) => FetchError::Policy(kanso_update::PolicyError::Invalid),
            // Unreachable hosts, timeouts and TLS failures all read the same
            // to a person: try again later.
            _ => FetchError::Offline,
        })?;
        let status = response.status().as_u16();
        let header = |name: &str| {
            response
                .headers()
                .get(name)
                .and_then(|value| value.to_str().ok())
                .map(str::to_owned)
        };
        Ok(Response {
            status,
            location: header("location"),
            rate_limit_remaining: header("x-ratelimit-remaining").and_then(|v| v.parse().ok()),
            etag: header("etag"),
            body: Box::new(response.into_body().into_reader()),
        })
    }
}

/// Why a check of the catalog did not give a catalog.
#[derive(Debug)]
pub(crate) enum CheckError {
    Fetch(FetchError),
    Index(kanso_gallery::IndexError),
}

impl std::fmt::Display for CheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CheckError::Fetch(err) => write!(f, "{err}"),
            CheckError::Index(err) => write!(f, "{err}"),
        }
    }
}

impl CheckError {
    /// The network, not the catalog, was the problem: a cached catalog may
    /// stand in for it. A catalog that failed a check is never stood in for.
    pub(crate) fn is_offline(&self) -> bool {
        matches!(
            self,
            CheckError::Fetch(
                FetchError::Offline
                    | FetchError::RateLimited
                    | FetchError::TooManyRedirects
                    | FetchError::Io(_)
                    | FetchError::Status(500..=599)
            )
        )
    }
}

/// A verified catalog: its bytes, its signature, the `ETag` that came with
/// it, and what they parse to.
pub(crate) struct Checked {
    pub(crate) bytes: Vec<u8>,
    pub(crate) signature: Vec<u8>,
    pub(crate) etag: Option<String>,
    pub(crate) index: Index,
}

/// Fetches the catalog and signature, verifies them, and uses the cached
/// copy only after verifying it again when the server answers `304`. A cache
/// that no longer verifies is not a failure: the etag is dropped and the
/// index fetched unconditionally.
pub(crate) fn fetch_and_check(
    source: &Source,
    etag: Option<&str>,
    root: &Path,
    floor: &kanso_gallery::Floor,
    current: &Version,
) -> Result<Checked, CheckError> {
    let transport = source.transport.as_ref();
    let index = kanso_update::fetch::get_bytes_if_none_match(
        transport,
        &source.policy,
        &source.index_url,
        kanso_gallery::MAX_INDEX_BYTES,
        etag,
    )
    .map_err(CheckError::Fetch)?;
    match index {
        Fetched::Body { bytes, etag } => {
            let signature = fetch_signature(source)?;
            let index = check_index(&bytes, &signature, &source.keys, floor, current)
                .map_err(CheckError::Index)?;
            Ok(Checked {
                bytes,
                signature,
                etag,
                index,
            })
        }
        Fetched::NotModified => {
            if let Some((bytes, signature, index)) = cached(root, source, floor, current) {
                return Ok(Checked {
                    bytes,
                    signature,
                    etag: etag.map(str::to_owned),
                    index,
                });
            }
            // The cache is gone or no longer verifies; fetch it fresh with no
            // conditional header so the server cannot answer 304 again.
            let bytes = kanso_update::fetch::get_bytes(
                transport,
                &source.policy,
                &source.index_url,
                kanso_gallery::MAX_INDEX_BYTES,
            )
            .map_err(CheckError::Fetch)?;
            let signature = fetch_signature(source)?;
            let index = check_index(&bytes, &signature, &source.keys, floor, current)
                .map_err(CheckError::Index)?;
            Ok(Checked {
                bytes,
                signature,
                etag: None,
                index,
            })
        }
    }
}

fn fetch_signature(source: &Source) -> Result<Vec<u8>, CheckError> {
    kanso_update::fetch::get_bytes(
        source.transport.as_ref(),
        &source.policy,
        &source.signature_url(),
        kanso_gallery::MAX_SIGNATURE_BYTES,
    )
    .map_err(CheckError::Fetch)
}

/// The last accepted catalog on disk, if it still verifies against the keys
/// and the floor.
pub(crate) fn cached(
    root: &Path,
    source: &Source,
    floor: &kanso_gallery::Floor,
    current: &Version,
) -> Option<(Vec<u8>, Vec<u8>, Index)> {
    let bytes = std::fs::read(root.join("index.json")).ok()?;
    let signature = std::fs::read(root.join("index.json.sig")).ok()?;
    let index = check_index(&bytes, &signature, &source.keys, floor, current).ok()?;
    Some((bytes, signature, index))
}
