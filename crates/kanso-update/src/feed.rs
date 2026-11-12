//! The release feed, `latest.json`, and its Ed25519 signature.
//!
//! The signature covers the feed's exact bytes and is checked before the
//! JSON is even parsed. The public keys are compiled in from
//! `keys/trusted-keys.txt`; a list, so a new key can ship a release before
//! the old one is retired.

use std::collections::BTreeMap;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;
use ed25519_dalek::{Signature, Signer as _, SigningKey, VerifyingKey};
use semver::Version;
use serde::{Deserialize, Serialize};

use crate::platform::Platform;
use crate::policy::{PolicyError, UrlPolicy};

/// The feed layout this updater reads. A newer one means a newer updater
/// is needed: the app sends people to the release page instead.
pub const SCHEMA: u32 = 1;
/// The largest feed and signature the app reads.
pub const MAX_FEED_BYTES: u64 = 256 * 1024;
pub const MAX_SIGNATURE_BYTES: u64 = 1024;
/// The largest download the app takes, whatever the feed says.
pub const MAX_ASSET_BYTES: u64 = 512 * 1024 * 1024;

/// `latest.json`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Feed {
    pub schema: u32,
    /// Semver, without a leading `v`.
    pub version: String,
    /// RFC 3339.
    pub pub_date: String,
    /// The release's page on GitHub.
    pub release_url: String,
    /// The release's changelog section, in Markdown.
    pub notes: String,
    /// By [`Platform::key`].
    pub platforms: BTreeMap<String, Asset>,
    /// The release's short What's new video, an animated WebP
    /// ([`crate::whats_new`]). Optional: a feed without it reads as before,
    /// and an app older than the field skips it (serde ignores fields it
    /// doesn't know, and the signature covers the bytes either way).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub whats_new: Option<Asset>,
    /// The notes of the releases before this one, newest first, for "Also
    /// since your last update" when someone skipped some. Optional too.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub earlier: Vec<Earlier>,
}

/// An earlier release's notes, carried in a later feed.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Earlier {
    pub version: String,
    /// RFC 3339 or a bare date; may be empty.
    #[serde(default)]
    pub pub_date: String,
    /// Its changelog section, in Markdown.
    pub notes: String,
}

/// One system's update.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Asset {
    pub url: String,
    /// Lowercase hex.
    pub sha256: String,
    pub size: u64,
}

/// A newer release, checked, with this system's download.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Update {
    pub version: String,
    pub pub_date: String,
    pub release_url: String,
    pub notes: String,
    pub asset: Asset,
    /// The checked What's new video, when the release has one.
    #[serde(default)]
    pub whats_new: Option<Asset>,
    #[serde(default)]
    pub earlier: Vec<Earlier>,
}

/// What a checked feed says.
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(clippy::large_enum_variant)]
pub enum Check {
    /// The feed's release is this one or older: nothing to do. An older
    /// feed is how a replayed or rolled-back feed looks, and is never
    /// installed.
    UpToDate {
        latest: String,
    },
    Available(Update),
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum FeedError {
    #[error("the update's signature doesn't match")]
    BadSignature,
    #[error("this build has no key to check updates with")]
    NoTrustedKeys,
    #[error("the update information is too large")]
    TooLarge,
    #[error("the update information can't be read")]
    Malformed,
    #[error("this release needs a newer Kanso to install it")]
    NewerSchema,
    #[error("the release's version isn't valid")]
    BadVersion,
    #[error("this release has no download for this system")]
    NoBuildForPlatform,
    #[error("the download's address isn't allowed: {0}")]
    BadUrl(PolicyError),
    #[error("the download's checksum or size isn't valid")]
    BadAsset,
}

/// The keys compiled into this build, from `keys/trusted-keys.txt`.
pub fn trusted_keys() -> Vec<VerifyingKey> {
    parse_key_list(include_str!("../keys/trusted-keys.txt"))
}

/// The keys in a key list: one base64 public key a line; blank lines and
/// lines starting with `#` are skipped, and so is anything that isn't a key.
pub fn parse_key_list(text: &str) -> Vec<VerifyingKey> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter_map(|line| parse_public_key(line).ok())
        .collect()
}

