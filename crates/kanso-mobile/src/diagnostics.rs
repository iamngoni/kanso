//! What the shell asks of the library about itself: its version, whether
//! certificates can be checked, and a panic on demand to prove panics come
//! back as errors on a device.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::Secrets;
use crate::backup;
use crate::boundary::guard;
use crate::errors::KansoError;
use crate::secrets;

/// The item the secrets probe writes and removes.
const PROBE_ITEM: &str = "diagnostic-probe";

/// The library's version, the workspace's.
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_owned()
}

/// Whether the library folder `folder` is kept out of the device's backups:
/// `Some(true)` once a library has been opened in it on iOS (or on a Mac, for
/// tests), `None` where the platform has no such flag (Android keeps the app
/// out of backups in its manifest).
pub fn backup_excluded(folder: &Path) -> Option<bool> {
    backup::is_excluded(folder)
}

/// Panics with `message` inside the boundary: the result is
/// [`KansoError::Internal`], and every other call works afterwards. It holds
/// no lock of the store's, so it can't poison one. It ships in the release
/// library so a device test can check that a panic is caught there too.
pub fn panic_probe(message: String) -> Result<(), KansoError> {
    guard(|| panic!("{message}"))
}

/// Whether the library can check a server's certificate now: always on iOS
/// and anywhere else, on Android once the app has handed over its `Context`.
pub fn tls_ready() -> bool {
    kanso_tls::ready().is_ok()
}

/// Writes a value to the phone's secret store (the Keychain, the Keystore),
/// reads it back, compares, removes it and checks it is gone: the whole round
/// trip a signed-in account's session takes, with nothing of the account's.
/// On Android this goes through JNI to the Keystore, so a device test can
/// prove that path with no server. The value is made new each time and is
/// never in an error.
pub fn secrets_probe() -> Result<(), KansoError> {
    guard(|| {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |since| since.as_nanos());
        // Not only ASCII: items hold JSON with any account name.
        let value = format!("kanso-probe-{nanos:x}-é-日本");
        probe_store(secrets::platform().as_ref(), PROBE_ITEM, &value)
    })
}

/// The round trip of [`secrets_probe`] on `store`.
pub(crate) fn probe_store(store: &dyn Secrets, item: &str, value: &str) -> Result<(), KansoError> {
    store.set(item, value)?;
    let read = store.get(item)?;
    if read.as_deref().map(String::as_str) != Some(value) {
        // Not the value: it may be a secret's, so it isn't named here.
        let _ = store.delete(item);
        return Err(KansoError::Internal {
            message: "the secret store gave back another value than it was given".into(),
        });
    }
    store.delete(item)?;
    if store.get(item)?.is_some() {
        return Err(KansoError::Internal {
            message: "the secret store kept an item it was asked to remove".into(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use kanso_sync::{MemorySecrets, SecretError};
    use proptest::prelude::*;
    use zeroize::Zeroizing;

    /// A store that fails, or lies, in one way.
    struct Faulty(Fault, std::sync::Mutex<Option<String>>);

    impl Faulty {
        fn new(fault: Fault) -> Faulty {
            Faulty(fault, std::sync::Mutex::new(None))
        }
    }

    #[derive(Clone, Copy)]
    enum Fault {
        Refuses,
        ReadsOther,
        KeepsDeleted,
    }

    impl Secrets for Faulty {
        fn get(&self, _item: &str) -> Result<Option<Zeroizing<String>>, SecretError> {
            match self.0 {
                Fault::Refuses => Err(SecretError("locked".into())),
                Fault::ReadsOther => Ok(Some(Zeroizing::new("other".into()))),
                Fault::KeepsDeleted => Ok(self.1.lock().unwrap().clone().map(Zeroizing::new)),
            }
        }
        fn set(&self, _item: &str, value: &str) -> Result<(), SecretError> {
            match self.0 {
                Fault::Refuses => Err(SecretError("locked".into())),
                _ => {
                    *self.1.lock().unwrap() = Some(value.into());
                    Ok(())
                }
            }
        }
        fn delete(&self, _item: &str) -> Result<(), SecretError> {
            Ok(())
        }
    }

    proptest! {
        /// Any value goes into a working store and comes out, and nothing is
        /// left behind.
        #[test]
        fn prop_the_probe_round_trips_any_value_and_leaves_nothing(
            item in "[a-z:_-]{1,24}",
            value in "\\PC{0,64}",
        ) {
            let store = MemorySecrets::new();
            prop_assert_eq!(probe_store(&store, &item, &value), Ok(()));
            prop_assert!(store.get(&item).unwrap().is_none());
        }
    }

    // no-prop-test: the three ways a store goes wrong, one case each.
    #[test]
    fn a_store_that_fails_or_lies_is_an_error_that_names_no_value() {
        let refused = probe_store(&Faulty::new(Fault::Refuses), "i", "secret-value");
        assert!(matches!(refused, Err(KansoError::Auth { .. })));
        for fault in [Fault::ReadsOther, Fault::KeepsDeleted] {
            let err = probe_store(&Faulty::new(fault), "i", "secret-value").unwrap_err();
            assert!(matches!(err, KansoError::Internal { .. }));
            assert!(!format!("{err:?}").contains("secret-value"));
        }
    }

    // no-prop-test: the call itself, on this computer's memory store.
    #[test]
    #[cfg_attr(
        target_os = "ios",
        ignore = "an unsigned simulator test binary has no Keychain access (-34018)"
    )]
    fn the_probe_passes_on_the_platforms_store() {
        assert_eq!(secrets_probe(), Ok(()));
    }

    // no-prop-test: the probe's message is arbitrary text; boundary.rs's
    // property covers every message, this checks the public call.
    #[test]
    fn the_probe_returns_internal_and_the_next_call_still_works() {
        let failed = panic_probe("probe".into());
        assert!(matches!(failed, Err(KansoError::Internal { message }) if message == "probe"));
        assert_eq!(version(), env!("CARGO_PKG_VERSION"));
        assert_eq!(tls_ready(), !cfg!(target_os = "android"));
    }
}
