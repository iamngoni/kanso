//! The sync that runs with no app: the C function the system's background
//! jobs call.
//!
//! ```c
//! int32_t kanso_background_sync(const char *library_path, uint32_t budget_ms);
//! ```
//!
//! `library_path` is the library's database file (the path the app opens,
//! `<application support>/kanso/library.db`), UTF-8 and NUL-terminated.
//! `budget_ms` is the time the job may take; the sync stops between steps when
//! it is spent. The result:
//!
//! | Result | Meaning |
//! |---:|---|
//! | `0` | Done, and nothing changed. |
//! | `1` | Done, and something changed (pulled or pushed). |
//! | `-1` | No account is signed in. |
//! | `-2` | The secret store (Keychain, Keystore) refused. |
//! | `-3` | The server couldn't be reached, or answered with a problem. |
//! | `-4` | The budget ran out first. What was done is kept. |
//! | `-5` | Internal: a panic was caught, the path isn't text, or the library couldn't be opened. |
//!
//! It blocks the calling thread, may be called from any thread, and never
//! unwinds into the caller. On Android the app must have run
//! `KansoNative.init` in this process first (`KansoApplication.onCreate`
//! does, and a WorkManager job runs in the app's process, after it).

use std::ffi::{CStr, c_char};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use kanso_mobile::background::{BackgroundOutcome, run};
use kanso_mobile::{Backend, Secrets, secrets};
use kanso_sync::HttpBackend;

/// The result for anything that goes wrong outside the sync itself.
const INTERNAL: i32 = -5;

/// Syncs the library at `library_path` once, within `budget_ms`.
///
/// # Safety
///
/// `library_path` must be null or point to a NUL-terminated string that stays
/// valid for the length of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kanso_background_sync(library_path: *const c_char, budget_ms: u32) -> i32 {
    // SAFETY: the caller promises what this function's contract says.
    unsafe {
        background_sync(
            library_path,
            budget_ms,
            Arc::new(HttpBackend::new()),
            secrets::platform(),
        )
    }
}

/// Closes the database connections of the library at `library_path`, if this
/// process has it open, so the app holds no SQLite lock on a file in the App
/// Group's container when iOS suspends it (`0xdead10cc`). The next use of the
/// library opens them again. Returns 0, or -5 for a path that isn't text.
///
/// ```c
/// int32_t kanso_library_release(const char *library_path);
/// ```
///
/// # Safety
///
/// As for [`kanso_background_sync`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kanso_library_release(library_path: *const c_char) -> i32 {
    let caught = catch_unwind(AssertUnwindSafe(|| {
        if library_path.is_null() {
            return INTERNAL;
        }
        // SAFETY: non-null, and the caller promises it is a NUL-terminated
        // string valid for this call.
        let text = unsafe { CStr::from_ptr(library_path) };
        let Ok(path) = text.to_str() else {
            return INTERNAL;
        };
        kanso_mobile::Library::release_for_suspend(Path::new(path));
        0
    }));
    caught.unwrap_or(INTERNAL)
}

