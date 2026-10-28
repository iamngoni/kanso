//! Android's side of the library: the native method the app calls once, as
//! the process starts, and the two the plugins call for the background sync
//! and the passkey sheet.
//!
//! # The contract with Kotlin
//!
//! ```kotlin
//! package app.kanso.kanso
//!
//! object KansoNative {
//!     /** Hands the library the app's Context. Idempotent. */
//!     @JvmStatic external fun init(context: android.content.Context)
//!
//!     /** Saves the shared items waiting in `inboxPath` into the library at
//!      *  `libraryPath`; the JSON of `crate::capture`, or null for a null
//!      *  argument. Blocks. */
//!     @JvmStatic external fun captureInbox(libraryPath: String, inboxPath: String): String?
//!     /** The sync that runs with no app; the codes of `background.rs`. */
//!     @JvmStatic external fun backgroundSync(path: String, budgetMs: Int): Int
//!
//!     /** A passkey sheet's answer; the codes of `passkey.rs`. */
//!     @JvmStatic external fun passkeyAnswer(ticket: Long, json: String): Int
//! }
//! ```
//!
//! Static native methods of the class `app.kanso.kanso.KansoNative`, found by
//! the JVM under the exported names `Java_app_kanso_kanso_KansoNative_init`,
//! `..._backgroundSync` and `..._passkeyAnswer`. The last two never throw and
//! never unwind into the JVM: a path or text the JVM can't give, a panic, or a
//! budget below zero is answered with `-5` (the budget with `0`). Neither
//! logs its arguments.
//!
//! `init` is the one the app calls from `Application.onCreate`. The app calls it from
//! `Application.onCreate`, after `System.loadLibrary("kanso_mobile_bridge")`,
//! with `applicationContext`. That is the only place it can be done in the
//! background too: WorkManager runs jobs in the app's process, after
//! `Application.onCreate`.
//!
//! It does two things, once:
//!
//! 1. Looks up the Kotlin class `app.kanso.kanso.KansoSecrets` (see
//!    `kanso_mobile::secrets::android` for its three static methods) on this
//!    thread, where the app's class loader finds it, and keeps a global
//!    reference for every other thread.
//! 2. Gives the system certificate verifier the `Context`
//!    (`kanso_tls::init_android`), without which no request over TLS is
//!    made.
//!
//! `JNI_OnLoad` is not used: Dart loads the library with `dlopen`, which
//! does not call it, and it has no `Context` in any case. Both Kotlin
//! classes must be kept by shrinking (`proguard-rules.pro`), along with the
//! verifier's own (`org.rustls.platformverifier.**`).
//!
//! A failure throws a `RuntimeException` into Kotlin, with the reason.
//!
//! `captureInbox` is `crate::capture`'s `kanso_capture_inbox` for Kotlin: it
//! needs `init` to have run first, and returns its JSON as a string. A panic
//! in it is caught and returned as a problem in the JSON.

use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};

use std::ffi::CString;
use std::panic::{AssertUnwindSafe, catch_unwind};

use jni::objects::{JByteArray, JClass, JObject, JString};
use jni::sys::{jint, jlong};
use jni::{Env, EnvUnowned};
use zeroize::Zeroize;

/// Set once the verifier and the Keystore bridge are set up.
static DONE: AtomicBool = AtomicBool::new(false);

/// Why `init` failed, in words for the exception.
#[derive(Debug)]
struct InitError(String);

impl fmt::Display for InitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for InitError {}

impl From<jni::errors::Error> for InitError {
    fn from(err: jni::errors::Error) -> InitError {
        InitError(err.to_string())
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_app_kanso_kanso_KansoNative_init<'caller>(
    mut unowned: EnvUnowned<'caller>,
    _class: JClass<'caller>,
    context: JObject<'caller>,
) {
    unowned
        .with_env(|env| setup(env, context))
        .resolve::<jni::errors::ThrowRuntimeExAndDefault>()
}

fn setup(env: &mut Env, context: JObject) -> Result<(), InitError> {
    if DONE.load(Ordering::Acquire) {
        return Ok(());
    }
    kanso_mobile::secrets::android::install(env).map_err(|err| InitError(err.0))?;
    kanso_tls::init_android(env, context).map_err(|err| InitError(err.to_string()))?;
    DONE.store(true, Ordering::Release);
    Ok(())
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_app_kanso_kanso_KansoNative_captureInbox<'caller>(
    mut unowned: EnvUnowned<'caller>,
    _class: JClass<'caller>,
    library_path: JString<'caller>,
    inbox_path: JString<'caller>,
) -> JString<'caller> {
    unowned
        .with_env(|env| capture(env, &library_path, &inbox_path))
        .resolve::<jni::errors::ThrowRuntimeExAndDefault>()
}

/// The JSON of saving `inbox_path` into `library_path`, or null for a null
/// argument.
fn capture<'local>(
    env: &mut Env<'local>,
    library_path: &JString,
    inbox_path: &JString,
) -> Result<JString<'local>, InitError> {
    if library_path.is_null() || inbox_path.is_null() {
        return Ok(JString::null());
    }
    let library = library_path.try_to_string(env)?;
    let inbox = inbox_path.try_to_string(env)?;
    // A panic must not unwind into the JVM.
    let json = std::panic::catch_unwind(|| crate::capture::saved_json_on_phone(&library, &inbox))
        .unwrap_or_else(|_| {
            r#"{"notes":[],"problems":["Shared items couldn't be saved: an internal error."],"kept":0}"#
                .to_owned()
        });
    Ok(JString::from_str(env, json)?)
}

/// The sync with no app, for WorkManager (`background::kanso_background_sync`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_app_kanso_kanso_KansoNative_backgroundSync<'caller>(
    mut unowned: EnvUnowned<'caller>,
    _class: JClass<'caller>,
    path: JString<'caller>,
    budget_ms: jint,
) -> jint {
    unowned
        .with_env(|env| -> Result<jint, jni::errors::Error> {
            let caught = catch_unwind(AssertUnwindSafe(|| {
                let Ok(path) = path.try_to_string(env) else {
                    return crate::passkey::INTERNAL;
                };
                let Ok(path) = CString::new(path) else {
                    return crate::passkey::INTERNAL;
                };
                let budget = u32::try_from(budget_ms).unwrap_or(0);
                // SAFETY: a NUL-terminated string that outlives the call.
                unsafe { crate::background::kanso_background_sync(path.as_ptr(), budget) }
            }));
            Ok(caught.unwrap_or(crate::passkey::INTERNAL))
        })
        .resolve::<jni::errors::LogErrorAndDefault>()
}

/// A passkey sheet's answer (`passkey::deliver`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_app_kanso_kanso_KansoNative_passkeyAnswer<'caller>(
    mut unowned: EnvUnowned<'caller>,
    _class: JClass<'caller>,
    ticket: jlong,
    json: JString<'caller>,
) -> jint {
    unowned
        .with_env(|env| -> Result<jint, jni::errors::Error> {
            let caught = catch_unwind(AssertUnwindSafe(|| {
                let Ok(ticket) = u64::try_from(ticket) else {
                    return crate::passkey::INTERNAL;
                };
                let Ok(json) = json.try_to_string(env) else {
                    return crate::passkey::INTERNAL;
                };
                crate::passkey::deliver(ticket, &json)
            }));
            Ok(caught.unwrap_or(crate::passkey::INTERNAL))
        })
        .resolve::<jni::errors::LogErrorAndDefault>()
}