/// A base64 Ed25519 public key.
pub fn parse_public_key(text: &str) -> Result<VerifyingKey, FeedError> {
    let bytes = B64.decode(text.trim()).map_err(|_| FeedError::Malformed)?;
    let bytes: [u8; 32] = bytes.try_into().map_err(|_| FeedError::Malformed)?;
    VerifyingKey::from_bytes(&bytes).map_err(|_| FeedError::Malformed)
}

/// A key's base64, as `trusted-keys.txt` lists it.
pub fn public_key_text(key: &VerifyingKey) -> String {
    B64.encode(key.to_bytes())
}

/// A base64 Ed25519 secret key (the 32-byte seed).
pub fn parse_signing_key(text: &str) -> Result<SigningKey, FeedError> {
    let bytes = zeroize::Zeroizing::new(B64.decode(text.trim()).map_err(|_| FeedError::Malformed)?);
    let seed: [u8; 32] = bytes
        .as_slice()
        .try_into()
        .map_err(|_| FeedError::Malformed)?;
    Ok(SigningKey::from_bytes(&seed))
}

/// A new signing key from the system's randomness, and its base64.
pub fn generate_signing_key() -> (SigningKey, zeroize::Zeroizing<String>) {
    let mut seed = zeroize::Zeroizing::new([0u8; 32]);
    getrandom::fill(seed.as_mut()).expect("the system's random source");
    let key = SigningKey::from_bytes(&seed);
    (key, zeroize::Zeroizing::new(B64.encode(*seed)))
}

/// `latest.json.sig`: the base64 signature over `feed`, and a newline.
pub fn sign_feed(feed: &[u8], key: &SigningKey) -> String {
    format!("{}\n", B64.encode(key.sign(feed).to_bytes()))
}

/// Checks `signature` (the text of `latest.json.sig`) over `feed`'s exact
/// bytes against each of `keys`.
pub fn verify_signature(
    feed: &[u8],
    signature: &[u8],
    keys: &[VerifyingKey],
) -> Result<(), FeedError> {
    verify_signature_with_cap(feed, signature, keys, MAX_FEED_BYTES, MAX_SIGNATURE_BYTES)
}

/// [`verify_signature`] with the caller's caps: the theme gallery's index is
/// larger than a release feed, so it passes its own limit here rather than
/// growing a second verifier.
pub fn verify_signature_with_cap(
    feed: &[u8],
    signature: &[u8],
    keys: &[VerifyingKey],
    max_feed: u64,
    max_signature: u64,
) -> Result<(), FeedError> {
    if keys.is_empty() {
        return Err(FeedError::NoTrustedKeys);
    }
    if feed.len() as u64 > max_feed || signature.len() as u64 > max_signature {
        return Err(FeedError::TooLarge);
    }
    let text = std::str::from_utf8(signature).map_err(|_| FeedError::BadSignature)?;
    let bytes = B64
        .decode(text.trim())
        .map_err(|_| FeedError::BadSignature)?;
    let bytes: [u8; 64] = bytes.try_into().map_err(|_| FeedError::BadSignature)?;
    let signature = Signature::from_bytes(&bytes);
    if keys
        .iter()
        .any(|key| key.verify_strict(feed, &signature).is_ok())
    {
        Ok(())
    } else {
        Err(FeedError::BadSignature)
    }
}

