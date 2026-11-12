//! A release's What's new video and the notes of releases someone skipped
//! (docs/RELEASING.md › What's new video).
//!
//! The video is an animated WebP: GPUI's `img` decodes and loops its frames
//! (it has no video player), and Flutter's `Image` plays the same file on the
//! phones, so one rendition serves both. It has no sound track, so it is
//! always muted. The feed names it as an [`Asset`] (URL, SHA-256, size),
//! covered by the feed's signature; it is fetched under the same
//! [`UrlPolicy`] as an update, each redirect hop included, checked against
//! the size and SHA-256 before it is kept, and cached in the app's data
//! folder by version.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use ed25519_dalek::VerifyingKey;
use semver::Version;
use sha2::{Digest, Sha256};

use crate::feed::{self, Asset, Earlier, Feed, FeedError};
use crate::fetch::{self, FetchError, Transport};
use crate::policy::{PolicyError, UrlPolicy};
use crate::{CheckError, REPOSITORY};

/// The largest video the app takes, whatever the feed says.
pub const MAX_MEDIA_BYTES: u64 = 8 * 1024 * 1024;
/// The video's file extension.
pub const MEDIA_EXTENSION: &str = "webp";
/// How many earlier releases' notes a feed carries, at most.
pub const MAX_EARLIER: usize = 12;
/// How much of the feed the earlier notes may take, at most.
pub const MAX_EARLIER_BYTES: usize = 96 * 1024;
/// The cache folder's name inside the data folder.
pub const CACHE_DIR: &str = "whats-new";

/// The release asset's name: `Kanso-<v>-whats-new.webp`.
pub fn media_name(version: &str) -> String {
    format!("Kanso-{version}-whats-new.{MEDIA_EXTENSION}")
}

/// Why the video wasn't kept.
#[derive(Debug, thiserror::Error)]
pub enum MediaError {
    #[error(transparent)]
    Fetch(#[from] FetchError),
    #[error("the video's address isn't allowed: {0}")]
    BadUrl(PolicyError),
    #[error("the video doesn't match its checksum or size")]
    Mismatch,
    #[error("the video couldn't be saved: {0}")]
    Io(#[from] io::Error),
}

/// Whether the feed's video may be fetched: one of this repository's
/// release assets, a `.webp`, with a lowercase SHA-256 and a size up to
/// [`MAX_MEDIA_BYTES`].
pub fn media_allowed(media: &Asset, policy: &UrlPolicy) -> Result<(), MediaError> {
    let url = policy.check_asset(&media.url).map_err(MediaError::BadUrl)?;
    if !url
        .path()
        .to_ascii_lowercase()
        .ends_with(&format!(".{MEDIA_EXTENSION}"))
    {
        return Err(MediaError::BadUrl(PolicyError::WrongPath));
    }
    if !feed::is_sha256_hex(&media.sha256) || media.size == 0 || media.size > MAX_MEDIA_BYTES {
        return Err(MediaError::Mismatch);
    }
    Ok(())
}

/// The feed's video when it may be fetched; otherwise none, and the
/// release's notes show without it.
pub fn checked_media(media: Option<Asset>, policy: &UrlPolicy) -> Option<Asset> {
    media.filter(|m| media_allowed(m, policy).is_ok())
}

/// The earlier releases a feed may carry: valid versions only, newest
/// first, no more than [`MAX_EARLIER`] and [`MAX_EARLIER_BYTES`].
pub fn checked_earlier(earlier: Vec<Earlier>) -> Vec<Earlier> {
    let mut list: Vec<(Version, Earlier)> = earlier
        .into_iter()
        .filter_map(|e| Some((Version::parse(&e.version).ok()?, e)))
        .collect();
    list.sort_by(|a, b| b.0.cmp(&a.0));
    list.dedup_by(|a, b| a.0 == b.0);
    let mut bytes = 0;
    list.into_iter()
        .take(MAX_EARLIER)
        .take_while(|(_, e)| {
            bytes += e.notes.len();
            bytes <= MAX_EARLIER_BYTES
        })
        .map(|(_, e)| e)
        .collect()
}

/// The earlier releases newer than `after` (the version run before the
/// update) and older than `before` (the one being shown), newest first.
/// With no `after`, none: a first install skipped nothing.
pub fn since<'a>(
    earlier: &'a [Earlier],
    after: Option<&Version>,
    before: &str,
) -> Vec<&'a Earlier> {
    let (Some(after), Ok(before)) = (after, Version::parse(before)) else {
        return Vec::new();
    };
    let mut out: Vec<(Version, &Earlier)> = earlier
        .iter()
        .filter_map(|e| Some((Version::parse(&e.version).ok()?, e)))
        .filter(|(v, _)| v > after && *v < before)
        .collect();
    out.sort_by(|a, b| b.0.cmp(&a.0));
    out.into_iter().map(|(_, e)| e).collect()
}

