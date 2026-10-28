//! Saving what other apps shared: the C function the share extension, the
//! App Intents and the app itself call, with no Flutter in the process.
//!
//! ```c
//! char *kanso_capture_inbox(const char *library_path, const char *inbox_path);
//! void kanso_string_free(char *text);
//! ```
//!
//! `library_path` is the library's database file (`<library
//! folder>/library.db`) and `inbox_path` the folder of drops beside it
//! (`<library folder>/inbox`), both UTF-8 and NUL-terminated. Every finished
//! drop in it is saved as a note in the Inbox, oldest first, and its folder
//! removed (`mobile/docs/capture.md` says what a drop is, and
//! `kanso_mobile::capture` which are removed and which kept). A library the
//! process has open already (the app's, when an App Intent runs inside it)
//! is the one that is written to, so its lists see the notes.
//!
//! The result is a JSON object, a string the caller frees with
//! `kanso_string_free`:
//!
//! | Field | Type | Meaning |
//! |---|---|---|
//! | `notes` | array of strings | The IDs of the notes made, oldest item first. A note an earlier try already made is here too. |
//! | `problems` | array of strings | What was left out or refused, a sentence each, for the caller to show. Also why saving failed altogether, when it did. |
//! | `kept` | number | Items kept to try again later (a busy database, a disk error, or an item a newer Kanso wrote). |
//!
//! The result is `NULL` when an argument is null or isn't UTF-8. It blocks
//! the calling thread, may be called from any thread, and never unwinds
//! into the caller. On Android the app must have run `KansoNative.init` in
//! this process first; the JNI method `KansoNative.captureInbox` returns the
//! same JSON.

use std::ffi::{CStr, CString, c_char};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;
use std::sync::Arc;

use kanso_mobile::capture::save_inbox;
use kanso_mobile::{Backend, DropsSaved, Secrets, secrets};
use kanso_sync::HttpBackend;

/// Saves the drops in `inbox_path` into the library at `library_path`.
///
/// # Safety
///
/// Each argument must be null or point to a NUL-terminated string that stays
/// valid for the length of the call. The result, when not null, must be
/// passed to [`kanso_string_free`] once and not used after.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kanso_capture_inbox(
    library_path: *const c_char,
    inbox_path: *const c_char,
) -> *mut c_char {
    // SAFETY: the caller promises what this function's contract says.
    unsafe {
        capture_inbox(
            library_path,
            inbox_path,
            secrets::platform(),
            Arc::new(HttpBackend::new()),
        )
    }
}

/// [`kanso_capture_inbox`] with the secret store and the server client
/// given, so tests can pass their own.
///
/// # Safety
///
/// As for [`kanso_capture_inbox`].
pub unsafe fn capture_inbox(
    library_path: *const c_char,
    inbox_path: *const c_char,
    secrets: Arc<dyn Secrets>,
    backend: Arc<dyn Backend>,
) -> *mut c_char {
    let caught = catch_unwind(AssertUnwindSafe(|| {
        if library_path.is_null() || inbox_path.is_null() {
            return std::ptr::null_mut();
        }
        // SAFETY: non-null, and the caller promises they are NUL-terminated
        // strings valid for this call.
        let (library, inbox) =
            unsafe { (CStr::from_ptr(library_path), CStr::from_ptr(inbox_path)) };
        let (Ok(library), Ok(inbox)) = (library.to_str(), inbox.to_str()) else {
            return std::ptr::null_mut();
        };
        let json = saved_json(library, inbox, secrets, backend);
        CString::new(json).map_or(std::ptr::null_mut(), CString::into_raw)
    }));
    caught.unwrap_or(std::ptr::null_mut())
}

/// Frees a string [`kanso_capture_inbox`] returned. Null does nothing.
///
/// # Safety
///
/// `text` must be null or a result of [`kanso_capture_inbox`] not freed yet.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kanso_string_free(text: *mut c_char) {
    if !text.is_null() {
        // SAFETY: made by `CString::into_raw` in `capture_inbox`, and freed
        // once, as the caller promises.
        drop(unsafe { CString::from_raw(text) });
    }
}

/// The JSON for saving `inbox` into `library`, whatever happens: a failure
/// is a problem in it.
pub(crate) fn saved_json(
    library: &str,
    inbox: &str,
    secrets: Arc<dyn Secrets>,
    backend: Arc<dyn Backend>,
) -> String {
    let saved =
        save_inbox(Path::new(library), Path::new(inbox), secrets, backend).unwrap_or_else(|err| {
            DropsSaved {
                problems: vec![format!("Shared items couldn't be saved: {err}")],
                ..DropsSaved::default()
            }
        });
    saved.to_json()
}

