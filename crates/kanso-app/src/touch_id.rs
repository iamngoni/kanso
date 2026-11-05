//! Touch ID, and Windows Hello on Windows, for unlocking locked notes
//! without typing the lock password.
//!
//! [`Biometrics`] is the seam: [`SystemTouchId`] asks macOS's
//! LocalAuthentication or Windows's UserConsentVerifier in the app, tests
//! set their own. Without the global, or on Linux, there is neither and the
//! password is asked for. Both keep the same lock keys in the system's
//! secret store ([`kanso_sync::Keychain`]: the login Keychain, or the
//! Credential Manager, which only the signed-in Windows user can read), and
//! are offered, turned on and forgotten the same way (`locked.rs`).

use std::sync::Arc;

use gpui_kit::{App, Global};

/// What the unlock is called on this system, as a literal for `concat!`:
/// "Windows Hello" on Windows, "Touch ID" elsewhere.
#[cfg(target_os = "windows")]
macro_rules! unlock_name {
    () => {
        "Windows Hello"
    };
}
#[cfg(not(target_os = "windows"))]
macro_rules! unlock_name {
    () => {
        "Touch ID"
    };
}
pub(crate) use unlock_name;

/// Where the system keeps Kanso's secrets, as a literal for `concat!`.
#[cfg(target_os = "windows")]
macro_rules! secret_store_name {
    () => {
        "Credential Manager"
    };
}
#[cfg(not(target_os = "windows"))]
macro_rules! secret_store_name {
    () => {
        "Keychain"
    };
}
pub(crate) use secret_store_name;

/// What the unlock is called on this system.
pub const NAME: &str = unlock_name!();

/// Words people might type for the unlock, in the palette.
#[cfg(target_os = "windows")]
pub const KEYWORDS: &[&str] = &["windows hello", "face", "fingerprint", "pin", "biometric"];
#[cfg(not(target_os = "windows"))]
pub const KEYWORDS: &[&str] = &["touch id", "fingerprint", "biometric"];

/// What Touch ID (or Windows Hello) can do.
pub trait Biometrics: Send + Sync {
    /// Whether this device has Touch ID (or Windows Hello) set up for its
    /// user now.
    fn available(&self) -> bool;
    /// Asks for a fingerprint (or a face or the Windows Hello PIN), saying
    /// why with `reason`; the answer comes on the channel (`false` for a
    /// cancel or a failure).
    fn authenticate(&self, reason: &str) -> async_channel::Receiver<bool>;
    /// Asks the person to prove they are this computer's user, for a secret
    /// that isn't a locked note (the library key, issue #243): Touch ID or
    /// the account password on macOS, which every Mac has; Windows Hello on
    /// Windows. `None` when the system has no such check here.
    fn confirm_owner(&self, reason: &str) -> Option<async_channel::Receiver<bool>> {
        self.available().then(|| self.authenticate(reason))
    }
}

/// The Touch ID the app uses.
pub struct TouchId(pub Arc<dyn Biometrics>);

impl Global for TouchId {}

/// Whether Touch ID can be used now.
pub fn available(cx: &App) -> bool {
    cx.try_global::<TouchId>()
        .is_some_and(|touch_id| touch_id.0.available())
}

/// Asks for a fingerprint, or `None` when there is no Touch ID.
pub fn authenticate(reason: &str, cx: &App) -> Option<async_channel::Receiver<bool>> {
    let touch_id = cx.try_global::<TouchId>()?;
    touch_id
        .0
        .available()
        .then(|| touch_id.0.authenticate(reason))
}

/// Asks the person to prove they are this computer's user
/// ([`Biometrics::confirm_owner`]), or `None` when the system can't.
pub fn confirm_owner(reason: &str, cx: &App) -> Option<async_channel::Receiver<bool>> {
    cx.try_global::<TouchId>()?.0.confirm_owner(reason)
}

