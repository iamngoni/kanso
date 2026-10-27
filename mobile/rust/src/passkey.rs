//! Where a passkey ceremony's answer comes back from the phone's own sheet.
//!
//! The native plugin (`kanso_passkeys`) runs the platform's passkey sheet and
//! hands what it produced straight here, so the PRF output is never in Dart.
//!
//! ```c
//! int32_t kanso_passkey_answer(uint64_t ticket, const char *json);
//! ```
//!
//! `ticket` is the number the ceremony gave the plugin in its request; `json`
//! is the answer, UTF-8 and NUL-terminated (the shapes are in
//! `kanso_mobile::passkey_bridge`). The result:
//!
//! | Result | Meaning |
//! |---:|---|
//! | `1` | Accepted: a ceremony was waiting for `ticket`. |
//! | `0` | Nobody waits for `ticket` (unknown, finished or timed out). |
//! | `-5` | Internal: a panic was caught, or `json` is null or not UTF-8. |
//!
//! On Android the same function is `KansoNative.passkeyAnswer(ticket: Long,
//! json: String): Int` (see `android.rs`).
//!
//! The answer is never logged, in any outcome: it holds the PRF output.

use std::ffi::{CStr, c_char};
use std::panic::{AssertUnwindSafe, catch_unwind};

/// The result for anything that goes wrong outside the ceremony itself.
pub(crate) const INTERNAL: i32 = -5;

/// Hands `json` to the ceremony waiting on `ticket`, or says there is none.
///
/// # Safety
///
/// `json` must be null or point to a NUL-terminated string that stays valid
/// for the length of the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kanso_passkey_answer(ticket: u64, json: *const c_char) -> i32 {
    let caught = catch_unwind(AssertUnwindSafe(|| {
        if json.is_null() {
            return INTERNAL;
        }
        // SAFETY: non-null, and the caller promises it is a NUL-terminated
        // string valid for this call.
        let text = unsafe { CStr::from_ptr(json) };
        let Ok(text) = text.to_str() else {
            return INTERNAL;
        };
        deliver(ticket, text)
    }));
    caught.unwrap_or(INTERNAL)
}

/// The answer to the ceremony: 1 when one was waiting, else 0.
pub(crate) fn deliver(ticket: u64, json: &str) -> i32 {
    i32::from(kanso_mobile::passkey_bridge::answer(ticket, json))
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;

    use proptest::prelude::*;

    use super::*;

    fn call(ticket: u64, json: &CString) -> i32 {
        // SAFETY: a NUL-terminated string that outlives the call.
        unsafe { kanso_passkey_answer(ticket, json.as_ptr()) }
    }

    // no-prop-test: the three fixed refusals through the C types.
    #[test]
    fn null_and_bad_text_are_internal_and_the_rest_is_a_ticket_question() {
        // SAFETY: null is allowed.
        assert_eq!(unsafe { kanso_passkey_answer(1, std::ptr::null()) }, -5);
        let bad = CString::new(vec![0xFF, 0xFE]).unwrap();
        assert_eq!(call(1, &bad), -5);
        let fine = CString::new(r#"{"error":"cancelled"}"#).unwrap();
        assert_eq!(call(1, &fine), 0);
    }

    proptest! {
        /// Whatever the ticket and text, the entry answers 0 or -5 for a
        /// ticket nobody waits on, and never panics out.
        #[test]
        fn prop_an_unknown_ticket_is_never_accepted(ticket in any::<u64>(), json in "[ -~]{0,64}") {
            let json = CString::new(json).unwrap();
            let result = call(ticket, &json);
            prop_assert!(result == 0 || result == INTERNAL, "{result}");
        }

        /// Bytes that aren't UTF-8 are refused as internal.
        #[test]
        fn prop_text_that_is_not_utf8_is_internal(ticket in any::<u64>(), tail in proptest::collection::vec(1u8..=255, 0..16)) {
            let mut bytes = vec![0xFF];
            bytes.extend(tail);
            let json = CString::new(bytes).unwrap();
            prop_assert_eq!(call(ticket, &json), INTERNAL);
        }
    }
}
