//! The answer of the platform's biometric check: the C function the native
//! lock plugin calls.
//!
//! ```c
//! int32_t kanso_biometric_answer(uint64_t ticket, int32_t status,
//!                                const uint8_t *secret, size_t len);
//! ```
//!
//! `ticket` is the number `begin_biometric_enrollment` or
//! `begin_biometric_unlock` gave. `status` is `0` when the check passed
//! (then `secret` is the 32 bytes of the platform's secret), `1` cancelled,
//! `2` invalidated (the biometrics changed or were removed), `3`
//! unavailable, `4` failed. The result:
//!
//! | Result | Meaning |
//! |---:|---|
//! | `1` | Applied. |
//! | `0` | No such ticket: it was answered, expired (5 minutes) or never given. |
//! | `-1` | The secret didn't open the kept keys. They are forgotten. |
//! | `-5` | A fault: the secret store refused, or a panic was caught. |
//!
//! A null `secret`, or a length other than 32, with status `0` counts as a
//! failed check (status `4`). The caller zeroes its copy of the secret after
//! the call; the library zeroes its own. It may be called from any thread
//! and never unwinds into the caller. `mobile/docs/locked-notes.md` has the
//! whole contract.

use std::panic::{AssertUnwindSafe, catch_unwind};

use kanso_mobile::biometrics::{self, answer, status};

/// Hands the answer for `ticket` to the library.
///
/// # Safety
///
/// `secret` must be null, or point to `len` readable bytes for the length of
/// the call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kanso_biometric_answer(
    ticket: u64,
    status_code: i32,
    secret: *const u8,
    len: usize,
) -> i32 {
    // SAFETY: the caller promises what this function's contract says.
    let bytes = (!secret.is_null()).then(|| unsafe { std::slice::from_raw_parts(secret, len) });
    answer_with(ticket, status_code, bytes)
}

/// [`kanso_biometric_answer`] on a slice: what the JNI entry calls.
pub fn answer_with(ticket: u64, status_code: i32, secret: Option<&[u8]>) -> i32 {
    catch_unwind(AssertUnwindSafe(|| {
        // Not 32 bytes of secret: a check that failed.
        let (status_code, secret) = match (status_code, secret) {
            (status::OK, Some(bytes)) if bytes.len() == 32 => (status_code, Some(bytes)),
            (status::OK, _) => (status::FAILED, None),
            (other, _) => (other, None),
        };
        biometrics::deliver(ticket, status_code, secret)
    }))
    .unwrap_or(answer::FAULT)
}