/// macOS's LocalAuthentication, biometrics only: the fallback is Kanso's own
/// lock password form, not the account password. On Windows, Windows Hello:
/// a face, a fingerprint or the Windows Hello PIN, never the account
/// password; the fallback is the lock password form there too. Showing the
/// library key ([`Biometrics::confirm_owner`]) is the exception on macOS:
/// Touch ID or the account password, as the system's own sheet asks.
pub struct SystemTouchId;

#[cfg(target_os = "macos")]
mod system {
    use block2::RcBlock;
    use objc2::rc::Retained;
    use objc2::runtime::{AnyObject, Bool};
    use objc2::{class, msg_send};
    use objc2_foundation::NSString;

    #[link(name = "LocalAuthentication", kind = "framework")]
    unsafe extern "C" {}

    /// `LAPolicyDeviceOwnerAuthenticationWithBiometrics`.
    const BIOMETRICS: isize = 1;

    /// `LAPolicyDeviceOwnerAuthentication`: Touch ID, an Apple Watch, or the
    /// account password.
    const OWNER: isize = 2;

    fn can_evaluate(policy: isize) -> bool {
        let context = context();
        // SAFETY: a valid context and policy; a null error pointer is
        // allowed.
        unsafe {
            msg_send![
                &*context,
                canEvaluatePolicy: policy,
                error: std::ptr::null_mut::<*mut AnyObject>()
            ]
        }
    }

    fn evaluate(policy: isize, reason: &str) -> async_channel::Receiver<bool> {
        let (tx, rx) = async_channel::bounded(1);
        let context = context();
        let reason = NSString::from_str(reason);
        // The context stays alive until it answers.
        let kept = context.clone();
        let reply = RcBlock::new(move |success: Bool, _error: *mut AnyObject| {
            let _ = &kept;
            tx.try_send(success.as_bool()).ok();
        });
        // SAFETY: a valid context, policy, reason and reply block of the
        // `void (^)(BOOL, NSError *)` shape LocalAuthentication calls once,
        // on a queue of its own.
        unsafe {
            let _: () = msg_send![
                &*context,
                evaluatePolicy: policy,
                localizedReason: &*reason,
                reply: &*reply
            ];
        }
        rx
    }

    fn context() -> Retained<AnyObject> {
        // SAFETY: `+[LAContext new]` returns a new, retained context.
        unsafe { msg_send![class!(LAContext), new] }
    }

    impl super::Biometrics for super::SystemTouchId {
        fn available(&self) -> bool {
            can_evaluate(BIOMETRICS)
        }

        fn authenticate(&self, reason: &str) -> async_channel::Receiver<bool> {
            evaluate(BIOMETRICS, reason)
        }

        /// The system's own sheet: Touch ID, or the account password where
        /// there is no Touch ID or the person prefers typing it.
        fn confirm_owner(&self, reason: &str) -> Option<async_channel::Receiver<bool>> {
            can_evaluate(OWNER).then(|| evaluate(OWNER, reason))
        }
    }
}

#[cfg(target_os = "windows")]
mod system {
    use std::time::Duration;

    use windows::Security::Credentials::UI::{
        UserConsentVerificationResult, UserConsentVerifier, UserConsentVerifierAvailability,
    };
    use windows::Win32::System::WinRT::IUserConsentVerifierInterop;
    use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;
    use windows::core::{HSTRING, factory};
    use windows_future::IAsyncOperation;

    /// How long asking whether Windows Hello is set up may hold the window.
    const AVAILABILITY_WAIT: Duration = Duration::from_secs(3);

    impl super::Biometrics for super::SystemTouchId {
        fn available(&self) -> bool {
            // Asked on a thread of its own, which joins the apartment
            // WinRT needs; the window waits a moment at most.
            let (tx, rx) = std::sync::mpsc::channel();
            let spawned = std::thread::Builder::new()
                .name("kanso-hello".into())
                .spawn(move || {
                    let available = UserConsentVerifier::CheckAvailabilityAsync()
                        .and_then(|operation| operation.join())
                        .is_ok_and(|answer| answer == UserConsentVerifierAvailability::Available);
                    tx.send(available).ok();
                });
            spawned.is_ok() && rx.recv_timeout(AVAILABILITY_WAIT).unwrap_or(false)
        }

