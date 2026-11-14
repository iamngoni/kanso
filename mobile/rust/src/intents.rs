//! What iOS's App Intents call, with no Flutter in the process (#390):
//!
//! ```c
//! char *kanso_task_preview(const char *text, int32_t year, uint32_t month,
//!                          uint32_t day, uint32_t hour, uint32_t minute,
//!                          bool month_first);
//! char *kanso_intent_notes(const char *library_path, const char *query,
//!                          uint32_t limit);
//! ```
//!
//! `kanso_task_preview` reads a typed task the way the desktop's Quick
//! Capture and Raycast do, at the phone's local time, and returns
//! `{"line": …, "words": …, "when": … or null, "confirmation": "Added: …"}`,
//! or `null` when there is no task in the text. "Add Kanso task" writes the
//! `line` into a `task` drop (`mobile/docs/capture.md`).
//!
//! `kanso_intent_notes` returns the notes "Open Kanso note" may offer, newest
//! edit first, every word of `query` in the title:
//! `{"notes": [{"id": …, "title": …, "notebook": …}]}`. Never a locked note
//! or a note in a locked notebook (`kanso_mobile::intents`). A library the
//! process has open already is the one read.
//!
//! Both return a string the caller frees with `kanso_string_free`, or `NULL`
//! when a string argument is null or isn't UTF-8 (or the library couldn't be
//! read, for the notes). Neither unwinds into the caller.

use std::ffi::{CStr, CString, c_char};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::Path;
use std::sync::Arc;

use kanso_mobile::intents::{intent_notes_at, intent_notes_json, task_preview_json};
use kanso_mobile::{Backend, LocalTime, Secrets, secrets};
use kanso_sync::HttpBackend;

/// `text` as a `&str`, when it is a non-null UTF-8 C string.
///
/// # Safety
///
/// `text` must be null or point to a NUL-terminated string valid for `'a`.
unsafe fn utf8<'a>(text: *const c_char) -> Option<&'a str> {
    if text.is_null() {
        return None;
    }
    // SAFETY: non-null, and the caller promises it is NUL-terminated and
    // valid for 'a.
    unsafe { CStr::from_ptr(text) }.to_str().ok()
}

/// The result string handed to C, or null.
fn out(json: Option<String>) -> *mut c_char {
    json.and_then(|json| CString::new(json).ok())
        .map_or(std::ptr::null_mut(), CString::into_raw)
}

/// Reads the task in `text` at the given local time.
///
/// # Safety
///
/// `text` must be null or point to a NUL-terminated string that stays valid
/// for the length of the call. The result, when not null, must be passed to
/// `kanso_string_free` once and not used after.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kanso_task_preview(
    text: *const c_char,
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    month_first: bool,
) -> *mut c_char {
    let caught = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: as this function's contract says.
        let text = unsafe { utf8(text) }?;
        let now = LocalTime {
            year,
            month,
            day,
            hour,
            minute,
            second: 0,
        };
        Some(task_preview_json(text, now, month_first))
    }));
    out(caught.ok().flatten())
}

/// The notes "Open Kanso note" may offer for `query`, at most `limit`.
///
/// # Safety
///
/// Each argument must be null or point to a NUL-terminated string that stays
/// valid for the length of the call. The result, when not null, must be
/// passed to `kanso_string_free` once and not used after.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kanso_intent_notes(
    library_path: *const c_char,
    query: *const c_char,
    limit: u32,
) -> *mut c_char {
    // SAFETY: as this function's contract says.
    unsafe {
        intent_notes(
            library_path,
            query,
            limit,
            secrets::platform(),
            Arc::new(HttpBackend::new()),
        )
    }
}

/// [`kanso_intent_notes`] with the secret store and the server client given,
/// so tests can pass their own.
///
/// # Safety
///
/// As for [`kanso_intent_notes`].
pub unsafe fn intent_notes(
    library_path: *const c_char,
    query: *const c_char,
    limit: u32,
    secrets: Arc<dyn Secrets>,
    backend: Arc<dyn Backend>,
) -> *mut c_char {
    let caught = catch_unwind(AssertUnwindSafe(|| {
        // SAFETY: as this function's contract says.
        let (library, query) = unsafe { (utf8(library_path)?, utf8(query)?) };
        let notes =
            intent_notes_at(Path::new(library), query, limit as usize, secrets, backend).ok()?;
        Some(intent_notes_json(&notes))
    }));
    out(caught.ok().flatten())
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;

    use super::*;
    use crate::capture::kanso_string_free;

    fn read(text: *mut c_char) -> Option<String> {
        if text.is_null() {
            return None;
        }
        // SAFETY: made by `out` above, freed once here.
        let owned = unsafe { CStr::from_ptr(text) }.to_str().unwrap().to_owned();
        unsafe { kanso_string_free(text) };
        Some(owned)
    }

    #[test]
    fn previews_through_c() {
        let text = CString::new("film Pixel review fri 3pm every week").unwrap();
        // SAFETY: a valid C string for the call.
        let json =
            read(unsafe { kanso_task_preview(text.as_ptr(), 2026, 11, 12, 9, 41, false) }).unwrap();
        assert!(
            json.contains(
                r#""confirmation":"Added: film Pixel review · Fri 13 Nov 15:00 · every week""#
            ),
            "{json}"
        );
        assert!(
            json.contains(r#""line":"- [ ] film Pixel review @2026-11-13T15:00 every week""#),
            "{json}"
        );
        // SAFETY: null is allowed.
        assert_eq!(
            read(unsafe { kanso_task_preview(std::ptr::null(), 2026, 11, 12, 9, 41, false) }),
            None
        );
        let blank = CString::new("  ").unwrap();
        // SAFETY: a valid C string for the call.
        assert_eq!(
            read(unsafe { kanso_task_preview(blank.as_ptr(), 2026, 11, 12, 9, 41, false) })
                .as_deref(),
            Some("null")
        );
    }

    #[test]
    fn notes_through_c_need_text_arguments() {
        let query = CString::new("").unwrap();
        // SAFETY: null is allowed.
        assert_eq!(
            read(unsafe { kanso_intent_notes(std::ptr::null(), query.as_ptr(), 5) }),
            None
        );
        let dir = tempfile::tempdir().unwrap();
        let library =
            CString::new(dir.path().join("library.db").to_str().unwrap().to_owned()).unwrap();
        // SAFETY: valid C strings for the call.
        assert_eq!(
            read(unsafe {
                intent_notes(
                    library.as_ptr(),
                    query.as_ptr(),
                    5,
                    Arc::new(kanso_mobile::MemorySecrets::new()),
                    Arc::new(kanso_sync::MemoryServer::new()),
                )
            })
            .as_deref(),
            Some(r#"{"notes":[]}"#)
        );
    }

    proptest::proptest! {
        /// Whatever is typed, at any valid or invalid time, the entry answers
        /// with `null` or an object that has a line and a confirmation, and
        /// never panics out.
        #[test]
        fn prop_any_text_gets_null_or_a_task(
            text in "[a-z0-9 :/\\[\\]-]{0,40}",
            day in 0u32..33,
            hour in 0u32..26,
        ) {
            let text = CString::new(text).unwrap();
            // SAFETY: a valid C string for the call.
            let json = read(unsafe { kanso_task_preview(text.as_ptr(), 2026, 11, day, hour, 0, false) })
                .unwrap();
            proptest::prop_assert!(
                json == "null"
                    || (json.contains(r#""line":"- [ ] "#) && json.contains(r#""confirmation":"Added: "#)),
                "{json}"
            );
        }
    }
}