/// A release's own feed, by tag (not `latest`): what a phone, or a desktop
/// that just updated, reads its What's new from.
pub fn release_feed_url(version: &str) -> String {
    format!("https://github.com/{REPOSITORY}/releases/download/v{version}/latest.json")
}

/// One release's notes and video, from its own signed feed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Release {
    pub version: String,
    pub pub_date: String,
    pub release_url: String,
    pub notes: String,
    pub whats_new: Option<Asset>,
    pub earlier: Vec<Earlier>,
}

/// Checks a release's own feed: the signature first, then that it is
/// `version`'s, with the video and earlier notes checked as
/// [`crate::check_feed`] does. A newer schema still has notes worth
/// reading, but nothing is taken from one.
pub fn check_release(
    bytes: &[u8],
    signature: &[u8],
    keys: &[VerifyingKey],
    version: &Version,
    policy: &UrlPolicy,
) -> Result<Release, FeedError> {
    feed::verify_signature(bytes, signature, keys)?;
    let feed: Feed = serde_json::from_slice(bytes).map_err(|_| FeedError::Malformed)?;
    if feed.schema != feed::SCHEMA {
        return Err(if feed.schema > feed::SCHEMA {
            FeedError::NewerSchema
        } else {
            FeedError::Malformed
        });
    }
    if Version::parse(&feed.version).map_err(|_| FeedError::BadVersion)? != *version {
        return Err(FeedError::BadVersion);
    }
    policy
        .check_release_page(&feed.release_url)
        .map_err(FeedError::BadUrl)?;
    Ok(Release {
        version: feed.version,
        pub_date: feed.pub_date,
        release_url: feed.release_url,
        notes: feed.notes,
        whats_new: checked_media(feed.whats_new, policy),
        earlier: checked_earlier(feed.earlier),
    })
}

/// Fetches `version`'s own feed and its signature and checks them.
pub fn fetch_release(
    transport: &dyn Transport,
    policy: &UrlPolicy,
    keys: &[VerifyingKey],
    version: &Version,
) -> Result<Release, CheckError> {
    if keys.is_empty() {
        return Err(FeedError::NoTrustedKeys.into());
    }
    let url = release_feed_url(&version.to_string());
    let bytes = fetch::get_bytes(transport, policy, &url, feed::MAX_FEED_BYTES)?;
    let signature = fetch::get_bytes(
        transport,
        policy,
        &crate::signature_url(&url),
        feed::MAX_SIGNATURE_BYTES,
    )?;
    Ok(check_release(&bytes, &signature, keys, version, policy)?)
}

/// Where `version`'s video is cached in `dir`.
pub fn cache_path(dir: &Path, version: &str) -> PathBuf {
    dir.join(media_name(version))
}

/// Whether `bytes` are `media`'s: its size and SHA-256.
pub fn matches(bytes: &[u8], media: &Asset) -> bool {
    bytes.len() as u64 == media.size && sha256_hex(bytes) == media.sha256
}

/// The cached video for `version` when it is still `media`'s bytes. A file
/// that doesn't match is removed.
pub fn cached(dir: &Path, version: &str, media: &Asset) -> Option<PathBuf> {
    let path = cache_path(dir, version);
    let meta = fs::symlink_metadata(&path).ok()?;
    if !meta.is_file() || meta.len() != media.size {
        fs::remove_file(&path).ok();
        return None;
    }
    let bytes = fs::read(&path).ok()?;
    if matches(&bytes, media) {
        Some(path)
    } else {
        fs::remove_file(&path).ok();
        None
    }
}

/// `version`'s video in `dir`: the cached copy when it matches, otherwise
/// fetched under `policy` (every redirect hop checked), checked against the
/// size and SHA-256, and written in place by a rename. Other versions'
/// videos are removed, so the folder holds one.
pub fn fetch_media(
    transport: &dyn Transport,
    policy: &UrlPolicy,
    dir: &Path,
    version: &str,
    media: &Asset,
) -> Result<PathBuf, MediaError> {
    media_allowed(media, policy)?;
    if let Some(path) = cached(dir, version, media) {
        return Ok(path);
    }
    let bytes = fetch::get_bytes(transport, policy, &media.url, media.size)?;
    if !matches(&bytes, media) {
        return Err(MediaError::Mismatch);
    }
    fs::create_dir_all(dir)?;
    let path = cache_path(dir, version);
    let partial = dir.join(format!(".{}.partial", media_name(version)));
    fs::write(&partial, &bytes)?;
    fs::rename(&partial, &path)?;
    prune(dir, version);
    Ok(path)
}

