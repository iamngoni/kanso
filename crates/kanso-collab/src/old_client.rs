//! A Kanso too old for shared notebooks (issue #390, slice 41).
//!
//! The Worker checks the `x-kanso-book-protocol` header on every notebook
//! route and answers a client below its minimum with `426 client_too_old`.
//! That answer says how to fix it; this module keeps the words (the same
//! ones the server and the apps use) and remembers, for this run, that the
//! server said so, which is what lets the apps show the update notice.

use std::sync::atomic::{AtomicBool, Ordering};

/// The error code of that answer.
pub const CLIENT_TOO_OLD: &str = "client_too_old";

/// What a refused request is told: what to do, not only what went wrong.
pub const UPDATE_FOR_BOOKS: &str = "Update Kanso to use shared notebooks. This version is too old to open them: choose Settings \u{203a} Updates, then try again.";

/// The line an invitation shows when this Kanso can't open it.
pub const UPDATE_FOR_INVITATION: &str = "Update Kanso to open this shared notebook.";

static REFUSED: AtomicBool = AtomicBool::new(false);

/// Remembers that the server refused this Kanso as too old.
pub(crate) fn note_refusal() {
    REFUSED.store(true, Ordering::Relaxed);
}

/// Whether the server has refused this Kanso as too old since it started.
/// The apps read it when they build the update notice's context.
pub fn client_refused() -> bool {
    REFUSED.load(Ordering::Relaxed)
}

/// Whether an answer is that refusal.
pub fn is_refusal(status: u16, code: &str) -> bool {
    status == 426 && code == CLIENT_TOO_OLD
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// The refusal is one exact answer: any other status, or any other
        /// code, is an ordinary error and never asks for an update.
        #[test]
        fn the_refusal_is_exactly_426_client_too_old(status in 100u16..600, code in "[a-z_]{0,20}") {
            prop_assert_eq!(
                is_refusal(status, &code),
                status == 426 && code == CLIENT_TOO_OLD
            );
            prop_assert!(is_refusal(426, CLIENT_TOO_OLD));
        }
    }

    #[test]
    fn only_a_426_with_the_code_is_the_refusal() {
        assert!(is_refusal(426, CLIENT_TOO_OLD));
        // A plain WebSocket upgrade demand is not.
        assert!(!is_refusal(426, "upgrade_required"));
        assert!(!is_refusal(403, CLIENT_TOO_OLD));
    }

    #[test]
    fn the_words_say_how_to_fix_it() {
        assert!(UPDATE_FOR_BOOKS.starts_with("Update Kanso to use shared notebooks"));
        assert!(UPDATE_FOR_BOOKS.contains("Settings"));
        assert_eq!(
            UPDATE_FOR_INVITATION,
            "Update Kanso to open this shared notebook."
        );
    }
}