/// Checks a fetched feed end to end: the signature first, then the JSON,
/// the version against `current`, this system's download and every address.
pub fn check_feed(
    feed: &[u8],
    signature: &[u8],
    keys: &[VerifyingKey],
    current: &Version,
    platform: Platform,
    policy: &UrlPolicy,
) -> Result<Check, FeedError> {
    verify_signature(feed, signature, keys)?;
    let feed: Feed = serde_json::from_slice(feed).map_err(|_| FeedError::Malformed)?;
    let version = Version::parse(&feed.version).map_err(|_| FeedError::BadVersion)?;
    if version <= *current {
        return Ok(Check::UpToDate {
            latest: feed.version,
        });
    }
    if feed.schema > SCHEMA {
        return Err(FeedError::NewerSchema);
    }
    if feed.schema != SCHEMA {
        return Err(FeedError::Malformed);
    }
    policy
        .check_release_page(&feed.release_url)
        .map_err(FeedError::BadUrl)?;
    let asset = feed
        .platforms
        .get(platform.key())
        .cloned()
        .ok_or(FeedError::NoBuildForPlatform)?;
    policy.check_asset(&asset.url).map_err(FeedError::BadUrl)?;
    if !is_sha256_hex(&asset.sha256) || asset.size == 0 || asset.size > MAX_ASSET_BYTES {
        return Err(FeedError::BadAsset);
    }
    Ok(Check::Available(Update {
        version: version.to_string(),
        pub_date: feed.pub_date,
        release_url: feed.release_url,
        notes: feed.notes,
        asset,
        whats_new: crate::whats_new::checked_media(feed.whats_new, policy),
        earlier: crate::whats_new::checked_earlier(feed.earlier),
    }))
}