/// [`kanso_background_sync`] with the server client and the secret store
/// given, so tests can pass their own.
///
/// # Safety
///
/// As for [`kanso_background_sync`].
pub unsafe fn background_sync(
    library_path: *const c_char,
    budget_ms: u32,
    backend: Arc<dyn Backend>,
    secrets: Arc<dyn Secrets>,
) -> i32 {
    let caught = catch_unwind(AssertUnwindSafe(|| {
        if library_path.is_null() {
            return INTERNAL;
        }
        // SAFETY: non-null, and the caller promises it is a NUL-terminated
        // string valid for this call.
        let text = unsafe { CStr::from_ptr(library_path) };
        let Ok(path) = text.to_str() else {
            return INTERNAL;
        };
        let budget = Duration::from_millis(u64::from(budget_ms));
        let outcome: BackgroundOutcome = run(Path::new(path), budget, backend, secrets);
        outcome.code()
    }));
    caught.unwrap_or(INTERNAL)
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;
    use std::sync::Arc;

    use kanso_mobile::{Library, MemorySecrets, NewNote};
    use kanso_sync::{DEFAULT_SERVER, DeviceIdentity, MemoryServer, Password};
    use proptest::prelude::*;

    use super::*;

    const ACCOUNT_PASSWORD: &str = "correct horse battery";

    fn call(
        path: &std::path::Path,
        budget_ms: u32,
        server: &Arc<MemoryServer>,
        secrets: &Arc<MemorySecrets>,
    ) -> i32 {
        let path = CString::new(path.to_str().unwrap()).unwrap();
        // SAFETY: `path` is a NUL-terminated string that outlives the call.
        unsafe { background_sync(path.as_ptr(), budget_ms, server.clone(), secrets.clone()) }
    }

    /// A library signed in to account "ada" on `server`.
    fn phone(
        server: &Arc<MemoryServer>,
    ) -> (tempfile::TempDir, std::path::PathBuf, Arc<MemorySecrets>) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("library.db");
        let secrets = Arc::new(MemorySecrets::new());
        let library = Library::open_for_test(&path, secrets.clone(), server.clone()).unwrap();
        library
            .sync()
            .sign_in_password(
                DEFAULT_SERVER.into(),
                "ada".into(),
                ACCOUNT_PASSWORD.into(),
                "Phone".into(),
            )
            .unwrap();
        drop(library);
        (dir, path, secrets)
    }

    // no-prop-test: the entry's four fixed outcomes through the C types.
    #[test]
    fn signed_out_signed_in_out_of_time_and_a_bad_path() {
        let server = Arc::new(MemoryServer::new());
        let device =
            DeviceIdentity::from_parts("l_ffffffffffffffffffffffffffffffff", "Setup", [9; 32]);
        let password = Password::new(ACCOUNT_PASSWORD, "ada").unwrap();
        kanso_sync::sign_up(
            &*server,
            &MemorySecrets::new(),
            DEFAULT_SERVER,
            "ada",
            &password,
            &device,
            &|_| {},
        )
        .unwrap();

        // Signed out: -1, in a process with no Dart.
        let dir = tempfile::tempdir().unwrap();
        let empty = Arc::new(MemorySecrets::new());
        assert_eq!(
            call(&dir.path().join("library.db"), 1_000, &server, &empty),
            -1
        );

        // Another phone writes; this one, closed, pulls it.
        let (_writer_dir, writer_path, writer_secrets) = phone(&server);
        let writer = Library::open_for_test(&writer_path, writer_secrets, server.clone()).unwrap();
        writer
            .create(NewNote {
                title: "From elsewhere".into(),
                ..NewNote::default()
            })
            .unwrap();
        writer.sync().sync_now().unwrap();
        let (_dir, path, secrets) = phone(&server);
        assert_eq!(call(&path, 30_000, &server, &secrets), 1);
        assert_eq!(call(&path, 30_000, &server, &secrets), 0);
        assert_eq!(call(&path, 0, &server, &secrets), -4);

        // A null path, and text that isn't UTF-8.
        // SAFETY: null is allowed, and the bytes are NUL-terminated.
        unsafe {
            assert_eq!(
                background_sync(std::ptr::null(), 10, server.clone(), secrets.clone()),
                -5
            );
            let bad = CString::new(vec![0xFF, 0xFE]).unwrap();
            assert_eq!(
                background_sync(bad.as_ptr(), 10, server.clone(), secrets.clone()),
                -5
            );
        }
    }

    // no-prop-test: the exported symbol itself, with the platform's store.
    #[test]
    fn the_exported_entry_is_signed_out_on_a_new_library() {
        let dir = tempfile::tempdir().unwrap();
        let path = CString::new(dir.path().join("library.db").to_str().unwrap()).unwrap();
        // SAFETY: a NUL-terminated string that outlives the call.
        let result = unsafe { kanso_background_sync(path.as_ptr(), 1_000) };
        assert_eq!(result, -1);
    }

    proptest! {
        /// Whatever the path is made of, the entry answers with one of its
        /// documented numbers and never panics out.
        #[test]
        fn prop_any_path_gets_a_documented_number(name in "[a-zA-Z0-9 ._/-]{1,24}", budget in 0u32..5) {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join(name.trim_start_matches('/')).join("library.db");
            let path = CString::new(path.to_str().unwrap()).unwrap();
            let secrets = Arc::new(MemorySecrets::new());
            let server = Arc::new(MemoryServer::new());
            // SAFETY: a NUL-terminated string that outlives the call.
            let result = unsafe { background_sync(path.as_ptr(), budget, server, secrets) };
            prop_assert!((-5..=1).contains(&result), "{result}");
        }
    }
}