/// [`saved_json`] with the phone's secret store and the real server client:
/// what the JNI method calls.
#[cfg(target_os = "android")]
pub(crate) fn saved_json_on_phone(library: &str, inbox: &str) -> String {
    saved_json(
        library,
        inbox,
        secrets::platform(),
        Arc::new(HttpBackend::new()),
    )
}

#[cfg(test)]
mod tests {
    use std::fs;

    use kanso_mobile::MemorySecrets;
    use kanso_sync::MemoryServer;
    use proptest::prelude::*;

    use super::*;

    /// Calls the testable entry and takes the answer back as text, freeing it.
    fn call(library: *const c_char, inbox: *const c_char) -> Option<String> {
        // SAFETY: each argument is null or a NUL-terminated string that
        // outlives the call.
        let result = unsafe {
            capture_inbox(
                library,
                inbox,
                Arc::new(MemorySecrets::new()),
                Arc::new(MemoryServer::new()),
            )
        };
        if result.is_null() {
            return None;
        }
        // SAFETY: not null, and a string `capture_inbox` made.
        let text = unsafe { CStr::from_ptr(result) }
            .to_str()
            .unwrap()
            .to_owned();
        // SAFETY: made by `capture_inbox` and freed once.
        unsafe { kanso_string_free(result) };
        Some(text)
    }

    fn path(path: &Path) -> CString {
        CString::new(path.to_str().unwrap()).unwrap()
    }

    // no-prop-test: the entry's fixed outcomes through the C types.
    #[test]
    fn null_and_non_text_paths_give_null_and_a_drop_is_saved() {
        let dir = tempfile::tempdir().unwrap();
        let library = path(&dir.path().join("library.db"));
        let inbox_dir = dir.path().join("inbox");
        let inbox = path(&inbox_dir);

        assert_eq!(call(std::ptr::null(), inbox.as_ptr()), None);
        assert_eq!(call(library.as_ptr(), std::ptr::null()), None);
        let bad = CString::new(vec![0xFF, 0xFE]).unwrap();
        assert_eq!(call(bad.as_ptr(), inbox.as_ptr()), None);
        assert_eq!(call(library.as_ptr(), bad.as_ptr()), None);
        // SAFETY: null is allowed.
        unsafe { kanso_string_free(std::ptr::null_mut()) };

        // Nothing waiting.
        assert_eq!(
            call(library.as_ptr(), inbox.as_ptr()).unwrap(),
            r#"{"notes":[],"problems":[],"kept":0}"#
        );

        // A real drop: a note and a PDF.
        let id = "ab".repeat(16);
        let folder = inbox_dir.join(&id);
        fs::create_dir_all(&folder).unwrap();
        fs::write(folder.join("0.pdf"), b"%PDF-1.7\n%%EOF\n").unwrap();
        fs::write(
            folder.join("drop.json"),
            r#"{"version":1,"createdAt":1,"text":"From the share sheet","files":[{"path":"0.pdf","name":"Plan.pdf","type":"application/pdf"}]}"#,
        )
        .unwrap();
        let answer = call(library.as_ptr(), inbox.as_ptr()).unwrap();
        assert_eq!(
            answer,
            format!(r#"{{"notes":["note:cap-{id}"],"problems":[],"kept":0}}"#)
        );
        assert!(!folder.exists());
    }

    // no-prop-test: the exported symbols themselves, with the platform's store.
    #[test]
    fn the_exported_entry_answers_for_an_empty_inbox() {
        let dir = tempfile::tempdir().unwrap();
        let library = path(&dir.path().join("library.db"));
        let inbox = path(&dir.path().join("inbox"));
        // SAFETY: NUL-terminated strings that outlive the call; the result is
        // freed once.
        let text = unsafe {
            let result = kanso_capture_inbox(library.as_ptr(), inbox.as_ptr());
            assert!(!result.is_null());
            let text = CStr::from_ptr(result).to_str().unwrap().to_owned();
            kanso_string_free(result);
            text
        };
        assert_eq!(text, r#"{"notes":[],"problems":[],"kept":0}"#);
    }

    proptest! {
        /// Whatever the paths are made of, the entry answers with JSON that
        /// has the three documented fields, and never panics out.
        #[test]
        fn prop_any_paths_get_the_documented_json(name in "[a-zA-Z0-9 ._/-]{1,24}", other in "[a-zA-Z0-9 ._/-]{1,24}") {
            let dir = tempfile::tempdir().unwrap();
            let library = path(&dir.path().join(name.trim_start_matches('/')).join("library.db"));
            let inbox = path(&dir.path().join(other.trim_start_matches('/')));
            let answer = call(library.as_ptr(), inbox.as_ptr()).unwrap();
            prop_assert!(answer.starts_with(r#"{"notes":["#), "{answer}");
            prop_assert!(answer.contains(r#"],"problems":["#) && answer.ends_with("}"), "{answer}");
            prop_assert!(answer.contains(r#"],"kept":"#), "{answer}");
        }
    }
}