/// Removes every cached video but `keep`'s.
pub fn prune(dir: &Path, keep: &str) {
    let keep = media_name(keep);
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name != keep && name.starts_with("Kanso-") && name.ends_with(".webp") {
            fs::remove_file(entry.path()).ok();
        }
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Whether `bytes` start as a WebP file does (`RIFF....WEBP`).
pub fn looks_like_webp(bytes: &[u8]) -> bool {
    bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch::fake::{Answer, FakeTransport};
    use crate::platform::Platform;
    use ed25519_dalek::SigningKey;
    use proptest::prelude::*;
    use std::collections::BTreeMap;

    const VIDEO_URL: &str =
        "https://github.com/iamngoni/kanso/releases/download/v0.10.7/Kanso-0.10.7-whats-new.webp";
    const STORAGE: &str = "https://release-assets.githubusercontent.com/x/whats-new.webp";

    fn video() -> Vec<u8> {
        let mut bytes = b"RIFF\0\0\0\0WEBPVP8X".to_vec();
        bytes.extend((0..2000u32).map(|i| (i % 251) as u8));
        bytes
    }

    fn media(bytes: &[u8]) -> Asset {
        Asset {
            url: VIDEO_URL.into(),
            sha256: sha256_hex(bytes),
            size: bytes.len() as u64,
        }
    }

    fn key() -> SigningKey {
        SigningKey::from_bytes(&[7; 32])
    }

    fn feed(version: &str, whats_new: Option<Asset>, earlier: Vec<Earlier>) -> Feed {
        let asset = Asset {
            url: format!(
                "https://github.com/iamngoni/kanso/releases/download/v{version}/{}",
                Platform::MacosArm64.asset_name(version)
            ),
            sha256: "ab".repeat(32),
            size: 10,
        };
        Feed {
            schema: feed::SCHEMA,
            version: version.into(),
            pub_date: "2026-11-12T10:00:00Z".into(),
            release_url: format!("https://github.com/iamngoni/kanso/releases/tag/v{version}"),
            notes: "### Fixed\n\n- A crash".into(),
            platforms: Platform::ALL
                .into_iter()
                .map(|p| (p.key().to_owned(), asset.clone()))
                .collect(),
            whats_new,
            earlier,
        }
    }

    fn earlier(version: &str) -> Earlier {
        Earlier {
            version: version.into(),
            pub_date: String::new(),
            notes: format!("### Fixed\n\n- In {version}"),
        }
    }

    fn signed(feed: &Feed) -> (Vec<u8>, Vec<u8>) {
        let bytes = serde_json::to_vec_pretty(feed).unwrap();
        let sig = feed::sign_feed(&bytes, &key()).into_bytes();
        (bytes, sig)
    }

    fn check(feed: &Feed, current: &str) -> crate::Check {
        let (bytes, sig) = signed(feed);
        crate::check_feed(
            &bytes,
            &sig,
            &[key().verifying_key()],
            &Version::parse(current).unwrap(),
            Platform::MacosArm64,
            &UrlPolicy::RELEASE,
        )
        .unwrap()
    }

    /// The feed as every Kanso up to 0.10.6 reads it: no `whats_new`, no
    /// `earlier`, and serde's default of skipping fields it doesn't know.
    #[derive(serde::Deserialize)]
    #[allow(dead_code)]
    struct OldFeed {
        schema: u32,
        version: String,
        pub_date: String,
        release_url: String,
        notes: String,
        platforms: BTreeMap<String, Asset>,
    }

    #[test]
    fn a_feed_without_the_video_reads_as_before() {
        let f = feed("0.10.7", None, Vec::new());
        let json = serde_json::to_string(&f).unwrap();
        assert!(!json.contains("whats_new") && !json.contains("earlier"));
        let crate::Check::Available(update) = check(&f, "0.10.6") else {
            panic!("expected an update");
        };
        assert_eq!(update.whats_new, None);
        assert!(update.earlier.is_empty());
    }

    #[test]
    fn a_feed_with_the_video_offers_it() {
        let m = media(&video());
        let f = feed("0.10.7", Some(m.clone()), vec![earlier("0.10.6")]);
        let crate::Check::Available(update) = check(&f, "0.10.5") else {
            panic!("expected an update");
        };
        assert_eq!(update.whats_new, Some(m));
        assert_eq!(update.earlier, vec![earlier("0.10.6")]);
    }

    #[test]
    fn older_parsers_still_accept_the_new_feed() {
        let f = feed("0.10.7", Some(media(&video())), vec![earlier("0.10.6")]);
        let (bytes, sig) = signed(&f);
        // The signature covers the new fields: it still verifies, and the
        // old shape parses past them.
        feed::verify_signature(&bytes, &sig, &[key().verifying_key()]).unwrap();
        let old: OldFeed = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(old.version, "0.10.7");
        assert_eq!(old.platforms.len(), Platform::ALL.len());
    }

    #[test]
    fn the_videos_address_is_held_to_the_allow_list() {
        let bytes = video();
        for (url, err) in [
            (
                "https://evil.example/Kanso-0.10.7-whats-new.webp",
                PolicyError::WrongHost,
            ),
            (
                "http://github.com/iamngoni/kanso/releases/download/v0.10.7/a.webp",
                PolicyError::NotHttps,
            ),
            (
                "https://github.com/someone/else/releases/download/v1/a.webp",
                PolicyError::WrongPath,
            ),
            (
                "https://github.com/iamngoni/kanso/releases/download/v0.10.7/a.mp4",
                PolicyError::WrongPath,
            ),
        ] {
            let mut m = media(&bytes);
            m.url = url.into();
            assert!(
                matches!(media_allowed(&m, &UrlPolicy::RELEASE), Err(MediaError::BadUrl(e)) if e == err),
                "{url}"
            );
            // The update itself is still offered, without the video.
            let crate::Check::Available(update) = check(&feed("0.10.7", Some(m), vec![]), "0.10.6")
            else {
                panic!("expected an update");
            };
            assert_eq!(update.whats_new, None, "{url}");
        }
        let mut huge = media(&bytes);
        huge.size = MAX_MEDIA_BYTES + 1;
        assert!(matches!(
            media_allowed(&huge, &UrlPolicy::RELEASE),
            Err(MediaError::Mismatch)
        ));
    }

    #[test]
    fn the_video_is_fetched_checked_and_cached() {
        let dir = tempfile::tempdir().unwrap();
        let bytes = video();
        let m = media(&bytes);
        let t = FakeTransport::default()
            .with(VIDEO_URL, Answer::Redirect(STORAGE.into()))
            .with(STORAGE, Answer::Body(bytes.clone()));
        std::fs::create_dir_all(dir.path()).unwrap();
        std::fs::write(cache_path(dir.path(), "0.10.6"), b"old").unwrap();
        let path = fetch_media(&t, &UrlPolicy::RELEASE, dir.path(), "0.10.7", &m).unwrap();
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        assert!(
            !cache_path(dir.path(), "0.10.6").exists(),
            "older videos go"
        );
        // Cached: no second download.
        let asked = t.asked.lock().unwrap().len();
        fetch_media(&t, &UrlPolicy::RELEASE, dir.path(), "0.10.7", &m).unwrap();
        assert_eq!(t.asked.lock().unwrap().len(), asked);
        assert_eq!(cached(dir.path(), "0.10.7", &m), Some(path));
    }

    #[test]
    fn a_hash_or_size_mismatch_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let bytes = video();
        let mut tampered = bytes.clone();
        tampered[100] ^= 1;
        let t = FakeTransport::default().with(VIDEO_URL, Answer::Body(tampered.clone()));
        let err = fetch_media(
            &t,
            &UrlPolicy::RELEASE,
            dir.path(),
            "0.10.7",
            &media(&bytes),
        );
        assert!(matches!(err, Err(MediaError::Mismatch)));
        assert!(!cache_path(dir.path(), "0.10.7").exists());

        let longer = [bytes.clone(), vec![0; 10]].concat();
        let t = FakeTransport::default().with(VIDEO_URL, Answer::Body(longer));
        let err = fetch_media(
            &t,
            &UrlPolicy::RELEASE,
            dir.path(),
            "0.10.7",
            &media(&bytes),
        );
        assert!(matches!(err, Err(MediaError::Fetch(FetchError::TooLarge))));

        // A cached file that changed on disk is dropped, not shown.
        std::fs::write(cache_path(dir.path(), "0.10.7"), &tampered).unwrap();
        assert_eq!(cached(dir.path(), "0.10.7", &media(&bytes)), None);
        assert!(!cache_path(dir.path(), "0.10.7").exists());
    }

    #[test]
    fn a_redirect_off_github_is_never_followed() {
        let dir = tempfile::tempdir().unwrap();
        let bytes = video();
        let t = FakeTransport::default()
            .with(
                VIDEO_URL,
                Answer::Redirect("https://evil.example/v.webp".into()),
            )
            .with("https://evil.example/v.webp", Answer::Body(bytes.clone()));
        let err = fetch_media(
            &t,
            &UrlPolicy::RELEASE,
            dir.path(),
            "0.10.7",
            &media(&bytes),
        );
        assert!(matches!(
            err,
            Err(MediaError::Fetch(FetchError::Policy(
                PolicyError::WrongHost
            )))
        ));
        assert_eq!(t.asked.lock().unwrap().as_slice(), [VIDEO_URL]);
    }

    #[test]
    fn a_releases_own_feed_is_read_by_tag() {
        let m = media(&video());
        let f = feed("0.10.7", Some(m.clone()), vec![earlier("0.10.6")]);
        let (bytes, sig) = signed(&f);
        let url = release_feed_url("0.10.7");
        let t = FakeTransport::default()
            .with(&url, Answer::Body(bytes.clone()))
            .with(&crate::signature_url(&url), Answer::Body(sig.clone()));
        let v = Version::new(0, 10, 7);
        let release = fetch_release(&t, &UrlPolicy::RELEASE, &[key().verifying_key()], &v).unwrap();
        assert_eq!(release.whats_new, Some(m));
        assert_eq!(release.earlier.len(), 1);
        // Another release's feed isn't taken for this one.
        assert!(matches!(
            check_release(
                &bytes,
                &sig,
                &[key().verifying_key()],
                &Version::new(0, 10, 8),
                &UrlPolicy::RELEASE
            ),
            Err(FeedError::BadVersion)
        ));
        // Nor an unsigned one.
        assert!(matches!(
            check_release(
                &bytes,
                b"",
                &[key().verifying_key()],
                &v,
                &UrlPolicy::RELEASE
            ),
            Err(FeedError::BadSignature)
        ));
    }

    #[test]
    fn webp_files_are_recognised() {
        assert!(looks_like_webp(&video()));
        assert!(!looks_like_webp(b"GIF89a......"));
        assert!(!looks_like_webp(b"RIFF"));
    }

    fn version() -> impl Strategy<Value = Version> {
        (0u64..2, 0u64..12, 0u64..12).prop_map(|(a, b, c)| Version::new(a, b, c))
    }

    proptest! {
        #![proptest_config(ProptestConfig { rng_seed: proptest::test_runner::RngSeed::Fixed(0x6b616e736f), ..ProptestConfig::default() })]

        /// "Also since your last update" lists exactly the releases between
        /// the one run before and the one shown, newest first.
        #[test]
        fn prop_since_is_the_skipped_releases(
            list in prop::collection::vec(version(), 0..20),
            after in prop::option::of(version()),
            before in version(),
        ) {
            let earlier: Vec<Earlier> = list.iter().map(|v| super::tests::earlier(&v.to_string())).collect();
            let got = since(&earlier, after.as_ref(), &before.to_string());
            let versions: Vec<Version> = got.iter().map(|e| Version::parse(&e.version).unwrap()).collect();
            for v in &versions {
                prop_assert!(after.as_ref().is_some_and(|a| v > a) && *v < before);
            }
            prop_assert!(versions.windows(2).all(|w| w[0] >= w[1]));
            let expected = list.iter().filter(|v| after.as_ref().is_some_and(|a| *v > a) && **v < before).count();
            prop_assert_eq!(versions.len(), expected);
        }

        /// Checked earlier notes are valid, unique, newest first and capped.
        #[test]
        fn prop_checked_earlier_is_bounded(list in prop::collection::vec((version(), any::<bool>()), 0..30)) {
            let input: Vec<Earlier> = list
                .iter()
                .map(|(v, ok)| if *ok { earlier(&v.to_string()) } else { Earlier { version: "nope".into(), ..earlier("0.0.0") } })
                .collect();
            let out = checked_earlier(input);
            prop_assert!(out.len() <= MAX_EARLIER);
            let vs: Vec<Version> = out.iter().map(|e| Version::parse(&e.version).unwrap()).collect();
            prop_assert!(vs.windows(2).all(|w| w[0] > w[1]));
        }

        /// Any bytes but the video's are refused.
        #[test]
        fn prop_only_the_signed_bytes_match(at in 0usize..2016, bit in 0u8..8) {
            let bytes = video();
            let m = media(&bytes);
            prop_assert!(matches(&bytes, &m));
            let mut other = bytes.clone();
            let ix = at % other.len();
            other[ix] ^= 1 << bit;
            prop_assert!(!matches(&other, &m));
        }
    }
}
