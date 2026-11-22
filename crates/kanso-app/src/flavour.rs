//! Kanso, Kanso Dev and Kanso Nightly (docs/DEVELOPING.md).
//!
//! A build is one of the three, chosen at compile time: `--features
//! dev-flavour` makes Kanso Dev, which `scripts/bundle-macos.sh --dev`
//! packs as `Kanso Dev.app`; `--features nightly-flavour` makes Kanso
//! Nightly, which `scripts/bundle-macos.sh --nightly` packs as `Kanso
//! Nightly.app`. Each has its own name, bundle ID, data folder, Keychain
//! service, API port and URL scheme, so it never opens or writes the
//! library or the account of another. Kanso Dev never updates itself;
//! Kanso Nightly updates only from the nightly feed.

#[cfg(all(feature = "dev-flavour", feature = "nightly-flavour"))]
compile_error!("a build is either Kanso Dev or Kanso Nightly, not both");

/// What tells one build apart from another.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Flavour {
    /// The app's name, in its window title and bundle.
    pub name: &'static str,
    /// The macOS bundle ID, and the login item's label.
    pub bundle_id: &'static str,
    /// The data folder's name on macOS and Windows.
    pub data_folder: &'static str,
    /// The data folder's name under `$XDG_DATA_HOME` on Linux, and the
    /// desktop entry's (and Wayland app ID's) name.
    pub xdg_name: &'static str,
    /// The Keychain (or Credential Manager, or Secret Service) service its
    /// secrets are kept under.
    pub keychain_service: &'static str,
    /// The local API's port in a new config.
    pub default_port: u16,
    /// The URL scheme it answers, such as `kanso` or `kanso-nightly`.
    pub scheme: &'static str,
    /// Whether it checks for, downloads and installs releases.
    pub updates: bool,
    /// Whether it is the development build (no updates, no metrics).
    pub dev: bool,
    /// Whether it is the nightly build, which reads the nightly feed.
    pub nightly: bool,
    /// A word the sidebar shows beside the name, so the builds are never
    /// mistaken for one another.
    pub badge: Option<&'static str>,
}

/// The build releases ship.
pub const RELEASE: Flavour = Flavour {
    name: "Kanso",
    bundle_id: "app.kanso.Kanso",
    data_folder: "Kanso",
    xdg_name: "kanso",
    keychain_service: kanso_sync::keychain::SERVICE,
    default_port: crate::config::DEFAULT_PORT,
    scheme: "kanso",
    updates: true,
    dev: false,
    nightly: false,
    badge: None,
};

/// The build made from a working copy to try changes on.
pub const DEV: Flavour = Flavour {
    name: "Kanso Dev",
    bundle_id: "app.kanso.Kanso.dev",
    data_folder: "Kanso Dev",
    xdg_name: "kanso-dev",
    keychain_service: "Kanso Dev",
    default_port: crate::config::DEFAULT_PORT + 1,
    scheme: "kanso-dev",
    updates: false,
    dev: true,
    nightly: false,
    badge: Some("DEV"),
};

/// The build made from a working copy on a schedule: it updates from the
/// nightly feed, never the stable one.
pub const NIGHTLY: Flavour = Flavour {
    name: "Kanso Nightly",
    bundle_id: "app.kanso.Kanso.nightly",
    data_folder: "Kanso Nightly",
    xdg_name: "kanso-nightly",
    keychain_service: "Kanso Nightly",
    default_port: crate::config::DEFAULT_PORT + 2,
    scheme: "kanso-nightly",
    updates: true,
    dev: false,
    nightly: true,
    badge: Some("NIGHTLY"),
};

/// This build's flavour.
pub const CURRENT: Flavour = if cfg!(feature = "nightly-flavour") {
    NIGHTLY
} else if cfg!(feature = "dev-flavour") {
    DEV
} else {
    RELEASE
};

/// The version this build reports: `KANSO_NIGHTLY_VERSION` when the nightly
/// workflow sets it, otherwise the crate's version.
pub const fn version() -> &'static str {
    match option_env!("KANSO_NIGHTLY_VERSION") {
        Some(version) => version,
        None => env!("CARGO_PKG_VERSION"),
    }
}

impl Flavour {
    pub fn is_dev(&self) -> bool {
        self.dev
    }

    pub fn is_nightly(&self) -> bool {
        self.nightly
    }

