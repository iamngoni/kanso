//! Whether this copy of Kanso can show system notifications, asked before
//! anything touches the platform's notification centre.
//!
//! On macOS, `UNUserNotificationCenter` raises an Objective-C exception
//! (and the process aborts) when the process has no bundle identifier, as
//! with `cargo run` or the bare `target/*/kanso` binary. So reminders only
//! go near it when the main bundle has an identifier and the executable
//! sits inside a `.app` (`….app/Contents/MacOS/…`). Otherwise reminders do
//! nothing, and say why once in the log and in Settings. Kanso and Kanso
//! Dev are separate bundles (`app.kanso.Kanso`, `app.kanso.Kanso.dev`), so
//! each has its own notification identity and permission.
//!
//! When the person turned notifications off for Kanso, reminders stay
//! quiet; Settings › Reminders says so in one line.

#[cfg(any(target_os = "macos", test))]
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

/// What notifications can do here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Availability {
    /// Allowed, or not asked yet (the first reminder asks).
    Ready,
    /// Not running as an app (macOS without a bundle): nothing is shown.
    NotAnApp,
    /// Turned off for Kanso in the system's settings.
    Denied,
}

/// Whether a process with bundle identifier `bundle_id` running the
/// executable at `exe` is an app the notification centre accepts: an
/// identifier, and the executable at `<Name>.app/Contents/MacOS/<exe>`.
/// Pure. Only macOS and the tests use it.
#[cfg(any(target_os = "macos", test))]
pub fn runs_as_app(bundle_id: Option<&str>, exe: &Path) -> bool {
    let has_id = bundle_id.is_some_and(|id| !id.trim().is_empty());
    let macos = exe.parent();
    let contents = macos.and_then(Path::parent);
    let app = contents.and_then(Path::parent);
    let in_app = macos
        .and_then(Path::file_name)
        .is_some_and(|n| n == "MacOS")
        && contents
            .and_then(Path::file_name)
            .is_some_and(|n| n == "Contents")
        && app
            .and_then(Path::extension)
            .is_some_and(|ext| ext.eq_ignore_ascii_case("app"));
    has_id && in_app
}

/// The system's answer about permission, as last heard.
const UNKNOWN: u8 = 0;
#[cfg(target_os = "macos")]
const ALLOWED: u8 = 1;
const DENIED: u8 = 2;
static PERMISSION: AtomicU8 = AtomicU8::new(UNKNOWN);
static LOGGED_NOT_AN_APP: AtomicBool = AtomicBool::new(false);

/// Whether this process runs as an app bundle; always true off macOS,
/// where notifications need no bundle.
pub fn is_app() -> bool {
    #[cfg(target_os = "macos")]
    {
        use objc2_foundation::NSBundle;
        let id = NSBundle::mainBundle()
            .bundleIdentifier()
            .map(|id| id.to_string());
        let exe = std::env::current_exe().unwrap_or_default();
        runs_as_app(id.as_deref(), &exe)
    }
    #[cfg(not(target_os = "macos"))]
    {
        true
    }
}

/// What notifications can do now. Logs once when this isn't an app.
pub fn availability() -> Availability {
    if !is_app() {
        if !LOGGED_NOT_AN_APP.swap(true, Ordering::Relaxed) {
            eprintln!(
                "kanso: reminders: not running from an app bundle, so no notifications are shown"
            );
        }
        return Availability::NotAnApp;
    }
    match PERMISSION.load(Ordering::Relaxed) {
        DENIED => Availability::Denied,
        _ => Availability::Ready,
    }
}

