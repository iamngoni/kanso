use std::sync::Mutex;
use std::sync::atomic::AtomicBool;

use kanso_update::{Asset, Check, CheckError, FeedError, FetchError, Update};

use super::{Backend, Phase, Recovery, Way};
use crate::updates::Problem;

fn update(version: &str) -> Update {
    Update {
        version: version.into(),
        pub_date: "2026-01-01".into(),
        release_url: "https://github.com/iamngoni/kanso/releases/tag/v1".into(),
        notes: String::new(),
        asset: Asset {
            url: "https://github.com/iamngoni/kanso/releases/download/v1/k.tar.gz".into(),
            size: 1000,
            sha256: "00".repeat(32),
        },
    }
}

/// A feed that answers from a script.
struct Fake {
    check: Mutex<Option<Result<Check, CheckError>>>,
    install: Mutex<Option<Result<(), Problem>>>,
}

impl Backend for Fake {
    fn check(&self) -> Result<Check, CheckError> {
        self.check.lock().unwrap().take().unwrap()
    }
    fn install(
        &self,
        _: &Update,
        progress: &mut dyn FnMut(u64, u64),
        _: &AtomicBool,
    ) -> Result<(), Problem> {
        progress(500, 1000);
        self.install.lock().unwrap().take().unwrap()
    }
}

fn fake(check: Result<Check, CheckError>, install: Result<(), Problem>) -> Fake {
    Fake {
        check: Mutex::new(Some(check)),
        install: Mutex::new(Some(install)),
    }
}

fn machine(way: Way, bug: bool) -> Recovery {
    Recovery::new(way, bug, &semver::Version::new(0, 10, 0))
}

fn latest() -> Result<Check, CheckError> {
    Ok(Check::UpToDate {
        latest: "0.10.0".into(),
    })
}

#[test]
fn starts_by_checking() {
    let r = machine(Way::Replace, true);
    assert_eq!(r.phase, Phase::Checking);
    assert_eq!(r.headline(), "Checking for updates…");
}

#[test]
fn available_then_installing() {
    let f = fake(Ok(Check::Available(update("0.10.1"))), Ok(()));
    let mut r = machine(Way::Replace, true);
    r.checked(f.check());
    assert!(matches!(r.phase, Phase::Available(_)));
    assert_eq!(
        r.headline(),
        "Sorry — this is a bug in Kanso 0.10.0, not a problem with your notes. Your notes are safe. Kanso 0.10.1 fixes it."
    );
    let u = r.update_now().expect("an update to install");
    assert!(matches!(
        r.phase,
        Phase::Installing {
            done: 0,
            total: 1000
        }
    ));
    let mut seen = (0, 0);
    let result = f.install(&u, &mut |d, t| seen = (d, t), &AtomicBool::new(false));
    r.progress(seen.0, seen.1);
    assert_eq!(r.headline(), "Downloading the update… 50%");
    r.installed(result);
    // Success restarts the app; the machine stays "installing".
    assert!(matches!(r.phase, Phase::Installing { .. }));
}

#[test]
fn available_without_a_bug_says_it_may_fix_it() {
    let mut r = machine(Way::Replace, false);
    r.checked(Ok(Check::Available(update("0.10.1"))));
    assert_eq!(r.headline(), "Kanso 0.10.1 is available — it may fix this.");
}

#[test]
fn latest_version() {
    let mut r = machine(Way::Replace, false);
    r.checked(latest());
    assert_eq!(r.headline(), "You're on the latest version.");
    assert!(r.update_now().is_none());
    let mut b = machine(Way::Replace, true);
    b.checked(latest());
    assert!(b.headline().starts_with(
        "Sorry — Kanso hit a bug opening your library. Your notes haven't been changed."
    ));
}

#[test]
fn a_failed_check_offers_retry_and_retry_checks_again() {
    let mut r = machine(Way::Replace, true);
    r.checked(Err(CheckError::Fetch(FetchError::Offline)));
    assert_eq!(r.phase, Phase::CheckFailed(Problem::Offline));
    assert!(r.can_retry());
    r.begin_check();
    assert_eq!(r.phase, Phase::Checking);
    r.checked(Err(CheckError::Feed(FeedError::NoTrustedKeys)));
    assert!(matches!(r.phase, Phase::CheckFailed(_)));
}

#[test]
fn a_failed_install_offers_retry() {
    let mut r = machine(Way::Replace, true);
    r.checked(Ok(Check::Available(update("0.10.1"))));
    r.update_now().unwrap();
    r.installed(Err(Problem::Verification));
    assert_eq!(r.phase, Phase::InstallFailed(Problem::Verification));
    assert!(r.can_retry());
}

#[test]
fn apt_and_manual_copies_never_install_themselves() {
    for way in [Way::Apt, Way::Page] {
        let mut r = machine(way, true);
        r.checked(Ok(Check::Available(update("0.10.1"))));
        assert!(r.update_now().is_none());
        assert!(matches!(r.phase, Phase::Available(_)));
    }
}

#[test]
fn a_late_answer_changes_nothing() {
    let mut r = machine(Way::Replace, true);
    r.checked(latest());
    r.checked(Ok(Check::Available(update("9.9.9"))));
    assert_eq!(r.phase, Phase::Latest);
}