/// 64 lowercase hex digits.
pub fn is_sha256_hex(text: &str) -> bool {
    text.len() == 64 && text.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn key(seed: u8) -> SigningKey {
        SigningKey::from_bytes(&[seed; 32])
    }

    fn asset(platform: Platform, version: &str) -> Asset {
        Asset {
            url: format!(
                "https://github.com/iamngoni/kanso/releases/download/v{version}/{}",
                platform.asset_name(version)
            ),
            sha256: "ab".repeat(32),
            size: 1234,
        }
    }

    fn feed(version: &str) -> Feed {
        Feed {
            schema: SCHEMA,
            version: version.into(),
            pub_date: "2026-10-20T10:00:00Z".into(),
            release_url: format!("https://github.com/iamngoni/kanso/releases/tag/v{version}"),
            notes: "### New\n\n- Updates.".into(),
            platforms: Platform::ALL
                .into_iter()
                .map(|p| (p.key().to_owned(), asset(p, version)))
                .collect(),
            whats_new: None,
            earlier: Vec::new(),
        }
    }

    fn signed(feed: &Feed, key: &SigningKey) -> (Vec<u8>, Vec<u8>) {
        let bytes = serde_json::to_vec_pretty(feed).unwrap();
        let sig = sign_feed(&bytes, key).into_bytes();
        (bytes, sig)
    }

    fn check(bytes: &[u8], sig: &[u8], current: &str) -> Result<Check, FeedError> {
        check_feed(
            bytes,
            sig,
            &[key(1).verifying_key()],
            &Version::parse(current).unwrap(),
            Platform::MacosArm64,
            &UrlPolicy::RELEASE,
        )
    }

    #[test]
    fn a_newer_signed_release_is_available() {
        let (bytes, sig) = signed(&feed("0.2.0"), &key(1));
        let Check::Available(update) = check(&bytes, &sig, "0.1.0").unwrap() else {
            panic!("expected an update");
        };
        assert_eq!(update.version, "0.2.0");
        assert_eq!(update.asset, asset(Platform::MacosArm64, "0.2.0"));
    }

    #[test]
    fn the_same_or_an_older_release_is_up_to_date() {
        for current in ["0.2.0", "0.3.0", "1.0.0"] {
            let (bytes, sig) = signed(&feed("0.2.0"), &key(1));
            assert_eq!(
                check(&bytes, &sig, current),
                Ok(Check::UpToDate {
                    latest: "0.2.0".into()
                })
            );
        }
    }

    #[test]
    fn each_failure_is_named() {
        let good = feed("0.2.0");
        let (bytes, sig) = signed(&good, &key(2));
        assert_eq!(
            check(&bytes, &sig, "0.1.0"),
            Err(FeedError::BadSignature),
            "another key"
        );
        let (bytes, _) = signed(&good, &key(1));
        assert_eq!(
            check(&bytes, b"", "0.1.0"),
            Err(FeedError::BadSignature),
            "no signature"
        );
        assert_eq!(
            check(&bytes, b"!!", "0.1.0"),
            Err(FeedError::BadSignature),
            "garbage"
        );
        assert_eq!(
            check_feed(
                &bytes,
                b"",
                &[],
                &Version::new(0, 1, 0),
                Platform::MacosArm64,
                &UrlPolicy::RELEASE
            ),
            Err(FeedError::NoTrustedKeys)
        );
        let huge = vec![b' '; MAX_FEED_BYTES as usize + 1];
        let sig = sign_feed(&huge, &key(1)).into_bytes();
        assert_eq!(check(&huge, &sig, "0.1.0"), Err(FeedError::TooLarge));

        let (bytes, sig) = {
            let bytes = b"{not json".to_vec();
            let sig = sign_feed(&bytes, &key(1)).into_bytes();
            (bytes, sig)
        };
        assert_eq!(check(&bytes, &sig, "0.1.0"), Err(FeedError::Malformed));

        let mut newer = good.clone();
        newer.schema = SCHEMA + 1;
        let (bytes, sig) = signed(&newer, &key(1));
        assert_eq!(check(&bytes, &sig, "0.1.0"), Err(FeedError::NewerSchema));

        let mut bad = good.clone();
        bad.version = "two".into();
        let (bytes, sig) = signed(&bad, &key(1));
        assert_eq!(check(&bytes, &sig, "0.1.0"), Err(FeedError::BadVersion));

        let mut missing = good.clone();
        missing.platforms.remove("macos-arm64");
        let (bytes, sig) = signed(&missing, &key(1));
        assert_eq!(
            check(&bytes, &sig, "0.1.0"),
            Err(FeedError::NoBuildForPlatform)
        );

        let mut elsewhere = good.clone();
        elsewhere.platforms.get_mut("macos-arm64").unwrap().url =
            "https://evil.example/Kanso.tar.gz".into();
        let (bytes, sig) = signed(&elsewhere, &key(1));
        assert_eq!(
            check(&bytes, &sig, "0.1.0"),
            Err(FeedError::BadUrl(PolicyError::WrongHost))
        );

        let mut page = good.clone();
        page.release_url = "http://github.com/iamngoni/kanso/releases/tag/v0.2.0".into();
        let (bytes, sig) = signed(&page, &key(1));
        assert_eq!(
            check(&bytes, &sig, "0.1.0"),
            Err(FeedError::BadUrl(PolicyError::NotHttps))
        );

        for (sha, size) in [
            ("AB".repeat(32), 1),
            ("ab".repeat(31), 1),
            ("ab".repeat(32), 0),
            ("ab".repeat(32), MAX_ASSET_BYTES + 1),
        ] {
            let mut odd = good.clone();
            let a = odd.platforms.get_mut("macos-arm64").unwrap();
            a.sha256 = sha;
            a.size = size;
            let (bytes, sig) = signed(&odd, &key(1));
            assert_eq!(check(&bytes, &sig, "0.1.0"), Err(FeedError::BadAsset));
        }
    }

    #[test]
    fn key_lists_skip_comments_and_rotate() {
        let a = public_key_text(&key(1).verifying_key());
        let b = public_key_text(&key(2).verifying_key());
        let list = format!("# Kanso update keys\n\n{a}\nnot-a-key\n  {b}  \n");
        let keys = parse_key_list(&list);
        assert_eq!(keys.len(), 2);
        // Either key signs a release the app takes.
        for signer in [key(1), key(2)] {
            let (bytes, sig) = signed(&feed("0.2.0"), &signer);
            assert!(verify_signature(&bytes, &sig, &keys).is_ok());
        }
        // The shipped list holds only keys (or none yet).
        let shipped = include_str!("../keys/trusted-keys.txt");
        let lines = shipped
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .count();
        assert_eq!(trusted_keys().len(), lines);
    }

    #[test]
    fn signing_keys_round_trip_through_text() {
        let (key, text) = generate_signing_key();
        let back = parse_signing_key(&text).unwrap();
        assert_eq!(back.verifying_key(), key.verifying_key());
        assert!(parse_signing_key("short").is_err());
    }

    fn version() -> impl Strategy<Value = Version> {
        (0u64..4, 0u64..12, 0u64..12).prop_map(|(a, b, c)| Version::new(a, b, c))
    }

    proptest! {
        /// What one key signs, it verifies; no other key does.
        #[test]
        fn prop_sign_verify_round_trip(seed in 1u8..=255, other in 1u8..=255, bytes in prop::collection::vec(any::<u8>(), 0..2048)) {
            let sig = sign_feed(&bytes, &key(seed));
            prop_assert!(verify_signature(&bytes, sig.as_bytes(), &[key(seed).verifying_key()]).is_ok());
            if other != seed {
                prop_assert_eq!(
                    verify_signature(&bytes, sig.as_bytes(), &[key(other).verifying_key()]),
                    Err(FeedError::BadSignature)
                );
            }
        }

        /// Flipping any bit of the feed or the signature is caught.
        #[test]
        fn prop_any_tampering_is_rejected(
            v in version(),
            at in any::<prop::sample::Index>(),
            bit in 0u8..8,
            in_signature in any::<bool>(),
        ) {
            let (mut bytes, mut sig) = signed(&feed(&v.to_string()), &key(1));
            if in_signature {
                // Within the base64, not its newline.
                let ix = at.index(sig.len() - 1);
                sig[ix] ^= 1 << bit;
            } else {
                let ix = at.index(bytes.len());
                bytes[ix] ^= 1 << bit;
            }
            prop_assert!(matches!(
                check(&bytes, &sig, "0.0.0"),
                Err(FeedError::BadSignature)
            ));
        }

        /// Only a strictly newer version is ever offered: no downgrade, no
        /// replay of the release already running.
        #[test]
        fn prop_only_newer_versions_are_offered(feed_v in version(), current in version()) {
            let (bytes, sig) = signed(&feed(&feed_v.to_string()), &key(1));
            match check(&bytes, &sig, &current.to_string()).unwrap() {
                Check::Available(update) => {
                    prop_assert!(feed_v > current);
                    prop_assert_eq!(update.version, feed_v.to_string());
                }
                Check::UpToDate { .. } => prop_assert!(feed_v <= current),
            }
        }

        /// The download offered is this system's, and only ever this
        /// system's.
        #[test]
        fn prop_the_platform_picks_its_own_asset(ix in 0usize..4, keep in prop::collection::vec(any::<bool>(), 4)) {
            let platform = Platform::ALL[ix];
            let mut f = feed("9.0.0");
            for (p, keep) in Platform::ALL.into_iter().zip(&keep) {
                if !keep {
                    f.platforms.remove(p.key());
                }
            }
            let (bytes, sig) = signed(&f, &key(1));
            let got = check_feed(&bytes, &sig, &[key(1).verifying_key()], &Version::new(0, 1, 0), platform, &UrlPolicy::RELEASE);
            if keep[ix] {
                let Ok(Check::Available(update)) = got else { panic!("expected an update") };
                prop_assert!(update.asset.url.ends_with(&platform.asset_name("9.0.0")));
            } else {
                prop_assert_eq!(got, Err(FeedError::NoBuildForPlatform));
            }
        }
    }
}