/// Asks the system, in the background, whether notifications are allowed;
/// [`availability`] has the answer once it comes. Does nothing when this
/// isn't an app, so the notification centre is never touched then.
pub fn refresh_permission() {
    #[cfg(target_os = "macos")]
    {
        if !is_app() {
            return;
        }
        use block2::RcBlock;
        use objc2_user_notifications::{
            UNAuthorizationStatus, UNNotificationSettings, UNUserNotificationCenter,
        };
        use std::ptr::NonNull;
        let center = UNUserNotificationCenter::currentNotificationCenter();
        let handler = RcBlock::new(|settings: NonNull<UNNotificationSettings>| {
            // SAFETY: the system passes valid settings for the callback.
            let status = unsafe { settings.as_ref() }.authorizationStatus();
            let value = if status == UNAuthorizationStatus::Denied {
                DENIED
            } else if status == UNAuthorizationStatus::NotDetermined {
                UNKNOWN
            } else {
                ALLOWED
            };
            let before = PERMISSION.swap(value, Ordering::Relaxed);
            if value == DENIED && before != DENIED {
                eprintln!("kanso: reminders: notifications are turned off for this app");
            }
        });
        center.getNotificationSettingsWithCompletionHandler(&handler);
    }
}

/// The one line Settings › Reminders shows when reminders can't reach the
/// screen, if any. Pure.
pub fn settings_note(availability: Availability, name: &str) -> Option<String> {
    match availability {
        Availability::Ready => None,
        Availability::NotAnApp => Some(format!(
            "This copy of {name} isn't running as an app, so it can't show notifications."
        )),
        Availability::Denied => Some(format!(
            "Notifications are off for {name}. Turn them on in System Settings › Notifications."
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::{Availability, runs_as_app, settings_note};
    use proptest::prelude::*;
    use std::path::{Path, PathBuf};

    #[test]
    fn only_a_bundled_app_may_notify() {
        let app = Path::new("/Applications/Kanso.app/Contents/MacOS/kanso");
        assert!(runs_as_app(Some("app.kanso.Kanso"), app));
        let dev = Path::new("/Users/me/Applications/Kanso Dev.app/Contents/MacOS/kanso");
        assert!(runs_as_app(Some("app.kanso.Kanso.dev"), dev));
        // `cargo run` and the bare binary: no identifier, no bundle.
        let bare = Path::new("/src/kanso/target/debug/kanso");
        assert!(!runs_as_app(None, bare));
        assert!(!runs_as_app(Some("app.kanso.Kanso"), bare));
        assert!(!runs_as_app(None, app));
        assert!(!runs_as_app(Some("  "), app));
        assert!(!runs_as_app(
            Some("app.kanso.Kanso"),
            Path::new("/x/Kanso.app/Contents/Resources/kanso")
        ));
        assert!(!runs_as_app(Some("app.kanso.Kanso"), Path::new("kanso")));
    }

    #[test]
    fn the_dev_flavour_has_its_own_identity() {
        assert_eq!(crate::flavour::DEV.bundle_id, "app.kanso.Kanso.dev");
        assert_ne!(
            crate::flavour::DEV.bundle_id,
            crate::flavour::RELEASE.bundle_id
        );
        // The identity GPUI is given (the AUMID on Windows) is the flavour's.
        assert!(
            crate::flavour::CURRENT
                .bundle_id
                .starts_with("app.kanso.Kanso")
        );
    }

    #[test]
    fn settings_say_why_in_one_line() {
        assert_eq!(settings_note(Availability::Ready, "Kanso"), None);
        for availability in [Availability::NotAnApp, Availability::Denied] {
            let note = settings_note(availability, "Kanso Dev").unwrap();
            assert!(!note.contains('\n') && note.contains("Kanso Dev"));
        }
    }

    proptest! {
        /// An executable outside `<x>.app/Contents/MacOS/`, or a missing
        /// identifier, never counts as an app.
        #[test]
        fn prop_outside_a_bundle_is_never_an_app(
            parts in prop::collection::vec("[A-Za-z ]{1,8}", 0..5),
            id in prop::option::of("[a-z.]{0,12}"),
        ) {
            let mut path = PathBuf::from("/");
            for p in &parts {
                path.push(p);
            }
            path.push("kanso");
            // No component ends in .app, so it can't be inside a bundle.
            prop_assert!(!runs_as_app(id.as_deref(), &path));
            let bundled = PathBuf::from("/A.app/Contents/MacOS/kanso");
            prop_assert_eq!(
                runs_as_app(id.as_deref(), &bundled),
                id.as_deref().is_some_and(|i| !i.trim().is_empty())
            );
        }
    }
}