        fn authenticate(&self, reason: &str) -> async_channel::Receiver<bool> {
            let (tx, rx) = async_channel::bounded(1);
            // Kanso's window is in front when someone asks to unlock, so
            // Windows Hello's prompt is put over it.
            // SAFETY: no arguments; a null window is handled below.
            let window = unsafe { GetForegroundWindow() };
            let message = HSTRING::from(reason);
            let request = if window.is_invalid() {
                UserConsentVerifier::RequestVerificationAsync(&message)
            } else {
                factory::<UserConsentVerifier, IUserConsentVerifierInterop>().and_then(|interop| {
                    // SAFETY: a window handle from the system and a message
                    // that outlives the call; the answer is the operation
                    // the interface documents.
                    unsafe {
                        interop.RequestVerificationForWindowAsync::<
                            IAsyncOperation<UserConsentVerificationResult>,
                        >(window, &message)
                    }
                })
            };
            let request = match request {
                Ok(request) => request,
                Err(_) => {
                    tx.try_send(false).ok();
                    return rx;
                }
            };
            // A thread that can't start drops the sender, and the closed
            // channel reads as a cancel.
            let _ = std::thread::Builder::new()
                .name("kanso-hello".into())
                .spawn(move || {
                    let verified = request
                        .join()
                        .is_ok_and(|answer| answer == UserConsentVerificationResult::Verified);
                    tx.try_send(verified).ok();
                });
            rx
        }
    }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
impl Biometrics for SystemTouchId {
    fn available(&self) -> bool {
        false
    }

    fn authenticate(&self, _: &str) -> async_channel::Receiver<bool> {
        let (tx, rx) = async_channel::bounded(1);
        tx.try_send(false).ok();
        rx
    }
}

/// A Touch ID that answers as told, for tests.
#[cfg(test)]
pub struct FakeTouchId {
    pub available: bool,
    pub answer: std::sync::Mutex<bool>,
    pub asked: std::sync::atomic::AtomicUsize,
}

#[cfg(test)]
impl FakeTouchId {
    pub fn new(answer: bool) -> Arc<FakeTouchId> {
        Arc::new(FakeTouchId {
            available: true,
            answer: std::sync::Mutex::new(answer),
            asked: Default::default(),
        })
    }

    pub fn answer(&self, answer: bool) {
        *self.answer.lock().unwrap() = answer;
    }

    pub fn asked(&self) -> usize {
        self.asked.load(std::sync::atomic::Ordering::Relaxed)
    }
}

#[cfg(test)]
impl Biometrics for FakeTouchId {
    fn available(&self) -> bool {
        self.available
    }

    fn authenticate(&self, _: &str) -> async_channel::Receiver<bool> {
        self.asked
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let (tx, rx) = async_channel::bounded(1);
        tx.try_send(*self.answer.lock().unwrap()).ok();
        rx
    }
}

#[cfg(test)]
mod tests {
    use super::{Biometrics as _, SystemTouchId};

    // no-prop-test: the system calls are FFI, asked only on demand below;
    // the names are fixed per platform.

    /// The real LocalAuthentication (or Windows Hello) links and answers
    /// whether it is set up, without asking for a fingerprint. Run on
    /// demand.
    #[test]
    #[ignore = "asks this device's LocalAuthentication or Windows Hello"]
    fn the_system_says_whether_touch_id_is_set_up() {
        println!("{} available: {}", super::NAME, SystemTouchId.available());
    }

    /// Windows says Windows Hello and the Credential Manager; elsewhere it's
    /// Touch ID and the Keychain.
    #[test]
    fn the_unlock_has_the_systems_name() {
        let (name, store) = if cfg!(target_os = "windows") {
            ("Windows Hello", "Credential Manager")
        } else {
            ("Touch ID", "Keychain")
        };
        assert_eq!(super::NAME, name);
        assert_eq!(super::unlock_name!(), name);
        assert_eq!(super::secret_store_name!(), store);
        assert!(
            super::KEYWORDS
                .iter()
                .any(|word| name.eq_ignore_ascii_case(word))
        );
    }
}
