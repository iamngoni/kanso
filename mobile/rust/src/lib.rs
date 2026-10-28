//! The phones' native library (#48): `kanso-mobile` behind
//! [flutter_rust_bridge](https://github.com/fzyzcjy/flutter_rust_bridge), and
//! the entry points the system's background jobs call with no Flutter at all.
//!
//! What is here, and why it is here and not in `kanso-mobile`:
//!
//! - [`api`] is the surface Dart sees. The bridge generator reads it
//!   (`mobile/flutter_rust_bridge.yaml`) and writes `frb_generated.rs` and the
//!   Dart under `mobile/lib/core/bridge`. It only wraps `kanso-mobile`: no
//!   decision is made here.
//! - [`background`] defines `kanso_background_sync`, the C function the iOS
//!   `BGAppRefreshTask` and the Android `WorkManager` job call. A
//!   `#[no_mangle]` function in a dependency is not reliably exported from a
//!   cdylib or staticlib (rust-lang/rust#47384), so it is defined in the
//!   crate that is built into the library.
//! - [`capture`] defines `kanso_capture_inbox` and `kanso_string_free`, the C
//!   functions the share extension, the App Intents and the app call to save
//!   what other apps shared, for the same reason.
//! - [`biometric`] defines `kanso_biometric_answer`, the C function the
//!   native lock plugin hands a biometric check's answer to.
//! - [`passkey`] defines `kanso_passkey_answer`, the C function the passkey
//!   plugin calls with the platform sheet's answer, so the PRF output goes
//!   from the plugin to Rust and never through Dart.
//! - `android` (Android only) defines `Java_app_kanso_kanso_KansoNative_init`,
//!   which hands the app's `Context` to the certificate verifier and the
//!   Keystore bridge, and the JNI forms of the background sync and the two
//!   answers above.
//!
//! The library is built in three crate types: `staticlib` (iOS links it),
//! `cdylib` (Android loads it; `flutter test` loads it on a computer) and
//! `rlib` (this crate's tests).

pub mod api;
pub mod background;
pub mod biometric;
pub mod capture;
pub mod passkey;

#[cfg(target_os = "android")]
mod android;

// The bridge generator's output (`frb_generated.rs`). Generated code isn't
// formatted or linted by us; `mobile/flutter_rust_bridge.yaml` sets
// `add_mod_to_lib: false` so the generator leaves this declaration alone.
#[rustfmt::skip]
#[allow(clippy::all, unused, unreachable_pub, unsafe_op_in_unsafe_fn, non_snake_case)]
mod frb_generated;