    /// Its Keychain, under its own service.
    pub fn keychain(&self) -> kanso_sync::Keychain {
        kanso_sync::Keychain::with_service(self.keychain_service)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn dev_shares_nothing_with_the_release() {
        assert_ne!(DEV.name, RELEASE.name);
        assert_ne!(DEV.bundle_id, RELEASE.bundle_id);
        assert_ne!(DEV.data_folder, RELEASE.data_folder);
        assert_ne!(DEV.xdg_name, RELEASE.xdg_name);
        assert_ne!(DEV.keychain_service, RELEASE.keychain_service);
        assert_ne!(DEV.default_port, RELEASE.default_port);
        assert_ne!(DEV.scheme, RELEASE.scheme);
        assert_ne!(RELEASE.updates, DEV.updates);
        const { assert!(!DEV.updates) };
        assert!(DEV.is_dev() && !RELEASE.is_dev());
        assert!(NIGHTLY.is_nightly() && !NIGHTLY.is_dev());
        assert_eq!(DEV.keychain().service(), "Kanso Dev");
        assert_eq!(RELEASE.keychain().service(), "Kanso");
    }

    #[test]
    fn nightly_shares_nothing_with_the_release_or_dev() {
        for other in [RELEASE, DEV] {
            assert_ne!(NIGHTLY.name, other.name);
            assert_ne!(NIGHTLY.bundle_id, other.bundle_id);
            assert_ne!(NIGHTLY.data_folder, other.data_folder);
            assert_ne!(NIGHTLY.xdg_name, other.xdg_name);
            assert_ne!(NIGHTLY.keychain_service, other.keychain_service);
            assert_ne!(NIGHTLY.default_port, other.default_port);
            assert_ne!(NIGHTLY.scheme, other.scheme);
        }
        const { assert!(NIGHTLY.updates) };
        assert!(!NIGHTLY.is_dev() && NIGHTLY.is_nightly());
        assert_eq!(NIGHTLY.keychain().service(), "Kanso Nightly");
    }

    #[test]
    fn the_release_keeps_its_names() {
        // What 0.1.0 used: changing any of these would lose people's
        // libraries and sign-ins.
        assert_eq!(RELEASE.bundle_id, "app.kanso.Kanso");
        assert_eq!(RELEASE.data_folder, "Kanso");
        assert_eq!(RELEASE.xdg_name, "kanso");
        assert_eq!(RELEASE.keychain_service, "Kanso");
        assert_eq!(RELEASE.default_port, 19840);
        assert_eq!(RELEASE.scheme, "kanso");
        assert_eq!(crate::launch::label_for(&RELEASE), "app.kanso.Kanso");
    }

    #[test]
    fn this_build_is_the_one_its_features_say() {
        assert_eq!(CURRENT == DEV, cfg!(feature = "dev-flavour"));
        assert_eq!(CURRENT == NIGHTLY, cfg!(feature = "nightly-flavour"));
    }

    #[test]
    fn dev_updates_are_off_whatever_the_install() {
        let dir = tempfile::tempdir().unwrap();
        let exe = dir.path().join("Kanso Dev.app/Contents/MacOS/kanso");
        let options = kanso_update::LocateOptions {
            dev_flavour: DEV.is_dev(),
            ..Default::default()
        };
        assert_eq!(
            kanso_update::install::locate(&exe, Some(kanso_update::Platform::MacosArm64), options),
            Err(kanso_update::NotInstallable::DevFlavour)
        );
        assert_eq!(
            kanso_update::NotInstallable::DevFlavour.reason(),
            "Development build — updates are off."
        );
    }

    proptest! {
        /// Whatever folder a flavour's data lives under, the other's is a
        /// different one: neither is inside the other.
        #[test]
        fn prop_data_folders_never_overlap(base in "[a-z]{1,8}(/[a-z]{1,8}){0,3}", pick in 0usize..3) {
            let base = std::path::PathBuf::from("/").join(base);
            let flavours = [RELEASE, DEV, NIGHTLY];
            let (a, b) = (flavours[pick], flavours[(pick + 1) % flavours.len()]);
            for (x, y) in [(a.data_folder, b.data_folder), (a.xdg_name, b.xdg_name)] {
                let (x, y) = (base.join(x), base.join(y));
                prop_assert!(!x.starts_with(&y) && !y.starts_with(&x));
            }
        }
    }
}
