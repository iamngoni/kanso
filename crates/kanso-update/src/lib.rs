//! Kanso's updater, apart from the window (docs/RELEASING.md › How updates
//! work).
//!
//! Every release carries `latest.json`, the [`Feed`], and `latest.json.sig`,
//! an Ed25519 signature over the feed's exact bytes. The app fetches both
//! from the newest *published* release, checks the signature against the
//! keys compiled in ([`trusted_keys`]), and takes the update only when it is
//! newer than itself, has a build for this system, and points at GitHub over
//! HTTPS ([`UrlPolicy`]). It then downloads the archive, checks its size and
//! SHA-256 against the signed feed, unpacks it next to the install
//! ([`stage`]) and, on restart or quit, swaps it in ([`install`]).
//!
//! Nothing here touches the window; the app drives it from a thread.

pub mod feed;
pub mod fetch;
pub mod install;
pub mod notes;
pub mod notice_gate;
pub mod platform;
pub mod policy;
pub mod stage;

pub use ed25519_dalek::VerifyingKey;
pub use feed::{
    Asset, Check, Feed, FeedError, SCHEMA, Update, check_feed, parse_public_key, sign_feed,
    trusted_keys, verify_signature, verify_signature_with_cap,
};
pub use fetch::{FetchError, HttpTransport, Transport};
pub use install::{Install, LocateOptions, NotInstallable};
pub use platform::{ArchiveKind, Platform};
pub use policy::{PolicyError, UrlPolicy};
pub use stage::{Pending, StageError};

/// Where the app looks for the newest published release's feed. GitHub
/// answers `releases/latest` with published releases only, never drafts.
pub const FEED_URL: &str = "https://github.com/iamngoni/kanso/releases/latest/download/latest.json";

/// The signature beside a feed at `feed_url`.
pub fn signature_url(feed_url: &str) -> String {
    format!("{feed_url}.sig")
}

/// The GitHub repository releases come from, as `owner/name`.
pub const REPOSITORY: &str = "iamngoni/kanso";

/// Why a check for updates didn't end in an answer.
#[derive(Debug, thiserror::Error)]
pub enum CheckError {
    #[error(transparent)]
    Fetch(#[from] FetchError),
    #[error(transparent)]
    Feed(#[from] FeedError),
}

/// Fetches the feed at `feed_url` and its signature, and checks them
/// ([`check_feed`]).
pub fn check(
    transport: &dyn Transport,
    policy: &UrlPolicy,
    feed_url: &str,
    keys: &[ed25519_dalek::VerifyingKey],
    current: &semver::Version,
    platform: Platform,
) -> Result<Check, CheckError> {
    if keys.is_empty() {
        return Err(FeedError::NoTrustedKeys.into());
    }
    let feed = fetch::get_bytes(transport, policy, feed_url, feed::MAX_FEED_BYTES)?;
    let signature = fetch::get_bytes(
        transport,
        policy,
        &signature_url(feed_url),
        feed::MAX_SIGNATURE_BYTES,
    )?;
    Ok(check_feed(
        &feed, &signature, keys, current, platform, policy,
    )?)
}
