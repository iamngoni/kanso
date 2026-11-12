//! What's new on the phone, once after an update (docs/RELEASING.md ›
//! What's new video).
//!
//! The phone reads its release's own signed feed by tag, as the desktop
//! does after an update, with the same keys and address rules; the video
//! is fetched under those rules, checked against the feed's size and
//! SHA-256 and cached in the folder the shell names. A video that fails
//! goes; the notes show alone. Nothing is shown on a first install, and
//! nothing on a build between releases.

use std::path::Path;

use kanso_core::notices::previous_version;
use kanso_store::Store;
use kanso_update::whats_new::{self, Release};
use kanso_update::{Transport, UrlPolicy, VerifyingKey};
use semver::Version;

use crate::boundary::guard;
use crate::errors::KansoError;
use crate::library::Library;
use crate::whats_new_model::{PhoneWhatsNew, PhoneWhatsNewEarlier, PhoneWhatsNewSection};

/// The version whose What's new was shown.
const SEEN_META: &str = "mobile.whats_new_seen";
/// Kept by `notices.rs`: the version that ran last, and the one before the
/// last update.
const LAST_RUN_META: &str = "mobile.last_run_version";
const PREVIOUS_META: &str = "update.previous_version";

impl Library {
    /// The What's new sheet to show now, once after an update, or none.
    /// Reads the release's feed over the network; any failure is none, and
    /// the next start asks again. `cache_dir` is where the video is kept.
    pub fn whats_new(&self, cache_dir: &str) -> Result<Option<PhoneWhatsNew>, KansoError> {
        guard(|| {
            let current = Version::parse(&crate::diagnostics::version())
                .map_err(|err| KansoError::storage(err.to_string()))?;
            let Some(previous) = due(&self.inner.store, &current)? else {
                return Ok(None);
            };
            let transport = crate::gallery_net::PhoneTransport::new();
            Ok(load(
                &transport,
                &UrlPolicy::RELEASE,
                &kanso_update::trusted_keys(),
                &current,
                &previous,
                Path::new(cache_dir),
            ))
        })
    }

    /// Notes that this version's What's new was shown.
    pub fn mark_whats_new_seen(&self) -> Result<(), KansoError> {
        guard(|| {
            self.inner
                .store
                .set_meta(SEEN_META, &crate::diagnostics::version())?;
            Ok(())
        })
    }
}

/// The version run before the update when What's new is due: a release
/// build, updated from an older one, not shown yet.
fn due(store: &Store, current: &Version) -> Result<Option<Version>, KansoError> {
    if !current.pre.is_empty() {
        return Ok(None);
    }
    if store.meta(SEEN_META)?.as_deref() == Some(current.to_string().as_str()) {
        return Ok(None);
    }
    let last_run = store.meta(LAST_RUN_META)?;
    let remembered = store.meta(PREVIOUS_META)?;
    Ok(
        previous_version(remembered.as_deref(), last_run.as_deref(), current)
            .filter(|previous| previous < current),
    )
}

/// The sheet from `current`'s own feed, with its video when it passes.
fn load(
    transport: &dyn Transport,
    policy: &UrlPolicy,
    keys: &[VerifyingKey],
    current: &Version,
    previous: &Version,
    cache_dir: &Path,
) -> Option<PhoneWhatsNew> {
    let release = whats_new::fetch_release(transport, policy, keys, current).ok()?;
    let video = release.whats_new.as_ref().and_then(|media| {
        whats_new::fetch_media(transport, policy, cache_dir, &release.version, media).ok()
    });
    Some(view(
        &release,
        previous,
        video
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default(),
    ))
}

fn sections(notes: &str) -> Vec<PhoneWhatsNewSection> {
    kanso_update::notes::plain_sections(notes)
        .into_iter()
        .map(|(title, items)| PhoneWhatsNewSection { title, items })
        .collect()
}

/// What the sheet shows of `release`. Pure.
fn view(release: &Release, previous: &Version, video_path: String) -> PhoneWhatsNew {
    let notes = kanso_update::notes::for_app(&release.notes);
    PhoneWhatsNew {
        version: release.version.clone(),
        date: release.pub_date.chars().take(10).collect(),
        summary: kanso_update::notes::summary(&notes),
        video_path,
        sections: sections(&notes),
        earlier: whats_new::since(&release.earlier, Some(previous), &release.version)
            .into_iter()
            .map(|e| PhoneWhatsNewEarlier {
                version: e.version.clone(),
                sections: sections(&e.notes),
            })
            .collect(),
        release_url: release.release_url.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kanso_update::Earlier;
    use proptest::prelude::*;

    fn release(earlier: &[&str]) -> Release {
        Release {
            version: "0.10.7".into(),
            pub_date: "2026-11-12T10:00:00Z".into(),
            release_url: "https://github.com/iamngoni/kanso/releases/tag/v0.10.7".into(),
            notes: "## [0.10.7](x) - 2026-11-12\n\n### Improved\n\n- Faster ([abcdef1](https://github.com/c))\n\n### Fixed\n\n- A crash\n\n### Under the hood\n\n- CI\n".into(),
            whats_new: None,
            earlier: earlier
                .iter()
                .map(|v| Earlier {
                    version: (*v).into(),
                    pub_date: String::new(),
                    notes: format!("### Fixed\n\n- In {v}"),
                })
                .collect(),
        }
    }

    #[test]
    fn the_sheet_reads_as_designed() {
        let got = view(
            &release(&["0.10.6", "0.10.5", "0.10.4"]),
            &Version::new(0, 10, 4),
            "/cache/Kanso-0.10.7-whats-new.webp".into(),
        );
        assert_eq!(got.date, "2026-11-12");
        assert_eq!(got.summary, "1 improvement and 1 fix.");
        assert_eq!(got.sections.len(), 2);
        assert_eq!(got.sections[0].items, vec!["Faster".to_owned()]);
        let skipped: Vec<_> = got.earlier.iter().map(|e| e.version.as_str()).collect();
        assert_eq!(skipped, ["0.10.6", "0.10.5"]);
        assert_eq!(got.video_path, "/cache/Kanso-0.10.7-whats-new.webp");
    }

    #[test]
    fn it_is_due_once_after_an_update() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("library.db")).unwrap();
        let current = Version::new(0, 10, 7);
        // A first install: nothing ran before.
        assert_eq!(due(&store, &current).unwrap(), None);
        store.set_meta(LAST_RUN_META, "0.10.5").unwrap();
        assert_eq!(due(&store, &current).unwrap(), Some(Version::new(0, 10, 5)));
        store.set_meta(SEEN_META, "0.10.7").unwrap();
        assert_eq!(due(&store, &current).unwrap(), None);
        // A build between releases never asks.
        let pre = Version::parse("0.10.8-dev.1").unwrap();
        assert_eq!(due(&store, &pre).unwrap(), None);
    }

    proptest! {
        #![proptest_config(ProptestConfig { rng_seed: proptest::test_runner::RngSeed::Fixed(0x6b616e736f), ..ProptestConfig::default() })]

        /// Only releases after the one run before are listed as skipped.
        #[test]
        fn prop_skipped_are_after_previous(prev in 0u64..8) {
            let got = view(&release(&["0.10.6", "0.10.5", "0.10.4", "0.10.3", "0.10.2"]), &Version::new(0, 10, prev), String::new());
            for e in &got.earlier {
                prop_assert!(Version::parse(&e.version).unwrap() > Version::new(0, 10, prev));
            }
            let expected = (2..=6u64).filter(|p| *p > prev).count();
            prop_assert_eq!(got.earlier.len(), expected);
        }
    }
}
